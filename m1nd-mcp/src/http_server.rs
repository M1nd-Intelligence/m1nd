// === m1nd-mcp HTTP server (axum) ===
//
// Embedded web UI server. Feature-gated behind "serve".
// Provides REST API for all 52 MCP tools + graph visualization endpoints.
// Uses the same dispatch_tool() free function as the stdio JSON-RPC transport.

#![allow(clippy::duplicated_attributes)]
#![cfg(feature = "serve")]

use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{header, StatusCode, Uri},
    response::{sse, IntoResponse, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::StreamExt;
use parking_lot::Mutex;
use rust_embed::Embed;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::http_types::SubgraphQuery;
use crate::server::{dispatch_tool, tool_schemas, McpConfig};
use crate::session::SessionState;

// ---------------------------------------------------------------------------
// Event log: append-only JSON lines file for cross-process SSE (Option B)
// ---------------------------------------------------------------------------

/// Write an SSE event as a JSON line to the event log file.
/// Thread-safe: opens file with append mode on each write.
fn append_event_to_log(path: &std::path::Path, event: &SseEvent) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        if let Ok(line) = serde_json::to_string(event) {
            let _ = writeln!(f, "{}", line);
        }
    }
}

/// Watch an event log file and broadcast new events via SSE.
/// Polls every 100ms for new lines appended to the file.
async fn watch_event_log(path: std::path::PathBuf, tx: broadcast::Sender<SseEvent>) {
    use tokio::io::AsyncBufReadExt;

    // Wait for file to exist
    loop {
        if path.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let file = match tokio::fs::File::open(&path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[m1nd-mcp] Failed to open event log for watching: {}", e);
            return;
        }
    };

    // Seek to end — only read NEW events
    let mut reader = tokio::io::BufReader::new(file);
    // Read and discard existing content
    {
        let mut discard = String::new();
        loop {
            discard.clear();
            match reader.read_line(&mut discard).await {
                Ok(0) => break, // EOF
                Ok(_) => continue,
                Err(_) => break,
            }
        }
    }

    eprintln!(
        "[m1nd-mcp] Watching event log: {} (tailing new events)",
        path.display()
    );

    // Now poll for new lines
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // No new data — poll interval
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(event) = serde_json::from_str::<SseEvent>(trimmed) {
                        let _ = tx.send(event);
                    }
                }
            }
            Err(e) => {
                eprintln!("[m1nd-mcp] Event log read error: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Embedded UI assets (rust-embed)
// ---------------------------------------------------------------------------

#[derive(Embed)]
#[folder = "../m1nd-ui/dist/"]
#[prefix = ""]
struct UiAssets;

// ---------------------------------------------------------------------------
// SSE event type
// ---------------------------------------------------------------------------

/// SSE event emitted after tool execution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SseEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

/// Shared state for axum handlers.
pub struct AppState {
    pub session: Arc<Mutex<SessionState>>,
    pub tool_schemas_cache: serde_json::Value,
    pub event_tx: broadcast::Sender<SseEvent>,
    /// Optional event log path for cross-process SSE (Option B).
    pub event_log_path: Option<std::path::PathBuf>,
}

// ---------------------------------------------------------------------------
// Tool execution timeout
// ---------------------------------------------------------------------------

const TOOL_TIMEOUT_SECS: u64 = 120; // 2 min — ingest de pastas grandes (clawd/memory ~106 files) precisa

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Spawn the HTTP server in background, sharing an existing SessionState.
/// Used by stdio mode to also serve the GUI without blocking the stdio loop.
/// Returns the tokio JoinHandle for the server task.
pub fn spawn_background(
    session: Arc<Mutex<SessionState>>,
    port: u16,
) -> tokio::task::JoinHandle<()> {
    // Build tool schemas cache
    let schemas_full = tool_schemas();
    let tool_schemas_cache = schemas_full
        .get("tools")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));

    // SSE broadcast channel
    let (event_tx, _) = broadcast::channel::<SseEvent>(64);

    // AppState
    let app_state = Arc::new(AppState {
        session,
        tool_schemas_cache,
        event_tx,
        event_log_path: None,
    });

    // Router (embedded UI, not dev mode)
    let router = build_router(app_state, false);

    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .expect("valid socket addr");

    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                eprintln!("[m1nd-mcp] m1nd GUI: http://localhost:{}", port);
                // Auto-open browser after short delay
                let open_port = port;
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    let url = format!("http://localhost:{}", open_port);
                    let _ = open_browser(&url);
                });
                // Serve until process exits (no graceful shutdown needed — stdio owns lifecycle)
                let _ = axum::serve(listener, router).await;
            }
            Err(e) => {
                eprintln!(
                    "[m1nd-mcp] Background HTTP server failed to bind to {}: {} (GUI unavailable)",
                    addr, e
                );
            }
        }
    })
}

