// === m1nd-mcp/src/perspective/keys.rs ===
// Theme 4: Content-addressable edge keys and stable route IDs.
// These functions produce deterministic identifiers that survive graph rebuild.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::state::RouteFamily;

// ---------------------------------------------------------------------------
// Edge content key (for lock baselines)
// ---------------------------------------------------------------------------

/// Produce a content-addressable key for an edge between two external node IDs.
///
/// For bidirectional edges, the caller should pass the source/target in
/// lexicographic order (normalize before calling). The returned format is:
/// `"{source_ext_id}->{target_ext_id}:{relation}"`.
///
/// The `key_format: "v1_content_addr"` tag stored in `LockSnapshot` identifies
/// keys produced by this version.
pub fn edge_content_key(source_ext_id: &str, target_ext_id: &str, relation: &str) -> String {
    format!("{}->{}: {}", source_ext_id, target_ext_id, relation)
}

/// Normalize a bidirectional edge key so that the lexicographically smaller
/// external ID always appears as "source". Returns `(lo, hi)`.
pub fn normalize_bidi_endpoints<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

// ---------------------------------------------------------------------------
// Route content ID (for stable route references)
// ---------------------------------------------------------------------------

/// Produce a stable, content-addressed route ID.
///
/// Format: `R_{hash[:6]}` where the hash is derived from the target node's
/// external ID and the route family. This survives graph rebuilds because it
/// depends only on external identifiers, not internal `EdgeIdx` values.
pub fn route_content_id(target_ext_id: &str, family: &RouteFamily) -> String {
    let mut hasher = DefaultHasher::new();
    target_ext_id.hash(&mut hasher);
    family_discriminant(family).hash(&mut hasher);
    let hash = hasher.finish();
    // Take 6 hex chars from the hash.
    format!("R_{:06x}", hash & 0x00FF_FFFF)
}

/// Map `RouteFamily` to a stable discriminant for hashing.
/// Using explicit integers so the hash remains stable even if enum order changes.
fn family_discriminant(family: &RouteFamily) -> u8 {
    match family {
        RouteFamily::Structural => 0,
        RouteFamily::Semantic => 1,
        RouteFamily::Temporal => 2,
        RouteFamily::Causal => 3,
        RouteFamily::Ghost => 4,
        RouteFamily::Hole => 5,
        RouteFamily::Resonant => 6,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_key_is_deterministic() {
        let k1 = edge_content_key("main.rs", "lib.rs", "imports");
        let k2 = edge_content_key("main.rs", "lib.rs", "imports");
        assert_eq!(k1, k2);
    }

    #[test]
    fn edge_key_differs_by_relation() {
        let k1 = edge_content_key("a", "b", "imports");
        let k2 = edge_content_key("a", "b", "calls");
        assert_ne!(k1, k2);
    }

    #[test]
    fn bidi_normalization() {
        let (lo, hi) = normalize_bidi_endpoints("z_node", "a_node");
        assert_eq!(lo, "a_node");
        assert_eq!(hi, "z_node");
    }

    #[test]
    fn route_id_format() {
        let id = route_content_id("session.rs", &RouteFamily::Structural);
        assert!(id.starts_with("R_"));
        assert_eq!(id.len(), 8); // "R_" + 6 hex chars
    }

    #[test]
    fn route_id_is_deterministic() {
        let a = route_content_id("foo", &RouteFamily::Ghost);
        let b = route_content_id("foo", &RouteFamily::Ghost);
        assert_eq!(a, b);
    }

    #[test]
    fn route_id_differs_by_family() {
        let a = route_content_id("foo", &RouteFamily::Structural);
        let b = route_content_id("foo", &RouteFamily::Semantic);
        assert_ne!(a, b);
    }
}
