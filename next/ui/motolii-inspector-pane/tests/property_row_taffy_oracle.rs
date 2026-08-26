//! taffy 転写 本丸(裁定183)の ±1px oracle: [`motolii_inspector_pane::
//! property_row_css`](= mock `.propertyRow`/`.columnHeader` の
//! `grid-template-columns` 宣言、`transform_row` が **実際に本体コードで使う**
//! CSS 宣言文字列)を、視覚正本(`next/reference/mocks/inspector-library.html`
//! v3.1)を `motolii-css-metrics`(blitz-dom+stylo)で測った実測値と突き合わせる。
//!
//! settings-pane の第1号(`motolii-settings-pane/tests/
//! comp_cells_row_taffy_oracle.rs`)との違い: Settings は視覚正本 mock を
//! 持たないため分母を taffy 自身の直接解にしていたが、**Inspector にはモックが
//! 実在する**(発注書の起点)ので、ここでは分母をモック側の実測値にできる —
//! 「モック側も実装側も同じ taffy が矩形を解く」構図(`docs/reviews/
//! 2026-08-22-weblike-layout-survey.md` §3)がここで初めて成立する。
//!
//! ## gap の既知の乖離(mock=0 / 実装=`spacing_xs`)
//!
//! mock の `.columnHeader, .propertyRow { grid-template-columns: ... }` 宣言
//! 自体には `gap` が無い(列間 gap 0)。実装([`property_row_css`])は裁定168
//! 施工の兄弟間隔(`spacing_xs`)を1本の grid gap として持つ — これは
//! [`property_row_css`] の doc に書いた**意図的な**乖離(既存の柵が踏む
//! 2スケールでは `spacing_xs == sibling_gap_px` なので幾何は保たれる)。
//!
//! 固定幅3列(`.valueCell`×3)と末尾1列(`.keyButton`)は grid の固定 track
//! なので gap の有無に関わらず**幅そのもの**は変わらない(gap は track どうしの
//! 隙間であって track 自体の幅ではない)——この4列は mock 実測と直接 ±1px
//! 照合する。1列目(`.propertyName` = label、`minmax(132px,1fr)`)だけは
//! gap ぶん(4 gap × `spacing_xs`)だけ実装側が狭くなる — これも黙って見逃さず、
//! 「差が gap 4本ぶんちょうどであること」を直接検算する。

use iced_core::layout::Limits;
use iced_core::widget::Tree;
use iced_core::{Element, Length, Size};
use iced_widget::Space;

use motolii_css_metrics::{extract, select, select_one, PANELS};
use motolii_inspector_pane::property_row_css;
// `taffy` crate 自体への direct dependency はこの crate に増やさない —
// `motolii-taffy` が再輸出する版を正本として使う(`comp_cells_row_taffy_oracle.rs`
// と同じ理由)。
use motolii_taffy::{style_from_css_decl, TaffyBox};
use motolii_tokens_rs::Dimensions;

type Renderer = ();
type Theme = iced_core::Theme;
type Message = ();

/// `motolii-taffy/tests/layout_oracle.rs`・
/// `motolii-settings-pane/tests/comp_cells_row_taffy_oracle.rs` と同じ手口
/// (null renderer `()`、子は `Space::Fill` — 列幅そのものだけを見る、中身の
/// 意匠は関係ない)。`root_css` に `width:{container_width}px` を含めて渡す
/// (`TaffyBox::layout` の tight-limits 差し替えに頼らず、oracle 側は常に
/// 明示 width で解く)。
fn widget_columns(root_css: &str, column_count: usize) -> Vec<f32> {
    let mut root = TaffyBox::new(style_from_css_decl(root_css).expect("root css"));
    for _ in 0..column_count {
        root = root.push(
            motolii_taffy::taffy::Style::default(),
            Space::new().width(Length::Fill).height(Length::Fill),
        );
    }
    let mut element: Element<'_, Message, Theme, Renderer> = root.into();
    let mut tree = Tree::new(element.as_widget());
    tree.diff(element.as_widget_mut());
    let node = element.as_widget_mut().layout(
        &mut tree,
        &(),
        &Limits::new(Size::ZERO, Size::new(4096.0, 4096.0)),
    );
    node.children().iter().map(|c| c.bounds().width).collect()
}

