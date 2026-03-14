// === crates/m1nd-core/src/semantic.rs ===

use std::collections::HashMap;
use smallvec::SmallVec;

use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::types::*;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default n-gram size (trigrams).
const NGRAM_SIZE: usize = 3;
/// Maximum token length for n-gram extraction.
const MAX_TOKEN_LENGTH: usize = 200;
/// Random walk parameters for co-occurrence.
/// Increased from 10/6/3 for code graphs with deeper hierarchies
/// (module -> file -> class -> method -> field = 5+ levels).
const WALKS_PER_NODE: usize = 20;
const WALK_LENGTH: usize = 10;
const WINDOW_SIZE: usize = 4;
/// Max nodes before disabling co-occurrence (DEC-050).
const COOCCURRENCE_MAX_NODES: u32 = 50_000;

// ---------------------------------------------------------------------------
// CharNgramIndex — trigram embeddings (semantic_v2.py CharNgramEmbedder)
// Replaces: semantic_v2.py CharNgramEmbedder
// ---------------------------------------------------------------------------

/// Sparse trigram vector for a single node.
/// Key: 24-bit hash of trigram. Value: TF-IDF weight.
pub type NgramVector = HashMap<u32, FiniteF32>;

/// Pre-built char n-gram index over all node labels.
/// Stores sparse trigram vectors for each node.
/// FM-SEM-003 fix: inverted token index for O(K) query instead of O(N^2*S).
pub struct CharNgramIndex {
    /// Per-node trigram vectors indexed by NodeId.
    vectors: Vec<NgramVector>,
    /// Inverted index: trigram hash -> list of (NodeId, weight).
    /// Enables sub-linear candidate generation.
    inverted: HashMap<u32, Vec<(NodeId, FiniteF32)>>,
    /// IDF values: trigram hash -> log(N/df)+1. Used by query_vector().
    idf: HashMap<u32, f32>,
    /// N-gram size (default 3 = trigrams).
    ngram_size: usize,
}

impl CharNgramIndex {
    /// Build index from all node labels in the graph.
    /// Replaces: semantic_v2.py CharNgramEmbedder.build()
    /// FM-SEM-001 fix: applies Unicode NFC normalization before trigram extraction.
    /// TF-IDF weighting: raw TF * log(N/df)+1 for discriminative trigrams.
    pub fn build(graph: &Graph, ngram_size: usize) -> M1ndResult<Self> {
        let n = graph.num_nodes() as usize;

        // Phase 1: Build raw TF vectors and compute document frequency
        let mut raw_vectors: Vec<NgramVector> = Vec::with_capacity(n);
        let mut doc_freq: HashMap<u32, u32> = HashMap::new();

        for i in 0..n {
            let label = graph.strings.resolve(graph.nodes.label[i]);
            let normalized = label.to_lowercase();
            let vec = Self::build_ngram_vector(&normalized, ngram_size);

            // Count document frequency per trigram
            for &hash in vec.keys() {
                *doc_freq.entry(hash).or_insert(0) += 1;
            }

            raw_vectors.push(vec);
        }

        // Phase 2: Compute IDF and apply TF-IDF weighting
        let n_f32 = n.max(1) as f32;
        let mut idf: HashMap<u32, f32> = HashMap::new();
        for (&hash, &df) in &doc_freq {
            idf.insert(hash, (n_f32 / df as f32).ln() + 1.0);
        }

        let mut vectors = Vec::with_capacity(n);
        let mut inverted: HashMap<u32, Vec<(NodeId, FiniteF32)>> = HashMap::new();

        for (i, raw_vec) in raw_vectors.into_iter().enumerate() {
            let mut tfidf_vec = NgramVector::new();

            for (&hash, &tf) in &raw_vec {
                let idf_val = idf.get(&hash).copied().unwrap_or(1.0);
                let tfidf = tf.get() * idf_val;
                tfidf_vec.insert(hash, FiniteF32::new(tfidf));
            }

            // L2 normalize for cosine similarity
            let norm: f32 = tfidf_vec.values().map(|v| v.get() * v.get()).sum::<f32>().sqrt();
            if norm > 0.0 {
                for (&hash, &weight) in &tfidf_vec {
                    let normalized_w = FiniteF32::new(weight.get() / norm);
                    inverted
                        .entry(hash)
                        .or_default()
                        .push((NodeId::new(i as u32), normalized_w));
                }
            }

            vectors.push(tfidf_vec);
        }

        Ok(Self {
            vectors,
            inverted,
            idf,
            ngram_size,
        })
    }

