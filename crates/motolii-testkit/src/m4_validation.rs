use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::perf::{m4_validation_manifest, M4_VALIDATION_BUNDLE_SCHEMA_VERSION, SCHEMA_VERSION};

pub const M4_VALIDATION_RUN_SCHEMA_VERSION: u32 = 2;
pub const M4_VALIDATION_VERIFICATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEvidence {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationRunRecord {
    pub schema_version: u32,
    pub manifest_schema_version: u32,
    pub repository_revision: String,
    pub command_id: String,
    pub working_directory: String,
    pub manifest_sha256: String,
    pub hardware_sha256: String,
    pub started_at_unix_ms: u64,
    pub duration_ms: u64,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub spawn_error: Option<String>,
    pub required_user_env_present: Vec<String>,
    pub optional_user_env_present: Vec<String>,
    pub stdout_log: FileEvidence,
    pub stderr_log: FileEvidence,
    pub artifact: Option<FileEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct M4ValidationVerification {
    pub schema_version: u32,
    pub manifest_schema_version: u32,
    pub repository_revision: String,
    pub hardware_os: Option<String>,
    pub hardware_arch: Option<String>,
    pub local_evidence_valid: bool,
    pub verified_commands: Vec<String>,
    pub failures: Vec<String>,
    pub external_gates_pending: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
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
    #[error("failed to encode expected manifest: {0}")]
    Encode(#[from] serde_json::Error),
}

pub fn file_evidence(path: &Path) -> Result<FileEvidence, VerificationError> {
    let bytes = std::fs::read(path).map_err(|source| VerificationError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let mut digest = Sha256::new();
    digest.update(&bytes);
    Ok(FileEvidence {
        path: path.display().to_string(),
        bytes: bytes.len() as u64,
        sha256: format!("{:x}", digest.finalize()),
    })
}

fn read_value(path: &Path) -> Result<serde_json::Value, VerificationError> {
    let bytes = std::fs::read(path).map_err(|source| VerificationError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| VerificationError::Decode {
        path: path.to_path_buf(),
        source,
    })
}

fn read_record(path: &Path) -> Result<ValidationRunRecord, VerificationError> {
    let bytes = std::fs::read(path).map_err(|source| VerificationError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| VerificationError::Decode {
        path: path.to_path_buf(),
        source,
    })
}

fn verify_file(
    label: &str,
    expected_path: &Path,
    recorded: &FileEvidence,
    failures: &mut Vec<String>,
) {
    if recorded.path != expected_path.display().to_string() {
        failures.push(format!("{label}: recorded path does not match bundle path"));
        return;
    }
    match file_evidence(expected_path) {
        Ok(actual) if actual.bytes == recorded.bytes && actual.sha256 == recorded.sha256 => {}
        Ok(_) => failures.push(format!("{label}: content digest or byte count changed")),
        Err(error) => failures.push(format!("{label}: {error}")),
    }
}

fn nonempty_string(value: Option<&serde_json::Value>) -> Option<String> {
    value?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn verify_hardware(
    hardware: &serde_json::Value,
    failures: &mut Vec<String>,
) -> (Option<String>, Option<String>) {
    if hardware
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(SCHEMA_VERSION))
    {
        failures.push("hardware.json: schema version mismatch".into());
    }
    let profile = hardware.get("hardware");
    let os = nonempty_string(profile.and_then(|value| value.get("os")));
    let arch = nonempty_string(profile.and_then(|value| value.get("arch")));
    if os.is_none() || arch.is_none() {
        failures.push("hardware.json: OS or architecture is missing".into());
    }
    if profile
        .and_then(|value| value.get("total_memory_bytes"))
        .and_then(serde_json::Value::as_u64)
        .is_none_or(|bytes| bytes == 0)
    {
        failures.push("hardware.json: total physical memory is unavailable".into());
    }
    let samples = hardware
        .get("samples")
        .and_then(serde_json::Value::as_array);
    let Some(samples) = samples else {
        failures.push("hardware.json: samples are missing".into());
        return (os, arch);
    };
    for required in [
        "harness_self_check",
        "plugin_registry_init",
        "ffmpeg_capabilities",
        "headless_gpu_ctx",
    ] {
        let sample = samples
            .iter()
            .find(|sample| sample.get("id").and_then(serde_json::Value::as_str) == Some(required));
        let Some(sample) = sample else {
            failures.push(format!("hardware.json: missing sample {required}"));
            continue;
        };
        if sample.get("status").and_then(serde_json::Value::as_str) != Some("ok") {
            failures.push(format!("hardware.json: sample {required} is not ok"));
        }
        if sample
            .get("idle_rss_bytes")
            .and_then(serde_json::Value::as_u64)
            .is_none_or(|bytes| bytes == 0)
        {
            failures.push(format!(
                "hardware.json: sample {required} has no resident-memory observation"
            ));
        }
    }
    (os, arch)
}

fn fixture_identity(value: &serde_json::Value) -> Option<(u64, String)> {
    if value.get("schema_version")?.as_u64()? != 3 {
        return None;
    }
    Some((
        value.get("fixture_bytes")?.as_u64()?,
        value.get("fixture_sha256")?.as_str()?.to_owned(),
    ))
}

pub fn verify_m4_validation_bundle(
    output_dir: &Path,
    repository_revision: &str,
) -> Result<M4ValidationVerification, VerificationError> {
    let manifest_path = output_dir.join("manifest.json");
    let hardware_path = output_dir.join("hardware.json");
    let actual_manifest = read_value(&manifest_path)?;
    let expected_manifest =
        m4_validation_manifest(Some(repository_revision.to_owned()), output_dir);
    let expected_manifest_value = serde_json::to_value(&expected_manifest)?;
    let mut failures = Vec::new();
    if actual_manifest != expected_manifest_value {
        failures.push("manifest.json does not match this revision and bundle path".into());
    }
    let manifest_evidence = file_evidence(&manifest_path)?;
    let hardware_evidence = file_evidence(&hardware_path)?;
    let hardware = read_value(&hardware_path)?;
    let (hardware_os, hardware_arch) = verify_hardware(&hardware, &mut failures);
    let mut verified_commands = Vec::new();
    let mut decode_identities = Vec::new();

    for command in &expected_manifest.commands {
        let record_path = output_dir.join(format!("run-{}.json", command.id));
        let record = match read_record(&record_path) {
            Ok(record) => record,
            Err(error) => {
                failures.push(format!("{}: {error}", command.id));
                continue;
            }
        };
        let failure_count_before = failures.len();
        if record.schema_version != M4_VALIDATION_RUN_SCHEMA_VERSION {
            failures.push(format!("{}: run schema mismatch", command.id));
        }
        if record.manifest_schema_version != M4_VALIDATION_BUNDLE_SCHEMA_VERSION
            || record.repository_revision != repository_revision
            || record.command_id != command.id
            || record.working_directory != command.working_directory
        {
            failures.push(format!("{}: run identity mismatch", command.id));
        }
        if record.manifest_sha256 != manifest_evidence.sha256
            || record.hardware_sha256 != hardware_evidence.sha256
        {
            failures.push(format!(
                "{}: manifest or hardware digest mismatch",
                command.id
            ));
        }
        if !record.success || record.exit_code != Some(0) || record.spawn_error.is_some() {
            failures.push(format!(
                "{}: command did not complete successfully",
                command.id
            ));
        }
        let expected_required: Vec<_> = command
            .required_user_env
            .iter()
            .map(|value| (*value).to_owned())
            .collect();
        if record.required_user_env_present != expected_required {
            failures.push(format!(
                "{}: required environment evidence mismatch",
                command.id
            ));
        }
        verify_file(
            &format!("{} stdout", command.id),
            &output_dir.join(format!("run-{}.stdout.log", command.id)),
            &record.stdout_log,
            &mut failures,
        );
        verify_file(
            &format!("{} stderr", command.id),
            &output_dir.join(format!("run-{}.stderr.log", command.id)),
            &record.stderr_log,
            &mut failures,
        );
        match (command.artifact, &record.artifact) {
            (Some(name), Some(artifact)) => {
                let path = output_dir.join(name);
                verify_file(command.id, &path, artifact, &mut failures);
                if matches!(command.id, "decode-software" | "decode-hardware-download") {
                    match read_value(&path).ok().as_ref().and_then(fixture_identity) {
                        Some(identity) => decode_identities.push((command.id, identity)),
                        None => failures.push(format!(
                            "{}: decode fixture identity is missing",
                            command.id
                        )),
                    }
                }
                if command.id == "decode-hardware-download"
                    && read_value(&path)
                        .ok()
                        .and_then(|value| value.get("command_route_comparison").cloned())
                        .is_none_or(|value| value.is_null())
                {
                    failures.push(
                        "decode-hardware-download: hardware route comparison is missing".into(),
                    );
                }
            }
            (None, None) => {}
            _ => failures.push(format!("{}: artifact evidence shape mismatch", command.id)),
        }
        if failures.len() == failure_count_before {
            verified_commands.push(command.id.to_owned());
        }
    }

    if decode_identities.len() == 2 && decode_identities[0].1 != decode_identities[1].1 {
        failures.push("decode software and hardware runs used different fixture content".into());
    }
    let external_gates_pending = expected_manifest
        .external_gates
        .iter()
        .filter(|gate| gate.status == "pending")
        .map(|gate| gate.id.to_owned())
        .collect();
    Ok(M4ValidationVerification {
        schema_version: M4_VALIDATION_VERIFICATION_SCHEMA_VERSION,
        manifest_schema_version: M4_VALIDATION_BUNDLE_SCHEMA_VERSION,
        repository_revision: repository_revision.to_owned(),
        hardware_os,
        hardware_arch,
        local_evidence_valid: failures.is_empty()
            && verified_commands.len() == expected_manifest.commands.len(),
        verified_commands,
        failures,
        external_gates_pending,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn file_evidence_binds_bytes_and_content() {
        let dir = crate::tmp_dir("m4-file-evidence");
        let first = dir.join("first.bin");
        let second = dir.join("second.bin");
        std::fs::write(&first, b"same-content").unwrap();
        std::fs::write(&second, b"same-content").unwrap();
        let first_evidence = file_evidence(&first).unwrap();
        let second_evidence = file_evidence(&second).unwrap();
        assert_eq!(first_evidence.bytes, 12);
        assert_eq!(first_evidence.sha256, second_evidence.sha256);
        std::fs::write(&second, b"changed").unwrap();
        assert_ne!(
            first_evidence.sha256,
            file_evidence(&second).unwrap().sha256
        );
    }

    #[test]
    fn missing_runs_never_pass_or_close_external_gates() {
        let dir = crate::tmp_dir("m4-verifier-missing-runs");
        let revision = "fixture-revision";
        let manifest = m4_validation_manifest(Some(revision.into()), &dir);
        std::fs::write(
            dir.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let samples: Vec<_> = [
            "harness_self_check",
            "plugin_registry_init",
            "ffmpeg_capabilities",
            "headless_gpu_ctx",
        ]
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "status": "ok",
                "startup_ms": 1.0,
                "idle_rss_bytes": 1024,
                "notes": {}
            })
        })
        .collect();
        let hardware = json!({
            "schema_version": SCHEMA_VERSION,
            "harness": "motolii-testkit/perf",
            "recorded_at_unix_ms": 1,
            "hardware": {
                "os": "windows",
                "arch": "x86_64",
                "logical_cpu_count": 4,
                "total_memory_bytes": 8_589_934_592_u64
            },
            "samples": samples,
            "external_bench_slots": []
        });
        std::fs::write(
            dir.join("hardware.json"),
            serde_json::to_vec_pretty(&hardware).unwrap(),
        )
        .unwrap();

        let report = verify_m4_validation_bundle(&dir, revision).unwrap();
        assert!(!report.local_evidence_valid);
        assert!(report.verified_commands.is_empty());
        assert_eq!(report.failures.len(), manifest.commands.len());
        assert_eq!(
            report.external_gates_pending.len(),
            manifest.external_gates.len()
        );
    }
}
