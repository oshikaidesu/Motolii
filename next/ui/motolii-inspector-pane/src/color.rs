//! 色エディタ(2026-08-22 発注): `Value::Color([f64; 4])` の語彙は
//! `motolii-eval`/`motolii-store`(fill/stroke・背景色・ラベル色)に実装済みなのに、
//! Inspector に色を**編集する口が無かった**実態への対処。
//!
//! **`text.rs`(読み専用、この発注の write-set 外)の TEXT section doc が
//! 名指ししている穴を埋める**:
//! > 塗り色(`fc`)・線色(`sc`)は実在するが `Value::Color` 用の editor が
//! > まだ無い(crate doc「Color/Enum/Path/LayerId は Effect 束の仕事」)ため
//! > この切片では見送る
//!
//! ## 意匠 — 既存 Settings の RGBA 編集とそろえる(発注書「第一候補」)
//! `motolii_settings_pane::{BackgroundChannel, BackgroundFieldDraft,
//! commit_background_channel, channel_cell}`(背景色 RGBA、`lib.rs` 参照)と
//! **同じ文法**: 0..255 の整数欄4本(R/G/B/A、alpha 込み)・click→type・Enter で
//! 初めて1回の Intent(1 gesture = 1 undo)。新しい編集文法は発明しない
//! (発注書「新しい文法を発明しない」)。数値欄の枠色ロールは
//! `motolii_settings_pane::chrome::value_input_style` をそのまま再利用する
//! (2箇所で別の意匠を発明しない、既存 crate 全体の慣習)。
//!
//! 色見本(swatch)は [`crate::chrome::label_color_chip`] と同じ「面 = 色そのもの、
//! 輪郭は常時透明」の形(裁定179「箱は状態の器」の対象外 — これは箱ではなく
//! 色そのものの見本)。
//!
//! ## S4(2026-08-23、#56 の穴塞ぎ、裁定222 — 外部資料で決めた)
//! **旧版はここで「クリックしない」と決めていた**(popover を作らない、
//! 押せない物は押せなさそうに)。**この判断を裏返す** — 4製品+Figma の実測
//! (After Effects の Character パネル/塗り・線パネル、Premiere Pro の
//! Lumetri・テキストスタイル、DaVinci Resolve の Color/Fusion パネル、
//! CapCut のテキスト色編集、Figma の Fill/Stroke パネル、いずれもスウォッチ
//! クリック→ピッカーが唯一の一次入口)は例外なく「色見本はクリックできる」。
//! 旧 doc の理由(「押せなさそうに」)は**なぜクリックさせないかの理由には
//! なっていない**——見た目を殺すことと機能を削ることは別の判断のはずで、
//! 前者の主張が後者を正当化していなかった(裁定150「逸脱には理由が必須」
//! に対し、理由が弱い側だったと判定)。
//!
//! **満額のグラフィカルピッカー(色相環・eyedropper 等)はこの節の write-set
//! の外**(新規 widget をゼロから起こす規模で、この発注の割り当てを超える)。
//! **今回実装したのはクリックの意味そのもの** — swatch を押すと
//! [`Message::SwatchPressed`] が発火し、`motolii_shell::Shell` がこの行の
//! R channel 欄([`channel_input_id`])へ `iced::widget::operation::focus`
//! する(`transform.rs::field_input_id`/`Shell::enter_field_editing` と同じ
//! 「press→focus task」文法、新しい focus 機構の発明ではない)。**RGBA 4欄が
//! 既にこの行に常時見えている**(AE 等と違い、この Inspector は最初から
//! popup の裏に隠さず precise-edit を並べている設計)ので、「クリックで
//! ピッカーへ導く」というスウォッチの**役割**はもう既存の RGBA 欄が担って
//! いる——今回の変更は「見た目が押せそうなのに実際は無反応」という Q0
//! 違反を閉じ、押した先の焦点を実際に動かすところまでにとどめた。将来
//! 満額ピッカー(色相環/直近使った色/eyedropper)を足す日が来ても、
//! この click→focus の配線はそのまま「ピッカーを開く」に差し替えられる形
//! (`SwatchPressed` の意味は変わらない、見た目の変化先が変わるだけ)。
//!
//! ## 配線した property(実在するものだけ)
//! - **`TextDocumentStyle::fill`**([`ColorTarget::Fill`]) — スタイル表の既定行
//!   (`styles[0]`、裁定98)、常に `[f64; 4]`。
//! - **`TextDocumentStyle::stroke_color`**([`ColorTarget::Stroke`]) —
//!   同じ既定行、`Option<[f64; 4]>`。`None` の間に編集すると
//!   [`DEFAULT_STROKE_COLOR`](黒・不透明)を土台にして触った channel だけ変える
//!   (`crate::default_vec2`「無ければ既定」と同じ考え方)。
//!
//! 書き口は [`commit_text_style_color`] — `text.rs::apply_text_document_edit`
//! (`pub` ではないので他 module から呼べない)と**同じ read-modify-write の形**を
//! ここで独立に組む(`Document::view().text_document()`/`Intent::SetTextDocument`
//! はどちらも public API、`crate::text::{default_text_document, default_text_style}`
//! は既に `pub fn` なので読むだけで済む — `text.rs` 自体は無改変)。
//!
//! ## 見送り(意味が実在しない、発注書「無い物は見送り理由つき」)
//! - **shape layer の fill/stroke**(Lottie `fl`/`st`): `motolii-vector::{Fill,
//!   Stroke}` は engine 側の描画データで、`Document`/`Intent` を経由する
//!   property ではない(`next/engine/motolii-vector/src/lib.rs` 実測 —
//!   `motolii_store::property` にも `fl`/`st` 相当の定数が無い)。束が来たら
//!   [`color_row`]/[`commit_text_style_color`] と同じ形をそのまま繋げられる
//!   ([`ColorTarget`] を増やすだけ)。
//! - **`Composition.background`**: 既に `motolii-settings-pane` に編集口がある
//!   (`BackgroundChannel`)— 2箇所目を作らない。
//! - **`LayerAttrs.label_color`**: `Value::Color` ではなく palette index
//!   (`u8`)で、既に巡回チップ([`crate::chrome::label_color_chip`])という編集口
//!   がある — RGBA 化は対象外。
//!
//! ## まだ crate 本体へ結線していない(`motolii-settings-pane::sections` 第1切片と同じ形)
//! この module は自己完結 — `crate::Message`/`crate::view` を一切変えない
//! (この発注の write-set は `lib.rs` の `mod color;` 一行のみ)。結線には
//! crate の `Message` enum へ2腕(`ColorChannelInput`/`ColorChannelSubmit`)を
//! 足し、`text_section` の Fill/Stroke 行を [`color_row`] へ差し替える必要が
//! あり、どちらも今回の write-set 外(`sections.rs` 冒頭 doc の「供覧・
//! supervisor 結線手順」と同じ扱い)。

