#![allow(unused_mut, unused_variables)]
// === m1nd-core criterion benchmarks ===
//
// 10+ benchmarks covering activation, impact, antibody, flow, epidemic,
// tremor, trust, layer, graph build, and resonance.
//
// Run: cargo bench -p m1nd-core

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use m1nd_core::activation::{ActivationEngine, WavefrontEngine};
use m1nd_core::antibody::{
    scan_antibodies, Antibody, AntibodyPattern, AntibodySeverity, PatternEdge, PatternNode,
};
use m1nd_core::builder::GraphBuilder;
use m1nd_core::epidemic::{EpidemicConfig, EpidemicDirection, EpidemicEngine};
use m1nd_core::flow::{FlowConfig, FlowEngine};
use m1nd_core::graph::Graph;
use m1nd_core::layer::LayerDetector;
use m1nd_core::resonance::{HarmonicAnalyzer, StandingWavePropagator};
use m1nd_core::tremor::{TremorRegistry, TremorWindow};
use m1nd_core::trust::{TrustLedger, TrustSortBy};
use m1nd_core::types::*;

// ── Test graph builder ──────────────────────────────────────────────────

/// Build a realistic 1000-node graph with ~3000 edges.
/// Structure: 10 modules x 100 nodes each.
/// Each module has internal chain + cross-module "imports" edges.
fn build_1k_graph() -> Graph {
    let mut b = GraphBuilder::with_capacity(1000, 3500);
    let mut nodes = Vec::with_capacity(1000);

    let types = [
        NodeType::File,
        NodeType::Function,
        NodeType::Class,
        NodeType::Module,
        NodeType::Struct,
    ];
    let relations = ["imports", "calls", "references", "contains", "inherits"];

    // Create 10 modules x 100 nodes
    for module in 0..10u32 {
        for i in 0..100u32 {
            let global_idx = module * 100 + i;
            let nt = types[(global_idx as usize) % types.len()];
            let id = format!("mod{}::node_{}", module, global_idx);
            let label = format!("handle_{}_component_{}", module, i);
            let tags: Vec<&str> = match nt {
                NodeType::Function => vec!["async", "handler"],
                NodeType::File => vec!["source"],
                NodeType::Class => vec!["model"],
                _ => vec![],
            };
            let nid = b.add_node(&id, &label, nt, &tags).expect("add_node failed");
            nodes.push(nid);
        }
    }

    // Intra-module chain: node[i] -> node[i+1] within each module
    for module in 0..10u32 {
        for i in 0..99u32 {
            let src = (module * 100 + i) as usize;
            let tgt = (module * 100 + i + 1) as usize;
            let rel = relations[i as usize % relations.len()];
            b.add_edge(nodes[src], nodes[tgt], rel, 0.5 + (i as f32) * 0.005)
                .expect("add_edge failed");
        }
    }

    // Cross-module edges: ~20 edges between adjacent modules
    for module in 0..9u32 {
        for link in 0..20u32 {
            let src_idx = (module * 100 + link * 5) as usize;
            let tgt_idx = ((module + 1) * 100 + link * 5 + 2) as usize;
            if src_idx < nodes.len() && tgt_idx < nodes.len() {
                let rel = relations[link as usize % relations.len()];
                b.add_edge(
                    nodes[src_idx],
                    nodes[tgt_idx],
                    rel,
                    0.3 + (link as f32) * 0.02,
                )
                .expect("cross edge failed");
            }
        }
    }

    // Hub nodes: a few high-connectivity nodes (index 0, 500, 999)
    for &hub in &[0usize, 500, 999] {
        for spoke in (0..1000).step_by(50) {
            if spoke != hub && spoke < nodes.len() {
                let _ = b.add_edge(nodes[hub], nodes[spoke], "references", 0.4);
            }
        }
    }

    b.finalize().expect("finalize failed")
}

/// Build a small 500-node graph for lighter benchmarks.
fn build_500_graph() -> Graph {
    let mut b = GraphBuilder::with_capacity(500, 1500);
    let mut nodes = Vec::with_capacity(500);

    for i in 0..500u32 {
        let nt = if i % 5 == 0 {
            NodeType::Function
        } else if i % 7 == 0 {
            NodeType::Class
        } else {
            NodeType::File
        };
        let id = format!("node_{}", i);
        let label = format!("handle_component_{}", i);
        let nid = b.add_node(&id, &label, nt, &[]).expect("add_node");
        nodes.push(nid);
    }

    // Chain
    for i in 0..499u32 {
        b.add_edge(nodes[i as usize], nodes[(i + 1) as usize], "calls", 0.5)
            .expect("edge");
    }

    // Cross links every 10
    for i in (0..490).step_by(10) {
        let _ = b.add_edge(nodes[i], nodes[i + 7], "imports", 0.4);
    }

    b.finalize().expect("finalize")
}

