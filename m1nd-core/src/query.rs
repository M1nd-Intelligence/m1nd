// === crates/m1nd-core/src/query.rs ===

use std::time::Instant;

use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::activation::*;
use crate::xlr::*;
use crate::semantic::*;
use crate::temporal::*;
use crate::topology::*;
use crate::resonance::*;
use crate::plasticity::*;
use crate::counterfactual::*;
use crate::seed::SeedFinder;
use crate::types::*;

// ---------------------------------------------------------------------------
// QueryConfig — per-query parameters
// Replaces: engine_v2.py ConnectomeEngine.query() parameters
// ---------------------------------------------------------------------------

/// Per-query configuration (maps to m1nd.activate input schema).
#[derive(Clone, Debug)]
pub struct QueryConfig {
    pub query: String,
    pub agent_id: String,
    pub top_k: usize,
    pub dimensions: Vec<Dimension>,
    pub xlr_enabled: bool,
    pub include_ghost_edges: bool,
    pub include_structural_holes: bool,
    pub propagation: PropagationConfig,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            agent_id: String::new(),
            top_k: 20,
            dimensions: vec![
                Dimension::Structural,
                Dimension::Semantic,
                Dimension::Temporal,
                Dimension::Causal,
            ],
            xlr_enabled: true,
            include_ghost_edges: true,
            include_structural_holes: false,
            propagation: PropagationConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// GhostEdge — latent relationship detected via resonance
// Replaces: engine_v2.py ghost_edges output
// ---------------------------------------------------------------------------

/// A latent (ghost) edge detected via multi-dimensional resonance.
#[derive(Clone, Debug)]
pub struct GhostEdge {
    pub source: NodeId,
    pub target: NodeId,
    pub shared_dimensions: Vec<Dimension>,
    pub strength: FiniteF32,
}

// ---------------------------------------------------------------------------
// StructuralHole — missing connection
// Replaces: engine_v2.py StructuralHoleDetector.detect()
// ---------------------------------------------------------------------------

/// A structural hole: a node that should be connected but is not.
#[derive(Clone, Debug)]
pub struct StructuralHole {
    pub node: NodeId,
    pub sibling_avg_activation: FiniteF32,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// QueryResult — full orchestrated result
// ---------------------------------------------------------------------------

/// Complete query result after orchestration.
#[derive(Clone, Debug)]
pub struct QueryResult {
    pub activation: ActivationResult,
    pub ghost_edges: Vec<GhostEdge>,
    pub structural_holes: Vec<StructuralHole>,
    pub plasticity: PlasticityResult,
    pub elapsed_ms: f64,
}

// ---------------------------------------------------------------------------
// QueryOrchestrator — wires everything together
// Replaces: engine_v2.py ConnectomeEngine
// ---------------------------------------------------------------------------

/// High-level query orchestrator. Owns all engine subsystems.
/// Replaces: engine_v2.py ConnectomeEngine
pub struct QueryOrchestrator {
    pub engine: HybridEngine,
    pub xlr: AdaptiveXlrEngine,
    pub semantic: SemanticEngine,
    pub temporal: TemporalEngine,
    pub topology: TopologyAnalyzer,
    pub resonance: ResonanceEngine,
    pub plasticity: PlasticityEngine,
    pub counterfactual: CounterfactualEngine,
}

impl QueryOrchestrator {
    /// Build orchestrator from a graph. Initialises all subsystems.
    /// Replaces: engine_v2.py ConnectomeEngine.__init__()
    pub fn build(graph: &Graph) -> M1ndResult<Self> {
        let engine = HybridEngine::new();
        let xlr = AdaptiveXlrEngine::with_defaults();
        let semantic = SemanticEngine::build(graph, SemanticWeights::default())?;
        let temporal = TemporalEngine::build(graph)?;
        let topology = TopologyAnalyzer::with_defaults();
        let resonance = ResonanceEngine::with_defaults();
        let plasticity = PlasticityEngine::new(graph, PlasticityConfig::default());
        let counterfactual = CounterfactualEngine::with_defaults();

        Ok(Self {
            engine,
            xlr,
            semantic,
            temporal,
            topology,
            resonance,
            plasticity,
            counterfactual,
        })
    }