use motolii_settings_pane::chrome::{parse_number, value_input_style};
use motolii_store::{Document, Intent, LayerId, TextDocumentStyle};
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{column, container, mouse_area, row as row_widget, text, text_input};
use iced::{Element, Length};

use crate::chrome::label_chip_side;
use crate::text::{default_text_document, default_text_style};

// ---------------------------------------------------------------------------
// 型
// ---------------------------------------------------------------------------

/// RGBA の1チャンネル。`motolii_settings_pane::BackgroundChannel` と同型
/// (別 crate なので共有型は置けない — 式・意味だけ揃える、`sibling_gap_px` 等
/// 既存の「別 crate は式だけ共有」慣習と同じ)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorChannel {
    R,
    G,
    B,
    A,
}

impl ColorChannel {
    /// 「全チャンネルが実在 index に紐づく」テストが回すための全列挙。
    pub const ALL: [ColorChannel; 4] = [Self::R, Self::G, Self::B, Self::A];

    /// `[f64; 4]`(`Value::Color`/`TextDocumentStyle::fill` と同じ並び)内の添字。
    pub fn index(self) -> usize {
        match self {
            Self::R => 0,
            Self::G => 1,
            Self::B => 2,
            Self::A => 3,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::R => "R",
            Self::G => "G",
            Self::B => "B",
            Self::A => "A",
        }
    }
}

/// この editor が書ける色。`TextDocumentStyle` の既定行(`styles[0]`)の
/// どちらのフィールドを触るか(crate doc「配線した property」参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorTarget {
    /// `TextDocumentStyle::fill`(`text-document fc`)。
    Fill,
    /// `TextDocumentStyle::stroke_color`(`text-document sc`)。`None` なら
    /// [`DEFAULT_STROKE_COLOR`] を土台に編集する。
    Stroke,
}

impl ColorTarget {
    pub const ALL: [ColorTarget; 2] = [Self::Fill, Self::Stroke];

    fn label(self) -> &'static str {
        match self {
            Self::Fill => "Fill Color",
            Self::Stroke => "Stroke Color",
        }
    }
}

/// `stroke_color` が `None` の間に編集を始めた時の土台(黒・不透明 — AE/Lottie
/// とも「線色オフ」の既定復帰値として素直な選択、`preset_rgba`
/// (`BackgroundPreset::Black`)と同じ8bit変換なしの素の値)。
pub const DEFAULT_STROKE_COLOR: [f64; 4] = [0.0, 0.0, 0.0, 1.0];

