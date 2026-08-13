//! Scale / Rotation / Opacity へ既存 DocParam 上で key を載せる prepare。
//! Position は AddPositionKey のまま。Command に新 variant は足さない。

use std::sync::Arc;

use motolii_core::RationalTime;
use motolii_doc::param_eval::{eval_f64, eval_vec2, ResolvedLayerParams};
use motolii_doc::{
    prepare_add_transform_param_key, prepare_remove_transform_param_key,
    prepare_set_transform_param_key_value, AddTransformParamKeyPreparation,
    AddTransformParamKeyPrepareError, Clip, ClipSource, Command, DocKeyframe, DocKeyframeTrack,
    DocParam, DocValue, Document, DocumentWriter, EffectId, ItemEnvelope, KeyframeId, LayerId,
    RemoveTransformParamKeyPrepareError, ScalarPropertyId, SetTransformParamKeyValuePrepareError,
    Track, TrackItem,
};
use motolii_eval::{DataTracks, Interp};
use motolii_plugin::reference::reference_catalog;

fn clip_doc(mut set: impl FnMut(&mut ItemEnvelope)) -> (Document, LayerId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("target").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let mut envelope = ItemEnvelope::new(layer);
    set(&mut envelope);
    for param in [
        &envelope.transform.scale,
        &envelope.transform.rotation,
        &envelope.opacity,
    ] {
        if let DocParam::Keyframes(track) = param {
            let mut max_id = doc.next_stable_id.peek_next();
            for key in track.keys() {
                max_id = max_id.max(key.id.get() + 1);
            }
            while doc.next_stable_id.peek_next() < max_id {
                doc.next_stable_id.allocate().unwrap();
            }
        }
    }
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(10),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().unwrap();
    (doc, layer)
}

fn writer(doc: Document) -> DocumentWriter {
    DocumentWriter::new(doc, Arc::new(reference_catalog().unwrap())).unwrap()
}

fn scale_keyframes(keys: Vec<(u64, RationalTime, [f64; 2])>) -> DocParam {
    let mut track = DocKeyframeTrack::new();
    for (id, t, value) in keys {
        track.insert(DocKeyframe {
            id: KeyframeId::from_raw(id),
            t,
            value: DocValue::Vec2(value),
            interp: Interp::Linear,
        });
    }
    DocParam::Keyframes(track)
}

#[test]
fn const_scale_prepares_set_property_keyframes_at_t() {
    let (doc, layer) = clip_doc(|env| env.transform.scale = DocParam::const_vec2([2.0, 3.0]));
    let t = RationalTime::from_seconds(1);
    let prepared =
        prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, t).unwrap();
    let (key_id, command) = match prepared {
        AddTransformParamKeyPreparation::Prepared { key_id, command } => (key_id, command),
        AddTransformParamKeyPreparation::AlreadyPresent { .. } => {
            panic!("const scale must prepare a key")
        }
    };
    let Command::SetProperty {
        target,
        property,
        old_value,
        new_value,
    } = command
    else {
        panic!("Scale key must be SetProperty, not a new Command variant");
    };
    assert_eq!(target, layer);
    assert_eq!(property, ScalarPropertyId::Scale);
    assert_eq!(old_value, DocParam::const_vec2([2.0, 3.0]));
    let DocParam::Keyframes(track) = new_value else {
        panic!("const must lower to Keyframes");
    };
    assert_eq!(track.keys().len(), 1);
    assert_eq!(track.keys()[0].id, key_id);
    assert_eq!(track.keys()[0].t, t);
    assert_eq!(track.keys()[0].value, DocValue::Vec2([2.0, 3.0]));
    assert_eq!(track.keys()[0].interp, Interp::Linear);
}

#[test]
fn second_prepare_at_same_t_is_already_present() {
    let t = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t, [2.0, 3.0])]);
    });
    let prepared =
        prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, t).unwrap();
    match prepared {
        AddTransformParamKeyPreparation::AlreadyPresent { key_id } => {
            assert_eq!(key_id, KeyframeId::from_raw(0));
        }
        AddTransformParamKeyPreparation::Prepared { .. } => panic!("same t must be AlreadyPresent"),
    }
}

#[test]
fn command_apply_dump_eval_at_t_returns_const_scale() {
    let (mut doc, layer) = clip_doc(|env| env.transform.scale = DocParam::const_vec2([2.0, 3.0]));
    let t = RationalTime::from_seconds(1);
    let prepared =
        prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, t).unwrap();
    let command = match prepared {
        AddTransformParamKeyPreparation::Prepared { command, .. } => command,
        AddTransformParamKeyPreparation::AlreadyPresent { .. } => {
            panic!("const scale must prepare")
        }
    };
    command.apply(&mut doc).unwrap();

    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("fixture clip");
    };
    let env = &clip.envelope;
    let dump = serde_json::to_value(&env.transform.scale).unwrap();
    assert!(
        dump.get("keyframes").is_some(),
        "dump must show Keyframes: {dump}"
    );
    let got = eval_vec2(
        &env.transform.scale,
        t,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap();
    assert_eq!(got, [2.0, 3.0]);
    assert_eq!(doc.next_stable_id.peek_next(), 1);
}

