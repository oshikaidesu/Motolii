//! soundtrack の音声波形帯。**MV を作る人にとって波形は同期の地図そのもの**で、
//! 「どこで曲が変わるか」を見ながら印(M)を打てることがこの帯の用である。
//!
//! 帯は**ルーラの直下**に置く。時間 ↔ x の換算は `TimelineView` の1本しかなく、
//! ルーラの目盛・ロケータの段・playhead・clip の bar が全部そこから引かれている。
//! 帯を同じ換算に乗せれば、**打った印は必ず見ていた波形の真上に落ちる**。
//! 下端のナビゲータ帯は composition 全域(`0..comp`)を固定で写す別の目盛であり、
//! ズーム/パンに追従しない — 波形をそこへ入れると「見ている所」と合わなくなる。
//!
//! # 縮約
//!
//! 描画は**毎フレーム O(見えている px)** で終わらせる。そのために PCM を
//! 一度だけ段(mip)へ畳んでおく:
//!
//! - 段0 = `motolii_audio::waveform_peaks` が全域を等幅 bucket へ割った絶対値 peak
//!   (bucket 幅は約 [`WaveformPeaks::BASE_BUCKET_FRAMES`] フレーム)
//! - 段 k+1 = 段 k の隣り合う2つの max(**畳んでも peak は落ちない**)
//!
//! 描くときは「1px が抱える秒数」以下で最も粗い段を選ぶので、1列あたりに
//! 触る bucket は常に高々3つになる。段は倍々なので総容量は段0の2倍で止まる。
//!
//! # decode
//!
//! 生成は**別 thread**で行い、届くまでは薄い placeholder を出す。フレームは
//! 止めない。decode 済み PCM は再生と同じ `(content_hash, ordinal)` の控えを
//! 共有するので、**同じファイルを二度 decode しない**(先に鳴らしてあれば
//! decode ゼロ、先に帯を作れば再生側が控えを拾う)。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;

use motolii_audio::{decode_file_audio_ordinal, to_canonical, waveform_peaks, PcmCache};
use motolii_doc::{resolve_asset_path, Document};

/// 帯の高さ(px)。**高さは固定**で、行の面と取り合わない。
const BAND_H: f32 = 44.0;

/// soundtrack の PCM stream は必ず先頭の1本(`AudioProgram::from_document` と同じ規約)。
const SOUNDTRACK_ORDINAL: u32 = 0;

/// 帯の高さ。**soundtrack が無ければ 0 = 帯ごと出さない。**
/// 空の帯を置いて行の面を削らない。
pub fn band_height(has_soundtrack: bool) -> f32 {
    if has_soundtrack {
        BAND_H
    } else {
        0.0
    }
}

/// 帯が写している時間の窓。`view_start` / `view_span` は timeline 秒(`TimelineView`
/// と同じ)、`start_offset` は soundtrack のソースin点(秒)。
///
/// **timeline 0 秒に居るのは音源の `start_offset` 秒**である
/// (`AudioProgram::from_document` の `TimeMap::constant_speed(start_offset, 1, 1)` と同じ意味)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveformWindow {
    pub view_start: f32,
    pub view_span: f32,
    pub width_px: f32,
    pub start_offset: f32,
}

/// 表示用に段へ畳んだ soundtrack の波形。**PCM 1本につき1度だけ作る。**
#[derive(Debug)]
pub struct WaveformPeaks {
    /// 段0が最細。段 k は隣り合う2つの max で作る。
    levels: Vec<Vec<f32>>,
    /// 段0の bucket 1つが抱える秒数。段 k は `base_span * 2^k`。
    base_span: f64,
    /// 音源の全長(秒)。**ここより外は無音として描く。**
    source_seconds: f64,
}

impl WaveformPeaks {
    /// 段0の bucket 1つが抱える正準サンプルフレーム数。48kHz で約 0.33ms —
    /// いちばん寄った窓(`MIN_SPAN` = 0.25s)でも 1px より細かい。
    pub const BASE_BUCKET_FRAMES: u64 = 16;

