//! D1m: project-scoped sidecar identity + inter-process read-write session lock.

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::limits::ResourceLimits;
use crate::migrate::{MigrateError, MigrateFileOptions, MigrateFileResult};
use crate::persist::SaveOptions;
use crate::{Document, PersistError};

use super::catalog::{CATALOG_FILENAME, GENERATIONS_DIR};
use super::format::JournalFormatError;
use super::fs::StdFs;
use super::project::{save_project_with_journal_fs, OpenProjectOutcome, SaveProjectOptions};
use super::recover::{
    recover_project, verify_sidecar_family_at_root, RecoveryError, RESTORE_ATTEMPTED_FILENAME,
};

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("project is already open in another process")]
    ProjectAlreadyOpen,
    #[error("legacy sidecar requires explicit migration")]
    LegacySidecarRequiresExplicitMigration,
    #[error("incomplete legacy migration staging remains")]
    IncompleteLegacyMigration,
    #[error("destination path is occupied")]
    DestinationPathOccupied,
    #[error("invalid project sidecar")]
    InvalidProjectSidecar,
    #[error("no legacy sidecar to migrate")]
    NoLegacySidecar,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Format(#[from] JournalFormatError),
    #[error(transparent)]
    Recovery(#[from] RecoveryError),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Migrate(#[from] MigrateError),
}

/// In-process diagnostic report for explicit legacy migration (non-persistent, non-serde).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacySidecarMigrationReport {
    pub disposition: LegacySidecarMigrationDisposition,
    pub untouched_legacy_entries: Vec<std::ffi::OsString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacySidecarMigrationDisposition {
    Installed,
    AlreadyValid,
}

impl std::fmt::Debug for ProjectSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectSession")
            .field("document_path", &self.document_path)
            .finish_non_exhaustive()
    }
}

/// Non-`Clone` read-write session holding an OS exclusive lock on the project identity.
pub struct ProjectSession {
    document_path: PathBuf,
    lock_file: File,
    limits: ResourceLimits,
}

impl ProjectSession {
    pub fn document_path(&self) -> &Path {
        &self.document_path
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Canonicalize identity, open/create sibling lock, non-blocking exclusive `try_lock`.
    pub fn acquire(path: &Path, limits: &ResourceLimits) -> Result<Self, SessionError> {
        let document_path = canonicalize_project_identity(path)?;
        let lock_path = super::format::project_lock_path_for_document(&document_path);
        let mut lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)?;
        lock_file.try_lock().map_err(|e| match e {
            TryLockError::WouldBlock => SessionError::ProjectAlreadyOpen,
            TryLockError::Error(io) => SessionError::Io(io),
        })?;
        let _ = writeln!(lock_file, "pid={}", std::process::id());
        Ok(Self {
            document_path,
            lock_file,
            limits: *limits,
        })
    }

    /// Acquire lock, apply closed state table, then non-destructive recovery.
    pub fn open(
        path: &Path,
        limits: &ResourceLimits,
    ) -> Result<(Self, OpenProjectOutcome), SessionError> {
        let session = Self::acquire(path, limits)?;
        session.check_ordinary_open_state()?;
        let mut fs = StdFs;
        let recovered = recover_project(&mut fs, &session.document_path, limits)?;
        Ok((session, recovered))
    }

    pub fn save_document(
        &mut self,
        doc: &Document,
        options: &SaveOptions,
    ) -> Result<(), PersistError> {
        crate::persist::save_document_with_options(&self.document_path, doc, options)
    }

    pub fn save_with_journal(
        &mut self,
        doc: &Document,
        options: &SaveProjectOptions,
    ) -> Result<(), Box<super::project::ProjectError>> {
        let mut fs = StdFs;
        save_project_with_journal_fs(&mut fs, &self.document_path, doc, options).map_err(Box::new)
    }

    pub fn migrate_document_file(
        &mut self,
        options: &MigrateFileOptions,
    ) -> Result<MigrateFileResult, MigrateError> {
        crate::migrate::migrate_document_file_with_limits(
            &self.document_path,
            options,
            &self.limits,
        )
    }

