//! wraps: iced daemon/multiwindow API の gate probe(裁定188)— 製品コードに載る前の実測器具。
//!
//! r5-multiwindow — iced multiwindow(窓の浮かし)の gate probe。
//!
//! 実行: `cargo run -p r5-multiwindow -- --auto`
//!   `--auto` は自走モード — 人手なしで
//!   main 窓を開く → Settings 風の第2窓を開く → 両窓の wgpu device を突き合わせる
//!   → Settings 窓を閉じる → main が生きていることを frame 数で確かめる
//!   → Settings を開き直して counter(daemon State)が保持されていることを見る
//!   → `PROBE ...` 行を stdout へ吐いて自分で終了する。
//! 引数なしだと手動モード(ボタンで同じ操作を人が踏める)。
//!
//! 計測器: `shader` widget の `Primitive::prepare(device, queue, ..)` が
//! 各窓の描画パスから呼ばれるので、そこで窓タグ別に device/queue を記録して
//! Eq 比較する(wgpu 29 の Device/Queue は inner dispatch handle 比較の Eq)。
//! さらに機能テストとして「main 窓の prepare で作った wgpu::Buffer へ
//! settings 窓の prepare から write_buffer する」— device が窓ごとに別れて
//! いれば wgpu の validation error(既定 handler は panic)で即座に落ちる。

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use iced::widget::{button, column, container, row, shader, text};
use iced::window;
use iced::{Element, Length, Subscription, Task, Theme};

// ---------------------------------------------------------------------------
// device 記録簿(窓タグ → 実測)
// ---------------------------------------------------------------------------

struct DeviceRecord {
    device: wgpu::Device,
    queue: wgpu::Queue,
    /// この窓の描画パスから prepare が呼ばれた回数(= 実 frame 数の下界)
    frames: u64,
}

fn registry() -> &'static Mutex<BTreeMap<&'static str, DeviceRecord>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<&'static str, DeviceRecord>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// (作成した窓タグ, buffer)。別窓の prepare がこの buffer へ write_buffer する —
/// cross-device なら wgpu validation で落ちるので、生きて通れば機能面でも共有確定。
fn shared_buffer() -> &'static OnceLock<(&'static str, wgpu::Buffer)> {
    static BUFFER: OnceLock<(&'static str, wgpu::Buffer)> = OnceLock::new();
    &BUFFER
}

/// registry の実測から「device/queue が窓間で共有か」を返す。
/// 両窓が最低 2 frame ずつ描くまで None。
fn measure_shared() -> Option<bool> {
    let registry = registry().lock().unwrap();
    let main = registry.get("main")?;
    let settings = registry.get("settings")?;
    if main.frames < 2 || settings.frames < 2 {
        return None;
    }
    Some(main.device == settings.device && main.queue == settings.queue)
}

fn frames_of(tag: &str) -> u64 {
    registry().lock().unwrap().get(tag).map_or(0, |r| r.frames)
}

// ---------------------------------------------------------------------------
// 計測 shader primitive(何も描かない — prepare で device を記録するだけ)
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct DeviceProbe {
    tag: &'static str,
}

struct ProbePipeline;

impl shader::Pipeline for ProbePipeline {
    fn new(_device: &wgpu::Device, _queue: &wgpu::Queue, _format: wgpu::TextureFormat) -> Self {
        ProbePipeline
    }
}

impl shader::Primitive for DeviceProbe {
    type Pipeline = ProbePipeline;

    fn prepare(
        &self,
        _pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        {
            let mut registry = registry().lock().unwrap();
            let record = registry.entry(self.tag).or_insert_with(|| DeviceRecord {
                device: device.clone(),
                queue: queue.clone(),
                frames: 0,
            });
            record.frames += 1;
        }

        let (creator, buffer) = shared_buffer().get_or_init(|| {
            (
                self.tag,
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("r5-multiwindow cross-window probe buffer"),
                    size: 4,
                    usage: wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
            )
        });
        if *creator != self.tag {
            // 別の窓が作った buffer へ書く。device が窓ごとに別なら
            // ここで wgpu validation error(既定で panic)になる。
            queue.write_buffer(buffer, 0, &[0xA5, 0x5A, 0xA5, 0x5A]);
        }
    }
}

struct ProbeProgram {
    tag: &'static str,
}

impl<Message> shader::Program<Message> for ProbeProgram {
    type State = ();
    type Primitive = DeviceProbe;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        DeviceProbe { tag: self.tag }
    }
}

