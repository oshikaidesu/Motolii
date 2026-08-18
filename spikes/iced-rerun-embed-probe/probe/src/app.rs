//! iced 側のアプリ本体。shader widget を窓いっぱいに置き、自分で自分を検分する。
//!
//! 窓の中身は shader widget **だけ**にしてある。そうすると
//! `iced::window::screenshot` の出力がそのまま widget の絵になり、E0 の四隅 oracle を
//! 切り出し無しでそのまま当てられる。probe の主張が「切り出し方」に依存しなくなる。
//!
//! 進行は tick で自動化してあり、人が触らなくても
//! (b) の PNG →(c) の click → (c) 後の PNG まで撮って終了する。

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use iced::widget::shader;
use iced::window::Screenshot;
use iced::{Element, Fill, Subscription, Task};

use crate::embed::{StageProgram, StagePrimitive};
use crate::oracle::Frame;
use crate::stage;

static OUT_DIR: OnceLock<PathBuf> = OnceLock::new();
/// `gui-interactive` では (c) の click を合成せず、**人が実際に押すのを待つ**。
static INTERACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_out_dir(path: PathBuf) {
    let _ = OUT_DIR.set(path);
}

pub fn set_interactive(interactive: bool) {
    INTERACTIVE.store(interactive, Ordering::Relaxed);
}

fn interactive() -> bool {
    INTERACTIVE.load(Ordering::Relaxed)
}

/// 自動走行の証拠を人手の走行で上書きしない。
fn file_prefix() -> &'static str {
    if interactive() {
        "interactive-"
    } else {
        ""
    }
}

fn out_dir() -> PathBuf {
    OUT_DIR.get().cloned().unwrap_or_else(|| PathBuf::from("."))
}

/// 窓の論理サイズ。E0 の 640x480 と同じ縦横比にしてある。
pub const WINDOW_W: f32 = 640.0;
pub const WINDOW_H: f32 = 480.0;

/// click 後のカメラ。1.0 が「document が画枠ちょうど」なので、引くと四隅に
/// Rerun の背景が出る。絵が変わったことを画素で言うための値。
const CLICKED_PULL_BACK: f32 = 1.6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Before,
    After,
}

impl Slot {
    fn file_name(self) -> &'static str {
        match self {
            Self::Before => "iced-b-document-camera.png",
            Self::After => "iced-c-after-click.png",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Shot(Slot, Screenshot),
    /// **(c)**: shader widget の上の click が、renderer の都合を含まない型付きの値として届く。
    StageClicked {
        x: f32,
        y: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Warmup,
    AwaitBefore,
    AwaitClick,
    AwaitAfter,
    Done,
}

pub struct App {
    ticks: u32,
    phase: Phase,
    pull_back: f32,
    generation: u64,
    shot_after_at: u32,
    before_digest: Option<u64>,
    clicks: Vec<(f32, f32)>,
    report: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            ticks: 0,
            phase: Phase::Warmup,
            pull_back: 1.0,
            generation: 1,
            shot_after_at: u32::MAX,
            before_digest: None,
            clicks: Vec::new(),
            report: Vec::new(),
        }
    }
}

impl App {
    fn note(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("[app] {line}");
        self.report.push(line);
    }

