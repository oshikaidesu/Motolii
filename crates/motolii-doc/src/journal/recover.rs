//! 非破壊recovery(監査S15)。
//!
//! - 原本(`document.json`)と`journal.wal`を直接上書き/truncateしない
//! - 成果物は`*.recovered-<ts>` / `*.corrupt-<ts>`へ分離
//! - 不正テールはscanの論理停止のみ(valid_bytes)

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::limits::ResourceLimits;
use crate::{Document, PersistError};

use super::catalog::{
    catalog_path_for_document, generation_path_for_document, load_catalog_fs, GenerationCatalog,
};
use super::format::{
    journal_path_for_document, motolii_dir_for_document, scan_journal_bytes, scan_journal_fs,
    JournalScanOutcome, JournalScanStop, ScanJournalOptions,
};
use super::fs::{DurabilityStage, FsError, JournalFs};
use super::replay::{
    document_fingerprint, frames_through_last_commit, load_generation_via_fs, replay_from_base,
    ReplayFailure, ReplayOutcome,
};

pub const RESTORE_ATTEMPTED_FILENAME: &str = "restore_attempted.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    MainFile,
    JournalReplay,
    SnapshotFallback,
    /// テール破損を論理無視してcommit済み範囲をリプレイ
    CommittedPrefixReplay,
    GenerationRecovery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryResult {
    pub document: Document,
    pub source: RecoverySource,
    pub ignored_tail_bytes: u64,
    pub recovered_path: Option<PathBuf>,
    pub corrupt_path: Option<PathBuf>,
    pub replay: Option<ReplayOutcome>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreAttemptMarker {
    pub format_version: u32,
    pub project_id: Uuid,
    pub generation_salt: u64,
    pub tip_record_id: Option<Uuid>,
    pub valid_bytes: u64,
}

#[derive(Debug, Error)]
pub enum RecoveryError {
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
    #[error("main document missing and no recoverable journal/generation at {path}")]
    Unrecoverable { path: PathBuf },
}

fn stamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

pub fn restore_attempted_path(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(RESTORE_ATTEMPTED_FILENAME)
}

pub fn recovered_document_path(document_path: &Path) -> PathBuf {
    let stem = document_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let parent = document_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}.recovered-{}.json", stamp()))
}

pub fn corrupt_journal_path(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(format!("journal.wal.corrupt-{}", stamp()))
}

fn load_marker(
    fs: &mut dyn JournalFs,
    document_path: &Path,
) -> Result<Option<RestoreAttemptMarker>, RecoveryError> {
    let path = restore_attempted_path(document_path);
    if !fs.exists(&path) {
        return Ok(None);
    }
    let bytes = fs.read(&path)?;
    match serde_json::from_slice(&bytes) {
        Ok(m) => Ok(Some(m)),
        Err(_) => {
            // 壊れたマーカーは無視(再試行ループの種にしない)。削除はしない —
            // 原本上書き回避のためcorruptへコピーだけ試みる。
            Ok(None)
        }
    }
}

fn write_marker(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    scan: &JournalScanOutcome,
) -> Result<(), RecoveryError> {
    let dir = motolii_dir_for_document(document_path);
    fs.create_dir_all(&dir)?;
    let path = restore_attempted_path(document_path);
    let marker = RestoreAttemptMarker {
        format_version: 1,
        project_id: scan.header.project_id,
        generation_salt: scan.header.generation_salt,
        tip_record_id: scan.frames.last().map(|f| f.record_id),
        valid_bytes: scan.valid_bytes,
    };
    let bytes = serde_json::to_vec_pretty(&marker)?;
    let tmp = path.with_extension("json.tmp");
    fs.write_create(&tmp, &bytes)?;
    fs.sync_file(&tmp)?;
    fs.rename(&tmp, &path)?;
    fs.sync_dir(&dir)?;
    fs.note_stage(DurabilityStage::RecoveryWrite)?;
    Ok(())
}

fn clear_marker(fs: &mut dyn JournalFs, document_path: &Path) -> Result<(), RecoveryError> {
    let path = restore_attempted_path(document_path);
    if fs.exists(&path) {
        // マーカー削除は空書き+renameではなく、corruptへ退避してから忘れる
        let dest = motolii_dir_for_document(document_path)
            .join(format!("restore_attempted.cleared-{}", stamp()));
        let bytes = fs.read(&path)?;
        fs.write_create(&dest, &bytes)?;
        // 空ファイルで「クリア」を表現(原本パスをtruncateしない — 上書きは空内容)
        fs.write_create(&path, b"{}")?;
        fs.sync_file(&path)?;
    }
    Ok(())
}

