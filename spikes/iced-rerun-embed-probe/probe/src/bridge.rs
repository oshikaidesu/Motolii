//! **入力ブリッジ**: iced の `shader::Program::update` が受けたイベントを
//! egui の `RawInput` へ翻訳する1本の橋。
//!
//! 元 probe は egui を headless(`RawInput` の events が空)で回していたので、
//! `SpatialStage` 自身の対話 — orbit / zoom / hover / picking — は全部死んでいた。
//! カメラは iced の `Message` を受けて `set_camera` を直呼びしていたのであって、
//! Rerun のカメラ操作が動いていた訳ではない。ここはその欠けている半分を埋める。
//!
//! ## egui-winit との対応
//!
//! `egui-winit` は winit の `WindowEvent` を `egui::Event` に変換する。こちらは
//! iced の `iced::Event` を同じ `egui::Event` に変換する。違いは2つだけである。
//!
//! 1. **座標の原点**。winit の座標は窓の左上原点だが、egui に渡すのは
//!    「stage が描かれている領域」の左上原点でなければならない。iced は
//!    `Program::update` に widget の `bounds`(論理座標)をくれるので、そこを引く。
//! 2. **pixels_per_point**。`Offscreen` は widget の**物理**画素数で作られ、
//!    `egui_ctx.set_pixels_per_point(1.0)` で「1 point = 1 pixel」にしてある。
//!    なので iced の論理座標に scale factor を掛けたものが egui の座標になる。
//!    scale factor は `Primitive::prepare` の `Viewport` にしか無いので、
//!    キューには**論理座標のまま**積み、取り出す側(prepare)で掛ける。
//!
//! ## なぜキューを挟むのか
//!
//! iced の1フレームは `update`(イベント配送)→ `view` → `prepare`/`draw` の順に走る。
//! egui の1フレームは `RawInput` を丸ごと受け取ってから始まる。つまり
//! 「iced が配ったイベントを溜めて、その frame の `prepare` でまとめて egui へ渡す」
//! 以外の形にはならない。溜め場がこのモジュールである。

use std::sync::Mutex;
use std::time::Duration;

use iced::mouse;
use iced::Rectangle;

/// widget 局所の**論理**座標で溜める中間表現。egui へは `drain` で変換して渡す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pending {
    /// widget 左上を原点とする論理座標。
    Moved { x: f32, y: f32 },
    Button {
        x: f32,
        y: f32,
        button: egui::PointerButton,
        pressed: bool,
    },
    /// カーソルが窓から出た。egui では「ホバーしている物は無い」。
    Gone,
    /// 行単位のホイール。`ScrollDelta::Lines`。
    WheelLines { x: f32, y: f32 },
    /// 画素単位のホイール/トラックパッド。`ScrollDelta::Pixels`。
    WheelPixels { x: f32, y: f32 },
    Modifiers(egui::Modifiers),
    Focus(bool),
}

static QUEUE: Mutex<Vec<Pending>> = Mutex::new(Vec::new());
static MODIFIERS: Mutex<egui::Modifiers> = Mutex::new(egui::Modifiers::NONE);

/// egui が「このフレームこのカーソルにしてほしい」と言った最後の値。(e) の観測点。
static CURSOR_ICON: Mutex<egui::CursorIcon> = Mutex::new(egui::CursorIcon::Default);
/// 観測できた cursor icon の履歴(重複は畳む)。
static CURSOR_SEEN: Mutex<Vec<egui::CursorIcon>> = Mutex::new(Vec::new());

/// egui が要求した再描画の待ち時間。(f) の観測点。
static REPAINT_DELAY: Mutex<Option<Duration>> = Mutex::new(None);
/// 観測できた repaint_delay の履歴(重複は畳む)。ms 単位、`u64::MAX` は「無し/無限」。
static REPAINT_SEEN: Mutex<Vec<u128>> = Mutex::new(Vec::new());

/// ブリッジを何回通ったかの内訳。ログに出して「本当に流れたか」を言うため。
#[derive(Debug, Clone, Copy, Default)]
pub struct Counters {
    pub moved: u32,
    pub pressed: u32,
    pub released: u32,
    pub wheel: u32,
    pub gone: u32,
    pub modifiers: u32,
    pub focus: u32,
    /// egui へ実際に渡した `egui::Event` の総数。
    pub delivered: u64,
}

