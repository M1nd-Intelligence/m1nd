// === crates/m1nd-core/src/temporal.rs ===

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use crate::domain::DomainConfig;
use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants from temporal_v2.py
// ---------------------------------------------------------------------------

/// Default half-life for temporal decay (hours).
pub const DEFAULT_HALF_LIFE_HOURS: f32 = 168.0; // 7 days
/// Decay formula: exp(-k * age_hours), k = ln(2) / half_life.
pub const LN2: f32 = std::f32::consts::LN_2;
/// Default chain budget (FM-TMP-005). Prevents combinatorial explosion.
pub const DEFAULT_CHAIN_BUDGET: u64 = 10_000;
/// Default max co-change matrix entries (FM-TMP-001). Prevents O(N^2) memory.
pub const DEFAULT_MATRIX_BUDGET: u64 = 500_000;
/// Default causal chain max depth.
pub const DEFAULT_CHAIN_MAX_DEPTH: u8 = 6;
/// Resurrection dormancy depth threshold (days).
pub const RESURRECTION_DORMANCY_THRESHOLD_DAYS: f32 = 35.0;
/// Co-change decay per BFS hop.
pub const CO_CHANGE_DECAY_FACTOR: f32 = 0.95;
/// Bootstrap weight relative to learned values.
pub const BOOTSTRAP_WEIGHT: f32 = 0.3;
/// Max entries per row in co-change matrix.
pub const CO_CHANGE_MAX_ROW: usize = 100;
/// Dormant hours threshold.
pub const DORMANT_HOURS: f64 = 720.0;
/// Resurrection additive floor.
pub const RESURRECTION_BASE_FLOOR: f32 = 0.3;
/// Resurrection depth scale.
pub const RESURRECTION_DEPTH_SCALE: f32 = 0.1;
/// Raw decay floor to prevent underflow.
pub const RAW_DECAY_FLOOR: f32 = 1e-6;

// ---------------------------------------------------------------------------
// CoChangeMatrix — sparse co-change tracking (temporal_v2.py CoChangeMatrix)
// FM-TMP-001 fix: CSR-like sparse storage with hard entry budget.
// Replaces: temporal_v2.py CoChangeMatrix (Python HashMap<HashMap>)
// ---------------------------------------------------------------------------

/// Entry in the co-change matrix: (target_node, coupling_strength).
#[derive(Clone, Copy, Debug)]
pub struct CoChangeEntry {
    pub target: NodeId,
    pub strength: FiniteF32,
}

/// Sparse co-change matrix with bounded entry count.
/// Bootstrapped from graph structure (BFS depth 3 from each node),
/// refined with real co-change observations via `record_co_change`.
/// FM-TMP-001: hard cap on total entries prevents O(N^2) memory.
pub struct CoChangeMatrix {
    /// Per-node sorted list of co-change entries.
    rows: Vec<Vec<CoChangeEntry>>,
    /// Total entries across all rows (for budget enforcement).
    total_entries: u64,
    /// Maximum total entries allowed.
    budget: u64,
    /// Whether bootstrap or learned from real observations.
    is_learned: bool,
}

