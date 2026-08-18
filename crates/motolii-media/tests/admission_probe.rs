//! N-MEDIA-PLACE(素材取り込み半分・media側): 実ファイル→`probe_admission_source`
//! →`AssetDraft::from_probed_source`→既存admission経路→Documentの統合審判。
//!
//! fixtureは既存慣行(ag1_stream_probe / mux)に合わせてffmpegで生成し、
//! ffmpeg不在時のskipは`motolii_testkit::ffmpeg_or_skip`を通す。

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use motolii_core::ColorSpace;
use motolii_doc::{
    resolve_asset_path, AssetDraft, AssetId, Document, DocumentWriter, SourceFingerprintDecode,
    SourceFingerprintV1,
};
use motolii_media::{probe, probe_admission_source, MediaError};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
use motolii_testkit::cpu_reference::expected_source_content_hash_of_file;

fn run_ffmpeg(args: &[&str]) {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y"])
        .args(args)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "ffmpeg failed: {args:?}");
}

fn make_video_aac(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=red:s=64x48:d=0.5:r=24",
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=0.5",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-shortest",
        path.to_str().unwrap(),
    ]);
}

fn make_audio_m4a(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=440:sample_rate=48000:duration=0.5",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
        path.to_str().unwrap(),
    ]);
}

/// 単色の静止画1枚。`-frames:v 1` なので中身は本当に1フレームしか無い。
fn make_still(path: &Path, color: &str) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        &format!("color=c={color}:s=64x48:d=0.04"),
        "-frames:v",
        "1",
        path.to_str().unwrap(),
    ]);
}

// ---------------------------------------------------------------------------
// 静止画の入口(2026-08-18 レーンA)
//
// 利用者の初回タッチで閉まっていた扉。ffprobe の実測(同日)はこうだった:
//   png  … codec_type=video / codec_name=png   / format=png_pipe  / duration 無し
//   jpg  … codec_type=video / codec_name=mjpeg / format=jpeg_pipe / duration 無し
//   webp … codec_type=video / codec_name=webp  / format=webp_pipe / duration 無し
// r_frame_rate は demuxer が置く 25/1 の作り値で、静止画に fps は無い。
// 色タグも素材の見た目とは無関係(png は gbr/pc、jpg は bt470bg/pc)で、
// motolii の decode は必ず ffmpeg の RGB→yuv420p を通る。その変換の出力は
// **BT.601 limited** であることを実測した(赤の PNG → Y=81 U=91 V=239。
// 601 limited の理論値 81/90/240 に一致、709 limited の 63/102/240 ではない)。
// ---------------------------------------------------------------------------

#[test]
fn probes_a_still_png_as_an_image_with_no_duration() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_still_png");
    let path = dir.join("still.png");
    make_still(&path, "red");

    let source = probe_admission_source(&path).unwrap();
    assert_eq!(source.asset_type, "image/png");
    assert_eq!(
        source.container.video_streams.len(),
        1,
        "静止画も video stream として現れる"
    );
    assert!(source.container.audio_streams.is_empty());
    assert_eq!(
        source.duration, None,
        "静止画に尺は無い。place は尺不明の意味(comp の残り)へ落ちる"
    );
    assert_eq!(
        source.fingerprint.content_hash(),
        expected_source_content_hash_of_file(&path)
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn probes_still_jpeg_and_webp_too() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_still_others");
    for (name, asset_type) in [
        ("still.jpg", "image/jpeg"),
        ("still.jpeg", "image/jpeg"),
        ("still.webp", "image/webp"),
    ] {
        let path = dir.join(name);
        make_still(&path, "green");
        let source = probe_admission_source(&path).unwrap();
        assert_eq!(source.asset_type, asset_type, "{name}");
        assert_eq!(source.container.video_streams.len(), 1, "{name}");
        assert_eq!(source.duration, None, "{name}");
    }
    std::fs::remove_dir_all(&dir).ok();
}

