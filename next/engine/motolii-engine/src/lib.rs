//! wraps: motolii-store + motolii-compositor — **1フレームを出す唯一の経路**。
//!
//! 背骨2: preview も export も [`Engine::render_frame`] を呼ぶ。窓の有無だけが違う。
//! ここに「書き出し専用の速い道」を足さない — 足した瞬間に「見た絵 ≠ 出る絵」が生まれる。
//!
//! この crate 自身は意味を持たない。Document の意味は `motolii-store`、
//! 補間は `motolii-eval`、描画は `re_renderer` にある。ここは繋ぐだけ。

use std::collections::{HashMap, VecDeque};

use motolii_compositor::GpuTexture2D;
use motolii_compositor::{CompSpec, Compositor, CompositorError, Layer};
use motolii_media::{probe, read_frame_at, MediaError, MediaInfo};
use motolii_store::{LayerSource, RationalTime, StoreView};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Compositor(#[from] CompositorError),
    #[error(transparent)]
    Media(#[from] MediaError),
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
    pub fn render_frame(
        &mut self,
        view: &StoreView<'_>,
        t: RationalTime,
        comp: CompSpec,
    ) -> Result<Vec<u8>, EngineError> {
        let resolved = view.resolved_layers(t);

        let mut layers = Vec::with_capacity(resolved.len());
        for layer in resolved {
            let (texture, natural) = self.texture_for(&layer.source, t)?;
            // track も declared も無い軸は素材の実寸で埋める(AE と同じ「キーを
            // 打っていない property は静止値」の延長)。
            let size = [
                if layer.size[0] > 0.0 { layer.size[0] } else { natural[0] },
                if layer.size[1] > 0.0 { layer.size[1] } else { natural[1] },
            ];
            layers.push(Layer {
                texture,
                top_left: layer.top_left,
                size,
                order: layer.order,
                opacity: layer.opacity,
            });
        }

        Ok(self.compositor.render(comp, &layers)?)
    }

    /// 素材の texture と、その実寸を返す。
    /// 抱えるフレーム数の上限。順次走査で要るのは直近の数枚だけ。
    const FRAME_CACHE_LIMIT: usize = 8;

    /// 実測用。抱えているフレーム数。
    pub fn cached_frame_count(&self) -> usize {
        self.frames.len()
    }

    fn remember_frame(&mut self, key: (String, i64), texture: GpuTexture2D) {
        if self.frames.insert(key.clone(), texture).is_none() {
            self.frame_order.push_back(key);
        }
        while self.frame_order.len() > Self::FRAME_CACHE_LIMIT {
            if let Some(oldest) = self.frame_order.pop_front() {
                self.frames.remove(&oldest);
            }
        }
    }

    fn texture_for(
        &mut self,
        source: &LayerSource,
        t: RationalTime,
    ) -> Result<(GpuTexture2D, [f32; 2]), EngineError> {
        match source {
            LayerSource::Solid {
                rgba,
                width,
                height,
            } => {
                let natural = [*width as f32, *height as f32];
                if let Some(texture) = self.textures.get(source) {
                    return Ok((texture.clone(), natural));
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
                Ok((texture, natural))
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

                // comp 時刻 → 素材のフレーム番号。素材の尺を超えたら最終フレームで止める
                // のではなく、**素材が無い時刻は描かない**(フリーズフレーム禁止、M4)。
                let frame = (t.num() as f64 / t.den() as f64
                    * info.fps.num() as f64
                    / info.fps.den() as f64)
                    .floor() as i64;
                let frame = frame.max(0);

                let key = (path.clone(), frame);
                if let Some(texture) = self.frames.get(&key) {
                    return Ok((texture.clone(), natural));
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
                Ok((texture, natural))
            }
        }
    }
}
