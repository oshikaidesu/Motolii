//! EFFECTS section(B38 編集側 第3切片、裁定184 型別 section 第2号)。
//!
//! **読む**: provider/device registry のパラメータカタログ([`GLOW_PLUGIN_ID`]/
//! [`plugin_params`])・**持つ**: stack の
//! remove/reorder/bypass の意味と書き口([`effects_with_removed`] 系の純関数+
//! `remove_inspector_effect` 系の `&mut Document` 書き口)・EFFECTS section の
//! view([`effects_section`]/`effect_ident_row`)。
//! plugin の表示名は [`crate::device`] registry を読む。
//!
//! **持たない**: param 値そのものの編集 ── [`crate::transform::TransformField::
//! EffectParam`] 経由で既存の値セル文法([`crate::transform::transform_row`])が
//! 書くので、ここに param の書き口は無い(MASK section と同じ分担)。

use motolii_store::{
    Document, EffectId, EffectInstance, Fps, Intent, LayerId, PropertyId, RationalTime, Value,
};
use motolii_settings_pane::chrome::section_header;
use motolii_tokens_rs::{Colors, Dimensions, Ink};

use iced::widget::{button, column, row as row_widget, text};
use iced::{Element, Length};

use crate::projection::EffectRowProjection;
use crate::transform::{edited_value_track, transform_row, FieldDraft};
use crate::chrome::{bordered_row, flat_button_style, glyph_button_style, glyph_height, key_glyph};
use crate::device::device_for_provider;
use crate::Message;

// ---------------------------------------------------------------------------
// EFFECTS: provider/device registry への読み口(B38 第3切片)
// ---------------------------------------------------------------------------

/// 内蔵 vism 第1号 Glow の plugin id(裁定153 S4)。**engine 側の変換表
/// (`next/engine/motolii-engine/src/lib.rs::translate_effect_passes`)と同期を
/// 保つ義務がある**([`SUPPORTED_BLEND_MODES`] と同じ二重化の形 — engine が
/// 対応する plugin だけをここに書く)。
pub const GLOW_PLUGIN_ID: &str = crate::device::GLOW_DEVICE.as_str();

/// 既存の呼び手向け互換名。catalog の実体は [`crate::device`] にあり、ここは
/// section からの読み口を再輸出するだけである。
pub use crate::device::parameters_for_provider as plugin_params;

/// plugin id → 表示名。既知 plugin だけ人間可読名、未知は plugin_id をそのまま
/// (M13: 無い意味を有るふりで出さない — id を隠して汎用名を出す方が嘘になる)。
pub fn plugin_display_name(plugin_id: &str) -> &str {
    device_for_provider(plugin_id)
        .map(|device| device.display_name)
        .unwrap_or(plugin_id)
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// EFFECTS section(B38 編集側 第3切片、裁定184 型別 section 第2号)— stack の
// remove / reorder / bypass の意味と書き口。param の値編集は
// `TransformField::EffectParam` 経由で既存の値セル文法が書くので、ここに
// param の書き口は無い(MASK section と同じ分担)。
// ---------------------------------------------------------------------------

/// 取り除いた後の effect 一覧(純関数 — [`masks_with_cycled_mode`] と同型)。
/// 対象が居なければ `None`(stale click — 呼び手は no-op)。param track の
/// 扱いは [`Message::RemoveEffect`] の doc(残す — 1 click = 1 undo を保つ)。
pub fn effects_with_removed(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    effects.iter().any(|effect| effect.id == target).then(|| {
        effects
            .iter()
            .filter(|effect| effect.id != target)
            .cloned()
            .collect()
    })
}

/// 1つ上(適用順の前)へ動かした後の一覧。対象が居ない**か既に先頭**なら
/// `None` — 端での click に空の `Intent::SetEffects`(実質無変更の undo 段)を
/// 積まないため(mask 系の「stale click は黙って捨てる」と同じ安全側の拡張)。
pub fn effects_with_moved_up(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    let index = effects.iter().position(|effect| effect.id == target)?;
    if index == 0 {
        return None;
    }
    let mut out = effects.to_vec();
    out.swap(index - 1, index);
    Some(out)
}

/// 1つ下(適用順の後)へ。[`effects_with_moved_up`] の対 — 末尾なら `None`。
pub fn effects_with_moved_down(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    let index = effects.iter().position(|effect| effect.id == target)?;
    if index + 1 >= effects.len() {
        return None;
    }
    let mut out = effects.to_vec();
    out.swap(index, index + 1);
    Some(out)
}

/// EFFECTS section の remove — 即1回の `Intent::SetEffects`
/// ([`cycle_inspector_mask_mode`] と同じ即時操作の形)。選択なし・対象なしは
/// 黙って no-op(`Ok(())`)、書き込み失敗だけ `Err` の理由文(M13)。
pub fn remove_inspector_effect(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_removed(effects, effect)
    })
}

