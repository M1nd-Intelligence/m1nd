// === crates/m1nd-core/src/graph.rs ===

use smallvec::SmallVec;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::{M1ndError, M1ndResult};
use crate::types::*;

// ---------------------------------------------------------------------------
// StringInterner — zero-allocation string comparison (04-SPEC Section 1.5)
// Replaces: Python string equality everywhere
// ---------------------------------------------------------------------------

/// Intern table mapping strings to unique u32 handles.
/// Thread-safe for reads; mutation requires &mut.
pub struct StringInterner {
    strings: Vec<String>,
    index: HashMap<String, InternedStr>,
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            strings: Vec::with_capacity(cap),
            index: HashMap::with_capacity(cap),
        }
    }

    /// Intern `s`, returning its handle. Idempotent.
    pub fn get_or_intern(&mut self, s: &str) -> InternedStr {
        if let Some(&idx) = self.index.get(s) {
            return idx;
        }
        let idx = InternedStr(self.strings.len() as u32);
        self.strings.push(s.to_owned());
        self.index.insert(s.to_owned(), idx);
        idx
    }

    /// Resolve handle back to string. Panics if idx out of range.
    pub fn resolve(&self, idx: InternedStr) -> &str {
        &self.strings[idx.0 as usize]
    }

    /// Try to resolve without panicking.
    pub fn try_resolve(&self, idx: InternedStr) -> Option<&str> {
        self.strings.get(idx.0 as usize).map(|s| s.as_str())
    }

    /// Lookup without interning. Returns `None` if not present.
    pub fn lookup(&self, s: &str) -> Option<InternedStr> {
        self.index.get(s).copied()
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

// ---------------------------------------------------------------------------
// CsrGraph — Compressed Sparse Row (04-SPEC Section 1.1)
// Replaces: engine_fast.py CSRAdjacency
// ---------------------------------------------------------------------------

/// Raw edge data stored before CSR construction.
#[derive(Clone)]
pub struct PendingEdge {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: FiniteF32,
    pub inhibitory: bool,
    pub relation: InternedStr,
    pub direction: EdgeDirection,
    pub causal_strength: FiniteF32,
}

/// Compressed Sparse Row graph with forward and reverse adjacency.
/// For node `i`, outgoing edges span `offsets[i]..offsets[i+1]`
/// into targets, weights, inhibitory, relations, directions, causal_strengths.
pub struct CsrGraph {
    // --- Forward CSR ---
    /// Length: num_nodes + 1. offsets[num_nodes] == total_edges.
    pub offsets: Vec<u64>,
    /// Length: total_edges. Target node for each edge.
    pub targets: Vec<NodeId>,
    /// Length: total_edges. Edge weight — atomic for lock-free plasticity updates (FM-ACT-021).
    pub weights: Vec<AtomicU32>, // bit-reinterpreted f32; use FiniteF32 for reads
    /// Length: total_edges. true = inhibitory edge.
    pub inhibitory: Vec<bool>,
    /// Length: total_edges. Relation type (interned).
    pub relations: Vec<InternedStr>,
    /// Length: total_edges. Forward or Bidirectional.
    pub directions: Vec<EdgeDirection>,
    /// Length: total_edges. Causal strength in [0.0, 1.0].
    pub causal_strengths: Vec<FiniteF32>,

    // --- Reverse CSR (built at finalize) ---
    /// Length: num_nodes + 1.
    pub rev_offsets: Vec<u64>,
    /// Length: total_edges. Source node for each reverse edge.
    pub rev_sources: Vec<NodeId>,
    /// Length: total_edges. Index into forward arrays for this reverse edge.
    pub rev_edge_idx: Vec<EdgeIdx>,

    /// Pre-finalize edge staging area.
    pub pending_edges: Vec<PendingEdge>,
}

impl CsrGraph {
    /// Create an empty CSR with no nodes/edges.
    pub fn empty() -> Self {
        Self {
            offsets: Vec::new(),
            targets: Vec::new(),
            weights: Vec::new(),
            inhibitory: Vec::new(),
            relations: Vec::new(),
            directions: Vec::new(),
            causal_strengths: Vec::new(),
            rev_offsets: Vec::new(),
            rev_sources: Vec::new(),
            rev_edge_idx: Vec::new(),
            pending_edges: Vec::new(),
        }
    }

    /// Number of edges in the forward CSR.
    pub fn num_edges(&self) -> usize {
        if self.offsets.is_empty() {
            0
        } else {
            *self.offsets.last().unwrap() as usize
        }
    }

    /// Outgoing edge range for `node`.
    #[inline]
    pub fn out_range(&self, node: NodeId) -> std::ops::Range<usize> {
        let lo = self.offsets[node.as_usize()] as usize;
        let hi = self.offsets[node.as_usize() + 1] as usize;
        lo..hi
    }

    /// Incoming edge range for `node` (reverse CSR).
    #[inline]
    pub fn in_range(&self, node: NodeId) -> std::ops::Range<usize> {
        let lo = self.rev_offsets[node.as_usize()] as usize;
        let hi = self.rev_offsets[node.as_usize() + 1] as usize;
        lo..hi
    }

    /// Read weight atomically as FiniteF32 (FM-ACT-021).
    #[inline]
    pub fn read_weight(&self, edge: EdgeIdx) -> FiniteF32 {
        let bits = self.weights[edge.as_usize()].load(Ordering::Relaxed);
        FiniteF32::new(f32::from_bits(bits))
    }

    /// Atomic CAS max on edge weight. Returns Ok on success, Err after retry limit (FM-ACT-019).
    /// Replaces: engine_fast.py direct weight assignment (now lock-free).
    pub fn atomic_max_weight(
        &self,
        edge: EdgeIdx,
        new_val: FiniteF32,
        max_retries: u32,
    ) -> M1ndResult<()> {
        let slot = &self.weights[edge.as_usize()];
        let new_bits = new_val.get().to_bits();
        for _ in 0..max_retries {
            let old_bits = slot.load(Ordering::Relaxed);
            let old_val = f32::from_bits(old_bits);
            if old_val >= new_val.get() {
                return Ok(());
            }
            if slot
                .compare_exchange_weak(old_bits, new_bits, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(());
            }
        }
        Err(M1ndError::CasRetryExhausted {
            edge,
            limit: max_retries,
        })
    }

    /// Atomic CAS write on edge weight (for plasticity). Returns Ok on success, Err after retry limit.
    pub fn atomic_write_weight(
        &self,
        edge: EdgeIdx,
        new_val: FiniteF32,
        max_retries: u32,
    ) -> M1ndResult<()> {
        let slot = &self.weights[edge.as_usize()];
        let new_bits = new_val.get().to_bits();
        for _ in 0..max_retries {
            let old_bits = slot.load(Ordering::Relaxed);
            match slot.compare_exchange_weak(
                old_bits,
                new_bits,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(_) => continue,
            }
        }
        Err(M1ndError::CasRetryExhausted {
            edge,
            limit: max_retries,
        })
    }
}

// ---------------------------------------------------------------------------
// PlasticityNode — per-node homeostatic metadata (04-SPEC Section 1.2)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct PlasticityNode {
    /// Sum of incoming edge weights (for homeostatic normalisation).
    pub incoming_weight_sum: FiniteF32,
    /// Homeostatic ceiling for this node.
    /// Default: HOMEOSTATIC_CEILING = 5.0 from plasticity.py
    pub ceiling: FiniteF32,
}

