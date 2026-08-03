//! M4-P05-C1 V1: 通常fileのtemp、FFmpeg、integrity、partial publish境界を確認する。

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_media::{probe, Encoder};
use sha2::{Digest, Sha256};
use tempfile::{Builder, TempDir};

fn digest(path: &Path) -> [u8; 32] {
    let bytes = fs::read(path).expect("artifact should be readable");
    Sha256::digest(bytes).into()
}

fn final_path(dir: &TempDir) -> PathBuf {
    dir.path().join("recipe-v1.mp4")
}

#[test]
fn same_directory_temp_publish_keeps_old_final_until_atomic_rename() {
    let dir = tempfile::tempdir().expect("temp dir");
    let final_path = dir.path().join("artifact.bin");
    fs::write(&final_path, b"old-complete").expect("old final");

    let mut temp = Builder::new()
        .prefix("recipe-")
        .suffix(".tmp")
        .tempfile_in(dir.path())
        .expect("same-directory temp");
    assert_eq!(temp.path().parent(), final_path.parent());
    temp.write_all(b"new-complete").expect("temp write");
    temp.as_file_mut().sync_all().expect("temp sync");
    assert_eq!(fs::read(&final_path).unwrap(), b"old-complete");

    let temp_path = temp.into_temp_path();
    fs::rename(&temp_path, &final_path).expect("same-filesystem rename");
    assert_eq!(fs::read(&final_path).unwrap(), b"new-complete");
    assert!(
        !temp_path.exists(),
        "temp path must not remain after publication"
    );
}

#[test]
fn ffmpeg_can_write_a_same_directory_temp_artifact_before_publish() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = tempfile::tempdir().expect("temp dir");
    let temp_path = Builder::new()
        .prefix("ffmpeg-")
        .suffix(".mp4")
        .tempfile_in(dir.path())
        .expect("ffmpeg temp")
        .into_temp_path();
    let desc = FrameDesc::packed(2, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut encoder = Encoder::open(&temp_path, &desc, Fps::try_new(1, 1).unwrap(), true)
        .expect("ffmpeg should open a normal temp path");
    encoder
        .write_frame(&[
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ])
        .expect("frame write");
    encoder.finish().expect("ffmpeg finish");
    let info = probe(&temp_path).expect("finished temp artifact should probe");
    assert_eq!(info.width, 2);
    assert_eq!(info.height, 2);
    assert!(fs::metadata(&temp_path).unwrap().len() > 0);

    let published = final_path(&dir);
    fs::rename(&temp_path, &published).expect("publish finished artifact");
    assert_eq!(probe(&published).unwrap().width, 2);
}

#[test]
fn content_digest_is_path_independent_and_detects_bit_flip() {
    let dir = tempfile::tempdir().expect("temp dir");
    let first = dir.path().join("first.bin");
    let second = dir.path().join("nested-second.bin");
    fs::create_dir_all(second.parent().unwrap()).expect("nested dir");
    fs::write(&first, b"recipe-output").expect("first artifact");
    fs::copy(&first, &second).expect("same bytes");
    assert_eq!(digest(&first), digest(&second));

    let mut corrupted = fs::read(&second).expect("corruptible copy");
    corrupted[0] ^= 0x01;
    fs::write(&second, corrupted).expect("bit flip");
    assert_ne!(digest(&first), digest(&second));
}

#[test]
fn incomplete_or_stale_temp_never_appears_as_final_artifact() {
    let dir = tempfile::tempdir().expect("temp dir");
    let final_path = dir.path().join("artifact.bin");
    let stale = dir.path().join("artifact.bin.tmp");
    fs::write(&stale, b"partial").expect("stale temp");

    assert!(!final_path.exists());
    let visible_finals = fs::read_dir(dir.path())
        .expect("scan workspace")
        .filter_map(Result::ok)
        .filter(|entry| entry.path() == final_path)
        .count();
    assert_eq!(visible_finals, 0);
    fs::remove_file(stale).expect("cleanup stale temp");
}
