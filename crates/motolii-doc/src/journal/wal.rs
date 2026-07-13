//! commit / checkpoint の耐久順序(SQLite WAL契約の直輸入)。
//!
//! ## commit順序(テストで固定)
//! 1. JournalAppend (Edit frame)
//! 2. JournalFsync
//! 3. JournalAppend (Commit frame)
//! 4. JournalFsync
//!
//! ## checkpoint順序(テストで固定)
//! 1. (上記commitまで、Snapshot+Commit)
//! 2. MainTempWrite → MainTempFsync → MainRename → MainDirFsync
//! 3. CheckpointAppend → CheckpointFsync (世代salt更新)
//! 4. CatalogWrite → CatalogFsync

use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::limits::{ResourceLimitError, ResourceLimits};
use crate::{Document, PersistError};

use super::catalog::{
    generation_path_for_document, save_catalog_fs, GenerationCatalog, RotateOptions,
};
use super::format::{
    encode_frame, encode_header, journal_path_for_document, motolii_dir_for_document,
    read_or_create_header, JournalFrame, JournalHeader, JournalRecordKind, HEADER_LEN,
};
use super::fs::{DurabilityStage, FsError, JournalFs};
use super::replay::{
    checkpoint_payload, document_fingerprint, edit_payload, snapshot_payload, JournalEdit,
};

