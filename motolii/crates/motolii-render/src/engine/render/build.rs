//! 1 コマ分の層を建てて焼く: 積み方・効果・マスク・Matte・板への焼き込み。

use super::*;

impl Engine {

    /// 平面へ収める。3D の素材を comp の絵へ一度焼き、以後は板として扱う
    /// (裁定 2026-08-30「平面に収めるのは選択肢」)。焼いた層にも blend・matte・
    /// エフェクトは今まで通り効く。
    pub(in crate::engine) fn flatten_if_asked(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        flatten: bool,
    ) -> Result<Layer, EngineError> {
        if !flatten || layer.content.texture().is_some() {
            return Ok(layer);
        }
        let mut baked_placement = layer.placement;
        baked_placement.opacity = 1.0;
        let source = LayerWithPasses {
            pass_sources: Vec::new(),
            padding: 0,
            cut: Vec::new(),
            layer: Layer {
                placement: baked_placement,
                ..layer.clone()
            },
            passes: Vec::new(),
        };
        let (texture, _view) = self.compositor.render_to_texture(
            comp,
            camera,
            std::slice::from_ref(&source),
            crate::render::compositor::NO_BACKGROUND,
        )?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Layer {
            content: crate::render::compositor::LayerContent::Texture(imported),
            size: [comp.width as f32, comp.height as f32],
            // 焼いた絵は既に comp の座標に居るので、もう一度動かさない。
            placement: crate::doc::core::LayerPlacement {
                transform: glam::Affine2::IDENTITY,
                world_transform: None,
                z: 0.0,
                rotation_x: 0.0,
                rotation_y: 0.0,
                ..layer.placement
            },
            projection: crate::doc::store::LayerProjection::TwoD,
            projection_camera: camera,
            blend_mode: layer.blend_mode,
            shading: Default::default(),
            displace: Default::default(),
            clip: None,
            shadow: layer.shadow,
            outline: layer.outline,
            frame: None,
        })
    }

    /// Track Matte は、Effect後の層をsourceのcoverageで切り、その結果を他層へblendする。
    /// EffectをMatte後のcomp大textureへ掛けると、0-input Effectが透明域を再び塗るため、
    /// 既存のlocal-texture Effect経路をここで一度だけcomp座標へ収めてからMatteへ渡す。
    pub(in crate::engine) fn apply_effects_before_matte(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layer: Layer,
        passes: &[EffectPass],
    ) -> Result<Layer, EngineError> {
        // 形・文字の輪郭のままの層(絵でない)は、切る前に 1 枚の絵に焼く(matte は絵同士で掛ける)。
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
        self.bake_isolated_layers(comp, camera, vec![LayerWithPasses { layer, passes: passes.to_vec(), pass_sources: Vec::new(), padding: 0, cut: Vec::new() }], blend, placement, false, 1.0)
    }

    /// 層(または 1 つの層の配置たち)を comp 大の 1 枚へ焼く。`average` なら写しを足す
    /// (Motion Blur: 各写しの不透明度は 1/枚数なので、足すと平均になる)。
    pub(in crate::engine) fn bake_isolated_layers(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        mut sources: Vec<LayerWithPasses>,
        output_blend: CompositeBlendMode,
        placement: crate::doc::core::LayerPlacement,
        average: bool,
        density: f32,
    ) -> Result<Layer, EngineError> {
        // BlendはMatteでcoverageを得た後、作品の下層との間に一度だけ掛ける。
        for source in &mut sources {
            source.layer.blend_mode = if average { CompositeBlendMode::Add } else { CompositeBlendMode::Normal };
        }
        // Lit by the frame's light (a frame prepared again because something read a plate early).
        // Needed before the frame is lit: this pass is a draft, the frame is prepared again lit.
        if self.known_frame_light.is_none() && self.deferring_plates {
            self.plates_needed_early = true;
        }
        let light = Some(self.known_frame_light.clone().unwrap_or_default());
        self.plate_bakes += 1;
        self.preparation_events.push(crate::render::engine::frame_graph_scene::PreparationEvent::Bake(light.as_ref().map_or(0, |light| light.serial)));
        let (texture, _view) = self.compositor.bake_picture(comp, camera, &sources, crate::render::compositor::NO_BACKGROUND, density, light.as_ref(), None)?;
        let imported = self.compositor.import_premultiplied(&texture)?;
        Ok(Self::plate_layer(imported, comp, camera, &sources, output_blend, placement))
    }

    /// A plate whose members are prepared now and whose picture is baked at the end of the frame's
    /// preparation, after the frame's one reflection capture has seen its members (2026-09-23): the
    /// members are the reflectable scene; the plate is their picture.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::engine) fn defer_isolated_layers(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        mut sources: Vec<LayerWithPasses>,
        placement: crate::doc::core::LayerPlacement,
        average: bool,
        density: f32,
    ) -> Result<Layer, EngineError> {
        for source in &mut sources {
            source.layer.blend_mode = if average { CompositeBlendMode::Add } else { CompositeBlendMode::Normal };
        }
        let size = |side: u32| if density >= 1.0 { side } else { ((side as f32 * density).ceil() as u32).max(1) };
        let target = self.compositor.create_blend_scratch_texture(size(comp.width), size(comp.height));
        let imported = self.compositor.import_premultiplied(&target)?;
        let layer = Self::plate_layer(imported.clone(), comp, camera, &sources, CompositeBlendMode::Normal, placement);
        self.reflectables.extend(sources.iter().cloned());
        self.pending_plates.push(PendingPlate { target, picture: imported, comp, camera, sources, density });
        Ok(layer)
    }

    /// The plates waiting for their picture, baked now: with the frame's world light when given,
    /// else each with its own capture (a plate something needs before the frame is lit).
    pub(in crate::engine) fn bake_pending_plates(&mut self, light: &crate::render::compositor::WorldLight) -> Result<(), EngineError> {
        for plate in std::mem::take(&mut self.pending_plates) {
            self.plate_bakes += 1;
            self.preparation_events.push(crate::render::engine::frame_graph_scene::PreparationEvent::Bake(light.serial));
            self.compositor.bake_picture(plate.comp, plate.camera, &plate.sources, crate::render::compositor::NO_BACKGROUND, plate.density, Some(light), Some(&plate.target))?;
        }
        Ok(())
    }

    fn plate_layer(
        imported: crate::render::compositor::GpuTexture2D,
        comp: CompSpec,
        camera: ResolvedCamera,
        sources: &[LayerWithPasses],
        output_blend: CompositeBlendMode,
        placement: crate::doc::core::LayerPlacement,
    ) -> Layer {
        Layer {
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
            shadow: sources.iter().map(|s| s.layer.shadow).fold(0.0, f32::max),
            outline: sources.iter().map(|s| s.layer.outline).max().unwrap_or(0),
            frame: None,
        }
    }

    pub(in crate::engine) fn apply_masks_to_layer(
        &mut self,
        mut layer: Layer,
        masks: &[ResolvedMask],
        natural: [f32; 2],
        frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
    ) -> Result<Layer, EngineError> {
        if masks.is_empty() {
            return Ok(layer);
        }
        let Some(texture) = layer.content.texture().cloned() else {
            self.layer_failures.push(
                "3D layer masks require the explicit flatten property before 2D coverage"
                    .to_owned(),
            );
            return Ok(layer);
        };
        let [width, height] = texture.width_height();
        let canvas = crate::picture::shapes_ops::Canvas {
            width,
            height,
            origin_x: 0,
            origin_y: 0,
        };
        let frame = frame.unwrap_or(crate::render::compositor::effects::vism::ImageFrame { size: natural, origin: [0.0;2], pixels: [width,height] });
        let sx = width as f64 / frame.size[0].max(1.0) as f64;
        let sy = height as f64 / frame.size[1].max(1.0) as f64;
        let masks: Vec<_> = masks.iter().cloned().map(|mut mask| {
            for vertex in &mut mask.shape.vertices {
                vertex.point[0] = (vertex.point[0] - frame.origin[0] as f64) * sx;
                vertex.point[1] = (vertex.point[1] - frame.origin[1] as f64) * sy;
                for point in [&mut vertex.in_tangent, &mut vertex.out_tangent] { point[0] *= sx; point[1] *= sy; }
            }
            mask.expansion *= (sx + sy) * 0.5;
            mask
        }).collect();
        let coverage = crate::render::engine::mask::fold_masks(&masks, &canvas)?;
        let rgba = coverage
            .bytes
            .into_iter()
            .flat_map(|alpha| [alpha, alpha, alpha, alpha])
            .collect::<Vec<_>>();

        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        "layer-mask-coverage".hash(&mut hasher);
        width.hash(&mut hasher);
        height.hash(&mut hasher);
        rgba.hash(&mut hasher);
        let key = hasher.finish();
        let mask_texture = self
            .compositor
            .cached_rgba(key, "layer-mask-coverage", || {
                Ok::<_, std::convert::Infallible>((rgba, width, height))
            })?;
        let masked = self
            .compositor
            .apply_local_alpha_mask(&texture, &mask_texture)?;
        layer.content = LayerContent::Texture(masked);
        Ok(layer)
    }

}

/// A plate whose picture is baked after the frame's reflection capture.
pub(in crate::engine) struct PendingPlate {
    target: wgpu::Texture,
    /// The plate's picture as layers refer to it (its identity until it is baked).
    picture: crate::render::compositor::GpuTexture2D,
    comp: CompSpec,
    camera: ResolvedCamera,
    sources: Vec<LayerWithPasses>,
    density: f32,
}

impl PendingPlate {
    pub(in crate::engine) fn is_picture_of(&self, content: &crate::render::compositor::LayerContent) -> bool {
        content.texture().is_some_and(|texture| texture.handle() == self.picture.handle())
    }
}
