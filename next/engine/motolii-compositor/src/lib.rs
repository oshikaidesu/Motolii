//! wraps: re_renderer — 合成器。
//!
//! layer は **板**(`TexturedRect`)、重ね順は `depth_offset`、不透明度は
//! `multiplicative_tint` の alpha。カメラは**透視**(`Projection::Perspective`、裁定115) —
//! world 単位 = ピクセル・原点左上 = **AE の comp 座標**(裁定14)のまま、層の z が
//! 動いた瞬間に視差が出る 2.5D になる。既定カメラ(center=[0,0]・zoom=1・roll=0)で
//! 全層 z=0 のときは、旧正射影(裁定14)と一致する(`motolii-core::camera` が投影の
//! 正本で、機械精度でそれを縛る単体試験を持つ)。
//!
//! 背骨2: **評価経路は1本**。preview も export も [`Compositor::render`] を呼び、
//! 違いは窓の有無だけである。第二経路を作れる公開 API をここに置かない。
//!
//! 2026-08-11「direct `re_renderer` scene を禁止」はリセット裁定3(viewer 層を引かない)で
//! 撤回済み。禁止の趣旨(第二 runtime を作らない)は上の背骨2で維持する。
//!
//! GPU の起こし方も上流のものをそのまま使う — instance descriptor・adapter 選択・
//! device limits はいずれも `re_renderer::device_caps` が持っている(自前で書かない)。
//!
//! ## blend mode(2026-08-20、KNOWN.md「bm/matte/ao 未消費」を1個塞ぐ)
//!
//! [`RectangleOptions`] は `multiplicative_tint`/`depth_offset`/`outline_mask` の3つしか
//! 持たず、上流 `rectangles.rs` は transparent phase 用のパイプラインを1本
//! (`wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING`)しか作らない —
//! `RectangleRenderer` の内部で固定されていて、呼び手からパイプラインを選べない
//! (一次確認: `rectangles.rs` の `render_pipeline_color_{opaque,transparent}` の2つのみ、
//! `lines.rs`/`point_cloud.rs`/`voxel_grid.rs`/`mesh_renderer.rs`/`world_grid.rs` も全部
//! 同じ定数を使い回している = crate 全体でこれ以外の blend equation が無い)。
//! この固定式 `out = 1×src + (1-src.a)×dst`(色・alpha とも同じ係数)は
//! [`BlendMode::Normal`] そのものなので**それだけは無改造で厳密に出せる**。
//!
//! [`BlendMode::Add`] も実は無改造で出せる: `multiplicative_tint` の **alpha だけ 0** に
//! すると(RGB は opacity のまま)、上の式は `out = src + dst`(alpha は dst のまま不変)
//! になる — 加算合成の定義そのもの。`fs_main` が返すのは
//! `texture_color * rect_info.multiplicative_tint` で tint の4成分は独立に効くので、
//! rgb と a を別の値にして良い(`Rgba::from_rgba_premultiplied` は素の4引数で不変式を
//! 強制しない)。**Store 側の `motolii_store::BlendMode` に `Add` 相当の値はまだ無い**
//! (裁定67「Add は velato も落としているので後回し」)ので今はどこからも選べないが、
//! 将来そこへ足された時にこの crate 側は無改造で受けられる。
//!
//! それ以外の14モード(Multiply/Screen/Overlay/Darken/Lighten/ColorDodge/ColorBurn/
//! HardLight/SoftLight/Difference/Exclusion/Hue/Saturation/Color/Luminosity)は
//! **固定式の係数を振るだけでは表現できない**(Multiply は `dst_factor` に src の色を
//! 使う必要があり、Screen 以降は非線形で dst を shader 内で読む必要がある — どちらも
//! `RectangleOptions` の外)。ここでは実装せず、`motolii-engine` 側が明示的に弾く
//! (fork seam 候補、終了報告に記載)。
//!
//! **色空間の注意**: `ColormappedTexture::from_unorm_rgba`(一次確認)は
//! `decode_srgb = !texture.format().is_srgb()` を立てる。`upload_rgba`/`upload_yuv420p`
//! が使う format は srgb タグ無しなので、**shader は素材を sRGB gamma と見なして
//! linear へ復元してから混ぜ**、結果は `MAIN_TARGET_COLOR_FORMAT`(`Rgba8UnormSrgb`)へ
//! 書き戻る時に GPU が再エンコードする。上の式(`out = src + dst` 等)は
//! blend 演算に**渡る値**についての式であって、読み戻す8bit値の単純な整数和には
//! ならない(`tests/compose.rs` の `add_blend_*` 系はこれを踏まえて exact値ではなく
//! 「明るくなる」「1.0超えは白に飽和する」で縛っている)。

use re_renderer::renderer::{
    ColorMapper, ColormappedTexture, RectangleDrawData, RectangleOptions, TextureAlpha,
    TexturedRect,
};
use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::{GpuTexture, RenderContext, Rgba, ScreenshotProcessor, ViewBuilderId};

mod effects;
mod headless;

/// 合成器がその場で表現できる blend mode。**`re_renderer::renderer::RectangleOptions` が
/// 賄える範囲**(上のモジュール doc 参照)に絞ってあり、Document 側の
/// `motolii_store::BlendMode`(16値)とは1対1ではない — 変換と「対応外は弾く」判断は
/// `motolii-engine` の仕事(この crate は Document の語彙を知らない)。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// 通常の alpha-over。`re_renderer` の transparent パイプラインの既定そのもの。
    #[default]
    Normal,
    /// 加算合成。`multiplicative_tint.a = 0` で無改造に出せる(モジュール doc 参照)。
    /// Document 側にまだ対応する値が無いので、今は誰も選べない(将来のための先取り)。
    Add,
}

