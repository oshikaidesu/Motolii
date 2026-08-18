//! 入力ブリッジを実測する走行。(d) orbit / (e) cursor / (f) 制約。
//!
//! `app.rs`((a)(b)(c) の走行)とは**別のアプリ**にしてある。既に取れている
//! (b)(c) の証拠をこのレーンの変更で塗り替えないためで、共有しているのは
//! `embed`(GPU 側一式)と `bridge`(翻訳)と `oracle`(判定)だけである。
//!
//! ## 1 tick = 1 egui フレーム
//!
//! iced の1フレームは `update`(イベント配送)→ `view` → `prepare` の順に走る。
//! `bridge` のキューは `update` で積まれ、同じフレームの `prepare` で空になる。
//! なので「tick T で押した入力は、tick T の絵に出る」。逆に**読み戻し**
//! (`read_offscreen` / `embed::eye`)は tick T の頭ではまだ tick T-1 の結果である。
//! 台本の `Capture` は「1つ前の step の結果を撮る」という意味になる。
//!
//! ## 3つの走らせ方
//!
//! | | 何を通るか |
//! |---|---|
//! | `bridge` | iced の窓 + 合成した `iced::Event` を `Program::update` へ直接。翻訳から先は本物 |
//! | `bridge-interactive` | iced の窓 + 人のドラッグ。winit → widget tree の配送も通る |
//! | `bridge-offscreen <kind>` | 窓なし。**同じ台本**を指定した device 記述の上で回す対照群 |
//!
//! 3つ目が要るのは、iced の窓の中で絵が止まったとき「ブリッジが悪いのか、
//! iced の device 制限が悪いのか」を分けるためである。`re_renderer` 記述の
//! device で同じ台本が通れば、原因は device 側だと言い切れる。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use iced::widget::shader;
use iced::window::Screenshot;
use iced::{mouse, Element, Fill, Point, Rectangle, Subscription, Task};

use crate::app::{WINDOW_H, WINDOW_W};
use crate::bridge;
use crate::embed::{BridgeProgram, CameraMode};
use crate::harness::{DeviceKind, Gpu, Offscreen};
use crate::oracle::Frame;
use crate::stage;

static OUT_DIR: OnceLock<PathBuf> = OnceLock::new();
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

fn out_dir() -> PathBuf {
    OUT_DIR.get().cloned().unwrap_or_else(|| PathBuf::from("."))
}

/// 証拠の接頭辞。走らせ方ごとに別の名前にして、互いを塗り潰さない。
fn prefix() -> &'static str {
    if interactive() {
        "interactive-bridge-live-"
    } else {
        "interactive-bridge-"
    }
}

/// 1 step あたりの drag 量(論理 point)。egui の drag 判定閾値より十分大きく取る。
const DRAG_STEP_X: f32 = 14.0;
const DRAG_STEP_Y: f32 = 4.0;
const DRAG_STEPS: u32 = 12;

/// Rerun の「orbit 中心の目印」は最後の操作から 0.35 秒で消える(`ui_3d.rs`)。
/// 固定 dt 1/60 秒で回しているので、消えるのを待つにはこれだけフレームが要る。
const FADE_FRAMES: u32 = 30;

