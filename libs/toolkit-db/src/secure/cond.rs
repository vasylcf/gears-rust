use sea_orm::sea_query::{Alias, Query};
use sea_orm::{ColumnTrait, Condition, EntityTrait, ExprTrait, sea_query::Expr};

use crate::secure::{AccessScope, ScopableEntity};
use toolkit_security::access_scope::{
    ScopeConstraint, ScopeFilter, ScopeValue, rg_tables, tenant_tables,
};

/// Convert a [`ScopeValue`] to a `sea_query::SimpleExpr` for SQL binding.
fn scope_value_to_sea_expr(v: &ScopeValue) -> sea_orm::sea_query::SimpleExpr {
    match v {
        ScopeValue::Uuid(u) => Expr::value(*u),
        ScopeValue::String(s) => Expr::value(s.clone()),
        ScopeValue::Int(n) => Expr::value(*n),
        ScopeValue::Bool(b) => Expr::value(*b),
    }
}

/// Convert a slice of [`ScopeValue`] to `Vec<sea_orm::Value>` for IN clauses.
fn scope_values_to_sea_values(values: &[ScopeValue]) -> Vec<sea_orm::Value> {
    values
        .iter()
        .map(|v| match v {
            ScopeValue::Uuid(u) => sea_orm::Value::from(*u),
            ScopeValue::String(s) => sea_orm::Value::from(s.clone()),
            ScopeValue::Int(n) => sea_orm::Value::from(*n),
            ScopeValue::Bool(b) => sea_orm::Value::from(*b),
        })
        .collect()
}

/// Build a deny-all condition (`WHERE false`).
fn deny_all() -> Condition {
    Condition::all().add(Expr::value(false))
}

/// Builds a `SeaORM` `Condition` from an `AccessScope` using property resolution.
///
/// # OR/AND Semantics
///
/// - Multiple constraints are OR-ed (alternative access paths)
/// - Filters within a constraint are AND-ed (all must match)
/// - Unknown `pep_properties` fail that constraint (fail-closed)
/// - If all constraints fail resolution, deny-all
///
/// # Policy Rules
///
/// | Scope | Behavior |
/// |-------|----------|
/// | deny-all (default) | `WHERE false` |
/// | unconstrained (allow-all) | No filtering (`WHERE true`) |
/// | single constraint | AND of resolved filters |
/// | multiple constraints | OR of ANDed filter groups |
pub fn build_scope_condition<E>(scope: &AccessScope) -> Condition
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if scope.is_unconstrained() {
        return Condition::all();
    }
    if scope.is_deny_all() {
        return deny_all();
    }

    let compiled: Vec<Condition> = scope
        .constraints()
        .iter()
        .filter_map(build_constraint_condition::<E>)
        .collect();

    match compiled.len() {
        0 => deny_all(),
        1 => compiled.into_iter().next().unwrap_or_else(deny_all),
        _ => {
            let mut or_cond = Condition::any();
            for c in compiled {
                or_cond = or_cond.add(c);
            }
            or_cond
        }
    }
}

// ── ADR-0002 probe: addressing a scope condition by graph variable ─────────
//
// ADR-0002 says the one substantive change the Secure ORM core needs for
// property-graph support is to parameterise *how a resolved column is
// addressed*, rather than to write a second scope compiler:
//
//     for_table()          -> "resources"."tenant_id"
//     for_graph_element(v) -> "dst"."tenant_id"
//
// This is that change, implemented to find out whether one compiler really can
// serve both paths. It does -- with one qualification the ADR's diagram does
// not carry, described on `AddressError`.

/// How a resolved scope column is addressed in the emitted predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnAddress {
    /// Qualified by the entity's own table, as an ordinary select needs.
    Table,
    /// Qualified by a graph pattern variable, as an element pattern needs.
    ///
    /// A pattern element has no table reference; it has a variable. The
    /// property name is the entity's column name, which is why the property
    /// graph's DDL has to expose scope columns as properties under their own
    /// names -- see ADR-0002 Policy 3.
    GraphElement(&'static str),
}

