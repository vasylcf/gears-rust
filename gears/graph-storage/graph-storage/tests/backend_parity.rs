//! All three hop backends must answer identically.
//!
//! ADR-0001 puts traversal behind a port and lets the port choose a backend per
//! request shape. That is only honest if the choice is invisible in the answer,
//! so this compares the three implementations directly rather than through
//! HTTP: same scope, same frontier, same filters, same ids back.
//!
//! Direct calls also reach what the API does not currently expose — edge-type
//! filters and multi-seed frontiers — which is where a backend is most likely
//! to differ.
//!
//! Skipped unless `GRAPH_STAND_DSN` is set:
//!
//! ```sh
//! GRAPH_STAND_DSN=postgres://graph:graph@127.0.0.1:55433/graph \
//!   cargo test -p cf-gears-graph-storage --test backend_parity
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::doc_markdown)]

use cf_gears_graph_storage::infra::storage::traversal::{expand_frontier, expand_frontier_cte};
use cf_gears_graph_storage::infra::storage::traversal_pgq::expand_frontier_pgq;
use toolkit_db::secure::{AccessScope, DbConn};

const STAND_TENANT: &str = "00000000-df51-5b42-9538-d2b56b7ee953";

async fn stand() -> Option<toolkit_db::secure::Db> {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the backend parity check");
        return None;
    };
    let opts = toolkit_db::ConnectOpts {
        max_conns: Some(2),
        min_conns: Some(1),
        ..Default::default()
    };
    Some(toolkit_db::connect_db(&dsn, opts).await.expect("connect"))
}

fn tenant_scope() -> AccessScope {
    AccessScope::for_tenant(uuid::Uuid::parse_str(STAND_TENANT).unwrap())
}

/// Run one frontier through all three backends and assert they agree.
async fn assert_agree(
    conn: &DbConn<'_>,
    scope: &AccessScope,
    frontier: &[i64],
    edge_types: Option<&[i32]>,
    case: &str,
) -> Vec<i64> {
    let two = expand_frontier(conn, scope, frontier, edge_types)
        .await
        .unwrap_or_else(|e| panic!("{case}: two-query hop failed: {e}"));
    let cte = expand_frontier_cte(conn, scope, frontier, edge_types)
        .await
        .unwrap_or_else(|e| panic!("{case}: cte hop failed: {e}"));
    let pgq = expand_frontier_pgq(conn, scope, frontier, edge_types)
        .await
        .unwrap_or_else(|e| panic!("{case}: pgq hop failed: {e}"));

    assert_eq!(two, cte, "{case}: two-query and cte disagreed");
    assert_eq!(two, pgq, "{case}: two-query and pgq disagreed");
    two
}

/// Deterministic pseudo-random seeds, so a failure is reproducible.
fn seeds(count: usize) -> Vec<i64> {
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            1000 + i64::try_from(state % 190_000).unwrap()
        })
        .collect()
}

#[tokio::test]
async fn the_backends_agree_on_single_seeds() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    let mut non_empty = 0;
    for seed in seeds(40) {
        let got = assert_agree(&conn, &scope, &[seed], None, &format!("seed {seed}")).await;
        if !got.is_empty() {
            non_empty += 1;
        }
    }
    assert!(
        non_empty >= 10,
        "only {non_empty} of 40 seeds had neighbours; the fixture is too sparse to prove agreement"
    );
}

/// A multi-seed frontier is the shape every hop past the first actually takes,
/// and it is where the pgq backend differs most structurally: it binds the
/// whole frontier as one array rather than a value list.
#[tokio::test]
async fn the_backends_agree_on_multi_seed_frontiers() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    let all = seeds(50);
    for (i, chunk) in all.chunks(5).enumerate() {
        assert_agree(&conn, &scope, chunk, None, &format!("frontier {i}")).await;
    }
}

/// Two hops deep, feeding each backend its own first-hop result. If they
/// diverged at depth 1 the second hop would compound it.
#[tokio::test]
async fn the_backends_agree_after_a_second_hop() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    for seed in seeds(10) {
        let first = assert_agree(&conn, &scope, &[seed], None, "hop 1").await;
        if first.is_empty() {
            continue;
        }
        assert_agree(&conn, &scope, &first, None, &format!("hop 2 from {seed}")).await;
    }
}

/// Edge-type filtering is not reachable through the HTTP API yet, so this is
/// the only place the three implementations of it are compared.
#[tokio::test]
async fn the_backends_agree_on_edge_type_filters() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    for seed in seeds(20) {
        assert_agree(&conn, &scope, &[seed], Some(&[2]), "type 2").await;
        assert_agree(&conn, &scope, &[seed], Some(&[1, 2, 3]), "types 1-3").await;

        let absent = assert_agree(&conn, &scope, &[seed], Some(&[i32::MAX]), "absent type").await;
        assert!(
            absent.is_empty(),
            "a type no edge carries returned {absent:?}"
        );
    }
}

/// An empty frontier is an empty answer everywhere, not an unbounded query.
#[tokio::test]
async fn the_backends_agree_on_an_empty_frontier() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    let got = assert_agree(&conn, &scope, &[], None, "empty frontier").await;
    assert!(got.is_empty());
}

/// The cross-tenant trap: our tenant has 1 -> 2 -> 3, a foreign tenant has
/// 1 -> 3 with the same surrogate ids. A backend that followed a foreign edge
/// would reach 3 in one hop.
#[tokio::test]
async fn every_backend_holds_the_cross_tenant_trap() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = tenant_scope();

    let first = assert_agree(&conn, &scope, &[1], None, "trap hop 1").await;
    assert!(
        !first.contains(&3),
        "a backend reached the foreign tenant's shortcut: {first:?}"
    );

    let second = assert_agree(&conn, &scope, &first, None, "trap hop 2").await;
    assert!(
        second.contains(&1) || second.contains(&3),
        "the trap fixture is not wired as expected: {second:?}"
    );
}

/// A scope for a tenant that owns nothing gets nothing from any backend, even
/// when it names ids that exist under another tenant.
#[tokio::test]
async fn every_backend_denies_a_foreign_scope() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let foreign = AccessScope::for_tenant(uuid::Uuid::from_u128(0xdead));

    let got = assert_agree(&conn, &foreign, &seeds(5).as_slice()[..3], None, "foreign").await;
    assert!(got.is_empty(), "a foreign scope saw {got:?}");
}

/// `deny_all` is an empty answer from every backend rather than an error or a
/// widening.
#[tokio::test]
async fn every_backend_returns_nothing_for_deny_all() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");

    let got = assert_agree(&conn, &AccessScope::deny_all(), &[1, 2, 3], None, "deny").await;
    assert!(got.is_empty(), "deny_all returned {got:?}");
}
