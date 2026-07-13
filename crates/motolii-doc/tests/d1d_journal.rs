//! D1d: journal checksum/salt/UUID・壊れ方catalog・故障注入・非破壊recovery・ResourceLimits。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::journal::{
    journal_path_for_document, scan_journal, CheckpointOptions, DurabilityStage, FaultInjectingFs,
    FaultPlan, FsOpKind, JournalEdit, JournalFs, JournalRecordKind, JournalScanStop, RecordingFs,
    StdFs, WalSession,
};
use motolii_doc::{
    checkpoint_with_fault_plan, inject_bad_checksum_at_last_frame, inject_corrupt_journal_tail,
    inject_salt_mismatch_frame, load_catalog, open_project, save_document, save_project_with_journal,
    Bpm, Document, PinGenerationOptions, ProjectError, RecoverySource, ResourceLimits,
    RotateOptions, SaveProjectOptions,
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

#[test]
fn journal_module_has_no_truncate_repair_path() {
    // 完了条件: 原本をtruncateして「修復」する経路が無いこと。
    let src = include_str!("../src/journal/format.rs");
    let wal = include_str!("../src/journal/wal.rs");
    let recover = include_str!("../src/journal/recover.rs");
    let project = include_str!("../src/journal/project.rs");
    for (name, text) in [
        ("format", src),
        ("wal", wal),
        ("recover", recover),
        ("project", project),
    ] {
        assert!(
            !text.contains("set_len"),
            "{name}.rs must not truncate via set_len"
        );
        assert!(
            !text.contains("truncate_journal"),
            "{name}.rs must not define truncate_journal"
        );
    }
}

#[test]
fn commit_and_checkpoint_fsync_order_is_fixed() {
    let dir = unique_dir("order");
    let path = dir.join("proj.json");
    let doc = Document::new_v1();

    let (mut fs, log) = RecordingFs::new(StdFs);
    let project_id = uuid::Uuid::new_v4();
    let salt = 0x1111_2222_3333_4444;
    let mut session =
        WalSession::open_or_create(&mut fs, &path, project_id, salt, 5).unwrap();

    motolii_doc::journal::commit_edit(
        &mut fs,
        &mut session,
        &JournalEdit::SetBpm { num: 128, den: 1 },
        &ResourceLimits::production(),
    )
    .unwrap();

    {
        let ops = log.lock().unwrap();
        let stages: Vec<_> = ops
            .iter()
            .filter(|o| o.kind == FsOpKind::NoteStage)
            .map(|o| o.detail.clone())
            .collect();
        assert!(
            stages.windows(4).any(|w| {
                w[0].contains("JournalAppend")
                    && w[1].contains("JournalFsync")
                    && w[2].contains("JournalAppend")
                    && w[3].contains("JournalFsync")
            }),
            "commit order must be append→fsync→append→fsync, got {stages:?}"
        );
    }

    log.lock().unwrap().clear();
    motolii_doc::journal::checkpoint(
        &mut fs,
        &path,
        &mut session,
        &doc,
        &CheckpointOptions::default(),
        &ResourceLimits::production(),
    )
    .unwrap();

    let ops = log.lock().unwrap();
    let stages: Vec<_> = ops
        .iter()
        .filter(|o| o.kind == FsOpKind::NoteStage)
        .map(|o| o.detail.clone())
        .collect();
    let required = [
        "MainTempWrite",
        "MainTempFsync",
        "MainRename",
        "MainDirFsync",
        "CheckpointAppend",
        "CheckpointFsync",
        "CatalogWrite",
        "CatalogFsync",
    ];
    let mut pos = 0usize;
    for req in required {
        let found = stages[pos..]
            .iter()
            .position(|s| s.contains(req))
            .unwrap_or_else(|| panic!("missing stage {req} in {stages:?}"));
        pos += found + 1;
    }
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn roundtrip_checkpoint_open() {
    let dir = unique_dir("roundtrip");
    let path = dir.join("proj.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(140, 1).unwrap();
    save_project_with_journal(&path, &doc, &SaveProjectOptions::default()).unwrap();
    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, doc.bpm);
    assert_eq!(opened.source, RecoverySource::MainFile);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn corrupt_tail_is_ignored_without_truncating_journal() {
    let dir = unique_dir("partial-tail");
    let path = dir.join("proj.json");
    let doc = Document::new_v1();
    save_project_with_journal(&path, &doc, &SaveProjectOptions::default()).unwrap();

    let journal = journal_path_for_document(&path);
    let before = fs::metadata(&journal).unwrap().len();
    inject_corrupt_journal_tail(&path, b"GARBAGE_PARTIAL_WRITE").unwrap();
    let after = fs::metadata(&journal).unwrap().len();
    assert!(after > before, "tail garbage must extend journal");

    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::PartialFrame));
    assert_eq!(scan.ignored_tail_bytes(), after - scan.valid_bytes);

    let opened = open_project(&path).unwrap();
    assert!(opened.ignored_tail_bytes > 0);
    // 原本長は変わらない(非破壊)
    assert_eq!(fs::metadata(&journal).unwrap().len(), after);
    assert!(
        opened.corrupt_path.is_some(),
        "corrupt copy should be written"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn bad_checksum_stops_scan_without_truncate() {
    let dir = unique_dir("checksum");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();
    let journal = journal_path_for_document(&path);
    let len_before = fs::metadata(&journal).unwrap().len();

    inject_bad_checksum_at_last_frame(&path).unwrap();
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::ChecksumMismatch));
    let opened = open_project(&path).unwrap();
    assert_eq!(fs::metadata(&journal).unwrap().len(), len_before);
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn salt_mismatch_stops_scan_without_truncate() {
    let dir = unique_dir("salt");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();
    let journal = journal_path_for_document(&path);

    inject_salt_mismatch_frame(&path).unwrap();
    let len_after = fs::metadata(&journal).unwrap().len();
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::SaltMismatch));
    let opened = open_project(&path).unwrap();
    assert_eq!(fs::metadata(&journal).unwrap().len(), len_after);
    assert!(opened.ignored_tail_bytes > 0);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn replay_applies_committed_edits_when_main_behind() {
    let dir = unique_dir("replay");
    let path = dir.join("proj.json");
    let doc = Document::new_v1();
    save_project_with_journal(&path, &doc, &SaveProjectOptions::default()).unwrap();

    let mut edited = Document::new_v1();
    edited.bpm = Bpm::try_new(160, 1).unwrap();
    save_project_with_journal(
        &path,
        &edited,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 160, den: 1 }),
            checkpoint: false,
            ..Default::default()
        },
    )
    .unwrap();

    // mainは旧のまま
    assert_eq!(
        motolii_doc::load_document(&path).unwrap().bpm,
        Bpm::try_new(120, 1).unwrap()
    );

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, Bpm::try_new(160, 1).unwrap());
    assert!(matches!(
        opened.source,
        RecoverySource::JournalReplay | RecoverySource::CommittedPrefixReplay
    ));
    // 原本mainは上書きしない
    assert_eq!(
        motolii_doc::load_document(&path).unwrap().bpm,
        Bpm::try_new(120, 1).unwrap()
    );
    assert!(opened.recovered_path.is_some());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn replay_failure_falls_back_to_snapshot() {
    let dir = unique_dir("replay-fail");
    let path = dir.join("proj.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(100, 1).unwrap();
    save_project_with_journal(&path, &doc, &SaveProjectOptions::default()).unwrap();

    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::ForceReplayFail),
            checkpoint: false,
            ..Default::default()
        },
    )
    .unwrap();

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.source, RecoverySource::SnapshotFallback);
    assert_eq!(opened.document.bpm, Bpm::try_new(100, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn pinned_generation_survives_rotation() {
    let dir = unique_dir("pinned");
    let path = dir.join("proj.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(110, 1).unwrap();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = load_catalog(&path).unwrap().unwrap();
    let pinned_id = catalog.generations[0].id;

    save_project_with_journal(
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
    )
    .unwrap();

    // さらに世代を作ってunpinnedを回す
    doc.bpm = Bpm::try_new(111, 1).unwrap();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let catalog = load_catalog(&path).unwrap().unwrap();
    assert!(
        catalog
            .generations
            .iter()
            .any(|g| g.id == pinned_id && g.pinned),
        "pinned generation must remain: {catalog:?}"
    );
    let gen_path = path
        .parent()
        .unwrap()
        .join(".motolii")
        .join("generations")
        .join(format!("{pinned_id}.json"));
    assert!(gen_path.exists(), "pinned snapshot file must survive");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn uuid_cross_refs_link_snapshot_and_journal_record() {
    let dir = unique_dir("uuid-refs");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();
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
    let err = save_project_with_journal(
        &path,
        &Document::new_v1(),
        &SaveProjectOptions {
            limits: tiny,
            journal_edit: Some(JournalEdit::SetBpm { num: 1, den: 1 }),
            checkpoint: false,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(
        matches!(err, ProjectError::Wal(motolii_doc::journal::WalError::RecordPayloadLimit { .. })),
        "got {err:?}"
    );
    let _ = fs::remove_dir_all(dir);

    let dir2 = unique_dir("limits-total");
    let path2 = dir2.join("proj.json");
    save_project_with_journal(&path2, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();
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
            ))
        ),
        "got {err:?}"
    );
    let _ = fs::remove_dir_all(dir2);
}

fn open_project_with_limits_expect(path: &Path, limits: &ResourceLimits) -> ProjectError {
    motolii_doc::open_project_with_limits(path, limits).unwrap_err()
}

#[test]
fn fault_enospace_on_journal_append() {
    let dir = unique_dir("enospc");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();

    let err = checkpoint_with_fault_plan(
        &path,
        &Document::new_v1(),
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 130, den: 1 }),
            checkpoint: false,
            ..Default::default()
        },
        FaultPlan::Enospc,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("ENOSPC") || msg.contains("StorageFull") || matches!(err, ProjectError::Fs(_)),
        "got {err:?}"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_kill_during_checkpoint_keeps_old_or_complete() {
    let dir = unique_dir("kill-ckpt");
    let path = dir.join("proj.json");
    let old = Document::new_v1();
    save_project_with_journal(&path, &old, &SaveProjectOptions::default()).unwrap();
    let old_bytes = fs::read(&path).unwrap();

    let mut newer = Document::new_v1();
    newer.bpm = Bpm::try_new(200, 1).unwrap();
    let err = checkpoint_with_fault_plan(
        &path,
        &newer,
        &SaveProjectOptions::default(),
        FaultPlan::KillAfter(DurabilityStage::MainTempFsync),
    );
    assert!(err.is_err(), "kill must abort checkpoint");
    // rename前killなら旧mainが残る
    let now = fs::read(&path).unwrap();
    assert_eq!(now, old_bytes);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_partial_write_then_recover_ignores_tail() {
    let dir = unique_dir("partial-fault");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();

    let mut faulty = FaultInjectingFs::new(FaultPlan::PartialWrite { max_bytes: 8 });
    faulty.seed_from_disk(&dir).unwrap();
    let journal = journal_path_for_document(&path);
    // 既存journalへ短いゴミ相当の部分append
    let _ = faulty.append(&journal, &[0u8; 64]);
    faulty.flush_durable_to_disk().unwrap();

    let opened = open_project(&path).unwrap();
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_rename_not_durable_loses_rename_on_crash() {
    let dir = unique_dir("rename-nd");
    let path = dir.join("proj.json");
    save_document(&path, &Document::new_v1()).unwrap();

    let mut faulty = FaultInjectingFs::new(FaultPlan::RenameNotDurable);
    faulty.seed_from_disk(&dir).unwrap();
    let tmp = dir.join("tmp.json");
    let mut newer = Document::new_v1();
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
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();

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
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();

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
    let marker_path = path.parent().unwrap().join(".motolii/restore_attempted.json");
    fs::write(&marker_path, serde_json::to_vec_pretty(&marker).unwrap()).unwrap();

    // mainを壊してgenerationフォールバック経路へ
    fs::write(&path, b"{not-json").unwrap();
    let opened = open_project(&path).unwrap();
    assert!(
        opened.warnings.iter().any(|w| w.contains("restore_attempted")),
        "warnings={:?}",
        opened.warnings
    );
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncated_journal_header_is_not_overwritten() {
    use motolii_doc::journal::{
        read_or_create_header, JournalFormatError, StdFs, HEADER_LEN,
    };
    use uuid::Uuid;

    let dir = unique_dir("trunc-header");
    let journal = dir.join(".motolii/journal.wal");
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
    assert_eq!(fs::read(&journal).unwrap(), before, "must not overwrite short wal");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn marker_remount_does_not_prefer_stale_main_over_journal_edits() {
    let dir = unique_dir("marker-stale-main");
    let path = dir.join("proj.json");
    save_project_with_journal(&path, &Document::new_v1(), &SaveProjectOptions::default()).unwrap();

    let mut edited = Document::new_v1();
    edited.bpm = Bpm::try_new(140, 1).unwrap();
    save_project_with_journal(
        &path,
        &edited,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 140, den: 1 }),
            checkpoint: false,
            ..Default::default()
        },
    )
    .unwrap();

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
    let marker_path = path.parent().unwrap().join(".motolii/restore_attempted.json");
    fs::write(&marker_path, serde_json::to_vec_pretty(&marker).unwrap()).unwrap();

    assert_eq!(
        motolii_doc::load_document(&path).unwrap().bpm,
        Bpm::try_new(120, 1).unwrap(),
        "precondition: main must stay stale"
    );

    let opened = open_project(&path).unwrap();
    assert_eq!(
        opened.document.bpm,
        Bpm::try_new(140, 1).unwrap(),
        "marker remount must replay committed edits, not return stale main"
    );
    assert!(
        opened.warnings.iter().any(|w| w.contains("restore_attempted")),
        "warnings={:?}",
        opened.warnings
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn d1c_save_document_alone_still_works() {
    let dir = unique_dir("d1c-compat");
    let path = dir.join("doc.json");
    let doc = Document::new_v1();
    save_document(&path, &doc).unwrap();
    assert_eq!(motolii_doc::load_document(&path).unwrap(), doc);
    let _ = fs::remove_dir_all(dir);
}
