//! journal replay 専用: v1 `LegacyJournalCommand` decode / adapter / apply。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::command::{envelope_of, find_envelope, find_items_vec, Command, ParentLocator};
use crate::legacy_effect_migrate::{
    apply_prepared_legacy_edit, check_document_collisions, check_payload_id_uniqueness,
    collect_legacy_effect_ids, document_definition_ids, legacy_effect_matches_definition,
    legacy_track_item_to_plan, plan_and_materialize_legacy_items, LegacyEffectMigrationError,
    LegacyEffectMigrationPlanner, LegacyInlineEffect, LegacyItemEnvelope,
    LegacyPlanMaterializeContext, LegacyTrackItem, PreparedLegacyEdit,
};
use crate::param::DocParam;
use crate::schema::{
    BlendMode, Clip, ClippingMaskSettings, EffectDefinition, EffectInstance, EffectUse, Group,
    ItemEnvelope, TrackItem,
};
use crate::stable_id::{EffectDefinitionId, EffectId};
use crate::{Document, LayerId};

/// v1 journal Edit の serde tag 全体(v2 lifecycle variant を含まない)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) enum LegacyJournalCommand {
    SetProperty {
        target: LayerId,
        property: crate::command::ScalarPropertyId,
        old_value: DocParam,
        new_value: DocParam,
    },
    SetBlendMode {
        target: LayerId,
        old: BlendMode,
        new: BlendMode,
    },
    SetClippingMask {
        target: LayerId,
        old: ClippingMaskSettings,
        new: ClippingMaskSettings,
    },
    SetTransformParent {
        target: LayerId,
        old: Option<LayerId>,
        new: Option<LayerId>,
    },
    AddEffect {
        target: LayerId,
        index: usize,
        effect: LegacyInlineEffect,
    },
    RemoveEffect {
        target: LayerId,
        index: usize,
        effect: LegacyInlineEffect,
    },
    SetEffectEnabled {
        target: LayerId,
        effect: EffectId,
        old: bool,
        new: bool,
    },
    DeleteEffectDefinition {
        definition: EffectDefinition,
    },
    AddEffectDefinition {
        definition: EffectDefinition,
    },
    SetAudioComponentEnabled {
        target: LayerId,
        index: usize,
        old: bool,
        new: bool,
    },
    SetAudioComponentGain {
        target: LayerId,
        index: usize,
        old: DocParam,
        new: DocParam,
    },
    AddTrackItem {
        parent: ParentLocator,
        index: usize,
        item: LegacyTrackItem,
        layer_names: BTreeMap<LayerId, String>,
    },
    RemoveTrackItem {
        parent: ParentLocator,
        index: usize,
        item: LegacyTrackItem,
        layer_names: BTreeMap<LayerId, String>,
    },
}

impl LegacyJournalCommand {
    fn to_current_command(&self) -> Command {
        match self.clone() {
            LegacyJournalCommand::SetProperty {
                target,
                property,
                old_value,
                new_value,
            } => Command::SetProperty {
                target,
                property,
                old_value,
                new_value,
            },
            LegacyJournalCommand::SetBlendMode { target, old, new } => {
                Command::SetBlendMode { target, old, new }
            }
            LegacyJournalCommand::SetClippingMask { target, old, new } => {
                Command::SetClippingMask { target, old, new }
            }
            LegacyJournalCommand::SetTransformParent { target, old, new } => {
                Command::SetTransformParent { target, old, new }
            }
            LegacyJournalCommand::SetEffectEnabled {
                target,
                effect,
                old,
                new,
            } => Command::SetEffectEnabled {
                target,
                effect,
                old,
                new,
            },
            LegacyJournalCommand::DeleteEffectDefinition { definition } => {
                Command::DeleteEffectDefinition { definition }
            }
            LegacyJournalCommand::AddEffectDefinition { definition } => {
                Command::AddEffectDefinition { definition }
            }
            LegacyJournalCommand::SetAudioComponentEnabled {
                target,
                index,
                old,
                new,
            } => Command::SetAudioComponentEnabled {
                target,
                index,
                old,
                new,
            },
            LegacyJournalCommand::SetAudioComponentGain {
                target,
                index,
                old,
                new,
            } => Command::SetAudioComponentGain {
                target,
                index,
                old,
                new,
            },
            LegacyJournalCommand::AddEffect { .. }
            | LegacyJournalCommand::RemoveEffect { .. }
            | LegacyJournalCommand::AddTrackItem { .. }
            | LegacyJournalCommand::RemoveTrackItem { .. } => {
                unreachable!("effect-bearing legacy commands use planner")
            }
        }
    }
}

