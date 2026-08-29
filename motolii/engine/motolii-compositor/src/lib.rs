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
//! ## 非分離(non-separable)blend 4 モード(BL4、2026-08-22)
//!
//! Hue/Saturation/Color/Luminosity は W3C Compositing 3.7節(SetLum/SetSat/ClipColor
//! の擬似コード)——分離可能11種と違い `B(Cb,Cs)` が RGB 全体を1単位として扱う
//! (per-channel には分解できない)。**同じ [`blend`] サブモジュールへ相乗りさせる**
//! (新規 pipeline を増やさない——2枚読みの土台(2 texture bind group・params uniform・
//! fullscreen triangle)も一般合成式(3.5節)も分離可能11種と完全に同じで、違うのは
//! `B(Cb,Cs)` の中身だけなので、[`blend::BlendPipelines`]/WGSL 内で
//! `params.mode` の範囲を 0〜10(分離可能)から 11〜14(非分離)へ拡張しただけ——
//! 数式・境界条件の出典は `blend` モジュール doc「非分離4種」節)。
//! [`separable_mode_index`]/[`nonseparable_mode_index`] のどちらが `Some` を返すかで
//! 分類の意味は保ったまま、[`accumulate_sequential`] の run 切り出し判定は
//! [`two_texture_pass_mode_index`](両者の union)1本に集約する。
//!
//! **[`Compositor::render`]/[`Compositor::render_with_timing`] は分離可能/非分離
//! どちらの2枚読み blend も実装しない** — この2つは「1つの ViewBuilder へ全 layer を
//! まとめて描く」一括経路([`Self::render_with_timing`] 参照)で、dst を読む2枚読み
//! パスが構造的に乗らない(乗せるなら `render_sequential` と同じ逐次経路へ丸ごと
//! 作り替える必要があり、この2つの「昔からある一括経路」を無改造に保つ既存規律に
//! 反する)。2枚読みを要る blend mode をこの2つへ渡すと
//! [`CompositorError::UnsupportedBlendMode`] を返す(黙って `Normal` へ近似しない、
//! `translate_blend_mode` と同じ fail-closed)。
//!
//! ## track matte(BL4、2026-08-22)
//!
//! AE 型(直上レイヤーを alpha/luma マットとして使う、`motolii_store::Matte`/`MatteMode`
//! の4値)。[`matte`] サブモジュールが第三の新規 WGSL パイプライン([`matte::MattePipelines`]、
//! `blend`/`effects::glow` と同じ「fork を触らず crate 内へ足す」手口)を持ち、
//! [`Compositor::matte_layer`] がそれを使う——数式の出典・設計判断は `matte` モジュール
//! doc 参照。**engine 側の消費経路はまだ「マット層を絵から除外」まで繋がっていない**
//! (`motolii-engine` の `EngineError::UnsupportedMatte` doc 参照、store 側に要る形を記す)。
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
    /// `SetLum(SetSat(Cs, Sat(Cb)), Lum(Cb))`(非分離、`blend` モジュール doc「非分離
    /// 4種」節参照)。
    Hue,
    /// `SetLum(SetSat(Cb, Sat(Cs)), Lum(Cb))`。
    Saturation,
    /// `SetLum(Cs, Lum(Cb))`。
    Color,
    /// `SetLum(Cb, Lum(Cs))`。
    Luminosity,
}

/// [`BlendMode`] が分離可能 blend([`blend`] サブモジュールの2枚読みパス、`params.mode`
/// 0〜10)に属すならその WGSL index(`blend::SHADER` の `blend_channel` と1対1)を返す。
/// `Normal`/`Add`(固定式)・非分離4種([`nonseparable_mode_index`] 参照)は `None`。
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
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => None,
    }
}

