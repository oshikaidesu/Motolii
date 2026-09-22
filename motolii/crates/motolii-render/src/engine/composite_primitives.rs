use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::render::compositor::{
    BlendMode as CompositeBlendMode, EffectPass, Layer, LayerContent, LayerWithPasses,
};
use crate::render::engine::translate::translate_matte_mode;
use crate::render::engine::{Engine, EngineError};

/// Low-level composition primitives retained after deleting the old whole-scene
/// build owner. These functions execute already-decided work; they do not
/// discover scene relationships or scheduling.
impl Engine {
    /// Resolve local effect passes into one isolated layer before a track matte
    /// consumes its coverage.
    pub(crate) fn apply_effects_before_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        if passes.is_empty() && layer.content.texture().is_some() {
            return Ok(layer);
        }
        self.bake_isolated_layer(comp, camera, layer, passes)
    }

    fn bake_isolated_layer(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        let (blend, placement) = (layer.blend_mode, layer.placement);
        self.bake_isolated_layers(
            comp,
            camera,
            vec![LayerWithPasses {
                layer,
                passes: passes.to_vec(),
                pass_sources: Vec::new(),
                padding: 0,
                cut: Vec::new(),
            }],
            blend,
            placement,
            false,
        )
    }

    /// Bake already-selected contributions into one comp-sized picture. This is
    /// an execution primitive, not a scene planner.
    pub(crate) fn bake_isolated_layers(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        mut sources: Vec<LayerWithPasses>,
        output_blend: CompositeBlendMode,
        placement: crate::doc::core::LayerPlacement,
        average: bool,
    ) -> Result<Layer, EngineError> {
        for source in &mut sources {
            source.layer.blend_mode =
                if average { CompositeBlendMode::Add } else { CompositeBlendMode::Normal };
        }
        let (texture, _view) = self.compositor.render_to_texture(
            comp,
            camera,
            &sources,
            crate::render::compositor::NO_BACKGROUND,
        )?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Layer {
            content: LayerContent::Texture(imported),
            size: [comp.width as f32, comp.height as f32],
            placement: crate::doc::core::LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                world_transform: None,
                opacity: 1.0,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                ..placement
            },
            projection: crate::doc::store::LayerProjection::TwoD,
            projection_camera: camera,
            blend_mode: output_blend,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            shadow: sources.iter().map(|source| source.layer.shadow).fold(0.0, f32::max),
            outline: sources.iter().map(|source| source.layer.outline).max().unwrap_or(0),
            frame: None,
        })
    }

    /// Apply a resolved track-matte mode to two already-prepared layers.
    pub(crate) fn apply_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        target: &Layer,
        matte_source: &Layer,
        mode: crate::doc::store::MatteMode,
    ) -> Result<Layer, EngineError> {
        Ok(self.compositor.matte_layer(
            comp,
            camera,
            target,
            matte_source,
            translate_matte_mode(mode),
        )?)
    }
}
