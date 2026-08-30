
use std::collections::HashMap;

pub mod mask;
pub mod shape;
pub mod text;

mod render;
mod texture;
mod translate;

use crate::render::compositor::GpuTexture2D;
use crate::render::compositor::{Compositor, CompositorError};
use crate::doc::core::ResolvedCamera;

use crate::render::media::ContainerInfo;
use crate::render::media::MediaError;
use crate::render::media::MediaInfo;
use crate::render::media::PointCloudData;
use crate::doc::store::{LayerSource, Matte, RationalTime, StoreView};

use crate::render::engine::texture::{ShapeCacheKey, TextCacheKey};

pub use crate::render::engine::translate::{known_effects, EffectDescriptor, EffectParamDescriptor};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Compositor(#[from] CompositorError),
    #[error(transparent)]
    Media(#[from] MediaError),
    #[error("時刻をフレームへ写せない: {0}")]
    Time(String),
    #[error("Document を読めない: {0}")]
    Store(String),
    #[error("comp の設定が Document に無い(解像度も fps も決まっていない)")]
    NoComposition,
    #[error("blend mode {0:?} はまだ合成器が対応していない(Normal のみ対応。fork 改造候補)")]
    UnsupportedBlendMode(crate::doc::store::BlendMode),
    #[error("matte はまだ engine が絵から除外しつつ消費する経路に繋がっていない({0:?})")]
    UnsupportedMatte(Matte),
    #[error(transparent)]
    Text(#[from] crate::render::engine::text::TextRenderError),
    #[error(transparent)]
    Shape(#[from] crate::render::vector::VectorError),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObservationCamera {
    pub pan: [f32; 2],
    pub zoom: f32,
}

impl Default for ObservationCamera {
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: 1.0,
        }
    }
}

impl ObservationCamera {
    fn as_resolved_camera(&self) -> ResolvedCamera {
        ResolvedCamera {
            center: self.pan,
            zoom: self.zoom,
            roll_degrees: 0.0,
        }
    }
}

pub struct Engine {
    compositor: Compositor,
    textures: HashMap<LayerSource, GpuTexture2D>,
    probes: HashMap<String, MediaInfo>,
    text_textures: HashMap<TextCacheKey, GpuTexture2D>,
    shape_textures: HashMap<ShapeCacheKey, GpuTexture2D>,
    failed_probes: HashMap<String, String>,
    layer_failures: Vec<String>,
    containers: HashMap<String, ContainerInfo>,
    failed_containers: HashMap<String, String>,
    point_clouds: HashMap<String, PointCloudData>,
    failed_point_clouds: HashMap<String, String>,
    point_cloud_textures: HashMap<(String, u32, u32), GpuTexture2D>,
    videos: HashMap<String, (Vec<u8>, re_renderer::video::Video)>,
    video_last_texture: HashMap<u64, GpuTexture2D>,
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            point_cloud_textures: HashMap::new(),
            videos: HashMap::new(),
            video_last_texture: HashMap::new(),
        })
    }

    pub fn gpu_device(&self) -> &wgpu::Device {
        self.compositor.device()
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::with_device_using_headless_defaults(device, queue)?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            point_cloud_textures: HashMap::new(),
            videos: HashMap::new(),
            video_last_texture: HashMap::new(),
        })
    }

    pub fn render_frame(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<Vec<u8>, EngineError> {
        self.render(view, t, true)
    }

    pub fn layer_failures(&self) -> &[String] {
        &self.layer_failures
    }

    pub fn render_frame_without_background(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<Vec<u8>, EngineError> {
        self.render(view, t, false)
    }

    pub fn render_frame_with_view_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        observation: &ObservationCamera,
    ) -> Result<Vec<u8>, EngineError> {
        self.render_with_camera_override(view, t, true, Some(observation.as_resolved_camera()))
    }

    fn render(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
    ) -> Result<Vec<u8>, EngineError> {
        self.render_with_camera_override(view, t, include_background, None)
    }

    pub fn cached_frame_count(&self) -> usize {
        self.video_last_texture.len()
    }

    pub fn cached_text_texture_count(&self) -> usize {
        self.text_textures.len()
    }

    pub fn cached_shape_texture_count(&self) -> usize {
        self.shape_textures.len()
    }

    pub fn media_frames(&mut self, path: &str) -> Option<i64> {
        self.container_probe(path)?
            .video_streams
            .first()
            .and_then(|stream| stream.nb_frames)
    }

    pub fn media_duration(&mut self, path: &str) -> Option<crate::doc::core::RationalTime> {
        self.container_probe(path)?.duration
    }

    fn container_probe(&mut self, path: &str) -> Option<ContainerInfo> {
        if let Some(info) = self.containers.get(path) {
            return Some(info.clone());
        }
        if self.failed_containers.contains_key(path) {
            return None;
        }
        match crate::render::media::probe_container(path) {
            Ok(info) => {
                self.containers.insert(path.to_string(), info.clone());
                Some(info)
            }
            Err(err) => {
                self.failed_containers.insert(path.to_string(), err.to_string());
                None
            }
        }
    }
}
