#![allow(unused)]
#![recursion_limit = "512"]

pub mod server;
pub mod protocol;
pub mod tools;
pub mod session;

// Perspective MCP — stateful navigation layer (12-PERSPECTIVE-SYNTHESIS)
pub mod perspective;
pub mod engine_ops;
pub mod perspective_handlers;
pub mod lock_handlers;
pub mod layer_handlers;