// ── Benchmarks ──────────────────────────────────────────────────────────

fn bench_graph_build_1k(c: &mut Criterion) {
    c.bench_function("graph_build_1k", |b| {
        b.iter(|| {
            let graph = black_box(build_1k_graph());
            assert!(graph.num_nodes() == 1000);
        });
    });
}

fn bench_activate_1k_nodes(c: &mut Criterion) {
    let graph = build_1k_graph();
    let engine = WavefrontEngine::new();
    let config = PropagationConfig::default();
    let seeds = vec![
        (NodeId::new(0), FiniteF32::new(1.0)),
        (NodeId::new(500), FiniteF32::new(0.8)),
    ];

    c.bench_function("activate_1k_nodes", |b| {
        b.iter(|| {
            let result = engine
                .propagate(black_box(&graph), black_box(&seeds), black_box(&config))
                .unwrap();
            black_box(&result);
        });
    });
}

fn bench_impact_depth3(c: &mut Criterion) {
    // Impact analysis = activation from a single seed, measuring blast radius
    let graph = build_1k_graph();
    let engine = WavefrontEngine::new();
    let config = PropagationConfig {
        max_depth: 3,
        ..PropagationConfig::default()
    };
    let seeds = vec![(NodeId::new(100), FiniteF32::new(1.0))];

    c.bench_function("impact_depth3", |b| {
        b.iter(|| {
            let result = engine
                .propagate(black_box(&graph), black_box(&seeds), black_box(&config))
                .unwrap();
            black_box(result.scores.len());
        });
    });
}

fn bench_antibody_scan_50_patterns(c: &mut Criterion) {
    let graph = build_1k_graph();

    // Create 50 antibody patterns of varying complexity
    let mut antibodies: Vec<Antibody> = (0..50)
        .map(|i| {
            let nodes = vec![
                PatternNode {
                    role: format!("src_{}", i),
                    node_type: Some("Function".to_string()),
                    required_tags: vec![],
                    label_contains: Some(format!("handle_{}", i % 10)),
                },
                PatternNode {
                    role: format!("tgt_{}", i),
                    node_type: Some("File".to_string()),
                    required_tags: vec![],
                    label_contains: None,
                },
            ];
            let edges = vec![PatternEdge {
                source_idx: 0,
                target_idx: 1,
                relation: Some("calls".to_string()),
            }];

            Antibody {
                id: format!("ab-{}", i),
                name: format!("test_pattern_{}", i),
                description: "bench pattern".to_string(),
                pattern: AntibodyPattern {
                    nodes,
                    edges,
                    negative_edges: vec![],
                },
                severity: AntibodySeverity::Warning,
                match_count: 0,
                created_at: 1000.0,
                last_match_at: None,
                created_by: "bench".to_string(),
                source_query: "bench".to_string(),
                source_nodes: vec![],
                enabled: true,
                specificity: 0.5,
            }
        })
        .collect();

    c.bench_function("antibody_scan_50_patterns", |b| {
        b.iter(|| {
            // Clone antibodies each iteration since scan mutates them
            let mut abs = antibodies.clone();
            let result = scan_antibodies(
                black_box(&graph),
                black_box(&mut abs),
                "all",
                0,
                100,
                AntibodySeverity::Info,
                None,
                10,
                "substring",
                0.0,
            );
            black_box(&result);
        });
    });
}

fn bench_flow_simulate_4_particles(c: &mut Criterion) {
    let graph = build_500_graph();
    let engine = FlowEngine::new();
    let config = FlowConfig {
        max_depth: 8,
        include_paths: false,
        max_total_steps: 20_000,
        ..FlowConfig::with_defaults()
    };

    // Use a few function nodes as entry points
    let entries: Vec<NodeId> = (0..500u32)
        .filter(|i| i % 5 == 0)
        .take(4)
        .map(NodeId::new)
        .collect();

    c.bench_function("flow_simulate_4_particles", |b| {
        b.iter(|| {
            let result = engine
                .simulate(
                    black_box(&graph),
                    black_box(&entries),
                    black_box(2),
                    black_box(&config),
                )
                .unwrap();
            black_box(&result.summary);
        });
    });
}

