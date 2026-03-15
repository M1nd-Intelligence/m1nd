// === crates/m1nd-mcp/src/session.rs ===

use m1nd_core::antibody::Antibody;
use m1nd_core::domain::DomainConfig;
use m1nd_core::error::M1ndResult;
use m1nd_core::graph::{Graph, SharedGraph};
use m1nd_core::query::QueryOrchestrator;
use m1nd_core::temporal::TemporalEngine;
use m1nd_core::counterfactual::CounterfactualEngine;
use m1nd_core::topology::TopologyAnalyzer;
use m1nd_core::resonance::ResonanceEngine;
use m1nd_core::plasticity::PlasticityEngine;
use m1nd_core::tremor::TremorRegistry;
use m1nd_core::trust::TrustLedger;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::perspective::state::{
    LockState, PerspectiveLimits, PerspectiveState, WatchTrigger, WatcherEvent,
    PeekSecurityConfig,
};

// ---------------------------------------------------------------------------
// AgentSession — per-agent session tracking
// ---------------------------------------------------------------------------

/// Lightweight session record for a connected agent.
pub struct AgentSession {
    pub agent_id: String,
    pub first_seen: Instant,
    pub last_seen: Instant,
    pub query_count: u64,
}

// ---------------------------------------------------------------------------
// SavingsTracker — tracks estimated token savings from m1nd usage
// ---------------------------------------------------------------------------

/// Tracks estimated token savings from using m1nd instead of grep/Read.
pub struct SavingsTracker {
    pub queries_by_tool: HashMap<String, u64>,
    pub tokens_saved: u64,
    pub file_reads_avoided: u64,
    pub lines_avoided: u64,
}

impl SavingsTracker {
    pub fn new() -> Self {
        Self {
            queries_by_tool: HashMap::new(),
            tokens_saved: 0,
            file_reads_avoided: 0,
            lines_avoided: 0,
        }
    }

    /// Call after every successful tool dispatch.
    pub fn record(&mut self, tool: &str, _result_nodes: usize) {
        *self.queries_by_tool.entry(tool.to_string()).or_insert(0) += 1;
        let (tokens, files, lines) = match tool {
            "m1nd.activate" | "m1nd.seek" | "m1nd.search" => (750, 5, 500),
            "m1nd.impact" | "m1nd.predict" | "m1nd.counterfactual" => (1000, 8, 800),
            "m1nd.surgical.context" => (3200, 8, 300),
            "m1nd.surgical.context.v2" => (4800, 12, 400),
            "m1nd.hypothesize" | "m1nd.missing" => (1000, 5, 200),
            "m1nd.apply" | "m1nd.apply.batch" => (900, 3, 200),
            "m1nd.scan" => (1000, 4, 400),
            _ => (500, 2, 200),
        };
        self.tokens_saved += tokens;
        self.file_reads_avoided += files;
        self.lines_avoided += lines;
    }
}

// ---------------------------------------------------------------------------
// QueryLogEntry — ring buffer entry for report/savings
// ---------------------------------------------------------------------------

/// A log entry for each tool call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryLogEntry {
    pub tool: String,
    pub agent_id: String,
    pub timestamp_ms: u64,
    pub elapsed_ms: f64,
    pub result_count: usize,
    pub query_preview: String,
}

/// Global savings state, persisted to disk.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GlobalSavingsState {
    pub total_sessions: u64,
    pub total_queries: u64,
    pub total_tokens_saved: u64,
    pub total_file_reads_avoided: u64,
}

// ---------------------------------------------------------------------------
// SessionState — all server state in one place
// Replaces: 03-MCP Section 1.1 server internal state
// ---------------------------------------------------------------------------

