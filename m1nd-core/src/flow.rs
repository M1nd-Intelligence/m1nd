// === m1nd-core/src/flow.rs ===
//
// Concurrent flow simulation for race condition detection.
// Particles traverse the graph; collisions on shared mutable state
// without synchronization are flagged as turbulence points.

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Instant;

// ── Constants ──

/// Default maximum traversal depth per particle.
pub const DEFAULT_MAX_DEPTH: u8 = 15;
/// Default number of particles spawned per entry point.
pub const DEFAULT_NUM_PARTICLES: u32 = 2;
/// Hard cap on total particles spawned across all entry points.
pub const MAX_PARTICLES: u32 = 100;
/// Hard cap on active in-flight particles per entry point to prevent memory blowup.
pub const MAX_ACTIVE_PARTICLES: usize = 10_000;
/// Default minimum edge weight — edges below this are skipped as noise.
pub const DEFAULT_MIN_EDGE_WEIGHT: f32 = 0.1;
/// Default turbulence score threshold — points below this are suppressed.
pub const DEFAULT_TURBULENCE_THRESHOLD: f32 = 0.5;

/// Default lock/synchronization patterns for valve detection (substring match).
pub const DEFAULT_LOCK_PATTERNS: &[&str] = &[
    r"asyncio\.Lock",
    r"threading\.Lock",
    r"Mutex",
    r"RwLock",
    r"Semaphore",
    r"asyncio\.Semaphore",
    r"Lock\(\)",
    r"\.acquire\(",
    r"\.lock\(",
];

/// Default read-only access patterns — nodes matching these are exempt from turbulence scoring.
pub const DEFAULT_READ_ONLY_PATTERNS: &[&str] = &[
    r"get_", r"read_", r"fetch_", r"list_", r"is_", r"has_", r"check_", r"count_", r"len\(",
    r"\bGET\b", r"select ", r"SELECT ",
];

/// Entry point auto-discovery patterns (matched case-insensitively against node labels).
const ENTRY_POINT_PATTERNS: &[&str] = &[
    "handle_",
    "route_",
    "api_",
    "endpoint_",
    "on_",
    "cmd_",
    "tick_",
    "daemon_",
];

// ── Core Types ──

/// Severity of a detected turbulence point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum TurbulenceSeverity {
    /// Score >= 0.8, >=3 unsynchronized particles, no lock protection.
    Critical,
    /// Score >= 0.6.
    High,
    /// Score >= 0.3.
    Medium,
    /// Score above threshold but below 0.3.
    Low,
}

/// A node reached by particles from more than one concurrent entry point.
///
/// Represents a potential race condition on shared mutable state.
#[derive(Clone, Debug, Serialize)]
pub struct TurbulencePoint {
    /// Graph node ID of the turbulence point.
    pub node: NodeId,
    /// Human-readable label of the node.
    pub node_label: String,
    /// Number of distinct unsynchronized concurrent paths reaching this node.
    pub particle_count: u32,
    /// Whether this node itself is a lock/synchronization point.
    pub has_lock: bool,
    /// Whether this node was classified as read-only access.
    pub is_read_only: bool,
    /// Composite turbulence score in [0.0, 1.0].
    pub turbulence_score: f32,
    /// Severity classification.
    pub severity: TurbulenceSeverity,
    /// Pairs of entry point labels that both reached this node.
    pub entry_pairs: Vec<(String, String)>,
    /// Label of the nearest upstream lock in any particle's path, if any.
    pub nearest_upstream_lock: Option<String>,
    /// Paths taken by each particle to reach this node (if `include_paths`).
    pub paths: Vec<Vec<String>>,
}

/// A lock/synchronization node that serializes concurrent particle paths.
#[derive(Clone, Debug, Serialize)]
pub struct ValvePoint {
    /// Graph node ID of the valve.
    pub node: NodeId,
    /// Human-readable label of the valve node.
    pub node_label: String,
    /// Matched lock pattern string (e.g. "heuristic:mutex").
    pub lock_type: String,
    /// Number of particle arrivals serialized by this valve.
    pub particles_serialized: u32,
    /// BFS count of nodes downstream of the valve (depth-limited).
    pub downstream_protected: u32,
}

/// A node that was skipped or score-reduced because it is protected by an
/// advisory lock (`lock.create`). Reported in `FlowSimulationResult` so
/// callers can see which turbulence was suppressed and why.
#[derive(Clone, Debug, Serialize)]
pub struct ProtectedNode {
    /// Human-readable label of the protected node.
    pub node_label: String,
    /// Advisory lock ID(s) covering this node.
    pub lock_ids: Vec<String>,
    /// Original (pre-reduction) turbulence score, if the node was a
    /// turbulence candidate. `None` when the node was visited but never
    /// reached from multiple origins.
    pub original_score: Option<f32>,
    /// Whether the node was completely suppressed (dropped below threshold
    /// after the protection factor was applied).
    pub suppressed: bool,
}

/// Aggregate statistics for a flow simulation run.
#[derive(Clone, Debug, Serialize)]
pub struct FlowSummary {
    /// Number of distinct entry points used.
    pub total_entry_points: u32,
    /// Total particles spawned across all entry points.
    pub total_particles: u32,
    /// Total distinct graph nodes visited.
    pub total_nodes_visited: u32,
    /// Number of turbulence points above threshold.
    pub turbulence_count: u32,
    /// Number of valve (lock) points detected.
    pub valve_count: u32,
    /// Number of nodes protected by advisory locks.
    pub advisory_lock_protected_count: u32,
    /// Fraction of graph nodes visited in [0.0, 1.0].
    pub coverage_pct: f32,
    /// Wall-clock time in milliseconds.
    pub elapsed_ms: f64,
}

/// Complete result of a flow simulation.
#[derive(Clone, Debug, Serialize)]
pub struct FlowSimulationResult {
    /// Turbulence points sorted by score descending.
    pub turbulence_points: Vec<TurbulencePoint>,
    /// Valve points sorted by node ID ascending.
    pub valve_points: Vec<ValvePoint>,
    /// Nodes protected by advisory locks — turbulence suppressed or reduced.
    pub protected_nodes: Vec<ProtectedNode>,
    /// Aggregate statistics.
    pub summary: FlowSummary,
}