static COUNTERS: Mutex<Counters> = Mutex::new(Counters {
    moved: 0,
    pressed: 0,
    released: 0,
    wheel: 0,
    gone: 0,
    modifiers: 0,
    focus: 0,
    delivered: 0,
});

pub fn counters() -> Counters {
    COUNTERS.lock().map(|c| *c).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// iced → Pending
// ---------------------------------------------------------------------------

/// iced のボタンを egui のボタンへ。`Other` は egui に対応が無いので捨てる。
fn button_of(button: mouse::Button) -> Option<egui::PointerButton> {
    match button {
        mouse::Button::Left => Some(egui::PointerButton::Primary),
        mouse::Button::Right => Some(egui::PointerButton::Secondary),
        mouse::Button::Middle => Some(egui::PointerButton::Middle),
        mouse::Button::Back => Some(egui::PointerButton::Extra1),
        mouse::Button::Forward => Some(egui::PointerButton::Extra2),
        mouse::Button::Other(_) => None,
    }
}

fn modifiers_of(modifiers: iced::keyboard::Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.control(),
        shift: modifiers.shift(),
        // macOS の Cmd。egui は「command」を OS ごとに解釈するので両方入れる。
        mac_cmd: cfg!(target_os = "macos") && modifiers.logo(),
        command: if cfg!(target_os = "macos") {
            modifiers.logo()
        } else {
            modifiers.control()
        },
    }
}

/// **翻訳の本体**。`Program::update` から呼ぶ。
///
/// `bounds` は widget の論理矩形(窓の左上原点)。`cursor` は窓の論理座標。
/// 返り値が空なら「この橋には関係のないイベント」で、iced 側で捨ててよい。
///
/// ドラッグ中はカーソルが widget の外へ出ても座標を送り続ける。これは
/// egui-winit と同じ振る舞いで、そうしないと枠際で orbit が止まる。
pub fn translate(event: &iced::Event, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<Pending> {
    let local = |point: iced::Point| (point.x - bounds.x, point.y - bounds.y);
    let cursor_local = || cursor.position().map(local);

    match event {
        iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            let (x, y) = local(*position);
            vec![Pending::Moved { x, y }]
        }
        iced::Event::Mouse(mouse::Event::CursorLeft) => vec![Pending::Gone],
        iced::Event::Mouse(mouse::Event::CursorEntered) => Vec::new(),
        iced::Event::Mouse(mouse::Event::ButtonPressed(button)) => {
            let (Some(button), Some((x, y))) = (button_of(*button), cursor_local()) else {
                return Vec::new();
            };
            // 押下の直前に位置を確定させる。egui は press の pos を drag の原点にする。
            vec![
                Pending::Moved { x, y },
                Pending::Button {
                    x,
                    y,
                    button,
                    pressed: true,
                },
            ]
        }
        iced::Event::Mouse(mouse::Event::ButtonReleased(button)) => {
            let (Some(button), Some((x, y))) = (button_of(*button), cursor_local()) else {
                return Vec::new();
            };
            vec![Pending::Button {
                x,
                y,
                button,
                pressed: false,
            }]
        }
        iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
            mouse::ScrollDelta::Lines { x, y } => vec![Pending::WheelLines { x: *x, y: *y }],
            mouse::ScrollDelta::Pixels { x, y } => vec![Pending::WheelPixels { x: *x, y: *y }],
        },
        iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            vec![Pending::Modifiers(modifiers_of(*modifiers))]
        }
        iced::Event::Window(iced::window::Event::Focused) => vec![Pending::Focus(true)],
        iced::Event::Window(iced::window::Event::Unfocused) => vec![Pending::Focus(false)],
        // キーボード本体・IME・タッチは今回の範囲外。経路の有無は README に書く。
        _ => Vec::new(),
    }
}

/// 翻訳した物をキューへ積む。`Program::update` の唯一の副作用。
pub fn push(pending: Vec<Pending>) {
    if pending.is_empty() {
        return;
    }
    if let Ok(mut counters) = COUNTERS.lock() {
        for item in &pending {
            match item {
                Pending::Moved { .. } => counters.moved += 1,
                Pending::Button { pressed: true, .. } => counters.pressed += 1,
                Pending::Button { pressed: false, .. } => counters.released += 1,
                Pending::WheelLines { .. } | Pending::WheelPixels { .. } => counters.wheel += 1,
                Pending::Gone => counters.gone += 1,
                Pending::Modifiers(_) => counters.modifiers += 1,
                Pending::Focus(_) => counters.focus += 1,
            }
        }
    }
    if let Ok(mut queue) = QUEUE.lock() {
        queue.extend(pending);
    }
}

