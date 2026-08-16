//! D2: コマンドシステム(apply/revert)・安定ID addressing・gesture merge・複製再写像の
//! 完了条件を機械判定する。
//!
//! - 全editコマンドの `apply` → `inverse().apply` が元状態と一致する(実装ガード5の対称設計)
//! - `EffectId`/`KeyframeId`(A8)の一意性・addressing(`get_by_id`)
//! - 1 gesture = 1 macro のmerge(#103⑨、merge key=S18)。undo/redoはmacro単位
//! - duplicate時: subtree内参照は新ID再写像、外向き参照は維持

#![allow(deprecated)]

pub mod common;

use common::identity_roundtrip::assert_identity_command_roundtrip;

use std::collections::BTreeMap;
use std::sync::Arc;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    layer_names_for_item, BlendMode, Clip, ClipSource, ClippingMaskSettings, Command, CommandError,
    DocKeyframe, DocKeyframeError, DocKeyframeTrack, DocParam, DocValue, Document, DocumentWriter,
    EffectDefinition, EffectDefinitionId, EffectId, EffectInstance, EffectUse, Group, ItemEnvelope,
    KeyframeId, LayerId, LookAtAxis, MaskMode, ParentLocator, ScalarPropertyId,
    StableIdReservation, Track, TrackId, TrackItem,
};
use motolii_eval::Interp;
use motolii_plugin::reference::reference_catalog;

fn reference_writer(doc: Document) -> DocumentWriter {
    DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap()
}
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

// ---------------------------------------------------------------------------
// フィクスチャ
// ---------------------------------------------------------------------------

struct Fixture {
    doc: Document,
    layer: LayerId,
    other_layer: LayerId,
    effect: EffectId,
    effect_def: EffectDefinitionId,
    track: TrackId,
}

fn allocate_effect_ids_for_add_effect_test(
    doc: &mut Document,
) -> (EffectId, EffectDefinitionId, EffectId, EffectDefinitionId) {
    let effect1_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let effect1_def = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let effect2_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let effect2_def = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    (effect1_id, effect1_def, effect2_id, effect2_def)
}

