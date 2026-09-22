use std::collections::HashMap;

pub mod mask;
pub mod text;
pub mod strokes;

mod analysis;
#[cfg(test)]
mod analysis_contracts;
mod motion;
mod overlay;
#[cfg(test)]
mod motion_contracts;
#[cfg(test)]
mod light_reach_contracts;
mod clip;
mod render;
mod texture;
mod material;
pub use texture::content_canvas;
pub use texture::{decode_still_linear_rgb, decode_still_srgb};
mod translate;
mod blocks;
mod physics;
mod frozen;
mod frame_graph;
mod frame_graph_scene;
mod gpu_exec;

use crate::doc::core::ResolvedCamera;
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
    frame_graph: Option<frame_graph::EngineFrameGraph>,
    gpu_resource_graph: gpu_exec::GpuResourceGraph,
    /// 1 コマの解決を 2 度しない。描く側と status が同じ (版, 時刻) を続けて訊くので、
    /// 解析入力が無い時(= 両者が同じ物を解く時)だけ覚える。鍵が外れたら捨てる。
    /// 直前に**実際に解いた**拍の中身(相ごと・層の種類ごと)。memo に当たった面では空。
    resolve_tally: Vec<(&'static str, String, u64, u32)>,
    /// その拍で重かった層(µs, 層番号, 種類)。
    resolve_worst: Vec<(u64, u64, &'static str)>,
    /// Taffy is renderer preparation state, so it survives playback without
    /// becoming Document/Undo state.
    layout_flow: std::rc::Rc<crate::picture::flow::FlowCache>,
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
    /// 抜いた後の形(層 → 素材座標の輪郭)。解析の段で読み、物理の当たりに使う。
    keyed_outlines: HashMap<LayerId, std::sync::Arc<Vec<[f32; 2]>>>,
    /// 形の覚え: 書類の版とコマが同じなら読み戻さない。止まった絵は 1 回だけ。
    keyed_cache: HashMap<LayerId, (u64, i64, std::sync::Arc<Vec<[f32; 2]>>)>,
    /// 今描いている窓(画面の道の feedback は窓ごとに状態を持つ)。読み戻しの道では None = 出力寸法。
    feedback_window: Option<crate::render::compositor::Window>,
    /// 動画の復号の流れの名前空間(0 = 本番)。合成を別の時刻で描く間だけ別の値にする。
    video_stream_namespace: u64,
    /// feedback の鍵の名前空間(0 = 本番)。別の時刻の合成を描く間だけ時刻のずれの値。
    feedback_namespace: u64,
    /// この frame に別の時刻の合成(SOURCE)があった: 辿り直しはフレームを丸ごと(t′ の列も進める)。
    feedback_saw_composites: bool,
    /// Freeze の cache(層の投影の前の絵、書類の隣)。
    pub(crate) frozen: frozen::FrozenStore,
    /// 今この層を焼いている(凍った絵で差し替えず、本物を組む)。
    freezing: Option<LayerId>,
    material_picture: Option<LayerId>,
    /// この frame の組み立てで刻んだ feedback の鍵(板に焼く途中で消費された物も含む)。
    feedback_keys_seen: Vec<crate::render::compositor::FeedbackKey>,
    /// 箱のブロックの GPU の道と、このコマに集めた箱。
    blocks: blocks::BlockState,
    /// このコマで誰かの clip の下地になっている層(形でも絵に描く)。
    clip_bases: std::collections::HashSet<LayerId>,
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
    /// このコマの粒子の層の点(build_layers の頭で書類から解く)。
    particle_frames: HashMap<LayerId, ParticleFrame>,
    /// Track Overlay のこのコマの塊(解析の後、描く時に読む)。
    overlay_frames: HashMap<LayerId, analysis::OverlayFrame>,
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
    /// test では GPU を 1 つずつ使う(cargo test の並走で `a_field_moves_*` が落ちた)。持っている間は他の Engine を待たせる。
    #[cfg(test)]
    _gpu: GpuLease,
}

/// test の GPU の貸し出し: 1 度に 1 本。同じ thread が 2 つ目の Engine を作る時は待たない(自分が持っている)。
/// `--test-threads=1` に頼らず、Engine を作った時点で構造として直列になる。
#[cfg(test)]
pub(crate) struct GpuLease(Option<std::sync::MutexGuard<'static, ()>>);

#[cfg(test)]
static GPU: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[cfg(test)]
thread_local!(static GPU_HELD: std::cell::Cell<usize> = const { std::cell::Cell::new(0) });

#[cfg(test)]
impl GpuLease {
    fn take() -> Self {
        let first = GPU_HELD.with(|held| { let n = held.get(); held.set(n + 1); n == 0 });
        Self(first.then(|| GPU.lock().unwrap_or_else(|poisoned| poisoned.into_inner())))
    }
}

#[cfg(test)]
impl Drop for GpuLease {
    fn drop(&mut self) {
        GPU_HELD.with(|held| held.set(held.get() - 1));
    }
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        #[cfg(test)]
        let _gpu = GpuLease::take();
        Ok(Self {
            frame_graph: None,
            gpu_resource_graph: Default::default(),
            resolve_tally: Vec::new(),
            resolve_worst: Vec::new(),
            layout_flow: Default::default(),
            #[cfg(test)]
            _gpu,
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
            particle_frames: HashMap::new(),
            overlay_frames: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
            realtime: false,
            renders_since_video_purge: 0,
            feedback_replaying: false,
            keyed_outlines: HashMap::new(),
            keyed_cache: HashMap::new(),
            feedback_window: None,
            video_stream_namespace: 0,
            feedback_namespace: 0,
            feedback_saw_composites: false,
            frozen: Default::default(),
            freezing: None,
            material_picture: None,
            feedback_keys_seen: Vec::new(),
            blocks: Default::default(),
            clip_bases: Default::default(),
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

    pub fn layout_solver(&self) -> std::rc::Rc<dyn crate::doc::store::LayoutSolver> {
        self.layout_flow.clone()
    }

    /// 直前の描画を queue へ出した時の番号。窓は待たずに返り、これを別 thread が待つ。
    pub fn last_submission(&self) -> Option<wgpu::SubmissionIndex> {
        self.compositor.last_submission()
    }

    pub fn gpu_device(&self) -> &wgpu::Device {
        self.compositor.device()
    }

    /// (all resources, retained resources, temporal resources) currently
    /// represented by the GPU control plane.
    pub fn gpu_resource_graph_stats(&self) -> (usize, usize, usize) {
        let stats = self.gpu_resource_graph.stats();
        (stats.resources, stats.retained, stats.temporal)
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
        #[cfg(test)]
        let _gpu = GpuLease::take();
        Ok(Self {
            frame_graph: None,
            gpu_resource_graph: Default::default(),
            resolve_tally: Vec::new(),
            resolve_worst: Vec::new(),
            layout_flow: Default::default(),
            #[cfg(test)]
            _gpu,
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
            particle_frames: HashMap::new(),
            overlay_frames: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            pixels: still_pixels(),
            videos: HashMap::new(),
            realtime: false,
            renders_since_video_purge: 0,
            feedback_replaying: false,
            keyed_outlines: HashMap::new(),
            keyed_cache: HashMap::new(),
            feedback_window: None,
            video_stream_namespace: 0,
            feedback_namespace: 0,
            feedback_saw_composites: false,
            frozen: Default::default(),
            freezing: None,
            material_picture: None,
            feedback_keys_seen: Vec::new(),
            blocks: Default::default(),
            clip_bases: Default::default(),
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

    /// 直前のコマの resolve を、回したデータで割った行。層の種類が増えれば行が増える。
    pub fn resolve_tally(&self) -> &[(&'static str, String, u64, u32)] { &self.resolve_tally }

    /// 直前のコマで重かった層。
    pub fn resolve_worst(&self) -> &[(u64, u64, &'static str)] { &self.resolve_worst }

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
mod spatial_cache_tests;

#[cfg(test)]
mod environment_tests;


/// preview = export。窓へ渡す形式(`PRESENTABLE_FORMAT`)で描いた画素が、export の
/// 読み戻しと**1 階調も違わない**ことを縛る。sRGB 形式にすると composite shader の
/// `srgb_from_linear` と hardware で二重に encode され、窓だけ白く浮く(2026-09-07)。
#[cfg(test)]
mod presentable_matches_export;

#[cfg(test)]
mod reflection_tests;
#[cfg(test)]
mod antialiasing_tests;

#[cfg(test)]
mod response_tests;

#[cfg(test)]
mod visibility_tests;

/// 粒子の層の 1 コマ分の点(層の局所 px、出す元が 0)。
#[derive(Clone)]
pub(crate) struct ParticleFrame {
    pub(crate) positions: std::sync::Arc<Vec<[f32; 3]>>,
    pub(crate) colors: std::sync::Arc<Vec<[u8; 4]>>,
    pub(crate) sizes: std::sync::Arc<Vec<f32>>,
    pub(crate) bounds: crate::render::media::SpatialBounds,
    pub(crate) links: Option<std::sync::Arc<crate::render::compositor::CloudLinks>>,
}

impl ParticleFrame {
    /// 閉じた式で解いた粒に乱流(年齢で動く fbm)を足す。bounds の min は 0 に据える — 置き方が min を層の原点にするので、
    /// 粒の散らばりで層が動かないように。
    pub(crate) fn from_particles(particles: &[crate::doc::store::particles::Particle], turbulence: crate::doc::store::particles::Turbulence, links: crate::doc::store::particles::Links) -> Self {
        let mut max = [1.0f32, 1.0];
        let positions: Vec<[f32; 3]> = particles.iter().map(|p| {
            let mut at = glam::Vec3::from(p.position);
            if turbulence.amount != 0.0 {
                let q = at / turbulence.size + glam::vec3(turbulence.seed * 1.31, turbulence.seed * 0.77, p.age * 0.6);
                at += glam::vec3(re_renderer::noise::fbm3(q, 3), re_renderer::noise::fbm3(q + glam::vec3(31.7, 0.0, 0.0), 3), 0.0) * turbulence.amount * p.age.min(1.0);
            }
            max = [max[0].max(at.x), max[1].max(at.y)];
            at.into()
        }).collect();
        let colors: Vec<[u8; 4]> = particles.iter().map(|p| p.color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)).collect();
        let links = (links.distance > 0.0 && links.width > 0.0 && links.opacity > 0.0)
            .then(|| std::sync::Arc::new(crate::render::compositor::CloudLinks::near(&positions, &colors, links.distance, links.width, links.opacity)));
        // Size は直径。re_renderer の点の大きさは半径。
        let sizes = particles.iter().map(|p| p.size * 0.5).collect();
        Self {
            positions: std::sync::Arc::new(positions),
            colors: std::sync::Arc::new(colors),
            sizes: std::sync::Arc::new(sizes),
            bounds: crate::render::media::SpatialBounds { min: [0.0, 0.0, 0.0], max: [max[0], max[1], 0.0] },
            links,
        }
    }
}
