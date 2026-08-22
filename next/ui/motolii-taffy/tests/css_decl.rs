//! 発注の落ちるテスト (d): CSS 宣言 parser の正例/負例。
//! 負例の本丸は fail-closed — 対応外の宣言が Err になり、黙って無視されないこと。

use motolii_taffy::{apply_css_decl, style_from_css_decl, CssDeclError};
use taffy::{AlignItems, Dimension, Display, FlexDirection, FlexWrap, JustifyContent, LengthPercentage};

// --- 正例 -----------------------------------------------------------------

#[test]
fn flex_container_vocabulary_maps_to_taffy_fields() {
    let style = style_from_css_decl(
        "display:flex; flex-direction:column; flex-wrap:wrap; \
         justify-content:space-between; align-items:center; gap:4px",
    )
    .expect("対応内の宣言のみ");

    assert_eq!(style.display, Display::Flex);
    assert_eq!(style.flex_direction, FlexDirection::Column);
    assert_eq!(style.flex_wrap, FlexWrap::Wrap);
    assert_eq!(style.justify_content, Some(JustifyContent::SPACE_BETWEEN));
    assert_eq!(style.align_items, Some(AlignItems::CENTER));
    assert_eq!(style.gap.width, LengthPercentage::length(4.0));
    assert_eq!(style.gap.height, LengthPercentage::length(4.0));
}

#[test]
fn two_value_gap_is_row_then_column() {
    let style = style_from_css_decl("gap:8px 2px").expect("gap 2値");
    // CSS: `gap: <row-gap> <column-gap>`。taffy: width=column, height=row。
    assert_eq!(style.gap.height, LengthPercentage::length(8.0));
    assert_eq!(style.gap.width, LengthPercentage::length(2.0));
}

#[test]
fn padding_shorthands_expand_like_css() {
    let s1 = style_from_css_decl("padding:8px").unwrap();
    assert_eq!(s1.padding.top, LengthPercentage::length(8.0));
    assert_eq!(s1.padding.right, LengthPercentage::length(8.0));
    assert_eq!(s1.padding.bottom, LengthPercentage::length(8.0));
    assert_eq!(s1.padding.left, LengthPercentage::length(8.0));

    let s2 = style_from_css_decl("padding:8px 12px").unwrap();
    assert_eq!(s2.padding.top, LengthPercentage::length(8.0));
    assert_eq!(s2.padding.right, LengthPercentage::length(12.0));
    assert_eq!(s2.padding.bottom, LengthPercentage::length(8.0));
    assert_eq!(s2.padding.left, LengthPercentage::length(12.0));

    let s4 = style_from_css_decl("padding:1px 2px 3px 4px").unwrap();
    assert_eq!(s4.padding.top, LengthPercentage::length(1.0));
    assert_eq!(s4.padding.right, LengthPercentage::length(2.0));
    assert_eq!(s4.padding.bottom, LengthPercentage::length(3.0));
    assert_eq!(s4.padding.left, LengthPercentage::length(4.0));
}

#[test]
fn sizes_accept_px_percent_auto_and_bare_zero() {
    let style = style_from_css_decl(
        "width:50%; height:auto; min-width:132px; min-height:0; max-width:100%; max-height:80px",
    )
    .unwrap();
    assert_eq!(style.size.width, Dimension::percent(0.5));
    assert_eq!(style.size.height, Dimension::auto());
    assert_eq!(style.min_size.width, Dimension::length(132.0));
    assert_eq!(style.min_size.height, Dimension::length(0.0));
    assert_eq!(style.max_size.width, Dimension::percent(1.0));
    assert_eq!(style.max_size.height, Dimension::length(80.0));
}