/// 1 effect(paramあり)を持つlayer + 参照先になる別layerを持つ最小文書。
fn fixture() -> Fixture {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("a").unwrap();
    let other_layer = doc.layers.allocate("b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let effect = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let effect_def = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        effect_def,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        Default::default(),
    ));

    let mut env = ItemEnvelope::new(layer);
    env.effects.push(EffectUse {
        id: effect,
        definition_id: effect_def,
    });

    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: env,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(other_layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
        ],
    });
    doc.validate().expect("fixture must be valid");
    Fixture {
        doc,
        layer,
        other_layer,
        effect,
        effect_def,
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

fn plugin_clip_doc(f: &Fixture, params: BTreeMap<String, DocParam>) -> Document {
    prepare(&f.doc, |d| {
        let TrackItem::Clip(c) = &mut d.tracks[0].items[0] else {
            panic!("expected clip")
        };
        c.source = ClipSource::Plugin {
            plugin_id: "core.layer_source.radial_repeater".into(),
            effect_version: 1,
            params,
            extra: Default::default(),
        };
    })
}

fn plugin_params<'a>(doc: &'a Document, layer: LayerId) -> &'a BTreeMap<String, DocParam> {
    for track in &doc.tracks {
        for item in &track.items {
            if let TrackItem::Clip(clip) = item {
                if clip.envelope.layer_id == layer {
                    let ClipSource::Plugin { params, .. } = &clip.source else {
                        panic!("expected plugin source")
                    };
                    return params;
                }
            }
        }
    }
    panic!("layer not found")
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
            let def = d.effect_definition_mut(f.effect_def).expect("effect definition");
            def.params.insert("amount".into(), DocParam::const_f64(old));
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
    fn set_property_source_param_f64_roundtrip(old in -10.0f64..10.0, new in -10.0f64..10.0) {
        let f = fixture();
        let doc = plugin_clip_doc(&f, BTreeMap::from([("count".into(), DocParam::const_f64(old))]));
        let cmd = Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::SourceParam("count".into()),
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
            let def = d.effect_definition_mut(f.effect_def).expect("effect definition");
            def.enabled = old;
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
        let base = f.doc.next_stable_id.peek_next();
        let new_effect_id = EffectId::from_raw(base);
        let new_definition_id = EffectDefinitionId::from_raw(base + 1);
        let effect = EffectInstance {
            id: new_effect_id,
            definition_id: new_definition_id,
            plugin_id: "core.filter.blur".into(),
            effect_version: 1,
            enabled,
            params: BTreeMap::from([("amount".into(), DocParam::const_f64(amount))]),
            extra: Default::default(),
        };
        let cmd = Command::AddEffect {
            target: f.layer,
            index: 1,
            effect: effect.clone(),
            introduced_definition: true,
        };
        assert_roundtrip(&f.doc, cmd);
    }

    #[test]
    fn remove_effect_roundtrip(_seed in any::<bool>()) {
        let f = fixture();
        let TrackItem::Clip(clip) = &f.doc.tracks[0].items[0] else {
            panic!("expected fixture clip at index 0");
        };
        let use_ = clip.envelope.effects[0].clone();
        let def = f
            .doc
            .effect_definition(use_.definition_id)
            .expect("effect definition")
            .clone();
        let effect = EffectInstance::from_use_and_definition(&use_, &def);
        let cmd = Command::RemoveEffect {
            target: f.layer,
            index: 0,
            effect,
            introduced_definition: false,
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
            source: ClipSource::asset_video_only(motolii_doc::AssetId::from_raw(0)),
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
fn set_property_source_param_writes_color_without_f64_ceiling() {
    let f = fixture();
    let old = DocParam::const_color([1.0, 1.0, 1.0, 1.0]);
    let new = DocParam::const_color([0.2, 0.4, 0.6, 1.0]);
    let doc = plugin_clip_doc(&f, BTreeMap::from([("color".into(), old.clone())]));
    let cmd = Command::SetProperty {
        target: f.layer,
        property: ScalarPropertyId::SourceParam("color".into()),
        old_value: old,
        new_value: new.clone(),
    };
    let mut working = doc.clone();
    cmd.apply(&mut working).expect("color source param apply");
    assert_eq!(plugin_params(&working, f.layer).get("color"), Some(&new));
    cmd.inverse().apply(&mut working).expect("color inverse");
    assert_eq!(&working, &doc);
}

#[test]
fn set_property_source_param_rejects_missing_vector_and_unknown_key() {
    let f = fixture();
    let missing = Command::SetProperty {
        target: f.layer,
        property: ScalarPropertyId::SourceParam("count".into()),
        old_value: DocParam::const_f64(12.0),
        new_value: DocParam::const_f64(8.0),
    };
    let mut asset_doc = f.doc.clone();
    assert!(matches!(
        missing.apply(&mut asset_doc),
        Err(CommandError::SourceNotPlugin { layer }) if layer == f.layer.get()
    ));
    assert_eq!(asset_doc, f.doc);

    let plugin = plugin_clip_doc(
        &f,
        BTreeMap::from([("count".into(), DocParam::const_f64(12.0))]),
    );
    let mut working = plugin.clone();
    let unknown = Command::SetProperty {
        target: f.layer,
        property: ScalarPropertyId::SourceParam("missing".into()),
        old_value: DocParam::const_f64(0.0),
        new_value: DocParam::const_f64(1.0),
    };
    assert!(matches!(
        unknown.apply(&mut working),
        Err(CommandError::SourceParamNotFound { layer, param })
            if layer == f.layer.get() && param == "missing"
    ));
    assert_eq!(working, plugin);
}

// ---------------------------------------------------------------------------
// 再締結ゲート B.3: 固定seedの異種編集列(複数gesture×各複数command) Undo/Redo 審判
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum RandomEditSpec {
    Position { x: f64, y: f64 },
    Rotation { radians: f64 },
    Opacity { new: f64 },
    Blend { mode_idx: usize },
    ClippingMask { enabled: bool, mode_idx: usize },
    TransformParent { set_parent: bool },
    EffectEnabled { enabled: bool },
    EffectParam { amount: f64 },
}

fn position_edit_spec_strategy() -> impl Strategy<Value = RandomEditSpec> {
    (-1000.0f64..1000.0, -1000.0f64..1000.0).prop_map(|(x, y)| RandomEditSpec::Position { x, y })
}

fn blend_edit_spec_strategy() -> impl Strategy<Value = RandomEditSpec> {
    (0usize..3).prop_map(|mode_idx| RandomEditSpec::Blend { mode_idx })
}

fn random_edit_spec_strategy() -> impl Strategy<Value = RandomEditSpec> {
    prop_oneof![
        position_edit_spec_strategy(),
        (-10.0f64..10.0).prop_map(|radians| RandomEditSpec::Rotation { radians }),
        (0.0f64..=1.0).prop_map(|new| RandomEditSpec::Opacity { new }),
        blend_edit_spec_strategy(),
        (any::<bool>(), 0usize..4)
            .prop_map(|(enabled, mode_idx)| RandomEditSpec::ClippingMask { enabled, mode_idx }),
        any::<bool>().prop_map(|set_parent| RandomEditSpec::TransformParent { set_parent }),
        any::<bool>().prop_map(|enabled| RandomEditSpec::EffectEnabled { enabled }),
        (-10.0f64..10.0).prop_map(|amount| RandomEditSpec::EffectParam { amount }),
    ]
}

/// gesture 0: 必須 Position + 0..=5 任意 tail → 1..=6 command。
fn gesture_0_strategy() -> impl Strategy<Value = Vec<RandomEditSpec>> {
    (
        position_edit_spec_strategy(),
        prop::collection::vec(random_edit_spec_strategy(), 0..=5),
    )
        .prop_map(|(head, tail)| {
            let mut edits = vec![head];
            edits.extend(tail);
            edits
        })
}

/// gesture 1: 必須 Blend + 0..=5 任意 tail → 1..=6 command。
fn gesture_1_strategy() -> impl Strategy<Value = Vec<RandomEditSpec>> {
    (
        blend_edit_spec_strategy(),
        prop::collection::vec(random_edit_spec_strategy(), 0..=5),
    )
        .prop_map(|(head, tail)| {
            let mut edits = vec![head];
            edits.extend(tail);
            edits
        })
}

/// gesture 2..: 1..=6 任意 command。
fn extra_gesture_strategy() -> impl Strategy<Value = Vec<RandomEditSpec>> {
    prop::collection::vec(random_edit_spec_strategy(), 1..=6)
}

/// 2..=12 gesture。shrink 後も gesture 0=Position・gesture 1=Blend を構造的に保持する。
fn multi_gesture_sequence_strategy() -> impl Strategy<Value = Vec<Vec<RandomEditSpec>>> {
    (
        gesture_0_strategy(),
        gesture_1_strategy(),
        prop::collection::vec(extra_gesture_strategy(), 0..=10),
    )
        .prop_map(|(g0, g1, extras)| {
            let mut gestures = vec![g0, g1];
            gestures.extend(extras);
            gestures
        })
}

fn build_random_edit_command(
    writer: &DocumentWriter,
    f: &Fixture,
    spec: &RandomEditSpec,
) -> Command {
    let env = writer
        .find_envelope(f.layer)
        .expect("fixture layer must exist in writer");
    let snap = writer.snapshot();
    match spec {
        RandomEditSpec::Position { x, y } => Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Position,
            old_value: env.transform.position.clone(),
            new_value: DocParam::const_vec2([*x, *y]),
        },
        RandomEditSpec::Rotation { radians } => Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Rotation,
            old_value: env.transform.rotation.clone(),
            new_value: DocParam::const_f64(*radians),
        },
        RandomEditSpec::Opacity { new } => Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Opacity,
            old_value: env.opacity.clone(),
            new_value: DocParam::const_f64(*new),
        },
        RandomEditSpec::Blend { mode_idx } => {
            let modes = [BlendMode::Normal, BlendMode::Add, BlendMode::Multiply];
            Command::SetBlendMode {
                target: f.layer,
                old: env.blend,
                new: modes[*mode_idx % modes.len()],
            }
        }
        RandomEditSpec::ClippingMask { enabled, mode_idx } => {
            let modes = [
                MaskMode::Alpha,
                MaskMode::Luminance,
                MaskMode::InvertAlpha,
                MaskMode::InvertLuminance,
            ];
            Command::SetClippingMask {
                target: f.layer,
                old: env.clipping_mask.clone(),
                new: ClippingMaskSettings {
                    enabled: *enabled,
                    mode: modes[*mode_idx % modes.len()],
                },
            }
        }
        RandomEditSpec::TransformParent { set_parent } => Command::SetTransformParent {
            target: f.layer,
            old: env.transform.parent,
            new: if *set_parent {
                Some(f.other_layer)
            } else {
                None
            },
        },
        RandomEditSpec::EffectEnabled { enabled } => {
            let definition_id = env
                .effects
                .iter()
                .find(|u| u.id == f.effect)
                .expect("fixture effect use")
                .definition_id;
            let old = snap
                .effect_definition(definition_id)
                .expect("fixture effect definition")
                .enabled;
            Command::SetEffectEnabled {
                target: f.layer,
                effect: f.effect,
                old,
                new: *enabled,
            }
        }
        RandomEditSpec::EffectParam { amount } => {
            let definition_id = env
                .effects
                .iter()
                .find(|u| u.id == f.effect)
                .expect("fixture effect use")
                .definition_id;
            let old = snap
                .effect_definition(definition_id)
                .expect("fixture effect definition")
                .params
                .get("amount")
                .expect("fixture amount param")
                .clone();
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::EffectParam(f.effect, "amount".into()),
                old_value: old,
                new_value: DocParam::const_f64(*amount),
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        // 再締結ゲート B.3: 固定seed (0x4D32_B303_5EED_0001) で再現可能な異種編集列審判
        cases: 32,
        rng_seed: RngSeed::Fixed(0x4D32_B303_5EED_0001),
        .. ProptestConfig::default()
    })]

    #[test]
    fn random_multi_gesture_sequence_undo_redo_restores_semantic_state(
        gestures in multi_gesture_sequence_strategy()
    ) {
        let f = fixture();
        let initial = f.doc.clone();
        let mut writer = reference_writer(initial.clone());
        let gesture_count = gestures.len();

        for edits in &gestures {
            let gesture = writer.begin_gesture();
            for spec in edits {
                let cmd = build_random_edit_command(&writer, &f, spec);
                writer
                    .apply_command(gesture, cmd)
                    .expect("apply_command must succeed");
                writer.validate().expect("document must validate after apply");
            }
        }

        let applied = writer.snapshot().as_ref().clone();
        assert_eq!(writer.undo_len(), gesture_count);

        for _ in 0..gesture_count {
            writer.undo().expect("undo");
        }
        assert_eq!(writer.undo_len(), 0);
        assert_eq!(writer.snapshot().as_ref(), &initial);

        for _ in 0..gesture_count {
            writer.redo().expect("redo");
        }
        assert_eq!(writer.snapshot().as_ref(), &applied);

        for _ in 0..gesture_count {
            writer.undo().expect("undo");
        }
        assert_eq!(writer.snapshot().as_ref(), &initial);
    }
}

// ---------------------------------------------------------------------------
// CU-201R: 固定seedによる乱択MOVE/TRIM(同一Clip)列の全Undo審判
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Cu201rEditSpec {
    Move { delta_ticks: i64 },
    TrimIn { delta_ticks: i64 },
    TrimOut { delta_ticks: i64 },
}

#[derive(Debug, Clone)]
struct Cu201rFixture {
    doc: Document,
    target: LayerId,
}

fn cu201r_fixture() -> Cu201rFixture {
    let mut doc = Document::new_current();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let target = doc.layers.allocate("target").unwrap();
    let left = doc.layers.allocate("left").unwrap();
    let right = doc.layers.allocate("right").unwrap();

    let source = ClipSource::asset_video_only(asset);

    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(left),
                start: RationalTime::try_new(0, 1).unwrap(),
                duration: RationalTime::try_new(2, 1).unwrap(),
                time_map: Default::default(),
                source: source.clone(),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(target),
                start: RationalTime::try_new(3, 1).unwrap(),
                duration: RationalTime::try_new(3, 1).unwrap(),
                time_map: TimeMap::identity(),
                source: source.clone(),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(right),
                start: RationalTime::try_new(7, 1).unwrap(),
                duration: RationalTime::try_new(2, 1).unwrap(),
                time_map: Default::default(),
                source,
            }),
        ],
    });
    doc.validate().expect("fixture must be valid");

    Cu201rFixture { doc, target }
}

