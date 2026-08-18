//! 窓なしで `SpatialStage` を1フレーム回し、結果を texture に残す最小ハーネス。
//!
//! **出所**: `spikes/rerun-e0-composition-probe/src/harness.rs`(E0 probe)。
//! E0 からの差分は2つだけで、どちらも本 probe の測定対象に直結する。
//!
//! 1. **device を外から受け取れるようにした**。E0 は自前で adapter/device を作っていたが、
//!    ここでは iced のランタイムが作った device を渡す経路が要る。これが (a) の実体である。
//! 2. **検証層エラーで `exit(3)` しない**。E0 は落ちて良かったが、ここでは
//!    「iced の device では Rerun が動かない」こと自体が測定結果なので、文字列で回収する。
//!
//! 窓が要らない理由は E0 と同じ: Rerun は `ViewBuilder` を `egui_wgpu::Callback` へ入れて
//! `ui.painter()` に積むだけで、実際の `draw()`/`composite()` は `egui_wgpu::Renderer` が
//! 走らせる。その `Renderer` は surface ではなく任意の `wgpu::TextureView` を相手にできる。

use std::sync::{Arc, Mutex};

use re_renderer::RenderContext;
use re_view_spatial::SpatialStage;

/// 読み出しやすさを優先して gamma 空間の RGBA8 を出力先にする。
/// `RenderContext::new` の `output_format_color`、`egui_wgpu::Renderer` の
/// `output_color_format`、実際の render attachment の3つは一致していなければならない。
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// wgpu の texture→buffer コピーは行頭が 256 byte 境界に揃っている必要がある。
const COPY_ALIGN: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

/// 検証層のメッセージ置き場。`device.on_uncaptured_error` は `'static` を要求するので共有する。
pub type ErrorLog = Arc<Mutex<Vec<String>>>;

/// probe が比較する2つの device 記述。
///
/// `IcedWindowed` は iced master の `iced_wgpu::window::compositor` が実際に投げる
/// 記述の写しである(rev 3de4514 / `wgpu/src/window/compositor.rs` の 160-190 行)。
/// **iced はこの記述を差し替える公開の seam を持たない**ので、iced 上で Rerun を回すとは
/// 「この記述で作られた device で `re_renderer` が動くか」と同義になる。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// iced の窓つき compositor と同じ device 記述。
    IcedWindowed,
    /// `re_renderer` 自身が要求する device 記述(E0 が使っていたもの)。対照群。
    ReRenderer,
}

impl DeviceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::IcedWindowed => "iced-windowed",
            Self::ReRenderer => "re_renderer",
        }
    }
}

/// 所有権の形が2通りあるので束ねておく。iced から借りる場合、instance と adapter は
/// こちらで用意した「同じ物理 adapter を指す別ハンドル」である(後述 `borrow_iced_device`)。
pub struct Gpu {
    pub _instance: Option<wgpu::Instance>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    /// probe 自身が adapter/device を作る。`kind` で device 記述を選ぶ。
    pub fn create(kind: DeviceKind) -> Result<(Self, String), String> {
        let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter = re_renderer::device_caps::select_adapter(
            &adapters,
            wgpu::Backends::all(),
            None, // surface 無し = 窓無し
        )?;

        let descriptor = match kind {
            DeviceKind::IcedWindowed => iced_device_descriptor(&adapter),
            DeviceKind::ReRenderer => {
                re_renderer::device_caps::DeviceCaps::from_adapter(&adapter)
                    .map_err(|error| {
                        format!("adapter does not meet re_renderer requirements: {error}")
                    })?
                    .device_descriptor()
            }
        };
        let note = format!(
            "device kind: {}\n  requested max_bind_groups = {}\n  requested features = {:?}\n  adapter = {:?}",
            kind.label(),
            descriptor.required_limits.max_bind_groups,
            descriptor.required_features,
            adapter.get_info(),
        );

        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .map_err(|error| format!("request device ({}): {error}", kind.label()))?;

        Ok((
            Self {
                _instance: Some(instance),
                adapter,
                device,
                queue,
            },
            note,
        ))
    }

