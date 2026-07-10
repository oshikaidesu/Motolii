//! M1 出口デモ(ヒーロー)のE2Eゴールデン。
//!
//! 「実写(生成)背景 + Bezierイージングで右へ流れる四角シェイプ」の2レイヤー合成を
//! `export-project` の経路そのものでmp4化し、**出力mp4をデコードして中身を検証**する。
//! これがM1の出口デモ(README/Redditのヒーロー)の自動判定であり、
//! パイプライン(デコード背景 → 正準座標オーバーレイ → 合成 → エンコード → デコード)が
//! E2Eで成立していることを先頭/中間/末尾の3フレームで示す。
//!
//! ffmpeg/ffprobeが無い、またはGPUアダプタが無い環境ではskip(CIはlavapipe+ffmpegで実行)。

use std::path::Path;

use motoly_cli::{export_project, load_project_v1_from_str};
use motoly_core::{ColorSpace, Fps, FrameDesc, PixelFormat, RationalTime};
use motoly_eval::DataTracks;
use motoly_gpu::yuv_to_rgba_reference;
use motoly_media::{probe, read_frame_at, Encoder};
use motoly_nodes::ViewportTransform;
use motoly_testkit::{gpu_or_skip, tmp_dir};

const W: u32 = 64;
const H: u32 = 48;
const FPS: Fps = Fps { num: 12, den: 1 };
const N_FRAMES: usize = 13; // frame 0..12 = t 0..1.0s
const BG_GRAY: u8 = 120;

/// 出口デモのオーバーレイ(center.xをBezierイージングで -0.3 → +0.3、可視な赤矩形)。
/// keyのinterpはそのkeyから次のkeyまでの区間に効く → ease-in-outはkey0に置く。
fn overlay_json() -> &'static str {
    r#"{
    "center": { "Keyframes": { "keys": [
      {"t": {"num": 0, "den": 1}, "value": {"Vec2": [-0.3, 0.0]},
       "interp": {"Bezier": {"x1": 0.42, "y1": 0.0, "x2": 0.58, "y2": 1.0}}},
      {"t": {"num": 1, "den": 1}, "value": {"Vec2": [0.3, 0.0]}, "interp": "Linear"}
    ] } },
    "size": [0.3, 0.4],
    "color": [1.0, 0.15, 0.1, 1.0]
  }"#
}

/// フラットなグレーの背景動画(実写素材の代用)。矩形の外側=この色が見えるべき。
fn make_bg_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for _ in 0..N_FRAMES {
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[BG_GRAY, BG_GRAY, BG_GRAY, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

fn pixel(rgba: &[u8], w: u32, x: i64, y: i64) -> [u8; 4] {
    let xi = x.clamp(0, w as i64 - 1) as usize;
    let yi = y.clamp(0, H as i64 - 1) as usize;
    let i = (yi * w as usize + xi) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

#[test]
fn exit_demo_video_bg_plus_eased_rect_matches_golden() {
    if !motoly_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("exit-demo");
    let input = dir.join("input.mp4");
    let output = dir.join("exit-demo.mp4");
    let project_path = dir.join("project.json");

    make_bg_video(&input);

    let full_json = format!(
        r#"{{
  "version": 1,
  "input": "{}",
  "output": "{}",
  "frame_count": {N_FRAMES},
  "qp0": true,
  "overlay": {}
}}"#,
        input.display(),
        output.display(),
        overlay_json()
    );
    std::fs::write(&project_path, &full_json).unwrap();

    // export-project 経路そのものを通す
    let report = export_project(&gpu, &project_path).unwrap();
    assert_eq!(report.frames_written, N_FRAMES);
    assert_eq!((report.desc.width, report.desc.height), (W, H));

    // 期待矩形位置は「出荷サンプルと同じJSON」を評価器に通して得る(ドッグフード)
    let project = load_project_v1_from_str(&full_json).unwrap();
    let overlay = project.overlay.into_param_overlay();
    let tracks = DataTracks::new();
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let tx = ViewportTransform::from_desc(&desc);

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));

    let samples = [(0i64, "start"), (6, "mid"), (12, "end")];
    let mut centers_x = Vec::new();

    for (idx, label) in samples {
        let t = RationalTime::from_frame(idx, FPS);
        let rect = overlay.eval(t, &tracks).unwrap();
        let c = tx.point_to_px(rect.center);
        centers_x.push(c.x);

        // 出力mp4をデコード(生YUV420p)→ 参照実装でRGBAへ
        let frame = read_frame_at(&output, &info, idx).unwrap();
        assert_eq!(frame.desc.format, PixelFormat::Yuv420p);
        let rgba = yuv_to_rgba_reference(&frame);

        // (1) 矩形中心 ≈ 赤(H.264 4:2:0 + YUV往復の許容を持たせる)
        let center = pixel(&rgba, W, c.x.round() as i64, c.y.round() as i64);
        assert!(
            center[0] > 150 && center[1] < 100 && center[2] < 100,
            "{label}: rect center expected red-ish, got {center:?} at ({:.1},{:.1})",
            c.x,
            c.y
        );

        // (2) 背景コーナー ≈ グレー(矩形は中央帯なので四隅は背景動画が見える)
        let corner = pixel(&rgba, W, 2, 2);
        let gray_ok = corner[..3]
            .iter()
            .all(|&v| (v as i32 - BG_GRAY as i32).abs() < 45);
        assert!(
            gray_ok,
            "{label}: bg corner expected gray~{BG_GRAY}, got {corner:?}"
        );
    }

    // (3) 右へ流れる: 先頭 < 中間 < 末尾(モーションの向き)
    assert!(
        centers_x[0] < centers_x[1] && centers_x[1] < centers_x[2],
        "rect should move right: {centers_x:?}"
    );

    std::fs::remove_dir_all(&dir).ok();
}
