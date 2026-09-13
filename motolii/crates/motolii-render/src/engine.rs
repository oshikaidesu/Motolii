use std::collections::HashMap;

pub mod mask;
pub mod text;

mod clip;
mod render;
mod texture;
mod material;
pub use texture::content_canvas;
pub use texture::{decode_still_linear_rgb, decode_still_srgb};
mod translate;

use crate::doc::core::ResolvedCamera;
use crate::render::compositor::GpuTexture2D;
use crate::render::compositor::{Compositor, CompositorError};

use crate::doc::store::{LayerId, Matte, RationalTime, StoreView};
use crate::render::media::ContainerInfo;
use crate::render::media::MediaError;
use crate::render::media::MediaInfo;
use crate::render::media::PointCloudData;

use crate::render::engine::texture::{ShapeCacheKey, TextCacheKey, TextTexture};

pub use crate::render::compositor::{Window, bind_catalog_runtime, catalog_errors, catalog_generation, catalog_reads_disk, catalog_source_roots, refresh_effect_catalog, refresh_effect_catalog_for, watch_effect_catalog, CatalogRefresh, CatalogRuntime, CatalogWatcher};
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
    Shape(#[from] crate::doc::vector::VectorError),
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
            roll_degrees: 0.0, ..Default::default() }
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
    pub(crate) compositor: Compositor,
    materials: HashMap<LayerId, material::MaterialCache>,
    probes: HashMap<String, MediaInfo>,
    text_textures: HashMap<TextCacheKey, TextTexture>,
    /// 入れた順。上限を越えたら古い物から落とす(comp 解像度の texture を無制限に貯めない)。
    text_order: std::collections::VecDeque<TextCacheKey>,
    shape_textures: HashMap<ShapeCacheKey, TextTexture>,
    failed_probes: HashMap<String, String>,
    layer_failures: Vec<String>,
    /// feedback の辿り直しの最中(入れ子で辿り直さない・素材の棚を掃除しない)。
    feedback_replaying: bool,
    /// 今描いている窓(画面の道の feedback は窓ごとに状態を持つ)。読み戻しの道では None = 出力寸法。
    feedback_window: Option<crate::render::compositor::Window>,
    /// 動画の復号の流れの名前空間(0 = 本番)。合成を別の時刻で描く間だけ別の値にする。
    video_stream_namespace: u64,
    /// feedback の鍵の名前空間(0 = 本番)。別の時刻の合成を描く間だけ時刻のずれの値。
    feedback_namespace: u64,
    /// この frame に別の時刻の合成(SOURCE)があった: 辿り直しはフレームを丸ごと(t′ の列も進める)。
    feedback_saw_composites: bool,
    /// この frame の組み立てで刻んだ feedback の鍵(板に焼く途中で消費された物も含む)。
    feedback_keys_seen: Vec<crate::render::compositor::FeedbackKey>,
    /// Stage で選ばれている層。`render_frame_into_with_camera` の間だけ入る(export の描画には載らない)。
    outline_layers: Vec<LayerId>,
    /// 直前の Stage 描画で番号を振った順。mask の id を層へ戻す。
    outline_order: Vec<LayerId>,
    /// 直前のフレームで実際に描いた層(配置の複製を含む)の数。画面外は数えない。
    drawn_layers: usize,
    models: HashMap<String, std::sync::Arc<crate::render::compositor::GpuModelData>>,
    /// 層ごとの押し出し(鍵 = 絵の handle と奥行き)。絵か奥行きが変われば作り直す。
    pub(crate) extrusions: HashMap<LayerId, (u64, std::sync::Arc<crate::render::compositor::GpuModelData>)>,
    failed_meshes: HashMap<String, String>,
    environments: HashMap<String, std::sync::Arc<crate::render::compositor::GpuEnvironmentData>>,
    containers: HashMap<String, ContainerInfo>,
    failed_containers: HashMap<String, String>,
    point_clouds: HashMap<String, PointCloudData>,
    failed_point_clouds: HashMap<String, String>,
    pixels: StillPixels,
    /// 動画は mmap で開く。触ったページだけ RAM に載り、閉じれば返る。
    videos: HashMap<String, (memmap2::Mmap, re_renderer::video::Video)>,
    renders_since_video_purge: u32,
    /// 復号した動画のコマを GPU texture のまま取っておく。2 周目は復号しない。
    frame_cache: HashMap<(String, i64), texture::CachedVideoFrame>,
    frame_cache_bytes: u64,
    frame_cache_budget: u64,
    frame_cache_tick: u64,
    frame_cache_hits: u64,
    /// 揃ったコマの texture。中身は frame の提出で GPU に届くので、写すのは次の frame の頭。
    pending_frame_copies: Vec<(String, i64, crate::render::compositor::GpuTexture2D)>,
    /// 再生中は間に合ったコマで描く。止めた時と書き出しは頼んだコマを待つ。
    realtime: bool,
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            materials: HashMap::new(),
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            text_order: std::collections::VecDeque::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            outline_layers: Vec::new(),
            outline_order: Vec::new(),
            drawn_layers: 0,
            models: HashMap::new(),
            extrusions: HashMap::new(),
            failed_meshes: HashMap::new(),
            environments: HashMap::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
            realtime: false,
            renders_since_video_purge: 0,
            feedback_replaying: false,
            feedback_window: None,
            video_stream_namespace: 0,
            feedback_namespace: 0,
            feedback_saw_composites: false,
            feedback_keys_seen: Vec::new(),
            frame_cache: HashMap::new(),
            frame_cache_bytes: 0,
            frame_cache_budget: texture::FRAME_CACHE_BUDGET,
            frame_cache_tick: 0,
            frame_cache_hits: 0,
            pending_frame_copies: Vec::new(),
        })
    }

    pub fn gpu_queue(&self) -> &wgpu::Queue {
        &self.compositor.render_context().queue
    }

    pub fn gpu_device(&self) -> &wgpu::Device {
        self.compositor.device()
    }

    /// Stage で選ばれている層の mask の番号(1..=255)。選ばれていなければ 0。
    fn outline_id(&self, id: LayerId) -> u8 {
        self.outline_layers.iter().position(|l| *l == id).map_or(0, |i| i as u8 + 1)
    }

    /// 直前の Stage 描画で、選ばれた層が実際に描かれた画面上の範囲(comp 画素、右下は外側)。
    /// GPU から届く前(初回)や何も描かれなかった層は入らない。
    pub fn take_selection_bounds(&mut self) -> Option<Vec<(LayerId, [f32; 4])>> {
        let order = self.outline_order.clone();
        self.compositor.selection_screen_bounds().map(|found| {
            found.into_iter().filter_map(|(id, b)| order.get(id as usize - 1).map(|l| (*l, b))).collect()
        })
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::with_device_using_headless_defaults(device, queue)?,
            materials: HashMap::new(),
            probes: HashMap::new(),
            text_textures: HashMap::new(),
            text_order: std::collections::VecDeque::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            outline_layers: Vec::new(),
            outline_order: Vec::new(),
            drawn_layers: 0,
            models: HashMap::new(),
            extrusions: HashMap::new(),
            failed_meshes: HashMap::new(),
            environments: HashMap::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
            realtime: false,
            renders_since_video_purge: 0,
            feedback_replaying: false,
            feedback_window: None,
            video_stream_namespace: 0,
            feedback_namespace: 0,
            feedback_saw_composites: false,
            feedback_keys_seen: Vec::new(),
            frame_cache: HashMap::new(),
            frame_cache_bytes: 0,
            frame_cache_budget: texture::FRAME_CACHE_BUDGET,
            frame_cache_tick: 0,
            frame_cache_hits: 0,
            pending_frame_copies: Vec::new(),
        })
    }

    pub fn set_realtime(&mut self, realtime: bool) {
        self.realtime = realtime;
    }

    /// 動画のコマ cache の上限(byte)。既定は [`texture::FRAME_CACHE_BUDGET`]。
    pub fn set_video_frame_cache_budget(&mut self, bytes: u64) {
        self.frame_cache_budget = bytes;
    }

    /// (当たった回数, 入っているコマ数, byte)。
    pub fn video_frame_cache_stats(&self) -> (u64, usize, u64) {
        (self.frame_cache_hits, self.frame_cache.len(), self.frame_cache_bytes)
    }

    pub fn set_gpu_instance_sharing_enabled(&mut self, enabled: bool) {
        self.compositor.gpu_instance_sharing_enabled = enabled;
    }

    pub fn set_render_measurement_enabled(&mut self, enabled: bool) {
        self.compositor.measurement_enabled = enabled;
    }

    pub fn frame_measurement(&self) -> crate::render::compositor::FrameMeasurement {
        self.compositor.measurement
    }

    /// 反射の撮影点を送り手の箱に固定し、受け手を撮影から外す(比較用の切替、Document は変えない)。
    pub fn set_reflection_scene_probe(&mut self, enabled: bool) {
        self.compositor.reflection_scene_probe = enabled;
        self.clear_reflection_cache();
    }

    /// Diagnostic switch; never changes the Document or reflection quality.
    pub fn set_reflection_cache_enabled(&mut self, enabled: bool) {
        self.compositor.reflection_cache_enabled = enabled;
        self.clear_reflection_cache();
    }

    pub fn clear_reflection_cache(&mut self) {
        if self.compositor.reflection_entry.take().is_some() {
            self.compositor.surface_work.cache_evictions += 1;
        }
        self.compositor.surface_work.cache_retained_texture_bytes = 0;
    }

    pub fn surface_work(&self) -> crate::render::compositor::SurfaceWork {
        self.compositor.surface_work
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

    pub fn drawn_layers(&self) -> usize {
        self.drawn_layers
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

#[cfg(test)]
mod environment_tests {
    use super::*;
    use crate::doc::store::{
        property, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta,
        LayerSource, LayerTiming, PropertyId, Value,
    };

    pub(super) const SIZE: u32 = 64;
    /// 網の中(位置 32,32 から scale 12 の板が右下へ広がる)。
    pub(super) const MESH_X: u32 = 44;
    pub(super) const MESH_Y: u32 = 44;

    pub(super) fn file_layer(doc: &mut Document, id: u64, order: i16, path: &std::path::Path) -> LayerId {
        let layer = LayerId(id);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::File { path: path.to_string_lossy().into_owned(), fingerprint: None },
                    order,
                    timing: LayerTiming::place(0, None, 1),
                },
            },
            Intent::SetConstant {
                layer,
                property: PropertyId::new(property::POSITION).unwrap(),
                value: Value::Vec2([SIZE as f64 / 2.0, SIZE as f64 / 2.0]),
            },
        ])
        .unwrap();
        layer
    }

    /// 上半分 `top`、下半分 `bottom` の等距円筒図。
    pub(super) fn sky_png(dir: &std::path::Path, name: &str, top: u8, bottom: u8) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut img = image::RgbaImage::new(8, 4);
        for (_, y, px) in img.enumerate_pixels_mut() {
            let v = if y < 2 { top } else { bottom };
            *px = image::Rgba([v, v, v, 255]);
        }
        img.save(&path).unwrap();
        path
    }

    pub(super) fn scene(dir: &std::path::Path, sky: &std::path::Path, environment: bool) -> Document {
        let obj = dir.join("quad.obj");
        // 法線はカメラ向き(世界の -z)。法線の無い obj は陰影が付かないので照明の test にならない。
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 0 -1\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: SIZE,
            height: SIZE,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 1,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let sky_layer = file_layer(&mut doc, 1, 0, sky);
        let mesh = file_layer(&mut doc, 2, 1, &obj);
        doc.apply(Intent::SetConstant {
            layer: mesh,
            property: PropertyId::new(property::SCALE).unwrap(),
            value: Value::Vec2([12.0, 12.0]),
        })
        .unwrap();
        doc.apply(Intent::SetAttrs {
            layer: sky_layer,
            patch: LayerAttrsPatch { environment: Some(environment), ..Default::default() },
        })
        .unwrap();
        doc
    }

    fn luma(pixels: &[u8], x: u32, y: u32) -> u8 {
        pixels[((y * SIZE + x) * 4) as usize]
    }

    fn ascii(pixels: &[u8]) -> String {
        (0..SIZE)
            .step_by(4)
            .map(|y| (0..SIZE).step_by(2).map(|x| b" .:-=+*#%@"[luma(pixels, x, y) as usize * 10 / 256] as char).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 環境層は板にならず、空として背景に敷かれ、網をその空で照らす。
    /// 上が白・下が黒の空: 画面の上は白、下は黒、正面を向いた網は照度 1/2 の灰。
    #[test]
    fn an_environment_layer_lights_the_mesh_and_fills_the_background() {
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "sky.png", 255, 0);
        let mut engine = Engine::new().unwrap();

        let doc = scene(dir.path(), &sky, true);
        let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
        let art = ascii(&pixels);
        assert!(luma(&pixels, 2, 2) >= 250, "上端は空の白\n{art}");
        assert!(luma(&pixels, 2, SIZE - 3) <= 5, "下端は空の黒\n{art}");
        let center = luma(&pixels, MESH_X, MESH_Y);
        assert!((150..=215).contains(&center), "正面の網は照度 1/2 (sRGB ≈ 188)、got {center}\n{art}");

        let plain = scene(dir.path(), &sky, false);
        let pixels = engine.render_frame(&plain.view(), RationalTime::ZERO).unwrap();
        let art = ascii(&pixels);
        assert!(luma(&pixels, 2, SIZE - 3) <= 5 && luma(&pixels, 2, 2) <= 5, "属性を外せば空は敷かれない\n{art}");
        let unlit = luma(&pixels, MESH_X, MESH_Y);
        assert!(unlit > 5 && unlit != center, "属性を外せば固定の灯に戻る、got {unlit}\n{art}");
    }

    /// 面の応え方: 同じ空(下だけ白)と上を向いた面で、艶消しは薄く、鏡は上の黒を映し、
    /// ガラスは屈折して下の白を通す。Glass は効果棚の 1 枚で、param は property。
    #[test]
    fn glass_and_mirror_answer_the_environment_differently_from_matte() {
        use crate::doc::store::{EffectId, EffectInstance};
        const GLASS: &str = "motolii.glass";
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "floor.png", 0, 255);
        let obj = dir.path().join("tilted.obj");
        // 法線は上(世界の -y)とカメラ(-z)の間。
        std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 1 1 0\nv -1 1 0\nvn 0 -0.7071 -0.7071\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
        let mut engine = Engine::new().unwrap();
        let render = |engine: &mut Engine, surface: &[(&str, f64)]| -> u8 {
            let mut doc = scene(dir.path(), &sky, true);
            std::fs::copy(&obj, dir.path().join("quad.obj")).unwrap();
            let mesh = LayerId(2);
            if !surface.is_empty() {
                doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: GLASS.into() }] }).unwrap();
                for (name, value) in surface {
                    doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
                }
            }
            engine.models.clear();
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            luma(&pixels, MESH_X, MESH_Y)
        };
        let matte = render(&mut engine, &[]);
        let mirror = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)]);
        let glass = render(&mut engine, &[("ior", 1.5), ("roughness", 0.0), ("transmission", 1.0)]);
        assert!(mirror < matte && matte < glass, "mirror {mirror} matte {matte} glass {glass}");
        assert!(mirror <= 20, "鏡は上の黒を映す、got {mirror}");
        assert!(glass >= 150, "ガラスは下の白を通す、got {glass}");
    }

    /// ガラスは背後に描かれた層を屈折して通す。白い空の前に赤い板、その手前のガラス越しに赤が見える。
    /// 鏡にすれば空の白を映して赤くならない。
    #[test]
    fn glass_refracts_the_layers_drawn_behind_it() {
        use crate::doc::store::{EffectId, EffectInstance};
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "white.png", 255, 255);
        let red = dir.path().join("red.png");
        let mut img = image::RgbaImage::new(SIZE, SIZE);
        for px in img.pixels_mut() { *px = image::Rgba([255, 0, 0, 255]); }
        img.save(&red).unwrap();
        let mut engine = Engine::new().unwrap();
        let render = |engine: &mut Engine, surface: &[(&str, f64)], opacity: f64| -> [u8; 3] {
            let mut doc = scene(dir.path(), &sky, true);
            let board = file_layer(&mut doc, 3, 0, &red);
            doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
            let mesh = LayerId(2);
            doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
            for (name, value) in surface {
                doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
            }
            doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(property::OPACITY).unwrap(), value: Value::F64(opacity) }).unwrap();
            engine.models.clear();
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
            [pixels[i], pixels[i + 1], pixels[i + 2]]
        };
        let glass = render(&mut engine, &[("ior", 1.5), ("roughness", 0.0), ("transmission", 1.0), ("metallic", 0.0)], 1.0);
        assert!(glass[0] > 150 && glass[1] < 80, "ガラス越しに赤い板が見える、got {glass:?}");
        let mirror = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)], 1.0);
        assert!(mirror[1] > 150, "鏡は白い空を映す、got {mirror:?}");
        let half = render(&mut engine, &[("metallic", 1.0), ("roughness", 0.0), ("transmission", 0.0)], 0.5);
        assert!(half[1] > 80 && half[1] < mirror[1] - 10, "mesh coverage attenuates reflection, like the rectangle: half {half:?}, solid {mirror:?}");
    }

    /// 分散は背後の像を波長で分ける(KHR_materials_dispersion、three.js の読み)。灰色しか無い場面 —
    /// 白い空、左黒右白の板、その手前の斜めのガラス — は dispersion 0 なら灰のまま、
    /// 分散を入れると境目で赤と青が割れ、強めるほど伸びる。
    #[test]
    fn dispersion_splits_the_backdrop_edge_into_colors() {
        use crate::doc::store::{EffectId, EffectInstance};
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "white.png", 255, 255);
        let edge = dir.path().join("edge.png");
        let mut img = image::RgbaImage::new(SIZE, SIZE);
        for (x, _, px) in img.enumerate_pixels_mut() {
            let v = if x < SIZE / 2 + 8 { 0 } else { 255 };
            *px = image::Rgba([v, v, v, 255]);
        }
        img.save(&edge).unwrap();
        let mut engine = Engine::new().unwrap();
        let mut spread = |dispersion: f64| -> u8 {
            let mut doc = scene(dir.path(), &sky, true);
            // 正面の板では屈折が曲がらず色も割れない。板を 45° に傾ける(法線は camera 側の -z 成分を持つ)。
            std::fs::write(dir.path().join("quad.obj"), "v -1 -1 -1\nv 1 -1 1\nv 1 1 1\nv -1 1 -1\nvn 0.7071 0 -0.7071\nf 1//1 2//1 3//1\nf 1//1 3//1 4//1\n").unwrap();
            let board = file_layer(&mut doc, 3, 0, &edge);
            doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
            let mesh = LayerId(2);
            doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
            for (name, value) in [("ior", 3.0), ("roughness", 0.0), ("transmission", 1.0), ("metallic", 0.0), ("dispersion", dispersion)] {
                doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(value) }).unwrap();
            }
            engine.models.clear();
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            pixels.chunks(4).map(|p| p[0].abs_diff(p[2])).max().unwrap()
        };
        let none = spread(0.0);
        let glass = spread(2.0);
        let exaggerated = spread(20.0);
        assert!(none <= 1, "灰色の場面は分散 0 で灰のまま、got {none}");
        // 64 px の場面では屈折角の差が画素の何分の一かにしかならない。割れが出て、値に比例して伸びることを見る。
        assert!(glass >= 3, "分散 2 で境目の赤と青が割れ始める、got {glass}");
        assert!(exaggerated > 40 && exaggerated > glass * 4, "分散を強めると割れが伸びる、got {glass} → {exaggerated}");
    }

    /// backdrop の mip は粗さが読む段までしか焼かない。粗さ 0 のガラスは写し 1 段、粗さ 1 は全段。
    #[test]
    fn backdrop_mips_stop_where_the_roughness_stops_reading() {
        use crate::doc::store::{EffectId, EffectInstance};
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "white.png", 255, 255);
        let red = dir.path().join("red.png");
        image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
        let mut engine = Engine::new().unwrap();
        let mut levels_per_copy = |roughness: f64| -> u64 {
            let mut doc = scene(dir.path(), &sky, true);
            let board = file_layer(&mut doc, 3, 0, &red);
            doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
            let mesh = LayerId(2);
            doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
            doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new("effect.0.param.roughness").unwrap(), value: Value::F64(roughness) }).unwrap();
            engine.models.clear();
            let before = engine.surface_work();
            engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            let after = engine.surface_work();
            let copies = after.backdrop_copies - before.backdrop_copies;
            assert!(copies > 0, "ガラスは backdrop を写す");
            (after.backdrop_mip_levels - before.backdrop_mip_levels) / copies
        };
        let full = u64::from(re_renderer::resource_managers::MipmapGenerator::mip_level_count(SIZE, SIZE));
        assert_eq!(levels_per_copy(0.0), 1, "粗さ 0 は写しだけ");
        let rough = levels_per_copy(1.0);
        assert!(rough > 1 && rough <= full, "粗さ 1 は段を焼く、got {rough} of {full}");
    }

    /// 光は環境から来て、作者は奪うだけ(裁定 2026-09-10)。空の一点が明るい環境(太陽は camera 側の左上)の
    /// 前に赤い板、その手前(camera 側)に板 1 枚。板に「光を遮る」を付けると、板の右下に影が落ちて
    /// 赤が暗くなる。付けなければ変わらない。遠くの赤も変わらない。
    #[test]
    fn a_layer_that_blocks_light_casts_a_shadow_on_the_board_behind_it() {
        let dir = tempfile::tempdir().unwrap();
        let sky = dir.path().join("sun.png");
        let mut img = image::RgbaImage::from_pixel(8, 4, image::Rgba([40, 40, 40, 255]));
        img.put_pixel(0, 1, image::Rgba([255, 255, 255, 255]));
        img.save(&sky).unwrap();
        let red = dir.path().join("red.png");
        image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
        let mut engine = Engine::new().unwrap();
        let mut render = |blocks: bool| -> (Vec<u8>, u64) {
            let mut doc = scene(dir.path(), &sky, true);
            let board = file_layer(&mut doc, 3, 0, &red);
            doc.apply(Intent::SetConstant { layer: board, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
            let blocker = LayerId(2);
            doc.apply(Intent::SetConstant { layer: blocker, property: PropertyId::new(property::POSITION_Z).unwrap(), value: Value::F64(-20.0) }).unwrap();
            doc.apply(Intent::SetAttrs { layer: blocker, patch: LayerAttrsPatch { blocks_light: Some(blocks), ..Default::default() } }).unwrap();
            engine.models.clear();
            let before = engine.surface_work().light_captures;
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            (pixels, engine.surface_work().light_captures - before)
        };
        let (lit, captures_lit) = render(false);
        let (shaded, captures) = render(true);
        assert_eq!(captures_lit, 0, "遮る層が無ければ型紙は描かない");
        assert_eq!(captures, 1, "遮る層があれば型紙 1 枚");
        let red_at = |p: &[u8], x: u32, y: u32| p[((y * SIZE + x) * 4) as usize];
        // 板(位置 32,32 から 12 倍、camera 側へ 20)の右下、板の外の赤。
        let (sx, sy) = (60, 60);
        assert!(red_at(&shaded, sx, sy) + 20 < red_at(&lit, sx, sy), "影で赤が暗くなる: lit {} shaded {}\n{}", red_at(&lit, sx, sy), red_at(&shaded, sx, sy), ascii(&shaded));
        assert_eq!(red_at(&shaded, 4, 4), red_at(&lit, 4, 4), "光線から外れた赤は変わらない");
    }

    /// The mesh Glass oracle, applied to a premultiplied 2D surface (GPU Gems 2 ch.19).
    /// Coverage is independent of optical transmission, including the mix-mode route.
    #[test]
    fn shared_surface_glass_refracts_on_rectangles_and_preserves_coverage() {
        use crate::doc::store::{BlendMode, EffectId, EffectInstance};
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "white.png", 255, 255);
        let red = dir.path().join("red.png");
        image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
        let cutout = dir.path().join("cutout.png");
        let mut img = image::RgbaImage::new(SIZE, SIZE);
        for (x, _, px) in img.enumerate_pixels_mut() {
            *px = image::Rgba([255, 255, 255, if x < 16 { 0 } else if x < 32 { 128 } else { 255 }]);
        }
        img.save(&cutout).unwrap();
        let mut doc = scene(dir.path(), &sky, true);
        doc.apply(Intent::RemoveLayer(LayerId(2))).unwrap();
        let board = file_layer(&mut doc, 3, 0, &red);
        let plate = file_layer(&mut doc, 4, 1, &cutout);
        for layer in [board, plate] {
            doc.apply(Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([0.0, 0.0]) }).unwrap();
        }
        doc.apply(Intent::SetEffects { layer: plate, effects: vec![EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() }] }).unwrap();
        let mut engine = Engine::new().unwrap();
        for blend_mode in [BlendMode::Normal, BlendMode::Screen] {
            doc.apply(Intent::SetAttrs { layer: plate, patch: LayerAttrsPatch { blend_mode: Some(blend_mode), ..Default::default() } }).unwrap();
            for (metallic, transmission) in [(0.0, 1.0), (1.0, 0.0)] {
                for (name, value) in [("ior", 1.5), ("roughness", 0.0), ("metallic", metallic), ("transmission", transmission)] {
                    doc.apply(Intent::SetConstant { layer: plate, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(value) }).unwrap();
                }
                let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
                assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
                let sample = |x: u32| { let i = ((32 * SIZE + x) * 4) as usize; &pixels[i..i+4] };
                let hole = sample(8);
                assert!(hole[0] > 150 && hole[1] < 10, "transparent hole retains red: {hole:?}, {blend_mode:?}");
                let solid = sample(44);
                if transmission > 0.0 {
                    assert!(solid[0] > 150 && solid[1] < 80, "2D glass transmits red: {solid:?}, {blend_mode:?}");
                } else {
                    assert!(solid[1] > 150, "2D mirror reflects white environment: {solid:?}, {blend_mode:?}");
                    let edge = sample(24);
                    assert!(edge[1] > 80 && edge[1] < solid[1] - 10, "half coverage blends reflected radiance: edge {edge:?}, solid {solid:?}");
                }
            }
        }
    }

    /// exr も hdr と同じ線形 f32 の道を通り、1.0 超が残る。
    #[test]
    fn exr_keeps_values_above_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sun.exr");
        let mut img = image::Rgb32FImage::new(2, 2);
        for px in img.pixels_mut() { *px = image::Rgb([4.0, 0.5, 1.0]); }
        img.save(&path).unwrap();
        let (rgb, w, h) = crate::render::engine::decode_still_linear_rgb(path.to_str().unwrap()).unwrap();
        assert_eq!((w, h), (2, 2));
        assert!((rgb[0] - 4.0).abs() < 0.05 && (rgb[1] - 0.5).abs() < 0.01, "{:?}", &rgb[..3]);
        assert!(crate::render::media::is_still_image_path(&path), "exr は画の門を通る");
    }

    /// Turbulent Displace は棚の 1 枚で、網の頂点を GPU で動かす: 掛けた絵は掛けない絵と違い、
    /// Evolution を進めるとまた違う(時刻で流れる)。
    #[test]
    fn turbulent_displace_moves_mesh_vertices_and_evolves() {
        use crate::doc::store::{EffectId, EffectInstance};
        const TURBULENT_DISPLACE: &str = "motolii.turbulent_displace";
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "sky.png", 255, 0);
        let mut engine = Engine::new().unwrap();
        let mut render = |params: Option<&[(&str, f64)]>| -> Vec<u8> {
            let mut doc = scene(dir.path(), &sky, true);
            let mesh = LayerId(2);
            if let Some(params) = params {
                doc.apply(Intent::SetEffects { layer: mesh, effects: vec![EffectInstance { id: EffectId(0), plugin_id: TURBULENT_DISPLACE.into() }] }).unwrap();
                for (name, value) in params {
                    doc.apply(Intent::SetConstant { layer: mesh, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
                }
            }
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            pixels
        };
        let differing = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x[0].abs_diff(y[0]) > 8).count();
        let still = render(None);
        let space = render(Some(&[("amount", 30.0), ("size", 6.0), ("along", 1.0)]));
        let later = render(Some(&[("amount", 30.0), ("size", 6.0), ("along", 1.0), ("evolution", 2.0)]));
        let normal = render(Some(&[("amount", 8.0), ("size", 6.0), ("along", 0.0)]));
        assert!(differing(&still, &space) > 20, "Space の変位で絵が変わる: {}", differing(&still, &space));
        assert!(differing(&space, &later) > 20, "Evolution で流れる: {}", differing(&space, &later));
        assert!(differing(&still, &normal) > 20, "Normal の変位で陰影が変わる: {}", differing(&still, &normal));
    }

    /// 2D は 3D の部分集合: 同じ Turbulent Displace が板にも効く。板は場を「標本位置のずれ」として見せる
    /// (面内の変位はそのまま、法線方向の変位は視差)。Space で絵が変わり、Evolution でまた変わり、
    /// Amount 0 は掛けない絵と 1 階調も違わない。
    #[test]
    fn turbulent_displace_warps_a_flat_picture_too() {
        use crate::doc::store::{EffectId, EffectInstance};
        const TURBULENT_DISPLACE: &str = "motolii.turbulent_displace";
        let dir = tempfile::tempdir().unwrap();
        let sky = sky_png(dir.path(), "sky.png", 0, 0);
        let checker = dir.path().join("checker.png");
        let mut img = image::RgbaImage::new(SIZE, SIZE);
        for (x, y, px) in img.enumerate_pixels_mut() {
            let v = if (x / 8 + y / 8) % 2 == 0 { 255 } else { 0 };
            *px = image::Rgba([v, v, v, 255]);
        }
        img.save(&checker).unwrap();
        let mut engine = Engine::new().unwrap();
        let mut render = |params: Option<&[(&str, f64)]>| -> Vec<u8> {
            let mut doc = scene(dir.path(), &sky, false);
            doc.apply(Intent::RemoveLayer(LayerId(2))).unwrap();
            let plate = file_layer(&mut doc, 3, 1, &checker);
            if let Some(params) = params {
                doc.apply(Intent::SetEffects { layer: plate, effects: vec![EffectInstance { id: EffectId(0), plugin_id: TURBULENT_DISPLACE.into() }] }).unwrap();
                for (name, value) in params {
                    doc.apply(Intent::SetConstant { layer: plate, property: PropertyId::new(&format!("effect.0.param.{name}")).unwrap(), value: Value::F64(*value) }).unwrap();
                }
            }
            let pixels = engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
            assert!(engine.layer_failures().is_empty(), "{:?}", engine.layer_failures());
            pixels
        };
        let differing = |a: &[u8], b: &[u8]| a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x[0].abs_diff(y[0]) > 8).count();
        let still = render(None);
        let zero = render(Some(&[("amount", 0.0)]));
        let space = render(Some(&[("amount", 6.0), ("size", 10.0), ("along", 1.0)]));
        let later = render(Some(&[("amount", 6.0), ("size", 10.0), ("along", 1.0), ("evolution", 2.0)]));
        assert_eq!(still, zero, "Amount 0 は掛けない絵と同じ");
        assert!(differing(&still, &space) > 20, "Space の変位で板の絵がずれる: {}", differing(&still, &space));
        assert!(differing(&space, &later) > 20, "Evolution で流れる: {}", differing(&space, &later));
    }

    /// hdr の 1.0 超は潰れない。環境の意味はここにある。
    #[test]
    fn hdr_keeps_values_above_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sun.hdr");
        let file = std::fs::File::create(&path).unwrap();
        image::codecs::hdr::HdrEncoder::new(file)
            .encode(&[image::Rgb([4.0f32, 0.5, 1.0]); 4], 2, 2)
            .unwrap();
        let (rgb, w, h) = crate::render::engine::decode_still_linear_rgb(path.to_str().unwrap()).unwrap();
        assert_eq!((w, h), (2, 2));
        assert!((rgb[0] - 4.0).abs() < 0.05 && (rgb[1] - 0.5).abs() < 0.01, "{:?}", &rgb[..3]);
        assert!(crate::render::media::is_still_image_path(&path), "hdr は画の門を通る");
    }
}


