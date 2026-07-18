#![allow(deprecated)]

//! AG-1: Asset Clip video/audio component schema / 互換 / 拒否。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    asset_components_require_newer_reader, migrate_bytes, AudioComponent, AudioOutOfRange, Clip,
    ClipSource, CompCameraDoc, Document, DocumentError, ItemEnvelope, StreamKind, StreamSelector,
    Track, TrackItem, VideoComponent, MIN_READER_VERSION_FOR_COMP_CAMERA, READER_VERSION,
    WRITER_VERSION,
};
use serde_json::json;

/// Legacy v2 wire without composition.camera (D1e migrate entry).
fn legacy_v2_wire_without_camera(doc: &Document) -> Vec<u8> {
    let mut value = serde_json::to_value(doc).unwrap();
    value["version"] = json!(2);
    value["min_reader_version"] = json!(2);
    value["composition"]
        .as_object_mut()
        .unwrap()
        .remove("camera");
    serde_json::to_vec(&value).unwrap()
}

fn push_asset_clip(doc: &mut Document, source: ClipSource) {
    let layer = doc.layers.allocate("clip").unwrap();
    let track_id = if doc.tracks.is_empty() {
        let tid = doc.track_ids.allocate("V1").unwrap();
        doc.tracks.push(Track {
            id: tid,
            items: Vec::new(),
        });
        tid
    } else {
        doc.tracks[0].id
    };
    let _ = track_id;
    doc.tracks[0].items.push(TrackItem::Clip(Clip {
        envelope: ItemEnvelope::new(layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(1, 1).unwrap(),
        time_map: TimeMap::default(),
        source,
    }));
}

fn register_asset(doc: &mut Document) -> motolii_doc::AssetId {
    let id = motolii_doc::AssetId::from_raw(0);
    doc.assets
        .insert(motolii_doc::Asset {
            id,
            name: "a".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:aa".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    id
}

#[test]
fn legacy_asset_json_defaults_to_video_only() {
    let v = json!({
        "source": "asset",
        "asset": 0
    });
    let source: ClipSource = serde_json::from_value(v).unwrap();
    match source {
        ClipSource::Asset { video, audio, .. } => {
            assert_eq!(video, Some(VideoComponent::ordinal(0)));
            assert!(audio.is_empty());
            assert!(!asset_components_require_newer_reader(&video, &audio));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn legacy_asset_roundtrip_omits_default_components() {
    let source = ClipSource::asset_video_only(motolii_doc::AssetId::from_raw(7));
    let v = serde_json::to_value(&source).unwrap();
    assert_eq!(
        v,
        json!({
            "source": "asset",
            "asset": 7
        })
    );
    let back: ClipSource = serde_json::from_value(v).unwrap();
    assert_eq!(back, source);
}

#[test]
fn audio_component_roundtrip_preserves_fields() {
    let source = ClipSource::Asset {
        asset: motolii_doc::AssetId::from_raw(1),
        video: Some(VideoComponent::ordinal(0)),
        audio: vec![AudioComponent {
            stream: StreamSelector {
                kind: StreamKind::Audio,
                ordinal: 0,
            },
            enabled: false,
            gain: motolii_doc::DocParam::const_f64(0.5),
            out_of_range: AudioOutOfRange::Loop,
        }],
    };
    let v = serde_json::to_value(&source).unwrap();
    let back: ClipSource = serde_json::from_value(v).unwrap();
    assert_eq!(back, source);
}

#[test]
fn audio_only_serializes_null_video() {
    let source = ClipSource::Asset {
        asset: motolii_doc::AssetId::from_raw(2),
        video: None,
        audio: vec![AudioComponent::ordinal(0)],
    };
    let v = serde_json::to_value(&source).unwrap();
    assert_eq!(v["video"], json!(null));
    assert!(v["audio"].is_array());
    let back: ClipSource = serde_json::from_value(v).unwrap();
    assert_eq!(back, source);
}

#[test]
fn plugin_rejects_audio_component_field() {
    let v = json!({
        "source": "plugin",
        "plugin_id": "core.source.clear",
        "audio": [{"stream": {"kind": "audio", "ordinal": 0}}]
    });
    let err = serde_json::from_value::<ClipSource>(v).unwrap_err();
    assert!(
        err.to_string().contains("cannot carry video/audio"),
        "got {err}"
    );
}

#[test]
fn vector_rejects_audio_component_field() {
    let v = json!({
        "source": "vector",
        "recipe": {
            "content": {
                "kind": "standard_shape",
                "shape": "rect",
                "width": {"const": {"F64": 0.5}},
                "height": {"const": {"F64": 0.5}}
            }
        },
        "audio": [{"stream": {"kind": "audio", "ordinal": 0}}]
    });
    let err = serde_json::from_value::<ClipSource>(v).unwrap_err();
    assert!(err.to_string().contains("unknown field"), "got {err}");
}

#[test]
fn validate_rejects_wrong_stream_kind() {
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: Some(VideoComponent {
                stream: StreamSelector {
                    kind: StreamKind::Audio,
                    ordinal: 0,
                },
            }),
            audio: Vec::new(),
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::VideoComponentKindMismatch { .. })
    ));
}

#[test]
fn validate_rejects_empty_components() {
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: None,
            audio: Vec::new(),
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::EmptyAssetComponents { .. })
    ));
}