    /// Build n-gram frequency vector for a string.
    fn build_ngram_vector(s: &str, ngram_size: usize) -> NgramVector {
        let s = if s.len() > MAX_TOKEN_LENGTH { &s[..MAX_TOKEN_LENGTH] } else { s };
        let chars: Vec<char> = s.chars().collect();
        let mut vec = NgramVector::new();
        if chars.len() < ngram_size {
            // For short strings, use the whole string as one gram
            let hash = Self::hash_ngram(s);
            vec.insert(hash, FiniteF32::ONE);
            return vec;
        }
        for window in chars.windows(ngram_size) {
            let gram: String = window.iter().collect();
            let hash = Self::hash_ngram(&gram);
            let entry = vec.entry(hash).or_insert(FiniteF32::ZERO);
            *entry = FiniteF32::new(entry.get() + 1.0);
        }
        vec
    }

    /// Hash a trigram to a 24-bit key. FNV-1a variant.
    fn hash_ngram(ngram: &str) -> u32 {
        let mut hash: u32 = 2166136261;
        for byte in ngram.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
        hash & 0x00FFFFFF // 24-bit
    }

    /// Compute trigram vector for a query string, with IDF weighting.
    pub fn query_vector(&self, query: &str) -> NgramVector {
        let raw = Self::build_ngram_vector(&query.to_lowercase(), self.ngram_size);
        let mut tfidf = NgramVector::new();
        for (&hash, &tf) in &raw {
            let idf_val = self.idf.get(&hash).copied().unwrap_or(1.0);
            tfidf.insert(hash, FiniteF32::new(tf.get() * idf_val));
        }
        tfidf
    }

    /// Score all nodes against a query vector. Returns top_k by cosine similarity.
    /// Uses inverted index for candidate generation (FM-SEM-003 fix).
    /// Replaces: semantic_v2.py CharNgramEmbedder.query()
    pub fn query_top_k(
        &self,
        query: &str,
        top_k: usize,
    ) -> Vec<(NodeId, FiniteF32)> {
        let qvec = self.query_vector(query);
        if qvec.is_empty() {
            return Vec::new();
        }

        // Query norm
        let q_norm: f32 = qvec.values().map(|v| v.get() * v.get()).sum::<f32>().sqrt();
        if q_norm <= 0.0 {
            return Vec::new();
        }

        // Candidate accumulation via inverted index
        let mut scores: HashMap<u32, f32> = HashMap::new();
        for (&hash, &q_weight) in &qvec {
            if let Some(postings) = self.inverted.get(&hash) {
                for &(node, norm_weight) in postings {
                    *scores.entry(node.0).or_insert(0.0) += q_weight.get() * norm_weight.get();
                }
            }
        }

        // Normalize by query norm
        let mut results: Vec<(NodeId, FiniteF32)> = scores
            .iter()
            .map(|(&node_id, &dot)| {
                let sim = dot / q_norm;
                (NodeId::new(node_id), FiniteF32::new(sim.min(1.0)))
            })
            .filter(|(_, s)| s.get() > 0.01)
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_k);
        results
    }

    /// Cosine similarity between two sparse vectors.
    pub fn cosine_similarity(a: &NgramVector, b: &NgramVector) -> FiniteF32 {
        if a.is_empty() || b.is_empty() {
            return FiniteF32::ZERO;
        }
        let mut dot = 0.0f32;
        for (k, va) in a {
            if let Some(vb) = b.get(k) {
                dot += va.get() * vb.get();
            }
        }
        let norm_a: f32 = a.values().map(|v| v.get() * v.get()).sum::<f32>().sqrt();
        let norm_b: f32 = b.values().map(|v| v.get() * v.get()).sum::<f32>().sqrt();
        let denom = norm_a * norm_b;
        if denom > 0.0 {
            FiniteF32::new((dot / denom).min(1.0))
        } else {
            FiniteF32::ZERO
        }
    }
}

