//! Composition root: wires configuration, services and adapters together.

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use toolkit::api::OpenApiRegistry;
use toolkit::{Gear, GearCtx, RestApiCapability};
use tracing::{debug, info};

use graph_storage_sdk::GraphStorageClientV1;

use crate::api::rest::routes;
use crate::config::GraphStorageConfig;
use crate::domain::local_client::GraphStorageLocalClient;
use crate::domain::service::GraphServices;

/// The graph-storage gear.
#[toolkit::gear(name = "graph-storage", capabilities = [rest])]
pub struct GraphStorage {
    services: OnceLock<Arc<GraphServices>>,
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self {
            services: OnceLock::new(),
        }
    }
}

#[async_trait]
impl Gear for GraphStorage {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let cfg: GraphStorageConfig = ctx.config_or_default()?;
        debug!(
            ingest_max_nodes = cfg.ingest_max_nodes,
            traversal_max_depth = cfg.traversal_max_depth,
            "loaded graph-storage config"
        );

        let services = Arc::new(GraphServices::new(cfg));

        self.services
            .set(services.clone())
            .map_err(|_| anyhow::anyhow!("{} gear already initialized", Self::MODULE_NAME))?;

        // Publish the in-process client so other gears can consume the graph
        // without going through HTTP.
        ctx.client_hub()
            .register::<dyn GraphStorageClientV1>(Arc::new(GraphStorageLocalClient::new(services)));

        info!("graph-storage gear initialized");
        Ok(())
    }
}

impl RestApiCapability for GraphStorage {
    fn register_rest(
        &self,
        _ctx: &GearCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let services = self
            .services
            .get()
            .ok_or_else(|| anyhow::anyhow!("graph-storage services not initialized"))?
            .clone();

        Ok(routes::register_routes(router, openapi, services))
    }
}
