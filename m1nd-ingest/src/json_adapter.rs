// === crates/m1nd-ingest/src/json_adapter.rs ===
//
// Domain-agnostic JSON ingestion adapter.
// Any domain (music production, supply chain, etc.) can describe its graph
// structure in a simple JSON format and ingest it into m1nd.

use m1nd_core::error::{M1ndError, M1ndResult};
use m1nd_core::graph::Graph;
use m1nd_core::types::*;
use std::time::Instant;

use crate::{IngestAdapter, IngestStats};

// ---------------------------------------------------------------------------
// JSON schema types (deserialized from input file)
// ---------------------------------------------------------------------------

/// A node in the JSON descriptor.
#[derive(Debug)]
struct JsonNode {
    id: String,
    label: String,
    node_type: String,
    tags: Vec<String>,
}

/// An edge in the JSON descriptor.
#[derive(Debug)]
struct JsonEdge {
    source: String,
    target: String,
    relation: String,
    weight: f32,
}

/// Top-level JSON descriptor.
#[derive(Debug)]
struct JsonDescriptor {
    nodes: Vec<JsonNode>,
    edges: Vec<JsonEdge>,
}

// ---------------------------------------------------------------------------
// Parsing helpers (using serde_json::Value to avoid Deserialize derive)
// ---------------------------------------------------------------------------

fn parse_descriptor(value: &serde_json::Value) -> M1ndResult<JsonDescriptor> {
    let empty_array = serde_json::Value::Array(vec![]);
    let nodes_val = value.get("nodes").unwrap_or(&empty_array);
    let edges_val = value.get("edges").unwrap_or(&empty_array);

    let nodes_arr = nodes_val.as_array().ok_or_else(|| {
        M1ndError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "\"nodes\" must be an array",
        ))
    })?;

    let edges_arr = edges_val.as_array().ok_or_else(|| {
        M1ndError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "\"edges\" must be an array",
        ))
    })?;

    let mut nodes = Vec::with_capacity(nodes_arr.len());
    for n in nodes_arr {
        let id = n
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let label = n
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();
        let node_type = n
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("Custom")
            .to_string();
        let tags = n
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        nodes.push(JsonNode {
            id,
            label,
            node_type,
            tags,
        });
    }

    let mut edges = Vec::with_capacity(edges_arr.len());
    for e in edges_arr {
        let source = e
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let target = e
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let relation = e
            .get("relation")
            .and_then(|v| v.as_str())
            .unwrap_or("relates_to")
            .to_string();
        let weight = e.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        edges.push(JsonEdge {
            source,
            target,
            relation,
            weight,
        });
    }

    Ok(JsonDescriptor { nodes, edges })
}

// ---------------------------------------------------------------------------
// NodeType mapping
// ---------------------------------------------------------------------------

/// Map a type string from JSON to the NodeType enum.
/// Supports all built-in variants plus domain-agnostic types.
fn map_node_type(type_str: &str) -> NodeType {
    match type_str {
        "File" => NodeType::File,
        "Directory" => NodeType::Directory,
        "Function" => NodeType::Function,
        "Class" => NodeType::Class,
        "Struct" => NodeType::Struct,
        "Enum" => NodeType::Enum,
        "Type" => NodeType::Type,
        "Module" => NodeType::Module,
        "Reference" => NodeType::Reference,
        "Concept" => NodeType::Concept,
        "Material" => NodeType::Material,
        "Process" => NodeType::Process,
        "Product" => NodeType::Product,
        "Supplier" => NodeType::Supplier,
        "Regulatory" => NodeType::Regulatory,
        "System" => NodeType::System,
        "Cost" => NodeType::Cost,
        // Anything else gets Custom(0)
        _ => NodeType::Custom(0),
    }
}

// ---------------------------------------------------------------------------
// JsonIngestAdapter
// ---------------------------------------------------------------------------

