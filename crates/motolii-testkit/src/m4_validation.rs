use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::perf::{m4_validation_manifest, M4_VALIDATION_BUNDLE_SCHEMA_VERSION, SCHEMA_VERSION};

pub const M4_VALIDATION_RUN_SCHEMA_VERSION: u32 = 3;
pub const M4_VALIDATION_VERIFICATION_SCHEMA_VERSION: u32 = 2;
pub const M4_VALIDATION_MATRIX_SCHEMA_VERSION: u32 = 1;

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
    pub context_sha256: String,
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
    pub machine_label: Option<String>,
    pub intended_persona: Option<String>,
    pub local_evidence_valid: bool,
    pub verified_commands: Vec<String>,
    pub failures: Vec<String>,
    pub external_gates_pending: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecodeRouteMetrics {
    pub sequential_120_frames_ms: f64,
    pub parallel_8_requests_wall_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AudioMadMetrics {
    pub clip_count: u64,
    pub effects_per_clip: u64,
    pub max_active_video_slots: u64,
    pub max_graph_steps: u64,
    pub sequential_max_ms: f64,
    pub scrub_max_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct M4ValidationMatrixEntry {
    pub bundle_path: String,
    pub machine_label: String,
    pub intended_persona: String,
    pub power_source: String,
    pub power_mode: String,
    pub display_width_px: u64,
    pub display_height_px: u64,
    pub hardware_os: String,
    pub hardware_arch: String,
    pub logical_cpu_count: u64,
    pub total_memory_bytes: u64,
    pub gpu_adapter_name: String,
    pub gpu_backend: String,
    pub gpu_device_type: String,
    pub gpu_driver: Option<String>,
    pub ffmpeg_version: String,
    pub hardware_decode_accel: String,
    pub hardware_decode_output_format: String,
    pub hardware_decode_surface_format: Option<String>,
    pub software_decode: DecodeRouteMetrics,
    pub hardware_download_decode: DecodeRouteMetrics,
    pub frame_zero_differing_bytes: u64,
    pub audio_mad: AudioMadMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct M4ValidationMatrix {
    pub schema_version: u32,
    pub repository_revision: String,
    pub fixture_bytes: u64,
    pub fixture_sha256: String,
    pub thresholds_selected: bool,
    pub repetition_policy_selected: bool,
    pub low_spec_windows_gate_closed: bool,
    pub entries: Vec<M4ValidationMatrixEntry>,
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

#[derive(Debug, thiserror::Error)]
pub enum MatrixError {
    #[error("at least two validation bundles are required")]
    TooFewBundles,
    #[error("bundle {path} did not pass local verification: {failures:?}")]
    InvalidBundle {
        path: PathBuf,
        failures: Vec<String>,
    },
    #[error("bundle {path} is missing comparison field {field}")]
    MissingField { path: PathBuf, field: &'static str },
    #[error(
        "bundle {path} used fixture {actual_bytes} bytes/{actual_sha256}, expected {expected_bytes} bytes/{expected_sha256}"
    )]
    FixtureMismatch {
        path: PathBuf,
        expected_bytes: u64,
        expected_sha256: String,
        actual_bytes: u64,
        actual_sha256: String,
    },
    #[error("failed to verify bundle: {0}")]
    Verification(#[from] VerificationError),
    #[error("verified comparison set did not produce a fixture identity")]
    MissingFixture,
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

fn verify_context(
    context: &serde_json::Value,
    failures: &mut Vec<String>,
) -> (Option<String>, Option<String>) {
    if context
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(u64::from(crate::perf::M4_VALIDATION_CONTEXT_SCHEMA_VERSION))
    {
        failures.push("context.json: schema version mismatch".into());
    }
    let machine_label = nonempty_string(context.get("machine_label"));
    let intended_persona = nonempty_string(context.get("intended_persona"));
    let power_source = nonempty_string(context.get("power_source"));
    let power_mode = nonempty_string(context.get("power_mode"));
    if machine_label.is_none() || intended_persona.is_none() || power_mode.is_none() {
        failures.push("context.json: required measurement label is missing".into());
    }
    if power_source
        .as_deref()
        .is_none_or(|value| !matches!(value, "ac" | "battery"))
    {
        failures.push("context.json: power source must be ac or battery".into());
    }
    for dimension in ["display_width_px", "display_height_px"] {
        if context
            .get(dimension)
            .and_then(serde_json::Value::as_u64)
            .is_none_or(|value| value == 0 || value > u64::from(u32::MAX))
        {
            failures.push(format!("context.json: {dimension} must be a positive u32"));
        }
    }
    (machine_label, intended_persona)
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

fn required_string(
    value: &serde_json::Value,
    field: &'static str,
    path: &Path,
) -> Result<String, MatrixError> {
    nonempty_string(value.pointer(field)).ok_or_else(|| MatrixError::MissingField {
        path: path.to_path_buf(),
        field,
    })
}

fn required_u64(
    value: &serde_json::Value,
    field: &'static str,
    path: &Path,
) -> Result<u64, MatrixError> {
    value
        .pointer(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| MatrixError::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

fn required_f64(
    value: &serde_json::Value,
    field: &'static str,
    path: &Path,
) -> Result<f64, MatrixError> {
    value
        .pointer(field)
        .and_then(serde_json::Value::as_f64)
        .filter(|number| number.is_finite() && *number >= 0.0)
        .ok_or_else(|| MatrixError::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

fn max_array_u64(
    value: &serde_json::Value,
    array_field: &'static str,
    item_field: &'static str,
    path: &Path,
) -> Result<u64, MatrixError> {
    value
        .get(array_field)
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.get(item_field)?.as_u64())
                .max()
        })
        .ok_or_else(|| MatrixError::MissingField {
            path: path.to_path_buf(),
            field: array_field,
        })
}

fn max_array_f64(
    value: &serde_json::Value,
    array_field: &'static str,
    item_field: &'static str,
    path: &Path,
) -> Result<f64, MatrixError> {
    value
        .get(array_field)
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .filter_map(|item| item.get(item_field)?.as_f64())
                .filter(|number| number.is_finite() && *number >= 0.0)
                .reduce(f64::max)
        })
        .ok_or_else(|| MatrixError::MissingField {
            path: path.to_path_buf(),
            field: array_field,
        })
}

fn sample_note<'a>(
    hardware: &'a serde_json::Value,
    sample_id: &str,
    field: &'static str,
    path: &Path,
) -> Result<&'a str, MatrixError> {
    hardware
        .get("samples")
        .and_then(serde_json::Value::as_array)
        .and_then(|samples| {
            samples.iter().find(|sample| {
                sample.get("id").and_then(serde_json::Value::as_str) == Some(sample_id)
            })
        })
        .and_then(|sample| sample.get("notes"))
        .and_then(|notes| notes.get(field))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| MatrixError::MissingField {
            path: path.to_path_buf(),
            field,
        })
}

fn optional_sample_note(
    hardware: &serde_json::Value,
    sample_id: &str,
    field: &str,
) -> Option<String> {
    hardware
        .get("samples")
        .and_then(serde_json::Value::as_array)
        .and_then(|samples| {
            samples.iter().find(|sample| {
                sample.get("id").and_then(serde_json::Value::as_str) == Some(sample_id)
            })
        })
        .and_then(|sample| sample.get("notes"))
        .and_then(|notes| notes.get(field))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn ensure_fixture_matches(
    path: &Path,
    expected: &(u64, String),
    actual: &(u64, String),
) -> Result<(), MatrixError> {
    if expected == actual {
        return Ok(());
    }
    Err(MatrixError::FixtureMismatch {
        path: path.to_path_buf(),
        expected_bytes: expected.0,
        expected_sha256: expected.1.clone(),
        actual_bytes: actual.0,
        actual_sha256: actual.1.clone(),
    })
}

fn matrix_entry(
    output_dir: &Path,
) -> Result<((u64, String), M4ValidationMatrixEntry), MatrixError> {
    let context = read_value(&output_dir.join("context.json"))?;
    let hardware = read_value(&output_dir.join("hardware.json"))?;
    let decode = read_value(&output_dir.join("decode-hardware-download.json"))?;
    let audio = read_value(&output_dir.join("audio-mad-graph.json"))?;
    let fixture = fixture_identity(&decode).ok_or_else(|| MatrixError::MissingField {
        path: output_dir.to_path_buf(),
        field: "decode fixture identity",
    })?;
    let comparison =
        decode
            .get("command_route_comparison")
            .ok_or_else(|| MatrixError::MissingField {
                path: output_dir.to_path_buf(),
                field: "command_route_comparison",
            })?;
    let software_decode = DecodeRouteMetrics {
        sequential_120_frames_ms: required_f64(
            comparison,
            "/software/sequential/elapsed_ms",
            output_dir,
        )?,
        parallel_8_requests_wall_ms: required_f64(
            comparison,
            "/software/parallel_wall_ms",
            output_dir,
        )?,
    };
    let hardware_download_decode = DecodeRouteMetrics {
        sequential_120_frames_ms: required_f64(
            comparison,
            "/hardware/sequential/elapsed_ms",
            output_dir,
        )?,
        parallel_8_requests_wall_ms: required_f64(
            comparison,
            "/hardware/parallel_wall_ms",
            output_dir,
        )?,
    };
    let audio_mad = AudioMadMetrics {
        clip_count: required_u64(&audio, "/clip_count", output_dir)?,
        effects_per_clip: required_u64(&audio, "/effects_per_clip", output_dir)?,
        max_active_video_slots: max_array_u64(
            &audio,
            "sequential",
            "active_video_slots",
            output_dir,
        )?
        .max(max_array_u64(
            &audio,
            "scrub",
            "active_video_slots",
            output_dir,
        )?),
        max_graph_steps: max_array_u64(&audio, "sequential", "graph_steps", output_dir)?
            .max(max_array_u64(&audio, "scrub", "graph_steps", output_dir)?),
        sequential_max_ms: max_array_f64(&audio, "sequential", "elapsed_ms", output_dir)?,
        scrub_max_ms: max_array_f64(&audio, "scrub", "elapsed_ms", output_dir)?,
    };
    let entry = M4ValidationMatrixEntry {
        bundle_path: output_dir.display().to_string(),
        machine_label: required_string(&context, "/machine_label", output_dir)?,
        intended_persona: required_string(&context, "/intended_persona", output_dir)?,
        power_source: required_string(&context, "/power_source", output_dir)?,
        power_mode: required_string(&context, "/power_mode", output_dir)?,
        display_width_px: required_u64(&context, "/display_width_px", output_dir)?,
        display_height_px: required_u64(&context, "/display_height_px", output_dir)?,
        hardware_os: required_string(&hardware, "/hardware/os", output_dir)?,
        hardware_arch: required_string(&hardware, "/hardware/arch", output_dir)?,
        logical_cpu_count: required_u64(&hardware, "/hardware/logical_cpu_count", output_dir)?,
        total_memory_bytes: required_u64(&hardware, "/hardware/total_memory_bytes", output_dir)?,
        gpu_adapter_name: sample_note(&hardware, "headless_gpu_ctx", "adapter_name", output_dir)?
            .to_owned(),
        gpu_backend: sample_note(&hardware, "headless_gpu_ctx", "backend", output_dir)?.to_owned(),
        gpu_device_type: sample_note(&hardware, "headless_gpu_ctx", "device_type", output_dir)?
            .to_owned(),
        gpu_driver: optional_sample_note(&hardware, "headless_gpu_ctx", "driver"),
        ffmpeg_version: sample_note(&hardware, "ffmpeg_capabilities", "version", output_dir)?
            .to_owned(),
        hardware_decode_accel: required_string(comparison, "/hardware/hwaccel", output_dir)?,
        hardware_decode_output_format: required_string(
            comparison,
            "/hardware/hw_output_format",
            output_dir,
        )?,
        hardware_decode_surface_format: nonempty_string(
            comparison.pointer("/hardware/hw_surface_format"),
        ),
        software_decode,
        hardware_download_decode,
        frame_zero_differing_bytes: required_u64(
            comparison,
            "/frame_zero_diff/differing_bytes",
            output_dir,
        )?,
        audio_mad,
    };
    Ok((fixture, entry))
}

pub fn compare_m4_validation_bundles(
    output_dirs: &[PathBuf],
    repository_revision: &str,
) -> Result<M4ValidationMatrix, MatrixError> {
    if output_dirs.len() < 2 {
        return Err(MatrixError::TooFewBundles);
    }
    let mut expected_fixture = None;
    let mut entries = Vec::with_capacity(output_dirs.len());
    for output_dir in output_dirs {
        let verification = verify_m4_validation_bundle(output_dir, repository_revision)?;
        if !verification.local_evidence_valid {
            return Err(MatrixError::InvalidBundle {
                path: output_dir.clone(),
                failures: verification.failures,
            });
        }
        let (fixture, entry) = matrix_entry(output_dir)?;
        if let Some(expected) = &expected_fixture {
            ensure_fixture_matches(output_dir, expected, &fixture)?;
        } else {
            expected_fixture = Some(fixture.clone());
        }
        entries.push(entry);
    }
    let (fixture_bytes, fixture_sha256) = expected_fixture.ok_or(MatrixError::MissingFixture)?;
    Ok(M4ValidationMatrix {
        schema_version: M4_VALIDATION_MATRIX_SCHEMA_VERSION,
        repository_revision: repository_revision.to_owned(),
        fixture_bytes,
        fixture_sha256,
        thresholds_selected: false,
        repetition_policy_selected: false,
        low_spec_windows_gate_closed: false,
        entries,
    })
}

pub fn verify_m4_validation_bundle(
    output_dir: &Path,
    repository_revision: &str,
) -> Result<M4ValidationVerification, VerificationError> {
    let manifest_path = output_dir.join("manifest.json");
    let hardware_path = output_dir.join("hardware.json");
    let context_path = output_dir.join("context.json");
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
    let context_evidence = file_evidence(&context_path)?;
    let hardware = read_value(&hardware_path)?;
    let (hardware_os, hardware_arch) = verify_hardware(&hardware, &mut failures);
    let context = read_value(&context_path)?;
    let (machine_label, intended_persona) = verify_context(&context, &mut failures);
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
            || record.context_sha256 != context_evidence.sha256
        {
            failures.push(format!(
                "{}: manifest, hardware, or context digest mismatch",
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
        machine_label,
        intended_persona,
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
    fn measurement_context_requires_explicit_comparable_conditions() {
        let mut failures = Vec::new();
        let context = json!({
            "schema_version": crate::perf::M4_VALIDATION_CONTEXT_SCHEMA_VERSION,
            "machine_label": "candidate-01",
            "intended_persona": "low-spec-windows-candidate",
            "power_source": "unknown",
            "power_mode": "",
            "display_width_px": 0,
            "display_height_px": 1080
        });
        let labels = verify_context(&context, &mut failures);
        assert_eq!(labels.0.as_deref(), Some("candidate-01"));
        assert_eq!(labels.1.as_deref(), Some("low-spec-windows-candidate"));
        assert_eq!(failures.len(), 3);
        assert!(failures
            .iter()
            .any(|failure| failure.contains("required measurement label")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("power source")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("display_width_px")));
    }

    #[test]
    fn matrix_entry_extracts_only_declared_raw_measurements() {
        let dir = crate::tmp_dir("m4-matrix-entry");
        let context = json!({
            "machine_label": "fixture-machine",
            "intended_persona": "candidate",
            "power_source": "ac",
            "power_mode": "balanced",
            "display_width_px": 1280,
            "display_height_px": 720
        });
        let hardware = json!({
            "hardware": {
                "os": "windows",
                "arch": "x86_64",
                "logical_cpu_count": 4,
                "total_memory_bytes": 8_589_934_592_u64
            },
            "samples": [
                {
                    "id": "headless_gpu_ctx",
                    "notes": {
                        "adapter_name": "fixture-gpu",
                        "backend": "Dx12",
                        "device_type": "IntegratedGpu",
                        "driver": "fixture-driver"
                    }
                },
                {
                    "id": "ffmpeg_capabilities",
                    "notes": {
                        "version": "fixture-ffmpeg"
                    }
                }
            ]
        });
        let decode = json!({
            "schema_version": 3,
            "fixture_bytes": 1234,
            "fixture_sha256": "fixture-sha",
            "command_route_comparison": {
                "software": {
                    "sequential": { "elapsed_ms": 10.0 },
                    "parallel_wall_ms": 20.0
                },
                "hardware": {
                    "hwaccel": "d3d11va",
                    "hw_output_format": "d3d11",
                    "hw_surface_format": "nv12",
                    "sequential": { "elapsed_ms": 30.0 },
                    "parallel_wall_ms": 40.0
                },
                "frame_zero_diff": { "differing_bytes": 0 }
            }
        });
        let audio = json!({
            "clip_count": 1000,
            "effects_per_clip": 3,
            "sequential": [{
                "elapsed_ms": 5.0,
                "active_video_slots": 4,
                "graph_steps": 1016
            }],
            "scrub": [{
                "elapsed_ms": 2.0,
                "active_video_slots": 3,
                "graph_steps": 1008
            }]
        });
        for (name, value) in [
            ("context.json", context),
            ("hardware.json", hardware),
            ("decode-hardware-download.json", decode),
            ("audio-mad-graph.json", audio),
        ] {
            std::fs::write(dir.join(name), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
        let (fixture, entry) = matrix_entry(&dir).unwrap();
        assert_eq!(fixture, (1234, "fixture-sha".into()));
        assert_eq!(entry.hardware_os, "windows");
        assert_eq!(entry.hardware_decode_accel, "d3d11va");
        assert_eq!(entry.ffmpeg_version, "fixture-ffmpeg");
        assert_eq!(entry.software_decode.sequential_120_frames_ms, 10.0);
        assert_eq!(
            entry.hardware_download_decode.parallel_8_requests_wall_ms,
            40.0
        );
        assert_eq!(entry.audio_mad.max_active_video_slots, 4);
        assert_eq!(entry.audio_mad.max_graph_steps, 1016);
        assert_eq!(entry.frame_zero_differing_bytes, 0);
    }

    #[test]
    fn comparison_rejects_fixture_drift_and_single_bundle() {
        let path = Path::new("candidate");
        let mismatch =
            ensure_fixture_matches(path, &(100, "first".into()), &(101, "second".into()))
                .unwrap_err();
        assert!(matches!(mismatch, MatrixError::FixtureMismatch { .. }));
        assert!(matches!(
            compare_m4_validation_bundles(&[path.into()], "revision"),
            Err(MatrixError::TooFewBundles)
        ));
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
        let context = json!({
            "schema_version": crate::perf::M4_VALIDATION_CONTEXT_SCHEMA_VERSION,
            "machine_label": "fixture-machine",
            "intended_persona": "candidate-only",
            "power_source": "ac",
            "power_mode": "balanced",
            "display_width_px": 1920,
            "display_height_px": 1080
        });
        std::fs::write(
            dir.join("context.json"),
            serde_json::to_vec_pretty(&context).unwrap(),
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
