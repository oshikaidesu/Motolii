use std::fs::{self, File};

use motolii_doc::{Asset, AssetId, SourceFingerprintV1};
use motolii_gpu::{AdmissionError, ResourceBudgets, ResourceLedger, ResourceOwner, ResourceTier};
use motolii_media::{SourceBinding, SourceBindingError};

fn ledger(disk_bytes: u64) -> ResourceLedger {
    ResourceLedger::new(ResourceBudgets {
        vram_bytes: 0,
        ram_bytes: 0,
        disk_bytes,
        shared_memory_bytes: None,
    })
}

fn asset(path: &std::path::Path, fingerprint: &SourceFingerprintV1) -> Asset {
    Asset {
        id: AssetId::from_raw(7),
        name: "source".into(),
        asset_type: "application/octet-stream".into(),
        content_hash: fingerprint.content_hash(),
        path_absolute: Some(path.to_string_lossy().into_owned()),
        path_project_relative: None,
        file_name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned()),
        size_bytes: Some(fingerprint.size_bytes()),
        head_hash: None,
        tail_hash: None,
    }
}

fn fingerprint(path: &std::path::Path) -> SourceFingerprintV1 {
    SourceFingerprintV1::from_reader(File::open(path).unwrap()).unwrap()
}

#[test]
fn exact_copy_is_read_only_pinned_and_released_after_last_capability() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.bin");
    let bytes = b"immutable source bytes";
    fs::write(&source, bytes).unwrap();
    let expected = fingerprint(&source);
    let asset = asset(&source, &expected);
    let ledger = ledger(bytes.len() as u64);

    let binding = SourceBinding::bind_asset(
        &asset,
        None,
        dir.path(),
        &ledger,
        ResourceOwner::new("test-job"),
    )
    .unwrap();
    let path = binding.path().to_path_buf();
    assert_ne!(path, source);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(binding.fingerprint(), &expected);
    assert_eq!(binding.size_bytes(), bytes.len() as u64);
    assert!(fs::metadata(&path).unwrap().permissions().readonly());
    assert_eq!(
        ledger.tier_live_bytes(ResourceTier::Disk),
        bytes.len() as u64
    );

    let clone = binding.clone();
    drop(binding);
    assert!(path.is_file());
    assert_eq!(
        ledger.tier_live_bytes(ResourceTier::Disk),
        bytes.len() as u64
    );
    drop(clone);
    assert!(!path.exists());
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Disk), 0);
}

#[test]
fn same_size_source_change_rejects_and_releases_the_reservation() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.bin");
    fs::write(&source, b"before").unwrap();
    let expected = fingerprint(&source);
    let asset = asset(&source, &expected);
    fs::write(&source, b"after!").unwrap();
    let ledger = ledger(6);

    let error = SourceBinding::bind_asset(
        &asset,
        None,
        dir.path(),
        &ledger,
        ResourceOwner::new("test-job"),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        SourceBindingError::SourceFingerprintChanged { .. }
    ));
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Disk), 0);
}

#[test]
fn disk_budget_refusal_writes_no_binding_and_leaves_accounting_zero() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.bin");
    fs::write(&source, b"too large").unwrap();
    let expected = fingerprint(&source);
    let asset = asset(&source, &expected);
    let ledger = ledger(1);

    let error = SourceBinding::bind_asset(
        &asset,
        None,
        dir.path(),
        &ledger,
        ResourceOwner::new("test-job"),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        SourceBindingError::Admission(AdmissionError::TierCapExceeded { .. })
    ));
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Disk), 0);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn legacy_fingerprint_and_offline_source_fail_before_reservation() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.bin");
    fs::write(&source, b"source").unwrap();
    let expected = fingerprint(&source);
    let mut asset = asset(&source, &expected);
    asset.content_hash = "legacy".into();
    let ledger = ledger(1024);

    let error = SourceBinding::bind_asset(
        &asset,
        None,
        dir.path(),
        &ledger,
        ResourceOwner::new("test-job"),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        SourceBindingError::UnverifiedFingerprint { .. }
    ));
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Disk), 0);

    asset.content_hash = expected.content_hash();
    fs::remove_file(&source).unwrap();
    let error = SourceBinding::bind_asset(
        &asset,
        None,
        dir.path(),
        &ledger,
        ResourceOwner::new("test-job"),
    )
    .unwrap_err();
    assert!(matches!(error, SourceBindingError::UnresolvedSource));
    assert_eq!(ledger.tier_live_bytes(ResourceTier::Disk), 0);
}
