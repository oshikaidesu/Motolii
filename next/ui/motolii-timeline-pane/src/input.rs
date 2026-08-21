//! canvas の入力(`update`/`mouse_interaction`)と drag 状態(`Interaction`)。
//! `TimelinePane`(`super::TimelinePane` の `canvas::Program` impl)から委譲
//! されるだけの純粋な translation — Document/Session を直接書かない
//! ([`Message`] 経由で `Shell::update` に委ねる、`super` モジュール doc 参照)。
//!
//! **TL-arch Phase 1**(2026-08-22): レーンバー(`super::lane_bar::hit_test`
//! への振り分け)はここから撤去した — rail は `super::rail::view` の実
//! widget になり、行選択・M/S/L は iced 標準の `mouse_area`/`button` の
//! event capture が担う(この canvas まで届く頃には rail 上のクリックは
//! 存在しない、`super::rail` モジュール doc「gesture」節参照)。この
//! canvas の `bounds` はもう時間場だけなので、`super::hit::hit_test` へは
//! 受け取った座標をそのまま渡す(`rail_width` を引く必要はもう無い —
//! `super::canvas` 冒頭のモジュール doc 参照)。
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
use iced::Rectangle;

use crate::Message;

use super::hit::{bar_span_x, classify_bar_part, hit_test, BarPart, Hit};
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
    let clip_width = bounds.width;
    match mouse_event {
        mouse::Event::ButtonPressed(mouse::Button::Left) => {
            let position = cursor.position_in(bounds)?;

            match hit_test(
                position,
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
                // に渡すのは今この1回の `position.x` で、以降の move では
                // 呼び直さない(`Interaction` は `Body`/`Edge*` を覚えない —
                // 呼び手の `Shell` 側 `TimelineDragState::part` が正本を持つ)。
                Hit::Bar(id) => {
                    let Some(row) = pane.rows.iter().find(|row| row.id == id) else {
                        return None;
                    };
                    let (start_x, end_x) = bar_span_x(row, clip_width, pane.duration_frames);
                    let part = classify_bar_part(position.x, start_x, end_x);
                    let at_frame = frame_at_x(position.x, clip_width, pane.duration_frames);
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
                    let frame = frame_at_x(position.x, clip_width, pane.duration_frames);
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
                let frame = frame_at_x(position.x, clip_width, pane.duration_frames);
                Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
            }
            Some(DragKind::Clip) => {
                let position = cursor.position_in(bounds)?;
                let at_frame = frame_at_x(position.x, clip_width, pane.duration_frames);
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
    let clip_width = bounds.width;

    match hit_test(
        position,
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
            match classify_bar_part(position.x, start_x, end_x) {
                BarPart::Body => mouse::Interaction::Grab,
                BarPart::EdgeIn | BarPart::EdgeOut => mouse::Interaction::ResizingHorizontally,
            }
        }
        Hit::Blank => mouse::Interaction::default(),
    }
}
