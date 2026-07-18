#![allow(deprecated)]

//! D1j: CompCameraDoc v5 schema, validation, D1e default migration.

use motolii_core::RationalTime;
use motolii_doc::{
    classify_open_mode, count_document, load_document_bytes, load_document_bytes_with_limits,
    migrate_bytes, migrate_bytes_with_limits, Clip, ClipSource, CompCameraDoc, DocKeyframe,
    DocKeyframeTrack, DocParam, DocValue, Document, DocumentError, ItemEnvelope, KeyframeId,
    LookAtAxis, MigrateError, OpenMode, PersistError, ResourceLimitError, ResourceLimits, Track,
    TrackItem, MIN_READER_VERSION_FOR_COMP_CAMERA, MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS,
    READER_VERSION, WRITER_VERSION,
};
use motolii_eval::{DataTrackId, Interp};
use serde_json::{json, Value};

fn default_camera() -> CompCameraDoc {
    CompCameraDoc::default_planar_orthographic()
}

fn minimal_v5_json(extra: Value) -> Value {
    let mut shell = json!({
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
        "bpm": {"num": 120, "den": 1}
    });
    if let Some(obj) = extra.as_object() {
        for (k, v) in obj {
            shell[k] = v.clone();
        }
    }
    shell
}

#[test]
fn new_current_has_v5_contract_and_default_planar_camera() {
    let doc = Document::new_current();
    assert_eq!(doc.version, WRITER_VERSION);
    assert_eq!(doc.version, 5);
    assert_eq!(doc.min_reader_version, MIN_READER_VERSION_FOR_COMP_CAMERA);
    assert_eq!(doc.composition.camera, default_camera());

    let bytes = serde_json::to_vec(&doc).unwrap();
    let opened = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap();
    assert_eq!(opened.open_mode, OpenMode::ReadWrite);
    assert_eq!(opened.document, doc);
}

#[test]
fn wire_roundtrip_preserves_camera_params_and_extra() {
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.25, -0.5]),
        roll_radians: DocParam::const_f64(0.1),
        height: DocParam::const_f64(2.0),
    };
    doc.extra.insert("vendor_note".into(), json!("keep-me"));
    doc.validate().unwrap();

    let bytes = serde_json::to_vec(&doc).unwrap();
    let back = load_document_bytes(&bytes).unwrap();
    assert_eq!(back, doc);
    assert_eq!(back.extra.get("vendor_note").unwrap(), &json!("keep-me"));
}

#[test]
fn migrate_v1_through_v4_inserts_default_camera_and_bumps_version() {
    for version in 1..=4 {
        let bytes = serde_json::to_vec(&json!({
            "version": version,
            "min_reader_version": 1,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1}
            },
            "bpm": {"num": 120, "den": 1},
            "legacy_tag": format!("v{version}")
        }))
        .unwrap();

        let (doc, report) = migrate_bytes(&bytes).unwrap();
        assert!(
            report.steps.contains(&"insert_default_comp_camera"),
            "v{version} steps={:?}",
            report.steps
        );
        assert_eq!(doc.version, 5);
        assert_eq!(doc.min_reader_version, 5);
        assert_eq!(doc.composition.camera, default_camera());
        assert_eq!(count_document(&doc).track_count, 0);
        assert_eq!(count_document(&doc).clip_count, 0);
        assert_eq!(count_document(&doc).keyframe_count, 0);
        assert_eq!(
            doc.extra.get("legacy_tag").unwrap(),
            &json!(format!("v{version}"))
        );

        let (again, report2) = migrate_bytes(&serde_json::to_vec(&doc).unwrap()).unwrap();
        assert!(
            !report2.did_migrate(),
            "v{version} re-migrate must be idempotent"
        );
        assert_eq!(again, doc);
    }
}

#[test]
fn rejects_unknown_camera_kind() {
    let mut shell = minimal_v5_json(json!({}));
    shell
        .get_mut("composition")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert(
            "camera".into(),
            json!({
                "kind": "perspective",
                "fov": 45.0
            }),
        );
    let err = load_document_bytes(&serde_json::to_vec(&shell).unwrap()).unwrap_err();
    assert!(matches!(err, PersistError::Json(_)), "{err:?}");
}

