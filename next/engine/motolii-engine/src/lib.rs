//! wraps: motolii-store + motolii-compositor — **1フレームを出す唯一の経路**。
//!
//! 背骨2: preview も export も [`Engine::render_frame`] を呼ぶ。窓の有無だけが違う。
//! ここに「書き出し専用の速い道」を足さない — 足した瞬間に「見た絵 ≠ 出る絵」が生まれる。
//! **唯一の例外**: [`Engine::render_frame_without_background`](市松の透明可視化専用、
//! 裁定141)。export は絶対に使わない。preview だけが市松 ON の間に切り替える口で、
//! 同じ合成器・同じ層構築を共有する差分入力として実装してある(第二経路ではない)。
//!
//! この crate 自身は意味を持たない。Document の意味は `motolii-store`、
//! 補間は `motolii-eval`、描画は `re_renderer` にある。ここは繋ぐだけ。

use std::collections::{HashMap, VecDeque};

use motolii_compositor::GpuTexture2D;
use motolii_compositor::{Compositor, CompositorError, Layer};

use motolii_media::{probe, read_frame_at, MediaError, MediaInfo};
use motolii_store::{LayerPlacement, LayerSource, Matte, RationalTime, StoreView};

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

    /// `render_frame`/`render_frame_without_background` の共通実装。
    /// `include_background` だけが分岐点(背景 pinned layer を足すかどうか) —
    /// それ以外の層の組み立て・合成呼び出しは完全に同じ経路を通る。
    fn render(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        include_background: bool,
    ) -> Result<Vec<u8>, EngineError> {
        let composition = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?;
        let comp = composition.spec();
        // カメラも comp と同じく Document が持つ(裁定113/115)。preview/export が
        // 違うカメラを渡せないよう、ここでも引数ではなく `view` から読む
        // (裁定40 が comp について立てた規律と同じ形)。
        let camera = view
            .resolve_camera(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let mut layers = Vec::with_capacity(resolved.len() + 1);

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
            layers.push(Layer {
                texture: background_texture.expect("LayerSource::Solid は常に texture を返す"),
                size: [comp.width as f32, comp.height as f32],
                placement: LayerPlacement {
                    order: BACKGROUND_ORDER,
                    ..Default::default()
                },
                pinned: true,
                blend_mode: motolii_compositor::BlendMode::Normal,
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
            layers.push(Layer {
                texture,
                size,
                // **置き方はそのまま持ち回る** — 並べ直すとそこが翻訳層になる。
                placement: layer.placement,
                pinned: layer.pinned,
                blend_mode,
            });
        }

        Ok(self.compositor.render(comp, camera, &layers)?)
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
            // layer-meta 束が足した3 variant。**まだ描画に繋いでいない** — null layer は
            // 元々絵を持たず(裁定どおり)、shape/text は演算子/組版から RGBA を焼く経路が
            // 未実装(`motolii-vector::render` はまだ engine から呼ばれていない)。
            // texture 無し = 「この layer は今描かない」という既存の意味(素材の外の
            // 時刻と同じ扱い)に乗せてあるので、フレーム全体は落ちない。
            LayerSource::Null | LayerSource::Shape | LayerSource::Text => Ok((None, [0.0, 0.0])),
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

/// `motolii_store::BlendMode`(Document の16値、裁定67)を
/// `motolii_compositor::BlendMode`(合成器が固定式 blend equation で表現できる分だけ、
/// `motolii-compositor` のモジュール doc 参照)へ写す。対応外は
/// [`EngineError::UnsupportedBlendMode`] — 黙って `Normal` へ近似しない。
fn translate_blend_mode(
    mode: motolii_store::BlendMode,
) -> Result<motolii_compositor::BlendMode, EngineError> {
    match mode {
        motolii_store::BlendMode::Normal => Ok(motolii_compositor::BlendMode::Normal),
        other => Err(EngineError::UnsupportedBlendMode(other)),
    }
}
