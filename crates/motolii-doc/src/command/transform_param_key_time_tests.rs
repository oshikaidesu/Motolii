use super::*;
use crate::doc_keyframe::DocKeyframeTrack;
use crate::position_key_prepare::{
    prepare_set_transform_param_key_time, SetTransformParamKeyTimePrepareError,
};
use crate::{Clip, ClipSource, Document, ItemEnvelope, Track, TrackItem};
use motolii_core::RationalTime;
use motolii_eval::Interp;

const CLIP_START_SECONDS: i64 = 1;

/// Scale に2つの key を載せた clip。`clip.start` は 0 以外にして、key 移動が
/// clip を動かさないことを見えるようにする。
fn scale_keyed_doc() -> (Document, LayerId, KeyframeId, KeyframeId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("a").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();

    let k0 = KeyframeId::from_raw(0);
    let k1 = KeyframeId::from_raw(1);
    let mut track = DocKeyframeTrack::new();
    track.insert(DocKeyframe {
        id: k0,
        t: RationalTime::from_seconds(2),
        value: DocValue::Vec2([1.0, 1.0]),
        interp: Interp::Linear,
    });
    track.insert(DocKeyframe {
        id: k1,
        t: RationalTime::from_seconds(5),
        value: DocValue::Vec2([2.0, 2.0]),
        interp: Interp::Hold,
    });

    let mut envelope = ItemEnvelope::new(layer);
    envelope.transform.scale = DocParam::Keyframes(track);
    while doc.next_stable_id.peek_next() <= k1.get() {
        doc.next_stable_id.allocate().unwrap();
    }
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope,
            start: RationalTime::from_seconds(CLIP_START_SECONDS),
            duration: RationalTime::from_seconds(8),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().unwrap();
    (doc, layer, k0, k1)
}

fn scale_key_snapshot(
    doc: &Document,
    layer: LayerId,
    key: KeyframeId,
) -> (RationalTime, DocValue, Interp) {
    let env = find_envelope(doc, layer).unwrap();
    let DocParam::Keyframes(track) = &env.transform.scale else {
        panic!("scale must stay keyframed");
    };
    let k = track.get_by_id(key).unwrap();
    (k.t, k.value.clone(), k.interp)
}

fn clip_start(doc: &Document) -> RationalTime {
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("first item must be a clip");
    };
    clip.start
}

#[test]
fn set_transform_param_key_time_moves_only_that_key() {
    let (mut doc, layer, k0, k1) = scale_keyed_doc();
    let other_before = scale_key_snapshot(&doc, layer, k1);
    let start_before = clip_start(&doc);

    let command = prepare_set_transform_param_key_time(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        k0,
        RationalTime::from_seconds(3),
    )
    .unwrap()
    .unwrap();
    let Command::SetTransformParamKeyTime {
        target,
        property,
        key,
        old,
        new,
    } = &command
    else {
        panic!("expected SetTransformParamKeyTime");
    };
    assert_eq!(*target, layer);
    assert_eq!(*property, ScalarPropertyId::Scale);
    assert_eq!(*key, k0);
    assert_eq!(*old, RationalTime::from_seconds(2));
    assert_eq!(*new, RationalTime::from_seconds(3));

    command.apply(&mut doc).unwrap();

    let (t0, v0, i0) = scale_key_snapshot(&doc, layer, k0);
    assert_eq!(t0, RationalTime::from_seconds(3));
    assert_eq!(v0, DocValue::Vec2([1.0, 1.0]));
    assert_eq!(i0, Interp::Linear);
    assert_eq!(scale_key_snapshot(&doc, layer, k1), other_before);
    assert_eq!(clip_start(&doc), start_before);
}

#[test]
fn set_transform_param_key_time_is_a_noop_when_unchanged() {
    let (doc, layer, k0, _) = scale_keyed_doc();
    assert!(prepare_set_transform_param_key_time(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        k0,
        RationalTime::from_seconds(2),
    )
    .unwrap()
    .is_none());

    let before = serde_json::to_vec(&doc).unwrap();
    let mut live = doc.clone();
    Command::SetTransformParamKeyTime {
        target: layer,
        property: ScalarPropertyId::Scale,
        key: k0,
        old: RationalTime::from_seconds(2),
        new: RationalTime::from_seconds(2),
    }
    .apply(&mut live)
    .unwrap();
    assert_eq!(serde_json::to_vec(&live).unwrap(), before);
}

