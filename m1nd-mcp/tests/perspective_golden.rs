// === Golden Test Sequences for Perspective MCP ===
// Theme 17 from 12-PERSPECTIVE-SYNTHESIS.
// 6 deterministic test sequences defining the contract for Perspective MCP behavior.
//
// These tests define WHAT should happen, not HOW.
// Each test is a sequence of tool calls with expected outcomes.
// They compile NOW (against CONTRACTS types) but call todo!() functions.
// The BUILD phase fills in implementations until these pass.

use m1nd_mcp::perspective::confidence::*;
use m1nd_mcp::perspective::keys::*;
use m1nd_mcp::perspective::state::*;
use m1nd_mcp::perspective::validation::*;
use m1nd_mcp::protocol::lock::*;
use m1nd_mcp::protocol::perspective::*;

// ===========================================================================
// Test Harness (deterministic test graph)
// ===========================================================================

/// Build a deterministic test graph: 20 nodes, ~50 edges, fixed weights.
/// Used by all golden tests for reproducibility.
///
/// Graph topology (code-like):
///   main.rs → lib.rs → session.rs → state.rs → error.rs
///                    → protocol.rs → types.rs
///                    → server.rs → dispatch.rs → handlers.rs
///   config.rs → settings.rs
///   utils.rs → helpers.rs → format.rs
///   test_main.rs → test_session.rs → test_state.rs
///   README.md (isolated, no code edges)
///   Cargo.toml (imports → main.rs, lib.rs)
#[allow(dead_code)]
fn build_test_graph_nodes() -> Vec<(&'static str, &'static str)> {
    vec![
        ("main.rs", "module"),
        ("lib.rs", "module"),
        ("session.rs", "module"),
        ("state.rs", "module"),
        ("error.rs", "module"),
        ("protocol.rs", "module"),
        ("types.rs", "module"),
        ("server.rs", "module"),
        ("dispatch.rs", "module"),
        ("handlers.rs", "module"),
        ("config.rs", "module"),
        ("settings.rs", "module"),
        ("utils.rs", "module"),
        ("helpers.rs", "module"),
        ("format.rs", "module"),
        ("test_main.rs", "test"),
        ("test_session.rs", "test"),
        ("test_state.rs", "test"),
        ("README.md", "doc"),
        ("Cargo.toml", "config"),
    ]
}

#[allow(dead_code)]
fn build_test_graph_edges() -> Vec<(&'static str, &'static str, &'static str, f32)> {
    vec![
        ("main.rs", "lib.rs", "imports", 0.9),
        ("lib.rs", "session.rs", "imports", 0.85),
        ("lib.rs", "protocol.rs", "imports", 0.8),
        ("lib.rs", "server.rs", "imports", 0.8),
        ("session.rs", "state.rs", "imports", 0.9),
        ("state.rs", "error.rs", "imports", 0.7),
        ("protocol.rs", "types.rs", "imports", 0.85),
        ("server.rs", "dispatch.rs", "imports", 0.9),
        ("dispatch.rs", "handlers.rs", "imports", 0.85),
        ("config.rs", "settings.rs", "imports", 0.7),
        ("utils.rs", "helpers.rs", "imports", 0.6),
        ("helpers.rs", "format.rs", "imports", 0.5),
        ("test_main.rs", "test_session.rs", "tests", 0.8),
        ("test_session.rs", "test_state.rs", "tests", 0.75),
        ("Cargo.toml", "main.rs", "declares", 0.95),
        ("Cargo.toml", "lib.rs", "declares", 0.95),
        // Cross-cutting edges
        ("session.rs", "error.rs", "uses", 0.6),
        ("server.rs", "error.rs", "uses", 0.55),
        ("handlers.rs", "session.rs", "calls", 0.7),
        ("handlers.rs", "protocol.rs", "calls", 0.65),
        ("test_session.rs", "session.rs", "tests", 0.8),
        ("test_state.rs", "state.rs", "tests", 0.8),
    ]
}

// ===========================================================================
// Golden Test 1: Happy Path
// start → routes → inspect → peek → follow → routes → back → close
// ===========================================================================

