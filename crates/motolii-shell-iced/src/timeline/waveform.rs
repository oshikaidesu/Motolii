//! soundtrack の波形帯の生成座席 — egui 版
//! (`motolii_ui::timeline_editor::waveform_band::WaveformSeat`)の移植。
//!
//! **縮約の意味(段 mip・O(見えている px))は移植ではなく共用**である:
//! [`WaveformPeaks`](motolii_ui::timeline_editor::waveform_band::WaveformPeaks) と
//! [`WaveformWindow`] は egui 版が `pub` で出している同じ型をそのまま使う。
//! ここに在るのは「decode と縮約を別 thread へ出し、届くまで placeholder」という
//! **座席の段取りだけ**で、egui 版 `WaveformSeat`(`pub(crate)` なので届かない)を
//! 同じ形に写した。decode の経路も同じ
//! (`decode_file_audio_ordinal` → `to_canonical` — 別の decoder を作らない)。
//!
//! ## iced 側の違いは「いつ poll するか」だけ
//!
//! egui 版は描画関数の中から毎フレーム poll する。iced の draw は `&self` なので
//! そこから座席を進められない — poll は `Shell::update`(Message パス)が行い、
//! 生成中だけ `window::frames()` の購読が `Message::WaveformPolled` を刻む
//! (書き出しの `ExportPolled` と同じ型の解決)。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;

use motolii_audio::{decode_file_audio_ordinal, to_canonical, PcmCache};
use motolii_doc::{resolve_asset_path, Document};
use motolii_ui::timeline_editor::waveform_band::WaveformPeaks;

/// soundtrack の PCM stream は必ず先頭の1本(egui 版 `SOUNDTRACK_ORDINAL` と同値)。
const SOUNDTRACK_ORDINAL: u32 = 0;

/// いま帯が写している soundtrack の身元。**変わったら作り直す。**
#[derive(Debug, Clone, PartialEq, Eq)]
struct SoundtrackId {
    content_hash: String,
    path: PathBuf,
}

/// 別 thread から戻ってくる荷物(egui 版 `Built` と同じ)。
struct Built {
    decoded: Option<Arc<PcmCache>>,
    outcome: Result<WaveformPeaks, String>,
}

/// 帯として描けるもの。**canvas はこれを読むだけ**(座席は Shell が進める)。
#[derive(Debug, Clone, Default)]
pub enum WaveformBandState {
    /// soundtrack が無い。帯ごと出さない。
    #[default]
    Absent,
    /// 生成中。薄い placeholder を出す(フレームは止めない)。
    Building,
    Ready(Arc<WaveformPeaks>),
    /// 帯は出すが、出せない理由を書く(黙って空の帯にしない)。
    Unavailable(String),
}

/// 波形の生成座席。decode と縮約を別 thread へ出し、届いたら控える。
#[derive(Default)]
pub struct WaveformSeat {
    id: Option<SoundtrackId>,
    pending: Option<Receiver<Built>>,
    peaks: Option<Arc<WaveformPeaks>>,
    failure: Option<String>,
}

impl WaveformSeat {
    /// 座席を1歩進める。**decode でフレームを止めない。**
    ///
    /// `caches` は `(content_hash, ordinal) → 正準PCM` の控え。egui 版と同じく、
    /// 既に居れば decode せずそれを畳み、新しく decode したらここへ入れて返す。
    pub fn poll(
        &mut self,
        document: &Document,
        project_root: Option<&Path>,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> WaveformBandState {
        let Some(soundtrack) = &document.soundtrack else {
            self.clear();
            return WaveformBandState::Absent;
        };
        let Some(asset) = document.assets.get(soundtrack.asset) else {
            return self.settle_on(None, "soundtrack asset missing");
        };
        let Some(path) = resolve_asset_path(asset, project_root) else {
            return self.settle_on(
                Some(SoundtrackId {
                    content_hash: asset.content_hash.to_string(),
                    path: PathBuf::new(),
                }),
                "soundtrack asset path unresolved",
            );
        };
        let id = SoundtrackId {
            content_hash: asset.content_hash.to_string(),
            path,
        };
        if self.id.as_ref() != Some(&id) {
            self.clear();
            self.id = Some(id.clone());
        }

        if let Some(peaks) = &self.peaks {
            return WaveformBandState::Ready(Arc::clone(peaks));
        }
        if let Some(reason) = &self.failure {
            return WaveformBandState::Unavailable(reason.clone());
        }
        if self.pending.is_some() {
            return self.receive(&id, caches);
        }
        self.start(&id, caches);
        WaveformBandState::Building
    }

    /// まだ荷物を待っているか(購読を刻み続けるかの判断)。
    pub fn is_building(&self) -> bool {
        self.pending.is_some()
    }

    fn start(&mut self, id: &SoundtrackId, caches: &HashMap<(String, u32), Arc<PcmCache>>) {
        let cached = caches
            .get(&(id.content_hash.clone(), SOUNDTRACK_ORDINAL))
            .map(Arc::clone);
        let path = id.path.clone();
        let (tx, rx) = channel();
        // 送り先が居なくなっていても静かに終わる(egui 版と同じ)。
        std::thread::spawn(move || {
            let built = match cached {
                Some(pcm) => Built {
                    decoded: None,
                    outcome: WaveformPeaks::build(&pcm),
                },
                None => match decode_canonical(&path) {
                    Ok(pcm) => Built {
                        outcome: WaveformPeaks::build(&pcm),
                        decoded: Some(pcm),
                    },
                    Err(error) => Built {
                        decoded: None,
                        outcome: Err(error),
                    },
                },
            };
            let _ = tx.send(built);
        });
        self.pending = Some(rx);
    }

    fn receive(
        &mut self,
        id: &SoundtrackId,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> WaveformBandState {
        let Some(rx) = &self.pending else {
            return WaveformBandState::Building;
        };
        match rx.try_recv() {
            Err(TryRecvError::Empty) => WaveformBandState::Building,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.failure = Some("waveform build stopped".to_owned());
                WaveformBandState::Unavailable("waveform build stopped".to_owned())
            }
            Ok(built) => {
                self.pending = None;
                if let Some(pcm) = built.decoded {
                    caches.insert((id.content_hash.clone(), SOUNDTRACK_ORDINAL), pcm);
                }
                match built.outcome {
                    Ok(peaks) => {
                        let peaks = Arc::new(peaks);
                        self.peaks = Some(Arc::clone(&peaks));
                        WaveformBandState::Ready(peaks)
                    }
                    Err(error) => {
                        self.failure = Some(error.clone());
                        WaveformBandState::Unavailable(error)
                    }
                }
            }
        }
    }

    fn settle_on(&mut self, id: Option<SoundtrackId>, reason: &str) -> WaveformBandState {
        if self.id != id {
            self.clear();
            self.id = id;
            self.failure = Some(reason.to_owned());
        }
        WaveformBandState::Unavailable(reason.to_owned())
    }

    fn clear(&mut self) {
        self.id = None;
        self.pending = None;
        self.peaks = None;
        self.failure = None;
    }
}

/// egui 版と同じ decode 経路。別の decoder を作らない。
fn decode_canonical(path: &Path) -> Result<Arc<PcmCache>, String> {
    let raw =
        decode_file_audio_ordinal(path, SOUNDTRACK_ORDINAL).map_err(|error| error.to_string())?;
    let canonical = to_canonical(&raw).map_err(|error| error.to_string())?;
    Ok(Arc::new(canonical))
}