/// Server session state. Owns the graph and all engine instances.
/// Single instance shared across all agent connections.
pub struct SessionState {
    /// Shared graph with RwLock for concurrent read access.
    pub graph: SharedGraph,
    /// Domain configuration (code, music, generic, etc.)
    pub domain: DomainConfig,
    /// Query orchestrator (owns HybridEngine, XLR, Semantic, etc.)
    pub orchestrator: QueryOrchestrator,
    /// Temporal engine (co-change, causal chains, decay, velocity, impact).
    pub temporal: TemporalEngine,
    /// Counterfactual engine.
    pub counterfactual: CounterfactualEngine,
    /// Topology analyzer.
    pub topology: TopologyAnalyzer,
    /// Resonance engine.
    pub resonance: ResonanceEngine,
    /// Plasticity engine.
    pub plasticity: PlasticityEngine,
    /// Query counter for auto-persist.
    pub queries_processed: u64,
    /// Auto-persist interval (persist every N queries).
    pub auto_persist_interval: u32,
    /// Server start time.
    pub start_time: Instant,
    /// Last persistence timestamp.
    pub last_persist_time: Option<Instant>,
    /// Path to graph snapshot file.
    pub graph_path: PathBuf,
    /// Path to plasticity state file.
    pub plasticity_path: PathBuf,
    /// Per-agent session tracking.
    pub sessions: HashMap<String, AgentSession>,

    // --- Perspective MCP state (12-PERSPECTIVE-SYNTHESIS) ---

    /// Generation counter: bumped on ingest, rebuild_engines (Theme 1).
    pub graph_generation: u64,
    /// Generation counter: bumped on learn (Theme 1).
    pub plasticity_generation: u64,
    /// Unified cache generation: max(graph_gen, plasticity_gen). Bumped on ALL mutations (Theme 1).
    pub cache_generation: u64,

    /// Perspective state per (agent_id, perspective_id) (Theme 2).
    pub perspectives: HashMap<(String, String), PerspectiveState>,
    /// Lock state per lock_id (Theme 2).
    pub locks: HashMap<String, LockState>,
    /// Per-agent monotonic counter for perspective IDs (Theme 2).
    pub perspective_counter: HashMap<String, u64>,
    /// Per-agent monotonic counter for lock IDs (Theme 2).
    pub lock_counter: HashMap<String, u64>,

    /// Pending watcher events queue (Theme 10).
    pub pending_watcher_events: Vec<WatcherEvent>,

    /// Hard caps for perspective/lock resources (Theme 5).
    pub perspective_limits: PerspectiveLimits,

    /// Peek security configuration (Theme 6).
    pub peek_security: PeekSecurityConfig,

    /// Ingest root paths for peek allow-list (Theme 6).
    pub ingest_roots: Vec<String>,

    // --- Superpowers: Antibody state ---

    /// All stored antibodies.
    pub antibodies: Vec<Antibody>,
    /// Path to antibodies persistence file.
    pub antibodies_path: PathBuf,
    /// Generation at last antibody scan (for "changed" scope).
    pub last_antibody_scan_generation: u64,

    // --- Superpowers: Tremor + Trust state ---

    /// Tremor registry: per-node time series of weight-change observations.
    pub tremor_registry: TremorRegistry,
    /// Path to tremor_state.json persistence file.
    pub tremor_path: PathBuf,
    /// Trust ledger: per-node actuarial defect records.
    pub trust_ledger: TrustLedger,
    /// Path to trust_state.json persistence file.
    pub trust_path: PathBuf,

    // --- v0.4.0: Savings + Query Log ---

    /// Savings tracker (token economy).
    pub savings_tracker: SavingsTracker,
    /// Query log ring buffer (capped at 1000 entries).
    pub query_log: Vec<QueryLogEntry>,
    /// Global savings state (persisted).
    pub global_savings: GlobalSavingsState,
    /// Path to savings_state.json persistence file.
    pub savings_path: PathBuf,
    /// Graph node count at session start.
    pub session_start_node_count: u32,
    /// Graph edge count at session start.
    pub session_start_edge_count: u64,
}