/// 台本の1手。1 tick で1手ずつ進む。
#[derive(Debug, Clone, Copy)]
enum Step {
    /// 何もしない。補間や目印の fade を進める。
    Idle,
    /// カメラの持ち主を変える。
    Camera(CameraMode),
    /// widget 局所の論理座標へカーソルを動かす。
    Move { x: f32, y: f32 },
    Press(mouse::Button),
    Release(mouse::Button),
    /// 行単位のホイール。
    Wheel { x: f32, y: f32 },
    /// 1つ前の step の結果を offscreen から撮る。
    Capture(&'static str),
    /// iced の窓そのものを撮る。「本当に画面に出ている」ことの証拠。窓なしでは飛ばす。
    WindowShot(&'static str),
    /// (e) の観測。egui が今何のカーソルを要求しているかを記録する。
    NoteCursor(&'static str),
}

fn script() -> Vec<Step> {
    let center = (WINDOW_W * 0.5, WINDOW_H * 0.5);
    let mut steps = Vec::new();
    let idle = |steps: &mut Vec<Step>, count: u32| {
        for _ in 0..count {
            steps.push(Step::Idle);
        }
    };

    // --- 立ち上げ。document camera のまま warmup が終わるのを待つ ---
    idle(&mut steps, 24);
    steps.push(Step::Capture("00-document-camera"));

    // --- カメラを Rerun 自身へ返す。ここから先が (d) の測定域 ---
    steps.push(Step::Camera(CameraMode::Free));
    idle(&mut steps, 40);
    steps.push(Step::Capture("01-free-camera-settled"));

    // --- (d) orbit: 左ドラッグ ---
    steps.push(Step::Move {
        x: center.0,
        y: center.1,
    });
    steps.push(Step::Idle);
    steps.push(Step::NoteCursor("hovering, no button"));
    steps.push(Step::Press(mouse::Button::Left));
    for index in 1..=DRAG_STEPS {
        steps.push(Step::Move {
            x: center.0 + DRAG_STEP_X * index as f32,
            y: center.1 + DRAG_STEP_Y * index as f32,
        });
    }
    steps.push(Step::Capture("02-during-drag"));
    steps.push(Step::NoteCursor("mid-drag, left button down"));
    // ドラッグしたまま「動かさない」フレームを1つ挟む。
    steps.push(Step::Idle);
    steps.push(Step::Capture("03-drag-held-but-still"));
    steps.push(Step::Release(mouse::Button::Left));
    idle(&mut steps, 3);
    steps.push(Step::Capture("04-just-after-release"));
    // 目印が消えるまで待つ。ここで絵が動き出すなら「止まっていたのは目印のせい」。
    idle(&mut steps, FADE_FRAMES);
    steps.push(Step::Capture("05-after-the-indicator-faded"));
    idle(&mut steps, FADE_FRAMES);
    steps.push(Step::Capture("06-long-after-the-drag"));

    // --- (d) zoom: ホイール ---
    steps.push(Step::Move {
        x: center.0,
        y: center.1,
    });
    steps.push(Step::Idle);
    for _ in 0..6 {
        steps.push(Step::Wheel { x: 0.0, y: -3.0 });
    }
    steps.push(Step::Capture("07-during-wheel"));
    idle(&mut steps, FADE_FRAMES);
    steps.push(Step::Capture("08-after-the-wheel-faded"));

    steps.push(Step::WindowShot("09-window"));
    steps
}

/// widget は窓いっぱいなので bounds は窓そのもの。合成イベントの座標系もこれで決まる。
fn bounds() -> Rectangle {
    Rectangle {
        x: 0.0,
        y: 0.0,
        width: WINDOW_W,
        height: WINDOW_H,
    }
}

/// 合成した `iced::Event` を作る小道具。窓ありでも窓なしでも同じ物を通す。
fn event_of(step: Step, cursor: &mut Option<Point>) -> Option<iced::Event> {
    match step {
        Step::Move { x, y } => {
            let position = Point::new(bounds().x + x, bounds().y + y);
            *cursor = Some(position);
            Some(iced::Event::Mouse(mouse::Event::CursorMoved { position }))
        }
        Step::Press(button) => Some(iced::Event::Mouse(mouse::Event::ButtonPressed(button))),
        Step::Release(button) => Some(iced::Event::Mouse(mouse::Event::ButtonReleased(button))),
        Step::Wheel { x, y } => Some(iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x, y },
        })),
        _ => None,
    }
}

/// 合成イベントを **widget の `Program::update` に通す**。翻訳を迂回しない。
fn feed(event: iced::Event, cursor: Option<Point>, camera: CameraMode, generation: u64) {
    let program = BridgeProgram { camera, generation };
    let cursor = match cursor {
        Some(point) => mouse::Cursor::Available(point),
        None => mouse::Cursor::Unavailable,
    };
    let mut state = ();
    // `Message` は使わないが、trait は型を要求する。窓なしでも同じ実装を通したいので
    // ここだけ単位型に対して呼ぶ。
    let _action = shader::Program::<()>::update(&program, &mut state, &event, bounds(), cursor);
}

