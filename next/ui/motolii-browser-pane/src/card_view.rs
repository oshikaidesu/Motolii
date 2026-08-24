//! カード grid(SP-6 分割: 元 `lib.rs` から移送 — カード表示(grid/list 両形式)+
//! 素材の欠落バッジ)。

use crate::context_menu;
use crate::filter_view::FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO;
use crate::model::{self, AssetListItem};
use crate::search_view::chip_style;
use crate::{CardSelectionModifiers, Message};
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, tooltip};
use iced::{Element, Length};
use motolii_tokens_rs::{Colors, Dimensions};

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
/// .thumbnailGrid{grid-template-columns:1fr}`、`browser-library.css:304`)。
/// カードそのものの水平レイアウト(サムネ小+テキスト右)は [`card_body`]/
/// [`card_frame_width`] が担う(B36 第5切片でこの波から実装済み)。
pub(crate) fn columns_for(mode: model::ViewMode) -> usize {
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
/// List モードのカード幅は [`card_frame_width`] — 行いっぱい(`Length::Fill`)
/// で、この比率定数は使わない(mock の list mode が `grid-template-columns:
/// 1fr` = 行幅そのものだから)。
pub const CARD_WIDTH_ROW_HEIGHT_RATIO: f32 = 6.0;

/// サムネの縦横比(mock `.libraryThumb{aspect-ratio:16/9}` の直接転写、
/// 分母=9)。**`pub`**: [`GRID_COLUMNS`] と同じ理由。Grid/List どちらの
/// thumb 幅にもこの比率をそのまま適用する(縦横比は表示形式に依らず一定 —
/// mock の `.libraryThumb{aspect-ratio:16/9}` 自体は list mode でも上書き
/// されない、`browser-library.css:304-307` 参照)。
pub const THUMB_ASPECT_W: f32 = 16.0;
pub const THUMB_ASPECT_H: f32 = 9.0;

// ---------------------------------------------------------------------------
// B36 第5切片: List 表示の水平カード(mock `browser-library.css:304-307`
// `.libraryBrowser[data-view="list"]` の転写、前切片(B08 第4切片)が列数
// 切替までに留めていた分の消化)。並べ方(サムネ固定幅+テキスト flex:1)は
// CSS 宣言の字面をほぼそのまま `motolii-taffy::TaffyBox` へ渡す(裁定183
// taffy 転写 — `motolii-settings-pane::sections::comp_cells_row` が確立した
// 型と同じ手口、この crate 初採用)。寸法は `dims.browser_list_thumb_width`
// (JSON 正本 `tokens/dimensions.json` の `_note_browser_list_thumb_width`、
// 裁定178「デザイン値は Rust に直書きしない」)+ 既存 `spacing_s`(cardCopy の
// 左 padding、mock 4px と同値)— 新しい寸法は thumb 幅の1個だけ。
// ---------------------------------------------------------------------------

/// list カードの外枠(mock `.libraryCard{display:flex;align-items:center}`、
/// `browser-library.css:305`)。定数寸法を含まない固定文字列なので JSON を
/// 経由しない(裁定178 は「デザイン値」が対象 — 並べ方のキーワードは値では
/// ない)。
const LIST_CARD_ROW_CSS: &str = "display:flex; align-items:center";

/// list カードの thumb セル(mock `.libraryThumb{width:46px;flex:0 0 46px}`、
/// `browser-library.css:306` — 字面そのまま。`dims.browser_list_thumb_width`
/// が JSON 正本)。
fn list_card_thumb_css(dims: Dimensions) -> String {
    format!(
        "width:{w}px; flex:0 0 {w}px",
        w = dims.browser_list_thumb_width
    )
}

/// list カードのテキストセル(mock `.cardCopy{min-width:0;flex:1;
/// padding-left:4px}`、`browser-library.css:307`)。**`padding-left` は
/// `motolii-taffy` の対応 property に無い**(shorthand `padding` のみ対応、
/// `motolii-taffy/src/css.rs` 冒頭 doc の対応サブセット参照)ため、4値
/// padding shorthand の左辺だけを埋める形へ機械的に読み替える(値そのものは
/// mock と不変 — subset の制約であって設計判断の逸脱ではない)。左 padding の
/// 値は `dims.spacing_s`(4、mock の `4px` と同値の既存 token — cardCopy 専用の
/// 新キーは起こさない、`tokens/dimensions.json` の `_note_browser_list_thumb_width`
/// と同じ判断)。
fn list_card_text_css(dims: Dimensions) -> String {
    format!(
        "min-width:0; flex:1; padding:0px 0px 0px {pad}px",
        pad = dims.spacing_s
    )
}

/// thumb の共通容器(media/preview 両カードで共有 — 種別グリフ+塗りだけが
/// 呼び手ごとに違う、[`card_view`]/[`preview_card_view`] doc 参照)。線化 D5
/// (裁定179 文法1): 輪郭線は透明化(幅だけ残す=幾何不変)— 塗りの面自体が
/// pane 地から明度/色段差で読める。
fn thumb_container(
    glyph: &'static str,
    fill: iced::Color,
    width: f32,
    height: f32,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    container(text(glyph).size(dims.micro_text).color(colors.text_primary))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(fill)),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// 名前/caption 共通の native ellipsis 手口(`rail.rs`(Timeline)と同じ
/// `Wrapping::None`+`Ellipsis::End` — 隣の箱へ決して入らない、裁定168
/// 不衝突文法)。`width` は呼び手が渡す(grid= 固定カード幅・list=
/// `Length::Fill`、[`card_body`] 参照)。
fn ellipsis_text(
    content: String,
    size: f32,
    color: iced::Color,
    width: Length,
) -> Element<'static, Message> {
    text(content)
        .size(size)
        .color(color)
        .width(width)
        .wrapping(iced::widget::text::Wrapping::None)
        .ellipsis(iced::widget::text::Ellipsis::End)
        .into()
}

