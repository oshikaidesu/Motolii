use crate::render::compositor::LayerWithPasses;

/// Concrete composite payload. Relationship identity/version and scheduling are
/// owned by the resource graph; this module deliberately contains no semantic
/// scene discovery.
#[derive(Clone)]
pub(crate) struct ResidentCompositeLayer {
    pub layer: LayerWithPasses,
}