    /// iced のランタイムが作った device/queue をそのまま使う。
    ///
    /// **ここが (a) の要**。iced は `Primitive::prepare` / `Pipeline::new` に
    /// `&wgpu::Device` と `&wgpu::Queue` しか渡さず、**adapter は一切公開しない**
    /// (`iced_wgpu::window::Compositor` の `adapter` フィールドは private、
    ///  `Settings` にも limits/features/adapter の口は無い)。
    /// 一方 `RenderContext::new` は adapter を要求する。
    ///
    /// なので probe 側で instance を作り直して adapter を1つ選ぶ。
    /// `DeviceCaps::from_adapter` は adapter の**読み取りしかしない**ので、
    /// 同じ物理デバイスを指していれば結果は同じになる。同じ物か疑えるように
    /// adapter 情報を戻り値の note に残す(device が2つになった訳ではない —
    /// device と queue は iced のものが1つだけである)。
    pub fn borrow_iced_device(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(Self, String), String> {
        let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
        let adapters = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter =
            re_renderer::device_caps::select_adapter(&adapters, wgpu::Backends::all(), None)?;

        let limits = device.limits();
        let note = format!(
            "borrowed iced's own device (no second device was created)\n  \
             iced device max_bind_groups = {}\n  \
             iced device max_non_sampler_bindings = {}\n  \
             iced device max_texture_dimension_2d = {}\n  \
             iced device features = {:?}\n  \
             adapter re-enumerated for RenderContext::new = {:?}",
            limits.max_bind_groups,
            limits.max_non_sampler_bindings,
            limits.max_texture_dimension_2d,
            device.features(),
            adapter.get_info(),
        );

        Ok((
            Self {
                _instance: Some(instance),
                adapter,
                device: device.clone(),
                queue: queue.clone(),
            },
            note,
        ))
    }
}

/// 検証エラーを1か所で記録する。ログにも積み、通算カウンタも上げる。
///
/// カウンタが要るのは、入力ブリッジのレーンでは「このフレームは wgpu に
/// 蹴られたのか、それとも本当に絵が変わらなかったのか」を区別しないと
/// (d) の合否が言えないからである。
fn record_validation(text: String, errors: &ErrorLog) {
    eprintln!("WGPU VALIDATION: {text}");
    crate::embed::VALIDATION_TOTAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut slot) = crate::embed::LAST_VALIDATION.lock() {
        *slot = Some(text.clone());
    }
    if let Ok(mut log) = errors.lock() {
        log.push(text);
    }
}

/// error scope を閉じ、拾えた検証エラーを記録する。
fn drain_scope(
    device: &wgpu::Device,
    scope: wgpu::ErrorScopeGuard,
    what: &str,
    errors: &ErrorLog,
) {
    let pending = scope.pop();
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });
    if let Some(error) = pollster::block_on(pending) {
        record_validation(format!("{what}: {error}"), errors);
    }
}

/// iced master(rev 3de4514)の `iced_wgpu::window::compositor` が投げる device 記述の写し。
///
/// 元は `limits` を2段(default → downlevel)試すループだが、default が通る環境では
/// 1段目で決まる。`max_bind_groups: 2` と `max_non_sampler_bindings: 2048` の上書きが要点。
fn iced_device_descriptor(adapter: &wgpu::Adapter) -> wgpu::DeviceDescriptor<'static> {
    let limits = wgpu::Limits {
        max_bind_groups: 2,
        max_non_sampler_bindings: 2048,
        ..wgpu::Limits::default().using_resolution(adapter.limits())
    };
    let required_features = if adapter.features().contains(wgpu::Features::SHADER_F16) {
        wgpu::Features::SHADER_F16
    } else {
        wgpu::Features::empty()
    };
    wgpu::DeviceDescriptor {
        label: Some("iced_wgpu::window::compositor device descriptor (probe copy)"),
        required_features,
        required_limits: limits,
        memory_hints: wgpu::MemoryHints::MemoryUsage,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }
}

pub struct Offscreen {
    gpu: Gpu,
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
    errors: ErrorLog,
    /// 直近フレームの `platform_output.cursor_icon`。(e)。
    last_cursor_icon: egui::CursorIcon,
    /// 直近フレームの root viewport の `repaint_delay`。(f)。
    last_repaint_delay: Option<std::time::Duration>,
}

impl Offscreen {
    pub fn new(gpu: Gpu, width: u32, height: u32) -> Result<Self, String> {
        let errors: ErrorLog = Arc::new(Mutex::new(Vec::new()));

        // 検証層のメッセージは既定でどこにも出ない。E0 は落としていたが、ここでは
        // 「落ちた」こと自体が答なので回収する。
        //
        // なお iced の device に対して呼ぶと iced 側のハンドラを置き換えることになる
        // (すぐ後に `RenderContext::new` が更に Rerun の物で上書きする)。
        // 取りこぼしを防ぐ本命は下の error scope の方である。
        {
            let sink = errors.clone();
            gpu.device.on_uncaptured_error(Arc::new(move |error| {
                record_validation(format!("{error}"), &sink);
            }));
        }

        let scope = gpu.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let render_ctx = RenderContext::new(
            &gpu.adapter,
            gpu.device.clone(),
            gpu.queue.clone(),
            TARGET_FORMAT,
            re_renderer::RenderConfig::best_for_device_caps,
        )
        .map_err(|error| format!("create Rerun render context: {error}"))?;
        drain_scope(&gpu.device, scope, "RenderContext::new", &errors);

        let renderer = egui_wgpu::Renderer::new(
            &gpu.device,
            TARGET_FORMAT,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                ..Default::default()
            },
        );

