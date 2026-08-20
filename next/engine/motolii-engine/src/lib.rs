//! wraps: motolii-store + motolii-compositor — **1フレームを出す唯一の経路**。
//!
//! 背骨2: preview も export も [`Engine::render_frame`] を呼ぶ。窓の有無だけが違う。
//! ここに「書き出し専用の速い道」を足さない — 足した瞬間に「見た絵 ≠ 出る絵」が生まれる。
//!
//! この crate 自身は意味を持たない。Document の意味は `motolii-store`、
//! 補間は `motolii-eval`、描画は `re_renderer` にある。ここは繋ぐだけ。

use std::collections::{HashMap, VecDeque};

use motolii_compositor::GpuTexture2D;
use motolii_compositor::{Compositor, CompositorError, Layer};

use motolii_media::{probe, read_frame_at, MediaError, MediaInfo};
use motolii_store::{LayerSource, RationalTime, StoreView};

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

    /// **唯一の評価経路**。Document のある時刻の姿を1枚の RGBA8 にする。
    ///
    /// **comp を引数で取らない**。取れると preview と export が違う解像度を渡せてしまい、
    /// 「評価経路が1本」が入力の一致に依存する保証に落ちる(2026-08-20 の敵対的レビュー)。
    /// 解像度も fps も Document が持つ。
    pub fn render_frame(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
    ) -> Result<Vec<u8>, EngineError> {
        let comp = view
            .composition()
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or(EngineError::NoComposition)?
            .spec();
        let resolved = view
            .resolved_layers(t)
            .map_err(|e| EngineError::Store(e.to_string()))?;

        let mut layers = Vec::with_capacity(resolved.len());
        for layer in resolved {
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
            });
        }

        Ok(self.compositor.render(comp, &layers)?)
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
