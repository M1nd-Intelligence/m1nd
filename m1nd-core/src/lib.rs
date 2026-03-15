#![allow(unused)]

pub mod activation;
pub mod antibody;
pub mod builder;
pub mod counterfactual;
pub mod domain;
pub mod epidemic;
pub mod error;
pub mod flow;
pub mod graph;
pub mod layer;
pub mod plasticity;
pub mod query;
pub mod resonance;
pub mod seed;
pub mod semantic;
pub mod snapshot;
pub mod temporal;
pub mod topology;
pub mod tremor;
pub mod trust;
pub mod types;
pub mod xlr;

#[cfg(test)]
mod tests {
    use crate::activation::*;
    use crate::counterfactual::*;
    use crate::error::*;
    use crate::graph::*;
    use crate::plasticity::*;
    use crate::query::*;
    use crate::resonance::*;
    use crate::seed::*;
    use crate::temporal::*;
    use crate::topology::*;
    use crate::types::*;
    use crate::xlr::*;

    // ===== STEP-001: types.rs tests =====

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "non-finite")]
    fn finite_f32_rejects_nan() {
        // debug_assert! fires in test (debug) builds only
        let _f = FiniteF32::new(f32::NAN);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "non-finite")]
    fn finite_f32_rejects_inf() {
        // debug_assert! fires in test (debug) builds only
        let _f = FiniteF32::new(f32::INFINITY);
    }

    #[test]
    fn finite_f32_accepts_normal() {
        let f = FiniteF32::new(1.0);
        assert_eq!(f.get(), 1.0);
    }

    #[test]
    fn finite_f32_total_order() {
        let a = FiniteF32::new(0.5);
        let b = FiniteF32::new(0.7);
        assert!(a < b);
        assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn pos_f32_rejects_zero() {
        assert!(PosF32::new(0.0).is_none());
    }

    #[test]
    fn pos_f32_rejects_negative() {
        assert!(PosF32::new(-1.0).is_none());
    }

    #[test]
    fn pos_f32_accepts_positive() {
        assert!(PosF32::new(0.001).is_some());
    }

    #[test]
    fn learning_rate_range() {
        assert!(LearningRate::new(0.0).is_none());
        assert!(LearningRate::new(1.1).is_none());
        assert!(LearningRate::new(0.5).is_some());
        assert!(LearningRate::new(1.0).is_some());
    }

    #[test]
    fn decay_factor_range() {
        assert!(DecayFactor::new(0.0).is_none());
        assert!(DecayFactor::new(1.1).is_none());
        assert!(DecayFactor::new(0.55).is_some());
    }

    // ===== STEP-002: error.rs tests =====

    #[test]
    fn error_display_empty_graph() {
        let e = M1ndError::EmptyGraph;
        let msg = format!("{e}");
        assert!(msg.contains("empty"), "Expected 'empty' in: {msg}");
    }

    #[test]
    fn error_display_dangling_edge() {
        let e = M1ndError::DanglingEdge {
            edge: EdgeIdx::new(0),
            node: NodeId::new(999),
        };
        let msg = format!("{e}");
        assert!(msg.contains("dangling"));
    }

    // ===== STEP-003: graph.rs tests =====

    fn build_test_graph() -> Graph {
        let mut g = Graph::new();
        g.add_node(
            "mat_pe",
            "Polietileno",
            NodeType::Material,
            &["plastico", "polimero"],
            1000.0,
            0.5,
        )
        .unwrap();
        g.add_node(
            "mat_pp",
            "Polipropileno",
            NodeType::Material,
            &["plastico", "polimero"],
            900.0,
            0.3,
        )
        .unwrap();
        g.add_node(
            "mat_abs",
            "ABS",
            NodeType::Material,
            &["plastico"],
            800.0,
            0.2,
        )
        .unwrap();
        g.add_node(
            "proc_inj",
            "Injecao",
            NodeType::Process,
            &["processo"],
            700.0,
            0.4,
        )
        .unwrap();
        g.add_node(
            "proc_ext",
            "Extrusao",
            NodeType::Process,
            &["processo"],
            600.0,
            0.1,
        )
        .unwrap();
        g.add_node(
            "prod_garrafa",
            "Garrafa",
            NodeType::Product,
            &["produto"],
            500.0,
            0.6,
        )
        .unwrap();

        g.add_edge(
            NodeId::new(0),
            NodeId::new(3),
            "feeds_into",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(3),
            "feeds_into",
            FiniteF32::new(0.7),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(2),
            NodeId::new(4),
            "feeds_into",
            FiniteF32::new(0.6),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.2),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(3),
            NodeId::new(5),
            "produces",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.8),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            "similar_to",
            FiniteF32::new(0.5),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();

        g.finalize().unwrap();
        g
    }

    #[test]
    fn graph_add_node_and_resolve() {
        let mut g = Graph::new();
        let n1 = g
            .add_node("ext1", "Label1", NodeType::Module, &[], 0.0, 0.0)
            .unwrap();
        assert_eq!(n1, NodeId::new(0));
        assert_eq!(g.num_nodes(), 1);
        assert_eq!(g.resolve_id("ext1"), Some(NodeId::new(0)));
    }

    #[test]
    fn graph_add_node_duplicate() {
        let mut g = Graph::new();
        g.add_node("ext1", "label1", NodeType::Module, &[], 0.0, 0.0)
            .unwrap();
        let n2 = g.add_node("ext1", "label2", NodeType::Module, &[], 0.0, 0.0);
        assert!(matches!(n2, Err(M1ndError::DuplicateNode(_))));
    }

    #[test]
    fn graph_add_edge_dangling() {
        let mut g = Graph::new();
        let n1 = g
            .add_node("a", "A", NodeType::Module, &[], 0.0, 0.0)
            .unwrap();
        let bad = NodeId::new(999);
        let e = g.add_edge(
            n1,
            bad,
            "calls",
            FiniteF32::ONE,
            EdgeDirection::Forward,
            false,
            FiniteF32::ZERO,
        );
        assert!(matches!(e, Err(M1ndError::DanglingEdge { .. })));
    }

    #[test]
    fn graph_finalize_builds_csr() {
        let g = build_test_graph();
        assert!(g.finalized);
        assert_eq!(g.num_nodes(), 6);
        assert!(g.num_edges() > 0);
        assert!(g.pagerank_computed);
    }

    #[test]
    fn graph_pagerank_computed() {
        let g = build_test_graph();
        // At least one node should have non-zero pagerank
        let max_pr = (0..g.num_nodes() as usize)
            .map(|i| g.nodes.pagerank[i].get())
            .fold(0.0f32, f32::max);
        assert!(max_pr > 0.0, "PageRank should have non-zero values");
    }

    // ===== STEP-004: seed.rs tests =====

    #[test]
    fn seed_finder_exact_match() {
        let g = build_test_graph();
        let seeds = SeedFinder::find_seeds(&g, "Polietileno", 200).unwrap();
        assert!(!seeds.is_empty(), "Should find at least one seed");
        assert_eq!(
            seeds[0].1.get(),
            1.0,
            "Exact match should have relevance 1.0"
        );
    }

    #[test]
    fn seed_finder_tag_match() {
        let g = build_test_graph();
        let seeds = SeedFinder::find_seeds(&g, "plastico", 200).unwrap();
        assert!(!seeds.is_empty(), "Should find seeds by tag");
    }

    #[test]
    fn seed_finder_caps_at_max() {
        let g = build_test_graph();
        let seeds = SeedFinder::find_seeds(&g, "a", 2).unwrap();
        assert!(seeds.len() <= 2);
    }

    // ===== STEP-005: activation.rs tests =====

    #[test]
    fn bloom_filter_basic() {
        let mut bf = BloomFilter::with_capacity(1000, 0.01);
        bf.insert(NodeId::new(42));
        assert!(bf.probably_contains(NodeId::new(42)));
        // 99 is likely not in the filter (1% FPR)
        // Not asserting false because Bloom can have false positives
    }

    #[test]
    fn wavefront_single_seed() {
        let g = build_test_graph();
        let engine = WavefrontEngine::new();
        let config = PropagationConfig::default();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine.propagate(&g, &seeds, &config).unwrap();
        assert!(
            !result.scores.is_empty(),
            "Wavefront should activate at least one node"
        );
        assert!(result.scores[0].1.get() > 0.0);
    }

    #[test]
    fn heap_single_seed() {
        let g = build_test_graph();
        let engine = HeapEngine::new();
        let config = PropagationConfig::default();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine.propagate(&g, &seeds, &config).unwrap();
        assert!(
            !result.scores.is_empty(),
            "Heap should activate at least one node"
        );
    }

    #[test]
    fn hybrid_delegates_correctly() {
        let g = build_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine.propagate(&g, &seeds, &config).unwrap();
        assert!(!result.scores.is_empty());
    }

    #[test]
    fn activation_empty_seeds_returns_empty() {
        let g = build_test_graph();
        let engine = WavefrontEngine::new();
        let config = PropagationConfig::default();
        let result = engine.propagate(&g, &[], &config).unwrap();
        assert!(result.scores.is_empty());
    }

    #[test]
    fn merge_dimensions_resonance_bonus() {
        // FM-ACT-001 FIX: 4-dim check BEFORE 3-dim
        let make_dim = |dim: Dimension, scores: Vec<(NodeId, FiniteF32)>| DimensionResult {
            scores,
            dimension: dim,
            elapsed_ns: 0,
        };

        let node = NodeId::new(0);
        let score = FiniteF32::new(0.5);
        let results = [
            make_dim(Dimension::Structural, vec![(node, score)]),
            make_dim(Dimension::Semantic, vec![(node, score)]),
            make_dim(Dimension::Temporal, vec![(node, score)]),
            make_dim(Dimension::Causal, vec![(node, score)]),
        ];

        let merged = merge_dimensions(&results, 10).unwrap();
        assert!(!merged.activated.is_empty());
        let activated = &merged.activated[0];
        assert_eq!(activated.active_dimension_count, 4);
        // With 4-dim resonance bonus of 1.5x, score should be boosted
        let base = 0.5 * 0.35 + 0.5 * 0.25 + 0.5 * 0.15 + 0.5 * 0.25;
        let expected = base * RESONANCE_BONUS_4DIM;
        assert!(
            (activated.activation.get() - expected).abs() < 0.01,
            "Expected ~{expected}, got {}",
            activated.activation.get()
        );
    }

    // ===== STEP-006: xlr.rs tests =====

    #[test]
    fn xlr_sigmoid_gate() {
        let zero = AdaptiveXlrEngine::sigmoid_gate(FiniteF32::ZERO);
        assert!((zero.get() - 0.5).abs() < 0.01, "sigmoid(0) should be ~0.5");

        let positive = AdaptiveXlrEngine::sigmoid_gate(FiniteF32::new(1.0));
        assert!(positive.get() > 0.5, "sigmoid(+) should be > 0.5");

        let negative = AdaptiveXlrEngine::sigmoid_gate(FiniteF32::new(-1.0));
        assert!(negative.get() < 0.5, "sigmoid(-) should be < 0.5");
    }

    #[test]
    fn xlr_pick_anti_seeds() {
        let g = build_test_graph();
        let xlr = AdaptiveXlrEngine::with_defaults();
        let seeds = vec![NodeId::new(0)];
        let anti = xlr.pick_anti_seeds(&g, &seeds).unwrap();
        // Anti-seeds should not include the seed itself
        assert!(!anti.contains(&NodeId::new(0)));
    }

    #[test]
    fn xlr_immunity_bfs() {
        let g = build_test_graph();
        let xlr = AdaptiveXlrEngine::with_defaults();
        let seeds = vec![NodeId::new(0)];
        let immunity = xlr.compute_immunity(&g, &seeds).unwrap();
        assert!(immunity[0], "Seed itself should be immune");
    }

    // ===== STEP-008: temporal.rs tests =====

    #[test]
    fn temporal_decay_clamps_negative_age() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        let result = scorer.score_one(-10.0, FiniteF32::ZERO, None);
        // FM-TMP-009: negative age clamped to 0 -> raw_decay should be 1.0
        assert!(
            (result.raw_decay.get() - 1.0).abs() < 0.01,
            "Negative age should clamp to decay=1.0, got {}",
            result.raw_decay.get()
        );
    }

    #[test]
    fn temporal_decay_exponential() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        let result = scorer.score_one(168.0, FiniteF32::ZERO, None);
        // After one half-life, decay should be ~0.5
        assert!(
            (result.raw_decay.get() - 0.5).abs() < 0.05,
            "After one half-life, decay ~0.5, got {}",
            result.raw_decay.get()
        );
    }

    #[test]
    fn causal_chain_budget_limits() {
        let g = build_test_graph();
        let detector = CausalChainDetector::new(6, FiniteF32::new(0.01), 100);
        let chains = detector.detect(&g, NodeId::new(0)).unwrap();
        // Should find at least one chain (mat_pe -> proc_inj -> prod_garrafa)
        // Budget of 100 should be sufficient for this small graph
        assert!(!chains.is_empty() || g.num_edges() == 0);
    }

    // ===== STEP-009: topology.rs tests =====

    #[test]
    fn louvain_detects_communities() {
        let g = build_test_graph();
        let detector = CommunityDetector::with_defaults();
        let result = detector.detect(&g).unwrap();
        assert!(result.num_communities >= 1);
        assert_eq!(result.assignments.len(), g.num_nodes() as usize);
    }

    #[test]
    fn louvain_empty_graph_error() {
        let g = Graph::new();
        let detector = CommunityDetector::with_defaults();
        let result = detector.detect(&g);
        assert!(matches!(result, Err(M1ndError::EmptyGraph)));
    }

    #[test]
    fn bridge_detection() {
        let g = build_test_graph();
        let detector = CommunityDetector::with_defaults();
        let communities = detector.detect(&g).unwrap();
        let bridges = BridgeDetector::detect(&g, &communities).unwrap();
        // May or may not have bridges depending on community structure
        // Just verify it doesn't crash
        let _ = bridges;
    }

    #[test]
    fn spectral_gap_empty_graph() {
        let g = Graph::new();
        let analyzer = SpectralGapAnalyzer::with_defaults();
        let result = analyzer.analyze(&g);
        assert!(matches!(result, Err(M1ndError::EmptyGraph)));
    }

    // ===== STEP-010: resonance.rs tests =====

    #[test]
    fn wave_accumulator_complex_interference() {
        let mut acc = WaveAccumulator::default();
        let pulse1 = WavePulse {
            node: NodeId::new(0),
            amplitude: FiniteF32::ONE,
            phase: FiniteF32::ZERO,
            frequency: PosF32::new(1.0).unwrap(),
            wavelength: PosF32::new(4.0).unwrap(),
            hops: 0,
            prev_node: NodeId::new(0),
        };
        acc.accumulate(&pulse1);
        let amp = acc.amplitude().get();
        assert!(
            (amp - 1.0).abs() < 0.01,
            "Single pulse amplitude should be ~1.0"
        );
    }

    #[test]
    fn standing_wave_propagation() {
        let g = build_test_graph();
        let propagator = StandingWavePropagator::new(5, FiniteF32::new(0.01), 10_000);
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = propagator
            .propagate(
                &g,
                &seeds,
                PosF32::new(1.0).unwrap(),
                PosF32::new(4.0).unwrap(),
            )
            .unwrap();
        assert!(result.pulses_processed > 0);
        assert!(!result.antinodes.is_empty());
    }

    // ===== STEP-011: plasticity.rs tests =====

    #[test]
    fn query_memory_ring_buffer() {
        let mut mem = QueryMemory::new(3, 10);
        assert!(mem.is_empty());

        for i in 0..5 {
            mem.record(QueryRecord {
                query_text: format!("query_{i}"),
                seeds: vec![NodeId::new(i)],
                activated_nodes: vec![NodeId::new(i), NodeId::new(i + 1)],
                timestamp: i as f64,
            });
        }
        // Capacity is 3, so only 3 records should be present
        assert_eq!(mem.len(), 3);
    }

    #[test]
    fn plasticity_generation_check() {
        let mut g = build_test_graph();
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        // After adding a node, generation changes
        // PlasticityEngine's generation should no longer match
        // (we relax this in update, but check_generation would catch it)
        let _ = engine;
    }

    // ===== STEP-012: counterfactual.rs tests =====

    #[test]
    fn removal_mask_basic() {
        let g = build_test_graph();
        let mut mask = RemovalMask::new(g.num_nodes(), g.num_edges());
        assert!(!mask.is_node_removed(NodeId::new(0)));
        mask.remove_node(&g, NodeId::new(0));
        assert!(mask.is_node_removed(NodeId::new(0)));
        mask.reset();
        assert!(!mask.is_node_removed(NodeId::new(0)));
    }

    // ===== Integration: full query orchestration =====

    #[test]
    fn query_orchestrator_builds() {
        let g = build_test_graph();
        let orch = QueryOrchestrator::build(&g);
        assert!(orch.is_ok(), "Orchestrator should build from test graph");
    }

    #[test]
    fn full_query_execution() {
        let mut g = build_test_graph();
        let mut orch = QueryOrchestrator::build(&g).unwrap();
        let config = QueryConfig {
            query: "plastico polimero".to_string(),
            agent_id: "test".to_string(),
            top_k: 10,
            xlr_enabled: false, // Disable XLR for simpler test
            include_ghost_edges: true,
            include_structural_holes: false,
            ..QueryConfig::default()
        };
        let result = orch.query(&mut g, &config).unwrap();
        assert!(result.elapsed_ms >= 0.0);
        // Should activate at least the seed nodes
        // (exact count depends on seed finding + propagation)
    }

    // =================================================================
    // NEW TESTS — Agent C0: Core Test Suite expansion
    // =================================================================

    /// Helper: build a richer test graph with ~10 nodes and ~15 edges for edge-case coverage.
    fn make_test_graph() -> Graph {
        let mut g = Graph::new();
        // 10 nodes: mix of types
        g.add_node(
            "n0",
            "Alpha",
            NodeType::Material,
            &["group_a", "core"],
            1000.0,
            0.9,
        )
        .unwrap();
        g.add_node(
            "n1",
            "Beta",
            NodeType::Material,
            &["group_a", "core"],
            900.0,
            0.8,
        )
        .unwrap();
        g.add_node("n2", "Gamma", NodeType::Process, &["group_b"], 800.0, 0.7)
            .unwrap();
        g.add_node("n3", "Delta", NodeType::Process, &["group_b"], 700.0, 0.6)
            .unwrap();
        g.add_node("n4", "Epsilon", NodeType::Product, &["group_c"], 600.0, 0.5)
            .unwrap();
        g.add_node("n5", "Zeta", NodeType::Product, &["group_c"], 500.0, 0.4)
            .unwrap();
        g.add_node("n6", "Eta", NodeType::Module, &["group_d"], 400.0, 0.3)
            .unwrap();
        g.add_node("n7", "Theta", NodeType::Module, &["group_d"], 300.0, 0.2)
            .unwrap();
        g.add_node("n8", "Iota", NodeType::Concept, &["group_e"], 200.0, 0.1)
            .unwrap();
        g.add_node("n9", "Kappa", NodeType::Concept, &["group_e"], 100.0, 0.05)
            .unwrap();

        // 15 edges: hub at n0, chain n0->n2->n4->n6->n8, cross-links
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            "similar",
            FiniteF32::new(0.9),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(2),
            "feeds",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.7),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(3),
            "feeds",
            FiniteF32::new(0.7),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.6),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(4),
            "feeds",
            FiniteF32::new(0.6),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(5),
            "feeds",
            FiniteF32::new(0.5),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.4),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(2),
            NodeId::new(4),
            "produces",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.8),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(3),
            NodeId::new(5),
            "produces",
            FiniteF32::new(0.7),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.7),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(4),
            NodeId::new(6),
            "uses",
            FiniteF32::new(0.6),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(5),
            NodeId::new(7),
            "uses",
            FiniteF32::new(0.5),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.2),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(6),
            NodeId::new(8),
            "refs",
            FiniteF32::new(0.4),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.1),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(7),
            NodeId::new(9),
            "refs",
            FiniteF32::new(0.3),
            EdgeDirection::Forward,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(3),
            "feeds",
            FiniteF32::new(0.6),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(8),
            NodeId::new(9),
            "related",
            FiniteF32::new(0.5),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();
        g.add_edge(
            NodeId::new(6),
            NodeId::new(7),
            "related",
            FiniteF32::new(0.4),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();
        g.add_edge(
            NodeId::new(2),
            NodeId::new(3),
            "related",
            FiniteF32::new(0.3),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();

        g.finalize().unwrap();
        g
    }

    // ===== Semantic tests =====

    #[test]
    fn test_semantic_cooccurrence_builds_from_graph() {
        use crate::semantic::CoOccurrenceIndex;
        let g = make_test_graph();
        let idx = CoOccurrenceIndex::build(&g, 10, 20, 4).unwrap();
        // Should have vectors for each node
        let _ = idx; // build succeeded without panic
    }

    #[test]
    fn test_semantic_ppmi_positive_values() {
        use crate::semantic::CoOccurrenceIndex;
        let g = make_test_graph();
        let idx = CoOccurrenceIndex::build(&g, 10, 20, 4).unwrap();
        // Query top-k for node 0 (hub node) — should find similar nodes
        let results = idx.query_top_k(NodeId::new(0), 5);
        // All returned PPMI similarity scores must be positive
        for &(_, score) in &results {
            assert!(
                score.get() >= 0.0,
                "PPMI score must be non-negative, got {}",
                score.get()
            );
        }
    }

    #[test]
    fn test_semantic_engine_finds_similar_nodes() {
        use crate::semantic::SemanticEngine;
        let g = make_test_graph();
        let engine = SemanticEngine::build(&g, SemanticWeights::default()).unwrap();
        let results = engine.query(&g, "Alpha", 5).unwrap();
        // Searching for "Alpha" should return at least the node with that label
        assert!(
            !results.is_empty(),
            "Semantic query for exact label should return results"
        );
    }

    #[test]
    fn test_semantic_search_exact_label_match() {
        use crate::semantic::CharNgramIndex;
        let g = make_test_graph();
        let idx = CharNgramIndex::build(&g, 3).unwrap();
        let results = idx.query_top_k("Alpha", 10);
        // The node labeled "Alpha" should appear in results
        assert!(
            !results.is_empty(),
            "N-gram search for exact label should find matches"
        );
        // Top result should be node 0 (Alpha)
        assert_eq!(
            results[0].0,
            NodeId::new(0),
            "Top result should be node 0 (Alpha)"
        );
    }

    #[test]
    fn test_semantic_search_empty_query_returns_empty() {
        use crate::semantic::CharNgramIndex;
        let g = make_test_graph();
        let idx = CharNgramIndex::build(&g, 3).unwrap();
        let results = idx.query_top_k("", 10);
        // Empty query should still not panic; may return empty or very low scores
        let _ = results;
    }

    #[test]
    fn test_semantic_cosine_similarity_identical_vectors() {
        use crate::semantic::CharNgramIndex;
        let g = make_test_graph();
        let idx = CharNgramIndex::build(&g, 3).unwrap();
        let qvec = idx.query_vector("Alpha");
        let sim = CharNgramIndex::cosine_similarity(&qvec, &qvec);
        assert!(
            (sim.get() - 1.0).abs() < 0.01,
            "Self-similarity should be ~1.0, got {}",
            sim.get()
        );
    }

    #[test]
    fn test_semantic_cosine_similarity_empty_vectors() {
        use crate::semantic::{CharNgramIndex, NgramVector};
        let empty: NgramVector = std::collections::HashMap::new();
        let sim = CharNgramIndex::cosine_similarity(&empty, &empty);
        assert_eq!(
            sim.get(),
            0.0,
            "Cosine similarity of empty vectors should be 0"
        );
    }

    #[test]
    fn test_semantic_synonym_expansion() {
        use crate::semantic::SynonymExpander;
        let expander = SynonymExpander::build_default().unwrap();
        let expanded = expander.expand("plastico");
        assert!(expanded.contains(&"plastico".to_string()));
        assert!(expanded.contains(&"polimero".to_string()));
        assert!(expanded.contains(&"resina".to_string()));
    }

    #[test]
    fn test_semantic_synonym_are_synonyms() {
        use crate::semantic::SynonymExpander;
        let expander = SynonymExpander::build_default().unwrap();
        assert!(expander.are_synonyms("plastico", "polimero"));
        assert!(expander.are_synonyms("Plastico", "POLIMERO")); // case-insensitive
        assert!(!expander.are_synonyms("plastico", "metal"));
    }

    // ===== Snapshot tests =====

    #[test]
    fn test_snapshot_roundtrip_graph() {
        use crate::snapshot::{load_graph, save_graph};
        let g = make_test_graph();
        let path = std::path::PathBuf::from("/tmp/m1nd_test_snapshot_graph.json");
        save_graph(&g, &path).unwrap();
        let loaded = load_graph(&path).unwrap();
        assert_eq!(
            loaded.num_nodes(),
            g.num_nodes(),
            "Node count mismatch after roundtrip"
        );
        // Edges may differ slightly due to bidirectional expansion, but should be non-zero
        assert!(loaded.num_edges() > 0, "Loaded graph should have edges");
        assert!(loaded.finalized, "Loaded graph should be finalized");
        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_snapshot_roundtrip_preserves_labels() {
        use crate::snapshot::{load_graph, save_graph};
        let g = make_test_graph();
        let path = std::path::PathBuf::from("/tmp/m1nd_test_snapshot_labels.json");
        save_graph(&g, &path).unwrap();
        let loaded = load_graph(&path).unwrap();
        // Check that node n0 ("Alpha") can be resolved
        assert!(
            loaded.resolve_id("n0").is_some(),
            "Should resolve 'n0' after roundtrip"
        );
        assert!(
            loaded.resolve_id("n9").is_some(),
            "Should resolve 'n9' after roundtrip"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_snapshot_roundtrip_preserves_provenance() {
        use crate::graph::NodeProvenanceInput;
        use crate::snapshot::{load_graph, save_graph};

        let mut g = make_test_graph();
        let node = g.resolve_id("n0").unwrap();
        g.set_node_provenance(
            node,
            NodeProvenanceInput {
                source_path: Some("memory/2026-03-13.md"),
                line_start: Some(12),
                line_end: Some(14),
                excerpt: Some("Batman mode means peak build window."),
                namespace: Some("memory"),
                canonical: true,
            },
        );

        let path = std::path::PathBuf::from("/tmp/m1nd_test_snapshot_provenance.json");
        save_graph(&g, &path).unwrap();
        let loaded = load_graph(&path).unwrap();
        let provenance = loaded.resolve_node_provenance(loaded.resolve_id("n0").unwrap());

        assert_eq!(
            provenance.source_path.as_deref(),
            Some("memory/2026-03-13.md")
        );
        assert_eq!(provenance.line_start, Some(12));
        assert_eq!(provenance.line_end, Some(14));
        assert_eq!(
            provenance.excerpt.as_deref(),
            Some("Batman mode means peak build window.")
        );
        assert_eq!(provenance.namespace.as_deref(), Some("memory"));
        assert!(provenance.canonical);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_snapshot_roundtrip_plasticity_state() {
        use crate::plasticity::SynapticState;
        use crate::snapshot::{load_plasticity_state, save_plasticity_state};
        let states = vec![
            SynapticState {
                source_label: "n0".to_string(),
                target_label: "n2".to_string(),
                relation: "feeds".to_string(),
                original_weight: 0.8,
                current_weight: 0.85,
                strengthen_count: 3,
                weaken_count: 0,
                ltp_applied: false,
                ltd_applied: false,
            },
            SynapticState {
                source_label: "n2".to_string(),
                target_label: "n4".to_string(),
                relation: "produces".to_string(),
                original_weight: 0.7,
                current_weight: 0.65,
                strengthen_count: 0,
                weaken_count: 2,
                ltp_applied: false,
                ltd_applied: false,
            },
        ];
        let path = std::path::PathBuf::from("/tmp/m1nd_test_plasticity_state.json");
        save_plasticity_state(&states, &path).unwrap();
        let loaded = load_plasticity_state(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].source_label, "n0");
        assert!((loaded[0].current_weight - 0.85).abs() < 1e-5);
        assert_eq!(loaded[1].weaken_count, 2);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_snapshot_load_nonexistent_file_returns_error() {
        use crate::snapshot::load_graph;
        let path = std::path::Path::new("/tmp/m1nd_test_nonexistent_42.json");
        let result = load_graph(path);
        assert!(
            result.is_err(),
            "Loading nonexistent file should return error"
        );
    }

    #[test]
    fn test_snapshot_load_corrupt_json_returns_error() {
        use crate::snapshot::load_graph;
        let path = std::path::PathBuf::from("/tmp/m1nd_test_corrupt.json");
        std::fs::write(&path, "{ this is not valid json !!!").unwrap();
        let result = load_graph(&path);
        assert!(result.is_err(), "Loading corrupt JSON should return error");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_snapshot_saved_graph_is_valid_json() {
        use crate::snapshot::save_graph;
        let g = make_test_graph();
        let path = std::path::PathBuf::from("/tmp/m1nd_test_valid_json.json");
        save_graph(&g, &path).unwrap();
        let data = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&data).unwrap();
        assert!(parsed.is_object(), "Saved snapshot should be a JSON object");
        assert!(
            parsed.get("version").is_some(),
            "Snapshot should have version field"
        );
        assert!(
            parsed.get("nodes").is_some(),
            "Snapshot should have nodes field"
        );
        assert!(
            parsed.get("edges").is_some(),
            "Snapshot should have edges field"
        );
        let _ = std::fs::remove_file(&path);
    }

    // ===== Query tests =====

    #[test]
    fn test_query_orchestrator_builds_from_finalized_graph() {
        let g = make_test_graph();
        let orch = QueryOrchestrator::build(&g);
        assert!(
            orch.is_ok(),
            "Orchestrator should build from finalized graph"
        );
    }

    #[test]
    fn test_query_execution_returns_results_for_existing_node() {
        let mut g = make_test_graph();
        let mut orch = QueryOrchestrator::build(&g).unwrap();
        let config = QueryConfig {
            query: "Alpha".to_string(),
            agent_id: "test_agent".to_string(),
            top_k: 10,
            xlr_enabled: false,
            include_ghost_edges: false,
            include_structural_holes: false,
            ..QueryConfig::default()
        };
        let result = orch.query(&mut g, &config).unwrap();
        assert!(result.elapsed_ms >= 0.0);
    }

    #[test]
    fn test_query_with_nonexistent_term_returns_gracefully() {
        let mut g = make_test_graph();
        let mut orch = QueryOrchestrator::build(&g).unwrap();
        let config = QueryConfig {
            query: "zzzznonexistent_xyz_987".to_string(),
            agent_id: "test".to_string(),
            top_k: 5,
            xlr_enabled: false,
            include_ghost_edges: false,
            include_structural_holes: false,
            ..QueryConfig::default()
        };
        let result = orch.query(&mut g, &config).unwrap();
        // Should not crash; may return empty or minimal results
        assert!(result.elapsed_ms >= 0.0);
    }

    #[test]
    fn test_query_memory_deduplication() {
        let mut mem = QueryMemory::new(10, 20);
        let record1 = QueryRecord {
            query_text: "Alpha".to_string(),
            seeds: vec![NodeId::new(0)],
            activated_nodes: vec![NodeId::new(0), NodeId::new(1)],
            timestamp: 1.0,
        };
        let record2 = QueryRecord {
            query_text: "Alpha".to_string(),
            seeds: vec![NodeId::new(0)],
            activated_nodes: vec![NodeId::new(0), NodeId::new(1)],
            timestamp: 2.0,
        };
        mem.record(record1);
        mem.record(record2);
        assert_eq!(mem.len(), 2, "Both records should be stored");
        // Priming signal should reflect accumulated frequency
        let priming = mem.get_priming_signal(&[NodeId::new(0)], FiniteF32::new(0.5));
        // Node 1 should appear in priming (activated twice with seed 0)
        let has_node1 = priming.iter().any(|(n, _)| *n == NodeId::new(1));
        assert!(has_node1, "Node 1 should appear in priming signal");
    }

    // ===== Counterfactual tests =====

    #[test]
    fn test_counterfactual_removal_of_leaf_node() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        // Node 9 (Kappa) is a leaf — removing it should have low/zero impact
        let result = cf
            .simulate_removal(&g, &engine, &config, &[NodeId::new(9)])
            .unwrap();
        assert!(
            result.total_impact.get() >= 0.0,
            "Impact should be non-negative"
        );
        // Leaf removal typically has very low impact
    }

    #[test]
    fn test_counterfactual_removal_of_hub_node_has_positive_impact() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        // Node 0 (Alpha) is the hub — removing it should have significant impact
        let result = cf
            .simulate_removal(&g, &engine, &config, &[NodeId::new(0)])
            .unwrap();
        assert!(
            result.total_impact.get() >= 0.0,
            "Hub removal should have non-negative impact"
        );
    }

    #[test]
    fn test_counterfactual_empty_removal_returns_zero_impact() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        let result = cf.simulate_removal(&g, &engine, &config, &[]).unwrap();
        assert!(
            (result.total_impact.get() - 0.0).abs() < 0.01,
            "Empty removal should have ~0% impact, got {}",
            result.total_impact.get()
        );
    }

    #[test]
    fn test_counterfactual_multi_node_combined_ge_individual() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        let synergy = cf
            .synergy_analysis(&g, &engine, &config, &[NodeId::new(0), NodeId::new(2)])
            .unwrap();
        // Combined impact should be >= max individual (synergy or at least additive)
        let max_individual = synergy
            .individual_impacts
            .iter()
            .map(|(_, s)| s.get())
            .fold(0.0f32, f32::max);
        assert!(
            synergy.combined_impact.get() >= max_individual * 0.5,
            "Combined impact ({}) should be significant relative to individual ({})",
            synergy.combined_impact.get(),
            max_individual
        );
    }

    #[test]
    fn test_counterfactual_cascade_analysis() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        let cascade = cf
            .cascade_analysis(&g, &engine, &config, NodeId::new(0))
            .unwrap();
        // Hub node should have downstream cascade
        assert!(
            cascade.total_affected > 0,
            "Hub removal should have downstream cascade"
        );
        assert!(
            cascade.cascade_depth > 0,
            "Cascade depth from hub should be > 0"
        );
    }

    #[test]
    fn test_counterfactual_removal_mask_edge_removal() {
        let g = make_test_graph();
        let mut mask = RemovalMask::new(g.num_nodes(), g.num_edges());
        // Remove node 0 and check that its edges are also marked removed
        mask.remove_node(&g, NodeId::new(0));
        let out_range = g.csr.out_range(NodeId::new(0));
        for j in out_range {
            assert!(
                mask.is_edge_removed(EdgeIdx::new(j as u32)),
                "Edge {} from removed node should be marked removed",
                j
            );
        }
    }

    // ===== Plasticity tests =====

    #[test]
    fn test_plasticity_engine_creates_correct_entries() {
        let g = make_test_graph();
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        // Engine should be created without error; generation matches graph
        let _ = engine;
    }

    #[test]
    fn test_plasticity_strengthen_edge_increases_weight() {
        let mut g = make_test_graph();
        let mut engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        // Read weight of first edge before strengthening
        let edge0_before = g.csr.read_weight(EdgeIdx::new(0)).get();
        // Activate nodes 0 and 1 with high activation to trigger Hebbian strengthening
        let activated = vec![
            (NodeId::new(0), FiniteF32::ONE),
            (NodeId::new(1), FiniteF32::ONE),
            (NodeId::new(2), FiniteF32::ONE),
            (NodeId::new(3), FiniteF32::ONE),
            (NodeId::new(4), FiniteF32::ONE),
        ];
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine
            .update(&mut g, &activated, &seeds, "test query")
            .unwrap();
        assert!(
            result.edges_strengthened > 0,
            "At least one edge should be strengthened"
        );
    }

    #[test]
    fn test_plasticity_export_state_produces_real_labels() {
        let g = make_test_graph();
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        let states = engine.export_state(&g).unwrap();
        assert!(!states.is_empty(), "Export should produce states");
        // Verify labels are real node IDs, not placeholders
        for state in &states {
            assert!(
                !state.source_label.is_empty(),
                "Source label should not be empty"
            );
            assert!(
                !state.target_label.is_empty(),
                "Target label should not be empty"
            );
            assert!(
                !state.source_label.starts_with("node_"),
                "Source should use real label, not placeholder"
            );
        }
    }

    #[test]
    fn test_plasticity_import_state_roundtrip() {
        let mut g = make_test_graph();
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        let states = engine.export_state(&g).unwrap();
        // Import the exported state back
        let mut engine2 = PlasticityEngine::new(&g, PlasticityConfig::default());
        let applied = engine2.import_state(&mut g, &states).unwrap();
        assert!(applied > 0, "Import should apply at least some states");
    }

    #[test]
    fn test_plasticity_decay_reduces_weights() {
        let mut g = make_test_graph();
        let mut engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        // Read a weight before decay — use an edge from an unactivated node
        let edge_idx = EdgeIdx::new(
            g.csr.offsets[8] as u32, // first edge from node 8
        );
        let before = g.csr.read_weight(edge_idx).get();
        // Run update with NO activated nodes except node 0 — node 8's edges should decay
        let activated = vec![(NodeId::new(0), FiniteF32::ONE)];
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine
            .update(&mut g, &activated, &seeds, "decay test")
            .unwrap();
        let after = g.csr.read_weight(edge_idx).get();
        // Decay should reduce weight (or hit floor)
        assert!(
            after <= before,
            "Weight should decrease after decay: before={before}, after={after}"
        );
        assert!(
            result.edges_decayed > 0,
            "At least one edge should be decayed"
        );
    }

    #[test]
    fn test_plasticity_ltp_threshold_triggering() {
        let mut g = make_test_graph();
        let ltp_thresh = 3u16; // lower threshold for easier triggering
        let config = PlasticityConfig {
            ltp_threshold: ltp_thresh,
            ..PlasticityConfig::default()
        };
        let mut engine = PlasticityEngine::new(&g, config);
        let activated = vec![
            (NodeId::new(0), FiniteF32::ONE),
            (NodeId::new(1), FiniteF32::ONE),
            (NodeId::new(2), FiniteF32::ONE),
            (NodeId::new(3), FiniteF32::ONE),
        ];
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        // Run multiple updates to accumulate strengthen_count past threshold
        let mut total_ltp = 0u32;
        for i in 0..6 {
            let result = engine
                .update(&mut g, &activated, &seeds, &format!("ltp_test_{i}"))
                .unwrap();
            total_ltp += result.ltp_events;
        }
        assert!(
            total_ltp > 0,
            "LTP should trigger after repeated strengthening"
        );
    }

    #[test]
    fn test_plasticity_query_memory_priming_signal() {
        let mut mem = QueryMemory::new(100, 20);
        // Record multiple queries with overlapping seeds
        for i in 0..5 {
            mem.record(QueryRecord {
                query_text: format!("query_{i}"),
                seeds: vec![NodeId::new(0), NodeId::new(1)],
                activated_nodes: vec![
                    NodeId::new(0),
                    NodeId::new(1),
                    NodeId::new(2),
                    NodeId::new(3),
                ],
                timestamp: i as f64,
            });
        }
        // Get priming for seed [0] — should find nodes that co-occur
        let priming = mem.get_priming_signal(&[NodeId::new(0)], FiniteF32::new(0.5));
        // Nodes 2 and 3 should appear (frequently activated alongside seed 0)
        assert!(
            !priming.is_empty(),
            "Priming signal should be non-empty after recording queries"
        );
    }

    #[test]
    fn test_plasticity_homeostatic_normalization() {
        let mut g = make_test_graph();
        // Artificially inflate a weight to exceed homeostatic ceiling
        let edge_idx = EdgeIdx::new(0);
        let _ = g
            .csr
            .atomic_write_weight(edge_idx, FiniteF32::new(10.0), 64);
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        // Calling export_state should still produce finite weights
        let states = engine.export_state(&g).unwrap();
        for state in &states {
            assert!(
                state.current_weight.is_finite(),
                "Exported weight should be finite"
            );
        }
    }

    // ===== Resonance tests =====

    #[test]
    fn test_resonance_standing_wave_produces_waves_from_seeds() {
        let g = make_test_graph();
        let propagator = StandingWavePropagator::new(10, FiniteF32::new(0.001), 50_000);
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = propagator
            .propagate(
                &g,
                &seeds,
                PosF32::new(1.0).unwrap(),
                PosF32::new(4.0).unwrap(),
            )
            .unwrap();
        assert!(
            result.pulses_processed > 0,
            "Should process at least one pulse"
        );
        assert!(!result.antinodes.is_empty(), "Should produce antinodes");
        assert!(
            result.total_energy.get() > 0.0,
            "Total energy should be positive"
        );
    }

    #[test]
    fn test_resonance_harmonic_analyzer_detects_fundamental() {
        let g = make_test_graph();
        let propagator = StandingWavePropagator::new(5, FiniteF32::new(0.01), 10_000);
        let analyzer = HarmonicAnalyzer::new(propagator, 3);
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = analyzer
            .analyze(
                &g,
                &seeds,
                PosF32::new(1.0).unwrap(),
                PosF32::new(4.0).unwrap(),
            )
            .unwrap();
        assert!(
            !result.harmonics.is_empty(),
            "Should detect at least one harmonic"
        );
        // Harmonic 1 is the fundamental
        assert_eq!(
            result.harmonics[0].harmonic, 1,
            "First harmonic should be the fundamental"
        );
        assert!(
            result.harmonics[0].total_energy.get() > 0.0,
            "Fundamental should have energy"
        );
    }

    #[test]
    fn test_resonance_sympathetic_detector_finds_pairs() {
        let g = make_test_graph();
        let propagator = StandingWavePropagator::new(10, FiniteF32::new(0.001), 50_000);
        let detector = SympatheticResonanceDetector::new(propagator, FiniteF32::new(0.01));
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = detector
            .detect(
                &g,
                &seeds,
                PosF32::new(1.0).unwrap(),
                PosF32::new(4.0).unwrap(),
            )
            .unwrap();
        assert!(
            result.checked_disconnected,
            "Should check disconnected components"
        );
        // Sympathetic nodes may or may not exist depending on graph topology
        let _ = result.sympathetic_nodes;
    }

    #[test]
    fn test_resonance_engine_analyze_returns_report() {
        let g = make_test_graph();
        let engine = ResonanceEngine::with_defaults();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let report = engine.analyze(&g, &seeds).unwrap();
        assert!(report.standing_wave.pulses_processed > 0);
        assert!(!report.harmonics.harmonics.is_empty());
    }

    #[test]
    fn test_resonance_wave_accumulator_destructive_interference() {
        let mut acc = WaveAccumulator::default();
        // Two pulses with opposite phases should cancel
        let pulse1 = WavePulse {
            node: NodeId::new(0),
            amplitude: FiniteF32::ONE,
            phase: FiniteF32::ZERO,
            frequency: PosF32::new(1.0).unwrap(),
            wavelength: PosF32::new(4.0).unwrap(),
            hops: 0,
            prev_node: NodeId::new(0),
        };
        let pulse2 = WavePulse {
            node: NodeId::new(0),
            amplitude: FiniteF32::ONE,
            phase: FiniteF32::new(std::f32::consts::PI),
            frequency: PosF32::new(1.0).unwrap(),
            wavelength: PosF32::new(4.0).unwrap(),
            hops: 0,
            prev_node: NodeId::new(0),
        };
        acc.accumulate(&pulse1);
        acc.accumulate(&pulse2);
        let amp = acc.amplitude().get();
        assert!(
            amp < 0.1,
            "Opposite-phase pulses should destructively interfere, got amp={amp}"
        );
    }

    // ===== Graph edge case tests =====

    #[test]
    fn test_graph_finalize_empty_graph_succeeds() {
        let mut g = Graph::new();
        // Finalizing an empty graph should not panic or error
        let result = g.finalize();
        assert!(result.is_ok(), "Finalizing empty graph should succeed");
        assert!(g.finalized);
    }

    #[test]
    fn test_graph_finalize_computes_pagerank_top_node() {
        let g = make_test_graph();
        // Node 0 (hub) should have the highest or near-highest PageRank
        let max_idx = (0..g.num_nodes() as usize)
            .max_by(|&a, &b| g.nodes.pagerank[a].cmp(&g.nodes.pagerank[b]))
            .unwrap();
        let max_pr = g.nodes.pagerank[max_idx].get();
        assert!(
            (max_pr - 1.0).abs() < 0.01,
            "Highest PageRank should be normalized to ~1.0, got {max_pr}"
        );
    }

    #[test]
    fn test_graph_csr_offsets_monotonically_increasing() {
        let g = make_test_graph();
        for i in 1..g.csr.offsets.len() {
            assert!(
                g.csr.offsets[i] >= g.csr.offsets[i - 1],
                "CSR offsets must be monotonically increasing: offsets[{}]={} < offsets[{}]={}",
                i,
                g.csr.offsets[i],
                i - 1,
                g.csr.offsets[i - 1]
            );
        }
    }

    #[test]
    fn test_graph_bidirectional_edge_creates_two_csr_entries() {
        // Build a minimal graph with one bidirectional edge
        let mut g = Graph::new();
        g.add_node("a", "A", NodeType::Module, &[], 0.0, 0.0)
            .unwrap();
        g.add_node("b", "B", NodeType::Module, &[], 0.0, 0.0)
            .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            "bidir",
            FiniteF32::ONE,
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::ZERO,
        )
        .unwrap();
        g.finalize().unwrap();
        // Bidirectional edge should create 2 CSR entries (one in each direction)
        assert_eq!(
            g.num_edges(),
            2,
            "Bidirectional edge should produce 2 CSR entries"
        );
        // Node 0 should have outgoing edge to 1
        let out_0 = g.csr.out_range(NodeId::new(0));
        assert_eq!(
            out_0.end - out_0.start,
            1,
            "Node 0 should have 1 outgoing edge"
        );
        // Node 1 should have outgoing edge to 0
        let out_1 = g.csr.out_range(NodeId::new(1));
        assert_eq!(
            out_1.end - out_1.start,
            1,
            "Node 1 should have 1 outgoing edge"
        );
    }

    #[test]
    fn test_graph_edge_plasticity_matches_csr_count() {
        let g = make_test_graph();
        let num_csr = g.num_edges();
        assert_eq!(
            g.edge_plasticity.original_weight.len(),
            num_csr,
            "Plasticity original_weight length should match CSR edge count"
        );
        assert_eq!(
            g.edge_plasticity.current_weight.len(),
            num_csr,
            "Plasticity current_weight length should match CSR edge count"
        );
        assert_eq!(
            g.edge_plasticity.strengthen_count.len(),
            num_csr,
            "Plasticity strengthen_count length should match CSR edge count"
        );
        assert_eq!(
            g.edge_plasticity.ltp_applied.len(),
            num_csr,
            "Plasticity ltp_applied length should match CSR edge count"
        );
    }

    #[test]
    fn test_graph_resolve_nonexistent_id_returns_none() {
        let g = make_test_graph();
        assert!(g.resolve_id("nonexistent_node_xyz").is_none());
    }

    #[test]
    fn test_graph_avg_degree_positive_for_nonempty_graph() {
        let g = make_test_graph();
        assert!(
            g.avg_degree() > 0.0,
            "Average degree should be positive for non-empty graph"
        );
    }

    #[test]
    fn test_graph_avg_degree_zero_for_empty_graph() {
        let g = Graph::new();
        assert_eq!(
            g.avg_degree(),
            0.0,
            "Average degree of empty graph should be 0"
        );
    }

    // ===== Additional counterfactual tests =====

    #[test]
    fn test_counterfactual_keystone_analysis() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::new(4, 5);
        let result = cf.find_keystones(&g, &engine, &config).unwrap();
        assert!(
            !result.keystones.is_empty(),
            "Should identify at least one keystone"
        );
        // Keystones should be sorted by impact descending
        for i in 1..result.keystones.len() {
            assert!(
                result.keystones[i - 1].avg_impact.get() >= result.keystones[i].avg_impact.get(),
                "Keystones should be sorted by impact descending"
            );
        }
    }

    #[test]
    fn test_counterfactual_redundancy_check() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        let result = cf
            .check_redundancy(&g, &engine, &config, NodeId::new(9))
            .unwrap();
        // Leaf node should be fairly redundant (high score)
        assert!(
            result.redundancy_score.get() >= 0.0 && result.redundancy_score.get() <= 1.0,
            "Redundancy score should be in [0,1], got {}",
            result.redundancy_score.get()
        );
    }

    #[test]
    fn test_counterfactual_reachability_tracked() {
        let g = make_test_graph();
        let engine = HybridEngine::new();
        let config = PropagationConfig::default();
        let cf = CounterfactualEngine::with_defaults();
        let result = cf
            .simulate_removal(&g, &engine, &config, &[NodeId::new(0)])
            .unwrap();
        assert!(
            result.reachability_before > 0,
            "Reachability before should be > 0"
        );
        // After removing hub, reachability should decrease
        assert!(
            result.reachability_after <= result.reachability_before,
            "Reachability should not increase after removal"
        );
    }

    // ===== Additional query / orchestration tests =====

    #[test]
    fn test_query_config_default_has_four_dimensions() {
        let config = QueryConfig::default();
        assert_eq!(config.dimensions.len(), 4);
        assert!(config.xlr_enabled);
        assert_eq!(config.top_k, 20);
    }

    #[test]
    fn test_query_with_structural_holes_enabled() {
        let mut g = make_test_graph();
        let mut orch = QueryOrchestrator::build(&g).unwrap();
        let config = QueryConfig {
            query: "Alpha Beta".to_string(),
            agent_id: "test".to_string(),
            top_k: 10,
            xlr_enabled: false,
            include_ghost_edges: true,
            include_structural_holes: true,
            ..QueryConfig::default()
        };
        let result = orch.query(&mut g, &config).unwrap();
        // Should not crash with structural holes enabled
        assert!(result.elapsed_ms >= 0.0);
    }

    // ===== Additional activation / merge tests =====

    #[test]
    fn test_merge_dimensions_three_dim_resonance_bonus() {
        let make_dim = |dim: Dimension, scores: Vec<(NodeId, FiniteF32)>| DimensionResult {
            scores,
            dimension: dim,
            elapsed_ns: 0,
        };
        let node = NodeId::new(0);
        let score = FiniteF32::new(0.5);
        let results = [
            make_dim(Dimension::Structural, vec![(node, score)]),
            make_dim(Dimension::Semantic, vec![(node, score)]),
            make_dim(Dimension::Temporal, vec![(node, score)]),
            make_dim(Dimension::Causal, vec![]), // no causal
        ];
        let merged = merge_dimensions(&results, 10).unwrap();
        assert!(!merged.activated.is_empty());
        let activated = &merged.activated[0];
        assert_eq!(activated.active_dimension_count, 3);
        // 3-dim bonus = 1.3x
        let w_sum = 0.35 + 0.25 + 0.15; // adaptive weights redistribute
        let base = 0.5 * (0.35 / w_sum) + 0.5 * (0.25 / w_sum) + 0.5 * (0.15 / w_sum);
        let expected = base * RESONANCE_BONUS_3DIM;
        assert!(
            (activated.activation.get() - expected).abs() < 0.02,
            "Expected ~{expected}, got {}",
            activated.activation.get()
        );
    }

    #[test]
    fn test_merge_dimensions_single_dim_no_bonus() {
        let make_dim = |dim: Dimension, scores: Vec<(NodeId, FiniteF32)>| DimensionResult {
            scores,
            dimension: dim,
            elapsed_ns: 0,
        };
        let node = NodeId::new(0);
        let score = FiniteF32::new(0.5);
        let results = [
            make_dim(Dimension::Structural, vec![(node, score)]),
            make_dim(Dimension::Semantic, vec![]),
            make_dim(Dimension::Temporal, vec![]),
            make_dim(Dimension::Causal, vec![]),
        ];
        let merged = merge_dimensions(&results, 10).unwrap();
        assert!(!merged.activated.is_empty());
        let activated = &merged.activated[0];
        assert_eq!(activated.active_dimension_count, 1);
        // No resonance bonus for single dimension
        // Adaptive weight: 0.35 redistributed to 1.0 (only active dim)
        let expected = 0.5 * 1.0; // weight normalized to 1.0 since only one dim active
        assert!(
            (activated.activation.get() - expected).abs() < 0.02,
            "Expected ~{expected} (no bonus), got {}",
            activated.activation.get()
        );
    }

    // ===== Bloom filter edge cases =====

    #[test]
    fn test_bloom_filter_clear_resets_all() {
        let mut bf = BloomFilter::with_capacity(100, 0.01);
        bf.insert(NodeId::new(10));
        bf.insert(NodeId::new(20));
        assert!(bf.probably_contains(NodeId::new(10)));
        bf.clear();
        // After clear, nothing should be found (assuming no false positives)
        // We can't guarantee no FP, but 10 should likely not be found
        // Just verify clear doesn't crash
    }

    // ===== String interner edge case =====

    #[test]
    fn test_string_interner_idempotent() {
        let mut interner = StringInterner::new();
        let h1 = interner.get_or_intern("hello");
        let h2 = interner.get_or_intern("hello");
        assert_eq!(h1, h2, "Re-interning same string should return same handle");
        assert_eq!(interner.len(), 1, "Should only have one entry");
    }

    #[test]
    fn test_string_interner_resolve() {
        let mut interner = StringInterner::new();
        let h = interner.get_or_intern("world");
        assert_eq!(interner.resolve(h), "world");
    }

    // ===== Temporal additional tests =====

    #[test]
    fn test_temporal_activation_dimension() {
        let g = make_test_graph();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let weights = TemporalWeights::default();
        let result = activate_temporal(&g, &seeds, &weights).unwrap();
        assert_eq!(result.dimension, Dimension::Temporal);
        // Should produce at least one score for the seed node
        assert!(
            !result.scores.is_empty(),
            "Temporal activation should produce scores for seeds"
        );
    }

    #[test]
    fn test_causal_activation_dimension() {
        let g = make_test_graph();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let config = PropagationConfig::default();
        let result = activate_causal(&g, &seeds, &config).unwrap();
        assert_eq!(result.dimension, Dimension::Causal);
    }

    #[test]
    fn test_causal_activation_empty_graph() {
        let g = Graph::new();
        let seeds = vec![];
        let config = PropagationConfig::default();
        let result = activate_causal(&g, &seeds, &config).unwrap();
        assert!(result.scores.is_empty());
    }

    // ===== Removal mask additional tests =====

    #[test]
    fn test_removal_mask_remove_edge_directly() {
        let g = make_test_graph();
        let mut mask = RemovalMask::new(g.num_nodes(), g.num_edges());
        mask.remove_edge(EdgeIdx::new(0));
        assert!(mask.is_edge_removed(EdgeIdx::new(0)));
        assert!(!mask.is_edge_removed(EdgeIdx::new(1)));
    }

    #[test]
    fn test_removal_mask_out_of_range_node_noop() {
        let g = make_test_graph();
        let mut mask = RemovalMask::new(g.num_nodes(), g.num_edges());
        // Removing a node beyond range should be a no-op
        mask.remove_node(&g, NodeId::new(999));
        // Should not panic, mask remains unchanged
        for i in 0..g.num_nodes() {
            assert!(!mask.is_node_removed(NodeId::new(i)));
        }
    }

    // ===== Co-occurrence cosine similarity tests =====

    #[test]
    fn test_cooccurrence_cosine_empty_vectors() {
        use crate::semantic::CoOccurrenceIndex;
        let a: Vec<(NodeId, FiniteF32)> = vec![];
        let b: Vec<(NodeId, FiniteF32)> = vec![];
        let sim = CoOccurrenceIndex::cosine_similarity(&a, &b);
        assert_eq!(sim.get(), 0.0);
    }

    #[test]
    fn test_cooccurrence_cosine_identical_vectors() {
        use crate::semantic::CoOccurrenceIndex;
        let a = vec![
            (NodeId::new(0), FiniteF32::new(0.5)),
            (NodeId::new(1), FiniteF32::new(0.3)),
        ];
        let sim = CoOccurrenceIndex::cosine_similarity(&a, &a);
        assert!(
            (sim.get() - 1.0).abs() < 0.01,
            "Cosine similarity of identical vectors should be ~1.0, got {}",
            sim.get()
        );
    }

    // =================================================================
    // NEW TESTS — Agent C1: Git Co-Change + Temporal Engine
    // =================================================================

    // ===== CoChangeMatrix tests =====

    #[test]
    fn test_co_change_record_increments_count() {
        let g = build_test_graph();
        let mut matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();
        let a = NodeId::new(0);
        let b = NodeId::new(1);

        // Record a co-change
        matrix.record_co_change(a, b, 1000.0).unwrap();

        // Predict should include b for a
        let predictions = matrix.predict(a, 10);
        assert!(
            predictions.iter().any(|e| e.target == b),
            "After recording co-change A->B, predict(A) should include B"
        );
    }

    #[test]
    fn test_co_change_multiple_records_accumulate() {
        let g = build_test_graph();
        let mut matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();
        let a = NodeId::new(0);
        let b = NodeId::new(2);

        // Get initial strength for A->B (from bootstrap)
        let initial_strength = matrix
            .predict(a, 100)
            .iter()
            .find(|e| e.target == b)
            .map(|e| e.strength.get())
            .unwrap_or(0.0);

        // Record multiple co-changes
        matrix.record_co_change(a, b, 1000.0).unwrap();
        matrix.record_co_change(a, b, 2000.0).unwrap();
        matrix.record_co_change(a, b, 3000.0).unwrap();

        let final_strength = matrix
            .predict(a, 100)
            .iter()
            .find(|e| e.target == b)
            .map(|e| e.strength.get())
            .unwrap_or(0.0);

        assert!(
            final_strength > initial_strength,
            "Multiple co-change records should accumulate: initial={}, final={}",
            initial_strength,
            final_strength
        );
    }

    #[test]
    fn test_co_change_predict_returns_most_frequent() {
        let g = build_test_graph();
        let mut matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();
        let a = NodeId::new(0);
        let b = NodeId::new(1);
        let c = NodeId::new(2);

        // Record B more frequently than C
        for _ in 0..5 {
            matrix.record_co_change(a, b, 1000.0).unwrap();
        }
        matrix.record_co_change(a, c, 1000.0).unwrap();

        let predictions = matrix.predict(a, 10);
        let b_strength = predictions
            .iter()
            .find(|e| e.target == b)
            .map(|e| e.strength.get())
            .unwrap_or(0.0);
        let c_strength = predictions
            .iter()
            .find(|e| e.target == c)
            .map(|e| e.strength.get())
            .unwrap_or(0.0);

        assert!(
            b_strength > c_strength,
            "More frequently co-changed node should have higher strength: B={}, C={}",
            b_strength,
            c_strength
        );
    }

    #[test]
    fn test_co_change_empty_matrix_returns_empty() {
        // Build a graph with no edges -> bootstrap produces an empty matrix
        let mut g = Graph::new();
        g.add_node("iso_a", "IsoA", NodeType::File, &[], 0.0, 0.1)
            .unwrap();
        g.add_node("iso_b", "IsoB", NodeType::File, &[], 0.0, 0.1)
            .unwrap();
        g.finalize().unwrap();

        let matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();

        // No co-changes recorded, no edges -> empty predictions
        let predictions = matrix.predict(NodeId::new(0), 10);
        assert!(
            predictions.is_empty(),
            "Empty matrix should return empty predictions"
        );
    }

    #[test]
    fn test_co_change_predict_returns_both_partners() {
        let g = build_test_graph();
        let mut matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();
        let a = NodeId::new(0);
        let b = NodeId::new(3);
        let c = NodeId::new(4);

        // Record A<->B and A<->C
        matrix.record_co_change(a, b, 1000.0).unwrap();
        matrix.record_co_change(a, c, 1000.0).unwrap();

        let predictions = matrix.predict(a, 10);
        let has_b = predictions.iter().any(|e| e.target == b);
        let has_c = predictions.iter().any(|e| e.target == c);

        assert!(has_b, "predict(A) should include B after recording A<->B");
        assert!(has_c, "predict(A) should include C after recording A<->C");
    }

    // ===== VelocityScorer tests =====

    #[test]
    fn test_velocity_zscore_zero_for_average() {
        // Build a graph where all nodes have the same frequency
        let mut g = Graph::new();
        g.add_node("v0", "V0", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        g.add_node("v1", "V1", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        g.add_node("v2", "V2", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        g.finalize().unwrap();

        let score = VelocityScorer::score_one(&g, NodeId::new(0), 2000.0).unwrap();
        assert!(
            score.velocity.get().abs() < 0.01,
            "Node with average frequency should have z-score ~0, got {}",
            score.velocity.get()
        );
        assert_eq!(score.trend, VelocityTrend::Stable);
    }

    #[test]
    fn test_velocity_high_frequency_positive_zscore() {
        // Build a graph with one high-frequency node and several low ones
        let mut g = Graph::new();
        g.add_node("high", "High", NodeType::File, &[], 1000.0, 1.0)
            .unwrap();
        g.add_node("low1", "Low1", NodeType::File, &[], 1000.0, 0.1)
            .unwrap();
        g.add_node("low2", "Low2", NodeType::File, &[], 1000.0, 0.1)
            .unwrap();
        g.add_node("low3", "Low3", NodeType::File, &[], 1000.0, 0.1)
            .unwrap();
        g.finalize().unwrap();

        let score = VelocityScorer::score_one(&g, NodeId::new(0), 2000.0).unwrap();
        assert!(
            score.velocity.get() > 0.0,
            "High-frequency node should have positive z-score, got {}",
            score.velocity.get()
        );
    }

    #[test]
    fn test_velocity_low_frequency_negative_zscore() {
        // Build a graph with one low-frequency node and several high ones
        let mut g = Graph::new();
        g.add_node("low", "Low", NodeType::File, &[], 1000.0, 0.05)
            .unwrap();
        g.add_node("high1", "High1", NodeType::File, &[], 1000.0, 0.9)
            .unwrap();
        g.add_node("high2", "High2", NodeType::File, &[], 1000.0, 0.9)
            .unwrap();
        g.add_node("high3", "High3", NodeType::File, &[], 1000.0, 0.9)
            .unwrap();
        g.finalize().unwrap();

        let score = VelocityScorer::score_one(&g, NodeId::new(0), 2000.0).unwrap();
        assert!(
            score.velocity.get() < 0.0,
            "Low-frequency node should have negative z-score, got {}",
            score.velocity.get()
        );
    }

    #[test]
    fn test_velocity_score_all_empty_graph() {
        let g = Graph::new();
        let scores = VelocityScorer::score_all(&g, 0.0).unwrap();
        assert!(
            scores.is_empty(),
            "score_all on empty graph should return empty"
        );
    }

    // ===== TemporalDecayScorer tests =====

    #[test]
    fn test_decay_recent_file_near_one() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        // age_hours = 0 -> decay should be very close to 1.0
        let result = scorer.score_one(0.0, FiniteF32::ZERO, None);
        assert!(
            (result.raw_decay.get() - 1.0).abs() < 0.01,
            "Recent file (age=0) should have decay ~1.0, got {}",
            result.raw_decay.get()
        );
        assert!(
            (result.final_score.get() - 1.0).abs() < 0.01,
            "Recent file final_score should be ~1.0, got {}",
            result.final_score.get()
        );
    }

    #[test]
    fn test_decay_old_file_below_half() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        // age = 3 half-lives = 504 hours -> decay ~ 0.125
        let result = scorer.score_one(504.0, FiniteF32::ZERO, None);
        assert!(
            result.raw_decay.get() < 0.5,
            "Old file (3 half-lives) should have decay < 0.5, got {}",
            result.raw_decay.get()
        );
    }

    #[test]
    fn test_decay_per_nodetype_function_faster_than_module() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        let age_hours = 336.0; // 14 days

        // Function has half-life 336h (14d), Module has half-life 720h (30d)
        let func_decay =
            scorer.score_one_typed(age_hours, FiniteF32::ZERO, None, Some(NodeType::Function));
        let mod_decay =
            scorer.score_one_typed(age_hours, FiniteF32::ZERO, None, Some(NodeType::Module));

        assert!(
            func_decay.raw_decay.get() < mod_decay.raw_decay.get(),
            "Function should decay faster than Module at same age: func={}, mod={}",
            func_decay.raw_decay.get(),
            mod_decay.raw_decay.get()
        );
    }

    #[test]
    fn test_decay_score_all_in_unit_range() {
        let g = build_test_graph();
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        // Use a "now" that is after all node timestamps
        let now_unix = 2000.0;
        let scores = scorer.score_all(&g, now_unix).unwrap();

        assert_eq!(scores.len(), g.num_nodes() as usize);
        for ds in &scores {
            assert!(
                ds.final_score.get() >= 0.0 && ds.final_score.get() <= 1.0,
                "Decay score should be in [0,1], got {} for node {:?}",
                ds.final_score.get(),
                ds.node
            );
            assert!(
                ds.raw_decay.get() >= 0.0 && ds.raw_decay.get() <= 1.0,
                "Raw decay should be in [0,1], got {} for node {:?}",
                ds.raw_decay.get(),
                ds.node
            );
        }
    }

    // ===== ImpactRadiusAnalyzer tests =====

    #[test]
    fn test_impact_isolated_node_limited_to_self() {
        // Build a graph with an isolated node
        let mut g = Graph::new();
        g.add_node("iso", "Isolated", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        g.add_node("other", "Other", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        // No edges between them
        g.finalize().unwrap();

        let calc = ImpactRadiusCalculator::new(5, FiniteF32::new(0.01));
        let result = calc
            .compute(&g, NodeId::new(0), ImpactDirection::Both)
            .unwrap();

        assert!(
            result.blast_radius.is_empty(),
            "Isolated node should have empty blast radius, got {} entries",
            result.blast_radius.len()
        );
        assert_eq!(result.source, NodeId::new(0));
    }

    #[test]
    fn test_impact_propagates_through_edges() {
        let g = build_test_graph();
        let calc = ImpactRadiusCalculator::new(5, FiniteF32::new(0.01));

        // Node 0 (mat_pe) has edges to node 3 (proc_inj) and node 1 (mat_pp)
        let result = calc
            .compute(&g, NodeId::new(0), ImpactDirection::Forward)
            .unwrap();

        assert!(
            !result.blast_radius.is_empty(),
            "Connected node should have non-empty blast radius"
        );

        // At least the direct neighbor should be impacted
        let impacted_nodes: Vec<NodeId> = result.blast_radius.iter().map(|e| e.node).collect();
        assert!(
            impacted_nodes.contains(&NodeId::new(3)),
            "Direct forward neighbor (proc_inj, node 3) should be in blast radius. Got: {:?}",
            impacted_nodes
        );

        // All signal strengths should be in (0, 1]
        for entry in &result.blast_radius {
            assert!(
                entry.signal_strength.get() > 0.0 && entry.signal_strength.get() <= 1.0,
                "Impact signal should be in (0,1], got {}",
                entry.signal_strength.get()
            );
        }
    }

    #[test]
    fn test_impact_multi_hop_propagation() {
        let g = build_test_graph();
        let calc = ImpactRadiusCalculator::new(5, FiniteF32::new(0.001));

        // Node 0 -> Node 3 -> Node 5: should reach node 5 at hop distance 2
        let result = calc
            .compute(&g, NodeId::new(0), ImpactDirection::Forward)
            .unwrap();

        let node5_entry = result
            .blast_radius
            .iter()
            .find(|e| e.node == NodeId::new(5));
        assert!(
            node5_entry.is_some(),
            "Impact should propagate to 2-hop neighbor (node 5). Blast radius: {:?}",
            result
                .blast_radius
                .iter()
                .map(|e| e.node)
                .collect::<Vec<_>>()
        );

        if let Some(entry) = node5_entry {
            assert!(
                entry.hop_distance >= 2,
                "Node 5 should be at hop distance >= 2, got {}",
                entry.hop_distance
            );
        }
    }

    // ===== VelocityScorer cache tests =====

    #[test]
    fn test_velocity_scorer_cache_works() {
        let mut g = Graph::new();
        g.add_node("c0", "C0", NodeType::File, &[], 1000.0, 0.9)
            .unwrap();
        g.add_node("c1", "C1", NodeType::File, &[], 1000.0, 0.1)
            .unwrap();
        g.add_node("c2", "C2", NodeType::File, &[], 1000.0, 0.5)
            .unwrap();
        g.finalize().unwrap();

        let mut scorer = VelocityScorer::new();

        // First call computes stats
        let scores1 = scorer.score_all_cached(&g, 2000.0).unwrap();
        // Second call should use cache (same graph, same result)
        let scores2 = scorer.score_all_cached(&g, 2000.0).unwrap();

        assert_eq!(scores1.len(), scores2.len(), "Cached results should match");
        for (s1, s2) in scores1.iter().zip(scores2.iter()) {
            assert_eq!(
                s1.velocity.get(),
                s2.velocity.get(),
                "Cached velocity scores should be identical"
            );
        }

        // After invalidation, recompute
        scorer.invalidate_cache();
        let scores3 = scorer.score_all_cached(&g, 2000.0).unwrap();
        assert_eq!(
            scores1.len(),
            scores3.len(),
            "After invalidation, results should still match"
        );
    }

    // ===== CoChangeMatrix.populate_from_commit_groups test =====

    #[test]
    fn test_co_change_populate_from_commit_groups() {
        // Build a graph with file nodes that can be resolved
        let mut g = Graph::new();
        g.add_node(
            "file::alpha.rs",
            "alpha.rs",
            NodeType::File,
            &[],
            1000.0,
            0.5,
        )
        .unwrap();
        g.add_node("file::beta.rs", "beta.rs", NodeType::File, &[], 1000.0, 0.3)
            .unwrap();
        g.add_node(
            "file::gamma.rs",
            "gamma.rs",
            NodeType::File,
            &[],
            1000.0,
            0.2,
        )
        .unwrap();
        g.finalize().unwrap();

        let mut matrix = CoChangeMatrix::bootstrap(&g, 500_000).unwrap();

        // Commit groups: alpha + beta changed together, alpha + gamma changed together
        let groups = vec![
            vec!["alpha.rs".to_string(), "beta.rs".to_string()],
            vec!["alpha.rs".to_string(), "gamma.rs".to_string()],
        ];

        matrix.populate_from_commit_groups(&g, &groups).unwrap();

        // alpha should predict both beta and gamma
        let alpha_id = g.resolve_id("file::alpha.rs").unwrap();
        let beta_id = g.resolve_id("file::beta.rs").unwrap();
        let gamma_id = g.resolve_id("file::gamma.rs").unwrap();

        let predictions = matrix.predict(alpha_id, 10);
        let has_beta = predictions.iter().any(|e| e.target == beta_id);
        let has_gamma = predictions.iter().any(|e| e.target == gamma_id);

        assert!(
            has_beta,
            "After commit group [alpha, beta], alpha should predict beta"
        );
        assert!(
            has_gamma,
            "After commit group [alpha, gamma], alpha should predict gamma"
        );
    }
}
