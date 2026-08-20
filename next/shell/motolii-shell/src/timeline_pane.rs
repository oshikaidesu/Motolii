//! Timeline pane(第1波: 読み取り投影 + scrub + 選択)。
//!
//! **`StoreView` と `&Session` の投影のみ**(裁定5)。Document の写しは持たない —
//! [`rows`] は毎 `view()` 呼び出しごとに `StoreView` から作り直す使い捨ての値であって、
//! `Shell` はこれを保持しない。
//!
//! 名詞は地図(`LayerAttrs.name` / `LayerTiming` / `markers()`)から逆算し、そこに無い
//! 物は表示しない。動詞は第1波の2つだけ(ルーラー/行の空白部の click・drag で scrub、
//! bar の click で選択)— M5〜M7(drag 移動・trim・split・複製・Copy-Paste)は第2波。
//! 死に chrome を避けるため、それらのボタンやメニューは1つも置かない(Q0)。
//!
//! bar の縦位置は [`crate::Session`] の選択に同期する読み取り専用のハイライトであり、
//! Document 上の所有者ではない(`docs/ui-score-model.md` — Lane を所有者にしない)。

use iced::mouse;
use iced::widget::canvas;
use iced::{Element, Length, Point, Rectangle, Size};

use motolii_store::{Fps, LayerId, Marker, StoreView};

use crate::tokens::{Colors, Dimensions};
use crate::{Message, Session};

/// 1層分の読み取り投影。**Document の写しではなく、1度描くための使い捨て値**。
#[derive(Clone, Debug, PartialEq)]
pub struct RowProjection {
    pub id: LayerId,
    pub name: String,
    pub hidden: bool,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
}

/// `store`/`session` から Timeline の行を組み立てる。**読むだけ**。
///
/// `store.layers()` は「present な layer」しか返さない(削除は墓標なので既に除外
/// 済み — `view.rs`)。ここでは並び順を `LayerId` 昇順のまま使う。bar の重ね順
/// (`meta.order`)は Stage 側の合成順であって、Timeline の縦位置の所有者にしない
/// (`ui-score-model.md` 4層構成: 縦位置は packing 結果にすぎない)。
pub fn rows(store: &StoreView<'_>, session: &Session) -> Vec<RowProjection> {
    let mut out = Vec::new();
    for id in store.layers() {
        let Ok(Some(meta)) = store.meta(id) else {
            continue;
        };
        let attrs = store.attrs(id).ok().flatten().unwrap_or_default();
        out.push(RowProjection {
            id,
            name: attrs.name,
            hidden: attrs.hidden,
            start: meta.timing.start,
            duration: meta.timing.duration,
            selected: session.selection == Some(id),
        });
    }
    out
}

/// comp フレーム → x px。`duration_frames <= 0` の空 comp では常に 0。
fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0.0;
    }
    let ratio = frame as f32 / duration_frames as f32;
    (ratio * width).clamp(0.0, width)
}

/// x px → comp フレーム。**scrub の core**(canvas の click/drag と単体 test の両方が
/// これを呼ぶ)。範囲外は端へ丸める。
pub fn frame_at_x(x: f32, width: f32, duration_frames: i64) -> i64 {
    if duration_frames <= 0 || width <= 0.0 {
        return 0;
    }
    let ratio = (x / width).clamp(0.0, 1.0);
    let frame = (ratio * duration_frames as f32).round() as i64;
    frame.clamp(0, (duration_frames - 1).max(0))
}

/// click/drag した先が何か。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// この layer の bar の上。
    Bar(LayerId),
    /// ルーラーまたは行の空白部(scrub の対象)。
    Blank,
}

/// `point` がどこに当たったか。ルーラー帯(`ruler_height` 未満)は常に [`Hit::Blank`]。
/// 行の内側では bar の区間(`start..start+duration`)だけを [`Hit::Bar`] とし、
/// それ以外はその行の空白部として [`Hit::Blank`] を返す。
pub fn hit_test(
    point: Point,
    rows: &[RowProjection],
    ruler_height: f32,
    row_height: f32,
    width: f32,
    duration_frames: i64,
) -> Hit {
    if point.y < ruler_height || row_height <= 0.0 {
        return Hit::Blank;
    }
    let row_index = ((point.y - ruler_height) / row_height).floor();
    if row_index < 0.0 {
        return Hit::Blank;
    }
    let Some(row) = rows.get(row_index as usize) else {
        return Hit::Blank;
    };
    let start_x = frame_to_x(row.start, width, duration_frames);
    let end_x = frame_to_x(row.start + row.duration, width, duration_frames).max(start_x + 1.0);
    if point.x >= start_x && point.x < end_x {
        Hit::Bar(row.id)
    } else {
        Hit::Blank
    }
}

/// canvas の drag 状態。**Document でも Session でもない、widget 内だけの一時状態**
/// (iced の `slider` 等が持つ内部 drag state と同格)。書ける物は持っていない —
/// ここが持つ真偽値は「今 button が下がっているか」だけで、実際の書き込みは
/// 全て [`Message`] 経由で `Shell::update` に委ねる。
#[derive(Default)]
pub struct Interaction {
    dragging: bool,
}

/// Timeline pane 本体。1回の `view()` で作り捨てる、`StoreView`/`Session` の投影。
pub struct TimelinePane {
    rows: Vec<RowProjection>,
    markers: Vec<Marker>,
    playhead: i64,
    duration_frames: i64,
    fps: Option<Fps>,
    dims: Dimensions,
    colors: Colors,
}

