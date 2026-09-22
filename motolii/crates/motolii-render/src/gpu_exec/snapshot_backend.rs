use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::frame_graph::{SceneContentValue, SceneImageSourceValue};
use crate::render::compositor::GpuTexture2D;

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
        self.gpu_set_source_clock(time);

        let built: Result<Option<GpuTexture2D>, crate::render::engine::EngineError> = (|| {
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
                        // Plate is already a composition resource. A snapshot
                        // backend must not recursively rebuild its member scene.
                        SceneContentValue::Plate(_) => return Ok(None),
                    };
                    let Some(texture) = content.and_then(|content| content.texture().cloned()) else {
                        return Ok(None);
                    };
                    self.compositor.snapshot_texture(&texture)
                }
                // Scene-valued image inputs require an explicit composite
                // resource producer. Recursive whole-scene execution is forbidden
                // here; absence remains a hard miss until that producer installs
                // the snapshot.
                SceneImageSourceValue::Scene { .. } => return Ok(None),
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
