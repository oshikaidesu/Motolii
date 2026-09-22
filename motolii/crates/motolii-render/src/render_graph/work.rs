/// Opaque resource identity inside one lowered render graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub u64);

/// Small backend-neutral execution vocabulary.
///
/// These are work families, not Motolii semantic nodes. Payloads stay deliberately
/// minimal while the first vertical slice is wired from the existing GpuScene
/// reference path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderWork {
    Raster { output: ResourceId },
    Compute { output: ResourceId },
    Filter { input: ResourceId, output: ResourceId },
    Composite { inputs: Vec<ResourceId>, output: ResourceId },
    Transfer { input: Option<ResourceId>, output: ResourceId },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderGraph {
    work: Vec<RenderWork>,
}

impl RenderGraph {
    pub fn new(work: Vec<RenderWork>) -> Self { Self { work } }
    pub fn work(&self) -> &[RenderWork] { &self.work }
    pub fn is_empty(&self) -> bool { self.work.is_empty() }
}