// ---------------------------------------------------------------------------
// 測定値
// ---------------------------------------------------------------------------

/// 1フレーム分の読み戻し。
#[derive(Debug, Clone, Copy)]
struct Sample {
    tick: u32,
    digest: u64,
    eye: Option<[f32; 6]>,
    /// このフレームまでに wgpu が文句を言った通算回数。前の行との差が
    /// 「このフレームは検証層に蹴られた」の印。
    validation: u64,
}

#[derive(Debug, Clone)]
struct Capture {
    name: String,
    digest: u64,
    eye: Option<[f32; 6]>,
    validation: u64,
}

#[derive(Debug, Default)]
struct Measurements {
    samples: Vec<Sample>,
    captures: Vec<Capture>,
    notes: Vec<String>,
}

impl Measurements {
    fn find(&self, name: &str) -> Option<&Capture> {
        self.captures.iter().find(|capture| capture.name == name)
    }

    /// 撮った物どうしを突き合わせて (d)(e)(f) の合否にする。
    fn verdicts(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let counters = bridge::counters();
        lines.push(format!(
            "bridge delivered {} egui events ({} moves, {} presses, {} releases, {} wheels, \
             {} focus)",
            counters.delivered,
            counters.moved,
            counters.pressed,
            counters.released,
            counters.wheel,
            counters.focus
        ));

        let settled = self.find("01-free-camera-settled");
        let during_drag = self.find("02-during-drag");
        let held = self.find("03-drag-held-but-still");
        let released = self.find("04-just-after-release");
        let faded = self.find("05-after-the-indicator-faded");
        let long_after = self.find("06-long-after-the-drag");
        let during_wheel = self.find("07-during-wheel");
        let after_wheel = self.find("08-after-the-wheel-faded");

        // --- (d) 到達 ---
        match (settled, during_drag) {
            (Some(base), Some(drag)) => {
                let picture_moved = base.digest != drag.digest;
                let travel = eye_distance(base.eye, drag.eye);
                let eye_moved = travel.is_some_and(|distance| distance > 1e-3);
                lines.push(format!(
                    "(d) reach — during the left-drag the picture {} and Rerun's own eye {} \
                     (by {})",
                    if picture_moved { "CHANGED" } else { "did NOT change" },
                    if eye_moved { "MOVED" } else { "did NOT move" },
                    format_distance(travel),
                ));
                if eye_moved {
                    lines.push(
                        "(d) reach PASS: iced -> bridge -> egui RawInput -> SpatialStage's own \
                         EyeController. Rerun turned its own camera; no set_camera was involved."
                            .to_owned(),
                    );
                } else {
                    lines.push("(d) reach FAIL: the drag never reached Rerun's camera".to_owned());
                }
            }
            _ => lines.push("(d) reach could not be measured: a capture is missing".to_owned()),
        }

        // --- (d) 絵が追随したか ---
        let drag_rows: Vec<&Sample> = self.drag_window();
        if !drag_rows.is_empty() {
            let distinct: std::collections::BTreeSet<u64> =
                drag_rows.iter().map(|row| row.digest).collect();
            let complained = drag_rows
                .windows(2)
                .filter(|pair| pair[1].validation > pair[0].validation)
                .count();
            lines.push(format!(
                "(d) picture — across the {} frames of the drag the offscreen texture took {} \
                 distinct values, and {} of those frames were refused by the wgpu validation layer",
                drag_rows.len(),
                distinct.len(),
                complained
            ));
            if distinct.len() <= 2 && complained > 0 {
                lines.push(
                    "(d) picture FAIL: the eye kept moving but the picture froze, and the frames \
                     that froze are exactly the ones wgpu refused. See the validation text below."
                        .to_owned(),
                );
            }
        }

        // --- (d) 残るか ---
        if let (Some(base), Some(drag), Some(end)) = (settled, during_drag, long_after) {
            let peak = eye_distance(base.eye, drag.eye);
            let rest = eye_distance(base.eye, end.eye);
            lines.push(format!(
                "(d) persistence — the drag took the eye {} away from the pre-drag pose; \
                 {} frames after the release it is {} away",
                format_distance(peak),
                2 * FADE_FRAMES + 4,
                format_distance(rest),
            ));
            match (peak, rest) {
                (Some(peak), Some(rest)) if rest < peak * 0.5 => lines.push(
                    "(d) persistence FAIL: the orbit does not accumulate — the eye eases back \
                     toward Rerun's default framing. SpatialStage swallows the blueprint write \
                     that would have kept it (see README)."
                        .to_owned(),
                ),
                (Some(_), Some(_)) => lines.push(
                    "(d) persistence PASS: the eye stayed where the drag put it".to_owned(),
                ),
                _ => {}
            }
        }
        if let (Some(drag), Some(held)) = (during_drag, held) {
            lines.push(format!(
                "(d) persistence — one frame later, still holding the button but not moving, \
                 the eye had already slid back by {}",
                format_distance(eye_distance(drag.eye, held.eye)),
            ));
        }
        if let (Some(released), Some(faded)) = (released, faded) {
            lines.push(format!(
                "(d) recovery — {FADE_FRAMES} frames after the release (long enough for Rerun's \
                 orbit-centre indicator to fade out) the picture {} what it was right after the \
                 release, and {} more validation errors arrived in between",
                if released.digest == faded.digest {
                    "is STILL"
                } else {
                    "differs from"
                },
                faded.validation - released.validation,
            ));
        }

        // --- (d) zoom ---
        match (settled, during_wheel, after_wheel) {
            (Some(base), Some(wheel), Some(after)) => {
                lines.push(format!(
                    "(d) zoom — six wheel notches moved the eye {} from the settled pose; \
                     {FADE_FRAMES} frames later it is {} away",
                    format_distance(eye_distance(base.eye, wheel.eye)),
                    format_distance(eye_distance(base.eye, after.eye)),
                ));
            }
            _ => lines.push("(d) zoom could not be measured: a capture is missing".to_owned()),
        }

        // --- (e) ---
        let icons = bridge::cursor_icons_seen();
        let unmapped: Vec<_> = icons
            .iter()
            .filter(|icon| bridge::to_iced_interaction(**icon).is_none())
            .collect();
        lines.push(format!(
            "(e) egui asked for these cursor icons across the run: {icons:?}; \
             {} of them have no iced::mouse::Interaction ({unmapped:?})",
            unmapped.len()
        ));

        // --- (f) ---
        let delays = bridge::repaint_delays_seen();
        lines.push(format!(
            "(f) repaint — egui asked for these repaint delays in ms: {}. \
             `Primitive::prepare` returns `()`, so there is no seam to answer them from; this \
             run redrew every frame with `iced::window::frames()` instead.",
            format_delays(&delays)
        ));

        if let Some(text) = crate::embed::last_validation() {
            lines.push(format!("last wgpu validation error: {text}"));
        }

        lines
    }

