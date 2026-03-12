// === crates/m1nd-core/src/types.rs ===

use std::fmt;

// ---------------------------------------------------------------------------
// FiniteF32 — NaN/Inf impossible by construction (FM-PL-001, FM-ACT-019)
// Replaces: all f32 activation values, edge weights, scores system-wide
// ---------------------------------------------------------------------------

/// A finite f32 that can never be NaN or Inf.
/// In debug builds, panics on non-finite input.
/// In release builds, clamps non-finite to 0.0.
#[derive(Clone, Copy, Default, PartialEq)]
#[repr(transparent)]
pub struct FiniteF32(f32);

impl FiniteF32 {
    /// Construct from raw f32. Debug-panics if non-finite; release-clamps to 0.0.
    #[inline]
    pub fn new(v: f32) -> Self {
        debug_assert!(v.is_finite(), "FiniteF32::new received non-finite: {v}");
        Self(if v.is_finite() { v } else { 0.0 })
    }

    /// Unchecked constructor. Caller guarantees finiteness.
    /// # Safety
    /// UB if `v` is NaN or Inf only in the sense of logic errors downstream.
    #[inline]
    pub const unsafe fn new_unchecked(v: f32) -> Self {
        Self(v)
    }

    #[inline]
    pub const fn get(self) -> f32 {
        self.0
    }

    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
}

impl fmt::Debug for FiniteF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FiniteF32({})", self.0)
    }
}

impl fmt::Display for FiniteF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for FiniteF32 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Total ordering is safe because NaN is excluded by construction.
impl Ord for FiniteF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Eq for FiniteF32 {}

impl std::hash::Hash for FiniteF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl std::ops::Add for FiniteF32 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.0 + rhs.0)
    }
}

impl std::ops::Sub for FiniteF32 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.0 - rhs.0)
    }
}

impl std::ops::Mul for FiniteF32 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.0 * rhs.0)
    }
}

// ---------------------------------------------------------------------------
// PosF32 — strictly positive finite f32 (FM-RES-001, FM-RES-002, FM-ACT-012)
// Replaces: wavelength, frequency, half_life_hours, decay_rate, threshold
// ---------------------------------------------------------------------------

/// Positive non-zero finite f32. Division by PosF32 can never produce NaN/Inf.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PosF32(f32);

impl PosF32 {
    /// Returns `None` if v is not strictly positive and finite.
    #[inline]
    pub fn new(v: f32) -> Option<Self> {
        if v > 0.0 && v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    #[inline]
    pub const fn get(self) -> f32 {
        self.0
    }

    /// 1.0 — always valid.
    pub const ONE: Self = Self(1.0);
}

impl fmt::Debug for PosF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PosF32({})", self.0)
    }
}

// ---------------------------------------------------------------------------
// LearningRate — (0.0, 1.0] (FM-PL-010)
// Replaces: Python DEFAULT_LEARNING_RATE = 0.08
// ---------------------------------------------------------------------------

/// Learning rate in the half-open interval (0.0, 1.0].
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct LearningRate(f32);

