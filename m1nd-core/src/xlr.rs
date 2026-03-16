// === crates/m1nd-core/src/xlr.rs ===

use std::collections::VecDeque;

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants from xlr_v2.py
// ---------------------------------------------------------------------------

/// Hot signal frequency (xlr_v2.py F_HOT = 1.0).
pub const F_HOT: f32 = 1.0;
/// Cold signal frequency (xlr_v2.py F_COLD = 3.7).
pub const F_COLD: f32 = 3.7;
/// Spectral overlap Gaussian kernel bandwidth (xlr_v2.py bw=0.8).
pub const SPECTRAL_BANDWIDTH: f32 = 0.8;
/// Default immunity distance in hops (xlr_v2.py 2-hop BFS).
pub const IMMUNITY_HOPS: u8 = 2;
/// Sigmoid steepness for gating (xlr_v2.py * 6.0).
pub const SIGMOID_STEEPNESS: f32 = 6.0;
/// Number of spectral buckets for overlap (DEC-003).
pub const SPECTRAL_BUCKETS: usize = 20;
/// Density clamp floor.
pub const DENSITY_FLOOR: f32 = 0.3;
/// Density clamp cap.
pub const DENSITY_CAP: f32 = 2.0;
/// Inhibitory cold attenuation (DEC-010).
pub const INHIBITORY_COLD_ATTENUATION: f32 = 0.5;

// ---------------------------------------------------------------------------
// SpectralPulse — per-node pulse (xlr_v2.py SpectralPulse)
// ---------------------------------------------------------------------------

/// A spectral pulse carrying amplitude, phase, and frequency.
/// 48 bytes — fits in one cache line.
/// Replaces: xlr_v2.py SpectralPulse dataclass
#[derive(Clone, Copy, Debug)]
pub struct SpectralPulse {
    pub node: NodeId,
    pub amplitude: FiniteF32,
    /// Phase in [0, 2*pi).
    pub phase: FiniteF32,
    /// Frequency: F_HOT for seeds, F_COLD for anti-seeds.
    pub frequency: PosF32,
    /// Hops from origin (for immunity check).
    pub hops: u8,
    /// Previous node (for path tracking, replaces unbounded Vec — FM-RES-007).
    pub prev_node: NodeId,
    /// Recent path (last 3 nodes) — bounded, replaces full path Vec.
    pub recent_path: [NodeId; 3],
}

// ---------------------------------------------------------------------------
// SpectralWaveBuffer — per-node accumulation (xlr_v2.py SpectralWaveBuffer)
// ---------------------------------------------------------------------------

/// Accumulated spectral energy at a node.
/// Replaces: xlr_v2.py SpectralWaveBuffer
#[derive(Clone, Debug, Default)]
pub struct SpectralWaveBuffer {
    /// Hot signal accumulated amplitudes.
    pub hot_amplitudes: Vec<FiniteF32>,
    /// Hot signal accumulated frequencies.
    pub hot_frequencies: Vec<FiniteF32>,
    /// Cold signal accumulated amplitudes.
    pub cold_amplitudes: Vec<FiniteF32>,
    /// Cold signal accumulated frequencies.
    pub cold_frequencies: Vec<FiniteF32>,
}

// ---------------------------------------------------------------------------
// XlrParams — configuration
// ---------------------------------------------------------------------------

/// XLR engine configuration.
/// Replaces: xlr_v2.py AdaptiveXLREngine.__init__ parameters
pub struct XlrParams {
    /// Number of anti-seeds to pick. Default: 3.
    pub num_anti_seeds: usize,
    /// Immunity hop distance from seeds. Default: 2 (FM-XLR-008 fix: BFS-based, not count-based).
    pub immunity_hops: u8,
    /// Minimum degree ratio for anti-seed candidates. Default: 0.3.
    pub min_degree_ratio: FiniteF32,
    /// Maximum Jaccard similarity between seed and anti-seed neighborhoods. Default: 0.2.
    pub max_jaccard_similarity: FiniteF32,
    /// Density adaptive clamp range. Default: [0.3, 2.0].
    pub density_clamp_min: FiniteF32,
    pub density_clamp_max: FiniteF32,
    /// Pulse propagation budget (FM-RES-004). Default: 50_000.
    pub pulse_budget: u64,
}

