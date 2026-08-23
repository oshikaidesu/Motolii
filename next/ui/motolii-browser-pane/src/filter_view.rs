//! filter shelf + 並べ替え/表示形式トグル(SP-6 分割: 元 `lib.rs` から移送)。

use crate::model::{self, LibraryTab, PreviewScope, RailScope, FILTER_CHIPS};
use crate::search_view::{chip_style, labeled_button, search_field};
use crate::Message;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

/// filter chip/Clear の角丸 = `dims.row_height` の何倍か(裁定165「形は
/// 比率で定数化・分母明記」)。**分母 = `Dimensions::row_height`**。mock
/// `.filterShelf button,.editorTags button{border-radius:8px}`
/// (`browser-library.css:206`、Clear ボタンも html:484 `class="clearFilter"`
/// として同一セレクタに同居、css:229)を既定 `row_height`(20px)で割った値 —
/// `0.4 × 20px = 8px`(mock の絶対pxと厳密一致、台帳1b節)。rail 行
/// (`.locationRow`)には角丸指定が無い(直角のまま、[`scope_button`] の
/// rail 呼び出し側は `0.0`)。
pub const FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO: f32 = 0.4;

/// filter shelf 本体(mock `.filterShelf` — 検索欄 + 種別チップ + Clear)。
/// チップは [`FILTER_CHIPS`](rail の `RAIL_SCOPES` から `AllMedia` を除いた
/// もの、mock に `All media` チップが無いのと同じ)。**台帳(1a/1c節)**:
/// 検索欄・チップ・Clear の文字は mock 実測で例外なく8px
/// (`#library-search`/`.filterShelf button` とも `browser-library.css`)—
/// `micro_text` を使う。チップ/Clear の padding は mock `.filterShelf
/// button{padding:2px 4px}`(css:205)と完全一致する `[spacing_xs,
/// spacing_s]`。角丸は [`FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`]。
pub(crate) fn filter_shelf_view(
    scope: RailScope,
    query: &str,
    sort_key: model::SortKey,
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let chip_padding = [dims.spacing_xs, dims.spacing_s];
    let chips: Vec<Element<'static, Message>> = FILTER_CHIPS
        .into_iter()
        .map(|option| {
            labeled_button(
                option.label(),
                option == scope,
                Message::SelectScope(option),
                Length::Shrink,
                chip_padding,
                chip_radius,
                dims,
                colors,
            )
        })
        .collect();

    shelf_row(
        query,
        chips,
        Some(sort_control_view(sort_key, dims, colors)),
        view_mode,
        dims,
        colors,
    )
}

/// 並べ替えチップ列(B08 第4切片、mock に類例が無い新規 UI —
/// [`model::SORT_KEYS`] の3チップを filter チップと同じ意匠(選択= 器、
/// 非選択= 素の文字+hover 面、[`chip_style`])で描く。media の filter shelf
/// にのみ現れる([`preview_filter_shelf_view`] は呼ばない — sort は実属性を
/// 持つ台帳データにしか意味を持たない、`state::Message::SelectSortKey` doc
/// 参照)。
fn sort_control_view(
    sort_key: model::SortKey,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let chip_padding = [dims.spacing_xs, dims.spacing_s];
    let chips: Vec<Element<'static, Message>> = model::SORT_KEYS
        .into_iter()
        .map(|key| {
            labeled_button(
                key.label(),
                key == sort_key,
                Message::SelectSortKey(key),
                Length::Shrink,
                chip_padding,
                chip_radius,
                dims,
                colors,
            )
        })
        .collect();
    row(chips).spacing(dims.spacing_xs).into()
}

/// 非 media タブの filter shelf(mock `.filterGroup[data-filter-group=
/// "effects"/"create"/"panels"]` の転写 — 構造の対称化)。チップは
/// [`model::preview_tags`](rail のカテゴリと同ラベル・同 Message — mock の
/// filterGroup チップと `.tabScoped-*` 行が同じ `data-tag-filter` を書くのと
/// 同型)。チップの意匠・Clear・検索欄は media の [`filter_shelf_view`] と
/// 完全に同一。
pub(crate) fn preview_filter_shelf_view(
    tab: LibraryTab,
    scope: PreviewScope,
    query: &str,
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let chip_padding = [dims.spacing_xs, dims.spacing_s];
    let chips: Vec<Element<'static, Message>> = model::preview_tags(tab)
        .iter()
        .map(|&tag| {
            labeled_button(
                tag.label(),
                scope == PreviewScope::Tag(tag),
                Message::SelectPreviewScope(PreviewScope::Tag(tag)),
                Length::Shrink,
                chip_padding,
                chip_radius,
                dims,
                colors,
            )
        })
        .collect();

    // sort チップは media 専用 — `None`([`sort_control_view`] doc 参照)。
    shelf_row(query, chips, None, view_mode, dims, colors)
}