// ---------------------------------------------------------------------------
// NodeProvenance — cold-path source metadata for nodes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct NodeProvenance {
    pub source_path: Option<InternedStr>,
    pub line_start: u32,
    pub line_end: u32,
    pub excerpt: Option<InternedStr>,
    pub namespace: Option<InternedStr>,
    pub canonical: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NodeProvenanceInput<'a> {
    pub source_path: Option<&'a str>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub excerpt: Option<&'a str>,
    pub namespace: Option<&'a str>,
    pub canonical: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedNodeProvenance {
    pub source_path: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub excerpt: Option<String>,
    pub namespace: Option<String>,
    pub canonical: bool,
}

impl ResolvedNodeProvenance {
    pub fn is_empty(&self) -> bool {
        self.source_path.is_none()
            && self.line_start.is_none()
            && self.line_end.is_none()
            && self.excerpt.is_none()
            && self.namespace.is_none()
            && !self.canonical
    }
}

// ---------------------------------------------------------------------------
// NodeStorage — SoA layout (04-SPEC Section 1.2)
// Replaces: engine_v2.py Node dataclass, engine_fast.py FastNode
// ---------------------------------------------------------------------------

/// All per-node data in Struct-of-Arrays layout for cache-friendly access.
pub struct NodeStorage {
    pub count: u32,

