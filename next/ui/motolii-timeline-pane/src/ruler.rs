//! ルーラー帯の常時固定ヘッダー(縦スクロール発注 2026-08-22、
//! `docs/reviews/2026-08-22-persona-lyric-mv-round2.md` — レーン一覧に縦
//! スクロールが無く、歌詞レイヤー50〜100枚が物理的に見えない欠落の根治)。
//!
//! ## 経緯・構造
//! `TimelinePane::view` は今まで `row![rail, field]` を1本の固定高
//! (ルーラー+全行の合計)で返しており、レーン数が増えると窓を超えて物理的に
//! 見えなくなっていた(縦スクロールが1つも無い)。この発注で:
//! - レーン一覧(`rail`)と行の canvas(`super::canvas` — 旧称「field」)を
//!   **同じ1つの `scrollable`** の中身にする(`TimelinePane::view` 参照)。
//!   別々の scrollable を2本同期させる手口(id 経由の `scroll_to`)は選ばな
//!   かった — 1本の scrollable が単一の子 `row![rail, canvas]` を丸ごと
//!   スクロールするので、rail の行位置と canvas の bar 位置が構造的に
//!   ずれ得ない(iced 自身の layout/clip/translate が両方へ同じオフセット
//!   を適用する。2つの scrollable を id で同期させる設計は「ずれ得る2つの
//!   正本」を作ってしまうので不採用 — T3b の「絵と当たりが別式でずれた」
//!   事故と同じ形の危険を、そもそも作らない)。
//! - **ルーラー・ループ帯・マーカー・playhead のルーラー内区間**は
//!   scrollable の**外**、常時固定のこの `RulerHeader` へ切り出した
//!   (発注 EXACT TARGET 4「playhead とルーラーは固定」)。scrollable に
//!   入れたままだと縦スクロールでルーラーごと流れてしまう —
//!   歌詞レイヤーを100枚下までスクロールした時にこそ、今どのフレームを
//!   見ているか(ルーラー)・scrub できる面(ルーラー)・作業範囲(ループ帯)が
//!   要る、という利用側の要求そのもの。
//! - `RulerHeader` は `TimelinePane` を直接借用しない
//!   (`canvas::Program` は実質 `'static` な値を要る — `TimelinePane`
//!   自体がこの後 `canvas(self)` で body 側へ move されるため、同じ
//!   `&pane` を生かしたまま header 用の借用を返すと move と衝突する)。
//!   代わりに [`RulerHeader::from_pane`] が必要な値だけ owned スナップ
//!   ショットへ写す — `super::rail::view` が widget を組み立てて即座に
//!   owned な `Element` を返すのと同じ「借用してから owned で持ち帰る」
//!   型(`markers: Vec<Marker>` の1回だけの複製はこの型の対価、`rows`/
//!   `property_rows`/`waveforms` のような毎フレーム重い物は複製しない)。
//! - 描画の実体(目盛り・hairline・ループ帯塗り・マーカー線・playhead 線)は
//!   [`super::canvas`] の `pub(crate)` 関数へ**そのまま移設**した(コード
//!   複製ではなく置き場の移動 — 唯一の出典は変わらず1箇所のまま)。
//! - 入力(scrub・ループ帯 drag)は旧 `super::input::loop_band_part_at`+
//!   `DragKind::Loop` 腕を**ここへ移設**した(body 側の `input.rs` はもう
//!   ループ帯を持たない — 行 canvas の y=0 が「行0の上端」になった今、
//!   ループ帯の判定式をそこに残すと最初の行の上端 11px 前後を誤ってループ
//!   帯として食ってしまう、という実害があるため移設は必須)。

use iced::widget::canvas;
use iced::{mouse, Element, Length, Point, Rectangle, Size};

use motolii_store::{Fps, Marker};

use super::projection::{frame_at_x, frame_to_x};
use super::work_area::{classify_loop_band, LoopBandPart, WorkArea};
use super::TimelinePane;
use crate::tokens::{Colors, Dimensions};
use crate::Message;

/// header canvas 専用の drag 種別。body 側 `input::DragKind` の姉妹型だが
/// `Clip` を持たない(ルーラーに bar は無い) — 別 enum にした理由は
/// モジュール doc の「入力の移設」節参照。
#[derive(Clone, Copy, PartialEq, Eq)]
enum HeaderDrag {
    Scrub,
    Loop,
}