/// A scope filter that cannot be expressed where the condition is going.
///
/// Three `ScopeFilter` arms -- `InGroup`, `InGroupSubtree`, `InTenantSubtree` --
/// compile to `col IN (SELECT ...)` over the membership or closure tables.
/// `PostgreSQL` 19 rejects a subquery anywhere inside `GRAPH_TABLE`, including
/// inside an element pattern's `WHERE`:
///
/// ```text
/// ERROR:  subqueries within GRAPH_TABLE reference are not supported
/// ```
///
/// So the addressing parameter alone does not make one compiler serve both
/// paths: in graph-element mode those arms have to fail, and fail **loudly**.
/// Dropping them would be fail-closed in the letter -- the constraint would
/// vanish and the scope would compile to `WHERE false` -- and that is exactly
/// the silent empty traversal ADR-0002 Policy 2 exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AddressError {
    /// The filter compiles to a subquery, which a pattern cannot hold.
    #[error(
        "scope filter `{0}` compiles to a subquery, which PostgreSQL rejects inside GRAPH_TABLE"
    )]
    SubqueryInPattern(&'static str),
}

impl ColumnAddress {
    /// Render a resolved column as an expression under this addressing.
    fn expr<C: ColumnTrait + Copy>(self, col: C) -> Expr {
        match self {
            Self::Table => col.into_expr(),
            Self::GraphElement(var) => {
                Expr::col((Alias::new(var), Alias::new(col.to_string().as_str())))
            }
        }
    }

    /// Whether a subquery-producing filter may be emitted here.
    const fn allows_subquery(self) -> bool {
        matches!(self, Self::Table)
    }
}

/// Compile `scope` into a condition addressed the given way.
///
/// The table addressing reproduces [`build_scope_condition`] exactly; the graph
/// addressing is the new path.
///
/// # Errors
/// Returns [`AddressError`] when a filter cannot be expressed under `address`.
pub fn build_scope_condition_addressed<E>(
    scope: &AccessScope,
    address: ColumnAddress,
) -> Result<Condition, AddressError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if scope.is_unconstrained() {
        return Ok(Condition::all());
    }
    if scope.is_deny_all() {
        return Ok(deny_all());
    }

    let mut compiled: Vec<Condition> = Vec::new();
    for constraint in scope.constraints() {
        if let Some(cond) = build_constraint_condition_addressed::<E>(constraint, address)? {
            compiled.push(cond);
        }
    }

    Ok(match compiled.len() {
        0 => deny_all(),
        1 => compiled.into_iter().next().unwrap_or_else(deny_all),
        _ => {
            let mut or_cond = Condition::any();
            for c in compiled {
                or_cond = or_cond.add(c);
            }
            or_cond
        }
    })
}

/// One constraint, addressed. `Ok(None)` keeps the fail-closed behaviour of the
/// original: a filter whose property does not resolve drops its constraint.
fn build_constraint_condition_addressed<E>(
    constraint: &ScopeConstraint,
    address: ColumnAddress,
) -> Result<Option<Condition>, AddressError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if constraint.is_empty() {
        return Ok(Some(Condition::all()));
    }
    let mut and_cond = Condition::all();
    for filter in constraint.filters() {
        let Some(col) = E::resolve_property(filter.property()) else {
            return Ok(None);
        };
        match filter {
            ScopeFilter::Eq(eq) => {
                let expr = scope_value_to_sea_expr(eq.value());
                and_cond = and_cond.add(address.expr(col).eq(expr));
            }
            ScopeFilter::In(inf) => {
                let sea_values = scope_values_to_sea_values(inf.values());
                and_cond = and_cond.add(address.expr(col).is_in(sea_values));
            }
            ScopeFilter::InGroup(_) if !address.allows_subquery() => {
                return Err(AddressError::SubqueryInPattern("InGroup"));
            }
            ScopeFilter::InGroupSubtree(_) if !address.allows_subquery() => {
                return Err(AddressError::SubqueryInPattern("InGroupSubtree"));
            }
            ScopeFilter::InTenantSubtree(_) if !address.allows_subquery() => {
                return Err(AddressError::SubqueryInPattern("InTenantSubtree"));
            }
            // The subquery arms under table addressing are unchanged; they are
            // reached through the original compiler so there is exactly one
            // definition of each.
            other => {
                let single = ScopeConstraint::new(vec![other.clone()]);
                if let Some(c) = build_constraint_condition::<E>(&single) {
                    and_cond = and_cond.add(c);
                } else {
                    return Ok(None);
                }
            }
        }
    }
    Ok(Some(and_cond))
}

