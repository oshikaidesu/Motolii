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
    pub effects: Vec<VersionedSemantic>,
    pub masks: Vec<VersionedSemantic>,
    pub matte_source: Option<GpuResourceKey>,
    pub plate: Option<VersionedSemantic>,
}

/// Logical resources produced by lowering one contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GpuContributionResources {
    pub content: Option<GpuResourceKey>,
    pub placement: GpuResourceKey,
    pub final_image_or_geometry: Option<GpuResourceKey>,
}

/// Greenfield lowering boundary.
///
/// Implementations add/update logical resources and producer passes. They do
/// not record or submit GPU commands.
pub(crate) trait GpuLowerer {
    type Error: From<GpuGraphError>;

    fn lower_contribution(
        &mut self,
        graph: &mut GpuResourceGraph,
        input: &GpuContributionInput,
    ) -> Result<GpuContributionResources, Self::Error>;
}
