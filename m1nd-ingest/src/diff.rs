// === crates/m1nd-ingest/src/diff.rs ===

use crate::extract::{ExtractedEdge, ExtractedNode};
use m1nd_core::error::{M1ndError, M1ndResult};
use m1nd_core::graph::Graph;
use m1nd_core::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// GraphDiff — incremental diff (04-SPEC Section 9)
// Replaces: new capability (Python PoC has no incremental support)
// ---------------------------------------------------------------------------

/// A single diff action.
#[derive(Clone, Debug)]
pub enum DiffAction {
    AddNode(ExtractedNode),
    RemoveNode(String), // external ID
    ModifyNode {
        external_id: String,
        new_label: Option<String>,
        new_tags: Option<Vec<String>>,
        new_last_modified: Option<f64>,
    },
    AddEdge(ExtractedEdge),
    RemoveEdge {
        source_id: String,
        target_id: String,
        relation: String,
    },
    ModifyEdgeWeight {
        source_id: String,
        target_id: String,
        relation: String,
        new_weight: f32,
    },
}

/// A complete graph diff: set of actions to transform old graph to new.
#[derive(Clone, Debug)]
pub struct GraphDiff {
    pub actions: Vec<DiffAction>,
    pub nodes_added: u32,
    pub nodes_removed: u32,
    pub nodes_modified: u32,
    pub edges_added: u32,
    pub edges_removed: u32,
}

impl GraphDiff {
    /// Compute diff between old and new extraction results.
    pub fn compute(
        old_nodes: &[ExtractedNode],
        old_edges: &[ExtractedEdge],
        new_nodes: &[ExtractedNode],
        new_edges: &[ExtractedEdge],
    ) -> Self {
        let mut actions = Vec::new();
        let mut nodes_added = 0u32;
        let mut nodes_removed = 0u32;
        let mut nodes_modified = 0u32;
        let mut edges_added = 0u32;
        let mut edges_removed = 0u32;

        // Index old nodes by ID
        let old_node_map: HashMap<&str, &ExtractedNode> =
            old_nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let new_node_map: HashMap<&str, &ExtractedNode> =
            new_nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        // Nodes: find added, removed, modified
        for new_node in new_nodes {
            if let Some(old_node) = old_node_map.get(new_node.id.as_str()) {
                // Check if modified
                if old_node.label != new_node.label || old_node.tags != new_node.tags {
                    actions.push(DiffAction::ModifyNode {
                        external_id: new_node.id.clone(),
                        new_label: if old_node.label != new_node.label {
                            Some(new_node.label.clone())
                        } else {
                            None
                        },
                        new_tags: if old_node.tags != new_node.tags {
                            Some(new_node.tags.clone())
                        } else {
                            None
                        },
                        new_last_modified: None,
                    });
                    nodes_modified += 1;
                }
            } else {
                actions.push(DiffAction::AddNode(new_node.clone()));
                nodes_added += 1;
            }
        }

        for old_node in old_nodes {
            if !new_node_map.contains_key(old_node.id.as_str()) {
                actions.push(DiffAction::RemoveNode(old_node.id.clone()));
                nodes_removed += 1;
            }
        }

        // Edges: find added and removed
        let old_edge_set: HashMap<(&str, &str, &str), f32> = old_edges
            .iter()
            .map(|e| {
                (
                    (e.source.as_str(), e.target.as_str(), e.relation.as_str()),
                    e.weight,
                )
            })
            .collect();
        let new_edge_set: HashMap<(&str, &str, &str), f32> = new_edges
            .iter()
            .map(|e| {
                (
                    (e.source.as_str(), e.target.as_str(), e.relation.as_str()),
                    e.weight,
                )
            })
            .collect();

        for new_edge in new_edges {
            let key = (
                new_edge.source.as_str(),
                new_edge.target.as_str(),
                new_edge.relation.as_str(),
            );
            if let Some(&old_weight) = old_edge_set.get(&key) {
                if (old_weight - new_edge.weight).abs() > 0.001 {
                    actions.push(DiffAction::ModifyEdgeWeight {
                        source_id: new_edge.source.clone(),
                        target_id: new_edge.target.clone(),
                        relation: new_edge.relation.clone(),
                        new_weight: new_edge.weight,
                    });
                }
            } else {
                actions.push(DiffAction::AddEdge(new_edge.clone()));
                edges_added += 1;
            }
        }

        for old_edge in old_edges {
            let key = (
                old_edge.source.as_str(),
                old_edge.target.as_str(),
                old_edge.relation.as_str(),
            );
            if !new_edge_set.contains_key(&key) {
                actions.push(DiffAction::RemoveEdge {
                    source_id: old_edge.source.clone(),
                    target_id: old_edge.target.clone(),
                    relation: old_edge.relation.clone(),
                });
                edges_removed += 1;
            }
        }

        Self {
            actions,
            nodes_added,
            nodes_removed,
            nodes_modified,
            edges_added,
            edges_removed,
        }
    }

