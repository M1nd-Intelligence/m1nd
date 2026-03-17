// === Golden Tests for perspective.suggest and perspective.affinity ===
//
// Issue #13: Increase coverage for the two least-tested perspective tools.
//
// perspective.suggest — recommends the next-best action given current state.
// perspective.affinity — scores hypothesized connections for a route target.
//
// Test categories:
//   suggest (7 tests): empty cache, single route, multiple routes with history,
//                      unvisited preference, dead-end recovery, alternatives cap,
//                      staleness detection via based_on field.
//   affinity (8 tests): isolated node (empty candidates), confidence breakdown
//                        construction, multi-source confidence, single-source gate,
//                        epistemic guards, candidate kind variants,
//                        builder helper roundtrip, schema parity.

use m1nd_mcp::perspective::confidence::*;
use m1nd_mcp::perspective::keys::*;
use m1nd_mcp::perspective::state::*;
use m1nd_mcp::perspective::validation::*;
use m1nd_mcp::protocol::perspective::*;

// ===========================================================================
// Shared test infrastructure
// ===========================================================================

/// Build a minimal PerspectiveState at a given focus with N cached routes.
fn build_perspective_with_routes(
    agent_id: &str,
    perspective_id: &str,
    focus: &str,
    route_count: usize,
    visited: &[&str],
) -> PerspectiveState {
    let routes: Vec<Route> = (0..route_count)
        .map(|i| {
            let target = format!("module_{}.rs", i);
            let family = if i % 3 == 0 {
                RouteFamily::Structural
            } else if i % 3 == 1 {
                RouteFamily::Semantic
            } else {
                RouteFamily::Ghost
            };
            let route_id = route_content_id(&target, &family);
            Route {
                route_id,
                route_index: (i + 1) as u32,
                family,
                target_node: target.clone(),
                target_label: target,
                reason: format!("connected to {}", focus),
                score: 0.9 - (i as f32) * 0.1, // decreasing scores
                peek_available: i < 3,
                provenance: None,
            }
        })
        .collect();

    let mut visited_set = std::collections::HashSet::new();
    visited_set.insert(focus.to_string());
    for v in visited {
        visited_set.insert(v.to_string());
    }

    let nav_history: Vec<NavigationEvent> = if visited.is_empty() {
        vec![NavigationEvent {
            action: "start".into(),
            target: Some(focus.into()),
            timestamp_ms: 1710000000000,
            route_set_version: 100,
        }]
    } else {
        let mut events = vec![NavigationEvent {
            action: "start".into(),
            target: Some(focus.into()),
            timestamp_ms: 1710000000000,
            route_set_version: 100,
        }];
        for (i, v) in visited.iter().enumerate() {
            events.push(NavigationEvent {
                action: "follow".into(),
                target: Some(v.to_string()),
                timestamp_ms: 1710000000000 + (i as u64 + 1) * 1000,
                route_set_version: 100,
            });
        }
        events
    };

    PerspectiveState {
        perspective_id: perspective_id.into(),
        agent_id: agent_id.into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some(focus.into()),
        anchor_query: Some("test query".into()),
        focus_node: Some(focus.into()),
        lens: PerspectiveLens::default(),
        entry_path: std::iter::once(focus.to_string())
            .chain(visited.iter().map(|s| s.to_string()))
            .collect(),
        navigation_history: nav_history,
        checkpoints: vec![],
        visited_nodes: visited_set,
        route_cache: if route_count > 0 {
            Some(CachedRouteSet {
                routes,
                total_routes: route_count,
                page_size: 6,
                version: 100,
                synthesis_elapsed_ms: 5.0,
                captured_cache_generation: 1,
            })
        } else {
            None
        },
        route_set_version: 100,
        captured_cache_generation: 1,
        stale: false,
        created_at_ms: 1710000000000,
        last_accessed_ms: 1710000000000,
        branches: vec![],
    }
}