/// [`BlendMode`] が非分離 blend(BL4、[`blend`] サブモジュールの同じ2枚読みパスへ
/// `params.mode` 11〜14 で相乗りする分)に属すならその index を返す。それ以外は `None`
/// (`separable_mode_index` と対で、両者が `Some` を返す variant は無い——
/// `two_texture_pass_mode_index` の union が構造的に排他になる)。
///
/// **`_` を使わない**(`separable_mode_index` と同じ fail-closed の形)。
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

/// [`separable_mode_index`] ∪ [`nonseparable_mode_index`] —— [`blend`] サブモジュールの
/// 2枚読みパスへ回す全 blend mode(0〜14、合計15)の統一判定。[`accumulate_sequential`]
/// の run 切り出しはこれ1本だけを見る(BL4 で非分離4種を追加した際、
/// `separable_mode_index`/`nonseparable_mode_index` それぞれの「分離可能かどうか」の
/// 意味の分類は保ったまま、呼び出し側の判定だけをここへ集約した)。
fn two_texture_pass_mode_index(mode: BlendMode) -> Option<u32> {
    separable_mode_index(mode).or_else(|| nonseparable_mode_index(mode))
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
pub(crate) fn to_point3(v: glam::Vec2, z: f32) -> glam::Vec3 {
    glam::vec3(v.x, v.y, z)
}

/// 板の**辺ベクトル**(`extent_u`/`extent_v`)。板は自分の z 平面に対して常に平行
/// (裁定115: 姿勢の表現はまだ開けない)なので、方向ベクトルの z 成分は常に 0。
pub(crate) fn to_vector3(v: glam::Vec2) -> glam::Vec3 {
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

/// ISF manifest の型(`effects::isf` モジュール doc 参照)。front/engine 側が
/// `EffectPass::Isf` の param カタログを**手書きした後で**この manifest と
/// 突き合わせて drift を検査できるように公開する(`motolii-engine::translate`
/// 側の cross-check test 参照——2026-08-27 の `TURBULENT_DISPLACE` 事故と同種の
/// 「front/engine が実体と食い違う」を、今回は compile-time ではなく test-time で
/// 検査する形)。
pub use effects::{IsfInput, IsfInputType, IsfManifest};

/// `BLOOM_SOURCE` を1回だけ parse した結果。**GPU device は要らない**
/// (manifest の JSON 解析だけなら device 非依存——`effects::isf::IsfProgram::compile`
/// が device を要るのは GLSL→WGSL コンパイル+pipeline 構築の方であって、
/// manifest 自体はどちらも同じ `parse_isf_source` を呼ぶ)。
pub fn isf_bloom_manifest() -> &'static IsfManifest {
    static MANIFEST: std::sync::OnceLock<IsfManifest> = std::sync::OnceLock::new();
    MANIFEST.get_or_init(|| {
        effects::isf::parse_isf_source(effects::BLOOM_SOURCE)
            .expect("bloom.fs はビルドに埋め込まれた定数——parse 失敗はここのバグ")
            .0
    })
}

/// track matte の重ね方(BL4、AE/Lottie の4値)。`matte` モジュール doc 参照。
pub use matte::MatteMode;

/// 共有面へ書く口の検査(裁定256)。`presentable` モジュール doc 参照。
pub use presentable::{check_presentable_target, PRESENTABLE_FORMAT};

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
    /// [`Compositor::with_device`] が `effects::isf::IsfProgram::compile` を
    /// 呼ぶ時だけ発生しうる(ISF の JSON ヘッダが読めない/naga が GLSL を
    /// 解析・検証・WGSL 書き出しできない、`effects::isf::IsfError` 参照)。
    #[error("ISF effect の読み込みに失敗した: {0}")]
    Isf(String),
    /// [`Compositor::render`]/[`Compositor::render_with_timing`] が分離可能 blend
    /// ([`separable_mode_index`] が `Some` を返す mode)を渡された時(モジュール doc
    /// 「分離可能 blend」節参照)。黙って `Normal` へ近似しない。
    #[error("この入口は分離可能 blend mode を表現できない: {0:?}")]
    UnsupportedBlendMode(BlendMode),
    #[error("共有面の画素形式が Host 仕様ではない: {got}")]
    PresentableFormat { got: String },
    #[error("共有面のサイズが comp と違う: got={got:?} expected={expected:?}")]
    PresentableSize { got: [u32; 2], expected: [u32; 2] },
    #[error("共有面に RENDER_ATTACHMENT が無い")]
    PresentableUsage,
    // `PresentableWriteNotWired` はここに居た。「rerun の
    // `ViewBuilder::new_with_external_resolved` が着くまで共有面へ直接書かない」
    // という**待ちの番人**で、裁定256 で fork にそれが着いた時点で前提が消えた。
    // 宣言は残ったが構築する側は無く、検査していたのは起こり得ない事態だった
    // (2026-08-27 撤去)。
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
    pub(crate) ctx: RenderContext,
    /// 読み戻しの識別子。1つの Compositor で連番にする。
    pub(crate) next_readback: u64,
    /// [`Compositor::render_with_effects`]/[`Compositor::render_sequential`] が
    /// texture_manager_2d へ import する時の cache key。`next_readback` とは別の
    /// keyspace(`import_gpu_premultiplied` の `key` は screenshot の識別子と無関係)。
    pub(crate) next_effect_key: u64,
    /// layer 単位オフスクリーンパスの中間 texture プール(裁定153 S2)。
    /// **Compositor 所有・フレームをまたいで再利用** — `effects` モジュール doc 参照。
    pub(crate) effect_scratch: effects::EffectScratch,
    /// Glow の shader pipeline(裁定153 S4)。**初回生成して以後使い回す**
    /// (`effects::glow` モジュール doc 参照)。他の pass 種別が増えても
    /// pipeline はサイズ非依存なので、layer やフレームをまたいで作り直さない。
    pub(crate) glow_pipelines: effects::GlowPipelines,
    /// 内蔵 vism 第2号(`EffectPass::Isf`、`effects::isf` モジュール doc 参照)。
    /// `glow_pipelines` と同じ規律 — 初回生成して以後使い回す。今のところ
    /// `effects::BLOOM_SOURCE` 1本だけを持つ(複数 ISF ファイルを同時に
    /// 持たせるなら `HashMap<PluginId, IsfProgram>` 化が要るが、今回は「1本を
    /// 実際に通す」が scope——`isf` モジュール doc「意図的に配線していない物」)。
    pub(crate) isf_bloom: effects::IsfProgram,
    /// 内蔵 vism 第3号(`EffectPass::Gradient`、`effects::wgsl_fragment` モジュール
    /// doc 参照)。`isf_bloom` と同じ規律 — 初回生成して以後使い回す。
    pub(crate) wgsl_gradient: effects::WgslFragmentProgram,
    /// 内蔵 vism 第4号(`EffectPass::TriLed`)。`wgsl_gradient` と同じ規律。
    pub(crate) wgsl_tri_led: effects::WgslFragmentProgram,
    /// 分離可能+非分離 blend(BL3/BL4)の shader pipeline。`glow_pipelines` と同じ規律 —
    /// 初回生成して以後使い回す(`blend` モジュール doc 参照)。
    pub(crate) blend_pipelines: blend::SeparableBlendPipelines,
    /// track matte(BL4)の shader pipeline。同じ規律 — 初回生成して以後使い回す
    /// (`matte` モジュール doc 参照)。
    pub(crate) matte_pipelines: matte::MattePipelines,
    /// 試験専用の introspection 累計カウンタ(`effect_passes_created_textures` と
    /// 同じ規律)。[`Self::accumulate_sequential`] 内で `queue.submit` を呼ぶ毎に
    /// 増分する——run-batching(BL3 merge の構造退行の根治)が「blend の切れ目
    /// (分離可能 blend layer の出現点)以外では submit が増えない」ことを縛る oracle。
    pub(crate) sequential_submits: u64,
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