    /// press から release までの読み戻し行。「絵が追随したか」を数えるのに使う。
    fn drag_window(&self) -> Vec<&Sample> {
        let Some(start) = self.notes.iter().position(|note| note.starts_with("@press ")) else {
            return Vec::new();
        };
        let Some(end) = self
            .notes
            .iter()
            .position(|note| note.starts_with("@release "))
        else {
            return Vec::new();
        };
        let (Some(start), Some(end)) = (tick_of(&self.notes[start]), tick_of(&self.notes[end]))
        else {
            return Vec::new();
        };
        self.samples
            .iter()
            .filter(|sample| sample.tick > start && sample.tick <= end + 1)
            .collect()
    }
}

fn tick_of(note: &str) -> Option<u32> {
    note.split_whitespace().nth(1)?.parse().ok()
}

fn format_eye(eye: Option<[f32; 6]>) -> String {
    match eye {
        Some([px, py, pz, fx, fy, fz]) => {
            format!("pos({px:.4}, {py:.4}, {pz:.4}) fwd({fx:.4}, {fy:.4}, {fz:.4})")
        }
        None => "none".to_owned(),
    }
}

fn format_distance(distance: Option<f32>) -> String {
    match distance {
        Some(distance) => format!("{distance:.5}"),
        None => "unknown".to_owned(),
    }
}