/// カードの外枠幅(mock の `data-view` 切替そのもの — grid= 固定カード幅
/// ([`CARD_WIDTH_ROW_HEIGHT_RATIO`])・list= 行いっぱい(mock
/// `grid-template-columns:1fr` = 1列が行幅そのもの)。media/preview の両カード
/// (button/mouse_area どちらの経路も)がこの1本を共有する。
pub(crate) fn card_frame_width(view_mode: model::ViewMode, dims: Dimensions) -> Length {
    match view_mode {
        model::ViewMode::Grid => Length::Fixed(dims.row_height * CARD_WIDTH_ROW_HEIGHT_RATIO),
        model::ViewMode::List => Length::Fill,
    }
}

/// カード本体(thumb + 名前/caption)。media/preview の両カードが共有する
/// (呼び手は glyph/塗り/文言だけを渡す)。
/// - grid: mock 既定表示の縦積み(`<button>` 内で thumb→name→caption を
///   `column!` で積む、旧 [`card_view`]/[`preview_card_view`] のまま不変)。
/// - list: mock `.libraryBrowser[data-view="list"]` の水平カード(サムネ
///   小+テキスト右、`browser-library.css:304-307` — [`LIST_CARD_ROW_CSS`]/
///   [`list_card_thumb_css`]/[`list_card_text_css`] を `TaffyBox` へ渡す)。
pub(crate) fn card_body(
    glyph: &'static str,
    thumb_fill: iced::Color,
    name: String,
    caption: String,
    view_mode: model::ViewMode,
    dims: Dimensions,
    colors: Colors,
    status_badge: Option<Element<'static, Message>>,
) -> Element<'static, Message> {
    match view_mode {
        model::ViewMode::Grid => {
            let card_width = dims.row_height * CARD_WIDTH_ROW_HEIGHT_RATIO;
            let thumb_height = card_width * THUMB_ASPECT_H / THUMB_ASPECT_W;
            let thumb = thumb_container(glyph, thumb_fill, card_width, thumb_height, dims, colors);
            let name = ellipsis_text(
                name,
                dims.micro_text,
                colors.text_primary,
                Length::Fixed(card_width),
            );
            let caption = ellipsis_text(
                caption,
                dims.micro_text,
                colors.text_muted,
                Length::Fixed(card_width),
            );
            let mut children: Vec<Element<'static, Message>> = vec![thumb];
            if let Some(badge) = status_badge {
                children.push(badge);
            }
            children.push(name);
            children.push(caption);
            column(children).spacing(dims.spacing_xs).into()
        }
        model::ViewMode::List => {
            let thumb_width = dims.browser_list_thumb_width;
            let thumb_height = thumb_width * THUMB_ASPECT_H / THUMB_ASPECT_W;
            let thumb = thumb_container(glyph, thumb_fill, thumb_width, thumb_height, dims, colors);
            let name = ellipsis_text(name, dims.micro_text, colors.text_primary, Length::Fill);
            let caption = ellipsis_text(caption, dims.micro_text, colors.text_muted, Length::Fill);
            let mut text_children: Vec<Element<'static, Message>> = vec![name, caption];
            if let Some(badge) = status_badge {
                text_children.push(badge);
            }
            let text_block: Element<'static, Message> = column(text_children)
                .spacing(dims.spacing_xs)
                .width(Length::Fill)
                .into();

            let row_style = motolii_taffy::style_from_css_decl(LIST_CARD_ROW_CSS)
                .expect("LIST_CARD_ROW_CSS は固定文字列 — 解釈は必ず成功する");
            let thumb_style = motolii_taffy::style_from_css_decl(&list_card_thumb_css(dims))
                .expect(
                    "list_card_thumb_css は固定テンプレート+dims の px 値のみ埋める — 解釈は必ず成功する",
                );
            let text_style = motolii_taffy::style_from_css_decl(&list_card_text_css(dims)).expect(
                "list_card_text_css は固定テンプレート+dims の px 値のみ埋める — 解釈は必ず成功する",
            );

            motolii_taffy::TaffyBox::new(row_style)
                .push(thumb_style, thumb)
                .push(text_style, text_block)
                .into()
        }
    }
}

