//! EFFECTS section(B38 編集側 第3切片、裁定184 型別 section 第2号)。
//!
//! **持つ**: 既知 plugin(Glow)のパラメータカタログ([`GlowParam`]/
//! [`GLOW_PLUGIN_ID`]/[`plugin_params`]/[`plugin_display_name`])・stack の
//! remove/reorder/bypass の意味と書き口([`effects_with_removed`] 系の純関数+
//! `remove_inspector_effect` 系の `&mut Document` 書き口)・EFFECTS section の
//! view([`effects_section`]/`effect_ident_row`)。
//!
//! **持たない**: param 値そのものの編集 ── [`crate::transform::TransformField::
//! EffectParam`] 経由で既存の値セル文法([`crate::transform::transform_row`])が
//! 書くので、ここに param の書き口は無い(MASK section と同じ分担)。

use motolii_store::{Document, EffectId, EffectInstance, Intent, LayerId};
use motolii_settings_pane::chrome::section_header;
use motolii_tokens_rs::{Colors, Dimensions, Ink};

use iced::widget::{button, column, row as row_widget, text};
use iced::{Element, Length};

use crate::projection::EffectRowProjection;
use crate::transform::{transform_row, FieldDraft};
use crate::chrome::{bordered_row, flat_button_style, glyph_button_style, glyph_height};
use crate::Message;

// ---------------------------------------------------------------------------
// EFFECTS: 既知 plugin の param カタログ(B38 第3切片)
// ---------------------------------------------------------------------------

/// 内蔵 vism 第1号 Glow の plugin id(裁定153 S4)。**engine 側の変換表
/// (`next/engine/motolii-engine/src/lib.rs::translate_effect_passes`)と同期を
/// 保つ義務がある**([`SUPPORTED_BLEND_MODES`] と同じ二重化の形 — engine が
/// 対応する plugin だけをここに書く)。
pub const GLOW_PLUGIN_ID: &str = "motolii.glow";

/// Glow の param カタログ(engine `translate_glow_params` が読む3つの named
/// param)。**enum で閉じる** — [`TransformField`]/[`KeyRow`] は `Copy` なので
/// param 名を `String` で運べない。既定値・小数桁・drag 感度もここに束ねる
/// (型別 editor registry の考え方、crate doc 参照)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlowParam {
    /// bright-pass 閾値(engine 既定 1.0 — proof `bright_fs` のハードコード値)。
    Threshold,
    /// composite の減衰率(engine 既定 0.75 — proof `composite_fs`)。
    Intensity,
    /// blur タップ間隔スケール(engine 既定 1.0 = proof の固定オフセット)。
    Radius,
}

impl GlowParam {
    /// 宣言順 = 表示順(engine `translate_glow_params` の読み出し順と同じ並び)。
    pub const ALL: [GlowParam; 3] = [
        GlowParam::Threshold,
        GlowParam::Intensity,
        GlowParam::Radius,
    ];

    /// track 名の断片(`effect.{id}.param.{name}` の `{name}`)。engine の
    /// `find("threshold", ..)` 等と一致する義務がある(上記の同期義務と同じ)。
    pub fn name(self) -> &'static str {
        match self {
            Self::Threshold => "threshold",
            Self::Intensity => "intensity",
            Self::Radius => "radius",
        }
    }

    /// 行ラベル(表示)。`name` の頭を大文字化しただけ — 発明ではない。
    pub fn label(self) -> &'static str {
        match self {
            Self::Threshold => "Threshold",
            Self::Intensity => "Intensity",
            Self::Radius => "Radius",
        }
    }

    /// track の無い param の既定値。**engine の既定
    /// (`GLOW_DEFAULT_THRESHOLD`/`INTENSITY`/`RADIUS`、private const)の写し** —
    /// engine と同期を保つ義務([`GLOW_PLUGIN_ID`] と同じ)。表示既定が engine
    /// 既定とズレると「値を出しただけで絵が変わって見える」誤読になるため。
    pub fn default_value(self) -> f64 {
        match self {
            Self::Threshold => 1.0,
            Self::Intensity => 0.75,
            Self::Radius => 1.0,
        }
    }
}

/// plugin id → param カタログ。**未知 plugin は空**(store は catalog を知らず、
/// engine も未知 plugin_id を無音 skip する — param 行を捏造しない、M13)。
pub fn plugin_params(plugin_id: &str) -> &'static [GlowParam] {
    if plugin_id == GLOW_PLUGIN_ID {
        &GlowParam::ALL
    } else {
        &[]
    }
}

/// plugin id → 表示名。既知 plugin だけ人間可読名、未知は plugin_id をそのまま
/// (M13: 無い意味を有るふりで出さない — id を隠して汎用名を出す方が嘘になる)。
pub fn plugin_display_name(plugin_id: &str) -> &str {
    if plugin_id == GLOW_PLUGIN_ID {
        "Glow"
    } else {
        plugin_id
    }
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

/// enabled を裏返した後の一覧([`masks_with_toggled_inverted`] と同型)。
pub fn effects_with_toggled_enabled(
    effects: &[EffectInstance],
    target: EffectId,
) -> Option<Vec<EffectInstance>> {
    effects.iter().any(|effect| effect.id == target).then(|| {
        effects
            .iter()
            .map(|effect| {
                if effect.id == target {
                    EffectInstance {
                        enabled: !effect.enabled,
                        ..effect.clone()
                    }
                } else {
                    effect.clone()
                }
            })
            .collect()
    })
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

/// EFFECTS section の bypass トグル(同上)。
pub fn toggle_inspector_effect_bypass(
    doc: &mut Document,
    selection: Option<LayerId>,
    effect: EffectId,
) -> Result<(), String> {
    apply_effect_list_edit(doc, selection, |effects| {
        effects_with_toggled_enabled(effects, effect)
    })
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
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(message)
        .style(move |_theme, status| flat_button_style(colors, status))
    };

    let content = row_widget![
        text(effect_row.name.clone())
            .size(dims.body_text)
            .color(name_color)
            .width(Length::Fill),
        caption_button("↑", Message::MoveEffectUp(id)),
        caption_button("↓", Message::MoveEffectDown(id)),
        // bypass トグル(mask Inverted と同じ「状態の器」文法 — on の時だけ
        // accent 縁。押しても消えない = 「消さずに切る」を器で語る)。
        button(
            text("Bypass")
                .size(dims.caption_text)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .height(Length::Fixed(glyph_height(dims)))
        .padding([0.0, dims.spacing_s])
        .on_press(Message::ToggleEffectBypass(id))
        .style(move |_theme, status| glyph_button_style(dims, colors, status, bypassed)),
        caption_button("Remove", Message::RemoveEffect(id)),
    ]
    .spacing(dims.spacing_xs)
    .align_y(iced::alignment::Vertical::Center);

    bordered_row(content.into(), dims)
}


