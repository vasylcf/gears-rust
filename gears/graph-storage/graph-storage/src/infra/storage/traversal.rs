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

/// Single-statement variant of [`expand_frontier`], using a scoped CTE.
///
/// Both the CTE body and the outer query carry the caller's scope, so the
/// tenant predicate is present on the edge table and on the node table in one
/// statement. This is what the two-query hop collapses into once `toolkit-db`
/// exposes safe CTEs; it exists here to measure the difference.
///
/// # Errors
/// Returns [`DomainError::Storage`] when the query fails.
pub async fn expand_frontier_cte<C: DBRunner>(
    conn: &C,
    scope: &AccessScope,
    frontier: &[i64],
    edge_type_ids: Option<&[i32]>,
) -> Result<Vec<i64>, DomainError> {
    use sea_orm::sea_query::{Expr, ExprTrait};
    use toolkit_db::secure::cte_columns_union;

    if frontier.is_empty() {
        return Ok(Vec::new());
    }

    let mut incident = sea_orm::Condition::any()
        .add(graph_edge::Column::SrcNodeId.is_in(frontier.iter().copied()))
        .add(graph_edge::Column::DstNodeId.is_in(frontier.iter().copied()));
    if let Some(types) = edge_type_ids {
        incident = sea_orm::Condition::all()
            .add(incident)
            .add(graph_edge::Column::TypeId.is_in(types.iter().copied()));
    }

    // Project the body: a CTE referenced twice is materialised, and the edge
    // table carries a jsonb payload the hop never reads.
    let edges_cte = graph_edge::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(incident)
        .into_cte_projected("scoped_edges", |q| {
            q.select_only()
                .column(graph_edge::Column::SrcNodeId)
                .column(graph_edge::Column::DstNodeId)
        });

    // One `IN` over the union of both endpoint columns, not two `IN`s joined by
    // `OR`: the `OR` form costs a sequential scan of `graph_node`. See
    // `cte_columns_union` and `dev/FINDINGS.md (F9)`.
    let endpoint_ids = cte_columns_union("scoped_edges", "src_node_id", &["dst_node_id"]);
    let endpoints =
        sea_orm::Condition::all().add(Expr::col(graph_node::Column::Id).in_subquery(endpoint_ids));

    let mut ids: Vec<i64> = graph_node::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(endpoints)
        .project_with_ctes([edges_cte], |q| {
            q.select_only().column(graph_node::Column::Id)
        })
        .map_err(|e| DomainError::Storage(e.to_string()))?
        .all_as::<NodeId>(conn)
        .await
        .map_err(|e| DomainError::Storage(e.to_string()))?
        .into_iter()
        .map(|n| n.id)
        .collect();

    ids.sort_unstable();
    Ok(ids)
}

/// Render the single-statement hop without executing it.
///
/// Used by the finding test to assert that the `WITH` clause survives and that
/// the scope predicate is present inside the CTE body, not only around it.
#[must_use]
pub fn expand_frontier_cte_sql(scope: &AccessScope, frontier: &[i64]) -> String {
    use sea_orm::sea_query::{Expr, ExprTrait};
    use toolkit_db::secure::cte_columns_union;

    let incident = sea_orm::Condition::any()
        .add(graph_edge::Column::SrcNodeId.is_in(frontier.iter().copied()))
        .add(graph_edge::Column::DstNodeId.is_in(frontier.iter().copied()));

    let edges_cte = graph_edge::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(incident)
        .into_cte_projected("scoped_edges", |q| {
            q.select_only()
                .column(graph_edge::Column::SrcNodeId)
                .column(graph_edge::Column::DstNodeId)
        });

    let endpoint_ids = cte_columns_union("scoped_edges", "src_node_id", &["dst_node_id"]);

    graph_node::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(
            sea_orm::Condition::all()
                .add(Expr::col(graph_node::Column::Id).in_subquery(endpoint_ids)),
        )
        .project_with_ctes([edges_cte], |q| {
            q.select_only().column(graph_node::Column::Id)
        })
        .map(toolkit_db::secure::SecureCteSelect::to_sql)
        .unwrap_or_default()
}

#[cfg(test)]
mod cte_tests {
    use super::*;
    use uuid::Uuid;

    /// The Level A invariant, checked on emitted SQL: the tenant predicate must
    /// appear **inside** the CTE body, not only around it, and the `WITH`
    /// clause must survive into the executed statement.
    #[test]
    fn scope_lands_inside_the_cte_body() {
        let tenant = Uuid::from_u128(0x1234);
        let scope = AccessScope::for_tenant(tenant);
        let sql = expand_frontier_cte_sql(&scope, &[1, 2, 3]);

        assert!(sql.starts_with("WITH "), "the WITH clause vanished: {sql}");

        let body_start = sql.find("AS (").expect("cte body");
        let body_end = sql.find(") SELECT").unwrap_or(sql.len());
        let body = &sql[body_start..body_end];
        assert!(
            body.contains("tenant_id"),
            "cte body carries no tenant predicate: {body}"
        );

        let outer = &sql[body_end..];
        assert!(
            outer.contains("tenant_id"),
            "outer query carries no tenant predicate: {outer}"
        );
    }

    /// The hop must probe `graph_node` through a single semi-join. Two `IN`
    /// subqueries joined by `OR` are logically equivalent but make `PostgreSQL`
    /// sequentially scan the node table (15.2 ms versus 0.30 ms on the stand),
    /// so the shape is load-bearing, not stylistic. See `dev/FINDINGS.md (F9)`.
    #[test]
    fn the_outer_query_probes_the_node_table_once() {
        let scope = AccessScope::for_tenant(Uuid::from_u128(0x1234));
        let sql = expand_frontier_cte_sql(&scope, &[1, 2, 3]);

        let outer = &sql[sql.find(") SELECT").expect("outer query")..];
        assert_eq!(
            outer.matches(" IN (SELECT ").count(),
            1,
            "the outer query must contain exactly one IN-subquery: {outer}"
        );
    }
}