fn bench_epidemic_sir_50_iterations(c: &mut Criterion) {
    let graph = build_500_graph();
    let engine = EpidemicEngine::new();
    let config = EpidemicConfig {
        infection_rate: Some(0.3),
        recovery_rate: 0.0,
        iterations: 50,
        direction: EpidemicDirection::Forward,
        top_k: 20,
        burnout_threshold: 0.95,
        promotion_threshold: 0.5,
    };
    let infected = vec![NodeId::new(0), NodeId::new(250)];

    c.bench_function("epidemic_sir_50_iterations", |b| {
        b.iter(|| {
            let result = engine
                .simulate(
                    black_box(&graph),
                    black_box(&infected),
                    black_box(&[]),
                    black_box(&config),
                )
                .unwrap();
            black_box(&result.summary);
        });
    });
}

fn bench_tremor_detect_500_nodes(c: &mut Criterion) {
    // Populate a registry with 500 nodes, each having 10 observations
    let mut registry = TremorRegistry::with_defaults();
    for i in 0..500u32 {
        let id = format!("node_{}", i);
        let base_time = 1_000_000.0;
        for j in 0..10u32 {
            let delta = (i as f32 + j as f32) * 0.1;
            registry.record_observation(&id, delta, (j + 1) as u16, base_time + j as f64 * 3600.0);
        }
    }

    c.bench_function("tremor_detect_500_nodes", |b| {
        b.iter(|| {
            let result = registry.analyze(
                black_box(TremorWindow::All),
                black_box(0.0),
                black_box(20),
                black_box(None),
                black_box(2_000_000.0),
                black_box(0),
            );
            black_box(&result.tremors);
        });
    });
}

fn bench_trust_report_500_nodes(c: &mut Criterion) {
    let mut ledger = TrustLedger::new();
    let now = 1_000_000.0;

    // Populate 500 entries with varied defect histories
    for i in 0..500u32 {
        let id = format!("file::module_{}.py", i);
        for j in 0..(i % 10 + 1) {
            ledger.record_defect(&id, now - (j as f64) * 3600.0);
        }
        if i % 3 == 0 {
            ledger.record_false_alarm(&id, now - 100.0);
        }
        if i % 5 == 0 {
            ledger.record_partial(&id, now - 50.0);
        }
    }

    c.bench_function("trust_report_500_nodes", |b| {
        b.iter(|| {
            let result = ledger.report(
                black_box("all"),
                black_box(1),
                black_box(50),
                black_box(None),
                black_box(TrustSortBy::TrustAsc),
                black_box(now),
                black_box(720.0),
                black_box(3.0),
            );
            black_box(&result.summary);
        });
    });
}

fn bench_layer_detect_500_nodes(c: &mut Criterion) {
    let graph = build_500_graph();
    let detector = LayerDetector::with_defaults();

    c.bench_function("layer_detect_500_nodes", |b| {
        b.iter(|| {
            let result = detector
                .detect(
                    black_box(&graph),
                    black_box(None),
                    black_box(&[]),
                    black_box(false),
                    black_box("auto"),
                )
                .unwrap();
            black_box(result.layers.len());
        });
    });
}

fn bench_resonance_5_harmonics(c: &mut Criterion) {
    let graph = build_500_graph();
    let propagator = StandingWavePropagator::new(8, FiniteF32::new(0.01), 10_000);
    let analyzer = HarmonicAnalyzer::new(
        StandingWavePropagator::new(8, FiniteF32::new(0.01), 10_000),
        5,
    );

    let seeds = vec![
        (NodeId::new(0), FiniteF32::new(1.0)),
        (NodeId::new(250), FiniteF32::new(0.8)),
    ];
    let base_freq = PosF32::new(1.0).unwrap();
    let base_wl = PosF32::new(4.0).unwrap();

    c.bench_function("resonance_5_harmonics", |b| {
        b.iter(|| {
            let result = analyzer
                .analyze(
                    black_box(&graph),
                    black_box(&seeds),
                    black_box(base_freq),
                    black_box(base_wl),
                )
                .unwrap();
            black_box(result.harmonics.len());
        });
    });
}

criterion_group!(
    benches,
    bench_graph_build_1k,
    bench_activate_1k_nodes,
    bench_impact_depth3,
    bench_antibody_scan_50_patterns,
    bench_flow_simulate_4_particles,
    bench_epidemic_sir_50_iterations,
    bench_tremor_detect_500_nodes,
    bench_trust_report_500_nodes,
    bench_layer_detect_500_nodes,
    bench_resonance_5_harmonics,
);

criterion_main!(benches);
