//! CSS 計算値の抽出器具(`motolii_ui::css_metrics::extract` —
//! `docs/reviews/2026-08-19-css-computed-metrics-extraction.md`)で
//! `docs/mocks-ui/public/*.html` を実測し、iced 側の主要寸法定数と突き合わせる
//! oracle。GPU 不要・描画無し(layout だけ解いて読み戻す)。
//!
//! ## 両側とも実物を突き合わせる
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
