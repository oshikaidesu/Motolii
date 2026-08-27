//! FX STACK — レイヤーに効果を積み、param を触り、絵が変わるまでの一本道。
//!
//! ## ここに出る効果は、engine が実際に描ける物だけ
//!
//! 以前この面は Inspector の宣言に直書きされた `TURBULENT DISPLACE` / `8 params` /
//! `Amount`・`Size`・`Offset`・`Complexity`・`Evolution` だった。**その plugin は
//! 存在しない** — engine が写せる `plugin_id` は `"motolii.glow"` 1本きりで、他は
//! 無音で skip される(= 積んでも pass が1枚も生えない)。押せるのに何も起きない物は
//! 不合格(Q0)なので、宣言ごと落とした。
//!
//! **カタログの家は engine で、front は写しを持たない。**
//! [`motolii_engine::known_effects`] が「今この合成器が実際に描ける plugin_id と、その
//! named param の名前・既定値・宣言された範囲」を返す唯一の正本で(engine 側の
//! `known_effects_are_all_actually_drawable` が、載っている plugin_id は
//! `translate_effect_passes` が必ず pass を積めることを縛っている)、この面はそれを
//! **読むだけ**である。効果が増えるのは engine の口が増えた時で、ここは1行も動かない。
//!
//! front がこの上に足しているのは2つだけで、どちらも engine の写しではない:
//!
//! - 表示名([`plugin_title`] / [`param_label`])— `plugin_id` と param 名からの**派生**
//!   であって第二の名前ではない。engine が知らない名前をここで作れない
//! - 積んだ瞬間に書く値([`seed_value`])— engine の既定は「意味の中立値」であって
//!   「積んだと分かる値」ではない。名前が一致しなければ黙って engine の既定へ落ちる
//!
//! ## store とのつなぎ方
//!
//! 書く動詞は `Intent::SetEffects`(列の出し入れ)と `Intent::SetTrack`(param と
//! on/off)の2つだけで、**effect 専用の Intent は無い**(裁定72/213: param も
//! enabled も平坦 track に乗るので新機構ゼロ)。[`apply`] が
//! `main.rs` の `toggle_lane_flag_from_timeline` と同じ手順を踏む —
//! 読む → `drop(store)` → 名指した物だけ差し替える → `apply_all` 1回 = 1 undo →
//! 状態行を返す。
use std::collections::HashMap;

use makepad_widgets::*;
use motolii_engine::{known_effects, EffectDescriptor, EffectParamDescriptor};
use motolii_shell_state::Session;
use motolii_store::{
    property, Document, EffectId, EffectInstance, Interp, Intent, Keyframe, KeyframeTrack, LayerId,
    PropertyId, RationalTime, StoreView, Value,
};