fn format_delays(delays: &[u128]) -> String {
    let rendered: Vec<String> = delays
        .iter()
        .map(|delay| {
            if *delay > 1_000_000 {
                "never".to_owned()
            } else {
                format!("{delay}")
            }
        })
        .collect();
    format!("[{}]", rendered.join(", "))
}

fn write_report(path: &Path, header: &str, measurements: &Measurements) {
    let verdicts = measurements.verdicts();
    for line in &verdicts {
        println!("[bridge] {line}");
    }

    let mut text = String::new();
    text.push_str(header);
    text.push_str("\n## verdicts\n\n");
    for line in &verdicts {
        text.push_str(&format!("{line}\n"));
    }

    text.push_str(&format!(
        "\n## what crossed the bridge\n\n{:#?}\n",
        bridge::counters()
    ));

    text.push_str("\n## captures (name, fnv1a, eye, validation errors so far)\n\n");
    for capture in &measurements.captures {
        text.push_str(&format!(
            "{}: {:016x}  eye {}  validation {}\n",
            capture.name,
            capture.digest,
            format_eye(capture.eye),
            capture.validation
        ));
    }

    text.push_str("\n## per-tick readback (tick, fnv1a, eye, validation errors so far)\n\n");
    text.push_str("Each row is the frame produced by the step of the *previous* tick.\n\n");
    for sample in &measurements.samples {
        text.push_str(&format!(
            "tick {:4}  {:016x}  {}  v={}\n",
            sample.tick,
            sample.digest,
            format_eye(sample.eye),
            sample.validation
        ));
    }

    text.push_str("\n## embed log\n\n");
    for line in crate::embed::snapshot_log() {
        text.push_str(&format!("{line}\n"));
    }
    text.push_str("\n## step log\n\n");
    for line in &measurements.notes {
        text.push_str(&format!("{line}\n"));
    }

    if let Err(error) = std::fs::write(path, text) {
        eprintln!("could not write {}: {error}", path.display());
    } else {
        println!("[bridge] wrote {}", path.display());
    }
}

// ---------------------------------------------------------------------------
// 窓ありの走行
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Shot(&'static str, Screenshot),
}

pub struct BridgeApp {
    ticks: u32,
    steps: Vec<Step>,
    cursor: Option<Point>,
    generation: u64,
    camera: CameraMode,
    next_step: usize,
    awaiting_shot: bool,
    done: bool,
    measurements: Measurements,
}

impl Default for BridgeApp {
    fn default() -> Self {
        Self {
            ticks: 0,
            steps: script(),
            cursor: None,
            generation: 1,
            camera: CameraMode::Document { pull_back: 1.0 },
            next_step: 0,
            awaiting_shot: false,
            done: false,
            measurements: Measurements::default(),
        }
    }
}

impl BridgeApp {
    fn note(&mut self, line: impl Into<String>) {
        let line = line.into();
        println!("[bridge] {line}");
        self.measurements.notes.push(line);
    }

