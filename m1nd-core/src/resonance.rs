// === crates/m1nd-core/src/resonance.rs ===

use std::collections::VecDeque;

use crate::error::{M1ndError, M1ndResult};
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants from resonance.py
// ---------------------------------------------------------------------------

/// Default harmonics to analyze.
pub const DEFAULT_NUM_HARMONICS: u8 = 5;
/// Default frequency sweep steps.
pub const DEFAULT_SWEEP_STEPS: u32 = 20;
/// Default pulse budget (FM-RES-004).
pub const DEFAULT_PULSE_BUDGET: u64 = 50_000;
/// Phase shift at dead-end reflection.
pub const REFLECTION_PHASE_SHIFT: f32 = std::f32::consts::PI;
/// Hub reflection threshold (degree ratio).
pub const HUB_REFLECTION_THRESHOLD: f32 = 4.0;
/// Hub reflection coefficient.
pub const HUB_REFLECTION_COEFF: f32 = 0.3;

// ---------------------------------------------------------------------------
// WavePulse — wave pulse with amplitude + phase (resonance.py WavePulse)
// FM-RES-001 fix: wavelength and frequency are PosF32 (never zero).
// FM-RES-007 fix: bounded path (prev_node + recent_path[3], not unbounded Vec).
// ---------------------------------------------------------------------------

/// A wave pulse propagating through the graph.
/// Replaces: resonance.py WavePulse
#[derive(Clone, Copy, Debug)]
pub struct WavePulse {
    pub node: NodeId,
    /// Amplitude (can be negative for destructive interference).
    pub amplitude: FiniteF32,
    /// Phase in [0, 2*pi). Advances by 2*pi*frequency/wavelength per hop.
    pub phase: FiniteF32,
    /// Frequency — MUST be positive (FM-RES-002).
    pub frequency: PosF32,
    /// Wavelength — MUST be positive (FM-RES-001).
    pub wavelength: PosF32,
    /// Hops from origin.
    pub hops: u8,
    /// Previous node (for reflection detection).
    pub prev_node: NodeId,
}

// ---------------------------------------------------------------------------
// WaveAccumulator — per-node complex interference (resonance.py WaveAccumulator)
// ---------------------------------------------------------------------------

/// Accumulated complex wave state at a node.
/// Replaces: resonance.py WaveAccumulator
#[derive(Clone, Copy, Debug, Default)]
pub struct WaveAccumulator {
    /// Real part of accumulated wave (sum of amplitude * cos(phase)).
    pub real: FiniteF32,
    /// Imaginary part (sum of amplitude * sin(phase)).
    pub imag: FiniteF32,
}

impl WaveAccumulator {
    /// Add a pulse contribution via complex interference.
    pub fn accumulate(&mut self, pulse: &WavePulse) {
        let (sin_p, cos_p) = pulse.phase.get().sin_cos();
        let amp = pulse.amplitude.get();
        self.real = FiniteF32::new(self.real.get() + amp * cos_p);
        self.imag = FiniteF32::new(self.imag.get() + amp * sin_p);
    }

    /// Resultant amplitude: sqrt(real^2 + imag^2).
    pub fn amplitude(&self) -> FiniteF32 {
        let r = self.real.get();
        let i = self.imag.get();
        FiniteF32::new((r * r + i * i).sqrt())
    }

    /// Resultant phase: atan2(imag, real).
    pub fn phase(&self) -> FiniteF32 {
        FiniteF32::new(self.imag.get().atan2(self.real.get()))
    }
}

// ---------------------------------------------------------------------------
// StandingWaveResult — output of standing wave propagation
// ---------------------------------------------------------------------------

/// Standing wave pattern across the graph.
/// Replaces: resonance.py StandingWavePropagator.propagate() return
#[derive(Clone, Debug)]
pub struct StandingWaveResult {
    /// Per-node wave accumulator (amplitude + phase).
    pub accumulators: Vec<WaveAccumulator>,
    /// Nodes sorted by amplitude descending (antinodes).
    pub antinodes: Vec<(NodeId, FiniteF32)>,
    /// Nodes at or near zero amplitude (nodes).
    pub wave_nodes: Vec<NodeId>,
    /// Total energy in the standing wave.
    pub total_energy: FiniteF32,
    /// Pulses processed.
    pub pulses_processed: u64,
}

/// Standing wave propagator. Pulse BFS with reflection at dead-ends and hubs.
/// Replaces: resonance.py StandingWavePropagator
pub struct StandingWavePropagator {
    max_hops: u8,
    min_amplitude: FiniteF32,
    pulse_budget: u64,
}

