//! D1d: 追記ジャーナル(ガード3/4/6) + 実FS故障注入(S11) + 非破壊recovery(S15)。
//!
//! - record checksum / generation salt / UUID相互参照
//! - 不正テールは論理無視(原本truncate禁止)
//! - replay失敗フォールバック・ピン留め世代
//! - `#101`の`ResourceLimits`を再利用(別limitsを発明しない)
//! - process間lockは契約が無いため扱わない

mod catalog;
mod format;
mod fs;
mod project;
mod recover;
mod replay;
mod v1_edit;
mod wal;

pub use catalog::{
    generation_path_for_document, load_catalog, GenerationCatalog, GenerationEntry,
    PinGenerationOptions, RotateOptions,
};
pub use format::{
    journal_path_for_document, read_or_create_header, scan_journal, JournalFormatError,
    JournalFrame, JournalHeader, JournalRecordKind, JournalScanOutcome, JournalScanStop,
    ScanJournalOptions, HEADER_LEN,
};
pub use fs::{
    DurabilityStage, FaultInjectingFs, FaultPlan, FsError, FsOp, FsOpKind, JournalFs, RecordingFs,
    StdFs,
};
pub use project::{
    checkpoint_with_fault_plan, inject_bad_checksum_at_last_frame, inject_corrupt_journal_tail,
    inject_salt_mismatch_frame, inject_unapplicable_committed_edit, open_project, open_project_fs,
    open_project_with_limits, save_project_with_journal, save_project_with_journal_fs,
    OpenProjectOutcome, ProjectError, SaveProjectOptions,
};
pub use recover::{
    recover_project, recovered_document_path, restore_attempted_path, RecoveryError,
    RecoveryResult, RecoverySource,
};
pub use replay::{
    document_fingerprint, edit_payload, replay_from_base, JournalEdit, ReplayFailure,
    ReplayOutcome, V1_EDIT_FORMAT_VERSION, V2_EDIT_FORMAT_VERSION,
};
pub use wal::{checkpoint, commit_edit, CheckpointOptions, WalError, WalSession};
