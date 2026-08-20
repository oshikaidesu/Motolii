//! AG-2完了条件(移植): `StoreView` 由来 `AudioProgram` の組み立てと `mix_audio` の
//! 決定論。旧 `crates/motolii-audio/tests/mix_program.rs` のうち、device/ring 経由の
//! 試験(`MixProducer`/`fill_or_silence`/`PlaybackCounters` の実デバイス相当)は
//! 第2切片へ持ち越し、store 由来の `AudioProgram` 組み立てだけをこの束で見る。
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::collections::HashMap;

use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value};
use motolii_store::{
    property, Composition, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource,
    LayerTiming, PropertyId, RationalTime, Speed,
};

use motolii_audio::AudioProgram;

use support::write_pcm16_wav;

fn fps30() -> Fps {
    Fps::try_new(30, 1).unwrap()
}

fn doc_with_comp(duration_frames: i64) -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: fps30(),
        duration_frames,
    }))
    .unwrap();
    doc
}

fn tempdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-audio-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn media_meta(path: &std::path::Path, start: i64, duration: i64) -> LayerMeta {
    LayerMeta {
        source: LayerSource::Media {
            path: path.to_string_lossy().into_owned(),
            fingerprint: None,
        },
        order: 0,
        timing: LayerTiming {
            start,
            duration,
            source_in: 0,
            speed: Speed::NORMAL,
        },
    }
}

#[test]
fn media_layers_become_mix_sources() {
    let dir = tempdir("basic");
    let bed = dir.join("bed.wav");
    let clip = dir.join("clip.wav");
    write_pcm16_wav(&bed, 48_000, 2, &[8_000, 8_000].repeat(480));
    write_pcm16_wav(&clip, 48_000, 2, &[-8_000, -8_000].repeat(480));

    let mut doc = doc_with_comp(30); // 1s @30fps
    let bed_layer = LayerId(1);
    let clip_layer = LayerId(2);
    doc.apply_all([
        Intent::AddLayer(bed_layer),
        Intent::SetMeta {
            layer: bed_layer,
            meta: media_meta(&bed, 0, 30),
        },
        Intent::AddLayer(clip_layer),
        Intent::SetMeta {
            layer: clip_layer,
            meta: media_meta(&clip, 0, 15),
        },
    ])
    .unwrap();

    // clip の gain を 0.5 に(標準 property `level`、単一 Hold キー)。
    let mut gain_track = KeyframeTrack::new();
    gain_track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::F64(0.5),
        interp: Interp::Hold,
        spatial: None,
    });
    doc.apply(Intent::SetTrack {
        layer: clip_layer,
        property: PropertyId::new(property::LEVEL).unwrap(),
        track: gain_track,
    })
    .unwrap();

    let view = doc.view();
    let mut caches = HashMap::new();
    let program = AudioProgram::from_view(&view, &mut caches).unwrap();
    assert_eq!(program.sources().len(), 2);
    assert_eq!(
        program.composition_duration(),
        RationalTime::from_seconds(1)
    );

    let (out, _report) = program.mix_audio(0, 1, None).unwrap();
    // bed ≈ 8000/32768 ≈ 0.244、clip ≈ -8000/32768 * gain(0.5) ≈ -0.122。
    // 和 ≈ 0.122。
    assert!(
        out[0] > 0.08 && out[0] < 0.16,
        "unexpected mixed sample: {}",
        out[0]
    );
    assert_eq!(out[0], out[1], "この fixture は L=R");
}

#[test]
fn hidden_layer_is_excluded_from_the_program() {
    let dir = tempdir("hidden");
    let bed = dir.join("bed.wav");
    write_pcm16_wav(&bed, 48_000, 2, &[8_000, 8_000].repeat(480));

    let mut doc = doc_with_comp(30);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: media_meta(&bed, 0, 30),
        },
        Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                hidden: Some(true),
                ..Default::default()
            },
        },
    ])
    .unwrap();

    let view = doc.view();
    let mut caches = HashMap::new();
    let program = AudioProgram::from_view(&view, &mut caches).unwrap();
    assert!(
        program.sources().is_empty(),
        "hidden な layer は音声にも出てはいけない"
    );
}

#[test]
fn non_media_layers_contribute_no_audio() {
    let mut doc = doc_with_comp(30);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Solid {
                    rgba: [255, 0, 0, 255],
                    width: 64,
                    height: 64,
                },
                order: 0,
                timing: LayerTiming {
                    start: 0,
                    duration: 30,
                    source_in: 0,
                    speed: Speed::NORMAL,
                },
            },
        },
    ])
    .unwrap();

    let view = doc.view();
    let mut caches = HashMap::new();
    let program = AudioProgram::from_view(&view, &mut caches).unwrap();
    assert!(program.sources().is_empty());
}

#[test]
fn silent_media_without_audio_track_is_skipped_not_fatal() {
    // ffmpeg 無しでも再現できる「音声を持たない素材」= 空のWAVコンテナではなく、
    // symphonia が音声トラック無しと判定する最小ケースの代わりに、そもそも
    // decode できないファイルを渡すと `AudioError` が伝播することだけ確認する
    // (壊れたファイルは skip しない、という契約)。
    let dir = tempdir("broken");
    let broken = dir.join("broken.wav");
    std::fs::write(&broken, b"not a wav file at all").unwrap();

    let mut doc = doc_with_comp(30);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: media_meta(&broken, 0, 30),
        },
    ])
    .unwrap();

    let view = doc.view();
    let mut caches = HashMap::new();
    let result = AudioProgram::from_view(&view, &mut caches);
    assert!(
        result.is_err(),
        "壊れたファイルは「音声が無い」として黙って skip してはいけない"
    );
}

#[test]
fn reverse_speed_is_rejected_not_silently_forward() {
    let dir = tempdir("reverse");
    let bed = dir.join("bed.wav");
    write_pcm16_wav(&bed, 48_000, 2, &[8_000, 8_000].repeat(480));

    let mut doc = doc_with_comp(30);
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::Media {
                    path: bed.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming {
                    start: 0,
                    duration: 30,
                    source_in: 0,
                    speed: Speed::try_new(-1, 1).unwrap(),
                },
            },
        },
    ])
    .unwrap();

    let view = doc.view();
    let mut caches = HashMap::new();
    assert!(AudioProgram::from_view(&view, &mut caches).is_err());
}
