//! D1l Stage B-1/2: v2 lifecycle command と reservation 契約の固定。

pub mod common;

use std::collections::BTreeMap;

use common::identity_roundtrip::assert_identity_command_roundtrip;

use motolii_core::RationalTime;
use motolii_doc::{
    load_document_bytes_with_limits, BlendMode, Clip, ClipSource, Command, CommandError,
    DocKeyframe, DocKeyframeTrack, DocParam, DocValue, Document, EffectDefinition,
    EffectDefinitionId, EffectId, EffectInstance, EffectUse, ItemEnvelope, KeyframeId, LayerId,
    ParentLocator, PersistError, ResourceLimits, ScalarPropertyId, StableIdReservation, Track,
    TrackId, TrackItem, MIN_READER_VERSION_FOR_COMP_CAMERA,
    MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS, WRITER_VERSION,
};
use motolii_eval::Interp;
use serde_json::{json, Value as JsonValue};

struct Fixture {
    doc: Document,
    layer: LayerId,
    track: TrackId,
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
        "core.filter.tint",
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
        track,
        use_id,
        def_id,
    }
}

fn create_command(doc: &Document, layer: LayerId, index: usize) -> Command {
    let before = doc.next_stable_id.peek_next();
    let use_id = EffectId::from_raw(before);
    let def_id = EffectDefinitionId::from_raw(before + 1);
    Command::CreateEffect {
        target: layer,
        index,
        use_: EffectUse {
            id: use_id,
            definition_id: def_id,
        },
        definition: EffectDefinition::new(
            def_id,
            "core.filter.blur",
            1,
            true,
            BTreeMap::new(),
            Default::default(),
        ),
        stable_id_reservation: StableIdReservation::new(before, before + 2),
    }
}

fn link_command(
    doc: &Document,
    layer: LayerId,
    index: usize,
    definition_id: EffectDefinitionId,
) -> Command {
    let before = doc.next_stable_id.peek_next();
    let use_id = EffectId::from_raw(before);
    Command::LinkEffectUse {
        target: layer,
        index,
        use_: EffectUse {
            id: use_id,
            definition_id,
        },
        stable_id_reservation: StableIdReservation::new(before, before + 1),
    }
}

fn copy_local_command(
    doc: &Document,
    use_id: EffectId,
    previous_definition_id: EffectDefinitionId,
) -> Command {
    let before = doc.next_stable_id.peek_next();
    let new_definition_id = EffectDefinitionId::from_raw(before);
    let source = doc.effect_definition(previous_definition_id).unwrap();
    let mut new_definition = source.clone();
    new_definition.id = new_definition_id;
    Command::CopyLocalEffect {
        use_id,
        previous_definition_id,
        new_definition,
        stable_id_reservation: StableIdReservation::new(before, before + 1),
    }
}

fn assert_reject_unchanged(doc: &Document, command: Command) -> CommandError {
    let before = doc.clone();
    let mut working = doc.clone();
    let err = command.apply(&mut working).expect_err("must reject");
    assert_eq!(working, before, "reject must keep whole document unchanged");
    err
}

