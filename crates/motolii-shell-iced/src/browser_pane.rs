//! Browser pane の絵(browser-revise レーン、2026-08-19)。`view.rs` から切り出した
//! (`inspector_pane.rs` と対称)。**モデルを書く道はここに無い**(`&Shell` / `&BrowserPane`
//! しか受け取らない — `view.rs` の module doc と同じ規律)。
//!
//! ## 見た目の正本は HTML/CSS そのもの(egui 実装は手本にしない)
//!
//! 利用者の訂正(2026-08-19): このパネルの egui 実装
//! (`motolii_ui::browser_panel`)は「html から egui への変換がうまくいかなかった
//! 部分」であり、**構造・階層・視覚の手本にしない**。見た目の正本は
//! `docs/mocks-ui/public/browser-library.html` + `browser-library.css` そのもの
//! (Inspector とは非対称 — Timeline は egui 版が正本のまま)。
//!
//! html を構造として読んだ結果(意味の階層):
//!
//! ```text
//! main.libraryBrowser
//! ├─ header.browserHeader        見出し + "LOCAL LIBRARY"
//! ├─ div.browserToolbar          history(‹›・常時disabled) / 検索欄 / Filters toggle
//! ├─ nav.libraryTabs             Media/Effects/Create/Panels ★機能が Media にしか無い
//! ├─ div.browserWorkspace(flex)
//! │  ├─ aside.librarySidebar     COLLECTIONS / LIBRARY / PLACES ★機能済み分だけ
//! │  └─ section.catalog
//! │     ├─ header.catalogHeader  タイトル+path / view mode 3態
//! │     ├─ div.filterShelf       FILTERS ラベル + chip 群 + Clear(開閉可能)
//! │     ├─ div.resultSummary     "Results" + 件数
//! │     ├─ div.thumbnailGrid     card 群(サムネ枠 + 主題行 + 補助行)
//! │     ├─ footer.selectionTray  選択の点 + 名前 + meta + "Edit tags" ★機能無し
//! │     ├─ aside.tagEditor       ★機能無し(tag は derive-only、追加APIが無い)
//! │     └─ div.contextMenu × 2   ★ほぼ全項目が html 自身で disabled — 置かない
//! ```
//!
//! card 1枚の内部構造(`.libraryCard`)は既存 [`crate::browser::BrowserCard`] と
//! 1対1: サムネ枠(`.libraryThumb`、16:9、左上に kind glyph)+ `.cardCopy`
//! (主題行 = name、補助行 = meta)。選択は `.selected`(bg + サムネ枠の accent
//! 枠 + 内側 inset shadow)、hover はサムネ枠の枠だけが動く、in-project は
//! meta 文字列が言う(既存の意味、`browser.rs` 側)。
//!
//! ## Q0: html にあるが機能が無い物は置かない
//!
//! - **history ‹›**: html 自身が無条件 `disabled`(css:61 opacity .4)。
//!   ナビゲーション履歴という feature が product に無い(Undo/Redo のように
//!   「空だから disabled」ではなく、機能そのものが存在しない)ので**置かない**
//!   (`inspector_pane.rs` が Audio section を丸ごと置かなかったのと同じ判断)
//! - **Tags toggle / tag editor / "Edit tags"**: `media_library` の tag は
//!   kind・拡張子・folder 名からの derive-only で、追加/削除する API が無い。
//!   html の tag editor はその無い API を前提にしている**置かない**
//! - **Media/Effects/Create/Panels タブ**: egui 版でも Effects/Create/Panels は
//!   `visible_cards()` が常に空(項目 model が無い)。1枚しか働かない tab
//!   strip は tab ではないので、strip 自体を**置かない**(Media 単体の内容だけ描く)
//! - **COLLECTIONS(Favorite/B-roll/Brand)**: 人が能動的に印を付ける「収集」概念が
//!   product に無い(tag は derive-only)。`docs/ui-interaction-language.md:77` も
//!   「Collections」を将来枠として名指ししているだけで実装済みとは言っていない。**置かない**
//! - **PLACES(複数登録 folder)**: `MediaLibrary` は root を複数持てる構造だが
//!   `project()` は `roots.first()` しか見ない(実質1本)。html の
//!   Starter Media/Project assets/Motion assets のような複数 location 切替は
//!   機能が無い。**置かない**(単一 root は既存の `rail` = All media が担う)
//! - **右クリック context menu**: html 自身、ほぼ全項目が `disabled`
//!   (Preview / Copy source path / Reveal in folder / Place in composition)。
//!   残る Favorite 登録・tag 編集も上記の理由で機能が無い。**置かない**
//!
//! ## 移植した機能(egui の意味関数を借用 — 見た目は html 由来)
//!
//! - **検索欄**(html:25): egui `BrowserPanel::set_query` と同じ正規化
//!   (trim + 小文字化)を [`crate::browser::BrowserPane::set_query`] へ移植。
//!   一致判定は name + meta の部分一致(meta に kind/拡張子/登録状態が
//!   既に入っているので、html の「tag も見る」検索と同じ実利になる)
//! - **filter chip**(html:103, Video/Image/Audio + Clear): egui
//!   `BrowserPanel::set_kind_filter` と同じ完全一致を
//!   [`crate::browser::BrowserPane::set_kind_filter`] へ移植。html は
//!   B-roll/Brand 等の派生 tag chip も動的に出すが、この product の実データ
//!   (starter media は平坦な1 folder)には kind 以外の意味のある tag が無いので、
//!   指示どおり Video/Image/Audio の3つ + Clear だけを置く
//! - **表示モード**(html:94-97, Thumbnails/Grid/List): egui
//!   `BrowserPanel::set_view` と同じ3態を [`crate::browser::BrowserViewMode`] へ
//!   移植。列数は html と同じ(Thumbnails=4 / Grid=2 / List=1、
//!   css:218,269,273)
//! - **filter shelf の開閉**(html:26 `#filter-toggle`): egui
//!   `BrowserPanel::shelf_open` と同じ意味。`docs/ui-interaction-language.md`:122
//!   「狭い常設panelではSearchからResultsへの視線を遮るkind tag/filter行を
//!   置かず…高度な絞り込みが必要な時だけ別のFilter Viewへ開き」の要求を、
//!   常設パネルのまま「開閉できる」形で満たす(既定は開 = html の初期状態)
//!
//! ## 寸法・色は `docs/mocks-ui/public/browser-library.css` の実測
//!
//! [`dims`] / [`colors`] の各定数は `motolii-css-metrics browser` の抽出結果
//! (2026-08-19、viewport 900x600 — `crates/motolii-ui/src/css_metrics/mod.rs`
//! の既定)で検証した。sidebar/filterShelf の中身は html が `data-tab`/
//! `data-view` を JS で設定するため素の html を抽出すると 0×0 になる
//! (`docs/reviews/2026-08-19-css-computed-metrics-extraction.md` 既知の罠) —
//! この2枚は、その文書が「次に自動化できそうなこと」として書き残していた
//! 手を実行して測った: html の `<script>` 冒頭の決定的な初期代入
//! (`state = {tab: 'media', ..., view: 'grid', ...}`)をそのまま
//! `data-tab="media" data-view="grid"` として `<main>` へ静的に差し込んだ
//! 一時 copy を `tests/css_metrics_oracle.rs` の
//! `extract_browser_with_initial_media_tab()` が作り、それを抽出している
//! (`docs/mocks-ui` 自体は書き換えない)。oracle: `tests/css_metrics_oracle.rs`
//! — このモジュールは `dims`/`colors` を `pub` にして、oracle が literal を
//! 転記するのではなく実物を `use` して比較できるようにした(Inspector 側は
//! private module で片側確認にしかできないと前レーンが書き残しており、
//! ここではその提案を実行した)。
//!
//! ## iced の flex 相当は widget 構成で素直に出る
//!
//! egui 版の `catalog_header_layout`(見出しと view-mode ボタンが重ならない
//! ための手計算)は、iced では `row![title.width(Fill), buttons]` が
//! css の `display:flex` + `margin-left:auto` と同じ結果を widget 構成だけで
//! 出すので**移植していない**(egui の raw painter が持たない retained-mode
//! layout の恩恵)。

