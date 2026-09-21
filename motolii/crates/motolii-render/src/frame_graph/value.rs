use std::collections::BTreeMap;

use crate::doc::core::RationalTime;

use super::cache::NodeValue;
use super::key::{FrameQuality, NodeKey};

/// Ordered by the native playback clock. A later generation supersedes every
/// unfinished submission from an earlier generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Generation(u64);

impl Generation {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A compiled document identity. It intentionally differs from generation:
/// seek/playback advances generations without recompiling topology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphRevision(pub(crate) u64);

impl GraphRevision {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameState {
    Current,
    Stale,
}

/// The shared upstream result of one graph evaluation. It has no document
/// reader; views can only project these already-evaluated nodes.
#[derive(Clone, Debug)]
pub struct EvaluatedFrame {
    pub(crate) revision: GraphRevision,
    pub(crate) generation: Generation,
    pub(crate) time: RationalTime,
    pub(crate) quality: FrameQuality,
    pub(crate) state: FrameState,
    pub(crate) values: BTreeMap<NodeKey, NodeValue>,
    pub(crate) executed: Vec<NodeKey>,
    pub(crate) reused: Vec<NodeKey>,
}

impl EvaluatedFrame {
    pub fn revision(&self) -> GraphRevision {
        self.revision
    }
    pub fn generation(&self) -> Generation {
        self.generation
    }
    pub fn time(&self) -> RationalTime {
        self.time
    }
    pub fn quality(&self) -> FrameQuality {
        self.quality
    }
    pub fn state(&self) -> FrameState {
        self.state
    }
    pub fn value(&self, key: NodeKey) -> Option<&NodeValue> {
        self.values.get(&key)
    }
    pub fn executed_nodes(&self) -> &[NodeKey] {
        &self.executed
    }
    pub fn reused_nodes(&self) -> &[NodeKey] {
        &self.reused
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewProjection {
    Camera,
    Stage,
    ReflectionProbe,
    Export,
}

/// An opaque host/compositor target. FrameGraph owns scheduling, not GPU
/// resources or IOSurfaces, so their concrete handles remain at the renderer
/// and native boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderTarget(u64);

impl RenderTarget {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    pub const fn id(self) -> u64 {
        self.0
    }
}

/// A request to render one view of an evaluated frame. Publishing remains a
/// separate graph-owned generation check.
#[derive(Clone, Debug)]
pub struct Submission {
    pub(crate) revision: GraphRevision,
    pub(crate) generation: Generation,
    pub(crate) projection: ViewProjection,
    pub(crate) target: RenderTarget,
    pub(crate) state: FrameState,
}

impl Submission {
    pub fn generation(&self) -> Generation {
        self.generation
    }
    pub fn projection(&self) -> ViewProjection {
        self.projection
    }
    pub fn target(&self) -> RenderTarget {
        self.target
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublishedFrame {
    pub generation: Generation,
    pub target: RenderTarget,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphStats {
    pub topology_compiles: u64,
    pub node_executions: u64,
    pub node_reuses: u64,
    pub cancelled_generations: u64,
    pub cancelled_evaluations: u64,
    pub cached_results: usize,
}
