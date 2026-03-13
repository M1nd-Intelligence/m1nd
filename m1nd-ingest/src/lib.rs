#![allow(unused)]
// === crates/m1nd-ingest/src/lib.rs ===

use m1nd_core::error::{M1ndError, M1ndResult};
use m1nd_core::graph::{Graph, NodeProvenanceInput};
use m1nd_core::types::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub mod walker;
pub mod extract;
pub mod resolve;
pub mod cross_file;
pub mod diff;
pub mod json_adapter;
pub mod memory_adapter;
pub mod merge;

// ---------------------------------------------------------------------------
// IngestAdapter — generic trait for domain-specific ingestion
// ---------------------------------------------------------------------------

/// Trait for domain-specific ingestion adapters.
/// Code ingestion is one adapter; music/DAW would be another; JSON is the
/// generic escape hatch for arbitrary domains.
pub trait IngestAdapter: Send + Sync {
    /// Domain name (e.g., "code", "music", "supply-chain")
    fn domain(&self) -> &str;

    /// Ingest from a path and return a populated graph + stats.
    fn ingest(&self, root: &std::path::Path) -> M1ndResult<(Graph, IngestStats)>;
}

// ---------------------------------------------------------------------------
// IngestConfig — ingestion parameters
// ---------------------------------------------------------------------------

/// Ingestion configuration.
/// FM-ING-002: timeout + node budget to prevent OOM.
pub struct IngestConfig {
    /// Root directory to ingest.
    pub root: PathBuf,
    /// Maximum time allowed for ingestion (FM-ING-002).
    pub timeout: Duration,
    /// Maximum number of nodes to create (FM-ING-002).
    pub max_nodes: u64,
    /// Skip directories matching these patterns.
    pub skip_dirs: Vec<String>,
    /// Skip files matching these patterns.
    pub skip_files: Vec<String>,
    /// Number of parallel extraction threads.
    pub parallelism: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            timeout: Duration::from_secs(300),
            max_nodes: 500_000,
            skip_dirs: vec![
                ".git".into(), "node_modules".into(), "__pycache__".into(),
                ".venv".into(), "target".into(), "dist".into(), "build".into(),
                ".next".into(), "vendor".into(),
            ],
            skip_files: vec![
                "package-lock.json".into(), "yarn.lock".into(),
                "Cargo.lock".into(), "poetry.lock".into(),
            ],
            parallelism: 8,
        }
    }
}

// ---------------------------------------------------------------------------
// IngestStats — ingestion result statistics
// ---------------------------------------------------------------------------

/// Statistics from an ingestion run.
#[derive(Clone, Debug, Default)]
pub struct IngestStats {
    pub files_scanned: u64,
    pub files_parsed: u64,
    pub files_skipped_binary: u64,
    pub files_skipped_encoding: u64,
    pub nodes_created: u64,
    pub edges_created: u64,
    pub references_resolved: u64,
    pub references_unresolved: u64,
    pub label_collisions: u64,
    pub elapsed_ms: f64,
    /// Groups of file paths that changed together in the same git commit.
    /// Used to populate the CoChangeMatrix after graph finalization.
    pub commit_groups: Vec<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Ingestor — main ingestion pipeline
// Replaces: ingest.py CodebaseIngestor
// ---------------------------------------------------------------------------

/// Codebase ingestion pipeline.
/// Walks directory -> detects language -> extracts structure via regex
/// -> resolves references -> builds graph.
/// FM-ING-009 fix: regex patterns handle indented defs.
/// FM-ING-004 fix: binary file detection (NUL byte in first 8KB).
/// FM-ING-003 fix: UTF-8 with lossy fallback.
/// Replaces: ingest.py CodebaseIngestor
pub struct Ingestor {
    config: IngestConfig,
}

impl Ingestor {
    pub fn new(config: IngestConfig) -> Self {
        Self { config }
    }

    fn source_path_from_node_id(node_id: &str) -> Option<&str> {
        node_id
            .strip_prefix("file::")
            .map(|rest| rest.split("::").next().unwrap_or(rest))
    }

