// === m1nd-mcp/src/brand.rs ===
//
// m1nd visual identity system.
// Unicode brand symbols chosen by the creator to represent the neuro-symbolic
// nature of m1nd — circuits, graphs, connections, dimensions.
//
// Symbol semantics:
//   ⍌  signal / flow       — activation, spreading, ingest, learn, warmup
//   ⍐  path / trace        — why, trace, timeline, routes, seek
//   ⍂  structure / holes   — missing, fingerprint, scan, topology, diverge
//   𝔻  dimension / analysis — impact, predict, counterfactual, differential, hypothesize
//   🁢  state / grid        — health, status, locks, trails, drift, validate_plan
//   ⟁  connection / graph  — perspective, federation, resonance

// ---------------------------------------------------------------------------
// Core symbols
// ---------------------------------------------------------------------------

/// Signal / flow — spreading activation, data flowing through the graph.
pub const SYM_SIGNAL: &str = "\u{234C}";     // ⍌

/// Path / trace — following connections, tracing dependencies.
pub const SYM_PATH: &str = "\u{2350}";       // ⍐

/// Structure / holes — structural analysis, gap detection, topology.
pub const SYM_STRUCTURE: &str = "\u{2342}";  // ⍂

/// Dimension / analysis — multi-dimensional impact, prediction, what-if.
pub const SYM_DIMENSION: &str = "\u{1D53B}"; // 𝔻

/// State / grid — system state, locks, trails, health status.
pub const SYM_STATE: &str = "\u{1F062}";     // 🁢

/// Connection / graph — perspectives, federation, resonance harmonics.
pub const SYM_CONNECTION: &str = "\u{27C1}"; // ⟁

// ---------------------------------------------------------------------------
// Composite banners
// ---------------------------------------------------------------------------

/// Full banner: all six symbols framing the name.
pub const BANNER: &str = "\u{234C}\u{2350}\u{2342}\u{1D53B} m1nd \u{1F062}\u{27C1}";

/// Compact signature for log lines.
pub const SIG: &str = "\u{234C}\u{1D53B}\u{27C1}";

// ---------------------------------------------------------------------------
// Tool → symbol mapping
// ---------------------------------------------------------------------------

/// Returns the brand symbol prefix for a given tool name.
/// Used to stamp every tool response with a semantic identity marker.
pub fn tool_glyph(tool_name: &str) -> &'static str {
    match tool_name {
        // ⍌ signal / flow
        "m1nd.activate"         => SYM_SIGNAL,
        "m1nd.ingest"           => SYM_SIGNAL,
        "m1nd.learn"            => SYM_SIGNAL,
        "m1nd.warmup"           => SYM_SIGNAL,

        // ⍐ path / trace
        "m1nd.why"              => SYM_PATH,
        "m1nd.trace"            => SYM_PATH,
        "m1nd.timeline"         => SYM_PATH,
        "m1nd.seek"             => SYM_PATH,

        // ⍂ structure / holes
        "m1nd.missing"          => SYM_STRUCTURE,
        "m1nd.fingerprint"      => SYM_STRUCTURE,
        "m1nd.scan"             => SYM_STRUCTURE,
        "m1nd.diverge"          => SYM_STRUCTURE,

        // 𝔻 dimension / analysis
        "m1nd.impact"           => SYM_DIMENSION,
        "m1nd.predict"          => SYM_DIMENSION,
        "m1nd.counterfactual"   => SYM_DIMENSION,
        "m1nd.differential"     => SYM_DIMENSION,
        "m1nd.hypothesize"      => SYM_DIMENSION,

        // 🁢 state / grid
        "m1nd.health"           => SYM_STATE,
        "m1nd.drift"            => SYM_STATE,
        "m1nd.lock.create"      => SYM_STATE,
        "m1nd.lock.watch"       => SYM_STATE,
        "m1nd.lock.diff"        => SYM_STATE,
        "m1nd.lock.rebase"      => SYM_STATE,
        "m1nd.lock.release"     => SYM_STATE,
        "m1nd.trail.save"       => SYM_STATE,
        "m1nd.trail.resume"     => SYM_STATE,
        "m1nd.trail.merge"      => SYM_STATE,
        "m1nd.trail.list"       => SYM_STATE,
        "m1nd.validate.plan"    => SYM_STATE,

        // ⟁ connection / graph
        "m1nd.resonate"             => SYM_CONNECTION,
        "m1nd.federate"             => SYM_CONNECTION,
        "m1nd.perspective.start"    => SYM_CONNECTION,
        "m1nd.perspective.routes"   => SYM_CONNECTION,
        "m1nd.perspective.inspect"  => SYM_CONNECTION,
        "m1nd.perspective.peek"     => SYM_CONNECTION,
        "m1nd.perspective.follow"   => SYM_CONNECTION,
        "m1nd.perspective.suggest"  => SYM_CONNECTION,
        "m1nd.perspective.affinity" => SYM_CONNECTION,
        "m1nd.perspective.branch"   => SYM_CONNECTION,
        "m1nd.perspective.back"     => SYM_CONNECTION,
        "m1nd.perspective.compare"  => SYM_CONNECTION,
        "m1nd.perspective.list"     => SYM_CONNECTION,
        "m1nd.perspective.close"    => SYM_CONNECTION,

        // fallback
        _ => SIG,
    }
}