#[test]
fn validate_requires_min_reader_for_audio_components() {
    // Typed v2+camera cannot reach AssetComponentsRequireNewerReader (camera gate runs first).
    // Legacy v2 wire without camera + audio components migrates to v5 with default camera.
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: Some(VideoComponent::ordinal(0)),
            audio: vec![AudioComponent::ordinal(0)],
        },
    );
    let bytes = legacy_v2_wire_without_camera(&doc);
    let (migrated, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"insert_default_comp_camera"));
    assert_eq!(migrated.version, WRITER_VERSION);
    assert_eq!(
        migrated.min_reader_version,
        MIN_READER_VERSION_FOR_COMP_CAMERA
    );
    assert_eq!(
        migrated.composition.camera,
        CompCameraDoc::default_planar_orthographic()
    );
    let TrackItem::Clip(clip) = &migrated.tracks[0].items[0] else {
        panic!("clip");
    };
    let ClipSource::Asset { audio, .. } = &clip.source else {
        panic!("asset");
    };
    assert_eq!(audio.len(), 1);
    migrated.validate().unwrap();
}

#[test]
fn validate_accepts_audio_components_with_reader_gate() {
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: Some(VideoComponent::ordinal(0)),
            audio: vec![AudioComponent::ordinal(0)],
        },
    );
    doc.validate().unwrap();
}

#[test]
fn legacy_v2_asset_migrates_to_current_and_validates() {
    // Legacy v2 wire (no camera) with video-only asset clip migrates to v5 and validates.
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(&mut doc, ClipSource::asset_video_only(asset));
    let bytes = legacy_v2_wire_without_camera(&doc);
    let (migrated, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"insert_default_comp_camera"));
    assert_eq!(migrated.version, WRITER_VERSION);
    assert_eq!(
        migrated.composition.camera,
        CompCameraDoc::default_planar_orthographic()
    );
    let TrackItem::Clip(clip) = &migrated.tracks[0].items[0] else {
        panic!("clip");
    };
    match &clip.source {
        ClipSource::Asset { video, audio, .. } => {
            assert_eq!(*video, Some(VideoComponent::ordinal(0)));
            assert!(audio.is_empty());
        }
        other => panic!("unexpected {other:?}"),
    }
    migrated.validate().unwrap();
}

#[test]
fn negative_gain_is_rejected() {
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    let mut audio = AudioComponent::ordinal(0);
    audio.gain = motolii_doc::DocParam::const_f64(-0.1);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: None,
            audio: vec![audio],
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::ValueOutOfRange { .. })
    ));
}

#[test]
fn open_mode_rejects_future_min_reader() {
    use motolii_doc::{classify_open_mode, OpenMode};
    assert_eq!(
        classify_open_mode(READER_VERSION, READER_VERSION + 1),
        OpenMode::Reject
    );
}

#[test]
fn non_zero_video_ordinal_is_rejected() {
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: Some(VideoComponent::ordinal(1)),
            audio: Vec::new(),
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::UnsupportedVideoStreamOrdinal { ordinal: 1, .. })
    ));
}

