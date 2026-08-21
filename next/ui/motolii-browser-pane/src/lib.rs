//! wraps: iced+motolii-store+motolii-tokens-rs — Browser pane 骨格(B0)+
//! 一覧 projection(B1)+ rail/filter(B2、この波)。
//!
//! ζ 縫い目調査(`docs/reviews/2026-08-21-browser-seam-survey.md`)+裁定162 の
//! 切片割り: B0 骨格(挙動ゼロ・PNG 一致)→ B1 素材列挙(`model.rs`)→
//! **B2 rail/filter(この crate、この波)** → B3 view(視覚、
//! `browser-library.html` 構造のみ借用しトンマナは tokens 読み替え・
//! `motolii-shell::Shell::view` への組み込み)→ B4 静止画サムネ ∥ B5 動画
//! サムネ → B6 ドラッグ配置。第一波は MEDIA 種別のみ(EFFECTS/CREATE は
//! 意味起草タスク#14 が空席)。
//!
//! ## B2 の範囲(この波)
//! [`model::visible`](rail scope + 検索文字列で一覧を絞る純関数)+
//! [`state::PaneState`](scope/検索欄の transient 状態)+ [`view`](mock
//! `.librarySidebar` の `LIBRARY` 節相当の rail 列 + `.filterShelf` 相当の
//! filter shelf を実 widget として描く — **rail/filter だけ**、mock のカード
//! grid・selection tray・tag editor 等は B3 が全体トンマナと一緒に転写する)。
//! `Collections`/`Places` は `browser-semantics.html` 救出台帳が明記する
//! 「予約地」のまま(タグ束・filesystem 走査裁定待ち) — この波では rail に
//! 出さない。
//!
//! **`motolii-shell::Shell::view` へはまだ組み込まない**(crate doc 冒頭・
//! `motolii-shell::browser_pane` re-export の doc も参照)— B1 と同じ
//! 「描画ゼロ = 挙動ゼロ変更の証明」を延長し、視覚配線(パネル開閉トグル込み)
//! は B3 で絵の全体と一緒に行う。この波の [`view`] は crate 自身の dev-only
//! `iced_test` 直叩き(`tests/rail_filter_atlas.rs`)でだけ検分される —
//! `inspector_pane` の `tests/value_cell_legibility.rs` と同じ「shell を経由
//! せず pane 単体を叩く」手口(Cargo.toml dev-dependency の doc 参照)。
//!
//! ## この crate の依存
//! pane split 流儀(`docs/reviews/2026-08-21-pane-split-survey.md`)が許す
//! 構成は `iced+motolii-core+motolii-store+motolii-tokens-rs+
//! motolii-shell-state(+motolii-media、`motolii-stage-pane` の
//! `motolii-engine` 依存と同型の単独例外)`。B1 で `motolii-store` を、
//! B2 で `motolii-tokens-rs`(view の寸法・色)を足した。rail scope/検索欄の
//! 状態は `Session` を必要としない pane-local な形(`state.rs` doc 参照)な
//! ので、`motolii-shell-state` はまだ引かない — サムネ(`motolii-media`)は
//! B4/B5。
//!
//! ## `motolii-shell` への組み込み(B1/B2 時点)
//! root `motolii_shell::Message::Browser(Message)` が1本で畳む想定
//! (`Settings`/`Stage`/`Timeline` と同型)。`motolii_shell::Message::Browser`
//! の match 腕は `state::Message` が非空になったのに合わせて
//! `self.browser.update(msg)`(`timeline_pane::PaneState::update` と同型の
//! 委譲)へ更新済み — ただし呼び先が消費するだけで `Shell::view` には
//! まだ何も現れない(上記「B2 の範囲」参照)。台帳への記帳自体
//! (`Intent::AdmitAsset` の発行)は `Shell::admit`(`motolii-shell` 側)が
//! 持つ — この crate は読み専用の projection+絞り込みしか持たない。

pub mod model;
pub mod state;

pub use model::{AssetListItem, Category, RailScope, FILTER_CHIPS, RAIL_SCOPES};
pub use state::{Message, PaneState};

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use motolii_tokens_rs::{Colors, Dimensions};

/// rail(mock `.librarySidebar` `LIBRARY` 節)+ filter shelf(mock
/// `.filterShelf`)+ 絞り込み結果の一覧を描く。**mock のカード grid/
/// selection tray/tag editor はまだ描かない**(B3、crate 冒頭 doc 参照) —
/// この波は「rail 選択+filter で一覧が絞れる」ことが実 widget で見える
/// ところまで。`items` は [`model::assets`](B1)がそのまま返す未絞り込みの
/// 投影 — 絞り込みはこの関数の中で [`model::visible`] を呼ぶ(呼び手は
/// フィルタ済みリストを別途作らなくてよい)。
pub fn view(
    items: &[AssetListItem],
    scope: RailScope,
    query: &str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let filtered = model::visible(items, scope, query);

    let rail = rail_view(scope, dims, colors);
    let catalog = catalog_view(scope, query, &filtered, dims, colors);

    row![rail, catalog].spacing(dims.spacing_xs).into()
}

