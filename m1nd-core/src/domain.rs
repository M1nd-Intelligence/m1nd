// === crates/m1nd-core/src/domain.rs ===

use crate::types::NodeType;
use std::collections::HashMap;

/// Domain configuration for m1nd. Controls how the engine behaves
/// for different domains (code, music production, supply chain, etc.).
pub struct DomainConfig {
    /// Human-readable domain name
    pub name: String,
    /// Temporal decay half-life (in hours) per NodeType.
    /// Missing entries use default_half_life.
    pub half_lives: HashMap<NodeType, f32>,
    /// Default half-life for types not in the map
    pub default_half_life: f32,
    /// Relation types recognized in this domain
    pub relations: Vec<String>,
    /// Whether to use git-based co-change (only makes sense for code)
    pub git_co_change: bool,
}

impl DomainConfig {
    /// Code intelligence domain (current behavior)
    pub fn code() -> Self {
        let mut half_lives = HashMap::new();
        half_lives.insert(NodeType::File, 168.0);       // 7 days
        half_lives.insert(NodeType::Function, 336.0);    // 14 days
        half_lives.insert(NodeType::Class, 504.0);       // 21 days
        half_lives.insert(NodeType::Struct, 504.0);      // 21 days
        half_lives.insert(NodeType::Enum, 504.0);        // 21 days
        half_lives.insert(NodeType::Module, 720.0);      // 30 days
        half_lives.insert(NodeType::Directory, 720.0);   // 30 days
        half_lives.insert(NodeType::Type, 504.0);        // 21 days
        Self {
            name: "code".into(),
            half_lives,
            default_half_life: 168.0,
            relations: vec![
                "contains".into(), "imports".into(), "calls".into(),
                "references".into(), "implements".into(),
            ],
            git_co_change: true,
        }
    }

    /// Music production / DAW domain
    pub fn music() -> Self {
        let mut half_lives = HashMap::new();
        // Rooms/buses are structural (slow decay)
        half_lives.insert(NodeType::System, 720.0);      // 30 days (rooms, buses)
        // Plugins/effects change more often
        half_lives.insert(NodeType::Process, 336.0);      // 14 days (plugins, effects)
        // Parameters change constantly
        half_lives.insert(NodeType::Material, 168.0);     // 7 days (audio signals)
        half_lives.insert(NodeType::Concept, 504.0);      // 21 days (presets, templates)
        Self {
            name: "music".into(),
            half_lives,
            default_half_life: 336.0,
            relations: vec![
                "routes_to".into(), "sends_to".into(), "controls".into(),
                "modulates".into(), "contains".into(), "monitors".into(),
            ],
            git_co_change: false,
        }
    }

    /// Generic domain (no assumptions)
    pub fn generic() -> Self {
        Self {
            name: "generic".into(),
            half_lives: HashMap::new(),
            default_half_life: 336.0, // 14 days
            relations: vec![
                "contains".into(), "references".into(), "depends_on".into(),
                "produces".into(), "consumes".into(),
            ],
            git_co_change: false,
        }
    }

    /// Get half-life for a node type
    pub fn half_life_for(&self, node_type: NodeType) -> f32 {
        self.half_lives.get(&node_type).copied().unwrap_or(self.default_half_life)
    }
}
