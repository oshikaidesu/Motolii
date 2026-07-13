//! D1e: migration枠・旧形式変換・意味保存・OpenMode拒否。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::{RationalTime, TimeMap};
use motolii_doc::{
    bump_min_reader_for_nest_schema_change, check_migration_allowed, count_document,
    legacy_timemap_source, load_document_bytes, load_document_bytes_with_limits, migrate_bytes,
    migrate_bytes_with_limits, migrate_document_file, modern_timemap_source, save_document,
    semantic_fingerprint, Clip, ClipSource, DocParam, Document, DocumentCounts, ItemEnvelope,
    MigrateError, MigrateFileOptions, OpenMode, PersistError, ResourceLimits, Track, TrackItem,
    VectorContent, BACKUP_SUFFIX, LATEST_DOCUMENT_VERSION, READER_VERSION, WRITER_VERSION,
};
use serde::Deserialize;
use serde_json::json;

const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus");

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    path: String,
    track_count: usize,
    clip_count: usize,
    keyframe_count: usize,
    generation: String,
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

#[test]
fn golden_corpus_migrates_both_legacy_generations() {
    let manifest: CorpusManifest =
        serde_json::from_str(&fs::read_to_string(corpus_path("manifest.json")).unwrap()).unwrap();
    let mut saw_timeline = false;
    let mut saw_path_ops = false;
    for entry in &manifest.entries {
        let bytes = fs::read(corpus_path(&entry.path)).unwrap();
        let before = DocumentCounts {
            track_count: entry.track_count,
            clip_count: entry.clip_count,
            keyframe_count: entry.keyframe_count,
        };
        let (doc, report) = migrate_bytes(&bytes).unwrap_or_else(|e| {
            panic!("migrate {} failed: {e}", entry.path);
        });
        assert_eq!(count_document(&doc), before, "{}", entry.path);
        match entry.generation.as_str() {
            "timeline_start" => {
                saw_timeline = true;
                assert!(
                    report.steps.contains(&"drop_timeline_start"),
                    "{} steps={:?}",
                    entry.path,
                    report.steps
                );
            }
            "path_ops" => {
                saw_path_ops = true;
                assert!(
                    report.steps.contains(&"move_path_ops_to_recipe"),
                    "{} steps={:?}",
                    entry.path,
                    report.steps
                );
            }
            "current" => {
                assert!(!report.did_migrate(), "{} should be noop", entry.path);
            }
            other => panic!("unknown generation {other}"),
        }
        // load経路は旧形式を拒否したまま(変換はmigrateのみ)
        if entry.generation != "current" {
            let load_err = load_document_bytes(&bytes).unwrap_err();
            let msg = load_err.to_string();
            assert!(
                msg.contains("timeline_start")
                    || msg.contains("path_ops")
                    || msg.contains("unknown field"),
                "load must keep rejecting legacy for {}: {msg}",
                entry.path
            );
        }
    }
    assert!(
        saw_timeline && saw_path_ops,
        "corpus must cover both legacy generations"
    );
}

