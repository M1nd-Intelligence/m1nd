// === crates/m1nd-ingest/src/extract/python.rs ===

use super::{
    strip_comments_and_strings, CommentSyntax, ExtractedEdge, ExtractedNode, ExtractionResult,
    Extractor,
};
use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use regex::Regex;

/// Python extractor using regex.
/// FM-ING-009 fix: patterns allow leading whitespace (captures indented defs).
/// Replaces: ingest.py PythonExtractor
pub struct PythonExtractor {
    re_func: Regex,
    re_class: Regex,
    re_import: Regex,
    re_from_import: Regex,
    re_from_import_names: Regex, // from X import A, B, C
    re_type_hint: Regex,         // Type hints: -> Type, : Type
    re_class_inherit: Regex,     // class Foo(Bar, Baz):
    re_decorator: Regex,         // @decorator_name (Task #3)
    re_method_call: Regex,       // receiver.method() calls (Task #7)
}

impl PythonExtractor {
    pub fn new() -> Self {
        Self {
            re_func: Regex::new(r"^\s*(?:async\s+)?def\s+(\w+)\s*\(").unwrap(),
            re_class: Regex::new(r"^\s*class\s+(\w+)\s*[:\(]").unwrap(),
            re_import: Regex::new(r"^\s*import\s+([\w.]+)").unwrap(),
            re_from_import: Regex::new(r"^\s*from\s+([\w.]+)\s+import").unwrap(),
            re_from_import_names: Regex::new(r"from\s+[\w.]+\s+import\s+(.+)").unwrap(),
            // FIX #2: was `->s*\s*` (missing backslash), now `->\s*`
            re_type_hint: Regex::new(r"(?::\s*|->\s*)([A-Z]\w+)").unwrap(),
            re_class_inherit: Regex::new(r"class\s+\w+\(([^)]+)\)").unwrap(),
            // Task #3: Python decorator extraction
            re_decorator: Regex::new(r"^\s*@(\w+(?:\.\w+)*)").unwrap(),
            // Task #7: receiver.method() calls
            re_method_call: Regex::new(r"(\w+)\.(\w+)\s*\(").unwrap(),
        }
    }
}

impl Default for PythonExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Extractor for PythonExtractor {
    fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult> {
        let text = String::from_utf8_lossy(content);
        let cleaned_lines = strip_comments_and_strings(&text, CommentSyntax::PYTHON);
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut unresolved_refs = Vec::new();

        // File node
        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        nodes.push(ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: NodeType::File,
            tags: vec!["python".into()],
            line: 1,
            end_line: text.lines().count() as u32,
        });

        // Track pending decorator for association with the next def/class
        let mut pending_decorators: Vec<String> = Vec::new();
        // Track last defined function/class node_id for decorator edge emission
        let mut last_node_id: Option<String> = None;

