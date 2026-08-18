//! Stage 島 — Rerun `SpatialStage` の合成絵を iced の shader widget から出す。
//!
//! `spikes/iced-rerun-embed-probe` の製品 adapter 化(M-2)。GPU の実体は
//! `motolii_ui::rerun_stage::EmbeddedSpatialStage`(RN native surface と同じ
//! toolkit 中立の口)で、この module は iced 側の受け皿だけを持つ:
//!
//! ```text
//! iced::Event ─ stage_bridge::translate ─ stage_arbiter::route ─┐(素通し/orbit中だけ)
//!                                                               ▼
//! iced runtime device ─→ EmbeddedSpatialStage::render ─→ offscreen texture
//!                     └→ blit ─→ iced の render pass(この widget の bounds)
//! ```
//!
//! egui はこの crate に**入っていない**。`EmbeddedSpatialStage` が自前 egui を
//! 1周だけ回す(`render()` 経路)— 「egui は Stage 島の内側の実装詳細に縮む」
//! (2026-08-18 ホスト移行裁定)の実装がこの形である。カメラも同じ理由で
//! こちらには無い: 正対既定(document camera)は `EmbeddedSpatialStage` の
//! 構築時に置かれ、ここは composition の縦横比と面の大きさを**伝えるだけ**
//! (egui shell の pane と同じ分担)。
//!
//! ## thread_local を使う理由(spike の詰まった箇所その4)
//!
//! iced の `Pipeline` は `Any + Send + Sync` を要求する。`EmbeddedSpatialStage`
//! (egui context を内蔵)はその境界を跨げないので、`unsafe impl Send` を書く
//! 代わりに thread_local に置く。iced の native runtime は `prepare`/`draw` を
//! 同じ thread(winit のイベントループ)から呼ぶのでこれで足りる。
//!
//! ## 失敗は黙らない
//!
//! Stage の初期化・texture import・描画・wgpu 検証層の失敗は
//! [`Message::StageReported`] になって `Shell::update` が transcript
//! (帯 / `--status-log`)へ写す。溜め場はこの module の mailbox で、
//! `RedrawRequested`(毎フレーム widget 木へ来る)が吐き出しの合図。
//! 同じ一言は process につき1回だけ言う(毎フレーム同じ失敗で帯を埋めない —
//! egui pane の latch と同じ判断)。
//!
//! ## bind group の床(fork seam 2)
//!
//! iced fork の `iced_wgpu::device_limits` は device 要求時の `max_bind_groups` の
//! **床**を公開している([iced fork seam 台帳]
//! (../../../docs/reviews/2026-08-18-iced-fork-seam-ledger.md) §3)。
//! [`install_rerun_device_floor`] を**窓 / headless renderer を建てる前に**呼ぶ。
//! 効いていることの常設 oracle は `tests/stage_bind_groups_oracle.rs` — 台帳 §4 が
//! 「M-2 の受け入れ条件に含める」とした穴を塞ぐ物である。
//!
//! ## 評価済みフレームの席(M-3)
//!
//! M-2 は `install_probe_frame` で CPU から差した既知絵だけを持っていた
//! (`presented_frame` / `frame_texture`)。M-3 はその隣に
//! `motolii_ui::stage_frame_seat::StageFrameSeat` を立て、**live Document が
//! 座っている間はそちらを使う** — export と同じ評価
//! (`build_document_frame_graph` + `render_graph_cached`)を経た、いまの
//! playhead 時刻の合成フレームが `EmbeddedSpatialStage::render` の
//! `evaluated_frame` へ渡る。probe 経路は消していない: `document: None` の
//! fixture 展示(および `stage_island_pixels.rs` の pixel oracle)はそのまま
//! `present_probe_frame` を通る。
//!
//! 非同期の完成を拾う repaint の合図は、egui 版の `on_frame_ready`
//! コールバックではなく `main.rs::Host::subscription` の `window::frames()`
//! 購読(live Document が座っている間だけ)に任せている — 詳しくは
//! [`Embed::drive_frame_seat`] の doc を見る。

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use iced::mouse;
use iced::widget::shader::{self, Viewport};
use iced::{Element, Fill, Rectangle};

