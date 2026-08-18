//! One breadth-first hop over the edge table, fully scoped.
//!
//! # Why this is two queries rather than one
//!
//! The natural single-statement shape is a scoped node query whose predicate
//! contains a subquery over the edge table:
//!
//! ```sql
//! SELECT n.id FROM graph_node n
//! WHERE <node scope> AND n.id IN (SELECT dst_node_id FROM graph_edge WHERE src_node_id = ANY($1))
//! ```
//!
//! A gear can build that with `sea_query`, but it cannot scope the subquery:
//! `build_scope_condition` is `pub` inside a private module
//! (`libs/toolkit-db/src/secure/mod.rs:106` declares `mod cond;`), so the scope
//! predicate is unobtainable outside `toolkit-db`. An unscoped edge subquery is
//! not merely imprecise — surrogate ids are per-tenant, so `src_node_id = 1`
//! matches an edge in every tenant that has a node 1, and the walk would follow
//! foreign edges. See `dev/FINDINGS.md (F1)`.
//!
//! Until a scoped custom-query primitive exists, each hop therefore runs two
//! scoped queries: edges first, then the authorised endpoints. Both go through
//! the secure ORM, so the tenant predicate is applied by construction.

use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QuerySelect};
use toolkit_db::secure::{AccessScope, DBRunner, SecureEntityExt};

use crate::domain::error::DomainError;
use crate::infra::storage::entity::{graph_edge, graph_node};

#[derive(Debug, FromQueryResult)]
struct EdgeEndpoints {
    src_node_id: i64,
    dst_node_id: i64,
}

#[derive(Debug, FromQueryResult)]
struct NodeId {
    id: i64,
}

/// Return the node ids one undirected hop away from `frontier`.
///
/// Edges are traversed in both directions. Only endpoints the caller is
/// authorised to see are returned, so the result can be used directly as the
/// next frontier: the walk never crosses a node outside the caller's scope.
///
/// # Errors
/// Returns [`DomainError::Storage`] when either query fails.
pub async fn expand_frontier<C: DBRunner>(
    conn: &C,
    scope: &AccessScope,
    frontier: &[i64],
    edge_type_ids: Option<&[i32]>,
) -> Result<Vec<i64>, DomainError> {
    if frontier.is_empty() {
        return Ok(Vec::new());
    }

    // 1. Scoped edge query: every edge incident to the frontier, either direction.
    let mut incident = sea_orm::Condition::any()
        .add(graph_edge::Column::SrcNodeId.is_in(frontier.iter().copied()))
        .add(graph_edge::Column::DstNodeId.is_in(frontier.iter().copied()));
    if let Some(types) = edge_type_ids {
        incident = sea_orm::Condition::all()
            .add(incident)
            .add(graph_edge::Column::TypeId.is_in(types.iter().copied()));
    }

    let endpoints: Vec<EdgeEndpoints> = graph_edge::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(incident)
        .project_all(conn, |q| {
            q.select_only()
                .column(graph_edge::Column::SrcNodeId)
                .column(graph_edge::Column::DstNodeId)
                .into_model::<EdgeEndpoints>()
        })
        .await
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    // 2. The endpoint on the far side of each incident edge.
    let mut candidates: Vec<i64> = Vec::with_capacity(endpoints.len());
    for e in endpoints {
        if frontier.contains(&e.src_node_id) {
            candidates.push(e.dst_node_id);
        }
        if frontier.contains(&e.dst_node_id) {
            candidates.push(e.src_node_id);
        }
    }
    candidates.sort_unstable();
    candidates.dedup();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Scoped node query: keep only endpoints the caller may see.
    let mut authorised: Vec<i64> = graph_node::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(sea_orm::Condition::all().add(graph_node::Column::Id.is_in(candidates)))
        .project_all(conn, |q| {
            q.select_only()
                .column(graph_node::Column::Id)
                .into_model::<NodeId>()
        })
        .await
        .map_err(|e| DomainError::Storage(e.to_string()))?
        .into_iter()
        .map(|n| n.id)
        .collect();

    authorised.sort_unstable();
    Ok(authorised)
}