    /// PCM を段へ畳む。**段0は `waveform_peaks` の生成物そのまま**で、
    /// bucket の割り方をここで作り直さない。
    pub fn build(pcm: &PcmCache) -> Result<Self, String> {
        let frames = pcm.frame_count();
        let rate = pcm.format().sample_rate;
        if frames == 0 || rate == 0 {
            return Err("empty soundtrack".to_owned());
        }
        let base_count = frames.div_ceil(Self::BASE_BUCKET_FRAMES).max(1) as usize;
        let base = waveform_peaks(pcm, base_count).map_err(|error| error.to_string())?;
        let source_seconds = frames as f64 / rate as f64;
        // `waveform_peaks` は全域を等幅に割るので、bucket の秒数は
        // 「全長 ÷ bucket 数」でぴったり出る(`BASE_BUCKET_FRAMES` から逆算しない)。
        let base_span = source_seconds / base.len().max(1) as f64;

        let mut levels = vec![base];
        while levels[levels.len() - 1].len() > 1 {
            let folded: Vec<f32> = levels[levels.len() - 1]
                .chunks(2)
                .map(|pair| pair.iter().copied().fold(0.0f32, f32::max))
                .collect();
            levels.push(folded);
        }
        Ok(Self {
            levels,
            base_span,
            source_seconds,
        })
    }

    /// 音源の全長(秒)。
    pub fn source_seconds(&self) -> f32 {
        self.source_seconds as f32
    }