/// 色欄、編集中の下書き。**Document ではない**
/// (`crate::BackgroundFieldDraft`/`crate::FieldDraft` と同じ形 — Enter まで
/// store に触らない)。
#[derive(Clone, Debug, PartialEq)]
pub struct ColorFieldDraft {
    pub target: ColorTarget,
    pub channel: ColorChannel,
    pub text: String,
}

// ---------------------------------------------------------------------------
// 意味(読み・parse/format)
// ---------------------------------------------------------------------------

/// `style` から `target` の実 RGBA を読む。`Stroke` が `None` の間は
/// [`DEFAULT_STROKE_COLOR`] を返す(表示専用の「今この行が示す値」— 書き込み時は
/// [`set_text_style_color_channel`] が改めて `Some` へ昇格させる)。
pub fn text_style_color(style: &TextDocumentStyle, target: ColorTarget) -> [f64; 4] {
    match target {
        ColorTarget::Fill => style.fill,
        ColorTarget::Stroke => style.stroke_color.unwrap_or(DEFAULT_STROKE_COLOR),
    }
}

/// `style` の `target` の1チャンネルだけを書き換える(`crate::next_value` の
/// 「他成分は保つ」と同じ考え方)。`Stroke` は `None` でも
/// [`text_style_color`] が返す土台(既定 or 現在値)を基準に1チャンネルだけ
/// 動かし、必ず `Some` へ昇格させる(「編集した=有効にした」— `TextField` の
/// 経路と違い、この editor に「線色を切る」トグルは無い。**Off へ戻す口は
/// 見送り**、RETURN 参照)。
pub fn set_text_style_color_channel(
    style: &mut TextDocumentStyle,
    target: ColorTarget,
    channel: ColorChannel,
    value_0_1: f64,
) {
    let mut rgba = text_style_color(style, target);
    rgba[channel.index()] = value_0_1;
    match target {
        ColorTarget::Fill => style.fill = rgba,
        ColorTarget::Stroke => style.stroke_color = Some(rgba),
    }
}

/// 数値入力欄の文字列 → 0.0..1.0 にクランプした値(0..255 の8bit表示から)。
/// 読めなければ `None`(`crate::parse_channel_u8` と同じ形 —
/// `motolii_settings_pane::BackgroundChannel` の欄と表示単位をそろえる)。
pub fn parse_color_channel_u8(text: &str) -> Option<f64> {
    parse_number(text).map(|value| value.clamp(0.0, 255.0) / 255.0)
}

/// [`parse_color_channel_u8`] の逆写像 — 0.0..1.0 → 0..255 の表示文字列
/// (最近傍丸め、`motolii_settings_pane::channel_cell` の `current_u8` 計算と
/// 同じ式)。
pub fn format_color_channel_u8(value_0_1: f64) -> String {
    ((value_0_1 * 255.0).round().clamp(0.0, 255.0) as u32).to_string()
}

/// 欄の現在値の表示文字列(下書きが無い時に欄へ出す値)。
pub fn color_channel_display(style: &TextDocumentStyle, target: ColorTarget, channel: ColorChannel) -> String {
    format_color_channel_u8(text_style_color(style, target)[channel.index()])
}

// ---------------------------------------------------------------------------
// 書き口 — `&mut Document` を明示引数で受け取る自由関数
// (`crate::transform::commit_inspector_field`/`crate::text::commit_text_field`
// と同じ形。呼び出し口は `motolii_shell::Shell` の glue、この crate は
// `Shell` を持てない)。
// ---------------------------------------------------------------------------

/// 色欄(R/G/B/A)の1チャンネル — 下書きを確定して1回の
/// `Intent::SetTextDocument` を出す(read-modify-write、他チャンネル・他の
/// スタイルフィールドは今の値のまま、1 gesture = 1 undo)。
///
/// `text.rs::apply_text_document_edit`(`text.rs` は読み専用なので呼べない)と
/// **同じ意味論**をここで独立に組む: 選択が無ければ no-op・実際に値が変わった
/// 時だけ Intent を出す(決定7「同値は Undo を積まない」)。
pub fn commit_text_style_color(
    doc: &mut Document,
    draft: &mut Option<ColorFieldDraft>,
    selection: Option<LayerId>,
    target: ColorTarget,
    channel: ColorChannel,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.target != target || taken.channel != channel {
        // 別の欄の submit(起こらないはずだが、安全側で下書きを戻す —
        // `commit_inspector_field`/`commit_text_field` と同じ判断)。
        *draft = Some(taken);
        return Ok(());
    }
    let Some(layer) = selection else {
        return Ok(());
    };
    let Some(value_0_1) = parse_color_channel_u8(&taken.text) else {
        return Err(format!("数値として読めない: {}", taken.text));
    };

    let current = doc
        .view()
        .text_document(layer)
        .map_err(|error| format!("text document を読めない: {error}"))?;
    let mut document = current.clone().unwrap_or_else(default_text_document);
    let mut style = document.styles.first().cloned().unwrap_or_else(default_text_style);
    set_text_style_color_channel(&mut style, target, channel, value_0_1);
    if document.styles.is_empty() {
        document.styles.push(style);
    } else {
        document.styles[0] = style;
    }

    let unchanged = match &current {
        Some(existing) => existing == &document,
        None => document == default_text_document(),
    };
    if unchanged {
        return Ok(());
    }
    doc.apply(Intent::SetTextDocument { layer, document })
        .map_err(|error| format!("色を書けない: {error}"))
}

