//! N-MEDIA-PLACE(素材取り込み半分・media側): 実ファイル→`probe_admission_source`
//! →`AssetDraft::from_probed_source`→既存admission経路→Documentの統合審判。
//!
//! fixtureは既存慣行(ag1_stream_probe / mux)に合わせてffmpegで生成し、
//! ffmpeg不在時のskipは`motolii_testkit::ffmpeg_or_skip`を通す。

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use motolii_doc::{
    resolve_asset_path, AssetDraft, AssetId, Document, DocumentWriter, SourceFingerprintDecode,
    SourceFingerprintV1,
};
use motolii_media::{probe_admission_source, MediaError};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};
use sha2::{Digest, Sha256};

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

/// 独立oracle: 実装と別経路(sha2直叩き)で正準content_hash文字列を組む。
fn expected_content_hash(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let digest = Sha256::digest(&bytes);
    let mut hash = String::from("motolii-source-v1:sha256:");
    for byte in digest {
        hash.push_str(&format!("{byte:02x}"));
    }
    hash
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
        expected_content_hash(&path)
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
        expected_content_hash(&path)
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
    assert_eq!(asset.content_hash, expected_content_hash(&path));
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
