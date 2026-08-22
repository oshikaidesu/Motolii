//! 発注の落ちるテスト (a)(b)(c)+grid: `TaffyBox` が解く矩形が
//! taffy 直接解(同じ style・同じ measure 意味論)と一致し、かつ手計算の
//! 画素値とも一致することの oracle。
//!
//! renderer は iced_core の null renderer(`()`)— レイアウトだけを検分するので
//! wgpu/tiny-skia を持ち込まない。子は `Space::new(Fill, Fill)`(与えられた
//! 上限いっぱいに解ける中身)で、寸法は全て taffy 側 style が決める。

use iced_core::layout::Limits;
use iced_core::widget::Tree;
use iced_core::{Element, Length, Rectangle, Size};
use iced_widget::Space;
use motolii_taffy::{style_from_css_decl, TaffyBox};

type Renderer = ();
type Theme = iced_core::Theme;
type Message = ();

const AVAIL_W: f32 = 800.0;
const AVAIL_H: f32 = 600.0;

/// 子(全て `Space::Fill`)つきの `TaffyBox` を組む。
fn boxed<'a>(root_css: &str, child_css: &[&str]) -> TaffyBox<'a, Message, Theme, Renderer> {
    let mut b = TaffyBox::new(style_from_css_decl(root_css).expect("root css"));
    for css in child_css {
        b = b.push(
            style_from_css_decl(css).expect("child css"),
            Space::new().width(Length::Fill).height(Length::Fill),
        );
    }
    b
}

/// widget 側の解: `Widget::layout` を直接呼び、root 相対の子矩形を返す。
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

/// taffy 直接解: 同じ style を widget を介さず taffy に渡す。measure は widget が
/// `Space::Fill` の子へ与える意味論(known → definite → 親上限、の順)を写す。
fn direct_taffy_solve(root_css: &str, child_css: &[&str]) -> (Size, Vec<Rectangle>) {
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
            taffy::Size {
                width: w,
                height: h,
            }
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
        assert_eq!(w, d, "子{i} の矩形が taffy 直接解と一致しない");
    }
}

/// (a) space-between: iced 素の Row では書けない語彙が、taffy 直接解と
/// 同一の矩形で解けている。
#[test]
fn space_between_matches_direct_taffy_and_hand_pixels() {
    let root = "display:flex; justify-content:space-between; width:640px; height:100px";
    let cell = "width:40px; height:20px";
    let children = [cell, cell, cell];

    let (size, rects) = widget_solve(boxed(root, &children));
    let (direct_size, direct_rects) = direct_taffy_solve(root, &children);

    assert_eq!(size, direct_size, "root 寸法が taffy 直接解と一致しない");
    assert_rects_eq(&rects, &direct_rects);

    // 手計算の画素: (640 - 3*40) / 2 = 260 の間隔 → x = 0, 300, 600。
    assert_eq!(size, Size::new(640.0, 100.0));
    let xs: Vec<f32> = rects.iter().map(|r| r.x).collect();
    assert_eq!(xs, vec![0.0, 300.0, 600.0], "space-between の x 配分");
    for r in &rects {
        assert_eq!((r.y, r.width, r.height), (0.0, 40.0, 20.0));
    }
}

/// (a') space-around も動く(語彙の面の確認 — 素の iced に無いもう1語)。
#[test]
fn space_around_matches_direct_taffy() {
    let root = "display:flex; justify-content:space-around; width:600px; height:50px";
    let cell = "width:100px; height:50px";
    let children = [cell, cell];

    let (_, rects) = widget_solve(boxed(root, &children));
    let (_, direct_rects) = direct_taffy_solve(root, &children);
    assert_rects_eq(&rects, &direct_rects);

    // 手計算: 余白 400 → 端 100・間 200 → x = 100, 400。
    let xs: Vec<f32> = rects.iter().map(|r| r.x).collect();
    assert_eq!(xs, vec![100.0, 400.0], "space-around の x 配分");
}

/// (b) gap / padding の画素一致。
#[test]
fn gap_and_padding_land_on_exact_pixels() {
    let root = "display:flex; flex-direction:column; gap:6px; padding:8px 12px; \
                width:200px; height:120px";
    let cell = "width:50px; height:10px";
    let children = [cell, cell];

    let (size, rects) = widget_solve(boxed(root, &children));
    let (direct_size, direct_rects) = direct_taffy_solve(root, &children);
    assert_eq!(size, direct_size);
    assert_rects_eq(&rects, &direct_rects);

    // 手計算: padding 上8 左12、gap 6 → (12,8) と (12, 8+10+6=24)。
    assert_eq!(rects[0], Rectangle { x: 12.0, y: 8.0, width: 50.0, height: 10.0 });
    assert_eq!(rects[1], Rectangle { x: 12.0, y: 24.0, width: 50.0, height: 10.0 });
}