#[test]
fn stable_id_reservation_closure_is_exactly_six_variants() {
    let f = v4_fixture();
    let reservation = StableIdReservation::new(10, 11);
    let use_ = EffectUse {
        id: f.use_id,
        definition_id: f.def_id,
    };
    let def = f.doc.effect_definition(f.def_id).unwrap().clone();

    let with_reservation = [
        Command::CreateEffect {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
            definition: def.clone(),
            stable_id_reservation: reservation,
        },
        Command::UndoCreateEffect {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
            definition: def.clone(),
            stable_id_reservation: reservation,
        },
        Command::LinkEffectUse {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
            stable_id_reservation: reservation,
        },
        Command::UndoLinkEffectUse {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
            stable_id_reservation: reservation,
        },
        Command::CopyLocalEffect {
            use_id: f.use_id,
            previous_definition_id: f.def_id,
            new_definition: def.clone(),
            stable_id_reservation: reservation,
        },
        Command::UndoCopyLocalEffect {
            use_id: f.use_id,
            previous_definition_id: f.def_id,
            new_definition: def,
            stable_id_reservation: reservation,
        },
    ];
    assert_eq!(with_reservation.len(), 6);
    assert!(with_reservation
        .iter()
        .all(|cmd| cmd.stable_id_reservation().is_some()));

    let without_reservation = [
        Command::SetProperty {
            target: f.layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.5),
        },
        Command::SetBlendMode {
            target: f.layer,
            old: BlendMode::Normal,
            new: BlendMode::Multiply,
        },
        Command::SetClippingMask {
            target: f.layer,
            old: Default::default(),
            new: Default::default(),
        },
        Command::SetTransformParent {
            target: f.layer,
            old: None,
            new: None,
        },
        Command::AddEffect {
            target: f.layer,
            index: 0,
            effect: EffectInstance::from_use_and_definition(
                &use_,
                f.doc.effect_definition(f.def_id).unwrap(),
            ),
            introduced_definition: false,
        },
        Command::RemoveEffect {
            target: f.layer,
            index: 0,
            effect: EffectInstance::from_use_and_definition(
                &use_,
                f.doc.effect_definition(f.def_id).unwrap(),
            ),
            introduced_definition: false,
        },
        Command::SetEffectEnabled {
            target: f.layer,
            effect: f.use_id,
            old: true,
            new: false,
        },
        Command::DeleteEffectDefinition {
            definition: f.doc.effect_definition(f.def_id).unwrap().clone(),
        },
        Command::AddEffectDefinition {
            definition: f.doc.effect_definition(f.def_id).unwrap().clone(),
        },
        Command::UnlinkEffectUse {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
        },
        Command::RestoreEffectUse {
            target: f.layer,
            index: 0,
            use_: use_.clone(),
        },
        Command::SetAudioComponentEnabled {
            target: f.layer,
            index: 0,
            old: true,
            new: false,
        },
        Command::SetAudioComponentGain {
            target: f.layer,
            index: 0,
            old: DocParam::const_f64(1.0),
            new: DocParam::const_f64(0.5),
        },
        Command::AddTrackItem {
            parent: ParentLocator::Track(f.track),
            index: 0,
            item: f.doc.tracks[0].items[0].clone(),
            layer_names: Default::default(),
        },
        Command::RemoveTrackItem {
            parent: ParentLocator::Track(f.track),
            index: 0,
            item: f.doc.tracks[0].items[0].clone(),
            layer_names: Default::default(),
        },
    ];
    assert!(without_reservation
        .iter()
        .all(|cmd| cmd.stable_id_reservation().is_none()));
}

#[test]
fn create_link_copy_and_inverse_are_identity_roundtrip() {
    let f = v4_fixture();
    assert_identity_command_roundtrip(&f.doc, create_command(&f.doc, f.layer, 1));
    assert_identity_command_roundtrip(&f.doc, link_command(&f.doc, f.layer, 1, f.def_id));
    assert_identity_command_roundtrip(&f.doc, copy_local_command(&f.doc, f.use_id, f.def_id));
}

#[test]
fn reservation_interval_cases_empty_reverse_middle_and_huge_are_rejected() {
    let f = v4_fixture();
    let base = f.doc.next_stable_id.peek_next();

    let mut empty = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        stable_id_reservation,
        ..
    } = &mut empty
    {
        *stable_id_reservation = StableIdReservation::new(base, base);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, empty),
        CommandError::InvalidStableIdReservationInterval { .. }
    ));

    let mut reverse = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        stable_id_reservation,
        ..
    } = &mut reverse
    {
        *stable_id_reservation = StableIdReservation::new(base + 2, base + 1);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, reverse),
        CommandError::InvalidStableIdReservationInterval { .. }
    ));

    let mut middle = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        use_,
        definition,
        stable_id_reservation,
        ..
    } = &mut middle
    {
        use_.id = EffectId::from_raw(base + 1);
        use_.definition_id = EffectDefinitionId::from_raw(base + 2);
        definition.id = EffectDefinitionId::from_raw(base + 2);
        *stable_id_reservation = StableIdReservation::new(base + 1, base + 3);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, middle),
        CommandError::StableIdReservationCounterMismatch { .. }
    ));

    let mut huge = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        stable_id_reservation,
        ..
    } = &mut huge
    {
        *stable_id_reservation = StableIdReservation::new(0, u64::MAX);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, huge),
        CommandError::StableIdReservationMismatch { .. }
    ));
}

