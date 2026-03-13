// === crates/m1nd-ingest/src/cross_file.rs ===
//
// Layer 1: Cross-File Edge Resolution
//
// Post-ingest pass that resolves ref:: edges into deterministic
// file-to-file cross-file edges. This is the keystone layer --
// without it, every file in the graph is an island.
//
// Edge types produced:
//   "imports"    — file A imports module from file B
//   "tests"      — test file A tests module B (naming convention)
//   "registers"  — A registers B as a route/plugin (include_router)
//
// Zero new dependencies -- uses only std + existing Graph API.

use std::collections::HashMap;
use std::path::Path;

use m1nd_core::error::M1ndResult;
use m1nd_core::graph::Graph;
use m1nd_core::types::*;
use regex::Regex;

// ---------------------------------------------------------------------------
// CrossFileStats — result statistics for the cross-file resolution pass
// ---------------------------------------------------------------------------

/// Statistics from cross-file edge resolution.
#[derive(Clone, Debug, Default)]
pub struct CrossFileStats {
    /// Total import edges resolved (file-to-file).
    pub imports_resolved: u64,
    /// Total import refs that could not be resolved (stdlib/third-party).
    pub imports_unresolved: u64,
    /// Test-to-module edges created via naming convention.
    pub test_edges_created: u64,
    /// Route registration edges created (include_router).
    pub register_edges_created: u64,
    /// Total cross-file edges created (sum of all types).
    pub total_edges_created: u64,
    /// Python files indexed in the module index.
    pub files_indexed: u64,
}

// ---------------------------------------------------------------------------
// PythonModuleIndex — maps module paths to file paths
// ---------------------------------------------------------------------------

/// Maps Python dotted module paths to file external IDs.
///
/// Built from discovered files during walk phase. Handles:
/// - Dotted paths: `backend.config` -> `file::backend/config.py`
/// - Bare names: `config` -> `file::backend/config.py` (flat layout)
/// - Package inits: `backend` -> `file::backend/__init__.py`
///
/// Flat Python projects (no `__init__.py` at root) need bare module
/// names like `config` to resolve directly to `config.py` within the
/// source directory.
struct PythonModuleIndex {
    /// dotted.module.path -> file external ID ("file::relative/path.py")
    module_to_file: HashMap<String, String>,
}

impl PythonModuleIndex {
    /// Build the module index from all file external IDs in the graph.
    ///
    /// Scans existing file nodes (NodeType::File) whose IDs end in `.py`.
    /// Registers each under multiple lookup keys:
    /// 1. Full dotted path: `backend.config` (from `backend/config.py`)
    /// 2. Bare filename (no extension): `config` (for flat-layout resolution)
    /// 3. Package init: `backend` (from `backend/__init__.py`)
    fn build(graph: &Graph) -> Self {
        let mut module_to_file: HashMap<String, String> = HashMap::new();

        for i in 0..graph.num_nodes() as usize {
            if graph.nodes.node_type[i] != NodeType::File {
                continue;
            }

            // Recover the external ID for this node
            let ext_id = match find_external_id(graph, NodeId::new(i as u32)) {
                Some(id) => id,
                None => continue,
            };

            let rel_path = match ext_id.strip_prefix("file::") {
                Some(p) => p,
                None => continue,
            };

            if !rel_path.ends_with(".py") {
                continue;
            }

            // Strip .py extension
            let without_ext = &rel_path[..rel_path.len() - 3];

            // Full dotted path: "backend/config" -> "backend.config"
            let dotted = without_ext.replace('/', ".");

            // Handle __init__.py: "backend/__init__" -> package "backend"
            if dotted.ends_with(".__init__") {
                let package = &dotted[..dotted.len() - 9]; // strip ".__init__"
                module_to_file.insert(package.to_string(), ext_id.clone());
                continue;
            }

            // Register full dotted path
            module_to_file.insert(dotted.clone(), ext_id.clone());

            // Register bare filename for flat-layout resolution.
            // "backend.config" -> also register "config" if it does not
            // collide with an existing entry.
            if let Some(bare) = without_ext.rsplit('/').next() {
                // Only register bare name if it is different from the dotted path
                // (i.e., the file is nested, not already at root level).
                if bare != dotted {
                    // Use entry API: first registration wins.
                    // In flat layouts with a single source dir, this ensures
                    // `config` -> `backend/config.py` is deterministic.
                    module_to_file.entry(bare.to_string()).or_insert_with(|| ext_id.clone());
                }
            }
        }

        Self { module_to_file }
    }

