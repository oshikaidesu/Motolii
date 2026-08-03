//! M4-P04-C4: `fs4`のdisk watermark probe。
//!
//! free-spaceとallocation granularityだけを検証し、hard budget／eviction／Document
//! failureはHost policyの責任としてこのfixtureへ持ち込まない。

use std::fs::{self, File};
use std::path::PathBuf;

fn probe_file() -> PathBuf {
    std::env::temp_dir().join(format!("motolii-m4-p04-fs4-{}", std::process::id()))
}

#[test]
fn reports_space_and_allocation_granularity_for_existing_paths() {
    let dir = std::env::temp_dir();
    let free = fs4::free_space(&dir).expect("free_space for temp filesystem");
    let granularity =
        fs4::allocation_granularity(&dir).expect("allocation granularity for temp filesystem");

    assert!(free > 0, "watermark probe must report usable space");
    assert!(granularity > 0, "allocation granularity must be positive");
}

#[test]
fn accepts_a_file_path_on_the_same_filesystem() {
    let path = probe_file();
    let _ = File::create(&path).expect("create targeted probe file");

    let dir_free = fs4::free_space(std::env::temp_dir()).expect("free space for directory");
    let file_free = fs4::free_space(&path).expect("free space for file path");
    let dir_granularity = fs4::allocation_granularity(std::env::temp_dir())
        .expect("allocation granularity for directory");
    let file_granularity =
        fs4::allocation_granularity(&path).expect("allocation granularity for file path");

    assert_eq!(dir_free, file_free);
    assert_eq!(dir_granularity, file_granularity);
    fs::remove_file(path).expect("remove targeted probe file");
}

#[test]
fn missing_path_is_a_typed_error_not_a_watermark_value() {
    let path = std::env::temp_dir().join("motolii-m4-p04-fs4-missing");
    let free = fs4::free_space(&path);
    let granularity = fs4::allocation_granularity(&path);

    assert!(free.is_err());
    assert!(granularity.is_err());
}
