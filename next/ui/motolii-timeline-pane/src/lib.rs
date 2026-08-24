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
mod viewport_canvas;
pub mod clip_gesture;
mod hit;
mod input;
pub mod key_gesture;
mod key_rows;
/// 第4切片(B15 キーフレーム束+B20 再生ヘッド移動束の残り)の部品。
/// `key_gesture`/`nav` と同格の意味の純関数置き場 — モジュール doc 参照。
pub mod keys2;
mod lane_bar;
pub mod markers;
pub mod nav;
mod projection;
mod rail;
pub mod rows;
/// 常時固定ヘッダー(ルーラー・ループ帯・マーカー・playhead のルーラー内
/// 区間、縦スクロール発注 2026-08-22)。`rail`/`canvas`(行だけ)と対で
/// `TimelinePane::view` が組む — モジュール doc 参照。
mod ruler;
pub mod shuttle;
pub mod split;
pub mod stacking;
mod transport;
pub mod waveform_view;
pub mod work_area;
mod write;

pub use hit::{bar_span_x, classify_bar_part, hit_test, BarPart, Hit, TRIM_EDGE};
pub use input::Interaction;
pub use projection::{
    frame_at_x, key_order, property_rows, rows, KeySelectionOp, KeySelector, PropertyKeyProjection,
    PropertyRowProjection, RowProjection,
};
/// 音声の有無 + ソース path(TL7 統合手順1)。`RowProjection` の兄弟型 —
/// `projection.rs` の `AudioRowProjection` doc 参照(`RowProjection` 本体を
/// 変えない理由)。
pub use projection::{audio_rows, AudioRowProjection};
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
    /// 波形取得状態(TL7 統合手順1・3)。`PaneState::waveforms()` を
    /// [`Self::with_waveforms`] で運ぶだけ(`work_area` と同じ形)。既定は空
    /// (波形を1本も描かない) — `canvas::draw` の bar 描画ループがこの
    /// `HashMap` を `row.id` で引き、`Ready` な layer だけ
    /// `waveform_view::waveform_state_segments`/`waveform_ink` を呼ぶ。
    waveforms: std::collections::HashMap<LayerId, crate::waveform_view::WaveformState>,
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
            waveforms: std::collections::HashMap::new(),
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

    /// `Shell::view` だけが呼ぶ想定(TL7 統合手順1・3)。`PaneState::waveforms()`
    /// をそのまま渡すだけの薄い builder — `with_work_area` と同じ形(既存の
    /// 呼び出し元・試験を1つも壊さない、既定は空 = 波形を描かない)。
    pub fn with_waveforms(
        mut self,
        waveforms: std::collections::HashMap<LayerId, crate::waveform_view::WaveformState>,
    ) -> Self {
        self.waveforms = waveforms;
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
    /// の全 `(layer, preview timing)` 列をそのまま渡すだけの薄い builder —
    /// `with_key_drag_active` と同じ形。**置換そのものは
    /// [`projection::apply_clip_preview`](投影段の純関数)がやる** — ここは
    /// 運ぶだけで if を持たない。`None`(非ドラッグ中)なら `rows` は無傷。
    pub fn with_clip_preview(mut self, preview: Option<Vec<(LayerId, LayerTiming)>>) -> Self {
        if preview.is_some() {
            self.preview_active = true;
        }
        self.rows = projection::apply_clip_preview(self.rows, preview.as_deref());
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

    /// 行(層行+property 行)だけの高さ、**ルーラーを含まない**
    /// (縦スクロール発注 2026-08-22 — EXACT TARGET: ヘッダー(ルーラー)と
    /// スクロールする本体(rail 行リスト+行だけの canvas)の境界そのもの)。
    /// `rail::view` の container 高さ・本体 canvas の widget 高さの唯一の
    /// 出典 — 2箇所で別の式を持たない([`Self::view`] 参照)。
    fn rows_area_height(&self) -> f32 {
        self.dims.row_height * self.rows.len() as f32
            + self.param_row_height() * self.property_rows.len() as f32
    }

    /// **縦スクロール発注(2026-08-22)**: `docs/reviews/2026-08-22-persona-lyric-mv-round2.md`
    /// が実証した欠落(レーン一覧に縦スクロールが無く、歌詞レイヤー
    /// 50〜100枚が物理的に見えない)の根治。
    ///
    /// 構造(`super::ruler` モジュール doc に詳細):
    /// - **常時固定ヘッダー** `row![rail::corner, ruler::view]`(ルーラー・
    ///   ループ帯・マーカー・playhead のルーラー内区間 — EXACT TARGET 4
    ///   「playhead とルーラーは固定」)
    /// - **スクロール本体** `scrollable(row![rail::view, canvas])`(rail の
    ///   行リストと、行だけを描く canvas を**同じ1つの `scrollable`** で
    ///   包む — rail と canvas の縦位置が構造的にずれ得ない、EXACT TARGET
    ///   「当たり判定の追随」はこの1点に集約される: iced 自身の
    ///   layout/clip/translate が両方の子へ同じオフセットを適用するので、
    ///   `hit::hit_test`/`projection::layer_row_top` 自体は無改造のまま
    ///   (呼び出し側が受け取る `bounds`/`cursor` は元から scroll 前後で
    ///   意味が変わらない — 2つの scrollable を id 経由で同期させる案は
    ///   不採用、`super::ruler` モジュール doc「経緯・構造」節に理由)
    ///
    /// **TL-arch Phase 1 からの継承**(`docs/reviews/2026-08-22-timeline-canvas-widget-survey.md`
    /// §6): rail の行リストは `&self` の借用で組み立て終える(rows/
    /// property_rows/dims/colors を読むだけ)— その後で `self` を
    /// `canvas(self)` へ move する(`Program` impl は `self` を値で持つ、
    /// 下の `impl canvas::Program` 参照)。canvas の x 原点は rail の右端の
    /// まま不変(TL-arch Phase 1)、**y 原点はルーラー下→行0の上端へ移った**
    /// (縦版の同じ手口 — `canvas.rs`/`hit.rs`/`input.rs`/`key_rows.rs` は
    /// どれも「自分の bounds がそのまま行の場」という前提へ揃えた、意味は
    /// 不変 — 発注書「座標系は関数境界で吸収」)。
    pub fn view(self) -> Element<'static, Message> {
        let ruler_height = self.ruler_height();
        let rows_height = self.rows_area_height();

        // 常時固定ヘッダー(借用のみ — `self` はまだ生きている)。
        // corner は rail 行リストと同じ固定幅 — ここを Fill にすると flex が
        // ヘッダー行を等分し、ルーラーが下のクリップ面と揃わない(rail::corner doc)。
        let header = row![
            rail::corner(self.dims, self.colors, ruler_height, self.rail_width()),
            ruler::view(&self)
        ]
        .height(Length::Fixed(ruler_height));

        // スクロール本体(rail 行リストも借用のみ、`canvas(self)` の直前)。
        let rail_rows = rail::view(&self);
        // **D-5(縦カリング)**: 標準の `iced::widget::canvas()` ではなく自前の
        // [`viewport_canvas::ViewportCanvas`] を使う — `canvas()` は
        // `Widget::draw`/`update` が受け取る `viewport: &Rectangle` を
        // `canvas::Program` へ転送しない(fork 実測、`viewport_canvas` モジュール
        // doc の「探索範囲」節)。差し替えは `canvas(self)` → `ViewportCanvas::new(self)`
        // の1行のみ、`.width()`/`.height()`/`.into()` の形は不変。
        let field = viewport_canvas::ViewportCanvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(rows_height));
        let body = iced::widget::scrollable(
            row![rail_rows, field].height(Length::Fixed(rows_height)),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        iced::widget::column![header, body].into()
    }

    /// [`Self::view`] の上に transport 帯(map 1041-1045・1138、
    /// [`transport`] モジュール doc)を積んだ版。shell の統合(下部 Play バー
    /// 撤去と同時)はこの `view_with_transport()` へ呼び出し1行を差し替える
    /// だけ(supervisor、RETURN の結線一覧)。
    ///
    /// **既知の逸脱(縦スクロール発注 RETURN 参照)**: `view()` の内部構造
    /// (`row![rail, field]` 単体 → `column![header, scrollable(...)]`)を
    /// 今回変えた — 旧 doc はここを「shell 側の生座標 click 検分
    /// (`iced_test_spike.rs`)を壊さないため無改変で残す」としていたが、
    /// 縦スクロールの根治(EXACT TARGET)自体がこの内部構造の変更を要求する
    /// ため、その制約より発注の目的を優先した。影響は
    /// `next/shell/motolii-shell/tests/suite/iced_test_spike.rs` の生座標
    /// click 試験2本(bar/ルーラー)— 座標の再計算が必要(この crate の
    /// write-set 外、RETURN で報告)。
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

    /// **D-5(縦カリング)**: `view()` はもう標準の `iced::widget::canvas()`
    /// ではなく [`viewport_canvas::ViewportCanvas`] を使う(下の
    /// `impl viewport_canvas::ViewportProgram` 参照)ので、実際の描画は
    /// `draw_viewport` 経由になり、この `Program::draw`(viewport を持たない
    /// 標準シグネチャ)はもう呼ばれない。それでも `canvas::Program` トレイト
    /// は満たす必要がある(`ViewportProgram: canvas::Program` — トレイトの
    /// 意味自体は再実装しない、モジュール doc 参照)ので、`bounds` 全体を
    /// 可視域として扱う安全側のフォールバック実装を残す(間引き無し=絵は
    /// 常に不変、呼ばれないコードパスなので性能は問わない)。
    fn draw(
        &self,
        _state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        canvas::draw(self, renderer, bounds, cursor, bounds)
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

/// **D-5(Timeline 縦カリング)**: [`viewport_canvas::ViewportProgram`] の
/// `draw_viewport` を上書きし、`viewport`(`ViewportCanvas` の `Widget::draw`
/// が受け取る実際の可視矩形)を `canvas::draw` へ渡す。既定実装
/// (`canvas::Program::draw` へ委譲するだけ)から変えたのはこの1点のみ —
/// `update`/`mouse_interaction` は上の `impl canvas::Program` のまま不変
/// (`viewport_canvas::ViewportCanvas::update`/`mouse_interaction` が
/// `canvas::Program` を経由して呼ぶ)。
impl viewport_canvas::ViewportProgram<Message> for TimelinePane {
    fn draw_viewport(
        &self,
        _state: &Interaction,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
        viewport: Rectangle,
    ) -> Vec<iced::widget::canvas::Geometry> {
        canvas::draw(self, renderer, bounds, cursor, viewport)
    }
}

/// 縦スクロール発注(2026-08-22、`docs/reviews/2026-08-22-persona-lyric-mv-round2.md`)
/// の落ちるテスト先行(裁定189: 検収線は `cargo check --tests` 緑まで —
/// テストは書くが**未実行**、実行は supervisor/後続レーン)。
///
/// **設計選択の直接の帰結としてのオラクル**: このレーンは
/// `iced::widget::scrollable` を採用し(`super::ruler` モジュール doc
/// 「経緯・構造」節)、canvas 側の自前スクロールオフセットは持たない —
/// scrollable が rail+canvas を**同じ1つの子**として丸ごとスクロールする
/// ので、当たり判定(`hit::hit_test`)・投影(`projection::layer_row_top`)
/// 自体はスクロール量を一切知らない(知る必要が無い、iced が cursor/bounds
/// を透過的に content 座標へ変換する)。よってここでの「当たり判定の追随」
/// オラクルは、**行の並びが深くなっても(100行規模でも)draw と hit が同じ
/// `layer_row_top` から同じ y を得ること**に帰着する — これは
/// `hit.rs::tests::hit_test_accounts_for_expanded_property_row_band`
/// (T3b)と同じ形の検収を、歌詞動画の実尺(50〜100レイヤー)へ延長したもの。
#[cfg(test)]
mod scroll_tests {
    use super::*;
    use motolii_store::LayerId;

    fn row(id: u64) -> RowProjection {
        RowProjection {
            id: LayerId(id),
            name: String::new(),
            hidden: false,
            solo: false,
            locked: false,
            label_color: None,
            start: 0,
            duration: 10,
            selected: false,
            dragging: false,
            depth: 0,
            has_children: false,
            children_open: true,
        }
    }

    fn pane_with_rows(count: u64) -> TimelinePane {
        let rows: Vec<RowProjection> = (0..count).map(row).collect();
        let mut dims = tokens::Dimensions::default();
        dims.row_height = 26.0;
        TimelinePane {
            rows,
            property_rows: Vec::new(),
            selected_row_index: None,
            markers: Vec::new(),
            playhead: 0,
            duration_frames: 300,
            fps: None,
            dims,
            colors: tokens::Colors::default(),
            modifiers: iced::keyboard::Modifiers::default(),
            key_drag_active: false,
            preview_active: false,
            playing: false,
            work_area: None,
            loop_enabled: false,
            rename: None,
            waveforms: std::collections::HashMap::new(),
        }
    }

    /// **オラクル(a)**: `rows_area_height` は行(層+property)だけの和 —
    /// ルーラーを1px も含まない(ヘッダーとスクロール本体の境界そのもの、
    /// [`TimelinePane::rows_area_height`] doc 参照)。この不変が崩れると
    /// ヘッダーとスクロール本体の間に隙間/重なりが生まれる。
    #[test]
    fn rows_area_height_excludes_the_ruler_and_sums_row_heights_only() {
        let pane = pane_with_rows(80);
        let expected = pane.dims.row_height * 80.0;
        assert_eq!(pane.rows_area_height(), expected);
        assert_ne!(
            pane.rows_area_height(),
            pane.ruler_height() + expected,
            "rows_area_height にルーラー分が紛れ込んでいる"
        );
    }

    /// **オラクル(b、レイヤーが少ない時にスクロールしない)**: iced の
    /// `scrollable` は content が viewport を超えた時だけスクロールバーを
    /// 出す(この crate の外側の事実 — 発明も検証もしない)。この crate 側の
    /// 責任は「content の高さがレイヤー数に単調に比例する」ことだけ —
    /// 数枚なら小さく、歌詞動画の実尺(50〜100枚)なら大きくなることを保証
    /// すれば、少数時に scrollable が無反応(=スクロールしない)になる。
    #[test]
    fn a_handful_of_layers_stay_far_smaller_than_the_full_lyric_mv_scale() {
        let few = pane_with_rows(3).rows_area_height();
        let many = pane_with_rows(80).rows_area_height();
        assert!(few < many, "行数に応じて高さが単調に増えていない");
        // 3行なら通常のウィンドウ内の1 pane に無理なく収まる代表値(実測では
        // なく「桁が違う」ことを示す下限 — 780pxの窓に3行10pane分は余裕で入る)。
        assert!(few < 200.0, "少数レイヤーの高さが想定より大きすぎる: {few}");
    }

    /// **オラクル(c、当たり判定の追随・端 = 先頭)**: 行0の bar は
    /// `layer_row_top(0)`(=0、ルーラー移設後の body canvas 規約)から
    /// `row_height` の範囲で当たる。`hit_test` の `ruler_height` 引数は
    /// 本文(body canvas)の呼び出し規約どおり常に `0.0`
    /// (`input.rs::update`/`mouse_interaction` の呼び出しと同じ値 —
    /// 2箇所で別の値を渡さない)。
    #[test]
    fn hit_test_lands_on_the_first_row_at_the_top_of_a_long_list() {
        let pane = pane_with_rows(80);
        let width = 300.0;
        let y = layer_row_top(pane.dims.row_height, pane.param_row_height(), 0, None, 0)
            + pane.dims.row_height / 2.0;
        let hit = hit_test(
            iced::Point::new(5.0, y),
            &pane.rows,
            0.0,
            pane.dims.row_height,
            width,
            pane.duration_frames,
            pane.param_row_height(),
            0,
            None,
        );
        assert_eq!(hit, Hit::Bar(LayerId(0)), "80行の先頭(行0)への当たりがずれている");
    }

    /// **オラクル(d、当たり判定の追随・端 = 末尾)**: 歌詞動画の実尺
    /// (80行、50〜100枚のレンジ内)の**最後の行**でも、`layer_row_top`と
    /// `hit_test` が同じ y から同じ layer を指す — スクロールで深く沈んだ
    /// 行ほど、絵と当たりがずれる古典的な事故(T3b と同型)が起きやすい
    /// ため、意図してリストの深部をオラクルにする。
    #[test]
    fn hit_test_lands_on_the_last_row_deep_in_an_eighty_row_list() {
        let pane = pane_with_rows(80);
        let width = 300.0;
        let last = pane.rows.len() - 1;
        let y = layer_row_top(pane.dims.row_height, pane.param_row_height(), 0, None, last)
            + pane.dims.row_height / 2.0;
        let hit = hit_test(
            iced::Point::new(5.0, y),
            &pane.rows,
            0.0,
            pane.dims.row_height,
            width,
            pane.duration_frames,
            pane.param_row_height(),
            0,
            None,
        );
        assert_eq!(
            hit,
            Hit::Bar(LayerId(last as u64)),
            "80行の末尾(行79)への当たりがずれている — 深い行ほど事故りやすい(T3bと同型)"
        );
        // 隣の行(1つ手前)には当たらない — 押し下げ量が1行分ずれていないこと。
        let neighbor = hit_test(
            iced::Point::new(5.0, y - pane.dims.row_height),
            &pane.rows,
            0.0,
            pane.dims.row_height,
            width,
            pane.duration_frames,
            pane.param_row_height(),
            0,
            None,
        );
        assert_eq!(neighbor, Hit::Bar(LayerId(last as u64 - 1)), "1つ手前の行にずれて当たっている");
    }

    /// **オラクル(e)**: 0行(空 Document)・1行・80行のどのレイヤー数でも
    /// `view()`/`view_with_transport()` が panic せず widget 木を組み終える
    /// (`waveform_view_fence.rs`/`transport_fence.rs` の既存スモークと同じ
    /// 型 — このレーンの直接の回帰ガードは「歌詞動画の実尺(80行)でも
    /// view() が破綻しない」こと自体)。
    #[test]
    fn view_builds_at_zero_one_and_lyric_mv_scale_row_counts() {
        for count in [0, 1, 80] {
            let pane = pane_with_rows(count);
            let _view = pane.view();
        }
        for count in [0, 1, 80] {
            let pane = pane_with_rows(count).with_playing(true);
            let _band_and_body = pane.view_with_transport();
        }
    }
}