use motolii_doc::Document;
use motolii_ui::rerun_stage::{EmbeddedSpatialStage, PointerPhase, StagePointerButton};
use motolii_ui::stage_frame_seat::{StageFrameEvidence, StageFrameSeat};

use crate::message::Message;
use crate::shell::Shell;
use crate::stage_arbiter::{route, GrabRegion, Route, StagePointerOwner};
use crate::stage_bridge::{translate, StageInput};

/// Rerun(`re_renderer`)が device に要求する bind group 数。
///
/// 出所は `re_renderer` の device 記述(`downlevel_webgl2_defaults` 由来の 4)。
/// 定数で持つのは、窓を建てる**前**(adapter がまだ無い時点)に床を上げる必要が
/// あるから。値が `re_renderer` の実要求を下回っていないことは
/// `tests/stage_bind_groups_oracle.rs` が adapter 実物で照合する。
pub const RERUN_MIN_MAX_BIND_GROUPS: u32 = 4;

/// fork seam 2 の床を上げる。**窓 / headless renderer を建てる前に**呼ぶ
/// (既に在る device には効かない)。何度呼んでも下がらない(床は上がる一方)。
pub fn install_rerun_device_floor() {
    iced_wgpu::device_limits::request_min_max_bind_groups(RERUN_MIN_MAX_BIND_GROUPS);
}

/// Stage 島の widget。座席が在る間の中央 pane。
///
/// 製品は掴み領域を持たない(`grab_probe: None` — ギズモ本体は M-2 後)。
pub fn stage_island(shell: &Shell) -> Element<'_, Message> {
    iced::widget::shader(StageIsland {
        composition_aspect: shell.composition_aspect(),
        // live 座席の Document + playhead(M-3)。座席が無ければ `None` —
        // 従来どおり幾何だけの fixture 展示のまま(egui shell の
        // `wants_evaluated_frame` と同じ分担)。
        document: shell.document(),
        playhead: shell.timeline_playhead(),
        grab_probe: None,
    })
    .width(Fill)
    .height(Fill)
    .into()
}

/// shader widget の `Program`。製品は `grab_probe = None`(掴みは発生しない)。
/// テストはダミー領域を差して3状態を審判する。
///
/// `Copy` は持てない — `document` が `Arc<Document>` を運ぶ(M-3 の stage frame
/// seat 結線)。
#[derive(Debug, Clone)]
pub struct StageIsland {
    /// composition の縦横比(幅/高さ)。座席の Document から。
    pub composition_aspect: Option<f32>,
    /// live 座席の Document snapshot。`None` なら fixture 展示のまま
    /// (`present_probe_frame` 経路。pixel oracle が使う)。
    pub document: Option<Arc<Document>>,
    /// `document` を評価する playhead(秒)。`document` が `None` の間は読まれない。
    pub playhead: f32,
    /// 掴み判定の seam(M-2 はテスト専用のダミー。ギズモ本体は M-2 後)。
    pub grab_probe: Option<GrabRegion>,
}

/// `Primitive` は `Debug + Send + Sync` を要求するので素のデータしか持てない。
/// GPU 資源は thread_local(`EMBED`)が持つ。`Document` は render worker が
/// 別 thread へ渡す物と同じ型で、既に `Send + Sync`(`Arc<Document>` を跨いで
/// 使い回している)。
#[derive(Debug, Clone)]
pub struct StagePrimitive {
    composition_aspect: Option<f32>,
    document: Option<Arc<Document>>,
    playhead: f32,
}

/// `Pipeline::new` は iced のランタイム device を渡してくる唯一の場所。
///
/// device 1つにつき1回呼ばれるので、ここで発行する `token` が「どの device の
/// 上に居るか」の名札になる。`prepare` は名札が変わったら GPU 一式を建て直す。
pub struct StagePipeline {
    token: u64,
    format: wgpu::TextureFormat,
    owner: std::thread::ThreadId,
}

