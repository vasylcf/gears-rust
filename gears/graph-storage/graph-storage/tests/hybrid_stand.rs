//! Single-statement hybrid retrieval against a real `PostgreSQL` 19.
//!
//! Skipped unless `GRAPH_STAND_DSN` is set:
//!
//! ```sh
//! GRAPH_STAND_DSN=postgres://graph:graph@127.0.0.1:55433/graph \
//!   cargo test -p cf-gears-graph-storage --test hybrid_stand
//! ```

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use cf_gears_graph_storage::infra::storage::hybrid::{HybridRequest, hybrid_neighbourhood};
use cf_gears_graph_storage::infra::storage::traversal_pgq::expand_frontier_pgq;
use toolkit_db::secure::AccessScope;

const STAND_TENANT: &str = "00000000-df51-5b42-9538-d2b56b7ee953";

async fn stand() -> Option<toolkit_db::secure::Db> {
    let Ok(dsn) = std::env::var("GRAPH_STAND_DSN") else {
        eprintln!("GRAPH_STAND_DSN unset - skipping the hybrid stand check");
        return None;
    };
    let opts = toolkit_db::ConnectOpts {
        max_conns: Some(2),
        min_conns: Some(1),
        ..Default::default()
    };
    Some(toolkit_db::connect_db(&dsn, opts).await.expect("connect"))
}

/// A deterministic query vector, so runs are comparable.
fn query_vector() -> Vec<f32> {
    (0..384).map(|i| (i as f32 % 17.0) / 17.0).collect()
}

/// The whole composition runs as one statement and returns ranked hits.
#[tokio::test]
async fn vector_graph_and_text_compose_in_one_statement() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = AccessScope::for_tenant(uuid::Uuid::parse_str(STAND_TENANT).unwrap());
    let v = query_vector();

    let hits = hybrid_neighbourhood(
        &conn,
        &scope,
        &HybridRequest {
            query_vector: &v,
            text: "alpha",
            seed_limit: 50,
            limit: 10,
        },
    )
    .await
    .expect("hybrid query");

    assert!(
        !hits.is_empty(),
        "the fixture produced no hits, so this proves nothing"
    );
    assert!(hits.len() <= 10, "the limit was not applied: {hits:?}");

    // Ranked by distance, ascending.
    let mut sorted = hits.clone();
    sorted.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    assert_eq!(hits, sorted, "results are not ordered by distance");
}

/// The text filter is a filter. Asking for a term nothing carries must return
/// nothing, even though the vector and graph halves still match plenty.
#[tokio::test]
async fn the_text_filter_narrows_the_result() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = AccessScope::for_tenant(uuid::Uuid::parse_str(STAND_TENANT).unwrap());
    let v = query_vector();

    let matched = hybrid_neighbourhood(
        &conn,
        &scope,
        &HybridRequest {
            query_vector: &v,
            text: "alpha",
            seed_limit: 50,
            limit: 10,
        },
    )
    .await
    .expect("hybrid query");

    let absent = hybrid_neighbourhood(
        &conn,
        &scope,
        &HybridRequest {
            query_vector: &v,
            text: "zzzznotaword",
            seed_limit: 50,
            limit: 10,
        },
    )
    .await
    .expect("hybrid query");

    assert!(!matched.is_empty(), "the matching term found nothing");
    assert!(
        absent.is_empty(),
        "a term nothing carries matched: {absent:?}"
    );
}

/// Every hit is genuinely one hop from a seed. Checked against the plain hop,
/// which is verified separately against the two-query backend — so the graph
/// half of the composition is not just decorative.
#[tokio::test]
async fn every_hit_is_reachable_from_the_seeds() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = AccessScope::for_tenant(uuid::Uuid::parse_str(STAND_TENANT).unwrap());
    let v = query_vector();

    let hits = hybrid_neighbourhood(
        &conn,
        &scope,
        &HybridRequest {
            query_vector: &v,
            text: "alpha",
            seed_limit: 50,
            limit: 10,
        },
    )
    .await
    .expect("hybrid query");

    let seeds = knn_seed_ids(&conn, &scope, &v, 50).await;
    let reachable = expand_frontier_pgq(&conn, &scope, &seeds, None)
        .await
        .expect("hop");

    for hit in &hits {
        assert!(
            reachable.contains(&hit.id),
            "hit {} is not one hop from any seed",
            hit.id
        );
    }
}

/// A foreign tenant sees nothing: the seed search, the pattern and the outer
/// query all carry the bound.
#[tokio::test]
async fn a_foreign_scope_sees_nothing() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let v = query_vector();

    let hits = hybrid_neighbourhood(
        &conn,
        &AccessScope::for_tenant(uuid::Uuid::from_u128(0xdead)),
        &HybridRequest {
            query_vector: &v,
            text: "alpha",
            seed_limit: 50,
            limit: 10,
        },
    )
    .await
    .expect("hybrid query");

    assert!(hits.is_empty(), "a foreign scope saw {hits:?}");
}