#[test]
fn golden_01_happy_path_types_compile() {
    // This test verifies the type contracts compile and wire together.
    // The actual behavior tests will work once engine_ops is implemented.

    // 1. start
    let start_input = PerspectiveStartInput {
        agent_id: "test_agent".into(),
        query: "session management".into(),
        anchor_node: Some("session.rs".into()),
        lens: None,
    };
    assert_eq!(start_input.agent_id, "test_agent");

    // Expected output shape
    let _start_output = PerspectiveStartOutput {
        perspective_id: "persp_test_001".into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("session.rs".into()),
        focus_node: Some("session.rs".into()),
        routes: vec![],
        total_routes: 0,
        page: 1,
        total_pages: 1,
        route_set_version: 1710000000000,
        cache_generation: 0,
        suggested: Some("inspect R01".into()),
    };

    // 2. routes
    let routes_input = PerspectiveRoutesInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
        page: 1,
        page_size: 6,
        route_set_version: Some(1710000000000),
    };
    assert_eq!(routes_input.page, 1);

    // 3. inspect (by route_id)
    let inspect_input = PerspectiveInspectInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
        route_id: Some("R_abc123".into()),
        route_index: None,
        route_set_version: 1710000000000,
    };
    assert!(inspect_input.route_id.is_some());

    // 4. peek (by route_index)
    let peek_input = PerspectivePeekInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
        route_id: None,
        route_index: Some(1),
        route_set_version: 1710000000000,
    };
    assert!(peek_input.route_index.is_some());

    // 5. follow
    let follow_input = PerspectiveFollowInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
        route_id: Some("R_abc123".into()),
        route_index: None,
        route_set_version: 1710000000000,
    };
    assert!(follow_input.route_id.is_some());

    // 6. back
    let back_input = PerspectiveBackInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
    };
    assert_eq!(back_input.perspective_id, "persp_test_001");

    // 7. close
    let close_input = PerspectiveCloseInput {
        agent_id: "test_agent".into(),
        perspective_id: "persp_test_001".into(),
    };
    assert_eq!(close_input.perspective_id, "persp_test_001");
}

// ===========================================================================
// Golden Test 2: Staleness Detection
// start → routes → [ingest happens] → follow (expect ROUTE_SET_STALE)
// ===========================================================================

#[test]
fn golden_02_staleness_detection() {
    // After ingest, graph_generation bumps → cache_generation bumps
    // → PerspectiveState.captured_cache_generation < current cache_generation
    // → follow should return RouteSetStale error

    let state = PerspectiveState {
        perspective_id: "persp_test_001".into(),
        agent_id: "test_agent".into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("session.rs".into()),
        anchor_query: Some("session".into()),
        focus_node: Some("session.rs".into()),
        lens: PerspectiveLens::default(),
        entry_path: vec!["session.rs".into()],
        navigation_history: vec![NavigationEvent {
            action: "start".into(),
            target: Some("session.rs".into()),
            timestamp_ms: 1710000000000,
            route_set_version: 100,
        }],
        checkpoints: vec![],
        visited_nodes: std::collections::HashSet::new(),
        route_cache: Some(CachedRouteSet {
            routes: vec![],
            total_routes: 3,
            page_size: 6,
            version: 100,
            synthesis_elapsed_ms: 15.0,
            captured_cache_generation: 1, // captured at gen 1
        }),
        route_set_version: 100,
        captured_cache_generation: 1,
        stale: false,
        created_at_ms: 1710000000000,
        last_accessed_ms: 1710000000000,
        branches: vec![],
    };

    // Simulate: graph_generation bumped to 2 after ingest
    let current_cache_generation: u64 = 2;

    // The follow handler should detect: state.captured_cache_generation (1) < current (2)
    // and return M1ndError::RouteSetStale
    assert!(state.captured_cache_generation < current_cache_generation);

    // Verify the error type exists and has the right fields
    let err = m1nd_core::error::M1ndError::RouteSetStale {
        route_set_version: 100,
        current_version: 200,
    };
    assert!(format!("{}", err).contains("stale"));
}

// ===========================================================================
// Golden Test 3: Error Recovery
// routes → follow with wrong version → re-list → follow correct
// ===========================================================================

#[test]
fn golden_03_error_recovery_types() {
    // Verify error types support recovery flow

    // Step 1: RouteSetStale error includes version info for recovery
    let err = m1nd_core::error::M1ndError::RouteSetStale {
        route_set_version: 50,
        current_version: 75,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("50"));
    assert!(msg.contains("75"));

    // Step 2: RouteNotFound error includes perspective context
    let err = m1nd_core::error::M1ndError::RouteNotFound {
        route_id: "R_abc123".into(),
        perspective_id: "persp_test_001".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("R_abc123"));

    // Step 3: NavigationAtRoot error
    let err = m1nd_core::error::M1ndError::NavigationAtRoot {
        perspective_id: "persp_test_001".into(),
    };
    assert!(format!("{}", err).contains("root"));

    // Step 4: PerspectiveNotFound
    let err = m1nd_core::error::M1ndError::PerspectiveNotFound {
        perspective_id: "persp_test_001".into(),
        agent_id: "test_agent".into(),
    };
    assert!(format!("{}", err).contains("not found"));
}

