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

/// Per-contribution lowering input. The semantic adapter constructs these from
/// contribution/content/transform/effect/mask bindings. The GPU lowerer never
/// receives StoreView or the whole Document.
#[derive(Clone, Debug)]
pub(crate) struct GpuContributionInput {
    pub contribution: VersionedSemantic,
    pub instance: u32,
    pub content: Option<VersionedSemantic>,
    pub placement: VersionedSemantic,
    pub direct_effects: Vec<VersionedSemantic>,
    pub after_effects: Vec<VersionedSemantic>,
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
    pub stencil: bool,
    pub plate: Option<VersionedSemantic>,
}

/// Logical resources produced by lowering one contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuContributionResources {
    pub content: Option<GpuResourceKey>,
    pub placement: GpuResourceKey,
    /// Stable contribution output declared before cross-contribution edges are
    /// connected. Matte/clip/composition backends must address this resource
    /// directly instead of rediscovering layers from SceneValue.
    pub contribution: GpuResourceKey,
    pub final_image_or_geometry: Option<GpuResourceKey>,
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
