// === crates/m1nd-mcp/src/main.rs ===
//
// m1nd MCP server binary. Reads config, starts server, serves over stdio.
// Handles SIGINT for graceful shutdown.

use m1nd_mcp::brand;
use m1nd_mcp::server::{McpConfig, McpServer};
use std::path::PathBuf;

fn load_config() -> McpConfig {
    // 1. Try config file from args
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let path = &args[1];
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<McpConfig>(&contents) {
                eprintln!("{}", brand::log_colored(&format!("Config loaded from {}", path)));
                return config;
            }
        }
    }

    // 2. Try env vars
    let graph_source = std::env::var("M1ND_GRAPH_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./graph_snapshot.json"));

    let plasticity_state = std::env::var("M1ND_PLASTICITY_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./plasticity_state.json"));

    let xlr_enabled = std::env::var("M1ND_XLR_ENABLED")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true);

    McpConfig {
        graph_source,
        plasticity_state,
        xlr_enabled,
        ..McpConfig::default()
    }
}

#[tokio::main]
async fn main() {
    eprintln!("{}", brand::banner_colored());

    let config = load_config();

    let mut server = match McpServer::new(config) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", brand::log_colored(&format!("Failed to create server: {}", e)));
            std::process::exit(1);
        }
    };

    if let Err(e) = server.start() {
        eprintln!("{}", brand::log_colored(&format!("Failed to start server: {}", e)));
        std::process::exit(1);
    }

    // Spawn the serve loop in a blocking task (it does synchronous stdio I/O)
    let serve_handle = tokio::task::spawn_blocking(move || {
        let result = server.serve();
        // On EOF or error, shutdown
        let _ = server.shutdown();
        result
    });

    // Wait for either SIGINT or serve completion
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            eprintln!("{}", brand::log_colored("SIGINT received."));
            // serve_handle will complete when stdin closes or next iteration
        }
        result = serve_handle => {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => eprintln!("{}", brand::log_colored(&format!("Server error: {}", e))),
                Err(e) => eprintln!("{}", brand::log_colored(&format!("Task error: {}", e))),
            }
        }
    }
}