/// `canvas::Program::State`。`input::Interaction` と同格(widget 内だけの
/// 一時状態、Document/Session ではない)。
#[derive(Default)]
pub(crate) struct HeaderState {
    drag: Option<HeaderDrag>,
}

/// 常時固定ヘッダー(ルーラー・ループ帯・マーカー・playhead のルーラー内
/// 区間)を持つ `canvas::Program`。`TimelinePane` から必要な値だけを写した
/// 使い捨てスナップショット(モジュール doc 参照)。
pub(crate) struct RulerHeader {
    duration_frames: i64,
    fps: Option<Fps>,
    dims: Dimensions,
    colors: Colors,
    work_area: Option<WorkArea>,
    loop_enabled: bool,
    markers: Vec<Marker>,
    playhead: i64,
}

impl RulerHeader {
    fn from_pane(pane: &TimelinePane) -> Self {
        Self {
            duration_frames: pane.duration_frames,
            fps: pane.fps,
            dims: pane.dims,
            colors: pane.colors,
            work_area: pane.work_area,
            loop_enabled: pane.loop_enabled,
            markers: pane.markers.clone(),
            playhead: pane.playhead,
        }
    }

    /// [`super::canvas::ruler_height`] と同じ比率(裁定172 §2)。ヘッダーの
    /// widget 高さそのもの — この crate 内でルーラー高を要る箇所は皆この
    /// 1つの比率(`canvas::ruler_height`)を経由する(`TimelinePane::ruler_height`
    /// と同型、2箇所で別の比を持たない)。
    fn ruler_height(&self) -> f32 {
        super::canvas::ruler_height(self.dims.row_height)
    }
}

/// `TimelinePane::view` から `&pane` の借用で呼ばれる(`pane` はこの後
/// canvas(self) で body 側へ move される、`rail::view` と同じ呼び出し順)。
pub(crate) fn view(pane: &TimelinePane) -> Element<'static, Message> {
    let header = RulerHeader::from_pane(pane);
    let height = header.ruler_height();
    canvas(header).width(Length::Fill).height(Length::Fixed(height)).into()
}

/// ループ帯の当たり判定(押した瞬間の座標1回だけ — 正典 §1)。`input.rs`
/// の同名ロジックの移設先(モジュール doc「入力の移設」節)。
fn loop_band_part_at(header: &RulerHeader, position: Point, width: f32) -> Option<LoopBandPart> {
    let band_height = super::canvas::loop_band_height(header.ruler_height());
    if position.y >= band_height {
        return None;
    }
    let span_x = header.work_area.map(|area| {
        let x0 = frame_to_x(area.start, width, header.duration_frames);
        let x1 = frame_to_x(area.end, width, header.duration_frames).max(x0 + 1.0);
        (x0, x1)
    });
    Some(classify_loop_band(position.x, span_x))
}

impl canvas::Program<Message> for RulerHeader {
    type State = HeaderState;

