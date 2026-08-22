//! TEXT section(B46 第1切片、裁定184)。
//!
//! **持つ**: [`TextField`]/[`TextFieldDraft`](`TransformField` とは別系統 ──
//! 対象は `KeyframeTrack` ではなく `TextDocumentStyle` の静止フィールド、
//! 裁定92)・font/size/line-height/tracking/justify の意味と書き口
//! ([`default_text_document`]/[`applied_text_field`]/`commit_text_field`/
//! `cycle_text_justify`/`reset_text_line_height`/`reset_text_tracking`)・
//! TEXT section の view([`text_section`]/`text_field_row`/`line_height_row`/
//! `tracking_row`/`justify_row`)。
//!
//! **持たない**: `TextDocumentStyle` は裁定92によりキーフレーム化しない(v1)
//! ので、[`crate::transform`] の値セル文法(`PropertyId`/`Intent::SetTrack`)
//! には乗らない ── 丸ごと差し替えの `Intent::SetTextDocument` を使う。

use motolii_settings_pane::chrome::{parse_number, section_header};
use motolii_store::{
    ContentKeyframe, ContentTrack, Document, FontRef, Intent, LayerId, RationalTime,
    TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId,
};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, column, row as row_widget, text, text_input};
use iced::{Element, Length};

use crate::projection::TextSectionProjection;
use crate::transform::format_number;
use crate::chrome::{bordered_row, flat_button_style, name_input_style, value_cell_padding};
use crate::Message;

/// TEXT section の text_input 系フィールドの識別。**`TransformField` とは
/// 別の enum にする** — 対象が `KeyframeTrack`(`property_id`/
/// `commit_inspector_field`/drag-to-scrub の経路)ではなく `TextDocumentStyle`
/// の静止フィールド(裁定92)なので、track を前提にした既存の型に無理に
/// 押し込まない。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextField {
    /// `text-document t`(Text、本文)。**他3腕とは書き先が違う** —
    /// `TextDocumentStyle` ではなく `TextDocument::content`(`ContentTrack`)
    /// 直下を書く(2026-08-22 発注「歌詞が入れられる道を通す」— 歌詞動画/MV
    /// ペルソナ致命的欠落〈TEXT section に文字列本体の入力欄が無い〉への対処、
    /// `docs/reviews/2026-08-22-persona-lyric-mv.md` 参照)。`commit_text_field`
    /// がこの1腕だけ特別扱いする(下記 doc 参照)。
    Content,
    /// `text-document f`(Font Family)。`FontRef::family` だけを書き換える
    /// (`path`/`fingerprint`/`style` はこの切片では触らない)。
    FontFamily,
    /// `text-document s`(Font Size)。
    Size,
    /// `text-document lh`(Line Height)。`None` = Auto。
    LineHeight,
    /// `text-document tr`(Tracking)。
    Tracking,
}

/// TEXT section 入力欄の下書き。**Document ではない**(`FieldDraft` と同型 —
/// commit(Enter)まで store に触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct TextFieldDraft {
    pub field: TextField,
    pub text: String,
}

// ---------------------------------------------------------------------------
// TEXT section(B46 第1切片、裁定184)— font/size/line-height/tracking/justify
// の意味と書き口。`TextDocumentStyle` は裁定92によりキーフレーム化しない
// (v1)ので、MASK opacity と違い `PropertyId`/`Intent::SetTrack` には乗らない
// — 丸ごと差し替えの `Intent::SetTextDocument`(`SetMasks` と同じ形)を使う。
// ---------------------------------------------------------------------------

/// text_document が未着手の layer に**表示専用**で見せる既定値。**保存しない**
/// — [`apply_text_document_edit`] がここから編集後コピーを作り、実際に値が
/// 変わった時だけ `Intent::SetTextDocument` を出す(`default_vec2` と同じ
/// 「無ければ既定」の形)。
pub fn default_text_document() -> TextDocument {
    TextDocument {
        content: ContentTrack::new(),
        // Lottie/AE とも既定は左揃え。
        justify: TextJustify::Left,
        wrap_size: None,
        styles: vec![default_text_style()],
        slot_id: None,
        ranges: Vec::new(),
        alignment: TextAlignmentOptions::default(),
        runs: Vec::new(),
    }
}

/// スタイル表の既定行(裁定98: `styles[0]` = document 既定値)。この切片は
/// この1行だけを編集する(範囲スタイル表・アニメーターは次切片)。
pub fn default_text_style() -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: FontRef::default(),
        size: 100.0,
        fill: [0.0, 0.0, 0.0, 1.0],
        line_height: None,
        tracking: 0.0,
        stroke_color: None,
        stroke_width: 0.0,
        stroke_over_fill: false,
        axes: Vec::new(),
        features: Vec::new(),
    }
}

