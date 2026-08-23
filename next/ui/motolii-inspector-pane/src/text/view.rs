//! SP-4(2026-08-23) 切り出し: TEXT section の view([`text_section`]/
//! `text_field_row`/`content_row`/`size_row`/`line_height_row`/
//! `tracking_row`/`justify_row`、`super::mod` doc 参照)。**中身は無改変** —
//! 旧 `text.rs` 中間部(`text_section` から `justify_row` まで)をそのまま
//! 移送しただけ。[`super::value`] の [`TextField`]/[`TextFieldDraft`]・
//! [`super::style_track`] の [`TextStyleField`] を `use super::*;` で読む。

use motolii_settings_pane::chrome::section_header;
use motolii_store::TextJustify;
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{
    button, column, mouse_area, pick_list, row as row_widget, text, text_editor, text_input,
};
use iced::{Element, Length};

use crate::projection::TextSectionProjection;
use crate::transform::{format_number, KeyCellState};
use crate::chrome::{
    bordered_row, bordered_row_sized, flat_button_style, key_glyph_for_state, name_input_style,
    pick_list_style, value_cell_padding,
};
use crate::Message;

use super::*;

/// TEXT section: テキストレイヤー選択時のみ現れる(裁定184 型別 section 第3号)。
/// **Key 列は無い** — `TextDocumentStyle`/`TextDocument::justify` はどれも
/// `KeyframeTrack` に乗らない静止フィールド(裁定92)なので、Position/Scale
/// 行の3状態 oracle は適用対象外。Content/Font/Size/Line Height/Tracking は
/// [`speed_row`] と同じ「即時 text_input・on_submit で1回の Intent」文法、
/// Justify は [`mask_ident_row`] の mode 巡回と同じ即時操作文法 — どちらも
/// **既存の grammar の適用**であって新しい視覚言語の発明ではない(NON-GOALS)。
///
/// **Content が先頭行**(2026-08-22 発注「歌詞が入れられる道を通す」)——
/// 「文字を打つ」がこの section の主目的で、Font/Size 等はその文字の見た目を
/// 整える付随行という優先順位(利用者が最初に触る行を最初に置く)。
///
/// **塗り色(`fc`)・線色(`sc`)は [`crate::color::color_row`] で結線**
/// (同じ発注、`crate::color` module 冒頭 doc 参照 — 以前の版のこのコメントが
/// 「まだ無い editor」として見送っていた穴を今回埋めた)。`color_row` は
/// 別の pane-local `Message` 型(`crate::color::Message`)を運ぶので、
/// `.map(Message::Color)` でこの section の `Message` へ畳む
/// (`Message::Timeline`/`Message::Settings` と同じ「子 pane の Message を
/// 親が wrap する」形をこの crate 内でも踏襲)。
///
/// **`content_editor`(S4、#46)**: `Some` の間は Content 行が本物の
/// `text_editor`(複数行)を組む — 呼び出し元([`crate::view_with_content_editor`])
/// が永続 `text_editor::Content`(cursor/undo を保つ実体、`motolii_shell::Shell`
/// が所有)への参照をここまで貫通させる。**`None` は旧来どおり**(この crate の
/// 他の view 関数群/既存テストが呼ぶ `text_section` 相当の経路)— 1行
/// `text_input` の read-only フォールバックへ戻る(下 [`content_row`] 参照)。
/// 戻り値の寿命が `content_editor` の借用寿命 `'a` に縛られるのはこの1関数
/// だけ(他の行は今までどおり所有権を持つ owned String しか読まないので
/// `'static` のままでも共存できる — variance で `'static` の Element は
/// 自動的に `'a` へ収まる)。
pub(crate) fn text_section<'a>(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    color_draft: Option<&crate::color::ColorFieldDraft>,
    content_editor: Option<&'a text_editor::Content>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    column![
        section_header("TEXT", dims, colors),
        content_row(
            text_projection.content.clone(),
            draft,
            content_editor,
            dims,
            colors,
        ),
        font_family_row(text_projection, draft, dims, colors),
        size_row(text_projection, draft, dims, colors),
        line_height_row(text_projection, draft, dims, colors),
        tracking_row(text_projection, draft, dims, colors),
        justify_row(text_projection.justify, dims, colors),
        crate::color::color_row(
            crate::color::ColorTarget::Fill,
            &text_projection.style,
            color_draft,
            dims,
            colors,
        )
        .map(Message::Color),
        crate::color::color_row(
            crate::color::ColorTarget::Stroke,
            &text_projection.style,
            color_draft,
            dims,
            colors,
        )
        .map(Message::Color),
    ]
    .into()
}

