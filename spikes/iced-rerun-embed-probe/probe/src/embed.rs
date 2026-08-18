//! **本 probe が新規に測る部分**: Rerun の `SpatialStage` が合成した frame を、
//! iced の `shader` widget から**同じ wgpu device のまま**画面へ出せるか。
//!
//! 経路はこうなる。
//!
//! ```text
//! iced runtime device ──┬─→ egui_wgpu::Renderer ─→ SpatialStage::show ─→ offscreen texture
//!                       └─→ blit pipeline ────────→ iced の render pass
//! ```
//!
//! device は1つしかない。offscreen texture は iced の device 上に作られ、iced の
//! render pass から直接 sample される。CPU への読み戻しも、texture の複製も挟まらない。
//!
//! ## thread_local を使う理由
//!
//! iced の `Pipeline` は `Any + Send + Sync` を要求する。`SpatialStage` と
//! `re_renderer::RenderContext` と `egui::Context` の束はその境界を跨げないので、
//! `unsafe impl Send` を書く代わりに thread_local に置く。iced の native runtime は
//! `prepare`/`draw` を同じスレッド(winit のイベントループ)から呼ぶので、これで足りる。
//! 別スレッドから来たら `prepare` が気付いて記録する。

use std::cell::RefCell;
use std::sync::Mutex;
use std::thread::ThreadId;

use iced::mouse;
use iced::widget::shader::{self, Viewport};
use iced::{Rectangle, Size};
use re_view_spatial::SpatialStage;

use crate::app::Message;
use crate::harness::{Gpu, Offscreen, TARGET_FORMAT};
use crate::stage;

/// probe の出力。GUI からも main からも読む。
pub static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn log(line: impl Into<String>) {
    let line = line.into();
    println!("[embed] {line}");
    if let Ok(mut log) = LOG.lock() {
        log.push(line);
    }
}

pub fn snapshot_log() -> Vec<String> {
    LOG.lock().map(|log| log.clone()).unwrap_or_default()
}

/// 「同一 device で成立したか」を1か所で持つ。
#[derive(Clone, Debug, Default)]
pub struct Status {
    pub initialized: bool,
    pub error: Option<String>,
    pub validation_errors: Vec<String>,
    pub frames: u64,
    pub offscreen_size: (u32, u32),
}

pub static STATUS: Mutex<Option<Status>> = Mutex::new(None);

pub fn status() -> Status {
    STATUS.lock().ok().and_then(|s| s.clone()).unwrap_or_default()
}

fn set_status(status: Status) {
    if let Ok(mut slot) = STATUS.lock() {
        *slot = Some(status);
    }
}

// ---------------------------------------------------------------------------
// thread_local に置く「Rerun 側一式」
// ---------------------------------------------------------------------------

struct Embed {
    offscreen: Offscreen,
    stage: SpatialStage,
    blit: Blit,
    applied_generation: u64,
    frames: u64,
    /// warmup 中に wgpu が報告した検証エラー。iced の device 制限に当たった記録。
    startup_validation: Vec<String>,
}

/// stage のカメラを誰が握るか。
///
/// - `Document` は元 probe の (c) と同じ「iced 側が `set_camera` で置く」形。
///   置かれている間、Rerun 自身の入力処理は `EyeState::update` の頭で短絡される。
/// - `Free` は `clear_camera` した状態で、カメラは Rerun のブループリント既定 +
///   Rerun 自身の入力処理が決める。**(d) の orbit を測れるのはこちらだけ**である。
#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    Document { pull_back: f32 },
    Free,
}

/// 直近フレームの eye(位置と前方)。(d) を画素だけでなく数値でも言うため。
pub static EYE: Mutex<Option<[f32; 6]>> = Mutex::new(None);

pub fn eye() -> Option<[f32; 6]> {
    EYE.lock().ok().and_then(|slot| *slot)
}

/// (e)(f) の観測値を1フレーム分まとめて外へ出す。
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code, reason = "走行ログに Debug で丸ごと出す欄がある")]
pub struct FrameObservation {
    pub cursor_icon_is_default: bool,
    pub wants_pointer_input: bool,
    pub repaint_delay_ms: Option<u128>,
}

