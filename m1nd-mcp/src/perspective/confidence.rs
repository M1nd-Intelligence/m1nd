// === m1nd-mcp/src/perspective/confidence.rs ===
// Theme 13: Confidence Calibration and Epistemic Safety.
// All normalization functions and the geometric mean combiner.

use super::state::{AffinityCandidate, AffinityCandidateKind, ConfidenceBreakdown};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum confidence for any affinity candidate (Theme 13).
pub const MAX_CONFIDENCE: f32 = 0.85;

/// Minimum threshold — below this is noise, dropped (Theme 13).
pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.15;

/// Minimum number of sources with non-None scores to be a candidate above 0.40.
pub const MIN_SOURCES_FOR_HIGH_CONFIDENCE: usize = 2;

// ---------------------------------------------------------------------------
// Per-source normalization (all to [0.0, 1.0])
// ---------------------------------------------------------------------------

/// Ghost edge strength: sqrt(raw) to spread biased-low distribution.
pub fn normalize_ghost_edge(raw: f32) -> f32 {
    raw.max(0.0).sqrt().min(1.0)
}

/// Structural hole pressure: (avg - min_sibling) / (1.0 - min_sibling).
/// Returns 0.0 if min_sibling >= 1.0 (degenerate case).
pub fn normalize_structural_hole(avg_activation: f32, min_sibling: f32) -> f32 {
    let denom = 1.0 - min_sibling;
    if denom <= 0.0 {
        return 0.0;
    }
    ((avg_activation - min_sibling) / denom).clamp(0.0, 1.0)
}

/// Resonant amplitude: divide by max amplitude in the resonance report.
/// Returns 0.0 if max_amplitude is zero.
pub fn normalize_resonant_amplitude(amplitude: f32, max_amplitude: f32) -> f32 {
    if max_amplitude <= 0.0 {
        return 0.0;
    }
    (amplitude / max_amplitude).clamp(0.0, 1.0)
}

/// Semantic overlap: cosine similarity, already [0, 1]. Pass through with clamp.
pub fn normalize_semantic_overlap(cosine: f32) -> f32 {
    cosine.clamp(0.0, 1.0)
}

/// Provenance overlap: 1.0 if same file within 50 lines, 0.5 if same file, 0.0 otherwise.
pub fn normalize_provenance_overlap(same_file: bool, line_distance: Option<u32>) -> f32 {
    if !same_file {
        return 0.0;
    }
    match line_distance {
        Some(d) if d <= 50 => 1.0,
        _ => 0.5,
    }
}

/// Route-path neighborhood: 1.0 / (1.0 + hop_distance).
pub fn normalize_route_path_neighborhood(hop_distance: u32) -> f32 {
    1.0 / (1.0 + hop_distance as f32)
}

// ---------------------------------------------------------------------------
// Combined confidence (geometric mean)
// ---------------------------------------------------------------------------

/// Compute combined confidence from a breakdown.
///
/// Uses weighted geometric mean (product, not sum) of normalized sources.
/// Product penalizes single-source candidates harder.
/// Capped at MAX_CONFIDENCE (0.85). Returns None if below MIN_CONFIDENCE_THRESHOLD (0.15).
pub fn compute_combined_confidence(breakdown: &ConfidenceBreakdown) -> Option<f32> {
    let scores: Vec<f32> = [
        breakdown.ghost_edge_strength,
        breakdown.structural_hole_pressure,
        breakdown.resonant_amplitude,
        breakdown.semantic_overlap,
        breakdown.provenance_overlap,
        breakdown.route_path_neighborhood,
    ]
    .iter()
    .filter_map(|s| *s)
    .collect();

    if scores.is_empty() {
        return None;
    }

    // Geometric mean
    let product: f64 = scores.iter().map(|&s| s as f64).product();
    let geo_mean = product.powf(1.0 / scores.len() as f64) as f32;

    // Cap
    let capped = geo_mean.min(MAX_CONFIDENCE);

    // Threshold
    if capped < MIN_CONFIDENCE_THRESHOLD {
        return None;
    }

    // Multi-source gate: if only 1 source and confidence > 0.40, require corroboration
    if scores.len() < MIN_SOURCES_FOR_HIGH_CONFIDENCE && capped > 0.40 {
        return Some(0.40_f32.min(MAX_CONFIDENCE));
    }

    Some(capped)
}

// ---------------------------------------------------------------------------
// Ranking formula helpers (Theme 13)
// ---------------------------------------------------------------------------

/// path_coherence: geometric mean of edge weights along shortest path,
/// normalized by path length. Returns 0.0 for structural holes (no path).
pub fn compute_path_coherence(edge_weights: &[f32]) -> f32 {
    if edge_weights.is_empty() {
        return 0.0;
    }
    let product: f64 = edge_weights.iter().map(|&w| w.max(0.0) as f64).product();
    let geo_mean = product.powf(1.0 / edge_weights.len() as f64) as f32;
    // Normalize by path length: shorter paths get higher coherence
    geo_mean / (1.0 + (edge_weights.len() as f32 - 1.0) * 0.1)
}

