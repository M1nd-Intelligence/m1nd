// === crates/m1nd-ingest/src/extract/tree_sitter_ext.rs ===
//
// Generic tree-sitter-based extractor for m1nd.
// One extractor struct handles ALL tree-sitter-backed languages by taking a
// LanguageConfig that maps language-specific node kinds to m1nd NodeTypes.

use super::{ExtractedEdge, ExtractedNode, ExtractionResult, Extractor};
use m1nd_core::error::M1ndResult;
use m1nd_core::types::NodeType;
use tree_sitter::{Language, Node, Parser};

// ---------------------------------------------------------------------------
// LanguageConfig — per-language node kind → m1nd NodeType mapping
// ---------------------------------------------------------------------------

/// Maps tree-sitter node kind strings to m1nd semantic concepts.
/// Each language grammar uses different kind names for the same concepts
/// (e.g., C uses "function_definition", Ruby uses "method").
#[derive(Clone, Debug)]
pub struct LanguageConfig {
    /// Language tag for node metadata (e.g., "c", "ruby", "kotlin").
    pub lang_tag: &'static str,
    /// File extensions handled by this config.
    pub extensions: &'static [&'static str],
    /// AST node kinds that represent function/method definitions.
    pub function_kinds: &'static [&'static str],
    /// AST node kinds that represent class definitions.
    pub class_kinds: &'static [&'static str],
    /// AST node kinds that represent struct definitions.
    pub struct_kinds: &'static [&'static str],
    /// AST node kinds that represent enum definitions.
    pub enum_kinds: &'static [&'static str],
    /// AST node kinds that represent type/interface/trait definitions.
    pub type_kinds: &'static [&'static str],
    /// AST node kinds that represent module/namespace definitions.
    pub module_kinds: &'static [&'static str],
    /// AST node kinds that represent import/require/use statements.
    pub import_kinds: &'static [&'static str],
    /// The field name used for the "name" of definitions (usually "name").
    pub name_field: &'static str,
    /// Alternative field names to try if `name_field` yields nothing.
    pub alt_name_fields: &'static [&'static str],
    /// Node kinds whose first named child is the name identifier
    /// (fallback when field access fails).
    pub name_from_first_child: bool,
}

// ---------------------------------------------------------------------------
// TreeSitterExtractor — implements Extractor trait
// ---------------------------------------------------------------------------

/// Tree-sitter-based code extractor. Parses source via tree-sitter and walks
/// the AST to extract functions, classes, structs, enums, imports, and
/// containment edges.
pub struct TreeSitterExtractor {
    language: Language,
    config: LanguageConfig,
}

impl TreeSitterExtractor {
    /// Create a new extractor for a given tree-sitter Language + config.
    pub fn new(language: Language, config: LanguageConfig) -> Self {
        Self { language, config }
    }

