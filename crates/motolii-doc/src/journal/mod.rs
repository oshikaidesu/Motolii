//! D1d: 追記ジャーナル(ガード3/4/6)。
//!
//! SQLite WAL の壊れ方カタログ(checksum・世代salt・テール切捨て)を踏襲する。
//! D1c のアトミック保存契約は変更せず、並走で journal/catalog/generations を管理する。

mod catalog;
mod format;
mod project;
mod replay;

pub use catalog::{
    load_catalog, GenerationCatalog, GenerationEntry, PinGenerationOptions, RotateOptions,
};
pub use format::{
    scan_journal, JournalFrame, JournalHeader, JournalRecordKind, JournalScanOutcome,
    JournalScanStop, ScanJournalOptions,
};
pub use project::{
    inject_bad_checksum_at_last_frame, inject_corrupt_journal_tail, inject_salt_mismatch_frame,
    open_project, save_project_with_journal, OpenProjectOutcome, ProjectError, RecoverySource,
    SaveProjectOptions,
};
pub use replay::{edit_payload, JournalEdit, ReplayFailure, ReplayOutcome};
