// === m1nd-mcp CLI argument parsing ===
//
// Clap derive struct for m1nd-mcp binary modes.
// Replaces manual std::env::args() parsing in main.rs.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "m1nd-mcp", about = "Neuro-symbolic connectome engine", version)]
pub struct Cli {
    /// Start HTTP server with embedded web UI
    #[arg(long)]
    pub serve: bool,

    /// HTTP server port
    #[arg(long, default_value = "1337")]
    pub port: u16,

    /// Bind address override (default: 127.0.0.1). Use 0.0.0.0 for network access.
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Serve frontend from disk instead of embedded (dev mode)
    #[arg(long)]
    pub dev: bool,

    /// Also run JSON-RPC stdio server alongside HTTP
    #[arg(long)]
    pub stdio: bool,

    /// Auto-open browser on startup
    #[arg(long)]
    pub open: bool,

    /// Path to config JSON file
    #[arg(long)]
    pub config: Option<String>,

    /// Graph source path override
    #[arg(long)]
    pub graph: Option<String>,

    /// Plasticity state path override
    #[arg(long)]
    pub plasticity: Option<String>,

    /// Domain: code, music, memory, generic
    #[arg(long, default_value = "code")]
    pub domain: String,

    /// Disable auto-launching the HTTP GUI in stdio mode (for CI, headless servers)
    #[arg(long)]
    pub no_gui: bool,

    /// Path to event log file (append-only JSON lines). Enables cross-process SSE via file bus.
    #[arg(long)]
    pub event_log: Option<String>,

    /// Watch an event log file and broadcast new events via SSE (HTTP-only mode).
    /// Use when a separate stdio process writes events to this file.
    #[arg(long)]
    pub watch_events: Option<String>,
}
