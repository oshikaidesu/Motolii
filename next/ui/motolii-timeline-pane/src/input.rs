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
//!
//! ## 単一クリップの move/trim(第2波T2、正典 §2)
//!
//! bar を掴んだ瞬間の座標だけで [`super::hit::classify_bar_part`] を1回呼び、
//! `Body`/`EdgeIn`/`EdgeOut` を確定してから [`Message::BarGrabbed`] を
//! 出す(正典 §1「判定は押した瞬間の座標」)。**ロック判定・スナップ・clamp・
//! `Intent::SetTiming` はここでは一切やらない** — Document を読めるのは
//! `Shell::update` だけなので、ここは「掴んだ座標」「今のポインタの frame と
//! 画面スケール」を運ぶだけの翻訳に徹する(モジュール doc の設計方針そのもの)。
//! 実際の preview 計算(スナップ込み)は `clip_gesture` の純関数を
//! `Shell::update` 側から呼ぶ(`lib.rs::continue_timeline_drag`)。

use iced::mouse;
use iced::widget::canvas;
use iced::{Point, Rectangle};

use crate::Message;

use super::hit::{bar_span_x, classify_bar_part, hit_test, BarPart, Hit};
use super::lane_bar::{self, Glyph};
use super::projection::frame_at_x;
use super::TimelinePane;

/// canvas の drag 状態。**Document でも Session でもない、widget 内だけの一時状態**
/// (iced の `slider` 等が持つ内部 drag state と同格)。書ける物は持っていない —
/// ここが持つのは「今どの種類のドラッグが進行中か」だけで、実際の書き込みは
/// 全て [`Message`] 経由で `Shell::update` に委ねる。
#[derive(Default)]
pub struct Interaction {
    drag: Option<DragKind>,
}

/// 進行中のドラッグの種類。scrub(ルーラー/空白部)と clip(bar の move/trim)は
/// 別腕 — release 時に出す `Message` が違う(scrub は move ごとに `ScrubTo` を
/// 出し切っているので release は無音、clip は release で初めて確定 intent が
/// 要る、`Message::DragReleased`)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum DragKind {
    Scrub,
    Clip,
}

