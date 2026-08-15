//! native Surfaceとpreview pipeline。GPU partsはproductが所有する。

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{CanonicalPoint, Fps, FpsError, Quality, RationalTime, RationalTimeError};
use motolii_doc::{
    Command, DocParam, DocValue, EffectId, EvaluationTime, KeyframeId, LayerId, TrackItem,
};
use motolii_eval::DataTracks;
use motolii_gpu::GpuCtx;
use motolii_transport::{FramePlan, PlaybackSession, PlaybackSessionError, TransportError};
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::window::Window;

use crate::app::canonical_drop_from_ndc;
use crate::browser_host::BrowserPlaceIntent;
use crate::browser_host_runtime::{
    BrowserFocusTarget, BrowserHostRuntime, BrowserHostRuntimeError, BrowserLifecycleEvent,
};
use crate::document_edit_runtime::{
    AddPositionKeyRequest, AttachEffectRequest, DocumentEditDispatchError, DocumentEditQueue,
    DocumentEditRuntime, DocumentEditRuntimeError, PlaceRectangleRequest, PublishedDocument,
    SetPositionKeyInterpRequest, SetPositionKeyValueRequest,
};
use crate::host_pointer_capture::{HostPointerCancel, HostPointerCandidate};
use crate::inspector_host_runtime::{
    resolve_effect_param_preview_command, InspectorGestureTerminal, InspectorGestureTerminalCause,
    InspectorHostRuntime, InspectorHostRuntimeError, InspectorPositionAxis,
    InspectorPositionGestureStart, InspectorPositionGestureTerminal,
    InspectorPositionGestureTerminalCause,
};
use crate::layout_authority::LayoutAuthority;
use crate::native_host_layout::{
    key_tools_logical_rect, timeline_ruler_logical_rect, timeline_time_surface_logical_rect,
    LogicalRect, NativeHostLayout, PhysicalRect,
};
use crate::product_easing_popup::{
    PopupTerminal, ProductEasingPopup, ProductEasingPopupError, ProductEasingPopupOpen,
};
use crate::render_worker::{
    RenderGeneration, RenderRequest, RenderWorker, RenderWorkerClient, RenderWorkerError,
};
use crate::stage_chrome_host_runtime::{
    StageChromeHostRuntime, StageChromeHostRuntimeError, StageEasingIntent, StageEasingIntentError,
    StagePlaybackIntentError, StagePlaybackState, StagePlaybackToggle, StageTransportSnapshot,
};
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::{TimelineMoveGesture, TimelineMoveRequest};
use crate::timeline_projection::{
    project_timeline, TimelineHit, TimelineMetrics, TimelineProjection, TimelineProjectionError,
    TimelineViewport,
};
use crate::timeline_tools_host_runtime::{TimelineToolsHostRuntime, TimelineToolsHostRuntimeError};
use crate::timeline_trim_gesture::{TimelineTrimEdge, TimelineTrimGesture};
use crate::{
    builtin_command_registry, default_user_keymap_override_path, load_user_keymap_override,
    product_builtin_keymap, resolve_keymap, CommandIdError, CommandRegistry, CommandRegistryError,
    DomainIntent, EffectiveTrigger, ImeGateState, InputPhase, InputRouter, InputRouterError,
    KeyToken, KeymapResolution, ModifierError, Modifiers, NormalizedInput,
    PlatformBindingConstraints, PlatformCommandModifier, RouterOutput, SafetyInterrupt,
};

use super::error::ProductRuntimeError;
use super::place_overlay::RectanglePlaceOverlay;

pub(super) struct ProductSurface {
    pub(super) instance: wgpu::Instance,
    pub(super) adapter: wgpu::Adapter,
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) gpu: Arc<GpuCtx>,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) preview_pipeline: wgpu::RenderPipeline,
    pub(super) preview_bind_group: wgpu::BindGroup,
    pub(super) place_overlay_pipeline: wgpu::RenderPipeline,
    pub(super) place_overlay_vertices: wgpu::Buffer,
    pub(super) occluded: bool,
}

pub(super) struct ProductGpuParts {
    pub(super) instance: wgpu::Instance,
    pub(super) adapter: wgpu::Adapter,
    pub(super) device: wgpu::Device,
}

