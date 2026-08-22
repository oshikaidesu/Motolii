//! create カード(mouse_area 経路、B36)の意匠 ORACLE —
//! `chip_outline_fence.rs`/`drop_target_fence.rs` と同型の style 関数レベル
//! 照合。
//!
//! create カードは button でなく `mouse_area`+container(ダブルクリックの
//! ため — crate 冒頭 doc 参照)なので、button 経路の `card_style` と**状態
//! 文法が同じ**であることを機械照合する: 選択=`state_selected` 地 / hover=
//! `surface_hover` 地 / 通常=透明地(裁定179「箱は状態の器」— 常時輪郭
//! なし)。2経路の意匠が食い違うと「同じカードの顔が場所で変わる」欠陥に
//! なるための柵。
//!
//! **テストは書くが実行しない**(裁定189 追いつきターンの規律)。

use motolii_browser_pane::create_card_face;
use motolii_tokens_rs::Colors;

/// 選択カードは `state_selected` の地(button 経路の selected と同役)。
#[test]
fn a_selected_create_card_has_the_selected_face() {
    let colors = Colors::default();
    let style = create_card_face(colors, true, false);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.state_selected)),
        "選択 create カードの地が state_selected でない"
    );
}

/// hover は `surface_hover` の面(button 経路の `Status::Hovered` と同役 —
/// Q0: 触れそうで触れない物は不合格、hover 無反応にしない)。
#[test]
fn a_hovered_create_card_has_the_hover_face() {
    let colors = Colors::default();
    let style = create_card_face(colors, false, true);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_hover)),
        "hover create カードの地が surface_hover でない"
    );
}

/// 通常状態は透明地+輪郭なし(裁定179「箱は状態の器」— 常時輪郭を
/// 持ち込まない)。選択が hover に勝つ(両立時は選択の地)。
#[test]
fn an_idle_create_card_is_bare_and_selection_wins_over_hover() {
    let colors = Colors::default();
    let idle = create_card_face(colors, false, false);
    assert_eq!(idle.background, None, "通常 create カードに地が付いている");
    assert_eq!(idle.border.width, 0.0, "通常 create カードに輪郭が付いている");

    let both = create_card_face(colors, true, true);
    assert_eq!(
        both.background,
        Some(iced::Background::Color(colors.state_selected)),
        "選択+hover 両立時に選択の地が勝たない"
    );
}