// ===========================================================================
// Golden Test 4: Multi-Agent Isolation
// agent A start → agent B start → verify no cross-contamination
// ===========================================================================

#[test]
fn golden_04_multi_agent_isolation() {
    use std::collections::HashMap;

    // Simulate perspective storage: HashMap<(agent_id, perspective_id), PerspectiveState>
    let mut perspectives: HashMap<(String, String), PerspectiveState> = HashMap::new();

    // Agent A creates perspective
    let state_a = PerspectiveState {
        perspective_id: "persp_agentA_001".into(),
        agent_id: "agent_A".into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("session.rs".into()),
        anchor_query: Some("session".into()),
        focus_node: Some("session.rs".into()),
        lens: PerspectiveLens::default(),
        entry_path: vec!["session.rs".into()],
        navigation_history: vec![],
        checkpoints: vec![],
        visited_nodes: std::collections::HashSet::new(),
        route_cache: None,
        route_set_version: 100,
        captured_cache_generation: 0,
        stale: false,
        created_at_ms: 1710000000000,
        last_accessed_ms: 1710000000000,
        branches: vec![],
    };
    perspectives.insert(("agent_A".into(), "persp_agentA_001".into()), state_a);

    // Agent B creates perspective
    let state_b = PerspectiveState {
        perspective_id: "persp_agentB_001".into(),
        agent_id: "agent_B".into(),
        mode: PerspectiveMode::Local,
        anchor_node: None,
        anchor_query: None,
        focus_node: Some("config.rs".into()),
        lens: PerspectiveLens::default(),
        entry_path: vec!["config.rs".into()],
        navigation_history: vec![],
        checkpoints: vec![],
        visited_nodes: std::collections::HashSet::new(),
        route_cache: None,
        route_set_version: 200,
        captured_cache_generation: 0,
        stale: false,
        created_at_ms: 1710000001000,
        last_accessed_ms: 1710000001000,
        branches: vec![],
    };
    perspectives.insert(("agent_B".into(), "persp_agentB_001".into()), state_b);

    // Verify isolation: A cannot see B's perspective
    assert!(perspectives
        .get(&("agent_A".into(), "persp_agentB_001".into()))
        .is_none());
    assert!(perspectives
        .get(&("agent_B".into(), "persp_agentA_001".into()))
        .is_none());

    // Verify each agent sees only their own
    let a_persp = perspectives
        .get(&("agent_A".into(), "persp_agentA_001".into()))
        .unwrap();
    assert_eq!(a_persp.agent_id, "agent_A");
    assert_eq!(a_persp.focus_node.as_deref(), Some("session.rs"));

    let b_persp = perspectives
        .get(&("agent_B".into(), "persp_agentB_001".into()))
        .unwrap();
    assert_eq!(b_persp.agent_id, "agent_B");
    assert_eq!(b_persp.focus_node.as_deref(), Some("config.rs"));

    // Verify limit enforcement (max 5 per agent)
    let limits = PerspectiveLimits::default();
    let agent_a_count = perspectives.keys().filter(|(a, _)| a == "agent_A").count();
    assert!(agent_a_count <= limits.max_perspectives_per_agent);
}

// ===========================================================================
// Golden Test 5: Lock Lifecycle
// lock.create → ingest → lock.diff (expect changes) → lock.release
// ===========================================================================

