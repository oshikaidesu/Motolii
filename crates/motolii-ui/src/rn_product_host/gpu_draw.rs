//! macOS surface への preview/overlay 描画と GPU op 入口。surface 寿命は gpu_surface。

use std::collections::{HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use motolii_audio::{AudioProgram, PcmCache, CANONICAL_SAMPLE_RATE};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat, PixelSize, Quality, RationalTime};
use motolii_doc::{
    build_document_frame_graph, layer_names_for_item, prepare_set_transform_param_key_value,
    Affine2D, Clip, ClipSource, Command, DocParam, DocValue, EffectId, EvaluationTime,
    ItemEnvelope, KeyframeId, LayerId, ParentLocator, ResolvedLayerParams, ScalarPropertyId,
    TrackItem,
};
use motolii_eval::{DataTracks, Interp};
use motolii_export::{export_document_video, ExportJob, VideoSourceBinder};
use motolii_gpu::GpuCtx;
use motolii_plugins_firstparty::{first_party_catalog, first_party_runtime};
use motolii_render::{
    render_graph_cached, validate_render_graph_wiring, RenderGraphInputs, RenderSession,
};
use motolii_transport::PlaybackSession;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_os = "macos")]
use wgpu::{
    Color, CompositeAlphaMode, CurrentSurfaceTexture, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, Surface, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

use crate::document_edit_runtime::{
    prepare_set_effect_param_command, prepare_set_source_param_command, AddPositionKeyRequest,
    AddTransformParamKeyRequest, AttachEffectRequest, DocumentEditQueue, DocumentEditRuntime,
    DocumentEditRuntimeError, PlaceEllipseRequest, PlaceMediaRequest, PlaceRectangleRequest,
    PlaceVismRequest, PublishedDocument, RemovePositionKeyRequest, SetEffectParamRequest,
    SetOpacityRequest, SetPositionConstRequest, SetPositionKeyInterpRequest,
    SetPositionKeyTimeRequest, SetPositionKeyValueRequest, SetSourceParamRequest,
};
use crate::media_library::{LibraryProjection, MediaLibrary};
use crate::shell::{open_project_runtime, ShellError};
use crate::stage_geometry_projection::{project_stage_geometry, StageLayerProjection};
use crate::stage_hit_test::{
    hit_test_projected_layers, view_local_in_stage, view_local_to_canonical, StageHit,
    StageHitTestReject,
};
use crate::stage_overlay_gpu::{overlay_dimensions_match, overlay_dirty, OverlayUploadKey};
use crate::stage_overlay_raster::{raster_selection_outline, StageOverlayRaster};
#[cfg(target_os = "macos")]
use crate::static_preview::{bootstrap_frame_desc, prepare_in_setup_worker, StaticPreview};
use crate::timeline_move_gesture::TimelineMoveRequest;
use crate::timeline_trim_gesture::TimelineTrimRequest;
use crate::{CommandId, DocumentCommandRequest, DomainIntent, InputPhase, RouterOutput};

use super::app_api::*;
use super::dispatch::*;
use super::error::*;
#[cfg(target_os = "macos")]
use super::ffi::*;
use super::ffi_surface::*;
use super::gpu_ops::*;
use super::gpu_surface::*;
use super::host::*;
use super::key_projection::*;
use super::playback::*;
use super::projection::*;
use super::registry::*;
use super::stage_projection::*;
use super::surfaces::*;
use super::timeline_gpu::*;
use super::wire::*;
use super::wire_io::*;
#[cfg(target_os = "macos")]
use super::MAX_PROJECT_PATH_BYTES;
use super::{
    HOST_TO_RN, MAX_DIAGNOSTICS, MAX_EFFECTS_PER_LAYER, MAX_JSON_BYTES, MAX_POSITION_KEYS,
    MAX_SNAPSHOT_JSON_BYTES, MAX_SOURCE_PARAMS_PER_LAYER, MAX_STAGE_BOUNDS, MAX_STAGE_SELECTION,
    PRODUCT_ROLE, RN_TO_HOST, WIRE_VERSION,
};

#[cfg(target_os = "macos")]
pub(super) fn draw_stage_preview(
    gpu: &HostGpuBundle,
    frame: wgpu::SurfaceTexture,
    overlay: Option<&StageOverlayGpu>,
) {
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-rn-stage-preview"),
        });
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-stage-preview-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&gpu.preview_pipeline);
        pass.set_bind_group(0, &gpu.preview_bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    if let Some(overlay) = overlay {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-stage-overlay-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&gpu.overlay_pipeline);
        pass.set_bind_group(0, &overlay.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.ctx.queue.submit(Some(encoder.finish()));
    frame.present();
}

#[cfg(target_os = "macos")]
pub(super) fn draw_timeline_seat(
    gpu: &HostGpuBundle,
    frame: wgpu::SurfaceTexture,
    raster: Option<&StageOverlayGpu>,
) {
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = gpu
        .ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("motolii-rn-timeline-seat"),
        });
    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("motolii-rn-timeline-seat-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.02,
                        g: 0.02,
                        b: 0.025,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(raster) = raster {
            pass.set_pipeline(&gpu.overlay_pipeline);
            pass.set_bind_group(0, &raster.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    gpu.ctx.queue.submit(Some(encoder.finish()));
    frame.present();
}

#[cfg(target_os = "macos")]
pub(super) fn upload_stage_overlay(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    current: &mut Option<StageOverlayGpu>,
    raster: StageOverlayRaster,
) {
    let recreate = current
        .as_ref()
        .is_none_or(|overlay| overlay.width != raster.width || overlay.height != raster.height);
    if recreate {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-rn-stage-overlay-texture"),
            size: wgpu::Extent3d {
                width: raster.width,
                height: raster.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-rn-stage-overlay-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-rn-stage-overlay-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        *current = Some(StageOverlayGpu {
            texture,
            _view: view,
            _sampler: sampler,
            bind_group,
            width: raster.width,
            height: raster.height,
        });
    }
    let overlay = current.as_ref().expect("overlay texture initialized");
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &overlay.texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &raster.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(raster.width * 4),
            rows_per_image: Some(raster.height),
        },
        wgpu::Extent3d {
            width: raster.width,
            height: raster.height,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(target_os = "macos")]
pub(super) fn create_overlay_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("motolii-rn-stage-overlay-layout"),
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
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("motolii-rn-stage-overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
            "struct V { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32> }\n@vertex fn vs(@builtin(vertex_index) i: u32) -> V { var p = array<vec2<f32>, 3>(vec2(-1., -1.), vec2(3., -1.), vec2(-1., 3.)); var u = array<vec2<f32>, 3>(vec2(0., 1.), vec2(2., 1.), vec2(0., -1.)); var o: V; o.p = vec4(p[i], 0., 1.); o.uv = u[i]; return o; }\n@group(0) @binding(0) var t: texture_2d<f32>;\n@group(0) @binding(1) var s: sampler;\n@fragment fn fs(i: V) -> @location(0) vec4<f32> { return textureSample(t, s, i.uv); }\n",
        )),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("motolii-rn-stage-overlay-pipeline-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("motolii-rn-stage-overlay-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    (pipeline, layout)
}

#[cfg(target_os = "macos")]
pub(super) fn supported_surface_config(
    surface: &Surface<'static>,
    adapter: &wgpu::Adapter,
    width: u32,
    height: u32,
) -> Result<SurfaceConfiguration, RnHostReasonCode> {
    let capabilities = surface.get_capabilities(adapter);
    if !capabilities.formats.contains(&TextureFormat::Bgra8Unorm)
        || !capabilities.present_modes.contains(&PresentMode::Fifo)
        || !capabilities.alpha_modes.contains(&CompositeAlphaMode::Auto)
    {
        return Err(RnHostReasonCode::InvalidIntent);
    }
    Ok(SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width,
        height,
        present_mode: PresentMode::Fifo,
        desired_maximum_frame_latency: 2,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: Vec::new(),
    })
}
