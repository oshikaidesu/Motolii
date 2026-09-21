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

/// Mapping from a consumer's exact comp time to one input's time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputTime {
    Same,
    Offset { delta: RationalTime, clamp_to_zero: bool },
}

impl InputTime {
    pub fn apply(self, time: RationalTime) -> RationalTime {
        match self {
            Self::Same => time,
            Self::Offset { delta, clamp_to_zero } => time.try_add(delta).ok()
                .filter(|value| !clamp_to_zero || *value >= RationalTime::ZERO)
                .unwrap_or(RationalTime::ZERO),
        }
    }
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
    PropertyConstant,
    PropertyTrack,
    PropertyLink,
    PropertySum,
    /// Coarse F2 world evaluation, before per-content node lowering.
    ResolvedWorld,
    /// Text documents collected from one resolved world.
    TextDocuments,
    /// Shape documents collected from one resolved world.
    ShapeDocuments,
    /// The authored camera resolved from one world.
    DocumentCamera,
    /// Coarse shared scene passed to final view projections.
    SharedScene,
    /// Canonical authored text content and its run/style references.
    TextContent,
    /// Canonical text style/font input.  It is separate so equal styles can
    /// be shared by otherwise different text documents.
    TextStyle,
    /// Shaped text paths/contours after the layout input is known.
    TextShape,
    /// Authored vector geometry plus evaluated shape/fill properties.
    ShapeGeometry,
    /// Shape path tessellation/raster input.  The renderer chooses the value.
    ShapeMesh,
    /// A group authoring/topology descriptor, before its time-varying layout.
    Group,
    /// The layout/slot result for a group and its direct children.
    Layout,
    /// Transition/stagger blend over exact-time Layout results.
    FlowWindow,
    /// A display group's generated background/clip geometry.
    GroupBackground,
    /// Resource dimensions and immutable source metadata.
    MediaExtent,
    /// Imported mesh/resource geometry.
    MeshSource,
    /// Material/resource surface input for a file or mesh.
    Material,
    /// A time-addressed media frame (video) or image source.
    MediaFrame,
    /// Exact-time authored participation: hidden/solo/timing.
    Visibility,
    /// Per-frame cumulative emission state for a particle layer.
    ParticleBirths,
    /// Semantic particle positions/colors/sizes for the exact composition time.
    Particle,
    /// A fixed authored temporal duplicate (Ghost) of a scene contribution.
    TemporalCopy,
    /// Evaluated placement-effect outputs before they become scene copies.
    PlacementSet,
    /// Motion-blur shutter measurement; samples only the two shutter edges.
    MotionMeasure,
    /// Exact motion-blur transform samples selected by MotionMeasure.
    MotionSamples,
    /// Authored layer-to-layer relation inputs (Follow, connector, trace).
    Relation,
    /// Exact-time relation table consumed by motion/solver lowering.
    RelationSet,
    /// Authored local placement after layout.
    Transform,
    /// Inherited world placement.
    WorldTransform,
    /// Resolved document/observer camera state.
    Camera,
    /// A view-specific camera/stage projection.
    CameraProjection,
    /// One authored layer's ordered contribution to the scene.
    CompositeContribution,
    /// A Group scope over its direct children. It becomes a plate only when an evaluated Whole effect requires one.
    GroupComposite,
    /// Attaches named-layer and temporal image dependencies to evaluated effects.
    EffectImages,
    /// The ordered shared scene before any view projection.
    SceneComposite,
    /// GPU-ready layer resources shared by every final projection.
    GpuScene,
    /// Effect node reserved for the later F4 lowering.
    Effect,
    Mask,
    Custom(u16),
}

/// A content-addressed node identity. `parameters` must be the complete,
/// canonical input encoding for the node kind; callers must not substitute a
/// layer id or document revision for those inputs.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeIdentity {
    pub kind: NodeKind,
    pub inputs: Vec<NodeKey>,
    pub input_times: Vec<InputTime>,
    pub parameters: Vec<u8>,
    pub time_dependency: TimeDependency,
    pub source_versions: Vec<u64>,
    pub quality_dependency: QualityDependency,
}

impl NodeIdentity {
    pub fn new(kind: NodeKind, inputs: Vec<NodeKey>) -> Self {
        let input_times = vec![InputTime::Same; inputs.len()];
        Self {
            kind,
            inputs,
            input_times,
            parameters: Vec::new(),
            time_dependency: TimeDependency::Static,
            source_versions: Vec::new(),
            quality_dependency: QualityDependency::Invariant,
        }
    }

    pub fn with_input_times(mut self, input_times: Vec<InputTime>) -> Self {
        assert_eq!(self.inputs.len(), input_times.len(), "every FrameGraph input needs one time mapping");
        self.input_times = input_times;
        self
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
