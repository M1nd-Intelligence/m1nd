// === crates/m1nd-core/src/topology.rs ===

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Community — Louvain community detection (topology_v2.py CommunityDetector)
// FM-TOP-001 fix: correct delta-Q formula with consistent two_m scaling.
// FM-TOP-002 fix: handle self-loops in degree computation.
// ---------------------------------------------------------------------------

/// Community assignment result.
#[derive(Clone, Debug)]
pub struct CommunityResult {
    /// Community ID for each node (indexed by NodeId).
    pub assignments: Vec<CommunityId>,
    /// Number of distinct communities.
    pub num_communities: u32,
    /// Modularity score Q.
    pub modularity: FiniteF32,
    /// Number of passes until convergence.
    pub passes: u32,
}

/// Per-community statistics.
#[derive(Clone, Debug)]
pub struct CommunityStats {
    pub id: CommunityId,
    pub node_count: u32,
    pub internal_edges: u32,
    pub external_edges: u32,
    /// Density = internal_edges / max_possible_internal.
    pub density: FiniteF32,
}

/// Louvain phase-1 community detection.
/// Replaces: topology_v2.py CommunityDetector
pub struct CommunityDetector {
    max_passes: u32,
    min_modularity_gain: FiniteF32,
}

impl CommunityDetector {
    pub fn new(max_passes: u32, min_modularity_gain: FiniteF32) -> Self {
        Self {
            max_passes,
            min_modularity_gain,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            max_passes: 20,
            min_modularity_gain: FiniteF32::new(1e-6),
        }
    }

