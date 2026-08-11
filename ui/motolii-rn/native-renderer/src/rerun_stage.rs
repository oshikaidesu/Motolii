use std::{sync::Arc, time::Instant};

use egui::{Event, PointerButton, Pos2, RawInput, Rect, Vec2};
use re_chunk::Chunk;
use re_log_types::TimePoint;
use re_sdk_types::archetypes::Mesh3D;

use crate::renderer_core::PointerPhase;

/// Owns only the adapter from the native Stage surface to Rerun's Spatial View.
///
/// Product chrome, persistence, and Document projection stay outside this adapter.
pub(crate) struct EmbeddedSpatialStage {
    egui_ctx: egui::Context,
    egui_renderer: egui_wgpu::Renderer,
    spatial_stage: re_view_spatial::SpatialStage,
    input_events: Vec<Event>,
    started_at: Instant,
}

impl EmbeddedSpatialStage {
    pub(crate) fn new(
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let mut egui_renderer = egui_wgpu::Renderer::new(
            device,
            surface_format,
            egui_wgpu::RendererOptions::default(),
        );
        let render_ctx = re_renderer::RenderContext::new(
            adapter,
            device.clone(),
            queue.clone(),
            surface_format,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .map_err(|error| format!("create Rerun render context: {error}"))?;
        egui_renderer.callback_resources.insert(render_ctx);

        let mut stage = Self {
            egui_ctx: egui::Context::default(),
            egui_renderer,
            spatial_stage: re_view_spatial::SpatialStage::new(re_log_types::ApplicationId::from(
                "motolii-rn-stage",
            ))
            .map_err(|error| format!("create Rerun spatial stage: {error}"))?,
            input_events: Vec::new(),
            started_at: Instant::now(),
        };
        if !stage.set_created_item("rectangle@0.500000,0.500000") {
            return Err("seed path rectangle for embedded stage".into());
        }
        Ok(stage)
    }

    pub(crate) fn pointer(&mut self, phase: PointerPhase, x: f64, y: f64) {
        let position = Pos2::new(x as f32, y as f32);
        self.input_events.push(Event::PointerMoved(position));

        match phase {
            PointerPhase::Down => self.input_events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Default::default(),
            }),
            PointerPhase::Up => self.input_events.push(Event::PointerButton {
                pos: position,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            }),
            PointerPhase::Cancel => self.input_events.push(Event::PointerGone),
            PointerPhase::Move => {}
        }
    }

    pub(crate) fn set_created_item(&mut self, item_id: &str) -> bool {
        if item_id.is_empty() {
            return true;
        }
        let Some((kind, coordinates)) = item_id.split_once('@') else {
            return false;
        };
        if kind != "rectangle" {
            return false;
        }
        let Some((x, y)) = coordinates.split_once(',') else {
            return false;
        };
        let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>()) else {
            return false;
        };
        if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
            return false;
        }

        let center = [x - 0.5, y - 0.5, 0.0];
        let (half_width, half_height) = (0.14, 0.10);
        let mesh = Mesh3D::new([
            [center[0] - half_width, center[1] - half_height, center[2]],
            [center[0] + half_width, center[1] - half_height, center[2]],
            [center[0] + half_width, center[1] + half_height, center[2]],
            [center[0] - half_width, center[1] + half_height, center[2]],
        ])
        .with_triangle_indices([[0, 1, 2], [0, 2, 3]])
        .with_vertex_normals([[0.0, 0.0, 1.0]; 4])
        .with_vertex_colors([0xE9_8C_6AFF; 4]);
        let Ok(chunk) = Chunk::builder("motolii/fixtures/path-rectangle")
            .with_archetype_auto_row(TimePoint::default(), &mesh)
            .build()
        else {
            return false;
        };
        self.spatial_stage.ingest_chunk(Arc::new(chunk)).is_ok()
    }

    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        let raw_input = RawInput {
            screen_rect: Some(Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(width as f32, height as f32),
            )),
            time: Some(self.started_at.elapsed().as_secs_f64()),
            events: std::mem::take(&mut self.input_events),
            ..Default::default()
        };

        let egui_ctx = self.egui_ctx.clone();
        let mut stage_error = None;
        let full_output = egui_ctx.run_ui(raw_input, |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    let mut render_ctx = self
                        .egui_renderer
                        .callback_resources
                        .remove::<re_renderer::RenderContext>()
                        .expect("Rerun render context is registered for the native Stage");
                    let result = self.spatial_stage.show(ui, &mut render_ctx);
                    self.egui_renderer.callback_resources.insert(render_ctx);
                    if let Err(error) = result {
                        stage_error = Some(error.to_string());
                    }
                });
        });
        if let Some(error) = stage_error {
            return Err(format!("run Rerun spatial stage: {error}"));
        }

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(device, queue, *id, delta);
        }
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = egui_ctx.tessellate(full_output.shapes, pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Motolii Rerun Spatial Stage"),
        });
        let callback_command_buffers = self.egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Motolii Rerun Spatial Stage present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }
        queue.submit(
            callback_command_buffers
                .into_iter()
                .chain([encoder.finish()]),
        );

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        Ok(())
    }
}
