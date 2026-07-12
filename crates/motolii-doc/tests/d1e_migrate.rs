//! D1e integration tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    bump_min_reader_for_nest_schema_change, count_document, load_document, load_document_bytes,
    load_document_bytes_with_reader_cap, migrate_bytes, migrate_document_file, save_document,
    Asset, AssetId, Clip, ClipSource, DocParam, Document, DocumentCounts, Group, ItemEnvelope,
    MigrateError, MigrateFileOptions, PersistError, Track, TrackItem, BACKUP_SUFFIX,
    LATEST_DOCUMENT_VERSION, READER_VERSION,
};
use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value as EvalValue};
use serde::Deserialize;

const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus");

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    path: String,
    track_count: usize,
    clip_count: usize,
    keyframe_count: usize,
}

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    entries: Vec<CorpusEntry>,
}

fn corpus_path(rel: &str) -> PathBuf {
    Path::new(CORPUS_DIR).join(rel)
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1e-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn rich_v1_document() -> Document {
    let mut doc = Document::new_v1();
    let asset_id = AssetId::from_raw(0);
    doc.assets
        .insert(Asset {
            id: asset_id,
            name: "bg".into(),
            asset_type: "video/mp4".into(),
            content_hash: "h".into(),
            path_absolute: None,
            path_project_relative: None,
            file_name: None,
            size_bytes: None,
            head_hash: None,
            tail_hash: None,
        })
        .unwrap();
    let clip_layer = doc.layers.allocate("a").unwrap();
    let group_layer = doc.layers.allocate("g").unwrap();
    let child_layer = doc.layers.allocate("c").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
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
    let child_clip = Clip {
        envelope: {
            let mut e = ItemEnvelope::new(child_layer);
            e.opacity = DocParam::Keyframes(keys);
            e
        },
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(5, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Asset { asset: asset_id },
        path_ops: Vec::new(),
    };
    let top_clip = Clip {
        envelope: ItemEnvelope::new(clip_layer),
        start: RationalTime::ZERO,
        duration: RationalTime::try_new(10, 1).unwrap(),
        time_map: TimeMap::identity(),
        source: ClipSource::Asset { asset: asset_id },
        path_ops: Vec::new(),
    };
    doc.tracks.push(Track {
        id: track_id,
        items: vec![
            TrackItem::Clip(top_clip),
            TrackItem::Group(Group {
                envelope: ItemEnvelope::new(group_layer),
                children: vec![TrackItem::Clip(child_clip)],
            }),
        ],
    });
    doc
}

#[test]
fn golden_corpus_migrates_with_stable_counts() {
    let manifest: CorpusManifest =
        serde_json::from_str(&fs::read_to_string(corpus_path("manifest.json")).unwrap()).unwrap();
    for entry in manifest.entries {
        let bytes = fs::read(corpus_path(&entry.path)).unwrap();
        let before = DocumentCounts {
            track_count: entry.track_count,
            clip_count: entry.clip_count,
            keyframe_count: entry.keyframe_count,
        };
        let (doc, report) = migrate_bytes(&bytes).unwrap();
        assert_eq!(count_document(&doc), before, "{}", entry.path);
        assert_eq!(doc.version, LATEST_DOCUMENT_VERSION);
        assert_eq!(doc.min_reader_version, LATEST_DOCUMENT_VERSION);
        assert!(!report.steps.is_empty(), "{}", entry.path);
    }
}

#[test]
fn nested_group_counts_are_included_in_invariants() {
    let doc = rich_v1_document();
    let counts = count_document(&doc);
    assert_eq!(counts.track_count, 1);
    assert_eq!(counts.clip_count, 2, "nested group child clip must count");
    assert_eq!(counts.keyframe_count, 2, "nested group keys must count");
    let bytes = serde_json::to_vec(&doc).unwrap();
    let (migrated, _) = migrate_bytes(&bytes).unwrap();
    assert_eq!(count_document(&migrated), counts);
}

#[test]
fn migrate_file_creates_backup_before_replace() {
    let dir = unique_dir("backup");
    let path = dir.join("legacy.json");
    let doc = rich_v1_document();
    let original = serde_json::to_vec_pretty(&doc).unwrap();
    fs::write(&path, &original).unwrap();
    let result = migrate_document_file(&path, &MigrateFileOptions::default()).unwrap();
    assert!(result.migrated);
    assert_eq!(fs::read(&result.backup_path).unwrap(), original);
    let loaded = load_document(&path).unwrap();
    assert_eq!(loaded.document.version, LATEST_DOCUMENT_VERSION);
    assert_eq!(loaded.document.min_reader_version, LATEST_DOCUMENT_VERSION);
    assert_eq!(count_document(&loaded.document), count_document(&doc));
    assert!(loaded.migrate_warnings.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn migrate_file_fails_closed_when_backup_exists() {
    let dir = unique_dir("bak-exists");
    let path = dir.join("legacy.json");
    let doc = rich_v1_document();
    let original = serde_json::to_vec_pretty(&doc).unwrap();
    fs::write(&path, &original).unwrap();
    let bak = path.with_file_name(format!("legacy.json{BACKUP_SUFFIX}"));
    let sentinel = b"last-known-good-backup";
    fs::write(&bak, sentinel).unwrap();

    let err = migrate_document_file(&path, &MigrateFileOptions::default()).unwrap_err();
    assert!(matches!(err, MigrateError::BackupExists(_)));
    assert_eq!(fs::read(&bak).unwrap(), sentinel);
    assert_eq!(fs::read(&path).unwrap(), original);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn forward_compat_min_reader_bump_rejects_old_reader() {
    let mut doc = Document::new_v1();
    bump_min_reader_for_nest_schema_change(&mut doc, 2);
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_reader_cap(&bytes, 1).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version: 2,
            reader_version: 1
        }
    ));
    let loaded = load_document_bytes_with_reader_cap(&bytes, READER_VERSION).unwrap();
    assert_eq!(loaded.document.min_reader_version, 2);
}

#[test]
fn v2_docs_reject_reader_version_1() {
    let doc = Document::new_v2();
    assert_eq!(doc.version, 2);
    assert!(doc.min_reader_version >= 2);
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_reader_cap(&bytes, 1).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version: 2,
            reader_version: 1
        }
    ));

    let v1 = Document::new_v1();
    let v1_bytes = serde_json::to_vec(&v1).unwrap();
    let (migrated, _) = migrate_bytes(&v1_bytes).unwrap();
    assert_eq!(migrated.version, 2);
    assert!(migrated.min_reader_version >= 2);
    let migrated_bytes = serde_json::to_vec(&migrated).unwrap();
    let err = load_document_bytes_with_reader_cap(&migrated_bytes, 1).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version: 2,
            reader_version: 1
        }
    ));
}