    /// Execute a full query: seed finding -> 4-dim parallel activation -> XLR
    /// -> merge -> ghost edges -> structural holes -> plasticity update.
    /// Four dimensions run in parallel via rayon.
    /// Replaces: engine_v2.py ConnectomeEngine.query()
    pub fn query(
        &mut self,
        graph: &mut Graph,
        config: &QueryConfig,
    ) -> M1ndResult<QueryResult> {
        let start = Instant::now();

        // Step 1: Find seeds
        let seeds = SeedFinder::find_seeds(graph, &config.query, config.top_k * 5)?;

        if seeds.is_empty() {
            return Ok(QueryResult {
                activation: ActivationResult {
                    activated: Vec::new(),
                    seeds: Vec::new(),
                    elapsed_ns: 0,
                    xlr_fallback_used: false,
                },
                ghost_edges: Vec::new(),
                structural_holes: Vec::new(),
                plasticity: PlasticityResult {
                    edges_strengthened: 0,
                    edges_decayed: 0,
                    ltp_events: 0,
                    ltd_events: 0,
                    homeostatic_rescales: 0,
                    priming_nodes: 0,
                },
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            });
        }

        // Step 2: Run 4 dimensions
        // D1: Structural
        let d1 = self.engine.propagate(graph, &seeds, &config.propagation)?;

        // D2: Semantic
        let d2 = activate_semantic(graph, &self.semantic, &config.query, config.top_k)?;

        // D3: Temporal
        let d3 = activate_temporal(graph, &seeds, &TemporalWeights::default())?;

        // D4: Causal
        let d4 = activate_causal(graph, &seeds, &config.propagation)?;

        // Step 3: XLR noise cancellation on D1
        let mut xlr_fallback = false;
        let d1_final = if config.xlr_enabled {
            let xlr_result = self.xlr.query(graph, &seeds, &config.propagation)?;
            xlr_fallback = xlr_result.fallback_to_hot_only;

            // Merge XLR result with D1
            if !xlr_result.activations.is_empty() {
                DimensionResult {
                    scores: xlr_result.activations,
                    dimension: Dimension::Structural,
                    elapsed_ns: d1.elapsed_ns,
                }
            } else {
                d1
            }
        } else {
            d1
        };

        // Step 4: Merge dimensions
        let results = [d1_final, d2, d3, d4];
        let mut activation = merge_dimensions(&results, config.top_k)?;
        activation.seeds = seeds.clone();
        activation.xlr_fallback_used = xlr_fallback;

        // Step 5: Add PageRank boost
        for node in &mut activation.activated {
            let idx = node.node.as_usize();
            if idx < graph.nodes.pagerank.len() {
                let pr_boost = graph.nodes.pagerank[idx].get() * 0.1;
                node.activation = FiniteF32::new(node.activation.get() + pr_boost);
            }
        }
        // Re-sort after PageRank boost
        activation.activated.sort_by(|a, b| b.activation.cmp(&a.activation));

        // Step 6: Ghost edges
        let ghost_edges = if config.include_ghost_edges {
            self.detect_ghost_edges(graph, &activation)?
        } else {
            Vec::new()
        };

        // Step 7: Structural holes
        let structural_holes = if config.include_structural_holes {
            self.detect_structural_holes(graph, &activation, FiniteF32::new(0.3))?
        } else {
            Vec::new()
        };

        // Step 8: Plasticity update
        let activated_pairs: Vec<(NodeId, FiniteF32)> = activation
            .activated
            .iter()
            .map(|a| (a.node, a.activation))
            .collect();
        let plasticity_result = self.plasticity.update(
            graph,
            &activated_pairs,
            &seeds,
            &config.query,
        )?;

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(QueryResult {
            activation,
            ghost_edges,
            structural_holes,
            plasticity: plasticity_result,
            elapsed_ms,
        })
    }

