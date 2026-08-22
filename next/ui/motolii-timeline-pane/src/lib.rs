//! wraps: iced — Timeline pane(投影・hit・ジェスチャ・レーンバー・キー行)。書き込みは Intent 経由のみ、Document の写しを持たない。
//! Timeline pane crate(裁定160 切片7、pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §6 切片7)。
//!
//! `motolii-shell/src/timeline/`(第2波第1切片で分割済みの9ファイル)+
//! `motolii-shell/src/lib.rs` 内の Timeline 書き込みロジック(§1.2 の
//! 584行主部: レーンバー M/S/L 以外の move/trim・キー選択・キー時刻
//! ドラッグ/リタイム・NudgeKeyframe)を、この crate へ**丸ごと**移した。
//!
//! ## 層の分担(元の `timeline/mod.rs` の節をそのまま踏襲)
//! - [`projection`] … 投影の純関数。Document/Session を読むだけ
//! - [`hit`] … 当たり判定(クリップ面専用)
//! - [`clip_gesture`] … 単一クリップの move/trim の意味関数(純関数)
//! - [`canvas`] … 絵
//! - [`input`] … 入力(`update`/`mouse_interaction`)と drag 状態
//! - [`lane_bar`] … レーンバーの比率・測定純関数(裁定172 §2 のスウォッチ/
//!   M・S・L 寸法比)。**draw/hit は持たない**(TL-arch Phase 1 で `rail` へ
//!   移設済み — モジュール doc 参照)
//! - [`rail`] … **新設**(TL-arch Phase 1、
//!   `docs/reviews/2026-08-22-timeline-canvas-widget-survey.md` §6)。
//!   レーンバー(行=container・スウォッチ=着色container・名前=widget text・
//!   M/S/L=実button)を canvas 手描きから実 widget へ置換。時間場(bar・
//!   ルーラー・菱形)は canvas のまま(Phase 2 の範囲、NON-GOALS)
//! - [`key_rows`] … property 行専用の draw+hit(rail 側の property 名だけ
//!   `rail` へ委譲、帯/菱形は canvas のまま) + キー時刻ドラッグ/リタイム
//! - [`key_gesture`] … キーの時刻編集の意味関数(純関数)
//! - [`nav`] … playhead ナビゲーション動詞束の意味関数(純関数。
//!   キー→`Message` の解決自体は `motolii-shell` 側に残る)
//! - `write` … **新設**(裁定160 切片7)。pane-local [`Message`] と
//!   [`PaneState`](旧 `Shell::timeline_drag`/`timeline_key_drag` + 対応する
//!   private メソッド群の移設先)。
//!
//! ## pane-local `Message`(survey §3.1)
//!
//! pane crate は root(`motolii-shell`)の `Message` を参照できない(循環に
//! なる)ので、`timeline/input.rs`・`timeline/key_rows.rs` は widget
//! コールバックの中でこの crate 自身の [`Message`] を組み立てる。
//! `motolii-shell` 側は `Message::Timeline(motolii_timeline_pane::Message)`
//! で1回だけ畳む([`write`] モジュール doc の「例外」節も参照)。
//!
//! ## 依存(survey §5 layer1)
//!
//! `motolii-tokens-rs`(寸法・色)・`motolii-shell-state`(`Session`/
//! `KeySelector` — root と pane 両方が読む共通の親、循環回避に必須)・
//! `motolii-store`・`iced` のみ。`motolii-shell`/他 pane crate への依存はゼロ。

mod canvas;
pub mod clip_gesture;
mod hit;
mod input;
pub mod key_gesture;
mod key_rows;
mod lane_bar;
pub mod markers;
pub mod nav;
mod projection;
mod rail;
pub mod rows;
pub mod shuttle;
pub mod split;
pub mod stacking;
mod transport;
pub mod work_area;
mod write;