/// 板の**位置**(原点や角)。z は `LayerPlacement::z`(pinned は常に 0、裁定113)。
fn to_point3(v: glam::Vec2, z: f32) -> glam::Vec3 {
    glam::vec3(v.x, v.y, z)
}

/// 板の**辺ベクトル**(`extent_u`/`extent_v`)。板は自分の z 平面に対して常に平行
/// (裁定115: 姿勢の表現はまだ開けない)なので、方向ベクトルの z 成分は常に 0。
fn to_vector3(v: glam::Vec2) -> glam::Vec3 {
    glam::vec3(v.x, v.y, 0.0)
}

/// `HeadlessGpu` を公開しているのは、`Compositor` が持たない描画(点群など)を
/// probe が組み立てたい時に、adapter/device の起こし方だけはここの物を使わせるため
/// (module doc の警告どおり、自前で limits を書くと rerun shader の床とずれた時に
/// 原因が分からなくなる)。`RenderContext` の組み立て自体は probe 側に任せる —
/// それは `Compositor::headless()` を薄く読み直せば分かる程度の量で、ここに
/// 二重化の危険は無い。
pub use headless::{HeadlessError, HeadlessGpu};

/// layer 単位オフスクリーンパスの枠(裁定153 S2)。`effects` モジュール doc 参照。
pub use effects::EffectPass;

/// 素材ハンドル。上流の型をそのまま通す(包み直さない)。
pub use re_renderer::resource_managers::GpuTexture2D;

/// 合成の器・カメラの投影数学。**定義は `motolii-core`** にある(背骨2を依存グラフで
/// 守るため)。
pub use motolii_core::{CompSpec, LayerPlacement, ResolvedCamera};

/// 1枚の layer。**空間に立つ板**であり、2D の完成フレームではない。
///
/// 置き方は `motolii-core::LayerPlacement` をそのまま持つ。store 側の
/// `ResolvedLayer` と**同じ型**を共有しているので、置き方の property が増えても
/// ここで並べ直さない。
#[derive(Clone)]
pub struct Layer {
    pub texture: GpuTexture2D,
    /// 素材の実寸(ピクセル)。板のローカル矩形は `(0,0)-(size)` で、
    /// そこへ `placement.transform` を掛けて comp 座標の四角形にする。
    pub size: [f32; 2],
    pub placement: LayerPlacement,
    /// `LayerAttrs::pinned`(裁定113)。true ならカメラ(center/zoom/roll)を一切受けず
    /// 画面に張り付く。実装は「z=0 平面でのカメラの写像の逆行列」を層の transform に
    /// 掛けてから同じ透視カメラへ渡す形(`motolii_core::camera_screen_from_world_z0`)
    /// — 2パス目を増やさない。
    pub pinned: bool,
    /// `LayerAttrs::blend_mode` のうち、この合成器が表現できる分だけ
    /// (モジュール doc 参照)。既定 `Normal`。
    pub blend_mode: BlendMode,
}

/// [`Layer`] + そこへ掛ける GPU pass 列(裁定153 S2)。
///
/// **`Layer` 自体は無改造**——`motolii-engine` は今も裸の `Layer` を組み立てて
/// [`Compositor::render`]/[`Compositor::render_with_timing`] へ渡しており(並走レーン、
/// この crate の外)、その2つの入口とシグネチャ・挙動を一切変えないことでそちらを
/// 壊さない。effect を持つ layer はこの型を経由する新しい入口
/// [`Compositor::render_with_effects`] を使う。
///
/// `passes` が空なら[`Compositor::render_with_effects`] は**その layer についてだけ**
/// オフスクリーンを作らず元の texture をそのまま使う——[`Compositor::render`] と
/// 完全に同じ経路(`tests/effects.rs` の
/// `passless_layer_matches_the_traditional_render_path` が縛る)。
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
}

/// 1フレームの内訳。どこで時間を使っているかを隠さない。
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderTiming {
    /// 板の組み立てと view の用意。
    pub build_us: u128,
    /// GPU への提出と完了待ち。
    pub gpu_us: u128,
    /// GPU → CPU の読み戻し。**preview には本来要らない**。
    pub readback_us: u128,
}

impl RenderTiming {
    pub fn total_us(&self) -> u128 {
        self.build_us + self.gpu_us + self.readback_us
    }
}

pub struct Compositor {
    ctx: RenderContext,
    /// 読み戻しの識別子。1つの Compositor で連番にする。
    next_readback: u64,
    /// [`Compositor::render_with_effects`]/[`Compositor::render_sequential`] が
    /// texture_manager_2d へ import する時の cache key。`next_readback` とは別の
    /// keyspace(`import_gpu_premultiplied` の `key` は screenshot の識別子と無関係)。
    next_effect_key: u64,
    /// layer 単位オフスクリーンパスの中間 texture プール(裁定153 S2)。
    /// **Compositor 所有・フレームをまたいで再利用** — `effects` モジュール doc 参照。
    effect_scratch: effects::EffectScratch,
    /// Glow の shader pipeline(裁定153 S4)。**初回生成して以後使い回す**
    /// (`effects::glow` モジュール doc 参照)。他の pass 種別が増えても
    /// pipeline はサイズ非依存なので、layer やフレームをまたいで作り直さない。
    glow_pipelines: effects::GlowPipelines,
}

