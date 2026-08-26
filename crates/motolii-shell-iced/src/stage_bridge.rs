//! Stage 島の入力ブリッジ — iced の生事象を **toolkit 中立な** stage 入力へ写す。
//!
//! `spikes/iced-rerun-embed-probe/probe/src/bridge.rs`(442行)の製品化だが、
//! 翻訳先が違う。spike は `egui::Event` へ直接訳した — 製品では egui はこの crate に
//! 入れない(柵: `ui_toolkit_dep_policy.rs` の `UI_TOOLKIT_CRATE_ALLOWLIST` に
//! この crate は**居ない**)。代わりに `motolii_ui::rerun_stage::EmbeddedSpatialStage` が
//! RN native surface 向けに既に持っている **toolkit 中立の口**
//! (`pointer(phase, button, modifiers, x, y)` / `scroll(…)`)へ写す。
//! egui への翻訳はあちら(motolii-ui)の中に1箇所だけ在り、ここでは増やさない。
//!
//! ## spike から持ち越した2つの座標規約
//!
//! 1. **原点は widget の左上**。iced は `Program::update` に widget の `bounds`
//!    (論理座標)をくれるので引く。
//! 2. **キューには論理座標のまま積む**。stage の offscreen は物理画素で建つが、
//!    scale factor は `Primitive::prepare` の `Viewport` にしか無い。掛けるのは
//!    取り出す側([`crate::stage_island`] の prepare)である。
//!
//! ## 押下の直前に Move を挟まない(spike との差)
//!
//! spike の bridge は egui が「press の pos を drag の原点にする」ため押下前に
//! `Moved` を挟んだ。`EmbeddedSpatialStage::pointer` は**1呼びごとに**
//! `PointerMoved(position)` を先に積む実装なので、ここで挟むと二重になる。

use iced::mouse;
use iced::Rectangle;

use motolii_ui::rerun_stage::{PointerPhase, StagePointerButton};

/// 行単位ホイール1ノッチぶんの point 換算。
///
/// iced の `ScrollDelta::Lines` に対応する物が `EmbeddedSpatialStage::scroll`
/// (point の delta)側に無いので、この橋が決める。egui-winit の既定
/// (1行 = 数十 point)に合わせた値で、トラックパッド(`Pixels`)はこの定数を通らない。
pub const POINTS_PER_SCROLL_LINE: f32 = 40.0;

/// modifiers のビット表現。`EmbeddedSpatialStage::pointer` / `scroll` の
/// `modifiers: u32` と**同じ読み方**(あちらの `stage_modifiers` が正)。
pub fn modifiers_bits(modifiers: iced::keyboard::Modifiers) -> u32 {
    let mut bits = 0;
    if modifiers.shift() {
        bits |= 1;
    }
    if modifiers.control() {
        bits |= 2;
    }
    if modifiers.alt() {
        bits |= 4;
    }
    if modifiers.logo() {
        bits |= 8;
    }
    bits
}

/// この橋を渡る1件。座標は widget 左上原点の**論理**座標。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StageInput {
    /// pointer の1手(移動・押下・離し・離脱)。
    Pointer {
        phase: PointerPhase,
        button: StagePointerButton,
        x: f32,
        y: f32,
    },
    /// ホイール / トラックパッドのスクロール(point 換算済み)。
    Scroll {
        delta_x: f32,
        delta_y: f32,
        x: f32,
        y: f32,
    },
    /// modifiers が変わった。次の手に乗る。
    Modifiers(u32),
}

/// iced のボタン → stage のボタン。`Back`/`Forward`/`Other` は stage 側に
/// 対応が無いので捨てる(spike は egui の Extra1/2 へ写せたが、中立 API は3値)。
fn button_of(button: mouse::Button) -> Option<StagePointerButton> {
    match button {
        mouse::Button::Left => Some(StagePointerButton::Primary),
        mouse::Button::Right => Some(StagePointerButton::Secondary),
        mouse::Button::Middle => Some(StagePointerButton::Middle),
        mouse::Button::Back | mouse::Button::Forward | mouse::Button::Other(_) => None,
    }
}

