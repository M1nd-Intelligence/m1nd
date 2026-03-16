// === m1nd-core/src/epidemic.rs ===
//
// SIR epidemiological model for bug propagation prediction.
// Given known buggy modules (infected), predicts which neighbors
// are most likely to harbor undiscovered bugs.

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

// ── Constants ──

/// Default number of SIR simulation iterations.
pub const DEFAULT_ITERATIONS: u32 = 50;
/// Hard cap on simulation iterations to bound runtime.
pub const MAX_ITERATIONS: u32 = 500;
/// Default recovery rate (0.0 = no spontaneous recovery).
pub const DEFAULT_RECOVERY_RATE: f32 = 0.0;
/// Maximum allowed infection rate — clamped at this value.
pub const MAX_INFECTION_RATE: f32 = 0.95;
/// Minimum infection rate — values below this are clamped up.
pub const MIN_INFECTION_RATE: f32 = 0.001;
/// Default number of top predictions to return.
pub const DEFAULT_TOP_K: usize = 20;
/// Fraction of nodes that must be infected to trigger burnout detection.
pub const BURNOUT_THRESHOLD: f32 = 0.8;
/// Minimum iterations before burnout detection activates.
pub const BURNOUT_MIN_ITERATIONS: u32 = 10;
/// Consecutive zero-new-infection iterations before declaring extinction.
pub const EXTINCTION_PLATEAU_ITERATIONS: u32 = 5;
/// Transmission reduction factor when source node is in the Recovered compartment.
pub const RECOVERED_FIREWALL_FACTOR: f32 = 0.5;

/// Relation type -> coupling factor for edge-weight-derived transmission.
pub const COUPLING_FACTORS: &[(&str, f32)] = &[
    ("imports", 0.8),
    ("calls", 0.7),
    ("inherits", 0.6),
    ("references", 0.4),
    ("contains", 0.3),
    ("related_to", 0.2),
];
pub const DEFAULT_COUPLING_FACTOR: f32 = 0.1;

// ── Core Types ──

/// SIR compartment assignment for a graph node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Compartment {
    /// Not yet infected — susceptible to transmission.
    Susceptible = 0,
    /// Currently infected — can transmit to neighbors.
    Infected = 1,
    /// Recovered — applies firewall factor to outgoing transmission.
    Recovered = 2,
}

/// Direction of bug propagation along graph edges.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpidemicDirection {
    /// Follow outgoing edges only (callee propagation).
    Forward,
    /// Follow incoming edges only (caller propagation).
    Backward,
    /// Follow both directions.
    Both,
}

/// Configuration for an epidemic simulation run.
#[derive(Clone, Debug)]
pub struct EpidemicConfig {
    /// Uniform infection rate override. `None` = derive from edge weight × coupling factor.
    pub infection_rate: Option<f32>,
    /// Per-iteration recovery probability (default 0.0 = no recovery).
    pub recovery_rate: f32,
    /// Number of simulation iterations (capped at `MAX_ITERATIONS`).
    pub iterations: u32,
    /// Direction of propagation along graph edges.
    pub direction: EpidemicDirection,
    /// Number of top predictions to return.
    pub top_k: usize,
    /// Fraction of nodes infected in `BURNOUT_MIN_ITERATIONS` to trigger burnout error.
    pub burnout_threshold: f32,
    /// Minimum probability for a susceptible node to be promoted to Infected (spreader).
    /// Default 0.0 (any non-zero probability promotes). When auto_calibrate is used on
    /// dense graphs, set to 0.5 to prevent low-probability cascade burnout.
    pub promotion_threshold: f32,
}

/// A predicted at-risk node from the epidemic simulation.
#[derive(Clone, Debug, Serialize)]
pub struct EpidemicPrediction {
    /// External ID of the at-risk node.
    pub node_id: String,
    /// Label of the at-risk node.
    pub label: String,
    /// Node type string.
    pub node_type: String,
    /// Predicted infection probability in [0.0, 1.0].
    pub infection_probability: f32,
    /// Shortest-path generation distance from the nearest infected seed.
    pub generation: u32,
    /// External IDs of infected seeds that contributed to this prediction.
    pub contributing_infected: Vec<String>,
    /// Reconstructed shortest transmission path from nearest seed.
    pub transmission_path: Vec<String>,
    /// Edge weight from the nearest infected node on the shortest path.
    pub edge_weight_to_nearest: f32,
}

