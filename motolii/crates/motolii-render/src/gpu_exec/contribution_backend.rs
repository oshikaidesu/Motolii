use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::frame_graph::{SceneContentValue, SceneLayerValue};
use crate::picture::resolved::ResolvedEffect;
use crate::render::compositor::{BlendMode as CompositeBlendMode, Layer, LayerWithPasses};

use super::{ResidentBlend, ResidentContent, ResidentEffectChain, ResidentPlacement, ResidentProjection};

/// Concrete per-contribution descriptor. This replaces the remaining
/// whole-scene LayerWithPasses assembly responsibility: inputs are already
/// resident/versioned resources.
impl crate::render::engine::Engine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn gpu_contribution_layer(
        &mut self,
        source: &SceneLayerValue,
        resident: ResidentContent,
        placement: ResidentPlacement,
        blend: ResidentBlend,
        projection: ResidentProjection,
        is_file_source: bool,
        direct_effects: &[ResolvedEffect],
        // Stage selection-highlight id (1..=255, 0 = none). Editor
        // interaction state, independent of document revision/comp time —
        // a plain passthrough from the caller, never scene/document data.
        outline: u8,
        effects: ResidentEffectChain,
        pass_sources: Vec<Vec<crate::render::compositor::GpuTexture2D>>,
        _comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<LayerWithPasses, crate::render::engine::EngineError> {
        let blend_mode = if blend.blend.is_stencil() {
            CompositeBlendMode::Normal
        } else {
            crate::render::engine::translate::translate_blend_mode(blend.blend)?
        };
        let frame = None;
        let layer = Layer {
            content: resident.content,
            size: resident.natural,
            placement: placement.placement,
            projection: projection.projection,
            projection_camera: camera,
            blend_mode,
            shading: self.compositor.surface_shading_for(direct_effects, false)
                .map_err(crate::render::engine::EngineError::Store)?,
            displace: crate::render::engine::translate::translate_point_displace(direct_effects),
            clip: crate::render::engine::translate::translate_clip(direct_effects),
            shadow: crate::render::engine::translate::translate_cast_shadow(direct_effects),
            outline,
            frame,
        };
        let source_tick = match &source.content {
            SceneContentValue::Media { time, .. } => {
                (time.as_seconds_f64() * 1_000_000.0).round() as i64
            }
            _ => 0,
        };
        let layer = self.apply_material_domains_semantic(
            layer,
            source.layer,
            direct_effects,
            is_file_source,
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


/// Materialize the already-lowered contribution into the resource store used by
/// relation operations. This is deliberately one contribution at a time; it
/// never scans or mutates a whole scene.
impl crate::render::engine::Engine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn gpu_materialize_contribution(
        &mut self,
        composites: &mut super::GpuResourceStore<super::ResidentCompositeLayer>,
        output: super::GpuResourceKey,
        version: super::GpuResourceVersion,
        generation: u64,
        source: &SceneLayerValue,
        resident: ResidentContent,
        placement: ResidentPlacement,
        blend: ResidentBlend,
        projection: ResidentProjection,
        is_file_source: bool,
        direct_effects: &[ResolvedEffect],
        outline: u8,
        effects: ResidentEffectChain,
        pass_sources: Vec<Vec<crate::render::compositor::GpuTexture2D>>,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<(), crate::render::engine::EngineError> {
        if composites.current(output, version).is_some() {
            composites.touch(output, generation);
            return Ok(());
        }
        let layer = self.gpu_contribution_layer(
            source,
            resident,
            placement,
            blend,
            projection,
            is_file_source,
            direct_effects,
            outline,
            effects,
            pass_sources,
            comp,
            camera,
        )?;
        composites.install(
            output,
            version,
            generation,
            super::ResidentCompositeLayer { layer },
        );
        Ok(())
    }
}
