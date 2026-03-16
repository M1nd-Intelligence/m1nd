// === crates/m1nd-mcp/src/server.rs ===

use crate::layer_handlers;
use crate::personality;
use crate::protocol::layers;
use crate::protocol::*;
use crate::report_handlers;
use crate::search_handlers;
use crate::session::SessionState;
use crate::surgical_handlers;
use crate::tools;
use m1nd_core::domain::DomainConfig;
use m1nd_core::error::{M1ndError, M1ndResult};
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// MCP protocol instructions — injected into initialize response so agents
// automatically understand how to use m1nd effectively.
// ---------------------------------------------------------------------------
const M1ND_INSTRUCTIONS: &str = "\
m1nd is a neuro-symbolic code graph engine. It ingests codebases into a weighted \
graph and provides spreading-activation queries, impact analysis, prediction, and \
stateful perspective navigation. All tool calls require an `agent_id` parameter.

## WORKFLOWS

**Session Start**: `health` → `drift` (recover what changed since last session) → \
`ingest` (if graph is empty or stale). This gives you codebase-aware context.

**Research**: `ingest` → `activate(query)` → `why(source, target)` → `missing(topic)` → \
`learn(feedback)`. Use `seek` for keyword search, `scan` for broad discovery, \
`trace` for dependency chains, `timeline` for temporal ordering.

**Code Change**: `impact(node)` (blast radius) → `predict(node)` (co-change likelihood) → \
`counterfactual(nodes)` (simulate removal) → `warmup(task_description)` (prime graph). \
Use `differential` to compare two subgraphs. Use `hypothesize` to test what-if scenarios.

**Deep Analysis**: `resonate(query)` for standing-wave harmonic patterns. \
`fingerprint(nodes)` for duplicate/equivalence detection. `diverge(node)` for \
exploring unexpected connections. `federate` to query across graph namespaces.

## PERSPECTIVE SYSTEM (stateful navigation)

Perspectives are named, agent-scoped navigation sessions through the graph. \
Flow: `perspective_start(name, seed_nodes)` → `perspective_follow(node)` (move focus) → \
`perspective_branch(name)` (fork exploration) → `perspective_back` (undo last move) → \
`perspective_close`. Use `perspective_inspect` to see current state, `perspective_peek` \
to look at a node without moving, `perspective_list` for all open perspectives, \
`perspective_compare` to diff two perspectives, `perspective_suggest` for next-step \
recommendations, `perspective_routes` for paths between nodes, `perspective_affinity` \
for related-node scoring.

## CONCURRENCY & STATE

`lock_create` / `lock_release` — advisory locks for multi-agent coordination on graph \
regions. `lock_watch` monitors lock state. `lock_diff` shows changes within a lock scope. \
`lock_rebase` replays external changes into a locked region. \
`trail_save` / `trail_list` / `trail_resume` / `trail_merge` — persist and restore \
exploration trails across sessions. `validate_plan` checks a proposed multi-step plan \
for structural soundness.

## CRITICAL PATTERNS

1. **Always call `learn` after using `activate` results.** Feedback (correct/wrong/partial) \
trains the graph weights via Hebbian learning. Skipping this degrades future queries.
2. **Use `ingest` at session start** if the graph has zero nodes or the codebase changed.
3. **Use `drift` to recover context** between sessions — it shows weight changes since \
a baseline timestamp.
4. **`warmup` before focused work** — primes activation patterns for a specific task, \
making subsequent queries faster and more relevant.
5. **Never call `activate` without `agent_id`** — multi-agent isolation depends on it.
6. **Prefer `impact` over `activate` for code changes** — impact gives directional \
blast-radius analysis; activate gives associative exploration.
7. **Graph persists automatically** every 50 queries and on shutdown. Use `trail_save` \
for explicit exploration checkpoints.
";

#[derive(Clone, Copy, Debug)]
enum TransportMode {
    Framed,
    Line,
}

fn read_request_payload<R: BufRead>(
    reader: &mut R,
) -> std::io::Result<Option<(String, TransportMode)>> {
    loop {
        let buffer = reader.fill_buf()?;
        if buffer.is_empty() {
            return Ok(None);
        }

        let first_non_ws = buffer
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace());
        let starts_framed = matches!(first_non_ws, Some(byte) if byte != b'{' && byte != b'[');
        if starts_framed {
            let mut content_length: Option<usize> = None;
            loop {
                let mut header_line = String::new();
                let bytes = reader.read_line(&mut header_line)?;
                if bytes == 0 {
                    return Ok(None);
                }
                let trimmed = header_line.trim_end_matches(['\r', '\n']);
                if trimmed.is_empty() {
                    break;
                }
                if let Some((name, value)) = trimmed.split_once(':') {
                    if name.trim().eq_ignore_ascii_case("Content-Length") {
                        content_length = value.trim().parse::<usize>().ok();
                    }
                }
            }

            let length = content_length.ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Missing Content-Length header",
                )
            })?;
            let mut body = vec![0_u8; length];
            reader.read_exact(&mut body)?;
            let payload = String::from_utf8(body)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
            return Ok(Some((payload, TransportMode::Framed)));
        }

        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        return Ok(Some((trimmed.to_owned(), TransportMode::Line)));
    }
}

fn write_response<W: Write>(
    writer: &mut W,
    response: &JsonRpcResponse,
    mode: TransportMode,
) -> std::io::Result<()> {
    let json = serde_json::to_string(response).unwrap_or_default();
    match mode {
        TransportMode::Framed => {
            write!(writer, "Content-Length: {}\r\n\r\n{}", json.len(), json)?;
        }
        TransportMode::Line => {
            writeln!(writer, "{}", json)?;
        }
    }
    writer.flush()
}

// ---------------------------------------------------------------------------
// McpConfig — server configuration
// Replaces: 03-MCP Section 1.2 initialization config
// ---------------------------------------------------------------------------

/// MCP server configuration.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct McpConfig {
    pub graph_source: PathBuf,
    pub plasticity_state: PathBuf,
    pub auto_persist_interval: u32,
    pub learning_rate: f32,
    pub decay_rate: f32,
    pub xlr_enabled: bool,
    pub max_concurrent_reads: usize,
    pub write_queue_size: usize,
    /// Domain name: "code" (default), "music", or "generic".
    /// Controls temporal decay half-lives and relation types.
    #[serde(default)]
    pub domain: Option<String>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            graph_source: PathBuf::from("./graph_snapshot.json"),
            plasticity_state: PathBuf::from("./plasticity_state.json"),
            auto_persist_interval: 50,
            learning_rate: 0.08,
            decay_rate: 0.005,
            xlr_enabled: true,
            max_concurrent_reads: 32,
            write_queue_size: 64,
            domain: None,
        }
    }
}

// ---------------------------------------------------------------------------
// McpServer — JSON-RPC stdio server
// Replaces: 03-MCP Section 1.1 deployment model
// ---------------------------------------------------------------------------

/// MCP server over JSON-RPC stdio. Single process, shared PropertyGraph.
/// Replaces: 03-MCP server architecture
pub struct McpServer {
    config: McpConfig,
    state: SessionState,
}

