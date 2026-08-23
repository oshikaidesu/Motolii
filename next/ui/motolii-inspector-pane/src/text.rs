//! TEXT section(B46 第1切片、裁定184)。
//!
//! **持つ**: [`TextField`]/[`TextFieldDraft`](`TransformField` とは別系統 ──
//! 対象は `KeyframeTrack` ではなく `TextDocumentStyle` の静止フィールド)・
//! font/size/line-height/tracking/justify の意味と書き口
//! ([`default_text_document`]/[`applied_text_field`]/`commit_text_field`/
//! `cycle_text_justify`/`reset_text_line_height`/`reset_text_tracking`)・
//! TEXT section の view([`text_section`]/`text_field_row`/`line_height_row`/
//! `tracking_row`/`justify_row`)。
//!
//! **持たない**: 型入力→Enter の即時 field(`TextField`/[`commit_text_field`])は
//! 今も丸ごと差し替えの `Intent::SetTextDocument` を使う(据え置き — RETURN
//! 参照。二重帳簿にはならない、下記 A-1b 節)。
//!
//! ## A-1b(裁定214 同日訂正版、evaluator overlay の後継発注)
//! A-1 の懸念(「`StoreView::text_document` はこの切片が置いた
//! `PropertyId::text_style_*`/`text_justify` track を一切評価しない」)は
//! **`view.rs`(store crate)側で解決済み** — `StoreView::resolved_text_document`
//! が「track があればその値、無ければ静的値」を返すようになり、
//! `motolii-engine` の描画経路もそちらを読むよう繋ぎ直した(write-set:
//! `next/core/motolii-store/src/{view,text}.rs`/`next/engine/motolii-engine/`、
//! 詳細は該当 crate の doc・RETURN 参照)。
//!
//! **静的値と track、どちらが正本か**: **track が正本**、`TextDocumentStyle`
//! の静的フィールドは「track が無い時の既定値」——`resolved_text_document` の
//! 読み優先順位そのもの。この切片の `TextField` 経由の型入力(Enter)は
//! **今も静的値だけを書く**(据え置き)——「無ければ既定」を書く経路として
//! 残る一方、track を書く経路([`commit_text_style_track_field`]/
//! [`toggle_text_style_key`]/drag 3関数)が**別腕として追加**された。両者は
//! 同じ値を取り合わない(track が有れば読み出し側が track を勝たせる)ので
//! 二重帳簿ではない。
//!
//! **Key 列/drag はまだ crate 本体(`text_section`/`crate::Message`、`lib.rs`)
//! へ結線していない**——`color.rs`(裁定214 の同型の先行発注)が採ったのと
//! 同じ「write-set 制約下での自己完結・供覧」の形。この切片の write-set は
//! `text.rs` 1ファイルのみで、実際に mouse で drag する/Key glyph を押せる
//! ようにするには (a) `crate::Message`(`lib.rs`)への新 variant 追加と
//! (b) `motolii-shell` の window-level drag ルーティング(`Shell::update` が
//! `continue_field_drag` 相当を呼ぶ箇所)の両方が要る——どちらも write-set 外
//! (RETURN 参照)。ここに置く3関数群([`commit_text_style_track_field`]・
//! [`toggle_text_style_key`]・[`start_text_style_drag`]/[`continue_text_style_drag`]/
//! [`finish_text_style_drag`])は**track の意味そのもの
//! (`crate::transform::{single_hold_track, edited_value_track,
//! toggled_key_track, key_cell_state}`)を再実装せず再利用**した、書き口の
//! 純関数——次の結線発注はこれを呼ぶだけで良い形にしてある。

use motolii_settings_pane::chrome::{parse_number, section_header};
use motolii_store::{
    ContentKeyframe, ContentTrack, Document, FontRef, Fps, Intent, LayerId, PropertyId,
    RationalTime, TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify,
    TextStyleId, Value,
};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, column, pick_list, row as row_widget, text, text_input};
use iced::{Element, Length};

