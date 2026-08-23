//! filter rail(SP-6 分割: 元 `lib.rs` から移送 — rail 列(media/preview 共通)+
//! catalog/rail の共通容器スタイル)。

use crate::card_view::card_grid_view;
use crate::filter_view::filter_shelf_view;
use crate::model::{self, AssetListItem, LibraryTab, PreviewScope, RailScope, RAIL_SCOPES};
use crate::search_view::labeled_button;
use crate::Message;
use iced::widget::{column, container, text};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

/// rail 列(mock `.librarySidebar` の `LIBRARY` 節。`COLLECTIONS`/`PLACES` は
/// 予約地、crate 冒頭 doc 参照)。**台帳(1c節)**: 行の padding は mock
/// `.locationRow{padding:2px 6px 0}`(`browser-library.css:150`)実測 —
/// 垂直は `spacing_xs`(2、一致)、水平は `spacing_s`(4)/`spacing_m`(8)が
/// 6px から同着(差2)なので、このリポジトリ全体で単行ボタンの定番の組である
/// `[spacing_xs, spacing_m]` を採る(inspector/settings/stage/shell が
/// 同一の組を使用済み、台帳 1c 節)。角丸=0(mock `.locationRow` に角丸
/// 指定なし)。容器自身の padding は `.librarySidebar{padding:2px 0 6px}`
/// (横0 — 横方向は行側が持つ)を転写し、`[spacing_xs, 0.0]`(旧実装の
/// 一律 `spacing_s` は行側 padding 新設後は横方向が二重になる、台帳 1c 節)。
pub(crate) fn rail_view(scope: RailScope, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let rows: Vec<Element<'static, Message>> = RAIL_SCOPES
        .into_iter()
        .map(|option| {
            labeled_button(
                option.label(),
                option == scope,
                Message::SelectScope(option),
                Length::Fill,
                [dims.spacing_xs, dims.spacing_m],
                0.0,
                dims,
                colors,
            )
        })
        .collect();

    rail_container(rows, dims, colors)
}

/// 非 media タブの rail 列(mock `.tabScoped-effects/-create/-panels` の
/// カテゴリ節 — 構造の対称化)。行の構成は「All …」行([`model::LibraryTab::
/// all_label`])+ タブ別カテゴリ([`model::preview_tags`]、mock 掲載順=S0)。
/// 行の意匠・padding・容器は media の [`rail_view`] と完全に同一
/// (`COLLECTIONS` 節は media 同様まだ描かない予約地 — crate 冒頭 doc 参照)。
pub(crate) fn preview_rail_view(
    tab: LibraryTab,
    scope: PreviewScope,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let row_padding = [dims.spacing_xs, dims.spacing_m];
    let mut rows: Vec<Element<'static, Message>> = vec![labeled_button(
        tab.all_label(),
        scope == PreviewScope::All,
        Message::SelectPreviewScope(PreviewScope::All),
        Length::Fill,
        row_padding,
        0.0,
        dims,
        colors,
    )];
    rows.extend(model::preview_tags(tab).iter().map(|&tag| {
        labeled_button(
            tag.label(),
            scope == PreviewScope::Tag(tag),
            Message::SelectPreviewScope(PreviewScope::Tag(tag)),
            Length::Fill,
            row_padding,
            0.0,
            dims,
            colors,
        )
    }));

    rail_container(rows, dims, colors)
}

/// rail/catalog 容器の共通スタイル。線化 D5(裁定179 文法1「枠は内容と同族色、
/// 段差は明度1段だけ」): 容器の輪郭線は描かず、`surface_panel` の面が app 地
/// (`surface_app`、theme 経由の窓地・pane 間隙間の色)から**明度1段**浮くこと
/// が輪郭 — 透明 border で幅だけ残す(幾何不変)。
/// `settings_pane::chrome::panel_container_style` と同文法だが、pane crate 間の
/// 相互依存を作らないためここに小さく複製する([`search_input_style`] と同じ
/// 判断)。`pub`: `tests/container_line_fence.rs` が機械照合する。
pub fn panel_container_style(dims: Dimensions, colors: Colors) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(colors.surface_panel)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// rail 列の容器(media/preview 共通 — 台帳 1c 節の padding・地・枠は
/// [`rail_view`] の doc 参照。2つの rail が同じ意匠を1箇所で共有する)。
fn rail_container(
    rows: Vec<Element<'static, Message>>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(
        column(rows)
            .spacing(dims.spacing_xs)
            .padding([dims.spacing_xs, 0.0]),
    )
    .width(Length::FillPortion(1))
    .style(move |_theme| panel_container_style(dims, colors))
    .into()
}

/// filter shelf(mock `.filterShelf`)+ 結果件数 + カード grid(mock
/// `.thumbnailGrid`、B3 — [`card_grid_view`])。
#[allow(clippy::too_many_arguments)]
pub(crate) fn catalog_view(
    scope: RailScope,
    query: &str,
    sort_key: model::SortKey,
    view_mode: model::ViewMode,
    filtered: &[AssetListItem],
    ledger_is_empty: bool,
    selected: Option<model::CardKey>,
    recent: &[motolii_store::AssetId],
    drop_hover: bool,
    single_selected_layer: Option<motolii_store::LayerId>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let shelf = filter_shelf_view(scope, query, sort_key, view_mode, dims, colors);

    // 台帳(1a節): `.resultSummary strong,span{font-size:8px}`
    // (`browser-library.css:225-226`)— `caption_text`(9)ではなく
    // `micro_text`(8)。
    let summary = text(format!("Results {}", filtered.len()))
        .size(dims.micro_text)
        .color(colors.text_muted);

    let grid = card_grid_view(
        filtered,
        ledger_is_empty,
        selected,
        recent,
        view_mode,
        single_selected_layer,
        dims,
        colors,
    );

    catalog_container(column![shelf, summary, grid], drop_hover, dims, colors)
}

/// カタログ容器(media/preview 共通 — 地・枠・padding を1箇所で共有する。
/// `FillPortion(4)` は rail の `FillPortion(1)` と対で mock
/// `.librarySidebar{width:112px}` : `.catalog{flex:1}` の比を近似した既決値)。
/// `drop_hover` は media タブの drop 中だけ真([`drop_target_style`] へ
/// 切り替わる — preview タブは常に偽)。
pub(crate) fn catalog_container(
    content: iced::widget::Column<'static, Message>,
    drop_hover: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(content.spacing(dims.spacing_xs).padding(dims.spacing_m))
        .width(Length::FillPortion(4))
        .style(move |_theme| {
            if drop_hover {
                drop_target_style(dims, colors)
            } else {
                panel_container_style(dims, colors)
            }
        })
        .into()
}

/// drop 受け入れ面のハイライト(B08 続編)。`motolii-shell` の pane_grid
/// `hovered_region`(題帯レーン #3)と**同じ文法・同じロール**: 面=
/// `surface_hover`(drag 中に cursor が乗っている受け入れ面)+ 縁=`focus`
/// (操作が着地する場所の合図)× 太さ `border_width * 2.0`(強調線の既存
/// 導出)。border は幅が [`panel_container_style`] と違うが、iced の border は
/// layout に効かない(bounds 内へ描く)ので幾何不変。**pub**:
/// `tests/drop_target_fence.rs` が pane_grid 文法との同型を機械照合する。
pub fn drop_target_style(dims: Dimensions, colors: Colors) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(colors.surface_hover)),
        border: iced::Border {
            color: colors.focus,
            width: dims.border_width * 2.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}
