// === crates/m1nd-ingest/src/extract/rust_lang.rs ===

use super::{
    strip_comments_and_strings, CommentSyntax, ExtractedEdge, ExtractedNode, ExtractionResult,
    Extractor,
};
use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use regex::Regex;

/// Rust extractor using regex.
/// Replaces: ingest.py RustExtractor
pub struct RustExtractor {
    re_fn: Regex,
    re_struct: Regex,
    re_enum: Regex,
    re_trait: Regex,
    re_impl: Regex,
    re_use: Regex,
    re_mod: Regex,
    // Call/reference detection (non-definition lines)
    re_method_call: Regex,  // .method_name( or ::method_name(
    re_type_ref: Regex,     // UpperCamelCase identifiers (type references)
    re_fn_sig_types: Regex, // Type names in fn signatures: &Type, Type, Box<Type>
    // Enum variant extraction
    re_variant: Regex, // Variant inside enum { } block
}

impl RustExtractor {
    pub fn new() -> Self {
        Self {
            re_fn: Regex::new(
                r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?fn\s+(\w+)",
            )
            .unwrap(),
            re_struct: Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+(\w+)").unwrap(),
            re_enum: Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+(\w+)").unwrap(),
            re_trait: Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)").unwrap(),
            re_impl: Regex::new(r"^\s*impl(?:<[^>]*>)?\s+(?:(\w+)\s+for\s+)?(\w+)").unwrap(),
            re_use: Regex::new(r"^\s*(?:pub\s+)?use\s+(.+);").unwrap(),
            re_mod: Regex::new(r"^\s*(?:pub\s+)?mod\s+(\w+)").unwrap(),
            // Detect Type::method( and .method( calls
            re_method_call: Regex::new(r"(?:(\w+)::(\w+)|\.(\w+))\s*[(<]").unwrap(),
            // UpperCamelCase type references (2+ chars, starts upper)
            // FIX #4: Allow second char to be uppercase (catches CSR, XLR, PPMI)
            re_type_ref: Regex::new(r"\b([A-Z][A-Za-z]\w+)\b").unwrap(),
            // Types in fn signatures: after :, ->, in <>, etc.
            // FIX: was `->s*` (missing backslash), now `->\s*`
            re_fn_sig_types: Regex::new(r"(?::\s*&?(?:mut\s+)?|->\s*&?(?:mut\s+)?|<\s*)([A-Z]\w+)")
                .unwrap(),
            // Enum variants: identifiers at the start of a line (with optional whitespace)
            // inside an enum block, e.g. `    VariantName,` or `    VariantName(...)` or `    VariantName { ... }`
            re_variant: Regex::new(r"^\s+([A-Z]\w+)\s*(?:[,({]|$)").unwrap(),
        }
    }
}

impl Default for RustExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl RustExtractor {
    /// Expand Rust use paths with braces into individual paths.
    /// e.g., "foo::bar::{A, B}" -> ["foo::bar::A", "foo::bar::B"]
    /// and "foo::bar::Baz" -> ["foo::bar::Baz"]
    fn expand_use_path(path: &str) -> Vec<String> {
        if let Some(brace_start) = path.find('{') {
            if let Some(brace_end) = path.find('}') {
                let prefix = &path[..brace_start];
                let items = &path[brace_start + 1..brace_end];
                return items
                    .split(',')
                    .map(|item| format!("{}{}", prefix, item.trim()))
                    .filter(|s| !s.ends_with("::"))
                    .collect();
            }
        }
        vec![path.to_string()]
    }
}

impl Extractor for RustExtractor {
    fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult> {
        let text = String::from_utf8_lossy(content);
        let cleaned_lines = strip_comments_and_strings(&text, CommentSyntax::RUST);
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut unresolved_refs = Vec::new();

        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        nodes.push(ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: NodeType::File,
            tags: vec!["rust".into()],
            line: 1,
            end_line: text.lines().count() as u32,
        });

        // Track enum/impl blocks for variant and trait-impl-method extraction
        let mut in_enum: Option<String> = None; // Some(enum_node_id) when inside enum { }
        let mut in_impl_block = false; // true when inside impl { }
        let mut impl_is_trait = false; // true when `impl Trait for Type`
        let mut brace_depth: i32 = 0;
        let mut block_start_depth: i32 = 0;