    /// Select the appropriate extractor for a file extension.
    /// Existing regex-based extractors are used for Python, TypeScript, Rust,
    /// Go, and Java. Tree-sitter extractors handle Tier 1 languages (C, C++,
    /// C#, Ruby, PHP, Swift, Kotlin, Scala, Bash, Lua, R, HTML, CSS, JSON).
    fn select_extractor(ext: &str) -> Box<dyn extract::Extractor> {
        match ext {
            // --- Existing regex extractors (battle-tested) ---
            "py" | "pyi" => Box::new(extract::python::PythonExtractor::new()),
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
                Box::new(extract::typescript::TypeScriptExtractor::new())
            }
            "rs" => Box::new(extract::rust_lang::RustExtractor::new()),
            "go" => Box::new(extract::go::GoExtractor::new()),
            "java" => Box::new(extract::java::JavaExtractor::new()),

            // --- Tier 1: tree-sitter extractors ---
            #[cfg(feature = "tier1")]
            "c" | "h" => Box::new(extract::tree_sitter_ext::c_extractor()),
            #[cfg(feature = "tier1")]
            "cpp" | "cxx" | "cc" | "hpp" | "hxx" | "hh" => {
                Box::new(extract::tree_sitter_ext::cpp_extractor())
            }
            #[cfg(feature = "tier1")]
            "cs" => Box::new(extract::tree_sitter_ext::csharp_extractor()),
            #[cfg(feature = "tier1")]
            "rb" | "rake" | "gemspec" => Box::new(extract::tree_sitter_ext::ruby_extractor()),
            #[cfg(feature = "tier1")]
            "php" => Box::new(extract::tree_sitter_ext::php_extractor()),
            #[cfg(feature = "tier1")]
            "swift" => Box::new(extract::tree_sitter_ext::swift_extractor()),
            #[cfg(feature = "tier1")]
            "kt" | "kts" => Box::new(extract::tree_sitter_ext::kotlin_extractor()),
            #[cfg(feature = "tier1")]
            "scala" | "sc" => Box::new(extract::tree_sitter_ext::scala_extractor()),
            #[cfg(feature = "tier1")]
            "sh" | "bash" | "zsh" => Box::new(extract::tree_sitter_ext::bash_extractor()),
            #[cfg(feature = "tier1")]
            "lua" => Box::new(extract::tree_sitter_ext::lua_extractor()),
            #[cfg(feature = "tier1")]
            "r" | "R" | "Rmd" => Box::new(extract::tree_sitter_ext::r_extractor()),
            #[cfg(feature = "tier1")]
            "html" | "htm" => Box::new(extract::tree_sitter_ext::html_extractor()),
            #[cfg(feature = "tier1")]
            "css" => Box::new(extract::tree_sitter_ext::css_extractor()),
            #[cfg(feature = "tier1")]
            "json" => Box::new(extract::tree_sitter_ext::json_extractor()),

            // --- Tier 2: tree-sitter extractors ---
            #[cfg(feature = "tier2")]
            "ex" | "exs" => Box::new(extract::tree_sitter_ext::elixir_extractor()),
            #[cfg(feature = "tier2")]
            "dart" => Box::new(extract::tree_sitter_ext::dart_extractor()),
            #[cfg(feature = "tier2")]
            "zig" => Box::new(extract::tree_sitter_ext::zig_extractor()),
            #[cfg(feature = "tier2")]
            "hs" | "lhs" => Box::new(extract::tree_sitter_ext::haskell_extractor()),
            #[cfg(feature = "tier2")]
            "ml" | "mli" => Box::new(extract::tree_sitter_ext::ocaml_extractor()),
            #[cfg(feature = "tier2")]
            "toml" => Box::new(extract::tree_sitter_ext::toml_extractor()),
            #[cfg(feature = "tier2")]
            "yml" | "yaml" => Box::new(extract::tree_sitter_ext::yaml_extractor()),
            #[cfg(feature = "tier2")]
            "sql" => Box::new(extract::tree_sitter_ext::sql_extractor()),
            // --- Fallback ---
            _ => Box::new(extract::generic::GenericExtractor::new()),
        }
    }

