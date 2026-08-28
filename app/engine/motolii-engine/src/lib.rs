//! wraps: motolii-store + motolii-compositor — **1フレームを出す唯一の経路**。
//!
//! 背骨2: preview も export も [`Engine::render_frame`] を呼ぶ。窓の有無だけが違う。
//! ここに「書き出し専用の速い道」を足さない — 足した瞬間に「見た絵 ≠ 出る絵」が生まれる。
//! **唯一の例外**: [`Engine::render_frame_without_background`](市松の透明可視化専用、
//! 裁定141)。export は絶対に使わない。preview だけが市松 ON の間に切り替える口で、
//! 同じ合成器・同じ層構築を共有する差分入力として実装してある(第二経路ではない)。
//!
//! **もう一つの入力差分**: [`Engine::render_frame_with_view_camera`](観測視点専用、
//! 裁定157)。Document のレンダリングカメラ(`Composition.camera`)ではなく
//! [`ObservationCamera`](Shell 直下の表示専用状態、Document 非搭載)で camera を組む
//! ——上と同型で、export は知らない・呼ぶのは shell の Stage 表示だけ。
//! [`Engine::render_frame`]の2引数固定シグネチャはこの追加で変わっていない。
//!
//! この crate 自身は意味を持たない。Document の意味は `motolii-store`、
//! 補間は `motolii-eval`、描画は `re_renderer` にある。ここは繋ぐだけ。

use std::collections::{HashMap, VecDeque};

pub mod mask;
/// `Vec<ShapeNode>` → `motolii-vector::render_tree` → RGBA8(発注「シェイプが画に
/// 出るようにする」、2026-08-22)。**`texture_for` の `LayerSource::Shape` 枝の
/// 呼び手(`Engine::shape_texture_for`/`Engine::shape_texture_from_shapes`)から
/// 繋がった** — module doc(`shape.rs`)参照。
pub mod shape;
/// TextDocument → 輪郭 → RGBA8(裁定190 切片2)。**BL4/切片3 で `texture_for` の
/// 呼び手(`Engine::text_texture_for`)から繋がった** — module doc(`text.rs`)参照。
pub mod text;

/// レイヤー組み立て・render 経路(`render_with_camera_override`/`layers_from_resolved`/
/// `render_resolved_to_texture*`/`render_frame_to_texture`/`apply_matte`)。SP-7(2026-08-23)で
/// `lib.rs` から移送——module doc(`render.rs`)参照。
mod render;
/// `texture_for`(素材/テキスト/シェイプの取得)とそのキャッシュ鍵。SP-7(2026-08-23)で
/// `lib.rs` から移送——module doc(`texture.rs`)参照。
mod texture;
/// 語彙の変換(BlendMode/MatteMode/effect)。SP-7(2026-08-23)で `lib.rs` から移送——
/// module doc(`translate.rs`)参照。
mod translate;

use motolii_compositor::GpuTexture2D;
use motolii_compositor::{Compositor, CompositorError};
use motolii_core::ResolvedCamera;

use motolii_media::ContainerInfo;
use motolii_media::MediaError;
use motolii_media::MediaInfo;
use motolii_media::PointCloudData;
use motolii_store::{LayerSource, Matte, RationalTime, StoreView};

use crate::texture::{ShapeCacheKey, TextCacheKey};

