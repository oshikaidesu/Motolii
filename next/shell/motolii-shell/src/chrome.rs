//! pane 横断で共有されるスタイル/描画ヘルパ(裁定160 切片5、pane split survey
//! `docs/reviews/2026-08-21-pane-split-survey.md` §2.4/§6)。
//!
//! `settings_pane` は `inspector_pane::{section_header, value_input_style,
//! parse_number}` と `crate::button_style`(旧 `lib.rs` 定義)を再利用していた
//! ——いずれも `tokens::{Colors, Dimensions}`(または生の `&str`)だけを読む
//! 純関数(`Session`/`Document` 非依存)で、pane 間の import 障壁の実体だった
//! (survey §2.4)。ここへ吸い上げることで `settings_pane → inspector_pane` の
//! import をゼロにする。
//!
//! **純粋な再配置・挙動ゼロ変更**: 以下4関数はいずれも移設元から本体を無改変で
//! 移した(シグネチャ・実装とも1文字も変えていない)。crate 化(survey §5 の
//! `motolii-shell-chrome`)は後続切片の仕事——今回はモジュール止まり。

use iced::widget::{button, container, text, text_input};
use iced::{Element, Length};

use crate::tokens::{Colors, Dimensions, Ink};
use crate::Message;

/// header の3ボタン共通スタイル。**意味色ロール経由**(raw 値の直書き禁止) —
/// hover/pressed/disabled をそれぞれ別ロールで塗り分ける(状態: hover・選択・無効)。
/// `pub(crate)`: `settings_pane` のプリセット/市松トグルボタンも同じ意味色
/// ロールを使う — 状態ごとに専用の色を新設しない。
pub(crate) fn button_style(dims: Dimensions, colors: Colors, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => colors.surface_hover,
        button::Status::Pressed => colors.state_selected,
        button::Status::Disabled => colors.surface_panel,
        button::Status::Active => colors.surface_raised,
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else {
        colors.text_primary
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}

/// `pub(crate)`: `settings_pane` も同じ見出し帯(パネルタイトル/section 見出し
/// 共通トークン)を再利用する — 2箇所で別の意匠を発明しない。
pub(crate) fn section_header(
    label: &'static str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    // `.width(Length::Fill)`: `header` と同じ理由(柵で発見) — mock の `.sec` も
    // block 要素で pane 全幅の帯(実測: 修正前は幅 65〜68px)。
    //
    // **背景は塗らない**(裁定137/139、2026-08-21 更正): mock `.sec` は
    // `background`/`border` のどちらも持たない — 見出しは letter-spacing +
    // ink3(`text_muted`)+ 行高だけで区別する(旧実装は `surface_app` で塗って
    // 「面色の塗り分けで区切る」を犯していた — TRANSFORM/APPEARANCE/ATTRS の
    // 帯が周囲の `.prow` 行と違う沈んだ色の箱に見えていたのが実体)。
    container(
        text(label)
            .size(dims.caption_text)
            .color(Ink::Muted.resolve(&colors)),
    )
    .width(Length::Fill)
    .height(Length::Fixed(dims.inspector_section_header_height))
    .padding([0.0, dims.spacing_m])
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// `pub(crate)`: `settings_pane` の数値欄(背景RGBA・ui_scale%)も同じ枠色
/// ロールを使う — 2箇所で別の意匠を発明しない。
pub(crate) fn value_input_style(
    dims: Dimensions,
    colors: Colors,
    status: text_input::Status,
) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => colors.action_active,
        _ => colors.border_default,
    };
    text_input::Style {
        background: iced::Background::Color(colors.surface_app),
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        icon: colors.text_muted,
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

/// 入力文字列 → 数値。mock(`inspector-library.html`)は負号に `−`(U+2212)を使うので
/// 両対応する。`settings_pane`(背景RGBA・ui_scale%)も同じパーサを使う
/// (2箇所で別の意匠を発明しない、裁定160 切片5で `inspector_pane` から移設)。
pub(crate) fn parse_number(text: &str) -> Option<f64> {
    text.trim().replace('\u{2212}', "-").parse::<f64>().ok()
}
