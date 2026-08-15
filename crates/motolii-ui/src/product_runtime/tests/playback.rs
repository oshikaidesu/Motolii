use super::*;
use crate::{AsciiKey, CommandId, KeymapDelta, Modifier, PlatformBindingConstraints};
use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_doc::{DocKeyframe, DocKeyframeTrack};
use motolii_transport::Transport;

#[test]
fn playback_lifecycle_cancels_stale_preparation_and_allows_one_session() {
    let mut lifecycle = PlaybackLifecycle::default();
    let stale_generation = lifecycle.begin_preparing().unwrap();
    assert!(lifecycle.accepts_preparation(stale_generation));
    lifecycle.cancel_preparing().unwrap();
    assert!(!lifecycle.accepts_preparation(stale_generation));

    let active_generation = lifecycle.begin_preparing().unwrap();
    lifecycle.activate(active_generation).unwrap();
    assert_eq!(lifecycle.state(), StagePlaybackState::Playing);
    assert!(lifecycle.activate(active_generation).is_err());
    lifecycle.invalidate().unwrap();
    assert_eq!(lifecycle.state(), StagePlaybackState::Idle);
    assert!(!lifecycle.session_active);
}

#[test]
fn canonical_playback_start_frame_uses_exact_zero_and_sample_rate_conversion() {
    assert_eq!(
        canonical_playback_start_frame(RationalTime::ZERO).unwrap(),
        0
    );
    assert_eq!(
        canonical_playback_start_frame(RationalTime::try_new(1, 24).unwrap()).unwrap(),
        2_000
    );
    assert_eq!(
        canonical_playback_start_frame(RationalTime::try_new(1, 48_000).unwrap()).unwrap(),
        1
    );
    assert!(matches!(
        canonical_playback_start_frame(RationalTime::try_new(-1, 48_000).unwrap()),
        Err(ProductPlaybackError::NegativePlayhead)
    ));
}

#[test]
fn transport_reports_absolute_time_and_repeats_without_counter_advance() {
    for sample_rate in [48_000_u32, 44_100_u32] {
        let counters = Arc::new(PlaybackCounters::default());
        counters.advance_supplied_for_simulation(u64::from(sample_rate) * 2);
        let mut transport = Transport::new(
            counters,
            Arc::new(DeviceWaitLatency::default()),
            Fps::try_new(30, 1).unwrap(),
            sample_rate,
            RationalTime::try_new(1, 1).unwrap(),
            Quality::DRAFT,
            false,
        )
        .unwrap();
        let first = transport.next_frame_plan().unwrap();
        let repeated = transport.next_frame_plan().unwrap();

        assert_eq!(first.timeline_time, RationalTime::try_new(3, 1).unwrap());
        assert_eq!(repeated.timeline_time, first.timeline_time);
    }
}
#[test]
fn stage_transport_snapshot_depends_only_on_document_primary_and_playhead() {
    let (document, layer, _) = position_keyframe_document();
    let interior = RationalTime::try_new(1, 1).unwrap();
    let outside = RationalTime::try_new(3, 1).unwrap();
    let active =
        serde_json::to_value(stage_transport_snapshot(&document, Some(layer), interior)).unwrap();
    assert_eq!(
        active["activeInterval"],
        serde_json::json!({ "objectName": "static-preview", "channel": "Position" })
    );
    assert_eq!(
        serde_json::to_value(stage_transport_snapshot(&document, None, interior)).unwrap()
            ["activeInterval"],
        serde_json::Value::Null,
    );
    assert_eq!(
        serde_json::to_value(stage_transport_snapshot(&document, Some(layer), outside)).unwrap()
            ["activeInterval"],
        serde_json::Value::Null,
    );
    let mut replaced = document.clone();
    match &mut replaced.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            clip.envelope.transform.position = DocParam::const_vec2([0.0, 0.0])
        }
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    assert_eq!(
        serde_json::to_value(stage_transport_snapshot(&replaced, Some(layer), interior)).unwrap()
            ["activeInterval"],
        serde_json::Value::Null,
    );

    let missing_name_layer = LayerId::from_raw(999);
    let mut missing_name = document.clone();
    match &mut missing_name.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip.envelope.layer_id = missing_name_layer,
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    assert_eq!(
        serde_json::to_value(stage_transport_snapshot(
            &missing_name,
            Some(missing_name_layer),
            interior,
        ))
        .unwrap()["activeInterval"],
        serde_json::Value::Null,
    );

    let mut empty_name = document.clone();
    let empty_name_layer = empty_name.layers.allocate("").unwrap();
    match &mut empty_name.tracks[0].items[0] {
        TrackItem::Clip(clip) => clip.envelope.layer_id = empty_name_layer,
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    assert_eq!(
        serde_json::to_value(stage_transport_snapshot(
            &empty_name,
            Some(empty_name_layer),
            interior,
        ))
        .unwrap()["activeInterval"],
        serde_json::Value::Null,
    );
}

#[test]
fn stage_transport_publish_delivers_one_exact_snapshot_without_document_write() {
    let (document, layer, _) = position_keyframe_document();
    let before = serde_json::to_vec(&document).unwrap();
    let mut published = Vec::new();
    publish_stage_transport_snapshot(
        &document,
        Some(layer),
        RationalTime::try_new(1, 1).unwrap(),
        |snapshot| {
            published.push(snapshot.clone());
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(
        serde_json::to_value(&published[0]).unwrap()["activeInterval"],
        serde_json::json!({ "objectName": "static-preview", "channel": "Position" })
    );
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);

    published.clear();
    publish_stage_transport_snapshot(&document, None, RationalTime::ZERO, |snapshot| {
        published.push(snapshot.clone());
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(
        serde_json::to_value(&published[0]).unwrap()["activeInterval"],
        serde_json::Value::Null,
    );
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);
}