fn cu201r_edit_spec_strategy() -> impl Strategy<Value = Cu201rEditSpec> {
    prop_oneof![
        (-4i64..=4).prop_map(|delta_ticks| Cu201rEditSpec::Move { delta_ticks }),
        (-3i64..=3).prop_map(|delta_ticks| Cu201rEditSpec::TrimIn { delta_ticks }),
        (-3i64..=3).prop_map(|delta_ticks| Cu201rEditSpec::TrimOut { delta_ticks }),
    ]
}

fn cu201r_edit_sequence_strategy() -> impl Strategy<Value = Vec<Cu201rEditSpec>> {
    prop::collection::vec(cu201r_edit_spec_strategy(), 64)
}

fn cu201r_track_layer_signature(doc: &Document) -> Vec<Vec<LayerId>> {
    doc.tracks
        .iter()
        .map(|track| {
            track
                .items
                .iter()
                .map(|item| match item {
                    TrackItem::Clip(clip) => clip.envelope.layer_id,
                    TrackItem::Group(group) => group.envelope.layer_id,
                })
                .collect()
        })
        .collect()
}

fn cu201r_collect_track_layer_multiset(doc: &Document) -> BTreeMap<LayerId, usize> {
    let mut counts = BTreeMap::new();
    for track in &doc.tracks {
        for item in &track.items {
            let layer_id = match item {
                TrackItem::Clip(clip) => clip.envelope.layer_id,
                TrackItem::Group(group) => group.envelope.layer_id,
            };
            *counts.entry(layer_id).or_insert(0) += 1;
        }
    }
    counts
}

fn cu201r_assert_layer_multiset_has_no_duplicates(doc: &Document) {
    let multiset = cu201r_collect_track_layer_multiset(doc);
    for (layer_id, count) in &multiset {
        assert_eq!(*count, 1, "LayerId {layer_id:?} appears {count} times");
    }
}

fn cu201r_sentinel_clips(doc: &Document, target: LayerId) -> BTreeMap<LayerId, Clip> {
    let mut sentinels = BTreeMap::new();
    for track in &doc.tracks {
        for item in &track.items {
            if let TrackItem::Clip(clip) = item {
                if clip.envelope.layer_id != target {
                    sentinels.insert(clip.envelope.layer_id, clip.clone());
                }
            }
        }
    }
    sentinels
}

fn cu201r_target_clip(doc: &Document, target: LayerId) -> Option<&Clip> {
    for track in &doc.tracks {
        for item in &track.items {
            if let TrackItem::Clip(clip) = item {
                if clip.envelope.layer_id == target {
                    return Some(clip);
                }
            }
        }
    }
    None
}

fn build_cu201r_command(
    writer: &DocumentWriter,
    target: LayerId,
    spec: &Cu201rEditSpec,
) -> Command {
    let doc = writer.snapshot();
    let snapshot = doc.as_ref();
    let clip = cu201r_target_clip(snapshot, target).expect("target clip must exist");
    let one = RationalTime::try_new(1, 1).unwrap();

    match spec {
        Cu201rEditSpec::Move { delta_ticks } => {
            let max_start = snapshot
                .composition
                .duration
                .try_sub(clip.duration)
                .unwrap();
            let range_start = max_start.try_sub(one.try_mul_i64(8).unwrap()).unwrap();
            let selected_index = *delta_ticks + 4;
            let selected = range_start
                .try_add(one.try_mul_i64(selected_index).unwrap())
                .unwrap();
            let new_start = if selected == clip.start {
                let adjacent_index = if selected_index < 8 {
                    selected_index + 1
                } else {
                    selected_index - 1
                };
                range_start
                    .try_add(one.try_mul_i64(adjacent_index).unwrap())
                    .unwrap()
            } else {
                selected
            };
            writer
                .prepare_set_clip_start(target, new_start)
                .expect("mapped MOVE must be valid")
                .expect("mapped MOVE must change the clip")
        }
        Cu201rEditSpec::TrimIn { delta_ticks } => {
            let old_end = clip
                .start
                .try_add(clip.duration)
                .expect("clip interval should be valid");
            let range_start = old_end.try_sub(one.try_mul_i64(7).unwrap()).unwrap();
            let selected_index = *delta_ticks + 3;
            let selected = range_start
                .try_add(one.try_mul_i64(selected_index).unwrap())
                .unwrap();
            let new_start = if selected == clip.start {
                let adjacent_index = if selected_index < 6 {
                    selected_index + 1
                } else {
                    selected_index - 1
                };
                range_start
                    .try_add(one.try_mul_i64(adjacent_index).unwrap())
                    .unwrap()
            } else {
                selected
            };
            let cmd = writer
                .prepare_trim_clip_in(target, new_start)
                .expect("mapped left TRIM must be valid")
                .expect("mapped left TRIM must change the clip");
            assert!(
                cmd.stable_id_reservation().is_none(),
                "SetTrimClipIn command should not reserve stable IDs in CU-201R"
            );
            cmd
        }
        Cu201rEditSpec::TrimOut { delta_ticks } => {
            let old_end = clip
                .start
                .try_add(clip.duration)
                .expect("clip interval should be valid");
            let available = snapshot
                .composition
                .duration
                .try_sub(clip.start)
                .expect("target start must precede composition end");
            let selected_index = *delta_ticks + 3;
            let selected = clip
                .start
                .try_add(
                    available
                        .try_mul(RationalTime::try_new(selected_index + 1, 8).unwrap())
                        .unwrap(),
                )
                .unwrap();
            let new_end = if selected == old_end {
                let adjacent_index = if selected_index < 6 {
                    selected_index + 1
                } else {
                    selected_index - 1
                };
                clip.start
                    .try_add(
                        available
                            .try_mul(RationalTime::try_new(adjacent_index + 1, 8).unwrap())
                            .unwrap(),
                    )
                    .unwrap()
            } else {
                selected
            };
            writer
                .prepare_trim_clip_out(target, new_end)
                .expect("mapped right TRIM must be valid")
                .expect("mapped right TRIM must change the clip")
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        // CU-201R: 64 steps/ケース×32 = 2048 で、固定seed下の十分数操作を検査。
        cases: 32,
        rng_seed: RngSeed::Fixed(0x4D32_B303_5EED_0001),
        .. ProptestConfig::default()
    })]

    #[test]
    fn cu_201r_random_move_trim_sequence_undo_restores_state(
        steps in cu201r_edit_sequence_strategy()
    ) {
        let fixture = cu201r_fixture();
        let initial = fixture.doc.clone();
        let initial_sentinels = cu201r_sentinel_clips(&initial, fixture.target);
        let initial_signature = cu201r_track_layer_signature(&initial);
        let target = cu201r_target_clip(&initial, fixture.target).expect("target clip must exist");
        let target_envelope = target.envelope.clone();
        let target_source = target.source.clone();
        let initial_next_stable_id = initial.next_stable_id.peek_next();
        let mut writer = reference_writer(initial.clone());
        let mut accepted = 0usize;

        for spec in steps {
            let cmd = build_cu201r_command(&writer, fixture.target, &spec);

            let gesture = writer.begin_gesture();
            writer
                .apply_command(gesture, cmd)
                .expect("apply_command must succeed");
            writer.validate().expect("document must validate after apply");
            accepted += 1;

            let snapshot = writer.snapshot();
            let after = snapshot.as_ref();
            cu201r_assert_layer_multiset_has_no_duplicates(after);
            assert_eq!(cu201r_track_layer_signature(after), initial_signature);

            let sentinels = cu201r_sentinel_clips(after, fixture.target);
            assert_eq!(sentinels, initial_sentinels);

            let after_target = cu201r_target_clip(after, fixture.target)
                .expect("target clip must still exist");
            assert_eq!(after_target.envelope, target_envelope);
            assert_eq!(after_target.source, target_source);
        }

        assert_eq!(accepted, 64);
        assert_eq!(writer.undo_len(), accepted);
        for _ in 0..accepted {
            writer.undo().expect("undo");
        }
        assert_eq!(writer.undo_len(), 0);
        assert_eq!(writer.snapshot().as_ref(), &initial);
        assert_eq!(writer.snapshot().as_ref().next_stable_id.peek_next(), initial_next_stable_id);
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
    let mut writer = reference_writer(f.doc);
    let gesture = writer.begin_gesture();
    let base = writer.snapshot().next_stable_id.peek_next();
    let effect = EffectInstance {
        id: EffectId::from_raw(base),
        definition_id: EffectDefinitionId::from_raw(base + 1),
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
                introduced_definition: true,
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
    let mut writer = reference_writer(f.doc);
    let gesture = writer.begin_gesture();
    let layer_names = BTreeMap::from([(new_layer, "x".to_string())]);
    let item = TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(new_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::asset_video_only(motolii_doc::AssetId::from_raw(0)),
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
    let mut doc = Document::new_current();
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

fn replace_fixture_position(doc: &mut Document, layer: LayerId, position: DocParam) {
    for track in &mut doc.tracks {
        for item in &mut track.items {
            let TrackItem::Clip(clip) = item else {
                continue;
            };
            if clip.envelope.layer_id == layer {
                clip.envelope.transform.position = position;
                return;
            }
        }
    }
    panic!("fixture layer must resolve to a clip");
}

fn keyed_position(first: KeyframeId, second: Option<KeyframeId>) -> DocParam {
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: first,
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
    });
    if let Some(second) = second {
        track.insert(DocKeyframe {
            id: second,
            t: RationalTime::from_seconds(1),
            value: DocValue::Vec2([1.0, 1.0]),
            interp: Interp::Hold,
        });
    }
    DocParam::Keyframes(track)
}

fn interp_command(target: LayerId, key: KeyframeId, old: Interp, new: Interp) -> Command {
    Command::SetPositionKeyInterp {
        target,
        key,
        old,
        new,
    }
}

fn document_json(doc: &Document) -> Vec<u8> {
    serde_json::to_vec(doc).unwrap()
}

#[test]
fn position_key_interp_noop_and_terminal_key_preserve_document_counters_and_bytes() {
    let mut f = fixture();
    let key = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    replace_fixture_position(&mut f.doc, f.layer, keyed_position(key, None));
    f.doc.validate().unwrap();
    let counter = f.doc.next_stable_id.peek_next();
    let before = f.doc.clone();
    let writer = reference_writer(f.doc);

    assert_eq!(
        writer
            .prepare_set_position_key_interp(f.layer, key, Interp::Linear)
            .unwrap(),
        None
    );
    assert_eq!(writer.snapshot().as_ref(), &before);
    assert_eq!(writer.revision, 0);
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), counter);

    let identity = interp_command(f.layer, key, Interp::Linear, Interp::Linear);
    let raw_before = document_json(writer.snapshot().as_ref());
    let mut raw_document = (*writer.snapshot()).clone();
    identity.apply(&mut raw_document).unwrap();
    assert_eq!(document_json(&raw_document), raw_before);
    assert_eq!(raw_document.next_stable_id.peek_next(), counter);

    let command = writer
        .prepare_set_position_key_interp(f.layer, key, Interp::Hold)
        .unwrap()
        .unwrap();
    command.apply(&mut raw_document).unwrap();
    let TrackItem::Clip(clip) = &raw_document.tracks[0].items[0] else {
        panic!("fixture target must be a clip");
    };
    let DocParam::Keyframes(track) = &clip.envelope.transform.position else {
        panic!("terminal position must remain keyframed");
    };
    assert_eq!(track.get_by_id(key).unwrap().interp, Interp::Hold);
    assert_eq!(raw_document.next_stable_id.peek_next(), counter);
}

