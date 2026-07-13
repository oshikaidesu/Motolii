//! バイナリジャーナル形式(SQLite WAL風: checksum + 世代salt + UUID相互参照)。
//!
//! **原本をtruncateしない** — 不正テールは`scan`が論理停止するだけ(監査S15)。

use std::path::{Path, PathBuf};

use crc32fast::Hasher;
use thiserror::Error;
use uuid::Uuid;

use super::fs::{FsError, JournalFs};

pub const JOURNAL_MAGIC: &[u8; 8] = b"MOTOLIIJ";
pub const JOURNAL_FORMAT_VERSION: u32 = 1;
pub const HEADER_LEN: usize = 48;
/// checksum(4)+ids(16*3)+salt(8)+kind(1)+pad(3)+payload_len(4)
pub const FRAME_PREFIX_LEN: usize = 4 + 16 + 16 + 16 + 8 + 1 + 3 + 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum JournalRecordKind {
    Snapshot = 1,
    Edit = 2,
    /// 直前までのフレームを耐久化したcommit record(SQLite commit相当)。
    Commit = 3,
    /// main保存後の世代salt更新マーカー。
    Checkpoint = 4,
}

impl JournalRecordKind {
    pub fn try_from_u8(v: u8) -> Result<Self, JournalFormatError> {
        match v {
            1 => Ok(Self::Snapshot),
            2 => Ok(Self::Edit),
            3 => Ok(Self::Commit),
            4 => Ok(Self::Checkpoint),
            _ => Err(JournalFormatError::UnknownKind(v)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalHeader {
    pub version: u32,
    pub generation_salt: u64,
    pub project_id: Uuid,
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
    BrokenPrevChain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalScanOutcome {
    pub header: JournalHeader,
    pub frames: Vec<JournalFrame>,
    /// 検証済みプレフィックス長。テール破損はこの先にあるがファイルは切らない。
    pub valid_bytes: u64,
    pub file_len: u64,
    pub stopped: Option<JournalScanStop>,
}

impl JournalScanOutcome {
    pub fn ignored_tail_bytes(&self) -> u64 {
        self.file_len.saturating_sub(self.valid_bytes)
    }
}

#[derive(Debug, Error)]
pub enum JournalFormatError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error("journal magic mismatch")]
    BadMagic,
    #[error("unsupported journal format version {0}")]
    UnsupportedVersion(u32),
    #[error("unknown journal record kind {0}")]
    UnknownKind(u8),
    #[error("journal frame checksum mismatch at offset {offset}")]
    ChecksumMismatch { offset: u64 },
    #[error(
        "journal record salt {record_salt} does not match generation salt {generation_salt} at offset {offset}"
    )]
    SaltMismatch {
        record_salt: u64,
        generation_salt: u64,
        offset: u64,
    },
    #[error("partial journal frame at offset {0}")]
    PartialFrame(u64),
    #[error("journal prev_id chain broken at record {record_id}")]
    BrokenPrevChain { record_id: Uuid },
    #[error("journal project_id mismatch: header={header} expected={expected}")]
    ProjectIdMismatch { header: Uuid, expected: Uuid },
}

pub fn motolii_dir_for_document(document_path: &Path) -> PathBuf {
    document_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(".motolii")
}

pub fn journal_path_for_document(document_path: &Path) -> PathBuf {
    motolii_dir_for_document(document_path).join("journal.wal")
}

pub fn encode_header(header: &JournalHeader) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[..8].copy_from_slice(JOURNAL_MAGIC);
    out[8..12].copy_from_slice(&header.version.to_le_bytes());
    out[12..20].copy_from_slice(&header.generation_salt.to_le_bytes());
    out[20..36].copy_from_slice(header.project_id.as_bytes());
    out
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
    let generation_salt = u64::from_le_bytes(data[12..20].try_into().expect("salt"));
    let project_id = Uuid::from_bytes(data[20..36].try_into().expect("project_id"));
    Ok(JournalHeader {
        version,
        generation_salt,
        project_id,
    })
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
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(frame.record_id.as_bytes());
    body.extend_from_slice(&uuid_to_bytes(frame.prev_id));
    body.extend_from_slice(&uuid_to_bytes(frame.snapshot_ref));
    body.extend_from_slice(&frame.record_salt.to_le_bytes());
    body.push(frame.kind as u8);
    body.extend_from_slice(&[0u8; 3]);
    body.extend_from_slice(&(frame.payload.len() as u32).to_le_bytes());
    body.extend_from_slice(&frame.payload);
    let checksum = frame_checksum(&body[4..]);
    body[..4].copy_from_slice(&checksum.to_le_bytes());
    body
}

