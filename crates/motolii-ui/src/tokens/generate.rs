use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use super::error::VerifyError;
use super::fixture_profile::{MechanismFixtureProfile, ResolvedMechanismTokens, ResolvedToken};
use super::TokenError;

pub(crate) const GENERATOR_VERSION: &str = "u0e1-mechanism/1";
pub(crate) const FIXTURE_REL_PATH: &str = "tokens/fixtures/u0e1_mechanism.tokens.json";
pub(crate) const OUTPUT_REL_PATH: &str = "tokens/generated/u0e1_mechanism_adapter.rs";

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
static ATOMIC_WRITE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtomicWriteFault {
    AfterOpen,
    AfterWrite,
    AfterSync,
    OnRename,
}

#[cfg(test)]
thread_local! {
    static ATOMIC_WRITE_FAULT: std::cell::Cell<Option<AtomicWriteFault>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_atomic_write_fault(fault: Option<AtomicWriteFault>) {
    ATOMIC_WRITE_FAULT.with(|slot| slot.set(fault));
}

#[cfg(test)]
pub(crate) struct AtomicWriteFaultGuard {
    previous: Option<AtomicWriteFault>,
}

#[cfg(test)]
impl AtomicWriteFaultGuard {
    pub(crate) fn inject(fault: AtomicWriteFault) -> Self {
        let previous = ATOMIC_WRITE_FAULT.with(|slot| {
            let previous = slot.get();
            slot.set(Some(fault));
            previous
        });
        Self { previous }
    }
}

#[cfg(test)]
impl Drop for AtomicWriteFaultGuard {
    fn drop(&mut self) {
        set_atomic_write_fault(self.previous);
    }
}

struct TempFileGuard {
    path: Option<PathBuf>,
}

impl TempFileGuard {
    fn disarm(&mut self) {
        self.path = None;
    }

    fn cleanup(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

pub(crate) fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
}

pub(crate) fn fixture_path(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(FIXTURE_REL_PATH)
}

pub(crate) fn output_path(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join(OUTPUT_REL_PATH)
}

pub(crate) fn load_resolved(manifest_dir: &Path) -> Result<ResolvedMechanismTokens, TokenError> {
    let path = fixture_path(manifest_dir);
    let raw = fs::read_to_string(&path)?;
    MechanismFixtureProfile::parse_str(&raw)
}

pub(crate) fn input_digest(raw_fixture: &[u8]) -> String {
    let hash = Sha256::digest(raw_fixture);
    format!("sha256:{}", hex_encode(&hash))
}

pub(crate) fn generate_adapter_source(
    resolved: &ResolvedMechanismTokens,
    input_digest: &str,
) -> String {
    let mut lines = vec![
        "// GENERATED — do not edit manually".to_string(),
        format!("// generator-version: {GENERATOR_VERSION}"),
        format!("// input-digest: {input_digest}"),
        format!("// output-path: {OUTPUT_REL_PATH}"),
        String::new(),
        "use egui::Color32;".to_string(),
        String::new(),
        "pub(crate) struct U0e1MechanismAdapter;".to_string(),
        String::new(),
        "impl U0e1MechanismAdapter {".to_string(),
    ];

    for (path, token) in &resolved.tokens {
        let fn_name = rust_fn_name(path);
        match token {
            ResolvedToken::Color { r, g, b, a } => {
                let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
                lines.push(format!("    pub(crate) fn {fn_name}() -> Color32 {{"));
                lines.push(format!(
                    "        Color32::from_rgba_unmultiplied({r}, {g}, {b}, {alpha})"
                ));
                lines.push("    }".to_string());
            }
            ResolvedToken::Dimension(px) => {
                lines.push(format!("    pub(crate) fn {fn_name}() -> f32 {{"));
                lines.push(format!("        {px:.1}"));
                lines.push("    }".to_string());
            }
        }
    }

    lines.push("}".to_string());
    lines.push(String::new());
    lines.join("\n")
}

pub(crate) fn generate_checked_in_bytes(manifest_dir: &Path) -> Result<Vec<u8>, TokenError> {
    let fixture = fixture_path(manifest_dir);
    let raw = fs::read(&fixture)?;
    let digest = input_digest(&raw);
    let resolved = load_resolved(manifest_dir)?;
    let source = generate_adapter_source(&resolved, &digest);
    Ok(source.into_bytes())
}

pub(crate) fn write_checked_in(manifest_dir: &Path) -> Result<(), TokenError> {
    let bytes = generate_checked_in_bytes(manifest_dir)?;
    atomic_write(&output_path(manifest_dir), &bytes)?;
    Ok(())
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), TokenError> {
    #[cfg(test)]
    let _test_lock = ATOMIC_WRITE_TEST_LOCK
        .lock()
        .expect("atomic write test lock poisoned");

    let parent = path.parent().ok_or_else(|| {
        TokenError::Verify(VerifyError::Semantic("output path has no parent".into()))
    })?;
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            TokenError::Verify(VerifyError::Semantic("output path has no file name".into()))
        })?;

    let (temp_path, mut file) = create_exclusive_temp(parent, file_name)?;
    let mut guard = TempFileGuard {
        path: Some(temp_path.clone()),
    };

    #[cfg(test)]
    if matches_fault(AtomicWriteFault::AfterOpen) {
        drop(file);
        guard.cleanup();
        return Err(TokenError::Verify(VerifyError::Semantic(
            "injected fault: after open".into(),
        )));
    }

    if let Err(err) = file.write_all(bytes) {
        drop(file);
        guard.cleanup();
        return Err(err.into());
    }

    #[cfg(test)]
    if matches_fault(AtomicWriteFault::AfterWrite) {
        drop(file);
        guard.cleanup();
        return Err(TokenError::Verify(VerifyError::Semantic(
            "injected fault: after write".into(),
        )));
    }

    if let Err(err) = file.sync_all() {
        drop(file);
        guard.cleanup();
        return Err(err.into());
    }

    #[cfg(test)]
    if matches_fault(AtomicWriteFault::AfterSync) {
        drop(file);
        guard.cleanup();
        return Err(TokenError::Verify(VerifyError::Semantic(
            "injected fault: after sync".into(),
        )));
    }

    drop(file);

    #[cfg(test)]
    if matches_fault(AtomicWriteFault::OnRename) {
        guard.cleanup();
        return Err(TokenError::Verify(VerifyError::Semantic(
            "injected fault: on rename".into(),
        )));
    }

    fs::rename(&temp_path, path)?;
    guard.disarm();
    Ok(())
}

#[cfg(test)]
pub(crate) fn mechanism_temp_files_in(dir: &Path, output_file_name: &str) -> Vec<PathBuf> {
    let prefix = format!("{output_file_name}.");
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".tmp"))
        })
        .collect()
}

fn create_exclusive_temp(
    parent: &Path,
    file_name: &str,
) -> Result<(PathBuf, fs::File), TokenError> {
    let pid = std::process::id();
    for _ in 0..1024 {
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let temp_path = parent.join(format!("{file_name}.{pid}.{seq}.tmp"));
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err.into()),
        }
    }
    Err(TokenError::Verify(VerifyError::Semantic(
        "failed to allocate exclusive temp file".into(),
    )))
}

#[cfg(test)]
fn matches_fault(expected: AtomicWriteFault) -> bool {
    ATOMIC_WRITE_FAULT.with(|slot| slot.get() == Some(expected))
}

fn rust_fn_name(token_path: &str) -> String {
    token_path.replace(['.', '-'], "_")
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
