// === crates/m1nd-ingest/src/extract/java.rs ===

use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use regex::Regex;
use super::{Extractor, ExtractionResult, ExtractedNode, ExtractedEdge, CommentSyntax, strip_comments_and_strings};

/// Java extractor using regex.
pub struct JavaExtractor {
    re_class: Regex,
    re_interface: Regex,
    re_enum: Regex,
    re_method: Regex,
    re_import: Regex,
}

impl JavaExtractor {
    pub fn new() -> Self {
        Self {
            re_class: Regex::new(r"^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:abstract\s+)?(?:final\s+)?class\s+(\w+)").unwrap(),
            re_interface: Regex::new(r"^\s*(?:public|private|protected)?\s*interface\s+(\w+)").unwrap(),
            re_enum: Regex::new(r"^\s*(?:public|private|protected)?\s*enum\s+(\w+)").unwrap(),
            re_method: Regex::new(r"^\s*(?:public|private|protected)\s+(?:static\s+)?(?:final\s+)?(?:synchronized\s+)?(?:abstract\s+)?(?:\w+(?:<[^>]*>)?)\s+(\w+)\s*\(").unwrap(),
            re_import: Regex::new(r"^\s*import\s+(?:static\s+)?([\w.]+)\s*;").unwrap(),
        }
    }
}

impl Default for JavaExtractor {
    fn default() -> Self { Self::new() }
}

impl Extractor for JavaExtractor {
    fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult> {
        let text = String::from_utf8_lossy(content);
        let cleaned_lines = strip_comments_and_strings(&text, CommentSyntax::C_STYLE);
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut unresolved_refs = Vec::new();

        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        nodes.push(ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: NodeType::File,
            tags: vec!["java".into()],
            line: 1,
            end_line: text.lines().count() as u32,
        });

        for (line_num, line) in cleaned_lines.iter().enumerate() {
            let ln = (line_num + 1) as u32;

            if let Some(caps) = self.re_class.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::class::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Class,
                    tags: vec!["java".into()], line: ln, end_line: ln,
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
                    tags: vec!["java".into(), "interface".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            } else if let Some(caps) = self.re_enum.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::enum::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(), label: name.to_string(),
                    node_type: NodeType::Enum,
                    tags: vec!["java".into()], line: ln, end_line: ln,
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
                    tags: vec!["java".into()], line: ln, end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: node_id,
                    relation: "contains".into(), weight: 1.0,
                });
            }

            if let Some(caps) = self.re_import.captures(line) {
                let path = caps.get(1).unwrap().as_str();
                let ref_id = format!("ref::{}", path);
                edges.push(ExtractedEdge {
                    source: file_id.to_string(), target: ref_id.clone(),
                    relation: "imports".into(), weight: 0.5,
                });
                unresolved_refs.push(ref_id);
            }
        }

        Ok(ExtractionResult { nodes, edges, unresolved_refs })
    }

    fn extensions(&self) -> &[&str] {
        &["java"]
    }
}
