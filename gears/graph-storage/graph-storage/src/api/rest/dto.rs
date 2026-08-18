//! REST DTOs. Serialization lives here and nowhere else.

use graph_storage_sdk::GraphStats;

/// Coarse counters describing the caller's graph.
#[derive(Debug, Clone, Copy)]
#[toolkit_macros::api_dto(response)]
pub struct GraphStatsDto {
    /// Number of nodes visible to the caller.
    pub nodes: u64,
    /// Number of edges visible to the caller.
    pub edges: u64,
    /// Monotonic revision, bumped whenever stored state changes.
    pub graph_revision: u64,
}

impl From<GraphStats> for GraphStatsDto {
    fn from(value: GraphStats) -> Self {
        Self {
            nodes: value.nodes,
            edges: value.edges,
            graph_revision: value.graph_revision,
        }
    }
}

/// Result of a bounded neighbourhood expansion.
#[derive(Debug, Clone)]
#[toolkit_macros::api_dto(response)]
pub struct NeighboursDto {
    /// Node ids reachable from the seeds within the requested depth,
    /// restricted to what the caller is authorised to see.
    pub nodes: Vec<i64>,
    /// Whether the node budget truncated the result.
    pub truncated: bool,
}