/// カード grid 本体。**サムネイルは代表フレーム抽出なし**(B5 境界、crate
/// 冒頭 doc 参照)— thumb は種別グリフ+種別で塗り分けた色地のみ、名前+尺の
/// 「カード骨格」まで(B3 OUTCOME (1))。空状態は2値を区別する(B08 続編、
/// 説明文は最小の1句 — 裁定185 の精神):
/// - 台帳自体が空(まだ何も取り込んでいない)= 「Drop files here」 —
///   取り込みの入口(drop)を言う。
/// - 台帳はあるが絞り込みで0件 = 「No matches」。
#[allow(clippy::too_many_arguments)]
pub(crate) fn card_grid_view(
    filtered: &[AssetListItem],
    ledger_is_empty: bool,
    selected: Option<model::CardKey>,
    recent: &[motolii_store::AssetId],
    view_mode: model::ViewMode,
    single_selected_layer: Option<motolii_store::LayerId>,
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
                    card_view(
                        item,
                        selected == Some(key),
                        is_recent,
                        view_mode,
                        single_selected_layer,
                        None,
                        false,
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
}

/// 複数選択対応のカード grid。`filtered` は scope/query/sort 済みの現在表示
/// 順であり、この関数はその順を [`Message::SelectCardWithModifiers`] の
/// `visible_cards` として各カードへ同じように渡す。
///
/// 旧 [`card_grid_view`] は `rail_view::catalog_view` と外部の旧 `view()` 利用者
/// の互換用に残す。新しい Shell WIRE は [`crate::pane_view_with_modifiers`] から
/// こちらを通る。選択集合は pane state から借りるだけで、この関数やカードが
/// Document/Store の第二所有者になることはない。
#[allow(clippy::too_many_arguments)]
pub(crate) fn card_grid_view_with_selection(
    filtered: &[AssetListItem],
    ledger_is_empty: bool,
    selected_cards: &[model::CardKey],
    modifiers: CardSelectionModifiers,
    recent: &[motolii_store::AssetId],
    view_mode: model::ViewMode,
    single_selected_layer: Option<motolii_store::LayerId>,
    context_menu_anchor: Option<model::CardKey>,
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

    let visible_cards: Vec<model::CardKey> = filtered
        .iter()
        .map(|item| model::CardKey::Media(item.id))
        .collect();
    let rows: Vec<Element<'static, Message>> = filtered
        .chunks(columns_for(view_mode))
        .map(|chunk| {
            let cards: Vec<Element<'static, Message>> = chunk
                .iter()
                .cloned()
                .map(|item| {
                    let key = model::CardKey::Media(item.id);
                    let is_recent = recent.contains(&item.id);
                    card_view_with_message(
                        item,
                        selected_cards.contains(&key),
                        is_recent,
                        view_mode,
                        single_selected_layer,
                        context_menu_anchor,
                        true,
                        dims,
                        colors,
                        Message::SelectCardWithModifiers {
                            key,
                            modifiers,
                            visible_cards: visible_cards.clone(),
                        },
                    )
                })
                .collect();
            row(cards).spacing(dims.spacing_s).into()
        })
        .collect();

    scrollable(column(rows).spacing(dims.spacing_s))
        .height(Length::Fill)
        .into()
}

/// 1枚のカード(mock `.libraryCard` — thumb + `.cardCopy`、本体は
/// [`card_body`] が grid/list 両表示形式を組む)。
///
/// カードは mock どおり `<button>`(`.libraryCard{cursor:pointer}`) —
/// click で [`Message::SelectCard`] を publish し(mock `selectCard` の
/// 単一選択)、選択中は [`card_style`] の意匠(mock `.libraryCard.selected`)。
///
/// `single_selected_layer`(第6切片、map B08 616/617): [`replace_affordance_row`]
/// が `Some` を返す時だけ、カード本体の下へ Replace 行を足す(触れない物に
/// 触れそうな顔をさせない — `None` の間は行ごと出さない)。
///
/// `selected`(`A01-entry.tsv` `RemoveAsset` 行): [`remove_affordance_row`]
/// が `Some` を返す時だけ、同じくカード本体の下へ Remove 行を足す —
/// こちらはカード自体の選択状態のみがゲート(layer 選択にもパス有無にも
/// 依存しない、`remove_affordance_row` doc 参照)。
fn card_view(
    item: AssetListItem,
    selected: bool,
    recent: bool,
    view_mode: model::ViewMode,
    single_selected_layer: Option<motolii_store::LayerId>,
    context_menu_anchor: Option<model::CardKey>,
    enable_context_menu: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let key = model::CardKey::Media(item.id);
    card_view_with_message(
        item,
        selected,
        recent,
        view_mode,
        single_selected_layer,
        context_menu_anchor,
        enable_context_menu,
        dims,
        colors,
        Message::SelectCard(key),
    )
}

/// カード本体の共通実装。旧単一選択と modifier 付き選択の差は publish する
/// Message だけで、カードの body/affordance/selected 見た目は同じ責任に置く。
fn card_view_with_message(
    item: AssetListItem,
    selected: bool,
    recent: bool,
    view_mode: model::ViewMode,
    single_selected_layer: Option<motolii_store::LayerId>,
    context_menu_anchor: Option<model::CardKey>,
    enable_context_menu: bool,
    dims: Dimensions,
    colors: Colors,
    select_message: Message,
) -> Element<'static, Message> {
    let category = model::category_of(&item.kind);
    let caption = format!(
        "{} · {}",
        category.label(),
        model::format_duration(item.duration)
    );
    // `item.name`/`item.id` を後段で move するため、Replace 行の判定に要る
    // 面は先に抜いておく(`AssetId` は `Copy`、`path.is_some()` は bool へ
    // 丸めて独立させる)。
    let asset_id = item.id;
    let key = model::CardKey::Media(asset_id);
    let has_usable_path = item.path.is_some();
    let status_badge = status_badge_view(&item.status, dims, colors);
    let body = card_body(
        category.glyph(),
        thumb_fill(category, colors),
        item.name,
        caption,
        view_mode,
        dims,
        colors,
        status_badge,
    );

    let card_button = button(body)
        .on_press(select_message)
        .width(card_frame_width(view_mode, dims))
        .padding(dims.spacing_xs)
        .style(move |_theme, status| card_style(dims, colors, selected, recent, status));

    // 選択 layer 1件+パス有りの素材の時だけ Replace 行を足す(第6切片、map
    // B08 616/617)。iced の button は入れ子にしない([`card_button`] とは
    // 別の兄弟行にする — カード本体の選択クリックと Replace クリックが
    // 干渉しない、`Message::ReplaceSelectedLayerSource` doc 参照)。
    let mut rows: Vec<Element<'static, Message>> = vec![card_button.into()];
    if let Some(replace_row) = replace_affordance_row(
        single_selected_layer,
        has_usable_path,
        asset_id,
        card_frame_width(view_mode, dims),
        dims,
        colors,
    ) {
        rows.push(replace_row);
    }
    // カードが選択中の時だけ Remove 行を足す(`A01-entry.tsv` `RemoveAsset`
    // 行、`Message::RemoveAssetFromCard` doc 参照)。Replace 行と違い
    // layer 選択にもパス有無にも依存しない — カード自体の選択状態だけが
    // ゲート(総監督裁定: 判断が割れたら摩擦を増やす側 — 常時×印の一発削除
    // ではなく「選択→ボタン出現」の2段階にする)。
    if context_menu_anchor != Some(key) {
        if let Some(remove_row) = remove_affordance_row(
            selected,
            asset_id,
            card_frame_width(view_mode, dims),
            dims,
            colors,
        ) {
            rows.push(remove_row);
        }
    }
    let card = if rows.len() == 1 {
        rows.pop().expect("rows has exactly 1 element")
    } else {
        column(rows).spacing(dims.spacing_xs).into()
    };
    let card = if enable_context_menu && context_menu_anchor == Some(key) {
        let menu = context_menu::view(Some(key), card_frame_width(view_mode, dims), dims, colors)
            .expect("media card context menu has a real action");
        column![card, menu].spacing(dims.spacing_xs).into()
    } else {
        card
    };

    if enable_context_menu {
        mouse_area(card)
            .on_right_press(Message::OpenContextMenu(key))
            .into()
    } else {
        card
    }
}

