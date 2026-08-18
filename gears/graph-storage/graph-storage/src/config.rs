//! Gear configuration.
//!
//! Values mirror the Capacity and Admission Contract in `docs/DESIGN.md`:
//! every bound is a named key with a safe default. Hard-range validation is
//! added together with the admission layer.

use serde::{Deserialize, Serialize};

/// Configuration of the graph-storage gear.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct GraphStorageConfig {
    /// Maximum number of nodes accepted in one ingest batch.
    pub ingest_max_nodes: u32,
    /// Maximum number of edges accepted in one ingest batch.
    pub ingest_max_edges: u32,
    /// Maximum traversal depth accepted by the graph API.
    pub traversal_max_depth: u8,
    /// Default node budget of a traversal response.
    pub traversal_max_nodes: u32,
}

impl Default for GraphStorageConfig {
    fn default() -> Self {
        Self {
            ingest_max_nodes: 10_000,
            ingest_max_edges: 20_000,
            traversal_max_depth: 5,
            traversal_max_nodes: 1_000,
        }
    }
}