impl Compositor {
    /// 窓を持たない GPU の上に合成器を建てる。**export も preview もこれ1つ**。
    ///
    /// 実体は [`Self::with_device`] の薄いラッパー——adapter/device/queue を
    /// [`headless::HeadlessGpu`] で自前に建てて渡すだけで、渡す format/config は
    /// 従来のまま(挙動は不変、`tests/with_device.rs` がバイト一致で縛る)。
    pub fn headless() -> Result<Self, CompositorError> {
        let gpu = headless::HeadlessGpu::new()?;
        Self::with_device(
            &gpu.adapter,
            gpu.device,
            gpu.queue,
            // 読み戻し形式に合わせる。窓へ出す時はここが surface の形式になる。
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            // **MSAA は切る**。layer は軸に沿った板であって、アンチエイリアスすべき
            // 幾何エッジを持たない。上流の `MsaaMode::Off` の doc も「device 差が出にくい」
            // と言っているので、決定性の点でもこちらが良い。
            // ただし **速度の理由ではない**: R1 実測で 1080p 40枚は 41.6ms → 37.8ms
            // (9%)しか変わらなかった。律速は fragment の量そのものである。
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
    }

    /// 外部から与えられた GPU device/queue の上に合成器を組む第二コンストラクタ
    /// (裁定170 M2、2026-08-21)。**まだ誰も呼ばない** — iced 側の配線はここでは
    /// やらない(配線ゼロ = 挙動ゼロ変更、browser B0 骨格と同じ手口)。
    ///
    /// [`Self::headless`] が「device を建てた後」にやっていた共通部分をここへ
    /// 抽出しただけで、`headless()` 自身の挙動は一切変えていない
    /// (`tests/with_device.rs` の `with_device_matches_headless` が
    /// バイト一致で縛る)。
    ///
    /// `adapter` を今も要求するのは、現行 fork rev(`Cargo.toml` の `[patch]` 参照)の
    /// `RenderContext::new` がまだ `&wgpu::Adapter` を引数に取るため——fork へ
    /// `new_from_device`(adapter 不要版)が入るのは M3 の仕事で、ここでは先取りしない。
    pub fn with_device(
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format: wgpu::TextureFormat,
        config_provider: impl FnOnce(&re_renderer::device_caps::DeviceCaps) -> re_renderer::RenderConfig,
    ) -> Result<Self, CompositorError> {
        let ctx = RenderContext::new(adapter, device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;

        let glow_pipelines = effects::GlowPipelines::new(&ctx.device);

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            glow_pipelines,
        })
    }

    /// premultiplied RGBA8 を GPU へ載せる。
    pub fn upload_rgba(
        &self,
        label: &str,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: rgba.to_vec().into(),
                    format: wgpu::TextureFormat::Rgba8Unorm.into(),
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// デコード直後の YUV420p(planar)をそのまま GPU へ載せる。
    ///
    /// **色変換は上流(`re_renderer`)がやる**(裁定23)。ffmpeg の
    /// `-f rawvideo -pix_fmt yuv420p` が吐くバイト列は、上流の `Y_U_V420` が期待する
    /// `width × (height + height/2)` の R8 連番そのものなので、詰め替えが要らない。
    /// 自前で WGSL を書くと「色事故を防ぐために自分で変換する」動機ごと二重になる。
    pub fn upload_yuv420p(
        &self,
        label: &str,
        data: &[u8],
        width: u32,
        height: u32,
        color: motolii_core::ColorSpace,
    ) -> Result<GpuTexture2D, CompositorError> {
        use re_renderer::resource_managers::{SourceImageDataFormat, YuvMatrixCoefficients, YuvPixelLayout, YuvRange};

        let (coefficients, range) = match color {
            motolii_core::ColorSpace::Rec709Limited => {
                (YuvMatrixCoefficients::Bt709, YuvRange::Limited)
            }
            motolii_core::ColorSpace::Rec709Full => (YuvMatrixCoefficients::Bt709, YuvRange::Full),
            motolii_core::ColorSpace::Rec601Limited => {
                (YuvMatrixCoefficients::Bt601, YuvRange::Limited)
            }
            // RGB 系の色空間で YUV を載せようとしている = 呼び手の取り違え。
            other => {
                return Err(CompositorError::Rectangles(format!(
                    "YUV420p に RGB 系の色空間が渡された: {other:?}"
                )))
            }
        };

        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: data.to_vec().into(),
                    format: SourceImageDataFormat::Yuv {
                        layout: YuvPixelLayout::Y_U_V420,
                        coefficients,
                        range,
                    },
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::Opaque,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// **唯一の評価経路**。RGBA8(premultiplied)を返す。
    ///
    /// preview はこの結果を窓へ出し、export は同じ結果を mux へ渡す。
    pub fn render(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        self.render_with_timing(comp, camera, layers)
            .map(|(frame, _)| frame)
    }

    /// 内訳つき。**どこが遅いかを隠さない**ための口で、製品経路は [`Self::render`]。
    pub fn render_with_timing(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<(Vec<u8>, RenderTiming), CompositorError> {
        let mut timing = RenderTiming::default();
        let build_start = std::time::Instant::now();

        // **投影の正本は `motolii-core::camera`**。ここでは組み立てず、そこが返す
        // 値をそのまま `macaw`/`re_renderer` の型へ詰め替えるだけ。
        let projection = motolii_core::camera_projection(comp, camera);
        // pinned layer(裁定113)用: z=0 平面でのカメラの写像の逆行列。層の transform に
        // 前もって掛けておけば、この後カメラを通しても打ち消し合って画面上不動になる。
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();

        let rects: Vec<TexturedRect> = layers
            .iter()
            .map(|layer| {
                let (transform, z) = if layer.pinned {
                    (pinned_cancel * layer.placement.transform, 0.0)
                } else {
                    (layer.placement.transform, layer.placement.z)
                };
                TexturedRect {
                    // **affine のまま板にする**。`TexturedRect` は左上と2本の辺ベクトルで
                    // 四角形を表すので、変換後の基底ベクトルをそのまま渡せば
                    // 回転も拡大も skew も**シェーダを1行も変えずに**通る。
                    top_left_corner_position: to_point3(
                        transform.transform_point2(glam::Vec2::ZERO),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])),
                    ),
                    colormapped_texture:
                        re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                            layer.texture.clone(),
                        ),
                    options: RectangleOptions {
                        // premultiplied なので rgb は opacity で揃えて掛ける。alpha だけ
                        // blend mode で分かれる: Normal は opacity と同じ(通常の
                        // premultiplied alpha-over)。Add は 0(module doc の式変形どおり、
                        // `out = 1×src + (1-src.a)×dst` の `src.a` を 0 にすると
                        // `out = src + dst` になる — 加算合成そのもの、alpha は不変)。
                        multiplicative_tint: {
                            let a = match layer.blend_mode {
                                BlendMode::Normal => layer.placement.opacity,
                                BlendMode::Add => 0.0,
                            };
                            Rgba::from_rgba_premultiplied(
                                layer.placement.opacity,
                                layer.placement.opacity,
                                layer.placement.opacity,
                                a,
                            )
                        },
                        depth_offset: layer.placement.order,
                        ..Default::default()
                    },
                }
            })
            .collect();

