// === crates/m1nd-core/src/activation.rs ===

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::Instant;

use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// BloomFilter — probabilistic visited set (engine_fast.py BloomFilter)
// Replaces: engine_fast.py BloomFilter class
// ---------------------------------------------------------------------------

/// Double-hashing Bloom filter for fast visited checks.
/// FPR ~ (1 - e^(-kn/m))^k where k=hash count, n=insertions, m=bits.
pub struct BloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: u32,
}

impl BloomFilter {
    /// Create with capacity for `expected_items` at the given false-positive rate.
    pub fn with_capacity(expected_items: usize, fpr: f64) -> Self {
        let expected = expected_items.max(1);
        // m = -(n * ln(p)) / (ln(2)^2)
        let m = (-(expected as f64) * fpr.ln() / (2.0f64.ln().powi(2))) as usize;
        let num_bits = m.max(64);
        let num_words = (num_bits + 63) / 64;
        // k = (m/n) * ln(2)
        let k = ((num_bits as f64 / expected as f64) * 2.0f64.ln()).ceil() as u32;
        let num_hashes = k.max(1).min(16);
        Self {
            bits: vec![0u64; num_words],
            num_bits,
            num_hashes,
        }
    }

    #[inline]
    fn compute_hashes(&self, item: u32, out: &mut [usize; 16]) -> u32 {
        let h1 = item.wrapping_mul(2654435761) as usize;
        let h2 = item.wrapping_mul(2246822519).wrapping_add(1) as usize;
        let m = self.num_bits;
        let k = self.num_hashes.min(16);
        for i in 0..k as usize {
            out[i] = h1.wrapping_add(i.wrapping_mul(h2)) % m;
        }
        k
    }

    pub fn insert(&mut self, item: NodeId) {
        let mut hashes = [0usize; 16];
        let k = self.compute_hashes(item.0, &mut hashes);
        for i in 0..k as usize {
            let h = hashes[i];
            self.bits[h >> 6] |= 1u64 << (h & 63);
        }
    }

    pub fn probably_contains(&self, item: NodeId) -> bool {
        let mut hashes = [0usize; 16];
        let k = self.compute_hashes(item.0, &mut hashes);
        for i in 0..k as usize {
            let h = hashes[i];
            if self.bits[h >> 6] & (1u64 << (h & 63)) == 0 {
                return false;
            }
        }
        true
    }

    pub fn clear(&mut self) {
        self.bits.fill(0);
    }
}

// ---------------------------------------------------------------------------
// ActivationResult — query output (04-SPEC Section 4.2)
// Replaces: engine_v2.py ConnectomeEngine.query() return value
// ---------------------------------------------------------------------------

/// Per-node activation detail in a query result.
#[derive(Clone, Debug)]
pub struct ActivatedNode {
    pub node: NodeId,
    /// Combined activation score after dimension weighting + resonance bonus.
    pub activation: FiniteF32,
    /// Per-dimension scores [structural, semantic, temporal, causal].
    pub dimensions: [FiniteF32; 4],
    /// Number of dimensions that contributed above threshold.
    pub active_dimension_count: u8,
}

/// Result of a full activation query.
#[derive(Clone, Debug)]
pub struct ActivationResult {
    /// Activated nodes sorted by activation descending. Truncated to top_k.
    pub activated: Vec<ActivatedNode>,
    /// Seed nodes that initiated propagation.
    pub seeds: Vec<(NodeId, FiniteF32)>,
    /// Wall-clock time in nanoseconds.
    pub elapsed_ns: u64,
    /// Whether XLR over-cancellation fallback was triggered (FM-XLR-010).
    pub xlr_fallback_used: bool,
}

/// Per-dimension raw result before merging.
/// Replaces: engine_v2.py DimensionResult
#[derive(Clone, Debug)]
pub struct DimensionResult {
    /// Sparse map: node -> raw score for this dimension.
    pub scores: Vec<(NodeId, FiniteF32)>,
    /// Dimension that produced this result.
    pub dimension: Dimension,
    /// Elapsed nanoseconds for this dimension.
    pub elapsed_ns: u64,
}

// ---------------------------------------------------------------------------
// ActivationEngine — trait for propagation strategies
// Replaces: engine_fast.py HeapActivationEngine, WavefrontEngine, HybridEngine
// ---------------------------------------------------------------------------

/// Propagation strategy for structural activation (D1).
/// Concrete impls below — no dyn dispatch.
pub trait ActivationEngine: Send + Sync {
    /// Propagate from seeds through the graph. Returns sparse activation map.
    /// Replaces: engine_v2.py D1_Structural.activate()
    fn propagate(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        config: &PropagationConfig,
    ) -> M1ndResult<DimensionResult>;
}

