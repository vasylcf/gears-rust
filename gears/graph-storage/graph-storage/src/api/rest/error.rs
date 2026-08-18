//! Boundary mapping from `DomainError` to the canonical error envelope.
//!
//! This is the single authoritative mapping: both the REST adapter and the
//! in-process client surface the same category for the same failure, as the
//! Error Model section of `docs/DESIGN.md` requires.

use toolkit::api::canonical_prelude::*;

use crate::domain::error::DomainError;

#[resource_error(gts_id!("cf.core.kg.node.v1~"))]
struct GraphResourceError;

impl From<DomainError> for CanonicalError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Storage(_) => GraphResourceError::unknown(err.to_string()).create(),
            DomainError::NotInitialised => GraphResourceError::failed_precondition()
                .with_precondition_violation(
                    "graph-storage",
                    err.to_string(),
                    "SERVICE_NOT_INITIALISED",
                )
                .create(),
        }
    }
}
