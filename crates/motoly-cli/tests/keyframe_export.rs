use motoly_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motoly_eval::{DataTracks, Interp, Keyframe, KeyframeTrack, ParamSource, Value};
use motoly_gpu::download_rgba;
use motoly_nodes::{CanonicalSize, ParamRectOverlay, RectOverlay, ViewportTransform};
use motoly_render::{render_frame, RenderFrameRequest, SolidSource};
use motoly_testkit::{assert_rgba_close, gpu_or_skip, RgbaImageDesc};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = Fps { num: 12, den: 1 };

fn moving_overlay() -> ParamRectOverlay {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Vec2([-0.25, 0.0]),
        interp: Interp::Linear,
    });
    track.insert(Keyframe {
        t: RationalTime::from_seconds(1),
        value: Value::Vec2([0.25, 0.0]),
        interp: Interp::Linear,
    });
    ParamRectOverlay {
        center: ParamSource::Keyframes(track),
        size: ParamSource::Const(Value::Vec2([0.5, 0.5])),
        color: ParamSource::Const(Value::Color([1.0, 0.0, 0.0, 1.0])),
    }
}

fn expected_rect_frame(desc: FrameDesc, bg: [u8; 4], fg: [u8; 4], rect: RectOverlay) -> Vec<u8> {
    let tx = ViewportTransform::from_desc(&desc);
    let center = tx.point_to_px(rect.center);
    let size = tx.size_to_px(rect.size);
    let min_x = center.x - size.width * 0.5;
    let max_x = center.x + size.width * 0.5;
    let min_y = center.y - size.height * 0.5;
    let max_y = center.y + size.height * 0.5;
    let mut out = vec![0u8; desc.data_size()];
    for y in 0..desc.height {
        for x in 0..desc.width {
            let cx = x as f64 + 0.5;
            let cy = y as f64 + 0.5;
            let inside = cx >= min_x && cx < max_x && cy >= min_y && cy < max_y;
            let i = ((y * desc.width + x) * 4) as usize;
            out[i..i + 4].copy_from_slice(if inside { &fg } else { &bg });
        }
    }
    out
}

#[test]
fn keyframed_overlay_matches_golden_at_start_mid_end() {
    let Some(gpu) = gpu_or_skip() else { return };

    let overlay = moving_overlay();
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let tracks = DataTracks::new();
    let samples = [
        (RationalTime::from_frame(0, FPS), "start"),
        (RationalTime::from_frame(6, FPS), "mid"),
        (RationalTime::from_frame(12, FPS), "end"),
    ];

    for (t, label) in samples {
        let rect = overlay.eval(t, &tracks).unwrap();
        let request = RenderFrameRequest {
            desc,
            timeline_time: t,
            source: SolidSource {
                color: [0.0, 0.0, 0.0, 1.0],
                time_map: TimeMap::identity(),
                reports_source_time: true,
            },
            overlay: rect,
        };
        let rendered = render_frame(&gpu, &request, Quality::FINAL).unwrap();
        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        let expected = expected_rect_frame(desc, [0, 0, 0, 255], [255, 0, 0, 255], rect);
        assert_rgba_close(
            &format!("keyframe-overlay-{label}"),
            RgbaImageDesc {
                width: W,
                height: H,
            },
            &actual,
            &expected,
            1,
        );
    }
}

#[test]
fn project_json_accepts_keyframed_center() {
    use motoly_cli::load_project_v1_from_str;

    let json = r#"{
  "version": 1,
  "input": "in.mp4",
  "output": "out.mp4",
  "overlay": {
    "center": {
      "Keyframes": {
        "keys": [
          {"t": {"num": 0, "den": 1}, "value": {"Vec2": [-0.25, 0.0]}, "interp": "Linear"},
          {"t": {"num": 1, "den": 1}, "value": {"Vec2": [0.25, 0.0]}, "interp": "Linear"}
        ]
      }
    },
    "size": [0.5, 0.5],
    "color": [1.0, 0.0, 0.0, 1.0]
  }
}"#;
    let project = load_project_v1_from_str(json).unwrap();
    let overlay = project.overlay.into_param_overlay();
    let tracks = DataTracks::new();
    let mid = overlay
        .eval(RationalTime::from_frame(6, FPS), &tracks)
        .unwrap();
    assert!(mid.center.x.abs() < 1e-9);
    assert_eq!(mid.center.y, 0.0);
    assert_eq!(
        mid.size,
        CanonicalSize {
            width: 0.5,
            height: 0.5
        }
    );
}