// ---------------------------------------------------------------------------
// WavefrontEngine — BFS depth-parallel (04-SPEC Section 2.1)
// Replaces: engine_fast.py WavefrontEngine
// ---------------------------------------------------------------------------

/// Breadth-first, depth-parallel spreading activation.
/// All active nodes at current depth fire simultaneously.
/// Signal accumulated via scatter-max into next depth's buffer.
pub struct WavefrontEngine;

impl WavefrontEngine {
    pub fn new() -> Self {
        Self
    }
}

impl ActivationEngine for WavefrontEngine {
    fn propagate(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        config: &PropagationConfig,
    ) -> M1ndResult<DimensionResult> {
        let start = Instant::now();
        let n = graph.num_nodes() as usize;
        if n == 0 || seeds.is_empty() {
            return Ok(DimensionResult {
                scores: Vec::new(),
                dimension: Dimension::Structural,
                elapsed_ns: start.elapsed().as_nanos() as u64,
            });
        }

        let threshold = config.threshold.get();
        let decay = config.decay.get();
        let max_depth = config.max_depth.min(20) as usize; // FM-ACT-012 cap

        // Dense activation buffer (scatter-max target)
        let mut activation = vec![0.0f32; n];
        let mut visited = vec![false; n];

        // Init seeds
        let mut frontier: Vec<NodeId> = Vec::new();
        for &(node, score) in seeds {
            let idx = node.as_usize();
            if idx < n {
                let s = score.get().min(config.saturation_cap.get());
                if s > activation[idx] {
                    activation[idx] = s;
                }
                if !visited[idx] {
                    frontier.push(node);
                    visited[idx] = true;
                }
            }
        }

        // BFS by depth
        for _depth in 0..max_depth {
            if frontier.is_empty() {
                break;
            }
            let mut next_frontier: Vec<NodeId> = Vec::new();

            for &src in &frontier {
                let src_act = activation[src.as_usize()];
                if src_act < threshold {
                    continue;
                }

                let range = graph.csr.out_range(src);
                for j in range {
                    let tgt = graph.csr.targets[j];
                    let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                    let is_inhib = graph.csr.inhibitory[j];

                    let mut signal = src_act * w * decay;
                    if is_inhib {
                        // DEC-054: capped proportional suppression
                        signal = -signal * config.inhibitory_factor.get();
                    }

                    let tgt_idx = tgt.as_usize();
                    if tgt_idx >= n {
                        continue;
                    }

                    if !is_inhib && signal > threshold {
                        // Scatter-max: keep strongest arrival
                        if signal > activation[tgt_idx] {
                            activation[tgt_idx] = signal;
                        }
                        if !visited[tgt_idx] {
                            visited[tgt_idx] = true;
                            next_frontier.push(tgt);
                        }
                    } else if is_inhib {
                        // Inhibitory: subtract (but floor at 0)
                        activation[tgt_idx] = (activation[tgt_idx] + signal).max(0.0);
                    }
                }
            }

            frontier = next_frontier;
        }

        // Collect non-zero activations
        let mut scores: Vec<(NodeId, FiniteF32)> = activation
            .iter()
            .enumerate()
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, &v)| (NodeId::new(i as u32), FiniteF32::new(v)))
            .collect();
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(DimensionResult {
            scores,
            dimension: Dimension::Structural,
            elapsed_ns: start.elapsed().as_nanos() as u64,
        })
    }
}

// ---------------------------------------------------------------------------
// HeapEngine — priority-queue activation (04-SPEC Section 2.2)
// Replaces: engine_fast.py HeapActivationEngine
// ---------------------------------------------------------------------------

/// Entry in the max-heap for HeapEngine.
#[derive(Clone, Copy)]
struct HeapEntry {
    node: NodeId,
    activation: f32,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.activation == other.activation
    }
}
impl Eq for HeapEntry {}
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.activation.total_cmp(&other.activation)
    }
}

/// Max-heap priority queue propagation. Processes strongest signal first.
/// Early-terminates when heap top drops below threshold.
/// Uses BloomFilter for fast visited checks.
pub struct HeapEngine;

impl HeapEngine {
    pub fn new() -> Self {
        Self
    }
}