pub use hit::{bar_span_x, classify_bar_part, hit_test, BarPart, Hit, TRIM_EDGE};
pub use input::Interaction;
pub use projection::{
    frame_at_x, key_order, property_rows, rows, KeySelectionOp, KeySelector, PropertyKeyProjection,
    PropertyRowProjection, RowProjection,
};
/// **裁定160 切片7で `pub(crate)` → `pub` に緩めた**(`projection.rs` 側の
/// 個々の宣言も同様)。`motolii-shell::screenshot`(cross-cutting な検分器具、
/// pane split survey §2.5 — どの pane crate にも属さず assembler 側に残る)が
/// これらを cross-crate で読む必要がある — 分割前は同一クレート内の
/// `pub(crate)` で足りていたが、crate 境界を跨ぐには `pub` が要る。
pub use projection::{
    frame_to_x, layer_row_top, selected_row_index, tick_steps, time_band_segment_frames,
};
/// 裁定172 §1/§2 の比率(ruler 高・bar 縦 inset/角丸・目盛り長)— 上と同じ理由
/// (`motolii-shell::screenshot` の cross-crate 参照)で `pub` にした。**この
/// レーンの write-set は `next/ui/motolii-timeline-pane/src/**` のみ**
/// (screenshot.rs は shell/M4 の領分)— screenshot 側が独自に持つ重複式
/// (`ruler_h = dims.row_height`・inset = `spacing_xs` 等)は、この export を
/// 使うよう置き換えるのが次の一手(未着手、report で明記)。
pub use canvas::{
    bar_corner_radius, bar_inset, loop_band_height, major_tick_length, minor_tick_length,
    ruler_height,
};
pub use lane_bar::glyph_size_px;
/// JKL シャトル(B21)と作業範囲/ループ帯(B18)の意味型 — shell の
/// PlaybackClock/keymap 層がこの型で結線する(モジュール doc 参照)。
pub use shuttle::{ShuttleCommand, ShuttleState, MAX_SHUTTLE_RATE};
/// Transport 帯(map 1041-1045・1138)の宣言 spec — テスト(絵と意味の対応)
/// と shell 側検分器具の両方が widget 木の代わりに読む継ぎ目。
pub use transport::{transport_spec, TransportButton, TransportSpec};
pub use work_area::{classify_loop_band, LoopBandPart, WorkArea, LOOP_GRAB};
/// Stage 重なり並べ替え(第3切片)の意味型 — shell の keymap/メニュー層が
/// `Message::RestackLayer(StackDirection)` で結線する。
pub use stacking::StackDirection;
/// Easy Ease 系プリセット(map 485〜490)— keymap/メニュー層が
/// `Message::SetKeyInterp(EASY_EASE)` 等で渡す定数(`write.rs` の doc 参照)。
pub use write::{EASY_EASE, EASY_EASE_IN, EASY_EASE_OUT};
pub use write::{Message, PaneState};

use iced::widget::row;
use iced::{Element, Length, Rectangle};

use motolii_store::{Fps, LayerId, LayerTiming, Marker, StoreView};

/// `Session` は `motolii-shell-state` crate(裁定160 切片6→切片7で crate 化)。
/// `motolii-shell` 側の `pub use motolii_shell_state as state;` と同じ
/// 「型 alias で外部参照を壊さない」手口 — `crate::state::Session` を読む
/// 既存参照(`projection.rs`・`write.rs`)は無改修で済む。
pub use motolii_shell_state as state;
/// `tokens` も同様(`motolii-tokens-rs` crate、裁定160 切片1)。
pub use motolii_tokens_rs as tokens;

