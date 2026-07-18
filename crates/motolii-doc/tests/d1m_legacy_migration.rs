//! D1m: legacy sidecar state table + explicit migration.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motolii_doc::{
    journal_path_for_document, legacy_shared_motolii_dir_for_document,
    legacy_staging_dir_for_document, motolii_dir_for_document, Document,
    LegacySidecarMigrationDisposition, ProjectSession, ResourceLimits, SaveProjectOptions,
    SessionError,
};

pub mod common;

use common::session::{acquire_session, open_recovered, save_journal};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1m-legacy-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn ordinary_open_rejects_legacy_family_without_mutation() {
    let dir = unique_dir("legacy-reject");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("journal.wal"), b"SHORT").unwrap();
    let legacy_bytes = fs::read(legacy.join("journal.wal")).unwrap();

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(
        err,
        SessionError::LegacySidecarRequiresExplicitMigration
    ));
    assert_eq!(fs::read(legacy.join("journal.wal")).unwrap(), legacy_bytes);
    assert!(!motolii_dir_for_document(&path).exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn incomplete_staging_rejects_ordinary_open() {
    let dir = unique_dir("incomplete");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let staging = legacy_staging_dir_for_document(&path);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("journal.wal"), b"PARTIAL").unwrap();

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::IncompleteLegacyMigration));
    assert!(staging.exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn explicit_migration_preserves_source_and_installs_final() {
    let dir = unique_dir("migrate");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );
    assert!(legacy.exists());
    assert!(final_dir.join("journal.wal").exists());
    assert!(!legacy_staging_dir_for_document(&path).exists());

    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::AlreadyValid
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn migrate_without_legacy_returns_typed_error() {
    let dir = unique_dir("no-legacy");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::NoLegacySidecar));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn destination_unknown_only_is_occupied() {
    let dir = unique_dir("occupied");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let final_dir = motolii_dir_for_document(&path);
    fs::create_dir_all(&final_dir).unwrap();
    fs::write(final_dir.join("notes.txt"), b"unknown").unwrap();

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::DestinationPathOccupied));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn catalog_missing_journal_family_is_rejected() {
    let dir = unique_dir("catalog-missing");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let final_dir = motolii_dir_for_document(&path);
    fs::remove_file(final_dir.join("catalog.json")).unwrap();
    assert!(final_dir.join("journal.wal").exists());
    assert!(!final_dir.join("catalog.json").exists());

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::InvalidProjectSidecar));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn invalid_final_family_is_rejected() {
    let dir = unique_dir("invalid-final");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let final_dir = motolii_dir_for_document(&path);
    fs::create_dir_all(&final_dir).unwrap();
    fs::write(final_dir.join("journal.wal"), b"BAD").unwrap();

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::InvalidProjectSidecar));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn migrated_project_recovers_via_session_open() {
    let dir = unique_dir("recover");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::rename(motolii_dir_for_document(&path), &legacy).unwrap();

    let mut session = acquire_session(&path);
    session.migrate_legacy_sidecar().unwrap();
    drop(session);

    let (_session, opened) = open_recovered(&path);
    assert!(opened.document.validate().is_ok());
    assert!(journal_path_for_document(&path).exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn destination_non_directory_rejects_ordinary_open_and_migrate() {
    let dir = unique_dir("dest-file");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let final_path = motolii_dir_for_document(&path);
    fs::write(&final_path, b"not-a-directory").unwrap();
    let before = fs::read(&final_path).unwrap();

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::DestinationPathOccupied));
    assert_eq!(fs::read(&final_path).unwrap(), before);

    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("journal.wal"), b"SHORT").unwrap();
    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::DestinationPathOccupied));
    assert_eq!(fs::read(&final_path).unwrap(), before);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn explicit_migration_does_not_copy_legacy_media() {
    let dir = unique_dir("no-media-copy");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();
    fs::create_dir_all(legacy.join("media")).unwrap();
    fs::write(legacy.join("media").join("clip.bin"), b"media-bytes").unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );

    assert!(legacy.join("media").join("clip.bin").exists());
    assert!(!final_dir.join("media").exists());
    assert!(report.untouched_legacy_entries.iter().any(|n| n == "media"));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn valid_final_with_staging_uses_final_without_merging() {
    let dir = unique_dir("final-staging");
    let path = dir.join("proj.json");
    let mut doc = Document::new_current();
    doc.bpm = motolii_doc::Bpm::try_new(131, 1).unwrap();
    save_journal(&path, &doc, &SaveProjectOptions::default());
    let final_dir = motolii_dir_for_document(&path);
    let staging = legacy_staging_dir_for_document(&path);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("journal.wal"), b"PARTIAL-STAGING").unwrap();
    let staging_bytes = fs::read(staging.join("journal.wal")).unwrap();
    let final_journal_before = fs::read(final_dir.join("journal.wal")).unwrap();

    let (_session, opened) = open_recovered(&path);
    assert!(opened.document.validate().is_ok());
    assert_eq!(opened.document.bpm.num(), 131);
    assert_eq!(
        fs::read(final_dir.join("journal.wal")).unwrap(),
        final_journal_before
    );
    assert_eq!(
        fs::read(staging.join("journal.wal")).unwrap(),
        staging_bytes
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn explicit_migration_quarantines_incomplete_staging_before_retry() {
    let dir = unique_dir("quarantine-retry");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();

    let staging = legacy_staging_dir_for_document(&path);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("journal.wal"), b"PARTIAL").unwrap();
    let staging_bytes = fs::read(staging.join("journal.wal")).unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );
    assert!(!staging.exists());
    assert!(final_dir.join("journal.wal").exists());

    let parent = path.parent().unwrap();
    let quarantined = fs::read_dir(parent)
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.contains(".importing.failed-"))
        })
        .expect("quarantined staging");
    assert_eq!(
        fs::read(quarantined.path().join("journal.wal")).unwrap(),
        staging_bytes
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn valid_final_without_legacy_returns_already_valid_empty_report() {
    let dir = unique_dir("already-valid-no-legacy");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::AlreadyValid
    );
    assert!(report.untouched_legacy_entries.is_empty());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn valid_final_with_legacy_root_reports_untouched_entries() {
    let dir = unique_dir("already-valid-legacy");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(legacy.join("media")).unwrap();
    fs::write(legacy.join("notes.txt"), b"keep").unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::AlreadyValid
    );
    let names: Vec<_> = report
        .untouched_legacy_entries
        .iter()
        .map(|n| n.to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["media".to_string(), "notes.txt".to_string()]);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn installed_migration_reports_media_and_unknown_sorted() {
    let dir = unique_dir("untouched-sorted");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();
    fs::create_dir_all(legacy.join("media")).unwrap();
    fs::write(legacy.join("zzz-unknown.bin"), b"x").unwrap();
    fs::write(legacy.join("aaa-unknown.bin"), b"y").unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );
    let names: Vec<_> = report
        .untouched_legacy_entries
        .iter()
        .map(|n| n.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        vec![
            "aaa-unknown.bin".to_string(),
            "media".to_string(),
            "zzz-unknown.bin".to_string()
        ]
    );
    let _ = fs::remove_dir_all(dir);
}