/// EFFECTS section の上へ移動([`remove_inspector_effect`] と同型)。端は no-op。
pub fn move_inspector_effect_up(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_moved_up(effects, effect)
    })
}

/// EFFECTS section の下へ移動(同上)。
pub fn move_inspector_effect_down(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_moved_down(effects, effect)
    })
}

/// EFFECTS section の bypass トグル。**裁定213/214 で設計をやり直した** —
/// 旧実装は `EffectInstance::enabled`(静止 `bool`)を裏返して丸ごと
/// `Intent::SetEffects` を再発行していたが、`enabled` はもう一覧の外へ出て
/// `effect.{id}.enabled` という普通の track になった(`effect.rs` モジュール
/// doc「2026-08-23」節)。**「一覧から外す」(`effects_with_removed`/
/// `Intent::SetEffects`)と「一時的に切る」は別物のまま** — こちらは
/// [`crate::transform::edited_value_track`](値セル Enter 確定と全く同じ AE 作法:
/// キー無しなら静的値の書き換え、キー持ちなら playhead へのキー upsert)で
/// `effect.{id}.enabled` の track だけを書く。**Key 列からの明示的な打点/除去**
/// (裁定214「Inspector に映る物は全て時間軸で評価できる」の帰結)は
/// [`crate::transform::KeyRow::EffectEnabled`] へ配線済み — `Message::KeyPressed`
/// →`toggle_inspector_key`(shell 側、他の行と共通の1本の経路)が
/// `key_row_property_id`/`toggled_key_track` 経由でこの同じ property を打つので、
/// この関数の専用ロジックは要らない(3状態 oracle・click 判定を複製しない)。
pub fn toggle_inspector_effect_bypass(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
    playhead_frame: i64,
    fps: Fps,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let effects = doc
        .view()
        .effects(layer)
        .map_err(|error| format!("effect を読めない: {error}"))?;
    if !effects.iter().any(|instance| instance.id == effect) {
        return Ok(()); // stale click(対象の effect が既に居ない)。
    }

    let property = PropertyId::effect_enabled(effect);
    let playhead_time = RationalTime::try_from_frame(playhead_frame, fps)
        .map_err(|error| format!("playhead を時刻へ写せない: {error}"))?;

    let store = doc.view();
    let track = store.track(layer, &property).ok().flatten();
    let current = match store
        .value_at(layer, &property, playhead_time)
        .map_err(|error| format!("effect の enabled を読めない: {error}"))?
    {
        Some(Value::Bool(v)) => v,
        Some(other) => {
            return Err(format!(
                "effect {effect} の enabled に真偽でない値が入っている: {other:?}"
            ))
        }
        None => true, // キーを打っていない = 既定で有効。
    };
    let new_track = edited_value_track(track.as_ref(), playhead_frame, fps, Value::Bool(!current))?;
    drop(store);

    doc.apply(Intent::SetTrack {
        layer,
        property,
        track: new_track,
    })
    .map_err(|error| format!("effect の enabled を書けない: {error}"))
}

