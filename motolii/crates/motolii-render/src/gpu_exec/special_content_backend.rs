use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneLayerValue};

use super::ResidentContent;

/// Resolve special content cases that sit above ordinary resident leaf content:
/// overlays/freeze, force-picture rasterization, plate composition and extrusion.
/// This is per-resource work, not whole-scene planning.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_frozen_content(
        &mut self,
        source: &SceneLayerValue,
    ) -> Option<(
        crate::render::compositor::LayerContent,
        [f32; 2],
        u32,
        Option<crate::render::compositor::effects::vism::ImageFrame>,
    )> {
        if !source.freeze_eligible || source.ghost || source.instance != 0 || self.freezing == Some(source.layer) {
            return None;
        }
        let comp_frame = self.compositor.clock?.get(2).copied()?.round() as i64;
        let layer_frame = comp_frame - source.timing_start;
        let Self { frozen, compositor, .. } = self;
        let picture = frozen.load(
            source.layer,
            layer_frame,
            &mut |bytes, width, height| compositor.upload_rgba16f("motolii-frozen", bytes, width, height).ok(),
        )?;
        Some((
            crate::render::compositor::LayerContent::LinearTexture(picture.texture),
            picture.natural,
            picture.padding,
            picture.frame,
        ))
    }

    pub(crate) fn gpu_extruded_content(
        &mut self,
        source: &SceneLayerValue,
        texture: crate::render::compositor::GpuTexture2D,
        natural: [f32; 2],
        solid: crate::render::compositor::extrude::Solid,
    ) -> Result<crate::render::compositor::LayerContent, crate::render::engine::EngineError> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        texture.handle.hash(&mut hasher);
        solid.hash_key(&mut hasher);
        source.shape_stretch[0].to_bits().hash(&mut hasher);
        source.shape_stretch[1].to_bits().hash(&mut hasher);
        let key = hasher.finish();
        if let Some((cached, model)) = self.extrusions.get(&source.layer) {
            if *cached == key { return Ok(crate::render::compositor::LayerContent::Model(model.clone())); }
        }
        let rectangle = |size: [f32; 2]| {
            let vertex = |x: f32, y: f32| re_renderer::renderer::PathVertex {
                point: glam::vec2(x, y), in_tangent: glam::Vec2::ZERO, out_tangent: glam::Vec2::ZERO,
            };
            vec![(vec![re_renderer::renderer::PathContour {
                closed: true,
                vertices: vec![vertex(0.0,0.0), vertex(size[0],0.0), vertex(size[0],size[1]), vertex(0.0,size[1])],
            }], re_renderer::renderer::PathFillRule::NonZero)]
        };
        let outlines = match &source.content {
            SceneContentValue::Text(text) => {
                let shapes = text.shapes();
                match crate::picture::shapes_ops::content_canvas(&shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(&shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            SceneContentValue::Shape(shapes) => {
                let stretched;
                let shapes = if source.shape_stretch != [1.0,1.0] {
                    stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                    stretched.as_slice()
                } else { shapes.as_slice() };
                match crate::picture::shapes_ops::content_canvas(shapes)? {
                    Some(canvas) => crate::render::compositor::paths::outlines(shapes, &canvas)?,
                    None => Vec::new(),
                }
            }
            _ => rectangle(natural),
        };
        let Some(model) = self.compositor.extrude_model(&outlines, texture.clone(), natural, solid)? else {
            return Ok(crate::render::compositor::LayerContent::Texture(texture));
        };
        let model = std::sync::Arc::new(model);
        self.extrusions.insert(source.layer, (key, model.clone()));
        Ok(crate::render::compositor::LayerContent::Model(model))
    }

    pub(crate) fn gpu_special_content(
        &mut self,
        source: &SceneLayerValue,
        resident: Option<ResidentContent>,
        force_picture: bool,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<(ResidentContent, u32, Option<crate::render::compositor::effects::vism::ImageFrame>, bool)>, crate::render::engine::EngineError> {
        if let Some((content, natural, padding, frame)) = self.gpu_frozen_content(source) {
            return Ok(Some((ResidentContent { content, natural }, padding, frame, true)));
        }
        if source.effects.iter().any(|effect| crate::extensions::overlay::is_track_overlay(&effect.plugin_id)) {
            if let Some((content, natural)) = self.overlay_content(source.layer, comp)? {
                return Ok(Some((ResidentContent { content, natural }, 0, None, false)));
            }
        }

        let key = LayerId(source.content_key.map_or(0, |node| node.as_u64()));
        let mut value = match (&source.content, force_picture) {
            (SceneContentValue::None, _) => return Ok(None),
            (SceneContentValue::Text(text), true) => {
                let (content, natural) = self.shape_texture_from_shapes(
                    &text.shapes(), key, false, 0.05, comp, None, true,
                )?;
                content.map(|content| ResidentContent { content, natural })
            }
            (SceneContentValue::Shape(shapes), true) => {
                let stretched;
                let shapes = if source.shape_stretch != [1.0, 1.0] {
                    stretched = crate::picture::shapes_ops::stretch_outline(shapes, source.shape_stretch);
                    stretched.as_slice()
                } else { shapes.as_slice() };
                let (content, natural) = self.shape_texture_from_shapes(
                    shapes, key, false, 0.05, comp, None,
                    source.shape_stretch == [1.0, 1.0],
                )?;
                content.map(|content| ResidentContent { content, natural })
            }
            (SceneContentValue::Plate(plate), _) => self.frame_graph_plate(source, plate, comp, camera)?
                .map(|(content, natural)| ResidentContent { content, natural }),
            _ => resident,
        };
        let Some(mut value) = value.take() else { return Ok(None); };

        let solid = crate::render::engine::translate::translate_solid(&source.effects)
            .map(|solid| if solid.depth > 0.0 { solid } else { crate::render::compositor::extrude::Solid { depth: source.depth, ..solid } })
            .unwrap_or(crate::render::compositor::extrude::Solid { depth: source.depth, bevel: None });
        if solid.extent() > 0.0 && source.projection != crate::doc::store::LayerProjection::TwoD && source.masks.is_empty() {
            if let crate::render::compositor::LayerContent::Texture(texture) = &value.content {
                value.content = self.gpu_extruded_content(
                    source, texture.clone(), value.natural, comp, solid,
                )?;
            }
        }
        if !source.image_sources.is_empty() {
            value.content = match &value.content {
                crate::render::compositor::LayerContent::Texture(texture) => self.compositor.snapshot_texture(texture)
                    .map(crate::render::compositor::LayerContent::Texture)
                    .unwrap_or_else(|| value.content.clone()),
                crate::render::compositor::LayerContent::LinearTexture(texture) => self.compositor.snapshot_texture(texture)
                    .map(crate::render::compositor::LayerContent::LinearTexture)
                    .unwrap_or_else(|| value.content.clone()),
                _ => value.content,
            };
        }
        Ok(Some((value, 0, None, false)))
    }
}
