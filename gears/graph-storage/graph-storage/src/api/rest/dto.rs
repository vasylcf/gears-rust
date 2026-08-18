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
