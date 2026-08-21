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
//!   `time_band_segment_frames`/`property_rows`/`layer_row_top`)。
//!   Document/Session を読むだけ
//! - [`hit`] … `Hit`/[`hit::BarPart`] 型と `hit_test`/[`hit::classify_bar_part`]
//!   (座標 → 当たり判定、クリップ面専用)
//! - [`clip_gesture`] … 単一クリップの move/trim の**意味関数**(第2波T2、
//!   正典 §2)。スナップ・clamp。`motolii_store` を持たない自己完結な純関数
//! - [`canvas`] … 絵(`draw`/`draw_ruler_ticks`/`draw_hairline`/`draw_time_bands`)
//! - [`input`] … 入力(`update`/`mouse_interaction`)と drag 状態
//!   ([`Interaction`])
//! - [`lane_bar`] … レーンバー(行ヘッダ列、裁定147)専用の draw+hit。
//!   スウォッチ・名前・M/S/L トグル。自分のゾーン(`x < rail_width`)だけを
//!   自己完結で持ち、`hit`/`canvas` のクリップ面ロジックには触れない
//! - [`key_rows`] … property 行(キー行、第2波 T3・裁定148/151)専用の draw+hit。
//!   `Program::update` はここを**先に**試し、掴めなければ `input::update` へ
//!   落ちる(`x < rail_width` のレーンバーと同じ「自分のゾーンだけ自己完結」の形 —
//!   ただしゾーンは x ではなく y の帯: 選択 layer の下に挿入された property 行の
//!   `y` 範囲)。**`input.rs`/`hit.rs` は編集しない**(並走レーン lane-shell の
//!   write-set)ので、その2ファイルが知らない行の押し下げ
//!   ([`projection::layer_row_top`])が絡む座標はここで完結して吸収する
//!   (`projection::layer_row_top` の doc の write-set 外 finding 参照)。
//!
//! `canvas::Program` は1トレイトにつき1つの impl しか持てない(Rust の制約)ので、
//! 本体の trait impl はここ(mod.rs)に置き、各メソッドは対応する層の関数へ
//! 委譲するだけ — 挙動は変えず、置き場所だけを変える(第2波第1切片: 純粋な
//! ファイル分割)。
//!
//! 旧 `crate::timeline_pane` への参照(`screenshot.rs`・`tests/suite/*.rs`)は
//! `crate::lib`側の `pub use timeline as timeline_pane;` エイリアスで壊さない。

mod canvas;
pub mod clip_gesture;
mod hit;
mod input;
mod key_rows;
mod lane_bar;
mod projection;

pub use hit::{bar_span_x, classify_bar_part, hit_test, BarPart, Hit, TRIM_EDGE};
pub use input::Interaction;
pub use projection::{
    frame_at_x, key_order, property_rows, rows, KeySelectionOp, KeySelector, PropertyKeyProjection,
    PropertyRowProjection, RowProjection,
};
pub(crate) use projection::{frame_to_x, layer_row_top, selected_row_index, time_band_segment_frames};

use iced::{Element, Length, Rectangle};

use motolii_store::{Fps, Marker, StoreView};

use crate::tokens::{Colors, Dimensions};
use crate::{Message, Session};

/// Timeline pane 本体。1回の `view()` で作り捨てる、`StoreView`/`Session` の投影。
pub struct TimelinePane {
    rows: Vec<RowProjection>,
    /// 選択 layer の下に挿入する property 行(キーを持つ property だけ、
    /// 裁定151・EXACT TARGET 1)。選択が無い/キーが無ければ空。
    property_rows: Vec<PropertyRowProjection>,
    /// `rows` 内で選択 layer が占める添字。property 行はこの直後に挿入される
    /// (`projection::layer_row_top` が押し下げの基準に使う)。
    selected_row_index: Option<usize>,
    markers: Vec<Marker>,
    playhead: i64,
    duration_frames: i64,
    fps: Option<Fps>,
    dims: Dimensions,
    colors: Colors,
    /// 直近の修飾キー状態(`Shell::keyboard_modifiers`)。property 行のキー選択
    /// (単独/Cmd トグル/Shift 範囲、正典 §3・§4 と同じ文法)がここを読む —
    /// canvas の `mouse::Event` 自体は修飾キーを運ばないため([`input`] の
    /// drag-to-scrub が既に同じ理由で `Shell::keyboard_modifiers` を別経路で
    /// 持ち回っているのと同じ形)。
    modifiers: iced::keyboard::Modifiers,
}

impl TimelinePane {
    pub fn new(
        store: &StoreView<'_>,
        session: &Session,
        dims: Dimensions,
        colors: Colors,
        modifiers: iced::keyboard::Modifiers,
    ) -> Self {
        let composition = store.composition().ok().flatten();
        let fps = composition.as_ref().map(|c| c.fps);
        let rows = rows(store, session);
        let selected_row_index = selected_row_index(&rows, session);
        let property_rows = property_rows(store, session, fps);
        Self {
            duration_frames: composition.as_ref().map(|c| c.duration_frames).unwrap_or(0),
            rows,
            property_rows,
            selected_row_index,
            markers: store.markers().unwrap_or_default(),
            playhead: session.playhead,
            fps,
            dims,
            colors,
            modifiers,
        }
    }

    /// 第1波は測定済みの行高をそのまま流用する(独自の寸法を発明しない)。
    fn ruler_height(&self) -> f32 {
        self.dims.row_height
    }

    /// レーンバー(行ヘッダ列)幅。座標シフトの唯一の出典 — ルーラ/クリップ面は
    /// この値ぶん右へずらして描く(`projection::frame_to_x`/`frame_at_x` 自体は
    /// 汚さない、mod doc 参照)。
    fn rail_width(&self) -> f32 {
        self.dims.timeline_lane_bar_width
    }

    /// property 行(キー行)1本の高さ。**本行(`row_height`)より低い**
    /// (egui 版 `timeline_editor::PROP_H`/`ROW_H` の比を踏襲、EXACT TARGET 1)。
    fn param_row_height(&self) -> f32 {
        self.dims.timeline_param_row_height
    }

    /// この行 `index`(`rows` 内)の描画 top(ルーラー下相対)。選択 layer の下に
    /// property 行が挿入されている間、それより後ろの層行を押し下げる
    /// (`projection::layer_row_top` 参照 — write-set 外 finding も同 doc)。
    fn layer_row_top(&self, index: usize) -> f32 {
        layer_row_top(
            self.dims.row_height,
            self.param_row_height(),
            self.property_rows.len(),
            self.selected_row_index,
            index,
        )
    }

    fn content_height(&self) -> f32 {
        self.ruler_height()
            + self.dims.row_height * self.rows.len() as f32
            + self.param_row_height() * self.property_rows.len() as f32
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
        // property 行(キー行)の帯を**先に**試す(mod doc の [`key_rows`] 節)。
        // `input.rs`/`hit.rs` はこの帯の押し下げを知らないので、当たれば
        // ここで完結して吸収し、外れた時だけ旧来の経路へ渡す。
        key_rows::update(self, event, bounds, cursor).or_else(|| input::update(self, state, event, bounds, cursor))
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
        key_rows::mouse_interaction(self, bounds, cursor)
            .unwrap_or_else(|| input::mouse_interaction(self, state, bounds, cursor))
    }
}