    /// Explicit legacy adoption: copy known journal family → verify → atomic install.
    pub fn migrate_legacy_sidecar(&mut self) -> Result<LegacySidecarMigrationReport, SessionError> {
        let legacy_dir = super::format::legacy_shared_motolii_dir_for_document(&self.document_path);
        let final_dir = super::format::motolii_dir_for_document(&self.document_path);
        let staging_dir = super::format::legacy_staging_dir_for_document(&self.document_path);

        // 1. final destination classification (always first)
        match classify_final_destination(&final_dir, &self.limits)? {
            FinalDestination::ValidFamily => {
                verify_project_sidecar_family(&final_dir, &self.limits)?;
                let untouched = preflight_untouched_legacy_entries(&legacy_dir)?;
                return Ok(LegacySidecarMigrationReport {
                    disposition: LegacySidecarMigrationDisposition::AlreadyValid,
                    untouched_legacy_entries: untouched,
                });
            }
            FinalDestination::InvalidFamily => return Err(SessionError::InvalidProjectSidecar),
            FinalDestination::Occupied => return Err(SessionError::DestinationPathOccupied),
            FinalDestination::Absent | FinalDestination::EmptyDirectory => {}
        }

        // 2. diagnostic preflight (before any mutation)
        let untouched = preflight_untouched_legacy_entries(&legacy_dir)?;

        // 3. active staging quarantine
        if staging_dir.exists() {
            quarantine_staging_dir(&staging_dir)?;
        }

        // 4. legacy family required for install
        if !journal_family_exists_at(&legacy_dir)? {
            return Err(SessionError::NoLegacySidecar);
        }

        copy_journal_family(&legacy_dir, &staging_dir)?;
        verify_project_sidecar_family(&staging_dir, &self.limits)?;
        sync_dir_all(&staging_dir)?;
        if final_dir.exists() && is_empty_dir(&final_dir)? {
            fs::remove_dir(&final_dir)?;
        }
        fs::rename(&staging_dir, &final_dir)?;
        if let Some(parent) = final_dir.parent() {
            sync_dir(parent)?;
        }
        Ok(LegacySidecarMigrationReport {
            disposition: LegacySidecarMigrationDisposition::Installed,
            untouched_legacy_entries: untouched,
        })
    }

