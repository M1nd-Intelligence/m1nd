// === crates/m1nd-core/src/snapshot.rs ===

use std::path::Path;
use std::io::{Write, BufWriter};
use crate::error::{M1ndError, M1ndResult};
use crate::graph::{Graph, NodeProvenanceInput, ResolvedNodeProvenance};
use crate::plasticity::SynapticState;
use crate::types::*;

// ---------------------------------------------------------------------------
// Snapshot — JSON graph persistence
// FM-PL-008 fix: atomic write (write to temp, rename).
// ---------------------------------------------------------------------------

/// Graph snapshot format version.
pub const SNAPSHOT_VERSION: u32 = 3;

// ---------------------------------------------------------------------------
// Serialization types
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct GraphSnapshot {
    version: u32,
    nodes: Vec<NodeSnapshot>,
    edges: Vec<EdgeSnapshot>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeSnapshot {
    external_id: String,
    label: String,
    node_type: u8,
    tags: Vec<String>,
    last_modified: f64,
    change_frequency: f32,
    #[serde(default, skip_serializing_if = "node_provenance_snapshot_is_empty")]
    provenance: NodeProvenanceSnapshot,
}

#[derive(Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct NodeProvenanceSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    excerpt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(default)]
    canonical: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EdgeSnapshot {
    source_id: String,
    target_id: String,
    relation: String,
    weight: f32,
    direction: u8, // 0=Forward, 1=Bidirectional
    inhibitory: bool,
    causal_strength: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CoChangeMetadata {
    version: u32,
    num_entries: u64,
}

// ---------------------------------------------------------------------------
// NodeType to/from u8 helpers
// ---------------------------------------------------------------------------

fn node_type_to_u8(nt: NodeType) -> u8 {
    match nt {
        NodeType::File => 0,
        NodeType::Directory => 1,
        NodeType::Function => 2,
        NodeType::Class => 3,
        NodeType::Struct => 4,
        NodeType::Enum => 5,
        NodeType::Type => 6,
        NodeType::Module => 7,
        NodeType::Reference => 8,
        NodeType::Concept => 9,
        NodeType::Material => 10,
        NodeType::Process => 11,
        NodeType::Product => 12,
        NodeType::Supplier => 13,
        NodeType::Regulatory => 14,
        NodeType::System => 15,
        NodeType::Cost => 16,
        NodeType::Custom(v) => 100 + v,
    }
}

fn u8_to_node_type(v: u8) -> NodeType {
    match v {
        0 => NodeType::File,
        1 => NodeType::Directory,
        2 => NodeType::Function,
        3 => NodeType::Class,
        4 => NodeType::Struct,
        5 => NodeType::Enum,
        6 => NodeType::Type,
        7 => NodeType::Module,
        8 => NodeType::Reference,
        9 => NodeType::Concept,
        10 => NodeType::Material,
        11 => NodeType::Process,
        12 => NodeType::Product,
        13 => NodeType::Supplier,
        14 => NodeType::Regulatory,
        15 => NodeType::System,
        16 => NodeType::Cost,
        v if v >= 100 => NodeType::Custom(v - 100),
        _ => NodeType::Custom(v),
    }
}

fn node_provenance_snapshot_is_empty(value: &NodeProvenanceSnapshot) -> bool {
    value == &NodeProvenanceSnapshot::default()
}

fn snapshot_from_provenance(value: ResolvedNodeProvenance) -> NodeProvenanceSnapshot {
    NodeProvenanceSnapshot {
        source_path: value.source_path,
        line_start: value.line_start,
        line_end: value.line_end,
        excerpt: value.excerpt,
        namespace: value.namespace,
        canonical: value.canonical,
    }
}

// ---------------------------------------------------------------------------
// Graph save/load
// ---------------------------------------------------------------------------

/// Save full graph to JSON snapshot. Atomic write: temp file + rename (FM-PL-008).
/// Serializes all nodes and edges so the graph can be fully reconstructed on load.
pub fn save_graph(graph: &Graph, path: &Path) -> M1ndResult<()> {
    let n = graph.num_nodes() as usize;

    // Build reverse map: NodeId -> external_id string
    let mut node_to_ext_id = vec![String::new(); n];
    for (&interned, &node_id) in &graph.id_to_node {
        node_to_ext_id[node_id.as_usize()] = graph.strings.resolve(interned).to_string();
    }

    // Serialize nodes
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let label = graph.strings.resolve(graph.nodes.label[i]).to_string();
        let tags: Vec<String> = graph.nodes.tags[i]
            .iter()
            .map(|&t| graph.strings.resolve(t).to_string())
            .collect();
        nodes.push(NodeSnapshot {
            external_id: node_to_ext_id[i].clone(),
            label,
            node_type: node_type_to_u8(graph.nodes.node_type[i]),
            tags,
            last_modified: graph.nodes.last_modified[i],
            change_frequency: graph.nodes.change_frequency[i].get(),
            provenance: snapshot_from_provenance(
                graph.resolve_node_provenance(NodeId::new(i as u32)),
            ),
        });
    }

    // Serialize edges from CSR (deduplicate bidirectional: only save source < target)
    let mut edges = Vec::new();
    for src in 0..n {
        let range = graph.csr.out_range(NodeId::new(src as u32));
        for j in range {
            let tgt = graph.csr.targets[j].as_usize();
            let dir = graph.csr.directions[j];
            // For bidirectional edges, only save the canonical direction (source < target)
            if dir == EdgeDirection::Bidirectional && src > tgt {
                continue;
            }
            let relation = graph.strings.resolve(graph.csr.relations[j]).to_string();
            let weight = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
            edges.push(EdgeSnapshot {
                source_id: node_to_ext_id[src].clone(),
                target_id: node_to_ext_id[tgt].clone(),
                relation,
                weight,
                direction: if dir == EdgeDirection::Bidirectional { 1 } else { 0 },
                inhibitory: graph.csr.inhibitory[j],
                causal_strength: graph.csr.causal_strengths[j].get(),
            });
        }
    }

    let snapshot = GraphSnapshot {
        version: SNAPSHOT_VERSION,
        nodes,
        edges,
    };

    let json = serde_json::to_string(&snapshot).map_err(M1ndError::Serde)?;

    // FM-PL-008: atomic write via temp file + rename
    let temp_path = path.with_extension("tmp");
    {
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
    }
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load full graph from JSON snapshot. Reconstructs the complete graph
/// with all nodes, edges, CSR, and PageRank.
pub fn load_graph(path: &Path) -> M1ndResult<Graph> {
    let data = std::fs::read_to_string(path)?;
    let snapshot: GraphSnapshot = serde_json::from_str(&data)
        .map_err(M1ndError::Serde)?;

    if snapshot.nodes.is_empty() {
        return Ok(Graph::new());
    }

    let mut graph = Graph::with_capacity(snapshot.nodes.len(), snapshot.edges.len());

    // Add all nodes
    for node in &snapshot.nodes {
        let tags: Vec<&str> = node.tags.iter().map(|s| s.as_str()).collect();
        if let Ok(node_id) = graph.add_node(
            &node.external_id,
            &node.label,
            u8_to_node_type(node.node_type),
            &tags,
            node.last_modified,
            node.change_frequency,
        ) {
            graph.set_node_provenance(
                node_id,
                NodeProvenanceInput {
                    source_path: node.provenance.source_path.as_deref(),
                    line_start: node.provenance.line_start,
                    line_end: node.provenance.line_end,
                    excerpt: node.provenance.excerpt.as_deref(),
                    namespace: node.provenance.namespace.as_deref(),
                    canonical: node.provenance.canonical,
                },
            );
        }
    }

    // Add all edges
    for edge in &snapshot.edges {
        if let (Some(src), Some(tgt)) = (
            graph.resolve_id(&edge.source_id),
            graph.resolve_id(&edge.target_id),
        ) {
            let direction = if edge.direction == 1 {
                EdgeDirection::Bidirectional
            } else {
                EdgeDirection::Forward
            };
            let _ = graph.add_edge(
                src,
                tgt,
                &edge.relation,
                FiniteF32::new(edge.weight),
                direction,
                edge.inhibitory,
                FiniteF32::new(edge.causal_strength),
            );
        }
    }

    // Finalize: build CSR + PageRank
    if graph.num_nodes() > 0 {
        graph.finalize()?;
    }

    Ok(graph)
}

// ---------------------------------------------------------------------------
// Plasticity state save/load
// ---------------------------------------------------------------------------

/// Save plasticity state to JSON. Atomic write (FM-PL-008).
/// FM-PL-001 NaN firewall: non-finite weights replaced with originals at export.
pub fn save_plasticity_state(
    states: &[SynapticState],
    path: &Path,
) -> M1ndResult<()> {
    // FM-PL-001: NaN firewall at export boundary
    let safe_states: Vec<SynapticState> = states.iter().map(|s| {
        let mut safe = s.clone();
        if !safe.current_weight.is_finite() {
            safe.current_weight = safe.original_weight;
        }
        safe
    }).collect();

    let json = serde_json::to_string_pretty(&safe_states)
        .map_err(M1ndError::Serde)?;

    // FM-PL-008: atomic write
    let temp_path = path.with_extension("tmp");
    {
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
    }
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load plasticity state from JSON.
/// FM-PL-007 fix: schema validation + error recovery.
pub fn load_plasticity_state(path: &Path) -> M1ndResult<Vec<SynapticState>> {
    let data = std::fs::read_to_string(path)?;
    let states: Vec<SynapticState> = serde_json::from_str(&data)
        .map_err(M1ndError::Serde)?;

    // FM-PL-007: validate each entry
    for state in &states {
        if !state.original_weight.is_finite() || !state.current_weight.is_finite() {
            return Err(M1ndError::CorruptState {
                reason: format!(
                    "Non-finite weight in state: {}->{}",
                    state.source_label, state.target_label
                ),
            });
        }
    }

    Ok(states)
}

// ---------------------------------------------------------------------------
// Co-change matrix save/load
// ---------------------------------------------------------------------------

/// Save co-change matrix metadata.
pub fn save_co_change_matrix(
    _matrix: &crate::temporal::CoChangeMatrix,
    path: &Path,
) -> M1ndResult<()> {
    let meta = CoChangeMetadata {
        version: 1,
        num_entries: _matrix.num_entries(),
    };
    let json = serde_json::to_string_pretty(&meta)
        .map_err(M1ndError::Serde)?;

    let temp_path = path.with_extension("tmp");
    {
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
        writer.flush()?;
    }
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load co-change matrix.
pub fn load_co_change_matrix(path: &Path) -> M1ndResult<crate::temporal::CoChangeMatrix> {
    let data = std::fs::read_to_string(path)?;
    let _meta: CoChangeMetadata = serde_json::from_str(&data)
        .map_err(M1ndError::Serde)?;

    // Return empty matrix; full deserialization needs graph context
    let graph = Graph::new();
    crate::temporal::CoChangeMatrix::bootstrap(&graph, 500_000)
}
