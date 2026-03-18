// === crates/m1nd-core/src/snapshot_bin.rs ===
// Compact binary snapshot (bincode) with atomic write. Mirrors snapshot.rs JSON logic.

use crate::error::{M1ndError, M1ndResult};
use crate::graph::{Graph, NodeProvenanceInput, ResolvedNodeProvenance};
use crate::types::*;
use crate::snapshot::SNAPSHOT_VERSION;
use std::io::{BufWriter, Write};
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize)]
struct GraphSnapshotBin {
    version: u32,
    nodes: Vec<NodeSnapshotBin>,
    edges: Vec<EdgeSnapshotBin>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct NodeSnapshotBin {
    external_id: String,
    label: String,
    node_type: u8,
    tags: Vec<String>,
    last_modified: f64,
    change_frequency: f32,
    provenance: NodeProvenanceBin,
}

#[derive(Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct NodeProvenanceBin {
    source_path: Option<String>,
    line_start: Option<u32>,
    line_end: Option<u32>,
    excerpt: Option<String>,
    namespace: Option<String>,
    canonical: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EdgeSnapshotBin {
    source_id: String,
    target_id: String,
    relation: String,
    weight: f32,
    direction: u8, // 0=Forward, 1=Bidirectional
    inhibitory: bool,
    causal_strength: f32,
}

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

fn provenance_from_resolved(p: ResolvedNodeProvenance) -> NodeProvenanceBin {
    NodeProvenanceBin {
        source_path: p.source_path,
        line_start: p.line_start,
        line_end: p.line_end,
        excerpt: p.excerpt,
        namespace: p.namespace,
        canonical: p.canonical,
    }
}

/// Save full graph to compact binary snapshot. Atomic write via temp+rename.
pub fn save_graph(graph: &Graph, path: &Path) -> M1ndResult<()> {
    let n = graph.num_nodes() as usize;

    // Build reverse map: NodeId -> external_id string
    let mut node_to_ext_id = vec![String::new(); n];
    for (&interned, &node_id) in &graph.id_to_node {
        node_to_ext_id[node_id.as_usize()] = graph.strings.resolve(interned).to_string();
    }

    // Nodes
    let mut nodes = Vec::with_capacity(n);
    for i in 0..n {
        let label = graph.strings.resolve(graph.nodes.label[i]).to_string();
        let tags: Vec<String> = graph.nodes.tags[i]
            .iter()
            .map(|&t| graph.strings.resolve(t).to_string())
            .collect();
        nodes.push(NodeSnapshotBin {
            external_id: node_to_ext_id[i].clone(),
            label,
            node_type: node_type_to_u8(graph.nodes.node_type[i]),
            tags,
            last_modified: graph.nodes.last_modified[i],
            change_frequency: graph.nodes.change_frequency[i].get(),
            provenance: provenance_from_resolved(
                graph.resolve_node_provenance(NodeId::new(i as u32)),
            ),
        });
    }

    // Edges (CSR)
    let mut edges = Vec::new();
    for src in 0..n {
        let range = graph.csr.out_range(NodeId::new(src as u32));
        for j in range {
            let tgt = graph.csr.targets[j].as_usize();
            let dir = graph.csr.directions[j];
            if dir == EdgeDirection::Bidirectional && src > tgt {
                continue;
            }
            let relation = graph.strings.resolve(graph.csr.relations[j]).to_string();
            let weight = graph.csr.read_weight(EdgeIdx::new(j as u32)).get();
            edges.push(EdgeSnapshotBin {
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

    let snapshot = GraphSnapshotBin {
        version: SNAPSHOT_VERSION,
        nodes,
        edges,
    };

    let bytes = bincode::serialize(&snapshot).map_err(|e| M1ndError::PersistenceFailed(e.to_string()))?;

    // Atomic write
    let temp_path = path.with_extension("tmp");
    {
        let file = std::fs::File::create(&temp_path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&bytes)?;
        writer.flush()?;
    }
    std::fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load full graph from compact binary snapshot.
pub fn load_graph(path: &Path) -> M1ndResult<Graph> {
    let data = std::fs::read(path)?;
    let snapshot: GraphSnapshotBin = bincode::deserialize(&data)
        .map_err(|e| M1ndError::PersistenceFailed(e.to_string()))?;

    if snapshot.nodes.is_empty() {
        return Ok(Graph::new());
    }

    let mut graph = Graph::with_capacity(snapshot.nodes.len(), snapshot.edges.len());

    // Nodes
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

    // Edges
    for edge in &snapshot.edges {
        if let (Some(src), Some(tgt)) = (graph.resolve_id(&edge.source_id), graph.resolve_id(&edge.target_id)) {
            let direction = if edge.direction == 1 { EdgeDirection::Bidirectional } else { EdgeDirection::Forward };
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

    if graph.num_nodes() > 0 {
        graph.finalize()?;
    }

    Ok(graph)
}
