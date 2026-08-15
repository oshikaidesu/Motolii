use super::*;
use crate::{AsciiKey, CommandId, KeymapDelta, Modifier, PlatformBindingConstraints};
use motolii_audio::{DeviceWaitLatency, PlaybackCounters};
use motolii_core::{ColorSpace, Fps, FrameDesc, PixelFormat};
use motolii_doc::{DocKeyframe, DocKeyframeTrack};
use motolii_transport::Transport;

#[test]
fn position_active_interval_returns_exact_strict_interior_identity_without_document_write() {
    let (document, layer, ids) = position_keyframe_document();
    let before = serde_json::to_vec(&document).unwrap();
    let playhead = RationalTime::try_new(1, 1).unwrap();

    assert_eq!(
        position_active_interval(&document, Some(layer), playhead),
        Some(PositionActiveInterval {
            layer,
            left_id: ids[0],
            left_t: RationalTime::ZERO,
            right_id: ids[1],
            right_t: RationalTime::try_new(2, 1).unwrap(),
            left_interp: motolii_eval::Interp::Linear,
        })
    );
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);
}

#[test]
fn position_key_value_gesture_requires_exact_key_and_unchanged_live_curve() {
    let (document, layer, ids) = position_keyframe_document();
    let before = serde_json::to_vec(&document).unwrap();
    let playhead = RationalTime::ZERO;
    let baseline = position_gesture_baseline(
        &document,
        Some(layer),
        playhead,
        InspectorPositionGestureStart {
            session: 1,
            sequence: 1,
            axis: InspectorPositionAxis::X,
            value: 0.0,
        },
    )
    .expect("exact current Vec2 key must admit Position editing");
    assert_eq!(baseline.key, ids[0]);
    assert_eq!(baseline.value, [0.0, 0.0]);
    assert_eq!(
        resolve_position_gesture_command(
            &document,
            Some(layer),
            playhead,
            &baseline,
            InspectorPositionAxis::X,
            0.25,
        ),
        Some(Command::SetPositionKeyValue {
            target: layer,
            key: ids[0],
            old: [0.0, 0.0],
            new: [0.25, 0.0],
        })
    );
    for (primary, time, axis, value) in [
        (
            Some(layer),
            RationalTime::try_new(1, 1).unwrap(),
            InspectorPositionAxis::X,
            0.25,
        ),
        (None, playhead, InspectorPositionAxis::X, 0.25),
        (Some(layer), playhead, InspectorPositionAxis::X, 0.0),
        (Some(layer), playhead, InspectorPositionAxis::X, f64::NAN),
    ] {
        assert!(
            resolve_position_gesture_command(&document, primary, time, &baseline, axis, value,)
                .is_none()
        );
    }
    let mut changed_curve = document.clone();
    match &mut changed_curve.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            let DocParam::Keyframes(track) = &mut clip.envelope.transform.position else {
                unreachable!();
            };
            let mut replacement = track.get_by_id(ids[1]).unwrap().clone();
            replacement.value = DocValue::Vec2([2.0, 2.0]);
            track.insert(replacement);
        }
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    assert!(resolve_position_gesture_command(
        &changed_curve,
        Some(layer),
        playhead,
        &baseline,
        InspectorPositionAxis::X,
        0.25,
    )
    .is_none());
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);
}

#[test]
fn easing_admission_rejects_no_interval_stale_duplicate_identity_and_same_value_before_queue() {
    let (document, layer, ids) = position_keyframe_document();
    let interval =
        position_active_interval(&document, Some(layer), RationalTime::try_new(1, 1).unwrap())
            .unwrap();
    assert_eq!(
        admit_easing_open(false, Some(7), 7, None),
        Err(EasingOpenReject::NoActiveInterval),
    );
    assert_eq!(
        admit_easing_open(true, Some(7), 7, Some(interval.clone())),
        Err(EasingOpenReject::PopupActive),
    );
    assert_eq!(
        admit_easing_open(false, Some(8), 7, Some(interval.clone())),
        Err(EasingOpenReject::LayoutEpochMismatch),
    );

    let replacement = motolii_eval::Interp::Bezier {
        x1: 0.4,
        y1: 0.0,
        x2: 0.2,
        y2: 1.0,
    };
    let accepted =
        admit_easing_terminal(11, Some(7), Some(&interval), 11, 7, &interval, replacement).unwrap();
    assert_eq!(
        accepted,
        SetPositionKeyInterpRequest {
            target: layer,
            key: ids[0],
            interp: replacement,
        },
    );

    let mut mismatched_interval = interval.clone();
    mismatched_interval.left_id = KeyframeId::from_raw(999);
    let rejected = [
        admit_easing_terminal(12, Some(7), Some(&interval), 11, 7, &interval, replacement),
        admit_easing_terminal(11, Some(8), Some(&interval), 11, 7, &interval, replacement),
        admit_easing_terminal(
            11,
            Some(7),
            Some(&mismatched_interval),
            11,
            7,
            &interval,
            replacement,
        ),
        admit_easing_terminal(
            11,
            Some(7),
            Some(&interval),
            11,
            7,
            &interval,
            interval.left_interp,
        ),
    ];
    assert_eq!(rejected[0], Err(EasingTerminalReject::GenerationMismatch),);
    assert_eq!(rejected[1], Err(EasingTerminalReject::LayoutEpochMismatch),);
    assert_eq!(rejected[2], Err(EasingTerminalReject::IntervalMismatch));
    assert_eq!(rejected[3], Err(EasingTerminalReject::SameValue));
    let mut queued = Vec::new();
    for request in rejected.into_iter().flatten() {
        queued.push(request);
    }
    assert!(queued.is_empty());
}