use state::Session;
use tokens::{Colors, Dimensions};

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
    /// Timeline キーの時刻ドラッグ/リタイムが進行中か(第2波T4)。**Shell 状態
    /// (`PaneState::key_drag_active`)を読み取り専用で pane へ運ぶだけ** —
    /// `modifiers` と同じ形で、canvas の `mouse::Event` 自体は「今 drag 中か」を
    /// 運ばないので別経路が要る。既定 `false`([`Self::with_key_drag_active`]
    /// でしか立たない) — 呼び出し元(試験含む)は継続イベントを試験しない限り
    /// 触らなくてよい。
    key_drag_active: bool,
    /// 第2波T5(正典 §5.5「プレビューは毎フレーム」): [`Self::with_clip_preview`]
    /// / [`Self::with_key_preview`] のどちらかが `Some` を渡した(=今フレーム
    /// ドラッグ中)ことを覚えるだけの読み取り専用フラグ。`canvas::draw` が
    /// ポインタ近くのタイムコードミニラベルを出すかどうかの判断材料
    /// (`key_drag_active` と同じ「Shell 状態を pane へ運ぶ」形だが、こちらは
    /// 2つの builder のどちらが立てても真になる)。
    preview_active: bool,
    /// 実時間再生が進行中か(transport 帯の Play‖Pause の顔が読む)。
    /// `modifiers`/`key_drag_active` と同じ「Shell 状態を読み取り専用で pane へ
    /// 運ぶだけ」の形 — 既定 `false`([`Self::with_playing`] でしか立たない)。
    playing: bool,
    /// 作業範囲(In-Out、B18 第1切片・正典 §5「ループ帯」)。`PaneState::work_area()`
    /// を [`Self::with_work_area`] で運ぶだけ(`playing` と同じ形)。ルーラ最上段の
    /// 帯の絵([`canvas`])と当たり([`input`])が読む。
    work_area: Option<WorkArea>,
    /// ループ on/off(map 1082/1083)。帯の ink(on=accent)と transport の
    /// ループボタンの顔が読む。
    loop_enabled: bool,
    /// inline rename の進行中下書き(第3切片、正典 §6)。`PaneState::rename_draft()`
    /// を [`Self::with_rename`] で運ぶだけ(`work_area` と同じ形)。`Some` の間、
    /// rail の該当行の名前 text が `text_input` に差し替わる(`rail::layer_row`)。
    rename: Option<(LayerId, String)>,
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
            key_drag_active: false,
            preview_active: false,
            playing: false,
            work_area: None,
            loop_enabled: false,
            rename: None,
        }
    }

    /// `Shell::view` だけが呼ぶ(transport 帯)。`Shell::is_playing()` 相当を
    /// そのまま渡すだけの薄い builder — `with_key_drag_active` と同じ形
    /// (既存の呼び出し元・試験を1つも壊さない)。
    pub fn with_playing(mut self, playing: bool) -> Self {
        self.playing = playing;
        self
    }

    /// `Shell::view` だけが呼ぶ(B21+B18 第1切片)。`PaneState::work_area()`/
    /// `loop_enabled()` をそのまま渡すだけの薄い builder — `with_playing` と
    /// 同じ形(既存の呼び出し元・試験を1つも壊さない)。
    pub fn with_work_area(mut self, area: Option<WorkArea>, loop_enabled: bool) -> Self {
        self.work_area = area;
        self.loop_enabled = loop_enabled;
        self
    }

    /// `Shell::view` だけが呼ぶ(第3切片、正典 §6「リネーム」)。
    /// `PaneState::rename_draft()` を owned へ写してそのまま渡すだけの薄い
    /// builder — `with_work_area` と同じ形(既存の呼び出し元・試験を1つも
    /// 壊さない)。
    pub fn with_rename(mut self, rename: Option<(LayerId, String)>) -> Self {
        self.rename = rename;
        self
    }

    /// `Shell::view` だけが呼ぶ(第2波T4)。`PaneState::key_drag_active()`
    /// をそのまま渡すだけの薄い builder — 新しい `TimelinePane::new` の必須引数に
    /// しない(既存の呼び出し元・試験を1つも壊さないため、`with_` 形にしてある)。
    pub fn with_key_drag_active(mut self, active: bool) -> Self {
        self.key_drag_active = active;
        self
    }

    /// `Shell::view`(実際には `Shell::build_timeline_pane`)だけが呼ぶ
    /// (第2波T5、正典 §5.5「プレビューは毎フレーム」)。`PaneState::clip_preview()`
    /// の `(layer, preview timing)` をそのまま渡すだけの薄い builder —
    /// `with_key_drag_active` と同じ形。**置換そのものは
    /// [`projection::apply_clip_preview`](投影段の純関数)がやる** — ここは
    /// 運ぶだけで if を持たない。`None`(非ドラッグ中)なら `rows` は無傷。
    pub fn with_clip_preview(mut self, preview: Option<(LayerId, LayerTiming)>) -> Self {
        if preview.is_some() {
            self.preview_active = true;
        }
        self.rows = projection::apply_clip_preview(self.rows, preview);
        self
    }

    /// 同上(第2波T5)、キー drag/リタイム版。`preview` は「掴んだ瞬間の
    /// selector(旧 frame)→ 新 frame」のペア列 — `PaneState::key_preview()` の
    /// 結果を呼び出し側がそのまま渡す(EXACT TARGET 4: リタイム中は選択キー
    /// 全部がこの1本の列に並ぶので、move/retime を pane 側で区別しない)。
    /// 置換は [`projection::apply_key_preview`] へ委譲。
    pub fn with_key_preview(mut self, preview: Option<Vec<(KeySelector, i64)>>) -> Self {
        if preview.is_some() {
            self.preview_active = true;
        }
        self.property_rows = projection::apply_key_preview(self.property_rows, preview.as_deref());
        self
    }

    /// **運転席専用**(第2波T5): `with_clip_preview`/`with_key_preview` 適用後の
    /// 行 — `canvas::draw` が実際に描くのと同じ値。`Shell::timeline_rows()`
    /// (Document を直読みする投影、preview 抜き)とは別物 — ドラッグ中に両者が
    /// 食い違うこと自体が「プレビューが絵に届いている」ことの証拠になる。
    pub fn rows(&self) -> &[RowProjection] {
        &self.rows
    }

    /// 同上、property 行(キー行)版。
    pub fn property_rows(&self) -> &[PropertyRowProjection] {
        &self.property_rows
    }

    /// ルーラー帯の高さ(裁定172 §2: `0.846×行高` — mock `timeline-semantics.html`
    /// `.ruler{height:22px}`/`.row{height:26px}` の実測 `22/26`)。第1波の
    /// 「行高をそのまま流用」(裁定167 違反 — 独自の中間値ではなく実測比を使う)
    /// を裁定172 で廃した。比の出典は [`canvas::ruler_height`] のみ(このメソッドと
    /// `motolii-shell::screenshot` 両方の重複を避けたいが、screenshot 側は
    /// crate 境界外の別 instrument — pane split survey §2.5 により M4/shell
    /// レーンの担当、裁定172 のこのレーンの write-set 外)。
    fn ruler_height(&self) -> f32 {
        canvas::ruler_height(self.dims.row_height)
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

    /// **TL-arch Phase 1**(`docs/reviews/2026-08-22-timeline-canvas-widget-survey.md`
    /// §6): rail(行ヘッダ列)を実 widget として左に置き、canvas は時間場
    /// (bar・ルーラー・菱形)だけを描く。`rail::view` は `&self` の借用で
    /// widget を組み立て終える(rows/property_rows/dims/colors を読むだけ) —
    /// その後で `self` を `canvas(self)` へ move する(`Program` impl は
    /// `self` を値で持つ、下の `impl canvas::Program` 参照)。
    ///
    /// canvas の x 原点は rail の右端へ移る(旧: `rail_width` を各所で
    /// 足していた/`bounds.width` から引いていた → 新: `canvas.rs`/`hit.rs`/
    /// `input.rs`/`key_rows.rs` はどれも「自分の bounds がそのまま時間場」
    /// という前提へ揃えた、意味は不変 — 発注書「座標系は関数境界で吸収」)。
    pub fn view(self) -> Element<'static, Message> {
        let height = self.content_height().max(self.ruler_height());
        let rail = rail::view(&self);
        let field = iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fixed(height));
        row![rail, field].height(Length::Fixed(height)).into()
    }

    /// [`Self::view`] の上に transport 帯(map 1041-1045・1138、
    /// [`transport`] モジュール doc)を積んだ版。**`view()` 自体は無改変で
    /// 残す** — shell 側の既存検分(`iced_test_spike.rs` が `pane.view()` へ
    /// 生座標で click する)は canvas の y 原点がルーラー上端であることを
    /// 前提にしており、`view()` に帯を差し込むとその前提ごと壊れる。shell の
    /// 統合(下部 Play バー撤去と同時)はこの `view_with_transport()` へ
    /// 呼び出し1行を差し替えるだけ(supervisor、RETURN の結線一覧)。
    pub fn view_with_transport(self) -> Element<'static, Message> {
        let band = transport::view(&self);
        let body = self.view();
        iced::widget::column![band, body].into()
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
        cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        canvas::draw(self, renderer, bounds, cursor)
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