use crate::inspector_surface::ScrubValueAction;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    let FxRule = SolidView{width: Fill height: mod.tokens.rule.size show_bg: true new_batch: true draw_bg.color: mod.tokens.rule.seam}

    // ON バッジ — Inspector の値行と同じ「箱いっぱいの CheckBox」。状態は色だけでなく
    // **語**(ON / OFF)も変える(色覚に預けない)
    let FxOnChip = CheckBoxFlat{
        width: 26
        height: mod.tokens.size.chip
        padding: 0
        margin: Inset{right: mod.tokens.space.s3}
        align: Align{x: 0.5 y: 0.5}
        text: "ON"
        active: true
        label_walk: Walk{width: Fit height: Fit margin: 0.}
        draw_bg.size: 26.0
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_bg.color: mod.tokens.face.well
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.color_active: mod.tokens.accent.on
        draw_bg.color_focus: mod.tokens.face.hover
        draw_bg.color_disabled: mod.tokens.face.well
        draw_bg.mark_color: #x00000000
        draw_bg.mark_color_hover: #x00000000
        draw_bg.mark_color_down: #x00000000
        draw_bg.mark_color_active: #x00000000
        draw_bg.mark_color_active_hover: #x00000000
        draw_bg.mark_color_focus: #x00000000
        draw_bg.mark_color_disabled: #x00000000
        draw_text.color: mod.tokens.ink.muted
        draw_text.color_hover: mod.tokens.ink.body
        draw_text.color_down: mod.tokens.ink.muted
        draw_text.color_active: mod.tokens.ink.on_fill
        draw_text.color_focus: mod.tokens.ink.body
        draw_text.color_disabled: mod.tokens.ink.faint
        draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}
    }

    // 外す — 面を持たない字。触れると赤(record)へ寄る
    let FxDrop = ButtonFlatter{
        width: 16
        height: mod.tokens.size.chip
        margin: Inset{right: mod.tokens.space.s3}
        padding: 0
        text: "x"
        draw_text.color: mod.tokens.ink.faint
        draw_text.color_hover: mod.tokens.accent.record
        draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}
    }

    // 積む — 窪みの札。文言は Rust 側がカタログから入れる(ここに効果名を書かない)
    let FxAddButton = ButtonFlat{
        width: Fit
        height: mod.tokens.size.chip
        padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4}
        draw_bg.color: mod.tokens.face.well
        draw_bg.color_hover: mod.tokens.face.hover
        draw_bg.color_down: mod.tokens.face.down
        draw_bg.border_size: 0.0
        draw_bg.border_radius: 0.0
        draw_text.color: mod.tokens.ink.body
        draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}
    }

    mod.widgets.FxStackBase = #(FxStack::register_widget(vm))
    mod.widgets.FxStack = set_type_default() do mod.widgets.FxStackBase{
        width: Fill
        height: Fill
        flow: Down
        show_bg: true
        new_batch: true
        draw_bg.color: mod.tokens.face.panel

        // 節見出し — Inspector の SectionCap と同じ 18px / face.area / 太字 xs
        cap := SolidView{width: Fill height: 18 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s4 right: mod.tokens.space.s4} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
            name := InkLabel{text: "FX STACK" width: Fill draw_text.color: mod.tokens.ink.muted draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.xs}}
            count := InkLabel{width: Fit draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
        }
        cap_rule := FxRule{}

        // 行は1枚も宣言に無い。全部 model から出る(browser の card_grid と同じ流儀)
        list := FlatList{width: Fill height: Fill flow: Down

            // 効果1つ — 左端の色付き 3px が「これが効果の頭」、右端が ON と外す
            FxHead := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.area
                fx_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.record}
                name := InkLabel{width: Fill padding: Inset{left: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.strong draw_text.text_style: theme.font_bold{font_size: mod.tokens.text.sm}}
                note := InkLabel{width: Fit padding: Inset{right: mod.tokens.space.s3} draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.xs}}
                power := FxOnChip{}
                drop := FxDrop{}
            }

            // param 1つ — Inspector の PropertyRowOne と同じ 25px / 左 3px / 菱形。
            // 掴み代の目盛り(step/precision)は宣言に置く(--hot で振れる)。
            // **min/max は書かない** — `known_effects()` の `EffectParamDescriptor::range`
            // は今どの param でも `None`(engine のシェーダが範囲を宣言していない)。
            // 無い範囲を front が発明すると、engine が受ける値を UI が拒む嘘になる
            FxParam := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                type_bar := SolidView{width: 3 height: Fill show_bg: true new_batch: true draw_bg.color: mod.tokens.accent.alt}
                name := InkLabel{width: 74 padding: Inset{left: mod.tokens.space.s5} draw_text.color: mod.tokens.ink.body draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
                v := ScrubValue{width: 56 step: 0.01 precision: 3}
                keyed := InkLabel{width: Fill align: Align{x: 1.0} padding: Inset{right: mod.tokens.space.s4} draw_text.color: mod.tokens.accent.on draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
            }

            // 積む札の行
            FxAdd := SolidView{width: Fill height: 25 flow: Right align: Align{y: 0.5} padding: Inset{left: mod.tokens.space.s5} show_bg: true new_batch: true draw_bg.color: mod.tokens.face.panel
                add := FxAddButton{}
            }

            // 触れない状態(選択なし・ロック中・読めない)を無言の空白で語らない
            FxNote := View{width: Fill height: Fit padding: Inset{left: mod.tokens.space.s5 top: mod.tokens.space.s3 right: mod.tokens.space.s4 bottom: mod.tokens.space.s2}
                note := InkLabel{width: Fill draw_text.color: mod.tokens.ink.faint draw_text.text_style: theme.font_regular{font_size: mod.tokens.text.sm}}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// カタログ — 家は engine。ここは読むだけ(モジュール doc 参照)
// ---------------------------------------------------------------------------

/// この `plugin_id` を engine が実際に描けるか。**描けない物は名前も param も出さない。**
fn plugin_spec(plugin_id: &str) -> Option<&'static EffectDescriptor> {
    known_effects()
        .iter()
        .find(|spec| spec.plugin_id == plugin_id)
}

/// `"motolii.glow"` → `"GLOW"`。**`plugin_id` からの派生であって第二の名前ではない** —
/// engine が知らない plugin の題をここで作れない、という形にしてある
/// (`TURBULENT DISPLACE` 事故は「front が engine の知らない名前を持てた」ことが原因)。
fn plugin_title(plugin_id: &str) -> String {
    plugin_id
        .rsplit('.')
        .next()
        .unwrap_or(plugin_id)
        .replace('_', " ")
        .to_uppercase()
}

/// `"threshold"` → `"Threshold"`。同じ理由で param 名からの派生。
fn param_label(name: &str) -> String {
    name.split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// 積んだ瞬間に track として書き込む値。
///
/// engine の `default` は **track を1本も持たない時に使われる意味の中立値**であって、
/// 「積んだと分かる値」ではない。glow の既定 `threshold = 1.0` は 8bit SDR の層では
/// bright-pass が何も抜かないので、積んでも絵が1画素も変わらない — `motolii-fixture` が
/// 「確実に bright-pass が起動する値」として同じ 0.35 / 1.5 を明示しているのと同じ話。
/// 積んだのに何も起きないのは「効いたように見えて黙って戻る」と同じ手触りなので、
/// **積む = 見える値を実際に書く**。書いてしまえば engine の既定は経路から外れる。
///
/// **これは engine の写しではない** — engine が持っていない front 側の見せ方の判断で、
/// 名前が一致しなければ黙って engine の既定へ落ちる(存在しない param を発明しない)。
fn seed_value(plugin_id: &str, param: &EffectParamDescriptor) -> f64 {
    match (plugin_id, param.name) {
        ("motolii.glow", "threshold") => 0.35,
        ("motolii.glow", "intensity") => 1.5,
        _ => param.default,
    }
}

// ---------------------------------------------------------------------------
// 面へ渡す投影(TimelineModel と同じ身分 — widget は Document を持たない)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum FxRow {
    /// 効果1つの頭。
    Head {
        effect: u32,
        title: String,
        note: String,
        enabled: bool,
    },
    /// その効果の param 1つ。`slot` はカタログ内の並び(entry id の一部)。
    Param {
        effect: u32,
        slot: usize,
        name: &'static str,
        label: String,
        value: f64,
        keyed: bool,
    },
    /// 積める効果1つ([`known_effects`] の添字)。
    Add { plugin: usize, title: String },
    /// 触れない理由、または読み取り専用の1行。
    Note { seq: u64, text: String },
}

#[derive(Clone, Debug, Default)]
pub struct FxStackModel {
    /// 見出しの右に出る計数。
    pub caption: String,
    pub rows: Vec<FxRow>,
}

/// 面が外へ出す唯一の口。**どのレイヤーかは言わない** — 選択は `main.rs` の
/// `Session` が持つ真実で、`ScrubValueAction` が `prop` しか運ばないのと同じ線引き。
#[derive(Clone, Debug, Default)]
pub enum FxStackAction {
    #[default]
    None,
    Add {
        plugin: usize,
    },
    Remove {
        effect: u32,
    },
    SetEnabled {
        effect: u32,
        on: bool,
    },
    /// **確定値だけ**。ドラッグ途中(`ScrubValueAction::Changed`)はここへ来ない —
    /// 1ジェスチャ = 1書き込み = 1 undo。
    SetParam {
        effect: u32,
        name: &'static str,
        value: f64,
    },
}

/// 行 → FlatList の entry id。**種を上位ビットに入れる** — entry id が同じなら
/// `FlatList::item` は template を無視して前の widget を返すので、種が変わり得る
/// 番号(単なる行番号)を使うと param 行の場所に頭行の widget が現れる。
fn row_entry(row: &FxRow) -> LiveId {
    match row {
        FxRow::Head { effect, .. } => LiveId((1 << 48) | u64::from(*effect)),
        FxRow::Param { effect, slot, .. } => {
            LiveId((2 << 48) | (u64::from(*effect) << 8) | *slot as u64)
        }
        FxRow::Add { plugin, .. } => LiveId((3 << 48) | *plugin as u64),
        FxRow::Note { seq, .. } => LiveId((4 << 48) | *seq),
    }
}

fn row_template(row: &FxRow) -> LiveId {
    match row {
        FxRow::Head { .. } => live_id!(FxHead),
        FxRow::Param { .. } => live_id!(FxParam),
        FxRow::Add { .. } => live_id!(FxAdd),
        FxRow::Note { .. } => live_id!(FxNote),
    }
}

// ---------------------------------------------------------------------------
// store → 投影
// ---------------------------------------------------------------------------

fn note_only(text: &str) -> FxStackModel {
    FxStackModel {
        caption: String::new(),
        rows: vec![FxRow::Note {
            seq: 0,
            text: text.to_owned(),
        }],
    }
}

fn effect_count(count: usize) -> String {
    match count {
        0 => "no effect".to_owned(),
        1 => "1 effect".to_owned(),
        n => format!("{n} effects"),
    }
}

/// 選択レイヤーの効果スタックを面の形へ写す。**読むだけ**。
pub fn model_for(view: &StoreView<'_>, session: &Session) -> FxStackModel {
    let Some(layer) = session.selection else {
        return note_only("Select a layer to stack effects on it.");
    };
    if !view.has_layer(layer) {
        return note_only("Select a layer to stack effects on it.");
    }
    let Some(composition) = view.composition().ok().flatten() else {
        return note_only("The composition is unreadable, so effect values cannot be shown.");
    };
    let Ok(t) = RationalTime::try_from_frame(session.playhead.max(0), composition.fps) else {
        return note_only("The playhead does not map to a time.");
    };
    let effects = view.effects(layer).unwrap_or_default();

    // ロック中/凍結中は store が書き込みを拒む(`check_not_locked`/`check_not_frozen`)。
    // 拒まれると分かっている操作面を出さない — 触れそうで触れない物は不合格(Q0)。
    // 代わりに**同じ中身を字として**出す: 隠すと「効果が無い」と読めてしまう。
    let locked = view
        .attrs(layer)
        .ok()
        .flatten()
        .map(|attrs| attrs.locked)
        .unwrap_or(false);
    let frozen = view.frozen_ancestor(layer).ok().flatten();
    if locked || frozen.is_some() {
        let mut rows = vec![FxRow::Note {
            seq: 0,
            text: if locked {
                "This layer is locked. Unlock it to change effects.".to_owned()
            } else {
                "This layer is inside a frozen group. Unfreeze it to change effects.".to_owned()
            },
        }];
        for (index, effect) in effects.iter().enumerate() {
            let title = plugin_spec(&effect.plugin_id)
                .map(|spec| plugin_title(spec.plugin_id))
                .unwrap_or_else(|| effect.plugin_id.clone());
            rows.push(FxRow::Note {
                seq: index as u64 + 1,
                text: format!("{title}  (read only)"),
            });
        }
        return FxStackModel {
            caption: effect_count(effects.len()),
            rows,
        };
    }

    let mut rows = Vec::new();
    for effect in &effects {
        let spec = plugin_spec(&effect.plugin_id);
        // キーを打っていない = 既定で有効(`PropertyId::effect_enabled` の doc)。
        let enabled = match view.value_at(layer, &PropertyId::effect_enabled(effect.id), t) {
            Ok(Some(Value::Bool(on))) => on,
            _ => true,
        };
        rows.push(FxRow::Head {
            effect: effect.id.0,
            title: spec
                .map(|spec| plugin_title(spec.plugin_id))
                .unwrap_or_else(|| effect.plugin_id.clone()),
            // 描ける物と描けない物を黙って同じ顔にしない。engine に pass が無い
            // plugin は積まれていても絵に出ないので、そう書く。
            note: match spec {
                Some(spec) => format!("{} params", spec.params.len()),
                None => "no renderer".to_owned(),
            },
            enabled,
        });
        let Some(spec) = spec else {
            continue;
        };
        for (slot, param) in spec.params.iter().enumerate() {
            let Ok(property) = PropertyId::effect_param(effect.id, param.name) else {
                continue;
            };
            let keyed = match view.track(layer, &property) {
                Ok(Some(track)) => !track.keys().is_empty(),
                _ => false,
            };
            let value = match view.value_at(layer, &property, t) {
                Ok(Some(Value::F64(value))) => value,
                // track が無い param は engine の既定で描かれている。その値を出す
                // (0 と書いて違う絵を出すのが一番悪い)。
                _ => param.default,
            };
            rows.push(FxRow::Param {
                effect: effect.id.0,
                slot,
                name: param.name,
                label: param_label(param.name),
                value,
                keyed,
            });
        }
    }
    for (plugin, spec) in known_effects().iter().enumerate() {
        rows.push(FxRow::Add {
            plugin,
            title: format!("+ {}", plugin_title(spec.plugin_id)),
        });
    }
    FxStackModel {
        caption: effect_count(effects.len()),
        rows,
    }
}

// ---------------------------------------------------------------------------
// 投影 → store
// ---------------------------------------------------------------------------

/// 書いた結果。`wrote` が真の時だけ絵と投影を引き直す。
pub struct FxWrite {
    pub status: String,
    pub wrote: bool,
}

fn refused(status: impl Into<String>) -> FxWrite {
    FxWrite {
        status: status.into(),
        wrote: false,
    }
}

/// 静止値1つの track(キーを1本、時刻0、Hold)。fixture が glow の param を書くのと
/// 同じ形。
fn constant_track(t0: RationalTime, value: Value) -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    track.insert(Keyframe {
        t: t0,
        value,
        interp: Interp::Hold,
        spatial: None,
    });
    track
}

