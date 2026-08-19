//! 窓を開く薄殻。
//!
//! ```text
//! cargo run -p motolii-shell-iced
//! cargo run -p motolii-shell-iced -- --project /path/to/project.json
//! cargo run -p motolii-shell-iced -- --intent-log /tmp/intents.jsonl
//! cargo run -p motolii-shell-iced -- --status-log /tmp/statuses.jsonl
//! cargo run -p motolii-shell-iced -- --screenshot /tmp/iced-shell.png [frames]
//! ```
//!
//! `--project` が無い通常起動は**最後に開いていた project へ座り直す**
//! (egui 側 wave D の `last_project` を共用。`resume` モジュール参照)。
//!
//! egui shell(`cargo run -p motolii-ui --bin motolii-blitz-shell`)は**そのまま在る**。
//! 既定 bin の切り替えは M-5 で、UX 台本が iced 側で通ってからである。
//!
//! `--screenshot` は egui 版(`crates/motolii-ui/src/blitz_shell/runner.rs`)と
//! **同じ引数の形・同じ役目**: 窓を1枚だけ描いてPNGにし、そのまま終了する。
//! 人が窓を開かずに見た目を確認する検証器具で、製品機能ではない
//! (2026-08-18 視覚再現レーン)。撮影の都合は `Host` を包む [`Capture`] に
//! 閉じ込め、`Message`(製品の意味の列)には1つも足さない — 台帳の柵
//! 「Message/intent/read-model/replay 台本に手を入れない」を守るため。

use std::path::PathBuf;

use image::ImageEncoder as _;

use motolii_shell_iced::{
    decide_resume, view, IntentLog, Launch, Message, NativePrompts, Outcome, ScreenshotRequest,
    Shell, StatusLog,
};
use motolii_ui::blitz_shell::Resume;

/// egui shell(`crates/motolii-ui/src/blitz_shell/runner.rs`)と同じ作業領域。
/// productStyles.ts:4 `shell: {minWidth: 980, minHeight: 650}` が両shellの
/// 共通正本 — iced 側だけ狭く既定しない。
const WINDOW_WIDTH: f32 = 980.0;
const WINDOW_HEIGHT: f32 = 650.0;

fn main() -> iced::Result {
    // Stage 島(Rerun renderer 同居)のための bind group 床。**窓を建てる前**で
    // なければ効かない(iced fork seam 2。台帳 §3)。
    motolii_shell_iced::stage_island::install_rerun_device_floor();

    let launch = match Launch::parse(std::env::args().skip(1)) {
        Ok(launch) => launch,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };

    // 記録先が開けないなら**起動前に**言って落ちる。窓を開いてから
    // 「実は記録していなかった」を起こさない。
    let logs = match open_logs(&launch) {
        Ok(logs) => logs,
        Err(reason) => {
            eprintln!("motolii-shell-iced: {reason}");
            std::process::exit(2);
        }
    };
    let screenshot = launch.screenshot.clone();

    // 「最後に開いていた project」の置き場(palette settings と同じ家)。
    // `--project` が来ていればそちらを優先し、無ければここへ座り直す(F-01)。
    // 開けない `--project` も、消えていた覚え project も、**窓は開いたまま**
    // 理由が帯へ出る(`decide_resume` / `ShellGateway::resumed` の3択)。
    let last_project = motolii_ui::default_last_project_path();
    let resume = decide_resume(&launch, last_project.as_deref());

    // `BootFn` は `Fn() -> _`(**`FnOnce` ではない**)なので、move で握った
    // 記録先/座席をそのまま渡せない。1回しか呼ばれない物を1回だけ渡す口として
    // `RefCell::take` を使う。2度目が来たら空の Boot になる — 起きないはずの
    // ことを黙って握り潰すのではなく、記録しない・座らないことがそのまま見えるように
    // してある。
    let boot = std::cell::RefCell::new(Some(Boot {
        logs,
        resume,
        last_project,
    }));
    iced::application(
        move || {
            let Boot {
                logs,
                resume,
                last_project,
            } = boot.borrow_mut().take().unwrap_or_default();
            Host::new(logs, resume, last_project, screenshot.clone())
        },
        Host::update,
        Host::view,
    )
    .subscription(Host::subscription)
    // 見た目は token 正本から(`theme::product`)。iced 既定 palette の藍色を
    // 1画面も残さない — 見た目も嘘をつかない(Q0 の精神)。
    .theme(|_host: &Host| motolii_shell_iced::theme::product())
    .title("Motolii")
    .window_size((WINDOW_WIDTH, WINDOW_HEIGHT))
    // 閉じるかどうかは殻が決める(未保存なら3択)。窓に勝手に閉じさせない。
    .exit_on_close_request(false)
    .run()
}

