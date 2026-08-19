//! CSS 計算値の抽出器具(`motolii_ui::css_metrics::extract` —
//! `docs/reviews/2026-08-19-css-computed-metrics-extraction.md`)で
//! `docs/mocks-ui/public/*.html` を実測し、iced 側の主要寸法定数と突き合わせる
//! oracle。GPU 不要・描画無し(layout だけ解いて読み戻す)。
//!
//! ## 両側とも実物を突き合わせる
//! ## 両側性が違う3本立て
//!
//! - **Timeline 側**(`timeline::semantics` の `pub const`)は実物を `use`
//!   して比較する — css か semantics.rs のどちらが変わってもこのテストが
//!   落ちる、正真の両側チェック
//! - **Inspector 側**(`inspector_pane::dims`)も同じ形にした。2026-08-19時点
//!   では `dims` が private module で外部 crate から到達できず、値を
//!   literal で転記して片側だけ(css 側が変われば落ちるが `dims` 自身の変更
//!   には気づかない)pin していた — 前レーンが「`pub` へ上げれば真の両方向に
//!   なる」と残した提案どおり、この round で `inspector_pane::dims` を
//!   `pub` に上げ(`inspector_pane.rs` の柵は「既存の意味を壊さない」で
//!   あって「可視性を変えない」ではない)、ここも `use` で実物を読むように
//!   直した。**注意**: `pub(crate)` では足りない — 統合テスト crate は
//!   ライブラリ crate の外側なので `pub(crate)` は見えず、真に両方向にする
//!   には `pub` が要る(実測済み)。
//! - **Browser 側**(`browser_pane::dims` の `pub const`)も実物を `use` して
//!   比較する **正真の両側チェック**。前レーンが「Inspector の `dims` を
//!   `pub(crate)` へ上げられれば両側化できる」と書き残していた提案を実行した
//!   ([`browser_pane`](motolii_shell_iced::browser_pane) は
//!   browser-revise レーンの新規ファイルなので、最初から `pub mod dims` で
//!   書ける — Inspector 側のような書き換え柵が無い)
//! - **Inspector 側**(`inspector_pane.rs` の `mod dims`)は private module
//!   で外部 crate から到達できず、かつ `inspector_pane.rs` は本レーンの柵
//!   (3レーン並走中のパネル実装)で書き換え禁止なので、`dims` の値を
//!   2026-08-19 時点の実測としてこのテストへ literal で転記して pin した。
//!   **片側の保証**にしかならない: css 側が変われば落ちるが、
//!   `inspector_pane.rs` の `dims` を変えても気づかない
//!   (`dims` を触る変更をする人が、このテストの literal も一緒に見直す
//!   運用が要る — 詳細は上記 review 文書)
//!
//! ## 既知の不一致は「一致」を主張しない
//!
//! Timeline の `RAIL_W` / `TRANSPORT_H` / `OVERVIEW_H` は現時点で css mock の
//! 値と食い違う(`timeline/semantics.rs` のコメントが `/tmp/egui-same-doc.png`
//! を出所と自己申告しており、css mock からの転記ではないと明言している —
//! review 文書参照)。ここでは不一致を等値 assert にしない: 現状の両側の値を
//! それぞれ literal で pin するだけの [`timeline_known_divergences_are_pinned`]
//! にして、テストを赤いまま残さずに将来の無自覚なドリフトだけ拾う。

use std::path::PathBuf;

use motolii_shell_iced::inspector_pane::dims;
use motolii_shell_iced::browser_pane::dims as browser_dims;
use motolii_shell_iced::timeline::semantics::{OVERVIEW_H, RAIL_W, ROW_H, TRANSPORT_H};
use motolii_ui::css_metrics::extract;
use serde_json::Value;

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

fn has_class(row: &Value, name: &str) -> bool {
    row["classes"]
        .as_array()
        .map(|classes| classes.iter().any(|c| c.as_str() == Some(name)))
        .unwrap_or(false)
}

fn tag_is(row: &Value, tag: &str) -> bool {
    row["tag"].as_str() == Some(tag)
}

fn path_ends_with(row: &Value, suffix: &str) -> bool {
    row["path"].as_str().is_some_and(|p| p.ends_with(suffix))
}

fn box_w(row: &Value) -> f64 {
    row["box"]["w"].as_f64().expect("box.w is a number")
}