/// Start the HTTP server (and optionally stdio).
#[allow(clippy::too_many_arguments)]
pub async fn run(
    config: McpConfig,
    port: u16,
    bind: String,
    dev_mode: bool,
    auto_open: bool,
    also_stdio: bool,
    event_log: Option<String>,
    watch_events: Option<String>,
) {
    // Warn about network exposure
    if bind == "0.0.0.0" {
        eprintln!("[m1nd-mcp] WARNING: Binding to 0.0.0.0 exposes the server to the network.");
        eprintln!("[m1nd-mcp] WARNING: No authentication is configured. Anyone on the network can access the API.");
    }

    // 1. Create McpServer to load graph + build engines
    let server = match crate::server::McpServer::new(config.clone()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[m1nd-mcp] Failed to create server: {}", e);
            std::process::exit(1);
        }
    };

    // 2. Extract SessionState, wrap in Arc<Mutex> for shared access
    let session_state = server.into_session_state();
    let session = Arc::new(Mutex::new(session_state));

    // 3. Cache tool schemas (static, computed once)
    let schemas_full = tool_schemas();
    let tool_schemas_cache = schemas_full
        .get("tools")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));

    // 4. Create SSE broadcast channel (64 event buffer)
    let (event_tx, _) = broadcast::channel::<SseEvent>(64);

    // 5. Resolve event log path (used by both Option A stdio and Option B)
    let event_log_path = event_log.map(std::path::PathBuf::from);

    // 6. Build shared AppState
    let app_state = Arc::new(AppState {
        session: session.clone(),
        tool_schemas_cache,
        event_tx: event_tx.clone(),
        event_log_path: event_log_path.clone(),
    });

    // 6b. If --watch-events is specified, spawn the event log watcher
    if let Some(ref watch_path) = watch_events {
        let path = std::path::PathBuf::from(watch_path);
        let tx = event_tx.clone();
        tokio::spawn(watch_event_log(path, tx));
    }

    // 7. Build router
    let router = build_router(app_state, dev_mode);

    // 8. Optionally spawn stdio JSON-RPC alongside HTTP
    if also_stdio {
        let stdio_session = session.clone();
        let stdio_event_tx = event_tx.clone();
        let stdio_event_log = event_log_path.clone();
        tokio::task::spawn_blocking(move || {
            eprintln!("[m1nd-mcp] Stdio JSON-RPC also active (--stdio). SSE cross-process bridge enabled.");
            // Run a minimal stdio loop sharing the same session state
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            let mut reader = stdin.lock();
            let mut writer = stdout.lock();

            use std::io::{BufRead, Write};
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    _ => {}
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                // Attempt parse as JSON-RPC tool call
                if let Ok(req) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if req.get("method").and_then(|m| m.as_str()) == Some("tools/call") {
                        let tool_name = req
                            .pointer("/params/name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let arguments = req
                            .pointer("/params/arguments")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));
                        let result = {
                            let mut s = stdio_session.lock();
                            dispatch_tool(&mut s, tool_name, &arguments)
                        };

                        // Broadcast SSE event for cross-process visibility (Option A)
                        let sse_event = SseEvent {
                            event_type: "tool_result".to_string(),
                            data: serde_json::json!({
                                "tool": tool_name,
                                "source": "stdio",
                                "agent_id": arguments.get("agent_id").and_then(|v| v.as_str()).unwrap_or("unknown"),
                                "success": result.is_ok(),
                                "result_preview": match &result {
                                    Ok(v) => truncate_json(v, 500),
                                    Err(e) => serde_json::json!({"error": e.to_string()}),
                                },
                                "timestamp_ms": std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis() as u64)
                                    .unwrap_or(0),
                            }),
                        };
                        let _ = stdio_event_tx.send(sse_event.clone());

                        // Also write to event log file if configured (Option B)
                        if let Some(ref log_path) = stdio_event_log {
                            append_event_to_log(log_path, &sse_event);
                        }

                        let resp = match result {
                            Ok(output) => serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": req.get("id").cloned().unwrap_or(serde_json::Value::Null),
                                "result": { "content": [{ "type": "text", "text": serde_json::to_string(&output).unwrap_or_default() }] }
                            }),
                            Err(e) => serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": req.get("id").cloned().unwrap_or(serde_json::Value::Null),
                                "error": { "code": -32603, "message": e.to_string() }
                            }),
                        };
                        let _ = writeln!(
                            writer,
                            "{}",
                            serde_json::to_string(&resp).unwrap_or_default()
                        );
                        let _ = writer.flush();
                    } else if req.get("method").and_then(|m| m.as_str()) == Some("tools/list") {
                        let schemas = tool_schemas();
                        let resp = serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": req.get("id").cloned().unwrap_or(serde_json::Value::Null),
                            "result": schemas
                        });
                        let _ = writeln!(
                            writer,
                            "{}",
                            serde_json::to_string(&resp).unwrap_or_default()
                        );
                        let _ = writer.flush();
                    }
                }
            }
        });
    }

    // 8. Bind and serve
    let addr: std::net::SocketAddr = format!("{}:{}", bind, port).parse().unwrap_or_else(|_| {
        eprintln!("[m1nd-mcp] Invalid bind address: {}:{}", bind, port);
        std::process::exit(1);
    });

    eprintln!("[m1nd-mcp] HTTP server listening on http://{}", addr);

    // 9. Auto-open browser
    if auto_open {
        let url = format!("http://localhost:{}", port);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = open_browser(&url);
        });
    }

    // 10. Graceful shutdown on SIGINT
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[m1nd-mcp] Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    let shutdown_session = session.clone();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            eprintln!("[m1nd-mcp] SIGINT received, shutting down...");
            // Persist state on shutdown
            let mut s = shutdown_session.lock();
            if let Err(e) = s.persist() {
                eprintln!("[m1nd-mcp] Failed to persist state on shutdown: {}", e);
            }
            eprintln!("[m1nd-mcp] State persisted. Goodbye.");
        })
        .await
        .expect("HTTP server failed");
}

