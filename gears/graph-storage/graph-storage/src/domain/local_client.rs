//! In-process adapter implementing the SDK client trait over domain services.
//!
//! Registered in `ClientHub` so other gears can call graph-storage without
//! going through HTTP. It shares the same services — and, once the admission
//! and policy layers land, the same enforcement — as the REST surface.

use std::sync::Arc;

use async_trait::async_trait;
use graph_storage_sdk::{GraphStats, GraphStorageClientV1, GraphStorageError};
use toolkit_security::SecurityContext;

use crate::domain::service::GraphServices;

/// `ClientHub` adapter for in-process consumers.
pub struct GraphStorageLocalClient {
    services: Arc<GraphServices>,
}

impl GraphStorageLocalClient {
    /// Wrap the domain services in an object-safe client.
    #[must_use]
    pub fn new(services: Arc<GraphServices>) -> Self {
        Self { services }
    }
}

#[async_trait]
impl GraphStorageClientV1 for GraphStorageLocalClient {
    async fn stats(&self, ctx: &SecurityContext) -> Result<GraphStats, GraphStorageError> {
        Ok(self.services.stats(ctx).await?)
    }
}
