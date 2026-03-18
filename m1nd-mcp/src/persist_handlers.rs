// === m1nd-mcp/src/persist_handlers.rs ===
// Persist/load handler with optional binary snapshots.

use crate::session::SessionState;
use m1nd_core::error::M1ndResult;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct PersistInput {
    pub agent_id: String,
    pub action: String, // save | load | checkpoint | status
    #[serde(default)]
    pub format: Option<String>, // json | bin (default json)
    #[serde(default)]
    pub path: Option<String>,  // override snapshot path
}

pub fn handle_persist(state: &mut SessionState, input: PersistInput) -> M1ndResult<serde_json::Value> {
    let fmt = input.format.as_deref().unwrap_or("json");
    let is_bin = fmt.eq_ignore_ascii_case("bin");

    // Default path: graph_path (json) or graph_path with .bin extension
    let default_path = if is_bin {
        state.graph_path.with_extension("bin")
    } else {
        state.graph_path.clone()
    };
    let path: PathBuf = input
        .path
        .as_ref()
        .map(|s| PathBuf::from(s))
        .unwrap_or(default_path.clone());

    match input.action.as_str() {
        "status" => {
            let g = state.graph.read();
            let result = serde_json::json!({
                "status": "ok",
                "nodes": g.num_nodes(),
                "edges": g.num_edges(),
                "queries_processed": state.queries_processed,
                "graph_path": state.graph_path.to_string_lossy(),
                "plasticity_path": state.plasticity_path.to_string_lossy(),
                "snapshot_path": path.to_string_lossy(),
                "format_supported": ["json", "bin"],
            });
            Ok(result)
        }
        "checkpoint" | "save" => {
            if is_bin {
                // Save graph to binary snapshot + keep JSON/plasticity as today
                let g = state.graph.read();
                m1nd_core::snapshot_bin::save_graph(&g, &path)?;
                // Plasticity stays JSON for readability
                if let Ok(states) = state.plasticity.export_state(&g) {
                    let _ = m1nd_core::snapshot::save_plasticity_state(&states, &state.plasticity_path);
                }
            } else {
                // Existing JSON path (graph + plasticity + antibodies)
                let _ = state.persist();
            }
            state.track_agent(&input.agent_id);
            Ok(serde_json::json!({
                "status": "saved",
                "graph_path": state.graph_path.to_string_lossy(),
                "plasticity_path": state.plasticity_path.to_string_lossy(),
                "bin_path": if is_bin { Some(path.to_string_lossy().to_string()) } else { None }
            }))
        }
        "load" => {
            // Load either JSON or BIN snapshot
            let graph = if is_bin {
                m1nd_core::snapshot_bin::load_graph(&path)?
            } else {
                m1nd_core::snapshot::load_graph(&path)?
            };

            // Swap graph and rebuild engines
            state.graph = Arc::new(parking_lot::RwLock::new(graph));
            state.rebuild_engines()?;
            state.bump_graph_generation();
            state.track_agent(&input.agent_id);

            let g = state.graph.read();
            Ok(serde_json::json!({
                "status": "loaded",
                "format": if is_bin { "bin" } else { "json" },
                "snapshot_path": path.to_string_lossy(),
                "nodes": g.num_nodes(),
                "edges": g.num_edges(),
            }))
        }
        other => Ok(serde_json::json!({
            "status": "error",
            "message": format!("Unknown action: {}", other)
        })),
    }
}