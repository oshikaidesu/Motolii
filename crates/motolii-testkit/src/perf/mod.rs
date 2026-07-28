//! M3E-2 / INF-2: 性能ハーネス枠(起動時間・アイドルRSS・将来ベンチの受け皿)。
//!
//! 数値目標の閾値はここでは固定しない(M3ガード10: U1実測で決める)。
//! 本モジュールは計測・JSONベースライン記録の口だけを提供する。
//!
//! # 使い方
//!
//! ```text
//! cargo test -p motolii-testkit --test perf_harness
//! # または
//! cargo bench -p motolii-testkit --bench perf_startup
//!
//! # ベースラインJSONをファイルへ:
//! MOTOLII_PERF_BASELINE_OUT=/tmp/perf-baseline.json \
//!   cargo test -p motolii-testkit --test perf_harness -- --nocapture
//! ```
//!
//! # 外部ベンチ拡張点
//!
//! [`EXTERNAL_BENCH_SLOTS`] は実装済み／未実装を含む外部ベンチ入口を記録する。
//! M4の機種別再実行recipeは [`m4_validation_manifest`] がshell非依存のargvとして返す。

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const BASELINE_OUT_ENV: &str = "MOTOLII_PERF_BASELINE_OUT";
pub const SCHEMA_VERSION: u32 = 2;
pub const M4_VALIDATION_BUNDLE_SCHEMA_VERSION: u32 = 1;

/// 外部ベンチの呼び出し口(未配線スロット — M3E-2)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExternalBenchSlot {
    pub id: &'static str,
    pub description: &'static str,
    /// 配線後に `cargo test` から起動する際の環境変数ゲート。
    pub env_var: &'static str,
    /// 人手/CI向けの起動例(実行はしない)。
    pub invoke_hint: &'static str,
}