    fn update(
        &self,
        state: &mut HeaderState,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Mouse(mouse_event) = event else {
            return None;
        };
        let width = bounds.width;
        match mouse_event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                let position = cursor.position_in(bounds)?;
                if let Some(part) = loop_band_part_at(self, position, width) {
                    let at_frame = frame_at_x(position.x, width, self.duration_frames);
                    state.drag = Some(HeaderDrag::Loop);
                    return Some(
                        canvas::Action::publish(Message::LoopBandGrabbed { part, at_frame })
                            .and_capture(),
                    );
                }
                // ルーラー帯は bar を持たないので常に scrub 対象
                // (`hit::Hit::Blank` 相当 — body 側 `input::update` の
                // `Hit::Blank` 腕と同じ意味)。
                state.drag = Some(HeaderDrag::Scrub);
                let frame = frame_at_x(position.x, width, self.duration_frames);
                Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
            }
            // 右クリック = ループ帯ドラッグ中ならキャンセル(正典 §2 Esc・
            // 裁定151「キャンセルの一般化」— body 側 `input.rs` の `Clip` 腕と
            // 同型)。ドラッグ中でなければ locator lane 右クリック追加
            // (S2 発注 #22「追加 UI が無い」の穴埋め — キーボード M
            // (`Message::AddMarkerAt` doc 参照)と併存する2入口目、S6 併存
            // 裁定195)。**クリック位置ではなく playhead へ置く**(裁定222の
            // 外部資料判断: Premiere Pro は timeline ruler 右クリック→
            // Markers > Add Marker が M キーと同じく playhead へ置く
            // https://helpx.adobe.com/premiere/desktop/organize-media/apply-labeling/add-a-marker-to-a-clip.html、
            // DaVinci Resolve も jog bar 右クリック→Add Marker が playhead へ
            // 置く仕様 https://www.steakunderwater.com/VFXPedia/__man/Resolve18-6/DaVinciResolve18_Manual_files/part996.htm
            // — 2製品とも「右クリックした x 座標」ではなく「今の再生位置」を
            // 使う。他の作法(クリック位置へ置く)は先例に反するため採らない)。
            // scrub 中の右クリックはまだ意味を持たない(発明しない)。
            mouse::Event::ButtonPressed(mouse::Button::Right) => match state.drag {
                Some(HeaderDrag::Loop) => {
                    state.drag = None;
                    Some(canvas::Action::publish(Message::LoopDragCancelled).and_capture())
                }
                Some(HeaderDrag::Scrub) => None,
                None => Some(canvas::Action::publish(Message::AddMarkerAt(self.playhead)).and_capture()),
            },
            mouse::Event::CursorMoved { .. } => match state.drag {
                Some(HeaderDrag::Scrub) => {
                    let position = cursor.position_in(bounds)?;
                    let frame = frame_at_x(position.x, width, self.duration_frames);
                    Some(canvas::Action::publish(Message::ScrubTo(frame)).and_capture())
                }
                Some(HeaderDrag::Loop) => {
                    let position = cursor.position_in(bounds)?;
                    let at_frame = frame_at_x(position.x, width, self.duration_frames);
                    Some(canvas::Action::publish(Message::LoopDragMoved { at_frame }).and_capture())
                }
                None => None,
            },
            mouse::Event::ButtonReleased(mouse::Button::Left) => match state.drag.take() {
                Some(HeaderDrag::Scrub) => Some(canvas::Action::capture()),
                Some(HeaderDrag::Loop) => {
                    Some(canvas::Action::publish(Message::LoopDragReleased).and_capture())
                }
                None => None,
            },
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &HeaderState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let width = bounds.width;
        let ruler_height = self.ruler_height();
        let hairline = self.dims.border_width;

        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            Size::new(width, ruler_height),
            self.colors.surface_raised,
        );
        super::canvas::draw_ruler_ticks(
            self.fps,
            self.duration_frames,
            self.dims,
            self.colors,
            &mut frame,
            0.0,
            width,
            ruler_height,
        );
        super::canvas::draw_hairline(
            hairline,
            &mut frame,
            0.0,
            width,
            ruler_height,
            self.colors.border_default,
        );

        if let Some(area) = self.work_area {
            super::canvas::draw_loop_band(
                area,
                self.loop_enabled,
                self.duration_frames,
                self.colors,
                &mut frame,
                width,
                ruler_height,
            );
        }

        for marker in &self.markers {
            if let Some(frame_no) = super::canvas::marker_frame(marker, self.fps) {
                let x = frame_to_x(frame_no, width, self.duration_frames);
                super::canvas::draw_marker_line(self.colors.way_timeline, hairline, x, ruler_height, &mut frame);
            }
        }

        let playhead_x = frame_to_x(self.playhead, width, self.duration_frames);
        super::canvas::draw_playhead_line(self.colors, hairline, playhead_x, 0.0, ruler_height, &mut frame);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &HeaderState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.drag.is_some() {
            return mouse::Interaction::Grabbing;
        }
        let Some(position) = cursor.position_in(bounds) else {
            return mouse::Interaction::default();
        };
        let width = bounds.width;
        if let Some(part) = loop_band_part_at(self, position, width) {
            return match part {
                LoopBandPart::EdgeIn | LoopBandPart::EdgeOut => mouse::Interaction::ResizingHorizontally,
                LoopBandPart::Body => mouse::Interaction::Grab,
                LoopBandPart::Blank => mouse::Interaction::Crosshair,
            };
        }
        // 正典 §5.5「空白面=Crosshair」— ルーラーは常に scrub 可能な操作面。
        mouse::Interaction::Crosshair
    }
}
