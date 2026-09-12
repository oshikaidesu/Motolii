use re_renderer::renderer::{
    ColorMapper, ColormappedTexture, RectangleOptions, TextureAlpha, TexturedRect,
};
use re_renderer::view_builder::{BlendWithBackground, OrthographicCameraMode, Projection, RenderMode, TargetConfiguration};
use re_renderer::{RenderContext, Rgba};

mod clip;
mod device;
pub(crate) mod effects;
mod environment;
pub(crate) mod extrude;
mod headless;
mod matte;
pub(crate) mod mesh;
mod surface_scene;
#[cfg(test)]
mod reflection_diagnostic;
mod reflection_cache;
mod measurement;
pub use measurement::FrameMeasurement;
mod point_cloud;
mod presentable;
mod render_basic;
mod render_effects;
pub(crate) mod paths;
mod sequential;
mod selection_bounds;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// 層の合成に使う、上流(`reference/vello-blend.wgsl`)の番号体系。
/// `(mix << 8) | compose` で、compose は常に `COMPOSE_SRC_OVER`(=3)。
/// Normal は固定ブレンド段で届くのでここへは来ない。Add は `COMPOSE_PLUS`(mix は Normal)。
fn vello_blend_mode(mode: BlendMode) -> Option<u32> {
    const SRC_OVER: u32 = 3;
    const PLUS: u32 = 12;
    let mix = match mode {
        BlendMode::Normal => return None,
        BlendMode::Add => return Some(PLUS),
        BlendMode::Multiply => 1,
        BlendMode::Screen => 2,
        BlendMode::Overlay => 3,
        BlendMode::Darken => 4,
        BlendMode::Lighten => 5,
        BlendMode::ColorDodge => 6,
        BlendMode::ColorBurn => 7,
        BlendMode::HardLight => 8,
        BlendMode::SoftLight => 9,
        BlendMode::Difference => 10,
        BlendMode::Exclusion => 11,
        BlendMode::Hue => 12,
        BlendMode::Saturation => 13,
        BlendMode::Color => 14,
        BlendMode::Luminosity => 15,
    };
    Some((mix << 8) | SRC_OVER)
}

/// 合成の中間テクスチャの形式(累算器・blend/matte の出力)。
pub(crate) const BLEND_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn fixed_function_tint_alpha(mode: BlendMode, opacity: f32) -> Result<f32, CompositorError> {
    match mode {
        BlendMode::Normal => Ok(opacity),
        BlendMode::Add => Ok(0.0),
        other => Err(CompositorError::UnsupportedBlendMode(other)),
    }
}

pub(crate) fn to_point3(v: glam::Vec2, z: f32) -> glam::Vec3 {
    glam::vec3(v.x, v.y, z)
}

pub(crate) fn to_vector3(v: glam::Vec2) -> glam::Vec3 {
    glam::vec3(v.x, v.y, 0.0)
}

pub(crate) fn tilt(rotation_x: f32, rotation_y: f32) -> glam::Quat {
    glam::Quat::from_rotation_y(rotation_y.to_radians())
        * glam::Quat::from_rotation_x(rotation_x.to_radians())
}

pub(crate) fn spatial_world_from_bounds(
    transform: glam::Affine2,
    z: f32,
    rotation_x: f32,
    rotation_y: f32,
    bounds: crate::render::media::SpatialBounds,
) -> glam::Affine3A {
    let rotation = tilt(rotation_x, rotation_y);
    let matrix = transform.matrix2;
    let basis_x = rotation * glam::vec3(matrix.x_axis.x, matrix.x_axis.y, 0.0);
    let basis_y = rotation * glam::vec3(matrix.y_axis.x, matrix.y_axis.y, 0.0);
    let basis_z = rotation * glam::Vec3::Z * crate::doc::core::depth_scale(basis_x, basis_y);
    let size = glam::Vec3::from(bounds.size());
    let center = size * 0.5;
    let center_xy = transform.transform_point2(center.truncate());
    let world_center = glam::vec3(center_xy.x, center_xy.y, z);
    let translation = world_center
        - basis_x * center.x
        - basis_y * center.y
        - basis_z * center.z;
    let world_from_normalized = glam::Affine3A::from_cols(
        basis_x.into(),
        basis_y.into(),
        basis_z.into(),
        translation.into(),
    );
    world_from_normalized
        * glam::Affine3A::from_translation(-glam::Vec3::from(bounds.min))
}