/// Seeds through the secure ORM, for the reachability check above.
async fn knn_seed_ids(
    conn: &toolkit_db::secure::DbConn<'_>,
    scope: &AccessScope,
    query_vector: &[f32],
    limit: u64,
) -> Vec<i64> {
    use cf_gears_graph_storage::infra::storage::entity::graph_node;
    use sea_orm::sea_query::ExprTrait;
    use sea_orm::sea_query::{Alias, Expr, Order};
    use sea_orm::{EntityTrait, FromQueryResult, QueryOrder, QuerySelect, Value};
    use toolkit_db::secure::SecureEntityExt;

    #[derive(Debug, FromQueryResult)]
    struct Id {
        id: i64,
    }

    let literal = format!(
        "[{}]",
        query_vector
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );

    graph_node::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(sea_orm::Condition::all().add(Expr::col(Alias::new("embedding")).is_not_null()))
        .project_all(conn, move |q| {
            q.select_only()
                .column(graph_node::Column::Id)
                .order_by(
                    Expr::cust_with_values("embedding <=> $1::vector", [Value::from(literal)]),
                    Order::Asc,
                )
                .limit(limit)
                .into_model::<Id>()
        })
        .await
        .expect("knn seeds")
        .into_iter()
        .map(|r| r.id)
        .collect()
}

/// Measurement, not an assertion: what the single statement buys against the
/// same result assembled from three round trips. Run explicitly:
///
/// ```sh
/// GRAPH_STAND_DSN=... cargo test -p cf-gears-graph-storage --test hybrid_stand \
///   -- --ignored --nocapture bench
/// ```
#[tokio::test]
#[ignore = "measurement, run explicitly"]
async fn bench_single_statement_against_decomposed() {
    let Some(db) = stand().await else { return };
    let conn = db.conn().expect("conn");
    let scope = AccessScope::for_tenant(uuid::Uuid::parse_str(STAND_TENANT).unwrap());
    let v = query_vector();
    let request = HybridRequest {
        query_vector: &v,
        text: "alpha",
        seed_limit: 50,
        limit: 10,
    };

    // Warm both paths so the comparison is not a first-plan artefact.
    let _ = hybrid_neighbourhood(&conn, &scope, &request).await.unwrap();
    let _ = decomposed(&conn, &scope, &request).await;

    let mut single = Vec::new();
    let mut three = Vec::new();
    for _ in 0..25 {
        let t = std::time::Instant::now();
        let a = hybrid_neighbourhood(&conn, &scope, &request).await.unwrap();
        single.push(t.elapsed().as_secs_f64() * 1000.0);

        let t = std::time::Instant::now();
        let b = decomposed(&conn, &scope, &request).await;
        three.push(t.elapsed().as_secs_f64() * 1000.0);

        assert_eq!(
            a.iter().map(|h| h.id).collect::<Vec<_>>(),
            b,
            "the two paths disagreed, so the timing means nothing"
        );
    }

    report("single statement", &mut single);
    report("three round trips", &mut three);
}

fn report(label: &str, samples: &mut [f64]) {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p = |q: f64| samples[((samples.len() as f64 - 1.0) * q) as usize];
    println!(
        "  {label:<20} p50={:.1}ms p95={:.1}ms max={:.1}ms",
        p(0.5),
        p(0.95),
        samples[samples.len() - 1]
    );
}

/// The same answer, assembled the way it must be without SQL/PGQ: seeds, then
/// expansion, then the text filter and ranking. Three statements, three round
/// trips, and an intermediate frontier crossing the process boundary twice.
async fn decomposed(
    conn: &toolkit_db::secure::DbConn<'_>,
    scope: &AccessScope,
    request: &HybridRequest<'_>,
) -> Vec<i64> {
    use cf_gears_graph_storage::infra::storage::entity::graph_node;
    use sea_orm::sea_query::{Expr, Order};
    use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryOrder, QuerySelect, Value};
    use toolkit_db::secure::SecureEntityExt;

    #[derive(Debug, FromQueryResult)]
    struct Id {
        id: i64,
    }

    let seeds = knn_seed_ids(
        conn,
        scope,
        request.query_vector,
        u64::from(request.seed_limit),
    )
    .await;
    let reached = expand_frontier_pgq(conn, scope, &seeds, None)
        .await
        .expect("hop");
    if reached.is_empty() {
        return Vec::new();
    }

    let literal = format!(
        "[{}]",
        request
            .query_vector
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    );
    let text = request.text.to_owned();
    let limit = u64::from(request.limit);

    graph_node::Entity::find()
        .secure()
        .scope_with(scope)
        .filter(
            sea_orm::Condition::all()
                .add(graph_node::Column::Id.is_in(reached))
                .add(Expr::cust_with_values(
                    "to_tsvector('simple', search_text) @@ plainto_tsquery('simple', $1)",
                    [Value::from(text)],
                )),
        )
        .project_all(conn, move |q| {
            q.select_only()
                .column(graph_node::Column::Id)
                .order_by(
                    Expr::cust_with_values("embedding <=> $1::vector", [Value::from(literal)]),
                    Order::Asc,
                )
                .limit(limit)
                .into_model::<Id>()
        })
        .await
        .expect("final query")
        .into_iter()
        .map(|r| r.id)
        .collect()
}