impl CoChangeMatrix {
    /// Bootstrap co-change from graph structure (BFS depth 3 from each node).
    /// Replaces: temporal_v2.py CoChangeMatrix.bootstrap()
    pub fn bootstrap(graph: &Graph, budget: u64) -> M1ndResult<Self> {
        let n = graph.num_nodes() as usize;
        let mut rows = vec![Vec::new(); n];
        let mut total_entries = 0u64;

        for start in 0..n {
            if total_entries >= budget {
                break;
            }

            let start_node = NodeId::new(start as u32);
            let mut visited = vec![false; n];
            visited[start] = true;

            let mut queue = VecDeque::new();
            queue.push_back((start_node, 0u8, 1.0f32));

            let mut entries: Vec<CoChangeEntry> = Vec::new();

            while let Some((node, depth, strength)) = queue.pop_front() {
                if depth >= 3 {
                    continue;
                }

                let range = graph.csr.out_range(node);
                for j in range {
                    let tgt = graph.csr.targets[j];
                    let tgt_idx = tgt.as_usize();
                    if tgt_idx >= n || visited[tgt_idx] {
                        continue;
                    }
                    visited[tgt_idx] = true;

                    let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                    let new_strength = strength * w * CO_CHANGE_DECAY_FACTOR * BOOTSTRAP_WEIGHT;

                    if new_strength > 0.001 && entries.len() < CO_CHANGE_MAX_ROW {
                        entries.push(CoChangeEntry {
                            target: tgt,
                            strength: FiniteF32::new(new_strength),
                        });
                    }

                    queue.push_back((tgt, depth + 1, new_strength));
                }
            }

            entries.sort_by(|a, b| b.strength.cmp(&a.strength));
            entries.truncate(CO_CHANGE_MAX_ROW);
            total_entries += entries.len() as u64;
            rows[start] = entries;
        }

        Ok(Self {
            rows,
            total_entries,
            budget,
            is_learned: false,
        })
    }

    /// Record an observed co-change between two nodes.
    /// Updates coupling strength. Respects budget cap (FM-TMP-001).
    /// Replaces: temporal_v2.py CoChangeMatrix.record_co_change()
    pub fn record_co_change(
        &mut self,
        source: NodeId,
        target: NodeId,
        _timestamp: f64,
    ) -> M1ndResult<()> {
        let src_idx = source.as_usize();
        if src_idx >= self.rows.len() {
            return Ok(());
        }

        // Check if entry already exists
        if let Some(entry) = self.rows[src_idx].iter_mut().find(|e| e.target == target) {
            // Strengthen existing
            entry.strength = FiniteF32::new((entry.strength.get() + 0.1).min(1.0));
            self.is_learned = true;
            return Ok(());
        }

        // Budget check
        if self.total_entries >= self.budget {
            return Err(M1ndError::MatrixBudgetExhausted {
                budget: self.budget,
            });
        }

        // Row capacity check
        if self.rows[src_idx].len() >= CO_CHANGE_MAX_ROW {
            // Replace weakest entry
            if let Some(weakest) = self.rows[src_idx]
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.strength.cmp(&b.1.strength))
                .map(|(i, _)| i)
            {
                self.rows[src_idx][weakest] = CoChangeEntry {
                    target,
                    strength: FiniteF32::new(0.1),
                };
            }
        } else {
            self.rows[src_idx].push(CoChangeEntry {
                target,
                strength: FiniteF32::new(0.1),
            });
            self.total_entries += 1;
        }

        self.is_learned = true;
        Ok(())
    }

    /// Predict co-change partners for a changed node, sorted by coupling strength.
    /// Replaces: temporal_v2.py CoChangeMatrix.predict()
    pub fn predict(&self, changed_node: NodeId, top_k: usize) -> Vec<CoChangeEntry> {
        let idx = changed_node.as_usize();
        if idx >= self.rows.len() {
            return Vec::new();
        }
        let mut entries = self.rows[idx].clone();
        entries.sort_by(|a, b| b.strength.cmp(&a.strength));
        entries.truncate(top_k);
        entries
    }

    /// Number of entries in the matrix.
    pub fn num_entries(&self) -> u64 {
        self.total_entries
    }

    /// Populate co-change data from git commit groups.
    /// Each group is a list of external_ids (e.g. "file::src/main.rs") that changed together.
    /// Resolves IDs via the graph, then records co-change for each pair in the group.
    pub fn populate_from_commit_groups(
        &mut self,
        graph: &Graph,
        commit_groups: &[Vec<String>],
    ) -> M1ndResult<()> {
        for group in commit_groups {
            // Resolve external IDs to NodeIds
            let node_ids: Vec<NodeId> = group
                .iter()
                .filter_map(|path| {
                    let file_id = if path.starts_with("file::") {
                        path.clone()
                    } else {
                        format!("file::{}", path)
                    };
                    graph.resolve_id(&file_id)
                })
                .collect();

            // Record co-change for each pair in the group
            for i in 0..node_ids.len() {
                for j in (i + 1)..node_ids.len() {
                    // Record both directions
                    let _ = self.record_co_change(node_ids[i], node_ids[j], 0.0);
                    let _ = self.record_co_change(node_ids[j], node_ids[i], 0.0);
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CausalChain — causal chain detection (temporal_v2.py CausalChainDetector)
// FM-TMP-005 fix: budget-limited DFS.
// ---------------------------------------------------------------------------

/// A single causal chain: path of nodes with cumulative strength.
/// Replaces: temporal_v2.py CausalChain dataclass
#[derive(Clone, Debug)]
pub struct CausalChain {
    /// Ordered list of nodes in the chain.
    pub path: Vec<NodeId>,
    /// Relation labels between consecutive nodes.
    pub relations: Vec<InternedStr>,
    /// Cumulative causal strength (product of edge causal_strengths).
    pub cumulative_strength: FiniteF32,
}

/// Heap entry for priority-queue chain detection.
#[derive(Clone)]
struct ChainEntry {
    path: Vec<NodeId>,
    relations: Vec<InternedStr>,
    cumulative: f32,
}

impl PartialEq for ChainEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cumulative == other.cumulative
    }
}
impl Eq for ChainEntry {}
impl PartialOrd for ChainEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ChainEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cumulative.total_cmp(&other.cumulative)
    }
}