/// Open browser (cross-platform).
fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a JSON value to at most `max_chars` when serialized.
/// Returns the original value if small enough, otherwise a truncated string.
fn truncate_json(value: &serde_json::Value, max_chars: usize) -> serde_json::Value {
    let s = serde_json::to_string(value).unwrap_or_default();
    if s.len() <= max_chars {
        value.clone()
    } else {
        serde_json::Value::String(format!("{}...(truncated)", &s[..max_chars]))
    }
}

/// Current timestamp in milliseconds since epoch.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Router construction
// ---------------------------------------------------------------------------

/// Build the axum router with all routes.
pub fn build_router(state: Arc<AppState>, dev_mode: bool) -> Router {
    let api = Router::new()
        .route("/api/health", get(handle_health))
        .route("/api/tools", get(handle_list_tools))
        .route("/api/tools/{*tool_name}", post(handle_tool_call))
        .route("/api/graph/stats", get(handle_graph_stats))
        .route("/api/graph/subgraph", get(handle_subgraph))
        .route("/api/graph/snapshot", get(handle_graph_snapshot))
        .route("/api/events", get(handle_sse))
        .with_state(state.clone())
        .layer(DefaultBodyLimit::max(1_048_576)); // 1MB body limit (FM-A-004)

    if dev_mode {
        let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../m1nd-ui/dist");
        api.fallback_service(tower_http::services::ServeDir::new(ui_dir))
            .layer(CorsLayer::permissive())
    } else {
        api.fallback(serve_embedded_ui)
            .layer(CorsLayer::permissive())
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn handle_health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let state = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let session = state.session.lock();
        let graph = session.graph.read();
        let node_count = graph.num_nodes() as usize;
        let edge_count = graph.num_edges();
        drop(graph);
        serde_json::json!({
            "status": if node_count > 0 { "ok" } else { "empty" },
            "uptime_secs": session.uptime_seconds(),
            "node_count": node_count,
            "edge_count": edge_count,
            "queries_processed": session.queries_processed,
            "agent_sessions": session.session_summary(),
            "domain": session.domain.name.as_str(),
            "graph_generation": session.graph_generation,
            "plasticity_generation": session.plasticity_generation,
        })
    })
    .await
    .expect("spawn_blocking panicked");

    (StatusCode::OK, Json(result))
}

async fn handle_list_tools(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "tools": state.tool_schemas_cache })),
    )
}