impl Default for XlrParams {
    fn default() -> Self {
        Self {
            num_anti_seeds: 3,
            immunity_hops: IMMUNITY_HOPS,
            min_degree_ratio: FiniteF32::new(0.3),
            max_jaccard_similarity: FiniteF32::new(0.2),
            density_clamp_min: FiniteF32::new(0.3),
            density_clamp_max: FiniteF32::new(2.0),
            pulse_budget: 50_000,
        }
    }
}

// ---------------------------------------------------------------------------
// XlrResult — output of XLR pipeline
// ---------------------------------------------------------------------------

/// Result of XLR adaptive noise cancellation.
/// Replaces: xlr_v2.py AdaptiveXLREngine.query() return
#[derive(Clone, Debug)]
pub struct XlrResult {
    /// Per-node activation after spectral cancellation + sigmoid gating.
    pub activations: Vec<(NodeId, FiniteF32)>,
    /// Anti-seed nodes that were selected.
    pub anti_seeds: Vec<NodeId>,
    /// Whether over-cancellation fallback was triggered (FM-XLR-010).
    pub fallback_to_hot_only: bool,
    /// Pulses processed (for budget monitoring).
    pub pulses_processed: u64,
}

// ---------------------------------------------------------------------------
// AdaptiveXlrEngine — main engine (xlr_v2.py AdaptiveXLREngine)
// ---------------------------------------------------------------------------

/// Adaptive XLR noise cancellation engine.
/// Dual propagation: hot from seeds, cold from anti-seeds.
/// Spectral overlap modulation, density-adaptive strength, sigmoid gating.
/// Replaces: xlr_v2.py AdaptiveXLREngine
pub struct AdaptiveXlrEngine {
    params: XlrParams,
}

impl AdaptiveXlrEngine {
    pub fn new(params: XlrParams) -> Self {
        Self { params }
    }

    pub fn with_defaults() -> Self {
        Self::new(XlrParams::default())
    }

