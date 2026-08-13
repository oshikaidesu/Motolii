//! D1cと並走するプロジェクト open/save(ジャーナル付き)。
//!
//! process間lock / stale lock / read-only fallbackは契約が無いため扱わない(#105スコープ外)。

use std::path::Path;

use thiserror::Error;
use uuid::Uuid;

use crate::limits::ResourceLimits;
use crate::{Document, PersistError};

use super::catalog::{load_catalog_fs, save_catalog_fs, PinGenerationOptions, RotateOptions};
use super::format::JournalFormatError;
use super::fs::{FsError, JournalFs};
use super::recover::{RecoveryError, RecoveryResult};
use super::replay::{load_generation_via_fs, JournalEdit};
use super::wal::{
    checkpoint, commit_edit, CheckpointOptions, JournalCommitReceipt, WalError, WalSession,
};

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
    #[error(transparent)]
    Plugin(#[from] crate::DocumentPluginError),
    #[error(transparent)]
    Session(#[from] super::session::SessionError),
    #[error("journal commit completed but its follow-up failed: {receipt:?}")]
    AfterJournalCommit {
        receipt: JournalCommitReceipt,
        #[source]
        source: Box<ProjectError>,
    },
    #[error("journal commit tip differs from both the previous and candidate tip (expected={expected}, previous={previous:?}, observed={observed:?})")]
    CommitTipConflict {
        expected: Uuid,
        previous: Option<Uuid>,
        observed: Option<Uuid>,
    },
}

impl ProjectError {
    pub fn uncertain_commit_receipt(&self) -> Option<JournalCommitReceipt> {
        match self {
            Self::Wal(WalError::CommitStateUncertain { receipt, .. })
            | Self::AfterJournalCommit { receipt, .. } => Some(*receipt),
            _ => None,
        }
    }
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
#[cfg(test)]
pub(crate) fn save_project_with_journal(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    use super::fs::StdFs;

    let mut fs = StdFs;
    save_project_with_journal_fs(&mut fs, document_path, doc, options)
}

pub(crate) fn save_project_with_journal_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    save_project_with_journal_outcome_fs(fs, document_path, doc, options).map(|_| ())
}

pub(crate) fn save_project_with_journal_outcome_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<Option<JournalCommitReceipt>, ProjectError> {
    doc.validate().map_err(PersistError::from)?;
    if options.journal_edit.is_some() {
        if fs.exists(document_path) {
            load_generation_via_fs(fs, document_path, &options.limits)?;
        } else {
            let catalog = load_catalog_fs(fs, document_path)?
                .ok_or(WalError::CommitVerificationBaseMissing)?;
            let generation = catalog
                .latest_generation()
                .ok_or(WalError::CommitVerificationBaseMissing)?;
            let path = super::catalog::generation_path_for_document(document_path, generation.id);
            load_generation_via_fs(fs, &path, &options.limits)?;
        }
    }
    let (project_id, salt, max_unpinned) = resolve_ids(fs, document_path, options)?;
    let mut session =
        WalSession::open_or_create(fs, document_path, project_id, salt, max_unpinned)?;

    let mut journal_receipt = None;
    if let Some(edit) = &options.journal_edit {
        let receipt = commit_edit(fs, document_path, &mut session, edit, doc, &options.limits)?;
        // editのみではfingerprintを進めない — main未更新のままtipが進むため、
        // open時に必ずcommitted Editをリプレイする。
        if let Err(source) = save_catalog_fs(fs, document_path, &session.catalog) {
            return Err(ProjectError::AfterJournalCommit {
                receipt,
                source: Box::new(ProjectError::Catalog(source)),
            });
        }
        journal_receipt = Some(receipt);
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
        let _gen_id = checkpoint(fs, document_path, &mut session, doc, &ckpt, &options.limits)?;
    }

    Ok(journal_receipt)
}

/// プロジェクトを開く(非破壊recovery込み)。crate内部・故障注入専用。
#[cfg(test)]
pub(crate) fn open_project(document_path: &Path) -> Result<OpenProjectOutcome, ProjectError> {
    open_project_with_limits(document_path, &ResourceLimits::production())
}

#[cfg(test)]
pub(crate) fn open_project_with_limits(
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<OpenProjectOutcome, ProjectError> {
    use super::fs::StdFs;

    let mut fs = StdFs;
    open_project_fs(&mut fs, document_path, limits)
}

#[cfg(test)]
pub(crate) fn open_project_fs(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<OpenProjectOutcome, ProjectError> {
    use super::recover::recover_project;

    Ok(recover_project(fs, document_path, limits)?)
}

/// 故障注入プラン付きでcheckpointを走らせる(単体テスト用)。
#[cfg(test)]
pub(crate) fn checkpoint_with_fault_plan(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
    plan: super::fs::FaultPlan,
) -> Result<(), ProjectError> {
    use super::fs::FaultInjectingFs;

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

#[cfg(test)]
pub(crate) fn inject_corrupt_journal_tail(
    document_path: &Path,
    garbage: &[u8],
) -> Result<(), ProjectError> {
    use super::format::journal_path_for_document;
    use super::fs::StdFs;

    let mut fs = StdFs;
    let path = journal_path_for_document(document_path);
    fs.append(&path, garbage)?;
    Ok(())
}

#[cfg(test)]
pub(crate) fn inject_bad_checksum_at_last_frame(document_path: &Path) -> Result<(), ProjectError> {
    use super::format::journal_path_for_document;
    use super::fs::StdFs;

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

#[cfg(test)]
pub(crate) fn inject_salt_mismatch_frame(document_path: &Path) -> Result<(), ProjectError> {
    use super::format::{encode_frame, journal_path_for_document, JournalFrame, JournalRecordKind};
    use super::fs::StdFs;

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

/// リプレイ失敗フォールバック試験用: 適用できない Command を commit する。
///
/// durable payload は通常の versioned `JournalEdit`/`Command` envelope のみ。
/// テスト専用の故障用 variant はオンディスク形式へ載せない。
#[cfg(test)]
pub(crate) fn inject_unapplicable_committed_edit(
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<(), ProjectError> {
    use super::catalog::{load_catalog_fs, save_catalog_fs};
    use super::format::{encode_frame, JournalFrame, JournalRecordKind};
    use super::fs::{DurabilityStage, JournalFs, StdFs};
    use super::replay::{edit_payload, JournalEdit};
    use super::session::ProjectSession;
    use super::wal::{WalError, WalSession};
    use crate::{Command, DocParam, LayerId, ScalarPropertyId};

    let edit = JournalEdit::new(Command::SetProperty {
        target: LayerId::from_raw(u64::MAX),
        property: ScalarPropertyId::Opacity,
        old_value: DocParam::const_f64(1.0),
        new_value: DocParam::const_f64(0.0),
    });
    let _session = ProjectSession::acquire(document_path, limits)?;
    let mut fs = StdFs;
    let catalog = load_catalog_fs(&mut fs, document_path)?
        .ok_or(ProjectError::Wal(WalError::CommitVerificationBaseMissing))?;
    let mut wal = WalSession::open_or_create(
        &mut fs,
        document_path,
        catalog.project_id,
        catalog.generation_salt,
        catalog.max_unpinned,
    )?;
    let payload = edit_payload(&edit)?;
    let observed = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    if observed > limits.max_command_payload_bytes {
        return Err(WalError::RecordPayloadLimit {
            observed,
            limit: limits.max_command_payload_bytes,
        }
        .into());
    }

    let edit_id = Uuid::new_v4();
    let edit_frame = encode_frame(&JournalFrame {
        record_id: edit_id,
        prev_id: wal.last_record,
        snapshot_ref: None,
        record_salt: wal.header.generation_salt,
        kind: JournalRecordKind::Edit,
        payload,
    });
    let commit_id = Uuid::new_v4();
    let commit_frame = encode_frame(&JournalFrame {
        record_id: commit_id,
        prev_id: Some(edit_id),
        snapshot_ref: None,
        record_salt: wal.header.generation_salt,
        kind: JournalRecordKind::Commit,
        payload: Vec::new(),
    });
    let current = fs.metadata_len(&wal.journal_path)?;
    limits
        .check_journal_bytes(current.saturating_add((edit_frame.len() + commit_frame.len()) as u64))
        .map_err(WalError::from)?;

    fs.append(&wal.journal_path, &edit_frame)?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&wal.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;
    fs.append(&wal.journal_path, &commit_frame)?;
    fs.note_stage(DurabilityStage::JournalAppend)?;
    fs.sync_file(&wal.journal_path)?;
    fs.note_stage(DurabilityStage::JournalFsync)?;
    wal.last_record = Some(commit_id);
    wal.catalog.edits_since_snapshot = wal.catalog.edits_since_snapshot.saturating_add(1);
    save_catalog_fs(&mut fs, document_path, &wal.catalog)?;
    Ok(())
}

#[cfg(test)]
mod v3_commit_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use motolii_core::RationalTime;

    use super::*;
    use crate::journal::format::{
        journal_path_for_document, V1_JOURNAL_FORMAT_VERSION, V2_JOURNAL_FORMAT_VERSION,
    };
    use crate::journal::fs::{DurabilityStage, FaultInjectingFs, FaultPlan, FsError};
    use crate::{
        Clip, ClipSource, Command, DocParam, ItemEnvelope, LayerId, ProjectSession,
        ScalarPropertyId, Track, TrackItem,
    };

    fn unique_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-v3-commit-{tag}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fixture() -> (Document, LayerId) {
        let mut document = Document::new_current();
        let layer = document.layers.allocate("clip").unwrap();
        let track = document.track_ids.allocate("V1").unwrap();
        let asset = document
            .assets
            .allocate("media", "video/mp4", "hash")
            .unwrap();
        document.tracks.push(Track {
            id: track,
            items: vec![TrackItem::Clip(Clip {
                envelope: ItemEnvelope::new(layer),
                start: RationalTime::ZERO,
                duration: RationalTime::try_new(5, 1).unwrap(),
                time_map: Default::default(),
                source: ClipSource::asset_video_only(asset),
            })],
        });
        document.validate().unwrap();
        (document, layer)
    }

    fn edit_and_candidate(document: &Document, layer: LayerId) -> (JournalEdit, Document) {
        let command = Command::SetProperty {
            target: layer,
            property: ScalarPropertyId::Opacity,
            old_value: DocParam::const_f64(1.0),
            new_value: DocParam::const_f64(0.25),
        };
        let mut candidate = document.clone();
        command.apply(&mut candidate).unwrap();
        (JournalEdit::new(command), candidate)
    }

    fn seeded_project(tag: &str, container_version: u32) -> (PathBuf, Document, LayerId) {
        let dir = unique_dir(tag);
        let path = dir.join("project.json");
        let (document, layer) = fixture();
        let mut std_fs = super::super::fs::StdFs;
        save_project_with_journal_fs(
            &mut std_fs,
            &path,
            &document,
            &SaveProjectOptions::default(),
        )
        .unwrap();
        if container_version == V1_JOURNAL_FORMAT_VERSION {
            let journal = journal_path_for_document(&path);
            let mut bytes = fs::read(&journal).unwrap();
            bytes[8..12].copy_from_slice(&V1_JOURNAL_FORMAT_VERSION.to_le_bytes());
            fs::write(journal, bytes).unwrap();
        }
        (path, document, layer)
    }

    #[test]
    fn pre_replace_failure_keeps_v1_and_v2_wal_bytes_unchanged() {
        for version in [V1_JOURNAL_FORMAT_VERSION, V2_JOURNAL_FORMAT_VERSION] {
            let (path, document, layer) = seeded_project("pre-replace", version);
            let journal = journal_path_for_document(&path);
            let before = fs::read(&journal).unwrap();
            let (edit, candidate) = edit_and_candidate(&document, layer);
            let mut faulty =
                FaultInjectingFs::new(FaultPlan::KillAfter(DurabilityStage::JournalTempFsync));
            faulty.seed_from_disk(path.parent().unwrap()).unwrap();

            let error = save_project_with_journal_fs(
                &mut faulty,
                &path,
                &candidate,
                &SaveProjectOptions {
                    journal_edit: Some(edit),
                    checkpoint: false,
                    ..Default::default()
                },
            )
            .unwrap_err();
            assert!(matches!(
                error,
                ProjectError::Wal(WalError::Fs(FsError::Aborted(
                    DurabilityStage::JournalTempFsync
                )))
            ));
            assert_eq!(faulty.durable_get(&journal), Some(before.as_slice()));
            let _ = fs::remove_dir_all(path.parent().unwrap());
        }
    }

    #[test]
    fn crash_after_v1_replace_recovers_the_complete_v3_candidate() {
        let (path, document, layer) = seeded_project("post-replace", V1_JOURNAL_FORMAT_VERSION);
        let (edit, candidate) = edit_and_candidate(&document, layer);
        let mut faulty =
            FaultInjectingFs::new(FaultPlan::KillAfter(DurabilityStage::JournalReplace));
        faulty.seed_from_disk(path.parent().unwrap()).unwrap();

        let error = save_project_with_journal_fs(
            &mut faulty,
            &path,
            &candidate,
            &SaveProjectOptions {
                journal_edit: Some(edit),
                checkpoint: false,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ProjectError::Wal(WalError::CommitStateUncertain {
                source: FsError::Aborted(DurabilityStage::JournalReplace),
                ..
            })
        ));
        faulty.flush_durable_to_disk().unwrap();

        let (_session, opened) =
            ProjectSession::open(&path, &ResourceLimits::production()).unwrap();
        assert_eq!(opened.document, candidate);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