        self.ctx.begin_frame();

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        // `rotation`/`eye` は `motolii-core::CameraProjection` が返す形(world → view の
        // 回転 + カメラ位置)。`macaw::IsoTransform::transform_point3` は
        // `rotation*p + translation` を計算するので、`translation = -(rotation*eye)`
        // にすれば `view = rotation*(p - eye)` になる。
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp".into(),
                // 「同じ絵が出る」ことが preview=export の前提なので beauty より決定性。
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [comp.width, comp.height],
                view_from_world,
                projection_from_view: Projection::Perspective {
                    vertical_fov: projection.vertical_fov_radians,
                    near_plane_distance: projection.near_plane_distance,
                    aspect_ratio: projection.aspect_ratio,
                },
                pixels_per_point: 1.0,
                // 既定 `No` は composite shader が `color = vec4f(color.rgb, 1.0)` へ
                // 強制する(上流 `composite.wgsl` 一次確認)ので、readback の alpha が
                // 常に 255 へ潰れる。`Premultiplied` は `color = vec4f(color.rgb, color.a)`
                // の素通し分岐 — 我々の layer は premultiplied alpha で描いているので
                // 意味が合う。`CompositingScreenshot` フェーズも同じ `CompositorDrawData`
                // (同じ uniform)を使う(`ViewBuilder::new` が一度だけ作って両フェーズへ
                // queue する)ので、screenshot 読み戻しにもこの分岐がそのまま効く
                // — fork 改造なしで alpha が生きることを `alpha_survives_the_composite_step`
                // (tests/compose.rs)で実測済み(2026-08-20)。
                blend_with_background: BlendWithBackground::Premultiplied,
                ..Default::default()
            },
            re_renderer::ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;

        view_builder.queue_draw(&self.ctx, draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.build_us = build_start.elapsed().as_micros();

        let gpu_start = std::time::Instant::now();

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);

        // 読み戻しの段取りは上流の frame 進行に埋まっている(`RenderContext::begin_frame`
        // が `GpuReadbackBelt::after_queue_submit`(map 開始)→ `begin_frame`(受け取り)
        // の順に呼ぶ)。窓のある側は次フレームで受け取るが、ここは窓が無いので
        // **同じ呼び出しの中でフレームを2回進めて**受け取る。
        //   1回目: map_async を開始する
        //   poll : map の完了と提出済み作業の完了を待つ
        //   2回目: receive_chunks が届いた chunk を拾う
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.gpu_us = gpu_start.elapsed().as_micros();

        let readback_start = std::time::Instant::now();
        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        re_renderer::ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        let frame = out.ok_or(CompositorError::ReadbackMissing)?;
        timing.readback_us = readback_start.elapsed().as_micros();
        Ok((frame, timing))
    }

    /// **layer 単位オフスクリーンパスの入口**(裁定153 S2、2026-08-21)。
    ///
    /// [`Self::render`]/[`Self::render_with_timing`] は**無改造のまま** — `motolii-engine`
    /// は今もそちらへ裸の `Layer` を渡しており(並走レーン、この crate の外)、
    /// この関数を新設するだけならその経路を一切変えない。effect を持たせたい呼び手は
    /// [`Layer`] を [`LayerWithPasses`] で包んでここへ渡す。
    ///
    /// **分岐はここ、layer 1枚ごと**: `passes` が空なら元の `layer.texture` を
    /// そのまま合成へ渡す(オフスクリーンを一切作らない — コスト増ゼロ)。非空なら
    /// [`effects::EffectScratch`] から中間 texture を借り、`passes` を順に適用してから
    /// その結果を合成へ渡す。texture(と、将来 pipeline が増えた時のそれ)は
    /// `Compositor` が所有し**フレームをまたいで再利用**する(毎フレーム作り直さない
    /// — `effects` モジュール doc の M5 proof 参照)。
    ///
    /// **第二 render パス禁止(裁定15/18)との関係**: `Compositor::render`/`render_frame`
    /// の呼び出し回数はこの関数を通っても増えない。増えるのは同じ `RenderContext`・
    /// 同じ `queue.submit` 呼び出しへ同乗する追加の `copy_texture_to_texture` コマンドで
    /// あって、別の合成器や別の描画エントリではない — `render_frame_without_background`
    /// (裁定141)が「第二経路ではなく同一合成器への入力差分」と整理したのと同じ論法。
    pub fn render_with_effects(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();

        // 1) layer ごとに「合成へ渡す実効 texture」を決める。pass が空な layer は
        //    `GpuTexture2D::clone()`(Arc clone 相当)だけで、新規 GPU texture を作らない。
        let mut effective_textures: Vec<GpuTexture2D> = Vec::with_capacity(layers.len());
        // **既知の穴の根治**(`next/reference/KNOWN.md`「effect pass は layer 自身の
        // テクスチャ境界内のみで計算」): layer ごとの出力拡張量(texel、上下左右均等、
        // `EffectPass::padding` 参照)。pass が空 or 全 pass が padding 0(Identity の
        // み)なら 0 のまま——2) の quad 組み立てはこの値が 0 だと従来と完全に同じ
        // 幾何になる。
        let mut effective_paddings: Vec<u32> = Vec::with_capacity(layers.len());
        // GPU が読み終わってからプールへ返すための控え(読み終わり前に返すと、次の
        // `acquire` が使用中の texture を上書きしてしまう)。
        let mut checked_out: Vec<(u32, u32, wgpu::TextureFormat, wgpu::Texture)> = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
            if lwp.passes.is_empty() {
                effective_textures.push(lwp.layer.texture.clone());
                effective_paddings.push(0);
                continue;
            }

            let [width, height] = lwp.layer.texture.width_height();
            // layer 単位で1つの padded canvas サイズへ揃える(複数 pass が居ても
            // scratch は1枚——既存の「各 pass は常に元の layer.texture を読んで
            // scratch へ書く」構造(下のループ)はそのまま、canvas サイズだけ最大の
            // 要求へ合わせる)。
            let padding = lwp
                .passes
                .iter()
                .map(EffectPass::padding)
                .max()
                .unwrap_or(0);
            let padded_width = width + 2 * padding;
            let padded_height = height + 2 * padding;

            // Glow を含む layer は中間・出力とも常に `Rgba16Float`(裁定153 S4、
            // `effects::glow` モジュール doc 参照 — bright-pass の閾値判定と加算合成が
            // 1.0 を超える値を扱う必要があるため、layer 本体の元 format とは独立)。
            // Identity だけの layer は従来どおり元 format のまま(既存試験の期待を壊さない)。
            let has_glow = lwp
                .passes
                .iter()
                .any(|pass| matches!(pass, EffectPass::Glow { .. }));
            let format = if has_glow {
                effects::GLOW_INTERMEDIATE_FORMAT
            } else {
                lwp.layer.texture.format()
            };

            let src_handle = lwp.layer.texture.handle();
            let src = self
                .ctx
                .gpu_resources
                .textures
                .get_from_handle(src_handle)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            let scratch =
                self.effect_scratch
                    .acquire(&self.ctx.device, padded_width, padded_height, format);

            let encoder = copy_encoder.get_or_insert_with(|| {
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("motolii-compositor-effect-pass"),
                    })
            });

            for pass in &lwp.passes {
                match pass {
                    // 恒等 pass。画素単位の copy がそのまま「絵を変えない」を満たす。
                    // padding は常に 0(`EffectPass::padding` 参照)なので
                    // `padded_width == width` — 中央 (padding, padding) へ置いても
                    // 実質 (0, 0) のままで従来と同じ絵になる(同じ layer に padding>0
                    // の別 pass が同居する場合に備えて、座標系を1本化しておく)。
                    EffectPass::Identity => {
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &scratch,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                    // 内蔵 vism 第1号(裁定153 S4)。bright-pass→水平blur→垂直blur→
                    // 加算合成の5パスを同じ encoder へ積む(`effects::glow` 参照)。
                    //
                    // **padding(既知の穴の根治)**: source をそのまま bright-pass へ
                    // 渡さず、まず「layer 実寸+両側 padding」の透明な padded canvas
                    // (`padded_source`)を用意し、実 layer を中央 (padding, padding)
                    // へ copy してから5パスを回す。`blur_at`(glow.rs の WGSL)の
                    // clamp は `textureDimensions(input_a)`(=呼ばれた pass の入力
                    // texture の実サイズ)由来なので、bright/blur/composite 全パスの
                    // 入出力を padded canvas サイズに揃えれば、clamp が padded canvas
                    // の縁で効くようになる——layer 実寸の縁で足踏みしていた従来の
                    // 穴が無くなり、blur が実際に周囲の透明領域へ滲み出す。
                    EffectPass::Glow {
                        threshold,
                        intensity,
                        radius,
                    } => {
                        let padded_source = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            lwp.layer.texture.format(),
                        );
                        let padded_source_view = padded_source.create_view(&Default::default());
                        // padding 領域を透明へ clear する——scratch プールの
                        // 使い回しで前フレームの残骸を引きずると、`blur_at` の
                        // clamp が縁で拾う値が汚れる(draw なしの render pass、
                        // `draw_pass` の clear と同じ書き方)。
                        {
                            let _clear_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("motolii-compositor-glow-padded-source-clear"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &padded_source_view,
                                        depth_slice: None,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    timestamp_writes: None,
                                    occlusion_query_set: None,
                                    multiview_mask: None,
                                });
                        }
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &src.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &padded_source,
                                mip_level: 0,
                                origin: wgpu::Origin3d {
                                    x: padding,
                                    y: padding,
                                    z: 0,
                                },
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                        );

                        let bloom = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let blur_ping = self.effect_scratch.acquire(
                            &self.ctx.device,
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                        );
                        let bloom_view = bloom.create_view(&Default::default());
                        let blur_ping_view = blur_ping.create_view(&Default::default());
                        let dst_view = scratch.create_view(&Default::default());

                        self.glow_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            encoder,
                            &padded_source_view,
                            &bloom_view,
                            &blur_ping_view,
                            &dst_view,
                            *threshold,
                            *intensity,
                            *radius,
                        );

                        checked_out.push((
                            padded_width,
                            padded_height,
                            lwp.layer.texture.format(),
                            padded_source,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            bloom,
                        ));
                        checked_out.push((
                            padded_width,
                            padded_height,
                            effects::GLOW_INTERMEDIATE_FORMAT,
                            blur_ping,
                        ));
                    }
                }
            }

            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &scratch)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;

            effective_textures.push(imported);
            effective_paddings.push(padding);
            checked_out.push((padded_width, padded_height, format, scratch));
        }

        // 2) 通常合成。組み立ては `render_with_timing` と同型 — 使う texture だけが
        //    「元の layer.texture」から「上で決めた実効 texture」に変わる。
        let rects: Vec<TexturedRect> = layers
            .iter()
            .zip(effective_textures.iter())
            .zip(effective_paddings.iter())
            .map(|((lwp, texture), &padding)| {
                let layer = &lwp.layer;
                let (transform, z) = if layer.pinned {
                    (pinned_cancel * layer.placement.transform, 0.0)
                } else {
                    (layer.placement.transform, layer.placement.z)
                };
                // **既知の穴の根治**: pass が出力を拡張した分(`padding`、texel、
                // `EffectPass::padding` 参照)だけ quad を local 空間で広げる——
                // `LayerPlacement::transform` 自体は一切変えず、この rect の
                // 組み立てだけが「実 texture が layer 実寸より大きい」事実を吸収
                // する(transform は affine なので、広げた local 矩形にそのまま
                // 掛ければ回転/拡大/skew があっても正しく追従する)。padding=0
                // (pass 無し/Identity のみ)なら local_min=(0,0)・local_size=
                // layer.size のまま、従来と完全に同じ幾何になる。
                let pad = padding as f32;
                let local_min = glam::Vec2::new(-pad, -pad);
                let local_size =
                    glam::Vec2::new(layer.size[0] + 2.0 * pad, layer.size[1] + 2.0 * pad);
                TexturedRect {
                    top_left_corner_position: to_point3(transform.transform_point2(local_min), z),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(local_size.x, 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, local_size.y)),
                    ),
                    colormapped_texture:
                        re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                            texture.clone(),
                        ),
                    options: RectangleOptions {
                        multiplicative_tint: {
                            let a = match layer.blend_mode {
                                BlendMode::Normal => layer.placement.opacity,
                                BlendMode::Add => 0.0,
                            };
                            Rgba::from_rgba_premultiplied(
                                layer.placement.opacity,
                                layer.placement.opacity,
                                layer.placement.opacity,
                                a,
                            )
                        },
                        depth_offset: layer.placement.order,
                        ..Default::default()
                    },
                }
            })
            .collect();

        self.ctx.begin_frame();

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp".into(),
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
            },
            re_renderer::ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;

        view_builder.queue_draw(&self.ctx, draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        // effect pass の copy(あれば)を最終合成と**同じ submit 呼び出し**に同乗させる。
        // 同一キューへの提出はこの順で実行される(裁定15/18: 第二 render パス禁止は
        // `Compositor::render` の呼び出し回数の話であって、同一 RenderContext 内の
        // 追加コマンドバッファは対象外)。
        match copy_encoder {
            Some(encoder) => {
                self.ctx.queue.submit([encoder.finish(), command_buffer]);
            }
            None => {
                self.ctx.queue.submit([command_buffer]);
            }
        }

        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        re_renderer::ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        let frame = out.ok_or(CompositorError::ReadbackMissing)?;

        // GPU が読み終わった後なので、scratch をプールへ返して次フレームで使い回す
        // (毎フレーム作り直さない)。
        for (width, height, format, texture) in checked_out {
            self.effect_scratch.release(width, height, format, texture);
        }

        Ok(frame)
    }

    /// 試験専用の introspection。`effect_scratch` が実際に**新規生成**した
    /// (プール再利用ではない)texture の総数。`RenderTiming` が時間の内訳を
    /// 隠さないのと同じ規律で、資源生成も隠さない。
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    /// **逐次合成(accumulator)経路**(裁定161 BL1b、2026-08-21)。
    ///
    /// BL1(裁定160)は「layer を1枚ずつ焼き込んでも一括描画と同じ絵が出るか」を
    /// **公開 API** [`ViewBuilder::composite`] だけで検証し、**赤**(半透明の重なりで
    /// バイト不一致)という結果と、その理由(fork の `composite()` は
    /// non-srgb-tagged 固定 format にしか描けず、本来1回で済む
    /// 「unmultiply→ガンマ encode→re-multiply」の変換を layer 毎= N 回踏んでしまう)
    /// を実測で固定した(`docs/reviews/2026-08-21-blend-fork-accessor-decision.md`)。
    ///
    /// 裁定161 は「fork へ read アクセサを1本足す」を選んだ
    /// ([`ViewBuilder::main_target`]、fork 側 pinned rev には未反映——この crate の
    /// `Cargo.toml` の `[patch]` section 参照)。この関数はその境界を実際に越える。
    ///
    /// ## 仕組み(実装は3段)
    ///
    /// layer i を描くたび、新しい [`ViewBuilder`] へ**2枚**の [`TexturedRect`] を
    /// 積む:
    ///
    /// 1. **背景 rect**(`depth_offset = i16::MIN`、最背面): 直前までの逐次合成結果
    ///    (前の layer の main_target、[`ViewBuilder::main_target`] で取得)を、
    ///    そのまま画面いっぱいに敷く。`ColormappedTexture` は手組みで
    ///    `decode_srgb: false`(main_target は既に `Rgba8UnormSrgb` — GPU が読み込み時に
    ///    自動 decode するので、ここで**また** decode すると二重補正になる)・
    ///    `texture_alpha: TextureAlpha::AlreadyPremultiplied`(accumulator は既に
    ///    premultiplied——`from_unorm_rgba` の既定 `SeparateAlpha` は使えない)にする。
    ///    位置は「画面に張り付く板」と同じ変換(`pinned_cancel`、pinned layer が
    ///    既に使っている値をそのまま流用——新しい数式を持ち込まない)。
    /// 2. **layer 自身の rect**(既存の `render_with_timing` と同型)を、その上に
    ///    通常の premultiplied-over パイプラインで重ねる。
    ///
    /// この2枚は**同じ ViewBuilder・同じ main_target の中**で描かれるので、
    /// GPU の blend(`LoadOp::Clear` → 背景を書く → layer を書く、2回とも
    /// srgb-tagged な main_target への自動 decode/encode)は、一括経路
    /// (`render_with_timing`)が N layer を1つの main_target の中で順に混ぜる時と
    /// **layer あたりの quantize 回数が同じ**になる——「背景を敷く」書き込みは
    /// 直前の main_target から dequantize した値をそのまま re-quantize するだけ
    /// (8bit sRGB の decode/encode は往復で可逆、値は変わらない)なので、実質
    /// タダで「前回までの答え」を持ち込める。`composite.wgsl` のガンマ変換は
    /// **一切踏まない**(BL1 が赤だった原因そのものを避ける経路)。
    ///
    /// **layer 自身の main_target を次の背景として持ち越す時の罠**: fork の
    /// `GpuTexturePool::begin_frame`(一次確認: `wgpu_resources/texture_pool.rs`)は、
    /// 通常(import ではない)texture の参照が尽きると reclaim 時に
    /// `res.texture.destroy()` を**明示的に**呼ぶ——`ViewBuilder` を drop した後の
    /// `begin_frame()` サイクルで、`texture_manager_2d.import_gpu_premultiplied` 越しに
    /// 持っている側から見ても実体が壊れる(import は「別の `wgpu::Texture` clone」を
    /// 作るだけで、元の pool エントリの生存とは独立に守ってくれない——doc 参照:
    /// `TextureManager2D::import_gpu_premultiplied` は「embedder は書き込み中に破棄
    /// してはいけない」とは言うが、reclaim 側の `destroy()` は防いでくれない)。
    /// そのため `view_builder.main_target().clone()` で `GpuTexture`(Arc)を
    /// **明示的に握り続け**(`background` タプルの `.0`)、次の layer の背景として
    /// 使い終わる(= 次の submit を poll で待ち終える)まで手放さない。
    ///
    /// 3. **最終変換は1回だけ**、既存の screenshot 経路
    ///    ([`ViewBuilder::schedule_screenshot`]/[`ScreenshotProcessor`])を
    ///    そのまま再利用する。全 layer を重ね終えた最後の main_target を、もう一度
    ///    「背景 rect だけの ViewBuilder」として描き、その screenshot 読み戻しで
    ///    `composite.wgsl` の unmultiply→ガンマ encode→re-multiply を**1回だけ**
    ///    通す——[`Self::render`]/[`Self::render_with_timing`] が既にやっているのと
    ///    **同じコード経路**を呼ぶだけで、この crate 側に新しい WGSL は1行も無い。
    ///
    /// **なぜこれでバイト一致するか**: 一括経路も逐次経路も、同じ順序で同じ
    /// premultiplied-alpha over を同じ GPU の同じ srgb decode/encode 精度
    /// (8bit 量子化込み)で行い、**最後に1回だけ**同じガンマ式を通す——
    /// パスの境界(1つの render pass の中か・複数の submit にまたがるか)は
    /// blend 演算そのものの数値には影響しない。`tests/sequential.rs` の overlap
    /// fixture がこれをバイト一致で縛る(ORACLE、裁定161 受入条件)。
    ///
    /// **実装中に見つかった罠(背景 rect のフィルタ)**: `background_rect` は
    /// `texture_filter_{magnification,minification}` を**両方明示的に `Nearest`**
    /// にする必要がある(`RectangleOptions::default()` の既定は minification が
    /// `Linear`)。`pinned_cancel` は既定カメラでも機械精度の恒等ではなく
    /// (`motolii-core::camera` の試験が eps=1e-2 で縛る程度の近似)、この
    /// full-canvas 矩形のスクリーン空間微分(`fwidth`)がちょうど 1.0 の際どい所を
    /// 振れることがあり、既定の Linear minification へ落ちると直前までの合成結果
    /// (内部に layer の縁を持つ)がバイリニアで滲んで layer の縁が最大 Δ29 ずれた
    /// (実測、`background_rect` の doc 参照)。Nearest 固定で解消し、この関数の
    /// バイト一致はそこに依存している。
    ///
    /// [`Self::render`]/[`Self::render_with_timing`]/[`Self::render_with_effects`] は
    /// **無改造のまま**——この関数は並設した入口で、既存3つの呼び出し元へ
    /// 一切波及しない。blend 式はまだ [`BlendMode::Normal`] 固定(式の分岐は
    /// BL3/BL4、NON-GOALS)。
    pub fn render_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        // 直前までの逐次合成結果。`.0` は fork pool の reclaim/destroy から実体を守る
        // ための保持、`.1` はそれを rect の texture として使うための import(module doc
        // 「罠」の節参照)。空 comp・1枚目の layer では背景 rect 自体を省く。
        let mut background: Option<(GpuTexture, GpuTexture2D)> = None;

        for layer in layers {
            self.ctx.begin_frame();

            let (transform, z) = if layer.pinned {
                (pinned_cancel * layer.placement.transform, 0.0)
            } else {
                (layer.placement.transform, layer.placement.z)
            };

            let mut rects: Vec<TexturedRect> = Vec::with_capacity(2);
            if let Some((_, imported)) = &background {
                rects.push(background_rect(comp, pinned_cancel, imported.clone()));
            }
            rects.push(TexturedRect {
                top_left_corner_position: to_point3(
                    transform.transform_point2(glam::Vec2::ZERO),
                    z,
                ),
                extent_u: to_vector3(
                    transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)),
                ),
                extent_v: to_vector3(
                    transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])),
                ),
                colormapped_texture: re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                    layer.texture.clone(),
                ),
                options: RectangleOptions {
                    multiplicative_tint: {
                        let a = match layer.blend_mode {
                            BlendMode::Normal => layer.placement.opacity,
                            BlendMode::Add => 0.0,
                        };
                        Rgba::from_rgba_premultiplied(
                            layer.placement.opacity,
                            layer.placement.opacity,
                            layer.placement.opacity,
                            a,
                        )
                    },
                    depth_offset: layer.placement.order,
                    ..Default::default()
                },
            });

            let draw_data = RectangleDrawData::new(&self.ctx, &rects)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

            let mut view_builder = ViewBuilder::new(
                &self.ctx,
                TargetConfiguration {
                    name: "motolii-comp-sequential-layer".into(),
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
                },
                ViewBuilderId::new(self.next_readback),
            )
            .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;

            view_builder.queue_draw(&self.ctx, draw_data);
            let command_buffer = view_builder
                .draw(&self.ctx, Rgba::TRANSPARENT)
                .map_err(|e| CompositorError::Draw(e.to_string()))?;

            self.ctx.before_submit();
            self.ctx.queue.submit([command_buffer]);
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;

            // **fork accessor 経由**(裁定161): 次の layer(または最終変換)の背景と
            // して使えるよう、この main_target を握り続けたまま import する
            // (module doc「罠」の節参照)。古い `background`(前回分)はここで
            // drop されるが、直前の poll でその GPU 読み取りは完了済みなので安全。
            let held: GpuTexture = view_builder.main_target().clone();
            self.next_effect_key += 1;
            let key = self.next_effect_key;
            let imported = self
                .ctx
                .texture_manager_2d
                .import_gpu_premultiplied(key, &self.ctx, &held.texture)
                .map_err(|e| CompositorError::Effect(e.to_string()))?;
            background = Some((held, imported));
        }

        // **最終変換は1回だけ**(module doc 3番)。既存の screenshot 経路を再利用する
        // ——fork の `composite.wgsl` は無改造のまま、`render()`/`render_with_timing`
        // と**同じ**コードで同じガンマ round-trip を1回だけ通す。
        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            final_rects.push(background_rect(comp, pinned_cancel, imported.clone()));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp-sequential-finalize".into(),
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
            },
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        final_view_builder.queue_draw(&self.ctx, final_draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        final_view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = final_view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);

        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        ScreenshotProcessor::next_readback_result::<()>(&self.ctx, identifier, |data, _extent, ()| {
            out = Some(data.to_vec());
        });

        out.ok_or(CompositorError::ReadbackMissing)
    }
}