#[test]
fn timeline_start_preserves_timemap_and_param_semantics() {
    let bytes = fs::read(corpus_path("timeline_start/speed_clip.json")).unwrap();
    let root: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let clip = &root["tracks"][0]["items"][0];
    let clip_start = RationalTime::try_new(2, 1).unwrap();
    let source_start = RationalTime::try_new(1, 2).unwrap();
    let timeline_start = RationalTime::try_new(2, 1).unwrap();
    assert_eq!(clip["start"]["num"], 2);
    assert_eq!(clip["time_map"]["timeline_start"]["num"], 2);

    let sample_timeline = RationalTime::try_new(4, 1).unwrap();
    let expected_src =
        legacy_timemap_source(source_start, timeline_start, 2, 1, sample_timeline).unwrap();

    let (doc, _) = migrate_bytes(&bytes).unwrap();
    let TrackItem::Clip(migrated) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    assert!(!format!("{:?}", migrated.time_map).contains("timeline_start"));
    let got = modern_timemap_source(&migrated.time_map, clip_start, sample_timeline).unwrap();
    assert_eq!(got, expected_src);

    let t0 = RationalTime::ZERO;
    let t1 = RationalTime::try_new(1, 1).unwrap();
    let fp = semantic_fingerprint(&doc, &[t0, t1]);
    assert!(
        fp.param_evals
            .iter()
            .any(|(layer, name, v)| *layer == 0 && *name == "opacity" && v.contains("0.25")),
        "opacity@0 must survive: {:?}",
        fp.param_evals
    );
    assert!(
        fp.param_evals
            .iter()
            .any(|(layer, name, v)| *layer == 0 && *name == "position" && v.contains("0.1")),
        "position must survive: {:?}",
        fp.param_evals
    );
    assert!(
        !fp.timemap_samples.is_empty(),
        "timemap samples present: {:?}",
        fp.timemap_samples
    );
    // clip_local=0 → source_start; clip_local=1 → source_start + speed
    assert!(
        fp.timemap_samples.iter().any(|(_, local, src)| {
            local.contains("num: 0") && src.contains("num: 1") && src.contains("den: 2")
        }),
        "identity-at-origin sample: {:?}",
        fp.timemap_samples
    );
}

#[test]
fn path_ops_preserves_modifiers_and_dependency_edges() {
    let bytes = fs::read(corpus_path("path_ops/svg_with_ops.json")).unwrap();
    let (doc, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"move_path_ops_to_recipe"));
    let TrackItem::Clip(clip) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    let ClipSource::Vector { recipe } = &clip.source else {
        panic!("svg+path_ops must become Vector");
    };
    assert!(matches!(recipe.content, VectorContent::SvgAsset { .. }));
    assert_eq!(recipe.modifiers.len(), 3);
    // Twist.center 注入
    assert!(
        matches!(
            &recipe.modifiers[1],
            motolii_doc::PathOp::Twist { center, .. }
                if matches!(center, DocParam::Const(motolii_doc::DocValue::Vec2([0.0, 0.0])))
        ),
        "twist center default injected"
    );
    // Wiggle.seed: DocParam → u64
    assert!(
        matches!(
            &recipe.modifiers[2],
            motolii_doc::PathOp::Wiggle { seed: 7, .. }
        ),
        "wiggle seed coerced"
    );

    let t = RationalTime::ZERO;
    let fp = semantic_fingerprint(&doc, &[t]);
    assert!(
        fp.param_evals
            .iter()
            .any(|(_, name, v)| *name == "offset.distance" && v.contains("0.05")),
        "offset amount preserved: {:?}",
        fp.param_evals
    );
    assert!(
        fp.param_evals
            .iter()
            .any(|(_, name, v)| *name == "twist.angle" && v.contains("0.3")),
        "twist angle preserved: {:?}",
        fp.param_evals
    );

    // vector_with_clip_ops: path_ops が既存 modifiers の前に付く
    let bytes2 = fs::read(corpus_path("path_ops/vector_with_clip_ops.json")).unwrap();
    let (doc2, _) = migrate_bytes(&bytes2).unwrap();
    let TrackItem::Clip(clip2) = &doc2.tracks[0].items[0] else {
        panic!("clip");
    };
    let ClipSource::Vector { recipe } = &clip2.source else {
        panic!("vector");
    };
    assert_eq!(recipe.modifiers.len(), 2);
    assert!(matches!(
        &recipe.modifiers[0],
        motolii_doc::PathOp::PuckerBloat { .. }
    ));
    assert!(matches!(
        &recipe.modifiers[1],
        motolii_doc::PathOp::Trim { .. }
    ));
}