impl ActivationEngine for HeapEngine {
    fn propagate(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        config: &PropagationConfig,
    ) -> M1ndResult<DimensionResult> {
        let start = Instant::now();
        let n = graph.num_nodes() as usize;
        if n == 0 || seeds.is_empty() {
            return Ok(DimensionResult {
                scores: Vec::new(),
                dimension: Dimension::Structural,
                elapsed_ns: start.elapsed().as_nanos() as u64,
            });
        }

        let threshold = config.threshold.get();
        let decay = config.decay.get();

        let mut activation = vec![0.0f32; n];
        let mut bloom = BloomFilter::with_capacity(n, 0.01);
        let mut heap = BinaryHeap::new();

        // Init seeds
        for &(node, score) in seeds {
            let idx = node.as_usize();
            if idx < n {
                let s = score.get().min(config.saturation_cap.get());
                activation[idx] = s;
                heap.push(HeapEntry {
                    node,
                    activation: s,
                });
                bloom.insert(node);
            }
        }

        let mut depth_counter = 0u32;
        let max_ops = (n as u32)
            .saturating_mul(config.max_depth as u32)
            .max(10000);

        while let Some(entry) = heap.pop() {
            if entry.activation < threshold {
                break; // Early termination
            }
            depth_counter += 1;
            if depth_counter > max_ops {
                break;
            }

            let src = entry.node;
            let src_act = activation[src.as_usize()];

            let range = graph.csr.out_range(src);
            for j in range {
                let tgt = graph.csr.targets[j];
                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                let is_inhib = graph.csr.inhibitory[j];

                let mut signal = src_act * w * decay;
                if is_inhib {
                    signal = -signal * config.inhibitory_factor.get();
                }

                let tgt_idx = tgt.as_usize();
                if tgt_idx >= n {
                    continue;
                }

                if !is_inhib && signal > threshold && signal > activation[tgt_idx] {
                    activation[tgt_idx] = signal;
                    if !bloom.probably_contains(tgt) {
                        bloom.insert(tgt);
                        heap.push(HeapEntry {
                            node: tgt,
                            activation: signal,
                        });
                    }
                } else if is_inhib {
                    activation[tgt_idx] = (activation[tgt_idx] + signal).max(0.0);
                }
            }
        }

        // Collect results
        let mut scores: Vec<(NodeId, FiniteF32)> = activation
            .iter()
            .enumerate()
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, &v)| (NodeId::new(i as u32), FiniteF32::new(v)))
            .collect();
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(DimensionResult {
            scores,
            dimension: Dimension::Structural,
            elapsed_ns: start.elapsed().as_nanos() as u64,
        })
    }
}

// ---------------------------------------------------------------------------
// HybridEngine — auto-select wavefront vs heap (04-SPEC Section 2.2)
// Replaces: engine_fast.py HybridEngine
// Heuristic: seed_ratio < 0.001 AND avg_degree < 8 -> heap, else wavefront
// ---------------------------------------------------------------------------

/// Selects HeapEngine or WavefrontEngine at runtime based on graph topology.
pub struct HybridEngine {
    wavefront: WavefrontEngine,
    heap: HeapEngine,
}

impl HybridEngine {
    pub fn new() -> Self {
        Self {
            wavefront: WavefrontEngine::new(),
            heap: HeapEngine::new(),
        }
    }

    /// Returns true if heap is preferred for these parameters.
    /// Heuristic from engine_fast.py: seed_ratio < 0.001 AND avg_degree < 8.
    fn prefer_heap(graph: &Graph, seed_count: usize) -> bool {
        let seed_ratio = seed_count as f64 / graph.num_nodes().max(1) as f64;
        seed_ratio < 0.001 && graph.avg_degree() < 8.0
    }
}

impl ActivationEngine for HybridEngine {
    fn propagate(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        config: &PropagationConfig,
    ) -> M1ndResult<DimensionResult> {
        if Self::prefer_heap(graph, seeds.len()) {
            self.heap.propagate(graph, seeds, config)
        } else {
            self.wavefront.propagate(graph, seeds, config)
        }
    }
}

// ---------------------------------------------------------------------------
// D2 Semantic dimension wrapper
// Replaces: engine_v2.py D2_Semantic.activate()
// ---------------------------------------------------------------------------

/// Semantic dimension scoring. Delegates to SemanticEngine.
/// Returns DimensionResult with Dimension::Semantic.
pub fn activate_semantic(
    _graph: &Graph,
    semantic: &crate::semantic::SemanticEngine,
    query: &str,
    top_k: usize,
) -> M1ndResult<DimensionResult> {
    let start = Instant::now();
    let scores = semantic.query_fast(_graph, query, top_k)?;
    Ok(DimensionResult {
        scores,
        dimension: Dimension::Semantic,
        elapsed_ns: start.elapsed().as_nanos() as u64,
    })
}