/// Causal chain detector. DFS along forward causal edges with budget.
/// Replaces: temporal_v2.py CausalChainDetector
pub struct CausalChainDetector {
    max_depth: u8,
    min_strength: FiniteF32,
    chain_budget: u64,
}

impl CausalChainDetector {
    pub fn new(max_depth: u8, min_strength: FiniteF32, chain_budget: u64) -> Self {
        Self {
            max_depth,
            min_strength,
            chain_budget,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            max_depth: DEFAULT_CHAIN_MAX_DEPTH,
            min_strength: FiniteF32::new(0.1),
            chain_budget: DEFAULT_CHAIN_BUDGET,
        }
    }

    /// Detect causal chains from a source node. Budget-limited priority queue (DEC-016, FM-TMP-005).
    /// Replaces: temporal_v2.py CausalChainDetector.detect()
    pub fn detect(&self, graph: &Graph, source: NodeId) -> M1ndResult<Vec<CausalChain>> {
        let n = graph.num_nodes() as usize;
        if source.as_usize() >= n {
            return Ok(Vec::new());
        }

        let mut heap = BinaryHeap::new();
        let mut chains = Vec::new();
        let mut ops = 0u64;

        heap.push(ChainEntry {
            path: vec![source],
            relations: Vec::new(),
            cumulative: 1.0,
        });

        while let Some(entry) = heap.pop() {
            ops += 1;
            if ops > self.chain_budget {
                break; // FM-TMP-005: budget exhausted
            }

            if entry.cumulative < self.min_strength.get() {
                continue;
            }

            let current = *entry.path.last().unwrap();
            let depth = entry.path.len();

            if depth > 1 {
                // Record complete chain
                chains.push(CausalChain {
                    path: entry.path.clone(),
                    relations: entry.relations.clone(),
                    cumulative_strength: FiniteF32::new(entry.cumulative),
                });
            }

            if depth > self.max_depth as usize {
                continue;
            }

            // Extend along causal edges
            let range = graph.csr.out_range(current);
            for j in range {
                let causal = graph.csr.causal_strengths[j].get();
                if causal <= 0.0 {
                    continue;
                }
                let tgt = graph.csr.targets[j];
                // Avoid cycles
                if entry.path.contains(&tgt) {
                    continue;
                }

                let new_cumulative = entry.cumulative * causal;
                if new_cumulative < self.min_strength.get() {
                    continue;
                }

                let mut new_path = entry.path.clone();
                new_path.push(tgt);
                let mut new_rels = entry.relations.clone();
                new_rels.push(graph.csr.relations[j]);

                heap.push(ChainEntry {
                    path: new_path,
                    relations: new_rels,
                    cumulative: new_cumulative,
                });
            }
        }

        chains.sort_by(|a, b| b.cumulative_strength.cmp(&a.cumulative_strength));
        Ok(chains)
    }
}

