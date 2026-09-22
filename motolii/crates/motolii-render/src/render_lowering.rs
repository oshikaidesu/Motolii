//! The only bridge from evaluated Motolii semantics to backend-neutral render
//! work. Every Motolii meaning a picture depends on is decided here; backends
//! execute the resulting graph without reading semantic values.

use std::sync::Arc;

use crate::doc::core::LayerPlacement;
use crate::doc::store::{LayerId, LayerProjection, LayerSource};
use crate::frame_graph::{plan_composite, PlannedContribution, SceneContentValue, SceneImageSourceValue, SceneLayerValue, SceneValue};
use crate::render::compositor::effects::catalog::CatalogSnapshot;
use crate::render::compositor::effects::surface_program::SurfaceRecipe;
use crate::render::compositor::extrude::Solid;
use crate::render::engine::translate;
use crate::render_graph::{Composed, Extrusion, ImageInput, LayerWork, RasterSource, RenderGraph};

mod blocks;

pub use blocks::{plan_blocks, BlockBatch, BlockPlan, Placed};

#[derive(Debug)]
pub enum RenderLoweringError {
    Unsupported(String),
}

impl std::fmt::Display for RenderLoweringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { Self::Unsupported(message) => f.write_str(message) }
    }
}

pub fn lower_scene(scene: &SceneValue, catalog: &CatalogSnapshot) -> Result<RenderGraph, RenderLoweringError> {
    lower(scene, catalog, false)
}

/// Every contribution drawn as a material-space picture, for a host that reads
/// the pixels back (freeze, analysis).
pub fn lower_scene_as_pictures(scene: &SceneValue, catalog: &CatalogSnapshot) -> Result<RenderGraph, RenderLoweringError> {
    lower(scene, catalog, true)
}

/// `pictures`: every contribution needs pixels in material space (summed by an
/// averaging plate, or read back by the host).
fn lower(scene: &SceneValue, catalog: &CatalogSnapshot, pictures: bool) -> Result<RenderGraph, RenderLoweringError> {
    let plan = plan_composite(scene);
    let clip_bases: std::collections::HashSet<usize> = plan.iter()
        .flat_map(|planned| {
            fn collect<'a>(planned: &'a PlannedContribution, groups: &mut Vec<&'a crate::frame_graph::ClipGroup>) {
                groups.push(&planned.group);
                for source in planned.matte.iter().flat_map(|matte| &matte.sources) { collect(source, groups); }
            }
            let mut groups = Vec::new();
            collect(planned, &mut groups);
            groups
        })
        .filter(|group| !group.clips.is_empty())
        .map(|group| group.base)
        .collect();
    let layers = scene.layers.iter().enumerate()
        .map(|(index, layer)| lower_layer(layer, clip_bases.contains(&index), pictures, catalog))
        .collect::<Result<_, _>>()?;
    Ok(RenderGraph { layers, output: plan.iter().map(composed).collect() })
}

fn composed(planned: &PlannedContribution) -> Composed {
    Composed {
        base: planned.group.base,
        atop: planned.group.clips.clone(),
        mask: planned.matte.as_ref().map(|matte| (matte.sources.iter().map(composed).collect(), translate::translate_matte_mode(matte.mode))),
    }
}

fn stretched(layer: &SceneLayerValue, shapes: &[crate::doc::store::ShapeNode]) -> Arc<Vec<crate::doc::store::ShapeNode>> {
    Arc::new(if layer.shape_stretch != [1.0, 1.0] {
        crate::picture::shapes_ops::stretch_outline(shapes, layer.shape_stretch)
    } else {
        shapes.to_vec()
    })
}

