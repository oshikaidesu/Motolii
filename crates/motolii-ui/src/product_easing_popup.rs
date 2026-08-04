//! P04-C2だけのprivate native Easing child surface。

use std::sync::Arc;

use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use motolii_eval::Interp;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId, WindowLevel},
};

use crate::native_host_layout::LogicalRect;

pub(crate) struct ProductEasingPopup {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    gpu: Arc<motolii_gpu::GpuCtx>,
    context: egui::Context,
    state: egui_winit::State,
    renderer: Renderer,
    curve: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PopupTerminal {
    Cancel,
    Commit(Interp),
}

impl ProductEasingPopup {
    pub(crate) fn open(
        event_loop: &ActiveEventLoop,
        host: &Window,
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        gpu: Arc<motolii_gpu::GpuCtx>,
        anchor: LogicalRect,
        interp: Interp,
    ) -> Result<Self, ProductEasingPopupError> {
        let scale = host.scale_factor();
        let origin = host.outer_position().unwrap_or(PhysicalPosition::new(0, 0));
        let position = PhysicalPosition::new(
            origin.x.saturating_add((anchor.x * scale).round() as i32),
            origin
                .y
                .saturating_add(((anchor.y + anchor.height) * scale).round() as i32),
        );
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_title("Motolii · Interval Easing")
                    .with_inner_size(LogicalSize::new(360.0, 240.0))
                    .with_position(position)
                    .with_decorations(false)
                    .with_resizable(false)
                    .with_window_level(WindowLevel::AlwaysOnTop),
            )?,
        );
        let surface = instance.create_surface(Arc::clone(&window))?;
        if !adapter.is_surface_supported(&surface) {
            return Err(ProductEasingPopupError::SurfaceUnsupported);
        }
        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .first()
            .copied()
            .ok_or(ProductEasingPopupError::SurfaceUnsupported)?;
        let alpha_mode = capabilities
            .alpha_modes
            .first()
            .copied()
            .ok_or(ProductEasingPopupError::SurfaceUnsupported)?;
        let size = window.inner_size();
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
        surface.configure(&gpu.device, &config);
        let context = egui::Context::default();
        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            &*window,
            None,
            None,
            None,
        );
        let curve = match interp {
            Interp::Linear => [0.0, 0.0, 1.0, 1.0],
            Interp::Bezier { x1, y1, x2, y2 } => [x1 as f32, y1 as f32, x2 as f32, y2 as f32],
            Interp::Hold => [0.0, 0.0, 1.0, 1.0],
        };
        Ok(Self {
            window,
            surface,
            config,
            gpu: Arc::clone(&gpu),
            context,
            state,
            renderer: Renderer::new(&gpu.device, format, RendererOptions::default()),
            curve,
        })
    }

    pub(crate) fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &winit::event::WindowEvent,
    ) -> Result<Option<PopupTerminal>, ProductEasingPopupError> {
        if matches!(
            event,
            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Focused(false)
        ) {
            return Ok(Some(PopupTerminal::Cancel));
        }
        if let winit::event::WindowEvent::Resized(size) = event {
            if size.width > 0 && size.height > 0 {
                self.config.width = size.width;
                self.config.height = size.height;
                self.surface.configure(&self.gpu.device, &self.config);
            }
        }
        let _ = self.state.on_window_event(&self.window, event);
        if matches!(event, winit::event::WindowEvent::RedrawRequested) {
            return self.paint();
        }
        self.window.request_redraw();
        Ok(None)
    }

    fn paint(&mut self) -> Result<Option<PopupTerminal>, ProductEasingPopupError> {
        let input = self.state.take_egui_input(&self.window);
        let mut terminal = None;
        let output = self.context.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("Interval Easing");
                ui.horizontal(|ui| {
                    if ui.button("Linear").clicked() {
                        terminal = Some(PopupTerminal::Commit(Interp::Linear));
                    }
                    if ui.button("Smooth").clicked() {
                        terminal = Some(PopupTerminal::Commit(Interp::Bezier {
                            x1: 0.4,
                            y1: 0.0,
                            x2: 0.2,
                            y2: 1.0,
                        }));
                    }
                    if ui.button("Ease In").clicked() {
                        terminal = Some(PopupTerminal::Commit(Interp::Bezier {
                            x1: 0.42,
                            y1: 0.0,
                            x2: 1.0,
                            y2: 1.0,
                        }));
                    }
                    if ui.button("Ease Out").clicked() {
                        terminal = Some(PopupTerminal::Commit(Interp::Bezier {
                            x1: 0.0,
                            y1: 0.0,
                            x2: 0.58,
                            y2: 1.0,
                        }));
                    }
                });
                ui.separator();
                ui.label("Custom Bezier");
                for value in &mut self.curve {
                    ui.add(egui::Slider::new(value, 0.0..=1.0));
                }
                if ui.button("Apply custom").clicked() {
                    terminal = Some(PopupTerminal::Commit(Interp::Bezier {
                        x1: self.curve[0] as f64,
                        y1: self.curve[1] as f64,
                        x2: self.curve[2] as f64,
                        y2: self.curve[3] as f64,
                    }));
                }
                ui.add_enabled(false, egui::Button::new("Hold"));
                ui.add_enabled(false, egui::Button::new("Bounce / Elastic"));
            });
        });
        self.state
            .handle_platform_output(&self.window, output.platform_output);
        let paint_jobs = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        for (id, delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, delta);
        }
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(terminal)
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.gpu.device, &self.config);
                return Ok(terminal);
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(ProductEasingPopupError::SurfaceFrame)
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("motolii-product-easing-popup"),
            });
        let commands = self.renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &paint_jobs,
            &descriptor,
        );
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-product-easing-popup-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &descriptor);
        }
        self.gpu.queue.submit(
            commands
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        frame.present();
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        Ok(terminal)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProductEasingPopupError {
    #[error(transparent)]
    Window(#[from] winit::error::OsError),
    #[error(transparent)]
    Surface(#[from] wgpu::CreateSurfaceError),
    #[error("Easing popup surface is unsupported")]
    SurfaceUnsupported,
    #[error("Easing popup could not acquire its surface")]
    SurfaceFrame,
}