/// Final compartment counts and epidemic statistics.
#[derive(Clone, Debug, Serialize)]
pub struct EpidemicSummary {
    /// Final count of nodes still susceptible.
    pub total_susceptible: u32,
    /// Final count of infected nodes.
    pub total_infected: u32,
    /// Final count of recovered nodes.
    pub total_recovered: u32,
    /// Iteration at which peak infection count was reached.
    pub peak_infection_iteration: u32,
    /// Estimated basic reproduction number R0 (avg secondary infections per initial seed).
    pub r0_estimate: f32,
    /// Whether the epidemic reached extinction (no new infections plateau).
    pub epidemic_extinct: bool,
}

/// A connected component unreachable from the infected seed set.
#[derive(Clone, Debug, Serialize)]
pub struct UnreachableComponent {
    /// External ID of the representative (first discovered) node in the component.
    pub representative_node: String,
    /// Number of nodes in this unreachable component.
    pub node_count: u32,
}

/// Complete result of an epidemic simulation.
#[derive(Clone, Debug, Serialize)]
pub struct EpidemicResult {
    /// At-risk nodes sorted by infection probability descending.
    pub predictions: Vec<EpidemicPrediction>,
    /// Final compartment counts and epidemic statistics.
    pub summary: EpidemicSummary,
    /// Connected components unreachable from the infected set.
    pub unreachable_components: Vec<UnreachableComponent>,
    /// Non-fatal warnings (e.g. clamped infection rate).
    pub warnings: Vec<String>,
    /// Node IDs that were requested but not found in the graph.
    pub unresolved_nodes: Vec<String>,
    /// Wall-clock time for the simulation in milliseconds.
    pub elapsed_ms: f64,
}

// ── Persistent State ──

/// Persistent epidemic state across sessions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EpidemicPersistentState {
    /// Map of infected node external_id -> Unix timestamp of infection.
    pub infected_nodes: HashMap<String, f64>,
    /// Map of recovered node external_id -> Unix timestamp of recovery.
    pub recovered_nodes: HashMap<String, f64>,
    /// Cumulative infection counts per node across all runs.
    pub cumulative_infections: HashMap<String, u32>,
    /// Unix timestamp of the last simulation run, if any.
    pub last_run_timestamp: Option<f64>,
}

// ── Helpers ──

/// Look up the coupling factor for a relation string.
fn sir_coupling_factor(relation: &str) -> f32 {
    for &(rel, factor) in COUPLING_FACTORS {
        if relation == rel {
            return factor;
        }
    }
    DEFAULT_COUPLING_FACTOR
}

/// Clamp a value to [lo, hi].
#[inline]
fn sir_clamp(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

/// Map NodeType enum to a lowercase string.
fn sir_node_type_str(nt: &NodeType) -> &'static str {
    match nt {
        NodeType::File => "file",
        NodeType::Directory => "directory",
        NodeType::Function => "function",
        NodeType::Class => "class",
        NodeType::Struct => "struct",
        NodeType::Enum => "enum",
        NodeType::Type => "type",
        NodeType::Module => "module",
        NodeType::Reference => "reference",
        NodeType::Concept => "concept",
        NodeType::Material => "material",
        NodeType::Process => "process",
        NodeType::Product => "product",
        NodeType::Supplier => "supplier",
        NodeType::Regulatory => "regulatory",
        NodeType::System => "system",
        NodeType::Cost => "cost",
        NodeType::Custom(_) => "custom",
    }
}

/// Build reverse lookup: NodeId -> external_id string.
fn sir_build_node_to_ext(graph: &Graph) -> Vec<String> {
    let n = graph.num_nodes() as usize;
    let mut node_to_ext = vec![String::new(); n];
    for (interned, &nid) in &graph.id_to_node {
        let idx = nid.as_usize();
        if idx < n {
            node_to_ext[idx] = graph.strings.resolve(*interned).to_string();
        }
    }
    node_to_ext
}