    pub fn boot() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "iced x Rerun SpatialStage input bridge probe".to_owned()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.done {
            Subscription::none()
        } else {
            iced::window::frames().map(|_| Message::Tick)
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        shader(BridgeProgram {
            camera: self.camera,
            generation: self.generation,
        })
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn capture(&mut self, name: &'static str) {
        let Some((rgba, width, height)) = crate::embed::read_offscreen() else {
            self.note(format!("capture {name}: no offscreen texture yet"));
            return;
        };
        let frame = Frame::new(rgba, width, height);
        let path = out_dir().join(format!("{}{name}.png", prefix()));
        let capture = Capture {
            name: name.to_owned(),
            digest: frame.digest(),
            eye: crate::embed::eye(),
            validation: crate::embed::validation_total(),
        };
        match frame.write_png(&path) {
            Ok(()) => self.note(format!(
                "capture {name}: wrote {} ({}x{}, fnv1a {:016x}, eye {})",
                path.display(),
                frame.width,
                frame.height,
                capture.digest,
                format_eye(capture.eye)
            )),
            Err(error) => self.note(format!("capture {name}: {error}")),
        }
        self.measurements.captures.push(capture);
    }

    fn run_step(&mut self, step: Step) -> Task<Message> {
        match step {
            Step::Camera(camera) => {
                self.camera = camera;
                self.generation += 1;
                self.note(format!(
                    "step: camera -> {camera:?} (generation {})",
                    self.generation
                ));
            }
            Step::Press(button) => {
                self.note(format!("@press {} {button:?}", self.ticks));
            }
            Step::Release(button) => {
                self.note(format!("@release {} {button:?}", self.ticks));
            }
            Step::Capture(name) => {
                self.capture(name);
                return Task::none();
            }
            Step::NoteCursor(what) => {
                self.note_cursor(what);
                return Task::none();
            }
            Step::WindowShot(name) => {
                self.awaiting_shot = true;
                return iced::window::latest()
                    .and_then(iced::window::screenshot)
                    .map(move |shot| Message::Shot(name, shot));
            }
            Step::Idle | Step::Move { .. } | Step::Wheel { .. } => {}
        }

        let mut cursor = self.cursor;
        if let Some(event) = event_of(step, &mut cursor) {
            self.cursor = cursor;
            feed(event, self.cursor, self.camera, self.generation);
        }
        Task::none()
    }

    fn note_cursor(&mut self, what: &'static str) {
        let icon = bridge::cursor_icon();
        let mapped = bridge::to_iced_interaction(icon);
        let observation = crate::embed::observation();
        self.note(format!(
            "(e) {what}: egui asked for {icon:?} -> iced {mapped:?}; \
             egui wants_pointer_input = {}",
            observation.wants_pointer_input
        ));
        // `Program::mouse_interaction` が実際に返す物も同じ経路で確かめる。
        let program = BridgeProgram {
            camera: self.camera,
            generation: self.generation,
        };
        let cursor = mouse::Cursor::Available(Point::new(
            bounds().x + WINDOW_W * 0.5,
            bounds().y + WINDOW_H * 0.5,
        ));
        let state = ();
        let interaction =
            shader::Program::<Message>::mouse_interaction(&program, &state, bounds(), cursor);
        self.note(format!(
            "(e) {what}: Program::mouse_interaction returned {interaction:?}"
        ));
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.done {
                    return Task::none();
                }
                self.ticks += 1;
                if self.awaiting_shot {
                    return Task::none();
                }

                if let Some((rgba, width, height)) = crate::embed::read_offscreen() {
                    let frame = Frame::new(rgba, width, height);
                    self.measurements.samples.push(Sample {
                        tick: self.ticks,
                        digest: frame.digest(),
                        eye: crate::embed::eye(),
                        validation: crate::embed::validation_total(),
                    });
                }

                if interactive() {
                    // 実走では台本を回さない。人がドラッグするのを待って、
                    // その間の eye と digest を溜め続ける。
                    if self.ticks > 60_000 {
                        return self.finish();
                    }
                    return Task::none();
                }

                let Some(step) = self.steps.get(self.next_step).copied() else {
                    return self.finish();
                };
                self.next_step += 1;
                self.run_step(step)
            }
            Message::Shot(name, shot) => {
                self.awaiting_shot = false;
                let frame = Frame::new(shot.rgba.to_vec(), shot.size.width, shot.size.height);
                let path = out_dir().join(format!("{}{name}.png", prefix()));
                match frame.write_png(&path) {
                    Ok(()) => self.note(format!(
                        "window shot {name}: wrote {} ({}x{}, scale factor {}, fnv1a {:016x})",
                        path.display(),
                        frame.width,
                        frame.height,
                        shot.scale_factor,
                        frame.digest()
                    )),
                    Err(error) => self.note(format!("window shot {name}: {error}")),
                }
                Task::none()
            }
        }
    }

    fn finish(&mut self) -> Task<Message> {
        if self.done {
            return Task::none();
        }
        self.done = true;

        let header = format!(
            "# iced x Rerun SpatialStage — input bridge run\n\nmode: {}\n",
            if interactive() {
                "bridge-interactive — a human drove the window; winit and the widget tree did \
                 the delivery"
            } else {
                "bridge — synthesized iced events handed to the widget's Program::update; \
                 the translation and everything downstream of it is real"
            }
        );
        let path = out_dir().join(format!("{}run.txt", prefix()));
        write_report(&path, &header, &self.measurements);

        iced::exit()
    }
}