// ---------------------------------------------------------------------------
// D3 Temporal dimension wrapper
// Replaces: engine_v2.py D3_Temporal.activate()
// ---------------------------------------------------------------------------

/// Temporal dimension scoring. Combines recency * weight + frequency * weight.
/// Replaces: engine_v2.py D3_Temporal.activate()
pub fn activate_temporal(
    graph: &Graph,
    seeds: &[(NodeId, FiniteF32)],
    weights: &TemporalWeights,
) -> M1ndResult<DimensionResult> {
    let start = Instant::now();
    let n = graph.num_nodes() as usize;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let half_life_secs = 168.0 * 3600.0; // 7 days in seconds
    let k = 0.693_147_2f64 / half_life_secs;

    let mut scores = Vec::new();
    for &(node, seed_strength) in seeds {
        let idx = node.as_usize();
        if idx >= n {
            continue;
        }
        let last_mod = graph.nodes.last_modified[idx];
        let age_secs = (now - last_mod).max(0.0);
        let recency = (-k * age_secs).exp() as f32;
        let frequency = graph.nodes.change_frequency[idx].get();

        let score = recency * weights.recency.get() + frequency * weights.frequency.get();
        let combined = score * seed_strength.get();
        if combined > 0.0 {
            scores.push((node, FiniteF32::new(combined)));
        }
    }
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(DimensionResult {
        scores,
        dimension: Dimension::Temporal,
        elapsed_ns: start.elapsed().as_nanos() as u64,
    })
}

// ---------------------------------------------------------------------------
// D4 Causal dimension wrapper
// Replaces: engine_v2.py D4_Causal.activate()
// ---------------------------------------------------------------------------

/// Causal dimension scoring. Forward causal + backward causal * 0.7.
/// Replaces: engine_v2.py D4_Causal.activate()
pub fn activate_causal(
    graph: &Graph,
    seeds: &[(NodeId, FiniteF32)],
    config: &PropagationConfig,
) -> M1ndResult<DimensionResult> {
    let start = Instant::now();
    let n = graph.num_nodes() as usize;
    if n == 0 || seeds.is_empty() {
        return Ok(DimensionResult {
            scores: Vec::new(),
            dimension: Dimension::Causal,
            elapsed_ns: start.elapsed().as_nanos() as u64,
        });
    }

    let threshold = config.threshold.get();
    let decay = config.decay.get();
    let max_depth = config.max_depth.min(20) as usize;

    // Forward causal propagation: only follow edges with causal_strength > 0
    let mut activation = vec![0.0f32; n];
    let mut frontier: Vec<NodeId> = Vec::new();
    let mut visited = vec![false; n];

    for &(node, score) in seeds {
        let idx = node.as_usize();
        if idx < n {
            activation[idx] = score.get();
            if !visited[idx] {
                frontier.push(node);
                visited[idx] = true;
            }
        }
    }

    for _depth in 0..max_depth {
        if frontier.is_empty() {
            break;
        }
        let mut next_frontier = Vec::new();
        for &src in &frontier {
            let src_act = activation[src.as_usize()];
            if src_act < threshold {
                continue;
            }
            let range = graph.csr.out_range(src);
            for j in range {
                let causal = graph.csr.causal_strengths[j].get();
                if causal <= 0.0 {
                    continue; // Skip non-causal edges
                }
                let tgt = graph.csr.targets[j];
                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                let signal = src_act * w * causal * decay;
                let tgt_idx = tgt.as_usize();
                if tgt_idx < n && signal > threshold && signal > activation[tgt_idx] {
                    activation[tgt_idx] = signal;
                    if !visited[tgt_idx] {
                        visited[tgt_idx] = true;
                        next_frontier.push(tgt);
                    }
                }
            }
        }
        frontier = next_frontier;
    }

    // Backward causal (reverse CSR) with 0.7 multiplier
    let mut back_frontier: Vec<NodeId> = Vec::new();
    let mut back_visited = vec![false; n];
    for &(node, _) in seeds {
        let idx = node.as_usize();
        if idx < n && !back_visited[idx] {
            back_frontier.push(node);
            back_visited[idx] = true;
        }
    }

    for _depth in 0..max_depth {
        if back_frontier.is_empty() {
            break;
        }
        let mut next = Vec::new();
        for &src in &back_frontier {
            let src_act = activation[src.as_usize()]
                .max(seeds.iter().find(|s| s.0 == src).map_or(0.0, |s| s.1.get()));
            if src_act < threshold {
                continue;
            }
            let range = graph.csr.in_range(src);
            for j in range {
                let fwd_idx = graph.csr.rev_edge_idx[j];
                let causal = graph.csr.causal_strengths[fwd_idx.as_usize()].get();
                if causal <= 0.0 {
                    continue;
                }
                let rev_src = graph.csr.rev_sources[j];
                let w = graph.csr.read_weight(fwd_idx).get();
                let signal = src_act * w * causal * decay * 0.7; // backward discount
                let idx = rev_src.as_usize();
                if idx < n && signal > threshold && signal > activation[idx] {
                    activation[idx] = signal;
                    if !back_visited[idx] {
                        back_visited[idx] = true;
                        next.push(rev_src);
                    }
                }
            }
        }
        back_frontier = next;
    }

    let mut scores: Vec<(NodeId, FiniteF32)> = activation
        .iter()
        .enumerate()
        .filter(|(_, &v)| v > 0.0)
        .map(|(i, &v)| (NodeId::new(i as u32), FiniteF32::new(v)))
        .collect();
    scores.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(DimensionResult {
        scores,
        dimension: Dimension::Causal,
        elapsed_ns: start.elapsed().as_nanos() as u64,
    })
}

