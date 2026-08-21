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
//! ## 分離可能(separable)blend 11 モード(BL3、2026-08-22)
//!
//! Multiply/Screen/Overlay/Darken/Lighten/ColorDodge/ColorBurn/HardLight/SoftLight/
//! Difference/Exclusion は**固定式の係数を振るだけでは表現できない**(Multiply は
//! `dst_factor` に src の色を使う必要があり、Screen 以降は非線形で dst を shader 内で
//! 読む必要がある — どちらも `RectangleOptions` の外)。[`blend`] サブモジュールが
//! 新規 WGSL パイプライン(`effects::glow` と同じ「fork を触らず crate 内へ足す」手口、
//! 裁定161 の main_target アクセサ経由で dst を読む)を1本持ち、[`Compositor::render_sequential`]/
//! [`Compositor::render_with_effects`]/[`Compositor::render_to_texture`] がそれを使う
//! ([`accumulate_sequential`] 参照)。数式の出典・gamma の扱いは `blend` モジュール doc。
//!
//! 非分離4種(Hue/Saturation/Color/Luminosity)は依然未実装(BL4、`motolii-engine` 側が
//! 明示的に弾く)。
//!
//! **[`Compositor::render`]/[`Compositor::render_with_timing`] は分離可能 blend を
//! 実装しない** — この2つは「1つの ViewBuilder へ全 layer をまとめて描く」一括経路
//! ([`Self::render_with_timing`] 参照)で、dst を読む2枚読みパスが構造的に乗らない
//! (乗せるなら `render_sequential` と同じ逐次経路へ丸ごと作り替える必要があり、
//! この2つの「昔からある一括経路」を無改造に保つ既存規律に反する)。分離可能
//! blend を持つ layer をこの2つへ渡すと [`CompositorError::UnsupportedBlendMode`]
//! を返す(黙って `Normal` へ近似しない、`translate_blend_mode` と同じ fail-closed)。
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

mod blend;
mod effects;
mod headless;

/// 合成器が表現できる blend mode。Document 側の `motolii_store::BlendMode`(17値
/// — Lottie 16値 + `Add`)のうち、非分離4種(Hue/Saturation/Color/Luminosity、BL4)
/// **以外の全部**——変換と「対応外は弾く」判断は `motolii-engine` の仕事
/// (この crate は Document の語彙を知らない)。
///
/// `Normal`/`Add` は固定式(`RectangleOptions::multiplicative_tint`、モジュール doc
/// 「blend mode」節)。それ以外(Multiply〜Exclusion、11値)は2枚読みの新規パス
/// ([`blend`] サブモジュール、モジュール doc「分離可能 blend」節)——
/// [`separable_mode_index`] がどちらに属すかを判定する。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// 通常の alpha-over。`re_renderer` の transparent パイプラインの既定そのもの。
    #[default]
    Normal,
    /// 加算合成。`multiplicative_tint.a = 0` で無改造に出せる(モジュール doc 参照)。
    Add,
    /// `Cs · Cb`(W3C Compositing 3.6)。
    Multiply,
    /// `Cs + Cb − Cs·Cb`。
    Screen,
    /// `HardLight(Cb, Cs)`(backdrop/source を入れ替えた HardLight)。
    Overlay,
    /// `min(Cb, Cs)`。
    Darken,
    /// `max(Cb, Cs)`。
    Lighten,
    /// `min(1, Cb / (1 − Cs))`(境界条件は `blend` モジュール WGSL 参照)。
    ColorDodge,
    /// `1 − min(1, (1 − Cb) / Cs)`(境界条件は `blend` モジュール WGSL 参照)。
    ColorBurn,
    /// `cs<=0.5` で `2·Cs·Cb`、それ以外で `1 − 2·(1−Cs)·(1−Cb)`。
    HardLight,
    /// W3C 版(Photoshop 版とは式が異なる、`blend` モジュール doc 参照)。
    SoftLight,
    /// `|Cb − Cs|`。
    Difference,
    /// `Cs + Cb − 2·Cs·Cb`。
    Exclusion,
}

/// [`BlendMode`] が分離可能 blend([`blend`] サブモジュールの2枚読みパス)に属すなら
/// その WGSL `params.mode` index(`blend::SHADER` の `blend_channel` と1対1)を返す。
/// `Normal`/`Add`(固定式で表現できる、モジュール doc 参照)は `None`。
///
/// **`_` を使わない**(全 variant を列挙)——将来 variant が増えた時にこの対応表を
/// 更新し忘れるとコンパイルが落ちる(`translate_blend_mode` と同じ fail-closed の形)。
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
    }
}