/// Configuration for the flow simulation engine.
#[derive(Clone, Debug)]
pub struct FlowConfig {
    /// Patterns used to identify lock/synchronization nodes.
    pub lock_patterns: Vec<String>,
    /// Patterns used to identify read-only access nodes.
    pub read_only_patterns: Vec<String>,
    /// Maximum BFS depth per particle.
    pub max_depth: u8,
    /// Minimum turbulence score to include in results.
    pub turbulence_threshold: f32,
    /// Whether to record full path traces per particle.
    pub include_paths: bool,
    /// Maximum particles per entry point (capped at `MAX_PARTICLES`).
    pub max_particles: u32,
    /// Minimum edge weight to traverse — edges below this are skipped.
    pub min_edge_weight: f32,
    /// Global step budget across all particles (default 50000).
    pub max_total_steps: usize,
    /// Optional substring filter to limit particle scope to matching node labels.
    pub scope_filter: Option<String>,
    /// Node labels protected by advisory locks (injected by MCP layer from
    /// active `lock.create` state). Mapping: node_label -> Vec<lock_id>.
    /// Protected nodes get a 0.15 turbulence factor (85% reduction).
    pub advisory_lock_protected_nodes: BTreeMap<String, Vec<String>>,
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            lock_patterns: Vec::new(),
            read_only_patterns: Vec::new(),
            max_depth: DEFAULT_MAX_DEPTH,
            turbulence_threshold: DEFAULT_TURBULENCE_THRESHOLD,
            include_paths: true,
            max_particles: MAX_PARTICLES,
            min_edge_weight: DEFAULT_MIN_EDGE_WEIGHT,
            max_total_steps: 50_000,
            scope_filter: None,
            advisory_lock_protected_nodes: BTreeMap::new(),
        }
    }
}

impl FlowConfig {
    /// Create config with default lock and read-only pattern strings.
    pub fn with_defaults() -> Self {
        Self {
            lock_patterns: DEFAULT_LOCK_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            read_only_patterns: DEFAULT_READ_ONLY_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ..Default::default()
        }
    }

    /// Create config with user-provided pattern strings.
    pub fn with_patterns(lock_patterns: &[String], read_only_patterns: &[String]) -> Self {
        Self {
            lock_patterns: lock_patterns.to_vec(),
            read_only_patterns: read_only_patterns.to_vec(),
            ..Default::default()
        }
    }
}

// ── Internal particle state ──

/// A single particle flowing through the graph during simulation.
#[derive(Clone)]
struct Particle {
    /// Unique particle ID within this simulation.
    id: u32,
    /// Entry point where this particle was spawned.
    origin: NodeId,
    /// Path taken so far (ordered node sequence).
    path: Vec<NodeId>,
    /// Current position in the graph.
    position: NodeId,
    /// Current depth from entry point.
    depth: u8,
    /// If serialized by a valve, which node.
    serialized_by: Option<NodeId>,
    /// Visited nodes (cycle detection). Vec<bool> for O(1) lookup.
    visited: Vec<bool>,
}

/// Per-node arrival record: which particles from which origins arrived.
struct NodeAccumulator {
    /// Indexed by node id. Each entry: Vec of (origin NodeId, particle_id, serialized_by, path).
    arrivals: Vec<Vec<ParticleArrival>>,
}

#[derive(Clone)]
struct ParticleArrival {
    origin: NodeId,
    particle_id: u32,
    serialized_by: Option<NodeId>,
    path: Vec<NodeId>,
}

impl NodeAccumulator {
    fn new(num_nodes: usize) -> Self {
        Self {
            arrivals: vec![Vec::new(); num_nodes],
        }
    }

    #[inline]
    fn record(&mut self, node: NodeId, arrival: ParticleArrival) {
        let idx = node.as_usize();
        if idx < self.arrivals.len() {
            self.arrivals[idx].push(arrival);
        }
    }

    /// Returns nodes with arrivals from >1 distinct origin.
    fn flow_turbulent_nodes(&self) -> Vec<(NodeId, &Vec<ParticleArrival>)> {
        self.arrivals
            .iter()
            .enumerate()
            .filter_map(|(i, arrivals)| {
                if arrivals.is_empty() {
                    return None;
                }
                let mut origins = BTreeSet::new();
                for a in arrivals {
                    origins.insert(a.origin.0);
                }
                if origins.len() > 1 {
                    Some((NodeId::new(i as u32), arrivals))
                } else {
                    None
                }
            })
            .collect()
    }
}

// ── Valve tracking ──

struct ValveTracker {
    /// node_id -> (matched pattern string, particles serialized count)
    valves: BTreeMap<u32, (String, u32)>,
}

impl ValveTracker {
    fn new() -> Self {
        Self {
            valves: BTreeMap::new(),
        }
    }

    fn record_serialization(&mut self, node: NodeId, lock_type: &str) {
        let entry = self
            .valves
            .entry(node.0)
            .or_insert_with(|| (lock_type.to_string(), 0));
        entry.1 += 1;
    }
}

// ── Helper functions ──

/// Check if a node label or excerpt matches any pattern (case-insensitive substring match).
/// Patterns use simple substring matching -- regex metacharacters are stripped for matching.
fn flow_matches_any_pattern(text: &str, patterns: &[String]) -> Option<String> {
    let text_lower = text.to_lowercase();
    for pat in patterns {
        // Strip common regex metacharacters for substring matching since we don't have regex crate.
        let clean = flow_clean_pattern(pat);
        if text_lower.contains(&clean) {
            return Some(pat.clone());
        }
    }
    None
}

/// Clean a pattern string for simple substring matching:
/// remove regex metacharacters like \, ^, $, etc. and lowercase.
fn flow_clean_pattern(pat: &str) -> String {
    pat.to_lowercase()
        .replace(['\\', '^', '$'], "")
        .replace("\\b", "")
}

/// Resolve a node label to a string.
fn flow_node_label(graph: &Graph, node: NodeId) -> String {
    let idx = node.as_usize();
    if idx < graph.nodes.count as usize {
        graph.strings.resolve(graph.nodes.label[idx]).to_string()
    } else {
        format!("node_{}", node.0)
    }
}

/// Get the node label + excerpt combined text for pattern matching.
fn flow_node_text(graph: &Graph, node: NodeId) -> String {
    let idx = node.as_usize();
    if idx >= graph.nodes.count as usize {
        return String::new();
    }
    let label = graph.strings.resolve(graph.nodes.label[idx]);
    let excerpt = graph.nodes.provenance[idx]
        .excerpt
        .map(|e| graph.strings.resolve(e))
        .unwrap_or("");
    format!("{} {}", label, excerpt)
}

