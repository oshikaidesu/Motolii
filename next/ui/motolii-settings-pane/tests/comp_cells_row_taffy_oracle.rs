//! taffy 転写 第1号(裁定183)の ±1px oracle: [`sections::comp_cells_row`] が
//! **実際に本体コードで使う** CSS 宣言(`sections::comp_cells_row_css` /
//! `sections::COMP_CELLS_LABEL_CSS` / `sections::comp_cell_css`)を、taffy の
//! 直接計算と突き合わせる。
//!
//! Settings に対応する mock(`next/reference/mocks/`)は無い(発注書の指示
//! どおり — Settings は視覚正本 HTML を持たない)。**分母は taffy 自身の
//! 直接解**にする: `motolii-taffy/tests/layout_oracle.rs` と全く同じ手口
//! (`TaffyBox` 経由の widget 解 vs `taffy::TaffyTree` 直接解、null renderer
//! `()`、子は `Space::Fill`)をこの crate の**本物の CSS 宣言文字列**に対して
//! 走らせる — 文字列の二重管理はしない(テストは `sections::` の pub 関数を
//! 直接呼ぶ)。
//!
//! 検分するのは2点:
//! 1. **(a) widget 解 == taffy 直接解**: `comp_cells_row` の実装
//!    (`TaffyBox::new(..).push(..)...`)が taffy の生の計算結果と1px の
//!    ズレも無く一致する(=「委譲」が実際に taffy の値をそのまま使っている
//!    ことの証拠、変換過程での丸め/取り違えが無い)。
//! 2. **(b) 手計算の画素**: `Dimensions::default()` から手で組んだ期待値
//!    (gap = `spacing_xs`、行高 = `inspector_row_height`、セル幅 =
//!    `inspector_value_width`、左 padding = `spacing_m`)と一致する —
//!    「CSS 宣言の字面が意図どおりの数字に化けている」ことの直接証拠。
//!
//! 値セル(`comp_cell_css`)は height を指定しない(旧実装どおり、内容の
//! 自然高で `align-items:center`)。この harness では中身の実測をしない
//! (`Space::Fill` — motolii-taffy 自身の oracle と同じ近似)ので、高さに
//! 依存する assertion はしない — 見る対象は主軸(横方向)の位置・幅と、
//! 明示 CSS 値が乗る行高/padding だけ。

use iced_core::layout::Limits;
use iced_core::widget::Tree;
use iced_core::{Element, Length, Rectangle, Size};
use iced_widget::Space;

use motolii_settings_pane::sections::{comp_cell_css, comp_cells_row_css, COMP_CELLS_LABEL_CSS};
// `taffy` crate 自体への direct dependency はこの crate に増やさない —
// `motolii-taffy` が再輸出する版を正本として使う(crate doc「バージョンの正本は
// この crate」参照)。
use motolii_taffy::{style_from_css_decl, taffy, TaffyBox};
use motolii_tokens_rs::Dimensions;

type Renderer = ();
type Theme = iced_core::Theme;
type Message = ();

/// COMPOSITION 節の実パネル程度の余裕(`inspector_panel_width` 相当 — Settings
/// 自体は固定幅トークンを持たないが、taffy 直接解と widget 解の一致を見るには
/// どの余白でも良い。行高より十分広い値を選ぶだけ)。
const AVAIL_W: f32 = 496.0;
const AVAIL_H: f32 = 600.0;

/// `boxed` / `widget_solve` / `direct_taffy_solve` は
/// `motolii-taffy/tests/layout_oracle.rs` と同一の手口(意図的に同型 — 分母を
/// 増やさない)。子は常に `Space::Fill`: 中身の実測ではなく「container が
/// 自分の style 通りに矩形を割ったか」だけを見る。
fn boxed<'a>(root_css: &str, child_css: &[String]) -> TaffyBox<'a, Message, Theme, Renderer> {
    let mut b = TaffyBox::new(style_from_css_decl(root_css).expect("root css"));
    for css in child_css {
        b = b.push(
            style_from_css_decl(css).expect("child css"),
            Space::new().width(Length::Fill).height(Length::Fill),
        );
    }
    b
}

fn widget_solve(root: TaffyBox<'_, Message, Theme, Renderer>) -> (Size, Vec<Rectangle>) {
    let mut element: Element<'_, Message, Theme, Renderer> = root.into();
    let mut tree = Tree::new(element.as_widget());
    tree.diff(element.as_widget_mut());
    let node = element.as_widget_mut().layout(
        &mut tree,
        &(),
        &Limits::new(Size::ZERO, Size::new(AVAIL_W, AVAIL_H)),
    );
    let children = node.children().iter().map(|c| c.bounds()).collect();
    (node.size(), children)
}

