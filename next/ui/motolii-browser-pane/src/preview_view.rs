//! preview-local カタログ(SP-6 分割: 元 `lib.rs` から移送 — effects/create/
//! panels タブの body・カード)。

use crate::card_view::{card_body, card_frame_width, card_style, columns_for};
use crate::filter_view::preview_filter_shelf_view;
use crate::model::{self, LibraryTab, PreviewScope};
use crate::rail_view::{catalog_container, preview_rail_view};
use crate::{CardSelectionModifiers, Message};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

// ---------------------------------------------------------------------------
// preview-local カタログ(effects/create/panels タブ、mock `data-tab` カード
// の転写 — 構造の対称化で media と同じ rail+shelf+catalog の3面構成)。
// ---------------------------------------------------------------------------

/// effects/create/panels タブの body(rail + カタログ — [`media_body`] と
/// 同じ骨格、構造の対称化)。rail は [`preview_rail_view`]、カタログは
/// [`preview_catalog_view`]。データは preview-local 静的カタログのみ
/// ([`model::preview_visible`] — 台帳 items は渡らない、`model::catalog`
/// doc の2系統の壁)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn preview_body(
    tab: LibraryTab,
    scope: PreviewScope,
    query: &str,
    view_mode: model::ViewMode,
    selected: Option<model::CardKey>,
    hovered: Option<model::CardKey>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let rail = preview_rail_view(tab, scope, dims, colors);
    let catalog = preview_catalog_view(
        tab, scope, query, view_mode, selected, hovered, dims, colors,
    );

    row![rail, catalog].spacing(dims.spacing_xs).into()
}

/// modifier 付きカード選択を使う preview body。`selected_cards` は pane state
/// から借りた表示用の集合、`CardSelectionModifiers` は Shell WIRE が後から
/// 注入する正規化済み入力であり、preview 側で OS イベントを読むことはない。
#[allow(clippy::too_many_arguments)]
pub(crate) fn preview_body_with_selection(
    tab: LibraryTab,
    scope: PreviewScope,
    query: &str,
    view_mode: model::ViewMode,
    selected_cards: &[model::CardKey],
    modifiers: CardSelectionModifiers,
    hovered: Option<model::CardKey>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let rail = preview_rail_view(tab, scope, dims, colors);
    let catalog = preview_catalog_view_impl(
        tab,
        scope,
        query,
        view_mode,
        CardSelectionMode::Modifiers {
            selected_cards,
            modifiers,
        },
        hovered,
        dims,
        colors,
    );

    row![rail, catalog].spacing(dims.spacing_xs).into()
}

/// 非 media タブの catalog(filter shelf + 結果件数 + カード grid —
/// [`catalog_view`] と同じ骨格を preview-local データで組む)。空状態の文言は
/// media の「絞り込みで0件」と同じ「No matches」(B08 続編の文言整理 —
/// preview カタログは静的なので「台帳が空」の面は存在しない)。
#[allow(clippy::too_many_arguments)]
fn preview_catalog_view(
    tab: LibraryTab,
    scope: PreviewScope,
    query: &str,
    view_mode: model::ViewMode,
    selected: Option<model::CardKey>,
    hovered: Option<model::CardKey>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    preview_catalog_view_impl(
        tab,
        scope,
        query,
        view_mode,
        CardSelectionMode::Legacy(selected),
        hovered,
        dims,
        colors,
    )
}

#[derive(Clone, Copy)]
enum CardSelectionMode<'a> {
    Legacy(Option<model::CardKey>),
    Modifiers {
        selected_cards: &'a [model::CardKey],
        modifiers: CardSelectionModifiers,
    },
}

impl CardSelectionMode<'_> {
    fn is_selected(self, key: model::CardKey) -> bool {
        match self {
            Self::Legacy(selected) => selected == Some(key),
            Self::Modifiers { selected_cards, .. } => selected_cards.contains(&key),
        }
    }

    fn message(self, key: model::CardKey, visible_cards: &[model::CardKey]) -> Message {
        match self {
            Self::Legacy(_) => Message::SelectCard(key),
            Self::Modifiers { modifiers, .. } => Message::SelectCardWithModifiers {
                key,
                modifiers,
                visible_cards: visible_cards.to_vec(),
            },
        }
    }
}

/// preview-local カタログの共通実装。`cards_data` の宣言順をそのまま可視列
/// として各カードへ渡す。Legacy は旧 `SelectCard`、Modifiers は新しい
/// `SelectCardWithModifiers` を発行するだけで、描画側が選択状態を所有しない。
#[allow(clippy::too_many_arguments)]
fn preview_catalog_view_impl(
    tab: LibraryTab,
    scope: PreviewScope,
    query: &str,
    view_mode: model::ViewMode,
    selection: CardSelectionMode<'_>,
    hovered: Option<model::CardKey>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let cards_data = model::preview_visible(tab, scope, query);

    let shelf = preview_filter_shelf_view(tab, scope, query, view_mode, dims, colors);

    let summary = text(format!("Results {}", cards_data.len()))
        .size(dims.micro_text)
        .color(colors.text_muted);

    let grid: Element<'static, Message> = if cards_data.is_empty() {
        container(
            text("No matches")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding(dims.spacing_m)
        .into()
    } else {
        let visible_cards: Vec<model::CardKey> = cards_data
            .iter()
            .map(|card| model::CardKey::Preview(card.id))
            .collect();
        let rows: Vec<Element<'static, Message>> = cards_data
            .chunks(columns_for(view_mode))
            .map(|chunk| {
                let cards: Vec<Element<'static, Message>> = chunk
                    .iter()
                    .map(|&card| {
                        let key = model::CardKey::Preview(card.id);
                        preview_card_view(
                            card,
                            selection.is_selected(key),
                            hovered == Some(key),
                            view_mode,
                            dims,
                            colors,
                            selection.message(key, &visible_cards),
                        )
                    })
                    .collect();
                row(cards).spacing(dims.spacing_s).into()
            })
            .collect();
        scrollable(column(rows).spacing(dims.spacing_s))
            .height(Length::Fill)
            .into()
    };

    catalog_container(column![shelf, summary, grid], false, dims, colors)
}

