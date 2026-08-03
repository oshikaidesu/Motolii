//! D1l Stage B-3: Writer prepare API 受け入れ正本。

pub mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::identity_roundtrip::assert_identity_command_roundtrip;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::journal::{
    replay_from_base, JournalEdit, JournalFrame, JournalHeader, JournalRecordKind,
    JournalScanOutcome, V2_EDIT_FORMAT_VERSION,
};
use motolii_doc::{
    Clip, ClipSource, Command, CommandError, DocParam, DocValue, Document, DocumentError,
    DocumentPluginError, DocumentWriter, DraftDocParam, DraftKeyframe, EffectDefinition,
    EffectDefinitionDraft, EffectDefinitionId, EffectId, EffectUse, Group, ItemEnvelope, LayerId,
    PrepareError, StableIdError, StableIdReservation, Track, TrackId, TrackItem,
    MIN_READER_VERSION_FOR_COMP_CAMERA, WRITER_VERSION,
};
use motolii_eval::Interp;
use motolii_plugin::reference::reference_catalog;
use serde_json::{Map, Value};

fn reference_writer(doc: Document) -> DocumentWriter {
    DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap()
}

fn collect_keyframe_ids_param(param: &DocParam, out: &mut Vec<u64>) {
    match param {
        DocParam::Const(_)
        | DocParam::Data { .. }
        | DocParam::LookAt { .. }
        | DocParam::Follow { .. } => {}
        DocParam::Keyframes(track) => {
            for key in track.keys() {
                out.push(key.id.get());
            }
        }
        DocParam::Vec2Axes { x, y } => {
            collect_keyframe_ids_param(x, out);
            collect_keyframe_ids_param(y, out);
        }
    }
}

fn introduced_ids_create(use_: &EffectUse, definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![use_.id.get(), definition.id.get()];
    for param in definition.params.values() {
        collect_keyframe_ids_param(param, &mut ids);
    }
    ids
}

fn introduced_ids_link(use_: &EffectUse) -> Vec<u64> {
    vec![use_.id.get()]
}

fn introduced_ids_copy_local(definition: &EffectDefinition) -> Vec<u64> {
    let mut ids = vec![definition.id.get()];
    for param in definition.params.values() {
        collect_keyframe_ids_param(param, &mut ids);
    }
    ids
}

fn doc_param_semantics_match_ignore_id(a: &DocParam, b: &DocParam) -> bool {
    match (a, b) {
        (DocParam::Const(va), DocParam::Const(vb)) => va == vb,
        (
            DocParam::Data {
                track: ta,
                fallback: fa,
            },
            DocParam::Data {
                track: tb,
                fallback: fb,
            },
        ) => ta == tb && fa == fb,
        (
            DocParam::LookAt {
                target: ta,
                axis: aa,
            },
            DocParam::LookAt {
                target: tb,
                axis: ab,
            },
        ) => ta == tb && aa == ab,
        (
            DocParam::Follow {
                target: ta,
                offset: oa,
            },
            DocParam::Follow {
                target: tb,
                offset: ob,
            },
        ) => ta == tb && oa == ob,
        (DocParam::Keyframes(ta), DocParam::Keyframes(tb)) => {
            let ka = ta.keys();
            let kb = tb.keys();
            ka.len() == kb.len()
                && ka
                    .iter()
                    .zip(kb.iter())
                    .all(|(a, b)| a.t == b.t && a.value == b.value && a.interp == b.interp)
        }
        (DocParam::Vec2Axes { x: ax, y: ay }, DocParam::Vec2Axes { x: bx, y: by }) => {
            doc_param_semantics_match_ignore_id(ax, bx)
                && doc_param_semantics_match_ignore_id(ay, by)
        }
        _ => false,
    }
}

fn definition_semantics_match_ignore_id(
    source: &EffectDefinition,
    payload: &EffectDefinition,
) -> bool {
    source.plugin_id == payload.plugin_id
        && source.effect_version == payload.effect_version
        && source.enabled == payload.enabled
        && source.extra == payload.extra
        && source.params.len() == payload.params.len()
        && source.params.iter().all(|(name, src_param)| {
            payload.params.get(name).is_some_and(|payload_param| {
                doc_param_semantics_match_ignore_id(src_param, payload_param)
            })
        })
}

fn layer_track_only_fixture() -> (Document, LayerId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("layer").unwrap();
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
    (doc, layer)
}

struct Fixture {
    doc: Document,
    layer: LayerId,
    use_id: EffectId,
    def_id: EffectDefinitionId,
}

fn v4_fixture() -> Fixture {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("layer").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let use_id = EffectId::from_raw(doc.next_stable_id.allocate().unwrap());
    let def_id = EffectDefinitionId::from_raw(doc.next_stable_id.allocate().unwrap());
    doc.effect_definitions.push(EffectDefinition::new(
        def_id,
        "vendor.filter.fixture",
        1,
        true,
        BTreeMap::from([("amount".into(), DocParam::const_f64(0.5))]),
        Default::default(),
    ));
    let mut env = ItemEnvelope::new(layer);
    env.effects.push(EffectUse {
        id: use_id,
        definition_id: def_id,
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
    doc.validate().unwrap();
    Fixture {
        doc,
        layer,
        use_id,
        def_id,
    }
}

fn nested_create_draft() -> EffectDefinitionDraft {
    let t0 = RationalTime::ZERO;
    let t1 = RationalTime::try_new(1, 1).unwrap();
    EffectDefinitionDraft {
        plugin_id: "core.filter.blur".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([
            (
                "alpha".into(),
                DraftDocParam::Keyframes(vec![
                    DraftKeyframe {
                        t: t0,
                        value: DocValue::F64(0.0),
                        interp: Interp::Hold,
                    },
                    DraftKeyframe {
                        t: t1,
                        value: DocValue::F64(1.0),
                        interp: Interp::Linear,
                    },
                ]),
            ),
            (
                "offset".into(),
                DraftDocParam::Vec2Axes {
                    x: Box::new(DraftDocParam::Keyframes(vec![DraftKeyframe {
                        t: t0,
                        value: DocValue::F64(0.0),
                        interp: Interp::Hold,
                    }])),
                    y: Box::new(DraftDocParam::Keyframes(vec![DraftKeyframe {
                        t: t0,
                        value: DocValue::F64(0.5),
                        interp: Interp::Hold,
                    }])),
                },
            ),
        ]),
        extra: Map::new(),
    }
}

fn exhaust_stable_id_counter(doc: &mut Document) {
    let mut json = serde_json::to_value(&*doc).expect("document json");
    json["next_stable_id"] = serde_json::json!(u64::MAX);
    *doc = serde_json::from_value(json).expect("document with exhausted counter");
}

fn assert_writer_unchanged(writer: &DocumentWriter, before: &Document, revision: u64) {
    assert_eq!(writer.snapshot().as_ref(), before);
    assert_eq!(writer.revision, revision);
    assert_eq!(writer.undo_len(), 0);
    assert_eq!(writer.redo_len(), 0);
}

#[test]
fn prepare_success_leaves_writer_document_revision_undo_redo_unchanged() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc.clone());
    let snap = writer.snapshot();
    let revision = writer.revision;

    let create = writer
        .prepare_create_effect(f.layer, 1, nested_create_draft())
        .expect("create");
    assert_writer_unchanged(&writer, &snap, revision);

    let link = writer
        .prepare_link_effect_use(f.layer, 1, f.def_id)
        .expect("link");
    assert_writer_unchanged(&writer, &snap, revision);

    let copy = writer
        .prepare_copy_local_effect(f.use_id)
        .expect("copy local");
    assert_writer_unchanged(&writer, &snap, revision);

    assert!(create.stable_id_reservation().is_some());
    assert!(link.stable_id_reservation().is_some());
    assert!(copy.stable_id_reservation().is_some());
}