use iced::widget::{button, column, container, mouse_area, row, scrollable, space, text, text_input};
use iced::{Center, Color, Element, Fill};

use crate::browser::{BrowserCard, BrowserPane, BrowserRail, BrowserViewMode};
use crate::message::Message;
use crate::shell::Shell;
use crate::widgets::DropEvent;

// ---------------------------------------------------------------------------
// 名乗り。運転席は全部この文字列で掴む(完全一致)。
// ---------------------------------------------------------------------------

/// pane の見出し(browser-library.html:18)。
pub const HEADER: &str = "Browser";
/// source rail: 登録 folder + Document asset の統合面。
pub const RAIL_ALL: &str = "All media";
/// source rail: Document の asset 台帳。
pub const RAIL_PROJECT: &str = "Project";
/// source rail: 同じ台帳を取り込みの新しい順で。
pub const RAIL_RECENT: &str = "Recent";
/// sidebar の見出し(html:45 `LIBRARY`)。3 rail はこの下に並ぶ。
pub const RAIL_HEADING: &str = "LIBRARY";
/// 検索欄の placeholder(html:25、字一句そのまま — 検索が実装された今は Q0 の
/// 死に chrome ではなく機能する chrome)。
pub const SEARCH_PLACEHOLDER: &str = "Search files and tags";
/// filter shelf の開閉ボタン(html:26)。
pub const FILTERS_TOGGLE: &str = "Filters";
/// filter shelf の見出し(html:103 `FILTERS`)。
pub const FILTERS_LABEL: &str = "FILTERS";
/// kind chip(html:103)。
pub const CHIP_VIDEO: &str = "Video";
pub const CHIP_IMAGE: &str = "Image";
pub const CHIP_AUDIO: &str = "Audio";
/// filter shelf の Clear(html:103)。
pub const CLEAR_FILTER: &str = "Clear";
/// 結果件数の見出し(html:117 `Results`)。
pub const RESULTS_LABEL: &str = "Results";
/// view mode ボタン(html:95-97)。css:167 `width:21px` の icon-only ボタンで
/// 英語 label を横に置く余地が無いため、html と同じく glyph だけを表示する
/// (aria-label 相当の意味は doc コメントに書く。運転席はこの glyph 文字列で
/// 完全一致で掴む)。
/// html:95 aria-label="Thumbnail view"。
pub const VIEW_THUMBNAILS: &str = "\u{25a6}"; // ▦
/// html:96 aria-label="Grid view"(既定)。
pub const VIEW_GRID: &str = "\u{25a4}"; // ▤
/// html:97 aria-label="List view"。
pub const VIEW_LIST: &str = "\u{2637}"; // ☷
/// 掴んだファイルが窓の上に居るあいだ、panel が受け皿であることを言う一行。
pub const DROP_TARGET: &str = "Drop anywhere in this window to add media";
/// Project rail の空状態(Q7: 空は空として、次の一手つきで言う)。
pub const EMPTY_PROJECT: &str =
    "No media in this project yet — drop files into this window, or double-click a card in All media.";
/// Recent rail の空状態。
pub const EMPTY_RECENT: &str = "Nothing has been added to this project yet.";
/// 検索/filter が結果を0件にした時の一言(Q7: 幽霊 card ではなく正直な空)。
pub const EMPTY_FILTERED: &str = "No results match this search or filter.";
/// selection tray の空状態。
pub const NO_SELECTION: &str = "No selection";

/// All media の空状態は登録 folder を名指しする(幽霊 card を出さない)。
pub fn empty_all(folder: &str) -> String {
    format!("No media found in {folder}.")
}

/// selection tray の名乗り。card の text と選択の text を運転席が
/// 別々に掴めるよう、tray は `●` を前置する。
pub fn tray_label(name: &str) -> String {
    format!("\u{25cf} {name}")
}

// ---------------------------------------------------------------------------
// 寸法 — browser-library.css の写し。`motolii-css-metrics browser` の抽出結果
// (2026-08-19、viewport 900x600)で検証済み。iced 側で新しい値を1つも決めない。
// ---------------------------------------------------------------------------

pub mod dims {
    /// browser-library.css:29 `.browserHeader { height: 26px }` — css_metrics 実測 26.0。
    pub const HEADER_H: f32 = 26.0;
    /// browser-library.css:33 `padding: 0 8px`。
    pub const HEADER_PAD_X: f32 = 8.0;
    /// browser-library.css:41 `.browserToolbar { height: 30px }` — css_metrics 実測 30.0。
    pub const TOOLBAR_H: f32 = 30.0;
    /// browser-library.css:46 `padding: 3px 5px`。
    pub const TOOLBAR_PAD_X: f32 = 5.0;
    pub const TOOLBAR_PAD_Y: f32 = 3.0;
    /// browser-library.css:45 `gap: 3px`(history ボタン間・検索欄との間)。
    pub const TOOLBAR_GAP: f32 = 3.0;
    /// browser-library.css:53 `.historyButton,.toolbarButton,.viewModes button { height: 21px }`
    /// — css_metrics 実測 21.0(共通の control 高さ)。
    pub const CONTROL_H: f32 = 21.0;
    /// browser-library.css:68 `#library-search { padding: 0 6px }`。
    pub const SEARCH_PAD_X: f32 = 6.0;
    /// browser-library.css:62 `.toolbarButton { padding: 0 5px }`(Filters toggle)。
    pub const TOOLBAR_BUTTON_PAD_X: f32 = 5.0;

    /// browser-library.css:102 `.librarySidebar { width: 112px }` — css_metrics 実測 112.0
    /// (viewport 900 では break point の外)。
    pub const SIDEBAR_W: f32 = 112.0;
    /// browser-library.css:349 `@media (max-width: 420px) { .librarySidebar { width: 92px } }`。
    pub const SIDEBAR_W_NARROW: f32 = 92.0;
    /// browser-library.css:348 の境目。
    pub const NARROW_BREAKPOINT_W: f32 = 420.0;
    /// browser-library.css:105 `padding: 3px 0 6px`。
    pub const SIDEBAR_PAD_TOP: f32 = 3.0;
    pub const SIDEBAR_PAD_BOTTOM: f32 = 6.0;
    /// browser-library.css:111 `.librarySidebar h2 { height: 16px }`。
    pub const SIDEBAR_H2_H: f32 = 16.0;
    /// browser-library.css:128 `.locationRow { height: 19px }` — css_metrics 実測
    /// 19.0(`data-tab="media"` を静的に補った一時 copy 経由。素の html は
    /// `data-tab` が JS 設定のため 0×0 になる既知の罠 —
    /// `tests/css_metrics_oracle.rs::extract_browser_with_initial_media_tab`)。
    pub const ROW_H: f32 = 19.0;
    /// browser-library.css:130 `padding: 3px 7px 0`。
    pub const ROW_PAD_X: f32 = 7.0;
    /// browser-library.css:143 `.locationRow.indent { padding-left: 13px }`。
    pub const ROW_INDENT: f32 = 13.0;
    /// browser-library.css:132/144 `border-left: 2px solid …`(選択の左帯)。
    pub const ROW_ACCENT_W: f32 = 2.0;

