use crate::{IngestAdapter, IngestStats};
use m1nd_core::error::M1ndResult;
use m1nd_core::graph::Graph;
use m1nd_core::graph::NodeProvenanceInput;
use m1nd_core::types::{EdgeDirection, FiniteF32, NodeType};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
struct MemoryNodeRecord {
    id: String,
    label: String,
    node_type: NodeType,
    tags: Vec<String>,
    last_modified: f64,
    change_frequency: f32,
    source_path: String,
    line_start: Option<u32>,
    line_end: Option<u32>,
    excerpt: Option<String>,
    namespace: String,
    canonical: bool,
}

#[derive(Clone, Debug)]
struct MemoryEdgeRecord {
    source: String,
    target: String,
    relation: String,
    weight: f32,
    direction: EdgeDirection,
    inhibitory: bool,
    causal_strength: f32,
}

pub struct MemoryIngestAdapter {
    namespace: String,
}

impl MemoryIngestAdapter {
    pub fn new(namespace: Option<String>) -> Self {
        let namespace = namespace
            .unwrap_or_else(|| "memory".to_string())
            .trim()
            .to_lowercase();
        let namespace = if namespace.is_empty() {
            "memory".to_string()
        } else {
            namespace
        };
        Self { namespace }
    }

    fn accepted_extension(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()),
            Some(ext) if matches!(ext.as_str(), "md" | "markdown" | "txt")
        )
    }

    fn collect_files(&self, root: &Path) -> Vec<PathBuf> {
        if root.is_file() {
            return if Self::accepted_extension(root) {
                vec![root.to_path_buf()]
            } else {
                vec![]
            };
        }

        WalkDir::new(root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() && Self::accepted_extension(entry.path()))
            .map(|entry| entry.into_path())
            .collect()
    }

    fn slugify(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        let mut prev_dash = false;

        for ch in raw.chars() {
            let lowered = ch.to_ascii_lowercase();
            if lowered.is_ascii_alphanumeric() {
                out.push(lowered);
                prev_dash = false;
            } else if !prev_dash {
                out.push('-');
                prev_dash = true;
            }
        }

        let trimmed = out.trim_matches('-');
        if trimmed.is_empty() {
            "entry".to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn file_timestamp(path: &Path) -> f64 {
        std::fs::metadata(path)
            .ok()
            .and_then(|meta| meta.modified().ok())
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs_f64())
            .unwrap_or_else(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|duration| duration.as_secs_f64())
                    .unwrap_or(0.0)
            })
    }

    fn excerpt(text: &str) -> Option<String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        let excerpt: String = trimmed.chars().take(220).collect();
        Some(excerpt)
    }

    fn document_kind(rel_path: &str) -> &'static str {
        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(rel_path);
        if Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$")
            .unwrap()
            .is_match(file_name)
        {
            "daily-note"
        } else if file_name.eq_ignore_ascii_case("memory.md") {
            "long-term-memory"
        } else {
            "memory-note"
        }
    }

    fn is_canonical_source(rel_path: &str) -> bool {
        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(rel_path)
            .to_ascii_lowercase();
        Regex::new(r"^\d{4}-\d{2}-\d{2}\.md$")
            .unwrap()
            .is_match(&file_name)
            || file_name == "memory.md"
            || file_name.ends_with("-active.md")
            || file_name.ends_with("-history.md")
            || file_name.contains("briefing")
    }

    fn push_node(
        nodes: &mut Vec<MemoryNodeRecord>,
        seen: &mut HashSet<String>,
        record: MemoryNodeRecord,
    ) {
        if seen.insert(record.id.clone()) {
            nodes.push(record);
        }
    }

    fn push_edge(
        edges: &mut Vec<MemoryEdgeRecord>,
        seen: &mut HashSet<(String, String, String, u8)>,
        record: MemoryEdgeRecord,
    ) {
        let key = match record.direction {
            EdgeDirection::Bidirectional => {
                if record.source <= record.target {
                    (
                        record.source.clone(),
                        record.target.clone(),
                        record.relation.clone(),
                        1,
                    )
                } else {
                    (
                        record.target.clone(),
                        record.source.clone(),
                        record.relation.clone(),
                        1,
                    )
                }
            }
            EdgeDirection::Forward => (
                record.source.clone(),
                record.target.clone(),
                record.relation.clone(),
                0,
            ),
        };

        if seen.insert(key) {
            edges.push(record);
        }
    }

    fn classify_entry(text: &str) -> (NodeType, Vec<String>, String) {
        let lower = text.to_ascii_lowercase();
        if text.contains("[ ]")
            || text.contains("[x]")
            || lower.contains("todo")
            || lower.contains("task")
        {
            (
                NodeType::Process,
                vec!["memory:item".into(), "memory:task".into()],
                "tracks".into(),
            )
        } else if lower.contains("decision")
            || lower.contains("decided")
            || lower.contains("resolved")
        {
            (
                NodeType::Concept,
                vec!["memory:item".into(), "memory:decision".into()],
                "decided".into(),
            )
        } else if lower.contains("mode") || lower.contains("state") || lower.contains("mood") {
            (
                NodeType::Concept,
                vec!["memory:item".into(), "memory:state".into()],
                "relates_to".into(),
            )
        } else if lower.contains("meeting") || lower.contains("session") || lower.contains("today")
        {
            (
                NodeType::Process,
                vec!["memory:item".into(), "memory:event".into()],
                "happened_on".into(),
            )
        } else {
            (
                NodeType::Concept,
                vec!["memory:item".into(), "memory:note".into()],
                "contains".into(),
            )
        }
    }

    fn parse_memory_file(
        &self,
        root: &Path,
        path: &Path,
        nodes: &mut Vec<MemoryNodeRecord>,
        edges: &mut Vec<MemoryEdgeRecord>,
        node_seen: &mut HashSet<String>,
        edge_seen: &mut HashSet<(String, String, String, u8)>,
    ) -> M1ndResult<()> {
        let text = std::fs::read_to_string(path)?;
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let file_slug = Self::slugify(&rel_path);
        let document_id = format!("memory::{}::file::{}", self.namespace, file_slug);
        let file_timestamp = Self::file_timestamp(path);
        let doc_kind = Self::document_kind(&rel_path);
        let canonical = Self::is_canonical_source(&rel_path);
        let file_label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&rel_path)
            .to_string();

        Self::push_node(
            nodes,
            node_seen,
            MemoryNodeRecord {
                id: document_id.clone(),
                label: file_label.clone(),
                node_type: NodeType::File,
                tags: vec![
                    "memory".into(),
                    format!("namespace:{}", self.namespace),
                    format!("kind:{}", doc_kind),
                ],
                last_modified: file_timestamp,
                change_frequency: 0.6,
                source_path: rel_path.clone(),
                line_start: None,
                line_end: None,
                excerpt: Some(file_label.clone()),
                namespace: self.namespace.clone(),
                canonical,
            },
        );

        let heading_re = Regex::new(r"^(#{1,6})\s+(.+?)\s*$").unwrap();
        let bullet_re = Regex::new(r"^\s*[-*+]\s+(.+?)\s*$").unwrap();
        let checkbox_re = Regex::new(r"^\s*[-*+]\s+\[[ xX]\]\s+(.+?)\s*$").unwrap();
        let table_sep_re = Regex::new(r"^\s*\|?[\s:-]+\|[\s|:-]*$").unwrap();
        let file_ref_re =
            Regex::new(r"(?P<path>[A-Za-z0-9_./-]+\.(?:rs|py|ts|tsx|js|jsx|json|md|toml))")
                .unwrap();

        let mut current_parent = document_id.clone();
        let mut section_counts: HashMap<String, usize> = HashMap::new();
        let mut in_code_block = false;

        for (idx, line) in text.lines().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }
            if in_code_block || trimmed.is_empty() || trimmed == "---" {
                continue;
            }

            if let Some(caps) = heading_re.captures(trimmed) {
                let heading = caps.get(2).unwrap().as_str().trim();
                let heading_slug = Self::slugify(heading);
                let occurrence = section_counts.entry(heading_slug.clone()).or_insert(0usize);
                *occurrence += 1;
                let section_id = format!(
                    "memory::{}::section::{}::{}-{}",
                    self.namespace, file_slug, heading_slug, occurrence
                );

                Self::push_node(
                    nodes,
                    node_seen,
                    MemoryNodeRecord {
                        id: section_id.clone(),
                        label: heading.to_string(),
                        node_type: NodeType::Module,
                        tags: vec![
                            "memory".into(),
                            "memory:section".into(),
                            format!("namespace:{}", self.namespace),
                        ],
                        last_modified: file_timestamp,
                        change_frequency: 0.45,
                        source_path: rel_path.clone(),
                        line_start: Some(line_no as u32),
                        line_end: Some(line_no as u32),
                        excerpt: Self::excerpt(heading),
                        namespace: self.namespace.clone(),
                        canonical,
                    },
                );
                Self::push_edge(
                    edges,
                    edge_seen,
                    MemoryEdgeRecord {
                        source: document_id.clone(),
                        target: section_id.clone(),
                        relation: "contains".into(),
                        weight: 1.0,
                        direction: EdgeDirection::Bidirectional,
                        inhibitory: false,
                        causal_strength: 0.8,
                    },
                );
                current_parent = section_id;
                continue;
            }

            if table_sep_re.is_match(trimmed) {
                continue;
            }

            let entry_text = if let Some(caps) = checkbox_re.captures(trimmed) {
                caps.get(1).unwrap().as_str().trim().to_string()
            } else if let Some(caps) = bullet_re.captures(trimmed) {
                caps.get(1).unwrap().as_str().trim().to_string()
            } else if trimmed.contains('|') && trimmed.matches('|').count() >= 2 {
                trimmed
                    .split('|')
                    .map(str::trim)
                    .filter(|cell| !cell.is_empty())
                    .collect::<Vec<_>>()
                    .join(" | ")
            } else if trimmed.len() >= 8 && trimmed.len() <= 240 {
                trimmed.to_string()
            } else {
                continue;
            };

            if entry_text.is_empty() {
                continue;
            }

            let (node_type, mut tags, relation) = Self::classify_entry(&entry_text);
            tags.push("memory".into());
            tags.push(format!("namespace:{}", self.namespace));

            let entry_id = format!(
                "memory::{}::entry::{}::{}::{}",
                self.namespace,
                file_slug,
                line_no,
                Self::slugify(&entry_text)
            );
            Self::push_node(
                nodes,
                node_seen,
                MemoryNodeRecord {
                    id: entry_id.clone(),
                    label: entry_text.clone(),
                    node_type,
                    tags,
                    last_modified: file_timestamp,
                    change_frequency: 0.5,
                    source_path: rel_path.clone(),
                    line_start: Some(line_no as u32),
                    line_end: Some(line_no as u32),
                    excerpt: Self::excerpt(&entry_text),
                    namespace: self.namespace.clone(),
                    canonical,
                },
            );
            Self::push_edge(
                edges,
                edge_seen,
                MemoryEdgeRecord {
                    source: current_parent.clone(),
                    target: entry_id.clone(),
                    relation,
                    weight: 0.85,
                    direction: EdgeDirection::Forward,
                    inhibitory: false,
                    causal_strength: 0.55,
                },
            );

            for file_ref in file_ref_re.captures_iter(&entry_text) {
                let referenced = file_ref.name("path").unwrap().as_str();
                let reference_id = format!(
                    "memory::{}::reference::{}",
                    self.namespace,
                    Self::slugify(referenced)
                );

                Self::push_node(
                    nodes,
                    node_seen,
                    MemoryNodeRecord {
                        id: reference_id.clone(),
                        label: referenced.to_string(),
                        node_type: NodeType::Reference,
                        tags: vec![
                            "memory".into(),
                            "memory:reference".into(),
                            format!("namespace:{}", self.namespace),
                        ],
                        last_modified: file_timestamp,
                        change_frequency: 0.35,
                        source_path: rel_path.clone(),
                        line_start: Some(line_no as u32),
                        line_end: Some(line_no as u32),
                        excerpt: Self::excerpt(referenced),
                        namespace: self.namespace.clone(),
                        canonical,
                    },
                );
                Self::push_edge(
                    edges,
                    edge_seen,
                    MemoryEdgeRecord {
                        source: entry_id.clone(),
                        target: reference_id,
                        relation: "references".into(),
                        weight: 0.7,
                        direction: EdgeDirection::Forward,
                        inhibitory: false,
                        causal_strength: 0.3,
                    },
                );
            }
        }

        Ok(())
    }
}

