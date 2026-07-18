//! D1e / D1l journal v1 adapter 共用の inline Effect → Definition+Use planner 核。
//!
//! 固定採番式・Track→Item→Group pre-order→effect 走査・移行計画はここが正本。

use std::collections::{BTreeMap, BTreeSet};

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use thiserror::Error;

use crate::command::Command;
use crate::migrate::MigrateError;
use crate::param::DocParam;
use crate::schema::{
    BlendMode, Clip, ClipSource, ClippingMaskSettings, EffectDefinition, EffectInstance, EffectUse,
    Group, ItemEnvelope, TrackItem, Transform2D,
};
use crate::stable_id::{EffectDefinitionId, EffectId};
use crate::validate::stable_id_in_use;
use crate::{Document, LayerId};
use motolii_core::{RationalTime, TimeMap};

/// D1e migrate と journal v1 adapter が参照する planner 核(API gate 用型名)。
pub(crate) struct LegacyEffectMigrationPlanner;

impl LegacyEffectMigrationPlanner {
    pub(crate) fn compute_counter_watermark(
        counter_before: u64,
        observed_ids: &[u64],
        new_definition_count: usize,
    ) -> Result<(u64, u64), LegacyEffectMigrationError> {
        compute_counter_watermark(counter_before, observed_ids, new_definition_count)
    }
}

fn default_true() -> bool {
    true
}

fn default_effect_version() -> u32 {
    1
}

/// v1 WAL の旧 inline Effect(payload に `definition_id` なし)。
#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LegacyInlineEffect {
    pub id: EffectId,
    pub plugin_id: String,
    #[serde(default = "default_effect_version")]
    pub effect_version: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub params: BTreeMap<String, DocParam>,
    #[serde(default, flatten)]
    pub extra: Map<String, serde_json::Value>,
}

impl<'de> Deserialize<'de> for LegacyInlineEffect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            id: EffectId,
            #[serde(default)]
            definition_id: Option<EffectDefinitionId>,
            plugin_id: String,
            #[serde(default = "default_effect_version")]
            effect_version: u32,
            #[serde(default = "default_true")]
            enabled: bool,
            #[serde(default)]
            params: BTreeMap<String, DocParam>,
            #[serde(default, flatten)]
            extra: Map<String, serde_json::Value>,
        }
        let raw = Raw::deserialize(deserializer)?;
        if raw.definition_id.is_some() {
            return Err(de::Error::custom(
                "v1 inline effect must not carry definition_id",
            ));
        }
        Ok(Self {
            id: raw.id,
            plugin_id: raw.plugin_id,
            effect_version: raw.effect_version,
            enabled: raw.enabled,
            params: raw.params,
            extra: raw.extra,
        })
    }
}

impl LegacyInlineEffect {
    pub(crate) fn to_definition(&self, id: EffectDefinitionId) -> EffectDefinition {
        EffectDefinition::new(
            id,
            &self.plugin_id,
            self.effect_version,
            self.enabled,
            self.params.clone(),
            self.extra.clone(),
        )
    }

    pub(crate) fn to_use(&self, definition_id: EffectDefinitionId) -> EffectUse {
        EffectUse {
            id: self.id,
            definition_id,
        }
    }