/// v1 command を `PreparedLegacyEdit` へ具体化する(journal adapter 正本)。
pub(crate) fn plan_v1_journal_command(
    doc: &Document,
    legacy: &LegacyJournalCommand,
) -> Result<PreparedLegacyEdit, LegacyEffectMigrationError> {
    let counter_before = doc.next_stable_id.peek_next();
    match legacy {
        LegacyJournalCommand::AddEffect {
            target,
            index,
            effect,
        } => plan_add_effect(doc, *target, *index, effect, counter_before),
        LegacyJournalCommand::RemoveEffect {
            target,
            index,
            effect,
        } => plan_remove_effect(doc, *target, *index, effect, counter_before),
        LegacyJournalCommand::AddTrackItem {
            parent,
            index,
            item,
            layer_names,
        } => plan_add_track_item(doc, *parent, *index, item, layer_names, counter_before),
        LegacyJournalCommand::RemoveTrackItem {
            parent,
            index,
            item,
            layer_names,
        } => plan_remove_track_item(doc, *parent, *index, item, layer_names, counter_before),
        other => Ok(PreparedLegacyEdit {
            expected_counter_before: counter_before,
            counter_after: counter_before,
            complete_payload: other.to_current_command(),
            introduced_definitions: Vec::new(),
            destroyed_definition_ids: Vec::new(),
        }),
    }
}

pub(crate) fn apply_v1_journal_command(
    doc: &mut Document,
    legacy: &LegacyJournalCommand,
) -> Result<(), LegacyEffectMigrationError> {
    let prepared = plan_v1_journal_command(doc, legacy)?;
    apply_prepared_legacy_edit(doc, &prepared)
}

fn plan_add_effect(
    doc: &Document,
    target: LayerId,
    index: usize,
    effect: &LegacyInlineEffect,
    counter_before: u64,
) -> Result<PreparedLegacyEdit, LegacyEffectMigrationError> {
    let introduced = collect_legacy_effect_ids(effect);
    check_payload_id_uniqueness(&introduced)?;
    check_document_collisions(doc, &introduced)?;
    let mut watermark = introduced.clone();
    watermark.extend(document_definition_ids(doc));
    let (plan_start, counter_after) =
        LegacyEffectMigrationPlanner::compute_counter_watermark(counter_before, &watermark, 1)?;
    check_document_collisions(doc, &[plan_start])?;
    let def_id = EffectDefinitionId::from_raw(plan_start);
    Ok(PreparedLegacyEdit {
        expected_counter_before: counter_before,
        counter_after,
        complete_payload: Command::AddEffect {
            target,
            index,
            effect: effect.to_instance(def_id),
            introduced_definition: true,
        },
        introduced_definitions: Vec::new(),
        destroyed_definition_ids: Vec::new(),
    })
}

fn plan_remove_effect(
    doc: &Document,
    target: LayerId,
    index: usize,
    effect: &LegacyInlineEffect,
    counter_before: u64,
) -> Result<PreparedLegacyEdit, LegacyEffectMigrationError> {
    let env = find_envelope(doc, target).ok_or(LegacyEffectMigrationError::Command(
        crate::command::CommandError::LayerNotFound(target.get()),
    ))?;
    if index >= env.effects.len() {
        return Err(LegacyEffectMigrationError::Command(
            crate::command::CommandError::IndexOutOfRange {
                index,
                len: env.effects.len(),
            },
        ));
    }
    let at = &env.effects[index];
    if at.id != effect.id {
        return Err(LegacyEffectMigrationError::RemoveEffectMismatch { index });
    }
    let def =
        doc.effect_definition(at.definition_id)
            .ok_or(LegacyEffectMigrationError::Command(
                crate::command::CommandError::EffectDefinitionNotFound {
                    id: at.definition_id.get(),
                },
            ))?;
    if !legacy_effect_matches_definition(effect, def) {
        return Err(LegacyEffectMigrationError::RemoveEffectMismatch { index });
    }
    if doc.effect_use_count(at.definition_id) > 1 {
        let use_ids = doc
            .effect_use_ids(at.definition_id)
            .into_iter()
            .map(|id| id.get())
            .collect();
        return Err(LegacyEffectMigrationError::DefinitionShared {
            id: at.definition_id.get(),
            use_ids,
        });
    }
    let instance = EffectInstance::from_use_and_definition(at, def);
    Ok(PreparedLegacyEdit {
        expected_counter_before: counter_before,
        counter_after: counter_before,
        complete_payload: Command::RemoveEffect {
            target,
            index,
            effect: instance,
            introduced_definition: true,
        },
        introduced_definitions: Vec::new(),
        destroyed_definition_ids: Vec::new(),
    })
}