fn direct_taffy_solve(root_css: &str, child_css: &[String]) -> (Size, Vec<Rectangle>) {
    let mut tree = taffy::TaffyTree::<()>::new();
    let kids: Vec<taffy::NodeId> = child_css
        .iter()
        .map(|css| {
            tree.new_leaf_with_context(style_from_css_decl(css).expect("child css"), ())
                .expect("leaf")
        })
        .collect();
    let root = tree
        .new_with_children(style_from_css_decl(root_css).expect("root css"), &kids)
        .expect("root");
    tree.compute_layout_with_measure(
        root,
        taffy::Size {
            width: taffy::AvailableSpace::Definite(AVAIL_W),
            height: taffy::AvailableSpace::Definite(AVAIL_H),
        },
        |known, space, _id, ctx: Option<&mut ()>, _style| {
            if ctx.is_none() {
                return taffy::Size::ZERO;
            }
            let w = known.width.unwrap_or(match space.width {
                taffy::AvailableSpace::Definite(w) => w,
                _ => AVAIL_W,
            });
            let h = known.height.unwrap_or(match space.height {
                taffy::AvailableSpace::Definite(h) => h,
                _ => AVAIL_H,
            });
            taffy::Size { width: w, height: h }
        },
    )
    .expect("direct compute_layout");

    let root_layout = tree.layout(root).expect("root layout");
    let size = Size::new(root_layout.size.width, root_layout.size.height);
    let children = kids
        .iter()
        .map(|k| {
            let l = tree.layout(*k).expect("child layout");
            Rectangle {
                x: l.location.x,
                y: l.location.y,
                width: l.size.width,
                height: l.size.height,
            }
        })
        .collect();
    (size, children)
}

fn assert_rects_eq(widget: &[Rectangle], direct: &[Rectangle]) {
    assert_eq!(widget.len(), direct.len(), "子の数が違う");
    for (i, (w, d)) in widget.iter().zip(direct).enumerate() {
        assert!(
            (w.x - d.x).abs() <= 1.0
                && (w.y - d.y).abs() <= 1.0
                && (w.width - d.width).abs() <= 1.0
                && (w.height - d.height).abs() <= 1.0,
            "子{i} の矩形が taffy 直接解と ±1px を超えてズレている: widget={w:?} direct={d:?}"
        );
    }
}

/// COMPOSITION 節「Size (px)」/「Time」行(2フィールド)と同じ形 — label 1本
/// + 値セル2本。**本体コード(`sections::comp_cells_row`)が実際に使う CSS
/// 宣言文字列をそのまま使う**(ここでハードコードした別文字列は書かない)。
fn size_row_css(dims: Dimensions) -> (String, Vec<String>) {
    let root = comp_cells_row_css(dims);
    let children = vec![
        COMP_CELLS_LABEL_CSS.to_owned(),
        comp_cell_css(dims),
        comp_cell_css(dims),
    ];
    (root, children)
}

/// (a) widget 解 == taffy 直接解(±1px)。
#[test]
fn comp_cells_row_widget_solve_matches_direct_taffy_solve() {
    let dims = Dimensions::default();
    let (root, children) = size_row_css(dims);

    let (size, rects) = widget_solve(boxed(&root, &children));
    let (direct_size, direct_rects) = direct_taffy_solve(&root, &children);

    assert!(
        (size.width - direct_size.width).abs() <= 1.0
            && (size.height - direct_size.height).abs() <= 1.0,
        "root 寸法が taffy 直接解と ±1px を超えてズレている: widget={size:?} direct={direct_size:?}"
    );
    assert_rects_eq(&rects, &direct_rects);
}

/// (b) 手計算の画素: `Dimensions::default()` の実値どおりに行高・gap・
/// セル幅・左 padding が乗ること。
#[test]
fn comp_cells_row_lands_on_the_expected_hand_computed_pixels() {
    let dims = Dimensions::default();
    let (root, children) = size_row_css(dims);

    let (size, rects) = widget_solve(boxed(&root, &children));

    assert!(
        (size.height - dims.inspector_row_height).abs() <= 1.0,
        "行高が dims.inspector_row_height と ±1px を超えてズレている: {} vs {}",
        size.height,
        dims.inspector_row_height
    );

    // 3子: label(flex:1, 残り幅) / cell0(固定幅) / cell1(固定幅)。
    assert_eq!(rects.len(), 3, "label + 値セル2本のはずが子の数が違う");
    let (label, cell0, cell1) = (rects[0], rects[1], rects[2]);

    // 左 padding = spacing_m: label の左端がパネル左端から spacing_m だけ空く。
    assert!(
        (label.x - dims.spacing_m).abs() <= 1.0,
        "label の左 padding が spacing_m と ±1px を超えてズレている: {} vs {}",
        label.x,
        dims.spacing_m
    );

    // セル幅は inspector_value_width ちょうど。
    for (name, cell) in [("cell0", cell0), ("cell1", cell1)] {
        assert!(
            (cell.width - dims.inspector_value_width).abs() <= 1.0,
            "{name} の幅が inspector_value_width と ±1px を超えてズレている: {} vs {}",
            cell.width,
            dims.inspector_value_width
        );
    }

    // label→cell0、cell0→cell1 の gap は両方とも spacing_xs
    // (旧実装の「外側 spacing(xs) = 内側 row(cells).spacing(xs)」が同値
    // だったため、1段へフラット化しても gap は変わらないはず — 冒頭 doc 参照)。
    let gap_label_cell0 = cell0.x - (label.x + label.width);
    let gap_cell0_cell1 = cell1.x - (cell0.x + cell0.width);
    for (name, gap) in [
        ("label→cell0", gap_label_cell0),
        ("cell0→cell1", gap_cell0_cell1),
    ] {
        assert!(
            (gap - dims.spacing_xs).abs() <= 1.0,
            "{name} の gap が spacing_xs と ±1px を超えてズレている: {} vs {}",
            gap,
            dims.spacing_xs
        );
    }

    // 右端: cell1 の右端 + 右 padding(spacing_m)が root 幅と一致するはず。
    let right_edge = cell1.x + cell1.width + dims.spacing_m;
    assert!(
        (right_edge - size.width).abs() <= 1.0,
        "右 padding 込みの右端が root 幅と ±1px を超えてズレている: {right_edge} vs {}",
        size.width
    );
}