#[test]
fn prepare_create_fixed_allocation_order_and_reservation_closure() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot().next_stable_id.peek_next();
    let cmd = writer
        .prepare_create_effect(f.layer, 0, nested_create_draft())
        .expect("create");

    let Command::CreateEffect {
        use_,
        definition,
        stable_id_reservation,
        ..
    } = cmd
    else {
        panic!("expected CreateEffect");
    };
    assert_eq!(stable_id_reservation.before(), before);
    let introduced = introduced_ids_create(&use_, &definition);
    assert_eq!(
        introduced,
        (before..stable_id_reservation.after()).collect::<Vec<_>>()
    );
    let ordered_kf: Vec<u64> = {
        let mut ids = Vec::new();
        for param in definition.params.values() {
            collect_keyframe_ids_param(param, &mut ids);
        }
        ids
    };
    assert_eq!(
        ordered_kf,
        vec![before + 2, before + 3, before + 4, before + 5]
    );
    assert_eq!(use_.id.get(), before);
    assert_eq!(definition.id.get(), before + 1);
}

#[test]
fn prepare_link_reserves_use_id_only() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot().next_stable_id.peek_next();
    let cmd = writer
        .prepare_link_effect_use(f.layer, 0, f.def_id)
        .expect("link");
    let Command::LinkEffectUse {
        use_,
        stable_id_reservation,
        ..
    } = cmd
    else {
        panic!("expected LinkEffectUse");
    };
    assert_eq!(introduced_ids_link(&use_), vec![before]);
    assert_eq!(
        stable_id_reservation,
        StableIdReservation::new(before, before + 1)
    );
}

#[test]
fn prepare_copy_local_remints_definition_not_use() {
    let (mut doc, layer) = layer_track_only_fixture();
    let seed_writer = reference_writer(doc.clone());
    let seed_cmd = seed_writer
        .prepare_create_effect(layer, 0, nested_create_draft())
        .expect("nested create");
    seed_cmd.apply(&mut doc).expect("apply nested create");
    let Command::CreateEffect {
        use_,
        definition: original_definition,
        ..
    } = seed_cmd
    else {
        panic!("expected CreateEffect");
    };
    let use_id = use_.id;
    let original_def_id = original_definition.id;
    let original_def = doc.effect_definition(original_def_id).unwrap().clone();

    let writer = reference_writer(doc);
    let before = writer.snapshot().next_stable_id.peek_next();
    let copy_cmd = writer.prepare_copy_local_effect(use_id).expect("copy");
    let Command::CopyLocalEffect {
        use_id: copy_use_id,
        previous_definition_id,
        ref new_definition,
        stable_id_reservation,
    } = copy_cmd
    else {
        panic!("expected CopyLocalEffect");
    };
    assert_eq!(copy_use_id, use_id);
    assert_eq!(previous_definition_id, original_def_id);
    assert_eq!(new_definition.id.get(), before);
    let introduced = introduced_ids_copy_local(new_definition);
    assert_eq!(
        introduced,
        (before..stable_id_reservation.after()).collect::<Vec<_>>()
    );
    assert!(
        !introduced.contains(&use_id.get()),
        "existing use id must stay outside copy-local reservation"
    );
    // 辞書順 alpha→offset、alpha Keyframes は格納順、Vec2Axes は x→y。
    let alpha = new_definition
        .params
        .get("alpha")
        .expect("alpha param must exist");
    match alpha {
        DocParam::Keyframes(track) => {
            let ids: Vec<u64> = track.keys().iter().map(|k| k.id.get()).collect();
            assert_eq!(
                ids,
                vec![before + 1, before + 2],
                "dict-order alpha: Keyframes remint IDs in storage order"
            );
        }
        other => panic!("alpha must be Keyframes, got {other:?}"),
    }
    let offset = new_definition
        .params
        .get("offset")
        .expect("offset param must exist");
    match offset {
        DocParam::Vec2Axes { x, y } => {
            match x.as_ref() {
                DocParam::Keyframes(track) => {
                    let ids: Vec<u64> = track.keys().iter().map(|k| k.id.get()).collect();
                    assert_eq!(
                        ids,
                        vec![before + 3],
                        "dict-order offset after alpha; Vec2Axes x before y"
                    );
                }
                other => panic!("offset.x must be Keyframes, got {other:?}"),
            }
            match y.as_ref() {
                DocParam::Keyframes(track) => {
                    let ids: Vec<u64> = track.keys().iter().map(|k| k.id.get()).collect();
                    assert_eq!(ids, vec![before + 4], "Vec2Axes y after x in remint order");
                }
                other => panic!("offset.y must be Keyframes, got {other:?}"),
            }
        }
        other => panic!("offset must be Vec2Axes, got {other:?}"),
    }
    assert!(definition_semantics_match_ignore_id(
        &original_def,
        new_definition
    ));
    assert_identity_command_roundtrip(writer.snapshot().as_ref(), copy_cmd);
}

#[test]
fn prepared_commands_satisfy_identity_roundtrip() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc.clone());
    let snap = writer.snapshot();

    for cmd in [
        writer
            .prepare_create_effect(f.layer, 0, nested_create_draft())
            .unwrap(),
        writer
            .prepare_link_effect_use(f.layer, 0, f.def_id)
            .unwrap(),
        writer.prepare_copy_local_effect(f.use_id).unwrap(),
    ] {
        assert_identity_command_roundtrip(&snap, cmd);
    }
}

#[test]
fn prepare_rejects_non_current_writer_contract() {
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Map::new(),
    };

    for (version, min) in [(1, 1), (2, 2), (3, 3), (4, 4)] {
        let mut doc = Document::new_current();
        doc.version = version;
        doc.min_reader_version = min;
        let err = DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap_err();
        assert!(
            matches!(
                err,
                DocumentPluginError::Structural(
                    DocumentError::CompCameraDisguisedOldVersion { .. }
                )
            ),
            "v{version} min={min}: {err:?}"
        );
    }

    let mut sub_floor = Document::new_current();
    sub_floor.min_reader_version = MIN_READER_VERSION_FOR_COMP_CAMERA - 1;
    let err = DocumentWriter::new(sub_floor, Arc::new(reference_catalog().unwrap())).unwrap_err();
    assert!(
        matches!(
            err,
            DocumentPluginError::Structural(DocumentError::CompCameraRequiresNewerReader { .. })
        ),
        "{err:?}"
    );

    let mut future = Document::new_current();
    future.version = WRITER_VERSION + 1;
    let writer = reference_writer(future);
    let err = writer
        .prepare_create_effect(LayerId::from_raw(0), 0, draft.clone())
        .unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Command(CommandError::EffectLifecycleRequiresV4Document { .. })
    ));
    assert!(writer
        .prepare_link_effect_use(LayerId::from_raw(0), 0, EffectDefinitionId::from_raw(0))
        .is_err());
    assert!(writer
        .prepare_copy_local_effect(EffectId::from_raw(0))
        .is_err());
}

#[test]
fn writer_constructor_rejects_intrinsically_invalid_document() {
    let mut f = v4_fixture();
    f.doc.tracks[0].id = TrackId::from_raw(99);
    let err = DocumentWriter::new(f.doc, Arc::new(reference_catalog().unwrap())).unwrap_err();
    assert!(matches!(
        err,
        DocumentPluginError::Structural(DocumentError::UnknownTrackId { id: 99 })
    ));
}