/// レーンバーの glyph → Message。M/S/L それぞれ独立した `Intent::SetAttrs` を
/// 1回叩く(`crate::Shell` 側の `toggle_layer_hidden`/`toggle_layer_solo`/
/// `toggle_layer_lock` — locked な行への hidden/solo 書き込みは Document 側の
/// `check`/`SetAttrs` 腕が理由つきで拒む、`locked` 自身の解除だけは常に通す)。
fn glyph_message(id: motolii_store::LayerId, glyph: Glyph) -> Message {
    match glyph {
        Glyph::Mute => Message::ToggleMute(id),
        Glyph::Solo => Message::ToggleSolo(id),
        Glyph::Lock => Message::ToggleLock(id),
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
                pane.param_row_height(),
                pane.property_rows.len(),
                pane.selected_row_index,
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
                pane.param_row_height(),
                pane.property_rows.len(),
                pane.selected_row_index,
            ) {
                // **判定は押した瞬間の座標だけ**(正典 §1) — `classify_bar_part`
                // に渡すのは今この1回の `clip_point.x` で、以降の move では
                // 呼び直さない(`Interaction` は `Body`/`Edge*` を覚えない —
                // 呼び手の `Shell` 側 `TimelineDragState::part` が正本を持つ)。
                Hit::Bar(id) => {
                    let Some(row) = pane.rows.iter().find(|row| row.id == id) else {
                        return None;
                    };
                    let (start_x, end_x) = bar_span_x(row, clip_width, pane.duration_frames);
                    let part = classify_bar_part(clip_point.x, start_x, end_x);
                    let at_frame = frame_at_x(clip_point.x, clip_width, pane.duration_frames);
                    state.drag = Some(DragKind::Clip);
                    Some(
                        canvas::Action::publish(Message::BarGrabbed {
                            layer: id,
                            part,
                            at_frame,
                        })
                        .and_capture(),
                    )
                }
                Hit::Blank => {
                    state.drag = Some(DragKind::Scrub);
                    let frame = frame_at_x(clip_point.x, clip_width, pane.duration_frames);
                    Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
                }
            }
        }
        // 右クリック = ドラッグ中のクリップ move/trim をキャンセル(正典 §2 Esc・
        // 裁定151「キャンセルの一般化」— Esc は window 全体の subscription
        // (`Message::EscapePressed`)が別経路で既に拾うので、ここは右クリック
        // だけを扱う)。scrub 中・非ドラッグ中は何もしない(scrub に右クリックの
        // 意味はまだ無い — 発明しない)。
        mouse::Event::ButtonPressed(mouse::Button::Right) => {
            if state.drag == Some(DragKind::Clip) {
                state.drag = None;
                Some(canvas::Action::publish(Message::DragCancelled).and_capture())
            } else {
                None
            }
        }
        mouse::Event::CursorMoved { .. } => match state.drag {
            Some(DragKind::Scrub) => {
                let position = cursor.position_in(bounds)?;
                let frame = frame_at_x(position.x - rail_width, clip_width, pane.duration_frames);
                Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
            }
            Some(DragKind::Clip) => {
                let position = cursor.position_in(bounds)?;
                let at_frame = frame_at_x(position.x - rail_width, clip_width, pane.duration_frames);
                // px/frame の換算は canvas 側でしか持てない実測値(窓幅依存) —
                // Shell は自分の窓幅を知らないので、スナップの画面距離しきい値
                // (`clip_gesture::SNAP_PX`)をフレームへ直すのに要るこの1個だけを
                // 運ぶ(`Shell` 側は届いた値をそのまま使うだけ)。
                let px_per_frame = if pane.duration_frames > 0 {
                    clip_width / pane.duration_frames as f32
                } else {
                    0.0
                };
                Some(
                    canvas::Action::publish(Message::DragMoved {
                        at_frame,
                        px_per_frame,
                    })
                    .and_capture(),
                )
            }
            None => None,
        },
        mouse::Event::ButtonReleased(mouse::Button::Left) => match state.drag.take() {
            Some(DragKind::Scrub) => Some(canvas::Action::capture()),
            Some(DragKind::Clip) => {
                Some(canvas::Action::publish(Message::DragReleased).and_capture())
            }
            None => None,
        },
        _ => None,
    }
}

/// カーソル形状は意味の予告(正典 §5.5)。**触れそうな物には必ず予告を出す**
/// (Q0「触れそうで触れない物は不合格」の逆写像): 端=`ResizingHorizontally` /
/// 本体 hover=`Grab` / ドラッグ中=`Grabbing` / ロック行=`NotAllowed`。
pub(crate) fn mouse_interaction(
    pane: &TimelinePane,
    state: &Interaction,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> mouse::Interaction {
    if state.drag.is_some() {
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
        pane.param_row_height(),
        pane.property_rows.len(),
        pane.selected_row_index,
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
        pane.param_row_height(),
        pane.property_rows.len(),
        pane.selected_row_index,
    ) {
        Hit::Bar(id) => {
            let Some(row) = pane.rows.iter().find(|row| row.id == id) else {
                return mouse::Interaction::default();
            };
            // ロック行は掴めない予告(正典 §5.5・M13)。端/本体の区別より優先 —
            // 触れない物には触れなさそうな形を出す。
            if row.locked {
                return mouse::Interaction::NotAllowed;
            }
            let (start_x, end_x) = bar_span_x(row, clip_width, pane.duration_frames);
            match classify_bar_part(clip_point.x, start_x, end_x) {
                BarPart::Body => mouse::Interaction::Grab,
                BarPart::EdgeIn | BarPart::EdgeOut => mouse::Interaction::ResizingHorizontally,
            }
        }
        Hit::Blank => mouse::Interaction::default(),
    }
}