impl LearningRate {
    pub fn new(v: f32) -> Option<Self> {
        if v > 0.0 && v <= 1.0 && v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    #[inline]
    pub const fn get(self) -> f32 {
        self.0
    }

    /// Default: 0.08 (from Python PoC plasticity.py DEFAULT_LEARNING_RATE)
    pub const DEFAULT: Self = Self(0.08);
}

// ---------------------------------------------------------------------------
// DecayFactor — (0.0, 1.0] for signal decay per hop (FM-ACT-012)
// Replaces: Python decay=0.55 in D1_Structural
// ---------------------------------------------------------------------------

/// Decay factor in (0.0, 1.0]. Signal multiplied by this per hop.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct DecayFactor(f32);

impl DecayFactor {
    pub fn new(v: f32) -> Option<Self> {
        if v > 0.0 && v <= 1.0 && v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    #[inline]
    pub const fn get(self) -> f32 {
        self.0
    }

    /// Default: 0.55 (from Python PoC engine_v2.py D1_Structural)
    pub const DEFAULT: Self = Self(0.55);
}

// ---------------------------------------------------------------------------
// Thin index wrappers for type-safe graph indices
// ---------------------------------------------------------------------------

/// Node index into NodeStorage arrays. u32 supports up to ~4B nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NodeId(pub u32);

impl NodeId {
    #[inline]
    pub const fn new(v: u32) -> Self {
        Self(v)
    }
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Edge index into CSR parallel arrays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EdgeIdx(pub u32);

impl EdgeIdx {
    #[inline]
    pub const fn new(v: u32) -> Self {
        Self(v)
    }
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Interned string index. Opaque handle into StringInterner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InternedStr(pub u32);

/// Community identifier from Louvain detection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CommunityId(pub u32);

/// Generation counter for graph mutation detection (FM-PL-006).
/// PlasticityEngine stores this at init; every operation asserts equality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Generation(pub u64);

impl Generation {
    #[inline]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Edge direction in CSR (from Python EdgeDirection).
/// Replaces: engine_v2.py Edge.direction field
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EdgeDirection {
    Forward = 0,
    Bidirectional = 1,
}

/// Node type tag (from Python Node.type field).
/// Replaces: engine_v2.py Node.type string + ingest.py type assignment
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeType {
    File = 0,
    Directory = 1,
    Function = 2,
    Class = 3,
    Struct = 4,
    Enum = 5,
    Type = 6,
    Module = 7,
    Reference = 8,
    Concept = 9,
    Material = 10,
    Process = 11,
    Product = 12,
    Supplier = 13,
    Regulatory = 14,
    System = 15,
    Cost = 16,
    Custom(u8),
}

/// Activation dimension selector.
/// Replaces: engine_v2.py dimensions=["structural","semantic","temporal","causal"]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dimension {
    Structural = 0,
    Semantic = 1,
    Temporal = 2,
    Causal = 3,
}

/// Dimension weight constants from 02-FEATURES-EDGE-CASES.md Section 1.
pub const DIMENSION_WEIGHTS: [f32; 4] = [0.35, 0.25, 0.15, 0.25];

/// Resonance bonus multipliers (02-FEATURES Section 1.6).
/// FM-ACT-001 FIX: check >=4 BEFORE >=3 (dead elif branch in Python).
pub const RESONANCE_BONUS_3DIM: f32 = 1.3;
pub const RESONANCE_BONUS_4DIM: f32 = 1.5;

// ---------------------------------------------------------------------------
// Configuration aggregates
// ---------------------------------------------------------------------------

/// Propagation parameters for structural activation (D1).
/// Replaces: engine_v2.py D1_Structural.__init__ parameters
#[derive(Clone, Debug)]
pub struct PropagationConfig {
    pub decay: DecayFactor,              // default 0.55
    pub threshold: PosF32,               // default 0.04
    pub max_depth: u8,                   // default 5, max 20 (FM-ACT-012)
    pub saturation_cap: FiniteF32,       // default 1.0
    pub inhibitory_factor: FiniteF32,    // default 0.5
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            decay: DecayFactor::DEFAULT,
            threshold: PosF32::new(0.04).unwrap(),
            max_depth: 5,
            saturation_cap: FiniteF32::ONE,
            inhibitory_factor: FiniteF32::new(0.5),
        }
    }
}

/// Semantic matching weights from semantic_v2.py query_fast.
pub struct SemanticWeights {
    pub ngram: FiniteF32,       // default 0.4
    pub cooccurrence: FiniteF32, // default 0.4
    pub synonym: FiniteF32,     // default 0.2
}

impl Default for SemanticWeights {
    fn default() -> Self {
        Self {
            ngram: FiniteF32::new(0.4),
            cooccurrence: FiniteF32::new(0.4),
            synonym: FiniteF32::new(0.2),
        }
    }
}

/// Temporal scoring weights from engine_v2.py D3_Temporal.
pub struct TemporalWeights {
    pub recency: FiniteF32,     // default 0.6
    pub frequency: FiniteF32,   // default 0.4
}

impl Default for TemporalWeights {
    fn default() -> Self {
        Self {
            recency: FiniteF32::new(0.6),
            frequency: FiniteF32::new(0.4),
        }
    }
}
