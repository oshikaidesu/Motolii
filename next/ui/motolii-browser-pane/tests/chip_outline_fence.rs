//! 線化 D4 ORACLE(裁定179「箱は状態の器」、`docs/reviews/
//! 2026-08-22-chrome-grammar-audit.md` D4): フィルタチップ(Video/Images/
//! Audio/Clear)と rail 項目の輪郭は**選択の器**としてのみ描く —
//! 非選択= 素の文字(地なし・輪郭なし)+ hover 面、選択= 現行の選択表現
//! (`state_selected` 地+`action_active` 縁/ink)を維持。
//!
//! `browser_ratio_ledger.rs` と同型の「実装側を `use` した両側チェック」を
//! style 関数レベル([`motolii_browser_pane::chip_style`] — rail 行/チップ/
//! Clear の3入口が共有する1本)で固定する。幾何不変(発注条件)は
//! 「border 幅は非選択でも `dims.border_width` のまま・色だけ透明」+
//! 「角丸は呼び出し側の値を素通し」で機械照合する。
//!
//! ## モック照合(2026-08-22 再確認)
//! - `.filterShelf button.selected{border-color:#d8b574; background:#3c382b;
//!   color:#e5cf9b}`(`browser-library.css:228`)— **選択状態の輪郭+琥珀地+
//!   accent ink は mock が宣言している** → 転写(現行表現の維持)。
//! - `.locationRow{border:0; border-left:2px solid transparent;
//!   background:transparent}`(css:135-152)— **rail 行の非選択は mock 自体が
//!   輪郭なし・地なし** → 転写。
//! - 非選択チップの常時 `border:1px solid border-default`(css:215-226)だけは
//!   裁定179(常時輪郭の廃止)が mock を上書きする — 沈黙部分の扱いは
//!   media/preview 両タブの既存文法(`tab_style`/`card_style`: idle 透明・
//!   hover `surface_hover`)に合わせる。

use iced::widget::button;

use motolii_browser_pane::{chip_style, FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO};
use motolii_tokens_rs::{Colors, Dimensions};

/// チップの角丸(呼び出し側 `filter_shelf_view` が渡す実値と同じ導出)。
fn chip_radius(dims: Dimensions) -> f32 {
    FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height
}

const ALL_STATUSES: [button::Status; 4] = [
    button::Status::Active,
    button::Status::Hovered,
    button::Status::Pressed,
    button::Status::Disabled,
];

/// **本命(D4)**: 非選択チップ/rail 行の border は全 status で透明 —
/// ただし幅は `dims.border_width` のまま(幾何不変: レイアウトに効く値を
/// 変えず、色だけ落とす — 発注書「透明 border で幅を保つ」)。
#[test]
fn unselected_border_is_transparent_in_every_status_but_keeps_its_width() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    for status in ALL_STATUSES {
        let style = chip_style(dims, colors, false, status, chip_radius(dims));
        assert_eq!(
            style.border.color.a, 0.0,
            "非選択({status:?})の border が透明でない(常時輪郭 D4 が残存): \
             {:?}",
            style.border.color
        );
        assert_eq!(
            style.border.width, dims.border_width,
            "非選択({status:?})の border 幅が dims.border_width から動いた\
             (幾何不変違反)"
        );
    }
}

/// **本命(選択の器)**: 選択中は border が不透明の `action_active`(mock
/// css:228 `border-color:#d8b574` の転写 — 現行の選択表現を維持)。
#[test]
fn selected_border_is_opaque_accent() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    for status in ALL_STATUSES {
        let style = chip_style(dims, colors, true, status, chip_radius(dims));
        assert_eq!(
            style.border.color, colors.action_active,
            "選択({status:?})の border が action_active でない"
        );
        assert!(
            style.border.color.a > 0.0,
            "選択({status:?})の border が透明(選択の器が消えた)"
        );
        assert_eq!(
            style.border.width, dims.border_width,
            "選択({status:?})の border 幅が dims.border_width でない"
        );
    }
}

/// 非選択 idle は素の文字 — 地なし(`background: None`)+ `text_primary`。
/// hover でだけ面(`surface_hover`)が浮く(裁定179「affordance は hover で
/// 現れる」、`tab_style`/`card_style`/transport と同じ既存文法)。
#[test]
fn unselected_is_bare_text_with_a_hover_face() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let radius = chip_radius(dims);

    let idle = chip_style(dims, colors, false, button::Status::Active, radius);
    assert!(
        idle.background.is_none(),
        "非選択 idle に面が乗っている(素の文字でない): {:?}",
        idle.background
    );
    assert_eq!(idle.text_color, colors.text_primary);

    let hovered = chip_style(dims, colors, false, button::Status::Hovered, radius);
    assert_eq!(
        hovered.background,
        Some(iced::Background::Color(colors.surface_hover)),
        "非選択 hover に surface_hover の面が出ない"
    );
}

/// 選択中の地と ink は現行表現の維持(mock css:228 の琥珀地+accent ink と
/// 同役の `state_selected`/`action_active`)。
#[test]
fn selected_face_and_ink_keep_the_current_selection_expression() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = chip_style(
        dims,
        colors,
        true,
        button::Status::Active,
        chip_radius(dims),
    );
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.state_selected))
    );
    assert_eq!(style.text_color, colors.action_active);
}

/// 幾何不変(発注条件): 角丸は呼び出し側の値(チップ= 0.4×行高、rail 行=
/// 0)を素通しする — 線化で形(比率台帳の既決値)を動かさない。
#[test]
fn radius_passes_through_unchanged_for_both_chip_and_rail_shapes() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    for (radius, which) in [(chip_radius(dims), "チップ"), (0.0, "rail 行")] {
        for selected in [false, true] {
            let style = chip_style(dims, colors, selected, button::Status::Active, radius);
            assert_eq!(
                style.border.radius,
                radius.into(),
                "{which}(selected={selected})の角丸が呼び出し値から動いた\
                 (幾何不変違反)"
            );
        }
    }
}