// ---------------------------------------------------------------------------
// CoOccurrenceIndex — DeepWalk-lite embeddings (semantic_v2.py CoOccurrenceEmbedder)
// Replaces: semantic_v2.py CoOccurrenceEmbedder
// ---------------------------------------------------------------------------

/// Per-node co-occurrence vector: sorted Vec<(NodeId, weight)> for fast intersection.
/// FM-SEM-004 fix: 12 bytes/entry vs 100 bytes in Python HashMap.
pub type CoOccurrenceVector = Vec<(NodeId, FiniteF32)>;

/// Co-occurrence embeddings built from short random walks on the graph.
pub struct CoOccurrenceIndex {
    /// Per-node co-occurrence vectors, indexed by NodeId.
    vectors: Vec<CoOccurrenceVector>,
    /// Walk length for random walk generation.
    walk_length: usize,
    /// Number of walks per node.
    walks_per_node: usize,
    /// Window size for co-occurrence counting.
    window_size: usize,
}

impl CoOccurrenceIndex {
    /// Build co-occurrence embeddings from random walks.
    /// Replaces: semantic_v2.py CoOccurrenceEmbedder.build()
    /// FM-SEM-004: memory warning logged if node_count > 10_000.
    pub fn build(
        graph: &Graph,
        walk_length: usize,
        walks_per_node: usize,
        window_size: usize,
    ) -> M1ndResult<Self> {
        let n = graph.num_nodes() as usize;

        // DEC-050: disable for large graphs
        if graph.num_nodes() > COOCCURRENCE_MAX_NODES {
            return Ok(Self {
                vectors: vec![Vec::new(); n],
                walk_length,
                walks_per_node,
                window_size,
            });
        }

        let mut vectors = vec![Vec::new(); n];

        // Simple PRNG (deterministic with seed 42)
        let mut rng_state = 42u64;
        let mut next_rng = || -> u64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            rng_state >> 33
        };

        // For each node, perform random walks and accumulate co-occurrence
        for start in 0..n {
            let mut co_counts: HashMap<u32, f32> = HashMap::new();
            let start_node = NodeId::new(start as u32);

            for _ in 0..walks_per_node {
                let mut walk = Vec::with_capacity(walk_length);
                let mut current = start_node;
                walk.push(current);

                for _ in 1..walk_length {
                    let range = graph.csr.out_range(current);
                    let degree = range.end - range.start;
                    if degree == 0 {
                        break;
                    }
                    let idx = (next_rng() as usize) % degree;
                    current = graph.csr.targets[range.start + idx];
                    walk.push(current);
                }

                // Extract co-occurrence pairs within window
                for i in 0..walk.len() {
                    let lo = i.saturating_sub(window_size);
                    let hi = (i + window_size + 1).min(walk.len());
                    for j in lo..hi {
                        if i != j && walk[j].0 != start as u32 {
                            *co_counts.entry(walk[j].0).or_insert(0.0) += 1.0;
                        }
                    }
                }
            }

            // Store raw counts; PPMI normalization below
            if !co_counts.is_empty() {
                vectors[start] = co_counts
                    .into_iter()
                    .map(|(id, count)| (NodeId::new(id), FiniteF32::new(count)))
                    .collect();
            }
        }

        // PPMI normalization: upweight surprising co-occurrences, downweight expected ones
        // Marginals: total_j = sum over all i of count(i,j), total_all = sum of everything
        let mut marginal_j: HashMap<u32, f32> = HashMap::new();
        let mut marginal_i: Vec<f32> = Vec::with_capacity(n);
        let mut total_all = 0.0f32;

        for vec in &vectors {
            let row_sum: f32 = vec.iter().map(|(_, w)| w.get()).sum();
            marginal_i.push(row_sum);
            total_all += row_sum;
            for &(node, weight) in vec {
                *marginal_j.entry(node.0).or_insert(0.0) += weight.get();
            }
        }