// ---------------------------------------------------------------------------
// Pending → egui::Event
// ---------------------------------------------------------------------------

/// 1フレーム分を取り出して egui のイベント列にする。`Primitive::prepare` から呼ぶ。
///
/// `scale` は `Viewport::scale_factor()`。論理座標 → 物理画素の変換係数で、
/// `Offscreen` が物理画素で作られている以上ここで掛けないと座標が半分ずれる。
pub fn drain(scale: f32) -> (Vec<egui::Event>, egui::Modifiers) {
    let pending: Vec<Pending> = QUEUE
        .lock()
        .map(|mut queue| std::mem::take(&mut *queue))
        .unwrap_or_default();

    let mut modifiers = MODIFIERS.lock().map(|m| *m).unwrap_or_default();
    let mut events = Vec::with_capacity(pending.len());
    let mut focused = None;

    for item in pending {
        match item {
            Pending::Modifiers(new) => {
                modifiers = new;
                // egui-winit も modifiers 単体ではイベントを積まない。次の
                // pointer/key イベントに乗せるだけである。
            }
            Pending::Moved { x, y } => {
                events.push(egui::Event::PointerMoved(egui::pos2(x * scale, y * scale)));
            }
            Pending::Button {
                x,
                y,
                button,
                pressed,
            } => {
                events.push(egui::Event::PointerButton {
                    pos: egui::pos2(x * scale, y * scale),
                    button,
                    pressed,
                    modifiers,
                });
            }
            Pending::Gone => events.push(egui::Event::PointerGone),
            Pending::WheelLines { x, y } => events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(x, y),
                phase: egui::TouchPhase::Move,
                modifiers,
            }),
            Pending::WheelPixels { x, y } => events.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                // 画素は論理 point で来る。egui の Point 単位もまた論理 point なので
                // ここは scale を掛けない(掛けると2倍速く回る)。
                delta: egui::vec2(x, y),
                phase: egui::TouchPhase::Move,
                modifiers,
            }),
            Pending::Focus(focus) => {
                focused = Some(focus);
                events.push(egui::Event::WindowFocused(focus));
            }
        }
    }

    if let Ok(mut slot) = MODIFIERS.lock() {
        *slot = modifiers;
    }
    if let Ok(mut counters) = COUNTERS.lock() {
        counters.delivered += events.len() as u64;
    }
    let _ = focused;
    (events, modifiers)
}

// ---------------------------------------------------------------------------
// egui → iced(戻りの半分)
// ---------------------------------------------------------------------------

/// egui が出した platform output のうち、iced 側へ写せる物を受け取る。
pub fn observe_platform_output(cursor_icon: egui::CursorIcon, repaint_delay: Option<Duration>) {
    if let Ok(mut slot) = CURSOR_ICON.lock() {
        *slot = cursor_icon;
    }
    if let Ok(mut seen) = CURSOR_SEEN.lock() {
        if !seen.contains(&cursor_icon) {
            seen.push(cursor_icon);
        }
    }
    if let Ok(mut slot) = REPAINT_DELAY.lock() {
        *slot = repaint_delay;
    }
    if let Ok(mut seen) = REPAINT_SEEN.lock() {
        let key = repaint_delay.map_or(u128::MAX, |delay| delay.as_millis());
        if !seen.contains(&key) {
            seen.push(key);
        }
    }
}

pub fn cursor_icon() -> egui::CursorIcon {
    CURSOR_ICON.lock().map(|icon| *icon).unwrap_or_default()
}

pub fn cursor_icons_seen() -> Vec<egui::CursorIcon> {
    CURSOR_SEEN.lock().map(|seen| seen.clone()).unwrap_or_default()
}

pub fn repaint_delays_seen() -> Vec<u128> {
    REPAINT_SEEN.lock().map(|seen| seen.clone()).unwrap_or_default()
}