#[test]
fn dependency_edges_survive_migration() {
    let mut doc = Document::new_v1();
    let a = doc.layers.allocate("a").unwrap();
    let b = doc.layers.allocate("b").unwrap();
    let tid = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("m", "video/mp4", "h").unwrap();
    let mut env_b = ItemEnvelope::new(b);
    env_b.transform.parent = Some(a);
    env_b.transform.position = DocParam::Follow {
        target: a,
        offset: [0.1, 0.0],
    };
    doc.tracks.push(Track {
        id: tid,
        items: vec![
            TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(a),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(1, 1).unwrap(),
                time_map: TimeMap::identity(),
                source: ClipSource::Asset { asset },
            }),
            TrackItem::Clip(Clip {
                envelope: env_b,
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(1, 1).unwrap(),
                time_map: TimeMap::identity(),
                source: ClipSource::Asset { asset },
            }),
        ],
    });
    // 現行スキーマを一度legacy time_map風に壊してから戻す: timeline_startをJSONへ注入
    let mut value = serde_json::to_value(&doc).unwrap();
    value["tracks"][0]["items"][0]["time_map"] = json!({
        "source_start": {"num": 0, "den": 1},
        "timeline_start": {"num": 0, "den": 1},
        "speed_num": 1,
        "speed_den": 1
    });
    let bytes = serde_json::to_vec(&value).unwrap();
    let before_fp = {
        // 移行前はDocumentとして読めないので、現行docから指紋を取る
        semantic_fingerprint(&doc, &[RationalTime::ZERO])
    };
    let (migrated, report) = migrate_bytes(&bytes).unwrap();
    assert!(report.steps.contains(&"drop_timeline_start"));
    let after_fp = semantic_fingerprint(&migrated, &[RationalTime::ZERO]);
    assert_eq!(before_fp.dependency_edges, after_fp.dependency_edges);
    assert!(after_fp
        .dependency_edges
        .contains(&(b.get(), "parent", a.get())));
    assert!(after_fp
        .dependency_edges
        .contains(&(b.get(), "follow", a.get())));
}

