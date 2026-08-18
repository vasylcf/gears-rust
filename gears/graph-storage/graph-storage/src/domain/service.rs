//! Domain services.
//!
//! Phase 1 carries a single read-only service so the wiring can be exercised
//! end to end. Ingest, search, traversal and analytics services land here as
//! the corresponding storage layers are implemented.

use graph_storage_sdk::GraphStats;
use toolkit_security::SecurityContext;

use crate::config::GraphStorageConfig;
use crate::domain::error::DomainError;

/// Composition of all domain services used by the gear.
#[derive(Debug)]
pub struct GraphServices {
    config: GraphStorageConfig,
}

impl GraphServices {
    /// Build the service composition from validated configuration.
    #[must_use]
    pub fn new(config: GraphStorageConfig) -> Self {
        Self { config }
    }

    /// Effective gear configuration.
    #[must_use]
    pub fn config(&self) -> &GraphStorageConfig {
        &self.config
    }

    /// Coarse counters for the caller's graph.
    ///
    /// Placeholder: storage is not wired yet, so an empty graph is reported.
    /// The signature already carries the security context and the fallible
    /// result the storage-backed implementation needs, so callers and the
    /// error mapping do not change when it lands.
    ///
    /// # Errors
    /// Returns [`DomainError`] once storage-backed counting is implemented.
    #[allow(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "placeholder body; storage-backed implementation lands with the ingest layer"
    )]
    pub fn stats(&self, _ctx: &SecurityContext) -> Result<GraphStats, DomainError> {
        Ok(GraphStats::default())
    }
}