pub static OBSERVATION: Mutex<Option<FrameObservation>> = Mutex::new(None);

pub fn observation() -> FrameObservation {
    OBSERVATION
        .lock()
        .ok()
        .and_then(|slot| *slot)
        .unwrap_or_default()
}

/// wgpu が文句を言った回数の通算。フレーム間の差が「このフレームは落ちた」の印。
pub static VALIDATION_TOTAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// 直近の検証エラー本文。同じ物が毎フレーム出るので1本だけ持つ。
pub static LAST_VALIDATION: Mutex<Option<String>> = Mutex::new(None);

pub fn validation_total() -> u64 {
    VALIDATION_TOTAL.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn last_validation() -> Option<String> {
    LAST_VALIDATION.lock().ok().and_then(|slot| slot.clone())
}

thread_local! {
    static EMBED: RefCell<Option<Embed>> = const { RefCell::new(None) };
    /// 初期化に失敗したらもう試さない。毎フレーム同じ検証エラーを吐かせない。
    static FAILED: RefCell<bool> = const { RefCell::new(false) };
}

/// offscreen texture を iced の render pass へ写すだけの最小パイプライン。
///
/// bind group は1つ(texture + sampler)。iced の device は `max_bind_groups = 2` なので、
/// ここで2つ以上使うと iced 側の都合で落ちる。1つに収めてある。
struct Blit {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

const BLIT_WGSL: &str = r#"
@group(0) @binding(0) var stage_texture: texture_2d<f32>;
@group(0) @binding(1) var stage_sampler: sampler;

struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// 頂点バッファ無しの全画面三角形。render pass の viewport は iced が
// widget の bounds に合わせてくれているので、NDC 全体を覆えばそれで足りる。
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
    var out: VsOut;
    let x = f32((index << 1u) & 2u);
    let y = f32(index & 2u);
    out.uv = vec2<f32>(x, y);
    out.position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(stage_texture, stage_sampler, in.uv);
}
"#;

impl Blit {
    fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        source: &wgpu::TextureView,
    ) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("stage blit"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("stage blit bind group layout"),
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("stage blit sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("stage blit bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("stage blit pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("stage blit pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
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

        Self {
            pipeline,
            bind_group,
        }
    }
}

// ---------------------------------------------------------------------------
// iced 側の型
// ---------------------------------------------------------------------------

/// `Pipeline::new` は iced のランタイム device を渡してくる唯一の場所。
/// ここで受け取った device が、そのまま Rerun の device になる。
pub struct StagePipeline {
    format: wgpu::TextureFormat,
    owner: ThreadId,
}

impl shader::Pipeline for StagePipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let limits = device.limits();
        log(format!(
            "iced handed us its runtime device: max_bind_groups={}, \
             max_non_sampler_bindings={}, max_texture_dimension_2d={}, \
             features={:?}, surface format={format:?}",
            limits.max_bind_groups,
            limits.max_non_sampler_bindings,
            limits.max_texture_dimension_2d,
            device.features(),
        ));
        let _ = queue;
        Self {
            format,
            owner: std::thread::current().id(),
        }
    }
}

/// `Primitive` は `Debug + Send + Sync` を要求するので、素のデータしか持てない。
/// GPU 資源は `StagePipeline` と thread_local が持つ。
#[derive(Debug, Clone, Copy)]
pub struct StagePrimitive {
    pub camera: CameraMode,
    pub generation: u64,
}

impl shader::Primitive for StagePrimitive {
    type Pipeline = StagePipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        if pipeline.owner != std::thread::current().id() {
            log("prepare() ran on a different thread than Pipeline::new() — bailing out");
            return;
        }
        if FAILED.with(|failed| *failed.borrow()) {
            return;
        }

        let scale = viewport.scale_factor();
        let size = Size::new(
            ((bounds.width * scale).round() as u32).max(1),
            ((bounds.height * scale).round() as u32).max(1),
        );