// ---------------------------------------------------------------------------
// TemporalDecay — decay + resurrection (temporal_v2.py TemporalDecayResurrection)
// ---------------------------------------------------------------------------

/// Per-node temporal decay score.
/// Replaces: temporal_v2.py TemporalDecayResurrection.score()
#[derive(Clone, Copy, Debug)]
pub struct DecayScore {
    pub node: NodeId,
    /// Raw decay: exp(-k * age_hours).
    pub raw_decay: FiniteF32,
    /// Resurrection multiplier (if node went dormant then active again).
    /// FM-TMP-007 fix: additive-floor + blend instead of pure multiplicative.
    pub resurrection_multiplier: FiniteF32,
    /// Final score: raw_decay * resurrection_mult * dormancy_depth.
    pub final_score: FiniteF32,
}

/// Temporal decay with resurrection for dormant-then-active nodes.
/// Per-NodeType half-lives: files decay fast (7d), modules/dirs slow (30d).
/// Replaces: temporal_v2.py TemporalDecayResurrection
pub struct TemporalDecayScorer {
    /// Default k = ln(2) / half_life
    default_k: PosF32,
}

impl TemporalDecayScorer {
    pub fn new(half_life_hours: PosF32) -> Self {
        let k = PosF32::new(LN2 / half_life_hours.get()).unwrap();
        Self { default_k: k }
    }

    /// Per-NodeType half-life in hours. Structural nodes decay slower.
    fn k_for_type(node_type: NodeType) -> f32 {
        let half_life = match node_type {
            NodeType::File => 168.0,     // 7 days — active dev artifact
            NodeType::Function => 336.0, // 14 days
            NodeType::Class | NodeType::Struct | NodeType::Enum => 504.0, // 21 days
            NodeType::Module | NodeType::Directory => 720.0, // 30 days — stable structure
            NodeType::Type => 504.0,     // 21 days
            _ => 168.0,                  // default 7 days
        };
        LN2 / half_life
    }

    /// Score a single node. Clamps negative age_hours to 0 (FM-TMP-009).
    /// DEC-004: additive-floor resurrection.
    /// Replaces: temporal_v2.py TemporalDecayResurrection.score_one()
    pub fn score_one(
        &self,
        age_hours: f64,
        change_frequency: FiniteF32,
        last_dormancy_hours: Option<f64>,
    ) -> DecayScore {
        self.score_one_typed(age_hours, change_frequency, last_dormancy_hours, None)
    }

    /// Score with NodeType-specific half-life.
    /// When `domain_config` is Some, uses domain-specific half-lives instead
    /// of the hardcoded k_for_type() values (backward compat fallback).
    pub fn score_one_typed(
        &self,
        age_hours: f64,
        change_frequency: FiniteF32,
        last_dormancy_hours: Option<f64>,
        node_type: Option<NodeType>,
    ) -> DecayScore {
        self.score_one_with_domain(
            age_hours,
            change_frequency,
            last_dormancy_hours,
            node_type,
            None,
        )
    }