#[test]
fn easing_popup_static_gpu_proof_retains_only_product_owned_gpu_parts() {
    let product = PRODUCTION_SOURCE;
    let popup = include_str!("../../product_easing_popup.rs");

    assert!(product.contains("instance: wgpu::Instance,"));
    assert!(product.contains("adapter: wgpu::Adapter,"));
    assert!(product.contains("&gfx.instance,"));
    assert!(product.contains("&gfx.adapter,"));
    assert!(product.contains("Arc::clone(&self.gpu),"));
    assert!(popup.contains("instance.create_surface(Arc::clone(&window))"));
    assert!(popup.contains("Renderer::new(&gpu.device"));
    assert!(popup.contains("&self.gpu.queue"));
    assert!(!popup.contains("request_device"));
    assert!(!popup.contains("EventLoop::new"));
}

#[test]
fn position_active_interval_rejects_missing_primary_endpoints_and_non_vec2_position() {
    let (mut document, layer, _) = position_keyframe_document();

    assert_eq!(
        position_active_interval(&document, None, RationalTime::ZERO),
        None
    );
    assert_eq!(
        position_active_interval(&document, Some(layer), RationalTime::ZERO),
        None
    );
    assert_eq!(
        position_active_interval(&document, Some(layer), RationalTime::try_new(2, 1).unwrap(),),
        None
    );

    match &mut document.tracks[0].items[0] {
        TrackItem::Clip(clip) => {
            clip.envelope.transform.position = DocParam::const_f64(1.0);
        }
        TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
    }
    let before_unsupported = serde_json::to_vec(&document).unwrap();
    assert_eq!(
        position_active_interval(&document, Some(layer), RationalTime::try_new(1, 1).unwrap()),
        None
    );
    assert_eq!(serde_json::to_vec(&document).unwrap(), before_unsupported);
}

#[test]
fn position_active_interval_fails_closed_for_every_non_position_or_incomplete_track_shape() {
    let (document, layer, ids) = position_keyframe_document();
    let interior = RationalTime::try_new(1, 1).unwrap();
    for playhead in [
        RationalTime::try_new(-1, 1).unwrap(),
        RationalTime::try_new(3, 1).unwrap(),
    ] {
        let before = serde_json::to_vec(&document).unwrap();
        assert_eq!(
            position_active_interval(&document, Some(layer), playhead),
            None
        );
        assert_eq!(serde_json::to_vec(&document).unwrap(), before);
    }

    let zero_keys = DocKeyframeTrack::new();
    let mut one_key = DocKeyframeTrack::new();
    one_key.insert(DocKeyframe {
        id: ids[0],
        t: RationalTime::ZERO,
        value: DocValue::Vec2([0.0, 0.0]),
        interp: motolii_eval::Interp::Linear,
    });
    let mut non_vec2 = DocKeyframeTrack::new();
    non_vec2.insert(DocKeyframe {
        id: ids[0],
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: motolii_eval::Interp::Linear,
    });
    non_vec2.insert(DocKeyframe {
        id: ids[1],
        t: RationalTime::try_new(2, 1).unwrap(),
        value: DocValue::F64(1.0),
        interp: motolii_eval::Interp::Linear,
    });
    let variants = [
        DocParam::Keyframes(zero_keys),
        DocParam::Keyframes(one_key),
        DocParam::Keyframes(non_vec2),
        DocParam::Vec2Axes {
            x: Box::new(DocParam::const_f64(0.0)),
            y: Box::new(DocParam::const_f64(0.0)),
        },
        DocParam::Data {
            track: motolii_eval::DataTrackId("position".to_owned()),
            fallback: DocValue::Vec2([0.0, 0.0]),
        },
        DocParam::LookAt {
            target: layer,
            axis: motolii_doc::LookAtAxis::PlusY,
        },
        DocParam::Follow {
            target: layer,
            offset: [0.0, 0.0],
        },
    ];
    for param in variants {
        let mut case = document.clone();
        match &mut case.tracks[0].items[0] {
            TrackItem::Clip(clip) => clip.envelope.transform.position = param,
            TrackItem::Group(_) => unreachable!("bootstrap fixture is a clip"),
        }
        let before = serde_json::to_vec(&case).unwrap();
        assert_eq!(position_active_interval(&case, Some(layer), interior), None);
        assert_eq!(serde_json::to_vec(&case).unwrap(), before);
    }

    let before = serde_json::to_vec(&document).unwrap();
    assert_eq!(
        position_active_interval(&document, Some(LayerId::from_raw(999)), interior),
        None
    );
    assert_eq!(serde_json::to_vec(&document).unwrap(), before);
}
