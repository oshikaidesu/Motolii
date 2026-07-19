use std::fs;
use std::path::Path;

use super::error::VerifyError;
use super::generate;
use super::TokenError;

pub(crate) fn check(manifest_dir: &Path) -> Result<(), TokenError> {
    let output = generate::output_path(manifest_dir);
    let checked_in = fs::read(&output).map_err(|source| {
        TokenError::Verify(VerifyError::Read {
            path: output,
            source,
        })
    })?;
    check_bytes(&checked_in, manifest_dir)
}

pub(crate) fn check_bytes(checked_in: &[u8], manifest_dir: &Path) -> Result<(), TokenError> {
    let regenerated = generate::generate_checked_in_bytes(manifest_dir)?;
    if checked_in != regenerated {
        return Err(TokenError::Verify(VerifyError::Mismatch));
    }
    Ok(())
}