    /// Full ingestion: walk -> parse -> resolve -> build graph.
    /// Returns the completed graph and statistics.
    /// Replaces: ingest.py CodebaseIngestor.ingest()
    pub fn ingest(&self) -> M1ndResult<(Graph, IngestStats)> {
        let start = Instant::now();
        let mut stats = IngestStats::default();

        // Phase 1: Walk directory
        let dir_walker = walker::DirectoryWalker::new(
            self.config.skip_dirs.clone(),
            self.config.skip_files.clone(),
        );
        let walk_result = dir_walker.walk(&self.config.root)?;
        stats.files_scanned = walk_result.files.len() as u64;

        // Phase 2: Extract from each file (parallel via rayon)
        use rayon::prelude::*;

        // Parallel extraction phase: read files and run extractors concurrently.
        // Graph building must remain single-threaded, so we collect results first.
        let extraction_results: Vec<(String, extract::ExtractionResult)> = walk_result.files
            .par_iter()
            .filter_map(|file| {
                let ext = file.extension.as_deref().unwrap_or("");
                let extractor = Self::select_extractor(ext);
                let content = std::fs::read(&file.path).ok()?;
                let file_id = format!("file::{}", file.relative_path);
                let result = extractor.extract(&content, &file_id).ok()?;
                Some((file_id, result))
            })
            .collect();

        // Sequential post-processing of parallel results
        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_unresolved = Vec::new();
        let mut import_hints: Vec<(String, String, String)> = Vec::new();

        for (file_id, result) in &extraction_results {
            // FM-ING-002: timeout check (after parallel phase)
            if start.elapsed() > self.config.timeout {
                eprintln!(
                    "[m1nd-ingest] Timeout after {} files",
                    stats.files_parsed
                );
                break;
            }

            // FM-ING-002: node budget check
            if all_nodes.len() as u64 >= self.config.max_nodes {
                eprintln!(
                    "[m1nd-ingest] Node budget reached: {}",
                    self.config.max_nodes
                );
                break;
            }

            stats.files_parsed += 1;

            // Collect unresolved references for later resolution
            for ref_id in &result.unresolved_refs {
                all_unresolved.push((
                    file_id.clone(),
                    ref_id.clone(),
                    "references".to_string(),
                ));
            }

            // Task #8: Build import hints from import edges
            let mut current_module_path: Option<String> = None;
            for edge in &result.edges {
                if edge.relation == "imports" && edge.target.starts_with("ref::") {
                    let target = edge.target.strip_prefix("ref::").unwrap_or(&edge.target);
                    if target.contains('.') || target.contains("::") {
                        current_module_path = Some(target.to_string());
                    } else if let Some(ref module) = current_module_path {
                        import_hints.push((
                            file_id.clone(),
                            edge.target.clone(),
                            module.clone(),
                        ));
                    }
                }
            }

            all_nodes.extend(result.nodes.iter().cloned());
            all_edges.extend(result.edges.iter().cloned());
        }

        // Track files that failed to read/parse (total scanned - successful extractions)
        stats.files_skipped_encoding = stats.files_scanned.saturating_sub(extraction_results.len() as u64);

        // Phase 3: Build graph
        let estimated_nodes = all_nodes.len();
        let estimated_edges = all_edges.len();
        let mut graph = Graph::with_capacity(estimated_nodes, estimated_edges);

        // Build file metadata lookup for proper temporal scoring
        let mut file_timestamps: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        let mut file_change_freq: std::collections::HashMap<String, f32> =
            std::collections::HashMap::new();
        for file in &walk_result.files {
            let file_id = format!("file::{}", file.relative_path);
            file_timestamps.insert(file_id.clone(), file.last_modified);
            // Change frequency from git commit count (normalized: 1 commit = 0.1, 50+ = 1.0)
            let freq = if file.commit_count > 0 {
                ((file.commit_count as f32) / 50.0).min(1.0).max(0.1)
            } else {
                0.3 // default for non-git repos
            };
            file_change_freq.insert(file_id, freq);
        }

        // Add all nodes
        for node in &all_nodes {
            let tags: Vec<&str> = node.tags.iter().map(|s| s.as_str()).collect();
            // Use real file timestamp for temporal scoring, not line number
            let file_prefix = node.id.split("::").take(2).collect::<Vec<_>>().join("::");
            let timestamp = file_timestamps
                .get(&file_prefix)
                .copied()
                .unwrap_or(0.0);
            // Change frequency from git history (or default)
            let change_freq = file_change_freq
                .get(&file_prefix)
                .copied()
                .unwrap_or(0.3);
            match graph.add_node(
                &node.id,
                &node.label,
                node.node_type,
                &tags,
                timestamp,
                change_freq,
            ) {
                Ok(node_id) => {
                    graph.set_node_provenance(
                        node_id,
                        NodeProvenanceInput {
                            source_path: Self::source_path_from_node_id(&node.id),
                            line_start: Some(node.line),
                            line_end: Some(node.end_line),
                            excerpt: None,
                            namespace: Some("code"),
                            canonical: false,
                        },
                    );
                    stats.nodes_created += 1
                }
                Err(M1ndError::DuplicateNode(_)) => {
                    stats.label_collisions += 1;
                }
                Err(_) => {}
            }
        }

        // Add all edges (skip edges with unresolvable endpoints)
        for edge in &all_edges {
            // Skip ref:: edges (will be resolved later)
            if edge.target.starts_with("ref::") {
                continue;
            }

            if let (Some(src), Some(tgt)) = (
                graph.resolve_id(&edge.source),
                graph.resolve_id(&edge.target),
            ) {
                // Set causal strength based on relation type:
                // "contains" = strong causal (parent changes → child affected)
                // "imports" = moderate causal (dependency changes → importer affected)
                // "calls" = moderate causal
                // "references" = weak causal
                let causal_strength = match edge.relation.as_str() {
                    "contains" => 0.8,
                    "imports" => 0.6,
                    "calls" => 0.5,
                    "implements" => 0.7,
                    "references" => 0.3,
                    _ => 0.4,
                };
                // Use Bidirectional for contains (parent ↔ child navigate both ways)
                let direction = if edge.relation == "contains" {
                    EdgeDirection::Bidirectional
                } else {
                    EdgeDirection::Forward
                };
                match graph.add_edge(
                    src,
                    tgt,
                    &edge.relation,
                    FiniteF32::new(edge.weight),
                    direction,
                    false,
                    FiniteF32::new(causal_strength),
                ) {
                    Ok(_) => stats.edges_created += 1,
                    Err(_) => {}
                }
            }
        }

        // Phase 4: Resolve references (with module-aware import hints)
        let res_stats = resolve::ReferenceResolver::resolve_with_hints(
            &mut graph,
            &all_unresolved,
            &import_hints,
        )?;
        stats.references_resolved = res_stats.resolved;
        stats.references_unresolved = res_stats.unresolved;

        // Phase 4.5: Cross-file edges (L1 — imports, tests, registers)
        // Resolves ref:: module paths to deterministic file-to-file edges.
        match cross_file::resolve_cross_file_edges(&mut graph, &self.config.root) {
            Ok(cf_stats) => {
                stats.edges_created += cf_stats.total_edges_created;
                eprintln!(
                    "[m1nd-ingest] Cross-file edges: {} imports, {} tests, {} registers ({} total)",
                    cf_stats.imports_resolved,
                    cf_stats.test_edges_created,
                    cf_stats.register_edges_created,
                    cf_stats.total_edges_created,
                );
            }
            Err(e) => {
                eprintln!("[m1nd-ingest] Cross-file edge resolution failed (non-fatal): {}", e);
            }
        }

        // Phase 5: Finalize (CSR + PageRank)
        if graph.num_nodes() > 0 {
            graph.finalize()?;
        }

        // Store commit groups for co-change matrix population
        stats.commit_groups = walk_result.commit_groups;

        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((graph, stats))
    }

