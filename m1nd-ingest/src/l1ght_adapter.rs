use crate::{IngestAdapter, IngestStats};
use m1nd_core::error::M1ndResult;
use m1nd_core::graph::{Graph, NodeProvenanceInput};
use m1nd_core::types::{EdgeDirection, FiniteF32, NodeType};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
struct L1ghtNodeRecord {
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
struct L1ghtEdgeRecord {
    source: String,
    target: String,
    relation: String,
    weight: f32,
    direction: EdgeDirection,
    inhibitory: bool,
    causal_strength: f32,
}

#[derive(Default)]
struct HeaderMeta {
    protocol: Option<String>,
    node: Option<String>,
    state: Option<String>,
    color: Option<String>,
    glyph: Option<String>,
    completeness: Option<String>,
    proof: Option<String>,
    depends_on: Vec<String>,
    next: Vec<String>,
}

pub struct L1ghtIngestAdapter {
    namespace: String,
}

impl L1ghtIngestAdapter {
    pub fn new(namespace: Option<String>) -> Self {
        let namespace = namespace
            .unwrap_or_else(|| "light".to_string())
            .trim()
            .to_lowercase();
        let namespace = if namespace.is_empty() {
            "light".to_string()
        } else {
            namespace
        };
        Self { namespace }
    }

    pub fn looks_like_l1ght(text: &str) -> bool {
        if text.contains("Protocol: L1GHT/") {
            return true;
        }

        let mut hits = 0;
        for marker in [
            "[⍂ entity:",
            "[⍐ state:",
            "[⍌ event:",
            "[𝔻 confidence:",
            "[𝔻 ambiguity:",
            "[𝔻 evidence:",
            "[⟁ depends_on:",
            "[⟁ binds_to:",
            "[⟁ tests:",
            "[RED blocker:",
            "[AMBER warning:",
        ] {
            if text.contains(marker) {
                hits += 1;
            }
        }

        hits >= 2
    }

