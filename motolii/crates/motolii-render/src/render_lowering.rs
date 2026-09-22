//! The only intended bridge from evaluated Motolii semantics to backend-neutral
//! render work. The existing GpuScene path remains the rendering oracle.

use crate::frame_graph::{SceneContentValue, SceneValue};
use crate::render_graph::{CompositeItem, RenderGraph, RenderWork, ResourceId, ResourceSource};

#[derive(Debug)]
pub enum RenderLoweringError { Unsupported }

fn resource_for(content: &SceneContentValue) -> Option<(ResourceSource, bool)> {
    match content {
        SceneContentValue::None => None,
        SceneContentValue::Text(_) => Some((ResourceSource::Text, true)),
        SceneContentValue::Shape(_) => Some((ResourceSource::Shape, true)),
        SceneContentValue::Material(material) => Some((ResourceSource::Material { path: material.source.path.clone() }, false)),
        SceneContentValue::Media { source, .. } => Some((ResourceSource::Media { path: source.path.clone() }, false)),
        SceneContentValue::Particles(_) => Some((ResourceSource::Particles, true)),
        SceneContentValue::Plate(_) => Some((ResourceSource::Plate, true)),
    }
}

/// Lower evaluated scene meaning into resource work followed by one ordered
/// composite. Resource ids are local to this graph and deliberately unrelated to
/// semantic NodeKey identity.
pub fn lower_scene(scene: &SceneValue) -> Result<RenderGraph, RenderLoweringError> {
    if scene.layers.is_empty() { return Ok(RenderGraph::default()); }
    let mut work = Vec::new();
    let mut items = Vec::with_capacity(scene.layers.len());
    let mut next = 1u64;
    for layer in &scene.layers {
        let resource = resource_for(&layer.content).map(|(source, raster)| {
            let id = ResourceId(next); next += 1;
            work.push(if raster {
                RenderWork::Raster { source, output: id }
            } else {
                RenderWork::Transfer { source, output: id }
            });
            id
        });
        items.push(CompositeItem {
            layer: layer.layer, resource, transform: layer.transform, opacity: layer.opacity,
            projection: layer.projection, blend: layer.blend, order: layer.order,
        });
    }
    work.push(RenderWork::Composite { items, output: ResourceId(0) });
    Ok(RenderGraph::new(work))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{BlendMode, LayerId, LayerProjection, LayerSource};
    use crate::frame_graph::{SceneLayerValue, TransformValue};

    fn layer(content: SceneContentValue) -> SceneLayerValue {
        SceneLayerValue {
            layer: LayerId(7), instance: 0, source: LayerSource::Null,
            transform: TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY },
            content_key: None, content, effects: Vec::new(), after_effects: Vec::new(),
            image_sources: Vec::new(), masks: Vec::new(), matte: None, clip_to_below: false,
            flatten: false, environment: false, ghost: false, freeze_eligible: false,
            timing_start: 0, opacity: 0.5, projection: LayerProjection::TwoD,
            blend: BlendMode::Normal, order: 3, shape_stretch: [1.0, 1.0], depth: 0.0,
        }
    }

    #[test]
    fn empty_scene_lowers_without_backend_knowledge() {
        assert!(lower_scene(&SceneValue::default()).unwrap().is_empty());
    }

    #[test]
    fn contentless_scene_still_lowers_ordered_composite() {
        let graph = lower_scene(&SceneValue { layers: vec![layer(SceneContentValue::None)] }).unwrap();
        let [RenderWork::Composite { items, .. }] = graph.work() else { panic!("expected composite") };
        assert_eq!(items[0].resource, None);
        assert_eq!(items[0].opacity, 0.5);
    }

    #[test]
    fn semantic_content_becomes_resource_work_before_composite() {
        let graph = lower_scene(&SceneValue { layers: vec![layer(SceneContentValue::Shape(Vec::new()))] }).unwrap();
        assert!(matches!(graph.work()[0], RenderWork::Raster { source: ResourceSource::Shape, output: ResourceId(1) }));
        let RenderWork::Composite { items, .. } = &graph.work()[1] else { panic!("expected composite") };
        assert_eq!(items[0].resource, Some(ResourceId(1)));
    }
}
