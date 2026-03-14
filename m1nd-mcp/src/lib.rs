#![allow(unused)]
#![recursion_limit = "512"]

pub mod brand;
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