// ---------------------------------------------------------------------------
// daemon 本体
// ---------------------------------------------------------------------------

fn main() -> iced::Result {
    let auto = std::env::args().any(|a| a == "--auto");
    iced::daemon(
        move || Probe::new(auto),
        Probe::update,
        Probe::view,
    )
    .title(Probe::title)
    .theme(Probe::theme)
    .subscription(Probe::subscription)
    .run()
}

/// 自走の進行段階。`--auto` でだけ進む(手動モードは Boot のまま)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// main 窓が描き始めるのを待つ
    Boot,
    /// settings 窓を開いた — 両窓の device 実測が揃うのを待つ
    SettingsOpen,
    /// 判定を吐き settings を閉じた — Closed イベント待ち
    ClosingSettings,
    /// settings 閉鎖後 — main が描き続けることを frame 数で確かめる
    VerifyMainAlive { frames_at_close: u64 },
    /// settings を開き直した — counter 保持を見て終わる。
    /// `baseline` は開き直し時点の settings 累計 frame 数(記録簿は窓を跨いで
    /// 累計なので、ここからの増分で「2回目も実際に描いた」を判定する)。
    /// SettingsOpened が届くまでは None。
    Reopened { baseline: Option<u64> },
    Done,
}

struct Probe {
    auto: bool,
    phase: Phase,
    main_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    /// daemon State に住む「設定値」役。settings 窓を閉じて開き直しても
    /// 保持されることが state_persists の証拠。
    counter: u32,
    /// 自走のタイムアウト番犬(frame 数)
    watchdog: u64,
}

#[derive(Debug, Clone)]
enum Message {
    MainOpened(window::Id),
    SettingsOpened(window::Id),
    WindowClosed(window::Id),
    /// `window::frames()` — 全窓の redraw で刻む。自走の駆動源。
    Frame,
    OpenSettings,
    CloseSettings,
    Increment,
}

impl Probe {
    fn new(auto: bool) -> (Self, Task<Message>) {
        let (_id, open) = window::open(window::Settings {
            size: iced::Size::new(640.0, 420.0),
            ..window::Settings::default()
        });
        (
            Probe {
                auto,
                phase: Phase::Boot,
                main_window: None,
                settings_window: None,
                counter: 0,
                watchdog: 0,
            },
            open.map(Message::MainOpened),
        )
    }

    fn title(&self, window: window::Id) -> String {
        if Some(window) == self.settings_window {
            "r5 probe — Settings".into()
        } else {
            "r5 probe — Main".into()
        }
    }

