//! リプレイ前の restore_attempted マーカーと毒 journal の隔離(実装ガード4)。
//!
//! 前回起動がリプレイ途中で落ちた場合、同じ tip のマーカーが残っていれば
//! 再試行せず隔離し、世代/main へフォールバックする。
//!
//! マーカー書き込みは D1c と同型: temp 書き込み → file fsync → rename → dir fsync。

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::catalog::motolii_dir_for_document;
use super::format::{journal_path_for_document, JournalScanOutcome};

pub const RESTORE_ATTEMPTED_FILENAME: &str = "restore_attempted.json";
pub const QUARANTINE_DIRNAME: &str = "journal.quarantine";
pub const MARKER_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreAttemptMarker {
    pub format_version: u32,
    pub file_salt: u64,
    pub tip_record_id: Option<Uuid>,
    pub valid_bytes: u64,
}

pub fn restore_attempted_path_for_document(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(RESTORE_ATTEMPTED_FILENAME)
}

pub fn quarantine_dir_for_document(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join(QUARANTINE_DIRNAME)
}

pub fn marker_from_scan(scan: &JournalScanOutcome) -> RestoreAttemptMarker {
    RestoreAttemptMarker {
        format_version: MARKER_FORMAT_VERSION,
        file_salt: scan.header.file_salt,
        tip_record_id: scan.frames.last().map(|f| f.record_id),
        valid_bytes: scan.valid_bytes,
    }
}

pub fn marker_matches_scan(marker: &RestoreAttemptMarker, scan: &JournalScanOutcome) -> bool {
    marker.file_salt == scan.header.file_salt
        && marker.valid_bytes == scan.valid_bytes
        && marker.tip_record_id == scan.frames.last().map(|f| f.record_id)
}

pub fn load_restore_attempted(
    document_path: &Path,
) -> Result<Option<RestoreAttemptMarker>, io::Error> {
    let path = restore_attempted_path_for_document(document_path);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path)?;
    match serde_json::from_slice(&bytes) {
        Ok(marker) => Ok(Some(marker)),
        Err(_) => {
            // 壊れたマーカーは無視して消す(再試行ループの種にしない)
            let _ = fs::remove_file(&path);
            Ok(None)
        }
    }
}

pub fn write_restore_attempted(
    document_path: &Path,
    scan: &JournalScanOutcome,
) -> Result<(), io::Error> {
    let dir = motolii_dir_for_document(document_path);
    fs::create_dir_all(&dir)?;
    let path = restore_attempted_path_for_document(document_path);
    let marker = marker_from_scan(scan);
    let bytes = serde_json::to_vec_pretty(&marker).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");

    let mut file = File::create(&tmp)?;
    file.write_all(&bytes)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    fs::rename(&tmp, &path)?;
    sync_dir(&dir)?;
    Ok(())
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let dir_file = File::open(dir)?;
        dir_file.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        // D1c と同じく非 Unix ではディレクトリ fsync を省略する。
        let _ = dir;
    }
    Ok(())
}

pub fn clear_restore_attempted(document_path: &Path) -> Result<(), io::Error> {
    let path = restore_attempted_path_for_document(document_path);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// `journal.wal` を `.motolii/journal.quarantine/journal.wal.corrupt.<ts>` へ移す。
pub fn quarantine_journal(document_path: &Path) -> Result<Option<PathBuf>, io::Error> {
    let journal_path = journal_path_for_document(document_path);
    if !journal_path.exists() {
        return Ok(None);
    }
    let qdir = quarantine_dir_for_document(document_path);
    fs::create_dir_all(&qdir)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dest = qdir.join(format!("journal.wal.corrupt.{ts}"));
    fs::rename(&journal_path, &dest)?;
    Ok(Some(dest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::format::{
        append_frame, encode_header, scan_journal, JournalFrame, JournalHeader, JournalRecordKind,
        ScanJournalOptions, HEADER_LEN, JOURNAL_FORMAT_VERSION,
    };
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;

    fn unique_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("motolii-restore-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn restore_marker_write_fsync_roundtrip() {
        let dir = unique_dir();
        let doc = dir.join("doc.json");
        fs::write(&doc, b"{}").unwrap();
        let journal = journal_path_for_document(&doc);
        fs::create_dir_all(journal.parent().unwrap()).unwrap();
        let header = JournalHeader {
            version: JOURNAL_FORMAT_VERSION,
            file_salt: 42,
        };
        fs::write(&journal, encode_header(&header)).unwrap();
        let frame = JournalFrame {
            record_id: Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: None,
            record_salt: 42,
            kind: JournalRecordKind::Edit,
            payload: b"{}".to_vec(),
        };
        append_frame(&journal, &frame).unwrap();
        let scan = scan_journal(&journal, &ScanJournalOptions::default()).unwrap();
        assert!(scan.valid_bytes > HEADER_LEN as u64);

        write_restore_attempted(&doc, &scan).unwrap();
        let path = restore_attempted_path_for_document(&doc);
        assert!(path.exists());
        // rename 後に tmp が残っていないこと
        assert!(!path.with_extension("json.tmp").exists());
        let loaded = load_restore_attempted(&doc).unwrap().expect("marker");
        assert!(marker_matches_scan(&loaded, &scan));
        let _ = fs::remove_dir_all(dir);
    }
}