        for (line_num, line) in cleaned_lines.iter().enumerate() {
            let ln = (line_num + 1) as u32;

            // Task #3: Decorator extraction
            if let Some(caps) = self.re_decorator.captures(line) {
                let decorator_name = caps.get(1).unwrap().as_str().to_string();
                pending_decorators.push(decorator_name);
                continue; // decorators are standalone lines, proceed to next
            }

            if let Some(caps) = self.re_class.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::class::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Class,
                    tags: vec!["python".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id.clone(),
                    relation: "contains".into(),
                    weight: 1.0,
                });
                // Emit decorator reference edges
                for dec in pending_decorators.drain(..) {
                    let ref_id = format!("ref::{}", dec);
                    edges.push(ExtractedEdge {
                        source: node_id.clone(),
                        target: ref_id.clone(),
                        relation: "references".into(),
                        weight: 0.4,
                    });
                    if !unresolved_refs.contains(&ref_id) {
                        unresolved_refs.push(ref_id);
                    }
                }
                last_node_id = Some(node_id);
            } else if let Some(caps) = self.re_func.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["python".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id.clone(),
                    relation: "contains".into(),
                    weight: 1.0,
                });
                // Emit decorator reference edges
                for dec in pending_decorators.drain(..) {
                    let ref_id = format!("ref::{}", dec);
                    edges.push(ExtractedEdge {
                        source: node_id.clone(),
                        target: ref_id.clone(),
                        relation: "references".into(),
                        weight: 0.4,
                    });
                    if !unresolved_refs.contains(&ref_id) {
                        unresolved_refs.push(ref_id);
                    }
                }
                last_node_id = Some(node_id);
            } else {
                // If we accumulated decorators but hit a non-def/non-class line,
                // clear them (stale decorators should not bleed into later defs).
                if !pending_decorators.is_empty() && !line.trim().is_empty() {
                    pending_decorators.clear();
                }
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
            } else if let Some(caps) = self.re_from_import.captures(line) {
                let module = caps.get(1).unwrap().as_str();
                let ref_id = format!("ref::{}", module);
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: ref_id.clone(),
                    relation: "imports".into(),
                    weight: 0.5,
                });
                unresolved_refs.push(ref_id);

                // Extract named imports: from X import A, B, C
                if let Some(names_caps) = self.re_from_import_names.captures(line) {
                    let names_str = names_caps.get(1).unwrap().as_str();
                    for name in names_str.split(',') {
                        let name = name.trim().split(" as ").next().unwrap_or("").trim();
                        if !name.is_empty() && name != "*" {
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

            // Class inheritance: class Foo(Bar, Baz)
            if let Some(caps) = self.re_class_inherit.captures(line) {
                let bases = caps.get(1).unwrap().as_str();
                for base in bases.split(',') {
                    let base = base.trim();
                    if !base.is_empty() && base != "object" {
                        let ref_id = format!("ref::{}", base);
                        if !unresolved_refs.contains(&ref_id) {
                            edges.push(ExtractedEdge {
                                source: file_id.to_string(),
                                target: ref_id.clone(),
                                relation: "implements".into(),
                                weight: 0.7,
                            });
                            unresolved_refs.push(ref_id);
                        }
                    }
                }
            }

            // Type hints: : TypeName, -> TypeName
            // Also: method calls (Task #7)
            if !line.trim_start().starts_with("import") && !line.trim_start().starts_with("from") {
                for caps in self.re_type_hint.captures_iter(line) {
                    let type_name = caps.get(1).unwrap().as_str();
                    if !matches!(
                        type_name,
                        "None"
                            | "True"
                            | "False"
                            | "Any"
                            | "Optional"
                            | "List"
                            | "Dict"
                            | "Tuple"
                            | "Set"
                            | "Union"
                            | "Type"
                            | "Callable"
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

                // Task #7: receiver.method() calls -> "calls" edges
                if !line.trim_start().starts_with("def ")
                    && !line.trim_start().starts_with("class ")
                    && !line.trim_start().starts_with("@")
                {
                    for caps in self.re_method_call.captures_iter(line) {
                        let receiver = caps.get(1).unwrap().as_str();
                        let method = caps.get(2).unwrap().as_str();
                        // Skip self.method and cls.method (internal calls, not cross-refs)
                        if receiver == "self" || receiver == "cls" || receiver == "super" {
                            continue;
                        }
                        // If receiver starts with uppercase, it's likely a type call: Type.method(
                        let ref_target =
                            if receiver.chars().next().map_or(false, |c| c.is_uppercase()) {
                                format!("ref::{}", receiver)
                            } else {
                                format!("ref::{}", method)
                            };
                        if !unresolved_refs.contains(&ref_target) {
                            edges.push(ExtractedEdge {
                                source: file_id.to_string(),
                                target: ref_target.clone(),
                                relation: "calls".into(),
                                weight: 0.4,
                            });
                            unresolved_refs.push(ref_target);
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
        &["py", "pyi"]
    }
}