    /// Run full XLR pipeline on a set of seed nodes.
    /// Steps: pick anti-seeds -> compute immunity -> propagate hot -> propagate cold
    ///        -> spectral overlap -> density modulation -> sigmoid gating -> rescale.
    /// Replaces: xlr_v2.py AdaptiveXLREngine.query()
    pub fn query(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        config: &PropagationConfig,
    ) -> M1ndResult<XlrResult> {
        let n = graph.num_nodes() as usize;
        if n == 0 || seeds.is_empty() {
            return Ok(XlrResult {
                activations: Vec::new(),
                anti_seeds: Vec::new(),
                fallback_to_hot_only: false,
                pulses_processed: 0,
            });
        }

        let seed_nodes: Vec<NodeId> = seeds.iter().map(|s| s.0).collect();

        // Step 1: Pick anti-seeds
        let anti_seeds = self.pick_anti_seeds(graph, &seed_nodes)?;

        // Step 2: Compute immunity
        let immunity = self.compute_immunity(graph, &seed_nodes)?;

        // Step 3: Propagate hot pulses from seeds
        let hot_freq = PosF32::new(F_HOT).unwrap();
        let half_budget = self.params.pulse_budget / 2;
        let hot_pulses = self.propagate_spectral(graph, seeds, hot_freq, config, half_budget)?;

        // Step 4: Propagate cold pulses from anti-seeds
        let cold_freq = PosF32::new(F_COLD).unwrap();
        let anti_seed_pairs: Vec<(NodeId, FiniteF32)> =
            anti_seeds.iter().map(|&n| (n, FiniteF32::ONE)).collect();
        let cold_pulses =
            self.propagate_spectral(graph, &anti_seed_pairs, cold_freq, config, half_budget)?;

        let total_pulses = hot_pulses.len() as u64 + cold_pulses.len() as u64;

        // Step 5: Accumulate per-node hot/cold amplitudes
        let mut hot_amp = vec![0.0f32; n];
        let mut cold_amp = vec![0.0f32; n];

        for p in &hot_pulses {
            let idx = p.node.as_usize();
            if idx < n {
                hot_amp[idx] += p.amplitude.get().abs();
            }
        }
        for p in &cold_pulses {
            let idx = p.node.as_usize();
            if idx < n {
                cold_amp[idx] += p.amplitude.get().abs();
            }
        }

        // Step 6: Adaptive differential with immunity, density, and sigmoid gating
        let mut activations = Vec::new();
        let mut all_zero = true;

        // Compute average degree for density modulation
        let avg_deg = graph.avg_degree();

        for (i, &hot) in hot_amp.iter().enumerate().take(n) {
            if hot <= 0.0 {
                continue;
            }

            // Immunity factor: immune nodes get full hot signal, no cold cancellation
            let immune = if i < immunity.len() {
                immunity[i]
            } else {
                false
            };

            let effective_cold = if immune { 0.0 } else { cold_amp[i] };

            // Raw differential
            let raw = hot - effective_cold;

            // Density modulation: nodes with degree near avg get density=1.0
            let out_deg = {
                let lo = graph.csr.offsets[i] as usize;
                let hi = if i + 1 < graph.csr.offsets.len() {
                    graph.csr.offsets[i + 1] as usize
                } else {
                    lo
                };
                (hi - lo) as f32
            };
            let density = if avg_deg > 0.0 {
                (out_deg / avg_deg).clamp(DENSITY_FLOOR, DENSITY_CAP)
            } else {
                1.0
            };

            // Sigmoid gate
            let gated = Self::sigmoid_gate(FiniteF32::new(raw * density));
            let val = gated.get();

            if val > 0.01 {
                activations.push((NodeId::new(i as u32), gated));
                all_zero = false;
            }
        }

        // FM-XLR-010: over-cancellation fallback
        let fallback = all_zero && !hot_pulses.is_empty();
        if fallback {
            // Return hot-only
            activations.clear();
            for (i, &amp) in hot_amp.iter().enumerate().take(n) {
                if amp > 0.01 {
                    activations.push((NodeId::new(i as u32), FiniteF32::new(amp)));
                }
            }
        }

        activations.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(XlrResult {
            activations,
            anti_seeds,
            fallback_to_hot_only: fallback,
            pulses_processed: total_pulses,
        })
    }

    /// Pick anti-seeds: structurally similar (degree), semantically different (Jaccard).
    /// Replaces: xlr_v2.py pick_anti_seeds()
    /// FM-XLR-008 fix: immunity computed from BFS reach, not seed count.
    pub fn pick_anti_seeds(&self, graph: &Graph, seeds: &[NodeId]) -> M1ndResult<Vec<NodeId>> {
        let n = graph.num_nodes() as usize;
        if n == 0 || seeds.is_empty() {
            return Ok(Vec::new());
        }

        // BFS to find seed neighborhood
        let mut seed_set = vec![false; n];
        let mut seed_neighbors = vec![false; n];
        for &s in seeds {
            let idx = s.as_usize();
            if idx < n {
                seed_set[idx] = true;
                seed_neighbors[idx] = true;
                let range = graph.csr.out_range(s);
                for j in range {
                    let tgt = graph.csr.targets[j].as_usize();
                    if tgt < n {
                        seed_neighbors[tgt] = true;
                    }
                }
            }
        }

        // Compute average seed degree
        let avg_seed_degree: f32 = if seeds.is_empty() {
            0.0
        } else {
            let sum: usize = seeds
                .iter()
                .map(|s| {
                    let r = graph.csr.out_range(*s);
                    r.end - r.start
                })
                .sum();
            sum as f32 / seeds.len() as f32
        };

        // Candidate scoring: structurally distant + similar degree
        let mut candidates: Vec<(NodeId, f32)> = Vec::new();
        for i in 0..n {
            if seed_set[i] {
                continue; // Skip seeds
            }

            let range = graph.csr.out_range(NodeId::new(i as u32));
            let degree = (range.end - range.start) as f32;

            // Degree ratio filter
            if avg_seed_degree > 0.0 {
                let ratio = degree / avg_seed_degree;
                if ratio < self.params.min_degree_ratio.get() {
                    continue;
                }
            }

            // Jaccard similarity with seed neighborhood (lower = better anti-seed)
            let mut intersection = 0usize;
            let mut union_size = 0usize;
            for j in range.clone() {
                let tgt = graph.csr.targets[j].as_usize();
                if tgt < n {
                    union_size += 1;
                    if seed_neighbors[tgt] {
                        intersection += 1;
                    }
                }
            }
            let jaccard = if union_size > 0 {
                intersection as f32 / union_size as f32
            } else {
                0.0
            };

            if jaccard > self.params.max_jaccard_similarity.get() {
                continue; // Too similar to seeds
            }

            // Score: higher = better anti-seed (distant + adequate degree)
            let distance_score = if seed_neighbors[i] { 0.0 } else { 1.0 };
            let score = distance_score + (1.0 - jaccard);
            candidates.push((NodeId::new(i as u32), score));
        }

        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
        let result: Vec<NodeId> = candidates
            .iter()
            .take(self.params.num_anti_seeds)
            .map(|c| c.0)
            .collect();
        Ok(result)
    }

