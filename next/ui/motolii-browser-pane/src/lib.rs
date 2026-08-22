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
//! ## B08 第4切片(素材の整理、この波)
//! 発注書「Browser 第4切片 — 素材の整理(B08 残り+タグ/コレクション)」への
//! 対応。normal-map の bundle B08(素材取り込み束、`intent-bundles.tsv` 参照)
//! の採用予定行を洗ったが、freq の上位行は import/replace/proxy/reload 系
//! ばかりで「並べ替え/表示形式/タグ/お気に入り」に該当する行は1本も無かった
//! (メニューコマンド抽出ベースの map にはツールバー慣習の sort/grid⇄list
//! toggle が現れない)。発注書 step2 の明示リストに従い、`Asset` の実属性
//! だけで組める2つを実装した:
//! - **並べ替え**([`model::SortKey`]/[`model::sorted`] — 名前/追加日
//!   (`AssetId` 昇順を代理指標に使う、store に wall-clock timestamp が無い
//!   ため)/種別)。media タブの filter shelf にのみチップで現れる
//!   ([`sort_control_view`])— preview-local カタログは宣言順を保つ契約
//!   ([`model::preview_visible`] doc 参照)なので対象外。
//! - **表示形式**([`model::ViewMode`]、`Icon::GridView`/`Icon::ViewList`
//!   在庫)。media/preview 両方の catalog grid の列数に効く
//!   ([`view_mode_toggle_view`])。List は mock の水平カードレイアウトまでは
//!   実装せず列数の切替に留める([`columns_for`] doc 参照 — 逸脱)。
//! - **検索の対象拡張**([`model::visible`] が `path` も見るようになった —
//!   `AssetListItem::path` は B1 から既に投影に乗っていた実在属性)。
//!
//! COLLECTIONS(Favorite)節・タグチップは**予約地のまま**(飾り禁止) —
//! `motolii_store::Asset` にタグ/お気に入り相当のフィールドが無いため
//! (`asset.rs` のフィールド一覧参照)。store にこの形の属性が要る、という
//! 要求としてこの doc に記録しておく(発注書 RETURN と同じ内容)。
//!
//! `Collections`/`Places`/selection tray/tag editor/context menu/履歴 ‹› は
//! `browser-semantics.html` 救出台帳が明記する「予約地」のまま(タグ束・
//! filesystem 走査裁定・意味起草タスク#14 待ち) — この波でも出さない(B2 の
//! 留保をそのまま延長)。**タブ4種(Media/Effects/Create/Panels)は予約地を
//! 脱した**(B3 取り残し回収、利用者実窓不合格 2026-08-22): タブ帯
//! ([`tab_band_view`]、[`pane_view`] が組む)+タブ状態
//! ([`state::Message::SelectTab`]/[`model::LibraryTab`])+ preview-local
//! 静的カタログ([`model::preview_catalog`] — mock 冒頭コメントの宣言どおり
//! filesystem/Document/Host/intent/persistence 非接続のプレビュー専用データ)。
//! media タブは従来どおり Document 台帳投影の経路で静的データを混ぜない。
//! タブ帯の寸法は `tokens/dimensions.json` の `browser_*` キーが正本
//! (2026-08-22 利用者裁定「デザイン値の外出し徹底」 — ここへ値を複製しない)。
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

//! ## Browser 第3切片(B36 create 実体化+B08 取り込み続編、この波)
//! - **create タブの実体化(B36)**: create タブのカードは全枚が「作る」を
//!   宣言する([`model::PreviewCard::creates`]/[`model::CreateKind`] — 消化した
//!   map 行と見送りは `CreateKind` doc の台帳)。シングルクリック=選択・
//!   **ダブルクリック=作成**(AE/Figma 慣習 S0、[`state::Message::
//!   CreateFromCard`])。button は press を capture して `on_double_click` へ
//!   届かない(fork `widget/src/mouse_area.rs::update` は content が capture
//!   すると return する実測)ため、create カードだけ `mouse_area` 経路 —
//!   hover 意匠は pane-local の [`state::Message::CardHovered`] で自前に持つ
//!   (Q0: hover 無反応にしない)。実際のレイヤー生成= shell 結線(次波)。
//!   drag で Stage/Timeline へ、は将来切片(見送り)。
//! - **B08 続編(取り込み UX)**: drop 中は media タブの catalog 容器を
//!   drop 受け入れ面として塗る([`drop_target_style`] — `motolii-shell` の
//!   pane_grid `hovered_region` と同じ文法: 面=`surface_hover`+縁=`focus`
//!   ×2)。取り込み直後の新規素材はカードの縁が `focus` で光る
//!   ([`state::Message::RecentlyAdmitted`]、カード選択かタブ切替で消灯)。
//!   Results の空状態は2値を区別する: 台帳自体が空=「Drop files here」
//!   (取り込みの入口を最小の1句で言う)/絞り込みで0件=「No matches」。
//!   `FileHovered`/`FilesHoveredLeft` → [`state::Message::DropHoverChanged`]、
//!   admit 後 → `RecentlyAdmitted` の shell 結線は次波(write-set 外)。