fn box_h(row: &Value) -> f64 {
    row["box"]["h"].as_f64().expect("box.h is a number")
}

fn box_x(row: &Value) -> f64 {
    row["box"]["x"].as_f64().expect("box.x is a number")
}

fn pad_left(row: &Value) -> f64 {
    row["padding"]["left"].as_f64().expect("padding.left is a number")
}

fn pad_top(row: &Value) -> f64 {
    row["padding"]["top"].as_f64().expect("padding.top is a number")
}

/// 条件に合う最初の行を返す。無ければ、探した条件と行数を添えて panic する
/// (抽出結果が空でも「マッチ0件」の理由が追いやすいように)。
fn find<'a>(rows: &'a [Value], what: &str, pred: impl Fn(&Value) -> bool) -> &'a Value {
    rows.iter()
        .find(|row| pred(row))
        .unwrap_or_else(|| panic!("'{what}' に一致する行が無い({} 行中)", rows.len()))
}

// ---------------------------------------------------------------------------
// Inspector — `inspector_pane::dims` は pub なので実物を比較する
// (2026-08-19 に private → pub へ上げて両方向にした)。
// ---------------------------------------------------------------------------

#[test]
fn inspector_dims_match_css_computed_values() {
    let html = repo_path("docs/mocks-ui/public/inspector-library.html");
    let rows = extract(&html, (520, 900)).expect("extract inspector-library.html");
    assert!(
        !rows.is_empty(),
        "抽出結果が空 — extract の呼び出し方が壊れていないか確認"
    );

    // inspector-library.css:29 `.panelHeader { height: ...29px }` — dims::PANEL_HEADER_H
    let panel_header = find(&rows, "header.panelHeader", |r| {
        has_class(r, "panelHeader") && tag_is(r, "header")
    });
    assert_eq!(
        box_h(panel_header),
        f64::from(dims::PANEL_HEADER_H),
        "dims::PANEL_HEADER_H"
    );

    // inspector-library.css:37 `.panelHeader::before { width:3px; height:13px }`
    // — dims::HEADER_ACCENT_W / HEADER_ACCENT_H。`::before` を歩かないと
    // この行自体が抽出結果に出ない(実測 — review 文書「詰まった点」参照)。
    let accent = find(&rows, "header.panelHeader::before", |r| {
        path_ends_with(r, "panelHeader::before")
    });
    assert_eq!(
        box_w(accent),
        f64::from(dims::HEADER_ACCENT_W),
        "dims::HEADER_ACCENT_W"
    );
    assert_eq!(
        box_h(accent),
        f64::from(dims::HEADER_ACCENT_H),
        "dims::HEADER_ACCENT_H"
    );

    // inspector-library.css:76 `.selectionSummary { height: 46px }` — dims::SUMMARY_H
    let summary = find(&rows, "section.selectionSummary", |r| {
        has_class(r, "selectionSummary")
    });
    assert_eq!(
        box_h(summary),
        f64::from(dims::SUMMARY_H),
        "dims::SUMMARY_H"
    );

    // inspector-library.css:99 `.layerStateButton { width:22px; height:21px }`
    // — dims::LAYER_STATE_W / LAYER_STATE_H
    let layer_state = find(&rows, "button.layerStateButton", |r| {
        has_class(r, "layerStateButton")
    });
    assert_eq!(
        box_w(layer_state),
        f64::from(dims::LAYER_STATE_W),
        "dims::LAYER_STATE_W"
    );
    assert_eq!(
        box_h(layer_state),
        f64::from(dims::LAYER_STATE_H),
        "dims::LAYER_STATE_H"
    );

    // inspector-library.css:122-126 `.columnHeader { height: 21px }` — dims::COLUMN_HEADER_H
    let column_header = find(&rows, "header.columnHeader", |r| {
        has_class(r, "columnHeader")
    });
    assert_eq!(
        box_h(column_header),
        f64::from(dims::COLUMN_HEADER_H),
        "dims::COLUMN_HEADER_H"
    );

    // inspector-library.css:141-142 `.tableSection h2 { height: 23px }` — dims::SECTION_H
    let section_h2 = find(&rows, "section.tableSection > h2", |r| {
        tag_is(r, "h2")
            && r["path"]
                .as_str()
                .is_some_and(|p| p.contains("tableSection"))
    });
    assert_eq!(
        box_h(section_h2),
        f64::from(dims::SECTION_H),
        "dims::SECTION_H"
    );

    // inspector-library.css:291 `.propertyRow::before { width: 3px }` — dims::ROW_BAND_W
    let row_band = find(&rows, "div.propertyRow::before", |r| {
        path_ends_with(r, "propertyRow::before")
    });
    assert_eq!(
        box_w(row_band),
        f64::from(dims::ROW_BAND_W),
        "dims::ROW_BAND_W"
    );

    // inspector-library.css:118 `repeat(3, 64px)`(値セルの grid 列)— dims::VALUE_COL_W。
    // 解決後の実測幅(css の literal をそのまま読むのではなく、grid が実際に
    // 割り付けた幅)。
    let value_col = find(&rows, "header.columnHeader > span (64px)", |r| {
        tag_is(r, "span") && path_ends_with(r, "columnHeader > span") && box_w(r) == 64.0
    });
    assert_eq!(
        box_w(value_col),
        f64::from(dims::VALUE_COL_W),
        "dims::VALUE_COL_W"
    );

    // inspector-library.css:207 `.effectBadge { width:17px; height:13px }`
    // — dims::FX_BADGE_W / FX_BADGE_H
    let fx_badge = find(&rows, "span.effectBadge", |r| has_class(r, "effectBadge"));
    assert_eq!(
        box_w(fx_badge),
        f64::from(dims::FX_BADGE_W),
        "dims::FX_BADGE_W"
    );
    assert_eq!(
        box_h(fx_badge),
        f64::from(dims::FX_BADGE_H),
        "dims::FX_BADGE_H"
    );

    // inspector-library.css:217 `.effectEnable { min-width:25px; height:15px }`
    // — dims::FX_PILL_MIN_W / FX_PILL_H
    let fx_pill = find(&rows, "button.effectEnable", |r| {
        has_class(r, "effectEnable")
    });
    assert_eq!(
        box_w(fx_pill),
        f64::from(dims::FX_PILL_MIN_W),
        "dims::FX_PILL_MIN_W"
    );
    assert_eq!(
        box_h(fx_pill),
        f64::from(dims::FX_PILL_H),
        "dims::FX_PILL_H"
    );

    // inspector-library.css:313-314 `.propertyName i { width:15px; height:15px }`
    // — dims::KIND_ICON(host TRANSFORM/APPEARANCE 行の kind icon。この round から
    // FX param 行にも同じ部品を使うが、値は host/FX で共通なのでここは1本のまま)。
    let kind_icon = find(&rows, "div.propertyName > i.hostIcon", |r| {
        tag_is(r, "i")
            && has_class(r, "hostIcon")
            && !r["path"].as_str().unwrap_or("").contains("::")
    });
    assert_eq!(
        box_w(kind_icon),
        f64::from(dims::KIND_ICON),
        "dims::KIND_ICON (width)"
    );
    assert_eq!(
        box_h(kind_icon),
        f64::from(dims::KIND_ICON),
        "dims::KIND_ICON (height)"
    );
}