fn decode_frame(
    data: &[u8],
    offset: u64,
    generation_salt: u64,
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
    let stored = u32::from_le_bytes(data[0..4].try_into().expect("checksum"));
    let computed = frame_checksum(&data[4..total]);
    if stored != computed {
        return Err(JournalFormatError::ChecksumMismatch { offset });
    }

    let record_id = Uuid::from_bytes(data[4..20].try_into().expect("record_id"));
    let prev_id = uuid_from_bytes(data[20..36].try_into().expect("prev"));
    let snapshot_ref = uuid_from_bytes(data[36..52].try_into().expect("snap"));
    let record_salt = u64::from_le_bytes(data[52..60].try_into().expect("record_salt"));
    if record_salt != generation_salt {
        return Err(JournalFormatError::SaltMismatch {
            record_salt,
            generation_salt,
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

#[derive(Debug, Clone, Default)]
pub struct ScanJournalOptions {
    pub verify_prev_chain: bool,
    /// 期待するproject_id。不一致ならエラー(UUID相互参照)。
    pub expected_project_id: Option<Uuid>,
}

/// 不正テールで論理停止する。**ファイルを切らない**。
pub fn scan_journal_bytes(
    data: &[u8],
    options: &ScanJournalOptions,
) -> Result<JournalScanOutcome, JournalFormatError> {
    let header = read_header(data)?;
    if let Some(expected) = options.expected_project_id {
        if header.project_id != expected {
            return Err(JournalFormatError::ProjectIdMismatch {
                header: header.project_id,
                expected,
            });
        }
    }

    let mut frames = Vec::new();
    let mut offset = HEADER_LEN;
    let mut expected_prev: Option<Uuid> = None;
    let mut generation_salt = header.generation_salt;
    let mut stopped = None;

    while offset < data.len() {
        match decode_frame(
            &data[offset..],
            offset as u64,
            generation_salt,
            expected_prev,
            options.verify_prev_chain,
        ) {
            Ok((frame, consumed)) => {
                // Checkpointフレームは以降の世代saltをpayload先頭8bytesへ更新する。
                if frame.kind == JournalRecordKind::Checkpoint && frame.payload.len() >= 8 {
                    generation_salt =
                        u64::from_le_bytes(frame.payload[0..8].try_into().expect("new salt"));
                }
                expected_prev = Some(frame.record_id);
                frames.push(frame);
                offset += consumed;
            }
            Err(JournalFormatError::ChecksumMismatch { .. }) => {
                stopped = Some(JournalScanStop::ChecksumMismatch);
                break;
            }
            Err(JournalFormatError::SaltMismatch { .. }) => {
                stopped = Some(JournalScanStop::SaltMismatch);
                break;
            }
            Err(JournalFormatError::PartialFrame(_)) => {
                stopped = Some(JournalScanStop::PartialFrame);
                break;
            }
            Err(JournalFormatError::UnknownKind(k)) => {
                stopped = Some(JournalScanStop::UnknownKind(k));
                break;
            }
            Err(JournalFormatError::BrokenPrevChain { .. }) => {
                stopped = Some(JournalScanStop::BrokenPrevChain);
                break;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(JournalScanOutcome {
        header,
        frames,
        valid_bytes: offset as u64,
        file_len: data.len() as u64,
        stopped,
    })
}

pub fn scan_journal_fs(
    fs: &mut dyn JournalFs,
    journal_path: &Path,
    options: &ScanJournalOptions,
) -> Result<JournalScanOutcome, JournalFormatError> {
    if !fs.exists(journal_path) {
        return Err(JournalFormatError::Fs(FsError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "journal missing",
        ))));
    }
    let data = fs.read(journal_path)?;
    scan_journal_bytes(&data, options)
}

pub fn scan_journal(
    journal_path: &Path,
    options: &ScanJournalOptions,
) -> Result<JournalScanOutcome, JournalFormatError> {
    let mut fs = super::fs::StdFs;
    scan_journal_fs(&mut fs, journal_path, options)
}

/// ヘッダが無ければ作る。既存があれば読む。
pub fn read_or_create_header(
    fs: &mut dyn JournalFs,
    journal_path: &Path,
    project_id: Uuid,
    generation_salt: u64,
) -> Result<JournalHeader, JournalFormatError> {
    if fs.exists(journal_path) {
        let data = fs.read(journal_path)?;
        if data.len() >= HEADER_LEN {
            let header = read_header(&data)?;
            if header.project_id != project_id {
                return Err(JournalFormatError::ProjectIdMismatch {
                    header: header.project_id,
                    expected: project_id,
                });
            }
            return Ok(header);
        }
    }
    let header = JournalHeader {
        version: JOURNAL_FORMAT_VERSION,
        generation_salt,
        project_id,
    };
    if let Some(parent) = journal_path.parent() {
        fs.create_dir_all(parent)?;
    }
    fs.write_create(journal_path, &encode_header(&header))?;
    fs.sync_file(journal_path)?;
    Ok(header)
}