/// `BootFn` へ1回だけ渡す一式(記録先 + 起動時の座席の決め方)。
struct Boot {
    logs: Logs,
    resume: Resume,
    last_project: Option<PathBuf>,
}

impl Default for Boot {
    /// `BootFn` が(起きないはずだが)2度呼ばれた時の逃げ場。記録しない・
    /// 座らないという「何もしない」側へ倒す — 黙って前回の座席を複製したりしない。
    fn default() -> Self {
        Self {
            logs: Logs::default(),
            resume: Resume::Nothing,
            last_project: None,
        }
    }
}

/// 記録の口ぜんたい。**殻の側には記録の都合を1つも足さない**(egui 版 `Harness` と同じ分担)。
#[derive(Default)]
struct Logs {
    intent: Option<IntentLog>,
    status: Option<StatusLog>,
}

impl Logs {
    /// 原因を先に流す。記録の順が「intent → その結果の status」だから。
    fn flush(&mut self, shell: &Shell) {
        if let Some(log) = self.intent.as_mut() {
            log.flush(shell);
        }
        if let Some(log) = self.status.as_mut() {
            log.flush(shell);
        }
    }
}

fn open_logs(launch: &Launch) -> Result<Logs, String> {
    Ok(Logs {
        intent: launch
            .intent_log
            .as_deref()
            .map(IntentLog::create)
            .transpose()?,
        status: launch
            .status_log
            .as_deref()
            .map(StatusLog::create)
            .transpose()?,
    })
}

/// 殻 + 記録先 + (在れば)撮影の進行。
struct Host {
    shell: Shell,
    logs: Logs,
    /// `--screenshot` のときだけ [`Some`]。無ければ撮影の刻みも購読しない —
    /// 通常起動の subscription を1本も増やさない。
    capture: Option<Capture>,
}

/// `--screenshot` の進行状態。**製品状態ではない** — `Shell` にも `Message` にも
/// 混ぜず、`Host` の外側だけに置く(egui 版 `runner::Harness::shot` と同じ分担)。
struct Capture {
    request: ScreenshotRequest,
    frame_count: u32,
    requested: bool,
}

/// 窓の Message ぜんたい。**製品の意味は [`Message::App`] の中だけ** —
/// 撮影の2状態(`Tick` / `Captured`)は検証器具の都合で、`Shell::update` は
/// 一度も見ない(台帳の柵「Message/intent/read-model/replay 台本に手を入れない」)。
#[derive(Debug, Clone)]
enum HostMessage {
    App(Message),
    /// 1フレーム経った(`--screenshot` 待ちのときだけ購読する刻み)。
    ScreenshotTick,
    /// 撮り終わった1枚が届いた。
    ScreenshotCaptured(iced::window::Screenshot),
}

impl Host {
    /// 座席は窓より先に決まっている(`resume`)。`last_project` を渡しているので、
    /// ここから先の座り直し(New / Open)も同じ置き場を覚え続ける。撮影要求は
    /// 検証器具で、製品の意味には触れない。
    fn new(
        logs: Logs,
        resume: Resume,
        last_project: Option<PathBuf>,
        screenshot: Option<ScreenshotRequest>,
    ) -> Self {
        Self {
            shell: Shell::resumed(NativePrompts, resume, last_project),
            logs,
            capture: screenshot.map(|request| Capture {
                request,
                frame_count: 0,
                requested: false,
            }),
        }
    }