    /// Try to extract the name of a definition node.
    /// Strategy:
    ///   1. field("name") on the node itself
    ///   2. alt_name_fields on the node itself
    ///   3. Recursive drill: for nodes like C's function_definition where the
    ///      name is buried inside a declarator child, drill into children that
    ///      are themselves declarators and try name/identifier extraction on them
    ///   4. First named child that looks like an identifier
    fn extract_name<'a>(&self, node: Node<'a>, source: &'a [u8]) -> Option<String> {
        self.extract_name_inner(node, source, 0)
    }

    fn extract_name_inner<'a>(
        &self,
        node: Node<'a>,
        source: &'a [u8],
        depth: usize,
    ) -> Option<String> {
        if depth > 4 {
            return None; // Prevent infinite recursion
        }

        // Try primary name field
        if let Some(name_node) = node.child_by_field_name(self.config.name_field) {
            // If the name node is a simple identifier, use its text
            if name_node.kind().contains("identifier")
                || name_node.kind() == "name"
                || name_node.kind() == "constant"
                || name_node.kind() == "simple_identifier"
                || name_node.named_child_count() == 0
            {
                let text = name_node.utf8_text(source).ok()?;
                if !text.is_empty() {
                    return Some(text.to_string());
                }
            }
            // If name node is compound (like a scoped identifier), try to
            // extract a simpler name from it
            if let Some(name) = self.extract_name_inner(name_node, source, depth + 1) {
                return Some(name);
            }
        }

        // Try alternative name fields
        for field in self.config.alt_name_fields {
            if let Some(child) = node.child_by_field_name(field) {
                // Drill into declarator nodes (C/C++ pattern:
                // function_definition → declarator: function_declarator → name: identifier)
                if child.kind().contains("declarator") {
                    if let Some(name) = self.extract_name_inner(child, source, depth + 1) {
                        return Some(name);
                    }
                }
                // Simple identifier child
                if child.kind().contains("identifier")
                    || child.kind() == "name"
                    || child.named_child_count() == 0
                {
                    let text = child.utf8_text(source).ok()?;
                    if !text.is_empty() {
                        return Some(text.to_string());
                    }
                }
            }
        }

        // Fallback: scan named children for identifiers, declarators, or
        // name-bearing sub-nodes. When name_from_first_child is true, we
        // recursively drill into children to find the actual name. This
        // handles grammars where the name is nested (e.g., OCaml's
        // value_definition > let_binding > value_name, or TOML's
        // table > bare_key, or SQL's create_table > object_reference > identifier).
        if self.config.name_from_first_child {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                let kind = child.kind();
                // Direct identifier — return immediately
                if kind.contains("identifier")
                    || kind == "name"
                    || kind == "constant"
                    || kind == "simple_identifier"
                    || kind == "bare_key"
                    || kind == "value_name"
                    || kind == "type_constructor"
                    || kind == "constructor_name"
                {
                    let text = child.utf8_text(source).ok()?;
                    if !text.is_empty() {
                        return Some(text.to_string());
                    }
                }
                // Skip keyword nodes (SQL: keyword_create, keyword_table, etc.)
                if kind.starts_with("keyword_") {
                    continue;
                }
                // Compound child — drill down recursively
                if kind.contains("declarator")
                    || kind.contains("binding")
                    || kind.contains("reference")
                    || kind.contains("definition")
                    || kind.contains("_name")
                    || kind.contains("spec")
                {
                    if let Some(name) = self.extract_name_inner(child, source, depth + 1) {
                        return Some(name);
                    }
                }
            }
        }

        None
    }

    /// Extract the import target string from an import node.
    fn extract_import_target<'a>(&self, node: Node<'a>, source: &'a [u8]) -> Vec<String> {
        let mut targets = Vec::new();

        // Try common patterns for import targets
        let mut cursor = node.walk();
        self.collect_import_identifiers(node, source, &mut targets, &mut cursor);

        // Fallback: use the full text of the import node, cleaned up
        if targets.is_empty() {
            if let Ok(text) = node.utf8_text(source) {
                // Strip the keyword and extract meaningful identifiers
                let cleaned = text.trim().replace('\n', " ");
                // Extract quoted strings (common in many languages)
                for part in cleaned.split('"').enumerate() {
                    if part.0 % 2 == 1 && !part.1.is_empty() {
                        targets.push(part.1.to_string());
                    }
                }
                // Extract single-quoted strings
                if targets.is_empty() {
                    for part in cleaned.split('\'').enumerate() {
                        if part.0 % 2 == 1 && !part.1.is_empty() {
                            targets.push(part.1.to_string());
                        }
                    }
                }
                // If still nothing, try to extract dotted/scoped names
                if targets.is_empty() {
                    for word in cleaned.split_whitespace() {
                        // Skip keywords
                        if matches!(
                            word,
                            "import"
                                | "from"
                                | "require"
                                | "use"
                                | "include"
                                | "using"
                                | "extern"
                                | "module"
                                | "package"
                                | "open"
                                | "static"
                                | "as"
                                | "*"
                                | "{"
                                | "}"
                                | "("
                                | ")"
                                | ";"
                        ) {
                            continue;
                        }
                        if word.contains('.') || word.contains("::") || word.len() > 1 {
                            targets.push(
                                word.trim_matches(|c: char| {
                                    !c.is_alphanumeric() && c != '.' && c != ':' && c != '_'
                                })
                                .to_string(),
                            );
                        }
                    }
                }
            }
        }

        targets.retain(|t| !t.is_empty());
        targets
    }

    /// Recursively collect identifier strings from import-related nodes.
    fn collect_import_identifiers<'a>(
        &self,
        node: Node<'a>,
        source: &'a [u8],
        targets: &mut Vec<String>,
        _cursor: &mut tree_sitter::TreeCursor<'a>,
    ) {
        let kind = node.kind();

        // Scoped identifiers, qualified names, etc.
        if kind.contains("identifier")
            || kind == "constant"
            || kind == "scope_resolution"
            || kind == "scoped_identifier"
            || kind == "qualified_name"
            || kind == "dotted_name"
            || kind == "member_expression"
        {
            if let Ok(text) = node.utf8_text(source) {
                let text = text.trim();
                if !text.is_empty()
                    && !matches!(
                        text,
                        "import"
                            | "from"
                            | "require"
                            | "use"
                            | "include"
                            | "using"
                            | "extern"
                            | "module"
                    )
                {
                    targets.push(text.to_string());
                    return; // Don't recurse into children of a qualified name
                }
            }
        }

        // String literals (for require("..."), include "...", etc.)
        if kind == "string_literal"
            || kind == "string"
            || kind == "string_content"
            || kind == "interpreted_string_literal"
        {
            if let Ok(text) = node.utf8_text(source) {
                let trimmed = text.trim_matches(|c: char| c == '"' || c == '\'' || c == '`');
                if !trimmed.is_empty() {
                    targets.push(trimmed.to_string());
                    return;
                }
            }
        }

        // Recurse into children
        let mut child_cursor = node.walk();
        for child in node.named_children(&mut child_cursor) {
            self.collect_import_identifiers(child, source, targets, &mut node.walk());
        }
    }

    /// Walk the AST and extract nodes + edges.
    fn walk_ast(
        &self,
        root: Node<'_>,
        source: &[u8],
        file_id: &str,
        nodes: &mut Vec<ExtractedNode>,
        edges: &mut Vec<ExtractedEdge>,
        unresolved_refs: &mut Vec<String>,
    ) {
        // Use a stack to avoid deep recursion on large files
        let mut stack: Vec<(Node<'_>, Option<String>)> = vec![(root, None)];

        while let Some((node, parent_id)) = stack.pop() {
            let kind = node.kind();
            let start_line = node.start_position().row as u32 + 1;
            let end_line = node.end_position().row as u32 + 1;

            // Check if this node is a definition we care about
            let (node_type, id_prefix) = if self.config.function_kinds.contains(&kind) {
                (Some(NodeType::Function), "fn")
            } else if self.config.class_kinds.contains(&kind) {
                (Some(NodeType::Class), "class")
            } else if self.config.struct_kinds.contains(&kind) {
                (Some(NodeType::Struct), "struct")
            } else if self.config.enum_kinds.contains(&kind) {
                (Some(NodeType::Enum), "enum")
            } else if self.config.type_kinds.contains(&kind) {
                (Some(NodeType::Type), "type")
            } else if self.config.module_kinds.contains(&kind) {
                (Some(NodeType::Module), "module")
            } else {
                (None, "")
            };

            // Handle import statements
            if self.config.import_kinds.contains(&kind) {
                let targets = self.extract_import_target(node, source);
                for target in targets {
                    let ref_id = format!("ref::{}", target);
                    edges.push(ExtractedEdge {
                        source: file_id.to_string(),
                        target: ref_id.clone(),
                        relation: "imports".into(),
                        weight: 0.5,
                    });
                    if !unresolved_refs.contains(&ref_id) {
                        unresolved_refs.push(ref_id);
                    }
                }
            }

            // Handle definitions
            if let Some(nt) = node_type {
                if let Some(name) = self.extract_name(node, source) {
                    let node_id = format!("{}::{}::{}", file_id, id_prefix, name);

                    nodes.push(ExtractedNode {
                        id: node_id.clone(),
                        label: name,
                        node_type: nt,
                        tags: vec![self.config.lang_tag.into()],
                        line: start_line,
                        end_line,
                    });

                    // Containment edge: parent → this node
                    let container = parent_id.as_deref().unwrap_or(file_id);
                    edges.push(ExtractedEdge {
                        source: container.to_string(),
                        target: node_id.clone(),
                        relation: "contains".into(),
                        weight: 1.0,
                    });

                    // Recurse into children with this node as parent
                    let mut cursor = node.walk();
                    for child in node.named_children(&mut cursor) {
                        stack.push((child, Some(node_id.clone())));
                    }
                    continue; // Skip the default child traversal below
                }
            }

            // Default: recurse into children with same parent
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                stack.push((child, parent_id.clone()));
            }
        }
    }
}

impl Extractor for TreeSitterExtractor {
    fn extract(&self, content: &[u8], file_id: &str) -> M1ndResult<ExtractionResult> {
        let mut parser = Parser::new();
        parser.set_language(&self.language).map_err(|e| {
            m1nd_core::error::M1ndError::IngestError(format!(
                "Failed to set tree-sitter language for {}: {}",
                self.config.lang_tag, e
            ))
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            m1nd_core::error::M1ndError::IngestError(format!(
                "tree-sitter parse returned None for {}",
                file_id
            ))
        })?;

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut unresolved_refs = Vec::new();

        // File node (every extractor must emit one)
        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        let line_count = content.iter().filter(|&&b| b == b'\n').count() as u32;
        nodes.push(ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: NodeType::File,
            tags: vec![self.config.lang_tag.into()],
            line: 1,
            end_line: line_count.max(1),
        });