/// preview = export。窓へ渡す形式(`PRESENTABLE_FORMAT`)で描いた画素が、export の
/// 読み戻しと**1 階調も違わない**ことを縛る。sRGB 形式にすると composite shader の
/// `srgb_from_linear` と hardware で二重に encode され、窓だけ白く浮く(2026-09-07)。
#[cfg(test)]
mod presentable_matches_export {
    use crate::doc::store::{Composition, Document, Fps, Intent, RationalTime};

    #[test]
    fn the_window_target_holds_the_same_bytes_as_the_export_readback() {
        let (w, h) = (64u32, 32u32);
        let fps = Fps::try_new(30, 1).unwrap();
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: w,
            height: h,
            fps,
            duration_frames: 1,
            // 中間調でないと encode の回数が見えない
            background: [0.5, 0.25, 0.1, 1.0],
        }))
        .unwrap();
        let mut engine = super::Engine::new().unwrap();
        let t = RationalTime::try_from_frame(0, fps).unwrap();
        let export = engine.render_frame(&doc.view(), t).unwrap();
        assert!(export[0] > 8 && export[0] < 247, "中間調のはず: {:?}", &export[..4]);

        let format = crate::render::compositor::PRESENTABLE_FORMAT;
        let device = engine.gpu_device().clone();
        let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        engine.render_frame_into(&doc.view(), t, &target).unwrap();

        let bytes_per_row = (w * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(bytes_per_row * h),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(h),
                },
            },
            size,
        );
        engine.gpu_queue().submit([encoder.finish()]);
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = slice.get_mapped_range();

        let bgra = format!("{format:?}").starts_with("Bgra");
        for y in 0..h as usize {
            for x in 0..w as usize {
                let window = &data[y * bytes_per_row as usize + x * 4..][..4];
                let exported = &export[(y * w as usize + x) * 4..][..4];
                for c in 0..3 {
                    let wc = if bgra { 2 - c } else { c };
                    assert_eq!(window[wc], exported[c], "({x},{y}) channel {c}: 窓 {:?} export {:?}", window, exported);
                }
            }
        }
    }

}

#[cfg(test)]
mod reflection_tests;
#[cfg(test)]
mod antialiasing_tests;

#[cfg(test)]
mod response_tests;

#[cfg(test)]
mod visibility_tests;
