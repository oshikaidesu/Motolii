//! AG-2: `StoreView` から評価順固定の `AudioProgram` を組み立てる。
//!
//! 旧 `crates/motolii-audio/src/program.rs` からの移植。**旧版との構造上の違い**
//! (発注書「Document(soundtrack層 — storeの既存LayerSourceを読むだけ)」への対応):
//!
//! - 旧版は `Document.soundtrack`(単一の背景音楽 asset。`Soundtrack::master_gain`
//!   を持つ)+ `Document.tracks` の `Clip.source.audio: Vec<AudioComponent>`
//!   (clip1本が複数audio streamを持て、streamごとに`gain`/`out_of_range`/`ordinal`
//!   を持つ)から source を組んでいた。
//! - 新 store には**この2つがどちらも無い**。「足りない口」として終了報告に書く —
//!   ここでは実装しない(発注書の指示どおり)。この束が実際に組めるのは:
//!   - `view.layers()` を舐め、`LayerSource::Media` を持つ layer ごとに、
//!     **既定ordinal(0)の audio stream 1本だけ**を `MixSource` へ変換する
//!   - gain は標準 property `level`(`property::LEVEL`)の `KeyframeTrack` を
//!     そのまま渡す(track が無ければ裁定20により静止値1.0)
//!   - `attrs.hidden` な layer は音声ごと skip する(store に音声専用の mute が無い
//!     ため — 視覚と音声の mute を分離できないのも「足りない口」)
//!   - audio stream を持たない素材(静止画・音無し動画)は
//!     `AudioError::NoAudioTrack`/`StreamNotFound` を「この layer は音声を持たない」
//!     として無視する。それ以外の decode 失敗(壊れたファイル等)は伝播する
//!   - master gain の置き場が store に無い(旧 `Soundtrack::master_gain` の代わりが
//!     無い) — 1.0 固定
//!   - out_of_range(Silence/Loop)の置き場も store に無い — `Silence` 固定
//!   - `LayerTiming.speed` が負(逆再生)の layer は `AudioError::InvalidMixRange` で
//!     拒む — この crate の `TimeMap` が正の速度しか表現しない(旧
//!     `motolii-core::TimeMap` も同じ制約だった、`time_map.rs` 参照)
//!
//! **B42(2026-08-22追記)**: `mix.rs` に `MixSource::pan`/`MixSource::fade`
//! (pan・fade in/out)を実装したが、store に `property::LEVEL` に相当する
//! pan/fade 標準 property がまだ無いため、ここでは `None`/`FadeSpec::NONE`
//! (無変化)を渡すだけに留めた — **これも「足りない口」**。store 側が
//! `property::PAN` 等を持てば `layer_mix_source` は1行足すだけで結線できる
//! (`gain` と全く同じ形)。
//!
//! **STORE3結線(2026-08-22追記)**: store が `property::PAN`/`FADE_IN`/`FADE_OUT`
//! を追加した(`motolii_store::property` docコメント参照)ので、上の「足りない口」
//! を実際に結んだ。
//!
//! - **pan**: `property::PAN` は `LEVEL` と全く同じ形(-1.0..=1.0 の
//!   `KeyframeTrack`、playback全体で連続的に評価される)なので、`gain` と同じ
//!   `view.track(...)` を1行足すだけ(doc の予告どおり)。
//! - **fade_in/fade_out**: `property::FADE_IN`/`FADE_OUT` の store 上の意味は
//!   「クリップ先頭/末尾からの相対**秒**」(`f64`、`RationalTime` ではなく他の
//!   property と同じ生の数値 — `motolii_store::property::FADE_IN` のdoc参照)。
//!   対して [`FadeSpec::fade_in`]/`fade_out` は **1個の `RationalTime`**(track
//!   ではない、静的な尺)なので、gain/pan のように track をそのまま渡せない —
//!   `view.value_at(layer, &property, RationalTime::ZERO)` で1点評価してから
//!   `RationalTime` へ変換する(`fade_seconds_at` 参照)。`RationalTime::ZERO`
//!   で読む理由: この2 property は「アニメーションする量」ではなく「クリップに
//!   固定の設定値」(store 側 test
//!   `pan_fade_in_fade_out_are_plain_animatable_properties_with_no_track_meaning_disabled`
//!   も常に `t(0)` へキーを置く) — `KeyframeTrack::eval` は `t <= keys[0].t` を
//!   先頭値へclampするので、キーがどの時刻に置かれていても `RationalTime::ZERO`
//!   評価で正しく拾える。秒 → `RationalTime` の変換は engine のサンプル精度
//!   (`CANONICAL_SAMPLE_RATE` = 48kHz)を分母に採る(mix 自体が最終的にサンプル
//!   格子で評価するため、それ以上細かい分数を持っても意味が無い)。
//!   track が無ければ(`value_at` が `None`)0.0秒 = フェード無し(裁定20と同型)。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_eval::Value;
use motolii_store::{property, LayerId, LayerSource, PropertyId, StoreView};