/// (c) 入れ子 flex: TaffyBox の中の TaffyBox。外は `flex:1` で子が伸び、
/// 内は column+gap で並ぶ。
#[test]
fn nested_flex_boxes_resolve_inner_rects() {
    let inner_cell = "width:30px; height:10px";
    let inner: TaffyBox<'_, Message, Theme, Renderer> = boxed(
        "display:flex; flex-direction:column; gap:4px",
        &[inner_cell, inner_cell],
    );

    let outer = TaffyBox::new(
        style_from_css_decl("display:flex; padding:5px; width:200px; height:60px")
            .expect("outer css"),
    )
    .push(style_from_css_decl("flex:1").expect("flex:1"), inner);

    let mut element: Element<'_, Message, Theme, Renderer> = outer.into();
    let mut tree = Tree::new(element.as_widget());
    tree.diff(element.as_widget_mut());
    let node = element.as_widget_mut().layout(
        &mut tree,
        &(),
        &Limits::new(Size::ZERO, Size::new(AVAIL_W, AVAIL_H)),
    );

    // 外: padding 5 の内側で flex:1 が枠いっぱい(190x50)へ伸びる。
    assert_eq!(node.size(), Size::new(200.0, 60.0));
    let inner_node = &node.children()[0];
    assert_eq!(
        inner_node.bounds(),
        Rectangle { x: 5.0, y: 5.0, width: 190.0, height: 50.0 },
        "flex:1 の子が外の padding 内枠いっぱいに解けていない"
    );

    // 内(inner 相対): column+gap 4 → (0,0) と (0,14)。
    let inner_rects: Vec<Rectangle> =
        inner_node.children().iter().map(|c| c.bounds()).collect();
    assert_eq!(inner_rects[0], Rectangle { x: 0.0, y: 0.0, width: 30.0, height: 10.0 });
    assert_eq!(inner_rects[1], Rectangle { x: 0.0, y: 14.0, width: 30.0, height: 10.0 });
}

/// grid: 調査の旗艦例 `minmax(132px,1fr) repeat(3,64px) 26px`(Inspector mock の
/// 列定義)が字面のまま解ける — iced 素の Grid には書けない語彙。
#[test]
fn grid_template_with_minmax_and_repeat_resolves_columns() {
    let root = "display:grid; \
                grid-template-columns:minmax(132px,1fr) repeat(3,64px) 26px; \
                width:400px; height:40px";
    // 行方向は子の定義高で決める(auto 行の content 測定は `Space::Fill` 子だと
    // 「上限いっぱい」に化けるため、この oracle では高さを宣言で固定する)。
    let children = ["height:40px"; 5];

    let (size, rects) = widget_solve(boxed(root, &children));
    let (direct_size, direct_rects) = direct_taffy_solve(root, &children);
    assert_eq!(size, direct_size);
    assert_rects_eq(&rects, &direct_rects);

    // 手計算: 固定列 64*3+26=218、残り 182 ≥ 132 → 1fr 列 = 182。
    assert_eq!(size, Size::new(400.0, 40.0));
    let xs: Vec<f32> = rects.iter().map(|r| r.x).collect();
    let ws: Vec<f32> = rects.iter().map(|r| r.width).collect();
    assert_eq!(xs, vec![0.0, 182.0, 246.0, 310.0, 374.0], "grid 列の x");
    assert_eq!(ws, vec![182.0, 64.0, 64.0, 64.0, 26.0], "grid 列の幅");
    for r in &rects {
        assert_eq!((r.y, r.height), (0.0, 40.0), "行方向は stretch で 40");
    }
}

/// wrap: 素の iced Row に無いもう1語。折返し行の y が直接解と一致する。
#[test]
fn flex_wrap_matches_direct_taffy() {
    // 高さは auto — 折返した行の積みが container の高さになる(=content 高)。
    let root = "display:flex; flex-wrap:wrap; gap:10px; width:250px";
    let cell = "width:100px; height:30px";
    let children = [cell, cell, cell];

    let (size, rects) = widget_solve(boxed(root, &children));
    let (direct_size, direct_rects) = direct_taffy_solve(root, &children);
    assert_eq!(size, direct_size);
    assert_rects_eq(&rects, &direct_rects);
    assert_eq!(size, Size::new(250.0, 70.0), "行の積み = 30+10+30");

    // 手計算: 100+10+100=210 ≤ 250、3つ目は折返し → (0,0),(110,0),(0,40)。
    let pos: Vec<(f32, f32)> = rects.iter().map(|r| (r.x, r.y)).collect();
    assert_eq!(pos, vec![(0.0, 0.0), (110.0, 0.0), (0.0, 40.0)], "wrap の折返し位置");
}
