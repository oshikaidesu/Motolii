//! The only intended bridge from evaluated Motolii semantics to backend-neutral
//! render work.
//!
//! The existing GpuScene path remains the rendering reference implementation while
//! this boundary is brought up vertically. Do not move Motolii semantics into
//! RenderWork.

use crate::frame_graph::SceneValue;
use crate::render_graph::{CompositeItem, RenderGraph, RenderWork, ResourceId};

#[derive(Debug)]
pub enum RenderLoweringError {
    Unsupported,
}

/// First vertical slice: lower the already-evaluated ordered scene into one
/// backend-neutral composite work item.
///
/// Content rasterization/effects/mattes still run through the frozen GpuScene
/// reference path. This slice proves that ordering/placement/blend no longer need a
/// GPU semantic node identity before the heavier resource work is copied across.
pub fn lower_scene(scene: &SceneValue) -> Result<RenderGraph, RenderLoweringError> {
    if scene.layers.is_empty() {
        return Ok(RenderGraph::default());
    }
    let items = scene.layers.iter().map(|layer| CompositeItem {
        layer: layer.layer,
        transform: layer.transform,
        opacity: layer.opacity,
        projection: layer.projection,
        blend: layer.blend,
        order: layer.order,
    }).collect();
    Ok(RenderGraph::new(vec![RenderWork::Composite {
        items,
        output: ResourceId(0),
    }]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{BlendMode, LayerId, LayerProjection, LayerSource};
    use crate::frame_graph::{SceneContentValue, SceneLayerValue, TransformValue};

    #[test]
    fn empty_scene_lowers_without_backend_knowledge() {
        assert!(lower_scene(&SceneValue::default()).unwrap().is_empty());
    }

    #[test]
    fn scene_order_and_placement_lower_to_composite_work() {
        let transform = TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY };
        let layer = SceneLayerValue {
            layer: LayerId(7), instance: 0, source: LayerSource::Null, transform,
            content_key: None, content: SceneContentValue::None, effects: Vec::new(),
            after_effects: Vec::new(), image_sources: Vec::new(), masks: Vec::new(),
            matte: None, clip_to_below: false, flatten: false, environment: false,
            ghost: false, freeze_eligible: false, timing_start: 0, opacity: 0.5,
            projection: LayerProjection::TwoD, blend: BlendMode::Normal, order: 3,
            shape_stretch: [1.0, 1.0], depth: 0.0,
        };
        let graph = lower_scene(&SceneValue { layers: vec![layer] }).unwrap();
        let [RenderWork::Composite { items, .. }] = graph.work() else { panic!("expected one composite work item") };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].layer, LayerId(7));
        assert_eq!(items[0].opacity, 0.5);
        assert_eq!(items[0].order, 3);
    }
}
