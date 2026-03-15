// === m1nd-mcp/src/protocol/mod.rs ===
//
// Split protocol type system (Theme 8: Dispatch Architecture).
// - core:        Existing 13 tool input/output types
// - perspective:  11 perspective tool + 2 management tool types
// - lock:         5 lock tool types

pub mod core;
pub mod perspective;
pub mod lock;
pub mod layers;
pub mod surgical;

// Re-export core types so existing `use crate::protocol::*` continues to work.
pub use self::core::*;
