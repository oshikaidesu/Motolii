//! M2E-12: unknown-keys roundtrip(実装ガード7の骨格先取り)。
//!
//! 未知フィールドはロード失敗せず保持され、再保存で書き戻されること。

use motolii_doc::Document;
use serde_json::{json, Value};

#[test]
fn unknown_keys_survive_json_roundtrip() {
    let input = json!({
        "version": 1,
        "min_reader_version": 1,
        "time_map": {
            "source_start": {"num": 0, "den": 1},
            "timeline_start": {"num": 0, "den": 1},
            "speed_num": 1,
            "speed_den": 1
        },
        "future_track": {"id": "t1", "kind": "video"},
        "experimental_flag": true
    });

    let doc: Document = serde_json::from_value(input.clone()).expect("deserialize with extras");
    assert_eq!(doc.version, 1);
    assert_eq!(doc.min_reader_version, 1);
    assert_eq!(doc.extra.get("future_track"), Some(&json!({"id": "t1", "kind": "video"})));
    assert_eq!(doc.extra.get("experimental_flag"), Some(&json!(true)));

    let output: Value = serde_json::to_value(&doc).expect("serialize");
    assert_eq!(output["future_track"], json!({"id": "t1", "kind": "video"}));
    assert_eq!(output["experimental_flag"], json!(true));
    assert_eq!(output["version"], 1);
    assert_eq!(output["min_reader_version"], 1);
    // 既知フィールドは extra に二重化しない
    assert!(doc.extra.get("version").is_none());
    assert!(doc.extra.get("time_map").is_none());
}

#[test]
fn unknown_keys_absent_yields_empty_extra() {
    let input = json!({
        "version": 1,
        "time_map": {
            "source_start": {"num": 0, "den": 1},
            "timeline_start": {"num": 0, "den": 1},
            "speed_num": 1,
            "speed_den": 1
        }
    });
    let doc: Document = serde_json::from_value(input).unwrap();
    assert!(doc.extra.is_empty());
    assert_eq!(doc.min_reader_version, 1);
}