/// Build SQL for a single constraint (AND of filters).
///
/// Returns `None` if any filter references an unknown property (fail-closed).
fn build_constraint_condition<E>(constraint: &ScopeConstraint) -> Option<Condition>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    if constraint.is_empty() {
        return Some(Condition::all());
    }
    let mut and_cond = Condition::all();
    for filter in constraint.filters() {
        let col = E::resolve_property(filter.property())?;
        match filter {
            ScopeFilter::Eq(eq) => {
                let expr = scope_value_to_sea_expr(eq.value());
                and_cond = and_cond.add(col.into_expr().eq(expr));
            }
            ScopeFilter::In(inf) => {
                let sea_values = scope_values_to_sea_values(inf.values());
                and_cond = and_cond.add(col.is_in(sea_values));
            }
            ScopeFilter::InGroup(gf) => {
                // col IN (SELECT resource_id FROM resource_group_membership
                //          WHERE group_id IN (...))
                let group_values = scope_values_to_sea_values(gf.group_ids());
                let subquery = Query::select()
                    .column(Alias::new(rg_tables::MEMBERSHIP_RESOURCE_ID))
                    .from(Alias::new(rg_tables::MEMBERSHIP_TABLE))
                    .and_where(
                        Expr::col(Alias::new(rg_tables::MEMBERSHIP_GROUP_ID)).is_in(group_values),
                    )
                    .to_owned();
                and_cond = and_cond.add(col.into_expr().in_subquery(subquery));
            }
            ScopeFilter::InGroupSubtree(sf) => {
                // col IN (SELECT resource_id FROM resource_group_membership
                //          WHERE group_id IN (
                //            SELECT descendant_id FROM resource_group_closure
                //            WHERE ancestor_id IN (...)
                //          ))
                let ancestor_values = scope_values_to_sea_values(sf.ancestor_ids());
                let closure_subquery = Query::select()
                    .column(Alias::new(rg_tables::CLOSURE_DESCENDANT_ID))
                    .from(Alias::new(rg_tables::CLOSURE_TABLE))
                    .and_where(
                        Expr::col(Alias::new(rg_tables::CLOSURE_ANCESTOR_ID))
                            .is_in(ancestor_values),
                    )
                    .to_owned();
                let membership_subquery = Query::select()
                    .column(Alias::new(rg_tables::MEMBERSHIP_RESOURCE_ID))
                    .from(Alias::new(rg_tables::MEMBERSHIP_TABLE))
                    .and_where(
                        Expr::col(Alias::new(rg_tables::MEMBERSHIP_GROUP_ID))
                            .in_subquery(closure_subquery),
                    )
                    .to_owned();
                and_cond = and_cond.add(col.into_expr().in_subquery(membership_subquery));
            }
            ScopeFilter::InTenantSubtree(sf) => {
                // Respect-barriers (default), no descendant_status filter:
                //   col IN (SELECT descendant_id FROM tenant_closure
                //            WHERE ancestor_id = root_tenant_id AND barrier = 0)
                // Ignore-barriers:
                //   col IN (SELECT descendant_id FROM tenant_closure
                //            WHERE ancestor_id = root_tenant_id)
                // Non-empty descendant_status appends:
                //   AND descendant_status IN (...)
                //
                // The closure invariant guarantees `(ancestor=X, descendant=X)`
                // is always present (self-row, barrier=0), enforced by AM's
                // `ck_tenant_closure_self_row_barrier` check constraint, so the
                // root tenant is included regardless of the barrier clamp.
                //
                // The composite index
                // `idx_tenant_closure_ancestor_barrier_status (ancestor_id, barrier, descendant_status)`
                // covers all three clauses, so a status filter does not
                // change the access path.
                let root_expr = scope_value_to_sea_expr(sf.root_tenant_id());
                let mut subquery = Query::select()
                    .column(Alias::new(tenant_tables::CLOSURE_DESCENDANT_ID))
                    .from(Alias::new(tenant_tables::CLOSURE_TABLE))
                    .and_where(
                        Expr::col(Alias::new(tenant_tables::CLOSURE_ANCESTOR_ID)).eq(root_expr),
                    )
                    .to_owned();
                if sf.respect_barriers() {
                    subquery
                        .and_where(Expr::col(Alias::new(tenant_tables::CLOSURE_BARRIER)).eq(0_i16));
                }
                if !sf.descendant_status().is_empty() {
                    let status_values = scope_values_to_sea_values(sf.descendant_status());
                    subquery.and_where(
                        Expr::col(Alias::new(tenant_tables::CLOSURE_DESCENDANT_STATUS))
                            .is_in(status_values),
                    );
                }
                and_cond = and_cond.add(col.into_expr().in_subquery(subquery));
            }
        }
    }
    Some(and_cond)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use toolkit_security::access_scope::{ScopeConstraint, ScopeFilter, pep_properties};

    #[test]
    fn test_deny_all_scope() {
        let scope = AccessScope::default();
        assert!(scope.is_deny_all());
    }

    #[test]
    fn test_allow_all_scope() {
        let scope = AccessScope::allow_all();
        assert!(scope.is_unconstrained());
    }

    #[test]
    fn test_tenant_scope_not_empty() {
        let tid = uuid::Uuid::new_v4();
        let scope = AccessScope::for_tenant(tid);
        assert!(!scope.is_deny_all());
        assert!(scope.contains_uuid(pep_properties::OWNER_TENANT_ID, tid));
    }

    #[test]
    fn test_or_scope_has_multiple_constraints() {
        let t1 = uuid::Uuid::new_v4();
        let t2 = uuid::Uuid::new_v4();
        let r1 = uuid::Uuid::new_v4();

        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![
                ScopeFilter::in_uuids(pep_properties::OWNER_TENANT_ID, vec![t1]),
                ScopeFilter::in_uuids(pep_properties::RESOURCE_ID, vec![r1]),
            ]),
            ScopeConstraint::new(vec![ScopeFilter::in_uuids(
                pep_properties::OWNER_TENANT_ID,
                vec![t2],
            )]),
        ]);
        assert_eq!(scope.constraints().len(), 2);
    }

    // --- Custom PEP property tests ---

    /// Test entity with a custom `department_id` property, mimicking what the
    /// derive macro generates for an entity with `pep_prop(department_id = "department_id")`.
    mod custom_prop_entity {
        use super::*;
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "custom_prop_test")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: Uuid,
            pub tenant_id: Uuid,
            pub department_id: Uuid,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        impl crate::secure::ScopableEntity for Entity {
            fn tenant_col() -> Option<Column> {
                Some(Column::TenantId)
            }
            fn resource_col() -> Option<Column> {
                Some(Column::Id)
            }
            fn owner_col() -> Option<Column> {
                None
            }
            fn type_col() -> Option<Column> {
                None
            }
            fn resolve_property(property: &str) -> Option<Column> {
                match property {
                    p if p == pep_properties::OWNER_TENANT_ID => Some(Column::TenantId),
                    p if p == pep_properties::RESOURCE_ID => Some(Column::Id),
                    "department_id" => Some(Column::DepartmentId),
                    _ => None,
                }
            }
        }
    }

    #[test]
    fn test_custom_property_resolves() {
        let dept = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::in_uuids(
                "department_id",
                vec![dept],
            )])]);
        // Should produce a real condition (not deny-all) since the entity resolves "department_id".
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        // A deny-all condition contains `Expr::value(false)` — verify this is NOT that.
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }

    #[test]
    fn test_unknown_property_deny_all() {
        let val = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::in_uuids(
                "nonexistent",
                vec![val],
            )])]);
        // Unknown property should cause the constraint to fail → deny-all.
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("Value(Bool(Some(false)))"),
            "Expected deny-all, got: {cond_str}"
        );
    }

    #[test]
    fn test_eq_filter_produces_equality_condition() {
        let tid = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::eq(
                pep_properties::OWNER_TENANT_ID,
                tid,
            )])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        // Should produce an equality condition, not an IN condition
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }

    #[test]
    fn test_in_group_filter_produces_subquery_condition() {
        let group_id = uuid::Uuid::new_v4();
        let scope =
            AccessScope::from_constraints(vec![ScopeConstraint::new(vec![ScopeFilter::in_group(
                pep_properties::RESOURCE_ID,
                vec![ScopeValue::Uuid(group_id)],
            )])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        // Should NOT be deny-all
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "InGroup should produce a real condition, got: {cond_str}"
        );
        // Verify the condition references the membership table and columns
        assert!(
            cond_str.contains("resource_group_membership"),
            "InGroup condition must reference resource_group_membership table, got: {cond_str}"
        );
        assert!(
            cond_str.contains("group_id"),
            "InGroup condition must filter by group_id, got: {cond_str}"
        );
        assert!(
            cond_str.contains("resource_id"),
            "InGroup condition must join on resource_id, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_respects_barrier_by_default() {
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                true,
                Vec::new(),
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "InTenantSubtree should produce a real condition, got: {cond_str}"
        );
        assert!(
            cond_str.contains("tenant_closure"),
            "InTenantSubtree condition must reference tenant_closure table, got: {cond_str}"
        );
        assert!(
            cond_str.contains("ancestor_id"),
            "InTenantSubtree condition must filter by ancestor_id, got: {cond_str}"
        );
        assert!(
            cond_str.contains("descendant_id"),
            "InTenantSubtree condition must select descendant_id, got: {cond_str}"
        );
        assert!(
            cond_str.contains("barrier"),
            "Respect-barriers mode must clamp closure subquery with barrier=0, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_ignore_barriers_omits_clamp() {
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                false,
                Vec::new(),
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("tenant_closure"),
            "Ignore-barriers must still produce closure subquery, got: {cond_str}"
        );
        assert!(
            !cond_str.contains("barrier"),
            "Ignore-barriers must NOT clamp on barrier column, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_binds_root_tenant_id() {
        // Single-root contract: the SQL must reference the root tenant UUID
        // exactly once via `ancestor_id = ?`, not an `IN` list. This guards
        // against accidental regressions to a multi-root encoding.
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                true,
                Vec::new(),
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains(&root_id.to_string()),
            "root tenant UUID must appear in the subquery, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_with_descendant_status_emits_clause() {
        // Non-empty descendant_status must add `AND descendant_status IN (...)`
        // to the closure subquery, binding the SMALLINT values verbatim.
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                true,
                vec![ScopeValue::Int(1), ScopeValue::Int(2)],
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("descendant_status"),
            "descendant_status clause must be present, got: {cond_str}"
        );
        // Status values are bound through sea-query as SMALLINT-compatible
        // integers — they appear as i64 placeholders in the debug print.
        assert!(
            cond_str.contains("BigInt(Some(1))") || cond_str.contains("Int(Some(1))"),
            "status value 1 must be bound, got: {cond_str}"
        );
        assert!(
            cond_str.contains("BigInt(Some(2))") || cond_str.contains("Int(Some(2))"),
            "status value 2 must be bound, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_empty_descendant_status_omits_clause() {
        // Empty descendant_status must NOT add a status predicate — the
        // ignore-barriers variant exposes this most cleanly because then the
        // only mention of `barrier` or `descendant_status` would come from
        // the status clause itself.
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                false,
                Vec::new(),
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("descendant_status"),
            "empty descendant_status must NOT emit clause, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_unknown_property_deny_all() {
        let root_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                "nonexistent",
                ScopeValue::Uuid(root_id),
                true,
                Vec::new(),
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("Value(Bool(Some(false)))"),
            "Unknown property must deny-all, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_group_subtree_filter_produces_subquery_condition() {
        let ancestor_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_group_subtree(
                pep_properties::RESOURCE_ID,
                vec![ScopeValue::Uuid(ancestor_id)],
            ),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "InGroupSubtree should produce a real condition, got: {cond_str}"
        );
        // Verify subtree condition references hierarchy tables
        assert!(
            cond_str.contains("resource_group_membership"),
            "InGroupSubtree condition must reference resource_group_membership table, got: {cond_str}"
        );
        assert!(
            cond_str.contains("resource_id"),
            "InGroupSubtree condition must join on resource_id, got: {cond_str}"
        );
    }

    #[test]
    fn test_tenant_plus_in_group_produces_and_condition() {
        let tid = uuid::Uuid::new_v4();
        let gid = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_uuids(pep_properties::OWNER_TENANT_ID, vec![tid]),
            ScopeFilter::in_group(pep_properties::RESOURCE_ID, vec![ScopeValue::Uuid(gid)]),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Combined tenant+group should produce a real condition, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_and_eq_produces_and_condition() {
        let root_id = uuid::Uuid::new_v4();
        let resource_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                true,
                Vec::new(),
            ),
            ScopeFilter::eq(pep_properties::RESOURCE_ID, resource_id),
        ])]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("tenant_closure"),
            "AND-composed condition must include closure subquery, got: {cond_str}"
        );
        assert!(
            cond_str.contains(&resource_id.to_string()),
            "AND-composed condition must include resource_id eq, got: {cond_str}"
        );
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "AND-composed condition must not deny-all, got: {cond_str}"
        );
    }

    #[test]
    fn test_in_tenant_subtree_or_with_in_produces_or_condition() {
        let root_id = uuid::Uuid::new_v4();
        let tenant_id = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                ScopeValue::Uuid(root_id),
                true,
                Vec::new(),
            )]),
            ScopeConstraint::new(vec![ScopeFilter::in_uuids(
                pep_properties::OWNER_TENANT_ID,
                vec![tenant_id],
            )]),
        ]);
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            cond_str.contains("tenant_closure"),
            "OR condition must include closure subquery branch, got: {cond_str}"
        );
        assert!(
            cond_str.contains(&tenant_id.to_string()),
            "OR condition must include plain IN branch, got: {cond_str}"
        );
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "OR condition must not deny-all, got: {cond_str}"
        );
    }

    #[test]
    fn test_standard_plus_custom_scope() {
        let tid = uuid::Uuid::new_v4();
        let dept = uuid::Uuid::new_v4();
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::in_uuids(pep_properties::OWNER_TENANT_ID, vec![tid]),
            ScopeFilter::in_uuids("department_id", vec![dept]),
        ])]);
        // Both standard and custom pep_properties should resolve successfully.
        let cond = build_scope_condition::<custom_prop_entity::Entity>(&scope);
        let cond_str = format!("{cond:?}");
        assert!(
            !cond_str.contains("Value(Bool(Some(false)))"),
            "Expected a real condition, got deny-all: {cond_str}"
        );
    }

    // ── ADR-0002 probe ────────────────────────────────────────────────────

    /// Table addressing must reproduce the original compiler exactly, or the
    /// "one compiler, two addressings" claim is a rewrite in disguise.
    #[test]
    fn table_addressing_matches_the_original_compiler() {
        use sea_orm::sea_query::{PostgresQueryBuilder, Query};

        let render = |cond: Condition| {
            Query::select()
                .expr(sea_orm::sea_query::Expr::val(1))
                .cond_where(cond)
                .to_owned()
                .to_string(PostgresQueryBuilder)
        };

        for scope in [
            AccessScope::for_tenant(uuid::Uuid::from_u128(1)),
            AccessScope::for_tenants(vec![uuid::Uuid::from_u128(1), uuid::Uuid::from_u128(2)]),
            AccessScope::deny_all(),
            AccessScope::allow_all(),
            AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                uuid::Uuid::from_u128(1),
                true,
                Vec::new(),
            )])),
        ] {
            let original = build_scope_condition::<custom_prop_entity::Entity>(&scope);
            let addressed = build_scope_condition_addressed::<custom_prop_entity::Entity>(
                &scope,
                ColumnAddress::Table,
            )
            .expect("table addressing never fails");
            assert_eq!(render(original), render(addressed), "scope: {scope:?}");
        }
    }

    /// Graph addressing qualifies by the pattern variable, not the table.
    #[test]
    fn graph_addressing_qualifies_by_the_pattern_variable() {
        use sea_orm::sea_query::{PostgresQueryBuilder, Query};

        let cond = build_scope_condition_addressed::<custom_prop_entity::Entity>(
            &AccessScope::for_tenant(uuid::Uuid::from_u128(1)),
            ColumnAddress::GraphElement("dst"),
        )
        .expect("an In filter is expressible in a pattern");

        let sql = Query::select()
            .expr(sea_orm::sea_query::Expr::val(1))
            .cond_where(cond)
            .to_owned()
            .to_string(PostgresQueryBuilder);

        assert!(sql.contains(r#""dst"."tenant_id""#), "{sql}");
        assert!(
            !sql.contains("custom_prop_test"),
            "the element was qualified by its table: {sql}"
        );
    }

    /// The three subquery arms must fail loudly under graph addressing.
    /// Dropping them would be fail-closed in the letter and a silent empty
    /// traversal in practice -- ADR-0002 Policy 2's hazard exactly.
    #[test]
    fn subquery_filters_are_refused_in_a_pattern_rather_than_dropped() {
        let subtree =
            AccessScope::single(ScopeConstraint::new(vec![ScopeFilter::in_tenant_subtree(
                pep_properties::OWNER_TENANT_ID,
                uuid::Uuid::from_u128(1),
                true,
                Vec::new(),
            )]));

        assert_eq!(
            build_scope_condition_addressed::<custom_prop_entity::Entity>(
                &subtree,
                ColumnAddress::GraphElement("dst")
            ),
            Err(AddressError::SubqueryInPattern("InTenantSubtree"))
        );

        // The same scope is fine against a table.
        assert!(
            build_scope_condition_addressed::<custom_prop_entity::Entity>(
                &subtree,
                ColumnAddress::Table
            )
            .is_ok()
        );
    }

    /// `deny_all` and `allow_all` mean the same under either addressing: they carry
    /// no column, so there is nothing to address.
    #[test]
    fn the_degenerate_scopes_are_addressing_independent() {
        for scope in [AccessScope::deny_all(), AccessScope::allow_all()] {
            assert!(
                build_scope_condition_addressed::<custom_prop_entity::Entity>(
                    &scope,
                    ColumnAddress::GraphElement("dst")
                )
                .is_ok(),
                "{scope:?}"
            );
        }
    }

    /// The exact predicate this compiler emits under graph addressing, pinned
    /// because it was executed inside a real `GRAPH_TABLE` on `PostgreSQL` 19
    /// beta2 in that form and accepted:
    ///
    /// ```sql
    /// MATCH (a IS node WHERE "a"."tenant_id" IN ('...') AND a.id = 5000)
    ///      -[e IS edge]->
    ///       (b IS node WHERE "b"."tenant_id" IN ('...'))
    /// ```
    ///
    /// It returned the owning tenant's two neighbours and nothing for a foreign
    /// tenant. If the rendering changes, this test fails and the executed
    /// evidence stops applying.
    #[test]
    fn the_graph_predicate_renders_as_the_shape_that_was_executed() {
        use sea_orm::sea_query::{PostgresQueryBuilder, Query};

        let tenant = uuid::Uuid::from_u128(0x1234);
        let cond = build_scope_condition_addressed::<custom_prop_entity::Entity>(
            &AccessScope::for_tenant(tenant),
            ColumnAddress::GraphElement("a"),
        )
        .expect("expressible");

        let sql = Query::select()
            .expr(sea_orm::sea_query::Expr::val(1))
            .cond_where(cond)
            .to_owned()
            .to_string(PostgresQueryBuilder);
        let predicate = sql.split(" WHERE ").nth(1).unwrap_or_default();

        assert_eq!(
            predicate,
            format!(r#""a"."tenant_id" IN ('{tenant}')"#),
            "the emitted predicate no longer matches the one that was executed"
        );
    }
}
