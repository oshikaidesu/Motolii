//! D1m: non-blocking exclusive session lock (subprocess / drop / kill).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use motolii_doc::{Document, ProjectSession, ResourceLimits, SaveProjectOptions, SessionError};

pub mod common;

use common::session::{open_recovered, save_journal};

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("motolii-d1m-lock-{tag}-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn lock_holder_exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_d1m-lock-holder"))
}

fn spawn_lock_holder(project_path: &Path, hold_ms: u64) -> std::process::Child {
    Command::new(lock_holder_exe())
        .arg(project_path)
        .arg(hold_ms.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn lock holder")
}

#[test]
fn subprocess_holding_lock_rejects_second_open() {
    let dir = unique_dir("subprocess");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let mut child = spawn_lock_holder(&path, 5_000);
    std::thread::sleep(Duration::from_millis(500));

    let err = ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err();
    assert!(
        matches!(err, SessionError::ProjectAlreadyOpen),
        "expected ProjectAlreadyOpen, got {err:?}"
    );
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn drop_releases_lock_for_new_session() {
    let dir = unique_dir("drop");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    {
        let (_session, _) = open_recovered(&path);
    }
    let (_session, opened) = open_recovered(&path);
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn forced_subprocess_termination_releases_lock() {
    let dir = unique_dir("kill");
    let path = dir.join("proj.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );

    let mut child = spawn_lock_holder(&path, 60_000);
    std::thread::sleep(Duration::from_millis(500));
    assert!(matches!(
        ProjectSession::open(&path, &ResourceLimits::production()).unwrap_err(),
        SessionError::ProjectAlreadyOpen
    ));
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(Duration::from_millis(300));
    let (_session, opened) = open_recovered(&path);
    assert!(opened.document.validate().is_ok());
    let _ = fs::remove_dir_all(dir);
}

#[test]
#[cfg(unix)]
fn symlink_alias_shares_lock_identity() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("symlink");
    let path = dir.join("proj.json");
    let alias = dir.join("alias.json");
    save_journal(
        &path,
        &Document::new_current(),
        &SaveProjectOptions::default(),
    );
    symlink(&path, &alias).unwrap();

    let mut child = spawn_lock_holder(&path, 5_000);
    std::thread::sleep(Duration::from_millis(500));
    let err = ProjectSession::open(&alias, &ResourceLimits::production()).unwrap_err();
    assert!(matches!(err, SessionError::ProjectAlreadyOpen));
    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn lock_path_directory_fails_closed_without_steal() {
    use motolii_doc::project_lock_path_for_document;

    let dir = unique_dir("lock-dir");
    let path = dir.join("proj.json");
    fs::write(&path, b"{}").unwrap();
    let lock_path = project_lock_path_for_document(&path);
    fs::create_dir_all(&lock_path).unwrap();

    let err = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap_err();
    assert!(
        matches!(err, SessionError::Io(_)),
        "expected fail-closed I/O error, got {err:?}"
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn missing_parent_directory_fails_closed_without_creating_parent() {
    let dir = unique_dir("missing-parent");
    let missing_parent = dir.join("nonexistent-parent");
    let path = missing_parent.join("proj.json");

    let err = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap_err();
    assert!(
        matches!(err, SessionError::Io(_)),
        "expected fail-closed I/O error, got {err:?}"
    );
    assert!(!missing_parent.exists());
    assert!(!path.exists());
    let lock_path = motolii_doc::project_lock_path_for_document(&path);
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn broken_parent_symlink_canonicalize_fails_closed() {
    use std::os::unix::fs::symlink;

    let dir = unique_dir("broken-parent-symlink");
    let broken = dir.join("broken-link");
    symlink(dir.join("does-not-exist"), &broken).unwrap();
    let path = broken.join("proj.json");
    assert!(!path.exists());

    let err = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap_err();
    assert!(
        matches!(err, SessionError::Io(_)),
        "expected fail-closed I/O error, got {err:?}"
    );
    let lock_path = motolii_doc::project_lock_path_for_document(&path);
    assert!(!lock_path.exists());
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn acquire_succeeds_for_existing_parent_and_nonexistent_project() {
    let dir = unique_dir("parent-exists");
    let path = dir.join("new-proj.json");
    assert!(!path.exists());

    let session = ProjectSession::acquire(&path, &ResourceLimits::production()).unwrap();
    let lock_path = motolii_doc::project_lock_path_for_document(session.document_path());
    assert!(lock_path.exists());
    assert!(!motolii_doc::motolii_dir_for_document(&path).exists());
    drop(session);
    let _ = fs::remove_dir_all(dir);
}
