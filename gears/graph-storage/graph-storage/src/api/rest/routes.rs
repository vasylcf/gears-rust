//! Route registration: one chain per endpoint describes the route, its
//! `OpenAPI` schema, authentication and every Problem status it can return.

use std::sync::Arc;

use axum::{Extension, Router};
use toolkit::api::OpenApiRegistry;
use toolkit::api::operation_builder::{
    CORE_GLOBAL_BASE_LICENSE_FEATURE, LicenseFeature, OperationBuilder,
};

use crate::api::rest::{dto, handlers};
use crate::domain::service::GraphServices;

const API_TAG: &str = "Graph Storage";
const BASE: &str = "/graph-storage/v1";

pub(crate) struct License;

impl AsRef<str> for License {
    fn as_ref(&self) -> &'static str {
        CORE_GLOBAL_BASE_LICENSE_FEATURE
    }
}

impl LicenseFeature for License {}

/// Register every REST route of the gear.
pub fn register_routes(
    router: Router,
    openapi: &dyn OpenApiRegistry,
    services: Arc<GraphServices>,
) -> Router {
    let router = OperationBuilder::get(format!("{BASE}/stats"))
        .operation_id("graph_storage.get_stats")
        .summary("Graph counters")
        .description("Coarse node and edge counters for the caller's graph")
        .tag(API_TAG)
        .authenticated()
        .require_license_features::<License>([])
        .handler(handlers::get_stats)
        .json_response_with_schema::<dto::GraphStatsDto>(
            openapi,
            http::StatusCode::OK,
            "Graph counters",
        )
        .error_401(openapi)
        .error_403(openapi)
        .error_500(openapi)
        .register(router, openapi);

    router.layer(Extension(services))
}
