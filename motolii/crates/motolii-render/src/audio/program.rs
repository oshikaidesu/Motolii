/* motolii-component
id = "audio.media_soundtrack_input"
kind = "semantic"
weight = "render_export"
maps = []
entry = ["AudioProgram::from_view"]
meaning = ["project_soundtrack_input"]
evaluation = ["layer_mix_source"]
render = ["MixSource"]
observable = ["media_layers_become_mix_sources"]
*/

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::{property, LayerId, LayerSource, PropertyId, StoreView};

use crate::render::audio::cache::PcmCache;
use crate::render::audio::convert::CANONICAL_SAMPLE_RATE;
use crate::render::audio::decode::decode_file_audio_ordinal;
use crate::render::audio::error::{AudioError, Result};
use crate::render::audio::meter::AudioMeter;
use crate::render::audio::mix::{
    mix_audio, AudioOutOfRange, FadeCurve, FadeSpec, MixReport, MixSource,
};
use crate::render::audio::time_map::TimeMap;
use crate::render::audio::waveform::{WaveformPeaks, WaveformTrack};

#[derive(Debug, Clone)]
pub struct AudioProgram {
    sources: Vec<MixSource>,
    waveform_tracks: Vec<WaveformTrack>,
    master_gain: f64,
    composition_duration: RationalTime,
}

enum PcmSlot {
    Ready(Arc<PcmCache>),
    Loading(std::sync::mpsc::Receiver<Result<PcmCache>>),
    /// 音のトラックが無い、または読めなかった。何度も開かない。
    Absent,
}

#[derive(Default)]
pub struct AudioProgramCache {
    pcm: HashMap<(String, u32), PcmSlot>,
    waveform: HashMap<usize, Arc<WaveformPeaks>>,
}

impl AudioProgramCache {
    /// 裏で復号中の物が届いていれば取り込む。届いた物があれば true(program を組み直す合図)。
    pub fn poll(&mut self) -> bool {
        let mut arrived = false;
        for slot in self.pcm.values_mut() {
            let PcmSlot::Loading(rx) = slot else { continue };
            match rx.try_recv() {
                Ok(result) => {
                    *slot = settle(result);
                    arrived = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    *slot = PcmSlot::Absent;
                    arrived = true;
                }
            }
        }
        arrived
    }

    pub fn loading(&self) -> bool {
        self.pcm.values().any(|slot| matches!(slot, PcmSlot::Loading(_)))
    }

    /// 復号中の物を全部待つ。書き出しは音が揃ってから始める。
    pub fn wait_all(&mut self) {
        for slot in self.pcm.values_mut() {
            let PcmSlot::Loading(rx) = slot else { continue };
            *slot = match rx.recv() {
                Ok(result) => settle(result),
                Err(_) => PcmSlot::Absent,
            };
        }
    }
}

