//! 主要 widget の style ヘルパ — 全部 [`Tokens`] を読むだけ。
//!
//! iced の widget は `.style(fn)` に `(&Theme, Status) -> Style` を受けるので、
//! ここのヘルパはそのまま関数名で渡せる。色の出所は [`Tokens`](super::Tokens) の
//! field doc(= 生成 CSS の行)に一元化してあり、ここに hex は1つも無い。
//!
//! 形の方針は Ableton 風フラット(2026-07-14 裁定 / UI視覚言語「面」):
//! 階層は小さな明度差と境界線で作る。丸角・shadow・gradient を足さない。

use iced::widget::{button, container, rule, text};
use iced::{Background, Border, Theme};

use super::Tokens;

/// 押せるもの — スタート画面の New / Open、帯の Export / Cancel。
///
/// - 平常: `surface_raised` の面 + `border_default` の 1px 線(触れそうに見える)
/// - hover / 押下中: `surface_hover`(マウス完結・無反応ゼロ、2026-08-13 裁定)
/// - disabled: 面を `surface_panel` へ落とし文字を `text_muted` に —
///   「触れそうで触れない物」に見せない(2026-08-12 の Q0)
pub fn action(theme: &Theme, status: button::Status) -> button::Style {
    let t = Tokens::resolve(theme);
    let (background, text_color) = match status {
        button::Status::Active => (t.surface_raised, t.text_primary),
        button::Status::Hovered | button::Status::Pressed => (t.surface_hover, t.text_primary),
        button::Status::Disabled => (t.surface_panel, t.text_muted),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border: Border {
            color: t.border_default,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// status 帯の面 — `surface_panel`。土台(`surface_app`)との段差は
/// 明度差 + 上辺の [`separator`] で作る。
pub fn status_band(theme: &Theme) -> container::Style {
    let t = Tokens::resolve(theme);
    container::Style {
        background: Some(Background::Color(t.surface_panel)),
        ..container::Style::default()
    }
}

/// 領域の区切り線 — `border_default` の 1px。帯の上辺に使う。
pub fn separator(theme: &Theme) -> rule::Style {
    let t = Tokens::resolve(theme);
    rule::Style {
        color: t.border_default,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

/// 補助の文字 — tagline や「いま何が無いか」の一行(`text_secondary`)。
pub fn text_secondary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Tokens::resolve(theme).text_secondary),
    }
}

/// 添え物の文字 — 近道キーの表示など(`text_muted`)。
pub fn text_muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Tokens::resolve(theme).text_muted),
    }
}