    /// Score with NodeType-specific half-life and optional DomainConfig override.
    pub fn score_one_with_domain(
        &self,
        age_hours: f64,
        change_frequency: FiniteF32,
        last_dormancy_hours: Option<f64>,
        node_type: Option<NodeType>,
        domain_config: Option<&DomainConfig>,
    ) -> DecayScore {
        // FM-TMP-009: clamp negative age (future timestamp)
        let age = age_hours.max(0.0);

        // Use domain config half-life when provided, else fall back to hardcoded k_for_type
        let k = match (domain_config, node_type) {
            (Some(dc), Some(nt)) => LN2 / dc.half_life_for(nt),
            (Some(dc), None) => LN2 / dc.default_half_life,
            (None, Some(nt)) => Self::k_for_type(nt),
            (None, None) => self.default_k.get(),
        };

        // Raw exponential decay
        let raw = (-(age as f32) * k).exp().max(RAW_DECAY_FLOOR);
        let raw_decay = FiniteF32::new(raw);

        // DEC-004: resurrection with additive floor
        let (resurrection, final_score) = match last_dormancy_hours {
            Some(dormancy) if dormancy > DORMANT_HOURS => {
                let dormancy_depth =
                    (dormancy / (RESURRECTION_DORMANCY_THRESHOLD_DAYS as f64 * 24.0)) as f32;
                let res = RESURRECTION_BASE_FLOOR
                    + RESURRECTION_DEPTH_SCALE * (dormancy_depth + 1.0).ln();
                let res_clamped = res.max(0.0).min(1.0);
                let final_val = raw.max(res_clamped);
                (FiniteF32::new(res_clamped), FiniteF32::new(final_val))
            }
            _ => (FiniteF32::ONE, raw_decay),
        };

        DecayScore {
            node: NodeId::default(),
            raw_decay,
            resurrection_multiplier: resurrection,
            final_score,
        }
    }

    /// Score all nodes in the graph with per-NodeType half-lives.
    /// Replaces: temporal_v2.py TemporalDecayResurrection.score()
    pub fn score_all(&self, graph: &Graph, now_unix: f64) -> M1ndResult<Vec<DecayScore>> {
        self.score_all_with_domain(graph, now_unix, None)
    }

    /// Score all nodes with optional DomainConfig override for half-lives.
    pub fn score_all_with_domain(
        &self,
        graph: &Graph,
        now_unix: f64,
        domain_config: Option<&DomainConfig>,
    ) -> M1ndResult<Vec<DecayScore>> {
        let n = graph.num_nodes() as usize;
        let mut scores = Vec::with_capacity(n);

        for i in 0..n {
            let last_mod = graph.nodes.last_modified[i];
            let age_hours = (now_unix - last_mod) / 3600.0;
            let freq = graph.nodes.change_frequency[i];
            let nt = graph.nodes.node_type[i];

            let mut ds = self.score_one_with_domain(age_hours, freq, None, Some(nt), domain_config);
            ds.node = NodeId::new(i as u32);
            scores.push(ds);
        }

        Ok(scores)
    }
}

// ---------------------------------------------------------------------------
// VelocityScorer — change velocity (temporal_v2.py VelocityScorer)
// ---------------------------------------------------------------------------

/// Velocity score for a single node.
/// Replaces: temporal_v2.py VelocityScorer.score() per-node output
#[derive(Clone, Copy, Debug)]
pub struct VelocityScore {
    pub node: NodeId,
    pub velocity: FiniteF32,
    pub trend: VelocityTrend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VelocityTrend {
    Accelerating,
    Decelerating,
    Stable,
}

/// Velocity scorer: measures rate of change frequency over time.
/// Caches mean+stddev stats and invalidates when graph node count changes.
/// Replaces: temporal_v2.py VelocityScorer
pub struct VelocityScorer {
    /// Cached (mean, stddev) stats and the node count they were computed for.
    cached_stats: Option<(u32, f32, f32)>,
}

impl VelocityScorer {
    pub fn new() -> Self {
        Self { cached_stats: None }
    }

    /// Compute or retrieve cached mean and standard deviation of change frequencies.
    /// Invalidates cache when graph node count changes (new nodes added).
    fn frequency_stats(&mut self, graph: &Graph) -> (f32, f32) {
        let n = graph.num_nodes();
        if n == 0 {
            return (0.0, 1.0);
        }

        // Check cache validity
        if let Some((cached_n, mean, stddev)) = self.cached_stats {
            if cached_n == n {
                return (mean, stddev);
            }
        }

        // Recompute
        let n_usize = n as usize;
        let mean: f32 = (0..n_usize)
            .map(|j| graph.nodes.change_frequency[j].get())
            .sum::<f32>()
            / n_usize as f32;
        let variance: f32 = (0..n_usize)
            .map(|j| {
                let d = graph.nodes.change_frequency[j].get() - mean;
                d * d
            })
            .sum::<f32>()
            / n_usize as f32;
        let stddev = variance.sqrt().max(0.01); // floor to avoid div-by-zero

        self.cached_stats = Some((n, mean, stddev));
        (mean, stddev)
    }