pub mod model;
pub mod state;

pub use model::{
    AssetListItem, CardKey, CatalogCard, Category, CreateKind, LibraryTab, PreviewCard,
    PreviewScope, PreviewTag, RailScope, FILTER_CHIPS, LIBRARY_TABS, RAIL_SCOPES,
};
pub use state::{Message, PaneState};

use iced::widget::{button, column, container, mouse_area, row, scrollable, text, text_input};
    AssetListItem, CardKey, CatalogCard, Category, LibraryTab, PreviewCard, PreviewScope,
    PreviewTag, RailScope, SortKey, ViewMode, FILTER_CHIPS, LIBRARY_TABS, RAIL_SCOPES, SORT_KEYS,
};
pub use state::{Message, PaneState};

use iced::widget::{button, column, container, row, scrollable, text, text_input, tooltip};
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

/// **タブ帯込みの pane 全体**(mock `.libraryTabs` html:411-416 + タブ別
/// catalog — B3 転写の取り残し回収、利用者実窓不合格 2026-08-22 対応)。
/// `Shell::view` はこちらへ乗り換えるのが正 — [`view`] は media タブの
/// body だけを描く旧入口(shell 側配線が write-set 外のため残置、逸脱として
/// RETURN 記載)。
///
/// - タブ帯: 4タブ常設可視(S6 — メニューの奥に隠さない)。寸法は全て
///   tokens 経由(`dims.browser_tab_bar_height`/`dims.browser_tab_underline`、
///   正本 `tokens/dimensions.json` の `_note_browser_*` — 2026-08-22 利用者
///   裁定「デザイン値の外出し徹底」)。active/inactive は既存トークンの
///   ロール差のみ(S4: `action_active` 下線+`surface_app` 地+`text_primary`
///   vs 透明地+`text_muted` — 新ロールなし)。
/// - media タブ: [`media_body`](Document 台帳投影の従来経路 — [`view`] と
///   同一の木+カード選択意匠)。
/// - effects/create/panels タブ: [`preview_body`](preview-local 静的カタログ、
///   [`model::preview_catalog`])。**media と同じ構造文法**(左 rail=スコープ
///   選択 / 上=検索+フィルタチップ / 右=カタログ — 構造の対称化、利用者
///   実窓指摘 2026-08-22「Browser の構造が media タブにしか適用されていない」
///   への対応)。rail/チップの中身は mock がタブ別に宣言する語彙
///   ([`model::preview_tags`])。検索欄は mock では toolbar 領域=全タブ共有
///   なので従来どおり全タブに出す。
pub fn pane_view(
    state: &PaneState,
    items: &[AssetListItem],
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let band = tab_band_view(state.tab(), dims, colors);
    let body: Element<'static, Message> = match state.tab() {
        LibraryTab::Media => media_body(
            items,
            state.scope(),
            state.query(),
            state.sort_key(),
            state.view_mode(),
            state.selected(),
            state.recently_admitted(),
            state.drop_hover(),
            dims,
            colors,
        ),
        tab => preview_body(
            tab,
            state.preview_scope(),
            state.query(),
            state.view_mode(),
            state.selected(),
            state.hovered(),
            dims,
            colors,
        ),
    };
    column![band, body].spacing(dims.spacing_xs).into()
}

