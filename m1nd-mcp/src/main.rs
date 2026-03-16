// === m1nd-mcp binary entry point ===
//
// Modes:
//   m1nd-mcp                     → JSON-RPC stdio + auto-launch GUI on :1337 (default)
//   m1nd-mcp --no-gui            → JSON-RPC stdio only (CI, headless)
//   m1nd-mcp --serve             → HTTP server + embedded UI on :1337
//   m1nd-mcp --serve --stdio     → Both transports simultaneously (SSE cross-process bridge)
//   m1nd-mcp --serve --dev       → HTTP with frontend served from disk (Vite HMR)
//   m1nd-mcp --serve --open      → HTTP + auto-open browser
//
// Cross-process SSE (new):
//   m1nd-mcp --serve --stdio                           → Option A: same process, shared state + broadcast
//   m1nd-mcp --serve --stdio --event-log /tmp/e.jsonl  → Option A + B: same process + file event bus
//   m1nd-mcp --serve --watch-events /tmp/e.jsonl       → Option B consumer: watch file, broadcast SSE

use clap::Parser;
use m1nd_mcp::cli::Cli;
use m1nd_mcp::server::{McpConfig, McpServer};
use std::path::PathBuf;

fn load_config_from_cli(cli: &Cli) -> McpConfig {
    // Priority: --config file > --graph/--plasticity/--domain flags > env vars > defaults

    // 1. Try config file
    if let Some(ref path) = cli.config {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<McpConfig>(&contents) {
                eprintln!("[m1nd-mcp] Config loaded from {}", path);
                return config;
            }
        }
    }

    // 2. Build from CLI flags + env vars
    let graph_source = cli
        .graph
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| std::env::var("M1ND_GRAPH_SOURCE").ok().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("./graph_snapshot.json"));

    let plasticity_state = cli
        .plasticity
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("M1ND_PLASTICITY_STATE")
                .ok()
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| PathBuf::from("./plasticity_state.json"));

    let xlr_enabled = std::env::var("M1ND_XLR_ENABLED")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);

    let domain = match cli.domain.as_str() {
        "code" | "music" | "memory" | "generic" => Some(cli.domain.clone()),
        _ => None,
    };

    McpConfig {
        graph_source,
        plasticity_state,
        xlr_enabled,
        domain,
        ..McpConfig::default()
    }
}

async fn run_stdio_server(config: McpConfig, event_log: Option<String>, no_gui: bool, port: u16) {
    if event_log.is_some() {
        eprintln!(
            "[m1nd-mcp] NOTE: --event-log in stdio-only mode writes events for external consumers."
        );
        eprintln!(
            "[m1nd-mcp]       For cross-process SSE, use --serve --stdio --event-log <path>."
        );
    }

    // Spawn background HTTP GUI server (unless --no-gui or serve feature disabled)
    #[cfg(feature = "serve")]
    let _gui_handle = if !no_gui {
        // Create a separate McpServer for the HTTP GUI (same config, independent state)
        match McpServer::new(config.clone()) {
            Ok(gui_server) => {
                let session_state = gui_server.into_session_state();
                let session = std::sync::Arc::new(parking_lot::Mutex::new(session_state));
                Some(m1nd_mcp::http_server::spawn_background(session, port))
            }
            Err(e) => {
                eprintln!(
                    "[m1nd-mcp] GUI server init failed (continuing without GUI): {}",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    #[cfg(not(feature = "serve"))]
    let _ = (no_gui, port); // suppress unused warnings

    let mut server = match McpServer::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[m1nd-mcp] Failed to create server: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = server.start() {
        eprintln!("[m1nd-mcp] Failed to start server: {}", e);
        std::process::exit(1);
    }

    // Spawn the serve loop in a blocking task (synchronous stdio I/O)
    let serve_handle = tokio::task::spawn_blocking(move || {
        let result = server.serve();
        let _ = server.shutdown();
        result
    });

    // Wait for either SIGINT or serve completion
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[m1nd-mcp] SIGINT received.");
        }
        result = serve_handle => {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => eprintln!("[m1nd-mcp] Server error: {}", e),
                Err(e) => eprintln!("[m1nd-mcp] Task error: {}", e),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = load_config_from_cli(&cli);

    let event_log = cli.event_log;
    let watch_events = cli.watch_events;

    if cli.serve {
        #[cfg(feature = "serve")]
        {
            m1nd_mcp::http_server::run(
                config,
                cli.port,
                cli.bind,
                cli.dev,
                cli.open,
                cli.stdio,
                event_log,
                watch_events,
            )
            .await;
        }
        #[cfg(not(feature = "serve"))]
        {
            let _ = (event_log, watch_events); // suppress unused warnings
            eprintln!("[m1nd-mcp] --serve requires the 'serve' feature.");
            eprintln!("  Rebuild with: cargo build --release --features serve");
            std::process::exit(1);
        }
    } else {
        run_stdio_server(config, event_log, cli.no_gui, cli.port).await;
    }
}
