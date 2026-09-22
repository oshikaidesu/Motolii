use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneLayerValue};

use super::ResidentContent;

/// Resolve special content cases that sit above ordinary resident leaf content:
/// overlays/freeze, force-picture rasterization, plate composition and extrusion.
/// This is per-resource work, not whole-scene planning.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_special_content(
        &mut self,
        source: &SceneLayerValue,
        resident: Option<ResidentContent>,
        force_picture: bool,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<(ResidentContent, u32, Option<crate::render::compositor::effects::vism::ImageFrame>, bool)>, crate::render::engine::EngineError> {
        if let Some((content, natural, padding, frame)) = self.frame_graph_frozen_content(source) {
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
                value.content = self.frame_graph_extruded_content(
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
