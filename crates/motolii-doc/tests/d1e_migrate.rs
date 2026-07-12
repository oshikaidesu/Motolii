//! D1e integration tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    bump_min_reader_for_nest_schema_change, count_document, load_document,
    load_document_bytes_with_reader_cap, migrate_bytes, migrate_document_file,
    Asset, AssetId, Clip, ClipSource, DocParam, Document, DocumentCounts, Group,
    ItemEnvelope, LATEST_DOCUMENT_VERSION, MigrateFileOptions, PersistError, READER_VERSION,
    Track, TrackItem,
};
use motolii_eval::{Interp, Keyframe, KeyframeTrack, Value as EvalValue};
use serde::Deserialize;

const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus");

#[derive(Debug, Deserialize)]
struct CorpusEntry { path: String, track_count: usize, clip_count: usize, keyframe_count: usize }
#[derive(Debug, Deserialize)]
struct CorpusManifest { entries: Vec<CorpusEntry> }

fn corpus_path(rel: &str) -> PathBuf { Path::new(CORPUS_DIR).join(rel) }
fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1e-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn rich_v1_document() -> Document {
    let mut doc = Document::new_v1();
    let asset_id = AssetId::from_raw(0);
    doc.assets.insert(Asset { id: asset_id, name: "bg".into(), asset_type: "video/mp4".into(), content_hash: "h".into(), path_absolute: None, path_project_relative: None, file_name: None, size_bytes: None, head_hash: None, tail_hash: None }).unwrap();
    let clip_layer = doc.layers.allocate("a").unwrap();
    let group_layer = doc.layers.allocate("g").unwrap();
    let child_layer = doc.layers.allocate("c").unwrap();
    let track_id = doc.track_ids.allocate("V1").unwrap();
    let mut keys = KeyframeTrack::new();
    keys.insert(Keyframe { t: RationalTime::ZERO, value: EvalValue::F64(0.0), interp: Interp::Linear });
    keys.insert(Keyframe { t: RationalTime::try_new(1, 1).unwrap(), value: EvalValue::F64(1.0), interp: Interp::Hold });
    let child_clip = Clip { envelope: { let mut e = ItemEnvelope::new(child_layer); e.opacity = DocParam::Keyframes(keys); e }, start: RationalTime::ZERO, duration: RationalTime::try_new(5,1).unwrap(), time_map: TimeMap::identity(), source: ClipSource::Asset { asset: asset_id }, path_ops: Vec::new() };
    let top_clip = Clip { envelope: ItemEnvelope::new(clip_layer), start: RationalTime::ZERO, duration: RationalTime::try_new(10,1).unwrap(), time_map: TimeMap::identity(), source: ClipSource::Asset { asset: asset_id }, path_ops: Vec::new() };
    doc.tracks.push(Track { id: track_id, items: vec![TrackItem::Clip(top_clip), TrackItem::Group(Group { envelope: ItemEnvelope::new(group_layer), children: vec![TrackItem::Clip(child_clip)] })] });
    doc
}

#[test]
fn golden_corpus_migrates_with_stable_counts() {
    let manifest: CorpusManifest = serde_json::from_str(&fs::read_to_string(corpus_path("manifest.json")).unwrap()).unwrap();
    for entry in manifest.entries {
        let bytes = fs::read(corpus_path(&entry.path)).unwrap();
        let before = DocumentCounts { track_count: entry.track_count, clip_count: entry.clip_count, keyframe_count: entry.keyframe_count };
        let (doc, report) = migrate_bytes(&bytes).unwrap();
        assert_eq!(count_document(&doc), before, "{}", entry.path);
        assert_eq!(doc.version, LATEST_DOCUMENT_VERSION);
        assert!(!report.steps.is_empty(), "{}", entry.path);
    }
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
    assert_eq!(loaded.version, LATEST_DOCUMENT_VERSION);
    assert_eq!(count_document(&loaded), count_document(&doc));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn forward_compat_min_reader_bump_rejects_old_reader() {
    let mut doc = Document::new_v1();
    bump_min_reader_for_nest_schema_change(&mut doc, 2);
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_reader_cap(&bytes, 1).unwrap_err();
    assert!(matches!(err, PersistError::ReaderTooOld { min_reader_version: 2, reader_version: 1 }));
    let loaded = load_document_bytes_with_reader_cap(&bytes, READER_VERSION).unwrap();
    assert_eq!(loaded.min_reader_version, 2);
}