#[test]
fn animated_audio_gain_duplicate_issues_fresh_keyframe_ids() {
    use motolii_doc::{
        DocKeyframe, DocKeyframeTrack, DocParam, DocValue, DocumentWriter, KeyframeId,
    };
    use motolii_eval::Interp;

    let mut doc = Document::new_current();
    let asset = doc.assets.allocate("a", "audio/wav", "sha256:aa").unwrap();
    let layer = doc.layers.allocate("clip").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let k0 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let k1 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut keys = DocKeyframeTrack::new();
    keys.insert(DocKeyframe {
        id: k0,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    keys.insert(DocKeyframe {
        id: k1,
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(0.25),
        interp: Interp::Linear,
    });
    let mut audio = AudioComponent::ordinal(0);
    audio.gain = DocParam::Keyframes(keys);
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(1, 1).unwrap(),
            time_map: TimeMap::default(),
            source: ClipSource::Asset {
                asset,
                video: None,
                audio: vec![audio],
            },
        })],
    });
    doc.validate().expect("fixture");

    let mut writer = DocumentWriter::new(
        doc,
        std::sync::Arc::new(motolii_plugin::reference::reference_catalog().unwrap()),
    )
    .unwrap();
    writer.duplicate_track_item(layer).expect("duplicate");
    writer.validate().expect("post-duplicate must validate");

    let snap = writer.snapshot();
    let TrackItem::Clip(orig) = &snap.tracks[0].items[0] else {
        panic!("orig clip");
    };
    let TrackItem::Clip(dup) = &snap.tracks[0].items[1] else {
        panic!("dup clip");
    };
    let ClipSource::Asset {
        audio: orig_audio, ..
    } = &orig.source
    else {
        panic!("orig asset");
    };
    let ClipSource::Asset {
        audio: dup_audio, ..
    } = &dup.source
    else {
        panic!("dup asset");
    };
    let DocParam::Keyframes(orig_keys) = &orig_audio[0].gain else {
        panic!("orig keys");
    };
    let DocParam::Keyframes(dup_keys) = &dup_audio[0].gain else {
        panic!("dup keys");
    };
    let orig_ids: Vec<_> = orig_keys.keys().iter().map(|k| k.id).collect();
    let dup_ids: Vec<_> = dup_keys.keys().iter().map(|k| k.id).collect();
    assert_eq!(orig_ids.len(), 2);
    assert_eq!(dup_ids.len(), 2);
    for id in &dup_ids {
        assert!(
            !orig_ids.contains(id),
            "duplicated gain keyframes must get fresh KeyframeIds"
        );
    }
}

#[test]
fn animated_audio_gain_respects_key_count_limit() {
    use motolii_doc::{
        load_document_bytes_with_limits, DocKeyframe, DocKeyframeTrack, DocParam, DocValue,
        KeyframeId, PersistError, ResourceLimitError, ResourceLimits,
    };
    use motolii_eval::Interp;

    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    let mut keys = DocKeyframeTrack::new();
    for (i, t) in [0i64, 1, 2].into_iter().enumerate() {
        keys.insert(DocKeyframe {
            id: KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
            t: RationalTime::try_new(t, 1).unwrap(),
            value: DocValue::F64(1.0 - i as f64 * 0.1),
            interp: Interp::Linear,
        });
    }
    let mut audio = AudioComponent::ordinal(0);
    audio.gain = DocParam::Keyframes(keys);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: None,
            audio: vec![audio],
        },
    );
    doc.validate().expect("validate ignores resource limits");

    let limits = ResourceLimits {
        max_keys_per_track: 2,
        ..ResourceLimits::production()
    };
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_limits(&bytes, &limits).unwrap_err();
    assert!(
        matches!(
            err,
            PersistError::ResourceLimit(ResourceLimitError::KeyCount { observed: 3, .. })
        ),
        "got {err:?}"
    );
}

#[test]
fn duplicate_audio_gain_keyframe_ids_are_rejected_without_remap() {
    use motolii_doc::{DocKeyframe, DocKeyframeTrack, DocParam, DocValue, KeyframeId};
    use motolii_eval::Interp;

    // walker無しだと複製後に同一KeyframeIdが2箇所に残りDuplicateStableIdになることを
    // 回帰として固定する(remap実装が外れたら赤)。
    let mut doc = Document::new_current();
    let asset = register_asset(&mut doc);
    let shared = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut keys = DocKeyframeTrack::new();
    keys.insert(DocKeyframe {
        id: shared,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    let mut audio = AudioComponent::ordinal(0);
    audio.gain = DocParam::Keyframes(keys);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: None,
            audio: vec![audio.clone()],
        },
    );
    // 意図的に同じgainキーを持つ2本目を手置き(remapを通さない)。
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: None,
            audio: vec![audio],
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::DuplicateStableId { id }) if id == shared.get()
    ));
}