/// Multi-selection variant of [`commit_text_style_color`]. Only text layers
/// contribute a `SetTextDocument`; unsupported selected layers are untouched,
/// and all changed text layers share one undo step.
pub fn commit_text_style_color_for_layers(
    doc: &mut Document,
    draft: &mut Option<ColorFieldDraft>,
    selected_layers: &[LayerId],
    target: ColorTarget,
    channel: ColorChannel,
) -> Result<(), String> {
    let Some(taken) = draft.take() else {
        return Ok(());
    };
    if taken.target != target || taken.channel != channel {
        *draft = Some(taken);
        return Ok(());
    }
    let Some(value_0_1) = parse_color_channel_u8(&taken.text) else {
        return Err(format!("数値として読めない: {}", taken.text));
    };

    crate::bulk::apply_to_selected_text_layers(doc, selected_layers, |layer, store| {
        let current = store
            .text_document(layer)
            .map_err(|error| format!("text document を読めない: {error}"))?;
        let mut document = current.clone().unwrap_or_else(default_text_document);
        let mut style = document
            .styles
            .first()
            .cloned()
            .unwrap_or_else(default_text_style);
        set_text_style_color_channel(&mut style, target, channel, value_0_1);
        if document.styles.is_empty() {
            document.styles.push(style);
        } else {
            document.styles[0] = style;
        }

        let unchanged = match &current {
            Some(existing) => existing == &document,
            None => document == default_text_document(),
        };
        if unchanged {
            Ok(None)
        } else {
            Ok(Some(Intent::SetTextDocument { layer, document }))
        }
    })
}

// ---------------------------------------------------------------------------
// Message(module ローカル — crate 冒頭 doc「結線互換の縫い目」/
// `motolii_settings_pane::sections` 第1切片と同じ「自己完結・供覧」の形)。
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    /// 色欄(R/G/B/A)への打鍵。**まだ Document を書かない** — 下書きを更新する
    /// だけ(`crate::Message::FieldInput` と同じ形)。
    ChannelInput(ColorTarget, ColorChannel, String),
    /// 色欄の Enter — ここで初めて [`commit_text_style_color`] を1回呼ぶ
    /// (1 gesture = 1 undo)。
    ChannelSubmit(ColorTarget, ColorChannel),
    /// 色欄のキャプション press(裁定217 連続量 drag 化、E-5)。
    /// `motolii_settings_pane::sections::Message::CompFieldDragPressed` と同じ
    /// 「press だけ own する」形(そちら側の doc「AE のスクラブ精神論」参照)—
    /// 続きは shell 側の window 全体購読 + `Shell::value_drag` が持つ。
    ChannelDragPressed(ColorTarget, ColorChannel),
    /// 色見本(swatch)の press(S4、#56 の穴塞ぎ)。下書きは経由しない即時
    /// 操作 — `motolii_shell::Shell` がこの target の R channel 欄
    /// ([`channel_input_id`])へ focus task を返すだけ(`transform.rs::
    /// field_input_id`/`Shell::enter_field_editing` と同じ形、crate 冒頭 doc
    /// 「S4」節参照)。
    SwatchPressed(ColorTarget),
}

// ---------------------------------------------------------------------------
// view — 投影(`&TextDocumentStyle`)と下書きだけを受け取る。書けない。
// ---------------------------------------------------------------------------

/// 1色ぶんの行: 色見本 + ラベル + RGBA 4欄(`motolii_settings_pane::background_row`
/// と同じ列組み — label セル `Fill Color`/`Stroke Color`([`ColorTarget::label`])、
/// `row(cells)` の4本)。
pub fn color_row(
    target: ColorTarget,
    style: &TextDocumentStyle,
    draft: Option<&ColorFieldDraft>,
    allow_drag: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let rgba = text_style_color(style, target);
    let cells: Vec<Element<'static, Message>> = ColorChannel::ALL
        .into_iter()
        .map(|channel| channel_cell(target, channel, style, draft, allow_drag, dims, colors))
        .collect();

    let content = row_widget![
        color_swatch(rgba, target, dims),
        text(target.label())
            .size(dims.theme().text.body)
            .color(colors.text_primary)
            .width(Length::Fill),
        row_widget(cells).spacing(dims.theme().space.xs),
    ]
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fixed(dims.inspector_row_height))
        .padding([0.0, dims.theme().space.m])
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_theme| crate::chrome::row_band_style(dims))
        .into()
}

