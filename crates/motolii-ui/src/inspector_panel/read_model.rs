//! Inspector read-model — 選択1層ぶんの**読み取り専用**要約。
//!
//! 意味の正本は `docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`
//! (`ui/motolii-web/src/read-model/inspectorReadModelDecoder.js` の出力形 D1〜D6)。
//! Rust 側はここでその形を**同じ意味で**持つ。JS decoder と食い違う変更をしない。
//!
//! JS 側との対応:
//! - `fixture_revision` = decoder R3(revision 1 のみ)
//! - `target { layer_id, layer_name, item_kind, child_count? }` = P1〜P3 / R7 / R7-b
//! - `effect_definitions[] { definition_id, plugin_id, params[] }` = P1 / R9。
//!   params は **文書の instance 値ではなく plugin 契約(`NodeDesc.params`)** から写す
//!   (JS decoder の `nodes` に当たるのが Rust では `PluginCatalog`)
//! - `position` = N6。`const {x, y}` / `animated` の2要約だけを閉じ、
//!   それ以外の `DocParam` は要約を発明せずエラーで返す
//!
//! JS decoder が `position` キーを省略する「transform.position 欠落」は、
//! Rust の `Transform2D.position` が必須 field なので**型上表現できない**。
//! そのため Rust 側の `position` は Option ではない(N6 の missing 分岐は空集合)。
//!
//! ここは投影だけで、Document を書かない(single writer 規律)。

use motolii_doc::{DocParam, DocValue, Document, ItemEnvelope, TrackItem};
use motolii_plugin::{F64Domain, PluginCatalog, Value, ValueType};

/// decoder R3 が受ける唯一の revision。
pub const INSPECTOR_READ_MODEL_REVISION: u32 = 1;

/// decoder 出力 D1: `{fixture_revision, target, effect_definitions, position}`。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorReadModel {
    pub fixture_revision: u32,
    pub target: InspectorTarget,
    pub effect_definitions: Vec<InspectorEffectDefinition>,
    pub position: InspectorPosition,
}

/// decoder 出力 D2: `{layer_id, layer_name, item_kind, child_count?}`。
/// `child_count` は group のときだけ載る(P2)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectorTarget {
    pub layer_id: u64,
    pub layer_name: String,
    pub item_kind: InspectorItemKind,
    pub child_count: Option<usize>,
}

/// `TrackItem` の kind タグ(`clip` / `group`)。decoder R6 が閉じる語彙と同じ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorItemKind {
    Clip,
    Group,
}

impl InspectorItemKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Clip => "clip",
            Self::Group => "group",
        }
    }
}

/// decoder 出力の effect_definitions 1件。
/// `params` は plugin 契約(`NodeDesc.params`)の写しで、文書 instance の値ではない。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorEffectDefinition {
    pub definition_id: u64,
    pub plugin_id: String,
    pub params: Vec<InspectorParam>,
}

/// decoder 出力 D5: `{id, value_type, default, f64_domain?}`。
/// 型は既存 `motolii_plugin::ParamDef` の語彙をそのまま使う(新設しない)。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorParam {
    pub id: String,
    pub value_type: ValueType,
    pub default: Value,
    pub f64_domain: Option<F64Domain>,
}

/// decoder N6: Position は閉じた2要約だけ。値の中身(keyframe列)は開かない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InspectorPosition {
    Const { x: f64, y: f64 },
    Animated,
}

/// 投影の失敗。decoder の R系 rule に対応する(該当 rule をコメントで持つ)。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InspectorReadModelError {
    /// R7: target の layer が文書のどこにも居ない。
    #[error("target layer {layer_id} does not exist in the document")]
    TargetLayerMissing { layer_id: u64 },
    /// R7-b: layer 台帳には居るが track item が1つも無い。
    #[error("target layer {layer_id} has no track item")]
    TargetItemMissing { layer_id: u64 },
    /// R7-b: 同じ layer を指す track item が複数ある。
    #[error("target layer {layer_id} resolves to {count} track items")]
    TargetItemAmbiguous { layer_id: u64, count: usize },
    /// R7: EffectUse が存在しない definition を指す。
    #[error("effect use references missing definition {definition_id}")]
    EffectDefinitionMissing { definition_id: u64 },
    /// R9: effect definition の plugin id が catalog で解決できない。
    #[error("plugin `{plugin_id}` is not in the catalog")]
    PluginUnresolved { plugin_id: String },
    /// N6: const(Vec2) / keyframes 以外の position は要約を発明しない。
    #[error("position kind `{kind}` has no closed summary")]
    UnsupportedPosition { kind: &'static str },
}

