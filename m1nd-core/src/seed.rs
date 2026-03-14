// === crates/m1nd-core/src/seed.rs ===

use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum seeds returned (FM-ACT-017: cap for broad queries).
const MAX_SEEDS: usize = 200;
/// Minimum relevance to include a seed.
const MIN_RELEVANCE: f32 = 0.1;
/// Relevance scores by match type.
const EXACT_MATCH_RELEVANCE: f32 = 1.0;
const PREFIX_MATCH_RELEVANCE: f32 = 0.9;
const TAG_MATCH_RELEVANCE: f32 = 0.85;
const FUZZY_RELEVANCE_SCALE: f32 = 0.7;

// ---------------------------------------------------------------------------
// SeedFinder — fuzzy query -> node matching
// Replaces: engine_v2.py SeedFinder, engine_fast.py FastSeedFinder
// ---------------------------------------------------------------------------

/// Finds seed nodes matching a natural-language query.
/// Uses label substring, tag intersection, type filtering, and synonym expansion.
pub struct SeedFinder;

impl SeedFinder {
    /// Tokenize query: lowercase, split on whitespace/punctuation, filter short tokens.
    fn tokenize(query: &str) -> Vec<String> {
        query
            .to_lowercase()
            .split(|c: char| c.is_whitespace() || c == '?' || c == '!' || c == '.' || c == ',')
            .filter(|t| t.len() > 2)
            .map(|t| t.to_string())
            .collect()
    }

    /// Trigram set for fuzzy matching.
    fn trigrams(s: &str) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() < 3 {
            return vec![s.to_string()];
        }
        chars.windows(3).map(|w| w.iter().collect()).collect()
    }

    /// Trigram cosine similarity between two strings.
    fn trigram_similarity(a: &str, b: &str) -> f32 {
        let ta = Self::trigrams(&a.to_lowercase());
        let tb = Self::trigrams(&b.to_lowercase());
        if ta.is_empty() || tb.is_empty() {
            return 0.0;
        }
        let mut dot = 0usize;
        for t in &ta {
            if tb.contains(t) {
                dot += 1;
            }
        }
        if dot == 0 {
            return 0.0;
        }
        dot as f32 / ((ta.len() as f32).sqrt() * (tb.len() as f32).sqrt())
    }

    /// Find seeds matching `query`. Returns (NodeId, relevance) sorted descending.
    /// Replaces: engine_v2.py SeedFinder.find_seeds()
    pub fn find_seeds(
        graph: &Graph,
        query: &str,
        max_seeds: usize,
    ) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
        let tokens = Self::tokenize(query);
        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let n = graph.num_nodes() as usize;
        let cap = max_seeds.min(MAX_SEEDS);

        // Per-node best relevance
        let mut relevance = vec![0.0f32; n];

        for i in 0..n {
            let node_id = NodeId::new(i as u32);
            let label = graph.strings.resolve(graph.nodes.label[i]);
            let label_lower = label.to_lowercase();

            let mut best = 0.0f32;

            for token in &tokens {
                // 1. Exact label match
                if label_lower == *token {
                    best = best.max(EXACT_MATCH_RELEVANCE);
                    continue;
                }

                // 2. Prefix match
                if label_lower.starts_with(token.as_str())
                    || token.starts_with(label_lower.as_str())
                {
                    best = best.max(PREFIX_MATCH_RELEVANCE);
                    continue;
                }

                // 3. Substring match
                if label_lower.contains(token.as_str()) || token.contains(label_lower.as_str()) {
                    best = best.max(0.8);
                    continue;
                }

                // 4. Tag match
                for &tag_interned in &graph.nodes.tags[i] {
                    let tag = graph.strings.resolve(tag_interned).to_lowercase();
                    if tag == *token || tag.contains(token.as_str()) {
                        best = best.max(TAG_MATCH_RELEVANCE);
                    }
                }

                // 5. Fuzzy trigram match
                let sim = Self::trigram_similarity(token, &label_lower);
                if sim > 0.3 {
                    best = best.max(FUZZY_RELEVANCE_SCALE * sim);
                }
            }

            relevance[i] = best;
        }

        // Collect, filter, sort, cap
        let mut results: Vec<(NodeId, FiniteF32)> = relevance
            .iter()
            .enumerate()
            .filter(|(_, &r)| r >= MIN_RELEVANCE)
            .map(|(i, &r)| (NodeId::new(i as u32), FiniteF32::new(r)))
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(cap);
        Ok(results)
    }

    /// Find seeds using semantic engine for enhanced matching.
    /// Two-phase: SeedFinder.find_seeds() + SemanticEngine re-rank.
    /// Replaces: engine_v2.py ConnectomeEngine._find_and_boost_seeds()
    pub fn find_seeds_semantic(
        graph: &Graph,
        semantic: &crate::semantic::SemanticEngine,
        query: &str,
        max_seeds: usize,
    ) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
        // Phase 1: basic seed finding
        let mut seeds = Self::find_seeds(graph, query, max_seeds * 3)?;

        // Phase 2: re-rank with semantic similarity
        let semantic_scores = semantic.query_fast(graph, query, max_seeds * 3)?;
        let mut sem_map = std::collections::HashMap::new();
        for (node, score) in &semantic_scores {
            sem_map.insert(node.0, score.get());
        }

        // Blend: 0.6 * basic + 0.4 * semantic
        for (node, ref mut score) in &mut seeds {
            let sem = sem_map.get(&node.0).copied().unwrap_or(0.0);
            let blended = score.get() * 0.6 + sem * 0.4;
            *score = FiniteF32::new(blended);
        }

        seeds.sort_by(|a, b| b.1.cmp(&a.1));
        seeds.truncate(max_seeds.min(MAX_SEEDS));
        Ok(seeds)
    }
}
