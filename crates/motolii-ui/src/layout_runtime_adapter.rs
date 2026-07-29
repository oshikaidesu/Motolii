//! U1a-2で仕様許可された単一のraw toolkit input adapter。

use crate::input_router::{ImeGateState, SafetyInterrupt};
use crate::layout::{SeparatorAction, SplitAxis};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum StageDropTerminal {
    Commit { ndc: [f64; 2] },
    Cancel,
}

pub(crate) fn read_stage_drop_terminal(
    ui: &egui::Ui,
    stage_rect: egui::Rect,
    ime_gate: ImeGateState,
) -> Option<StageDropTerminal> {
    ui.input(|input| {
        let interrupted = input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::PointerGone | egui::Event::WindowFocused(false)
            )
        }) || (ime_gate == ImeGateState::Inactive && input.key_pressed(egui::Key::Escape));
        let released_at = input.events.iter().rev().find_map(|event| match event {
            egui::Event::PointerButton {
                pos,
                pressed: false,
                ..
            } => Some(*pos),
            _ => None,
        });
        resolve_stage_drop_terminal(
            stage_rect,
            released_at,
            released_at.is_some(),
            interrupted,
        )
    })
}

pub(crate) fn read_safety_interrupt(ui: &egui::Ui) -> Option<SafetyInterrupt> {
    ui.input(|input| {
        input.events.iter().find_map(|event| match event {
            egui::Event::PointerGone => Some(SafetyInterrupt::PointerCaptureLost),
            egui::Event::WindowFocused(false) => Some(SafetyInterrupt::WindowFocusLost),
            _ => None,
        })
    })
}

pub(crate) fn read_layout_cancel(
    ui: &egui::Ui,
    gesture_in_flight: bool,
    ime_gate: ImeGateState,
) -> bool {
    let escape_pressed = ui.input(|input| input.key_pressed(egui::Key::Escape));
    resolve_layout_cancel(gesture_in_flight, ime_gate, escape_pressed)
}