/// 効果1つぶんの track を全部空にする(param・enabled)。列から外した効果の
/// param 行が Timeline に残り続けるのを防ぐ — `StoreView::properties` は
/// component の有無で列挙するので、空にしないと「消したのに行がある」になる。
fn clear_effect_tracks(view: &StoreView<'_>, layer: LayerId, effect: EffectId) -> Vec<Intent> {
    let prefix = format!("{}{effect}.", property::EFFECT_PREFIX);
    view.properties(layer)
        .into_iter()
        .filter(|property| property.name().starts_with(prefix.as_str()))
        .map(|property| Intent::SetTrack {
            layer,
            property,
            track: KeyframeTrack::new(),
        })
        .collect()
}

/// # 効果を store へ書く
///
/// `main.rs` の `toggle_lane_flag_from_timeline` と同じ手順。動詞が
/// `SetEffects`/`SetTrack` に変わるだけで、順序も理由も同じ:
/// 読む → `drop(store)` → 名指した物だけ差し替える → `apply_all` 1回 → 状態行。
///
/// ## param の値をどこへ書くか(AE の指の規約)
///
/// - **キーが1本も無い property** → 時刻0に Hold のキーを1本置く(= 静止値)。
///   AE で「ストップウォッチを押していない property の値を変える」に当たる
/// - **既にキーがある property** → **プレイヘッドの時刻にキーを打つ**。AE で
///   「キーのある property を別の時刻で触るとそこにキーが増える」に当たる。
///   打ったキーは Timeline の property 行にそのまま現れるので、増えたことは見える。
///   新しいキーの `interp` は**直前のキーから写す** — 区間イージングは別の動詞で、
///   値を触る意図はそれを名指していない(裁定271)
///
/// ## on/off だけは時刻を持たない
///
/// `effect.{id}.enabled` は裁定213 で animatable になったが、バッジを押す意図は
/// 「今この効果を切る」であって「この時刻から切る」ではない。プレイヘッドにキーを
/// 打つと**別の時刻へ動かした瞬間に黙って戻る** — 一度でもそれをやると、利用者は
/// 他のどの操作が本物かも分からなくなる。よってここは常に時刻0の Hold 1本で
/// 丸ごと置き換える(front には enabled を打ち分ける UI がまだ無いので、
/// この置き換えで潰れる物は今のところ存在しない)。
pub fn apply(doc: &mut Document, session: &Session, action: &FxStackAction) -> FxWrite {
    if matches!(action, FxStackAction::None) {
        return refused(String::new());
    }
    let Some(layer) = session.selection else {
        return refused("FX: no layer is selected");
    };

    // 1. 今の値を store から読む。
    let store = doc.view();
    if !store.has_layer(layer) {
        return refused(format!("FX: layer {} no longer exists", layer.0));
    }
    let Some(composition) = store.composition().ok().flatten() else {
        return refused("FX: the composition is unreadable");
    };
    let fps = composition.fps;
    let Ok(t0) = RationalTime::try_from_frame(0, fps) else {
        return refused("FX: frame 0 does not map to a time");
    };
    let effects = match store.effects(layer) {
        Ok(effects) => effects,
        Err(error) => return refused(format!("FX: cannot read the effect stack: {error}")),
    };

    let (intents, status): (Vec<Intent>, String) = match action {
        FxStackAction::None => return refused(String::new()),

        FxStackAction::Add { plugin } => {
            let Some(spec) = known_effects().get(*plugin) else {
                return refused("FX: that effect is not in the catalog");
            };
            let next = effects
                .iter()
                .map(|effect| effect.id.0)
                .max()
                .map_or(0, |max| max.saturating_add(1));
            let id = EffectId(next);
            if effects.iter().any(|effect| effect.id == id) {
                return refused("FX: ran out of effect ids on this layer");
            }
            let mut list = effects.clone();
            list.push(EffectInstance {
                id,
                plugin_id: spec.plugin_id.to_owned(),
            });
            let mut intents = vec![Intent::SetEffects {
                layer,
                effects: list,
            }];
            // 積んだ瞬間に見える値を書く([`seed_value`] の doc)。
            for param in spec.params {
                let Ok(property) = PropertyId::effect_param(id, param.name) else {
                    continue;
                };
                intents.push(Intent::SetTrack {
                    layer,
                    property,
                    track: constant_track(t0, Value::F64(seed_value(spec.plugin_id, param))),
                });
            }
            (
                intents,
                format!(
                    "FX: {} added to layer {}",
                    plugin_title(spec.plugin_id),
                    layer.0
                ),
            )
        }

        FxStackAction::Remove { effect } => {
            let id = EffectId(*effect);
            let Some(instance) = effects.iter().find(|instance| instance.id == id) else {
                return refused(format!("FX: effect {effect} is no longer on this layer"));
            };
            let title = plugin_spec(&instance.plugin_id)
                .map(|spec| plugin_title(spec.plugin_id))
                .unwrap_or_else(|| format!("effect {effect}"));
            let list: Vec<EffectInstance> = effects
                .iter()
                .filter(|instance| instance.id != id)
                .cloned()
                .collect();
            let mut intents = vec![Intent::SetEffects {
                layer,
                effects: list,
            }];
            intents.extend(clear_effect_tracks(&store, layer, id));
            (
                intents,
                format!("FX: {title} removed from layer {}", layer.0),
            )
        }

        FxStackAction::SetEnabled { effect, on } => {
            let id = EffectId(*effect);
            if !effects.iter().any(|instance| instance.id == id) {
                return refused(format!("FX: effect {effect} is no longer on this layer"));
            }
            (
                vec![Intent::SetTrack {
                    layer,
                    property: PropertyId::effect_enabled(id),
                    track: constant_track(t0, Value::Bool(*on)),
                }],
                format!("FX: effect {effect} {}", if *on { "ON" } else { "OFF" }),
            )
        }

        FxStackAction::SetParam {
            effect,
            name,
            value,
        } => {
            let id = EffectId(*effect);
            if !effects.iter().any(|instance| instance.id == id) {
                return refused(format!("FX: effect {effect} is no longer on this layer"));
            }
            let Ok(property) = PropertyId::effect_param(id, name) else {
                return refused(format!("FX: `{name}` is not a usable parameter name"));
            };
            let mut track = store.track(layer, &property).ok().flatten().unwrap_or_default();
            let (t, interp) = if track.keys().is_empty() {
                (t0, Interp::Hold)
            } else {
                let Ok(t) = RationalTime::try_from_frame(session.playhead.max(0), fps) else {
                    return refused("FX: the playhead does not map to a time");
                };
                // 直前のキーから写す。区間イージングを勝手に発明しない。
                let interp = track
                    .keys()
                    .iter()
                    .rev()
                    .find(|key| key.t <= t)
                    .or_else(|| track.keys().first())
                    .map(|key| key.interp)
                    .unwrap_or(Interp::Hold);
                (t, interp)
            };
            let keys_before = track.keys().len();
            track.insert(Keyframe {
                t,
                value: Value::F64(*value),
                interp,
                spatial: None,
            });
            let added = track.keys().len() > keys_before;
            (
                vec![Intent::SetTrack {
                    layer,
                    property,
                    track,
                }],
                format!(
                    "FX: effect {effect} {name} = {value:.3}{}",
                    if added { "  ·  KEY ADDED" } else { "" }
                ),
            )
        }
    };

    // 2. `view()` は借用なので、持ったままでは書けない。
    drop(store);

    // 3. 1ジェスチャ = 1呼び出し = 1 undo。
    if let Err(error) = doc.apply_all(intents) {
        return refused(format!("FX: {error}"));
    }
    FxWrite {
        status,
        wrote: true,
    }
}

