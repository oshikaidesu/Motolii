#![allow(deprecated)]

//! D1d: journal checksum/salt/UUID・壊れ方catalog・故障注入・非破壊recovery・ResourceLimits。

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_core::RationalTime;
use motolii_doc::journal::{
    generation_path_for_document, journal_path_for_document, restore_attempted_path, scan_journal,
    FaultInjectingFs, FaultPlan, JournalEdit, JournalFs, JournalRecordKind,
};
use motolii_doc::{
    load_catalog, Bpm, Clip, ClipSource, Command, DocParam, Document, ItemEnvelope, LayerId,
    PinGenerationOptions, ProjectError, ProjectSession, RecoverySource, ResourceLimits,
    RotateOptions, SaveProjectOptions, ScalarPropertyId, SessionError, Track, TrackItem, WalError,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1d-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn tiny_limits() -> ResourceLimits {
    ResourceLimits {
        max_file_bytes: 1_000_000,
        max_group_depth: 64,
        max_tracks: 64,
        max_layers: 64,
        max_keys_per_track: 64,
        max_string_bytes: 1024,
        max_extra_bytes: 1024,
        max_command_payload_bytes: 32,
        max_journal_bytes: 2_000,
        max_samples: 8,
    }
}

/// 1 clip を持つ最小文書(Command リプレイ試験用)。
fn doc_with_clip() -> (Document, LayerId) {
    let mut doc = Document::new_current();
    let layer = doc.layers.allocate("a").unwrap();
    let track = doc.track_ids.allocate("V1").unwrap();
    let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
    doc.tracks.push(Track {
        id: track,
        items: vec![TrackItem::Clip(Clip {
            envelope: ItemEnvelope::new(layer),
            start: RationalTime::ZERO,
            duration: RationalTime::try_new(5, 1).unwrap(),
            time_map: Default::default(),
            source: ClipSource::asset_video_only(asset),
        })],
    });
    doc.validate().expect("fixture must validate");
    (doc, layer)
}

fn set_opacity_cmd(layer: LayerId, old: f64, new: f64) -> JournalEdit {
    JournalEdit::new(Command::SetProperty {
        target: layer,
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(old),
        new_value: DocParam::const_f64(new),
    })
}

fn clip_opacity(doc: &Document) -> DocParam {
    let TrackItem::Clip(c) = &doc.tracks[0].items[0] else {
        panic!("expected clip");
    };
    c.envelope.opacity.clone()
}

#[test]
fn journal_module_has_no_truncate_repair_path() {
    // 完了条件: 原本をtruncateして「修復」する経路が無いこと。
    let src = include_str!("../src/journal/format.rs");
    let wal = include_str!("../src/journal/wal.rs");
    let recover = include_str!("../src/journal/recover.rs");
    let project = include_str!("../src/journal/project.rs");
    let replay = include_str!("../src/journal/replay.rs");
    for (name, text) in [
        ("format", src),
        ("wal", wal),
        ("recover", recover),
        ("project", project),
        ("replay", replay),
    ] {
        assert!(
            !text.contains("set_len"),
            "{name}.rs must not truncate via set_len"
        );
        assert!(
            !text.contains("truncate_journal"),
            "{name}.rs must not define truncate_journal"
        );
        assert!(
            !text.contains("ForceReplayFail"),
            "{name}.rs must not bake ForceReplayFail into durable journal types"
        );
    }
}

#[test]
fn roundtrip_checkpoint_open() {
    let dir = unique_dir("roundtrip");
    let path = dir.join("proj.json");
    let mut doc = Document::new_current();
    doc.bpm = Bpm::try_new(140, 1).unwrap();
    common::session::save_journal(&path, &doc, &SaveProjectOptions::default());
    let (_session, opened) = common::session::open_recovered(&path);
    assert_eq!(opened.document.bpm, doc.bpm);
    assert_eq!(opened.source, RecoverySource::MainFile);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn replay_applies_committed_edits_when_main_behind() {
    let dir = unique_dir("replay");
    let path = dir.join("proj.json");
    let (doc, layer) = doc_with_clip();
    common::session::save_journal(&path, &doc, &SaveProjectOptions::default());

    let edit = set_opacity_cmd(layer, 1.0, 0.25);
    common::session::save_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(edit),
            checkpoint: false,
            ..Default::default()
        },
    );

    // mainは旧のまま
    assert_eq!(
        clip_opacity(&motolii_doc::load_document(&path).unwrap()),
        DocParam::const_f64(1.0)
    );

    let (_session, opened) = common::session::open_recovered(&path);
    assert_eq!(clip_opacity(&opened.document), DocParam::const_f64(0.25));
    assert!(matches!(
        opened.source,
        RecoverySource::JournalReplay | RecoverySource::CommittedPrefixReplay
    ));
    // 原本mainは上書きしない
    assert_eq!(
        clip_opacity(&motolii_doc::load_document(&path).unwrap()),
        DocParam::const_f64(1.0)
    );
    assert!(opened.recovered_path.is_some());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn pinned_generation_survives_rotation() {
    let dir = unique_dir("pinned");
    let path = dir.join("proj.json");
    let mut doc = Document::new_current();
    doc.bpm = Bpm::try_new(110, 1).unwrap();
    common::session::save_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    );
    let catalog = load_catalog(&path).unwrap().unwrap();
    let pinned_id = catalog.generations[0].id;

    common::session::save_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            pin_generation: Some(PinGenerationOptions {
                generation_id: pinned_id,
            }),
            max_unpinned_generations: Some(0),
            rotate: RotateOptions {
                max_unpinned: Some(0),
            },
            ..Default::default()
        },
    );

    // さらに世代を作ってunpinnedを回す
    doc.bpm = Bpm::try_new(111, 1).unwrap();
    common::session::save_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    );

    let catalog = load_catalog(&path).unwrap().unwrap();
    assert!(
        catalog
            .generations
            .iter()
            .any(|g| g.id == pinned_id && g.pinned),
        "pinned generation must remain: {catalog:?}"
    );
    let gen_path = generation_path_for_document(&path, pinned_id);
    assert!(gen_path.exists(), "pinned snapshot file must survive");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn uuid_cross_refs_link_snapshot_and_journal_record() {
    let dir = unique_dir("uuid-refs");
    let path = dir.join("proj.json");
    common::session::save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let catalog = load_catalog(&path).unwrap().unwrap();
    let entry = &catalog.generations[0];
    assert_ne!(entry.id, uuid::Uuid::nil());
    assert_ne!(entry.journal_record, uuid::Uuid::nil());
    assert_eq!(catalog.project_id, {
        let journal = journal_path_for_document(&path);
        scan_journal(&journal, &Default::default())
            .unwrap()
            .header
            .project_id
    });

    let journal = journal_path_for_document(&path);
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    let snap = scan
        .frames
        .iter()
        .find(|f| f.kind == JournalRecordKind::Snapshot)
        .expect("snapshot frame");
    assert_eq!(snap.snapshot_ref, Some(entry.id));
    assert_eq!(snap.record_id, entry.journal_record);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn journal_limits_return_typed_errors() {
    let dir = unique_dir("limits-payload");
    let path = dir.join("proj.json");

    let mut tiny = tiny_limits();
    tiny.max_command_payload_bytes = 1;
    let err = common::session::save_journal_result(
        &path,
        &Document::new_current(),
        &SaveProjectOptions {
            limits: tiny,
            journal_edit: Some(set_opacity_cmd(LayerId::from_raw(1), 1.0, 0.5)),
            checkpoint: false,
            ..Default::default()
        },
    )
    .map_err(ProjectError::from)
    .unwrap_err();
    assert!(
        matches!(err, ProjectError::Wal(WalError::RecordPayloadLimit { .. })),
        "got {err:?}"
    );
    let _ = fs::remove_dir_all(dir);

    let dir2 = unique_dir("limits-total");
    let path2 = dir2.join("proj.json");
    common::session::save_journal(
        &path2,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let journal = journal_path_for_document(&path2);
    let jlen = fs::metadata(&journal).unwrap().len();
    let mut tight = ResourceLimits::production();
    tight.max_journal_bytes = jlen.saturating_sub(1).max(1);
    let err = open_project_with_limits_expect(&path2, &tight);
    assert!(
        matches!(
            err,
            ProjectError::Recovery(motolii_doc::journal::RecoveryError::Persist(
                motolii_doc::PersistError::ResourceLimit(
                    motolii_doc::ResourceLimitError::JournalBytes { .. }
                )
            )) | ProjectError::Persist(motolii_doc::PersistError::ResourceLimit(
                motolii_doc::ResourceLimitError::JournalBytes { .. }
            )) | ProjectError::Session(SessionError::Recovery(
                motolii_doc::journal::RecoveryError::Persist(
                    motolii_doc::PersistError::ResourceLimit(
                        motolii_doc::ResourceLimitError::JournalBytes { .. }
                    )
                )
            ))
        ),
        "got {err:?}"
    );
    let _ = fs::remove_dir_all(dir2);
}

fn open_project_with_limits_expect(path: &Path, limits: &ResourceLimits) -> ProjectError {
    ProjectSession::open(path, limits)
        .map_err(ProjectError::from)
        .unwrap_err()
}

#[test]
fn fault_partial_write_then_recover_ignores_tail() {
    let dir = unique_dir("partial-fault");
    let path = dir.join("proj.json");
    common::session::save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let mut faulty = FaultInjectingFs::new(FaultPlan::PartialWrite { max_bytes: 8 });
    faulty.seed_from_disk(&dir).unwrap();
    let journal = journal_path_for_document(&path);
    // 既存journalへ短いゴミ相当の部分append
    let _ = faulty.append(&journal, &[0u8; 64]);
    faulty.flush_durable_to_disk().unwrap();

    let (_session, opened) = common::session::open_recovered(&path);
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_rename_not_durable_loses_rename_on_crash() {
    let dir = unique_dir("rename-nd");
    let path = dir.join("proj.json");
    common::session::save_document_via_session(&path, &Document::new_current());

    let mut faulty = FaultInjectingFs::new(FaultPlan::RenameNotDurable);
    faulty.seed_from_disk(&dir).unwrap();
    let tmp = dir.join("tmp.json");
    let mut newer = Document::new_current();
    newer.bpm = Bpm::try_new(150, 1).unwrap();
    let bytes = serde_json::to_vec_pretty(&newer).unwrap();
    faulty.write_create(&tmp, &bytes).unwrap();
    faulty.sync_file(&tmp).unwrap();
    faulty.rename(&tmp, &path).unwrap();
    // SyncDir前にcrash → rename未永続
    faulty.crash();
    faulty.flush_durable_to_disk().unwrap();

    let loaded = motolii_doc::load_document(&path).unwrap();
    assert_eq!(loaded.bpm, Bpm::try_new(120, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_reorder_append_not_visible_until_sync() {
    let dir = unique_dir("reorder");
    let path = dir.join("proj.json");
    common::session::save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let mut faulty = FaultInjectingFs::new(FaultPlan::ReorderPendingAppend);
    faulty.seed_from_disk(&dir).unwrap();
    let journal = journal_path_for_document(&path);
    let before = faulty.read(&journal).unwrap().len();
    faulty.append(&journal, b"NOT_YET").unwrap();
    // sync前はdurableに見えない
    assert_eq!(faulty.durable_get(&journal).unwrap().len(), before);
    faulty.sync_file(&journal).unwrap();
    assert!(faulty.durable_get(&journal).unwrap().len() > before);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn recovery_crash_loop_skips_replay_via_marker() {
    let dir = unique_dir("recovery-recrash");
    let path = dir.join("proj.json");
    common::session::save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    // tipに合わせたrestore_attemptedを手書き
    let journal = journal_path_for_document(&path);
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    let marker = serde_json::json!({
        "format_version": 1,
        "project_id": scan.header.project_id,
        "generation_salt": scan.header.generation_salt,
        "tip_record_id": scan.frames.last().map(|f| f.record_id),
        "valid_bytes": scan.valid_bytes,
    });
    let marker_path = restore_attempted_path(&path);
    fs::write(&marker_path, serde_json::to_vec_pretty(&marker).unwrap()).unwrap();

    // mainを壊してgenerationフォールバック経路へ
    fs::write(&path, b"{not-json").unwrap();
    let (_session, opened) = common::session::open_recovered(&path);
    assert!(
        opened
            .warnings
            .iter()
            .any(|w| w.contains("restore_attempted")),
        "warnings={:?}",
        opened.warnings
    );
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncated_journal_header_is_not_overwritten() {
    use motolii_doc::journal::{read_or_create_header, JournalFormatError, StdFs, HEADER_LEN};
    use uuid::Uuid;

    let dir = unique_dir("trunc-header");
    let path = dir.join("proj.json");
    let journal = journal_path_for_document(&path);
    fs::create_dir_all(journal.parent().unwrap()).unwrap();
    fs::write(&journal, b"SHORT").unwrap();
    let before = fs::read(&journal).unwrap();
    let mut fs_impl = StdFs;
    let err = read_or_create_header(&mut fs_impl, &journal, Uuid::nil(), 1).unwrap_err();
    assert!(
        matches!(
            err,
            JournalFormatError::TruncatedHeader {
                observed: 5,
                needed: HEADER_LEN
            }
        ),
        "unexpected {err:?}"
    );
    assert_eq!(
        fs::read(&journal).unwrap(),
        before,
        "must not overwrite short wal"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn marker_remount_does_not_prefer_stale_main_over_journal_edits() {
    let dir = unique_dir("marker-stale-main");
    let path = dir.join("proj.json");
    let (doc, layer) = doc_with_clip();
    common::session::save_journal(&path, &doc, &SaveProjectOptions::default());

    common::session::save_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(set_opacity_cmd(layer, 1.0, 0.4)),
            checkpoint: false,
            ..Default::default()
        },
    );

    // tipマーカーを残し、mainは編集前のまま(stale)
    let journal = journal_path_for_document(&path);
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    let marker = serde_json::json!({
        "format_version": 1,
        "project_id": scan.header.project_id,
        "generation_salt": scan.header.generation_salt,
        "tip_record_id": scan.frames.last().map(|f| f.record_id),
        "valid_bytes": scan.valid_bytes,
    });
    let marker_path = restore_attempted_path(&path);
    fs::write(&marker_path, serde_json::to_vec_pretty(&marker).unwrap()).unwrap();

    assert_eq!(
        clip_opacity(&motolii_doc::load_document(&path).unwrap()),
        DocParam::const_f64(1.0),
        "precondition: main must stay stale"
    );

    let (_session, opened) = common::session::open_recovered(&path);
    assert_eq!(
        clip_opacity(&opened.document),
        DocParam::const_f64(0.4),
        "marker remount must replay committed edits, not return stale main"
    );
    assert!(
        opened
            .warnings
            .iter()
            .any(|w| w.contains("restore_attempted")),
        "warnings={:?}",
        opened.warnings
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn d1c_save_document_alone_still_works() {
    let dir = unique_dir("d1c-compat");
    let path = dir.join("doc.json");
    let doc = Document::new_current();
    common::session::save_document_via_session(&path, &doc);
    assert_eq!(motolii_doc::load_document(&path).unwrap(), doc);
    let _ = fs::remove_dir_all(dir);
}
