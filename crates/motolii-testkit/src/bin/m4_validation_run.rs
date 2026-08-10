use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use motolii_testkit::m4_validation::{
    bundle_file_evidence, file_evidence, ValidationRunRecord, M4_VALIDATION_RUN_SCHEMA_VERSION,
};
use motolii_testkit::perf::{
    m4_validation_manifest, ValidationCommand, M4_VALIDATION_BUNDLE_SCHEMA_VERSION,
};

#[derive(Debug, thiserror::Error)]
enum RunError {
    #[error("usage: m4_validation_run <bundle-directory> <command-id>")]
    Usage,
    #[error("bundle directory does not exist or cannot be resolved: {path}: {source}")]
    BundleDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("repository HEAD is unavailable")]
    RepositoryRevision,
    #[error("repository has tracked, staged, or untracked changes; validation evidence requires a clean commit")]
    DirtyRepository,
    #[error("unknown validation command: {0}")]
    UnknownCommand(String),
    #[error("bundle input is missing: {0}")]
    MissingBundleInput(PathBuf),
    #[error("bundle manifest does not exactly match this revision, schema, and output directory")]
    ManifestMismatch,
    #[error("required environment variables are missing or empty: {0:?}")]
    MissingEnvironment(Vec<&'static str>),
    #[error("refusing to overwrite existing evidence: {0}")]
    ExistingEvidence(PathBuf),
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode JSON {path}: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to encode run record: {0}")]
    Encode(#[from] serde_json::Error),
    #[error("manifest bundle member is not one relative file component: {0}")]
    UnsafeBundleMember(String),
    #[error("failed to read validation evidence: {0}")]
    ReadEvidence(String),
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn repository_revision(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn repository_is_clean(root: &Path) -> bool {
    Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .current_dir(root)
        .output()
        .is_ok_and(|output| output.status.success() && output.stdout.is_empty())
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn present_environment(names: &[&'static str]) -> Vec<&'static str> {
    names
        .iter()
        .copied()
        .filter(|name| std::env::var_os(name).is_some_and(|value| !value.is_empty()))
        .collect()
}

fn evidence_path(output_dir: &Path, command_id: &str, suffix: &str) -> PathBuf {
    output_dir.join(format!("run-{command_id}.{suffix}"))
}

fn bundle_member_path(output_dir: &Path, member: &str) -> Result<PathBuf, RunError> {
    let mut components = Path::new(member).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(RunError::UnsafeBundleMember(member.to_owned()));
    }
    Ok(output_dir.join(member))
}

fn reject_existing(path: &Path) -> Result<(), RunError> {
    if path.exists() {
        return Err(RunError::ExistingEvidence(path.to_path_buf()));
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<serde_json::Value, RunError> {
    let bytes = std::fs::read(path).map_err(|source| RunError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| RunError::Decode {
        path: path.to_path_buf(),
        source,
    })
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    std::fs::write(path, bytes).map_err(|source| RunError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn write_record(path: &Path, record: &ValidationRunRecord) -> Result<(), RunError> {
    let mut bytes = serde_json::to_vec_pretty(record)?;
    bytes.push(b'\n');
    write_bytes(path, &bytes)
}

fn validate_bundle(output_dir: &Path, revision: &str) -> Result<Vec<ValidationCommand>, RunError> {
    let manifest_path = output_dir.join("manifest.json");
    let hardware_path = output_dir.join("hardware.json");
    let context_path = output_dir.join("context.json");
    for path in [&manifest_path, &hardware_path, &context_path] {
        if !path.is_file() {
            return Err(RunError::MissingBundleInput(path.to_path_buf()));
        }
    }
    let actual = read_json(&manifest_path)?;
    let expected_manifest = m4_validation_manifest(Some(revision.to_owned()), output_dir);
    let expected = serde_json::to_value(&expected_manifest)?;
    if actual != expected {
        return Err(RunError::ManifestMismatch);
    }
    Ok(expected_manifest.commands)
}

fn run() -> Result<(PathBuf, bool), RunError> {
    let mut args = std::env::args_os().skip(1);
    let output_arg = args.next().ok_or(RunError::Usage)?;
    let command_id = args
        .next()
        .and_then(|value| value.into_string().ok())
        .ok_or(RunError::Usage)?;
    if args.next().is_some() {
        return Err(RunError::Usage);
    }

    let output_input = PathBuf::from(output_arg);
    let output_dir =
        std::fs::canonicalize(&output_input).map_err(|source| RunError::BundleDirectory {
            path: output_input,
            source,
        })?;
    let root = repository_root();
    if !repository_is_clean(&root) {
        return Err(RunError::DirtyRepository);
    }
    let revision = repository_revision(&root).ok_or(RunError::RepositoryRevision)?;
    let commands = validate_bundle(&output_dir, &revision)?;
    let manifest_evidence = file_evidence(&output_dir.join("manifest.json"))
        .map_err(|error| RunError::ReadEvidence(error.to_string()))?;
    let hardware_evidence = file_evidence(&output_dir.join("hardware.json"))
        .map_err(|error| RunError::ReadEvidence(error.to_string()))?;
    let context_evidence = file_evidence(&output_dir.join("context.json"))
        .map_err(|error| RunError::ReadEvidence(error.to_string()))?;
    let command = commands
        .into_iter()
        .find(|candidate| candidate.id == command_id)
        .ok_or_else(|| RunError::UnknownCommand(command_id.clone()))?;

    if command.working_directory != "repository_root" {
        return Err(RunError::ManifestMismatch);
    }
    let required_present = present_environment(command.required_user_env);
    if required_present.len() != command.required_user_env.len() {
        let missing = command
            .required_user_env
            .iter()
            .copied()
            .filter(|name| !required_present.contains(name))
            .collect();
        return Err(RunError::MissingEnvironment(missing));
    }
    let optional_present = present_environment(command.optional_user_env);

    let record_path = evidence_path(&output_dir, command.id, "json");
    let stdout_path = evidence_path(&output_dir, command.id, "stdout.log");
    let stderr_path = evidence_path(&output_dir, command.id, "stderr.log");
    for path in [&record_path, &stdout_path, &stderr_path] {
        reject_existing(path)?;
    }
    let artifact_path = command
        .artifact
        .map(|name| bundle_member_path(&output_dir, name))
        .transpose()?;
    if let Some(path) = &artifact_path {
        reject_existing(path)?;
    }

    let started_at_unix_ms = unix_ms_now();
    let start = Instant::now();
    let mut process = Command::new(command.program);
    process.args(command.args).current_dir(&root);
    for (name, value) in &command.env {
        process.env(
            name,
            OsString::from(bundle_member_path(&output_dir, value)?),
        );
    }
    let output = process.output();
    let duration_ms = start.elapsed().as_millis() as u64;
    let (exit_code, spawn_error, stdout, stderr, status_success) = match output {
        Ok(output) => (
            output.status.code(),
            None,
            output.stdout,
            output.stderr,
            output.status.success(),
        ),
        Err(error) => (
            None,
            Some(error.to_string()),
            Vec::new(),
            error.to_string().into_bytes(),
            false,
        ),
    };
    write_bytes(&stdout_path, &stdout)?;
    write_bytes(&stderr_path, &stderr)?;

    let artifact = match (artifact_path.as_deref(), command.artifact) {
        (Some(path), Some(name)) if path.is_file() => Some(
            bundle_file_evidence(path, name)
                .map_err(|error| RunError::ReadEvidence(error.to_string()))?,
        ),
        _ => None,
    };
    let artifact_complete = command.artifact.is_none() || artifact.is_some();
    let success = status_success && artifact_complete;
    let record = ValidationRunRecord {
        schema_version: M4_VALIDATION_RUN_SCHEMA_VERSION,
        manifest_schema_version: M4_VALIDATION_BUNDLE_SCHEMA_VERSION,
        repository_revision: revision,
        command_id: command.id.to_owned(),
        working_directory: command.working_directory.to_owned(),
        manifest_sha256: manifest_evidence.sha256,
        hardware_sha256: hardware_evidence.sha256,
        context_sha256: context_evidence.sha256,
        started_at_unix_ms,
        duration_ms,
        success,
        exit_code,
        spawn_error,
        required_user_env_present: required_present.into_iter().map(str::to_owned).collect(),
        optional_user_env_present: optional_present.into_iter().map(str::to_owned).collect(),
        stdout_log: bundle_file_evidence(&stdout_path, &format!("run-{}.stdout.log", command.id))
            .map_err(|error| RunError::ReadEvidence(error.to_string()))?,
        stderr_log: bundle_file_evidence(&stderr_path, &format!("run-{}.stderr.log", command.id))
            .map_err(|error| RunError::ReadEvidence(error.to_string()))?,
        artifact,
    };
    write_record(&record_path, &record)?;
    Ok((record_path, success))
}

fn main() -> ExitCode {
    match run() {
        Ok((path, success)) => {
            println!("{}", path.display());
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_names_are_command_scoped_and_shell_independent() {
        let root = Path::new("bundle");
        assert_eq!(
            evidence_path(root, "decode-software", "stdout.log"),
            root.join("run-decode-software.stdout.log")
        );
    }

    #[test]
    fn empty_environment_value_is_not_present() {
        let name = "MOTOLII_TESTKIT_EMPTY_ENV_FIXTURE";
        // SAFETY: このtest専用名で、同じprocess内の他testは参照しない。
        unsafe { std::env::set_var(name, "") };
        assert!(present_environment(&[name]).is_empty());
        // SAFETY: 上と同じ専用名を元へ戻す。
        unsafe { std::env::remove_var(name) };
    }
}
