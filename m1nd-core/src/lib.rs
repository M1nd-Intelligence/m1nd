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
pub mod snapshot_bin;
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
        let max_pr = (0..g.num_nodes() as usize)
            .map(|i| g.nodes.pagerank[i].get())
            .fold(0.0f32, f32::max);
        assert!(max_pr > 0.0, "PageRank should have non-zero values");
    }

    #[test]
    fn seed_finder_exact_match() {
        let g = build_test_graph();
        let seeds = SeedFinder::find_seeds(&g, "Polietileno", 200).unwrap();
        assert!(!seeds.is_empty(), "Should find at least one seed");
        assert_eq!(seeds[0].1.get(), 1.0, "Exact match should have relevance 1.0");
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

    #[test]
    fn bloom_filter_basic() {
        let mut bf = BloomFilter::with_capacity(1000, 0.01);
        bf.insert(NodeId::new(42));
        assert!(bf.probably_contains(NodeId::new(42)));
    }

    #[test]
    fn wavefront_single_seed() {
        let g = build_test_graph();
        let engine = WavefrontEngine::new();
        let config = PropagationConfig::default();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine.propagate(&g, &seeds, &config).unwrap();
        assert!(!result.scores.is_empty(), "Wavefront should activate at least one node");
        assert!(result.scores[0].1.get() > 0.0);
    }

    #[test]
    fn heap_single_seed() {
        let g = build_test_graph();
        let engine = HeapEngine::new();
        let config = PropagationConfig::default();
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = engine.propagate(&g, &seeds, &config).unwrap();
        assert!(!result.scores.is_empty(), "Heap should activate at least one node");
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
        let base = 0.5 * 0.35 + 0.5 * 0.25 + 0.5 * 0.15 + 0.5 * 0.25;
        let expected = base * RESONANCE_BONUS_4DIM;
        assert!((activated.activation.get() - expected).abs() < 0.01, "Expected ~{expected}, got {}", activated.activation.get());
    }

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

    #[test]
    fn temporal_decay_clamps_negative_age() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        let result = scorer.score_one(-10.0, FiniteF32::ZERO, None);
        assert!((result.raw_decay.get() - 1.0).abs() < 0.01, "Negative age should clamp to decay=1.0, got {}", result.raw_decay.get());
    }

    #[test]
    fn temporal_decay_exponential() {
        let scorer = TemporalDecayScorer::new(PosF32::new(168.0).unwrap());
        let result = scorer.score_one(168.0, FiniteF32::ZERO, None);
        assert!((result.raw_decay.get() - 0.5).abs() < 0.05, "After one half-life, decay ~0.5, got {}", result.raw_decay.get());
    }

    #[test]
    fn causal_chain_budget_limits() {
        let g = build_test_graph();
        let detector = CausalChainDetector::new(6, FiniteF32::new(0.01), 100);
        let chains = detector.detect(&g, NodeId::new(0)).unwrap();
        assert!(!chains.is_empty() || g.num_edges() == 0);
    }

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
        let _ = bridges;
    }

    #[test]
    fn spectral_gap_empty_graph() {
        let g = Graph::new();
        let analyzer = SpectralGapAnalyzer::with_defaults();
        let result = analyzer.analyze(&g);
        assert!(matches!(result, Err(M1ndError::EmptyGraph)));
    }

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
        assert!((amp - 1.0).abs() < 0.01, "Single pulse amplitude should be ~1.0");
    }

    #[test]
    fn standing_wave_propagation() {
        let g = build_test_graph();
        let propagator = StandingWavePropagator::new(5, FiniteF32::new(0.01), 10_000);
        let seeds = vec![(NodeId::new(0), FiniteF32::ONE)];
        let result = propagator.propagate(&g, &seeds, PosF32::new(1.0).unwrap(), PosF32::new(4.0).unwrap()).unwrap();
        assert!(result.pulses_processed > 0);
        assert!(!result.antinodes.is_empty());
    }

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
        assert_eq!(mem.len(), 3);
    }

    #[test]
    fn plasticity_generation_check() {
        let g = build_test_graph();
        let engine = PlasticityEngine::new(&g, PlasticityConfig::default());
        let _ = engine;
    }

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
}
