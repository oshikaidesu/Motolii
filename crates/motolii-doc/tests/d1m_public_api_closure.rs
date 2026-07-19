//! D1m: root-public path mutation bypass must be closed.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn lib_rs_does_not_export_raw_path_mutation_or_wal_session() {
    let lib = fs::read_to_string(workspace_root().join("crates/motolii-doc/src/lib.rs")).unwrap();
    for banned in [
        "save_document,",
        "save_document_with_options",
        "save_project_with_journal",
        "migrate_document_file",
        "open_project,",
        "open_project_with_limits",
        "open_project_fs",
        "recover_project",
        "WalSession",
        "checkpoint_with_fault_plan",
        "inject_corrupt_journal_tail",
        "inject_bad_checksum_at_last_frame",
        "inject_salt_mismatch_frame",
        "inject_unapplicable_committed_edit",
    ] {
        assert!(!lib.contains(banned), "lib.rs must not pub-use `{banned}`");
    }
    assert!(lib.contains("ProjectSession"));
    assert!(lib.contains("open_project_resolved"));
}

#[test]
fn journal_mod_keeps_wal_and_recover_crate_private() {
    let journal =
        fs::read_to_string(workspace_root().join("crates/motolii-doc/src/journal/mod.rs")).unwrap();
    assert!(!journal.contains("pub use recover::recover_project"));
    assert!(!journal.contains("pub(crate) use recover::recover_project"));
    assert!(!journal.contains("pub(crate) use wal::"));
    assert!(!journal.contains("pub use wal::WalSession"));
    assert!(journal.contains("pub use session::{ProjectSession"));
    assert!(journal.contains("pub use project::{OpenProjectOutcome"));
    for banned in [
        "checkpoint_with_fault_plan",
        "inject_corrupt_journal_tail",
        "inject_bad_checksum_at_last_frame",
        "inject_salt_mismatch_frame",
        "inject_unapplicable_committed_edit",
    ] {
        assert!(
            !journal.contains(&format!("pub use project::{{{banned}")),
            "journal/mod.rs must not pub-use `{banned}`"
        );
        assert!(
            !journal.contains(&format!("pub fn {banned}")),
            "journal/mod.rs must not expose `{banned}` as pub fn"
        );
    }
    let sep = '_';
    let keep_alive_hack = format!("{sep}keep_fault{sep}injection{sep}symbols_alive");
    assert!(
        !journal.contains(&keep_alive_hack),
        "keep-alive hack must be removed"
    );
}