/// rail 列(mock `.librarySidebar` の `LIBRARY` 節。`COLLECTIONS`/`PLACES` は
/// 予約地、crate 冒頭 doc 参照)。
fn rail_view(scope: RailScope, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let rows: Vec<Element<'static, Message>> = RAIL_SCOPES
        .into_iter()
        .map(|option| scope_button(option, option == scope, Length::Fill, dims, colors))
        .collect();

    container(column(rows).spacing(dims.spacing_xs).padding(dims.spacing_s))
        .width(Length::FillPortion(1))
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_panel)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// filter shelf(mock `.filterShelf`)+ 結果件数 + 一覧(mock のカード grid
/// ではなく、この波は簡易な行リスト — 視覚の grid/thumbnail は B3・B4)。
fn catalog_view(
    scope: RailScope,
    query: &str,
    filtered: &[AssetListItem],
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let shelf = filter_shelf_view(scope, query, dims, colors);

    let summary = text(format!("Results {}", filtered.len()))
        .size(dims.caption_text)
        .color(colors.text_muted);

    let list: Element<'static, Message> = if filtered.is_empty() {
        container(
            text("No matching media")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding(dims.spacing_m)
        .into()
    } else {
        column(
            filtered
                .iter()
                .cloned()
                .map(|item| result_row(item, dims, colors))
                .collect::<Vec<_>>(),
        )
        .spacing(dims.spacing_xs)
        .into()
    };

    container(
        column![shelf, summary, scrollable(list).height(Length::Fill)]
            .spacing(dims.spacing_xs)
            .padding(dims.spacing_m),
    )
    .width(Length::FillPortion(4))
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors.surface_panel)),
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

/// filter shelf 本体(mock `.filterShelf` — 検索欄 + 種別チップ + Clear)。
/// チップは [`FILTER_CHIPS`](rail の `RAIL_SCOPES` から `AllMedia` を除いた
/// もの、mock に `All media` チップが無いのと同じ)。
fn filter_shelf_view(
    scope: RailScope,
    query: &str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chips: Vec<Element<'static, Message>> = FILTER_CHIPS
        .into_iter()
        .map(|option| scope_button(option, option == scope, Length::Shrink, dims, colors))
        .collect();

    // 裁定170 M01: fork(0.15.0-dev)の `text_input()` は `&str`/`&String` を
    // `Fragment::Borrowed` として受け、返り値のライフタイムを入力の借用に
    // 縛る(`settings_pane::channel_cell`/`ui_scale_row` と同じ実測済みの
    // 事情、両方の doc comment 参照)。この関数のシグネチャは `Element<'static,
    // _>` を返す必要があるため、owned のまま move する。
    let query_owned = query.to_owned();
    row![
        text_input("Search files and tags", query_owned)
            .on_input(Message::QueryChanged)
            .size(dims.caption_text)
            .width(Length::FillPortion(2))
            .style(move |_theme, status| search_input_style(dims, colors, status)),
        row(chips).spacing(dims.spacing_xs),
        button(text("Clear").size(dims.caption_text))
            .on_press(Message::ClearFilters)
            .style(move |_theme, status| chip_style(dims, colors, false, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// rail 行/filter チップ、共通のボタン(選択状態を1箇所で塗り分ける —
/// 2つの入口が同じ意匠を共有する、Ableton可視性原理どおり)。
fn scope_button(
    scope: RailScope,
    selected: bool,
    width: Length,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(scope.label()).size(dims.caption_text))
        .on_press(Message::SelectScope(scope))
        .width(width)
        .style(move |_theme, status| chip_style(dims, colors, selected, status))
        .into()
}

/// [`scope_button`]/Clear ボタン共通のスタイル。選択中は `action_active`
/// (accent、mock `.selected`/`.filterShelf button.selected` の金色枠と同じ
/// 意味役割)で縁取る。
fn chip_style(
    dims: Dimensions,
    colors: Colors,
    selected: bool,
    status: button::Status,
) -> button::Style {
    let background = if selected {
        colors.state_selected
    } else {
        match status {
            button::Status::Hovered => colors.surface_hover,
            button::Status::Pressed => colors.state_selected,
            button::Status::Disabled => colors.surface_panel,
            button::Status::Active => colors.surface_raised,
        }
    };
    let border_color = if selected {
        colors.action_active
    } else {
        colors.border_default
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else if selected {
        colors.action_active
    } else {
        colors.text_primary
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
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
            width: dims.border_width,
            radius: 0.0.into(),
        },
        placeholder: colors.text_muted,
        value: colors.text_primary,
        selection: colors.action_active,
    }
}

/// 一覧の1行(mock のカード grid ではなく、この波の簡易表示 — 名前+種別)。
fn result_row(item: AssetListItem, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    row![
        text(item.name).size(dims.body_text).color(colors.text_primary).width(Length::Fill),
        text(item.kind).size(dims.caption_text).color(colors.text_muted),
    ]
    .spacing(dims.spacing_xs)
    .into()
}