    fn accepted_extension(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()),
            Some(ext) if matches!(ext.as_str(), "md" | "markdown")
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
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
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
            None
        } else {
            Some(trimmed.chars().take(220).collect())
        }
    }

    fn push_node(nodes: &mut Vec<L1ghtNodeRecord>, seen: &mut HashSet<String>, record: L1ghtNodeRecord) {
        if seen.insert(record.id.clone()) {
            nodes.push(record);
        }
    }

    fn push_edge(
        edges: &mut Vec<L1ghtEdgeRecord>,
        seen: &mut HashSet<(String, String, String, u8)>,
        record: L1ghtEdgeRecord,
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

    fn parse_header(lines: &[&str]) -> HeaderMeta {
        let mut meta = HeaderMeta::default();
        let mut current_list: Option<&str> = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed == "---" {
                continue;
            }
            if let Some(value) = trimmed.strip_prefix("Protocol:") {
                meta.protocol = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("Node:") {
                meta.node = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("State:") {
                meta.state = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("Color:") {
                meta.color = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("Glyph:") {
                meta.glyph = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("Completeness:") {
                meta.completeness = Some(value.trim().to_string());
                current_list = None;
            } else if let Some(value) = trimmed.strip_prefix("Proof:") {
                meta.proof = Some(value.trim().to_string());
                current_list = None;
            } else if trimmed == "Depends on:" {
                current_list = Some("depends_on");
            } else if trimmed == "Next:" {
                current_list = Some("next");
            } else if let Some(value) = trimmed.strip_prefix("- ") {
                match current_list {
                    Some("depends_on") => meta.depends_on.push(value.trim().to_string()),
                    Some("next") => meta.next.push(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        meta
    }

    fn parse_file(
        &self,
        root: &Path,
        path: &Path,
        nodes: &mut Vec<L1ghtNodeRecord>,
        edges: &mut Vec<L1ghtEdgeRecord>,
        node_seen: &mut HashSet<String>,
        edge_seen: &mut HashSet<(String, String, String, u8)>,
    ) -> M1ndResult<()> {
        let text = std::fs::read_to_string(path)?;
        if !Self::looks_like_l1ght(&text) {
            return Ok(());
        }

        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let file_slug = Self::slugify(&rel_path);
        let file_id = format!("light::{}::file::{}", self.namespace, file_slug);
        let timestamp = Self::file_timestamp(path);
        let file_label = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&rel_path)
            .to_string();

        Self::push_node(
            nodes,
            node_seen,
            L1ghtNodeRecord {
                id: file_id.clone(),
                label: file_label.clone(),
                node_type: NodeType::File,
                tags: vec!["light".into(), format!("namespace:{}", self.namespace)],
                last_modified: timestamp,
                change_frequency: 0.7,
                source_path: rel_path.clone(),
                line_start: None,
                line_end: None,
                excerpt: Some(file_label.clone()),
                namespace: self.namespace.clone(),
                canonical: true,
            },
        );

        let lines: Vec<&str> = text.lines().collect();
        let header_meta = Self::parse_header(&lines[..lines.len().min(40)]);
        let section_re = Regex::new(r"^##\s+(.+?)\s*$").unwrap();
        let tag_re = Regex::new(r"\[(?P<tag>[^\]]+)\]").unwrap();

        let mut current_parent = file_id.clone();
        let mut section_counts: HashMap<String, usize> = HashMap::new();

        for (key, value) in [
            ("protocol", header_meta.protocol.clone()),
            ("node", header_meta.node.clone()),
            ("state", header_meta.state.clone()),
            ("color", header_meta.color.clone()),
            ("glyph", header_meta.glyph.clone()),
            ("completeness", header_meta.completeness.clone()),
            ("proof", header_meta.proof.clone()),
        ] {
            if let Some(value) = value {
                let meta_id = format!("light::{}::meta::{}::{}", self.namespace, file_slug, key);
                Self::push_node(
                    nodes,
                    node_seen,
                    L1ghtNodeRecord {
                        id: meta_id.clone(),
                        label: value.clone(),
                        node_type: NodeType::Concept,
                        tags: vec!["light".into(), format!("light:{}", key)],
                        last_modified: timestamp,
                        change_frequency: 0.55,
                        source_path: rel_path.clone(),
                        line_start: None,
                        line_end: None,
                        excerpt: Self::excerpt(&value),
                        namespace: self.namespace.clone(),
                        canonical: true,
                    },
                );
                let relation = match key {
                    "protocol" => "defines_protocol",
                    "state" => "has_state",
                    "glyph" => "has_glyph",
                    "color" => "has_color",
                    _ => "has_metadata",
                };
                Self::push_edge(
                    edges,
                    edge_seen,
                    L1ghtEdgeRecord {
                        source: file_id.clone(),
                        target: meta_id,
                        relation: relation.into(),
                        weight: 1.0,
                        direction: EdgeDirection::Forward,
                        inhibitory: false,
                        causal_strength: 0.85,
                    },
                );
            }
        }

        for dep in header_meta.depends_on {
            let dep_id = format!("light::{}::dep::{}::{}", self.namespace, file_slug, Self::slugify(&dep));
            Self::push_node(
                nodes,
                node_seen,
                L1ghtNodeRecord {
                    id: dep_id.clone(),
                    label: dep.clone(),
                    node_type: NodeType::Reference,
                    tags: vec!["light".into(), "light:dependency".into()],
                    last_modified: timestamp,
                    change_frequency: 0.45,
                    source_path: rel_path.clone(),
                    line_start: None,
                    line_end: None,
                    excerpt: Self::excerpt(&dep),
                    namespace: self.namespace.clone(),
                    canonical: true,
                },
            );
            Self::push_edge(
                edges,
                edge_seen,
                L1ghtEdgeRecord {
                    source: file_id.clone(),
                    target: dep_id,
                    relation: "depends_on".into(),
                    weight: 1.0,
                    direction: EdgeDirection::Forward,
                    inhibitory: false,
                    causal_strength: 0.9,
                },
            );
        }

        for next in header_meta.next {
            let next_id = format!("light::{}::next::{}::{}", self.namespace, file_slug, Self::slugify(&next));
            Self::push_node(
                nodes,
                node_seen,
                L1ghtNodeRecord {
                    id: next_id.clone(),
                    label: next.clone(),
                    node_type: NodeType::Reference,
                    tags: vec!["light".into(), "light:next".into()],
                    last_modified: timestamp,
                    change_frequency: 0.5,
                    source_path: rel_path.clone(),
                    line_start: None,
                    line_end: None,
                    excerpt: Self::excerpt(&next),
                    namespace: self.namespace.clone(),
                    canonical: true,
                },
            );
            Self::push_edge(
                edges,
                edge_seen,
                L1ghtEdgeRecord {
                    source: file_id.clone(),
                    target: next_id,
                    relation: "next_binding".into(),
                    weight: 0.95,
                    direction: EdgeDirection::Forward,
                    inhibitory: false,
                    causal_strength: 0.82,
                },
            );
        }

        for (idx, line) in lines.iter().enumerate() {
            let line_no = idx as u32 + 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(caps) = section_re.captures(trimmed) {
                let heading = caps.get(1).unwrap().as_str().trim();
                let slug = Self::slugify(heading);
                let count = section_counts.entry(slug.clone()).or_insert(0);
                *count += 1;
                let section_id = format!(
                    "light::{}::section::{}::{}-{}",
                    self.namespace, file_slug, slug, count
                );
                Self::push_node(
                    nodes,
                    node_seen,
                    L1ghtNodeRecord {
                        id: section_id.clone(),
                        label: heading.to_string(),
                        node_type: NodeType::Module,
                        tags: vec!["light".into(), "light:section".into()],
                        last_modified: timestamp,
                        change_frequency: 0.5,
                        source_path: rel_path.clone(),
                        line_start: Some(line_no),
                        line_end: Some(line_no),
                        excerpt: Self::excerpt(heading),
                        namespace: self.namespace.clone(),
                        canonical: true,
                    },
                );
                Self::push_edge(
                    edges,
                    edge_seen,
                    L1ghtEdgeRecord {
                        source: file_id.clone(),
                        target: section_id.clone(),
                        relation: "contains_section".into(),
                        weight: 1.0,
                        direction: EdgeDirection::Forward,
                        inhibitory: false,
                        causal_strength: 0.8,
                    },
                );
                current_parent = section_id;
            }

            for caps in tag_re.captures_iter(trimmed) {
                let raw = caps.name("tag").unwrap().as_str().trim();
                let relation = if raw.starts_with('⍂') && raw.contains("entity:") {
                    "declares_entity"
                } else if raw.starts_with('⍐') && raw.contains("state:") {
                    "declares_state"
                } else if raw.starts_with('⍌') && raw.contains("event:") {
                    "declares_event"
                } else if raw.starts_with('⟁') && raw.contains("depends_on:") {
                    "depends_on"
                } else if raw.starts_with('⟁') && raw.contains("binds_to:") {
                    "binds_to"
                } else if raw.starts_with('⟁') && raw.contains("tests:") {
                    "declares_test"
                } else if raw.starts_with("RED blocker:") {
                    "declares_blocker"
                } else if raw.starts_with("AMBER warning:") {
                    "declares_warning"
                } else {
                    "declares_metadata"
                };

                let node_type = if relation == "declares_test" {
                    NodeType::Process
                } else {
                    NodeType::Concept
                };

                let tag_id = format!(
                    "light::{}::tag::{}::{}::{}",
                    self.namespace,
                    file_slug,
                    line_no,
                    Self::slugify(raw)
                );
                Self::push_node(
                    nodes,
                    node_seen,
                    L1ghtNodeRecord {
                        id: tag_id.clone(),
                        label: raw.to_string(),
                        node_type,
                        tags: vec!["light".into(), format!("light:{}", relation)],
                        last_modified: timestamp,
                        change_frequency: 0.45,
                        source_path: rel_path.clone(),
                        line_start: Some(line_no),
                        line_end: Some(line_no),
                        excerpt: Self::excerpt(raw),
                        namespace: self.namespace.clone(),
                        canonical: true,
                    },
                );
                Self::push_edge(
                    edges,
                    edge_seen,
                    L1ghtEdgeRecord {
                        source: current_parent.clone(),
                        target: tag_id,
                        relation: relation.into(),
                        weight: 0.9,
                        direction: EdgeDirection::Forward,
                        inhibitory: false,
                        causal_strength: 0.7,
                    },
                );
            }
        }

        Ok(())
    }
}

impl IngestAdapter for L1ghtIngestAdapter {
    fn domain(&self) -> &str {
        "light"
    }

    fn ingest(&self, root: &Path) -> M1ndResult<(Graph, IngestStats)> {
        let start = Instant::now();
        let files = self.collect_files(root);
        let mut stats = IngestStats::default();
        stats.files_scanned = files.len() as u64;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_seen = HashSet::new();
        let mut edge_seen = HashSet::new();

        for path in files {
            let text = std::fs::read_to_string(&path)?;
            if !Self::looks_like_l1ght(&text) {
                continue;
            }
            self.parse_file(root, &path, &mut nodes, &mut edges, &mut node_seen, &mut edge_seen)?;
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
            if let (Some(source), Some(target)) = (graph.resolve_id(&edge.source), graph.resolve_id(&edge.target)) {
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