    /// browser-library.css:156 `.catalogHeader { height: 31px }` — css_metrics 実測 31.0。
    pub const CATALOG_HEADER_H: f32 = 31.0;
    /// browser-library.css:160 `padding: 0 6px`。
    pub const CATALOG_HEADER_PAD_X: f32 = 6.0;
    /// browser-library.css:167 `.viewModes button { width: 21px }`。
    pub const VIEW_BUTTON_W: f32 = 21.0;
    /// browser-library.css:166 `.viewModes { gap: 2px }`。
    pub const VIEW_BUTTON_GAP: f32 = 2.0;

    /// browser-library.css:171 `.filterShelf { min-height: 24px }` — css_metrics
    /// 実測 24.0(同じ一時 copy 経由。素の html は `.filterGroup{display:none}`
    /// が `data-tab` 依存で外れず 0×0)。
    pub const SHELF_MIN_H: f32 = 24.0;
    /// browser-library.css:176 `padding: 3px 5px`。
    pub const SHELF_PAD_X: f32 = 5.0;
    pub const SHELF_PAD_Y: f32 = 3.0;
    /// browser-library.css:174 `gap: 3px`。
    pub const SHELF_GAP: f32 = 3.0;
    /// browser-library.css:191-192 `.filterShelf button { min-height: 17px; padding: 2px 5px }`。
    pub const CHIP_MIN_H: f32 = 17.0;
    pub const CHIP_PAD_X: f32 = 5.0;
    pub const CHIP_PAD_Y: f32 = 2.0;
    /// browser-library.css:193 `border: 1px solid …`。
    pub const CHIP_BORDER_W: f32 = 1.0;
    /// browser-library.css:194 `border-radius: 8px`。
    pub const CHIP_RADIUS: f32 = 8.0;

    /// browser-library.css:205 `.resultSummary { height: 21px }` — css_metrics 実測 21.0。
    pub const SUMMARY_H: f32 = 21.0;
    /// browser-library.css:209 `padding: 0 6px`。
    pub const SUMMARY_PAD_X: f32 = 6.0;

    /// browser-library.css:227 `.libraryCard { padding: 3px }` — css_metrics 実測
    /// (`out/browser.json` の `button.libraryCard` 行、四辺とも 3.0)。
    pub const CARD_PAD: f32 = 3.0;
    /// browser-library.css:270-271 `[data-view="thumbnails"] { padding: 2px }`。
    pub const CARD_PAD_THUMBNAILS: f32 = 2.0;
    /// browser-library.css:245 `.libraryThumb { padding: 3px }`(glyph の inset)。
    pub const THUMB_PAD: f32 = 3.0;
    /// browser-library.css:246 thumb 枠 `border: 1px solid …`。
    pub const THUMB_BORDER_W: f32 = 1.0;
    /// browser-library.css:241 `aspect-ratio: 16 / 9`。**比率だけ**が css の
    /// literal — 解決後の絶対 px は width が動的な block box なので
    /// css_metrics の絶対座標を転記しない(`docs/reviews/
    /// 2026-08-19-css-computed-metrics-extraction.md`「対象は個別レイアウトの
    /// 絶対座標ではない」と同じ理由。実測でも `<span>` + `display:flex` +
    /// `aspect-ratio` のみで明示 width が無い箱は、Blitz の flex 解決が
    /// 内容量ベースの極小値を返す既知のずれがあった — grid 列幅から
    /// [`thumb_height`] で導く)。
    pub const THUMB_ASPECT: f32 = 16.0 / 9.0;
    /// browser-library.css:222 `.thumbnailGrid { padding: 0 1px 3px }`。
    pub const GRID_PAD_X: f32 = 1.0;
    pub const GRID_PAD_BOTTOM: f32 = 3.0;
    /// browser-library.css:275 `[data-view="list"] .libraryThumb { width: 46px }`。
    pub const LIST_THUMB_W: f32 = 46.0;
    /// browser-library.css:276 `[data-view="list"] .cardCopy { padding-left: 5px }`。
    pub const LIST_COPY_GAP: f32 = 5.0;
    /// browser-library.css:266 `.cardCopy strong { margin-top: 2px }`
    /// (thumb とキャプションの間)。
    pub const CARD_COPY_GAP: f32 = 2.0;

    /// browser-library.css:279 `.selectionTray { height: 27px }` — css_metrics 実測 27.0。
    pub const TRAY_H: f32 = 27.0;
    /// browser-library.css:285 `padding: 0 6px`。
    pub const TRAY_PAD_X: f32 = 6.0;
    /// browser-library.css:283 `gap: 4px`。
    pub const TRAY_GAP: f32 = 4.0;
    /// browser-library.css:291 `.selectionDot { width: 5px; height: 5px }`。
    pub const TRAY_DOT: f32 = 5.0;

    /// 罫線の太さ。この mock は角丸を使わず、境界は一貫して 1px
    /// (header/toolbar/catalogHeader/filterShelf/tray の border-* が全部
    /// `1px solid`)。
    pub const HAIRLINE: f32 = 1.0;

    // ---- 列数(html:94-97 view mode / css:218,269,273) ----
    pub const COLUMNS_THUMBNAILS: usize = 4;
    pub const COLUMNS_GRID: usize = 2;
    pub const COLUMNS_LIST: usize = 1;

    /// browser-library.css:102-103 / css:349 `.librarySidebar` の幅。
    /// pane 幅が break point 以下なら狭い方(css:348 の narrow media query)。
    pub fn sidebar_width(pane_w: f32) -> f32 {
        if pane_w <= NARROW_BREAKPOINT_W {
            SIDEBAR_W_NARROW
        } else {
            SIDEBAR_W
        }
    }

    /// view mode → 列数(html:94-97 の3態)。
    pub fn columns(view: super::BrowserViewMode) -> usize {
        match view {
            super::BrowserViewMode::Thumbnails => COLUMNS_THUMBNAILS,
            super::BrowserViewMode::Grid => COLUMNS_GRID,
            super::BrowserViewMode::List => COLUMNS_LIST,
        }
    }

    /// grid/thumbnails の thumb 高さ。css の `aspect-ratio: 16/9` そのもの
    /// ([`THUMB_ASPECT`])を、既知の pane 幅から解いた列幅に当てる — iced は
    /// レイアウト前に子の高さを明示しないと `Fill` が0になる(egui の
    /// immediate-mode と違い、`ui.available_width()` を実行時に読めない)ので、
    /// この pane は幅を固定(`PANE_W`)にしている前提で計算できる。
    pub fn thumb_height(pane_w: f32, columns: usize) -> f32 {
        let catalog_w = (pane_w - sidebar_width(pane_w) - 2.0 * GRID_PAD_X).max(1.0);
        let cell_w = catalog_w / columns.max(1) as f32;
        let content_w = (cell_w - 2.0 * CARD_PAD).max(1.0);
        content_w / THUMB_ASPECT
    }
}

// ---------------------------------------------------------------------------
// 色 — browser-library.css の写し。単位は CSS 16進数そのまま。
// `theme::Tokens` の21 semantic role には対応が無い raw hex なので(egui 側
// `browser_panel/theme.rs` と同じ事情)、ここは独自の意味 role として持つ
// (勝手な新規 hex ではなく、全部 css:行 の写し)。
// ---------------------------------------------------------------------------

pub mod colors {
    use iced::Color;

    const fn rgb(v: u32) -> Color {
        Color::from_rgb(
            ((v >> 16) & 0xff) as f32 / 255.0,
            ((v >> 8) & 0xff) as f32 / 255.0,
            (v & 0xff) as f32 / 255.0,
        )
    }