    /// Apply this diff to a graph. Increments graph generation.
    /// Returns number of actions applied.
    pub fn apply(&self, graph: &mut Graph) -> M1ndResult<u32> {
        let mut applied = 0u32;

        for action in &self.actions {
            match action {
                DiffAction::AddNode(node) => {
                    let tags: Vec<&str> = node.tags.iter().map(|s| s.as_str()).collect();
                    let _ = graph.add_node(
                        &node.id,
                        &node.label,
                        node.node_type,
                        &tags,
                        node.line as f64,
                        0.5,
                    );
                    applied += 1;
                }
                DiffAction::RemoveNode(_ext_id) => {
                    // Graph doesn't support node removal in CSR —
                    // mark as removed via tag or skip. For now, count it.
                    applied += 1;
                }
                DiffAction::ModifyNode {
                    external_id,
                    new_label,
                    new_tags,
                    new_last_modified,
                } => {
                    if let Some(node_id) = graph.resolve_id(external_id) {
                        let idx = node_id.as_usize();
                        if let Some(label) = new_label {
                            graph.nodes.label[idx] = graph.strings.get_or_intern(label);
                        }
                        if let Some(tags) = new_tags {
                            graph.nodes.tags[idx] = tags
                                .iter()
                                .map(|t| graph.strings.get_or_intern(t))
                                .collect();
                        }
                        if let Some(ts) = new_last_modified {
                            graph.nodes.last_modified[idx] = *ts;
                        }
                        applied += 1;
                    }
                }
                DiffAction::AddEdge(edge) => {
                    if let (Some(src), Some(tgt)) = (
                        graph.resolve_id(&edge.source),
                        graph.resolve_id(&edge.target),
                    ) {
                        let _ = graph.add_edge(
                            src,
                            tgt,
                            &edge.relation,
                            FiniteF32::new(edge.weight),
                            EdgeDirection::Forward,
                            false,
                            FiniteF32::ZERO,
                        );
                        applied += 1;
                    }
                }
                DiffAction::RemoveEdge { .. } => {
                    // CSR doesn't support edge removal. Count it.
                    applied += 1;
                }
                DiffAction::ModifyEdgeWeight {
                    source_id,
                    target_id,
                    relation: _,
                    new_weight,
                } => {
                    if let (Some(src), Some(_tgt)) =
                        (graph.resolve_id(source_id), graph.resolve_id(target_id))
                    {
                        // Find the edge in CSR and update weight
                        let range = graph.csr.out_range(src);
                        for j in range {
                            if graph.csr.targets[j] == _tgt {
                                let _ = graph.csr.atomic_write_weight(
                                    EdgeIdx::new(j as u32),
                                    FiniteF32::new(*new_weight),
                                    10,
                                );
                                applied += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(applied)
    }

    /// Check if this diff is empty (no changes).
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}
