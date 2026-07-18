#![allow(deprecated)]

//! D1c: アトミック保存の各段 abort 注入と min_reader / roundtrip / 競合・再保存。
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    detect_cloud_sync, load_document, Bpm, CloudSyncHint, Document, PersistError, SaveAbortAfter,
    SaveOptions, READER_VERSION,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1c-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn count_motolii_tmps(dir: &Path) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".motolii-tmp"))
        .count()
}

#[test]
fn save_load_roundtrip_preserves_document() {
    let dir = unique_dir("roundtrip");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    doc.bpm = Bpm::try_new(140, 1).unwrap();
    common::session::save_document_via_session(&path, &doc);
    assert_eq!(load_document(&path).unwrap(), doc);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn overwrite_existing_file_succeeds() {
    // 対象OSでの既存ファイル再保存(Windows置換API / Unix rename の回帰)
    let dir = unique_dir("overwrite");
    let path = dir.join("doc.json");

    let mut first = Document::new_v1();
    first.bpm = Bpm::try_new(100, 1).unwrap();
    common::session::save_document_via_session(&path, &first);
    assert_eq!(load_document(&path).unwrap(), first);

    let mut second = Document::new_v1();
    second.bpm = Bpm::try_new(160, 1).unwrap();
    common::session::save_document_via_session(&path, &second);
    assert_eq!(load_document(&path).unwrap(), second);

    let mut third = Document::new_v1();
    third.bpm = Bpm::try_new(180, 1).unwrap();
    common::session::save_document_via_session(&path, &third);
    assert_eq!(load_document(&path).unwrap(), third);
    assert_eq!(count_motolii_tmps(&dir), 0);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn concurrent_saves_leave_one_complete_document() {
    // 固定temp名だと互いのtempを上書きし、呼び出しと保存内容が食い違う。
    // 一意 create_new + 原子的置換なら、最終ファイルは常に完全なDocumentのどれか。
    let dir = unique_dir("concurrent");
    let path = Arc::new(dir.join("doc.json"));
    common::session::save_document_via_session(&path, &Document::new_v1());

    const N: usize = 8;
    let barrier = Arc::new(Barrier::new(N));
    let mut handles = Vec::with_capacity(N);
    for i in 0..N {
        let path = Arc::clone(&path);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            let mut doc = Document::new_v1();
            // bpm は既約正 — スレッドごとに一意な完全ドキュメント
            doc.bpm = Bpm::try_new(100 + i as i64, 1).unwrap();
            barrier.wait();
            common::session::save_document_via_session(&path, &doc);
            doc
        }));
    }
    let written: Vec<Document> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    let loaded = load_document(&path).unwrap();
    assert!(
        written.iter().any(|d| d == &loaded),
        "final file must equal one of the concurrent saves, got bpm={:?}",
        loaded.bpm
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn abort_after_temp_write_keeps_old_file() {
    let dir = unique_dir("abort-write");
    let path = dir.join("doc.json");
    let old = Document::new_v1();
    common::session::save_document_via_session(&path, &old);
    let old_bytes = fs::read(&path).unwrap();

    let mut newer = Document::new_v1();
    newer.bpm = Bpm::try_new(200, 1).unwrap();
    let err = common::session::save_document_via_session_with_options(
        &path,
        &newer,
        &SaveOptions {
            abort_after: Some(SaveAbortAfter::TempWrite),
        },
    )
    .unwrap_err();
    let PersistError::Aborted { stage, temp_path } = err else {
        panic!("expected Aborted, got {err:?}");
    };
    assert_eq!(stage, SaveAbortAfter::TempWrite);
    assert!(temp_path.exists());
    assert!(temp_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .ends_with(".motolii-tmp"));

    assert_eq!(fs::read(&path).unwrap(), old_bytes);
    assert_eq!(load_document(&path).unwrap(), old);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn abort_after_temp_fsync_keeps_old_file() {
    let dir = unique_dir("abort-fsync");
    let path = dir.join("doc.json");
    let old = Document::new_v1();
    common::session::save_document_via_session(&path, &old);

    let mut newer = Document::new_v1();
    newer.bpm = Bpm::try_new(88, 1).unwrap();
    let err = common::session::save_document_via_session_with_options(
        &path,
        &newer,
        &SaveOptions {
            abort_after: Some(SaveAbortAfter::TempFsync),
        },
    )
    .unwrap_err();
    let PersistError::Aborted { stage, temp_path } = err else {
        panic!("expected Aborted, got {err:?}");
    };
    assert_eq!(stage, SaveAbortAfter::TempFsync);
    assert!(temp_path.exists());
    assert_eq!(load_document(&path).unwrap(), old);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn abort_after_rename_new_file_is_complete() {
    let dir = unique_dir("abort-rename");
    let path = dir.join("doc.json");
    let old = Document::new_v1();
    common::session::save_document_via_session(&path, &old);

    let mut newer = Document::new_v1();
    newer.bpm = Bpm::try_new(99, 1).unwrap();
    let err = common::session::save_document_via_session_with_options(
        &path,
        &newer,
        &SaveOptions {
            abort_after: Some(SaveAbortAfter::Rename),
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        PersistError::Aborted {
            stage: SaveAbortAfter::Rename,
            ..
        }
    ));

    // 置換済みなので本ファイルは新内容で完全に読める
    assert_eq!(load_document(&path).unwrap(), newer);
    assert_eq!(count_motolii_tmps(&dir), 0);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn abort_temps_do_not_block_subsequent_save() {
    // 固定名だと abort 残骸が次の truncate と衝突する。一意名なら後続が通る。
    let dir = unique_dir("abort-then-save");
    let path = dir.join("doc.json");
    common::session::save_document_via_session(&path, &Document::new_v1());

    let mut mid = Document::new_v1();
    mid.bpm = Bpm::try_new(50, 1).unwrap();
    let _ = common::session::save_document_via_session_with_options(
        &path,
        &mid,
        &SaveOptions {
            abort_after: Some(SaveAbortAfter::TempFsync),
        },
    )
    .unwrap_err();
    assert!(count_motolii_tmps(&dir) >= 1);

    let mut final_doc = Document::new_v1();
    final_doc.bpm = Bpm::try_new(70, 1).unwrap();
    common::session::save_document_via_session(&path, &final_doc);
    assert_eq!(load_document(&path).unwrap(), final_doc);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn save_rejects_invalid_document() {
    let dir = unique_dir("invalid");
    let path = dir.join("doc.json");
    let mut doc = Document::new_v1();
    // version < min_reader_version は validate が型付き拒否する(方針1)。
    doc.version = 1;
    doc.min_reader_version = 2;
    let err = common::session::save_document_via_session_with_options(
        &path,
        &doc,
        &SaveOptions::default(),
    )
    .unwrap_err();
    assert!(matches!(err, PersistError::Validate(_)));
    assert!(!path.exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn load_rejects_reader_too_old() {
    let dir = unique_dir("reader");
    let path = dir.join("future.json");
    let json = format!(
        r#"{{
            "version":1,
            "min_reader_version":{},
            "composition":{{
                "aspect_num":16,"aspect_den":9,
                "duration":{{"num":10,"den":1}},
                "fps":{{"num":30,"den":1}}
            }},
            "bpm":{{"num":120,"den":1}}
        }}"#,
        READER_VERSION + 1
    );
    fs::write(&path, json).unwrap();
    let err = load_document(&path).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version: m,
            reader_version: READER_VERSION
        } if m == READER_VERSION + 1
    ));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn cloud_sync_hint_on_open_path() {
    assert_eq!(
        detect_cloud_sync(std::path::Path::new("/tmp/local/a.json")),
        CloudSyncHint::None
    );
    assert_eq!(
        detect_cloud_sync(std::path::Path::new(
            "/Users/a/Library/Mobile Documents/com~apple~CloudDocs/x.json"
        )),
        CloudSyncHint::Suspected { provider: "iCloud" }
    );
}
