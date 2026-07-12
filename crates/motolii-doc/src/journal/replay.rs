//! ジャーナルリプレイと失敗時スナップショットフォールバック(ガード4)。

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{Bpm, Document, DocumentError, PersistError};

use super::catalog::{generation_path_for_document, GenerationCatalog};
use super::format::{JournalFrame, JournalRecordKind, JournalScanOutcome};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum JournalEdit {
    SetBpm {
        num: i64,
        den: i64,
    },
    /// テスト注入専用 — 意図的にリプレイ失敗させる。
    ForceReplayFail,
    /// テスト注入専用 — リプレイ中に panic(ガード4 の catch_unwind / 隔離)。
    ForceReplayPanic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    ForcedByTest {
        record_id: Uuid,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ReplayApplyError {
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error(transparent)]
    Bpm(#[from] crate::BpmError),
    #[error("forced replay failure")]
    Forced,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayOutcome {
    pub document: Document,
    pub applied_records: usize,
    pub fallback_generation: Option<Uuid>,
    pub replay_failures: Vec<ReplayFailure>,
}

#[derive(Debug, Clone, Default)]
pub struct ReplayOptions {
    /// true なら最初の適用失敗で直前スナップショットへフォールバックする。
    pub fallback_on_failure: bool,
}

pub fn load_generation_snapshot(
    document_path: &Path,
    generation_id: Uuid,
) -> Result<Document, PersistError> {
    let path = generation_path_for_document(document_path, generation_id);
    let bytes = fs::read(path)?;
    crate::load_document_bytes(&bytes)
}

fn apply_edit(doc: &mut Document, edit: &JournalEdit) -> Result<(), ReplayApplyError> {
    match edit {
        JournalEdit::SetBpm { num, den } => {
            doc.bpm = Bpm::try_new(*num, *den)?;
        }
        JournalEdit::ForceReplayFail => return Err(ReplayApplyError::Forced),
        JournalEdit::ForceReplayPanic => {
            panic!("motolii: injected journal replay panic");
        }
    }
    doc.validate()?;
    Ok(())
}

fn decode_edit(payload: &[u8]) -> Result<JournalEdit, String> {
    serde_json::from_slice(payload).map_err(|e| e.to_string())
}

fn snapshot_from_frame(
    document_path: &Path,
    frame: &JournalFrame,
) -> Result<Document, ReplayFailure> {
    let generation_id = frame
        .snapshot_ref
        .or_else(|| {
            serde_json::from_slice::<SnapshotPayload>(&frame.payload)
                .ok()
                .map(|p| p.generation_id)
        })
        .ok_or_else(|| ReplayFailure::InvalidEditPayload {
            record_id: frame.record_id,
            reason: "snapshot frame missing generation ref".into(),
        })?;
    load_generation_snapshot(document_path, generation_id)
        .map_err(|_| ReplayFailure::MissingSnapshot { generation_id })
}

#[derive(Debug, Serialize, Deserialize)]
struct SnapshotPayload {
    generation_id: Uuid,
}

pub fn replay_journal(
    document_path: &Path,
    base: Document,
    scan: &JournalScanOutcome,
    _catalog: &GenerationCatalog,
    options: &ReplayOptions,
) -> ReplayOutcome {
    let mut doc = base.clone();
    let mut last_snapshot: Option<Document> = Some(base);
    let mut applied = 0usize;
    let mut failures = Vec::new();
    let mut fallback_generation = None;

    for frame in &scan.frames {
        match frame.kind {
            JournalRecordKind::Snapshot => match snapshot_from_frame(document_path, frame) {
                Ok(snapshot) => {
                    fallback_generation = frame.snapshot_ref.or_else(|| {
                        serde_json::from_slice::<SnapshotPayload>(&frame.payload)
                            .ok()
                            .map(|p| p.generation_id)
                    });
                    doc = snapshot.clone();
                    last_snapshot = Some(snapshot);
                    applied += 1;
                }
                Err(err) => {
                    failures.push(err);
                    if options.fallback_on_failure {
                        // Edit 失敗と同じく直前の成功スナップショットへ戻す
                        if let Some(snapshot) = last_snapshot.clone() {
                            doc = snapshot;
                        }
                        break;
                    }
                }
            },
            JournalRecordKind::Edit => match decode_edit(&frame.payload) {
                Ok(edit) => {
                    if let Err(source) = apply_edit(&mut doc, &edit) {
                        failures.push(ReplayFailure::ApplyEdit {
                            record_id: frame.record_id,
                            source,
                        });
                        if options.fallback_on_failure {
                            if let Some(snapshot) = last_snapshot.clone() {
                                doc = snapshot;
                            }
                            break;
                        }
                    } else {
                        applied += 1;
                    }
                }
                Err(reason) => {
                    failures.push(ReplayFailure::InvalidEditPayload {
                        record_id: frame.record_id,
                        reason,
                    });
                    if options.fallback_on_failure {
                        if let Some(snapshot) = last_snapshot.clone() {
                            doc = snapshot;
                        }
                        break;
                    }
                }
            },
            JournalRecordKind::PinGeneration => {
                // カタログ側のメタデータ — リプレイ状態には影響しない。
                applied += 1;
            }
        }
    }

    ReplayOutcome {
        document: doc,
        applied_records: applied,
        fallback_generation,
        replay_failures: failures,
    }
}

pub fn snapshot_payload(generation_id: Uuid) -> Vec<u8> {
    serde_json::to_vec(&SnapshotPayload { generation_id }).expect("snapshot payload")
}

pub fn edit_payload(edit: &JournalEdit) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(edit)
}

/// Document 内容の安定指紋(main 先行判定用)。
pub fn document_fingerprint(doc: &Document) -> Result<u64, serde_json::Error> {
    let bytes = serde_json::to_vec(doc)?;
    Ok(u64::from(crc32fast::hash(&bytes)))
}
