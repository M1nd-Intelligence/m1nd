// === crates/m1nd-core/src/builder.rs ===

use crate::error::M1ndResult;
use crate::graph::Graph;
use crate::types::{EdgeDirection, FiniteF32, NodeId, NodeType};

/// Programmatic graph builder for non-code domains.
/// Allows building graphs without going through file extractors.
pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: Graph::with_capacity(nodes, edges),
        }
    }

    /// Add a node with arbitrary type and metadata
    pub fn add_node(
        &mut self,
        id: &str,
        label: &str,
        node_type: NodeType,
        tags: &[&str],
    ) -> M1ndResult<NodeId> {
        self.graph.add_node(id, label, node_type, tags, 0.0, 0.3)
    }

    /// Add a node with full temporal data
    pub fn add_node_with_temporal(
        &mut self,
        id: &str,
        label: &str,
        node_type: NodeType,
        tags: &[&str],
        timestamp: f64,
        change_freq: f32,
    ) -> M1ndResult<NodeId> {
        self.graph
            .add_node(id, label, node_type, tags, timestamp, change_freq)
    }

    /// Add a directed edge
    pub fn add_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        relation: &str,
        weight: f32,
    ) -> M1ndResult<()> {
        self.graph.add_edge(
            source,
            target,
            relation,
            FiniteF32::new(weight),
            EdgeDirection::Forward,
            false,
            FiniteF32::new(weight * 0.8), // causal = 80% of weight
        )?;
        Ok(())
    }

    /// Add a bidirectional edge
    pub fn add_bidi_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        relation: &str,
        weight: f32,
    ) -> M1ndResult<()> {
        self.graph.add_edge(
            source,
            target,
            relation,
            FiniteF32::new(weight),
            EdgeDirection::Bidirectional,
            false,
            FiniteF32::new(weight * 0.8),
        )?;
        Ok(())
    }

    /// Finalize and return the graph
    pub fn finalize(mut self) -> M1ndResult<Graph> {
        if self.graph.num_nodes() > 0 {
            self.graph.finalize()?;
        }
        Ok(self.graph)
    }
}