#[test]
fn document_writer_apply_macro_commits_scale_keyframe_id() {
    let (doc, layer) = clip_doc(|env| env.transform.scale = DocParam::const_vec2([2.0, 3.0]));
    let t = RationalTime::from_seconds(1);
    let prepared =
        prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, t).unwrap();
    let command = match prepared {
        AddTransformParamKeyPreparation::Prepared { command, .. } => command,
        AddTransformParamKeyPreparation::AlreadyPresent { .. } => {
            panic!("const scale must prepare")
        }
    };
    let inverse = command.inverse();

    let mut writer = writer(doc);
    writer.apply_macro(vec![command]).unwrap();
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), 1);
    let TrackItem::Clip(clip) = &writer.snapshot().tracks[0].items[0] else {
        panic!("fixture clip");
    };
    let got = eval_vec2(
        &clip.envelope.transform.scale,
        t,
        &DataTracks::new(),
        &ResolvedLayerParams::default(),
    )
    .unwrap();
    assert_eq!(got, [2.0, 3.0]);

    writer.apply_macro(vec![inverse]).unwrap();
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), 1);
    let TrackItem::Clip(clip) = &writer.snapshot().tracks[0].items[0] else {
        panic!("fixture clip");
    };
    assert_eq!(
        clip.envelope.transform.scale,
        DocParam::const_vec2([2.0, 3.0])
    );
}

#[test]
fn scale_const_f64_is_type_mismatch() {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("target").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.scale = DocParam::const_f64(1.0);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::ZERO,
            duration: RationalTime::from_seconds(10),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    let err =
        prepare_add_transform_param_key(&doc, layer, ScalarPropertyId::Scale, RationalTime::ZERO)
            .expect_err("F64 scale must fail");
    assert!(matches!(
        err,
        AddTransformParamKeyPrepareError::ValueTypeMismatch { .. }
    ));
}

#[test]
fn position_source_param_and_effect_param_are_rejected() {
    let (doc, layer) = clip_doc(|env| env.transform.scale = DocParam::const_vec2([1.0, 1.0]));
    let t = RationalTime::ZERO;
    for property in [
        ScalarPropertyId::Position,
        ScalarPropertyId::SourceParam("count".into()),
        ScalarPropertyId::EffectParam(EffectId::from_raw(0), "amount".into()),
    ] {
        let err = prepare_add_transform_param_key(&doc, layer, property.clone(), t)
            .expect_err("unsupported property");
        assert!(
            matches!(
                err,
                AddTransformParamKeyPrepareError::PropertyUnsupported { .. }
            ),
            "unexpected {err:?} for {property:?}"
        );
    }
}

#[test]
fn rotation_and_opacity_const_emit_set_property_f64_keyframes() {
    let (doc, layer) = clip_doc(|env| {
        env.transform.rotation = DocParam::const_f64(0.25);
        env.opacity = DocParam::const_f64(0.5);
    });
    let t = RationalTime::from_seconds(2);

    for (property, expected) in [
        (ScalarPropertyId::Rotation, 0.25),
        (ScalarPropertyId::Opacity, 0.5),
    ] {
        let prepared = prepare_add_transform_param_key(&doc, layer, property.clone(), t).unwrap();
        let command = match prepared {
            AddTransformParamKeyPreparation::Prepared { command, .. } => command,
            AddTransformParamKeyPreparation::AlreadyPresent { .. } => panic!("const must prepare"),
        };
        let Command::SetProperty {
            property: got_prop,
            new_value: DocParam::Keyframes(track),
            ..
        } = command
        else {
            panic!("expected SetProperty Keyframes for {property:?}");
        };
        assert_eq!(got_prop, property);
        assert_eq!(track.keys()[0].value, DocValue::F64(expected));
        let got = eval_f64(
            &DocParam::Keyframes(track),
            t,
            &DataTracks::new(),
            &ResolvedLayerParams::default(),
        )
        .unwrap();
        assert_eq!(got, expected);
    }
}