#[test]
fn position_key_interp_rejections_are_ordered_typed_and_byte_preserving() {
    let mut f = fixture();
    let key = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    replace_fixture_position(&mut f.doc, f.layer, keyed_position(key, None));
    f.doc.validate().unwrap();

    let reject = |doc: &Document, command: Command| {
        let before = document_json(doc);
        let mut working = doc.clone();
        let error = command.apply(&mut working).unwrap_err();
        assert_eq!(document_json(&working), before);
        error
    };
    assert!(matches!(
        reject(
            &f.doc,
            interp_command(LayerId::from_raw(999), key, Interp::Linear, Interp::Hold)
        ),
        CommandError::LayerNotFound(999)
    ));

    let unsupported = [
        DocParam::const_vec2([0.0, 0.0]),
        DocParam::Vec2Axes {
            x: Box::new(DocParam::const_f64(0.0)),
            y: Box::new(DocParam::const_f64(0.0)),
        },
        DocParam::Data {
            track: motolii_eval::DataTrackId("position.source".into()),
            fallback: DocValue::Vec2([0.0, 0.0]),
        },
        DocParam::LookAt {
            target: f.other_layer,
            axis: LookAtAxis::PlusX,
        },
        DocParam::Follow {
            target: f.other_layer,
            offset: [0.0, 0.0],
        },
    ];
    for position in unsupported {
        let mut doc = f.doc.clone();
        replace_fixture_position(&mut doc, f.layer, position);
        assert!(matches!(
            reject(&doc, interp_command(f.layer, key, Interp::Linear, Interp::Hold)),
            CommandError::PositionKeyInterpSourceUnsupported { layer } if layer == f.layer.get()
        ));
    }

    let mut wrong_value = DocKeyframeTrack::new();
    wrong_value.insert(DocKeyframe {
        id: key,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    let mut wrong_value_doc = f.doc.clone();
    replace_fixture_position(
        &mut wrong_value_doc,
        f.layer,
        DocParam::Keyframes(wrong_value),
    );
    assert!(matches!(
        reject(
            &wrong_value_doc,
            interp_command(f.layer, key, Interp::Linear, Interp::Hold)
        ),
        CommandError::PositionKeyInterpValueTypeMismatch { layer } if layer == f.layer.get()
    ));

    let mut empty_doc = f.doc.clone();
    replace_fixture_position(
        &mut empty_doc,
        f.layer,
        DocParam::Keyframes(DocKeyframeTrack::new()),
    );
    assert!(matches!(
        reject(&empty_doc, interp_command(f.layer, key, Interp::Linear, Interp::Hold)),
        CommandError::PositionKeyNotFound { layer, key_id }
            if layer == f.layer.get() && key_id == key.get()
    ));
    let missing = KeyframeId::from_raw(key.get() + 1);
    assert!(matches!(
        reject(&f.doc, interp_command(f.layer, missing, Interp::Linear, Interp::Hold)),
        CommandError::PositionKeyNotFound { layer, key_id }
            if layer == f.layer.get() && key_id == missing.get()
    ));

    let non_finite = Interp::Bezier {
        x1: f64::NAN,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
    assert!(matches!(
        reject(
            &f.doc,
            interp_command(f.layer, key, non_finite, Interp::Hold)
        ),
        CommandError::PositionKeyInterpInvalid {
            source: DocKeyframeError::NonFiniteBezier,
            ..
        }
    ));
    let invalid_x = Interp::Bezier {
        x1: -0.1,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    };
    assert!(matches!(
        reject(
            &f.doc,
            interp_command(f.layer, key, Interp::Linear, invalid_x)
        ),
        CommandError::PositionKeyInterpInvalid {
            source: DocKeyframeError::InvalidBezier { .. },
            ..
        }
    ));
    assert!(matches!(
        reject(&f.doc, interp_command(f.layer, key, Interp::Hold, Interp::Linear)),
        CommandError::PositionKeyInterpPayloadMismatch { layer, key_id }
            if layer == f.layer.get() && key_id == key.get()
    ));
}

#[test]
fn position_key_interp_merge_identity_keeps_keys_and_layers_distinct() {
    let mut f = fixture();
    let first = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let second = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let other = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    replace_fixture_position(&mut f.doc, f.layer, keyed_position(first, Some(second)));
    replace_fixture_position(&mut f.doc, f.other_layer, keyed_position(other, None));
    f.doc.validate().unwrap();
    let counter = f.doc.next_stable_id.peek_next();
    let before = f.doc.clone();
    let mut writer = reference_writer(f.doc);
    let gesture = writer.begin_gesture();

    let first_bezier = Interp::Bezier {
        x1: 0.2,
        y1: 0.0,
        x2: 0.8,
        y2: 1.0,
    };
    let first_change = interp_command(f.layer, first, Interp::Linear, first_bezier);
    let first_final = interp_command(f.layer, first, first_bezier, Interp::Hold);
    let second_change = interp_command(f.layer, second, Interp::Hold, Interp::Linear);
    let other_change = interp_command(f.other_layer, other, Interp::Linear, Interp::Hold);
    assert_ne!(first_change.property(), second_change.property());
    assert_ne!(
        first_change.target_stable_id(),
        other_change.target_stable_id()
    );
    assert_ne!(
        first_change.merge_key(gesture),
        second_change.merge_key(gesture)
    );
    assert_ne!(
        first_change.merge_key(gesture),
        other_change.merge_key(gesture)
    );
    assert_eq!(
        first_change.inverse(),
        interp_command(f.layer, first, first_bezier, Interp::Linear,)
    );

    writer.apply_command(gesture, first_change).unwrap();
    writer.apply_command(gesture, first_final).unwrap();
    writer.apply_command(gesture, second_change).unwrap();
    writer.apply_command(gesture, other_change).unwrap();
    assert_eq!(writer.undo_len(), 1);
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), counter);
    let after = (*writer.snapshot()).clone();
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &before);
    writer.redo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &after);
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), counter);
}

