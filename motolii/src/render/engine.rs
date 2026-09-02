use std::collections::HashMap;

pub mod mask;
pub mod shape;
pub mod text;

mod render;
mod texture;
mod translate;

use crate::doc::core::ResolvedCamera;
use crate::render::compositor::GpuTexture2D;
use crate::render::compositor::{Compositor, CompositorError};

use crate::doc::store::{Matte, RationalTime, StoreView};
use crate::render::media::ContainerInfo;
use crate::render::media::MediaError;
use crate::render::media::MediaInfo;
use crate::render::media::PointCloudData;

use crate::render::engine::texture::{ShapeCacheKey, TextCacheKey};

pub use crate::render::engine::translate::{
    known_effects, EffectDescriptor, EffectParamDescriptor,
};

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
    #[error(transparent)]
    Mask(#[from] crate::render::engine::mask::MaskFoldError),
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
    pub fn as_resolved_camera(&self) -> ResolvedCamera {
        ResolvedCamera {
            center: self.pan,
            zoom: self.zoom,
            roll_degrees: 0.0,
        }
    }
}

/// 焼いた画素。テクスチャは**フレーム単位**で捨てられる(`begin_frame` が
/// 触られなかった物を全部落とす)ので、画素をこちら側で長く持たないと、
/// 再生位置が層の範囲を出入りするたび読み直しになる。
pub(crate) struct StillImage {
    pub premultiplied_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// 画素の予算。**個数ではなく重さで切る** —— 4K1枚は小さい絵の千枚分ある。
const STILL_PIXEL_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct ByBytes;

impl quick_cache::Weighter<String, std::sync::Arc<StillImage>> for ByBytes {
    fn weight(&self, _key: &String, image: &std::sync::Arc<StillImage>) -> u64 {
        image.premultiplied_rgba.len() as u64
    }
}

pub(crate) type StillPixels = quick_cache::sync::Cache<String, std::sync::Arc<StillImage>, ByBytes>;

fn still_pixels() -> StillPixels {
    StillPixels::with_weighter(64, STILL_PIXEL_BUDGET_BYTES, ByBytes)
}

pub struct Engine {
    compositor: Compositor,
    probes: HashMap<String, MediaInfo>,
    text_textures: HashMap<TextCacheKey, GpuTexture2D>,
    shape_textures: HashMap<ShapeCacheKey, GpuTexture2D>,
    failed_probes: HashMap<String, String>,
    layer_failures: Vec<String>,
    models: HashMap<String, std::sync::Arc<crate::render::compositor::GpuModelData>>,
    failed_meshes: HashMap<String, String>,
    containers: HashMap<String, ContainerInfo>,
    failed_containers: HashMap<String, String>,
    point_clouds: HashMap<String, PointCloudData>,
    failed_point_clouds: HashMap<String, String>,
    pixels: StillPixels,
    videos: HashMap<String, (Vec<u8>, re_renderer::video::Video)>,
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            models: HashMap::new(),
            failed_meshes: HashMap::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
        })
    }

    pub fn gpu_device(&self) -> &wgpu::Device {
        self.compositor.device()
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::with_device_using_headless_defaults(device, queue)?,
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            models: HashMap::new(),
            failed_meshes: HashMap::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
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
                self.failed_containers
                    .insert(path.to_string(), err.to_string());
                None
            }
        }
    }
}

#[cfg(test)]
mod spatial_cache_tests {
    use super::*;
    use crate::doc::store::{
        property, Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource,
        LayerTiming, PropertyId, Value,
    };

    fn document(path: &std::path::Path) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::File {
                        path: path.to_string_lossy().into_owned(),
                        fingerprint: None,
                    },
                    order: 0,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2([16.0, 16.0]),
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::SCALE).unwrap(),
                value: Value::Vec2([16.0, 16.0]),
            },
        ])
        .unwrap();
        doc
    }

    #[test]
    fn repeated_spatial_frames_reuse_model_uploads_and_point_arrays() {
        let dir = tempfile::tempdir().unwrap();
        let obj = dir.path().join("triangle.obj");
        let ply = dir.path().join("points.ply");
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n").unwrap();
        std::fs::write(
            &ply,
            "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nend_header\n-1 -1 0\n1 -1 0\n0 1 0\n",
        )
        .unwrap();

        let mut engine = Engine::new().unwrap();
        let obj_doc = document(&obj);
        engine
            .render_frame(&obj_doc.view(), RationalTime::ZERO)
            .unwrap();
        let model = engine.models.get(obj.to_str().unwrap()).unwrap().clone();
        engine
            .render_frame(&obj_doc.view(), RationalTime::ZERO)
            .unwrap();
        assert_eq!(engine.models.len(), 1);
        assert!(std::sync::Arc::ptr_eq(
            &model,
            engine.models.get(obj.to_str().unwrap()).unwrap()
        ));

        let ply_doc = document(&ply);
        engine
            .render_frame(&ply_doc.view(), RationalTime::ZERO)
            .unwrap();
        let positions = engine
            .point_clouds
            .get(ply.to_str().unwrap())
            .unwrap()
            .positions
            .clone();
        engine
            .render_frame(&ply_doc.view(), RationalTime::ZERO)
            .unwrap();
        assert_eq!(engine.point_clouds.len(), 1);
        assert!(std::sync::Arc::ptr_eq(
            &positions,
            &engine
                .point_clouds
                .get(ply.to_str().unwrap())
                .unwrap()
                .positions
        ));
    }
}