// ---------------------------------------------------------------------------
// Timeline — timeline::semantics は pub const なので実物を比較する。
// ---------------------------------------------------------------------------

/// `ROW_H` は css の `.timelineRow{height:24px}` と実際に一致する — 唯一の
/// 「一致」を主張する Timeline 側の assert。
#[test]
fn timeline_row_h_matches_css() {
    let html = repo_path("docs/mocks-ui/public/timeline-library.html");
    let rows = extract(&html, (1200, 760)).expect("extract timeline-library.html");

    // 折りたたみ済みの group の中身は `hidden` で height=0 になるので、
    // height>0 の実物(root row)を拾う。
    let row = find(&rows, "div.timelineRow (visible)", |r| {
        has_class(r, "timelineRow") && box_h(r) > 0.0
    });
    assert_eq!(
        box_h(row) as f32,
        ROW_H,
        "timeline::semantics::ROW_H は .timelineRow の計算済み高さと一致するはず"
    );
}

/// 2026-08-19 時点で iced 側の値が css mock と一致しない箇所。
/// `timeline/semantics.rs` のコメントは寸法の出所を `/tmp/egui-same-doc.png`
/// (egui 版のスクリーンショット)だと自己申告しており、css mock からの転記
/// だと主張していない — ここでの不一致はその申告と矛盾しない。
///
/// 「一致」を主張する assert にはしない: 現状の両側の値をそれぞれ literal で
/// pin するだけにして、どちらかが無自覚に動いたらこのテストが落ちるように
/// する(指示: 不一致は現状の実測値で固定し、テストを赤いまま残さない)。
#[test]
fn timeline_known_divergences_are_pinned() {
    let html = repo_path("docs/mocks-ui/public/timeline-library.html");
    let rows = extract(&html, (1200, 760)).expect("extract timeline-library.html");

    // css: `.timelineHead{height:34px}`。TRANSPORT_H(30) は意味的に別の帯
    // (playhead読み・行数・grid刻み)を指しており、たまたま .overview と
    // 同じ 30px なだけで .timelineHead とは無関係(review 文書参照)。
    let head = find(&rows, "header.timelineHead", |r| {
        has_class(r, "timelineHead")
    });
    assert_eq!(box_h(head), 34.0, "css .timelineHead の計算済み高さ");
    assert_eq!(
        TRANSPORT_H, 30.0,
        "timeline::semantics::TRANSPORT_H(意味の違う帯)"
    );

    // css: `.overview{height:30px}`。OVERVIEW_H(22) は egui スクショ由来。
    let overview = find(&rows, "section.overview", |r| {
        has_class(r, "overview") && tag_is(r, "section")
    });
    assert_eq!(box_h(overview), 30.0, "css .overview の計算済み高さ");
    assert_eq!(OVERVIEW_H, 22.0, "timeline::semantics::OVERVIEW_H");

    // css: `.arrangement{grid-template-columns:196px ...}`(rail 列の実測幅)。
    // RAIL_W(210) は semantics.rs:30-33 のコメントで意図的な拡張だと明言
    // 済み(M/S ボタンが名前を圧迫しないための +14px、2026-08-19)。
    let rail = find(&rows, "div.columnHead", |r| has_class(r, "columnHead"));
    assert_eq!(
        box_w(rail),
        196.0,
        "css の rail 列(.columnHead)の計算済み幅"
    );
    assert_eq!(RAIL_W, 210.0, "timeline::semantics::RAIL_W(意図的な拡張)");
}

