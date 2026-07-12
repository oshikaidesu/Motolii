//! D1c: アトミック保存・読込(ガード2)と`min_reader_version`拒否(ガード7のI/O側)。
//!
//! 保存手順は SQLite atomic commit と同型: **一意temp**書き込み → file fsync → **置換** → dir fsync。
//! ジャーナル本体はD1d。本モジュールの abort 注入は「旧ファイルが残る/新ファイルが完全」を機械判定する。
//!
//! ## 耐久性契約(プラットフォーム差)
//!
//! - **Unix**: tempの`fsync`に加え、親ディレクトリの`fsync`でディレクトリエントリの永続まで狙う。
//! - **非Unix(Windows等)**: ファイル内容の`fsync`と`replace_file`(既存を置換)まで。親ディレクトリの
//!   fsyncはOSが実質サポートしない/不要なため**省略**する。保証水準は
//!   「プロセスクラッシュ後に、完全な旧ファイルか完全な新ファイルのどちらかが見える」こと。
//!   電源断時のディレクトリメタデータ永続はファイルシステム依存。

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use thiserror::Error;

use crate::plugin_catalog::{collect_plugin_warnings, LoadResult, PluginCatalog};
use crate::{Document, DocumentError};

/// このリーダーが開ける`min_reader_version`の上限(=自版の読取能力)。
pub const READER_VERSION: u32 = 1;

/// クラッシュ注入用の保存段。本番は`None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAbortAfter {
    /// tempへ書き込んだ直後(fsync前)
    TempWrite,
    /// tempのfsync直後(replace前)
    TempFsync,
    /// 置換直後(dir fsync前)
    Rename,
}

#[derive(Debug, Error)]
pub enum PersistError {
    #[error(transparent)]
    Validate(#[from] DocumentError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(
        "document requires reader version {min_reader_version}, but this reader is {reader_version}"
    )]
    ReaderTooOld {
        min_reader_version: u32,
        reader_version: u32,
    },
    /// テスト用 abort 注入。本番経路では返さない。
    #[error("save aborted after {stage:?} at {temp_path} (injection)")]
    Aborted {
        stage: SaveAbortAfter,
        temp_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudSyncHint {
    None,
    /// パス成分から疑わしい同期フォルダを検出。警告口(UIはM3)。
    Suspected {
        provider: &'static str,
    },
}

/// オープン経路の口(ガード11)。検出のみ — 拒否しない。
pub fn detect_cloud_sync(path: &Path) -> CloudSyncHint {
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        if lower.contains("dropbox") {
            return CloudSyncHint::Suspected {
                provider: "Dropbox",
            };
        }
        if lower.contains("icloud") || lower == "mobile documents" {
            return CloudSyncHint::Suspected { provider: "iCloud" };
        }
        if lower.contains("google drive") || lower == "googledrive" {
            return CloudSyncHint::Suspected {
                provider: "Google Drive",
            };
        }
        if lower.contains("onedrive") {
            return CloudSyncHint::Suspected {
                provider: "OneDrive",
            };
        }
    }
    CloudSyncHint::None
}

/// 保存オプション。abort は単体テスト専用。
#[derive(Debug, Clone, Default)]
pub struct SaveOptions {
    pub abort_after: Option<SaveAbortAfter>,
}

/// 検証→一意temp→fsync→置換→dir fsync。
///
/// 失敗時に途中のtempは可能な範囲で掃除しない(注入テストが残骸を観察できるようにする)。
pub fn save_document(path: &Path, doc: &Document) -> Result<(), PersistError> {
    save_document_with_options(path, doc, &SaveOptions::default())
}

pub fn save_document_with_options(
    path: &Path,
    doc: &Document,
    options: &SaveOptions,
) -> Result<(), PersistError> {
    doc.validate()?;

    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let bytes = serde_json::to_vec_pretty(doc)?;
    let (temp_path, mut file) = create_unique_temp(parent, path)?;

    file.write_all(&bytes)?;
    file.flush()?;
    if options.abort_after == Some(SaveAbortAfter::TempWrite) {
        return Err(PersistError::Aborted {
            stage: SaveAbortAfter::TempWrite,
            temp_path,
        });
    }
    file.sync_all()?;
    // 以降はパスだけ使う(ハンドルを閉じてから置換)
    drop(file);

    if options.abort_after == Some(SaveAbortAfter::TempFsync) {
        return Err(PersistError::Aborted {
            stage: SaveAbortAfter::TempFsync,
            temp_path,
        });
    }

    replace_file(&temp_path, path)?;
    if options.abort_after == Some(SaveAbortAfter::Rename) {
        return Err(PersistError::Aborted {
            stage: SaveAbortAfter::Rename,
            temp_path: PathBuf::new(), // 置換済みでtempは消えている
        });
    }

    sync_dir(parent)?;
    Ok(())
}

