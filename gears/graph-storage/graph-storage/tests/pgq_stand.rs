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

use cf_gears_graph_storage::infra::storage::pgq::outgoing_hop_statement;

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
    let (sql, values) = outgoing_hop_statement(5000, tenant);
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

    let (sql, values) = outgoing_hop_statement(5000, uuid::Uuid::from_u128(0xdead));
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
