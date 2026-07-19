//! Transport → render_graph ヘッドレス統合(D5骨格)。

use std::sync::Arc;

use motolii_core::{
    CanonicalPoint as CoreCanonicalPoint, ColorSpace, CompCamera, Fps, FrameDesc, PixelFormat,
    Quality, TimeMap,
};
use motolii_nodes::{CanonicalPoint, CanonicalSize, RectOverlay};
use motolii_render::{render_frame, RenderFrameRequest, SolidSource};
use motolii_testkit::gpu_or_skip;
use motolii_transport::Transport;

#[test]
fn transport_frame_plan_drives_render_with_quality_and_time() {
    let Some(gpu) = gpu_or_skip() else {
        return;
    };

    let counters = Arc::new(motolii_audio::PlaybackCounters::default());
    let wait = Arc::new(motolii_audio::DeviceWaitLatency::default());
    counters.advance_supplied_for_simulation(48_000);
    wait.set_wait_frames(480);

    let mut transport = Transport::new(
        counters,
        wait,
        Fps::try_new(30, 1).unwrap(),
        48_000,
        Quality::DRAFT,
        motolii_gpu::drs_available(&gpu.device),
    )
    .unwrap();

    let plan = transport.next_frame_plan().unwrap();
    let desc = FrameDesc::packed(64, 36, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, true);

    let request = RenderFrameRequest {
        desc,
        timeline_time: plan.timeline_time,
        source: SolidSource {
            color: [0.0, 1.0, 0.0, 1.0],
            time_map: TimeMap::identity(),
            reports_source_time: true,
        },
        overlay: RectOverlay {
            center: CanonicalPoint::CENTER,
            size: CanonicalSize {
                width: 0.5,
                height: 0.5,
            },
            color: [1.0, 0.0, 0.0, 0.5],
        },
        camera: CompCamera::try_new(
            CoreCanonicalPoint::CENTER,
            0.0,
            1.0,
            i64::from(desc.width),
            i64::from(desc.height),
        )
        .unwrap(),
    };

    let rendered = render_frame(&gpu, &request, plan.quality).expect("render_frame");
    assert_eq!(
        rendered.desc.width,
        desc.width / plan.quality.resolution_scale
    );
    assert_eq!(
        rendered.desc.height,
        desc.height / plan.quality.resolution_scale
    );
    assert_eq!(
        rendered.source_time, plan.timeline_time,
        "render uses transport timeline_time"
    );
}