        // Walk AST
        self.walk_ast(
            tree.root_node(),
            content,
            file_id,
            &mut nodes,
            &mut edges,
            &mut unresolved_refs,
        );

        Ok(ExtractionResult {
            nodes,
            edges,
            unresolved_refs,
        })
    }

    fn extensions(&self) -> &[&str] {
        self.config.extensions
    }
}

// ---------------------------------------------------------------------------
// Language configs for all Tier 1 languages
// ---------------------------------------------------------------------------

/// C language config.
pub fn c_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "c",
        extensions: &["c", "h"],
        function_kinds: &["function_definition"],
        class_kinds: &[],
        struct_kinds: &["struct_specifier"],
        enum_kinds: &["enum_specifier"],
        type_kinds: &["type_definition"],
        module_kinds: &[],
        import_kinds: &["preproc_include"],
        name_field: "name",
        alt_name_fields: &["declarator"],
        name_from_first_child: true,
    }
}

/// C++ language config.
pub fn cpp_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "cpp",
        extensions: &["cpp", "cxx", "cc", "hpp", "hxx", "hh"],
        function_kinds: &["function_definition"],
        class_kinds: &["class_specifier"],
        struct_kinds: &["struct_specifier"],
        enum_kinds: &["enum_specifier"],
        type_kinds: &["type_definition", "alias_declaration"],
        module_kinds: &["namespace_definition"],
        import_kinds: &["preproc_include", "using_declaration"],
        name_field: "name",
        alt_name_fields: &["declarator"],
        name_from_first_child: true,
    }
}

/// C# language config.
pub fn csharp_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "csharp",
        extensions: &["cs"],
        function_kinds: &[
            "method_declaration",
            "constructor_declaration",
            "local_function_statement",
        ],
        class_kinds: &["class_declaration", "record_declaration"],
        struct_kinds: &["struct_declaration"],
        enum_kinds: &["enum_declaration"],
        type_kinds: &["interface_declaration", "delegate_declaration"],
        module_kinds: &["namespace_declaration"],
        import_kinds: &["using_directive"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Ruby language config.
pub fn ruby_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "ruby",
        extensions: &["rb", "rake", "gemspec"],
        function_kinds: &["method", "singleton_method"],
        class_kinds: &["class"],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &["module"],
        import_kinds: &["call"], // require/require_relative are method calls
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// PHP language config.
pub fn php_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "php",
        extensions: &["php"],
        function_kinds: &["function_definition", "method_declaration"],
        class_kinds: &["class_declaration"],
        struct_kinds: &[],
        enum_kinds: &["enum_declaration"],
        type_kinds: &["interface_declaration", "trait_declaration"],
        module_kinds: &["namespace_definition"],
        import_kinds: &["namespace_use_declaration"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Swift language config.
/// Note: Swift's tree-sitter grammar maps both `class` and `struct` to
/// `class_declaration` (struct has a `struct` keyword child). We map
/// class_declaration to Class. This is semantically correct for the m1nd
/// graph since both are type definitions with containment.
pub fn swift_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "swift",
        extensions: &["swift"],
        function_kinds: &["function_declaration", "init_declaration"],
        class_kinds: &["class_declaration"],
        struct_kinds: &[], // Swift grammar uses class_declaration for both
        enum_kinds: &["enum_declaration"],
        type_kinds: &["protocol_declaration", "typealias_declaration"],
        module_kinds: &[],
        import_kinds: &["import_declaration"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Kotlin language config.
pub fn kotlin_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "kotlin",
        extensions: &["kt", "kts"],
        function_kinds: &["function_declaration"],
        class_kinds: &["class_declaration"],
        struct_kinds: &[],
        enum_kinds: &["enum_class_body"], // Kotlin enums are class_declaration with enum modifier
        type_kinds: &["interface_declaration", "type_alias"],
        module_kinds: &["package_header"],
        import_kinds: &["import_header"],
        name_field: "name",
        alt_name_fields: &["simple_identifier"],
        name_from_first_child: true,
    }
}

/// Scala language config.
pub fn scala_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "scala",
        extensions: &["scala", "sc"],
        function_kinds: &["function_definition"],
        class_kinds: &["class_definition"],
        struct_kinds: &[],
        enum_kinds: &["enum_definition"],
        type_kinds: &["trait_definition", "type_definition"],
        module_kinds: &["object_definition", "package_clause"],
        import_kinds: &["import_declaration"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Bash/Shell language config.
pub fn bash_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "bash",
        extensions: &["sh", "bash", "zsh"],
        function_kinds: &["function_definition"],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["command"], // source/. commands are regular commands
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Lua language config.
pub fn lua_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "lua",
        extensions: &["lua"],
        function_kinds: &["function_declaration", "local_function_declaration"],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["function_call"], // require() calls
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// R language config.
pub fn r_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "r",
        extensions: &["r", "R", "Rmd"],
        // R doesn't have function_definition as a node kind;
        // functions are assigned via `name <- function(...)`.
        // We pick up left_assignment where the RHS is a function_definition.
        function_kinds: &["function_definition"],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["call"], // library() and require() are function calls
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// HTML language config (structural extraction: tags as nodes is low value,
/// but we extract <script>/<link>/<style> as imports).
pub fn html_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "html",
        extensions: &["html", "htm"],
        function_kinds: &[],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["script_element", "style_element"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: false,
    }
}

/// CSS language config.
pub fn css_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "css",
        extensions: &["css"],
        function_kinds: &[],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["import_statement"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: false,
    }
}

/// JSON language config (structural only — top-level keys as nodes).
pub fn json_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "json",
        extensions: &["json"],
        function_kinds: &[],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &[],
        name_field: "key",
        alt_name_fields: &[],
        name_from_first_child: false,
    }
}

// ---------------------------------------------------------------------------
// Factory functions — create configured extractors
// ---------------------------------------------------------------------------

/// Create a C extractor.
pub fn c_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_c::LANGUAGE.into(), c_config())
}

/// Create a C++ extractor.
pub fn cpp_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_cpp::LANGUAGE.into(), cpp_config())
}

/// Create a C# extractor.
pub fn csharp_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_c_sharp::LANGUAGE.into(), csharp_config())
}

/// Create a Ruby extractor.
pub fn ruby_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_ruby::LANGUAGE.into(), ruby_config())
}

/// Create a PHP extractor.
pub fn php_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_php::LANGUAGE_PHP.into(), php_config())
}

/// Create a Swift extractor.
pub fn swift_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_swift::LANGUAGE.into(), swift_config())
}

/// Create a Kotlin extractor.
pub fn kotlin_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_kotlin_ng::LANGUAGE.into(), kotlin_config())
}

