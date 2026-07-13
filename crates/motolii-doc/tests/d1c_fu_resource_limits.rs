//! D1c-FU(#101, 監査S10): `ResourceLimits`をロード入口へ注入した境界/超過テスト。
//!
//! 「巨大入力拒否」は本物の巨大ファイルを作らず、小さい上限を注入して同じ拒否経路を
//! 踏む(fuzz corpus相当: 深いGroup入れ子・巨大extra・巨大stringの敵対的入力を模す)。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    load_document_bytes_with_limits, load_document_with_limits, AssetId, Clip, ClipSource,
    ClippingMaskSettings, Document, Group, ItemEnvelope, LayerId, PersistError, ResourceLimitError,
    ResourceLimits, Track, TrackId, TrackItem, Transform2D,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1c-fu-limits-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn base_doc() -> Document {
    Document::new_v1()
}

fn simple_clip(layer_id: LayerId, asset: AssetId) -> TrackItem {
    TrackItem::Clip(Clip {
        envelope: ItemEnvelope {
            layer_id,
            effects: Vec::new(),
            transform: Transform2D::identity(),
            clipping_mask: ClippingMaskSettings::default(),
            blend: Default::default(),
            opacity: motolii_doc::DocParam::const_f64(1.0),
        },
        start: motolii_core::RationalTime::ZERO,
        duration: motolii_core::RationalTime::try_new(1, 1).unwrap(),
        time_map: Default::default(),
        source: ClipSource::Asset { asset },
    })
}

/// depth重ねのGroupを持つ単一TrackのDocumentを構築する。
fn doc_with_group_depth(depth: u32) -> Document {
    let mut doc = base_doc();
    let asset = doc.assets.allocate("a", "video/mp4", "h").unwrap();
    let track_id: TrackId = doc.track_ids.allocate("t").unwrap();
    let leaf_layer = doc.layers.allocate("leaf").unwrap();
    let mut item = simple_clip(leaf_layer, asset);
    for i in 0..depth {
        let layer = doc.layers.allocate(format!("g{i}")).unwrap();
        item = TrackItem::Group(Group {
            envelope: ItemEnvelope::new(layer),
            children: vec![item],
        });
    }
    doc.tracks.push(Track {
        id: track_id,
        items: vec![item],
    });
    doc
}

fn to_bytes(doc: &Document) -> Vec<u8> {
    serde_json::to_vec(doc).unwrap()
}

// --- file bytes ---

#[test]
fn file_bytes_over_limit_is_rejected_before_full_parse() {
    let dir = unique_dir("file-bytes");
    let path = dir.join("doc.json");
    let doc = base_doc();
    fs::write(&path, to_bytes(&doc)).unwrap();
    let actual_len = fs::metadata(&path).unwrap().len();

    let tiny = ResourceLimits {
        max_file_bytes: actual_len - 1,
        ..ResourceLimits::production()
    };
    let err = load_document_with_limits(&path, &tiny).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ResourceLimit(ResourceLimitError::FileBytes { limit, .. }) if limit == actual_len - 1
    ));

    let exact = ResourceLimits {
        max_file_bytes: actual_len,
        ..ResourceLimits::production()
    };
    assert!(load_document_with_limits(&path, &exact).is_ok());
    let _ = fs::remove_dir_all(dir);
}

// --- group depth (fuzz corpus相当: 敵対的な深い入れ子) ---

#[test]
fn deeply_nested_groups_are_rejected_with_observed_and_limit() {
    let doc_ok = doc_with_group_depth(5);
    let limits = ResourceLimits {
        max_group_depth: 5,
        ..ResourceLimits::production()
    };
    assert!(load_document_bytes_with_limits(&to_bytes(&doc_ok), &limits).is_ok());

    // 敵対的入力: 上限を大きく超えるGroup入れ子(serde_jsonの再帰上限未満に留め、
    // 我々のGroupDepth拒否が先に効くことを確認する)
    let doc_adversarial = doc_with_group_depth(40);
    let err = load_document_bytes_with_limits(&to_bytes(&doc_adversarial), &limits).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ResourceLimit(ResourceLimitError::GroupDepth { observed, limit, .. })
            if observed == 6 && limit == 5
    ));
}

// --- track / layer count ---

#[test]
fn track_count_over_limit_is_rejected() {
    let mut doc = base_doc();
    for i in 0..3 {
        let id = doc.track_ids.allocate(format!("t{i}")).unwrap();
        doc.tracks.push(Track {
            id,
            items: Vec::new(),
        });
    }
    let limits = ResourceLimits {
        max_tracks: 2,
        ..ResourceLimits::production()
    };
    let err = load_document_bytes_with_limits(&to_bytes(&doc), &limits).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ResourceLimit(ResourceLimitError::TrackCount {
            observed: 3,
            limit: 2
        })
    ));
}

// --- string bytes: 巨大な文字列(fuzz corpus相当) ---

#[test]
fn huge_string_field_is_rejected() {
    let mut doc = base_doc();
    let huge_name = "x".repeat(10_000);
    doc.assets.allocate(huge_name, "video/mp4", "h").unwrap();

    let limits = ResourceLimits {
        max_string_bytes: 1_000,
        ..ResourceLimits::production()
    };
    let err = load_document_bytes_with_limits(&to_bytes(&doc), &limits).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ResourceLimit(ResourceLimitError::StringBytes {
            observed: 10_000,
            limit: 1_000,
            ..
        })
    ));
}

// --- extra bytes: 巨大なextra flatten(fuzz corpus相当) ---

#[test]
fn huge_extra_payload_is_rejected() {
    let mut doc = base_doc();
    for i in 0..5_000 {
        doc.extra
            .insert(format!("k{i}"), serde_json::Value::String("v".repeat(20)));
    }
    let limits = ResourceLimits {
        max_extra_bytes: 1_000,
        ..ResourceLimits::production()
    };
    let err = load_document_bytes_with_limits(&to_bytes(&doc), &limits).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ResourceLimit(ResourceLimitError::ExtraBytes { limit: 1_000, .. })
    ));
}

// --- production既定は通常プロジェクトを拒否しない ---

#[test]
fn production_limits_accept_typical_small_document() {
    let doc = doc_with_group_depth(3);
    let bytes = to_bytes(&doc);
    let opened = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap();
    assert_eq!(opened.document.tracks.len(), 1);
}