        let result = EMBED.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot
                .as_ref()
                .is_some_and(|embed| embed.offscreen.size() != (size.width, size.height))
            {
                log(format!(
                    "widget size changed to {}x{} — rebuilding the stage",
                    size.width, size.height
                ));
                *slot = None;
            }
            if slot.is_none() {
                *slot = Some(build_embed(
                    device,
                    queue,
                    pipeline.format,
                    size.width,
                    size.height,
                )?);
            }
            let embed = slot.as_mut().expect("just built");

            if embed.applied_generation != self.generation {
                match self.camera {
                    CameraMode::Document { pull_back } => {
                        embed.stage.set_camera(stage::document_camera(pull_back));
                        log(format!(
                            "applied camera generation {} (document, pull_back = {pull_back:.3})",
                            self.generation
                        ));
                    }
                    CameraMode::Free => {
                        embed.stage.clear_camera();
                        log(format!(
                            "applied camera generation {} (free — Rerun's own camera state is \
                             back in charge)",
                            self.generation
                        ));
                    }
                }
                embed.applied_generation = self.generation;
                for _ in 0..stage::SETTLE_FRAMES {
                    embed.offscreen.frame(&mut embed.stage)?;
                    embed.frames += 1;
                }
            }

            // **入力ブリッジの出口**。この frame までに iced が配ったイベントを
            // egui の RawInput へ載せて、`SpatialStage::show` を1回だけ回す。
            let (events, modifiers) = crate::bridge::drain(scale);
            embed
                .offscreen
                .frame_with_input(&mut embed.stage, events, modifiers)?;
            embed.frames += 1;

            // (d) の数値側。Rerun 自身が持っている eye を読み戻す。
            if let Ok(mut slot) = EYE.lock() {
                *slot = embed.stage.last_eye().map(|eye| {
                    let position = eye.pos_in_world();
                    let forward = eye.forward_in_world();
                    [
                        position.x, position.y, position.z, forward.x, forward.y, forward.z,
                    ]
                });
            }
            if let Ok(mut slot) = OBSERVATION.lock() {
                *slot = Some(FrameObservation {
                    cursor_icon_is_default: embed.offscreen.last_cursor_icon()
                        == egui::CursorIcon::Default,
                    wants_pointer_input: embed.offscreen.wants_pointer_input(),
                    repaint_delay_ms: embed
                        .offscreen
                        .last_repaint_delay()
                        .map(|delay| delay.as_millis()),
                });
            }

            let mut validation_errors = embed.startup_validation.clone();
            validation_errors.extend(embed.offscreen.take_errors());
            Ok::<_, String>(Status {
                initialized: true,
                error: None,
                validation_errors,
                frames: embed.frames,
                offscreen_size: embed.offscreen.size(),
            })
        });

        match result {
            Ok(status) => set_status(status),
            Err(error) => {
                log(format!("stage embed failed: {error}"));
                FAILED.with(|failed| *failed.borrow_mut() = true);
                set_status(Status {
                    initialized: false,
                    error: Some(error),
                    ..Default::default()
                });
            }
        }
    }

    fn draw(&self, _pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        EMBED.with(|cell| {
            let borrow = cell.borrow();
            let Some(embed) = borrow.as_ref() else {
                return false;
            };
            render_pass.set_pipeline(&embed.blit.pipeline);
            render_pass.set_bind_group(0, &embed.blit.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            true
        })
    }
}