    pub(crate) fn to_instance(&self, definition_id: EffectDefinitionId) -> EffectInstance {
        EffectInstance {
            id: self.id,
            definition_id,
            plugin_id: self.plugin_id.clone(),
            effect_version: self.effect_version,
            enabled: self.enabled,
            params: self.params.clone(),
            extra: self.extra.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LegacyItemEnvelope {
    pub layer_id: LayerId,
    #[serde(default)]
    pub effects: Vec<LegacyInlineEffect>,
    pub transform: Transform2D,
    pub opacity: DocParam,
    #[serde(default)]
    pub blend: BlendMode,
    #[serde(default)]
    pub clipping_mask: ClippingMaskSettings,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub solo: bool,
    #[serde(default)]
    pub lock: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LegacyTrackItem {
    Clip {
        envelope: Box<LegacyItemEnvelope>,
        start: RationalTime,
        duration: RationalTime,
        #[serde(default)]
        time_map: TimeMap,
        source: Box<ClipSource>,
    },
    Group {
        envelope: Box<LegacyItemEnvelope>,
        #[serde(default)]
        children: Vec<LegacyTrackItem>,
    },
}

/// planner が適用前に具体化する完全 payload + counter watermark。
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedLegacyEdit {
    pub expected_counter_before: u64,
    pub counter_after: u64,
    pub complete_payload: Command,
    pub introduced_definitions: Vec<EffectDefinition>,
    pub destroyed_definition_ids: Vec<EffectDefinitionId>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub(crate) enum LegacyEffectMigrationError {
    #[error("stable id sequence exhausted")]
    StableIdExhausted,
    #[error("stable id {id} already exists in document")]
    DocumentIdCollision { id: u64 },
    #[error("duplicate stable id {id} in legacy payload")]
    PayloadIdCollision { id: u64 },
    #[error(
        "stable id reservation counter mismatch (next={next}, expected_before={expected_before})"
    )]
    CounterMismatch { next: u64, expected_before: u64 },
    #[error("effect definition {id} is shared by other use(s) {use_ids:?}")]
    DefinitionShared { id: u64, use_ids: Vec<u64> },
    #[error("legacy remove effect does not match document at index {index}")]
    RemoveEffectMismatch { index: usize },
    #[error("legacy remove track item does not match document at index {index}")]
    RemoveTrackItemMismatch { index: usize },
    #[error(transparent)]
    Command(#[from] crate::command::CommandError),
    #[error(transparent)]
    Validate(#[from] crate::DocumentError),
}

/// canonical plan tree: LegacyInlineEffect と migrated EffectUse を区別する。
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LegacyPlanEffect {
    Inline(LegacyInlineEffect),
    Use(EffectUse),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LegacyPlanEnvelope {
    pub layer_id: LayerId,
    pub effects: Vec<LegacyPlanEffect>,
    pub transform: Transform2D,
    pub opacity: DocParam,
    pub blend: BlendMode,
    pub clipping_mask: ClippingMaskSettings,
    pub visible: bool,
    pub solo: bool,
    pub lock: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LegacyPlanTrackItem {
    Clip {
        envelope: Box<LegacyPlanEnvelope>,
        start: RationalTime,
        duration: RationalTime,
        time_map: TimeMap,
        source: Box<ClipSource>,
    },
    Group {
        envelope: Box<LegacyPlanEnvelope>,
        children: Vec<LegacyPlanTrackItem>,
    },
}

pub(crate) fn legacy_track_item_to_plan(item: &LegacyTrackItem) -> LegacyPlanTrackItem {
    match item {
        LegacyTrackItem::Clip {
            envelope,
            start,
            duration,
            time_map,
            source,
        } => LegacyPlanTrackItem::Clip {
            envelope: Box::new(legacy_envelope_to_plan(envelope)),
            start: *start,
            duration: *duration,
            time_map: *time_map,
            source: source.clone(),
        },
        LegacyTrackItem::Group { envelope, children } => LegacyPlanTrackItem::Group {
            envelope: Box::new(legacy_envelope_to_plan(envelope)),
            children: children.iter().map(legacy_track_item_to_plan).collect(),
        },
    }
}

fn legacy_envelope_to_plan(envelope: &LegacyItemEnvelope) -> LegacyPlanEnvelope {
    LegacyPlanEnvelope {
        layer_id: envelope.layer_id,
        effects: envelope
            .effects
            .iter()
            .map(|e| LegacyPlanEffect::Inline(e.clone()))
            .collect(),
        transform: envelope.transform.clone(),
        opacity: envelope.opacity.clone(),
        blend: envelope.blend,
        clipping_mask: envelope.clipping_mask.clone(),
        visible: envelope.visible,
        solo: envelope.solo,
        lock: envelope.lock,
    }
}

fn plan_envelope_to_item_envelope(envelope: &LegacyPlanEnvelope) -> ItemEnvelope {
    ItemEnvelope {
        layer_id: envelope.layer_id,
        effects: Vec::new(),
        transform: envelope.transform.clone(),
        opacity: envelope.opacity.clone(),
        blend: envelope.blend,
        clipping_mask: envelope.clipping_mask.clone(),
        visible: envelope.visible,
        solo: envelope.solo,
        lock: envelope.lock,
    }
}

fn decode_plan_effect_from_json(
    value: &Value,
    path: &str,
) -> Result<LegacyPlanEffect, MigrateError> {
    let Value::Object(map) = value else {
        return Err(MigrateError::NotAnObject);
    };
    if map.contains_key("definition_id") {
        if map.contains_key("plugin_id")
            || map.contains_key("params")
            || map.contains_key("effect_version")
            || map.contains_key("enabled")
        {
            return Err(MigrateError::HybridEffectEntry { path: path.into() });
        }
        let effect_use: EffectUse = serde_json::from_value(value.clone())?;
        return Ok(LegacyPlanEffect::Use(effect_use));
    }
    if map.contains_key("plugin_id") {
        let inline: LegacyInlineEffect = serde_json::from_value(value.clone())?;
        return Ok(LegacyPlanEffect::Inline(inline));
    }
    Err(MigrateError::NotAnObject)
}

fn decode_plan_envelope_from_json(
    env: &Value,
    path: &str,
) -> Result<LegacyPlanEnvelope, MigrateError> {
    let mut sans_effects = env.clone();
    if let Some(obj) = sans_effects.as_object_mut() {
        obj.remove("effects");
    }
    let base: LegacyItemEnvelope = serde_json::from_value(sans_effects)?;
    let mut effects = Vec::new();
    if let Some(arr) = env.get("effects").and_then(|e| e.as_array()) {
        for (ei, eff) in arr.iter().enumerate() {
            effects.push(decode_plan_effect_from_json(
                eff,
                &format!("{path}.effects[{ei}]"),
            )?);
        }
    }
    Ok(LegacyPlanEnvelope {
        layer_id: base.layer_id,
        effects,
        transform: base.transform,
        opacity: base.opacity,
        blend: base.blend,
        clipping_mask: base.clipping_mask,
        visible: base.visible,
        solo: base.solo,
        lock: base.lock,
    })
}

fn decode_plan_item_from_json(
    item: &Value,
    path: &str,
) -> Result<LegacyPlanTrackItem, MigrateError> {
    let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("");
    match kind {
        "clip" => {
            let env = item
                .get("envelope")
                .ok_or_else(|| MigrateError::StableId(format!("missing envelope at {path}")))?;
            let envelope = decode_plan_envelope_from_json(env, &format!("{path}.envelope"))?;
            #[derive(Deserialize)]
            struct ClipBody {
                start: RationalTime,
                duration: RationalTime,
                #[serde(default)]
                time_map: TimeMap,
                source: Box<ClipSource>,
            }
            let body: ClipBody = serde_json::from_value(item.clone())?;
            Ok(LegacyPlanTrackItem::Clip {
                envelope: Box::new(envelope),
                start: body.start,
                duration: body.duration,
                time_map: body.time_map,
                source: body.source,
            })
        }
        "group" => {
            let env = item
                .get("envelope")
                .ok_or_else(|| MigrateError::StableId(format!("missing envelope at {path}")))?;
            let envelope = decode_plan_envelope_from_json(env, &format!("{path}.envelope"))?;
            let children_json = item
                .get("children")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default();
            let mut children = Vec::with_capacity(children_json.len());
            for (ci, child) in children_json.iter().enumerate() {
                children.push(decode_plan_item_from_json(
                    child,
                    &format!("{path}.children[{ci}]"),
                )?);
            }
            Ok(LegacyPlanTrackItem::Group {
                envelope: Box::new(envelope),
                children,
            })
        }
        other => Err(MigrateError::StableId(format!(
            "unsupported track item kind `{other}` at {path}"
        ))),
    }
}

struct PlannedInlineAllocation<'a> {
    inline: &'a LegacyInlineEffect,
    definition_id: EffectDefinitionId,
}

/// watermark(max_observed) と introduced(一意性/衝突) を分離して effect 1 件分を記録する。
fn note_plan_effect_ids(
    eff: &LegacyPlanEffect,
    watermark: &mut Vec<u64>,
    introduced: &mut Vec<u64>,
) {
    match eff {
        LegacyPlanEffect::Inline(inline) => {
            let ids = collect_legacy_effect_ids(inline);
            watermark.extend(&ids);
            introduced.extend(ids);
        }
        LegacyPlanEffect::Use(u) => {
            watermark.push(u.id.get());
            watermark.push(u.definition_id.get());
            introduced.push(u.id.get());
        }
    }
}

enum PlanWalkPhase<'p, 'a> {
    Collect {
        inline_effects: &'p mut Vec<&'a LegacyInlineEffect>,
        watermark_ids: &'p mut Vec<u64>,
        introduced_ids: &'p mut Vec<u64>,
    },
    Materialize {
        alloc_iter: &'p mut std::slice::Iter<'a, PlannedInlineAllocation<'a>>,
        definitions: &'p mut Vec<EffectDefinition>,
    },
}

/// canonical plan tree の唯一の再帰 match: Clip/Group/children + envelope effect stack。
fn walk_plan_item<'a>(
    item: &'a LegacyPlanTrackItem,
    phase: &mut PlanWalkPhase<'_, 'a>,
) -> Option<TrackItem> {
    match item {
        LegacyPlanTrackItem::Clip {
            envelope,
            start,
            duration,
            time_map,
            source,
        } => match phase {
            PlanWalkPhase::Collect {
                inline_effects,
                watermark_ids,
                introduced_ids,
            } => {
                for eff in &envelope.effects {
                    note_plan_effect_ids(eff, watermark_ids, introduced_ids);
                    if let LegacyPlanEffect::Inline(inline) = eff {
                        inline_effects.push(inline);
                    }
                }
                None
            }
            PlanWalkPhase::Materialize {
                alloc_iter,
                definitions,
            } => {
                let (effects, _migrated) = materialize_envelope_effects_in_walk(
                    &envelope.effects,
                    alloc_iter,
                    definitions,
                );
                let mut env = plan_envelope_to_item_envelope(envelope);
                env.effects = effects;
                Some(TrackItem::Clip(Clip {
                    envelope: env,
                    start: *start,
                    duration: *duration,
                    time_map: *time_map,
                    source: source.as_ref().clone(),
                }))
            }
        },
        LegacyPlanTrackItem::Group { envelope, children } => match phase {
            PlanWalkPhase::Collect {
                inline_effects,
                watermark_ids,
                introduced_ids,
            } => {
                for eff in &envelope.effects {
                    note_plan_effect_ids(eff, watermark_ids, introduced_ids);
                    if let LegacyPlanEffect::Inline(inline) = eff {
                        inline_effects.push(inline);
                    }
                }
                for child in children {
                    let _ = walk_plan_item(child, phase);
                }
                None
            }
            PlanWalkPhase::Materialize {
                alloc_iter,
                definitions,
            } => {
                let (effects, _migrated) = materialize_envelope_effects_in_walk(
                    &envelope.effects,
                    alloc_iter,
                    definitions,
                );
                let mut migrated_children = Vec::with_capacity(children.len());
                for child in children {
                    migrated_children.push(walk_plan_item(child, phase).expect("materialize walk"));
                }
                let mut env = plan_envelope_to_item_envelope(envelope);
                env.effects = effects;
                Some(TrackItem::Group(Group {
                    envelope: env,
                    children: migrated_children,
                }))
            }
        },
    }
}

/// effect stack 順の非再帰ループ(materialize 第2相専用)。
fn materialize_envelope_effects_in_walk<'a>(
    effects: &[LegacyPlanEffect],
    alloc_iter: &mut std::slice::Iter<'a, PlannedInlineAllocation<'a>>,
    definitions: &mut Vec<EffectDefinition>,
) -> (Vec<EffectUse>, bool) {
    let mut out = Vec::with_capacity(effects.len());
    let mut migrated = false;
    for eff in effects {
        match eff {
            LegacyPlanEffect::Use(u) => out.push(u.clone()),
            LegacyPlanEffect::Inline(_) => {
                let alloc = alloc_iter
                    .next()
                    .expect("planner allocations must match inline effect count");
                definitions.push(alloc.inline.to_definition(alloc.definition_id));
                out.push(alloc.inline.to_use(alloc.definition_id));
                migrated = true;
            }
        }
    }
    (out, migrated)
}

fn walk_plan_items_collect(
    items: &[LegacyPlanTrackItem],
) -> (Vec<&LegacyInlineEffect>, Vec<u64>, Vec<u64>) {
    let mut inline_effects = Vec::new();
    let mut watermark_ids = Vec::new();
    let mut introduced_ids = Vec::new();
    {
        let mut phase = PlanWalkPhase::Collect {
            inline_effects: &mut inline_effects,
            watermark_ids: &mut watermark_ids,
            introduced_ids: &mut introduced_ids,
        };
        for item in items {
            let _ = walk_plan_item(item, &mut phase);
        }
    }
    (inline_effects, watermark_ids, introduced_ids)
}

fn walk_plan_items_materialize<'a>(
    items: &[LegacyPlanTrackItem],
    allocs: &'a [PlannedInlineAllocation<'a>],
    definitions: &mut Vec<EffectDefinition>,
) -> (Vec<TrackItem>, bool) {
    let mut alloc_iter = allocs.iter();
    let mut materialized = Vec::with_capacity(items.len());
    {
        let mut phase = PlanWalkPhase::Materialize {
            alloc_iter: &mut alloc_iter,
            definitions,
        };
        for item in items {
            materialized.push(walk_plan_item(item, &mut phase).expect("materialize walk"));
        }
    }
    let migrated = !allocs.is_empty();
    (materialized, migrated)
}

/// planner 文脈: registry/document Definition ID は watermark のみ、payload 導入 ID は introduced。
pub(crate) struct LegacyPlanMaterializeContext<'a> {
    pub counter_before: u64,
    pub registry_definition_ids: &'a [u64],
    pub document: Option<&'a Document>,
}

fn collect_root_definition_ids(root: &Value) -> Vec<u64> {
    root.get("effect_definitions")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("id").and_then(|id| id.as_u64()))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn document_definition_ids(doc: &Document) -> Vec<u64> {
    doc.effect_definitions.iter().map(|d| d.id.get()).collect()
}

fn plan_inline_effect_allocations_from_collect<'a>(
    ctx: &LegacyPlanMaterializeContext<'_>,
    collected: (Vec<&'a LegacyInlineEffect>, Vec<u64>, Vec<u64>),
) -> Result<(Vec<PlannedInlineAllocation<'a>>, u64, u64), LegacyEffectMigrationError> {
    let (inline_effects, mut watermark_ids, introduced_ids) = collected;
    check_payload_id_uniqueness(&introduced_ids)?;
    check_definition_registry_uniqueness(ctx.registry_definition_ids)?;
    if let Some(doc) = ctx.document {
        check_document_collisions(doc, &introduced_ids)?;
    }
    watermark_ids.extend_from_slice(ctx.registry_definition_ids);
    let n = inline_effects.len();
    let (plan_start, counter_after) = LegacyEffectMigrationPlanner::compute_counter_watermark(
        ctx.counter_before,
        &watermark_ids,
        n,
    )?;
    if n > 0 {
        let new_def_ids: Vec<u64> = (0..n as u64)
            .map(|i| {
                plan_start
                    .checked_add(i)
                    .ok_or(LegacyEffectMigrationError::StableIdExhausted)
            })
            .collect::<Result<_, _>>()?;
        if let Some(doc) = ctx.document {
            check_document_collisions(doc, &new_def_ids)?;
        }
    }
    let mut allocations = Vec::with_capacity(n);
    let mut next_def = plan_start;
    for inline in inline_effects {
        allocations.push(PlannedInlineAllocation {
            inline,
            definition_id: EffectDefinitionId::from_raw(next_def),
        });
        next_def = next_def
            .checked_add(1)
            .ok_or(LegacyEffectMigrationError::StableIdExhausted)?;
    }
    Ok((allocations, plan_start, counter_after))
}

/// 台帳内 Definition identity の不正重複のみ拒否(参照 ID の反復は watermark のみ)。
pub(crate) fn check_definition_registry_uniqueness(
    ids: &[u64],
) -> Result<(), LegacyEffectMigrationError> {
    check_payload_id_uniqueness(ids)
}

pub(crate) struct InlineMigrationBundle {
    pub materialized_items: Vec<TrackItem>,
    pub introduced_definitions: Vec<EffectDefinition>,
    pub counter_after: u64,
    pub migrated: bool,
}

/// D1e / journal 共用: 文書(または payload)全体を一括 planner で materialize する。
pub(crate) fn plan_and_materialize_legacy_items(
    ctx: &LegacyPlanMaterializeContext<'_>,
    items: &[LegacyPlanTrackItem],
) -> Result<InlineMigrationBundle, LegacyEffectMigrationError> {
    let collected = walk_plan_items_collect(items);
    let (allocations, _plan_start, counter_after) =
        plan_inline_effect_allocations_from_collect(ctx, collected)?;
    let mut definitions = Vec::with_capacity(allocations.len());
    let (materialized, migrated) =
        walk_plan_items_materialize(items, &allocations, &mut definitions);
    Ok(InlineMigrationBundle {
        materialized_items: materialized,
        introduced_definitions: definitions,
        counter_after,
        migrated,
    })
}

fn legacy_err_to_migrate(err: LegacyEffectMigrationError) -> MigrateError {
    match err {
        LegacyEffectMigrationError::StableIdExhausted => {
            MigrateError::StableId("stable id sequence exhausted".into())
        }
        LegacyEffectMigrationError::PayloadIdCollision { id } => {
            MigrateError::StableId(format!("duplicate stable id {id} in legacy payload"))
        }
        other => MigrateError::StableId(other.to_string()),
    }
}

/// D1e JSON 上の inline Effect を Definition+Use へ分離する。
pub(crate) fn migrate_inline_effects_json(root: &mut Value) -> Result<bool, MigrateError> {
    let counter_before = root
        .get("next_stable_id")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let tracks_snapshot = root
        .get("tracks")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();

    struct ItemSlot {
        track_idx: usize,
        item_idx: usize,
        plan: LegacyPlanTrackItem,
    }
    let mut slots: Vec<ItemSlot> = Vec::new();
    for (ti, track) in tracks_snapshot.iter().enumerate() {
        let Some(items) = track.get("items").and_then(|i| i.as_array()) else {
            continue;
        };
        for (ii, item) in items.iter().enumerate() {
            let path = format!("tracks[{ti}].items[{ii}]");
            let plan = decode_plan_item_from_json(item, &path)?;
            slots.push(ItemSlot {
                track_idx: ti,
                item_idx: ii,
                plan,
            });
        }
    }

    if slots.is_empty() {
        return Ok(false);
    }

    let plans: Vec<LegacyPlanTrackItem> = slots.iter().map(|s| s.plan.clone()).collect();
    let registry_ids = collect_root_definition_ids(root);
    let ctx = LegacyPlanMaterializeContext {
        counter_before,
        registry_definition_ids: &registry_ids,
        document: None,
    };
    let bundle = plan_and_materialize_legacy_items(&ctx, &plans).map_err(legacy_err_to_migrate)?;

    if !bundle.migrated {
        return Ok(false);
    }

    let Value::Object(map) = root else {
        return Err(MigrateError::NotAnObject);
    };
    let tracks_mut = map
        .get_mut("tracks")
        .and_then(|t| t.as_array_mut())
        .ok_or(MigrateError::NotAnObject)?;

    for (slot, materialized) in slots.iter().zip(bundle.materialized_items.iter()) {
        let item_json = serde_json::to_value(materialized)
            .map_err(|e| MigrateError::StableId(e.to_string()))?;
        tracks_mut[slot.track_idx]["items"][slot.item_idx] = item_json;
    }

    let def_json: Vec<Value> = bundle
        .introduced_definitions
        .iter()
        .map(|d| serde_json::to_value(d).map_err(|e| MigrateError::StableId(e.to_string())))
        .collect::<Result<_, _>>()?;
    if !def_json.is_empty() {
        match map
            .get_mut("effect_definitions")
            .and_then(|d| d.as_array_mut())
        {
            Some(arr) => arr.extend(def_json),
            None => {
                map.insert("effect_definitions".into(), Value::Array(def_json));
            }
        }
    }
    map.insert("next_stable_id".into(), json!(bundle.counter_after));
    Ok(true)
}

pub(crate) fn collect_legacy_effect_ids(effect: &LegacyInlineEffect) -> Vec<u64> {
    let mut ids = vec![effect.id.get()];
    for param in effect.params.values() {
        collect_doc_param_ids(param, &mut ids);
    }
    ids
}

pub(crate) fn collect_doc_param_ids(param: &DocParam, out: &mut Vec<u64>) {
    match param {
        DocParam::Keyframes(k) => {
            for kf in k.keys() {
                out.push(kf.id.get());
            }
        }
        DocParam::Vec2Axes { x, y } => {
            collect_doc_param_ids(x, out);
            collect_doc_param_ids(y, out);
        }
        _ => {}
    }
}

pub(crate) fn legacy_effect_matches_definition(
    inline: &LegacyInlineEffect,
    def: &EffectDefinition,
) -> bool {
    inline.plugin_id == def.plugin_id
        && inline.effect_version == def.effect_version
        && inline.enabled == def.enabled
        && inline.params == def.params
        && inline.extra == def.extra
}

/// planner 固定式: `plan_start = max(counter, max_observed+1)`、`counter_after = plan_start + n`(n>=1) / counter(n==0)。
pub(crate) fn compute_counter_watermark(
    counter_before: u64,
    observed_ids: &[u64],
    new_definition_count: usize,
) -> Result<(u64, u64), LegacyEffectMigrationError> {
    let max_observed = observed_ids.iter().copied().max();
    let plan_start = match max_observed {
        Some(max_id) => counter_before.max(
            max_id
                .checked_add(1)
                .ok_or(LegacyEffectMigrationError::StableIdExhausted)?,
        ),
        None => counter_before,
    };
    let counter_after = if new_definition_count == 0 {
        counter_before
    } else {
        plan_start
            .checked_add(new_definition_count as u64)
            .ok_or(LegacyEffectMigrationError::StableIdExhausted)?
    };
    Ok((plan_start, counter_after))
}

pub(crate) fn check_payload_id_uniqueness(ids: &[u64]) -> Result<(), LegacyEffectMigrationError> {
    let mut seen = BTreeSet::new();
    for &id in ids {
        if !seen.insert(id) {
            return Err(LegacyEffectMigrationError::PayloadIdCollision { id });
        }
    }
    Ok(())
}

pub(crate) fn check_document_collisions(
    doc: &Document,
    new_ids: &[u64],
) -> Result<(), LegacyEffectMigrationError> {
    for &id in new_ids {
        if stable_id_in_use(doc, id) {
            return Err(LegacyEffectMigrationError::DocumentIdCollision { id });
        }
    }
    Ok(())
}

pub(crate) fn apply_prepared_legacy_edit(
    doc: &mut Document,
    prepared: &PreparedLegacyEdit,
) -> Result<(), LegacyEffectMigrationError> {
    let next = doc.next_stable_id.peek_next();
    if next != prepared.expected_counter_before {
        return Err(LegacyEffectMigrationError::CounterMismatch {
            next,
            expected_before: prepared.expected_counter_before,
        });
    }
    for def in &prepared.introduced_definitions {
        if stable_id_in_use(doc, def.id.get()) {
            return Err(LegacyEffectMigrationError::DocumentIdCollision { id: def.id.get() });
        }
    }

    let mut working = doc.clone();
    for def in &prepared.introduced_definitions {
        working.effect_definitions.push(def.clone());
    }
    prepared
        .complete_payload
        .apply(&mut working)
        .map_err(LegacyEffectMigrationError::Command)?;
    for def_id in &prepared.destroyed_definition_ids {
        let idx = working
            .effect_definitions
            .iter()
            .position(|d| d.id == *def_id)
            .ok_or(LegacyEffectMigrationError::Command(
                crate::command::CommandError::EffectDefinitionNotFound { id: def_id.get() },
            ))?;
        working.effect_definitions.remove(idx);
    }
    working
        .next_stable_id
        .commit_validated_reservation(prepared.counter_after);
    working
        .validate()
        .map_err(LegacyEffectMigrationError::Validate)?;
    *doc = working;
    Ok(())
}

#[cfg(test)]
mod apply_tests {
    use super::*;
    use crate::command::Command;
    use crate::schema::EffectDefinition;
    use crate::stable_id::EffectDefinitionId;
    use crate::{Document, LayerId};

