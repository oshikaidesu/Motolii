use super::*;
use re_renderer::environment::SceneReflection;
use re_renderer::renderer::SurfaceProgram;
use std::collections::HashSet;
use std::sync::Arc;

const MAX_INPUTS: usize = 16_384;
const MAX_TEXTURE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(PartialEq)]
struct InputKey {
    content: (u8, u64),
    local_min: glam::Vec2,
    local_size: glam::Vec2,
    placement: LayerPlacement,
    projection: crate::doc::store::LayerProjection,
    camera: ResolvedCamera,
    opacity: f32,
    blend: BlendMode,
    params: [f32; 12],
    program: usize,
    clip: Option<ClipSpec>,
}

pub(super) struct ReflectionKey {
    comp: CompSpec,
    generation: u64,
    inputs: Vec<InputKey>,
    environment: Option<(glam::Mat3, f32)>,
    textures: Vec<GpuTexture2D>,
    programs: Vec<Arc<SurfaceProgram>>,
    bytes: u64,
}

impl PartialEq for ReflectionKey {
    fn eq(&self, other: &Self) -> bool {
        self.comp == other.comp
            && self.generation == other.generation
            && self.inputs == other.inputs
            && self.environment == other.environment
            && self.textures.len() == other.textures.len()
            && self
                .textures
                .iter()
                .zip(&other.textures)
                .all(|(a, b)| a.handle() == b.handle())
    }
}

pub(crate) struct ReflectionEntry {
    key: ReflectionKey,
    reflection: SceneReflection,
}

impl Compositor {
    pub(super) fn reflection_key(
        &self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
    ) -> Option<ReflectionKey> {
        if inputs.len() > MAX_INPUTS {
            return None;
        }
        let mut key = ReflectionKey {
            comp,
            generation: self.catalog.generation,
            inputs: Vec::with_capacity(inputs.len()),
            environment: environment
                .map(|e| (e.environment.environment_from_world, e.environment.strength)),
            textures: Vec::new(),
            programs: Vec::new(),
            bytes: 0,
        };
        let mut texture_ids = std::collections::HashMap::new();
        let mut texture = |t: &GpuTexture2D, key: &mut ReflectionKey| -> Option<u64> {
            if self.ctx.gpu_resources.textures.is_imported(t.handle()) {
                return None;
            }
            if let Some(id) = texture_ids.get(&t.handle()) {
                return Some(*id);
            }
            let d = &t.creation_desc;
            let pixel_bytes = d.format.block_copy_size(None)? as u64;
            let mut w = d.size.width as u64;
            let mut h = d.size.height as u64;
            for _ in 0..d.mip_level_count {
                key.bytes += w
                    * h
                    * pixel_bytes
                    * d.size.depth_or_array_layers as u64
                    * d.sample_count as u64;
                w = (w / 2).max(1);
                h = (h / 2).max(1);
            }
            if key.bytes > MAX_TEXTURE_BYTES {
                return None;
            }
            let id = key.textures.len() as u64;
            texture_ids.insert(t.handle(), id);
            key.textures.push(t.clone());
            Some(id)
        };
        if let Some(e) = environment {
            texture(&e.environment.radiance, &mut key)?;
            texture(&e.environment.irradiance, &mut key)?;
        }
        let mut programs = HashSet::new();
        let mut metadata_bytes = inputs.len() * std::mem::size_of::<InputKey>();
        for i in inputs {
            let content = match i.content {
                SequentialContent::Rect(t) => (0, texture(t, &mut key)?),
                SequentialContent::Model(m) => (1, m.revision),
                SequentialContent::Environment(_) => (2, 0),
                SequentialContent::Cloud { .. } => return None,
            };
            let program = i
                .shading
                .program
                .as_ref()
                .map_or(0, |p| Arc::as_ptr(p) as usize);
            if key.inputs.last().is_none_or(|last| last.program != program)
                && programs.insert(program)
            {
                if let Some(p) = &i.shading.program {
                    metadata_bytes += p.desc().field.as_ref().map_or(0, String::len)
                        + p.desc().surface.as_ref().map_or(0, String::len);
                    if metadata_bytes > 8 * 1024 * 1024 {
                        return None;
                    }
                    key.programs.push(p.clone());
                }
            }
            key.inputs.push(InputKey {
                content,
                local_min: i.local_min,
                local_size: i.local_size,
                placement: i.placement,
                projection: i.projection,
                camera: i.projection_camera,
                opacity: i.opacity,
                blend: i.blend_mode,
                params: i.shading.params,
                program,
                clip: i.clip,
            });
        }
        Some(key)
    }

    pub(super) fn cached_scene_reflection(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
    ) -> Result<Option<SceneReflection>, CompositorError> {
        if !inputs.iter().any(|i| {
            i.shading
                .program
                .as_ref()
                .is_some_and(|p| p.desc().surface.is_some())
        }) {
            if self.reflection_entry.take().is_some() {
                self.surface_work.cache_evictions += 1;
            }
            self.surface_work.cache_retained_texture_bytes = 0;
            return Ok(None);
        }
        if !self.reflection_cache_enabled {
            self.reflection_entry = None;
            self.surface_work.cache_retained_texture_bytes = 0;
            return self.capture_scene_reflection(comp, inputs, environment);
        }
        let start = std::time::Instant::now();
        let key = self.reflection_key(comp, inputs, environment);
        let hit = key
            .as_ref()
            .zip(self.reflection_entry.as_ref())
            .is_some_and(|(k, e)| *k == e.key);
        self.surface_work.cache_key_us += start.elapsed().as_micros() as u64;
        if hit {
            self.surface_work.cache_hits += 1;
            return Ok(self.reflection_entry.as_ref().map(|e| e.reflection.clone()));
        }
        self.surface_work.cache_misses += 1;
        if key.is_none() {
            self.surface_work.cache_bypasses += 1;
        }
        if self.reflection_entry.take().is_some() {
            self.surface_work.cache_evictions += 1;
        }
        self.surface_work.cache_retained_texture_bytes = 0;
        let result = self.capture_scene_reflection(comp, inputs, environment)?;
        if let (Some(key), Some(reflection)) = (key, result.clone()) {
            self.surface_work.cache_retained_texture_bytes = key.bytes;
            self.reflection_entry = Some(ReflectionEntry { key, reflection });
        }
        Ok(result)
    }
}
