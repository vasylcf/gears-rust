//! Domain error type.
//!
//! The mapping to `CanonicalError` lives at the API boundary
//! (`api::rest::error`), per the Error Model section of `docs/DESIGN.md`.

use thiserror::Error;

/// Errors produced by the domain layer.
#[derive(Debug, Error)]
pub enum DomainError {
    /// The gear is not fully initialised yet.
    #[error("graph-storage service is not initialised")]
    NotInitialised,
    /// A storage operation failed.
    #[error("storage failure: {0}")]
    Storage(String),
}