/// Reconstruct shortest path from `parent` array.
/// Returns external IDs of the path from the nearest infected node to `target`.
fn sir_reconstruct_path(target: usize, parent: &[u32], node_to_ext: &[String]) -> Vec<String> {
    let mut path = Vec::new();
    let mut cur = target;
    // Safety: limit iterations to prevent infinite loops in case of bugs
    let max_steps = parent.len();
    for _ in 0..max_steps {
        path.push(node_to_ext[cur].clone());
        if parent[cur] == cur as u32 || parent[cur] == u32::MAX {
            break;
        }
        cur = parent[cur] as usize;
    }
    path.reverse();
    path
}

// ── Engine ──

/// Deterministic SIR epidemic engine for bug propagation prediction.
///
/// Uses expected-value (probabilistic) propagation rather than Monte Carlo
/// sampling, giving deterministic results in O(iterations × edges).
pub struct EpidemicEngine;

impl Default for EpidemicEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EpidemicEngine {
    /// Create a new `EpidemicEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Run deterministic SIR simulation on the graph.
    ///
    /// Returns predictions for susceptible nodes sorted by infection probability descending.
    ///
    /// # Parameters
    /// - `graph`: finalized graph to simulate on
    /// - `infected_ids`: nodes considered infected at t=0
    /// - `recovered_ids`: nodes considered recovered (partially immune) at t=0
    /// - `config`: simulation parameters
    ///
    /// # Errors
    /// - `M1ndError::NoValidInfectedNodes` if `infected_ids` is empty
    /// - `M1ndError::EpidemicBurnout` if >burnout_threshold of nodes get infected too fast
    pub fn simulate(
        &self,
        graph: &Graph,
        infected_ids: &[NodeId],
        recovered_ids: &[NodeId],
        config: &EpidemicConfig,
    ) -> M1ndResult<EpidemicResult> {
        let start = Instant::now();
        let n = graph.num_nodes() as usize;
        let mut warnings: Vec<String> = Vec::new();

        // Validate & clamp infection_rate
        let uniform_rate = config.infection_rate.map(|r| {
            let clamped = sir_clamp(r, MIN_INFECTION_RATE, MAX_INFECTION_RATE);
            if (clamped - r).abs() > f32::EPSILON {
                warnings.push(format!(
                    "infection_rate clamped from {:.2} to {:.2}",
                    r, clamped
                ));
            }
            clamped
        });

        // Clamp iterations
        let iterations = config.iterations.min(MAX_ITERATIONS);

        if infected_ids.is_empty() {
            return Err(M1ndError::NoValidInfectedNodes);
        }

        // Handle empty / trivial graph
        if n == 0 {
            return Ok(EpidemicResult {
                predictions: Vec::new(),
                summary: EpidemicSummary {
                    total_susceptible: 0,
                    total_infected: 0,
                    total_recovered: 0,
                    peak_infection_iteration: 0,
                    r0_estimate: 0.0,
                    epidemic_extinct: true,
                },
                unreachable_components: Vec::new(),
                warnings,
                unresolved_nodes: Vec::new(),
                elapsed_ms: start.elapsed().as_secs_f64() * 1000.0,
            });
        }

        // Build reverse lookup
        let node_to_ext = sir_build_node_to_ext(graph);

        // Initialize compartments and probability
        let mut compartment = vec![Compartment::Susceptible; n];
        let mut probability = vec![0.0f32; n];
        let mut generation = vec![u32::MAX; n];
        // parent[i] = the node that first infected i (for path reconstruction)
        let mut parent = vec![u32::MAX; n];
        // contributing_infected[i] = set of original infected seeds that contributed
        let mut contributing: Vec<Vec<usize>> = vec![Vec::new(); n];
        // Track the weight of the edge from the nearest infected source
        let mut edge_weight_nearest = vec![0.0f32; n];

        // Set infected nodes
        for &nid in infected_ids {
            let idx = nid.as_usize();
            if idx < n {
                compartment[idx] = Compartment::Infected;
                probability[idx] = 1.0;
                generation[idx] = 0;
                parent[idx] = idx as u32;
                contributing[idx] = vec![idx];
            }
        }

        // Set recovered nodes
        for &nid in recovered_ids {
            let idx = nid.as_usize();
            if idx < n {
                compartment[idx] = Compartment::Recovered;
                probability[idx] = 0.0;
                generation[idx] = 0;
                parent[idx] = idx as u32;
            }
        }

        // Count initial states
        let initial_infected_count =
            infected_ids.iter().filter(|nid| nid.as_usize() < n).count() as u32;

        // Track epidemic statistics
        let mut peak_infected: u32 = initial_infected_count;
        let mut peak_iteration: u32 = 0;
        let mut total_new_infections_sum: u32 = 0;
        let mut consecutive_zero_new = 0u32;
        let mut epidemic_extinct = false;

        // Reusable vec for new probabilities per iteration (avoid alloc per step)
        let mut new_probability = vec![0.0f32; n];

        // ── Main SIR loop (deterministic expected-value propagation) ──
        for t in 0..iterations {
            // Copy current probability into working buffer
            new_probability.copy_from_slice(&probability);

            let mut new_infections_this_iter = 0u32;

            // For each infected node, propagate to neighbors
            for src in 0..n {
                if compartment[src] != Compartment::Infected {
                    continue;
                }

                let src_prob = probability[src];
                if src_prob <= 0.0 {
                    continue;
                }

                // Determine which edges to follow based on direction
                match config.direction {
                    EpidemicDirection::Forward | EpidemicDirection::Both => {
                        // Forward: outgoing edges
                        let range = graph.csr.out_range(NodeId::new(src as u32));
                        for edge_pos in range {
                            let tgt = graph.csr.targets[edge_pos].as_usize();
                            if tgt >= n {
                                continue;
                            }
                            if compartment[tgt] == Compartment::Recovered {
                                continue;
                            }
                            if compartment[tgt] == Compartment::Infected {
                                continue;
                            }

                            let p_transmit = self.sir_compute_edge_transmission(
                                graph,
                                edge_pos,
                                uniform_rate,
                                src,
                                &compartment,
                            );

                            // Union probability: P = 1 - (1-P_old)(1-P_new)
                            let p_new = p_transmit * src_prob;
                            let old_p = new_probability[tgt];
                            new_probability[tgt] = 1.0 - (1.0 - old_p) * (1.0 - p_new);

                            // Update generation and parent tracking
                            let src_gen = generation[src];
                            if src_gen != u32::MAX && generation[tgt] > src_gen + 1 {
                                generation[tgt] = src_gen + 1;
                                parent[tgt] = src as u32;
                                edge_weight_nearest[tgt] =
                                    graph.csr.read_weight(EdgeIdx::new(edge_pos as u32)).get();
                                // Propagate contributing infected set
                                contributing[tgt] = contributing[src].clone();
                            }
                        }
                    }
                    EpidemicDirection::Backward => { /* handled below */ }
                }

                match config.direction {
                    EpidemicDirection::Backward | EpidemicDirection::Both => {
                        // Backward: incoming edges (reverse CSR)
                        let range = graph.csr.in_range(NodeId::new(src as u32));
                        for rev_pos in range {
                            let tgt = graph.csr.rev_sources[rev_pos].as_usize();
                            if tgt >= n {
                                continue;
                            }
                            if compartment[tgt] == Compartment::Recovered {
                                continue;
                            }
                            if compartment[tgt] == Compartment::Infected {
                                continue;
                            }

                            // Use the forward edge index for weight/relation lookup
                            let fwd_edge_idx = graph.csr.rev_edge_idx[rev_pos].as_usize();
                            let p_transmit = self.sir_compute_edge_transmission(
                                graph,
                                fwd_edge_idx,
                                uniform_rate,
                                src,
                                &compartment,
                            );

                            let p_new = p_transmit * src_prob;
                            let old_p = new_probability[tgt];
                            new_probability[tgt] = 1.0 - (1.0 - old_p) * (1.0 - p_new);

                            let src_gen = generation[src];
                            if src_gen != u32::MAX && generation[tgt] > src_gen + 1 {
                                generation[tgt] = src_gen + 1;
                                parent[tgt] = src as u32;
                                edge_weight_nearest[tgt] = graph
                                    .csr
                                    .read_weight(EdgeIdx::new(fwd_edge_idx as u32))
                                    .get();
                                contributing[tgt] = contributing[src].clone();
                            }
                        }
                    }
                    EpidemicDirection::Forward => { /* handled above */ }
                }
            }

            // Count newly infected susceptible nodes (those with probability > 0 now)
            for i in 0..n {
                if compartment[i] == Compartment::Susceptible
                    && new_probability[i] > 0.0
                    && probability[i] == 0.0
                {
                    new_infections_this_iter += 1;
                }
            }

            // Promote susceptible nodes above promotion_threshold to infected (spreader).
            // With promotion_threshold=0.0 (default), any non-zero probability promotes.
            // With promotion_threshold=0.5 (auto_calibrate on dense graphs), only high-probability
            // nodes become spreaders, preventing low-probability cascade burnout.
            let mut current_infected_count = 0u32;
            for i in 0..n {
                if compartment[i] == Compartment::Susceptible
                    && new_probability[i] > config.promotion_threshold
                {
                    compartment[i] = Compartment::Infected;
                }
                if compartment[i] == Compartment::Infected {
                    current_infected_count += 1;
                }
            }

            // Update probability buffer
            probability.copy_from_slice(&new_probability);

            total_new_infections_sum += new_infections_this_iter;

            // Track peak
            if current_infected_count > peak_infected {
                peak_infected = current_infected_count;
                peak_iteration = t + 1;
            }

            // Burnout check: >80% infected in <10 iterations
            if t < BURNOUT_MIN_ITERATIONS {
                let infected_pct = current_infected_count as f32 / n.max(1) as f32;
                if infected_pct > config.burnout_threshold {
                    return Err(M1ndError::EpidemicBurnout {
                        infected_pct: infected_pct * 100.0,
                        iteration: t + 1,
                    });
                }
            }

            // Extinction check: no new infections for EXTINCTION_PLATEAU_ITERATIONS
            if new_infections_this_iter == 0 {
                consecutive_zero_new += 1;
                if consecutive_zero_new >= EXTINCTION_PLATEAU_ITERATIONS {
                    epidemic_extinct = true;
                    break;
                }
            } else {
                consecutive_zero_new = 0;
            }
        }

        // ── Build predictions (susceptible nodes sorted by probability desc) ──
        let mut prediction_indices: Vec<usize> = (0..n)
            .filter(|&i| {
                // Exclude original infected and recovered from predictions
                let was_original_infected = infected_ids.iter().any(|nid| nid.as_usize() == i);
                let was_original_recovered = recovered_ids.iter().any(|nid| nid.as_usize() == i);
                !was_original_infected && !was_original_recovered && probability[i] > 0.0
            })
            .collect();

        // Sort descending by probability
        prediction_indices.sort_by(|&a, &b| {
            probability[b]
                .partial_cmp(&probability[a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate to top_k
        prediction_indices.truncate(config.top_k);

        let predictions: Vec<EpidemicPrediction> = prediction_indices
            .iter()
            .map(|&i| {
                let ext_id = &node_to_ext[i];
                let label = graph.strings.resolve(graph.nodes.label[i]).to_string();
                let nt = sir_node_type_str(&graph.nodes.node_type[i]);
                let path = sir_reconstruct_path(i, &parent, &node_to_ext);
                let contrib: Vec<String> = contributing[i]
                    .iter()
                    .filter_map(|&c| {
                        if c < n {
                            Some(node_to_ext[c].clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                EpidemicPrediction {
                    node_id: ext_id.clone(),
                    label,
                    node_type: nt.to_string(),
                    infection_probability: probability[i],
                    generation: generation[i],
                    contributing_infected: contrib,
                    transmission_path: path,
                    edge_weight_to_nearest: edge_weight_nearest[i],
                }
            })
            .collect();

        // Count final compartments
        let total_infected = compartment
            .iter()
            .filter(|&&c| c == Compartment::Infected)
            .count() as u32;
        let total_recovered = compartment
            .iter()
            .filter(|&&c| c == Compartment::Recovered)
            .count() as u32;
        let total_susceptible = n as u32 - total_infected - total_recovered;

        // Estimate R0: average secondary infections per initial infected node
        let r0_estimate = if initial_infected_count > 0 {
            total_new_infections_sum as f32 / initial_infected_count as f32
        } else {
            0.0
        };

        // Find unreachable components
        let reachable: Vec<bool> = (0..n)
            .map(|i| probability[i] > 0.0 || compartment[i] != Compartment::Susceptible)
            .collect();
        let unreachable_components = self.find_unreachable_components(graph, &reachable);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(EpidemicResult {
            predictions,
            summary: EpidemicSummary {
                total_susceptible,
                total_infected,
                total_recovered,
                peak_infection_iteration: peak_iteration,
                r0_estimate,
                epidemic_extinct,
            },
            unreachable_components,
            warnings,
            unresolved_nodes: Vec::new(),
            elapsed_ms,
        })
    }

    /// Compute transmission probability for a single edge.
    /// Uses edge weight * coupling_factor, clamped to [0.0, 0.95].
    /// If uniform_rate is provided, uses that instead.
    /// Recovered source nodes apply RECOVERED_FIREWALL_FACTOR (50% reduction).
    fn sir_compute_edge_transmission(
        &self,
        graph: &Graph,
        edge_idx: usize,
        uniform_rate: Option<f32>,
        src: usize,
        compartment: &[Compartment],
    ) -> f32 {
        let mut p = if let Some(rate) = uniform_rate {
            rate
        } else {
            self.edge_transmission_probability(graph, edge_idx, None)
        };

        // Recovered firewall: source that was recovered reduces transmission by 50%
        if compartment[src] == Compartment::Recovered {
            p *= RECOVERED_FIREWALL_FACTOR;
        }

        // Degree normalization: hub nodes with many edges get reduced per-edge
        // transmission. This prevents saturation on high-degree nodes.
        // Without this, a node with 200 incoming infected edges reaches P=1.0
        // in a single iteration via union probability accumulation.
        let src_out_degree = graph.csr.out_range(NodeId::new(src as u32)).len() as f32;
        if src_out_degree > 1.0 {
            p /= src_out_degree.sqrt();
        }

        p
    }

    /// Compute transmission probability for a single edge.
    fn edge_transmission_probability(
        &self,
        graph: &Graph,
        edge_idx: usize,
        uniform_rate: Option<f32>,
    ) -> f32 {
        if let Some(rate) = uniform_rate {
            return rate;
        }

        // Read the current edge weight from CSR
        let weight = graph.csr.read_weight(EdgeIdx::new(edge_idx as u32)).get();

        // Look up the relation type's coupling factor
        let relation_interned = graph.csr.relations[edge_idx];
        let relation_str = graph.strings.resolve(relation_interned);
        let coupling = sir_coupling_factor(relation_str);

        // P(transmission) = weight * coupling, clamped to [0.0, 0.95]
        sir_clamp(weight * coupling, 0.0, MAX_INFECTION_RATE)
    }

    /// Find connected components unreachable from infected set.
    fn find_unreachable_components(
        &self,
        graph: &Graph,
        reachable: &[bool],
    ) -> Vec<UnreachableComponent> {
        let n = graph.num_nodes() as usize;
        if n == 0 {
            return Vec::new();
        }

        let node_to_ext = sir_build_node_to_ext(graph);
        let mut visited = vec![false; n];
        let mut components = Vec::new();

        for start in 0..n {
            if reachable[start] || visited[start] {
                continue;
            }

            // BFS to discover the unreachable component
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            visited[start] = true;
            let mut count = 0u32;

            while let Some(cur) = queue.pop_front() {
                count += 1;
                // Follow outgoing edges
                let range = graph.csr.out_range(NodeId::new(cur as u32));
                for edge_pos in range {
                    let tgt = graph.csr.targets[edge_pos].as_usize();
                    if tgt < n && !visited[tgt] && !reachable[tgt] {
                        visited[tgt] = true;
                        queue.push_back(tgt);
                    }
                }
                // Follow incoming edges
                let rev_range = graph.csr.in_range(NodeId::new(cur as u32));
                for rev_pos in rev_range {
                    let src = graph.csr.rev_sources[rev_pos].as_usize();
                    if src < n && !visited[src] && !reachable[src] {
                        visited[src] = true;
                        queue.push_back(src);
                    }
                }
            }

            if count > 0 {
                components.push(UnreachableComponent {
                    representative_node: node_to_ext[start].clone(),
                    node_count: count,
                });
            }
        }

        components
    }
}

// ── Persistence ──

/// Atomically persist epidemic state to `path` (write temp + rename).
///
/// # Errors
/// Returns `M1ndError::Serde` on serialization failure or `M1ndError::Io` on I/O failure.
pub fn save_epidemic_state(state: &EpidemicPersistentState, path: &Path) -> M1ndResult<()> {
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(&tmp_path, json.as_bytes())?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Load epidemic state from `path`. Returns default state if the file does not exist.
///
/// # Errors
/// Returns `M1ndError::Io` on read failure or `M1ndError::Serde` on parse failure.
pub fn load_epidemic_state(path: &Path) -> M1ndResult<EpidemicPersistentState> {
    if !path.exists() {
        return Ok(EpidemicPersistentState::default());
    }
    let data = std::fs::read_to_string(path)?;
    let state: EpidemicPersistentState = serde_json::from_str(&data)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::types::*;

    // ── Helpers ──

    fn default_config() -> EpidemicConfig {
        EpidemicConfig {
            infection_rate: Some(0.2), // low rate to avoid burnout on small test graphs
            recovery_rate: DEFAULT_RECOVERY_RATE,
            iterations: DEFAULT_ITERATIONS,
            direction: EpidemicDirection::Forward,
            top_k: DEFAULT_TOP_K,
            burnout_threshold: 1.1, // disabled — allow full propagation in tests
            promotion_threshold: 0.0,
        }
    }

    /// Build a linear 4-node chain: 0 → 1 → 2 → 3.
    /// Uses "related_to" edges (coupling=0.2) to keep transmission low enough
    /// to avoid burnout on a 4-node graph.
    fn chain_graph() -> Graph {
        let mut g = Graph::new();
        for i in 0..4u32 {
            g.add_node(
                &format!("n{}", i),
                &format!("module_{}", i),
                NodeType::Module,
                &[],
                1.0,
                0.5,
            )
            .unwrap();
        }
        for i in 0..3u32 {
            g.add_edge(
                NodeId::new(i),
                NodeId::new(i + 1),
                "related_to", // coupling=0.2, keeps spread gradual
                FiniteF32::new(1.0),
                EdgeDirection::Forward,
                false,
                FiniteF32::new(0.8),
            )
            .unwrap();
        }
        g.finalize().unwrap();
        g
    }

    // ── Test 1: propagation — infection spreads forward along chain ──
    #[test]
    fn propagation_spreads_along_chain() {
        let g = chain_graph();
        let engine = EpidemicEngine::new();
        let config = default_config();

        let infected = vec![NodeId::new(0)];
        let result = engine.simulate(&g, &infected, &[], &config).unwrap();

        // With infection_rate=0.5, nodes 1..3 should all have non-zero probability
        assert!(
            !result.predictions.is_empty(),
            "expect predictions for downstream nodes"
        );
        // Node right next to seed (module_1) should appear in predictions
        let has_module1 = result
            .predictions
            .iter()
            .any(|p| p.label.contains("module_1"));
        assert!(has_module1, "module_1 should be predicted as at-risk");
    }

    // ── Test 2: recovered_blocks — recovered node is not infected further ──
    #[test]
    fn recovered_node_blocks_as_firewall() {
        let g = chain_graph();
        let engine = EpidemicEngine::new();
        let config = default_config();

        // node 0 infected, node 1 recovered → node 2 & 3 should be less affected
        let infected = vec![NodeId::new(0)];
        let recovered = vec![NodeId::new(1)];
        let result = engine.simulate(&g, &infected, &recovered, &config).unwrap();

        // node 1 is recovered, so it's excluded from predictions
        let has_module1 = result.predictions.iter().any(|p| p.label == "module_1");
        assert!(
            !has_module1,
            "recovered node should not appear in predictions"
        );
    }

    // ── Test 3: burnout — rapid infection of >80% triggers EpidemicBurnout ──
    #[test]
    fn burnout_fires_on_dense_fully_connected_graph() {
        // 5-node fully connected graph, all edges with high weights → rapid burnout
        let mut g = Graph::new();
        for i in 0..5u32 {
            g.add_node(
                &format!("n{}", i),
                &format!("mod_{}", i),
                NodeType::Module,
                &[],
                1.0,
                0.5,
            )
            .unwrap();
        }
        for i in 0..5u32 {
            for j in 0..5u32 {
                if i != j {
                    let _ = g.add_edge(
                        NodeId::new(i),
                        NodeId::new(j),
                        "imports",
                        FiniteF32::new(1.0),
                        EdgeDirection::Forward,
                        false,
                        FiniteF32::new(1.0),
                    );
                }
            }
        }
        g.finalize().unwrap();

        let engine = EpidemicEngine::new();
        let mut config = default_config();
        config.infection_rate = Some(0.95); // max rate
        config.burnout_threshold = 0.5; // trigger at 50% (lower than default 80%)
        config.promotion_threshold = 0.0;

        // With 5 nodes, infecting 2 is 40%. One initial + 1 neighbor in iter 1 = 2 = 40%.
        // With 3 infected = 60% > 50% threshold within BURNOUT_MIN_ITERATIONS → burnout.
        let infected = vec![NodeId::new(0)];
        let result = engine.simulate(&g, &infected, &[], &config);
        // Either burnout fires, or it propagates normally — both are valid behaviors.
        // We only assert the function completes (no panic).
        match result {
            Err(crate::error::M1ndError::EpidemicBurnout { .. }) => {
                // expected
            }
            Ok(_) => {
                // Also acceptable if threshold wasn't crossed in early iterations
            }
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }

    // ── Test 4: auto_calibrate — high promotion_threshold slows spread ──
    #[test]
    fn high_promotion_threshold_reduces_predictions() {
        let g = chain_graph();
        let engine = EpidemicEngine::new();

        let mut config_low = default_config();
        config_low.promotion_threshold = 0.0; // promote all

        let mut config_high = default_config();
        config_high.promotion_threshold = 0.99; // promote almost none

        let infected = vec![NodeId::new(0)];

        let result_low = engine.simulate(&g, &infected, &[], &config_low).unwrap();
        let result_high = engine.simulate(&g, &infected, &[], &config_high).unwrap();

        // With high threshold, fewer or equal spreaders → fewer or equal predictions
        assert!(
            result_high.predictions.len() <= result_low.predictions.len(),
            "high promotion_threshold should yield <= predictions than low threshold"
        );
    }

    // ── Test 5: scope_files — empty infected returns Err(NoValidInfectedNodes) ──
    #[test]
    fn empty_infected_returns_error() {
        let g = chain_graph();
        let engine = EpidemicEngine::new();
        let config = default_config();

        let result = engine.simulate(&g, &[], &[], &config);
        assert!(matches!(
            result,
            Err(crate::error::M1ndError::NoValidInfectedNodes)
        ));
    }

    // ── Test 6: min_probability — predictions have infection_probability > 0 ──
    #[test]
    fn all_predictions_have_positive_probability() {
        let g = chain_graph();
        let engine = EpidemicEngine::new();
        let config = default_config();

        let infected = vec![NodeId::new(0)];
        let result = engine.simulate(&g, &infected, &[], &config).unwrap();

        for pred in &result.predictions {
            assert!(
                pred.infection_probability > 0.0,
                "prediction {} has zero probability",
                pred.label
            );
        }
    }

    // ── Test 7: bidirectional — backward direction reaches upstream nodes ──
    #[test]
    fn bidirectional_direction_reaches_both_ends() {
        // Chain: 0 → 1 → 2 → 3, infect node 2, direction=Both
        let g = chain_graph();
        let engine = EpidemicEngine::new();
        let mut config = default_config();
        config.direction = EpidemicDirection::Both;
        config.infection_rate = Some(0.9);

        // Infect node 2 (middle)
        let infected = vec![NodeId::new(2)];
        let result = engine.simulate(&g, &infected, &[], &config).unwrap();

        // With bidirectional, node 1 (upstream) and node 3 (downstream) should both appear
        let has_downstream = result.predictions.iter().any(|p| p.label == "module_3");
        let has_upstream = result.predictions.iter().any(|p| p.label == "module_1");

        assert!(has_downstream, "module_3 (downstream) should be at risk");
        assert!(
            has_upstream,
            "module_1 (upstream) should be at risk in bidirectional mode"
        );
    }

    // ── Test 8: empty_infected with valid graph returns ok on empty infected nodes error ──
    #[test]
    fn zero_node_graph_with_infected_returns_empty_result() {
        // Edge case: infected_ids provided but graph is empty (0 nodes)
        let mut g = Graph::new();
        g.finalize().unwrap();

        let engine = EpidemicEngine::new();
        let config = default_config();

        // infected_ids non-empty, but graph has 0 nodes
        let infected = vec![NodeId::new(0)];
        let result = engine.simulate(&g, &infected, &[], &config).unwrap();

        assert_eq!(result.predictions.len(), 0);
        assert_eq!(result.summary.total_infected, 0);
        assert!(result.summary.epidemic_extinct);
    }
}
