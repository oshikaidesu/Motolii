//! 発注(INS-taffy の要求): `TaffyBox` に丸め無効化の口があることの oracle。
//!
//! taffy 0.13 は解いた矩形を既定で最近偶数丸めにより整数 px へ丸める
//! (`TaffyTree::disable_rounding()` を呼ばない限り)。`inspector_row_height=25`
//! を 150% した 37.5 のような半端値は、丸め有効だと 38.0 に化ける — これが
//! 既存の ±1px 柵(`inspector_pixel_fence`、EPS=0.05)と衝突する本丸の理由。
//!
//! この oracle は:
//! (a) 既定(`TaffyBox::new`)= 丸め有効 → 37.5 が 38.0 に丸まる。
//! (b) `.rounding(false)` / `TaffyBox::unrounded` = 丸め無効 → 37.5 が
//!     端数のまま届く。
//! (c) 丸め有無を切り替えても、丸めが関与しない整数 px の値(既存 19 テストの
//!     対象)は不変であること — 非退行の直接確認。

use iced_core::layout::Limits;
use iced_core::widget::Tree;
use iced_core::{Element, Length, Rectangle, Size};
use iced_widget::Space;
use motolii_taffy::TaffyBox;

type Renderer = ();
type Theme = iced_core::Theme;
type Message = ();

const AVAIL_W: f32 = 800.0;
const AVAIL_H: f32 = 600.0;

/// 半端値(37.5px)を持つ 1 child の root style。
fn root_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        size: taffy::Size {
            width: taffy::Dimension::length(100.0),
            height: taffy::Dimension::length(100.0),
        },
        ..Default::default()
    }
}

fn child_style() -> taffy::Style {
    taffy::Style {
        size: taffy::Size {
            width: taffy::Dimension::length(40.0),
            height: taffy::Dimension::length(37.5),
        },
        flex_shrink: 0.0,
        ..Default::default()
    }
}

fn solve<'a>(
    root: TaffyBox<'a, Message, Theme, Renderer>,
) -> (Size, Vec<Rectangle>) {
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

fn fill_child() -> Element<'static, Message, Theme, Renderer> {
    Space::new().width(Length::Fill).height(Length::Fill).into()
}

/// (a) 既定(丸め有効): 37.5 は最近偶数丸めで 38.0 に化ける。
#[test]
fn default_rounding_snaps_half_pixel_to_integer() {
    let root = TaffyBox::new(root_style()).push(child_style(), fill_child());
    let (_, rects) = solve(root);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].height, 38.0, "丸め有効の既定では 37.5 → 38.0(最近偶数)");
}

/// (b) `.rounding(false)`: 37.5 は端数のまま届く。
#[test]
fn rounding_false_preserves_fractional_pixel() {
    let root = TaffyBox::new(root_style())
        .rounding(false)
        .push(child_style(), fill_child());
    let (_, rects) = solve(root);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].height, 37.5, "丸め無効では 37.5 が端数のまま");
}

/// (b') `TaffyBox::unrounded` は `.rounding(false)` と同じ結果になる糖衣。
#[test]
fn unrounded_constructor_matches_rounding_false() {
    let root = TaffyBox::unrounded(root_style()).push(child_style(), fill_child());
    let (_, rects) = solve(root);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].height, 37.5, "unrounded() も端数のまま");
}

/// (c) 丸めが関与しない整数 px の値は、丸め有無を切り替えても不変
/// (既存 layout_oracle 群の非退行を、同じ oracle 形で確認)。
#[test]
fn integer_pixel_layout_is_unaffected_by_rounding_toggle() {
    let root_css = taffy::Style {
        display: taffy::Display::Flex,
        justify_content: Some(taffy::JustifyContent::SPACE_BETWEEN),
        size: taffy::Size {
            width: taffy::Dimension::length(640.0),
            height: taffy::Dimension::length(100.0),
        },
        ..Default::default()
    };
    let cell = || taffy::Style {
        size: taffy::Size {
            width: taffy::Dimension::length(40.0),
            height: taffy::Dimension::length(20.0),
        },
        ..Default::default()
    };

    let rounded = TaffyBox::new(root_css.clone())
        .push(cell(), fill_child())
        .push(cell(), fill_child())
        .push(cell(), fill_child());
    let unrounded = TaffyBox::unrounded(root_css)
        .push(cell(), fill_child())
        .push(cell(), fill_child())
        .push(cell(), fill_child());

    let (rounded_size, rounded_rects) = solve(rounded);
    let (unrounded_size, unrounded_rects) = solve(unrounded);

    assert_eq!(rounded_size, unrounded_size);
    assert_eq!(rounded_rects, unrounded_rects);
    let xs: Vec<f32> = rounded_rects.iter().map(|r| r.x).collect();
    assert_eq!(xs, vec![0.0, 300.0, 600.0], "space-between の x 配分(既存 oracle 同値)");
}