/// List of all registered MCP tool schemas with full inputSchema per MCP spec.
/// Public so the HTTP server can cache and serve it.
pub fn tool_schemas() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            {
                "name": "m1nd_activate",
                "description": "Spreading activation query across the connectome",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query for spreading activation" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "top_k": { "type": "integer", "default": 20, "description": "Number of top results to return" },
                        "dimensions": {
                            "type": "array",
                            "items": { "type": "string", "enum": ["structural", "semantic", "temporal", "causal"] },
                            "default": ["structural", "semantic", "temporal", "causal"],
                            "description": "Activation dimensions to include"
                        },
                        "xlr": { "type": "boolean", "default": true, "description": "Enable XLR noise cancellation" },
                        "include_ghost_edges": { "type": "boolean", "default": true, "description": "Include ghost edge detection" },
                        "include_structural_holes": { "type": "boolean", "default": false, "description": "Include structural hole detection" }
                    },
                    "required": ["query", "agent_id"]
                }
            },
            {
                "name": "m1nd_impact",
                "description": "Impact radius / blast analysis for a node",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "string", "description": "Target node identifier" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "direction": {
                            "type": "string",
                            "enum": ["forward", "reverse", "both"],
                            "default": "forward",
                            "description": "Propagation direction for impact analysis"
                        },
                        "include_causal_chains": { "type": "boolean", "default": true, "description": "Include causal chain detection" }
                    },
                    "required": ["node_id", "agent_id"]
                }
            },
            {
                "name": "m1nd_missing",
                "description": "Detect structural holes and missing connections",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query to find structural holes around" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "min_sibling_activation": { "type": "number", "default": 0.3, "description": "Minimum sibling activation threshold" }
                    },
                    "required": ["query", "agent_id"]
                }
            },
            {
                "name": "m1nd_why",
                "description": "Path explanation between two nodes",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Source node identifier" },
                        "target": { "type": "string", "description": "Target node identifier" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "max_hops": { "type": "integer", "default": 6, "description": "Maximum hops in path search" }
                    },
                    "required": ["source", "target", "agent_id"]
                }
            },
            {
                "name": "m1nd_warmup",
                "description": "Task-based warmup and priming",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_description": { "type": "string", "description": "Description of the task to warm up for" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "boost_strength": { "type": "number", "default": 0.15, "description": "Priming boost strength" }
                    },
                    "required": ["task_description", "agent_id"]
                }
            },
            {
                "name": "m1nd_counterfactual",
                "description": "What-if node removal simulation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_ids": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Node identifiers to simulate removal of"
                        },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "include_cascade": { "type": "boolean", "default": true, "description": "Include cascade analysis" }
                    },
                    "required": ["node_ids", "agent_id"]
                }
            },
            {
                "name": "m1nd_predict",
                "description": "Co-change prediction for a modified node",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "changed_node": { "type": "string", "description": "Node identifier that was changed" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "top_k": { "type": "integer", "default": 10, "description": "Number of top predictions to return" },
                        "include_velocity": { "type": "boolean", "default": true, "description": "Include velocity scoring" }
                    },
                    "required": ["changed_node", "agent_id"]
                }
            },
            {
                "name": "m1nd_fingerprint",
                "description": "Activation fingerprint and equivalence detection",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target_node": { "type": "string", "description": "Optional target node to find equivalents for" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "similarity_threshold": { "type": "number", "default": 0.85, "description": "Cosine similarity threshold for equivalence" },
                        "probe_queries": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional probe queries for fingerprinting"
                        }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_drift",
                "description": "Weight and structural drift analysis",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "since": { "type": "string", "default": "last_session", "description": "Baseline reference point for drift comparison" },
                        "include_weight_drift": { "type": "boolean", "default": true, "description": "Include edge weight drift analysis" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_learn",
                "description": "Explicit feedback-based edge adjustment",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Original query this feedback relates to" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "feedback": {
                            "type": "string",
                            "enum": ["correct", "wrong", "partial"],
                            "description": "Feedback type: correct, wrong, or partial"
                        },
                        "node_ids": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Node identifiers to apply feedback to"
                        },
                        "strength": { "type": "number", "default": 0.2, "description": "Feedback strength for edge adjustment" }
                    },
                    "required": ["query", "agent_id", "feedback", "node_ids"]
                }
            },
            {
                "name": "m1nd_ingest",
                "description": "Ingest or re-ingest a codebase, descriptor, or memory corpus",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Filesystem path to the source root or memory corpus" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "incremental": { "type": "boolean", "default": false, "description": "Incremental ingest (code adapter only)" },
                        "adapter": {
                            "type": "string",
                            "default": "code",
                            "enum": ["code", "json", "memory"],
                            "description": "Adapter to use for parsing the input corpus"
                        },
                        "mode": {
                            "type": "string",
                            "default": "replace",
                            "enum": ["replace", "merge"],
                            "description": "Replace the active graph or merge the ingest into it"
                        },
                        "namespace": {
                            "type": "string",
                            "description": "Optional namespace tag for memory/non-code nodes"
                        }
                    },
                    "required": ["path", "agent_id"]
                }
            },
            {
                "name": "m1nd_resonate",
                "description": "Resonance analysis: harmonics, sympathetic pairs, and resonant frequencies",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search query to find seed nodes for resonance analysis" },
                        "node_id": { "type": "string", "description": "Specific node identifier to use as seed (alternative to query)" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "top_k": { "type": "integer", "default": 20, "description": "Number of top resonance results to return" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_health",
                "description": "Server health and statistics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" }
                    },
                    "required": ["agent_id"]
                }
            },
            // --- Perspective MCP tools (12-PERSPECTIVE-SYNTHESIS) ---
            {
                "name": "m1nd_perspective_start",
                "description": "Enter a perspective: creates a navigable route surface from a query",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "query": { "type": "string", "description": "Seed query for route synthesis" },
                        "anchor_node": { "type": "string", "description": "Optional: anchor to a specific node (activates anchored mode)" },
                        "lens": { "type": "object", "description": "Optional: starting lens configuration" }
                    },
                    "required": ["agent_id", "query"]
                }
            },
            {
                "name": "m1nd_perspective_routes",
                "description": "Browse the current route set with pagination",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "page": { "type": "integer", "default": 1, "description": "Page number (1-based)" },
                        "page_size": { "type": "integer", "default": 6, "description": "Routes per page (clamped to 1-10)" },
                        "route_set_version": { "type": "integer", "description": "Version from previous response for staleness check" }
                    },
                    "required": ["agent_id", "perspective_id"]
                }
            },
            {
                "name": "m1nd_perspective_inspect",
                "description": "Expand a route with fuller path, metrics, provenance, and affinity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "route_id": { "type": "string", "description": "Stable content-addressed route ID" },
                        "route_index": { "type": "integer", "description": "1-based page-local position" },
                        "route_set_version": { "type": "integer" }
                    },
                    "required": ["agent_id", "perspective_id", "route_set_version"]
                }
            },
            {
                "name": "m1nd_perspective_peek",
                "description": "Extract a small relevant code/doc slice from a route target",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "route_id": { "type": "string" },
                        "route_index": { "type": "integer" },
                        "route_set_version": { "type": "integer" }
                    },
                    "required": ["agent_id", "perspective_id", "route_set_version"]
                }
            },
            {
                "name": "m1nd_perspective_follow",
                "description": "Follow a route: move focus to target, synthesize new routes",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "route_id": { "type": "string" },
                        "route_index": { "type": "integer" },
                        "route_set_version": { "type": "integer" }
                    },
                    "required": ["agent_id", "perspective_id", "route_set_version"]
                }
            },
            {
                "name": "m1nd_perspective_suggest",
                "description": "Get the next best move suggestion based on navigation history",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "route_set_version": { "type": "integer" }
                    },
                    "required": ["agent_id", "perspective_id", "route_set_version"]
                }
            },
            {
                "name": "m1nd_perspective_affinity",
                "description": "Discover probable connections a route target might have",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "route_id": { "type": "string" },
                        "route_index": { "type": "integer" },
                        "route_set_version": { "type": "integer" }
                    },
                    "required": ["agent_id", "perspective_id", "route_set_version"]
                }
            },
            {
                "name": "m1nd_perspective_branch",
                "description": "Fork the current navigation state into a new branch",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" },
                        "branch_name": { "type": "string", "description": "Optional branch name" }
                    },
                    "required": ["agent_id", "perspective_id"]
                }
            },
            {
                "name": "m1nd_perspective_back",
                "description": "Navigate back to previous focus, restoring checkpoint state",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" }
                    },
                    "required": ["agent_id", "perspective_id"]
                }
            },
            {
                "name": "m1nd_perspective_compare",
                "description": "Compare two perspectives on shared/unique nodes and dimension deltas",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id_a": { "type": "string" },
                        "perspective_id_b": { "type": "string" },
                        "dimensions": { "type": "array", "items": { "type": "string" }, "description": "Dimensions to compare (empty = all)" }
                    },
                    "required": ["agent_id", "perspective_id_a", "perspective_id_b"]
                }
            },
            {
                "name": "m1nd_perspective_list",
                "description": "List all perspectives for an agent",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_perspective_close",
                "description": "Close a perspective and release associated locks",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "perspective_id": { "type": "string" }
                    },
                    "required": ["agent_id", "perspective_id"]
                }
            },
            // --- Lock tools ---
            {
                "name": "m1nd_lock_create",
                "description": "Pin a subgraph region and capture a baseline for change monitoring",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "scope": { "type": "string", "enum": ["node", "subgraph", "query_neighborhood", "path"] },
                        "root_nodes": { "type": "array", "items": { "type": "string" } },
                        "radius": { "type": "integer", "description": "BFS radius for subgraph scope (1-4)" },
                        "query": { "type": "string", "description": "Query for query_neighborhood scope" },
                        "path_nodes": { "type": "array", "items": { "type": "string" }, "description": "Ordered nodes for path scope" }
                    },
                    "required": ["agent_id", "scope", "root_nodes"]
                }
            },
            {
                "name": "m1nd_lock_watch",
                "description": "Set a watcher strategy on a lock (manual, on_ingest, on_learn)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "lock_id": { "type": "string" },
                        "strategy": { "type": "string", "enum": ["manual", "on_ingest", "on_learn"] }
                    },
                    "required": ["agent_id", "lock_id", "strategy"]
                }
            },
            {
                "name": "m1nd_lock_diff",
                "description": "Compute what changed in a locked region since baseline",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "lock_id": { "type": "string" }
                    },
                    "required": ["agent_id", "lock_id"]
                }
            },
            {
                "name": "m1nd_lock_rebase",
                "description": "Re-capture lock baseline from current graph without releasing",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "lock_id": { "type": "string" }
                    },
                    "required": ["agent_id", "lock_id"]
                }
            },
            {
                "name": "m1nd_lock_release",
                "description": "Release a lock and free its resources",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string" },
                        "lock_id": { "type": "string" }
                    },
                    "required": ["agent_id", "lock_id"]
                }
            },
            // =================================================================
            // L2: Semantic Search
            // =================================================================
            {
                "name": "m1nd_seek",
                "description": "Intent-aware semantic code search — find code by PURPOSE, not text pattern",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Natural language description of what the agent is looking for" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "top_k": { "type": "integer", "default": 20, "description": "Maximum results to return" },
                        "scope": { "type": "string", "description": "File path prefix to limit search scope" },
                        "node_types": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Filter by node type: function, class, struct, module, file" },
                        "min_score": { "type": "number", "default": 0.1, "description": "Minimum combined score threshold" },
                        "graph_rerank": { "type": "boolean", "default": true, "description": "Whether to run graph re-ranking on embedding candidates" }
                    },
                    "required": ["query", "agent_id"]
                }
            },
            {
                "name": "m1nd_scan",
                "description": "Pattern-aware structural code analysis with graph-validated findings",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Pattern ID (error_handling, resource_cleanup, api_surface, state_mutation, concurrency, auth_boundary, test_coverage, dependency_injection) or custom pattern" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "scope": { "type": "string", "description": "File path prefix to limit scan scope" },
                        "severity_min": { "type": "number", "default": 0.3, "description": "Minimum severity threshold [0.0, 1.0]" },
                        "graph_validate": { "type": "boolean", "default": true, "description": "Whether to validate findings against graph edges" },
                        "limit": { "type": "integer", "default": 50, "description": "Maximum findings to return" }
                    },
                    "required": ["pattern", "agent_id"]
                }
            },
            // =================================================================
            // L3: Temporal Intelligence
            // =================================================================
            {
                "name": "m1nd_timeline",
                "description": "Git-based temporal history for a node — changes, co-changes, velocity, stability",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node": { "type": "string", "description": "Node external_id (e.g. file::backend/chat_handler.py)" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "depth": { "type": "string", "default": "30d", "description": "Time depth: 7d, 30d, 90d, all" },
                        "include_co_changes": { "type": "boolean", "default": true, "description": "Include co-changed files with coupling scores" },
                        "include_churn": { "type": "boolean", "default": true, "description": "Include lines added/deleted churn data" },
                        "top_k": { "type": "integer", "default": 10, "description": "Max co-change partners to return" }
                    },
                    "required": ["node", "agent_id"]
                }
            },
            {
                "name": "m1nd_diverge",
                "description": "Detect structural drift between a baseline and current graph state",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "baseline": { "type": "string", "description": "Baseline reference: ISO date, git ref, or last_session" },
                        "scope": { "type": "string", "description": "File path glob to limit scope" },
                        "include_coupling_changes": { "type": "boolean", "default": true, "description": "Include coupling matrix delta" },
                        "include_anomalies": { "type": "boolean", "default": true, "description": "Detect anomalies (test deficits, velocity spikes)" }
                    },
                    "required": ["agent_id", "baseline"]
                }
            },
            // =================================================================
            // L4: Investigation Memory
            // =================================================================
            {
                "name": "m1nd_trail_save",
                "description": "Persist current investigation state — nodes visited, hypotheses, conclusions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "label": { "type": "string", "description": "Human-readable label for this investigation" },
                        "hypotheses": { "type": "array", "items": { "type": "object" }, "default": [], "description": "Hypotheses formed during investigation" },
                        "conclusions": { "type": "array", "items": { "type": "object" }, "default": [], "description": "Conclusions reached" },
                        "open_questions": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Open questions remaining" },
                        "tags": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Tags for organization and search" },
                        "summary": { "type": "string", "description": "Optional summary (auto-generated if omitted)" },
                        "visited_nodes": { "type": "array", "items": { "type": "object" }, "default": [], "description": "Explicitly list visited nodes with annotations" },
                        "activation_boosts": { "type": "object", "default": {}, "description": "Map of node_external_id -> boost weight [0.0, 1.0]" }
                    },
                    "required": ["agent_id", "label"]
                }
            },
            {
                "name": "m1nd_trail_resume",
                "description": "Restore a saved investigation — re-inject activation boosts, detect staleness",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "trail_id": { "type": "string", "description": "Trail ID to resume" },
                        "force": { "type": "boolean", "default": false, "description": "Resume even if trail is stale (>50% missing nodes)" }
                    },
                    "required": ["agent_id", "trail_id"]
                }
            },
            {
                "name": "m1nd_trail_merge",
                "description": "Combine two or more investigation trails — discover cross-connections",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "trail_ids": { "type": "array", "items": { "type": "string" }, "description": "Two or more trail IDs to merge" },
                        "label": { "type": "string", "description": "Label for the merged trail (auto-generated if omitted)" }
                    },
                    "required": ["agent_id", "trail_ids"]
                }
            },
            {
                "name": "m1nd_trail_list",
                "description": "List saved investigation trails with optional filters",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "filter_agent_id": { "type": "string", "description": "Filter to a specific agent's trails" },
                        "filter_status": { "type": "string", "description": "Filter by status: active, saved, archived, stale, merged" },
                        "filter_tags": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Filter by tags (any match)" }
                    },
                    "required": ["agent_id"]
                }
            },
            // =================================================================
            // L5: Hypothesis Engine
            // =================================================================
            {
                "name": "m1nd_hypothesize",
                "description": "Test a structural claim about the codebase — graph-based hypothesis testing",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "claim": { "type": "string", "description": "Natural language claim (e.g. 'chat_handler never validates session tokens')" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "max_hops": { "type": "integer", "default": 5, "description": "Max BFS hops for evidence search" },
                        "include_ghost_edges": { "type": "boolean", "default": true, "description": "Include ghost edges as weak evidence" },
                        "include_partial_flow": { "type": "boolean", "default": true, "description": "Include partial flow when full path not found" },
                        "path_budget": { "type": "integer", "default": 1000, "description": "Budget cap for all-paths enumeration" }
                    },
                    "required": ["claim", "agent_id"]
                }
            },
            {
                "name": "m1nd_differential",
                "description": "Focused structural diff between two graph snapshots",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "snapshot_a": { "type": "string", "description": "Path to snapshot A, or 'current'" },
                        "snapshot_b": { "type": "string", "description": "Path to snapshot B, or 'current'" },
                        "question": { "type": "string", "description": "Focus question to narrow the diff output" },
                        "focus_nodes": { "type": "array", "items": { "type": "string" }, "default": [], "description": "Limit diff to neighborhood of specific nodes" }
                    },
                    "required": ["agent_id", "snapshot_a", "snapshot_b"]
                }
            },
            // =================================================================
            // L6: Execution Feedback
            // =================================================================
            {
                "name": "m1nd_trace",
                "description": "Map runtime errors to structural root causes via stacktrace analysis",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "error_text": { "type": "string", "description": "Full error output (stacktrace + error message)" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "language": { "type": "string", "description": "Language hint: python, rust, typescript, javascript, go (auto-detected if omitted)" },
                        "window_hours": { "type": "number", "default": 24.0, "description": "Temporal window (hours) for co-change suspect scan" },
                        "top_k": { "type": "integer", "default": 10, "description": "Max suspects to return" }
                    },
                    "required": ["error_text", "agent_id"]
                }
            },
            {
                "name": "m1nd_validate_plan",
                "description": "Validate a modification plan against the code graph — detect gaps and risk",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "actions": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "action_type": { "type": "string", "description": "modify, create, delete, rename, or test" },
                                    "file_path": { "type": "string", "description": "Relative file path" },
                                    "description": { "type": "string" },
                                    "depends_on": { "type": "array", "items": { "type": "string" }, "default": [] }
                                },
                                "required": ["action_type", "file_path"]
                            },
                            "description": "Ordered list of planned actions"
                        },
                        "include_test_impact": { "type": "boolean", "default": true, "description": "Analyze test coverage for modified files" },
                        "include_risk_score": { "type": "boolean", "default": true, "description": "Compute composite risk score" }
                    },
                    "required": ["agent_id", "actions"]
                }
            },
            // =================================================================
            // L7: Multi-Repository Federation
            // =================================================================
            {
                "name": "m1nd_federate",
                "description": "Ingest multiple repos into a unified federated graph with cross-repo edge detection",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "repos": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Repository name (namespace prefix)" },
                                    "path": { "type": "string", "description": "Absolute path to repository root" },
                                    "adapter": { "type": "string", "default": "code", "description": "Ingest adapter override" }
                                },
                                "required": ["name", "path"]
                            },
                            "description": "List of repositories to federate"
                        },
                        "detect_cross_repo_edges": { "type": "boolean", "default": true, "description": "Auto-detect cross-repo edges" },
                        "incremental": { "type": "boolean", "default": false, "description": "Only re-ingest repos that changed" }
                    },
                    "required": ["agent_id", "repos"]
                }
            },
            // =================================================================
            // Superpowers: Antibody / Flow / Epidemic / Tremor / Trust / Layers
            // =================================================================
            {
                "name": "m1nd_antibody_scan",
                "description": "Scan code graph against stored bug antibodies (immune memory patterns). Returns matches where known bug patterns recur.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "scope": { "type": "string", "default": "all", "description": "\"all\" = entire graph, \"changed\" = nodes since last scan" },
                        "antibody_ids": { "type": "array", "items": { "type": "string" }, "description": "Optional: only scan specific antibodies" },
                        "max_matches": { "type": "integer", "default": 50, "description": "Maximum matches to return" },
                        "min_severity": { "type": "string", "default": "info", "description": "Minimum severity: info, warning, critical" },
                        "similarity_threshold": { "type": "number", "default": 0.7, "description": "Fuzzy match threshold for label matching (0.0-1.0)" },
                        "match_mode": { "type": "string", "default": "substring", "description": "Label match mode: exact, substring, regex" },
                        "max_matches_per_antibody": { "type": "integer", "default": 50, "description": "Maximum matches per individual antibody" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_antibody_list",
                "description": "List all stored bug antibodies with metadata, match history, and specificity scores.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "include_disabled": { "type": "boolean", "default": false, "description": "Include disabled antibodies" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_antibody_create",
                "description": "Create, disable, enable, or delete a bug antibody pattern.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "action": { "type": "string", "default": "create", "description": "Action: create, disable, enable, delete" },
                        "antibody_id": { "type": "string", "description": "Required for disable/enable/delete" },
                        "name": { "type": "string", "description": "Antibody name (for create)" },
                        "description": { "type": "string", "description": "What this pattern detects" },
                        "severity": { "type": "string", "default": "warning", "description": "info, warning, critical" },
                        "pattern": { "type": "object", "description": "Pattern definition with nodes/edges/negative_edges" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_flow_simulate",
                "description": "Simulate concurrent execution flow. Detects race conditions via particle collision on shared mutable state without synchronization.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "entry_nodes": { "type": "array", "items": { "type": "string" }, "description": "Starting nodes. Auto-discovered if empty." },
                        "num_particles": { "type": "integer", "default": 2, "description": "Particles per entry point" },
                        "lock_patterns": { "type": "array", "items": { "type": "string" }, "description": "Regex patterns for lock/mutex detection" },
                        "read_only_patterns": { "type": "array", "items": { "type": "string" }, "description": "Regex patterns for read-only operations" },
                        "max_depth": { "type": "integer", "default": 15, "description": "Maximum BFS depth" },
                        "turbulence_threshold": { "type": "number", "default": 0.5, "description": "Minimum score to report" },
                        "include_paths": { "type": "boolean", "default": true, "description": "Include particle paths in output" },
                        "max_total_steps": { "type": "integer", "default": 50000, "description": "Global step budget across all particles" },
                        "scope_filter": { "type": "string", "description": "Substring filter to limit which nodes particles can enter" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_epidemic",
                "description": "Predict bug propagation via SIR epidemiological model. Given known buggy modules, predicts which neighbors are most likely to harbor undiscovered bugs.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "infected_nodes": { "type": "array", "items": { "type": "string" }, "description": "Known buggy node IDs" },
                        "recovered_nodes": { "type": "array", "items": { "type": "string" }, "description": "Already-fixed node IDs" },
                        "infection_rate": { "type": "number", "description": "Uniform infection rate. If omitted, derived from edge weights." },
                        "recovery_rate": { "type": "number", "default": 0, "description": "SIR recovery rate" },
                        "iterations": { "type": "integer", "default": 50, "description": "Simulation iterations" },
                        "direction": { "type": "string", "default": "both", "description": "Propagation direction: forward, backward, both" },
                        "top_k": { "type": "integer", "default": 20, "description": "Max predictions to return" },
                        "auto_calibrate": { "type": "boolean", "default": true, "description": "Auto-adjust infection_rate based on graph density" },
                        "scope": { "type": "string", "default": "all", "description": "Filter predictions: files, functions, all" },
                        "min_probability": { "type": "number", "default": 0.001, "description": "Filter out predictions below this probability" }
                    },
                    "required": ["agent_id", "infected_nodes"]
                }
            },
            {
                "name": "m1nd_tremor",
                "description": "Detect code tremors: modules with accelerating change frequency (second derivative). Earthquake precursor analogy for imminent bugs.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "window": { "type": "string", "default": "30d", "description": "Time window: 7d, 30d, 90d, all" },
                        "threshold": { "type": "number", "default": 0.1, "description": "Minimum magnitude to report" },
                        "top_k": { "type": "integer", "default": 20, "description": "Max results" },
                        "node_filter": { "type": "string", "description": "Filter to nodes matching this prefix" },
                        "include_history": { "type": "boolean", "default": false, "description": "Include observation history" },
                        "min_observations": { "type": "integer", "default": 3, "description": "Minimum data points to compute tremor" },
                        "sensitivity": { "type": "number", "default": 1.0, "description": "Multiplier on acceleration threshold (higher = more sensitive)" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_trust",
                "description": "Per-module trust scores from defect history. Actuarial risk assessment: more confirmed bugs = lower trust = higher risk.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "scope": { "type": "string", "default": "file", "description": "Node type scope: file, function, class, all" },
                        "min_history": { "type": "integer", "default": 1, "description": "Minimum learn events for inclusion" },
                        "top_k": { "type": "integer", "default": 20, "description": "Max results" },
                        "node_filter": { "type": "string", "description": "Filter to nodes matching this prefix" },
                        "sort_by": { "type": "string", "default": "trust_asc", "description": "Sort: trust_asc, trust_desc, defects_desc, recency" },
                        "decay_half_life_days": { "type": "number", "default": 30.0, "description": "How fast old defects lose weight (days)" },
                        "risk_cap": { "type": "number", "default": 3.0, "description": "Maximum risk multiplier" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_layers",
                "description": "Auto-detect architectural layers from graph topology. Returns layer assignments plus dependency violations (edges going against expected flow).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "scope": { "type": "string", "description": "File path prefix to limit scope" },
                        "max_layers": { "type": "integer", "default": 8, "description": "Maximum layers to detect" },
                        "include_violations": { "type": "boolean", "default": true, "description": "Include violation analysis" },
                        "min_nodes_per_layer": { "type": "integer", "default": 2, "description": "Minimum nodes for a layer to be reported" },
                        "node_types": { "type": "array", "items": { "type": "string" }, "description": "Filter by node types" },
                        "naming_strategy": { "type": "string", "default": "auto", "description": "Layer naming: auto, path_prefix, pagerank" },
                        "exclude_tests": { "type": "boolean", "default": false, "description": "Exclude test files from layer detection" },
                        "violation_limit": { "type": "integer", "default": 100, "description": "Maximum violations to return" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_layer_inspect",
                "description": "Inspect a specific architectural layer: nodes, connections, violations, and health metrics.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "level": { "type": "integer", "description": "Layer level to inspect" },
                        "scope": { "type": "string", "description": "File path prefix to limit scope" },
                        "include_edges": { "type": "boolean", "default": true, "description": "Include inter-layer edges" },
                        "top_k": { "type": "integer", "default": 50, "description": "Max nodes to return per layer" }
                    },
                    "required": ["agent_id", "level"]
                }
            },
            // =================================================================
            // Surgical: context + apply
            // =================================================================
            {
                "name": "m1nd_surgical_context",
                "description": "Return full context for surgical LLM editing: file contents, symbols, and graph neighbourhood (callers, callees, tests). Use before m1nd.apply.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute or workspace-relative path to the file being edited" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "symbol": { "type": "string", "description": "Optional: narrow context to a specific symbol (function/struct/class name)" },
                        "radius": { "type": "integer", "default": 1, "description": "BFS radius for graph neighbourhood (1 or 2)" },
                        "include_tests": { "type": "boolean", "default": true, "description": "Include test files in the neighbourhood" }
                    },
                    "required": ["file_path", "agent_id"]
                }
            },
            {
                "name": "m1nd_apply",
                "description": "Write LLM-edited code back to a file and trigger incremental re-ingest so the graph stays coherent. Always call m1nd.surgical_context first.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": { "type": "string", "description": "Absolute or workspace-relative path of the file to overwrite" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "new_content": { "type": "string", "description": "New file contents (full replacement, UTF-8)" },
                        "description": { "type": "string", "description": "Human-readable description of the edit" },
                        "reingest": { "type": "boolean", "default": true, "description": "Re-ingest the file after writing (recommended)" }
                    },
                    "required": ["file_path", "agent_id", "new_content"]
                }
            },
            // =================================================================
            // Surgical V2: context_v2 + apply_batch
            // =================================================================
            {
                "name": "m1nd_surgical_context_v2",
                "description": "Get full surgical context for a file PLUS source code of connected files (callers, callees, tests). Returns a complete workspace snapshot in one call. Superset of m1nd.surgical_context.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "file_path": { "type": "string", "description": "Absolute or workspace-relative path to the primary file" },
                        "symbol": { "type": "string", "description": "Optional: narrow context to a specific symbol (function/struct/class name)" },
                        "include_tests": { "type": "boolean", "default": true, "description": "Include test files in the neighbourhood" },
                        "radius": { "type": "integer", "default": 1, "description": "BFS radius for graph neighbourhood (1 or 2)" },
                        "max_connected_files": { "type": "integer", "default": 5, "description": "Maximum number of connected files to include source for" },
                        "max_lines_per_file": { "type": "integer", "default": 60, "description": "Maximum lines per connected file (primary file is unbounded)" }
                    },
                    "required": ["agent_id", "file_path"]
                }
            },
            {
                "name": "m1nd_apply_batch",
                "description": "Atomically write multiple files and trigger a single bulk re-ingest. Use after m1nd.surgical_context_v2 when editing a file and its callers/tests together. All-or-nothing by default.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "edits": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "file_path": { "type": "string", "description": "Absolute or workspace-relative path of the file to write" },
                                    "new_content": { "type": "string", "description": "New file contents (full replacement, UTF-8)" },
                                    "description": { "type": "string", "description": "Optional human-readable label for this edit" }
                                },
                                "required": ["file_path", "new_content"]
                            },
                            "description": "List of file edits to apply"
                        },
                        "atomic": { "type": "boolean", "default": true, "description": "All-or-nothing: if any file fails, none are written" },
                        "reingest": { "type": "boolean", "default": true, "description": "Re-ingest all modified files after writing" }
                    },
                    "required": ["agent_id", "edits"]
                }
            },
            // =================================================================
            // v0.4.0: search, help, report, panoramic, savings
            // =================================================================
            {
                "name": "m1nd_search",
                "description": "Unified code search: literal, regex (with multiline), or semantic. Searches graph node labels AND file contents on disk. Supports invert (grep -v), count-only (grep -c), multiline regex (rg -U), and filename pattern filtering (grep --include). v0.5.0: regex mode now searches file contents (not just node IDs).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "query": { "type": "string", "description": "Search query string" },
                        "mode": {
                            "type": "string",
                            "enum": ["literal", "regex", "semantic"],
                            "default": "literal",
                            "description": "Search mode: literal (substring), regex (pattern), semantic (graph-aware)"
                        },
                        "scope": { "type": "string", "description": "File path prefix filter" },
                        "top_k": { "type": "integer", "default": 50, "description": "Max results (1-500)" },
                        "context_lines": { "type": "integer", "default": 2, "description": "Lines of context before/after match (0-10)" },
                        "case_sensitive": { "type": "boolean", "default": false, "description": "Case-sensitive matching" },
                        "invert": { "type": "boolean", "default": false, "description": "Return lines that DON'T match (grep -v)" },
                        "count_only": { "type": "boolean", "default": false, "description": "Return just the count, no results (grep -c)" },
                        "multiline": { "type": "boolean", "default": false, "description": "Enable multiline regex: dot matches newline (rg -U). Only for regex mode." },
                        "auto_ingest": { "type": "boolean", "default": false, "description": "Auto-ingest scope directory if not in graph (reserved)" },
                        "filename_pattern": { "type": "string", "description": "Glob pattern to filter filenames (e.g. '*.rs', 'test_*.py')" }
                    },
                    "required": ["agent_id", "query"]
                }
            },
            {
                "name": "m1nd_glob",
                "description": "Graph-aware file glob: find files in the ingested graph by glob pattern. Zero I/O — pure graph query. Replaces find/glob for indexed codebases.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "pattern": { "type": "string", "description": "Glob pattern (e.g. '**/*.rs', 'src/**/mod.rs', '*.toml')" },
                        "scope": { "type": "string", "description": "Root directory prefix to narrow scope" },
                        "top_k": { "type": "integer", "default": 200, "description": "Max results (1-10000)" },
                        "sort": {
                            "type": "string",
                            "enum": ["path", "activation"],
                            "default": "path",
                            "description": "Sort order: path (alphabetical) or activation (most connected first)"
                        }
                    },
                    "required": ["agent_id", "pattern"]
                }
            },
            {
                "name": "m1nd_help",
                "description": "Get help text for m1nd tools. Returns overview or detailed help for a specific tool with visual identity.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "tool_name": { "type": "string", "description": "Specific tool name for detailed help (omit for overview)" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_report",
                "description": "Session intelligence report: queries, bugs, graph evolution, and estimated savings.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_panoramic",
                "description": "Panoramic graph health overview: per-module risk scores combining blast radius, centrality, and churn signals.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "scope": { "type": "string", "description": "File path prefix filter" },
                        "top_n": { "type": "integer", "default": 50, "description": "Max modules to return (1-1000)" }
                    },
                    "required": ["agent_id"]
                }
            },
            {
                "name": "m1nd_savings",
                "description": "Estimated token and cost savings from using m1nd. Shows current session and global totals with Gaia counter.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent_id": { "type": "string", "description": "Calling agent identifier" }
                    },
                    "required": ["agent_id"]
                }
            }
        ]
    })
}