use crate::cache::PcmCache;
use crate::convert::{to_canonical, CANONICAL_SAMPLE_RATE};
use crate::decode::decode_file_audio_ordinal;
use crate::error::{AudioError, Result};
use crate::meter::AudioMeter;
use crate::mix::{mix_audio, AudioOutOfRange, FadeCurve, FadeSpec, MixReport, MixSource};
use crate::time_map::TimeMap;

/// `StoreView` 由来の音声プログラム(正準mix入力)。
#[derive(Debug, Clone)]
pub struct AudioProgram {
    sources: Vec<MixSource>,
    master_gain: f64,
    composition_duration: RationalTime,
}

impl AudioProgram {
    /// `StoreView::layers()` の順で source を列挙する。この並びは store の重ね順
    /// (`meta.order`)とは独立 — 音声の加算に順序不変(可換)なので、`layers()` が
    /// 返す決定論的な並びにそのまま従う(同じ Document なら常に同じ順で mix する)。
    ///
    /// `caches` は `(識別キー, audio_ordinal) → 正準PcmCache`。識別キーは
    /// `LayerSource::Media::fingerprint`(無ければ `path`)。
    pub fn from_view(
        view: &StoreView<'_>,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> Result<Self> {
        let Some(composition) = view.composition()? else {
            return Ok(Self {
                sources: Vec::new(),
                master_gain: 1.0,
                composition_duration: RationalTime::ZERO,
            });
        };
        let fps = composition.fps;
        let composition_duration = RationalTime::try_from_frame(composition.duration_frames, fps)
            .map_err(|_| AudioError::InvalidMixRange)?;

        let mut sources = Vec::new();
        for layer in view.layers() {
            if let Some(source) = layer_mix_source(view, layer, fps, caches)? {
                sources.push(source);
            }
        }

        Ok(Self {
            sources,
            master_gain: 1.0,
            composition_duration,
        })
    }

    pub fn sources(&self) -> &[MixSource] {
        &self.sources
    }

    pub fn master_gain(&self) -> f64 {
        self.master_gain
    }

    /// Documentが所有するcomposition尺。
    pub fn composition_duration(&self) -> RationalTime {
        self.composition_duration
    }

    /// preview/export同一の `mix_audio` 入口。
    pub fn mix_audio(
        &self,
        start_frame: u64,
        frame_count: usize,
        meter: Option<&AudioMeter>,
    ) -> Result<(Vec<f32>, MixReport)> {
        mix_audio(
            &self.sources,
            self.master_gain,
            start_frame,
            frame_count,
            meter,
        )
    }
}