/// [`Compositor::render_sequential`] の「背景 rect」(module doc 参照): 直前までの
/// 逐次合成結果を、画面に張り付く板として画面いっぱいに敷く。`pinned` layer が
/// 使っているのと**同じ** `pinned_cancel` 変換を、full-canvas な矩形
/// (`(0,0)-(comp.width,comp.height)`)に適用するだけ——新しい幾何は無い。
fn background_rect(
    comp: CompSpec,
    pinned_cancel: glam::Affine2,
    imported: GpuTexture2D,
) -> TexturedRect {
    TexturedRect {
        top_left_corner_position: to_point3(pinned_cancel.transform_point2(glam::Vec2::ZERO), 0.0),
        extent_u: to_vector3(
            pinned_cancel.transform_vector2(glam::Vec2::new(comp.width as f32, 0.0)),
        ),
        extent_v: to_vector3(
            pinned_cancel.transform_vector2(glam::Vec2::new(0.0, comp.height as f32)),
        ),
        colormapped_texture: ColormappedTexture {
            texture: imported,
            range: [0.0, 1.0],
            // main_target は既に `Rgba8UnormSrgb`——GPU が読み込み時に自動 decode する。
            // ここで software decode も重ねると二重補正になる(module doc 参照)。
            decode_srgb: false,
            // accumulator は既に premultiplied alpha——`from_unorm_rgba` の既定
            // `SeparateAlpha`(もう一度 alpha を掛ける)は使えない。
            texture_alpha: TextureAlpha::AlreadyPremultiplied,
            gamma: 1.0,
            color_mapper: ColorMapper::OffRGB,
            shader_decoding: None,
        },
        options: RectangleOptions {
            // 素通し(tint なし)。
            multiplicative_tint: Rgba::from_rgba_premultiplied(1.0, 1.0, 1.0, 1.0),
            // 常に最背面——同じ z(全て 0)の他 rect より必ず先に描かれる
            // (`draw_phase_manager.rs` の transparent sort: 同じ distance では
            // `depth_offset` 昇順、`i16::MIN` は他のどの layer の `order` よりも
            // 小さい)。
            depth_offset: i16::MIN,
            // **両方 Nearest が必須**(実測で発見、`RectangleOptions::default()` は
            // magnification=Nearest だが minification=Linear)。`pinned_cancel` は
            // 既定カメラでも機械精度の恒等ではない(`motolii-core::camera` のテストが
            // eps=1e-2 で縛っている程度の近似)ので、この full-canvas 矩形の
            // スクリーン空間微分(`fwidth`)が 1.0 のごく僅か下に振れることがあり、
            // `is_magnifying()` が稀に「縮小」側と判定して既定の Linear minification
            // フィルタへ落ちる。すると直前までの合成結果(base+overlay の縁のような
            // 内部エッジを持つ)がバイリニアで滲み、layer の縁が数 LSB〜Δ29 ずれる
            // (実測: `debug_two_layer_grid` probe で overlay 左上角に集中して再現、
            // Nearest/Nearest に固定した瞬間に消えた)。この矩形は「直前までの答えを
            // そのまま複製する」だけの内部機構であって、ユーザー可視な layer の
            // フィルタ方針(`render_with_timing` の既定のまま)とは別物なので、
            // ここだけ明示的に固定する——layer 自身の rect(下のループ)は
            // `render_with_timing` と同じ既定のまま変更しない。
            texture_filter_magnification: re_renderer::renderer::TextureFilterMag::Nearest,
            texture_filter_minification: re_renderer::renderer::TextureFilterMin::Nearest,
            ..Default::default()
        },
    }
}
