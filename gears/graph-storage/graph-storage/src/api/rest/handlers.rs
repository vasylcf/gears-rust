//! REST handlers. Thin: they delegate to domain services and map results.

use std::sync::Arc;

use axum::extract::Query;
use axum::{Extension, Json};
use serde::Deserialize;
use toolkit_canonical_errors::CanonicalError;
use toolkit_security::SecurityContext;

use crate::api::rest::dto::{GraphStatsDto, NeighboursDto};
use crate::domain::service::GraphServices;

/// Handler result alias.
pub type ApiResult<T> = Result<T, CanonicalError>;

/// Return coarse counters for the caller's graph.
#[tracing::instrument(skip(services, ctx), fields(user.id = %ctx.subject_id()))]
pub async fn get_stats(
    Extension(ctx): Extension<SecurityContext>,
    Extension(services): Extension<Arc<GraphServices>>,
) -> ApiResult<Json<GraphStatsDto>> {
    let stats = services.stats(&ctx).await?;
    Ok(Json(GraphStatsDto::from(stats)))
}

/// Query parameters of the neighbourhood endpoint.
#[derive(Debug, Deserialize)]
pub struct NeighboursParams {
    /// Comma-separated seed node ids.
    pub seeds: String,
    /// Requested depth; clamped to the configured maximum.
    #[serde(default = "default_depth")]
    pub depth: u8,
}

const fn default_depth() -> u8 {
    2
}

/// Expand a bounded neighbourhood around the given seeds.
#[tracing::instrument(skip(services, ctx), fields(user.id = %ctx.subject_id()))]
pub async fn get_neighbours(
    Extension(ctx): Extension<SecurityContext>,
    Extension(services): Extension<Arc<GraphServices>>,
    Query(params): Query<NeighboursParams>,
) -> ApiResult<Json<NeighboursDto>> {
    let seeds: Vec<i64> = params
        .seeds
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect();

    let budget = services.config().traversal_max_nodes as usize;
    let nodes = services.neighbours(&ctx, &seeds, params.depth).await?;
    let truncated = nodes.len() >= budget;

    Ok(Json(NeighboursDto { nodes, truncated }))
}
