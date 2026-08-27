//! **engine の口 #1/#4**(`docs/reviews/2026-08-28-current-position.md` 「★ 次の一手」):
//! `Engine::media_frames`/`Engine::media_duration` が実ファイルに対して嘘を返さないこと。
//!
//! ここで縛るのは「窓を叩いても見えない嘘」1点だけ(裁定274 の対象そのもの):
//! **audio-only ファイルを `media_frames`/`media_duration` に渡した時、黙って
//! ズレた数を返さないこと**——`motolii_media::probe`(video専用)は先頭 video stream を
//! 要求するので audio-only で必ず `Err` になる既知バグがあった。`Engine` 側は
//! `probe_container` を使うので、これが再発していないことをここで固定する。

use std::process::Command;

use motolii_engine::Engine;

fn make_sine_audio(path: &std::path::Path, seconds: f64) {
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
    assert!(status.success(), "audio fixture failed");
}

fn make_tiny_video(path: &std::path::Path, frames: u32, fps: u32) {
    let status = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=black:s=16x16:r={fps}:d={}", frames as f64 / fps as f64),
            "-frames:v",
            &frames.to_string(),
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(path)
        .status()
        .expect("spawn ffmpeg");
    assert!(status.success(), "video fixture failed");
}

/// **既知バグ(裁定274)の再発防止**: audio-only ファイルは `media_frames` では
/// 「フレーム数という概念が無い」ので `None`(嘘の0や嘘のフレーム数を返さない)が、
/// `media_duration` では実際の尺が返る——`probe()`(video専用)が失敗しても
/// `probe_container` 経由でここまで通ることを実測で固定する。
#[test]
fn audio_only_file_has_no_frame_count_but_has_a_real_duration() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("engine-media-frames-audio-only");
    let audio = dir.join("voice.m4a");
    make_sine_audio(&audio, 2.0);

    let mut engine = Engine::new().expect("headless engine");
    let path = audio.to_str().unwrap();

    assert_eq!(
        engine.media_frames(path),
        None,
        "audio-only has no video stream, so no frame count exists — this must not be a lie"
    );
    let duration = engine
        .media_duration(path)
        .expect("audio-only duration must be readable (this is exactly the bug decision 274 names)");
    // 2秒指定でエンコードした素材。コンテナのオーバーヘッドで厳密に2.000ではない
    // ことがあるので、狭い許容(±0.2秒)で実測値を確かめる。
    let seconds = duration.as_seconds_f64();
    assert!(
        (seconds - 2.0).abs() < 0.2,
        "expected ~2.0s, got {seconds}s"
    );
}

/// video ファイルは今まで通り `nb_frames` が `media_frames` に届く
/// (`crate::texture` の `LayerSource::Media` 分岐が同じ値を使っている、既存の使い方を
/// 壊していないことの確認)。
#[test]
fn video_file_reports_its_native_frame_count() {
    if !motolii_testkit::ffmpeg_or_skip() {
        return;
    }
    let dir = motolii_testkit::tmp_dir("engine-media-frames-video");
    let video = dir.join("clip.mp4");
    make_tiny_video(&video, 10, 5);

    let mut engine = Engine::new().expect("headless engine");
    let path = video.to_str().unwrap();

    assert_eq!(engine.media_frames(path), Some(10));
    let duration = engine
        .media_duration(path)
        .expect("video files carry a container-level duration too");
    assert!(
        (duration.as_seconds_f64() - 2.0).abs() < 0.2,
        "10 frames @5fps = 2.0s, got {}s",
        duration.as_seconds_f64()
    );
}

/// probe が2回とも失敗する(存在しないパス)場合は両方 `None`——沈黙して「壊れて
/// いない体」の数字を返さない。
#[test]
fn missing_file_yields_none_for_both() {
    let mut engine = Engine::new().expect("headless engine");
    assert_eq!(engine.media_frames("/nonexistent/does-not-exist.mov"), None);
    assert_eq!(
        engine.media_duration("/nonexistent/does-not-exist.mov"),
        None
    );
}
