use std::path::Path;

use motolii_cli::verify_b4_project_v1;
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
fn verify_b4_passes_for_project_export_roundtrip() {
    if !ffmpeg_or_skip() {
        return;
    }
    let Some(gpu) = gpu_or_skip() else { return };

    let dir = tmp_dir("r9-b4");
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

    let report = verify_b4_project_v1(&gpu, &project_path, 8, true).unwrap();
    assert_eq!(report.frames_passed, report.frames_checked);
    assert_eq!(report.export_frames, 3);
    assert!(output.is_file());

    let info = probe(&output).unwrap();
    assert_eq!((info.width, info.height), (W, H));

    std::fs::remove_dir_all(&dir).ok();
}