#[test]
fn prepare_rejects_missing_target_and_bad_index() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let revision = writer.revision;
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Map::new(),
    };

    let err = writer
        .prepare_create_effect(LayerId::from_raw(999), 0, draft.clone())
        .unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Command(CommandError::LayerNotFound(999))
    ));
    assert_writer_unchanged(&writer, &before, revision);

    let err = writer
        .prepare_create_effect(f.layer, 99, draft)
        .unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Command(CommandError::IndexOutOfRange { index: 99, len: 1 })
    ));
    assert_writer_unchanged(&writer, &before, revision);
}

#[test]
fn prepare_link_rejects_missing_definition() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let err = writer
        .prepare_link_effect_use(f.layer, 0, EffectDefinitionId::from_raw(999))
        .unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Command(CommandError::EffectDefinitionNotFound { id: 999 })
    ));
    assert_writer_unchanged(&writer, &before, writer.revision);
}

#[test]
fn prepare_copy_local_rejects_missing_use() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let err = writer
        .prepare_copy_local_effect(EffectId::from_raw(999))
        .unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Command(CommandError::EffectUseNotFound { use_id: 999 })
    ));
    assert_writer_unchanged(&writer, &before, writer.revision);
}

#[test]
fn prepare_create_rejects_duplicate_keyframe_times_without_consuming_ids() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before_counter = writer.snapshot().next_stable_id.peek_next();
    let snap = writer.snapshot();
    let t = RationalTime::ZERO;
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([(
            "amount".into(),
            DraftDocParam::Keyframes(vec![
                DraftKeyframe {
                    t,
                    value: DocValue::F64(0.0),
                    interp: Interp::Hold,
                },
                DraftKeyframe {
                    t,
                    value: DocValue::F64(1.0),
                    interp: Interp::Hold,
                },
            ]),
        )]),
        extra: Map::new(),
    };
    let err = writer.prepare_create_effect(f.layer, 0, draft).unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Keyframe(motolii_doc::DocKeyframeError::UnsortedOrDuplicateKeys)
    ));
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), before_counter);
    assert_eq!(writer.snapshot().as_ref(), snap.as_ref());
}

#[test]
fn prepare_create_rejects_unsorted_keyframe_times_without_consuming_ids() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before_counter = writer.snapshot().next_stable_id.peek_next();
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([(
            "amount".into(),
            DraftDocParam::Keyframes(vec![
                DraftKeyframe {
                    t: RationalTime::try_new(2, 1).unwrap(),
                    value: DocValue::F64(1.0),
                    interp: Interp::Hold,
                },
                DraftKeyframe {
                    t: RationalTime::try_new(1, 1).unwrap(),
                    value: DocValue::F64(0.0),
                    interp: Interp::Hold,
                },
            ]),
        )]),
        extra: Map::new(),
    };
    let err = writer.prepare_create_effect(f.layer, 0, draft).unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Keyframe(motolii_doc::DocKeyframeError::UnsortedOrDuplicateKeys)
    ));
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), before_counter);
}

#[test]
fn prepare_create_rejects_invalid_interp_without_consuming_ids() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before_counter = writer.snapshot().next_stable_id.peek_next();
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([(
            "amount".into(),
            DraftDocParam::Keyframes(vec![DraftKeyframe {
                t: RationalTime::ZERO,
                value: DocValue::F64(0.0),
                interp: Interp::Bezier {
                    x1: 2.0,
                    y1: 0.0,
                    x2: 0.0,
                    y2: 0.0,
                },
            }]),
        )]),
        extra: Map::new(),
    };
    let err = writer.prepare_create_effect(f.layer, 0, draft).unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Keyframe(motolii_doc::DocKeyframeError::InvalidBezier { .. })
    ));
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), before_counter);
}

#[test]
fn prepare_create_rejects_non_finite_value_without_consuming_ids() {
    let f = v4_fixture();
    let writer = reference_writer(f.doc);
    let before_counter = writer.snapshot().next_stable_id.peek_next();
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::from([(
            "amount".into(),
            DraftDocParam::Keyframes(vec![DraftKeyframe {
                t: RationalTime::ZERO,
                value: DocValue::F64(f64::NAN),
                interp: Interp::Hold,
            }]),
        )]),
        extra: Map::new(),
    };
    let err = writer.prepare_create_effect(f.layer, 0, draft).unwrap_err();
    assert!(matches!(
        err,
        PrepareError::Validate(DocumentError::NonFiniteValue { .. })
    ));
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), before_counter);
}

#[test]
fn prepare_create_rejects_stable_id_exhaustion_without_mutation() {
    let mut f = v4_fixture();
    exhaust_stable_id_counter(&mut f.doc);
    let writer = reference_writer(f.doc);
    let before = writer.snapshot();
    let draft = EffectDefinitionDraft {
        plugin_id: "p".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Map::new(),
    };
    let err = writer.prepare_create_effect(f.layer, 0, draft).unwrap_err();
    assert!(matches!(
        err,
        PrepareError::StableId(StableIdError::Exhausted)
    ));
    assert_writer_unchanged(&writer, &before, writer.revision);
}

#[test]
fn new_current_contract_matches_prepare_gate() {
    let doc = Document::new_current();
    assert_eq!(doc.version, WRITER_VERSION);
    assert_eq!(doc.min_reader_version, MIN_READER_VERSION_FOR_COMP_CAMERA);
    let writer = reference_writer(doc);
    assert!(writer.validate().is_ok());
}

fn clip_at(doc: &Document, layer: LayerId) -> &Clip {
    let TrackItem::Clip(clip) = find_track_item(doc, layer) else {
        panic!("expected clip");
    };
    clip
}

fn find_track_item(doc: &Document, layer: LayerId) -> &TrackItem {
    doc.tracks
        .iter()
        .flat_map(|track| track.items.iter())
        .find(|item| match item {
            TrackItem::Clip(c) => c.envelope.layer_id == layer,
            TrackItem::Group(g) => g.envelope.layer_id == layer,
        })
        .unwrap_or_else(|| panic!("layer {} not found", layer.get()))
}

fn find_track_item_mut(doc: &mut Document, layer: LayerId) -> Option<&mut TrackItem> {
    for track in &mut doc.tracks {
        for item in &mut track.items {
            let current_layer = match item {
                TrackItem::Clip(clip) => clip.envelope.layer_id,
                TrackItem::Group(group) => group.envelope.layer_id,
            };
            if current_layer == layer {
                return Some(item);
            }
            if let TrackItem::Group(group) = item {
                if let Some(found) = find_track_item_mut_in_group(group, layer) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn find_track_item_mut_in_group(group: &mut Group, layer: LayerId) -> Option<&mut TrackItem> {
    for item in &mut group.children {
        let current_layer = match item {
            TrackItem::Clip(clip) => clip.envelope.layer_id,
            TrackItem::Group(child_group) => child_group.envelope.layer_id,
        };
        if current_layer == layer {
            return Some(item);
        }
        if let TrackItem::Group(child) = item {
            if let Some(found) = find_track_item_mut_in_group(child, layer) {
                return Some(found);
            }
        }
    }
    None
}

fn group_fixture() -> (Document, LayerId) {
    let mut doc = Document::new_current();
    let group_layer = doc.layers.allocate("group").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Group(Group {
            envelope: ItemEnvelope::new(group_layer),
            children: vec![],
        })],
    });
    doc.validate().unwrap();
    (doc, group_layer)
}

fn overlap_fixture() -> (Document, LayerId, LayerId) {
    let mut doc = Document::new_current();
    let layer_a = doc.layers.allocate("a").unwrap();
    let layer_b = doc.layers.allocate("b").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer_a),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer_b),
                start: RationalTime::try_new(2, 1).unwrap(),
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            }),
        ],
    });
    doc.validate().unwrap();
    (doc, layer_a, layer_b)
}

