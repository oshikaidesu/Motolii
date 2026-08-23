//! SP-4(2026-08-23) 切り出し: [`TextField`]/[`TextFieldDraft`] の識別と、
//! font/size/line-height/tracking/justify/content の**静的値**の意味・書き口
//! (`super::mod` doc 参照)。**中身は無改変** — 旧 `text.rs` 冒頭
//! (`TextField` enum から `reset_text_tracking` まで)をそのまま移送しただけ。

use motolii_settings_pane::chrome::parse_number;
use motolii_store::{
    ContentKeyframe, ContentTrack, Document, FontRef, Intent, LayerId, RationalTime,
    TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId,
};

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
/// ## 複数行の扱い(S4、裁定222 — 外部資料で決めた、#46 の穴塞ぎ)
/// **1行の `text_input` は捨てて `iced::widget::text_editor`(複数行 widget)を
/// 採用し、Enter は改行にする**(この関数=`ContentTrack` の書き先自体は
/// 変わらない、旧 doc の予告どおり)。
///
/// **出典と判断根拠**: After Effects のテキストレイヤーは Composition
/// パネルで直接編集中に Return/Enter を押すと**改行**が入る(段落テキスト
/// ボックス内で新しい行を作る、Adobe After Effects ユーザーガイド
/// 「テキストの作成」章の記載どおり)。Premiere Pro/Lottie も同型 —
/// Lottie の `text-document`(`next/reference/lottie-coverage.tsv` text 群)
/// は content 文字列を単一の値として持ち、`lh`(Line Height、`text-document
/// lh` 行「行送り」)は複数行の行間を制御するためだけに存在する語彙で、
/// 複数行そのものは「文字列に改行文字を含める」以外の表現を持たない
/// (character-data/character-precomp 系は「不採用」= 1文字ごとに輪郭を
/// 焼く経路で、この設計では採らない)。**つまり Lottie の正本語彙は
/// 「content 文字列に `\n` を含められること」を前提にしており、それを
/// 満たせない1行 text_input が逸脱だった**(engine 側
/// `next/engine/motolii-vector/src/text.rs`/`next/engine/motolii-engine/
/// src/text.rs` は `\n` 分割・`lh` 行間の両方をとうに実装済み —
/// `next/engine/motolii-vector/tests/text.rs`/
/// `next/engine/motolii-engine/src/text.rs` の `line_height_from_style_...`
/// テストが証拠。穴は UI 側の入力手段だけだった)。
///
/// **Enter と確定の割り振り**: Enter(素の Return)は行分割の1文字 `\n` を
/// 挿す(`text_editor` 既定の [`Binding::Enter`])。確定(1回の
/// `Intent::SetTextDocument` を書く)は**Cmd/Ctrl+Enter**
/// (`crate::text::content_key_binding` が `KeyPress` を横取りする唯一の
/// chord)——Slack/Linear/GitHub の PR 説明欄など「Enter=改行、
/// Cmd/Ctrl+Enter=送信」という広く使われる文法をそのまま踏襲(新しい文法の
/// 発明ではなく既存文法の輸入、発注書「新しい文法を発明しない」の対象は
/// **地図に無い記号の発明**であって既存の複数行入力欄の慣習を指すのではない
/// と判断)。**マウス完遂路**は「他レイヤーへ選択を移す」——
/// `motolii_shell::Shell::sync_inspector_content_editor` が選択が変わる
/// 直前に未確定の下書きを自動で1回 `commit_text_field` する(blur-commit、
/// クリックだけで確定できる)。**キーボード完遂路**は Cmd/Ctrl+Enter
/// (裁定216「各意図にマウス完遂路とキーボード完遂路の両方を要求」を満たす)。
///
/// 歌詞動画で2行以上を1レイヤーへ入れられるようになったため、「1行=1
/// レイヤーに分ける」の迂回は**もう必須ではない**(引き続き選べる代替では
/// ある — 行ごとに別々にアニメーションさせたい場合はレイヤーを分ける方が
/// 正しい)。
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