    // --- Hot path: activation engine reads every query ---
    /// Activation levels [structural, semantic, temporal, causal] per node.
    /// Packed as [f32; 4] because all 4 dims accessed together per node.
    pub activation: Vec<[FiniteF32; 4]>,
    /// PageRank score, computed once at finalize.
    pub pagerank: Vec<FiniteF32>,

    // --- Warm path: plasticity reads per query ---
    pub plasticity: Vec<PlasticityNode>,

    // --- Cold path: seed finding, display, export ---
    /// Interned label index.
    pub label: Vec<InternedStr>,
    /// Node type tag.
    pub node_type: Vec<NodeType>,
    /// Tag set: SmallVec of interned string indices.
    pub tags: Vec<SmallVec<[InternedStr; 6]>>,
    /// Last modification time (Unix seconds).
    pub last_modified: Vec<f64>,
    /// Change frequency normalised [0.0, 1.0].
    pub change_frequency: Vec<FiniteF32>,
    /// Provenance / source metadata for cold-path inspection.
    pub provenance: Vec<NodeProvenance>,
}

impl Default for NodeStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            count: 0,
            activation: Vec::new(),
            pagerank: Vec::new(),
            plasticity: Vec::new(),
            label: Vec::new(),
            node_type: Vec::new(),
            tags: Vec::new(),
            last_modified: Vec::new(),
            change_frequency: Vec::new(),
            provenance: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            count: 0,
            activation: Vec::with_capacity(cap),
            pagerank: Vec::with_capacity(cap),
            plasticity: Vec::with_capacity(cap),
            label: Vec::with_capacity(cap),
            node_type: Vec::with_capacity(cap),
            tags: Vec::with_capacity(cap),
            last_modified: Vec::with_capacity(cap),
            change_frequency: Vec::with_capacity(cap),
            provenance: Vec::with_capacity(cap),
        }
    }
}

// ---------------------------------------------------------------------------
// EdgePlasticity — per-edge Hebbian state (04-SPEC Section 1.3)
// Replaces: plasticity.py SynapticState per-edge tracking
// ---------------------------------------------------------------------------

/// Per-edge plasticity metadata. Parallel arrays alongside CSR edges.
pub struct EdgePlasticity {
    /// Original weight at graph construction.
    pub original_weight: Vec<FiniteF32>,
    /// Current weight (canonical — CSR AtomicWeights mirror this).
    pub current_weight: Vec<FiniteF32>,
    /// Number of times this edge was strengthened.
    pub strengthen_count: Vec<u16>,
    /// Number of times this edge was weakened.
    pub weaken_count: Vec<u16>,
    /// Whether LTP (long-term potentiation) was applied.
    pub ltp_applied: Vec<bool>,
    /// Whether LTD (long-term depression) was applied.
    pub ltd_applied: Vec<bool>,
    /// Query index at which this edge was last used.
    pub last_used_query: Vec<u32>,
}