/// mock 側: `.propertyRow`(文書順で最初 = TRANSFORM 節の Position 行)自身の
/// box 幅と、5列(`.propertyName`/`.valueCell`×3/`.keyButton`)の box 幅。
/// `select_one`/`select` は文書順で最初の一致を返す(`motolii-css-metrics`
/// の selector doc 参照)ので、Position 行が確実に拾える(mock 内で最初に
/// 出現する `.propertyRow`)。
fn mock_row_and_columns() -> (f32, [f32; 5]) {
    let panel = PANELS
        .iter()
        .find(|p| p.name == "inspector")
        .expect("inspector panel が PANELS に登録されていない");
    let rows = extract(&panel.html_path(), panel.default_viewport)
        .expect("inspector-library.html の抽出に失敗(モックのパス・構造が変わった疑い)");

    let row = select_one(&rows, ".propertyRow").expect(".propertyRow が見つからない");
    let row_width = row["box"]["w"].as_f64().expect("propertyRow box.w が無い") as f32;

    let width_of = |value: &serde_json::Value| -> f32 {
        value["box"]["w"].as_f64().expect("box.w が無い") as f32
    };

    let name = select_one(&rows, ".propertyRow .propertyName")
        .expect(".propertyRow .propertyName が見つからない(Position 行の label)");
    let cells = select(&rows, ".propertyRow .valueCell");
    assert!(
        cells.len() >= 3,
        ".propertyRow .valueCell が3個未満(Position 行の X/Y/Z が揃っていない): {}",
        cells.len()
    );
    let key = select_one(&rows, ".propertyRow .keyButton")
        .expect(".propertyRow .keyButton が見つからない(Position 行の Key セル)");

    (
        row_width,
        [
            width_of(name),
            width_of(cells[0]),
            width_of(cells[1]),
            width_of(cells[2]),
            width_of(key),
        ],
    )
}

/// (1) 固定4列(`.valueCell`×3 + `.keyButton`)は gap の有無に関わらず幅が
/// 変わらない track なので、実装(`property_row_css`)の解とモック実測が
/// ±1px で一致すること。
#[test]
fn property_row_fixed_columns_match_the_mock_within_1px() {
    let dims = Dimensions::default();
    let (row_width, mock_widths) = mock_row_and_columns();

    // `property_row_css` 自身も `width` を持つ(FINDING 参照 — `motolii-taffy`
    // 側の tight-limits 浮動小数点分岐に依存しない決定論的な明示 width)。
    // CSS の「後勝ち」どおり、モック実測の `row_width` で明示的に上書きして
    // 「同じ幅を渡したら同じ列幅を返すか」の oracle にする(後ろに書くのが肝 —
    // 前に書くと `property_row_css` 自身の `width` に上書きされて row_width が
    // 効かない)。
    let root_css = format!("{} width:{row_width}px;", property_row_css(dims));
    let taffy_widths = widget_columns(&root_css, 5);
    assert_eq!(taffy_widths.len(), 5, "grid の列数が5でない");

    // index 0 = label(`.propertyName`、gap ぶんの既知の乖離あり — 次のテスト
    // が別途検算する)。1..=4 が固定4列(value×3 + key)。
    for i in 1..=4 {
        let mock_w = mock_widths[i];
        let taffy_w = taffy_widths[i];
        assert!(
            (mock_w - taffy_w).abs() <= 1.0,
            "列{i}の幅がモックと±1pxで一致しない: mock={mock_w}px taffy={taffy_w}px \
             (row_width={row_width}px)"
        );
    }
}

/// (2) label 列(`.propertyName`、`minmax(132px,1fr)`)は実装側だけが4本の
/// grid gap(`spacing_xs`)を持つ分だけモックより狭い — この既知の乖離を
/// 「gap 4本ぶんちょうど」と直接検算する([`property_row_css`] の doc
/// 「gap の既知の乖離」参照)。
#[test]
fn property_row_label_column_narrows_by_exactly_the_documented_gap_budget() {
    let dims = Dimensions::default();
    let (row_width, mock_widths) = mock_row_and_columns();

    // `property_row_css` 自身も `width` を持つ(FINDING 参照 — `motolii-taffy`
    // 側の tight-limits 浮動小数点分岐に依存しない決定論的な明示 width)。
    // CSS の「後勝ち」どおり、モック実測の `row_width` で明示的に上書きして
    // 「同じ幅を渡したら同じ列幅を返すか」の oracle にする(後ろに書くのが肝 —
    // 前に書くと `property_row_css` 自身の `width` に上書きされて row_width が
    // 効かない)。
    let root_css = format!("{} width:{row_width}px;", property_row_css(dims));
    let taffy_widths = widget_columns(&root_css, 5);

    let mock_label = mock_widths[0];
    let taffy_label = taffy_widths[0];
    let expected_gap_budget = 4.0 * dims.theme().space.xs; // label→v0, v0→v1, v1→v2, v2→key の4本。

    assert!(
        ((mock_label - taffy_label) - expected_gap_budget).abs() <= 1.0,
        "label 列の縮み幅が gap 4本ぶん(spacing_xs×4={expected_gap_budget}px)と \
         ±1pxで一致しない: mock={mock_label}px taffy={taffy_label}px \
         (差={})",
        mock_label - taffy_label
    );
}
