//! 窓なし(OS window 無し)で `SpatialStage` を1フレーム回し、結果の pixel を読み出す最小ハーネス。
//!
//! 2026-08-11裁定の拘束を守る: **第二の re_renderer scene も第二 runtime も作らない**。
//! ここが作るのは wgpu device と egui の driver だけで、シーンの構築・カメラ・描画は
//! すべて `SpatialStage::show` → `SpatialView3D` → `ViewBuilder` という製品と同じ経路を通る。
//!
//! 窓が要らない理由は `re_viewer_context::gpu_bridge::re_renderer_callback` を読むと分かる:
//! Rerun は `ViewBuilder` を `egui_wgpu::Callback` へ入れて `ui.painter()` に積むだけで、
//! 実際の `draw()`/`composite()` は `egui_wgpu::Renderer` が走らせる。その `Renderer` は
//! surface ではなく任意の `wgpu::TextureView` を相手にできる。

use std::sync::Arc;

use re_renderer::RenderContext;
use re_view_spatial::SpatialStage;

/// 読み出しやすさを優先して gamma 空間の RGBA8 を出力先にする。
/// `RenderContext::new` に渡す `output_format_color` と `egui_wgpu::Renderer` の
/// `output_color_format` と、実際の render attachment の3つは一致していなければならない。
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// wgpu の texture→buffer コピーは行頭が 256 byte 境界に揃っている必要がある。
const COPY_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub struct Offscreen {
    // adapter/instance は device より長生きさせる必要があるので抱えておく。
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    egui_ctx: egui::Context,
    renderer: egui_wgpu::Renderer,
    /// `stage.show` が使う間だけ取り出し、描画中は `renderer.callback_resources` に置く。
    render_ctx: Option<RenderContext>,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    frame: u32,
}

impl Offscreen {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter = re_renderer::device_caps::select_adapter(
            &adapters,
            wgpu::Backends::all(),
            None, // surface 無し = 窓無し。ここが (a) の主張の実体である。
        )?;
        let caps = re_renderer::device_caps::DeviceCaps::from_adapter(&adapter)
            .map_err(|error| format!("adapter does not meet re_renderer requirements: {error}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&caps.device_descriptor()))
            .map_err(|error| format!("request headless device: {error}"))?;

        // 検証層のメッセージは既定でどこにも出ない。落ちるべきものを黙って落とさせない。
        device.on_uncaptured_error(Arc::new(|error| {
            eprintln!("WGPU VALIDATION: {error}");
            std::process::exit(3);
        }));

        let render_ctx = RenderContext::new(
            &adapter,
            device.clone(),
            queue.clone(),
            TARGET_FORMAT,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .map_err(|error| format!("create Rerun render context: {error}"))?;

        let renderer = egui_wgpu::Renderer::new(
            &device,
            TARGET_FORMAT,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                // 決定性を最優先する。dithering は乱数ではないが、比較対象を減らしておく。
                dithering: false,
                ..Default::default()
            },
        );

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("e0 offscreen target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let padded_bytes_per_row = (width * 4).div_ceil(COPY_ALIGN) * COPY_ALIGN;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("e0 offscreen readback"),
            size: u64::from(padded_bytes_per_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let egui_ctx = egui::Context::default();
        // 1 point = 1 pixel。`Eye::ui_from_world` は point 空間を返すので、
        // ここを 1.0 にしておくと (b) の期待座標がそのまま pixel 座標になる。
        egui_ctx.set_pixels_per_point(1.0);

        Ok(Self {
            _instance: instance,
            _adapter: adapter,
            device,
            queue,
            egui_ctx,
            renderer,
            render_ctx: Some(render_ctx),
            target,
            target_view,
            readback,
            width,
            height,
            padded_bytes_per_row,
            frame: 0,
        })
    }

    /// `stage.show` が実際に受け取る ui 矩形。`Context::run_ui` が渡す root ui は
    /// viewport 全体を `max_rect` に持ち、panel の margin が挟まらないので画面全体と一致する。
    /// (b) の投影はこの矩形を基準にする。
    pub fn ui_rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(self.width as f32, self.height as f32),
        )
    }

    /// GPU texture を `copy_gpu_image` へ渡す前に用意するための device/queue。
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// `SpatialStage::copy_gpu_image` は `&RenderContext` を要求する。
    pub fn render_ctx(&mut self) -> &mut RenderContext {
        self.render_ctx
            .as_mut()
            .expect("render context is only absent inside frame()")
    }

    /// 1フレーム進める。egui の run → tessellate → egui_wgpu の render を
    /// offscreen texture に対して行う。窓もイベントループも使わない。
    pub fn frame(&mut self, stage: &mut SpatialStage) -> Result<(), String> {
        let mut render_ctx = self
            .render_ctx
            .take()
            .ok_or_else(|| "render context missing".to_owned())?;
        let egui_ctx = self.egui_ctx.clone();

        let raw_input = egui::RawInput {
            screen_rect: Some(self.ui_rect()),
            // 実時間ではなく固定 dt を使う。カメラの補間もアニメーションも
            // フレーム番号だけの関数になるので、2回実行が同じ絵になる。
            time: Some(f64::from(self.frame) / 60.0),
            predicted_dt: 1.0 / 60.0,
            max_texture_side: Some(8192),
            focused: true,
            ..Default::default()
        };

        let mut stage_error = None;
        let full_output = egui_ctx.run_ui(raw_input, |ui| {
            if let Err(error) = stage.show(ui, &mut render_ctx) {
                stage_error = Some(error.to_string());
            }
        });
        if let Some(error) = stage_error {
            self.render_ctx = Some(render_ctx);
            return Err(format!("SpatialStage::show: {error}"));
        }

        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.width, self.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.device, &self.queue, *id, delta);
        }

        // paint callback(= Rerun の ViewBuilder)はここから RenderContext を引く。
        self.renderer.callback_resources.insert(render_ctx);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("e0 offscreen encoder"),
            });
        // `prepare` = `ViewBuilder::draw`。シーン本体はここで描かれる。
        let user_buffers =
            self.renderer
                .update_buffers(&self.device, &self.queue, &mut encoder, &paint_jobs, &screen_descriptor);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("e0 offscreen pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.target_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            // `paint` = `ViewBuilder::composite`。ここで offscreen texture へ出る。
            self.renderer
                .render(&mut pass, &paint_jobs, &screen_descriptor);
        }
        self.queue
            .submit(user_buffers.into_iter().chain(std::iter::once(encoder.finish())));

        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        self.render_ctx = self.renderer.callback_resources.remove::<RenderContext>();
        if self.render_ctx.is_none() {
            return Err("render context vanished from callback resources".to_owned());
        }
        self.frame += 1;
        Ok(())
    }

    /// 直近に描いた offscreen texture を RGBA8 として読み出す。
    pub fn read_rgba(&self) -> Result<Vec<u8>, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("e0 readback encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = self.readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|error| format!("poll for readback: {error}"))?;
        receiver
            .recv()
            .map_err(|error| format!("readback channel: {error}"))?
            .map_err(|error| format!("map readback buffer: {error}"))?;

        let mapped = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let start = (row * self.padded_bytes_per_row) as usize;
            rgba.extend_from_slice(&mapped[start..start + (self.width * 4) as usize]);
        }
        drop(mapped);
        self.readback.unmap();
        Ok(rgba)
    }
}
