//! 入力イベント → 意図(`TimelineMsg`)の翻訳と、canvas の描画呼び出し。
//!
//! `canvas::Program` は3つのメソッドを別々の時点で呼ぶ:
//!
//! | | 借り方 | できること |
//! |---|---|---|
//! | `update` | `&self` + `&mut State` | イベントを読む・**Message を1つ**出す |
//! | `draw` | `&self` + `&State` | 幾何を組む。**モデルを書く道が無い** |
//! | `mouse_interaction` | `&self` + `&State` | カーソル形状を返す |
//!
//! 三段が別メソッドに割れているのが egui との一番の構造差である。
//! egui は `ui.allocate_response()` が「レイアウト・入力・描画」を1点で返すので、
//! 分岐と副作用が同じ関数に集まる。iced は集めようとしても `draw` が `&self` なので
//! 集まらない。

use iced::keyboard::key::Named;
use iced::keyboard::{Event as KeyEvent, Key};
use iced::mouse;
use iced::widget::canvas::{self, Event, Geometry};
use iced::{Point, Rectangle, Renderer, Theme};

use crate::app::{hit_at, App, Drag};
use crate::message::TimelineMsg;
use crate::model::{Grab, Hit};
use crate::view;

/// `view()` が毎フレーム作り直す、モデルへの参照だけを持つ薄い殻。
pub struct TimelineProgram<'a> {
    pub app: &'a App,
}

impl canvas::Program<TimelineMsg> for TimelineProgram<'_> {
    /// **意図的に空にしてある。**
    ///
    /// iced はここに widget ローカルな可変状態を置かせてくれる(bezier_tool の
    /// `Option<Pending>` がそれ)。使えば下の `drag_position` の面倒も、
    /// `App::Drag` が持つモデル複製も消せた。使わなかったのは
    /// 「全部 Message にしたら何行になるか」を測るのがこの spike の目的だから。
    /// 実測は RETURN に、判断は書かない。
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<TimelineMsg>> {
        let app = self.app;
        let vp = &app.viewport;

        match event {
            // ── 押した ────────────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                let additive = app.modifiers.command();
                // hit test はここで解決してしまう。以降の message は
                // 「何をしようとしたか」だけを運ぶ。
                let msg = match hit_at(app, p.x, p.y) {
                    Hit::Ruler => TimelineMsg::ScrubStarted {
                        frame: vp.x_to_frame(p.x),
                    },
                    Hit::Clip { id, grab } => TimelineMsg::ClipGrabbed {
                        id,
                        grab,
                        at_frame: vp.x_to_frame(p.x),
                        additive,
                    },
                    Hit::Empty => TimelineMsg::EmptyPressed { additive },
                };
                Some(canvas::Action::publish(msg).and_capture())
            }

            // ── 引きずった ────────────────────────────────────────────
            // ジェスチャ中でないただのホバー移動では message を出さない。
            // 出すと 1 フレームに何十本も update が走って、モデルが
            // 「動いていないのに毎回書き換わる」状態になる。
            Event::Mouse(mouse::Event::CursorMoved { .. }) if app.drag.is_some() => {
                // 窓の外へ引きずっても追随させたいので `position_in` ではなく絶対座標から引く。
                let p = drag_position(cursor, bounds)?;
                Some(
                    canvas::Action::publish(TimelineMsg::PointerMoved {
                        frame: vp.x_to_frame(p.x),
                    })
                    .and_capture(),
                )
            }

            // ── 離した ────────────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if app.drag.is_some() =>
            {
                Some(canvas::Action::publish(TimelineMsg::PointerReleased).and_capture())
            }

            // ── ホイール ──────────────────────────────────────────────
            // 面の割り当ては AE / Premiere と同型:
            //   素     = 縦スクロール
            //   Shift  = 横パン
            //   Cmd    = 横 zoom(カーソル下のフレームが動かない)
            //
            // `WheelScrolled` は modifiers を運ばない。`app.modifiers` は
            // 別経路(`main.rs` の subscription)で追いかけている値である。
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let p = cursor.position_in(bounds)?;
                let (dx, dy) = match *delta {
                    mouse::ScrollDelta::Lines { x, y } => (x, y),
                    // trackpad は px で来る。1ノッチ相当を 40px として揃える。
                    mouse::ScrollDelta::Pixels { x, y } => (x / 40.0, y / 40.0),
                };
                let msg = if app.modifiers.command() {
                    TimelineMsg::ZoomedAt {
                        anchor_x: p.x,
                        notches: dy,
                    }
                } else if app.modifiers.shift() {
                    // Shift+ホイールは OS/デバイスによって dy が dx に化ける。
                    // 両方見て、大きい方を採る。
                    TimelineMsg::PannedX {
                        notches: if dx.abs() > dy.abs() { dx } else { dy },
                    }
                } else if dx.abs() > dy.abs() {
                    // trackpad の横スワイプは修飾無しでも横パン。
                    TimelineMsg::PannedX { notches: dx }
                } else {
                    TimelineMsg::ScrolledY { notches: dy }
                };
                Some(canvas::Action::publish(msg).and_capture())
            }

            // ── キーボード ────────────────────────────────────────────
            // ← → で 1 フレーム、Shift 併用で 10 フレーム(製品 F-04 と同じ意味)。
            //
            // 注意: canvas にはフォーカスの概念が無い。この腕はこの窓に canvas が
            // 2枚あれば**両方**が拾う。ここでは1枚しか無いので問題が出ない、
            // というだけである(詰まった箇所その3)。
            Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                let step = if modifiers.shift() { 10 } else { 1 };
                let frames = match key {
                    Key::Named(Named::ArrowLeft) => -step,
                    Key::Named(Named::ArrowRight) => step,
                    _ => return None,
                };
                Some(canvas::Action::publish(TimelineMsg::Nudged { frames }).and_capture())
            }

            // ── 計測のための連続再描画 ────────────────────────────────
            // 何もしなければ iced は入力があった時しか描かない。frame 時間を
            // 測るために毎フレーム次を要求する。製品ではやらない。
            Event::Window(iced::window::Event::RedrawRequested(_)) => {
                Some(canvas::Action::request_redraw())
            }

            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        view::draw(self.app, renderer, theme, bounds, cursor)
    }

    /// カーソル形状。`update` と**同じ `hit_test`** を通しているので、
    /// 「trim のカーソルが出ているのに掴むと移動した」が起きない。
    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // ジェスチャ中はカーソルを固定する。引きずって別の物の上に乗るたび
        // 形が変わるのは目障りなので。
        match &self.app.drag {
            Some(Drag::Move { .. }) => return mouse::Interaction::Grabbing,
            Some(Drag::Trim { .. }) => return mouse::Interaction::ResizingHorizontally,
            Some(Drag::Scrub) => return mouse::Interaction::ResizingHorizontally,
            None => {}
        }

        let Some(p) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        match hit_at(self.app, p.x, p.y) {
            Hit::Ruler => mouse::Interaction::ResizingHorizontally,
            Hit::Clip {
                grab: Grab::LeftEdge | Grab::RightEdge,
                ..
            } => mouse::Interaction::ResizingHorizontally,
            Hit::Clip { grab: Grab::Body, .. } => mouse::Interaction::Grab,
            Hit::Empty => mouse::Interaction::default(),
        }
    }
}

/// ドラッグ中の座標。canvas の外へ出ても止まらないよう、bounds でクリップせずに引く。
fn drag_position(cursor: mouse::Cursor, bounds: Rectangle) -> Option<Point> {
    let p = cursor.position()?;
    Some(Point::new(p.x - bounds.x, p.y - bounds.y))
}
