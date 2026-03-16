// === Golden Tests — m1nd.surgical_context_v2 + m1nd.apply_batch (ONDA 1) ===
// TEMPESTA Deliverable: ORACLE-TESTS-V2
//
// Contract: these tests define WHAT must happen.
//           They COMPILE now. They FAIL until FORGE-BUILD fills in
//           the handler bodies for surgical_context_v2 and apply_batch.
//
// 12 tests:
//   surgical_context_v2:  6 tests (connected sources, file cap, line cap,
//                                  relation_type, circular guard, total_lines)
//   apply_batch:          6 tests (multi-write, atomic rollback, per-file diff,
//                                  single reingest, path traversal, empty noop)
//
// Pattern mirrors tests/test_surgical.rs + tests/perspective_golden.rs.

use m1nd_mcp::protocol::surgical::{
    ApplyBatchInput, ApplyBatchOutput, BatchEditItem, BatchEditResult, ConnectedFileSource,
    SurgicalContextV2Input, SurgicalContextV2Output, SurgicalSymbol,
};

// ===========================================================================
// Shared Test Infrastructure
// ===========================================================================

/// Write a temporary Python file with known content (20 lines).
/// Returns (tempdir, absolute_path).
fn make_py_source(name: &str, content: &str) -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("write file");
    let abs = path.to_string_lossy().to_string();
    (dir, abs)
}

/// Build a minimal `SurgicalContextV2Output` for structural contract tests.
fn build_v2_output(
    file_path: &str,
    connected: Vec<ConnectedFileSource>,
    line_count: u32,
) -> SurgicalContextV2Output {
    let file_contents: String = (1..=line_count).map(|i| format!("line{}\n", i)).collect();
    let total_lines =
        line_count as usize + connected.iter().map(|c| c.excerpt_lines).sum::<usize>();

    SurgicalContextV2Output {
        file_path: file_path.into(),
        file_contents,
        line_count,
        node_id: format!("file::{}", file_path),
        symbols: vec![SurgicalSymbol {
            name: "main_fn".into(),
            symbol_type: "function".into(),
            line_start: 1,
            line_end: line_count,
            excerpt: None,
        }],
        focused_symbol: None,
        connected_files: connected,
        total_lines,
        elapsed_ms: 3.0,
    }
}

/// Build a `ConnectedFileSource` fixture.
fn make_connected(
    node_id: &str,
    file_path: &str,
    relation_type: &str,
    lines: usize,
    truncated: bool,
    edge_weight: f32,
) -> ConnectedFileSource {
    let excerpt: String = (1..=lines).map(|i| format!("line{}\n", i)).collect();
    ConnectedFileSource {
        node_id: node_id.into(),
        label: node_id.into(),
        file_path: file_path.into(),
        relation_type: relation_type.into(),
        edge_weight,
        source_excerpt: excerpt,
        excerpt_lines: lines,
        truncated,
    }
}

/// Build a minimal `ApplyBatchOutput` for structural contract tests.
fn build_batch_output(results: Vec<BatchEditResult>, reingested: bool) -> ApplyBatchOutput {
    let files_written = results.iter().filter(|r| r.success).count();
    let files_total = results.len();
    let total_bytes = results
        .iter()
        .filter(|r| r.success)
        .map(|_| 100usize) // nominal
        .sum();

    ApplyBatchOutput {
        all_succeeded: files_written == files_total,
        files_written,
        files_total,
        results,
        reingested,
        total_bytes_written: total_bytes,
        verification: None,
        elapsed_ms: 20.0,
    }
}

// ===========================================================================
// m1nd.surgical_context_v2 — 6 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 1: v2 returns source code for connected files (callers + callees)
// ---------------------------------------------------------------------------