    /// Resolve a Python import path to a file external ID.
    ///
    /// Tries exact match first, then progressively shorter prefixes
    /// to handle `from backend.config import Settings` -> resolve
    /// `backend.config` even if the ref was `backend.config.Settings`.
    ///
    /// Returns None for stdlib/third-party modules not in the project.
    fn resolve(&self, import_path: &str) -> Option<&str> {
        // Exact match: "config" or "backend.config"
        if let Some(file_id) = self.module_to_file.get(import_path) {
            return Some(file_id.as_str());
        }

        // Progressively shorter prefixes for dotted paths:
        // "backend.config.Settings" -> try "backend.config" -> "backend"
        if import_path.contains('.') {
            let mut parts: Vec<&str> = import_path.split('.').collect();
            while parts.len() > 1 {
                parts.pop();
                let prefix = parts.join(".");
                if let Some(file_id) = self.module_to_file.get(&prefix) {
                    return Some(file_id.as_str());
                }
            }
            // Also try the last segment alone (bare name fallback)
            if let Some(last) = import_path.rsplit('.').next() {
                if let Some(file_id) = self.module_to_file.get(last) {
                    return Some(file_id.as_str());
                }
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// resolve_cross_file_edges — main entry point
// ---------------------------------------------------------------------------

/// Post-ingest pass that resolves ref:: edges into deterministic
/// file-to-file cross-file edges.
///
/// Must be called AFTER all nodes and intra-file edges have been added
/// to the graph, but BEFORE finalize().
///
/// Produces three edge types:
/// 1. **imports** — from import ref:: edges already in the graph
/// 2. **tests** — from test file naming convention (test_X.py -> X.py)
/// 3. **registers** — from include_router() patterns in source files
///
/// All edges are Forward direction with appropriate causal strengths.
pub fn resolve_cross_file_edges(
    graph: &mut Graph,
    root: &Path,
) -> M1ndResult<CrossFileStats> {
    let mut stats = CrossFileStats::default();

    // Step 1: Build the module index from file nodes already in the graph.
    let module_index = PythonModuleIndex::build(graph);
    stats.files_indexed = module_index.module_to_file.len() as u64;

    // Step 2: Collect existing import ref:: edges from pending edges.
    // We scan the pending edges (pre-finalize) looking for "imports" relation
    // edges whose targets are ref:: prefixed. Since ref:: edges were skipped
    // during graph building (lib.rs:296-299), we need a different approach:
    // scan the extraction results. But we do not have access to extraction
    // results here -- we only have the graph.
    //
    // Alternative approach: scan all existing file nodes and reconstruct
    // import relationships by examining the source files on disk.
    // This is cleaner and avoids coupling to the extraction pipeline.
    let import_edges = collect_import_edges_from_files(graph, root, &module_index);

    for (source_file_id, target_file_id, relation) in &import_edges {
        if add_cross_file_edge(graph, source_file_id, target_file_id, relation, &mut stats) {
            stats.imports_resolved += 1;
        } else {
            stats.imports_unresolved += 1;
        }
    }

    // Step 3: Test-to-module edges via naming convention.
    let test_edges = infer_test_edges(graph, &module_index);
    for (test_file_id, source_file_id) in &test_edges {
        if add_cross_file_edge(graph, test_file_id, source_file_id, "tests", &mut stats) {
            stats.test_edges_created += 1;
        }
    }

    // Step 4: Route registration edges from include_router() in source.
    let register_edges = detect_route_registrations(graph, root, &module_index);
    for (main_file_id, route_file_id) in &register_edges {
        if add_cross_file_edge(graph, main_file_id, route_file_id, "registers", &mut stats) {
            stats.register_edges_created += 1;
        }
    }

    Ok(stats)
}

// ---------------------------------------------------------------------------
// Import edge collection — scan Python files for import statements
// ---------------------------------------------------------------------------

/// Scan all Python file nodes in the graph, read their source from disk,
/// extract import statements, and resolve them to file-to-file edges.
///
/// Returns Vec<(source_file_id, target_file_id, relation)>.
fn collect_import_edges_from_files(
    graph: &Graph,
    root: &Path,
    module_index: &PythonModuleIndex,
) -> Vec<(String, String, String)> {
    let re_import = Regex::new(r"^\s*import\s+([\w.]+)").unwrap();
    let re_from_import = Regex::new(r"^\s*from\s+([\w.]+)\s+import").unwrap();

    let mut edges: Vec<(String, String, String)> = Vec::new();
    // Track (source, target) pairs to avoid duplicate edges
    let mut seen: HashMap<(String, String), bool> = HashMap::new();

    // Skip list: imports that should not generate cross-file edges
    let skip_modules: &[&str] = &[
        "__future__", "typing", "typing_extensions",
        "abc", "collections", "dataclasses", "enum", "functools",
        "os", "sys", "io", "re", "json", "time", "datetime",
        "pathlib", "logging", "asyncio", "contextlib", "traceback",
        "uuid", "hashlib", "base64", "copy", "math", "random",
        "subprocess", "shutil", "tempfile", "signal", "socket",
        "urllib", "http", "ssl", "inspect", "importlib",
        "threading", "multiprocessing", "concurrent",
        "unittest", "pytest", "textwrap", "string",
    ];

    for i in 0..graph.num_nodes() as usize {
        if graph.nodes.node_type[i] != NodeType::File {
            continue;
        }

        let ext_id = match find_external_id(graph, NodeId::new(i as u32)) {
            Some(id) => id,
            None => continue,
        };

        let rel_path = match ext_id.strip_prefix("file::") {
            Some(p) if p.ends_with(".py") => p,
            _ => continue,
        };

        // Read the source file from disk
        let file_path = root.join(rel_path);
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let module_path = if let Some(caps) = re_from_import.captures(trimmed) {
                caps.get(1).unwrap().as_str().to_string()
            } else if let Some(caps) = re_import.captures(trimmed) {
                caps.get(1).unwrap().as_str().to_string()
            } else {
                continue;
            };

            // Skip stdlib/third-party modules
            let first_segment = module_path.split('.').next().unwrap_or(&module_path);
            if skip_modules.contains(&first_segment) {
                continue;
            }

            // Resolve to file
            if let Some(target_file_id) = module_index.resolve(&module_path) {
                // Skip self-imports
                if target_file_id == ext_id {
                    continue;
                }

                let key = (ext_id.clone(), target_file_id.to_string());
                if !seen.contains_key(&key) {
                    seen.insert(key, true);
                    edges.push((
                        ext_id.clone(),
                        target_file_id.to_string(),
                        "imports".to_string(),
                    ));
                }
            }
        }
    }

    edges
}

// ---------------------------------------------------------------------------
// Test-to-module inference — naming convention
// ---------------------------------------------------------------------------

/// Infer test-to-module edges from the naming convention:
///   `test_X.py` -> `X.py`
///   `tests/test_X.py` -> `X.py`
///   `backend/tests/test_X.py` -> `backend/X.py`
///
/// Returns Vec<(test_file_id, source_file_id)>.
fn infer_test_edges(
    graph: &Graph,
    module_index: &PythonModuleIndex,
) -> Vec<(String, String)> {
    let mut edges = Vec::new();

    for i in 0..graph.num_nodes() as usize {
        if graph.nodes.node_type[i] != NodeType::File {
            continue;
        }

        let ext_id = match find_external_id(graph, NodeId::new(i as u32)) {
            Some(id) => id,
            None => continue,
        };

        let rel_path = match ext_id.strip_prefix("file::") {
            Some(p) if p.ends_with(".py") => p,
            _ => continue,
        };

        // Extract the bare filename without extension
        let filename = match Path::new(rel_path).file_stem().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };

        // Must start with "test_"
        let target_module = match filename.strip_prefix("test_") {
            Some(t) if !t.is_empty() => t,
            _ => continue,
        };

        // Try to resolve the tested module
        if let Some(target_file_id) = module_index.resolve(target_module) {
            // Skip if test file points to itself somehow
            if target_file_id != ext_id {
                edges.push((ext_id.clone(), target_file_id.to_string()));
            }
        }
    }

    edges
}

// ---------------------------------------------------------------------------
// Route registration detection — include_router() pattern
// ---------------------------------------------------------------------------

/// Detect FastAPI route registrations via `include_router()` calls.
///
/// Scans Python files for patterns like:
///   `app.include_router(module_name.router)`
///   `router.include_router(sub_router.router)`
///
/// Returns Vec<(main_file_id, route_file_id)>.
fn detect_route_registrations(
    graph: &Graph,
    root: &Path,
    module_index: &PythonModuleIndex,
) -> Vec<(String, String)> {
    // Match: app.include_router(module_name.router)
    // Captures the module name before the dot
    let re_include_router = Regex::new(
        r"(?:app|router)\s*\.\s*include_router\s*\(\s*(\w+)\s*\."
    ).unwrap();

    let mut edges = Vec::new();
    let mut seen: HashMap<(String, String), bool> = HashMap::new();

    for i in 0..graph.num_nodes() as usize {
        if graph.nodes.node_type[i] != NodeType::File {
            continue;
        }

        let ext_id = match find_external_id(graph, NodeId::new(i as u32)) {
            Some(id) => id,
            None => continue,
        };

        let rel_path = match ext_id.strip_prefix("file::") {
            Some(p) if p.ends_with(".py") => p,
            _ => continue,
        };

        // Read the source file
        let file_path = root.join(rel_path);
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Scan for include_router calls
        for line in content.lines() {
            for caps in re_include_router.captures_iter(line) {
                let module_name = caps.get(1).unwrap().as_str();

                // Resolve the module name to a file
                if let Some(target_file_id) = module_index.resolve(module_name) {
                    if target_file_id != ext_id {
                        let key = (ext_id.clone(), target_file_id.to_string());
                        if !seen.contains_key(&key) {
                            seen.insert(key, true);
                            edges.push((ext_id.clone(), target_file_id.to_string()));
                        }
                    }
                }
            }
        }
    }

    edges
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Add a cross-file edge between two file nodes.
///
/// Returns true if the edge was successfully added, false if either
/// node could not be resolved.
fn add_cross_file_edge(
    graph: &mut Graph,
    source_id: &str,
    target_id: &str,
    relation: &str,
    stats: &mut CrossFileStats,
) -> bool {
    let source = match graph.resolve_id(source_id) {
        Some(id) => id,
        None => return false,
    };
    let target = match graph.resolve_id(target_id) {
        Some(id) => id,
        None => return false,
    };

    // Causal strength and weight depend on edge type:
    // - imports: moderate causal (dependency changes -> importer affected)
    // - tests: moderate causal (test verifies module behavior)
    // - registers: strong causal (registration is explicit wiring)
    let (weight, causal_strength) = match relation {
        "imports" => (0.6, 0.6),
        "tests" => (0.5, 0.5),
        "registers" => (0.7, 0.7),
        _ => (0.5, 0.4),
    };

    match graph.add_edge(
        source,
        target,
        relation,
        FiniteF32::new(weight),
        EdgeDirection::Forward,
        false,
        FiniteF32::new(causal_strength),
    ) {
        Ok(_) => {
            stats.total_edges_created += 1;
            true
        }
        Err(_) => false,
    }
}

/// Find the external ID string for a node.
///
/// Iterates the id_to_node map to find the interned string matching
/// the given NodeId. Returns None if not found.
fn find_external_id(graph: &Graph, node: NodeId) -> Option<String> {
    for (interned, &nid) in &graph.id_to_node {
        if nid == node {
            return Some(graph.strings.resolve(*interned).to_string());
        }
    }
    None
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use m1nd_core::graph::Graph;
    use m1nd_core::types::*;

    /// Helper: add a Python file node to the graph.
    fn add_file_node(graph: &mut Graph, rel_path: &str) -> NodeId {
        let ext_id = format!("file::{}", rel_path);
        let label = rel_path.rsplit('/').next().unwrap_or(rel_path);
        graph.add_node(
            &ext_id,
            label,
            NodeType::File,
            &["python"],
            0.0,
            0.3,
        ).unwrap()
    }

    // -----------------------------------------------------------------------
    // PythonModuleIndex tests
    // -----------------------------------------------------------------------

    #[test]
    fn module_index_resolves_dotted_path() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/config.py");
        add_file_node(&mut graph, "backend/models.py");

        let index = PythonModuleIndex::build(&graph);

        assert_eq!(
            index.resolve("backend.config"),
            Some("file::backend/config.py"),
            "Dotted path should resolve to file"
        );
        assert_eq!(
            index.resolve("backend.models"),
            Some("file::backend/models.py"),
        );
    }

    #[test]
    fn module_index_resolves_bare_name_flat_layout() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/config.py");
        add_file_node(&mut graph, "backend/worker.py");

        let index = PythonModuleIndex::build(&graph);

        // Bare names should resolve (flat layout, no __init__.py)
        assert_eq!(
            index.resolve("config"),
            Some("file::backend/config.py"),
            "Bare module name should resolve in flat layout"
        );
        assert_eq!(
            index.resolve("worker"),
            Some("file::backend/worker.py"),
        );
    }

    #[test]
    fn module_index_resolves_dotted_name_import() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/config.py");

        let index = PythonModuleIndex::build(&graph);

        // "backend.config.Settings" should resolve to backend/config.py
        // via progressive prefix shortening
        assert_eq!(
            index.resolve("backend.config.Settings"),
            Some("file::backend/config.py"),
            "Dotted name with attribute should resolve via prefix"
        );
    }

