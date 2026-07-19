//! UI/event-loop 外で Document を one-shot レンダし display pool へ GPU copy する。

use std::collections::BTreeMap;
use std::thread::{self, JoinHandle};

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, Composition, DocParam, Document, EvaluationTime,
    ItemEnvelope, Track, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession, RenderedFrame};

use crate::display_pool::DisplayPool;

const VIEWPORT_WIDTH: u32 = 320;
const VIEWPORT_HEIGHT: u32 = 180;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StaticFrameError {
    #[error("document validation failed: {0}")]
    Document(#[from] motolii_doc::DocumentError),
    #[error("document graph failed: {0}")]
    Graph(#[from] motolii_doc::GraphError),
    #[error("render failed: {0}")]
    Render(#[from] motolii_render::RenderError),
    #[error("first-party runtime unavailable")]
    FirstParty(#[from] motolii_plugins_firstparty::FirstPartyError),
    #[error("composition setup failed: {0}")]
    Composition(#[from] motolii_doc::CompositionError),
    #[error("failed to spawn setup thread: {0}")]
    SetupThreadSpawn(#[from] std::io::Error),
    #[error("setup thread panicked")]
    SetupThreadPanic,
}

/// shell から呼ぶ唯一の本番入口: off-event-loop spawn/join をここで完結させる。
pub(crate) fn spawn_join_display_pool(gpu: GpuCtx) -> Result<DisplayPool, StaticFrameError> {
    let handle: JoinHandle<Result<DisplayPool, StaticFrameError>> = thread::Builder::new()
        .name("motolii-u1a1-setup".into())
        .spawn(move || prepare_display_pool(&gpu))?;
    handle
        .join()
        .map_err(|_| StaticFrameError::SetupThreadPanic)?
}

fn frame_desc() -> FrameDesc {
    FrameDesc::packed(
        VIEWPORT_WIDTH,
        VIEWPORT_HEIGHT,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )
}

fn fixed_document() -> Result<Document, StaticFrameError> {
    let mut doc = Document::new_current();
    doc.composition = Composition::try_new(
        i64::from(VIEWPORT_WIDTH),
        i64::from(VIEWPORT_HEIGHT),
        RationalTime::try_new(10, 1).unwrap(),
        doc.composition.fps,
    )?;
    let layer = doc.layers.allocate("preview-rect").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([0.55, 0.55])),
                    ("color".into(), DocParam::const_color([0.2, 0.6, 1.0, 1.0])),
                ]),
                extra: Default::default(),
            },
        })],
    });
    doc.validate()?;
    Ok(doc)
}

fn render_static_frame(gpu: &GpuCtx, doc: &Document) -> Result<RenderedFrame, StaticFrameError> {
    debug_assert!(
        !thread::current()
            .name()
            .is_some_and(|name| name.contains("winit")),
        "static document render must not run on the UI/event-loop thread"
    );
    let frame_desc = frame_desc();
    let runtime = first_party_runtime()?;
    let built = build_document_frame_graph(
        doc,
        EvaluationTime::new(RationalTime::ZERO),
        frame_desc,
        &DataTracks::new(),
        &runtime,
        None,
    )?;
    let mut session = RenderSession::new(gpu);
    let rendered = render_graph_cached(
        gpu,
        &mut session,
        RationalTime::ZERO,
        &built.graph,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::DRAFT,
    )?;
    Ok(rendered)
}

fn prepare_display_pool(gpu: &GpuCtx) -> Result<DisplayPool, StaticFrameError> {
    let doc = fixed_document()?;
    let rendered = render_static_frame(gpu, &doc)?;
    let pool = DisplayPool::new(gpu, rendered.desc);
    pool.copy_from_rendered(gpu, &rendered);
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use std::thread;

    use motolii_gpu::GpuCtx;
    use motolii_testkit::unavailable_dep;

    use super::*;

    #[test]
    fn setup_render_runs_off_ui_thread() {
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let handle = thread::Builder::new()
            .name("motolii-u1a1-setup".into())
            .spawn(move || {
                let name = thread::current().name().unwrap_or("").to_string();
                let pool = prepare_display_pool(&gpu).expect("pool");
                (pool, name)
            })
            .expect("spawn setup thread");
        let (pool, setup_thread) = handle.join().expect("setup thread join");
        assert_eq!(setup_thread, "motolii-u1a1-setup");
        assert!(pool.desc().width > 0);
        assert!(pool.desc().height > 0);
        assert_eq!(
            pool.desc().width,
            pool.desc().height * VIEWPORT_WIDTH / VIEWPORT_HEIGHT,
            "viewport aspect must match composition"
        );
    }

    #[test]
    fn spawn_join_entry_owns_off_event_loop_setup() {
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let pool = spawn_join_display_pool(gpu).expect("display pool");
        assert!(pool.desc().width > 0);
        assert!(pool.desc().height > 0);
        assert_eq!(
            pool.desc().width,
            pool.desc().height * VIEWPORT_WIDTH / VIEWPORT_HEIGHT,
            "viewport aspect must match composition"
        );
    }

    #[test]
    fn static_viewport_renders_and_copies_to_display_pool_without_cpu_readback() {
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let pool = prepare_display_pool(&gpu).expect("display pool");
        assert!(pool.desc().width > 0);
        assert!(pool.desc().height > 0);
        assert_eq!(
            pool.desc().width,
            pool.desc().height * VIEWPORT_WIDTH / VIEWPORT_HEIGHT,
            "viewport aspect must match composition"
        );
    }

    #[test]
    fn fixed_document_uses_in_memory_rect_layer_source() {
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let doc = fixed_document().expect("document");
        render_static_frame(&gpu, &doc).expect("render");
    }

    #[test]
    fn register_once_counter_stays_one_after_repeated_projection_seam() {
        use egui_wgpu::Renderer;

        DisplayPool::reset_register_count_for_test();
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let pool = prepare_display_pool(&gpu).expect("pool");
        let mut renderer = Renderer::new(
            &gpu.device,
            wgpu::TextureFormat::Bgra8Unorm,
            egui_wgpu::RendererOptions::default(),
        );
        let first = pool.register_once(&gpu.device, &mut renderer);
        let second = pool.register_once(&gpu.device, &mut renderer);
        assert_eq!(first, second);
        assert_eq!(DisplayPool::register_count_for_test(), 1);

        let mut browser = crate::browser_panel_spike::BrowserPanelState::default();
        for _ in 0..3 {
            egui::__run_test_ui(|ui| {
                crate::layout_preset::paint(ui, first, egui::vec2(320.0, 180.0), &mut browser);
            });
        }
        assert_eq!(DisplayPool::register_count_for_test(), 1);
    }
}