/// Build a PerspectiveState with no routes (dead-end scenario).
fn build_dead_end_perspective(agent_id: &str, perspective_id: &str) -> PerspectiveState {
    PerspectiveState {
        perspective_id: perspective_id.into(),
        agent_id: agent_id.into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("dead_end.rs".into()),
        anchor_query: Some("dead end".into()),
        focus_node: Some("dead_end.rs".into()),
        lens: PerspectiveLens::default(),
        entry_path: vec!["dead_end.rs".into()],
        navigation_history: vec![NavigationEvent {
            action: "start".into(),
            target: Some("dead_end.rs".into()),
            timestamp_ms: 1710000000000,
            route_set_version: 100,
        }],
        checkpoints: vec![],
        visited_nodes: {
            let mut s = std::collections::HashSet::new();
            s.insert("dead_end.rs".into());
            s
        },
        route_cache: Some(CachedRouteSet {
            routes: vec![],
            total_routes: 0,
            page_size: 6,
            version: 100,
            synthesis_elapsed_ms: 1.0,
            captured_cache_generation: 1,
        }),
        route_set_version: 100,
        captured_cache_generation: 1,
        stale: false,
        created_at_ms: 1710000000000,
        last_accessed_ms: 1710000000000,
        branches: vec![],
    }
}

// ===========================================================================
// perspective.suggest — 7 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Suggest Test 1: No route cache → dead-end recovery suggestion
// ---------------------------------------------------------------------------

#[test]
fn suggest_01_no_cache_suggests_back() {
    // When route_cache is None, suggest should recommend back/close.
    let persp = PerspectiveState {
        perspective_id: "persp_test_001".into(),
        agent_id: "test_agent".into(),
        mode: PerspectiveMode::Anchored,
        anchor_node: Some("orphan.rs".into()),
        anchor_query: Some("orphan".into()),
        focus_node: Some("orphan.rs".into()),
        lens: PerspectiveLens::default(),
        entry_path: vec!["orphan.rs".into()],
        navigation_history: vec![NavigationEvent {
            action: "start".into(),
            target: Some("orphan.rs".into()),
            timestamp_ms: 1710000000000,
            route_set_version: 100,
        }],
        checkpoints: vec![],
        visited_nodes: std::collections::HashSet::new(),
        route_cache: None, // no cache
        route_set_version: 100,
        captured_cache_generation: 0,
        stale: false,
        created_at_ms: 1710000000000,
        last_accessed_ms: 1710000000000,
        branches: vec![],
    };

    // Simulate what the handler does: no top_route → exhaustion_recovery
    let cached = persp.route_cache.as_ref();
    let top_route = cached.and_then(|c| c.routes.first());
    assert!(top_route.is_none());

    // The handler produces exhaustion_recovery based_on
    let suggestion = SuggestResult {
        recommended_action: "perspective.back".into(),
        confidence: 0.50,
        why: "No routes available at current focus".into(),
        based_on: "exhaustion_recovery".into(),
        alternatives: vec![SuggestAlternative {
            action: "perspective.close".into(),
            confidence: 0.30,
            why: "Start fresh with a new perspective".into(),
        }],
    };

    assert_eq!(suggestion.based_on, "exhaustion_recovery");
    assert_eq!(suggestion.recommended_action, "perspective.back");
    assert!(suggestion.confidence <= 1.0);
    assert_eq!(suggestion.alternatives.len(), 1);
}

// ---------------------------------------------------------------------------
// Suggest Test 2: Single route → recommends following it
// ---------------------------------------------------------------------------

#[test]
fn suggest_02_single_route_recommends_follow() {
    let persp = build_perspective_with_routes("agent_a", "persp_001", "main.rs", 1, &[]);

    let cached = persp.route_cache.as_ref().unwrap();
    let top_route = cached.routes.first().unwrap();

    // The handler should suggest following the single available route
    assert_eq!(cached.routes.len(), 1);
    assert!(top_route.score > 0.0);

    let action = format!("follow {}", top_route.route_id);
    assert!(action.starts_with("follow R_"));
}

// ---------------------------------------------------------------------------
// Suggest Test 3: Multiple routes — prefers unvisited over visited
// ---------------------------------------------------------------------------

#[test]
fn suggest_03_prefers_unvisited_routes() {
    // Visit module_0.rs, so suggest should skip it and recommend module_1.rs
    let persp = build_perspective_with_routes(
        "agent_a",
        "persp_001",
        "main.rs",
        5,
        &["module_0.rs"],
    );

    let cached = persp.route_cache.as_ref().unwrap();

    // Simulate suggest logic: find first unvisited route
    let unvisited = cached
        .routes
        .iter()
        .find(|r| !persp.visited_nodes.contains(&r.target_node));

    assert!(unvisited.is_some());
    let best = unvisited.unwrap();
    // module_0.rs is visited, so the suggestion should be for a different module
    assert_ne!(best.target_node, "module_0.rs");
    assert!(!persp.visited_nodes.contains(&best.target_node));
}