impl shader::Pipeline for StagePipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let _ = queue;
        // fork seam 2 の実効 oracle の観測点: iced が実際に建てた device が
        // 取得できた bind group 数。「要求した」ではなく「取得できた」を記録する。
        if let Ok(mut seen) = OBSERVED_MAX_BIND_GROUPS.lock() {
            seen.push(device.limits().max_bind_groups);
        }
        Self {
            token: PIPELINE_TOKENS.fetch_add(1, Ordering::Relaxed) + 1,
            format,
            owner: std::thread::current().id(),
        }
    }
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
            // iced の native runtime では起きない(spike の実測)。起きたら黙らず言う。
            report_once("stage: prepare が Pipeline と別 thread で呼ばれた(絵を止める)");
            return;
        }
        if FAILED_TOKEN.with(|failed| *failed.borrow() == Some(pipeline.token)) {
            // この device では一度建て損ねている。毎フレーム同じ失敗を繰り返さない。
            return;
        }

        let scale = viewport.scale_factor();
        let width = ((bounds.width * scale).round() as u32).max(1);
        let height = ((bounds.height * scale).round() as u32).max(1);

        let result = EMBED.with(|cell| {
            let mut slot = cell.borrow_mut();

            // device が載せ替わっていたら(= 別の Pipeline 世代)、古い GPU 一式は捨てる。
            if slot
                .as_ref()
                .is_some_and(|embed| embed.token != pipeline.token)
            {
                *slot = None;
            }
            if slot.is_none() {
                *slot = Some(Embed::build(pipeline.token, device, queue)?);
            }
            let embed = slot.as_mut().expect("just built");

            embed.frame(
                device,
                queue,
                pipeline.format,
                width,
                height,
                scale,
                self.composition_aspect,
                self.document.as_ref(),
                self.playhead,
            )
        });

        if let Err(error) = result {
            FAILED_TOKEN.with(|failed| *failed.borrow_mut() = Some(pipeline.token));
            report_once(format!("stage: Stage 島を建てられない: {error}"));
        }
    }

    fn draw(&self, _pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        EMBED.with(|cell| {
            let borrow = cell.borrow();
            let Some(embed) = borrow.as_ref() else {
                return false;
            };
            let Some(blit) = embed.blit.as_ref() else {
                return false;
            };
            render_pass.set_pipeline(&blit.pipeline);
            render_pass.set_bind_group(0, &blit.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            true
        })
    }
}

impl shader::Program<Message> for StageIsland {
    type State = StagePointerOwner;
    type Primitive = StagePrimitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        // 失敗報告の吐き出し。RedrawRequested は毎フレーム widget 木へ来るので、
        // mailbox が空でない限り次のフレームには帯へ届く。
        if matches!(
            event,
            iced::Event::Window(iced::window::Event::RedrawRequested(_))
        ) {
            let lines = take_reports();
            if !lines.is_empty() {
                return Some(iced::widget::Action::publish(Message::StageReported(lines)));
            }
            return None;
        }