/// Returns a short human-readable category tag for the tool's semantic domain.
pub fn tool_domain(tool_name: &str) -> &'static str {
    match tool_name {
        "m1nd.activate" | "m1nd.ingest" | "m1nd.learn" | "m1nd.warmup"
            => "signal",
        "m1nd.why" | "m1nd.trace" | "m1nd.timeline" | "m1nd.seek"
            => "path",
        "m1nd.missing" | "m1nd.fingerprint" | "m1nd.scan" | "m1nd.diverge"
            => "structure",
        "m1nd.impact" | "m1nd.predict" | "m1nd.counterfactual"
            | "m1nd.differential" | "m1nd.hypothesize"
            => "dimension",
        "m1nd.health" | "m1nd.drift" | "m1nd.validate.plan"
            => "state",
        name if name.starts_with("m1nd.lock.")
            => "state",
        name if name.starts_with("m1nd.trail.")
            => "state",
        "m1nd.resonate" | "m1nd.federate"
            => "connection",
        name if name.starts_with("m1nd.perspective.")
            => "connection",
        _ => "signal",
    }
}

/// Stamp a JSON result value with the m1nd brand identity.
///
/// Injects a `_m1nd` key at the top level of the JSON object containing:
/// - `glyph`: the semantic symbol for this tool
/// - `tool`: the tool name
/// - `domain`: the semantic domain category
///
/// If the value is not an object, wraps it in one.
pub fn stamp(tool_name: &str, mut value: serde_json::Value) -> serde_json::Value {
    let glyph = tool_glyph(tool_name);
    let domain = tool_domain(tool_name);

    // Extract the short tool name (e.g. "activate" from "m1nd.activate")
    let short_name = tool_name.strip_prefix("m1nd.").unwrap_or(tool_name);

    let brand_meta = serde_json::json!({
        "glyph": glyph,
        "tool": short_name,
        "domain": domain,
    });

    match value {
        serde_json::Value::Object(ref mut map) => {
            map.insert("_m1nd".to_string(), brand_meta);
            value
        }
        other => {
            serde_json::json!({
                "_m1nd": brand_meta,
                "data": other,
            })
        }
    }
}

/// Format a branded text header line for the MCP text content.
/// Example: "⍌ m1nd.activate — \"query\" — 15 results in 31ms"
pub fn header(tool_name: &str, detail: &str) -> String {
    let glyph = tool_glyph(tool_name);
    let short_name = tool_name.strip_prefix("m1nd.").unwrap_or(tool_name);
    format!("{} m1nd.{} \u{2014} {}", glyph, short_name, detail)
}

/// Format a branded error line.
/// Example: "⍌𝔻⟁ m1nd error — tool not found: foo"
pub fn error_line(detail: &str) -> String {
    format!("{} m1nd error \u{2014} {}", SIG, detail)
}

