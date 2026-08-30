
use re_renderer::renderer::{
    ColorMapper, ColormappedTexture, RectangleOptions, TextureAlpha,
    TexturedRect,
};
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration,
};
use re_renderer::{GpuTexture, RenderContext, Rgba};

mod blend;
mod device;
mod effects;
mod headless;
mod matte;
mod point_cloud;
mod presentable;
mod render_basic;
mod render_effects;
mod sequential;

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

fn separable_mode_index(mode: BlendMode) -> Option<u32> {
    match mode {
        BlendMode::Normal | BlendMode::Add => None,
        BlendMode::Multiply => Some(0),
        BlendMode::Screen => Some(1),
        BlendMode::Overlay => Some(2),
        BlendMode::Darken => Some(3),
        BlendMode::Lighten => Some(4),
        BlendMode::ColorDodge => Some(5),
        BlendMode::ColorBurn => Some(6),
        BlendMode::HardLight => Some(7),
        BlendMode::SoftLight => Some(8),
        BlendMode::Difference => Some(9),
        BlendMode::Exclusion => Some(10),
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => None,
    }
}

fn nonseparable_mode_index(mode: BlendMode) -> Option<u32> {
    match mode {
        BlendMode::Normal
        | BlendMode::Add
        | BlendMode::Multiply
        | BlendMode::Screen
        | BlendMode::Overlay
        | BlendMode::Darken
        | BlendMode::Lighten
        | BlendMode::ColorDodge
        | BlendMode::ColorBurn
        | BlendMode::HardLight
        | BlendMode::SoftLight
        | BlendMode::Difference
        | BlendMode::Exclusion => None,
        BlendMode::Hue => Some(11),
        BlendMode::Saturation => Some(12),
        BlendMode::Color => Some(13),
        BlendMode::Luminosity => Some(14),
    }
}

fn two_texture_pass_mode_index(mode: BlendMode) -> Option<u32> {
    separable_mode_index(mode).or_else(|| nonseparable_mode_index(mode))
}

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

pub(crate) fn accumulator_plane_z(
    comp: CompSpec,
    camera: motolii_core::ResolvedCamera,
    centers: impl Iterator<Item = glam::Vec3>,
) -> f32 {
    let projection = motolii_core::camera_projection(comp, camera);
    let base = motolii_core::distance_from_camera(comp, 0.0);
    let farthest = centers
        .map(|c| (c - projection.eye).length())
        .fold(base, f32::max);
    farthest - base + 1.0
}

pub(crate) fn tilted_corners(
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

pub(crate) fn premultiplied_texture(texture: GpuTexture2D) -> ColormappedTexture {
    ColormappedTexture {
        decode_srgb: !texture.format().is_srgb(),
        texture,
        range: [0.0, 1.0],
        gamma: 1.0,
        texture_alpha: TextureAlpha::AlreadyPremultiplied,
        color_mapper: ColorMapper::OffRGB,
        shader_decoding: None,
    }
}

pub fn clear_color(background: [f32; 4]) -> Rgba {
    let q = |c: f32| (c * 255.0).round().clamp(0.0, 255.0) as u8;
    Rgba::from_srgba_unmultiplied(q(background[0]), q(background[1]), q(background[2]), q(background[3]))
}

pub const NO_BACKGROUND: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub use headless::{HeadlessError, HeadlessGpu};

pub use effects::EffectPass;

pub use effects::{IsfInput, IsfInputType, IsfManifest};

pub fn isf_bloom_manifest() -> &'static IsfManifest {
    static MANIFEST: std::sync::OnceLock<IsfManifest> = std::sync::OnceLock::new();
    MANIFEST.get_or_init(|| {
        effects::isf::parse_isf_source(effects::BLOOM_SOURCE)
            .expect("bloom.fs はビルドに埋め込まれた定数——parse 失敗はここのバグ")
            .0
    })
}