/// `Document` + plugin catalog + 選択 layer から read-model を作る。
///
/// live 接続(editor selection → ここ)の配線は後続レーン。
/// この関数は fixture の Document でも live の snapshot でも同じに動く。
pub fn project_inspector_read_model(
    document: &Document,
    catalog: &PluginCatalog,
    target_layer: u64,
) -> Result<InspectorReadModel, InspectorReadModelError> {
    // R7: 文書内の全 EffectUse が definition へ届くことを先に見る
    //     (decoder は target 以外の item の dangling も落とす)。
    validate_effect_uses(document)?;

    // R7 / R7-b: target item の解決。
    let mut matches: Vec<&TrackItem> = Vec::new();
    for track in &document.tracks {
        collect_items_for_layer(&track.items, target_layer, &mut matches);
    }
    let layer_known = document
        .layers
        .iter()
        .any(|(id, _)| id.get() == target_layer);
    let item = match matches.as_slice() {
        [] if !layer_known => {
            return Err(InspectorReadModelError::TargetLayerMissing {
                layer_id: target_layer,
            })
        }
        [] => {
            return Err(InspectorReadModelError::TargetItemMissing {
                layer_id: target_layer,
            })
        }
        [item] => *item,
        many => {
            return Err(InspectorReadModelError::TargetItemAmbiguous {
                layer_id: target_layer,
                count: many.len(),
            })
        }
    };

    let (envelope, child_count) = match item {
        TrackItem::Clip(clip) => (&clip.envelope, None),
        TrackItem::Group(group) => (&group.envelope, Some(group.children.len())),
    };
    let layer_name = document
        .layers
        .iter()
        .find(|(id, _)| id.get() == target_layer)
        .map(|(_, name)| name.to_owned())
        .unwrap_or_default();

    // R9: 文書の effect_definitions を catalog(= JS の nodes)の params で結ぶ。
    let mut effect_definitions = Vec::with_capacity(document.effect_definitions.len());
    for definition in &document.effect_definitions {
        let Some(contract) = catalog.get(&definition.plugin_id) else {
            return Err(InspectorReadModelError::PluginUnresolved {
                plugin_id: definition.plugin_id.clone(),
            });
        };
        effect_definitions.push(InspectorEffectDefinition {
            definition_id: definition.id.get(),
            plugin_id: definition.plugin_id.clone(),
            params: contract
                .node
                .params
                .iter()
                .map(|param| InspectorParam {
                    id: param.id.to_owned(),
                    value_type: param.value_type,
                    default: param.default.clone(),
                    f64_domain: param.f64_domain,
                })
                .collect(),
        });
    }

    Ok(InspectorReadModel {
        fixture_revision: INSPECTOR_READ_MODEL_REVISION,
        target: InspectorTarget {
            layer_id: target_layer,
            layer_name,
            item_kind: match item {
                TrackItem::Clip(_) => InspectorItemKind::Clip,
                TrackItem::Group(_) => InspectorItemKind::Group,
            },
            child_count,
        },
        effect_definitions,
        position: project_position(envelope)?,
    })
}

fn collect_items_for_layer<'doc>(
    items: &'doc [TrackItem],
    target_layer: u64,
    out: &mut Vec<&'doc TrackItem>,
) {
    for item in items {
        let envelope = match item {
            TrackItem::Clip(clip) => &clip.envelope,
            TrackItem::Group(group) => &group.envelope,
        };
        if envelope.layer_id.get() == target_layer {
            out.push(item);
        }
        if let TrackItem::Group(group) = item {
            collect_items_for_layer(&group.children, target_layer, out);
        }
    }
}

fn validate_effect_uses(document: &Document) -> Result<(), InspectorReadModelError> {
    fn walk(items: &[TrackItem], document: &Document) -> Result<(), InspectorReadModelError> {
        for item in items {
            let envelope = match item {
                TrackItem::Clip(clip) => &clip.envelope,
                TrackItem::Group(group) => &group.envelope,
            };
            for effect_use in &envelope.effects {
                if !document
                    .effect_definitions
                    .iter()
                    .any(|definition| definition.id == effect_use.definition_id)
                {
                    return Err(InspectorReadModelError::EffectDefinitionMissing {
                        definition_id: effect_use.definition_id.get(),
                    });
                }
            }
            if let TrackItem::Group(group) = item {
                walk(&group.children, document)?;
            }
        }
        Ok(())
    }
    for track in &document.tracks {
        walk(&track.items, document)?;
    }
    Ok(())
}

/// N6: `const {Vec2}` → `Const {x, y}`、keyframes → `Animated`、他は閉じない。
fn project_position(envelope: &ItemEnvelope) -> Result<InspectorPosition, InspectorReadModelError> {
    match &envelope.transform.position {
        DocParam::Const(DocValue::Vec2([x, y])) => Ok(InspectorPosition::Const { x: *x, y: *y }),
        DocParam::Const(value) => Err(InspectorReadModelError::UnsupportedPosition {
            kind: value.kind_name(),
        }),
        DocParam::Keyframes(_) => Ok(InspectorPosition::Animated),
        DocParam::Data { .. } => Err(InspectorReadModelError::UnsupportedPosition { kind: "data" }),
        DocParam::Vec2Axes { .. } => Err(InspectorReadModelError::UnsupportedPosition {
            kind: "vec2_axes",
        }),
        DocParam::LookAt { .. } => {
            Err(InspectorReadModelError::UnsupportedPosition { kind: "look_at" })
        }
        DocParam::Follow { .. } => {
            Err(InspectorReadModelError::UnsupportedPosition { kind: "follow" })
        }
    }
}