use crate::projection::TextSectionProjection;
use crate::transform::format_number;
use crate::chrome::{bordered_row, flat_button_style, name_input_style, pick_list_style, value_cell_padding};
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
///
/// **font は `FontRef::default()`(空 path)を使わない**(2026-08-22 追い発注
/// 「フォントが選べる・選ばなくても落ちない」— `lyric_text_layer_drive.rs`
/// FINDING の直接対処、[`default_font_ref`] 参照)。
pub fn default_text_style() -> TextDocumentStyle {
    TextDocumentStyle {
        id: TextStyleId(0),
        font: default_font_ref(),
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

/// 既定フォント。**必ず解決できる path を持つ** —
/// [`motolii_font_catalog::default_font`] が返すのはシステムに実在するファイル
/// だけ(その crate の契約)。文字を打った瞬間に空 path でのフォント読込へ
/// 進んで `TextShapeError::FontFile` を出し `render_frame` 全体が `Err` になる
/// 事故(`lyric_text_layer_drive.rs` モジュール doc の FINDING)は、既定値の
/// 時点でこれを塞ぐことで発生源を断つ。
///
/// システムフォントが1つも見つからない環境(この crate では検出できない・
/// 直せない)でだけ `FontRef::default()`(空 path)へ落ちる — その場合はそもそも
/// 描く文字の材料が無い環境なので、これ以上の頑健化は engine 側の仕事
/// (RETURN 参照)。
fn default_font_ref() -> FontRef {
    motolii_font_catalog::default_font()
        .map(|entry| FontRef {
            path: entry.path.clone(),
            fingerprint: None,
            family: entry.family.clone(),
            style: entry.style.clone(),
        })
        .unwrap_or_default()
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
        TextField::FontFamily => {
            next.font.family = input.to_owned();
            // 頑健化(2026-08-22 追い発注): 手打ちの family が既知のシステム
            // フォントに一致すれば path/style も一緒に追従させる —
            // 「family だけ書いて path が空/不一致のまま」という事故の再発を
            // ここでも塞ぐ。一致しない自由入力(まだ無い/仮の名前)は path を
            // **変えない**(前の値が有効な path のままなら描画は前のフォント
            // の見た目で続く — 黙って壊すより M13「無反応ゼロ」に近い)。
            if let Some(entry) = motolii_font_catalog::find_family(input) {
                next.font.path = entry.path.clone();
                next.font.style = entry.style.clone();
            }
        }
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

/// pick_list からの選択 — **family と path を同時に**書く(2026-08-22 追い
/// 発注「フォントが選べる」の主要口、`font_family_row` 参照)。手打ち欄
/// (`TextField::FontFamily` の `commit_text_field`)と違い下書きを経由しない
/// 即時操作(`CycleBlendMode` と同じ形) — 選ぶ対象が[`motolii_font_catalog::
/// system_fonts`]から選んだ1件そのものなので、確定を待つ理由(誤字の途中
/// 状態)が無い。
///
/// **カタログに無い family が来たら Document には一切触らず `Err`**
/// (頑健化の要石 — 呼び出し側のバグで options とカタログがずれても、
/// 「解決できない path」を書いてしまう経路をここで断つ)。
pub fn commit_text_font_pick(
    doc: &mut Document,
    selection: Option<LayerId>,
    family: &str,
) -> Result<(), String> {
    let entry = motolii_font_catalog::find_family(family)
        .ok_or_else(|| format!("フォントが見つからない: {family}"))?
        .clone();
    apply_text_document_edit(doc, selection, move |mut document| {
        let mut style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(default_text_style);
        style.font = FontRef {
            path: entry.path.clone(),
            fingerprint: None,
            family: entry.family.clone(),
            style: entry.style.clone(),
        };
        if document.styles.is_empty() {
            document.styles.push(style);
        } else {
            document.styles[0] = style;
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
        font_family_row(text_projection, draft, dims, colors),
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

// ---------------------------------------------------------------------------
// A-1b: Size/Line Height/Tracking の track 書き口(裁定214/217)。
//
// **まだ crate 本体(`text_section`/`crate::Message`)へ結線していない**
// (module 冒頭 doc 参照)。ここに置くのは (1) この3フィールドの `PropertyId`
// への対応 (2) 型入力(Enter)版のコミット (3) Key 列 click の意味
// (4) drag-to-scrub の3関数——**いずれも `crate::transform` の track 意味論を
// 再実装せず呼ぶだけ**(`single_hold_track`/`edited_value_track`/
// `toggled_key_track`/`key_cell_state`/`KeyCellState`、裁定215「借りる」)。
// ---------------------------------------------------------------------------

/// track へ書く3フィールド。**id 付きではない** — この section は既定行
/// (`styles[0]`、裁定98)しか編集しない現行スコープをそのまま踏襲する
/// (`TextField::Size`/`LineHeight`/`Tracking` と同じ対象)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextStyleField {
    Size,
    LineHeight,
    Tracking,
}

impl TextStyleField {
    /// この field の `PropertyId`(裁定214 の器、A-1 が `text.rs`(store crate)
    /// へ足した名前付きコンストラクタをそのまま呼ぶ)。
    pub fn property_id(self, style: TextStyleId) -> PropertyId {
        match self {
            TextStyleField::Size => PropertyId::text_style_size(style),
            TextStyleField::LineHeight => PropertyId::text_style_line_height(style),
            TextStyleField::Tracking => PropertyId::text_style_tracking(style),
        }
    }

    /// track が無い時の基準値 — **静的フィールドをそのまま読む**
    /// (`StoreView::resolved_text_document` の「track が正本、無ければ静的値」の
    /// 「無ければ」側と同じ優先順位)。Line Height の `None`(Auto)は 0.0 を
    /// 基準にする(untracked Position/Anchor の既定「未指定 = 0」と同じ判断)。
    pub fn static_value(self, style: &TextDocumentStyle) -> f64 {
        match self {
            TextStyleField::Size => style.size as f64,
            TextStyleField::LineHeight => style.line_height.unwrap_or(0.0) as f64,
            TextStyleField::Tracking => style.tracking as f64,
        }
    }
}

/// `style` を document から引く(無ければ [`default_text_style`] — `TextField`
/// 経由の既存書き口と同じ「無ければ既定」)。
fn find_text_style(document: Option<&TextDocument>, style_id: TextStyleId) -> TextDocumentStyle {
    document
        .and_then(|document| document.styles.iter().find(|style| style.id == style_id))
        .cloned()
        .unwrap_or_else(default_text_style)
}

/// track 版の下書き(`FieldDraft`/`TextFieldDraft` と同じ形 — Enter まで store に
/// 触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyleTrackDraft {
    pub style: TextStyleId,
    pub field: TextStyleField,
    pub text: String,
}

/// TEXT section の track 版コミット(`crate::transform::commit_inspector_field`
/// と同じ意味論を [`crate::transform::edited_value_track`] 経由でそのまま
/// 借りる——キー無しなら静的値書き換え、キー持ちなら playhead へのキー
/// upsert)。下書きが無い・別 field/style の submit・選択が無い、のいずれも
/// `Ok(())`。
pub fn commit_text_style_track_field(
    doc: &mut Document,
    draft: &mut Option<TextStyleTrackDraft>,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    style_id: TextStyleId,
    field: TextStyleField,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.style != style_id || taken.field != field {
        // 別の欄の submit(起こらないはずだが、安全側で下書きを戻す —
        // `commit_inspector_field`/`commit_text_field` と同じ判断)。
        *draft = Some(taken);
        return Ok(());
    }
    let Some(layer) = selection else {
        return Ok(());
    };
    let value =
        parse_number(&taken.text).ok_or_else(|| format!("数値として読めない: {}", taken.text))?;
    let property = field.property_id(style_id);
    let track = doc
        .view()
        .track(layer, &property)
        .map_err(|error| format!("track を読めない: {error}"))?;
    let new_track = crate::transform::edited_value_track(
        track.as_ref(),
        playhead_frame,
        fps,
        Value::F64(value),
    )?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("値を書けない: {error}"))
}

/// Key 列 click の意味(`crate::transform::toggled_key_track` をそのまま
/// 呼ぶ)。track が無い時の基準値は [`TextStyleField::static_value`]。
pub fn toggle_text_style_key(
    doc: &mut Document,
    selection: Option<LayerId>,
    playhead_frame: i64,
    fps: Fps,
    style_id: TextStyleId,
    field: TextStyleField,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let property = field.property_id(style_id);
    let store = doc.view();
    let track = store
        .track(layer, &property)
        .map_err(|error| format!("track を読めない: {error}"))?;
    let document = store
        .text_document(layer)
        .map_err(|error| format!("text document を読めない: {error}"))?;
    let style = find_text_style(document.as_ref(), style_id);
    let current_value = Value::F64(field.static_value(&style));
    let new_track = crate::transform::toggled_key_track(
        track.as_ref(),
        playhead_frame,
        fps,
        current_value,
    )?;
    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("キーを打てない: {error}"))
}

/// Inspector 値セルの drag-to-scrub、進行中の一時状態(`crate::transform::
/// FieldDragState` と同型 — Vec2 系を持たない分、text-style は常に scalar
/// なので `current_vec2`/`field: TransformField` は要らない)。
pub struct TextStyleDragState {
    field: TextStyleField,
    style: TextStyleId,
    layer: LayerId,
    playhead_frame: i64,
    fps: Fps,
    start_value: f64,
    origin_x: Option<f32>,
    moved: bool,
    last_value: Option<f64>,
}

/// drag 感度(px→単位、そのまま1:1)。**発明ではなく write-set の都合による
/// 複製** — `crate::transform::drag_step_per_pixel` は `TransformField` 専用の
/// private 関数なので再利用できず、text-style 用の値をここで独立に持つ
/// (RETURN 参照: 感度チューニング自体は実窓 φ 期の仕事、次発注)。
const TEXT_STYLE_DRAG_STEP: f64 = 1.0;

/// press — click か drag かはまだ未確定(`start_field_drag` と同じ形)。選択
/// なし、のいずれも黙って無視。press 時点の値は `resolved_text_document`
/// 相当(track があればその評価値、無ければ静的値)を読む——drag の起点が
/// 既にキー付き track の値と食い違う事故を防ぐ。
pub fn start_text_style_drag(
    drag: &mut Option<TextStyleDragState>,
    doc: &Document,
    selection: Option<LayerId>,
    style_id: TextStyleId,
    field: TextStyleField,
    playhead_frame: i64,
    fps: Fps,
) {
    if drag.is_some() {
        return; // 既に別の drag が進行中 — 多重起動しない
    }
    let Some(layer) = selection else {
        return;
    };
    let Ok(playhead_time) = RationalTime::try_from_frame(playhead_frame, fps) else {
        return;
    };
    let property = field.property_id(style_id);
    let store = doc.view();
    let start_value = match store.value_at(layer, &property, playhead_time) {
        Ok(Some(Value::F64(v))) => v,
        // track が無い/型不一致 — 静的値へ(裁定20 の応用)。
        _ => {
            let document = store.text_document(layer).ok().flatten();
            field.static_value(&find_text_style(document.as_ref(), style_id))
        }
    };
    *drag = Some(TextStyleDragState {
        field,
        style: style_id,
        layer,
        playhead_frame,
        fps,
        start_value,
        origin_x: None,
        moved: false,
        last_value: None,
    });
}

/// window 全体の cursor 移動(`continue_field_drag` と同じ形 — transient
/// overlay だけを毎 move 書き換える、edit timeline には触れない)。
pub fn continue_text_style_drag(
    doc: &mut Document,
    drag: &mut Option<TextStyleDragState>,
    point: iced::Point,
    fine: bool,
) {
    let Some(state) = drag.as_mut() else {
        return;
    };
    let Some(origin_x) = state.origin_x else {
        state.origin_x = Some(point.x);
        return;
    };
    let delta_px = point.x - origin_x;
    if delta_px == 0.0 && !state.moved {
        return;
    }

    let factor = if fine {
        crate::transform::DRAG_SHIFT_FACTOR
    } else {
        1.0
    };
    let new_value = state.start_value + f64::from(delta_px) * TEXT_STYLE_DRAG_STEP * factor;
    let property = state.field.property_id(state.style);
    doc.set_transient(state.layer, property, Value::F64(new_value));
    if let Some(state) = drag.as_mut() {
        state.moved = true;
        state.last_value = Some(new_value);
    }
}

/// release(`finish_field_drag` と同じ形): 実際に動いていたら最後の transient
/// 値を1回の `Intent::SetTrack`(`edited_value_track` 経由)として確定し、
/// `clear_transient` で overlay を必ず外す。
pub fn finish_text_style_drag(
    doc: &mut Document,
    drag: &mut Option<TextStyleDragState>,
) -> Result<(), String> {
    let Some(state) = drag.take() else {
        return Ok(());
    };
    let property = state.field.property_id(state.style);
    if !state.moved {
        return Ok(());
    }
    let mut write_error = None;
    if let Some(value) = state.last_value {
        let base_track = doc.view().track(state.layer, &property).ok().flatten();
        match crate::transform::edited_value_track(
            base_track.as_ref(),
            state.playhead_frame,
            state.fps,
            Value::F64(value),
        ) {
            Ok(track) => {
                if let Err(error) = doc.apply(Intent::SetTrack {
                    layer: state.layer,
                    property: property.clone(),
                    track,
                }) {
                    write_error = Some(format!("値を書けない: {error}"));
                }
            }
            Err(error) => write_error = Some(error),
        }
    }
    doc.clear_transient(state.layer, &property);
    match write_error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

#[cfg(test)]
mod track_field_tests {
    use super::*;
    use motolii_store::{Composition, LayerMeta, LayerSource, LayerTiming};

    fn doc_with_text_layer() -> (Document, LayerId) {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order: 0,
                    timing: LayerTiming {
                        start: 0,
                        duration: 100,
                        source_in: 0,
                        ..Default::default()
                    },
                },
            },
            Intent::SetTextDocument {
                layer,
                document: default_text_document(),
            },
        ])
        .unwrap();
        (doc, layer)
    }

    #[test]
    fn commit_writes_a_static_hold_track_when_no_track_exists_yet() {
        let (mut doc, layer) = doc_with_text_layer();
        let mut draft = Some(TextStyleTrackDraft {
            style: TextStyleId(0),
            field: TextStyleField::Size,
            text: "200".to_owned(),
        });
        commit_text_style_track_field(
            &mut doc,
            &mut draft,
            Some(layer),
            0,
            Fps::try_new(30, 1).unwrap(),
            TextStyleId(0),
            TextStyleField::Size,
        )
        .expect("書けるはず");
        assert!(draft.is_none());

        let value = doc
            .view()
            .value_at(layer, &PropertyId::text_style_size(TextStyleId(0)), RationalTime::ZERO)
            .unwrap();
        assert_eq!(value, Some(Value::F64(200.0)));
    }

    /// Line Height は `None`(Auto)の間、track が無ければ 0.0 を基準にする
    /// (`TextStyleField::static_value` の doc 参照)。
    #[test]
    fn line_height_commit_uses_zero_as_the_base_while_auto() {
        let (mut doc, layer) = doc_with_text_layer();
        assert_eq!(
            default_text_style().line_height,
            None,
            "既定 style は Auto(None)のはず(この試験の前提)"
        );
        toggle_text_style_key(
            &mut doc,
            Some(layer),
            0,
            Fps::try_new(30, 1).unwrap(),
            TextStyleId(0),
            TextStyleField::LineHeight,
        )
        .expect("キーを打てるはず");

        let value = doc
            .view()
            .value_at(
                layer,
                &PropertyId::text_style_line_height(TextStyleId(0)),
                RationalTime::ZERO,
            )
            .unwrap();
        assert_eq!(value, Some(Value::F64(0.0)), "Auto の基準値は 0.0 のはず");
    }

    #[test]
    fn drag_round_trip_writes_the_dragged_value_as_a_track() {
        let (mut doc, layer) = doc_with_text_layer();
        let fps = Fps::try_new(30, 1).unwrap();
        let mut drag: Option<TextStyleDragState> = None;
        start_text_style_drag(&mut drag, &doc, Some(layer), TextStyleId(0), TextStyleField::Size, 0, fps);
        assert!(drag.is_some());

        continue_text_style_drag(&mut doc, &mut drag, iced::Point::new(0.0, 0.0), false);
        continue_text_style_drag(&mut doc, &mut drag, iced::Point::new(40.0, 0.0), false);

        finish_text_style_drag(&mut doc, &mut drag).expect("確定できるはず");
        assert!(drag.is_none());

        let value = doc
            .view()
            .value_at(layer, &PropertyId::text_style_size(TextStyleId(0)), RationalTime::ZERO)
            .unwrap();
        let expected = default_text_style().size as f64 + 40.0 * TEXT_STYLE_DRAG_STEP;
        assert_eq!(value, Some(Value::F64(expected)));
    }

    #[test]
    fn drag_that_never_moves_leaves_no_track_and_no_transient_overlay() {
        let (mut doc, layer) = doc_with_text_layer();
        let fps = Fps::try_new(30, 1).unwrap();
        let mut drag: Option<TextStyleDragState> = None;
        start_text_style_drag(&mut drag, &doc, Some(layer), TextStyleId(0), TextStyleField::Size, 0, fps);
        finish_text_style_drag(&mut doc, &mut drag).expect("no-op のはず");

        assert_eq!(
            doc.view()
                .track(layer, &PropertyId::text_style_size(TextStyleId(0)))
                .unwrap(),
            None,
            "動かない drag が track を作ってしまった"
        );
    }
}