pub fn tri_led_manifest() -> &'static IsfManifest {
    static MANIFEST: std::sync::OnceLock<IsfManifest> = std::sync::OnceLock::new();
    MANIFEST.get_or_init(|| {
        effects::isf::parse_isf_source(effects::TRI_LED_SOURCE)
            .expect("tri_led.wgsl はビルドに埋め込まれた定数——parse 失敗はここのバグ")
            .0
    })
}

pub use matte::MatteMode;

pub use presentable::{check_presentable_target, PRESENTABLE_FORMAT};

pub use re_renderer::resource_managers::GpuTexture2D;

pub use motolii_core::{CompSpec, LayerPlacement, ResolvedCamera};

#[derive(Clone)]
pub struct Layer {
    pub texture: GpuTexture2D,
    pub size: [f32; 2],
    pub placement: LayerPlacement,
    pub pinned: bool,
    pub blend_mode: BlendMode,
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

pub struct Compositor {
    pub(crate) ctx: RenderContext,
    pub(crate) next_readback: u64,
    pub(crate) next_effect_key: u64,
    pub(crate) effect_scratch: effects::EffectScratch,
    pub(crate) glow_pipelines: effects::GlowPipelines,
    pub(crate) isf_bloom: effects::IsfProgram,
    pub(crate) wgsl_gradient: effects::WgslFragmentProgram,
    pub(crate) wgsl_tri_led: effects::WgslFragmentProgram,
    pub(crate) blend_pipelines: blend::SeparableBlendPipelines,
    pub(crate) matte_pipelines: matte::MattePipelines,
    pub(crate) sequential_submits: u64,
}

type AccumulatorBacking = wgpu::Texture;

struct SequentialInput<'a> {
    texture: &'a GpuTexture2D,
    local_min: glam::Vec2,
    local_size: glam::Vec2,
    transform: glam::Affine2,
    z: f32,
    rotation_x: f32,
    rotation_y: f32,
    pinned: bool,
    opacity: f32,
    depth_offset: i16,
    blend_mode: BlendMode,
}

pub(crate) fn sequential_target_config(
    name: &'static str,
    comp: CompSpec,
    view_from_world: macaw::IsoTransform,
    projection: motolii_core::CameraProjection,
) -> TargetConfiguration {
    TargetConfiguration {
        name: name.into(),
        render_mode: RenderMode::Deterministic,
        resolution_in_pixel: [comp.width, comp.height],
        view_from_world,
        projection_from_view: Projection::Perspective {
            vertical_fov: projection.vertical_fov_radians,
            near_plane_distance: projection.near_plane_distance,
            aspect_ratio: projection.aspect_ratio,
        },
        pixels_per_point: 1.0,
        blend_with_background: BlendWithBackground::Premultiplied,
        ..Default::default()
    }
}

fn background_rect(
    comp: CompSpec,
    camera: motolii_core::ResolvedCamera,
    imported: GpuTexture2D,
    depth_offset: i16,
    plane_z: f32,
) -> TexturedRect {
    let cancel = motolii_core::camera_screen_from_world_at_z(comp, camera, plane_z).inverse();
    TexturedRect {
        top_left_corner_position: to_point3(cancel.transform_point2(glam::Vec2::ZERO), plane_z),
        extent_u: to_vector3(cancel.transform_vector2(glam::Vec2::new(comp.width as f32, 0.0))),
        extent_v: to_vector3(cancel.transform_vector2(glam::Vec2::new(0.0, comp.height as f32))),
        colormapped_texture: premultiplied_texture(imported),
        options: RectangleOptions {
            multiplicative_tint: Rgba::from_rgba_premultiplied(1.0, 1.0, 1.0, 1.0),
            depth_offset,
            texture_filter_magnification: re_renderer::renderer::TextureFilterMag::Nearest,
            texture_filter_minification: re_renderer::renderer::TextureFilterMin::Nearest,
            ..Default::default()
        },
    }
}
