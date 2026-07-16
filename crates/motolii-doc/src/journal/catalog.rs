//! スナップショット世代カタログ。件数ローテーションはピン留めを尊重する(ガード6)。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::format::motolii_dir_for_document;
use super::fs::{FsError, JournalFs};

pub const CATALOG_FILENAME: &str = "catalog.json";
pub const GENERATIONS_DIR: &str = "generations";
pub const CATALOG_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationEntry {
    pub id: Uuid,
    /// この世代を記したjournalレコードUUID(相互参照)。
    pub journal_record: Uuid,
    pub pinned: bool,
    pub created_seq: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationCatalog {
    pub format_version: u32,
    pub project_id: Uuid,
    pub generation_salt: u64,
    pub max_unpinned: u32,
    pub next_seq: u64,
    pub generations: Vec<GenerationEntry>,
    #[serde(default)]
    pub edits_since_snapshot: u32,
    /// mainとjournal tipの照合用指紋。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_journaled_fingerprint: Option<u64>,
}

impl GenerationCatalog {
    pub fn new(project_id: Uuid, generation_salt: u64, max_unpinned: u32) -> Self {
        Self {
            format_version: CATALOG_FORMAT_VERSION,
            project_id,
            generation_salt,
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

    /// ピン留めされていない世代のうち古いものから削除対象を返す。
    pub fn rotate_unpinned(&mut self, max_unpinned: u32) -> Vec<Uuid> {
        let mut removed = Vec::new();
        while self.unpinned_count() > max_unpinned as usize {
            let oldest = self
                .generations
                .iter()
                .filter(|g| !g.pinned)
                .min_by_key(|g| g.created_seq)
                .map(|g| g.id);
            let Some(id) = oldest else {
                break;
            };
            self.generations.retain(|g| g.id != id);
            removed.push(id);
        }
        removed
    }
}

#[derive(Debug, Clone, Default)]
pub struct RotateOptions {
    pub max_unpinned: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct PinGenerationOptions {
    pub generation_id: Uuid,
}

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unknown generation {0}")]
    UnknownGeneration(Uuid),
    #[error("catalog project_id mismatch")]
    ProjectIdMismatch { catalog: Uuid, expected: Uuid },
}

pub fn catalog_path_for_document(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(CATALOG_FILENAME)
}

pub fn generation_path_for_document(document_path: &Path, generation_id: Uuid) -> PathBuf {
    motolii_dir_for_document(document_path)
        .join(GENERATIONS_DIR)
        .join(format!("{generation_id}.json"))
}

pub fn load_catalog_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
) -> Result<Option<GenerationCatalog>, CatalogError> {
    let path = catalog_path_for_document(document_path);
    if !fs.exists(&path) {
        return Ok(None);
    }
    let bytes = fs.read(&path)?;
    let catalog: GenerationCatalog = serde_json::from_slice(&bytes)?;
    Ok(Some(catalog))
}

pub fn load_catalog(document_path: &Path) -> Result<Option<GenerationCatalog>, CatalogError> {
    let mut fs = super::fs::StdFs;
    load_catalog_fs(&mut fs, document_path)
}

pub fn save_catalog_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    catalog: &GenerationCatalog,
) -> Result<(), CatalogError> {
    let dir = motolii_dir_for_document(document_path);
    fs.create_dir_all(&dir)?;
    let path = catalog_path_for_document(document_path);
    let bytes = serde_json::to_vec_pretty(catalog)?;
    let tmp = path.with_extension("json.tmp");
    fs.write_create(&tmp, &bytes)?;
    fs.sync_file(&tmp)?;
    fs.rename(&tmp, &path)?;
    fs.sync_dir(&dir)?;
    Ok(())
}