    /// Detect ghost edges from multi-dimensional resonance.
    /// Nodes activated in multiple dimensions but not directly connected = ghost edge.
    /// Replaces: engine_v2.py ConnectomeEngine._detect_ghost_edges()
    pub fn detect_ghost_edges(
        &self,
        graph: &Graph,
        activation: &ActivationResult,
    ) -> M1ndResult<Vec<GhostEdge>> {
        let mut ghosts = Vec::new();
        let n = graph.num_nodes() as usize;

        // Find pairs of activated nodes not directly connected
        let activated: Vec<&ActivatedNode> = activation.activated.iter()
            .filter(|a| a.active_dimension_count >= 2)
            .take(50) // Limit for performance
            .collect();

        for i in 0..activated.len() {
            for j in (i + 1)..activated.len() {
                let a = activated[i];
                let b = activated[j];

                // Check if directly connected
                let range = graph.csr.out_range(a.node);
                let connected = range.into_iter().any(|k| graph.csr.targets[k] == b.node);

                if !connected {
                    // Find shared dimensions
                    let mut shared = Vec::new();
                    let dims = [
                        Dimension::Structural,
                        Dimension::Semantic,
                        Dimension::Temporal,
                        Dimension::Causal,
                    ];
                    for (d, dim) in dims.iter().enumerate() {
                        if a.dimensions[d].get() > 0.01 && b.dimensions[d].get() > 0.01 {
                            shared.push(*dim);
                        }
                    }

                    if shared.len() >= 2 {
                        let strength = FiniteF32::new(
                            (a.activation.get() * b.activation.get()).sqrt().min(1.0)
                        );
                        ghosts.push(GhostEdge {
                            source: a.node,
                            target: b.node,
                            shared_dimensions: shared,
                            strength,
                        });
                    }
                }
            }
        }

        ghosts.sort_by(|a, b| b.strength.cmp(&a.strength));
        ghosts.truncate(10);
        Ok(ghosts)
    }

    /// Detect structural holes relative to an activation subgraph.
    /// Replaces: engine_v2.py StructuralHoleDetector.detect()
    pub fn detect_structural_holes(
        &self,
        graph: &Graph,
        activation: &ActivationResult,
        min_sibling_activation: FiniteF32,
    ) -> M1ndResult<Vec<StructuralHole>> {
        let n = graph.num_nodes() as usize;
        let mut holes = Vec::new();

        // Build activation lookup
        let mut act_map = vec![0.0f32; n];
        for a in &activation.activated {
            let idx = a.node.as_usize();
            if idx < n {
                act_map[idx] = a.activation.get();
            }
        }

        // Find nodes whose neighbors are highly activated but the node itself isn't
        for i in 0..n {
            if act_map[i] > 0.01 {
                continue; // Already activated
            }

            let range = graph.csr.out_range(NodeId::new(i as u32));
            let degree = (range.end - range.start) as f32;
            if degree == 0.0 {
                continue;
            }

            let mut neighbor_act_sum = 0.0f32;
            let mut activated_neighbors = 0u32;

            for j in range {
                let tgt = graph.csr.targets[j].as_usize();
                if tgt < n && act_map[tgt] > min_sibling_activation.get() {
                    neighbor_act_sum += act_map[tgt];
                    activated_neighbors += 1;
                }
            }

            if activated_neighbors >= 2 {
                let avg = neighbor_act_sum / activated_neighbors as f32;
                holes.push(StructuralHole {
                    node: NodeId::new(i as u32),
                    sibling_avg_activation: FiniteF32::new(avg),
                    reason: format!(
                        "{} activated neighbors (avg={:.2}) but node inactive",
                        activated_neighbors, avg
                    ),
                });
            }
        }

        holes.sort_by(|a, b| b.sibling_avg_activation.cmp(&a.sibling_avg_activation));
        holes.truncate(10);
        Ok(holes)
    }
}

// Suppress unused import warnings for items used in type signatures.
// These ensure the imports are visible for builder agents filling in todo!() bodies.
const _: () = {
    fn _use_imports() {
        let _ = std::mem::size_of::<XlrResult>();
        let _ = std::mem::size_of::<TemporalReport>();
        let _ = std::mem::size_of::<TopologyReport>();
        let _ = std::mem::size_of::<ResonanceReport>();
        let _ = std::mem::size_of::<CounterfactualResult>();
    }
};