    #[test]
    fn prepared_counter_mismatch_rejects_without_mutation() {
        let mut doc = Document::new_current();
        let before = doc.clone();
        let prepared = PreparedLegacyEdit {
            expected_counter_before: 42,
            counter_after: 43,
            complete_payload: Command::SetProperty {
                target: LayerId::from_raw(0),
                property: crate::command::ScalarPropertyId::Opacity,
                old_value: crate::param::DocParam::const_f64(1.0),
                new_value: crate::param::DocParam::const_f64(0.5),
            },
            introduced_definitions: vec![EffectDefinition::new(
                EffectDefinitionId::from_raw(99),
                "p",
                1,
                true,
                Default::default(),
                Default::default(),
            )],
            destroyed_definition_ids: Vec::new(),
        };
        let err = apply_prepared_legacy_edit(&mut doc, &prepared).unwrap_err();
        assert!(matches!(
            err,
            LegacyEffectMigrationError::CounterMismatch { .. }
        ));
        assert_eq!(doc, before);
    }

    #[test]
    fn prepared_destroy_mismatch_leaves_original_document_unchanged() {
        use crate::schema::{Clip, ClipSource, ItemEnvelope, Track, TrackItem};
        use motolii_core::RationalTime;

        let mut doc = Document::new_current();
        let layer = doc.layers.allocate("a").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        doc.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            })],
        });
        doc.validate().unwrap();

        let before = doc.clone();
        let counter = doc.next_stable_id.peek_next();
        let prepared = PreparedLegacyEdit {
            expected_counter_before: counter,
            counter_after: counter,
            complete_payload: Command::SetProperty {
                target: layer,
                property: crate::command::ScalarPropertyId::Opacity,
                old_value: crate::param::DocParam::const_f64(1.0),
                new_value: crate::param::DocParam::const_f64(0.5),
            },
            introduced_definitions: Vec::new(),
            destroyed_definition_ids: vec![EffectDefinitionId::from_raw(999)],
        };
        let err = apply_prepared_legacy_edit(&mut doc, &prepared).unwrap_err();
        assert!(matches!(
            err,
            LegacyEffectMigrationError::Command(
                crate::command::CommandError::EffectDefinitionNotFound { .. }
            )
        ));
        assert_eq!(doc, before);
    }
}