#[test]
fn position_key_interp_is_key_only_cas_mergeable_and_undoable() {
    let mut f = fixture();
    let first = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let second = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let mut position = DocKeyframeTrack::new();
    position.insert(DocKeyframe {
        id: first,
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: Interp::Linear,
    });
    position.insert(DocKeyframe {
        id: second,
        t: RationalTime::from_seconds(1),
        value: DocValue::Vec2([1.0, 1.0]),
        interp: Interp::Hold,
    });
    let TrackItem::Clip(clip) = &mut f.doc.tracks[0].items[0] else {
        panic!("fixture target must be a clip");
    };
    clip.envelope.transform.position = DocParam::Keyframes(position);
    f.doc.validate().unwrap();

    let counter = f.doc.next_stable_id.peek_next();
    let before = f.doc.clone();
    let mut writer = reference_writer(f.doc);
    let first_new = Interp::Bezier {
        x1: 0.2,
        y1: -0.3,
        x2: 0.8,
        y2: 1.4,
    };
    let gesture = writer.begin_gesture();
    let first_command = writer
        .prepare_set_position_key_interp(f.layer, first, first_new)
        .unwrap()
        .unwrap();
    assert_eq!(
        first_command.kind(),
        motolii_doc::CommandKind::SetPositionKeyInterp
    );
    assert_eq!(first_command.target_stable_id(), f.layer.get());
    assert!(first_command.stable_id_reservation().is_none());
    writer.apply_command(gesture, first_command).unwrap();

    let last_new = Interp::Hold;
    let last_command = writer
        .prepare_set_position_key_interp(f.layer, first, last_new)
        .unwrap()
        .unwrap();
    writer.apply_command(gesture, last_command).unwrap();
    assert_eq!(writer.undo_len(), 1);
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), counter);

    let snapshot = writer.snapshot();
    let TrackItem::Clip(after) = &snapshot.tracks[0].items[0] else {
        panic!("fixture target must be a clip");
    };
    let DocParam::Keyframes(after_position) = &after.envelope.transform.position else {
        panic!("position must remain keyframed");
    };
    assert_eq!(after_position.get_by_id(first).unwrap().interp, last_new);
    assert_eq!(
        after_position.get_by_id(second).unwrap().interp,
        Interp::Hold
    );
    assert_eq!(
        after_position.get_by_id(first).unwrap().t,
        RationalTime::ZERO
    );
    assert_eq!(
        after_position.get_by_id(first).unwrap().value,
        DocValue::Vec2([0.0, 0.0])
    );

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &before);
    writer.redo().unwrap();
    let stale = Command::SetPositionKeyInterp {
        target: f.layer,
        key: first,
        old: Interp::Linear,
        new: Interp::Hold,
    };
    let mut stale_doc = (*writer.snapshot()).clone();
    assert!(matches!(
        stale.apply(&mut stale_doc),
        Err(CommandError::PositionKeyInterpPayloadMismatch { .. })
    ));
}

#[test]
fn duplicate_track_item_allocates_fresh_ids_via_writer() {
    let f = fixture();
    let mut writer = reference_writer(f.doc);
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

#[test]
fn split_clip_splits_window_without_stretching_source_and_inverse_restores() {
    let f = fixture();
    let mut writer = reference_writer(f.doc);
    let at = RationalTime::try_new(2, 1).unwrap();
    let original_source = writer.snapshot().tracks[0].items[0]
        .as_clip()
        .unwrap()
        .source
        .clone();
    let original_time_map = writer.snapshot().tracks[0].items[0]
        .as_clip()
        .unwrap()
        .time_map;
    let cmd = writer
        .prepare_split_clip(f.layer, at)
        .expect("split prepare")
        .expect("split must change");
    assert_eq!(cmd.kind(), motolii_doc::CommandKind::SplitClip);
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("SplitClip"), "{json}");
    let before = writer.snapshot();
    assert_roundtrip(before.as_ref(), cmd.clone());

    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, cmd).expect("split apply");
    let after = writer.snapshot();
    let left = after.tracks[0].items[0].as_clip().expect("left");
    let right = after.tracks[0].items[1].as_clip().expect("right");
    assert_eq!(left.envelope.layer_id, f.layer);
    assert_ne!(right.envelope.layer_id, f.layer);
    assert_eq!(left.start, RationalTime::ZERO);
    assert_eq!(left.duration, RationalTime::try_new(2, 1).unwrap());
    assert_eq!(left.time_map, original_time_map);
    assert_eq!(right.start, at);
    assert_eq!(right.duration, RationalTime::try_new(3, 1).unwrap());
    assert_eq!(
        right.time_map.source_start,
        RationalTime::try_new(2, 1).unwrap()
    );
    assert_eq!(left.source, original_source);
    assert_eq!(left.source, right.source);

    writer.undo().expect("split undo");
    let restored = writer.snapshot();
    let left = restored.tracks[0].items[0].as_clip().expect("restored");
    assert_eq!(restored.tracks[0].items.len(), 2);
    assert_eq!(left.envelope.layer_id, f.layer);
    assert_eq!(left.duration, RationalTime::try_new(5, 1).unwrap());
    assert_eq!(left.time_map, original_time_map);
}

#[test]
fn split_clip_rejects_boundary_and_leaves_document_unchanged() {
    let f = fixture();
    let mut writer = reference_writer(f.doc.clone());
    let before = writer.snapshot();
    let err = writer
        .prepare_split_clip(f.layer, RationalTime::ZERO)
        .expect_err("start is not interior");
    assert!(matches!(err, CommandError::SplitNotInterior { .. }));
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());

    let mut writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let end = RationalTime::try_new(5, 1).unwrap();
    let err = writer
        .prepare_split_clip(f.layer, end)
        .expect_err("end is not interior");
    assert!(matches!(err, CommandError::SplitNotInterior { .. }));
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());
}

#[test]
fn reparent_clip_moves_clip_to_dest_lane_and_inverse_restores() {
    let f = fixture();
    let mut doc = f.doc;
    let track_b = doc.track_ids.allocate("V2").unwrap();
    doc.tracks.push(Track {
        id: track_b,
        items: vec![],
    });
    doc.validate().expect("two-track fixture");
    let writer = reference_writer(doc);
    let none = writer
        .prepare_reparent_clip(f.layer, ParentLocator::Track(f.track), 0, None)
        .expect("same seat");
    assert!(none.is_none());

    let cmd = writer
        .prepare_reparent_clip(f.layer, ParentLocator::Track(track_b), 0, None)
        .expect("reparent prepare")
        .expect("must move");
    assert_eq!(cmd.kind(), motolii_doc::CommandKind::ReparentClip);
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("ReparentClip"), "{json}");
    assert_roundtrip(writer.snapshot().as_ref(), cmd.clone());
    let mut working = writer.snapshot().as_ref().clone();
    cmd.apply(&mut working).expect("apply");
    assert!(working.tracks[0].items.iter().all(|item| match item {
        TrackItem::Clip(clip) => clip.envelope.layer_id != f.layer,
        TrackItem::Group(group) => group.envelope.layer_id != f.layer,
    }));
    let moved = working.tracks[1].items[0].as_clip().unwrap();
    assert_eq!(moved.envelope.layer_id, f.layer);
    assert_eq!(moved.start, RationalTime::ZERO);
}