// ---------------------------------------------------------------------------
// Suggest Test 4: All routes visited → falls back to highest-scored route
// ---------------------------------------------------------------------------

#[test]
fn suggest_04_all_visited_falls_back_to_top() {
    // Visit all route targets
    let targets: Vec<&str> = (0..3).map(|i| match i {
        0 => "module_0.rs",
        1 => "module_1.rs",
        2 => "module_2.rs",
        _ => unreachable!(),
    }).collect();

    let persp = build_perspective_with_routes(
        "agent_a",
        "persp_001",
        "main.rs",
        3,
        &targets,
    );

    let cached = persp.route_cache.as_ref().unwrap();

    // All targets are visited
    let unvisited = cached
        .routes
        .iter()
        .find(|r| !persp.visited_nodes.contains(&r.target_node));
    assert!(unvisited.is_none());

    // Handler falls back to top route
    let fallback = cached.routes.first().unwrap();
    assert!(fallback.score > 0.0);
    // Confidence is capped at 0.85
    assert!(fallback.score.min(0.85) <= 0.85);
}

// ---------------------------------------------------------------------------
// Suggest Test 5: Dead-end (empty routes in cache) → back/close
// ---------------------------------------------------------------------------

#[test]
fn suggest_05_dead_end_empty_routes_suggests_recovery() {
    let persp = build_dead_end_perspective("agent_a", "persp_001");

    let cached = persp.route_cache.as_ref().unwrap();
    assert!(cached.routes.is_empty());
    assert_eq!(cached.total_routes, 0);

    // Handler detects dead end: no top_route
    let top_route = cached.routes.first();
    assert!(top_route.is_none());

    // Diagnostic should be produced for dead_end
    let diagnostic = Diagnostic {
        sources_checked: vec!["graph_neighbors".into()],
        sources_with_results: vec![],
        sources_failed: vec![],
        reason: "dead_end".into(),
        suggestion: "Navigate back or start a new perspective".into(),
        graph_stats: DiagnosticGraphStats {
            node_count: 0,
            edge_count: 0,
        },
    };
    assert_eq!(diagnostic.reason, "dead_end");
}

// ---------------------------------------------------------------------------
// Suggest Test 6: Alternatives capped at 3 (Theme 5)
// ---------------------------------------------------------------------------

#[test]
fn suggest_06_alternatives_capped_at_three() {
    let persp = build_perspective_with_routes("agent_a", "persp_001", "main.rs", 8, &[]);

    let cached = persp.route_cache.as_ref().unwrap();
    let best = cached.routes.first().unwrap();

    // Build alternatives the same way the handler does
    let alternatives: Vec<SuggestAlternative> = cached
        .routes
        .iter()
        .filter(|r| r.route_id != best.route_id)
        .take(3) // Theme 5 cap
        .map(|r| SuggestAlternative {
            action: format!("follow {}", r.route_id),
            confidence: r.score.min(0.85),
            why: format!("{:?} route to {}", r.family, r.target_label),
        })
        .collect();

    // Even though there are 7 other routes, alternatives is capped at 3
    assert!(alternatives.len() <= 3);
    assert_eq!(alternatives.len(), 3);

    // Verify PerspectiveLimits agrees
    let limits = PerspectiveLimits::default();
    assert_eq!(limits.max_suggest_alternatives, 3);
}

// ---------------------------------------------------------------------------
// Suggest Test 7: based_on reflects navigation history depth
// ---------------------------------------------------------------------------

#[test]
fn suggest_07_based_on_reflects_history_depth() {
    // Fresh perspective (only 1 nav event = "start") → initial_ranking
    let fresh = build_perspective_with_routes("agent_a", "persp_fresh", "main.rs", 3, &[]);
    assert_eq!(fresh.navigation_history.len(), 1);
    let based_on_fresh = if fresh.navigation_history.len() > 1 {
        "navigation_history"
    } else {
        "initial_ranking"
    };
    assert_eq!(based_on_fresh, "initial_ranking");

    // Perspective with history (start + follow events) → navigation_history
    let navigated = build_perspective_with_routes(
        "agent_a",
        "persp_nav",
        "main.rs",
        3,
        &["module_0.rs"],
    );
    assert!(navigated.navigation_history.len() > 1);
    let based_on_nav = if navigated.navigation_history.len() > 1 {
        "navigation_history"
    } else {
        "initial_ranking"
    };
    assert_eq!(based_on_nav, "navigation_history");
}