    /// Compute stats without caching (static helper for backward compat).
    fn frequency_stats_static(graph: &Graph) -> (f32, f32) {
        let n = graph.num_nodes() as usize;
        if n == 0 {
            return (0.0, 1.0);
        }
        let mean: f32 = (0..n)
            .map(|j| graph.nodes.change_frequency[j].get())
            .sum::<f32>()
            / n as f32;
        let variance: f32 = (0..n)
            .map(|j| {
                let d = graph.nodes.change_frequency[j].get() - mean;
                d * d
            })
            .sum::<f32>()
            / n as f32;
        let stddev = variance.sqrt().max(0.01);
        (mean, stddev)
    }

    /// Score all nodes using z-score velocity. Returns nodes with |z| > 0.1.
    /// Replaces: temporal_v2.py VelocityScorer.score()
    pub fn score_all(graph: &Graph, _now_unix: f64) -> M1ndResult<Vec<VelocityScore>> {
        let n = graph.num_nodes() as usize;
        let (mean, stddev) = Self::frequency_stats_static(graph);
        let mut scores = Vec::new();

        for i in 0..n {
            let freq = graph.nodes.change_frequency[i].get();
            let z = (freq - mean) / stddev;
            let trend = if z > 0.5 {
                VelocityTrend::Accelerating
            } else if z < -0.5 {
                VelocityTrend::Decelerating
            } else {
                VelocityTrend::Stable
            };

            if z.abs() > 0.1 {
                scores.push(VelocityScore {
                    node: NodeId::new(i as u32),
                    velocity: FiniteF32::new(z),
                    trend,
                });
            }
        }

        Ok(scores)
    }

    /// Score all nodes using cached stats. Prefer this when scorer is reused.
    pub fn score_all_cached(
        &mut self,
        graph: &Graph,
        _now_unix: f64,
    ) -> M1ndResult<Vec<VelocityScore>> {
        let n = graph.num_nodes() as usize;
        let (mean, stddev) = self.frequency_stats(graph);
        let mut scores = Vec::new();

        for i in 0..n {
            let freq = graph.nodes.change_frequency[i].get();
            let z = (freq - mean) / stddev;
            let trend = if z > 0.5 {
                VelocityTrend::Accelerating
            } else if z < -0.5 {
                VelocityTrend::Decelerating
            } else {
                VelocityTrend::Stable
            };

            if z.abs() > 0.1 {
                scores.push(VelocityScore {
                    node: NodeId::new(i as u32),
                    velocity: FiniteF32::new(z),
                    trend,
                });
            }
        }

        Ok(scores)
    }

    /// Score a single node using z-score.
    pub fn score_one(graph: &Graph, node: NodeId, _now_unix: f64) -> M1ndResult<VelocityScore> {
        let idx = node.as_usize();
        let n = graph.num_nodes() as usize;
        let (mean, stddev) = Self::frequency_stats_static(graph);
        let freq = if idx < n {
            graph.nodes.change_frequency[idx].get()
        } else {
            0.0
        };
        let z = (freq - mean) / stddev;
        let trend = if z > 0.5 {
            VelocityTrend::Accelerating
        } else if z < -0.5 {
            VelocityTrend::Decelerating
        } else {
            VelocityTrend::Stable
        };
        Ok(VelocityScore {
            node,
            velocity: FiniteF32::new(z),
            trend,
        })
    }