#[test]
fn reparent_clip_rejects_missing_dest_track_and_leaves_document_unchanged() {
    let f = fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let missing = motolii_doc::TrackId::from_raw(u64::MAX);
    let err = writer
        .prepare_reparent_clip(f.layer, ParentLocator::Track(missing), 0, None)
        .expect_err("missing dest track");
    assert!(matches!(err, CommandError::TrackNotFound(_)));
    assert_eq!(writer.snapshot().as_ref(), before.as_ref());
}

#[test]
fn set_item_visible_and_solo_roundtrip() {
    let f = fixture();
    let visible = Command::SetItemVisible {
        target: f.layer,
        old: true,
        new: false,
    };
    assert_roundtrip(&f.doc, visible);
    let solo = Command::SetItemSolo {
        target: f.layer,
        old: false,
        new: true,
    };
    assert_roundtrip(&f.doc, solo);
}

// ---------------------------------------------------------------------------
// 完了条件3: gesture merge(#103⑨、merge key=S18)
// ---------------------------------------------------------------------------

#[test]
fn empty_macro_rejects_without_changing_writer_state() {
    let f = fixture();
    let mut writer = reference_writer(f.doc);
    let gesture = writer.begin_gesture();
    writer
        .apply_command(
            gesture,
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::Opacity,
                old_value: DocParam::const_f64(1.0),
                new_value: DocParam::const_f64(0.5),
            },
        )
        .unwrap();
    writer.undo().unwrap();

    let before = serde_json::to_vec(&*writer.snapshot()).unwrap();
    let before_revision = writer.revision;
    let before_undo = writer.undo_len();
    let before_redo = writer.redo_len();

    assert_eq!(
        writer.apply_macro(Vec::new()),
        Err(CommandError::EmptyMacro)
    );
    assert_eq!(serde_json::to_vec(&*writer.snapshot()).unwrap(), before);
    assert_eq!(writer.revision, before_revision);
    assert_eq!(writer.undo_len(), before_undo);
    assert_eq!(writer.redo_len(), before_redo);
    assert_eq!(writer.begin_gesture().get(), 1);
}

#[test]
fn macro_applies_once_and_undoes_and_redoes_as_one_gesture() {
    let f = fixture();
    let mut writer = reference_writer(f.doc.clone());

    let gesture = writer
        .apply_macro(vec![
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::Position,
                old_value: DocParam::const_vec2([0.0, 0.0]),
                new_value: DocParam::const_vec2([10.0, 0.0]),
            },
            Command::SetProperty {
                target: f.layer,
                property: ScalarPropertyId::Position,
                old_value: DocParam::const_vec2([10.0, 0.0]),
                new_value: DocParam::const_vec2([20.0, 0.0]),
            },
            Command::SetProperty {
                target: f.other_layer,
                property: ScalarPropertyId::Opacity,
                old_value: DocParam::const_f64(1.0),
                new_value: DocParam::const_f64(0.5),
            },
        ])
        .unwrap();

    assert_eq!(gesture.get(), 0);
    assert_eq!(writer.revision, 1);
    assert_eq!(writer.undo_len(), 1);
    let applied = writer.snapshot();
    let TrackItem::Clip(first) = &applied.tracks[0].items[0] else {
        panic!("expected first fixture clip");
    };
    let TrackItem::Clip(second) = &applied.tracks[0].items[1] else {
        panic!("expected second fixture clip");
    };
    assert_eq!(
        first.envelope.transform.position,
        DocParam::const_vec2([20.0, 0.0]),
        "same-target updates must retain the final merged value"
    );
    assert_eq!(
        second.envelope.opacity,
        DocParam::const_f64(0.5),
        "a different target must remain a distinct command in the macro"
    );

    writer.undo().unwrap();
    assert_eq!(&*writer.snapshot(), &f.doc);
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 1);

    writer.redo().unwrap();
    assert_eq!(writer.snapshot(), applied);
    assert_eq!(writer.undo_len(), 1);
    assert_eq!(writer.redo_len(), 0);
}

#[test]
fn macro_command_failure_at_each_position_restores_all_writer_state() {
    let f = fixture();

    for failure_index in 0..3 {
        let mut writer = reference_writer(f.doc.clone());
        let prior_gesture = writer.begin_gesture();
        writer
            .apply_command(
                prior_gesture,
                Command::SetProperty {
                    target: f.layer,
                    property: ScalarPropertyId::Opacity,
                    old_value: DocParam::const_f64(1.0),
                    new_value: DocParam::const_f64(0.75),
                },
            )
            .unwrap();
        writer.undo().unwrap();

        let valid_first = Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Position,
            old_value: DocParam::const_vec2([0.0, 0.0]),
            new_value: DocParam::const_vec2([10.0, 0.0]),
        };
        let invalid = Command::SetProperty {
            target: LayerId::from_raw(u64::MAX),
            property: ScalarPropertyId::Position,
            old_value: DocParam::const_vec2([0.0, 0.0]),
            new_value: DocParam::const_vec2([20.0, 0.0]),
        };
        let valid_last = Command::SetProperty {
            target: f.other_layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.5),
        };
        let mut commands = vec![valid_first, valid_last];
        commands.insert(failure_index, invalid);

        let before = serde_json::to_vec(&*writer.snapshot()).unwrap();
        let before_revision = writer.revision;
        let before_undo = writer.undo_len();
        let before_redo = writer.redo_len();

        assert!(matches!(
            writer.apply_macro(commands),
            Err(CommandError::LayerNotFound(id)) if id == u64::MAX
        ));
        assert_eq!(
            serde_json::to_vec(&*writer.snapshot()).unwrap(),
            before,
            "failure index {failure_index}"
        );
        assert_eq!(writer.revision, before_revision);
        assert_eq!(writer.undo_len(), before_undo);
        assert_eq!(writer.redo_len(), before_redo);
        assert_eq!(writer.begin_gesture().get(), 1);
    }
}

#[test]
fn same_gesture_drag_merges_into_one_macro_and_undoes_atomically() {
    let f = fixture();
    let mut writer = reference_writer(f.doc.clone());
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
    let mut writer = reference_writer(f.doc.clone());

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
    let mut doc = f.doc.clone();
    let (effect1_id, effect1_def, effect2_id, effect2_def) =
        allocate_effect_ids_for_add_effect_test(&mut doc);
    let mut writer = reference_writer(doc);
    let gesture = writer.begin_gesture();

    let effect1 = EffectInstance {
        id: effect1_id,
        definition_id: effect1_def,
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let effect2 = EffectInstance {
        id: effect2_id,
        definition_id: effect2_def,
        plugin_id: "vendor.filter.fixture".into(),
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
                introduced_definition: true,
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
                introduced_definition: true,
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
    let mut doc = f.doc.clone();
    let (effect1_id, effect1_def, effect2_id, effect2_def) =
        allocate_effect_ids_for_add_effect_test(&mut doc);
    let mut writer = reference_writer(doc);
    let gesture = writer.begin_gesture();

    let effect1 = EffectInstance {
        id: effect1_id,
        definition_id: effect1_def,
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let effect2 = EffectInstance {
        id: effect2_id,
        definition_id: effect2_def,
        plugin_id: "vendor.filter.fixture".into(),
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
                introduced_definition: true,
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
                introduced_definition: true,
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
    let mut doc = Document::new_current();
    let external_layer = doc.layers.allocate("external").unwrap();
    let group_layer = doc.layers.allocate("group").unwrap();
    let child_a = doc.layers.allocate("child_a").unwrap();
    let child_b = doc.layers.allocate("child_b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

    let mut env_a = ItemEnvelope::new(child_a);
    // subtree内参照(sibling child_b) — 複製後は新IDへ再写像されるべき。
    env_a.transform.rotation = DocParam::LookAt {
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
    // (LookAt は rotation のみ許可 — d1h_validate::look_at_on_rotation_ok 参照)
    env_b.transform.rotation = DocParam::LookAt {
        target: external_layer,
        axis: LookAtAxis::PlusY,
    };

    let mut group_env = ItemEnvelope::new(group_layer);
    let effect_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let effect_def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        effect_def_id,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::new(),
        Default::default(),
    ));
    group_env.effects.push(EffectUse {
        id: effect_id,
        definition_id: effect_def_id,
    });

    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(external_layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
            TrackItem::Group(Group {
                envelope: group_env,
                children: vec![
                    TrackItem::Clip(Clip {
                        envelope: env_a,
                        start: RationalTime::ZERO,
                        duration: RationalTime::try_new(2, 1).unwrap(),
                        time_map: Default::default(),
                        source: ClipSource::asset_video_only(asset),
                    }),
                    TrackItem::Clip(Clip {
                        envelope: env_b,
                        start: RationalTime::ZERO,
                        duration: RationalTime::try_new(2, 1).unwrap(),
                        time_map: Default::default(),
                        source: ClipSource::asset_video_only(asset),
                    }),
                ],
            }),
        ],
    });
    doc.validate().expect("fixture must validate");

    let mut writer = reference_writer(doc.clone());
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
    match &cloned_a.envelope.transform.rotation {
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
    match &cloned_b.envelope.transform.rotation {
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
    let mut doc = Document::new_current();
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
                    source: ClipSource::asset_video_only(asset),
                }),
                TrackItem::Clip(Clip {
                    envelope: ItemEnvelope::new(child_b),
                    start: RationalTime::ZERO,
                    duration: RationalTime::try_new(1, 1).unwrap(),
                    time_map: Default::default(),
                    source: ClipSource::asset_video_only(asset),
                }),
            ],
        })],
    });
    doc.validate().expect("fixture");

    let baseline = layer_entries(&doc);
    let mut writer = reference_writer(doc);
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
    let mut doc = Document::new_current();
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
                    source: ClipSource::asset_video_only(asset),
                }),
            ],
        })],
    });
    doc.validate().expect("fixture must validate");

    let mut writer = reference_writer(doc.clone());
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