/// Format a branded stderr log line.
/// Example: "[⍌𝔻⟁ m1nd] Server ready. 4927 nodes, 12345 edges"
pub fn log(msg: &str) -> String {
    format!("[{} m1nd] {}", SIG, msg)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_glyph_covers_all_core_tools() {
        // Verify no core tool falls through to the fallback
        let core_tools = [
            "m1nd.activate", "m1nd.impact", "m1nd.missing", "m1nd.why",
            "m1nd.warmup", "m1nd.counterfactual", "m1nd.predict",
            "m1nd.fingerprint", "m1nd.drift", "m1nd.learn", "m1nd.ingest",
            "m1nd.resonate", "m1nd.health",
        ];
        for tool in &core_tools {
            let glyph = tool_glyph(tool);
            assert_ne!(glyph, SIG, "Core tool {} should have a specific glyph, not fallback", tool);
        }
    }

    #[test]
    fn tool_glyph_covers_layer_tools() {
        let layer_tools = [
            "m1nd.seek", "m1nd.scan", "m1nd.timeline", "m1nd.diverge",
            "m1nd.trace", "m1nd.hypothesize", "m1nd.differential",
            "m1nd.federate", "m1nd.validate.plan",
        ];
        for tool in &layer_tools {
            let glyph = tool_glyph(tool);
            assert_ne!(glyph, SIG, "Layer tool {} should have a specific glyph", tool);
        }
    }

    #[test]
    fn tool_glyph_covers_perspective_tools() {
        let perspective_tools = [
            "m1nd.perspective.start", "m1nd.perspective.routes",
            "m1nd.perspective.follow", "m1nd.perspective.close",
        ];
        for tool in &perspective_tools {
            assert_eq!(tool_glyph(tool), SYM_CONNECTION);
        }
    }

    #[test]
    fn tool_glyph_covers_lock_tools() {
        let lock_tools = [
            "m1nd.lock.create", "m1nd.lock.watch", "m1nd.lock.diff",
            "m1nd.lock.rebase", "m1nd.lock.release",
        ];
        for tool in &lock_tools {
            assert_eq!(tool_glyph(tool), SYM_STATE);
        }
    }

    #[test]
    fn tool_glyph_covers_trail_tools() {
        let trail_tools = [
            "m1nd.trail.save", "m1nd.trail.resume",
            "m1nd.trail.merge", "m1nd.trail.list",
        ];
        for tool in &trail_tools {
            assert_eq!(tool_glyph(tool), SYM_STATE);
        }
    }

    #[test]
    fn stamp_injects_brand_on_object() {
        let val = serde_json::json!({"query": "test", "results": 5});
        let stamped = stamp("m1nd.activate", val);
        let m = stamped.get("_m1nd").expect("_m1nd field must exist");
        assert_eq!(m["glyph"].as_str().unwrap(), SYM_SIGNAL);
        assert_eq!(m["tool"].as_str().unwrap(), "activate");
        assert_eq!(m["domain"].as_str().unwrap(), "signal");
        // Original fields preserved
        assert_eq!(stamped["query"].as_str().unwrap(), "test");
    }

    #[test]
    fn stamp_wraps_non_object() {
        let val = serde_json::json!(42);
        let stamped = stamp("m1nd.health", val);
        assert!(stamped.get("_m1nd").is_some());
        assert_eq!(stamped["data"], 42);
    }

    #[test]
    fn header_format() {
        let h = header("m1nd.activate", "\"chat_handler\" \u{2014} 15 results in 31ms");
        assert!(h.starts_with(SYM_SIGNAL));
        assert!(h.contains("m1nd.activate"));
    }

    #[test]
    fn error_line_format() {
        let e = error_line("tool not found: foo");
        assert!(e.starts_with(SIG));
        assert!(e.contains("m1nd error"));
    }

    #[test]
    fn log_format() {
        let l = log("Server ready");
        assert!(l.contains(SIG));
        assert!(l.contains("m1nd"));
    }

    #[test]
    fn banner_contains_all_symbols() {
        assert!(BANNER.contains(SYM_SIGNAL));
        assert!(BANNER.contains(SYM_PATH));
        assert!(BANNER.contains(SYM_STRUCTURE));
        assert!(BANNER.contains("m1nd"));
    }

    #[test]
    fn semantic_groupings_are_correct() {
        // Signal domain
        assert_eq!(tool_domain("m1nd.activate"), "signal");
        assert_eq!(tool_domain("m1nd.ingest"), "signal");

        // Path domain
        assert_eq!(tool_domain("m1nd.why"), "path");
        assert_eq!(tool_domain("m1nd.seek"), "path");

        // Structure domain
        assert_eq!(tool_domain("m1nd.missing"), "structure");
        assert_eq!(tool_domain("m1nd.scan"), "structure");

        // Dimension domain
        assert_eq!(tool_domain("m1nd.impact"), "dimension");
        assert_eq!(tool_domain("m1nd.predict"), "dimension");

        // State domain
        assert_eq!(tool_domain("m1nd.health"), "state");
        assert_eq!(tool_domain("m1nd.lock.create"), "state");

        // Connection domain
        assert_eq!(tool_domain("m1nd.resonate"), "connection");
        assert_eq!(tool_domain("m1nd.perspective.start"), "connection");
    }
}