fn layer_mix_source(
    view: &StoreView<'_>,
    layer: LayerId,
    fps: motolii_core::Fps,
    caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
) -> Result<Option<MixSource>> {
    let Some(meta) = view.meta(layer)? else {
        return Ok(None);
    };
    let LayerSource::Media { path, fingerprint } = &meta.source else {
        // Solid/Null/Shape/Text は音声を持たない。
        return Ok(None);
    };
    let attrs = view.attrs(layer)?.unwrap_or_default();
    if attrs.hidden {
        return Ok(None);
    }

    let timing = meta.timing;
    if timing.speed.num() <= 0 {
        return Err(AudioError::InvalidMixRange);
    }

    let timeline_start =
        RationalTime::try_from_frame(timing.start, fps).map_err(|_| AudioError::InvalidMixRange)?;
    let timeline_duration = RationalTime::try_from_frame(timing.duration, fps)
        .map_err(|_| AudioError::InvalidMixRange)?;
    if timeline_duration <= RationalTime::ZERO {
        return Ok(None);
    }
    let source_start = RationalTime::try_from_frame(timing.source_in, fps)
        .map_err(|_| AudioError::InvalidMixRange)?;
    let time_map = TimeMap::constant_speed(source_start, timing.speed.num(), timing.speed.den())
        .map_err(|_| AudioError::InvalidMixRange)?;

    let cache_key = fingerprint.clone().unwrap_or_else(|| path.clone());
    let pcm = match load_canonical_stream(Path::new(path), &cache_key, 0, caches) {
        Ok(pcm) => pcm,
        Err(AudioError::NoAudioTrack) | Err(AudioError::StreamNotFound { .. }) => {
            // この素材には音声が無い(静止画・音無し動画) — mix対象から外す。
            return Ok(None);
        }
        Err(other) => return Err(other),
    };

    let gain = view.track(layer, &PropertyId::new(property::LEVEL)?)?;
    // pan は gain と全く同じ形(playback全体で連続評価する `KeyframeTrack`) —
    // track が無ければ `MixSource::pan` の `None`(裁定20: 静止値0.0=中央)。
    let pan = view.track(layer, &PropertyId::new(property::PAN)?)?;
    let fade = FadeSpec {
        fade_in: fade_seconds_at(view, layer, property::FADE_IN)?,
        fade_out: fade_seconds_at(view, layer, property::FADE_OUT)?,
        // store はカーブ選択を持たない(`property::FADE_IN`/`FADE_OUT` は尺のみ)
        // — `FadeSpec::default`/`NONE` と同じ既定(等パワー)を使う。
        curve: FadeCurve::default(),
    };

    Ok(Some(MixSource {
        pcm,
        timeline_start,
        timeline_duration,
        time_map,
        gain,
        pan,
        fade,
        out_of_range: AudioOutOfRange::Silence,
        enabled: true,
    }))
}

/// `property::FADE_IN`/`FADE_OUT` を「クリップ端からの相対秒」として1点評価し
/// `RationalTime` へ変換する(`FadeSpec::fade_in`/`fade_out` は track ではなく
/// 1個の静的な尺 — module doc の「STORE3結線」節参照)。
///
/// `RationalTime::ZERO` で読む: この2 propertyは値がアニメーションする対象では
/// なくクリップに固定の設定値なので、`KeyframeTrack::eval` の「`t <= keys[0].t`
/// は先頭キーの値へclamp」という挙動を利用して、キーがどの時刻に置かれていても
/// 同じ値を拾う。`view.value_at` を使う(track の生値ではなくスロット参照・
/// overlay 込みの評価値) — gain/pan と違い、これは「playback中に動く量」では
/// なく「単一のクリップ設定」で、他の scalar property(opacity 等)と同じ読み方
/// が自然なため。
///
/// track が無ければ0秒(裁定20と同型、フェード無し)。値が有限かつ0以上でない
/// 場合は `AudioError::InvalidFade` を返す(負のフェード尺は
/// `mix.rs::fade_envelope` の前提を破る)。
fn fade_seconds_at(
    view: &StoreView<'_>,
    layer: LayerId,
    property_name: &str,
) -> Result<RationalTime> {
    let property = PropertyId::new(property_name)?;
    let seconds = match view.value_at(layer, &property, RationalTime::ZERO)? {
        None => 0.0,
        Some(Value::F64(v)) => v,
        Some(_) => return Err(AudioError::InvalidFade { fade: f64::NAN }),
    };
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(AudioError::InvalidFade { fade: seconds });
    }
    // engine のサンプル精度(48kHz)を分母に採る — mix自体が最終的にサンプル格子
    // で評価するため、それ以上細かい分数を持っても意味が無い。
    let samples = (seconds * f64::from(CANONICAL_SAMPLE_RATE)).round() as i64;
    RationalTime::try_new(samples, CANONICAL_SAMPLE_RATE as i64).map_err(AudioError::Time)
}

fn load_canonical_stream(
    path: &Path,
    cache_key: &str,
    ordinal: u32,
    caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
) -> Result<Arc<PcmCache>> {
    let key = (cache_key.to_string(), ordinal);
    if let Some(hit) = caches.get(&key) {
        return Ok(Arc::clone(hit));
    }
    let raw = decode_file_audio_ordinal(path, ordinal)?;
    let canonical = Arc::new(to_canonical(&raw)?);
    caches.insert(key, Arc::clone(&canonical));
    Ok(canonical)
}

/// テスト用: source を直接渡して `AudioProgram` を作る(store を経由しない)。
pub fn program_from_sources(
    sources: Vec<MixSource>,
    master_gain: f64,
    composition_duration: RationalTime,
) -> AudioProgram {
    AudioProgram {
        sources,
        master_gain,
        composition_duration,
    }
}