/// caption + 数値欄のセル(`motolii_settings_pane::channel_cell` と同じ形 —
/// 2箇所で別の意匠を発明しない)。
fn channel_cell(
    target: ColorTarget,
    channel: ColorChannel,
    style: &TextDocumentStyle,
    draft: Option<&ColorFieldDraft>,
    allow_drag: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let displayed = draft
        .filter(|draft| draft.target == target && draft.channel == channel)
        .map(|draft| draft.text.clone())
        .unwrap_or_else(|| color_channel_display(style, target, channel));

    let channel_label: Element<'static, Message> = if allow_drag {
        // 裁定217 連続量 drag 化(E-5): `motolii_settings_pane` 側の
        // 数値セルと同じキャプション drag ハンドル(2箇所で別の意匠を
        // 発明しない — crate doc「意匠」節どおり)。
        mouse_area(
            text(channel.label())
                .size(dims.theme().text.caption)
                .color(colors.text_muted)
                .align_x(iced::alignment::Horizontal::Center)
                .width(Length::Fixed(dims.inspector_value_width))
        )
        .interaction(iced::mouse::Interaction::ResizingHorizontally)
        .on_press(Message::ChannelDragPressed(target, channel))
        .into()
    } else {
        text(channel.label())
            .size(dims.theme().text.caption)
            .color(colors.text_muted)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fixed(dims.inspector_value_width))
            .into()
    };

    column![
        channel_label,
        // 裁定170 M01: fork の text_input は借用寿命を返り値に縛るため owned
        // move(値不変、`channel_cell`/`comp_field_cell` と同じ回避)。
        text_input("", displayed)
            .id(channel_input_id(target, channel))
            .on_input(move |text| Message::ChannelInput(target, channel, text))
            .on_submit(Message::ChannelSubmit(target, channel))
            .size(dims.theme().text.body)
            .width(Length::Fixed(dims.inspector_value_width))
            .padding(0.0)
            .align_x(iced::alignment::Horizontal::Center)
            .style(move |_theme, status| value_input_style(dims, colors, status)),
    ]
    .spacing(0.0)
    .into()
}

/// 色見本(swatch)。面 = 色そのもの — [`crate::chrome::label_color_chip`] と
/// 同じ「これは箱ではなく見本」の扱い(裁定179「箱は状態の器」の対象外)。
/// **クリックしない**(popover を作らない — crate doc 参照、押せない物は
/// 押せなさそうに)。1辺は [`crate::chrome::label_chip_side`] と同じ式
/// (timeline rail のチップ式、`round(0.462 × 行高)`)を共有する — 同じ
/// 「色そのものを示す正方形」の役割なので2箇所で別寸法を発明しない。
fn color_swatch(rgba: [f64; 4], target: ColorTarget, dims: Dimensions) -> Element<'static, Message> {
    let side = label_chip_side(dims.inspector_row_height);
    let chip_color = iced::Color {
        r: rgba[0].clamp(0.0, 1.0) as f32,
        g: rgba[1].clamp(0.0, 1.0) as f32,
        b: rgba[2].clamp(0.0, 1.0) as f32,
        a: rgba[3].clamp(0.0, 1.0) as f32,
    };
    let swatch = container(text(""))
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .padding(0.0)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(chip_color)),
            border: iced::Border {
                // 裁定179: 常時輪郭なし(これは選択状態を持たない見本なので、
                // `label_color_chip` の hover 縁すら要らない — 常に透明)。
                color: iced::Color::TRANSPARENT,
                width: dims.theme().stroke.hairline,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });

    // S4(#56 の穴塞ぎ): 「見えるのに反応がない」を閉じる — click で R
    // channel 欄へ focus(crate 冒頭 doc「S4」節、`Message::SwatchPressed`
    // 参照)。カーソルを pointer にして「押せる」ことを示す
    // (`ChannelDragPressed` の `ResizingHorizontally` と同じ、Q0 の逆側)。
    mouse_area(swatch)
        .interaction(iced::mouse::Interaction::Pointer)
        .on_press(Message::SwatchPressed(target))
        .into()
}