#[test]
fn reservation_payload_cases_hole_overwide_out_of_order_and_collision_are_rejected() {
    let f = v4_fixture();
    let base = f.doc.next_stable_id.peek_next();

    let mut hole = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        use_, definition, ..
    } = &mut hole
    {
        use_.id = EffectId::from_raw(base);
        use_.definition_id = EffectDefinitionId::from_raw(base + 2);
        definition.id = EffectDefinitionId::from_raw(base + 2);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, hole),
        CommandError::StableIdReservationMismatch { .. }
    ));

    let mut overwide = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        stable_id_reservation,
        ..
    } = &mut overwide
    {
        *stable_id_reservation = StableIdReservation::new(base, base + 3);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, overwide),
        CommandError::StableIdReservationMismatch { .. }
    ));

    let mut out_of_order = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        use_, definition, ..
    } = &mut out_of_order
    {
        use_.id = EffectId::from_raw(base + 1);
        use_.definition_id = EffectDefinitionId::from_raw(base);
        definition.id = EffectDefinitionId::from_raw(base);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, out_of_order),
        CommandError::StableIdReservationMismatch { .. }
    ));

    let mut collision = create_command(&f.doc, f.layer, 1);
    if let Command::CreateEffect {
        use_,
        definition,
        stable_id_reservation,
        ..
    } = &mut collision
    {
        use_.id = f.use_id;
        use_.definition_id = EffectDefinitionId::from_raw(f.use_id.get() + 1);
        definition.id = EffectDefinitionId::from_raw(f.use_id.get() + 1);
        *stable_id_reservation = StableIdReservation::new(f.use_id.get(), f.use_id.get() + 2);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, collision),
        CommandError::StableIdCollision { .. }
    ));
}

#[test]
fn undo_create_invalid_reservation_rejects_without_mutation() {
    let f = v4_fixture();
    let create = create_command(&f.doc, f.layer, 1);
    let mut applied = f.doc.clone();
    create.apply(&mut applied).unwrap();
    let mut undo = create.inverse();
    if let Command::UndoCreateEffect {
        stable_id_reservation,
        ..
    } = &mut undo
    {
        let before = stable_id_reservation.before();
        *stable_id_reservation = StableIdReservation::new(before, before);
    }
    assert!(matches!(
        assert_reject_unchanged(&applied, undo),
        CommandError::InvalidStableIdReservationInterval { .. }
    ));
}

#[test]
fn undo_link_invalid_reservation_rejects_without_mutation() {
    let f = v4_fixture();
    let link = link_command(&f.doc, f.layer, 1, f.def_id);
    let mut applied = f.doc.clone();
    link.apply(&mut applied).unwrap();
    let mut undo = link.inverse();
    if let Command::UndoLinkEffectUse {
        stable_id_reservation,
        ..
    } = &mut undo
    {
        let before = stable_id_reservation.before();
        *stable_id_reservation = StableIdReservation::new(before, before + 2);
    }
    assert!(matches!(
        assert_reject_unchanged(&applied, undo),
        CommandError::StableIdReservationMismatch { .. }
    ));
}

#[test]
fn undo_copy_invalid_reservation_and_counter_underflow_reject_without_mutation() {
    let f = v4_fixture();
    let copy = copy_local_command(&f.doc, f.use_id, f.def_id);
    let mut applied = f.doc.clone();
    copy.apply(&mut applied).unwrap();

    let mut invalid_shape = copy.inverse();
    if let Command::UndoCopyLocalEffect {
        stable_id_reservation,
        ..
    } = &mut invalid_shape
    {
        let before = stable_id_reservation.before();
        *stable_id_reservation = StableIdReservation::new(before + 2, before + 1);
    }
    assert!(matches!(
        assert_reject_unchanged(&applied, invalid_shape),
        CommandError::InvalidStableIdReservationInterval { .. }
    ));

    let mut counter_underflow = copy.inverse();
    if let Command::UndoCopyLocalEffect {
        new_definition,
        stable_id_reservation,
        ..
    } = &mut counter_underflow
    {
        let next = applied.next_stable_id.peek_next();
        new_definition.id = EffectDefinitionId::from_raw(next);
        *stable_id_reservation = StableIdReservation::new(next, next + 1);
    }
    assert!(matches!(
        assert_reject_unchanged(&applied, counter_underflow),
        CommandError::StableIdReservationCounterMismatch { .. }
    ));
}