// ---------------------------------------------------------------------------
// 面
// ---------------------------------------------------------------------------

#[derive(Script, ScriptHook, WidgetRegister)]
pub struct FxStack {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[deref]
    view: View,
    /// 面が映している物。**正本ではない**(正本は Document)。
    #[rust]
    model: FxStackModel,
    /// その欄へ最後に押し込んだ値。**掴んでいる最中/打っている最中の欄を上書き
    /// しない**ための記録 — model の値が変わった時だけ押し込む。毎フレーム押し込むと
    /// ドラッグ中の値が毎フレーム元へ戻り、タイプ中の文字は打つそばから消える。
    ///
    /// live reload はこの記録を捨てる(`Event::LiveEdit`)。reload は widget を宣言
    /// 状態(欄は空)へ戻すので、記録を残したままだと「押し込み済み」と思って何も
    /// 書かず、値が全部 0 に見える。
    #[rust]
    shown: HashMap<LiveId, f64>,
    /// 同じ理由の ON バッジ版。`set_active` は animator を毎回 cut し直すので、
    /// 変わっていない時に呼ばない。
    #[rust]
    powered: HashMap<LiveId, bool>,
}

impl FxStack {
    pub fn set_model(&mut self, cx: &mut Cx, model: FxStackModel) {
        self.model = model;
        self.view.redraw(cx);
    }
}