    /// browser-library.css:25 `.libraryBrowser { background: #202225 }`。
    pub const PANEL_BG: Color = rgb(0x202225);
    /// browser-library.css:34 `.browserHeader { border-bottom: 1px solid #3a3c40 }`。
    pub const HEADER_BORDER: Color = rgb(0x3a3c40);
    /// browser-library.css:38 `.browserHeader span { color: #7d8284 }`。
    pub const HEADER_SPAN_FG: Color = rgb(0x7d8284);
    /// browser-library.css:47 `.browserToolbar { border-bottom: 1px solid #383b3e }`。
    pub const TOOLBAR_BORDER: Color = rgb(0x383b3e);
    /// browser-library.css:54-56 共通 control: border #45494d / bg #1a1c1f / fg #bec1bf。
    pub const CONTROL_BORDER: Color = rgb(0x45494d);
    pub const CONTROL_BG: Color = rgb(0x1a1c1f);
    pub const CONTROL_FG: Color = rgb(0xbec1bf);
    /// browser-library.css:69/71/72 `#library-search`: border #474b50 / bg #15171a / fg #eeefeb。
    pub const SEARCH_BORDER: Color = rgb(0x474b50);
    pub const SEARCH_BG: Color = rgb(0x15171a);
    pub const SEARCH_FG: Color = rgb(0xeeefeb);
    /// browser-library.css:76 `#library-search::placeholder { color: #74797f }`。
    pub const SEARCH_PLACEHOLDER_FG: Color = rgb(0x74797f);
    /// browser-library.css:77/144/168/201/250/291 — この mock 全体の accent gold。
    pub const ACCENT: Color = rgb(0xb9a660);

    /// browser-library.css:107 `.librarySidebar { background: #181a1d }`。
    pub const SIDEBAR_BG: Color = rgb(0x181a1d);
    /// browser-library.css:106 `.librarySidebar { border-right: 1px solid #3a3c40 }`。
    pub const SIDEBAR_BORDER: Color = rgb(0x3a3c40);
    /// browser-library.css:114 `.librarySidebar h2 { color: #727679 }`。
    pub const SIDEBAR_H2_FG: Color = rgb(0x727679);
    /// browser-library.css:134 `.locationRow { color: #b9bcba }`。
    pub const ROW_FG: Color = rgb(0xb9bcba);
    /// browser-library.css:144 selected: bg #353738 / fg #f1f1ed(border-left は ACCENT)。
    pub const ROW_SELECTED_BG: Color = rgb(0x353738);
    pub const ROW_SELECTED_FG: Color = rgb(0xf1f1ed);
    /// browser-library.css:145 hover: bg #292c2f。
    pub const ROW_HOVER_BG: Color = rgb(0x292c2f);

    /// browser-library.css:161 `.catalogHeader { border-bottom: 1px solid #373a3d }`。
    pub const CATALOG_HEADER_BORDER: Color = rgb(0x373a3d);
    /// browser-library.css:164 `.catalogHeader strong { color: #e2e3df }`。
    pub const CATALOG_TITLE_FG: Color = rgb(0xe2e3df);
    /// browser-library.css:165 `.catalogHeader span { color: #808487 }`。
    pub const CATALOG_PATH_FG: Color = rgb(0x808487);
    /// browser-library.css:168 pressed view button: bg #38362c / fg #f0ebce
    /// (border は ACCENT)。
    pub const VIEW_PRESSED_BG: Color = rgb(0x38362c);
    pub const VIEW_PRESSED_FG: Color = rgb(0xf0ebce);

    /// browser-library.css:178 `.filterShelf { background: #1b1d20 }`。
    pub const SHELF_BG: Color = rgb(0x1b1d20);
    /// browser-library.css:177 `.filterShelf { border-bottom: 1px solid #34373a }`。
    pub const SHELF_BORDER: Color = rgb(0x34373a);
    /// browser-library.css:187 `.filterShelf span { color: #777c7e }`(FILTERS label)。
    pub const SHELF_LABEL_FG: Color = rgb(0x777c7e);
    /// browser-library.css:193/195/196 chip: border #484b4e / bg #26292c / fg #c9ccca。
    pub const CHIP_BORDER: Color = rgb(0x484b4e);
    pub const CHIP_BG: Color = rgb(0x26292c);
    pub const CHIP_FG: Color = rgb(0xc9ccca);
    /// browser-library.css:201 selected chip: bg #3c382b / fg #f1e8bd(border は ACCENT)。
    pub const CHIP_SELECTED_BG: Color = rgb(0x3c382b);
    pub const CHIP_SELECTED_FG: Color = rgb(0xf1e8bd);
    /// browser-library.css:202 `.clearFilter { color: #9b9e9c }`。
    pub const CLEAR_FG: Color = rgb(0x9b9e9c);

    /// browser-library.css:213 `.resultSummary span { color: #898d8e }`(件数)。
    pub const SUMMARY_COUNT_FG: Color = rgb(0x898d8e);

    /// browser-library.css:237 `.libraryCard.selected { background: #36393a }`。
    pub const CARD_SELECTED_BG: Color = rgb(0x36393a);
    /// browser-library.css:246 `.libraryThumb { border: 1px solid #3e4245 }`。
    pub const THUMB_BORDER: Color = rgb(0x3e4245);
    /// browser-library.css:249 hover: `border-color: #85898b`。
    pub const THUMB_HOVER_BORDER: Color = rgb(0x85898b);
    /// browser-library.css:251 `.libraryThumb b { color: #f2f2ed }`(kind glyph)。
    pub const THUMB_GLYPH_FG: Color = rgb(0xf2f2ed);
    /// browser-library.css:252-255(mock の実例に合わせた kind → 色の割当)。
    pub const THUMB_VIDEO: Color = rgb(0x5d7899); // .thumb-blue(starter-clip.mp4)
    pub const THUMB_SVG: Color = rgb(0x746398); // .thumb-purple(starter-mark.svg)
    pub const THUMB_IMAGE: Color = rgb(0x88704e); // .thumb-ochre(starter-still.png)
    pub const THUMB_AUDIO: Color = rgb(0x557f6d); // .thumb-green(starter-tone.wav)
    /// browser-library.css:266 `.cardCopy strong { color: #edede9 }`。
    pub const CARD_NAME_FG: Color = rgb(0xedede9);
    /// browser-library.css:267 `.cardCopy small { color: #9ea2a3 }`。
    pub const CARD_META_FG: Color = rgb(0x9ea2a3);

    /// browser-library.css:287 `.selectionTray { background: #191b1e }`。
    pub const TRAY_BG: Color = rgb(0x191b1e);
    /// browser-library.css:286 `.selectionTray { border-top: 1px solid #3a3d40 }`。
    pub const TRAY_BORDER: Color = rgb(0x3a3d40);
    /// browser-library.css:292 `.selectionTray strong { color: #e8e9e4 }`。
    pub const TRAY_NAME_FG: Color = rgb(0xe8e9e4);
    /// browser-library.css:293 `.selectionTray span { color: #83888a }`。
    pub const TRAY_META_FG: Color = rgb(0x83888a);

    /// 受け皿点灯(html に対応 class は無い — iced 固有の状態表示。ACCENT を
    /// 薄めて使う、`view.rs` 旧実装と同じ判断)。
    pub fn drop_hover_bg(accent: Color) -> Color {
        Color { a: 0.12, ..accent }
    }
}

// ---------------------------------------------------------------------------
// フォントサイズ — browser-library.css の font-size をそのまま(px = point)。
// ---------------------------------------------------------------------------

