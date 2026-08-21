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
//!
//! ## 層の分担(第2波レーン分割時の write-set の割り当て先)
//! - [`projection`] … 投影の純関数(`rows`/`frame_to_x`/`frame_at_x`/
//!   `time_band_segment_frames`)。Document/Session を読むだけ
//! - [`hit`] … `Hit` 型と `hit_test`(座標 → 当たり判定)
//! - [`canvas`] … 絵(`draw`/`draw_ruler_ticks`/`draw_hairline`/`draw_time_bands`)
//! - [`input`] … 入力(`update`/`mouse_interaction`)と drag 状態
//!   ([`Interaction`])
//!
//! `canvas::Program` は1トレイトにつき1つの impl しか持てない(Rust の制約)ので、
//! 本体の trait impl はここ(mod.rs)に置き、各メソッドは対応する層の関数へ
//! 委譲するだけ — 挙動は変えず、置き場所だけを変える(第2波第1切片: 純粋な
//! ファイル分割)。
//!
//! 旧 `crate::timeline_pane` への参照(`screenshot.rs`・`tests/suite/*.rs`)は
//! `crate::lib`側の `pub use timeline as timeline_pane;` エイリアスで壊さない。

mod canvas;
mod hit;
mod input;
mod projection;

pub use hit::{hit_test, Hit};
pub use input::Interaction;
pub use projection::{frame_at_x, rows, RowProjection};
pub(crate) use projection::{frame_to_x, time_band_segment_frames};

use iced::{Element, Length, Rectangle};

use motolii_store::{Fps, Marker, StoreView};

use crate::tokens::{Colors, Dimensions};
use crate::{Message, Session};

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

impl iced::widget::canvas::Program<Message> for TimelinePane {
    type State = Interaction;

    fn update(
        &self,
        state: &mut Interaction,
        event: &iced::widget::canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<iced::widget::canvas::Action<Message>> {
        input::update(self, state, event, bounds, cursor)
    }

    fn draw(
        &self,
        _state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        canvas::draw(self, renderer, bounds)
    }

    fn mouse_interaction(
        &self,
        state: &Interaction,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        input::mouse_interaction(self, state, bounds, cursor)
    }
}
