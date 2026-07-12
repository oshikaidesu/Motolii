//! D1a: スキーマ本体のJSON roundtripと境界宣言の機械判定。

use std::collections::BTreeMap;

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    Asset, AssetId, BlendMode, Bpm, Clip, ClipSource, ClippingMaskSettings, DocParam, Document,
    EffectInstance, Group, ItemEnvelope, LookAtAxis, MaskMode, PathOp, Soundtrack, Track,
    TrackItem,
};
use motolii_eval::{DataTrackId, Interp, Keyframe, KeyframeTrack, Value as EvalValue};
use serde_json::{json, Map, Value};

fn sample_document() -> Document {
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(240, 2).unwrap(); // 既約化されて 120/1

    let asset_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: asset_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "sha256:deadbeef".into(),
            path_absolute: Some(r"C:\media\bg.mp4".into()),
            path_project_relative: Some(r"media\bg.mp4".into()),
            file_name: Some("bg.mp4".into()),
            size_bytes: Some(2048),
            head_hash: Some("head".into()),
            tail_hash: Some("tail".into()),
        })
        .unwrap();

    let clip_layer = doc.layers.allocate("矩形").unwrap();
    let group_layer = doc.layers.allocate("グループ").unwrap();
    let child_layer = doc.layers.allocate("子").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();

    let mut effect_extra = Map::new();
    effect_extra.insert("vendor_meta".into(), json!({"x": 1}));

    let child_clip = Clip {
        envelope: {
            let mut env = ItemEnvelope::new(child_layer);
            env.transform.parent = Some(group_layer);
            env.effects.push(EffectInstance {
                plugin_id: "core.filter.tint".into(),
                effect_version: 1,
                enabled: true,
                params: BTreeMap::from([(
                    "color".into(),
                    DocParam::const_color([1.0, 0.0, 0.0, 1.0]),
                )]),
                extra: effect_extra,
            });
            env.clipping_mask = ClippingMaskSettings {
                enabled: true,
                mode: MaskMode::Luminance,
            };
            env.blend = BlendMode::Multiply;
            env
        },
        start: RationalTime::try_new(0, 1).unwrap(),
        duration: RationalTime::try_new(5, 1).unwrap(),
        time_map: TimeMap::constant_speed(RationalTime::ZERO, RationalTime::ZERO, 1, 1).unwrap(),
        source: ClipSource::Plugin {
            plugin_id: "core.layer_source.clear".into(),
            effect_version: 1,
            params: BTreeMap::new(),
            extra: Map::new(),
        },
        path_ops: vec![PathOp::Trim {
            start: DocParam::const_f64(0.0),
            end: DocParam::const_f64(1.0),
            offset: DocParam::const_f64(0.0),
        }],
    };

    let group = Group {
        envelope: {
            let mut env = ItemEnvelope::new(group_layer);
            env.effects.push(EffectInstance {
                plugin_id: "core.filter.opacity".into(),
                effect_version: 1,
                enabled: true,
                params: BTreeMap::from([("opacity".into(), DocParam::const_f64(0.8))]),
                extra: Map::new(),
            });
            env.transform.position = DocParam::Follow {
                target: clip_layer,
                offset: [0.1, 0.0],
            };
            env
        },
        children: vec![TrackItem::Clip(child_clip)],
    };

    let top_clip = Clip {
        envelope: {
            let mut env = ItemEnvelope::new(clip_layer);
            env.transform.position = DocParam::LookAt {
                target: group_layer,
                axis: LookAtAxis::PlusY,
            };
            env
        },
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Asset { asset: asset_id },
        path_ops: Vec::new(),
    };

    doc.soundtrack = Some(
        Soundtrack::try_new(asset_id, RationalTime::ZERO, 1.0).unwrap(),
    );

    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(top_clip), TrackItem::Group(group)],
    });

    doc
}

#[test]
fn nested_group_effects_timemap_parent_roundtrip() {
    let doc = sample_document();
    assert_eq!(doc.bpm.num(), 120);
    assert_eq!(doc.bpm.den(), 1);

    let json = serde_json::to_value(&doc).expect("serialize");
    let back: Document = serde_json::from_value(json).expect("deserialize");
    assert_eq!(doc, back);

    assert_eq!(
        doc.bpm.try_beat_duration().unwrap(),
        RationalTime::try_new(1, 2).unwrap()
    );

    let a = doc.assets.get(AssetId::from_raw(0)).unwrap();
    assert_eq!(a.path_absolute.as_deref(), Some("C:/media/bg.mp4"));
    assert_eq!(a.path_project_relative.as_deref(), Some("media/bg.mp4"));
}

#[test]
fn composition_has_no_camera_field() {
    let doc = sample_document();
    let json: Value = serde_json::to_value(&doc).unwrap();
    assert!(json.get("camera").is_none());
    assert!(json["composition"].get("camera").is_none());
    assert_eq!(json["composition"]["aspect_num"], 16);
    assert_eq!(json["composition"]["aspect_den"], 9);
}

