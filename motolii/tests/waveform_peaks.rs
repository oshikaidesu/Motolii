use std::sync::Arc;

use motolii::doc::store::{
    Composition, Document, Fps, Intent, LayerId, LayerMeta, LayerSource, LayerTiming,
};
use motolii::render::audio::{
    AudioProgram, AudioProgramCache, PcmCache, PcmFormat, WaveformPeaks, WaveformStatus,
    BASE_SCALE,
};

fn pcm(samples: Vec<f32>, channels: u16, sample_rate: u32) -> PcmCache {
    PcmCache::from_interleaved(
        samples,
        PcmFormat {
            channels,
            sample_rate,
        },
    )
    .unwrap()
}

#[test]
fn the_same_pcm_makes_the_same_peak_bytes() {
    let samples = (0..512)
        .map(|i| ((i as f32 * 0.073).sin() * 0.8).clamp(-1.0, 1.0))
        .collect::<Vec<_>>();
    let pcm = pcm(samples, 1, 48_000);

    let a = WaveformPeaks::from_pcm(&pcm).unwrap();
    let b = WaveformPeaks::from_pcm(&pcm).unwrap();

    assert_eq!(a.deterministic_bytes(), b.deterministic_bytes());
}

#[test]
fn out_of_range_pcm_is_clamped_before_the_upstream_generator() {
    let samples = (0..128)
        .map(|i| if i % 2 == 0 { -4.0 } else { 4.0 })
        .collect::<Vec<_>>();
    let pcm = pcm(samples, 1, 48_000);
    let peaks = WaveformPeaks::from_pcm(&pcm).unwrap();
    let columns = peaks.columns(0.0, 1.0, 750.0).unwrap();

    assert!(!columns.is_empty());
    assert!(columns.iter().all(|column| column.min >= -1.0 && column.max <= 1.0));
    assert!(columns.iter().all(|column| column.min <= -0.99 && column.max >= 0.99));
}

#[test]
fn coarse_levels_keep_the_envelope_of_the_fine_columns() {
    let mut samples = vec![0.0f32; 256];
    samples[8] = -0.25;
    samples[40] = 0.5;
    samples[72] = -0.75;
    samples[104] = 0.25;
    samples[136] = -0.5;
    samples[168] = 0.75;
    samples[200] = -1.0;
    samples[232] = 1.0;
    let pcm = pcm(samples, 1, 48_000);
    let peaks = WaveformPeaks::from_pcm(&pcm).unwrap();

    let fine = peaks
        .columns(0.0, 1.0, 48_000.0 / f64::from(BASE_SCALE))
        .unwrap();
    let coarse = peaks
        .columns(0.0, 1.0, 48_000.0 / f64::from(BASE_SCALE * 2))
        .unwrap();
    assert_eq!(fine.len(), 4);
    assert_eq!(coarse.len(), 2);
    for (pair, envelope) in fine.chunks_exact(2).zip(&coarse) {
        let min = pair.iter().map(|column| column.min).fold(f32::INFINITY, f32::min);
        let max = pair
            .iter()
            .map(|column| column.max)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((envelope.min - min).abs() < 1e-6);
        assert!((envelope.max - max).abs() < 1e-6);
    }
}

#[test]
fn a_five_minute_peak_pyramid_stays_small() {
    let frames = 5 * 60 * 48_000u64;
    let bound = WaveformPeaks::resident_upper_bound(frames);

    assert!(bound < 2_000_000, "five-minute mono peak pyramid is {bound} bytes");

    let small = pcm(vec![0.0; 48_000], 1, 48_000);
    let peaks = WaveformPeaks::from_pcm(&small).unwrap();
    assert!(peaks.resident_bytes().unwrap() <= WaveformPeaks::resident_upper_bound(48_000));
}

#[test]
fn a_second_audio_layer_is_only_a_second_track_value() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("two-layers.wav");
    write_silent_wav(&path, 128, 48_000);

    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    for layer in [LayerId(1), LayerId(2)] {
        doc.apply(Intent::AddLayer(layer)).unwrap();
        doc.apply(Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: Some("same-pcm".into()),
                },
                order: layer.0 as i16,
                timing: LayerTiming::place(0, None, 300),
            },
        })
        .unwrap();
    }

    let mut cache = AudioProgramCache::default();
    let program = AudioProgram::from_view(&doc.view(), &mut cache).unwrap();
    let tracks = program.waveform_tracks();

    assert_eq!(tracks.len(), 2);
    let mut layers = tracks.iter().map(|track| track.layer).collect::<Vec<_>>();
    layers.sort_by_key(|layer| layer.0);
    assert_eq!(layers, vec![LayerId(1), LayerId(2)]);
    assert!(Arc::ptr_eq(&tracks[0].peaks, &tracks[1].peaks));
    for _ in 0..100 {
        if tracks[0].peaks.status() == WaveformStatus::Ready {
            break;
        }
        std::thread::yield_now();
    }
    assert_eq!(tracks[0].peaks.status(), WaveformStatus::Ready);
    assert!(!tracks[0].columns(0.0, 1.0, 100.0).unwrap().is_empty());
}

#[test]
fn a_known_still_image_never_reaches_the_audio_probe() {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 640,
        height: 360,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: Composition::default_background(),
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply(Intent::AddLayer(layer)).unwrap();
    doc.apply(Intent::SetMeta {
        layer,
        meta: LayerMeta {
            source: LayerSource::File {
                // 存在しないので、音声probeへ入ればI/O errorになる。
                path: "/definitely/missing/still.png".to_owned(),
                fingerprint: None,
            },
            order: 0,
            timing: LayerTiming::place(0, None, 300),
        },
    })
    .unwrap();

    let mut cache = AudioProgramCache::default();
    let program = AudioProgram::from_view(&doc.view(), &mut cache).unwrap();

    assert!(program.sources().is_empty());
    assert!(program.waveform_tracks().is_empty());
}

fn write_silent_wav(path: &std::path::Path, frames: u32, sample_rate: u32) {
    let data_len = frames * 2;
    let mut wav = Vec::with_capacity((44 + data_len) as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.resize((44 + data_len) as usize, 0);
    std::fs::write(path, wav).unwrap();
}