#[test]
fn rejects_non_finite_camera_params() {
    for (camera, path_suffix) in [
        (
            CompCameraDoc::PlanarOrthographic {
                center: DocParam::const_vec2([f64::NAN, 0.0]),
                roll_radians: DocParam::const_f64(0.0),
                height: DocParam::const_f64(1.0),
            },
            "center",
        ),
        (
            CompCameraDoc::PlanarOrthographic {
                center: DocParam::const_vec2([0.0, 0.0]),
                roll_radians: DocParam::const_f64(f64::INFINITY),
                height: DocParam::const_f64(1.0),
            },
            "roll_radians",
        ),
        (
            CompCameraDoc::PlanarOrthographic {
                center: DocParam::const_vec2([0.0, 0.0]),
                roll_radians: DocParam::const_f64(0.0),
                height: DocParam::const_f64(f64::NEG_INFINITY),
            },
            "height",
        ),
    ] {
        let mut doc = Document::new_current();
        doc.composition.camera = camera;
        let err = doc.validate().unwrap_err();
        assert!(
            matches!(err, DocumentError::NonFiniteValue { .. }),
            "{path_suffix}: {err:?}"
        );
    }
}

#[test]
fn rejects_non_positive_height() {
    for height in [0.0, -1.0] {
        let mut shell = minimal_v5_json(json!({}));
        shell
            .get_mut("composition")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .get_mut("camera")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("height".into(), json!({"const": {"F64": height}}));
        let err = load_document_bytes(&serde_json::to_vec(&shell).unwrap()).unwrap_err();
        assert!(
            matches!(
                err,
                PersistError::Validate(DocumentError::ValueOutOfRange { .. })
            ),
            "height={height}: {err:?}"
        );
    }
}

#[test]
fn rejects_min_reader_above_reader_version() {
    let shell = minimal_v5_json(json!({
        "min_reader_version": READER_VERSION + 1
    }));
    let err = load_document_bytes(&serde_json::to_vec(&shell).unwrap()).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version,
            reader_version,
        } if min_reader_version == READER_VERSION + 1 && reader_version == READER_VERSION
    ));
}

#[test]
fn disguised_v1_through_v4_camera_payload_rejected_on_migrate_and_load() {
    let camera = json!({
        "kind": "planar_orthographic",
        "center": {"const": {"Vec2": [1.0, 2.0]}},
        "roll_radians": {"const": {"F64": 0.5}},
        "height": {"const": {"F64": 2.0}}
    });
    for version in 1..=4 {
        let bytes = serde_json::to_vec(&json!({
            "version": version,
            "min_reader_version": 1,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1},
                "camera": camera
            },
            "bpm": {"num": 120, "den": 1}
        }))
        .unwrap();

        let migrate_err = migrate_bytes(&bytes).unwrap_err();
        assert!(
            matches!(migrate_err, MigrateError::DisguisedCompCamera { version: v } if v == version),
            "v{version} migrate: {migrate_err:?}"
        );

        let load_err = load_document_bytes(&bytes).unwrap_err();
        assert!(
            matches!(load_err, PersistError::DisguisedCompCamera { .. }),
            "v{version} load: {load_err:?}"
        );
    }
}

#[test]
fn migrate_failure_disguised_camera_does_not_mutate_input_bytes() {
    let bytes = serde_json::to_vec(&json!({
        "version": 2,
        "min_reader_version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 10, "den": 1},
            "fps": {"num": 30, "den": 1},
            "camera": {
                "kind": "planar_orthographic",
                "center": {"const": {"Vec2": [0.0, 0.0]}},
                "roll_radians": {"const": {"F64": 0.0}},
                "height": {"const": {"F64": 0.0}}
            }
        },
        "bpm": {"num": 120, "den": 1}
    }))
    .unwrap();
    let before = bytes.clone();
    let err = migrate_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap_err();
    assert_eq!(bytes, before, "migrate failure must not mutate input bytes");
    assert!(
        matches!(err, MigrateError::DisguisedCompCamera { version: 2 }),
        "{err:?}"
    );
}

#[test]
fn migrate_failure_validate_error_does_not_mutate_input_bytes() {
    let bytes = serde_json::to_vec(&json!({
        "version": 2,
        "min_reader_version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 0, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1}
    }))
    .unwrap();
    let before = bytes.clone();
    let err = migrate_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap_err();
    assert_eq!(bytes, before, "migrate failure must not mutate input bytes");
    assert!(
        matches!(
            err,
            MigrateError::Validate(DocumentError::NonPositiveCompositionDuration { .. })
        ),
        "{err:?}"
    );
}

#[test]
fn typed_old_version_document_rejected_without_validate_bypass() {
    let mut doc = Document::new_current();
    doc.version = 2;
    doc.min_reader_version = 2;
    assert!(matches!(
        doc.validate(),
        Err(DocumentError::CompCameraDisguisedOldVersion {
            version: 2,
            required: MIN_READER_VERSION_FOR_COMP_CAMERA
        })
    ));
}