pub fn run(out_dir: PathBuf, interactive: bool) -> Result<(), iced::Error> {
    set_out_dir(out_dir);
    set_interactive(interactive);

    iced::application(BridgeApp::boot, BridgeApp::update, BridgeApp::view)
        .title(BridgeApp::title)
        .subscription(BridgeApp::subscription)
        .window_size((WINDOW_W, WINDOW_H))
        .resizable(false)
        .antialiasing(false)
        .run()
}

// ---------------------------------------------------------------------------
// 窓なしの対照群
// ---------------------------------------------------------------------------

/// **同じ台本**を、窓を開かずに、指定した device 記述の上で回す。
///
/// `iced-windowed` と `re_renderer` の2つを比べると、絵が止まる原因が
/// ブリッジ側か device 側かが分かれる。翻訳は `Program::update` を通す物と
/// 同じ関数を使うので、経路の差は「窓と配送があるかどうか」だけになる。
pub fn run_offscreen(kind: DeviceKind, out_dir: &Path) -> Result<(), String> {
    let width = WINDOW_W as u32;
    let height = WINDOW_H as u32;
    let label = kind.label();
    let prefix = format!("interactive-bridge-offscreen-{label}-");

    let (gpu, note) = Gpu::create(kind)?;
    println!("[bridge] {note}");
    let mut offscreen = Offscreen::new(gpu, width, height)?;
    let mut spatial = stage::new_stage(&format!("bridge-offscreen-{label}"))?;
    stage::install_quadrants(&mut offscreen, &mut spatial)?;
    for _ in 0..stage::WARMUP_FRAMES {
        offscreen.frame(&mut spatial)?;
    }
    spatial.set_camera(stage::document_camera(1.0));
    for _ in 0..stage::SETTLE_FRAMES {
        offscreen.frame(&mut spatial)?;
    }

    let mut measurements = Measurements::default();
    measurements.notes.push(note);
    let mut cursor = None;
    let mut generation = 1_u64;
    let mut camera = CameraMode::Document { pull_back: 1.0 };

    // 1 step = 1 frame。窓ありの走行と同じ順序(読み戻し → step → 1 frame)にする。
    for (index, step) in script().into_iter().enumerate() {
        let tick = index as u32 + 1;

        let rgba = offscreen.read_rgba()?;
        let frame = Frame::new(rgba, width, height);
        let sample = Sample {
            tick,
            digest: frame.digest(),
            eye: last_eye(&spatial),
            validation: crate::embed::validation_total(),
        };
        measurements.samples.push(sample);

        match step {
            Step::Camera(new_camera) => {
                camera = new_camera;
                generation += 1;
                match camera {
                    CameraMode::Document { pull_back } => {
                        spatial.set_camera(stage::document_camera(pull_back));
                    }
                    CameraMode::Free => spatial.clear_camera(),
                }
                measurements
                    .notes
                    .push(format!("step: camera -> {camera:?} (generation {generation})"));
            }
            Step::Capture(name) => {
                let path = out_dir.join(format!("{prefix}{name}.png"));
                let capture = Capture {
                    name: name.to_owned(),
                    digest: sample.digest,
                    eye: sample.eye,
                    validation: sample.validation,
                };
                frame.write_png(&path)?;
                measurements.notes.push(format!(
                    "capture {name}: wrote {} (fnv1a {:016x}, eye {})",
                    path.display(),
                    capture.digest,
                    format_eye(capture.eye)
                ));
                measurements.captures.push(capture);
            }
            Step::NoteCursor(what) => {
                measurements.notes.push(format!(
                    "(e) {what}: egui asked for {:?} -> iced {:?}",
                    offscreen.last_cursor_icon(),
                    bridge::to_iced_interaction(offscreen.last_cursor_icon())
                ));
            }
            Step::Press(button) => measurements.notes.push(format!("@press {tick} {button:?}")),
            Step::Release(button) => {
                measurements.notes.push(format!("@release {tick} {button:?}"));
            }
            Step::WindowShot(_) => {}
            Step::Idle | Step::Move { .. } | Step::Wheel { .. } => {}
        }

        if let Some(event) = event_of(step, &mut cursor) {
            feed(event, cursor, camera, generation);
        }

        let (events, modifiers) = bridge::drain(1.0);
        offscreen.frame_with_input(&mut spatial, events, modifiers)?;
    }

    let header = format!(
        "# iced x Rerun SpatialStage — input bridge run (offscreen control)\n\n\
         device kind: {label}\n\
         mode: bridge-offscreen — the same script, the same translation, no window\n"
    );
    write_report(&out_dir.join(format!("{prefix}run.txt")), &header, &measurements);
    Ok(())
}