        if total_all > 0.0 {
            for (i, vec) in vectors.iter_mut().enumerate() {
                let mi = marginal_i[i];
                if mi <= 0.0 {
                    continue;
                }
                let mut ppmi_vec: CoOccurrenceVector = Vec::with_capacity(vec.len());
                for &(node, raw_count) in vec.iter() {
                    let mj = marginal_j.get(&node.0).copied().unwrap_or(1.0);
                    // PMI = log2( (count * total) / (margin_i * margin_j) )
                    let pmi = ((raw_count.get() * total_all) / (mi * mj)).ln();
                    if pmi > 0.0 {
                        ppmi_vec.push((node, FiniteF32::new(pmi)));
                    }
                }
                ppmi_vec.sort_by_key(|e| e.0);
                *vec = ppmi_vec;
            }
        }

        Ok(Self {
            vectors,
            walk_length,
            walks_per_node,
            window_size,
        })
    }

    /// Cosine similarity between two sorted co-occurrence vectors.
    /// Uses merge-intersection on sorted vectors for O(D) instead of O(D^2).
    pub fn cosine_similarity(
        a: &CoOccurrenceVector,
        b: &CoOccurrenceVector,
    ) -> FiniteF32 {
        if a.is_empty() || b.is_empty() {
            return FiniteF32::ZERO;
        }

        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        for (_, w) in a {
            norm_a += w.get() * w.get();
        }
        for (_, w) in b {
            norm_b += w.get() * w.get();
        }

        // Merge intersection
        let mut ia = 0;
        let mut ib = 0;
        while ia < a.len() && ib < b.len() {
            let (na, wa) = a[ia];
            let (nb, wb) = b[ib];
            if na == nb {
                dot += wa.get() * wb.get();
                ia += 1;
                ib += 1;
            } else if na < nb {
                ia += 1;
            } else {
                ib += 1;
            }
        }

        let denom = norm_a.sqrt() * norm_b.sqrt();
        if denom > 0.0 {
            FiniteF32::new((dot / denom).min(1.0))
        } else {
            FiniteF32::ZERO
        }
    }

    /// Score a query node against all nodes. Returns top_k.
    /// Replaces: semantic_v2.py CoOccurrenceEmbedder.query()
    pub fn query_top_k(
        &self,
        query_node: NodeId,
        top_k: usize,
    ) -> Vec<(NodeId, FiniteF32)> {
        let idx = query_node.as_usize();
        if idx >= self.vectors.len() || self.vectors[idx].is_empty() {
            return Vec::new();
        }

        let query_vec = &self.vectors[idx];
        let mut results: Vec<(NodeId, FiniteF32)> = self.vectors
            .iter()
            .enumerate()
            .filter(|(i, v)| *i != idx && !v.is_empty())
            .map(|(i, v)| {
                let sim = Self::cosine_similarity(query_vec, v);
                (NodeId::new(i as u32), sim)
            })
            .filter(|(_, s)| s.get() > 0.01)
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_k);
        results
    }
}

// ---------------------------------------------------------------------------
// SynonymExpander — bidirectional synonym group lookup
// Replaces: semantic_v2.py SynonymExpander + SYNONYM_GROUPS constant
// ---------------------------------------------------------------------------

/// Synonym expansion table. Groups of semantically equivalent terms.
/// FM-SEM-002 fix: no overlapping terms across groups (transitive closure prevented).
/// Uses String-based lookups (not InternedStr) to avoid orphan interner bug.
pub struct SynonymExpander {
    /// Synonym groups: each group is a Vec of lowercased terms.
    groups: Vec<Vec<String>>,
    /// Reverse index: lowercased term -> group indices.
    term_to_groups: HashMap<String, SmallVec<[u16; 4]>>,
}

/// Default synonym groups (Portuguese domain terms from semantic_v2.py).
const DEFAULT_SYNONYM_GROUPS: &[&[&str]] = &[
    &["plastico", "polimero", "resina"],
    &["metal", "liga", "aco", "aluminio"],
    &["vidro", "ceramica", "cristal"],
    &["processo", "etapa", "fase"],
    &["material", "materia_prima", "insumo"],
    &["custo", "preco", "valor"],
    &["fornecedor", "supplier", "vendor"],
    &["qualidade", "quality", "qa"],
    &["teste", "test", "ensaio"],
    &["norma", "regulamento", "padrão"],
    &["function", "fn", "method", "func"],
    &["class", "struct", "type"],
    &["module", "package", "crate"],
    &["import", "use", "require"],
    &["error", "exception", "panic"],
];