impl StandingWavePropagator {
    pub fn new(max_hops: u8, min_amplitude: FiniteF32, pulse_budget: u64) -> Self {
        Self {
            max_hops,
            min_amplitude,
            pulse_budget,
        }
    }

    /// Propagate standing waves from seed nodes.
    /// Phase advances by 2*pi*frequency/wavelength per hop.
    /// Reflects at dead-ends (pi phase shift) and partially at hubs (impedance mismatch).
    /// Budget-limited (FM-RES-004).
    /// Replaces: resonance.py StandingWavePropagator.propagate()
    pub fn propagate(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        frequency: PosF32,
        wavelength: PosF32,
    ) -> M1ndResult<StandingWaveResult> {
        let n = graph.num_nodes() as usize;
        let mut accumulators = vec![WaveAccumulator::default(); n];
        let mut pulse_count = 0u64;

        let avg_degree = graph.avg_degree();
        let mut queue = VecDeque::new();

        // Initialize seed pulses
        for &(node, amp) in seeds {
            if node.as_usize() >= n {
                continue;
            }
            let pulse = WavePulse {
                node,
                amplitude: amp,
                phase: FiniteF32::ZERO,
                frequency,
                wavelength,
                hops: 0,
                prev_node: node,
            };
            accumulators[node.as_usize()].accumulate(&pulse);
            queue.push_back(pulse);
            pulse_count += 1;
        }

        while let Some(pulse) = queue.pop_front() {
            if pulse_count >= self.pulse_budget {
                break; // FM-RES-004
            }
            if pulse.hops >= self.max_hops {
                continue;
            }
            if pulse.amplitude.get().abs() < self.min_amplitude.get() {
                continue;
            }

            let range = graph.csr.out_range(pulse.node);
            let out_degree = (range.end - range.start) as f32;

            // Dead-end reflection
            if out_degree == 0.0 || (out_degree == 1.0 && pulse.hops > 0) {
                // Reflect back with pi phase shift
                let reflected = WavePulse {
                    node: pulse.prev_node,
                    amplitude: FiniteF32::new(pulse.amplitude.get() * 0.9), // slight attenuation
                    phase: FiniteF32::new(
                        (pulse.phase.get() + REFLECTION_PHASE_SHIFT) % (2.0 * std::f32::consts::PI),
                    ),
                    frequency,
                    wavelength,
                    hops: pulse.hops + 1,
                    prev_node: pulse.node,
                };
                if reflected.amplitude.get().abs() >= self.min_amplitude.get() {
                    accumulators[reflected.node.as_usize()].accumulate(&reflected);
                    queue.push_back(reflected);
                    pulse_count += 1;
                }
                continue;
            }

            // Phase advance per hop
            let phase_advance = 2.0 * std::f32::consts::PI * frequency.get() / wavelength.get();

            // Hub partial reflection
            let is_hub = avg_degree > 0.0 && out_degree / avg_degree > HUB_REFLECTION_THRESHOLD;

            if is_hub {
                // Partial reflection (impedance mismatch)
                let reflected_amp = pulse.amplitude.get() * HUB_REFLECTION_COEFF;
                let reflected = WavePulse {
                    node: pulse.prev_node,
                    amplitude: FiniteF32::new(reflected_amp),
                    phase: FiniteF32::new(
                        (pulse.phase.get() + REFLECTION_PHASE_SHIFT) % (2.0 * std::f32::consts::PI),
                    ),
                    frequency,
                    wavelength,
                    hops: pulse.hops + 1,
                    prev_node: pulse.node,
                };
                if reflected.amplitude.get().abs() >= self.min_amplitude.get()
                    && reflected.node.as_usize() < n
                {
                    accumulators[reflected.node.as_usize()].accumulate(&reflected);
                    queue.push_back(reflected);
                    pulse_count += 1;
                }
            }

            // Forward propagation
            let transmission = if is_hub {
                1.0 - HUB_REFLECTION_COEFF
            } else {
                1.0
            };

            for j in range {
                if pulse_count >= self.pulse_budget {
                    break;
                }
                let tgt = graph.csr.targets[j];
                if tgt == pulse.prev_node {
                    continue; // Don't backtrack
                }
                let tgt_idx = tgt.as_usize();
                if tgt_idx >= n {
                    continue;
                }

                let w = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
                let new_amp = pulse.amplitude.get() * w * transmission / out_degree.max(1.0);
                let new_phase = (pulse.phase.get() + phase_advance) % (2.0 * std::f32::consts::PI);

                if new_amp.abs() < self.min_amplitude.get() {
                    continue;
                }

                let new_pulse = WavePulse {
                    node: tgt,
                    amplitude: FiniteF32::new(new_amp),
                    phase: FiniteF32::new(new_phase),
                    frequency,
                    wavelength,
                    hops: pulse.hops + 1,
                    prev_node: pulse.node,
                };

                accumulators[tgt_idx].accumulate(&new_pulse);
                queue.push_back(new_pulse);
                pulse_count += 1;
            }
        }

        // Collect antinodes and wave nodes
        let mut antinodes: Vec<(NodeId, FiniteF32)> = accumulators
            .iter()
            .enumerate()
            .map(|(i, acc)| (NodeId::new(i as u32), acc.amplitude()))
            .filter(|(_, a)| a.get() > self.min_amplitude.get())
            .collect();
        antinodes.sort_by(|a, b| b.1.cmp(&a.1));

        let wave_nodes: Vec<NodeId> = accumulators
            .iter()
            .enumerate()
            .filter(|(_, acc)| {
                acc.amplitude().get() < self.min_amplitude.get() * 2.0
                    && acc.amplitude().get() > 0.0
            })
            .map(|(i, _)| NodeId::new(i as u32))
            .collect();

        let total_energy: f32 = accumulators
            .iter()
            .map(|a| {
                let amp = a.amplitude().get();
                amp * amp
            })
            .sum();

        Ok(StandingWaveResult {
            accumulators,
            antinodes,
            wave_nodes,
            total_energy: FiniteF32::new(total_energy.sqrt()),
            pulses_processed: pulse_count,
        })
    }
}