#[test]
fn set_scale_key_value_replaces_only_that_key() {
    let t0 = RationalTime::ZERO;
    let t1 = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t0, [1.0, 1.0]), (1, t1, [2.0, 3.0])]);
    });
    let command = prepare_set_transform_param_key_value(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        KeyframeId::from_raw(1),
        DocValue::Vec2([4.0, 5.0]),
    )
    .unwrap()
    .expect("changed value must emit SetProperty");
    let Command::SetProperty {
        property,
        new_value,
        ..
    } = &command
    else {
        panic!("Scale key value must be SetProperty, not a new Command variant");
    };
    assert_eq!(*property, ScalarPropertyId::Scale);
    let DocParam::Keyframes(track) = new_value else {
        panic!("must stay Keyframes");
    };
    assert_eq!(track.keys().len(), 2);
    assert_eq!(track.keys()[0].id, KeyframeId::from_raw(0));
    assert_eq!(track.keys()[0].t, t0);
    assert_eq!(track.keys()[0].value, DocValue::Vec2([1.0, 1.0]));
    assert_eq!(track.keys()[0].interp, Interp::Linear);
    assert_eq!(track.keys()[1].id, KeyframeId::from_raw(1));
    assert_eq!(track.keys()[1].t, t1);
    assert_eq!(track.keys()[1].value, DocValue::Vec2([4.0, 5.0]));
    assert_eq!(track.keys()[1].interp, Interp::Linear);

    let mut live = doc;
    command.apply(&mut live).unwrap();
    let TrackItem::Clip(clip) = &live.tracks[0].items[0] else {
        panic!("fixture clip");
    };
    assert_eq!(
        eval_vec2(
            &clip.envelope.transform.scale,
            t0,
            &DataTracks::new(),
            &ResolvedLayerParams::default(),
        )
        .unwrap(),
        [1.0, 1.0]
    );
    assert_eq!(
        eval_vec2(
            &clip.envelope.transform.scale,
            t1,
            &DataTracks::new(),
            &ResolvedLayerParams::default(),
        )
        .unwrap(),
        [4.0, 5.0]
    );
}

#[test]
fn set_scale_key_same_value_is_none() {
    let t = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t, [2.0, 3.0])]);
    });
    let prepared = prepare_set_transform_param_key_value(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        KeyframeId::from_raw(0),
        DocValue::Vec2([2.0, 3.0]),
    )
    .unwrap();
    assert_eq!(prepared, None);
}

#[test]
fn remove_last_scale_key_collapses_to_const() {
    let t = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t, [2.0, 3.0])]);
    });
    let command = prepare_remove_transform_param_key(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        KeyframeId::from_raw(0),
    )
    .unwrap();
    let Command::SetProperty { new_value, .. } = &command else {
        panic!("remove must be SetProperty, not a new Command variant");
    };
    assert_eq!(new_value, &DocParam::const_vec2([2.0, 3.0]));
}

#[test]
fn writer_apply_macro_remove_does_not_rewind_counter() {
    let t = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t, [2.0, 3.0])]);
    });
    assert_eq!(doc.next_stable_id.peek_next(), 1);
    let mut writer = writer(doc);
    let command = writer
        .prepare_remove_transform_param_key(layer, ScalarPropertyId::Scale, KeyframeId::from_raw(0))
        .unwrap();
    writer.apply_macro(vec![command]).unwrap();
    assert_eq!(writer.snapshot().next_stable_id.peek_next(), 1);
    let TrackItem::Clip(clip) = &writer.snapshot().tracks[0].items[0] else {
        panic!("fixture clip");
    };
    assert_eq!(
        clip.envelope.transform.scale,
        DocParam::const_vec2([2.0, 3.0])
    );
}

#[test]
fn set_and_remove_reject_missing_wrong_type_and_unsupported_property() {
    let t = RationalTime::from_seconds(1);
    let (doc, layer) = clip_doc(|env| {
        env.transform.scale = scale_keyframes(vec![(0, t, [2.0, 3.0])]);
    });

    let missing = prepare_set_transform_param_key_value(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        KeyframeId::from_raw(99),
        DocValue::Vec2([1.0, 1.0]),
    )
    .expect_err("missing key");
    assert!(matches!(
        missing,
        SetTransformParamKeyValuePrepareError::KeyNotFound { .. }
    ));

    let wrong_type = prepare_set_transform_param_key_value(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        KeyframeId::from_raw(0),
        DocValue::F64(1.0),
    )
    .expect_err("F64 is not Scale");
    assert!(matches!(
        wrong_type,
        SetTransformParamKeyValuePrepareError::ValueTypeMismatch { .. }
    ));

    for property in [
        ScalarPropertyId::Position,
        ScalarPropertyId::SourceParam("count".into()),
        ScalarPropertyId::EffectParam(EffectId::from_raw(0), "amount".into()),
    ] {
        let set_err = prepare_set_transform_param_key_value(
            &doc,
            layer,
            property.clone(),
            KeyframeId::from_raw(0),
            DocValue::Vec2([1.0, 1.0]),
        )
        .expect_err("unsupported property");
        assert!(
            matches!(
                set_err,
                SetTransformParamKeyValuePrepareError::PropertyUnsupported { .. }
            ),
            "unexpected {set_err:?} for {property:?}"
        );
        let remove_err = prepare_remove_transform_param_key(
            &doc,
            layer,
            property.clone(),
            KeyframeId::from_raw(0),
        )
        .expect_err("unsupported property");
        assert!(
            matches!(
                remove_err,
                RemoveTransformParamKeyPrepareError::PropertyUnsupported { .. }
            ),
            "unexpected {remove_err:?} for {property:?}"
        );
    }
}
