#![allow(unused)]
// === crates/m1nd-ingest/src/lib.rs ===

use m1nd_core::error::{M1ndError, M1ndResult};
use m1nd_core::types::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub mod cross_file;
pub mod diff;
pub mod extract;
pub mod json_adapter;
pub mod l1ght_adapter;
pub mod memory_adapter;
pub mod merge;
pub mod resolve;
pub mod walker;

pub use l1ght_adapter::L1ghtIngestAdapter;

pub trait IngestAdapter: Send + Sync {
    fn domain(&self) -> &str;
    fn ingest(&self, root: &std::path::Path) -> M1ndResult<(m1nd_core::graph::Graph, IngestStats)>;
}

pub struct IngestConfig {
    pub root: PathBuf,
    pub timeout: Duration,
    pub max_nodes: u64,
    pub skip_dirs: Vec<String>,
    pub skip_files: Vec<String>,
    pub parallelism: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            timeout: Duration::from_secs(300),
            max_nodes: 500_000,
            skip_dirs: vec![
                ".git".into(),
                "node_modules".into(),
                "__pycache__".into(),
                ".venv".into(),
                "target".into(),
                "dist".into(),
                "build".into(),
                ".next".into(),
                "vendor".into(),
            ],
            skip_files: vec![
                "package-lock.json".into(),
                "yarn.lock".into(),
                "Cargo.lock".into(),
                "poetry.lock".into(),
            ],
            parallelism: std::thread::available_parallelism()
                .map(|p| p.get().min(16))
                .unwrap_or(8),
        }
    }
}

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
    pub commit_groups: Vec<Vec<String>>,
}

pub struct Ingestor {
    config: IngestConfig,
}

impl Ingestor {
    pub fn new(config: IngestConfig) -> Self {
        Self { config }
    }

    fn select_extractor(ext: &str) -> Box<dyn extract::Extractor> {
        match ext {
            "py" | "pyi" => Box::new(extract::python::PythonExtractor::new()),
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => {
                Box::new(extract::typescript::TypeScriptExtractor::new())
            }
            "rs" => Box::new(extract::rust_lang::RustExtractor::new()),
            "go" => Box::new(extract::go::GoExtractor::new()),
            "java" => Box::new(extract::java::JavaExtractor::new()),
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
            "html" | "htm" => Box::new(extract::tree_sitter_ext::EmbeddedExtractor::html_embedded()),
            #[cfg(feature = "tier1")]
            "css" => Box::new(extract::tree_sitter_ext::css_extractor()),
            #[cfg(feature = "tier1")]
            "json" => Box::new(extract::tree_sitter_ext::json_extractor()),
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
            _ => Box::new(extract::generic::GenericExtractor::new()),
        }
    }

    pub fn ingest(&self) -> M1ndResult<(m1nd_core::graph::Graph, IngestStats)> {
        let start = Instant::now();
        let mut stats = IngestStats::default();

        let dir_walker = walker::DirectoryWalker::new(
            self.config.skip_dirs.clone(),
            self.config.skip_files.clone(),
        );
        let walk_result = dir_walker.walk(&self.config.root)?;
        stats.files_scanned = walk_result.files.len() as u64;

        use rayon::prelude::*;
        let num_threads = self.config.parallelism.clamp(1, 64);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| M1ndError::InvalidParams {
                tool: "ingest".into(),
                detail: format!("thread pool: {}", e),
            })?;

        let extraction_results: Vec<(String, extract::ExtractionResult)> = pool.install(|| {
            walk_result
                .files
                .par_iter()
                .filter_map(|file| {
                    let ext = file.extension.as_deref().unwrap_or("");
                    let extractor = Self::select_extractor(ext);
                    let content = std::fs::read(&file.path).ok()?;
                    let file_id = format!("file::{}", file.relative_path);
                    let result = extractor.extract(&content, &file_id).ok()?;
                    Some((file_id, result))
                })
                .collect()
        });

        let mut all_nodes = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_unresolved = Vec::new();
        let mut import_hints: Vec<(String, String, String)> = Vec::new();

        for (file_id, result) in &extraction_results {
            if start.elapsed() > self.config.timeout {
                eprintln!("[m1nd-ingest] Timeout after {} files", stats.files_parsed);
                break;
            }

            stats.files_parsed += 1;
            stats.nodes_created += result.nodes.len() as u64;
            stats.edges_created += result.edges.len() as u64;
            all_nodes.extend_from_slice(&result.nodes);
            all_edges.extend_from_slice(&result.edges);
            all_unresolved.extend(result.unresolved_refs.iter().cloned());

            for unresolved in &result.unresolved_refs {
                if let Some(import_idx) = unresolved.rfind("::") {
                    let import_path = unresolved[..import_idx].replace("::", "/");
                    let symbol = unresolved[import_idx + 2..].to_string();
                    import_hints.push((file_id.clone(), import_path, symbol));
                }
            }
        }

        let mut graph = m1nd_core::graph::Graph::new();
        for node in &all_nodes {
            let tags: Vec<&str> = node.tags.iter().map(String::as_str).collect();
            let _ = graph.add_node(
                &node.id,
                &node.label,
                node.node_type,
                &tags,
                0.0,
                0.0,
            );
        }

        for edge in &all_edges {
            if let (Some(source), Some(target)) = (graph.resolve_id(&edge.source), graph.resolve_id(&edge.target)) {
                let _ = graph.add_edge(
                    source,
                    target,
                    &edge.relation,
                    FiniteF32::new(edge.weight),
                    EdgeDirection::Forward,
                    false,
                    FiniteF32::new(0.0),
                );
            }
        }

        graph.finalize()?;
        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok((graph, stats))
    }
}