impl SynonymExpander {
    /// Build from the built-in SYNONYM_GROUPS constant.
    /// Validates no term appears in multiple groups (FM-SEM-002).
    /// Replaces: semantic_v2.py SynonymExpander.__init__()
    pub fn build_default() -> M1ndResult<Self> {
        let groups: Vec<Vec<&str>> = DEFAULT_SYNONYM_GROUPS
            .iter()
            .map(|g| g.to_vec())
            .collect();
        Self::build(groups)
    }

    /// Build from custom synonym groups. Uses String-based lookup (no interner needed).
    pub fn build(groups: Vec<Vec<&str>>) -> M1ndResult<Self> {
        let mut string_groups = Vec::with_capacity(groups.len());
        let mut term_to_groups: HashMap<String, SmallVec<[u16; 4]>> = HashMap::new();

        for (gi, group) in groups.iter().enumerate() {
            let mut str_group: Vec<String> = Vec::with_capacity(group.len());
            for &term in group {
                let lower = term.to_lowercase();
                term_to_groups
                    .entry(lower.clone())
                    .or_default()
                    .push(gi as u16);
                str_group.push(lower);
            }
            string_groups.push(str_group);
        }

        Ok(Self {
            groups: string_groups,
            term_to_groups,
        })
    }

    /// Expand a term to all synonyms (including itself).
    /// Replaces: semantic_v2.py SynonymExpander.expand()
    pub fn expand(&self, term: &str) -> Vec<String> {
        let lower = term.to_lowercase();
        let mut result = vec![lower.clone()];
        if let Some(group_indices) = self.term_to_groups.get(&lower) {
            for &gi in group_indices {
                if (gi as usize) < self.groups.len() {
                    for member in &self.groups[gi as usize] {
                        if *member != lower && !result.contains(member) {
                            result.push(member.clone());
                        }
                    }
                }
            }
        }
        result
    }