pub(crate) fn spatial_placement_from_bounds(
    placement: LayerPlacement,
    bounds: crate::render::media::SpatialBounds,
) -> glam::Affine3A {
    match placement.world_transform {
        Some(world) => {
            let origin = glam::vec3(bounds.min[0], bounds.min[1], (bounds.min[2] + bounds.max[2]) * 0.5);
            crate::doc::core::depth_scaled(world) * glam::Affine3A::from_translation(-origin)
        }
        None => spatial_world_from_bounds(
            placement.transform, placement.z, placement.rotation_x, placement.rotation_y, bounds,
        ),
    }
}

pub(crate) fn projected_spatial_placement(
    comp: CompSpec,
    camera: ResolvedCamera,
    projection: crate::doc::store::LayerProjection,
    placement: LayerPlacement,
    bounds: crate::render::media::SpatialBounds,
) -> glam::Affine3A {
    let world = spatial_placement_from_bounds(placement, bounds);
    let center = world.transform_point3((glam::Vec3::from(bounds.min) + glam::Vec3::from(bounds.max)) * 0.5);
    crate::doc::core::layer_projection_transform(comp, camera, projection, center) * world
}

pub fn tilted_corners(
    transform: glam::Affine2,
    local_min: glam::Vec2,
    local_size: glam::Vec2,
    z: f32,
    rotation_x: f32,
    rotation_y: f32,
) -> (glam::Vec3, glam::Vec3, glam::Vec3) {
    let q = tilt(rotation_x, rotation_y);
    let u = q * to_vector3(transform.transform_vector2(glam::Vec2::new(local_size.x, 0.0)));
    let v = q * to_vector3(transform.transform_vector2(glam::Vec2::new(0.0, local_size.y)));
    let center = to_point3(transform.transform_point2(local_min + local_size * 0.5), z);
    (center - (u + v) * 0.5, u, v)
}

#[allow(clippy::too_many_arguments)]
pub fn projected_corners(
    comp: CompSpec,
    camera: ResolvedCamera,
    projection: crate::doc::store::LayerProjection,
    transform: glam::Affine2,
    local_min: glam::Vec2,
    local_size: glam::Vec2,
    z: f32,
    rotation_x: f32,
    rotation_y: f32,
) -> (glam::Vec3, glam::Vec3, glam::Vec3) {
    let (corner, u, v) = tilted_corners(transform, local_min, local_size, z, rotation_x, rotation_y);
    let correction = crate::doc::core::layer_projection_transform(comp, camera, projection, corner + (u + v) * 0.5);
    (correction.transform_point3(corner), correction.transform_vector3(u), correction.transform_vector3(v))
}

pub fn projected_placement_corners(
    comp: CompSpec,
    camera: ResolvedCamera,
    projection: crate::doc::store::LayerProjection,
    placement: LayerPlacement,
    local_min: glam::Vec2,
    local_size: glam::Vec2,
) -> (glam::Vec3, glam::Vec3, glam::Vec3) {
    let Some(world) = placement.world_transform else {
        return projected_corners(comp, camera, projection, placement.transform, local_min, local_size,
            placement.z, placement.rotation_x, placement.rotation_y);
    };
    let corner = world.transform_point3(local_min.extend(0.0));
    let u = world.transform_vector3(glam::vec3(local_size.x, 0.0, 0.0));
    let v = world.transform_vector3(glam::vec3(0.0, local_size.y, 0.0));
    let correction = crate::doc::core::layer_projection_transform(comp, camera, projection, corner + (u + v) * 0.5);
    (correction.transform_point3(corner), correction.transform_vector3(u), correction.transform_vector3(v))
}

