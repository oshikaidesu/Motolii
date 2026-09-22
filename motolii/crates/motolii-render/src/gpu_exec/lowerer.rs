use crate::frame_graph::NodeKey;

use super::graph::{GpuGraphError, GpuResourceGraph};
use super::types::{GpuResourceKey, GpuResourceVersion};

/// A semantic recipe plus the version of its currently evaluated value.
///
/// The version changes only when the GPU-visible value changes. Root frame
/// time is never a valid version by itself: static values keep their version
/// while playback advances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VersionedSemantic {
    pub node: NodeKey,
    pub version: GpuResourceVersion,
}

/// An effect's version handle plus the resolved parameters it was hashed
/// from. Backends that need real effect parameters (not just a cache key)
/// read `value`; the graph/version bookkeeping still flows through `semantic`
/// exactly like every other `VersionedSemantic` consumer.
#[derive(Clone, Debug)]
pub(crate) struct VersionedEffect {
    pub semantic: VersionedSemantic,
    pub value: crate::picture::resolved::ResolvedEffect,
}

/// Per-contribution lowering input. The semantic adapter constructs these from
/// contribution/content/transform/effect/mask bindings. The GPU lowerer never
/// receives StoreView or the whole Document.
#[derive(Clone, Debug)]
pub(crate) struct GpuContributionInput {
    pub contribution: VersionedSemantic,
    pub instance: u32,
    pub content: Option<VersionedSemantic>,
    pub placement: VersionedSemantic,
    /// Blend mode is always an explicit authoring value. Its version hashes
    /// only the declared `BlendMode`, never derived render state.
    pub blend: VersionedSemantic,
    /// Projection is always an explicit authoring-time pin. HARD CONSTRAINT:
    /// its version hashes only the declared `LayerProjection`, never
    /// evaluated/resolved geometry, transform, or camera state.
    pub projection: VersionedSemantic,
    /// Plain derived payload, same shape/treatment as `stencil`: computed
    /// once at the semantic adapter lowering stage from `LayerSource`.
    pub is_file_source: bool,
    pub direct_effects: Vec<VersionedEffect>,
    pub after_effects: Vec<VersionedEffect>,
    pub image_sources: Vec<Vec<crate::frame_graph::SceneImageSourceValue>>,
    pub masks: Vec<VersionedSemantic>,
    /// Explicit cross-contribution dependency. The semantic adapter resolves
    /// authoring LayerId to a stable contribution resource exactly once.
    pub matte_source: Option<GpuResourceKey>,
    /// Explicit clip/base contribution edge. Kept separate from matte because
    /// clip consumes the current contribution into its base rather than
    /// producing an ordinary matte result for the current contribution.
    pub clip_base: Option<GpuResourceKey>,
    pub clip_to_below: bool,
    /// Concrete relation mode is payload, not discovery input. The adapter
    /// resolves it once; backends execute the planned edge without consulting
    /// SceneValue again.
    pub matte_mode: Option<crate::doc::store::MatteMode>,
    pub stencil: bool,
    pub plate: Option<VersionedSemantic>,
}

/// Logical resources produced by lowering one contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuContributionResources {
    pub content: Option<GpuResourceKey>,
    pub placement: GpuResourceKey,
    pub blend: GpuResourceKey,
    pub projection: GpuResourceKey,
    /// Stable contribution output declared before cross-contribution edges are
    /// connected. Matte/clip/composition backends must address this resource
    /// directly instead of rediscovering layers from SceneValue.
    pub contribution: GpuResourceKey,
    /// Per-contribution output before cross-contribution relations. Clip/matte
    /// are lowered only after every contribution has reached this stage.
    pub prepared: Option<GpuResourceKey>,
    /// Snapshot resource keys grouped exactly like the effect image-source rows.
    /// Concrete contribution materialization reads these stores directly; it
    /// never rediscovers temporal sources from SceneLayerValue.
    pub snapshot_rows: Vec<Vec<GpuResourceKey>>,
    pub operations: Vec<(super::types::GpuPassKey, super::engine_backend::EngineGpuOperation)>,
}

/// Greenfield lowering boundary.
///
/// Implementations add/update logical resources and producer passes. They do
/// not record or submit GPU commands.
pub(crate) trait GpuLowerer {
    /// Phase 1 declares stable contribution output identities before any
    /// cross-contribution edge (matte/clip/scene composition) is connected.
    fn declare_contribution(
        &mut self,
        graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuResourceKey, Self::Error>;


    type Error: From<GpuGraphError>;

    fn lower_contribution(
        &mut self,
        graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuContributionResources, Self::Error>;
}
