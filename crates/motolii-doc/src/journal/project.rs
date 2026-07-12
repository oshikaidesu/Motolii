//! D1c と並走するプロジェクト open/save。D1c 契約はそのまま呼び出す。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use uuid::Uuid;

use crate::persist::{save_document, save_document_with_options, SaveOptions};
use crate::{Document, PersistError};

use super::catalog::{
    generation_path_for_document, load_catalog, rotate_generations, save_catalog,
    GenerationCatalog, PinGenerationOptions, RotateOptions,
};
use super::format::{
    append_frame, encode_frame, journal_path_for_document, read_or_create_header, scan_journal,
    truncate_journal, JournalFrame, JournalHeader, JournalRecordKind, JournalScanOutcome,
    ScanJournalOptions, HEADER_LEN,
};
use super::replay::{
    edit_payload, replay_journal, snapshot_payload, JournalEdit, ReplayOptions, ReplayOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    MainFile,
    JournalReplay,
    SnapshotFallback,
    TruncatedJournalThenReplay,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenProjectOutcome {
    pub document: Document,
    pub source: RecoverySource,
    pub truncated_bytes: u64,
    pub replay: Option<ReplayOutcome>,
}

#[derive(Debug, Clone, Default)]
pub struct SaveProjectOptions {
    pub persist: SaveOptions,
    pub journal_edit: Option<JournalEdit>,
    pub snapshot_every_n_edits: Option<u32>,
    /// true ならこの保存ではスナップショット世代を作らない(テール注入用)。
    pub skip_snapshot: bool,
    pub pin_generation: Option<PinGenerationOptions>,
    pub rotate: Option<RotateOptions>,
    pub max_unpinned_generations: Option<u32>,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Journal(#[from] super::format::JournalFormatError),
    #[error(transparent)]
    Catalog(#[from] super::catalog::CatalogError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("main document missing at {0}")]
    MissingMainDocument(PathBuf),
}

fn new_journal_salt(document_path: &Path) -> Result<u64, ProjectError> {
    if let Some(catalog) = load_catalog(document_path)? {
        return Ok(catalog.journal_salt);
    }
    let journal_path = journal_path_for_document(document_path);
    if journal_path.exists() {
        let data = fs::read(&journal_path)?;
        if data.len() >= super::format::HEADER_LEN {
            return Ok(super::format::read_header(&data)?.file_salt);
        }
    }
    Ok(Uuid::new_v4().as_u128() as u64)
}

struct ProjectLayout {
    journal_path: PathBuf,
    catalog: GenerationCatalog,
    header: JournalHeader,
    last_record: Option<Uuid>,
}

fn ensure_layout(document_path: &Path, file_salt: u64) -> Result<ProjectLayout, ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let header = read_or_create_header(&journal_path, file_salt)?;
    let catalog = match load_catalog(document_path)? {
        Some(mut c) => {
            c.journal_salt = header.file_salt;
            c
        }
        None => GenerationCatalog::new(
            header.file_salt,
            5, // デフォルト — SaveProjectOptions で上書き可
        ),
    };
    let last_record = scan_journal(&journal_path, &ScanJournalOptions::default())
        .ok()
        .and_then(|s| s.frames.last().map(|f| f.record_id));
    Ok(ProjectLayout {
        journal_path,
        catalog,
        header,
        last_record,
    })
}

fn write_generation_snapshot(
    document_path: &Path,
    generation_id: Uuid,
    doc: &Document,
) -> Result<(), ProjectError> {
    let path = generation_path_for_document(document_path, generation_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    save_document(&path, doc)?;
    Ok(())
}

fn push_frame(
    layout: &mut ProjectLayout,
    kind: JournalRecordKind,
    snapshot_ref: Option<Uuid>,
    payload: Vec<u8>,
) -> Result<Uuid, ProjectError> {
    let record_id = Uuid::new_v4();
    let frame = JournalFrame {
        record_id,
        prev_id: layout.last_record,
        snapshot_ref,
        record_salt: layout.header.file_salt,
        kind,
        payload,
    };
    append_frame(&layout.journal_path, &frame)?;
    layout.last_record = Some(record_id);
    Ok(record_id)
}

/// D1c の atomic save の後にジャーナル追記・世代管理を行う。
pub fn save_project_with_journal(
    document_path: &Path,
    doc: &Document,
    options: &SaveProjectOptions,
) -> Result<(), ProjectError> {
    save_document_with_options(document_path, doc, &options.persist)?;

    let mut layout = ensure_layout(document_path, new_journal_salt(document_path)?)?;
    if let Some(max) = options.max_unpinned_generations {
        layout.catalog.max_unpinned = max;
    }

    let mut edits_since_snapshot = layout
        .catalog
        .generations
        .len()
        .saturating_sub(1) as u32;

    if let Some(edit) = &options.journal_edit {
        let payload = edit_payload(edit)?;
        push_frame(
            &mut layout,
            JournalRecordKind::Edit,
            None,
            payload,
        )?;
        edits_since_snapshot += 1;
    }

    let snapshot_interval = options.snapshot_every_n_edits.unwrap_or(0);
    let need_snapshot = !options.skip_snapshot
        && snapshot_interval > 0
        && edits_since_snapshot >= snapshot_interval;
    if need_snapshot || (!options.skip_snapshot
        && options.journal_edit.is_none()
        && layout.catalog.generations.is_empty())
    {
        let generation_id = Uuid::new_v4();
        write_generation_snapshot(document_path, generation_id, doc)?;
        let payload = snapshot_payload(generation_id);
        let journal_record = push_frame(
            &mut layout,
            JournalRecordKind::Snapshot,
            Some(generation_id),
            payload,
        )?;
        layout
            .catalog
            .register_generation(generation_id, journal_record, false);
    }

    if let Some(pin) = &options.pin_generation {
        layout.catalog.pin_generation(pin.generation_id)?;
        let payload = serde_json::to_vec(&PinPayload {
            generation_id: pin.generation_id,
        })?;
        push_frame(
            &mut layout,
            JournalRecordKind::PinGeneration,
            Some(pin.generation_id),
            payload,
        )?;
    }

    if let Some(rotate) = &options.rotate {
        rotate_generations(document_path, &mut layout.catalog, rotate)?;
    }

    save_catalog(document_path, &layout.catalog)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct PinPayload {
    generation_id: Uuid,
}

fn load_main_or_missing(document_path: &Path) -> Result<Option<Document>, ProjectError> {
    if !document_path.exists() {
        return Ok(None);
    }
    Ok(Some(crate::load_document(document_path)?))
}

fn heal_journal(document_path: &Path) -> Result<(JournalScanOutcome, u64), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    if !journal_path.exists() {
        return Err(ProjectError::Journal(
            super::format::JournalFormatError::PartialFrame(0),
        ));
    }
    let data = fs::read(&journal_path)?;
    let before_len = data.len() as u64;
    let scan = super::format::scan_journal_bytes(
        &data,
        &ScanJournalOptions::default(),
    )?;
    let truncated = if scan.valid_bytes < before_len {
        truncate_journal(&journal_path, scan.valid_bytes)?;
        before_len - scan.valid_bytes
    } else {
        0
    };
    Ok((scan, truncated))
}

/// 本体 JSON + ジャーナルリプレイ + スナップショットフォールバックで復元する。
pub fn open_project(document_path: &Path) -> Result<OpenProjectOutcome, ProjectError> {
    let base = match load_main_or_missing(document_path)? {
        Some(doc) => doc,
        None => return Err(ProjectError::MissingMainDocument(document_path.to_path_buf())),
    };

    let journal_path = journal_path_for_document(document_path);
    if !journal_path.exists() {
        return Ok(OpenProjectOutcome {
            document: base,
            source: RecoverySource::MainFile,
            truncated_bytes: 0,
            replay: None,
        });
    }

    let (scan, truncated) = match heal_journal(document_path) {
        Ok(v) => v,
        Err(ProjectError::Journal(_)) => {
            return Ok(OpenProjectOutcome {
                document: base,
                source: RecoverySource::MainFile,
                truncated_bytes: 0,
                replay: None,
            });
        }
        Err(e) => return Err(e),
    };

    if scan.frames.is_empty() {
        return Ok(OpenProjectOutcome {
            document: base,
            source: if truncated > 0 {
                RecoverySource::TruncatedJournalThenReplay
            } else {
                RecoverySource::MainFile
            },
            truncated_bytes: truncated,
            replay: None,
        });
    }

    let catalog = load_catalog(document_path)?.unwrap_or_else(|| {
        GenerationCatalog::new(scan.header.file_salt, 5)
    });

    let replay = replay_journal(
        document_path,
        base.clone(),
        &scan,
        &catalog,
        &ReplayOptions {
            fallback_on_failure: true,
        },
    );

    let source = if !replay.replay_failures.is_empty() {
        RecoverySource::SnapshotFallback
    } else if truncated > 0 {
        RecoverySource::TruncatedJournalThenReplay
    } else if scan.frames.iter().any(|f| f.kind == JournalRecordKind::Edit) {
        RecoverySource::JournalReplay
    } else {
        RecoverySource::MainFile
    };

    Ok(OpenProjectOutcome {
        document: replay.document.clone(),
        source,
        truncated_bytes: truncated,
        replay: Some(replay),
    })
}

/// テスト注入: ジャーナル末尾に壊れたバイト列を付与する。
#[doc(hidden)]
pub fn inject_corrupt_journal_tail(document_path: &Path, garbage: &[u8]) -> Result<(), ProjectError> {
    use std::io::Write;
    let journal_path = journal_path_for_document(document_path);
    let mut file = fs::OpenOptions::new().append(true).open(journal_path)?;
    file.write_all(garbage)?;
    file.sync_all()?;
    Ok(())
}

/// テスト注入: 特定フレームの checksum を破壊する。
#[doc(hidden)]
pub fn inject_bad_checksum_at_last_frame(document_path: &Path) -> Result<(), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let data = fs::read(&journal_path)?;
    let scan = super::format::scan_journal_bytes(
        &data,
        &ScanJournalOptions::default(),
    )?;
    if scan.frames.is_empty() {
        return Ok(());
    }
    let mut offset = HEADER_LEN;
    for (index, frame) in scan.frames.iter().enumerate() {
        if index + 1 == scan.frames.len() {
            let mut corrupted = data;
            corrupted[offset] ^= 0xFF;
            fs::write(journal_path, corrupted)?;
            return Ok(());
        }
        offset += encode_frame(frame).len();
    }
    Ok(())
}

/// テスト注入: 旧 salt のフレームを末尾に追記する(WAL 世代不一致)。
#[doc(hidden)]
pub fn inject_salt_mismatch_frame(document_path: &Path) -> Result<(), ProjectError> {
    let journal_path = journal_path_for_document(document_path);
    let scan = scan_journal(&journal_path, &ScanJournalOptions::default())?;
    let bad_frame = JournalFrame {
        record_id: Uuid::new_v4(),
        prev_id: scan.frames.last().map(|f| f.record_id),
        snapshot_ref: None,
        record_salt: scan.header.file_salt.wrapping_add(1),
        kind: JournalRecordKind::Edit,
        payload: br#"{"op":"set_bpm","num":1,"den":1}"#.to_vec(),
    };
    append_frame(&journal_path, &bad_frame)?;
    Ok(())
}

#[cfg(test)]
mod project_tests {
    use super::*;
    use crate::{Bpm, Document, JournalEdit, SaveProjectOptions};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-d1d-unit-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn clean_journal_scans_without_truncation() {
        let dir = dir();
        let path = dir.join("doc.json");
        let mut doc = Document::new_v1();
        save_project_with_journal(
            &path,
            &doc,
            &SaveProjectOptions {
                snapshot_every_n_edits: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        doc.bpm = Bpm::try_new(150, 1).unwrap();
        save_project_with_journal(
            &path,
            &doc,
            &SaveProjectOptions {
                journal_edit: Some(JournalEdit::SetBpm { num: 150, den: 1 }),
                snapshot_every_n_edits: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        let journal_path = journal_path_for_document(&path);
        let len = fs::metadata(&journal_path).unwrap().len();
        let scan = scan_journal(&journal_path, &ScanJournalOptions::default()).unwrap();
        assert_eq!(scan.stopped, None, "unexpected stop: {:?}", scan.stopped);
        assert_eq!(scan.valid_bytes, len);
        let _ = fs::remove_dir_all(dir);
    }
}