#[test]
fn copy_local_stale_previous_and_non_copy_payload_reject_without_mutation() {
    let f = v4_fixture();

    let mut stale_previous = copy_local_command(&f.doc, f.use_id, f.def_id);
    if let Command::CopyLocalEffect {
        previous_definition_id,
        ..
    } = &mut stale_previous
    {
        *previous_definition_id = EffectDefinitionId::from_raw(9999);
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, stale_previous),
        CommandError::CopyLocalDefinitionMismatch { .. }
    ));

    let mut non_copy = copy_local_command(&f.doc, f.use_id, f.def_id);
    if let Command::CopyLocalEffect { new_definition, .. } = &mut non_copy {
        new_definition.plugin_id = "tampered.plugin".into();
    }
    assert!(matches!(
        assert_reject_unchanged(&f.doc, non_copy),
        CommandError::CopyLocalPayloadMismatch
    ));
}

#[test]
fn undo_copy_local_shared_interference_rejects_without_mutation() {
    let f = v4_fixture();
    let copy = copy_local_command(&f.doc, f.use_id, f.def_id);
    let Command::CopyLocalEffect { new_definition, .. } = &copy else {
        panic!("expected copy command");
    };
    let new_definition_id = new_definition.id;
    let mut working = f.doc.clone();
    copy.apply(&mut working).unwrap();

    let extra_use_id = EffectId::from_raw(working.next_stable_id.allocate().unwrap());
    let cloned_definition = working
        .effect_definition(new_definition_id)
        .unwrap()
        .clone();
    Command::AddEffect {
        target: f.layer,
        index: 1,
        effect: EffectInstance::from_use_and_definition(
            &EffectUse {
                id: extra_use_id,
                definition_id: new_definition_id,
            },
            &cloned_definition,
        ),
        introduced_definition: false,
    }
    .apply(&mut working)
    .unwrap();

    let undo = copy.inverse();
    let before_undo = working.clone();
    let err = undo.apply(&mut working).expect_err("must reject");
    assert!(matches!(
        err,
        CommandError::UndoCopyLocalDefinitionInUse { .. }
    ));
    assert_eq!(working, before_undo);
}

#[test]
fn gate_rejects_v1_v2_v3_and_future_or_read_only_documents_without_mutation() {
    let f = v4_fixture();
    let modes = [
        (1, 1),
        (2, 2),
        (3, 3),
        (
            WRITER_VERSION + 1,
            MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS,
        ),
        (WRITER_VERSION, MIN_READER_VERSION_FOR_COMP_CAMERA + 1),
    ];
    for (version, min_reader_version) in modes {
        let mut doc = f.doc.clone();
        doc.version = version;
        doc.min_reader_version = min_reader_version;
        let before = doc.clone();
        let cmd = create_command(&doc, f.layer, 1);
        let mut working = doc.clone();
        let err = cmd.apply(&mut working).expect_err("gate reject");
        assert!(matches!(
            err,
            CommandError::EffectLifecycleRequiresV4Document { .. }
        ));
        assert_eq!(working, before);
    }
}

fn v4_inline_effect_envelope_json() -> JsonValue {
    json!({
        "layer_id": 0,
        "effects": [{
            "id": 1,
            "plugin_id": "core.filter.tint",
            "params": {"amount": {"const": {"F64": 0.5}}}
        }],
        "transform": {
            "position": {"const": {"Vec2": [0.0, 0.0]}},
            "anchor": {"const": {"Vec2": [0.0, 0.0]}},
            "scale": {"const": {"Vec2": [1.0, 1.0]}},
            "rotation": {"const": {"F64": 0.0}}
        },
        "opacity": {"const": {"F64": 1.0}}
    })
}

fn v4_document_shell(envelope: JsonValue) -> JsonValue {
    json!({
        "version": WRITER_VERSION,
        "min_reader_version": MIN_READER_VERSION_FOR_COMP_CAMERA,
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
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": envelope,
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
        "next_stable_id": 2
    })
}