/// Justify 巡回ボタンの次の値。`TextJustify` の宣言順(Left → Right →
/// Center)をそのまま辿る(`next_mask_mode`/`next_blend_mode` と同じ
/// 「型の宣言順を巡回順の正本にする」判断)。
pub fn next_text_justify(current: TextJustify) -> TextJustify {
    match current {
        TextJustify::Left => TextJustify::Right,
        TextJustify::Right => TextJustify::Center,
        TextJustify::Center => TextJustify::Left,
    }
}

/// 下書き文字列を [`TextField`] の意味で `style` へ適用した新しいコピー
/// (`next_value` の Vec2 保存と同じ「他フィールドは保つ」考え方)。数値として
/// 読めない入力は `Err` の理由文(M13)。`FontFamily` だけは数値変換をしない
/// — 空文字列も許す(「フォント未指定」を表現できる、`FontRef` の既定と同型)。
///
/// **`TextField::Content` はここへ来ない** — 書き先が `TextDocumentStyle`
/// ではなく `TextDocument::content` 直下([`applied_text_content`] 参照)なので、
/// [`commit_text_field`] がこの関数を呼ぶ前に分岐して弾く。ここに来たら
/// 呼び出し側のバグ(`commit_text_field` の分岐漏れ)なので `unreachable!` で
/// 安全側に倒す(黙って空文字へ倒すと M13「無反応ゼロ」に反する)。
pub fn applied_text_field(
    style: &TextDocumentStyle,
    field: TextField,
    input: &str,
) -> Result<TextDocumentStyle, String> {
    let mut next = style.clone();
    match field {
        TextField::Content => {
            unreachable!("TextField::Content は commit_text_field が先に分岐して弾く")
        }
        TextField::FontFamily => next.font.family = input.to_owned(),
        TextField::Size => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.size = value as f32;
        }
        TextField::LineHeight => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.line_height = Some(value as f32);
        }
        TextField::Tracking => {
            let value =
                parse_number(input).ok_or_else(|| format!("数値として読めない: {input}"))?;
            next.tracking = value as f32;
        }
    }
    Ok(next)
}

/// [`TextDocument::content`] の**現在値**を読む(表示専用)。裁定92の対象外
/// (Content は `TextDocumentStyle` の静止フィールドではなく `TextDocument`
/// 直下の Hold-keyed トラック)だが、この切片は v1 の「打鍵→Enter で1回の
/// `Intent`」文法をそのまま流用するため、**アニメーション(複数キー)は
/// 作らない** — 常に [`RationalTime::ZERO`] で評価する(`t` に関わらず同じ
/// 文字列を返す、`[`applied_text_content`]` が常に単一キーしか書かないことの
/// 対称)。将来の「歌詞を時間で切り替える」機能(range-selector/animator、
/// text.rs crate doc 「第2/3切片」参照)は Content 欄とは別の入口になる。
pub fn text_document_content(document: &TextDocument) -> String {
    document.content.eval(RationalTime::ZERO).to_owned()
}

/// 下書き文字列を新しい `content`(`ContentTrack`)へ適用する。**単一の Hold
/// キー(t=0)で丸ごと差し替える** — 複数行/改行はここでは分割しない
/// (`content` 文字列そのものに `\n` が含まれていれば cosmic-text 側が行分割
/// する、`motolii-vector::text::shape_text` doc「改行はそのまま行分割」参照)。
///
/// ## 複数行の扱い(発注の未決事項、ここで決めた)
/// **1行の `text_input`(on_submit で確定、他の TEXT section 欄と同じ文法)を
/// 採用し、Enter は改行ではなく確定にする** — この Inspector の全ての
/// text_input(Name/Speed/Font/Size/Line Height/Tracking)が同じ「Enter=確定」
/// 文法なので、Content だけ「Enter=改行」にすると同じ widget が欄によって
/// 違う意味を持つことになり(Q0 一貫性違反)、新しい編集文法を発明しないと
/// いう発注の指示にも反する。歌詞動画で2行以上を同時に見せたい場合は
/// **レイヤーを複数(1行=1 text layer)に分ける**のが現状の道 —
/// `next/DECISIONS.md` の「Split」等と同じく「1操作の代わりに複数レイヤーで
/// 迂回できる」形(`docs/reviews/2026-08-22-persona-lyric-mv.md` 工程4の
/// 迂回可能表と同じ考え方)。将来 `text_editor`(iced の複数行 widget)を
/// 導入する日が来ても、この関数(`ContentTrack` 書き込みの形)自体は変わらない
/// ── 変わるのは view 側の widget 選択だけ。
pub fn applied_text_content(document: &TextDocument, input: &str) -> TextDocument {
    let mut next = document.clone();
    let mut content = ContentTrack::new();
    content.insert(ContentKeyframe {
        t: RationalTime::ZERO,
        content: input.to_owned(),
    });
    next.content = content;
    next
}