impl SessionState {
    /// Initialize from a loaded graph. Builds all engines.
    /// Replaces: 03-MCP Section 1.2 startup sequence steps 3-6.
    pub fn initialize(graph: Graph, config: &crate::server::McpConfig, domain: DomainConfig) -> M1ndResult<Self> {
        // Build all engines from graph
        let orchestrator = QueryOrchestrator::build(&graph)?;
        let temporal = TemporalEngine::build(&graph)?;
        let counterfactual = CounterfactualEngine::with_defaults();
        let topology = TopologyAnalyzer::with_defaults();
        let resonance = ResonanceEngine::with_defaults();
        let plasticity = PlasticityEngine::new(
            &graph,
            m1nd_core::plasticity::PlasticityConfig::default(),
        );

        let shared = Arc::new(parking_lot::RwLock::new(graph));

        Ok(Self {
            graph: shared,
            domain,
            orchestrator,
            temporal,
            counterfactual,
            topology,
            resonance,
            plasticity,
            queries_processed: 0,
            auto_persist_interval: config.auto_persist_interval,
            start_time: Instant::now(),
            last_persist_time: None,
            graph_path: config.graph_source.clone(),
            plasticity_path: config.plasticity_state.clone(),
            sessions: HashMap::new(),
            // Perspective MCP state
            graph_generation: 0,
            plasticity_generation: 0,
            cache_generation: 0,
            perspectives: HashMap::new(),
            locks: HashMap::new(),
            perspective_counter: HashMap::new(),
            lock_counter: HashMap::new(),
            pending_watcher_events: Vec::new(),
            perspective_limits: PerspectiveLimits::default(),
            peek_security: PeekSecurityConfig::default(),
            ingest_roots: Vec::new(),
            // Superpowers: Antibody state
            antibodies: {
                let ab_path = config.graph_source.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("antibodies.json");
                m1nd_core::antibody::load_antibodies(&ab_path).unwrap_or_default()
            },
            antibodies_path: config.graph_source.parent()
                .unwrap_or(std::path::Path::new("."))
                .join("antibodies.json"),
            last_antibody_scan_generation: 0,
            // Superpowers: Tremor + Trust state
            tremor_registry: {
                let tr_path = config.graph_source.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("tremor_state.json");
                m1nd_core::tremor::load_tremor_state(&tr_path).unwrap_or_else(|_| TremorRegistry::with_defaults())
            },
            tremor_path: config.graph_source.parent()
                .unwrap_or(std::path::Path::new("."))
                .join("tremor_state.json"),
            trust_ledger: {
                let tl_path = config.graph_source.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("trust_state.json");
                m1nd_core::trust::load_trust_state(&tl_path).unwrap_or_else(|_| TrustLedger::new())
            },
            trust_path: config.graph_source.parent()
                .unwrap_or(std::path::Path::new("."))
                .join("trust_state.json"),
            // v0.4.0: Savings + Query Log
            savings_tracker: SavingsTracker::new(),
            query_log: Vec::new(),
            global_savings: {
                let sv_path = config.graph_source.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("savings_state.json");
                std::fs::read_to_string(&sv_path).ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            },
            savings_path: config.graph_source.parent()
                .unwrap_or(std::path::Path::new("."))
                .join("savings_state.json"),
            session_start_node_count: 0,
            session_start_edge_count: 0,
        })
    }

    /// Check if auto-persist should trigger. Returns true every N queries.
    pub fn should_persist(&self) -> bool {
        self.queries_processed > 0
            && self.queries_processed % self.auto_persist_interval as u64 == 0
    }

