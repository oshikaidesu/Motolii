//! エンコード→probe→デコード→シークの往復テスト。
//! 「フレームNを要求→正しいフレームが返る」(M1-T2の完了条件)を、
//! フレーム番号を輝度に焼き込んだ自作素材で数値検証する。
//!
//! レビュー指摘対応:
//! - #2: デコードは生YUV420pで受ける(ffmpegに色変換させない)→ Y面の輝度で検証
//! - #5: 書き出しmp4にBT.709色タグが付くことをprobeで検証
//! - #4: 回転メタデータ付き素材でprobe寸法とデコードが整合することを検証
//!
//! ffmpeg/ffprobeが無い環境ではskip(CIでは必ずインストールして実行する)。

use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use motolii_media::{probe, read_frame_at, Encoder, FrameReader};

const W: u32 = 64;
const H: u32 = 48;
const N_FRAMES: i64 = 30;
const FPS: Fps = Fps { num: 30, den: 1 };
/// RGB→YUV→RGBの量子化で±数値の誤差は出るため許容幅を持つ
const TOL: i32 = 6;

fn frame_gray(index: i64) -> u8 {
    (index * 8) as u8
}

/// グレーRGB(g,g,g)のBT.709 limited輝度: Y = 16 + 219*g/255(グレーは係数に依らない)
fn expected_luma(gray: u8) -> i32 {
    (16.0 + 219.0 * gray as f64 / 255.0).round() as i32
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

/// YUV420pフレームのY面中央値
fn center_luma(frame: &motolii_core::CpuFrame) -> i32 {
    let w = frame.desc.width as usize;
    let x = w / 2;
    let y = (frame.desc.height / 2) as usize;
    frame.data[y * w + x] as i32
}

fn assert_frame_is(frame: &motolii_core::CpuFrame, index: i64) {
    assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
    let got = center_luma(frame);
    let want = expected_luma(frame_gray(index));
    assert!(
        (got - want).abs() <= TOL,
        "expected frame {index} (luma {want}), got luma {got}"
    );
}

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("motolii-media-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn encode_probe_decode_seek_roundtrip() {
    if !motolii_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let dir = tmp_dir("roundtrip");
    let path = dir.join("counter.mp4");
    make_test_video(&path);

    // 最低バージョン検証が通ること(#5)
    motolii_media::verify_tool_versions().expect("ffmpeg/ffprobe version check");

    // --- probe: 寸法・fps・長さ・色タグ ---
    let info = probe(&path).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);
    assert_eq!(info.rotation, 0);
    // 書き出し色タグ検証(#5): Encoderが付けたBT.709 limitedをprobeが読める
    assert_eq!(info.color_space, ColorSpace::Rec709Limited);
    if let Some(d) = info.duration {
        assert_eq!(d, RationalTime::from_frame(N_FRAMES, FPS));
    }

    // --- 先頭からの順次デコード: 全フレームが順番通り(生YUVで受かる) ---
    let mut reader = FrameReader::open(&path, &info, 0).unwrap();
    let mut count = 0i64;
    while let Some(frame) = reader.next_frame().unwrap() {
        assert_frame_is(&frame, count);
        assert_eq!(frame.pts, RationalTime::from_frame(count, FPS));
        assert_eq!(frame.desc.color_space, ColorSpace::Rec709Limited);
        count += 1;
    }
    assert_eq!(count, N_FRAMES);

    // --- フレーム正確シーク: 先頭・境界・ランダム点・末尾 ---
    for index in [0, 1, 14, 15, 17, 28, 29] {
        let frame = read_frame_at(&path, &info, index).unwrap();
        assert_frame_is(&frame, index);
        assert_eq!(frame.pts, RationalTime::from_frame(index, FPS));
    }

    // --- 厳密レベル検証(#7): TOL=6の緩さでレンジ取り違えを隠さない ---
    // 黒(gray=0)→ Y=16、中間(gray=128)→ Y=126 が±2で出ること
    let black = read_frame_at(&path, &info, 0).unwrap();
    assert!(
        (center_luma(&black) - 16).abs() <= 2,
        "black level: got {} want 16±2 (full/limited range mixup?)",
        center_luma(&black)
    );
    let mid = read_frame_at(&path, &info, 16).unwrap(); // gray = 16*8 = 128
    assert!(
        (center_luma(&mid) - 126).abs() <= 2,
        "mid gray level: got {} want 126±2",
        center_luma(&mid)
    );

    // --- 範囲外シークはエラー ---
    assert!(read_frame_at(&path, &info, N_FRAMES + 10).is_err());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn seek_then_sequential_read_stays_aligned() {
    if !motolii_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let dir = tmp_dir("seq");
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

#[test]
fn rotated_footage_dimensions_match_decode() {
    if !motolii_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let dir = tmp_dir("rotate");
    let src = dir.join("counter.mp4");
    make_test_video(&src);

    // 回転メタデータ(display matrix)を付けた素材を作る(スマホ縦動画の再現)
    let rotated = dir.join("rotated.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-display_rotation", "90"])
        .arg("-i")
        .arg(&src)
        .args(["-c", "copy"])
        .arg(&rotated)
        .status()
        .unwrap();
    assert!(status.success(), "ffmpeg -display_rotation failed");

    // probeは表示寸法(W/H入れ替え)を返し、デコード出力と一致すること(#4)
    let info = probe(&rotated).unwrap();
    assert_ne!(info.rotation, 0);
    assert_eq!(
        (info.width, info.height),
        (H, W),
        "rotated footage should swap dimensions"
    );

    // デコードがその寸法で正しくフレームを切り出せる(truncated frameにならない)
    let mut reader = FrameReader::open(&rotated, &info, 0).unwrap();
    let mut count = 0i64;
    while let Some(frame) = reader.next_frame().unwrap() {
        assert_eq!((frame.desc.width, frame.desc.height), (H, W));
        // グレー素材なので回転しても輝度は同じ
        assert_frame_is(&frame, count);
        count += 1;
    }
    assert_eq!(count, N_FRAMES);

    std::fs::remove_dir_all(&dir).ok();
}