/// 上げた画素の扱い。sRGB 形式(blend の scratch)は乗算済みで、decode は hardware。
/// それ以外(文字・図形・静止画・動画)は**非乗算の sRGB** で上げ、shader が decode → 乗算の順で扱う。
/// 乗算済みを非乗算として decode すると α の中間(文字の縁)が暗く沈む(色の再点検 CV2)。
pub(crate) fn premultiplied_texture(texture: GpuTexture2D) -> ColormappedTexture {
    let srgb = texture.format().is_srgb();
    ColormappedTexture {
        decode_srgb: !srgb,
        texture,
        range: [0.0, 1.0],
        gamma: 1.0,
        texture_alpha: if srgb { TextureAlpha::AlreadyPremultiplied } else { TextureAlpha::SeparateAlpha },
        color_mapper: ColorMapper::OffRGB,
        shader_decoding: None,
    }
}

pub fn clear_color(background: [f32; 4]) -> Rgba {
    let q = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba::from_srgba_unmultiplied(
        q(background[0]),
        q(background[1]),
        q(background[2]),
        q(background[3]),
    )
}

pub const NO_BACKGROUND: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub use headless::{HeadlessError, HeadlessGpu};

pub use effects::EffectPass;
pub use effects::catalog::EffectStage;
pub use clip::ClipSpec;

pub(crate) use effects::catalog::catalog_snapshot;
pub use effects::catalog::{bind_catalog_runtime, catalog_generation, catalog_source_roots, refresh_effect_catalog, refresh_effect_catalog_for, watch_effect_catalog, CatalogRefresh, CatalogRuntime, CatalogWatcher, EffectDescriptor, EffectParamDescriptor};
pub use effects::{IsfInput, IsfInputType, IsfManifest};
pub(crate) use effects::IsfStage;

pub use matte::MatteMode;

pub use presentable::{check_presentable_target, PRESENTABLE_FORMAT};

/// 描く先の窓: target の画素寸法と、comp 画像(出力寸法の投影)のどの矩形をそこへ写すか。
/// Camera View は出力そのもの(窓 = comp、関心域 = 全体)。Stage はタブの寸法へ描き、
/// Flutter の zoom/pan を関心域として受ける(rerun の `viewport_transformation`、pan & scan)。
/// 世界の座標は変わらない — 変わるのは投影の切り取りだけ。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub width: u32,
    pub height: u32,
    /// comp 画像の px で `[x, y, w, h]`。
    pub roi: [f32; 4],
    /// 2D・2.5D を置くカメラ。`None` は作中カメラ(出力 = 箱の中身)。Stage は既定カメラ:
    /// Boxcam の Original Comp で、作中カメラが動いても世界は動かず、箱だけが動く。
    pub projection_camera: Option<crate::doc::core::ResolvedCamera>,
}

impl Window {
    pub fn output(comp: CompSpec) -> Self {
        Self { width: comp.width, height: comp.height, roi: [0.0, 0.0, comp.width as f32, comp.height as f32], projection_camera: None }
    }
    pub(crate) fn viewport(&self, comp: CompSpec) -> re_renderer::RectTransform {
        re_renderer::RectTransform {
            region_of_interest: re_renderer::RectF32 { min: glam::vec2(self.roi[0], self.roi[1]), extent: glam::vec2(self.roi[2], self.roi[3]) },
            region: re_renderer::RectF32 { min: glam::Vec2::ZERO, extent: glam::vec2(comp.width as f32, comp.height as f32) },
        }
    }
    pub(crate) fn size(&self) -> [u32; 2] { [self.width, self.height] }
    pub(crate) fn size_f32(&self) -> [f32; 2] { [self.width as f32, self.height as f32] }
}

pub use re_renderer::resource_managers::GpuTexture2D;

pub use crate::doc::core::{CompSpec, LayerPlacement, ResolvedCamera};

