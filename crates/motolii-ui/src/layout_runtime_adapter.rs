//! U1a-2で仕様許可された単一のraw toolkit input adapter。

use crate::input_router::{ImeGateState, SafetyInterrupt};
use crate::layout::{SeparatorAction, SplitAxis};

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
}
