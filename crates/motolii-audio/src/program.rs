//! AG-2: Documentから評価順固定の `AudioProgram` を組み立てる。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    resolve_asset_path, AudioComponent, AudioOutOfRange, ClipSource, Document, TrackItem,
};

use crate::cache::PcmCache;
use crate::convert::to_canonical;
use crate::decode::decode_file_audio_ordinal;
use crate::error::{AudioError, Result};
use crate::meter::AudioMeter;
use crate::mix::{mix_audio, MixReport, MixSource};

/// Document由来の音声プログラム(正準mix入力)。
#[derive(Debug, Clone)]
pub struct AudioProgram {
    sources: Vec<MixSource>,
    master_gain: f64,
}

impl AudioProgram {
    /// `Soundtrack → Track順 → item順 → component ordinal順`でsourceを列挙する。
    ///
    /// `caches` は `(content_hash, audio_ordinal) → 正準PcmCache`。無ければdecodeして埋める。
    pub fn from_document(
        doc: &Document,
        project_root: Option<&Path>,
        caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    ) -> Result<Self> {
        let mut sources = Vec::new();
        let mut master_gain = 1.0;

        if let Some(st) = &doc.soundtrack {
            master_gain = st.master_gain();
            let asset = doc
                .assets
                .get(st.asset)
                .ok_or(AudioError::InvalidMixRange)?;
            let path = resolve_asset_path(asset, project_root).ok_or(AudioError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "soundtrack asset path unresolved",
                ),
            ))?;
            let pcm = load_canonical_stream(&path, asset.content_hash.as_str(), 0, caches)?;
            let duration = frames_to_time(pcm.frame_count())?;
            sources.push(MixSource {
                pcm,
                timeline_start: RationalTime::ZERO,
                timeline_duration: duration,
                time_map: TimeMap::constant_speed(st.start_offset, 1, 1)
                    .map_err(|_| AudioError::InvalidMixRange)?,
                gain: motolii_doc::DocParam::const_f64(1.0),
                out_of_range: AudioOutOfRange::Silence,
                enabled: true,
            });
        }

        for track in &doc.tracks {
            collect_item_sources(doc, project_root, &track.items, caches, &mut sources)?;
        }

        Ok(Self {
            sources,
            master_gain,
        })
    }

    pub fn sources(&self) -> &[MixSource] {
        &self.sources
    }

    pub fn master_gain(&self) -> f64 {
        self.master_gain
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

fn collect_item_sources(
    doc: &Document,
    project_root: Option<&Path>,
    items: &[TrackItem],
    caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
    out: &mut Vec<MixSource>,
) -> Result<()> {
    for item in items {
        match item {
            TrackItem::Clip(clip) => {
                let ClipSource::Asset { asset, audio, .. } = &clip.source else {
                    continue;
                };
                let asset_meta = doc.assets.get(*asset).ok_or(AudioError::InvalidMixRange)?;
                let path = resolve_asset_path(asset_meta, project_root).ok_or_else(|| {
                    AudioError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "clip asset path unresolved",
                    ))
                })?;
                // component ordinal順はVec順(=構築時ordinal昇順を呼び出し側が保証)。
                // 安定のためordinalでソートした参照を使う。
                let mut comps: Vec<&AudioComponent> = audio.iter().collect();
                comps.sort_by_key(|c| c.stream.ordinal);
                for comp in comps {
                    if !comp.enabled {
                        continue;
                    }
                    let pcm = load_canonical_stream(
                        &path,
                        asset_meta.content_hash.as_str(),
                        comp.stream.ordinal,
                        caches,
                    )?;
                    out.push(MixSource {
                        pcm,
                        timeline_start: clip.start,
                        timeline_duration: clip.duration,
                        time_map: clip.time_map,
                        gain: comp.gain.clone(),
                        out_of_range: comp.out_of_range,
                        enabled: true,
                    });
                }
            }
            TrackItem::Group(group) => {
                collect_item_sources(doc, project_root, &group.children, caches, out)?;
            }
        }
    }
    Ok(())
}

fn load_canonical_stream(
    path: &Path,
    content_hash: &str,
    ordinal: u32,
    caches: &mut HashMap<(String, u32), Arc<PcmCache>>,
) -> Result<Arc<PcmCache>> {
    let key = (content_hash.to_string(), ordinal);
    if let Some(hit) = caches.get(&key) {
        return Ok(Arc::clone(hit));
    }
    let raw = decode_file_audio_ordinal(path, ordinal)?;
    let canonical = Arc::new(to_canonical(&raw)?);
    caches.insert(key, Arc::clone(&canonical));
    Ok(canonical)
}

fn frames_to_time(frames: u64) -> Result<RationalTime> {
    RationalTime::try_new(frames as i64, crate::convert::CANONICAL_SAMPLE_RATE as i64)
        .map_err(|_| AudioError::InvalidMixRange)
}

/// テスト用: パス解決を省略して直接sourcesを渡す。
pub fn program_from_sources(sources: Vec<MixSource>, master_gain: f64) -> AudioProgram {
    AudioProgram {
        sources,
        master_gain,
    }
}
