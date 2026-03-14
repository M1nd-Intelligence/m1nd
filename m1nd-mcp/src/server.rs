// === crates/m1nd-mcp/src/server.rs ===

use m1nd_core::domain::DomainConfig;
use m1nd_core::error::{M1ndError, M1ndResult};
use crate::brand;
use crate::session::SessionState;
use crate::protocol::*;
use crate::protocol::layers;
use crate::tools;
use crate::layer_handlers;
use std::io::{BufRead, Read, Write};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// MCP protocol instructions — injected into initialize response so agents
// automatically understand how to use m1nd effectively.
// ---------------------------------------------------------------------------
const M1ND_INSTRUCTIONS: &str = "\
\u{234C}\u{2350}\u{2342}\u{1D53B} \u{27C1} \u{2014} neuro-symbolic code graph engine\n\
\n\
Ingests codebases into a weighted graph and provides spreading-activation \
queries, impact analysis, prediction, and stateful perspective navigation. \
All tool calls require an `agent_id` parameter.\n\
\n\
## SYMBOL LEGEND\n\
\u{234C} signal/flow (activate, ingest, learn, warmup) \u{2502} \
\u{2350} path/trace (why, trace, timeline, seek) \u{2502} \
\u{2342} structure (missing, fingerprint, scan, diverge)\n\
\u{1D53B} dimension (impact, predict, counterfactual) \u{2502} \
\u{27C1} connection (perspective, federate, resonate)\n\
\n\
## WORKFLOWS\n\
\n\
**Session Start**: `health` \u{2192} `drift` (recover what changed since last session) \u{2192} \
`ingest` (if graph is empty or stale). This gives you codebase-aware context.\n\
\n\
**Research**: `ingest` \u{2192} `activate(query)` \u{2192} `why(source, target)` \u{2192} `missing(topic)` \u{2192} \
`learn(feedback)`. Use `seek` for keyword search, `scan` for broad discovery, \
`trace` for dependency chains, `timeline` for temporal ordering.\n\
\n\
**Code Change**: `impact(node)` (blast radius) \u{2192} `predict(node)` (co-change likelihood) \u{2192} \
`counterfactual(nodes)` (simulate removal) \u{2192} `warmup(task_description)` (prime graph). \
Use `differential` to compare two subgraphs. Use `hypothesize` to test what-if scenarios.\n\
\n\
**Deep Analysis**: `resonate(query)` for standing-wave harmonic patterns. \
`fingerprint(nodes)` for duplicate/equivalence detection. `diverge(node)` for \
exploring unexpected connections. `federate` to query across graph namespaces.\n\
\n\
## PERSPECTIVE SYSTEM (stateful navigation)\n\
\n\
Perspectives are named, agent-scoped navigation sessions through the graph. \
Flow: `perspective_start(name, seed_nodes)` \u{2192} `perspective_follow(node)` (move focus) \u{2192} \
`perspective_branch(name)` (fork exploration) \u{2192} `perspective_back` (undo last move) \u{2192} \
`perspective_close`. Use `perspective_inspect` to see current state, `perspective_peek` \
to look at a node without moving, `perspective_list` for all open perspectives, \
`perspective_compare` to diff two perspectives, `perspective_suggest` for next-step \
recommendations, `perspective_routes` for paths between nodes, `perspective_affinity` \
for related-node scoring.\n\
\n\
## CONCURRENCY & STATE\n\
\n\
`lock_create` / `lock_release` \u{2014} advisory locks for multi-agent coordination on graph \
regions. `lock_watch` monitors lock state. `lock_diff` shows changes within a lock scope. \
`lock_rebase` replays external changes into a locked region. \
`trail_save` / `trail_list` / `trail_resume` / `trail_merge` \u{2014} persist and restore \
exploration trails across sessions. `validate_plan` checks a proposed multi-step plan \
for structural soundness.\n\
\n\
## CRITICAL PATTERNS\n\
\n\
1. **Always call `learn` after using `activate` results.** Feedback (correct/wrong/partial) \
trains the graph weights via Hebbian learning. Skipping this degrades future queries.\n\
2. **Use `ingest` at session start** if the graph has zero nodes or the codebase changed.\n\
3. **Use `drift` to recover context** between sessions \u{2014} it shows weight changes since \
a baseline timestamp.\n\
4. **`warmup` before focused work** \u{2014} primes activation patterns for a specific task, \
making subsequent queries faster and more relevant.\n\
5. **Never call `activate` without `agent_id`** \u{2014} multi-agent isolation depends on it.\n\
6. **Prefer `impact` over `activate` for code changes** \u{2014} impact gives directional \
blast-radius analysis; activate gives associative exploration.\n\
7. **Graph persists automatically** every 50 queries and on shutdown. Use `trail_save` \
for explicit exploration checkpoints.\n\
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
            let payload = String::from_utf8(body).map_err(|err| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, err)
            })?;
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
            write!(writer, "Content-Length: {}\r\n\r\n{}", json.as_bytes().len(), json)?;
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
fn tool_schemas() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            {
                "name": "m1nd.activate",
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
                "name": "m1nd.impact",
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
                "name": "m1nd.missing",
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
                "name": "m1nd.why",
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
                "name": "m1nd.warmup",
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
                "name": "m1nd.counterfactual",
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
                "name": "m1nd.predict",
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
                "name": "m1nd.fingerprint",
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
                "name": "m1nd.drift",
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
                "name": "m1nd.learn",
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
                "name": "m1nd.ingest",
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
                "name": "m1nd.resonate",
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
                "name": "m1nd.health",
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
                "name": "m1nd.perspective.start",
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
                "name": "m1nd.perspective.routes",
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
                "name": "m1nd.perspective.inspect",
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
                "name": "m1nd.perspective.peek",
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
                "name": "m1nd.perspective.follow",
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
                "name": "m1nd.perspective.suggest",
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
                "name": "m1nd.perspective.affinity",
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
                "name": "m1nd.perspective.branch",
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
                "name": "m1nd.perspective.back",
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
                "name": "m1nd.perspective.compare",
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
                "name": "m1nd.perspective.list",
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
                "name": "m1nd.perspective.close",
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
                "name": "m1nd.lock.create",
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
                "name": "m1nd.lock.watch",
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
                "name": "m1nd.lock.diff",
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
                "name": "m1nd.lock.rebase",
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
                "name": "m1nd.lock.release",
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
                "name": "m1nd.seek",
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
                "name": "m1nd.scan",
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
                "name": "m1nd.timeline",
                "description": "Git-based temporal history for a node — changes, co-changes, velocity, stability",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node": { "type": "string", "description": "Node external_id (e.g. file::backend/handler.py)" },
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
                "name": "m1nd.diverge",
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
                "name": "m1nd.trail.save",
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
                "name": "m1nd.trail.resume",
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
                "name": "m1nd.trail.merge",
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
                "name": "m1nd.trail.list",
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
                "name": "m1nd.hypothesize",
                "description": "Test a structural claim about the codebase — graph-based hypothesis testing",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "claim": { "type": "string", "description": "Natural language claim (e.g. 'handler never validates session tokens')" },
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
                "name": "m1nd.differential",
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
                "name": "m1nd.trace",
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
                "name": "m1nd.validate_plan",
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
                "name": "m1nd.federate",
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
            }
        ]
    })
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
                eprintln!("{}", brand::log_colored(&format!("Unknown domain '{}', falling back to 'code'", other)));
                DomainConfig::code()
            }
        };
        eprintln!("{}", brand::log_colored(&format!("Domain: {}", domain_config.name)));

        // Step 1: Try to load graph snapshot
        let (mut graph, graph_loaded) = if config.graph_source.exists() {
            match m1nd_core::snapshot::load_graph(&config.graph_source) {
                Ok(g) => {
                    eprintln!(
                        "{}",
                        brand::log_colored(&format!("Loaded graph snapshot: {} nodes, {} edges", g.num_nodes(), g.num_edges())),
                    );
                    (g, true)
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        brand::log_colored(&format!("Failed to load graph snapshot ({}), starting fresh", e)),
                    );
                    (m1nd_core::graph::Graph::new(), false)
                }
            }
        } else {
            eprintln!("{}", brand::log_colored("No graph snapshot found, starting fresh"));
            (m1nd_core::graph::Graph::new(), false)
        };

        // Step 2: Finalize loaded graph if needed
        if graph_loaded && !graph.finalized && graph.num_nodes() > 0 {
            if let Err(e) = graph.finalize() {
                eprintln!(
                    "{}",
                    brand::log_colored(&format!("Failed to finalize loaded graph ({}), starting fresh", e)),
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
                                "{}",
                                brand::log_colored(&format!("Loaded plasticity state: {} synaptic records", states.len())),
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                brand::log_colored(&format!("Failed to import plasticity state ({}), continuing without it", e)),
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{}",
                        brand::log_colored(&format!("Failed to load plasticity state ({}), continuing without it", e)),
                    );
                }
            }
        }

        Ok(Self { config, state })
    }

    /// Startup sequence (03-MCP Section 1.2):
    /// 1. Load graph snapshot       (done in new())
    /// 2. Load plasticity state     (done in new())
    /// 3. Compute PageRank          (done in new() via finalize)
    /// 4. Build CSR (finalize)      (done in new() via finalize)
    /// 5. Warm up engines           (engines built in new())
    /// 6. Register MCP tools (43 tools)
    /// 7. Ready for connections
    pub fn start(&mut self) -> M1ndResult<()> {
        eprintln!(
            "{}",
            brand::log_colored(&format!(
                "Server ready. {} nodes, {} edges",
                self.state.graph.read().num_nodes(),
                self.state.graph.read().num_edges(),
            )),
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
        eprintln!("{}", brand::log_colored("Shutting down..."));
        let _ = self.state.persist();
        eprintln!("{}", brand::log_colored("State persisted. Goodbye."));
        Ok(())
    }

    /// Dispatch a single JSON-RPC request to the appropriate tool handler.
    fn dispatch(&mut self, request: &JsonRpcRequest) -> M1ndResult<JsonRpcResponse> {
        let method = request.method.as_str();

        // Handle MCP protocol methods
        match method {
            "initialize" => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(serde_json::json!({
                        "protocolVersion": "2024-11-05",
                        "serverInfo": {
                            "name": format!("{}", brand::SIG),
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "capabilities": {
                            "tools": {},
                        },
                        "instructions": M1ND_INSTRUCTIONS,
                    })),
                    error: None,
                });
            }
            "notifications/initialized" => {
                // No response needed for notifications, but we return one since caller expects it
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(serde_json::Value::Null),
                    error: None,
                });
            }
            "tools/list" => {
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: Some(tool_schemas()),
                    error: None,
                });
            }
            "tools/call" => {
                // Extract tool name and arguments from params
                let tool_name = request.params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = request.params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                // Track agent session from arguments
                if let Some(agent_id) = arguments.get("agent_id").and_then(|v| v.as_str()) {
                    self.state.track_agent(agent_id);
                }

                // Normalize tool name for brand lookup (same as dispatch_tool)
                let brand_name = tool_name.replace('_', ".");

                // MCP spec: tool execution errors -> isError content, not JSON-RPC errors
                match self.dispatch_tool(tool_name, &arguments) {
                    Ok(result) => {
                        let stamped = brand::stamp(&brand_name, result);
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id: request.id.clone(),
                            result: Some(serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&stamped).unwrap_or_default(),
                                }]
                            })),
                            error: None,
                        });
                    }
                    Err(e) => {
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id: request.id.clone(),
                            result: Some(serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": brand::error_line(&format!("{}", e)),
                                }],
                                "isError": true
                            })),
                            error: None,
                        });
                    }
                }
            }
            _ => {
                // Method not found — JSON-RPC protocol error
                return Ok(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", method),
                        data: None,
                    }),
                });
            }
        }
    }

    /// Dispatch a tool call by name (Theme 8: split into sub-dispatchers).
    /// Tool name normalization: _ → . before matching.
    fn dispatch_tool(
        &mut self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> M1ndResult<serde_json::Value> {
        // Theme 8: normalize underscores to dots
        let normalized = tool_name.replace('_', ".");

        match normalized.as_str() {
            name if name.starts_with("m1nd.perspective.") => {
                self.dispatch_perspective_tool(name, params)
            }
            name if name.starts_with("m1nd.lock.") => {
                self.dispatch_lock_tool(name, params)
            }
            _ => self.dispatch_core_tool(&normalized, params),
        }
    }

    /// Dispatch core tools (existing 43 tools).
    fn dispatch_core_tool(
        &mut self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> M1ndResult<serde_json::Value> {
        match tool_name {
            "m1nd.activate" => {
                let input: ActivateInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = tools::handle_activate(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.impact" => {
                let input: ImpactInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = tools::handle_impact(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.missing" => {
                let input: MissingInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_missing(&mut self.state, input)
            }
            "m1nd.why" => {
                let input: WhyInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_why(&mut self.state, input)
            }
            "m1nd.warmup" => {
                let input: WarmupInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_warmup(&mut self.state, input)
            }
            "m1nd.counterfactual" => {
                let input: CounterfactualInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_counterfactual(&mut self.state, input)
            }
            "m1nd.predict" => {
                let input: PredictInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_predict(&mut self.state, input)
            }
            "m1nd.fingerprint" => {
                let input: FingerprintInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_fingerprint(&mut self.state, input)
            }
            "m1nd.drift" => {
                let input: DriftInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_drift(&mut self.state, input)
            }
            "m1nd.learn" => {
                let input: LearnInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_learn(&mut self.state, input)
            }
            "m1nd.ingest" => {
                let input: IngestInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_ingest(&mut self.state, input)
            }
            "m1nd.resonate" => {
                let input: ResonateInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                tools::handle_resonate(&mut self.state, input)
            }
            "m1nd.health" => {
                let input: HealthInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = tools::handle_health(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }

            // =============================================================
            // L2-L7: Superpowers layer tools
            // =============================================================
            "m1nd.seek" => {
                let input: layers::SeekInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_seek(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.scan" => {
                let input: layers::ScanInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_scan(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.timeline" => {
                let input: layers::TimelineInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_timeline(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.diverge" => {
                let input: layers::DivergeInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_diverge(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.trail.save" => {
                let input: layers::TrailSaveInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_trail_save(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.trail.resume" => {
                let input: layers::TrailResumeInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_trail_resume(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.trail.merge" => {
                let input: layers::TrailMergeInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_trail_merge(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.trail.list" => {
                let input: layers::TrailListInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_trail_list(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.hypothesize" => {
                let input: layers::HypothesizeInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_hypothesize(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.differential" => {
                let input: layers::DifferentialInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_differential(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.trace" => {
                let input: layers::TraceInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_trace(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.validate.plan" => {
                let input: layers::ValidatePlanInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_validate_plan(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }
            "m1nd.federate" => {
                let input: layers::FederateInput = serde_json::from_value(params.clone())
                    .map_err(M1ndError::Serde)?;
                let output = layer_handlers::handle_federate(&mut self.state, input)?;
                serde_json::to_value(output).map_err(M1ndError::Serde)
            }

            _ => Err(M1ndError::UnknownTool { name: tool_name.to_string() }),
        }
    }

    /// Dispatch perspective tools (12 tools).
    fn dispatch_perspective_tool(
        &mut self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> M1ndResult<serde_json::Value> {
        use crate::protocol::perspective::*;
        use crate::perspective_handlers;

        match tool_name {
            "m1nd.perspective.start" => {
                let input: PerspectiveStartInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.start".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_start(&mut self.state, input)
            }
            "m1nd.perspective.routes" => {
                let input: PerspectiveRoutesInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.routes".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_routes(&mut self.state, input)
            }
            "m1nd.perspective.inspect" => {
                let input: PerspectiveInspectInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.inspect".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_inspect(&mut self.state, input)
            }
            "m1nd.perspective.peek" => {
                let input: PerspectivePeekInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.peek".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_peek(&mut self.state, input)
            }
            "m1nd.perspective.follow" => {
                let input: PerspectiveFollowInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.follow".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_follow(&mut self.state, input)
            }
            "m1nd.perspective.suggest" => {
                let input: PerspectiveSuggestInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.suggest".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_suggest(&mut self.state, input)
            }
            "m1nd.perspective.affinity" => {
                let input: PerspectiveAffinityInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.affinity".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_affinity(&mut self.state, input)
            }
            "m1nd.perspective.branch" => {
                let input: PerspectiveBranchInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.branch".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_branch(&mut self.state, input)
            }
            "m1nd.perspective.back" => {
                let input: PerspectiveBackInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.back".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_back(&mut self.state, input)
            }
            "m1nd.perspective.compare" => {
                let input: PerspectiveCompareInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.compare".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_compare(&mut self.state, input)
            }
            "m1nd.perspective.list" => {
                let input: PerspectiveListInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.list".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_list(&self.state, input)
            }
            "m1nd.perspective.close" => {
                let input: PerspectiveCloseInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.perspective.close".into(),
                        detail: e.to_string(),
                    })?;
                perspective_handlers::handle_perspective_close(&mut self.state, input)
            }
            _ => Err(M1ndError::UnknownTool { name: tool_name.to_string() }),
        }
    }

    /// Dispatch lock tools (5 tools).
    fn dispatch_lock_tool(
        &mut self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> M1ndResult<serde_json::Value> {
        use crate::protocol::lock::*;
        use crate::lock_handlers;

        match tool_name {
            "m1nd.lock.create" => {
                let input: LockCreateInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.lock.create".into(),
                        detail: e.to_string(),
                    })?;
                lock_handlers::handle_lock_create(&mut self.state, input)
            }
            "m1nd.lock.watch" => {
                let input: LockWatchInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.lock.watch".into(),
                        detail: e.to_string(),
                    })?;
                lock_handlers::handle_lock_watch(&mut self.state, input)
            }
            "m1nd.lock.diff" => {
                let input: LockDiffInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.lock.diff".into(),
                        detail: e.to_string(),
                    })?;
                lock_handlers::handle_lock_diff(&mut self.state, input)
            }
            "m1nd.lock.rebase" => {
                let input: LockRebaseInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.lock.rebase".into(),
                        detail: e.to_string(),
                    })?;
                lock_handlers::handle_lock_rebase(&mut self.state, input)
            }
            "m1nd.lock.release" => {
                let input: LockReleaseInput = serde_json::from_value(params.clone())
                    .map_err(|e| M1ndError::InvalidParams {
                        tool: "m1nd.lock.release".into(),
                        detail: e.to_string(),
                    })?;
                lock_handlers::handle_lock_release(&mut self.state, input)
            }
            _ => Err(M1ndError::UnknownTool { name: tool_name.to_string() }),
        }
    }
}
