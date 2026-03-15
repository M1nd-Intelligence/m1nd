#![allow(unused)]
#![recursion_limit = "512"]

pub mod server;
pub mod protocol;
pub mod tools;
pub mod session;
pub mod cli;

// Perspective MCP — stateful navigation layer (12-PERSPECTIVE-SYNTHESIS)
pub mod perspective;
pub mod engine_ops;
pub mod perspective_handlers;
pub mod lock_handlers;
pub mod layer_handlers;
pub mod surgical_handlers;

// HTTP server + types (feature-gated behind "serve")
#[cfg(feature = "serve")]
pub mod http_server;
#[cfg(feature = "serve")]
pub mod http_types;