/// タブ帯(mock `.libraryTabs` の転写)。寸法は tokens 経由のみ:
/// - 帯高 = `dims.browser_tab_bar_height`(mock `height:26px` 実測、JSON 正本)
/// - active 下線 = `dims.browser_tab_underline`(mock `border-bottom:2px` 実測)
///   × 色は `colors.action_active`(mock `#d8b574` と同役)
/// - 帯下の罫線 = 線化 D5(裁定179 文法1)で**塗らない** — `dims.border_width`
///   ぶんの間隔だけ残す(mock `border-bottom:1px solid border-default` は
///   「区切りは明度1段+間隔」が上書き。幾何不変)
/// - 文字 = `dims.micro_text`(mock `.libraryTabs button{font-size:8px}`)
/// - タブ幅 = 等分(mock `flex:1` → `Length::FillPortion(1)`)
fn tab_band_view(
    active: LibraryTab,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let underline_height = dims.browser_tab_underline;
    let button_height =
        (dims.browser_tab_bar_height - underline_height - dims.border_width).max(0.0);

    let tabs: Vec<Element<'static, Message>> = LIBRARY_TABS
        .into_iter()
        .map(|tab| {
            let selected = tab == active;
            let label = container(text(tab.label()).size(dims.micro_text))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center);
            let tab_button = button(label)
                .on_press(Message::SelectTab(tab))
                .width(Length::Fill)
                .height(Length::Fixed(button_height))
                .padding(0)
                .style(move |_theme, status| tab_style(colors, selected, status));
            let underline = container(
                iced::widget::Space::new()
                    .width(Length::Fill)
                    .height(Length::Fixed(underline_height)),
            )
            .style(move |_theme| container::Style {
                // mock: 非選択タブも `border-bottom:2px solid transparent` を
                // 持つ(タブ切替で高さが動かない) — 透明のまま場所だけ確保。
                background: selected.then_some(iced::Background::Color(colors.action_active)),
                ..container::Style::default()
            });
            column![tab_button, underline]
                .width(Length::FillPortion(1))
                .into()
        })
        .collect();

    // 線化 D5(裁定179 文法1): mock の帯下罫線(`border-bottom:1px solid
    // border-default`)は「区切りは明度1段+間隔」が上書き — 塗らずに
    // `border_width` ぶんの間隔だけ残す(幾何不変。帯と body の区切りは
    // app 地の隙間と body 容器の `surface_panel` 明度段が担う)。
    let divider = container(
        iced::widget::Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(dims.border_width)),
    );

    column![row(tabs), divider].into()
}

/// タブの塗り分け(mock `.libraryTabs button` / `[aria-selected="true"]`)。
/// S4: 既存トークンのロール差のみ — active = `surface_app` 地+
/// `text_primary`、inactive = 透明地+`text_muted`(hover は既存の
/// `surface_hover`)。新ロールは起こさない。
fn tab_style(colors: Colors, selected: bool, status: button::Status) -> button::Style {
    let background = if selected {
        Some(iced::Background::Color(colors.surface_app))
    } else {
        match status {
            button::Status::Hovered => Some(iced::Background::Color(colors.surface_hover)),
            _ => None,
        }
    };
    button::Style {
        background,
        text_color: if selected {
            colors.text_primary
        } else {
            colors.text_muted
        },
        border: iced::Border {
            radius: 0.0.into(),
            ..iced::Border::default()
        },
        ..button::Style::default()
    }
}

/// rail(mock `.librarySidebar` `LIBRARY` 節)+ filter shelf(mock
/// `.filterShelf`)+ カード grid(mock `.thumbnailGrid`、B3)を描く。
/// **selection tray/tag editor/context menu はまだ描かない**(予約地、crate
/// 冒頭 doc 参照)。`items` は [`model::assets`](B1)がそのまま返す未絞り込みの
/// 投影 — 絞り込みはこの関数の中で [`model::visible`] を呼ぶ(呼び手は
/// フィルタ済みリストを別途作らなくてよい)。
///
/// **media タブの body 専用の旧入口**(タブ帯を含まない・カード選択意匠なし)
/// — タブ帯込みの全体は [`pane_view`](こちらが選択意匠も渡す)。既存試験
/// (`rail_filter_atlas.rs`/`browser_ratio_ledger.rs`)が直読みするため
/// シグネチャは据え置き([`media_body`] の `selected: None` 特殊形)。
pub fn view(
    items: &[AssetListItem],
    scope: RailScope,
    query: &str,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    media_body(items, scope, query, None, &[], false, dims, colors)
}