async fn handle_tool_call(
    State(state): State<Arc<AppState>>,
    Path(tool_name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let event_tx = state.event_tx.clone();
    let event_log_path = state.event_log_path.clone();
    let tool_for_event = tool_name.clone();
    let agent_id_for_event = body
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let state = state.clone();
    let tool = tool_name.clone();

    // Wrap in timeout (FM-C-004: 30s per tool)
    let result = tokio::time::timeout(
        Duration::from_secs(TOOL_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            let mut session = state.session.lock();
            dispatch_tool(&mut session, &tool, &body)
        }),
    )
    .await;

    match result {
        Err(_elapsed) => {
            // Broadcast timeout event
            let sse_event = SseEvent {
                event_type: "tool_timeout".to_string(),
                data: serde_json::json!({
                    "tool": tool_for_event,
                    "source": "http",
                    "agent_id": agent_id_for_event,
                    "timeout_secs": TOOL_TIMEOUT_SECS,
                    "timestamp_ms": now_ms(),
                }),
            };
            let _ = event_tx.send(sse_event.clone());
            if let Some(ref log_path) = event_log_path {
                append_event_to_log(log_path, &sse_event);
            }

            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(serde_json::json!({
                    "error": "timeout",
                    "detail": format!("Tool execution exceeded {}s limit.", TOOL_TIMEOUT_SECS)
                })),
            )
                .into_response()
        }
        Ok(inner) => {
            let inner = inner.expect("spawn_blocking panicked");

            // Broadcast SSE event for the tool result
            let sse_event = SseEvent {
                event_type: "tool_result".to_string(),
                data: serde_json::json!({
                    "tool": tool_for_event,
                    "source": "http",
                    "agent_id": agent_id_for_event,
                    "success": inner.is_ok(),
                    "result_preview": match &inner {
                        Ok(v) => truncate_json(v, 500),
                        Err(e) => serde_json::json!({"error": e.to_string()}),
                    },
                    "timestamp_ms": now_ms(),
                }),
            };
            let _ = event_tx.send(sse_event.clone());
            if let Some(ref log_path) = event_log_path {
                append_event_to_log(log_path, &sse_event);
            }

            match inner {
                Ok(output) => (
                    StatusCode::OK,
                    Json(serde_json::json!({ "result": output })),
                )
                    .into_response(),
                Err(e) => {
                    let (status, error_type) = match &e {
                        m1nd_core::error::M1ndError::UnknownTool { .. } => {
                            (StatusCode::NOT_FOUND, "unknown_tool")
                        }
                        m1nd_core::error::M1ndError::InvalidParams { .. } => {
                            (StatusCode::BAD_REQUEST, "invalid_params")
                        }
                        m1nd_core::error::M1ndError::Serde(_) => {
                            (StatusCode::BAD_REQUEST, "invalid_json")
                        }
                        _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
                    };
                    (
                        status,
                        Json(serde_json::json!({
                            "error": error_type,
                            "detail": e.to_string(),
                        })),
                    )
                        .into_response()
                }
            }
        }
    }
}