    /// Compute seed neighborhood immunity set via BFS.
    /// Returns bitset of immune nodes (within immunity_hops of any seed).
    /// Replaces: xlr_v2.py compute_seed_neighborhood()
    /// FM-XLR-008 fix: BFS-based distance, not seed count threshold.
    pub fn compute_immunity(&self, graph: &Graph, seeds: &[NodeId]) -> M1ndResult<Vec<bool>> {
        let n = graph.num_nodes() as usize;
        let mut immune = vec![false; n];

        let mut queue = VecDeque::new();
        let mut dist = vec![u8::MAX; n];

        for &s in seeds {
            let idx = s.as_usize();
            if idx < n {
                queue.push_back((s, 0u8));
                dist[idx] = 0;
                immune[idx] = true;
            }
        }

        while let Some((node, d)) = queue.pop_front() {
            if d >= self.params.immunity_hops {
                continue;
            }
            let range = graph.csr.out_range(node);
            for j in range {
                let tgt = graph.csr.targets[j];
                let tgt_idx = tgt.as_usize();
                if tgt_idx < n && d + 1 < dist[tgt_idx] {
                    dist[tgt_idx] = d + 1;
                    immune[tgt_idx] = true;
                    queue.push_back((tgt, d + 1));
                }
            }
        }

        Ok(immune)
    }