    /// Incremental ingestion: only process changed files.
    /// Returns a diff that can be applied to an existing graph.
    /// Replaces: ingest.py CodebaseIngestor.ingest_incremental() (new capability)
    #[allow(dead_code)]
    pub fn ingest_incremental(
        &self,
        existing: &Graph,
        changed_files: &[PathBuf],
    ) -> M1ndResult<(diff::GraphDiff, IngestStats)> {
        let start = Instant::now();
        let mut stats = IngestStats::default();

        // Extract from changed files
        let mut new_nodes = Vec::new();
        let mut new_edges = Vec::new();

        for file_path in changed_files {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let extractor = Self::select_extractor(ext);

            let content = match std::fs::read(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let relative = file_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            let file_id = format!("file::{}", relative);

            match extractor.extract(&content, &file_id) {
                Ok(result) => {
                    stats.files_parsed += 1;
                    new_nodes.extend(result.nodes);
                    new_edges.extend(result.edges);
                }
                Err(_) => {}
            }
        }

        // Get old nodes/edges for changed files (approximate: use file IDs)
        // For a proper implementation, we'd need to store previous extraction results.
        // For now, return a diff with all new nodes as additions.
        let graph_diff = diff::GraphDiff::compute(
            &[], // old nodes (empty = treat all as new)
            &[], // old edges
            &new_nodes,
            &new_edges,
        );

        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((graph_diff, stats))
    }
}

// ---------------------------------------------------------------------------
// IngestAdapter implementation for Ingestor (code domain)
// ---------------------------------------------------------------------------

impl IngestAdapter for Ingestor {
    fn domain(&self) -> &str {
        "code"
    }

    fn ingest(&self, root: &std::path::Path) -> M1ndResult<(Graph, IngestStats)> {
        let config = IngestConfig {
            root: root.to_path_buf(),
            ..IngestConfig::default()
        };
        let ingestor = Ingestor::new(config);
        ingestor.ingest()
    }
}

// ===========================================================================
// Tests — comprehensive unit tests for all extractors and resolver
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::extract::*;
    use super::extract::rust_lang::RustExtractor;
    use super::extract::python::PythonExtractor;
    use super::extract::typescript::TypeScriptExtractor;
    use super::extract::go::GoExtractor;
    use super::extract::java::JavaExtractor;
    use super::extract::generic::GenericExtractor;
    use super::resolve::ReferenceResolver;
    use m1nd_core::graph::Graph;
    use m1nd_core::types::*;