// ---------------------------------------------------------------------------
// Free dispatch functions — used by both JSON-RPC stdio and HTTP API.
// Zero duplication: McpServer::dispatch_tool() delegates to these.
// ---------------------------------------------------------------------------

/// Dispatch a tool call by name. Normalizes underscores to dots.
/// Used by both JSON-RPC stdio and HTTP API -- zero duplication.
///
/// v0.4.0: wraps all responses with _m1nd metadata, tracks savings.
pub fn dispatch_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    let normalized = tool_name.to_string();
    let start = std::time::Instant::now();

    // Extract agent_id for tracking
    let agent_id = params
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let query_preview = params
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            params
                .get("claim")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| params.get("node_id").and_then(|v| v.as_str()).unwrap_or(""))
        })
        .to_string();

    let result = match normalized.as_str() {
        name if name.starts_with("m1nd_perspective_") => {
            dispatch_perspective_tool(state, name, params)
        }
        name if name.starts_with("m1nd_lock_") => dispatch_lock_tool(state, name, params),
        _ => dispatch_core_tool(state, &normalized, params),
    };

    // Post-dispatch: track savings + log query + add _m1nd metadata
    if let Ok(ref value) = result {
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        let result_count = value
            .get("results")
            .and_then(|v| v.as_array())
            .map_or(0, |a| a.len());

        // Track savings (skip meta tools)
        if !matches!(
            normalized.as_str(),
            "m1nd_health" | "m1nd_help" | "m1nd_savings" | "m1nd_report"
        ) {
            state.savings_tracker.record(&normalized, result_count);
            state.global_savings.total_queries += 1;
        }

        // Log query
        state.log_query(
            &normalized,
            &agent_id,
            elapsed_ms,
            result_count,
            &query_preview,
        );
    }

    result
}

