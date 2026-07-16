//! D1l Stage C: JournalEdit v1→v2 互換境界(決定書 §3 / §6(3–5,9,11,12) / §8(3))。

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use motolii_core::RationalTime;
use motolii_doc::journal::{
    replay_from_base, JournalEdit, JournalRecordKind, JournalScanOutcome, ReplayFailure,
    V1_EDIT_FORMAT_VERSION, V2_EDIT_FORMAT_VERSION,
};
use motolii_doc::{
    migrate_bytes, Clip, ClipSource, Command, DocParam, Document, EffectDefinition,
    EffectDefinitionId, EffectId, EffectUse, ItemEnvelope, LayerId, ScalarPropertyId, Track,
    TrackItem,
};
use serde_json::{json, Value as JsonValue};
use syn::parse_file;

const FIXTURE_CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/journal_v1/commands.jsonl"
);

fn v1_edit_payload(command: JsonValue) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "format_version": V1_EDIT_FORMAT_VERSION,
        "command": command,
    }))
    .unwrap()
}

fn empty_clip_base() -> (Document, LayerId, motolii_doc::TrackId) {
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

fn legacy_inline_doc_json(effect: JsonValue) -> JsonValue {
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
        "layers": {"next": 1, "entries": [{"id": 0, "name": "a"}]},
        "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [effect],
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
        }],
        "assets": {"next": 1, "entries": [{
            "id": 0, "name": "media", "asset_type": "video/mp4", "content_hash": "hash"
        }]},
        "next_stable_id": 6
    })
}

fn inline_effect_json() -> JsonValue {
    json!({
        "id": 5,
        "plugin_id": "vendor.unknown.glow",
        "effect_version": 2,
        "enabled": false,
        "params": {"amount": {"const": {"F64": 0.75}}},
        "vendor_custom": "keep-me"
    })
}

fn legacy_empty_base_json() -> JsonValue {
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
        "layers": {"next": 1, "entries": [{"id": 0, "name": "a"}]},
        "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "effects": [],
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
        }],
        "assets": {"next": 1, "entries": [{
            "id": 0, "name": "media", "asset_type": "video/mp4", "content_hash": "hash"
        }]},
        "next_stable_id": 6
    })
}

fn assert_ledger_and_tracks_eq(a: &Document, b: &Document) {
    assert_eq!(a.tracks, b.tracks);
    assert_eq!(a.effect_definitions, b.effect_definitions);
}

fn assert_remove_restores_ledger(base: &Document, after_remove: &Document) {
    assert_ledger_and_tracks_eq(after_remove, base);
}

fn replay_single_edit(base: Document, payload: &[u8]) -> Document {
    let scan = JournalScanOutcome {
        header: motolii_doc::journal::JournalHeader {
            version: 1,
            generation_salt: 1,
            project_id: uuid::Uuid::new_v4(),
        },
        frames: vec![motolii_doc::journal::JournalFrame {
            record_id: uuid::Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: None,
            record_salt: 1,
            kind: JournalRecordKind::Edit,
            payload: payload.to_vec(),
        }],
        valid_bytes: 0,
        file_len: 0,
        stopped: None,
    };
    let outcome = replay_from_base(base, &scan, &mut |_| unreachable!(), false);
    assert!(
        outcome.replay_failures.is_empty(),
        "failures={:?}",
        outcome.replay_failures
    );
    outcome.document
}

fn replay_single_edit_or_fail(base: Document, payload: &[u8]) -> (Document, Vec<ReplayFailure>) {
    let scan = JournalScanOutcome {
        header: motolii_doc::journal::JournalHeader {
            version: 1,
            generation_salt: 1,
            project_id: uuid::Uuid::new_v4(),
        },
        frames: vec![motolii_doc::journal::JournalFrame {
            record_id: uuid::Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: None,
            record_salt: 1,
            kind: JournalRecordKind::Edit,
            payload: payload.to_vec(),
        }],
        valid_bytes: 0,
        file_len: 0,
        stopped: None,
    };
    let outcome = replay_from_base(base, &scan, &mut |_| unreachable!(), false);
    (outcome.document, outcome.replay_failures)
}

fn legacy_journal_command_tags() -> BTreeSet<String> {
    let source = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/journal/v1_edit.rs"
    ))
    .unwrap();
    let file = parse_file(&source).expect("parse v1_edit.rs");
    let mut tags = BTreeSet::new();
    for item in &file.items {
        let syn::Item::Enum(e) = item else {
            continue;
        };
        if e.ident != "LegacyJournalCommand" {
            continue;
        }
        for variant in &e.variants {
            tags.insert(variant.ident.to_string());
        }
    }
    tags
}

fn shared_remove_fixture() -> (Document, LayerId) {
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
    (doc, layer_a)
}