    /// Persist all state to disk.
    ///
    /// Ordering: graph first (source of truth), then plasticity.
    /// If graph save fails, skip plasticity to avoid inconsistent state.
    /// If plasticity save fails after graph succeeds, log warning but don't crash.
    pub fn persist(&mut self) -> M1ndResult<()> {
        let graph = self.graph.read();

        // Graph is the source of truth — save it first.
        m1nd_core::snapshot::save_graph(&graph, &self.graph_path)?;

        // Graph succeeded. Now try plasticity — failure here is non-fatal.
        match self.plasticity.export_state(&graph) {
            Ok(states) => {
                if let Err(e) = m1nd_core::snapshot::save_plasticity_state(&states, &self.plasticity_path) {
                    eprintln!("[m1nd] WARNING: graph saved but plasticity persist failed: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[m1nd] WARNING: graph saved but plasticity export failed: {}", e);
            }
        }

        // Antibodies — failure here is non-fatal.
        if !self.antibodies.is_empty() {
            if let Err(e) = m1nd_core::antibody::save_antibodies(&self.antibodies, &self.antibodies_path) {
                eprintln!("[m1nd] WARNING: antibody persist failed: {}", e);
            }
        }

        self.last_persist_time = Some(Instant::now());
        Ok(())
    }

    /// Rebuild all engines after graph replacement (e.g. after ingest).
    /// Critical: SemanticEngine indexes, TemporalEngine, PlasticityEngine
    /// are all built from graph state and become stale on graph swap.
    ///
    /// Also invalidates all perspective and lock state (Theme 16).
    pub fn rebuild_engines(&mut self) -> M1ndResult<()> {
        // Scope the read lock so it's dropped before &mut self methods
        {
            let graph = self.graph.read();
            self.orchestrator = QueryOrchestrator::build(&graph)?;
            self.temporal = TemporalEngine::build(&graph)?;
            self.plasticity = PlasticityEngine::new(
                &graph,
                m1nd_core::plasticity::PlasticityConfig::default(),
            );
        }

        // Theme 16: invalidate all perspective and lock state after rebuild
        self.invalidate_all_perspectives();
        self.mark_all_lock_baselines_stale();
        self.graph_generation += 1;
        self.cache_generation = self.cache_generation.max(self.graph_generation);

        Ok(())
    }

    // --- Perspective MCP methods (12-PERSPECTIVE-SYNTHESIS) ---

    /// Bump graph generation (Theme 1). Called after ingest and rebuild_engines.
    pub fn bump_graph_generation(&mut self) {
        self.graph_generation += 1;
        self.cache_generation = self.cache_generation.max(self.graph_generation);
    }

    /// Bump plasticity generation (Theme 1). Called after learn.
    pub fn bump_plasticity_generation(&mut self) {
        self.plasticity_generation += 1;
        self.cache_generation = self.cache_generation.max(self.plasticity_generation);
    }

    /// Invalidate all perspectives (Theme 16).
    /// Sets stale=true, clears route caches, bumps route_set_version.
    /// Does NOT close perspectives — agents may still want them.
    pub fn invalidate_all_perspectives(&mut self) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        for state in self.perspectives.values_mut() {
            state.stale = true;
            state.route_cache = None;
            state.route_set_version = now_ms;
        }
    }

    /// Mark all lock baselines as stale (Theme 16).
    /// Does NOT release locks. lock.diff reports staleness and suggests lock.rebase.
    pub fn mark_all_lock_baselines_stale(&mut self) {
        for lock in self.locks.values_mut() {
            lock.baseline_stale = true;
        }
    }

    /// Get a perspective for an agent (Theme 2).
    pub fn get_perspective(&self, agent_id: &str, perspective_id: &str) -> Option<&PerspectiveState> {
        self.perspectives.get(&(agent_id.to_string(), perspective_id.to_string()))
    }

    /// Get a mutable perspective for an agent (Theme 2).
    pub fn get_perspective_mut(&mut self, agent_id: &str, perspective_id: &str) -> Option<&mut PerspectiveState> {
        self.perspectives.get_mut(&(agent_id.to_string(), perspective_id.to_string()))
    }

    /// Generate a new perspective ID for an agent (Theme 2).
    pub fn next_perspective_id(&mut self, agent_id: &str) -> String {
        let counter = self.perspective_counter.entry(agent_id.to_string()).or_insert(0);
        *counter += 1;
        let short_id = &agent_id[..agent_id.len().min(8)];
        format!("persp_{}_{:03}", short_id, counter)
    }

    /// Generate a new lock ID for an agent (Theme 2).
    pub fn next_lock_id(&mut self, agent_id: &str) -> String {
        let counter = self.lock_counter.entry(agent_id.to_string()).or_insert(0);
        *counter += 1;
        let short_id = &agent_id[..agent_id.len().min(8)];
        format!("lock_{}_{:03}", short_id, counter)
    }

    /// Count perspectives for an agent (for limit enforcement, Theme 5).
    pub fn agent_perspective_count(&self, agent_id: &str) -> usize {
        self.perspectives.keys().filter(|(a, _)| a == agent_id).count()
    }

    /// Count locks for an agent (for limit enforcement, Theme 5).
    pub fn agent_lock_count(&self, agent_id: &str) -> usize {
        self.locks.values().filter(|l| l.agent_id == agent_id).count()
    }

