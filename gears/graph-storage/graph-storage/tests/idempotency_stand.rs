//! Repeating a write must converge, not fail.
//!
//! `DESIGN` promises idempotent, conflict-rejecting type registration and a
//! bulk-ingest path where "repeating an identical batch converges instead of
//! duplicating". The implementation did neither: both `upsert_type` and
//! `upsert_edges` resolve their conflict with `ON CONFLICT DO NOTHING`, and when
//! the clause skips every row `SeaORM` reports the insert as
//! `DbErr::RecordNotInserted` — which both call sites treated as a failure.
//!
//! It was invisible to the whole existing suite because every test wrote fresh
//! keys. It surfaced on the first live repository import, which registers eight
//! types of which one already existed, as an opaque 500. Hence this file: the
//! second call is the assertion.
//!
//! Skipped unless `GRAPH_STAND_DSN` is set:
//!
//! ```sh
//! GRAPH_STAND_DSN=postgres://graph:graph@127.0.0.1:55433/graph \
//!   cargo test -p cf-gears-graph-storage --test idempotency_stand
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::doc_markdown)]

use cf_gears_graph_storage::infra::storage::ingest_repo::{
    current_revision, upsert_edges, upsert_nodes, upsert_type,
};
use toolkit_db::secure::{AccessScope, DbConn};
use uuid::Uuid;

/// A tenant of this test's own, so a rerun cannot collide with stand fixtures.
const TEST_TENANT: &str = "00000000-1de3-4001-8000-000000000001";

async fn stand() -> Option<toolkit_db::secure::Db> {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the idempotency check");
        return None;
    };
    let opts = toolkit_db::ConnectOpts {
        max_conns: Some(2),
        min_conns: Some(1),
        ..Default::default()
    };
    Some(toolkit_db::connect_db(&dsn, opts).await.expect("connect"))
}

fn tenant() -> Uuid {
    Uuid::parse_str(TEST_TENANT).unwrap()
}

fn scope() -> AccessScope {
    AccessScope::for_tenant(tenant())
}

/// Registering a type twice returns the same id both times.
///
/// The first call inserts, the second finds every row skipped by the conflict
/// clause. Before the fix the second call was an error.
#[tokio::test]
async fn a_type_can_be_registered_twice() {
    let Some(db) = stand().await else { return };
    let conn: DbConn<'_> = db.conn().expect("conn");
    let sc = scope();

    let type_id = "cf.test.idempotency.node.v1~";
    let first = upsert_type(&conn, &sc, tenant(), type_id, "node")
        .await
        .expect("first registration");
    let second = upsert_type(&conn, &sc, tenant(), type_id, "node")
        .await
        .expect("re-registering an existing type must not fail");

    assert_eq!(
        first, second,
        "a re-registration must keep the interned id, not mint a new one"
    );
}

/// Re-ingesting an identical batch of edges converges.
///
/// Edges conflict to `DO NOTHING`, so the second call skips every row — the
/// exact shape that used to be reported as a failure.
#[tokio::test]
async fn an_identical_edge_batch_can_be_ingested_twice() {
    let Some(db) = stand().await else { return };
    let conn: DbConn<'_> = db.conn().expect("conn");
    let sc = scope();

    let node_type = upsert_type(&conn, &sc, tenant(), "cf.test.idem.thing.v1~", "node")
        .await
        .expect("node type");
    let edge_type = upsert_type(&conn, &sc, tenant(), "cf.test.idem.links.v1~", "edge")
        .await
        .expect("edge type");

    let nodes = vec![
        ("idem:a".to_owned(), node_type, "a".to_owned()),
        ("idem:b".to_owned(), node_type, "b".to_owned()),
    ];
    upsert_nodes(&conn, &sc, tenant(), nodes.clone())
        .await
        .expect("nodes");

    // Resolve the ids the same way the ingest path does.
    let ids = cf_gears_graph_storage::infra::storage::ingest_repo::resolve_node_ids(
        &conn,
        &sc,
        &["idem:a".to_owned(), "idem:b".to_owned()],
    )
    .await
    .expect("resolve");
    let edges = vec![(
        "cf.test.idem.links.v1~|idem:a|idem:b".to_owned(),
        edge_type,
        ids["idem:a"],
        ids["idem:b"],
    )];

    upsert_edges(&conn, &sc, tenant(), edges.clone())
        .await
        .expect("first edge batch");
    upsert_edges(&conn, &sc, tenant(), edges)
        .await
        .expect("re-ingesting an unchanged edge batch must not fail");
}

/// The revision moves on a write and is readable without one.
///
/// It is documented as monotonic and returned the literal zero before this
/// change, so the assertion is that it moves at all — not what it reaches,
/// which depends on what else the stand has written.
#[tokio::test]
async fn the_revision_advances_on_a_write() {
    let Some(db) = stand().await else { return };
    let conn: DbConn<'_> = db.conn().expect("conn");
    let sc = scope();

    let before = current_revision(&conn, &sc).await.expect("read revision");

    let node_type = upsert_type(&conn, &sc, tenant(), "cf.test.idem.rev.v1~", "node")
        .await
        .expect("type");
    let key = format!("idem:rev:{}", Uuid::new_v4());
    upsert_nodes(
        &conn,
        &sc,
        tenant(),
        vec![(key, node_type, "rev".to_owned())],
    )
    .await
    .expect("node");
    cf_gears_graph_storage::infra::storage::ingest_repo::bump_revision(&conn, &sc, tenant())
        .await
        .expect("bump");

    let after = current_revision(&conn, &sc).await.expect("read revision");
    assert!(
        after > before,
        "the revision must advance on a write: {before} -> {after}"
    );
}
