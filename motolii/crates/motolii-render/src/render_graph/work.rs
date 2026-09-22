use crate::doc::store::{BlendMode, LayerId, LayerProjection};
use crate::frame_graph::TransformValue;

/// Opaque resource identity inside one lowered render graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

/// Backend-neutral description of one already-evaluated scene contribution.
///
/// This is intentionally smaller than `SceneLayerValue`: it contains only values
/// needed to schedule/composite a contribution and does not carry Motolii node
/// identities or wgpu resources.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeItem {
    pub layer: LayerId,
    pub transform: TransformValue,
    pub opacity: f32,
    pub projection: LayerProjection,
    pub blend: BlendMode,
    pub order: i16,
}

/// Small backend-neutral execution vocabulary.
///
/// These are work families, not Motolii semantic nodes. Payloads grow only when a
/// production vertical slice needs them.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderWork {
    Raster { output: ResourceId },
    Compute { output: ResourceId },
    Filter { input: ResourceId, output: ResourceId },
    Composite { items: Vec<CompositeItem>, output: ResourceId },
    Transfer { input: Option<ResourceId>, output: ResourceId },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderGraph {
    work: Vec<RenderWork>,
}

impl RenderGraph {
    pub fn new(work: Vec<RenderWork>) -> Self { Self { work } }
    pub fn work(&self) -> &[RenderWork] { &self.work }
    pub fn is_empty(&self) -> bool { self.work.is_empty() }
}
