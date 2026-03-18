# Embeddings Search Report -- Where OpenCode Implemented Embeddings for m1nd

**Date**: 2026-03-15
**Searcher**: oracle-search agent
**Query**: Find where OpenCode implemented embeddings in m1nd

---

## FOUND: OpenCode Session `ses_311ab6ec0ffed1e68EcKHwqsYx`

**Title**: "Configure m1nd embeddings"
**Timestamp**: 2026-03-14 (~22:50 local)
**Model**: claude-sonnet-4-6 via Anthropic
**Messages**: 24 (all tool-call chain, single user prompt)
**Total cost**: ~$0.59

---

## What Was Implemented

### Single File Modified

**File**: `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-mcp/src/layer_handlers.rs`

The change wired the existing `SemanticEngine` (char n-gram TF-IDF + DeepWalk-lite co-occurrence) into `handle_seek`, which previously ran only inline keyword matching and trigram similarity.

### Before (Original Code)

```rust
// Phase 2: Combine with graph re-ranking.
// V1 formula: keyword_match * 0.6 + graph_activation(PageRank) * 0.3 + trigram * 0.1
let combined = kw * 0.6 + graph_activation * 0.3 + tri * 0.1;
// ...
embeddings_used: false,  // hardcoded
```

### After (OpenCode's Fix)

```rust
// Phase 2: SemanticEngine scores (trigram TF-IDF + co-occurrence).
let semantic_scores: HashMap<usize, f32> = {
    let sem_results = state.orchestrator.semantic
        .query(&*graph, &input.query, input.top_k * 5)
        .unwrap_or_default();
    sem_results.into_iter()
        .map(|(nid, score)| (nid.as_usize(), score.get()))
        .collect()
};
let semantic_used = !semantic_scores.is_empty();

// Phase 3: Updated formula
// V2 formula: keyword * 0.4 + semantic * 0.3 + PageRank * 0.2 + trigram * 0.1
let combined = kw * 0.4 + sem * 0.3 + graph_activation * 0.2 + tri * 0.1;
// ...
embeddings_used: semantic_used,  // dynamic, true when SemanticEngine returns results
```

### Key Findings from the Session

1. **`embeddings_used` was hardcoded `false`** at lines 54 and 217 of `layer_handlers.rs`
2. The `SemanticEngine` already existed in `state.orchestrator.semantic` but was never called from `handle_seek`
3. After ingest, `rebuild_engines()` properly rebuilds the SemanticEngine -- so it was ready but unused
4. On tiny graphs (<10 nodes), the SemanticEngine returns empty scores (correct behavior -- not enough structure for co-occurrence)
5. On real codebases, `embeddings_used: true` is correctly reported

---

## What Was NOT Implemented (Still Pending)

### fastembed-rs / Neural Embeddings

**No neural embedding model was added.** The "embeddings" OpenCode wired in are the existing graph-native semantic features:

- **CharNgramIndex**: TF-IDF weighted trigram similarity with inverted index (already in `m1nd-core/src/semantic.rs`)
- **CoOccurrenceIndex**: DeepWalk-lite random walk embeddings with PPMI normalization (same file)
- **SynonymExpander**: Bidirectional synonym groups (15 groups)

The `fastembed-rs` integration with real neural models (jina-embeddings-v2-base-code, 768-dim) is still on the roadmap as a v0.3.0 feature. The planned issues are documented at:

- `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/.github/PLANNED_ISSUES.md` (Issue #1)
- `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/.github/wiki/Roadmap.md` (v0.3 section)

### Cargo.toml Unchanged

Neither `m1nd-core/Cargo.toml` nor `m1nd-mcp/Cargo.toml` were modified to add `fastembed`, `ort`, `candle`, or any neural embedding dependencies.

---

## Related Files and Research

### Existing Semantic Engine Implementation
- `/Users/cosmophonix/clawd/roomanizer-os/mcp/m1nd/m1nd-core/src/semantic.rs` -- 700 lines, CharNgramIndex + CoOccurrenceIndex + SynonymExpander + SemanticEngine

### L2 Semantic Search Research (comprehensive)
- `/Users/cosmophonix/clawd/roomanizer-os/docs/m1nd/research/L2-SEMANTIC-SEARCH.md` -- Full research report comparing fastembed-rs, Voyage Code 3, OpenAI embeddings, ast-grep, tree-sitter queries. Recommends hybrid approach with fastembed-rs local + Voyage API for re-ranking.

### Ollama Embeddings (Python, backend)
- `/Users/cosmophonix/clawd/roomanizer-os/backend/ollama_client.py` -- Python embeddings via Ollama/nomic-embed-text (768-dim), separate from m1nd's Rust engine

### OpenCode Session Artifacts
- Session DB: `/Users/cosmophonix/.local/share/opencode/opencode.db` (table: `session`, id: `ses_311ab6ec0ffed1e68EcKHwqsYx`)
- Session diff: `/Users/cosmophonix/.local/share/opencode/storage/session_diff/ses_311ab6ec0ffed1e68EcKHwqsYx.json` (1.8MB)
- Binary was recompiled: `mcp/m1nd/target/release/m1nd-mcp` and `mcp/m1nd/target/release/libm1nd_mcp.rlib`

---

## Integration Recommendation

To add real neural embeddings (fastembed-rs), the L2 research report recommends:

1. **Add `fastembed` 5.12.1 to `m1nd-core/Cargo.toml`** with `ort` for ONNX inference
2. **New fields in `NodeStorage`**: `intent_summary: Vec<Option<InternedStr>>` and `embedding: Vec<Option<Vec<f32>>>`
3. **Heuristic intent summary generation** during ingest (split identifiers, add tags, file path context)
4. **Brute-force cosine search** initially (1ms for 10K nodes at 768-dim), upgrade to `usearch` HNSW at >50K nodes
5. **Combined scoring**: `embedding * 0.5 + existing_semantic * 0.3 + graph_activation * 0.2`

This is estimated at ~10 days of agent work for a working `m1nd.seek` with real embeddings, plus ~10 more for `m1nd.scan` with ast-grep patterns.
