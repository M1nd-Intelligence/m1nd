#![allow(unused)]
#![recursion_limit = "512"]

pub mod cli;
pub mod protocol;
pub mod server;
pub mod session;
pub mod tools;

// Perspective MCP — stateful navigation layer (12-PERSPECTIVE-SYNTHESIS)
pub mod engine_ops;
pub mod layer_handlers;
pub mod lock_handlers;
pub mod perspective;
pub mod perspective_handlers;
pub mod surgical_handlers;
pub mod persist_handlers;

// v0.4.0: new tool handlers + personality
pub mod personality;
pub mod report_handlers;
pub mod search_handlers;

// HTTP server + types (feature-gated behind "serve")
#[cfg(feature = "serve")]
pub mod http_server;
#[cfg(feature = "serve")]
pub mod http_types;