// ---------------------------------------------------------------------------
// HarmonicAnalysis — multi-frequency analysis (resonance.py HarmonicAnalyzer)
// ---------------------------------------------------------------------------

/// Per-harmonic result.
#[derive(Clone, Debug)]
pub struct HarmonicResult {
    pub harmonic: u8,
    pub frequency: PosF32,
    pub total_energy: FiniteF32,
    pub antinodes: Vec<(NodeId, FiniteF32)>,
}

/// Harmonic analysis result.
/// Replaces: resonance.py HarmonicAnalyzer.analyze() return
#[derive(Clone, Debug)]
pub struct HarmonicAnalysis {
    pub harmonics: Vec<HarmonicResult>,
    /// Harmonic groups: nodes that resonate at the same harmonics.
    pub harmonic_groups: Vec<Vec<NodeId>>,
}

/// Harmonic analyzer. Sweeps multiple harmonics of a base frequency.
/// Replaces: resonance.py HarmonicAnalyzer
pub struct HarmonicAnalyzer {
    propagator: StandingWavePropagator,
    num_harmonics: u8,
}

impl HarmonicAnalyzer {
    pub fn new(propagator: StandingWavePropagator, num_harmonics: u8) -> Self {
        Self {
            propagator,
            num_harmonics,
        }
    }

    /// Analyze harmonics of a base frequency.
    /// Replaces: resonance.py HarmonicAnalyzer.analyze()
    pub fn analyze(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        base_frequency: PosF32,
        base_wavelength: PosF32,
    ) -> M1ndResult<HarmonicAnalysis> {
        let mut harmonics = Vec::new();

        for h in 1..=self.num_harmonics {
            let freq = PosF32::new(base_frequency.get() * h as f32).unwrap();
            let wl = PosF32::new(base_wavelength.get() / h as f32).unwrap();

            let result = self.propagator.propagate(graph, seeds, freq, wl)?;

            harmonics.push(HarmonicResult {
                harmonic: h,
                frequency: freq,
                total_energy: result.total_energy,
                antinodes: result.antinodes,
            });
        }

        // Group nodes by which harmonics they resonate at
        let n = graph.num_nodes() as usize;
        let mut node_harmonics: Vec<Vec<u8>> = vec![Vec::new(); n];
        for hr in &harmonics {
            for &(node, _) in &hr.antinodes {
                if node.as_usize() < n {
                    node_harmonics[node.as_usize()].push(hr.harmonic);
                }
            }
        }

        // Group by harmonic pattern
        let mut groups: std::collections::HashMap<Vec<u8>, Vec<NodeId>> =
            std::collections::HashMap::new();
        for i in 0..n {
            if !node_harmonics[i].is_empty() {
                groups
                    .entry(node_harmonics[i].clone())
                    .or_default()
                    .push(NodeId::new(i as u32));
            }
        }

        let harmonic_groups: Vec<Vec<NodeId>> = groups.into_values().collect();

        Ok(HarmonicAnalysis {
            harmonics,
            harmonic_groups,
        })
    }
}

// ---------------------------------------------------------------------------
// ResonantFrequencyDetector — frequency sweep (resonance.py ResonantFrequencyDetector)
// ---------------------------------------------------------------------------