mod font_size {
    /// browser-library.css:37 `.browserHeader strong { font-size: 10px }`。
    pub const HEADER_TITLE: f32 = 10.0;
    /// browser-library.css:38 `.browserHeader span { font-size: 6px }`。
    pub const HEADER_SPAN: f32 = 6.0;
    /// browser-library.css:62 `.toolbarButton { font-size: 7px }`。
    pub const TOOLBAR_BUTTON: f32 = 7.0;
    /// browser-library.css:73 `#library-search { font-size: 8px }`。
    pub const SEARCH: f32 = 8.0;
    /// browser-library.css:115 `.librarySidebar h2 { font-size: 6px }`。
    pub const SIDEBAR_H2: f32 = 6.0;
    /// browser-library.css:136 `.locationRow { font-size: 8px }`。
    pub const ROW: f32 = 8.0;
    /// browser-library.css:164 `.catalogHeader strong { font-size: 9px }`。
    pub const CATALOG_TITLE: f32 = 9.0;
    /// browser-library.css:165 `.catalogHeader span { font-size: 6px }`。
    pub const CATALOG_PATH: f32 = 6.0;
    /// browser-library.css:167 `.viewModes button { font-size: 8px }`。
    pub const VIEW_BUTTON: f32 = 8.0;
    /// browser-library.css:187 `.filterShelf span { font-size: 6px }`。
    pub const SHELF_LABEL: f32 = 6.0;
    /// browser-library.css:198 `.filterShelf button { font-size: 7px }`。
    pub const CHIP: f32 = 7.0;
    /// browser-library.css:212 `.resultSummary strong { font-size: 8px }`。
    pub const SUMMARY_TITLE: f32 = 8.0;
    /// browser-library.css:213 `.resultSummary span { font-size: 7px }`。
    pub const SUMMARY_COUNT: f32 = 7.0;
    /// browser-library.css:251 `.libraryThumb b { font-size: 8px }`。
    pub const THUMB_GLYPH: f32 = 8.0;
    /// browser-library.css:266 `.cardCopy strong { font-size: 8px }`。
    pub const CARD_NAME: f32 = 8.0;
    /// browser-library.css:267 `.cardCopy small { font-size: 6.5px }`。
    pub const CARD_META: f32 = 6.5;
    /// browser-library.css:292 `.selectionTray strong { font-size: 7px }`。
    pub const TRAY_NAME: f32 = 7.0;
    /// browser-library.css:293 `.selectionTray span { font-size: 6px }`。
    pub const TRAY_META: f32 = 6.0;
}

// ---------------------------------------------------------------------------
// 描画
// ---------------------------------------------------------------------------

/// Browser pane の幅(既定配置の左)。panel の resize / dock 文法は
/// M-5 以降の統合 wave で来る(`view.rs` の既存注記と同じ理由で、mock の
/// `min(100%,700px)` とは意図的に不一致 — dock の既定 share から採る)。
pub const PANE_W: f32 = 316.0;

/// Browser pane 1面。上から見出し・toolbar(検索+Filters)・sidebar(rail)+catalog
/// (header/filter shelf/summary/grid)・selection tray。**Document を書く道は
/// ここに無い** — カードのダブルクリックは Message になり、殻が
/// `UiIntent::AdmitPaths` へ写す。
pub fn browser_pane(shell: &Shell) -> Element<'_, Message> {
    let pane = shell.browser();
    let cards = shell.browser_cards();

    // html:170 `<strong id="selection-name">` + `<span id="selection-meta">`
    // の2行。card が既に持つ meta(`kind · secondary[· in project]`)を
    // そのまま補助行にする — 二重帳簿を作らない(card.meta が正本)。
    let selected_card = pane
        .selected_id()
        .and_then(|selected| cards.iter().find(|card| card.id == selected));
    let tray_line = selected_card
        .map(|card| tray_label(&card.name))
        .unwrap_or_else(|| NO_SELECTION.to_owned());
    let tray_meta = selected_card.map(|card| card.meta.clone());

    let mut panel = column![header(pane), hairline_h(colors::HEADER_BORDER)]
        .width(PANE_W)
        .height(Fill);
    panel = panel.push(toolbar(pane));
    panel = panel.push(hairline_h(colors::TOOLBAR_BORDER));
    if pane.drop_hover() {
        panel = panel.push(
            container(text(DROP_TARGET).size(12))
                .padding(6)
                .width(Fill)
                .style(drop_hover_style),
        );
    }
    panel = panel.push(
        row![sidebar(pane), hairline_v(colors::SIDEBAR_BORDER), catalog(pane, &cards)]
            .height(Fill),
    );
    panel = panel.push(hairline_h(colors::TRAY_BORDER));
    panel = panel.push(selection_tray(&tray_line, tray_meta.as_deref()));

    os_file_drop_zone(
        container(panel)
            .style(panel_style)
            .height(Fill)
            .into(),
        shell.is_seated(),
        |event| Message::BrowserDropHover(matches!(event, DropEvent::HoverEnter)),
    )
}

/// browser-library.css:28-38 `.browserHeader`。
fn header(pane: &BrowserPane) -> Element<'static, Message> {
    container(
        row![
            text(HEADER)
                .size(font_size::HEADER_TITLE)
                .font(bold_font()),
            space().width(Fill),
            text(pane.library_root_name())
                .size(font_size::HEADER_SPAN)
                .color(colors::HEADER_SPAN_FG),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .padding([0, dims::HEADER_PAD_X as u16])
    .width(Fill)
    .height(dims::HEADER_H)
    .style(header_style)
    .into()
}

/// browser-library.css:40-77 `.browserToolbar`(検索欄 + Filters toggle)。
/// history ‹›(html:23-24)は product にナビゲーション履歴の feature が無いため
/// 置かない(module doc の Q0 節)。
fn toolbar(pane: &BrowserPane) -> Element<'static, Message> {
    // 先に owned な値へ落としてから closure へ渡す(`&BrowserPane` を
    // `move` closure が抱えると Element の寿命が `pane` の借用に縛られる —
    // `sidebar`/`catalog_header`/`filter_shelf` と同じ流儀)。
    let shelf_open = pane.shelf_open();
    let search = text_input(SEARCH_PLACEHOLDER, pane.query().to_owned())
        .size(font_size::SEARCH)
        .padding([0, dims::SEARCH_PAD_X as u16])
        .width(Fill)
        .style(search_style)
        .on_input(Message::BrowserQueryChanged);

    let filters = button(
        text(FILTERS_TOGGLE)
            .size(font_size::TOOLBAR_BUTTON)
            .color(if shelf_open {
                colors::CHIP_SELECTED_FG
            } else {
                colors::CONTROL_FG
            }),
    )
    .padding([0, dims::TOOLBAR_BUTTON_PAD_X as u16])
    .height(dims::CONTROL_H)
    .style(move |_theme, status| toggle_button_style(status, shelf_open))
    .on_press(Message::BrowserFilterShelfToggled);

    container(
        row![search, filters]
            .spacing(dims::TOOLBAR_GAP)
            .align_y(Center)
            .height(dims::CONTROL_H),
    )
    .padding([dims::TOOLBAR_PAD_Y as u16, dims::TOOLBAR_PAD_X as u16])
    .width(Fill)
    .height(dims::TOOLBAR_H)
    .style(toolbar_style)
    .into()
}

/// browser-library.css:101-151 `.librarySidebar`(html:45-49 LIBRARY 相当 —
/// COLLECTIONS/PLACES は module doc の Q0 節により置かない)。既存の3 rail
/// (All media/Project/Recent)を `.locationRow` の見た目で描く。
fn sidebar(pane: &BrowserPane) -> Element<'static, Message> {
    let mut list = column![sidebar_heading(RAIL_HEADING)];
    for (label, value) in [
        (RAIL_ALL, BrowserRail::AllMedia),
        (RAIL_PROJECT, BrowserRail::Project),
        (RAIL_RECENT, BrowserRail::Recent),
    ] {
        let selected = pane.rail() == value;
        // browser-library.css:132/144 `border-left: 2px solid …`(選択の左帯)。
        // `iced::Border` は四辺一括(module doc「iced の flex 相当」節と同じ
        // 制約)なので、accent は button の外側に**別の要素**として置く
        // (`hairline_h`/`_v` と同じ判断)。accent 自身は押せなくてよい —
        // 2px 幅の的を外しても隣の button がそのまま受ける。
        let accent = container(space())
            .width(dims::ROW_ACCENT_W)
            .height(dims::ROW_H)
            .style(move |_theme: &iced::Theme| container::Style {
                background: Some(if selected {
                    colors::ACCENT.into()
                } else {
                    Color::TRANSPARENT.into()
                }),
                ..container::Style::default()
            });
        let label_button = button(
            text(label)
                .size(font_size::ROW)
                .color(if selected {
                    colors::ROW_SELECTED_FG
                } else {
                    colors::ROW_FG
                }),
        )
        .padding([0, dims::ROW_PAD_X as u16])
        .width(Fill)
        .height(dims::ROW_H)
        .style(move |_theme, status| row_button_style(status, selected))
        .on_press(Message::BrowserRailChosen(value));
        list = list.push(row![accent, label_button].height(dims::ROW_H));
    }

    container(scrollable(list))
        .padding(
            iced::padding::top(dims::SIDEBAR_PAD_TOP).bottom(dims::SIDEBAR_PAD_BOTTOM),
        )
        .width(dims::sidebar_width(PANE_W))
        .height(Fill)
        .style(sidebar_style)
        .into()
}

