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

use motolii_core::RationalTime;
use motolii_doc::{DocParam, DocValue, Document, ItemEnvelope, TrackItem};
use motolii_plugin::{F64Domain, PluginCatalog, Value, ValueType};

/// decoder R3 が受ける唯一の revision。
pub const INSPECTOR_READ_MODEL_REVISION: u32 = 1;

/// decoder 出力 D1: `{fixture_revision, target, effect_definitions, position}`。
///
/// `editable` は decoder には無い **live 専用**の追加面で、fixture 投影
/// (`project_inspector_read_model`)では常に空である。decoder 由来の
/// 4 field の意味はここでは一切変えていない。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorReadModel {
    pub fixture_revision: u32,
    pub target: InspectorTarget,
    pub effect_definitions: Vec<InspectorEffectDefinition>,
    pub position: InspectorPosition,
    /// 直打ち・◇ が効く行。live 投影(`project_inspector_live_model`)だけが埋める。
    pub editable: Vec<InspectorEditableRow>,
    /// M / S の**押下状態**。`editable` と同じく decoder D1 には無い追加面で、
    /// 出所は target item の `ItemEnvelope`(`visible` / `solo`)である。
    ///
    /// 面がこれを読むから、Inspector は M / S の局所 bool を持たない
    /// (2026-08-18 外部診断 F-03)。Timeline 行の M / S も同じ envelope を読む。
    pub flags: InspectorFlags,
}

/// M / S の押下状態。**正本は Document**。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InspectorFlags {
    /// `envelope.visible == false`。Timeline 行の M と同じ意味。
    pub muted: bool,
    pub solo: bool,
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
    /// ON/OFF の**押下状態**。decoder D1 には無い追加面で、出所は文書の
    /// `EffectDefinition.enabled`(評価がこの旗で effect を飛ばす)。
    /// 面がこれを読むから、Inspector は OFF の集合を局所に持たない(F-03)。
    pub enabled: bool,
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

/// live 編集の対象になる Transform param。**本レーンで閉じたのはこの2つだけ**で、
/// Anchor / Scale / Rotation / Effect param / Vector / Color は residual。
/// 増やすときは `TimelineEditor` 側の適用口(`ParamRef`)と一対で増やすこと。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorEditParam {
    Position,
    Opacity,
}

impl InspectorEditParam {
    /// 行に出す名前(inspector-library.html:29 の `Position` と同じ字)。
    pub fn label(self) -> &'static str {
        match self {
            Self::Position => "Position",
            Self::Opacity => "Opacity",
        }
    }

    /// 成分の軸ラベル。Position は X/Y の2本、Opacity は軸を持たない1本。
    pub fn axes(self) -> &'static [Option<&'static str>] {
        match self {
            Self::Position => &[Some("X"), Some("Y")],
            Self::Opacity => &[None],
        }
    }
}

/// ◇ / ◆ の状態。**キー有無の見せ方はここ1本から決まる**(html:33 の keyCell)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorKeyState {
    /// Const。キーを1つも持たない → ◇
    Unkeyed,
    /// keyframes を持つが playhead には無い → ◇(accent)
    Animated,
    /// playhead にキーがある → ◆
    KeyedAtPlayhead,
}

/// 1行ぶんの live 値。値は **playhead 時刻で評価したもの**で、
/// keyframes の中身(キー列)はここでは開かない。
#[derive(Debug, Clone, PartialEq)]
pub struct InspectorEditableRow {
    pub param: InspectorEditParam,
    /// 成分値。Position は [x, y]、Opacity は [a]。
    pub components: Vec<f64>,
    pub key_state: InspectorKeyState,
    /// 直打ち・◇ を受け付けるか。閉じていない `DocParam` 種
    /// (`Data` / `Vec2Axes` / `LookAt` / `Follow`)は `false` で、値は出すが触らせない。
    pub editable: bool,
}

impl InspectorEditableRow {
    pub fn label(&self) -> &'static str {
        self.param.label()
    }
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
    project(document, catalog, target_layer, None)
}

/// live 投影。decoder と同じ形の上に、playhead 時刻で評価した `editable` を足す。
///
/// **Document は読むだけ**(single writer)。直打ちと ◇ の適用は `TimelineEditor`
/// が抱える writer だけが行い、ここは値と ◇/◆ の状態を出すところまでを持つ。
pub fn project_inspector_live_model(
    document: &Document,
    catalog: &PluginCatalog,
    target_layer: u64,
    playhead: RationalTime,
) -> Result<InspectorReadModel, InspectorReadModelError> {
    project(document, catalog, target_layer, Some(playhead))
}

fn project(
    document: &Document,
    catalog: &PluginCatalog,
    target_layer: u64,
    playhead: Option<RationalTime>,
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
            enabled: definition.enabled,
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
        editable: match playhead {
            Some(at) => project_editable_rows(envelope, at),
            None => Vec::new(),
        },
        // M は Timeline 行と同じ約束(`ItemFlag::Mute => !visible`)。
        flags: InspectorFlags {
            muted: !envelope.visible,
            solo: envelope.solo,
        },
    })
}

/// live の直打ち行を組む。**read-model に無い値を発明しない** — 値は Document の
/// `DocParam` を playhead 時刻で評価したものだけで、キー列は開かない。
fn project_editable_rows(envelope: &ItemEnvelope, at: RationalTime) -> Vec<InspectorEditableRow> {
    [
        (InspectorEditParam::Position, &envelope.transform.position),
        (InspectorEditParam::Opacity, &envelope.opacity),
    ]
    .into_iter()
    .map(|(param, doc_param)| project_editable_row(param, doc_param, at))
    .collect()
}

fn project_editable_row(
    param: InspectorEditParam,
    doc_param: &DocParam,
    at: RationalTime,
) -> InspectorEditableRow {
    let arity = param.axes().len();
    match doc_param {
        DocParam::Const(value) => InspectorEditableRow {
            param,
            components: components_of(value, arity),
            key_state: InspectorKeyState::Unkeyed,
            editable: true,
        },
        DocParam::Keyframes(track) => {
            let keyed_here = track.keys().iter().any(|key| key.t == at);
            // 値は playhead の評価値。キーの上ならそのキーの値そのものになる。
            InspectorEditableRow {
                param,
                components: eval_components(&track.eval(at), arity),
                key_state: if keyed_here {
                    InspectorKeyState::KeyedAtPlayhead
                } else {
                    InspectorKeyState::Animated
                },
                // 空の keyframes track は prepare_* が断るので触らせない。
                editable: !track.keys().is_empty(),
            }
        }
        // 閉じていない種(Data / Vec2Axes / LookAt / Follow)。値も要約も発明しない。
        _ => InspectorEditableRow {
            param,
            components: vec![0.0; arity],
            key_state: InspectorKeyState::Unkeyed,
            editable: false,
        },
    }
}

/// `DocValue` を成分の並びへ。arity に足りない分は出さない(0 で埋めない)。
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

/// 評価層の値(D3)を成分の並びへ。`DocValue` 側と同じ規則で切り出す。
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
