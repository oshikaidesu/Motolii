//! export の報告=現物 — red 先行。
//!
//! [実走観察(2026-08-18)](../../../docs/reviews/2026-08-18-first-real-run-observations.md)(2):
//! comp 10s の export が「wrote 300 frames」と報告しつつ、現物は音声(6s)側で
//! 黙って切られ 178 フレームだった。契約は2つ:
//! - **出力の video は composition の長さぶん在る**(soundtrack が短くても切らない。
//!   音が先に終わるのは普通のエディタの通常事態で、絵まで消える理由にならない)
//! - **`ExportReport.frames_written` は現物のフレーム数と一致する**(報告=現物)

use std::path::Path;
use std::process::Command as Process;
use std::sync::Arc;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, Clip, ClipSource, Composition, Document, ItemEnvelope, Soundtrack, Track,
    TrackItem,
};
use motolii_eval::DataTracks;
use motolii_export::{export_document_video, ExportJob};
use motolii_media::Encoder;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};

/// 単色ソース動画を frames 枚で作る(source が comp より十分長い状態)。
fn make_solid_video(path: &Path, frames: usize) {
    let desc = FrameDesc::packed(320, 180, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    let frame = vec![0x40u8; desc.data_size()];
    for _ in 0..frames {
        enc.write_frame(&frame).unwrap();
    }
    enc.finish().unwrap();
}

/// 2秒の実音声(m4a)。ffmpeg CLI で合成する(`ffmpeg_or_skip` 前提)。
fn make_short_audio(path: &Path) {
    let status = Process::new("ffmpeg")
        .args(["-v", "error", "-y", "-f", "lavfi", "-i"])
        .arg("sine=frequency=440:duration=2")
        .args(["-c:a", "aac"])
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success(), "ffmpeg sine synth failed");
}

/// 現物の video フレーム数(ffprobe -count_frames)。
fn artifact_video_frames(path: &Path) -> usize {
    let out = Process::new("ffprobe")
        .args([
            "-v",
            "error",
            "-count_frames",
            "-select_streams",
            "v",
            "-show_entries",
            "stream=nb_read_frames",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "ffprobe failed");
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

/// comp 4s @12fps(=48枚)+ video clip 4s + soundtrack 2s の Document。
fn document_with_short_soundtrack(video: &Path, audio: &Path) -> Document {
    let mut doc = Document::new_current();
    doc.composition =
        Composition::try_new(16, 9, RationalTime::try_new(4, 1).unwrap(), FPS).unwrap();

    let video_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: video_id,
            name: "src".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:report-matches-src".into(),
            path_absolute: Some(video.display().to_string()),
            path_project_relative: None,
            file_name: video.file_name().map(|n| n.to_string_lossy().into_owned()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        })
        .unwrap();
    let audio_id = AssetId::from_raw(1);
    doc.assets
        .insert(Asset {
            id: audio_id,
            name: "song".into(),
            asset_type: "audio/mp4".into(),
            content_hash: "sha256:report-matches-song".into(),
            path_absolute: Some(audio.display().to_string()),
            path_project_relative: None,
            file_name: audio.file_name().map(|n| n.to_string_lossy().into_owned()),
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
            duration: None,
        })
        .unwrap();

    let layer = doc.layers.allocate("src").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(4, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::asset_video_only(video_id),
        })],
    });
    doc.soundtrack =
        Some(Soundtrack::try_new(audio_id, RationalTime::ZERO, 1.0).unwrap());
    doc.validate().unwrap();
    doc
}

#[test]
fn a_short_soundtrack_does_not_silently_truncate_the_video() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("export_report_matches_artifact");
    let video = dir.join("src.mp4");
    let audio = dir.join("song.m4a");
    let out = dir.join("out.mp4");
    make_solid_video(&video, 60); // 5s ぶん: comp(4s)より長い
    make_short_audio(&audio); // 2s: comp より短い

    let doc = document_with_short_soundtrack(&video, &audio);
    let runtime = first_party_runtime().unwrap();
    let report = export_document_video(
        &gpu,
        &ExportJob {
            doc: &doc,
            runtime: &runtime,
            output_path: &out,
            project_root: Some(&dir),
            // frame_count は渡さない: comp の長さから導く経路(CLI/GUI と同じ)を審判する。
            frame_count: None,
            qp0: false,
            data_tracks: DataTracks::new(),
        },
    )
    .unwrap();

    let expected = 48; // comp 4s × 12fps
    let actual = artifact_video_frames(&out);
    assert_eq!(
        actual, expected,
        "soundtrack(2s)が短くても video は comp の長さ(4s=48枚)ぶん出る"
    );
    assert_eq!(
        report.frames_written, actual,
        "報告(frames_written)は現物のフレーム数と一致する"
    );
}
