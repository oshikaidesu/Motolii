use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::doc::core::RationalTime;

/// The fidelity requested for a node result. Quality-sensitive recipes use it
/// in their `WorkKey`, so a preview result is never an export result by accident.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FrameQuality {
    Preview { scale: u8 },
    Export,
}

/// Whether a node's result is keyed by the exact composition time passed to
/// `evaluate`. This is recipe metadata, not part of compiled topology identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TimeDependency {
    Static,
    Exact,
}

/// Whether preview and export may share the same result. This is recipe
/// metadata, not a concrete preview/export choice captured at compile time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum QualityDependency {
    Invariant,
    Sensitive,
}

/// The semantic role of a graph node. New evaluators may add roles without
/// changing the graph's ownership or scheduling contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeKind {
    Content,
    Transform,
    Effect,
    Composite,
    World,
    Projection,
    Custom(u16),
}

/// A content-addressed node identity. `parameters` must be the complete,
/// canonical input encoding for the node kind; callers must not substitute a
/// layer id or document revision for those inputs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeIdentity {
    pub kind: NodeKind,
    pub inputs: Vec<NodeKey>,
    pub parameters: Vec<u8>,
    pub time_dependency: TimeDependency,
    pub source_versions: Vec<u64>,
    pub quality_dependency: QualityDependency,
}

impl NodeIdentity {
    pub fn new(kind: NodeKind, inputs: Vec<NodeKey>) -> Self {
        Self {
            kind,
            inputs,
            parameters: Vec::new(),
            time_dependency: TimeDependency::Static,
            source_versions: Vec::new(),
            quality_dependency: QualityDependency::Invariant,
        }
    }
}

/// The cached result identity for one node recipe under a particular frame
/// context. It deliberately includes only dimensions the recipe consumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkKey {
    node: NodeKey,
    time: Option<RationalTime>,
    quality: Option<FrameQuality>,
}

impl WorkKey {
    pub fn for_node(identity: &NodeIdentity, time: RationalTime, quality: FrameQuality) -> Self {
        Self {
            node: NodeKey::for_identity(identity),
            time: matches!(identity.time_dependency, TimeDependency::Exact).then_some(time),
            quality: matches!(identity.quality_dependency, QualityDependency::Sensitive)
                .then_some(quality),
        }
    }

    pub fn node(self) -> NodeKey {
        self.node
    }
}

/// A compact lookup key. `GraphTopology` retains and compares the complete
/// [`NodeIdentity`] so a hash collision cannot silently become identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeKey(u64);

impl NodeKey {
    pub fn for_identity(identity: &NodeIdentity) -> Self {
        let mut hasher = DefaultHasher::new();
        identity.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}