fn settle(result: Result<PcmCache>) -> PcmSlot {
    match result {
        Ok(pcm) => PcmSlot::Ready(Arc::new(pcm)),
        Err(AudioError::NoAudioTrack) | Err(AudioError::StreamNotFound { .. }) => PcmSlot::Absent,
        Err(err) => {
            eprintln!("motolii-audio: decode failed: {err}");
            PcmSlot::Absent
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoundtrackInput {
    path: String,
    cache_key: String,
}

fn project_soundtrack_input(
    meta: &crate::doc::store::LayerMeta,
    hidden: bool,
) -> Option<SoundtrackInput> {
    if hidden {
        return None;
    }
    let LayerSource::File { path, fingerprint } = &meta.source else {
        return None;
    };
    if !file_source_can_have_audio(path) {
        return None;
    }
    Some(SoundtrackInput {
        path: path.clone(),
        cache_key: fingerprint.clone().unwrap_or_else(|| path.clone()),
    })
}

pub fn file_source_can_have_audio(path: &str) -> bool {
    let Some(extension) = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    else {
        // 古いprojectや外部生成Documentの未知形式は、従来どおりSymphoniaへ委ねる。
        return true;
    };
    match crate::render::media::asset_type_for_extension(extension) {
        Some(asset_type) => asset_type.starts_with("audio/") || asset_type.starts_with("video/"),
        None => true,
    }
}

impl AudioProgram {
    pub fn from_view(
        view: &StoreView<'_>,
        cache: &mut AudioProgramCache,
    ) -> Result<Self> {
        let Some(composition) = view.composition()? else {
            return Ok(Self {
                sources: Vec::new(),
                waveform_tracks: Vec::new(),
                master_gain: 1.0,
                composition_duration: RationalTime::ZERO,
            });
        };
        let fps = composition.fps;
        let composition_duration = RationalTime::try_from_frame(composition.duration_frames, fps)
            .map_err(|_| AudioError::InvalidMixRange)?;

        let mut sources = Vec::new();
        let mut waveform_tracks = Vec::new();
        // S(solo)は音の系(DAW・NLE)。誰かが solo なら、その層だけ鳴る。
        let soloed: Vec<_> = view
            .layers()
            .into_iter()
            .filter(|l| view.attrs(*l).ok().flatten().is_some_and(|a| a.solo))
            .collect();
        for layer in view.layers() {
            if !soloed.is_empty() && !soloed.contains(&layer) {
                continue;
            }
            match layer_mix_source(view, layer, fps, &mut cache.pcm) {
                Ok(Some(source)) => {
                    let pcm_key = Arc::as_ptr(&source.pcm) as usize;
                    let peaks = cache
                        .waveform
                        .entry(pcm_key)
                        .or_insert_with(|| WaveformPeaks::spawn(Arc::clone(&source.pcm)))
                        .clone();
                    waveform_tracks.push(WaveformTrack {
                        layer,
                        peaks,
                        timeline_start: source.timeline_start,
                        timeline_duration: source.timeline_duration,
                        time_map: source.time_map,
                    });
                    sources.push(source);
                }
                Ok(None)
                | Err(AudioError::NoAudioTrack)
                | Err(AudioError::StreamNotFound { .. }) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(Self {
            sources,
            waveform_tracks,
            master_gain: 1.0,
            composition_duration,
        })
    }

    /// 音が全部届くまで待って組む。書き出しはこちら。
    pub fn from_view_blocking(view: &StoreView<'_>, cache: &mut AudioProgramCache) -> Result<Self> {
        loop {
            let program = Self::from_view(view, cache)?;
            if !cache.loading() {
                return Ok(program);
            }
            cache.wait_all();
        }
    }

    pub fn sources(&self) -> &[MixSource] {
        &self.sources
    }

    pub fn waveform_tracks(&self) -> &[WaveformTrack] {
        &self.waveform_tracks
    }

    pub fn master_gain(&self) -> f64 {
        self.master_gain
    }

    pub fn composition_duration(&self) -> RationalTime {
        self.composition_duration
    }

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
    fps: crate::doc::core::Fps,
    caches: &mut HashMap<(String, u32), PcmSlot>,
) -> Result<Option<MixSource>> {
    let Some(meta) = view.meta(layer)? else {
        return Ok(None);
    };
    let attrs = view.attrs(layer)?.unwrap_or_default();
    let Some(input) = project_soundtrack_input(&meta, attrs.hidden) else {
        return Ok(None);
    };

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

    let Some(pcm) = load_canonical_stream(Path::new(&input.path), &input.cache_key, 0, caches) else {
        return Ok(None);
    };

    let gain = view.track(layer, &PropertyId::new(property::LEVEL)?)?;
    let pan = view.track(layer, &PropertyId::new(property::PAN)?)?;
    let fade = FadeSpec {
        fade_in: fade_seconds_at(view, layer, property::FADE_IN)?,
        fade_out: fade_seconds_at(view, layer, property::FADE_OUT)?,
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
    let samples = (seconds * f64::from(CANONICAL_SAMPLE_RATE)).round() as i64;
    RationalTime::try_new(samples, CANONICAL_SAMPLE_RATE as i64).map_err(AudioError::Time)
}

/// 在れば返す。無ければ裏で復号を始めて `None`(その層は届くまで絵だけで走る)。
fn load_canonical_stream(
    path: &Path,
    cache_key: &str,
    ordinal: u32,
    caches: &mut HashMap<(String, u32), PcmSlot>,
) -> Option<Arc<PcmCache>> {
    let key = (cache_key.to_string(), ordinal);
    match caches.get(&key) {
        Some(PcmSlot::Ready(pcm)) => return Some(Arc::clone(pcm)),
        Some(PcmSlot::Loading(_)) | Some(PcmSlot::Absent) => return None,
        None => {}
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let path = path.to_path_buf();
    match std::thread::Builder::new()
        .name("motolii-audio-decode".into())
        .spawn(move || {
            let result = decode_file_audio_ordinal(&path, ordinal);
            println!(
                "PROBE room=audio verdict={} path={}",
                if result.is_ok() { "decoded" } else { "absent" },
                path.display()
            );
            let _ = tx.send(result);
        })
    {
        Ok(_) => caches.insert(key, PcmSlot::Loading(rx)),
        Err(_) => caches.insert(key, PcmSlot::Absent),
    };
    None
}

pub fn program_from_sources(
    sources: Vec<MixSource>,
    master_gain: f64,
    composition_duration: RationalTime,
) -> AudioProgram {
    AudioProgram {
        sources,
        waveform_tracks: Vec::new(),
        master_gain,
        composition_duration,
    }
}

/// 置いた瞬間は絵だけで走り、音は裏で復号して後から付く。書き出しは揃うまで待つ。
#[cfg(test)]
mod background_decode {
    use super::{AudioProgram, AudioProgramCache};
    use crate::doc::store::{Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming};

    fn ffmpeg_available() -> bool {
        crate::render::media::test_encoders_available(&[])
    }

    #[test]
    fn audio_arrives_after_the_layer_is_placed_and_export_waits_for_it() {
        if !ffmpeg_available() {
            eprintln!("skip: ffmpeg not on PATH");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let wav = dir.path().join("tone.wav");
        let status = crate::render::media::tool_command(crate::render::media::ffmpeg_bin())
            .args(["-v", "error", "-y", "-f", "lavfi", "-i", "sine=frequency=440:sample_rate=48000:duration=1", "-c:a", "pcm_s16le"])
            .arg(&wav)
            .status()
            .unwrap();
        assert!(status.success());

        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 16,
            height: 16,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 30,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File { path: wav.to_str().unwrap().to_owned(), fingerprint: None },
                order: 0,
                timing: LayerTiming::place(0, Some(30), 30),
            },
        })
        .unwrap();

        let mut cache = AudioProgramCache::default();
        let first = AudioProgram::from_view(&doc.view(), &mut cache).unwrap();
        assert!(first.sources().is_empty(), "the first build must not wait for the decode");
        assert!(cache.loading());

        let blocking = AudioProgram::from_view_blocking(&doc.view(), &mut cache).unwrap();
        assert_eq!(blocking.sources().len(), 1, "export waits until the audio is there");
        assert!(!cache.loading());
        assert!(!cache.poll(), "nothing left to arrive");
    }
}
