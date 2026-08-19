//! Traversal under a scope narrower than a tenant.
//!
//! Every other parity case uses a tenant-only scope, which is the blind spot
//! this file exists to close: `graph_node` and `graph_edge` both map the `id`
//! resource property to their own primary key, so a scope naming node
//! identifiers means something different on each table. Applying the caller's
//! whole scope to both makes a hop return nothing — silently, and only for
//! callers whose authorization is narrower than a tenant.
//!
//! Skipped unless `GRAPH_STAND_DSN` is set.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::doc_markdown)]

use cf_gears_graph_storage::config::HopStrategy;
use cf_gears_graph_storage::infra::storage::traversal::{
    expand_frontier, expand_frontier_cte, is_tenant_only,
};
use cf_gears_graph_storage::infra::storage::traversal_pgq::expand_frontier_pgq;
use toolkit_db::secure::{AccessScope, DbConn};
use toolkit_security::access_scope::{ScopeConstraint, ScopeFilter, ScopeValue, pep_properties};

const STAND_TENANT: &str = "00000000-df51-5b42-9538-d2b56b7ee953";

async fn stand() -> Option<toolkit_db::secure::Db> {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the narrow-scope check");
        return None;
    };
    let opts = toolkit_db::ConnectOpts {
        max_conns: Some(2),
        min_conns: Some(1),
        ..Default::default()
    };
    Some(toolkit_db::connect_db(&dsn, opts).await.expect("connect"))
}

/// A scope of `tenant` plus an explicit list of authorized node ids.
fn scope_over(tenant: uuid::Uuid, node_ids: &[i64]) -> AccessScope {
    AccessScope::single(ScopeConstraint::new(vec![
        ScopeFilter::in_uuids(pep_properties::OWNER_TENANT_ID, vec![tenant]),
        ScopeFilter::r#in(
            pep_properties::RESOURCE_ID,
            node_ids.iter().map(|id| ScopeValue::from(*id)).collect(),
        ),
    ]))
}

/// A seed with at least three neighbours, so restricting to two of them is a
/// visible narrowing rather than a no-op.
async fn seed_with_neighbours(conn: &DbConn<'_>, tenant: uuid::Uuid) -> (i64, Vec<i64>) {
    let tenant_only = AccessScope::for_tenant(tenant);
    for candidate in [5000_i64, 1000, 2259, 1875, 42, 9737] {
        let got = expand_frontier(conn, &tenant_only, &[candidate], None)
            .await
            .expect("hop");
        if got.len() >= 3 {
            return (candidate, got);
        }
    }
    panic!("no seed with enough neighbours in the fixture");
}

/// The hop must return exactly the authorized neighbours — not all of them, and
/// not none of them. Returning none is the failure this test was written for:
/// it is silent, it looks like an empty neighbourhood, and it only happens to
/// callers who are not tenant-wide.
#[tokio::test]
async fn a_resource_narrowed_scope_returns_its_authorized_neighbours() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let (seed, neighbours) = seed_with_neighbours(&conn, tenant).await;
    let authorized: Vec<i64> = vec![seed, neighbours[0], neighbours[1]];
    let expected: Vec<i64> = {
        let mut e = vec![neighbours[0], neighbours[1]];
        e.sort_unstable();
        e
    };
    let scope = scope_over(tenant, &authorized);

    let two = expand_frontier(&conn, &scope, &[seed], None)
        .await
        .expect("two-query hop");
    assert_eq!(two, expected, "two-query hop under a narrowed scope");

    let pgq = expand_frontier_pgq(&conn, &scope, &[seed], None)
        .await
        .expect("pgq hop");
    assert_eq!(pgq, expected, "pgq hop under a narrowed scope");
}

/// An unauthorized neighbour stays out. The complement of the test above: the
/// narrowing has to remove rows, not merely preserve the ones it keeps.
#[tokio::test]
async fn an_unauthorized_neighbour_is_not_returned() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let (seed, neighbours) = seed_with_neighbours(&conn, tenant).await;
    let excluded = neighbours[2];
    let scope = scope_over(tenant, &[seed, neighbours[0], neighbours[1]]);

    for (name, got) in [
        (
            "two_query",
            expand_frontier(&conn, &scope, &[seed], None).await,
        ),
        (
            "pgq",
            expand_frontier_pgq(&conn, &scope, &[seed], None).await,
        ),
    ] {
        let ids = got.unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            !ids.contains(&excluded),
            "{name} returned the unauthorized neighbour {excluded}: {ids:?}"
        );
        assert!(!ids.is_empty(), "{name} returned nothing at all");
    }
}

/// The CTE hop cannot serve this scope, and the port must know it rather than
/// let it answer wrongly. Its edge query is a CTE body, and the safe-CTE API
/// scopes every body with the outer query's own scope by construction.
#[tokio::test]
async fn the_cte_hop_is_deflected_rather_than_wrong() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let (seed, neighbours) = seed_with_neighbours(&conn, tenant).await;
    let scope = scope_over(tenant, &[seed, neighbours[0], neighbours[1]]);

    assert!(
        !is_tenant_only(&scope),
        "the predicate the port dispatches on does not see this scope as narrowed"
    );

    // Called directly it under-returns, which is exactly why the port must not
    // route here. Pinned so the day the API can scope a CTE body separately,
    // this test fails and the deflection can be removed.
    let direct = expand_frontier_cte(&conn, &scope, &[seed], None)
        .await
        .expect("cte hop");
    assert!(
        direct.is_empty(),
        "the CTE hop now serves narrowed scopes; drop the deflection: {direct:?}"
    );
}

/// A tenant-only scope is not deflected — the predicate must not be so broad
/// that it routes everything to the fallback.
#[test]
fn a_tenant_scope_is_recognised_as_tenant_only() {
    assert!(is_tenant_only(&AccessScope::for_tenant(uuid::Uuid::nil())));
    assert!(is_tenant_only(&AccessScope::for_tenants(vec![
        uuid::Uuid::nil(),
        uuid::Uuid::from_u128(2)
    ])));
    assert!(is_tenant_only(&AccessScope::deny_all()));
    assert!(is_tenant_only(&AccessScope::allow_all()));
    assert!(!is_tenant_only(&AccessScope::for_resources(vec![
        uuid::Uuid::nil()
    ])));
    assert_eq!(HopStrategy::default(), HopStrategy::TwoQuery);
}
