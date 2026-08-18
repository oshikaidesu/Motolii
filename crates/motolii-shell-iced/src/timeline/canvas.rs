//! Timeline の絵と、生イベント → [`TimelineMsg`] の翻訳。
//!
//! spike の `program.rs` / `view.rs` を Document 投影に差し替えたもの。
//! **描画パスに `&mut` は1つも無い**(`canvas::Program::draw` は `&self` で
//! モデルを借りるので、描きながらモデルを直す道が型として無い)。
//!
//! hit test・吸着・時間↔x は全部 [`semantics`](super::semantics) の純関数で、
//! `update`(イベントパス)と `draw`(絵)と `mouse_interaction`(手の形)が
//! **同じ関数**を呼ぶ — カーソルが trim の形なのに掴むと移動する、
//! 触れそうに見えて掴めない、が構造的に起きない(Q0)。

use iced::alignment;
use iced::keyboard::key::Named;
use iced::keyboard::{Event as KeyEvent, Key};
use iced::mouse;
use iced::widget::canvas::{self, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Pixels, Point, Rectangle, Renderer, Size, Theme};

use motolii_doc::{Document, LayerId};
use motolii_ui::blitz_shell::UiItemFlag;
use motolii_ui::timeline_editor::waveform_band::WaveformWindow;
use motolii_ui::timeline_editor::TrimEdge;
use motolii_ui::timeline_rows::{rows, TimelineFoldState, TimelineRow};

use super::pane::{GrabZone, TimelineDrag, TimelineMsg};
use super::semantics::{
    bar_span, flag_button_rect_y, frame_step, hit_test, item_mute_solo, movable_clips,
    mute_button_x, ruler_step_seconds, snap_candidates, snapped, solo_button_x, BarZone,
    PaneGeometry, TimelineHit, OVERVIEW_H, ROW_H, RULER_H,
};
use super::waveform::WaveformBandState;
use crate::message::Message;
use crate::shell::Shell;
use crate::theme::Tokens;

// ── 配色 ───────────────────────────────────────────────────────────────────
// egui 版 `timeline_editor/mod.rs` の mock_tokens 採用値をそのまま写す
// (見た目の正本は mock_tokens — ここで色を発明しない)。
const BG: Color = Color::from_rgb(0x29 as f32 / 255.0, 0x29 as f32 / 255.0, 0x29 as f32 / 255.0);
const HEAD_BG: Color =
    Color::from_rgb(0x38 as f32 / 255.0, 0x38 as f32 / 255.0, 0x38 as f32 / 255.0);
const CELL: Color = Color::from_rgb(0x36 as f32 / 255.0, 0x36 as f32 / 255.0, 0x36 as f32 / 255.0);
const TRACK_A: Color =
    Color::from_rgb(0x37 as f32 / 255.0, 0x37 as f32 / 255.0, 0x37 as f32 / 255.0);
const TRACK_B: Color =
    Color::from_rgb(0x25 as f32 / 255.0, 0x25 as f32 / 255.0, 0x25 as f32 / 255.0);
const RULE: Color = Color::from_rgb(0x11 as f32 / 255.0, 0x11 as f32 / 255.0, 0x11 as f32 / 255.0);
const INK: Color = Color::from_rgb(0xd4 as f32 / 255.0, 0xd4 as f32 / 255.0, 0xd4 as f32 / 255.0);
const DIM: Color = Color::from_rgb(0x8d as f32 / 255.0, 0x8d as f32 / 255.0, 0x8d as f32 / 255.0);
// 元は playhead に使っていた gold(#e9cf72)。timeline-library.css:93,96 では
// この色は `.timeGuide`(drag 中の snap 案内線)のもので、`.playhead` は白
// (2026-08-18 訂正、下の `SELECTED` へ差し替えた)。この canvas にはまだ
// snap 案内線の描画が無い(`snap_candidates`/`snapped` は当たりだけ返し、絵は
// 描かない)ので、定数ごと削った — 描くようになったら同じ #e9cf72 を
// ここへ戻す(README に残差として記載)。
const SELECTED: Color =
    Color::from_rgb(0xf2 as f32 / 255.0, 0xf2 as f32 / 255.0, 0xf2 as f32 / 255.0);
const WAVE: Color = Color::from_rgb(0x7f as f32 / 255.0, 0x92 as f32 / 255.0, 0x8c as f32 / 255.0);
const WAVE_BG: Color =
    Color::from_rgb(0x22 as f32 / 255.0, 0x22 as f32 / 255.0, 0x22 as f32 / 255.0);
