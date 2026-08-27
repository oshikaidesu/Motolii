//! owns: mixプロデューサスレッド(A2)。`AudioProgram::mix_audio`をバックグラウンド
//! スレッドで実行し、mixed PCMを`rtrb`リングへ供給する。**音声コールバックは
//! 絶対にこのスレッドをブロックしない**(callback側は`ring.rs::fill_or_silence`
//! で読むだけ、KNOWN.md「音声」節: audio callback内でallocしない)。
//!
//! OWNS-JUSTIFICATION(A): `next/reference/KNOWN.md`「音声」の明記された規律
//! (audio callback内でallocしない)から、mixをバックグラウンドスレッドへ
//! 追い出す必然性が直接導かれる(裁定215 棚卸し 2026-08-23 #8)。
//!
//! 旧 `crates/motolii-audio/src/producer.rs::MixProducer` の移植 — **旧との
//! 構造上の違い**:
//! - リング型 → 旧の自前`RingProducer`(`push_frames`/`free_frames`を持つ)から
//!   `rtrb::Producer<f32>`直叩きへ(A1の方針、`ring.rs` doc 参照)。push上限は
//!   `Producer::slots()`(空きスロット数)から自分で数える([`push_frames`])
//! - **seek対応**(旧`MixProducer`に無かった新規口): 発注書「再生中の scrub は
//!   seek」を満たすため、走行中のスレッドへ`Arc<AtomicU64>`経由で目標
//!   (正準サンプルフレーム)を送れる([`MixProducer::seek`])。反映は次ループ
//!   先頭 — **リングに既に積んだ分はそのまま流れきる**(容量分だけ古い位置の
//!   音が続く既知の制約。KNOWN参照、実機確認課題)
//! - 終端到達を`finished`フラグ(`Arc<AtomicBool>`)で外部へ知らせる —
//!   `Shell`側のtick処理はこれを見ずに`PlaybackClock::position()`とcomp尺の
//!   比較だけで「終端で停止」を判定する(producer側はPCM供給を止めるだけでよい)
//!
//! **持ってこなかったもの**: 旧`AudioProducer`(decode-onlyの経路、
//! `MixProducer`と別クラスだった)は無い — A2発注書はProgram/mix経路だけを
//! 要求している。旧の「expected_device_frames」精密トリム(resample flush が
//! 尺をぴったり超えないようにする計算)も簡略化した——このMVPは
//! `MAX_FLUSH_CHUNKS`到達 or flush が空を返した時点で単純に打ち切る(数十msの
//! 余剰無音が出うるが、Shell側の終端判定はclockの供給フレーム数で行うので
//! 機能上の実害は無い)。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::convert::{time_to_canonical_frames, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::error::{AudioError, Result};
use crate::program::AudioProgram;
use crate::resample::FixedRatioResampler;

/// リングが満杯、または供給側が追いつけない時の再試行間隔。
const POLL_INTERVAL: Duration = Duration::from_millis(1);

/// 1回のmix呼び出しで作る正準(48k)フレーム数(プロデューサ側alloc。
/// callbackでは行わない)。
const MIX_CHUNK_FRAMES: usize = 1_024;

/// 終端フラッシュの安全弁(無限ループ防止)。
const MAX_FLUSH_CHUNKS: usize = 8;

/// `seek_to`の「保留seek無し」番兵値。正準フレーム数は`u64::MAX`まで届かない
/// (5分ステレオ48kHzでも1.4e7程度)ので実用上衝突しない。
const NO_SEEK: u64 = u64::MAX;

/// `AudioProgram`をmixしてリングへ供給するプロデューサ(A2)。
pub struct MixProducer {
    running: Arc<AtomicBool>,
    seek_to: Arc<AtomicU64>,
    finished: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl MixProducer {
    /// `start_frame`(正準48kサンプルフレーム — comp動画フレームではない、
    /// `crate::convert::time_to_canonical_frames`で変換する)から
    /// `device_sample_rate`向けに供給を始める。不一致時のみリサンプルを挿入する。
    pub fn spawn(
        program: Arc<AudioProgram>,
        producer: rtrb::Producer<f32>,
        start_frame: u64,
        device_sample_rate: u32,
    ) -> Result<Self> {
        if device_sample_rate == 0 {
            return Err(AudioError::UnsupportedSampleRate {
                sample_rate: device_sample_rate,
            });
        }

        let running = Arc::new(AtomicBool::new(true));
        let seek_to = Arc::new(AtomicU64::new(NO_SEEK));
        let finished = Arc::new(AtomicBool::new(false));
        let running_thread = Arc::clone(&running);
        let seek_thread = Arc::clone(&seek_to);
        let finished_thread = Arc::clone(&finished);

        let handle = thread::Builder::new()
            .name("motolii-audio-mix-producer".into())
            .spawn(move || {
                producer_loop(
                    &program,
                    producer,
                    start_frame,
                    device_sample_rate,
                    &running_thread,
                    &seek_thread,
                    &finished_thread,
                );
            })
            .map_err(AudioError::ProducerSpawn)?;

        Ok(Self {
            running,
            seek_to,
            finished,
            handle: Some(handle),
        })
    }

    /// 走行中に目標(正準48kサンプルフレーム)へ跳ぶ。次ループ先頭で反映される —
    /// モジュールdoc「seek対応」参照。
    pub fn seek(&self, frame: u64) {
        self.seek_to.store(frame.min(NO_SEEK - 1), Ordering::Release);
    }

    /// 終端まで供給し終えた(flush含む)か。
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }

    /// 供給を止めてスレッドをjoinする(`?`早期returnで飛ばされないよう、
    /// Dropからも同じ経路を呼ぶ — AGENTS.md「後始末を飛ばさない」)。
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for MixProducer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// `producer`(rtrbの空きスロット数)に収まる分だけフレーム単位で押し込む。
/// `fill_or_silence`のpop側と対称 — `slots()`が数えた範囲内なので`push`は
/// 必ず成功する。
fn push_frames(producer: &mut rtrb::Producer<f32>, samples: &[f32], channels: usize) -> usize {
    if channels == 0 || samples.is_empty() {
        return 0;
    }
    let free_frames = producer.slots() / channels;
    let want_frames = samples.len() / channels;
    let frames = free_frames.min(want_frames);
    for &sample in &samples[..frames * channels] {
        let _ = producer.push(sample);
    }
    frames
}

/// source終端とDocument composition尺の大きい方(正準48kフレーム)。
fn program_end_frame(program: &AudioProgram) -> u64 {
    let mut end = time_to_canonical_frames(program.composition_duration());
    for source in program.sources() {
        let start = time_to_canonical_frames(source.timeline_start);
        let dur = time_to_canonical_frames(source.timeline_duration);
        end = end.max(start.saturating_add(dur));
    }
    end
}

#[allow(clippy::too_many_arguments)]
fn producer_loop(
    program: &AudioProgram,
    mut producer: rtrb::Producer<f32>,
    start_frame: u64,
    device_sample_rate: u32,
    running: &AtomicBool,
    seek_to: &AtomicU64,
    finished: &AtomicBool,
) {
    let channels = CANONICAL_CHANNELS as usize;
    let resampling = device_sample_rate != CANONICAL_SAMPLE_RATE;
    let mut resampler = if resampling {
        match FixedRatioResampler::new(CANONICAL_SAMPLE_RATE, device_sample_rate, CANONICAL_CHANNELS) {
            Ok(mut resampler) => {
                resampler.reset();
                Some(resampler)
            }
            // `negotiate_output`が既にデバイス対応レートへ絞っているので通常は
            // 起きない。起きても黙って恒等パスへ倒す(M16: panicしない)。
            Err(_) => None,
        }
    } else {
        None
    };

    let end_frame = program_end_frame(program);
    let mut playhead = start_frame.min(end_frame);
    let mut pending: Vec<f32> = Vec::new();
    let mut pending_off = 0usize;
    let mut flushing = playhead >= end_frame;
    let mut flush_chunks = 0usize;

    while running.load(Ordering::Acquire) {
        // 走行中に来たseek要求(唯一の外部入力)を先頭で反映する。
        let seek = seek_to.swap(NO_SEEK, Ordering::AcqRel);
        if seek != NO_SEEK {
            playhead = seek.min(end_frame);
            pending.clear();
            pending_off = 0;
            flush_chunks = 0;
            flushing = playhead >= end_frame;
            finished.store(false, Ordering::Release);
            if let Some(resampler) = resampler.as_mut() {
                resampler.reset();
            }
        }

        // pending(直近のmix/resample結果で未送出分)を先に出す(部分pushに耐える)。
        if pending_off < pending.len() {
            let frames_left = (pending.len() - pending_off) / channels;
            if frames_left == 0 {
                pending.clear();
                pending_off = 0;
                continue;
            }
            let end = pending_off + frames_left * channels;
            let pushed = push_frames(&mut producer, &pending[pending_off..end], channels);
            if pushed == 0 {
                thread::sleep(POLL_INTERVAL);
                continue;
            }
            pending_off += pushed * channels;
            if pending_off >= pending.len() {
                pending.clear();
                pending_off = 0;
            }
            continue;
        }

        if flushing {
            let Some(resampler) = resampler.as_mut() else {
                finished.store(true, Ordering::Release);
                break;
            };
            if flush_chunks >= MAX_FLUSH_CHUNKS {
                finished.store(true, Ordering::Release);
                break;
            }
            let Ok(out) = resampler.flush_silence_chunk() else {
                finished.store(true, Ordering::Release);
                break;
            };
            flush_chunks += 1;
            if out.is_empty() {
                finished.store(true, Ordering::Release);
                break;
            }
            pending.extend_from_slice(out);
            continue;
        }

        if playhead >= end_frame {
            if resampler.is_some() {
                flushing = true;
                continue;
            }
            finished.store(true, Ordering::Release);
            break;
        }

        let need = resampler
            .as_ref()
            .map_or(MIX_CHUNK_FRAMES, FixedRatioResampler::input_frames_next)
            .min((end_frame - playhead) as usize);
        if need == 0 {
            flushing = true;
            continue;
        }
        let Ok((pcm, _report)) = program.mix_audio(playhead, need, None) else {
            finished.store(true, Ordering::Release);
            break;
        };
        playhead += need as u64;

        match resampler.as_mut() {
            Some(resampler) => match resampler.process_interleaved(&pcm) {
                Ok(out) => pending.extend_from_slice(out),
                Err(_) => {
                    finished.store(true, Ordering::Release);
                    break;
                }
            },
            None => pending.extend_from_slice(&pcm),
        }
        if playhead >= end_frame {
            flushing = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// pureヘルパの落とし穴柵: 空きスロットを超えて押し込まない。
    #[test]
    fn push_frames_stops_at_free_slots() {
        let (mut producer, _consumer) = rtrb::RingBuffer::<f32>::new(4); // 2 frames stereo
        let samples = [1.0, -1.0, 2.0, -2.0, 3.0, -3.0]; // 3 frames
        let pushed = push_frames(&mut producer, &samples, 2);
        assert_eq!(pushed, 2, "capacity(2フレーム)を超えて押し込んでいる");
    }

    #[test]
    fn push_frames_ignores_channel_zero() {
        let (mut producer, _consumer) = rtrb::RingBuffer::<f32>::new(4);
        assert_eq!(push_frames(&mut producer, &[1.0, 2.0], 0), 0);
    }
}