/// [`Compositor::render`]/[`Compositor::render_with_timing`] だけが使う、固定式
/// (`RectangleOptions::multiplicative_tint`)の alpha 係数。`Normal`/`Add` のみ
/// 表現できる(モジュール doc 参照)——分離可能 blend は
/// [`CompositorError::UnsupportedBlendMode`] で明示的に拒む。
fn fixed_function_tint_alpha(mode: BlendMode, opacity: f32) -> Result<f32, CompositorError> {
    match mode {
        BlendMode::Normal => Ok(opacity),
        BlendMode::Add => Ok(0.0),
        other => Err(CompositorError::UnsupportedBlendMode(other)),
    }
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
    /// [`Compositor::render`]/[`Compositor::render_with_timing`] が分離可能 blend
    /// ([`separable_mode_index`] が `Some` を返す mode)を渡された時(モジュール doc
    /// 「分離可能 blend」節参照)。黙って `Normal` へ近似しない。
    #[error("この入口は分離可能 blend mode を表現できない: {0:?}")]
    UnsupportedBlendMode(BlendMode),
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
    /// 分離可能 blend の shader pipeline(BL3)。`glow_pipelines` と同じ規律 —
    /// 初回生成して以後使い回す(`blend` モジュール doc 参照)。
    blend_pipelines: blend::SeparableBlendPipelines,
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
    /// (裁定170 M2、2026-08-21。adapter 引数は M3 で落とした、2026-08-22)。
    /// **まだ誰も呼ばない** — iced 側の配線はここではやらない(配線ゼロ = 挙動ゼロ
    /// 変更、browser B0 骨格と同じ手口)。
    ///
    /// [`Self::headless`] が「device を建てた後」にやっていた共通部分をここへ
    /// 抽出しただけで、`headless()` 自身の挙動は一切変えていない
    /// (`tests/with_device.rs` の `with_device_matches_headless` が
    /// バイト一致で縛る)。
    ///
    /// `adapter` を要求しないのは、rerun fork(`Cargo.toml` の `[patch]` 参照、
    /// 裁定170 M3)に足した `RenderContext::new_from_device` が adapter なしで
    /// `DeviceCaps`/`AdapterInfo` を device 自身(`device.adapter_info()`)から
    /// 導けるため——ここが「adapter なしで pipeline が実際に建つ」の実体。
    pub fn with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format: wgpu::TextureFormat,
        config_provider: impl FnOnce(&re_renderer::device_caps::DeviceCaps) -> re_renderer::RenderConfig,
    ) -> Result<Self, CompositorError> {
        let ctx = RenderContext::new_from_device(device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;

        let glow_pipelines = effects::GlowPipelines::new(&ctx.device);
        let blend_pipelines = blend::SeparableBlendPipelines::new(&ctx.device);

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            glow_pipelines,
            blend_pipelines,
        })
    }

    /// [`Self::with_device`]の既定 config 版(裁定171 v2 M4、**additive** —
    /// `with_device`/[`Self::headless`]自体は無改造)。[`Self::headless`]と
    /// **同じ** `output_format`/`RenderConfig`(`MsaaMode::Off`、`headless()`の
    /// doc 参照)を使う——呼び出し側(`motolii-engine::Engine::with_device`)が
    /// `re_renderer::RenderConfig`/`MsaaMode`/`ScreenshotProcessor` を一切知らずに
    /// 済むための薄いラッパー。`output_format` の実際の効き所は
    /// `composite()`/screenshot 読み戻し側(`RenderContext` の「出力」format)で、
    /// [`Self::render_to_texture`]はそちらを一切通らない(readback しない)ので
    /// ここで固定しても実害が無い——`headless()`と同じ値を選んだのは新しい
    /// 意味を持ち込まないため。
    pub fn with_device_using_headless_defaults(
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<Self, CompositorError> {
        Self::with_device(
            device,
            queue,
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
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
        use re_renderer::resource_managers::{
            SourceImageDataFormat, YuvMatrixCoefficients, YuvPixelLayout, YuvRange,
        };

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
                // **この入口は分離可能 blend を実装しない**(モジュール doc「分離可能
                // blend」節、`fixed_function_tint_alpha` 参照)——`Normal`/`Add` 以外は
                // `Err(CompositorError::UnsupportedBlendMode)` で明示的に拒む。
                let a = fixed_function_tint_alpha(layer.blend_mode, layer.placement.opacity)?;
                Ok(TexturedRect {
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
                    colormapped_texture: re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                        layer.texture.clone(),
                    ),
                    options: RectangleOptions {
                        // premultiplied なので rgb は opacity で揃えて掛ける。alpha だけ
                        // blend mode で分かれる: Normal は opacity と同じ(通常の
                        // premultiplied alpha-over)。Add は 0(module doc の式変形どおり、
                        // `out = 1×src + (1-src.a)×dst` の `src.a` を 0 にすると
                        // `out = src + dst` になる — 加算合成そのもの、alpha は不変)。
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            layer.placement.opacity,
                            layer.placement.opacity,
                            layer.placement.opacity,
                            a,
                        ),
                        depth_offset: layer.placement.order,
                        ..Default::default()
                    },
                })
            })
            .collect::<Result<Vec<_>, CompositorError>>()?;

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

        // 2) 通常合成——逐次 accumulator 経路([`Self::accumulate_sequential`]、BL3で
        //    分離可能 blend 対応へ拡張、`crate` module doc「分離可能 blend」節)。
        //    使う texture だけが「元の layer.texture」から「上で決めた実効 texture」に
        //    変わる(padding 込みの quad 拡張は `SequentialInput::local_min`/
        //    `local_size` へそのまま持ち込む——`render_with_timing` 旧経路と同じ計算)。
        //
        // effect pass の copy(あれば)を、逐次経路が実効 texture を読み始める前に
        // 確定させる——旧実装は「最終合成と同じ submit へ同乗」させていたが、逐次経路は
        // 複数回に分けて submit する(layer 毎)ので、先に1回 flush して依存を切る。
        if let Some(encoder) = copy_encoder.take() {
            self.ctx.before_submit();
            self.ctx.queue.submit([encoder.finish()]);
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;
        }

        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .zip(effective_textures.iter())
            .zip(effective_paddings.iter())
            .map(|((lwp, texture), &padding)| {
                let layer = &lwp.layer;
                // **既知の穴の根治**: pass が出力を拡張した分(`padding`、texel、
                // `EffectPass::padding` 参照)だけ quad を local 空間で広げる——
                // `LayerPlacement::transform` 自体は一切変えず、この rect の
                // 組み立てだけが「実 texture が layer 実寸より大きい」事実を吸収
                // する(transform は affine なので、広げた local 矩形にそのまま
                // 掛ければ回転/拡大/skew があっても正しく追従する)。padding=0
                // (pass 無し/Identity のみ)なら local_min=(0,0)・local_size=
                // layer.size のまま、従来と完全に同じ幾何になる。
                let pad = padding as f32;
                SequentialInput {
                    texture,
                    local_min: glam::Vec2::new(-pad, -pad),
                    local_size: glam::Vec2::new(
                        layer.size[0] + 2.0 * pad,
                        layer.size[1] + 2.0 * pad,
                    ),
                    transform: layer.placement.transform,
                    z: layer.placement.z,
                    pinned: layer.pinned,
                    opacity: layer.placement.opacity,
                    depth_offset: layer.placement.order,
                    blend_mode: layer.blend_mode,
                }
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        let frame = self.finalize_readback(comp, camera, background)?;

        // GPU が読み終わった後なので、scratch をプールへ返して次フレームで使い回す
        // (毎フレーム作り直さない)。
        for (width, height, format, texture) in checked_out {
            self.effect_scratch.release(width, height, format, texture);
        }

        Ok(frame)
    }

    /// **裁定171 v2(M4)— zero-copy GPU 出力**。[`Self::render_with_effects`]と
    /// **同じ層構築・同じ合成**を通すが、CPU 読み戻し(`ScreenshotProcessor`)を
    /// 一切しない——呼び出し側(embedder、`motolii-shell` の presenter Pipeline)が
    /// 同じ `queue` へ後続の描画コマンドを積める GPU texture をそのまま返す。
    ///
    /// **既存4メソッド([`Self::render`]/[`Self::render_with_timing`]/
    /// [`Self::render_with_effects`]/[`Self::render_sequential`])は無改造** ——
    /// これは並設した新しい入口で、上の4つのどれの呼び出し元にも一切波及しない
    /// (裁定171 M4 supervisor 裁定「additive のみ」)。
    ///
    /// ## 順序保証(fence 不要)
    ///
    /// このメソッドは `self.ctx.queue.submit(...)` までで止まる——`device.poll`
    /// もしない。呼び出し側が返された texture を**同じ `queue`** 上の後続コマンド
    /// (例: iced の render pass でこの texture を sample する)で使う限り、GPU は
    /// submission 順に実行するので「書いてから読む」が構造的に成立する
    /// (裁定171 v2 §0-5)。
    ///
    /// ## main_target の生存期間(fork の doc がそのまま保証を与える)
    ///
    /// fork の `GpuTexture`(`Arc<DynamicResource<..>>`)は「全参照が drop されたら
    /// **次のフレーム**で回収対象になる」(`wgpu_resources/texture_pool.rs` の doc
    /// comment)。ここで返す `wgpu::Texture`/`wgpu::TextureView` は
    /// `view_builder`(このメソッドを抜けると drop される)が最後に握っていた
    /// `GpuTexture` の中身の clone であって、次に `self.ctx.begin_frame()` が
    /// 呼ばれる(= 次にこのメソッドか他の render系メソッドが呼ばれる)まで
    /// 回収は起きない。呼び出し側がこのメソッドを「内容が変わった時だけ」
    /// (世代ゲート)呼ぶ設計である限り、前回の texture は次回このメソッドを
    /// 呼ぶ**その時**まで有効——ちょうど呼び出し側が新しい texture へ差し替える
    /// タイミングと一致するので、古い方が消えても実害が無い。
    ///
    /// ## effect pass の scratch(`effect_scratch` を汚さない)
    ///
    /// [`Self::render_with_effects`]の scratch 解放(`effect_scratch.release`)は
    /// 「GPU が読み終わってから」が前提(`effects::EffectScratch::release` の doc
    /// 参照——読み終わりを `device.poll` で確認してから解放している)。この
    /// メソッドは poll をしない(順序保証を queue の submission 順だけに置く
    /// 設計、上記)ので、**取得した scratch は一度も `release` しない** ——
    /// `effect_scratch.acquire` 自体は「空きが無ければ新規生成」なだけの
    /// 安全な操作なので呼んでよいが(プールの再利用資産を消費するだけ)、
    /// このメソッドが確保した scratch を `free` リストへ戻すと、まだ GPU が
    /// 読み書き中かもしれない texture を次の `acquire` が上書き支給してしまう
    /// (`effects::EffectScratch::release` の doc がまさにこの罠を警告している)。
    /// 戻さなければ、その texture は関数を抜けた時点でこの関数のローカル変数
    /// (`scratch_textures_kept_alive`)の drop に委ねられる——wgpu 自身が
    /// 「GPU がその texture を参照するコマンドを使い終わるまで実体を残す」
    /// (標準の refcount 付き Drop、fork の独自 `.destroy()` 即時破棄とは別物)
    /// ので、poll なしでも安全。代償は「今回確保した scratch は次回使い回されない
    /// (次のフレームでまた新規生成)」——effect 付き layer が動いている間だけ
    /// 余分な確保が起きるが、`render_with_effects`(readback 経路、export/
    /// screenshot が実際に使う)の scratch 資産とは別物なので、そちらの
    /// oracle(`tests/effects.rs`)には一切影響しない。
    pub fn render_to_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[LayerWithPasses],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        // 1) layer ごとに「合成へ渡す実効 texture」を決める。
        // [`Self::render_with_effects`] の step 1 と**同じ構造**——唯一の違いは
        // `effect_scratch.release` を一度も呼ばないこと(上のモジュール doc 参照)。
        let mut effective_textures: Vec<GpuTexture2D> = Vec::with_capacity(layers.len());
        let mut effective_paddings: Vec<u32> = Vec::with_capacity(layers.len());
        // GPU が読み終わるまで実体を生かしておくための保持(release はしない —
        // 単に drop に任せる、上のモジュール doc「effect pass の scratch」参照)。
        let mut scratch_textures_kept_alive: Vec<wgpu::Texture> = Vec::new();
        let mut copy_encoder: Option<wgpu::CommandEncoder> = None;

        for lwp in layers {
            if lwp.passes.is_empty() {
                effective_textures.push(lwp.layer.texture.clone());
                effective_paddings.push(0);
                continue;
            }

            let [width, height] = lwp.layer.texture.width_height();
            let padding = lwp
                .passes
                .iter()
                .map(EffectPass::padding)
                .max()
                .unwrap_or(0);
            let padded_width = width + 2 * padding;
            let padded_height = height + 2 * padding;

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
                        label: Some("motolii-compositor-effect-pass-zero-copy"),
                    })
            });

            for pass in &lwp.passes {
                match pass {
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
                        {
                            let _clear_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some(
                                        "motolii-compositor-glow-padded-source-clear-zero-copy",
                                    ),
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

                        scratch_textures_kept_alive.push(padded_source);
                        scratch_textures_kept_alive.push(bloom);
                        scratch_textures_kept_alive.push(blur_ping);
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
            scratch_textures_kept_alive.push(scratch);
        }

        // 2) 通常合成。[`Self::render_with_effects`] の step 2 と同じ
        //    `accumulate_sequential` 経由の組み立て(`crate` module doc「分離可能
        //    blend」節)。
        //
        // **この関数だけの注意**: 元の実装は「readback をしない」ことを活かして
        // `device.poll` を一度も呼ばず、GPU キューへの submit 順序だけで
        // 正しさを保証していた(モジュール doc「順序保証」節)。分離可能 blend の
        // 逐次経路は layer 毎に新しい `ViewBuilder`(fork の per-frame 資源プール)を
        // 作る必要があり、そのために `accumulate_sequential` 内部で layer 毎に
        // `begin_frame`/`poll` を挟む(`render_sequential` 旧実装からの既存規律、
        // 単一 ViewBuilder では起きなかった同期点が増える——GPU 高速路の
        // パイプライン化を部分的に失うトレードオフ、FINDING 参照)。
        if let Some(encoder) = copy_encoder.take() {
            self.ctx.before_submit();
            self.ctx.queue.submit([encoder.finish()]);
            self.ctx.begin_frame();
            self.ctx
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .map_err(|e| CompositorError::Draw(e.to_string()))?;
        }

        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .zip(effective_textures.iter())
            .zip(effective_paddings.iter())
            .map(|((lwp, texture), &padding)| {
                let layer = &lwp.layer;
                let pad = padding as f32;
                SequentialInput {
                    texture,
                    local_min: glam::Vec2::new(-pad, -pad),
                    local_size: glam::Vec2::new(
                        layer.size[0] + 2.0 * pad,
                        layer.size[1] + 2.0 * pad,
                    ),
                    transform: layer.placement.transform,
                    z: layer.placement.z,
                    pinned: layer.pinned,
                    opacity: layer.placement.opacity,
                    depth_offset: layer.placement.order,
                    blend_mode: layer.blend_mode,
                }
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        let (texture, view) = self.finalize_texture(comp, camera, background)?;

        // `scratch_textures_kept_alive` はここで drop される——モジュール doc
        // 「effect pass の scratch」の保証がここで効く(GPU がまだ使用中でも wgpu
        // 自身が実体を残す)。
        drop(scratch_textures_kept_alive);

        Ok((texture, view))
    }

    /// 試験専用の introspection。`effect_scratch` が実際に**新規生成**した
    /// (プール再利用ではない)texture の総数。`RenderTiming` が時間の内訳を
    /// 隠さないのと同じ規律で、資源生成も隠さない。
    pub fn effect_passes_created_textures(&self) -> u64 {
        self.effect_scratch.created_count()
    }

    /// [`Self::render_sequential`]/[`Self::render_with_effects`]/[`Self::render_to_texture`]
    /// が共有する、逐次合成の核(裁定161 BL1b が確立した main_target アクセサ経路を、
    /// BL3で「Normal/Add の固定式高速路」と「分離可能 blend の2段パス」の分岐へ拡張)。
    ///
    /// layer を順に accumulator へ焼き込み、直前までの accumulator の裏付け
    /// ([`AccumulatorBacking`]、fork pool の reclaim から守る Arc か、自前 scratch か)を
    /// 返す——CPU 読み戻しをするか([`Self::finalize_readback`])・GPU texture のまま
    /// 返すか([`Self::finalize_texture`])は呼び手が選ぶ(この関数自体はどちらもしない)。
    ///
    /// ## Normal/Add(`separable_mode_index` が `None`)
    ///
    /// 「background rect + layer rect を同じ `ViewBuilder` で描く」固定式の高速路
    /// (旧 `render_sequential` の本体そのまま)。この2枚は**同じ main_target の中**で
    /// 描かれるので、一括経路(`render_with_timing`)が N layer を1つの main_target の
    /// 中で順に混ぜる時と**layer あたりの quantize 回数が同じ**になり、バイト一致する
    /// (裁定161、`tests/sequential.rs` の overlap fixture が縛る)。
    ///
    /// ## 分離可能 blend(`Some`、Multiply〜Exclusion)
    ///
    /// 固定式では表現できない(`crate` module doc「分離可能 blend」節)ので2段:
    /// 1. layer 単体を(dst 無しで)自分の main_target へ描く——「layer 単体を transparent
    ///    へ premultiplied-over した canvas」を得る。
    /// 2. 直前までの accumulator が有れば、[`blend::SeparableBlendPipelines`] で
    ///    2枚読み混ぜて新しい accumulator を作る(`blend` モジュール doc の一般合成式)。
    ///    **無ければ**(1枚目)layer 単体の描画結果がそのまま新しい accumulator になる
    ///    ——`blend` モジュール doc が導出するとおり `αb=0` では `Co = αs·Cs` と数学的に
    ///    一致するので、混ぜる処理自体を省いてよい。
    ///
    /// ## fork pool の罠(`AccumulatorBacking::Fork` にのみ効く)
    ///
    /// fork の `GpuTexturePool::begin_frame` は、通常(import ではない)texture の参照が
    /// 尽きると reclaim 時に `res.texture.destroy()` を**明示的に**呼ぶ——import は
    /// 「別の `wgpu::Texture` clone」を作るだけで、元の pool エントリの生存とは独立に
    /// 守ってくれない。そのため `ViewBuilder::main_target().clone()` で `GpuTexture`
    /// (Arc)を明示的に握り続け、次の layer の背景として使い終わる(= 次の submit を
    /// poll で待ち終える)まで手放さない([`AccumulatorBacking::Fork`] 参照)。
    /// [`AccumulatorBacking::Scratch`](blend pass の出力)はこのプールに属さない
    /// 素の `wgpu::Texture` なので、この罠は無関係(Rust の所有権だけで足りる)。
    fn accumulate_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
    ) -> Result<Option<(AccumulatorBacking, GpuTexture2D)>, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut background: Option<(AccumulatorBacking, GpuTexture2D)> = None;

        for input in inputs {
            self.ctx.begin_frame();

            let (transform, z) = if input.pinned {
                (pinned_cancel * input.transform, 0.0)
            } else {
                (input.transform, input.z)
            };

            if let Some(mode_index) = separable_mode_index(input.blend_mode) {
                // --- 分離可能 blend: layer 単体をまず自分の main_target へ描く ---
                let solo_rect = TexturedRect {
                    top_left_corner_position: to_point3(
                        transform.transform_point2(input.local_min),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(input.local_size.x, 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, input.local_size.y)),
                    ),
                    colormapped_texture: ColormappedTexture::from_unorm_rgba(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            input.opacity,
                        ),
                        depth_offset: 0,
                        ..Default::default()
                    },
                };
                let draw_data = RectangleDrawData::new(&self.ctx, &[solo_rect])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let mut solo_view_builder = ViewBuilder::new(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-sequential-solo",
                        comp,
                        view_from_world,
                        projection,
                    ),
                    ViewBuilderId::new(self.next_readback),
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;

                solo_view_builder.queue_draw(&self.ctx, draw_data);
                let command_buffer = solo_view_builder
                    .draw(&self.ctx, Rgba::TRANSPARENT)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;

                self.ctx.before_submit();
                self.ctx.queue.submit([command_buffer]);
                self.ctx.begin_frame();
                self.ctx
                    .device
                    .poll(wgpu::PollType::wait_indefinitely())
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;

                let layer_canvas: GpuTexture = solo_view_builder.main_target().clone();

                match background.take() {
                    None => {
                        // 混ぜる相手が無い(1枚目)——layer 単体の結果がそのまま新しい
                        // accumulator になる(関数 doc「αb=0」節)。
                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &layer_canvas.texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((AccumulatorBacking::Fork(layer_canvas), imported));
                    }
                    Some((backing, _)) => {
                        let dst_view = backing.texture().create_view(&Default::default());
                        let src_view = layer_canvas.default_view.clone();
                        let out_texture =
                            self.create_blend_scratch_texture(comp.width, comp.height);
                        let out_view = out_texture.create_view(&Default::default());

                        let mut encoder = self.ctx.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("motolii-compositor-blend-pass-encoder"),
                            },
                        );
                        self.blend_pipelines.record(
                            &self.ctx.device,
                            &self.ctx.queue,
                            &mut encoder,
                            &dst_view,
                            &src_view,
                            &out_view,
                            mode_index,
                        );
                        self.ctx.before_submit();
                        self.ctx.queue.submit([encoder.finish()]);
                        self.ctx.begin_frame();
                        self.ctx
                            .device
                            .poll(wgpu::PollType::wait_indefinitely())
                            .map_err(|e| CompositorError::Draw(e.to_string()))?;

                        self.next_effect_key += 1;
                        let key = self.next_effect_key;
                        let imported = self
                            .ctx
                            .texture_manager_2d
                            .import_gpu_premultiplied(key, &self.ctx, &out_texture)
                            .map_err(|e| CompositorError::Effect(e.to_string()))?;
                        background = Some((AccumulatorBacking::Scratch(out_texture), imported));
                        // `backing`(直前の accumulator)はここで drop される——直前の
                        // poll で GPU 読み取りは完了済みなので安全(fork pool の罠は
                        // 関数 doc 参照、`Scratch` ならそもそも罠が無い)。
                    }
                }
            } else {
                // --- 固定式高速路(Normal/Add): background rect + layer rect を
                //     同じ ViewBuilder で描く(旧 `render_sequential` 本体そのまま)。
                let mut rects: Vec<TexturedRect> = Vec::with_capacity(2);
                if let Some((_, imported)) = &background {
                    // 「今回重ねる layer より1小さい」だけ——`background_rect` doc の
                    // 「極端値を使わない」節参照(過去に `i16::MIN` で外周1px欠落を
                    // 引いた)。
                    rects.push(background_rect(
                        comp,
                        pinned_cancel,
                        imported.clone(),
                        input.depth_offset.saturating_sub(1),
                    ));
                }
                let a = match input.blend_mode {
                    BlendMode::Normal => input.opacity,
                    BlendMode::Add => 0.0,
                    // `separable_mode_index` が `None` を返した mode(Normal/Add)のみ
                    // この分岐へ来る——他の全 variant は上の `if let Some(...)` 側。
                    _ => unreachable!(
                        "separable_mode_index が None を返した blend_mode のみここへ来る"
                    ),
                };
                rects.push(TexturedRect {
                    top_left_corner_position: to_point3(
                        transform.transform_point2(input.local_min),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(input.local_size.x, 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, input.local_size.y)),
                    ),
                    colormapped_texture: ColormappedTexture::from_unorm_rgba(input.texture.clone()),
                    options: RectangleOptions {
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            input.opacity,
                            input.opacity,
                            input.opacity,
                            a,
                        ),
                        depth_offset: input.depth_offset,
                        ..Default::default()
                    },
                });

                let draw_data = RectangleDrawData::new(&self.ctx, &rects)
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

                let mut view_builder = ViewBuilder::new(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-sequential-layer",
                        comp,
                        view_from_world,
                        projection,
                    ),
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

                let held: GpuTexture = view_builder.main_target().clone();
                self.next_effect_key += 1;
                let key = self.next_effect_key;
                let imported = self
                    .ctx
                    .texture_manager_2d
                    .import_gpu_premultiplied(key, &self.ctx, &held.texture)
                    .map_err(|e| CompositorError::Effect(e.to_string()))?;
                background = Some((AccumulatorBacking::Fork(held), imported));
            }
        }

        Ok(background)
    }

    /// 分離可能 blend パスの出力先。fork の texture pool(`effect_scratch`)には
    /// **属さない**普通の `wgpu::Texture`——[`Self::accumulate_sequential`] doc の
    /// 「fork pool の罠」が無いので、素の Rust 所有権(drop で破棄)だけで足りる。
    fn create_blend_scratch_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-compositor-blend-output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: blend::SEPARABLE_BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    /// [`Self::accumulate_sequential`]の結果を**1回だけ** screenshot 経由で CPU 読み
    /// 戻す(旧 `render_sequential`「最終変換は1回だけ」節と同じ理由——分離可能 blend
    /// パスは新しい gamma 変換を持ち込まない、8bit sRGB の unmultiply→encode→
    /// re-multiply は依然としてここ1箇所だけ、`composite.wgsl` は無改造のまま)。
    fn finalize_readback(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
    ) -> Result<Vec<u8>, CompositorError> {
        let projection = motolii_core::camera_projection(comp, camera);
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut final_rects: Vec<TexturedRect> = Vec::with_capacity(1);
        if let Some((_, imported)) = &background {
            // ここは常に単独 rect(sort 順の懸念は無い)——`background_rect` doc の
            // 「極端値を使わない」節に沿って、小さい定数値を渡す。
            final_rects.push(background_rect(comp, pinned_cancel, imported.clone(), -1));
        }

        let final_draw_data = RectangleDrawData::new(&self.ctx, &final_rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut final_view_builder = ViewBuilder::new(
            &self.ctx,
            sequential_target_config(
                "motolii-comp-sequential-finalize",
                comp,
                view_from_world,
                projection,
            ),
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
        ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        out.ok_or(CompositorError::ReadbackMissing)
    }

    /// [`Self::accumulate_sequential`]の結果を CPU 読み戻しせずそのまま返す
    /// (`Self::render_to_texture` 専用)。**`background_rect` を経由する追加
    /// `ViewBuilder` を挟まない**——accumulator の main_target 自身が既に最終画な
    /// ので、screenshot 経路(`composite.wgsl` のガンマ round-trip)は不要(旧
    /// `render_to_texture` が単一 `ViewBuilder` の `main_target()` を直接返して
    /// いたのと同じ理由)。`background` が `None`(layers が空)の時だけ、何も
    /// 描かない transparent な `ViewBuilder` を1つ作って返す(旧実装の空 layers
    /// 挙動を保つ)。
    fn finalize_texture(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        background: Option<(AccumulatorBacking, GpuTexture2D)>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), CompositorError> {
        match background {
            Some((backing, _imported)) => {
                let texture = backing.texture().clone();
                let view = texture.create_view(&Default::default());
                Ok((texture, view))
            }
            None => {
                let projection = motolii_core::camera_projection(comp, camera);
                let view_from_world = macaw::IsoTransform::from_rotation_translation(
                    projection.rotation,
                    -(projection.rotation * projection.eye),
                );
                let draw_data = RectangleDrawData::new(&self.ctx, &[])
                    .map_err(|e| CompositorError::Rectangles(e.to_string()))?;
                let mut view_builder = ViewBuilder::new(
                    &self.ctx,
                    sequential_target_config(
                        "motolii-comp-zero-copy-empty",
                        comp,
                        view_from_world,
                        projection,
                    ),
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

                let target = view_builder.main_target();
                let texture = target.texture.clone();
                let view = target.default_view.clone();
                drop(view_builder);
                Ok((texture, view))
            }
        }
    }

    /// **逐次合成(accumulator)経路の薄い入口**(裁定161 BL1b、BL3 で分離可能 blend
    /// 対応へ拡張、2026-08-22)。実装は [`Self::accumulate_sequential`]+
    /// [`Self::finalize_readback`](両方とも [`Self::render_with_effects`]/
    /// [`Self::render_to_texture`] と共有) —— この関数自体は「`Layer` を
    /// [`SequentialInput`] へ詰め替えるだけ」。
    ///
    /// [`Self::render`]/[`Self::render_with_timing`]/[`Self::render_with_effects`] は
    /// **無改造のまま**——この関数は並設した入口で、既存3つの呼び出し元へ
    /// 一切波及しない。`tests/sequential.rs` の overlap fixture(Normal/Add の
    /// バイト一致)は裁定161 のまま維持——固定式高速路は無改造。
    pub fn render_sequential(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        let inputs: Vec<SequentialInput<'_>> = layers
            .iter()
            .map(|layer| SequentialInput {
                texture: &layer.texture,
                local_min: glam::Vec2::ZERO,
                local_size: glam::Vec2::new(layer.size[0], layer.size[1]),
                transform: layer.placement.transform,
                z: layer.placement.z,
                pinned: layer.pinned,
                opacity: layer.placement.opacity,
                depth_offset: layer.placement.order,
                blend_mode: layer.blend_mode,
            })
            .collect();

        let background = self.accumulate_sequential(comp, camera, &inputs)?;
        self.finalize_readback(comp, camera, background)
    }
}

/// [`Compositor::accumulate_sequential`]が扱う「直前までの accumulator」の裏付け。
/// **`Fork`**: `ViewBuilder::main_target()`(裁定161 fork accessor)由来——fork の
/// texture pool の reclaim/destroy から実体を守るため、明示的に `GpuTexture`(Arc)を
/// 握る(`Compositor::accumulate_sequential` doc「fork pool の罠」節参照)。
/// **`Scratch`**: 分離可能 blend パスの出力(このファイルで直接
/// `device.create_texture` した物、`Compositor::create_blend_scratch_texture`)。
/// fork のプールに属さないので reclaim の心配はない——普通の Rust 所有権で足りる。
enum AccumulatorBacking {
    Fork(GpuTexture),
    Scratch(wgpu::Texture),
}

impl AccumulatorBacking {
    fn texture(&self) -> &wgpu::Texture {
        match self {
            Self::Fork(g) => &g.texture,
            Self::Scratch(t) => t,
        }
    }
}

/// [`Compositor::accumulate_sequential`]が受け取る、1 layer 分の入力。`Layer`
/// (`Compositor::render_sequential`)と `LayerWithPasses`+実効 texture
/// (`Compositor::render_with_effects`/`Compositor::render_to_texture`、padding 込み)の
/// 両方をこの共通形へ詰め替える。
struct SequentialInput<'a> {
    texture: &'a GpuTexture2D,
    /// 板のローカル矩形の左上(padding が無ければ `Vec2::ZERO`、`EffectPass::padding`
    /// で拡張された分だけ負に振れる——`render_with_effects` 旧 step2 と同じ計算)。
    local_min: glam::Vec2,
    /// 板のローカル矩形の大きさ(`layer.size` に padding 拡張を足した分)。
    local_size: glam::Vec2,
    /// `LayerPlacement::transform`(pinned 解決前の生値)。
    transform: glam::Affine2,
    /// `LayerPlacement::z`(pinned なら無視され 0 になる)。
    z: f32,
    /// `LayerAttrs::pinned`(裁定113)。
    pinned: bool,
    opacity: f32,
    depth_offset: i16,
    blend_mode: BlendMode,
}

/// `Compositor::accumulate_sequential`/`finalize_readback`/`finalize_texture` が
/// 繰り返し組み立てる `TargetConfiguration`(既存メソッド群のリテラルをそのまま
/// 関数化しただけ、新しいフィールドは無い)。
fn sequential_target_config(
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

/// [`Compositor::accumulate_sequential`]/[`Compositor::finalize_readback`]/
/// [`Compositor::finalize_texture`]の「背景 rect」: 直前までの逐次合成結果を、
/// 画面に張り付く板として画面いっぱいに敷く。`pinned` layer が使っているのと
/// **同じ** `pinned_cancel` 変換を、full-canvas な矩形
/// (`(0,0)-(comp.width,comp.height)`)に適用するだけ——新しい幾何は無い。
///
/// `depth_offset` は呼び手が決める(BL3 で `i16::MIN` 固定から変更——理由は下記)。
fn background_rect(
    comp: CompSpec,
    pinned_cancel: glam::Affine2,
    imported: GpuTexture2D,
    depth_offset: i16,
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
            // **`i16::MIN` のような極端値を使わない**(BL3 で発見・修正、`background.rs`
            // の `opaque_background_leaves_no_transparent_border_pixels` 回帰)。
            // `depth_offset` は単なる sort key ではなく、透視投影カメラ(裁定115)の
            // 下では実際に world の z 方向へ板を押し込む——押し込み量が `background`
            // rect の他 rect との差(この場合 `i16::MIN` 級)ほど大きいと、透視の
            // 遠近効果で矩形が画面上わずかに縮み、外周 1px の被覆が丸ごと抜ける
            // (実測: 640×360 comp・pinned 背景 1枚だけで再現、`depth_offset` を
            // `-1` まで下げた瞬間に外周欠落が消えて `render()` とバイト一致した)。
            // 呼び手は「この呼び出しで実際に上へ重ねる rect の depth_offset より
            // 1 小さい値」を渡す(`accumulate_sequential`/`finalize_readback` 参照)
            // ——sort 順の正しさは保ちつつ、押し込み量を最小に保つ。
            depth_offset,
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