// ---------------------------------------------------------------------------
// Browser — browser_pane::dims は pub const なので実物を比較する
// (browser-revise レーン、2026-08-19)。
// ---------------------------------------------------------------------------

/// html:16 `<main class="libraryBrowser">` に静的な `data-tab`/`data-view` を
/// 差し込んだ一時 copy を作って `extract()` へ渡す。
///
/// html:205 `const state = {tab: 'media', ..., view: 'grid', ...}` の初期値を
/// そのまま属性へ写すだけ(JS エンジンは持ち込まない — 決定的な代入なので
/// 静的な文字列置換で足りる)。**`docs/mocks-ui` は書き換えない** — 一時 dir
/// (`std::env::temp_dir()`)へ copy を書く。
///
/// これが要るのは sidebar(`.tabScoped-media`)と filter shelf の中身
/// (`.filterGroup[data-filter-group="media"]`)だけ — どちらも `data-tab`
/// 一致の CSS attribute selector 経由でしか `display:block`/`flex` にならず、
/// Blitz は `<script>` を実行しないので既定の `display:none` のまま
/// (`docs/reviews/2026-08-19-css-computed-metrics-extraction.md`
/// 「Blitz 側で詰まった点」4 — 同文書が次の一手として書いていた自動化を実行)。
fn extract_browser_with_initial_media_tab() -> Vec<Value> {
    let html_path = repo_path("docs/mocks-ui/public/browser-library.html");
    let raw = std::fs::read_to_string(&html_path).expect("browser-library.html を読める");
    let marker = "<main class=\"libraryBrowser\" aria-label=\"Browser library concept\">";
    let patched_marker =
        "<main class=\"libraryBrowser\" data-tab=\"media\" data-view=\"grid\" aria-label=\"Browser library concept\">";
    assert!(
        raw.contains(marker),
        "html:16 の <main> tag が見つからない — mock の marker が変わっていないか確認 \
         (browser-library.html の変更に合わせてこの oracle も直す)"
    );
    let patched = raw.replacen(marker, patched_marker, 1);

    let dir = std::env::temp_dir().join("motolii-css-metrics-oracle-browser");
    std::fs::create_dir_all(&dir).expect("一時 dir を作れる");
    let patched_html = dir.join("browser-library-media-tab.html");
    std::fs::write(&patched_html, &patched).expect("一時 html を書ける");
    // `<link href="/browser-library.css">` は html 自身の隣(`resolve_href`)を
    // 見るので、css も同じ一時 dir へ複製する。
    std::fs::copy(
        html_path.with_file_name("browser-library.css"),
        dir.join("browser-library.css"),
    )
    .expect("browser-library.css を一時 dir へ複製できる");

    extract(&patched_html, (900, 600)).expect("extract patched browser-library.html")
}

