// === m1nd-mcp/src/perspective/validation.rs ===
// Theme 9: Input Validation and Parameter Bounds.
// Centralized validation layer called at the top of every handler.

use m1nd_core::error::{M1ndError, M1ndResult};

use super::state::{LockScope, LockScopeConfig, PerspectiveLens, RouteFamily};

// ---------------------------------------------------------------------------
// Validated types (output of validation layer)
// ---------------------------------------------------------------------------

/// Validated lens — all fields clamped and normalized.
#[derive(Clone, Debug)]
pub struct ValidatedLens {
    pub dimensions: Vec<String>,
    pub route_families: Vec<RouteFamily>,
    pub xlr: bool,
    pub include_ghost_edges: bool,
    pub include_structural_holes: bool,
    pub top_k: u32,
    pub namespaces: Vec<String>,
    pub tags: Vec<String>,
    pub node_types: Vec<String>,
}

/// Validated pagination parameters.
#[derive(Clone, Debug)]
pub struct ValidatedPagination {
    pub page: u32,
    pub page_size: u32,
    pub total_items: usize,
    pub total_pages: u32,
    pub offset: usize,
    /// Whether page_size was clamped.
    pub page_size_clamped: bool,
}

/// Validated lock scope.
#[derive(Clone, Debug)]
pub struct ValidatedScope {
    pub scope_type: LockScope,
    pub root_nodes: Vec<String>,
    pub radius: Option<u32>,
    pub query: Option<String>,
    pub path_nodes: Option<Vec<String>>,
}

/// Validated route reference — exactly one of route_id or route_index.
#[derive(Clone, Debug)]
pub enum ValidatedRouteRef {
    ById(String),
    ByIndex(u32),
}

// ---------------------------------------------------------------------------
// Known dimensions (for validation)
// ---------------------------------------------------------------------------

const VALID_DIMENSIONS: &[&str] = &["structural", "semantic", "temporal", "causal"];

// ---------------------------------------------------------------------------
// Validation functions
// ---------------------------------------------------------------------------

/// Validate a PerspectiveLens against the graph.
/// Returns ValidatedLens with all fields clamped and normalized.
///
/// Rules (Theme 9):
/// - dimensions: non-empty, case-insensitive, unknown → reject with valid options.
/// - route_families: empty = all. Case-insensitive via serde.
/// - top_k: clamped to [1, min(requested, graph_node_count)].
/// - namespaces/tags/node_types: empty = all (no filter).
pub fn validate_lens(lens: &PerspectiveLens, graph_node_count: usize) -> M1ndResult<ValidatedLens> {
    // Dimensions: must be non-empty
    let dimensions: Vec<String> = if lens.dimensions.is_empty() {
        VALID_DIMENSIONS.iter().map(|s| s.to_string()).collect()
    } else {
        let mut normalized = Vec::with_capacity(lens.dimensions.len());
        for d in &lens.dimensions {
            let lower = d.to_ascii_lowercase();
            if !VALID_DIMENSIONS.contains(&lower.as_str()) {
                return Err(M1ndError::InvalidParams {
                    tool: "perspective".into(),
                    detail: format!(
                        "unknown dimension '{}'. Valid: {:?}",
                        d, VALID_DIMENSIONS
                    ),
                });
            }
            normalized.push(lower);
        }
        normalized
    };

    // top_k: clamp to [1, graph_node_count]
    let max_k = graph_node_count.max(1) as u32;
    let top_k = lens.top_k.max(1).min(max_k);

    Ok(ValidatedLens {
        dimensions,
        route_families: lens.route_families.clone(),
        xlr: lens.xlr,
        include_ghost_edges: lens.include_ghost_edges,
        include_structural_holes: lens.include_structural_holes,
        top_k,
        namespaces: lens.namespaces.clone(),
        tags: lens.tags.clone(),
        node_types: lens.node_types.clone(),
    })
}