fn replay_single_edit(base: Document, edit: &JournalEdit) -> Document {
    let payload = serde_json::to_vec(edit).expect("journal edit json");
    let scan = JournalScanOutcome {
        header: JournalHeader {
            version: 1,
            generation_salt: 1,
            project_id: uuid::Uuid::new_v4(),
        },
        frames: vec![JournalFrame {
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
    let outcome = replay_from_base(base, &scan, &mut |_| unreachable!(), false);
    assert!(
        outcome.replay_failures.is_empty(),
        "failures={:?}",
        outcome.replay_failures
    );
    outcome.document
}

#[test]
fn prepare_set_clip_start_leaves_writer_unchanged() {
    let (doc, layer) = layer_track_only_fixture();
    let writer = reference_writer(doc);
    let snap = writer.snapshot();
    let revision = writer.revision;
    let new = RationalTime::try_new(2, 1).unwrap();
    let cmd = writer
        .prepare_set_clip_start(layer, new)
        .expect("prepare")
        .expect("command");
    assert_writer_unchanged(&writer, &snap, revision);
    assert!(cmd.stable_id_reservation().is_none());
    let Command::SetClipStart {
        old,
        new: prepared_new,
        ..
    } = cmd
    else {
        panic!("expected SetClipStart");
    };
    assert_eq!(old, RationalTime::ZERO);
    assert_eq!(prepared_new, new);
}

#[test]
fn set_clip_start_changes_only_start() {
    let (doc, layer) = layer_track_only_fixture();
    let before = doc.clone();
    let new = RationalTime::try_new(2, 1).unwrap();
    let writer = reference_writer(doc);
    let cmd = writer.prepare_set_clip_start(layer, new).unwrap().unwrap();
    let mut working = before.clone();
    cmd.apply(&mut working).unwrap();
    let before_clip = clip_at(&before, layer);
    let after_clip = clip_at(&working, layer);
    assert_eq!(after_clip.start, new);
    assert_eq!(after_clip.duration, before_clip.duration);
    assert_eq!(after_clip.time_map, before_clip.time_map);
    assert_eq!(after_clip.source, before_clip.source);
    assert_eq!(after_clip.envelope, before_clip.envelope);
    let mut expected = before.clone();
    if let TrackItem::Clip(c) = &mut expected.tracks[0].items[0] {
        c.start = new;
    }
    assert_eq!(working, expected);
}

#[test]
fn set_clip_start_negative_overlap_and_end_at_composition_succeed() {
    let (doc, layer_a, layer_b) = overlap_fixture();
    let comp_duration = doc.composition.duration;
    let duration = RationalTime::try_new(5, 1).unwrap();
    let end_at_comp_start = comp_duration.try_sub(duration).unwrap();

    for (target, new) in [
        (layer_a, RationalTime::try_new(-1, 1).unwrap()),
        (layer_b, RationalTime::try_new(4, 1).unwrap()),
        (layer_a, end_at_comp_start),
    ] {
        let mut working = doc.clone();
        let cmd = Command::SetClipStart {
            target,
            old: clip_at(&doc, target).start,
            new,
        };
        cmd.apply(&mut working).unwrap();
        assert_eq!(clip_at(&working, target).start, new);
        working.validate().unwrap();
    }
}

#[test]
fn set_clip_start_rejects_in_precedence_order_without_mutation() {
    let (doc, layer) = layer_track_only_fixture();
    let duration = RationalTime::try_new(5, 1).unwrap();
    let comp = doc.composition.duration;
    let past_start = comp
        .try_sub(duration)
        .unwrap()
        .try_add(RationalTime::try_new(1, 100).unwrap())
        .unwrap();
    let overflow_start = RationalTime::try_new(i64::MAX, 1).unwrap();
    let (group_doc, group_layer) = group_fixture();

    struct Case<'a> {
        label: &'a str,
        doc: Document,
        target: LayerId,
        new: RationalTime,
    }

    let cases = [
        Case {
            label: "missing",
            doc: doc.clone(),
            target: LayerId::from_raw(999),
            new: RationalTime::ZERO,
        },
        Case {
            label: "group",
            doc: group_doc,
            target: group_layer,
            new: RationalTime::ZERO,
        },
        Case {
            label: "overflow",
            doc: doc.clone(),
            target: layer,
            new: overflow_start,
        },
        Case {
            label: "past composition",
            doc,
            target: layer,
            new: past_start,
        },
    ];

    for case in cases {
        let before = case.doc.clone();
        let writer = reference_writer(case.doc);
        let err = writer
            .prepare_set_clip_start(case.target, case.new)
            .unwrap_err();
        match case.label {
            "missing" => assert!(matches!(err, CommandError::LayerNotFound(999)), "{err:?}"),
            "group" => assert!(
                matches!(err, CommandError::TrackItemNotClip { layer } if layer == group_layer.get()),
                "{err:?}"
            ),
            "overflow" => assert!(
                matches!(
                    err,
                    CommandError::Validate(DocumentError::ClipIntervalOverflow { layer_id })
                    if layer_id == layer.get()
                ),
                "{err:?}"
            ),
            "past composition" => assert!(
                matches!(
                    err,
                    CommandError::Validate(DocumentError::ClipPastComposition { layer_id, .. })
                    if layer_id == layer.get()
                ),
                "{err:?}"
            ),
            _ => unreachable!(),
        }
        assert_eq!(writer.snapshot().as_ref(), &before, "{}", case.label);
        assert_eq!(writer.revision, 0, "{}", case.label);

        let mut direct = before.clone();
        let err = Command::SetClipStart {
            target: case.target,
            old: RationalTime::ZERO,
            new: case.new,
        }
        .apply(&mut direct)
        .unwrap_err();
        match case.label {
            "missing" => assert!(matches!(err, CommandError::LayerNotFound(999)), "{err:?}"),
            "group" => assert!(
                matches!(err, CommandError::TrackItemNotClip { layer } if layer == group_layer.get()),
                "{err:?}"
            ),
            "overflow" => assert!(
                matches!(
                    err,
                    CommandError::Validate(DocumentError::ClipIntervalOverflow { layer_id })
                    if layer_id == layer.get()
                ),
                "{err:?}"
            ),
            "past composition" => assert!(
                matches!(
                    err,
                    CommandError::Validate(DocumentError::ClipPastComposition { layer_id, .. })
                    if layer_id == layer.get()
                ),
                "{err:?}"
            ),
            _ => unreachable!(),
        }
        assert_eq!(direct, before, "{}", case.label);
    }
}

#[test]
fn prepare_set_clip_start_same_value_returns_none() {
    let (doc, layer) = layer_track_only_fixture();
    let writer = reference_writer(doc);
    let current = clip_at(writer.snapshot().as_ref(), layer).start;
    assert!(writer
        .prepare_set_clip_start(layer, current)
        .unwrap()
        .is_none());
}

#[test]
fn set_clip_start_same_value_raw_apply_is_identity() {
    let (doc, layer) = layer_track_only_fixture();
    let mut working = doc.clone();
    let current = clip_at(&doc, layer).start;
    Command::SetClipStart {
        target: layer,
        old: current,
        new: current,
    }
    .apply(&mut working)
    .unwrap();
    assert_eq!(working, doc);
}

#[test]
fn set_clip_start_non_cas_raw_old_writes_new_and_inverse_swaps_payload() {
    let (doc, layer) = layer_track_only_fixture();
    let new = RationalTime::try_new(2, 1).unwrap();
    let stale_old = RationalTime::try_new(1, 1).unwrap();
    let cmd = Command::SetClipStart {
        target: layer,
        old: stale_old,
        new,
    };
    let mut working = doc.clone();
    cmd.apply(&mut working).unwrap();
    assert_eq!(clip_at(&working, layer).start, new);

    cmd.inverse().apply(&mut working).unwrap();
    assert_eq!(clip_at(&working, layer).start, stale_old);
}

#[test]
fn writer_prepared_set_clip_start_undo_redo_restores_exact_values() {
    let (doc, layer) = layer_track_only_fixture();
    let new = RationalTime::try_new(3, 1).unwrap();
    let mut writer = reference_writer(doc.clone());
    let cmd = writer.prepare_set_clip_start(layer, new).unwrap().unwrap();
    let gesture = writer.begin_gesture();
    writer.apply_command(gesture, cmd).unwrap();
    assert_eq!(clip_at(writer.snapshot().as_ref(), layer).start, new);

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &doc);

    writer.redo().unwrap();
    assert_eq!(clip_at(writer.snapshot().as_ref(), layer).start, new);
}

#[test]
fn set_clip_start_same_gesture_merge_first_old_last_new() {
    let (doc, layer_a, layer_b) = overlap_fixture();
    let mut writer = reference_writer(doc.clone());
    let gesture = writer.begin_gesture();
    let a_old = clip_at(&doc, layer_a).start;
    let b_old = clip_at(&doc, layer_b).start;
    let a_mid = RationalTime::try_new(1, 1).unwrap();
    let a_new = RationalTime::try_new(2, 1).unwrap();
    let b_new = RationalTime::try_new(3, 1).unwrap();

    writer
        .apply_command(
            gesture,
            Command::SetClipStart {
                target: layer_a,
                old: a_old,
                new: a_mid,
            },
        )
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::SetClipStart {
                target: layer_a,
                old: a_mid,
                new: a_new,
            },
        )
        .unwrap();
    writer
        .apply_command(
            gesture,
            Command::SetClipStart {
                target: layer_b,
                old: b_old,
                new: b_new,
            },
        )
        .unwrap();

    assert_eq!(writer.undo_len(), 1);
    assert_eq!(clip_at(writer.snapshot().as_ref(), layer_a).start, a_new);
    assert_eq!(clip_at(writer.snapshot().as_ref(), layer_b).start, b_new);

    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &doc);
}