const TRIM_BAND: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.28);
// ARRANGEMENT 俯瞰帯の下地。`timeline-library.css:3` `.overview{background:#242424}`
// (TRACK_B の #252525 とほぼ同値だが、帯とトラック縞は別要素なので取り違えないよう
// 別定数にする。egui 版参照は overview 行に専用の下地色を持たないので、この帯だけ
// 周りの行から浮かせる目的でこの値を採る)。
const OVERVIEW_BG: Color =
    Color::from_rgb(0x24 as f32 / 255.0, 0x24 as f32 / 255.0, 0x24 as f32 / 255.0);
/// 仮のパレット(egui 版 `LAYER_COLORS` と同じ並び。id から導く側だけを写す)。
const LAYER_COLORS: [Color; 8] = [
    Color::from_rgb(0x8c as f32 / 255.0, 0x6b as f32 / 255.0, 0x6b as f32 / 255.0),
    Color::from_rgb(0x8c as f32 / 255.0, 0x7d as f32 / 255.0, 0x5c as f32 / 255.0),
    Color::from_rgb(0x7d as f32 / 255.0, 0x8c as f32 / 255.0, 0x5c as f32 / 255.0),
    Color::from_rgb(0x5c as f32 / 255.0, 0x8c as f32 / 255.0, 0x6f as f32 / 255.0),
    Color::from_rgb(0x5c as f32 / 255.0, 0x7f as f32 / 255.0, 0x8c as f32 / 255.0),
    Color::from_rgb(0x64 as f32 / 255.0, 0x66 as f32 / 255.0, 0x8c as f32 / 255.0),
    Color::from_rgb(0x7d as f32 / 255.0, 0x5c as f32 / 255.0, 0x8c as f32 / 255.0),
    Color::from_rgb(0x8c as f32 / 255.0, 0x5c as f32 / 255.0, 0x74 as f32 / 255.0),
];

fn layer_color(layer: LayerId) -> Color {
    LAYER_COLORS[(layer.get() % LAYER_COLORS.len() as u64) as usize]
}

/// 角丸の矩形。行頭スウォッチ・bar・ARRANGEMENT の窓・下端スクロールバーが
/// 共用する(`/tmp/egui-same-doc.png` はどれも角丸 — 直角の矩形は無い)。
fn rounded_rect(top_left: Point, size: Size, radius: f32) -> Path {
    Path::rounded_rectangle(top_left, size, radius.into())
}

/// `view()` が毎フレーム作り直す、殻への参照だけを持つ薄い殻。
pub struct TimelineProgram<'a> {
    pub shell: &'a Shell,
}

/// canvas ローカルの transient 状態。ARRANGEMENT 帯を掴んでいる間だけ真になる —
/// Document を持たない chrome の話なので `TimelinePane::drag`(Move/Trim/Scrub の
/// preview)には混ぜない。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TimelineCanvasState {
    overview_dragging: bool,
}

/// この canvas が読む一式。座席が無い時は `None`(canvas 自体が立たないはずだが、
/// 立っていても黙って空の絵になるだけにする)。
struct Scene {
    document: std::sync::Arc<Document>,
    rows: Vec<TimelineRow>,
    selected: Vec<LayerId>,
    playhead: f32,
}

impl TimelineProgram<'_> {
    fn scene(&self) -> Option<Scene> {
        let document = self.shell.timeline_snapshot()?;
        let rows = rows(&document, &TimelineFoldState::default());
        Some(Scene {
            selected: self.shell.timeline_selection(),
            playhead: self.shell.timeline_playhead(),
            document,
            rows,
        })
    }

    fn geometry(&self, bounds: Rectangle, document: &Document) -> PaneGeometry {
        PaneGeometry {
            width: bounds.width,
            height: bounds.height,
            wave_h: motolii_ui::timeline_editor::waveform_band::band_height(
                document.soundtrack.is_some(),
            ),
        }
    }
}