pub(crate) fn read_separator_action(
    ui: &egui::Ui,
    response: &egui::Response,
    axis: SplitAxis,
    ime_gate: ImeGateState,
) -> Option<SeparatorAction> {
    let pressed = ui.input(|input| {
        [
            (egui::Key::Home, SeparatorKey::Home),
            (egui::Key::Escape, SeparatorKey::Escape),
            (egui::Key::ArrowLeft, SeparatorKey::ArrowLeft),
            (egui::Key::ArrowRight, SeparatorKey::ArrowRight),
            (egui::Key::ArrowUp, SeparatorKey::ArrowUp),
            (egui::Key::ArrowDown, SeparatorKey::ArrowDown),
        ]
        .into_iter()
        .find_map(|(key, output)| input.key_pressed(key).then_some(output))
    });
    resolve_separator_input(
        axis,
        response.has_focus(),
        response.double_clicked(),
        ime_gate,
        pressed,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeparatorKey {
    Home,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

fn resolve_separator_input(
    axis: SplitAxis,
    focused: bool,
    double_clicked: bool,
    ime_gate: ImeGateState,
    pressed: Option<SeparatorKey>,
) -> Option<SeparatorAction> {
    if double_clicked {
        return Some(SeparatorAction::Reset);
    }
    if !focused || ime_gate == ImeGateState::PreeditActive {
        return None;
    }
    match (axis, pressed?) {
        (_, SeparatorKey::Home) => Some(SeparatorAction::Reset),
        (_, SeparatorKey::Escape) => Some(SeparatorAction::Cancel),
        (SplitAxis::Horizontal, SeparatorKey::ArrowLeft)
        | (SplitAxis::Vertical, SeparatorKey::ArrowUp) => Some(SeparatorAction::DecreaseLeading),
        (SplitAxis::Horizontal, SeparatorKey::ArrowRight)
        | (SplitAxis::Vertical, SeparatorKey::ArrowDown) => Some(SeparatorAction::IncreaseLeading),
        _ => None,
    }
}

fn resolve_layout_cancel(
    gesture_in_flight: bool,
    ime_gate: ImeGateState,
    escape_pressed: bool,
) -> bool {
    gesture_in_flight && ime_gate == ImeGateState::Inactive && escape_pressed
}

fn resolve_stage_drop_terminal(
    stage_rect: egui::Rect,
    pointer: Option<egui::Pos2>,
    released: bool,
    interrupted: bool,
) -> Option<StageDropTerminal> {
    if interrupted {
        return Some(StageDropTerminal::Cancel);
    }
    if !released {
        return None;
    }
    let pointer = pointer?;
    if !stage_rect.contains(pointer) || stage_rect.width() <= 0.0 || stage_rect.height() <= 0.0 {
        return Some(StageDropTerminal::Cancel);
    }
    let egui::Pos2 {
        x: pointer_x,
        y: pointer_y,
    } = pointer;
    let u = f64::from((pointer_x - stage_rect.left()) / stage_rect.width());
    let v = f64::from((pointer_y - stage_rect.top()) / stage_rect.height());
    Some(StageDropTerminal::Commit {
        ndc: [u.mul_add(2.0, -1.0), 1.0 - 2.0 * v],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_rejects_unrelated_arrows() {
        assert_eq!(
            resolve_separator_input(
                SplitAxis::Horizontal,
                true,
                false,
                ImeGateState::Inactive,
                Some(SeparatorKey::ArrowUp)
            ),
            None
        );
        assert_eq!(
            resolve_separator_input(
                SplitAxis::Vertical,
                true,
                false,
                ImeGateState::Inactive,
                Some(SeparatorKey::ArrowRight)
            ),
            None
        );
    }

    #[test]
    fn home_and_escape_are_private_layout_actions() {
        for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
            assert_eq!(
                resolve_separator_input(
                    axis,
                    true,
                    false,
                    ImeGateState::Inactive,
                    Some(SeparatorKey::Home)
                ),
                Some(SeparatorAction::Reset)
            );
            assert_eq!(
                resolve_separator_input(
                    axis,
                    true,
                    false,
                    ImeGateState::Inactive,
                    Some(SeparatorKey::Escape)
                ),
                Some(SeparatorAction::Cancel)
            );
        }
    }

    #[test]
    fn preedit_owns_separator_keys_but_not_pointer_reset() {
        assert_eq!(
            resolve_separator_input(
                SplitAxis::Horizontal,
                true,
                false,
                ImeGateState::PreeditActive,
                Some(SeparatorKey::Escape)
            ),
            None
        );
        assert_eq!(
            resolve_separator_input(
                SplitAxis::Horizontal,
                true,
                true,
                ImeGateState::PreeditActive,
                None
            ),
            Some(SeparatorAction::Reset)
        );
    }

    #[test]
    fn global_escape_gate_requires_an_active_layout_gesture_and_inactive_ime() {
        assert!(!resolve_layout_cancel(false, ImeGateState::Inactive, true));
        assert!(!resolve_layout_cancel(
            true,
            ImeGateState::PreeditActive,
            true
        ));
        assert!(resolve_layout_cancel(true, ImeGateState::Inactive, true));
    }

    #[test]
    fn stage_drop_normalizes_to_y_up_ndc_and_cancels_outside() {
        let rect = egui::Rect::from_min_max(egui::pos2(10.0, 20.0), egui::pos2(210.0, 120.0));
        assert_eq!(
            resolve_stage_drop_terminal(rect, Some(egui::pos2(110.0, 45.0)), true, false),
            Some(StageDropTerminal::Commit { ndc: [0.0, 0.5] })
        );
        assert_eq!(
            resolve_stage_drop_terminal(rect, Some(egui::pos2(9.0, 45.0)), true, false),
            Some(StageDropTerminal::Cancel)
        );
        assert_eq!(
            resolve_stage_drop_terminal(rect, Some(egui::pos2(110.0, 45.0)), false, true),
            Some(StageDropTerminal::Cancel)
        );
    }
}
