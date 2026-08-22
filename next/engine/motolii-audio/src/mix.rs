//! AG-2: 決定論的 `mix_audio`。preview/export 同一意味の正準PCM境界。
//!
//! 旧 `crates/motolii-audio/src/mix.rs` からの移植。**型の読み替え**(発注書の指示):
//! - `motolii_core::TimeMap` → この crate 自身の [`crate::TimeMap`](`time_map.rs`。
//!   `next/core/motolii-core` はこの型を落としており、store の `LayerTiming` は
//!   comp フレーム単位の整数写像なので mix のサンプル精度に使えない)
//! - `motolii_doc::DocParam`(`Const`/`Keyframes` の2 variant)→
//!   `Option<motolii_eval::KeyframeTrack>`。`None` = track が無い = 裁定20
//!   「キーを打っていない property は静止値」を音声にもそのまま適用し、gain の
//!   既定値 1.0 を返す。`Some(track)` は `track.eval(t)` を正本として使う
//!   (旧版の `DocKeyframeTrack::eval` 委譲と同じ形)
//! - `motolii_doc::AudioOutOfRange` → この crate が持つ [`AudioOutOfRange`](store に
//!   同義の概念が無い。音声の範囲外挙動は engine 層の判断、という store 側の設計
//!   (`ResolvedLayer::source_frame` のdocコメント参照)にそのまま従う)
//!
//! ## B42(音声内容整形束)の拡張 — 2026-08-22
//!
//! map の bundle B42(採用予定13行)のうち **mix 経路で表現できる意味だけ**を
//! ここへ実装した(EQ/コンプ級のフィルタは vism 圏として見送り、AI解析系
//! (Audio Enhancer/Vocal isolation)・素材追加系(Music/Sound effects)・
//! マイク入力(Voiceover)・波形帯UI(Show Clip Gain Line)は audio engine の
//! mix 経路そのものには意味が無いので対象外):
//!
//! - **gain/volume**(id11/27/28): 既存の `MixSource::gain` がそのままカバー
//!   (変更なし)
//! - **pan**(定位。map に直接の行は無いが発注書が明示指定): [`MixSource::pan`]
//!   を新設。等パワー則 — **W3C Web Audio API `StereoPannerNode` の
//!   panning algorithm をそのまま採用**(仕様:
//!   <https://webaudio.github.io/web-audio-api/#stereopanner-algorithm>、
//!   stereo入力パス。`apply_pan_stereo` docコメント参照)。線形balanceでなく
//!   等パワー則を選んだ理由: 線形crossfadeは中央付近で知覚音量が下がる
//!   (pan law問題)ことがオーディオ工学で広く知られており、等パワー則は
//!   これを避ける業界標準(Pro Tools/Logic 等のbalance/pan既定則も同型)
//! - **fade in/out**(id3/23 Apply/Batch Fade Settings): [`FadeSpec`]/
//!   [`FadeCurve`] を新設。既定は等パワー(`EqualPower`) — 隣接クリップと
//!   重なるクロスフェードで音量の谷を作らない古典的な理由がここでも成立する
//! - **mute**(音側): 既存の `MixSource::enabled` がそのままカバー(裁定135の
//!   「store 側にaudio専用muteが無い」問題はこの crate の外 — program.rs
//!   doc参照。engine側の口は既にある)
//! - **正規化**(id42 Loudness normalisation): [`normalize_gain_for_peak`]。
//!   **真の LUFS(ITU-R BS.1770 K-weighting + gating)はエフェクト級のDSPで
//!   vism 圏 — 見送り**。ここでは決定論的な peak-based 正規化のみを実装し、
//!   既存の `gain` 経路(1個の定数gain)へ計算結果を乗せる薄い顔に留めた

use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_eval::{KeyframeTrack, Value};

use crate::cache::PcmCache;
use crate::convert::{canonical_format, CANONICAL_CHANNELS, CANONICAL_SAMPLE_RATE};
use crate::error::{AudioError, Result};
use crate::meter::AudioMeter;
use crate::time_map::TimeMap;

/// ソース採取が素材の実尺を外れたときの挙動。旧 `motolii_doc::AudioOutOfRange` と
/// 同じ2値(store にはこの概念が無いので、この crate が持つ)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioOutOfRange {
    /// 無音。
    #[default]
    Silence,
    /// 素材の実尺で wrap する。
    Loop,
}

/// フェードの補間則(B42)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadeCurve {
    /// 直線(振幅を時間に対して線形に変化させる)。
    Linear,
    /// 等パワー(`sin`/`cos`)。隣接クリップとのクロスフェードで音量の谷を
    /// 作らない古典的な理由により既定に採る(`mod`docの選定理由参照)。
    #[default]
    EqualPower,
}

/// レイヤー単位のフェード仕様(B42、map id3/23)。
///
/// clip-local な相対 duration で持つ(絶対タイムライン時刻の `KeyframeTrack`
/// ではない) — 「クリップの端から何秒」という指定はAE/Premiere/CapCut共通の
/// 語彙で、`timeline_start`/`timeline_duration` が変わっても再計算不要なほうが
/// 自然なため。`fade_in`/`fade_out` が `timeline_duration` を超える、または
/// 両者が重なる場合でも [`fade_envelope`] が両方の区間で乗算し、0..=1 の範囲を
/// 保つ(発注書「fade の境界」試験対応)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeSpec {
    pub fade_in: RationalTime,
    pub fade_out: RationalTime,
    pub curve: FadeCurve,
}

