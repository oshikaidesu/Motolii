//! MATTE(track matte)行 ── 2026-08-22 発注「レイヤーを指す」文法、第1号。
//!
//! **背景**: engine は本日 `MatteMode`/`Matte` の消費を開始した
//! (`next/engine/motolii-engine/src/lib.rs::Engine::apply_matte`/
//! `matte_sources`)が、指定する UI がまだ無かった(P3b 壁9)。`LayerAttrs.matte`
//! フィールドは既に在る(`next/shell/motolii-shell/src/clipboard.rs::full_patch`
//! が `matte: Some(attrs.matte)` を複製している)——ここが初めての書き口。
//!
//! **持つ**: mode 巡回の意味([`next_matte_mode`] — `motolii_store::MatteMode`
//! 4値の宣言順を巡回順の正本にする、`next_blend_mode`/`next_mask_mode` と同じ
//! 判断)・書き口([`set_inspector_matte_source`]/[`cycle_inspector_matte_mode`]/
//! [`clear_inspector_matte`] ── `&mut Document` を明示引数で受け取る自由関数、
//! `mask.rs` と同じ「書ける物を持たない、が書き口は持つ」形)・MATTE 行の
//! view([`matte_row`])。
//!
//! **持たない**: 候補レイヤーの絞り込み(自分自身を選べない・matte 連鎖の
//! 循環を選べない)は [`crate::projection::project`] が `&StoreView` を持つ
//! 時点で計算し、[`crate::projection::AttrsProjection::matte_candidates`] に
//! 詰めて渡す ── この module は絞り込み済みの投影を描くだけ。

use motolii_store::{Document, Intent, LayerAttrsPatch, LayerId, Matte, MatteMode};

use motolii_tokens_rs::{Colors, Dimensions};

use iced::widget::{button, pick_list, row as row_widget, text};
use iced::{Element, Length};

use crate::chrome::{bordered_row, flat_button_style, pick_list_style, value_cell_padding};
use crate::projection::LayerCandidate;
use crate::Message;

/// mode 巡回ボタンの次の値。`motolii_store::MatteMode` の宣言順
/// (Alpha→InvertedAlpha→Luma→InvertedLuma→Alpha)をそのまま辿る
/// (`next_blend_mode`/`next_mask_mode` と同じ「型の宣言順を巡回順の正本に
/// する」判断)。4値とも対応外は無い(engine `translate_matte_mode` が BL4 で
/// 4モード全部を実装済み、`next/engine/motolii-engine/src/lib.rs` 参照)。
pub fn next_matte_mode(mode: MatteMode) -> MatteMode {
    match mode {
        MatteMode::Alpha => MatteMode::InvertedAlpha,
        MatteMode::InvertedAlpha => MatteMode::Luma,
        MatteMode::Luma => MatteMode::InvertedLuma,
        MatteMode::InvertedLuma => MatteMode::Alpha,
    }
}

/// MATTE 元を選ぶ(pick_list からの選択)。**即1回の `Intent::SetAttrs`**
/// (`CycleBlendMode`/`PickFont` と同じ即時操作の形)。新規に元を選んだ時は
/// mode を [`MatteMode::Alpha`](Lottie/AE の既定 4値の最初)で初期化し、既に
/// matte が有れば mode は保つ(元だけ差し替え)。選択なしは黙って no-op。
pub fn set_inspector_matte_source(
    doc: &mut Document,
    selection: Option<LayerId>,
    source: LayerId,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let current = doc
        .view()
        .attrs(layer)
        .map_err(|error| format!("attrs を読めない: {error}"))?
        .unwrap_or_default();
    let mode = current.matte.map(|matte| matte.mode).unwrap_or(MatteMode::Alpha);
    let patch = LayerAttrsPatch {
        matte: Some(Some(Matte { layer: source, mode })),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("attrs を書けない: {error}"))
}

/// MATTE mode 巡回 ── 即1回の `Intent::SetAttrs`(`CycleBlendMode`/
/// `CycleMaskMode` と同じ即時操作の形)。matte が無い(まだ元を選んでいない)
/// なら**巡回する対象が無いので no-op**(`ResetSpeed` の「既に既定値なら
/// no-op」と同じ判断)。
pub fn cycle_inspector_matte_mode(
    doc: &mut Document,
    selection: Option<LayerId>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let current = doc
        .view()
        .attrs(layer)
        .map_err(|error| format!("attrs を読めない: {error}"))?
        .unwrap_or_default();
    let Some(matte) = current.matte else {
        return Ok(());
    };
    let patch = LayerAttrsPatch {
        matte: Some(Some(Matte {
            mode: next_matte_mode(matte.mode),
            ..matte
        })),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("attrs を書けない: {error}"))
}

/// MATTE を外す(`matte: None`)。既に無ければ no-op(決定7「同値は Undo を
/// 積まない」と同じ判断)。
pub fn clear_inspector_matte(doc: &mut Document, selection: Option<LayerId>) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let current = doc
        .view()
        .attrs(layer)
        .map_err(|error| format!("attrs を読めない: {error}"))?
        .unwrap_or_default();
    if current.matte.is_none() {
        return Ok(());
    }
    let patch = LayerAttrsPatch {
        matte: Some(None),
        ..Default::default()
    };
    doc.apply(Intent::SetAttrs { layer, patch })
        .map_err(|error| format!("attrs を書けない: {error}"))
}

/// MATTE 行の view。pick_list で元を選び(`matte_candidates` ── 自己参照/
/// 循環を除いた候補、`project` が絞り込み済み)、mode は巡回ボタン(BL2 と
/// 同じ「小さい閉じた集合は巡回ボタン」判断 ── 値は4値のみ)。Clear は常に
/// 出す(Speed Reset と同じ「無反応ゼロより一貫」判断 ── matte が無い時は
/// no-op)。
pub(crate) fn matte_row(
    current: Option<Matte>,
    candidates: &[LayerCandidate],
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let selected = current.and_then(|matte| {
        candidates
            .iter()
            .find(|candidate| candidate.id == matte.layer)
            .cloned()
    });
    let options = candidates.to_vec();

    let picker = pick_list(selected, options, |candidate: &LayerCandidate| {
        candidate.label.clone()
    })
    .on_select(|candidate: LayerCandidate| Message::PickMatteSource(candidate.id))
    .text_size(dims.body_text)
    .width(Length::Fixed(dims.inspector_value_width))
    .padding(value_cell_padding(dims))
    .placeholder("Pick…")
    .style(move |_theme, status| pick_list_style(dims, colors, status));

    let mode_label = current
        .map(|matte| format!("{:?}", matte.mode))
        .unwrap_or_else(|| "—".to_owned());

    let content = row_widget![
        text("Matte")
            .size(dims.body_text)
            .color(colors.text_primary)
            .width(Length::Fill),
        picker,
        button(text(mode_label).size(dims.body_text))
            .on_press(Message::CycleMatteMode)
            .style(move |_theme, status| flat_button_style(colors, status)),
        button(text("Clear").size(dims.caption_text))
            .on_press(Message::ClearMatte)
            .style(move |_theme, status| flat_button_style(colors, status)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}
