//! commit / checkpoint の耐久順序(SQLite WAL契約の直輸入)。
//!
//! ## v3 commit順序(テストで固定)
//! 1. 既存accepted bytes + Edit + Commitをsibling tempへwrite
//! 2. tempをreplay検証してfsync
//! 3. journal.walをatomic replace
//! 4. journal directoryをfsync
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
    read_or_create_header, scan_journal_bytes, JournalFrame, JournalHeader, JournalRecordKind,
    JournalScanOutcome, JournalScanStop, ScanJournalOptions, HEADER_LEN, V1_JOURNAL_FORMAT_VERSION,
    V2_JOURNAL_FORMAT_VERSION,
};

/// Checkpoint frame 走査後の実効 generation salt。
pub(crate) fn tip_generation_salt_from_frames(header_salt: u64, frames: &[JournalFrame]) -> u64 {
    let mut salt = header_salt;
    for frame in frames {
        if frame.kind == JournalRecordKind::Checkpoint && frame.payload.len() >= 8 {
            salt = u64::from_le_bytes(frame.payload[0..8].try_into().expect("new salt"));
        }
    }
    salt
}
use super::fs::{DurabilityStage, FsError, JournalFs};
use super::replay::{
    checkpoint_payload, document_fingerprint, edit_payload, load_generation_via_fs,
    replay_from_base, snapshot_payload, JournalEdit, ReplayFailure, V3_EDIT_FORMAT_VERSION,
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
    #[error("journal edit format {observed} is not the current writer format {required}")]
    UnsupportedEditFormat { observed: u32, required: u32 },
    #[error("journal container version {0} cannot accept a v3 edit")]
    UnsupportedWriteContainer(u32),
    #[error(
        "journal has no fully accepted terminal prefix (stopped={stopped:?}, terminal={terminal:?})"
    )]
    UnacceptedTail {
        stopped: Option<JournalScanStop>,
        terminal: Option<JournalRecordKind>,
    },
    #[error("journal commit verification requires a main document or generation snapshot")]
    CommitVerificationBaseMissing,
    #[error("existing journal replay failed during commit verification: {failures:?}")]
    ExistingReplayFailed { failures: Vec<ReplayFailure> },
    #[error("candidate journal replay failed during commit verification: {failures:?}")]
    CandidateReplayFailed { failures: Vec<ReplayFailure> },
    #[error("candidate document does not equal the replayed v3 journal commit")]
    CandidateDocumentMismatch,
    #[error("journal temp write changed bytes (expected={expected}, observed={observed})")]
    TempWriteMismatch { expected: usize, observed: usize },
}

#[derive(Debug, Clone)]
pub struct WalSession {
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
                    let tip_salt =
                        tip_generation_salt_from_frames(scan.header.generation_salt, &scan.frames);
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
            header,
            catalog,
            last_record,
            journal_path,
        })
    }
}

fn check_commit_limits(
    payload: &[u8],
    candidate_bytes: usize,
    limits: &ResourceLimits,
) -> Result<(), WalError> {
    let observed = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    if observed > limits.max_command_payload_bytes {
        return Err(WalError::RecordPayloadLimit {
            observed,
            limit: limits.max_command_payload_bytes,
        });
    }
    limits.check_journal_bytes(candidate_bytes as u64)?;
    Ok(())
}

fn accepted_terminal(scan: &JournalScanOutcome) -> bool {
    scan.stopped.is_none()
        && matches!(
            scan.frames.last().map(|frame| frame.kind),
            None | Some(JournalRecordKind::Commit | JournalRecordKind::Checkpoint)
        )
}

fn load_commit_verification_base(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    session: &WalSession,
    limits: &ResourceLimits,
) -> Result<Document, WalError> {
    if fs.exists(document_path) {
        return Ok(load_generation_via_fs(fs, document_path, limits)?);
    }
    let generation = session
        .catalog
        .latest_generation()
        .ok_or(WalError::CommitVerificationBaseMissing)?;
    let path = generation_path_for_document(document_path, generation.id);
    Ok(load_generation_via_fs(fs, &path, limits)?)
}

