//! The only intended bridge from evaluated Motolii semantics to backend-neutral
//! render work.
//!
//! The existing GpuScene path remains a frozen reference implementation while this
//! boundary is brought up vertically. Do not move Motolii semantics into RenderWork.

use crate::frame_graph::SceneValue;
use crate::render_graph::RenderGraph;

#[derive(Debug)]
pub enum RenderLoweringError {
    Unsupported,
}

/// Lower an evaluated semantic scene into backend-neutral render work.
///
/// Bootstrap intentionally returns an empty graph: the first production slice will
/// be copied from the existing GpuScene behavior one operation at a time, with
/// parity evidence before the old path is removed.
pub fn lower_scene(_scene: &SceneValue) -> Result<RenderGraph, RenderLoweringError> {
    Ok(RenderGraph::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scene_lowers_without_backend_knowledge() {
        let graph = lower_scene(&SceneValue::default()).unwrap();
        assert!(graph.is_empty());
    }
}