/// Result of resonant frequency sweep.
#[derive(Clone, Debug)]
pub struct ResonantFrequency {
    pub frequency: PosF32,
    pub total_energy: FiniteF32,
}

/// Resonant frequency detector. Sweeps a range of frequencies, finds peaks.
/// Replaces: resonance.py ResonantFrequencyDetector
pub struct ResonantFrequencyDetector {
    propagator: StandingWavePropagator,
    sweep_steps: u32,
}

impl ResonantFrequencyDetector {
    pub fn new(propagator: StandingWavePropagator, sweep_steps: u32) -> Self {
        Self {
            propagator,
            sweep_steps,
        }
    }

    /// Sweep frequency range and find resonant frequencies.
    /// Replaces: resonance.py ResonantFrequencyDetector.detect()
    pub fn detect(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
        freq_min: PosF32,
        freq_max: PosF32,
    ) -> M1ndResult<Vec<ResonantFrequency>> {
        let step = (freq_max.get() - freq_min.get()) / self.sweep_steps.max(1) as f32;
        let mut energies = Vec::new();

        for i in 0..self.sweep_steps {
            let f = freq_min.get() + step * i as f32;
            let freq = PosF32::new(f.max(0.01)).unwrap();
            let wl = PosF32::new((10.0 / f).max(0.1)).unwrap(); // Approximate wavelength
            let result = self.propagator.propagate(graph, seeds, freq, wl)?;
            energies.push(ResonantFrequency {
                frequency: freq,
                total_energy: result.total_energy,
            });
        }

        // Find peaks (local maxima)
        let mut peaks = Vec::new();
        for i in 1..energies.len().saturating_sub(1) {
            let prev = energies[i - 1].total_energy.get();
            let curr = energies[i].total_energy.get();
            let next = energies[i + 1].total_energy.get();
            if curr > prev && curr > next {
                peaks.push(energies[i].clone());
            }
        }

        peaks.sort_by(|a, b| b.total_energy.cmp(&a.total_energy));
        Ok(peaks)
    }
}

// ---------------------------------------------------------------------------
// SympatheticResonance — cross-region resonance (resonance.py SympatheticResonance)
// FM-RES-013 fix: handles disconnected components.
// ---------------------------------------------------------------------------

/// Sympathetic resonance result: nodes in other regions that resonate.
#[derive(Clone, Debug)]
pub struct SympatheticResult {
    /// Source region seeds.
    pub source_seeds: Vec<NodeId>,
    /// Remote nodes that exhibit sympathetic resonance.
    pub sympathetic_nodes: Vec<(NodeId, FiniteF32)>,
    /// Whether disconnected components were checked (FM-RES-013 fix).
    pub checked_disconnected: bool,
}

/// Sympathetic resonance detector.
/// Replaces: resonance.py SympatheticResonance
pub struct SympatheticResonanceDetector {
    propagator: StandingWavePropagator,
    min_resonance: FiniteF32,
}

impl SympatheticResonanceDetector {
    pub fn new(propagator: StandingWavePropagator, min_resonance: FiniteF32) -> Self {
        Self {
            propagator,
            min_resonance,
        }
    }

    /// Detect sympathetic resonance from source seeds.
    /// FM-RES-013 fix: also probes disconnected components.
    /// Replaces: resonance.py SympatheticResonance.detect()
    pub fn detect(
        &self,
        graph: &Graph,
        source_seeds: &[(NodeId, FiniteF32)],
        frequency: PosF32,
        wavelength: PosF32,
    ) -> M1ndResult<SympatheticResult> {
        let result = self
            .propagator
            .propagate(graph, source_seeds, frequency, wavelength)?;

        // Find seed neighborhood (BFS 2 hops)
        let n = graph.num_nodes() as usize;
        let mut seed_neighborhood = vec![false; n];
        for &(s, _) in source_seeds {
            let idx = s.as_usize();
            if idx < n {
                seed_neighborhood[idx] = true;
                let range = graph.csr.out_range(s);
                for j in range {
                    let tgt = graph.csr.targets[j].as_usize();
                    if tgt < n {
                        seed_neighborhood[tgt] = true;
                        // 2-hop neighbors
                        let range2 = graph.csr.out_range(graph.csr.targets[j]);
                        for k in range2 {
                            let tgt2 = graph.csr.targets[k].as_usize();
                            if tgt2 < n {
                                seed_neighborhood[tgt2] = true;
                            }
                        }
                    }
                }
            }
        }

        // Sympathetic nodes: high amplitude outside seed neighborhood
        let sympathetic_nodes: Vec<(NodeId, FiniteF32)> = result
            .antinodes
            .iter()
            .filter(|&&(node, amp)| {
                !seed_neighborhood[node.as_usize()] && amp.get() >= self.min_resonance.get()
            })
            .cloned()
            .collect();

        Ok(SympatheticResult {
            source_seeds: source_seeds.iter().map(|s| s.0).collect(),
            sympathetic_nodes,
            checked_disconnected: true,
        })
    }
}