    fn update(&mut self, message: HostMessage) -> iced::Task<HostMessage> {
        match message {
            HostMessage::App(inner) => {
                let outcome = self.shell.update(inner);
                self.logs.flush(&self.shell);
                match outcome {
                    Outcome::Stay => iced::Task::none(),
                    Outcome::Close => iced::exit(),
                }
            }
            HostMessage::ScreenshotTick => {
                let Some(capture) = self.capture.as_mut() else {
                    return iced::Task::none();
                };
                if capture.requested {
                    return iced::Task::none();
                }
                capture.frame_count += 1;
                // Browser のサムネイル・波形の decode など非同期の届き物が
                // 来るのを待つ(egui 版と同じ理由。`launch.rs` の doc 参照)。
                if capture.frame_count < capture.request.frames {
                    return iced::Task::none();
                }
                capture.requested = true;
                iced::window::latest().then(|id| match id {
                    Some(id) => iced::window::screenshot(id).map(HostMessage::ScreenshotCaptured),
                    None => iced::Task::none(),
                })
            }
            HostMessage::ScreenshotCaptured(shot) => {
                if let Some(capture) = self.capture.take() {
                    write_png(
                        &capture.request.path,
                        &shot.rgba,
                        shot.size.width,
                        shot.size.height,
                    );
                    println!(
                        "motolii-shell-iced: {} ({}x{})",
                        capture.request.path.display(),
                        shot.size.width,
                        shot.size.height
                    );
                }
                iced::exit()
            }
        }
    }

    /// 走っている書き出しの返事を受けるための刻み。**書き出し中だけ**購読する
    /// (egui 版が `request_repaint_after(200ms)` で自分を起こしているのと同じ役目)。
    ///
    /// 近道キーと OS ドロップを**ここに置かない**のは意図的である。購読は
    /// `iced_test::Simulator` の外に居るので、置くと運転席から注入できなくなる —
    /// あれらは widget 木の中(`window_input`)で受ける。
    fn subscription(&self) -> iced::Subscription<HostMessage> {
        let mut ticks = Vec::new();
        if self.shell.export_running() {
            ticks.push(iced::window::frames().map(|_| HostMessage::App(Message::ExportPolled)));
        }
        // 波形の decode が別 thread で走っているあいだだけ、受け取りの刻みを持つ
        // (書き出しと同じ型の解決。生成が済めば購読ごと消える)。
        if self.shell.waveform_building() {
            ticks.push(iced::window::frames().map(|_| HostMessage::App(Message::WaveformPolled)));
        }
        // 撮影が済むまでだけ、自分で描き続ける刻みを持つ(egui 版
        // `ctx.request_repaint()` と同じ理由 — 入力が無いと再描画が来ない)。
        if self
            .capture
            .as_ref()
            .is_some_and(|capture| !capture.requested)
        {
            ticks.push(iced::window::frames().map(|_| HostMessage::ScreenshotTick));
        }
        // Stage 島の評価済みフレームの席は live Document が座っている間だけ走る
        // (`StagePane::wants_evaluated_frame` と同じ条件)。playhead 評価は
        // render worker thread の非同期な返事なので、egui のように自分で
        // repaint を起こす手段が無いここでは、この刻みが無いと完成したフレームが
        // 画面に届かない。
        if self.shell.document().is_some() {
            ticks.push(iced::window::frames().map(|_| HostMessage::App(Message::StagePolled)));
        }
        // 再生中だけ playhead を進める刻みを持つ(egui 版が毎フレームの `show()`
        // で `ctx.input(|i| i.stable_dt)` を読んでいるのと同じ役目 —
        // iced にはそれを持ってくるフレームワークの仕組みが無いので、
        // 購読が生きているあいだだけ `window::frames()` を dt の刻みに使う)。
        // pause 中は購読ごと外れるので、壁時計は動かない
        // (2026-08-19 iced 再生機構移植レーン)。
        if self.shell.timeline_playing() {
            ticks.push(iced::window::frames().map(|_| HostMessage::App(Message::PlaybackTick)));
        }
        iced::Subscription::batch(ticks)
    }

    /// **`fn` 項目であって closure ではない。**
    ///
    /// `|host: &Host| -> Element<'_, _>` はクロージャの推論が高階の生存期間を
    /// 作れず `implementation of ViewFn is not general enough` で落ちる
    /// (`spikes/iced-rerun-embed-probe` の詰まった箇所その4)。
    fn view(&self) -> iced::Element<'_, HostMessage> {
        view(&self.shell).map(HostMessage::App)
    }
}

/// 撮った1枚を8bit RGBAのPNGにする。egui 版
/// `blitz_shell::runner::write_png` と同じ役目・同じ形。
fn write_png(path: &PathBuf, rgba: &[u8], w: u32, h: u32) {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = std::fs::File::create(path).expect("png create");
    image::codecs::png::PngEncoder::new(std::io::BufWriter::new(file))
        .write_image(rgba, w, h, image::ExtendedColorType::Rgba8)
        .expect("png write");
}
