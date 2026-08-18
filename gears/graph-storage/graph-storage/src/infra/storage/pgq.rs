//! SQL/PGQ `GRAPH_TABLE` as a `FROM` source, built through `sea_query`.
//!
//! # Why this module exists
//!
//! `sea_query` has no AST node for `GRAPH_TABLE`, so the obvious way to reach
//! SQL/PGQ from Rust is to fork `sea_query` and add one. That turns out to be
//! unnecessary. Three existing pieces compose into the construct:
//!
//! * [`TableRef::FunctionCall`] puts a function call in the `FROM` clause;
//! * `Func::Custom` renders the function's name **raw and unquoted**
//!   (`sea-query-1.0.2/src/backend/query_builder.rs:768`) — which matters,
//!   because `GRAPH_TABLE(...)` parses and `"GRAPH_TABLE"(...)` does not;
//! * `Expr::cust_with_values` renders arbitrary text while still **binding**
//!   its values, so the graph pattern carries parameters rather than literals.
//!
//! The pattern body is therefore the only free-form text in the statement, and
//! everything around it stays an ordinary `sea_query` select that the secure
//! ORM can scope.
//!
//! # What this module is not
//!
//! This is the pinned capability, not the query builder. It takes a pattern
//! body that the caller has already produced and does not inspect it, so it
//! must not be handed anything derived from request input. The typed builder
//! that makes the body safe to produce is the next step; until it exists, the
//! only callers are tests.
//!
//! `Expr::cust_with_values` is raw SQL, which gear code is not allowed to write
//! (`docs/arch/toolkit_unified_system/11_database_patterns.md`). On the
//! development stand that is a deliberate, contained exception so the approach
//! can be measured. The production home for this construct is inside
//! `toolkit-db`, which the platform CTE policy already exempts for exactly this
//! kind of dialect-specific assembly.

use std::borrow::Cow;

use sea_orm::Value;
use sea_orm::sea_query::{Alias, Expr, Func, IntoIden, TableRef};

/// Build a `GRAPH_TABLE (...) AS <alias>` source for a `FROM` clause.
///
/// `body` is everything between the parentheses — the graph name, the `MATCH`
/// pattern, its `WHERE`, and the `COLUMNS` list — with `$1`, `$2`, … marking
/// where `values` are bound. Placeholder numbering refers to `values` by
/// position; a placeholder used twice binds its value twice, and `sea_query`
/// renumbers the emitted placeholders accordingly.
///
/// `body` is `Cow<'static, str>` rather than `&str` because `sea_query` stores
/// it for the lifetime of the statement: a pattern is either a literal template
/// or a string the builder owns, never a slice borrowed from a request.
#[must_use]
pub fn graph_table_source(
    body: impl Into<Cow<'static, str>>,
    values: Vec<Value>,
    alias: &str,
) -> TableRef {
    let call = Func::cust(Alias::new("GRAPH_TABLE")).arg(Expr::cust_with_values(body, values));
    TableRef::FunctionCall(call, Alias::new(alias).into_iden())
}

/// The one-hop pattern body the traversal backend needs, as a template.
///
/// Kept next to the capability it exercises so the pinned test asserts on the
/// shape the gear will actually emit, rather than on a toy. Direction is
/// explicit because the undirected shorthand plans as an all-vertex probe on
/// the initial `PostgreSQL` 19 implementation (see `docs/SPIKE-pg19-sqlpgq.md`).
///
/// `$1` is the seed node id, `$2` the tenant. The tenant predicate is repeated
/// on both endpoints: composite element keys already make an edge unable to
/// reach another tenant's node, but a pattern with no tenant predicate at all
/// still returns rows from every tenant.
pub(crate) const OUTGOING_HOP: &str = "kb_pgq MATCH (a IS node)-[e IS edge]->(b IS node) \
     WHERE a.id = $1 AND a.tenant_id = $2 AND b.tenant_id = $2 \
     COLUMNS (b.id AS neighbour)";