fn plan_add_track_item(
    doc: &Document,
    parent: ParentLocator,
    index: usize,
    item: &LegacyTrackItem,
    layer_names: &BTreeMap<LayerId, String>,
    counter_before: u64,
) -> Result<PreparedLegacyEdit, LegacyEffectMigrationError> {
    let len = find_items_vec(doc, parent)
        .map_err(LegacyEffectMigrationError::Command)?
        .len();
    if index > len {
        return Err(LegacyEffectMigrationError::Command(
            crate::command::CommandError::IndexOutOfRange { index, len },
        ));
    }
    let plan_item = legacy_track_item_to_plan(item);
    let registry_ids = document_definition_ids(doc);
    let ctx = LegacyPlanMaterializeContext {
        counter_before,
        registry_definition_ids: &registry_ids,
        document: Some(doc),
    };
    let bundle = plan_and_materialize_legacy_items(&ctx, std::slice::from_ref(&plan_item))?;
    for def in &bundle.introduced_definitions {
        check_document_collisions(doc, &[def.id.get()])?;
    }
    let migrated_item = bundle
        .materialized_items
        .into_iter()
        .next()
        .expect("single item migration");
    Ok(PreparedLegacyEdit {
        expected_counter_before: counter_before,
        counter_after: bundle.counter_after,
        complete_payload: Command::AddTrackItem {
            parent,
            index,
            item: migrated_item,
            layer_names: layer_names.clone(),
        },
        introduced_definitions: bundle.introduced_definitions,
        destroyed_definition_ids: Vec::new(),
    })
}

fn plan_remove_track_item(
    doc: &Document,
    parent: ParentLocator,
    index: usize,
    item: &LegacyTrackItem,
    layer_names: &BTreeMap<LayerId, String>,
    counter_before: u64,
) -> Result<PreparedLegacyEdit, LegacyEffectMigrationError> {
    let items = find_items_vec(doc, parent).map_err(LegacyEffectMigrationError::Command)?;
    if index >= items.len() {
        return Err(LegacyEffectMigrationError::Command(
            crate::command::CommandError::IndexOutOfRange {
                index,
                len: items.len(),
            },
        ));
    }
    let migrated_expected = migrate_legacy_track_item_for_remove(doc, item)?;
    if items[index] != migrated_expected {
        return Err(LegacyEffectMigrationError::RemoveTrackItemMismatch { index });
    }
    let destroyed = collect_sole_use_definitions_in_subtree(doc, &items[index])?;
    Ok(PreparedLegacyEdit {
        expected_counter_before: counter_before,
        counter_after: counter_before,
        complete_payload: Command::RemoveTrackItem {
            parent,
            index,
            item: items[index].clone(),
            layer_names: layer_names.clone(),
        },
        introduced_definitions: Vec::new(),
        destroyed_definition_ids: destroyed,
    })
}