// ---------------------------------------------------------------------------
// §6.3–4: v1 raw replay ↔ D1e migration 意味一致
// ---------------------------------------------------------------------------

#[test]
fn v1_add_effect_replay_matches_d1e_migration() {
    let effect = inline_effect_json();
    let (expected, _) =
        migrate_bytes(&serde_json::to_vec(&legacy_inline_doc_json(effect.clone())).unwrap())
            .expect("d1e migrate");

    let (base, layer, _) = empty_clip_base();
    let payload = v1_edit_payload(json!({
        "AddEffect": {
            "target": layer.get(),
            "index": 0,
            "effect": effect
        }
    }));
    let replayed = replay_single_edit(base.clone(), &payload);
    assert_eq!(replayed.effect_definitions, expected.effect_definitions);
    assert_eq!(clip_effects(&replayed), clip_effects(&expected));
    assert_eq!(replayed.next_stable_id, expected.next_stable_id);
    replayed.validate().unwrap();
}

#[test]
fn v1_remove_effect_roundtrip_restores_base_ledger() {
    let effect = inline_effect_json();
    let (mut base, layer, _) = empty_clip_base();
    let add = v1_edit_payload(json!({
        "AddEffect": {"target": layer.get(), "index": 0, "effect": effect.clone()}
    }));
    let before_add = base.clone();
    base = replay_single_edit(base, &add);
    let remove = v1_edit_payload(json!({
        "RemoveEffect": {"target": layer.get(), "index": 0, "effect": effect}
    }));
    let after = replay_single_edit(base, &remove);
    assert_remove_restores_ledger(&before_add, &after);
}

#[test]
fn v1_nested_add_remove_track_item_matches_d1e_and_restores_base() {
    let nested = json!({
        "kind": "clip",
        "envelope": {
            "layer_id": 1,
            "effects": [{"id": 7, "plugin_id": "p.nested", "vendor": "n"}],
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
    });
    let mut legacy_doc = legacy_empty_base_json();
    legacy_doc["layers"]["next"] = json!(2);
    legacy_doc["layers"]["entries"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id": 1, "name": "nested"}));
    let items = legacy_doc["tracks"][0]["items"].as_array_mut().unwrap();
    items.push(nested.clone());

    let (expected, _) = migrate_bytes(&serde_json::to_vec(&legacy_doc).unwrap()).unwrap();

    let (mut base, _, track) = empty_clip_base();
    let add = v1_edit_payload(json!({
        "AddTrackItem": {
            "parent": {"Track": track.get()},
            "index": 1,
            "item": nested,
            "layer_names": {"1": "nested"}
        }
    }));
    let before_add = base.clone();
    base = replay_single_edit(base, &add);
    assert_eq!(base.tracks[0].items.len(), expected.tracks[0].items.len());
    assert_eq!(base.effect_definitions, expected.effect_definitions);

    let remove = v1_edit_payload(json!({
        "RemoveTrackItem": {
            "parent": {"Track": track.get()},
            "index": 1,
            "item": {
                "kind": "clip",
                "envelope": {
                    "layer_id": 1,
                    "effects": [{"id": 7, "plugin_id": "p.nested", "vendor": "n"}],
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
            },
            "layer_names": {"1": "nested"}
        }
    }));
    let after = replay_single_edit(base, &remove);
    assert_remove_restores_ledger(&before_add, &after);
}

#[test]
fn fixture_corpus_covers_all_legacy_journal_command_tags() {
    let expected = legacy_journal_command_tags();
    let corpus_tags: BTreeSet<String> = fs::read_to_string(FIXTURE_CORPUS)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let v: JsonValue = serde_json::from_str(line).unwrap();
            v.as_object().unwrap().keys().next().unwrap().clone()
        })
        .collect();
    assert_eq!(
        corpus_tags, expected,
        "commands.jsonl must list every v1 tag"
    );
}

#[test]
fn zero_new_definitions_leaves_counter_unchanged() {
    let (base, layer, _) = empty_clip_base();
    let counter_before = base.next_stable_id.peek_next();
    let payload = v1_edit_payload(json!({
        "SetProperty": {
            "target": layer.get(),
            "property": "Opacity",
            "old_value": {"const": {"F64": 1.0}},
            "new_value": {"const": {"F64": 0.5}}
        }
    }));
    let doc = replay_single_edit(base, &payload);
    assert_eq!(doc.next_stable_id.peek_next(), counter_before);
}

fn clip_effects(doc: &Document) -> Vec<EffectUse> {
    let TrackItem::Clip(c) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    c.envelope.effects.clone()
}

#[test]
fn new_definition_advances_counter_by_planner_formula() {
    let (base, layer, _) = empty_clip_base();
    let counter_before = base.next_stable_id.peek_next();
    let payload = v1_edit_payload(json!({
        "AddEffect": {
            "target": layer.get(),
            "index": 0,
            "effect": inline_effect_json()
        }
    }));
    let doc = replay_single_edit(base, &payload);
    let plan_start = counter_before.max(5 + 1);
    assert_eq!(doc.next_stable_id.peek_next(), plan_start + 1);
}

// ---------------------------------------------------------------------------
// §6.5,9,11,12: 拒否・非公開・decode 分岐
// ---------------------------------------------------------------------------

#[test]
fn unknown_edit_format_version_is_typed_rejection_not_fallback() {
    let (base, _, _) = empty_clip_base();
    let payload = serde_json::to_vec(&json!({
        "format_version": 99,
        "command": {"SetProperty": {
            "target": 0,
            "property": "Opacity",
            "old_value": {"const": {"F64": 1.0}},
            "new_value": {"const": {"F64": 0.5}}
        }}
    }))
    .unwrap();
    let scan = JournalScanOutcome {
        header: motolii_doc::journal::JournalHeader {
            version: 1,
            generation_salt: 1,
            project_id: uuid::Uuid::new_v4(),
        },
        frames: vec![motolii_doc::journal::JournalFrame {
            record_id: uuid::Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: None,
            record_salt: 1,
            kind: JournalRecordKind::Edit,
            payload,
        }],
        valid_bytes: 0,
        file_len: 0,
        stopped: None,
    };
    let before = base.clone();
    let outcome = replay_from_base(base, &scan, &mut |_| unreachable!(), true);
    assert_eq!(outcome.document, before);
    assert!(outcome.fallback_generation.is_none());
    assert_eq!(outcome.replay_failures.len(), 1);
    assert!(matches!(
        outcome.replay_failures[0],
        ReplayFailure::InvalidEditPayload { .. }
    ));
}

#[test]
fn shared_definition_v1_remove_rejects_document_unchanged() {
    let (base, layer) = shared_remove_fixture();
    let before = base.clone();
    let payload = v1_edit_payload(json!({
        "RemoveEffect": {
            "target": layer.get(),
            "index": 0,
            "effect": inline_effect_json()
        }
    }));
    let (doc, failures) = replay_single_edit_or_fail(base, &payload);
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0], ReplayFailure::ApplyEdit { .. }));
    assert_eq!(doc, before);
}

