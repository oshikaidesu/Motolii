//! D2: コマンドシステム(apply/revert)・安定ID addressing・gesture merge・複製再写像の
//! 完了条件を機械判定する。
//!
//! - 全editコマンドの `apply` → `inverse().apply` が元状態と一致する(実装ガード5の対称設計)
//! - `EffectId`/`KeyframeId`(A8)の一意性・addressing(`get_by_id`)
//! - 1 gesture = 1 macro のmerge(#103⑨、merge key=S18)。undo/redoはmacro単位
//! - duplicate時: subtree内参照は新ID再写像、外向き参照は維持

use std::collections::BTreeMap;

use motolii_core::RationalTime;
use motolii_doc::{
    layer_names_for_item, BlendMode, Clip, ClipSource, ClippingMaskSettings, Command, DocKeyframe,
    DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter, EffectId, EffectInstance,
    Group, ItemEnvelope, KeyframeId, LayerId, LookAtAxis, MaskMode, ParentLocator,
    ScalarPropertyId, Track, TrackId, TrackItem,
};
use motolii_eval::Interp;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// フィクスチャ
// ---------------------------------------------------------------------------

struct Fixture {
    doc: Document,
    layer: LayerId,
    other_layer: LayerId,
    effect: EffectId,
    track: TrackId,
}

/// 1 effect(paramあり)を持つlayer + 参照先になる別layerを持つ最小文書。
fn fixture() -> Fixture {
    let mut doc = Document::new_v1();
    let layer = doc.layers.allocate("a").unwrap();
    let other_layer = doc.layers.allocate("b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let effect = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());

    let mut env = ItemEnvelope::new(layer);
    env.effects.push(EffectInstance {
        id: effect,
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        extra: Default::default(),
    });
    // stable id(effect)を含むため(M2E-11①)。version自体もこのテスト文書専用に2へ
    // (`Document::new_v1()`の既定はversion=1のまま — 他テストの前提を変えない)。
    doc.version = 2;
    doc.min_reader_version = 2;

    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: env,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::Asset { asset },
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(other_layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::Asset { asset },
            }),
        ],
    });
    doc.validate().expect("fixture must be valid");
    Fixture {
        doc,
        layer,
        other_layer,
        effect,
        track,
    }
}

/// `cmd`を適用→`inverse()`を適用した結果が元の`doc`と一致することを確認する
/// (実装ガード5: commandは決定済みの値を記録するのでapply/inverseは対称)。
///
/// 呼び出し側は`doc`の実際の現在値と`cmd`の`old_value`/`old`が一致するように
/// 準備すること — commandは「意図」でなく「決定済みの値」を記録するので、
/// old側が現在値と噛み合っていないケースはそもそも実際のUI操作では発生しない。
fn assert_roundtrip(doc: &Document, cmd: Command) {
    let mut working = doc.clone();
    cmd.apply(&mut working).expect("apply must succeed");
    cmd.inverse()
        .apply(&mut working)
        .expect("inverse apply must succeed");
    assert_eq!(&working, doc, "apply -> revert must restore original state");
}

/// `doc`を複製し、`f`で実際の現在値を`old`側に揃えてから返す。
fn prepare(doc: &Document, f: impl FnOnce(&mut Document)) -> Document {
    let mut d = doc.clone();
    f(&mut d);
    d
}