    pub fn boot() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "iced x Rerun SpatialStage embed probe".to_owned()
    }

    /// `iced::time::every` は `tokio`/`smol` feature が要る(既定の `thread-pool`
    /// executor には `every` が無い)。executor を増やさずに済ませたいので、
    /// 描画ごとに来る `window::frames()` を tick にする。副作用として毎フレーム
    /// 再描画が要求されるので、shader widget の `prepare` も毎フレーム回る。
    pub fn subscription(&self) -> Subscription<Message> {
        if self.phase == Phase::Done {
            Subscription::none()
        } else {
            iced::window::frames().map(|_| Message::Tick)
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        shader(StageProgram {
            pull_back: self.pull_back,
            generation: self.generation,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.ticks += 1;

                // 窓が出て最初の何枚かが描かれるまで待つ。warmup 90 フレームを
                // 最初の prepare で回すので、1枚目は重い。
                if self.phase == Phase::Warmup && self.ticks >= 30 {
                    self.phase = Phase::AwaitBefore;
                    self.note("requesting the (b) screenshot");
                    return screenshot_task(Slot::Before);
                }
                if self.phase == Phase::AwaitAfter && self.ticks >= self.shot_after_at {
                    self.shot_after_at = u32::MAX;
                    self.note("requesting the (c) screenshot");
                    return screenshot_task(Slot::After);
                }
                // 進まないまま時間が過ぎたら、そこまでの結果で終わる。
                let patience = if interactive() { 60_000 } else { 3_000 };
                if self.ticks > patience && self.phase != Phase::Done {
                    self.note(format!("timed out in phase {:?}", self.phase));
                    return self.finish();
                }
                Task::none()
            }

            Message::Shot(slot, shot) => self.on_shot(slot, shot),

            Message::StageClicked { x, y } => {
                self.note(format!(
                    "App::update received Message::StageClicked {{ x: {x:.1}, y: {y:.1} }}"
                ));
                self.clicks.push((x, y));
                if self.phase == Phase::AwaitClick {
                    self.generation += 1;
                    self.pull_back = CLICKED_PULL_BACK;
                    self.phase = Phase::AwaitAfter;
                    self.shot_after_at = self.ticks + 10;
                    self.note(format!(
                        "the click moved the stage camera: set_camera(pull_back = {CLICKED_PULL_BACK}), \
                         camera generation is now {}",
                        self.generation
                    ));
                }
                Task::none()
            }
        }
    }

    fn on_shot(&mut self, slot: Slot, shot: Screenshot) -> Task<Message> {
        let frame = Frame::new(
            shot.rgba.to_vec(),
            shot.size.width,
            shot.size.height,
        );
        let path = out_dir().join(format!("{}{}", file_prefix(), slot.file_name()));
        match frame.write_png(&path) {
            Ok(()) => self.note(format!(
                "wrote {} ({}x{}, scale factor {})",
                path.display(),
                frame.width,
                frame.height,
                shot.scale_factor
            )),
            Err(error) => self.note(format!("could not write {}: {error}", path.display())),
        }
        self.note(format!("  fnv1a: {:016x}", frame.digest()));

        match slot {
            Slot::Before => {
                let (histogram, present_failures) = stage::check_all_quadrants_present(&frame);
                self.report_block(&histogram);
                let (corners, corner_failures) = stage::check_frame_corners(&frame);
                self.report_block(&corners);

                let status = crate::embed::status();
                self.note(format!(
                    "embed status: initialized={} frames={} offscreen={}x{} error={:?}",
                    status.initialized,
                    status.frames,
                    status.offscreen_size.0,
                    status.offscreen_size.1,
                    status.error
                ));
                if status.validation_errors.is_empty() {
                    self.note("no wgpu validation errors on iced's device");
                } else {
                    for error in &status.validation_errors {
                        self.note(format!("wgpu validation error on iced's device: {error}"));
                    }
                }

                if present_failures.is_empty() && corner_failures.is_empty() && status.initialized {
                    self.note(
                        "(b) PASS: the Rerun-composited frame reached the iced window and passes \
                         the E0 corner oracle",
                    );
                } else {
                    for failure in present_failures.iter().chain(corner_failures.iter()) {
                        self.note(format!("(b) FAIL: {failure}"));
                    }
                }
                self.before_digest = Some(frame.digest());

                // Rerun が描いた物そのものも残す。iced の surface を通る前の絵なので、
                // E0 の PNG と直接見比べられる。
                if let Some((rgba, width, height)) = crate::embed::read_offscreen() {
                    let offscreen = Frame::new(rgba, width, height);
                    let path = out_dir().join("iced-b-offscreen-source.png");
                    match offscreen.write_png(&path) {
                        Ok(()) => self.note(format!(
                            "wrote {} (the offscreen texture the widget samples, fnv1a {:016x})",
                            path.display(),
                            offscreen.digest()
                        )),
                        Err(error) => self.note(format!("offscreen PNG: {error}")),
                    }
                }

                // --- (c) の起点 ---
                //
                // OS のマウスを合成する手立ては iced_test 抜きには無い(今回の範囲外)。
                // 代わりに、**widget 自身の `Program::update`** に click イベントを渡し、
                // それが返した型付き Message を**ランタイム経由で** `App::update` へ流す。
                // つまり測っているのは「widget の翻訳」と「アプリの反応」で、
                // winit → widget tree の配送だけは人の click に委ねている。
                self.phase = Phase::AwaitClick;
                if interactive() {
                    self.note(
                        "(c) interactive mode: waiting for a real left click inside the window — \
                         the runtime has to route it through winit and the widget tree itself",
                    );
                    return Task::none();
                }
                match synthesize_click(self.pull_back, self.generation) {
                    Some(message) => {
                        self.note(format!(
                            "(c) the shader widget's Program::update turned a left-button press \
                             into {message:?}"
                        ));
                        Task::done(message)
                    }
                    None => {
                        self.note("(c) FAIL: Program::update produced no Message for a click");
                        self.finish()
                    }
                }
            }

            Slot::After => {
                let changed = self.before_digest != Some(frame.digest());
                self.note(format!(
                    "(c) the picture after the click {} the picture before it",
                    if changed { "differs from" } else { "is IDENTICAL to" }
                ));
                let (corners, corner_failures) = stage::check_frame_corners(&frame);
                self.report_block(&corners);
                if changed && !corner_failures.is_empty() {
                    self.note(
                        "(c) PASS: the typed Message reached update(), set_camera() ran, and the \
                         frame corners are no longer the document (the camera pulled back)",
                    );
                } else if changed {
                    self.note(
                        "(c) PARTIAL: the picture changed, but the corners still read as the \
                         document — check the PNGs by eye",
                    );
                } else {
                    self.note("(c) FAIL: set_camera() did not change the picture");
                }
                self.finish()
            }
        }
    }

    fn report_block(&mut self, block: &str) {
        for line in block.lines() {
            self.note(line.to_owned());
        }
    }

    fn finish(&mut self) -> Task<Message> {
        if self.phase == Phase::Done {
            return Task::none();
        }
        self.phase = Phase::Done;

        let mut text = String::new();
        text.push_str("# iced x Rerun SpatialStage embed probe — GUI run\n\n");
        text.push_str("## embed log (what the shader widget did with iced's device)\n\n");
        for line in crate::embed::snapshot_log() {
            text.push_str(&format!("{line}\n"));
        }
        text.push_str("\n## app log (what the probe checked)\n\n");
        for line in &self.report {
            text.push_str(&format!("{line}\n"));
        }
        text.push_str(&format!(
            "\nclick positions that reached App::update: {:?}\n\
             (mode: {}. In `gui` the press is handed to the widget's Program::update and the \
             Message it returns is delivered by the runtime; winit's own dispatch is only \
             exercised in `gui-interactive`.)\n",
            self.clicks,
            if interactive() {
                "gui-interactive — these came from a human pressing the window"
            } else {
                "gui — synthesized press, runtime-delivered Message"
            },
        ));

        let path = out_dir().join(format!("{}gui-run.txt", file_prefix()));
        if let Err(error) = std::fs::write(&path, text) {
            eprintln!("could not write {}: {error}", path.display());
        } else {
            println!("[app] wrote {}", path.display());
        }

        iced::exit()
    }
}

fn screenshot_task(slot: Slot) -> Task<Message> {
    iced::window::latest()
        .and_then(iced::window::screenshot)
        .map(move |shot| Message::Shot(slot, shot))
}

/// widget の `Program::update` を直接叩いて、click → Message の翻訳だけを取り出す。
fn synthesize_click(pull_back: f32, generation: u64) -> Option<Message> {
    use iced::advanced::mouse;

    let program = StageProgram {
        pull_back,
        generation,
    };
    let bounds = iced::Rectangle {
        x: 0.0,
        y: 0.0,
        width: WINDOW_W,
        height: WINDOW_H,
    };
    let cursor = mouse::Cursor::Available(iced::Point::new(WINDOW_W * 0.25, WINDOW_H * 0.25));
    let event = iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

    let mut state = ();
    let action = shader::Program::<Message>::update(&program, &mut state, &event, bounds, cursor)?;
    let (message, _redraw, _status) = action.into_inner();
    message
}

/// 型が本当に繋がっていることをコンパイル時に固定しておく。
const _: fn() = || {
    fn assert_primitive<T: shader::Primitive>() {}
    assert_primitive::<StagePrimitive>();
};
