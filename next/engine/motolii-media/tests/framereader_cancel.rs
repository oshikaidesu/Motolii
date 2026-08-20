//! FrameReader のキャンセル + kill ハンドル分離(M3E-8 / GR-7)。

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_media::{probe, Encoder, FrameReader, MediaError};
use motolii_testkit::ffmpeg_or_skip;

const W: u32 = 64;
const H: u32 = 48;
const N_FRAMES: i64 = 300;
const FPS: Fps = match Fps::try_new(30, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-media-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_long_test_video(path: &std::path::Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for i in 0..N_FRAMES {
        let g = (i % 256) as u8;
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[g, g, g, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

#[test]
fn stale_decode_request_is_cancelled() {
    if !ffmpeg_or_skip() {
        return;
    }

    let dir = tmp_dir("cancel-stale");
    let path = dir.join("long.mp4");
    make_long_test_video(&path);
    let info = probe(&path).unwrap();

    let mut reader = FrameReader::open(&path, &info, 0).unwrap();
    let cancel = reader.cancel_token();
    let kill = reader.kill_handle();

    let (started_tx, started_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        started_tx.send(()).unwrap();
        loop {
            match reader.next_frame() {
                Err(MediaError::Cancelled) => return Ok(()),
                Ok(Some(_)) => continue,
                Ok(None) => {
                    return Err("decode finished before cancel".to_string());
                }
                Err(e) => return Err(format!("unexpected error: {e}")),
            }
        }
    });

    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    cancel.cancel();
    kill.kill().unwrap();

    worker.join().unwrap().expect("stale request should cancel");
}

#[test]
fn new_reader_works_after_cancelling_stale_request() {
    if !ffmpeg_or_skip() {
        return;
    }

    let dir = tmp_dir("cancel-fresh");
    let path = dir.join("long.mp4");
    make_long_test_video(&path);
    let info = probe(&path).unwrap();

    let mut stale = FrameReader::open(&path, &info, 0).unwrap();
    let cancel = stale.cancel_token();
    let kill = stale.kill_handle();

    let (started_tx, started_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        started_tx.send(()).unwrap();
        let _ = stale.next_frame();
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    cancel.cancel();
    kill.kill().unwrap();
    let _ = worker.join();

    let mut fresh = FrameReader::open(&path, &info, 10).unwrap();
    let frame = fresh.next_frame().unwrap().expect("frame at index 10");
    assert_eq!(frame.pts.try_to_frame_floor(info.fps).unwrap(), 10);
}