async fn handle_subgraph(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SubgraphQuery>,
) -> impl IntoResponse {
    let state = state.clone();
    let top_k = params.clamped_top_k(); // Cap at 100 (FM-FE-001)
    let query = params.query.clone();

    let result: serde_json::Value = tokio::task::spawn_blocking(move || {
        let start = std::time::Instant::now();
        let mut session = state.session.lock();

        // 1. Run activate internally to get top-K nodes
        let activate_params = serde_json::json!({
            "query": query,
            "agent_id": "gui-subgraph",
            "top_k": top_k,
            "include_ghost_edges": true,
            "include_structural_holes": false,
        });
        let activate_result = dispatch_tool(&mut session, "m1nd_activate", &activate_params);

        match activate_result {
            Err(e) => {
                // Return empty subgraph on activate failure
                serde_json::json!({
                    "nodes": [],
                    "edges": [],
                    "meta": {
                        "total_nodes": 0,
                        "rendered_nodes": 0,
                        "query": query,
                        "elapsed_ms": start.elapsed().as_millis() as u64,
                        "error": e.to_string(),
                    }
                })
            }
            Ok(output) => {
                let graph = session.graph.read();
                let n = graph.num_nodes() as usize;

                // Build reverse map: NodeId -> external_id
                let mut node_to_ext: Vec<String> = vec![String::new(); n];
                for (interned, &nid) in &graph.id_to_node {
                    let idx = nid.as_usize();
                    if idx < n {
                        node_to_ext[idx] = graph.strings.resolve(*interned).to_string();
                    }
                }

                // 2. Extract activated node IDs from activate result
                let activated = output
                    .get("activated")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let total_nodes = activated.len();

                // Collect top_k node external IDs and resolve to NodeIds
                let mut top_node_ids: Vec<m1nd_core::types::NodeId> = Vec::new();
                let mut top_ext_ids: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                let mut subgraph_nodes: Vec<serde_json::Value> = Vec::new();

                for node_val in activated.iter().take(top_k) {
                    let ext_id = node_val
                        .get("node_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if ext_id.is_empty() {
                        continue;
                    }
                    if let Some(nid) = graph.resolve_id(ext_id) {
                        let idx = nid.as_usize();
                        if idx < n {
                            top_node_ids.push(nid);
                            top_ext_ids.insert(ext_id.to_string());

                            let label = graph.strings.resolve(graph.nodes.label[idx]).to_string();
                            let node_type_val = node_type_to_u8(graph.nodes.node_type[idx]);
                            let activation = node_val
                                .get("activation")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0) as f32;
                            let tags: Vec<String> = graph.nodes.tags[idx]
                                .iter()
                                .map(|&t| graph.strings.resolve(t).to_string())
                                .collect();
                            let provenance = graph.resolve_node_provenance(nid);
                            let pagerank = if idx < graph.nodes.pagerank.len() {
                                graph.nodes.pagerank[idx].get()
                            } else {
                                0.0
                            };

                            subgraph_nodes.push(serde_json::json!({
                                "id": ext_id,
                                "label": label,
                                "node_type": node_type_val,
                                "activation": activation,
                                "tags": tags,
                                "source_path": provenance.source_path,
                                "pagerank": pagerank,
                            }));
                        }
                    }
                }

                // 3. Collect edges between top-K nodes
                let mut subgraph_edges: Vec<serde_json::Value> = Vec::new();
                for &nid in &top_node_ids {
                    if !graph.finalized {
                        continue;
                    }
                    let range = graph.csr.out_range(nid);
                    for j in range {
                        let tgt = graph.csr.targets[j];
                        let tgt_idx = tgt.as_usize();
                        if tgt_idx < n && top_ext_ids.contains(&node_to_ext[tgt_idx]) {
                            let src_ext = &node_to_ext[nid.as_usize()];
                            let tgt_ext = &node_to_ext[tgt_idx];
                            let weight = graph
                                .csr
                                .read_weight(m1nd_core::types::EdgeIdx::new(j as u32))
                                .get();
                            let relation =
                                graph.strings.resolve(graph.csr.relations[j]).to_string();
                            subgraph_edges.push(serde_json::json!({
                                "source": src_ext,
                                "target": tgt_ext,
                                "weight": weight,
                                "relation": relation,
                            }));
                        }
                    }
                }

                // 4. Also add ghost edges from activate output
                if let Some(ghost_edges) = output.get("ghost_edges").and_then(|v| v.as_array()) {
                    for ge in ghost_edges {
                        let src = ge.get("source").and_then(|v| v.as_str()).unwrap_or("");
                        let tgt = ge.get("target").and_then(|v| v.as_str()).unwrap_or("");
                        if top_ext_ids.contains(src) && top_ext_ids.contains(tgt) {
                            let strength =
                                ge.get("strength").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            subgraph_edges.push(serde_json::json!({
                                "source": src,
                                "target": tgt,
                                "weight": strength,
                                "relation": "ghost",
                            }));
                        }
                    }
                }

                let rendered = subgraph_nodes.len();
                serde_json::json!({
                    "nodes": subgraph_nodes,
                    "edges": subgraph_edges,
                    "meta": {
                        "total_nodes": total_nodes,
                        "rendered_nodes": rendered,
                        "query": query,
                        "elapsed_ms": start.elapsed().as_millis() as u64,
                    }
                })
            }
        }
    })
    .await
    .expect("spawn_blocking panicked");

    (StatusCode::OK, Json(result))
}

async fn handle_graph_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let state = state.clone();
    let result = tokio::task::spawn_blocking(move || {
        let session = state.session.lock();
        let graph = session.graph.read();
        serde_json::json!({
            "node_count": graph.num_nodes(),
            "edge_count": graph.num_edges(),
            "domain": session.domain.name.as_str(),
            "namespaces": serde_json::Value::Array(vec![]),
            "memory_estimate_bytes": 0_usize,
        })
    })
    .await
    .expect("spawn_blocking panicked");

    (StatusCode::OK, Json(result))
}

