//! Domain services.
//!
//! Phase 2 carries a single read-only service backed by the secure ORM, so the
//! storage and authorization wiring can be exercised end to end. Ingest,
//! search, traversal and analytics land here as their layers are implemented.

use std::sync::Arc;

use graph_storage_sdk::GraphStats;
use toolkit_db::{DBProvider, DbError};
use toolkit_security::{AccessScope, SecurityContext};

use crate::config::GraphStorageConfig;
use crate::domain::error::DomainError;
use crate::infra::storage::counts;

/// Composition of all domain services used by the gear.
pub struct GraphServices {
    config: GraphStorageConfig,
    db: Arc<DBProvider<DbError>>,
}

impl GraphServices {
    /// Build the service composition from validated configuration.
    #[must_use]
    pub fn new(config: GraphStorageConfig, db: Arc<DBProvider<DbError>>) -> Self {
        Self { config, db }
    }

    /// Effective gear configuration.
    #[must_use]
    pub fn config(&self) -> &GraphStorageConfig {
        &self.config
    }

    /// Coarse counters for the caller's graph.
    ///
    /// The scope is derived from the caller's tenant. A PDP-issued scope
    /// replaces this once the policy-enforcement layer lands; the call site
    /// does not change, because the repository already takes an `AccessScope`.
    ///
    /// # Errors
    /// Returns [`DomainError::Storage`] when the query fails.
    pub async fn stats(&self, ctx: &SecurityContext) -> Result<GraphStats, DomainError> {
        let scope = AccessScope::for_tenant(ctx.subject_tenant_id());
        let conn = self
            .db
            .conn()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        counts::graph_stats(&conn, &scope).await
    }
}