fn migrate_legacy_track_item_for_remove(
    doc: &Document,
    item: &LegacyTrackItem,
) -> Result<TrackItem, LegacyEffectMigrationError> {
    match item {
        LegacyTrackItem::Clip {
            envelope,
            start,
            duration,
            time_map,
            source,
        } => {
            let effects = resolve_legacy_effects(doc, envelope.as_ref())?;
            Ok(TrackItem::Clip(Clip {
                envelope: ItemEnvelope {
                    layer_id: envelope.layer_id,
                    effects,
                    transform: envelope.transform.clone(),
                    opacity: envelope.opacity.clone(),
                    blend: envelope.blend,
                    clipping_mask: envelope.clipping_mask.clone(),
                    visible: envelope.visible,
                    solo: envelope.solo,
                    lock: envelope.lock,
                },
                start: *start,
                duration: *duration,
                time_map: *time_map,
                source: source.as_ref().clone(),
            }))
        }
        LegacyTrackItem::Group { envelope, children } => {
            let effects = resolve_legacy_effects(doc, envelope.as_ref())?;
            let mut migrated_children = Vec::new();
            for child in children {
                migrated_children.push(migrate_legacy_track_item_for_remove(doc, child)?);
            }
            Ok(TrackItem::Group(Group {
                envelope: ItemEnvelope {
                    layer_id: envelope.layer_id,
                    effects,
                    transform: envelope.transform.clone(),
                    opacity: envelope.opacity.clone(),
                    blend: envelope.blend,
                    clipping_mask: envelope.clipping_mask.clone(),
                    visible: envelope.visible,
                    solo: envelope.solo,
                    lock: envelope.lock,
                },
                children: migrated_children,
            }))
        }
    }
}

fn resolve_legacy_effects(
    doc: &Document,
    env: &LegacyItemEnvelope,
) -> Result<Vec<EffectUse>, LegacyEffectMigrationError> {
    let layer = env.layer_id;
    let found = find_envelope(doc, layer).ok_or(LegacyEffectMigrationError::Command(
        crate::command::CommandError::LayerNotFound(layer.get()),
    ))?;
    let mut out = Vec::with_capacity(env.effects.len());
    for inline in &env.effects {
        let at = found
            .effects
            .iter()
            .find(|u| u.id == inline.id)
            .ok_or(LegacyEffectMigrationError::RemoveTrackItemMismatch { index: 0 })?;
        let def =
            doc.effect_definition(at.definition_id)
                .ok_or(LegacyEffectMigrationError::Command(
                    crate::command::CommandError::EffectDefinitionNotFound {
                        id: at.definition_id.get(),
                    },
                ))?;
        if !legacy_effect_matches_definition(inline, def) {
            return Err(LegacyEffectMigrationError::RemoveTrackItemMismatch { index: 0 });
        }
        out.push(at.clone());
    }
    Ok(out)
}

fn collect_sole_use_definitions_in_subtree(
    doc: &Document,
    item: &TrackItem,
) -> Result<Vec<EffectDefinitionId>, LegacyEffectMigrationError> {
    let mut defs = Vec::new();
    walk_item_definitions(doc, item, &mut defs)?;
    Ok(defs)
}