impl IngestAdapter for MemoryIngestAdapter {
    fn domain(&self) -> &str {
        "memory"
    }

    fn ingest(&self, root: &Path) -> M1ndResult<(Graph, IngestStats)> {
        let start = Instant::now();
        let mut stats = IngestStats::default();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_seen = HashSet::new();
        let mut edge_seen = HashSet::new();

        let files = self.collect_files(root);
        stats.files_scanned = files.len() as u64;

        let root_dir = if root.is_dir() {
            root.to_path_buf()
        } else {
            root.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        };

        for file in &files {
            self.parse_memory_file(
                &root_dir,
                file,
                &mut nodes,
                &mut edges,
                &mut node_seen,
                &mut edge_seen,
            )?;
            stats.files_parsed += 1;
        }

        let mut graph = Graph::with_capacity(nodes.len(), edges.len());
        for node in &nodes {
            let tags: Vec<&str> = node.tags.iter().map(String::as_str).collect();
            if let Ok(node_id) = graph.add_node(
                &node.id,
                &node.label,
                node.node_type,
                &tags,
                node.last_modified,
                node.change_frequency,
            ) {
                graph.set_node_provenance(
                    node_id,
                    NodeProvenanceInput {
                        source_path: Some(&node.source_path),
                        line_start: node.line_start,
                        line_end: node.line_end,
                        excerpt: node.excerpt.as_deref(),
                        namespace: Some(&node.namespace),
                        canonical: node.canonical,
                    },
                );
                stats.nodes_created += 1;
            }
        }

        for edge in &edges {
            if let (Some(source), Some(target)) = (
                graph.resolve_id(&edge.source),
                graph.resolve_id(&edge.target),
            ) {
                if graph
                    .add_edge(
                        source,
                        target,
                        &edge.relation,
                        FiniteF32::new(edge.weight),
                        edge.direction,
                        edge.inhibitory,
                        FiniteF32::new(edge.causal_strength),
                    )
                    .is_ok()
                {
                    stats.edges_created += 1;
                }
            }
        }

        if graph.num_nodes() > 0 {
            graph.finalize()?;
        }
        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((graph, stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn memory_adapter_extracts_headings_lists_and_tables() {
        let tmpdir =
            std::env::temp_dir().join(format!("m1nd_memory_adapter_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmpdir);
        fs::create_dir_all(&tmpdir).unwrap();

        let note = tmpdir.join("2026-03-13.md");
        fs::write(
            &note,
            "# Night Shift\n- Decision: use m1nd first\n| Time | Mode | Note |\n| --- | --- | --- |\n| 22:00-04:00 | Batman mode | Peak build window |\nA plain memory line about graph perspective.\n",
        )
        .unwrap();

        let adapter = MemoryIngestAdapter::new(Some("memory".into()));
        let (graph, stats) = adapter.ingest(&tmpdir).unwrap();

        assert_eq!(stats.files_parsed, 1);
        assert!(stats.nodes_created >= 4);
        assert!(graph
            .resolve_id("memory::memory::file::2026-03-13-md")
            .is_some());
        let provenance = graph.resolve_node_provenance(
            graph
                .resolve_id("memory::memory::file::2026-03-13-md")
                .unwrap(),
        );
        assert_eq!(provenance.source_path.as_deref(), Some("2026-03-13.md"));
        assert!(provenance.canonical);
        assert!(graph.id_to_node.keys().any(|key| graph
            .strings
            .resolve(*key)
            .contains("batman-mode-peak-build-window")));

        let _ = fs::remove_dir_all(&tmpdir);
    }
}