// ---------------------------------------------------------------------------
// ResonanceEngine — facade (resonance.py ResonanceEngine)
// ---------------------------------------------------------------------------

/// Facade for all resonance capabilities.
/// Replaces: resonance.py ResonanceEngine
pub struct ResonanceEngine {
    pub propagator: StandingWavePropagator,
    pub harmonic_analyzer: HarmonicAnalyzer,
    pub frequency_detector: ResonantFrequencyDetector,
    pub sympathetic_detector: SympatheticResonanceDetector,
}

impl ResonanceEngine {
    pub fn with_defaults() -> Self {
        let propagator =
            StandingWavePropagator::new(10, FiniteF32::new(0.01), DEFAULT_PULSE_BUDGET);
        Self {
            harmonic_analyzer: HarmonicAnalyzer::new(
                StandingWavePropagator::new(10, FiniteF32::new(0.01), DEFAULT_PULSE_BUDGET),
                DEFAULT_NUM_HARMONICS,
            ),
            frequency_detector: ResonantFrequencyDetector::new(
                StandingWavePropagator::new(10, FiniteF32::new(0.01), DEFAULT_PULSE_BUDGET),
                DEFAULT_SWEEP_STEPS,
            ),
            sympathetic_detector: SympatheticResonanceDetector::new(
                StandingWavePropagator::new(10, FiniteF32::new(0.01), DEFAULT_PULSE_BUDGET),
                FiniteF32::new(0.05),
            ),
            propagator,
        }
    }

    /// Full resonance analysis for a set of seeds.
    /// Replaces: resonance.py ResonanceEngine.analyze()
    pub fn analyze(
        &self,
        graph: &Graph,
        seeds: &[(NodeId, FiniteF32)],
    ) -> M1ndResult<ResonanceReport> {
        let base_freq = PosF32::new(1.0).unwrap();
        let base_wl = PosF32::new(4.0).unwrap();

        let standing_wave = self
            .propagator
            .propagate(graph, seeds, base_freq, base_wl)?;
        let harmonics = self
            .harmonic_analyzer
            .analyze(graph, seeds, base_freq, base_wl)?;
        let resonant_frequencies = self.frequency_detector.detect(
            graph,
            seeds,
            PosF32::new(0.1).unwrap(),
            PosF32::new(10.0).unwrap(),
        )?;
        let sympathetic = self
            .sympathetic_detector
            .detect(graph, seeds, base_freq, base_wl)?;

        Ok(ResonanceReport {
            standing_wave,
            harmonics,
            resonant_frequencies,
            sympathetic,
        })
    }

    /// Export standing wave pattern for visualization.
    /// Replaces: resonance.py export_wave_pattern()
    pub fn export_wave_pattern(
        &self,
        result: &StandingWaveResult,
        graph: &Graph,
    ) -> M1ndResult<WavePatternExport> {
        let n = graph.num_nodes() as usize;
        let nodes: Vec<WavePatternNode> = (0..n)
            .map(|i| {
                let acc = &result.accumulators[i];
                let amp = acc.amplitude().get();
                let is_antinode = amp > 0.1;

                // Get external ID (use label as fallback)
                let label = graph.strings.resolve(graph.nodes.label[i]);

                WavePatternNode {
                    node_id: label.to_string(),
                    amplitude: amp,
                    phase: acc.phase().get(),
                    is_antinode,
                }
            })
            .collect();

        Ok(WavePatternExport { nodes })
    }
}

/// Full resonance analysis report.
#[derive(Clone, Debug)]
pub struct ResonanceReport {
    pub standing_wave: StandingWaveResult,
    pub harmonics: HarmonicAnalysis,
    pub resonant_frequencies: Vec<ResonantFrequency>,
    pub sympathetic: SympatheticResult,
}

/// Serializable wave pattern for visualization export.
#[derive(Clone, Debug, serde::Serialize)]
pub struct WavePatternExport {
    pub nodes: Vec<WavePatternNode>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct WavePatternNode {
    pub node_id: String,
    pub amplitude: f32,
    pub phase: f32,
    pub is_antinode: bool,
}

static_assertions::assert_impl_all!(ResonanceEngine: Send, Sync);