/// Render a full statement selecting neighbours through `GRAPH_TABLE`.
///
/// Exists so the pinned test can assert on the emitted SQL and so the execution
/// check can run the exact statement the builder produces.
#[must_use]
pub fn outgoing_hop_statement(seed: i64, tenant: uuid::Uuid) -> (String, sea_orm::Values) {
    use sea_orm::sea_query::{PostgresQueryBuilder, Query};

    Query::select()
        .column(Alias::new(NEIGHBOUR_COLUMN))
        .from(graph_table_source(
            OUTGOING_HOP,
            vec![Value::from(seed), Value::from(tenant)],
            "g",
        ))
        .to_owned()
        .build(PostgresQueryBuilder)
}

/// Column name the hop projects, so tests and callers agree on it.
pub(crate) const NEIGHBOUR_COLUMN: &str = "neighbour";

/// Assert at compile time that the entity columns the pattern names still
/// exist. The pattern body is a string, so a rename in the entity would
/// otherwise be caught only at runtime, by the database.
#[cfg(test)]
pub(crate) fn pattern_columns_still_exist() {
    use crate::infra::storage::entity::{graph_edge, graph_node};
    use sea_orm::ColumnTrait;
    let _ = graph_node::Column::Id.as_column_ref();
    let _ = graph_node::Column::TenantId.as_column_ref();
    let _ = graph_edge::Column::SrcNodeId.as_column_ref();
    let _ = graph_edge::Column::DstNodeId.as_column_ref();
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// `Func::Custom` must render the name unquoted. `GRAPH_TABLE(...)` parses;
    /// `"GRAPH_TABLE"(...)` is a syntax error, because the construct is a
    /// keyword and not a function the catalog knows. A `sea_query` upgrade that
    /// starts quoting custom function names would break SQL/PGQ silently at
    /// runtime, so it is pinned here instead.
    #[test]
    fn the_construct_name_is_not_quoted() {
        let (sql, _) = outgoing_hop_statement(5000, Uuid::nil());

        assert!(
            sql.contains("FROM GRAPH_TABLE("),
            "expected an unquoted construct name: {sql}"
        );
        assert!(
            !sql.contains("\"GRAPH_TABLE\""),
            "the construct name was quoted, which PostgreSQL rejects: {sql}"
        );
    }

    /// The pattern carries bound parameters, not literals. A pattern that
    /// interpolated its values would defeat the plan cache and, far worse, put
    /// caller-derived text inside a construct nothing else validates.
    #[test]
    fn pattern_values_are_bound_not_interpolated() {
        let tenant = Uuid::from_u128(0x5eed);
        let (sql, values) = outgoing_hop_statement(5000, tenant);

        assert!(
            !sql.contains(&tenant.to_string()),
            "the tenant was interpolated into the pattern: {sql}"
        );
        assert!(!sql.contains("5000"), "the seed was interpolated: {sql}");
        assert_eq!(
            values.0.len(),
            3,
            "expected seed plus the tenant bound once per endpoint: {values:?}"
        );
    }

    /// A placeholder used twice binds its value twice and the emitted
    /// placeholders are renumbered. The hop relies on this: `$2` names the
    /// tenant on both endpoints of the pattern.
    #[test]
    fn a_repeated_placeholder_is_renumbered() {
        let (sql, _) = outgoing_hop_statement(5000, Uuid::nil());

        assert!(
            sql.contains("a.id = $1") && sql.contains("a.tenant_id = $2"),
            "unexpected placeholder numbering: {sql}"
        );
        assert!(
            sql.contains("b.tenant_id = $3"),
            "the repeated placeholder was not renumbered: {sql}"
        );
    }

    /// Both endpoints carry the tenant. Composite element keys already stop an
    /// edge from reaching another tenant's node, but a pattern with no tenant
    /// predicate returns rows from every tenant, so the predicate is not
    /// optional.
    #[test]
    fn both_endpoints_carry_the_tenant_predicate() {
        let (sql, _) = outgoing_hop_statement(1, Uuid::nil());

        assert!(
            sql.contains("a.tenant_id ="),
            "seed endpoint unscoped: {sql}"
        );
        assert!(
            sql.contains("b.tenant_id ="),
            "target endpoint unscoped: {sql}"
        );
    }

    /// The pattern is a string, so a column rename in the entity would be caught
    /// by the database rather than the compiler. This keeps the names the
    /// pattern spells tied to columns that exist.
    #[test]
    fn the_columns_the_pattern_names_still_exist() {
        pattern_columns_still_exist();
        assert!(OUTGOING_HOP.contains(NEIGHBOUR_COLUMN));
    }
}