impl canvas::Program<Message> for TimelineProgram<'_> {
    type State = TimelineCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let scene = self.scene()?;
        let pane = self.shell.timeline_pane();
        let geometry = self.geometry(bounds, &scene.document);
        let view = pane.view;

        let publish = |msg: TimelineMsg| {
            Some(canvas::Action::publish(Message::Timeline(msg)).and_capture())
        };

        match event {
            // ── 押した ────────────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let p = cursor.position_in(bounds)?;
                let additive = pane.modifiers.command();
                let hit = hit_test(
                    &scene.document,
                    &scene.rows,
                    view,
                    &geometry,
                    pane.scroll_y,
                    p.x,
                    p.y,
                );
                state.overview_dragging = matches!(hit, TimelineHit::Overview { .. });
                let msg = match hit {
                    TimelineHit::Overview { at_seconds } => {
                        TimelineMsg::OverviewSeek { at_seconds }
                    }
                    TimelineHit::Ruler { at_seconds } => TimelineMsg::ScrubStarted { at_seconds },
                    TimelineHit::FlagButton { layer, flag } => {
                        TimelineMsg::FlagPressed { layer, flag }
                    }
                    TimelineHit::Rail { layer } => TimelineMsg::RowPicked { layer, additive },
                    TimelineHit::Bar {
                        layer,
                        zone,
                        at_seconds,
                    } => TimelineMsg::BarGrabbed {
                        layer,
                        zone: match zone {
                            BarZone::Body => GrabZone::Body,
                            BarZone::In => GrabZone::Edge(TrimEdge::In),
                            BarZone::Out => GrabZone::Edge(TrimEdge::Out),
                        },
                        at_seconds,
                        additive,
                    },
                    TimelineHit::Empty => TimelineMsg::EmptyPressed { additive },
                };
                publish(msg)
            }

            // ── ARRANGEMENT 帯を引きずった ────────────────────────────
            // Document とは無縁の view chrome なので `pane.drag` ではなく
            // canvas ローカルの `state.overview_dragging` で追う。
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.overview_dragging => {
                let p = cursor.position()?;
                let x = p.x - bounds.x;
                let (x0, x1) = geometry.overview_track_x();
                let comp = scene.document.composition.duration.as_seconds_f64() as f32;
                if comp <= 0.0 || x1 <= x0 {
                    return None;
                }
                let ratio = ((x - x0) / (x1 - x0)).clamp(0.0, 1.0);
                publish(TimelineMsg::OverviewSeek {
                    at_seconds: ratio * comp,
                })
            }

            // ── 引きずった ────────────────────────────────────────────
            // ジェスチャ中でないただのホバー移動では message を出さない
            // (spike と同じ: 動いていないのに毎回 update が走るのを避ける)。
            Event::Mouse(mouse::Event::CursorMoved { .. }) if pane.drag.is_some() => {
                // 窓の外へ引きずっても追随させたいので絶対座標から引く。
                let p = cursor.position()?;
                let x = p.x - bounds.x;
                let mut at = geometry.x_to_time(view, x);
                // Move / Trim は吸着済みの時刻を運ぶ(候補は自分以外)。
                if matches!(
                    pane.drag,
                    Some(TimelineDrag::Move { .. }) | Some(TimelineDrag::Trim { .. })
                ) {
                    let exclude = pane.moving_layers();
                    let candidates =
                        snap_candidates(&scene.document, &scene.rows, scene.playhead, &exclude);
                    at = snapped(at, &candidates, geometry.px_per_second(view));
                }
                publish(TimelineMsg::PointerMoved { at_seconds: at })
            }

            // ── 離した ────────────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.overview_dragging =>
            {
                state.overview_dragging = false;
                None
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if pane.drag.is_some() =>
            {
                publish(TimelineMsg::PointerReleased)
            }

            // ── ホイール(面の割り当ては AE / Premiere 同型・spike と同じ)──
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let p = cursor.position_in(bounds)?;
                let (dx, dy) = match *delta {
                    mouse::ScrollDelta::Lines { x, y } => (x, y),
                    mouse::ScrollDelta::Pixels { x, y } => (x / 40.0, y / 40.0),
                };
                let px_per_second = geometry.px_per_second(view);
                let msg = if pane.modifiers.command() {
                    // 1ノッチ = 1.2倍(spike と同じ段)。span は倍率の逆数で狭まる。
                    TimelineMsg::ZoomedAt {
                        anchor_seconds: geometry.x_to_time(view, p.x),
                        factor: 1.2f32.powf(-dy),
                    }
                } else if pane.modifiers.shift() {
                    // Shift+ホイールは OS/デバイスで dy が dx に化ける。大きい方を採る。
                    let notches = if dx.abs() > dy.abs() { dx } else { dy };
                    TimelineMsg::PannedX {
                        delta_seconds: -(notches * 48.0) / px_per_second,
                    }
                } else if dx.abs() > dy.abs() {
                    TimelineMsg::PannedX {
                        delta_seconds: -(dx * 48.0) / px_per_second,
                    }
                } else {
                    let viewport = (bounds.height - geometry.rows_top()).max(0.0);
                    let max = (scene.rows.len() as f32 * ROW_H - viewport).max(0.0);
                    TimelineMsg::ScrolledY {
                        delta_px: -dy * 32.0,
                        max,
                    }
                };
                publish(msg)
            }

            // ── キーボード ────────────────────────────────────────────
            Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                publish(TimelineMsg::ModifiersChanged(*modifiers))
            }
            Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => {
                let msg = match key {
                    Key::Named(Named::Escape) if pane.drag.is_some() => {
                        TimelineMsg::GestureCancelled
                    }
                    Key::Named(Named::Delete) | Key::Named(Named::Backspace) => {
                        TimelineMsg::DeletePressed
                    }
                    Key::Named(Named::ArrowLeft) | Key::Named(Named::ArrowRight) => {
                        let frames = frame_step(
                            matches!(key, Key::Named(Named::ArrowLeft)),
                            matches!(key, Key::Named(Named::ArrowRight)),
                            modifiers.shift(),
                        );
                        if frames == 0 {
                            return None;
                        }
                        TimelineMsg::PlayheadStepped { frames }
                    }
                    Key::Character(character)
                        if modifiers.command() && character.eq_ignore_ascii_case("z") =>
                    {
                        if modifiers.shift() {
                            TimelineMsg::RedoPressed
                        } else {
                            TimelineMsg::UndoPressed
                        }
                    }
                    _ => return None,
                };
                publish(msg)
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
        let tokens = Tokens::resolve(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), BG);
        let Some(scene) = self.scene() else {
            return vec![frame.into_geometry()];
        };
        let pane = self.shell.timeline_pane();
        let geometry = self.geometry(bounds, &scene.document);
        let view = pane.view;
        let size = bounds.size();

        // ── 行の下地とレール ─────────────────────────────────────────
        let hover = cursor.position_in(bounds).map(|p| {
            hit_test(
                &scene.document,
                &scene.rows,
                view,
                &geometry,
                pane.scroll_y,
                p.x,
                p.y,
            )
        });

        for (index, row) in scene.rows.iter().enumerate() {
            let y = geometry.row_top(index, pane.scroll_y);
            if y + ROW_H < geometry.rows_top() || y > size.height {
                continue;
            }
            // トラック面の縞。
            frame.fill_rectangle(
                Point::new(geometry.track_left(), y),
                Size::new(geometry.track_w(), ROW_H),
                if index % 2 == 0 { TRACK_A } else { TRACK_B },
            );
            // レール(名前)。
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(geometry.track_left(), ROW_H),
                CELL,
            );
            let selected = scene.selected.contains(&row.layer);
            let span = bar_span(&scene.document, row.layer);
            let is_group_icon = span.map(|(_, _, g)| g).unwrap_or(false);
            let indent = 8.0 + f32::from(row.depth) * 12.0;
            // 種別色の四角(角丸)。`/tmp/egui-same-doc.png` の行頭スウォッチ —
            // bar に既に使っている色をそのまま流用する(新しい色は発明しない)。
            frame.fill(
                &rounded_rect(
                    Point::new(indent, y + ROW_H * 0.5 - 7.0),
                    Size::new(14.0, 14.0),
                    3.0,
                ),
                if is_group_icon {
                    HEAD_BG
                } else {
                    layer_color(row.layer)
                },
            );
            let name = scene
                .document
                .layers
                .display_name(row.layer)
                .unwrap_or("?")
                .to_owned();
            frame.fill_text(Text {
                content: name,
                position: Point::new(indent + 20.0, y + ROW_H * 0.5),
                color: if selected { SELECTED } else { INK },
                size: Pixels(11.0),
                align_y: alignment::Vertical::Center.into(),
                ..Text::default()
            });
            // M / S。押下状態は Document(`ItemEnvelope`)から読む — ボタン側に
            // 状態を持たない(egui 版 `item_flags_for` と同じ判断)。
            let (muted, solo) = item_mute_solo(&scene.document, row.layer);
            let (btn_y0, btn_h) = flag_button_rect_y(y);
            let mute_hovered = matches!(
                hover,
                Some(TimelineHit::FlagButton { layer, flag: UiItemFlag::Mute }) if layer == row.layer
            );
            let solo_hovered = matches!(
                hover,
                Some(TimelineHit::FlagButton { layer, flag: UiItemFlag::Solo }) if layer == row.layer
            );
            draw_flag_button(
                &mut frame,
                mute_button_x(),
                btn_y0,
                btn_h,
                "M",
                muted,
                mute_hovered,
                tokens.text_muted,
                tokens,
            );
            draw_flag_button(
                &mut frame,
                solo_button_x(),
                btn_y0,
                btn_h,
                "S",
                solo,
                solo_hovered,
                tokens.action_active,
                tokens,
            );
            frame.fill_rectangle(
                Point::new(0.0, y + ROW_H - 1.0),
                Size::new(size.width, 1.0),
                RULE,
            );

            // ── bar ─────────────────────────────────────────────────
            let Some((mut start, mut end, is_group)) = span else {
                continue;
            };
            // 進行中ジェスチャの preview を映す(Document は release まで触らない)。
            match &pane.drag {
                Some(TimelineDrag::Move {
                    targets,
                    preview_delta,
                    ..
                }) => {
                    let row_moves = if is_group {
                        let children = movable_clips(&scene.document, row.layer);
                        !children.is_empty()
                            && children
                                .iter()
                                .all(|(child, _, _)| targets.iter().any(|(l, _, _)| l == child))
                    } else {
                        targets.iter().any(|(l, _, _)| *l == row.layer)
                    };
                    if row_moves {
                        start += preview_delta;
                        end += preview_delta;
                    }
                }
                Some(TimelineDrag::Trim {
                    layer,
                    edge,
                    preview,
                    ..
                }) if *layer == row.layer => match edge {
                    TrimEdge::In => start = *preview,
                    TrimEdge::Out => end = *preview,
                },
                _ => {}
            }

            let x0 = geometry.time_to_x(view, start);
            let x1 = geometry.time_to_x(view, end);
            if x1 < geometry.track_left() || x0 > size.width {
                continue;
            }
            let x0c = x0.max(geometry.track_left());
            let wc = (x1.min(size.width) - x0c).max(1.0);
            let bar_top = y + 3.0;
            let bar_h = ROW_H - 6.0;
            let bar_color = if is_group {
                HEAD_BG
            } else {
                layer_color(row.layer)
            };
            frame.fill(
                &rounded_rect(Point::new(x0c, bar_top), Size::new(wc, bar_h), 3.0),
                bar_color,
            );
            // 畳んである Group は中身をその bar の中に出す(egui と同じ)。
            if is_group {
                for (child, cs, ce) in movable_clips(&scene.document, row.layer) {
                    let offset = match &pane.drag {
                        Some(TimelineDrag::Move {
                            targets,
                            preview_delta,
                            ..
                        }) if targets.iter().any(|(l, _, _)| *l == child) => *preview_delta,
                        _ => 0.0,
                    };
                    let cx0 = geometry.time_to_x(view, cs + offset).max(geometry.track_left());
                    let cx1 = geometry.time_to_x(view, ce + offset).min(size.width);
                    if cx1 - cx0 > 0.5 {
                        frame.fill_rectangle(
                            Point::new(cx0, bar_top + bar_h * 0.55),
                            Size::new(cx1 - cx0, bar_h * 0.45 - 1.0),
                            layer_color(child),
                        );
                    }
                }
            }
            frame.stroke(
                &rounded_rect(Point::new(x0c, bar_top), Size::new(wc, bar_h), 3.0),
                Stroke::default()
                    .with_color(if selected { tokens.action_active } else { RULE })
                    .with_width(if selected { 1.5 } else { 1.0 }),
            );

            // trim 帯: 掴める場所を掴む前に見せる(触れそうで触れない物を作らない)。
            if let Some(TimelineHit::Bar {
                layer,
                zone: zone @ (BarZone::In | BarZone::Out),
                ..
            }) = hover
            {
                if layer == row.layer && pane.drag.is_none() {
                    let band_x = match zone {
                        BarZone::In => x0,
                        _ => x1 - super::semantics::TRIM_EDGE,
                    };
                    frame.fill_rectangle(
                        Point::new(band_x, bar_top),
                        Size::new(super::semantics::TRIM_EDGE, bar_h),
                        TRIM_BAND,
                    );
                }
            }
        }

        // 行の面より上をマスクし直す(スクロールで行がルーラへ食い込まないように)。
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(size.width, geometry.rows_top()),
            BG,
        );

        // ── transport 帯(表示専用)──────────────────────────────────
        // `/tmp/egui-same-doc.png` の最上段。再生ボタン・`space=play` 等の
        // **対応する intent が無い項目は描かない**(2026-08-19 裁定、Q0) —
        // 残すのは既に読める状態(playhead・行数・view 範囲・ルーラ刻み)だけ。
        let comp = scene.document.composition.duration.as_seconds_f64() as f32;
        let step = ruler_step_seconds(geometry.px_per_second(view));
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(size.width, geometry.transport_bottom()),
            HEAD_BG,
        );
        frame.fill_text(Text {
            content: playhead_clock(scene.playhead),
            position: Point::new(8.0, geometry.transport_bottom() * 0.5),
            color: INK,
            size: Pixels(15.0),
            font: iced::Font::MONOSPACE,
            align_y: alignment::Vertical::Center.into(),
            ..Text::default()
        });
        frame.fill_text(Text {
            content: format!(
                "{} rows   view {:.2}-{:.2}s   grid {}",
                scene.rows.len(),
                view.start,
                view.start + view.span,
                grid_label(step),
            ),
            position: Point::new(96.0, geometry.transport_bottom() * 0.5),
            color: DIM,
            size: Pixels(10.0),
            font: iced::Font::MONOSPACE,
            align_y: alignment::Vertical::Center.into(),
            ..Text::default()
        });
        frame.fill_rectangle(
            Point::new(0.0, geometry.transport_bottom() - 1.0),
            Size::new(size.width, 1.0),
            RULE,
        );

        // ── ARRANGEMENT 俯瞰帯 ───────────────────────────────────────
        // `/tmp/egui-same-doc.png` — 灰色の丸みを帯びた1本の帯(セグメント
        // 表示ではない)。押す/引きずると `OverviewSeek` が飛び、view がその
        // 時刻へ寄る(機能する chrome だけを置く — 押しても動かない飾りは
        // 作らない)。
        frame.fill_rectangle(
            Point::new(0.0, geometry.transport_bottom()),
            Size::new(size.width, OVERVIEW_H),
            OVERVIEW_BG,
        );
        let (ov_x0, ov_x1) = geometry.overview_track_x();
        if comp > 0.0 {
            let to_ov_x = |t: f32| ov_x0 + (t.clamp(0.0, comp) / comp) * (ov_x1 - ov_x0);
            let win_x0 = to_ov_x(view.start);
            let win_x1 = to_ov_x(view.start + view.span);
            let pill_top = geometry.transport_bottom() + 4.0;
            let pill_h = OVERVIEW_H - 8.0;
            frame.fill(
                &rounded_rect(
                    Point::new(win_x0, pill_top),
                    Size::new((win_x1 - win_x0).max(pill_h), pill_h),
                    pill_h * 0.5,
                ),
                // `/tmp/egui-same-doc.png` の窓は中間グレー — token の
                // `border_strong`(既存 role)がほぼ同値なので、これも新しい
                // hex は発明しない。
                tokens.border_strong,
            );
        }

        // ── ルーラ ──────────────────────────────────────────────────
        let ruler_top = geometry.ruler_top();
        frame.fill_rectangle(
            Point::new(0.0, ruler_top),
            Size::new(size.width, RULER_H),
            HEAD_BG,
        );
        // 副目盛(主目盛の 1/5)。`/tmp/egui-same-doc.png` はルーラの中が
        // 細かい縦線で密に埋まる — 主目盛だけだと間延びして見える。
        let minor_step = step / 5.0;
        if minor_step > 0.5 {
            let mut minor = (view.start / minor_step).floor() * minor_step;
            let last = view.start + view.span;
            while minor <= last {
                if minor >= 0.0 {
                    let x = geometry.time_to_x(view, minor);
                    if x >= geometry.track_left() {
                        frame.fill_rectangle(
                            Point::new(x, ruler_top + RULER_H - 4.0),
                            Size::new(1.0, 4.0),
                            Color { a: 0.35, ..DIM },
                        );
                    }
                }
                minor += minor_step;
            }
        }
        let mut tick = (view.start / step).floor() * step;
        let last = view.start + view.span;
        while tick <= last {
            if tick >= 0.0 {
                let x = geometry.time_to_x(view, tick);
                if x >= geometry.track_left() {
                    frame.fill_rectangle(
                        Point::new(x, ruler_top + RULER_H - 8.0),
                        Size::new(1.0, 8.0),
                        DIM,
                    );
                    frame.fill_text(Text {
                        content: ruler_label(tick),
                        position: Point::new(x + 3.0, ruler_top + RULER_H - 14.0),
                        color: DIM,
                        size: Pixels(10.0),
                        font: iced::Font::MONOSPACE,
                        ..Text::default()
                    });
                    // 行の面へ落ちる細いグリッド。
                    frame.fill_rectangle(
                        Point::new(x, geometry.rows_top()),
                        Size::new(1.0, size.height - geometry.rows_top()),
                        Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                    );
                }
            }
            tick += step;
        }
        frame.fill_rectangle(
            Point::new(0.0, ruler_top + RULER_H - 1.0),
            Size::new(size.width, 1.0),
            RULE,
        );

        // ── 波形帯(ルーラの直下・同じ時間換算)──────────────────────
        if geometry.wave_h > 0.0 {
            let top = geometry.ruler_bottom();
            frame.fill_rectangle(
                Point::new(0.0, top),
                Size::new(size.width, geometry.wave_h),
                WAVE_BG,
            );
            frame.fill_text(Text {
                content: "soundtrack".to_owned(),
                position: Point::new(10.0, top + geometry.wave_h * 0.5),
                color: DIM,
                size: Pixels(9.0),
                align_y: alignment::Vertical::Center.into(),
                ..Text::default()
            });
            let mid = top + geometry.wave_h * 0.5;
            frame.fill_rectangle(
                Point::new(geometry.track_left(), mid),
                Size::new(geometry.track_w(), 1.0),
                Color::from_rgb(0x3a as f32 / 255.0, 0x3a as f32 / 255.0, 0x3a as f32 / 255.0),
            );
            match self.shell.waveform_state() {
                WaveformBandState::Ready(peaks) => {
                    let start_offset = scene
                        .document
                        .soundtrack
                        .as_ref()
                        .map(|st| st.start_offset.as_seconds_f64() as f32)
                        .unwrap_or(0.0);
                    let mut columns = Vec::new();
                    peaks.columns(
                        WaveformWindow {
                            view_start: view.start,
                            view_span: view.span,
                            width_px: geometry.track_w(),
                            start_offset,
                        },
                        &mut columns,
                    );
                    let half = geometry.wave_h * 0.5 - 2.0;
                    for (column, peak) in columns.iter().enumerate() {
                        let h = (peak * half).max(0.5);
                        frame.fill_rectangle(
                            Point::new(geometry.track_left() + column as f32, mid - h),
                            Size::new(1.0, h * 2.0),
                            WAVE,
                        );
                    }
                }
                WaveformBandState::Building => {
                    frame.fill_text(Text {
                        content: "waveform…".to_owned(),
                        position: Point::new(geometry.track_left() + 8.0, mid),
                        color: DIM,
                        size: Pixels(10.0),
                        align_y: alignment::Vertical::Center.into(),
                        ..Text::default()
                    });
                }
                WaveformBandState::Unavailable(reason) => {
                    frame.fill_text(Text {
                        content: reason,
                        position: Point::new(geometry.track_left() + 8.0, mid),
                        color: DIM,
                        size: Pixels(10.0),
                        align_y: alignment::Vertical::Center.into(),
                        ..Text::default()
                    });
                }
                WaveformBandState::Absent => {}
            }
            frame.fill_rectangle(
                Point::new(0.0, top + geometry.wave_h - 1.0),
                Size::new(size.width, 1.0),
                RULE,
            );
        }

        // ── レールとトラックの縦の境界(ARRANGEMENT 帯の下だけ — あの帯は
        // レール/トラックの2列に分かれていない)───────────────────────
        frame.fill_rectangle(
            Point::new(geometry.track_left() - 1.0, ruler_top),
            Size::new(1.0, size.height - ruler_top),
            RULE,
        );

        // ── playhead(ルーラから下だけ。ARRANGEMENT 帯には俯瞰トラック側の
        // 目印(上で描いた `em` 相当)が別にある)─────────────────────
        let playhead = pane.scrub_preview().unwrap_or(scene.playhead);
        let px = geometry.time_to_x(view, playhead);
        if px >= geometry.track_left() - 1.0 && px <= size.width + 1.0 {
            frame.fill_rectangle(
                Point::new(px, ruler_top),
                Size::new(1.0, size.height - ruler_top),
                SELECTED,
            );
            let head = Path::new(|p| {
                p.move_to(Point::new(px - 5.0, ruler_top));
                p.line_to(Point::new(px + 5.0, ruler_top));
                p.line_to(Point::new(px, ruler_top + 9.0));
                p.close();
            });
            frame.fill(&head, SELECTED);
        }

        // ── 下端の横スクロールバー(表示専用)──────────────────────────
        // `/tmp/egui-same-doc.png` 最下段の丸い灰色バー。今回はドラッグの
        // 意味を増やさない(ARRANGEMENT の窓が既にその役目を持つ)ので、
        // view/comp の比率を映すだけの飾りとして置く(2026-08-19 裁定 —
        // 「表示するだけで正しい物」に該当し、状態は既にある)。
        if comp > 0.0 {
            let (sb_y0, sb_h) = geometry.scrollbar_y();
            let (tx0, tx1) = geometry.overview_track_x();
            let to_x = |t: f32| tx0 + (t.clamp(0.0, comp) / comp) * (tx1 - tx0);
            let sb_x0 = to_x(view.start);
            let sb_x1 = to_x(view.start + view.span);
            frame.fill(
                &rounded_rect(
                    Point::new(sb_x0, sb_y0),
                    Size::new((sb_x1 - sb_x0).max(sb_h), sb_h),
                    sb_h * 0.5,
                ),
                Color {
                    a: 0.6,
                    ..tokens.border_strong
                },
            );
        }

        vec![frame.into_geometry()]
    }

    /// カーソル形状。`update` と**同じ `hit_test`** を通す。
    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let pane = self.shell.timeline_pane();
        // ジェスチャ中はカーソルを固定する(引きずって別の物の上に乗るたび
        // 形が変わるのは目障り — spike と同じ)。
        if state.overview_dragging {
            return mouse::Interaction::ResizingHorizontally;
        }
        match &pane.drag {
            Some(TimelineDrag::Move { .. }) => return mouse::Interaction::Grabbing,
            Some(TimelineDrag::Trim { .. }) | Some(TimelineDrag::Scrub { .. }) => {
                return mouse::Interaction::ResizingHorizontally
            }
            None => {}
        }
        let Some(scene) = self.scene() else {
            return mouse::Interaction::default();
        };
        let Some(p) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let geometry = self.geometry(bounds, &scene.document);
        match hit_test(
            &scene.document,
            &scene.rows,
            pane.view,
            &geometry,
            pane.scroll_y,
            p.x,
            p.y,
        ) {
            TimelineHit::Overview { .. } => mouse::Interaction::Pointer,
            TimelineHit::Ruler { .. } => mouse::Interaction::ResizingHorizontally,
            TimelineHit::FlagButton { .. } => mouse::Interaction::Pointer,
            TimelineHit::Bar {
                zone: BarZone::In | BarZone::Out,
                ..
            } => mouse::Interaction::ResizingHorizontally,
            TimelineHit::Bar {
                zone: BarZone::Body,
                ..
            } => mouse::Interaction::Grab,
            TimelineHit::Rail { .. } => mouse::Interaction::Pointer,
            TimelineHit::Empty => mouse::Interaction::default(),
        }
    }
}

