//! スナップショット世代カタログ。件数ローテーションはピン留めを尊重する(ガード6)。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::format::JournalFormatError;

pub const CATALOG_FILENAME: &str = "catalog.json";
pub const GENERATIONS_DIR: &str = "generations";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationEntry {
    pub id: Uuid,
    pub journal_record: Uuid,
    pub pinned: bool,
    pub created_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationCatalog {
    pub format_version: u32,
    pub journal_salt: u64,
    pub max_unpinned: u32,
    pub next_seq: u64,
    pub generations: Vec<GenerationEntry>,
    /// 直近スナップショット以降の Edit 件数。跨ぎ save でも保持する。
    #[serde(default)]
    pub edits_since_snapshot: u32,
    /// 最後にジャーナル追記した Document の指紋。main 先行判定に使う。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_journaled_fingerprint: Option<u64>,
}

impl GenerationCatalog {
    pub fn new(journal_salt: u64, max_unpinned: u32) -> Self {
        Self {
            format_version: 1,
            journal_salt,
            max_unpinned,
            next_seq: 0,
            generations: Vec::new(),
            edits_since_snapshot: 0,
            last_journaled_fingerprint: None,
        }
    }

    pub fn register_generation(&mut self, id: Uuid, journal_record: Uuid, pinned: bool) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.generations.push(GenerationEntry {
            id,
            journal_record,
            pinned,
            created_seq: seq,
        });
        seq
    }

    pub fn pin_generation(&mut self, id: Uuid) -> Result<(), CatalogError> {
        let entry = self
            .generations
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or(CatalogError::UnknownGeneration(id))?;
        entry.pinned = true;
        Ok(())
    }

    pub fn unpinned_count(&self) -> usize {
        self.generations.iter().filter(|g| !g.pinned).count()
    }

    pub fn find(&self, id: Uuid) -> Option<&GenerationEntry> {
        self.generations.iter().find(|g| g.id == id)
    }

    pub fn latest_generation(&self) -> Option<&GenerationEntry> {
        self.generations.iter().max_by_key(|g| g.created_seq)
    }
}

#[derive(Debug, Clone, Default)]
pub struct RotateOptions {
    /// 省略時は catalog.max_unpinned を使う。
    pub max_unpinned: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PinGenerationOptions {
    pub generation_id: Uuid,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Format(#[from] JournalFormatError),
    #[error("unknown generation {0}")]
    UnknownGeneration(Uuid),
}

pub fn motolii_dir_for_document(document_path: &Path) -> PathBuf {
    document_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(".motolii")
}

pub fn catalog_path_for_document(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(CATALOG_FILENAME)
}

pub fn generation_path_for_document(document_path: &Path, generation_id: Uuid) -> PathBuf {
    motolii_dir_for_document(document_path)
        .join(GENERATIONS_DIR)
        .join(format!("{generation_id}.json"))
}

pub fn load_catalog(document_path: &Path) -> Result<Option<GenerationCatalog>, CatalogError> {
    let path = catalog_path_for_document(document_path);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)?;
    Ok(Some(serde_json::from_slice(&bytes)?))
}

/// 壊れた catalog はエラーにせず None(呼び出し側で再構築)。
pub fn load_catalog_lenient(
    document_path: &Path,
) -> Result<(Option<GenerationCatalog>, bool), CatalogError> {
    let path = catalog_path_for_document(document_path);
    if !path.exists() {
        return Ok((None, false));
    }
    let bytes = fs::read(&path)?;
    match serde_json::from_slice::<GenerationCatalog>(&bytes) {
        Ok(catalog) => Ok((Some(catalog), false)),
        Err(_) => Ok((None, true)),
    }
}

/// journal salt と generations ディレクトリから最小カタログを再構築する。
pub fn rebuild_catalog_from_generations(
    document_path: &Path,
    journal_salt: u64,
    max_unpinned: u32,
) -> Result<GenerationCatalog, CatalogError> {
    let mut catalog = GenerationCatalog::new(journal_salt, max_unpinned);
    let gen_dir = motolii_dir_for_document(document_path).join(GENERATIONS_DIR);
    if !gen_dir.exists() {
        return Ok(catalog);
    }
    let mut ids = Vec::new();
    for entry in fs::read_dir(gen_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".json") else {
            continue;
        };
        if let Ok(id) = Uuid::parse_str(stem) {
            ids.push(id);
        }
    }
    ids.sort_by_key(|id| *id.as_bytes());
    for id in ids {
        catalog.register_generation(id, Uuid::nil(), false);
    }
    Ok(catalog)
}

pub fn save_catalog(document_path: &Path, catalog: &GenerationCatalog) -> Result<(), CatalogError> {
    let dir = motolii_dir_for_document(document_path);
    fs::create_dir_all(&dir)?;
    let path = dir.join(CATALOG_FILENAME);
    let bytes = serde_json::to_vec_pretty(catalog)?;
    fs::write(path, bytes)?;
    Ok(())
}

/// ピン留めされていない古い世代だけを削除する。ピン留めは件数上限を無視する(ガード6)。
pub fn rotate_generations(
    document_path: &Path,
    catalog: &mut GenerationCatalog,
    options: &RotateOptions,
) -> Result<Vec<Uuid>, CatalogError> {
    let max_unpinned = options.max_unpinned.unwrap_or(catalog.max_unpinned) as usize;
    let mut removed = Vec::new();
    while catalog.unpinned_count() > max_unpinned {
        let oldest = catalog
            .generations
            .iter()
            .filter(|g| !g.pinned)
            .min_by_key(|g| g.created_seq)
            .map(|g| g.id);
        let Some(victim) = oldest else {
            break;
        };
        let path = generation_path_for_document(document_path, victim);
        if path.exists() {
            fs::remove_file(path)?;
        }
        catalog.generations.retain(|g| g.id != victim);
        removed.push(victim);
    }
    Ok(removed)
}