// ---------------------------------------------------------------------------
// 完了条件1: 全editコマンドのapply->revert->状態一致 property test
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn set_property_position_roundtrip(x in -1000.0f64..1000.0, y in -1000.0f64..1000.0) {
        let f = fixture();
        let cmd = Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Position,
            old_value: DocParam::const_vec2([0.0, 0.0]),
            new_value: DocParam::const_vec2([x, y]),
        };
        assert_roundtrip(&f.doc, cmd);
    }

    #[test]
    fn set_property_opacity_roundtrip(old in 0.0f64..=1.0, new in 0.0f64..=1.0) {
        let f = fixture();
        let doc = prepare(&f.doc, |d| {
            let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else { panic!("expected clip") };
            c.envelope.opacity = DocParam::const_f64(old);
        });
        let cmd = Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(old),
            new_value: DocParam::const_f64(new),
        };
        assert_roundtrip(&doc, cmd);
    }

    #[test]
    fn set_property_effect_param_roundtrip(old in -10.0f64..10.0, new in -10.0f64..10.0) {
        let f = fixture();
        let doc = prepare(&f.doc, |d| {
            let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else { panic!("expected clip") };
            c.envelope.effects[0]
                .params
                .insert("amount".into(), DocParam::const_f64(old));
        });
        let cmd = Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::EffectParam(f.effect, "amount".into()),
            old_value: DocParam::const_f64(old),
            new_value: DocParam::const_f64(new),
        };
        assert_roundtrip(&doc, cmd);
    }

    #[test]
    fn set_blend_mode_roundtrip(old_idx in 0usize..3, new_idx in 0usize..3) {
        let f = fixture();
        let modes = [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply];
        let doc = prepare(&f.doc, |d| {
            let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else { panic!("expected clip") };
            c.envelope.blend = modes[old_idx];
        });
        let cmd = Command::SetBlendMode {
            target: f.layer,
            old: modes[old_idx],
            new: modes[new_idx],
        };
        assert_roundtrip(&doc, cmd);
    }

    #[test]
    fn set_clipping_mask_roundtrip(old_enabled in any::<bool>(), new_enabled in any::<bool>()) {
        let f = fixture();
        let old = ClippingMaskSettings { enabled: old_enabled, mode: MaskMode::Alpha };
        let doc = prepare(&f.doc, |d| {
            let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else { panic!("expected clip") };
            c.envelope.clipping_mask = old.clone();
        });
        let cmd = Command::SetClippingMask {
            target: f.layer,
            old,
            new: ClippingMaskSettings { enabled: new_enabled, mode: MaskMode::Luminance },
        };
        assert_roundtrip(&doc, cmd);
    }

    #[test]
    fn set_transform_parent_roundtrip(set_new in any::<bool>()) {
        let f = fixture();
        let cmd = Command::SetTransformParent {
            target: f.layer,
            old: None,
            new: if set_new { Some(f.other_layer) } else { None },
        };
        assert_roundtrip(&f.doc, cmd);
    }

    #[test]
    fn set_effect_enabled_roundtrip(old in any::<bool>(), new in any::<bool>()) {
        let f = fixture();
        let doc = prepare(&f.doc, |d| {
            let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else { panic!("expected clip") };
            c.envelope.effects[0].enabled = old;
        });
        let cmd = Command::SetEffectEnabled {
            target: f.layer,
            effect: f.effect,
            old,
            new,
        };
        assert_roundtrip(&doc, cmd);
    }

    #[test]
    fn add_remove_effect_roundtrip(enabled in any::<bool>(), amount in -5.0f64..5.0) {
        let f = fixture();
        let new_effect_id = EffectId::from_raw(f.doc.next_stable_id.peek_next());
        let effect = EffectInstance {
            id: new_effect_id,
            plugin_id: "core.filter.blur".into(),
            effect_version: 1,
            enabled,
            params: BTreeMap::from([("amount".into(), DocParam::const_f64(amount))]),
            extra: Default::default(),
        };
        let cmd = Command::AddEffect {
            target: f.layer,
            index: 1,
            effect,
        };
        assert_roundtrip(&f.doc, cmd);
    }

    #[test]
    fn remove_effect_roundtrip(_seed in any::<bool>()) {
        let f = fixture();
        let TrackItem::Clip(clip) = &f.doc.tracks[0].items[0] else {
            panic!("expected fixture clip at index 0");
        };
        let effect = clip.envelope.effects[0].clone();
        let cmd = Command::RemoveEffect {
            target: f.layer,
            index: 0,
            effect,
        };
        assert_roundtrip(&f.doc, cmd);
    }

    #[test]
    fn add_remove_track_item_roundtrip(start in 0i64..4) {
        let mut f = fixture();
        // エントリ無しでIDだけ予約 — applyが台帳へ載せ、inverseが外すので Document 全体が戻る。
        let new_layer = f.doc.layers.reserve().unwrap();
        let layer_names = BTreeMap::from([(new_layer, "new".to_string())]);
        let item = TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(new_layer),
            start: RationalTime::try_new(start, 1).unwrap(),
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::Asset {
                asset: motolii_doc::AssetId::from_raw(0),
            },
        });
        let cmd = Command::AddTrackItem {
            parent: ParentLocator::Track(f.track),
            index: 2,
            item,
            layer_names,
        };
        assert_roundtrip(&f.doc, cmd);
    }
}