#[derive(Clone)]
pub struct Layer {
    pub content: LayerContent,
    pub size: [f32; 2],
    pub placement: LayerPlacement,
    pub projection: crate::doc::store::LayerProjection,
    pub projection_camera: ResolvedCamera,
    pub blend_mode: BlendMode,
    /// 板・網が共有する場と表面のプログラムとパラメータ(板は場を標本位置のずれとして見せる)。
    pub shading: effects::surface_program::SurfaceShading,
    /// 点群を動かす場(Turbulent Displace の CPU の写し)。板と網は `shading` の hook が動かす。
    pub displace: point_cloud::PointDisplace,
    /// 世界の平面で切る(板・点群・網が同じ式)。
    pub clip: Option<clip::ClipSpec>,
    /// 光を遮る: 太陽から見た型紙に描かれ、表面を持つ全ての層へ影(透過なら色)を落とす。
    pub blocks_light: bool,
    /// Stage で選ばれている層の番号(1..=255、0 は無し): outline の object-id mask に描かれ、
    /// その画面上の広がりが籠になる(export には出ない)。
    pub outline: u8,
    /// 絵の論理の枠(素材座標の大きさ・原点・画素数)。効果はこの枠の論理 px で評価し、
    /// 描画密度を上げても reach・radius が変わらない。無ければ 1 px = 1 論理 px。
    pub frame: Option<effects::vism::ImageFrame>,
}

#[derive(Clone)]
pub struct LayerWithPasses {
    pub layer: Layer,
    pub passes: Vec<EffectPass>,
}

