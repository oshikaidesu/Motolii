use crate::doc::core::CompSpec;
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue, SceneValue};
use crate::render::compositor::GpuTexture2D;
use crate::render::engine::ResolvedCamera;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentSnapshot {
    pub texture: GpuTexture2D,
}

impl crate::render::engine::Engine {
    pub(crate) fn gpu_snapshot_source(
        &mut self,
        store: &mut GpuResourceStore<ResidentSnapshot>,
        key: GpuResourceKey,
        version: GpuResourceVersion,
        generation: u64,
        source: &SceneImageSourceValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<GpuTexture2D>, crate::render::engine::EngineError> {
        if let Some(hit) = store.current(key, version).cloned() {
            store.touch(key, generation);
            return Ok(Some(hit.texture));
        }

        let previous_clock = self.compositor.clock;
        let previous_namespace = self.feedback_namespace;
        let (time, namespace) = match source {
            SceneImageSourceValue::Content { time, namespace, .. }
            | SceneImageSourceValue::Scene { time, namespace, .. } => (*time, *namespace),
        };
        self.feedback_namespace = namespace;
        self.set_frame_graph_source_clock(time);

        let built = (|| {
            let texture = match source {
                SceneImageSourceValue::Content { layer, content, .. } => {
                    let content = match content {
                        SceneContentValue::None => return Ok(None),
                        SceneContentValue::Text(text) => self.shape_texture_from_shapes(
                            &text.shapes(), *layer, false, 0.05, comp, None, false,
                        )?.0,
                        SceneContentValue::Shape(shapes) => self.shape_texture_from_shapes(
                            shapes, *layer, false, 0.05, comp, None, false,
                        )?.0,
                        SceneContentValue::Material(_) | SceneContentValue::Particles(_) => return Ok(None),
                        SceneContentValue::Media { source, time } => self.file_content_for(
                            &source.path, *time, *layer, comp,
                        )?.0,
                        SceneContentValue::Plate(plate) => {
                            let nested = SceneValue {
                                layers: plate.members.iter().filter_map(|member| member.layer.clone()).collect(),
                            };
                            let prepared = self.prepare_gpu_scene(&nested, comp, camera)?;
                            if prepared.layers.is_empty() { return Ok(None); }
                            let (texture, _) = self.compositor.render_to_texture(
                                comp,
                                camera,
                                &prepared.layers,
                                crate::render::compositor::NO_BACKGROUND,
                            )?;
                            return Ok(self.compositor.import_premultiplied(&texture).ok());
                        }
                    };
                    let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                        return Ok(None);
                    };
                    self.compositor.snapshot_texture(&texture)
                }
                SceneImageSourceValue::Scene { scene, background, .. } => {
                    let prepared = self.prepare_gpu_scene(scene, comp, camera)?;
                    let (texture, _) = self.compositor.render_to_texture(
                        comp, camera, &prepared.layers, *background,
                    )?;
                    self.compositor.import_premultiplied(&texture).ok()
                }
            };
            Ok(texture)
        })();

        self.compositor.clock = previous_clock;
        self.feedback_namespace = previous_namespace;

        let texture = built?;
        if let Some(texture) = texture.clone() {
            store.install(key, version, generation, ResidentSnapshot { texture });
        }
        Ok(texture)
    }
}