/// Ingests a graph from a JSON descriptor file.
///
/// The JSON format is domain-agnostic:
/// ```json
/// {
///   "nodes": [
///     { "id": "room::studio_a", "label": "Studio A", "type": "System", "tags": ["room", "main"] },
///     { "id": "bus::master", "label": "Master Bus", "type": "Process", "tags": ["bus"] }
///   ],
///   "edges": [
///     { "source": "room::studio_a", "target": "bus::master", "relation": "routes_to", "weight": 1.0 }
///   ]
/// }
/// ```
///
/// This is the escape hatch: ANY domain can write a JSON descriptor and ingest
/// it into m1nd without writing a custom adapter.
pub struct JsonIngestAdapter;

impl IngestAdapter for JsonIngestAdapter {
    fn domain(&self) -> &str {
        "json"
    }

    fn ingest(&self, root: &std::path::Path) -> M1ndResult<(Graph, IngestStats)> {
        let start = Instant::now();
        let mut stats = IngestStats::default();

        // Read and parse the JSON file
        let content = std::fs::read_to_string(root)?;
        let value: serde_json::Value = serde_json::from_str(&content)?;
        let descriptor = parse_descriptor(&value)?;

        stats.files_scanned = 1;
        stats.files_parsed = 1;

        // Build graph
        let estimated_nodes = descriptor.nodes.len();
        let estimated_edges = descriptor.edges.len();
        let mut graph = Graph::with_capacity(estimated_nodes, estimated_edges);

        // Phase 1: Add all nodes
        for node in &descriptor.nodes {
            let node_type = map_node_type(&node.node_type);
            let tags: Vec<&str> = node.tags.iter().map(|s| s.as_str()).collect();
            match graph.add_node(
                &node.id,
                &node.label,
                node_type,
                &tags,
                0.0, // no timestamp for JSON-sourced nodes
                0.3, // default change frequency
            ) {
                Ok(_) => stats.nodes_created += 1,
                Err(M1ndError::DuplicateNode(_)) => {
                    stats.label_collisions += 1;
                }
                Err(_) => {}
            }
        }

        // Phase 2: Add all edges
        for edge in &descriptor.edges {
            if let (Some(src), Some(tgt)) = (
                graph.resolve_id(&edge.source),
                graph.resolve_id(&edge.target),
            ) {
                let causal_strength = match edge.relation.as_str() {
                    "contains" => 0.8,
                    "imports" | "depends_on" => 0.6,
                    "calls" | "routes_to" => 0.5,
                    "implements" => 0.7,
                    "references" => 0.3,
                    _ => 0.4,
                };
                let direction = if edge.relation == "contains" {
                    EdgeDirection::Bidirectional
                } else {
                    EdgeDirection::Forward
                };
                if graph
                    .add_edge(
                        src,
                        tgt,
                        &edge.relation,
                        FiniteF32::new(edge.weight),
                        direction,
                        false,
                        FiniteF32::new(causal_strength),
                    )
                    .is_ok()
                {
                    stats.edges_created += 1;
                }
            }
        }

        // Phase 3: Finalize (CSR + PageRank)
        if graph.num_nodes() > 0 {
            graph.finalize()?;
        }

        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((graph, stats))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IngestAdapter;
    use std::io::Write;

    /// Helper: write JSON content to a temp file and return its path.
    fn write_temp_json(content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("m1nd_json_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!(
            "test_{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_json_adapter_basic() {
        let json = r#"{
            "nodes": [
                { "id": "room::studio_a", "label": "Studio A", "type": "System", "tags": ["room", "main"] },
                { "id": "bus::master", "label": "Master Bus", "type": "Process", "tags": ["bus"] }
            ],
            "edges": [
                { "source": "room::studio_a", "target": "bus::master", "relation": "routes_to", "weight": 1.0 }
            ]
        }"#;
        let path = write_temp_json(json);
        let adapter = JsonIngestAdapter;

        assert_eq!(adapter.domain(), "json");

        let (graph, stats) = adapter.ingest(&path).unwrap();

        assert_eq!(stats.nodes_created, 2);
        assert_eq!(stats.edges_created, 1);
        assert_eq!(graph.num_nodes(), 2);

        // Verify nodes are resolvable
        assert!(graph.resolve_id("room::studio_a").is_some());
        assert!(graph.resolve_id("bus::master").is_some());

        // Verify node types
        let studio_id = graph.resolve_id("room::studio_a").unwrap();
        assert_eq!(
            graph.nodes.node_type[studio_id.as_usize()],
            NodeType::System
        );
        let bus_id = graph.resolve_id("bus::master").unwrap();
        assert_eq!(graph.nodes.node_type[bus_id.as_usize()], NodeType::Process);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_json_adapter_custom_types() {
        let json = r#"{
            "nodes": [
                { "id": "concept::signal_flow", "label": "Signal Flow", "type": "Concept", "tags": ["audio"] },
                { "id": "material::copper_wire", "label": "Copper Wire", "type": "Material", "tags": ["cable"] },
                { "id": "process::routing", "label": "Audio Routing", "type": "Process", "tags": [] },
                { "id": "product::console", "label": "Mixing Console", "type": "Product", "tags": ["hardware"] },
                { "id": "supplier::acme", "label": "ACME Corp", "type": "Supplier", "tags": [] },
                { "id": "reg::iec60065", "label": "IEC 60065", "type": "Regulatory", "tags": ["safety"] },
                { "id": "cost::bom", "label": "Bill of Materials", "type": "Cost", "tags": [] },
                { "id": "custom::widget", "label": "Widget", "type": "FooBar", "tags": [] }
            ],
            "edges": [
                { "source": "concept::signal_flow", "target": "process::routing", "relation": "describes" },
                { "source": "material::copper_wire", "target": "product::console", "relation": "used_in", "weight": 0.8 },
                { "source": "supplier::acme", "target": "material::copper_wire", "relation": "supplies" }
            ]
        }"#;
        let path = write_temp_json(json);
        let adapter = JsonIngestAdapter;
        let (graph, stats) = adapter.ingest(&path).unwrap();

        assert_eq!(stats.nodes_created, 8);
        assert_eq!(stats.edges_created, 3);

        // Verify domain-specific NodeType mappings
        let concept = graph.resolve_id("concept::signal_flow").unwrap();
        assert_eq!(graph.nodes.node_type[concept.as_usize()], NodeType::Concept);

        let material = graph.resolve_id("material::copper_wire").unwrap();
        assert_eq!(
            graph.nodes.node_type[material.as_usize()],
            NodeType::Material
        );

        let process = graph.resolve_id("process::routing").unwrap();
        assert_eq!(graph.nodes.node_type[process.as_usize()], NodeType::Process);

        let product = graph.resolve_id("product::console").unwrap();
        assert_eq!(graph.nodes.node_type[product.as_usize()], NodeType::Product);

        let supplier = graph.resolve_id("supplier::acme").unwrap();
        assert_eq!(
            graph.nodes.node_type[supplier.as_usize()],
            NodeType::Supplier
        );

        let regulatory = graph.resolve_id("reg::iec60065").unwrap();
        assert_eq!(
            graph.nodes.node_type[regulatory.as_usize()],
            NodeType::Regulatory
        );

        let cost = graph.resolve_id("cost::bom").unwrap();
        assert_eq!(graph.nodes.node_type[cost.as_usize()], NodeType::Cost);

        // Unknown type string -> Custom(0)
        let custom = graph.resolve_id("custom::widget").unwrap();
        assert_eq!(
            graph.nodes.node_type[custom.as_usize()],
            NodeType::Custom(0)
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_json_adapter_empty() {
        let json = r#"{ "nodes": [], "edges": [] }"#;
        let path = write_temp_json(json);
        let adapter = JsonIngestAdapter;
        let (graph, stats) = adapter.ingest(&path).unwrap();

        assert_eq!(stats.nodes_created, 0);
        assert_eq!(stats.edges_created, 0);
        assert_eq!(graph.num_nodes(), 0);

        std::fs::remove_file(&path).ok();
    }
}