#[test]
fn flex_shorthand_follows_css_defaults() {
    let one = style_from_css_decl("flex:1").unwrap();
    assert_eq!((one.flex_grow, one.flex_shrink), (1.0, 1.0));
    assert_eq!(one.flex_basis, Dimension::percent(0.0));

    let none = style_from_css_decl("flex:none").unwrap();
    assert_eq!((none.flex_grow, none.flex_shrink), (0.0, 0.0));
    assert_eq!(none.flex_basis, Dimension::auto());

    let auto = style_from_css_decl("flex:auto").unwrap();
    assert_eq!((auto.flex_grow, auto.flex_shrink), (1.0, 1.0));
    assert_eq!(auto.flex_basis, Dimension::auto());

    let full = style_from_css_decl("flex:2 3 26px").unwrap();
    assert_eq!((full.flex_grow, full.flex_shrink), (2.0, 3.0));
    assert_eq!(full.flex_basis, Dimension::length(26.0));

    let grow_basis = style_from_css_decl("flex:1 30%").unwrap();
    assert_eq!((grow_basis.flex_grow, grow_basis.flex_shrink), (1.0, 1.0));
    assert_eq!(grow_basis.flex_basis, Dimension::percent(0.3));

    let longhand = style_from_css_decl("flex-grow:2; flex-shrink:0; flex-basis:auto").unwrap();
    assert_eq!((longhand.flex_grow, longhand.flex_shrink), (2.0, 0.0));
    assert_eq!(longhand.flex_basis, Dimension::auto());
}

#[test]
fn grid_template_splits_only_outside_parens() {
    let style = style_from_css_decl(
        "display:grid; grid-template-columns:minmax(132px,1fr) repeat(3, 64px) 26px",
    )
    .expect("旗艦例(Inspector mock の列定義)");
    assert_eq!(style.display, Display::Grid);
    assert_eq!(
        style.grid_template_columns.len(),
        3,
        "repeat(3, 64px) 内の空白・カンマで割れてはいけない"
    );
    // 各 track の中身の等価は taffy 自身の parser を正とする。
    let expected: Vec<taffy::GridTemplateComponent<String>> = vec![
        "minmax(132px,1fr)".parse().unwrap(),
        "repeat(3, 64px)".parse().unwrap(),
        "26px".parse().unwrap(),
    ];
    assert_eq!(style.grid_template_columns, expected);
}

#[test]
fn apply_css_decl_layers_on_top_of_a_base_style() {
    let mut style = style_from_css_decl("display:flex; gap:4px").unwrap();
    apply_css_decl(&mut style, "gap:2px; align-items:center").expect("上書き");
    assert_eq!(style.display, Display::Flex, "触っていない field は保存");
    assert_eq!(style.gap.width, LengthPercentage::length(2.0), "後勝ち");
    assert_eq!(style.align_items, Some(AlignItems::CENTER));
}

#[test]
fn empty_and_whitespace_declarations_are_the_default_style() {
    assert_eq!(style_from_css_decl("").unwrap(), taffy::Style::default());
    assert_eq!(style_from_css_decl(" ; ; ").unwrap(), taffy::Style::default());
}

// --- 負例(fail-closed)---------------------------------------------------

#[test]
fn unsupported_properties_are_an_error_not_silence() {
    for decl in [
        "position:absolute",
        "color:red",
        "margin:4px",
        "display:flex; float:left",
        "row-gap:4px",
        "align-content:center",
    ] {
        let err = style_from_css_decl(decl).expect_err(decl);
        assert!(
            matches!(err, CssDeclError::UnsupportedProperty { .. }),
            "{decl:?} は UnsupportedProperty で落ちるべき: {err}"
        );
    }
}

#[test]
fn bad_values_are_an_error() {
    for decl in [
        "width:red",
        "width:4em",
        "width:calc(100% - 26px)",
        "gap:1px 2px 3px",
        "padding:1px 2px 3px 4px 5px",
        "flex-grow:-1",
        "flex-grow:two",
        "justify-content:sideways",
        "flex:1 2 3 4",
        "grid-template-columns:minmax(132px)",
    ] {
        let err = style_from_css_decl(decl).expect_err(decl);
        assert!(
            matches!(err, CssDeclError::InvalidValue { .. }),
            "{decl:?} は InvalidValue で落ちるべき: {err}"
        );
    }
}

#[test]
fn malformed_fragments_are_an_error() {
    for decl in ["display flex", "width", "display:flex; :4px", "gap:"] {
        let err = style_from_css_decl(decl).expect_err(decl);
        assert!(
            matches!(err, CssDeclError::MalformedDeclaration { .. }),
            "{decl:?} は MalformedDeclaration で落ちるべき: {err}"
        );
    }
}