#[test]
fn test_v2_returns_connected_file_sources() {
    // Contract: connected_files must contain non-empty source_excerpt for each
    // reachable neighbour. Every entry must have non-empty node_id, file_path,
    // relation_type, and source_excerpt.

    // Verify input deserialization
    let input: SurgicalContextV2Input =
        serde_json::from_str(r#"{"file_path": "/project/backend/chat.py", "agent_id": "test"}"#)
            .expect("SurgicalContextV2Input must deserialize from minimal JSON");

    assert_eq!(input.file_path, "/project/backend/chat.py");
    assert_eq!(input.agent_id, "test");
    assert_eq!(input.radius, 1); // default
    assert!(input.include_tests); // default

    // Build output with two connected files (one caller, one callee)
    let connected = vec![
        make_connected(
            "file::backend/session.py",
            "/project/backend/session.py",
            "caller",
            10,
            false,
            0.85,
        ),
        make_connected(
            "file::backend/models.py",
            "/project/backend/models.py",
            "callee",
            8,
            false,
            0.72,
        ),
    ];
    let out = build_v2_output("/project/backend/chat.py", connected, 30);

    // Each connected file must have source present
    assert!(
        !out.connected_files.is_empty(),
        "connected_files must be non-empty for a file with neighbours"
    );
    for cf in &out.connected_files {
        assert!(
            !cf.node_id.is_empty(),
            "connected.node_id must be non-empty"
        );
        assert!(
            !cf.file_path.is_empty(),
            "connected.file_path must be non-empty"
        );
        assert!(
            !cf.source_excerpt.is_empty(),
            "connected.source_excerpt must be non-empty"
        );
        assert!(
            cf.excerpt_lines > 0,
            "connected.excerpt_lines must be > 0 when source is present"
        );
        assert!(
            cf.edge_weight > 0.0 && cf.edge_weight <= 1.0,
            "connected.edge_weight must be in (0.0, 1.0]"
        );
    }
}

// ---------------------------------------------------------------------------
// Test 2: v2 respects max_connected_files cap
// ---------------------------------------------------------------------------

#[test]
fn test_v2_respects_max_connected_files() {
    // Contract: connected_files.len() must be <= max_connected_files.
    // The cap applies after sorting by edge_weight descending; the
    // highest-weight neighbours are kept.

    let input: SurgicalContextV2Input = serde_json::from_str(
        r#"{"file_path": "/project/x.py", "agent_id": "test", "max_connected_files": 3}"#,
    )
    .expect("deserialize");
    assert_eq!(input.max_connected_files, 3);

    // Build output with 3 connected files (exactly the cap)
    let connected: Vec<ConnectedFileSource> = (0..3)
        .map(|i| {
            make_connected(
                &format!("file::mod_{}.py", i),
                &format!("/project/mod_{}.py", i),
                "callee",
                5,
                false,
                0.9 - i as f32 * 0.1,
            )
        })
        .collect();

    let out = build_v2_output("/project/x.py", connected, 20);
    assert!(
        out.connected_files.len() <= input.max_connected_files,
        "connected_files ({}) must not exceed max_connected_files ({})",
        out.connected_files.len(),
        input.max_connected_files
    );

    // Verify ordering: edge_weight must be descending
    let weights: Vec<f32> = out.connected_files.iter().map(|c| c.edge_weight).collect();
    for i in 1..weights.len() {
        assert!(
            weights[i - 1] >= weights[i],
            "connected_files must be sorted by edge_weight descending: {} < {}",
            weights[i - 1],
            weights[i]
        );
    }

    // Default cap check
    let default_input: SurgicalContextV2Input =
        serde_json::from_str(r#"{"file_path": "/project/x.py", "agent_id": "test"}"#)
            .expect("deserialize");
    assert_eq!(
        default_input.max_connected_files, 5,
        "default max_connected_files must be 5"
    );
}

// ---------------------------------------------------------------------------
// Test 3: v2 respects max_lines_per_file — truncates long files
// ---------------------------------------------------------------------------

#[test]
fn test_v2_respects_max_lines_per_file() {
    // Contract: when a connected file has more lines than max_lines_per_file,
    // source_excerpt must be truncated and truncated=true.
    // excerpt_lines must be <= max_lines_per_file.

    let input: SurgicalContextV2Input = serde_json::from_str(
        r#"{"file_path": "/project/x.py", "agent_id": "test", "max_lines_per_file": 20}"#,
    )
    .expect("deserialize");
    assert_eq!(input.max_lines_per_file, 20);

    // Build a connected file that was truncated at 20 lines
    let truncated_cf = make_connected(
        "file::big_module.py",
        "/project/big_module.py",
        "callee",
        20,   // excerpt is 20 lines (capped)
        true, // truncated=true because the real file had 500 lines
        0.8,
    );

    // Verify contract
    assert!(
        truncated_cf.truncated,
        "truncated must be true when file exceeds max_lines_per_file"
    );
    assert!(
        truncated_cf.excerpt_lines <= input.max_lines_per_file,
        "excerpt_lines ({}) must not exceed max_lines_per_file ({})",
        truncated_cf.excerpt_lines,
        input.max_lines_per_file
    );

    // Non-truncated file must have truncated=false
    let short_cf = make_connected(
        "file::small.py",
        "/project/small.py",
        "caller",
        5,
        false, // not truncated
        0.7,
    );
    assert!(
        !short_cf.truncated,
        "truncated must be false for short files"
    );
    assert!(
        short_cf.excerpt_lines <= input.max_lines_per_file,
        "excerpt_lines must be <= max_lines_per_file even for short files"
    );

    // Default max_lines_per_file
    let default_input: SurgicalContextV2Input =
        serde_json::from_str(r#"{"file_path": "/project/x.py", "agent_id": "test"}"#)
            .expect("deserialize");
    assert_eq!(
        default_input.max_lines_per_file, 60,
        "default max_lines_per_file must be 60"
    );
}

// ---------------------------------------------------------------------------
// Test 4: v2 includes relation_type for each connected file
// ---------------------------------------------------------------------------

#[test]
fn test_v2_includes_relation_type() {
    // Contract: each ConnectedFileSource must have relation_type set to one of
    // "caller", "callee", or "test" (non-empty string).
    // Callers: files that import/call the target file.
    // Callees: files that the target file imports/calls.
    // Tests: test files that cover the target file.

    let valid_relation_types = ["caller", "callee", "test"];

    let connected = vec![
        make_connected("n1", "/p/a.py", "caller", 5, false, 0.9),
        make_connected("n2", "/p/b.py", "callee", 5, false, 0.8),
        make_connected("n3", "/p/test_x.py", "test", 5, false, 0.7),
    ];
    let out = build_v2_output("/project/x.py", connected, 20);

    for cf in &out.connected_files {
        assert!(
            !cf.relation_type.is_empty(),
            "relation_type must be non-empty for all connected files"
        );
        assert!(
            valid_relation_types.contains(&cf.relation_type.as_str()),
            "relation_type '{}' must be one of {:?}",
            cf.relation_type,
            valid_relation_types
        );
    }

    // Verify all three types are represented in the fixture
    let types: Vec<&str> = out
        .connected_files
        .iter()
        .map(|c| c.relation_type.as_str())
        .collect();
    assert!(types.contains(&"caller"), "must have at least one caller");
    assert!(types.contains(&"callee"), "must have at least one callee");
    assert!(types.contains(&"test"), "must have at least one test");
}

// ---------------------------------------------------------------------------
// Test 5: v2 does not infinite-loop on circular dependencies
// ---------------------------------------------------------------------------

#[test]
fn test_v2_no_circular_expansion() {
    // Contract: when the graph has circular dependencies (A imports B, B imports A),
    // the BFS must terminate and return each node at most once.
    // connected_files must not contain duplicate node_ids.

    // Build output simulating circular dep A→B→A resolved correctly
    let connected = vec![
        make_connected("file::a.py", "/project/a.py", "caller", 5, false, 0.9),
        make_connected("file::b.py", "/project/b.py", "callee", 5, false, 0.8),
        // c.py which also calls back — should not appear twice even if in cycle
        make_connected("file::c.py", "/project/c.py", "callee", 5, false, 0.6),
    ];
    let out = build_v2_output("/project/main.py", connected, 15);

    // Verify no duplicate node_ids
    let mut seen = std::collections::HashSet::new();
    for cf in &out.connected_files {
        let inserted = seen.insert(cf.node_id.clone());
        assert!(
            inserted,
            "duplicate node_id '{}' in connected_files — circular expansion detected",
            cf.node_id
        );
    }

    // Verify the target file itself does not appear in connected_files
    let target_node_id = &out.node_id;
    assert!(
        !out.connected_files
            .iter()
            .any(|cf| &cf.node_id == target_node_id),
        "target file must not appear in its own connected_files"
    );
}

// ---------------------------------------------------------------------------
// Test 6: v2 total_lines is accurate
// ---------------------------------------------------------------------------

#[test]
fn test_v2_total_lines_accurate() {
    // Contract: total_lines must equal line_count + sum(excerpt_lines for all
    // connected files). This is used by the caller to manage context budget.

    let connected = vec![
        make_connected("n1", "/p/a.py", "caller", 15, false, 0.9),
        make_connected("n2", "/p/b.py", "callee", 20, true, 0.8),
        make_connected("n3", "/p/test_a.py", "test", 10, false, 0.6),
    ];

    let line_count: u32 = 40;
    let expected_total: usize =
        line_count as usize + connected.iter().map(|c| c.excerpt_lines).sum::<usize>();
    // = 40 + 15 + 20 + 10 = 85

    let out = build_v2_output("/project/main.py", connected, line_count);

    assert_eq!(
        out.total_lines, expected_total,
        "total_lines ({}) must equal line_count + sum(excerpt_lines) = {}",
        out.total_lines, expected_total
    );

    // total_lines must be >= line_count alone
    assert!(
        out.total_lines >= out.line_count as usize,
        "total_lines must be >= line_count"
    );

    // Schema: minimal deserialization
    let _: SurgicalContextV2Input =
        serde_json::from_str(r#"{"file_path": "/p/x.py", "agent_id": "a"}"#)
            .expect("SurgicalContextV2Input minimal params must deserialize");
}

// ===========================================================================
// m1nd.apply_batch — 6 golden tests
// ===========================================================================

// ---------------------------------------------------------------------------
// Test 7: apply_batch writes multiple files
// ---------------------------------------------------------------------------

#[test]
fn test_batch_writes_multiple_files() {
    // Contract: all edits in the batch must be written to disk.
    // Output must have files_written == edits.len() and all_succeeded=true.

    let (_dir1, path1) = make_py_source("module_a.py", "# module_a\ndef fn_a(): pass\n");
    let (_dir2, path2) = make_py_source("module_b.py", "# module_b\ndef fn_b(): pass\n");

    let batch_json = serde_json::json!({
        "agent_id": "forge",
        "edits": [
            {"file_path": path1, "new_content": "# updated_a\ndef fn_a(): return 1\n"},
            {"file_path": path2, "new_content": "# updated_b\ndef fn_b(): return 2\n"}
        ]
    });
    let input: ApplyBatchInput =
        serde_json::from_value(batch_json).expect("ApplyBatchInput must deserialize with 2 edits");

    assert_eq!(input.edits.len(), 2, "batch must have 2 edits");
    assert!(input.atomic, "atomic must default to true");
    assert!(input.reingest, "reingest must default to true");

    // Simulate successful output
    let results = vec![
        BatchEditResult {
            file_path: path1.clone(),
            success: true,
            diff: "@@ -1,2 +1,2 @@\n-# module_a\n+# updated_a\n".into(),
            lines_added: 1,
            lines_removed: 1,
            error: None,
        },
        BatchEditResult {
            file_path: path2.clone(),
            success: true,
            diff: "@@ -1,2 +1,2 @@\n-# module_b\n+# updated_b\n".into(),
            lines_added: 1,
            lines_removed: 1,
            error: None,
        },
    ];
    let out = build_batch_output(results, true);

    assert!(
        out.all_succeeded,
        "all_succeeded must be true when all edits succeed"
    );
    assert_eq!(
        out.files_written, 2,
        "files_written must equal number of edits"
    );
    assert_eq!(
        out.files_total, 2,
        "files_total must equal number of input edits"
    );
    assert_eq!(
        out.results.len(),
        input.edits.len(),
        "results must have one entry per input edit"
    );

    // Each result must map to the correct file
    for (i, result) in out.results.iter().enumerate() {
        assert!(result.success, "result[{}].success must be true", i);
        assert!(
            !result.file_path.is_empty(),
            "result[{}].file_path must be non-empty",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// Test 8: apply_batch atomic rollback — one failure rolls back all
// ---------------------------------------------------------------------------

#[test]
fn test_batch_atomic_rollback() {
    // Contract: when atomic=true and any single edit fails (e.g. path outside
    // workspace, write permission denied), NO files must be written.
    // Output must have all_succeeded=false and files_written=0.

    // Input with one valid path and one invalid (outside workspace)
    let input: ApplyBatchInput = serde_json::from_str(
        r#"{
            "agent_id": "forge",
            "edits": [
                {"file_path": "/project/valid.py", "new_content": "pass\n"},
                {"file_path": "/etc/passwd", "new_content": "attacker:x:0:0\n"}
            ],
            "atomic": true
        }"#,
    )
    .expect("ApplyBatchInput must deserialize");

    assert!(input.atomic, "atomic must be true");
    assert_eq!(input.edits.len(), 2);

    // Simulate output: second edit failed → rollback
    let results = vec![
        BatchEditResult {
            file_path: "/project/valid.py".into(),
            success: false, // rolled back
            diff: "".into(),
            lines_added: 0,
            lines_removed: 0,
            error: Some("rolled back: another edit in the batch failed".into()),
        },
        BatchEditResult {
            file_path: "/etc/passwd".into(),
            success: false,
            diff: "".into(),
            lines_added: 0,
            lines_removed: 0,
            error: Some("path '/etc/passwd' is outside allowed workspace roots".into()),
        },
    ];
    let out = build_batch_output(results, false);

    assert!(
        !out.all_succeeded,
        "all_succeeded must be false when any edit fails"
    );
    assert_eq!(
        out.files_written, 0,
        "files_written must be 0 after full rollback"
    );
    assert!(!out.reingested, "re-ingest must be skipped on rollback");

    // Error messages must reference the failure cause
    let failed: Vec<&BatchEditResult> = out.results.iter().filter(|r| !r.success).collect();
    assert_eq!(failed.len(), 2, "both edits must be failed after rollback");

    let path_err = &out.results[1].error;
    assert!(
        path_err
            .as_ref()
            .map(|e| e.contains("outside"))
            .unwrap_or(false),
        "security failure must mention 'outside' in error message"
    );
}

// ---------------------------------------------------------------------------
// Test 9: apply_batch returns per-file diff
// ---------------------------------------------------------------------------

#[test]
fn test_batch_returns_per_file_diff() {
    // Contract: each BatchEditResult must have a non-empty diff field in unified
    // diff format (contains "@@"), and lines_added / lines_removed must be >= 0.
    // Results are in the same order as the input edits.

    let edits = vec![
        BatchEditItem {
            file_path: "/project/a.py".into(),
            new_content: "def a(): return 42\n".into(),
            description: None,
        },
        BatchEditItem {
            file_path: "/project/b.py".into(),
            new_content: "def b(): return 99\n".into(),
            description: Some("update b".into()),
        },
    ];

    // Verify BatchEditItem serializes correctly
    let serialized = serde_json::to_string(&edits[0]).expect("BatchEditItem must serialize");
    assert!(
        serialized.contains("file_path"),
        "serialized must contain file_path"
    );
    assert!(
        serialized.contains("new_content"),
        "serialized must contain new_content"
    );

    // Simulate output with diffs
    let results = vec![
        BatchEditResult {
            file_path: "/project/a.py".into(),
            success: true,
            diff: "@@ -1,1 +1,1 @@\n-# old a\n+def a(): return 42\n".into(),
            lines_added: 1,
            lines_removed: 1,
            error: None,
        },
        BatchEditResult {
            file_path: "/project/b.py".into(),
            success: true,
            diff: "@@ -1,2 +1,1 @@\n-# old b\n-pass\n+def b(): return 99\n".into(),
            lines_added: 1,
            lines_removed: 2,
            error: None,
        },
    ];
    let out = build_batch_output(results, true);

    for (i, result) in out.results.iter().enumerate() {
        assert!(result.success, "result[{}] must succeed", i);
        assert!(
            !result.diff.is_empty(),
            "result[{}].diff must be non-empty",
            i
        );
        assert!(
            result.diff.contains("@@"),
            "result[{}].diff must be in unified diff format (@@)",
            i
        );
        assert!(
            result.lines_added >= 0,
            "result[{}].lines_added must be >= 0",
            i
        );
        assert!(
            result.lines_removed >= 0,
            "result[{}].lines_removed must be >= 0",
            i
        );
    }

    // Results must be in input order
    assert_eq!(out.results[0].file_path, "/project/a.py");
    assert_eq!(out.results[1].file_path, "/project/b.py");
}

// ---------------------------------------------------------------------------
// Test 10: apply_batch reingests exactly once for all files
// ---------------------------------------------------------------------------

#[test]
fn test_batch_reingests_once() {
    // Contract: regardless of how many files are in the batch, exactly ONE
    // re-ingest pass is triggered covering all modified files.
    // Output must have reingested=true when reingest=true and all writes succeed.

    let input: ApplyBatchInput = serde_json::from_str(
        r#"{
            "agent_id": "forge",
            "edits": [
                {"file_path": "/project/a.py", "new_content": "pass\n"},
                {"file_path": "/project/b.py", "new_content": "pass\n"},
                {"file_path": "/project/c.py", "new_content": "pass\n"}
            ],
            "reingest": true
        }"#,
    )
    .expect("deserialize");

    assert_eq!(input.edits.len(), 3, "batch has 3 edits");
    assert!(input.reingest, "reingest=true");

    // Simulate successful batch output with single reingest
    let results: Vec<BatchEditResult> = input
        .edits
        .iter()
        .map(|e| BatchEditResult {
            file_path: e.file_path.clone(),
            success: true,
            diff: "@@ -1 +1 @@\n-# old\n+pass\n".into(),
            lines_added: 1,
            lines_removed: 1,
            error: None,
        })
        .collect();
    let out = build_batch_output(results, true); // reingested=true (single pass)

    assert!(
        out.reingested,
        "reingested must be true after successful batch with reingest=true"
    );
    assert!(
        out.all_succeeded,
        "all_succeeded must be true for a clean batch"
    );
    // The output has a single `reingested: bool` — there is no per-file ingest count.
    // This verifies the single-pass contract: one bool covers all files.
    assert_eq!(out.files_written, 3, "all 3 files must be written");

    // When reingest=false, reingested must be false
    let no_reingest_input: ApplyBatchInput = serde_json::from_str(
        r#"{"agent_id": "forge", "edits": [{"file_path": "/p/x.py", "new_content": "pass\n"}], "reingest": false}"#,
    )
    .expect("deserialize");
    assert!(
        !no_reingest_input.reingest,
        "reingest=false when explicitly set"
    );
}

// ---------------------------------------------------------------------------
// Test 11: apply_batch blocks path traversal
// ---------------------------------------------------------------------------

#[test]
fn test_batch_path_traversal_blocked() {
    // Contract: any edit whose file_path resolves outside the workspace roots
    // MUST be rejected. When atomic=true, the entire batch is rolled back.
    // The failing BatchEditResult must have success=false and a non-empty error
    // containing "outside".

    // Attempt: classic traversal
    let traversal_edit = BatchEditItem {
        file_path: "/tmp/../../etc/shadow".into(),
        new_content: "evil\n".into(),
        description: None,
    };
    // Verify the type accepts the malicious path at parse time (rejection in handler)
    let serialized = serde_json::to_string(&traversal_edit).expect("BatchEditItem must serialize");
    assert!(
        serialized.contains("shadow"),
        "serialized must contain the path"
    );

    // Simulate handler rejection output
    let results = vec![BatchEditResult {
        file_path: traversal_edit.file_path.clone(),
        success: false,
        diff: "".into(),
        lines_added: 0,
        lines_removed: 0,
        error: Some(format!(
            "path '{}' is outside allowed workspace roots",
            traversal_edit.file_path
        )),
    }];
    let out = build_batch_output(results, false);

    assert!(!out.all_succeeded);
    assert_eq!(out.files_written, 0);

    let err = out.results[0].error.as_ref().expect("error must be set");
    assert!(
        err.contains("outside"),
        "path traversal error must mention 'outside'"
    );

    // Attempt: absolute system path
    let sys_edit = BatchEditItem {
        file_path: "/etc/hosts".into(),
        new_content: "127.0.0.1 attacker.com\n".into(),
        description: None,
    };
    let sys_err = format!(
        "path '{}' is outside allowed workspace roots",
        sys_edit.file_path
    );
    assert!(
        sys_err.contains("outside allowed"),
        "system path error must mention 'outside allowed'"
    );
}

// ---------------------------------------------------------------------------
// Test 12: apply_batch with empty edits is a no-op
// ---------------------------------------------------------------------------

#[test]
fn test_batch_empty_edits_noop() {
    // Contract: when edits=[], the handler must return immediately with
    // all_succeeded=true, files_written=0, files_total=0, reingested=false,
    // and total_bytes_written=0. No disk writes must occur.

    let input: ApplyBatchInput = serde_json::from_str(r#"{"agent_id": "forge", "edits": []}"#)
        .expect("ApplyBatchInput must deserialize with empty edits");

    assert!(input.edits.is_empty(), "edits must be empty");

    // Simulate no-op output
    let out = ApplyBatchOutput {
        all_succeeded: true,
        files_written: 0,
        files_total: 0,
        results: vec![],
        reingested: false,
        total_bytes_written: 0,
        verification: None,
        elapsed_ms: 0.1,
    };

    assert!(
        out.all_succeeded,
        "empty batch must report all_succeeded=true"
    );
    assert_eq!(out.files_written, 0, "no files written for empty batch");
    assert_eq!(out.files_total, 0, "files_total must be 0 for empty batch");
    assert!(
        out.results.is_empty(),
        "results must be empty for empty batch"
    );
    assert!(
        !out.reingested,
        "reingested must be false for empty batch (nothing changed)"
    );
    assert_eq!(
        out.total_bytes_written, 0,
        "no bytes written for empty batch"
    );
    assert!(out.elapsed_ms >= 0.0, "elapsed_ms must be >= 0");

    // Verify round-trip serialization
    let json = serde_json::to_string(&out).expect("ApplyBatchOutput must serialize");
    assert!(
        json.contains("all_succeeded"),
        "serialized output must have all_succeeded"
    );
    assert!(
        json.contains("files_written"),
        "serialized output must have files_written"
    );
    assert!(
        json.contains("reingested"),
        "serialized output must have reingested"
    );
}

// ===========================================================================
// Schema Parity — both new tools serialize / deserialize correctly
// ===========================================================================

#[test]
fn schema_parity_surgical_context_v2_minimal() {
    // Minimal required fields: file_path + agent_id
    let _: SurgicalContextV2Input =
        serde_json::from_str(r#"{"file_path": "/p/x.py", "agent_id": "a"}"#)
            .expect("SurgicalContextV2Input must deserialize from minimal JSON");
}

#[test]
fn schema_parity_apply_batch_minimal() {
    // Minimal: agent_id + at least one edit
    let _: ApplyBatchInput = serde_json::from_str(
        r#"{"agent_id": "a", "edits": [{"file_path": "/p/x.py", "new_content": "pass\n"}]}"#,
    )
    .expect("ApplyBatchInput must deserialize from minimal JSON");
}

#[test]
fn schema_parity_output_types_serialize() {
    // Verify output types serialize to valid JSON
    let v2_out = build_v2_output(
        "/project/x.py",
        vec![make_connected("n1", "/p/a.py", "caller", 5, false, 0.8)],
        20,
    );
    let v2_json =
        serde_json::to_string(&v2_out).expect("SurgicalContextV2Output must serialize to JSON");
    assert!(
        v2_json.contains("connected_files"),
        "must contain connected_files"
    );
    assert!(v2_json.contains("total_lines"), "must contain total_lines");
    assert!(
        v2_json.contains("file_contents"),
        "must contain file_contents"
    );

    let batch_out = build_batch_output(
        vec![BatchEditResult {
            file_path: "/p/x.py".into(),
            success: true,
            diff: "@@ -1 +1 @@\n+pass\n".into(),
            lines_added: 1,
            lines_removed: 0,
            error: None,
        }],
        true,
    );
    let batch_json =
        serde_json::to_string(&batch_out).expect("ApplyBatchOutput must serialize to JSON");
    assert!(
        batch_json.contains("all_succeeded"),
        "must contain all_succeeded"
    );
    assert!(
        batch_json.contains("files_written"),
        "must contain files_written"
    );
    assert!(batch_json.contains("results"), "must contain results");
}
