//! MASK section(B02 第1切片、裁定184)。
//!
//! **持つ**: mode 巡回・inverted トグルの意味([`next_mask_mode`]/
//! [`masks_with_cycled_mode`]/[`masks_with_toggled_inverted`])と書き口
//! (`cycle_inspector_mask_mode`/`toggle_inspector_mask_inverted`、
//! `&mut Document` を明示引数で受け取る自由関数)・MASK section の view
//! ([`mask_section`]/`mask_ident_row`)。
//!
//! **持たない**: opacity 値そのものの編集 ── [`crate::transform::
//! TransformField::MaskOpacity`] 経由で既存の値セル文法
//! ([`crate::transform::transform_row`])が書くので、ここに opacity の書き口は
//! 無い。

use motolii_store::{Document, Intent, LayerId, Mask, MaskId, MaskMode};
use motolii_settings_pane::chrome::section_header;
use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, column, row as row_widget, text};
use iced::{Element, Length};

use crate::projection::MaskRowProjection;
use crate::transform::{transform_row, FieldDraft};
use crate::chrome::{bordered_row, flat_button_style, glyph_button_style, glyph_height};
use crate::Message;

// ---------------------------------------------------------------------------
// MASK section(B02 第1切片、裁定184)— mode 巡回・inverted トグルの意味と書き口。
// 値(opacity)は `TransformField::MaskOpacity` 経由で既存の値セル文法が書くので、
// ここに opacity の書き口は無い。
// ---------------------------------------------------------------------------

/// mask mode 巡回ボタンの次の値。並びは `motolii_store::MaskMode` の宣言順
/// (= Lottie `mask-mode` から `None` を落とした6値、store の設計どおり)。
/// [`next_blend_mode`] と同じ巡回ボタン文法 — pick_list は next/ に前例が無い
/// (BL2 の決定)ので導入しない。blend と違い「engine 未対応の mode」は無い
/// (MK2 被覆代数が6値全部を実装済み)ため、対応表の部分集合も持たない。
pub fn next_mask_mode(mode: MaskMode) -> MaskMode {
    match mode {
        MaskMode::Add => MaskMode::Subtract,
        MaskMode::Subtract => MaskMode::Intersect,
        MaskMode::Intersect => MaskMode::Lighten,
        MaskMode::Lighten => MaskMode::Darken,
        MaskMode::Darken => MaskMode::Difference,
        MaskMode::Difference => MaskMode::Add,
    }
}

/// mode 巡回後の mask 一覧(純関数 — Document には触れない)。対象の mask が
/// 居なければ `None`(呼び手は no-op — 選択が edit の合間に変わる稀なケースを
/// 捨てる、`commit_inspector_field` と同じ安全側)。**他の mask・並び順・
/// inverted は一切変えない**(`Intent::SetMasks` は一覧の丸ごと差し替えなので、
/// ここが「対象だけを動かす」ことの正本)。
pub fn masks_with_cycled_mode(masks: &[Mask], target: MaskId) -> Option<Vec<Mask>> {
    masks.iter().any(|mask| mask.id == target).then(|| {
        masks
            .iter()
            .map(|mask| {
                if mask.id == target {
                    Mask {
                        mode: next_mask_mode(mask.mode),
                        ..*mask
                    }
                } else {
                    *mask
                }
            })
            .collect()
    })
}

/// inverted トグル後の mask 一覧([`masks_with_cycled_mode`] と同型の純関数)。
pub fn masks_with_toggled_inverted(masks: &[Mask], target: MaskId) -> Option<Vec<Mask>> {
    masks.iter().any(|mask| mask.id == target).then(|| {
        masks
            .iter()
            .map(|mask| {
                if mask.id == target {
                    Mask {
                        inverted: !mask.inverted,
                        ..*mask
                    }
                } else {
                    *mask
                }
            })
            .collect()
    })
}

/// MASK section の mode 巡回 — 即1回の `Intent::SetMasks` を出す(1 click =
/// 1 undo、`ToggleHidden`/`CycleBlendMode` と同じ即時操作の形)。選択なし・
/// 対象 mask なしは黙って no-op(`Ok(())`)。書き込み失敗だけ `Err` の理由文
/// (M13、呼び出し側が status 帯へ渡す)。
pub fn cycle_inspector_mask_mode(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
) -> Result<(), String> {
    apply_mask_list_edit(doc, selection, mask, masks_with_cycled_mode)
}

/// MASK section の inverted トグル([`cycle_inspector_mask_mode`] と同型)。
pub fn toggle_inspector_mask_inverted(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
) -> Result<(), String> {
    apply_mask_list_edit(doc, selection, mask, masks_with_toggled_inverted)
}

/// mode 巡回・inverted トグル共通の書き口: 今の一覧を読み、純関数で編集後の
/// 一覧を作り、1回の `Intent::SetMasks` で書く。
fn apply_mask_list_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    mask: MaskId,
    edit: fn(&[Mask], MaskId) -> Option<Vec<Mask>>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let masks = doc
        .view()
        .masks(layer)
        .map_err(|error| format!("mask を読めない: {error}"))?;
    let Some(new_masks) = edit(&masks, mask) else {
        return Ok(()); // 対象 mask が居ない(stale click)— 黙って捨てる。
    };
    doc.apply(Intent::SetMasks {
        layer,
        masks: new_masks,
    })
    .map_err(|error| format!("mask を書けない: {error}"))
}

/// MASK section: mask 1枚 = ident 行(id + mode 巡回 + Inverted トグル)+
/// opacity 値行([`transform_row`] そのまま — 値セル/Key 列の文法を再利用)。
/// section header・行高・余白はすべて既存トークン([`section_header`]/
/// [`bordered_row`])— 新しい寸法・色ロールを発明しない(裁定179/S4)。
pub(crate) fn mask_section(
    masks: &[MaskRowProjection],
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut section = column![section_header("MASK", dims, colors)];
    for mask_row in masks {
        section = section.push(mask_ident_row(mask_row, dims, colors));
        section = section.push(transform_row(&mask_row.opacity, field_draft, dims, colors));
    }
    section.into()
}

/// mask 1枚の ident 行: 「Mask {id}」ラベル + mode 巡回ボタン
/// ([`flat_button_style`]、Blend 行と同じ文法)+ Inverted トグル
/// ([`glyph_button_style`]、M glyph と同じ「状態の器」文法 — inverted=on の
/// 時だけ accent 縁)。
fn mask_ident_row(
    mask_row: &MaskRowProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let id = mask_row.id;
    let inverted = mask_row.inverted;

    let content = row_widget![
        text(format!("Mask {id}"))
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        // mode 巡回(`CycleBlendMode` と同じ即時操作・同じ意匠)。表示は
        // `MaskMode` の `Debug`(`Add`/`Subtract`/… — blend の表示と同じ流儀)。
        button(text(format!("{:?}", mask_row.mode)).size(dims.body_text))
            .on_press(Message::CycleMaskMode(id))
            .style(move |_theme, status| flat_button_style(colors, status)),
        // inverted トグル(M glyph と同じ「チップ輪郭=状態の器」文法 —
        // 裁定179。glyph 幅1文字では意図が読めないので語で出す: 意図優先・裁定174)。
        button(
            text("Inverted")
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(Message::ToggleMaskInverted(id))
        .style(move |_theme, status| glyph_button_style(dims, colors, status, inverted)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}

