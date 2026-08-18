//! Inspector の read-model 投影 — 選択1層ぶんの**読み取り専用**要約。
//!
//! 移植元は egui 版 `crates/motolii-ui/src/inspector_panel/read_model.rs`
//! (JS decoder D1〜D6 の意味の写し)。ここはその iced 版で、行の意味を
//! **同じ規則**で導く上に、2026-08-13 裁定の mapping どおり Transform を
//! `Position / Scale / Rotation / Opacity` の4行へ広げている(egui 版は
//! Position / Opacity の2行が閉域だった。エディタの適用口 `ParamRef` は
//! 元から4つとも受けるので、行を広げても書き口は増えていない)。
//!
//! **ここは投影だけで、Document を書かない**(single writer)。書く道は
//! `ShellGateway::dispatch` の編集 intent だけである。状態はぜんぶ
//! **accepted snapshot から毎回導出**する — 面にも殻にも局所の key 状態・
//! M/S bool を持たない(2026-08-18 外部診断 F-03 / 2026-08-13 裁定の
//! optimistic 禁止)。

use motolii_core::RationalTime;
use motolii_doc::{DocParam, DocValue, Document, ItemEnvelope, LayerId, TrackItem};
use motolii_plugin::{PluginCatalog, Value};
use motolii_ui::blitz_shell::UiEditParam;

use crate::widgets_stub::KeyState;

/// Inspector が座る相手。**空も理由も黙らせない**(Q7 / Q3)。
#[derive(Debug, Clone, PartialEq)]
pub enum InspectorSeat {
    /// 選択なし。fixture へ落ちない(Q7)。
    NoSelection,
    /// 2つ以上選ばれている。property は出さない(egui 版と同じ)。
    Multi(usize),
    /// 投影できない理由(存在しない layer・catalog 不整合など)。
    Unreadable(String),
    /// 1層ぶんの投影。
    Ready(InspectorModel),
}

/// 選択1層の要約。値は全て snapshot の評価結果で、この型は状態を持たない。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorModel {
    pub layer_id: u64,
    pub layer_name: String,
    /// `clip` / `group`(decoder R6 と同じ語彙)。
    pub item_kind: &'static str,
    /// group のときだけ載る。
    pub child_count: Option<usize>,
    /// M / S の押下状態。**正本は Document**(`ItemEnvelope`)。
    pub muted: bool,
    pub solo: bool,
    /// Transform の4行(Position / Scale / Rotation / Opacity の順)。
    pub transform: Vec<ParamRow>,
    /// 共有 FX(`Document.effect_definitions`)。egui 版と同じく **recipe 単位**。
    pub effects: Vec<EffectSection>,
}

impl InspectorModel {
    pub fn transform_row(&self, param: UiEditParam) -> Option<&ParamRow> {
        self.transform.iter().find(|row| row.param == param)
    }
}

/// Transform 1行。Vec2(Position / Scale)は `components` が `[x, y]`、
/// scalar(Rotation / Opacity)は `[v]`。
#[derive(Debug, Clone, PartialEq)]
pub struct ParamRow {
    pub param: UiEditParam,
    /// playhead 時刻で評価した成分値。キー列はここでは開かない。
    pub components: Vec<f64>,
    pub key_state: KeyState,
    /// 直打ち・◇ を受け付けるか。閉じていない `DocParam` 種
    /// (`Data` / `Vec2Axes` / `LookAt` / `Follow`)は `false` で、行は出すが
    /// 触れる部品を置かない(触れそうで触れない物を作らない — Q0)。
    pub editable: bool,
}

/// 共有 FX 1 recipe。
#[derive(Debug, Clone, PartialEq)]
pub struct EffectSection {
    pub definition_id: u64,
    pub plugin_id: String,
    /// ON/OFF の押下状態。出所は Document の `EffectDefinition.enabled`。
    pub enabled: bool,
    /// 行を出す param だけ(F64 / Vec2 / Vec3 / Color)。`AssetRef` / `List` は
    /// widget 未決なので**行を出さない**(egui 版 parameter_control の慣行。
    /// AssetRef 同格 = 最新 main の扱い)。
    pub params: Vec<EffectParamRow>,
}

/// FX param 1行。値は plugin 契約(`NodeDesc.params`)の既定値
/// (egui 版 read-model と同じで、文書 instance の値ではない)。
#[derive(Debug, Clone, PartialEq)]
pub struct EffectParamRow {
    pub id: String,
    pub value: EffectParamValue,
}

