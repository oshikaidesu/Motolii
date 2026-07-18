//! D1l Stage B-3: Writer prepare API 受け入れ正本。

mod common;

use std::collections::BTreeMap;
use std::sync::Arc;

use common::identity_roundtrip::assert_identity_command_roundtrip;

use motolii_core::RationalTime;
use motolii_doc::{
    Clip, ClipSource, Command, CommandError, DocParam, DocValue, Document, DocumentError,
    DocumentPluginError, DocumentWriter, DraftDocParam, DraftKeyframe, EffectDefinition,
    EffectDefinitionDraft, EffectDefinitionId, EffectId, EffectUse, ItemEnvelope, LayerId,
    PrepareError, StableIdError, StableIdReservation, Track, TrackId, TrackItem,
    MIN_READER_VERSION_FOR_COMP_CAMERA, WRITER_VERSION,
};
use motolii_eval::Interp;
use motolii_plugin::reference::reference_catalog;
use serde_json::Map;

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
