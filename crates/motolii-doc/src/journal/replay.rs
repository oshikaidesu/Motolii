//! ジャーナルリプレイと失敗時スナップショットフォールバック(ガード4)。
//!
//! Editレコードの耐久payloadは versioned `Command` envelope のみ。
//! 故障注入は適用不能 Command の commit 等で行い、durable 形式へテスト専用 variant を焼かない。

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{Command, CommandError, Document, DocumentError};

use super::format::{JournalFrame, JournalRecordKind, JournalScanOutcome};
use super::fs::JournalFs;

/// Editレコードのオンディスクpayload(恒久面)。`Command`を版付きで包む。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEdit {
    pub format_version: u32,
    pub command: Command,
}

impl JournalEdit {
    pub const FORMAT_VERSION: u32 = 1;

    pub fn new(command: Command) -> Self {
        Self {
            format_version: Self::FORMAT_VERSION,
            command,
        }
    }
}

impl From<Command> for JournalEdit {
    fn from(command: Command) -> Self {
        Self::new(command)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReplayFailure {
    InvalidEditPayload {
        record_id: Uuid,
        reason: String,
    },
    ApplyEdit {
        record_id: Uuid,
        source: ReplayApplyError,
    },
    MissingSnapshot {
        generation_id: Uuid,
    },
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ReplayApplyError {
    #[error(transparent)]
    Command(#[from] CommandError),
    #[error(transparent)]
    Document(#[from] DocumentError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayOutcome {
    pub document: Document,
    pub applied_records: usize,
    pub fallback_generation: Option<Uuid>,
    pub replay_failures: Vec<ReplayFailure>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub generation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointPayload {
    pub new_salt: u64,
    pub generation_id: Uuid,
}

pub fn edit_payload(edit: &JournalEdit) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(edit)
}

pub fn snapshot_payload(generation_id: Uuid) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&SnapshotPayload { generation_id })
}

pub fn checkpoint_payload(
    new_salt: u64,
    generation_id: Uuid,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = new_salt.to_le_bytes().to_vec();
    bytes.extend(serde_json::to_vec(&CheckpointPayload {
        new_salt,
        generation_id,
    })?);
    Ok(bytes)
}

pub fn document_fingerprint(doc: &Document) -> u64 {
    // 安定したJSON指紋 — tip照合用(暗号強度は不要)。
    let bytes = serde_json::to_vec(doc).unwrap_or_default();
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn apply_edit(doc: &mut Document, edit: &JournalEdit) -> Result<(), ReplayApplyError> {
    edit.command.apply(doc)?;
    doc.validate()?;
    Ok(())
}

fn decode_edit(payload: &[u8]) -> Result<JournalEdit, String> {
    let edit: JournalEdit = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    if edit.format_version != JournalEdit::FORMAT_VERSION {
        return Err(format!(
            "unsupported journal edit format_version {} (expected {})",
            edit.format_version,
            JournalEdit::FORMAT_VERSION
        ));
    }
    Ok(edit)
}

/// `base`からscan内のEditを順に適用。失敗時は直近Snapshot世代へフォールバック可能。
pub fn replay_from_base(
    mut base: Document,
    scan: &JournalScanOutcome,
    load_snapshot: &mut dyn FnMut(Uuid) -> Result<Document, ReplayFailure>,
    fallback_on_failure: bool,
) -> ReplayOutcome {
    let mut applied = 0usize;
    let mut failures = Vec::new();
    let mut last_snapshot: Option<Uuid> = None;
    let mut doc_at_last_snapshot = base.clone();

    for frame in &scan.frames {
        match frame.kind {
            JournalRecordKind::Snapshot => {
                let generation_id = frame.snapshot_ref.or_else(|| {
                    serde_json::from_slice::<SnapshotPayload>(&frame.payload)
                        .ok()
                        .map(|p| p.generation_id)
                });
                if let Some(gid) = generation_id {
                    match load_snapshot(gid) {
                        Ok(doc) => {
                            base = doc.clone();
                            doc_at_last_snapshot = doc;
                            last_snapshot = Some(gid);
                        }
                        Err(e) => failures.push(e),
                    }
                }
            }
            JournalRecordKind::Edit => match decode_edit(&frame.payload) {
                Ok(edit) => match apply_edit(&mut base, &edit) {
                    Ok(()) => applied += 1,
                    Err(source) => {
                        failures.push(ReplayFailure::ApplyEdit {
                            record_id: frame.record_id,
                            source,
                        });
                        if fallback_on_failure {
                            if let Some(gid) = last_snapshot {
                                return ReplayOutcome {
                                    document: doc_at_last_snapshot,
                                    applied_records: applied,
                                    fallback_generation: Some(gid),
                                    replay_failures: failures,
                                };
                            }
                        }
                    }
                },
                Err(reason) => {
                    failures.push(ReplayFailure::InvalidEditPayload {
                        record_id: frame.record_id,
                        reason,
                    });
                    if fallback_on_failure {
                        if let Some(gid) = last_snapshot {
                            return ReplayOutcome {
                                document: doc_at_last_snapshot,
                                applied_records: applied,
                                fallback_generation: Some(gid),
                                replay_failures: failures,
                            };
                        }
                    }
                }
            },
            JournalRecordKind::Commit | JournalRecordKind::Checkpoint => {}
        }
    }

    ReplayOutcome {
        document: base,
        applied_records: applied,
        fallback_generation: None,
        replay_failures: failures,
    }
}

pub fn load_generation_via_fs(
    fs: &mut dyn JournalFs,
    path: &std::path::Path,
    limits: &crate::limits::ResourceLimits,
) -> Result<Document, crate::PersistError> {
    let bytes = fs.read(path).map_err(|e| match e {
        super::fs::FsError::Io(io) => crate::PersistError::Io(io),
        other => crate::PersistError::Io(std::io::Error::other(other.to_string())),
    })?;
    crate::load_document_bytes_with_limits(&bytes, limits).map(|opened| opened.document)
}

/// フレーム列から「最後にCommitされた」範囲だけを残す(未commitテールは無視)。
pub fn frames_through_last_commit(frames: &[JournalFrame]) -> &[JournalFrame] {
    let last_commit = frames
        .iter()
        .rposition(|f| f.kind == JournalRecordKind::Commit);
    match last_commit {
        Some(i) => &frames[..=i],
        None => &[],
    }
}