async fn handle_graph_snapshot(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let state = state.clone();
    let result: serde_json::Value = tokio::task::spawn_blocking(move || {
        let session = state.session.lock();
        let graph = session.graph.read();
        let n = graph.num_nodes() as usize;

        // Build reverse map: NodeId -> external_id
        let mut node_to_ext: Vec<String> = vec![String::new(); n];
        for (interned, &nid) in &graph.id_to_node {
            let idx = nid.as_usize();
            if idx < n {
                node_to_ext[idx] = graph.strings.resolve(*interned).to_string();
            }
        }

        // Serialize nodes
        let mut nodes = Vec::with_capacity(n);
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            let label = graph.strings.resolve(graph.nodes.label[i]).to_string();
            let tags: Vec<String> = graph.nodes.tags[i]
                .iter()
                .map(|&t| graph.strings.resolve(t).to_string())
                .collect();
            let provenance = graph.resolve_node_provenance(m1nd_core::types::NodeId::new(i as u32));
            nodes.push(serde_json::json!({
                "external_id": node_to_ext[i],
                "label": label,
                "node_type": node_type_to_u8(graph.nodes.node_type[i]),
                "tags": tags,
                "last_modified": graph.nodes.last_modified[i],
                "change_frequency": graph.nodes.change_frequency[i].get(),
                "provenance": {
                    "source_path": provenance.source_path,
                    "line_start": provenance.line_start,
                    "line_end": provenance.line_end,
                    "namespace": provenance.namespace,
                    "canonical": provenance.canonical,
                },
            }));
        }

        // Serialize edges from CSR
        let mut edges = Vec::new();
        if graph.finalized {
            for src in 0..n {
                let range = graph.csr.out_range(m1nd_core::types::NodeId::new(src as u32));
                for j in range {
                    let tgt = graph.csr.targets[j].as_usize();
                    let dir = graph.csr.directions[j];
                    // For bidirectional edges, only save canonical direction
                    if dir == m1nd_core::types::EdgeDirection::Bidirectional && src > tgt {
                        continue;
                    }
                    let relation = graph.strings.resolve(graph.csr.relations[j]).to_string();
                    let weight = graph.csr.read_weight(m1nd_core::types::EdgeIdx::new(j as u32)).get();
                    edges.push(serde_json::json!({
                        "source_id": node_to_ext[src],
                        "target_id": node_to_ext[tgt],
                        "relation": relation,
                        "weight": weight,
                        "direction": if dir == m1nd_core::types::EdgeDirection::Bidirectional { 1 } else { 0 },
                        "inhibitory": graph.csr.inhibitory[j],
                        "causal_strength": graph.csr.causal_strengths[j].get(),
                    }));
                }
            }
        }

        serde_json::json!({
            "version": 1,
            "nodes": nodes,
            "edges": edges,
        })
    })
    .await
    .expect("spawn_blocking panicked");

    (StatusCode::OK, Json(result))
}

async fn handle_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures::Stream<Item = Result<sse::Event, std::convert::Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|event| async {
        match event {
            Ok(e) => {
                let sse_event = sse::Event::default()
                    .event(e.event_type)
                    .json_data(e.data)
                    .ok()?;
                Some(Ok(sse_event))
            }
            Err(_) => None,
        }
    });
    Sse::new(stream)
}

/// Map NodeType to u8 for JSON serialization.
fn node_type_to_u8(nt: m1nd_core::types::NodeType) -> u8 {
    use m1nd_core::types::NodeType;
    match nt {
        NodeType::File => 0,
        NodeType::Directory => 1,
        NodeType::Function => 2,
        NodeType::Class => 3,
        NodeType::Struct => 4,
        NodeType::Enum => 5,
        NodeType::Type => 6,
        NodeType::Module => 7,
        NodeType::Reference => 8,
        NodeType::Concept => 9,
        NodeType::Material => 10,
        NodeType::Process => 11,
        NodeType::Product => 12,
        NodeType::Supplier => 13,
        NodeType::Regulatory => 14,
        NodeType::System => 15,
        NodeType::Cost => 16,
        NodeType::Custom(v) => v,
    }
}

/// Serve embedded UI assets (rust-embed). SPA fallback to index.html.
async fn serve_embedded_ui(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match UiAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            // SPA fallback: serve index.html for client-side routing
            match UiAssets::get("index.html") {
                Some(content) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html".to_string())],
                    content.data.into_owned(),
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "UI not built").into_response(),
            }
        }
    }
}