    /// Invalidate the cached stats (call when graph structure changes).
    pub fn invalidate_cache(&mut self) {
        self.cached_stats = None;
    }
}

// ---------------------------------------------------------------------------
// ImpactRadius — blast radius calculation (temporal_v2.py ImpactRadiusCalculator)
// ---------------------------------------------------------------------------

/// Impact result for a single downstream node.
/// Replaces: temporal_v2.py ImpactResult
#[derive(Clone, Debug)]
pub struct ImpactEntry {
    pub node: NodeId,
    pub signal_strength: FiniteF32,
    pub hop_distance: u8,
}

/// Impact radius result.
/// Replaces: temporal_v2.py ImpactRadiusCalculator.compute() return
#[derive(Clone, Debug)]
pub struct ImpactResult {
    pub source: NodeId,
    /// Nodes in blast radius sorted by signal strength descending.
    pub blast_radius: Vec<ImpactEntry>,
    /// Total energy dispersed.
    pub total_energy: FiniteF32,
    /// Maximum hops reached.
    pub max_hops_reached: u8,
}

/// Impact radius calculator. BFS from source along causal edges.
/// Replaces: temporal_v2.py ImpactRadiusCalculator
pub struct ImpactRadiusCalculator {
    max_hops: u8,
    min_signal: FiniteF32,
}

impl ImpactRadiusCalculator {
    pub fn new(max_hops: u8, min_signal: FiniteF32) -> Self {
        Self {
            max_hops,
            min_signal,
        }
    }