/// Create a Scala extractor.
pub fn scala_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_scala::LANGUAGE.into(), scala_config())
}

/// Create a Bash extractor.
pub fn bash_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_bash::LANGUAGE.into(), bash_config())
}

/// Create a Lua extractor.
pub fn lua_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_lua::LANGUAGE.into(), lua_config())
}

/// Create an R extractor.
pub fn r_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_r::LANGUAGE.into(), r_config())
}

/// Create an HTML extractor.
pub fn html_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_html::LANGUAGE.into(), html_config())
}

/// Create a CSS extractor.
pub fn css_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_css::LANGUAGE.into(), css_config())
}

/// Create a JSON extractor.
pub fn json_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_json::LANGUAGE.into(), json_config())
}

/// JavaScript language config.
pub fn javascript_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "javascript",
        extensions: &["js", "mjs", "cjs"],
        function_kinds: &[
            "function_declaration",
            "function_expression",
            "arrow_function",
            "method_definition",
            "generator_function_declaration",
            "generator_function",
        ],
        class_kinds: &["class_declaration", "class_expression"],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["import_statement", "call_expression"],
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: false,
    }
}

/// Create a JavaScript extractor.
pub fn javascript_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_javascript::LANGUAGE.into(), javascript_config())
}

// ---------------------------------------------------------------------------
// EmbeddedExtractor — parses host language (e.g. HTML) and routes embedded
// language blocks (e.g. <script>, <style>) to inner extractors.
// ---------------------------------------------------------------------------

/// Descriptor for an embedded language block found inside a host document.
struct EmbeddedBlock {
    /// Text content of the embedded block (without surrounding tags).
    content: String,
    /// Line offset in the original host file where this block starts (0-indexed).
    line_offset: u32,
    /// Language tag (e.g., "javascript", "css").
    lang: String,
}

/// Extracts embedded language blocks from a host language document (e.g. HTML).
///
/// Algorithm:
///   1. Parse host document with tree-sitter (e.g. tree_sitter_html).
///   2. Walk AST looking for `script_element` / `style_element` nodes.
///   3. For each block, extract raw_text child content.
///   4. Re-parse that content with the appropriate inner extractor.
///   5. Offset extracted node line numbers by the block's start line.
///   6. Merge all results into one ExtractionResult.
pub struct EmbeddedExtractor {
    host_language: tree_sitter::Language,
    host_config: LanguageConfig,
    /// (ast_node_kind, language_tag) pairs for embedded block detection.
    embedded_kinds: Vec<(&'static str, &'static str)>,
    /// Inner extractors keyed by language tag.
    inner_extractors: Vec<(String, Box<dyn super::Extractor>)>,
}

impl EmbeddedExtractor {
    /// Create an HTML embedded extractor that handles <script> and <style> blocks.
    pub fn html_embedded() -> Self {
        Self {
            host_language: tree_sitter_html::LANGUAGE.into(),
            host_config: html_config(),
            embedded_kinds: vec![("script_element", "javascript"), ("style_element", "css")],
            inner_extractors: vec![
                ("javascript".into(), Box::new(javascript_extractor())),
                ("css".into(), Box::new(css_extractor())),
            ],
        }
    }

    /// Find embedded blocks in the host AST.
    fn find_embedded_blocks(&self, source: &[u8]) -> Vec<EmbeddedBlock> {
        let mut parser = tree_sitter::Parser::new();
        if parser.set_language(&self.host_language).is_err() {
            return Vec::new();
        }
        let tree = match parser.parse(source, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut blocks = Vec::new();
        let mut stack = vec![tree.root_node()];

        while let Some(node) = stack.pop() {
            let kind = node.kind();

            // Check if this is an embedded block we care about
            let lang_opt = self
                .embedded_kinds
                .iter()
                .find(|(k, _)| *k == kind)
                .map(|(_, l)| *l);

            if let Some(lang_tag) = lang_opt {
                // For <script src="..."> (external), skip — no raw_text child
                // For <script type="..."> (inline), extract raw_text
                let content = self.extract_block_text(node, source, lang_tag);
                if let Some((text, line_offset)) = content {
                    if !text.trim().is_empty() {
                        blocks.push(EmbeddedBlock {
                            content: text,
                            line_offset,
                            lang: lang_tag.to_string(),
                        });
                    }
                }
                // Don't recurse into embedded blocks
                continue;
            }

            // Recurse into children
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                stack.push(child);
            }
        }

        blocks
    }

    /// Extract text from an embedded block node.
    /// Returns (content, line_offset_of_content_start) or None if external/empty.
    fn extract_block_text(
        &self,
        node: tree_sitter::Node<'_>,
        source: &[u8],
        _lang: &str,
    ) -> Option<(String, u32)> {
        // tree-sitter-html represents inline script/style content as a
        // `raw_text` child node inside script_element/style_element.
        // Check for `src` attribute (external script) — skip those.
        // We detect external scripts by checking if there's NO raw_text child.
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "raw_text" {
                let text = child.utf8_text(source).ok()?.to_string();
                let line_offset = child.start_position().row as u32;
                return Some((text, line_offset));
            }
        }
        None
    }
}

