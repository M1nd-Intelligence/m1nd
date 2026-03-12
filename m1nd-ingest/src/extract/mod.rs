// === crates/m1nd-ingest/src/extract/mod.rs ===

use m1nd_core::types::NodeType;
use m1nd_core::error::M1ndResult;

pub mod python;
pub mod typescript;
pub mod rust_lang;
pub mod go;
pub mod java;
pub mod generic;

// ---------------------------------------------------------------------------
// Comment/string stripping — shared preprocessing for all extractors
// FM-ING-010: strip comments and string literals before regex extraction
// so that e.g. `"fn main()"` in a string literal is not extracted as a function.
// ---------------------------------------------------------------------------

/// Language-specific comment syntax for the pre-processor.
#[derive(Clone, Copy)]
pub struct CommentSyntax {
    /// Single-line comment prefix (e.g., "//", "#", "--").
    pub line_comment: &'static str,
    /// Block comment open (e.g., "/*"). Empty string means none.
    pub block_open: &'static str,
    /// Block comment close (e.g., "*/"). Empty string means none.
    pub block_close: &'static str,
    /// Triple-quote doc comment (e.g., `"""` for Python). Empty string means none.
    pub triple_quote: &'static str,
}

impl CommentSyntax {
    pub const RUST: Self = Self {
        line_comment: "//",
        block_open: "/*",
        block_close: "*/",
        triple_quote: "",
    };
    pub const PYTHON: Self = Self {
        line_comment: "#",
        block_open: "",
        block_close: "",
        triple_quote: "\"\"\"",
    };
    pub const C_STYLE: Self = Self {
        line_comment: "//",
        block_open: "/*",
        block_close: "*/",
        triple_quote: "",
    };
    pub const GO: Self = Self {
        line_comment: "//",
        block_open: "/*",
        block_close: "*/",
        triple_quote: "",
    };
    pub const GENERIC: Self = Self {
        line_comment: "#",
        block_open: "",
        block_close: "",
        triple_quote: "",
    };
}

/// Strips comments and string literals from source text, line by line.
/// Returns a Vec of cleaned lines (one per input line).
/// Block comment / triple-quote state is tracked across lines.
///
/// Import lines (e.g., `import "fmt"`, `from 'react'`, `use crate::foo`)
/// have their string content preserved so that module names are not lost.
/// Only comments are stripped on import lines.
pub fn strip_comments_and_strings(text: &str, syntax: CommentSyntax) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_block_comment = false;
    let mut in_triple_quote = false;

    for line in text.lines() {
        // If we are NOT inside a block comment / triple-quote **and** the
        // line looks like an import statement, preserve string content —
        // only strip comments.
        let preserve_strings = !in_block_comment
            && !in_triple_quote
            && is_import_line(line);

        let cleaned = strip_line(
            line,
            &syntax,
            &mut in_block_comment,
            &mut in_triple_quote,
            preserve_strings,
        );
        result.push(cleaned);
    }
    result
}

/// Returns true if `line` looks like an import/use statement in any
/// supported language, meaning string literals on this line contain
/// module names that must not be stripped.
fn is_import_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    // Go / TS / JS / Java / Python: `import ...`
    if trimmed.starts_with("import ") || trimmed.starts_with("import\t") {
        return true;
    }
    // Python: `from foo import bar`
    if trimmed.starts_with("from ") || trimmed.starts_with("from\t") {
        return true;
    }
    // Rust: `use crate::...` or `pub use ...`
    if trimmed.starts_with("use ") || trimmed.starts_with("use\t") {
        return true;
    }
    if trimmed.starts_with("pub use ") || trimmed.starts_with("pub use\t") {
        return true;
    }
    // Go grouped import: line inside `import ( ... )` block — these are
    // typically just `"package/path"`, detected by leading quote after
    // optional whitespace.
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        // Could be inside an import block; preserve conservatively.
        // Callers that are NOT in an import block still benefit because
        // a bare string-only line has no function/class defs to confuse.
        return true;
    }
    false
}

