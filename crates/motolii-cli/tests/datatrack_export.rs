use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, Quality, RationalTime};
use motolii_eval::{DataTrackId, DataTracks, ParamSource, Value};
use motolii_gpu::download_rgba;
use motolii_nodes::{CanonicalSize, ParamRectOverlay};
use motolii_plugin::{
    reference::SINE_PARAM_DRIVER, ParamDriverContext, ParamDriverPlugin, ResolvedParams,
};
use motolii_render::{render_frame, RenderFrameRequest, SolidSource};
use motolii_testkit::cpu_reference::expected_rect_frame;
use motolii_testkit::{assert_rgba_close, gpu_or_skip, tol, RgbaImageDesc};

const W: u32 = 32;
const H: u32 = 24;
const FPS: Fps = match Fps::try_new(12, 1) {
    Ok(fps) => fps,
    Err(_) => panic!("invalid const fps"),
};

fn sine_x_tracks() -> DataTracks {
    let mut params = ResolvedParams::new();
    params.insert("amplitude", Value::F64(0.25));
    params.insert("frequency_hz", Value::F64(0.5));
    params.insert("offset", Value::F64(0.0));

    let track = SINE_PARAM_DRIVER
        .build_track(
            ParamDriverContext {
                start: RationalTime::ZERO,
                duration: RationalTime::from_seconds(1),
                sample_rate: FPS,
            },
            &params,
        )
        .unwrap();

    let mut tracks = DataTracks::new();
    tracks.insert(DataTrackId("sine_x".into()), track);
    tracks
}

fn sine_driven_overlay() -> ParamRectOverlay {
    ParamRectOverlay {
        center: ParamSource::Vec2Axes {
            x: Box::new(ParamSource::Data {
                track: DataTrackId("sine_x".into()),
                fallback: Value::F64(0.0),
            }),
            y: Box::new(ParamSource::Const(Value::F64(0.0))),
        },
        size: ParamSource::Const(Value::Vec2([0.5, 0.5])),
        color: ParamSource::Const(Value::Color([1.0, 0.0, 0.0, 1.0])),
    }
}

#[test]
fn datatrack_overlay_matches_golden_at_start_mid_end() {
    let Some(gpu) = gpu_or_skip() else { return };

    let tracks = sine_x_tracks();
    let overlay = sine_driven_overlay();
    let desc = FrameDesc::packed(W, H, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);
    let samples = [
        (RationalTime::try_from_frame(0, FPS).unwrap(), "start"),
        (RationalTime::try_from_frame(6, FPS).unwrap(), "mid"),
        (RationalTime::try_from_frame(12, FPS).unwrap(), "end"),
    ];

    for (t, label) in samples {
        let rect = overlay.eval(t, &tracks).unwrap();
        let request = RenderFrameRequest {
            desc,
            timeline_time: t,
            source: SolidSource {
                color: [0.0, 0.0, 0.0, 1.0],
                time_map: motolii_core::TimeMap::identity(),
                reports_source_time: true,
            },
            overlay: rect,
        };
        let rendered = render_frame(&gpu, &request, Quality::FINAL).unwrap();
        let actual = download_rgba(&gpu, &rendered.texture).unwrap();
        let expected = expected_rect_frame(
            desc,
            [0, 0, 0, 255],
            [255, 0, 0, 255],
            [rect.center.x, rect.center.y],
            [rect.size.width, rect.size.height],
        );
        assert_rgba_close(
            &format!("datatrack-overlay-{label}"),
            RgbaImageDesc {
                width: W,
                height: H,
            },
            &actual,
            &expected,
            tol::GPU_RASTER,
        );
    }
}

#[test]
fn project_json_accepts_datatrack_center() {
    use motolii_cli::{build_data_tracks, load_project_v1_from_str};

    let json = r#"{
  "version": 1,
  "input": "in.mp4",
  "output": "out.mp4",
  "param_drivers": [
    {
      "plugin": "core.param.sine",
      "track": "sine_x",
      "params": {
        "amplitude": {"F64": 0.25},
        "frequency_hz": {"F64": 0.5},
        "offset": {"F64": 0.0}
      }
    }
  ],
  "overlay": {
    "center": {
      "Vec2Axes": {
        "x": {"Data": {"track": "sine_x", "fallback": {"F64": 0.0}}},
        "y": {"Const": {"F64": 0.0}}
      }
    },
    "size": [0.5, 0.5],
    "color": [1.0, 0.0, 0.0, 1.0]
  }
}"#;
    let project = load_project_v1_from_str(json).unwrap();
    let tracks = build_data_tracks(
        &project.param_drivers,
        RationalTime::ZERO,
        RationalTime::from_seconds(1),
        FPS,
    )
    .unwrap();
    let overlay = project.overlay.into_param_overlay();
    let mid = overlay
        .eval(RationalTime::try_from_frame(6, FPS).unwrap(), &tracks)
        .unwrap();
    assert!((mid.center.x - 0.25).abs() < 1e-9);
    assert_eq!(mid.center.y, 0.0);
    assert_eq!(
        mid.size,
        CanonicalSize {
            width: 0.5,
            height: 0.5
        }
    );
}
