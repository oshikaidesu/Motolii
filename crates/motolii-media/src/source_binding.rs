use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use motolii_doc::{
    resolve_asset_path, Asset, SourceFingerprintDecode, SourceFingerprintError, SourceFingerprintV1,
};
use motolii_gpu::{
    AdmissionError, AdmissionPermit, ResidentClass, ResourceLedger, ResourceOwner, ResourcePurpose,
    ResourceTier, UsageError,
};
use tempfile::{Builder, TempPath};

const COPY_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum SourceBindingError {
    #[error("asset has no resolvable source path")]
    UnresolvedSource,
    #[error("asset has no verified SourceFingerprintV1: {decoded:?}")]
    UnverifiedFingerprint { decoded: SourceFingerprintDecode },
    #[error("source size changed: expected {expected}, observed {observed}")]
    SourceSizeChanged { expected: u64, observed: u64 },
    #[error("source fingerprint changed: expected {expected}, observed {observed}")]
    SourceFingerprintChanged { expected: String, observed: String },
    #[error(transparent)]
    Admission(#[from] AdmissionError),
    #[error(transparent)]
    Usage(#[from] UsageError),
    #[error(transparent)]
    Fingerprint(#[from] SourceFingerprintError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct SourceBinding {
    inner: Arc<SourceBindingInner>,
}

#[derive(Debug)]
struct SourceBindingInner {
    path: ReadOnlyTempPath,
    fingerprint: SourceFingerprintV1,
    _permit: AdmissionPermit,
}

#[derive(Debug)]
struct ReadOnlyTempPath(TempPath);

impl Drop for ReadOnlyTempPath {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if let Ok(metadata) = fs::metadata(&self.0) {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(false);
                let _ = fs::set_permissions(&self.0, permissions);
            }
        }
    }
}

impl SourceBinding {
    pub fn bind_asset(
        asset: &Asset,
        project_root: Option<&Path>,
        binding_dir: &Path,
        ledger: &ResourceLedger,
        owner: ResourceOwner,
    ) -> Result<Self, SourceBindingError> {
        let expected =
            match SourceFingerprintV1::decode_persisted(&asset.content_hash, asset.size_bytes) {
                SourceFingerprintDecode::V1(fingerprint) => fingerprint,
                decoded => return Err(SourceBindingError::UnverifiedFingerprint { decoded }),
            };
        let source_path =
            resolve_asset_path(asset, project_root).ok_or(SourceBindingError::UnresolvedSource)?;
        Self::bind_source(
            &source_path,
            binding_dir,
            expected,
            ledger,
            owner,
            ResourcePurpose::new(format!("asset-{}-source-binding", asset.id.get())),
        )
    }

    pub fn path(&self) -> &Path {
        &self.inner.path.0
    }

    pub fn fingerprint(&self) -> &SourceFingerprintV1 {
        &self.inner.fingerprint
    }

    pub fn size_bytes(&self) -> u64 {
        self.inner.fingerprint.size_bytes()
    }

    fn bind_source(
        source_path: &Path,
        binding_dir: &Path,
        expected: SourceFingerprintV1,
        ledger: &ResourceLedger,
        owner: ResourceOwner,
        purpose: ResourcePurpose,
    ) -> Result<Self, SourceBindingError> {
        let mut source = File::open(source_path)?;
        let source_size = source.metadata()?.len();
        if source_size != expected.size_bytes() {
            return Err(SourceBindingError::SourceSizeChanged {
                expected: expected.size_bytes(),
                observed: source_size,
            });
        }

        let mut permit = ledger.admit(
            owner,
            ResourceTier::Disk,
            purpose,
            ResidentClass::Pinned,
            source_size,
        )?;
        let suffix = source_suffix(source_path);
        let mut temp = Builder::new()
            .prefix("motolii-source-")
            .suffix(&suffix)
            .tempfile_in(binding_dir)?;

        copy_exact_bounded(&mut source, temp.as_file_mut(), source_size)?;
        temp.as_file_mut().flush()?;
        temp.as_file().sync_all()?;

        let observed = SourceFingerprintV1::from_reader(File::open(temp.path())?)?;
        if observed != expected {
            return Err(SourceBindingError::SourceFingerprintChanged {
                expected: expected.content_hash(),
                observed: observed.content_hash(),
            });
        }

        permit.commit_usage(observed.size_bytes())?;
        let temp_path = temp.into_temp_path();
        let mut permissions = fs::metadata(&temp_path)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&temp_path, permissions)?;

        Ok(Self {
            inner: Arc::new(SourceBindingInner {
                path: ReadOnlyTempPath(temp_path),
                fingerprint: observed,
                _permit: permit,
            }),
        })
    }
}

fn copy_exact_bounded(
    source: &mut File,
    destination: &mut File,
    expected_size: u64,
) -> Result<(), SourceBindingError> {
    let mut copied = 0u64;
    let mut buffer = [0u8; COPY_CHUNK_BYTES];
    while copied < expected_size {
        let remaining = expected_size - copied;
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .expect("copy chunk length always fits usize");
        let count = source.read(&mut buffer[..limit])?;
        if count == 0 {
            return Err(SourceBindingError::SourceSizeChanged {
                expected: expected_size,
                observed: copied,
            });
        }
        destination.write_all(&buffer[..count])?;
        copied += count as u64;
    }

    let mut extra = [0u8; 1];
    if source.read(&mut extra)? != 0 {
        return Err(SourceBindingError::SourceSizeChanged {
            expected: expected_size,
            observed: expected_size.saturating_add(1),
        });
    }
    Ok(())
}

fn source_suffix(source_path: &Path) -> OsString {
    source_path
        .extension()
        .map(|extension| {
            let mut suffix = OsString::from(".");
            suffix.push(extension);
            suffix
        })
        .unwrap_or_default()
}