        let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rerun stage offscreen target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            // `TEXTURE_BINDING` が E0 との差。iced の shader widget から sample するため。
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let padded_bytes_per_row = (width * 4).div_ceil(COPY_ALIGN) * COPY_ALIGN;
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rerun stage offscreen readback"),
            size: u64::from(padded_bytes_per_row) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let egui_ctx = egui::Context::default();
        // 1 point = 1 pixel。期待座標がそのまま pixel 座標になる。
        egui_ctx.set_pixels_per_point(1.0);

        Ok(Self {
            gpu,
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
            errors,
            last_cursor_icon: egui::CursorIcon::Default,
            last_repaint_delay: None,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn ui_rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(self.width as f32, self.height as f32),
        )
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.gpu.queue
    }

    /// iced の shader widget が sample する先。
    pub fn target_view(&self) -> &wgpu::TextureView {
        &self.target_view
    }

    pub fn render_ctx(&mut self) -> &mut RenderContext {
        self.render_ctx
            .as_mut()
            .expect("render context is only absent inside frame()")
    }

    pub fn take_errors(&self) -> Vec<String> {
        self.errors
            .lock()
            .map(|mut log| std::mem::take(&mut *log))
            .unwrap_or_default()
    }


    /// 1フレーム進める。egui の run → tessellate → egui_wgpu の render を
    /// offscreen texture に対して行う。窓もイベントループも使わない。
    ///
    /// 入力ゼロの frame。(a)(b)(c) の経路はこちらのまま。
    pub fn frame(&mut self, stage: &mut SpatialStage) -> Result<(), String> {
        self.frame_with_input(stage, Vec::new(), egui::Modifiers::NONE)
    }

    /// **入力つきの1フレーム**。`bridge` が翻訳した `egui::Event` 列をそのまま渡す。
    ///
    /// egui の1フレームは `RawInput` を丸ごと受け取ってから始まるので、
    /// 「iced が1フレームで配ったイベント」と「egui の1フレーム」はここで1:1に対応する。
    pub fn frame_with_input(
        &mut self,
        stage: &mut SpatialStage,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> Result<(), String> {
        // `on_uncaptured_error` は `RenderContext::new` が自分の物(Rerun の `ErrorTracker`)で
        // 上書きしてしまう。確実に拾うために error scope を使う。
        let scope = self
            .gpu
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let result = self.frame_inner(stage, events, modifiers);
        drain_scope(
            &self.gpu.device,
            scope,
            "SpatialStage frame",
            &self.errors,
        );
        result
    }

    /// 直近のフレームで egui が要求した cursor icon。(e) の観測点。
    pub fn last_cursor_icon(&self) -> egui::CursorIcon {
        self.last_cursor_icon
    }

    /// 直近のフレームで egui が要求した再描画までの待ち時間。(f) の観測点。
    pub fn last_repaint_delay(&self) -> Option<std::time::Duration> {
        self.last_repaint_delay
    }

    /// egui がこのフレームでポインタ入力を「使いたい」と言ったか。
    pub fn wants_pointer_input(&self) -> bool {
        self.egui_ctx.egui_wants_pointer_input()
    }

    fn frame_inner(
        &mut self,
        stage: &mut SpatialStage,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
    ) -> Result<(), String> {
        let mut render_ctx = self
            .render_ctx
            .take()
            .ok_or_else(|| "render context missing".to_owned())?;
        let egui_ctx = self.egui_ctx.clone();

        let raw_input = egui::RawInput {
            screen_rect: Some(self.ui_rect()),
            // 実時間ではなく固定 dt。2回実行が同じ絵になる。
            time: Some(f64::from(self.frame) / 60.0),
            predicted_dt: 1.0 / 60.0,
            max_texture_side: Some(8192),
            focused: true,
            modifiers,
            events,
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

        // 戻りの半分。egui が「このカーソルにしてほしい」「いつ再描画してほしい」と
        // 言った物を回収する。iced 側へ写せるかどうかは `bridge` が判定する。
        self.last_cursor_icon = full_output.platform_output.cursor_icon;
        self.last_repaint_delay = full_output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|viewport| viewport.repaint_delay);
        crate::bridge::observe_platform_output(self.last_cursor_icon, self.last_repaint_delay);

        let paint_jobs = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.width, self.height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, delta);
        }

        // paint callback(= Rerun の ViewBuilder)はここから RenderContext を引く。
        self.renderer.callback_resources.insert(render_ctx);

        let mut encoder =
            self.gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("rerun stage offscreen encoder"),
                });
        // `prepare` = `ViewBuilder::draw`。シーン本体はここで描かれる。
        let user_buffers = self.renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("rerun stage offscreen pass"),
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
        self.gpu
            .queue
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
        let mut encoder =
            self.gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("rerun stage readback encoder"),
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
        self.gpu.queue.submit(std::iter::once(encoder.finish()));

        let slice = self.readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        self.gpu
            .device
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
