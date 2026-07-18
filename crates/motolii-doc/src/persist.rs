//! D1c: アトミック保存・読込(ガード2)と`min_reader_version`拒否(ガード7のI/O側)。
//! D1c-FU(#101): `ResourceLimits`(監査S10)注入と`OpenMode`(監査S14)。
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
//!
//! ## OpenMode(#101 / 監査S14)
//!
//! `min_reader_version`単独の合否(旧: Reject/OK二値)を、読み/書き互換を分離した3値へ拡張する。
//! 「未知ネストを読めたこと」と「再保存可能」を同一視しない — `Document.version`が自版の書き込み
//! 能力(`WRITER_VERSION`)より新しい場合は**読めるが再保存・migrationは拒否**する(黙って新フィールドを
//! 消して保存しないため)。
//!
//! - [`OpenMode::ReadWrite`]: `min_reader_version <= READER_VERSION` かつ `version <= WRITER_VERSION`
//! - [`OpenMode::ReadOnlyNewer`]: `min_reader_version <= READER_VERSION` だが `version > WRITER_VERSION`
//! - [`OpenMode::Reject`]: `min_reader_version > READER_VERSION`(Documentを返さない)

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use thiserror::Error;

use crate::limits::{check_document_resource_limits, ResourceLimitError, ResourceLimits};
use crate::{Document, DocumentError};

/// このリーダーが開ける`min_reader_version`の上限(=自版の読取能力)。
///
/// D1lでEffectDefinition/EffectUse共有schemaを追加したため4へ。
/// version 3以下(旧inline EffectInstance)はdefaultで読める。共有schemaを含む文書は
/// `min_reader_version>=4`を要求し、旧readerの再保存消失を防ぐ。
pub const READER_VERSION: u32 = 4;

/// このリーダーが**再保存・migrationしてよい**`Document.version`の上限(=自版の書込能力)。
/// D1lのEffectDefinition入り文書は`version=4`へ上がるため、書き込み能力も4へ揃える。
/// `Document.version`がこれを超える場合は`OpenMode::ReadOnlyNewer`(#101 / 監査S14)。
pub const WRITER_VERSION: u32 = 4;

const _: [(); 4] = [(); READER_VERSION as usize];
const _: [(); 4] = [(); WRITER_VERSION as usize];
const _: [(); READER_VERSION as usize] = [(); WRITER_VERSION as usize];
const _: [(); READER_VERSION as usize] =
    [(); crate::validate::MIN_READER_VERSION_FOR_EFFECT_DEFINITIONS as usize];

/// 読み/書き互換を分離した3状態(監査S14)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// 読込・再保存・migrationいずれも可能。
    ReadWrite,
    /// 読込は可能だが、自版より新しい`version`のため再保存・migrationは拒否(警告つき)。
    ReadOnlyNewer,
    /// `min_reader_version`超過。Documentを返さない。
    Reject,
}

/// `version`/`min_reader_version`から`OpenMode`を判定する(I/O副作用なし)。
pub fn classify_open_mode(document_version: u32, min_reader_version: u32) -> OpenMode {
    if min_reader_version > READER_VERSION {
        OpenMode::Reject
    } else if document_version > WRITER_VERSION {
        OpenMode::ReadOnlyNewer
    } else {
        OpenMode::ReadWrite
    }
}