/// TEXT section の text_input 行の共通形(`speed_row` の value_field と同じ
/// 組み方)。下書きがあればそれを、無ければ投影の確定値を表示する。
fn text_field_row(
    label: &'static str,
    field: TextField,
    current: String,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == field)
        .map(|d| d.text.clone())
        .unwrap_or(current);

    // 裁定170 M01: fork の text_input は借用寿命を返り値に縛るため owned move
    // (`speed_row`/`ident_band` と同じ回避)。
    let value_field = text_input("", displayed)
        .on_input(move |text| Message::TextFieldInput(field, text))
        .on_submit(Message::TextFieldSubmit(field))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text(label)
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Content 行(S4、#46 の穴塞ぎ)。**`content_editor` が `Some` の間だけ本物の
/// 複数行 `text_editor` を組む**——`None`(この crate の他の呼び出し元・
/// `text_section` の既存テストが通る経路)は完全に旧来どおりの1行
/// `text_field_row("Content", ...)` へフォールバックする(挙動もテストも
/// 無改修)。
///
/// **なぜ2つの経路が要るか**: `text_editor::new(&'a Content)` は `Content` の
/// 借用寿命 `'a` をそのまま widget の寿命として運ぶため、呼び出し元が
/// **永続する**(1フレームで消えない)`text_editor::Content` を持っていない
/// 限り組めない(`motolii_shell::Shell::inspector_content_editor` がその
/// 永続実体、`view_with_content_editor` から `Some(&editor)` で貫通させる)。
/// この crate の素の `view`/`view_with_speed_draft`/`view_with_text_draft`/
/// `view_with_color_draft`(既存テストが直接呼ぶ)はそのような永続実体を
/// 持たない呼び出し元なので `None` を渡し続ける — その経路では今までどおり
/// 1行入力のまま(表示上の後退はない、Content は元々1行入力だった)。
fn content_row<'a>(
    current: String,
    draft: Option<&TextFieldDraft>,
    content_editor: Option<&'a text_editor::Content>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let Some(editor_content) = content_editor else {
        return text_field_row("Content", TextField::Content, current, draft, dims, colors);
    };

    // 複数行ぶんの高さ(`inspector_value_width * 3.0` と同じ「既存トークンの
    // 算術合成」慣習 — 専用トークンを新規発明しない)。3行分あれば2行の
    // 歌詞タイトル+サブタイトルは常に見える。
    let row_height = dims.inspector_row_height * 3.0;
    let editor = text_editor(editor_content)
        .on_action(Message::ContentEditorAction)
        .key_binding(content_key_binding)
        .placeholder("歌詞をここに(Enter=改行、⌘/Ctrl+Enter=確定)")
        .size(dims.body_text)
        .padding(value_cell_padding(dims))
        .height(Length::Fixed(row_height))
        .style(move |_theme, status| content_editor_style(dims, colors, status));

    // 他の行と違い、ラベルは値欄と同じ固定幅ではなく自然幅(`Shrink`)——
    // 数値欄(64px 固定)と違い editor は行の残り幅を全部使う方が2行の歌詞が
    // 見える(他行の狭い固定幅を Content にも押し付けない)。
    let content = row_widget![
        text("Content")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Shrink),
        Element::from(editor),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Top);

    bordered_row_sized(content.into(), dims, row_height)
}