/// **(e) の本体**: egui の cursor icon を iced の `mouse::Interaction` へ写す。
///
/// 写せない物は `None` を返す。`Program::mouse_interaction` はここを見る。
pub fn to_iced_interaction(icon: egui::CursorIcon) -> Option<mouse::Interaction> {
    use egui::CursorIcon as E;
    use mouse::Interaction as I;
    Some(match icon {
        E::Default => I::Idle,
        E::None => I::Hidden,
        E::ContextMenu => I::ContextMenu,
        E::Help => I::Help,
        E::PointingHand => I::Pointer,
        E::Progress => I::Progress,
        E::Wait => I::Wait,
        E::Cell => I::Cell,
        E::Crosshair => I::Crosshair,
        E::Text => I::Text,
        E::Alias => I::Alias,
        E::Copy => I::Copy,
        E::Move => I::Move,
        E::NoDrop => I::NoDrop,
        E::NotAllowed => I::NotAllowed,
        E::Grab => I::Grab,
        E::Grabbing => I::Grabbing,
        E::AllScroll => I::AllScroll,
        E::ResizeHorizontal => I::ResizingHorizontally,
        E::ResizeVertical => I::ResizingVertically,
        E::ResizeNeSw => I::ResizingDiagonallyUp,
        E::ResizeNwSe => I::ResizingDiagonallyDown,
        E::ResizeColumn => I::ResizingColumn,
        E::ResizeRow => I::ResizingRow,
        E::ZoomIn => I::ZoomIn,
        E::ZoomOut => I::ZoomOut,
        // 8方向の辺/角リサイズは iced には2軸+2斜めしか無い。近い物へ丸める。
        E::ResizeEast | E::ResizeWest => I::ResizingHorizontally,
        E::ResizeNorth | E::ResizeSouth => I::ResizingVertically,
        E::ResizeNorthEast | E::ResizeSouthWest => I::ResizingDiagonallyUp,
        E::ResizeNorthWest | E::ResizeSouthEast => I::ResizingDiagonallyDown,
        // VerticalText には iced 側に対応が無い。写せない物としてここで落とす。
        E::VerticalText => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Rectangle {
        Rectangle {
            x: 20.0,
            y: 10.0,
            width: 200.0,
            height: 100.0,
        }
    }

    #[test]
    fn cursor_moved_is_relative_to_the_widget_origin() {
        let event = iced::Event::Mouse(mouse::Event::CursorMoved {
            position: iced::Point::new(70.0, 60.0),
        });
        let pending = translate(&event, bounds(), mouse::Cursor::Unavailable);
        assert_eq!(pending, vec![Pending::Moved { x: 50.0, y: 50.0 }]);
    }

    #[test]
    fn a_press_carries_the_position_and_is_preceded_by_a_move() {
        let cursor = mouse::Cursor::Available(iced::Point::new(120.0, 60.0));
        let event = iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        let pending = translate(&event, bounds(), cursor);
        assert_eq!(
            pending,
            vec![
                Pending::Moved { x: 100.0, y: 50.0 },
                Pending::Button {
                    x: 100.0,
                    y: 50.0,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                },
            ]
        );
    }

    #[test]
    fn a_press_without_a_known_cursor_produces_nothing() {
        let event = iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        assert!(translate(&event, bounds(), mouse::Cursor::Unavailable).is_empty());
    }

    #[test]
    fn wheel_keeps_its_unit() {
        let lines = iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 3.0 },
        });
        assert_eq!(
            translate(&lines, bounds(), mouse::Cursor::Unavailable),
            vec![Pending::WheelLines { x: 0.0, y: 3.0 }]
        );
        let pixels = iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 1.0, y: -2.0 },
        });
        assert_eq!(
            translate(&pixels, bounds(), mouse::Cursor::Unavailable),
            vec![Pending::WheelPixels { x: 1.0, y: -2.0 }]
        );
    }

    /// (e) の主張は「写せる/写せない」の two-way なので、両方を固定しておく。
    #[test]
    fn cursor_icons_map_to_iced_except_one() {
        assert_eq!(
            to_iced_interaction(egui::CursorIcon::Grabbing),
            Some(mouse::Interaction::Grabbing)
        );
        assert_eq!(
            to_iced_interaction(egui::CursorIcon::Crosshair),
            Some(mouse::Interaction::Crosshair)
        );
        assert_eq!(to_iced_interaction(egui::CursorIcon::VerticalText), None);
    }
}