        let translated = translate(event, bounds, cursor);
        if translated.is_empty() {
            return None;
        }
        for input in translated {
            match route(state, &input, self.grab_probe.as_ref()) {
                Route::Forward => {
                    FORWARDED.fetch_add(1, Ordering::Relaxed);
                    push_input(input);
                }
                Route::Swallow => {
                    // 掴み中 — Rerun へは流さない。M-2 ではここで消えるだけ
                    // (ギズモの意味論は M-2 後にこの席へ座る)。
                    SWALLOWED.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        // Message は publish しない。絵を進めるのは stage 自身なので、
        // 次のフレームで prepare が回ることだけを頼む。
        Some(iced::widget::Action::request_redraw())
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        StagePrimitive {
            composition_aspect: self.composition_aspect,
            document: self.document.clone(),
            playhead: self.playhead,
        }
    }

    /// 「掴んでいる感」は埋め込み側が出す(spike の実測 13 番: Rerun は orbit 中に
    /// `Grab`/`Grabbing` を要求しない)。
    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match state {
            StagePointerOwner::Grabbing { .. } | StagePointerOwner::Orbiting { .. } => {
                mouse::Interaction::Grabbing
            }
            StagePointerOwner::PassThrough => {
                let hovering_grab = cursor.position_in(bounds).is_some_and(|position| {
                    self.grab_probe
                        .is_some_and(|region| region.contains(position.x, position.y))
                });
                if hovering_grab {
                    mouse::Interaction::Grab
                } else {
                    mouse::Interaction::default()
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// GPU 一式(thread_local)
// ---------------------------------------------------------------------------

/// 読み出しやすさ優先の gamma 空間 RGBA8(E0 probe と同じ)。
/// `EmbeddedSpatialStage::new` の surface_format と offscreen target を揃える。
const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

struct Embed {
    token: u64,
    stage: EmbeddedSpatialStage,
    target: Option<Target>,
    blit: Option<Blit>,
    /// 直近に stage へ伝えた面の大きさ(物理 px)。変わった時だけ fit_view。
    fitted: Option<(u32, u32)>,
    /// 直近の pointer 位置(物理 px)。Cancel(位置なし)に差す。
    last_pointer: (f32, f32),
    /// 直近の modifiers ビット。手ごとに stage へ渡す。
    modifiers: u32,
    /// probe 経路(fixture 展示・pixel oracle)が載せた世代。変わった時だけ
    /// upload し直す。**`document` が座っている間は使わない** — その間は
    /// `frame_seat` の texture がそのまま評価済みフレームになる。
    presented_frame: Option<u64>,
    frame_texture: Option<wgpu::Texture>,
    /// live Document の評価済みフレームの席(M-3)。`document` が初めて座った
    /// 時に立てる。egui shell の `StagePane::frame_seat` と同じ役目 —
    /// 第二の render 経路を作らず、`motolii_ui::stage_frame_seat` を1つ共有する。
    frame_seat: Option<StageFrameSeat>,
    /// 席を立てられなかった(GPU/worker)。理由は1度だけ言い、以後は probe 経路
    /// (絵は出ないまま)にとどまる。
    frame_seat_failed: bool,
}

struct Target {
    /// view が生きている間 texture も生かしておくための所有(直接は読まない)。
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
}

impl Embed {
    fn build(token: u64, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, String> {
        // iced は adapter を渡してこない(spike の詰まった箇所その3)。instance を
        // 作り直して同じ物理 device の adapter を選ぶ。`DeviceCaps::from_adapter` は
        // 読み取りだけなので、device が2つになる訳ではない。
        let instance = wgpu::Instance::new(re_renderer::device_caps::instance_descriptor(None));
        let adapters =
            iced::futures::executor::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
        let adapter =
            re_renderer::device_caps::select_adapter(&adapters, wgpu::Backends::all(), None)
                .map_err(|error| format!("adapter を選べない: {error}"))?;

        let stage = EmbeddedSpatialStage::new(&adapter, device, queue, TARGET_FORMAT)?;

        Ok(Self {
            token,
            stage,
            target: None,
            blit: None,
            fitted: None,
            last_pointer: (0.0, 0.0),
            modifiers: 0,
            presented_frame: None,
            frame_texture: None,
            frame_seat: None,
            frame_seat_failed: false,
        })
    }

    #[allow(clippy::too_many_arguments, reason = "iced の prepare が運ぶ物そのまま")]
    fn frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        scale: f32,
        composition_aspect: Option<f32>,
        document: Option<&Arc<Document>>,
        playhead: f32,
    ) -> Result<(), String> {
        // 面の大きさに合わせて offscreen target と blit を持ち替える。
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != (width, height))
        {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("motolii stage island offscreen"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TARGET_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.blit = Some(Blit::new(device, pass_format, &view));
            self.target = Some(Target {
                _texture: texture,
                view,
                size: (width, height),
            });
        }

        // camera seat 結線: composition の形は Document から、面の形はここから。
        // どちらも同じ値なら stage 側で弾かれる(再生中に視点が跳ねない)。
        if let Some(aspect) = composition_aspect {
            self.stage.set_composition_aspect(aspect);
        }
        if self.fitted != Some((width, height)) {
            if !self.stage.fit_view(width, height) {
                report_once("stage: 面の大きさを camera へ伝えられない(fit_view が拒否)");
            }
            self.fitted = Some((width, height));
        }

        // 評価済みフレームの席。**live Document が座っている間だけ**
        // `stage_frame_seat` を回す — 座っていなければ probe 経路のまま
        // (`present_probe_frame`。pixel oracle が使う fixture 展示)。
        if let Some(document) = document {
            self.drive_frame_seat(device, queue, document, playhead);
        } else {
            self.present_probe_frame(device, queue)?;
        }

        // 入力を流す(論理 → 物理)。調停済みの分だけがここへ来ている。
        for input in take_inputs() {
            match input {
                StageInput::Pointer {
                    phase,
                    button,
                    x,
                    y,
                } => {
                    let position = if x.is_finite() && y.is_finite() {
                        (x * scale, y * scale)
                    } else {
                        // Cancel は位置を持たない — 直近の位置で言う。
                        self.last_pointer
                    };
                    self.last_pointer = position;
                    self.stage.pointer(
                        phase,
                        button,
                        self.modifiers,
                        f64::from(position.0),
                        f64::from(position.1),
                    );
                }
                StageInput::Scroll {
                    delta_x,
                    delta_y,
                    x,
                    y,
                } => {
                    let position = (x * scale, y * scale);
                    self.last_pointer = position;
                    // delta は論理 point のまま(spike の実測: scale を掛けると2倍速い)。
                    if !self.stage.scroll(
                        f64::from(delta_x),
                        f64::from(delta_y),
                        0.0,
                        self.modifiers,
                        f64::from(position.0),
                        f64::from(position.1),
                    ) {
                        report_once("stage: スクロールを stage へ渡せない(値が壊れている)");
                    }
                }
                StageInput::Modifiers(bits) => {
                    self.modifiers = bits;
                }
            }
        }

        // 1フレーム描く。wgpu 検証層の失敗は error scope で拾って報告する
        // (spike の実測 8 番: 蹴られたフレームは絵ごと捨てられ、黙ると
        // 「ドラッグ中だけ絵が止まる」が原因不明のまま残る)。
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let evaluated: Option<&wgpu::Texture> = if document.is_some() {
            // live: 席が完成させた最新の合成フレーム。間に合っていなければ
            // `None`(seat 自身が前の絵を保つ側の面倒を見る — ここは持たない)。
            self.frame_seat.as_ref().and_then(StageFrameSeat::frame)
        } else {
            let evaluated_is_current = self.presented_frame.is_some()
                && self.presented_frame == probe_frame_generation();
            if evaluated_is_current {
                self.frame_texture.as_ref()
            } else {
                None
            }
        };
        let target = self.target.as_ref().expect("target was just ensured");
        let render_result = self
            .stage
            .render(device, queue, &target.view, width, height, evaluated);
        let pending = scope.pop();
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        if let Some(error) = iced::futures::executor::block_on(pending) {
            report_once(format!("stage: wgpu の検証層に蹴られた: {error}"));
        }
        render_result.map_err(|error| format!("Stage の描画に失敗: {error}"))?;

        // 絵と幾何の載せ替えで落ちた分。stage は Err を返さず溜めて返す
        // (egui pane と同じ契約 — 2026-08-18 外部診断 F-08)。
        for failure in self.stage.take_failures() {
            report_once(format!("stage: {failure}"));
        }
        Ok(())
    }

    /// live Document の評価済みフレームの席を回す。**戻すものは無い** —
    /// 出す1枚は `frame_seat.frame()` から `frame()` が直接取る
    /// (egui shell の `StagePane::drive_frame_seat` と同じ分担)。
    ///
    /// repaint の合図は egui 版の `on_frame_ready` コールバックではなく、
    /// `main.rs::Host::subscription` の `window::frames()` 購読(document が
    /// 座っている間だけ)に任せている — iced の shader widget はホストの
    /// redraw に乗じて `prepare` が回るので、非同期の完成を毎フレーム
    /// `poll()` で見に行けば足りる(egui のように自分で repaint を起こす
    /// 手段がここには無い)。
    fn drive_frame_seat(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        document: &Arc<Document>,
        playhead: f32,
    ) {
        if self.frame_seat.is_none() && !self.frame_seat_failed {
            // **iced のランタイム device と同じ device** で立てる。完成 texture は
            // Rerun の texture pool へ実 handle のまま渡すので、別 device のものは
            // import できない(`StageFrameSeat::spawn` の doc と同じ理由)。
            let gpu = Arc::new(motolii_gpu::GpuCtx::from_device_queue(
                device.clone(),
                queue.clone(),
            ));
            match StageFrameSeat::spawn(gpu) {
                Ok(seat) => self.frame_seat = Some(seat),
                Err(error) => {
                    self.frame_seat_failed = true;
                    report_once(format!("stage: 評価済みフレームの席を立てられない: {error}"));
                }
            }
        }
        let Some(seat) = self.frame_seat.as_mut() else {
            return;
        };
        seat.request(document, playhead);
        seat.poll();
        record_frame_seat_evidence(seat.evidence());
        if let Some(error) = seat.take_error() {
            report_once(format!("stage: 評価済みフレームの合成に失敗: {error}"));
        }
    }

    /// 評価済みフレームの席(probe 経路)。CPU の RGBA を device の texture へ上げ、
    /// `render(…, evaluated_frame)` として stage の comp 平面に載せる。
    /// **`document` が `None`(fixture 展示・pixel oracle)の間だけ使う** —
    /// live 座席は [`Self::drive_frame_seat`] が `stage_frame_seat` で評価する。
    fn present_probe_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), String> {
        let generation = probe_frame_generation();
        if generation.is_none() || self.presented_frame == generation {
            return Ok(());
        }
        let Some((width, height, rgba)) = probe_frame_pixels() else {
            return Ok(());
        };
        if width == 0 || height == 0 || rgba.len() != (width * height * 4) as usize {
            // 席に着けない絵は黙って捨てない。世代は进めて毎フレーム言わない。
            self.presented_frame = generation;
            report_once(format!(
                "stage: 評価済みフレームの寸法が合わない({width}x{height}, {} bytes)",
                rgba.len()
            ));
            return Ok(());
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii stage island evaluated frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
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
        self.frame_texture = Some(texture);
        self.presented_frame = generation;
        Ok(())
    }
}

/// offscreen texture を iced の render pass へ写すだけの最小パイプライン
/// (spike の `Blit` と同じ)。bind group は1つ — iced の device は既定で
/// `max_bind_groups = 2` なので、床が効かない環境でもここは通る。
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
    fn new(device: &wgpu::Device, format: wgpu::TextureFormat, source: &wgpu::TextureView) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii stage island blit"),
            source: wgpu::ShaderSource::Wgsl(BLIT_WGSL.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii stage island blit layout"),
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
            label: Some("motolii stage island blit sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii stage island blit bind group"),
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
            label: Some("motolii stage island blit pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii stage island blit pipeline"),
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
// 溜め場(入力キュー・mailbox・観測)
// ---------------------------------------------------------------------------

thread_local! {
    static EMBED: RefCell<Option<Embed>> = const { RefCell::new(None) };
    /// この device(Pipeline の token)では建て損ねた、の印。
    static FAILED_TOKEN: RefCell<Option<u64>> = const { RefCell::new(None) };
}

/// 調停を通った入力の溜め場。iced の1フレームは update(配送)→ prepare の順なので、
/// 「update で溜めて prepare でまとめて渡す」以外の形にならない(spike と同じ)。
static QUEUE: Mutex<Vec<StageInput>> = Mutex::new(Vec::new());
/// 窓が止まっていても溜まり続けないための天井。運転席(snapshot を撮らない走行)対策。
const QUEUE_CAP: usize = 4096;

fn push_input(input: StageInput) {
    if let Ok(mut queue) = QUEUE.lock() {
        if queue.len() < QUEUE_CAP {
            queue.push(input);
        }
    }
}

fn take_inputs() -> Vec<StageInput> {
    QUEUE
        .lock()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default()
}

/// 失敗報告の mailbox。`RedrawRequested` が `Message::StageReported` として吐き出す。
static FAILURES: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// 一度言った一言(process 単位の latch)。帯を同じ失敗で埋めない。
static REPORTED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn report_once(text: impl Into<String>) {
    let text = text.into();
    if let Ok(mut reported) = REPORTED.lock() {
        if reported.iter().any(|known| *known == text) {
            return;
        }
        reported.push(text.clone());
    }
    if let Ok(mut failures) = FAILURES.lock() {
        failures.push(text);
    }
}

fn take_reports() -> Vec<String> {
    FAILURES
        .lock()
        .map(|mut failures| std::mem::take(&mut *failures))
        .unwrap_or_default()
}

static PIPELINE_TOKENS: AtomicU64 = AtomicU64::new(0);
static OBSERVED_MAX_BIND_GROUPS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static FORWARDED: AtomicU64 = AtomicU64::new(0);
static SWALLOWED: AtomicU64 = AtomicU64::new(0);

/// 評価済みフレーム(合成絵)の席。RGBA8(premultiplied)を CPU から差す。
///
/// M-2 ではテストの既知絵(E0 と同じ4象限)がここへ入り、pixel oracle が
/// 正対既定を審判する。M-3 で `stage_frame_seat`(playhead 時刻の合成フレーム、
/// GPU 常駐)がこの席を置き換える。
pub fn install_probe_frame(width: u32, height: u32, rgba: Vec<u8>) {
    if let Ok(mut slot) = PROBE_FRAME.lock() {
        let generation = slot.as_ref().map_or(0, |frame| frame.generation) + 1;
        *slot = Some(ProbeFrame {
            generation,
            width,
            height,
            rgba,
        });
    }
}

struct ProbeFrame {
    generation: u64,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

static PROBE_FRAME: Mutex<Option<ProbeFrame>> = Mutex::new(None);

fn probe_frame_generation() -> Option<u64> {
    PROBE_FRAME
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|frame| frame.generation))
}

fn probe_frame_pixels() -> Option<(u32, u32, Vec<u8>)> {
    PROBE_FRAME
        .lock()
        .ok()
        .and_then(|slot| {
            slot.as_ref()
                .map(|frame| (frame.width, frame.height, frame.rgba.clone()))
        })
}

// ---------------------------------------------------------------------------
// 観測の口(常設 oracle・運転席が読む)
// ---------------------------------------------------------------------------

/// `Pipeline::new` が渡された device の `max_bind_groups` の履歴(古い順)。
///
/// fork seam 2 の**実効** oracle がここを読む: 床を上げる前は上流既定の 2、
/// 上げた後は [`RERUN_MIN_MAX_BIND_GROUPS`] 以上が**取得**できていること。
pub fn observed_max_bind_groups() -> Vec<u32> {
    OBSERVED_MAX_BIND_GROUPS
        .lock()
        .map(|seen| seen.clone())
        .unwrap_or_default()
}

/// ブリッジを渡った入力の内訳(forwarded, swallowed)。調停の GPU 側審判が読む。
pub fn input_tally() -> (u64, u64) {
    (
        FORWARDED.load(Ordering::Relaxed),
        SWALLOWED.load(Ordering::Relaxed),
    )
}

/// live 評価済みフレームの席(`stage_frame_seat`)の直近の実測。
/// `document` が一度でも座って `drive_frame_seat` が回った後だけ `Some`。
///
/// 運転席が「submit した」「受けた」を確定的に見るための口(`thread_local`
/// の `Embed` は crate 外から直接読めないので、他の観測口と同じ static 経由)。
static FRAME_SEAT_EVIDENCE: Mutex<Option<StageFrameEvidence>> = Mutex::new(None);

fn record_frame_seat_evidence(evidence: StageFrameEvidence) {
    if let Ok(mut slot) = FRAME_SEAT_EVIDENCE.lock() {
        *slot = Some(evidence);
    }
}

/// [`FRAME_SEAT_EVIDENCE`] の読み口。運転席が `submitted`/`accepted` の
/// 変化をポーリングして「評価済みフレームが実際に流れた」ことを審判する。
pub fn frame_seat_evidence() -> Option<StageFrameEvidence> {
    FRAME_SEAT_EVIDENCE.lock().ok().and_then(|slot| *slot)
}