impl super::Extractor for EmbeddedExtractor {
    fn extract(
        &self,
        content: &[u8],
        file_id: &str,
    ) -> m1nd_core::error::M1ndResult<super::ExtractionResult> {
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_refs = Vec::new();

        // File node (always present)
        let file_label = file_id.rsplit("::").next().unwrap_or(file_id);
        let line_count = content.iter().filter(|&&b| b == b'\n').count() as u32;
        all_nodes.push(super::ExtractedNode {
            id: file_id.to_string(),
            label: file_label.to_string(),
            node_type: m1nd_core::types::NodeType::File,
            tags: vec!["html".into()],
            line: 1,
            end_line: line_count.max(1),
        });

        // Find all embedded blocks
        let blocks = self.find_embedded_blocks(content);

        for block in &blocks {
            // Find the inner extractor for this language
            let extractor = self
                .inner_extractors
                .iter()
                .find(|(lang, _)| lang == &block.lang)
                .map(|(_, ext)| ext.as_ref());

            let extractor = match extractor {
                Some(e) => e,
                None => continue,
            };

            // Parse the embedded block content
            let block_bytes = block.content.as_bytes();
            let inner_result = match extractor.extract(block_bytes, file_id) {
                Ok(r) => r,
                Err(e) => {
                    // Graceful degradation: log and skip this block
                    eprintln!(
                        "[m1nd] EmbeddedExtractor: failed to parse {} block in {}: {}",
                        block.lang, file_id, e
                    );
                    continue;
                }
            };

            // Merge results, offsetting line numbers and skipping duplicate file node
            for node in inner_result.nodes {
                if node.node_type == m1nd_core::types::NodeType::File {
                    // Skip the inner file node — we already have one
                    continue;
                }
                // Offset line numbers by block start
                all_nodes.push(super::ExtractedNode {
                    line: node.line + block.line_offset,
                    end_line: node.end_line + block.line_offset,
                    ..node
                });
            }

            for edge in inner_result.edges {
                all_edges.push(edge);
            }
            for r in inner_result.unresolved_refs {
                if !all_refs.contains(&r) {
                    all_refs.push(r);
                }
            }
        }

        // Add containment edges from file to all top-level nodes
        for node in &all_nodes {
            if node.node_type != m1nd_core::types::NodeType::File {
                // Only add containment edge if one doesn't already exist
                let already_has_edge = all_edges
                    .iter()
                    .any(|e| e.target == node.id && e.relation == "contains");
                if !already_has_edge {
                    all_edges.push(super::ExtractedEdge {
                        source: file_id.to_string(),
                        target: node.id.clone(),
                        relation: "contains".into(),
                        weight: 1.0,
                    });
                }
            }
        }

        Ok(super::ExtractionResult {
            nodes: all_nodes,
            edges: all_edges,
            unresolved_refs: all_refs,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["html", "htm"]
    }
}

// ===========================================================================
// Tier 2 language configs + factory functions (8 languages)
// ===========================================================================
// All Tier 2 grammar crates use the new tree-sitter-language API (LanguageFn).
// No old-API crates allowed — they pull in separate tree-sitter C runtimes
// whose symbols collide with the main tree-sitter 0.24 runtime.
//
// Dropped: Dockerfile (tree-sitter-dockerfile 0.2 depends on tree-sitter 0.20)
// Replaced: tree-sitter-toml -> tree-sitter-toml-ng (same grammar, new API)
// Replaced: tree-sitter-sql -> tree-sitter-sequel (same grammar, new API)

/// Elixir language config.
/// Elixir's grammar represents def/defmodule as `call` nodes — the extractor
/// picks up function_signature patterns from the grammar's dedicated node kinds.
#[cfg(feature = "tier2")]
pub fn elixir_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "elixir",
        extensions: &["ex", "exs"],
        // Elixir uses `call` for everything, but the grammar has some
        // dedicated node types we can match on
        function_kinds: &["call"],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &["call"],
        import_kinds: &["call"],
        name_field: "target",
        alt_name_fields: &["name"],
        name_from_first_child: true,
    }
}

/// Dart language config.
/// AST: class_declaration > identifier, function_signature > identifier.
#[cfg(feature = "tier2")]
pub fn dart_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "dart",
        extensions: &["dart"],
        function_kinds: &["function_signature", "method_signature"],
        class_kinds: &["class_declaration"],
        struct_kinds: &[],
        enum_kinds: &["enum_declaration"],
        type_kinds: &["mixin_declaration"],
        module_kinds: &[],
        import_kinds: &["import_or_export"],
        name_field: "name",
        alt_name_fields: &[],
        // class_declaration > identifier (first named child after skipping type_identifier)
        name_from_first_child: true,
    }
}

/// Zig language config.
#[cfg(feature = "tier2")]
pub fn zig_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "zig",
        extensions: &["zig"],
        function_kinds: &["function_declaration"],
        class_kinds: &[],
        struct_kinds: &["container_declaration"],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &["builtin_function"], // @import("...")
        name_field: "name",
        alt_name_fields: &[],
        name_from_first_child: true,
    }
}

/// Haskell language config.
#[cfg(feature = "tier2")]
pub fn haskell_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "haskell",
        extensions: &["hs", "lhs"],
        function_kinds: &["function", "bind", "signature"],
        class_kinds: &["class_declaration"],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &["data_type", "newtype", "type_alias"],
        module_kinds: &["header"],
        import_kinds: &["import"],
        name_field: "name",
        alt_name_fields: &["module"],
        name_from_first_child: true,
    }
}

/// OCaml language config.
/// AST: value_definition > let_binding > value_name. The name is deep in children.
/// type_definition > type_binding > type_constructor.
#[cfg(feature = "tier2")]
pub fn ocaml_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "ocaml",
        extensions: &["ml", "mli"],
        function_kinds: &["value_definition"],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &["type_definition"],
        module_kinds: &["module_definition"],
        import_kinds: &["open_module"],
        name_field: "name",
        alt_name_fields: &["pattern"],
        // value_definition > let_binding > value_name — drill into children
        name_from_first_child: true,
    }
}

/// TOML language config (structural: tables as nodes).
/// toml-ng AST: table -> bare_key (first named child is the section name).
#[cfg(feature = "tier2")]
pub fn toml_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "toml",
        extensions: &["toml"],
        function_kinds: &[],
        class_kinds: &[],
        struct_kinds: &["table", "table_array_element"],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &[],
        name_field: "name",
        alt_name_fields: &[],
        // table's first named child is bare_key with the section name
        name_from_first_child: true,
    }
}

/// YAML language config (structural: limited extraction).
/// YAML has no functions or imports — we primarily extract the file node
/// for connectivity in the graph.
#[cfg(feature = "tier2")]
pub fn yaml_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "yaml",
        extensions: &["yml", "yaml"],
        function_kinds: &[],
        class_kinds: &[],
        struct_kinds: &[],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &[],
        import_kinds: &[],
        name_field: "key",
        alt_name_fields: &[],
        name_from_first_child: false,
    }
}

/// SQL language config.
/// Uses tree-sitter-sequel grammar. AST: create_table -> object_reference -> identifier.
#[cfg(feature = "tier2")]
pub fn sql_config() -> LanguageConfig {
    LanguageConfig {
        lang_tag: "sql",
        extensions: &["sql"],
        function_kinds: &["create_function"],
        class_kinds: &[],
        struct_kinds: &["create_table", "create_view", "create_index"],
        enum_kinds: &[],
        type_kinds: &[],
        module_kinds: &["create_schema"],
        import_kinds: &[],
        name_field: "name",
        alt_name_fields: &[],
        // create_table's named children include object_reference which has identifier
        name_from_first_child: true,
    }
}

// NOTE: Dockerfile extractor dropped — tree-sitter-dockerfile 0.2.0 depends on
// tree-sitter 0.20.10 (old C runtime) which causes symbol collisions with the
// main tree-sitter 0.24 runtime. No new-API Dockerfile grammar crate exists.
// Dockerfile files fall back to GenericExtractor.

// ---------------------------------------------------------------------------
// Tier 2 factory functions
// ---------------------------------------------------------------------------

/// Create an Elixir extractor.
#[cfg(feature = "tier2")]
pub fn elixir_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_elixir::LANGUAGE.into(), elixir_config())
}

/// Create a Dart extractor.
#[cfg(feature = "tier2")]
pub fn dart_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_dart::LANGUAGE.into(), dart_config())
}

