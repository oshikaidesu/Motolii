//! macOS通常project sessionのdirect native Surface Host。
//!
//! 責任は各moduleが持ち、ここは組み立てと公開pathの再exportだけを行う。

mod app;
mod browser;
mod easing;
mod error;
mod inspector;
mod place;
mod place_overlay;
mod playback;
mod playhead;
mod position;
mod projection;
mod publish;
mod stage_transport;
mod surface;
mod timeline;
mod window_input;

use std::sync::Arc;
use std::time::Instant;

use motolii_core::{Quality, RationalTime};
use motolii_doc::EvaluationTime;
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use winit::event_loop::{ControlFlow, EventLoop};

use crate::browser_host_runtime::BrowserLifecycleEvent;
use crate::document_edit_runtime::DocumentEditRuntime;
use crate::render_worker::{RenderRequest, RenderWorker};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker};

use surface::ProductGpuParts;

pub(crate) use app::ProductApp;
pub(crate) use error::ProductRuntimeError;
pub(crate) use playback::ProductPlaybackError;
pub(crate) use surface::create_preview_pipeline;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ProductEvent {
    Wake,
    BrowserLifecycle(BrowserLifecycleEvent),
}

pub(crate) fn run(document_runtime: DocumentEditRuntime) -> Result<(), ProductRuntimeError> {
    let startup = Instant::now();
    crate::ui_numeric_trace::emit(format_args!("kind=startup phase=begin elapsed_ms=0.000"));
    let document = document_runtime.snapshot();
    let gpu_started = Instant::now();
    let (gpu, parts) = GpuCtx::new_for_ui()?;
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=gpu-ready phase_ms={:.3} elapsed_ms={:.3}",
        elapsed_ms(gpu_started),
        elapsed_ms(startup),
    ));
    let parts = ProductGpuParts {
        instance: parts.instance,
        adapter: parts.adapter,
        device: parts.device,
    };
    let gpu = Arc::new(gpu);
    let preview_started = Instant::now();
    let preview = Arc::new(prepare_in_setup_worker(
        Arc::clone(&gpu),
        Arc::clone(&document),
        bootstrap_frame_desc()?,
    )?);
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=preview-ready phase_ms={:.3} elapsed_ms={:.3} width={} height={}",
        elapsed_ms(preview_started),
        elapsed_ms(startup),
        preview.slot().desc().width,
        preview.slot().desc().height,
    ));
    let mut render_worker = RenderWorker::spawn(Arc::clone(&gpu))?;
    let render_client = render_worker.client();
    let event_loop = EventLoop::<ProductEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let proxy = event_loop.create_proxy();
    let wake_proxy = proxy.clone();
    render_client.register_repaint_signal(Arc::new(move || {
        let _ = wake_proxy.send_event(ProductEvent::Wake);
    }))?;
    let render_request_template = RenderRequest {
        document,
        data_tracks: Arc::new(DataTracks::new()),
        evaluation_time: EvaluationTime::new(RationalTime::ZERO),
        desc: bootstrap_frame_desc()?,
        quality: Quality::DRAFT,
    };
    let mut app = ProductApp::new(
        gpu,
        parts,
        preview,
        document_runtime,
        render_client,
        render_request_template,
        proxy,
    )?;
    crate::ui_numeric_trace::emit(format_args!(
        "kind=startup phase=event-loop-ready elapsed_ms={:.3}",
        elapsed_ms(startup),
    ));
    let run_result = event_loop.run_app(&mut app);
    render_worker.close();
    let join_result = render_worker.join();
    crate::ui_numeric_trace::emit(format_args!(
        "kind=shutdown phase=event-loop-exit elapsed_ms={:.3}",
        elapsed_ms(startup),
    ));
    run_result?;
    join_result?;
    if let Some(error) = app.failure {
        return Err(ProductRuntimeError::Runtime(error));
    }
    Ok(())
}

// Linux/Windows CIでもmacOS製品runtime全体をcompileし、private境界の接続欠落を検出する。
#[cfg(not(target_os = "macos"))]
fn compile_product_runtime() {
    let _: fn(DocumentEditRuntime) -> Result<(), ProductRuntimeError> = run;
    let _ = BrowserLifecycleEvent::ProcessTerminated { instance_epoch: 0 };
}

#[cfg(not(target_os = "macos"))]
const _: fn() = compile_product_runtime;

pub(super) fn elapsed_ms(started_at: Instant) -> f64 {
    started_at.elapsed().as_secs_f64() * 1_000.0
}