#[test]
fn set_clip_start_journal_v2_roundtrip_and_replay() {
    let (doc, layer) = layer_track_only_fixture();
    let new = RationalTime::try_new(2, 1).unwrap();
    let cmd = reference_writer(doc.clone())
        .prepare_set_clip_start(layer, new)
        .unwrap()
        .unwrap();
    let edit = JournalEdit::new(cmd);
    assert_eq!(edit.format_version, V2_EDIT_FORMAT_VERSION);
    let json = serde_json::to_string(&edit).unwrap();
    assert!(json.contains("SetClipStart"));
    let decoded: JournalEdit = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, edit);
    assert!(decoded.command.stable_id_reservation().is_none());

    let replayed = replay_single_edit(doc, &edit);
    assert_eq!(clip_at(&replayed, layer).start, new);
}

#[test]
fn set_clip_start_preserves_document_version_and_stable_ids() {
    let (doc, layer) = layer_track_only_fixture();
    let version = doc.version;
    let min_reader = doc.min_reader_version;
    let counter = doc.next_stable_id.peek_next();
    let new = RationalTime::try_new(1, 1).unwrap();
    let mut working = doc;
    reference_writer(working.clone())
        .prepare_set_clip_start(layer, new)
        .unwrap()
        .unwrap()
        .apply(&mut working)
        .unwrap();
    assert_eq!(working.version, version);
    assert_eq!(working.min_reader_version, min_reader);
    assert_eq!(working.next_stable_id.peek_next(), counter);
}

#[test]
fn set_clip_start_random_valid_sequence_undo_restores_document() {
    let (doc, layer) = layer_track_only_fixture();
    let duration = RationalTime::try_new(5, 1).unwrap();
    let comp = doc.composition.duration;
    let max_start = comp.try_sub(duration).unwrap();
    let candidates = [
        RationalTime::try_new(-2, 1).unwrap(),
        RationalTime::ZERO,
        RationalTime::try_new(1, 1).unwrap(),
        RationalTime::try_new(3, 2).unwrap(),
        max_start,
    ];
    let mut writer = reference_writer(doc.clone());
    let gesture = writer.begin_gesture();
    for new in candidates {
        assert!(new.try_add(duration).unwrap() <= comp);
        let cmd = writer.prepare_set_clip_start(layer, new).unwrap().unwrap();
        writer.apply_command(gesture, cmd).unwrap();
    }
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &doc);
}

#[test]
fn trim_clip_in_shrink_and_extend_preserve_right_edge_and_source_mapping() {
    let (mut doc, layer) = layer_track_only_fixture();
    let time_map = TimeMap::constant_speed(RationalTime::try_new(1, 1).unwrap(), 3, 2).unwrap();
    let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] else {
        unreachable!();
    };
    clip.time_map = time_map;
    let original = clip.clone();
    let old_end = original.start.try_add(original.duration).unwrap();

    for new_start in [
        RationalTime::try_new(1, 1).unwrap(),
        RationalTime::try_new(-1, 1).unwrap(),
    ] {
        let writer = reference_writer(doc.clone());
        let cmd = writer
            .prepare_trim_clip_in(layer, new_start)
            .unwrap()
            .unwrap();
        assert!(cmd.stable_id_reservation().is_none());
        let mut working = doc.clone();
        cmd.apply(&mut working).unwrap();
        let changed = clip_at(&working, layer);
        assert_eq!(changed.start, new_start);
        assert_eq!(changed.start.try_add(changed.duration).unwrap(), old_end);
        assert_eq!(changed.time_map.speed_num(), original.time_map.speed_num());
        assert_eq!(changed.time_map.speed_den(), original.time_map.speed_den());
        assert_eq!(
            changed.time_map.overrun_mode,
            original.time_map.overrun_mode
        );
        for timeline_time in [new_start, RationalTime::try_new(2, 1).unwrap(), old_end] {
            let old_local = timeline_time.try_sub(original.start).unwrap();
            let new_local = timeline_time.try_sub(changed.start).unwrap();
            assert_eq!(
                original.time_map.try_map(old_local).unwrap(),
                changed.time_map.try_map(new_local).unwrap()
            );
        }
        assert_eq!(changed.source, original.source);
        assert_eq!(changed.envelope, original.envelope);
        working.validate().unwrap();
    }
}

#[test]
fn trim_clip_out_shrink_and_extend_change_only_duration() {
    let (mut doc, layer) = layer_track_only_fixture();
    let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] else {
        unreachable!();
    };
    clip.start = RationalTime::try_new(-1, 1).unwrap();
    clip.time_map = TimeMap::constant_speed(RationalTime::try_new(2, 1).unwrap(), 2, 1).unwrap();
    let before = clip.clone();

    for new_end in [
        RationalTime::try_new(2, 1).unwrap(),
        RationalTime::try_new(6, 1).unwrap(),
    ] {
        let cmd = reference_writer(doc.clone())
            .prepare_trim_clip_out(layer, new_end)
            .unwrap()
            .unwrap();
        let mut working = doc.clone();
        cmd.apply(&mut working).unwrap();
        let changed = clip_at(&working, layer);
        assert_eq!(changed.start, before.start);
        assert_eq!(changed.time_map, before.time_map);
        assert_eq!(changed.duration, new_end.try_sub(before.start).unwrap());
        assert_eq!(changed.source, before.source);
        assert_eq!(changed.envelope, before.envelope);
        working.validate().unwrap();
    }
}

