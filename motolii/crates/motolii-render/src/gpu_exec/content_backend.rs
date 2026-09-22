use crate::doc::core::{CompSpec, RationalTime};
use crate::doc::store::LayerId;
use crate::frame_graph::{SceneContentValue, SceneLayerValue};

use super::types::{GpuResourceKey, GpuResourceVersion};
use super::resource_store::GpuResourceStore;

#[derive(Clone)]
pub(crate) struct ResidentContent {
    pub content: crate::render::compositor::LayerContent,
    pub natural: [f32; 2],
}

/// Concrete C2 backend. It deliberately reuses the existing proven content
/// builders while moving their lifetime under logical GPU resource identity.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_resident_content(
        &mut self,
        store: &mut GpuResourceStore<ResidentContent>,
        key: GpuResourceKey,
        version: GpuResourceVersion,
        generation: u64,
        layer: &SceneLayerValue,
        comp: CompSpec,
        at: RationalTime,
    ) -> Result<Option<ResidentContent>, crate::render::engine::EngineError> {
        if let Some(hit) = store.current(key, version).cloned() {
            store.touch(key, generation);
            return Ok(Some(hit));
        }

        let semantic_key = LayerId(layer.content_key.map_or(0, |node| node.as_u64()));
        let built = match &layer.content {
            SceneContentValue::None => return Ok(None),
            SceneContentValue::Text(text) => {
                let (content, natural) = self.text_texture_from_shapes(&text.shapes(), semantic_key, comp)?;
                content.map(|content| ResidentContent { content, natural })
            }
            SceneContentValue::Shape(shapes) => {
                let stretched;
                let shapes = if layer.shape_stretch != [1.0, 1.0] {
                    stretched = crate::picture::shapes_ops::stretch_outline(shapes, layer.shape_stretch);
                    stretched.as_slice()
                } else {
                    shapes.as_slice()
                };
                let (content, natural) = self.shape_texture_from_shapes(
                    shapes,
                    semantic_key,
                    true,
                    0.05,
                    comp,
                    None,
                    layer.shape_stretch == [1.0, 1.0],
                )?;
                content.map(|content| ResidentContent { content, natural })
            }
            SceneContentValue::Material(material) => {
                let (content, natural) = self.mesh_content_for(&material.source.path, comp)?;
                content.map(|content| ResidentContent { content, natural })
            }
            SceneContentValue::Media { source, time } => {
                let (content, natural) = if layer.environment && crate::render::media::is_still_image_path(&source.path) {
                    self.environment_content_for(&source.path)?
                } else {
                    self.file_content_for(&source.path, *time, layer.layer, comp)?
                };
                content.map(|content| ResidentContent { content, natural })
            }
            SceneContentValue::Particles(value) => {
                let frame = crate::render::engine::ParticleFrame::from_particles(
                    &value.particles,
                    value.turbulence,
                    value.links,
                );
                let natural = [frame.bounds.max[0].max(1.0), frame.bounds.max[1].max(1.0)];
                Some(ResidentContent {
                    content: crate::render::compositor::LayerContent::Cloud {
                        positions: frame.positions,
                        colors: frame.colors,
                        bounds: frame.bounds,
                        point_size: 1.0,
                        sizes: Some(frame.sizes),
                        sprites: true,
                        links: frame.links,
                    },
                    natural,
                })
            },
            // Plate is a composition resource, not leaf content.
            SceneContentValue::Plate(_) => return Ok(None),
        };

        if let Some(value) = built.clone() {
            store.install(key, version, generation, value);
        }
        let _ = at; // reserved for temporal backend operations; content value already contains media time.
        Ok(built)
    }
}