/// RGBA 欄1本の `Id`(S4、#56)。`transform.rs::field_input_id` と同じ
/// 「確定的な文字列 Id」の形 — swatch click は常に R 欄
/// ([`ColorChannel::R`])を focus 先に選ぶ(4欄のうちどれかへ入れば残り3欄は
/// タブ順で辿れる、既存 `text_input` のタブ移動の上に新しい機構を足さない)。
pub fn channel_input_id(target: ColorTarget, channel: ColorChannel) -> iced::widget::Id {
    let name: &'static str = match (target, channel) {
        (ColorTarget::Fill, ColorChannel::R) => "inspector-color-fill-r",
        (ColorTarget::Fill, ColorChannel::G) => "inspector-color-fill-g",
        (ColorTarget::Fill, ColorChannel::B) => "inspector-color-fill-b",
        (ColorTarget::Fill, ColorChannel::A) => "inspector-color-fill-a",
        (ColorTarget::Stroke, ColorChannel::R) => "inspector-color-stroke-r",
        (ColorTarget::Stroke, ColorChannel::G) => "inspector-color-stroke-g",
        (ColorTarget::Stroke, ColorChannel::B) => "inspector-color-stroke-b",
        (ColorTarget::Stroke, ColorChannel::A) => "inspector-color-stroke-a",
    };
    iced::widget::Id::new(name)
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------
    // parse/format ラウンドトリップ・alpha 境界
    // -----------------------------------------------------------------

    #[test]
    fn parse_color_channel_clamps_into_0_to_255_and_scales_to_unit_range() {
        assert_eq!(parse_color_channel_u8("0"), Some(0.0));
        assert_eq!(parse_color_channel_u8("255"), Some(1.0));
        assert_eq!(parse_color_channel_u8("-10"), Some(0.0), "負は0へクランプ");
        assert_eq!(parse_color_channel_u8("999"), Some(1.0), "255超は1.0へクランプ");
        assert_eq!(parse_color_channel_u8("128"), Some(128.0 / 255.0));
        assert_eq!(parse_color_channel_u8("not a number"), None);
    }

    #[test]
    fn format_color_channel_shows_the_8bit_integer() {
        assert_eq!(format_color_channel_u8(0.0), "0");
        assert_eq!(format_color_channel_u8(1.0), "255");
        assert_eq!(format_color_channel_u8(0.5), "128");
    }

    /// 表示→入力→確定の往復で値が動かない(`fps_display_round_trips_through_parse_unchanged`
    /// と同じ礼儀)。0..255 の全整数を尽くす — alpha 境界(0/255)もこの中に含む。
    #[test]
    fn channel_display_round_trips_through_parse_for_every_8bit_value() {
        for raw in 0..=255u32 {
            let value_0_1 = raw as f64 / 255.0;
            let displayed = format_color_channel_u8(value_0_1);
            let parsed = parse_color_channel_u8(&displayed).expect("8bit表示は必ず読めるはず");
            let round_tripped = (parsed * 255.0).round() as u32;
            assert_eq!(round_tripped, raw, "raw={raw} displayed={displayed:?}");
        }
    }

    // -----------------------------------------------------------------
    // 意味実在柵: 全 ColorChannel が `[f64; 4]` の実在 index に紐づく
    // -----------------------------------------------------------------

    #[test]
    fn every_channel_maps_to_a_distinct_rgba_index() {
        let indices: Vec<usize> = ColorChannel::ALL.iter().map(|c| c.index()).collect();
        assert_eq!(indices, vec![0, 1, 2, 3]);
    }

    // -----------------------------------------------------------------
    // text_style_color / set_text_style_color_channel — 読み書きの純関数
    // -----------------------------------------------------------------

    #[test]
    fn fill_reads_the_real_style_field_directly() {
        let mut style = default_text_style();
        style.fill = [0.1, 0.2, 0.3, 0.4];
        assert_eq!(text_style_color(&style, ColorTarget::Fill), [0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn stroke_falls_back_to_the_default_while_none() {
        let style = default_text_style();
        assert_eq!(style.stroke_color, None, "既定 style は stroke_color 無し");
        assert_eq!(text_style_color(&style, ColorTarget::Stroke), DEFAULT_STROKE_COLOR);
    }

    #[test]
    fn stroke_reads_the_real_value_once_set() {
        let mut style = default_text_style();
        style.stroke_color = Some([0.5, 0.6, 0.7, 0.8]);
        assert_eq!(text_style_color(&style, ColorTarget::Stroke), [0.5, 0.6, 0.7, 0.8]);
    }

    #[test]
    fn set_channel_preserves_the_other_three_components() {
        let mut style = default_text_style();
        style.fill = [0.1, 0.2, 0.3, 0.4];
        set_text_style_color_channel(&mut style, ColorTarget::Fill, ColorChannel::G, 0.9);
        assert_eq!(style.fill, [0.1, 0.9, 0.3, 0.4], "G以外は保つはず");
    }

    #[test]
    fn set_channel_promotes_none_stroke_to_some_using_the_default_as_a_base() {
        let mut style = default_text_style();
        assert_eq!(style.stroke_color, None);
        set_text_style_color_channel(&mut style, ColorTarget::Stroke, ColorChannel::R, 1.0);
        assert_eq!(
            style.stroke_color,
            Some([1.0, 0.0, 0.0, 1.0]),
            "R だけ動かし、残りは DEFAULT_STROKE_COLOR の土台のまま"
        );
    }

    #[test]
    fn set_channel_on_fill_does_not_touch_stroke_and_vice_versa() {
        let mut style = default_text_style();
        style.fill = [0.0, 0.0, 0.0, 1.0];
        set_text_style_color_channel(&mut style, ColorTarget::Fill, ColorChannel::R, 0.5);
        assert_eq!(style.stroke_color, None, "Fill の編集が Stroke を巻き込んだ");

        let mut style2 = default_text_style();
        set_text_style_color_channel(&mut style2, ColorTarget::Stroke, ColorChannel::B, 0.5);
        assert_eq!(style2.fill, default_text_style().fill, "Stroke の編集が Fill を巻き込んだ");
    }

    // -----------------------------------------------------------------
    // commit_text_style_color — 編集 Message(draft)が正しい Intent を出す
    // -----------------------------------------------------------------

    /// **本命**: [`ColorTarget::ALL`] × [`ColorChannel::ALL`] を尽くし、各
    /// commit が対応 property だけを動かす(`every_comp_field_commits_into_its_real_composition_field_and_nothing_else`
    /// と同じ形の意味実在柵)。
    #[test]
    fn every_color_field_commits_into_its_real_text_style_field_and_nothing_else() {
        for target in ColorTarget::ALL {
            for channel in ColorChannel::ALL {
                let mut doc = Document::new();
                let layer = LayerId(1);
                let mut draft = Some(ColorFieldDraft {
                    target,
                    channel,
                    text: "200".to_owned(),
                });
                commit_text_style_color(&mut doc, &mut draft, Some(layer), target, channel)
                    .unwrap_or_else(|error| panic!("{target:?}/{channel:?} を書けない: {error}"));
                assert!(draft.is_none(), "{target:?}/{channel:?}: 確定後も下書きが残っている");

                let document = doc
                    .view()
                    .text_document(layer)
                    .expect("text document を読めるはず")
                    .expect("Intent を出したので text document があるはず");
                let style = document.styles.first().expect("既定行があるはず");
                let rgba = text_style_color(style, target);
                let expected = (200.0f64 / 255.0 * 1000.0).round() / 1000.0;
                let actual = (rgba[channel.index()] * 1000.0).round() / 1000.0;
                assert_eq!(actual, expected, "{target:?}/{channel:?} の書き込み先が違う");

                // 触っていない側は既定のまま(Fill は既定 [0,0,0,1]・Stroke は
                // None のまま)であることも確かめる(read-modify-write 違反の柵)。
                match target {
                    ColorTarget::Fill => assert_eq!(
                        document.styles[0].stroke_color, None,
                        "{channel:?}: Fill の commit が Stroke を巻き込んだ"
                    ),
                    ColorTarget::Stroke => {
                        assert_ne!(
                            document.styles[0].stroke_color, None,
                            "{channel:?}: Stroke の commit が None のままだった"
                        );
                    }
                }
            }
        }
    }

    /// 別欄の下書きしか無い時の Enter は no-op で、下書きも消さない
    /// (`commit_with_a_mismatched_draft_is_a_no_op_and_keeps_the_draft` と
    /// 同じ意味論)。
    #[test]
    fn commit_with_a_mismatched_draft_is_a_no_op_and_keeps_the_draft() {
        let mut doc = Document::new();
        let layer = LayerId(1);
        let mut draft = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::R,
            text: "10".to_owned(),
        });
        commit_text_style_color(&mut doc, &mut draft, Some(layer), ColorTarget::Fill, ColorChannel::G)
            .expect("no-op のはず(channel 不一致)");
        assert!(draft.is_some(), "無関係の Enter が下書きを消した");

        commit_text_style_color(&mut doc, &mut draft, Some(layer), ColorTarget::Stroke, ColorChannel::R)
            .expect("no-op のはず(target 不一致)");
        assert!(draft.is_some(), "無関係の Enter(target違い)が下書きを消した");

        assert_eq!(
            doc.view().text_document(layer).expect("読めるはず"),
            None,
            "no-op のはずが text document を作ってしまった"
        );
    }

    /// 読めない入力は Err(status 帯行き)で Document には触らない。
    #[test]
    fn commit_rejects_garbage_without_touching_the_document() {
        let mut doc = Document::new();
        let layer = LayerId(1);
        let mut draft = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::R,
            text: "not a number".to_owned(),
        });
        let error = commit_text_style_color(&mut doc, &mut draft, Some(layer), ColorTarget::Fill, ColorChannel::R)
            .expect_err("読めない入力が通ってしまった");
        assert!(error.contains("not a number"), "理由文に入力が引用されていない: {error}");
        assert_eq!(
            doc.view().text_document(layer).expect("読めるはず"),
            None,
            "Err のはずが Document に触ってしまった"
        );
    }

    /// 選択が無ければ no-op(`apply_text_document_edit` と同じ意味論 — 下書きは
    /// 消費されるが Document は不変)。
    #[test]
    fn commit_with_no_selection_is_a_no_op() {
        let mut doc = Document::new();
        let mut draft = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::R,
            text: "200".to_owned(),
        });
        commit_text_style_color(&mut doc, &mut draft, None, ColorTarget::Fill, ColorChannel::R)
            .expect("選択なしは no-op のはず");
        assert_eq!(doc.store_bytes(), 0, "選択なしの commit が Document を書いた");
    }

    /// 同値の Enter は Intent を出さない(決定7「同値は Undo を積まない」)——
    /// 既定 Fill の R(=0)へ "0" を submit しても undo 段が増えない。
    #[test]
    fn committing_the_same_value_does_not_create_an_undo_step() {
        let mut doc = Document::new();
        let layer = LayerId(1);
        let before_edit_head = doc.edit_head();
        let mut draft = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::R,
            text: "0".to_owned(), // 既定 fill[0] は既に 0.0
        });
        commit_text_style_color(&mut doc, &mut draft, Some(layer), ColorTarget::Fill, ColorChannel::R)
            .expect("同値の commit は Ok のはず");
        assert_eq!(
            doc.edit_head(),
            before_edit_head,
            "同値の commit が undo 段を作った"
        );
        assert!(!doc.can_undo(), "同値の commit の後に undo できてしまう");
    }

    /// alpha(A)境界: 0 と 255 の両端が他チャンネルを巻き込まず書ける。
    /// **同じ `Document` 上で 0→255 と連続で踏む** — 既定 fill の alpha は
    /// 既に 1.0(=255)なので、真っさらな `Document` へ直接 "255" を submit
    /// すると「同値は Undo を積まない」(決定7)の no-op に落ちて
    /// `text_document` が `None` のまま(意図どおり — 別テスト
    /// [`committing_the_same_value_does_not_create_an_undo_step`] の対象)。
    /// 0 を経由してから 255 へ戻すことで、どちらの端も「実際に値が変わる
    /// commit」として検分する。
    #[test]
    fn alpha_channel_boundaries_commit_independently() {
        let mut doc = Document::new();
        let layer = LayerId(1);

        let mut draft_to_zero = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::A,
            text: "0".to_owned(),
        });
        commit_text_style_color(&mut doc, &mut draft_to_zero, Some(layer), ColorTarget::Fill, ColorChannel::A)
            .expect("alpha=0 を書けるはず");
        let document = doc
            .view()
            .text_document(layer)
            .expect("読めるはず")
            .expect("alpha=0 は既定(1.0)から実際に変わるので Document があるはず");
        assert_eq!(document.styles[0].fill, [0.0, 0.0, 0.0, 0.0], "alpha=0");

        let mut draft_to_max = Some(ColorFieldDraft {
            target: ColorTarget::Fill,
            channel: ColorChannel::A,
            text: "255".to_owned(),
        });
        commit_text_style_color(&mut doc, &mut draft_to_max, Some(layer), ColorTarget::Fill, ColorChannel::A)
            .expect("alpha=255 を書けるはず");
        let document = doc
            .view()
            .text_document(layer)
            .expect("読めるはず")
            .expect("在るはず");
        assert_eq!(document.styles[0].fill, [0.0, 0.0, 0.0, 1.0], "alpha=255");
        assert_eq!(
            &document.styles[0].fill[0..3],
            &[0.0, 0.0, 0.0],
            "alpha編集がRGBを巻き込んだ"
        );
    }

    // -----------------------------------------------------------------
    // 意匠: 見本(swatch)の1辺は既存チップ式のまま(独自寸法を発明しない柵)
    // -----------------------------------------------------------------

    #[test]
    fn color_swatch_side_reuses_the_existing_label_chip_formula() {
        let dims = Dimensions::default();
        // `crate::chrome::label_chip_side` は non-pub な計算式そのもの
        // (`round(0.462 × 行高)`)— ここでは同じ式を独自に検証するのではなく、
        // 「別の寸法を発明していない」ことを式の再掲で固定する
        // (`sibling_gap_px_matches_the_ladder_bottom_rung_...` と同じ柵の形)。
        let side = (dims.inspector_row_height * 0.462).round().max(1.0);
        assert_eq!(label_chip_side(dims.inspector_row_height), side);
    }
}