    /// Notify watchers after ingest/learn (Theme 10).
    /// Records (lock_id, trigger, timestamp) in pending_watcher_events.
    /// Diff computed lazily on next lock.diff call.
    pub fn notify_watchers(&mut self, trigger: WatchTrigger) {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let matching_locks: Vec<String> = self.locks.values()
            .filter(|l| {
                l.watcher.as_ref().map_or(false, |w| match (&trigger, &w.strategy) {
                    (WatchTrigger::Ingest, crate::perspective::state::WatchStrategy::OnIngest) => true,
                    (WatchTrigger::Learn, crate::perspective::state::WatchStrategy::OnLearn) => true,
                    _ => false,
                })
            })
            .map(|l| l.lock_id.clone())
            .collect();

        for lock_id in matching_locks {
            self.pending_watcher_events.push(WatcherEvent {
                lock_id,
                trigger: trigger.clone(),
                timestamp_ms: now_ms,
            });
        }
    }

    /// Cleanup all state for an agent (called on session timeout, Theme 2).
    pub fn cleanup_agent_state(&mut self, agent_id: &str) {
        // Remove perspectives
        self.perspectives.retain(|(a, _), _| a != agent_id);
        // Remove locks owned by this agent
        let agent_locks: Vec<String> = self.locks.values()
            .filter(|l| l.agent_id == agent_id)
            .map(|l| l.lock_id.clone())
            .collect();
        for lock_id in &agent_locks {
            self.locks.remove(lock_id);
        }
        // Clean pending watcher events for removed locks
        self.pending_watcher_events.retain(|e| !agent_locks.contains(&e.lock_id));
        // Clean counters
        self.perspective_counter.remove(agent_id);
        self.lock_counter.remove(agent_id);
    }

    /// Estimate memory usage of perspective + lock state (Theme 5).
    /// Used for 50MB budget enforcement.
    pub fn perspective_and_lock_memory_bytes(&self) -> usize {
        // Rough estimate: serialize to JSON and measure
        let persp_size: usize = self.perspectives.values()
            .map(|p| std::mem::size_of_val(p) + p.navigation_history.len() * 100 + p.visited_nodes.len() * 40)
            .sum();
        let lock_size: usize = self.locks.values()
            .map(|l| std::mem::size_of_val(l) + l.baseline.nodes.len() * 40 + l.baseline.edges.len() * 120)
            .sum();
        persp_size + lock_size
    }

    /// Uptime in seconds.
    pub fn uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Track an agent session. Creates a new session if first contact,
    /// otherwise updates last_seen and increments query_count.
    pub fn track_agent(&mut self, agent_id: &str) {
        let now = Instant::now();
        let session = self.sessions.entry(agent_id.to_string()).or_insert_with(|| {
            AgentSession {
                agent_id: agent_id.to_string(),
                first_seen: now,
                last_seen: now,
                query_count: 0,
            }
        });
        session.last_seen = now;
        session.query_count += 1;
    }

    /// Log a tool call to the query log ring buffer (max 1000 entries).
    pub fn log_query(&mut self, tool: &str, agent_id: &str, elapsed_ms: f64, result_count: usize, query_preview: &str) {
        let entry = QueryLogEntry {
            tool: tool.to_string(),
            agent_id: agent_id.to_string(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            elapsed_ms,
            result_count,
            query_preview: query_preview.chars().take(100).collect(),
        };
        if self.query_log.len() >= 1000 {
            self.query_log.remove(0);
        }
        self.query_log.push(entry);
    }

    /// Persist global savings state to disk.
    pub fn persist_savings(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.global_savings) {
            let _ = std::fs::write(&self.savings_path, json);
        }
    }

    /// Generate a summary of active agent sessions for health output.
    pub fn session_summary(&self) -> Vec<serde_json::Value> {
        self.sessions
            .values()
            .map(|s| {
                serde_json::json!({
                    "agent_id": s.agent_id,
                    "first_seen_secs_ago": s.first_seen.elapsed().as_secs_f64(),
                    "last_seen_secs_ago": s.last_seen.elapsed().as_secs_f64(),
                    "query_count": s.query_count,
                })
            })
            .collect()
    }
}
