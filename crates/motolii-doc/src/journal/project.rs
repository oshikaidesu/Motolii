//! D1cと並走するプロジェクト open/save(ジャーナル付き)。
//!
//! process間lock / stale lock / read-only fallbackは契約が無いため扱わない(#105スコープ外)。

use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

use crate::limits::ResourceLimits;
use crate::{Document, PersistError};

use super::catalog::{load_catalog_fs, save_catalog_fs, PinGenerationOptions, RotateOptions};
use super::format::{journal_path_for_document, JournalFormatError};
use super::fs::{FaultInjectingFs, FaultPlan, FsError, JournalFs, StdFs};
use super::recover::{recover_project, RecoveryError, RecoveryResult};
use super::replay::JournalEdit;
use super::wal::{checkpoint, commit_edit, CheckpointOptions, WalError, WalSession};

#[derive(Debug, Clone)]
pub struct SaveProjectOptions {
    pub limits: ResourceLimits,
    pub journal_edit: Option<JournalEdit>,
    /// trueならcheckpoint(世代スナップショット+main保存)を行う。
    pub checkpoint: bool,
    pub pin_generation: Option<PinGenerationOptions>,
    pub rotate: RotateOptions,
    pub max_unpinned_generations: Option<u32>,
    /// 既存project_idを引き継ぐ。新規なら生成。
    pub project_id: Option<Uuid>,
}

impl Default for SaveProjectOptions {
    fn default() -> Self {
        Self {
            limits: ResourceLimits::production(),
            journal_edit: None,
            checkpoint: true,
            pin_generation: None,
            rotate: RotateOptions::default(),
            max_unpinned_generations: None,
            project_id: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Wal(#[from] WalError),
    #[error(transparent)]
    Recovery(#[from] RecoveryError),
    #[error(transparent)]
    Format(#[from] JournalFormatError),
    #[error(transparent)]
    Catalog(#[from] super::catalog::CatalogError),
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type OpenProjectOutcome = RecoveryResult;

fn resolve_ids(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    options: &SaveProjectOptions,
) -> Result<(Uuid, u64, u32), ProjectError> {
    let max_unpinned = options.max_unpinned_generations.unwrap_or(5);
    if let Some(catalog) = load_catalog_fs(fs, document_path)? {
        let project_id = options.project_id.unwrap_or(catalog.project_id);
        return Ok((project_id, catalog.generation_salt, max_unpinned));
    }
    let project_id = options.project_id.unwrap_or_else(Uuid::new_v4);
    let salt = Uuid::new_v4().as_u128() as u64;
    Ok((project_id, salt, max_unpinned))
}

/// ジャーナル付き保存。
pub fn save_project_with_journal(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    let mut fs = StdFs;
    save_project_with_journal_fs(&mut fs, document_path, doc, options)
}

pub fn save_project_with_journal_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    doc.validate().map_err(PersistError::from)?;
    let (project_id, salt, max_unpinned) = resolve_ids(fs, document_path, options)?;
    let mut session =
        WalSession::open_or_create(fs, document_path, project_id, salt, max_unpinned)?;

    if let Some(edit) = &options.journal_edit {
        commit_edit(fs, &mut session, edit, &options.limits)?;
        // editのみではfingerprintを進めない — main未更新のままtipが進むため、
        // open時に必ずcommitted Editをリプレイする。
        save_catalog_fs(fs, document_path, &session.catalog)?;
    }

    // ピンはcheckpointのrotateより先(ガード6)。
    if let Some(pin) = &options.pin_generation {
        session.catalog.pin_generation(pin.generation_id)?;
        save_catalog_fs(fs, document_path, &session.catalog)?;
    }

    if options.checkpoint {
        let mut ckpt = CheckpointOptions {
            persist: Default::default(),
            rotate: options.rotate.clone(),
            pin: false,
        };
        if let Some(max) = options.max_unpinned_generations {
            ckpt.rotate.max_unpinned = Some(max);
            session.catalog.max_unpinned = max;
        }
        let _gen_id = checkpoint(
            fs,
            document_path,
            &mut session,
            doc,
            &ckpt,
            &options.limits,
        )?;
    }

    Ok(())
}

/// プロジェクトを開く(非破壊recovery込み)。
pub fn open_project(document_path: &Path) -> Result<OpenProjectOutcome, ProjectError> {
    open_project_with_limits(document_path, &ResourceLimits::production())
}

pub fn open_project_with_limits(
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<OpenProjectOutcome, ProjectError> {
    let mut fs = StdFs;
    open_project_fs(&mut fs, document_path, limits)
}

pub fn open_project_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<OpenProjectOutcome, ProjectError> {
    Ok(recover_project(fs, document_path, limits)?)
}

/// 故障注入プラン付きでcheckpointを走らせる(単体テスト用)。
pub fn checkpoint_with_fault_plan(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
    plan: FaultPlan,
) -> Result<(), ProjectError> {
    let mut faulty = FaultInjectingFs::new(plan);
    let parent = document_path.parent().unwrap_or_else(|| Path::new("."));
    faulty.seed_from_disk(parent)?;
    let motolii = super::format::motolii_dir_for_document(document_path);
    if motolii.exists() {
        faulty.seed_from_disk(&motolii)?;
    }
    let result = save_project_with_journal_fs(&mut faulty, document_path, doc, options);
    faulty.flush_durable_to_disk()?;
    result
}

// --- 壊れ方catalog注入(原本をtruncateしない) ---

pub fn inject_corrupt_journal_tail(document_path: &Path, garbage: &[u8]) -> Result<(), ProjectError> {
    let mut fs = StdFs;
    let path = journal_path_for_document(document_path);
    fs.append(&path, garbage)?;
    Ok(())
}

pub fn inject_bad_checksum_at_last_frame(document_path: &Path) -> Result<(), ProjectError> {
    let mut fs = StdFs;
    let path = journal_path_for_document(document_path);
    let mut data = fs.read(&path)?;
    if data.is_empty() {
        return Ok(());
    }
    let last = data.len() - 1;
    data[last] ^= 0xff;
    fs.write_create(&path, &data)?;
    Ok(())
}

pub fn inject_salt_mismatch_frame(document_path: &Path) -> Result<(), ProjectError> {
    use super::format::{encode_frame, JournalFrame, JournalRecordKind};
    let mut fs = StdFs;
    let path = journal_path_for_document(document_path);
    let data = fs.read(&path)?;
    let scan = super::format::scan_journal_bytes(&data, &Default::default())?;
    let bad = JournalFrame {
        record_id: Uuid::new_v4(),
        prev_id: scan.frames.last().map(|f| f.record_id),
        snapshot_ref: None,
        record_salt: scan.header.generation_salt ^ 0xdead_beef,
        kind: JournalRecordKind::Edit,
        payload: b"{}".to_vec(),
    };
    fs.append(&path, &encode_frame(&bad))?;
    Ok(())
}