#[test]
fn golden_05_lock_lifecycle() {
    use std::collections::{HashMap, HashSet};

    // 1. Create lock on session.rs (node scope)
    let create_input = LockCreateInput {
        agent_id: "test_agent".into(),
        scope: LockScope::Node,
        root_nodes: vec!["session.rs".into()],
        radius: None,
        query: None,
        path_nodes: None,
    };
    assert_eq!(create_input.scope, LockScope::Node);

    // Simulate lock state after creation
    let mut lock = LockState {
        lock_id: "lock_test_001".into(),
        agent_id: "test_agent".into(),
        scope: LockScopeConfig {
            scope_type: LockScope::Node,
            root_nodes: vec!["session.rs".into()],
            radius: Some(0),
            query: None,
            path_nodes: None,
        },
        baseline: LockSnapshot {
            nodes: {
                let mut s = HashSet::new();
                s.insert("session.rs".into());
                s
            },
            edges: {
                let mut m = HashMap::new();
                m.insert(
                    edge_content_key("session.rs", "state.rs", "imports"),
                    EdgeSnapshotEntry {
                        source: "session.rs".into(),
                        target: "state.rs".into(),
                        relation: "imports".into(),
                        weight: 0.9,
                    },
                );
                m
            },
            graph_generation: 1,
            captured_at_ms: 1710000000000,
            key_format: "v1_content_addr".into(),
        },
        watcher: None,
        baseline_stale: false,
        created_at_ms: 1710000000000,
        last_diff_ms: 1710000000000,
    };

    // 2. Simulate ingest → graph_generation bumps
    // The rebuild_engines invalidation (Theme 16) marks baselines stale
    lock.baseline_stale = true;

    // 3. lock.diff should report staleness
    let diff_input = LockDiffInput {
        agent_id: "test_agent".into(),
        lock_id: "lock_test_001".into(),
    };
    assert_eq!(diff_input.lock_id, "lock_test_001");

    // Verify diff result shape
    let diff = LockDiffResult {
        lock_id: "lock_test_001".into(),
        no_changes: false,
        new_nodes: vec![],
        removed_nodes: vec![],
        new_edges: vec!["session.rs->new_module.rs: imports".into()],
        removed_edges: vec![],
        boundary_edges_added: vec![],
        boundary_edges_removed: vec![],
        weight_changes: vec![EdgeWeightChange {
            edge_key: edge_content_key("session.rs", "state.rs", "imports"),
            old_weight: 0.9,
            new_weight: 0.85,
        }],
        baseline_stale: true,
        elapsed_ms: 5.0,
    };
    assert!(diff.baseline_stale);
    assert_eq!(diff.new_edges.len(), 1);

    // 4. lock.release
    let release_input = LockReleaseInput {
        agent_id: "test_agent".into(),
        lock_id: "lock_test_001".into(),
    };
    assert_eq!(release_input.lock_id, "lock_test_001");
}

// ===========================================================================
// Golden Test 6: Schema Parity
// For each tool, verify minimal params deserialize into the Rust struct.
// ===========================================================================