fn replay_for_commit_verification(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    limits: &ResourceLimits,
    base: Document,
    scan: &JournalScanOutcome,
) -> super::replay::ReplayOutcome {
    let mut load_snapshot = |generation_id| {
        load_generation_via_fs(
            fs,
            &generation_path_for_document(document_path, generation_id),
            limits,
        )
        .map_err(|_| ReplayFailure::MissingSnapshot { generation_id })
    };
    replay_from_base(base, scan, &mut load_snapshot, false)
}

/// v3 Editを既存WALへ論理追記し、全bytesを一つのatomic replaceで閉じる。
pub fn commit_edit(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    session: &mut WalSession,
    edit: &JournalEdit,
    candidate: &Document,
    limits: &ResourceLimits,
) -> Result<Uuid, WalError> {
    if edit.format_version != V3_EDIT_FORMAT_VERSION {
        return Err(WalError::UnsupportedEditFormat {
            observed: edit.format_version,
            required: V3_EDIT_FORMAT_VERSION,
        });
    }
    let payload = edit_payload(edit)?;
    let original = fs.read(&session.journal_path)?;
    let options = ScanJournalOptions {
        verify_prev_chain: true,
        expected_project_id: Some(session.header.project_id),
    };
    let original_scan = scan_journal_bytes(&original, &options)?;
    if !matches!(
        original_scan.header.version,
        V1_JOURNAL_FORMAT_VERSION | V2_JOURNAL_FORMAT_VERSION
    ) {
        return Err(WalError::UnsupportedWriteContainer(
            original_scan.header.version,
        ));
    }
    if !accepted_terminal(&original_scan) {
        return Err(WalError::UnacceptedTail {
            stopped: original_scan.stopped.clone(),
            terminal: original_scan.frames.last().map(|frame| frame.kind),
        });
    }

    let base = load_commit_verification_base(fs, document_path, session, limits)?;
    let existing =
        replay_for_commit_verification(fs, document_path, limits, base.clone(), &original_scan);
    if !existing.replay_failures.is_empty() {
        return Err(WalError::ExistingReplayFailed {
            failures: existing.replay_failures,
        });
    }

    let record_id = Uuid::new_v4();
    let tip_salt = tip_generation_salt_from_frames(
        original_scan.header.generation_salt,
        &original_scan.frames,
    );
    let edit_frame = JournalFrame {
        record_id,
        prev_id: original_scan.frames.last().map(|frame| frame.record_id),
        snapshot_ref: None,
        record_salt: tip_salt,
        kind: JournalRecordKind::Edit,
        payload,
    };

    let commit_id = Uuid::new_v4();
    let commit_frame = JournalFrame {
        record_id: commit_id,
        prev_id: Some(record_id),
        snapshot_ref: None,
        record_salt: tip_salt,
        kind: JournalRecordKind::Commit,
        payload: Vec::new(),
    };

    let mut candidate_bytes = original;
    candidate_bytes[8..12].copy_from_slice(&V2_JOURNAL_FORMAT_VERSION.to_le_bytes());
    candidate_bytes.extend_from_slice(&encode_frame(&edit_frame));
    candidate_bytes.extend_from_slice(&encode_frame(&commit_frame));
    check_commit_limits(&edit_frame.payload, candidate_bytes.len(), limits)?;

    let parent = session
        .journal_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temp_path = parent.join(format!(".journal.wal.{}.motolii-v3-tmp", Uuid::new_v4()));
    fs.write_create(&temp_path, &candidate_bytes)?;
    fs.note_stage(DurabilityStage::JournalTempWrite)?;

    let temp_bytes = fs.read(&temp_path)?;
    if temp_bytes != candidate_bytes {
        return Err(WalError::TempWriteMismatch {
            expected: candidate_bytes.len(),
            observed: temp_bytes.len(),
        });
    }
    let candidate_scan = scan_journal_bytes(&temp_bytes, &options)?;
    if candidate_scan.header.version != V2_JOURNAL_FORMAT_VERSION
        || !accepted_terminal(&candidate_scan)
    {
        return Err(WalError::UnacceptedTail {
            stopped: candidate_scan.stopped.clone(),
            terminal: candidate_scan.frames.last().map(|frame| frame.kind),
        });
    }
    let replayed = replay_for_commit_verification(fs, document_path, limits, base, &candidate_scan);
    if !replayed.replay_failures.is_empty() {
        return Err(WalError::CandidateReplayFailed {
            failures: replayed.replay_failures,
        });
    }
    if replayed.document != *candidate {
        return Err(WalError::CandidateDocumentMismatch);
    }

    fs.sync_file(&temp_path)?;
    fs.note_stage(DurabilityStage::JournalTempFsync)?;
    fs.rename(&temp_path, &session.journal_path)?;
    fs.note_stage(DurabilityStage::JournalReplace)?;
    fs.sync_dir(parent)?;
    fs.note_stage(DurabilityStage::JournalDirFsync)?;

    session.header.version = V2_JOURNAL_FORMAT_VERSION;
    session.header.generation_salt = tip_salt;
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

#[cfg(test)]
mod fs_order_tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use motolii_core::RationalTime;
    use uuid::Uuid;

    use crate::journal::replay::JournalEdit;
    use crate::{
        Clip, ClipSource, Command, DocParam, Document, ItemEnvelope, LayerId, ResourceLimits,
        ScalarPropertyId, Track, TrackItem,
    };

    use super::*;
    use crate::journal::fs::{FsOpKind, RecordingFs, StdFs};

    fn set_opacity_cmd(layer: LayerId, old: f64, new: f64) -> JournalEdit {
        JournalEdit::new(Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(old),
            new_value: DocParam::const_f64(new),
        })
    }

    fn doc_with_clip() -> (Document, LayerId) {
        let mut doc = Document::new_current();
        let layer = doc.layers.allocate("a").unwrap();
        let track = doc.track_ids.allocate("V1").unwrap();
        let asset = doc.assets.allocate("media", "video/mp4", "hash").unwrap();
        doc.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            })],
        });
        doc.validate().expect("fixture must validate");
        (doc, layer)
    }

    #[test]
    fn commit_and_checkpoint_fsync_order_is_fixed() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("motolii-wal-order-{nanos}"));
        std::fs::create_dir_all(&path).unwrap();
        let path = path.join("proj.json");
        let (doc, layer) = doc_with_clip();

        let (mut fs, log) = RecordingFs::new(StdFs);
        let project_id = Uuid::new_v4();
        let salt = 0x1111_2222_3333_4444;
        let mut session = WalSession::open_or_create(&mut fs, &path, project_id, salt, 5).unwrap();
        checkpoint(
            &mut fs,
            &path,
            &mut session,
            &doc,
            &CheckpointOptions::default(),
            &ResourceLimits::production(),
        )
        .unwrap();
        log.lock().unwrap().clear();

        let edit = set_opacity_cmd(layer, 1.0, 0.5);
        let mut candidate = doc.clone();
        edit.command.apply(&mut candidate).unwrap();
        candidate.validate().unwrap();

        commit_edit(
            &mut fs,
            &path,
            &mut session,
            &edit,
            &candidate,
            &ResourceLimits::production(),
        )
        .unwrap();

        {
            let ops = log.lock().unwrap();
            let stages: Vec<_> = ops
                .iter()
                .filter(|o| o.kind == FsOpKind::NoteStage)
                .map(|o| o.detail.clone())
                .collect();
            assert!(
                stages.windows(4).any(|w| {
                    w[0].contains("JournalTempWrite")
                        && w[1].contains("JournalTempFsync")
                        && w[2].contains("JournalReplace")
                        && w[3].contains("JournalDirFsync")
                }),
                "v3 commit order must be temp-write→fsync→replace→dir-fsync, got {stages:?}"
            );
        }

        log.lock().unwrap().clear();
        checkpoint(
            &mut fs,
            &path,
            &mut session,
            &candidate,
            &CheckpointOptions::default(),
            &ResourceLimits::production(),
        )
        .unwrap();

        let ops = log.lock().unwrap();
        let stages: Vec<_> = ops
            .iter()
            .filter(|o| o.kind == FsOpKind::NoteStage)
            .map(|o| o.detail.clone())
            .collect();
        let required = [
            "MainTempWrite",
            "MainTempFsync",
            "MainRename",
            "MainDirFsync",
            "CheckpointAppend",
            "CheckpointFsync",
            "CatalogWrite",
            "CatalogFsync",
        ];
        let mut pos = 0usize;
        for req in required {
            let found = stages[pos..]
                .iter()
                .position(|s| s.contains(req))
                .unwrap_or_else(|| panic!("missing stage {req} in {stages:?}"));
            pos += found + 1;
        }
    }
}
