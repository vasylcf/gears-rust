//! Domain services.
//!
//! Phase 2 carries a single read-only service backed by the secure ORM, so the
//! storage and authorization wiring can be exercised end to end. Ingest,
//! search, traversal and analytics land here as their layers are implemented.

use std::sync::Arc;

use graph_storage_sdk::GraphStats;
use toolkit_db::{DBProvider, DbError};
use toolkit_security::{AccessScope, SecurityContext};

use crate::config::GraphStorageConfig;
use crate::domain::error::DomainError;
use crate::infra::storage::{counts, traversal};

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
