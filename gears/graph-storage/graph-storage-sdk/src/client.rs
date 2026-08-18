//! Object-safe client trait registered in `ClientHub`.

use async_trait::async_trait;
use toolkit_security::SecurityContext;

use crate::{GraphStats, GraphStorageError};

/// Object-safe client for in-process consumption by other gears (version 1).
#[async_trait]
pub trait GraphStorageClientV1: Send + Sync {
    /// Return coarse counters for the caller's graph.
    async fn stats(&self, ctx: &SecurityContext) -> Result<GraphStats, GraphStorageError>;
}