impl WidgetNode for FxStack {
    fn widget_uid(&self) -> WidgetUid {
        self.uid
    }

    fn children(&self, visit: &mut dyn FnMut(LiveId, WidgetRef)) {
        self.view.children(visit);
    }

    fn walk(&mut self, cx: &mut Cx) -> Walk {
        self.view.walk(cx)
    }

    fn area(&self) -> Area {
        self.view.area()
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.view.redraw(cx);
    }
}

impl Widget for FxStack {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // live reload は面を宣言状態へ戻す。押し込み済みの記録も一緒に捨てないと、
        // 空になった欄へ二度と値が入らない(`shown` の doc)。
        if matches!(event, Event::LiveEdit) {
            self.shown.clear();
            self.powered.clear();
            self.view.redraw(cx);
        }
        let actions = cx.capture_actions(|cx| self.view.handle_event(cx, event, scope));
        if !actions.is_empty() {
            let mut emit: Vec<FxStackAction> = Vec::new();
            // 今のモデルに居る行だけを見る。列から消えた効果の widget は FlatList に
            // 残り続けるので、素通しにすると「消したはずの行」が返事をしてしまう。
            let rows: Vec<(LiveId, FxRow)> = self
                .model
                .rows
                .iter()
                .map(|row| (row_entry(row), row.clone()))
                .collect();
            let list = self.view.flat_list(cx, ids!(list));
            for (entry, item) in list.items_with_actions(&actions) {
                let Some((_, row)) = rows.iter().find(|(id, _)| *id == entry) else {
                    continue;
                };
                match row {
                    FxRow::Head { effect, .. } => {
                        if let Some(on) = item.check_box(cx, ids!(power)).changed(&actions) {
                            emit.push(FxStackAction::SetEnabled {
                                effect: *effect,
                                on,
                            });
                        }
                        if item.button(cx, ids!(drop)).clicked(&actions) {
                            emit.push(FxStackAction::Remove { effect: *effect });
                        }
                    }
                    FxRow::Param { effect, name, .. } => {
                        let cell = item.widget(cx, ids!(v));
                        if cell.is_empty() {
                            continue;
                        }
                        for action in actions.filter_widget_actions(cell.widget_uid()) {
                            // 確定だけを店へ運ぶ。ドラッグ途中は欄の中だけで動く。
                            if let ScrubValueAction::Committed { value, .. } = action.cast() {
                                emit.push(FxStackAction::SetParam {
                                    effect: *effect,
                                    name,
                                    value,
                                });
                            }
                        }
                    }
                    FxRow::Add { plugin, .. } => {
                        if item.button(cx, ids!(add)).clicked(&actions) {
                            emit.push(FxStackAction::Add { plugin: *plugin });
                        }
                    }
                    FxRow::Note { .. } => {}
                }
            }
            for action in emit {
                cx.widget_action(self.uid, action);
            }
        }
        cx.extend_actions(actions);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let rows = self.model.rows.clone();
        let caption = self.model.caption.clone();
        self.view.label(cx, ids!(cap.count)).set_text(cx, &caption);
        let mut shown = std::mem::take(&mut self.shown);
        let mut powered = std::mem::take(&mut self.powered);

