use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;

use super::error::{ProfileError, VerifyError};
use super::fixture_profile::MechanismFixtureProfile;
use super::generate::{
    self, atomic_write, mechanism_temp_files_in, AtomicWriteFault, AtomicWriteFaultGuard,
};
use super::verify::check_bytes;
use super::{check, manifest_dir, write_checked_in, TokenError};

static UNIQUE_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

struct UniqueTempDir {
    path: PathBuf,
}

impl UniqueTempDir {
    fn new() -> Self {
        let base = std::env::temp_dir();
        let pid = process::id();
        loop {
            let seq = UNIQUE_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
            let path = base.join(format!("motolii-u0e1-{pid}-{seq}"));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => panic!("failed to create unique temp dir: {err}"),
            }
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for UniqueTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn u0e1_mechanism_write_checked_in_isolated_temp_root() {
    let dir = UniqueTempDir::new();
    let temp_root = dir.path();

    let repo_fixture = generate::fixture_path(&manifest_dir());
    let fixture_bytes = fs::read(&repo_fixture).expect("read repository fixture");
    let temp_fixture = generate::fixture_path(temp_root);
    if let Some(parent) = temp_fixture.parent() {
        fs::create_dir_all(parent).expect("create temp fixture parent");
    }
    fs::write(&temp_fixture, &fixture_bytes).expect("seed temp fixture");

    write_checked_in(temp_root).expect("write_checked_in");

    let written = fs::read(generate::output_path(temp_root)).expect("read generated adapter");
    let expected = generate::generate_checked_in_bytes(temp_root).expect("generate");
    assert_eq!(written, expected);

    check(temp_root).expect("check temp root");

    let output_path = generate::output_path(temp_root);
    let output_file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("output file name");
    let generated_parent = output_path.parent().expect("generated parent");
    let temps = mechanism_temp_files_in(generated_parent, output_file_name);
    assert!(temps.is_empty());
}

#[test]
fn u0e1_mechanism_fixture_profile_parses_synthetic_fixture() {
    let manifest_dir = manifest_dir();
    let resolved = generate::load_resolved(&manifest_dir).expect("fixture should parse");
    assert_eq!(resolved.tokens.len(), 4);
    assert!(resolved.tokens.contains_key("mechanism.sample-color"));
    assert!(resolved.tokens.contains_key("spacing.unit-b"));
}

#[test]
fn u0e1_mechanism_generation_is_deterministic() {
    let manifest_dir = manifest_dir();
    let first = generate::generate_checked_in_bytes(&manifest_dir).expect("generate");
    let second = generate::generate_checked_in_bytes(&manifest_dir).expect("generate");
    assert_eq!(first, second);
}

#[test]
fn u0e1_mechanism_checked_in_adapter_verifies() {
    let manifest_dir = manifest_dir();
    check(&manifest_dir).expect("checked-in adapter should verify");
}

#[test]
fn u0e1_mechanism_fixture_profile_parse_error_preserves_serde_json_source() {
    let raw = r#"{
        "$schema": "https://www.designtokens.org/schemas/2025.10/formatSchema.json",
        "mechanism": {
            "broken": { "$type": "color"
    }"#;

    let err = MechanismFixtureProfile::parse_str(raw).unwrap_err();
    let TokenError::Profile(ProfileError::Parse { source }) = err else {
        panic!("expected Profile(Parse), got {err:?}");
    };
    assert!(source.line() > 0);
    assert!(source.column() > 0);

    let err = MechanismFixtureProfile::parse_str(raw).unwrap_err();
    let json_err = Error::source(&err)
        .and_then(|profile| profile.source())
        .and_then(|source| source.downcast_ref::<serde_json::Error>())
        .expect("serde_json::Error reachable via Error::source");
    assert_eq!(json_err.line(), source.line());
    assert_eq!(json_err.column(), source.column());
}

#[test]
fn u0e1_mechanism_rejects_missing_value() {
    let raw = json!({
        "$schema": "https://www.designtokens.org/schemas/2025.10/formatSchema.json",
        "mechanism": {
            "broken": { "$type": "color" }
        }
    });
    let err = MechanismFixtureProfile::parse_str(&serde_json::to_string(&raw).expect("json"))
        .unwrap_err();
    assert!(matches!(
        err,
        TokenError::Profile(ProfileError::Parse { .. })
            | TokenError::Profile(ProfileError::Semantic(_))
    ));
}

#[test]
fn u0e1_mechanism_rejects_wrong_typed_value() {
    let raw = json!({
        "$schema": "https://www.designtokens.org/schemas/2025.10/formatSchema.json",
        "mechanism": {
            "broken": {
                "$type": "color",
                "$value": { "value": 8.0, "unit": "px" }
            }
        }
    });
    let err = MechanismFixtureProfile::parse_str(&serde_json::to_string(&raw).expect("json"))
        .unwrap_err();
    assert!(matches!(
        err,
        TokenError::Profile(ProfileError::Semantic(_))
            | TokenError::Profile(ProfileError::Parse { .. })
    ));
}

#[test]
fn u0e1_mechanism_rejects_duration_in_fixture_profile() {
    let raw = json!({
        "$schema": "https://www.designtokens.org/schemas/2025.10/formatSchema.json",
        "timing": {
            "fade": {
                "$type": "duration",
                "$value": { "value": 200, "unit": "ms" }
            }
        }
    });
    let err = MechanismFixtureProfile::parse_str(&serde_json::to_string(&raw).expect("json"))
        .unwrap_err();
    assert!(matches!(
        err,
        TokenError::Profile(ProfileError::Semantic(_))
            | TokenError::Profile(ProfileError::Parse { .. })
    ));
}

#[test]
fn u0e1_mechanism_rejects_unknown_type_in_fixture_profile() {
    let raw = json!({
        "$schema": "https://www.designtokens.org/schemas/2025.10/formatSchema.json",
        "misc": {
            "shadow": {
                "$type": "shadow",
                "$value": { "offsetX": 0, "offsetY": 1, "blur": 2, "color": "#000" }
            }
        }
    });
    let err = MechanismFixtureProfile::parse_str(&serde_json::to_string(&raw).expect("json"))
        .unwrap_err();
    assert!(matches!(
        err,
        TokenError::Profile(ProfileError::Semantic(_))
            | TokenError::Profile(ProfileError::Parse { .. })
    ));
}

#[test]
fn u0e1_mechanism_verify_rejects_hand_edited_output() {
    let manifest_dir = manifest_dir();
    let regenerated = generate::generate_checked_in_bytes(&manifest_dir).expect("generate");
    let mut tampered = regenerated.clone();
    tampered[0] ^= 0x01;
    let err = check_bytes(&tampered, &manifest_dir).unwrap_err();
    assert!(matches!(err, TokenError::Verify(VerifyError::Mismatch)));
}

#[test]
fn u0e1_mechanism_verify_rejects_stale_output() {
    let manifest_dir = manifest_dir();
    let stale_digest = generate::input_digest(b"stale-input");
    let resolved = generate::load_resolved(&manifest_dir).expect("fixture parse");
    let stale_source = generate::generate_adapter_source(&resolved, &stale_digest);
    let err = check_bytes(stale_source.as_bytes(), &manifest_dir).unwrap_err();
    assert!(matches!(err, TokenError::Verify(VerifyError::Mismatch)));
}

#[test]
fn u0e1_mechanism_verify_read_error_preserves_io_source() {
    let dir = UniqueTempDir::new();
    let err = check(dir.path()).unwrap_err();
    let TokenError::Verify(VerifyError::Read { ref source, .. }) = err else {
        panic!("expected Verify(Read), got {err:?}");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    let io_err = Error::source(&err)
        .and_then(|verify| verify.source())
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("io::Error reachable via Error::source");
    assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn u0e1_mechanism_write_leaves_no_temp_file_on_success() {
    let manifest_dir = manifest_dir();
    let bytes = generate::generate_checked_in_bytes(&manifest_dir).expect("generate");
    let dir = UniqueTempDir::new();
    let output = dir.path().join("output.rs");
    let file_name = "output.rs";

    atomic_write(&output, &bytes).expect("atomic write");

    let written = fs::read(&output).expect("read output");
    assert_eq!(written, bytes);

    let temps = mechanism_temp_files_in(dir.path(), file_name);
    assert!(temps.is_empty());
}

#[test]
fn u0e1_mechanism_atomic_write_failure_preserves_existing_output_and_cleans_temp() {
    let dir = UniqueTempDir::new();
    let output = dir.path().join("output.rs");
    let file_name = "output.rs";

    let original = b"original output bytes";
    fs::write(&output, original).expect("seed output");
    let replacement = b"replacement bytes that must not land";

    for fault in [
        AtomicWriteFault::AfterOpen,
        AtomicWriteFault::AfterWrite,
        AtomicWriteFault::AfterSync,
        AtomicWriteFault::OnRename,
    ] {
        let _guard = AtomicWriteFaultGuard::inject(fault);
        let err = atomic_write(&output, replacement).expect_err("fault should fail write");
        assert!(matches!(err, TokenError::Verify(VerifyError::Semantic(_))));

        let after = fs::read(&output).expect("read output after failed write");
        assert_eq!(after, original);

        let temps = mechanism_temp_files_in(dir.path(), file_name);
        assert!(temps.is_empty(), "temp residue after {fault:?}: {temps:?}");
    }
}
