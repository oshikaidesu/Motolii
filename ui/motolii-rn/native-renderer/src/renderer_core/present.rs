use std::time::Instant;

use motolii_ui::host_render_frame_for_app;

use super::RendererCore;
use super::host::{HostStageGeometryCommand, host_stage_geometry_command, stage_selection_commit};
use super::types::{SceneKind, StageResources, TimelineResources};

impl RendererCore {
    pub(crate) fn render(&mut self) -> Result<(), String> {
        let started = Instant::now();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(output)
            | wgpu::CurrentSurfaceTexture::Suboptimal(output) => output,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Err("surface lost".into()),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err("surface validation failure".into());
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        match self.scene {
            SceneKind::Stage => {
                self.process_stage_gizmo_action();
                self.present_stage(&view)?;
            }
            SceneKind::Timeline => {
                // 可視frameでもhost snapshotを載せる。warmup未達・stamp更新の両方を拾う。
                self.sync_host_timeline_projection();
                self.render_timeline(&view);
            }
        }

        output.present();
        self.frame = self.frame.wrapping_add(1);
        let cpu_us = started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64;
        self.stats.frame_count = self.stats.frame_count.wrapping_add(1);
        self.stats.last_cpu_us = cpu_us;
        self.stats.max_cpu_us = self.stats.max_cpu_us.max(cpu_us);
        self.stats.vertex_bytes = 0;
        Ok(())
    }

    fn present_stage(&mut self, view: &wgpu::TextureView) -> Result<(), String> {
        self.sync_host_stage_geometry();
        self.sync_host_stage_frame();
        let width = self.config.width;
        let height = self.config.height;
        let Some(stage) = self.stage.as_mut() else {
            return Ok(());
        };
        let StageResources {
            rerun, frame, gpu, ..
        } = stage;
        let selected_entity_path = rerun.render(
            &gpu.device,
            &gpu.queue,
            view,
            width,
            height,
            frame.as_ref().map(|frame| &frame.texture),
        )?;
        if let Some(selected_entity_path) = selected_entity_path {
            let commit = stage_selection_commit(selected_entity_path.as_deref());
            let _ = self.dispatch_timeline_selection(&commit);
        }
        Ok(())
    }

    fn sync_host_stage_frame(&mut self) {
        #[cfg(target_os = "macos")]
        {
            let Some(stage) = self.stage.as_mut() else {
                return;
            };
            if stage.preview_active {
                return;
            }
            let Some(handle) = crate::host_bridge::try_host_handle() else {
                return;
            };
            let _ =
                host_render_frame_for_app(handle, &stage.gpu, &mut stage.session, &mut stage.frame);
        }
    }