/// 将来配線先の台帳。レポートJSONにも含め、審判の拡張点を可視化する。
pub const EXTERNAL_BENCH_SLOTS: &[ExternalBenchSlot] = &[
    ExternalBenchSlot {
        id: "timeline-bench",
        description: "M3 guard 2: 1,000 clips + 100k keys single-texture draw (issue #57)",
        env_var: "MOTOLII_PERF_EXTERNAL_TIMELINE_BENCH",
        invoke_hint: "cd spikes/timeline-bench && cargo run --release -- --json",
    },
    ExternalBenchSlot {
        id: "render-1080p-40layer",
        description: "performance-model §7: 40 active 1080p video layers frame time (future)",
        env_var: "MOTOLII_PERF_EXTERNAL_RENDER_1080P_40",
        invoke_hint: "(not implemented — U1 measurement will define PerfScenario)",
    },
    ExternalBenchSlot {
        id: "decode-demand-matrix",
        description: "M4 validation: sequential/seek-storm/parallel clip decode demand",
        env_var: "MOTOLII_PERF_EXTERNAL_DECODE_MATRIX",
        invoke_hint: "cargo test -p motolii-media --test decode_demand_bench record_decode_demand_matrix_without_thresholds -- --ignored --nocapture",
    },
    ExternalBenchSlot {
        id: "audio-mad-edit-density",
        description:
            "M4 validation: many short clips/effects aligned to audio without timeline stalls",
        env_var: "MOTOLII_PERF_EXTERNAL_AUDIO_MAD",
        invoke_hint: "cargo test --release -p motolii-doc --test audio_mad_density_bench record_audio_mad_graph_demand_without_thresholds -- --ignored --nocapture",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationKind {
    Observation,
    Contract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationCommand {
    pub id: &'static str,
    pub kind: ValidationKind,
    pub program: &'static str,
    pub args: &'static [&'static str],
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<&'static str, String>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub required_user_env: &'static [&'static str],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub optional_user_env: &'static [&'static str],
    pub artifact: Option<&'static str>,
    pub proves: &'static str,
    pub does_not_prove: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnresolvedPolicyInput {
    pub id: &'static str,
    pub selected_value: Option<u64>,
    pub unit: &'static str,
    pub evidence_required: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalValidationGate {
    pub id: &'static str,
    pub status: &'static str,
    pub required_evidence: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct M4ValidationManifest {
    pub schema_version: u32,
    pub repository_revision: Option<String>,
    pub generated_artifacts: &'static [&'static str],
    pub commands: Vec<ValidationCommand>,
    pub unresolved_policy_inputs: &'static [UnresolvedPolicyInput],
    pub external_gates: &'static [ExternalValidationGate],
}

const DECODE_ARGS: &[&str] = &[
    "test",
    "-p",
    "motolii-media",
    "--test",
    "decode_demand_bench",
    "record_decode_demand_matrix_without_thresholds",
    "--",
    "--ignored",
    "--nocapture",
];
const AUDIO_MAD_ARGS: &[&str] = &[
    "test",
    "--release",
    "-p",
    "motolii-doc",
    "--test",
    "audio_mad_density_bench",
    "record_audio_mad_graph_demand_without_thresholds",
    "--",
    "--ignored",
    "--nocapture",
];
const LEDGER_ARGS: &[&str] = &["test", "-p", "motolii-gpu", "resource_ledger"];
const TIER_TRANSFER_ARGS: &[&str] = &[
    "test",
    "-p",
    "motolii-testkit",
    "--test",
    "m4_tier_transfer_contract",
];
const YUV_PLAN_ARGS: &[&str] = &[
    "test",
    "-p",
    "motolii-testkit",
    "--test",
    "m4_yuv_materialization_plan",
];

const UNRESOLVED_POLICY_INPUTS: &[UnresolvedPolicyInput] = &[
    UnresolvedPolicyInput {
        id: "vram_hard_budget",
        selected_value: None,
        unit: "bytes",
        evidence_required: "low-spec Windows working-set observations plus explicit product policy",
    },
    UnresolvedPolicyInput {
        id: "texture_allocation_alignment",
        selected_value: None,
        unit: "bytes",
        evidence_required: "backend allocation observations and a conservative accounting policy",
    },
    UnresolvedPolicyInput {
        id: "yuv_live_lane_cap",
        selected_value: None,
        unit: "lanes",
        evidence_required:
            "corrected product lifetime owner plus mixed-resolution active-set measurements",
    },
];

const EXTERNAL_VALIDATION_GATES: &[ExternalValidationGate] = &[
    ExternalValidationGate {
        id: "low_spec_windows",
        status: "pending",
        required_evidence: "same bundle and fixture revision on the target low-spec Windows persona",
    },
    ExternalValidationGate {
        id: "gpu_surface_import",
        status: "pending",
        required_evidence: "same decode demand sequence through a GPU-import route with pixel oracle",
    },
    ExternalValidationGate {
        id: "product_preview_path",
        status: "pending",
        required_evidence: "decode, upload/import, render, display, cancellation, and queue depth in Motolii Studio Preview",
    },
];

pub fn m4_validation_manifest(
    repository_revision: Option<String>,
    artifact_dir: impl AsRef<Path>,
) -> M4ValidationManifest {
    let artifact_dir = artifact_dir.as_ref();
    let artifact_path = |name: &str| artifact_dir.join(name).display().to_string();
    let software_env = BTreeMap::from([(
        "MOTOLII_DECODE_DEMAND_OUT",
        artifact_path("decode-software.json"),
    )]);
    let hardware_env = BTreeMap::from([(
        "MOTOLII_DECODE_DEMAND_OUT",
        artifact_path("decode-hardware-download.json"),
    )]);
    let audio_mad_env = BTreeMap::from([(
        "MOTOLII_AUDIO_MAD_DEMAND_OUT",
        artifact_path("audio-mad-graph.json"),
    )]);
    M4ValidationManifest {
        schema_version: M4_VALIDATION_BUNDLE_SCHEMA_VERSION,
        repository_revision,
        generated_artifacts: &["manifest.json", "hardware.json"],
        commands: vec![
            ValidationCommand {
                id: "decode-software",
                kind: ValidationKind::Observation,
                program: "cargo",
                args: DECODE_ARGS,
                env: software_env,
                required_user_env: &[],
                optional_user_env: &["MOTOLII_DECODE_FIXTURE"],
                artifact: Some("decode-software.json"),
                proves: "software decode demand for sequential, seek, and parallel requests",
                does_not_prove: "hardware decode, GPU import, preview latency, or a minimum specification",
            },
            ValidationCommand {
                id: "decode-hardware-download",
                kind: ValidationKind::Observation,
                program: "cargo",
                args: DECODE_ARGS,
                env: hardware_env,
                required_user_env: &[
                    "MOTOLII_DECODE_HWACCEL",
                    "MOTOLII_DECODE_HW_OUTPUT_FORMAT",
                ],
                optional_user_env: &[
                    "MOTOLII_DECODE_FIXTURE",
                    "MOTOLII_DECODE_HW_SURFACE_FORMAT",
                ],
                artifact: Some("decode-hardware-download.json"),
                proves: "an explicitly configured hardware-surface-to-CPU-download comparison",
                does_not_prove: "zero-copy GPU import or that hardware decode is faster",
            },
            ValidationCommand {
                id: "audio-mad-graph-demand",
                kind: ValidationKind::Observation,
                program: "cargo",
                args: AUDIO_MAD_ARGS,
                env: audio_mad_env,
                required_user_env: &[],
                optional_user_env: &[],
                artifact: Some("audio-mad-graph.json"),
                proves: "Document-to-render-graph demand for the fixed 1,000-clip fixture",
                does_not_prove: "decode, GPU render, display, UI responsiveness, or preview latency",
            },
            ValidationCommand {
                id: "resource-ledger-contract",
                kind: ValidationKind::Contract,
                program: "cargo",
                args: LEDGER_ARGS,
                env: BTreeMap::new(),
                required_user_env: &[],
                optional_user_env: &[],
                artifact: None,
                proves: "typed hard-cap and accounting invariants",
                does_not_prove: "a safe numeric budget for any device",
            },
            ValidationCommand {
                id: "tier-transfer-contract",
                kind: ValidationKind::Contract,
                program: "cargo",
                args: TIER_TRANSFER_ARGS,
                env: BTreeMap::new(),
                required_user_env: &[],
                optional_user_env: &[],
                artifact: None,
                proves: "source retention, double-residency, LRU, cancellation, and stale-generation negatives",
                does_not_prove: "product transfer throughput or eviction thresholds",
            },
            ValidationCommand {
                id: "yuv-materialization-plan-contract",
                kind: ValidationKind::Contract,
                program: "cargo",
                args: YUV_PLAN_ARGS,
                env: BTreeMap::new(),
                required_user_env: &[],
                optional_user_env: &[],
                artifact: None,
                proves: "size-keyed lane reuse and atomic refusal in the test-only planner",
                does_not_prove: "that the product YUV lifetime alias is fixed",
            },
        ],
        unresolved_policy_inputs: UNRESOLVED_POLICY_INPUTS,
        external_gates: EXTERNAL_VALIDATION_GATES,
    }
}

pub fn write_m4_validation_bundle(
    output_dir: impl AsRef<Path>,
    repository_revision: Option<String>,
) -> Result<M4ValidationManifest, BaselineError> {
    let output_dir = output_dir.as_ref();
    std::fs::create_dir_all(output_dir).map_err(|source| BaselineError::CreateDir {
        path: output_dir.to_path_buf(),
        source,
    })?;
    let hardware = run_harness();
    write_baseline_json(output_dir.join("hardware.json"), &hardware)?;
    let manifest = m4_validation_manifest(repository_revision, output_dir);
    write_serialized_json(output_dir.join("manifest.json"), &manifest)?;
    Ok(manifest)
}

fn write_serialized_json(
    path: impl AsRef<Path>,
    value: &impl Serialize,
) -> Result<(), BaselineError> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(value).map_err(BaselineError::Serialize)?;
    std::fs::write(path, json).map_err(|source| BaselineError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleStatus {
    Ok,
    Skipped,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PerfSample {
    pub id: String,
    pub status: SampleStatus,
    /// 初期化完了までの経過[ms]。未計測なら `None`。
    pub startup_ms: Option<f64>,
    /// アイドル時のプロセスRSS[bytes]。取得不可プラットフォームでは `None`。
    pub idle_rss_bytes: Option<u64>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub notes: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PerfReport {
    pub schema_version: u32,
    pub harness: &'static str,
    pub recorded_at_unix_ms: u64,
    pub hardware: HardwareProfile,
    pub samples: Vec<PerfSample>,
    pub external_bench_slots: &'static [ExternalBenchSlot],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HardwareProfile {
    pub os: &'static str,
    pub arch: &'static str,
    pub logical_cpu_count: Option<usize>,
    pub total_memory_bytes: Option<u64>,
}

/// 初期化クロージャの所要時間[ms]を計測する。
pub fn measure_startup<F, T>(init: F) -> (T, f64)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let value = init();
    let startup_ms = start.elapsed().as_secs_f64() * 1000.0;
    (value, startup_ms)
}

/// Linux `/proc/self/status` またはmacOS `ps`のRSSをbytesで返す。
pub fn current_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        parse_vm_rss_kb(&read_proc_status()?).and_then(|kb| kb.checked_mul(1024))
    }
    #[cfg(target_os = "macos")]
    {
        let pid = std::process::id().to_string();
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout)
            .ok()?
            .trim()
            .parse::<u64>()
            .ok()?
            .checked_mul(1024)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn read_proc_status() -> Option<String> {
    std::fs::read_to_string("/proc/self/status").ok()
}

#[cfg(any(target_os = "linux", test))]
fn parse_vm_rss_kb(status: &str) -> Option<u64> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb_str = rest.trim().trim_end_matches(" kB").trim();
            return kb_str.parse().ok();
        }
    }
    None
}

#[cfg(any(target_os = "linux", test))]
fn parse_mem_total_kb(meminfo: &str) -> Option<u64> {
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            return rest.trim().trim_end_matches(" kB").trim().parse().ok();
        }
    }
    None
}

fn total_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
        parse_mem_total_kb(&meminfo)?.checked_mul(1024)
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8(output.stdout).ok()?.trim().parse().ok()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

fn hardware_profile() -> HardwareProfile {
    HardwareProfile {
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        logical_cpu_count: std::thread::available_parallelism().ok().map(usize::from),
        total_memory_bytes: total_memory_bytes(),
    }
}

/// 初期化直後に短いアイドル待ちを入れてRSSを読む。
pub fn idle_rss_after_init(idle: Duration) -> Option<u64> {
    std::thread::sleep(idle);
    current_rss_bytes()
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn harness_self_check() -> PerfSample {
    let (_, startup_ms) = measure_startup(|| ());
    PerfSample {
        id: "harness_self_check".into(),
        status: SampleStatus::Ok,
        startup_ms: Some(startup_ms),
        idle_rss_bytes: current_rss_bytes(),
        notes: HashMap::new(),
    }
}

fn headless_gpu_ctx() -> PerfSample {
    let id = "headless_gpu_ctx";
    let (result, startup_ms) = measure_startup(motolii_gpu::GpuCtx::new_headless);
    match result {
        Ok(gpu) => {
            let mut notes = HashMap::new();
            if let Some(info) = &gpu.adapter_info {
                notes.insert("adapter_name".into(), info.name.clone());
                notes.insert("backend".into(), format!("{:?}", info.backend));
                notes.insert("device_type".into(), format!("{:?}", info.device_type));
                notes.insert("driver".into(), info.driver.clone());
                notes.insert("driver_info".into(), info.driver_info.clone());
            }
            drop(gpu);
            PerfSample {
                id: id.into(),
                status: SampleStatus::Ok,
                startup_ms: Some(startup_ms),
                idle_rss_bytes: idle_rss_after_init(Duration::from_millis(50)),
                notes,
            }
        }
        Err(e) => {
            let mut notes = HashMap::new();
            notes.insert("error".into(), e.to_string());
            PerfSample {
                id: id.into(),
                status: SampleStatus::Unavailable,
                startup_ms: Some(startup_ms),
                idle_rss_bytes: current_rss_bytes(),
                notes,
            }
        }
    }
}

fn ffmpeg_capabilities() -> PerfSample {
    let id = "ffmpeg_capabilities";
    let start = Instant::now();
    let version = Command::new("ffmpeg").arg("-version").output();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    let Ok(version) = version else {
        return PerfSample {
            id: id.into(),
            status: SampleStatus::Unavailable,
            startup_ms: Some(elapsed_ms),
            idle_rss_bytes: current_rss_bytes(),
            notes: HashMap::new(),
        };
    };
    if !version.status.success() {
        let mut notes = HashMap::new();
        notes.insert(
            "error".into(),
            String::from_utf8_lossy(&version.stderr).into(),
        );
        return PerfSample {
            id: id.into(),
            status: SampleStatus::Unavailable,
            startup_ms: Some(elapsed_ms),
            idle_rss_bytes: current_rss_bytes(),
            notes,
        };
    }

    let mut notes = HashMap::new();
    let version_text = String::from_utf8_lossy(&version.stdout);
    if let Some(line) = version_text.lines().next() {
        notes.insert("version".into(), line.to_owned());
    }
    match Command::new("ffmpeg")
        .args(["-hide_banner", "-hwaccels"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let accelerators = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with("Hardware acceleration"))
                .collect::<Vec<_>>()
                .join(",");
            notes.insert("hwaccels".into(), accelerators);
        }
        Ok(output) => {
            notes.insert(
                "hwaccels_error".into(),
                String::from_utf8_lossy(&output.stderr).into(),
            );
        }
        Err(error) => {
            notes.insert("hwaccels_error".into(), error.to_string());
        }
    }
    PerfSample {
        id: id.into(),
        status: SampleStatus::Ok,
        startup_ms: Some(elapsed_ms),
        idle_rss_bytes: current_rss_bytes(),
        notes,
    }
}

fn plugin_registry_init() -> PerfSample {
    let id = "plugin_registry_init";
    let (registry, startup_ms) = measure_startup(motolii_plugin::PluginRegistry::new);
    drop(registry);
    PerfSample {
        id: id.into(),
        status: SampleStatus::Ok,
        startup_ms: Some(startup_ms),
        idle_rss_bytes: idle_rss_after_init(Duration::from_millis(10)),
        notes: HashMap::new(),
    }
}

/// 内蔵シナリオを実行してレポートを組み立てる。
pub fn run_harness() -> PerfReport {
    let samples = vec![
        harness_self_check(),
        plugin_registry_init(),
        ffmpeg_capabilities(),
        headless_gpu_ctx(),
    ];
    PerfReport {
        schema_version: SCHEMA_VERSION,
        harness: "motolii-testkit/perf",
        recorded_at_unix_ms: unix_ms_now(),
        hardware: hardware_profile(),
        samples,
        external_bench_slots: EXTERNAL_BENCH_SLOTS,
    }
}

/// レポートをstderrへ人間可読サマリとして出力する(CIログ用)。
pub fn log_report_summary(report: &PerfReport) {
    eprintln!("=== motolii perf harness (M3E-2) ===");
    eprintln!("schema_version={}", report.schema_version);
    eprintln!("recorded_at_unix_ms={}", report.recorded_at_unix_ms);
    eprintln!(
        "hardware os={} arch={} logical_cpu_count={:?} total_memory_bytes={:?}",
        report.hardware.os,
        report.hardware.arch,
        report.hardware.logical_cpu_count,
        report.hardware.total_memory_bytes
    );
    for sample in &report.samples {
        eprintln!(
            "  [{}] status={:?} startup_ms={:?} idle_rss_bytes={:?}",
            sample.id, sample.status, sample.startup_ms, sample.idle_rss_bytes
        );
    }
    eprintln!("external_bench_slots={}", report.external_bench_slots.len());
    for slot in report.external_bench_slots {
        eprintln!(
            "  slot {} env={} hint={}",
            slot.id, slot.env_var, slot.invoke_hint
        );
    }
}

/// `MOTOLII_PERF_BASELINE_OUT` が設定されていればJSONを書き出す。
pub fn emit_baseline(report: &PerfReport) -> Result<Option<PathBuf>, BaselineError> {
    log_report_summary(report);
    let Some(path) = baseline_out_from_env() else {
        return Ok(None);
    };
    write_baseline_json(&path, report)?;
    eprintln!("perf baseline written to {}", path.display());
    Ok(Some(path))
}

pub fn baseline_out_from_env() -> Option<PathBuf> {
    std::env::var_os(BASELINE_OUT_ENV).map(PathBuf::from)
}

pub fn write_baseline_json(
    path: impl AsRef<Path>,
    report: &PerfReport,
) -> Result<(), BaselineError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|source| BaselineError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    let json = serde_json::to_string_pretty(report).map_err(BaselineError::Serialize)?;
    std::fs::write(path, json).map_err(|source| BaselineError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error("failed to create baseline directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize perf baseline: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to write baseline to {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vm_rss_from_status_text() {
        let status = "Name:\tcargo\nVmRSS:\t  12345 kB\n";
        assert_eq!(parse_vm_rss_kb(status), Some(12345));
    }

    #[test]
    fn parse_total_memory_from_meminfo_text() {
        let meminfo = "MemTotal:       16384256 kB\nMemFree:         1024 kB\n";
        assert_eq!(parse_mem_total_kb(meminfo), Some(16_384_256));
    }

    #[test]
    fn measure_startup_returns_elapsed() {
        let (_, ms) = measure_startup(|| std::thread::sleep(Duration::from_millis(5)));
        assert!(ms >= 4.0);
    }

    #[test]
    fn run_harness_includes_self_check_ok() {
        let report = run_harness();
        assert_eq!(report.schema_version, SCHEMA_VERSION);
        assert_eq!(report.hardware.os, std::env::consts::OS);
        assert_eq!(report.hardware.arch, std::env::consts::ARCH);
        assert!(report.hardware.logical_cpu_count.is_some());
        let self_check = report
            .samples
            .iter()
            .find(|s| s.id == "harness_self_check")
            .expect("self check sample");
        assert_eq!(self_check.status, SampleStatus::Ok);
        assert!(self_check.startup_ms.is_some());
    }

    #[test]
    fn baseline_json_roundtrip_fields() {
        let report = run_harness();
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("external_bench_slots"));
        assert!(json.contains("timeline-bench"));
        assert!(json.contains("render-1080p-40layer"));
    }
}
