use m1nd_core::error::M1ndResult;
use m1nd_core::graph::{Graph, NodeProvenanceInput};
use m1nd_core::types::{EdgeDirection, EdgeIdx, NodeId};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct EdgeKey {
    source: String,
    target: String,
    relation: String,
    direction: u8,
    inhibitory: bool,
}

#[derive(Clone, Debug)]
struct EdgeRecord {
    key: EdgeKey,
    weight: f32,
    causal_strength: f32,
}

fn node_external_ids(graph: &Graph) -> Vec<String> {
    let mut ids = vec![String::new(); graph.num_nodes() as usize];
    for (interned, &node_id) in &graph.id_to_node {
        let idx = node_id.as_usize();
        if idx < ids.len() {
            ids[idx] = graph.strings.resolve(*interned).to_string();
        }
    }
    ids
}

fn canonical_edge_key(
    source: &str,
    target: &str,
    relation: &str,
    direction: EdgeDirection,
    inhibitory: bool,
) -> EdgeKey {
    if direction == EdgeDirection::Bidirectional && source > target {
        EdgeKey {
            source: target.to_string(),
            target: source.to_string(),
            relation: relation.to_string(),
            direction: 1,
            inhibitory,
        }
    } else {
        EdgeKey {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            direction: if direction == EdgeDirection::Bidirectional {
                1
            } else {
                0
            },
            inhibitory,
        }
    }
}

fn merge_tags(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut merged = Vec::with_capacity(existing.len() + incoming.len());
    let mut seen = HashSet::new();
    for tag in existing.iter().chain(incoming.iter()) {
        if seen.insert(tag.clone()) {
            merged.push(tag.clone());
        }
    }
    merged
}

fn collect_edges(graph: &Graph) -> Vec<EdgeRecord> {
    let node_ids = node_external_ids(graph);
    let mut out = Vec::new();

    for src in 0..graph.num_nodes() as usize {
        for edge_idx in graph.csr.out_range(NodeId::new(src as u32)) {
            let target = graph.csr.targets[edge_idx].as_usize();
            let direction = graph.csr.directions[edge_idx];
            if direction == EdgeDirection::Bidirectional && src > target {
                continue;
            }

            let relation = graph
                .strings
                .resolve(graph.csr.relations[edge_idx])
                .to_string();
            out.push(EdgeRecord {
                key: canonical_edge_key(
                    &node_ids[src],
                    &node_ids[target],
                    &relation,
                    direction,
                    graph.csr.inhibitory[edge_idx],
                ),
                weight: graph.csr.read_weight(EdgeIdx::new(edge_idx as u32)).get(),
                causal_strength: graph.csr.causal_strengths[edge_idx].get(),
            });
        }
    }

    out
}