fn sidebar_heading(title: &'static str) -> Element<'static, Message> {
    container(text(title).size(font_size::SIDEBAR_H2).color(colors::SIDEBAR_H2_FG))
        .padding(iced::padding::left(dims::ROW_PAD_X).top(6.0))
        .width(Fill)
        .height(dims::SIDEBAR_H2_H)
        .into()
}

/// `.catalog`(header / filter shelf / result summary / grid)。
fn catalog(pane: &BrowserPane, cards: &[BrowserCard]) -> Element<'static, Message> {
    let mut column = column![
        catalog_header(pane),
        hairline_h(colors::CATALOG_HEADER_BORDER),
    ];
    if pane.shelf_open() {
        column = column.push(filter_shelf(pane));
        column = column.push(hairline_h(colors::SHELF_BORDER));
    }
    column = column.push(result_summary(cards.len()));
    column = column.push(grid(pane, cards));

    container(column).width(Fill).height(Fill).into()
}

/// browser-library.css:155-168 `.catalogHeader`(タイトル+path / view mode)。
/// css:158 `display:flex` + css:166 `margin-left:auto` は
/// `row![title.width(Fill), buttons]` が iced の layout で素直に再現する
/// (module doc「iced の flex 相当」節)。
fn catalog_header(pane: &BrowserPane) -> Element<'static, Message> {
    let (title, path) = catalog_title(pane);
    let title_col = column![
        text(title).size(font_size::CATALOG_TITLE).color(colors::CATALOG_TITLE_FG),
        text(path).size(font_size::CATALOG_PATH).color(colors::CATALOG_PATH_FG),
    ]
    .width(Fill);

    let mut modes = row![].spacing(dims::VIEW_BUTTON_GAP);
    for (glyph, mode) in [
        (VIEW_THUMBNAILS, BrowserViewMode::Thumbnails),
        (VIEW_GRID, BrowserViewMode::Grid),
        (VIEW_LIST, BrowserViewMode::List),
    ] {
        let pressed = pane.view() == mode;
        modes = modes.push(
            button(
                text(glyph)
                    .size(font_size::VIEW_BUTTON)
                    .color(if pressed {
                        colors::VIEW_PRESSED_FG
                    } else {
                        colors::CONTROL_FG
                    }),
            )
            .padding(0)
            .width(dims::VIEW_BUTTON_W)
            .height(dims::CONTROL_H)
            .style(move |_theme, status| toggle_button_style(status, pressed))
            .on_press(Message::BrowserViewChosen(mode)),
        );
    }

    container(
        row![title_col, modes]
            .spacing(dims::VIEW_BUTTON_GAP)
            .align_y(Center),
    )
    .padding([0, dims::CATALOG_HEADER_PAD_X as u16])
    .width(Fill)
    .height(dims::CATALOG_HEADER_H)
    .style(catalog_header_style)
    .into()
}

/// browser-library.html:222-223 titles(source rail の名乗り)。
fn catalog_title(pane: &BrowserPane) -> (String, String) {
    match pane.rail() {
        BrowserRail::AllMedia => (RAIL_ALL.to_owned(), format!("Library \u{b7} {}", pane.library_root_name())),
        BrowserRail::Project => (RAIL_PROJECT.to_owned(), "Library \u{b7} this project".to_owned()),
        BrowserRail::Recent => (RAIL_RECENT.to_owned(), "Library \u{b7} newest first".to_owned()),
    }
}

/// browser-library.css:170-202 `.filterShelf`(kind chip 3つ + Clear)。
fn filter_shelf(pane: &BrowserPane) -> Element<'static, Message> {
    let mut row = row![text(FILTERS_LABEL)
        .size(font_size::SHELF_LABEL)
        .color(colors::SHELF_LABEL_FG)]
    .spacing(dims::SHELF_GAP)
    .align_y(Center);

    for (label, kind) in [
        (CHIP_VIDEO, "video"),
        (CHIP_IMAGE, "image"),
        (CHIP_AUDIO, "audio"),
    ] {
        let selected = pane.kind_filter() == Some(kind);
        row = row.push(
            button(
                text(label)
                    .size(font_size::CHIP)
                    .color(if selected {
                        colors::CHIP_SELECTED_FG
                    } else {
                        colors::CHIP_FG
                    }),
            )
            .padding([dims::CHIP_PAD_Y as u16, dims::CHIP_PAD_X as u16])
            .height(dims::CHIP_MIN_H)
            .style(move |_theme, status| chip_style(status, selected))
            .on_press(Message::BrowserKindFilterChosen(kind)),
        );
    }

    row = row.push(space().width(Fill));
    row = row.push(
        button(text(CLEAR_FILTER).size(font_size::CHIP).color(colors::CLEAR_FG))
            .padding(0)
            .style(|_theme, _status| button::Style {
                background: None,
                border: iced::Border::default(),
                text_color: colors::CLEAR_FG,
                ..button::Style::default()
            })
            .on_press(Message::BrowserFiltersCleared),
    );

    container(row)
        .padding([dims::SHELF_PAD_Y as u16, dims::SHELF_PAD_X as u16])
        .width(Fill)
        .height(iced::Length::Shrink)
        .style(shelf_style)
        .into()
}

/// browser-library.css:204-213 `.resultSummary`。
fn result_summary(count: usize) -> Element<'static, Message> {
    container(
        row![
            text(RESULTS_LABEL).size(font_size::SUMMARY_TITLE),
            space().width(Fill),
            text(count.to_string())
                .size(font_size::SUMMARY_COUNT)
                .color(colors::SUMMARY_COUNT_FG),
        ]
        .align_y(Center),
    )
    .padding([0, dims::SUMMARY_PAD_X as u16])
    .width(Fill)
    .height(dims::SUMMARY_H)
    .into()
}

/// browser-library.css:215-276 `.thumbnailGrid` + `.libraryCard`(3 view mode)。
fn grid(pane: &BrowserPane, cards: &[BrowserCard]) -> Element<'static, Message> {
    if cards.is_empty() {
        let line = empty_line(pane);
        return container(text(line).size(12)).padding(8).width(Fill).into();
    }

    let view = pane.view();
    let columns = dims::columns(view);
    let mut body = column![].spacing(0).width(Fill);
    let mut cells = row![].spacing(0);
    let mut in_row = 0;
    for card in cards {
        cells = cells.push(browser_card(card.clone(), view, columns));
        in_row += 1;
        if in_row == columns {
            body = body.push(cells);
            cells = row![];
            in_row = 0;
        }
    }
    if in_row > 0 {
        while in_row < columns {
            cells = cells.push(space::horizontal());
            in_row += 1;
        }
        body = body.push(cells);
    }
    // browser-library.css:222 `.thumbnailGrid { padding: 0 1px 3px }`。
    container(scrollable(body))
        .padding(
            iced::padding::left(dims::GRID_PAD_X)
                .right(dims::GRID_PAD_X)
                .bottom(dims::GRID_PAD_BOTTOM),
        )
        .width(Fill)
        .height(Fill)
        .into()
}

