//! REST handlers. Thin: they delegate to domain services and map results.

use std::sync::Arc;

use axum::{Extension, Json};
use toolkit_canonical_errors::CanonicalError;
use toolkit_security::SecurityContext;

use crate::api::rest::dto::GraphStatsDto;
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
