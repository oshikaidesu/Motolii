//! D1d: SQLite WAL 壊れ方カタログの単体/注入テスト。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    inject_bad_checksum_at_last_frame, inject_corrupt_catalog, inject_corrupt_journal_tail,
    inject_corrupt_main, inject_salt_mismatch_frame, load_catalog, load_document, open_project,
    save_document, save_project_with_journal, scan_journal, Bpm, Document, GenerationCatalog,
    JournalEdit, JournalScanStop, PinGenerationOptions, RecoverySource, RotateOptions,
    SaveProjectOptions, ScanJournalOptions,
};
use uuid::Uuid;

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1d-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn save_with_edit(path: &Path, doc: &Document, edit: JournalEdit) {
    save_project_with_journal(
        path,
        doc,
        &SaveProjectOptions {
            journal_edit: Some(edit),
            snapshot_every_n_edits: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
}

fn save_edit_only(path: &Path, doc: &Document, edit: JournalEdit) {
    save_project_with_journal(
        path,
        doc,
        &SaveProjectOptions {
            journal_edit: Some(edit),
            skip_snapshot: true,
            ..Default::default()
        },
    )
    .unwrap();
}

#[test]
fn journal_replay_applies_edits_after_snapshot() {
    let dir = unique_dir("replay");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            snapshot_every_n_edits: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    doc.bpm = Bpm::try_new(150, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 150, den: 1 });

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, Bpm::try_new(150, 1).unwrap());
    assert_eq!(opened.source, RecoverySource::JournalReplay);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncates_partial_tail_after_crash() {
    let dir = unique_dir("partial-tail");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 120, den: 1 });
    doc.bpm = Bpm::try_new(130, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 130, den: 1 });

    inject_corrupt_journal_tail(&path, b"GARBAGE_PARTIAL_WRITE").unwrap();

    let opened = open_project(&path).unwrap();
    assert!(opened.truncated_bytes >= 21);
    assert_eq!(opened.document.bpm, Bpm::try_new(130, 1).unwrap());
    assert_eq!(opened.source, RecoverySource::TruncatedJournalThenReplay);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncates_bad_checksum_keeps_newer_main() {
    // journal tip が切捨てで古くなっても、有効な main は巻き戻さない
    let dir = unique_dir("bad-checksum");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(110, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 110, den: 1 });
    doc.bpm = Bpm::try_new(111, 1).unwrap();
    save_edit_only(&path, &doc, JournalEdit::SetBpm { num: 111, den: 1 });

    inject_bad_checksum_at_last_frame(&path).unwrap();
    let opened = open_project(&path).unwrap();
    assert!(opened.truncated_bytes > 0);
    assert_eq!(opened.document.bpm, Bpm::try_new(111, 1).unwrap());
    assert_eq!(opened.source, RecoverySource::MainFile);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn truncates_salt_mismatch_tail() {
    let dir = unique_dir("salt-mismatch");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(105, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 105, den: 1 });
    inject_salt_mismatch_frame(&path).unwrap();

    let scan = scan_journal(
        &dir.join(".motolii/journal.wal"),
        &ScanJournalOptions::default(),
    )
    .unwrap();
    assert_eq!(scan.stopped, Some(JournalScanStop::SaltMismatch));

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, Bpm::try_new(105, 1).unwrap());
    assert!(opened.truncated_bytes > 0);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn replay_failure_falls_back_to_snapshot() {
    let dir = unique_dir("replay-fallback");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(100, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 100, den: 1 });

    save_edit_only(&path, &doc, JournalEdit::ForceReplayFail);

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.source, RecoverySource::SnapshotFallback);
    assert_eq!(opened.document.bpm, Bpm::try_new(100, 1).unwrap());
    let replay = opened.replay.expect("replay metadata");
    assert!(!replay.replay_failures.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn pinned_generation_survives_rotation() {
    let dir = unique_dir("pinned");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();

    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            snapshot_every_n_edits: Some(1),
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = load_catalog(&path).unwrap().expect("catalog");
    let pinned_id = catalog.generations[0].id;

    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 101, den: 1 }),
            snapshot_every_n_edits: Some(1),
            pin_generation: Some(PinGenerationOptions {
                generation_id: pinned_id,
            }),
            rotate: Some(RotateOptions {
                max_unpinned: Some(0),
            }),
            max_unpinned_generations: Some(0),
            ..Default::default()
        },
    )
    .unwrap();

    doc.bpm = Bpm::try_new(102, 1).unwrap();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 102, den: 1 }),
            snapshot_every_n_edits: Some(1),
            rotate: Some(RotateOptions {
                max_unpinned: Some(1),
            }),
            max_unpinned_generations: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let catalog = load_catalog(&path).unwrap().expect("catalog");
    assert!(
        catalog
            .generations
            .iter()
            .any(|g| g.id == pinned_id && g.pinned),
        "pinned generation must remain"
    );
    assert!(
        catalog
            .generations
            .iter()
            .any(|g| g.id != pinned_id && !g.pinned),
        "new unpinned generation expected"
    );
    assert!(
        dir.join(".motolii/generations")
            .join(format!("{pinned_id}.json"))
            .exists(),
        "pinned snapshot file must survive rotation"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn d1c_contract_unchanged_without_journal() {
    let dir = unique_dir("d1c-compat");
    let path = dir.join("doc.json");
    let doc = Document::new_v1();
    save_document(&path, &doc).unwrap();
    assert_eq!(load_document(&path).unwrap(), doc);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn uuid_cross_refs_link_snapshot_and_journal_record() {
    let dir = unique_dir("uuid-refs");
    let path = dir.join("doc.json");
    let doc = Document::new_v1();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            snapshot_every_n_edits: Some(1),
            ..Default::default()
        },
    )
    .unwrap();

    let catalog: GenerationCatalog = load_catalog(&path).unwrap().expect("catalog");
    let entry = &catalog.generations[0];
    assert_ne!(entry.id, Uuid::nil());
    assert_ne!(entry.journal_record, Uuid::nil());
    assert_ne!(entry.id, entry.journal_record);

    let scan = scan_journal(
        &dir.join(".motolii/journal.wal"),
        &ScanJournalOptions {
            verify_prev_chain: true,
        },
    )
    .unwrap();
    let snapshot_frame = scan
        .frames
        .iter()
        .find(|f| f.snapshot_ref == Some(entry.id))
        .expect("snapshot frame");
    assert_eq!(snapshot_frame.record_id, entry.journal_record);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn save_document_alone_does_not_rewind_on_open() {
    let dir = unique_dir("main-ahead");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(100, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 100, den: 1 });

    let mut newer = Document::new_v1();
    newer.bpm = Bpm::try_new(200, 1).unwrap();
    save_document(&path, &newer).unwrap();

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, Bpm::try_new(200, 1).unwrap());
    assert_eq!(opened.source, RecoverySource::MainFile);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn corrupt_main_falls_back_to_generation_replay() {
    let dir = unique_dir("corrupt-main");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(140, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 140, den: 1 });

    inject_corrupt_main(&path).unwrap();
    let opened = open_project(&path).unwrap();
    assert_eq!(opened.source, RecoverySource::GenerationRecovery);
    assert_eq!(opened.document.bpm, Bpm::try_new(140, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn snapshot_load_failure_falls_back_to_last_snapshot() {
    let dir = unique_dir("snap-fail");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(70, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 70, den: 1 });
    let first_gen = load_catalog(&path).unwrap().unwrap().generations[0].id;

    doc.bpm = Bpm::try_new(80, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 80, den: 1 });
    let second_gen = load_catalog(&path)
        .unwrap()
        .unwrap()
        .generations
        .iter()
        .find(|g| g.id != first_gen)
        .unwrap()
        .id;

    // 2つ目の世代ファイルを消して Snapshot フレームを失敗させる
    fs::remove_file(
        dir.join(".motolii/generations")
            .join(format!("{second_gen}.json")),
    )
    .unwrap();

    let opened = open_project(&path).unwrap();
    assert_eq!(opened.source, RecoverySource::SnapshotFallback);
    assert_eq!(opened.document.bpm, Bpm::try_new(70, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn edits_since_snapshot_persists_across_saves() {
    let dir = unique_dir("edit-counter");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            snapshot_every_n_edits: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        load_catalog(&path).unwrap().unwrap().edits_since_snapshot,
        0
    );

    doc.bpm = Bpm::try_new(11, 1).unwrap();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 11, den: 1 }),
            snapshot_every_n_edits: Some(2),
            skip_snapshot: false,
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = load_catalog(&path).unwrap().unwrap();
    assert_eq!(catalog.edits_since_snapshot, 1);
    let gens_after_one = catalog.generations.len();

    doc.bpm = Bpm::try_new(12, 1).unwrap();
    save_project_with_journal(
        &path,
        &doc,
        &SaveProjectOptions {
            journal_edit: Some(JournalEdit::SetBpm { num: 12, den: 1 }),
            snapshot_every_n_edits: Some(2),
            ..Default::default()
        },
    )
    .unwrap();
    let catalog = load_catalog(&path).unwrap().unwrap();
    assert_eq!(catalog.edits_since_snapshot, 0);
    assert!(catalog.generations.len() > gens_after_one);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn corrupt_catalog_does_not_block_open() {
    let dir = unique_dir("corrupt-catalog");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(55, 1).unwrap();
    save_with_edit(&path, &doc, JournalEdit::SetBpm { num: 55, den: 1 });

    inject_corrupt_catalog(&path).unwrap();
    let opened = open_project(&path).unwrap();
    assert_eq!(opened.document.bpm, Bpm::try_new(55, 1).unwrap());
    let _ = fs::remove_dir_all(dir);
}