    /// Compute impact radius from source. Direction: forward, reverse, or both.
    /// DEC-009: sum-within-hop (superposition), max-across-hops (strongest arrival wins).
    /// Replaces: temporal_v2.py ImpactRadiusCalculator.compute()
    pub fn compute(
        &self,
        graph: &Graph,
        source: NodeId,
        direction: ImpactDirection,
    ) -> M1ndResult<ImpactResult> {
        let n = graph.num_nodes() as usize;
        if source.as_usize() >= n {
            return Ok(ImpactResult {
                source,
                blast_radius: Vec::new(),
                total_energy: FiniteF32::ZERO,
                max_hops_reached: 0,
            });
        }

        let mut best_signal = vec![0.0f32; n];
        let mut hop_dist = vec![u8::MAX; n];
        best_signal[source.as_usize()] = 1.0;
        hop_dist[source.as_usize()] = 0;

        let decay = 0.55f32; // Use default decay
        let mut frontier = vec![source];
        let mut max_hops = 0u8;

        for depth in 0..self.max_hops {
            if frontier.is_empty() {
                break;
            }
            max_hops = depth + 1;
            let mut next = Vec::new();

            for &node in &frontier {
                let signal = best_signal[node.as_usize()];

                // Forward edges
                if direction != ImpactDirection::Reverse {
                    let range = graph.csr.out_range(node);
                    for j in range {
                        let tgt = graph.csr.targets[j];
                        let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                        let new_signal = signal * w * decay;
                        let tgt_idx = tgt.as_usize();
                        if tgt_idx < n && new_signal > self.min_signal.get() {
                            // DEC-009: max-across-hops
                            if new_signal > best_signal[tgt_idx] {
                                best_signal[tgt_idx] = new_signal;
                                hop_dist[tgt_idx] = depth + 1;
                                next.push(tgt);
                            }
                        }
                    }
                }

                // Reverse edges
                if direction != ImpactDirection::Forward {
                    let range = graph.csr.in_range(node);
                    for j in range {
                        let rev_src = graph.csr.rev_sources[j];
                        let fwd_idx = graph.csr.rev_edge_idx[j];
                        let w = graph.csr.read_weight(fwd_idx).get();
                        let new_signal = signal * w * decay;
                        let idx = rev_src.as_usize();
                        if idx < n && new_signal > self.min_signal.get() {
                            if new_signal > best_signal[idx] {
                                best_signal[idx] = new_signal;
                                hop_dist[idx] = depth + 1;
                                next.push(rev_src);
                            }
                        }
                    }
                }
            }

            frontier = next;
        }

        let mut blast_radius: Vec<ImpactEntry> = (0..n)
            .filter(|&i| i != source.as_usize() && best_signal[i] > self.min_signal.get())
            .map(|i| ImpactEntry {
                node: NodeId::new(i as u32),
                signal_strength: FiniteF32::new(best_signal[i]),
                hop_distance: hop_dist[i],
            })
            .collect();

        blast_radius.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
        let total_energy: f32 = blast_radius.iter().map(|e| e.signal_strength.get()).sum();

        Ok(ImpactResult {
            source,
            blast_radius,
            total_energy: FiniteF32::new(total_energy),
            max_hops_reached: max_hops,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactDirection {
    Forward,
    Reverse,
    Both,
}

// ---------------------------------------------------------------------------
// TemporalEngine — facade combining all temporal capabilities
// Replaces: temporal_v2.py TemporalPredictor
// ---------------------------------------------------------------------------

/// Facade for all temporal analysis capabilities.
/// Replaces: temporal_v2.py TemporalPredictor
pub struct TemporalEngine {
    pub co_change: CoChangeMatrix,
    pub chain_detector: CausalChainDetector,
    pub decay_scorer: TemporalDecayScorer,
    pub impact_calculator: ImpactRadiusCalculator,
}

impl TemporalEngine {
    /// Build from graph with default parameters.
    /// Replaces: temporal_v2.py TemporalPredictor.__init__()
    pub fn build(graph: &Graph) -> M1ndResult<Self> {
        let co_change = CoChangeMatrix::bootstrap(graph, DEFAULT_MATRIX_BUDGET)?;
        let chain_detector = CausalChainDetector::with_defaults();
        let decay_scorer = TemporalDecayScorer::new(PosF32::new(DEFAULT_HALF_LIFE_HOURS).unwrap());
        let impact_calculator = ImpactRadiusCalculator::new(5, FiniteF32::new(0.01));

        Ok(Self {
            co_change,
            chain_detector,
            decay_scorer,
            impact_calculator,
        })
    }

    /// Populate co-change matrix from git commit groups.
    /// Call after build() with commit groups from ingestion.
    pub fn populate_co_change(
        &mut self,
        graph: &Graph,
        commit_groups: &[Vec<String>],
    ) -> M1ndResult<()> {
        self.co_change
            .populate_from_commit_groups(graph, commit_groups)
    }

    /// Full temporal report for a node: co-change predictions + causal chains
    /// + decay score + velocity + impact radius.
    /// Replaces: temporal_v2.py TemporalPredictor.full_report()
    pub fn full_report(
        &self,
        graph: &Graph,
        node: NodeId,
        now_unix: f64,
    ) -> M1ndResult<TemporalReport> {
        let co_change_predictions = self.co_change.predict(node, 10);
        let causal_chains = self.chain_detector.detect(graph, node)?;

        let idx = node.as_usize();
        let n = graph.num_nodes() as usize;
        let last_mod = if idx < n {
            graph.nodes.last_modified[idx]
        } else {
            0.0
        };
        let age_hours = (now_unix - last_mod) / 3600.0;
        let freq = if idx < n {
            graph.nodes.change_frequency[idx]
        } else {
            FiniteF32::ZERO
        };
        let nt = if idx < n {
            Some(graph.nodes.node_type[idx])
        } else {
            None
        };
        let mut decay = self.decay_scorer.score_one_typed(age_hours, freq, None, nt);
        decay.node = node;

        let velocity = VelocityScorer::score_one(graph, node, now_unix)?;
        let impact = self
            .impact_calculator
            .compute(graph, node, ImpactDirection::Both)?;

        Ok(TemporalReport {
            node,
            co_change_predictions,
            causal_chains,
            decay,
            velocity,
            impact,
        })
    }
}

/// Complete temporal analysis for one node.
#[derive(Clone, Debug)]
pub struct TemporalReport {
    pub node: NodeId,
    pub co_change_predictions: Vec<CoChangeEntry>,
    pub causal_chains: Vec<CausalChain>,
    pub decay: DecayScore,
    pub velocity: VelocityScore,
    pub impact: ImpactResult,
}

// Ensure Send + Sync.
static_assertions::assert_impl_all!(TemporalEngine: Send, Sync);