    fn check_ordinary_open_state(&self) -> Result<(), SessionError> {
        let legacy_dir = super::format::legacy_shared_motolii_dir_for_document(&self.document_path);
        let legacy_family = journal_family_exists_at(&legacy_dir)?;
        let final_dir = super::format::motolii_dir_for_document(&self.document_path);
        let staging_dir = super::format::legacy_staging_dir_for_document(&self.document_path);
        let active_staging = staging_dir.exists();
        let final_state = classify_final_destination(&final_dir, &self.limits)?;

        match final_state {
            FinalDestination::ValidFamily => Ok(()),
            FinalDestination::InvalidFamily => Err(SessionError::InvalidProjectSidecar),
            FinalDestination::Occupied => Err(SessionError::DestinationPathOccupied),
            FinalDestination::Absent | FinalDestination::EmptyDirectory => {
                if active_staging {
                    Err(SessionError::IncompleteLegacyMigration)
                } else if legacy_family {
                    Err(SessionError::LegacySidecarRequiresExplicitMigration)
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl Drop for ProjectSession {
    fn drop(&mut self) {
        let _ = self.lock_file.unlock();
    }
}

fn canonicalize_project_identity(path: &Path) -> Result<PathBuf, SessionError> {
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        SessionError::Io(std::io::Error::new(
            ErrorKind::InvalidInput,
            "project path has no file name",
        ))
    })?;
    if !parent.exists() {
        return Err(SessionError::Io(std::io::Error::new(
            ErrorKind::NotFound,
            "parent directory does not exist",
        )));
    }
    let canon_parent = fs::canonicalize(parent)?;
    Ok(canon_parent.join(file_name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinalDestination {
    Absent,
    EmptyDirectory,
    ValidFamily,
    InvalidFamily,
    Occupied,
}

fn is_regular_file_non_follow(path: &Path) -> Result<bool, SessionError> {
    match fs::symlink_metadata(path) {
        Ok(meta) => Ok(meta.is_file()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(SessionError::Io(e)),
    }
}

fn is_real_dir_non_follow(path: &Path) -> Result<bool, SessionError> {
    match fs::symlink_metadata(path) {
        Ok(meta) => Ok(meta.is_dir()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(SessionError::Io(e)),
    }
}

fn journal_family_exists_at(dir: &Path) -> Result<bool, SessionError> {
    if !dir.is_dir() {
        return Ok(false);
    }
    Ok(is_regular_file_non_follow(&dir.join("journal.wal"))?
        || is_regular_file_non_follow(&dir.join(CATALOG_FILENAME))?
        || is_real_dir_non_follow(&dir.join(GENERATIONS_DIR))?
        || is_regular_file_non_follow(&dir.join(RESTORE_ATTEMPTED_FILENAME))?
        || has_corrupt_journal_marker(dir)?)
}

fn os_str_starts_with(name: &OsStr, prefix: &str) -> bool {
    name.as_encoded_bytes().starts_with(prefix.as_bytes())
}

fn is_journal_family_copy_member(name: &OsStr) -> bool {
    name == "journal.wal"
        || name == CATALOG_FILENAME
        || name == GENERATIONS_DIR
        || name == RESTORE_ATTEMPTED_FILENAME
        || os_str_starts_with(name, "journal.wal.corrupt-")
}

fn has_corrupt_journal_marker(dir: &Path) -> Result<bool, SessionError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if os_str_starts_with(&entry.file_name(), "journal.wal.corrupt-")
            && entry.file_type()?.is_file()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn preflight_untouched_legacy_entries(
    legacy_dir: &Path,
) -> Result<Vec<std::ffi::OsString>, SessionError> {
    if !legacy_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut untouched = Vec::new();
    for entry in fs::read_dir(legacy_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if is_journal_family_copy_member(&name) {
            continue;
        }
        untouched.push(name);
    }
    untouched.sort();
    Ok(untouched)
}

fn classify_final_destination(
    dir: &Path,
    limits: &ResourceLimits,
) -> Result<FinalDestination, SessionError> {
    if !dir.exists() {
        return Ok(FinalDestination::Absent);
    }
    if !dir.is_dir() {
        return Ok(FinalDestination::Occupied);
    }
    if journal_family_exists_at(dir)? {
        return match verify_project_sidecar_family(dir, limits) {
            Ok(()) => Ok(FinalDestination::ValidFamily),
            Err(e @ SessionError::Recovery(RecoveryError::Persist(_))) => Err(e),
            Err(_) => Ok(FinalDestination::InvalidFamily),
        };
    }
    if is_empty_dir(dir)? {
        return Ok(FinalDestination::EmptyDirectory);
    }
    Ok(FinalDestination::Occupied)
}

fn is_empty_dir(dir: &Path) -> Result<bool, SessionError> {
    Ok(fs::read_dir(dir)?.next().is_none())
}

fn verify_project_sidecar_family(dir: &Path, limits: &ResourceLimits) -> Result<(), SessionError> {
    if !journal_family_exists_at(dir)? {
        return Err(SessionError::InvalidProjectSidecar);
    }
    let mut fs = super::fs::StdFs;
    verify_sidecar_family_at_root(&mut fs, dir, limits).map_err(map_verify_error)
}

fn map_verify_error(err: RecoveryError) -> SessionError {
    match err {
        RecoveryError::Fs(e) => SessionError::Recovery(RecoveryError::Fs(e)),
        RecoveryError::Format(e) => SessionError::Format(e),
        RecoveryError::Persist(e) => SessionError::Recovery(RecoveryError::Persist(e)),
        RecoveryError::Catalog(e) => SessionError::Recovery(RecoveryError::Catalog(e)),
        RecoveryError::Json(_) | RecoveryError::Unrecoverable { .. } => {
            SessionError::InvalidProjectSidecar
        }
    }
}

fn copy_journal_family(source: &Path, dest: &Path) -> Result<(), SessionError> {
    if dest.exists() {
        return Err(SessionError::DestinationPathOccupied);
    }
    fs::create_dir_all(dest)?;
    for name in ["journal.wal", CATALOG_FILENAME, RESTORE_ATTEMPTED_FILENAME] {
        let src = source.join(name);
        if is_regular_file_non_follow(&src)? {
            fs::copy(&src, dest.join(name))?;
        }
    }
    let src_gen = source.join(GENERATIONS_DIR);
    if is_real_dir_non_follow(&src_gen)? {
        copy_dir_recursive(&src_gen, &dest.join(GENERATIONS_DIR))?;
    }
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if os_str_starts_with(&name, "journal.wal.corrupt-") && entry.file_type()?.is_file() {
            fs::copy(entry.path(), dest.join(name))?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), SessionError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn quarantine_staging_dir(staging_dir: &Path) -> Result<(), SessionError> {
    let failed_name = format!(
        "{}.failed-{}",
        staging_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("staging"),
        Uuid::new_v4()
    );
    let parent = staging_dir
        .parent()
        .ok_or_else(|| SessionError::Io(std::io::Error::new(ErrorKind::NotFound, "no parent")))?;
    let quarantine = parent.join(failed_name);
    fs::rename(staging_dir, &quarantine)?;
    sync_dir(parent)?;
    Ok(())
}

fn sync_dir(path: &Path) -> Result<(), SessionError> {
    #[cfg(unix)]
    {
        let dir_file = File::open(path)?;
        dir_file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn os_str_starts_with_matches_corrupt_journal_prefix() {
        let name = OsStr::new("journal.wal.corrupt-20260101");
        assert!(os_str_starts_with(&name, "journal.wal.corrupt-"));
    }

    #[test]
    fn os_str_starts_with_rejects_non_matching_name() {
        let name = OsStr::new("journal.wal");
        assert!(!os_str_starts_with(&name, "journal.wal.corrupt-"));
    }

    #[test]
    fn os_str_starts_with_rejects_prefix_only_partial() {
        let name = OsStr::new("journal.wal.corrupt");
        assert!(!os_str_starts_with(&name, "journal.wal.corrupt-"));
    }

    #[cfg(unix)]
    #[test]
    fn os_str_starts_with_non_utf8_name_matches_prefix_lossless() {
        use std::os::unix::ffi::OsStrExt;
        let name = OsStr::from_bytes(b"journal.wal.corrupt-\xFFtail");
        assert!(os_str_starts_with(&name, "journal.wal.corrupt-"));
    }
}

fn sync_dir_all(dir: &Path) -> Result<(), SessionError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            sync_dir_all(&entry.path())?;
        } else if file_type.is_file() {
            let f = File::open(entry.path())?;
            f.sync_all()?;
        }
    }
    sync_dir(dir)?;
    Ok(())
}
