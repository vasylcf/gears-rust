//! Transport-agnostic models exposed by the graph-storage contract.

/// Coarse counters describing the current state of a tenant's graph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphStats {
    /// Number of nodes visible to the caller.
    pub nodes: u64,
    /// Number of edges visible to the caller.
    pub edges: u64,
    /// Monotonic revision, bumped whenever stored state changes.
    pub graph_revision: u64,
}