/// Strip a single line, mutating block-comment/triple-quote tracking state.
/// When `preserve_strings` is true, string literal content is kept intact
/// (only comments are stripped). This is used for import lines where module
/// names live inside quotes.
fn strip_line(
    line: &str,
    syntax: &CommentSyntax,
    in_block_comment: &mut bool,
    in_triple_quote: &mut bool,
    preserve_strings: bool,
) -> String {
    let mut out = String::with_capacity(line.len());
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // --- Inside a block comment: scan for close ---
        if *in_block_comment {
            if !syntax.block_close.is_empty() {
                let close_chars: Vec<char> = syntax.block_close.chars().collect();
                if i + close_chars.len() <= len
                    && chars[i..i + close_chars.len()] == close_chars[..]
                {
                    *in_block_comment = false;
                    i += close_chars.len();
                    continue;
                }
            }
            i += 1;
            continue;
        }

        // --- Inside a triple-quote string: scan for closing triple-quote ---
        if *in_triple_quote {
            if !syntax.triple_quote.is_empty() {
                let tq_chars: Vec<char> = syntax.triple_quote.chars().collect();
                if i + tq_chars.len() <= len
                    && chars[i..i + tq_chars.len()] == tq_chars[..]
                {
                    *in_triple_quote = false;
                    i += tq_chars.len();
                    continue;
                }
            }
            i += 1;
            continue;
        }

        // --- Check for triple-quote open ---
        if !syntax.triple_quote.is_empty() {
            let tq_chars: Vec<char> = syntax.triple_quote.chars().collect();
            if i + tq_chars.len() <= len
                && chars[i..i + tq_chars.len()] == tq_chars[..]
            {
                *in_triple_quote = true;
                i += tq_chars.len();
                continue;
            }
        }

        // --- Check for block comment open ---
        if !syntax.block_open.is_empty() {
            let bo_chars: Vec<char> = syntax.block_open.chars().collect();
            if i + bo_chars.len() <= len
                && chars[i..i + bo_chars.len()] == bo_chars[..]
            {
                *in_block_comment = true;
                i += bo_chars.len();
                continue;
            }
        }

        // --- Check for line comment ---
        if !syntax.line_comment.is_empty() {
            let lc_chars: Vec<char> = syntax.line_comment.chars().collect();
            if i + lc_chars.len() <= len
                && chars[i..i + lc_chars.len()] == lc_chars[..]
            {
                // Rest of line is comment; stop processing this line
                break;
            }
        }

        // --- Check for string literals: "..." or '...' ---
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            out.push(quote); // keep the quote delimiters
            i += 1;
            // Skip content until matching close quote (handle escapes)
            while i < len {
                if chars[i] == '\\' {
                    if preserve_strings {
                        out.push(chars[i]);
                        if i + 1 < len {
                            out.push(chars[i + 1]);
                        }
                    }
                    i += 2; // skip escaped char
                    continue;
                }
                if chars[i] == quote {
                    out.push(quote);
                    i += 1;
                    break;
                }
                if preserve_strings {
                    out.push(chars[i]);
                }
                // Otherwise: strip string content (original behavior)
                i += 1;
            }
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

// ---------------------------------------------------------------------------
// ExtractedNode / ExtractedEdge — extraction output
// Replaces: ingest.py per-extractor output tuples
// ---------------------------------------------------------------------------

/// A node extracted from source code.
#[derive(Clone, Debug)]
pub struct ExtractedNode {
    /// Unique ID within the file (e.g., "file::src/main.rs::fn::main").
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Node type (function, class, struct, etc.).
    pub node_type: NodeType,
    /// Tags (e.g., ["async", "public", "test"]).
    pub tags: Vec<String>,
    /// Line number in source file.
    pub line: u32,
    /// End line number.
    pub end_line: u32,
}

/// An edge extracted from source code.
#[derive(Clone, Debug)]
pub struct ExtractedEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID (may be unresolved reference).
    pub target: String,
    /// Relation type (e.g., "contains", "calls", "imports", "ref::").
    pub relation: String,
    /// Edge weight (default 1.0).
    pub weight: f32,
}

/// Result of extracting a single file.
#[derive(Clone, Debug)]
pub struct ExtractionResult {
    pub nodes: Vec<ExtractedNode>,
    pub edges: Vec<ExtractedEdge>,
    /// Unresolved references (target IDs that need resolution).
    pub unresolved_refs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Extractor — trait for language-specific extraction
// Replaces: ingest.py PythonExtractor, TypeScriptExtractor, etc.
// ---------------------------------------------------------------------------

/// Language-specific code structure extractor.
/// All impls use tree-sitter (not regex) for correct extraction.
/// FM-ING-009: tree-sitter captures indented defs.
/// FM-ING-010: tree-sitter AST excludes strings/comments.
pub trait Extractor: Send + Sync {
    /// Extract nodes and edges from file content.
    /// `file_id` is the canonical file identifier (e.g., "file::src/main.rs").
    fn extract(
        &self,
        content: &[u8],
        file_id: &str,
    ) -> M1ndResult<ExtractionResult>;

    /// File extensions this extractor handles.
    fn extensions(&self) -> &[&str];
}
