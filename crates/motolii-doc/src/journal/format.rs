//! バイナリジャーナル形式。SQLite WAL フレームに倣い checksum + salt で検証する。

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

use crc32fast::Hasher;
use thiserror::Error;
use uuid::Uuid;

pub const JOURNAL_MAGIC: &[u8; 8] = b"MOTOLIIJ";
pub const JOURNAL_FORMAT_VERSION: u32 = 1;
pub const HEADER_LEN: usize = 32;
/// フレーム固定部(checksum除く) = kind(1)+reserved(3)+ids+salt+payload_len
pub const FRAME_PREFIX_LEN: usize = 4 + 16 + 16 + 16 + 8 + 1 + 3 + 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JournalRecordKind {
    Snapshot = 1,
    Edit = 2,
    PinGeneration = 3,
}

impl JournalRecordKind {
    pub fn try_from_u8(v: u8) -> Result<Self, JournalFormatError> {
        match v {
            1 => Ok(Self::Snapshot),
            2 => Ok(Self::Edit),
            3 => Ok(Self::PinGeneration),
            _ => Err(JournalFormatError::UnknownKind(v)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalHeader {
    pub version: u32,
    pub file_salt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFrame {
    pub record_id: Uuid,
    pub prev_id: Option<Uuid>,
    pub snapshot_ref: Option<Uuid>,
    pub record_salt: u64,
    pub kind: JournalRecordKind,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalScanStop {
    ChecksumMismatch,
    SaltMismatch,
    UnknownKind(u8),
    PartialFrame,
    InvalidPrevChain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalScanOutcome {
    pub header: JournalHeader,
    pub frames: Vec<JournalFrame>,
    pub valid_bytes: u64,
    pub stopped: Option<JournalScanStop>,
}

#[derive(Debug, Clone, Default)]
pub struct ScanJournalOptions {
    /// true なら prev_id 鎖の欠落も停止理由にする(WAL の順序破損相当)。
    pub verify_prev_chain: bool,
}

#[derive(Debug, Error)]
pub enum JournalFormatError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("journal magic mismatch")]
    BadMagic,
    #[error("unsupported journal format version {0}")]
    UnsupportedVersion(u32),
    #[error("unknown journal record kind {0}")]
    UnknownKind(u8),
    #[error("journal frame checksum mismatch at offset {offset}")]
    ChecksumMismatch { offset: u64 },
    #[error("journal record salt {record_salt} does not match file salt {file_salt} at offset {offset}")]
    SaltMismatch {
        record_salt: u64,
        file_salt: u64,
        offset: u64,
    },
    #[error("partial journal frame at offset {0}")]
    PartialFrame(u64),
    #[error("journal prev_id chain broken at record {record_id}")]
    BrokenPrevChain { record_id: Uuid },
}

pub fn journal_path_for_document(document_path: &Path) -> std::path::PathBuf {
    document_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(".motolii")
        .join("journal.wal")
}

pub fn read_header(data: &[u8]) -> Result<JournalHeader, JournalFormatError> {
    if data.len() < HEADER_LEN {
        return Err(JournalFormatError::PartialFrame(0));
    }
    if &data[..8] != JOURNAL_MAGIC {
        return Err(JournalFormatError::BadMagic);
    }
    let version = u32::from_le_bytes(data[8..12].try_into().expect("version"));
    if version != JOURNAL_FORMAT_VERSION {
        return Err(JournalFormatError::UnsupportedVersion(version));
    }
    let file_salt = u64::from_le_bytes(data[12..20].try_into().expect("salt"));
    Ok(JournalHeader {
        version,
        file_salt,
    })
}

pub fn encode_header(header: &JournalHeader) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[..8].copy_from_slice(JOURNAL_MAGIC);
    out[8..12].copy_from_slice(&header.version.to_le_bytes());
    out[12..20].copy_from_slice(&header.file_salt.to_le_bytes());
    out
}

fn frame_checksum(body: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(body);
    hasher.finalize()
}

fn uuid_from_bytes(bytes: [u8; 16]) -> Option<Uuid> {
    if bytes == [0u8; 16] {
        None
    } else {
        Some(Uuid::from_bytes(bytes))
    }
}

fn uuid_to_bytes(id: Option<Uuid>) -> [u8; 16] {
    id.map(|u| *u.as_bytes()).unwrap_or([0u8; 16])
}

pub fn encode_frame(frame: &JournalFrame) -> Vec<u8> {
    let mut body = Vec::with_capacity(FRAME_PREFIX_LEN + frame.payload.len());
    body.extend_from_slice(&0u32.to_le_bytes()); // checksum placeholder
    body.extend_from_slice(frame.record_id.as_bytes());
    body.extend_from_slice(&uuid_to_bytes(frame.prev_id));
    body.extend_from_slice(&uuid_to_bytes(frame.snapshot_ref));
    body.extend_from_slice(&frame.record_salt.to_le_bytes());
    body.push(frame.kind as u8);
    body.extend_from_slice(&[0u8; 3]);
    body.extend_from_slice(&(frame.payload.len() as u32).to_le_bytes());
    body.extend_from_slice(&frame.payload);
    let frame_len = body.len();
    let checksum = frame_checksum(&body[4..frame_len]);
    body[..4].copy_from_slice(&checksum.to_le_bytes());
    body
}

fn decode_frame(
    data: &[u8],
    offset: u64,
    file_salt: u64,
    expected_prev: Option<Uuid>,
    verify_prev_chain: bool,
) -> Result<(JournalFrame, usize), JournalFormatError> {
    if data.len() < FRAME_PREFIX_LEN {
        return Err(JournalFormatError::PartialFrame(offset));
    }
    let payload_len = u32::from_le_bytes(data[64..68].try_into().expect("payload_len")) as usize;
    let total = FRAME_PREFIX_LEN + payload_len;
    if data.len() < total {
        return Err(JournalFormatError::PartialFrame(offset));
    }
    let stored_checksum = u32::from_le_bytes(data[0..4].try_into().expect("checksum"));
    let computed = frame_checksum(&data[4..total]);
    if stored_checksum != computed {
        return Err(JournalFormatError::ChecksumMismatch { offset });
    }

    let record_id = Uuid::from_bytes(data[4..20].try_into().expect("record_id"));
    let prev_id = uuid_from_bytes(data[20..36].try_into().expect("prev"));
    let snapshot_ref = uuid_from_bytes(data[36..52].try_into().expect("snap"));
    let record_salt = u64::from_le_bytes(data[52..60].try_into().expect("record_salt"));
    if record_salt != file_salt {
        return Err(JournalFormatError::SaltMismatch {
            record_salt,
            file_salt,
            offset,
        });
    }
    let kind = JournalRecordKind::try_from_u8(data[60])?;
    if verify_prev_chain && prev_id != expected_prev {
        return Err(JournalFormatError::BrokenPrevChain { record_id });
    }
    let payload = data[FRAME_PREFIX_LEN..total].to_vec();
    Ok((
        JournalFrame {
            record_id,
            prev_id,
            snapshot_ref,
            record_salt,
            kind,
            payload,
        },
        total,
    ))
}

pub fn scan_journal_bytes(
    data: &[u8],
    options: &ScanJournalOptions,
) -> Result<JournalScanOutcome, JournalFormatError> {
    if data.is_empty() {
        return Err(JournalFormatError::PartialFrame(0));
    }
    let header = read_header(data)?;
    let mut offset = HEADER_LEN;
    let mut frames = Vec::new();
    let mut stopped = None;
    let mut expected_prev = None;

    while offset < data.len() {
        match decode_frame(
            &data[offset..],
            offset as u64,
            header.file_salt,
            expected_prev,
            options.verify_prev_chain,
        ) {
            Ok((frame, consumed)) => {
                expected_prev = Some(frame.record_id);
                frames.push(frame);
                offset += consumed;
            }
            Err(JournalFormatError::PartialFrame(off)) => {
                stopped = Some(JournalScanStop::PartialFrame);
                let _ = off;
                break;
            }
            Err(JournalFormatError::ChecksumMismatch { offset: off }) => {
                stopped = Some(JournalScanStop::ChecksumMismatch);
                let _ = off;
                break;
            }
            Err(JournalFormatError::SaltMismatch { .. }) => {
                stopped = Some(JournalScanStop::SaltMismatch);
                break;
            }
            Err(JournalFormatError::BrokenPrevChain { .. }) => {
                stopped = Some(JournalScanStop::InvalidPrevChain);
                break;
            }
            Err(JournalFormatError::UnknownKind(k)) => {
                stopped = Some(JournalScanStop::UnknownKind(k));
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(JournalScanOutcome {
        header,
        frames,
        valid_bytes: offset as u64,
        stopped,
    })
}

pub fn scan_journal(path: &Path, options: &ScanJournalOptions) -> Result<JournalScanOutcome, JournalFormatError> {
    let data = std::fs::read(path)?;
    scan_journal_bytes(&data, options)
}

/// 最後の正当フレーム境界でファイルを切り詰める(SQLite WAL の hot journal 回収相当)。
pub fn truncate_journal(path: &Path, valid_bytes: u64) -> Result<(), JournalFormatError> {
    let file = OpenOptions::new().write(true).open(path)?;
    file.set_len(valid_bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn init_journal_file(path: &Path, file_salt: u64) -> Result<(), JournalFormatError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let header = JournalHeader {
        version: JOURNAL_FORMAT_VERSION,
        file_salt,
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(&encode_header(&header))?;
    file.sync_all()?;
    Ok(())
}

pub fn append_frame(path: &Path, frame: &JournalFrame) -> Result<(), JournalFormatError> {
    let mut file = OpenOptions::new().append(true).open(path)?;
    let bytes = encode_frame(frame);
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn read_or_create_header(path: &Path, file_salt: u64) -> Result<JournalHeader, JournalFormatError> {
    if path.exists() {
        let data = std::fs::read(path)?;
        read_header(&data)
    } else {
        init_journal_file(path, file_salt)?;
        Ok(JournalHeader {
            version: JOURNAL_FORMAT_VERSION,
            file_salt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_two_frames_appended() {
        let header = JournalHeader {
            version: JOURNAL_FORMAT_VERSION,
            file_salt: 42,
        };
        let f1 = JournalFrame {
            record_id: Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: Some(Uuid::new_v4()),
            record_salt: 42,
            kind: JournalRecordKind::Snapshot,
            payload: br#"{"generation_id":"550e8400-e29b-41d4-a716-446655440000"}"#.to_vec(),
        };
        let f2 = JournalFrame {
            record_id: Uuid::new_v4(),
            prev_id: Some(f1.record_id),
            snapshot_ref: None,
            record_salt: 42,
            kind: JournalRecordKind::Edit,
            payload: br#"{"op":"set_bpm","num":150,"den":1}"#.to_vec(),
        };
        let mut data = encode_header(&header).to_vec();
        data.extend_from_slice(&encode_frame(&f1));
        data.extend_from_slice(&encode_frame(&f2));
        let outcome = scan_journal_bytes(&data, &ScanJournalOptions::default()).unwrap();
        assert_eq!(outcome.frames.len(), 2, "stop={:?}", outcome.stopped);
        assert!(outcome.stopped.is_none());
    }

    #[test]
    fn roundtrip_single_frame() {
        let frame = JournalFrame {
            record_id: Uuid::new_v4(),
            prev_id: None,
            snapshot_ref: Some(Uuid::new_v4()),
            record_salt: 42,
            kind: JournalRecordKind::Snapshot,
            payload: br#"{"generation":"abc"}"#.to_vec(),
        };
        let encoded = encode_frame(&frame);
        let header = JournalHeader {
            version: JOURNAL_FORMAT_VERSION,
            file_salt: 42,
        };
        let mut data = encode_header(&header).to_vec();
        data.extend_from_slice(&encoded);
        let outcome = scan_journal_bytes(&data, &ScanJournalOptions::default()).unwrap();
        assert_eq!(outcome.frames.len(), 1);
        assert_eq!(outcome.frames[0], frame);
        assert!(outcome.stopped.is_none());
    }
}