#[derive(Debug, Error)]
pub enum WalError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error(transparent)]
    Format(#[from] super::format::JournalFormatError),
    #[error(transparent)]
    Catalog(#[from] super::catalog::CatalogError),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    ResourceLimit(#[from] ResourceLimitError),
    #[error("journal record payload {observed} bytes exceeds limit {limit} bytes")]
    RecordPayloadLimit { observed: u32, limit: u32 },
}

#[derive(Debug, Clone)]
pub struct WalSession {
    pub project_id: Uuid,
    pub header: JournalHeader,
    pub catalog: GenerationCatalog,
    pub last_record: Option<Uuid>,
    pub journal_path: PathBuf,
}

impl WalSession {
    pub fn open_or_create(
        fs: &mut dyn JournalFs,
        document_path: &Path,
        project_id: Uuid,
        generation_salt: u64,
        max_unpinned: u32,
    ) -> Result<Self, WalError> {
        let dir = motolii_dir_for_document(document_path);
        fs.create_dir_all(&dir)?;
        let journal_path = journal_path_for_document(document_path);
        let header = read_or_create_header(fs, &journal_path, project_id, generation_salt)?;
        let catalog = match super::catalog::load_catalog_fs(fs, document_path)? {
            Some(mut c) => {
                if c.project_id != project_id {
                    return Err(WalError::Catalog(
                        super::catalog::CatalogError::ProjectIdMismatch {
                            catalog: c.project_id,
                            expected: project_id,
                        },
                    ));
                }
                c.generation_salt = header.generation_salt;
                c
            }
            None => GenerationCatalog::new(project_id, header.generation_salt, max_unpinned),
        };
        let (last_record, tip_salt) = if fs.exists(&journal_path) {
            let data = fs.read(&journal_path)?;
            match super::format::scan_journal_bytes(&data, &Default::default()) {
                Ok(scan) => {
                    // Checkpoint後の実効saltをsessionへ引き継ぐ(ヘッダ先頭の旧saltのままにしない)。
                    let tip_salt = {
                        let mut salt = scan.header.generation_salt;
                        for frame in &scan.frames {
                            if frame.kind == JournalRecordKind::Checkpoint
                                && frame.payload.len() >= 8
                            {
                                salt = u64::from_le_bytes(
                                    frame.payload[0..8].try_into().expect("new salt"),
                                );
                            }
                        }
                        salt
                    };
                    (scan.frames.last().map(|f| f.record_id), tip_salt)
                }
                Err(_) => (None, header.generation_salt),
            }
        } else {
            (None, header.generation_salt)
        };
        let mut header = header;
        header.generation_salt = tip_salt;
        let mut catalog = catalog;
        catalog.generation_salt = tip_salt;
        Ok(Self {
            project_id,
            header,
            catalog,
            last_record,
            journal_path,
        })
    }
}

fn check_payload_limits(
    payload: &[u8],
    journal_path: &Path,
    fs: &mut dyn JournalFs,
    limits: &ResourceLimits,
) -> Result<(), WalError> {
    let observed = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    if observed > limits.max_command_payload_bytes {
        return Err(WalError::RecordPayloadLimit {
            observed,
            limit: limits.max_command_payload_bytes,
        });
    }
    let current = if fs.exists(journal_path) {
        fs.metadata_len(journal_path)?
    } else {
        HEADER_LEN as u64
    };
    let frame_overhead = super::format::FRAME_PREFIX_LEN as u64;
    let upcoming = (frame_overhead + observed as u64) * 2;
    limits.check_journal_bytes(current.saturating_add(upcoming))?;
    Ok(())
}

/// Editを追記しCommit recordで閉じる。
pub fn commit_edit(
    fs: &mut dyn JournalFs,
    session: &mut WalSession,
    edit: &JournalEdit,
    limits: &ResourceLimits,
) -> Result<Uuid, WalError> {
    let payload = edit_payload(edit)?;
    check_payload_limits(&payload, &session.journal_path, fs, limits)?;

    let record_id = Uuid::new_v4();
    let edit_frame = JournalFrame {
        record_id,
        prev_id: session.last_record,
        snapshot_ref: None,
        record_salt: session.header.generation_salt,
        kind: JournalRecordKind::Edit,
        payload,
    };
    fs.append(&session.journal_path, &encode_frame(&edit_frame))?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&session.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;

    let commit_id = Uuid::new_v4();
    let commit_frame = JournalFrame {
        record_id: commit_id,
        prev_id: Some(record_id),
        snapshot_ref: None,
        record_salt: session.header.generation_salt,
        kind: JournalRecordKind::Commit,
        payload: Vec::new(),
    };
    fs.append(&session.journal_path, &encode_frame(&commit_frame))?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&session.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;

    session.last_record = Some(commit_id);
    session.catalog.edits_since_snapshot = session.catalog.edits_since_snapshot.saturating_add(1);
    Ok(record_id)
}

#[derive(Debug, Clone, Default)]
pub struct CheckpointOptions {
    pub persist: crate::SaveOptions,
    pub rotate: RotateOptions,
    pub pin: bool,
}

/// mainをアトミック保存し、世代saltを更新するcheckpoint。
pub fn checkpoint(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    session: &mut WalSession,
    doc: &Document,
    options: &CheckpointOptions,
    limits: &ResourceLimits,
) -> Result<Uuid, WalError> {
    let current = if fs.exists(&session.journal_path) {
        fs.metadata_len(&session.journal_path)?
    } else {
        HEADER_LEN as u64
    };
    limits.check_journal_bytes(current.saturating_add(4096))?;

    let generation_id = Uuid::new_v4();
    let snap_record = Uuid::new_v4();
    let payload = snapshot_payload(generation_id)?;
    if payload.len() as u32 > limits.max_command_payload_bytes {
        return Err(WalError::RecordPayloadLimit {
            observed: payload.len() as u32,
            limit: limits.max_command_payload_bytes,
        });
    }

    let snap_frame = JournalFrame {
        record_id: snap_record,
        prev_id: session.last_record,
        snapshot_ref: Some(generation_id),
        record_salt: session.header.generation_salt,
        kind: JournalRecordKind::Snapshot,
        payload,
    };
    fs.append(&session.journal_path, &encode_frame(&snap_frame))?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&session.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;

    let commit_id = Uuid::new_v4();
    let commit_frame = JournalFrame {
        record_id: commit_id,
        prev_id: Some(snap_record),
        snapshot_ref: Some(generation_id),
        record_salt: session.header.generation_salt,
        kind: JournalRecordKind::Commit,
        payload: Vec::new(),
    };
    fs.append(&session.journal_path, &encode_frame(&commit_frame))?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&session.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;

    let gen_path = generation_path_for_document(document_path, generation_id);
    if let Some(parent) = gen_path.parent() {
        fs.create_dir_all(parent)?;
    }
    let gen_bytes = serde_json::to_vec_pretty(doc)?;
    fs.write_create(&gen_path, &gen_bytes)?;
    fs.sync_file(&gen_path)?;

    let parent = document_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs.create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.motolii-ckpt-tmp",
        document_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("doc")
    ));
    let main_bytes = serde_json::to_vec_pretty(doc)?;
    fs.write_create(&tmp, &main_bytes)?;
    fs.note_stage(DurabilityStage::MainTempWrite)?;
    fs.sync_file(&tmp)?;
    fs.note_stage(DurabilityStage::MainTempFsync)?;
    if options.persist.abort_after == Some(crate::SaveAbortAfter::TempFsync) {
        return Err(WalError::Persist(PersistError::Aborted {
            stage: crate::SaveAbortAfter::TempFsync,
            temp_path: tmp,
        }));
    }
    fs.rename(&tmp, document_path)?;
    fs.note_stage(DurabilityStage::MainRename)?;
    fs.sync_dir(parent)?;
    fs.note_stage(DurabilityStage::MainDirFsync)?;

    let new_salt = Uuid::new_v4().as_u128() as u64;
    let cp_payload = checkpoint_payload(new_salt, generation_id)?;
    let cp_id = Uuid::new_v4();
    let cp_frame = JournalFrame {
        record_id: cp_id,
        prev_id: Some(commit_id),
        snapshot_ref: Some(generation_id),
        record_salt: session.header.generation_salt,
        kind: JournalRecordKind::Checkpoint,
        payload: cp_payload,
    };
    fs.append(&session.journal_path, &encode_frame(&cp_frame))?;
    fs.note_stage(DurabilityStage::CheckpointAppend)?;
    fs.sync_file(&session.journal_path)?;
    fs.note_stage(DurabilityStage::CheckpointFsync)?;

    session.header.generation_salt = new_salt;
    session.last_record = Some(cp_id);
    session.catalog.generation_salt = new_salt;
    session
        .catalog
        .register_generation(generation_id, snap_record, options.pin);
    session.catalog.edits_since_snapshot = 0;
    session.catalog.last_journaled_fingerprint = Some(document_fingerprint(doc));

    let max_unpinned = options
        .rotate
        .max_unpinned
        .unwrap_or(session.catalog.max_unpinned);
    let _removed = session.catalog.rotate_unpinned(max_unpinned);

    save_catalog_fs(fs, document_path, &session.catalog)?;
    fs.note_stage(DurabilityStage::CatalogWrite)?;
    let catalog_path = super::catalog::catalog_path_for_document(document_path);
    if fs.exists(&catalog_path) {
        fs.sync_file(&catalog_path)?;
    }
    fs.note_stage(DurabilityStage::CatalogFsync)?;

    Ok(generation_id)
}

/// 新規journalヘッダを明示saltで書く(テスト用)。
#[allow(dead_code)]
pub fn write_fresh_header(
    fs: &mut dyn JournalFs,
    journal_path: &Path,
    header: &JournalHeader,
) -> Result<(), WalError> {
    if let Some(parent) = journal_path.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.write_create(journal_path, &encode_header(header))?;
    fs.sync_file(journal_path)?;
    Ok(())
}