// macOS/APFS rejects non-UTF-8 path bytes at the VFS layer; Linux CI covers lossless OsString.
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_untouched_entry_stays_os_string() {
    use std::ffi::OsStr;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;

    let dir = unique_dir("non-utf8");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();
    let non_utf = OsStr::from_bytes(b"\xFFentry.bin");
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(legacy.join(non_utf))
        .unwrap();
    f.write_all(b"x").unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );
    assert_eq!(report.untouched_legacy_entries.len(), 1);
    assert_eq!(report.untouched_legacy_entries[0], non_utf);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn migrate_without_legacy_and_invalid_final_rejects_before_legacy_check() {
    let dir = unique_dir("no-legacy-invalid");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let final_dir = motolii_dir_for_document(&path);
    fs::create_dir_all(&final_dir).unwrap();
    fs::write(final_dir.join("journal.wal"), b"BAD").unwrap();
    let wal_before = fs::read(final_dir.join("journal.wal")).unwrap();

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::InvalidProjectSidecar));
    assert_eq!(fs::read(final_dir.join("journal.wal")).unwrap(), wal_before);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn migrate_without_legacy_and_occupied_final_rejects_before_legacy_check() {
    let dir = unique_dir("no-legacy-occupied");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let final_dir = motolii_dir_for_document(&path);
    fs::create_dir_all(&final_dir).unwrap();
    fs::write(final_dir.join("notes.txt"), b"unknown").unwrap();
    let notes_before = fs::read(final_dir.join("notes.txt")).unwrap();

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::DestinationPathOccupied));
    assert_eq!(fs::read(final_dir.join("notes.txt")).unwrap(), notes_before);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn staging_without_legacy_quarantines_then_returns_no_legacy() {
    let dir = unique_dir("staging-no-legacy");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let staging = legacy_staging_dir_for_document(&path);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("journal.wal"), b"PARTIAL").unwrap();
    let staging_bytes = fs::read(staging.join("journal.wal")).unwrap();

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::NoLegacySidecar));
    assert!(!staging.exists());

    let parent = path.parent().unwrap();
    let quarantined = fs::read_dir(parent)
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.contains(".importing.failed-"))
        })
        .expect("quarantined staging");
    assert_eq!(
        fs::read(quarantined.path().join("journal.wal")).unwrap(),
        staging_bytes
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn explicit_migration_preserves_nested_generations_regular_files() {
    let dir = unique_dir("nested-gen");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();

    let rel = "generations/archive/deep/payload.bin";
    let file_bytes = b"nested-generation-payload";
    fs::create_dir_all(legacy.join("generations").join("archive").join("deep")).unwrap();
    fs::write(legacy.join(rel), file_bytes).unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );

    let final_file = final_dir.join(rel);
    assert!(
        final_file.is_file(),
        "nested regular file must exist in final generations"
    );
    assert_eq!(fs::read(&final_file).unwrap(), file_bytes);
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn explicit_migration_skips_generations_symlink_without_follow() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("symlink-skip");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();

    let target = legacy.join("symlink-target.bin");
    let target_bytes = b"symlink-target-bytes";
    fs::write(&target, target_bytes).unwrap();

    let gen_dir = legacy.join("generations");
    fs::create_dir_all(&gen_dir).unwrap();
    let link_name = "linked-gen";
    symlink(&target, gen_dir.join(link_name)).unwrap();
    let link_before = fs::read_link(gen_dir.join(link_name)).unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );

    let final_gen = final_dir.join("generations");
    assert!(!final_gen.join(link_name).exists());
    assert!(!final_gen.join("symlink-target.bin").exists());
    assert_eq!(fs::read_link(gen_dir.join(link_name)).unwrap(), link_before);
    assert_eq!(fs::read(&target).unwrap(), target_bytes);
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn top_level_journal_wal_symlink_does_not_establish_legacy_family() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("wal-symlink");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(&legacy).unwrap();

    let external = dir.join("external-journal.wal");
    let external_bytes = b"external-wal-payload";
    fs::write(&external, external_bytes).unwrap();
    symlink(&external, legacy.join("journal.wal")).unwrap();
    let link_before = fs::read_link(legacy.join("journal.wal")).unwrap();

    let open_result = ProjectSession::open(&path, &ResourceLimits::production());
    assert!(!matches!(
        open_result.as_ref(),
        Err(SessionError::LegacySidecarRequiresExplicitMigration)
    ));

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::NoLegacySidecar));
    assert!(!motolii_dir_for_document(&path).exists());
    assert!(!legacy_staging_dir_for_document(&path).exists());
    assert_eq!(fs::read(&external).unwrap(), external_bytes);
    assert_eq!(
        fs::read_link(legacy.join("journal.wal")).unwrap(),
        link_before
    );
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn top_level_generations_symlink_does_not_establish_legacy_family() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("gen-symlink");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(&legacy).unwrap();

    let external = dir.join("external-generations");
    fs::create_dir_all(external.join("archive")).unwrap();
    let nested_bytes = b"external-generation-payload";
    fs::write(external.join("archive").join("payload.bin"), nested_bytes).unwrap();
    symlink(&external, legacy.join("generations")).unwrap();
    let link_before = fs::read_link(legacy.join("generations")).unwrap();

    let open_result = ProjectSession::open(&path, &ResourceLimits::production());
    assert!(!matches!(
        open_result.as_ref(),
        Err(SessionError::LegacySidecarRequiresExplicitMigration)
    ));

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::NoLegacySidecar));
    assert!(!motolii_dir_for_document(&path).exists());
    assert!(!legacy_staging_dir_for_document(&path).exists());
    assert_eq!(
        fs::read(external.join("archive").join("payload.bin")).unwrap(),
        nested_bytes
    );
    assert_eq!(
        fs::read_link(legacy.join("generations")).unwrap(),
        link_before
    );
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn explicit_migration_syncs_readonly_copied_regular_files() {
    use std::os::unix::fs::PermissionsExt;

    let dir = unique_dir("readonly-fsync");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    let final_dir = motolii_dir_for_document(&path);
    fs::rename(&final_dir, &legacy).unwrap();

    let rel = "generations/archive/deep/payload.bin";
    let file_bytes = b"readonly-nested-payload";
    fs::create_dir_all(legacy.join("generations").join("archive").join("deep")).unwrap();
    fs::write(legacy.join(rel), file_bytes).unwrap();
    fs::set_permissions(legacy.join(rel), fs::Permissions::from_mode(0o444)).unwrap();
    let wal_before = fs::read(legacy.join("journal.wal")).unwrap();

    let mut session = acquire_session(&path);
    let report = session.migrate_legacy_sidecar().unwrap();
    assert_eq!(
        report.disposition,
        LegacySidecarMigrationDisposition::Installed
    );

    let final_file = final_dir.join(rel);
    assert!(final_file.is_file());
    assert_eq!(fs::read(&final_file).unwrap(), file_bytes);
    assert_eq!(fs::read(legacy.join("journal.wal")).unwrap(), wal_before);
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn preflight_read_dir_failure_leaves_filesystem_unchanged() {
    use std::os::unix::fs::PermissionsExt;

    let dir = unique_dir("readdir-fail");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let legacy = legacy_shared_motolii_dir_for_document(&path);
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("journal.wal"), b"SHORT").unwrap();
    let wal_before = fs::read(legacy.join("journal.wal")).unwrap();
    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o000)).unwrap();

    let mut session = acquire_session(&path);
    let err = session.migrate_legacy_sidecar().unwrap_err();
    assert!(matches!(err, SessionError::Io(_)));
    assert!(!motolii_dir_for_document(&path).exists());
    assert!(!legacy_staging_dir_for_document(&path).exists());

    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(fs::read(legacy.join("journal.wal")).unwrap(), wal_before);
    let _ = fs::remove_dir_all(dir);
}