#[test]
fn remove_track_item_roundtrip() {
    let f = fixture();
    let item = f.doc.tracks[0].items[1].clone();
    let layer_names = layer_names_for_item(&f.doc, &item).unwrap();
    let cmd = Command::RemoveTrackItem {
        parent: ParentLocator::Track(f.track),
        index: 1,
        item,
        layer_names,
    };
    assert_roundtrip(&f.doc, cmd);
}

#[test]
fn add_effect_rejects_index_past_end() {
    let f = fixture();
    let before = f.doc.clone();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    let effect = EffectInstance {
        id: EffectId::from_raw(writer.snapshot().next_stable_id.peek_next()),
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let err = writer
        .apply_command(
            gesture,
            Command::AddEffect {
                target: f.layer,
                index: 99,
                effect,
            },
        )
        .expect_err("index past end");
    assert!(matches!(
        err,
        motolii_doc::CommandError::IndexOutOfRange { index: 99, len: 1 }
    ));
    assert_eq!(writer.snapshot().as_ref(), &before);
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 0);
}

#[test]
fn add_track_item_rejects_index_past_end() {
    let mut f = fixture();
    let new_layer = f.doc.layers.reserve().unwrap();
    let before = f.doc.clone();
    let mut writer = DocumentWriter::new(f.doc);
    let gesture = writer.begin_gesture();
    let layer_names = BTreeMap::from([(new_layer, "x".to_string())]);
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(new_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::Asset {
            asset: motolii_doc::AssetId::from_raw(0),
        },
    });
    let err = writer
        .apply_command(
            gesture,
            Command::AddTrackItem {
                parent: ParentLocator::Track(f.track),
                index: 99,
                item,
                layer_names,
            },
        )
        .expect_err("index past end");
    assert!(matches!(
        err,
        motolii_doc::CommandError::IndexOutOfRange { index: 99, len: 2 }
    ));
    assert_eq!(writer.snapshot().as_ref(), &before);
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 0);
}

/// 台帳エントリ(ID→表示名)を比較用に取り出す。`next`は含めない。
fn layer_entries(doc: &Document) -> BTreeMap<u64, String> {
    doc.layers
        .iter()
        .map(|(id, name)| (id.get(), name.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// 完了条件2: 安定ID addressing(A8)
// ---------------------------------------------------------------------------

#[test]
fn effect_and_keyframe_ids_never_repeat_and_are_addressable() {
    let mut doc = Document::new_v1();
    let a = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let b = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let c = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    assert_ne!(a.get(), b.get());
    assert_ne!(b.get(), c.get());
    assert_ne!(a.get(), c.get());

    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: b,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    assert!(track.get_by_id(b).is_some());
    assert!(track.get_by_id(KeyframeId::from_raw(999)).is_none());
    let removed = track.remove_by_id(b);
    assert_eq!(removed.map(|k| k.id), Some(b));
    assert!(track.get_by_id(b).is_none());
}

#[test]
fn duplicate_track_item_allocates_fresh_ids_via_writer() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc);
    let before_next = writer.snapshot().next_stable_id.peek_next();
    writer.duplicate_track_item(f.layer).expect("duplicate");
    let after_next = writer.snapshot().next_stable_id.peek_next();
    assert!(
        after_next > before_next,
        "duplication must mint fresh stable ids"
    );
    writer
        .validate()
        .expect("duplicated document must validate");
}

// ---------------------------------------------------------------------------
// 完了条件3: gesture merge(#103⑨、merge key=S18)
// ---------------------------------------------------------------------------

#[test]
fn same_gesture_drag_merges_into_one_macro_and_undoes_atomically() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc.clone());
    let gesture = writer.begin_gesture();

    // 「ドラッグ中」の3ステップ: 決定済みの値を都度記録するが、同一merge keyなので1つに畳まれる。
    for x in [10.0, 20.0, 30.0] {
        writer
            .apply_command(
                gesture,
                Command::SetProperty {
                    target: f.layer,
                    property: ScalarPropertyId::Position,
                    old_value: DocParam::const_vec2([0.0, 0.0]),
                    new_value: DocParam::const_vec2([x, 0.0]),
                },
            )
            .expect("apply_command");
    }
    assert_eq!(
        writer.undo_len(),
        1,
        "same gesture must merge into a single macro"
    );

    let snap = writer.snapshot();
    let TrackItem::Clip(clip) = &snap.tracks[0].items[0] else {
        panic!("expected fixture clip at index 0");
    };
    assert_eq!(
        clip.envelope.transform.position,
        DocParam::const_vec2([30.0, 0.0])
    );

    writer.undo().expect("undo");
    assert_eq!(writer.snapshot(), std::sync::Arc::new(f.doc.clone()));
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 1);

    writer.redo().expect("redo");
    assert_eq!(writer.undo_len(), 1);
}