/// カードに Remove affordance を出すかのゲーティング(`A01-entry.tsv`
/// `RemoveAsset` 行の穴埋め)。`selected` = カード自体が選択中かどうかのみで
/// ゲートする([`replace_affordance_row`] と違い `single_selected_layer`/
/// パス有無は無関係 — `Intent::RemoveAsset { asset }` はレイヤーもパスも
/// 見ない、台帳から `AssetId` を外すだけの Intent、`document.rs` 参照)。
/// 未選択の間は行ごと出さない(常時×印の一発削除にしない = 誤操作防止の
/// 摩擦を1段入れる、総監督裁定「割れたら厳しい側」)。
fn remove_affordance_row(
    selected: bool,
    asset_id: motolii_store::AssetId,
    card_width: Length,
    dims: Dimensions,
    colors: Colors,
) -> Option<Element<'static, Message>> {
    if !selected {
        return None;
    }

    let radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    let icon_element = motolii_icons::icon(
        motolii_icons::Icon::Delete,
        motolii_icons::frame_px_for_glyph_px(dims.micro_text),
        colors.text_muted,
    );
    let action = button(icon_element)
        .on_press(Message::RemoveAssetFromCard(asset_id))
        .width(card_width)
        .padding([dims.spacing_xs, dims.spacing_s])
        .style(move |_theme, status| chip_style(dims, colors, false, status, radius));

    Some(
        tooltip(
            action,
            container(
                text("Remove from library — undo with Cmd+Z")
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
        .into(),
    )
}

/// カードに Replace affordance を出すかのゲーティング(第6切片、map B08
/// 616/617)。**authoritative な判定は [`model::can_replace_source`]**
/// (supervisor が `Intent::SetSource` dispatch 直前に実 `Asset` で呼ぶ) —
/// ここは view 側のミラーで、`AssetListItem` が運ぶ最小面(`has_usable_path`
/// = `AssetListItem::path.is_some()`、= `Asset::path_absolute` のみ、
/// `model.rs` 冒頭 doc「EXACT TARGET #1」参照。`path_project_relative` は
/// 運ばない)から同じ形の判定をする。
///
/// **逸脱として RETURN 記載**: `path_absolute` が無く `path_project_relative`
/// だけを持つ素材はこの近似では affordance が出ない(false negative)。
/// 「押しても失敗するボタンを出す」方向の誤りより「出せる場面で一部出ない」
/// 方向の誤りを選ぶ(Q0 の安全側 — supervisor 側の authoritative な判定は
/// 正しく `path_project_relative` も見るので、実害は「ボタンが無い」だけで
/// 誤動作は起きない)。
fn replace_affordance_row(
    single_selected_layer: Option<motolii_store::LayerId>,
    has_usable_path: bool,
    asset_id: motolii_store::AssetId,
    card_width: Length,
    dims: Dimensions,
    colors: Colors,
) -> Option<Element<'static, Message>> {
    single_selected_layer?;
    if !has_usable_path {
        return None;
    }

    let radius = FILTER_CHIP_CORNER_RADIUS_ROW_HEIGHT_RATIO * dims.row_height;
    Some(
        button(text("Replace").size(dims.micro_text))
            .on_press(Message::ReplaceSelectedLayerSource(asset_id))
            .width(card_width)
            .padding([dims.spacing_xs, dims.spacing_s])
            .style(move |_theme, status| chip_style(dims, colors, false, status, radius))
            .into(),
    )
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
pub(crate) fn card_style(
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
///
/// **Q0 判定(`A07-empty.tsv` 20行目、色面+glyph が実サムネイルの代用か)**:
/// 違反ではないと判断する。根拠は Browser の視覚の正本
/// `docs/mocks-ui/public/browser-library.html` — 冒頭コメント3行目
/// 「Visual-review browser concept. It keeps the established thumbnail
/// result modes.」が、色面+glyph(`.libraryThumb thumb-blue`〜
/// `.libraryThumb thumb-cyan`、124-166行、実画像を1枚も持たない)を
/// **「確立済みのサムネイル結果モード」として明示的に採用**している。
/// つまり実データのサムネイル画像は最初からこの正本の設計に無い —
/// `thumb_fill` は「実サムネイルが無いことを一時的に隠す代用品」ではなく、
/// 正本が意図した最終形をそのまま実装している。裁定187(icon-first)とも
/// 整合する(種別を色+glyph で即答する設計は「文字で説明しない」の系)。
/// よって「触れそうで機能しない」(Q0)には該当せず、コード変更は不要。
fn thumb_fill(category: model::Category, colors: Colors) -> iced::Color {
    match category {
        model::Category::Video => colors.way_timeline,
        model::Category::Image => colors.shape,
        model::Category::Audio => colors.data,
        model::Category::Other => colors.surface_raised,
    }
}

/// 素材の欠落表示(A05 の穴閉じ、この発注の本題)。**`Unchecked`/`Present`
/// は何も返さない**(`None`)— 「在る」と偽らないが、大半を占める未確認状態を
/// 「無い」と誤警告することもしない([`AssetStatus`] doc の既定値の意味を
/// そのままカードへ持ち込む)。`Missing`/`Unreadable` の時だけ icon-first
/// (裁定187 利用者裁定)の警告 glyph を返す。
///
/// 色は既存ロール `Colors::status_warning` を流用(専用ロールを新設しない
/// 判断)。`motolii-export-pane` の cap 超過警告([`motolii_export_pane`]、
/// `lib.rs:566` 付近)が同じロールを同じ理由(危険色の専用ロールが正本
/// DTCG に無いため既存 `status.warning` を仮当てする、`motolii-tokens-rs`
/// `derive_state_colors` doc 参照)で再利用している先例に揃える。
///
/// 理由文言(「なぜ見つからないか」)は裁定185(利用者裁定「文字で説明する
/// のはクールではない」)に従いカード本体へベタ置きしない。この pane は
/// カード内に自前の status 帯を持たない(`AssetListItem`/`card_body` は
/// 一覧+grid骨格までの投影で、帯を描く箱がまだ無い) — 代わりに、この
/// crate が既に持つ最小の在庫である `tooltip`([`view_mode_button`] と
/// 同じ手口)へ理由を乗せる。tooltip はクリック可能物ではないので Q0
/// (「触れそうで機能しない」)には抵触しない。理由帯そのもの(常設テキスト
/// 行)の新設は次段(shell 側)の仕事として持ち出さない。
fn status_badge_view(
    status: &motolii_store::AssetStatus,
    dims: Dimensions,
    colors: Colors,
) -> Option<Element<'static, Message>> {
    use motolii_store::AssetStatus;

    let reason = match status {
        AssetStatus::Unchecked | AssetStatus::Present { .. } => return None,
        AssetStatus::Missing => "Missing".to_owned(),
        AssetStatus::Unreadable { reason } => reason.clone(),
    };

    let icon_element = motolii_icons::icon(
        motolii_icons::Icon::Warning,
        motolii_icons::frame_px_for_glyph_px(dims.micro_text),
        colors.status_warning,
    );

    Some(
        tooltip(
            icon_element,
            container(
                text(reason)
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
        .into(),
    )
}