/// JS の初期状態が要らない箇所(header/toolbar/catalogHeader/resultSummary/
/// grid/selectionTray)は素の html をそのまま実測する。
#[test]
fn browser_dims_match_css_computed_values() {
    let html = repo_path("docs/mocks-ui/public/browser-library.html");
    let rows = extract(&html, (900, 600)).expect("extract browser-library.html");
    assert!(!rows.is_empty(), "抽出結果が空");

    // browser-library.css:29 `.browserHeader{height:26px}` — dims::HEADER_H。
    let header = find(&rows, "header.browserHeader", |r| {
        has_class(r, "browserHeader")
    });
    assert_eq!(box_h(header), browser_dims::HEADER_H as f64, "dims::HEADER_H");
    assert_eq!(pad_left(header), browser_dims::HEADER_PAD_X as f64, "dims::HEADER_PAD_X");

    // browser-library.css:41 `.browserToolbar{height:30px}` — dims::TOOLBAR_H。
    let toolbar = find(&rows, "div.browserToolbar", |r| {
        has_class(r, "browserToolbar")
    });
    assert_eq!(box_h(toolbar), browser_dims::TOOLBAR_H as f64, "dims::TOOLBAR_H");
    assert_eq!(pad_top(toolbar), browser_dims::TOOLBAR_PAD_Y as f64, "dims::TOOLBAR_PAD_Y");
    assert_eq!(pad_left(toolbar), browser_dims::TOOLBAR_PAD_X as f64, "dims::TOOLBAR_PAD_X");

    // browser-library.css:53 共通 control 高さ(history/toolbar/viewModes button)
    // — dims::CONTROL_H。ここでは検索欄(実測 21px)で確かめる。
    let search = find(&rows, "input#library-search", |r| {
        r["id"].as_str() == Some("library-search")
    });
    assert_eq!(box_h(search), browser_dims::CONTROL_H as f64, "dims::CONTROL_H");

    // browser-library.css:156 `.catalogHeader{height:31px}` — dims::CATALOG_HEADER_H。
    let catalog_header = find(&rows, "header.catalogHeader", |r| {
        has_class(r, "catalogHeader") && tag_is(r, "header")
    });
    assert_eq!(
        box_h(catalog_header),
        browser_dims::CATALOG_HEADER_H as f64,
        "dims::CATALOG_HEADER_H"
    );
    assert_eq!(
        pad_left(catalog_header),
        browser_dims::CATALOG_HEADER_PAD_X as f64,
        "dims::CATALOG_HEADER_PAD_X"
    );

    // browser-library.css:167 `.viewModes button{width:21px}` + css:166 `gap:2px`
    // — dims::VIEW_BUTTON_W / VIEW_BUTTON_GAP。
    let view_buttons: Vec<&Value> = rows
        .iter()
        .filter(|r| r["path"].as_str().is_some_and(|p| p.ends_with("viewModes > button")))
        .collect();
    assert_eq!(view_buttons.len(), 3, "view mode ボタンは3つ(html:95-97)");
    for button in &view_buttons {
        assert_eq!(box_w(button), browser_dims::VIEW_BUTTON_W as f64, "dims::VIEW_BUTTON_W");
        assert_eq!(box_h(button), browser_dims::CONTROL_H as f64, "dims::CONTROL_H(view button)");
    }
    let gap = box_x(view_buttons[1]) - (box_x(view_buttons[0]) + box_w(view_buttons[0]));
    assert_eq!(gap, browser_dims::VIEW_BUTTON_GAP as f64, "dims::VIEW_BUTTON_GAP");

    // browser-library.css:205 `.resultSummary{height:21px}` — dims::SUMMARY_H。
    let summary = find(&rows, "div.resultSummary", |r| {
        has_class(r, "resultSummary")
    });
    assert_eq!(box_h(summary), browser_dims::SUMMARY_H as f64, "dims::SUMMARY_H");
    assert_eq!(pad_left(summary), browser_dims::SUMMARY_PAD_X as f64, "dims::SUMMARY_PAD_X");

    // browser-library.css:222 `.thumbnailGrid{padding:0 1px 3px}` — dims::GRID_PAD_X/BOTTOM。
    let grid = find(&rows, "div#thumbnail-grid.thumbnailGrid", |r| {
        has_class(r, "thumbnailGrid")
    });
    assert_eq!(pad_left(grid), browser_dims::GRID_PAD_X as f64, "dims::GRID_PAD_X");
    assert_eq!(
        grid["padding"]["bottom"].as_f64(),
        Some(browser_dims::GRID_PAD_BOTTOM as f64),
        "dims::GRID_PAD_BOTTOM"
    );

    // browser-library.css:227 `.libraryCard{padding:3px}` — dims::CARD_PAD。
    let card = find(&rows, "button.libraryCard (first)", |r| {
        has_class(r, "libraryCard")
    });
    assert_eq!(pad_left(card), browser_dims::CARD_PAD as f64, "dims::CARD_PAD");

    // browser-library.css:245 `.libraryThumb{padding:3px}` — dims::THUMB_PAD。
    let thumb = find(&rows, "span.libraryThumb (first)", |r| {
        has_class(r, "libraryThumb")
    });
    assert_eq!(pad_left(thumb), browser_dims::THUMB_PAD as f64, "dims::THUMB_PAD");

    // browser-library.css:279 `.selectionTray{height:27px}` — dims::TRAY_H。
    let tray = find(&rows, "footer.selectionTray", |r| {
        has_class(r, "selectionTray")
    });
    assert_eq!(box_h(tray), browser_dims::TRAY_H as f64, "dims::TRAY_H");
    assert_eq!(pad_left(tray), browser_dims::TRAY_PAD_X as f64, "dims::TRAY_PAD_X");

    // browser-library.css:291 `.selectionDot{width:5px;height:5px}` — dims::TRAY_DOT。
    let dot = find(&rows, "span.selectionDot", |r| has_class(r, "selectionDot"));
    assert_eq!(box_w(dot), browser_dims::TRAY_DOT as f64, "dims::TRAY_DOT (width)");
    assert_eq!(box_h(dot), browser_dims::TRAY_DOT as f64, "dims::TRAY_DOT (height)");
}