/// Dispatch core + superpowers tools (35 tools).
fn dispatch_core_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    match tool_name {
        "m1nd_activate" => {
            let input: ActivateInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = tools::handle_activate(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_impact" => {
            let input: ImpactInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = tools::handle_impact(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_missing" => {
            let input: MissingInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_missing(state, input)
        }
        "m1nd_why" => {
            let input: WhyInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_why(state, input)
        }
        "m1nd_warmup" => {
            let input: WarmupInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_warmup(state, input)
        }
        "m1nd_counterfactual" => {
            let input: CounterfactualInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_counterfactual(state, input)
        }
        "m1nd_predict" => {
            let input: PredictInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_predict(state, input)
        }
        "m1nd_fingerprint" => {
            let input: FingerprintInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_fingerprint(state, input)
        }
        "m1nd_drift" => {
            let input: DriftInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_drift(state, input)
        }
        "m1nd_learn" => {
            let input: LearnInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_learn(state, input)
        }
        "m1nd_ingest" => {
            let input: IngestInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_ingest(state, input)
        }
        "m1nd_resonate" => {
            let input: ResonateInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            tools::handle_resonate(state, input)
        }
        "m1nd_health" => {
            let input: HealthInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = tools::handle_health(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        // L2-L7: Superpowers layer tools
        "m1nd_seek" => {
            let input: layers::SeekInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_seek(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_scan" => {
            let input: layers::ScanInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_scan(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_timeline" => {
            let input: layers::TimelineInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_timeline(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_diverge" => {
            let input: layers::DivergeInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_diverge(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_trail_save" => {
            let input: layers::TrailSaveInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_trail_save(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_trail_resume" => {
            let input: layers::TrailResumeInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_trail_resume(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_trail_merge" => {
            let input: layers::TrailMergeInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_trail_merge(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_trail_list" => {
            let input: layers::TrailListInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_trail_list(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_hypothesize" => {
            let input: layers::HypothesizeInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_hypothesize(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_differential" => {
            let input: layers::DifferentialInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_differential(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_trace" => {
            let input: layers::TraceInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_trace(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_validate_plan" => {
            let input: layers::ValidatePlanInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_validate_plan(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_federate" => {
            let input: layers::FederateInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = layer_handlers::handle_federate(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_antibody_scan" => {
            let input: layers::AntibodyScanInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_antibody_scan(state, input)
        }
        "m1nd_antibody_list" => {
            let input: layers::AntibodyListInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_antibody_list(state, input)
        }
        "m1nd_antibody_create" => {
            let input: layers::AntibodyCreateInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_antibody_create(state, input)
        }
        "m1nd_flow_simulate" => {
            let input: layers::FlowSimulateInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_flow_simulate(state, input)
        }
        "m1nd_epidemic" => {
            let input: layers::EpidemicInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_epidemic(state, input)
        }
        "m1nd_tremor" => {
            let input: layers::TremorInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_tremor(state, input)
        }
        "m1nd_trust" => {
            let input: layers::TrustInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_trust(state, input)
        }
        "m1nd_layers" => {
            let input: layers::LayersInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_layers(state, input)
        }
        "m1nd_layer_inspect" => {
            let input: layers::LayerInspectInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            layer_handlers::handle_layer_inspect(state, input)
        }
        // -----------------------------------------------------------------
        // v0.4.0: search, help, panoramic, savings, report
        // -----------------------------------------------------------------
        "m1nd_search" => {
            let input: layers::SearchInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = search_handlers::handle_search(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_glob" => {
            let input: layers::GlobInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = search_handlers::handle_glob(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_help" => {
            let input: layers::HelpInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = search_handlers::handle_help(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_report" => {
            let input: layers::ReportInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = report_handlers::handle_report(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_panoramic" => {
            let input: layers::PanoramicInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = report_handlers::handle_panoramic(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_savings" => {
            let input: layers::SavingsInput =
                serde_json::from_value(params.clone()).map_err(M1ndError::Serde)?;
            let output = report_handlers::handle_savings(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        // -----------------------------------------------------------------
        // Surgical: context + apply
        // -----------------------------------------------------------------
        "m1nd_surgical_context" => {
            let input: crate::protocol::surgical::SurgicalContextInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_surgical_context".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_surgical_context(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_apply" => {
            let input: crate::protocol::surgical::ApplyInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_apply".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_apply(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        // -----------------------------------------------------------------
        // Surgical V2: context_v2 + apply_batch
        // -----------------------------------------------------------------
        "m1nd_surgical_context_v2" => {
            let input: crate::protocol::surgical::SurgicalContextV2Input =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_surgical_context_v2".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_surgical_context_v2(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        "m1nd_apply_batch" => {
            let input: crate::protocol::surgical::ApplyBatchInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_apply_batch".into(),
                    detail: e.to_string(),
                })?;
            let output = surgical_handlers::handle_apply_batch(state, input)?;
            serde_json::to_value(output).map_err(M1ndError::Serde)
        }
        _ => Err(M1ndError::UnknownTool {
            name: tool_name.to_string(),
        }),
    }
}

/// Dispatch perspective tools (12 tools).
fn dispatch_perspective_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    use crate::perspective_handlers;
    use crate::protocol::perspective::*;

    match tool_name {
        "m1nd_perspective_start" => {
            let input: PerspectiveStartInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_start".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_start(state, input)
        }
        "m1nd_perspective_routes" => {
            let input: PerspectiveRoutesInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_routes".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_routes(state, input)
        }
        "m1nd_perspective_inspect" => {
            let input: PerspectiveInspectInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_inspect".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_inspect(state, input)
        }
        "m1nd_perspective_peek" => {
            let input: PerspectivePeekInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_peek".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_peek(state, input)
        }
        "m1nd_perspective_follow" => {
            let input: PerspectiveFollowInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_follow".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_follow(state, input)
        }
        "m1nd_perspective_suggest" => {
            let input: PerspectiveSuggestInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_suggest".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_suggest(state, input)
        }
        "m1nd_perspective_affinity" => {
            let input: PerspectiveAffinityInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_affinity".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_affinity(state, input)
        }
        "m1nd_perspective_branch" => {
            let input: PerspectiveBranchInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_branch".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_branch(state, input)
        }
        "m1nd_perspective_back" => {
            let input: PerspectiveBackInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_back".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_back(state, input)
        }
        "m1nd_perspective_compare" => {
            let input: PerspectiveCompareInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_compare".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_compare(state, input)
        }
        "m1nd_perspective_list" => {
            let input: PerspectiveListInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_list".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_list(state, input)
        }
        "m1nd_perspective_close" => {
            let input: PerspectiveCloseInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_perspective_close".into(),
                    detail: e.to_string(),
                })?;
            perspective_handlers::handle_perspective_close(state, input)
        }
        _ => Err(M1ndError::UnknownTool {
            name: tool_name.to_string(),
        }),
    }
}

/// Dispatch lock tools (5 tools).
fn dispatch_lock_tool(
    state: &mut SessionState,
    tool_name: &str,
    params: &serde_json::Value,
) -> M1ndResult<serde_json::Value> {
    use crate::lock_handlers;
    use crate::protocol::lock::*;

    match tool_name {
        "m1nd_lock_create" => {
            let input: LockCreateInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_lock_create".into(),
                    detail: e.to_string(),
                })?;
            lock_handlers::handle_lock_create(state, input)
        }
        "m1nd_lock_watch" => {
            let input: LockWatchInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_lock_watch".into(),
                    detail: e.to_string(),
                })?;
            lock_handlers::handle_lock_watch(state, input)
        }
        "m1nd_lock_diff" => {
            let input: LockDiffInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_lock_diff".into(),
                    detail: e.to_string(),
                })?;
            lock_handlers::handle_lock_diff(state, input)
        }
        "m1nd_lock_rebase" => {
            let input: LockRebaseInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_lock_rebase".into(),
                    detail: e.to_string(),
                })?;
            lock_handlers::handle_lock_rebase(state, input)
        }
        "m1nd_lock_release" => {
            let input: LockReleaseInput =
                serde_json::from_value(params.clone()).map_err(|e| M1ndError::InvalidParams {
                    tool: "m1nd_lock_release".into(),
                    detail: e.to_string(),
                })?;
            lock_handlers::handle_lock_release(state, input)
        }
        _ => Err(M1ndError::UnknownTool {
            name: tool_name.to_string(),
        }),
    }
}

impl McpServer {
    /// Create server with config. Does not start serving yet.
    ///
    /// Startup sequence:
    /// 1. Try to load graph snapshot from disk
    /// 2. If loaded, finalize (PageRank + CSR) if needed
    /// 3. Build all engines from graph
    /// 4. Try to load plasticity state and import into graph
    /// 5. Fall back gracefully to empty graph on any failure
    pub fn new(config: McpConfig) -> M1ndResult<Self> {
        // Build domain config from config.domain
        let domain_config = match config.domain.as_deref() {
            Some("music") => DomainConfig::music(),
            Some("memory") => DomainConfig::memory(),
            Some("generic") => DomainConfig::generic(),
            Some("code") | None => DomainConfig::code(),
            Some(other) => {
                eprintln!("[m1nd] Unknown domain '{}', falling back to 'code'", other);
                DomainConfig::code()
            }
        };
        eprintln!("[m1nd] Domain: {}", domain_config.name);

        // Step 1: Try to load graph snapshot
        let (mut graph, graph_loaded) = if config.graph_source.exists() {
            match m1nd_core::snapshot::load_graph(&config.graph_source) {
                Ok(g) => {
                    eprintln!(
                        "[m1nd] Loaded graph snapshot: {} nodes, {} edges",
                        g.num_nodes(),
                        g.num_edges(),
                    );
                    (g, true)
                }
                Err(e) => {
                    eprintln!(
                        "[m1nd] Failed to load graph snapshot ({}), starting fresh",
                        e,
                    );
                    (m1nd_core::graph::Graph::new(), false)
                }
            }
        } else {
            eprintln!("[m1nd] No graph snapshot found, starting fresh");
            (m1nd_core::graph::Graph::new(), false)
        };

        // Step 2: Finalize loaded graph if needed
        if graph_loaded && !graph.finalized && graph.num_nodes() > 0 {
            if let Err(e) = graph.finalize() {
                eprintln!(
                    "[m1nd] Failed to finalize loaded graph ({}), starting fresh",
                    e,
                );
                graph = m1nd_core::graph::Graph::new();
            }
        }

        // Step 3: Build all engines (handled by SessionState::initialize)
        let mut state = SessionState::initialize(graph, &config, domain_config)?;

        // Step 4: Try to load plasticity state
        if graph_loaded && config.plasticity_state.exists() {
            match m1nd_core::snapshot::load_plasticity_state(&config.plasticity_state) {
                Ok(states) => {
                    let mut g = state.graph.write();
                    match state.plasticity.import_state(&mut g, &states) {
                        Ok(_) => {
                            eprintln!(
                                "[m1nd] Loaded plasticity state: {} synaptic records",
                                states.len(),
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[m1nd] Failed to import plasticity state ({}), continuing without it",
                                e,
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[m1nd] Failed to load plasticity state ({}), continuing without it",
                        e,
                    );
                }
            }
        }

        Ok(Self { config, state })
    }

    /// Consume the McpServer and return the SessionState.
    /// Used by the HTTP server to take ownership of the session state
    /// and wrap it in Arc<Mutex<>> for shared concurrent access.
    pub fn into_session_state(self) -> SessionState {
        self.state
    }

    /// Startup sequence (03-MCP Section 1.2):
    /// 1. Load graph snapshot       (done in new())
    /// 2. Load plasticity state     (done in new())
    /// 3. Compute PageRank          (done in new() via finalize)
    /// 4. Build CSR (finalize)      (done in new() via finalize)
    /// 5. Warm up engines           (engines built in new())
    /// 6. Register MCP tools (13 tools)
    /// 7. Ready for connections
    pub fn start(&mut self) -> M1ndResult<()> {
        eprintln!(
            "[m1nd-mcp] Server ready. {} nodes, {} edges",
            self.state.graph.read().num_nodes(),
            self.state.graph.read().num_edges(),
        );

        Ok(())
    }

    /// Main event loop: read JSON-RPC from stdin, dispatch, write response to stdout.
    /// Blocks until EOF or shutdown signal.
    pub fn serve(&mut self) -> M1ndResult<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        loop {
            let (payload, transport_mode) = match read_request_payload(&mut reader) {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(_) => break,
            };
            let trimmed = payload.trim();
            if trimmed.is_empty() {
                continue;
            }

            // MCP notifications (no "id" field) must be silently ignored per spec.
            // Check for notification before attempting full request parse.
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if raw.get("id").is_none() {
                    // This is a notification — no response required.
                    continue;
                }
            }

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: serde_json::Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    let _ = write_response(&mut writer, &err_resp, transport_mode);
                    continue;
                }
            };

            // Dispatch and get response
            let response = self.dispatch(&request);

            let resp = match response {
                Ok(r) => r,
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("{}", e),
                        data: None,
                    }),
                },
            };

            if write_response(&mut writer, &resp, transport_mode).is_err() {
                break; // stdout closed
            }
        }

        Ok(())
    }

    /// Graceful shutdown: persist state, flush writes, close connections.
    pub fn shutdown(&mut self) -> M1ndResult<()> {
        eprintln!("[m1nd-mcp] Shutting down...");
        let _ = self.state.persist();
        eprintln!("[m1nd-mcp] State persisted. Goodbye.");
        Ok(())
    }

    /// Dispatch a single JSON-RPC request to the appropriate tool handler.
    fn dispatch(&mut self, request: &JsonRpcRequest) -> M1ndResult<JsonRpcResponse> {
        let method = request.method.as_str();

        // Handle MCP protocol methods
        match method {
            "initialize" => Ok(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "m1nd-mcp",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                    "capabilities": {
                        "tools": {},
                    },
                    "instructions": M1ND_INSTRUCTIONS,
                })),
                error: None,
            }),
            "notifications/initialized" => {
                // No response needed for notifications, but we return one since caller expects it
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(serde_json::Value::Null),
                    error: None,
                })
            }
            "tools/list" => Ok(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: request.id.clone(),
                result: Some(tool_schemas()),
                error: None,
            }),
            "tools/call" => {
                // Extract tool name and arguments from params
                let tool_name = request
                    .params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                // Track agent session from arguments
                if let Some(agent_id) = arguments.get("agent_id").and_then(|v| v.as_str()) {
                    self.state.track_agent(agent_id);
                }

                // MCP spec: tool execution errors -> isError content, not JSON-RPC errors
                match self.dispatch_tool_call(tool_name, &arguments) {
                    Ok(result) => Ok(JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: request.id.clone(),
                        result: Some(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_default(),
                            }]
                        })),
                        error: None,
                    }),
                    Err(e) => Ok(JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: request.id.clone(),
                        result: Some(serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e),
                            }],
                            "isError": true
                        })),
                        error: None,
                    }),
                }
            }
            _ => {
                // Method not found — JSON-RPC protocol error
                Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", method),
                        data: None,
                    }),
                })
            }
        }
    }

    /// Dispatch a tool call by name. Delegates to the free dispatch_tool() function.
    fn dispatch_tool_call(
        &mut self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> M1ndResult<serde_json::Value> {
        dispatch_tool(&mut self.state, tool_name, params)
    }
}
