//! bootstrap Documentから静止display slotまでを閉じる単一preparation入口。

use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

use motolii_core::{ColorSpace, FrameDesc, PixelFormat, Quality, RationalTime, TimeMap};
use motolii_doc::{
    build_document_frame_graph, Clip, ClipSource, DocParam, Document, EvaluationTime, ItemEnvelope,
    LayerIdError, Track, TrackIdError, TrackItem, RECT_LAYER_SOURCE,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::first_party_runtime;
use motolii_render::{render_graph_cached, RenderGraphInputs, RenderSession};

use crate::display_slot::{DisplaySlot, DisplaySlotError, DisplaySlotEvidence};

const BOOTSTRAP_WIDTH: u32 = 64;
const BOOTSTRAP_HEIGHT: u32 = 36;
const BOOTSTRAP_COLOR: [f64; 4] = [0.0, 1.0, 0.0, 1.0];

#[derive(Debug, thiserror::Error)]
pub enum StaticPreviewError {
    #[error(transparent)]
    Document(#[from] motolii_doc::DocumentError),
    #[error(transparent)]
    Graph(#[from] motolii_doc::GraphError),
    #[error(transparent)]
    Render(#[from] motolii_render::RenderError),
    #[error(transparent)]
    Runtime(#[from] motolii_plugins_firstparty::FirstPartyError),
    #[error(transparent)]
    LayerId(#[from] LayerIdError),
    #[error(transparent)]
    TrackId(#[from] TrackIdError),
    #[error(transparent)]
    FrameDesc(#[from] motolii_core::FrameDescError),
    #[error("display slot requires Rgba8Unorm, got {0:?}")]
    DisplayUnsupportedFormat(PixelFormat),
    #[error("display slot descriptor mismatch: expected {expected:?}, got {actual:?}")]
    DisplayDescriptorMismatch {
        expected: FrameDesc,
        actual: FrameDesc,
    },
    #[error(transparent)]
    Gpu(#[from] motolii_gpu::GpuRuntimeError),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("failed to spawn static preview setup worker: {0}")]
    SetupThreadSpawn(#[from] std::io::Error),
    #[error("static preview setup worker panicked")]
    SetupThreadPanic,
}

impl From<DisplaySlotError> for StaticPreviewError {
    fn from(error: DisplaySlotError) -> Self {
        match error {
            DisplaySlotError::UnsupportedFormat(format) => Self::DisplayUnsupportedFormat(format),
            DisplaySlotError::DescriptorMismatch { expected, actual } => {
                Self::DisplayDescriptorMismatch { expected, actual }
            }
        }
    }
}

pub(crate) struct StaticPreview {
    _gpu: Arc<GpuCtx>,
    document_json: String,
    slot: DisplaySlot,
    render_count: u32,
}

impl StaticPreview {
    #[cfg(test)]
    pub(crate) fn gpu(&self) -> &GpuCtx {
        &self._gpu
    }

    pub(crate) fn slot(&self) -> &DisplaySlot {
        &self.slot
    }

    pub(crate) fn invariant_evidence(&self) -> StaticPreviewEvidence {
        StaticPreviewEvidence {
            document_json: self.document_json.clone(),
            slot: self.slot.evidence(),
            render_count: self.render_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticPreviewEvidence {
    pub(crate) document_json: String,
    pub(crate) slot: DisplaySlotEvidence,
    pub(crate) render_count: u32,
}

pub(crate) fn bootstrap_frame_desc() -> Result<FrameDesc, StaticPreviewError> {
    Ok(FrameDesc::try_packed(
        BOOTSTRAP_WIDTH,
        BOOTSTRAP_HEIGHT,
        PixelFormat::Rgba8Unorm,
        ColorSpace::Srgb,
        true,
    )?)
}

pub(crate) fn bootstrap_document() -> Result<Document, StaticPreviewError> {
    fixture_document(BOOTSTRAP_COLOR)
}

fn fixture_document(color: [f64; 4]) -> Result<Document, StaticPreviewError> {
    let mut document = Document::new_current();
    let layer = document.layers.allocate("static-preview")?;
    let track = document.track_ids.allocate("static-preview")?;
    document.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: document.composition.duration,
            time_map: TimeMap::identity(),
            source: ClipSource::Plugin {
                plugin_id: RECT_LAYER_SOURCE.into(),
                effect_version: 1,
                params: BTreeMap::from([
                    ("center".into(), DocParam::const_vec2([0.0, 0.0])),
                    ("size".into(), DocParam::const_vec2([4.0, 4.0])),
                    ("color".into(), DocParam::const_color(color)),
                ]),
                extra: Default::default(),
            },
        })],
    });
    document.validate()?;
    Ok(document)
}

pub(crate) fn prepare_in_setup_worker(
    gpu: Arc<GpuCtx>,
    document: Arc<Document>,
    desc: FrameDesc,
) -> Result<StaticPreview, StaticPreviewError> {
    let handle = thread::Builder::new()
        .name("motolii-u1a1-static-setup".into())
        .spawn(move || {
            prepare_static_viewport(gpu, document, EvaluationTime::new(RationalTime::ZERO), desc)
        })?;
    handle
        .join()
        .map_err(|_| StaticPreviewError::SetupThreadPanic)?
}

fn prepare_static_viewport(
    gpu: Arc<GpuCtx>,
    document: Arc<Document>,
    evaluation_time: EvaluationTime,
    desc: FrameDesc,
) -> Result<StaticPreview, StaticPreviewError> {
    document.validate()?;
    let document_json = serde_json::to_string(&*document)?;
    gpu.check_health()?;
    let runtime = first_party_runtime()?;
    let built = build_document_frame_graph(
        &document,
        evaluation_time,
        desc,
        &DataTracks::new(),
        &runtime,
        None,
    )?;
    let mut session = RenderSession::new(&gpu);
    let rendered = render_graph_cached(
        &gpu,
        &mut session,
        evaluation_time.timeline_time,
        &built.graph,
        &RenderGraphInputs {
            camera: built.camera,
            video_sources: &[],
            source_time: Some(built.source_time),
            plugins: Some(runtime.executors()),
        },
        Quality::DRAFT,
    )?;
    let slot = DisplaySlot::copy_from_rendered(&gpu, &rendered)?;
    gpu.check_health()?;
    Ok(StaticPreview {
        _gpu: gpu,
        document_json,
        slot,
        render_count: 1,
    })
}

#[cfg(test)]
mod tests {
    use egui_wgpu::{Renderer, RendererOptions};
    use motolii_gpu::{download_rgba, GpuRuntimeError};
    use motolii_testkit::unavailable_dep;

    use super::*;
    use crate::app::{ShellLifecycleInput, StaticViewportProjection};

    const RED: [f64; 4] = [1.0, 0.0, 0.0, 1.0];
    const GREEN: [f64; 4] = [0.0, 1.0, 0.0, 1.0];

    #[test]
    fn one_private_path_proves_document_render_copy_and_register_once() {
        let Ok(gpu) = GpuCtx::new_headless() else {
            unavailable_dep("GPU adapter", "new_headless failed");
            return;
        };
        let gpu = Arc::new(gpu);
        let desc = bootstrap_frame_desc().expect("frame desc");
        let red = prepare_static_viewport(
            Arc::clone(&gpu),
            Arc::new(fixture_document(RED).expect("red fixture")),
            EvaluationTime::new(RationalTime::ZERO),
            desc,
        )
        .expect("red preview");
        let green = prepare_static_viewport(
            Arc::clone(&gpu),
            Arc::new(fixture_document(GREEN).expect("green fixture")),
            EvaluationTime::new(RationalTime::ZERO),
            desc,
        )
        .expect("green preview");

        let red_bytes = download_rgba(&gpu, red.slot().texture()).expect("red download");
        let green_bytes = download_rgba(&gpu, green.slot().texture()).expect("green download");
        let rendered_desc = Quality::DRAFT.render_desc(desc);
        assert_eq!(red.slot().desc(), rendered_desc);
        assert_eq!(green.slot().desc(), rendered_desc);
        assert_solid(&red_bytes, rendered_desc, [255, 0, 0, 255]);
        assert_solid(&green_bytes, rendered_desc, [0, 255, 0, 255]);
        assert_ne!(red_bytes, green_bytes);

        let mut renderer = Renderer::new(
            &gpu.device,
            wgpu::TextureFormat::Bgra8Unorm,
            RendererOptions::default(),
        );
        let first = red.slot().register_once(&gpu.device, &mut renderer);
        let second = red.slot().register_once(&gpu.device, &mut renderer);
        assert_eq!(first, second);
        assert_eq!(red.slot().evidence().registration_count, 1);
    }

    #[test]
    fn lifecycle_projection_cannot_change_document_or_gpu_evidence() {
        let Ok(gpu) = GpuCtx::new_headless() else {
            unavailable_dep("GPU adapter", "new_headless failed");
            return;
        };
        let preview = prepare_static_viewport(
            Arc::new(gpu),
            Arc::new(bootstrap_document().expect("fixture")),
            EvaluationTime::new(RationalTime::ZERO),
            bootstrap_frame_desc().expect("desc"),
        )
        .expect("preview");
        let mut renderer = Renderer::new(
            &preview.gpu().device,
            wgpu::TextureFormat::Bgra8Unorm,
            RendererOptions::default(),
        );
        preview
            .slot()
            .register_once(&preview.gpu().device, &mut renderer);
        let before = preview.invariant_evidence();
        let mut projection = StaticViewportProjection::new(&preview);
        for input in [
            ShellLifecycleInput::Resized([960.0, 640.0]),
            ShellLifecycleInput::ScaleFactorChanged(2.0),
            ShellLifecycleInput::Minimized,
            ShellLifecycleInput::Restored,
            ShellLifecycleInput::Resized([800.0, 600.0]),
        ] {
            projection
                .observe(input, &preview)
                .expect("shell lifecycle must preserve preview evidence");
        }
        assert_eq!(preview.invariant_evidence(), before);
    }

    #[test]
    fn ui_shared_display_slot_still_rejects_cpu_readback() {
        let Ok((gpu, _parts)) = GpuCtx::new_for_ui() else {
            unavailable_dep("GPU adapter", "new_for_ui failed");
            return;
        };
        let preview = prepare_static_viewport(
            Arc::new(gpu),
            Arc::new(bootstrap_document().expect("fixture")),
            EvaluationTime::new(RationalTime::ZERO),
            bootstrap_frame_desc().expect("desc"),
        )
        .expect("preview");
        let error = download_rgba(preview.gpu(), preview.slot().texture()).unwrap_err();
        assert!(matches!(error, GpuRuntimeError::SyncReadbackForbidden));
        assert!(matches!(
            preview.gpu().poll_wait(None),
            Err(GpuRuntimeError::SyncReadbackForbidden)
        ));
    }

    fn solid_rgba(desc: FrameDesc, pixel: [u8; 4]) -> Vec<u8> {
        let mut bytes = vec![0; desc.data_size()];
        for output in bytes.chunks_exact_mut(4) {
            output.copy_from_slice(&pixel);
        }
        bytes
    }

    fn assert_solid(actual: &[u8], desc: FrameDesc, pixel: [u8; 4]) {
        let expected = solid_rgba(desc, pixel);
        let mismatches: Vec<_> = actual
            .iter()
            .zip(&expected)
            .enumerate()
            .filter_map(|(index, (actual, expected))| {
                (actual != expected).then_some((index, *actual, *expected))
            })
            .take(16)
            .collect();
        assert_eq!(
            actual.len(),
            expected.len(),
            "length mismatch; first pixel mismatches: {mismatches:?}"
        );
        assert!(
            mismatches.is_empty(),
            "expected independent solid oracle; first mismatches: {mismatches:?}"
        );
    }
}