#[cfg(test)]
mod planner_parity_tests {
    use super::*;
    use crate::schema::{Track, TrackItem};
    use motolii_core::RationalTime;
    use serde_json::json;

    #[derive(Debug, PartialEq)]
    pub struct PlannerSemanticSnapshot {
        pub definition_ids: Vec<u64>,
        pub use_pairs: Vec<(u64, u64)>,
        pub keyframe_ids: Vec<u64>,
        pub keyframe_details: Vec<(u64, RationalTime, String, String)>,
        pub counter_after: u64,
        pub extras: Vec<Value>,
    }

    pub fn semantics_from_document(doc: &Document, counter_after: u64) -> PlannerSemanticSnapshot {
        let mut keyframe_ids = Vec::new();
        let mut keyframe_details = Vec::new();
        for def in &doc.effect_definitions {
            for param in def.params.values() {
                collect_keyframe_semantics(param, &mut keyframe_ids, &mut keyframe_details);
            }
        }
        PlannerSemanticSnapshot {
            definition_ids: doc.effect_definitions.iter().map(|d| d.id.get()).collect(),
            use_pairs: collect_all_use_pairs(doc),
            keyframe_ids,
            keyframe_details,
            counter_after,
            extras: doc
                .effect_definitions
                .iter()
                .map(|d| Value::Object(d.extra.clone()))
                .collect(),
        }
    }

