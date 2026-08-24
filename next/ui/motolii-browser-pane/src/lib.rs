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
//!   ([`view_mode_toggle_view`])。List の水平カードレイアウト(サムネ小+
//!   テキスト右)は B36 第5切片で実装済み([`card_body`] 参照 — 当時の
//!   「列数の切替に留める」逸脱は解消)。
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
//!
//! ## Browser 第5切片(List 表示の水平カード+B36 残り点検、この波)
//! - **List 表示の水平カード**(前切片 B08 が列数切替に留めていた分の消化)。
//!   mock `browser-library.css:304-307`
//!   `.libraryBrowser[data-view="list"]` の3宣言(`.libraryCard{display:flex;
//!   align-items:center}`/`.libraryThumb{width:46px;flex:0 0 46px}`/
//!   `.cardCopy{min-width:0;flex:1;padding-left:4px}`)を、CSS 宣言の字面の
//!   まま `motolii_taffy::TaffyBox` へ渡して組む([`card_body`] — 裁定183
//!   taffy 転写、この crate 初採用・型は `motolii-settings-pane::sections::
//!   comp_cells_row`)。新設寸法は `dims.browser_list_thumb_width`
//!   (JSON 正本 `tokens/dimensions.json`、裁定178)の1個だけ — cardCopy の
//!   左 padding(4px)は既存 `spacing_s` と同値なので流用。media/preview の
//!   両カード(button/mouse_area どちらの経路も)が [`card_body`]/
//!   [`card_frame_width`] を共有する。
//! - **B36(新規コンテンツ作成)の残り点検**: normal-map bundle B36 の
//!   「採用予定」行を再点検したが、[`model::CreateKind`] doc の消化台帳
//!   (Rectangle/Ellipse/Solid/Null、前切片で消化済み)+見送り2群(store 拡張
//!   要/pane 外の領分)で全行を説明できており、追加で `LayerSource` へ
//!   1:1で落ちる未消化行は無かった(見送り、RETURN 参照)。テキストレイヤー
//!   作成の map 行(id 1284「New text layer」)は **B36 でなく B46** に属し
//!   verdict も既に「採用済」(store 側 `LayerSource::Text`/`TextDocument`
//!   実装済みを指す)— この切片の EXACT TARGET(この crate のみ)の範囲外
//!   (`CreateKind` の新 variant 追加は `motolii-shell::create_from_card` の
//!   match 網羅性を壊す cross-crate 変更になるため、見送り・RETURN 参照)。
//!
//! ## Browser 第6切片(2026-08-22 発注、map B08 616/617 消化)
//! `normal-map.tsv` B08 束の以下2行を消化: 616「Replace selected footage
//! item」/617「Replace selected source footage for selected layers」—
//! どちらも「store 側に `Intent::SetSource` 実装済み(裁定112c)だが UI
//! (Browser 差替)は未」と注記済みの行(機構は実在、UI だけが空席 — 飾り禁止の
//! 逆側で、作ってよい対象)。この波の範囲はボタン起点の置換のみ:
//! [`model::asset_to_layer_source`](`Asset` → `LayerSource` の純関数)+
//! [`model::can_replace_source`](単一選択のゲーティング)+
//! [`state::Message::ReplaceSelectedLayerSource`](カードの affordance が
//! publish する)+ [`pane_view`] への `single_selected_layer` 引数(export-pane
//! の `WorkAreaFrames` と同型 — pane crate は他 pane/`motolii-shell-state` に
//! 依存しないので、supervisor が `Session::selected_layers` から詰め替えた
//! 値だけを受ける、下記「shell 結線」参照)。
//!
//! **対象外(この波では作らない)**:
//! - **618「Replace source for selected layer(ドラッグ)」**: Stage/Timeline
//!   側に drop target を作る必要があり、この crate の write-set
//!   (`motolii-browser-pane` 単体)を越える cross-pane 機構 — 次波へ持ち越し。
//! - B08/B36 の他行のうち、紐づく実装が無いため見送った物: B36 のドラッグ系
//!   行(618 と同じ cross-pane drag target 不在)、B08 の Collections/Places
//!   系行(タグ束・filesystem 走査裁定待ち、crate 冒頭「B3 の範囲」節の予約地
//!   と同一理由)。実能力の無い選択肢へ顔を作らない(飾り禁止)。
//!
//! ## shell 結線(supervisor 手順)
//! 1. `single_selected_layer: Option<motolii_store::LayerId>` を
//!    `Session::selected_layers`(`motolii-shell-state`)から詰め替える —
//!    `Some(id)` は `len() == 1` の時だけ(0件・2件以上は `None`、`model::
//!    can_replace_source` doc 参照)。これを [`pane_view`] へ渡す(このパネル
//!    は `motolii-shell-state` を引かない、crate 冒頭「この crate の依存」節
//!    どおり — export-pane が `WorkAreaFrames` でやるのと同じ、値だけの写し)。
//! 2. `Message::Browser(msg)` を受けた時、委譲(`self.browser.update(msg)`)は
//!    今までどおり総委譲のまま変えない。その手前で
//!    `if let Message::ReplaceSelectedLayerSource(asset_id) = &msg { .. }` を
//!    1本足す: `Document::view().asset(asset_id)` で `Asset` を引き、
//!    `Some(asset)` かつ `model::asset_to_layer_source(&asset)` が
//!    `Some(source)` の時だけ、選択中 layer へ `Intent::SetSource { layer,
//!    source }` を通常の `Document::apply` 経路で dispatch する。
//!    ([`state::PaneState::update`] はこの腕を no-op 実装済み — 委譲は
//!    そのまま呼んでよい、二重処理にはならない。)

