//! egui eventをTimelineの意図へ翻訳する。Documentにも描画にも触れない。

use egui::{Pos2, Vec2};
use motolii_core::RationalTime;
use motolii_doc::{KeyframeId, LayerId};

use super::clip_band::hit_at;
use super::geometry::TimelineGeometry;
use super::rows::TimelineRow;
use crate::timeline_projection::TimelineProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelinePointerPhase {
    Down,
    Drag,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineCommand {
    Escape,
    Delete,
    Backspace,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Duplicate,
    SelectAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimelineWheelAction {
    Zoom,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EguiTimelineHit {
    Key { layer: LayerId, key: KeyframeId },
    Body { layer: LayerId },
    Left { layer: LayerId },
    Right { layer: LayerId },
    None,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TimelineIntent {
    Pointer {
        phase: TimelinePointerPhase,
        position: Pos2,
        time: Option<RationalTime>,
        hit: EguiTimelineHit,
        modifiers: egui::Modifiers,
    },
    Command {
        command: TimelineCommand,
        modifiers: egui::Modifiers,
    },
    Wheel {
        action: TimelineWheelAction,
        delta: Vec2,
        anchor: Option<Pos2>,
        modifiers: egui::Modifiers,
    },
}

pub(super) fn collect_intents(
    ui: &egui::Ui,
    response: &egui::Response,
    geometry: &TimelineGeometry,
    rows: &[TimelineRow],
    projection: Option<&TimelineProjection>,
) -> Vec<TimelineIntent> {
    let events = ui.input(|input| input.events.clone());
    let modifiers = ui.input(|input| input.modifiers);
    let mut intents = Vec::new();

    for event in events {
        match event {
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            } if geometry.rect.contains(pos) || (!pressed && response.dragged()) => {
                intents.push(TimelineIntent::Pointer {
                    phase: if pressed {
                        TimelinePointerPhase::Down
                    } else {
                        TimelinePointerPhase::Up
                    },
                    position: pos,
                    time: geometry.time_at(pos),
                    hit: hit_at(pos, geometry, rows, projection),
                    modifiers,
                });
            }
            egui::Event::PointerMoved(pos) if response.dragged() && geometry.rect.contains(pos) => {
                intents.push(TimelineIntent::Pointer {
                    phase: TimelinePointerPhase::Drag,
                    position: pos,
                    time: geometry.time_at(pos),
                    hit: hit_at(pos, geometry, rows, projection),
                    modifiers,
                });
            }
            egui::Event::MouseWheel {
                delta, modifiers, ..
            } if response.hovered() => {
                let anchor = response.hover_pos();
                intents.push(TimelineIntent::Wheel {
                    action: if modifiers.command || modifiers.ctrl {
                        TimelineWheelAction::Zoom
                    } else {
                        TimelineWheelAction::Pan
                    },
                    delta,
                    anchor,
                    modifiers,
                });
            }
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                if let Some(command) = timeline_command_for_key(key, modifiers) {
                    intents.push(TimelineIntent::Command { command, modifiers });
                }
            }
            _ => {}
        }
    }

    intents
}

pub(crate) fn timeline_command_for_key(
    key: egui::Key,
    modifiers: egui::Modifiers,
) -> Option<TimelineCommand> {
    let has_no_modifier =
        !(modifiers.command || modifiers.ctrl || modifiers.alt || modifiers.shift);
    match key {
        egui::Key::Escape if has_no_modifier => Some(TimelineCommand::Escape),
        egui::Key::Delete if has_no_modifier => Some(TimelineCommand::Delete),
        egui::Key::Backspace if has_no_modifier => Some(TimelineCommand::Backspace),
        egui::Key::Z if modifiers.command => Some(if modifiers.shift {
            TimelineCommand::Redo
        } else {
            TimelineCommand::Undo
        }),
        egui::Key::C if modifiers.command => Some(TimelineCommand::Copy),
        egui::Key::X if modifiers.command => Some(TimelineCommand::Cut),
        egui::Key::V if modifiers.command => Some(TimelineCommand::Paste),
        egui::Key::D if modifiers.command => Some(TimelineCommand::Duplicate),
        egui::Key::A if modifiers.command => Some(TimelineCommand::SelectAll),
        _ => None,
    }
}
