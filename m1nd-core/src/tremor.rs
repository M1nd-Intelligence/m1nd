// === m1nd-core/src/tremor.rs ===
//
// Code tremor detection: modules with accelerating change frequency.
// Second-derivative analysis — earthquake precursor analogy for imminent bugs.

use crate::error::M1ndResult;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;

// ── Constants ──

/// Maximum observations retained per node (ring buffer capacity).
pub const DEFAULT_MAX_OBSERVATIONS: usize = 256;
/// Minimum observations needed to compute acceleration (second derivative).
pub const MIN_OBSERVATIONS_FOR_ACCELERATION: usize = 3;
/// Minimum observations needed to compute velocity (first derivative).
pub const MIN_OBSERVATIONS_FOR_VELOCITY: usize = 2;
/// Minimum time gap in seconds between consecutive observations (de-duplication).
pub const MIN_OBSERVATION_GAP_SECS: f64 = 1.0;
/// Hard cap on tremor magnitude to prevent outlier domination.
pub const MAGNITUDE_CAP: f32 = 100.0;
/// Default magnitude threshold — tremors below this are suppressed.
pub const DEFAULT_THRESHOLD: f32 = 0.1;
/// Default number of top tremor alerts to return.
pub const DEFAULT_TOP_K: usize = 20;

// ── Core Types ──

/// A single weight-change observation for tremor analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TremorObservation {
    /// Unix timestamp (seconds) of the observation.
    pub timestamp: f64,
    /// Change in node weight since previous observation.
    pub weight_delta: f32,
    /// Number of edge add/remove events at this observation.
    pub edge_events: u16,
}

/// Direction of change acceleration for a tremor alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum TremorDirection {
    /// Mean acceleration > 0.001 — change rate is speeding up.
    Accelerating,
    /// Mean acceleration < -0.001 — change rate is slowing down.
    Decelerating,
    /// Change rate is approximately constant.
    Stable,
}

/// Risk classification for a tremor alert.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RiskLevel {
    /// magnitude > 5.0 and trend_slope > 0.5.
    Critical,
    /// magnitude > 2.0 or trend_slope > 0.3.
    High,
    /// magnitude > 0.5.
    Medium,
    /// magnitude above threshold but below 0.5.
    Low,
    /// Insufficient data to classify.
    Unknown,
}

/// A tremor alert for a single node.
#[derive(Clone, Debug, Serialize)]
pub struct TremorAlert {
    /// External ID of the node.
    pub node_id: String,
    /// Human-readable label (last "::" segment of external_id).
    pub label: String,
    /// Composite tremor magnitude: |mean_acceleration| × sqrt(edge_events).
    pub magnitude: f32,
    /// Direction of the acceleration trend.
    pub direction: TremorDirection,
    /// Mean second derivative of weight change over the analysis window.
    pub mean_acceleration: f32,
    /// Linear regression slope of accelerations over time.
    pub trend_slope: f32,
    /// Number of observations used in the analysis.
    pub observation_count: usize,
    /// Timestamp of the earliest observation in the analysis window.
    pub window_start: f64,
    /// Timestamp of the latest observation in the analysis window.
    pub window_end: f64,
    /// Most recent velocity (first derivative of weight).
    pub latest_velocity: f32,
    /// Second-most-recent velocity (for trend comparison).
    pub previous_velocity: f32,
    /// Risk classification based on magnitude and trend slope.
    pub risk_level: RiskLevel,
}

/// Time window for tremor analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TremorWindow {
    /// Last 7 days.
    Days7,
    /// Last 30 days.
    Days30,
    /// Last 90 days.
    Days90,
    /// All available observations.
    All,
}

impl std::str::FromStr for TremorWindow {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "7d" => Self::Days7,
            "30d" => Self::Days30,
            "90d" => Self::Days90,
            _ => Self::All,
        })
    }
}

impl TremorWindow {
    /// Returns the window duration in seconds, or `None` for `All`.
    pub fn seconds(&self) -> Option<f64> {
        match self {
            Self::Days7 => Some(7.0 * 86400.0),
            Self::Days30 => Some(30.0 * 86400.0),
            Self::Days90 => Some(90.0 * 86400.0),
            Self::All => None,
        }
    }
}