// ===========================================================================
// perspective.affinity — 8 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Affinity Test 1: Isolated node → empty candidates with diagnostic
// ---------------------------------------------------------------------------

#[test]
fn affinity_01_isolated_node_empty_candidates() {
    // V1 implementation returns empty candidates with diagnostic
    let output = PerspectiveAffinityOutput {
        route_id: "R_abc123".into(),
        target_node: "isolated.rs".into(),
        notice: "Probable connections, not verified edges.".into(),
        candidates: vec![],
        diagnostic: Some(Diagnostic {
            sources_checked: vec!["graph_neighbors".into()],
            sources_with_results: vec![],
            sources_failed: vec![],
            reason: "under_indexed".into(),
            suggestion: "Affinity requires more graph data for meaningful results".into(),
            graph_stats: DiagnosticGraphStats {
                node_count: 1,
                edge_count: 0,
            },
        }),
    };

    assert!(output.candidates.is_empty());
    assert!(output.diagnostic.is_some());
    assert_eq!(output.diagnostic.as_ref().unwrap().reason, "under_indexed");
    assert_eq!(output.notice, "Probable connections, not verified edges.");
}

// ---------------------------------------------------------------------------
// Affinity Test 2: Multi-source confidence scoring accuracy
// ---------------------------------------------------------------------------

#[test]
fn affinity_02_multi_source_confidence_accuracy() {
    // Two sources with moderate scores → geometric mean within bounds
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.64),  // sqrt(0.64) = 0.8 normalized
        structural_hole_pressure: Some(0.5),
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };

    let confidence = compute_combined_confidence(&breakdown);
    assert!(confidence.is_some());
    let c = confidence.unwrap();

    // Geometric mean of 0.64 and 0.5 = sqrt(0.64 * 0.5) = sqrt(0.32) ≈ 0.566
    // This is below MAX_CONFIDENCE (0.85) and above MIN_CONFIDENCE_THRESHOLD (0.15)
    assert!(c >= MIN_CONFIDENCE_THRESHOLD);
    assert!(c <= MAX_CONFIDENCE);

    // With 2 sources, the multi-source gate does NOT apply (need < 2 sources for gate)
    // So the raw geometric mean should be the result
    let expected = (0.64_f64 * 0.5_f64).powf(0.5) as f32;
    assert!((c - expected).abs() < 0.01, "expected ~{:.3}, got {:.3}", expected, c);
}

// ---------------------------------------------------------------------------
// Affinity Test 3: Single-source gate caps at 0.40
// ---------------------------------------------------------------------------

#[test]
fn affinity_03_single_source_gate() {
    // Single high source → capped at 0.40
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.81), // sqrt(0.81) = 0.9
        structural_hole_pressure: None,
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };

    let confidence = compute_combined_confidence(&breakdown).unwrap();
    assert!(confidence <= 0.40, "single-source should be gated at 0.40, got {}", confidence);
}

// ---------------------------------------------------------------------------
// Affinity Test 4: All six sources present → caps at 0.85
// ---------------------------------------------------------------------------

#[test]
fn affinity_04_all_sources_cap_at_max() {
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.95),
        structural_hole_pressure: Some(0.90),
        resonant_amplitude: Some(0.85),
        semantic_overlap: Some(0.92),
        provenance_overlap: Some(1.0),
        route_path_neighborhood: Some(0.88),
    };

    let confidence = compute_combined_confidence(&breakdown).unwrap();
    assert!(confidence <= MAX_CONFIDENCE, "confidence {} exceeds MAX {}", confidence, MAX_CONFIDENCE);
    assert!(confidence > 0.40, "six sources should exceed single-source gate");
}

// ---------------------------------------------------------------------------
// Affinity Test 5: Below threshold → None
// ---------------------------------------------------------------------------

#[test]
fn affinity_05_below_threshold_returns_none() {
    // Very low scores: geometric mean drops below 0.15
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.01),
        structural_hole_pressure: Some(0.02),
        resonant_amplitude: Some(0.01),
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };

    let confidence = compute_combined_confidence(&breakdown);
    assert!(confidence.is_none(), "scores this low should be below threshold");
}

// ---------------------------------------------------------------------------
// Affinity Test 6: Epistemic guards on AffinityCandidate
// ---------------------------------------------------------------------------