    fn theme(&self, window: window::Id) -> Option<Theme> {
        // 窓別 theme が API として通ることの確認を兼ねる(settings だけ Light)
        Some(if Some(window) == self.settings_window {
            Theme::Light
        } else {
            Theme::Dark
        })
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::Frame),
            window::close_events().map(Message::WindowClosed),
        ])
    }

    fn open_settings(&mut self) -> Task<Message> {
        if self.settings_window.is_some() {
            return Task::none();
        }
        let (_id, open) = window::open(window::Settings {
            size: iced::Size::new(420.0, 320.0),
            ..window::Settings::default()
        });
        open.map(Message::SettingsOpened)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainOpened(id) => {
                self.main_window = Some(id);
                println!("PROBE main_opened id={id:?}");
                Task::none()
            }
            Message::SettingsOpened(id) => {
                self.settings_window = Some(id);
                println!("PROBE settings_opened id={id:?} counter={}", self.counter);
                if self.auto && matches!(self.phase, Phase::Reopened { baseline: None }) {
                    // 開き直し(2回目)— counter 保持の判定はここ
                    println!(
                        "PROBE state_persists={} counter={}",
                        self.counter == 2,
                        self.counter
                    );
                    self.phase = Phase::Reopened {
                        baseline: Some(frames_of("settings")),
                    };
                }
                Task::none()
            }
            Message::WindowClosed(id) => {
                if Some(id) == self.settings_window {
                    self.settings_window = None;
                    println!("PROBE settings_closed");
                    if self.auto && self.phase == Phase::ClosingSettings {
                        self.phase = Phase::VerifyMainAlive {
                            frames_at_close: frames_of("main"),
                        };
                    } else if self.auto && matches!(self.phase, Phase::Reopened { .. }) {
                        println!("PROBE RESULT: OK");
                        self.phase = Phase::Done;
                        return iced::exit();
                    }
                } else if Some(id) == self.main_window {
                    self.main_window = None;
                    println!("PROBE main_closed (daemon keeps running — exiting explicitly)");
                    return iced::exit();
                }
                Task::none()
            }
            Message::Frame => self.drive(),
            Message::OpenSettings => self.open_settings(),
            Message::CloseSettings => match self.settings_window {
                Some(id) => window::close(id),
                None => Task::none(),
            },
            Message::Increment => {
                self.counter += 1;
                Task::none()
            }
        }
    }

    /// 自走の状態機械(`--auto` 時のみ、frame tick で駆動)。
    fn drive(&mut self) -> Task<Message> {
        if !self.auto || self.phase == Phase::Done {
            return Task::none();
        }
        self.watchdog += 1;
        if self.watchdog > 3000 {
            println!("PROBE RESULT: TIMEOUT phase={:?}", self.phase);
            self.phase = Phase::Done;
            // 番犬は異常系 — 終了コードで失敗を報せる
            std::process::exit(2);
        }
        match self.phase {
            Phase::Boot => {
                if frames_of("main") >= 2 {
                    // 「設定を1つ触った」役 — settings 窓を開く前の状態変更
                    self.counter += 1;
                    self.phase = Phase::SettingsOpen;
                    return self.open_settings();
                }
                Task::none()
            }
            Phase::SettingsOpen => {
                if let Some(shared) = measure_shared() {
                    println!(
                        "PROBE device_shared={shared} queue_shared={shared} \
                         main_frames={} settings_frames={} cross_write_buffer=survived",
                        frames_of("main"),
                        frames_of("settings"),
                    );
                    // settings 窓の中で「設定を触った」役(counter=2 になる)
                    self.counter += 1;
                    self.phase = Phase::ClosingSettings;
                    if let Some(id) = self.settings_window {
                        return window::close(id);
                    }
                }
                Task::none()
            }
            Phase::VerifyMainAlive { frames_at_close } => {
                let now = frames_of("main");
                if now >= frames_at_close + 5 {
                    println!(
                        "PROBE main_alive_after_settings_close=true frames_after_close={}",
                        now - frames_at_close
                    );
                    // 開き直し — SettingsOpened 側で state_persists を判定。
                    // phase を先に進めるので open が二重発火することはない。
                    self.phase = Phase::Reopened { baseline: None };
                    return self.open_settings();
                }
                Task::none()
            }
            Phase::Reopened { baseline: Some(b) } => {
                // settings が2回目に開いた後 — 2 frame 描かせてから閉じて終了へ
                if frames_of("settings") >= b + 2 {
                    if let Some(id) = self.settings_window {
                        return window::close(id);
                    }
                }
                Task::none()
            }
            Phase::Reopened { baseline: None } | Phase::ClosingSettings | Phase::Done => {
                Task::none()
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if Some(window) == self.settings_window {
            self.view_settings()
        } else if Some(window) == self.main_window {
            self.view_main()
        } else {
            // 開いた直後 / 閉じた直後の 1 frame で来うる
            text("unknown window").into()
        }
    }

    fn view_main(&self) -> Element<'_, Message> {
        container(
            column![
                text("r5 multiwindow probe — main window").size(20),
                text(format!("counter (daemon state) = {}", self.counter)),
                row![
                    button(text("Open settings window")).on_press(Message::OpenSettings),
                    button(text("+1")).on_press(Message::Increment),
                ]
                .spacing(8),
                // 計測器: この窓の描画パスの device を記録する
                shader(ProbeProgram { tag: "main" })
                    .width(Length::Fixed(4.0))
                    .height(Length::Fixed(4.0)),
            ]
            .spacing(12),
        )
        .padding(16)
        .into()
    }

    fn view_settings(&self) -> Element<'_, Message> {
        container(
            column![
                text("Settings (probe)").size(20),
                text(format!("counter (shared state) = {}", self.counter)),
                row![
                    button(text("+1")).on_press(Message::Increment),
                    button(text("Close")).on_press(Message::CloseSettings),
                ]
                .spacing(8),
                shader(ProbeProgram { tag: "settings" })
                    .width(Length::Fixed(4.0))
                    .height(Length::Fixed(4.0)),
            ]
            .spacing(12),
        )
        .padding(16)
        .into()
    }
}