    // -----------------------------------------------------------------------
    // Rust extractor tests (8 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn rust_extracts_pub_function() {
        let src = b"pub fn hello_world() -> String { todo!() }";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "hello_world"
            && n.node_type == NodeType::Function));
    }

    #[test]
    fn rust_extracts_private_function() {
        let src = b"fn internal_helper(x: u32) -> bool { true }";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "internal_helper"
            && n.node_type == NodeType::Function));
    }

    #[test]
    fn rust_extracts_struct_and_enum_and_trait() {
        let src = b"pub struct Config { }\npub enum Color { Red, Blue }\npub trait Drawable { }";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Config" && n.node_type == NodeType::Struct));
        assert!(result.nodes.iter().any(|n| n.label == "Color" && n.node_type == NodeType::Enum));
        assert!(result.nodes.iter().any(|n| n.label == "Drawable" && n.node_type == NodeType::Type));
    }

    #[test]
    fn rust_extracts_use_imports_with_braces() {
        let src = b"use std::collections::{HashMap, HashSet};\nuse std::io::Read;";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        let import_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "imports")
            .collect();
        // Should have 3 imports: HashMap, HashSet, Read
        assert!(import_edges.len() >= 3,
            "Expected >= 3 imports, got {}: {:?}",
            import_edges.len(),
            import_edges.iter().map(|e| &e.target).collect::<Vec<_>>());
        assert!(import_edges.iter().any(|e| e.target.contains("HashMap")));
        assert!(import_edges.iter().any(|e| e.target.contains("HashSet")));
        assert!(import_edges.iter().any(|e| e.target.contains("Read")));
    }

    #[test]
    fn rust_extracts_impl_implements_edge() {
        let src = b"impl Display for Config {\n    fn fmt(&self, f: &mut Formatter) -> Result { Ok(()) }\n}";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        let impl_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "implements")
            .collect();
        assert!(!impl_edges.is_empty(), "Should have implements edge");
        assert!(impl_edges.iter().any(|e| e.target.contains("Config")));
    }

    #[test]
    fn rust_extracts_enum_variants() {
        let src = b"pub enum Direction {\n    North,\n    South,\n    East,\n    West,\n}";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Direction" && n.node_type == NodeType::Enum));
        assert!(result.nodes.iter().any(|n| n.label == "North" && n.tags.contains(&"variant".to_string())));
        assert!(result.nodes.iter().any(|n| n.label == "South" && n.tags.contains(&"variant".to_string())));
        // Verify contains edges from enum to variants
        let contains_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "contains" && e.target.contains("::Direction::"))
            .collect();
        assert!(contains_edges.len() >= 1, "Enum should have contains edges to variants");
    }

    #[test]
    fn rust_comment_stripping_no_false_node() {
        let src = b"// fn fake_function() { }\nfn real_function() { }";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(!result.nodes.iter().any(|n| n.label == "fake_function"),
            "Commented-out function should not be extracted");
        assert!(result.nodes.iter().any(|n| n.label == "real_function"),
            "Real function should be extracted");
    }

    #[test]
    fn rust_string_stripping_no_false_node() {
        let src = b"fn real() {\n    let s = \"fn fake_in_string() { }\";\n}";
        let ext = RustExtractor::new();
        let result = ext.extract(src, "file::test.rs").unwrap();
        assert!(!result.nodes.iter().any(|n| n.label == "fake_in_string"),
            "Function name inside string literal should not be extracted");
        assert!(result.nodes.iter().any(|n| n.label == "real"),
            "Real function should be extracted");
    }

    // -----------------------------------------------------------------------
    // Python extractor tests (7 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn python_extracts_class_and_function() {
        let src = b"class MyService:\n    def process(self, data):\n        pass\n\ndef standalone():\n    pass";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "MyService" && n.node_type == NodeType::Class));
        assert!(result.nodes.iter().any(|n| n.label == "process" && n.node_type == NodeType::Function));
        assert!(result.nodes.iter().any(|n| n.label == "standalone" && n.node_type == NodeType::Function));
    }

    #[test]
    fn python_extracts_imports() {
        let src = b"import os\nfrom pathlib import Path\nfrom collections import OrderedDict, defaultdict";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        let import_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "imports")
            .collect();
        assert!(import_edges.iter().any(|e| e.target.contains("os")));
        assert!(import_edges.iter().any(|e| e.target.contains("pathlib")));
        assert!(import_edges.iter().any(|e| e.target.contains("Path")));
        assert!(import_edges.iter().any(|e| e.target.contains("OrderedDict")));
    }

    #[test]
    fn python_extracts_class_inheritance() {
        let src = b"class Dog(Animal):\n    pass";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        let impl_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "implements")
            .collect();
        assert!(impl_edges.iter().any(|e| e.target.contains("Animal")),
            "Should have implements edge to Animal base class");
    }

    #[test]
    fn python_extracts_decorators() {
        let src = b"@staticmethod\ndef my_func():\n    pass\n\n@app.route\nclass Handler:\n    pass";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        let ref_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "references" && e.target.starts_with("ref::"))
            .collect();
        // Decorators produce reference edges from the decorated item
        assert!(ref_edges.iter().any(|e| e.target.contains("staticmethod")),
            "Should have reference to @staticmethod decorator. Edges: {:?}",
            ref_edges.iter().map(|e| (&e.source, &e.target)).collect::<Vec<_>>());
    }

    #[test]
    fn python_extracts_type_hints() {
        let src = b"def process(data: DataFrame) -> ResultSet:\n    pass";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        let ref_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "references")
            .collect();
        assert!(ref_edges.iter().any(|e| e.target.contains("DataFrame")),
            "Should reference DataFrame type hint");
        assert!(ref_edges.iter().any(|e| e.target.contains("ResultSet")),
            "Should reference ResultSet return type hint");
    }

    #[test]
    fn python_extracts_method_calls() {
        let src = b"def run():\n    result = Parser.parse(data)\n    logger.info(msg)";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        let call_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "calls")
            .collect();
        // Parser.parse -> ref to Parser (uppercase receiver = type call)
        assert!(call_edges.iter().any(|e| e.target.contains("Parser")),
            "Should have calls edge to Parser. Call edges: {:?}",
            call_edges.iter().map(|e| &e.target).collect::<Vec<_>>());
    }

    #[test]
    fn python_comment_stripping() {
        let src = b"# class FakeClass:\n#     pass\nclass RealClass:\n    pass";
        let ext = PythonExtractor::new();
        let result = ext.extract(src, "file::test.py").unwrap();
        assert!(!result.nodes.iter().any(|n| n.label == "FakeClass"),
            "Commented-out class should not be extracted");
        assert!(result.nodes.iter().any(|n| n.label == "RealClass"),
            "Real class should be extracted");
    }

    // -----------------------------------------------------------------------
    // TypeScript extractor tests (4 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn typescript_extracts_function_class_interface() {
        let src = b"export function fetchData(url: string): Promise<Response> { }\n\
                     export class ApiClient { }\n\
                     export interface Config { }";
        let ext = TypeScriptExtractor::new();
        let result = ext.extract(src, "file::test.ts").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "fetchData" && n.node_type == NodeType::Function));
        assert!(result.nodes.iter().any(|n| n.label == "ApiClient" && n.node_type == NodeType::Class));
        assert!(result.nodes.iter().any(|n| n.label == "Config" && n.node_type == NodeType::Type));
    }

    #[test]
    fn typescript_extracts_type_references() {
        // TypeScript type annotations produce reference edges
        let src = b"function process(data: UserConfig): AppResult {\n    const x: ServiceClient = init();\n}";
        let ext = TypeScriptExtractor::new();
        let result = ext.extract(src, "file::test.ts").unwrap();
        let ref_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "references")
            .collect();
        assert!(ref_edges.iter().any(|e| e.target.contains("UserConfig")),
            "Should reference UserConfig type annotation. Ref edges: {:?}",
            ref_edges.iter().map(|e| &e.target).collect::<Vec<_>>());
        assert!(ref_edges.iter().any(|e| e.target.contains("ServiceClient")),
            "Should reference ServiceClient type annotation");
    }

    #[test]
    fn typescript_comment_stripping() {
        let src = b"// function fakeFunc() { }\nfunction realFunc() { }";
        let ext = TypeScriptExtractor::new();
        let result = ext.extract(src, "file::test.ts").unwrap();
        assert!(!result.nodes.iter().any(|n| n.label == "fakeFunc"),
            "Commented-out function should not be extracted");
        assert!(result.nodes.iter().any(|n| n.label == "realFunc"),
            "Real function should be extracted");
    }

    #[test]
    fn typescript_extracts_arrow_functions() {
        let src = b"export const handler = (req: Request) => { };";
        let ext = TypeScriptExtractor::new();
        let result = ext.extract(src, "file::test.ts").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "handler" && n.node_type == NodeType::Function),
            "Should extract arrow function. Nodes: {:?}",
            result.nodes.iter().map(|n| &n.label).collect::<Vec<_>>());
    }

    // -----------------------------------------------------------------------
    // Go extractor tests (3 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn go_extracts_functions_and_structs() {
        let src = b"package main\n\ntype Config struct {\n    Host string\n}\n\nfunc NewConfig() *Config {\n    return &Config{}\n}";
        let ext = GoExtractor::new();
        let result = ext.extract(src, "file::main.go").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Config" && n.node_type == NodeType::Struct),
            "Should extract struct. Nodes: {:?}",
            result.nodes.iter().map(|n| (&n.label, &n.node_type)).collect::<Vec<_>>());
        assert!(result.nodes.iter().any(|n| n.label == "NewConfig" && n.node_type == NodeType::Function));
    }

    #[test]
    fn go_extracts_imports_and_string_strip_interaction() {
        // Go imports use quoted strings ("fmt"), but comment/string stripping
        // removes string contents. Verify the extractor handles block imports
        // by checking that the import block is tracked (in_import_block state).
        // The current regex-based approach cannot extract import paths after
        // string stripping -- this test documents that behavior.
        let src = b"package main\n\nimport (\n    \"fmt\"\n    \"net/http\"\n)\n\nfunc main() { }";
        let ext = GoExtractor::new();
        let result = ext.extract(src, "file::main.go").unwrap();
        // The function should still be extracted despite the import block
        assert!(result.nodes.iter().any(|n| n.label == "main" && n.node_type == NodeType::Function));
        // File node should always be present
        assert!(result.nodes.iter().any(|n| n.node_type == NodeType::File));
    }

    #[test]
    fn go_extracts_interface() {
        let src = b"package main\n\ntype Reader interface {\n    Read(p []byte) (n int, err error)\n}";
        let ext = GoExtractor::new();
        let result = ext.extract(src, "file::io.go").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Reader" && n.node_type == NodeType::Type),
            "Should extract Go interface");
    }

    #[test]
    fn go_extracts_methods() {
        let src = b"func (c *Config) Validate() error {\n    return nil\n}";
        let ext = GoExtractor::new();
        let result = ext.extract(src, "file::config.go").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Validate" && n.node_type == NodeType::Function),
            "Should extract method receiver function");
    }

    // -----------------------------------------------------------------------
    // Java extractor tests (3 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn java_extracts_class_and_methods() {
        let src = b"public class UserService {\n    public void createUser(String name) { }\n    private int countUsers() { return 0; }\n}";
        let ext = JavaExtractor::new();
        let result = ext.extract(src, "file::UserService.java").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "UserService" && n.node_type == NodeType::Class));
        assert!(result.nodes.iter().any(|n| n.label == "createUser" && n.node_type == NodeType::Function));
        assert!(result.nodes.iter().any(|n| n.label == "countUsers" && n.node_type == NodeType::Function));
    }

    #[test]
    fn java_extracts_imports() {
        let src = b"import java.util.List;\nimport java.io.IOException;\n\npublic class Main { }";
        let ext = JavaExtractor::new();
        let result = ext.extract(src, "file::Main.java").unwrap();
        let import_edges: Vec<_> = result.edges.iter()
            .filter(|e| e.relation == "imports")
            .collect();
        assert!(import_edges.iter().any(|e| e.target.contains("java.util.List")));
        assert!(import_edges.iter().any(|e| e.target.contains("java.io.IOException")));
    }

    #[test]
    fn java_extracts_interface_and_enum() {
        let src = b"public interface Serializable { }\npublic enum Status { ACTIVE, INACTIVE }";
        let ext = JavaExtractor::new();
        let result = ext.extract(src, "file::Types.java").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "Serializable" && n.node_type == NodeType::Type));
        assert!(result.nodes.iter().any(|n| n.label == "Status" && n.node_type == NodeType::Enum));
    }

    // -----------------------------------------------------------------------
    // Generic extractor test (1 test)
    // -----------------------------------------------------------------------

    #[test]
    fn generic_extracts_function_and_class() {
        let src = b"def helper():\n    pass\n\nclass Widget:\n    pass";
        let ext = GenericExtractor::new();
        let result = ext.extract(src, "file::script.txt").unwrap();
        assert!(result.nodes.iter().any(|n| n.label == "helper" && n.node_type == NodeType::Function));
        assert!(result.nodes.iter().any(|n| n.label == "Widget" && n.node_type == NodeType::Struct));
    }

    // -----------------------------------------------------------------------
    // Comment/string stripping tests (2 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn strip_block_comments() {
        let src = "fn real() { }\n/* fn fake() { } */\nfn also_real() { }";
        let cleaned = strip_comments_and_strings(src, CommentSyntax::RUST);
        assert!(cleaned[0].contains("real"));
        assert!(!cleaned[1].contains("fake"), "Block comment content should be stripped");
        assert!(cleaned[2].contains("also_real"));
    }

    #[test]
    fn strip_python_triple_quote() {
        let src = "class Real:\n    \"\"\"This is a docstring with class Fake: inside\"\"\"\n    pass";
        let cleaned = strip_comments_and_strings(src, CommentSyntax::PYTHON);
        assert!(cleaned[0].contains("Real"));
        assert!(!cleaned[1].contains("Fake"),
            "Triple-quoted string content should be stripped. Got: {:?}", cleaned[1]);
    }

    // -----------------------------------------------------------------------
    // Resolver tests (3 tests)
    // -----------------------------------------------------------------------

    #[test]
    fn resolver_exact_label_match() {
        let mut graph = Graph::with_capacity(4, 4);
        // Add file node
        graph.add_node("file::a.rs", "a.rs", NodeType::File, &["rust"], 0.0, 0.3).unwrap();
        // Add a struct node
        graph.add_node("file::a.rs::struct::Config", "Config", NodeType::Struct, &["rust"], 0.0, 0.3).unwrap();
        // Add source file that references Config
        graph.add_node("file::b.rs", "b.rs", NodeType::File, &["rust"], 0.0, 0.3).unwrap();

        let unresolved = vec![
            ("file::b.rs".to_string(), "ref::Config".to_string(), "references".to_string()),
        ];
        let stats = ReferenceResolver::resolve(&mut graph, &unresolved).unwrap();
        assert_eq!(stats.resolved, 1, "Should resolve Config reference");
        assert_eq!(stats.unresolved, 0);
    }

    #[test]
    fn resolver_same_file_disambiguation() {
        let mut graph = Graph::with_capacity(8, 8);
        // Two files, each with a "Config" node
        graph.add_node("file::a.rs", "a.rs", NodeType::File, &["rust"], 0.0, 0.3).unwrap();
        graph.add_node("file::a.rs::struct::Config", "Config", NodeType::Struct, &["rust"], 0.0, 0.3).unwrap();
        graph.add_node("file::b.rs", "b.rs", NodeType::File, &["rust"], 0.0, 0.3).unwrap();
        graph.add_node("file::b.rs::struct::Config", "Config", NodeType::Struct, &["rust"], 0.0, 0.3).unwrap();
        // Source referencing Config from a.rs should prefer a.rs::Config
        graph.add_node("file::a.rs::fn::main", "main", NodeType::Function, &["rust"], 0.0, 0.3).unwrap();

        let unresolved = vec![
            ("file::a.rs::fn::main".to_string(), "ref::Config".to_string(), "references".to_string()),
        ];
        let stats = ReferenceResolver::resolve(&mut graph, &unresolved).unwrap();
        assert_eq!(stats.resolved, 1, "Should resolve despite ambiguity");
        assert_eq!(stats.ambiguous, 1, "Should report ambiguity");
    }

    #[test]
    fn resolver_unresolvable_reference() {
        let mut graph = Graph::with_capacity(4, 4);
        graph.add_node("file::a.rs", "a.rs", NodeType::File, &["rust"], 0.0, 0.3).unwrap();
        graph.add_node("file::a.rs::fn::main", "main", NodeType::Function, &["rust"], 0.0, 0.3).unwrap();

        let unresolved = vec![
            ("file::a.rs".to_string(), "ref::NonExistentType".to_string(), "references".to_string()),
        ];
        let stats = ReferenceResolver::resolve(&mut graph, &unresolved).unwrap();
        assert_eq!(stats.unresolved, 1, "Should report unresolved reference");
        assert_eq!(stats.resolved, 0);
    }

    // -----------------------------------------------------------------------
    // File node creation test (1 test)
    // -----------------------------------------------------------------------

    #[test]
    fn all_extractors_create_file_node() {
        // Every extractor should create a file-level node as the first node
        let rust_ext = RustExtractor::new();
        let py_ext = PythonExtractor::new();
        let ts_ext = TypeScriptExtractor::new();
        let go_ext = GoExtractor::new();
        let java_ext = JavaExtractor::new();

        let r = rust_ext.extract(b"fn x() {}", "file::t.rs").unwrap();
        assert!(r.nodes[0].node_type == NodeType::File);

        let r = py_ext.extract(b"def x(): pass", "file::t.py").unwrap();
        assert!(r.nodes[0].node_type == NodeType::File);

        let r = ts_ext.extract(b"function x() {}", "file::t.ts").unwrap();
        assert!(r.nodes[0].node_type == NodeType::File);

        let r = go_ext.extract(b"package main\nfunc x() {}", "file::t.go").unwrap();
        assert!(r.nodes[0].node_type == NodeType::File);

        let r = java_ext.extract(b"public class X {}", "file::X.java").unwrap();
        assert!(r.nodes[0].node_type == NodeType::File);
    }
}