        while let Some(step) = self.view.draw_walk(cx, scope, walk).step() {
            let list = step.as_flat_list();
            let Some(mut list) = list.borrow_mut() else {
                continue;
            };
            for row in &rows {
                let entry = row_entry(row);
                let Some(item) = list.item(cx, entry, row_template(row)) else {
                    continue;
                };
                match row {
                    FxRow::Head {
                        title,
                        note,
                        enabled,
                        ..
                    } => {
                        item.label(cx, ids!(name)).set_text(cx, title);
                        item.label(cx, ids!(note)).set_text(cx, note);
                        item.widget(cx, ids!(power))
                            .set_text(cx, if *enabled { "ON" } else { "OFF" });
                        if powered.get(&entry) != Some(enabled) {
                            item.check_box(cx, ids!(power))
                                .set_active(cx, *enabled, Animate::No);
                            powered.insert(entry, *enabled);
                        }
                    }
                    FxRow::Param {
                        label,
                        value,
                        keyed,
                        ..
                    } => {
                        item.label(cx, ids!(name)).set_text(cx, label);
                        item.label(cx, ids!(keyed))
                            .set_text(cx, if *keyed { "\u{25c6}" } else { "\u{25c7}" });
                        let stale = shown
                            .get(&entry)
                            .map(|previous| previous != value)
                            .unwrap_or(true);
                        if stale {
                            item.widget(cx, ids!(v)).set_text(cx, &format!("{value:.6}"));
                            shown.insert(entry, *value);
                        }
                    }
                    FxRow::Add { title, .. } => {
                        item.button(cx, ids!(add)).set_text(cx, title);
                    }
                    FxRow::Note { text, .. } => {
                        item.label(cx, ids!(note)).set_text(cx, text);
                    }
                }
                item.draw_all_unscoped(cx);
            }
        }

        self.shown = shown;
        self.powered = powered;
        DrawStep::done()
    }
}