/// 空 grid の一言(Q7)。検索/filter が0件にしたのか rail 自体が空なのかを
/// 区別する(黙って同じ文言を返さない)。
fn empty_line(pane: &BrowserPane) -> String {
    if !pane.query().is_empty() || pane.kind_filter().is_some() {
        return EMPTY_FILTERED.to_owned();
    }
    match pane.rail() {
        BrowserRail::AllMedia => empty_all(&pane.library_root_name()),
        BrowserRail::Project => EMPTY_PROJECT.to_owned(),
        BrowserRail::Recent => EMPTY_RECENT.to_owned(),
    }
}

/// card 1枚。click=選択 / double-click=配置(Q1)。view mode で thumb/copy の
/// 並びが変わる(css:269-276)。
///
/// `columns` は呼び手([`grid`])が既に知っている列数をそのまま渡す —
/// [`dims::thumb_height`] は iced が実行時に読めない「幅から高さを導く」css の
/// `aspect-ratio` 計算を、既知の pane 幅 + 列数から前もって解く。
fn browser_card(card: BrowserCard, view: BrowserViewMode, columns: usize) -> Element<'static, Message> {
    let selected = card.selected;
    let thumb_h = match view {
        BrowserViewMode::List => dims::LIST_THUMB_W / dims::THUMB_ASPECT,
        BrowserViewMode::Grid | BrowserViewMode::Thumbnails => {
            dims::thumb_height(PANE_W, columns)
        }
    };
    let thumb = browser_thumb(&card, selected, thumb_h);

    let body: Element<'static, Message> = match view {
        BrowserViewMode::List => row![
            container(thumb).width(dims::LIST_THUMB_W).height(thumb_h),
            card_copy(&card),
        ]
        .spacing(dims::LIST_COPY_GAP)
        .align_y(Center)
        .into(),
        BrowserViewMode::Grid => column![thumb, card_copy(&card)]
            .spacing(dims::CARD_COPY_GAP)
            .into(),
        // css:272 thumbnails view は cardCopy を出さない。
        BrowserViewMode::Thumbnails => thumb,
    };

    let pad = if view == BrowserViewMode::Thumbnails {
        dims::CARD_PAD_THUMBNAILS
    } else {
        dims::CARD_PAD
    };

    mouse_area(
        container(body)
            .padding(pad)
            .width(Fill)
            .style(move |theme| card_style(theme, selected)),
    )
    .interaction(iced::mouse::Interaction::Pointer)
    .on_press(Message::BrowserCardClicked(card.id.clone()))
    .on_double_click(Message::BrowserCardActivated(card.id))
    .into()
}

/// browser-library.css:240-262 `.libraryThumb`(16:9 枠 + kind glyph)。
/// `height` は呼び手が [`dims::thumb_height`] 等で解いた実 px(iced は
/// 幅から高さを自動で導けない — module doc「iced の flex 相当」節の逆側)。
fn browser_thumb(card: &BrowserCard, selected: bool, height: f32) -> Element<'static, Message> {
    let is_svg = card.name.to_lowercase().ends_with(".svg");
    let (bg, glyph) = match card.kind {
        "video" => (colors::THUMB_VIDEO, "\u{25b6}"), // html:123
        "audio" => (colors::THUMB_AUDIO, "\u{266a}"), // html:132
        _ if is_svg => (colors::THUMB_SVG, "SVG"),    // html:126
        _ => (colors::THUMB_IMAGE, ""),               // html:129
    };
    let inner: Element<'static, Message> = match &card.thumbnail {
        Some(path) => iced::widget::image(iced::widget::image::Handle::from_path(path.clone()))
            .width(Fill)
            .height(Fill)
            .into(),
        None if !glyph.is_empty() => container(
            text(glyph)
                .size(font_size::THUMB_GLYPH)
                .color(colors::THUMB_GLYPH_FG),
        )
        .padding(dims::THUMB_PAD)
        .into(),
        None => space().into(),
    };
    container(inner)
        .width(Fill)
        .height(height)
        .style(move |theme: &iced::Theme| thumb_style(theme, bg, selected))
        .into()
}

/// browser-library.css:264-267 `.cardCopy`(主題行 = name / 補助行 = meta)。
fn card_copy(card: &BrowserCard) -> Element<'static, Message> {
    column![
        text(card.name.clone())
            .size(font_size::CARD_NAME)
            .color(colors::CARD_NAME_FG),
        text(card.meta.clone())
            .size(font_size::CARD_META)
            .color(colors::CARD_META_FG),
    ]
    .width(Fill)
    .into()
}

/// browser-library.css:278-295 `.selectionTray`(html:170 の2行: 名前 + meta)。
/// "Edit tags" は tag 編集 API が無いため置かない(module doc の Q0 節)。
fn selection_tray(line: &str, meta: Option<&str>) -> Element<'static, Message> {
    let mut content = row![
        container(space())
            .width(dims::TRAY_DOT)
            .height(dims::TRAY_DOT)
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(colors::ACCENT.into()),
                ..container::Style::default()
            }),
        text(line.to_owned())
            .size(font_size::TRAY_NAME)
            .color(colors::TRAY_NAME_FG),
    ]
    .spacing(dims::TRAY_GAP)
    .align_y(Center);
    if let Some(meta) = meta {
        content = content.push(
            text(meta.to_owned())
                .size(font_size::TRAY_META)
                .color(colors::TRAY_META_FG),
        );
    }

    container(content)
        .padding([0, dims::TRAY_PAD_X as u16])
        .width(Fill)
        .height(dims::TRAY_H)
        .style(tray_style)
        .into()
}

// ---------------------------------------------------------------------------
// OS drag のファイル受け皿(view.rs の実装をそのまま切り出し — 意味は不変)。
// ---------------------------------------------------------------------------

/// OS drag のファイル受け皿表示。**`crate::widgets::drop_zone` とは意味が違う**
/// — widgets 版はマウス cursor の面内滞在を hover と呼ぶ汎用 widget で、
/// Browser が要るのは OS がファイルを窓上へ運んでいる間だけ点く受け皿表示
/// (`FileHovered` / `FilesHoveredLeft` window event)である。
fn os_file_drop_zone<'a, M>(
    inner: iced::Element<'a, M>,
    accepting: bool,
    on_event: impl Fn(DropEvent) -> M + 'a,
) -> iced::Element<'a, M>
where
    M: 'a,
{
    use iced::advanced::widget::{Operation, Tree};
    use iced::advanced::{layout, mouse, overlay, renderer, Layout, Shell as IcedShell, Widget};
    use iced::{Event, Length, Rectangle, Size, Vector};

    struct OsFileDropZone<'a, M> {
        inner: Element<'a, M>,
        accepting: bool,
        on_event: Box<dyn Fn(DropEvent) -> M + 'a>,
    }

    impl<M> Widget<M, iced::Theme, iced::Renderer> for OsFileDropZone<'_, M> {
        fn size(&self) -> Size<Length> {
            self.inner.as_widget().size()
        }

        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.inner
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn diff(&mut self, tree: &mut Tree) {
            tree.diff_children(std::slice::from_mut(&mut self.inner));
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &iced::Renderer,
            operation: &mut dyn Operation,
        ) {
            self.inner
                .as_widget_mut()
                .operate(&mut tree.children[0], layout, renderer, operation);
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &iced::Renderer,
            shell: &mut IcedShell<'_, M>,
            viewport: &Rectangle,
        ) {
            self.inner.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                shell,
                viewport,
            );

            match event {
                Event::Window(iced::window::Event::FileHovered(_)) if self.accepting => {
                    shell.publish((self.on_event)(DropEvent::HoverEnter));
                }
                Event::Window(
                    iced::window::Event::FilesHoveredLeft | iced::window::Event::FileDropped(_),
                ) => {
                    shell.publish((self.on_event)(DropEvent::HoverLeave));
                }
                _ => {}
            }
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &iced::Renderer,
        ) -> mouse::Interaction {
            self.inner.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut iced::Renderer,
            theme: &iced::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.inner.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &iced::Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, M, iced::Theme, iced::Renderer>> {
            self.inner.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }

    Element::new(OsFileDropZone {
        inner,
        accepting,
        on_event: Box::new(on_event),
    })
}

