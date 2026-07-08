use std::path::Path;

use oc_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use oc_eval::DataTracks;
use oc_export::{export_overlay_video, ExportOverlayRequest};
use oc_gpu::GpuCtx;
use oc_media::{probe, Encoder};
use oc_nodes::{CanonicalPoint, CanonicalSize, ParamRectOverlay, RectOverlay};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = Fps { num: 12, den: 1 };
const N_FRAMES: usize = 6;

fn gpu_or_skip() -> Option<GpuCtx> {
    match GpuCtx::new_headless() {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("SKIP: no GPU adapter: {e}");
            None
        }
    }
}

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("oc-export-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_test_video(path: &Path) {
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
    let mut enc = Encoder::open(path, &desc, FPS, true).unwrap();
    for i in 0..N_FRAMES {
        let g = (i * 24) as u8;
        let mut data = vec![0u8; desc.data_size()];
        for px in data.chunks_exact_mut(4) {
            px.copy_from_slice(&[g, g, g, 255]);
        }
        enc.write_frame(&data).unwrap();
    }
    enc.finish().unwrap();
}

#[test]
fn exports_video_overlay_to_mp4() {
    if !oc_media::tools_available() {
        eprintln!("SKIP: ffmpeg/ffprobe not found on PATH");
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };
    let dir = tmp_dir("overlay");
    let input = dir.join("input.mp4");
    let output = dir.join("output.mp4");
    make_test_video(&input);

    let report = export_overlay_video(
        &gpu,
        &ExportOverlayRequest {
            input_path: &input,
            output_path: &output,
            start_frame: 1,
            frame_count: Some(3),
            overlay: ParamRectOverlay::constant(RectOverlay {
                center: CanonicalPoint::CENTER,
                size: CanonicalSize {
                    width: 0.5,
                    height: 0.5,
                },
                color: [1.0, 0.0, 0.0, 0.0],
            }),
            data_tracks: DataTracks::new(),
            qp0: true,
        },
    )
    .unwrap();

    assert_eq!(report.frames_written, 3);
    assert_eq!((report.desc.width, report.desc.height), (W, H));
    assert_eq!(report.fps, FPS);

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);
    assert_eq!(info.color_space, ColorSpace::Rec709Limited);

    std::fs::remove_dir_all(&dir).ok();
}