/// 行にする値の閉じた語彙。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectParamValue {
    F64(f64),
    Vec2([f64; 2]),
    Vec3([f64; 3]),
    Color([f64; 4]),
}

/// snapshot + catalog + 選択 + playhead から Inspector の座席を導く。
///
/// 判断は egui 版 `InspectorPanel::seat_live` と同じ3分岐(なし / 1つ / 多数)。
pub fn project_inspector(
    document: &Document,
    catalog: &PluginCatalog,
    selection: &[LayerId],
    playhead: RationalTime,
) -> InspectorSeat {
    match selection {
        [] => InspectorSeat::NoSelection,
        [one] => match project_one(document, catalog, *one, playhead) {
            Ok(model) => InspectorSeat::Ready(model),
            Err(reason) => InspectorSeat::Unreadable(reason),
        },
        many => InspectorSeat::Multi(many.len()),
    }
}

fn project_one(
    document: &Document,
    catalog: &PluginCatalog,
    target: LayerId,
    playhead: RationalTime,
) -> Result<InspectorModel, String> {
    // R7 / R7-b(read_model.rs と同じ判断): target item の解決。
    let mut matches: Vec<&TrackItem> = Vec::new();
    for track in &document.tracks {
        collect_items_for_layer(&track.items, target, &mut matches);
    }
    let layer_known = document.layers.iter().any(|(id, _)| id == target);
    let item = match matches.as_slice() {
        [] if !layer_known => {
            return Err(format!(
                "target layer {} does not exist in the document",
                target.get()
            ))
        }
        [] => return Err(format!("target layer {} has no track item", target.get())),
        [item] => *item,
        many => {
            return Err(format!(
                "target layer {} resolves to {} track items",
                target.get(),
                many.len()
            ))
        }
    };
    let (envelope, child_count) = match item {
        TrackItem::Clip(clip) => (&clip.envelope, None),
        TrackItem::Group(group) => (&group.envelope, Some(group.children.len())),
    };
    let layer_name = document
        .layers
        .iter()
        .find(|(id, _)| *id == target)
        .map(|(_, name)| name.to_owned())
        .unwrap_or_default();

    // R9: 共有 recipe を catalog の契約で結ぶ。解決できない plugin は理由ごと断る。
    let mut effects = Vec::with_capacity(document.effect_definitions.len());
    for definition in &document.effect_definitions {
        let Some(contract) = catalog.get(&definition.plugin_id) else {
            return Err(format!(
                "plugin `{}` is not in the catalog",
                definition.plugin_id
            ));
        };
        effects.push(EffectSection {
            definition_id: definition.id.get(),
            plugin_id: definition.plugin_id.clone(),
            enabled: definition.enabled,
            params: contract
                .node
                .params
                .iter()
                .filter_map(|param| {
                    effect_param_value(&param.default).map(|value| EffectParamRow {
                        id: param.id.to_owned(),
                        value,
                    })
                })
                .collect(),
        });
    }

    Ok(InspectorModel {
        layer_id: target.get(),
        layer_name,
        item_kind: match item {
            TrackItem::Clip(_) => "clip",
            TrackItem::Group(_) => "group",
        },
        child_count,
        // M は Timeline 行と同じ約束(`ItemFlag::Mute => !visible`)。
        muted: !envelope.visible,
        solo: envelope.solo,
        transform: transform_rows(envelope, playhead),
        effects,
    })
}

/// 2026-08-13 mapping の4行。Position / Scale は Vec2、Rotation / Opacity は scalar。
fn transform_rows(envelope: &ItemEnvelope, at: RationalTime) -> Vec<ParamRow> {
    [
        (UiEditParam::Position, &envelope.transform.position),
        (UiEditParam::Scale, &envelope.transform.scale),
        (UiEditParam::Rotation, &envelope.transform.rotation),
        (UiEditParam::Opacity, &envelope.opacity),
    ]
    .into_iter()
    .map(|(param, doc_param)| param_row(param, doc_param, at))
    .collect()
}

/// 成分の数(Vec2 = 2、scalar = 1)。
pub fn arity(param: UiEditParam) -> usize {
    match param {
        UiEditParam::Position | UiEditParam::Scale => 2,
        UiEditParam::Rotation | UiEditParam::Opacity => 1,
    }
}