/// TEXT section 共通の書き口(`apply_mask_list_edit` と同型): 選択が無ければ
/// no-op、選択層の `TextDocument` を読み(無ければ [`default_text_document`])、
/// `edit` で編集後コピーを作り、**実際に値が変わった時だけ**1回の
/// `Intent::SetTextDocument` を出す(決定7「同値は Undo を積まない」と同じ
/// 判断 — Reset ボタンが既に既定値の時・打鍵で同値を submit した時の両方を
/// この1箇所で満たす)。
fn apply_text_document_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    edit: impl FnOnce(TextDocument) -> Result<TextDocument, String>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let current = doc
        .view()
        .text_document(layer)
        .map_err(|error| format!("text document を読めない: {error}"))?;
    let base = current.clone().unwrap_or_else(default_text_document);
    let next = edit(base)?;
    let unchanged = match &current {
        Some(existing) => existing == &next,
        None => next == default_text_document(),
    };
    if unchanged {
        return Ok(());
    }
    doc.apply(Intent::SetTextDocument {
        layer,
        document: next,
    })
    .map_err(|error| format!("text document を書けない: {error}"))
}

/// TEXT section の text_input 系フィールド — 下書きを確定して1回の
/// `Intent::SetTextDocument` を出す(1 gesture = 1 undo、`commit_inspector_field`
/// と同じ形)。下書きが無い・別 field の submit・選択が無い、のいずれも
/// `Ok(())`(何もしない)。
///
/// **`Content` はここで分岐する** — 書き先が `TextDocumentStyle`(`styles[0]`)
/// ではなく `TextDocument::content` 直下なので、[`applied_text_field`]
/// (style 専用)を経由せず [`applied_text_content`] を直接呼ぶ。
pub fn commit_text_field(
    doc: &mut Document,
    draft: &mut Option<TextFieldDraft>,
    selection: Option<LayerId>,
    field: TextField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.field != field {
        // 別の field の submit(起こらないはずだが、安全側で下書きを戻す —
        // `commit_inspector_field` と同じ判断)。
        *draft = Some(taken);
        return Ok(());
    }
    if field == TextField::Content {
        return apply_text_document_edit(doc, selection, |document| {
            Ok(applied_text_content(&document, &taken.text))
        });
    }
    apply_text_document_edit(doc, selection, |mut document| {
        let style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(default_text_style);
        let new_style = applied_text_field(&style, field, &taken.text)?;
        if document.styles.is_empty() {
            document.styles.push(new_style);
        } else {
            document.styles[0] = new_style;
        }
        Ok(document)
    })
}

/// Justify の巡回 — 即1回の `Intent::SetTextDocument`(`CycleBlendMode`/
/// `CycleMaskMode` と同じ即時操作の形)。選択なしは黙って no-op。
pub fn cycle_text_justify(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        document.justify = next_text_justify(document.justify);
        Ok(document)
    })
}

/// Line Height を Auto(`None`)へ戻す(`ResetSpeed` と同じ即時操作の形)。
/// styles が空のまま(まだ何も書かれていない)なら既に Auto —
/// [`apply_text_document_edit`] の同値判定が no-op にする。
pub fn reset_text_line_height(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        if let Some(style) = document.styles.first_mut() {
            style.line_height = None;
        }
        Ok(document)
    })
}

/// Tracking を 0 へ戻す(map「Reset tracking to 0」、[`reset_text_line_height`]
/// と同型)。
pub fn reset_text_tracking(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    apply_text_document_edit(doc, selection, |mut document| {
        if let Some(style) = document.styles.first_mut() {
            style.tracking = 0.0;
        }
        Ok(document)
    })
}
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
pub(crate) fn text_section(
    text_projection: &TextSectionProjection,
    draft: Option<&TextFieldDraft>,
    color_draft: Option<&crate::color::ColorFieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    column![
        section_header("TEXT", dims, colors),
        text_field_row(
            "Content",
            TextField::Content,
            text_projection.content.clone(),
            draft,
            dims,
            colors,
        ),
        text_field_row(
            "Font",
            TextField::FontFamily,
            text_projection.font_family.clone(),
            draft,
            dims,
            colors,
        ),
        text_field_row(
            "Size",
            TextField::Size,
            format_number(text_projection.size as f64, 1),
            draft,
            dims,
            colors,
        ),
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

