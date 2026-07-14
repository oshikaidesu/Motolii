//! D1l: Effect Definition/Use(GAP-14 / Issue #172)完了条件の機械判定。
//!
//! 決定の正本: `docs/reviews/2026-07-15-shared-effect-lifecycle-decision.md`。
//! §4不変条件・§5試験を1:1で機械判定へ落とす。
//! Cascade/purge/一斉Make Uniqueは延期(本ファイルの対象外)。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map as JsonMap, Value as JsonValue};

use motolii_core::RationalTime;
use motolii_doc::{
    load_document, migrate_bytes, save_document, Clip, ClipSource, Command, CommandError, DocParam,
    Document, DocumentError, DocumentWriter, EffectDefinition, EffectDefinitionId, EffectId,
    EffectInstance, EffectUse, ItemEnvelope, LayerId, Track, TrackItem,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1l-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// フィクスチャ: 決定doc §3の before共通と同じ形
//   definitions: { D1(shared, 未知plugin+extra), D2(unused orphan) }
//   stack[A]: [Use(U1→D1), Use(U2→D1)]
//   stack[B]: [Use(U3→D1)]
// ---------------------------------------------------------------------------

struct Shared {
    doc: Document,
    layer_a: LayerId,
    layer_b: LayerId,
    u1: EffectId,
    u2: EffectId,
    u3: EffectId,
    d1: EffectDefinitionId,
    d2_orphan: EffectDefinitionId,
}

fn shared_fixture() -> Shared {
    let mut doc = Document::new_v1();
    let layer_a = doc.layers.allocate("a").unwrap();
    let layer_b = doc.layers.allocate("b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

    let u1 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let u2 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let u3 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let d1 = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let d2_orphan = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());

    // D1l/F-9: 未知pluginでも`extra`を保持したままlifecycle操作を通す(§2.6)。
    let mut extra = JsonMap::new();
    extra.insert("vendor_custom".into(), json!({"nested": [1, 2, 3]}));
    doc.effect_definitions.push(EffectDefinition::new(
        d1,
        "vendor.unknown.glow",
        1,
        true,
        std::collections::BTreeMap::from([("amount".into(), DocParam::const_f64(0.4))]),
        extra,
    ));
    doc.effect_definitions.push(EffectDefinition::new(
        d2_orphan,
        "core.filter.tint",
        1,
        true,
        std::collections::BTreeMap::from([("amount".into(), DocParam::const_f64(0.1))]),
        Default::default(),
    ));

    let mut env_a = ItemEnvelope::new(layer_a);
    env_a.effects.push(EffectUse {
        id: u1,
        definition_id: d1,
    });
    env_a.effects.push(EffectUse {
        id: u2,
        definition_id: d1,
    });
    let mut env_b = ItemEnvelope::new(layer_b);
    env_b.effects.push(EffectUse {
        id: u3,
        definition_id: d1,
    });

    doc.version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;
    doc.min_reader_version = motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS;

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
    doc.validate().expect("fixture must be valid");

    Shared {
        doc,
        layer_a,
        layer_b,
        u1,
        u2,
        u3,
        d1,
        d2_orphan,
    }
}

fn effects_on(doc: &Document, layer: LayerId) -> Vec<EffectUse> {
    doc.find_effect_use_all(layer)
}

// `Document`に無い薄い読み取りヘルパ(テスト専用)。
trait FindAllEffects {
    fn find_effect_use_all(&self, layer: LayerId) -> Vec<EffectUse>;
}

impl FindAllEffects for Document {
    fn find_effect_use_all(&self, layer: LayerId) -> Vec<EffectUse> {
        fn walk(items: &[TrackItem], layer: LayerId) -> Option<Vec<EffectUse>> {
            for item in items {
                match item {
                    TrackItem::Clip(c) if c.envelope.layer_id == layer => {
                        return Some(c.envelope.effects.clone())
                    }
                    TrackItem::Group(g) if g.envelope.layer_id == layer => {
                        return Some(g.envelope.effects.clone())
                    }
                    TrackItem::Group(g) => {
                        if let Some(found) = walk(&g.children, layer) {
                            return Some(found);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        self.tracks
            .iter()
            .find_map(|t| walk(&t.items, layer))
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// §3.1 / §5-1: 参照中Definitionの削除はReject
// ---------------------------------------------------------------------------

#[test]
fn delete_definition_while_used_is_rejected() {
    let s = shared_fixture();
    let before = s.doc.clone();
    let def = s.doc.effect_definition(s.d1).cloned().unwrap();
    let cmd = Command::DeleteEffectDefinition { definition: def };
    let mut working = s.doc.clone();
    let err = cmd.apply(&mut working).expect_err("must reject while used");
    assert_eq!(
        err,
        CommandError::DefinitionInUse {
            id: s.d1.get(),
            use_count: 3,
        }
    );
    // Document不変(Reject — §3.1)。
    assert_eq!(working, before);
}

// ---------------------------------------------------------------------------
// §3.2 / §5-2: 1つのUseをUnlink(RemoveEffect)。Definition・他Useは触らない
// ---------------------------------------------------------------------------

#[test]
fn unlink_one_use_keeps_definition_and_other_uses() {
    let s = shared_fixture();
    let use_ = s.doc.find_effect_use(s.layer_a, s.u2).unwrap().clone();
    let def = s.doc.effect_definition(s.d1).cloned().unwrap();
    let effect = motolii_doc::EffectInstance::from_use_and_definition(&use_, &def);
    let cmd = Command::RemoveEffect {
        target: s.layer_a,
        index: 1,
        effect,
    };
    let mut working = s.doc.clone();
    cmd.apply(&mut working).expect("unlink must succeed");

    let remaining = effects_on(&working, s.layer_a);
    assert_eq!(
        remaining,
        vec![EffectUse {
            id: s.u1,
            definition_id: s.d1
        }]
    );
    // 他layerのUseとDefinitionは無傷。
    assert_eq!(
        effects_on(&working, s.layer_b),
        vec![EffectUse {
            id: s.u3,
            definition_id: s.d1
        }]
    );
    assert_eq!(working.effect_definition(s.d1), Some(&def));

    // §4-5: Undo 1回で復元(同一index・同一definition_id)。
    let restored = cmd.inverse();
    cmd.inverse();
    let mut undone = working.clone();
    restored.apply(&mut undone).expect("undo must succeed");
    assert_eq!(undone.tracks, s.doc.tracks);
}

// ---------------------------------------------------------------------------
// §3.3 / §5-3: Copy Local(Materialize)。対象Useだけ付け替え、extraも複製
// ---------------------------------------------------------------------------

#[test]
fn copy_local_retargets_only_that_use_and_preserves_extra() {
    let mut s = shared_fixture();
    let new_def_id = EffectDefinitionId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    let cmd = Command::CopyLocalEffect {
        target: s.layer_b,
        use_id: s.u3,
        old_definition_id: s.d1,
        new_definition_id: new_def_id,
    };
    let mut working = s.doc.clone();
    cmd.apply(&mut working).expect("copy local must succeed");

    // U3だけが新Definitionへ付け替わる。U1/U2はD1のまま。
    assert_eq!(
        effects_on(&working, s.layer_b),
        vec![EffectUse {
            id: s.u3,
            definition_id: new_def_id
        }]
    );
    assert_eq!(
        effects_on(&working, s.layer_a),
        vec![
            EffectUse {
                id: s.u1,
                definition_id: s.d1
            },
            EffectUse {
                id: s.u2,
                definition_id: s.d1
            },
        ]
    );
    let original_def = s.doc.effect_definition(s.d1).cloned().unwrap();
    let copied_def = working.effect_definition(new_def_id).cloned().unwrap();
    assert_eq!(copied_def.plugin_id, original_def.plugin_id);
    assert_eq!(copied_def.params, original_def.params);
    // 未知plugin extraをbyte同等で複製(§2.6)。
    assert_eq!(copied_def.extra, original_def.extra);
    assert_eq!(working.effect_definition(s.d1), Some(&original_def));

    working
        .validate()
        .expect("post copy-local doc must validate");

    // Undo 1回で復元。
    let undo = cmd.inverse();
    let before_copy = s.doc.clone();
    undo.apply(&mut working).expect("undo must succeed");
    assert_eq!(working, before_copy);
    let _ = s.doc.effect_definitions.len();
}

#[test]
fn undo_copy_local_keeps_definition_if_other_uses_remain() {
    // Bugbot: UndoCopyLocalが共有中のnew definitionを落とすとdanglingになる。
    let mut s = shared_fixture();
    let new_def_id = EffectDefinitionId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    let copy = Command::CopyLocalEffect {
        target: s.layer_b,
        use_id: s.u3,
        old_definition_id: s.d1,
        new_definition_id: new_def_id,
    };
    let mut working = s.doc.clone();
    copy.apply(&mut working).unwrap();

    let u4 = EffectId::from_raw(working.next_stable_id.allocate().unwrap());
    let def = working.effect_definition(new_def_id).unwrap().clone();
    Command::AddEffect {
        target: s.layer_a,
        index: 2,
        effect: EffectInstance::from_use_and_definition(
            &EffectUse {
                id: u4,
                definition_id: new_def_id,
            },
            &def,
        ),
    }
    .apply(&mut working)
    .unwrap();

    copy.inverse().apply(&mut working).unwrap();
    assert_eq!(
        working
            .find_effect_use(s.layer_b, s.u3)
            .unwrap()
            .definition_id,
        s.d1
    );
    assert!(working.effect_definition(new_def_id).is_some());
    assert_eq!(
        working
            .find_effect_use(s.layer_a, u4)
            .unwrap()
            .definition_id,
        new_def_id
    );
    working.validate().unwrap();
}

// ---------------------------------------------------------------------------
// §3.4 / §5-4: 最後のUseを削除してもDefinitionはorphanとして残す
// ---------------------------------------------------------------------------

#[test]
fn unlink_last_use_keeps_orphan_definition() {
    let s = shared_fixture();
    let mut working = s.doc.clone();

    // U1, U2, U3を順に外す(最後の参照まで)。
    for (layer, use_id, index) in [
        (s.layer_a, s.u1, 0usize),
        (s.layer_a, s.u2, 0usize),
        (s.layer_b, s.u3, 0usize),
    ] {
        let use_ = working.find_effect_use(layer, use_id).unwrap().clone();
        let def = working
            .effect_definition(use_.definition_id)
            .cloned()
            .unwrap();
        let effect = motolii_doc::EffectInstance::from_use_and_definition(&use_, &def);
        Command::RemoveEffect {
            target: layer,
            index,
            effect,
        }
        .apply(&mut working)
        .expect("unlink must succeed");
    }

    assert!(effects_on(&working, s.layer_a).is_empty());
    assert!(effects_on(&working, s.layer_b).is_empty());
    // D1はOrphanKeep — 台帳に残る。
    assert_eq!(working.effect_use_count(s.d1), 0);
    assert!(working.effect_definition(s.d1).is_some());
    working.validate().expect("orphan document must validate");
}

// ---------------------------------------------------------------------------
// §3.5 / §5-5: Orphan Definitionの削除→Undoで同一ID・同一fieldsが戻る
// ---------------------------------------------------------------------------

#[test]
fn delete_orphan_definition_then_undo_restores_same_id_and_fields() {
    let s = shared_fixture();
    let before = s.doc.clone();
    assert_eq!(s.doc.effect_use_count(s.d2_orphan), 0);

    let mut writer = DocumentWriter::new(s.doc.clone());
    let gesture = writer.begin_gesture();
    let def = writer
        .snapshot()
        .effect_definition(s.d2_orphan)
        .cloned()
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::DeleteEffectDefinition {
                definition: def.clone(),
            },
        )
        .expect("delete unused definition must succeed");
    assert!(writer.snapshot().effect_definition(s.d2_orphan).is_none());
    // §4-8: 1 gesture = 1 undo。
    assert_eq!(writer.undo_len(), 1);

    writer.undo().expect("undo must succeed");
    assert_eq!(writer.snapshot().as_ref(), &before);
    let restored = writer.snapshot().effect_definition(s.d2_orphan).cloned();
    assert_eq!(restored, Some(def));
}

// ---------------------------------------------------------------------------
// §3.6 / §5-6: orphan Definitionはsave/reloadで同一ID・同一fieldで残る
// ---------------------------------------------------------------------------

#[test]
fn orphan_definition_survives_save_reload() {
    let s = shared_fixture();
    let dir = unique_dir("orphan-reload");
    let path = dir.join("doc.json");
    save_document(&path, &s.doc).unwrap();
    let reloaded = load_document(&path).unwrap();
    assert_eq!(reloaded, s.doc);
    assert_eq!(reloaded.effect_use_count(s.d2_orphan), 0);
    assert!(reloaded.effect_definition(s.d2_orphan).is_some());
    reloaded.validate().expect("reloaded doc must validate");
    let _ = fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------
// §3.3/§3.6 / §5-7: Copy Local後もsave/reloadで新旧2つのDefinitionが残る
// ---------------------------------------------------------------------------

#[test]
fn copy_local_then_save_reload_preserves_two_definitions() {
    let mut s = shared_fixture();
    let new_def_id = EffectDefinitionId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    Command::CopyLocalEffect {
        target: s.layer_b,
        use_id: s.u3,
        old_definition_id: s.d1,
        new_definition_id: new_def_id,
    }
    .apply(&mut s.doc)
    .expect("copy local must succeed");
    s.doc.validate().expect("post copy-local doc must validate");

    let dir = unique_dir("copy-local-reload");
    let path = dir.join("doc.json");
    save_document(&path, &s.doc).unwrap();
    let reloaded = load_document(&path).unwrap();
    assert_eq!(reloaded, s.doc);
    assert!(reloaded.effect_definition(s.d1).is_some());
    assert!(reloaded.effect_definition(new_def_id).is_some());
    assert_eq!(
        effects_on(&reloaded, s.layer_b),
        vec![EffectUse {
            id: s.u3,
            definition_id: new_def_id
        }]
    );
    let _ = fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------
// §4-8 / §5-8: Delete unused / Unlink / Copy Localはそれぞれ1 gesture = 1 undo
// ---------------------------------------------------------------------------

#[test]
fn delete_definition_unused_is_one_undo() {
    let s = shared_fixture();
    let mut writer = DocumentWriter::new(s.doc.clone());
    let gesture = writer.begin_gesture();
    let def = writer
        .snapshot()
        .effect_definition(s.d2_orphan)
        .cloned()
        .unwrap();
    writer
        .apply_command(gesture, Command::DeleteEffectDefinition { definition: def })
        .unwrap();
    assert_eq!(writer.undo_len(), 1);
    assert_eq!(writer.redo_len(), 0);
    writer.undo().unwrap();
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 1);
}

#[test]
fn unlink_is_one_undo() {
    let s = shared_fixture();
    let mut writer = DocumentWriter::new(s.doc.clone());
    let gesture = writer.begin_gesture();
    let use_ = writer.find_envelope(s.layer_a).unwrap().effects[1].clone();
    let def = writer
        .snapshot()
        .effect_definition(use_.definition_id)
        .cloned()
        .unwrap();
    let effect = motolii_doc::EffectInstance::from_use_and_definition(&use_, &def);
    writer
        .apply_command(
            gesture,
            Command::RemoveEffect {
                target: s.layer_a,
                index: 1,
                effect,
            },
        )
        .unwrap();
    assert_eq!(writer.undo_len(), 1);
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &s.doc);
}

#[test]
fn copy_local_is_one_undo() {
    let mut s = shared_fixture();
    let new_def_id = EffectDefinitionId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    let mut writer = DocumentWriter::new(s.doc.clone());
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::CopyLocalEffect {
                target: s.layer_b,
                use_id: s.u3,
                old_definition_id: s.d1,
                new_definition_id: new_def_id,
            },
        )
        .unwrap();
    assert_eq!(writer.undo_len(), 1);
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &s.doc);
}

// ---------------------------------------------------------------------------
// §4-1: 参照整合 — dangling / 重複IDは型付きエラー(黙ってdropしない)
// ---------------------------------------------------------------------------

#[test]
fn dangling_effect_use_definition_is_typed_error() {
    let mut s = shared_fixture();
    // 存在しないdefinition_idを指すUseを注入する。
    let dangling_def = EffectDefinitionId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    let TrackItem::Clip(clip) = &mut s.doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    let dangling_use_id = EffectId::from_raw(s.doc.next_stable_id.allocate().unwrap());
    clip.envelope.effects.push(EffectUse {
        id: dangling_use_id,
        definition_id: dangling_def,
    });
    let err = s.doc.validate().expect_err("dangling ref must be rejected");
    assert_eq!(
        err,
        DocumentError::DanglingEffectDefinition {
            layer_id: s.layer_a.get(),
            id: dangling_use_id.get(),
            definition_id: dangling_def.get(),
        }
    );
}

#[test]
fn duplicate_effect_definition_id_is_typed_error() {
    let mut s = shared_fixture();
    // d2_orphanと同じIDでもう1件definitionを追加する(id空間の一意性違反)。
    let dup = s.doc.effect_definition(s.d2_orphan).cloned().unwrap();
    s.doc.effect_definitions.push(dup);
    let err = s
        .doc
        .validate()
        .expect_err("duplicate definition id must be rejected");
    assert_eq!(
        err,
        DocumentError::DuplicateStableId {
            id: s.d2_orphan.get()
        }
    );
}

// ---------------------------------------------------------------------------
// migration: 旧inline EffectInstance → 1 Definition + 1 Use(1:1、extra保持)
// ---------------------------------------------------------------------------

#[test]
fn migration_inline_effect_becomes_one_definition_and_one_use_preserving_extra() {
    let raw = json!({
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
                    "effects": [{
                        "id": 5,
                        "plugin_id": "vendor.unknown.glow",
                        "effect_version": 2,
                        "enabled": false,
                        "params": {"amount": {"const": {"F64": 0.75}}},
                        "vendor_custom": "keep-me"
                    }],
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
        }]}
    });
    let bytes = serde_json::to_vec(&raw).unwrap();
    let (doc, report) = migrate_bytes(&bytes).expect("migration must succeed");
    assert!(report.did_migrate());
    assert!(report.steps.contains(&"inline_effects_to_definition_use"));

    // 1:1: 共有ゼロ。ちょうど1つのDefinitionと1つのUseが生まれる(§4-9)。
    assert_eq!(doc.effect_definitions.len(), 1);
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    assert_eq!(clip.envelope.effects.len(), 1);
    let use_ = &clip.envelope.effects[0];
    assert_eq!(use_.id.get(), 5);
    let def = doc.effect_definition(use_.definition_id).unwrap();
    assert_eq!(def.plugin_id, "vendor.unknown.glow");
    assert_eq!(def.effect_version, 2);
    assert!(!def.enabled);
    assert_eq!(def.params.get("amount"), Some(&DocParam::const_f64(0.75)));
    // 未知フィールドはDefinition側の`extra`として保持(F-9)。
    assert_eq!(
        def.extra.get("vendor_custom"),
        Some(&JsonValue::String("keep-me".into()))
    );

    doc.validate().expect("migrated document must validate");
    assert!(doc.min_reader_version >= motolii_doc::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS);

    // idempotent: 再migrateしても差分なし(同一形へ到達済み)。
    let reserialized = serde_json::to_vec(&doc).unwrap();
    let (doc2, report2) = migrate_bytes(&reserialized).expect("second migration must succeed");
    assert!(!report2.did_migrate());
    assert_eq!(doc2, doc);
}