#[test]
fn migrate_file_backup_before_replace_and_fail_closed() {
    let dir = unique_dir("backup");
    let path = dir.join("legacy.json");
    let original = fs::read(corpus_path("timeline_start/speed_clip.json")).unwrap();
    fs::write(&path, &original).unwrap();

    let result = migrate_document_file(&path, &MigrateFileOptions::default()).unwrap();
    assert!(result.migrated);
    assert_eq!(fs::read(&result.backup_path).unwrap(), original);
    let loaded = load_document_bytes(&fs::read(&path).unwrap()).unwrap();
    assert!(matches!(loaded.tracks[0].items[0], TrackItem::Clip(_)));
    let TrackItem::Clip(c) = &loaded.tracks[0].items[0] else {
        unreachable!()
    };
    assert_eq!(
        c.time_map.source_start,
        RationalTime::try_new(1, 2).unwrap()
    );
    assert_eq!(c.time_map.speed_num(), 2);

    // 既存bakがあると原本を壊さない
    let path2 = dir.join("legacy2.json");
    fs::write(&path2, &original).unwrap();
    let bak2 = path2.with_file_name(format!("legacy2.json{BACKUP_SUFFIX}"));
    let sentinel = b"last-known-good";
    fs::write(&bak2, sentinel).unwrap();
    let err = migrate_document_file(&path2, &MigrateFileOptions::default()).unwrap_err();
    assert!(matches!(err, MigrateError::BackupExists(_)));
    assert_eq!(fs::read(&bak2).unwrap(), sentinel);
    assert_eq!(fs::read(&path2).unwrap(), original);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn dry_run_and_noop_do_not_touch_files() {
    let dir = unique_dir("dry");
    let path = dir.join("legacy.json");
    let original = fs::read(corpus_path("path_ops/svg_with_ops.json")).unwrap();
    fs::write(&path, &original).unwrap();
    let result = migrate_document_file(&path, &MigrateFileOptions { dry_run: true }).unwrap();
    assert!(result.migrated);
    assert!(!result.backup_path.exists());
    assert_eq!(fs::read(&path).unwrap(), original);

    let current = dir.join("current.json");
    let doc = Document::new_v1();
    save_document(&current, &doc).unwrap();
    let before = fs::read(&current).unwrap();
    let noop = migrate_document_file(&current, &MigrateFileOptions::default()).unwrap();
    assert!(!noop.migrated);
    assert!(!noop.backup_path.exists());
    assert_eq!(fs::read(&current).unwrap(), before);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn open_mode_readonly_newer_and_reject_block_migration() {
    // Reject: min_reader超過
    let reject = json!({
        "version": 1,
        "min_reader_version": READER_VERSION + 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 1, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1}
    });
    let err = migrate_bytes(&serde_json::to_vec(&reject).unwrap()).unwrap_err();
    assert!(
        matches!(
            err,
            MigrateError::Persist(PersistError::ReaderTooOld { .. })
        ),
        "{err:?}"
    );

    // ReadOnlyNewer: version > WRITER
    let newer = json!({
        "version": WRITER_VERSION + 1,
        "min_reader_version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 1, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "time_map_dummy": true
    });
    // Document.version が高いだけなら deserializeは通るが OpenMode で拒否
    let err = migrate_bytes(&serde_json::to_vec(&newer).unwrap()).unwrap_err();
    assert!(
        matches!(
            err,
            MigrateError::Persist(PersistError::SaveRejectedReadOnlyNewer { .. })
        ),
        "{err:?}"
    );

    let mut doc = Document::new_v1();
    doc.version = WRITER_VERSION + 1;
    assert!(matches!(
        check_migration_allowed(&doc),
        Err(PersistError::SaveRejectedReadOnlyNewer { .. })
    ));
}

#[test]
fn migration_honors_resource_limits_no_bypass() {
    let bytes = fs::read(corpus_path("timeline_start/speed_clip.json")).unwrap();
    let mut tight = ResourceLimits::production();
    tight.max_file_bytes = 16;
    let err = migrate_bytes_with_limits(&bytes, &tight).unwrap_err();
    assert!(matches!(err, MigrateError::ResourceLimit(_)), "{err:?}");
    // 同じlimitsでloadも拒否(別経路なし)
    let load_err = load_document_bytes_with_limits(&bytes, &tight).unwrap_err();
    assert!(matches!(load_err, PersistError::ResourceLimit(_)));
}

#[test]
fn forward_compat_min_reader_bump_path() {
    let mut doc = Document::new_v1();
    bump_min_reader_for_nest_schema_change(&mut doc, LATEST_DOCUMENT_VERSION);
    assert!(doc.min_reader_version >= LATEST_DOCUMENT_VERSION);
    assert!(doc.version >= LATEST_DOCUMENT_VERSION);
    let bytes = serde_json::to_vec(&doc).unwrap();
    let err = load_document_bytes_with_limits(
        &bytes,
        &ResourceLimits {
            max_file_bytes: ResourceLimits::production().max_file_bytes,
            ..ResourceLimits::production()
        },
    );
    // READER_VERSION==LATEST なので読める
    let opened = err.unwrap();
    assert_eq!(opened.open_mode, OpenMode::ReadWrite);
    assert_eq!(opened.document.min_reader_version, LATEST_DOCUMENT_VERSION);
}

#[test]
fn raster_path_ops_are_rejected_not_silently_dropped() {
    let doc = json!({
        "version": 1,
        "composition": {
            "aspect_num": 16,
            "aspect_den": 9,
            "duration": {"num": 1, "den": 1},
            "fps": {"num": 30, "den": 1}
        },
        "bpm": {"num": 120, "den": 1},
        "assets": {
            "next": 1,
            "entries": [{
                "id": 0,
                "name": "vid",
                "asset_type": "video/mp4",
                "content_hash": "h"
            }]
        },
        "layers": {"next": 1, "entries": [{"id": 0, "name": "A"}]},
        "track_ids": {"next": 1, "entries": [{"id": 0, "name": "V1"}]},
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
                    },
                    "opacity": {"const": {"F64": 1.0}}
                },
                "start": {"num": 0, "den": 1},
                "duration": {"num": 1, "den": 1},
                "source": {"source": "asset", "asset": 0},
                "path_ops": [{"op": "offset", "distance": {"const": {"F64": 0.1}}}]
            }]
        }]
    });
    let err = migrate_bytes(&serde_json::to_vec(&doc).unwrap()).unwrap_err();
    assert!(
        matches!(err, MigrateError::PathOpsOnRaster { .. }),
        "{err:?}"
    );
}
