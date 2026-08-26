//! 検索欄+rail/filter チップ共通ボタン(SP-6 分割: 元 `lib.rs` から移送)。

use crate::Message;
use iced::widget::{button, text, text_input};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

/// 検索欄(mock `#library-search` — mock では toolbar 領域=**全タブ共有**
/// なので、media の filter shelf と preview タブの両方がこの1本を使う)。
///
/// 裁定170 M01: fork(0.15.0-dev)の `text_input()` は `&str`/`&String` を
/// `Fragment::Borrowed` として受け、返り値のライフタイムを入力の借用に
/// 縛る(`settings_pane::channel_cell`/`ui_scale_row` と同じ実測済みの
/// 事情、両方の doc comment 参照)。呼び手のシグネチャは `Element<'static,
/// _>` を返す必要があるため、owned のまま move する。
pub(crate) fn search_field(query: &str, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let query_owned = query.to_owned();
    text_input("Search files and tags", query_owned)
        .on_input(Message::QueryChanged)
        .size(dims.theme().text.micro)
        .width(Length::FillPortion(2))
        .style(move |_theme, status| search_input_style(dims, colors, status))
        .into()
}

/// rail 行/filter チップ、共通のボタン(選択状態を1箇所で塗り分ける —
/// 2つの入口が同じ意匠を共有する、Ableton可視性原理どおり。media の
/// `RailScope` と非 media の `PreviewScope` の両語彙が同じ1本を使う —
/// 構造の対称化はこの関数の共有が実体)。**台帳(1a節)**: 文字は mock
/// 実測でどちらも8px — `micro_text`(旧 `caption_text`=9 は自前判断だった)。
/// `padding`/`radius` は呼び出し側(rail 行 vs filter チップ)で異なる mock
/// 実測値を渡す([`rail_view`]/[`filter_shelf_view`] の doc 参照 — 色/選択
/// 状態の意匠だけを共有し、形(padding・角丸)は mock どおり呼び出し側で
/// 分ける)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn labeled_button(
    label: &'static str,
    selected: bool,
    message: Message,
    width: Length,
    padding: [f32; 2],
    radius: f32,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(label).size(dims.theme().text.micro))
        .on_press(message)
        .width(width)
        .padding(padding)
        .style(move |_theme, status| chip_style(dims, colors, selected, status, radius))
        .into()
}

/// [`labeled_button`]/Clear ボタン共通のスタイル(裁定179「箱は状態の器」、
/// chrome 監査 D4 の線化 — `docs/reviews/2026-08-22-chrome-grammar-audit.md`)。
/// 輪郭は**選択の器**としてのみ描く:
/// - 非選択= 素の文字(地なし)+ hover 面(`surface_hover`) —
///   `tab_style`/[`card_style`]/transport と同じ既存文法。border は色だけ
///   透明にし、幅は `dims.theme().stroke.hairline` のまま(幾何不変 — レイアウトに
///   効く値を動かさない)。mock が非選択チップへ宣言する常時
///   `border-default` 輪郭(`browser-library.css:215-226`)はこの裁定が
///   上書き(rail 行 `.locationRow` は mock 自体が非選択透明、css:135-152)。
/// - 選択= 現行表現の維持: `state_selected` 地+`action_active` 縁/ink
///   (mock `.filterShelf button.selected{border-color:#d8b574}` css:228 の
///   宣言どおり — 選択状態の輪郭は mock が明示する部分)。
///
/// `radius` は呼び出し側の mock 実測値
/// (Browser component token/rail の `0.0`)を素通し。
/// **pub**: `tests/chip_outline_fence.rs` が「非選択= border 透明・選択=
/// 不透明」を style 関数レベルで固定する(`browser_ratio_ledger.rs` と同型の
/// 両側チェック)。
pub fn chip_style(
    dims: Dimensions,
    colors: Colors,
    selected: bool,
    status: button::Status,
    radius: f32,
) -> button::Style {
    let background = if selected {
        Some(colors.state_selected)
    } else {
        match status {
            button::Status::Hovered => Some(colors.surface_hover),
            button::Status::Pressed => Some(colors.state_selected),
            button::Status::Disabled | button::Status::Active => None,
        }
    };
    let border_color = if selected {
        colors.action_active
    } else {
        iced::Color::TRANSPARENT
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else if selected {
        colors.action_active
    } else {
        colors.text_primary
    };
    button::Style {
        background: background.map(iced::Background::Color),
        text_color,
        border: iced::Border {
            color: border_color,
            width: dims.theme().stroke.hairline,
            radius: radius.into(),
        },
        ..button::Style::default()
    }
}

/// 検索欄の枠色ロール(`settings_pane::chrome::value_input_style` と同じ
/// 「focus は accent、それ以外は既定枠」の形 — 別 crate なのでここに小さく
/// 複製する、pane crate 間の相互依存を作らないため)。
fn search_input_style(
    dims: Dimensions,
    colors: Colors,
    status: text_input::Status,
) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => colors.action_active,
        _ => colors.border_default,
    };
    text_input::Style {
        background: iced::Background::Color(colors.surface_raised),
        border: iced::Border {
            color: border_color,
            width: dims.theme().stroke.hairline,
            radius: 0.0.into(),
        },
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}
