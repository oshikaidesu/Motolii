//! D1d fault-injection acceptance (crate-private unit tests; not journal-public).

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::journal::format::{journal_path_for_document, scan_journal, JournalScanStop};
use crate::journal::fs::{DurabilityStage, FaultPlan};
use crate::journal::project::{
    checkpoint_with_fault_plan, inject_bad_checksum_at_last_frame, inject_corrupt_journal_tail,
    inject_salt_mismatch_frame, inject_unapplicable_committed_edit, open_project_with_limits,
    save_project_with_journal, ProjectError, SaveProjectOptions,
};
use crate::journal::replay::JournalEdit;
use crate::{
    Bpm, Command, DocParam, Document, LayerId, RecoverySource, ResourceLimits, ScalarPropertyId,
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

fn save_journal(path: &PathBuf, doc: &Document, options: &SaveProjectOptions) {
    save_project_with_journal(path, doc, options).expect("save with journal");
}

fn open_recovered(path: &PathBuf) -> crate::journal::RecoveryResult {
    open_project_with_limits(path, &ResourceLimits::production()).expect("open project")
}

fn set_opacity_cmd(layer: LayerId, old: f64, new: f64) -> JournalEdit {
    JournalEdit::new(Command::SetProperty {
        target: layer,
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(old),
        new_value: DocParam::const_f64(new),
    })
}

#[test]
fn corrupt_tail_is_ignored_without_truncating_journal() {
    let dir = unique_dir("partial-tail");
    let path = dir.join("proj.json");
    let doc = Document::new_current();
    save_journal(&path, &doc, &SaveProjectOptions::default());

    let journal = journal_path_for_document(&path);
    let before = fs::metadata(&journal).unwrap().len();
    inject_corrupt_journal_tail(&path, b"GARBAGE_PARTIAL_WRITE").unwrap();
    let after = fs::metadata(&journal).unwrap().len();
    assert!(after > before, "tail garbage must extend journal");

    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::PartialFrame));
    assert_eq!(scan.ignored_tail_bytes(), after - scan.valid_bytes);

    let opened = open_recovered(&path);
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
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let journal = journal_path_for_document(&path);
    let len_before = fs::metadata(&journal).unwrap().len();

    inject_bad_checksum_at_last_frame(&path).unwrap();
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::ChecksumMismatch));
    let opened = open_recovered(&path);
    assert_eq!(fs::metadata(&journal).unwrap().len(), len_before);
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn salt_mismatch_stops_scan_without_truncate() {
    let dir = unique_dir("salt");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let journal = journal_path_for_document(&path);

    inject_salt_mismatch_frame(&path).unwrap();
    let len_after = fs::metadata(&journal).unwrap().len();
    let scan = scan_journal(&journal, &Default::default()).unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::SaltMismatch));
    let opened = open_recovered(&path);
    assert_eq!(fs::metadata(&journal).unwrap().len(), len_after);
    assert!(opened.ignored_tail_bytes > 0);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn replay_failure_falls_back_to_snapshot() {
    let dir = unique_dir("replay-fail");
    let path = dir.join("proj.json");
    let mut doc = Document::new_current();
    doc.bpm = Bpm::try_new(100, 1).unwrap();
    save_journal(&path, &doc, &SaveProjectOptions::default());

    // 適用不能 Command を通常の durable envelope で commit(テスト専用 variant は載せない)
    inject_unapplicable_committed_edit(&path, &ResourceLimits::production()).unwrap();

    let opened = open_recovered(&path);
    assert_eq!(opened.source, RecoverySource::SnapshotFallback);
    assert_eq!(opened.document.bpm, Bpm::try_new(100, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn fault_enospace_on_journal_append() {
    let dir = unique_dir("enospc");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let err = checkpoint_with_fault_plan(
        &path,
        &Document::new_current(),
        &SaveProjectOptions {
            journal_edit: Some(set_opacity_cmd(LayerId::from_raw(1), 1.0, 0.5)),
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
    let old = Document::new_current();
    save_journal(&path, &old, &SaveProjectOptions::default());
    let old_bytes = fs::read(&path).unwrap();

    let mut newer = Document::new_current();
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