    fn sync_host_stage_geometry(&mut self) {
        #[cfg(target_os = "macos")]
        {
            if self.stage.is_none() {
                return;
            }
            if self
                .stage
                .as_ref()
                .is_some_and(|stage| stage.preview_active)
            {
                return;
            }
            let viewport = (self.config.width, self.config.height);
            // stampゲートはstage/timeline共通。forceと初回はfull読み。
            let read_stamp = crate::host_bridge::try_read_projection_stamp();
            let force = self.force_next_host_snapshot;
            let read_needed =
                Self::host_snapshot_read_needed(self.host_projection_stamp, read_stamp, force);
            let projection = if read_needed {
                let projection = crate::host_bridge::try_read_timeline_projection();
                if projection.is_some() {
                    self.host_projection_stamp = read_stamp;
                    if force {
                        // Stageはforceをscene再投影に使わないが、消費して無限full読みを防ぐ。
                        self.force_next_host_snapshot = false;
                    }
                } else {
                    self.host_projection_stamp = None;
                }
                projection
            } else {
                None
            };
            let stage = self.stage.as_mut().expect("stage present");
            let command = match projection {
                Some(ref projection) => {
                    if let Some(host_handle) = projection.host_handle.clone() {
                        self.host_handle = Some(host_handle);
                    }
                    self.host_revision = Some(projection.revision.clone());
                    self.host_projection_generation =
                        Some(projection.projection_generation.clone());
                    stage
                        .rerun
                        .set_host_primary_layer_id(projection.primary_layer_id.clone());
                    host_stage_geometry_command(self.host_stage_geometry.as_ref(), Some(projection))
                }
                None if read_needed => {
                    stage.rerun.set_host_primary_layer_id(None);
                    host_stage_geometry_command(self.host_stage_geometry.as_ref(), None)
                }
                None => {
                    // stamp不変: 既存geometryのviewport再適用だけ。
                    HostStageGeometryCommand::Noop
                }
            };
            match command {
                HostStageGeometryCommand::Apply(geometry) => {
                    if stage
                        .rerun
                        .apply_host_stage_geometry(&geometry, viewport.0, viewport.1)
                    {
                        self.host_stage_geometry = Some(geometry);
                        self.host_stage_viewport = Some(viewport);
                    }
                }
                HostStageGeometryCommand::Clear => {
                    if stage.rerun.clear_host_projection() {
                        self.host_stage_geometry = None;
                        self.host_stage_viewport = None;
                        stage.preview_active = false;
                    }
                }
                HostStageGeometryCommand::Noop => {
                    // geometry不変でもviewport aspectが変わったら再投影する。
                    if self.host_stage_viewport != Some(viewport) {
                        if let Some(geometry) = self.host_stage_geometry.clone() {
                            if stage
                                .rerun
                                .apply_host_stage_geometry(&geometry, viewport.0, viewport.1)
                            {
                                self.host_stage_viewport = Some(viewport);
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_timeline(&mut self, view: &wgpu::TextureView) {
        let needs_raster = self.timeline.as_ref().is_some_and(|t| t.dirty);
        if needs_raster {
            let scene = self
                .timeline_session
                .as_ref()
                .expect("timeline session")
                .scene
                .clone();
            let width = self.config.width;
            let height = self.config.height;
            let playhead = self.playhead;
            let selected = self.selected_object_index;
            let timeline = self.timeline.as_mut().expect("timeline resources");
            let raster_started = Instant::now();
            crate::timeline_skia::draw_timeline(
                &scene,
                &mut timeline.pixels,
                width,
                height,
                playhead,
                selected,
            );
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &timeline.surface_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &timeline.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            timeline.dirty = false;
            self.stats.overlay_uploads += 1;
            self.stats.overlay_last_us = raster_started.elapsed().as_micros() as u64;
        }

        let timeline = self.timeline.as_mut().expect("timeline resources");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Motolii Skia timeline"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Skia timeline blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.035,
                            g: 0.041,
                            b: 0.050,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&timeline.blit_pipeline);
            pass.set_bind_group(0, &timeline.blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
    }
}

pub(super) fn create_timeline_resources(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> TimelineResources {
    let surface_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Motolii Skia timeline raster"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Motolii Skia timeline blit shader"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var raster:texture_2d<f32>; @group(0) @binding(1) var samp:sampler;
            struct O { @builtin(position) p:vec4<f32>, @location(0) uv:vec2<f32> };
            @vertex fn vs(@builtin(vertex_index) i:u32)->O { var p=array<vec2<f32>,3>(vec2(-1.,-3.),vec2(3.,1.),vec2(-1.,1.)); var o:O; o.p=vec4(p[i],0.,1.); o.uv=vec2((p[i].x+1.)*.5,(1.-p[i].y)*.5); return o; }
            @fragment fn fs(i:O)->@location(0) vec4<f32> { return vec4(textureSample(raster,samp,i.uv).rgb,1.); }
        "#
            .into(),
        ),
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Motolii Skia timeline blit layout"),
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
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Motolii Skia timeline blit pipeline"),
        layout: Some(&layout),
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
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let blit_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &surface_texture.create_view(&Default::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    TimelineResources {
        surface_texture,
        blit_pipeline,
        blit_bind_group,
        pixels: vec![0; width as usize * height as usize * 4],
        dirty: true,
    }
}