    fn collect_keyframe_semantics(
        param: &DocParam,
        ids: &mut Vec<u64>,
        details: &mut Vec<(u64, RationalTime, String, String)>,
    ) {
        match param {
            DocParam::Keyframes(k) => {
                for kf in k.keys() {
                    ids.push(kf.id.get());
                    details.push((
                        kf.id.get(),
                        kf.t,
                        serde_json::to_string(&kf.value).unwrap(),
                        format!("{:?}", kf.interp),
                    ));
                }
            }
            DocParam::Vec2Axes { x, y } => {
                collect_keyframe_semantics(x, ids, details);
                collect_keyframe_semantics(y, ids, details);
            }
            _ => {}
        }
    }

    fn collect_all_use_pairs(doc: &Document) -> Vec<(u64, u64)> {
        let mut out = Vec::new();
        for track in &doc.tracks {
            for item in &track.items {
                collect_item_use_pairs(item, &mut out);
            }
        }
        out
    }

    fn collect_item_use_pairs(item: &TrackItem, out: &mut Vec<(u64, u64)>) {
        match item {
            TrackItem::Clip(c) => {
                for u in &c.envelope.effects {
                    out.push((u.id.get(), u.definition_id.get()));
                }
            }
            TrackItem::Group(g) => {
                for u in &g.envelope.effects {
                    out.push((u.id.get(), u.definition_id.get()));
                }
                for child in &g.children {
                    collect_item_use_pairs(child, out);
                }
            }
        }
    }