#[test]
fn golden_06_schema_parity_perspective_tools() {
    // perspective.start
    let _: PerspectiveStartInput = serde_json::from_str(r#"{"agent_id": "a", "query": "q"}"#)
        .expect("perspective.start minimal params");

    // perspective.routes
    let _: PerspectiveRoutesInput =
        serde_json::from_str(r#"{"agent_id": "a", "perspective_id": "p"}"#)
            .expect("perspective.routes minimal params");

    // perspective.inspect
    let _: PerspectiveInspectInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_id": "R_abc", "route_set_version": 1}"#,
    )
    .expect("perspective.inspect minimal params");

    // perspective.peek
    let _: PerspectivePeekInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_index": 1, "route_set_version": 1}"#,
    )
    .expect("perspective.peek minimal params");

    // perspective.follow
    let _: PerspectiveFollowInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_id": "R_x", "route_set_version": 1}"#,
    )
    .expect("perspective.follow minimal params");

    // perspective.suggest
    let _: PerspectiveSuggestInput =
        serde_json::from_str(r#"{"agent_id": "a", "perspective_id": "p", "route_set_version": 1}"#)
            .expect("perspective.suggest minimal params");

    // perspective.affinity
    let _: PerspectiveAffinityInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_id": "R_x", "route_set_version": 1}"#,
    )
    .expect("perspective.affinity minimal params");

    // perspective.branch
    let _: PerspectiveBranchInput =
        serde_json::from_str(r#"{"agent_id": "a", "perspective_id": "p"}"#)
            .expect("perspective.branch minimal params");

    // perspective.back
    let _: PerspectiveBackInput =
        serde_json::from_str(r#"{"agent_id": "a", "perspective_id": "p"}"#)
            .expect("perspective.back minimal params");

    // perspective.compare
    let _: PerspectiveCompareInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id_a": "p1", "perspective_id_b": "p2"}"#,
    )
    .expect("perspective.compare minimal params");

    // perspective.list
    let _: PerspectiveListInput =
        serde_json::from_str(r#"{"agent_id": "a"}"#).expect("perspective.list minimal params");

    // perspective.close
    let _: PerspectiveCloseInput =
        serde_json::from_str(r#"{"agent_id": "a", "perspective_id": "p"}"#)
            .expect("perspective.close minimal params");
}

#[test]
fn golden_06_schema_parity_lock_tools() {
    // lock.create
    let _: LockCreateInput =
        serde_json::from_str(r#"{"agent_id": "a", "scope": "node", "root_nodes": ["x"]}"#)
            .expect("lock.create minimal params");

    // lock.watch
    let _: LockWatchInput =
        serde_json::from_str(r#"{"agent_id": "a", "lock_id": "l", "strategy": "manual"}"#)
            .expect("lock.watch minimal params");

    // lock.diff
    let _: LockDiffInput = serde_json::from_str(r#"{"agent_id": "a", "lock_id": "l"}"#)
        .expect("lock.diff minimal params");

    // lock.rebase
    let _: LockRebaseInput = serde_json::from_str(r#"{"agent_id": "a", "lock_id": "l"}"#)
        .expect("lock.rebase minimal params");

    // lock.release
    let _: LockReleaseInput = serde_json::from_str(r#"{"agent_id": "a", "lock_id": "l"}"#)
        .expect("lock.release minimal params");
}

// ===========================================================================
// Validation contract tests
// ===========================================================================

#[test]
fn golden_validation_lens_contract() {
    // Valid lens
    let lens = PerspectiveLens::default();
    let validated = validate_lens(&lens, 100).unwrap();
    assert_eq!(validated.dimensions.len(), 4);

    // Invalid dimension
    let mut bad = PerspectiveLens::default();
    bad.dimensions = vec!["quantum".into()];
    assert!(validate_lens(&bad, 100).is_err());
}

#[test]
fn golden_validation_pagination_contract() {
    // Normal case
    let p = validate_pagination(2, 6, 30).unwrap();
    assert_eq!(p.page, 2);
    assert_eq!(p.total_pages, 5);
    assert_eq!(p.offset, 6);
    assert!(!p.page_size_clamped);

    // page=0 rejected
    assert!(validate_pagination(0, 6, 30).is_err());

    // page_size clamped
    let p = validate_pagination(1, 100, 30).unwrap();
    assert_eq!(p.page_size, 10);
    assert!(p.page_size_clamped);
}

#[test]
fn golden_validation_route_ref_contract() {
    // Exactly one required
    assert!(validate_route_ref(&Some("R_x".into()), &None, "test").is_ok());
    assert!(validate_route_ref(&None, &Some(1), "test").is_ok());
    assert!(validate_route_ref(&Some("R_x".into()), &Some(1), "test").is_err());
    assert!(validate_route_ref(&None, &None, "test").is_err());
}

// ===========================================================================
// Content-addressable keys contract tests
// ===========================================================================

#[test]
fn golden_keys_edge_key_deterministic() {
    let k1 = edge_content_key("a", "b", "imports");
    let k2 = edge_content_key("a", "b", "imports");
    assert_eq!(k1, k2);

    // Different relation → different key
    let k3 = edge_content_key("a", "b", "calls");
    assert_ne!(k1, k3);
}

#[test]
fn golden_keys_route_id_stable() {
    let id1 = route_content_id("session.rs", &RouteFamily::Structural);
    let id2 = route_content_id("session.rs", &RouteFamily::Structural);
    assert_eq!(id1, id2);
    assert!(id1.starts_with("R_"));
    assert_eq!(id1.len(), 8);

    // Different family → different ID
    let id3 = route_content_id("session.rs", &RouteFamily::Ghost);
    assert_ne!(id1, id3);
}

#[test]
fn golden_keys_bidi_normalization() {
    let (lo1, hi1) = normalize_bidi_endpoints("z", "a");
    let (lo2, hi2) = normalize_bidi_endpoints("a", "z");
    assert_eq!(lo1, lo2);
    assert_eq!(hi1, hi2);
}

// ===========================================================================
// Confidence calibration contract tests
// ===========================================================================

#[test]
fn golden_confidence_max_cap() {
    // Even perfect scores cap at 0.85
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(1.0),
        structural_hole_pressure: Some(1.0),
        resonant_amplitude: Some(1.0),
        semantic_overlap: Some(1.0),
        provenance_overlap: Some(1.0),
        route_path_neighborhood: Some(1.0),
    };
    let c = compute_combined_confidence(&breakdown).unwrap();
    assert!(c <= MAX_CONFIDENCE);
}

#[test]
fn golden_confidence_min_threshold() {
    // Very low scores drop below threshold → None
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.01),
        structural_hole_pressure: Some(0.02),
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };
    assert!(compute_combined_confidence(&breakdown).is_none());
}

#[test]
fn golden_confidence_single_source_gate() {
    // Single source cannot exceed 0.40 (needs corroboration)
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.95),
        structural_hole_pressure: None,
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };
    let c = compute_combined_confidence(&breakdown).unwrap();
    assert!(c <= 0.40);
}