mod card_view;
mod filter_view;
pub mod model;
mod preview_view;
mod rail_view;
mod search_view;
pub mod state;

pub use model::{
    AssetListItem, CardKey, CatalogCard, Category, CreateKind, FILTER_CHIPS, LIBRARY_TABS,
    LibraryTab, PreviewCard, PreviewScope, PreviewTag, RAIL_SCOPES, RailScope, SORT_KEYS,
    SelectionAction, ShapeOpKind, SortKey, ViewMode,
};
pub use state::{CardSelectionModifiers, Message, PaneState};

// SP-6(裁定220 レーン)分割: `lib.rs`(旧1,799行)の view 実装は責任ごとに
// 5分割した(`rail_view`=rail+catalog 容器・`filter_view`=filter shelf+
// sort/view-mode トグル・`search_view`=検索欄+チップ共通スタイル・
// `card_view`=カード grid+素材の欠落バッジ・`preview_view`=preview-local
// カタログ)。crate 外から呼ばれていた `pub` 項目はここから再輸出する
// (呼び出し経路 `motolii_browser_pane::X` を変えないため)。
pub use card_view::{CARD_WIDTH_ROW_HEIGHT_RATIO, GRID_COLUMNS, THUMB_ASPECT_H, THUMB_ASPECT_W};
pub use filter_view::FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO;
pub use preview_view::create_card_face;
pub use rail_view::{drop_target_style, panel_container_style};
pub use search_view::chip_style;

use card_view::card_grid_view_with_selection;
use filter_view::filter_shelf_view;
use preview_view::{preview_body, preview_body_with_selection};
use rail_view::{catalog_container, catalog_view, rail_view};