/// sidebar(`.locationRow`)と filter shelf(chip)は `data-tab="media"` が
/// 無いと静的抽出で 0×0 になる — [`extract_browser_with_initial_media_tab`]
/// で JS の初期状態を静的に補ってから実測する。
#[test]
fn browser_sidebar_and_filter_shelf_dims_match_css_computed_values() {
    let rows = extract_browser_with_initial_media_tab();
    assert!(!rows.is_empty(), "抽出結果が空");

    // browser-library.css:111 `.librarySidebar h2{height:16px}` — dims::SIDEBAR_H2_H。
    let heading = find(&rows, "aside.librarySidebar h2 (first, visible)", |r| {
        tag_is(r, "h2") && box_h(r) > 0.0
    });
    assert_eq!(box_h(heading), browser_dims::SIDEBAR_H2_H as f64, "dims::SIDEBAR_H2_H");
    assert_eq!(
        pad_left(heading),
        browser_dims::ROW_PAD_X as f64,
        "dims::ROW_PAD_X(sidebar h2 の左 padding も同じ 7px)"
    );

    // browser-library.css:128 `.locationRow{height:19px}` + css:130
    // `padding:3px 7px 0` — dims::ROW_H / ROW_PAD_X。
    let row = find(&rows, "button.locationRow (visible, no modifier class)", |r| {
        r.get("classes")
            .and_then(|c| c.as_array())
            .is_some_and(|c| {
                c.iter().any(|c| c.as_str() == Some("locationRow"))
                    && c.len() == 1 // indent/selected/collection 修飾なしの素の行
            })
            && box_h(r) > 0.0
    });
    assert_eq!(box_h(row), browser_dims::ROW_H as f64, "dims::ROW_H");
    assert_eq!(pad_left(row), browser_dims::ROW_PAD_X as f64, "dims::ROW_PAD_X");

    // browser-library.css:143 `.locationRow.indent{padding-left:13px}`
    // (7px を上書き) — dims::ROW_INDENT。
    let indent = find(&rows, "button.locationRow.indent", |r| {
        has_class(r, "indent") && has_class(r, "locationRow")
    });
    assert_eq!(pad_left(indent), browser_dims::ROW_INDENT as f64, "dims::ROW_INDENT");

    // browser-library.css:171 `.filterShelf{min-height:24px}` + css:176
    // `padding:3px 5px` — dims::SHELF_MIN_H / SHELF_PAD_X / SHELF_PAD_Y。
    let shelf = find(&rows, "div#filter-shelf.filterShelf", |r| {
        has_class(r, "filterShelf")
    });
    assert_eq!(box_h(shelf), browser_dims::SHELF_MIN_H as f64, "dims::SHELF_MIN_H");
    assert_eq!(pad_top(shelf), browser_dims::SHELF_PAD_Y as f64, "dims::SHELF_PAD_Y");
    assert_eq!(pad_left(shelf), browser_dims::SHELF_PAD_X as f64, "dims::SHELF_PAD_X");

    // browser-library.css:191-192 `.filterShelf button{min-height:17px;
    // padding:2px 5px}` — dims::CHIP_MIN_H / CHIP_PAD_X / CHIP_PAD_Y。
    let chip = find(&rows, "div.filterGroup > button (first)", |r| {
        r["path"]
            .as_str()
            .is_some_and(|p| p.ends_with("filterGroup > button") && box_h(r) > 0.0)
    });
    assert_eq!(box_h(chip), browser_dims::CHIP_MIN_H as f64, "dims::CHIP_MIN_H");
    assert_eq!(pad_top(chip), browser_dims::CHIP_PAD_Y as f64, "dims::CHIP_PAD_Y");
    assert_eq!(pad_left(chip), browser_dims::CHIP_PAD_X as f64, "dims::CHIP_PAD_X");
}