    /// 段の数(段0が最細)。
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// 段の中身。**縮約の期待値をそのまま覗ける窓**(段が嘘をついていないことは
    /// 描画ではなくここで判定する)。
    pub fn level_peaks(&self, level: usize) -> &[f32] {
        self.levels
            .get(level)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// 列(=1px)ごとの peak を `out` へ書く。長さは `width_px` の切り捨て。
    ///
    /// **音源の外は 0**(負の時刻も終端の後ろも、無かったことにして描かない)。
    /// 触る bucket は1列あたり高々3つなので、ここは O(見えている px) で終わる。
    pub fn columns(&self, window: WaveformWindow, out: &mut Vec<f32>) {
        out.clear();
        let width = if window.width_px.is_finite() {
            window.width_px.floor().max(0.0) as usize
        } else {
            0
        };
        if width == 0 || self.levels.is_empty() {
            return;
        }
        if !window.view_span.is_finite() || window.view_span <= 0.0 {
            return;
        }
        let per_px = window.view_span as f64 / width as f64;
        let level = self.level_for(per_px);
        let buckets = &self.levels[level];
        let span = self.base_span * (1u64 << level) as f64;
        let origin = window.view_start as f64 + window.start_offset as f64;

        out.reserve(width);
        for column in 0..width {
            let from = origin + per_px * column as f64;
            out.push(peak_between(
                buckets,
                span,
                from,
                from + per_px,
                self.source_seconds,
            ));
        }
    }

    /// 1px が抱える秒数以下で**最も粗い**段。1列あたりの bucket 数を 1〜2 に保つ。
    fn level_for(&self, per_px: f64) -> usize {
        let last = self.levels.len() - 1;
        if self.base_span <= 0.0 || per_px <= self.base_span {
            return 0;
        }
        let steps = (per_px / self.base_span).log2().floor();
        if !steps.is_finite() || steps <= 0.0 {
            0
        } else {
            (steps as usize).min(last)
        }
    }
}

/// `[from, to)` 秒に掛かる bucket の max。音源の外は 0。
fn peak_between(buckets: &[f32], span: f64, from: f64, to: f64, source_seconds: f64) -> f32 {
    if span <= 0.0 || to <= 0.0 || from >= source_seconds || buckets.is_empty() {
        return 0.0;
    }
    let first = (from.max(0.0) / span).floor() as usize;
    if first >= buckets.len() {
        return 0.0;
    }
    let last = ((to.min(source_seconds) / span).ceil() as usize)
        .max(first + 1)
        .min(buckets.len());
    buckets[first..last].iter().copied().fold(0.0f32, f32::max)
}

// ---------------------------------------------------------------------------
// 座席(生成の面倒を見る側)
// ---------------------------------------------------------------------------

/// いま帯が写している soundtrack の身元。**変わったら作り直す。**
#[derive(Debug, Clone, PartialEq, Eq)]
struct SoundtrackId {
    content_hash: String,
    path: PathBuf,
}

/// 別 thread から戻ってくる荷物。
struct Built {
    /// この thread が新しく decode したときだけ Some(控えへ入れ直す)。
    decoded: Option<Arc<PcmCache>>,
    outcome: Result<WaveformPeaks, String>,
}

/// 帯として描けるもの。**毎フレームこれを1つ取って描く。**
pub(crate) enum WaveformBand {
    /// soundtrack が無い。帯ごと出さない。
    Absent,
    /// 生成中。薄い placeholder を出す(**フレームは止めない**)。
    Building,
    Ready(Arc<WaveformPeaks>),
    /// 帯は出すが、出せない理由を書く(黙って空の帯にしない)。
    Unavailable(String),
}

/// 波形の生成座席。decode と縮約を別 thread へ出し、届いたら控える。
#[derive(Default)]
pub(crate) struct WaveformSeat {
    id: Option<SoundtrackId>,
    pending: Option<Receiver<Built>>,
    peaks: Option<Arc<WaveformPeaks>>,
    failure: Option<String>,
}

impl WaveformSeat {
    /// 毎フレーム呼ぶ。**decode でフレームを止めない。**
    ///
    /// `caches` は再生と同じ `(content_hash, ordinal) → 正準PCM` の控えである。
    /// 既に居れば decode せずそれを畳み、新しく decode したらここへ入れて返すので、
    /// 再生側があとで開いても decode し直さない。
    pub(crate) fn poll(
        &mut self,
        document: &Document,
        project_root: Option<&Path>,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> WaveformBand {
        let Some(soundtrack) = &document.soundtrack else {
            self.clear();
            return WaveformBand::Absent;
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
            return WaveformBand::Ready(Arc::clone(peaks));
        }
        if let Some(reason) = &self.failure {
            return WaveformBand::Unavailable(reason.clone());
        }
        if self.pending.is_some() {
            return self.receive(&id, caches);
        }
        self.start(&id, caches);
        WaveformBand::Building
    }

    /// 生成を始める。控えに居れば decode を飛ばして畳むだけ。
    fn start(&mut self, id: &SoundtrackId, caches: &HashMap<(String, u32), Arc<PcmCache>>) {
        let cached = caches
            .get(&(id.content_hash.clone(), SOUNDTRACK_ORDINAL))
            .map(Arc::clone);
        let path = id.path.clone();
        let (tx, rx) = channel();
        // 送り先が居なくなっていても(帯を作り直した / 窓を閉じた)ここは
        // 静かに終わる。`send` の失敗は捨てる。
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

    /// 届いていれば控える。**届くまでは placeholder のまま。**
    fn receive(
        &mut self,
        id: &SoundtrackId,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> WaveformBand {
        let Some(rx) = &self.pending else {
            return WaveformBand::Building;
        };
        match rx.try_recv() {
            Err(TryRecvError::Empty) => WaveformBand::Building,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                self.failure = Some("waveform build stopped".to_owned());
                WaveformBand::Unavailable("waveform build stopped".to_owned())
            }
            Ok(built) => {
                self.pending = None;
                // **decode したぶんは再生と同じ控えへ入れる。** 次に鳴らす人が
                // 同じファイルをもう一度 decode しない。
                if let Some(pcm) = built.decoded {
                    caches.insert((id.content_hash.clone(), SOUNDTRACK_ORDINAL), pcm);
                }
                match built.outcome {
                    Ok(peaks) => {
                        let peaks = Arc::new(peaks);
                        self.peaks = Some(Arc::clone(&peaks));
                        WaveformBand::Ready(peaks)
                    }
                    Err(error) => {
                        self.failure = Some(error.clone());
                        WaveformBand::Unavailable(error)
                    }
                }
            }
        }
    }

    /// 身元が取れない/解決できないときの出口。理由を出したまま座り続ける。
    fn settle_on(&mut self, id: Option<SoundtrackId>, reason: &str) -> WaveformBand {
        if self.id != id {
            self.clear();
            self.id = id;
            self.failure = Some(reason.to_owned());
        }
        WaveformBand::Unavailable(reason.to_owned())
    }

    fn clear(&mut self) {
        self.id = None;
        self.pending = None;
        self.peaks = None;
        self.failure = None;
    }
}

/// 再生と同じ decode 経路(`decode_file_audio_ordinal` → `to_canonical`)。
/// 別の decoder を作らない。
fn decode_canonical(path: &Path) -> Result<Arc<PcmCache>, String> {
    let raw =
        decode_file_audio_ordinal(path, SOUNDTRACK_ORDINAL).map_err(|error| error.to_string())?;
    let canonical = to_canonical(&raw).map_err(|error| error.to_string())?;
    Ok(Arc::new(canonical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_audio::PcmFormat;

    fn ramp(frames: usize) -> PcmCache {
        let samples: Vec<f32> = (0..frames)
            .map(|i| (i as f32 / frames as f32) * 2.0 - 1.0)
            .collect();
        PcmCache::from_interleaved(
            samples,
            PcmFormat {
                channels: 1,
                sample_rate: 48_000,
            },
        )
        .expect("valid PCM")
    }

    /// **段を粗くしても peak は落ちない。** どの段で読んでも全体の max は同じ。
    #[test]
    fn every_level_keeps_the_overall_peak() {
        let peaks = WaveformPeaks::build(&ramp(4_096)).expect("pyramid");
        assert!(peaks.level_count() > 1, "段が積まれている");
        let top = peaks.level_peaks(0).iter().copied().fold(0.0f32, f32::max);
        for level in 0..peaks.level_count() {
            let here = peaks
                .level_peaks(level)
                .iter()
                .copied()
                .fold(0.0f32, f32::max);
            assert!((here - top).abs() < 1e-6, "段 {level} で peak が落ちた");
        }
    }

    /// **1列あたりに触る bucket は高々3つ。** これが「毎フレーム O(見えている px)」
    /// の中身であり、ズームを引いても列の値段が上がらない理由である。
    #[test]
    fn the_chosen_level_keeps_at_most_a_few_buckets_per_column() {
        let peaks = WaveformPeaks::build(&ramp(48_000 * 8)).expect("pyramid");
        for (span, width) in [(0.25f32, 1_200.0f32), (8.0, 1_200.0), (8.0, 120.0)] {
            let per_px = span as f64 / width as f64;
            let level = peaks.level_for(per_px);
            let bucket = peaks.base_span * (1u64 << level) as f64;
            let per_column = per_px / bucket;
            assert!(
                per_column <= 2.0 || level == peaks.level_count() - 1,
                "span={span} width={width}: 1列 {per_column} bucket は多すぎる"
            );
        }
    }

    /// 空の PCM は帯にならない(黙って0本の波形を描かない)。
    #[test]
    fn an_empty_source_is_a_reason_not_a_flat_line() {
        let empty = PcmCache::from_interleaved(
            Vec::new(),
            PcmFormat {
                channels: 1,
                sample_rate: 48_000,
            },
        )
        .expect("valid PCM");
        assert!(WaveformPeaks::build(&empty).is_err());
    }

    /// **soundtrack が無い document は座席を空にする。** thread も立てない。
    #[test]
    fn a_document_without_a_soundtrack_never_starts_a_build() {
        let (document, _names) = crate::timeline_editor::lab_fixture();
        let mut seat = WaveformSeat::default();
        let mut caches = HashMap::new();
        assert!(matches!(
            seat.poll(&document, None, &mut caches),
            WaveformBand::Absent
        ));
        assert!(seat.pending.is_none(), "生成を始めていない");
        assert!(caches.is_empty(), "decode もしていない");
    }
}
