// === crates/m1nd-ingest/src/extract/typescript.rs ===

use super::{
    strip_comments_and_strings, CommentSyntax, ExtractedEdge, ExtractedNode, ExtractionResult,
    Extractor,
};
use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use regex::Regex;

/// TypeScript/JavaScript extractor using regex.
/// Replaces: ingest.py TypeScriptExtractor
pub struct TypeScriptExtractor {
    re_func: Regex,
    re_class: Regex,
    re_interface: Regex,
    re_import: Regex,
    re_arrow: Regex,
    re_import_names: Regex, // Extract named imports: import { A, B } from ...
    re_type_ref: Regex,     // TypeScript type references
}

impl TypeScriptExtractor {
    pub fn new() -> Self {
        Self {
            re_func: Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*[<(]").unwrap(),
            re_class: Regex::new(r"^\s*(?:export\s+)?(?:abstract\s+)?class\s+(\w+)").unwrap(),
            re_interface: Regex::new(r"^\s*(?:export\s+)?interface\s+(\w+)").unwrap(),
            re_import: Regex::new(r#"^\s*import\s+.*from\s+['"]([@\w./\-]+)['"]"#).unwrap(),
            re_arrow: Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?(?:\([^)]*\)|[^=])\s*=>").unwrap(),
            re_import_names: Regex::new(r#"import\s*\{([^}]+)\}"#).unwrap(),
            re_type_ref: Regex::new(r":\s*([A-Z]\w+)").unwrap(),
        }
    }
}

impl Default for TypeScriptExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extractor for TypeScriptExtractor {
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
            tags: vec!["typescript".into()],
            line: 1,
            end_line: text.lines().count() as u32,
        });

        for (line_num, line) in cleaned_lines.iter().enumerate() {
            let ln = (line_num + 1) as u32;

            if let Some(caps) = self.re_class.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::class::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Class,
                    tags: vec!["typescript".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_interface.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::interface::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Type,
                    tags: vec!["typescript".into(), "interface".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_func.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["typescript".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_arrow.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["typescript".into(), "arrow".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            }

            if let Some(caps) = self.re_import.captures(line) {
                let module = caps.get(1).unwrap().as_str();
                let ref_id = format!("ref::{}", module);
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: ref_id.clone(),
                    relation: "imports".into(),
                    weight: 0.5,
                });
                unresolved_refs.push(ref_id);

                // Also extract named imports: import { Foo, Bar } from '...'
                if let Some(names) = self.re_import_names.captures(line) {
                    let names_str = names.get(1).unwrap().as_str();
                    for name in names_str.split(',') {
                        let name = name.trim().split(" as ").next().unwrap_or("").trim();
                        if !name.is_empty() {
                            let ref_id = format!("ref::{}", name);
                            if !unresolved_refs.contains(&ref_id) {
                                edges.push(ExtractedEdge {
                                    source: file_id.to_string(),
                                    target: ref_id.clone(),
                                    relation: "imports".into(),
                                    weight: 0.5,
                                });
                                unresolved_refs.push(ref_id);
                            }
                        }
                    }
                }
            }

            // Type references in annotations (: TypeName)
            // Comments already stripped by pre-processor
            if !line.trim_start().starts_with("import") {
                for caps in self.re_type_ref.captures_iter(line) {
                    let type_name = caps.get(1).unwrap().as_str();
                    if !matches!(
                        type_name,
                        "String"
                            | "Number"
                            | "Boolean"
                            | "Promise"
                            | "Array"
                            | "Record"
                            | "Partial"
                            | "Required"
                            | "Readonly"
                            | "Map"
                            | "Set"
                            | "Date"
                            | "Error"
                            | "Function"
                            | "Object"
                            | "Omit"
                            | "Pick"
                    ) {
                        let ref_id = format!("ref::{}", type_name);
                        if !unresolved_refs.contains(&ref_id) {
                            edges.push(ExtractedEdge {
                                source: file_id.to_string(),
                                target: ref_id.clone(),
                                relation: "references".into(),
                                weight: 0.3,
                            });
                            unresolved_refs.push(ref_id);
                        }
                    }
                }
            }
        }

        Ok(ExtractionResult {
            nodes,
            edges,
            unresolved_refs,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx", "mjs", "cjs"]
    }
}