// ---------------------------------------------------------------------------
// style ヘルパ — 全部 [`colors`] だけを読む(勝手な hex を発明しない)。
// ---------------------------------------------------------------------------

fn bold_font() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..iced::Font::DEFAULT
    }
}

fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::PANEL_BG.into()),
        ..container::Style::default()
    }
}

fn header_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::PANEL_BG.into()),
        ..container::Style::default()
    }
}

fn toolbar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::PANEL_BG.into()),
        ..container::Style::default()
    }
}

fn sidebar_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::SIDEBAR_BG.into()),
        ..container::Style::default()
    }
}

fn catalog_header_style(_theme: &iced::Theme) -> container::Style {
    container::Style::default()
}

fn shelf_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::SHELF_BG.into()),
        ..container::Style::default()
    }
}

fn tray_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::TRAY_BG.into()),
        ..container::Style::default()
    }
}

/// 単辺の罫線(css の `border-bottom`/`border-top`/`border-right` 相当)。
/// `iced::Border` は四辺共通幅なので `container::Style.border` では単辺を
/// 表せない — この製品は既に `rule::horizontal`/`vertical` を1px の同胞
/// widget として置く流儀(`theme/style.rs::separator`)なので、Browser も
/// それに揃える。
fn hairline_h(color: Color) -> Element<'static, Message> {
    iced::widget::rule::horizontal(dims::HAIRLINE)
        .style(move |_theme: &iced::Theme| iced::widget::rule::Style {
            color,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn hairline_v(color: Color) -> Element<'static, Message> {
    iced::widget::rule::vertical(dims::HAIRLINE)
        .style(move |_theme: &iced::Theme| iced::widget::rule::Style {
            color,
            radius: 0.0.into(),
            fill_mode: iced::widget::rule::FillMode::Full,
            snap: true,
        })
        .into()
}

fn drop_hover_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(colors::drop_hover_bg(colors::ACCENT).into()),
        border: iced::Border {
            color: colors::ACCENT,
            width: dims::HAIRLINE,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// source rail の1行(browser-library.css:126-147 `.locationRow`)。左の
/// accent bar は `sidebar()` が別要素で描く(このボタンは面+字だけ)。
fn row_button_style(status: button::Status, selected: bool) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed if !selected => {
            Some(colors::ROW_HOVER_BG.into())
        }
        _ if selected => Some(colors::ROW_SELECTED_BG.into()),
        _ => None,
    };
    button::Style {
        background,
        text_color: if selected {
            colors::ROW_SELECTED_FG
        } else {
            colors::ROW_FG
        },
        border: iced::Border::default(),
        ..button::Style::default()
    }
}

/// Filters / view-mode ボタン共通(browser-library.css:53-58 + selected 段の
/// css:168 の写し)。
fn toggle_button_style(status: button::Status, pressed: bool) -> button::Style {
    let (bg, border, fg) = if pressed {
        (colors::VIEW_PRESSED_BG, colors::ACCENT, colors::VIEW_PRESSED_FG)
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        (colors::CONTROL_BG, colors::THUMB_HOVER_BORDER, colors::CONTROL_FG)
    } else {
        (colors::CONTROL_BG, colors::CONTROL_BORDER, colors::CONTROL_FG)
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: iced::Border {
            color: border,
            width: dims::HAIRLINE,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// filter chip(browser-library.css:188-201)。
fn chip_style(status: button::Status, selected: bool) -> button::Style {
    let (bg, border, fg) = if selected {
        (colors::CHIP_SELECTED_BG, colors::ACCENT, colors::CHIP_SELECTED_FG)
    } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
        (colors::CHIP_BG, colors::THUMB_HOVER_BORDER, colors::CHIP_FG)
    } else {
        (colors::CHIP_BG, colors::CHIP_BORDER, colors::CHIP_FG)
    };
    button::Style {
        background: Some(bg.into()),
        text_color: fg,
        border: iced::Border {
            color: border,
            width: dims::CHIP_BORDER_W,
            radius: dims::CHIP_RADIUS.into(),
        },
        ..button::Style::default()
    }
}

/// card 面(browser-library.css:237 selected bg)。
fn card_style(theme: &iced::Theme, selected: bool) -> container::Style {
    let mut style = container::Style::default();
    let _ = theme;
    if selected {
        style.background = Some(colors::CARD_SELECTED_BG.into());
    }
    style
}

/// thumb 枠(browser-library.css:246/249/250)。
fn thumb_style(_theme: &iced::Theme, bg: Color, selected: bool) -> container::Style {
    container::Style {
        background: Some(bg.into()),
        border: iced::Border {
            color: if selected {
                colors::ACCENT
            } else {
                colors::THUMB_BORDER
            },
            width: dims::THUMB_BORDER_W,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

fn search_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => colors::ACCENT,
        _ => colors::SEARCH_BORDER,
    };
    text_input::Style {
        background: colors::SEARCH_BG.into(),
        border: iced::Border {
            color: border_color,
            width: dims::HAIRLINE,
            radius: 0.0.into(),
        },
        placeholder: colors::SEARCH_PLACEHOLDER_FG,
        value: colors::SEARCH_FG,
        selection: colors::ACCENT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// browser-library.css:349 の narrow sidebar(html:39 の値は静的抽出できない
    /// ため css の宣言値を直接検証する)。
    #[test]
    fn a_narrow_pane_hands_the_sidebar_width_back_to_the_catalog() {
        assert_eq!(dims::sidebar_width(900.0), dims::SIDEBAR_W);
        assert_eq!(dims::sidebar_width(PANE_W), dims::SIDEBAR_W_NARROW);
        assert!(PANE_W <= dims::NARROW_BREAKPOINT_W, "既定 pane 幅は narrow 側");
    }

    /// html:94-97 / css:218,269,273 の3態。
    #[test]
    fn view_modes_map_to_the_css_column_counts() {
        assert_eq!(dims::columns(BrowserViewMode::Thumbnails), 4);
        assert_eq!(dims::columns(BrowserViewMode::Grid), 2);
        assert_eq!(dims::columns(BrowserViewMode::List), 1);
    }

    /// **thumbnail は0高さへ潰れない(潰れ回帰レーン、2026-08-19)。**
    /// `Shrink` の中の `Fill` は iced ではレイアウト前に高さが定まらず0になる
    /// 既知の罠(module doc「移植した機能」節・`thumb_height` の doc コメント
    /// 参照) — この pane は列数が一番多い Thumbnails(4列、最も潰れやすい)を
    /// 含め、`thumb_height` で先に実 px を解いてから `browser_thumb` へ渡す
    /// ことでその罠を避けている。両方の grid view で常に正の高さを返す
    /// ことをここで固定する(List view は `LIST_THUMB_W` 由来の別式なので
    /// 対象外)。
    #[test]
    fn thumb_height_is_never_crushed_flat() {
        for columns in [dims::COLUMNS_THUMBNAILS, dims::COLUMNS_GRID] {
            let height = dims::thumb_height(PANE_W, columns);
            assert!(
                height > 1.0,
                "thumb_height(PANE_W={PANE_W}, columns={columns}) = {height} — \
                 潰れたサムネイルの回帰"
            );
        }
    }
}
