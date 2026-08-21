//! canvas の入力(`update`/`mouse_interaction`)と drag 状態(`Interaction`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)から委譲
//! されるだけの純粋な translation — Document/Session を直接書かない
//! ([`Message`] 経由で `Shell::update` に委ねる、`super` モジュール doc 参照)。
//!
//! **座標の振り分け順**(裁定147・EXACT TARGET 3): まずレーンバー
//! (`super::lane_bar::hit_test`、`x < rail_width`)を試し、当たらなければ
//! クリップ面(`super::hit::hit_test`)へ回す。クリップ面側へは常に
//! `rail_width` を引いたローカル座標を渡す — `projection::frame_to_x`/
//! `frame_at_x` 自体は汚さない(`super::canvas` と同じ「呼び出し側で足す」
//! 約束の裏返し=呼び出し側で引く)。

use iced::mouse;
use iced::widget::canvas;
use iced::{Point, Rectangle};

use crate::Message;

use super::hit::{hit_test, Hit};
use super::lane_bar::{self, Glyph};
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

/// レーンバーの glyph → Message。M/S/L それぞれ独立した `Intent::SetAttrs` を
/// 1回叩く(`crate::Shell` 側の `toggle_layer_hidden`/`toggle_layer_solo`/
/// `toggle_layer_lock` — locked な行への hidden/solo 書き込みは Document 側の
/// `check`/`SetAttrs` 腕が理由つきで拒む、`locked` 自身の解除だけは常に通す)。
fn glyph_message(id: motolii_store::LayerId, glyph: Glyph) -> Message {
    match glyph {
        Glyph::Mute => Message::LaneBarToggleMute(id),
        Glyph::Solo => Message::LaneBarToggleSolo(id),
        Glyph::Lock => Message::LaneBarToggleLock(id),
    }
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
    let rail_width = pane.rail_width();
    let clip_width = (bounds.width - rail_width).max(0.0);
    match mouse_event {
        mouse::Event::ButtonPressed(mouse::Button::Left) => {
            let position = cursor.position_in(bounds)?;

            if let Some(hit) = lane_bar::hit_test(
                position,
                &pane.rows,
                pane.ruler_height(),
                pane.dims.row_height,
                rail_width,
                &pane.dims,
            ) {
                return match hit {
                    lane_bar::Hit::Row(id) => {
                        Some(canvas::Action::publish(Message::Select(id)).and_capture())
                    }
                    lane_bar::Hit::Glyph(id, glyph) => {
                        Some(canvas::Action::publish(glyph_message(id, glyph)).and_capture())
                    }
                };
            }

            let clip_point = Point::new(position.x - rail_width, position.y);
            match hit_test(
                clip_point,
                &pane.rows,
                pane.ruler_height(),
                pane.dims.row_height,
                clip_width,
                pane.duration_frames,
            ) {
                Hit::Bar(id) => Some(canvas::Action::publish(Message::Select(id)).and_capture()),
                Hit::Blank => {
                    state.dragging = true;
                    let frame = frame_at_x(clip_point.x, clip_width, pane.duration_frames);
                    Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
                }
            }
        }
        mouse::Event::CursorMoved { .. } => {
            if !state.dragging {
                return None;
            }
            let position = cursor.position_in(bounds)?;
            let frame = frame_at_x(position.x - rail_width, clip_width, pane.duration_frames);
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
    let rail_width = pane.rail_width();
    let clip_width = (bounds.width - rail_width).max(0.0);

    if lane_bar::hit_test(
        position,
        &pane.rows,
        pane.ruler_height(),
        pane.dims.row_height,
        rail_width,
        &pane.dims,
    )
    .is_some()
    {
        return mouse::Interaction::Pointer;
    }

    let clip_point = Point::new(position.x - rail_width, position.y);
    match hit_test(
        clip_point,
        &pane.rows,
        pane.ruler_height(),
        pane.dims.row_height,
        clip_width,
        pane.duration_frames,
    ) {
        Hit::Bar(_) => mouse::Interaction::Pointer,
        Hit::Blank => mouse::Interaction::default(),
    }
}