fn raster(layer: &SceneLayerValue, vector: bool, field_step: bool, catalog: &CatalogSnapshot) -> Result<RasterSource, RenderLoweringError> {
    Ok(match &layer.content {
        SceneContentValue::None => RasterSource::None,
        SceneContentValue::Text(text) if !vector => RasterSource::Vector { shapes: Arc::new(text.shapes()), vector: false, remember: true, field_step: false },
        SceneContentValue::Text(text) => RasterSource::CanvasVector { shapes: Arc::new(text.shapes()) },
        SceneContentValue::Shape(shapes) => RasterSource::Vector {
            shapes: stretched(layer, shapes),
            vector,
            remember: layer.shape_stretch == [1.0, 1.0],
            field_step,
        },
        SceneContentValue::Material(material) => RasterSource::Mesh { path: material.source.path.clone() },
        SceneContentValue::Media { source, .. } if layer.environment && crate::render::media::is_still_image_path(&source.path) => {
            RasterSource::EnvironmentMap { path: source.path.clone() }
        }
        SceneContentValue::Media { source, time } => RasterSource::Image { path: source.path.clone(), time: *time },
        SceneContentValue::Particles(value) => {
            let frame = crate::render::engine::ParticleFrame::from_particles(&value.particles, value.turbulence, value.links);
            RasterSource::Points { positions: frame.positions, colors: frame.colors, sizes: frame.sizes, bounds: frame.bounds, links: frame.links }
        }
        SceneContentValue::Plate(plate) => RasterSource::Isolate {
            graph: Arc::new(lower(&plate_scene(plate), catalog, plate.average)?),
            average: plate.average,
        },
    })
}

fn plate_scene(plate: &crate::frame_graph::ScenePlateValue) -> SceneValue {
    SceneValue { layers: plate.members.iter().filter_map(|member| member.layer.clone()).collect() }
}

fn image_input(source: &SceneImageSourceValue, catalog: &CatalogSnapshot) -> Result<ImageInput, RenderLoweringError> {
    Ok(match source {
        SceneImageSourceValue::Content { layer, content, time, namespace } => {
            let source = match content {
                SceneContentValue::None | SceneContentValue::Material(_) | SceneContentValue::Particles(_) => return Ok(ImageInput::Absent),
                SceneContentValue::Text(text) => RasterSource::Vector { shapes: Arc::new(text.shapes()), vector: false, remember: false, field_step: false },
                SceneContentValue::Shape(shapes) => RasterSource::Vector { shapes: Arc::new(shapes.clone()), vector: false, remember: false, field_step: false },
                SceneContentValue::Media { source, time } => RasterSource::Image { path: source.path.clone(), time: *time },
                SceneContentValue::Plate(plate) => return Ok(ImageInput::Graph {
                    graph: Arc::new(lower_scene(&plate_scene(plate), catalog)?),
                    background: crate::render::compositor::NO_BACKGROUND,
                    absent_when_empty: true,
                    time: *time,
                    namespace: *namespace,
                }),
            };
            ImageInput::Raster { id: *layer, source, time: *time, namespace: *namespace }
        }
        SceneImageSourceValue::Refused { layer } => ImageInput::Refused { layer: *layer },
        SceneImageSourceValue::Scene { scene, background, time, namespace } => ImageInput::Graph {
            graph: Arc::new(lower_scene(scene, catalog)?),
            background: *background,
            absent_when_empty: false,
            time: *time,
            namespace: *namespace,
        },
    })
}

