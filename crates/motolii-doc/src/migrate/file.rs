use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::limits::ResourceLimits;
use crate::persist::save_document;

use super::{
    migrate_bytes_with_limits, MigrateError, MigrateFileOptions, MigrateFileResult, BACKUP_SUFFIX,
};

/// ファイルをbackup後に現行スキーマへ書換える。dry_run/noopでは原本を触らない。
pub(crate) fn migrate_document_file_with_limits(
    path: &Path,
    options: &MigrateFileOptions,
    limits: &ResourceLimits,
) -> Result<MigrateFileResult, MigrateError> {
    // loadと同じbounded readを使う(別経路禁止)。
    let bytes = read_file_bounded(path, limits)?;
    let (doc, report) = migrate_bytes_with_limits(&bytes, limits)?;
    let migrated = report.did_migrate();
    let backup_path = pre_migrate_backup_path(path);

    if options.dry_run || !migrated {
        return Ok(MigrateFileResult {
            backup_path,
            report,
            migrated,
        });
    }

    // exists()+copy の TOCTOU を避け、create_new で排他作成してから内容を書く。
    // backup の fsync(+親dir fsync)が終わるまで save_document しない。
    let mut backup_file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&backup_path)
    {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
            return Err(MigrateError::BackupExists(backup_path));
        }
        Err(e) => return Err(MigrateError::Io(e)),
    };
    // 読んだ bytes を書く(再読込 TOCTOU も避ける)。失敗時は不完全 bak を消す。
    if let Err(e) = (|| -> io::Result<()> {
        backup_file.write_all(&bytes)?;
        backup_file.flush()?;
        backup_file.sync_all()?;
        Ok(())
    })() {
        let _ = fs::remove_file(&backup_path);
        return Err(MigrateError::Io(e));
    }
    drop(backup_file);

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if let Err(e) = sync_dir(parent) {
        // bak は残してよい(最後の既知良品)。原本は未書換。
        return Err(MigrateError::Io(e));
    }

    save_document(path, &doc)?;
    Ok(MigrateFileResult {
        backup_path,
        report,
        migrated,
    })
}

/// 原本ファイル名に `BACKUP_SUFFIX` をバイト列のまま付与する。
/// `to_str()` フォールバックで `document.json.bak` に化けるのを防ぐ。
fn pre_migrate_backup_path(path: &Path) -> PathBuf {
    let mut backup_name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| OsString::from("document.json"));
    backup_name.push(BACKUP_SUFFIX);
    path.with_file_name(backup_name)
}

/// persist.rs と同型: Unix は親ディレクトリ fsync、非Unix は省略。
fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let dir_file = File::open(dir)?;
        dir_file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
    }
    Ok(())
}

fn read_file_bounded(path: &Path, limits: &ResourceLimits) -> Result<Vec<u8>, MigrateError> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    Read::by_ref(&mut file)
        .take(limits.max_file_bytes.saturating_add(1))
        .read_to_end(&mut buf)?;
    limits.check_file_bytes(buf.len() as u64)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// 非UTF-8ファイル名でも原本名+suffixのbakになり、document.json.bakへフォールバックしない。
    /// (macOSは非UTF-8パスを作れないので、パス組み立てのみを検証する)
    #[cfg(unix)]
    #[test]
    fn pre_migrate_backup_path_preserves_non_utf8_file_name() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let name = OsStr::from_bytes(b"legacy\xff.json");
        let path = Path::new("/tmp").join(name);
        let bak = pre_migrate_backup_path(&path);
        let mut expected = name.to_os_string();
        expected.push(BACKUP_SUFFIX);
        assert_eq!(bak.file_name(), Some(expected.as_os_str()));
        assert_ne!(
            bak.file_name().and_then(|s| s.to_str()),
            Some(format!("document.json{BACKUP_SUFFIX}").as_str())
        );
    }
}