/// preview-local カタログの1枚(mock `.libraryCard` と同じ thumb +
/// `.cardCopy` 骨格 — 本体は [`card_body`] を [`card_view`] と共有する。
/// 寸法・文字は media カードと同一(比率台帳3節の既決値をそのまま共有)。
/// thumb の塗りは `surface_raised` 一律 — mock の装飾色(`.thumb-magenta` 等の
/// 直書き hex)は tokens に対応ロールが無く、S4「新ロールを起こさない」を
/// 優先して転写しない(逸脱として RETURN 記載)。
///
/// **B36**: `creates: Some` のカード(create タブ)は button でなく
/// `mouse_area` 経路 — シングル(press)=選択・**ダブルクリック=作成**
/// ([`Message::CreateFromCard`])。button は press を capture して
/// `on_double_click` へ届かないための乗り換えで、hover 意匠は
/// [`Message::CardHovered`]/[`Message::CardUnhovered`] の pane-local 状態が
/// 担う(crate 冒頭 doc 参照)。cursor は `Interaction::Pointer`(mock
/// `.libraryCard{cursor:pointer}` — button 経路と同じ手触り、文法§5.5)。
fn preview_card_view(
    card: model::PreviewCard,
    selected: bool,
    hovered: bool,
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
    select_message: Message,
) -> Element<'static, Message> {
    let key = model::CardKey::Preview(card.id);
    let body = card_body(
        card.glyph,
        colors.surface_raised,
        card.name.to_owned(),
        card.caption.to_owned(),
        view_mode,
        dims,
        colors,
        // preview カタログは静的カタログ由来(`model::PreviewCard`)で
        // `AssetStatus` を持たない — 状態バッジは media カードのみ。
        None,
    );
    let frame_width = card_frame_width(view_mode, dims);

    if let Some(kind) = card.creates {
        // B36: create カードは mouse_area 経路(doc 冒頭参照)。意匠は
        // [`create_card_face`] が button 経路の [`card_style`] と同じ文法を
        // container で再現する(hover は pane-local 状態)。
        let face = container(body)
            .width(frame_width)
            .padding(dims.spacing_xs)
            .style(move |_theme| create_card_face(colors, selected, hovered));
        return mouse_area(face)
            .on_press(select_message.clone())
            .on_double_click(Message::CreateFromCard { kind })
            .on_enter(Message::CardHovered(key))
            .on_exit(Message::CardUnhovered(key))
            .interaction(iced::mouse::Interaction::Pointer)
            .into();
    }

    if let Some(action) = card.applies_to_selection {
        // 裁定205 施工第2号: 「選択中レイヤーへ足す」カード(Effects タブの
        // Glow/Mask)も create カードと**全く同じ** mouse_area 経路 — 新しい
        // 見た目/文法は作らない(`create_card_face` をそのまま流用)。運ぶ
        // Message だけが `SelectionAction` の payload で分かれる。
        let message = match action {
            model::SelectionAction::AddMask => Message::AddMaskFromCard,
            model::SelectionAction::ApplyEffect(plugin_id) => {
                Message::ApplyEffectFromCard { plugin_id }
            }
            model::SelectionAction::ApplyOp(op) => Message::ApplyOpFromCard { op },
        };
        let face = container(body)
            .width(frame_width)
            .padding(dims.spacing_xs)
            .style(move |_theme| create_card_face(colors, selected, hovered));
        return mouse_area(face)
            .on_press(select_message.clone())
            .on_double_click(message)
            .on_enter(Message::CardHovered(key))
            .on_exit(Message::CardUnhovered(key))
            .interaction(iced::mouse::Interaction::Pointer)
            .into();
    }

    button(body)
        .on_press(select_message)
        .width(frame_width)
        .padding(dims.spacing_xs)
        .style(move |_theme, status| card_style(dims, colors, selected, false, status))
        .into()
}

/// create カード(mouse_area 経路)の面 — button 経路の [`card_style`] と
/// 同じ状態文法を container で再現する: 選択=`state_selected` 地 / hover=
/// `surface_hover` 地 / 通常=透明(裁定179「箱は状態の器」)。文字色は中身の
/// `text()` が自前で持つ(`text_primary`/`text_muted`)ので指定不要。
/// **pub**: `tests/create_card_fence.rs` が button 経路との同文法を機械照合する。
pub fn create_card_face(colors: Colors, selected: bool, hovered: bool) -> container::Style {
    let background = if selected {
        Some(iced::Background::Color(colors.state_selected))
    } else if hovered {
        Some(iced::Background::Color(colors.surface_hover))
    } else {
        None
    };
    container::Style {
        background,
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