        for (line_num, line) in cleaned_lines.iter().enumerate() {
            let ln = (line_num + 1) as u32;
            // Use the cleaned line for regex matching (comments/strings stripped)
            // but we still need to track brace depth across lines.

            // Update brace depth
            let open_count = line.chars().filter(|&c| c == '{').count() as i32;
            let close_count = line.chars().filter(|&c| c == '}').count() as i32;

            // Check if we're exiting the current enum or impl block
            if in_enum.is_some() && brace_depth + open_count - close_count <= block_start_depth {
                in_enum = None;
            }
            if in_impl_block && brace_depth + open_count - close_count <= block_start_depth {
                in_impl_block = false;
                impl_is_trait = false;
            }

            // --- Enum variant extraction (Task #5) ---
            if let Some(ref enum_id) = in_enum {
                if let Some(caps) = self.re_variant.captures(line) {
                    let variant_name = caps.get(1).unwrap().as_str();
                    // Skip common Rust keywords that might match
                    if !matches!(
                        variant_name,
                        "Self"
                            | "Some"
                            | "None"
                            | "Ok"
                            | "Err"
                            | "Box"
                            | "Vec"
                            | "String"
                            | "Option"
                            | "Result"
                    ) {
                        let variant_id = format!("{}::{}", enum_id, variant_name);
                        nodes.push(ExtractedNode {
                            id: variant_id.clone(),
                            label: variant_name.to_string(),
                            node_type: NodeType::Type,
                            tags: vec!["rust".into(), "variant".into()],
                            line: ln,
                            end_line: ln,
                        });
                        edges.push(ExtractedEdge {
                            source: enum_id.clone(),
                            target: variant_id,
                            relation: "contains".into(),
                            weight: 1.0,
                        });
                    }
                }
            }

            // --- Trait impl method extraction (Task #6) ---
            if in_impl_block && impl_is_trait {
                if let Some(caps) = self.re_fn.captures(line) {
                    let name = caps.get(1).unwrap().as_str();
                    let node_id = format!("{}::fn::{}", file_id, name);
                    // Only add if not already added by the general fn branch below
                    if !nodes.iter().any(|n| n.id == node_id) {
                        nodes.push(ExtractedNode {
                            id: node_id.clone(),
                            label: name.to_string(),
                            node_type: NodeType::Function,
                            tags: vec!["rust".into(), "impl_method".into()],
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
                }
            }

            // --- Standard extraction (struct, enum, trait, impl, fn, mod) ---
            if let Some(caps) = self.re_struct.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::struct::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Struct,
                    tags: vec!["rust".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_enum.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::enum::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Enum,
                    tags: vec!["rust".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id.clone(),
                    relation: "contains".into(),
                    weight: 1.0,
                });
                // Start tracking enum block for variant extraction
                if line.contains('{') {
                    in_enum = Some(node_id);
                    block_start_depth = brace_depth;
                }
            } else if let Some(caps) = self.re_trait.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::trait::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Type,
                    tags: vec!["rust".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_impl.captures(line) {
                let type_name = caps.get(2).unwrap().as_str();
                let ref_id = format!("ref::{}", type_name);
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: ref_id.clone(),
                    relation: "implements".into(),
                    weight: 0.8,
                });
                unresolved_refs.push(ref_id);

                // Track impl block for trait impl method extraction
                let is_trait_impl = caps.get(1).is_some();
                if line.contains('{') {
                    in_impl_block = true;
                    impl_is_trait = is_trait_impl;
                    block_start_depth = brace_depth;
                }
            } else if let Some(caps) = self.re_fn.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::fn::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Function,
                    tags: vec!["rust".into()],
                    line: ln,
                    end_line: ln,
                });
                edges.push(ExtractedEdge {
                    source: file_id.to_string(),
                    target: node_id,
                    relation: "contains".into(),
                    weight: 1.0,
                });
            } else if let Some(caps) = self.re_mod.captures(line) {
                let name = caps.get(1).unwrap().as_str();
                let node_id = format!("{}::mod::{}", file_id, name);
                nodes.push(ExtractedNode {
                    id: node_id.clone(),
                    label: name.to_string(),
                    node_type: NodeType::Module,
                    tags: vec!["rust".into()],
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

            if let Some(caps) = self.re_use.captures(line) {
                let path = caps.get(1).unwrap().as_str().trim();
                // Expand brace imports: use foo::{A, B} -> ref::foo::A, ref::foo::B
                let refs = Self::expand_use_path(path);
                for r in refs {
                    let ref_id = format!("ref::{}", r);
                    edges.push(ExtractedEdge {
                        source: file_id.to_string(),
                        target: ref_id.clone(),
                        relation: "imports".into(),
                        weight: 0.5,
                    });
                    unresolved_refs.push(ref_id);
                }
            }

            // Detect Type::method() calls and .method() calls (not on definition lines)
            if !line.trim_start().starts_with("pub")
                && !line.trim_start().starts_with("fn ")
                && !line.trim_start().starts_with("struct ")
                && !line.trim_start().starts_with("enum ")
                && !line.trim_start().starts_with("trait ")
                && !line.trim_start().starts_with("use ")
                && !line.trim_start().starts_with("mod ")
            {
                // Type::method( calls -- create ref to Type
                for caps in self.re_method_call.captures_iter(line) {
                    if let Some(type_match) = caps.get(1) {
                        let type_name = type_match.as_str();
                        // Only UpperCamelCase (likely a type, not a variable)
                        if type_name.chars().next().is_some_and(|c| c.is_uppercase())
                            && type_name.len() > 1
                        {
                            let ref_id = format!("ref::{}", type_name);
                            if !unresolved_refs.contains(&ref_id) {
                                edges.push(ExtractedEdge {
                                    source: file_id.to_string(),
                                    target: ref_id.clone(),
                                    relation: "calls".into(),
                                    weight: 0.4,
                                });
                                unresolved_refs.push(ref_id);
                            }
                        }
                    }
                }

                // Type references in fn signatures and type annotations
                for caps in self.re_fn_sig_types.captures_iter(line) {
                    if let Some(type_match) = caps.get(1) {
                        let type_name = type_match.as_str();
                        // Skip common std types and primitives
                        if !matches!(
                            type_name,
                            "Self"
                                | "String"
                                | "Vec"
                                | "Option"
                                | "Result"
                                | "Box"
                                | "Arc"
                                | "Rc"
                                | "HashMap"
                                | "HashSet"
                                | "BTreeMap"
                                | "Some"
                                | "None"
                                | "Ok"
                                | "Err"
                                | "Default"
                                | "Debug"
                                | "Clone"
                                | "Send"
                                | "Sync"
                                | "Sized"
                                | "Copy"
                                | "Display"
                                | "From"
                                | "Into"
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

            brace_depth += open_count - close_count;
        }

        Ok(ExtractionResult {
            nodes,
            edges,
            unresolved_refs,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rs"]
    }
}