#[test]
fn different_gestures_do_not_merge() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc.clone());

    let g1 = writer.begin_gesture();
    writer
        .apply_command(
            g1,
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::Opacity,
                old_value: DocParam::const_f64(1.0),
                new_value: DocParam::const_f64(0.5),
            },
        )
        .unwrap();

    let g2 = writer.begin_gesture();
    writer
        .apply_command(
            g2,
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::Opacity,
                old_value: DocParam::const_f64(0.5),
                new_value: DocParam::const_f64(0.2),
            },
        )
        .unwrap();

    assert_eq!(writer.undo_len(), 2, "distinct gestures must not merge");
    writer.undo().unwrap();
    writer.undo().unwrap();
    assert_eq!(writer.snapshot(), std::sync::Arc::new(f.doc.clone()));
}

#[test]
fn same_gesture_two_add_effects_do_not_merge() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc.clone());
    let gesture = writer.begin_gesture();

    let effect1_id = writer.allocate_effect_id().expect("allocate effect1");
    let effect2_id = writer.allocate_effect_id().expect("allocate effect2");
    let effect1 = EffectInstance {
        id: effect1_id,
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let effect2 = EffectInstance {
        id: effect2_id,
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };

    writer
        .apply_command(
            gesture,
            Command::AddEffect {
                target: f.layer,
                index: 1,
                effect: effect1,
            },
        )
        .expect("add effect1");
    writer
        .apply_command(
            gesture,
            Command::AddEffect {
                target: f.layer,
                index: 2,
                effect: effect2,
            },
        )
        .expect("add effect2");

    let snap = writer.snapshot();
    let TrackItem::Clip(clip) = &snap.tracks[0].items[0] else {
        panic!("expected fixture clip at index 0");
    };
    assert_eq!(
        clip.envelope.effects.len(),
        3,
        "distinct effect ids must not merge: both AddEffects must apply"
    );
    assert_eq!(writer.undo_len(), 1, "same gesture still forms one macro");
}

