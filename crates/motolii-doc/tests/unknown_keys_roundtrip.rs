//! M2E-12: unknown-keys roundtrip(実装ガード7の骨格先取り)。

use motolii_doc::{Document, MIN_READER_VERSION_FOR_COMP_CAMERA, WRITER_VERSION};
use serde_json::{json, Value};

#[test]
fn unknown_keys_survive_json_roundtrip() {
    let input = json!({
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
        "future_track": {"id": "t1", "kind": "video"},
        "experimental_flag": true
    });

    let doc: Document = serde_json::from_value(input.clone()).expect("deserialize with extras");
    assert_eq!(doc.version, WRITER_VERSION);
    assert_eq!(doc.min_reader_version, MIN_READER_VERSION_FOR_COMP_CAMERA);
    assert_eq!(
        doc.extra.get("future_track"),
        Some(&json!({"id": "t1", "kind": "video"}))
    );
    assert_eq!(doc.extra.get("experimental_flag"), Some(&json!(true)));

    let output: Value = serde_json::to_value(&doc).expect("serialize");
    assert_eq!(output["future_track"], json!({"id": "t1", "kind": "video"}));
    assert_eq!(output["experimental_flag"], json!(true));
    assert_eq!(output["version"], WRITER_VERSION);
    assert_eq!(
        output["min_reader_version"],
        MIN_READER_VERSION_FOR_COMP_CAMERA
    );
    assert!(doc.extra.get("version").is_none());
    assert!(doc.extra.get("composition").is_none());
    assert!(doc.extra.get("bpm").is_none());
}

#[test]
fn unknown_keys_absent_yields_empty_extra() {
    let input = json!({
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
    let doc: Document = serde_json::from_value(input).unwrap();
    assert!(doc.extra.is_empty());
    assert_eq!(doc.min_reader_version, MIN_READER_VERSION_FOR_COMP_CAMERA);
}