use iced::widget::{button, column, container, row, text};
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
///
/// `single_selected_layer`(第6切片、map B08 616/617): supervisor が
/// `Session::selected_layers` から詰め替えた「単一選択」の写し(export-pane
/// の `WorkAreaFrames` と同型 — この crate は `motolii-shell-state` を引かない
/// ので値だけ受ける、crate 冒頭「shell 結線」参照)。`Some` かつカードの素材が
/// 置換可能な時だけ [`media_body`] がカードへ Replace affordance を出す —
/// preview タブ(effects/create/panels)には効かない(素材置換は media 台帳の
/// 概念、[`preview_body`] は今までどおり受け取らない)。
pub fn pane_view(
    state: &PaneState,
    items: &[AssetListItem],
    single_selected_layer: Option<motolii_store::LayerId>,
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
            single_selected_layer,
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

/// modifier-aware な Browser pane の入口。
///
/// 既存の [`pane_view`] は旧 `SelectCard` publish を維持する互換ラッパーで
/// あり、既存の atlas/外部利用者を壊さない。Shell WIRE が Iced の現在の
/// Cmd/Ctrl/Shift を [`CardSelectionModifiers`] へ正規化できるようになったら、
/// こちらへ切り替える。view 側は OS イベントを読まず、Document/Store も所有
/// しない。
#[allow(clippy::too_many_arguments)]
pub fn pane_view_with_modifiers(
    state: &PaneState,
    items: &[AssetListItem],
    single_selected_layer: Option<motolii_store::LayerId>,
    modifiers: CardSelectionModifiers,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let band = tab_band_view(state.tab(), dims, colors);
    let body: Element<'static, Message> = match state.tab() {
        LibraryTab::Media => media_body_with_selection(
            items,
            state.scope(),
            state.query(),
            state.sort_key(),
            state.view_mode(),
            state.selected_cards(),
            modifiers,
            state.recently_admitted(),
            state.drop_hover(),
            single_selected_layer,
            dims,
            colors,
        ),
        tab => preview_body_with_selection(
            tab,
            state.preview_scope(),
            state.query(),
            state.view_mode(),
            state.selected_cards(),
            modifiers,
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
    // 旧入口は選択 layer の情報を持たない呼び手向け — Replace affordance は
    // 出さない(`None`、触れない物に触れそうな顔をさせない、crate 冒頭
    // 「shell 結線」参照)。
    media_body(
        items,
        scope,
        query,
        model::SortKey::default(),
        model::ViewMode::default(),
        None,
        &[],
        false,
        None,
        dims,
        colors,
    )
}

/// media タブの body(rail + カタログ)。B2/B3 の [`view`] と同一の木に、
/// カード選択意匠(`selected`、mock `.libraryCard.selected`)+ 並べ替え/
/// 表示形式(B08 第4切片「素材の整理」、[`model::SortKey`]/[`model::ViewMode`])
/// + 新規素材ハイライト(`recent`)+ drop 先ハイライト(`drop_hover`、B08 続編)
/// + Replace affordance の元(`single_selected_layer`、第6切片)を足した形。
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
    single_selected_layer: Option<motolii_store::LayerId>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let filtered = model::sorted(&model::visible(items, scope, query), sort_key);
    let ledger_is_empty = items.is_empty();

    let rail = rail_view(scope, dims, colors);
    let catalog = catalog_view(
        scope,
        query,
        sort_key,
        view_mode,
        &filtered,
        ledger_is_empty,
        selected,
        recent,
        drop_hover,
        single_selected_layer,
        dims,
        colors,
    );

    row![rail, catalog].spacing(dims.spacing_xs).into()
}

/// modifier-aware な media body。`filtered` の scope/query/sort 後の順序を
/// [`card_grid_view_with_selection`] へ渡し、カード click が同じ可視列を
/// Shift 範囲選択の入力として運ぶ。旧 [`media_body`] は互換経路として
/// `rail_view::catalog_view` を使い続ける。
#[allow(clippy::too_many_arguments)]
fn media_body_with_selection(
    items: &[AssetListItem],
    scope: RailScope,
    query: &str,
    sort_key: model::SortKey,
    view_mode: model::ViewMode,
    selected_cards: &[CardKey],
    modifiers: CardSelectionModifiers,
    recent: &[motolii_store::AssetId],
    drop_hover: bool,
    single_selected_layer: Option<motolii_store::LayerId>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let filtered = model::sorted(&model::visible(items, scope, query), sort_key);
    let rail = rail_view(scope, dims, colors);
    let shelf = filter_shelf_view(scope, query, sort_key, view_mode, dims, colors);
    let summary = text(format!("Results {}", filtered.len()))
        .size(dims.micro_text)
        .color(colors.text_muted);
    let grid = card_grid_view_with_selection(
        &filtered,
        items.is_empty(),
        selected_cards,
        modifiers,
        recent,
        view_mode,
        single_selected_layer,
        dims,
        colors,
    );
    let catalog = catalog_container(column![shelf, summary, grid], drop_hover, dims, colors);

    row![rail, catalog].spacing(dims.spacing_xs).into()
}
