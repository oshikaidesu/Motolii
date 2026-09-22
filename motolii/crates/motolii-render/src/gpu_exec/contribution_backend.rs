use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::frame_graph::{SceneContentValue, SceneLayerValue};
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};

use super::{ResidentContent, ResidentEffectChain, ResidentPlacement};

/// Concrete per-contribution descriptor. This replaces the remaining
/// whole-scene LayerWithPasses assembly responsibility: inputs are already
/// resident/versioned resources.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_contribution_layer(
        &mut self,
        source: &SceneLayerValue,
        resident: ResidentContent,
        placement: ResidentPlacement,
        effects: ResidentEffectChain,
        pass_sources: Vec<Vec<crate::render::compositor::GpuTexture2D>>,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<LayerWithPasses, crate::render::engine::EngineError> {
        let blend_mode = if source.blend.is_stencil() {
            CompositeBlendMode::Normal
        } else {
            crate::render::engine::translate::translate_blend_mode(source.blend)?
        };
        let frame = None;
        let mut layer = Layer {
            content: resident.content,
            size: resident.natural,
            placement: placement.placement,
            projection: source.projection,
            projection_camera: camera,
            blend_mode,
            shading: self.compositor.surface_shading_for(&source.effects, false)
                .map_err(crate::render::engine::EngineError::Store)?,
            displace: crate::render::engine::translate::translate_point_displace(&source.effects),
            clip: crate::render::engine::translate::translate_clip(&source.effects),
            shadow: crate::render::engine::translate::translate_cast_shadow(&source.effects),
            outline: self.outline_order.iter().position(|id| *id == source.layer).map(|index| (index + 1).min(u8::MAX as usize) as u8).unwrap_or(0),
            frame,
        };
        layer = self.frame_graph_process_mask_flatten(
            source,
            layer,
            resident.natural,
            frame,
            comp,
            camera,
        )?;
        let source_tick = match &source.content {
            SceneContentValue::Media { time, .. } => {
                (time.as_seconds_f64() * 1_000_000.0).round() as i64
            }
            _ => 0,
        };
        layer = self.apply_material_domains_semantic(
            layer,
            source.layer,
            &source.effects,
            matches!(source.source, crate::doc::store::LayerSource::File { .. }),
            source_tick,
            resident.natural,
            frame,
        )?;
        Ok(LayerWithPasses {
            layer,
            passes: effects.passes.into_iter().chain(effects.plate_passes).collect(),
            padding: 0,
            pass_sources,
            cut: Vec::new(),
        })
    }
}