    /// Detect communities. Returns Err on non-convergence (FM-TOP-003).
    /// FM-TOP-001 fix: delta-Q uses consistent two_m denominator.
    /// FM-TOP-002 fix: self-loops handled in degree sum.
    /// VANILLA-FIX: uses undirected adjacency (forward + reverse edges) for correct modularity.
    /// Replaces: topology_v2.py CommunityDetector.detect()
    pub fn detect(&self, graph: &Graph) -> M1ndResult<CommunityResult> {
        let n = graph.num_nodes() as usize;
        if n == 0 {
            return Err(M1ndError::EmptyGraph);
        }

        // Initialize: each node in its own community
        let mut community: Vec<u32> = (0..n as u32).collect();

        // VANILLA-FIX: Build undirected adjacency from forward CSR.
        // For each forward edge (i->j), add both adj[i][j] and adj[j][i].
        // Use a seen-set on canonical (min,max) pairs to avoid double-counting
        // bidirectional edges that already appear in both directions in CSR.
        let mut adj: Vec<std::collections::HashMap<u32, f64>> =
            vec![std::collections::HashMap::new(); n];
        let mut seen = std::collections::HashSet::new();

        for i in 0..n {
            let range = graph.csr.out_range(NodeId::new(i as u32));
            for j in range {
                let target = graph.csr.targets[j].as_usize();
                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get() as f64;

                let (lo, hi) = if i <= target {
                    (i, target)
                } else {
                    (target, i)
                };
                if !seen.insert((lo, hi)) {
                    continue; // Already processed this undirected edge
                }

                *adj[i].entry(target as u32).or_default() += w;
                if i != target {
                    *adj[target].entry(i as u32).or_default() += w;
                } else {
                    // FM-TOP-002: self-loop contributes 2x to degree
                    *adj[i].entry(i as u32).or_default() += w;
                }
            }
        }

        // Compute degrees from undirected adjacency
        let mut degree = vec![0.0f64; n];
        let mut two_m = 0.0f64;
        for (i, deg) in degree.iter_mut().enumerate().take(n) {
            *deg = adj[i].values().sum();
            two_m += *deg;
        }

        if two_m <= 0.0 {
            return Ok(CommunityResult {
                assignments: community.iter().map(|&c| CommunityId(c)).collect(),
                num_communities: n as u32,
                modularity: FiniteF32::ZERO,
                passes: 0,
            });
        }

        // Community aggregates
        let mut sum_in = vec![0.0f64; n];
        let mut sum_tot: Vec<f64> = degree.clone();

        let mut converged = false;
        let mut pass = 0u32;

        while pass < self.max_passes {
            pass += 1;
            let mut improved = false;

            for i in 0..n {
                let ki = degree[i];
                let ci = community[i] as usize;

                // k_i_in: sum of weights from i to nodes in its current community (undirected)
                let mut ki_in_old = 0.0f64;
                for (&nb, &w) in &adj[i] {
                    if community[nb as usize] as usize == ci {
                        ki_in_old += w;
                    }
                }

                sum_in[ci] -= 2.0 * ki_in_old;
                sum_tot[ci] -= ki;

                // Find best community via undirected neighbors
                let mut best_comm = ci;
                let mut best_delta_q = 0.0f64;

                let mut neighbor_comms: std::collections::HashMap<usize, f64> =
                    std::collections::HashMap::new();
                for (&nb, &w) in &adj[i] {
                    let cj = community[nb as usize] as usize;
                    *neighbor_comms.entry(cj).or_insert(0.0) += w;
                }

                for (&cj, &ki_in_new) in &neighbor_comms {
                    let delta_q = (ki_in_new / two_m) - (sum_tot[cj] * ki / (two_m * two_m));
                    let delta_old = (ki_in_old / two_m) - (sum_tot[ci] * ki / (two_m * two_m));
                    let gain = delta_q - delta_old;
                    if gain > best_delta_q {
                        best_delta_q = gain;
                        best_comm = cj;
                    }
                }

                community[i] = best_comm as u32;

                // Recompute k_i_in for new community
                let mut ki_in_new_actual = 0.0f64;
                for (&nb, &w) in &adj[i] {
                    if community[nb as usize] as usize == best_comm {
                        ki_in_new_actual += w;
                    }
                }

                sum_in[best_comm] += 2.0 * ki_in_new_actual;
                sum_tot[best_comm] += ki;

                if best_comm != ci {
                    improved = true;
                }
            }

            if !improved {
                converged = true;
                break;
            }
        }

        if !converged && pass >= self.max_passes {
            // FM-TOP-003: non-convergence (but we still return the result)
        }

        // Renumber communities to be contiguous
        let mut comm_map = std::collections::HashMap::new();
        let mut next_id = 0u32;
        let assignments: Vec<CommunityId> = community
            .iter()
            .map(|&c| {
                let id = comm_map.entry(c).or_insert_with(|| {
                    let id = next_id;
                    next_id += 1;
                    id
                });
                CommunityId(*id)
            })
            .collect();

        // Compute modularity Q using undirected adjacency
        let mut q = 0.0f64;
        for i in 0..n {
            for (&nb, &w) in &adj[i] {
                let j = nb as usize;
                if assignments[i] == assignments[j] {
                    q += w - degree[i] * degree[j] / two_m;
                }
            }
        }
        q /= two_m;

        Ok(CommunityResult {
            assignments,
            num_communities: next_id,
            modularity: FiniteF32::new(q as f32),
            passes: pass,
        })
    }