pub fn merge_graphs(base: &Graph, overlay: &Graph) -> M1ndResult<Graph> {
    let base_ids = node_external_ids(base);
    let overlay_ids = node_external_ids(overlay);
    let mut merged = Graph::with_capacity(
        base.num_nodes() as usize + overlay.num_nodes() as usize,
        base.num_edges() + overlay.num_edges(),
    );

    for graph in [base, overlay] {
        let external_ids = if std::ptr::eq(graph, base) {
            &base_ids
        } else {
            &overlay_ids
        };

        #[allow(clippy::needless_range_loop)]
        for idx in 0..graph.num_nodes() as usize {
            let external_id = &external_ids[idx];
            let label = graph.strings.resolve(graph.nodes.label[idx]).to_string();
            let tags: Vec<String> = graph.nodes.tags[idx]
                .iter()
                .map(|&tag| graph.strings.resolve(tag).to_string())
                .collect();

            if let Some(existing) = merged.resolve_id(external_id) {
                let existing_idx = existing.as_usize();
                let current_tags: Vec<String> = merged.nodes.tags[existing_idx]
                    .iter()
                    .map(|&tag| merged.strings.resolve(tag).to_string())
                    .collect();
                let merged_tags = merge_tags(&current_tags, &tags);
                merged.nodes.tags[existing_idx] = merged_tags
                    .iter()
                    .map(|tag| merged.strings.get_or_intern(tag))
                    .collect();
                merged.nodes.last_modified[existing_idx] =
                    merged.nodes.last_modified[existing_idx].max(graph.nodes.last_modified[idx]);
                let provenance = graph.resolve_node_provenance(NodeId::new(idx as u32));
                merged.merge_node_provenance(
                    existing,
                    NodeProvenanceInput {
                        source_path: provenance.source_path.as_deref(),
                        line_start: provenance.line_start,
                        line_end: provenance.line_end,
                        excerpt: provenance.excerpt.as_deref(),
                        namespace: provenance.namespace.as_deref(),
                        canonical: provenance.canonical,
                    },
                );
                continue;
            }

            let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
            let node_id = merged.add_node(
                external_id,
                &label,
                graph.nodes.node_type[idx],
                &tag_refs,
                graph.nodes.last_modified[idx],
                graph.nodes.change_frequency[idx].get(),
            )?;
            let provenance = graph.resolve_node_provenance(NodeId::new(idx as u32));
            merged.set_node_provenance(
                node_id,
                NodeProvenanceInput {
                    source_path: provenance.source_path.as_deref(),
                    line_start: provenance.line_start,
                    line_end: provenance.line_end,
                    excerpt: provenance.excerpt.as_deref(),
                    namespace: provenance.namespace.as_deref(),
                    canonical: provenance.canonical,
                },
            );
        }
    }

    let mut edge_records: HashMap<EdgeKey, EdgeRecord> = HashMap::new();
    for record in collect_edges(base)
        .into_iter()
        .chain(collect_edges(overlay))
    {
        edge_records
            .entry(record.key.clone())
            .and_modify(|existing| {
                existing.weight = existing.weight.max(record.weight);
                existing.causal_strength = existing.causal_strength.max(record.causal_strength);
            })
            .or_insert(record);
    }

    for record in edge_records.values() {
        let source = merged.resolve_id(&record.key.source).unwrap();
        let target = merged.resolve_id(&record.key.target).unwrap();
        merged.add_edge(
            source,
            target,
            &record.key.relation,
            m1nd_core::types::FiniteF32::new(record.weight),
            if record.key.direction == 1 {
                EdgeDirection::Bidirectional
            } else {
                EdgeDirection::Forward
            },
            record.key.inhibitory,
            m1nd_core::types::FiniteF32::new(record.causal_strength),
        )?;
    }

    if merged.num_nodes() > 0 {
        merged.finalize()?;
    }

    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use m1nd_core::types::{FiniteF32, NodeType};

    #[test]
    fn merge_graphs_preserves_base_and_overlay_nodes() {
        let mut base = Graph::with_capacity(2, 1);
        let a = base
            .add_node(
                "file::alpha.rs",
                "alpha.rs",
                NodeType::File,
                &["code"],
                10.0,
                0.3,
            )
            .unwrap();
        let b = base
            .add_node(
                "file::beta.rs",
                "beta.rs",
                NodeType::File,
                &["code"],
                20.0,
                0.3,
            )
            .unwrap();
        base.add_edge(
            a,
            b,
            "references",
            FiniteF32::new(0.7),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(0.3),
        )
        .unwrap();
        base.finalize().unwrap();

        let mut overlay = Graph::with_capacity(2, 1);
        let note = overlay
            .add_node(
                "memory::memory::file::daily-note",
                "daily-note",
                NodeType::File,
                &["memory"],
                30.0,
                0.6,
            )
            .unwrap();
        let state = overlay
            .add_node(
                "memory::memory::entry::batman-mode",
                "Batman mode",
                NodeType::Concept,
                &["memory", "memory:state"],
                30.0,
                0.5,
            )
            .unwrap();
        overlay
            .add_edge(
                note,
                state,
                "contains",
                FiniteF32::ONE,
                EdgeDirection::Bidirectional,
                false,
                FiniteF32::new(0.8),
            )
            .unwrap();
        overlay.finalize().unwrap();

        let merged = merge_graphs(&base, &overlay).unwrap();

        assert!(merged.resolve_id("file::alpha.rs").is_some());
        assert!(merged
            .resolve_id("memory::memory::entry::batman-mode")
            .is_some());
        assert!(merged.num_edges() >= 2);
    }
}