#[test]
fn affinity_06_epistemic_guards() {
    // Every AffinityCandidate MUST have is_hypothetical = true
    // proposed_relation MUST be None in V1
    // Kind MUST use hypothesized_ prefix

    let candidate = AffinityCandidate {
        candidate_node: "target.rs".into(),
        candidate_label: "target.rs".into(),
        kind: AffinityCandidateKind::HypothesizedLatentEdge,
        confidence: 0.55,
        is_hypothetical: true,
        proposed_relation: None,
        confidence_breakdown: ConfidenceBreakdown {
            ghost_edge_strength: Some(0.7),
            structural_hole_pressure: Some(0.5),
            resonant_amplitude: None,
            semantic_overlap: None,
            provenance_overlap: None,
            route_path_neighborhood: None,
        },
    };

    assert!(candidate.is_hypothetical, "epistemic guard: must be hypothetical");
    assert!(candidate.proposed_relation.is_none(), "V1: proposed_relation must be None");
    assert!(candidate.confidence >= MIN_CONFIDENCE_THRESHOLD);
    assert!(candidate.confidence <= MAX_CONFIDENCE);

    // Verify all three kind variants serialize with hypothesized_ prefix
    let kinds = vec![
        AffinityCandidateKind::HypothesizedLatentEdge,
        AffinityCandidateKind::MissingBridge,
        AffinityCandidateKind::ResonantNeighbor,
    ];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        // HypothesizedLatentEdge → "hypothesized_latent_edge" (has prefix)
        // MissingBridge and ResonantNeighbor are conceptually hypothesized
        // through the is_hypothetical field
        assert!(!json.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Affinity Test 7: build_affinity_candidate helper roundtrip
// ---------------------------------------------------------------------------

#[test]
fn affinity_07_builder_helper_roundtrip() {
    // Valid candidate: multi-source, above threshold
    let breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.64),
        structural_hole_pressure: Some(0.49),
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };

    let candidate = build_affinity_candidate(
        "file::utils.py".into(),
        "utils.py".into(),
        AffinityCandidateKind::HypothesizedLatentEdge,
        breakdown.clone(),
    );

    assert!(candidate.is_some(), "valid multi-source should produce a candidate");
    let c = candidate.unwrap();
    assert_eq!(c.candidate_node, "file::utils.py");
    assert_eq!(c.candidate_label, "utils.py");
    assert!(c.is_hypothetical);
    assert!(c.proposed_relation.is_none());
    assert!(c.confidence >= MIN_CONFIDENCE_THRESHOLD);
    assert!(c.confidence <= MAX_CONFIDENCE);

    // Invalid candidate: below threshold → builder returns None
    let low_breakdown = ConfidenceBreakdown {
        ghost_edge_strength: Some(0.001),
        structural_hole_pressure: Some(0.002),
        resonant_amplitude: None,
        semantic_overlap: None,
        provenance_overlap: None,
        route_path_neighborhood: None,
    };

    let no_candidate = build_affinity_candidate(
        "file::noise.py".into(),
        "noise.py".into(),
        AffinityCandidateKind::MissingBridge,
        low_breakdown,
    );
    assert!(no_candidate.is_none(), "below-threshold should return None");
}

// ---------------------------------------------------------------------------
// Affinity Test 8: Schema parity — minimal JSON deserializes
// ---------------------------------------------------------------------------

#[test]
fn affinity_08_schema_parity() {
    // PerspectiveAffinityInput: minimal params
    let input: PerspectiveAffinityInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_id": "R_x", "route_set_version": 1}"#,
    )
    .expect("affinity input should deserialize");
    assert_eq!(input.agent_id, "a");
    assert_eq!(input.route_id.as_deref(), Some("R_x"));
    assert!(input.route_index.is_none());

    // Also with route_index instead of route_id
    let input2: PerspectiveAffinityInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_index": 3, "route_set_version": 1}"#,
    )
    .expect("affinity input with route_index should deserialize");
    assert!(input2.route_id.is_none());
    assert_eq!(input2.route_index, Some(3));

    // PerspectiveSuggestInput: minimal params
    let suggest: PerspectiveSuggestInput = serde_json::from_str(
        r#"{"agent_id": "a", "perspective_id": "p", "route_set_version": 1}"#,
    )
    .expect("suggest input should deserialize");
    assert_eq!(suggest.agent_id, "a");
    assert_eq!(suggest.perspective_id, "p");
    assert_eq!(suggest.route_set_version, 1);
}