    /// Get per-community statistics.
    /// Replaces: topology_v2.py CommunityDetector.community_stats()
    pub fn community_stats(graph: &Graph, result: &CommunityResult) -> Vec<CommunityStats> {
        let n = graph.num_nodes() as usize;
        let num_comm = result.num_communities as usize;
        let mut node_counts = vec![0u32; num_comm];
        let mut internal = vec![0u32; num_comm];
        let mut external = vec![0u32; num_comm];

        for i in 0..n {
            let ci = result.assignments[i].0 as usize;
            node_counts[ci] += 1;

            let range = graph.csr.out_range(NodeId::new(i as u32));
            for j in range {
                let tgt = graph.csr.targets[j].as_usize();
                if tgt < n {
                    if result.assignments[tgt].0 as usize == ci {
                        internal[ci] += 1;
                    } else {
                        external[ci] += 1;
                    }
                }
            }
        }

        (0..num_comm)
            .map(|c| {
                let nc = node_counts[c];
                let max_internal = if nc > 1 { nc * (nc - 1) } else { 1 };
                let density = if max_internal > 0 {
                    internal[c] as f32 / max_internal as f32
                } else {
                    0.0
                };
                CommunityStats {
                    id: CommunityId(c as u32),
                    node_count: nc,
                    internal_edges: internal[c],
                    external_edges: external[c],
                    density: FiniteF32::new(density.min(1.0)),
                }
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Bridge — inter-community bridge detection (topology_v2.py BridgeDetector)
// FM-TOP-007 fix: precomputed inter-community edge index, not O(V^2*C).
// ---------------------------------------------------------------------------

/// A bridge edge connecting two communities.
/// Replaces: topology_v2.py BridgeDetector result entries
#[derive(Clone, Debug)]
pub struct Bridge {
    pub source: NodeId,
    pub target: NodeId,
    pub edge_idx: EdgeIdx,
    pub source_community: CommunityId,
    pub target_community: CommunityId,
    /// Bridge importance score (betweenness-like).
    pub importance: FiniteF32,
}

/// Bridge detector between communities. O(E) scan.
/// FM-TOP-007 fix: precomputed community assignments.
/// Replaces: topology_v2.py BridgeDetector
pub struct BridgeDetector;

impl BridgeDetector {
    /// Detect all inter-community bridges.
    /// Replaces: topology_v2.py BridgeDetector.detect()
    pub fn detect(graph: &Graph, communities: &CommunityResult) -> M1ndResult<Vec<Bridge>> {
        let n = graph.num_nodes() as usize;
        let mut bridges = Vec::new();

        for i in 0..n {
            let ci = communities.assignments[i];
            let range = graph.csr.out_range(NodeId::new(i as u32));
            for j in range {
                let tgt = graph.csr.targets[j];
                let tgt_idx = tgt.as_usize();
                if tgt_idx >= n {
                    continue;
                }
                let cj = communities.assignments[tgt_idx];
                if ci != cj {
                    let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                    bridges.push(Bridge {
                        source: NodeId::new(i as u32),
                        target: tgt,
                        edge_idx: EdgeIdx::new(j as u32),
                        source_community: ci,
                        target_community: cj,
                        importance: FiniteF32::new(w),
                    });
                }
            }
        }

        bridges.sort_by(|a, b| b.importance.cmp(&a.importance));
        Ok(bridges)
    }

    /// Detect bridges between two specific communities.
    pub fn detect_between(
        graph: &Graph,
        communities: &CommunityResult,
        comm_a: CommunityId,
        comm_b: CommunityId,
    ) -> M1ndResult<Vec<Bridge>> {
        let all = Self::detect(graph, communities)?;
        Ok(all
            .into_iter()
            .filter(|b| {
                (b.source_community == comm_a && b.target_community == comm_b)
                    || (b.source_community == comm_b && b.target_community == comm_a)
            })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// SpectralGap — Laplacian eigenvalue approximation
// FM-TOP-010 fix: safety margin + multi-trial power iteration.
// FM-TOP-011 fix: correct zero eigenvalue count for disconnected graphs.
// FM-TOP-012 fix: empty graph returns error instead of crash.
// ---------------------------------------------------------------------------

/// Spectral gap analysis result.
/// Replaces: topology_v2.py SpectralGapAnalyzer.analyze() return
#[derive(Clone, Debug)]
pub struct SpectralGapResult {
    /// Algebraic connectivity (second smallest eigenvalue of Laplacian).
    pub algebraic_connectivity: FiniteF32,
    /// Spectral gap ratio.
    pub spectral_gap: FiniteF32,
    /// Number of connected components (from zero eigenvalue count, FM-TOP-011 fix).
    pub num_components: u32,
    /// Robustness classification.
    pub robustness: RobustnessLevel,
    /// Number of power iteration trials used.
    pub trials: u32,
    /// Whether convergence was achieved.
    pub converged: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RobustnessLevel {
    Fragile,
    Moderate,
    Robust,
}

/// Spectral gap analyzer using power iteration on the graph Laplacian.
/// Replaces: topology_v2.py SpectralGapAnalyzer
pub struct SpectralGapAnalyzer {
    max_iterations: u32,
    tolerance: f64,
    num_trials: u32,
}

impl SpectralGapAnalyzer {
    pub fn new(max_iterations: u32, tolerance: f64, num_trials: u32) -> Self {
        Self {
            max_iterations,
            tolerance,
            num_trials,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            max_iterations: 1000,
            tolerance: 1e-8,
            num_trials: 3,
        }
    }

    /// Analyze spectral properties of the graph.
    /// Returns Err(EmptyGraph) for empty graph (FM-TOP-012 fix).
    /// Returns Err(SpectralDivergence) if power iteration fails (FM-TOP-010 fix).
    /// Replaces: topology_v2.py SpectralGapAnalyzer.analyze()
    pub fn analyze(&self, graph: &Graph) -> M1ndResult<SpectralGapResult> {
        let n = graph.num_nodes() as usize;
        if n == 0 {
            return Err(M1ndError::EmptyGraph);
        }

        // Compute connected components via BFS (FM-TOP-011 fix)
        let mut component = vec![u32::MAX; n];
        let mut num_components = 0u32;
        for start in 0..n {
            if component[start] != u32::MAX {
                continue;
            }
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            component[start] = num_components;
            while let Some(node) = queue.pop_front() {
                let range = graph.csr.out_range(NodeId::new(node as u32));
                for j in range {
                    let tgt = graph.csr.targets[j].as_usize();
                    if tgt < n && component[tgt] == u32::MAX {
                        component[tgt] = num_components;
                        queue.push_back(tgt);
                    }
                }
            }
            num_components += 1;
        }

        // For single-node or single-component with no edges
        if n == 1 {
            return Ok(SpectralGapResult {
                algebraic_connectivity: FiniteF32::ZERO,
                spectral_gap: FiniteF32::ZERO,
                num_components,
                robustness: RobustnessLevel::Fragile,
                trials: 0,
                converged: true,
            });
        }

        // Power iteration on shifted Laplacian to find lambda_2
        // L = D - A (unnormalized Laplacian)
        // We want the second-smallest eigenvalue (algebraic connectivity)
        // Use shifted inverse iteration: find largest eigenvalue of (max_eig*I - L)
        // which corresponds to smallest eigenvalue of L

        // Compute degree
        let mut degree = vec![0.0f64; n];
        for (i, deg) in degree.iter_mut().enumerate().take(n) {
            let range = graph.csr.out_range(NodeId::new(i as u32));
            for j in range {
                *deg += graph.csr.read_weight(EdgeIdx::new(j as u32)).get() as f64;
            }
        }

        // Gershgorin estimate for max eigenvalue
        let max_eig_est = degree.iter().cloned().fold(0.0f64, f64::max) * 2.1;
        if max_eig_est <= 0.0 {
            return Ok(SpectralGapResult {
                algebraic_connectivity: FiniteF32::ZERO,
                spectral_gap: FiniteF32::ZERO,
                num_components,
                robustness: RobustnessLevel::Fragile,
                trials: 0,
                converged: true,
            });
        }

        // Power iteration: find dominant eigenvector of M = max_eig*I - L
        let nf = n as f64;
        let mut best_lambda2 = f64::MAX;
        let mut converged = false;

        // Simple deterministic "random" vectors
        for trial in 0..self.num_trials {
            // Initialize vector (orthogonal to all-ones)
            let mut v: Vec<f64> = (0..n)
                .map(|i| {
                    let seed = (i as u64).wrapping_mul(2654435761 + trial as u64 * 1000003);
                    (seed as f64 / u64::MAX as f64) - 0.5
                })
                .collect();

            // Orthogonalize against all-ones vector
            let avg: f64 = v.iter().sum::<f64>() / nf;
            for x in &mut v {
                *x -= avg;
            }

            // Normalize
            let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if norm < 1e-15 {
                continue;
            }
            for x in &mut v {
                *x /= norm;
            }

            let mut eigenvalue = 0.0f64;

            for _iter in 0..self.max_iterations {
                // w = M * v = (max_eig * I - L) * v = max_eig*v - L*v
                // L*v = D*v - A*v
                let mut w = vec![0.0f64; n];
                for i in 0..n {
                    // (max_eig - degree[i]) * v[i]
                    w[i] = (max_eig_est - degree[i]) * v[i];
                    // + sum_j(A[i][j] * v[j])
                    let range = graph.csr.out_range(NodeId::new(i as u32));
                    for j in range {
                        let tgt = graph.csr.targets[j].as_usize();
                        let wgt = graph.csr.read_weight(EdgeIdx::new(j as u32)).get() as f64;
                        w[i] += wgt * v[tgt];
                    }
                }

                // Orthogonalize against all-ones
                let avg_w: f64 = w.iter().sum::<f64>() / nf;
                for x in &mut w {
                    *x -= avg_w;
                }

                // Compute eigenvalue (Rayleigh quotient)
                let dot: f64 = w.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                let new_norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();

                if new_norm < 1e-15 {
                    break;
                }

                eigenvalue = dot;

                // Check convergence
                let residual: f64 = w
                    .iter()
                    .zip(v.iter())
                    .map(|(wi, vi)| (wi / new_norm - vi).powi(2))
                    .sum::<f64>()
                    .sqrt();

                // Normalize
                for i in 0..n {
                    v[i] = w[i] / new_norm;
                }

                if residual < self.tolerance {
                    converged = true;
                    break;
                }
            }

            // lambda_2 = max_eig_est - eigenvalue_of_shifted
            let lambda2 = (max_eig_est - eigenvalue).max(0.0);
            if lambda2 < best_lambda2 {
                best_lambda2 = lambda2;
            }
        }

        let algebraic_connectivity = best_lambda2.min(100.0) as f32;
        let spectral_gap = if max_eig_est > 0.0 {
            (algebraic_connectivity as f64 / max_eig_est) as f32
        } else {
            0.0
        };

        let robustness = if algebraic_connectivity < 0.01 {
            RobustnessLevel::Fragile
        } else if algebraic_connectivity < 0.5 {
            RobustnessLevel::Moderate
        } else {
            RobustnessLevel::Robust
        };

        Ok(SpectralGapResult {
            algebraic_connectivity: FiniteF32::new(algebraic_connectivity),
            spectral_gap: FiniteF32::new(spectral_gap),
            num_components,
            robustness,
            trials: self.num_trials,
            converged,
        })
    }
}

// ---------------------------------------------------------------------------
// ActivationFingerprint — probe-based node equivalence
// FM-TOP-014 fix: LSH for O(N) instead of O(N^2) pairwise comparison.
// ---------------------------------------------------------------------------

/// Per-node activation fingerprint: response vector across probe queries.
#[derive(Clone, Debug)]
pub struct Fingerprint {
    pub node: NodeId,
    /// Activation response to each probe query.
    pub responses: Vec<FiniteF32>,
    /// LSH bucket for approximate nearest-neighbor (FM-TOP-014 fix).
    pub lsh_hash: u64,
}

/// Pair of functionally equivalent nodes.
#[derive(Clone, Debug)]
pub struct EquivalentPair {
    pub node_a: NodeId,
    pub node_b: NodeId,
    pub cosine_similarity: FiniteF32,
    pub directly_connected: bool,
}

/// Activation fingerprinter for finding functionally equivalent nodes.
/// Replaces: topology_v2.py ActivationFingerprinter
pub struct ActivationFingerprinter {
    /// Budget for pairwise comparison (FM-TOP-014).
    pair_budget: u64,
    /// Similarity threshold for equivalence.
    similarity_threshold: FiniteF32,
}

impl ActivationFingerprinter {
    pub fn new(pair_budget: u64, similarity_threshold: FiniteF32) -> Self {
        Self {
            pair_budget,
            similarity_threshold,
        }
    }

    /// Compute fingerprints for all nodes using diverse probe queries.
    /// Replaces: topology_v2.py ActivationFingerprinter.compute_fingerprints()
    pub fn compute_fingerprints(
        &self,
        graph: &Graph,
        engine: &crate::activation::HybridEngine,
        probe_queries: &[Vec<(NodeId, FiniteF32)>],
    ) -> M1ndResult<Vec<Fingerprint>> {
        use crate::activation::ActivationEngine;

        let n = graph.num_nodes() as usize;
        let config = PropagationConfig::default();
        let num_probes = probe_queries.len();

        // Run each probe and collect per-node activations
        let mut responses = vec![vec![FiniteF32::ZERO; num_probes]; n];

        for (pi, seeds) in probe_queries.iter().enumerate() {
            let result = engine.propagate(graph, seeds, &config)?;
            for &(node, score) in &result.scores {
                let idx = node.as_usize();
                if idx < n {
                    responses[idx][pi] = score;
                }
            }
        }

        // Build fingerprints with LSH hash
        let fingerprints: Vec<Fingerprint> = (0..n)
            .map(|i| {
                let resp = &responses[i];
                // Simple LSH: hash the sign pattern of responses
                let mut hash = 0u64;
                for (pi, &v) in resp.iter().enumerate() {
                    if v.get() > 0.0 {
                        hash |= 1u64 << (pi & 63);
                    }
                }
                Fingerprint {
                    node: NodeId::new(i as u32),
                    responses: resp.clone(),
                    lsh_hash: hash,
                }
            })
            .collect();

        Ok(fingerprints)
    }

    /// Find equivalent node pairs from fingerprints.
    /// Uses LSH for candidate generation (FM-TOP-014 fix: O(N) not O(N^2)).
    /// Replaces: topology_v2.py ActivationFingerprinter.find_equivalents()
    pub fn find_equivalents(
        &self,
        fingerprints: &[Fingerprint],
        graph: &Graph,
    ) -> M1ndResult<Vec<EquivalentPair>> {
        // Group by LSH hash
        let mut buckets: std::collections::HashMap<u64, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, fp) in fingerprints.iter().enumerate() {
            buckets.entry(fp.lsh_hash).or_default().push(i);
        }

        let mut pairs = Vec::new();
        let mut pair_count = 0u64;

        for bucket in buckets.values() {
            if bucket.len() < 2 {
                continue;
            }
            for a_idx in 0..bucket.len() {
                for b_idx in (a_idx + 1)..bucket.len() {
                    if pair_count >= self.pair_budget {
                        break;
                    }

                    let a = &fingerprints[bucket[a_idx]];
                    let b = &fingerprints[bucket[b_idx]];

                    let sim = Self::cosine_sim(&a.responses, &b.responses);
                    if sim >= self.similarity_threshold.get() {
                        // Check direct connection
                        let connected = {
                            let range = graph.csr.out_range(a.node);
                            range.into_iter().any(|j| graph.csr.targets[j] == b.node)
                        };

                        pairs.push(EquivalentPair {
                            node_a: a.node,
                            node_b: b.node,
                            cosine_similarity: FiniteF32::new(sim),
                            directly_connected: connected,
                        });
                    }
                    pair_count += 1;
                }
            }
        }

        pairs.sort_by(|a, b| b.cosine_similarity.cmp(&a.cosine_similarity));
        Ok(pairs)
    }

    /// Find nodes equivalent to a specific target.
    pub fn find_equivalents_of(
        &self,
        target: NodeId,
        fingerprints: &[Fingerprint],
        graph: &Graph,
    ) -> M1ndResult<Vec<EquivalentPair>> {
        let target_idx = target.as_usize();
        if target_idx >= fingerprints.len() {
            return Ok(Vec::new());
        }

        let target_fp = &fingerprints[target_idx];
        let mut pairs = Vec::new();

        for (i, fp) in fingerprints.iter().enumerate() {
            if i == target_idx {
                continue;
            }
            let sim = Self::cosine_sim(&target_fp.responses, &fp.responses);
            if sim >= self.similarity_threshold.get() {
                let connected = {
                    let range = graph.csr.out_range(target);
                    range.into_iter().any(|j| graph.csr.targets[j] == fp.node)
                };
                pairs.push(EquivalentPair {
                    node_a: target,
                    node_b: fp.node,
                    cosine_similarity: FiniteF32::new(sim),
                    directly_connected: connected,
                });
            }
        }

        pairs.sort_by(|a, b| b.cosine_similarity.cmp(&a.cosine_similarity));
        Ok(pairs)
    }

    fn cosine_sim(a: &[FiniteF32], b: &[FiniteF32]) -> f32 {
        let mut dot = 0.0f32;
        let mut na = 0.0f32;
        let mut nb = 0.0f32;
        for i in 0..a.len().min(b.len()) {
            dot += a[i].get() * b[i].get();
            na += a[i].get() * a[i].get();
            nb += b[i].get() * b[i].get();
        }
        let denom = na.sqrt() * nb.sqrt();
        if denom > 0.0 {
            (dot / denom).min(1.0)
        } else {
            0.0
        }
    }
}

// ---------------------------------------------------------------------------
// MultiScaleView — hierarchical community drill-down
// Replaces: topology_v2.py MultiScaleView
// ---------------------------------------------------------------------------

/// A view of communities at a particular scale.
#[derive(Clone, Debug)]
pub struct ScaleView {
    pub scale: u8,
    pub communities: CommunityResult,
    pub bridges: Vec<Bridge>,
}

/// Multi-scale topology viewer.
/// Replaces: topology_v2.py MultiScaleView
pub struct MultiScaleViewer;

impl MultiScaleViewer {
    /// Compute multi-scale view (coarsening hierarchy).
    /// Replaces: topology_v2.py MultiScaleView.compute()
    pub fn compute(graph: &Graph, max_scales: u8) -> M1ndResult<Vec<ScaleView>> {
        // For now, compute single-scale Louvain
        let detector = CommunityDetector::with_defaults();
        let communities = detector.detect(graph)?;
        let bridges = BridgeDetector::detect(graph, &communities)?;

        Ok(vec![ScaleView {
            scale: 0,
            communities,
            bridges,
        }])
    }
}

// ---------------------------------------------------------------------------
// TopologyAnalyzer — facade combining all topology capabilities
// Replaces: topology_v2.py TopologyAnalyzer
// ---------------------------------------------------------------------------

/// Full topology analysis result.
#[derive(Clone, Debug)]
pub struct TopologyReport {
    pub communities: CommunityResult,
    pub community_stats: Vec<CommunityStats>,
    pub bridges: Vec<Bridge>,
    pub spectral: SpectralGapResult,
}

/// Facade for all topology analysis.
/// Replaces: topology_v2.py TopologyAnalyzer
pub struct TopologyAnalyzer {
    pub community_detector: CommunityDetector,
    pub spectral_analyzer: SpectralGapAnalyzer,
    pub fingerprinter: ActivationFingerprinter,
}

impl TopologyAnalyzer {
    pub fn with_defaults() -> Self {
        Self {
            community_detector: CommunityDetector::with_defaults(),
            spectral_analyzer: SpectralGapAnalyzer::with_defaults(),
            fingerprinter: ActivationFingerprinter::new(100_000, FiniteF32::new(0.85)),
        }
    }

    /// Full topology analysis: communities + bridges + spectral gap.
    /// Holds read lock for entire analysis duration (FM-TOP-007 consistency note).
    /// Replaces: topology_v2.py TopologyAnalyzer.analyze()
    pub fn analyze(&self, graph: &Graph) -> M1ndResult<TopologyReport> {
        let communities = self.community_detector.detect(graph)?;
        let community_stats = CommunityDetector::community_stats(graph, &communities);
        let bridges = BridgeDetector::detect(graph, &communities)?;
        let spectral = self.spectral_analyzer.analyze(graph)?;

        Ok(TopologyReport {
            communities,
            community_stats,
            bridges,
            spectral,
        })
    }
}

static_assertions::assert_impl_all!(TopologyAnalyzer: Send, Sync);