#[derive(Debug, thiserror::Error)]
pub enum CompositorError {
    #[error("GPU の用意に失敗した: {0}")]
    Headless(#[from] HeadlessError),
    #[error("re_renderer の context を作れない: {0}")]
    Context(String),
    #[error("板の組み立てに失敗した: {0}")]
    Rectangles(String),
    #[error("view の組み立てに失敗した: {0}")]
    View(String),
    #[error("描画に失敗した: {0}")]
    Draw(String),
    #[error("読み戻しが返ってこなかった")]
    ReadbackMissing,
    #[error("effect pass のオフスクリーン往復に失敗した: {0}")]
    Effect(String),
    #[error("ISF effect の読み込みに失敗した: {0}")]
    Isf(String),
    #[error("この入口は分離可能 blend mode を表現できない: {0:?}")]
    UnsupportedBlendMode(BlendMode),
    #[error("共有面の画素形式が Host 仕様ではない: {got}")]
    PresentableFormat { got: String },
    #[error("共有面のサイズが comp と違う: got={got:?} expected={expected:?}")]
    PresentableSize { got: [u32; 2], expected: [u32; 2] },
    #[error("共有面に RENDER_ATTACHMENT が無い")]
    PresentableUsage,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderTiming {
    pub build_us: u128,
    pub gpu_us: u128,
    pub readback_us: u128,
}

impl RenderTiming {
    pub fn total_us(&self) -> u128 {
        self.build_us + self.gpu_us + self.readback_us
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SurfaceWork {
    pub scene_captures: u64,
    /// 太陽から見た型紙(light cookie)を描いた回数。
    pub light_captures: u64,
    pub main_runs: u64,
    pub backdrop_copies: u64,
    pub backdrop_allocations: u64,
    /// backdrop の写しごとに焼いた mip の段数(base 込み)の合計。
    pub backdrop_mip_levels: u64,
    pub mesh_batches: u64,
    pub mesh_instances_uploaded: u64,
    pub mesh_instance_upload_bytes: u64,
    pub draw_data_prepare_us: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    /// 焼いた効果の持ち物: 使い回した回数と焼いた回数。
    pub baked_hits: u64,
    pub bakes: u64,
    pub cache_bypasses: u64,
    pub cache_evictions: u64,
    pub cache_key_us: u64,
    pub cache_retained_texture_bytes: u64,
}

pub struct Compositor {
    pub(crate) ctx: RenderContext,
    /// 今描いている窓。`render_into_window` が置く。
    pub(crate) window: Window,
    pub(crate) measurement_enabled: bool,
    pub(crate) measurement: FrameMeasurement,
    pub(crate) surface_work: SurfaceWork,
    pub(crate) backdrop_resource: Option<sequential::BackdropResource>,
    pub(crate) reflection_cache_enabled: bool,
    pub(crate) gpu_instance_sharing_enabled: bool,
    /// 反射の撮影点を受け手ではなく送り手の箱に固定し、受け手は撮影から外す(Arm の local cubemap)。
    pub(crate) reflection_scene_probe: bool,
    #[cfg(test)]
    pub(crate) reflection_probe_experiment: u8,
    #[cfg(test)]
    pub(crate) reflection_diagnostic_enabled: bool,
    #[cfg(test)]
    pub(crate) reflection_diagnostic: Option<reflection_diagnostic::CaptureDiagnostic>,
    #[cfg(test)]
    pub(crate) reflection_diagnostic_skip: Option<usize>,
    #[cfg(test)]
    pub(crate) reflection_diagnostic_near: Option<f32>,
    pub(crate) reflection_entry: Option<reflection_cache::ReflectionEntry>,
    pub(crate) reflection_resources: Option<surface_scene::ReflectionResources>,
    pub(crate) light_cookie: Option<surface_scene::LightCookieResources>,
    pub(crate) next_readback: u64,
    pub(crate) next_effect_key: u64,
    pub(crate) effect_scratch: effects::EffectScratch,
    /// 層の持ち物: 焼いた効果。view が何枚でも、静止した層は焼かない。
    pub(crate) baked_effects: render_effects::BakedEffects,
    pub(crate) effect_programs: std::collections::HashMap<String, effects::EffectProgram>,
    /// hook の変種。鍵は「field の id | surface の id | catalog の世代」。
    pub(crate) surface_programs: std::collections::HashMap<String, std::sync::Arc<re_renderer::renderer::SurfaceProgram>>,
    /// 層と背景を混ぜる Vism(vism/blend.wgsl + 借りた式)。
    pub(crate) blend_vism: effects::EffectProgram,
    pub(crate) selection_bounds: Option<selection_bounds::SelectionBounds>,
    /// 層をマットで切る Vism(vism/matte.wgsl + 借りた svg_lum)。
    pub(crate) matte_vism: effects::EffectProgram,
    /// 同じ matte を、生成器の出力 format ごとに組んだ物(生成器を素材の alpha に閉じ込める)。
    pub(crate) coverage_programs: std::collections::HashMap<wgpu::TextureFormat, effects::EffectProgram>,
    pub(crate) catalog: std::sync::Arc<effects::catalog::CatalogSnapshot>,
    pub(crate) sequential_submits: u64,
    /// フレーム中に記録したパスの束。層ごとに submit せず、読み戻しが要る所まで貯める。
    pub(crate) pending: Vec<wgpu::CommandBuffer>,
}

type AccumulatorBacking = wgpu::Texture;

#[derive(Clone)]
pub struct GpuModelData {
    pub(crate) planar_size: Option<[f32; 2]>,
    pub(crate) revision: u64,
    pub(crate) instances: std::sync::Arc<Vec<re_renderer::renderer::GpuMeshInstance>>,
    pub(crate) bounds: crate::render::media::SpatialBounds,
    /// Every drawn vertex in model space. The Stage fits its frame to these, not to `bounds`.
    pub(crate) vertices: std::sync::Arc<Vec<glam::Vec3>>,
}

impl GpuModelData {
    pub fn bounds(&self) -> crate::render::media::SpatialBounds {
        self.bounds
    }
}

pub use environment::GpuEnvironmentData;
pub use effects::surface_program::SurfaceShading;
pub use point_cloud::PointDisplace;

/// 層が持つ中身。3D の素材はテクスチャにならず、点のまま run へ渡る。
#[derive(Clone)]
pub enum LayerContent {
    Texture(GpuTexture2D),
    /// Explicit linear premultiplied image, independent of its storage format.
    LinearTexture(GpuTexture2D),
    Cloud {
        positions: std::sync::Arc<Vec<[f32; 3]>>,
        colors: std::sync::Arc<Vec<[u8; 4]>>,
        bounds: crate::render::media::SpatialBounds,
        /// 点の直径(comp のピクセル)。
        point_size: f32,
    },
    Model(std::sync::Arc<GpuModelData>),
    /// 環境(空)。板にならず、run の背景と網の照明になる。
    Environment(std::sync::Arc<GpuEnvironmentData>),
}

impl LayerContent {
    pub fn texture(&self) -> Option<&GpuTexture2D> {
        match self {
            Self::Texture(t) | Self::LinearTexture(t) => Some(t),
            Self::Cloud { .. } | Self::Model(_) | Self::Environment(_) => None,
        }
    }
}

/// 層が run へ差し出す物。板は矩形、3D の素材は**上流の draw data のまま**積む
/// (焼かない。深度で刺さり合う)。
pub(crate) enum SequentialContent<'a> {
    Rect(&'a GpuTexture2D),
    LinearRect(&'a GpuTexture2D),
    Cloud {
        positions: &'a [[f32; 3]],
        colors: &'a [[u8; 4]],
        bounds: crate::render::media::SpatialBounds,
        point_size: f32,
    },
    Model(&'a GpuModelData),
    Environment(&'a GpuEnvironmentData),
}

impl SequentialContent<'_> {
    fn image(&self) -> Option<ColormappedTexture> {
        match self {
            Self::Rect(t) => Some(premultiplied_texture((*t).clone())),
            Self::LinearRect(t) => Some(linear_texture((*t).clone())),
            _ => None,
        }
    }
}

fn linear_texture(texture: GpuTexture2D) -> ColormappedTexture {
    let mut mapped = premultiplied_texture(texture);
    mapped.decode_srgb = false;
    mapped.texture_alpha = TextureAlpha::AlreadyPremultiplied;
    mapped
}

pub(crate) struct SequentialInput<'a> {
    content: SequentialContent<'a>,
    local_min: glam::Vec2,
    local_size: glam::Vec2,
    placement: LayerPlacement,
    projection: crate::doc::store::LayerProjection,
    projection_camera: ResolvedCamera,
    opacity: f32,
    depth_offset: i16,
    blend_mode: BlendMode,
    shading: effects::surface_program::SurfaceShading,
    displace: point_cloud::PointDisplace,
    clip: Option<clip::ClipSpec>,
    blocks_light: bool,
    outline: u8,
}

/// 選択の mask: channel A に層の番号。B は空けておく(hover を後で載せる口)。
pub(crate) fn outline_mask(id: u8) -> re_renderer::OutlineMaskPreference {
    if id != 0 { re_renderer::OutlineMaskPreference::some(id, 0) } else { re_renderer::OutlineMaskPreference::NONE }
}

pub(crate) fn sequential_target_config(
    name: &'static str,
    comp: CompSpec,
    window: Window,
    view_from_world: macaw::IsoTransform,
    projection: crate::doc::core::CameraProjection,
    environment: Option<&GpuEnvironmentData>,
) -> TargetConfiguration {
    TargetConfiguration {
        name: name.into(),
        render_mode: RenderMode::Deterministic,
        resolution_in_pixel: window.size(),
        viewport_transformation: window.viewport(comp),
        view_from_world,
        projection_from_view: Projection::Perspective {
            vertical_fov: projection.vertical_fov_radians,
            near_plane_distance: projection.near_plane_distance,
            aspect_ratio: projection.aspect_ratio,
        },
        pixels_per_point: 1.0,
        blend_with_background: BlendWithBackground::Premultiplied,
        environment: environment.map(|e| e.environment.clone()),
        ..Default::default()
    }
}

/// 出力そのものを見る view: 画素 1:1、カメラを通さない(形の描画と同じ型)。
pub(crate) fn screen_target_config(name: &'static str, window: Window) -> TargetConfiguration {
    TargetConfiguration {
        name: name.into(),
        render_mode: RenderMode::Deterministic,
        resolution_in_pixel: window.size(),
        view_from_world: macaw::IsoTransform::IDENTITY,
        projection_from_view: Projection::Orthographic {
            camera_mode: OrthographicCameraMode::TopLeftCornerAndExtendZ,
            vertical_world_size: window.height as f32,
            far_plane_distance: 1000.0,
        },
        pixels_per_point: 1.0,
        blend_with_background: BlendWithBackground::Premultiplied,
        ..Default::default()
    }
}

/// 累算(合成の地)を出力の枠いっぱいに 1 枚。`screen_target_config` の view で描く。
fn screen_rect(window: Window, imported: GpuTexture2D) -> TexturedRect {
    TexturedRect {
        top_left_corner_position: glam::Vec3::ZERO,
        extent_u: glam::vec3(window.width as f32, 0.0, 0.0),
        extent_v: glam::vec3(0.0, window.height as f32, 0.0),
        colormapped_texture: premultiplied_texture(imported),
        options: RectangleOptions {
            multiplicative_tint: Rgba::from_rgba_premultiplied(1.0, 1.0, 1.0, 1.0),
            texture_filter_magnification: re_renderer::renderer::TextureFilterMag::Nearest,
            texture_filter_minification: re_renderer::renderer::TextureFilterMin::Nearest,
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod spatial_transform_tests {
    use super::*;

    fn near(actual: glam::Vec3, expected: glam::Vec3) {
        assert!((actual - expected).length() < 1e-3, "{actual:?} != {expected:?}");
    }

    /// 大きさは x/y しか無いが、3D の素材は奥行きにも同じ倍率が掛かる(球が円盤に潰れない)。
    #[test]
    fn spatial_scale_reaches_depth() {
        let bounds = crate::render::media::SpatialBounds::from_points([[-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]]).unwrap();
        let world = spatial_world_from_bounds(
            glam::Affine2::from_scale(glam::vec2(10.0, 10.0)), 0.0, 0.0, 0.0, bounds,
        );
        let depth = world.transform_vector3(glam::Vec3::Z).length();
        assert!((depth - 10.0).abs() < 1e-3, "{depth}");
        let world3d = crate::doc::core::LayerPlacement::spatial_from_transform(
            glam::Affine2::from_scale(glam::vec2(4.0, 6.0)), [0.0, 0.0], 0.0, 0.0, 0.0, 1.0,
        );
        let placed = spatial_placement_from_bounds(
            LayerPlacement { world_transform: Some(world3d), ..Default::default() }, bounds,
        );
        assert!((placed.transform_vector3(glam::Vec3::Z).length() - 5.0).abs() < 1e-3);
    }

    #[test]
    fn world_pose_places_padded_corners_without_reapplying_legacy_tilt() {
        let parent = glam::Affine3A::from_rotation_translation(
            glam::Quat::from_rotation_y(0.7), glam::vec3(30.0, 40.0, 90.0));
        let local = glam::Affine3A::from_scale_rotation_translation(
            glam::vec3(2.0, 3.0, 1.0), glam::Quat::from_rotation_x(-0.4), glam::vec3(8.0, 10.0, 12.0));
        let world = parent * local;
        let placement = LayerPlacement {
            world_transform: Some(world),
            transform: glam::Affine2::from_translation(glam::vec2(900.0, 800.0)),
            z: 700.0, rotation_x: 60.0, rotation_y: 45.0,
            ..Default::default()
        };
        let comp = CompSpec { width: 640, height: 480 };
        let min = glam::vec2(-12.0, -12.0);
        let size = glam::vec2(124.0, 74.0);
        let (corner, u, v) = projected_placement_corners(comp, ResolvedCamera::default(),
            crate::doc::store::LayerProjection::ThreeD, placement, min, size);
        near(corner, world.transform_point3(glam::vec3(-12.0, -12.0, 0.0)));
        near(corner + u, world.transform_point3(glam::vec3(112.0, -12.0, 0.0)));
        near(corner + v, world.transform_point3(glam::vec3(-12.0, 62.0, 0.0)));
    }

    #[test]
    fn native_bounds_pose_matches_planar_corners_and_camera_projection() {
        let bounds = crate::render::media::SpatialBounds {
            min: [-5.0, 8.0, -3.0], max: [15.0, 38.0, 7.0],
        };
        let placement = LayerPlacement {
            world_transform: Some(glam::Affine3A::from_scale_rotation_translation(
                glam::vec3(2.0, 3.0, 1.0), glam::Quat::from_rotation_y(0.6), glam::vec3(20.0, 30.0, 40.0))),
            z: -900.0, rotation_x: 87.0, rotation_y: 70.0,
            ..Default::default()
        };
        let comp = CompSpec { width: 640, height: 480 };
        let camera = ResolvedCamera { center: [20.0, 40.0], zoom: 1.4, roll_degrees: 15.0, ..Default::default() };
        for projection in [crate::doc::store::LayerProjection::ThreeD,
            crate::doc::store::LayerProjection::TwoD, crate::doc::store::LayerProjection::TwoPointFiveD] {
            let native = projected_spatial_placement(comp, camera, projection, placement, bounds);
            let (corner, u, v) = projected_placement_corners(comp, camera, projection, placement,
                glam::Vec2::ZERO, glam::vec2(20.0, 30.0));
            near(native.transform_point3(glam::vec3(-5.0, 8.0, 2.0)), corner);
            near(native.transform_point3(glam::vec3(15.0, 8.0, 2.0)), corner + u);
            near(native.transform_point3(glam::vec3(-5.0, 38.0, 2.0)), corner + v);
        }
    }

    #[test]
    fn native_world_pose_preserves_zero_tilt_source_normalization() {
        let bounds = crate::render::media::SpatialBounds {
            min: [-5.0, 8.0, -3.0], max: [15.0, 38.0, 7.0],
        };
        let transform = glam::Affine2::from_scale_angle_translation(
            glam::vec2(2.0, 3.0), 0.3, glam::vec2(50.0, 70.0));
        let world = glam::Affine3A::from_cols(
            to_vector3(transform.matrix2.x_axis).into(), to_vector3(transform.matrix2.y_axis).into(),
            glam::Vec3A::Z, glam::vec3(50.0, 70.0, 30.0).into());
        let legacy = LayerPlacement { transform, z: 30.0, ..Default::default() };
        let migrated = LayerPlacement { world_transform: Some(world), ..legacy };
        let a = spatial_placement_from_bounds(legacy, bounds);
        let b = spatial_placement_from_bounds(migrated, bounds);
        for point in [bounds.min, bounds.max, [5.0, 23.0, 2.0]] {
            near(a.transform_point3(point.into()), b.transform_point3(point.into()));
        }
        let comp = CompSpec { width: 640, height: 480 };
        let fallback = projected_placement_corners(comp, ResolvedCamera::default(),
            crate::doc::store::LayerProjection::ThreeD, legacy, glam::Vec2::ZERO, glam::vec2(20.0, 30.0));
        let original = projected_corners(comp, ResolvedCamera::default(), crate::doc::store::LayerProjection::ThreeD,
            transform, glam::Vec2::ZERO, glam::vec2(20.0, 30.0), 30.0, 0.0, 0.0);
        assert_eq!(fallback, original);
    }

    #[test]
    fn mesh_and_points_share_a_center_preserving_xyz_transform() {
        let bounds = crate::render::media::SpatialBounds {
            min: [-1.0, -1.0, -1.0],
            max: [1.0, 1.0, 1.0],
        };
        let transform = glam::Affine2::from_scale_angle_translation(
            glam::Vec2::splat(20.0),
            0.0,
            glam::vec2(100.0, 200.0),
        );
        let world = spatial_world_from_bounds(transform, 30.0, 90.0, 0.0, bounds);

        let center = world.transform_point3(glam::Vec3::ZERO);
        assert!((center - glam::vec3(120.0, 220.0, 30.0)).length() < 1e-4);
        let above = world.transform_point3(glam::vec3(0.0, 1.0, 0.0));
        assert!((above.x - center.x).abs() < 1e-4);
        assert!((above.y - center.y).abs() < 1e-4);
        assert!((above.z - (center.z + 20.0)).abs() < 1e-4);
    }
}