#[test]
fn load_rejects_v4_inline_and_hybrid_effects_without_partial_document() {
    let limits = ResourceLimits::production();

    let inline = v4_document_shell(v4_inline_effect_envelope_json());
    let inline_err =
        load_document_bytes_with_limits(&serde_json::to_vec(&inline).unwrap(), &limits)
            .expect_err("inline at v4 must fail");
    assert!(
        matches!(inline_err, PersistError::Json(_)),
        "unexpected inline error: {inline_err:?}"
    );

    let mut hybrid_envelope = v4_inline_effect_envelope_json();
    let effects = hybrid_envelope
        .get_mut("effects")
        .unwrap()
        .as_array_mut()
        .unwrap();
    effects[0]
        .as_object_mut()
        .unwrap()
        .insert("definition_id".into(), JsonValue::from(99));
    let hybrid = v4_document_shell(hybrid_envelope);
    let hybrid_err =
        load_document_bytes_with_limits(&serde_json::to_vec(&hybrid).unwrap(), &limits)
            .expect_err("hybrid at v4 must fail");
    assert!(
        matches!(hybrid_err, PersistError::Json(_)),
        "unexpected hybrid error: {hybrid_err:?}"
    );
}

#[test]
fn keyframed_vec2_axes_copy_local_remints_ids_in_x_then_y_order() {
    let mut f = v4_fixture();
    let old_x = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let old_y = KeyframeId::from_raw(f.doc.next_stable_id.allocate().unwrap());
    let mut x_track = DocKeyframeTrack::new();
    x_track.insert(DocKeyframe {
        id: old_x,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.1),
        interp: Interp::Hold,
    });
    let mut y_track = DocKeyframeTrack::new();
    y_track.insert(DocKeyframe {
        id: old_y,
        t: RationalTime::try_new(1, 30).unwrap(),
        value: DocValue::F64(0.2),
        interp: Interp::Linear,
    });
    f.doc
        .effect_definition_mut(f.def_id)
        .unwrap()
        .params
        .insert(
            "offset".into(),
            DocParam::Vec2Axes {
                x: Box::new(DocParam::Keyframes(x_track)),
                y: Box::new(DocParam::Keyframes(y_track)),
            },
        );

    let before = f.doc.next_stable_id.peek_next();
    let mut seq = f.doc.next_stable_id;
    let new_def_id = EffectDefinitionId::from_raw(seq.allocate().unwrap());
    let new_x = KeyframeId::from_raw(seq.allocate().unwrap());
    let new_y = KeyframeId::from_raw(seq.allocate().unwrap());
    let after = seq.peek_next();
    let mut new_definition = f.doc.effect_definition(f.def_id).unwrap().clone();
    new_definition.id = new_def_id;
    let mut new_x_track = DocKeyframeTrack::new();
    new_x_track.insert(DocKeyframe {
        id: new_x,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.1),
        interp: Interp::Hold,
    });
    let mut new_y_track = DocKeyframeTrack::new();
    new_y_track.insert(DocKeyframe {
        id: new_y,
        t: RationalTime::try_new(1, 30).unwrap(),
        value: DocValue::F64(0.2),
        interp: Interp::Linear,
    });
    new_definition.params.insert(
        "offset".into(),
        DocParam::Vec2Axes {
            x: Box::new(DocParam::Keyframes(new_x_track)),
            y: Box::new(DocParam::Keyframes(new_y_track)),
        },
    );
    let cmd = Command::CopyLocalEffect {
        use_id: f.use_id,
        previous_definition_id: f.def_id,
        new_definition,
        stable_id_reservation: StableIdReservation::new(before, after),
    };
    let mut working = f.doc.clone();
    cmd.apply(&mut working).unwrap();
    let copied = working.effect_definition(new_def_id).unwrap();
    let ids = match copied.params.get("offset").unwrap() {
        DocParam::Vec2Axes { x, y } => {
            let mut ids = Vec::new();
            if let DocParam::Keyframes(track) = x.as_ref() {
                ids.extend(track.keys().iter().map(|k| k.id));
            }
            if let DocParam::Keyframes(track) = y.as_ref() {
                ids.extend(track.keys().iter().map(|k| k.id));
            }
            ids
        }
        other => panic!("unexpected param: {other:?}"),
    };
    assert_eq!(ids, vec![new_x, new_y]);
    assert!(ids
        .iter()
        .all(|id| id.get() != old_x.get() && id.get() != old_y.get()));
    assert_identity_command_roundtrip(&f.doc, cmd);
}