    fn complex_legacy_inline_document_json() -> Value {
        json!({
            "version": 1,
            "min_reader_version": 1,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1}
            },
            "bpm": {"num": 120, "den": 1},
            "layers": {
                "next": 3,
                "entries": [
                    {"id": 0, "name": "a"},
                    {"id": 1, "name": "grp"},
                    {"id": 2, "name": "nested"}
                ]
            },
            "track_ids": {
                "next": 2,
                "entries": [
                    {"id": 0, "name": "V1"},
                    {"id": 1, "name": "V2"}
                ]
            },
            "effect_definitions": [{
                "id": 52,
                "plugin_id": "p.modern.existing",
                "effect_version": 1,
                "enabled": true,
                "params": {},
                "registry_tag": "keep-registry"
            }],
            "tracks": [
                {
                    "id": 0,
                    "items": [{
                        "kind": "group",
                        "envelope": {
                            "layer_id": 1,
                            "effects": [
                                {
                                    "id": 55,
                                    "plugin_id": "p.group",
                                    "params": {
                                        "amount": {
                                            "keyframes": {
                                                "keys": [{"id": 54, "t": {"num": 0, "den": 1}, "value": {"F64": 0.5}, "interp": "Linear"}]
                                            }
                                        }
                                    },
                                    "vendor_grp": "keep-group"
                                },
                                {"id": 58, "definition_id": 52}
                            ],
                            "transform": {
                                "position": {"const": {"Vec2": [0.0, 0.0]}},
                                "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                                "scale": {"const": {"Vec2": [1.0, 1.0]}},
                                "rotation": {"const": {"F64": 0.0}}
                            },
                            "opacity": {"const": {"F64": 1.0}}
                        },
                        "children": [{
                            "kind": "clip",
                            "envelope": {
                                "layer_id": 2,
                                "effects": [
                                    {"id": 56, "plugin_id": "p.nested.a", "vendor_a": "x"},
                                    {"id": 57, "plugin_id": "p.nested.b", "enabled": false, "vendor_b": "y"}
                                ],
                                "transform": {
                                    "position": {"const": {"Vec2": [0.0, 0.0]}},
                                    "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                                    "scale": {"const": {"Vec2": [1.0, 1.0]}},
                                    "rotation": {"const": {"F64": 0.0}}
                                },
                                "opacity": {"const": {"F64": 1.0}}
                            },
                            "start": {"num": 0, "den": 1},
                            "duration": {"num": 5, "den": 1},
                            "time_map": {
                                "source_start": {"num": 0, "den": 1},
                                "speed_num": 1,
                                "speed_den": 1,
                                "overrun_mode": "freeze"
                            },
                            "source": {
                                "source": "asset",
                                "asset": 0,
                                "video": {"stream": {"kind": "video", "ordinal": 0}},
                                "audio": []
                            }
                        }]
                    }]
                },
                {
                    "id": 1,
                    "items": [{
                        "kind": "clip",
                        "envelope": {
                            "layer_id": 0,
                            "effects": [{"id": 59, "plugin_id": "p.track2", "vendor_t2": "z"}],
                            "transform": {
                                "position": {"const": {"Vec2": [0.0, 0.0]}},
                                "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                                "scale": {"const": {"Vec2": [1.0, 1.0]}},
                                "rotation": {"const": {"F64": 0.0}}
                            },
                            "opacity": {"const": {"F64": 1.0}}
                        },
                        "start": {"num": 0, "den": 1},
                        "duration": {"num": 5, "den": 1},
                        "time_map": {
                            "source_start": {"num": 0, "den": 1},
                            "speed_num": 1,
                            "speed_den": 1,
                            "overrun_mode": "freeze"
                        },
                        "source": {
                            "source": "asset",
                            "asset": 0,
                            "video": {"stream": {"kind": "video", "ordinal": 0}},
                            "audio": []
                        }
                    }]
                }
            ],
            "assets": {"next": 1, "entries": [{
                "id": 0, "name": "media", "asset_type": "video/mp4", "content_hash": "hash"
            }]},
            "next_stable_id": 50
        })
    }

    fn decode_all_plans(root: &Value) -> Vec<LegacyPlanTrackItem> {
        let tracks = root["tracks"].as_array().expect("tracks");
        let mut plans = Vec::new();
        for (ti, track) in tracks.iter().enumerate() {
            let items = track["items"].as_array().expect("items");
            for (ii, item) in items.iter().enumerate() {
                let path = format!("tracks[{ti}].items[{ii}]");
                plans.push(decode_plan_item_from_json(item, &path).expect("decode"));
            }
        }
        plans
    }

    fn d1e_semantics(json: &Value) -> PlannerSemanticSnapshot {
        let bytes = serde_json::to_vec(json).unwrap();
        let (doc, _) = crate::migrate::migrate_bytes(&bytes).expect("d1e migrate");
        semantics_from_document(&doc, doc.next_stable_id.peek_next())
    }

    fn journal_batch_semantics(json: &Value) -> PlannerSemanticSnapshot {
        let counter_before = json["next_stable_id"].as_u64().unwrap_or(0);
        let registry_ids = collect_root_definition_ids(json);
        let plans = decode_all_plans(json);
        let ctx = LegacyPlanMaterializeContext {
            counter_before,
            registry_definition_ids: &registry_ids,
            document: None,
        };
        let bundle = plan_and_materialize_legacy_items(&ctx, &plans).expect("batch plan");
        let mut definition_ids: Vec<u64> = registry_ids;
        definition_ids.extend(bundle.introduced_definitions.iter().map(|d| d.id.get()));
        let mut doc = Document::new_current();
        doc.effect_definitions = json["effect_definitions"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|v| serde_json::from_value::<EffectDefinition>(v.clone()).unwrap())
                    .collect()
            })
            .unwrap_or_default();
        doc.effect_definitions
            .extend(bundle.introduced_definitions.clone());
        doc.tracks.push(Track {
            id: doc.track_ids.allocate("V1").unwrap(),
            items: vec![bundle.materialized_items[0].clone()],
        });
        doc.tracks.push(Track {
            id: doc.track_ids.allocate("V2").unwrap(),
            items: vec![bundle.materialized_items[1].clone()],
        });
        PlannerSemanticSnapshot {
            definition_ids,
            use_pairs: collect_all_use_pairs_from_items(&bundle.materialized_items),
            keyframe_ids: collect_keyframe_ids_from_definitions(&doc.effect_definitions),
            keyframe_details: collect_keyframe_details_from_definitions(&doc.effect_definitions),
            counter_after: bundle.counter_after,
            extras: doc
                .effect_definitions
                .iter()
                .map(|d| Value::Object(d.extra.clone()))
                .collect(),
        }
    }

    fn collect_all_use_pairs_from_items(items: &[TrackItem]) -> Vec<(u64, u64)> {
        let mut out = Vec::new();
        for item in items {
            collect_item_use_pairs(item, &mut out);
        }
        out
    }

    fn collect_keyframe_ids_from_definitions(defs: &[EffectDefinition]) -> Vec<u64> {
        let mut ids = Vec::new();
        let mut details = Vec::new();
        for def in defs {
            for param in def.params.values() {
                collect_keyframe_semantics(param, &mut ids, &mut details);
            }
        }
        ids
    }

    fn collect_keyframe_details_from_definitions(
        defs: &[EffectDefinition],
    ) -> Vec<(u64, RationalTime, String, String)> {
        let mut ids = Vec::new();
        let mut details = Vec::new();
        for def in defs {
            for param in def.params.values() {
                collect_keyframe_semantics(param, &mut ids, &mut details);
            }
        }
        details
    }

    /// integration test からも同一 batch parity を審判する（製品 API ではない）。
    pub fn assert_d1e_journal_batch_planner_semantic_parity(json: &Value) {
        let max_observed = [54u64, 55, 56, 57, 58, 59, 52].into_iter().max().unwrap();
        let counter = json["next_stable_id"].as_u64().unwrap();
        assert!(
            max_observed > counter,
            "fixture must have max_observed > next_stable_id"
        );

        let d1e = d1e_semantics(json);
        let batch = journal_batch_semantics(json);

        assert_eq!(d1e.definition_ids, batch.definition_ids, "definition ids");
        assert_eq!(d1e.use_pairs, batch.use_pairs, "use/definition pairs");
        assert_eq!(d1e.keyframe_ids, batch.keyframe_ids, "keyframe ids");
        assert_eq!(
            d1e.keyframe_details, batch.keyframe_details,
            "keyframe value/time/interp"
        );
        assert_eq!(d1e.counter_after, batch.counter_after, "counter_after");
        assert_eq!(d1e.extras, batch.extras, "extra fields");
    }

    #[test]
    fn d1e_and_journal_batch_planner_semantic_parity() {
        assert_d1e_journal_batch_planner_semantic_parity(&complex_legacy_inline_document_json());
    }
}