/// media タブの body(rail + カタログ)。B2/B3 の [`view`] と同一の木に、
/// カード選択意匠(`selected`、mock `.libraryCard.selected`)+ 新規素材
/// ハイライト(`recent`)+ drop 先ハイライト(`drop_hover`、B08 続編)を
/// 足した形。
    media_body(
        items,
        scope,
        query,
        model::SortKey::default(),
        model::ViewMode::default(),
        None,
        dims,
        colors,
    )
}

/// media タブの body(rail + カタログ)。B2/B3 の [`view`] と同一の木に、
/// カード選択意匠(`selected`、mock `.libraryCard.selected`)+ 並べ替え/
/// 表示形式(B08 第4切片「素材の整理」、[`model::SortKey`]/[`model::ViewMode`])
/// を足した形。
#[allow(clippy::too_many_arguments)]
fn media_body(
    items: &[AssetListItem],
    scope: RailScope,
    query: &str,
    sort_key: model::SortKey,
    view_mode: model::ViewMode,
    selected: Option<model::CardKey>,
    recent: &[motolii_store::AssetId],
    drop_hover: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let filtered = model::visible(items, scope, query);
    let ledger_is_empty = items.is_empty();

    let rail = rail_view(scope, dims, colors);
    let catalog = catalog_view(
        scope,
        query,
        &filtered,
        ledger_is_empty,
        selected,
        recent,
        drop_hover,
        dims,
        colors,
    let filtered = model::sorted(&model::visible(items, scope, query), sort_key);

    let rail = rail_view(scope, dims, colors);
    let catalog = catalog_view(
        scope, query, sort_key, view_mode, &filtered, selected, dims, colors,
    );

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
fn preview_rail_view(
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
fn catalog_view(
    scope: RailScope,
    query: &str,
    sort_key: model::SortKey,
    view_mode: model::ViewMode,
    filtered: &[AssetListItem],
    ledger_is_empty: bool,
    selected: Option<model::CardKey>,
    recent: &[motolii_store::AssetId],
    drop_hover: bool,
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

    let grid = card_grid_view(filtered, ledger_is_empty, selected, recent, dims, colors);
    let grid = card_grid_view(filtered, selected, view_mode, dims, colors);

    catalog_container(column![shelf, summary, grid], drop_hover, dims, colors)
}

/// カタログ容器(media/preview 共通 — 地・枠・padding を1箇所で共有する。
/// `FillPortion(4)` は rail の `FillPortion(1)` と対で mock
/// `.librarySidebar{width:112px}` : `.catalog{flex:1}` の比を近似した既決値)。
/// `drop_hover` は media タブの drop 中だけ真([`drop_target_style`] へ
/// 切り替わる — preview タブは常に偽)。
fn catalog_container(
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
fn preview_filter_shelf_view(
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

/// 検索欄(mock `#library-search` — mock では toolbar 領域=**全タブ共有**
/// なので、media の filter shelf と preview タブの両方がこの1本を使う)。
///
/// 裁定170 M01: fork(0.15.0-dev)の `text_input()` は `&str`/`&String` を
/// `Fragment::Borrowed` として受け、返り値のライフタイムを入力の借用に
/// 縛る(`settings_pane::channel_cell`/`ui_scale_row` と同じ実測済みの
/// 事情、両方の doc comment 参照)。呼び手のシグネチャは `Element<'static,
/// _>` を返す必要があるため、owned のまま move する。
fn search_field(query: &str, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let query_owned = query.to_owned();
    text_input("Search files and tags", query_owned)
        .on_input(Message::QueryChanged)
        .size(dims.micro_text)
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
fn labeled_button(
    label: &'static str,
    selected: bool,
    message: Message,
    width: Length,
    padding: [f32; 2],
    radius: f32,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    button(text(label).size(dims.micro_text))
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
///   透明にし、幅は `dims.border_width` のまま(幾何不変 — レイアウトに
///   効く値を動かさない)。mock が非選択チップへ宣言する常時
///   `border-default` 輪郭(`browser-library.css:215-226`)はこの裁定が
///   上書き(rail 行 `.locationRow` は mock 自体が非選択透明、css:135-152)。
/// - 選択= 現行表現の維持: `state_selected` 地+`action_active` 縁/ink
///   (mock `.filterShelf button.selected{border-color:#d8b574}` css:228 の
///   宣言どおり — 選択状態の輪郭は mock が明示する部分)。
///
/// `radius` は呼び出し側の mock 実測値
/// ([`FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO`]/rail の `0.0`)を素通し。
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

/// [`model::ViewMode`] に応じた grid の列数(B08 第4切片「表示形式」)。
/// List は1列 — mock の list mode(`.libraryBrowser[data-view="list"]
/// .thumbnailGrid{grid-template-columns:1fr}`、`browser-library.css:304`)は
/// 加えてカードを水平レイアウト(サムネ小+テキスト右)へ変えるが、この波は
/// 列数の切替までに留める(水平レイアウトは thumb 縮小サイズ等の新しい比率
/// 定数を要求し、裁定165 の外側の値を発明することになるため — 次波の課題、
/// RETURN 記載の逸脱)。
fn columns_for(mode: model::ViewMode) -> usize {
    match mode {
        model::ViewMode::Grid => GRID_COLUMNS,
        model::ViewMode::List => 1,
    }
}

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
/// 「カード骨格」まで(B3 OUTCOME (1))。空状態は2値を区別する(B08 続編、
/// 説明文は最小の1句 — 裁定185 の精神):
/// - 台帳自体が空(まだ何も取り込んでいない)= 「Drop files here」 —
///   取り込みの入口(drop)を言う。
/// - 台帳はあるが絞り込みで0件 = 「No matches」。
fn card_grid_view(
    filtered: &[AssetListItem],
    ledger_is_empty: bool,
    selected: Option<model::CardKey>,
    recent: &[motolii_store::AssetId],
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    if filtered.is_empty() {
        let copy = if ledger_is_empty {
            "Drop files here"
        } else {
            "No matches"
        };
        return container(text(copy).size(dims.caption_text).color(colors.text_muted))
            .padding(dims.spacing_m)
            .into();
    }

    let rows: Vec<Element<'static, Message>> = filtered
        .chunks(columns_for(view_mode))
        .map(|chunk| {
            let cards: Vec<Element<'static, Message>> = chunk
                .iter()
                .cloned()
                .map(|item| {
                    let key = model::CardKey::Media(item.id);
                    let is_recent = recent.contains(&item.id);
                    card_view(item, selected == Some(key), is_recent, dims, colors)
                })
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
///
/// カードは mock どおり `<button>`(`.libraryCard{cursor:pointer}`) —
/// click で [`Message::SelectCard`] を publish し(mock `selectCard` の
/// 単一選択)、選択中は [`card_style`] の意匠(mock `.libraryCard.selected`)。
fn card_view(
    item: AssetListItem,
    selected: bool,
    recent: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
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
        // 線化 D5(裁定179 文法1): thumb の輪郭線は透明化(幅だけ残す=幾何
        // 不変)— 種別色の面自体が pane 地からの明度/色段差で読める。
        background: Some(iced::Background::Color(thumb_fill(category, colors))),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
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

    let caption = text(format!(
        "{} · {}",
        category.label(),
        model::format_duration(item.duration)
    ))
    .size(dims.micro_text)
    .color(colors.text_muted)
    .width(Length::Fixed(text_width))
    .wrapping(iced::widget::text::Wrapping::None)
    .ellipsis(iced::widget::text::Ellipsis::End);

    button(column![thumb, name, caption].spacing(dims.spacing_xs))
        .on_press(Message::SelectCard(model::CardKey::Media(item.id)))
        .width(Length::Fixed(card_width))
        .padding(dims.spacing_xs)
        .style(move |_theme, status| card_style(dims, colors, selected, recent, status))
        .into()
}

/// カード共通のスタイル(mock `.libraryCard{background:transparent}` /
/// `.libraryCard.selected{background:白20%mix}` — 後者は既存の
/// `state_selected` ロールと同役)。hover は rail 行と同じ `surface_hover`
/// (mock は `.libraryCard:hover .libraryThumb` の枠色変化だが、iced の
/// button style から子 container の枠へは届かないため、pane 内の既存 hover
/// 文法へ読み替える — 逸脱として RETURN 記載)。
///
/// `recent`(B08 続編: 取り込み直後の新規素材)は縁が `focus` で光る —
/// [`drop_target_style`] と同じ「操作が着地した場所の合図」ロール(drop 先
/// ハイライトの続きとして同族色で受け止める)。border は色だけ動かし幅は
/// `dims.border_width` 固定(裁定179 の幾何不変 — 非 recent は透明)。
fn card_style(
    dims: Dimensions,
    colors: Colors,
    selected: bool,
    recent: bool,
    status: button::Status,
) -> button::Style {
    let background = if selected {
        Some(iced::Background::Color(colors.state_selected))
    } else {
        match status {
            button::Status::Hovered => Some(iced::Background::Color(colors.surface_hover)),
            _ => None,
        }
    };
    let border_color = if recent {
        colors.focus
    } else {
        iced::Color::TRANSPARENT
    };
    button::Style {
        background,
        text_color: colors.text_primary,
        border: iced::Border {
            color: border_color,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
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
fn preview_body(
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
    let catalog = preview_catalog_view(tab, scope, query, selected, hovered, dims, colors);
    let catalog = preview_catalog_view(tab, scope, query, view_mode, selected, dims, colors);

    row![rail, catalog].spacing(dims.spacing_xs).into()
}

/// 非 media タブの catalog(filter shelf + 結果件数 + カード grid —
/// [`catalog_view`] と同じ骨格を preview-local データで組む)。空状態の文言は
/// media の「絞り込みで0件」と同じ「No matches」(B08 続編の文言整理 —
/// preview カタログは静的なので「台帳が空」の面は存在しない)。
/// [`catalog_view`] と同じ骨格を preview-local データで組む)。
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
        let rows: Vec<Element<'static, Message>> = cards_data
            .chunks(columns_for(view_mode))
            .map(|chunk| {
                let cards: Vec<Element<'static, Message>> = chunk
                    .iter()
                    .map(|&card| {
                        let key = model::CardKey::Preview(card.id);
                        preview_card_view(
                            card,
                            selected == Some(key),
                            hovered == Some(key),
                            dims,
                            colors,
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
/// `.cardCopy` 骨格 — [`card_view`] の静的データ版で、click/選択意匠も同一
/// ([`Message::SelectCard`]/[`card_style`])。寸法・文字は media カードと
/// 同一(比率台帳3節の既決値をそのまま共有)。thumb の塗りは
/// `surface_raised` 一律 — mock の装飾色(`.thumb-magenta` 等の直書き hex)は
/// tokens に対応ロールが無く、S4「新ロールを起こさない」を優先して転写しない
/// (逸脱として RETURN 記載)。
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
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let card_width = dims.row_height * CARD_WIDTH_ROW_HEIGHT_RATIO;
    let thumb_height = card_width * THUMB_ASPECT_H / THUMB_ASPECT_W;

    let thumb = container(
        text(card.glyph)
            .size(dims.micro_text)
            .color(colors.text_primary),
    )
    .width(Length::Fixed(card_width))
    .height(Length::Fixed(thumb_height))
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .style(move |_theme| container::Style {
        // 線化 D5(裁定179 文法1): thumb の輪郭線は透明化(幅だけ残す=幾何
        // 不変)— `surface_raised` の面が pane 地から明度1段浮く。
        background: Some(iced::Background::Color(colors.surface_raised)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    });

    let name = text(card.name)
        .size(dims.micro_text)
        .color(colors.text_primary)
        .width(Length::Fixed(card_width))
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End);

    let caption = text(card.caption)
        .size(dims.micro_text)
        .color(colors.text_muted)
        .width(Length::Fixed(card_width))
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End);

    let key = model::CardKey::Preview(card.id);
    let body = column![thumb, name, caption].spacing(dims.spacing_xs);

    if let Some(kind) = card.creates {
        // B36: create カードは mouse_area 経路(doc 冒頭参照)。意匠は
        // [`create_card_face`] が button 経路の [`card_style`] と同じ文法を
        // container で再現する(hover は pane-local 状態)。
        let face = container(body)
            .width(Length::Fixed(card_width))
            .padding(dims.spacing_xs)
            .style(move |_theme| create_card_face(colors, selected, hovered));
        return mouse_area(face)
            .on_press(Message::SelectCard(key))
            .on_double_click(Message::CreateFromCard { kind })
            .on_enter(Message::CardHovered(key))
            .on_exit(Message::CardUnhovered(key))
            .interaction(iced::mouse::Interaction::Pointer)
            .into();
    }

    button(body)
        .on_press(Message::SelectCard(key))
        .width(Length::Fixed(card_width))
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
