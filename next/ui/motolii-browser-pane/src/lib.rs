//! wraps: iced+motolii-store+motolii-tokens-rs — Browser pane 骨格(B0)+
//! 一覧 projection(B1)+ rail/filter(B2)+ view 配線(B3、この波)。
//!
//! ζ 縫い目調査(`docs/reviews/2026-08-21-browser-seam-survey.md`)+裁定162 の
//! 切片割り: B0 骨格(挙動ゼロ・PNG 一致)→ B1 素材列挙(`model.rs`)→
//! B2 rail/filter → **B3 view(この crate、この波: 視覚、
//! `browser-library.html` 構造のみ借用しトンマナは tokens 読み替え・
//! `motolii-shell::Shell::view` への組み込み)** → B4 静止画サムネ ∥ B5 動画
//! サムネ → B6 ドラッグ配置。第一波は MEDIA 種別のみ(EFFECTS/CREATE は
//! 意味起草タスク#14 が空席)。
//!
//! ## B3 の範囲(この波)
//! - カード grid(mock `.thumbnailGrid`/`.libraryCard`/`.libraryThumb`/
//!   `.cardCopy` の構造転写、[`card_grid_view`])。**サムネイルは代表フレーム
//!   抽出なし**(B5 境界、`docs/reviews/2026-08-21-browser-seam-survey.md`
//!   FINDING 1「動画サムネは旧世界でも未実装」)— thumb は種別グリフ
//!   ([`model::Category::glyph`])+ 種別で塗り分けた色地のみ。名前+尺
//!   ([`model::format_duration`])の「カード骨格」まで(OUTCOME (1))
//! - `Shell::view` への実配線(パネル開閉トグル込み、[`state::Message::
//!   ToggleBrowserPanel`]/[`state::PaneState::is_open`])。`Shell` 側は
//!   `self.browser.view(...)` が返す木を `is_open()` の時だけ差し込むだけ
//!   (`Settings` パネルと同じ「表示だけの分岐」)
//! - rail/filter(B2)はそのまま残す — この波は「カード grid を実装し、
//!   pane 全体を Shell へつなぐ」ことが範囲で、rail/filter 自体の意匠は
//!   変えない
//!
//! `Collections`/`Places`/タブ4種(EFFECTS/CREATE/PANELS)/selection tray/
//! tag editor/context menu/履歴 ‹› は `browser-semantics.html` 救出台帳が
//! 明記する「予約地」のまま(タグ束・filesystem 走査裁定・意味起草タスク#14
//! 待ち) — この波でも出さない(B2 の留保をそのまま延長)。
//!
//! ## 寸法の裁定165/167/168 遵守(カード grid)
//! `motolii-tokens-rs` は書き換えない(この波の allowlist 外) — 新しい寸法は
//! 全部この crate 内のローカル定数として、既存 token(`Dimensions::
//! row_height`)からの**比率**で導出する(裁定165「形は比率で定数化」)。
//! [`CARD_WIDTH_ROW_HEIGHT_RATIO`]/[`THUMB_ASPECT_W`]/[`THUMB_ASPECT_H`] の
//! doc に分母と出典を明記する。余白は既存の spacing ラダー
//! (`dims.spacing_xs`/`spacing_s`)をそのまま再利用する(裁定167 のラダー自体
//! は `Dimensions` 側で既に量子化済みの段 — 新しい段を発明しない)。文字は
//! `dims.micro_text`(mock `.cardCopy strong/small{font-size:8px}` と一致する
//! 既存の未消費段 — 裁定168 の em 族はこの crate 独自の余白計算をしない分
//! 適用対象が無い、名前/caption の非衝突は rail.rs と同じ native ellipsis
//! 手口([`card_view`] 参照)で満たす)。
//!
//! ## この crate の依存
//! pane split 流儀(`docs/reviews/2026-08-21-pane-split-survey.md`)が許す
//! 構成は `iced+motolii-core+motolii-store+motolii-tokens-rs+
//! motolii-shell-state(+motolii-media、`motolii-stage-pane` の
//! `motolii-engine` 依存と同型の単独例外)`。B1 で `motolii-store` を、
//! B2 で `motolii-tokens-rs`(view の寸法・色)を足した。rail scope/検索欄/
//! パネル開閉の状態は `Session` を必要としない pane-local な形(`state.rs`
//! doc 参照)なので、`motolii-shell-state` はまだ引かない — サムネ
//! (`motolii-media`)は B4/B5。
//!
//! ## `motolii-shell` への組み込み(B3、この波で完了)
//! root `motolii_shell::Message::Browser(Message)` が1本で畳む
//! (`Settings`/`Stage`/`Timeline` と同型)。`Shell::update` は
//! `Message::Browser(msg) => self.browser.update(msg)` のまま(B1/B2 から
//! 不変 — `ToggleBrowserPanel` も含め `PaneState` が丸ごと引き取る、`state.rs`
//! 冒頭 doc 参照)。`Shell::view` はヘッダに "Browser" トグルボタンを持ち、
//! `self.browser.is_open()` の間だけ [`view`] の出力を木へ差し込む。台帳への
//! 記帳自体(`Intent::AdmitAsset` の発行)は `Shell::admit`(`motolii-shell`
//! 側)が持つ — この crate は読み専用の projection+絞り込み+view しか持たない。