/// remove/reorder/bypass 共通の書き口([`apply_mask_list_edit`] と同型):
/// 今の一覧を読み、純関数で編集後の一覧を作り、1回の `Intent::SetEffects` で書く。
/// `edit` が `None` を返したら Intent を出さない(stale click・端 reorder)。
fn apply_effect_list_edit(
    doc: &mut Document,
    selection: Option<LayerId>,
    edit: impl FnOnce(&[EffectInstance]) -> Option<Vec<EffectInstance>>,
) -> Result<(), String> {
    let Some(layer) = selection else {
        return Ok(());
    };
    let effects = doc
        .view()
        .effects(layer)
        .map_err(|error| format!("effect を読めない: {error}"))?;
    let Some(new_effects) = edit(&effects) else {
        return Ok(());
    };
    doc.apply(Intent::SetEffects {
        layer,
        effects: new_effects,
    })
    .map_err(|error| format!("effect を書けない: {error}"))
}

/// EFFECTS section: effect 1本 = ident 行(名前 + ↑↓ reorder + Bypass トグル +
/// Remove)+ param 値行([`transform_row`] そのまま — 値セル/Key 列の文法を
/// 再利用)。[`mask_section`] と同じ構成 — section header・行高・余白はすべて
/// 既存トークン、新しい寸法・色ロールを発明しない(裁定179/S4)。
pub(crate) fn effects_section(
    effects: &[EffectRowProjection],
    field_draft: Option<&FieldDraft>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let mut section = column![section_header("EFFECTS", dims, colors)];
    for effect_row in effects {
        section = section.push(effect_ident_row(effect_row, dims, colors));
        for param_row in &effect_row.params {
            section = section.push(transform_row(param_row, field_draft, dims, colors));
        }
    }
    section.into()
}

/// effect 1本の ident 行: 名前ラベル(bypass 中は ink2 — 「効いていない」の
/// 視覚合図、hidden layer の扱いと同型)+ ↑/↓(reorder、[`flat_button_style`])+
/// Bypass トグル(mask の Inverted と同じ「チップ輪郭=状態の器」文法 —
/// bypass=on の時だけ accent 縁)+ Remove([`flat_button_style`])。
/// glyph 1文字では意図が読めない語(Bypass/Remove)は語で出す(意図優先・
/// 裁定174、mask Inverted と同じ判断)。↑↓ は「上へ/下へ」の意図がそのまま
/// 読める最小の記号なので語にしない。
fn effect_ident_row(
    effect_row: &EffectRowProjection,
    dims: Dimensions,
    colors: Colors,
) -> Element<'static, Message> {
    let id = effect_row.id;
    let bypassed = !effect_row.enabled;
    let name_color = if bypassed {
        Ink::Secondary.resolve(&colors)
    } else {
        colors.text_primary
    };

    let caption_button = |label: &'static str, message: Message| {
        button(
            text(label)
                .size(dims.theme().text.caption)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.theme().space.s])
        .on_press(message)
        .style(move |_theme, status| flat_button_style(colors, status))
    };

    let content = row_widget![
        text(effect_row.name.clone())
            .size(dims.theme().text.body)
            .color(name_color)
            .width(Length::Fill),
        caption_button("↑", Message::MoveEffectUp(id)),
        caption_button("↓", Message::MoveEffectDown(id)),
        // bypass トグル(mask Inverted と同じ「状態の器」文法 — on の時だけ
        // accent 縁。押しても消えない = 「消さずに切る」を器で語る)。
        button(
            text("Bypass")
                .size(dims.theme().text.caption)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.theme().space.s])
        .on_press(Message::ToggleEffectBypass(id))
        .style(move |_theme, status| glyph_button_style(dims, colors, status, bypassed)),
        // Key 列(K1、裁定214)— 他の行と全く同じ3状態 oracle・click 文法
        // ([`key_glyph`])。Bypass は「今の値を反転する」即時操作、こちらは
        // 「playhead にキーを打つ/外す」明示操作 — 別の書き口だが同じ
        // `effect.{id}.enabled` property を狙う。
        key_glyph(effect_row.enabled_key, dims, colors),
        caption_button("Remove", Message::RemoveEffect(id)),
    ]
    .spacing(dims.theme().space.xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}