#[test]
fn planner_and_prepared_are_not_public_api() {
    let lib_rs = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();
    assert!(!lib_rs.contains("pub use legacy_effect_migrate"));
    assert!(!lib_rs.contains("PreparedLegacyEdit"));
    assert!(!lib_rs.contains("LegacyEffectMigrationPlanner"));

    let legacy_rs = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/legacy_effect_migrate.rs"
    ))
    .unwrap();
    assert!(!legacy_rs.contains("pub fn migrate_inline_effects_json"));
    assert!(!legacy_rs.contains("pub struct PreparedLegacyEdit"));
    assert!(!legacy_rs.contains("pub struct LegacyEffectMigrationPlanner"));
    assert!(!legacy_rs.contains("pub enum LegacyEffectMigrationError"));
    assert!(!legacy_rs.contains("LEGACY_EFFECT_MIGRATION_PLANNER_SYMBOL"));
}

#[test]
fn shared_definition_v1_remove_track_item_rejects_document_unchanged() {
    let (base, layer) = shared_remove_fixture();
    let before = base.clone();
    let track = base.tracks[0].id;
    let payload = v1_edit_payload(json!({
        "RemoveTrackItem": {
            "parent": {"Track": track.get()},
            "index": 0,
            "item": {
                "kind": "clip",
                "envelope": {
                    "layer_id": layer.get(),
                    "effects": [inline_effect_json()],
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
            },
            "layer_names": {"0": "a"}
        }
    }));
    let (doc, failures) = replay_single_edit_or_fail(base, &payload);
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0], ReplayFailure::ApplyEdit { .. }));
    assert_eq!(doc, before);
}

fn complex_legacy_inline_document_json() -> JsonValue {
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

#[derive(Debug, PartialEq)]
struct PlannerSemanticSnapshot {
    definition_ids: Vec<u64>,
    use_pairs: Vec<(u64, u64)>,
    keyframe_ids: Vec<u64>,
    keyframe_details: Vec<(u64, RationalTime, String, String)>,
    counter_after: u64,
    extras: Vec<JsonValue>,
}

fn semantics_from_document(doc: &Document) -> PlannerSemanticSnapshot {
    let mut keyframe_ids = Vec::new();
    let mut keyframe_details = Vec::new();
    for def in &doc.effect_definitions {
        for param in def.params.values() {
            collect_keyframe_semantics(param, &mut keyframe_ids, &mut keyframe_details);
        }
    }
    PlannerSemanticSnapshot {
        definition_ids: doc.effect_definitions.iter().map(|d| d.id.get()).collect(),
        use_pairs: effect_uses_in_plan_order(doc),
        keyframe_ids,
        keyframe_details,
        counter_after: doc.next_stable_id.peek_next(),
        extras: definition_extras(doc),
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

fn effect_uses_in_plan_order(doc: &Document) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    for track in &doc.tracks {
        for item in &track.items {
            collect_item_uses(item, &mut out);
        }
    }
    out
}

fn collect_item_uses(item: &TrackItem, out: &mut Vec<(u64, u64)>) {
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
                collect_item_uses(child, out);
            }
        }
    }
}