impl FadeSpec {
    /// フェード無し(既定)。
    pub const NONE: FadeSpec = FadeSpec {
        fade_in: RationalTime::ZERO,
        fade_out: RationalTime::ZERO,
        curve: FadeCurve::EqualPower,
    };
}

impl Default for FadeSpec {
    fn default() -> Self {
        Self::NONE
    }
}

/// mixへ投入する1 source(正準48k stereo cache前提)。
#[derive(Debug, Clone)]
pub struct MixSource {
    pub pcm: Arc<PcmCache>,
    /// タイムライン上の開始。
    pub timeline_start: RationalTime,
    /// タイムライン上の尺(半開)。
    pub timeline_duration: RationalTime,
    /// clip_local → source 時刻(varispeed含む)。
    pub time_map: TimeMap,
    /// linear gain。`None` = track が無い(裁定20: 静止値 1.0)。`Some` は
    /// `KeyframeTrack::eval` をそのまま使う。
    pub gain: Option<KeyframeTrack>,
    /// stereo pan(定位、B42)。range -1.0(full left)..=1.0(full right)。
    /// `None` = track が無い(裁定20と同型の既定 — 静止値 0.0 = 中央=無変化)。
    /// `Some` の評価値は `apply_pan_stereo` へそのまま渡す前に -1..1へclampする
    /// (`eval_pan_at` 参照 — 有限性だけ検査し、範囲外はエラーでなくclamp)。
    pub pan: Option<KeyframeTrack>,
    /// フェード仕様(B42)。既定は無フェード([`FadeSpec::NONE`])。
    pub fade: FadeSpec,
    pub out_of_range: AudioOutOfRange,
    pub enabled: bool,
}

impl MixSource {
    pub fn validate(&self) -> Result<()> {
        if self.pcm.format() != canonical_format() {
            return Err(AudioError::Resample {
                detail: "MixSource.pcm must be canonical 48kHz stereo",
            });
        }
        // 先頭時刻で型だけ検査(track はあるのに数値でない、を拒む)。
        let _ = eval_gain_at(&self.gain, RationalTime::ZERO)?;
        let _ = eval_pan_at(&self.pan, RationalTime::ZERO)?;
        if self.timeline_duration <= RationalTime::ZERO {
            return Err(AudioError::InvalidMixRange);
        }
        Ok(())
    }
}

/// `mix_audio` の結果メタ(正規silenceとunderflowの区別用)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MixReport {
    /// 出力フレーム数。
    pub frames: usize,
    /// sourceが無くgap/silenceで埋めたフレーム数(正規silence。underflowではない)。
    pub silence_frames: usize,
}

/// 正準フレーム範囲 `[start_frame, start_frame + frame_count)` を mix する。
///
/// - 評価順は呼び出し側が並べた `sources` 順(soundtrack/layer の重ね順は
///   呼び出し側=`program.rs` が決める)
/// - 毎source clamp / normalize / limiter は行わない
/// - `meter` を渡してもPCM結果は変わらない
pub fn mix_audio(
    sources: &[MixSource],
    master_gain: f64,
    start_frame: u64,
    frame_count: usize,
    meter: Option<&AudioMeter>,
) -> Result<(Vec<f32>, MixReport)> {
    if !master_gain.is_finite() || master_gain < 0.0 {
        return Err(AudioError::InvalidGain { gain: master_gain });
    }
    for source in sources {
        source.validate()?;
    }

    let mut out = vec![0.0f32; frame_count.saturating_mul(CANONICAL_CHANNELS as usize)];
    if frame_count == 0 {
        return Ok((out, MixReport::default()));
    }

    let mut silence_frames = 0usize;
    for i in 0..frame_count {
        let frame_index = start_frame + i as u64;
        let timeline_t = frame_to_time(frame_index)?;
        let mut left = 0.0f64;
        let mut right = 0.0f64;
        let mut any = false;

        for source in sources {
            if !source.enabled {
                continue;
            }
            let Some((l, r)) = sample_source(source, timeline_t)? else {
                continue;
            };
            let local = local_for_gain(source, timeline_t)?;
            let gain = eval_gain_at(&source.gain, local)?;
            let envelope = fade_envelope(local, source.timeline_duration, &source.fade);
            let pan = eval_pan_at(&source.pan, local)?;
            any = true;
            let (pl, pr) = apply_pan_stereo(l * gain * envelope, r * gain * envelope, pan);
            left += pl;
            right += pr;
        }

        if !any {
            silence_frames += 1;
        }

        left *= master_gain;
        right *= master_gain;
        // 毎source / mix結果のclampはしない(AG-2 / metering契約)。
        let base = i * CANONICAL_CHANNELS as usize;
        out[base] = left as f32;
        out[base + 1] = right as f32;
    }

    if let Some(meter) = meter {
        meter.observe_interleaved_stereo(&out);
    }

    Ok((
        out,
        MixReport {
            frames: frame_count,
            silence_frames,
        },
    ))
}