/// Check if a node is a valve (lock/synchronization point).
/// Returns the matched pattern string if it is.
fn flow_is_valve(graph: &Graph, node: NodeId, config: &FlowConfig) -> Option<String> {
    let text = flow_node_text(graph, node);
    if let Some(pat) = flow_matches_any_pattern(&text, &config.lock_patterns) {
        return Some(pat);
    }

    // Also check: does node have an outgoing edge to a node matching lock patterns?
    let idx = node.as_usize();
    if idx < graph.nodes.count as usize {
        let range = graph.csr.out_range(node);
        for j in range {
            let target = graph.csr.targets[j];
            let target_text = flow_node_text(graph, target);
            if let Some(pat) = flow_matches_any_pattern(&target_text, &config.lock_patterns) {
                return Some(pat);
            }
        }
    }

    // Heuristic fallback: case-insensitive keyword check
    let text_lower = text.to_lowercase();
    let heuristic_keywords = [
        "lock",
        "mutex",
        "guard",
        "semaphore",
        "synchronize",
        "serialize",
    ];
    for kw in &heuristic_keywords {
        if text_lower.contains(kw) {
            return Some(format!("heuristic:{}", kw));
        }
    }

    None
}

/// Check if a node represents read-only access.
fn flow_is_read_only(graph: &Graph, node: NodeId, config: &FlowConfig) -> bool {
    let text = flow_node_text(graph, node);
    if flow_matches_any_pattern(&text, &config.read_only_patterns).is_none() {
        return false;
    }

    // Override: if downstream nodes look like writers, not read-only.
    let idx = node.as_usize();
    if idx < graph.nodes.count as usize {
        let range = graph.csr.out_range(node);
        for j in range {
            let target = graph.csr.targets[j];
            let target_text = flow_node_text(graph, target).to_lowercase();
            // Check for write-like downstream nodes
            if target_text.contains("set_")
                || target_text.contains("write_")
                || target_text.contains("update_")
                || target_text.contains("delete_")
                || target_text.contains("insert_")
                || target_text.contains("put_")
                || target_text.contains("remove_")
                || target_text.contains("mutate")
            {
                return false;
            }
        }
    }

    true
}

/// Count downstream reachable nodes from a given node via BFS.
fn flow_count_downstream(graph: &Graph, node: NodeId, max_depth: u8) -> u32 {
    let n = graph.num_nodes() as usize;
    let mut visited = vec![false; n];
    let mut queue = VecDeque::new();
    let idx = node.as_usize();
    if idx >= n {
        return 0;
    }
    visited[idx] = true;
    queue.push_back((node, 0u8));
    let mut count = 0u32;

    while let Some((current, depth)) = queue.pop_front() {
        if depth >= max_depth {
            continue;
        }
        let range = graph.csr.out_range(current);
        for j in range {
            let target = graph.csr.targets[j];
            let tidx = target.as_usize();
            if tidx < n && !visited[tidx] {
                visited[tidx] = true;
                count += 1;
                queue.push_back((target, depth + 1));
            }
        }
    }

    count
}

/// Find nearest upstream lock in a particle's path.
fn flow_find_nearest_upstream_lock(
    graph: &Graph,
    path: &[NodeId],
    config: &FlowConfig,
) -> Option<NodeId> {
    path.iter()
        .rev()
        .find(|&&node| flow_is_valve(graph, node, config).is_some())
        .copied()
}

/// Compute in-degree for a node (used as centrality proxy per D-12).
fn flow_in_degree(graph: &Graph, node: NodeId) -> u32 {
    let idx = node.as_usize();
    if idx >= graph.nodes.count as usize {
        return 0;
    }
    let range = graph.csr.in_range(node);
    (range.end - range.start) as u32
}

/// Max in-degree across all nodes (for normalization).
fn flow_max_in_degree(graph: &Graph) -> u32 {
    let n = graph.num_nodes();
    let mut max_deg = 1u32;
    for i in 0..n {
        let node = NodeId::new(i);
        let deg = flow_in_degree(graph, node);
        if deg > max_deg {
            max_deg = deg;
        }
    }
    max_deg
}

// ── Engine ──

/// Concurrent flow simulation engine for race condition detection.
///
/// Spawns particles at entry points and propagates them through the graph.
/// Nodes reached by particles from more than one distinct entry point are
/// flagged as turbulence points — potential race conditions on shared mutable state.
pub struct FlowEngine;

impl Default for FlowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowEngine {
    /// Create a new `FlowEngine`.
    pub fn new() -> Self {
        Self
    }

