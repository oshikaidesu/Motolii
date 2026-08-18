//! 「最後に開いていた project」— **引数なし起動で続きを開く**ための user 設定。
//!
//! `docs/ux-check-first-ten-minutes.md` P5 は「保存→再起動→続きがそのまま開く」を
//! 約束しているのに、窓は毎回スタート画面から始まっていた
//! (外部診断 F-01: `docs/reviews/2026-08-18-external-ux-diagnosis.md`)。
//! 続きを開くには、**窓が閉じたあとも残る場所**に最後の project が要る。
//!
//! 置き場も保存方式も palette settings に合わせる — 家は
//! `~/Library/Application Support/Motolii/` の1つだけ、中身は version 付きの JSON、
//! 書き込みは `palette_settings::write_atomically`(temp→fsync→rename)。
//! **新しい保存方式は発明しない。**
//!
//! ## ここに無いもの
//!
//! - **recent files の一覧**。覚えるのは最後の1本だけで、履歴 UI は本レーンの外
//! - **autosave**。覚えるのは「どの project に座っていたか」であって中身ではない。
//!   保存は相変わらず明示保存(`Cmd+S`)だけが行う

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const LAST_PROJECT_VERSION: u32 = 1;
pub const LAST_PROJECT_FILE_NAME: &str = "last-project.json";
/// path 1本しか入らないので、palette より桁で小さい上限で足りる。
const MAX_FILE_BYTES: u64 = 64 * 1024;

/// 置き場の中身。**field の並びがそのまま JSON の形**である。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LastProject {
    version: u32,
    path: PathBuf,
}

/// 覚えている project。**まだ何も覚えていない(初回起動)は `Ok(None)`** で、
/// 失敗ではない。
///
/// 壊れている・知らない version は `Err` — 黙って `None` に丸めない。呼び手が
/// 帯へ理由を出せるようにするためで、palette settings の扱いと同じである。
pub fn load_last_project(path: &Path) -> Result<Option<PathBuf>, LastProjectError> {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(LastProjectError::Io(source)),
    };
    if metadata.len() > MAX_FILE_BYTES {
        return Err(LastProjectError::TooLarge);
    }
    let bytes = std::fs::read(path)?;
    let stored: LastProject = serde_json::from_slice(&bytes)?;
    if stored.version != LAST_PROJECT_VERSION {
        return Err(LastProjectError::UnsupportedVersion(stored.version));
    }
    if stored.path.as_os_str().is_empty() {
        return Err(LastProjectError::EmptyPath);
    }
    Ok(Some(stored.path))
}

/// いま座った project を覚える。**最後の1本だけ**を残す(履歴ではない)。
pub fn remember_last_project(path: &Path, project: &Path) -> Result<(), LastProjectError> {
    if project.as_os_str().is_empty() {
        return Err(LastProjectError::EmptyPath);
    }
    let bytes = serde_json::to_vec_pretty(&LastProject {
        version: LAST_PROJECT_VERSION,
        path: project.to_path_buf(),
    })?;
    crate::palette_settings::write_atomically(path, &bytes)?;
    Ok(())
}

/// 置き場の既定。palette settings / user keymap と**同じ家**に居る。
pub fn default_last_project_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/Application Support/Motolii")
            .join(LAST_PROJECT_FILE_NAME),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum LastProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid last-project JSON: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("unsupported last-project version {0}")]
    UnsupportedVersion(u32),
    #[error("last-project file is too large")]
    TooLarge,
    #[error("last-project path is empty")]
    EmptyPath,
}