pub use crate::translate::{known_effects, EffectDescriptor, EffectParamDescriptor};

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
    /// **BL4(2026-08-22)時点でもう構築されない**——`motolii_store::BlendMode` の17値
    /// すべてが `translate_blend_mode` で `motolii_compositor::BlendMode` へ写せる
    /// ようになった(非分離4種も `motolii-compositor` 側に実装が揃った)。型は
    /// 将来 `motolii_compositor::BlendMode` に variant が増えた時の枠として残す。
    #[error("blend mode {0:?} はまだ合成器が対応していない(Normal のみ対応。fork 改造候補)")]
    UnsupportedBlendMode(motolii_store::BlendMode),
    /// **この発注(テキスト+matte 結線、2026-08-22)で `render_frame` 経路はもう
    /// 構築しない**——`render_with_camera_override` の `for layer in &resolved`
    /// ループは `ResolvedLayer.id`(BL4 で store 側に追加済み)から `LayerId → &ResolvedLayer`
    /// の索引(`by_id`)を作り、`matte.layer` をそこで引いて `Engine::apply_matte`
    /// (`motolii_compositor::Compositor::matte_layer` の薄いラッパー)へ実際に渡す
    /// ようになった——matte 元は `matte_sources`(`HashSet<LayerId>`)で通常描画
    /// リストから除外されるので、二重描画も起きない。
    ///
    /// **2026-08-22(ゼロコピー経路にも matte とテキストを通す発注)で
    /// `layers_from_resolved`(裁定171 v2 M4、zero-copy GPU 出力の並走レーン)も
    /// この `Err` を返さなくなった**——`render_with_camera_override` と同じ手口
    /// (`by_id`/`matte_sources`/`Engine::apply_matte`)をそのまま複製して繋いだ
    /// (`layers_from_resolved` の doc 参照)。matte 自体は `TextDocument` を
    /// 必要としない(`StoreView` が無くても `resolved: &[ResolvedLayer]` の `id`
    /// だけで消費できる)ので、この結線に `&StoreView` を新たに持ち込む必要は
    /// 無かった——`layers_from_resolved` は今も `Document`/`StoreView` を
    /// 一切知らない。この variant 自体は型として残す(`EngineError::UnsupportedBlendMode`
    /// と同じ「将来また使う枠」の扱い)が、**この crate のどちらの層構築経路からも
    /// もう構築されない**。
    #[error("matte はまだ engine が絵から除外しつつ消費する経路に繋がっていない({0:?})")]
    UnsupportedMatte(Matte),
    /// `LayerSource::Text` のラスタライズ失敗(`crate::text::rasterize_text_document`
    /// が返す2種の失敗をそのまま畳む——フォントが読めない/OpenType feature タグが
    /// 不正、または canvas が描けない大きさ)。`Ok(None)`(styles 空/content 空)は
    /// エラーではないので、ここには乗らない([`crate::text`] module doc 参照)。
    #[error(transparent)]
    Text(#[from] crate::text::TextRenderError),
    /// `LayerSource::Shape` のラスタライズ失敗(`crate::shape::rasterize_shapes` が
    /// そのまま返す `motolii_vector::VectorError` を畳む)。text と違いシェイピング
    /// 段(cosmic-text 相当)を持たないので失敗理由は1種類だけ——`TextRenderError`
    /// のような複合型は要らない。
    #[error(transparent)]
    Shape(#[from] motolii_vector::VectorError),
}

/// 背景 layer の `LayerPlacement::order`(= `re_renderer::DepthOffset`、`i16`)。
///
/// **`i16::MIN` を使ってはいけない**(2026-08-21 実測・真因)。`order` は
/// `motolii-compositor::render_with_timing` で `RectangleOptions::depth_offset` へ
/// そのまま渡り、上流 shader `depth_offset.wgsl` の `apply_depth_offset` が
/// `w_scale = 1.0 - f32eps * offset`(`f32eps = 2^-23`)で clip 空間の `w` を
/// スケールする。これは NDC 座標(= `x_proj / w_proj`)を原点へ向けて一様に縮める —
/// `order` の絶対値が大きいほど、板が画面中心へわずかに縮む。
///
/// 縮み幅は近似的に `half_dimension_px * f32eps * |offset|`(画素)。
/// `order = i16::MIN`(32768)・640x360 comp(横の半幅 320px)では
/// `320 * 2^-23 * 32768 = 1.25px` — 0.5px を超えるので**外周ちょうど1画素幅**が
/// ラスタライズから漏れる(ピクセル中心 0.5 が板の左端 1.25 より内側に入ってしまい
/// カバーされない。右端・上端・下端も対称に同じだけ縮むので四辺とも同様)。
/// これが 640x360 comp で外周 1996 画素(`2*(640+360)-4`)が alpha=0 になっていた
/// 直接の機序 —「comp 実内容と無関係」「非乱数的」「厳密に外周1周ぶん」という
/// 観測はすべてこの一様スケールで説明がつく(回帰試験:
/// `tests/background.rs::opaque_background_leaves_no_transparent_border_pixels`)。
///
/// 背景 layer に要る性質は「実 layer より必ず奥」だけで、`i16` の理論最小値である
/// 必要はない。実 layer の `order` は今のところ `LayerId` 由来の小さい非負整数
/// (`motolii-shell::Message::AddLayer`/`admit` の `order: id.0 as i16` が唯一の
/// 発生源)なので、`-1` で十分かつ厳密に安全 — このスケール式での縮みは
/// `half_dimension_px * f32eps * 1` で、8K(半幅 3840px)でも `3840 * 2^-23 ≈ 0.00046px`
/// と機械精度未満(0.5px 閾値の千倍以上小さい)。
pub(crate) const BACKGROUND_ORDER: i16 = -1;

/// 観測視点(裁定157) — 作業用の見る位置。**Document には乗らない** —
/// `Composition.camera`(裁定113/115/116、`view.resolve_camera` が読むレンダリング
/// カメラ)とは別物で、意味を持たない純表示状態(縫い目調査
/// `docs/reviews/2026-08-21-camera-seam-survey.md` §3 の「表示専用・Document 非搭載」
/// precedent と同格)。
///
/// **最小の型**: z=0 平面上のパン(`center`と同じ単位・意味、`motolii_core::ResolvedCamera`
/// 参照)+ ズームのみ(裁定113: 世界1つ・z=0 既定)。roll は持たない — 観測視点に
/// ひねりを入れる要求がまだ無く、要る設計になれば `ResolvedCamera` と同じ形へ拡張できる。
/// 3D 軌道(向きの回転)は裁定115 が「まだ開けない」と留保した領域なので今回は持ち込まない。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObservationCamera {
    /// comp 中心からのパン量(ピクセル、world 単位)。`ResolvedCamera::center` と同じ規約。
    pub pan: [f32; 2],
    /// 1.0 が既定。`ResolvedCamera::zoom` と同じ規約(値が大きいほど拡大)。
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
    /// `motolii_core::camera_projection` へそのまま渡せる形へ写す。roll は持たない
    /// ので常に 0 度(基準姿勢のまま、`ResolvedCamera::default().roll_degrees` と同じ)。
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
    /// 素材 → GPU texture。同じ素材を毎フレーム上げ直さない。
    /// **静止した素材だけ**が入る(実素材は時刻ごとに絵が違うので下の `frames` へ)。
    textures: HashMap<LayerSource, GpuTexture2D>,
    /// パス → probe 結果。probe は ffprobe のプロセス起動なので毎フレームは回さない。
    probes: HashMap<String, MediaInfo>,
    /// (パス, フレーム番号)→ GPU texture。**上限つき**。
    ///
    /// 上限が要る理由: 3〜5分の MV は 1080p30 で 5,400〜9,000フレームある。
    /// 溜め込むと 1フレーム 3MB(YUV420)× 9,000 = 約 27GB になり、書き出しが
    /// メモリで死ぬ。順次走査で要るのは直近の数枚だけなので、それを超えたら古い順に捨てる。
    /// (試験 `long_export_does_not_accumulate_frames` がこの上限を守らせる)
    ///
    /// **暫定**: フレームごとに reader を開き直している。書き出しのような順次走査では
    /// これは無駄で、本来は1本の reader を進めるべき。UI が付いて「どう走査するか」が
    /// 決まってから直す(先に最適化すると、決まっていない走査順に合わせた形になる)。
    frames: HashMap<(String, i64), GpuTexture2D>,
    /// `frames` の投入順。古い順に捨てるためだけに持つ。
    frame_order: VecDeque<(String, i64)>,
    /// text layer → GPU texture。**`textures`(`LayerSource` 鍵)を再利用しない**
    /// ([`TextCacheKey`] の doc 参照——`LayerSource::Text` は中身を持たない unit
    /// variant なので、複数の text layer がそのまま使うと1つの鍵に衝突する)。
    text_textures: HashMap<TextCacheKey, GpuTexture2D>,
    /// shape layer → GPU texture。`text_textures` と同じ理由で `textures`
    /// (`LayerSource` 鍵)を再利用しない——`LayerSource::Shape` も中身を持たない
    /// unit variant なので、複数の shape layer が同じ鍵に衝突する
    /// ([`ShapeCacheKey`] の doc 参照)。
    shape_textures: HashMap<ShapeCacheKey, GpuTexture2D>,
    /// probe に失敗した path → 理由(表示用文字列)。**`probes`(成功キャッシュ)と
    /// 対称のキャッシュ**——素材が壊れている/削除されている間、再生中は毎フレーム
    /// この layer の `texture_for` が呼ばれる。キャッシュが無いと壊れた素材1つで
    /// 毎フレーム ffprobe プロセスを起動し続けることになる(`probes` のフィールド doc
    /// が説明する「probe は毎フレーム回さない」規律の裏返し)。
    failed_probes: HashMap<String, String>,
    /// **A05(`next/reference/axis/A05-missing.tsv`)**: 直近の `render_frame`/
    /// `render_frame_to_texture` 系呼び出し1回ぶんで、`texture_for` の
    /// `LayerSource::Media` 枝が probe/decode 失敗を隔離した layer の理由。
    ///
    /// 呼び出しのたび(`render_with_camera_override`/`layers_from_resolved` の
    /// 冒頭)に空へ戻す——「前フレームの理由を今のフレームのように見せない」
    /// ため(Q0: 実データの無い状態を実データのように見せない、の裏返し)。
    /// 黙って握りつぶさない(Q3)ための読み出し口は [`Self::layer_failures`]。
    ///
    /// **shell 側の表示配線はまだ無い**(この crate の write-set 外——本 doc の
    /// 発注 RETURN 参照)。理由を UI の帯へ実際に出すには `motolii-shell` 側が
    /// 毎フレームこれを読んで `self.status` 等へ書く配線が別途要る。
    layer_failures: Vec<String>,
    /// パス → `probe_container` 結果。[`Self::media_frames`]/[`Self::media_duration`]
    /// が使う——`probes`(上記、video専用 `MediaInfo`、texture 生成の hot path)とは
    /// **別のキャッシュ**。理由: `motolii_media::probe`(`probes` を埋める方)は
    /// 先頭 video stream を要求し、audio-only ファイルで必ず `Err` になる
    /// (裁定274「仕様の穴ではなくバグ」)。`probe_container` は video の有無を
    /// 問わず成功する(container 内の全 stream を列挙するだけ)ので、こちらを
    /// 総フレーム数/尺の問い合わせの正本にする。
    containers: HashMap<String, ContainerInfo>,
    /// `probe_container` に失敗した path → 理由。`failed_probes` と対称
    /// (壊れた素材で毎回 ffprobe を起動し直さないため)。
    failed_containers: HashMap<String, String>,
    /// パス → 点群の幾何(`motolii_media::load_point_cloud` 結果)。`probes` と対称
    /// (parse は毎フレーム回さない)——`point_cloud_texture_for` 参照。
    point_clouds: HashMap<String, PointCloudData>,
    /// 点群の読み込みに失敗した path → 理由。`failed_probes` と対称。
    failed_point_clouds: HashMap<String, String>,
    /// (パス, comp幅, comp高さ) → GPU texture。点群は時刻非依存(`frames` と違い
    /// フレーム番号を鍵に含めない)——comp resize でだけ再描画する
    /// (`point_cloud_texture_for` の鍵の doc 参照)。
    point_cloud_textures: HashMap<(String, u32, u32), GpuTexture2D>,
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            frames: HashMap::new(),
            frame_order: VecDeque::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            point_cloud_textures: HashMap::new(),
        })
    }

    /// **裁定171 v2(M4、supervisor 裁定でこのメソッドを additive 許可)**。iced
    /// 側の device/queue の上に Engine を組む第二コンストラクタ——
    /// [`Self::new`](headless)は無改造。実体は
    /// `Compositor::with_device_using_headless_defaults`(`headless()` と同じ
    /// format/config、モジュール doc 参照)の薄いラッパーで、decode/upload
    /// キャッシュ(`textures`/`probes`/`frames`/`frame_order`)は**この Engine
    /// インスタンス自身が新しく持つ**——呼び出し側(`motolii-shell` の presenter
    /// Pipeline)が `Engine::new()` の headless インスタンスと取り違えて共有する
    /// ことはない(構造的に別インスタンス)。
    pub fn gpu_device(&self) -> &wgpu::Device {
        self.compositor.device()
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::with_device_using_headless_defaults(device, queue)?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            frames: HashMap::new(),
            frame_order: VecDeque::new(),
            text_textures: HashMap::new(),
            shape_textures: HashMap::new(),
            failed_probes: HashMap::new(),
            layer_failures: Vec::new(),
            containers: HashMap::new(),
            failed_containers: HashMap::new(),
            point_clouds: HashMap::new(),
            failed_point_clouds: HashMap::new(),
            point_cloud_textures: HashMap::new(),
        })
    }

    /// **唯一の評価経路**。Document のある時刻の姿を1枚の RGBA8 にする。
    ///
    /// **comp を引数で取らない**。取れると preview と export が違う解像度を渡せてしまい、
    /// 「評価経路が1本」が入力の一致に依存する保証に落ちる(2026-08-20 の敵対的レビュー)。
    /// 解像度も fps も Document が持つ。
    ///
    /// **export は常にこれを呼ぶ**(背景込み)。preview は市松 OFF の間もこれを呼ぶが、
    /// 市松 ON の間だけ [`Self::render_frame_without_background`] へ切り替わる
    /// (裁定141、呼び分けは `motolii-shell` 側の仕事 — この crate は市松を知らない)。
    pub fn render_frame(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<Vec<u8>, EngineError> {
        self.render(view, t, true)
    }

    /// **A05 隔離の読み出し口**(Q3: 黙って握りつぶさない)。直近の
    /// `render_frame`/`render_frame_without_background`/`render_frame_with_view_camera`/
    /// `render_frame_to_texture`/`render_resolved_to_texture*` 呼び出し1回ぶんで、
    /// `Media` layer の probe/decode 失敗を「このlayerだけ落とす」形で隔離した理由の
    /// 一覧(空なら1件も隔離していない)。
    ///
    /// 呼ぶたびに前フレームの内容を上書きする(`render_with_camera_override`/
    /// `layers_from_resolved` の冒頭で `clear` する)——呼び出し側が毎フレーム
    /// これを読まなくても内容が際限なく増え続けることはない。
    ///
    /// **この crate のどのレンダリング API 呼び出しからも自動で UI へは届かない**
    /// ——読み出すかどうかは呼び手(`motolii-shell`)の仕事。現時点では
    /// `motolii-shell` 側の配線はまだ無い(write-set 外、本発注 RETURN 参照)。
    pub fn layer_failures(&self) -> &[String] {
        &self.layer_failures
    }

    /// 市松「AE型の透明可視化モード」専用の入力(裁定141)。[`Self::render_frame`]と
    /// **同じ合成器・同じ層**を使い、`Composition.background` の pinned layer だけを
    /// 省く — 第二 render パスではなく、同一合成器への入力差分として実装してある
    /// (裁定141「同一合成器へ『背景を敷かない』入力を渡す可視化モードと整理する」)。
    ///
    /// **export はこの口を使わない**。背景 layer が無い分、層に覆われていない画素は
    /// 合成器の clear 色(`motolii-compositor::render_with_timing` が渡す
    /// `Rgba::TRANSPARENT`、`blend_with_background: Premultiplied` で alpha も
    /// 素通しになる)がそのまま出る — つまり alpha=0(回帰試験:
    /// `tests/background.rs::render_frame_without_background_leaves_uncovered_pixels_transparent`)。
    pub fn render_frame_without_background(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<Vec<u8>, EngineError> {
        self.render(view, t, false)
    }

    /// 観測視点(裁定157)で描く第二エントリ。[`Self::render_frame`]と**同じ合成器・
    /// 同じ層構築**を使い、camera だけ Document のレンダリングカメラ
    /// (`view.resolve_camera(t)`)ではなく `observation` から組む — 第二 render パス
    /// ではなく同一合成器への入力差分として実装してある([`Self::render_frame_without_background`]
    /// が裁定141 でやったのと同型)。
    ///
    /// **export はこの口を一切知らない**。呼び手は shell の Stage 表示だけを想定する
    /// (縫い目調査 `docs/reviews/2026-08-21-camera-seam-survey.md` §3)。[`Self::render_frame`]
    /// の2引数固定シグネチャ・実装は本メソッド追加で1文字も変わっていない —
    /// export/refresh_frame の呼び手2箇所は今まで通り `render_frame` だけを呼び続けられる。
    pub fn render_frame_with_view_camera(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        observation: &ObservationCamera,
    ) -> Result<Vec<u8>, EngineError> {
        self.render_with_camera_override(view, t, true, Some(observation.as_resolved_camera()))
    }

    /// `render_frame`/`render_frame_without_background` の共通実装。
    /// `include_background` だけが分岐点(背景 pinned layer を足すかどうか) —
    /// それ以外の層の組み立て・合成呼び出しは完全に同じ経路を通る。
    ///
    /// camera の決め方だけ [`Self::render_with_camera_override`] へさらに委譲する
    /// (camera_override 無し = 常に Document のレンダリングカメラを読む、今までと
    /// 完全に同じ挙動)。
    fn render(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
    ) -> Result<Vec<u8>, EngineError> {
        self.render_with_camera_override(view, t, include_background, None)
    }

    /// 素材の texture と、その実寸を返す。
    ///
    /// texture が `None` = **この時刻にこの layer は無い**(素材の外)。
    /// エラーではないので、フレーム全体を落とさずにこの layer だけ描かない。
    /// 抱えるフレーム数の上限。
    ///
    /// 大きさの根拠は「直近の数枚」ではなく **同時に描ける media layer の枚数**である。
    /// これより小さいと、layer 数がこれを超えた瞬間に毎フレーム全 evict になり
    /// **1フレームあたり layer 数ぶんの ffmpeg 起動**が走る。捨てる順も FIFO では
    /// 「最初に描く layer から捨てる」= 最悪順になるので、**最後に触った物を残す**。
    pub const FRAME_CACHE_LIMIT: usize = 64;

    /// 実測用。抱えているフレーム数。
    pub fn cached_frame_count(&self) -> usize {
        self.frames.len()
    }

    /// 実測用。抱えている text texture の枚数([`TextCacheKey`] 鍵の数)——
    /// 「同じ内容は再利用し、内容が変わったら増える」ことをテストが数で確かめられる
    /// ようにするための窓口(`cached_frame_count` と同じ形)。
    pub fn cached_text_texture_count(&self) -> usize {
        self.text_textures.len()
    }

    /// 実測用。抱えている shape texture の枚数([`ShapeCacheKey`] 鍵の数)——
    /// `cached_text_texture_count` と同じ形の窓口。
    pub fn cached_shape_texture_count(&self) -> usize {
        self.shape_textures.len()
    }

    /// 素材の総フレーム数(トリムの壁、裁定272:
    /// `source_in + duration × speed ≦ 総フレーム数`)。front から聞く口が無かった
    /// (`probes` が private だった)ので、これがその口——`docs/reviews/2026-08-28-current-position.md`
    /// 「★ 次の一手」の #1。
    ///
    /// **単位は素材ネイティブの fps で数えたフレーム番号**——`crate::texture` の
    /// `LayerSource::Media` 分岐が `info.nb_frames` をそのまま `LayerTiming::source_frame`
    /// の出力と比較している既存の実装(`texture.rs` 参照)と同じ単位にわざと揃えた。
    /// comp fps と素材 fps が食い違う場合にこの比較が厳密に正しいかは、この関数を
    /// 足す前から存在する別論点(EVIDENCE_GAP、報告に記載)。
    ///
    /// video stream を持たない素材(audio-only)は `None`——フレーム数という概念が
    /// そもそも無い(裁定274: これはバグではない。尺は [`Self::media_duration`] で取る)。
    /// probe に失敗した場合(壊れている/存在しない)も `None`。
    pub fn media_frames(&mut self, path: &str) -> Option<i64> {
        self.container_probe(path)?
            .video_streams
            .first()
            .and_then(|stream| stream.nb_frames)
    }

    /// 素材の総尺。video/audio を問わず、container が持っていれば返す
    /// (`ContainerInfo::duration` は format level で決まり、audio-only でも入る——
    /// `motolii_media::probe_container` の doc 参照)。
    ///
    /// **既知バグの修正**(裁定274 (3)): `motolii_media::probe`(video専用、`probes`
    /// キャッシュが使う方)は先頭 video stream を要求するので audio-only ファイルを
    /// 開くと必ず `Err` になる。ここでは代わりに `probe_container` を使う——video の
    /// 有無を問わず container 内の全 stream を列挙するだけなので、audio-only でも
    /// 成功する。soundtrack として貼る音声ファイル(2026-08-18 裁定)の尺はここから取る。
    pub fn media_duration(&mut self, path: &str) -> Option<motolii_core::RationalTime> {
        self.container_probe(path)?.duration
    }

    /// [`Self::media_frames`]/[`Self::media_duration`] の共有キャッシュ経路。
    /// `probes`(video decode 用、`texture.rs` の doc 参照)と同じ理由でキャッシュする
    /// ——ffprobe はプロセス起動なので、front が壁の判定のたびに毎回叩かない。
    ///
    /// **`probes` とは別プロセス起動になる**(video ファイルは `probe()`/`probe_container()`
    /// を1回ずつ、計2回 ffprobe を叩く)——video decode の hot path(`texture.rs`、
    /// 毎フレーム呼ばれ得る)と、素材取り込み時に一度だけ聞かれるこの口とで、
    /// 更新のタイミングも呼び手も違うため、キャッシュを共有すると片方の呼び出し順に
    /// もう片方が引きずられる。ファイルごとに一度きりのコストなので今は分けたままにする。
    fn container_probe(&mut self, path: &str) -> Option<ContainerInfo> {
        if let Some(info) = self.containers.get(path) {
            return Some(info.clone());
        }
        if self.failed_containers.contains_key(path) {
            return None;
        }
        match motolii_media::probe_container(path) {
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
