// === m1nd-mcp/src/protocol/lock.rs ===
// Input/Output types for the 5 lock MCP tools.
// From 12-PERSPECTIVE-SYNTHESIS Themes 2, 4, 10, 14.

use serde::{Deserialize, Serialize};

use crate::perspective::state::{LockDiffResult, LockScope, LockState, WatchStrategy};

// ---------------------------------------------------------------------------
// lock.create (Theme 2, 14)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct LockCreateInput {
    pub agent_id: String,
    /// Scope type: node, subgraph, query_neighborhood, path.
    pub scope: LockScope,
    /// Root nodes for the lock scope. Non-empty required.
    pub root_nodes: Vec<String>,
    /// BFS radius for subgraph scope. Min 1, max 4.
    #[serde(default)]
    pub radius: Option<u32>,
    /// Query string for query_neighborhood scope.
    #[serde(default)]
    pub query: Option<String>,
    /// Ordered node list for path scope.
    #[serde(default)]
    pub path_nodes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LockCreateOutput {
    pub lock_id: String,
    pub scope: LockScope,
    pub baseline_nodes: usize,
    pub baseline_edges: usize,
    pub graph_generation: u64,
    pub created_at_ms: u64,
}

// ---------------------------------------------------------------------------
// lock.watch (Theme 10)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct LockWatchInput {
    pub agent_id: String,
    pub lock_id: String,
    /// Strategy: manual, on_ingest, on_learn. Periodic returns error.
    pub strategy: WatchStrategy,
}

#[derive(Clone, Debug, Serialize)]
pub struct LockWatchOutput {
    pub lock_id: String,
    pub strategy: WatchStrategy,
    /// Previous strategy if one was replaced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_strategy: Option<WatchStrategy>,
}

// ---------------------------------------------------------------------------
// lock.diff (Theme 14)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct LockDiffInput {
    pub agent_id: String,
    pub lock_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LockDiffOutput {
    pub diff: LockDiffResult,
    /// Whether watcher events were drained for this diff.
    pub watcher_events_drained: usize,
    /// If baseline_stale, suggest lock.rebase.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rebase_suggested: Option<String>,
}

// ---------------------------------------------------------------------------
// lock.rebase (Theme 14 — new tool)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct LockRebaseInput {
    pub agent_id: String,
    pub lock_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LockRebaseOutput {
    pub lock_id: String,
    pub previous_generation: u64,
    pub new_generation: u64,
    pub baseline_nodes: usize,
    pub baseline_edges: usize,
    /// Watcher config preserved across rebase.
    pub watcher_preserved: bool,
}

// ---------------------------------------------------------------------------
// lock.release (Theme 14)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Deserialize)]
pub struct LockReleaseInput {
    pub agent_id: String,
    pub lock_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LockReleaseOutput {
    pub lock_id: String,
    pub released: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_create_input_deserializes_node_scope() {
        let json = r#"{
            "agent_id": "jimi",
            "scope": "node",
            "root_nodes": ["session.rs"]
        }"#;
        let input: LockCreateInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.scope, LockScope::Node);
        assert_eq!(input.root_nodes.len(), 1);
        assert!(input.radius.is_none());
    }

    #[test]
    fn lock_create_input_deserializes_subgraph_scope() {
        let json = r#"{
            "agent_id": "jimi",
            "scope": "subgraph",
            "root_nodes": ["session.rs"],
            "radius": 2
        }"#;
        let input: LockCreateInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.scope, LockScope::Subgraph);
        assert_eq!(input.radius, Some(2));
    }

    #[test]
    fn lock_watch_input_deserializes() {
        let json = r#"{"agent_id": "jimi", "lock_id": "lock_jimi_001", "strategy": "on_ingest"}"#;
        let input: LockWatchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.strategy, WatchStrategy::OnIngest);
    }

    #[test]
    fn lock_diff_input_minimal() {
        let json = r#"{"agent_id": "jimi", "lock_id": "lock_jimi_001"}"#;
        let input: LockDiffInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.lock_id, "lock_jimi_001");
    }

    #[test]
    fn lock_release_input_minimal() {
        let json = r#"{"agent_id": "jimi", "lock_id": "lock_jimi_001"}"#;
        let input: LockReleaseInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.lock_id, "lock_jimi_001");
    }
}