/// novelty: 1.0 if target not visited, 0.0 if visited, 0.5 if neighbor of visited.
pub fn compute_novelty(target: &str, visited: &std::collections::HashSet<String>, neighbor_of_visited: bool) -> f32 {
    if visited.contains(target) {
        0.0
    } else if neighbor_of_visited {
        0.5
    } else {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Builder helper
// ---------------------------------------------------------------------------

/// Build an AffinityCandidate with all epistemic guards.
pub fn build_affinity_candidate(
    candidate_node: String,
    candidate_label: String,
    kind: AffinityCandidateKind,
    breakdown: ConfidenceBreakdown,
) -> Option<AffinityCandidate> {
    let confidence = compute_combined_confidence(&breakdown)?;

    Some(AffinityCandidate {
        candidate_node,
        candidate_label,
        kind,
        confidence,
        is_hypothetical: true, // always true for affinity candidates
        proposed_relation: None, // V1: always None
        confidence_breakdown: breakdown,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ghost_edge_normalization() {
        assert_eq!(normalize_ghost_edge(0.0), 0.0);
        assert!((normalize_ghost_edge(0.25) - 0.5).abs() < 0.001);
        assert!((normalize_ghost_edge(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn structural_hole_normalization() {
        assert!((normalize_structural_hole(0.6, 0.2) - 0.5).abs() < 0.001);
        assert_eq!(normalize_structural_hole(0.5, 1.0), 0.0); // degenerate
    }

    #[test]
    fn resonant_normalization_zero_max() {
        assert_eq!(normalize_resonant_amplitude(0.5, 0.0), 0.0);
    }

    #[test]
    fn provenance_overlap_same_file_close() {
        assert_eq!(normalize_provenance_overlap(true, Some(30)), 1.0);
        assert_eq!(normalize_provenance_overlap(true, Some(100)), 0.5);
        assert_eq!(normalize_provenance_overlap(false, None), 0.0);
    }

    #[test]
    fn route_path_neighborhood() {
        assert_eq!(normalize_route_path_neighborhood(0), 1.0);
        assert!((normalize_route_path_neighborhood(1) - 0.5).abs() < 0.001);
    }

    #[test]
    fn combined_confidence_caps_at_max() {
        let breakdown = ConfidenceBreakdown {
            ghost_edge_strength: Some(1.0),
            structural_hole_pressure: Some(1.0),
            resonant_amplitude: Some(1.0),
            semantic_overlap: Some(1.0),
            provenance_overlap: Some(1.0),
            route_path_neighborhood: Some(1.0),
        };
        let c = compute_combined_confidence(&breakdown).unwrap();
        assert!(c <= MAX_CONFIDENCE);
    }

    #[test]
    fn combined_confidence_drops_below_threshold() {
        let breakdown = ConfidenceBreakdown {
            ghost_edge_strength: Some(0.01),
            structural_hole_pressure: Some(0.01),
            resonant_amplitude: None,
            semantic_overlap: None,
            provenance_overlap: None,
            route_path_neighborhood: None,
        };
        assert!(compute_combined_confidence(&breakdown).is_none());
    }

    #[test]
    fn combined_confidence_empty_returns_none() {
        let breakdown = ConfidenceBreakdown {
            ghost_edge_strength: None,
            structural_hole_pressure: None,
            resonant_amplitude: None,
            semantic_overlap: None,
            provenance_overlap: None,
            route_path_neighborhood: None,
        };
        assert!(compute_combined_confidence(&breakdown).is_none());
    }

    #[test]
    fn path_coherence_empty() {
        assert_eq!(compute_path_coherence(&[]), 0.0);
    }

    #[test]
    fn path_coherence_single_edge() {
        assert!((compute_path_coherence(&[0.8]) - 0.8).abs() < 0.001);
    }

    #[test]
    fn novelty_scoring() {
        let mut visited = HashSet::new();
        visited.insert("a".to_string());
        assert_eq!(compute_novelty("a", &visited, false), 0.0);
        assert_eq!(compute_novelty("b", &visited, true), 0.5);
        assert_eq!(compute_novelty("c", &visited, false), 1.0);
    }

    #[test]
    fn single_source_gate() {
        // Single source with high score should be capped at 0.40
        let breakdown = ConfidenceBreakdown {
            ghost_edge_strength: Some(0.9),
            structural_hole_pressure: None,
            resonant_amplitude: None,
            semantic_overlap: None,
            provenance_overlap: None,
            route_path_neighborhood: None,
        };
        let c = compute_combined_confidence(&breakdown).unwrap();
        assert!(c <= 0.40);
    }
}