/// [`crate::chrome::name_input_style`] の `text_editor` 版(型が別 —
/// `text_editor::Status`/`Style` は `text_input` のそれとフィールド形は
/// 同じだが独立した型)。同じ配色規則をそのまま複製するだけ(2箇所で別の
/// 意匠を発明しない、crate 全体の慣習)。
fn content_editor_style(
    dims: Dimensions,
    colors: Colors,
    status: text_editor::Status,
) -> text_editor::Style {
    let (background, border_color) = match status {
        text_editor::Status::Focused { .. } => (colors.surface_app, colors.action_active),
        text_editor::Status::Hovered => (colors.surface_hover, colors.border_default),
        _ => (iced::Color::TRANSPARENT, iced::Color::TRANSPARENT),
    };
    text_editor::Style {
        background: iced::Background::Color(background),
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

/// `text_editor` の key binding(S4、#46)。**Cmd/Ctrl+Enter だけを横取りして
/// 確定にする** — 素の Enter/Shift+Enter 等その他すべてのキー押下は
/// [`text_editor::Binding::from_key_press`](既定挙動、Enter=改行を含む)へ
/// そのまま委譲する。Slack/Linear/GitHub の複数行コメント欄と同じ
/// 「Enter=改行、Cmd/Ctrl+Enter=送信」文法(`applied_text_content` doc
/// 「Enter と確定の割り振り」節、出典参照)。
fn content_key_binding(press: text_editor::KeyPress) -> Option<text_editor::Binding<Message>> {
    let is_enter = press.modified_key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter);
    if is_enter && press.modifiers.command() {
        return Some(text_editor::Binding::Custom(Message::ContentEditorCommit));
    }
    text_editor::Binding::from_key_press(press)
}

/// **D-1 結線(2026-08-23)**: `TextField::Size`/`LineHeight`/`Tracking` の
/// Enter 確定を track 書き口(`commit_text_style_track_field`)へ渡すべき物か
/// (`Content`/`FontFamily` はそのまま静的値の口 [`commit_text_field`] を使う)。
/// **`Shell::update_inspector` の `TextFieldSubmit` 腕がこれで分岐する**
/// (write-set 外の `text_section`/`view` を一切改修せずに track 化するための
/// 唯一の橋 — `TextFieldDraft` は Content/FontFamily と共有のまま、Shell 側で
/// 一時的に [`TextStyleTrackDraft`] へ包み替えて `commit_text_style_track_field`
/// を呼ぶ、詳細は RETURN 参照)。
pub fn text_field_track_target(field: TextField) -> Option<TextStyleField> {
    match field {
        TextField::Size => Some(TextStyleField::Size),
        TextField::LineHeight => Some(TextStyleField::LineHeight),
        TextField::Tracking => Some(TextStyleField::Tracking),
        TextField::Content | TextField::FontFamily => None,
    }
}

/// Size 行。`text_field_row` と同じ text_input(Enter は
/// [`text_field_track_target`] 経由で `commit_text_style_track_field` が書く
/// ——static ではなく track、A-1b が意図した「track が正本」の優先順位)+
/// drag ハンドル([`text_style_drag_handle`])+ Key 列
/// ([`text_style_key_button`])。**投影(`TextSectionProjection`)は
/// `resolved_text_document` 経由**(E-3、2026-08-23 訂正)なので、drag 中の
/// transient 値もこの表示に即時反映される。
fn size_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::Size)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| format_number(text_projection.size as f64, 1));

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::Size, text))
        .on_submit(Message::TextFieldSubmit(TextField::Size))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Size")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        text_style_drag_handle(TextStyleField::Size, dims, colors),
        text_style_key_button(TextStyleField::Size, text_projection.size_key, dims, colors),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// drag ハンドル(`↕`)。**値欄そのものには重ねない** — `text_input` が press
/// を own するため、同じ矩形に `mouse_area` を重ねても drag の press が
/// text_input に食われる(write-set 制約下の簡略化、`chrome.rs::
/// draggable_value_cell` の hover-swap 機構は `chrome.rs` が write-set 外の
/// ため複製できない)。press だけを持つ(move/release は window 全体購読
/// 経由で `Shell.inspector_text_style_drag` が追う、`FieldDragState` と同型)。
fn text_style_drag_handle(field: TextStyleField, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    mouse_area(
        text("\u{2195}")
            .size(dims.caption_text)
            .color(colors.text_muted),
    )
    .interaction(iced::mouse::Interaction::ResizingHorizontally)
    .on_press(Message::TextStyleValuePressed(field))
    .into()
}

/// Key 列ボタン。**3状態 oracle の表示は Position/Scale 行と共通**
/// (`KeyCellState::{Static,Between,AtKey}` に応じた ◇/◆薄/◆濃 の出し分け ──
/// `crate::chrome::key_glyph_for_state` が `crate::chrome::key_glyph`
/// [`TransformRowProjection`/`KeyCellProjection` 経由]と同じ視覚を描く、
/// E-3・2026-08-23 で結線)。click の**意味**は3状態 oracle と同じ
/// ([`toggle_text_style_key`] が `crate::transform::toggled_key_track` を
/// そのまま呼ぶ)——`Message` だけが `KeyRow` ではなく `TextStyleField` を
/// 運ぶ別腕([`crate::projection::TextSectionProjection`] struct doc 参照)。
fn text_style_key_button(
    field: TextStyleField,
    state: KeyCellState,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    key_glyph_for_state(state, Message::TextStyleKeyPressed(field), dims, colors)
}