fn copy_journal_to_corrupt(
    fs: &mut dyn JournalFs,
    document_path: &Path,
) -> Result<PathBuf, RecoveryError> {
    let src = journal_path_for_document(document_path);
    let dest = corrupt_journal_path(document_path);
    let bytes = fs.read(&src)?;
    if let Some(parent) = dest.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.write_create(&dest, &bytes)?;
    fs.sync_file(&dest)?;
    Ok(dest)
}

fn write_recovered_doc(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    doc: &Document,
) -> Result<PathBuf, RecoveryError> {
    let path = recovered_document_path(document_path);
    let bytes = serde_json::to_vec_pretty(doc)?;
    if let Some(parent) = path.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.note_stage(DurabilityStage::RecoveryWrite)?;
    fs.write_create(&path, &bytes)?;
    fs.sync_file(&path)?;
    fs.note_stage(DurabilityStage::RecoveryFsync)?;
    Ok(path)
}

fn try_load_main(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<Option<Document>, RecoveryError> {
    if !fs.exists(document_path) {
        return Ok(None);
    }
    let bytes = fs.read(document_path)?;
    match crate::load_document_bytes_with_limits(&bytes, limits) {
        Ok(opened) => Ok(Some(opened.document)),
        Err(_) => Ok(None),
    }
}

fn marker_matches(marker: &RestoreAttemptMarker, scan: &JournalScanOutcome) -> bool {
    marker.project_id == scan.header.project_id
        && marker.generation_salt == scan.header.generation_salt
        && marker.valid_bytes == scan.valid_bytes
        && marker.tip_record_id == scan.frames.last().map(|f| f.record_id)
}

/// 非破壊でプロジェクトを開く/回復する。
pub fn recover_project(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    limits: &ResourceLimits,
) -> Result<RecoveryResult, RecoveryError> {
    let catalog = load_catalog_fs(fs, document_path)?.unwrap_or_else(|| {
        GenerationCatalog::new(Uuid::nil(), 0, 5)
    });
    let main = try_load_main(fs, document_path, limits)?;
    let journal_path = journal_path_for_document(document_path);

    if !fs.exists(&journal_path) {
        let doc = main.ok_or_else(|| RecoveryError::Unrecoverable {
            path: document_path.to_path_buf(),
        })?;
        return Ok(RecoveryResult {
            document: doc,
            source: RecoverySource::MainFile,
            ignored_tail_bytes: 0,
            recovered_path: None,
            corrupt_path: None,
            replay: None,
            warnings: Vec::new(),
        });
    }

    let jlen = fs.metadata_len(&journal_path)?;
    limits
        .check_journal_bytes(jlen)
        .map_err(PersistError::from)?;

    let options = ScanJournalOptions {
        verify_prev_chain: true,
        expected_project_id: if catalog.project_id.is_nil() {
            None
        } else {
            Some(catalog.project_id)
        },
    };
    let scan = scan_journal_fs(fs, &journal_path, &options)?;
    let ignored = scan.ignored_tail_bytes();

    // recovery中の再crash検知: 同一tipのマーカーが残っていれば再リプレイしない
    let prior = load_marker(fs, document_path)?;
    if let Some(marker) = prior {
        if marker_matches(&marker, &scan) {
            let corrupt_path = copy_journal_to_corrupt(fs, document_path).ok();
            let (doc, source) = fallback_document(fs, document_path, main, &catalog)?;
            let recovered_path = if source != RecoverySource::MainFile {
                Some(write_recovered_doc(fs, document_path, &doc)?)
            } else {
                None
            };
            let _ = clear_marker(fs, document_path);
            return Ok(RecoveryResult {
                document: doc,
                source,
                ignored_tail_bytes: ignored,
                recovered_path,
                corrupt_path,
                replay: None,
                warnings: vec![
                    "restore_attempted marker matched tip; skipped replay to avoid crash loop"
                        .into(),
                ],
            });
        }
    }

    let committed = frames_through_last_commit(&scan.frames);
    let committed_scan = JournalScanOutcome {
        header: scan.header.clone(),
        frames: committed.to_vec(),
        valid_bytes: scan.valid_bytes,
        file_len: scan.file_len,
        stopped: scan.stopped.clone(),
    };

    let main_fp = main.as_ref().map(document_fingerprint);
    let journal_fp = catalog.last_journaled_fingerprint;

    // tipとmainが一致し、かつcheckpoint以降の未反映Editが無いときだけmainを採用。
    if let (Some(m), Some(j)) = (main_fp, journal_fp) {
        if m == j && scan.stopped.is_none() && catalog.edits_since_snapshot == 0 {
            return Ok(RecoveryResult {
                document: main.expect("main present"),
                source: RecoverySource::MainFile,
                ignored_tail_bytes: ignored,
                recovered_path: None,
                corrupt_path: None,
                replay: None,
                warnings: Vec::new(),
            });
        }
    }

    write_marker(fs, document_path, &scan)?;

    let base = if let Some(doc) = main.clone() {
        doc
    } else if let Some(gen) = catalog.latest_generation() {
        load_generation_via_fs(fs, &generation_path_for_document(document_path, gen.id))?
    } else {
        let _ = clear_marker(fs, document_path);
        return Err(RecoveryError::Unrecoverable {
            path: document_path.to_path_buf(),
        });
    };

    let mut load_snap = |gid: Uuid| {
        load_generation_via_fs(fs, &generation_path_for_document(document_path, gid)).map_err(
            |_| ReplayFailure::MissingSnapshot {
                generation_id: gid,
            },
        )
    };

    let replay = replay_from_base(base, &committed_scan, &mut load_snap, true);
    let _ = clear_marker(fs, document_path);

    let source = if replay.fallback_generation.is_some() {
        RecoverySource::SnapshotFallback
    } else if scan.stopped.is_some() {
        RecoverySource::CommittedPrefixReplay
    } else if main.is_none() {
        RecoverySource::GenerationRecovery
    } else {
        RecoverySource::JournalReplay
    };

    let need_recovered = match &main {
        Some(m) => m != &replay.document,
        None => true,
    };
    let recovered_path = if need_recovered {
        Some(write_recovered_doc(fs, document_path, &replay.document)?)
    } else {
        None
    };

    let corrupt_path = if matches!(
        scan.stopped,
        Some(
            JournalScanStop::ChecksumMismatch
                | JournalScanStop::SaltMismatch
                | JournalScanStop::PartialFrame
                | JournalScanStop::UnknownKind(_)
                | JournalScanStop::BrokenPrevChain
        )
    ) {
        copy_journal_to_corrupt(fs, document_path).ok()
    } else {
        None
    };

    let mut warnings = Vec::new();
    if ignored > 0 {
        warnings.push(format!(
            "ignored {ignored} trailing journal bytes without truncating original"
        ));
    }
    for f in &replay.replay_failures {
        warnings.push(format!("replay failure: {f:?}"));
    }

    Ok(RecoveryResult {
        document: replay.document.clone(),
        source,
        ignored_tail_bytes: ignored,
        recovered_path,
        corrupt_path,
        replay: Some(replay),
        warnings,
    })
}

fn fallback_document(
    fs: &mut dyn JournalFs,
    document_path: &Path,
    main: Option<Document>,
    catalog: &GenerationCatalog,
) -> Result<(Document, RecoverySource), RecoveryError> {
    if let Some(doc) = main {
        return Ok((doc, RecoverySource::MainFile));
    }
    if let Some(gen) = catalog.latest_generation() {
        let doc =
            load_generation_via_fs(fs, &generation_path_for_document(document_path, gen.id))?;
        return Ok((doc, RecoverySource::SnapshotFallback));
    }
    Err(RecoveryError::Unrecoverable {
        path: document_path.to_path_buf(),
    })
}

/// テスト用: バイト列をscanするだけの薄い入口。
#[allow(dead_code)]
pub fn scan_bytes_for_test(data: &[u8]) -> Result<JournalScanOutcome, super::format::JournalFormatError> {
    scan_journal_bytes(data, &ScanJournalOptions::default())
}

#[allow(dead_code)]
pub fn catalog_path(document_path: &Path) -> PathBuf {
    catalog_path_for_document(document_path)
}