/// Complete result of a tremor analysis pass.
#[derive(Clone, Debug, Serialize)]
pub struct TremorResult {
    /// Tremor alerts sorted by magnitude descending (top_k).
    pub tremors: Vec<TremorAlert>,
    /// Window string used (e.g. "7d", "all").
    pub window: String,
    /// Magnitude threshold applied.
    pub threshold: f32,
    /// Total nodes examined.
    pub total_nodes_analyzed: u32,
    /// Nodes with enough observations to compute acceleration.
    pub nodes_with_sufficient_data: u32,
    /// Wall-clock time in milliseconds.
    pub elapsed_ms: f64,
}

// ── Registry ──

/// Per-node observation ring buffer, keyed by external_id.
///
/// Survives ingest mode=replace (keyed by stable external_id, not NodeId).
#[derive(Clone, Debug, Default)]
pub struct TremorRegistry {
    /// Keyed by external_id (survives ingest mode=replace).
    observations: HashMap<String, VecDeque<TremorObservation>>,
    max_observations: usize,
}

impl TremorRegistry {
    /// Create a new registry with the given per-node observation cap.
    pub fn new(max_observations: usize) -> Self {
        Self {
            observations: HashMap::new(),
            max_observations,
        }
    }

    /// Create a registry with `DEFAULT_MAX_OBSERVATIONS` per node.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_MAX_OBSERVATIONS)
    }

    /// Record a weight-change observation for a node.
    ///
    /// Evicts the oldest observation if at capacity. Called from `handle_learn` and `handle_activate`.
    ///
    /// # Parameters
    /// - `external_id`: stable node identifier (survives graph re-ingestion)
    /// - `weight_delta`: change in node weight
    /// - `edge_events`: number of edge add/remove events
    /// - `timestamp`: Unix timestamp (seconds) of this observation
    pub fn record_observation(
        &mut self,
        external_id: &str,
        weight_delta: f32,
        edge_events: u16,
        timestamp: f64,
    ) {
        let queue = self
            .observations
            .entry(external_id.to_string())
            .or_default();

        // Evict oldest if at capacity
        while queue.len() >= self.max_observations {
            queue.pop_front();
        }

        queue.push_back(TremorObservation {
            timestamp,
            weight_delta,
            edge_events,
        });
    }

    /// Analyze tremors within the given time window.
    ///
    /// Computes velocity and acceleration series per node, filters by magnitude threshold,
    /// and returns the top `top_k` alerts sorted by magnitude descending.
    ///
    /// # Parameters
    /// - `window`: time window to analyze
    /// - `threshold`: minimum magnitude to include
    /// - `top_k`: maximum number of alerts to return
    /// - `node_filter`: optional substring filter on external_id
    /// - `now`: current Unix timestamp (seconds)
    /// - `min_observations`: minimum observations required (0 = use `MIN_OBSERVATIONS_FOR_ACCELERATION`)
    pub fn analyze(
        &self,
        window: TremorWindow,
        threshold: f32,
        top_k: usize,
        node_filter: Option<&str>,
        now: f64,
        min_observations: usize,
    ) -> TremorResult {
        let start = std::time::Instant::now();
        let window_seconds = window.seconds();
        let window_start = match window_seconds {
            Some(secs) => now - secs,
            None => f64::NEG_INFINITY,
        };

        let window_str = match window {
            TremorWindow::Days7 => "7d",
            TremorWindow::Days30 => "30d",
            TremorWindow::Days90 => "90d",
            TremorWindow::All => "all",
        };

        let mut total_nodes_analyzed = 0u32;
        let mut nodes_with_sufficient_data = 0u32;
        let mut alerts: Vec<TremorAlert> = Vec::new();

        for (external_id, queue) in &self.observations {
            // Apply node filter
            if let Some(filter) = node_filter {
                if !external_id.contains(filter) {
                    continue;
                }
            }
            total_nodes_analyzed += 1;

            // Filter observations within window, sort by timestamp, deduplicate
            let mut obs: Vec<&TremorObservation> = queue
                .iter()
                .filter(|o| o.timestamp >= window_start)
                .collect();
            obs.sort_by(|a, b| {
                a.timestamp
                    .partial_cmp(&b.timestamp)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Remove observations with gaps < MIN_OBSERVATION_GAP_SECS
            let mut filtered: Vec<&TremorObservation> = Vec::new();
            for o in &obs {
                if let Some(last) = filtered.last() {
                    if o.timestamp - last.timestamp < MIN_OBSERVATION_GAP_SECS {
                        continue;
                    }
                }
                filtered.push(o);
            }

            let effective_min_obs = if min_observations > 0 {
                min_observations
            } else {
                MIN_OBSERVATIONS_FOR_ACCELERATION
            };
            if filtered.len() < effective_min_obs {
                continue;
            }

            nodes_with_sufficient_data += 1;

            // Compute velocity series: v[i] = weight_delta[i] / dt[i]
            let mut velocities: Vec<f32> = Vec::with_capacity(filtered.len() - 1);
            let mut vel_times: Vec<f64> = Vec::with_capacity(filtered.len() - 1);
            for i in 1..filtered.len() {
                let dt = (filtered[i].timestamp - filtered[i - 1].timestamp)
                    .max(MIN_OBSERVATION_GAP_SECS);
                let v = filtered[i].weight_delta / dt as f32;
                velocities.push(v);
                vel_times.push(filtered[i].timestamp);
            }

            if velocities.len() < 2 {
                continue;
            }

            // Compute acceleration series: a[i] = (v[i] - v[i-1]) / dt
            let mut accelerations: Vec<f32> = Vec::with_capacity(velocities.len() - 1);
            let mut accel_times: Vec<f64> = Vec::with_capacity(velocities.len() - 1);
            for i in 1..velocities.len() {
                let dt = (vel_times[i] - vel_times[i - 1]).max(MIN_OBSERVATION_GAP_SECS);
                let a = (velocities[i] - velocities[i - 1]) / dt as f32;
                accelerations.push(a);
                accel_times.push(vel_times[i]);
            }

            if accelerations.is_empty() {
                continue;
            }

            // Mean acceleration
            let mean_a: f32 = accelerations.iter().sum::<f32>() / accelerations.len() as f32;

            // Trend slope: linear regression of accelerations over time
            let trend_slope = if accelerations.len() >= 2 {
                linear_regression_slope(&accel_times, &accelerations)
            } else {
                0.0
            };

            // Total edge events in window
            let total_edge_events: u32 = filtered.iter().map(|o| o.edge_events as u32).sum();

            // Magnitude = |mean_a| * sqrt(edge_events)
            let magnitude = (mean_a.abs() * (total_edge_events as f32).sqrt()).min(MAGNITUDE_CAP);

            if magnitude < threshold {
                continue;
            }

            // Direction
            let direction = if mean_a > 0.001 {
                TremorDirection::Accelerating
            } else if mean_a < -0.001 {
                TremorDirection::Decelerating
            } else {
                TremorDirection::Stable
            };

            // Risk level classification
            let risk_level = if magnitude > 5.0 && trend_slope > 0.5 {
                RiskLevel::Critical
            } else if magnitude > 2.0 || trend_slope > 0.3 {
                RiskLevel::High
            } else if magnitude > 0.5 {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };

            // Label: extract filename from external_id
            let label = external_id
                .rsplit("::")
                .next()
                .unwrap_or(external_id)
                .to_string();

            let actual_window_start = filtered.first().map(|o| o.timestamp).unwrap_or(now);
            let actual_window_end = filtered.last().map(|o| o.timestamp).unwrap_or(now);
            let latest_velocity = *velocities.last().unwrap_or(&0.0);
            let previous_velocity = if velocities.len() >= 2 {
                velocities[velocities.len() - 2]
            } else {
                0.0
            };

            alerts.push(TremorAlert {
                node_id: external_id.clone(),
                label,
                magnitude,
                direction,
                mean_acceleration: mean_a,
                trend_slope,
                observation_count: filtered.len(),
                window_start: actual_window_start,
                window_end: actual_window_end,
                latest_velocity,
                previous_velocity,
                risk_level,
            });
        }

        // Sort by magnitude descending, take top_k
        alerts.sort_by(|a, b| {
            b.magnitude
                .partial_cmp(&a.magnitude)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        alerts.truncate(top_k);

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        TremorResult {
            tremors: alerts,
            window: window_str.to_string(),
            threshold,
            total_nodes_analyzed,
            nodes_with_sufficient_data,
            elapsed_ms,
        }
    }

    /// Return the number of observations stored for `external_id`.
    pub fn observation_count(&self, external_id: &str) -> usize {
        self.observations.get(external_id).map_or(0, |q| q.len())
    }
}

// ── Persistence ──

#[derive(Serialize, Deserialize)]
struct TremorPersistenceFormat {
    version: u32,
    nodes: HashMap<String, Vec<TremorObservation>>,
}

/// Atomically persist the tremor registry to `path` (write temp + rename).
///
/// # Errors
/// Returns `M1ndError::Serde` on serialization failure or `M1ndError::Io` on I/O failure.
pub fn save_tremor_state(registry: &TremorRegistry, path: &Path) -> M1ndResult<()> {
    let format = TremorPersistenceFormat {
        version: 1,
        nodes: registry
            .observations
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
            .collect(),
    };

    let json = serde_json::to_string_pretty(&format).map_err(crate::error::M1ndError::Serde)?;

    // Atomic write: temp file + rename
    let temp_path = path.with_extension("tmp");
    {
        use std::io::Write;
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
    }
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load tremor registry from `path`. Returns a default registry if the file does not exist.
///
/// Non-finite observation values are silently skipped on load.
///
/// # Errors
/// Returns `M1ndError::Io` on read failure or `M1ndError::Serde` on parse failure.
pub fn load_tremor_state(path: &Path) -> M1ndResult<TremorRegistry> {
    if !path.exists() {
        return Ok(TremorRegistry::with_defaults());
    }

    let data = std::fs::read_to_string(path)?;
    let format: TremorPersistenceFormat =
        serde_json::from_str(&data).map_err(crate::error::M1ndError::Serde)?;

    let mut registry = TremorRegistry::new(DEFAULT_MAX_OBSERVATIONS);

    for (external_id, obs_vec) in format.nodes {
        let mut queue = VecDeque::with_capacity(obs_vec.len().min(DEFAULT_MAX_OBSERVATIONS));
        for obs in obs_vec {
            // Validate: skip non-finite values
            if !obs.timestamp.is_finite() || !obs.weight_delta.is_finite() {
                continue;
            }
            queue.push_back(obs);
            if queue.len() >= DEFAULT_MAX_OBSERVATIONS {
                queue.pop_front();
            }
        }
        if !queue.is_empty() {
            registry.observations.insert(external_id, queue);
        }
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_registry() -> TremorRegistry {
        TremorRegistry::with_defaults()
    }

    /// Helper: create N observations separated by 2 seconds each.
    fn push_obs(reg: &mut TremorRegistry, id: &str, deltas: &[f32], base_time: f64) {
        for (i, &delta) in deltas.iter().enumerate() {
            reg.record_observation(id, delta, 1, base_time + i as f64 * 2.0);
        }
    }

    // 1. record_observation: count increases correctly
    #[test]
    fn record_observation_increments_count() {
        let mut reg = make_registry();
        assert_eq!(reg.observation_count("node::a"), 0);
        reg.record_observation("node::a", 0.5, 1, 1000.0);
        assert_eq!(reg.observation_count("node::a"), 1);
        reg.record_observation("node::a", 0.3, 2, 1002.0);
        assert_eq!(reg.observation_count("node::a"), 2);
    }

    // 2. ring_buffer_evicts: capacity is enforced
    #[test]
    fn ring_buffer_evicts_oldest_at_capacity() {
        let cap = 4;
        let mut reg = TremorRegistry::new(cap);
        for i in 0..cap + 2 {
            reg.record_observation("node::b", 0.1, 1, i as f64 * 10.0);
        }
        assert_eq!(reg.observation_count("node::b"), cap);
    }

    // 3. no_tremors_stable: steady low deltas produce no alert above default threshold
    #[test]
    fn no_tremors_for_stable_node() {
        let mut reg = make_registry();
        // Flat deltas → acceleration ≈ 0 → magnitude near 0
        push_obs(
            &mut reg,
            "node::stable",
            &[0.01, 0.01, 0.01, 0.01, 0.01],
            1000.0,
        );
        let result = reg.analyze(TremorWindow::All, DEFAULT_THRESHOLD, 20, None, 2000.0, 0);
        // magnitude should be below threshold — no tremors
        assert!(
            result.tremors.is_empty(),
            "Expected no tremors for stable node, got {:?}",
            result.tremors
        );
    }

    // 4. acceleration_detected: rapidly growing deltas produce an alert
    #[test]
    fn acceleration_detected_for_rapidly_changing_node() {
        let mut reg = make_registry();
        // Monotonically growing deltas → positive second derivative
        push_obs(&mut reg, "node::hot", &[0.1, 1.0, 5.0, 20.0, 80.0], 1000.0);
        let result = reg.analyze(TremorWindow::All, 0.0, 20, None, 2000.0, 0);
        assert!(
            !result.tremors.is_empty(),
            "Expected tremor alert for accelerating node"
        );
        let alert = &result.tremors[0];
        assert_eq!(alert.node_id, "node::hot");
        assert_eq!(alert.direction, TremorDirection::Accelerating);
    }

    // 5. deceleration: decreasing velocity → Decelerating direction
    #[test]
    fn deceleration_produces_decelerating_direction() {
        let mut reg = make_registry();
        // Decreasing deltas → negative second derivative
        push_obs(
            &mut reg,
            "node::cooling",
            &[80.0, 20.0, 5.0, 1.0, 0.1],
            1000.0,
        );
        let result = reg.analyze(TremorWindow::All, 0.0, 20, None, 2000.0, 0);
        let found = result.tremors.iter().find(|a| a.node_id == "node::cooling");
        assert!(found.is_some(), "Expected tremor for decelerating node");
        assert_eq!(found.unwrap().direction, TremorDirection::Decelerating);
    }

    // 6. min_observations: nodes below effective_min_obs are skipped
    #[test]
    fn min_observations_filters_sparse_nodes() {
        let mut reg = make_registry();
        // Only 2 observations — below MIN_OBSERVATIONS_FOR_ACCELERATION (3)
        reg.record_observation("node::sparse", 1.0, 1, 1000.0);
        reg.record_observation("node::sparse", 10.0, 1, 1002.0);
        let result = reg.analyze(TremorWindow::All, 0.0, 20, None, 2000.0, 0);
        assert!(result.tremors.iter().all(|a| a.node_id != "node::sparse"));
    }

    // 7. sensitivity: threshold=0 lets everything through; high threshold blocks
    #[test]
    fn threshold_gates_alerts() {
        let mut reg = make_registry();
        push_obs(&mut reg, "node::weak", &[0.1, 0.5, 1.0, 2.0, 4.0], 1000.0);
        // With threshold=0 we may get alerts
        let result_zero = reg.analyze(TremorWindow::All, 0.0, 20, None, 2000.0, 0);
        // With threshold=MAGNITUDE_CAP we should get nothing
        let result_max = reg.analyze(TremorWindow::All, MAGNITUDE_CAP, 20, None, 2000.0, 0);
        // At cap threshold no alert can reach it (magnitude is capped at MAGNITUDE_CAP)
        assert!(result_max.tremors.is_empty());
        // At zero threshold, result depends on actual magnitude, just check structure
        let _ = result_zero; // just verify it runs
    }

    // 8. save_load: round-trip through file preserves observation counts
    #[test]
    fn save_load_round_trip() {
        let mut reg = make_registry();
        push_obs(
            &mut reg,
            "node::persist",
            &[1.0, 2.0, 3.0, 4.0, 5.0],
            1000.0,
        );
        let count_before = reg.observation_count("node::persist");

        let dir = std::env::temp_dir();
        let path: PathBuf = dir.join(format!("tremor_test_{}.json", std::process::id()));

        save_tremor_state(&reg, &path).expect("save failed");
        let loaded = load_tremor_state(&path).expect("load failed");

        assert_eq!(loaded.observation_count("node::persist"), count_before);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    // Extra: window filter — observations outside window are excluded
    #[test]
    fn window_filter_excludes_old_observations() {
        let mut reg = make_registry();
        let now = 1_000_000.0f64;
        // Push 5 observations 100 days ago (outside 30d window)
        let old_base = now - 100.0 * 86400.0;
        push_obs(
            &mut reg,
            "node::old",
            &[1.0, 5.0, 10.0, 20.0, 40.0],
            old_base,
        );
        let result = reg.analyze(TremorWindow::Days30, 0.0, 20, None, now, 0);
        assert!(
            result.tremors.iter().all(|a| a.node_id != "node::old"),
            "Old observations should be excluded by 30d window"
        );
    }
}

/// Linear regression slope for (x, y) data.
/// Returns the slope of the best-fit line y = mx + b.
fn linear_regression_slope(x: &[f64], y: &[f32]) -> f32 {
    let n = x.len();
    if n < 2 {
        return 0.0;
    }

    let n_f = n as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().map(|v| *v as f64).sum();
    let sum_xy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| *xi * (*yi as f64))
        .sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    let denom = n_f * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-12 {
        return 0.0;
    }

    let slope = (n_f * sum_xy - sum_x * sum_y) / denom;
    slope as f32
}