/// Font 行。手打ち欄(`text_input`、`TextField::FontFamily` — 既存文法、
/// `applied_text_field` 参照)に加えて、実在するシステムフォントから選ぶ
/// `pick_list` を並べる(2026-08-22 追い発注「フォントが選べる・選ばなくても
/// 落ちない」)。選ぶと [`Message::PickFont`] → [`commit_text_font_pick`] が
/// family と path を**同時に**書く — 手打ち欄だけでは `FontRef::path` を
/// 編集する手段が無かった穴(`lyric_text_layer_drive.rs` FINDING)への直接の
/// 対処。
///
/// **pick_list は次/ に前例が無い**(BL2 は blend/mask mode のような小さい
/// 固定集合の巡回ボタン採用の決定 — `attrs.rs`/`mask.rs` 冒頭 doc 参照)が、
/// フォント一覧は開放的で数十件になり得る集合なので同じ理由が当てはまらない
/// (数十件を巡回ボタンで1つずつ送るのは Q0「触れそうで触れない」寄りの手触り
/// になる)。本発注が pick_list を名指ししているのはこの区別に基づく —
/// BL2 の対象外として扱う。
fn font_family_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::FontFamily)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| text_projection.font_family.clone());

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::FontFamily, text))
        .on_submit(Message::TextFieldSubmit(TextField::FontFamily))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    // options はカタログの family 一覧そのまま([`motolii_font_catalog::
    // system_fonts`] が既に family ごとに一意 — 重複除去済み)。選択中の値は
    // 「カタログに実在する family の時だけ」ハイライトさせる(手打ちで
    // カタログに無い自由文字列を打った直後に、pick_list が無関係な項目を
    // ハイライトして見えるのを避ける)。
    let options: Vec<String> = motolii_font_catalog::system_fonts()
        .iter()
        .map(|entry| entry.family.clone())
        .collect();
    let current_family = text_projection.font_family.clone();
    let selected = options.contains(&current_family).then_some(current_family);
    // 引数順は `pick_list(selected, options, to_string)`(helpers.rs 実測 —
    // options を先に置く直感とは逆順)。選択の通知は `.on_select`(payload は
    // そのまま `String`、`Message::PickFont` を直接渡せる — `.map(Message::
    // Color)` と同じ「バリアントを関数として渡す」形)。
    let picker = pick_list(selected, options, |family: &String| family.clone())
        .on_select(Message::PickFont)
        .text_size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .placeholder("Pick…")
        .style(move |_theme, status| pick_list_style(dims, colors, status));

    let content = row_widget![
        text("Font")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        picker,
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Line Height 行。`None`(Auto)は「Auto」文字列で表示し、`Auto` ボタンで
/// 明示的に戻せる(`speed_row` の Reset ボタンと同じ即時操作文法。map
/// 「Auto leading for selected text」、採用済)。
fn line_height_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::LineHeight)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| {
            text_projection
                .line_height
                .map(|value| format_number(value as f64, 1))
                .unwrap_or_else(|| "Auto".to_owned())
        });

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::LineHeight, text))
        .on_submit(Message::TextFieldSubmit(TextField::LineHeight))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Line Height")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        button(text("Auto").size(dims.caption_text))
            .on_press(Message::ResetLineHeightAuto)
            .style(move |_theme, status| flat_button_style(colors, status)),
        text_style_drag_handle(TextStyleField::LineHeight, dims, colors),
        text_style_key_button(TextStyleField::LineHeight, text_projection.line_height_key, dims, colors),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Tracking 行。`Reset` ボタンは 0 でも常に出す(`speed_row` の Reset と同じ
/// 「無反応ゼロより一貫を優先」判断。map「Reset tracking to 0」、採用予定)。
fn tracking_row(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|d| d.field == TextField::Tracking)
        .map(|d| d.text.clone())
        .unwrap_or_else(|| format_number(text_projection.tracking as f64, 1));

    let value_field = text_input("", displayed)
        .on_input(|text| Message::TextFieldInput(TextField::Tracking, text))
        .on_submit(Message::TextFieldSubmit(TextField::Tracking))
        .size(dims.body_text)
        .width(Length::Fixed(dims.inspector_value_width))
        .padding(value_cell_padding(dims))
        .align_x(iced::alignment::Horizontal::Center)
        .style(move |_theme, status| name_input_style(dims, colors, status));

    let content = row_widget![
        text("Tracking")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        value_field,
        button(text("Reset").size(dims.caption_text))
            .on_press(Message::ResetTracking)
            .style(move |_theme, status| flat_button_style(colors, status)),
        text_style_drag_handle(TextStyleField::Tracking, dims, colors),
        text_style_key_button(TextStyleField::Tracking, text_projection.tracking_key, dims, colors),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

/// Justify(揃え)行。`mask_ident_row` の mode 巡回ボタンと同じ即時操作文法 —
/// 表示は `TextJustify` の `Debug`(`Left`/`Right`/`Center` — blend/mask の
/// 表示と同じ流儀)。
fn justify_row(justify: TextJustify, dims: Dimensions, colors: Colors) -> Element<'static, Message> {
    let content = row_widget![
        text("Justify")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        button(text(format!("{justify:?}")).size(dims.body_text))
            .on_press(Message::CycleTextJustify)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}