impl TimelinePane {
    pub fn new(
        store: &StoreView<'_>,
        session: &Session,
        dims: Dimensions,
        colors: Colors,
    ) -> Self {
        let composition = store.composition().ok().flatten();
        Self {
            rows: rows(store, session),
            markers: store.markers().unwrap_or_default(),
            playhead: session.playhead,
            duration_frames: composition.as_ref().map(|c| c.duration_frames).unwrap_or(0),
            fps: composition.map(|c| c.fps),
            dims,
            colors,
        }
    }

    /// 第1波は測定済みの行高をそのまま流用する(独自の寸法を発明しない)。
    fn ruler_height(&self) -> f32 {
        self.dims.row_height
    }

    fn content_height(&self) -> f32 {
        self.ruler_height() + self.dims.row_height * self.rows.len() as f32
    }

    /// マーカーの comp フレーム位置。fps が引けない(comp が無い)時は `None` —
    /// 黙って誤った位置に描くより、描かない方がまし(M13 と同じ理由)。
    fn marker_frame(&self, marker: &Marker) -> Option<i64> {
        let fps = self.fps?;
        marker.time.try_to_frame_floor(fps).ok()
    }

    pub fn view(self) -> Element<'static, Message> {
        let height = self.content_height().max(self.ruler_height());
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .into()
    }
}

impl canvas::Program<Message> for TimelinePane {
    type State = Interaction;

    fn update(
        &self,
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
                    &self.rows,
                    self.ruler_height(),
                    self.dims.row_height,
                    bounds.width,
                    self.duration_frames,
                ) {
                    Hit::Bar(id) => {
                        Some(canvas::Action::publish(Message::Select(id)).and_capture())
                    }
                    Hit::Blank => {
                        state.dragging = true;
                        let frame = frame_at_x(position.x, bounds.width, self.duration_frames);
                        Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
                    }
                }
            }
            mouse::Event::CursorMoved { .. } => {
                if !state.dragging {
                    return None;
                }
                let position = cursor.position_in(bounds)?;
                let frame = frame_at_x(position.x, bounds.width, self.duration_frames);
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

    fn draw(
        &self,
        _state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let width = bounds.width;
        let ruler_height = self.ruler_height();
        let row_height = self.dims.row_height;

        // 背景。
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), self.colors.surface_panel);

        // ルーラー帯 + 目盛り。
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(width, ruler_height),
            self.colors.surface_raised,
        );
        self.draw_ruler_ticks(&mut frame, width, ruler_height);

        // マーカー(comp 側の名前つきロケータ)。ルーラー帯へ縦線として重ねる。
        for marker in &self.markers {
            if let Some(frame_no) = self.marker_frame(marker) {
                let x = frame_to_x(frame_no, width, self.duration_frames);
                let marker_path =
                    canvas::Path::line(Point::new(x, 0.0), Point::new(x, ruler_height));
                frame.stroke(
                    &marker_path,
                    canvas::Stroke::default()
                        .with_color(self.colors.way_timeline)
                        .with_width(2.0),
                );
            }
        }

        // 層の行。
        for (index, row) in self.rows.iter().enumerate() {
            let row_top = ruler_height + row_height * index as f32;

            if row.selected {
                frame.fill_rectangle(
                    Point::new(0.0, row_top),
                    Size::new(width, row_height),
                    self.colors.surface_hover,
                );
            }

            let start_x = frame_to_x(row.start, width, self.duration_frames);
            let end_x = frame_to_x(row.start + row.duration, width, self.duration_frames)
                .max(start_x + 1.0);
            let bar_color = if row.hidden {
                self.colors.text_muted
            } else {
                self.colors.way_timeline
            };
            frame.fill_rectangle(
                Point::new(start_x, row_top + 2.0),
                Size::new((end_x - start_x).max(1.0), (row_height - 4.0).max(1.0)),
                bar_color,
            );

            let label_color = if row.hidden {
                self.colors.text_muted
            } else {
                self.colors.text_primary
            };
            let label = if row.name.is_empty() {
                format!("layer {}", row.id.0)
            } else {
                row.name.clone()
            };
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new(start_x + 4.0, row_top + row_height / 2.0),
                color: label_color,
                size: iced::Pixels(self.dims.small_text),
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });
        }

        // playhead(Session が正本)。
        let playhead_x = frame_to_x(self.playhead, width, self.duration_frames);
        let playhead_path =
            canvas::Path::line(Point::new(playhead_x, 0.0), Point::new(playhead_x, bounds.height));
        frame.stroke(
            &playhead_path,
            canvas::Stroke::default()
                .with_color(self.colors.action_active)
                .with_width(1.5),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
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
            &self.rows,
            self.ruler_height(),
            self.dims.row_height,
            bounds.width,
            self.duration_frames,
        ) {
            Hit::Bar(_) => mouse::Interaction::Pointer,
            Hit::Blank => mouse::Interaction::default(),
        }
    }
}

impl TimelinePane {
    fn draw_ruler_ticks(&self, frame: &mut canvas::Frame, width: f32, height: f32) {
        if self.duration_frames <= 0 || width <= 0.0 {
            return;
        }
        const DIVISIONS: i64 = 8;
        for tick in 0..=DIVISIONS {
            let frame_no = (self.duration_frames - 1).max(0) * tick / DIVISIONS;
            let x = frame_to_x(frame_no, width, self.duration_frames);
            let tick_path =
                canvas::Path::line(Point::new(x, height - 4.0), Point::new(x, height));
            frame.stroke(
                &tick_path,
                canvas::Stroke::default()
                    .with_color(self.colors.border_strong)
                    .with_width(1.0),
            );
            frame.fill_text(canvas::Text {
                content: frame_no.to_string(),
                position: Point::new(x + 2.0, 0.0),
                color: self.colors.text_secondary,
                size: iced::Pixels(self.dims.small_text),
                ..Default::default()
            });
        }
    }
}
