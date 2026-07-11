use std::path::Path;

use motolii_cli::export_project;
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_media::{probe, Encoder};
use motolii_testkit::{ffmpeg_or_skip, gpu_or_skip, tmp_dir};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = Fps { num: 12, den: 1 };
const N_FRAMES: usize = 6;

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
fn exports_project_overlay_to_mp4() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("project");
    let input = dir.join("input.mp4");
    let output = dir.join("output.mp4");
    let project_path = dir.join("project_v1.json");

    make_test_video(&input);

    let input_display = input.display().to_string();
    let output_display = output.display().to_string();

    let json = format!(
        r#"{{
  "version": 1,
  "input": "{input_display}",
  "output": "{output_display}",
  "start_frame": 1,
  "frame_count": 3,
  "qp0": true,
  "overlay": {{
    "center": [0.0, 0.0],
    "size": [0.5, 0.5],
    "color": [1.0, 0.0, 0.0, 0.0]
  }}
}}"#
    );

    std::fs::write(&project_path, json).unwrap();

    let report = export_project(&gpu, &project_path).unwrap();
    assert_eq!(report.frames_written, 3);
    assert_eq!((report.desc.width, report.desc.height), (W, H));
    assert_eq!(report.fps, FPS);

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));
    assert_eq!(info.fps, FPS);
    assert_eq!(info.color_space, ColorSpace::Rec709Limited);

    std::fs::remove_dir_all(&dir).ok();
}