fn definition_extras(doc: &Document) -> Vec<JsonValue> {
    doc.effect_definitions
        .iter()
        .map(|d| JsonValue::Object(d.extra.clone()))
        .collect()
}

#[test]
fn d1e_and_journal_share_single_planner_traversal() {
    let legacy = complex_legacy_inline_document_json();
    let max_observed = [54u64, 55, 56, 57, 58, 59, 52].into_iter().max().unwrap();
    let counter = legacy["next_stable_id"].as_u64().unwrap();
    assert!(
        max_observed > counter,
        "fixture must have max_observed > next_stable_id"
    );

    let (d1e_doc, _) = migrate_bytes(&serde_json::to_vec(&legacy).unwrap()).expect("d1e migrate");
    let d1e_sem = semantics_from_document(&d1e_doc);

    // 逐次 AddTrackItem は counter が進み D1e 一括とずれる。payload 一括 batch parity は
    // legacy_effect_migrate::planner_parity_tests::d1e_and_journal_batch_planner_semantic_parity が審判する。
    assert_eq!(
        d1e_sem.definition_ids,
        vec![52, 60, 61, 62, 63],
        "definition id order"
    );
    assert_eq!(
        d1e_sem.use_pairs,
        vec![(55, 60), (58, 52), (56, 61), (57, 62), (59, 63)],
        "use/definition pairs in planner walk order"
    );
    assert_eq!(
        d1e_sem.keyframe_ids,
        vec![54],
        "nested keyframe id preserved"
    );
    assert_eq!(
        d1e_sem.keyframe_details,
        vec![(
            54,
            RationalTime::try_new(0, 1).unwrap(),
            serde_json::to_string(&motolii_doc::DocValue::F64(0.5)).unwrap(),
            format!("{:?}", motolii_eval::Interp::Linear),
        )],
        "keyframe value/time/interp"
    );
    assert_eq!(d1e_sem.counter_after, 64, "counter_after = plan_start + n");
    assert_eq!(
        d1e_sem.extras,
        vec![
            json!({"registry_tag": "keep-registry"}),
            json!({"vendor_grp": "keep-group"}),
            json!({"vendor_a": "x"}),
            json!({"vendor_b": "y"}),
            json!({"vendor_t2": "z"}),
        ],
        "definition extra preservation order"
    );
    d1e_doc.validate().unwrap();
}

#[test]
fn raw_journal_edit_helpers_are_not_product_public_api() {
    let journal_mod =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/journal/mod.rs")).unwrap();
    assert!(!journal_mod.contains("apply_edit_payload"));
    assert!(!journal_mod.contains("commit_edit_payload"));
    assert!(!journal_mod.contains("v1_edit_bytes"));

    let lib_rs = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();
    assert!(!lib_rs.contains("apply_edit_payload"));
    assert!(!lib_rs.contains("commit_edit_payload"));
    assert!(!lib_rs.contains("v1_edit_bytes"));
}

#[test]
fn v1_decode_does_not_use_direct_command_serde_path() {
    let replay_rs = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/journal/replay.rs"
    ))
    .unwrap();
    let decode_fn = replay_rs
        .split("fn decode_edit")
        .nth(1)
        .expect("decode_edit");
    assert!(decode_fn.contains("LegacyJournalCommand"));
    assert!(decode_fn.contains("DecodedJournalEdit::V1"));
    let v1_arm = decode_fn
        .split("V1_EDIT_FORMAT_VERSION")
        .nth(1)
        .and_then(|s| s.split("V2_EDIT_FORMAT_VERSION").next())
        .expect("v1 arm");
    assert!(!v1_arm.contains("let edit: JournalEdit"));
    assert!(!v1_arm.contains("edit.command"));
}

#[test]
fn new_journal_edit_writes_format_version_two() {
    let edit = JournalEdit::new(Command::SetProperty {
        target: LayerId::from_raw(0),
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(1.0),
        new_value: DocParam::const_f64(0.5),
    });
    assert_eq!(edit.format_version, V2_EDIT_FORMAT_VERSION);
    let bytes = motolii_doc::journal::edit_payload(&edit).unwrap();
    let v: JsonValue = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["format_version"], V2_EDIT_FORMAT_VERSION);
    assert_ne!(v["format_version"], V1_EDIT_FORMAT_VERSION);
}