/// Validate pagination parameters.
///
/// Rules (Theme 9):
/// - page: >= 1. Reject 0 with INVALID_PAGE.
/// - page_size: clamped to [1, 10]. Default 6.
pub fn validate_pagination(page: u32, page_size: u32, total_items: usize) -> M1ndResult<ValidatedPagination> {
    if page == 0 {
        return Err(M1ndError::InvalidParams {
            tool: "perspective".into(),
            detail: "page must be >= 1".into(),
        });
    }

    let clamped_size = page_size.max(1).min(10);
    let page_size_clamped = clamped_size != page_size;

    let total_pages = if total_items == 0 {
        1
    } else {
        ((total_items as u32) + clamped_size - 1) / clamped_size
    };

    let safe_page = page.min(total_pages);
    let offset = ((safe_page - 1) * clamped_size) as usize;

    Ok(ValidatedPagination {
        page: safe_page,
        page_size: clamped_size,
        total_items,
        total_pages,
        offset,
        page_size_clamped,
    })
}

/// Validate a lock scope configuration.
///
/// Rules (Theme 9):
/// - root_nodes: non-empty. Each resolved against graph.
/// - radius: subgraph min 1, max 4. Node scope: exactly 0.
/// - path: consecutive pairs must have direct edges (validated at handler time, not here).
pub fn validate_lock_scope(
    scope: &LockScopeConfig,
    known_nodes: &[String],
) -> M1ndResult<ValidatedScope> {
    // root_nodes: non-empty
    if scope.root_nodes.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "lock.create".into(),
            detail: "root_nodes must be non-empty".into(),
        });
    }

    // Validate root nodes exist in graph
    let mut invalid_roots = Vec::new();
    for root in &scope.root_nodes {
        if !known_nodes.contains(root) {
            invalid_roots.push(root.clone());
        }
    }
    if !invalid_roots.is_empty() {
        return Err(M1ndError::InvalidParams {
            tool: "lock.create".into(),
            detail: format!("unknown root nodes: {:?}", invalid_roots),
        });
    }

    // Radius validation
    let radius = match scope.scope_type {
        LockScope::Subgraph => {
            let r = scope.radius.unwrap_or(2);
            if r < 1 || r > 4 {
                return Err(M1ndError::InvalidParams {
                    tool: "lock.create".into(),
                    detail: format!("subgraph radius must be 1-4, got {}", r),
                });
            }
            Some(r)
        }
        LockScope::Node => Some(0),
        LockScope::QueryNeighborhood => {
            if scope.query.is_none() {
                return Err(M1ndError::InvalidParams {
                    tool: "lock.create".into(),
                    detail: "query_neighborhood scope requires a query".into(),
                });
            }
            None
        }
        LockScope::Path => {
            if scope.path_nodes.as_ref().map_or(true, |p| p.is_empty()) {
                return Err(M1ndError::InvalidParams {
                    tool: "lock.create".into(),
                    detail: "path scope requires non-empty path_nodes".into(),
                });
            }
            None
        }
    };

    Ok(ValidatedScope {
        scope_type: scope.scope_type.clone(),
        root_nodes: scope.root_nodes.clone(),
        radius,
        query: scope.query.clone(),
        path_nodes: scope.path_nodes.clone(),
    })
}