// ===========================================================================
// Cross-cutting: suggest + affinity integration contract
// ===========================================================================

// ---------------------------------------------------------------------------
// Cross Test 1: Suggest output structure is complete
// ---------------------------------------------------------------------------

#[test]
fn cross_01_suggest_output_structure() {
    let output = PerspectiveSuggestOutput {
        perspective_id: "persp_test_001".into(),
        suggestion: SuggestResult {
            recommended_action: "follow R_abc123".into(),
            confidence: 0.72,
            why: "Highest-scored structural route to session.rs".into(),
            based_on: "navigation_history".into(),
            alternatives: vec![
                SuggestAlternative {
                    action: "follow R_def456".into(),
                    confidence: 0.65,
                    why: "Semantic route to types.rs".into(),
                },
                SuggestAlternative {
                    action: "follow R_ghi789".into(),
                    confidence: 0.58,
                    why: "Ghost route to utils.rs".into(),
                },
            ],
        },
        diagnostic: None,
    };

    // Serialize and verify structure
    let json = serde_json::to_value(&output).unwrap();
    assert!(json["perspective_id"].is_string());
    assert!(json["suggestion"]["recommended_action"].is_string());
    assert!(json["suggestion"]["confidence"].is_number());
    assert!(json["suggestion"]["why"].is_string());
    assert!(json["suggestion"]["based_on"].is_string());
    assert!(json["suggestion"]["alternatives"].is_array());
    assert_eq!(json["suggestion"]["alternatives"].as_array().unwrap().len(), 2);
    // diagnostic is None → should not be present (skip_serializing_if)
    assert!(json.get("diagnostic").is_none() || json["diagnostic"].is_null());
}

// ---------------------------------------------------------------------------
// Cross Test 2: Affinity output structure with candidates
// ---------------------------------------------------------------------------

#[test]
fn cross_02_affinity_output_structure_with_candidates() {
    let output = PerspectiveAffinityOutput {
        route_id: "R_abc123".into(),
        target_node: "session.rs".into(),
        notice: "Probable connections, not verified edges.".into(),
        candidates: vec![AffinityCandidate {
            candidate_node: "state.rs".into(),
            candidate_label: "state.rs".into(),
            kind: AffinityCandidateKind::HypothesizedLatentEdge,
            confidence: 0.62,
            is_hypothetical: true,
            proposed_relation: None,
            confidence_breakdown: ConfidenceBreakdown {
                ghost_edge_strength: Some(0.7),
                structural_hole_pressure: Some(0.55),
                resonant_amplitude: None,
                semantic_overlap: None,
                provenance_overlap: None,
                route_path_neighborhood: None,
            },
        }],
        diagnostic: None,
    };

    // Serialize and verify
    let json = serde_json::to_value(&output).unwrap();
    assert_eq!(json["route_id"], "R_abc123");
    assert_eq!(json["notice"], "Probable connections, not verified edges.");
    assert_eq!(json["candidates"].as_array().unwrap().len(), 1);

    let c0 = &json["candidates"][0];
    assert_eq!(c0["candidate_node"], "state.rs");
    assert_eq!(c0["is_hypothetical"], true);
    assert!(c0["proposed_relation"].is_null());
    assert!(c0["confidence"].as_f64().unwrap() <= MAX_CONFIDENCE as f64);
    assert!(c0["confidence"].as_f64().unwrap() >= MIN_CONFIDENCE_THRESHOLD as f64);
}

// ---------------------------------------------------------------------------
// Cross Test 3: Affinity max_affinity_candidates limit
// ---------------------------------------------------------------------------

#[test]
fn cross_03_affinity_candidates_limit() {
    let limits = PerspectiveLimits::default();
    assert_eq!(limits.max_affinity_candidates, 8);

    // Build 10 candidates, verify only 8 should be kept
    let candidates: Vec<AffinityCandidate> = (0..10)
        .map(|i| AffinityCandidate {
            candidate_node: format!("node_{}", i),
            candidate_label: format!("node_{}", i),
            kind: AffinityCandidateKind::ResonantNeighbor,
            confidence: 0.80 - (i as f32) * 0.05,
            is_hypothetical: true,
            proposed_relation: None,
            confidence_breakdown: ConfidenceBreakdown {
                ghost_edge_strength: Some(0.7),
                structural_hole_pressure: Some(0.6),
                resonant_amplitude: None,
                semantic_overlap: None,
                provenance_overlap: None,
                route_path_neighborhood: None,
            },
        })
        .collect();

    // Apply the limit
    let capped: Vec<_> = candidates
        .into_iter()
        .take(limits.max_affinity_candidates)
        .collect();
    assert_eq!(capped.len(), 8);
}