    #[test]
    fn module_index_handles_init_py() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/__init__.py");
        add_file_node(&mut graph, "backend/config.py");

        let index = PythonModuleIndex::build(&graph);

        // Package init should resolve as the package name
        assert_eq!(
            index.resolve("backend"),
            Some("file::backend/__init__.py"),
            "__init__.py should register under package name"
        );
        assert_eq!(
            index.resolve("backend.config"),
            Some("file::backend/config.py"),
        );
    }

    #[test]
    fn module_index_returns_none_for_stdlib() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/config.py");

        let index = PythonModuleIndex::build(&graph);

        assert_eq!(index.resolve("os"), None, "stdlib should not resolve");
        assert_eq!(index.resolve("json"), None, "stdlib should not resolve");
        assert_eq!(index.resolve("fastapi"), None, "third-party should not resolve");
    }

    #[test]
    fn module_index_root_level_files() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "main.py");
        add_file_node(&mut graph, "config.py");

        let index = PythonModuleIndex::build(&graph);

        // Root-level files: dotted path equals bare name
        assert_eq!(
            index.resolve("main"),
            Some("file::main.py"),
        );
        assert_eq!(
            index.resolve("config"),
            Some("file::config.py"),
        );
    }

    // -----------------------------------------------------------------------
    // Test-to-module inference tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_edge_naming_convention() {
        let mut graph = Graph::with_capacity(8, 8);
        add_file_node(&mut graph, "backend/config.py");
        add_file_node(&mut graph, "backend/doctor.py");
        add_file_node(&mut graph, "backend/tests/test_config.py");
        add_file_node(&mut graph, "backend/tests/test_doctor.py");
        add_file_node(&mut graph, "backend/tests/test_e2e_integration.py"); // no match

        let index = PythonModuleIndex::build(&graph);
        let edges = infer_test_edges(&graph, &index);

        // Should find test_config -> config and test_doctor -> doctor
        assert!(
            edges.iter().any(|(t, s)| t.contains("test_config") && s.contains("config.py") && !s.contains("test_")),
            "test_config.py should map to config.py. Edges: {:?}", edges
        );
        assert!(
            edges.iter().any(|(t, s)| t.contains("test_doctor") && s.contains("doctor.py") && !s.contains("test_")),
            "test_doctor.py should map to doctor.py"
        );
        // test_e2e_integration.py should NOT match anything
        assert!(
            !edges.iter().any(|(t, _)| t.contains("test_e2e_integration")),
            "Integration test with no matching module should not create edge"
        );
    }

    // -----------------------------------------------------------------------
    // Cross-file edge addition test
    // -----------------------------------------------------------------------

    #[test]
    fn cross_file_edge_adds_to_graph() {
        let mut graph = Graph::with_capacity(4, 8);
        add_file_node(&mut graph, "backend/main.py");
        add_file_node(&mut graph, "backend/config.py");

        let initial_pending = graph.csr.pending_edges.len();
        let mut stats = CrossFileStats::default();

        let result = add_cross_file_edge(
            &mut graph,
            "file::backend/main.py",
            "file::backend/config.py",
            "imports",
            &mut stats,
        );

        assert!(result, "Edge should be added successfully");
        assert_eq!(stats.total_edges_created, 1);
        assert_eq!(graph.csr.pending_edges.len(), initial_pending + 1);

        // Verify edge properties
        let edge = &graph.csr.pending_edges[initial_pending];
        assert_eq!(edge.weight.get(), 0.6);
        assert_eq!(edge.causal_strength.get(), 0.6);
        assert_eq!(edge.direction, EdgeDirection::Forward);
        assert!(!edge.inhibitory);
    }

    #[test]
    fn cross_file_edge_fails_for_missing_node() {
        let mut graph = Graph::with_capacity(4, 4);
        add_file_node(&mut graph, "backend/main.py");

        let mut stats = CrossFileStats::default();
        let result = add_cross_file_edge(
            &mut graph,
            "file::backend/main.py",
            "file::nonexistent.py",
            "imports",
            &mut stats,
        );

        assert!(!result, "Edge should fail for non-existent target");
        assert_eq!(stats.total_edges_created, 0);
    }

    #[test]
    fn edge_weights_vary_by_relation() {
        let mut graph = Graph::with_capacity(6, 12);
        add_file_node(&mut graph, "a.py");
        add_file_node(&mut graph, "b.py");
        add_file_node(&mut graph, "c.py");

        let mut stats = CrossFileStats::default();

        // imports edge
        add_cross_file_edge(&mut graph, "file::a.py", "file::b.py", "imports", &mut stats);
        let import_edge = &graph.csr.pending_edges[0];
        assert!((import_edge.weight.get() - 0.6).abs() < f32::EPSILON);

        // tests edge
        add_cross_file_edge(&mut graph, "file::a.py", "file::c.py", "tests", &mut stats);
        let test_edge = &graph.csr.pending_edges[1];
        assert!((test_edge.weight.get() - 0.5).abs() < f32::EPSILON);

        // registers edge
        add_cross_file_edge(&mut graph, "file::b.py", "file::c.py", "registers", &mut stats);
        let reg_edge = &graph.csr.pending_edges[2];
        assert!((reg_edge.weight.get() - 0.7).abs() < f32::EPSILON);
    }
}