/// 読込成功時の戻り値。`open_mode`は`ReadWrite`または`ReadOnlyNewer`のみ
/// (`Reject`は`load_*`がDocumentを返さず`Err`にする — S14「Documentを返さない」)。
#[derive(Debug, Clone, PartialEq)]
pub struct OpenedDocument {
    pub document: Document,
    pub open_mode: OpenMode,
}

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
    #[error(transparent)]
    ResourceLimit(#[from] ResourceLimitError),
    #[error(transparent)]
    Migrate(#[from] Box<crate::migrate::MigrateError>),
    #[error(
        "document requires reader version {min_reader_version}, but this reader is {reader_version}"
    )]
    ReaderTooOld {
        min_reader_version: u32,
        reader_version: u32,
    },
    /// S14: `OpenMode::ReadOnlyNewer`からの再保存・migration拒否(型付き警告)。
    #[error(
        "document version {document_version} is newer than this writer ({writer_version}); \
         opened as ReadOnlyNewer — save/migration refused to avoid silently dropping newer fields"
    )]
    SaveRejectedReadOnlyNewer {
        document_version: u32,
        writer_version: u32,
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

/// `OpenMode`を判定し、`ReadWrite`以外は型付きエラーで拒否する(S14)。
/// `ReadOnlyNewer`(自版より新しい`version`)・`Reject`(`min_reader_version`超過)からの
/// 再保存・migrationはここで止める — 「未知ネストを読めたこと」と「再保存可能」を同一視しない。
fn guard_open_mode_for_write(doc: &Document) -> Result<(), PersistError> {
    match classify_open_mode(doc.version, doc.min_reader_version) {
        OpenMode::ReadWrite => Ok(()),
        OpenMode::ReadOnlyNewer => Err(PersistError::SaveRejectedReadOnlyNewer {
            document_version: doc.version,
            writer_version: WRITER_VERSION,
        }),
        OpenMode::Reject => Err(PersistError::ReaderTooOld {
            min_reader_version: doc.min_reader_version,
            reader_version: READER_VERSION,
        }),
    }
}

/// D1e(migration本体)向けの型の席のみ(#101はD1e本体を実装しない)。
/// `OpenMode`ゲートだけをここで固定し、実際の変換ロジックはD1eが実装する。
pub fn check_migration_allowed(doc: &Document) -> Result<(), PersistError> {
    guard_open_mode_for_write(doc)
}

/// 検証→一意temp→fsync→置換→dir fsync。
///
/// 失敗時に途中のtempは可能な範囲で掃除しない(注入テストが残骸を観察できるようにする)。
pub(crate) fn save_document(path: &Path, doc: &Document) -> Result<(), PersistError> {
    save_document_with_options(path, doc, &SaveOptions::default())
}

pub(crate) fn save_document_with_options(
    path: &Path,
    doc: &Document,
    options: &SaveOptions,
) -> Result<(), PersistError> {
    // 構造不変条件(validate)を先に見る — OpenMode判定はversion整合を前提にするため。
    doc.validate()?;
    guard_open_mode_for_write(doc)?;

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

/// 読込。`min_reader_version`超過はデシリアライズ前に拒否(ガード7 / S14 `Reject`)。
/// クラウド同期検出は呼び出し側が`detect_cloud_sync`で参照する(ここでは拒否しない)。
/// resource limitsはproduction既定(`ResourceLimits::production`)を使う。境界を絞るテストは
/// [`load_document_with_limits`]を使うこと。
pub fn load_document(path: &Path) -> Result<Document, PersistError> {
    Ok(load_document_with_limits(path, &ResourceLimits::production())?.document)
}

pub fn load_document_bytes(bytes: &[u8]) -> Result<Document, PersistError> {
    Ok(load_document_bytes_with_limits(bytes, &ResourceLimits::production())?.document)
}

/// #101: `ResourceLimits`を注入できるファイル読込。`OpenMode::Reject`は`Err`のみ返し、
/// Documentを一切構築しない(S14「Documentを返さない」)。
///
/// ファイルbytes上限は**同一`File`ハンドル**から`max_file_bytes + 1`までのbounded readで
/// 強制する。`metadata`→`fs::read`の二段だと検査と読込の間に拡大・差し替えされ、上限超過分を
/// 確保してから拒否し得る(S10)。超過時は`FileBytes`を返し、超過分はメモリに載せない。
pub fn load_document_with_limits(
    path: &Path,
    limits: &ResourceLimits,
) -> Result<OpenedDocument, PersistError> {
    let bytes = read_file_bounded(path, limits)?;
    load_document_bytes_with_limits(&bytes, limits)
}

/// `max_file_bytes + 1`までしか読まない。超過なら全文を確保せず型付き拒否する。
fn read_file_bounded(path: &Path, limits: &ResourceLimits) -> Result<Vec<u8>, PersistError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    // takeは上限+1で打ち切る — 巨大ファイルでも確保量をlimit近傍に閉じる。
    Read::by_ref(&mut file)
        .take(limits.max_file_bytes.saturating_add(1))
        .read_to_end(&mut buf)?;
    limits.check_file_bytes(buf.len() as u64)?;
    Ok(buf)
}

/// #101: `ResourceLimits`を注入できるbytes読込。
pub fn load_document_bytes_with_limits(
    bytes: &[u8],
    limits: &ResourceLimits,
) -> Result<OpenedDocument, PersistError> {
    limits.check_file_bytes(bytes.len() as u64)?;
    // 全文デシリアライズ前に版だけ読む — 未知フィールドで落ちる前にOpenModeを判定するため
    let header: VersionHeader = serde_json::from_slice(bytes)?;
    let open_mode = classify_open_mode(header.version, header.min_reader_version);
    if let OpenMode::Reject = open_mode {
        return Err(PersistError::ReaderTooOld {
            min_reader_version: header.min_reader_version,
            reader_version: READER_VERSION,
        });
    }
    let doc: Document = serde_json::from_slice(bytes)?;
    // S10: Group深度・キー数などの資源上限をvalidateの再帰全走査より先に拒否する。
    check_document_resource_limits(&doc, limits)?;
    doc.validate()?;
    Ok(OpenedDocument {
        document: doc,
        open_mode,
    })
}

#[derive(Debug, Deserialize)]
struct VersionHeader {
    version: u32,
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
#[allow(deprecated)]
mod tests {
    use super::*;
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
        let loaded = load_document(&path).unwrap();
        assert_eq!(loaded, doc);
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
        let err = load_document_bytes(json.as_bytes()).unwrap_err();
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
