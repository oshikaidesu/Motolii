//! 落ちるテスト先行(発注書 §4)で書いた既知値テスト。期待値はすべて
//! `next/reference/mocks/*.html` の CSS 宣言から人手で読んだもの(mock は
//! `* { box-sizing: border-box }` なので、宣言した height/width が
//! そのまま border box の寸法になる)。この器具の役割は「モックを機械で測り、
//! 実装(taffy container / pane)の実寸と照合する ±1px oracle の分母」 —
//! ここで mock の宣言値と抽出値が一致することが、その分母の信用の根拠。

use std::path::PathBuf;
use std::sync::OnceLock;

use motolii_css_metrics::{extract, select, select_one, PANELS};
use serde_json::Value;

/// extract は1枚あたり数百 ms かかるので、テスト間で1回だけ解く。
fn browser_rows() -> &'static [Value] {
    static ROWS: OnceLock<Vec<Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let panel = PANELS.iter().find(|p| p.name == "browser").expect("browser panel");
        extract(&panel.html_path(), panel.default_viewport).expect("browser mock を解ける")
    })
}

fn inspector_rows() -> &'static [Value] {
    static ROWS: OnceLock<Vec<Value>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let panel = PANELS.iter().find(|p| p.name == "inspector").expect("inspector panel");
        extract(&panel.html_path(), panel.default_viewport).expect("inspector mock を解ける")
    })
}

fn box_of(row: &Value) -> (f64, f64, f64, f64) {
    let b = &row["box"];
    (
        b["x"].as_f64().unwrap(),
        b["y"].as_f64().unwrap(),
        b["w"].as_f64().unwrap(),
        b["h"].as_f64().unwrap(),
    )
}

/// oracle と同じ ±1px。mock の宣言は整数 px なので実質は一致要求。
fn assert_px(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= 1.0,
        "{what}: expected {expected} (±1px), got {actual}"
    );
}

// ---------------------------------------------------------------------------
// (a) browser-library.html の既知値
// ---------------------------------------------------------------------------

/// `.libraryTabs { height: 26px }`(browser-library.html の <style> 実測)。
#[test]
fn browser_library_tabs_height_is_26() {
    let row = select_one(browser_rows(), ".libraryTabs").expect(".libraryTabs が居る");
    let (_, _, _, h) = box_of(row);
    assert_px(h, 26.0, ".libraryTabs height");
}

/// `.libraryTabs button { font-size: 8px; border-bottom: 2px solid transparent }`。
/// 文字寸は computed value(文字列)で、下線は layout の border 辺で確かめる —
/// 両方の読み出し口が生きていることの証明。
#[test]
fn browser_library_tab_button_font_8px_underline_2px() {
    let rows = select(browser_rows(), ".libraryTabs button");
    assert!(!rows.is_empty(), ".libraryTabs button が1つ以上居る");
    let row = rows[0];
    assert_eq!(
        row["computed"]["font_size"].as_str().unwrap(),
        "8px",
        "タブ文字の font-size"
    );
    assert_px(
        row["border"]["bottom"].as_f64().unwrap(),
        2.0,
        "タブ下線(border-bottom)の太さ",
    );
}

// ---------------------------------------------------------------------------
// (b) 存在しない selector は Err
// ---------------------------------------------------------------------------

#[test]
fn unknown_selector_is_err() {
    assert!(select_one(browser_rows(), ".doesNotExistAnywhere").is_err());
    // 前段(ancestor)が実在しても、末尾が居なければ Err。
    assert!(select_one(browser_rows(), ".libraryTabs .doesNotExistAnywhere").is_err());
}

// ---------------------------------------------------------------------------
// (c) inspector モックの .keyButton 寸法
// ---------------------------------------------------------------------------

/// `.propertyRow { grid-template-columns: minmax(132px,1fr) repeat(3,64px) 26px }`
/// の末尾 26px 段が `.keyButton` の幅、`min-height: 25px` の行 stretch が高さ。
#[test]
fn inspector_key_button_is_26_by_25() {
    let rows = select(inspector_rows(), ".keyButton");
    assert!(!rows.is_empty(), ".keyButton が1つ以上居る");
    let (_, _, w, h) = box_of(rows[0]);
    assert_px(w, 26.0, ".keyButton width(grid 末尾 26px 段)");
    assert_px(h, 25.0, ".keyButton height(propertyRow min-height 25)");
}

// ---------------------------------------------------------------------------
// dimensions.json(トークン正本)との突き合わせ — 本命の用途の証明
// ---------------------------------------------------------------------------

/// トークン正本 `next/ui/motolii-tokens-rs/tokens/dimensions.json` の inspector
/// 節(I-tokens 2026-08-22 で inspector-library.html v3.1 へ出典統一済み)を、
/// この器具が同じ mock から独立に測り直して検証する。token → mock の転記が
/// ズレたらここが落ちる。
#[test]
fn dimensions_json_inspector_tokens_match_extracted_mock() {
    let tokens_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../motolii-tokens-rs/tokens/dimensions.json");
    let tokens: Value =
        serde_json::from_str(&std::fs::read_to_string(&tokens_path).expect("dimensions.json"))
            .expect("dimensions.json は JSON");
    let token = |key: &str| -> f64 {
        tokens[key]
            .as_f64()
            .unwrap_or_else(|| panic!("dimensions.json に {key} が居る"))
    };

    let rows = inspector_rows();

    // inspector_panel_width = 496 ← .inspectorShell { width: min(100%, 496px) }
    // (viewport 幅 520 > 496 なので min は 496 側に倒れる)
    let shell = select_one(rows, ".inspectorShell").expect(".inspectorShell");
    assert_px(box_of(shell).2, token("inspector_panel_width"), "inspector_panel_width");

    // inspector_row_height = 25 ← .propertyRow { min-height: 25px }
    let prow = select_one(rows, ".propertyRow").expect(".propertyRow");
    assert_px(box_of(prow).3, token("inspector_row_height"), "inspector_row_height");

    // inspector_section_header_height = 26 ← .tableSection h2 { height: 26px }
    let h2 = select_one(rows, ".tableSection h2").expect(".tableSection h2");
    assert_px(
        box_of(h2).3,
        token("inspector_section_header_height"),
        "inspector_section_header_height",
    );

    // inspector_value_width = 64 ← grid-template-columns の 64px 段(.valueCell)
    let cell = select_one(rows, ".valueCell").expect(".valueCell");
    assert_px(box_of(cell).2, token("inspector_value_width"), "inspector_value_width");

    // inspector_glyph_width = 26 ← 同 grid 末尾の 26px 段(.keyButton)
    let key = select_one(rows, ".keyButton").expect(".keyButton");
    assert_px(box_of(key).2, token("inspector_glyph_width"), "inspector_glyph_width");

    // panel_header_height = 29 ← .panelHeader { height: var(--mock-role-panel-header-height, 29px) }
    // (--mock-role-* を定義する mock-candidates.css は next/ に無く <link> は
    // 解決されない既知実測 — fallback の 29px が効く)
    let header = select_one(rows, ".panelHeader").expect(".panelHeader");
    assert_px(box_of(header).3, token("panel_header_height"), "panel_header_height");
}