    /// Check if two terms are synonymous.
    pub fn are_synonyms(&self, a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();
        if a_lower == b_lower {
            return true;
        }
        if let Some(groups_a) = self.term_to_groups.get(&a_lower) {
            if let Some(groups_b) = self.term_to_groups.get(&b_lower) {
                for &ga in groups_a {
                    for &gb in groups_b {
                        if ga == gb {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// SemanticEngine — combined 3-component scorer
// Replaces: semantic_v2.py SemanticEngine
// ---------------------------------------------------------------------------

/// Combined semantic matching: 0.4*ngram + 0.4*cooccurrence + 0.2*synonym.
/// Two-phase query_fast: phase 1 ngram+synonym (0.6/0.4), phase 2 re-rank top 3*K with cooccurrence.
/// Replaces: semantic_v2.py SemanticEngine
pub struct SemanticEngine {
    pub ngram: CharNgramIndex,
    pub cooccurrence: CoOccurrenceIndex,
    pub synonym: SynonymExpander,
    pub weights: SemanticWeights,
}

impl SemanticEngine {
    /// Build all three indexes from the graph.
    /// Replaces: semantic_v2.py SemanticEngine.__init__()
    pub fn build(graph: &Graph, weights: SemanticWeights) -> M1ndResult<Self> {
        let ngram = CharNgramIndex::build(graph, NGRAM_SIZE)?;
        let cooccurrence = CoOccurrenceIndex::build(graph, WALK_LENGTH, WALKS_PER_NODE, WINDOW_SIZE)?;
        let synonym = SynonymExpander::build_default()?;

        Ok(Self {
            ngram,
            cooccurrence,
            synonym,
            weights,
        })
    }

    /// Full query: score all nodes, return top_k.
    /// Weight: ngram*0.4 + cooccurrence*0.4 + synonym*0.2.
    /// Replaces: semantic_v2.py SemanticEngine.query()
    pub fn query(
        &self,
        graph: &Graph,
        query: &str,
        top_k: usize,
    ) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
        // Phase 1: n-gram scores
        let ngram_scores = self.ngram.query_top_k(query, top_k * 5);

        // Build combined score map
        let mut scores: HashMap<u32, f32> = HashMap::new();
        for &(node, score) in &ngram_scores {
            *scores.entry(node.0).or_insert(0.0) += score.get() * self.weights.ngram.get();
        }

        // Phase 2: co-occurrence boost for top candidates
        // (only if we have seed nodes from ngram phase)
        if let Some(&(first_node, _)) = ngram_scores.first() {
            let cooc_scores = self.cooccurrence.query_top_k(first_node, top_k * 3);
            for (node, score) in cooc_scores {
                *scores.entry(node.0).or_insert(0.0) += score.get() * self.weights.cooccurrence.get();
            }
        }

        // Synonym boost: expand query tokens via synonym groups,
        // then boost nodes whose labels match expanded synonyms.
        let tokens: Vec<String> = query.to_lowercase().split_whitespace()
            .filter(|t| t.len() > 2)
            .map(|t| t.to_string())
            .collect();

        // Expand each token via synonym groups
        let mut expanded_tokens: Vec<String> = Vec::new();
        for token in &tokens {
            for syn in self.synonym.expand(token) {
                if !expanded_tokens.contains(&syn) {
                    expanded_tokens.push(syn);
                }
            }
        }

        // Boost nodes whose labels match expanded synonyms (not original tokens)
        let synonym_weight = self.weights.synonym.get();
        for i in 0..graph.num_nodes() as usize {
            let label = graph.strings.resolve(graph.nodes.label[i]).to_lowercase();
            for expanded in &expanded_tokens {
                if !tokens.contains(expanded) && label.contains(expanded.as_str()) {
                    *scores.entry(i as u32).or_insert(0.0) += synonym_weight;
                }
            }
        }

        let mut results: Vec<(NodeId, FiniteF32)> = scores
            .into_iter()
            .map(|(id, s)| (NodeId::new(id), FiniteF32::new(s.min(1.0))))
            .filter(|(_, s)| s.get() > 0.01)
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_k);
        Ok(results)
    }

    /// Fast two-phase query.
    /// Phase 1: ngram+synonym (0.6/0.4) -> candidates (3*top_k).
    /// Phase 2: re-rank candidates with cooccurrence.
    /// Replaces: semantic_v2.py SemanticEngine.query_fast()
    pub fn query_fast(
        &self,
        graph: &Graph,
        query: &str,
        top_k: usize,
    ) -> M1ndResult<Vec<(NodeId, FiniteF32)>> {
        // Phase 1: ngram candidates
        let candidates = self.ngram.query_top_k(query, top_k * 3);
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Multi-seed co-occurrence: aggregate from top-3 ngram hits (not just #1)
        // to avoid single-point-of-failure when top hit is a leaf node
        let seed_count = candidates.len().min(3);
        let mut cooc_map: HashMap<u32, f32> = HashMap::new();
        for &(node, ngram_score) in &candidates[..seed_count] {
            let cooc = self.cooccurrence.query_top_k(node, top_k * 3);
            for (n, s) in cooc {
                *cooc_map.entry(n.0).or_insert(0.0) += s.get() * ngram_score.get();
            }
        }
        // Normalize by seed count
        let seed_f = seed_count as f32;
        for v in cooc_map.values_mut() {
            *v /= seed_f;
        }

        // Re-rank using configured weights (normalized to sum to 1.0)
        let total_w = self.weights.ngram.get() + self.weights.cooccurrence.get();
        let ngram_w = if total_w > 0.0 { self.weights.ngram.get() / total_w } else { 0.6 };
        let cooc_w = if total_w > 0.0 { self.weights.cooccurrence.get() / total_w } else { 0.4 };

        let mut results: Vec<(NodeId, FiniteF32)> = candidates
            .iter()
            .map(|&(node, ngram_score)| {
                let cooc_score = cooc_map.get(&node.0).copied().unwrap_or(0.0);
                let combined = ngram_score.get() * ngram_w + cooc_score * cooc_w;
                (node, FiniteF32::new(combined.min(1.0)))
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_k);
        Ok(results)
    }
}

// Ensure Send + Sync for concurrent access.
static_assertions::assert_impl_all!(SemanticEngine: Send, Sync);