#[cfg(test)]
mod zero_inline_modern_tests {
    use super::*;
    use crate::schema::{Clip, ClipSource, ItemEnvelope, Track, TrackItem};
    use motolii_core::RationalTime;
    use serde_json::json;

    fn minimal_clip_envelope_json(effects: serde_json::Value) -> serde_json::Value {
        json!({
            "layer_id": 0,
            "effects": effects,
            "transform": {
                "position": {"const": {"Vec2": [0.0, 0.0]}},
                "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                "scale": {"const": {"Vec2": [1.0, 1.0]}},
                "rotation": {"const": {"F64": 0.0}}
            },
            "opacity": {"const": {"F64": 1.0}}
        })
    }

    fn minimal_clip_item_json(effects: serde_json::Value) -> serde_json::Value {
        json!({
            "kind": "clip",
            "envelope": minimal_clip_envelope_json(effects),
            "start": {"num": 0, "den": 1},
            "duration": {"num": 5, "den": 1},
            "time_map": {
                "source_start": {"num": 0, "den": 1},
                "speed_num": 1,
                "speed_den": 1,
                "overrun_mode": "freeze"
            },
            "source": {
                "source": "asset",
                "asset": 0,
                "video": {"stream": {"kind": "video", "ordinal": 0}},
                "audio": []
            }
        })
    }

