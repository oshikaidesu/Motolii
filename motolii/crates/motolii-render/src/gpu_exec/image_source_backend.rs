use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::frame_graph::SceneImageSourceValue;

use super::resource_store::GpuResourceStore;
use super::types::{GpuResourceKey, GpuResourceVersion};

#[derive(Clone)]
pub(crate) struct ResidentImageSource {
    pub texture: crate::render::compositor::GpuTexture2D,
}

/// Immutable snapshot backing for effect image inputs. Media decoders may reuse
/// and overwrite their player texture; only this copied resource is allowed to
/// represent an exact semantic source value/time.
impl crate::render::engine::Engine {
    pub(crate) fn gpu_resident_image_source(
        &mut self,
        store: &mut GpuResourceStore<ResidentImageSource>,
        key: GpuResourceKey,
        version: GpuResourceVersion,
        generation: u64,
        source: &SceneImageSourceValue,
        comp: CompSpec,
        camera: ResolvedCamera,
    ) -> Result<Option<ResidentImageSource>, crate::render::engine::EngineError> {
        if let Some(hit) = store.current(key, version).cloned() {
            store.touch(key, generation);
            return Ok(Some(hit));
        }

        let Some(texture) = self.frame_graph_image_source(source, comp, camera)? else {
            return Ok(None);
        };
        let value = ResidentImageSource { texture };
        store.install(key, version, generation, value.clone());
        Ok(Some(value))
    }
}
