//! エンコード→probe→デコード→シークの往復テスト。
//! 「フレームNを要求→正しいフレームが返る」(M1-T2の完了条件)を、
//! フレーム番号を輝度に焼き込んだ自作素材で数値検証する。
//!
//! ffmpeg/ffprobeが無い環境ではskip(CIでは必ずインストールして実行する)。

use oc_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use oc_media::{probe, read_frame_at, Encoder, FrameReader};

const W: u32 = 64;
const H: u32 = 64;
const N_FRAMES: i64 = 30;
const FPS: Fps = Fps { num: 30, den: 1 };
/// qp0でも4:4:4のRGB→YUV変換で±数値の誤差は出るため許容幅を持つ
const TOL: i32 = 6;

fn frame_gray(index: i64) -> u8 {
    (index * 8) as u8
}

fn make_test_video(path: &std::path::Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for i in 0..N_FRAMES {
        let g = frame_gray(i);
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[g, g, g, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

fn center_gray(frame: &oc_core::CpuFrame) -> i32 {
    let x = (W / 2) as usize;
    let y = (H / 2) as usize;
    let off = (y * frame.desc.stride as usize) + x * 4;
    frame.data[off] as i32
}

fn assert_frame_is(frame: &oc_core::CpuFrame, index: i64) {
    let got = center_gray(frame);
    let want = frame_gray(index) as i32;
    assert!(
        (got - want).abs() <= TOL,
        "expected frame {index} (gray {want}), got gray {got}"
    );
}

#[test]
fn encode_probe_decode_seek_roundtrip() {
    if !oc_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let dir = std::env::temp_dir().join(format!("oc-media-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("counter.mp4");
    make_test_video(&path);

    // --- probe ---
    let info = probe(&path).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);
    if let Some(d) = info.duration {
        assert_eq!(d, RationalTime::from_seconds(1));
    }

    // --- 先頭からの順次デコード: 全フレームが順番通り ---
    let mut reader = FrameReader::open(&path, &info, 0).unwrap();
    let mut count = 0i64;
    while let Some(frame) = reader.next_frame().unwrap() {
        assert_frame_is(&frame, count);
        assert_eq!(frame.pts, RationalTime::from_frame(count, FPS));
        count += 1;
    }
    assert_eq!(count, N_FRAMES);

    // --- フレーム正確シーク: 先頭・境界・ランダム点・末尾 ---
    for index in [0, 1, 14, 15, 17, 28, 29] {
        let frame = read_frame_at(&path, &info, index).unwrap();
        assert_frame_is(&frame, index);
        assert_eq!(frame.pts, RationalTime::from_frame(index, FPS));
    }

    // --- 範囲外シークはエラー ---
    assert!(read_frame_at(&path, &info, N_FRAMES + 10).is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn seek_then_sequential_read_stays_aligned() {
    if !oc_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let dir = std::env::temp_dir().join(format!("oc-media-test-seq-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("counter.mp4");
    make_test_video(&path);
    let info = probe(&path).unwrap();

    // フレーム10から読み始めて以降が連番であること
    let mut reader = FrameReader::open(&path, &info, 10).unwrap();
    for index in 10..N_FRAMES {
        let frame = reader.next_frame().unwrap().expect("frame expected");
        assert_frame_is(&frame, index);
    }
    assert!(reader.next_frame().unwrap().is_none());

    std::fs::remove_dir_all(&dir).ok();
}