// ---------------------------------------------------------------------------
// Dimension merger — combine 4 dimensions + resonance bonus
// Replaces: engine_v2.py ConnectomeEngine._merge_dimensions()
// FM-ACT-001 FIX: check 4-dim BEFORE 3-dim (dead elif in Python)
// ---------------------------------------------------------------------------

/// PageRank boost factor.
const PAGERANK_BOOST: f32 = 0.1;
/// Minimum score threshold for dimension contribution.
const DIM_CONTRIBUTION_THRESHOLD: f32 = 0.01;

/// Merge four DimensionResults into a single ranked ActivationResult.
/// Applies DIMENSION_WEIGHTS, resonance bonus (FM-ACT-001 fix: 4-dim checked first).
pub fn merge_dimensions(
    results: &[DimensionResult; 4],
    top_k: usize,
) -> M1ndResult<ActivationResult> {
    let start = Instant::now();

    // Compute adaptive weights: if a dimension is empty, redistribute
    let mut weights = DIMENSION_WEIGHTS;
    let mut total_active_weight = 0.0f32;
    let mut active_mask = [false; 4];
    for (i, r) in results.iter().enumerate() {
        if !r.scores.is_empty() {
            active_mask[i] = true;
            total_active_weight += weights[i];
        }
    }

    // DEC-049: adaptive redistribution
    if total_active_weight > 0.0 && total_active_weight < 1.0 {
        for i in 0..4 {
            if active_mask[i] {
                weights[i] /= total_active_weight;
            } else {
                weights[i] = 0.0;
            }
        }
    }

    // Merge into per-node combined scores
    let mut node_scores: HashMap<u32, [f32; 4]> = HashMap::new();
    for (dim_idx, result) in results.iter().enumerate() {
        for &(node, score) in &result.scores {
            let entry = node_scores.entry(node.0).or_insert([0.0; 4]);
            entry[dim_idx] = score.get();
        }
    }

    // Build activated nodes
    let mut activated: Vec<ActivatedNode> = node_scores
        .iter()
        .map(|(&node_id, dims)| {
            // Weighted sum
            let mut combined = 0.0f32;
            let mut dim_count = 0u8;
            for i in 0..4 {
                combined += dims[i] * weights[i];
                if dims[i] > DIM_CONTRIBUTION_THRESHOLD {
                    dim_count += 1;
                }
            }

            // FM-ACT-001 FIX: check 4-dim BEFORE 3-dim (dead elif in Python)
            if dim_count >= 4 {
                combined *= RESONANCE_BONUS_4DIM;
            } else if dim_count >= 3 {
                combined *= RESONANCE_BONUS_3DIM;
            }

            ActivatedNode {
                node: NodeId::new(node_id),
                activation: FiniteF32::new(combined),
                dimensions: [
                    FiniteF32::new(dims[0]),
                    FiniteF32::new(dims[1]),
                    FiniteF32::new(dims[2]),
                    FiniteF32::new(dims[3]),
                ],
                active_dimension_count: dim_count,
            }
        })
        .collect();

    // Sort descending, truncate to top_k
    activated.sort_by(|a, b| b.activation.cmp(&a.activation));
    activated.truncate(top_k);

    Ok(ActivationResult {
        activated,
        seeds: Vec::new(), // filled by caller
        elapsed_ns: start.elapsed().as_nanos() as u64,
        xlr_fallback_used: false,
    })
}
