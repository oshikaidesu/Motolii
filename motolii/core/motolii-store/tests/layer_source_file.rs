//! `LayerSource::Media`/`PointCloud` を `File` へ畳んだ後も旧保存形式が読めること。

use motolii_store::LayerSource;

#[test]
fn old_media_and_point_cloud_json_deserialize_to_file() {
    let media_json = r#"{"Media":{"path":"a.mp4","fingerprint":"abc"}}"#;
    let pc_json = r#"{"PointCloud":{"path":"b.ply","fingerprint":null}}"#;

    let media: LayerSource = serde_json::from_str(media_json).unwrap();
    let pc: LayerSource = serde_json::from_str(pc_json).unwrap();

    assert_eq!(
        media,
        LayerSource::File {
            path: "a.mp4".to_string(),
            fingerprint: Some("abc".to_string()),
        }
    );
    assert_eq!(
        pc,
        LayerSource::File {
            path: "b.ply".to_string(),
            fingerprint: None,
        }
    );
}

#[test]
fn file_round_trips() {
    let source = LayerSource::File {
        path: "c.png".to_string(),
        fingerprint: Some("xyz".to_string()),
    };
    let json = serde_json::to_string(&source).unwrap();
    let back: LayerSource = serde_json::from_str(&json).unwrap();
    assert_eq!(source, back);
}