fn build_embed(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> Result<Embed, String> {
    let (gpu, note) = Gpu::borrow_iced_device(device, queue)?;
    log(note);

    let mut offscreen = Offscreen::new(gpu, width, height)?;
    log(format!(
        "built a Rerun RenderContext on iced's device; offscreen target {width}x{height} {TARGET_FORMAT:?}"
    ));

    let mut spatial = stage::new_stage("iced-rerun-embed")?;
    stage::install_quadrants(&mut offscreen, &mut spatial)?;
    for _ in 0..stage::WARMUP_FRAMES {
        offscreen.frame(&mut spatial)?;
    }
    // 検証エラーが出ても止めない。preflight で分かっている通り、iced の
    // `max_bind_groups = 2` に当たるのは `LineRenderer` だけで、矩形の経路は通る。
    // 「絵は出るが一部の renderer は作れない」という状態そのものを記録して先へ進む。
    let errors = offscreen.take_errors();
    if errors.is_empty() {
        log(format!(
            "warmed up {} frames of SpatialStage on iced's device without validation errors",
            stage::WARMUP_FRAMES
        ));
    } else {
        log(format!(
            "warmed up {} frames of SpatialStage on iced's device, but wgpu reported:\n  {}",
            stage::WARMUP_FRAMES,
            errors.join("\n  ")
        ));
    }

    let blit = Blit::new(device, format, offscreen.target_view());

    Ok(Embed {
        offscreen,
        stage: spatial,
        blit,
        // 0 は「まだ何も適用していない」。最初の prepare で generation 1 が入る。
        applied_generation: u64::MAX,
        frames: u64::from(stage::WARMUP_FRAMES),
        startup_validation: errors,
    })
}

/// offscreen の中身をそのまま読み出す。iced の screenshot が窓全体なのに対し、
/// こちらは「Rerun が描いた物そのもの」で、E0 の絵と直接比べられる。
pub fn read_offscreen() -> Option<(Vec<u8>, u32, u32)> {
    EMBED.with(|cell| {
        let borrow = cell.borrow();
        let embed = borrow.as_ref()?;
        let (width, height) = embed.offscreen.size();
        embed
            .offscreen
            .read_rgba()
            .ok()
            .map(|rgba| (rgba, width, height))
    })
}

// ---------------------------------------------------------------------------
// Program: 入力 → 型付き Message
// ---------------------------------------------------------------------------

pub struct StageProgram {
    pub pull_back: f32,
    pub generation: u64,
}

// ---------------------------------------------------------------------------
// BridgeProgram: 入力 → egui の RawInput(本レーンが新しく測る経路)
// ---------------------------------------------------------------------------

/// **(d)(e)(f) の要**。`StageProgram` が click を1つだけ型付き Message にしていたのに対し、
/// こちらは mouse move / press / release / wheel を丸ごと `egui::RawInput` へ渡す。
/// カメラを動かすのは `set_camera` ではなく `SpatialStage` 自身の orbit である。
pub struct BridgeProgram {
    pub camera: CameraMode,
    pub generation: u64,
}

impl<Message> shader::Program<Message> for BridgeProgram {
    type State = ();
    type Primitive = StagePrimitive;

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        let pending = crate::bridge::translate(event, bounds, cursor);
        if pending.is_empty() {
            return None;
        }
        crate::bridge::push(pending);
        // Message は publish しない。カメラを動かすのは stage 自身であって
        // アプリではないので、アプリへ渡す物が無い。次のフレームで絵が変わる
        // ことだけが要るので、再描画だけ頼む。
        //
        // `and_capture` はしない。この widget は窓いっぱいで他に競合が無く、
        // 捕まえると (f) の「フォーカスの奪い合い」を自分で潰してしまう。
        Some(iced::widget::Action::request_redraw())
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        StagePrimitive {
            camera: self.camera,
            generation: self.generation,
        }
    }

    /// **(e)**: egui が要求した cursor icon を iced の `Interaction` へ写す。
    /// 写せない icon(`VerticalText`)だけは iced の既定へ落ちる。
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if !cursor.is_over(bounds) {
            return mouse::Interaction::default();
        }
        crate::bridge::to_iced_interaction(crate::bridge::cursor_icon())
            .unwrap_or(mouse::Interaction::default())
    }
}

impl shader::Program<Message> for StageProgram {
    type State = ();
    type Primitive = StagePrimitive;

    /// **(c) の要**。shader widget の上での click が、renderer の都合を一切含まない
    /// 型付き Message になって `App::update` へ届く。座標は widget 相対の論理座標で、
    /// これがそのままカメラ操作の意味になる。
    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = cursor.position_in(bounds)?;
                Some(
                    iced::widget::Action::publish(Message::StageClicked {
                        x: position.x,
                        y: position.y,
                    })
                    .and_capture(),
                )
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        StagePrimitive {
            camera: CameraMode::Document {
                pull_back: self.pull_back,
            },
            generation: self.generation,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}
