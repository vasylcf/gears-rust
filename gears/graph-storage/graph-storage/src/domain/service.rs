//! Domain services.
//!
//! Phase 2 carries a single read-only service backed by the secure ORM, so the
//! storage and authorization wiring can be exercised end to end. Ingest,
//! search, traversal and analytics land here as their layers are implemented.

use std::sync::Arc;

use graph_storage_sdk::{EdgeInput, GraphStats, IngestResult, NodeInput};
use toolkit_db::{DBProvider, DbError};
use toolkit_security::{AccessScope, SecurityContext};

use crate::config::GraphStorageConfig;
use crate::domain::error::DomainError;
use crate::infra::storage::{counts, ingest_repo, traversal};

/// Composition of all domain services used by the gear.
pub struct GraphServices {
    config: GraphStorageConfig,
    db: Arc<DBProvider<DbError>>,
}

impl GraphServices {
    /// Build the service composition from validated configuration.
    #[must_use]
    pub fn new(config: GraphStorageConfig, db: Arc<DBProvider<DbError>>) -> Self {
        Self { config, db }
    }

    /// Effective gear configuration.
    #[must_use]
    pub fn config(&self) -> &GraphStorageConfig {
        &self.config
    }

    /// Coarse counters for the caller's graph.
    ///
    /// The scope is derived from the caller's tenant. A PDP-issued scope
    /// replaces this once the policy-enforcement layer lands; the call site
    /// does not change, because the repository already takes an `AccessScope`.
    ///
    /// # Errors
    /// Returns [`DomainError::Storage`] when the query fails.
    pub async fn stats(&self, ctx: &SecurityContext) -> Result<GraphStats, DomainError> {
        let scope = AccessScope::for_tenant(ctx.subject_tenant_id());
        let conn = self
            .db
            .conn()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        counts::graph_stats(&conn, &scope).await
    }

    /// Register a GTS type for the caller's tenant, returning its interned id.
    ///
    /// # Errors
    /// Returns [`DomainError::Storage`] when the write fails.
    pub async fn register_type(
        &self,
        ctx: &SecurityContext,
        type_id: &str,
        kind: &str,
    ) -> Result<i32, DomainError> {
        let tenant = ctx.subject_tenant_id();
        let scope = AccessScope::for_tenant(tenant);
        let conn = self
            .db
            .conn()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        ingest_repo::upsert_type(&conn, &scope, tenant, type_id, kind).await
    }

    /// Upsert a batch of nodes and edges.
    ///
    /// Validation runs before any write: every referenced type must be
    /// registered, and every edge endpoint must resolve to a node that either
    /// arrives in this batch or already exists. A batch that fails validation
    /// writes nothing.
    ///
    /// # Errors
    /// Returns [`DomainError::UnknownType`], [`DomainError::UnknownEndpoint`]
    /// or [`DomainError::Storage`].
    pub async fn ingest(
        &self,
        ctx: &SecurityContext,
        nodes: &[NodeInput],
        edges: &[EdgeInput],
    ) -> Result<IngestResult, DomainError> {
        if nodes.len() > self.config.ingest_max_nodes as usize {
            return Err(DomainError::BatchTooLarge {
                kind: "nodes",
                limit: self.config.ingest_max_nodes,
                requested: nodes.len(),
            });
        }
        if edges.len() > self.config.ingest_max_edges as usize {
            return Err(DomainError::BatchTooLarge {
                kind: "edges",
                limit: self.config.ingest_max_edges,
                requested: edges.len(),
            });
        }

        let tenant = ctx.subject_tenant_id();
        let scope = AccessScope::for_tenant(tenant);
        let conn = self
            .db
            .conn()
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        // Resolve every referenced type before writing anything.
        let mut type_ids = std::collections::HashMap::new();
        for t in nodes
            .iter()
            .map(|n| n.type_id.as_str())
            .chain(edges.iter().map(|e| e.type_id.as_str()))
        {
            if !type_ids.contains_key(t) {
                let id = ingest_repo::interned_type_id(&conn, &scope, t).await?;
                type_ids.insert(t.to_owned(), id);
            }
        }

        let node_rows: Vec<(String, i32, String)> = nodes
            .iter()
            .map(|n| (n.node_key.clone(), type_ids[&n.type_id], n.name.clone()))
            .collect();
        let nodes_upserted = ingest_repo::upsert_nodes(&conn, &scope, tenant, node_rows).await?;

        // Endpoints may arrive in this batch or already exist.
        let mut endpoint_keys: Vec<String> = edges
            .iter()
            .flat_map(|e| [e.from.clone(), e.to.clone()])
            .collect();
        endpoint_keys.sort();
        endpoint_keys.dedup();
        let ids = ingest_repo::resolve_node_ids(&conn, &scope, &endpoint_keys).await?;

        let mut edge_rows = Vec::with_capacity(edges.len());
        for e in edges {
            let src = *ids
                .get(&e.from)
                .ok_or_else(|| DomainError::UnknownEndpoint(e.from.clone()))?;
            let dst = *ids
                .get(&e.to)
                .ok_or_else(|| DomainError::UnknownEndpoint(e.to.clone()))?;
            let edge_key = format!("{}|{}|{}", e.type_id, e.from, e.to);
            edge_rows.push((edge_key, type_ids[&e.type_id], src, dst));
        }
        let edges_upserted = ingest_repo::upsert_edges(&conn, &scope, tenant, edge_rows).await?;

        Ok(IngestResult {
            nodes_upserted,
            edges_upserted,
        })
    }

    /// Expand a breadth-first neighbourhood around `seeds`.
    ///
    /// Depth is clamped to the configured maximum and the result to the node
    /// budget, so an unbounded request is rejected by construction rather than
    /// attempted. Only nodes the caller may see enter the frontier, so the walk
    /// stays inside the caller-authorised subgraph.
    ///
    /// # Errors
    /// Returns [`DomainError::Storage`] when a hop query fails.
    pub async fn neighbours(
        &self,
        ctx: &SecurityContext,
        seeds: &[i64],
        depth: u8,
    ) -> Result<Vec<i64>, DomainError> {
        let depth = depth.min(self.config.traversal_max_depth);
        let budget = self.config.traversal_max_nodes as usize;
        let scope = AccessScope::for_tenant(ctx.subject_tenant_id());
        let conn = self
            .db
            .conn()
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        let mut visited: Vec<i64> = seeds.to_vec();
        visited.sort_unstable();
        visited.dedup();
        let mut frontier = visited.clone();

        for _ in 0..depth {
            if frontier.is_empty() || visited.len() >= budget {
                break;
            }
            let neighbours = traversal::expand_frontier(&conn, &scope, &frontier, None).await?;
            frontier = neighbours
                .into_iter()
                .filter(|id| !visited.contains(id))
                .collect();
            visited.extend(frontier.iter().copied());
            visited.sort_unstable();
            visited.dedup();
        }

        visited.truncate(budget);
        Ok(visited)
    }
}