// ---------------------------------------------------------------------------
// D1l: lifecycle undo/redoはDocument全体を復元する
// ---------------------------------------------------------------------------

fn assert_writer_roundtrip(mut writer: DocumentWriter, before: Document, cmd: Command) {
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, cmd.clone()).expect("apply");
    writer.undo().expect("undo");
    assert_eq!(writer.snapshot().as_ref(), &before);
    writer.redo().expect("redo");
    let mut expected = before.clone();
    cmd.apply(&mut expected).expect("re-apply");
    assert_eq!(writer.snapshot().as_ref(), &expected);
}

#[test]
fn add_effect_create_undo_redo_restores_full_document() {
    let mut f = fixture();
    let effect_id = f.doc.next_stable_id.allocate().unwrap();
    let definition_id = f.doc.next_stable_id.allocate().unwrap();
    let effect = EffectInstance {
        id: EffectId::from_raw(effect_id),
        definition_id: EffectDefinitionId::from_raw(definition_id),
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let cmd = Command::AddEffect {
        target: f.layer,
        index: 1,
        effect,
        introduced_definition: true,
    };
    assert_writer_roundtrip(reference_writer(f.doc.clone()), f.doc, cmd);
}

#[test]
fn add_effect_link_undo_redo_restores_full_document() {
    let mut f = fixture();
    let effect_id = f.doc.next_stable_id.allocate().unwrap();
    let effect = EffectInstance::from_use_and_definition(
        &EffectUse {
            id: EffectId::from_raw(effect_id),
            definition_id: f.effect_def,
        },
        f.doc.effect_definition(f.effect_def).unwrap(),
    );
    let cmd = Command::AddEffect {
        target: f.layer,
        index: 1,
        effect,
        introduced_definition: false,
    };
    assert_writer_roundtrip(reference_writer(f.doc.clone()), f.doc, cmd);
}

#[test]
fn add_effect_link_rejects_use_id_colliding_with_existing_effect() {
    let f = fixture();
    let def = f.doc.effect_definition(f.effect_def).unwrap().clone();
    let cmd = Command::AddEffect {
        target: f.layer,
        index: 1,
        effect: EffectInstance::from_use_and_definition(
            &EffectUse {
                id: f.effect,
                definition_id: f.effect_def,
            },
            &def,
        ),
        introduced_definition: false,
    };
    let mut working = f.doc.clone();
    let before = working.clone();
    let err = cmd
        .apply(&mut working)
        .expect_err("colliding use id must reject");
    assert_eq!(err, CommandError::StableIdCollision { id: f.effect.get() });
    assert_eq!(working, before);
}

#[test]
fn add_effect_create_rejects_use_id_collision_without_inserting_definition() {
    let f = fixture();
    let new_definition_id = EffectDefinitionId::from_raw(f.doc.next_stable_id.peek_next());
    let cmd = Command::AddEffect {
        target: f.layer,
        index: 1,
        effect: EffectInstance {
            id: f.effect,
            definition_id: new_definition_id,
            plugin_id: "core.filter.blur".into(),
            effect_version: 1,
            enabled: true,
            params: BTreeMap::new(),
            extra: Default::default(),
        },
        introduced_definition: true,
    };
    let mut working = f.doc.clone();
    let before = working.clone();
    let err = cmd
        .apply(&mut working)
        .expect_err("create with colliding use id must reject");
    assert_eq!(err, CommandError::StableIdCollision { id: f.effect.get() });
    assert_eq!(working, before);
    assert!(working.effect_definition(new_definition_id).is_none());
}

#[test]
fn add_effect_link_rejects_use_id_colliding_with_existing_keyframe() {
    let mut f = fixture();
    let kf_id = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let TrackItem::Clip(clip) = &mut f.doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    let mut opacity_track = DocKeyframeTrack::new();
    opacity_track.insert(DocKeyframe {
        id: kf_id,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Hold,
    });
    clip.envelope.opacity = DocParam::Keyframes(opacity_track);
    f.doc.validate().unwrap();

    let def = f.doc.effect_definition(f.effect_def).unwrap().clone();
    let new_use_id = EffectId::from_raw(kf_id.get());
    let cmd = Command::AddEffect {
        target: f.layer,
        index: 1,
        effect: EffectInstance::from_use_and_definition(
            &EffectUse {
                id: new_use_id,
                definition_id: f.effect_def,
            },
            &def,
        ),
        introduced_definition: false,
    };
    let mut working = f.doc.clone();
    let before = working.clone();
    let err = cmd
        .apply(&mut working)
        .expect_err("keyframe collision must reject");
    assert_eq!(err, CommandError::StableIdCollision { id: kf_id.get() });
    assert_eq!(working, before);
}

#[test]
fn unlink_undo_redo_restores_full_document() {
    let f = fixture();
    let use_ = f.doc.tracks[0].items[0].as_clip().unwrap().envelope.effects[0].clone();
    let def = f.doc.effect_definition(use_.definition_id).unwrap().clone();
    let cmd = Command::RemoveEffect {
        target: f.layer,
        index: 0,
        effect: EffectInstance::from_use_and_definition(&use_, &def),
        introduced_definition: false,
    };
    assert_writer_roundtrip(reference_writer(f.doc.clone()), f.doc, cmd);
}

#[test]
fn copy_local_last_reference_undo_redo_restores_full_document() {
    let s = shared_fixture_from_d2();
    let before = s.doc.next_stable_id.peek_next();
    let new_def_id = EffectDefinitionId::from_raw(before);
    let source = s.doc.effect_definition(s.d1).unwrap();
    let mut new_def = source.clone();
    new_def.id = new_def_id;
    let cmd = Command::CopyLocalEffect {
        use_id: s.u3,
        previous_definition_id: s.d1,
        new_definition: new_def,
        stable_id_reservation: StableIdReservation::new(before, before + 1),
    };
    assert_identity_command_roundtrip(&s.doc, cmd);
}

#[test]
fn delete_unused_definition_undo_redo_restores_full_document() {
    let s = shared_fixture_from_d2();
    let def = s.doc.effect_definition(s.d2_orphan).unwrap().clone();
    let cmd = Command::DeleteEffectDefinition { definition: def };
    assert_writer_roundtrip(reference_writer(s.doc.clone()), s.doc, cmd);
}

#[test]
fn duplicate_track_item_shares_definition_but_mints_new_use_id() {
    let s = shared_fixture_from_d2();
    let orig_uses = s.doc.tracks[0].items[0]
        .as_clip()
        .unwrap()
        .envelope
        .effects
        .clone();
    let mut writer = reference_writer(s.doc);
    writer.duplicate_track_item(s.layer_a).expect("duplicate");
    let snap = writer.snapshot();
    let cloned_uses = snap.tracks[0].items[1]
        .as_clip()
        .unwrap()
        .envelope
        .effects
        .clone();
    assert_eq!(cloned_uses.len(), orig_uses.len());
    for (orig, cloned) in orig_uses.iter().zip(cloned_uses.iter()) {
        assert_ne!(orig.id, cloned.id);
        assert_eq!(orig.definition_id, cloned.definition_id);
    }
}

struct SharedD2 {
    doc: Document,
    layer_a: LayerId,
    u3: EffectId,
    d1: EffectDefinitionId,
    d2_orphan: EffectDefinitionId,
}

fn shared_fixture_from_d2() -> SharedD2 {
    let mut doc = Document::new_current();
    let layer_a = doc.layers.allocate("a").unwrap();
    let layer_b = doc.layers.allocate("b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let u1 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let u2 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let u3 = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let d1 = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    let d2_orphan = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        d1,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.4))]),
        Default::default(),
    ));
    doc.effect_definitions.push(EffectDefinition::new(
        d2_orphan,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.1))]),
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
    doc.validate().unwrap();
    SharedD2 {
        doc,
        layer_a,
        u3,
        d1,
        d2_orphan,
    }
}

