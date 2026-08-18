//! Graph Storage gear.
//!
//! Stores a typed, multi-tenant knowledge graph and serves lexical, vector and
//! graph queries over it. See `gears/graph-storage/docs/` for the PRD, DESIGN
//! and the architecture decisions this implementation follows.
//!
//! Layering (one-way dependencies, `api` -> `domain` -> `infra`):
//!
//! * `graph-storage-sdk` — public contract, no transport or storage types;
//! * `api` — REST surface: DTOs, handlers, route registration;
//! * `domain` — services and ports, free of infrastructure types;
//! * `gear` — the composition root that wires everything together.

// === PUBLIC API (from SDK) ===
pub use graph_storage_sdk::{GraphStats, GraphStorageClientV1, GraphStorageError};

// === GEAR ENTRY POINT ===
pub mod gear;
pub use gear::GraphStorage;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