fn lower_layer(layer: &SceneLayerValue, clip_base: bool, picture: bool, catalog: &CatalogSnapshot) -> Result<LayerWork, RenderLoweringError> {
    let solid = translate::translate_solid(&layer.effects)
        .map(|solid| if solid.depth > 0.0 { solid } else { Solid { depth: layer.depth, ..solid } })
        .unwrap_or(Solid { depth: layer.depth, bevel: None });
    let flat = solid.extent() <= 0.0;
    let extrude = (!flat && layer.projection != LayerProjection::TwoD && layer.masks.is_empty()).then(|| Extrusion {
        solid,
        outline: match &layer.content {
            SceneContentValue::Text(text) => Some(Arc::new(text.shapes())),
            SceneContentValue::Shape(shapes) => Some(stretched(layer, shapes)),
            _ => None,
        },
        stretch: layer.shape_stretch,
    });
    let has_stage = |stage: crate::render::compositor::EffectStage| catalog.descriptors.iter()
        .any(|d| d.stage == stage && layer.effects.iter().any(|e| e.plugin_id == d.plugin_id));
    let passes = translate::translate_effect_passes(&layer.effects);
    let needs_warp = has_stage(crate::render::compositor::EffectStage::Warp);
    let needs_field = has_stage(crate::render::compositor::EffectStage::Field) && !needs_warp;
    // A pass that reads neighbouring pixels, or any pass after a field, needs a
    // picture in material space; a pointwise pass alone can shade the outline.
    let needs_image = passes.iter().any(|pass| pass.padding() > 0) || (needs_field && !passes.is_empty()) || picture;
    let vector = flat && layer.masks.is_empty() && !needs_warp && !needs_image && !clip_base;
    let blend = if layer.blend.is_stencil() {
        crate::render::compositor::BlendMode::Normal
    } else {
        translate::translate_blend_mode(layer.blend).map_err(|error| RenderLoweringError::Unsupported(error.to_string()))?
    };
    Ok(LayerWork {
        id: layer.layer,
        instance: layer.instance,
        content_key: LayerId(layer.content_key.map_or(0, |key| key.as_u64())),
        content: raster(layer, vector, vector && needs_field, catalog)?,
        host_picture: layer.effects.iter().any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id)),
        freeze: (layer.freeze_eligible && !layer.ghost && layer.instance == 0).then_some(layer.timing_start),
        extrude,
        placement: LayerPlacement {
            transform: layer.transform.affine,
            world_transform: Some(layer.transform.spatial),
            order: i32::from(layer.order),
            opacity: layer.opacity,
            z: layer.transform.spatial.translation.z,
            rotation_x: 0.0,
            rotation_y: 0.0,
            plane: None,
        },
        projection: layer.projection,
        blend,
        surface: SurfaceRecipe::from_effects(&layer.effects, catalog, false),
        displace: translate::translate_point_displace(&layer.effects),
        clip: translate::translate_clip(&layer.effects),
        shadow: translate::translate_cast_shadow(&layer.effects),
        passes,
        after_passes: translate::translate_plate_passes(&layer.after_effects),
        image_inputs: layer.image_sources.iter()
            .map(|row| row.iter().map(|source| image_input(source, catalog)).collect())
            .collect::<Result<_, _>>()?,
        masks: layer.masks.clone(),
        material: crate::render::engine::material::material_recipe(&layer.effects, catalog).map_err(RenderLoweringError::Unsupported)?,
        source_is_file: matches!(layer.source, LayerSource::File { .. }),
        source_tick: match &layer.content {
            SceneContentValue::Media { time, .. } => (time.as_seconds_f64() * 1_000_000.0).round() as i64,
            _ => 0,
        },
        isolate: layer.flatten,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{BlendMode, Matte, MatteMode};
    use crate::frame_graph::TransformValue;
    use crate::render_backend::{CountingBackend, RenderBackend};

    fn layer(id: u64) -> SceneLayerValue {
        SceneLayerValue {
            layer: LayerId(id), instance: 0, source: LayerSource::Null,
            transform: TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY },
            content_key: None, content: SceneContentValue::Shape(Vec::new()), effects: Vec::new(), after_effects: Vec::new(),
            image_sources: Vec::new(), masks: Vec::new(), matte: None, clip_to_below: false,
            flatten: false, environment: false, ghost: false, freeze_eligible: false,
            timing_start: 0, opacity: 1.0, projection: LayerProjection::TwoD,
            blend: BlendMode::Normal, order: 0, shape_stretch: [1.0, 1.0], depth: 0.0,
        }
    }

    #[test]
    fn composition_rules_arrive_as_generic_masks_and_atop_lists() {
        let catalog = crate::render::compositor::catalog_snapshot();
        let clip = SceneLayerValue { clip_to_below: true, matte: Some(Matte { layer: LayerId(1), mode: MatteMode::Alpha }), ..layer(2) };
        let target = SceneLayerValue { matte: Some(Matte { layer: LayerId(4), mode: MatteMode::InvertedLuma }), ..layer(3) };
        let stencil = SceneLayerValue { blend: BlendMode::StencilAlpha, ..layer(5) };
        let scene = SceneValue { layers: vec![layer(1), clip, target, layer(4), stencil] };
        let graph = lower_scene(&scene, &catalog).unwrap();
        let plain = |base| Composed { base, atop: Vec::new(), mask: None };
        assert_eq!(graph.output, vec![
            Composed { base: 0, atop: vec![1], mask: None },
            Composed { mask: Some((vec![plain(3)], crate::render::compositor::MatteMode::InvertedLuma)), ..plain(2) },
        ]);
        assert!(matches!(graph.layers[0].content, RasterSource::Vector { vector: false, .. }), "a clipping base rasterizes as a picture");
        assert_eq!(graph.layers[4].blend, crate::render::compositor::BlendMode::Normal);
        assert_eq!(CountingBackend.execute(&graph).unwrap(), (5, 2));
    }

    #[test]
    fn an_empty_scene_lowers_to_an_empty_graph() {
        assert!(lower_scene(&SceneValue::default(), &crate::render::compositor::catalog_snapshot()).unwrap().is_empty());
    }
}