/// **翻訳の本体**。`Program::update` から呼ぶ。
///
/// 返り値が空なら「この橋には関係のない事象」。ドラッグ中はカーソルが widget の
/// 外へ出ても座標を送り続ける(spike と同じ。egui-winit の振る舞いで、
/// そうしないと枠際で orbit が止まる)。
pub fn translate(event: &iced::Event, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<StageInput> {
    let local = |point: iced::Point| (point.x - bounds.x, point.y - bounds.y);
    let cursor_local = || cursor.position().map(local);

    match event {
        iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
            let (x, y) = local(*position);
            vec![StageInput::Pointer {
                phase: PointerPhase::Move,
                button: StagePointerButton::Primary,
                x,
                y,
            }]
        }
        iced::Event::Mouse(mouse::Event::CursorLeft) => vec![StageInput::Pointer {
            phase: PointerPhase::Cancel,
            button: StagePointerButton::Primary,
            // 位置は意味を持たない(Cancel は「もう何も指していない」)。取り出す側が
            // 直近の位置に差し替える。
            x: f32::NAN,
            y: f32::NAN,
        }],
        iced::Event::Mouse(mouse::Event::ButtonPressed(button)) => {
            let (Some(button), Some((x, y))) = (button_of(*button), cursor_local()) else {
                return Vec::new();
            };
            vec![StageInput::Pointer {
                phase: PointerPhase::Down,
                button,
                x,
                y,
            }]
        }
        iced::Event::Mouse(mouse::Event::ButtonReleased(button)) => {
            let (Some(button), Some((x, y))) = (button_of(*button), cursor_local()) else {
                return Vec::new();
            };
            vec![StageInput::Pointer {
                phase: PointerPhase::Up,
                button,
                x,
                y,
            }]
        }
        iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            let Some((x, y)) = cursor_local() else {
                return Vec::new();
            };
            let (delta_x, delta_y) = match delta {
                mouse::ScrollDelta::Lines { x, y } => {
                    (x * POINTS_PER_SCROLL_LINE, y * POINTS_PER_SCROLL_LINE)
                }
                mouse::ScrollDelta::Pixels { x, y } => (*x, *y),
            };
            vec![StageInput::Scroll {
                delta_x,
                delta_y,
                x,
                y,
            }]
        }
        iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            vec![StageInput::Modifiers(modifiers_bits(*modifiers))]
        }
        // キーボード本体・IME・タッチは M-2 の範囲外(spike README 11 番の実測どおり
        // shader widget には IME を要求する口も無い)。
        _ => Vec::new(),
    }
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
        assert_eq!(
            translate(&event, bounds(), mouse::Cursor::Unavailable),
            vec![StageInput::Pointer {
                phase: PointerPhase::Move,
                button: StagePointerButton::Primary,
                x: 50.0,
                y: 50.0,
            }]
        );
    }

    #[test]
    fn a_press_carries_the_cursor_position_in_widget_space() {
        let cursor = mouse::Cursor::Available(iced::Point::new(120.0, 60.0));
        let event = iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        assert_eq!(
            translate(&event, bounds(), cursor),
            vec![StageInput::Pointer {
                phase: PointerPhase::Down,
                button: StagePointerButton::Primary,
                x: 100.0,
                y: 50.0,
            }]
        );
    }

    #[test]
    fn a_press_without_a_known_cursor_produces_nothing() {
        let event = iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
        assert!(translate(&event, bounds(), mouse::Cursor::Unavailable).is_empty());
    }

    /// 行ホイールは point へ換算され、トラックパッドの pixel はそのまま通る。
    /// spike の実測「Pixels に scale を掛けると2倍速く回る」の教訓ごと固定する。
    #[test]
    fn wheel_lines_become_points_and_pixels_pass_through() {
        let cursor = mouse::Cursor::Available(iced::Point::new(120.0, 60.0));
        let lines = iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: 3.0 },
        });
        assert_eq!(
            translate(&lines, bounds(), cursor),
            vec![StageInput::Scroll {
                delta_x: 0.0,
                delta_y: 3.0 * POINTS_PER_SCROLL_LINE,
                x: 100.0,
                y: 50.0,
            }]
        );
        let pixels = iced::Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 1.0, y: -2.0 },
        });
        assert_eq!(
            translate(&pixels, bounds(), cursor),
            vec![StageInput::Scroll {
                delta_x: 1.0,
                delta_y: -2.0,
                x: 100.0,
                y: 50.0,
            }]
        );
    }

    /// 窓からカーソルが出たら Cancel(= stage 側では PointerGone)。
    #[test]
    fn leaving_the_window_cancels_the_pointer() {
        let event = iced::Event::Mouse(mouse::Event::CursorLeft);
        let translated = translate(&event, bounds(), mouse::Cursor::Unavailable);
        assert!(matches!(
            translated.as_slice(),
            [StageInput::Pointer {
                phase: PointerPhase::Cancel,
                ..
            }]
        ));
    }

    /// modifiers のビット表現は `EmbeddedSpatialStage` 側の読み方と同じ。
    #[test]
    fn modifiers_bits_match_the_stage_reading() {
        use iced::keyboard::Modifiers;
        assert_eq!(modifiers_bits(Modifiers::SHIFT), 1);
        assert_eq!(modifiers_bits(Modifiers::CTRL), 2);
        assert_eq!(modifiers_bits(Modifiers::ALT), 4);
        assert_eq!(modifiers_bits(Modifiers::LOGO), 8);
        assert_eq!(modifiers_bits(Modifiers::SHIFT | Modifiers::LOGO), 9);
    }
}