#[test]
fn trim_clip_prepare_same_edges_return_none() {
    let (doc, layer) = layer_track_only_fixture();
    let writer = reference_writer(doc);
    let snapshot = writer.snapshot();
    let clip = clip_at(snapshot.as_ref(), layer);
    let end = clip.start.try_add(clip.duration).unwrap();
    assert!(writer
        .prepare_trim_clip_in(layer, clip.start)
        .unwrap()
        .is_none());
    assert!(writer.prepare_trim_clip_out(layer, end).unwrap().is_none());
}

#[test]
fn trim_clip_rejects_targets_boundaries_and_corrupt_payload_without_mutation() {
    let (doc, layer) = layer_track_only_fixture();
    let (group_doc, group_layer) = group_fixture();
    for prepare_in in [true, false] {
        let writer = reference_writer(doc.clone());
        let missing = if prepare_in {
            writer
                .prepare_trim_clip_in(LayerId::from_raw(999), RationalTime::ZERO)
                .unwrap_err()
        } else {
            writer
                .prepare_trim_clip_out(LayerId::from_raw(999), RationalTime::ZERO)
                .unwrap_err()
        };
        assert!(matches!(missing, CommandError::LayerNotFound(999)));

        let group_writer = reference_writer(group_doc.clone());
        let group = if prepare_in {
            group_writer
                .prepare_trim_clip_in(group_layer, RationalTime::ZERO)
                .unwrap_err()
        } else {
            group_writer
                .prepare_trim_clip_out(group_layer, RationalTime::ZERO)
                .unwrap_err()
        };
        assert!(
            matches!(group, CommandError::TrackItemNotClip { layer } if layer == group_layer.get())
        );
    }

    let writer = reference_writer(doc.clone());
    let start = clip_at(&doc, layer).start;
    let end = start.try_add(clip_at(&doc, layer).duration).unwrap();
    for err in [
        writer.prepare_trim_clip_in(layer, end).unwrap_err(),
        writer.prepare_trim_clip_out(layer, start).unwrap_err(),
    ] {
        assert!(matches!(
            err,
            CommandError::Validate(DocumentError::NonPositiveClipDuration { layer_id })
            if layer_id == layer.get()
        ));
    }
    let past = doc
        .composition
        .duration
        .try_add(RationalTime::try_new(1, 1).unwrap())
        .unwrap();
    assert!(matches!(
        writer.prepare_trim_clip_out(layer, past).unwrap_err(),
        CommandError::Validate(DocumentError::ClipPastComposition { layer_id, .. })
        if layer_id == layer.get()
    ));

    let valid = writer
        .prepare_trim_clip_in(layer, RationalTime::try_new(1, 1).unwrap())
        .unwrap()
        .unwrap();
    let Command::TrimClipIn {
        target,
        old_start,
        old_duration,
        old_time_map,
        new_start,
        new_duration,
        ..
    } = valid
    else {
        unreachable!();
    };
    let corrupt = Command::TrimClipIn {
        target,
        old_start,
        old_duration,
        old_time_map,
        new_start,
        new_duration,
        new_time_map: old_time_map,
    };
    let mut working = doc.clone();
    assert!(matches!(
        corrupt.apply(&mut working).unwrap_err(),
        CommandError::InvalidClipTrim { layer: invalid } if invalid == layer.get()
    ));
    assert_eq!(working, doc);
}

#[test]
fn trim_clip_non_cas_inverse_merge_and_replay_use_absolute_payloads() {
    let (doc, layer) = layer_track_only_fixture();
    let prepared = reference_writer(doc.clone())
        .prepare_trim_clip_in(layer, RationalTime::try_new(1, 1).unwrap())
        .unwrap()
        .unwrap();
    let mut divergent = doc.clone();
    let TrackItem::Clip(clip) = &mut divergent.tracks[0].items[0] else {
        unreachable!();
    };
    clip.start = RationalTime::try_new(-1, 1).unwrap();
    clip.duration = RationalTime::try_new(6, 1).unwrap();
    prepared.apply(&mut divergent).unwrap();
    let after = divergent.clone();
    prepared.inverse().apply(&mut divergent).unwrap();
    assert_eq!(divergent, doc);

    let edit = JournalEdit::new(prepared.clone());
    let json = serde_json::to_string(&edit).unwrap();
    assert!(json.contains("TrimClipIn"));
    let decoded: JournalEdit = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, edit);
    let replayed = replay_single_edit(doc.clone(), &edit);
    assert_eq!(replayed, after);

    let mut writer = reference_writer(doc.clone());
    let gesture = writer.begin_gesture();
    let first = writer
        .prepare_trim_clip_out(layer, RationalTime::try_new(4, 1).unwrap())
        .unwrap()
        .unwrap();
    writer.apply_command(gesture, first).unwrap();
    let second = writer
        .prepare_trim_clip_out(layer, RationalTime::try_new(3, 1).unwrap())
        .unwrap()
        .unwrap();
    writer.apply_command(gesture, second).unwrap();
    assert_eq!(writer.undo_len(), 1);
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &doc);
    writer.redo().unwrap();
    assert_eq!(
        clip_at(writer.snapshot().as_ref(), layer).duration,
        RationalTime::try_new(3, 1).unwrap()
    );
}

#[test]
fn trim_clip_arithmetic_overflow_precedes_duration_rejection() {
    let (mut in_doc, in_layer) = layer_track_only_fixture();
    let TrackItem::Clip(in_clip) = &mut in_doc.tracks[0].items[0] else {
        unreachable!();
    };
    in_clip.time_map = TimeMap::constant_speed(RationalTime::ZERO, 2, 1).unwrap();
    assert!(matches!(
        reference_writer(in_doc)
            .prepare_trim_clip_in(in_layer, RationalTime::try_new(i64::MAX, 1).unwrap())
            .unwrap_err(),
        CommandError::Validate(DocumentError::ClipIntervalOverflow { layer_id })
        if layer_id == in_layer.get()
    ));

    let (mut out_doc, out_layer) = layer_track_only_fixture();
    let TrackItem::Clip(out_clip) = &mut out_doc.tracks[0].items[0] else {
        unreachable!();
    };
    out_clip.start = RationalTime::try_new(-1, 1).unwrap();
    assert!(matches!(
        reference_writer(out_doc)
            .prepare_trim_clip_out(out_layer, RationalTime::try_new(i64::MAX, 1).unwrap())
            .unwrap_err(),
        CommandError::Validate(DocumentError::ClipIntervalOverflow { layer_id })
        if layer_id == out_layer.get()
    ));
}

#[test]
fn trim_clip_raw_same_value_is_identity_and_merge_keys_keep_edges_separate() {
    let (doc, layer) = layer_track_only_fixture();
    let clip = clip_at(&doc, layer);
    let same_in = Command::TrimClipIn {
        target: layer,
        old_start: clip.start,
        old_duration: clip.duration,
        old_time_map: clip.time_map,
        new_start: clip.start,
        new_duration: clip.duration,
        new_time_map: clip.time_map,
    };
    let same_out = Command::TrimClipOut {
        target: layer,
        old_duration: clip.duration,
        new_duration: clip.duration,
    };
    for command in [&same_in, &same_out] {
        let mut working = doc.clone();
        command.apply(&mut working).unwrap();
        assert_eq!(working, doc);
    }

    let gesture = motolii_doc::GestureId::from_raw(9);
    let move_command = Command::SetClipStart {
        target: layer,
        old: clip.start,
        new: clip.start,
    };
    assert_ne!(same_in.merge_key(gesture), same_out.merge_key(gesture));
    assert_ne!(same_in.merge_key(gesture), move_command.merge_key(gesture));

    let other_in = Command::TrimClipIn {
        target: LayerId::from_raw(layer.get() + 1),
        old_start: clip.start,
        old_duration: clip.duration,
        old_time_map: clip.time_map,
        new_start: clip.start,
        new_duration: clip.duration,
        new_time_map: clip.time_map,
    };
    assert_ne!(same_in.merge_key(gesture), other_in.merge_key(gesture));

    let mut writer = reference_writer(doc.clone());
    let first = writer
        .prepare_trim_clip_in(layer, RationalTime::try_new(1, 1).unwrap())
        .unwrap()
        .unwrap();
    writer.apply_command(gesture, first).unwrap();
    let second = writer
        .prepare_trim_clip_in(layer, RationalTime::try_new(2, 1).unwrap())
        .unwrap()
        .unwrap();
    writer.apply_command(gesture, second).unwrap();
    assert_eq!(writer.undo_len(), 1);
    writer.undo().unwrap();
    assert_eq!(writer.snapshot().as_ref(), &doc);
}

