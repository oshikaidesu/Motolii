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

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_store::{property, LayerId, LayerSource, PropertyId, StoreView};

use crate::cache::PcmCache;
use crate::convert::to_canonical;
use crate::decode::decode_file_audio_ordinal;
use crate::error::{AudioError, Result};
use crate::meter::AudioMeter;
use crate::mix::{mix_audio, AudioOutOfRange, MixReport, MixSource};
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

    Ok(Some(MixSource {
        pcm,
        timeline_start,
        timeline_duration,
        time_map,
        gain,
        out_of_range: AudioOutOfRange::Silence,
        enabled: true,
    }))
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