#[test]
fn prelude_time_map_applies_to_clips_including_nested() {
    let doc = rich_v1_document();
    // prelude 風 JSON: composition 無し + 直下 time_map + tracks あり
    let tm = TimeMap::constant_speed(
        RationalTime::ZERO,
        RationalTime::try_new(1, 2).unwrap(),
        2,
        1,
    )
    .unwrap();
    let json = serde_json::json!({
        "version": 1,
        "time_map": tm,
        "bpm": doc.bpm,
        "assets": doc.assets,
        "layers": doc.layers,
        "track_ids": doc.track_ids,
        "tracks": doc.tracks,
    });
    let bytes = serde_json::to_vec(&json).unwrap();
    let (migrated, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"prelude_time_map_to_clips"));
    assert!(report.warnings.is_empty());
    assert!(!migrated.extra.contains_key("_migrated_prelude_time_map"));

    fn collect_maps(item: &TrackItem, out: &mut Vec<TimeMap>) {
        match item {
            TrackItem::Clip(c) => out.push(c.time_map),
            TrackItem::Group(g) => {
                for child in &g.children {
                    collect_maps(child, out);
                }
            }
        }
    }
    let mut maps = Vec::new();
    for track in &migrated.tracks {
        for item in &track.items {
            collect_maps(item, &mut maps);
        }
    }
    assert_eq!(maps.len(), 2);
    assert!(maps.iter().all(|m| m == &tm));
}

#[test]
fn prelude_non_identity_time_map_without_clips_is_dropped_with_warning() {
    let tm = TimeMap::constant_speed(
        RationalTime::ZERO,
        RationalTime::try_new(1, 2).unwrap(),
        2,
        1,
    )
    .unwrap();
    let json = serde_json::json!({
        "version": 1,
        "time_map": tm,
    });
    let bytes = serde_json::to_vec(&json).unwrap();
    let (doc, report) = migrate_bytes(&bytes).unwrap();
    assert!(report
        .warnings
        .contains(&"prelude_time_map_dropped_no_clips"));
    assert!(!doc.extra.contains_key("_migrated_prelude_time_map"));
    assert!(!doc.extra.contains_key("time_map"));
}

#[test]
fn load_path_surfaces_prelude_time_map_dropped_warning() {
    let tm = TimeMap::constant_speed(
        RationalTime::ZERO,
        RationalTime::try_new(1, 2).unwrap(),
        2,
        1,
    )
    .unwrap();
    let json = serde_json::json!({
        "version": 1,
        "time_map": tm,
    });
    let bytes = serde_json::to_vec(&json).unwrap();
    let loaded = load_document_bytes(&bytes).unwrap();
    assert!(loaded
        .migrate_warnings
        .contains(&"prelude_time_map_dropped_no_clips"));
    assert_eq!(loaded.document.version, LATEST_DOCUMENT_VERSION);
}

#[test]
fn v1_json_omits_color_interpretation() {
    let doc = Document::new_v1();
    let value = serde_json::to_value(&doc).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["min_reader_version"], 1);
    assert!(value.get("color_interpretation").is_none());

    let v2 = Document::new_v2();
    let v2_value = serde_json::to_value(&v2).unwrap();
    assert_eq!(v2_value["version"], 2);
    assert_eq!(v2_value["min_reader_version"], 2);
    assert_eq!(v2_value["color_interpretation"], "straight_srgb");
}

#[test]
fn dry_run_does_not_create_backup() {
    let dir = unique_dir("dry-run");
    let path = dir.join("legacy.json");
    let doc = rich_v1_document();
    let original = serde_json::to_vec_pretty(&doc).unwrap();
    fs::write(&path, &original).unwrap();
    let result = migrate_document_file(&path, &MigrateFileOptions { dry_run: true }).unwrap();
    assert!(result.migrated);
    assert!(!result.backup_path.exists());
    assert_eq!(fs::read(&path).unwrap(), original);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn noop_migrate_already_latest_does_not_create_backup() {
    let dir = unique_dir("noop");
    let path = dir.join("current.json");
    let doc = Document::new_v2();
    save_document(&path, &doc).unwrap();
    let before = fs::read(&path).unwrap();
    let result = migrate_document_file(&path, &MigrateFileOptions::default()).unwrap();
    assert!(!result.migrated);
    assert!(!result.backup_path.exists());
    assert_eq!(fs::read(&path).unwrap(), before);
    let _ = fs::remove_dir_all(dir);
}