/// 静止画の1枚は `probe`(MediaInfo)からも1フレームとして見える。
/// 尺が無いので export 側の freeze は nb_frames を頼りにクランプする。
#[test]
fn a_still_image_probes_as_exactly_one_frame() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_still_frames");
    let path = dir.join("still.png");
    make_still(&path, "red");

    let info = probe(&path).unwrap();
    assert_eq!((info.width, info.height), (64, 48));
    assert_eq!(info.duration, None, "静止画に尺は無い");
    assert_eq!(info.nb_frames, Some(1), "静止画はちょうど1フレーム");
    assert_eq!(
        info.color_space,
        ColorSpace::Rec601Limited,
        "decode の RGB→yuv420p が実際に出す行列(実測)。素材の gbr/pc タグではない"
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn probes_video_container_fingerprint_and_duration() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_video");
    let path = dir.join("clip.mp4");
    make_video_aac(&path);

    let source = probe_admission_source(&path).unwrap();
    assert_eq!(source.asset_type, "video/mp4");
    assert_eq!(source.container.video_streams.len(), 1);
    assert_eq!(source.container.audio_streams.len(), 1);
    let secs = source
        .duration
        .expect("container duration")
        .as_seconds_f64();
    assert!((0.3..2.0).contains(&secs), "unexpected duration {secs}");
    assert_eq!(
        source.fingerprint.content_hash(),
        expected_source_content_hash_of_file(&path)
    );
    assert_eq!(
        source.fingerprint.size_bytes(),
        std::fs::metadata(&path).unwrap().len()
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn probes_audio_only_m4a() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_audio");
    let path = dir.join("song.m4a");
    make_audio_m4a(&path);

    let source = probe_admission_source(&path).unwrap();
    assert_eq!(source.asset_type, "audio/mp4");
    assert!(source.container.video_streams.is_empty());
    assert_eq!(source.container.audio_streams.len(), 1);
    let secs = source
        .duration
        .expect("container duration")
        .as_seconds_f64();
    assert!((0.3..2.0).contains(&secs), "unexpected duration {secs}");
    assert_eq!(
        source.fingerprint.content_hash(),
        expected_source_content_hash_of_file(&path)
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn rejects_non_media_extension_before_probing() {
    // 拡張子で先に落ちるのでffmpeg不要。
    let dir = tmp_dir("admission_probe_ext");
    let path = dir.join("note.txt");
    std::fs::write(&path, b"not media").unwrap();
    let err = probe_admission_source(&path).unwrap_err();
    assert!(matches!(err, MediaError::Probe(_)), "got {err:?}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn rejects_video_extension_without_video_stream() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_probe_mismatch");
    let path = dir.join("audio_only.mp4");
    make_audio_m4a(&path);
    let err = probe_admission_source(&path).unwrap_err();
    assert!(matches!(err, MediaError::Probe(_)), "got {err:?}");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn real_file_probe_to_document_admission_via_existing_route() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("admission_full");
    let root = dir.join("proj");
    std::fs::create_dir_all(root.join("media")).unwrap();
    let path = root.join("media").join("clip.mp4");
    make_video_aac(&path);

    let source = probe_admission_source(&path).unwrap();
    let draft = AssetDraft::from_probed_source(
        source.asset_type.clone(),
        &source.fingerprint,
        &path,
        Some(&root),
    );

    let catalog = Arc::new(motolii_doc::motolii_plugin::reference::reference_catalog().unwrap());
    let mut writer = DocumentWriter::new(Document::new_current(), catalog).unwrap();
    let admit = writer.prepare_admit_asset(draft).unwrap();
    let gesture = writer.begin_gesture();
    writer
        .apply_prepared_asset_admission(gesture, admit)
        .unwrap();

    let snapshot = writer.snapshot();
    let asset = snapshot.assets.get(AssetId::from_raw(0)).unwrap().clone();
    assert_eq!(asset.name, "clip");
    assert_eq!(asset.asset_type, "video/mp4");
    assert_eq!(asset.file_name.as_deref(), Some("clip.mp4"));
    assert_eq!(
        asset.path_project_relative.as_deref(),
        Some("media/clip.mp4")
    );
    assert_eq!(asset.content_hash, expected_source_content_hash_of_file(&path));
    assert_eq!(
        asset.size_bytes,
        Some(std::fs::metadata(&path).unwrap().len())
    );
    assert_eq!(asset.head_hash, None);
    assert_eq!(asset.tail_hash, None);

    // Documentに載った値はstrict V1としてdecodeでき、実ファイルの再hashと一致する。
    match SourceFingerprintV1::decode_persisted(&asset.content_hash, asset.size_bytes) {
        SourceFingerprintDecode::V1(decoded) => {
            let rehashed =
                SourceFingerprintV1::from_reader(std::fs::File::open(&path).unwrap()).unwrap();
            assert_eq!(decoded, rehashed);
        }
        other => panic!("expected strict V1 fingerprint, got {other:?}"),
    }

    // ガード10の解決順で実在ファイルへ辿り着く。
    let resolved = resolve_asset_path(&asset, Some(&root)).unwrap();
    assert_eq!(resolved, root.join("media/clip.mp4"));
    std::fs::remove_dir_all(&dir).ok();
}
