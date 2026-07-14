//! AG-1: Asset Clip video/audio component schema / 互換 / 拒否。

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    asset_components_require_newer_reader, AudioComponent, AudioOutOfRange, Clip, ClipSource,
    Document, DocumentError, ItemEnvelope, StreamKind, StreamSelector, Track, TrackItem,
    VideoComponent, MIN_READER_VERSION_FOR_ASSET_COMPONENTS, READER_VERSION,
};
use serde_json::json;

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
    let mut doc = Document::new_v1();
    doc.version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
    doc.min_reader_version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
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
    let mut doc = Document::new_v1();
    doc.version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
    doc.min_reader_version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
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
    let mut doc = Document::new_v1();
    doc.version = 2;
    doc.min_reader_version = 2;
    let asset = register_asset(&mut doc);
    push_asset_clip(
        &mut doc,
        ClipSource::Asset {
            asset,
            video: Some(VideoComponent::ordinal(0)),
            audio: vec![AudioComponent::ordinal(0)],
        },
    );
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::AssetComponentsRequireNewerReader {
            min_reader_version: 2,
            required: MIN_READER_VERSION_FOR_ASSET_COMPONENTS,
        })
    ));
}

#[test]
fn validate_accepts_audio_components_with_reader_gate() {
    let mut doc = Document::new_v1();
    doc.version = READER_VERSION;
    doc.min_reader_version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
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
fn old_project_with_legacy_asset_still_validates_at_v2() {
    let mut doc = Document::new_v1();
    doc.version = 2;
    doc.min_reader_version = 2;
    let asset = register_asset(&mut doc);
    push_asset_clip(&mut doc, ClipSource::asset_video_only(asset));
    doc.validate().unwrap();
}

#[test]
fn negative_gain_is_rejected() {
    let mut doc = Document::new_v1();
    doc.version = READER_VERSION;
    doc.min_reader_version = MIN_READER_VERSION_FOR_ASSET_COMPONENTS;
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