/// 行に出す名前(egui 版 `param_label` と同じ字)。
pub fn param_label(param: UiEditParam) -> &'static str {
    match param {
        UiEditParam::Position => "Position",
        UiEditParam::Scale => "Scale",
        UiEditParam::Rotation => "Rotation",
        UiEditParam::Opacity => "Opacity",
    }
}

/// read_model.rs `project_editable_row` の写し。値は playhead の評価値で、
/// ◇/◆ の状態はキー列の有無と playhead 一致**だけ**から決まる。
fn param_row(param: UiEditParam, doc_param: &DocParam, at: RationalTime) -> ParamRow {
    let arity = arity(param);
    match doc_param {
        DocParam::Const(value) => ParamRow {
            param,
            components: components_of(value, arity),
            key_state: KeyState::Unkeyed,
            editable: true,
        },
        DocParam::Keyframes(track) => {
            let keyed_here = track.keys().iter().any(|key| key.t == at);
            ParamRow {
                param,
                components: eval_components(&track.eval(at), arity),
                key_state: if keyed_here {
                    KeyState::Current
                } else {
                    KeyState::Animated
                },
                // 空の keyframes track は prepare_* が断るので触らせない。
                editable: !track.keys().is_empty(),
            }
        }
        // 閉じていない種(Data / Vec2Axes / LookAt / Follow)。値も要約も発明しない。
        _ => ParamRow {
            param,
            components: vec![0.0; arity],
            key_state: KeyState::Unkeyed,
            editable: false,
        },
    }
}

/// FX param の行にする値。`AssetRef` / `List` は widget 未決 → 行を出さない。
fn effect_param_value(value: &Value) -> Option<EffectParamValue> {
    match value {
        Value::F64(v) => Some(EffectParamValue::F64(*v)),
        Value::Vec2(v) => Some(EffectParamValue::Vec2(*v)),
        Value::Vec3(v) => Some(EffectParamValue::Vec3(*v)),
        Value::Color(v) => Some(EffectParamValue::Color(*v)),
        Value::AssetRef(_) | Value::List(_) => None,
    }
}

/// `DocValue` を成分の並びへ(read_model.rs `components_of` の写し)。
fn components_of(value: &DocValue, arity: usize) -> Vec<f64> {
    let all: Vec<f64> = match value {
        DocValue::F64(v) => vec![*v],
        DocValue::Vec2(v) => v.to_vec(),
        DocValue::Vec3(v) => v.to_vec(),
        DocValue::Color(v) => v.to_vec(),
        DocValue::AssetRef(_) | DocValue::List(_) => Vec::new(),
    };
    all.into_iter().take(arity).collect()
}

/// 評価層の値を成分の並びへ(read_model.rs `eval_components` の写し)。
fn eval_components(value: &motolii_eval::Value, arity: usize) -> Vec<f64> {
    let all: Vec<f64> = match value {
        motolii_eval::Value::F64(v) => vec![*v],
        motolii_eval::Value::Vec2(v) => v.to_vec(),
        motolii_eval::Value::Vec3(v) => v.to_vec(),
        motolii_eval::Value::Color(v) => v.to_vec(),
        motolii_eval::Value::AssetRef(_) | motolii_eval::Value::List(_) => Vec::new(),
    };
    all.into_iter().take(arity).collect()
}

fn collect_items_for_layer<'doc>(
    items: &'doc [TrackItem],
    target: LayerId,
    out: &mut Vec<&'doc TrackItem>,
) {
    for item in items {
        let envelope = match item {
            TrackItem::Clip(clip) => &clip.envelope,
            TrackItem::Group(group) => &group.envelope,
        };
        if envelope.layer_id == target {
            out.push(item);
        }
        if let TrackItem::Group(group) = item {
            collect_items_for_layer(&group.children, target, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `List` / `AssetRef` の FX param は**行にならない**(widget 未決を
    /// 触れそうな行として出さない)。F64 / Vec2 / Vec3 / Color だけが行になる。
    #[test]
    fn unsupported_effect_params_get_no_row() {
        assert_eq!(effect_param_value(&Value::List(Vec::new())), None);
        assert_eq!(effect_param_value(&Value::AssetRef(3)), None);
        assert_eq!(
            effect_param_value(&Value::F64(0.5)),
            Some(EffectParamValue::F64(0.5))
        );
        assert_eq!(
            effect_param_value(&Value::Color([1.0, 0.5, 0.0, 1.0])),
            Some(EffectParamValue::Color([1.0, 0.5, 0.0, 1.0]))
        );
    }
}