    fn modern_only_plan(effects: serde_json::Value) -> LegacyPlanTrackItem {
        decode_plan_item_from_json(&minimal_clip_item_json(effects), "test").expect("decode")
    }

    fn doc_with_existing_modern_use(use_id: u64, def_id: u64) -> Document {
        let mut doc = Document::new_current();
        let layer = doc.layers.allocate("a").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        doc.effect_definitions.push(EffectDefinition::new(
            EffectDefinitionId::from_raw(def_id),
            "p.existing",
            1,
            true,
            Default::default(),
            Default::default(),
        ));
        let mut env = ItemEnvelope::new(layer);
        env.effects.push(EffectUse {
            id: EffectId::from_raw(use_id),
            definition_id: EffectDefinitionId::from_raw(def_id),
        });
        doc.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: env,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            })],
        });
        let max_id = use_id.max(def_id);
        doc.next_stable_id.commit_validated_reservation(max_id + 1);
        doc.validate().unwrap();
        doc
    }

    #[test]
    fn zero_inline_duplicate_use_id_in_payload_rejects_before_materialize() {
        let plan = modern_only_plan(json!([
            {"id": 10, "definition_id": 5},
            {"id": 10, "definition_id": 6},
        ]));
        let doc = doc_with_existing_modern_use(99, 5);
        let before = doc.clone();
        let registry_ids = document_definition_ids(&doc);
        let counter = doc.next_stable_id.peek_next();
        let ctx = LegacyPlanMaterializeContext {
            counter_before: counter,
            registry_definition_ids: &registry_ids,
            document: Some(&doc),
        };
        let result = plan_and_materialize_legacy_items(&ctx, std::slice::from_ref(&plan));
        assert!(matches!(
            result,
            Err(LegacyEffectMigrationError::PayloadIdCollision { id: 10 })
        ));
        assert_eq!(doc, before);
    }

    #[test]
    fn zero_inline_use_id_document_collision_rejects_with_full_document_unchanged() {
        let plan = modern_only_plan(json!([{"id": 10, "definition_id": 5}]));
        let doc = doc_with_existing_modern_use(10, 5);
        let before = doc.clone();
        let registry_ids = document_definition_ids(&doc);
        let counter = doc.next_stable_id.peek_next();
        let ctx = LegacyPlanMaterializeContext {
            counter_before: counter,
            registry_definition_ids: &registry_ids,
            document: Some(&doc),
        };
        let result = plan_and_materialize_legacy_items(&ctx, std::slice::from_ref(&plan));
        assert!(matches!(
            result,
            Err(LegacyEffectMigrationError::DocumentIdCollision { id: 10 })
        ));
        assert_eq!(doc, before);
    }

    #[test]
    fn zero_inline_legal_modern_use_materializes_with_unchanged_counter() {
        let plan = modern_only_plan(json!([{"id": 20, "definition_id": 5}]));
        let doc = doc_with_existing_modern_use(99, 5);
        let registry_ids = document_definition_ids(&doc);
        let counter = doc.next_stable_id.peek_next();
        let ctx = LegacyPlanMaterializeContext {
            counter_before: counter,
            registry_definition_ids: &registry_ids,
            document: Some(&doc),
        };
        let bundle = plan_and_materialize_legacy_items(&ctx, std::slice::from_ref(&plan)).unwrap();
        assert!(!bundle.migrated);
        assert_eq!(bundle.counter_after, counter);
        assert!(bundle.introduced_definitions.is_empty());
        let TrackItem::Clip(clip) = &bundle.materialized_items[0] else {
            panic!("expected clip");
        };
        assert_eq!(clip.envelope.effects.len(), 1);
        assert_eq!(clip.envelope.effects[0].id.get(), 20);
        assert_eq!(clip.envelope.effects[0].definition_id.get(), 5);
    }

    #[test]
    fn fully_modern_d1e_document_migrate_is_idempotent() {
        let root = json!({
            "version": 5,
            "min_reader_version": 5,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1},
                "camera": {
                    "kind": "planar_orthographic",
                    "center": {"const": {"Vec2": [0.0, 0.0]}},
                    "roll_radians": {"const": {"F64": 0.0}},
                    "height": {"const": {"F64": 1.0}}
                }
            },
            "bpm": {"num": 120, "den": 1},
            "layers": {"next": 1, "entries": [{"id": 0, "name": "a"}]},
            "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
            "effect_definitions": [{
                "id": 5,
                "plugin_id": "p.modern",
                "effect_version": 1,
                "enabled": true,
                "params": {}
            }],
            "tracks": [{
                "id": 0,
                "items": [minimal_clip_item_json(json!([{"id": 10, "definition_id": 5}]))]
            }],
            "assets": {"next": 1, "entries": [{
                "id": 0, "name": "media", "asset_type": "video/mp4", "content_hash": "hash"
            }]},
            "next_stable_id": 11
        });
        let bytes = serde_json::to_vec(&root).unwrap();
        let (doc, report) = crate::migrate::migrate_bytes(&bytes).expect("first migrate");
        assert!(!report.did_migrate());
        let reserialized = serde_json::to_vec(&doc).unwrap();
        let (doc2, report2) = crate::migrate::migrate_bytes(&reserialized).expect("second migrate");
        assert!(!report2.did_migrate());
        assert_eq!(doc2, doc);
    }
}