#[test]
fn same_gesture_two_add_effects_undo_removes_both() {
    let f = fixture();
    let mut writer = DocumentWriter::new(f.doc.clone());
    let gesture = writer.begin_gesture();

    let effect1_id = writer.allocate_effect_id().expect("allocate effect1");
    let effect2_id = writer.allocate_effect_id().expect("allocate effect2");
    let effect1 = EffectInstance {
        id: effect1_id,
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let effect2 = EffectInstance {
        id: effect2_id,
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };

    writer
        .apply_command(
            gesture,
            Command::AddEffect {
                target: f.layer,
                index: 1,
                effect: effect1,
            },
        )
        .expect("add effect1");
    writer
        .apply_command(
            gesture,
            Command::AddEffect {
                target: f.layer,
                index: 2,
                effect: effect2,
            },
        )
        .expect("add effect2");

    let snap = writer.snapshot();
    let TrackItem::Clip(clip) = &snap.tracks[0].items[0] else {
        panic!("expected fixture clip at index 0");
    };
    assert_eq!(clip.envelope.effects.len(), 3);

    writer
        .undo()
        .expect("undo gesture removes both added effects");
    let after_undo = writer.snapshot();
    let TrackItem::Clip(clip) = &after_undo.tracks[0].items[0] else {
        panic!("expected fixture clip at index 0");
    };
    assert_eq!(clip.envelope.effects.len(), 1);
    assert_eq!(clip.envelope.effects[0].id, f.effect);
    assert_eq!(
        after_undo.tracks, f.doc.tracks,
        "tree content must match pre-edit state"
    );
}

// ---------------------------------------------------------------------------
// 完了条件4: duplicate/paste時のID再写像(subtree内=新規、外向き=維持)
// ---------------------------------------------------------------------------

#[test]
fn duplicate_remaps_internal_refs_and_preserves_external_refs() {
    let mut doc = Document::new_v1();
    let external_layer = doc.layers.allocate("external").unwrap();
    let group_layer = doc.layers.allocate("group").unwrap();
    let child_a = doc.layers.allocate("child_a").unwrap();
    let child_b = doc.layers.allocate("child_b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

    let mut env_a = ItemEnvelope::new(child_a);
    // subtree内参照(sibling child_b) — 複製後は新IDへ再写像されるべき。
    env_a.transform.position = DocParam::LookAt {
        target: child_b,
        axis: LookAtAxis::PlusY,
    };
    let keyframe_id = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut opacity_track = DocKeyframeTrack::new();
    opacity_track.insert(DocKeyframe {
        id: keyframe_id,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    env_a.opacity = DocParam::Keyframes(opacity_track);

    let mut env_b = ItemEnvelope::new(child_b);
    // subtree外参照(external_layer) — 複製後も維持されるべき。
    // (rotationはLookAt/Followを許可しない — positionのみ許可。d1h_validate::look_at_on_rotation_fails参照)
    env_b.transform.position = DocParam::LookAt {
        target: external_layer,
        axis: LookAtAxis::PlusY,
    };

    let mut group_env = ItemEnvelope::new(group_layer);
    let effect_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    group_env.effects.push(EffectInstance {
        id: effect_id,
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    });
    doc.version = 2;
    doc.min_reader_version = 2;

    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(external_layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::Asset { asset },
            }),
            TrackItem::Group(Group {
                envelope: group_env,
                children: vec![
                    TrackItem::Clip(Clip {
                        envelope: env_a,
                        start: RationalTime::ZERO,
                        duration: RationalTime::try_new(2, 1).unwrap(),
                        time_map: Default::default(),
                        source: ClipSource::Asset { asset },
                    }),
                    TrackItem::Clip(Clip {
                        envelope: env_b,
                        start: RationalTime::ZERO,
                        duration: RationalTime::try_new(2, 1).unwrap(),
                        time_map: Default::default(),
                        source: ClipSource::Asset { asset },
                    }),
                ],
            }),
        ],
    });
    doc.validate().expect("fixture must validate");

    let mut writer = DocumentWriter::new(doc.clone());
    writer
        .duplicate_track_item(group_layer)
        .expect("duplicate group");
    writer
        .validate()
        .expect("post-duplicate document must validate");

    let snap = writer.snapshot();
    assert_eq!(
        snap.tracks[0].items.len(),
        3,
        "duplicate inserts right after source"
    );

    let TrackItem::Group(original_group) = &snap.tracks[0].items[1] else {
        panic!("expected original group at index 1");
    };
    let TrackItem::Group(cloned_group) = &snap.tracks[0].items[2] else {
        panic!("expected cloned group at index 2");
    };

    assert_ne!(
        cloned_group.envelope.layer_id,
        original_group.envelope.layer_id
    );
    assert_ne!(
        cloned_group.envelope.effects[0].id, original_group.envelope.effects[0].id,
        "effect id must be freshly minted, not reused"
    );

    let TrackItem::Clip(cloned_a) = &cloned_group.children[0] else {
        panic!("expected clip child_a clone");
    };
    let TrackItem::Clip(cloned_b) = &cloned_group.children[1] else {
        panic!("expected clip child_b clone");
    };

    // subtree内参照は複製先の新IDへ再写像される。
    match &cloned_a.envelope.transform.position {
        DocParam::LookAt { target, .. } => {
            assert_eq!(*target, cloned_b.envelope.layer_id);
            assert_ne!(
                *target, child_b,
                "internal ref must not still point at the original"
            );
        }
        other => panic!("expected LookAt, got {other:?}"),
    }

    // subtree外参照は維持される。
    match &cloned_b.envelope.transform.position {
        DocParam::LookAt { target, .. } => {
            assert_eq!(
                *target, external_layer,
                "external ref must be preserved verbatim"
            );
        }
        other => panic!("expected LookAt, got {other:?}"),
    }

    // keyframeも複製先で新IDを持つ。
    match &cloned_a.envelope.opacity {
        DocParam::Keyframes(track) => {
            assert_eq!(track.keys().len(), 1);
            assert_ne!(track.keys()[0].id, keyframe_id);
        }
        other => panic!("expected Keyframes, got {other:?}"),
    }

    // 単一writer境界を保ったまま1回のundoで複製全体(1 gesture)が取り消せる。
    // LayerId/EffectId/KeyframeIdの採番カウンタは非再利用規律により巻き戻らない。
    // 台帳エントリ自体はRemoveで外れる — max_layersに孤児が溜まらない。
    let allocated_next = snap.next_stable_id.peek_next();
    let layers_before = layer_entries(&doc);
    let layers_after_dup = layer_entries(&snap);
    let duplicated: BTreeMap<u64, String> = layers_after_dup
        .iter()
        .filter(|(id, _)| !layers_before.contains_key(id))
        .map(|(id, name)| (*id, name.clone()))
        .collect();
    assert_eq!(
        duplicated.len(),
        3,
        "nested group duplicate must register group+2 children in LayerIdTable"
    );

    writer.undo().expect("undo duplicate");
    let after_undo = writer.snapshot();
    assert_eq!(
        after_undo.tracks, doc.tracks,
        "tree content must match pre-duplication state"
    );
    assert_eq!(
        layer_entries(&after_undo),
        layers_before,
        "undo must restore LayerIdTable entries (ids+names), not only tracks"
    );
    assert_eq!(
        after_undo.next_stable_id.peek_next(),
        allocated_next,
        "stable id counter must not be rewound by undo (non-reuse discipline)"
    );

    // redoで同じ既発行IDと表示名が復帰する(insertではなくrestore経路)。
    writer.redo().expect("redo duplicate");
    writer.validate().expect("post-redo document must validate");
    let after_redo = writer.snapshot();
    assert_eq!(
        layer_entries(&after_redo),
        layers_after_dup,
        "redo must restore the same LayerId entries and display names"
    );
    for (id, name) in &duplicated {
        assert_eq!(
            after_redo.layers.display_name(LayerId::from_raw(*id)),
            Some(name.as_str())
        );
    }
}

