//! AG-4: fast path維持と mixed PCM export の審判。

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use motolii_audio::AudioProgram;
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, AudioComponent, Clip, ClipSource, Composition, DocParam, Document,
    ItemEnvelope, Soundtrack, Track, TrackItem, VideoComponent, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob};
use motolii_media::{choose_audio_encode_mode, probe, probe_audio, AudioEncodeMode, Encoder};
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};
const N_FRAMES: usize = 12;

fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    let frame = vec![0u8; desc.data_size()];
    for _ in 0..N_FRAMES {
        enc.write_frame(&frame).unwrap();
    }
    enc.finish().unwrap();
}

fn make_aac(path: &Path, seconds: f64) {
    let status = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency=440:sample_rate=48000:duration={seconds}"),
            "-c:a",
            "aac",
            "-b:a",
            "128k",
        ])
        .arg(path)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
}

fn make_wav_stereo(path: &Path, seconds: f64, freq: f32) {
    let status = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("sine=frequency={freq}:sample_rate=48000:duration={seconds}"),
            "-ac",
            "2",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(path)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success());
}

fn extract_f32_stereo(path: &Path, out: &Path, duration: &str) {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"])
        .arg(path)
        .args([
            "-t", duration, "-vn", "-ac", "2", "-ar", "48000", "-f", "f32le",
        ])
        .arg(out)
        .status()
        .expect("spawn");
    assert!(status.success(), "pcm extract failed");
}

fn build_video_doc(video_name: &str) -> Document {
    let mut doc = Document::new_v1();
    doc.version = 3;
    doc.min_reader_version = 3;
    doc.composition = Composition::try_new(
        W as i64,
        H as i64,
        RationalTime::try_new(N_FRAMES as i64, 12).unwrap(),
        FPS,
    )
    .unwrap();

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:ag4-bg".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some(video_name.into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();

    let layer = doc.layers.allocate("bg").unwrap();
    let overlay_layer = doc.layers.allocate("overlay").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let clip_duration = RationalTime::try_new(N_FRAMES as i64, 12).unwrap();
    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: clip_duration,
                time_map: TimeMap::identity(),
                source: ClipSource::asset_video_only(video_id),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(overlay_layer),
                start: RationalTime::ZERO,
                duration: clip_duration,
                time_map: TimeMap::identity(),
                source: ClipSource::Plugin {
                    plugin_id: RECT_LAYER_SOURCE.into(),
                    effect_version: 1,
                    params: BTreeMap::from([
                        ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                        ("size".into(), DocParam::const_vec2([0.2, 0.2])),
                        ("color".into(), DocParam::const_color([1.0, 0.0, 0.0, 0.0])),
                    ]),
                    extra: Default::default(),
                },
            }),
        ],
    });
    doc
}

#[test]
fn soundtrack_only_still_prefers_stream_copy_mode() {
    if !ffmpeg_or_skip() {
        return;
    }
    let audio = std::env::temp_dir().join("ag4-st.aac");
    make_aac(&audio, 1.5);
    let info = probe_audio(&audio).unwrap();
    assert_eq!(
        choose_audio_encode_mode(&info.codec_name, 1.0),
        AudioEncodeMode::StreamCopy
    );
}

#[test]
fn clip_audio_forces_mixed_export_and_matches_preview_pcm() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("ag4-mixed");
    let video = dir.join("bg.mp4");
    let wav = dir.join("clip.wav");
    let out = dir.join("out.mp4");
    make_bg_video(&video);
    make_wav_stereo(&wav, 1.2, 880.0);

    let mut doc = build_video_doc("bg.mp4");
    let aid = AssetId::from_raw(1);
    doc.assets
        .insert(Asset {
            id: aid,
            name: "clip-a".into(),
            asset_type: "audio/wav".into(),
            content_hash: "sha256:clip-a".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some("clip.wav".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    let layer = doc.layers.allocate("AUD").unwrap();
    let track = doc.track_ids.allocate("A1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(N_FRAMES as i64, 12).unwrap(),
            time_map: TimeMap::IDENTITY,
            source: ClipSource::Asset {
                asset: aid,
                video: None,
                audio: vec![AudioComponent::ordinal(0)],
            },
        })],
    });
    doc.validate().unwrap();

    let report = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            output_path: &out,
            project_root: Some(&dir),
            frame_count: Some(N_FRAMES),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .expect("mixed export");
    assert_eq!(report.frames_written, N_FRAMES);
    let info = probe(&out).unwrap();
    assert_eq!((info.width, info.height), (W, H));

    let pcm = dir.join("got.f32");
    extract_f32_stereo(&out, &pcm, "0.5");
    let bytes = std::fs::read(&pcm).unwrap();
    assert!(bytes.len() > 1000);

    let mut caches = std::collections::HashMap::new();
    let program = AudioProgram::from_document(&doc, Some(&dir), &mut caches).unwrap();
    let (want, _) = program.mix_audio(0, 480, None).unwrap();
    let got: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    let n = want.len().min(got.len()).min(200);
    for i in 0..n {
        let d = (want[i] - got[i]).abs();
        assert!(
            d < 0.2,
            "sample {i} preview={} export={} delta={d}",
            want[i],
            got[i]
        );
    }
}

#[test]
fn soundtrack_plus_clip_audio_takes_mixed_path() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else {
        return;
    };
    let dir = tmp_dir("ag4-both");
    let video = dir.join("bg.mp4");
    let bed = dir.join("bed.wav");
    let clip = dir.join("clip.wav");
    let out = dir.join("out.mp4");
    make_bg_video(&video);
    make_wav_stereo(&bed, 1.5, 440.0);
    make_wav_stereo(&clip, 1.2, 220.0);

    let mut doc = build_video_doc("bg.mp4");
    let bed_id = AssetId::from_raw(1);
    let clip_id = AssetId::from_raw(2);
    doc.assets
        .insert(Asset {
            id: bed_id,
            name: "bed".into(),
            asset_type: "audio/wav".into(),
            content_hash: "sha256:bed".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some("bed.wav".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    doc.assets
        .insert(Asset {
            id: clip_id,
            name: "c".into(),
            asset_type: "audio/wav".into(),
            content_hash: "sha256:c".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: Some("clip.wav".into()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    doc.soundtrack = Some(Soundtrack::try_new(bed_id, RationalTime::ZERO, 1.0).unwrap());
    let layer = doc.layers.allocate("AUD").unwrap();
    let track = doc.track_ids.allocate("A1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(N_FRAMES as i64, 12).unwrap(),
            time_map: TimeMap::IDENTITY,
            source: ClipSource::Asset {
                asset: clip_id,
                video: None,
                audio: vec![AudioComponent::ordinal(0)],
            },
        })],
    });
    doc.validate().unwrap();

    export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            output_path: &out,
            project_root: Some(&dir),
            frame_count: Some(N_FRAMES),
            qp0: true,
            data_tracks: DataTracks::new(),
        },
    )
    .expect("bed+clip mixed export");

    let pcm = dir.join("got.f32");
    extract_f32_stereo(&out, &pcm, "0.3");
    assert!(std::fs::metadata(&pcm).unwrap().len() > 1000);
}

#[test]
fn video_component_helper_still_compiles_for_asset_shape() {
    // schema sanity: AG-4は公開契約を変えない
    let _ = VideoComponent::ordinal(0);
}