/// pane 幅から解く2つの派生値(module doc「iced の flex 相当」節): 実測できない
/// (幅依存の box は css_metrics の対象外 — 上記2テストのコメント参照)ので、
/// css の宣言値(narrow media query の breakpoint と幅)を直接検証する。
#[test]
fn browser_sidebar_width_and_thumb_height_follow_the_css_declarations() {
    // browser-library.css:348-349 `@media (max-width: 420px) { .librarySidebar { width: 92px } }`。
    assert_eq!(browser_dims::sidebar_width(900.0), browser_dims::SIDEBAR_W);
    assert_eq!(
        browser_dims::sidebar_width(browser_pane_pane_w()),
        browser_dims::SIDEBAR_W_NARROW,
        "既定 pane 幅は narrow 側(css:348 の breakpoint 420px 以下)"
    );

    // browser-library.css:241 `aspect-ratio: 16/9` — `thumb_height` が列幅から
    // 高さを逆算する式そのものを、独立に組み直して突き合わせる(実装の
    // 中身をコピーしているのではなく、css の宣言 `aspect-ratio:16/9` と
    // `.thumbnailGrid`/`.libraryCard` の padding という**別々の入力**から
    // 同じ答えに辿り着くかを見る)。
    let pane_w = browser_pane_pane_w();
    let columns = 2;
    let catalog_w = pane_w - browser_dims::sidebar_width(pane_w) - 2.0 * browser_dims::GRID_PAD_X;
    let cell_w = catalog_w / columns as f32;
    let expected_content_w = cell_w - 2.0 * browser_dims::CARD_PAD;
    let height = browser_dims::thumb_height(pane_w, columns);
    assert!(
        (height * browser_dims::THUMB_ASPECT - expected_content_w).abs() < 0.01,
        "thumb_height({pane_w}, {columns}) = {height} は 16:9 で列幅から解いた \
         高さと一致するはず(期待 content 幅 {expected_content_w})"
    );
}

/// `browser_pane::PANE_W` は private ではなく `pub` だが、この oracle は
/// `dims` だけを `use` しているので、循環的に依存を増やさないようここだけ
/// 直接参照する小さなヘルパにしている。
fn browser_pane_pane_w() -> f32 {
    motolii_shell_iced::browser_pane::PANE_W
}
