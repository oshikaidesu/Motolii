//! D1c-FU(#101, 監査S14): `OpenMode`(ReadWrite / ReadOnlyNewer / Reject)の読込/保存可否。
//!
//! 「未知ネストを読めたこと」と「再保存可能」を同一視しない — `ReadOnlyNewer`は読めても
//! save/migrationは型付きエラーで拒否する。`Reject`はDocumentを一切返さない。

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    check_migration_allowed, classify_open_mode, load_document_bytes_with_limits,
    load_document_with_limits, save_document, Document, OpenMode, PersistError, ResourceLimits,
    READER_VERSION, WRITER_VERSION,
};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1c-fu-openmode-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn minimal_json(version: u32, min_reader_version: u32) -> String {
    format!(
        r#"{{
            "version": {version},
            "min_reader_version": {min_reader_version},
            "composition": {{
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {{"num": 10, "den": 1}},
                "fps": {{"num": 30, "den": 1}}
            }},
            "bpm": {{"num": 120, "den": 1}}
        }}"#
    )
}

// --- classify_open_mode(純判定) ---

#[test]
fn classify_read_write_when_within_both_bounds() {
    assert_eq!(
        classify_open_mode(WRITER_VERSION, READER_VERSION),
        OpenMode::ReadWrite
    );
    assert_eq!(classify_open_mode(1, 1), OpenMode::ReadWrite);
}

#[test]
fn classify_read_only_newer_when_version_exceeds_writer() {
    assert_eq!(
        classify_open_mode(WRITER_VERSION + 1, READER_VERSION),
        OpenMode::ReadOnlyNewer
    );
}

#[test]
fn classify_reject_when_min_reader_exceeds_reader() {
    assert_eq!(
        classify_open_mode(WRITER_VERSION, READER_VERSION + 1),
        OpenMode::Reject
    );
    // versionが未来でもmin_reader_version超過はRejectが優先(読めないものは読めない)
    assert_eq!(
        classify_open_mode(WRITER_VERSION + 5, READER_VERSION + 1),
        OpenMode::Reject
    );
}

// --- ReadWrite: 読込・保存いずれも可能 ---

#[test]
fn read_write_loads_and_saves() {
    let dir = unique_dir("rw");
    let path = dir.join("doc.json");
    fs::write(&path, minimal_json(1, 1)).unwrap();

    let opened = load_document_with_limits(&path, &ResourceLimits::production()).unwrap();
    assert_eq!(opened.open_mode, OpenMode::ReadWrite);

    check_migration_allowed(&opened.document).expect("ReadWrite must allow migration gate");
    save_document(&path, &opened.document).expect("ReadWrite must allow save");
    let _ = fs::remove_dir_all(dir);
}

// --- ReadOnlyNewer: 読めるが save/migration は型付きエラー ---

#[test]
fn read_only_newer_loads_but_refuses_save() {
    let dir = unique_dir("ronewer");
    let path = dir.join("doc.json");
    let newer_version = WRITER_VERSION + 1;
    fs::write(&path, minimal_json(newer_version, 1)).unwrap();

    let opened = load_document_with_limits(&path, &ResourceLimits::production())
        .expect("ReadOnlyNewer must still return a Document");
    assert_eq!(opened.open_mode, OpenMode::ReadOnlyNewer);
    assert_eq!(opened.document.version, newer_version);

    let save_err = save_document(&path, &opened.document).unwrap_err();
    assert!(
        matches!(
            save_err,
            PersistError::SaveRejectedReadOnlyNewer {
                document_version,
                writer_version
            } if document_version == newer_version && writer_version == WRITER_VERSION
        ),
        "unexpected error: {save_err:?}"
    );

    let migrate_err = check_migration_allowed(&opened.document).unwrap_err();
    assert!(
        matches!(
            migrate_err,
            PersistError::SaveRejectedReadOnlyNewer {
                document_version,
                writer_version
            } if document_version == newer_version && writer_version == WRITER_VERSION
        ),
        "unexpected error: {migrate_err:?}"
    );

    // 元ファイルは触れられていないこと(拒否は書込前に効く)
    let untouched = fs::read_to_string(&path).unwrap();
    assert!(untouched.contains(&format!("\"version\": {newer_version}")));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn read_only_newer_bytes_variant_matches_open_mode() {
    let bytes = minimal_json(WRITER_VERSION + 3, 1).into_bytes();
    let opened = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap();
    assert_eq!(opened.open_mode, OpenMode::ReadOnlyNewer);
}

// --- Reject: Documentを一切返さない ---

#[test]
fn reject_never_returns_a_document_from_path() {
    let dir = unique_dir("reject");
    let path = dir.join("doc.json");
    fs::write(&path, minimal_json(1, READER_VERSION + 1)).unwrap();

    let err = load_document_with_limits(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(
        err,
        PersistError::ReaderTooOld {
            min_reader_version,
            reader_version: READER_VERSION
        } if min_reader_version == READER_VERSION + 1
    ));
    // Err型なのでこの時点でDocument/OpenedDocumentのインスタンスは型上存在しない。
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn reject_never_returns_a_document_from_bytes() {
    let bytes = minimal_json(1, READER_VERSION + 1).into_bytes();
    let err = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, PersistError::ReaderTooOld { .. }));
}

#[test]
fn reject_even_when_document_version_also_newer() {
    // min_reader_version超過が最優先(未来versionでも読めないものは読めない)
    let bytes = minimal_json(WRITER_VERSION + 9, READER_VERSION + 1).into_bytes();
    let err = load_document_bytes_with_limits(&bytes, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, PersistError::ReaderTooOld { .. }));
}

// --- 新規作成ドキュメントは常にReadWrite(既存動作の回帰なし) ---

#[test]
fn freshly_created_document_is_always_read_write() {
    let doc = Document::new_v1();
    assert_eq!(
        classify_open_mode(doc.version, doc.min_reader_version),
        OpenMode::ReadWrite
    );
    check_migration_allowed(&doc).unwrap();
}