fn frame_to_time(frame: u64) -> Result<RationalTime> {
    // frame / 48000。分母をレートに固定して肥大化を避ける。
    RationalTime::try_new(frame as i64, CANONICAL_SAMPLE_RATE as i64)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn local_for_gain(source: &MixSource, timeline_t: RationalTime) -> Result<RationalTime> {
    timeline_t
        .try_sub(source.timeline_start)
        .map_err(|_| AudioError::InvalidMixRange)
}

fn eval_gain_at(gain: &Option<KeyframeTrack>, t: RationalTime) -> Result<f64> {
    let raw = match gain {
        // track が無い property は静止値(裁定20)。gain の既定は等倍。
        None => 1.0,
        Some(track) => match track.eval(t) {
            Value::F64(v) => v,
            _ => return Err(AudioError::InvalidGain { gain: f64::NAN }),
        },
    };
    if raw.is_finite() && raw >= 0.0 {
        Ok(raw)
    } else {
        Err(AudioError::InvalidGain { gain: raw })
    }
}

/// stereo pan(定位)を評価する。`None` = track無し = 静止値 0.0(裁定20と同型、
/// 中央=無変化)。有限性だけ検査し、範囲外(-1..1超)はエラーでなく
/// `apply_pan_stereo` 側でclampする(W3C仕様の挙動と同型)。
fn eval_pan_at(pan: &Option<KeyframeTrack>, t: RationalTime) -> Result<f64> {
    let raw = match pan {
        None => 0.0,
        Some(track) => match track.eval(t) {
            Value::F64(v) => v,
            _ => return Err(AudioError::InvalidPan { pan: f64::NAN }),
        },
    };
    if raw.is_finite() {
        Ok(raw)
    } else {
        Err(AudioError::InvalidPan { pan: raw })
    }
}

/// stereo pan を1サンプルへ適用する。
///
/// **W3C Web Audio API `StereoPannerNode` の panning algorithm(stereo入力
/// パス)をそのまま採用**(仕様: <https://webaudio.github.io/web-audio-api/#stereopanner-algorithm>、
/// 2026-08-22時点の editor's draft、該当節「StereoPannerNode Panning」の
/// アルゴリズム定義):
///
/// ```text
/// x = pan <= 0 ? pan + 1 : pan
/// gainL = cos(x * PI/2); gainR = sin(x * PI/2)
/// pan <= 0: outputL = inputL + inputR*gainL; outputR = inputR*gainR
/// pan >  0: outputL = inputL*gainL;          outputR = inputR + inputL*gainR
/// ```
///
/// `pan=0` は恒等写像(gainL=0, gainR=1 → 元のL/Rがそのまま出る)。`pan=±1` は
/// 両チャンネルの内容が片方へ合算される(hard left/right)。**等パワー則を
/// 線形balanceより優先した理由**: 線形crossfadeは中央付近で知覚音量が
/// 下がる(pan law問題)ことがオーディオ工学で広く知られており、等パワー則が
/// これを避ける業界標準(Pro Tools/Logic 等のbalance/pan既定則も同型)。
/// stereo入力に対する式は本アルゴリズムの定義どおり単純な「反対chへの
/// crossfeed」であって、mono入力用の「2ch分岐」とは別物(仕様が両者を
/// 明示的に書き分けている)。
fn apply_pan_stereo(left: f64, right: f64, pan: f64) -> (f64, f64) {
    let pan = pan.clamp(-1.0, 1.0);
    let x = if pan <= 0.0 { pan + 1.0 } else { pan };
    let gain_l = (x * std::f64::consts::FRAC_PI_2).cos();
    let gain_r = (x * std::f64::consts::FRAC_PI_2).sin();
    if pan <= 0.0 {
        (left + right * gain_l, right * gain_r)
    } else {
        (left * gain_l, right + left * gain_r)
    }
}

/// clip-local 時刻 `local`(`0 <= local < duration` の前提、範囲外は呼び出し側
/// `sample_source` が既に弾いている)における fade envelope(0.0..=1.0)。
///
/// `fade_in`/`fade_out` は各々 `duration` を超えないよう内部でclampし、両者が
/// 重なる場合は該当区間で両方のenvelopeを乗算する(オーバーラップしたfade
/// handleは両方効く、というPremiere/Resolve等の一般的な挙動と同型 — 発注書
/// 「fade の境界」試験対応)。
fn fade_envelope(local: RationalTime, duration: RationalTime, fade: &FadeSpec) -> f64 {
    let mut envelope = 1.0;

    if fade.fade_in > RationalTime::ZERO {
        let in_len = fade.fade_in.min(duration);
        if local < in_len {
            envelope *= curve_value(ratio_unit(local, in_len), fade.curve);
        }
    }

    if fade.fade_out > RationalTime::ZERO {
        let out_len = fade.fade_out.min(duration);
        if let Ok(out_start) = duration.try_sub(out_len) {
            if local > out_start {
                if let Ok(remaining) = duration.try_sub(local) {
                    envelope *= curve_value(ratio_unit(remaining, out_len), fade.curve);
                } else {
                    // durationを跨ぐ数値的異常(呼び出し前提が破れている) — 無音側へ倒す。
                    envelope = 0.0;
                }
            }
        }
    }

    envelope
}

/// `numerator / denominator` を `[0.0, 1.0]` へclampした比。`denominator <= 0`
/// は呼び出し元(`fade_envelope`)が `.min(duration)` 済みなので実質起きないが、
/// 型として保証されていないため防御的に1.0(=フェード済み)を返す。
fn ratio_unit(numerator: RationalTime, denominator: RationalTime) -> f64 {
    if denominator <= RationalTime::ZERO {
        return 1.0;
    }
    (numerator.as_seconds_f64() / denominator.as_seconds_f64()).clamp(0.0, 1.0)
}

fn curve_value(t: f64, curve: FadeCurve) -> f64 {
    match curve {
        FadeCurve::Linear => t,
        FadeCurve::EqualPower => (t * std::f64::consts::FRAC_PI_2).sin(),
    }
}

/// map id42(Loudness normalisation)の薄い顔。**真の LUFS(ITU-R BS.1770
/// K-weighting + gating)はエフェクト級のDSPで vism 圏 — 見送り**。ここでは
/// 決定論的な peak-based 正規化のみ実装し、計算結果を既存の
/// `MixSource::gain`(1個の定数 Hold keyframe)へ乗せる薄い顔として使う想定
/// (この関数自体は `mix_audio` のコード経路を増やさない — 呼び出し側が
/// オフラインで1回呼び、結果をgainへ書く)。
///
/// `target_peak` は線形振幅(dBFSではない。例: -1dBFS相当なら
/// `10f64.powf(-1.0 / 20.0)`)。`pcm` が完全な無音(peak==0)の場合は
/// `1.0`(無変換)を返す — 0除算/無限大gainを避ける。
pub fn normalize_gain_for_peak(pcm: &PcmCache, target_peak: f64) -> Result<f64> {
    if !target_peak.is_finite() || target_peak < 0.0 {
        return Err(AudioError::InvalidGain { gain: target_peak });
    }
    let frame_count = pcm.frame_count();
    let mut peak = 0.0f64;
    if frame_count > 0 {
        let samples = pcm.read_frames(0, frame_count as usize)?;
        for &sample in samples {
            peak = peak.max(sample.abs() as f64);
        }
    }
    if peak <= 0.0 {
        Ok(1.0)
    } else {
        Ok(target_peak / peak)
    }
}

fn sample_source(source: &MixSource, timeline_t: RationalTime) -> Result<Option<(f64, f64)>> {
    let local = match timeline_t.try_sub(source.timeline_start) {
        Ok(t) if t >= RationalTime::ZERO && t < source.timeline_duration => t,
        _ => return Ok(None),
    };
    let source_t = source
        .time_map
        .try_map(local)
        .map_err(|_| AudioError::InvalidMixRange)?;
    let src_frames = source.pcm.frame_count() as f64;
    if src_frames <= 0.0 {
        return Ok(None);
    }
    // seconds経由だと10分級でfloat丸めが乗るので、num*rate/denで直接フレーム位置へ。
    let mut pos =
        (source_t.num() as f64) * f64::from(CANONICAL_SAMPLE_RATE) / (source_t.den() as f64);
    if !(0.0..src_frames).contains(&pos) {
        match source.out_of_range {
            AudioOutOfRange::Silence => return Ok(None),
            AudioOutOfRange::Loop => {
                pos = pos.rem_euclid(src_frames);
            }
        }
    }
    Ok(Some(lerp_stereo(source.pcm.as_ref(), pos)))
}

fn lerp_stereo(pcm: &PcmCache, pos: f64) -> (f64, f64) {
    let max_index = pcm.frame_count().saturating_sub(1);
    if pcm.frame_count() == 0 {
        return (0.0, 0.0);
    }
    let i0 = (pos.floor() as u64).min(max_index);
    let i1 = (i0 + 1).min(max_index);
    let frac = (pos - i0 as f64).clamp(0.0, 1.0);
    let f0 = pcm.frame_at(i0).expect("in-range");
    let f1 = pcm.frame_at(i1).expect("in-range");
    let l = f0[0] as f64 * (1.0 - frac) + f1[0] as f64 * frac;
    let r = f0[1] as f64 * (1.0 - frac) + f1[1] as f64 * frac;
    (l, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::to_canonical;
    use crate::meter::MeterSnapshot;
    use motolii_eval::{Interp, Keyframe};

    fn stereo_cache(samples: Vec<f32>) -> Arc<PcmCache> {
        Arc::new(PcmCache::from_interleaved(samples, canonical_format()).expect("valid"))
    }

    fn identity_source(pcm: Arc<PcmCache>, gain: f64) -> MixSource {
        MixSource {
            pcm,
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: const_gain(gain),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        }
    }

    /// 旧 `DocParam::const_f64` に相当する「動かない gain」。定数値は `None`
    /// (静止値=1.0)と衝突しないように、1本だけキーを持つ Hold track で表す
    /// (裁定20 の「キーを打っていない=静止値」は 1.0 専用の近道であって、
    /// 他の定数値まで `None` へ潰すと「1.0 以外の定数」を表現できなくなる)。
    fn const_gain(value: f64) -> Option<KeyframeTrack> {
        if value == 1.0 {
            return None;
        }
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(value),
            interp: Interp::Hold,
            spatial: None,
        });
        Some(track)
    }

    /// pan用の定数track。gainと違い0.0を特別扱いしない(`None`と0.0は既に
    /// 同じ意味 — 裁定20どおり「track無し=静止値」で、静止値がちょうど0.0
    /// なので特別扱いする定数が無い)。
    fn const_track(value: f64) -> Option<KeyframeTrack> {
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(value),
            interp: Interp::Hold,
            spatial: None,
        });
        Some(track)
    }

    #[test]
    fn two_sources_sum_deterministically() {
        // 1 frame: [0.25, 0.5] + [0.5, 0.25] = [0.75, 0.75]
        let a = stereo_cache(vec![0.25, 0.5]);
        let b = stereo_cache(vec![0.5, 0.25]);
        let (out, report) = mix_audio(
            &[identity_source(a, 1.0), identity_source(b, 1.0)],
            1.0,
            0,
            1,
            None,
        )
        .unwrap();
        assert_eq!(out, vec![0.75, 0.75]);
        assert_eq!(report.silence_frames, 0);
    }

    #[test]
    fn master_gain_applies_last_without_clamp() {
        let a = stereo_cache(vec![0.8, 0.8]);
        let (out, _) = mix_audio(&[identity_source(a, 1.0)], 2.0, 0, 1, None).unwrap();
        assert_eq!(out, vec![1.6, 1.6]);
    }

    #[test]
    fn gap_is_silence_not_underflow_counter() {
        let a = stereo_cache(vec![1.0, 1.0]);
        let mut source = identity_source(a, 1.0);
        source.timeline_start = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, report) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        assert_eq!(&out[..2], &[0.0, 0.0]);
        assert_eq!(&out[2..], &[1.0, 1.0]);
        assert_eq!(report.silence_frames, 1);
    }

    #[test]
    fn out_of_range_loop_wraps() {
        let a = stereo_cache(vec![0.1, 0.2, 0.3, 0.4]); // 2 frames
        let mut source = identity_source(a, 1.0);
        source.timeline_duration = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.out_of_range = AudioOutOfRange::Loop;
        // speed 1, source frames 0,1,0,1
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        assert_eq!(&out[0..2], &[0.1, 0.2]);
        assert_eq!(&out[2..4], &[0.3, 0.4]);
        assert_eq!(&out[4..6], &[0.1, 0.2]);
        assert_eq!(&out[6..8], &[0.3, 0.4]);
    }

    #[test]
    fn metering_does_not_change_pcm() {
        let a = stereo_cache(vec![1.5, -1.25]);
        let meter = AudioMeter::new();
        let (with_m, _) =
            mix_audio(&[identity_source(a.clone(), 1.0)], 1.0, 0, 1, Some(&meter)).unwrap();
        let (without_m, _) = mix_audio(&[identity_source(a, 1.0)], 1.0, 0, 1, None).unwrap();
        assert_eq!(with_m, without_m);
        assert_eq!(
            meter.snapshot(),
            MeterSnapshot {
                peak_l: 1.5,
                peak_r: 1.25,
                clipped: true,
            }
        );
    }

    #[test]
    fn mono_44100_and_stereo_48000_mix() {
        let mono_441 = PcmCache::from_interleaved(
            vec![0.2; 441], // 0.01s @ 44100
            crate::cache::PcmFormat {
                channels: 1,
                sample_rate: 44_100,
            },
        )
        .unwrap();
        let stereo_48 = PcmCache::from_interleaved(
            [0.1, -0.1].repeat(480), // 0.01s @ 48000
            canonical_format(),
        )
        .unwrap();
        let a = Arc::new(to_canonical(&mono_441).unwrap());
        let b = Arc::new(to_canonical(&stereo_48).unwrap());
        let duration = RationalTime::try_new(1, 100).unwrap(); // 0.01s
        let sources = [
            MixSource {
                pcm: a,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::IDENTITY,
                gain: None,
                pan: None,
                fade: FadeSpec::NONE,
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
            MixSource {
                pcm: b,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::IDENTITY,
                gain: None,
                pan: None,
                fade: FadeSpec::NONE,
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            },
        ];
        let frames = 480; // 0.01s @ 48k
        let (out, _) = mix_audio(&sources, 1.0, 0, frames, None).unwrap();
        // mono 0.2→L=R + stereo 0.1/-0.1 ≈ 0.3 / 0.1 (resample誤差あり)
        assert!(out[0] > 0.25 && out[0] < 0.35);
        assert!(out[1] > 0.05 && out[1] < 0.15);
    }

    #[test]
    fn varispeed_doubles_source_advance() {
        // 4 source frames of distinct L values.
        let pcm = stereo_cache(vec![0.0, 0.0, 0.25, 0.0, 0.5, 0.0, 0.75, 0.0]);
        let mut source = identity_source(pcm, 1.0);
        source.time_map = TimeMap::constant_speed(RationalTime::ZERO, 2, 1).unwrap();
        source.timeline_duration = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        // t=0 → source 0, t=1/48000 → source 2/48000 (speed 2)
        assert_eq!(out[0], 0.0);
        assert_eq!(out[2], 0.5);
    }

    #[test]
    fn chunk_size_rebuild_matches_whole() {
        let a = stereo_cache(
            (0..20)
                .flat_map(|i| [i as f32 * 0.01, -(i as f32) * 0.01])
                .collect(),
        );
        let sources = [identity_source(a, 0.5)];
        let (whole, _) = mix_audio(&sources, 1.0, 0, 10, None).unwrap();
        let mut rebuilt = Vec::new();
        for (start, count) in [(0u64, 3usize), (3, 3), (6, 3), (9, 1)] {
            let (chunk, _) = mix_audio(&sources, 1.0, start, count, None).unwrap();
            rebuilt.extend_from_slice(&chunk);
        }
        assert_eq!(whole, rebuilt);
    }

    #[test]
    fn ten_minute_timeline_frame_maps_without_drift() {
        // 10分 = 28_800_000 frames @48k。全展開せず末尾付近の既知サンプル対応だけ審判する。
        let ten_min = 10u64 * 60 * u64::from(CANONICAL_SAMPLE_RATE);
        let pcm_frames = 64u64;
        let mut samples = Vec::with_capacity(pcm_frames as usize * 2);
        for i in 0..pcm_frames {
            samples.push(i as f32 * 0.01);
            samples.push(-(i as f32) * 0.01);
        }
        let source = MixSource {
            pcm: stereo_cache(samples),
            timeline_start: RationalTime::try_new(
                (ten_min - 32) as i64,
                CANONICAL_SAMPLE_RATE as i64,
            )
            .unwrap(),
            timeline_duration: RationalTime::try_new(
                pcm_frames as i64,
                CANONICAL_SAMPLE_RATE as i64,
            )
            .unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: None,
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (out, _) = mix_audio(&[source], 1.0, ten_min - 32, 4, None).unwrap();
        assert_eq!(&out[0..2], &[0.0, 0.0]);
        assert_eq!(&out[2..4], &[0.01, -0.01]);
        assert_eq!(&out[4..6], &[0.02, -0.02]);
        assert_eq!(&out[6..8], &[0.03, -0.03]);
    }

    #[test]
    fn hold_gain_keyframes_follow_eval() {
        // Hold区間の中点で線形補間(1.25)にならず 0.5 のまま — 正本evalと一致。
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.5),
            interp: Interp::Hold,
            spatial: None,
        });
        track.insert(Keyframe {
            t: RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            value: Value::F64(2.0),
            interp: Interp::Hold,
            spatial: None,
        });
        let mid = RationalTime::try_new(1, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let expected = match track.eval(mid) {
            Value::F64(v) => v,
            _ => panic!("expected f64"),
        };
        assert!((expected - 0.5).abs() < 1e-12);

        let source = MixSource {
            pcm: stereo_cache(vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0]),
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(3, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            time_map: TimeMap::IDENTITY,
            gain: Some(track),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (out, _) = mix_audio(&[source], 1.0, 1, 1, None).unwrap();
        assert!((out[0] as f64 - expected).abs() < 1e-6);
        assert!((out[1] as f64 - expected).abs() < 1e-6);
    }

    /// 決定論(発注書の指示: 「決定論のテスト(同入力→byte一致)を先に」)。
    /// 同じ入力を2回 mix しても byte 単位で一致する — preview/export が同じ関数を
    /// 通る前提(GOALS M15)の音声側の土台。
    #[test]
    fn same_input_mixes_to_byte_identical_output() {
        let a = stereo_cache(
            (0..64)
                .flat_map(|i| [(i as f32 * 0.013).sin(), (i as f32 * 0.029).cos()])
                .collect(),
        );
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::ZERO,
            value: Value::F64(0.2),
            interp: Interp::Linear,
            spatial: None,
        });
        track.insert(Keyframe {
            t: RationalTime::try_new(64, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            value: Value::F64(0.9),
            interp: Interp::Bezier {
                x1: 0.25,
                y1: 0.1,
                x2: 0.75,
                y2: 0.9,
            },
            spatial: None,
        });
        let source = MixSource {
            pcm: a,
            timeline_start: RationalTime::ZERO,
            timeline_duration: RationalTime::try_new(64, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            time_map: TimeMap::constant_speed(RationalTime::ZERO, 3, 2).unwrap(),
            gain: Some(track),
            pan: None,
            fade: FadeSpec::NONE,
            out_of_range: AudioOutOfRange::Loop,
            enabled: true,
        };
        let meter = AudioMeter::new();
        let (first, first_report) = mix_audio(&[source.clone()], 0.8, 0, 64, Some(&meter)).unwrap();
        let (second, second_report) = mix_audio(&[source], 0.8, 0, 64, Some(&meter)).unwrap();
        assert_eq!(
            first, second,
            "mix_audio は同一入力に対しbyte一致でなければならない"
        );
        assert_eq!(first_report, second_report);
    }

    // ---- B42: gain の線形性 ----------------------------------------------

    #[test]
    fn gain_scales_output_linearly() {
        let a = stereo_cache(vec![0.4, -0.2]);
        let (base, _) = mix_audio(&[identity_source(a.clone(), 1.0)], 1.0, 0, 1, None).unwrap();
        let (scaled, _) = mix_audio(&[identity_source(a, 2.5)], 1.0, 0, 1, None).unwrap();
        assert!((scaled[0] - base[0] * 2.5).abs() < 1e-9);
        assert!((scaled[1] - base[1] * 2.5).abs() < 1e-9);
    }

    #[test]
    fn gain_zero_is_silence_without_affecting_other_sources() {
        let a = stereo_cache(vec![1.0, 1.0]);
        let b = stereo_cache(vec![0.5, 0.5]);
        let (out, _) = mix_audio(
            &[identity_source(a, 0.0), identity_source(b, 1.0)],
            1.0,
            0,
            1,
            None,
        )
        .unwrap();
        assert_eq!(out, vec![0.5, 0.5]);
    }

    // ---- B42: pan(等パワー則、W3C Web Audio StereoPannerNode 相当) --------

    #[test]
    fn pan_center_is_identity() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, 0.0);
        assert!((l - 0.6).abs() < 1e-12);
        assert!((r - 0.4).abs() < 1e-12);
    }

    #[test]
    fn pan_hard_left_sums_both_channels_into_left() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, -1.0);
        assert!((l - 1.0).abs() < 1e-12, "l={l}");
        assert!(r.abs() < 1e-12, "r={r}");
    }

    #[test]
    fn pan_hard_right_sums_both_channels_into_right() {
        let (l, r) = apply_pan_stereo(0.6, 0.4, 1.0);
        assert!(l.abs() < 1e-12, "l={l}");
        assert!((r - 1.0).abs() < 1e-12, "r={r}");
    }

    #[test]
    fn pan_out_of_range_values_clamp_instead_of_erroring() {
        let hard_right = apply_pan_stereo(0.6, 0.4, 1.0);
        let past_right = apply_pan_stereo(0.6, 0.4, 5.0);
        assert_eq!(hard_right, past_right);
        let hard_left = apply_pan_stereo(0.6, 0.4, -1.0);
        let past_left = apply_pan_stereo(0.6, 0.4, -5.0);
        assert_eq!(hard_left, past_left);
    }

    #[test]
    fn pan_field_routes_through_mix_audio() {
        let pcm = stereo_cache(vec![0.5, 0.5]); // 1 frame, L=R=0.5
        let mut source = identity_source(pcm, 1.0);
        source.pan = const_track(-1.0);
        let (out, _) = mix_audio(&[source], 1.0, 0, 1, None).unwrap();
        assert!((out[0] - 1.0).abs() < 1e-9, "L should sum both channels: {out:?}");
        assert!(out[1].abs() < 1e-9, "R should be silenced: {out:?}");
    }

    #[test]
    fn pan_none_track_defaults_to_center() {
        // 裁定20と同型: track無し = 静止値(pan=0.0=中央=無変化)。
        assert_eq!(eval_pan_at(&None, RationalTime::ZERO).unwrap(), 0.0);
    }

    // ---- B42: fade の境界 --------------------------------------------------

    #[test]
    fn linear_fade_in_ramps_from_zero_to_full() {
        let pcm = stereo_cache([1.0, 1.0].repeat(4));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: dur,
            fade_out: RationalTime::ZERO,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        assert!(out[0].abs() < 1e-6, "frame0 should be silent: {out:?}");
        assert!((out[2] - 0.25).abs() < 1e-5, "frame1: {out:?}");
        assert!((out[4] - 0.5).abs() < 1e-5, "frame2: {out:?}");
        assert!((out[6] - 0.75).abs() < 1e-5, "frame3: {out:?}");
    }

    #[test]
    fn linear_fade_out_ramps_from_full_to_zero() {
        let pcm = stereo_cache([1.0, 1.0].repeat(4));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: RationalTime::ZERO,
            fade_out: dur,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 4, None).unwrap();
        // remaining/out_len: frame0 -> 4/4=1.0, frame1 -> 3/4, frame2 -> 2/4, frame3 -> 1/4
        assert!((out[0] - 1.0).abs() < 1e-5, "frame0: {out:?}");
        assert!((out[2] - 0.75).abs() < 1e-5, "frame1: {out:?}");
        assert!((out[4] - 0.5).abs() < 1e-5, "frame2: {out:?}");
        assert!((out[6] - 0.25).abs() < 1e-5, "frame3: {out:?}");
    }

    #[test]
    fn equal_power_fade_differs_from_linear_at_midpoint() {
        let linear = curve_value(0.5, FadeCurve::Linear);
        let equal_power = curve_value(0.5, FadeCurve::EqualPower);
        assert!((linear - 0.5).abs() < 1e-12);
        assert!((equal_power - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
        assert!(equal_power > linear, "等パワー則は中間点でlinearより大きいはず");
    }

    #[test]
    fn overlapping_fade_in_and_out_multiply_without_leaving_unit_range() {
        let dur = RationalTime::try_new(4, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let fade = FadeSpec {
            fade_in: dur,
            fade_out: dur,
            curve: FadeCurve::Linear,
        };
        for i in 0..4i64 {
            let local = RationalTime::try_new(i, CANONICAL_SAMPLE_RATE as i64).unwrap();
            let envelope = fade_envelope(local, dur, &fade);
            assert!(
                (0.0..=1.0).contains(&envelope),
                "envelope out of [0,1] at frame {i}: {envelope}"
            );
        }
        // 両端(0とduration-1に最も近いフレーム)は最も暗い — 単調である必要はないが
        // 中央がどちらの端よりも明るいことは保証されるべき(オーバーラップの意味)。
        let start = fade_envelope(RationalTime::ZERO, dur, &fade);
        let mid = fade_envelope(
            RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            dur,
            &fade,
        );
        assert!(mid > start, "中央({mid})は端({start})より明るいはず");
    }

    #[test]
    fn fade_duration_exceeding_clip_length_is_clamped_not_rejected() {
        // fade_in が clip 全長より長くても mix_audio が panic/errorしない(境界試験)。
        let pcm = stereo_cache([1.0, 1.0].repeat(2));
        let mut source = identity_source(pcm, 1.0);
        let dur = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        source.timeline_duration = dur;
        source.fade = FadeSpec {
            fade_in: RationalTime::try_new(100, CANONICAL_SAMPLE_RATE as i64).unwrap(),
            fade_out: RationalTime::ZERO,
            curve: FadeCurve::Linear,
        };
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        assert!(out.iter().all(|s| s.is_finite()));
        assert!(out[0].abs() < 1e-6, "先頭はほぼ無音のはず: {out:?}");
    }

    /// 決定論(発注書「決定論(同入力→同出力の既存保証を維持)」): pan/fade を
    /// 使っても `same_input_mixes_to_byte_identical_output` と同じ保証が壊れない。
    #[test]
    fn same_input_with_pan_and_fade_mixes_to_byte_identical_output() {
        let pcm = stereo_cache(
            (0..32)
                .flat_map(|i| [(i as f32 * 0.037).sin(), (i as f32 * 0.051).cos()])
                .collect(),
        );
        let dur = RationalTime::try_new(32, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let mut source = MixSource {
            pcm,
            timeline_start: RationalTime::ZERO,
            timeline_duration: dur,
            time_map: TimeMap::IDENTITY,
            gain: const_track(0.7),
            pan: const_track(-0.3),
            fade: FadeSpec {
                fade_in: RationalTime::try_new(8, CANONICAL_SAMPLE_RATE as i64).unwrap(),
                fade_out: RationalTime::try_new(8, CANONICAL_SAMPLE_RATE as i64).unwrap(),
                curve: FadeCurve::EqualPower,
            },
            out_of_range: AudioOutOfRange::Silence,
            enabled: true,
        };
        let (first, first_report) = mix_audio(&[source.clone()], 1.0, 0, 32, None).unwrap();
        let (second, second_report) = mix_audio(&[source.clone()], 1.0, 0, 32, None).unwrap();
        assert_eq!(first, second, "pan/fade込みでも同一入力はbyte一致でなければならない");
        assert_eq!(first_report, second_report);

        // gain==0 の場合と同型のガード: pan/fadeの評価はsourceを消費しない(&mut不要)。
        source.enabled = false;
        let (silent, report) = mix_audio(&[source], 1.0, 0, 32, None).unwrap();
        assert!(silent.iter().all(|&s| s == 0.0));
        assert_eq!(report.silence_frames, 32);
    }

    // ---- B42: 正規化(peak-based、id42の薄い顔) -----------------------------

    #[test]
    fn normalize_gain_for_peak_computes_linear_scalar() {
        let pcm = stereo_cache(vec![0.5, -0.25, 0.4, -0.1]); // |peak| = 0.5
        let gain = normalize_gain_for_peak(&pcm, 1.0).unwrap();
        assert!((gain - 2.0).abs() < 1e-9, "gain={gain}");
    }

    #[test]
    fn normalize_gain_for_peak_silence_is_noop() {
        let pcm = stereo_cache(vec![0.0, 0.0, 0.0, 0.0]);
        assert_eq!(normalize_gain_for_peak(&pcm, 1.0).unwrap(), 1.0);
    }

    #[test]
    fn normalize_gain_for_peak_rejects_invalid_target() {
        let pcm = stereo_cache(vec![0.5, 0.5]);
        assert!(normalize_gain_for_peak(&pcm, -1.0).is_err());
        assert!(normalize_gain_for_peak(&pcm, f64::NAN).is_err());
        assert!(normalize_gain_for_peak(&pcm, f64::INFINITY).is_err());
    }

    #[test]
    fn normalize_gain_applied_through_existing_gain_path_hits_target_peak() {
        // 正規化gainを既存の `MixSource::gain`(Hold keyframe)へそのまま乗せて、
        // mix結果のpeakがtarget_peakに一致することを確認する — 「既存gain経路を
        // 増やさず薄い顔として通す」という設計の実証。
        let target_peak = 0.8_f64;
        let pcm = stereo_cache(vec![0.25f32, -0.25, 0.5, -0.5]); // |peak| = 0.5
        let gain_value = normalize_gain_for_peak(&pcm, target_peak).unwrap();

        let mut source = identity_source(pcm, gain_value);
        source.timeline_duration = RationalTime::try_new(2, CANONICAL_SAMPLE_RATE as i64).unwrap();
        let (out, _) = mix_audio(&[source], 1.0, 0, 2, None).unwrap();
        let peak = out.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        assert!((peak as f64 - target_peak).abs() < 1e-6, "peak={peak}");
    }
}