trait ClipItem {
    fn as_clip(&self) -> Option<&Clip>;
}

impl ClipItem for TrackItem {
    fn as_clip(&self) -> Option<&Clip> {
        match self {
            TrackItem::Clip(c) => Some(c),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 削除: subtree を外し、Undo で同じ LayerId と表示名を戻す
// ---------------------------------------------------------------------------

/// **Group を消すと子も一緒に消え、Undo で全員が同じ id で戻る。**
///
/// 複製の裏返しである。`AddTrackItem` が台帳へ載せるのと同じ経路を逆へ通し、
/// `remove` した LayerId は `restore` で戻る(非再利用カウンタは巻き戻さない)。
#[test]
fn removing_a_group_takes_its_children_and_undo_puts_them_back() {
    let mut doc = Document::new_current();
    let group_layer = doc.layers.allocate("group").unwrap();
    let child_a = doc.layers.allocate("child_a").unwrap();
    let child_b = doc.layers.allocate("child_b").unwrap();
    let keeper = doc.layers.allocate("keeper").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let clip = |layer: LayerId| {
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })
    };
    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Group(Group {
                envelope: ItemEnvelope::new(group_layer),
                children: vec![clip(child_a), clip(child_b)],
            }),
            clip(keeper),
        ],
    });
    doc.validate().expect("fixture");

    let before = layer_entries(&doc);
    let mut writer = reference_writer(doc);
    let command = writer
        .prepare_remove_track_item(group_layer)
        .expect("prepare remove");
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, command).expect("apply remove");

    let after = writer.snapshot();
    writer.validate().expect("post-remove document must validate");
    assert_eq!(after.tracks[0].items.len(), 1, "Group が1つ外れた");
    assert!(
        !after.layers.contains(group_layer)
            && !after.layers.contains(child_a)
            && !after.layers.contains(child_b),
        "子も台帳から外れる"
    );
    assert!(after.layers.contains(keeper), "兄弟は消えない");

    writer.undo().expect("undo remove");
    let restored = writer.snapshot();
    writer.validate().expect("post-undo document must validate");
    assert_eq!(
        layer_entries(&restored),
        before,
        "**同じ LayerId と表示名が戻る**。id を振り直さない"
    );
    assert_eq!(restored.tracks[0].items.len(), 2);
}

/// 消すのは1つ。**兄弟の index がずれても、次の削除は正しい位置を消す。**
#[test]
fn removing_the_first_of_two_clips_leaves_the_second_addressable() {
    let mut doc = Document::new_current();
    let first = doc.layers.allocate("first").unwrap();
    let second = doc.layers.allocate("second").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let clip = |layer: LayerId| {
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })
    };
    doc.tracks.push(Track {
        id: track,
        items: vec![clip(first), clip(second)],
    });
    doc.validate().expect("fixture");

    let mut writer = reference_writer(doc);
    let gesture = writer.begin_gesture();
    let command = writer.prepare_remove_track_item(first).expect("prepare");
    writer.apply_command(gesture, command).expect("apply");

    let command = writer.prepare_remove_track_item(second).expect("prepare");
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, command).expect("apply second");
    assert!(writer.snapshot().tracks[0].items.is_empty());

    assert!(
        matches!(
            writer.prepare_remove_track_item(first),
            Err(CommandError::LayerNotFound(_))
        ),
        "既に消えたものは消せない"
    );
}

/// **グループ化は「空のGroupを置く」+「ReparentClipで入れる」で表せる。**
/// 新しい意味のcommandを足さずに、逆操作は既存の逆で閉じる。
#[test]
fn an_empty_group_can_be_added_and_filled_by_reparenting() {
    let mut doc = Document::new_current();
    let first = doc.layers.allocate("first").unwrap();
    let second = doc.layers.allocate("second").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let clip = |layer: LayerId| {
        TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })
    };
    doc.tracks.push(Track {
        id: track,
        items: vec![clip(first), clip(second)],
    });
    doc.validate().expect("fixture");

    let before = layer_entries(&doc);
    let mut writer = reference_writer(doc);
    let gesture = writer.begin_gesture();

    let command = writer
        .prepare_add_group(ParentLocator::Track(track), 0, "Group 1")
        .expect("prepare group");
    let Command::AddTrackItem { item, .. } = &command else {
        panic!("AddTrackItem を返す");
    };
    let group_layer = match item {
        TrackItem::Group(g) => g.envelope.layer_id,
        _ => panic!("Group を返す"),
    };
    writer.apply_command(gesture, command).expect("apply group");

    for (index, layer) in [first, second].into_iter().enumerate() {
        let command = writer
            .prepare_reparent_clip(layer, ParentLocator::Group(group_layer), index, None)
            .expect("prepare reparent")
            .expect("command");
        writer.apply_command(gesture, command).expect("apply reparent");
    }

    writer.validate().expect("空でなくなったGroupは検証を通る");
    let after = writer.snapshot();
    assert_eq!(after.tracks[0].items.len(), 1, "トップレベルはGroup1つ");
    match &after.tracks[0].items[0] {
        TrackItem::Group(g) => {
            assert_eq!(g.children.len(), 2, "2枚とも中に入った");
            assert_eq!(g.envelope.layer_id, group_layer);
        }
        _ => panic!("Group であること"),
    }

    // **1 gesture なので、1回の Undo で全部戻る**
    writer.undo().expect("undo");
    let restored = writer.snapshot();
    assert_eq!(restored.tracks[0].items.len(), 2, "元の並びへ戻る");
    assert_eq!(
        layer_entries(&restored),
        before,
        "Group の LayerId は台帳から外れる(孤児を残さない)"
    );
}

/// **名前は識別子ではない。** 変えても参照は動かず、Undo で元の名前へ戻る。
#[test]
fn renaming_a_layer_changes_only_the_ledger_entry() {
    let f = fixture();
    let mut writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let old_name = before.layers.display_name(f.layer).unwrap().to_owned();
    let tracks_before = before.tracks.clone();

    let command = writer
        .prepare_set_layer_name(f.layer, "renamed")
        .expect("prepare")
        .expect("変化があるので command が出る");
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, command).expect("apply");

    let after = writer.snapshot();
    writer.validate().expect("post-rename document must validate");
    assert_eq!(after.layers.display_name(f.layer), Some("renamed"));
    assert_eq!(after.tracks, tracks_before, "**ツリーは1バイトも動かない**");
    assert_eq!(
        after.layers.len(),
        before.layers.len(),
        "エントリは増えも減りもしない"
    );

    // same-value は command を出さない
    assert!(writer
        .prepare_set_layer_name(f.layer, "renamed")
        .expect("prepare")
        .is_none());

    writer.undo().expect("undo");
    assert_eq!(
        writer.snapshot().layers.display_name(f.layer),
        Some(old_name.as_str()),
        "1回の Undo で元の名前へ"
    );

    // 居ない layer は断る
    assert!(matches!(
        writer.prepare_set_layer_name(LayerId::from_raw(9_999), "x"),
        Err(CommandError::LayerNotFound(_))
    ));
}
