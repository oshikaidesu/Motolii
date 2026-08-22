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
use super::projection::{frame_at_x, frame_to_x};
use super::work_area::{classify_loop_band, LoopBandPart};
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
    /// ループ帯(ルーラ最上段、B21+B18 第1切片)。新規/リサイズ/移動の区別は
    /// `PaneState`(`LoopDragKind`)が正本 — ここは「今ループ帯ドラッグ中」
    /// という種別だけを覚える(clip と同じ役割分担)。
    Loop,
}

/// ループ帯の当たり判定(押した瞬間の座標1回だけ — 正典 §1)。帯の縦域
/// (`loop_band_height`)内なら、作業範囲の画面 span に対し
/// [`classify_loop_band`] で部位を返す。帯の外は `None`(既存の scrub/clip 経路へ)。
fn loop_band_part_at(pane: &TimelinePane, position: iced::Point, width: f32) -> Option<LoopBandPart> {
    let band_height = super::canvas::loop_band_height(pane.ruler_height());
    if position.y >= band_height {
        return None;
    }
    let span_x = pane.work_area.map(|area| {
        let x0 = frame_to_x(area.start, width, pane.duration_frames);
        let x1 = frame_to_x(area.end, width, pane.duration_frames).max(x0 + 1.0);
        (x0, x1)
    });
    Some(classify_loop_band(position.x, span_x))
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

            // ループ帯(ルーラ最上段)が scrub より先に取る(正典 §5 の専用面 —
            // key_rows が property 帯を先に吸収するのと同じ優先順の型)。
            if let Some(part) = loop_band_part_at(pane, position, clip_width) {
                let at_frame = frame_at_x(position.x, clip_width, pane.duration_frames);
                state.drag = Some(DragKind::Loop);
                return Some(
                    canvas::Action::publish(Message::LoopBandGrabbed { part, at_frame })
                        .and_capture(),
                );
            }

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
        mouse::Event::ButtonPressed(mouse::Button::Right) => match state.drag {
            Some(DragKind::Clip) => {
                state.drag = None;
                Some(canvas::Action::publish(Message::DragCancelled).and_capture())
            }
            // ループ帯も同じ「キャンセルの一般化」(裁定151)— 掴んだ瞬間の
            // 範囲へ復元される(`PaneState::cancel_loop_drag`)。
            Some(DragKind::Loop) => {
                state.drag = None;
                Some(canvas::Action::publish(Message::LoopDragCancelled).and_capture())
            }
            _ => None,
        },
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
            Some(DragKind::Loop) => {
                let position = cursor.position_in(bounds)?;
                let at_frame = frame_at_x(position.x, clip_width, pane.duration_frames);
                Some(canvas::Action::publish(Message::LoopDragMoved { at_frame }).and_capture())
            }
            None => None,
        },
        mouse::Event::ButtonReleased(mouse::Button::Left) => match state.drag.take() {
            Some(DragKind::Scrub) => Some(canvas::Action::capture()),
            Some(DragKind::Clip) => {
                Some(canvas::Action::publish(Message::DragReleased).and_capture())
            }
            Some(DragKind::Loop) => {
                Some(canvas::Action::publish(Message::LoopDragReleased).and_capture())
            }
            None => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod mouse_interaction_tests {
    //! `mouse_interaction` の純関数テスト(hover 位置→Interaction の対応表、
    //! 発注書「red 保存」の器具)。`TimelinePane` の各 field は crate root
    //! (`lib.rs`)の定義に対して private だが、Rust の可視性は「定義モジュール
    //! とその子孫モジュール」まで届くので、この `input` 子モジュールから
    //! struct literal で直接組める(新しい pub API を増やさずに済む —
    //! 発注書の write-set は `src+tests` のみ、新規 pub 面は増やさない)。

    use iced::{mouse, Rectangle};

    use super::super::projection::RowProjection;
    use super::super::{Interaction, TimelinePane};
    use motolii_store::LayerId;

    /// duration_frames == bounds.width の 1px=1frame 縮尺(`hit.rs` の既存試験
    /// と同じ約束、新しい縮尺を発明しない)。
    const WIDTH: f32 = 300.0;
    const DURATION: i64 = 300;
    const ROW_HEIGHT: f32 = 26.0;

    fn row(id: u64, start: i64, duration: i64, locked: bool) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked,
            label_color: None,
            start,
            duration,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    fn pane(rows: Vec<RowProjection>) -> TimelinePane {
        let mut dims = motolii_tokens_rs::Dimensions::default();
        dims.row_height = ROW_HEIGHT;
        TimelinePane {
            rows,
            property_rows: Vec::new(),
            selected_row_index: None,
            markers: Vec::new(),
            playhead: 0,
            duration_frames: DURATION,
            fps: None,
            dims,
            colors: motolii_tokens_rs::Colors::default(),
            modifiers: iced::keyboard::Modifiers::default(),
            key_drag_active: false,
            preview_active: false,
            playing: false,
            work_area: None,
            loop_enabled: false,
            rename: None,
        }
    }

    fn bounds() -> Rectangle {
        Rectangle::new(iced::Point::ORIGIN, iced::Size::new(WIDTH, 300.0))
    }

    /// ルーラー下・行内の y。`TimelinePane::ruler_height()` は private だが
    /// 式自体は `canvas::ruler_height` と同じ(`0.846 * row_height` の丸め、
    /// モジュール冒頭 doc の「比率の出典」参照) — ここでは行0の中央を直接
    /// 計算する(新しい式を発明せず、既存の比率をそのまま使う)。
    fn row_y() -> f32 {
        let ruler = (0.846 * ROW_HEIGHT).round();
        ruler + ROW_HEIGHT / 2.0
    }

    fn point_at(x: f32) -> iced::Point {
        iced::Point::new(x, row_y())
    }

    /// 対応表(1): 幅広 bar の本体 hover → Grab(掴める予告)。
    #[test]
    fn hovering_bar_body_shows_grab() {
        let p = pane(vec![row(1, 100, 100, false)]);
        let state = Interaction::default();
        let cursor = mouse::Cursor::Available(point_at(150.0));
        assert_eq!(
            super::mouse_interaction(&p, &state, bounds(), cursor),
            mouse::Interaction::Grab,
        );
    }

    /// 対応表(2): 幅広 bar の端(trim 可能)hover → ResizingHorizontally。
    #[test]
    fn hovering_bar_edge_shows_resizing_horizontally() {
        let p = pane(vec![row(1, 100, 100, false)]);
        let state = Interaction::default();
        let cursor = mouse::Cursor::Available(point_at(100.0)); // start_x ちょうど = EdgeIn
        assert_eq!(
            super::mouse_interaction(&p, &state, bounds(), cursor),
            mouse::Interaction::ResizingHorizontally,
        );
    }

    /// 対応表(3): ロック行の bar hover → NotAllowed(端/本体の区別より優先)。
    #[test]
    fn hovering_locked_bar_shows_not_allowed() {
        let p = pane(vec![row(1, 100, 100, true)]);
        let state = Interaction::default();
        let cursor = mouse::Cursor::Available(point_at(150.0));
        assert_eq!(
            super::mouse_interaction(&p, &state, bounds(), cursor),
            mouse::Interaction::NotAllowed,
        );
    }

    /// 対応表(4): 空白面(ルーラー/playhead を含む scrub 対象)hover →
    /// Crosshair(正典 §5.5「空白面=Crosshair」— 掴める/掴んでいる/端/禁止の
    /// どれでもない場だが、クリックで scrub=playhead を動かせる以上「反応
    /// ゼロ」の既定矢印のままではいけない、という §5.5 の明文どおり)。
    #[test]
    fn hovering_blank_field_shows_crosshair() {
        let p = pane(vec![row(1, 100, 100, false)]);
        let state = Interaction::default();
        let cursor = mouse::Cursor::Available(point_at(10.0)); // bar の外(空白)
        assert_eq!(
            super::mouse_interaction(&p, &state, bounds(), cursor),
            mouse::Interaction::Crosshair,
        );
    }

    /// 対応表(5): ドラッグ中はどこにいても Grabbing(位置判定より優先)。
    #[test]
    fn dragging_shows_grabbing_regardless_of_position() {
        let p = pane(vec![row(1, 100, 100, false)]);
        let mut state = Interaction::default();
        state.drag = Some(super::DragKind::Scrub);
        let cursor = mouse::Cursor::Available(point_at(10.0));
        assert_eq!(
            super::mouse_interaction(&p, &state, bounds(), cursor),
            mouse::Interaction::Grabbing,
        );
    }
}

/// カーソル形状は意味の予告(正典 §5.5)。**触れそうな物には必ず予告を出す**
/// (Q0「触れそうで触れない物は不合格」の逆写像): 端=`ResizingHorizontally` /
/// 本体 hover=`Grab` / ドラッグ中=`Grabbing` / ロック行=`NotAllowed` /
/// 空白面(ルーラー・playhead を含む scrub 対象)=`Crosshair`(正典 §5.5 の
/// 5状態、実装済み調査 §4.4 のつけ得TOP5①)。
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

    // ループ帯(ルーラ最上段)の予告(正典 §5.5 の5状態を帯にも適用):
    // 端=ResizingHorizontally / 中=Grab / 空白=Crosshair(引けば新規の帯 —
    // scrub の空白面と同じ「ドラッグで意味が生まれる面」の予告)。
    if let Some(part) = loop_band_part_at(pane, position, clip_width) {
        return match part {
            LoopBandPart::EdgeIn | LoopBandPart::EdgeOut => mouse::Interaction::ResizingHorizontally,
            LoopBandPart::Body => mouse::Interaction::Grab,
            LoopBandPart::Blank => mouse::Interaction::Crosshair,
        };
    }

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
        // 正典 §5.5「空白面=Crosshair」: 空白部(ルーラー含む)は click-drag で
        // scrub=playhead を動かせる操作面 — 反応ゼロの既定矢印のままにしない
        // (Q0 の逆写像)。以前はここが `default()`(=`Interaction::None`、矢印)
        // のままで、正典が定める5状態のうち唯一未配線だった。
        Hit::Blank => mouse::Interaction::Crosshair,
    }
}
