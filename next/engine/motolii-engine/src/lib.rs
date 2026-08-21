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

use motolii_compositor::GpuTexture2D;
use motolii_compositor::{Compositor, CompositorError, Layer, LayerWithPasses};
use motolii_core::{CompSpec, ResolvedCamera};

use motolii_media::{probe, read_frame_at, MediaError, MediaInfo};
use motolii_store::{LayerPlacement, LayerSource, Matte, RationalTime, ResolvedLayer, StoreView};

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
    /// `motolii_store::BlendMode` の16値のうち、合成器がまだ表現できない分
    /// (`motolii_compositor::BlendMode` のモジュール doc 参照 — 固定式の
    /// blend equation では出せず fork 改造が要る)。黙って Normal へ近似しない。
    #[error("blend mode {0:?} はまだ合成器が対応していない(Normal のみ対応。fork 改造候補)")]
    UnsupportedBlendMode(motolii_store::BlendMode),
    /// matte はまだ合成器に繋いでいない。単一 `TexturedRect` は自分の texture 1枚しか
    /// 持てず、matte 元レイヤーの alpha/luma を per-pixel で参照するには2枚目の texture
    /// を読む shader が要る(`rectangle_fs.wgsl` は現状1枚専用) — fork seam 候補。
    /// 黙って型抜き前の絵を出すよりは、ここで明示的に止める。
    #[error("matte はまだ合成器が対応していない({0:?}。fork seam 候補)")]
    UnsupportedMatte(Matte),
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
const BACKGROUND_ORDER: i16 = -1;

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
}

