// === crates/m1nd-mcp/src/server.rs ===

use m1nd_core::domain::DomainConfig;
use m1nd_core::error::{M1ndError, M1ndResult};
use crate::session::SessionState;
use crate::protocol::*;
use crate::tools;
use std::io::{BufRead, Write};
use std::path::PathBuf;

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
                "description": "Ingest or re-ingest a codebase",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Filesystem path to the codebase root" },
                        "agent_id": { "type": "string", "description": "Calling agent identifier" },
                        "incremental": { "type": "boolean", "default": false, "description": "Incremental ingest (only changed files)" }
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
        let reader = stdin.lock();
        let mut writer = stdout.lock();

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break, // EOF or read error
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
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
                    let _ = writeln!(writer, "{}", serde_json::to_string(&err_resp).unwrap_or_default());
                    let _ = writer.flush();
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

            let json = serde_json::to_string(&resp).unwrap_or_default();
            if writeln!(writer, "{}", json).is_err() {
                break; // stdout closed
            }
            let _ = writer.flush();
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
            "initialize" => {
                return Ok(JsonRpcResponse {
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

                // MCP spec: tool execution errors -> isError content, not JSON-RPC errors
                match self.dispatch_tool(tool_name, &arguments) {
                    Ok(result) => {
                        return Ok(JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id: request.id.clone(),
                            result: Some(serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&result).unwrap_or_default(),
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
                                    "text": format!("Error: {}", e),
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

    /// Dispatch a tool call by name.
    fn dispatch_tool(
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
            _ => Err(M1ndError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown tool: {}", tool_name),
            ))),
        }
    }
}