/// Validate a route reference (exactly one of route_id or route_index).
///
/// Returns AMBIGUOUS_ROUTE_REF if both provided, MISSING_ROUTE_REF if neither.
pub fn validate_route_ref(
    route_id: &Option<String>,
    route_index: &Option<u32>,
    tool: &str,
) -> M1ndResult<ValidatedRouteRef> {
    match (route_id, route_index) {
        (Some(id), None) => Ok(ValidatedRouteRef::ById(id.clone())),
        (None, Some(idx)) => Ok(ValidatedRouteRef::ByIndex(*idx)),
        (Some(_), Some(_)) => Err(M1ndError::InvalidParams {
            tool: tool.into(),
            detail: "provide route_id OR route_index, not both".into(),
        }),
        (None, None) => Err(M1ndError::InvalidParams {
            tool: tool.into(),
            detail: "provide route_id or route_index".into(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_lens_defaults() {
        let lens = PerspectiveLens::default();
        let result = validate_lens(&lens, 100).unwrap();
        assert_eq!(result.dimensions.len(), 4);
        assert_eq!(result.top_k, 8);
    }

    #[test]
    fn validate_lens_rejects_unknown_dimension() {
        let mut lens = PerspectiveLens::default();
        lens.dimensions = vec!["structural".into(), "magic".into()];
        let result = validate_lens(&lens, 100);
        assert!(result.is_err());
    }

    #[test]
    fn validate_lens_normalizes_case() {
        let mut lens = PerspectiveLens::default();
        lens.dimensions = vec!["STRUCTURAL".into(), "Semantic".into()];
        let result = validate_lens(&lens, 100).unwrap();
        assert_eq!(result.dimensions, vec!["structural", "semantic"]);
    }

    #[test]
    fn validate_lens_clamps_top_k() {
        let mut lens = PerspectiveLens::default();
        lens.top_k = 1000;
        let result = validate_lens(&lens, 50).unwrap();
        assert_eq!(result.top_k, 50);
    }

    #[test]
    fn validate_lens_empty_dimensions_defaults_to_all() {
        let mut lens = PerspectiveLens::default();
        lens.dimensions = Vec::new();
        let result = validate_lens(&lens, 100).unwrap();
        assert_eq!(result.dimensions.len(), 4);
    }

    #[test]
    fn validate_pagination_rejects_page_zero() {
        let result = validate_pagination(0, 6, 20);
        assert!(result.is_err());
    }

    #[test]
    fn validate_pagination_clamps_page_size() {
        let result = validate_pagination(1, 50, 20).unwrap();
        assert_eq!(result.page_size, 10);
        assert!(result.page_size_clamped);
    }

    #[test]
    fn validate_pagination_correct_total_pages() {
        let result = validate_pagination(1, 6, 20).unwrap();
        assert_eq!(result.total_pages, 4); // ceil(20/6) = 4
    }

    #[test]
    fn validate_pagination_clamps_page_to_max() {
        let result = validate_pagination(100, 6, 20).unwrap();
        assert_eq!(result.page, 4); // max page
    }

    #[test]
    fn validate_lock_scope_rejects_empty_roots() {
        let scope = LockScopeConfig {
            scope_type: LockScope::Node,
            root_nodes: Vec::new(),
            radius: None,
            query: None,
            path_nodes: None,
        };
        let result = validate_lock_scope(&scope, &["a".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn validate_lock_scope_rejects_invalid_radius() {
        let scope = LockScopeConfig {
            scope_type: LockScope::Subgraph,
            root_nodes: vec!["a".into()],
            radius: Some(10),
            query: None,
            path_nodes: None,
        };
        let result = validate_lock_scope(&scope, &["a".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn validate_route_ref_rejects_both() {
        let result = validate_route_ref(
            &Some("R_abc".into()),
            &Some(1),
            "test",
        );
        assert!(result.is_err());
    }

    #[test]
    fn validate_route_ref_rejects_neither() {
        let result = validate_route_ref(&None, &None, "test");
        assert!(result.is_err());
    }

    #[test]
    fn validate_route_ref_accepts_id() {
        let result = validate_route_ref(&Some("R_abc".into()), &None, "test").unwrap();
        matches!(result, ValidatedRouteRef::ById(_));
    }

    #[test]
    fn validate_route_ref_accepts_index() {
        let result = validate_route_ref(&None, &Some(3), "test").unwrap();
        matches!(result, ValidatedRouteRef::ByIndex(3));
    }
}
