//! T9完了条件の検証(R5)。
//!
//! 1. タイムコード(フレーム番号)焼き込み素材を30fps数秒書き出し、全フレームの時刻対応を検証
//! 2. 書き出しmp4の色タグ(BT.709 limited)をprobeで検証

use std::path::Path;

use motoly_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use motoly_eval::DataTracks;
use motoly_export::{export_overlay_video, ExportOverlayRequest};
use motoly_media::{probe, Encoder, FrameReader};
use motoly_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};
use motoly_testkit::{gpu_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 48;
const FPS: Fps = Fps { num: 30, den: 1 };
/// 3秒 @ 30fps
const N_FRAMES: i64 = 90;
/// H.264往復の量子化誤差
const LUMA_TOL: i32 = 8;

/// フレーム番号を中央輝度に焼き込む(0..=255で一意)。
fn frame_gray(index: i64) -> u8 {
    index as u8
}

fn expected_luma(gray: u8) -> i32 {
    (16.0 + 219.0 * gray as f64 / 255.0).round() as i32
}

fn center_luma(frame: &motoly_core::CpuFrame) -> i32 {
    let w = frame.desc.width as usize;
    let x = w / 2;
    let y = (frame.desc.height / 2) as usize;
    frame.data[y * w + x] as i32
}

fn assert_frame_matches_index(frame: &motoly_core::CpuFrame, index: i64, fps: Fps) {
    let got = center_luma(frame);
    let want = expected_luma(frame_gray(index));
    assert!(
        (got - want).abs() <= LUMA_TOL,
        "frame {index}: luma {got} want {want}±{LUMA_TOL}"
    );
    assert_eq!(
        frame.pts,
        RationalTime::from_frame(index, fps),
        "frame {index}: pts mismatch"
    );
}

fn make_timecode_video(path: &Path) {
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

/// 合成を変えず背景をそのまま通す透明オーバーレイ。
fn passthrough_overlay() -> ParamRectOverlay {
    ParamRectOverlay::constant(RectOverlay {
        center: CanonicalPoint::CENTER,
        size: CanonicalSize {
            width: 0.5,
            height: 0.5,
        },
        color: [1.0, 0.0, 0.0, 0.0],
    })
}

#[test]
fn export_preserves_timeline_for_all_frames_at_30fps() {
    if !motoly_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("timeline");
    let input = dir.join("timecode_src.mp4");
    let output = dir.join("timecode_out.mp4");
    make_timecode_video(&input);

    let report = export_overlay_video(
        &gpu,
        &ExportOverlayRequest {
            input_path: &input,
            output_path: &output,
            start_frame: 0,
            frame_count: Some(N_FRAMES as usize),
            overlay: passthrough_overlay(),
            data_tracks: DataTracks::new(),
            qp0: true,
        },
    )
    .unwrap();

    assert_eq!(report.frames_written, N_FRAMES as usize);
    assert_eq!(report.fps, FPS);
    assert_eq!((report.desc.width, report.desc.height), (W, H));

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);
    if let Some(nb) = info.nb_frames {
        assert_eq!(nb, N_FRAMES);
    }
    if let Some(d) = info.duration {
        assert_eq!(d, RationalTime::from_frame(N_FRAMES, FPS));
    }

    let mut reader = FrameReader::open(&output, &info, 0).unwrap();
    let mut count = 0i64;
    while let Some(frame) = reader.next_frame().unwrap() {
        assert_frame_matches_index(&frame, count, FPS);
        assert_eq!(frame.desc.color_space, ColorSpace::Rec709Limited);
        count += 1;
    }
    assert_eq!(
        count, N_FRAMES,
        "decoded frame count must match exported timeline length"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn exported_mp4_probe_reports_bt709_limited() {
    if !motoly_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("colortags");
    let input = dir.join("input.mp4");
    let output = dir.join("output.mp4");
    make_timecode_video(&input);

    export_overlay_video(
        &gpu,
        &ExportOverlayRequest {
            input_path: &input,
            output_path: &output,
            start_frame: 0,
            frame_count: Some(N_FRAMES as usize),
            overlay: passthrough_overlay(),
            data_tracks: DataTracks::new(),
            qp0: true,
        },
    )
    .unwrap();

    let info = probe(&output).unwrap();
    assert_eq!(
        info.color_space,
        ColorSpace::Rec709Limited,
        "export output must carry BT.709 limited color tags (review #5)"
    );

    // probeの高レベル型に加え、ffprobe生タグも確認する。
    let raw = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=color_space,color_range",
            "-of",
            "default=nw=1",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(raw.status.success(), "ffprobe color tag query failed");
    let text = String::from_utf8_lossy(&raw.stdout);
    assert!(
        text.contains("color_space=bt709"),
        "expected bt709 matrix tag, got:\n{text}"
    );
    assert!(
        text.contains("color_range=tv"),
        "expected tv (limited) range tag, got:\n{text}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
