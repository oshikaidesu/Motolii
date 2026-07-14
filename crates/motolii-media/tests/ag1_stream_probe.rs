//! AG-1: 全stream probeとkind/ordinal選択のfixture審判。

use std::path::{Path, PathBuf};
use std::process::Command;

use motolii_media::{
    probe, probe_container, require_supported_audio, select_audio_stream, select_video_stream,
    MediaError, MediaStreamKind,
};
use motolii_testkit::{ffmpeg_or_skip, tmp_dir};

fn run_ffmpeg(args: &[&str]) {
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y"])
        .args(args)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "ffmpeg failed: {args:?}");
}

fn make_video_only(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "color=c=black:s=64x48:d=0.5:r=24",
        "-an",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        path.to_str().unwrap(),
    ]);
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

fn make_audio_only_wav(path: &Path) {
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=880:sample_rate=44100:duration=0.25",
        "-c:a",
        "pcm_s16le",
        path.to_str().unwrap(),
    ]);
}

fn make_dual_audio_mp4(path: &Path) {
    // 2本のaudio stream(言語タグ付き)を持つcontainer。
    let dir = path.parent().unwrap();
    let a0 = dir.join("a0.wav");
    let a1 = dir.join("a1.wav");
    make_audio_only_wav(&a0);
    run_ffmpeg(&[
        "-f",
        "lavfi",
        "-i",
        "sine=frequency=220:sample_rate=48000:duration=0.25",
        "-c:a",
        "pcm_s16le",
        a1.to_str().unwrap(),
    ]);
    let v = dir.join("v.mp4");
    make_video_only(&v);
    run_ffmpeg(&[
        "-i",
        v.to_str().unwrap(),
        "-i",
        a0.to_str().unwrap(),
        "-i",
        a1.to_str().unwrap(),
        "-map",
        "0:v:0",
        "-map",
        "1:a:0",
        "-map",
        "2:a:0",
        "-c:v",
        "copy",
        "-c:a",
        "aac",
        "-metadata:s:a:0",
        "language=eng",
        "-metadata:s:a:1",
        "language=jpn",
        path.to_str().unwrap(),
    ]);
}

#[test]
fn probes_video_only_mp4() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("ag1_video_only");
    let path = dir.join("video_only.mp4");
    make_video_only(&path);
    let info = probe_container(&path).unwrap();
    assert_eq!(info.video_streams.len(), 1);
    assert!(info.audio_streams.is_empty());
    assert_eq!(select_video_stream(&info, 0).unwrap().ordinal, 0);
    assert!(matches!(
        select_audio_stream(&info, 0),
        Err(MediaError::StreamNotFound {
            kind: MediaStreamKind::Audio,
            ordinal: 0
        })
    ));
    // 旧probe互換
    let legacy = probe(&path).unwrap();
    assert_eq!(legacy.width, info.video_streams[0].width);
}

#[test]
fn probes_video_plus_aac() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("ag1_av");
    let path = dir.join("av.mp4");
    make_video_aac(&path);
    let info = probe_container(&path).unwrap();
    assert_eq!(info.video_streams.len(), 1);
    assert_eq!(info.audio_streams.len(), 1);
    let audio = select_audio_stream(&info, 0).unwrap();
    assert_eq!(audio.codec_name, "aac");
    require_supported_audio(audio).unwrap();
}

#[test]
fn probes_audio_only_wav() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("ag1_wav");
    let path = dir.join("audio.wav");
    make_audio_only_wav(&path);
    let info = probe_container(&path).unwrap();
    assert!(info.video_streams.is_empty());
    assert_eq!(info.audio_streams.len(), 1);
    assert_eq!(info.audio_streams[0].sample_rate, Some(44_100));
    assert!(matches!(
        select_video_stream(&info, 0),
        Err(MediaError::StreamNotFound {
            kind: MediaStreamKind::Video,
            ordinal: 0
        })
    ));
    require_supported_audio(select_audio_stream(&info, 0).unwrap()).unwrap();
}

#[test]
fn probes_dual_audio_language_streams_stable_ordinals() {
    if !ffmpeg_or_skip() {
        return;
    }
    let dir = tmp_dir("ag1_dual_audio");
    let path: PathBuf = dir.join("dual_audio.mp4");
    make_dual_audio_mp4(&path);
    let info = probe_container(&path).unwrap();
    assert_eq!(info.video_streams.len(), 1);
    assert_eq!(info.audio_streams.len(), 2);
    let a0 = select_audio_stream(&info, 0).unwrap();
    let a1 = select_audio_stream(&info, 1).unwrap();
    assert_eq!(a0.ordinal, 0);
    assert_eq!(a1.ordinal, 1);
    // language tagは再probe可能なcache。欠落してもordinal選択は安定。
    assert!(a0.language.as_deref() == Some("eng") || a0.language.is_none());
    assert!(a1.language.as_deref() == Some("jpn") || a1.language.is_none());
    assert!(matches!(
        select_audio_stream(&info, 2),
        Err(MediaError::StreamNotFound {
            kind: MediaStreamKind::Audio,
            ordinal: 2
        })
    ));
}
