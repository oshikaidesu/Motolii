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
///
/// `pub(crate)`: screenshot 器具(`crate::screenshot`)が Timeline canvas と同じ
/// x 座標計算を使うため(マーカー・bar の位置を2箇所で別の式にしない)。
pub(crate) fn frame_to_x(frame: i64, width: f32, duration_frames: i64) -> f32 {
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

/// 時間方向の明暗リズム(裁定148(1))の区間幅(フレーム数)。fps が引ければ
/// 1秒、引けなければ [`RULER_TICK_DIVISIONS`] 等分へ落ちる。`draw_time_bands`
/// と screenshot 器具の両方がこの1つの式から区間境界を出す(2箇所で別の
/// フォールバックを持たない)。
///
/// `pub(crate)`: `crate::screenshot` が Timeline canvas と同じ区間の刻み方を
/// 再現するため(`frame_to_x` と同じ理由)。
pub(crate) fn time_band_segment_frames(fps: Option<Fps>, duration_frames: i64) -> i64 {
    fps.map(|fps| fps.as_f64().round().max(1.0) as i64)
        .unwrap_or_else(|| (duration_frames / RULER_TICK_DIVISIONS).max(1))
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
    pub fn new(store: &StoreView<'_>, session: &Session, dims: Dimensions, colors: Colors) -> Self {
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
        // 罫線幅の倍数で意味を分ける — 1x: ルーラー目盛り(hairline)、1.5x: playhead、
        // 2x: マーカー(最も強い accent)。新しい寸法トークンを増やさず、単一の
        // `border_width` から比で導出する(裁定117 の「寸法は token 経由」の範囲内)。
        let hairline = self.dims.border_width;

        // 背景。
        frame.fill_rectangle(Point::ORIGIN, bounds.size(), self.colors.surface_panel);

        // ルーラー帯 + 目盛り。
        frame.fill_rectangle(
            Point::ORIGIN,
            Size::new(width, ruler_height),
            self.colors.surface_raised,
        );
        self.draw_ruler_ticks(&mut frame, width, ruler_height);

        // ルーラー帯とクリップ面の境界(裁定139: 面色の塗り分け=`surface_raised`
        // だけに頼らず hairline を足す — `.tp`/`.ruler` が border-bottom を持つ
        // mock と同じ扱い。ルーラーは「地」の第2段なので不透明な強い hairline
        // (`border_default`、`.cols`/`.ptitle` と同じロール)を使う)。
        self.draw_hairline(&mut frame, 0.0, width, ruler_height, self.colors.border_default);

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
                        .with_width(hairline * 2.0),
                );
            }
        }

        // 明暗のリズム(裁定148・正典 §1.6): クリップ面の「地」に2方向の読解補助を
        // 重ねる。**区切りの手段ではない**(裁定137 との両立整理) — 区切りは
        // 上の hairline と、行ごとの下 hairline([`Self::draw`] 末尾)が担う。
        // 順序は 行方向ゼブラ → 時間方向 の順で薄い wash を積む(どちらも
        // token 経由の白 wash、raw 値直書きではない)。
        let rows_top = ruler_height;
        let rows_bottom = self.content_height();
        for index in 0..self.rows.len() {
            if index % 2 == 0 {
                continue; // 偶数行は地のまま(奇数行だけへ wash を乗せる)。
            }
            let row_top = rows_top + row_height * index as f32;
            frame.fill_rectangle(
                Point::new(0.0, row_top),
                Size::new(width, row_height),
                self.colors.timeline_row_zebra,
            );
        }
        self.draw_time_bands(&mut frame, width, rows_top, rows_bottom);

        // 層の行。
        for (index, row) in self.rows.iter().enumerate() {
            let row_top = ruler_height + row_height * index as f32;

            if row.selected {
                // 状態: 選択(`state_selected`)。hover(`surface_hover`、中立グレー)とは
                // 別ロール — 選択は accent 味、hover は明度差だけ(意味色ロールの区別)。
                frame.fill_rectangle(
                    Point::new(0.0, row_top),
                    Size::new(width, row_height),
                    self.colors.state_selected,
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
                Point::new(start_x, row_top + self.dims.spacing_xs),
                Size::new(
                    (end_x - start_x).max(1.0),
                    (row_height - self.dims.spacing_s).max(1.0),
                ),
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
                position: Point::new(start_x + self.dims.spacing_s, row_top + row_height / 2.0),
                color: label_color,
                size: iced::Pixels(self.dims.caption_text),
                align_y: iced::alignment::Vertical::Center,
                ..Default::default()
            });

            // 行の区切り(裁定139: 面色の塗り分け=ゼブラの明暗だけに頼らず
            // hairline を足す — mock `.trow{border-bottom:...}` と同じ役目)。
            // 行同士は `.prow` と同じ弱い hairline ロール(区切り=線、
            // リズム=地の微差 — §1.6 の両立整理どおり見て区別がつく)。
            self.draw_hairline(
                &mut frame,
                0.0,
                width,
                row_top + row_height,
                self.colors.border_hairline_weak,
            );
        }

        // playhead(Session が正本)。
        let playhead_x = frame_to_x(self.playhead, width, self.duration_frames);
        let playhead_path = canvas::Path::line(
            Point::new(playhead_x, 0.0),
            Point::new(playhead_x, bounds.height),
        );
        frame.stroke(
            &playhead_path,
            canvas::Stroke::default()
                .with_color(self.colors.action_active)
                .with_width(hairline * 1.5),
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

/// ルーラー目盛りの分割数。fps が引けない(comp 無し)時の時間方向リズム
/// ([`TimelinePane::draw_time_bands`])のフォールバックも同じ分割を使う —
/// 「ルーラーと違う区間の刻み方」という新しい規則を増やさない。
/// `pub(crate)`: `screenshot.rs` 器具が同じ区間の刻み方を再現するのにも使う
/// (`frame_to_x` と同じ「同じ位置関係を再現する」理由)。
pub(crate) const RULER_TICK_DIVISIONS: i64 = 8;

impl TimelinePane {
    fn draw_ruler_ticks(&self, frame: &mut canvas::Frame, width: f32, height: f32) {
        if self.duration_frames <= 0 || width <= 0.0 {
            return;
        }
        for tick in 0..=RULER_TICK_DIVISIONS {
            let frame_no = (self.duration_frames - 1).max(0) * tick / RULER_TICK_DIVISIONS;
            let x = frame_to_x(frame_no, width, self.duration_frames);
            let tick_path = canvas::Path::line(
                Point::new(x, height - self.dims.spacing_s),
                Point::new(x, height),
            );
            frame.stroke(
                &tick_path,
                canvas::Stroke::default()
                    .with_color(self.colors.border_strong)
                    .with_width(self.dims.border_width),
            );
            frame.fill_text(canvas::Text {
                content: frame_no.to_string(),
                position: Point::new(x + self.dims.spacing_xs, 0.0),
                color: self.colors.text_secondary,
                size: iced::Pixels(self.dims.caption_text),
                ..Default::default()
            });
        }
    }

    /// 水平の hairline を1本引く(`Point`/`Size` を毎回組まずに済む共通口)。
    /// `inspector_pane.rs::bordered_row` の canvas 版 — こちらは per-edge の
    /// border-bottom そのもの(canvas は4辺一律の制約が無いので、Inspector側の
    /// 「既知の限界」はここには適用されない)。
    fn draw_hairline(&self, frame: &mut canvas::Frame, x0: f32, x1: f32, y: f32, color: iced::Color) {
        let path = canvas::Path::line(Point::new(x0, y), Point::new(x1, y));
        frame.stroke(
            &path,
            canvas::Stroke::default()
                .with_color(color)
                .with_width(self.dims.border_width),
        );
    }

    /// 時間方向の明暗リズム(裁定148(1)・正典 §1.6)。区間幅は fps が引ければ
    /// 1秒(Ableton の拍グリッド陰影と同型)、comp が無く fps が引けない時は
    /// ルーラー目盛りと同じ [`RULER_TICK_DIVISIONS`] 分割へ落ちる(新しい
    /// フォールバック規則を増やさない)。奇数番目の区間にだけ
    /// `timeline_time_band` の薄い wash を乗せる — 偶数番目は地のまま
    /// (行方向ゼブラと同じ「交互」の言葉遣い)。
    fn draw_time_bands(&self, frame: &mut canvas::Frame, width: f32, top: f32, bottom: f32) {
        if self.duration_frames <= 0 || width <= 0.0 || bottom <= top {
            return;
        }
        let segment_frames = time_band_segment_frames(self.fps, self.duration_frames);

        let mut segment_index: i64 = 0;
        let mut start_frame: i64 = 0;
        while start_frame < self.duration_frames {
            let end_frame = (start_frame + segment_frames).min(self.duration_frames);
            if segment_index % 2 == 1 {
                let x0 = frame_to_x(start_frame, width, self.duration_frames);
                let x1 = frame_to_x(end_frame, width, self.duration_frames).max(x0 + 1.0);
                frame.fill_rectangle(
                    Point::new(x0, top),
                    Size::new(x1 - x0, bottom - top),
                    self.colors.timeline_time_band,
                );
            }
            start_frame = end_frame;
            segment_index += 1;
        }
    }
}
