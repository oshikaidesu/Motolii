use crate::doc::store::{BlendMode, LayerId, LayerProjection};
use crate::frame_graph::TransformValue;

/// Opaque resource identity inside one lowered render graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

/// Backend-neutral source recipe. It describes the class of resource required by
/// rendering without carrying wgpu handles or re-encoding Motolii semantic nodes.
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceSource {
    Text,
    Shape,
    Material { path: String },
    Media { path: String },
    Particles,
    Plate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompositeItem {
    pub layer: LayerId,
    pub resource: Option<ResourceId>,
    pub transform: TransformValue,
    pub opacity: f32,
    pub projection: LayerProjection,
    pub blend: BlendMode,
    pub order: i16,
}

/// Small backend-neutral execution vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderWork {
    Raster { source: ResourceSource, output: ResourceId },
    Compute { output: ResourceId },
    Filter { input: ResourceId, output: ResourceId },
    Composite { items: Vec<CompositeItem>, output: ResourceId },
    Transfer { source: ResourceSource, output: ResourceId },
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