fn last_eye(stage: &re_view_spatial::SpatialStage) -> Option<[f32; 6]> {
    stage.last_eye().map(|eye| {
        let position = eye.pos_in_world();
        let forward = eye.forward_in_world();
        [
            position.x, position.y, position.z, forward.x, forward.y, forward.z,
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_presses_before_it_drags_and_releases_after() {
        let steps = script();
        let press = steps
            .iter()
            .position(|step| matches!(step, Step::Press(_)))
            .expect("a press");
        let release = steps
            .iter()
            .position(|step| matches!(step, Step::Release(_)))
            .expect("a release");
        let moves_between = steps[press..release]
            .iter()
            .filter(|step| matches!(step, Step::Move { .. }))
            .count();
        assert!(press < release);
        assert_eq!(moves_between, DRAG_STEPS as usize);
    }

    #[test]
    fn the_free_camera_comes_before_the_drag() {
        let steps = script();
        let free = steps
            .iter()
            .position(|step| matches!(step, Step::Camera(CameraMode::Free)))
            .expect("a free-camera step");
        let press = steps
            .iter()
            .position(|step| matches!(step, Step::Press(_)))
            .expect("a press");
        assert!(
            free < press,
            "orbit can only be measured once set_camera has let go"
        );
    }

    /// 目印の fade を待つ間は本当に何も起きていないこと。ここに入力が混じると
    /// 「絵が戻ったのは目印が消えたからだ」と言えなくなる。
    #[test]
    fn the_fade_wait_is_pure_idling() {
        let steps = script();
        let released = steps
            .iter()
            .position(|step| matches!(step, Step::Release(_)))
            .expect("a release");
        let faded = steps
            .iter()
            .position(|step| matches!(step, Step::Capture("05-after-the-indicator-faded")))
            .expect("the fade capture");
        let idles = steps[released..faded]
            .iter()
            .filter(|step| matches!(step, Step::Idle))
            .count();
        assert!(idles >= FADE_FRAMES as usize, "idled {idles} frames");
        assert!(
            !steps[released..faded]
                .iter()
                .any(|step| matches!(step, Step::Move { .. } | Step::Wheel { .. })),
            "no input may sneak into the fade wait"
        );
    }

    #[test]
    fn eye_distance_is_zero_for_the_same_eye() {
        let eye = Some([1.0, 2.0, 3.0, 0.0, 0.0, -1.0]);
        assert_eq!(eye_distance(eye, eye), Some(0.0));
        assert_eq!(eye_distance(eye, None), None);
    }
}

/// 位置と前方をまとめた「どれだけ動いたか」。単位は無い(比較用)。
fn eye_distance(a: Option<[f32; 6]>, b: Option<[f32; 6]>) -> Option<f32> {
    let (a, b) = (a?, b?);
    let sum: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
    Some(sum.sqrt())
}