#[test]
fn duplicate_undo_redo_loop_does_not_grow_layer_table() {
    let mut doc = Document::new_v1();
    let group_layer = doc.layers.allocate("group").unwrap();
    let child_a = doc.layers.allocate("child_a").unwrap();
    let child_b = doc.layers.allocate("child_b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Group(Group {
            envelope: ItemEnvelope::new(group_layer),
            children: vec![
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(child_a),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::Asset { asset },
                }),
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(child_b),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::Asset { asset },
                }),
            ],
        })],
    });
    doc.validate().expect("fixture");

    let baseline = layer_entries(&doc);
    let mut writer = DocumentWriter::new(doc);
    for _ in 0..8 {
        writer
            .duplicate_track_item(group_layer)
            .expect("duplicate nested group");
        writer.undo().expect("undo duplicate");
        assert_eq!(
            layer_entries(&writer.snapshot()),
            baseline,
            "duplicate↔undo must not accumulate LayerIdTable orphans"
        );
    }
    assert_eq!(writer.snapshot().layers.len(), baseline.len());
}

#[test]
fn duplicate_remaps_plugin_lookat_within_subtree() {
    let mut doc = Document::new_v1();
    let group_layer = doc.layers.allocate("group").unwrap();
    let child_a = doc.layers.allocate("child_a").unwrap();
    let child_b = doc.layers.allocate("child_b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

    let plugin_params = BTreeMap::from([(
        "aim".into(),
        DocParam::LookAt {
            target: child_b,
            axis: LookAtAxis::PlusY,
        },
    )]);

    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Group(Group {
            envelope: ItemEnvelope::new(group_layer),
            children: vec![
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(child_a),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(2, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::Plugin {
                        plugin_id: "vendor.test.plugin".into(),
                        effect_version: 1,
                        params: plugin_params,
                        extra: Default::default(),
                    },
                }),
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(child_b),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(2, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::Asset { asset },
                }),
            ],
        })],
    });
    doc.validate().expect("fixture must validate");

    let mut writer = DocumentWriter::new(doc.clone());
    writer
        .duplicate_track_item(group_layer)
        .expect("duplicate group");

    let snap = writer.snapshot();
    let TrackItem::Group(cloned_group) = &snap.tracks[0].items[1] else {
        panic!("expected cloned group at index 1");
    };
    let TrackItem::Clip(cloned_a) = &cloned_group.children[0] else {
        panic!("expected plugin clip clone");
    };
    let TrackItem::Clip(cloned_b) = &cloned_group.children[1] else {
        panic!("expected sibling clip clone");
    };

    let ClipSource::Plugin { params, .. } = &cloned_a.source else {
        panic!("expected plugin source on cloned clip");
    };
    match params.get("aim").expect("aim param") {
        DocParam::LookAt { target, .. } => {
            assert_eq!(
                *target, cloned_b.envelope.layer_id,
                "plugin LookAt must remap to cloned sibling inside subtree"
            );
            assert_ne!(
                *target, child_b,
                "plugin LookAt must not still point at the original layer"
            );
        }
        other => panic!("expected LookAt, got {other:?}"),
    }
}
