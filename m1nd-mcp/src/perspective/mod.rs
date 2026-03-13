// === m1nd-mcp/src/perspective/mod.rs ===
// Perspective subsystem: stateful navigation, locking, and route management.
// From 12-PERSPECTIVE-SYNTHESIS prerequisite chain.
//
// Submodules:
// - state:         Core types (PerspectiveState, LockState, enums, limits)
// - keys:          Content-addressable edge keys and route IDs (Theme 4)
// - validation:    Input validation and parameter bounds (Theme 9)
// - confidence:    Confidence calibration and epistemic safety (Theme 13)
// - peek_security: Peek file security pipeline (Theme 6)

pub mod state;
pub mod keys;
pub mod validation;
pub mod confidence;
pub mod peek_security;