impl Default for EdgePlasticity {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgePlasticity {
    pub fn new() -> Self {
        Self {
            original_weight: Vec::new(),
            current_weight: Vec::new(),
            strengthen_count: Vec::new(),
            weaken_count: Vec::new(),
            ltp_applied: Vec::new(),
            ltd_applied: Vec::new(),
            last_used_query: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            original_weight: Vec::with_capacity(cap),
            current_weight: Vec::with_capacity(cap),
            strengthen_count: Vec::with_capacity(cap),
            weaken_count: Vec::with_capacity(cap),
            ltp_applied: Vec::with_capacity(cap),
            ltd_applied: Vec::with_capacity(cap),
            last_used_query: Vec::with_capacity(cap),
        }
    }
}

// ---------------------------------------------------------------------------
// Graph — top-level property graph (04-SPEC Section 1.6)
// Replaces: engine_v2.py PropertyGraph + engine_fast.py FastPropertyGraph
// ---------------------------------------------------------------------------

/// The complete property graph. Owns all storage.
/// Mutation methods increment `generation` for desync detection (FM-PL-006).
pub struct Graph {
    pub nodes: NodeStorage,
    pub csr: CsrGraph,
    pub edge_plasticity: EdgePlasticity,
    pub strings: StringInterner,
    /// Maps interned external ID -> internal NodeId.
    pub id_to_node: HashMap<InternedStr, NodeId>,
    /// Monotonic counter incremented on every structural mutation.
    pub generation: Generation,
    pub pagerank_computed: bool,
    pub finalized: bool,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: NodeStorage::new(),
            csr: CsrGraph::empty(),
            edge_plasticity: EdgePlasticity::new(),
            strings: StringInterner::new(),
            id_to_node: HashMap::new(),
            generation: Generation(0),
            pagerank_computed: false,
            finalized: false,
        }
    }

    pub fn with_capacity(node_cap: usize, edge_cap: usize) -> Self {
        Self {
            nodes: NodeStorage::with_capacity(node_cap),
            csr: CsrGraph::empty(),
            edge_plasticity: EdgePlasticity::with_capacity(edge_cap),
            strings: StringInterner::with_capacity(node_cap),
            id_to_node: HashMap::with_capacity(node_cap),
            generation: Generation(0),
            pagerank_computed: false,
            finalized: false,
        }
    }

    /// Add a node. Returns its NodeId. Increments generation.
    /// Replaces: engine_v2.py PropertyGraph.add_node()
    pub fn add_node(
        &mut self,
        external_id: &str,
        label: &str,
        node_type: NodeType,
        tags: &[&str],
        last_modified: f64,
        change_frequency: f32,
    ) -> M1ndResult<NodeId> {
        // FM-ACT-016: duplicate node check
        let ext_interned = self.strings.get_or_intern(external_id);
        if let Some(&existing) = self.id_to_node.get(&ext_interned) {
            return Err(M1ndError::DuplicateNode(existing));
        }

        let id = NodeId::new(self.nodes.count);
        self.nodes.count += 1;

        let label_interned = self.strings.get_or_intern(label);
        let tag_interned: SmallVec<[InternedStr; 6]> =
            tags.iter().map(|t| self.strings.get_or_intern(t)).collect();

        self.nodes.label.push(label_interned);
        self.nodes.node_type.push(node_type);
        self.nodes.tags.push(tag_interned);
        self.nodes.last_modified.push(last_modified);
        self.nodes
            .change_frequency
            .push(FiniteF32::new(change_frequency));
        self.nodes.activation.push([FiniteF32::ZERO; 4]);
        self.nodes.pagerank.push(FiniteF32::ZERO);
        self.nodes.plasticity.push(PlasticityNode::default());
        self.nodes.provenance.push(NodeProvenance::default());

        self.id_to_node.insert(ext_interned, id);
        self.generation = self.generation.next();
        self.finalized = false;
        Ok(id)
    }

    /// Add an edge. Validates source/target existence (FM-ACT-011). Increments generation.
    /// Replaces: engine_v2.py PropertyGraph.add_edge()
    #[allow(clippy::too_many_arguments)]
    pub fn add_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        relation: &str,
        weight: FiniteF32,
        direction: EdgeDirection,
        inhibitory: bool,
        causal_strength: FiniteF32,
    ) -> M1ndResult<EdgeIdx> {
        // FM-ACT-011: dangling edge check
        if source.as_usize() >= self.nodes.count as usize {
            return Err(M1ndError::DanglingEdge {
                edge: EdgeIdx::new(self.edge_plasticity.original_weight.len() as u32),
                node: source,
            });
        }
        if target.as_usize() >= self.nodes.count as usize {
            return Err(M1ndError::DanglingEdge {
                edge: EdgeIdx::new(self.edge_plasticity.original_weight.len() as u32),
                node: target,
            });
        }

        let edge_idx = EdgeIdx::new(self.edge_plasticity.original_weight.len() as u32);
        let rel_interned = self.strings.get_or_intern(relation);

        // Store in pending edge list (will be turned into CSR on finalize)
        self.edge_plasticity.original_weight.push(weight);
        self.edge_plasticity.current_weight.push(weight);
        self.edge_plasticity.strengthen_count.push(0);
        self.edge_plasticity.weaken_count.push(0);
        self.edge_plasticity.ltp_applied.push(false);
        self.edge_plasticity.ltd_applied.push(false);
        self.edge_plasticity.last_used_query.push(0);

        // Store raw edge data for CSR building later
        self.csr.pending_edges.push(PendingEdge {
            source,
            target,
            weight,
            inhibitory,
            relation: rel_interned,
            direction,
            causal_strength,
        });

        self.generation = self.generation.next();
        self.finalized = false;
        Ok(edge_idx)
    }

    /// Build CSR forward + reverse adjacency. Compute PageRank.
    /// Must be called before any query. Sets `finalized = true`.
    /// Replaces: engine_fast.py FastPropertyGraph.finalize()
    pub fn finalize(&mut self) -> M1ndResult<()> {
        if self.finalized {
            return Ok(());
        }
        let n = self.nodes.count as usize;

        // Build forward CSR from pending edges
        // Sort edges by source for CSR layout, preserving original insertion index
        let edges = std::mem::take(&mut self.csr.pending_edges);
        // Pair each edge with its original insertion index (into edge_plasticity)
        let mut indexed_edges: Vec<(usize, PendingEdge)> = edges.into_iter().enumerate().collect();
        indexed_edges.sort_by_key(|(_, e)| e.source.0);

        let total_edges = indexed_edges.len();

        let mut offsets = vec![0u64; n + 1];
        let mut targets = Vec::with_capacity(total_edges);
        let mut weights = Vec::with_capacity(total_edges);
        let mut inhibitory = Vec::with_capacity(total_edges);
        let mut relations = Vec::with_capacity(total_edges);
        let mut directions = Vec::with_capacity(total_edges);
        let mut causal_strengths = Vec::with_capacity(total_edges);

        // Count edges per source
        for (_, e) in &indexed_edges {
            offsets[e.source.as_usize() + 1] += 1;
            // Bidirectional edges also get a reverse entry in forward CSR
            if e.direction == EdgeDirection::Bidirectional {
                offsets[e.target.as_usize() + 1] += 1;
            }
        }
        // Prefix sum
        for i in 1..=n {
            offsets[i] += offsets[i - 1];
        }

        let total_csr_edges = *offsets.last().unwrap_or(&0) as usize;
        targets.resize(total_csr_edges, NodeId::default());
        weights.extend((0..total_csr_edges).map(|_| AtomicU32::new(0)));
        inhibitory.resize(total_csr_edges, false);
        relations.resize(total_csr_edges, InternedStr::default());
        directions.resize(total_csr_edges, EdgeDirection::Forward);
        causal_strengths.resize(total_csr_edges, FiniteF32::ZERO);

        // Fill using write cursors, tracking original->CSR mapping for plasticity rebuild
        // Each entry: (original_insertion_idx, csr_position)
        let mut plasticity_mapping: Vec<(usize, usize)> = Vec::with_capacity(total_csr_edges);

        let mut cursors = offsets[..n].to_vec();

        for &(orig_idx, ref e) in &indexed_edges {
            let src = e.source.as_usize();
            let pos = cursors[src] as usize;
            targets[pos] = e.target;
            weights[pos] = AtomicU32::new(e.weight.get().to_bits());
            inhibitory[pos] = e.inhibitory;
            relations[pos] = e.relation;
            directions[pos] = e.direction;
            causal_strengths[pos] = e.causal_strength;
            cursors[src] += 1;
            plasticity_mapping.push((orig_idx, pos));

            if e.direction == EdgeDirection::Bidirectional {
                let tgt = e.target.as_usize();
                let pos2 = cursors[tgt] as usize;
                targets[pos2] = e.source;
                weights[pos2] = AtomicU32::new(e.weight.get().to_bits());
                inhibitory[pos2] = e.inhibitory;
                relations[pos2] = e.relation;
                directions[pos2] = e.direction;
                causal_strengths[pos2] = e.causal_strength;
                cursors[tgt] += 1;
                // Bidirectional reverse direction gets same plasticity data (cloned)
                plasticity_mapping.push((orig_idx, pos2));
            }
        }

        // Rebuild edge_plasticity arrays to match CSR order and count
        let old_plasticity = &self.edge_plasticity;
        let mut new_plasticity = EdgePlasticity::with_capacity(total_csr_edges);
        new_plasticity
            .original_weight
            .resize(total_csr_edges, FiniteF32::ZERO);
        new_plasticity
            .current_weight
            .resize(total_csr_edges, FiniteF32::ZERO);
        new_plasticity.strengthen_count.resize(total_csr_edges, 0);
        new_plasticity.weaken_count.resize(total_csr_edges, 0);
        new_plasticity.ltp_applied.resize(total_csr_edges, false);
        new_plasticity.ltd_applied.resize(total_csr_edges, false);
        new_plasticity.last_used_query.resize(total_csr_edges, 0);

        for &(orig_idx, csr_pos) in &plasticity_mapping {
            new_plasticity.original_weight[csr_pos] = old_plasticity.original_weight[orig_idx];
            new_plasticity.current_weight[csr_pos] = old_plasticity.current_weight[orig_idx];
            new_plasticity.strengthen_count[csr_pos] = old_plasticity.strengthen_count[orig_idx];
            new_plasticity.weaken_count[csr_pos] = old_plasticity.weaken_count[orig_idx];
            new_plasticity.ltp_applied[csr_pos] = old_plasticity.ltp_applied[orig_idx];
            new_plasticity.ltd_applied[csr_pos] = old_plasticity.ltd_applied[orig_idx];
            new_plasticity.last_used_query[csr_pos] = old_plasticity.last_used_query[orig_idx];
        }

        self.edge_plasticity = new_plasticity;

        // Build reverse CSR (in-edges)
        // Count in-degree per node
        let mut rev_offsets = vec![0u64; n + 1];
        for i in 0..n {
            let lo = offsets[i] as usize;
            let hi = offsets[i + 1] as usize;
            for tgt_node in &targets[lo..hi] {
                let tgt = tgt_node.as_usize();
                rev_offsets[tgt + 1] += 1;
            }
        }
        for i in 1..=n {
            rev_offsets[i] += rev_offsets[i - 1];
        }

        let total_rev = *rev_offsets.last().unwrap_or(&0) as usize;
        let mut rev_sources = vec![NodeId::default(); total_rev];
        let mut rev_edge_idx = vec![EdgeIdx::default(); total_rev];

        let mut rev_cursors = rev_offsets[..n].to_vec();
        #[allow(clippy::needless_range_loop)]
        for src in 0..n {
            let lo = offsets[src] as usize;
            let hi = offsets[src + 1] as usize;
            for j in lo..hi {
                let tgt = targets[j].as_usize();
                let pos = rev_cursors[tgt] as usize;
                rev_sources[pos] = NodeId::new(src as u32);
                rev_edge_idx[pos] = EdgeIdx::new(j as u32);
                rev_cursors[tgt] += 1;
            }
        }

        self.csr = CsrGraph {
            offsets,
            targets,
            weights,
            inhibitory,
            relations,
            directions,
            causal_strengths,
            rev_offsets,
            rev_sources,
            rev_edge_idx,
            pending_edges: Vec::new(),
        };

        // Compute PageRank
        self.compute_pagerank(0.85, 50, 1e-6);

        self.finalized = true;
        Ok(())
    }

    /// Number of nodes.
    pub fn num_nodes(&self) -> u32 {
        self.nodes.count
    }

    /// Number of edges (forward CSR).
    pub fn num_edges(&self) -> usize {
        self.csr.num_edges()
    }

    /// Resolve external string ID to NodeId.
    pub fn resolve_id(&self, external_id: &str) -> Option<NodeId> {
        let interned = self.strings.lookup(external_id)?;
        self.id_to_node.get(&interned).copied()
    }

    pub fn set_node_provenance(&mut self, node: NodeId, provenance: NodeProvenanceInput<'_>) {
        let idx = node.as_usize();
        if idx >= self.nodes.count as usize {
            return;
        }

        self.nodes.provenance[idx] = NodeProvenance {
            source_path: provenance
                .source_path
                .filter(|value| !value.is_empty())
                .map(|value| self.strings.get_or_intern(value)),
            line_start: provenance.line_start.unwrap_or(0),
            line_end: provenance.line_end.or(provenance.line_start).unwrap_or(0),
            excerpt: provenance
                .excerpt
                .filter(|value| !value.is_empty())
                .map(|value| self.strings.get_or_intern(value)),
            namespace: provenance
                .namespace
                .filter(|value| !value.is_empty())
                .map(|value| self.strings.get_or_intern(value)),
            canonical: provenance.canonical,
        };
    }

    pub fn merge_node_provenance(&mut self, node: NodeId, incoming: NodeProvenanceInput<'_>) {
        let idx = node.as_usize();
        if idx >= self.nodes.count as usize {
            return;
        }

        let current = self.resolve_node_provenance(node);
        let line_start = current.line_start.or(incoming.line_start);
        let line_end = match (current.line_end, incoming.line_end.or(incoming.line_start)) {
            (Some(existing), Some(extra)) => Some(existing.max(extra)),
            (Some(existing), None) => Some(existing),
            (None, Some(extra)) => Some(extra),
            (None, None) => line_start,
        };

        self.set_node_provenance(
            node,
            NodeProvenanceInput {
                source_path: current.source_path.as_deref().or(incoming.source_path),
                line_start,
                line_end,
                excerpt: current.excerpt.as_deref().or(incoming.excerpt),
                namespace: current.namespace.as_deref().or(incoming.namespace),
                canonical: current.canonical || incoming.canonical,
            },
        );
    }

    pub fn resolve_node_provenance(&self, node: NodeId) -> ResolvedNodeProvenance {
        let idx = node.as_usize();
        if idx >= self.nodes.count as usize {
            return ResolvedNodeProvenance::default();
        }

        let provenance = self.nodes.provenance[idx];
        ResolvedNodeProvenance {
            source_path: provenance
                .source_path
                .and_then(|value| self.strings.try_resolve(value).map(str::to_owned)),
            line_start: (provenance.line_start > 0).then_some(provenance.line_start),
            line_end: (provenance.line_end > 0).then_some(provenance.line_end),
            excerpt: provenance
                .excerpt
                .and_then(|value| self.strings.try_resolve(value).map(str::to_owned)),
            namespace: provenance
                .namespace
                .and_then(|value| self.strings.try_resolve(value).map(str::to_owned)),
            canonical: provenance.canonical,
        }
    }

    /// Average out-degree.
    pub fn avg_degree(&self) -> f32 {
        if self.nodes.count == 0 {
            0.0
        } else {
            self.csr.num_edges() as f32 / self.nodes.count as f32
        }
    }

    /// Iterative PageRank on CSR. Power iteration until convergence.
    /// Replaces: engine_v2.py PropertyGraph.compute_pagerank()
    /// DEC-040: damping=0.85, iterations=50, convergence=1e-6
    fn compute_pagerank(&mut self, damping: f32, max_iterations: u32, convergence: f32) {
        let n = self.nodes.count as usize;
        if n == 0 {
            self.pagerank_computed = true;
            return;
        }

        let nf = n as f32;
        let base = (1.0 - damping) / nf;
        let mut pr = vec![1.0f32 / nf; n];
        let mut new_pr = vec![0.0f32; n];

        // Precompute out-degree from forward CSR
        let mut out_degree = vec![0u32; n];
        for (i, deg) in out_degree.iter_mut().enumerate().take(n) {
            let lo = self.csr.offsets[i] as usize;
            let hi = self.csr.offsets[i + 1] as usize;
            *deg = (hi - lo) as u32;
        }

        for _iter in 0..max_iterations {
            new_pr.fill(base);

            // For each node i, accumulate contribution from in-neighbors
            for (i, new_rank) in new_pr.iter_mut().enumerate().take(n) {
                let lo = self.csr.rev_offsets[i] as usize;
                let hi = self.csr.rev_offsets[i + 1] as usize;
                let mut rank_sum = 0.0f32;
                for j in lo..hi {
                    let src = self.csr.rev_sources[j].as_usize();
                    let deg = out_degree[src];
                    if deg > 0 {
                        rank_sum += pr[src] / deg as f32;
                    }
                }
                *new_rank += damping * rank_sum;
            }

            // Check convergence (L1 norm)
            let mut delta = 0.0f32;
            for i in 0..n {
                delta += (new_pr[i] - pr[i]).abs();
            }
            std::mem::swap(&mut pr, &mut new_pr);
            if delta < convergence {
                break;
            }
        }

        // Normalize to [0, 1] by max value
        let max_pr = pr.iter().cloned().fold(0.0f32, f32::max);
        if max_pr > 0.0 {
            for (i, &rank) in pr.iter().enumerate().take(n) {
                self.nodes.pagerank[i] = FiniteF32::new(rank / max_pr);
            }
        }
        self.pagerank_computed = true;
    }
}

// ---------------------------------------------------------------------------
// SharedGraph — concurrent access (04-SPEC Section 5.1)
// Uses parking_lot for fairness (prevents write starvation from queries).
// ---------------------------------------------------------------------------

/// Shared graph handle. Many readers (queries), one writer (ingestion/plasticity).
pub type SharedGraph = std::sync::Arc<parking_lot::RwLock<Graph>>;