fn walk_item_definitions(
    doc: &Document,
    item: &TrackItem,
    out: &mut Vec<EffectDefinitionId>,
) -> Result<(), LegacyEffectMigrationError> {
    let env = envelope_of(item);
    for u in &env.effects {
        let count = doc.effect_use_count(u.definition_id);
        if count > 1 {
            let use_ids = doc
                .effect_use_ids(u.definition_id)
                .into_iter()
                .map(|id| id.get())
                .collect();
            return Err(LegacyEffectMigrationError::DefinitionShared {
                id: u.definition_id.get(),
                use_ids,
            });
        }
        if count == 1 {
            out.push(u.definition_id);
        }
    }
    if let TrackItem::Group(g) = item {
        for child in &g.children {
            walk_item_definitions(doc, child, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::command::ParentLocator;
    use crate::schema::{Clip, ClipSource, EffectDefinition, ItemEnvelope, Track, TrackItem};
    use crate::stable_id::{EffectDefinitionId, EffectId};
    use crate::{Document, LayerId};
    use motolii_core::RationalTime;
    use serde_json::json;

    const FIXTURE_CORPUS: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/journal_v1/commands.jsonl"
    );

    #[test]
    fn corpus_lines_decode_as_legacy_journal_command() {
        for line in fs::read_to_string(FIXTURE_CORPUS).unwrap().lines() {
            if line.trim().is_empty() {
                continue;
            }
            let _: LegacyJournalCommand = serde_json::from_str(line).unwrap_or_else(|e| {
                panic!("corpus line must decode as LegacyJournalCommand: {e}\n{line}")
            });
        }
    }

    #[test]
    fn journal_adapter_counter_mismatch_rejects_without_mutation() {
        let (mut doc, layer, _) = empty_clip_base_for_tests();
        let legacy = LegacyJournalCommand::SetProperty {
            target: layer,
            property: crate::command::ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.5),
        };
        let prepared = plan_v1_journal_command(&doc, &legacy).unwrap();
        doc.next_stable_id.allocate().unwrap();
        let before = doc.clone();
        let err = apply_prepared_legacy_edit(&mut doc, &prepared).unwrap_err();
        assert!(matches!(
            err,
            LegacyEffectMigrationError::CounterMismatch { .. }
        ));
        assert_eq!(doc, before);
    }

    #[test]
    fn shared_definition_remove_effect_rejects_without_mutation() {
        let (mut doc, layer, _) = shared_remove_fixture();
        let before = doc.clone();
        let legacy = LegacyJournalCommand::RemoveEffect {
            target: layer,
            index: 0,
            effect: shared_inline_effect(),
        };
        let err = apply_v1_journal_command(&mut doc, &legacy).unwrap_err();
        assert!(matches!(
            err,
            LegacyEffectMigrationError::DefinitionShared { .. }
        ));
        assert_eq!(doc, before);
    }

    #[test]
    fn shared_definition_remove_track_item_rejects_without_mutation() {
        let (mut doc, _, track) = shared_remove_fixture();
        let before = doc.clone();
        let legacy = LegacyJournalCommand::RemoveTrackItem {
            parent: ParentLocator::Track(track),
            index: 0,
            item: shared_track_item_payload(),
            layer_names: BTreeMap::from([(LayerId::from_raw(0), "a".into())]),
        };
        let err = apply_v1_journal_command(&mut doc, &legacy).unwrap_err();
        assert!(matches!(
            err,
            LegacyEffectMigrationError::DefinitionShared { .. }
        ));
        assert_eq!(doc, before);
    }

    fn shared_inline_effect() -> LegacyInlineEffect {
        serde_json::from_value(json!({
            "id": 5,
            "plugin_id": "vendor.unknown.glow",
            "effect_version": 2,
            "enabled": false,
            "params": {"amount": {"const": {"F64": 0.75}}},
            "vendor_custom": "keep-me"
        }))
        .unwrap()
    }

    fn shared_track_item_payload() -> LegacyTrackItem {
        let asset = 0u64;
        serde_json::from_value(json!({
            "kind": "clip",
            "envelope": {
                "layer_id": 0,
                "effects": [{"id": 5, "plugin_id": "vendor.unknown.glow", "effect_version": 2, "enabled": false, "params": {"amount": {"const": {"F64": 0.75}}}, "vendor_custom": "keep-me"}],
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
                "asset": asset,
                "video": {"stream": {"kind": "video", "ordinal": 0}},
                "audio": []
            }
        }))
        .unwrap()
    }

    fn empty_clip_base_for_tests() -> (Document, LayerId, crate::TrackId) {
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
        (doc, layer, track)
    }

    fn shared_remove_fixture() -> (Document, LayerId, crate::TrackId) {
        let mut doc = Document::new_current();
        let layer_a = doc.layers.allocate("a").unwrap();
        let layer_b = doc.layers.allocate("b").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        let u1 = EffectId::from_raw(5);
        let u2 = EffectId::from_raw(6);
        let d1 = EffectDefinitionId::from_raw(10);
        doc.effect_definitions.push(EffectDefinition::new(
            d1,
            "vendor.unknown.glow",
            2,
            false,
            BTreeMap::from([("amount".into(), DocParam::const_f64(0.75))]),
            serde_json::Map::from_iter([("vendor_custom".into(), json!("keep-me"))]),
        ));
        let mut env_a = ItemEnvelope::new(layer_a);
        env_a.effects.push(EffectUse {
            id: u1,
            definition_id: d1,
        });
        let mut env_b = ItemEnvelope::new(layer_b);
        env_b.effects.push(EffectUse {
            id: u2,
            definition_id: d1,
        });
        doc.tracks.push(Track {
            id: track,
            items: vec![
                TrackItem::Clip(Clip {
                    envelope: env_a,
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(5, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                }),
                TrackItem::Clip(Clip {
                    envelope: env_b,
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(5, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                }),
            ],
        });
        while doc.next_stable_id.peek_next() < 11 {
            let _ = doc.next_stable_id.allocate();
        }
        doc.validate().unwrap();
        (doc, layer_a, track)
    }
}
