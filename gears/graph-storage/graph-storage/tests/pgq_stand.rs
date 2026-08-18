//! Execution check for the SQL/PGQ `FROM` source, against a real PostgreSQL 19.
//!
//! The unit tests in `infra::storage::pgq` assert what SQL is *built*. This
//! asserts that PostgreSQL accepts it, which is the half that cannot be checked
//! without a server: `GRAPH_TABLE` is a keyword construct, so a rendering
//! change that would be invisible in a string comparison — quoting the name,
//! interpolating instead of binding — surfaces here as a syntax error.
//!
//! SQL/PGQ exists only in PostgreSQL 19+, and the property graph is created by
//! the gear's migrations, so this needs the development stand rather than an
//! in-memory database. It is skipped when `GRAPH_STAND_DSN` is unset, so the
//! ordinary test run stays green:
//!
//! ```sh
//! GRAPH_STAND_DSN=postgres://graph:graph@127.0.0.1:55433/graph \
//!   cargo test -p cf-gears-graph-storage --test pgq_stand
//! ```

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::doc_markdown)]

use sea_orm::{ConnectionTrait, Database, Statement};

use cf_gears_graph_storage::infra::storage::pgq::{Direction, hop_statement};

/// Tenant the stand seeds under `auth_disabled`, where every request runs as
/// the platform's default tenant.
const STAND_TENANT: &str = "00000000-df51-5b42-9538-d2b56b7ee953";

#[tokio::test]
async fn the_built_pattern_executes_on_postgres_19() {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the stand execution check");
        return;
    };

    let db = Database::connect(&dsn).await.expect("connect to the stand");
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    // A seed with no outgoing edges is still a valid answer: the point is that
    // the statement parses, plans and runs, not that the fixture has data.
    let (sql, values) = hop_statement(&[5000], tenant, Direction::Outgoing, None);
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values,
        ))
        .await
        .expect("PostgreSQL rejected the generated GRAPH_TABLE statement");

    for row in &rows {
        let id: i64 = row.try_get("", "neighbour").expect("neighbour column");
        assert!(id > 0, "unexpected neighbour id: {id}");
    }
}

/// The tenant predicate must actually filter. Running the same pattern under a
/// tenant that owns nothing has to come back empty — if it does not, the
/// predicate is decorative and the pattern is reading across tenants.
#[tokio::test]
async fn a_foreign_tenant_sees_nothing() {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the stand execution check");
        return;
    };

    let db = Database::connect(&dsn).await.expect("connect to the stand");

    let (sql, values) = hop_statement(
        &[5000],
        uuid::Uuid::from_u128(0xdead),
        Direction::Outgoing,
        None,
    );
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values,
        ))
        .await
        .expect("statement failed");

    assert!(
        rows.is_empty(),
        "a tenant that owns nothing saw {} rows",
        rows.len()
    );
}

/// A frontier binds as one array parameter. This checks the database agrees:
/// several seeds in one statement return the union of their neighbours, and the
/// result matches what the seeds return one at a time.
#[tokio::test]
async fn a_multi_seed_frontier_returns_the_union_of_its_neighbours() {
    let Some(db) = stand().await else { return };
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let seeds = [5000_i64, 1000, 2259];
    let together = neighbours(&db, &seeds, tenant, None).await;

    let mut apart: Vec<i64> = Vec::new();
    for seed in seeds {
        apart.extend(neighbours(&db, &[seed], tenant, None).await);
    }
    apart.sort_unstable();
    apart.dedup();

    assert_eq!(
        together, apart,
        "one statement over the whole frontier disagreed with one statement per seed"
    );
}

/// The edge-type restriction reaches the edge element and filters. A predicate
/// that landed on the wrong variable, or was dropped, would show up here as the
/// unfiltered result.
#[tokio::test]
async fn an_edge_type_restriction_narrows_the_result() {
    let Some(db) = stand().await else { return };
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let all = neighbours(&db, &[5000], tenant, None).await;
    let typed = neighbours(&db, &[5000], tenant, Some(&[2])).await;
    let absent = neighbours(&db, &[5000], tenant, Some(&[i32::MAX])).await;

    assert!(
        typed.len() <= all.len(),
        "the restriction widened the result: {typed:?} vs {all:?}"
    );
    assert!(
        absent.is_empty(),
        "a type no edge carries still matched: {absent:?}"
    );
}

/// Both directions are reachable and they are not the same set — otherwise the
/// arrow rendering would be decorative.
#[tokio::test]
async fn the_two_directions_traverse_different_edges() {
    let Some(db) = stand().await else { return };
    let tenant = uuid::Uuid::parse_str(STAND_TENANT).unwrap();

    let out = run(
        &db,
        hop_statement(&[5000], tenant, Direction::Outgoing, None),
    )
    .await;
    let inc = run(
        &db,
        hop_statement(&[5000], tenant, Direction::Incoming, None),
    )
    .await;

    assert!(
        !out.is_empty() || !inc.is_empty(),
        "the fixture seed has no edges at all, so this proves nothing"
    );
    assert_ne!(
        out, inc,
        "both directions returned the same set, so the arrow is not being honoured"
    );
}

// ── helpers ────────────────────────────────────────────────────────────────

async fn stand() -> Option<sea_orm::DatabaseConnection> {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the stand execution check");
        return None;
    };
    Some(Database::connect(&dsn).await.expect("connect to the stand"))
}

async fn run(db: &sea_orm::DatabaseConnection, stmt: (String, sea_orm::Values)) -> Vec<i64> {
    let (sql, values) = stmt;
    let rows = db
        .query_all_raw(Statement::from_sql_and_values(
            db.get_database_backend(),
            &sql,
            values,
        ))
        .await
        .expect("PostgreSQL rejected the generated GRAPH_TABLE statement");

    let mut ids: Vec<i64> = rows
        .iter()
        .map(|r| r.try_get("", "neighbour").expect("neighbour column"))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    ids
}

async fn neighbours(
    db: &sea_orm::DatabaseConnection,
    frontier: &[i64],
    tenant: uuid::Uuid,
    edge_types: Option<&[i32]>,
) -> Vec<i64> {
    run(
        db,
        hop_statement(frontier, tenant, Direction::Outgoing, edge_types),
    )
    .await
}