impl Engine {
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::headless()?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            frames: HashMap::new(),
            frame_order: VecDeque::new(),
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
    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self, EngineError> {
        Ok(Self {
            compositor: Compositor::with_device_using_headless_defaults(device, queue)?,
            textures: HashMap::new(),
            probes: HashMap::new(),
            frames: HashMap::new(),
            frame_order: VecDeque::new(),
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

    /// [`Self::render`]の実体。`camera_override` が `Some` の間だけ Document の
    /// レンダリングカメラ(`view.resolve_camera(t)`)を読まず、渡された値をそのまま使う
    /// (観測視点、[`Self::render_frame_with_view_camera`]専用の分岐)。`None` の間は
    /// 今までの `render` と完全に同じ経路(`render_frame`/`render_frame_without_background`
    /// はこの分岐に触れない)。
    fn render_with_camera_override(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        // カメラも comp と同じく Document が持つ(裁定113/115)。preview/export が
        // 違うカメラを渡せないよう、ここでも引数ではなく `view` から読む
        // (裁定40 が comp について立てた規律と同じ形)。**観測視点だけが例外**
        // (`camera_override`、裁定157) — Document を一切読まず渡された値をそのまま使う。
        let camera = match camera_override {
            Some(camera) => camera,
            None => view
                .resolve_camera(t)
                .map_err(|e| EngineError::Store(e.to_string()))?,
        };
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        // `LayerWithPasses` で包んで `Compositor::render_with_effects` へ渡す(裁定153 S3、
        // S2 の注意書きどおり)。`passes` が空な layer は `render_with_effects` 内部で
        // オフスクリーンを一切作らず元の texture をそのまま使う従来コストの分岐を通るので
        // (`motolii-compositor` のモジュール doc/`tests/effects.rs` 参照)、effect を
        // 持たない layer(今のところ背景 layer を含め全部——`translate_effect_passes` は
        // 2026-08-21 時点で常に空を返す)はここを経由しても速度もアロケーションも変わらない。
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        if include_background {
            // comp の背景色(`Composition::background`、利用者要望: 黒だと気分が上がらない)。
            // **`motolii-compositor` の clear 色は変えない**(compositor は書き込み禁止の
            // 並列レーンが触っている最中)。代わりに comp 全域を覆う不透明の layer を
            // どの実 layer よりも奥(`order = BACKGROUND_ORDER`、定数の doc 参照 —
            // `i16::MIN` は depth_offset の shader 側スケールで外周1px を欠落させる
            // ので使わない)に足す — pinned layer(裁定113、カメラの pan/zoom を受けず
            // 画面に張り付く機構)を流用すれば、camera がどこを向いていても render
            // target をちょうど覆う「クリア色」として働く。
            // 既定値([0,0,0,1] 不透明黒)は旧 clear 色と同じ見た目になるので、
            // 既存テストの期待画素は変わらない(合成器の実測: `TRANSPARENT` clear は
            // 読み戻すと不透明黒になる — `motolii-compositor` の
            // `default_camera_all_z0_matches_orthographic_pixel_mapping` 参照)。
            // export は必ずこの分岐を通る([`Self::render_frame`] からしか
            // `include_background = false` は選ばれない)ので、背景も書き出しに乗る。
            let (background_texture, _) = self.texture_for(
                &LayerSource::Solid {
                    rgba: to_u8_rgba(composition.background),
                    // 1x1 で足りる — 単色は quad の `size` で comp 全域まで引き伸ばすので、
                    // texture 自体の解像度は意味を持たない。
                    width: 1,
                    height: 1,
                },
                0,
            )?;
            // 背景 layer には pass を積まない(S3 EXACT TARGET 3) — 背景は engine が
            // ここで直接組み立てる単色 pinned layer であって `ResolvedLayer` を経由しない
            // ので、そもそも effect スタックを持ち得ない。`passes: vec![]` で明示する。
            layers.push(LayerWithPasses {
                layer: Layer {
                    texture: background_texture.expect("LayerSource::Solid は常に texture を返す"),
                    size: [comp.width as f32, comp.height as f32],
                    placement: LayerPlacement {
                        order: BACKGROUND_ORDER,
                        ..Default::default()
                    },
                    pinned: true,
                    blend_mode: motolii_compositor::BlendMode::Normal,
                },
                passes: vec![],
            });
        }

        for layer in resolved {
            // matte/blend の判定を texture のアップロードより先にやる — 対応外なら
            // 無駄な ffmpeg 起動/GPU アップロードをする前に落とす(裁定108(c) 系の
            // 「素材の外なら描かない」と同じく、ここも早く判定する)。
            if let Some(matte) = layer.matte {
                return Err(EngineError::UnsupportedMatte(matte));
            }
            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            // store→compositor の effect 語彙変換(裁定153 S3、EXACT TARGET 1)。
            // texture のアップロードより前に計算しても副作用は無い(純関数)ので、
            // 上の matte/blend と同じ「早く判定する」並びに合わせてここに置く。
            let passes = translate_effect_passes(&layer.effects);

            let (texture, natural) = self.texture_for(&layer.source, layer.source_frame)?;
            let Some(texture) = texture else {
                // 素材の外の時刻。この layer は今フレームに居ない。
                continue;
            };
            // 素材の寸法は Document が知らないことがある(実素材は probe しないと
            // 分からない)。その時だけ実寸で埋める。**大きさは transform の scale で
            // 動く**ので、ここは「板のローカル矩形」を決めているだけ(裁定59)。
            let size = [
                if layer.declared_size[0] > 0.0 {
                    layer.declared_size[0]
                } else {
                    natural[0]
                },
                if layer.declared_size[1] > 0.0 {
                    layer.declared_size[1]
                } else {
                    natural[1]
                },
            ];
            layers.push(LayerWithPasses {
                layer: Layer {
                    texture,
                    size,
                    // **置き方はそのまま持ち回る** — 並べ直すとそこが翻訳層になる。
                    placement: layer.placement,
                    pinned: layer.pinned,
                    blend_mode,
                },
                passes,
            });
        }

        Ok(self.compositor.render_with_effects(comp, camera, &layers)?)
    }

    /// **裁定171 v2(M4)**: [`Self::render_with_camera_override`]の層構築
    /// (`include_background`/`for layer in resolved`のループ、上記)を
    /// **そのまま複製**した private ヘルパー。既存の
    /// [`Self::render_with_camera_override`](延いては
    /// [`Self::render_frame`]/[`Self::render_frame_without_background`]/
    /// [`Self::render_frame_with_view_camera`])は1行も触っていない
    /// (supervisor 裁定「additive のみ」)——複製の理由は、この新しいヘルパーが
    /// `&StoreView<'_>` を取らず**既に resolve 済みの所有データ**
    /// (`comp`/`background`/`camera`/`resolved: &[ResolvedLayer]`)を取ることだけ
    /// が違うため([`Self::render_resolved_to_texture`]のモジュール doc 参照 —
    /// `motolii-shell` の presenter `Primitive` は `Document`(非 `Clone`、
    /// `re_entity_db::EntityDb` が `testing` feature 外では `Clone` を持たない)
    /// を共有できないので、`StoreView` を後から作り直せない)。
    fn layers_from_resolved(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        resolved: &[ResolvedLayer],
    ) -> Result<Vec<LayerWithPasses>, EngineError> {
        let mut layers: Vec<LayerWithPasses> = Vec::with_capacity(resolved.len() + 1);

        let (background_texture, _) = self.texture_for(
            &LayerSource::Solid {
                rgba: to_u8_rgba(background),
                width: 1,
                height: 1,
            },
            0,
        )?;
        layers.push(LayerWithPasses {
            layer: Layer {
                texture: background_texture.expect("LayerSource::Solid は常に texture を返す"),
                size: [comp.width as f32, comp.height as f32],
                placement: LayerPlacement {
                    order: BACKGROUND_ORDER,
                    ..Default::default()
                },
                pinned: true,
                blend_mode: motolii_compositor::BlendMode::Normal,
            },
            passes: vec![],
        });

        for layer in resolved {
            if let Some(matte) = layer.matte {
                return Err(EngineError::UnsupportedMatte(matte));
            }
            let blend_mode = translate_blend_mode(layer.blend_mode)?;
            let passes = translate_effect_passes(&layer.effects);

            let (texture, natural) = self.texture_for(&layer.source, layer.source_frame)?;
            let Some(texture) = texture else {
                continue;
            };
            let size = [
                if layer.declared_size[0] > 0.0 {
                    layer.declared_size[0]
                } else {
                    natural[0]
                },
                if layer.declared_size[1] > 0.0 {
                    layer.declared_size[1]
                } else {
                    natural[1]
                },
            ];
            layers.push(LayerWithPasses {
                layer: Layer {
                    texture,
                    size,
                    placement: layer.placement,
                    pinned: layer.pinned,
                    blend_mode,
                },
                passes,
            });
        }

        Ok(layers)
    }

    /// **裁定171 v2(M4)— zero-copy GPU 出力、resolve 済みスナップショット版**。
    /// CPU readback を一切しない([`motolii_compositor::Compositor::render_to_texture`]
    /// をそのまま呼ぶ)。`motolii-shell` の presenter `Primitive::prepare` は
    /// `Document`(非 `Clone`)を共有できないので、`Shell::refresh_frame` が
    /// **世代が変わった時だけ**(裁定171 v2 EXACT TARGET 2 の世代ゲート)
    /// `StoreView::resolved_layers`/`resolve_camera`/`composition` から
    /// 抜き出した所有データのスナップショットをここへ渡す設計 — この関数自体は
    /// `Document`/`StoreView` を一切知らない(層は既に resolve 済み)。
    ///
    /// `include_background` は常に `true` 固定(`Self::render_frame` と同じ
    /// 「唯一の評価経路は背景込み」の規律——export 専用の
    /// `render_frame_without_background` 相当は zero-copy 経路にはまだ無い、
    /// NON-GOALS「市松の GPU 化」の裏返し。市松 ON は
    /// `motolii-shell` 側が CPU フォールバック経路(`render_frame_without_background`)
    /// へ切り替える、裁定171 v2 §0-6)。
    pub fn render_resolved_to_texture(
        &mut self,
        comp: CompSpec,
        background: [f32; 4],
        camera: ResolvedCamera,
        resolved: &[ResolvedLayer],
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let layers = self.layers_from_resolved(comp, background, resolved)?;
        Ok(self.compositor.render_to_texture(comp, camera, &layers)?)
    }

    /// [`Self::render_resolved_to_texture`]の `&StoreView<'_>` 版
    /// (`Self::render_frame`の"resolve してから渡す"部分をここでもやるだけの
    /// 薄いラッパー)。`view`/`t` から `comp`/`background`/`camera`/`resolved` を
    /// 抜き出して委譲する——`motolii-shell` の presenter は(上記の理由で)
    /// これを直接呼べない(`StoreView` を保持できない)ので、こちらは主に
    /// この crate 自身のテスト・「将来 Document を直接持てる呼び手」向けの
    /// 対称な入口として用意する。[`Self::render_frame`]/
    /// [`Self::render_with_camera_override`]は無改造 — 独立した新規メソッド。
    pub fn render_frame_to_texture(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        let camera = view
            .resolve_camera(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        self.render_resolved_to_texture(comp, composition.background, camera, &resolved)
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

    fn remember_frame(&mut self, key: (String, i64), texture: GpuTexture2D) {
        if self.frames.insert(key.clone(), texture).is_none() {
            self.frame_order.push_back(key);
        } else {
            self.touch_frame(&key);
        }
        while self.frame_order.len() > Self::FRAME_CACHE_LIMIT {
            if let Some(oldest) = self.frame_order.pop_front() {
                self.frames.remove(&oldest);
            }
        }
    }

    /// 触った物を末尾へ回す(= 捨てるのは最後に触ってから最も古い物)。
    fn touch_frame(&mut self, key: &(String, i64)) {
        if let Some(at) = self.frame_order.iter().position(|k| k == key) {
            let key = self.frame_order.remove(at).expect("position が指した要素");
            self.frame_order.push_back(key);
        }
    }

    fn texture_for(
        &mut self,
        source: &LayerSource,
        source_frame: i64,
    ) -> Result<(Option<GpuTexture2D>, [f32; 2]), EngineError> {
        match source {
            LayerSource::Solid {
                rgba,
                width,
                height,
            } => {
                let natural = [*width as f32, *height as f32];
                if let Some(texture) = self.textures.get(source) {
                    return Ok((Some(texture.clone()), natural));
                }
                let pixels: Vec<u8> = rgba
                    .iter()
                    .copied()
                    .cycle()
                    .take((width * height * 4) as usize)
                    .collect();
                let texture = self
                    .compositor
                    .upload_rgba("solid", &pixels, *width, *height)?;
                self.textures.insert(source.clone(), texture.clone());
                Ok((Some(texture), natural))
            }
            LayerSource::Media { path, .. } => {
                let info = match self.probes.get(path) {
                    Some(info) => info.clone(),
                    None => {
                        let info = probe(path)?;
                        self.probes.insert(path.clone(), info.clone());
                        info
                    }
                };
                let natural = [info.width as f32, info.height as f32];

                // **時間の計算はしない**。comp 時刻 → 素材フレームの写像は Document が
                // 持つ(`LayerTiming::source_frame`)。engine が別の写像を持つと
                // 時刻の正本が2本になる(2026-08-20 の敵対的レビューで一度やった失敗)。
                let frame = source_frame;

                // 素材の外は描かない(フリーズフレーム禁止、M4)。ここで Err を返すと
                // フレーム全体が出なくなるので、この layer だけ落とす。
                let last_frame = info.nb_frames.map(|n| n - 1);
                if frame < 0 || last_frame.is_some_and(|last| frame > last) {
                    return Ok((None, natural));
                }

                let key = (path.clone(), frame);
                if let Some(texture) = self.frames.get(&key) {
                    let texture = texture.clone();
                    self.touch_frame(&key);
                    return Ok((Some(texture), natural));
                }

                let cpu = read_frame_at(path, &info, frame)?;
                let texture = self.compositor.upload_yuv420p(
                    "media",
                    &cpu.data,
                    info.width,
                    info.height,
                    info.color_space,
                )?;
                self.remember_frame(key, texture.clone());
                Ok((Some(texture), natural))
            }
            // layer-meta 束が足した3 variant + 裁定173 の `Group`。**まだ描画に
            // 繋いでいない** — null layer は元々絵を持たず(裁定どおり)、shape/text は
            // 演算子/組版から RGBA を焼く経路が未実装(`motolii-vector::render` はまだ
            // engine から呼ばれていない)。`Group` も同じく絵を持たない(裁定173 —
            // Group は「子を持てる」という印だけの layer、合成は世界合成
            // (`motolii-store::view::world_affine`)が親 transform として使うだけで、
            // Group 自身のピクセルは無い)。texture 無し = 「この layer は今描かない」
            // という既存の意味(素材の外の時刻と同じ扱い)に乗せてあるので、フレーム
            // 全体は落ちない。
            LayerSource::Null | LayerSource::Shape | LayerSource::Text | LayerSource::Group => {
                Ok((None, [0.0, 0.0]))
            }
        }
    }
}

/// `Composition::background`([f32;4]・0.0〜1.0)を素材アップロードが取る 8bit RGBA
/// へ写す。`round` で丸める(`as u8` の単純切り捨てだと 1.0 が 254 に落ちて
/// 「不透明のつもりが微妙に透ける」事故になる)。
fn to_u8_rgba(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[1] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[2] * 255.0).round().clamp(0.0, 255.0) as u8,
        (c[3] * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}

/// `motolii_store::BlendMode`(Document の17値、裁定67 + BL2 の `Add`)を
/// `motolii_compositor::BlendMode`(合成器が表現できる分だけ、`motolii-compositor`
/// のモジュール doc 参照)へ写す。対応外(非分離4種、BL4)は
/// [`EngineError::UnsupportedBlendMode`] — 黙って `Normal` へ近似しない。
///
/// **BL3(2026-08-22)**: Multiply〜Exclusion の11値(分離可能 blend、
/// `motolii_compositor::separable_mode_index` が扱う分)を Normal/Add と同じ1対1で
/// 追加。**`_` を使わない**(全 variant を列挙)——`motolii_store::BlendMode` に
/// variant が増えた時、ここを更新し忘れるとコンパイルが落ちる。
fn translate_blend_mode(
    mode: motolii_store::BlendMode,
) -> Result<motolii_compositor::BlendMode, EngineError> {
    use motolii_compositor::BlendMode as Dst;
    use motolii_store::BlendMode as Src;
    match mode {
        Src::Normal => Ok(Dst::Normal),
        Src::Add => Ok(Dst::Add),
        Src::Multiply => Ok(Dst::Multiply),
        Src::Screen => Ok(Dst::Screen),
        Src::Overlay => Ok(Dst::Overlay),
        Src::Darken => Ok(Dst::Darken),
        Src::Lighten => Ok(Dst::Lighten),
        Src::ColorDodge => Ok(Dst::ColorDodge),
        Src::ColorBurn => Ok(Dst::ColorBurn),
        Src::HardLight => Ok(Dst::HardLight),
        Src::SoftLight => Ok(Dst::SoftLight),
        Src::Difference => Ok(Dst::Difference),
        Src::Exclusion => Ok(Dst::Exclusion),
        // 非分離4種(Hue/Saturation/Color/Luminosity、BL4)——`motolii-compositor` に
        // 対応する variant が無い、まだ実装していない。
        other @ (Src::Hue | Src::Saturation | Src::Color | Src::Luminosity) => {
            Err(EngineError::UnsupportedBlendMode(other))
        }
    }
}

#[cfg(test)]
mod translate_blend_mode_tests {
    use super::translate_blend_mode;
    use crate::EngineError;

    /// **BL2**: `Add` は `motolii-compositor` が無改造で出せる(モジュール doc
    /// 参照)ので `Ok` — `translate_effect_passes_tests` と同型の、private 関数への
    /// crate 内 unit test(`tests/` からは呼べないので colocate する)。
    #[test]
    fn add_is_accepted() {
        // `EngineError` は `PartialEq` を derive していない(`CompositorError`/
        // `MediaError` 由来の `#[from]` があるため)ので `Result` ごとの
        // `assert_eq!` はできない — `Ok` の中身だけを比較する。
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Add).unwrap(),
            motolii_compositor::BlendMode::Add
        );
    }

    /// **BL3**: 分離可能 blend の11値も同じ1対1で `Ok`(代表して Multiply/SoftLight
    /// の2つを固定 — 全11の網羅は `tests/blend_separable.rs` の数値検証が担う)。
    #[test]
    fn separable_modes_are_accepted() {
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::Multiply).unwrap(),
            motolii_compositor::BlendMode::Multiply
        );
        assert_eq!(
            translate_blend_mode(motolii_store::BlendMode::SoftLight).unwrap(),
            motolii_compositor::BlendMode::SoftLight
        );
    }

    /// 非分離4種(BL4)は依然として明示的に `Err`(黙って近似しない)。
    #[test]
    fn hue_is_still_rejected() {
        assert!(matches!(
            translate_blend_mode(motolii_store::BlendMode::Hue),
            Err(EngineError::UnsupportedBlendMode(
                motolii_store::BlendMode::Hue
            ))
        ));
    }
}

/// `motolii_store::ResolvedEffect` の列(裁定153 S1、`resolve()` が運ぶ effect スタック)を
/// `motolii_compositor::EffectPass` の列(裁定153 S2、`LayerWithPasses::passes`)へ写す。
/// `translate_blend_mode` と**同型の語彙変換**だが、失敗のさせ方は逆にしてある:
///
/// **未知 plugin_id は `Err` にしない。無音で skip する**(pass を1本も積まない)。
/// 理由 — `motolii_compositor::EffectPass` は今のところ `Identity`(絵を変えない pass、
/// 枠の正しさを固定するためだけの variant)と `Glow`(裁定153 S4、最初の実 shader pass)
/// しか持たない(`motolii_compositor::effects` モジュール doc 参照)。つまり
/// **「対応している」plugin_id は `"motolii.glow"` 1本だけ**で、それ以外は全て未知
/// である。`translate_blend_mode` のように未知を `Err` で fail-closed にすると、
/// 対応外の effect を1つでも積んだ layer が一律描画不能になる — それは
/// 「壊れているのではなくまだ描けない」という effect の実情に反する。
/// blend mode(16値のうち対応外があれば明確に壊れている)と effect
/// (対応する pass がまだ一部しか実装されていないのが今の常態)とでは
/// 「未対応」の意味が違う、というのがこの非対称の理由。
///
/// **`"motolii.glow"` → `EffectPass::Glow` が最初の対応表エントリ**(裁定153 S4)。
/// 名前つき param(`threshold`/`intensity`/`radius`)は `ResolvedEffect.params` から
/// 探す — 無い param(track を触っていない)は proof の既定値で埋める
/// (`translate_glow_params` 参照)。**型が合わない値が入っていたら pass を1本も
/// 積まない**(EXACT TARGET #2 — パニックしない。fail-closed だが `Err` にはしない、
/// この layer の他の effect や layer 自体は普通に描ける)。
///
/// パニックしない: `effects` が空でも、全 plugin_id が未知でも、param の型が
/// 壊れていても、ここは常に正常終了する。
fn translate_effect_passes(
    effects: &[motolii_store::ResolvedEffect],
) -> Vec<motolii_compositor::EffectPass> {
    effects
        .iter()
        .filter_map(|effect| match effect.plugin_id.as_str() {
            "motolii.glow" => translate_glow_params(&effect.params),
            // それ以外の plugin_id はまだ対応する pass が無い。無音で skip する
            // (= pass を積まない、`translate_blend_mode` とは非対称——上のdoc参照)。
            _ => None,
        })
        .collect()
}

/// proof(`spikes/m5-known-implementation/M5-R0/src/glow.rs`)の既定値。
/// `threshold`/`intensity` は proof のハードコード値そのまま
/// (`bright_fs` の `1.0`、`composite_fs` の `0.75`)。`radius` は proof に
/// 名前つき param が無い(5-tap のオフセットが固定 1texel/2texel)ので、
/// `motolii_compositor::effects::glow` が選んだ「`radius = 1.0` が proof の固定
/// オフセットと厳密に一致する」写像に合わせた値(`EffectPass::Glow` の doc 参照)。
const GLOW_DEFAULT_THRESHOLD: f64 = 1.0;
const GLOW_DEFAULT_INTENSITY: f64 = 0.75;
const GLOW_DEFAULT_RADIUS: f64 = 1.0;

/// `"motolii.glow"` の named param map(`effect.{id}.param.{name}` track が実在する分
/// だけ、裁定153 S1 `ResolvedEffect::params`)を `EffectPass::Glow` へ写す。
/// track の無い param は proof の既定値(上記定数)。**値はあるが型が `Value::F64`
/// でない**場合は `None` を返して pass を1本も積まない(EXACT TARGET #2、
/// `translate_effect_passes` の doc 参照)。
fn translate_glow_params(params: &[(String, motolii_store::Value)]) -> Option<motolii_compositor::EffectPass> {
    let find = |name: &str, default: f64| -> Option<f64> {
        match params.iter().find(|(param_name, _)| param_name == name) {
            Some((_, motolii_store::Value::F64(v))) => Some(*v),
            Some(_other_type) => None,
            None => Some(default),
        }
    };
    let threshold = find("threshold", GLOW_DEFAULT_THRESHOLD)?;
    let intensity = find("intensity", GLOW_DEFAULT_INTENSITY)?;
    let radius = find("radius", GLOW_DEFAULT_RADIUS)?;
    Some(motolii_compositor::EffectPass::Glow {
        threshold: threshold as f32,
        intensity: intensity as f32,
        radius: radius as f32,
    })
}

#[cfg(test)]
mod translate_effect_passes_tests {
    use super::translate_effect_passes;
    use motolii_store::ResolvedEffect;

    /// effect が無い layer は pass も無い(空 → 空)。
    #[test]
    fn no_effects_yields_no_passes() {
        assert_eq!(translate_effect_passes(&[]), Vec::new());
    }

    /// 未知 plugin_id はパニックせず無音で skip される —
    /// 2026-08-21 時点は「既知」の plugin_id が1つも無いので、これは
    /// 「何を入れても今は空になる」ことの直接固定でもある。
    #[test]
    fn unknown_plugin_id_is_skipped_silently() {
        let effects = vec![
            ResolvedEffect {
                plugin_id: "motolii.not-yet-implemented".to_owned(),
                params: vec![],
            },
            ResolvedEffect {
                plugin_id: "third-party.whatever".to_owned(),
                params: vec![],
            },
        ];
        assert_eq!(translate_effect_passes(&effects), Vec::new());
    }
}