/// filter shelf の骨格(media/preview 共通 — 検索欄+チップ列+Clear。mock
/// `.filterShelf` の並びを1箇所で共有する)。Clear は両タブ群とも
/// [`Message::ClearFilters`](従来 Message)を publish する。
fn shelf_row(
    query: &str,
    chips: Vec<Element<'static, Message>>,
    sort_control: Option<Element<'static, Message>>,
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let chip_padding = [dims.spacing_xs, dims.spacing_s];

    let mut controls: Vec<Element<'static, Message>> = vec![
        search_field(query, dims, colors),
        row(chips).spacing(dims.spacing_xs).into(),
    ];
    // 並べ替えチップは media タブのみ([`sort_control_view`] doc 参照)。
    if let Some(sort_control) = sort_control {
        controls.push(sort_control);
    }
    controls.push(
        button(text("Clear").size(dims.micro_text))
            .on_press(Message::ClearFilters)
            .padding(chip_padding)
            .style(move |_theme, status| chip_style(dims, colors, false, status, chip_radius))
            .into(),
    );
    // grid/list 表示形式トグル(B08 第4切片)。全タブ共通(検索欄と同じ toolbar
    // 領域の慣習)。
    controls.push(view_mode_toggle_view(view_mode, dims, colors));

    row(controls)
        .spacing(dims.spacing_xs)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

/// grid/list 表示形式トグル(B08 第4切片、裁定187 icon+tooltip ペア —
/// `motolii-shell::Shell::header_icon_action` と同じ形をこの pane 内に
/// 小さく複製する。理由は [`search_input_style`]/[`panel_container_style`]
/// と同じ: pane crate 間の相互依存を作らないため)。
fn view_mode_toggle_view(
    active: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let modes = [
        (model::ViewMode::Grid, motolii_icons::Icon::GridView),
        (model::ViewMode::List, motolii_icons::Icon::ViewList),
    ];
    let buttons: Vec<Element<'static, Message>> = modes
        .into_iter()
        .map(|(mode, glyph)| view_mode_button(mode, glyph, mode == active, dims, colors))
        .collect();
    row(buttons).spacing(dims.spacing_xs).into()
}

/// 1個の view mode icon ボタン(輪郭なし・hover/選択で面、裁定179)+ tooltip
/// (裁定187)。アイコン枠寸は shelf の文字寸(`micro_text`)を
/// [`motolii_icons::frame_px_for_glyph_px`](Material live area 比 24/20)で
/// 写した視覚同等寸 — この shelf 行の他要素(検索欄/チップ)と字高を揃える。
fn view_mode_button(
    mode: model::ViewMode,
    glyph: motolii_icons::Icon,
    selected: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let ink = if selected {
        colors.action_active
    } else {
        colors.text_muted
    };
    let icon_element =
        motolii_icons::icon(glyph, motolii_icons::frame_px_for_glyph_px(dims.micro_text), ink);
    let action = button(icon_element)
        .on_press(Message::SelectViewMode(mode))
        .padding(dims.spacing_xs)
        .style(move |_theme, status| {
            let background = if selected {
                Some(iced::Background::Color(colors.state_selected))
            } else {
                match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        Some(iced::Background::Color(colors.surface_hover))
                    }
                    _ => None,
                }
            };
            button::Style {
                background,
                // svg には効かない(tint が正)が、契約として ink を宣言して
                // おく(`motolii_icons` module doc・`header_icon_action` と
                // 同じ注記)。
                text_color: ink,
                ..button::Style::default()
            }
        });
    tooltip(
        action,
        container(
            text(mode.tooltip_label())
                .size(dims.caption_text)
                .color(colors.text_primary),
        )
        .padding([dims.spacing_xs, dims.spacing_s])
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        }),
        tooltip::Position::Bottom,
    )
    .gap(dims.spacing_xs)
    .into()
}