pub mod model;
pub mod state;

pub use model::{AssetListItem, Category, RailScope, FILTER_CHIPS, RAIL_SCOPES};
pub use state::{Message, PaneState};

use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};

use motolii_tokens_rs::{Colors, Dimensions};

/// `Shell::view` がパネル全体へ割く高さ = `dims.row_height` の何倍か(裁定165
/// 「形は比率で定数化・分母明記」、**分母 = `Dimensions::row_height`**)。
/// filter shelf+結果件数(≈2行)+ カード2行ぶん([`CARD_WIDTH_ROW_HEIGHT_RATIO`]/
/// [`THUMB_ASPECT_W`]/[`THUMB_ASPECT_H`] から逆算した1行あたりの高さ)+ 余白を
/// 目安に丸めた値。**`pub`**: `motolii-shell::lib.rs::Shell::view`(実配線)と
/// `motolii-shell::screenshot`(トンマナ検分 instrument)の両方がこの1つの値を
/// 共有する(値を複製しない — 複製すると2箇所が食い違う典型的な二重保守)。
/// スクロール自体は内側の `scrollable`(mock 同様)が持つので、この高さは
/// 「全カードが常に見える高さ」である必要はない。
pub const PANEL_HEIGHT_ROW_HEIGHT_RATIO: f32 = 14.0;

/// rail(mock `.librarySidebar` `LIBRARY` 節)+ filter shelf(mock
/// `.filterShelf`)+ カード grid(mock `.thumbnailGrid`、B3)を描く。
/// **selection tray/tag editor/context menu はまだ描かない**(予約地、crate
/// 冒頭 doc 参照)。`items` は [`model::assets`](B1)がそのまま返す未絞り込みの
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