    /// Run flow simulation. Main entry point for the MCP tool.
    ///
    /// # Parameters
    /// - `graph`: finalized graph to simulate on
    /// - `entry_nodes`: starting positions for particles (at least one required)
    /// - `num_particles`: particles spawned per entry point (capped by `config.max_particles`)
    /// - `config`: simulation parameters
    ///
    /// # Errors
    /// Returns `M1ndError::NoEntryPoints` if `entry_nodes` is empty or graph has no nodes.
    pub fn simulate(
        &self,
        graph: &Graph,
        entry_nodes: &[NodeId],
        num_particles: u32,
        config: &FlowConfig,
    ) -> M1ndResult<FlowSimulationResult> {
        let start = Instant::now();
        let n = graph.num_nodes() as usize;

        // EC-8: empty graph or no entry points
        if n == 0 || entry_nodes.is_empty() {
            return Err(M1ndError::NoEntryPoints);
        }

        let num_particles = num_particles.min(config.max_particles);
        let max_depth = config.max_depth;

        // Auto-scale step budget for dense graphs to prevent latency explosion.
        // Dense graph = high edges/nodes ratio. Scale budget down proportionally.
        let edges = graph.num_edges() as f64;
        let nodes = n as f64;
        let density = if nodes > 0.0 { edges / nodes } else { 1.0 };
        let budget_scale = if density > 10.0 {
            // Very dense: reduce budget to prevent >100ms queries
            (10.0 / density).max(0.1)
        } else {
            1.0
        };
        let effective_steps = ((config.max_total_steps as f64) * budget_scale) as usize;
        let max_total_steps = effective_steps.max(1000); // floor at 1000

        // Accumulator: track particle arrivals per node
        let mut accumulator = NodeAccumulator::new(n);
        // Valve tracker
        let mut valve_tracker = ValveTracker::new();
        // Global visited set for coverage tracking
        let mut global_visited = vec![false; n];
        // Total particle counter
        let mut total_particles_spawned = 0u32;

        // Pre-compute scope filter: which nodes are in scope
        let scope_allowed: Option<Vec<bool>> = config.scope_filter.as_ref().map(|filter| {
            let filter_lower = filter.to_lowercase();
            (0..n)
                .map(|i| {
                    let label = graph.strings.resolve(graph.nodes.label[i]);
                    label.to_lowercase().contains(&filter_lower)
                })
                .collect()
        });

        let mut global_steps: usize = 0;

        // 1. Spawn particles at entry points and propagate via BFS
        for &entry in entry_nodes {
            let entry_idx = entry.as_usize();
            if entry_idx >= n {
                continue;
            }

            for p_idx in 0..num_particles {
                total_particles_spawned += 1;
                let pid = total_particles_spawned;

                let mut visited = vec![false; n];
                visited[entry_idx] = true;
                global_visited[entry_idx] = true;

                // BFS queue: (particle state snapshot for each active front)
                let mut queue: VecDeque<Particle> = VecDeque::new();
                let initial = Particle {
                    id: pid,
                    origin: entry,
                    path: vec![entry],
                    position: entry,
                    depth: 0,
                    serialized_by: None,
                    visited,
                };

                // Record arrival at entry
                accumulator.record(
                    entry,
                    ParticleArrival {
                        origin: entry,
                        particle_id: pid,
                        serialized_by: None,
                        path: vec![entry],
                    },
                );

                queue.push_back(initial);

                let mut active_count = 1usize;

                while let Some(particle) = queue.pop_front() {
                    if particle.depth >= max_depth {
                        continue;
                    }

                    let pos = particle.position;
                    let pos_idx = pos.as_usize();
                    if pos_idx >= n {
                        continue;
                    }

                    // Check if this node is a valve
                    let mut serialized_by = particle.serialized_by;
                    if let Some(lock_type) = flow_is_valve(graph, pos, config) {
                        serialized_by = Some(pos);
                        valve_tracker.record_serialization(pos, &lock_type);
                    }

                    // Check global step budget
                    if global_steps >= max_total_steps {
                        break;
                    }

                    // Propagate along outgoing causal edges
                    let range = graph.csr.out_range(pos);
                    for j in range {
                        global_steps += 1;
                        if global_steps >= max_total_steps {
                            break;
                        }

                        // Skip inhibitory edges
                        if graph.csr.inhibitory[j] {
                            continue;
                        }

                        // Skip low-weight edges (noise)
                        let weight = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                        if weight < config.min_edge_weight {
                            continue;
                        }

                        let target = graph.csr.targets[j];
                        let tidx = target.as_usize();
                        if tidx >= n {
                            continue;
                        }

                        // Scope filter: skip nodes not matching the scope
                        if let Some(ref allowed) = scope_allowed {
                            if !allowed[tidx] {
                                continue;
                            }
                        }

                        // EC-1: cycle detection per particle
                        if particle.visited[tidx] {
                            // Record arrival but don't propagate further
                            accumulator.record(
                                target,
                                ParticleArrival {
                                    origin: entry,
                                    particle_id: pid,
                                    serialized_by,
                                    path: if config.include_paths {
                                        let mut p = particle.path.clone();
                                        p.push(target);
                                        p
                                    } else {
                                        Vec::new()
                                    },
                                },
                            );
                            global_visited[tidx] = true;
                            continue;
                        }

                        // FM-FLOW-011: cap active particles
                        if active_count >= MAX_ACTIVE_PARTICLES {
                            break;
                        }

                        let mut new_path = if config.include_paths {
                            let mut p = particle.path.clone();
                            p.push(target);
                            p
                        } else {
                            Vec::new()
                        };

                        // Record arrival
                        accumulator.record(
                            target,
                            ParticleArrival {
                                origin: entry,
                                particle_id: pid,
                                serialized_by,
                                path: new_path.clone(),
                            },
                        );

                        global_visited[tidx] = true;

                        // Create child particle
                        let mut child_visited = particle.visited.clone();
                        child_visited[tidx] = true;

                        let child = Particle {
                            id: pid,
                            origin: entry,
                            path: if config.include_paths {
                                new_path
                            } else {
                                Vec::new()
                            },
                            position: target,
                            depth: particle.depth + 1,
                            serialized_by,
                            visited: child_visited,
                        };

                        queue.push_back(child);
                        active_count += 1;
                    }

                    // FM-FLOW-011: if too many active, stop spawning
                    if active_count >= MAX_ACTIVE_PARTICLES {
                        break;
                    }
                    // Global step budget exceeded
                    if global_steps >= max_total_steps {
                        break;
                    }
                }
            }
        }

        // 2. Compute max in-degree for centrality normalization
        let max_in_deg = flow_max_in_degree(graph);

        // 3. Identify turbulence points (nodes with arrivals from >1 distinct origin)
        let turbulent = accumulator.flow_turbulent_nodes();
        let mut turbulence_points = Vec::new();
        let mut protected_nodes: Vec<ProtectedNode> = Vec::new();

        for (node, arrivals) in &turbulent {
            let node_label = flow_node_label(graph, *node);
            let has_lock = flow_is_valve(graph, *node, config).is_some();
            let is_read_only = flow_is_read_only(graph, *node, config);

            // Check advisory lock protection (from lock.create)
            let advisory_lock_ids = config
                .advisory_lock_protected_nodes
                .get(&node_label)
                .cloned()
                .unwrap_or_default();
            let is_advisory_protected = !advisory_lock_ids.is_empty();

            // Count unserialized particles from distinct origins
            let mut origins_unserialized: BTreeSet<u32> = BTreeSet::new();
            let mut all_origins: BTreeSet<u32> = BTreeSet::new();
            for a in *arrivals {
                all_origins.insert(a.origin.0);
                if a.serialized_by.is_none() {
                    origins_unserialized.insert(a.origin.0);
                }
            }

            // Effective particle count: only unserialized particles from distinct origins
            let particle_count = if origins_unserialized.len() > 1 {
                origins_unserialized.len() as u32
            } else if all_origins.len() > 1 {
                all_origins.len() as u32
            } else {
                continue;
            };

            // Turbulence scoring per PRD F5
            let base_score = (particle_count as f32) / (num_particles as f32).max(1.0);
            let base_score = base_score.min(1.0);

            // Find nearest upstream lock from any particle's path
            let nearest_lock = arrivals
                .iter()
                .find_map(|a| flow_find_nearest_upstream_lock(graph, &a.path, config));

            let lock_factor = if has_lock {
                0.0
            } else if nearest_lock.is_some() {
                0.3 // partially protected
            } else {
                1.0 // no protection
            };

            let read_factor = if is_read_only { 0.2 } else { 1.0 };

            // Advisory lock protection factor: nodes under active lock.create
            // get an 85% score reduction (factor 0.15). The region is being
            // actively worked on by an agent, so concurrent access there is
            // expected and not a real race condition.
            let advisory_factor = if is_advisory_protected { 0.15 } else { 1.0 };

            // Centrality factor: use in-degree as proxy (D-12)
            let in_deg = flow_in_degree(graph, *node);
            let centrality_normalized = (in_deg as f32) / (max_in_deg as f32).max(1.0);
            let centrality_factor = 0.5 + 0.5 * centrality_normalized;

            // EC-10: high fan-out utility nodes get score reduction
            let utility_factor = if is_read_only && centrality_normalized > 0.9 {
                0.1
            } else {
                1.0
            };

            // Score without advisory factor (for reporting original_score)
            let pre_advisory_score =
                base_score * lock_factor * read_factor * centrality_factor * utility_factor;

            let turbulence_score = pre_advisory_score * advisory_factor;

            // Record protected node regardless of threshold
            if is_advisory_protected {
                let suppressed = turbulence_score < config.turbulence_threshold
                    && pre_advisory_score >= config.turbulence_threshold;
                protected_nodes.push(ProtectedNode {
                    node_label: node_label.clone(),
                    lock_ids: advisory_lock_ids,
                    original_score: Some(pre_advisory_score),
                    suppressed,
                });
            }

            if turbulence_score < config.turbulence_threshold {
                continue;
            }

            // Severity mapping per PRD F5
            let severity = if turbulence_score >= 0.8
                && particle_count >= 3
                && nearest_lock.is_none()
                && !has_lock
                && !is_advisory_protected
            {
                TurbulenceSeverity::Critical
            } else if turbulence_score >= 0.6 {
                TurbulenceSeverity::High
            } else if turbulence_score >= 0.3 {
                TurbulenceSeverity::Medium
            } else {
                TurbulenceSeverity::Low
            };

            // Entry point pair attribution (F6)
            let origin_list: Vec<u32> = all_origins.iter().copied().collect();
            let mut entry_pairs = Vec::new();
            for i in 0..origin_list.len() {
                for j in (i + 1)..origin_list.len() {
                    let a_label = flow_node_label(graph, NodeId::new(origin_list[i]));
                    let b_label = flow_node_label(graph, NodeId::new(origin_list[j]));
                    entry_pairs.push((a_label, b_label));
                }
            }

            // Collect paths if requested
            let paths = if config.include_paths {
                arrivals
                    .iter()
                    .filter(|a| !a.path.is_empty())
                    .map(|a| a.path.iter().map(|n| flow_node_label(graph, *n)).collect())
                    .collect()
            } else {
                Vec::new()
            };

            let nearest_upstream_lock_label = nearest_lock.map(|n| flow_node_label(graph, n));

            turbulence_points.push(TurbulencePoint {
                node: *node,
                node_label,
                particle_count,
                has_lock,
                is_read_only,
                turbulence_score,
                severity,
                entry_pairs,
                nearest_upstream_lock: nearest_upstream_lock_label,
                paths,
            });
        }

        // Sort by turbulence score descending (deterministic: FM-FLOW-012)
        turbulence_points.sort_by(|a, b| {
            b.turbulence_score
                .partial_cmp(&a.turbulence_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.node.0.cmp(&b.node.0))
        });

        // 4. Build valve points (F7)
        let mut valve_points: Vec<ValvePoint> = valve_tracker
            .valves
            .iter()
            .map(|(&node_id, (lock_type, serialized))| {
                let node = NodeId::new(node_id);
                let downstream = flow_count_downstream(graph, node, config.max_depth);
                ValvePoint {
                    node,
                    node_label: flow_node_label(graph, node),
                    lock_type: lock_type.clone(),
                    particles_serialized: *serialized,
                    downstream_protected: downstream,
                }
            })
            .collect();

        // Sort valves deterministically by node id
        valve_points.sort_by_key(|v| v.node.0);

        // 5. Compute coverage
        let visited_count = global_visited.iter().filter(|&&v| v).count() as u32;
        let coverage_pct = if n > 0 {
            (visited_count as f32) / (n as f32)
        } else {
            0.0
        };

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Sort protected nodes deterministically by label
        protected_nodes.sort_by(|a, b| a.node_label.cmp(&b.node_label));
        let advisory_lock_protected_count = protected_nodes.len() as u32;

        Ok(FlowSimulationResult {
            summary: FlowSummary {
                total_entry_points: entry_nodes.len() as u32,
                total_particles: total_particles_spawned,
                total_nodes_visited: visited_count,
                turbulence_count: turbulence_points.len() as u32,
                valve_count: valve_points.len() as u32,
                advisory_lock_protected_count,
                coverage_pct,
                elapsed_ms,
            },
            turbulence_points,
            valve_points,
            protected_nodes,
        })
    }

