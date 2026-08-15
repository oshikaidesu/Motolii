use super::*;
use crate::position_key_prepare::prepare_add_position_key;
use crate::{Clip, ClipSource, Document, ItemEnvelope, Track, TrackItem};
use motolii_core::RationalTime;

fn keyed_doc_two_keys() -> (Document, LayerId, KeyframeId, KeyframeId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("a").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(10, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().unwrap();
    let crate::AddPositionKeyPreparation::Prepared {
        key_id: k0,
        command: c0,
    } = prepare_add_position_key(&doc, layer, RationalTime::from_seconds(2)).unwrap()
    else {
        panic!("key0");
    };
    c0.apply(&mut doc).unwrap();
    let crate::AddPositionKeyPreparation::Prepared {
        key_id: k1,
        command: c1,
    } = prepare_add_position_key(&doc, layer, RationalTime::from_seconds(5)).unwrap()
    else {
        panic!("key1");
    };
    c1.apply(&mut doc).unwrap();
    (doc, layer, k0, k1)
}

fn key_snapshot(
    doc: &Document,
    layer: LayerId,
    key: KeyframeId,
) -> (RationalTime, DocValue, motolii_eval::Interp) {
    let env = find_envelope(doc, layer).unwrap();
    let DocParam::Keyframes(track) = &env.transform.position else {
        panic!("keyframes");
    };
    let k = track.get_by_id(key).unwrap();
    (k.t, k.value.clone(), k.interp)
}

#[test]
fn set_position_key_time_forward_inverse_restores_document() {
    let (mut doc, layer, k0, k1) = keyed_doc_two_keys();
    let before = serde_json::to_vec(&doc).unwrap();
    let (t1, v1, i1) = key_snapshot(&doc, layer, k1);
    let command = prepare_set_position_key_time(&doc, layer, k0, RationalTime::from_seconds(3))
        .unwrap()
        .unwrap();
    command.apply(&mut doc).unwrap();
    let (t0, v0, i0) = key_snapshot(&doc, layer, k0);
    assert_eq!(t0, RationalTime::from_seconds(3));
    assert_eq!(v0, DocValue::Vec2([0.0, 0.0]));
    assert_eq!(i0, motolii_eval::Interp::Linear);
    assert_eq!(key_snapshot(&doc, layer, k1), (t1, v1, i1));
    command.inverse().apply(&mut doc).unwrap();
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);
}

#[test]
fn set_position_key_time_rejects_cas_occupied_missing_without_mutation() {
    let (doc, layer, k0, k1) = keyed_doc_two_keys();
    let before = serde_json::to_vec(&doc).unwrap();

    let cas = Command::SetPositionKeyTime {
        target: layer,
        key: k0,
        old: RationalTime::from_seconds(9),
        new: RationalTime::from_seconds(3),
    };
    let mut live = doc.clone();
    assert!(matches!(
        cas.apply(&mut live),
        Err(CommandError::PositionKeyTimePayloadMismatch { .. })
    ));
    assert_eq!(serde_json::to_vec(&live).unwrap(), before);

    assert!(matches!(
        prepare_set_position_key_time(&doc, layer, k0, RationalTime::from_seconds(5)),
        Err(CommandError::PositionKeyTimeOccupied { .. })
    ));
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);

    let missing = KeyframeId::from_raw(k1.get().saturating_add(99));
    assert!(matches!(
        prepare_set_position_key_time(&doc, layer, missing, RationalTime::from_seconds(3)),
        Err(CommandError::PositionKeyTimeNotFound { .. })
    ));
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);

    assert!(matches!(
        prepare_set_position_key_time(&doc, layer, k0, RationalTime::try_new(-1, 1).unwrap()),
        Err(CommandError::PositionKeyTimeNegative { .. })
    ));
    assert_eq!(serde_json::to_vec(&doc).unwrap(), before);
}

#[test]
fn set_position_key_time_same_time_is_noop() {
    let (doc, layer, k0, _) = keyed_doc_two_keys();
    assert!(
        prepare_set_position_key_time(&doc, layer, k0, RationalTime::from_seconds(2))
            .unwrap()
            .is_none()
    );
    let mut live = doc.clone();
    let before = serde_json::to_vec(&doc).unwrap();
    Command::SetPositionKeyTime {
        target: layer,
        key: k0,
        old: RationalTime::from_seconds(2),
        new: RationalTime::from_seconds(2),
    }
    .apply(&mut live)
    .unwrap();
    assert_eq!(serde_json::to_vec(&live).unwrap(), before);
}