#[test]
fn nested_unknown_fields_are_dropped_by_design() {
    // 仕様「ネスト未知フィールドの方針」: Composition等はextraを持たず黙殺する。
    // だからネストへフィールドを足す変更は必ずmin_reader_versionを上げる。
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1},
            "camera": {"position": [0, 0, 1]}
        },
        "bpm": {"num": 120, "den": 1}
    });
    let doc: Document = serde_json::from_value(input).unwrap();
    let out = serde_json::to_value(&doc).unwrap();
    assert!(out["composition"].get("camera").is_none());
    assert!(doc.extra.get("camera").is_none());
}

#[test]
fn doc_param_keyframes_data_vec2axes_roundtrip() {
    let mut keys = KeyframeTrack::new();
    keys.insert(Keyframe {
        t: RationalTime::ZERO,
        value: EvalValue::F64(0.0),
        interp: Interp::Linear,
    });
    keys.insert(Keyframe {
        t: RationalTime::try_new(1, 1).unwrap(),
        value: EvalValue::F64(1.0),
        interp: Interp::Hold,
    });

    let params = [
        DocParam::Keyframes(keys),
        DocParam::Data {
            track: DataTrackId("amp".into()),
            fallback: EvalValue::F64(0.5),
        },
        DocParam::Vec2Axes {
            x: Box::new(DocParam::const_f64(0.1)),
            y: Box::new(DocParam::Data {
                track: DataTrackId("y".into()),
                fallback: EvalValue::F64(0.0),
            }),
        },
    ];

    for param in params {
        let json = serde_json::to_value(&param).unwrap();
        let back: DocParam = serde_json::from_value(json).unwrap();
        assert_eq!(param, back);
    }
}

#[test]
fn effect_unknown_fields_survive_roundtrip() {
    let doc = sample_document();
    let json = serde_json::to_value(&doc).unwrap();
    let back: Document = serde_json::from_value(json).unwrap();
    let TrackItem::Group(group) = &back.tracks[0].items[1] else {
        panic!("expected group");
    };
    let TrackItem::Clip(child) = &group.children[0] else {
        panic!("expected child clip");
    };
    assert_eq!(
        child.envelope.effects[0].extra.get("vendor_meta"),
        Some(&json!({"x": 1}))
    );
}

#[test]
fn asset_multi_keys_normalize_slashes_on_load() {
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "assets": {
            "next": 1,
            "entries": [{
                "id": 0,
                "name": "a",
                "asset_type": "video/mp4",
                "content_hash": "h",
                "path_absolute": "D:\\x\\a.mp4",
                "path_project_relative": "media\\a.mp4",
                "file_name": "a.mp4"
            }]
        }
    });
    let doc: Document = serde_json::from_value(input).unwrap();
    let a = doc.assets.get(AssetId::from_raw(0)).unwrap();
    assert_eq!(a.path_absolute.as_deref(), Some("D:/x/a.mp4"));
    assert_eq!(a.path_project_relative.as_deref(), Some("media/a.mp4"));
}

#[test]
fn soundtrack_rejects_out_of_range_gain() {
    let err = Soundtrack::try_new(AssetId::from_raw(0), RationalTime::ZERO, 1.5).unwrap_err();
    assert!(err.to_string().contains("master_gain"));
}

#[test]
fn plugin_source_unknown_fields_survive_roundtrip() {
    let input = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "layers": {
            "next": 1,
            "entries": [{"id": 0, "name": "src"}]
        },
        "track_ids": {
            "next": 1,
            "entries": [{"id": 0, "name": "V1"}]
        },
        "tracks": [{
            "id": 0,
            "items": [{
                "kind": "clip",
                "envelope": {
                    "layer_id": 0,
                    "transform": {
                        "position": {"const": {"Vec2": [0.0, 0.0]}},
                        "anchor": {"const": {"Vec2": [0.0, 0.0]}},
                        "scale": {"const": {"Vec2": [1.0, 1.0]}},
                        "rotation": {"const": {"F64": 0.0}}
                    }
                },
                "start": {"num": 0, "den": 1},
                "duration": {"num": 1, "den": 1},
                "source": {
                    "source": "plugin",
                    "plugin_id": "core.layer_source.clear",
                    "future_flag": true
                }
            }]
        }]
    });
    let doc: Document = serde_json::from_value(input).unwrap();
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    let ClipSource::Plugin { extra, .. } = &clip.source else {
        panic!("expected plugin source");
    };
    assert_eq!(extra.get("future_flag"), Some(&json!(true)));
    let back: Document = serde_json::from_value(serde_json::to_value(&doc).unwrap()).unwrap();
    let TrackItem::Clip(clip) = &back.tracks[0].items[0] else {
        panic!("expected clip");
    };
    let ClipSource::Plugin { extra, .. } = &clip.source else {
        panic!("expected plugin source");
    };
    assert_eq!(extra.get("future_flag"), Some(&json!(true)));
}