/// Create a Zig extractor.
#[cfg(feature = "tier2")]
pub fn zig_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_zig::LANGUAGE.into(), zig_config())
}

/// Create a Haskell extractor.
#[cfg(feature = "tier2")]
pub fn haskell_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_haskell::LANGUAGE.into(), haskell_config())
}

/// Create an OCaml extractor.
#[cfg(feature = "tier2")]
pub fn ocaml_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_ocaml::LANGUAGE_OCAML.into(), ocaml_config())
}

/// Create a TOML extractor.
/// Uses tree-sitter-toml-ng (new API, no C symbol collisions).
#[cfg(feature = "tier2")]
pub fn toml_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_toml_ng::LANGUAGE.into(), toml_config())
}

/// Create a YAML extractor.
#[cfg(feature = "tier2")]
pub fn yaml_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_yaml::LANGUAGE.into(), yaml_config())
}

/// Create a SQL extractor.
/// Uses tree-sitter-sequel (new API, no C symbol collisions).
#[cfg(feature = "tier2")]
pub fn sql_extractor() -> TreeSitterExtractor {
    TreeSitterExtractor::new(tree_sitter_sequel::LANGUAGE.into(), sql_config())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Debug helper: dump all AST node kinds for a given source + language.
    #[allow(dead_code)]
    fn dump_ast(lang: Language, src: &[u8]) -> String {
        let mut parser = Parser::new();
        parser.set_language(&lang).unwrap();
        let tree = parser.parse(src, None).unwrap();
        let mut out = String::new();
        dump_node(&tree.root_node(), src, &mut out, 0);
        out
    }

    #[allow(dead_code)]
    fn dump_node(node: &Node, src: &[u8], out: &mut String, depth: usize) {
        let indent = "  ".repeat(depth);
        let text = node.utf8_text(src).unwrap_or("?");
        let short_text = if text.len() > 60 { &text[..60] } else { text };
        out.push_str(&format!(
            "{}{} [{}..{}] {:?}\n",
            indent,
            node.kind(),
            node.start_position().row,
            node.end_position().row,
            short_text.replace('\n', "\\n")
        ));
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            dump_node(&child, src, out, depth + 1);
        }
    }

    #[test]
    fn c_extracts_function_and_struct() {
        let src = b"struct Point { int x; int y; };\nint add(int a, int b) { return a + b; }";
        let ext = c_extractor();
        let result = ext.extract(src, "file::test.c").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "add" && n.node_type == NodeType::Function),
            "Should extract C function. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Point" && n.node_type == NodeType::Struct),
            "Should extract C struct. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn c_extracts_enum() {
        let src = b"enum Color { RED, GREEN, BLUE };";
        let ext = c_extractor();
        let result = ext.extract(src, "file::test.c").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Color" && n.node_type == NodeType::Enum),
            "Should extract C enum. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn c_extracts_include() {
        let src = b"#include <stdio.h>\n#include \"myheader.h\"\nint main() { return 0; }";
        let ext = c_extractor();
        let result = ext.extract(src, "file::test.c").unwrap();
        let import_edges: Vec<_> = result
            .edges
            .iter()
            .filter(|e| e.relation == "imports")
            .collect();
        assert!(
            !import_edges.is_empty(),
            "Should have import edges for #include. Edges: {:?}",
            result.edges
        );
    }

    #[test]
    fn cpp_extracts_class_and_namespace() {
        let src = b"namespace myns {\nclass Widget {\npublic:\n    void draw();\n};\n}";
        let ext = cpp_extractor();
        let result = ext.extract(src, "file::test.cpp").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "myns" && n.node_type == NodeType::Module),
            "Should extract C++ namespace. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Widget" && n.node_type == NodeType::Class),
            "Should extract C++ class. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn csharp_extracts_class_and_method() {
        let src = b"namespace MyApp {\n    public class UserService {\n        public void CreateUser(string name) { }\n    }\n}";
        let ext = csharp_extractor();
        let result = ext.extract(src, "file::UserService.cs").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "UserService" && n.node_type == NodeType::Class),
            "Should extract C# class. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "CreateUser" && n.node_type == NodeType::Function),
            "Should extract C# method. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ruby_extracts_class_and_method() {
        let src = b"class Dog\n  def bark\n    puts 'woof'\n  end\nend";
        let ext = ruby_extractor();
        let result = ext.extract(src, "file::dog.rb").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Dog" && n.node_type == NodeType::Class),
            "Should extract Ruby class. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "bark" && n.node_type == NodeType::Function),
            "Should extract Ruby method. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ruby_extracts_module() {
        let src = b"module Animals\n  class Cat\n    def meow; end\n  end\nend";
        let ext = ruby_extractor();
        let result = ext.extract(src, "file::animals.rb").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Animals" && n.node_type == NodeType::Module),
            "Should extract Ruby module. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn php_extracts_class_and_function() {
        let src = b"<?php\nnamespace App;\nclass Controller {\n    public function index() { }\n}\nfunction helper() { }";
        let ext = php_extractor();
        let result = ext.extract(src, "file::Controller.php").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Controller" && n.node_type == NodeType::Class),
            "Should extract PHP class. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn swift_extracts_struct_and_func() {
        // Swift's tree-sitter grammar represents both struct and class as
        // class_declaration, so Point is extracted as Class (not Struct).
        let src = b"struct Point {\n    var x: Int\n    var y: Int\n}\n\nfunc add(_ a: Int, _ b: Int) -> Int {\n    return a + b\n}";
        let ext = swift_extractor();
        let result = ext.extract(src, "file::test.swift").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Point" && n.node_type == NodeType::Class),
            "Should extract Swift struct (as Class). Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "add" && n.node_type == NodeType::Function),
            "Should extract Swift function. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn swift_extracts_class_and_protocol() {
        let src = b"protocol Drawable {\n    func draw()\n}\n\nclass Circle: Drawable {\n    func draw() { }\n}";
        let ext = swift_extractor();
        let result = ext.extract(src, "file::test.swift").unwrap();
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Drawable" && n.node_type == NodeType::Type),
            "Should extract Swift protocol. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
        assert!(
            result
                .nodes
                .iter()
                .any(|n| n.label == "Circle" && n.node_type == NodeType::Class),
            "Should extract Swift class. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn bash_extracts_function() {
        let src = b"#!/bin/bash\n\nmy_func() {\n    echo 'hello'\n}\n\nfunction another_func {\n    echo 'world'\n}";
        let ext = bash_extractor();
        let result = ext.extract(src, "file::test.sh").unwrap();
        // At least one function should be found
        let funcs: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Function)
            .collect();
        assert!(
            !funcs.is_empty(),
            "Should extract at least one bash function. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn lua_extracts_function() {
        let src = b"function greet(name)\n    print('hello ' .. name)\nend\n\nlocal function helper()\n    return 42\nend";
        let ext = lua_extractor();
        let result = ext.extract(src, "file::test.lua").unwrap();
        let funcs: Vec<_> = result
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Function)
            .collect();
        assert!(
            !funcs.is_empty(),
            "Should extract Lua functions. Nodes: {:?}",
            result
                .nodes
                .iter()
                .map(|n| (&n.label, &n.node_type))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn file_node_always_present() {
        // Every tree-sitter extractor must produce a File node as the first node
        let extractors: Vec<(&str, Box<dyn Extractor>)> = vec![
            ("c", Box::new(c_extractor())),
            ("cpp", Box::new(cpp_extractor())),
            ("csharp", Box::new(csharp_extractor())),
            ("ruby", Box::new(ruby_extractor())),
            ("php", Box::new(php_extractor())),
            ("swift", Box::new(swift_extractor())),
            ("kotlin", Box::new(kotlin_extractor())),
            ("scala", Box::new(scala_extractor())),
            ("bash", Box::new(bash_extractor())),
            ("lua", Box::new(lua_extractor())),
            ("r", Box::new(r_extractor())),
            ("html", Box::new(html_extractor())),
            ("css", Box::new(css_extractor())),
            ("json", Box::new(json_extractor())),
        ];
        for (lang, ext) in extractors {
            let result = ext
                .extract(b"/* empty */", &format!("file::test.{}", lang))
                .unwrap();
            assert!(
                !result.nodes.is_empty(),
                "{} extractor should produce at least a file node",
                lang
            );
            assert_eq!(
                result.nodes[0].node_type,
                NodeType::File,
                "{} extractor first node should be File, got {:?}",
                lang,
                result.nodes[0].node_type
            );
        }
    }

    #[test]
    fn containment_edges_have_correct_parent() {
        // Verify that a class containing a method produces correct containment
        let src = b"class Dog\n  def bark\n    puts 'woof'\n  end\nend";
        let ext = ruby_extractor();
        let result = ext.extract(src, "file::dog.rb").unwrap();

        // Find the Dog class node
        let dog_node = result.nodes.iter().find(|n| n.label == "Dog").unwrap();
        // Find containment edge from file to Dog
        let file_to_dog = result
            .edges
            .iter()
            .find(|e| e.target == dog_node.id && e.relation == "contains");
        assert!(
            file_to_dog.is_some(),
            "Should have contains edge to Dog class"
        );

        // Find the bark method node
        let bark_node = result.nodes.iter().find(|n| n.label == "bark").unwrap();
        // Find containment edge from Dog to bark
        let dog_to_bark = result.edges.iter().find(|e| {
            e.target == bark_node.id && e.relation == "contains" && e.source == dog_node.id
        });
        assert!(
            dog_to_bark.is_some(),
            "Should have contains edge from Dog to bark. Edges: {:?}",
            result
                .edges
                .iter()
                .filter(|e| e.relation == "contains")
                .collect::<Vec<_>>()
        );
    }

    // ===================================================================
    // Tier 2 language tests
    // ===================================================================

    #[cfg(feature = "tier2")]
    mod tier2 {
        use super::*;

        // -- Dart --
        #[test]
        fn dart_extracts_class_and_methods() {
            let src = b"class Worker {\n  void process(String data) {}\n  int _helper(int x) => x * 2;\n}";
            let ext = dart_extractor();
            let result = ext.extract(src, "file::worker.dart").unwrap();
            assert!(
                result
                    .nodes
                    .iter()
                    .any(|n| n.label == "Worker" && n.node_type == NodeType::Class),
                "Should extract Dart class. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn dart_file_node_first() {
            let ext = dart_extractor();
            let result = ext.extract(b"class X {}", "file::test.dart").unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // -- Zig --
        #[test]
        fn zig_extracts_functions() {
            let src = b"pub fn main() void {}\n\nfn helper(x: i32) i32 {\n    return x * 2;\n}";
            let ext = zig_extractor();
            let result = ext.extract(src, "file::main.zig").unwrap();
            assert!(
                result
                    .nodes
                    .iter()
                    .any(|n| n.label == "main" && n.node_type == NodeType::Function),
                "Should extract Zig pub fn. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
            assert!(
                result
                    .nodes
                    .iter()
                    .any(|n| n.label == "helper" && n.node_type == NodeType::Function),
                "Should extract Zig private fn"
            );
        }

        #[test]
        fn zig_extracts_import() {
            let src = b"const std = @import(\"std\");";
            let ext = zig_extractor();
            let result = ext.extract(src, "file::main.zig").unwrap();
            let import_edges: Vec<_> = result
                .edges
                .iter()
                .filter(|e| e.relation == "imports")
                .collect();
            assert!(
                !import_edges.is_empty(),
                "Zig @import should produce import edge. All edges: {:?}",
                result
                    .edges
                    .iter()
                    .map(|e| (&e.relation, &e.target))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn zig_file_node_first() {
            let ext = zig_extractor();
            let result = ext.extract(b"fn x() void {}", "file::test.zig").unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // -- Haskell --
        #[test]
        fn haskell_extracts_functions() {
            let src = b"module Main where\n\nmain :: IO ()\nmain = putStrLn \"hello\"\n\nhelper :: Int -> Int\nhelper x = x * 2";
            let ext = haskell_extractor();
            let result = ext.extract(src, "file::Main.hs").unwrap();
            // Should extract at least one function (main or helper)
            let fns: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::Function)
                .collect();
            assert!(
                !fns.is_empty(),
                "Haskell should extract functions. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn haskell_extracts_data_type() {
            let src = b"module Main where\n\ndata Color = Red | Blue | Green";
            let ext = haskell_extractor();
            let result = ext.extract(src, "file::Types.hs").unwrap();
            let types: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::Type)
                .collect();
            assert!(
                !types.is_empty(),
                "Haskell should extract data types. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn haskell_extracts_imports() {
            let src = b"module Main where\n\nimport Data.List\nimport qualified Data.Map as Map\n\nmain = return ()";
            let ext = haskell_extractor();
            let result = ext.extract(src, "file::Main.hs").unwrap();
            let import_edges: Vec<_> = result
                .edges
                .iter()
                .filter(|e| e.relation == "imports")
                .collect();
            assert!(
                !import_edges.is_empty(),
                "Haskell should have import edges. All edges: {:?}",
                result
                    .edges
                    .iter()
                    .map(|e| (&e.relation, &e.target))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn haskell_file_node_first() {
            let ext = haskell_extractor();
            let result = ext.extract(b"module Main where", "file::Main.hs").unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // -- OCaml --
        #[test]
        fn ocaml_extracts_let_bindings() {
            let src = b"let main () =\n  print_endline \"hello\"\n\nlet helper x = x * 2";
            let ext = ocaml_extractor();
            let result = ext.extract(src, "file::main.ml").unwrap();
            let fns: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::Function)
                .collect();
            assert!(
                !fns.is_empty(),
                "OCaml should extract let bindings as functions. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn ocaml_extracts_type_definition() {
            let src = b"type color = Red | Blue | Green";
            let ext = ocaml_extractor();
            let result = ext.extract(src, "file::types.ml").unwrap();
            let types: Vec<_> = result
                .nodes
                .iter()
                .filter(|n| n.node_type == NodeType::Type)
                .collect();
            assert!(
                !types.is_empty(),
                "OCaml should extract type definitions. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn ocaml_extracts_open() {
            let src = b"open Printf\n\nlet main () = printf \"hello\\n\"";
            let ext = ocaml_extractor();
            let result = ext.extract(src, "file::main.ml").unwrap();
            let import_edges: Vec<_> = result
                .edges
                .iter()
                .filter(|e| e.relation == "imports")
                .collect();
            assert!(
                !import_edges.is_empty(),
                "OCaml 'open' should produce import edge. All edges: {:?}",
                result
                    .edges
                    .iter()
                    .map(|e| (&e.relation, &e.target))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn ocaml_file_node_first() {
            let ext = ocaml_extractor();
            let result = ext.extract(b"let x = 1", "file::test.ml").unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // -- TOML --
        #[test]
        fn toml_extracts_tables() {
            let src = b"[package]\nname = \"myapp\"\n\n[dependencies]\nserde = \"1.0\"";
            let ext = toml_extractor();
            let result = ext.extract(src, "file::Cargo.toml").unwrap();
            assert!(
                result.nodes.iter().any(|n| n.label == "package"),
                "Should extract [package] table. Nodes: {:?}",
                result.nodes.iter().map(|n| &n.label).collect::<Vec<_>>()
            );
            assert!(
                result.nodes.iter().any(|n| n.label == "dependencies"),
                "Should extract [dependencies] table. Nodes: {:?}",
                result.nodes.iter().map(|n| &n.label).collect::<Vec<_>>()
            );
        }

        #[test]
        fn toml_file_node_first() {
            let ext = toml_extractor();
            let result = ext
                .extract(b"[section]\nkey = \"val\"", "file::test.toml")
                .unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // -- YAML --
        #[test]
        fn yaml_file_node_present() {
            let src = b"key: value\nlist:\n  - item1\n  - item2";
            let ext = yaml_extractor();
            let result = ext.extract(src, "file::config.yaml").unwrap();
            assert_eq!(
                result.nodes[0].node_type,
                NodeType::File,
                "YAML extractor should produce File node first"
            );
        }

        // -- SQL --
        #[test]
        fn sql_extracts_create_table() {
            let src = b"CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  name TEXT NOT NULL\n);";
            let ext = sql_extractor();
            let result = ext.extract(src, "file::schema.sql").unwrap();
            assert!(
                result
                    .nodes
                    .iter()
                    .any(|n| n.label == "users" && n.node_type == NodeType::Struct),
                "Should extract CREATE TABLE as struct. Nodes: {:?}",
                result
                    .nodes
                    .iter()
                    .map(|n| (&n.label, &n.node_type))
                    .collect::<Vec<_>>()
            );
        }

        #[test]
        fn sql_file_node_first() {
            let ext = sql_extractor();
            let result = ext.extract(b"SELECT 1;", "file::test.sql").unwrap();
            assert_eq!(result.nodes[0].node_type, NodeType::File);
        }

        // NOTE: Dockerfile tests removed — grammar crate dropped due to
        // C symbol collisions (see Cargo.toml comment).

        // -- Integration tests --
        #[test]
        fn tier2_file_node_always_present() {
            let extractors: Vec<(&str, Box<dyn Extractor>, &[u8])> = vec![
                ("dart", Box::new(dart_extractor()), b"class X {}" as &[u8]),
                ("zig", Box::new(zig_extractor()), b"fn x() void {}"),
                (
                    "haskell",
                    Box::new(haskell_extractor()),
                    b"module Main where",
                ),
                ("ocaml", Box::new(ocaml_extractor()), b"let x = 1"),
                ("toml", Box::new(toml_extractor()), b"[section]"),
                ("yaml", Box::new(yaml_extractor()), b"key: value"),
                ("sql", Box::new(sql_extractor()), b"SELECT 1;"),
            ];
            for (lang, ext, src) in extractors {
                let result = ext.extract(src, &format!("file::test.{}", lang)).unwrap();
                assert!(
                    !result.nodes.is_empty(),
                    "{} extractor should produce at least a file node",
                    lang
                );
                assert_eq!(
                    result.nodes[0].node_type,
                    NodeType::File,
                    "{} extractor first node should be File, got {:?}",
                    lang,
                    result.nodes[0].node_type
                );
            }
        }

        // -- Verify existing regex extractors are NOT affected --
        #[test]
        fn regex_extractors_unchanged_by_tier2() {
            use crate::extract::go::GoExtractor;
            use crate::extract::java::JavaExtractor;
            use crate::extract::python::PythonExtractor;
            use crate::extract::rust_lang::RustExtractor;
            use crate::extract::typescript::TypeScriptExtractor;

            // Python
            let py = PythonExtractor::new();
            let r = py
                .extract(b"class Foo:\n    def bar(self): pass", "file::t.py")
                .unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "Foo" && n.node_type == NodeType::Class));
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "bar" && n.node_type == NodeType::Function));

            // Rust
            let rs = RustExtractor::new();
            let r = rs.extract(b"pub fn hello() {}", "file::t.rs").unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "hello" && n.node_type == NodeType::Function));

            // TypeScript
            let ts = TypeScriptExtractor::new();
            let r = ts
                .extract(b"export function greet() {}", "file::t.ts")
                .unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "greet" && n.node_type == NodeType::Function));

            // Go
            let go = GoExtractor::new();
            let r = go
                .extract(b"package main\nfunc main() {}", "file::main.go")
                .unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "main" && n.node_type == NodeType::Function));

            // Java
            let java = JavaExtractor::new();
            let r = java
                .extract(
                    b"public class App { public void run() {} }",
                    "file::App.java",
                )
                .unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "App" && n.node_type == NodeType::Class));
        }

        // -- Verify generic fallback still works --
        #[test]
        fn generic_fallback_still_works() {
            use crate::extract::generic::GenericExtractor;
            let ext = GenericExtractor::new();
            let r = ext
                .extract(
                    b"def helper():\n    pass\nclass Widget:\n    pass",
                    "file::unknown.xyz",
                )
                .unwrap();
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "helper" && n.node_type == NodeType::Function));
            assert!(r
                .nodes
                .iter()
                .any(|n| n.label == "Widget" && n.node_type == NodeType::Struct));
        }
    }
}
