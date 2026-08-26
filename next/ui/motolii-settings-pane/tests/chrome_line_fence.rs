//! 線化 D5 ORACLE(裁定179 文法1「区切りは明度1段+間隔、線で引かない」、
//! `docs/reviews/2026-08-22-chrome-grammar-audit.md`):
//! `chrome::button_style`(Settings のプリセット/トグルボタン)と
//! `chrome::panel_container_style`(Settings/Inspector の pane 容器)の常時輪郭を
//! 廃止する — 区別は面の明度段(`surface_raised`/`surface_panel` vs app 地)が
//! 担い、border は**透明のまま幅だけ残す**(幾何不変)。
//!
//! `chip_outline_fence.rs`(browser-pane、D4)と同型の「実装側を `use` した
//! style 関数レベルの両側チェック」。

use iced::widget::button;

use motolii_settings_pane::chrome::{button_style, panel_container_style};
use motolii_tokens_rs::{Colors, Dimensions};

const ALL_STATUSES: [button::Status; 4] = [
    button::Status::Active,
    button::Status::Hovered,
    button::Status::Pressed,
    button::Status::Disabled,
];

/// **本命(D5)**: ボタンの border は全 status で透明 — 幅は
/// `dims.theme().stroke.hairline` のまま(幾何不変: レイアウトに効く値を変えず、色だけ
/// 落とす — 透明 border 方式)。
#[test]
fn button_border_is_transparent_in_every_status_but_keeps_its_width() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    for status in ALL_STATUSES {
        let style = button_style(dims, colors, status);
        assert_eq!(
            style.border.color.a, 0.0,
            "button({status:?})の border が透明でない(常時輪郭 D5 が残存): {:?}",
            style.border.color
        );
        assert_eq!(
            style.border.width, dims.theme().stroke.hairline,
            "button({status:?})の border 幅が dims.theme().stroke.hairline から動いた(幾何不変違反)"
        );
    }
}

/// 明度段は残る(消しすぎ防止 — S6「区切りが見えなくなった」は不合格):
/// idle は `surface_raised` の面で panel 地から1段浮き、hover/pressed/disabled
/// の面も従来の意味色ロールのまま。
#[test]
fn button_faces_keep_the_one_step_luminance_grammar() {
    let dims = Dimensions::default();
    let colors = Colors::default();

    let idle = button_style(dims, colors, button::Status::Active);
    assert_eq!(
        idle.background,
        Some(iced::Background::Color(colors.surface_raised)),
        "idle の面が surface_raised でない(明度段が消えた)"
    );
    assert_eq!(idle.text_color, colors.text_primary);

    let hovered = button_style(dims, colors, button::Status::Hovered);
    assert_eq!(
        hovered.background,
        Some(iced::Background::Color(colors.surface_hover))
    );

    let pressed = button_style(dims, colors, button::Status::Pressed);
    assert_eq!(
        pressed.background,
        Some(iced::Background::Color(colors.state_selected))
    );

    let disabled = button_style(dims, colors, button::Status::Disabled);
    assert_eq!(disabled.text_color, colors.state_disabled);
}

/// **本命(D5)**: pane 容器は `surface_panel` の面+透明 border(幅は
/// `dims.theme().stroke.hairline` のまま=幾何不変)。容器の輪郭は app 地からの明度1段が
/// 担う(`panel_container_style` doc 参照)。
#[test]
fn panel_container_has_a_panel_face_and_a_transparent_border() {
    let dims = Dimensions::default();
    let colors = Colors::default();
    let style = panel_container_style(dims, colors);
    assert_eq!(
        style.background,
        Some(iced::Background::Color(colors.surface_panel)),
        "容器の面が surface_panel でない(明度段が消えた)"
    );
    assert_eq!(
        style.border.color.a, 0.0,
        "容器の border が透明でない(常時輪郭 D5 が残存): {:?}",
        style.border.color
    );
    assert_eq!(
        style.border.width, dims.theme().stroke.hairline,
        "容器の border 幅が dims.theme().stroke.hairline から動いた(幾何不変違反)"
    );
    assert_eq!(style.border.radius, 0.0.into(), "容器の角丸が動いた(幾何不変違反)");
}