#[test]
fn trim_clip_out_v2_replay_and_both_edges_preserve_versions_and_ids() {
    let (doc, layer) = layer_track_only_fixture();
    let version = doc.version;
    let min_reader = doc.min_reader_version;
    let counter = doc.next_stable_id.peek_next();
    let in_command = reference_writer(doc.clone())
        .prepare_trim_clip_in(layer, RationalTime::try_new(1, 1).unwrap())
        .unwrap()
        .unwrap();
    let out_command = reference_writer(doc.clone())
        .prepare_trim_clip_out(layer, RationalTime::try_new(4, 1).unwrap())
        .unwrap()
        .unwrap();

    for (tag, command) in [("TrimClipIn", in_command), ("TrimClipOut", out_command)] {
        let edit = JournalEdit::new(command);
        assert_eq!(edit.format_version, V2_EDIT_FORMAT_VERSION);
        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains(tag));
        let decoded: JournalEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, edit);
        assert!(decoded.command.stable_id_reservation().is_none());
        let replayed = replay_single_edit(doc.clone(), &edit);
        assert_eq!(replayed.version, version);
        assert_eq!(replayed.min_reader_version, min_reader);
        assert_eq!(replayed.next_stable_id.peek_next(), counter);
    }
}

#[test]
fn trim_clip_allows_overlap_and_composition_end_boundary() {
    let (doc, layer_a, layer_b) = overlap_fixture();
    let comp = doc.composition.duration;
    let a_end = clip_at(&doc, layer_a)
        .start
        .try_add(clip_at(&doc, layer_a).duration)
        .unwrap();
    let mut in_working = doc.clone();
    reference_writer(doc.clone())
        .prepare_trim_clip_in(layer_a, RationalTime::try_new(4, 1).unwrap())
        .unwrap()
        .unwrap()
        .apply(&mut in_working)
        .unwrap();
    assert_eq!(
        clip_at(&in_working, layer_a)
            .start
            .try_add(clip_at(&in_working, layer_a).duration)
            .unwrap(),
        a_end
    );
    in_working.validate().unwrap();

    let mut out_working = doc.clone();
    reference_writer(doc)
        .prepare_trim_clip_out(layer_b, comp)
        .unwrap()
        .unwrap()
        .apply(&mut out_working)
        .unwrap();
    assert_eq!(
        clip_at(&out_working, layer_b)
            .start
            .try_add(clip_at(&out_working, layer_b).duration)
            .unwrap(),
        comp
    );
    out_working.validate().unwrap();
}

#[test]
fn trim_clip_raw_rejection_precedence_conflicts_do_not_mutate_document() {
    let (group_doc, group_layer) = group_fixture();
    let mut group_working = group_doc.clone();
    let group_in = Command::TrimClipIn {
        target: group_layer,
        old_start: RationalTime::try_new(0, 1).unwrap(),
        old_duration: RationalTime::try_new(1, 1).unwrap(),
        old_time_map: TimeMap::try_new(RationalTime::ZERO, 2, 1, motolii_core::OverrunMode::Freeze)
            .unwrap(),
        new_start: RationalTime::try_new(i64::MAX, 1).unwrap(),
        new_duration: RationalTime::try_new(1, 1).unwrap(),
        new_time_map: TimeMap::try_new(RationalTime::ZERO, 2, 1, motolii_core::OverrunMode::Freeze)
            .unwrap(),
    };
    assert!(matches!(
        group_in.apply(&mut group_working),
        Err(CommandError::TrackItemNotClip { layer }) if layer == group_layer.get()
    ));
    assert_eq!(group_working, group_doc);

    let (doc, layer) = layer_track_only_fixture();
    let clip = clip_at(&doc, layer);
    let mut non_positive = doc.clone();
    let in_non_positive = Command::TrimClipIn {
        target: layer,
        old_start: clip.start,
        old_duration: clip.duration,
        old_time_map: clip.time_map,
        new_start: clip.start,
        new_duration: RationalTime::try_new(0, 1).unwrap(),
        new_time_map: TimeMap::try_new(
            RationalTime::try_new(1, 1).unwrap(),
            2,
            1,
            motolii_core::OverrunMode::Freeze,
        )
        .unwrap(),
    };
    assert!(matches!(
        in_non_positive.apply(&mut non_positive),
        Err(CommandError::Validate(
            motolii_doc::DocumentError::NonPositiveClipDuration {
                layer_id
            }
        )) if layer_id == layer.get()
    ));
    assert_eq!(non_positive, doc);

    let mut invalid = doc.clone();
    let mut invalid_duration = doc.composition.duration;
    invalid_duration = invalid_duration
        .try_add(RationalTime::try_new(1, 1).unwrap())
        .unwrap();
    let in_invalid_after_clip = Command::TrimClipIn {
        target: layer,
        old_start: clip.start,
        old_duration: clip.duration,
        old_time_map: clip.time_map,
        new_start: clip.start,
        new_duration: invalid_duration,
        new_time_map: TimeMap::try_new(
            clip.time_map.source_start,
            2,
            1,
            motolii_core::OverrunMode::Loop,
        )
        .unwrap(),
    };
    assert!(matches!(
        in_invalid_after_clip.apply(&mut invalid),
        Err(CommandError::InvalidClipTrim { layer: got }) if got == layer.get()
    ));
    assert_eq!(invalid, doc);
}