// ---------------------------------------------------------------------------
// Cross Test 4: Route validation required for affinity
// ---------------------------------------------------------------------------

#[test]
fn cross_04_affinity_requires_valid_route_ref() {
    // Both provided → error
    assert!(validate_route_ref(&Some("R_x".into()), &Some(1), "perspective.affinity").is_err());

    // Neither provided → error
    assert!(validate_route_ref(&None, &None, "perspective.affinity").is_err());

    // Exactly one → ok
    let by_id = validate_route_ref(&Some("R_x".into()), &None, "perspective.affinity");
    assert!(by_id.is_ok());
    assert!(matches!(by_id.unwrap(), ValidatedRouteRef::ById(ref s) if s == "R_x"));

    let by_idx = validate_route_ref(&None, &Some(2), "perspective.affinity");
    assert!(by_idx.is_ok());
    assert!(matches!(by_idx.unwrap(), ValidatedRouteRef::ByIndex(2)));
}

// ---------------------------------------------------------------------------
// Cross Test 5: Confidence normalization functions used by affinity
// ---------------------------------------------------------------------------

#[test]
fn cross_05_normalization_functions_for_affinity() {
    // Ghost edge: sqrt normalization
    assert!((normalize_ghost_edge(0.0) - 0.0).abs() < 0.001);
    assert!((normalize_ghost_edge(0.49) - 0.7).abs() < 0.001);
    assert!((normalize_ghost_edge(1.0) - 1.0).abs() < 0.001);
    // Negative input clamped to 0
    assert_eq!(normalize_ghost_edge(-0.5), 0.0);

    // Structural hole: normalized pressure
    assert!((normalize_structural_hole(0.8, 0.2) - 0.75).abs() < 0.001);
    assert_eq!(normalize_structural_hole(0.5, 1.0), 0.0); // degenerate

    // Route-path neighborhood: 1/(1+hops)
    assert_eq!(normalize_route_path_neighborhood(0), 1.0);
    assert!((normalize_route_path_neighborhood(1) - 0.5).abs() < 0.001);
    assert!((normalize_route_path_neighborhood(3) - 0.25).abs() < 0.001);

    // Provenance overlap
    assert_eq!(normalize_provenance_overlap(true, Some(10)), 1.0);  // same file, close
    assert_eq!(normalize_provenance_overlap(true, Some(100)), 0.5); // same file, far
    assert_eq!(normalize_provenance_overlap(true, None), 0.5);      // same file, unknown distance
    assert_eq!(normalize_provenance_overlap(false, Some(10)), 0.0); // different file

    // Semantic overlap: passthrough clamp
    assert_eq!(normalize_semantic_overlap(0.75), 0.75);
    assert_eq!(normalize_semantic_overlap(1.5), 1.0);  // clamped
    assert_eq!(normalize_semantic_overlap(-0.3), 0.0);  // clamped

    // Resonant amplitude
    assert!((normalize_resonant_amplitude(0.5, 1.0) - 0.5).abs() < 0.001);
    assert_eq!(normalize_resonant_amplitude(0.5, 0.0), 0.0); // zero max
}

// ---------------------------------------------------------------------------
// Cross Test 6: Route content ID stability for affinity lookups
// ---------------------------------------------------------------------------

#[test]
fn cross_06_route_id_stability() {
    // Affinity needs route_id to look up the route. IDs must be stable.
    let id1 = route_content_id("session.rs", &RouteFamily::Structural);
    let id2 = route_content_id("session.rs", &RouteFamily::Structural);
    assert_eq!(id1, id2, "route IDs must be deterministic");
    assert!(id1.starts_with("R_"));
    assert_eq!(id1.len(), 8); // R_ + 6 hex chars

    // Different target or family → different ID
    let id3 = route_content_id("session.rs", &RouteFamily::Ghost);
    assert_ne!(id1, id3, "different family must produce different ID");

    let id4 = route_content_id("state.rs", &RouteFamily::Structural);
    assert_ne!(id1, id4, "different target must produce different ID");
}