#[test]
fn camera_keyframes_counted_and_stable_ids_integrate_with_next_counter() {
    let mut doc = Document::new_current();
    let k0 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let k1 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut height_track = DocKeyframeTrack::new();
    height_track.insert(DocKeyframe {
        id: k0,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    height_track.insert(DocKeyframe {
        id: k1,
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(2.0),
        interp: Interp::Linear,
    });

    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Keyframes(height_track),
    };
    doc.validate().unwrap();
    assert_eq!(count_document(&doc).keyframe_count, 2);
    assert_eq!(doc.next_stable_id.peek_next(), 2);
}

#[test]
fn camera_keyframe_id_duplicate_with_track_param_is_rejected() {
    let mut doc = Document::new_current();
    let shared = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let k1 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut height_track = DocKeyframeTrack::new();
    height_track.insert(DocKeyframe {
        id: shared,
        t: RationalTime::ZERO,
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    height_track.insert(DocKeyframe {
        id: k1,
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(2.0),
        interp: Interp::Linear,
    });
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Keyframes(height_track),
    };

    let layer = doc.layers.allocate("layer").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    let mut rotation_track = DocKeyframeTrack::new();
    rotation_track.insert(DocKeyframe {
        id: shared,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    rotation_track.insert(DocKeyframe {
        id: KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    let mut env = ItemEnvelope::new(layer);
    env.transform.rotation = DocParam::Keyframes(rotation_track);
    doc.tracks.push(Track {
        id: track_id,
        items: vec![TrackItem::Clip(Clip {
            envelope: env,
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });

    assert!(matches!(
        doc.validate(),
        Err(DocumentError::DuplicateStableId { id }) if id == shared.get()
    ));
}

#[test]
fn camera_accepts_keyframes_data_and_vec2_axes_variants() {
    let mut doc = Document::new_current();
    let k0 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let k1 = KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap());
    let mut keys = DocKeyframeTrack::new();
    keys.insert(DocKeyframe {
        id: k0,
        t: RationalTime::ZERO,
        value: DocValue::F64(0.5),
        interp: Interp::Linear,
    });
    keys.insert(DocKeyframe {
        id: k1,
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(1.5),
        interp: Interp::Hold,
    });

    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::Vec2Axes {
            x: Box::new(DocParam::const_f64(0.1)),
            y: Box::new(DocParam::Data {
                track: DataTrackId("cam.y".into()),
                fallback: DocValue::F64(-0.2),
            }),
        },
        roll_radians: DocParam::Keyframes(keys),
        height: DocParam::Data {
            track: DataTrackId("cam.h".into()),
            fallback: DocValue::F64(1.25),
        },
    };
    doc.validate().unwrap();
    assert_eq!(count_document(&doc).keyframe_count, 2);
}

#[test]
fn camera_vec2_axes_bad_axis_type_is_rejected() {
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::Vec2Axes {
            x: Box::new(DocParam::const_color([1.0, 0.0, 0.0, 1.0])),
            y: Box::new(DocParam::const_f64(0.0)),
        },
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::const_f64(1.0),
    };
    assert!(
        matches!(doc.validate(), Err(DocumentError::ParamTypeMismatch { .. })),
        "{:?}",
        doc.validate()
    );
}

#[test]
fn camera_look_at_and_follow_are_rejected() {
    let mut doc = Document::new_current();
    let target = doc.layers.allocate("target").unwrap();
    for (field, param) in [
        (
            "roll_radians",
            DocParam::LookAt {
                target,
                axis: LookAtAxis::PlusY,
            },
        ),
        (
            "height",
            DocParam::Follow {
                target,
                offset: [0.0, 0.0],
            },
        ),
    ] {
        let mut trial = Document::new_current();
        trial.composition.camera = CompCameraDoc::PlanarOrthographic {
            center: DocParam::const_vec2([0.0, 0.0]),
            roll_radians: DocParam::const_f64(0.0),
            height: DocParam::const_f64(1.0),
        };
        match field {
            "roll_radians" => {
                let CompCameraDoc::PlanarOrthographic { roll_radians, .. } =
                    &mut trial.composition.camera;
                *roll_radians = param;
            }
            "height" => {
                let CompCameraDoc::PlanarOrthographic { height, .. } =
                    &mut trial.composition.camera;
                *height = param;
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(
                trial.validate(),
                Err(DocumentError::SpatialLinkNotAllowed { .. })
            ),
            "{field}"
        );
    }
}

#[test]
fn rejects_non_positive_height_via_keyframes() {
    let mut keys = DocKeyframeTrack::new();
    keys.insert(DocKeyframe {
        id: KeyframeId::from_raw(1),
        t: RationalTime::ZERO,
        value: DocValue::F64(0.0),
        interp: Interp::Linear,
    });
    keys.insert(DocKeyframe {
        id: KeyframeId::from_raw(2),
        t: RationalTime::try_new(1, 1).unwrap(),
        value: DocValue::F64(1.0),
        interp: Interp::Linear,
    });
    let mut shell = minimal_v5_json(json!({}));
    shell["composition"]["camera"]["height"] =
        serde_json::to_value(DocParam::Keyframes(keys)).unwrap();
    let err = load_document_bytes(&serde_json::to_vec(&shell).unwrap()).unwrap_err();
    assert!(
        matches!(
            err,
            PersistError::Validate(DocumentError::ValueOutOfRange { .. })
        ),
        "{err:?}"
    );
}

#[test]
fn migrate_v1_without_camera_preserves_existing_keyframe_counts() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/timeline_start/speed_clip.json"
    );
    let bytes = std::fs::read(path).unwrap();
    let (doc, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"insert_default_comp_camera"));
    assert_eq!(count_document(&doc).keyframe_count, 2);
    assert_eq!(count_document(&doc).clip_count, 1);
}

#[test]
fn v5_document_requires_comp_camera_min_reader_floor() {
    let mut shell = minimal_v5_json(json!({
        "min_reader_version": MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS
    }));
    shell["min_reader_version"] = json!(4);
    let err = load_document_bytes(&serde_json::to_vec(&shell).unwrap()).unwrap_err();
    assert!(
        matches!(
            err,
            PersistError::Validate(DocumentError::CompCameraRequiresNewerReader { .. })
        ),
        "{err:?}"
    );
}

#[test]
fn camera_height_keyframes_within_key_count_limit_loads() {
    let mut doc = Document::new_current();
    let mut keys = DocKeyframeTrack::new();
    for (i, t) in [0i64, 1].into_iter().enumerate() {
        keys.insert(DocKeyframe {
            id: KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
            t: RationalTime::try_new(t, 1).unwrap(),
            value: DocValue::F64(1.0 + i as f64),
            interp: Interp::Linear,
        });
    }
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Keyframes(keys),
    };
    doc.validate().unwrap();

    let limits = ResourceLimits {
        max_keys_per_track: 2,
        ..ResourceLimits::production()
    };
    let bytes = serde_json::to_vec(&doc).unwrap();
    load_document_bytes_with_limits(&bytes, &limits).unwrap();
}

#[test]
fn camera_data_track_within_string_bytes_limit_loads() {
    let limit = 8;
    let track_name = "a".repeat(limit as usize);
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Data {
            track: DataTrackId(track_name),
            fallback: DocValue::F64(1.0),
        },
    };
    doc.validate().unwrap();

    let limits = ResourceLimits {
        max_string_bytes: limit,
        ..ResourceLimits::production()
    };
    let bytes = serde_json::to_vec(&doc).unwrap();
    load_document_bytes_with_limits(&bytes, &limits).unwrap();
}

#[test]
fn camera_height_keyframes_exceeding_key_count_limit_rejected_on_load() {
    let mut doc = Document::new_current();
    let mut keys = DocKeyframeTrack::new();
    for (i, t) in [0i64, 1, 2].into_iter().enumerate() {
        keys.insert(DocKeyframe {
            id: KeyframeId::from_raw(doc.next_stable_id.allocate().unwrap()),
            t: RationalTime::try_new(t, 1).unwrap(),
            value: DocValue::F64(1.0 + i as f64),
            interp: Interp::Linear,
        });
    }
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Keyframes(keys),
    };
    doc.validate().unwrap();

    let limits = ResourceLimits {
        max_keys_per_track: 2,
        ..ResourceLimits::production()
    };
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_limits(&bytes, &limits).unwrap_err();
    assert!(
        matches!(
            err,
            PersistError::ResourceLimit(ResourceLimitError::KeyCount {
                ref path,
                observed: 3,
                limit: 2,
            }) if path == "composition.camera.height"
        ),
        "got {err:?}"
    );
}

#[test]
fn camera_data_track_exceeding_string_bytes_limit_rejected_on_load() {
    let limit = 8;
    let track_name = "a".repeat(limit as usize + 1);
    let mut doc = Document::new_current();
    doc.composition.camera = CompCameraDoc::PlanarOrthographic {
        center: DocParam::const_vec2([0.0, 0.0]),
        roll_radians: DocParam::const_f64(0.0),
        height: DocParam::Data {
            track: DataTrackId(track_name),
            fallback: DocValue::F64(1.0),
        },
    };
    doc.validate().unwrap();

    let limits = ResourceLimits {
        max_string_bytes: limit,
        ..ResourceLimits::production()
    };
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_limits(&bytes, &limits).unwrap_err();
    assert!(
        matches!(
            err,
            PersistError::ResourceLimit(ResourceLimitError::StringBytes {
                ref path,
                observed: 9,
                limit: 8,
            }) if path == "composition.camera.height.track"
        ),
        "got {err:?}"
    );
}

#[test]
fn open_mode_read_write_requires_current_writer_contract() {
    let doc = Document::new_current();
    assert_eq!(
        classify_open_mode(doc.version, doc.min_reader_version),
        OpenMode::ReadWrite
    );
}
