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
//! [`CARD_WIDTH_ROW_HEIGHT_RATIO`]/[`THUMB_ASPECT_W`]/[`THUMB_ASPECT_H`]/
//! [`FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`] の doc に分母と出典を明記
//! する。余白は既存の spacing ラダー(`dims.spacing_xs`/`spacing_s`/
//! `spacing_m`)をそのまま再利用する(裁定167 のラダー自体は `Dimensions`
//! 側で既に量子化済みの段 — 新しい段を発明しない)。文字は `dims.micro_text`
//! (mock `.cardCopy strong/small{font-size:8px}` と一致する既存の未消費段 —
//! 裁定168 の em 族はこの crate 独自の余白計算をしない分適用対象が無い、
//! 名前/caption の非衝突は rail.rs と同じ native ellipsis 手口([`card_view`]
//! 参照)で満たす)。
//!
//! ## B3 の比率台帳による転写(この波、利用者実窓不合格 2026-08-22 朝への対応)
//! B3 着地時点は「構造は `browser-library.html` から借用したが比率・余白・
//! 文字は自前判断」だったため実窓がモックと別物に見えていた。
//! `docs/reviews/2026-08-22-browser-ratio-ledger.md`(`browser-library.css`
//! 実測、Inspector I-ratio→I-tokens と同型の2段を1レーンで実施)により、
//! rail 行/filter チップ/検索欄/結果件数の文字を `caption_text`(9、自前
//! 判断)から `micro_text`(8、mock 実測)へ、filter チップ/Clear の角丸を
//! `FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`(mock `border-radius:8px`
//! 実測)へ、rail 行/filter チップの padding を mock 実測へ最近傍の既存
//! token へ転写した。card 幅・rail:catalog 比・card 間 gap は
//! `motolii-shell::screenshot`(ALLOWLIST 外)が同じ定数を直読みしており、
//! 転写すると shell 側と desync するため据え置き(台帳4節 FINDING)。
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
/// 予約地、crate 冒頭 doc 参照)。**台帳(1c節)**: 行の padding は mock
/// `.locationRow{padding:2px 6px 0}`(`browser-library.css:150`)実測 —
/// 垂直は `spacing_xs`(2、一致)、水平は `spacing_s`(4)/`spacing_m`(8)が
/// 6px から同着(差2)なので、このリポジトリ全体で単行ボタンの定番の組である
/// `[spacing_xs, spacing_m]` を採る(inspector/settings/stage/shell が
/// 同一の組を使用済み、台帳 1c 節)。角丸=0(mock `.locationRow` に角丸
/// 指定なし)。容器自身の padding は `.librarySidebar{padding:2px 0 6px}`
/// (横0 — 横方向は行側が持つ)を転写し、`[spacing_xs, 0.0]`(旧実装の
/// 一律 `spacing_s` は行側 padding 新設後は横方向が二重になる、台帳 1c 節)。
fn rail_view(scope: RailScope, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let rows: Vec<Element<'static, Message>> = RAIL_SCOPES
        .into_iter()
        .map(|option| {
            scope_button(
                option,
                option == scope,
                Length::Fill,
                [dims.spacing_xs, dims.spacing_m],
                0.0,
                dims,
                colors,
            )
        })
        .collect();

    container(column(rows).spacing(dims.spacing_xs).padding([dims.spacing_xs, 0.0]))
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

    // 台帳(1a節): `.resultSummary strong,span{font-size:8px}`
    // (`browser-library.css:225-226`)— `caption_text`(9)ではなく
    // `micro_text`(8)。
    let summary = text(format!("Results {}", filtered.len()))
        .size(dims.micro_text)
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
fn filter_shelf_view(
    scope: RailScope,
    query: &str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let chip_radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let chip_padding = [dims.spacing_xs, dims.spacing_s];
    let chips: Vec<Element<'static, Message>> = FILTER_CHIPS
        .into_iter()
        .map(|option| {
            scope_button(
                option,
                option == scope,
                Length::Shrink,
                chip_padding,
                chip_radius,
                dims,
                colors,
            )
        })
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
            .size(dims.micro_text)
            .width(Length::FillPortion(2))
            .style(move |_theme, status| search_input_style(dims, colors, status)),
        row(chips).spacing(dims.spacing_xs),
        button(text("Clear").size(dims.micro_text))
            .on_press(Message::ClearFilters)
            .padding(chip_padding)
            .style(move |_theme, status| chip_style(dims, colors, false, status, chip_radius)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// rail 行/filter チップ、共通のボタン(選択状態を1箇所で塗り分ける —
/// 2つの入口が同じ意匠を共有する、Ableton可視性原理どおり)。**台帳
/// (1a節)**: 文字は mock 実測でどちらも8px — `micro_text`(旧
/// `caption_text`=9 は自前判断だった)。`padding`/`radius` は呼び出し側
/// (rail 行 vs filter チップ)で異なる mock 実測値を渡す([`rail_view`]/
/// [`filter_shelf_view`] の doc 参照 — 色/選択状態の意匠だけを共有し、
/// 形(padding・角丸)は mock どおり呼び出し側で分ける)。
fn scope_button(
    scope: RailScope,
    selected: bool,
    width: Length,
    padding: [f32; 2],
    radius: f32,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(scope.label()).size(dims.micro_text))
        .on_press(Message::SelectScope(scope))
        .width(width)
        .padding(padding)
        .style(move |_theme, status| chip_style(dims, colors, selected, status, radius))
        .into()
}

/// [`scope_button`]/Clear ボタン共通のスタイル。選択中は `action_active`
/// (accent、mock `.selected`/`.filterShelf button.selected` の金色枠と同じ
/// 意味役割)で縁取る。`radius` は呼び出し側の mock 実測値
/// ([`FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`]/rail の `0.0`)。
fn chip_style(
    dims: Dimensions,
    colors: Colors,
    selected: bool,
    status: button::Status,
    radius: f32,
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
