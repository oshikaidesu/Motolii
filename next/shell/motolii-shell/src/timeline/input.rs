//! canvas の入力(`update`/`mouse_interaction`)と drag 状態(`Interaction`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)から委譲
//! されるだけの純粋な translation — Document/Session を直接書かない
//! ([`Message`] 経由で `Shell::update` に委ねる、`super` モジュール doc 参照)。

use iced::mouse;
use iced::widget::canvas;
use iced::Rectangle;

use crate::Message;

use super::hit::{hit_test, Hit};
use super::projection::frame_at_x;
use super::TimelinePane;

/// canvas の drag 状態。**Document でも Session でもない、widget 内だけの一時状態**
/// (iced の `slider` 等が持つ内部 drag state と同格)。書ける物は持っていない —
/// ここが持つ真偽値は「今 button が下がっているか」だけで、実際の書き込みは
/// 全て [`Message`] 経由で `Shell::update` に委ねる。
#[derive(Default)]
pub struct Interaction {
    dragging: bool,
}

pub(crate) fn update(
    pane: &TimelinePane,
    state: &mut Interaction,
    event: &canvas::Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<canvas::Action<Message>> {
    let canvas::Event::Mouse(mouse_event) = event else {
        return None;
    };
    match mouse_event {
        mouse::Event::ButtonPressed(mouse::Button::Left) => {
            let position = cursor.position_in(bounds)?;
            match hit_test(
                position,
                &pane.rows,
                pane.ruler_height(),
                pane.dims.row_height,
                bounds.width,
                pane.duration_frames,
            ) {
                Hit::Bar(id) => Some(canvas::Action::publish(Message::Select(id)).and_capture()),
                Hit::Blank => {
                    state.dragging = true;
                    let frame = frame_at_x(position.x, bounds.width, pane.duration_frames);
                    Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
                }
            }
        }
        mouse::Event::CursorMoved { .. } => {
            if !state.dragging {
                return None;
            }
            let position = cursor.position_in(bounds)?;
            let frame = frame_at_x(position.x, bounds.width, pane.duration_frames);
            Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
        }
        mouse::Event::ButtonReleased(mouse::Button::Left) => {
            if state.dragging {
                state.dragging = false;
                Some(canvas::Action::capture())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn mouse_interaction(
    pane: &TimelinePane,
    state: &Interaction,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> mouse::Interaction {
    if state.dragging {
        return mouse::Interaction::Grabbing;
    }
    let Some(position) = cursor.position_in(bounds) else {
        return mouse::Interaction::default();
    };
    match hit_test(
        position,
        &pane.rows,
        pane.ruler_height(),
        pane.dims.row_height,
        bounds.width,
        pane.duration_frames,
    ) {
        Hit::Bar(_) => mouse::Interaction::Pointer,
        Hit::Blank => mouse::Interaction::default(),
    }
}