/// filter shelf(mock `.filterShelf`)+ 結果件数 + カード grid(mock
/// `.thumbnailGrid`、B3 — [`card_grid_view`])。
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

    let grid = card_grid_view(filtered, dims, colors);

    container(
        column![shelf, summary, grid]
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

// ---------------------------------------------------------------------------
// B3: カード grid(mock `.thumbnailGrid`/`.libraryCard` の構造転写)。
// ---------------------------------------------------------------------------

/// grid の列数(mock 既定表示 `data-view="grid"` の
/// `grid-template-columns: repeat(2, minmax(0,1fr))` — 直接転写。thumbnails
/// モード(4列)/list モード(1列)の view mode トグルは selection tray/tag
/// editor と同様まだ描かない予約地、crate 冒頭 doc 参照)。**`pub`**:
/// `motolii-shell::screenshot`(トンマナ検分 instrument)が同じ比率で近似矩形を
/// 描くため、値を複製せずここから読む。
pub const GRID_COLUMNS: usize = 2;

/// カード幅 = `dims.row_height` の何倍か(裁定165「形は比率で定数化・分母
/// 明記」)。**分母 = `Dimensions::row_height`**。旧世界のカード寸(意味論
/// モック「未決」台帳が挙げる「旧124×84踏襲か」)を絶対px のまま転写せず、
/// 既存 token への比率へ変換した値 — `6.0 × 20px(既定 row_height)= 120px`
/// (旧値124との差3%は「絶対px正本を持たない」制約を比率丸めで解消した結果)。
/// **`pub`**: [`GRID_COLUMNS`] と同じ理由(`motolii-shell::screenshot` 参照)。
pub const CARD_WIDTH_ROW_HEIGHT_RATIO: f32 = 6.0;

/// サムネの縦横比(mock `.libraryThumb{aspect-ratio:16/9}` の直接転写、
/// 分母=9)。**`pub`**: [`GRID_COLUMNS`] と同じ理由。
pub const THUMB_ASPECT_W: f32 = 16.0;
pub const THUMB_ASPECT_H: f32 = 9.0;

/// カード grid 本体。**サムネイルは代表フレーム抽出なし**(B5 境界、crate
/// 冒頭 doc 参照)— thumb は種別グリフ+種別で塗り分けた色地のみ、名前+尺の
/// 「カード骨格」まで(B3 OUTCOME (1))。`filtered` が空なら mock 同様「無い」
/// ことを1行で言う(B2 から不変の文言)。
fn card_grid_view(
    filtered: &[AssetListItem],
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    if filtered.is_empty() {
        return container(
            text("No matching media")
                .size(dims.caption_text)
                .color(colors.text_muted),
        )
        .padding(dims.spacing_m)
        .into();
    }

    let rows: Vec<Element<'static, Message>> = filtered
        .chunks(GRID_COLUMNS)
        .map(|chunk| {
            let cards: Vec<Element<'static, Message>> = chunk
                .iter()
                .cloned()
                .map(|item| card_view(item, dims, colors))
                .collect();
            row(cards).spacing(dims.spacing_s).into()
        })
        .collect();

    scrollable(column(rows).spacing(dims.spacing_s))
        .height(Length::Fill)
        .into()
}

/// 1枚のカード(mock `.libraryCard` — thumb + `.cardCopy`)。名前/caption は
/// `rail.rs`(Timeline)と同じ native ellipsis 手口(`Wrapping::None` +
/// `Ellipsis::End`、`iced_test::simulator` 上の `canvas::Text` には効かない
/// バグの回避 — この widget は実 `text()` なので影響しない、TL-arch §2.5
/// 実測)で「隣の箱へ決して入らない」(裁定168 不衝突文法)。
fn card_view(item: AssetListItem, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let card_width = dims.row_height * CARD_WIDTH_ROW_HEIGHT_RATIO;
    let thumb_height = card_width * THUMB_ASPECT_H / THUMB_ASPECT_W;
    let category = model::category_of(&item.kind);

    let thumb = container(
        text(category.glyph())
            .size(dims.micro_text)
            .color(colors.text_primary),
    )
    .width(Length::Fixed(card_width))
    .height(Length::Fixed(thumb_height))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(thumb_fill(category, colors))),
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let text_width = card_width;
    let name = text(item.name)
        .size(dims.micro_text)
        .color(colors.text_primary)
        .width(Length::Fixed(text_width))
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End);

    let caption = text(format!("{} · {}", category.label(), model::format_duration(item.duration)))
        .size(dims.micro_text)
        .color(colors.text_muted)
        .width(Length::Fixed(text_width))
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End);

    container(column![thumb, name, caption].spacing(dims.spacing_xs))
        .width(Length::Fixed(card_width))
        .padding(dims.spacing_xs)
        .into()
}

/// カード thumb の塗り(種別ごとに既存 `Colors` ロールを再利用 — 新ロールは
/// 起こさない。装飾的な塗り分けであって新しい意味役割ではないため、裁定164
/// 「意味役割が新しい時は専用ロールを起こす」の対象外と判断)。
fn thumb_fill(category: model::Category, colors: Colors) -> iced::Color {
    match category {
        model::Category::Video => colors.way_timeline,
        model::Category::Image => colors.shape,
        model::Category::Audio => colors.data,
        model::Category::Other => colors.surface_raised,
    }
}