/// M / S の1枚。inspector_pane.rs の `flag_button_style` と同じ3状態
/// (pressed / hover / 既定)を token だけで写す — Timeline と Inspector の
/// M/S が同じ手触りになる(新しい色を発明しない)。
#[allow(clippy::too_many_arguments)]
fn draw_flag_button(
    frame: &mut Frame,
    x_range: (f32, f32),
    y0: f32,
    h: f32,
    label: &str,
    pressed: bool,
    hovered: bool,
    accent: Color,
    tokens: &Tokens,
) {
    let (x0, x1) = x_range;
    let w = (x1 - x0).max(1.0);
    let (border, background, text_color) = if pressed {
        (
            accent,
            Color { a: 0.18, ..accent },
            if label == "S" { accent } else { tokens.text_primary },
        )
    } else if hovered {
        (
            tokens.border_strong,
            tokens.surface_panel,
            tokens.text_primary,
        )
    } else {
        (tokens.border_default, tokens.surface_app, tokens.text_muted)
    };
    let rect = rounded_rect(Point::new(x0, y0), Size::new(w, h), 2.0);
    frame.fill(&rect, background);
    frame.stroke(&rect, Stroke::default().with_color(border).with_width(1.0));
    frame.fill_text(Text {
        content: label.to_owned(),
        position: Point::new(x0 + w * 0.5, y0 + h * 0.5),
        color: text_color,
        size: Pixels(9.0),
        font: iced::Font::MONOSPACE,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center.into(),
        ..Text::default()
    });
}

/// ルーラの目盛の文字。分:秒.小数第1位まで固定
/// (`/tmp/egui-same-doc.png` の `0:00.0 | 0:02.0 | …` と同じ書式)。
fn ruler_label(t: f32) -> String {
    let minutes = (t / 60.0).floor() as i64;
    let seconds = t - minutes as f32 * 60.0;
    format!("{minutes}:{seconds:04.1}")
}

/// transport 帯の playhead 読み。時:分:秒(`/tmp/egui-same-doc.png` の
/// `0:00:00`)。
fn playhead_clock(seconds: f32) -> String {
    let total = seconds.max(0.0).round() as i64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    format!("{h}:{m:02}:{s:02}")
}

/// transport 帯の `grid n` — 整数なら小数を出さない
/// (`/tmp/egui-same-doc.png` の `grid 1`)。
fn grid_label(step: f32) -> String {
    if step.fract().abs() < f32::EPSILON {
        format!("{}", step as i64)
    } else {
        format!("{step}")
    }
}