    /// Propagate spectral pulses (hot or cold) from origins.
    /// Budget-limited (FM-RES-004).
    /// Replaces: xlr_v2.py SpectralPropagator.propagate()
    /// FM-XLR-014 fix: inhibitory edges do NOT flip cold phase.
    pub fn propagate_spectral(
        &self,
        graph: &Graph,
        origins: &[(NodeId, FiniteF32)],
        frequency: PosF32,
        config: &PropagationConfig,
        budget: u64,
    ) -> M1ndResult<Vec<SpectralPulse>> {
        let n = graph.num_nodes() as usize;
        let decay = config.decay.get();
        let threshold = config.threshold.get();
        let mut pulses_out = Vec::new();
        let mut pulse_count = 0u64;

        let mut queue: VecDeque<SpectralPulse> = VecDeque::new();

        // Init from origins
        for &(node, amp) in origins {
            if node.as_usize() >= n {
                continue;
            }
            let pulse = SpectralPulse {
                node,
                amplitude: amp,
                phase: FiniteF32::ZERO,
                frequency,
                hops: 0,
                prev_node: node,
                recent_path: [node; 3],
            };
            queue.push_back(pulse);
            pulses_out.push(pulse);
            pulse_count += 1;
        }

        let max_depth = config.max_depth.min(20);

        while let Some(pulse) = queue.pop_front() {
            if pulse_count >= budget {
                break; // FM-RES-004: budget exhausted
            }
            if pulse.hops >= max_depth {
                continue;
            }
            if pulse.amplitude.get().abs() < threshold {
                continue;
            }

            let range = graph.csr.out_range(pulse.node);
            for j in range {
                let tgt = graph.csr.targets[j];
                if tgt == pulse.prev_node {
                    continue; // Don't backtrack to immediate predecessor
                }

                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                let is_inhib = graph.csr.inhibitory[j];

                let mut new_amp = pulse.amplitude.get() * w * decay;

                // FM-XLR-014 FIX: inhibitory + cold does NOT flip phase.
                // Just attenuate by INHIBITORY_COLD_ATTENUATION (DEC-010).
                if is_inhib {
                    new_amp *= INHIBITORY_COLD_ATTENUATION;
                }

                if new_amp.abs() < threshold {
                    continue;
                }

                // Phase advance
                let phase_advance = 2.0 * std::f32::consts::PI * frequency.get();
                let new_phase = (pulse.phase.get() + phase_advance) % (2.0 * std::f32::consts::PI);

                // Update recent path (shift)
                let mut rp = pulse.recent_path;
                rp[2] = rp[1];
                rp[1] = rp[0];
                rp[0] = pulse.node;

                let new_pulse = SpectralPulse {
                    node: tgt,
                    amplitude: FiniteF32::new(new_amp),
                    phase: FiniteF32::new(new_phase),
                    frequency,
                    hops: pulse.hops + 1,
                    prev_node: pulse.node,
                    recent_path: rp,
                };

                pulses_out.push(new_pulse);
                pulse_count += 1;
                if pulse_count < budget {
                    queue.push_back(new_pulse);
                }
            }
        }

        Ok(pulses_out)
    }

    /// Compute spectral overlap between hot and cold signals at each node.
    /// DEC-003: bucket-based overlap for O(B) per node.
    /// Replaces: xlr_v2.py adaptive_differential() spectral overlap section
    pub fn spectral_overlap(hot_freqs: &[FiniteF32], cold_freqs: &[FiniteF32]) -> FiniteF32 {
        if hot_freqs.is_empty() || cold_freqs.is_empty() {
            return FiniteF32::ZERO;
        }

        // Bucket both signals
        let mut hot_buckets = [0.0f32; SPECTRAL_BUCKETS];
        let mut cold_buckets = [0.0f32; SPECTRAL_BUCKETS];

        let max_freq = 10.0f32; // Reasonable max for bucketing
        let bucket_width = max_freq / SPECTRAL_BUCKETS as f32;

        for f in hot_freqs {
            let b = ((f.get() / bucket_width) as usize).min(SPECTRAL_BUCKETS - 1);
            hot_buckets[b] += 1.0;
        }
        for f in cold_freqs {
            let b = ((f.get() / bucket_width) as usize).min(SPECTRAL_BUCKETS - 1);
            cold_buckets[b] += 1.0;
        }

        // Overlap = sum(min(hot, cold)) / sum(hot)
        let mut overlap = 0.0f32;
        let mut hot_total = 0.0f32;
        for b in 0..SPECTRAL_BUCKETS {
            overlap += hot_buckets[b].min(cold_buckets[b]);
            hot_total += hot_buckets[b];
        }

        if hot_total > 0.0 {
            FiniteF32::new(overlap / hot_total)
        } else {
            FiniteF32::ZERO
        }
    }

    /// Sigmoid gating: activation = sigmoid(x * SIGMOID_STEEPNESS).
    /// Replaces: xlr_v2.py sigmoid gating in adaptive_differential()
    pub fn sigmoid_gate(net_signal: FiniteF32) -> FiniteF32 {
        let x = net_signal.get() * SIGMOID_STEEPNESS;
        // Clamp to avoid overflow in exp
        let clamped = x.clamp(-20.0, 20.0);
        let result = 1.0 / (1.0 + (-clamped).exp());
        FiniteF32::new(result)
    }
}

// Ensure Send + Sync for concurrent query serving.
static_assertions::assert_impl_all!(AdaptiveXlrEngine: Send, Sync);