#[test]
fn trim_clip_keyframed_clip_and_sibling_remain_unchanged_except_target_fields() {
    let fixture = v4_fixture();
    let mut doc = fixture.doc;
    let target = fixture.layer;

    let mut with_keyframes = doc.clone();
    let keyframe_cmd = reference_writer(doc.clone())
        .prepare_create_effect(target, 1, nested_create_draft())
        .unwrap();
    keyframe_cmd.apply(&mut with_keyframes).unwrap();
    let definition_id = match &keyframe_cmd {
        Command::CreateEffect { definition, .. } => definition.id,
        _ => unreachable!(),
    };
    let definition = with_keyframes.effect_definition(definition_id).unwrap();
    let mut saw_nested_keyframes = false;
    for param in definition.params.values() {
        if let DocParam::Keyframes(track) = param {
            saw_nested_keyframes = true;
            let mut key_iters = track.keys().iter();
            let first = key_iters.next().unwrap();
            let second = key_iters.next().unwrap();
            assert_eq!(first.t, RationalTime::ZERO);
            assert_eq!(second.t, RationalTime::try_new(1, 1).unwrap());
            assert_ne!(first.id, second.id);
            assert!(key_iters.next().is_none());
        }
    }
    assert!(saw_nested_keyframes);

    doc = with_keyframes;
    let sibling = doc.layers.allocate("sibling").unwrap();
    let source = clip_at(&doc, target).source.clone();
    let source_start = TimeMap::identity().source_start;
    doc.tracks[0].items.push(TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(sibling),
        start: RationalTime::try_new(2, 1).unwrap(),
        duration: RationalTime::try_new(3, 1).unwrap(),
        time_map: TimeMap::try_new(source_start, 3, 2, motolii_core::OverrunMode::Freeze).unwrap(),
        source,
    }));
    doc.validate().unwrap();

    let sibling_before = clip_at(&doc, sibling);

    let mut in_working = doc.clone();
    let mut in_expected = doc.clone();
    let in_target = reference_writer(doc.clone())
        .prepare_trim_clip_in(target, RationalTime::try_new(1, 1).unwrap())
        .unwrap()
        .unwrap();
    let (in_new_start, in_new_duration, in_new_time_map) = match &in_target {
        Command::TrimClipIn {
            new_start,
            new_duration,
            new_time_map,
            ..
        } => (*new_start, *new_duration, *new_time_map),
        _ => unreachable!(),
    };
    in_target.apply(&mut in_working).unwrap();
    if let Some(item) = find_track_item_mut(&mut in_expected, target) {
        let TrackItem::Clip(clip) = item else {
            panic!("target should remain clip");
        };
        clip.start = in_new_start;
        clip.duration = in_new_duration;
        clip.time_map = in_new_time_map;
    } else {
        panic!("target should exist");
    }
    assert_eq!(in_working, in_expected);
    assert_eq!(clip_at(&in_working, sibling), sibling_before);

    let mut out_working = doc.clone();
    let mut out_expected = doc.clone();
    let out_target = reference_writer(doc.clone())
        .prepare_trim_clip_out(target, RationalTime::try_new(4, 1).unwrap())
        .unwrap()
        .unwrap();
    out_target.apply(&mut out_working).unwrap();
    if let Some(item) = find_track_item_mut(&mut out_expected, target) {
        let TrackItem::Clip(clip) = item else {
            panic!("target should remain clip");
        };
        clip.duration = RationalTime::try_new(4, 1).unwrap();
    } else {
        panic!("target should exist");
    }
    assert_eq!(out_working, out_expected);
    assert_eq!(clip_at(&out_working, sibling), sibling_before);
}

#[test]
fn trim_clip_journal_v2_wire_tags_and_snake_case_fields_are_exact_for_both_variants() {
    let in_json = r#"{
        "format_version": 2,
        "command": {
            "TrimClipIn": {
                "target": 9,
                "old_start": {"num": 0, "den": 1},
                "old_duration": {"num": 5, "den": 1},
                "old_time_map": {
                    "source_start": {"num": 10, "den": 1},
                    "speed_num": 3,
                    "speed_den": 1,
                    "overrun_mode": "black"
                },
                "new_start": {"num": 1, "den": 1},
                "new_duration": {"num": 4, "den": 1},
                "new_time_map": {
                    "source_start": {"num": 13, "den": 1},
                    "speed_num": 3,
                    "speed_den": 1,
                    "overrun_mode": "black"
                }
            }
        }
    }"#;
    let in_value: Value = serde_json::from_str(in_json).unwrap();
    assert_eq!(
        in_value.get("format_version").and_then(Value::as_u64),
        Some(2)
    );
    let in_cmds = in_value.get("command").unwrap().as_object().unwrap();
    assert_eq!(in_cmds.len(), 1);
    let (in_tag, in_payload) = in_cmds.iter().next().unwrap();
    assert_eq!(in_tag, "TrimClipIn");
    let in_payload = in_payload.as_object().unwrap();
    assert_eq!(
        in_payload.len(),
        7,
        "TrimClipIn payload must only use snake_case interval/time_map keys"
    );
    for key in [
        "target",
        "old_start",
        "old_duration",
        "old_time_map",
        "new_start",
        "new_duration",
        "new_time_map",
    ] {
        assert!(in_payload.contains_key(key));
    }
    let decoded_in: JournalEdit = serde_json::from_str(in_json).unwrap();
    assert_eq!(
        decoded_in,
        JournalEdit {
            format_version: V2_EDIT_FORMAT_VERSION,
            command: Command::TrimClipIn {
                target: LayerId::from_raw(9),
                old_start: RationalTime::try_new(0, 1).unwrap(),
                old_duration: RationalTime::try_new(5, 1).unwrap(),
                old_time_map: TimeMap::try_new(
                    RationalTime::try_new(10, 1).unwrap(),
                    3,
                    1,
                    motolii_core::OverrunMode::Black,
                )
                .unwrap(),
                new_start: RationalTime::try_new(1, 1).unwrap(),
                new_duration: RationalTime::try_new(4, 1).unwrap(),
                new_time_map: TimeMap::try_new(
                    RationalTime::try_new(13, 1).unwrap(),
                    3,
                    1,
                    motolii_core::OverrunMode::Black,
                )
                .unwrap(),
            },
        }
    );

    let out_json = r#"{
        "format_version": 2,
        "command": {
            "TrimClipOut": {
                "target": 9,
                "old_duration": {"num": 5, "den": 1},
                "new_duration": {"num": 3, "den": 1}
            }
        }
    }"#;
    let out_value: Value = serde_json::from_str(out_json).unwrap();
    let out_cmds = out_value.get("command").unwrap().as_object().unwrap();
    assert_eq!(out_cmds.len(), 1);
    let (out_tag, out_payload) = out_cmds.iter().next().unwrap();
    assert_eq!(out_tag, "TrimClipOut");
    let out_payload = out_payload.as_object().unwrap();
    assert_eq!(out_payload.len(), 3);
    for key in ["target", "old_duration", "new_duration"] {
        assert!(out_payload.contains_key(key));
    }
    let decoded_out: JournalEdit = serde_json::from_str(out_json).unwrap();
    assert_eq!(
        decoded_out,
        JournalEdit {
            format_version: V2_EDIT_FORMAT_VERSION,
            command: Command::TrimClipOut {
                target: LayerId::from_raw(9),
                old_duration: RationalTime::try_new(5, 1).unwrap(),
                new_duration: RationalTime::try_new(3, 1).unwrap(),
            },
        }
    );
}

#[test]
fn trim_clip_overrun_mode_survives_for_black_and_loop_even_when_source_is_out_of_range() {
    for overrun in [
        motolii_core::OverrunMode::Black,
        motolii_core::OverrunMode::Loop,
    ] {
        let (mut doc, layer) = layer_track_only_fixture();
        let out_of_range_source_start = RationalTime::try_new(5000, 1).unwrap();
        {
            let TrackItem::Clip(clip) = &mut doc.tracks[0].items[0] else {
                unreachable!();
            };
            clip.time_map = TimeMap::try_new(out_of_range_source_start, 2, 1, overrun).unwrap();
        }
        let baseline = clip_at(&doc, layer).time_map;

        let in_cmd = reference_writer(doc.clone())
            .prepare_trim_clip_in(layer, RationalTime::try_new(-1, 1).unwrap())
            .unwrap()
            .unwrap();
        let mut in_working = doc.clone();
        in_cmd.apply(&mut in_working).unwrap();
        let in_after = clip_at(&in_working, layer);
        assert_eq!(in_after.time_map.overrun_mode, overrun);
        assert_eq!(in_after.time_map.speed_num(), baseline.speed_num());
        assert_eq!(in_after.time_map.speed_den(), baseline.speed_den());

        let out_cmd = reference_writer(doc.clone())
            .prepare_trim_clip_out(layer, RationalTime::try_new(4, 1).unwrap())
            .unwrap()
            .unwrap();
        let mut out_working = doc.clone();
        out_cmd.apply(&mut out_working).unwrap();
        let out_after = clip_at(&out_working, layer);
        assert_eq!(out_after.time_map.overrun_mode, overrun);
        assert_eq!(out_after.time_map, baseline);
    }
}
