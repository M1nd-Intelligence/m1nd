// === crates/m1nd-mcp/src/session.rs ===

use m1nd_core::domain::DomainConfig;
use m1nd_core::error::M1ndResult;
use m1nd_core::graph::{Graph, SharedGraph};
use m1nd_core::query::QueryOrchestrator;
use m1nd_core::temporal::TemporalEngine;
use m1nd_core::counterfactual::CounterfactualEngine;
use m1nd_core::topology::TopologyAnalyzer;
use m1nd_core::resonance::ResonanceEngine;
use m1nd_core::plasticity::PlasticityEngine;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

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

        self.last_persist_time = Some(Instant::now());
        Ok(())
    }

    /// Rebuild all engines after graph replacement (e.g. after ingest).
    /// Critical: SemanticEngine indexes, TemporalEngine, PlasticityEngine
    /// are all built from graph state and become stale on graph swap.
    pub fn rebuild_engines(&mut self) -> M1ndResult<()> {
        let graph = self.graph.read();
        self.orchestrator = QueryOrchestrator::build(&graph)?;
        self.temporal = TemporalEngine::build(&graph)?;
        self.plasticity = PlasticityEngine::new(
            &graph,
            m1nd_core::plasticity::PlasticityConfig::default(),
        );
        Ok(())
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