#[test]
fn unlink_restore_json_has_no_reservation_and_counter_does_not_move() {
    let f = v4_fixture();
    let use_ = EffectUse {
        id: f.use_id,
        definition_id: f.def_id,
    };
    let unlink = Command::UnlinkEffectUse {
        target: f.layer,
        index: 0,
        use_,
    };
    let unlink_json = serde_json::to_value(&unlink).unwrap();
    assert!(unlink_json["UnlinkEffectUse"]
        .get("stable_id_reservation")
        .is_none());
    let restore = unlink.inverse();
    let restore_json = serde_json::to_value(&restore).unwrap();
    assert!(restore_json["RestoreEffectUse"]
        .get("stable_id_reservation")
        .is_none());

    let counter_before = f.doc.next_stable_id.peek_next();
    let mut working = f.doc.clone();
    unlink.apply(&mut working).unwrap();
    assert_eq!(working.next_stable_id.peek_next(), counter_before);
    restore.apply(&mut working).unwrap();
    assert_eq!(working.next_stable_id.peek_next(), counter_before);
    assert_eq!(working, f.doc);
}

#[test]
fn wire_key_is_use_not_use_() {
    let f = v4_fixture();
    let create = create_command(&f.doc, f.layer, 1);
    let link = link_command(&f.doc, f.layer, 1, f.def_id);
    let copy = copy_local_command(&f.doc, f.use_id, f.def_id);
    for (variant, json) in [
        ("CreateEffect", serde_json::to_value(&create).unwrap()),
        (
            "UndoCreateEffect",
            serde_json::to_value(create.inverse()).unwrap(),
        ),
        ("LinkEffectUse", serde_json::to_value(&link).unwrap()),
        (
            "UndoLinkEffectUse",
            serde_json::to_value(link.inverse()).unwrap(),
        ),
        (
            "UnlinkEffectUse",
            serde_json::to_value(Command::UnlinkEffectUse {
                target: f.layer,
                index: 0,
                use_: EffectUse {
                    id: f.use_id,
                    definition_id: f.def_id,
                },
            })
            .unwrap(),
        ),
        (
            "RestoreEffectUse",
            serde_json::to_value(
                Command::UnlinkEffectUse {
                    target: f.layer,
                    index: 0,
                    use_: EffectUse {
                        id: f.use_id,
                        definition_id: f.def_id,
                    },
                }
                .inverse(),
            )
            .unwrap(),
        ),
        ("CopyLocalEffect", serde_json::to_value(copy).unwrap()),
    ] {
        let body = &json[variant];
        if variant != "CopyLocalEffect" {
            assert!(body.get("use").is_some(), "{variant} must expose use key");
            assert!(
                body.get("use_").is_none(),
                "{variant} must not expose use_ key"
            );
        }
    }
}

#[test]
fn v1_add_effect_remove_effect_shape_is_unchanged() {
    let effect = EffectInstance {
        id: EffectId::from_raw(10),
        definition_id: EffectDefinitionId::from_raw(11),
        plugin_id: "core.filter.tint".into(),
        effect_version: 1,
        enabled: true,
        params: BTreeMap::new(),
        extra: Default::default(),
    };
    let add = Command::AddEffect {
        target: LayerId::from_raw(1),
        index: 0,
        effect: effect.clone(),
        introduced_definition: true,
    };
    let add_json = serde_json::to_value(&add).unwrap();
    assert_eq!(
        add_json["AddEffect"]["introduced_definition"],
        JsonValue::Bool(true)
    );
    assert!(add_json["AddEffect"].get("stable_id_reservation").is_none());

    let remove = Command::RemoveEffect {
        target: LayerId::from_raw(1),
        index: 0,
        effect,
        introduced_definition: true,
    };
    let remove_json = serde_json::to_value(remove).unwrap();
    assert_eq!(
        remove_json["RemoveEffect"]["introduced_definition"],
        JsonValue::Bool(true)
    );
    assert!(remove_json["RemoveEffect"]
        .get("stable_id_reservation")
        .is_none());
}

#[test]
fn sanity_fixture_uses_current_v5_writer_contract() {
    let f = v4_fixture();
    assert_eq!(f.doc.version, WRITER_VERSION);
    assert_eq!(f.doc.min_reader_version, MIN_READER_VERSION_FOR_COMP_CAMERA);
    assert_eq!(f.doc.version, 5);
    assert_eq!(f.doc.min_reader_version, 5);
    let _ = f.track;
}
