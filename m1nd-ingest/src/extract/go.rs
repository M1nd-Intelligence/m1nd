// === crates/m1nd-ingest/src/extract/go.rs ===

use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use regex::Regex;
use super::{Extractor, ExtractionResult, ExtractedNode, ExtractedEdge, CommentSyntax, strip_comments_and_strings};

/// Go extractor using regex.
pub struct GoExtractor {
    re_func: Regex,
    re_method: Regex,
    re_struct: Regex,
    re_interface: Regex,
    re_import: Regex,
}

impl GoExtractor {
    pub fn new() -> Self {
        Self {
            re_func: Regex::new(r"^func\s+(\w+)\s*\(").unwrap(),
            re_method: Regex::new(r"^func\s+\([^)]+\)\s+(\w+)\s*\(").unwrap(),
            re_struct: Regex::new(r"^type\s+(\w+)\s+struct\b").unwrap(),
            re_interface: Regex::new(r"^type\s+(\w+)\s+interface\b").unwrap(),
            re_import: Regex::new(r#"^\s*"([^"]+)""#).unwrap(),
        }
    }
}

impl Default for GoExtractor {
    fn default() -> Self { Self::new() }
}

impl Extractor for GoExtractor {
    fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult> {
        let text = String::from_utf8_lossy(content);
        let cleaned_lines = strip_comments_and_strings(&text, CommentSyntax::GO);
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut unresolved_refs = Vec::new();

        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        nodes.push(ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: NodeType::File,
            tags: vec!["go".into()],
            line: 1,
            end_line: text.lines().count() as u32,
        });

        let mut in_import_block = false;

        for (line_num, line) in cleaned_lines.iter().enumerate() {
            let ln = (line_num + 1) as u32;
            let trimmed = line.trim();

            // Track import blocks
            if trimmed.starts_with("import (") {
                in_import_block = true;
                continue;
            }
            if in_import_block {
                if trimmed == ")" {
                    in_import_block = false;
                    continue;
                }
                if let Some(caps) = self.re_import.captures(trimmed) {
                    let path = caps.get(1).unwrap().as_str();
                    let ref_id = format!("ref::{}", path);
                    edges.push(ExtractedEdge {
                        source: file_id.to_string(), target: ref_id.clone(),
                        relation: "imports".into(), weight: 0.5,
                    });
                    unresolved_refs.push(ref_id);
                }
                continue;
            }

            if let Some(caps) = self.re_struct.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::struct::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Struct,
                    tags: vec!["go".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            } else if let Some(caps) = self.re_interface.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::interface::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Type,
                    tags: vec!["go".into(), "interface".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            } else if let Some(caps) = self.re_method.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["go".into(), "method".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            } else if let Some(caps) = self.re_func.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["go".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            }

            // Single-line import
            if trimmed.starts_with("import ") && !trimmed.contains('(') {
                if let Some(caps) = self.re_import.captures(trimmed.trim_start_matches("import ")) {
                    let path = caps.get(1).unwrap().as_str();
                    let ref_id = format!("ref::{}", path);
                    edges.push(ExtractedEdge {
                        source: file_id.to_string(), target: ref_id.clone(),
                        relation: "imports".into(), weight: 0.5,
                    });
                    unresolved_refs.push(ref_id);
                }
            }
        }

        Ok(ExtractionResult { nodes, edges, unresolved_refs })
    }

    fn extensions(&self) -> &[&str] {
        &["go"]
    }
}