    /// Auto-discover entry points from graph topology and naming patterns.
    ///
    /// Prefers `Function` nodes matching `ENTRY_POINT_PATTERNS`. Falls back to
    /// `File` nodes with entry-like names if no functions match.
    /// Results are sorted by PageRank (or in-degree) descending, capped at 100.
    ///
    /// # Parameters
    /// - `graph`: finalized graph to search
    /// - `max_entries`: maximum number of entry points to return (capped at 100)
    pub fn discover_entry_points(&self, graph: &Graph, max_entries: usize) -> Vec<NodeId> {
        let n = graph.num_nodes();
        if n == 0 {
            return Vec::new();
        }

        let mut candidates: Vec<(NodeId, f32)> = Vec::new();

        for i in 0..n {
            let node = NodeId::new(i);
            let idx = i as usize;

            // Filter by node_type: Function preferred, File as fallback
            let nt = graph.nodes.node_type[idx];
            let is_function = matches!(nt, NodeType::Function);

            let label = graph.strings.resolve(graph.nodes.label[idx]).to_lowercase();

            // Check against entry point patterns
            let matches_pattern = ENTRY_POINT_PATTERNS.iter().any(|p| label.contains(p));

            if matches_pattern && is_function {
                // Use pagerank if available, otherwise in-degree as priority
                let priority = if graph.pagerank_computed {
                    graph.nodes.pagerank[idx].get()
                } else {
                    let range = graph.csr.in_range(node);
                    (range.end - range.start) as f32
                };
                candidates.push((node, priority));
            }
        }

        // If no function-level entries found, try file-level with entry patterns
        if candidates.is_empty() {
            for i in 0..n {
                let node = NodeId::new(i);
                let idx = i as usize;
                let nt = graph.nodes.node_type[idx];
                if !matches!(nt, NodeType::File) {
                    continue;
                }
                let label = graph.strings.resolve(graph.nodes.label[idx]).to_lowercase();
                let matches_pattern = ENTRY_POINT_PATTERNS.iter().any(|p| label.contains(p));
                // Also match files with "main", "app", "server" in name
                let is_entry_file = label.contains("main")
                    || label.contains("app.")
                    || label.contains("server")
                    || label.contains("__init__");

                if matches_pattern || is_entry_file {
                    let priority = if graph.pagerank_computed {
                        graph.nodes.pagerank[idx].get()
                    } else {
                        let range = graph.csr.in_range(node);
                        (range.end - range.start) as f32
                    };
                    candidates.push((node, priority));
                }
            }
        }

        // Sort by priority descending (deterministic: break ties by node id)
        candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0 .0.cmp(&b.0 .0))
        });

        // Cap at max_entries (AC-6: cap at 100 per PRD F8)
        let cap = max_entries.min(100);
        candidates.truncate(cap);

        candidates.into_iter().map(|(node, _)| node).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::types::*;

    // ── Helpers ──

    /// Build a minimal 2-node finalized graph: A → B
    fn two_node_graph(label_a: &str, label_b: &str, relation: &str) -> Graph {
        let mut g = Graph::new();
        g.add_node("a", label_a, NodeType::Function, &[], 1.0, 0.5)
            .unwrap();
        g.add_node("b", label_b, NodeType::Function, &[], 0.8, 0.3)
            .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            relation,
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.finalize().unwrap();
        g
    }

    /// Build a 4-node graph with two entry points converging on a shared node:
    ///   entry1 → shared
    ///   entry2 → shared
    ///   shared → sink
    fn convergent_graph() -> Graph {
        let mut g = Graph::new();
        g.add_node("entry1", "handle_alpha", NodeType::Function, &[], 1.0, 0.5)
            .unwrap(); // 0
        g.add_node("entry2", "handle_beta", NodeType::Function, &[], 1.0, 0.5)
            .unwrap(); // 1
        g.add_node("shared", "shared_state", NodeType::Function, &[], 0.9, 0.4)
            .unwrap(); // 2
        g.add_node("sink", "output", NodeType::Function, &[], 0.5, 0.2)
            .unwrap(); // 3
        g.add_edge(
            NodeId::new(0),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(2),
            NodeId::new(3),
            "calls",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        g.finalize().unwrap();
        g
    }

    // ── Test 1: empty graph returns Err(NoEntryPoints) ──
    #[test]
    fn empty_graph_returns_no_entry_points_error() {
        let mut g = Graph::new();
        g.finalize().unwrap();
        let engine = FlowEngine::new();
        let config = FlowConfig::default();
        let result = engine.simulate(&g, &[], 2, &config);
        assert!(matches!(
            result,
            Err(crate::error::M1ndError::NoEntryPoints)
        ));
    }

    // ── Test 2: turbulence_detection — two entry points converging produce a turbulence point ──
    #[test]
    fn turbulence_detected_on_convergent_graph() {
        let g = convergent_graph();
        let engine = FlowEngine::new();
        let mut config = FlowConfig::default();
        config.turbulence_threshold = 0.0; // capture everything
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result = engine.simulate(&g, &entry_nodes, 1, &config).unwrap();
        // shared_state (node 2) receives particles from two distinct origins
        assert!(
            result.summary.turbulence_count > 0,
            "expected turbulence at convergence node, got 0"
        );
    }

    // ── Test 3: valve_detection — node with lock label becomes a valve ──
    #[test]
    fn valve_detected_on_lock_node() {
        let mut g = Graph::new();
        g.add_node("ep", "handle_req", NodeType::Function, &[], 1.0, 0.5)
            .unwrap();
        g.add_node("lk", "mutex_guard", NodeType::Function, &[], 0.9, 0.4)
            .unwrap();
        g.add_node("wr", "write_state", NodeType::Function, &[], 0.8, 0.3)
            .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let mut config = FlowConfig::default();
        // mutex is in the heuristic keyword list, so no need to add explicit patterns
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();

        let result = engine.simulate(&g, &[NodeId::new(0)], 1, &config).unwrap();
        // mutex_guard heuristic should register a valve
        assert!(
            result.summary.valve_count > 0,
            "expected a valve at mutex_guard node"
        );
    }

    // ── Test 4: max_depth — particles don't travel beyond max_depth ──
    #[test]
    fn max_depth_limits_propagation() {
        // Chain: 0 → 1 → 2 → 3 → 4 (5 nodes)
        let mut g = Graph::new();
        for i in 0..5u32 {
            g.add_node(
                &format!("n{}", i),
                &format!("node_{}", i),
                NodeType::Function,
                &[],
                1.0,
                0.5,
            )
            .unwrap();
        }
        for i in 0..4u32 {
            g.add_edge(
                NodeId::new(i),
                NodeId::new(i + 1),
                "calls",
                FiniteF32::new(0.9),
                EdgeDirection::Forward,
                false,
                FiniteF32::new(0.5),
            )
            .unwrap();
        }
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let mut config = FlowConfig::default();
        config.max_depth = 2; // only 2 hops from entry
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();

        let result = engine.simulate(&g, &[NodeId::new(0)], 1, &config).unwrap();
        // nodes 0, 1, 2 visited (depth 0, 1, 2); nodes 3, 4 NOT visited
        assert!(
            result.summary.total_nodes_visited <= 3,
            "expected at most 3 nodes visited with max_depth=2, got {}",
            result.summary.total_nodes_visited
        );
    }

    // ── Test 5: max_steps budget — simulation stops at budget ──
    #[test]
    fn max_steps_budget_stops_simulation() {
        // Dense graph: 10 nodes fully connected
        let mut g = Graph::new();
        for i in 0..10u32 {
            g.add_node(
                &format!("n{}", i),
                &format!("fn_{}", i),
                NodeType::Function,
                &[],
                1.0,
                0.5,
            )
            .unwrap();
        }
        for i in 0..10u32 {
            for j in 0..10u32 {
                if i != j {
                    let _ = g.add_edge(
                        NodeId::new(i),
                        NodeId::new(j),
                        "calls",
                        FiniteF32::new(0.9),
                        EdgeDirection::Forward,
                        false,
                        FiniteF32::new(0.5),
                    );
                }
            }
        }
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let mut config = FlowConfig::default();
        config.max_total_steps = 5; // tiny budget
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();

        // Should complete without panic or hang
        let result = engine.simulate(&g, &[NodeId::new(0)], 1, &config);
        assert!(
            result.is_ok(),
            "simulation should succeed even with tiny budget"
        );
    }

    // ── Test 6: scope_filter — only nodes matching filter are visited ──
    #[test]
    fn scope_filter_restricts_visited_nodes() {
        // 3-node graph: entry → alpha_fn → beta_fn
        let mut g = Graph::new();
        g.add_node("e", "entry_point", NodeType::Function, &[], 1.0, 0.5)
            .unwrap();
        g.add_node("a", "alpha_fn", NodeType::Function, &[], 0.9, 0.4)
            .unwrap();
        g.add_node("b", "beta_fn", NodeType::Function, &[], 0.8, 0.3)
            .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(1),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let mut config = FlowConfig::default();
        config.scope_filter = Some("alpha".to_string()); // only alpha_fn passes
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();

        let result = engine.simulate(&g, &[NodeId::new(0)], 1, &config).unwrap();
        // entry always visited (scope filter only restricts propagation targets),
        // alpha_fn matches scope → visited; beta_fn does NOT match → not visited
        assert!(
            result.summary.total_nodes_visited <= 2,
            "scope filter should restrict to at most entry + alpha, got {}",
            result.summary.total_nodes_visited
        );
    }

    // ── Test 7: read_only — read-only node reduces turbulence score ──
    #[test]
    fn read_only_node_gets_reduced_turbulence() {
        // Two entries both converging on a "get_" prefixed node
        let mut g = Graph::new();
        g.add_node("e1", "handle_alpha", NodeType::Function, &[], 1.0, 0.5)
            .unwrap(); // 0
        g.add_node("e2", "handle_beta", NodeType::Function, &[], 1.0, 0.5)
            .unwrap(); // 1
        g.add_node("ro", "get_state", NodeType::Function, &[], 0.9, 0.4)
            .unwrap(); // 2
        g.add_edge(
            NodeId::new(0),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.9),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.5),
        )
        .unwrap();
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let mut config = FlowConfig::with_defaults();
        config.turbulence_threshold = 0.0;

        let result = engine
            .simulate(&g, &[NodeId::new(0), NodeId::new(1)], 1, &config)
            .unwrap();
        // get_state is read-only: score multiplied by 0.2 factor.
        // Either no turbulence points, or turbulence score is low.
        for tp in &result.turbulence_points {
            if tp.node_label.contains("get_state") {
                assert!(
                    tp.turbulence_score <= 0.3,
                    "read-only node should have low turbulence score, got {}",
                    tp.turbulence_score
                );
            }
        }
    }

    // ── Test 8: auto_discover — entry point discovery finds handle_ functions ──
    #[test]
    fn auto_discover_finds_handle_functions() {
        let mut g = Graph::new();
        g.add_node("h1", "handle_request", NodeType::Function, &[], 1.0, 0.5)
            .unwrap();
        g.add_node("h2", "handle_event", NodeType::Function, &[], 0.9, 0.4)
            .unwrap();
        g.add_node("u", "utility_helper", NodeType::Function, &[], 0.5, 0.2)
            .unwrap();
        g.add_edge(
            NodeId::new(0),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        g.add_edge(
            NodeId::new(1),
            NodeId::new(2),
            "calls",
            FiniteF32::new(0.8),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let entries = engine.discover_entry_points(&g, 10);
        // Both handle_ functions should be discovered
        assert_eq!(
            entries.len(),
            2,
            "expected 2 handle_ entry points, got {}",
            entries.len()
        );
    }

    // ── Test 9: advisory lock protection suppresses turbulence ──
    #[test]
    fn advisory_lock_suppresses_turbulence() {
        // Two entries converging on shared_state: same as convergent_graph
        let g = convergent_graph();
        let engine = FlowEngine::new();

        // First: run WITHOUT advisory lock — should produce turbulence
        let mut config_no_lock = FlowConfig::default();
        config_no_lock.turbulence_threshold = 0.0;
        config_no_lock.lock_patterns = Vec::new();
        config_no_lock.read_only_patterns = Vec::new();

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result_no_lock = engine
            .simulate(&g, &entry_nodes, 1, &config_no_lock)
            .unwrap();
        assert!(
            result_no_lock.summary.turbulence_count > 0,
            "baseline: expected turbulence without advisory lock"
        );
        assert!(result_no_lock.protected_nodes.is_empty());
        assert_eq!(result_no_lock.summary.advisory_lock_protected_count, 0);

        // Now: run WITH advisory lock protecting shared_state
        let mut config_locked = FlowConfig::default();
        config_locked.turbulence_threshold = 0.0;
        config_locked.lock_patterns = Vec::new();
        config_locked.read_only_patterns = Vec::new();
        config_locked
            .advisory_lock_protected_nodes
            .insert("shared_state".to_string(), vec!["lock_jimi_001".to_string()]);

        let result_locked = engine
            .simulate(&g, &entry_nodes, 1, &config_locked)
            .unwrap();

        // The protected_nodes list should contain shared_state
        assert_eq!(
            result_locked.summary.advisory_lock_protected_count, 1,
            "expected 1 protected node"
        );
        assert_eq!(result_locked.protected_nodes[0].node_label, "shared_state");
        assert_eq!(
            result_locked.protected_nodes[0].lock_ids,
            vec!["lock_jimi_001"]
        );
        assert!(result_locked.protected_nodes[0].original_score.is_some());

        // The turbulence score for shared_state should be 85% lower
        let tp_no_lock = result_no_lock
            .turbulence_points
            .iter()
            .find(|tp| tp.node_label == "shared_state");
        let tp_locked = result_locked
            .turbulence_points
            .iter()
            .find(|tp| tp.node_label == "shared_state");

        if let (Some(no_lock), Some(locked)) = (tp_no_lock, tp_locked) {
            assert!(
                locked.turbulence_score < no_lock.turbulence_score,
                "advisory lock should reduce turbulence score: {} should be < {}",
                locked.turbulence_score,
                no_lock.turbulence_score
            );
            let ratio = locked.turbulence_score / no_lock.turbulence_score;
            assert!(
                (ratio - 0.15).abs() < 0.01,
                "expected ~0.15 ratio, got {}",
                ratio
            );
        }
    }

    // ── Test 10: advisory lock suppresses below threshold ──
    #[test]
    fn advisory_lock_suppresses_below_threshold() {
        let g = convergent_graph();
        let engine = FlowEngine::new();

        // Use a threshold that the protected score falls below
        let mut config = FlowConfig::default();
        config.turbulence_threshold = 0.3;
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();
        config
            .advisory_lock_protected_nodes
            .insert("shared_state".to_string(), vec!["lock_test_001".to_string()]);

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result = engine.simulate(&g, &entry_nodes, 1, &config).unwrap();

        // shared_state should be in protected_nodes with suppressed=true
        let protected = result
            .protected_nodes
            .iter()
            .find(|p| p.node_label == "shared_state");
        assert!(
            protected.is_some(),
            "shared_state should appear in protected_nodes"
        );
        let protected = protected.unwrap();
        assert!(
            protected.suppressed,
            "shared_state should be marked as suppressed (score dropped below threshold)"
        );

        // It should NOT appear in turbulence_points (suppressed)
        let in_turbulence = result
            .turbulence_points
            .iter()
            .any(|tp| tp.node_label == "shared_state");
        assert!(
            !in_turbulence,
            "suppressed node should not appear in turbulence_points"
        );
    }

    // ── Test 11: advisory lock does not affect unprotected nodes ──
    #[test]
    fn advisory_lock_does_not_affect_unprotected_nodes() {
        let g = convergent_graph();
        let engine = FlowEngine::new();

        // Protect a node that is NOT in the convergence path
        let mut config = FlowConfig::default();
        config.turbulence_threshold = 0.0;
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();
        config
            .advisory_lock_protected_nodes
            .insert("unrelated_node".to_string(), vec!["lock_other_001".to_string()]);

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result = engine.simulate(&g, &entry_nodes, 1, &config).unwrap();

        // No protected nodes should appear (unrelated_node is not in the graph's turbulence set)
        assert!(
            result.protected_nodes.is_empty(),
            "unrelated lock should not produce protected_nodes"
        );
        assert_eq!(result.summary.advisory_lock_protected_count, 0);

        // Turbulence should still be detected at shared_state
        assert!(
            result.summary.turbulence_count > 0,
            "turbulence should still be detected for unprotected nodes"
        );
    }

    // ── Test 12: advisory lock with multiple lock IDs ──
    #[test]
    fn advisory_lock_multiple_lock_ids() {
        let g = convergent_graph();
        let engine = FlowEngine::new();

        let mut config = FlowConfig::default();
        config.turbulence_threshold = 0.0;
        config.lock_patterns = Vec::new();
        config.read_only_patterns = Vec::new();
        config.advisory_lock_protected_nodes.insert(
            "shared_state".to_string(),
            vec!["lock_a_001".to_string(), "lock_b_002".to_string()],
        );

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result = engine.simulate(&g, &entry_nodes, 1, &config).unwrap();

        let protected = result
            .protected_nodes
            .iter()
            .find(|p| p.node_label == "shared_state")
            .expect("shared_state should be in protected_nodes");
        assert_eq!(protected.lock_ids.len(), 2);
        assert!(protected.lock_ids.contains(&"lock_a_001".to_string()));
        assert!(protected.lock_ids.contains(&"lock_b_002".to_string()));
    }

    // ── Test 13: advisory lock prevents Critical severity ──
    #[test]
    fn advisory_lock_prevents_critical_severity() {
        // Build a graph with 4+ entries all converging on one node
        let mut g = Graph::new();
        for i in 0..4u32 {
            g.add_node(
                &format!("e{}", i),
                &format!("handle_ep{}", i),
                NodeType::Function,
                &[],
                1.0,
                0.5,
            )
            .unwrap();
        }
        g.add_node("shared", "shared_critical", NodeType::Function, &[], 0.9, 0.4)
            .unwrap(); // node 4
        for i in 0..4u32 {
            g.add_edge(
                NodeId::new(i),
                NodeId::new(4),
                "calls",
                FiniteF32::new(0.9),
                EdgeDirection::Forward,
                false,
                FiniteF32::new(0.5),
            )
            .unwrap();
        }
        g.finalize().unwrap();

        let engine = FlowEngine::new();
        let entries: Vec<NodeId> = (0..4).map(NodeId::new).collect();

        // Without lock: should be Critical
        let mut config_no_lock = FlowConfig::default();
        config_no_lock.turbulence_threshold = 0.0;
        config_no_lock.lock_patterns = Vec::new();
        config_no_lock.read_only_patterns = Vec::new();

        let result_no = engine.simulate(&g, &entries, 1, &config_no_lock).unwrap();
        let tp_no = result_no
            .turbulence_points
            .iter()
            .find(|tp| tp.node_label == "shared_critical");
        assert!(tp_no.is_some(), "should find turbulence without lock");

        // With lock: severity should NOT be Critical
        let mut config_locked = FlowConfig::default();
        config_locked.turbulence_threshold = 0.0;
        config_locked.lock_patterns = Vec::new();
        config_locked.read_only_patterns = Vec::new();
        config_locked.advisory_lock_protected_nodes.insert(
            "shared_critical".to_string(),
            vec!["lock_crit_001".to_string()],
        );

        let result_locked = engine.simulate(&g, &entries, 1, &config_locked).unwrap();
        let tp_locked = result_locked
            .turbulence_points
            .iter()
            .find(|tp| tp.node_label == "shared_critical");
        if let Some(tp) = tp_locked {
            assert_ne!(
                tp.severity,
                TurbulenceSeverity::Critical,
                "advisory-locked node should not be Critical severity"
            );
        }
    }

    // ── Test 14: empty advisory lock map is backward compatible ──
    #[test]
    fn empty_advisory_lock_map_backward_compatible() {
        let g = convergent_graph();
        let engine = FlowEngine::new();

        let config = FlowConfig {
            turbulence_threshold: 0.0,
            lock_patterns: Vec::new(),
            read_only_patterns: Vec::new(),
            advisory_lock_protected_nodes: BTreeMap::new(),
            ..FlowConfig::default()
        };

        let entry_nodes = vec![NodeId::new(0), NodeId::new(1)];
        let result = engine.simulate(&g, &entry_nodes, 1, &config).unwrap();

        assert!(result.protected_nodes.is_empty());
        assert_eq!(result.summary.advisory_lock_protected_count, 0);
        // Turbulence still works normally
        assert!(result.summary.turbulence_count > 0);
    }
}