impl ProductSurface {
    pub(super) fn new(
        window: &Arc<Window>,
        parts: ProductGpuParts,
        gpu: &Arc<GpuCtx>,
        preview: &StaticPreview,
    ) -> Result<Self, ProductRuntimeError> {
        let surface = parts.instance.create_surface(Arc::clone(window))?;
        if !parts.adapter.is_surface_supported(&surface) {
            return Err(ProductRuntimeError::SurfaceUnsupported);
        }
        let size = window.inner_size();
        let capabilities = surface.get_capabilities(&parts.adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(ProductRuntimeError::SurfaceUnsupported)?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&parts.device, &config);
        let (preview_pipeline, preview_bind_group) =
            create_preview_pipeline(&parts.device, format, preview.slot().view());
        let place_overlay_pipeline = create_place_overlay_pipeline(&parts.device, format);
        let place_overlay_vertices = parts.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-product-place-overlay-vertices"),
            size: 48,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            instance: parts.instance,
            adapter: parts.adapter,
            surface,
            gpu: Arc::clone(gpu),
            config,
            preview_pipeline,
            preview_bind_group,
            place_overlay_pipeline,
            place_overlay_vertices,
            occluded: false,
        })
    }

    pub(super) fn configure(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.reconfigure();
    }

    pub(super) fn reconfigure(&self) {
        self.surface.configure(&self.gpu.device, &self.config);
    }

    pub(super) fn render(
        &mut self,
        layout: NativeHostLayout,
        window: &Window,
        place_overlay: Option<&RectanglePlaceOverlay>,
    ) -> Result<(), ProductSurfaceError> {
        if self.occluded || self.config.width == 0 || self.config.height == 0 {
            return Err(ProductSurfaceError::Skip);
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Occluded => {
                return Err(ProductSurfaceError::Retry);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return Err(ProductSurfaceError::Recover);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ProductSurfaceError::Fatal(
                    "native product Surface validation failed".to_owned(),
                ));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-product-native-frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-product-stage-timeline"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.020,
                            b: 0.024,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            draw_rect(
                &mut pass,
                layout.stage_physical,
                &self.preview_pipeline,
                Some(&self.preview_bind_group),
            );
            if let Some(place_overlay) = place_overlay {
                let bytes = place_overlay.vertex_bytes();
                self.gpu
                    .queue
                    .write_buffer(&self.place_overlay_vertices, 0, &bytes);
                pass.set_pipeline(&self.place_overlay_pipeline);
                pass.set_vertex_buffer(0, self.place_overlay_vertices.slice(..));
                pass.set_viewport(
                    layout.stage_physical.x as f32,
                    layout.stage_physical.y as f32,
                    layout.stage_physical.width as f32,
                    layout.stage_physical.height as f32,
                    0.0,
                    1.0,
                );
                pass.set_scissor_rect(
                    layout.stage_physical.x,
                    layout.stage_physical.y,
                    layout.stage_physical.width,
                    layout.stage_physical.height,
                );
                pass.draw(0..6, 0..1);
            }
        }
        self.gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        frame.present();
        Ok(())
    }
}

pub(super) fn draw_rect<'a>(
    pass: &mut wgpu::RenderPass<'a>,
    rect: PhysicalRect,
    pipeline: &'a wgpu::RenderPipeline,
    bind_group: Option<&'a wgpu::BindGroup>,
) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = bind_group {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.set_viewport(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
        0.0,
        1.0,
    );
    pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);
    pass.draw(0..3, 0..1);
}

pub(crate) fn create_preview_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    view: &wgpu::TextureView,
) -> (wgpu::RenderPipeline, wgpu::BindGroup) {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("motolii-product-preview-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-product-preview-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("motolii-product-preview-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    (
        create_pipeline(device, format, Some(&layout), PREVIEW_SHADER),
        bind_group,
    )
}

pub(super) fn create_place_overlay_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-place-overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(PLACE_OVERLAY_SHADER)),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-place-overlay-layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-place-overlay-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: 8,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: Option<&wgpu::BindGroupLayout>,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-product-native-shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source)),
    });
    let layouts: Vec<_> = bind_group_layout.into_iter().map(Some).collect();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-product-native-pipeline-layout"),
        bind_group_layouts: &layouts,
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-product-native-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(format.into())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

pub(super) const PREVIEW_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }
@vertex fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    var uvs = array<vec2<f32>, 3>(vec2(0.0,1.0), vec2(2.0,1.0), vec2(0.0,-1.0));
    var out: VertexOut; out.position = vec4(positions[index],0.0,1.0); out.uv = uvs[index]; return out;
}
@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@fragment fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(source_texture, source_sampler, in.uv);
}
"#;
pub(super) const PLACE_OVERLAY_SHADER: &str = r#"
struct VertexOut { @builtin(position) position: vec4<f32> }
@vertex fn vs_main(@location(0) position: vec2<f32>) -> VertexOut {
    var out: VertexOut; out.position = vec4(position, 0.0, 1.0); return out;
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.8, 0.58431375, 0.5294118, 0.42);
}
"#;

#[derive(Debug, thiserror::Error)]
pub(super) enum ProductSurfaceError {
    #[error("native product Surface must be reconfigured")]
    Recover,
    #[error("native product Surface frame must be retried")]
    Retry,
    #[error("native product Surface frame is skipped")]
    Skip,
    #[error("native product Surface failed: {0}")]
    Fatal(String),
}