/// 読込。`min_reader_version`超過はデシリアライズ前に拒否(ガード7)。
/// 未知`plugin_id`は拒否せず`LoadResult::warnings`へ(D1f / ガード9の開く側)。
/// クラウド同期検出は呼び出し側が`detect_cloud_sync`で参照する(ここでは拒否しない)。
pub fn load_document(path: &Path, catalog: &PluginCatalog) -> Result<LoadResult, PersistError> {
    let bytes = fs::read(path)?;
    load_document_bytes(&bytes, catalog)
}

pub fn load_document_bytes(
    bytes: &[u8],
    catalog: &PluginCatalog,
) -> Result<LoadResult, PersistError> {
    // 全文デシリアライズ前に版だけ読む — 未知フィールドで落ちる前に拒否するため
    let header: VersionHeader = serde_json::from_slice(bytes)?;
    if header.min_reader_version > READER_VERSION {
        return Err(PersistError::ReaderTooOld {
            min_reader_version: header.min_reader_version,
            reader_version: READER_VERSION,
        });
    }
    let doc: Document = serde_json::from_slice(bytes)?;
    doc.validate()?;
    let warnings = collect_plugin_warnings(&doc, catalog);
    Ok(LoadResult {
        document: doc,
        warnings,
    })
}

#[derive(Debug, Deserialize)]
struct VersionHeader {
    #[serde(default = "default_min_reader")]
    min_reader_version: u32,
}

fn default_min_reader() -> u32 {
    1
}

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// 同一ディレクトリ内に**一意**なtempを`create_new`で作る(並行保存で互いを上書きしない)。
fn create_unique_temp(parent: &Path, final_path: &Path) -> io::Result<(PathBuf, File)> {
    let stem = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.json");
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    for _ in 0..64 {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!(".{stem}.{pid}.{nanos}.{seq}.motolii-tmp");
        let path = parent.join(&name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "exhausted unique temp name attempts",
    ))
}

/// 同一ボリューム内で `from` を `to` に**置換**する(既存 `to` があっても成功する)。
///
/// - Unix: `rename(2)` — 既存を原子的に置き換え
/// - Windows: `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)` —
///   `std::fs::rename` に頼らず置換フラグを明示(既存ファイルへの2回目以降保存のため)
fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        fs::rename(from, to)
    }
    #[cfg(windows)]
    {
        replace_file_windows(from, to)
    }
    #[cfg(not(any(unix, windows)))]
    {
        // その他: ベストエフォートで rename(既存置換の可否はOS依存)
        fs::rename(from, to)
    }
}

#[cfg(windows)]
fn replace_file_windows(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(
            lp_existing_file_name: *const u16,
            lp_new_file_name: *const u16,
            dw_flags: u32,
        ) -> i32;
    }

    // winbase.h
    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str().encode_wide().chain(Some(0)).collect()
    }

    let from_w = wide(from);
    let to_w = wide(to);
    // SAFETY: NUL終端の絶対/相対パス。同一ボリューム前提は呼び出し側(同dir temp)が保証。
    let ok = unsafe {
        MoveFileExW(
            from_w.as_ptr(),
            to_w.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let dir_file = File::open(dir)?;
        dir_file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        // モジュール先頭の耐久性契約を参照 — 非Unixではディレクトリfsyncを省略する。
        let _ = dir;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_catalog::PluginCatalog;
    use crate::Document;

    fn unique_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-d1c-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn roundtrip_save_load() {
        let dir = unique_dir();
        let path = dir.join("proj.json");
        let doc = Document::new_v1();
        save_document(&path, &doc).unwrap();
        let loaded = load_document(&path, &PluginCatalog::new()).unwrap();
        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.document, doc);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_min_reader_newer_than_us() {
        let json = r#"{
            "version": 1,
            "min_reader_version": 99,
            "composition": {
                "aspect_num": 16,
                "aspect_den": 9,
                "duration": {"num": 10, "den": 1},
                "fps": {"num": 30, "den": 1}
            },
            "bpm": {"num": 120, "den": 1}
        }"#;
        let err = load_document_bytes(json.as_bytes(), &PluginCatalog::new()).unwrap_err();
        assert!(matches!(
            err,
            PersistError::ReaderTooOld {
                min_reader_version: 99,
                reader_version: READER_VERSION
            }
        ));
    }

    #[test]
    fn detect_dropbox_path() {
        let hint = detect_cloud_sync(Path::new("/Users/x/Dropbox/projects/a.json"));
        assert_eq!(
            hint,
            CloudSyncHint::Suspected {
                provider: "Dropbox"
            }
        );
    }

    #[test]
    fn unique_temps_do_not_collide() {
        let dir = unique_dir();
        let final_path = dir.join("doc.json");
        let (a, _fa) = create_unique_temp(&dir, &final_path).unwrap();
        let (b, _fb) = create_unique_temp(&dir, &final_path).unwrap();
        assert_ne!(a, b);
        let _ = fs::remove_dir_all(dir);
    }
}