#[test]
fn set_transform_param_key_time_round_trips_through_undo() {
    let (mut doc, layer, k0, _) = scale_keyed_doc();
    let before = serde_json::to_vec(&doc).unwrap();
    let command = prepare_set_transform_param_key_time(
        &doc,
        layer,
        ScalarPropertyId::Scale,
        k0,
        RationalTime::from_seconds(4),
    )
    .unwrap()
    .unwrap();
    command.apply(&mut doc).unwrap();
    assert_ne!(serde_json::to_vec(&doc).unwrap(), before);

    let inverse = command.inverse();
    let Command::SetTransformParamKeyTime { old, new, .. } = &inverse else {
        panic!("inverse must stay the same variant");
    };
    assert_eq!(*old, RationalTime::from_seconds(4));
    assert_eq!(*new, RationalTime::from_seconds(2));

    inverse.apply(&mut doc).unwrap();
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);
}

#[test]
fn set_transform_param_key_time_rejects_cas_occupied_missing_without_mutation() {
    let (doc, layer, k0, k1) = scale_keyed_doc();
    let before = serde_json::to_vec(&doc).unwrap();

    let cas = Command::SetTransformParamKeyTime {
        target: layer,
        property: ScalarPropertyId::Scale,
        key: k0,
        old: RationalTime::from_seconds(9),
        new: RationalTime::from_seconds(3),
    };
    let mut live = doc.clone();
    assert!(matches!(
        cas.apply(&mut live),
        Err(CommandError::TransformParamKeyTimePayloadMismatch { .. })
    ));
    assert_eq!(serde_json::to_vec(&live).unwrap(), before);

    assert!(matches!(
        prepare_set_transform_param_key_time(
            &doc,
            layer,
            ScalarPropertyId::Scale,
            k0,
            RationalTime::from_seconds(5),
        ),
        Err(SetTransformParamKeyTimePrepareError::Command(
            CommandError::TransformParamKeyTimeOccupied { .. }
        ))
    ));

    let missing = KeyframeId::from_raw(k1.get().saturating_add(99));
    assert!(matches!(
        prepare_set_transform_param_key_time(
            &doc,
            layer,
            ScalarPropertyId::Scale,
            missing,
            RationalTime::from_seconds(3),
        ),
        Err(SetTransformParamKeyTimePrepareError::Command(
            CommandError::TransformParamKeyTimeNotFound { .. }
        ))
    ));

    assert!(matches!(
        prepare_set_transform_param_key_time(
            &doc,
            layer,
            ScalarPropertyId::Scale,
            k0,
            RationalTime::try_new(-1, 1).unwrap(),
        ),
        Err(SetTransformParamKeyTimePrepareError::Command(
            CommandError::TransformParamKeyTimeNegative { .. }
        ))
    ));

    // Position も property としては受け付ける(2026-08-16 に受け付け集合を統合)。
    // **時刻を動かすのに値の型は要らない**ので、envelope が持つ property は全部通る。
    // ここで落ちる理由は property ではなく、この fixture の Position が key 列を
    // 持たないこと。**「property が範囲外」と「その property が key 列を持たない」は別。**
    let position_result = prepare_set_transform_param_key_time(
        &doc,
        layer,
        ScalarPropertyId::Position,
        k0,
        RationalTime::from_seconds(3),
    );
    assert!(
        !matches!(
            position_result,
            Err(SetTransformParamKeyTimePrepareError::PropertyUnsupported { .. })
        ),
        "Position は property として受け付けられるべき。実際: {position_result:?}"
    );

    // plugin 由来の property は envelope の外にあるので、まだ到達できない。
    // 広げるには catalog を引いて型を決める必要がある(台帳の「決定待ち」)。
    assert!(matches!(
        prepare_set_transform_param_key_time(
            &doc,
            layer,
            ScalarPropertyId::SourceParam("whatever".to_owned()),
            k0,
            RationalTime::from_seconds(3),
        ),
        Err(SetTransformParamKeyTimePrepareError::PropertyUnsupported { .. })
    ));

    // Rotation は Const のまま — key 列を持たない source は typed 拒否。
    assert!(matches!(
        prepare_set_transform_param_key_time(
            &doc,
            layer,
            ScalarPropertyId::Rotation,
            k0,
            RationalTime::from_seconds(3),
        ),
        Err(SetTransformParamKeyTimePrepareError::Command(
            CommandError::TransformParamKeyTimeSourceUnsupported { .. }
        ))
    ));

    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);
}
