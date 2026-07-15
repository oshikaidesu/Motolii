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
//! # 将来の外部ベンチ拡張点
//!
//! [`EXTERNAL_BENCH_SLOTS`] にスロットを宣言し、配線時は各 `env_var` で
//! `cargo test` / CI から起動する想定。現時点では定義と記録のみ。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const BASELINE_OUT_ENV: &str = "MOTOLII_PERF_BASELINE_OUT";
pub const SCHEMA_VERSION: u32 = 1;

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
];

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
    pub samples: Vec<PerfSample>,
    pub external_bench_slots: &'static [ExternalBenchSlot],
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

/// Linux `/proc/self/status` の VmRSS を bytes で返す。他OSは `None`。
pub fn current_rss_bytes() -> Option<u64> {
    parse_vm_rss_kb(&read_proc_status()?).map(|kb| kb * 1024)
}

fn read_proc_status() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status").ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

fn parse_vm_rss_kb(status: &str) -> Option<u64> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb_str = rest.trim().trim_end_matches(" kB").trim();
            return kb_str.parse().ok();
        }
    }
    None
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
            drop(gpu);
            PerfSample {
                id: id.into(),
                status: SampleStatus::Ok,
                startup_ms: Some(startup_ms),
                idle_rss_bytes: idle_rss_after_init(Duration::from_millis(50)),
                notes: HashMap::new(),
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
        headless_gpu_ctx(),
    ];
    PerfReport {
        schema_version: SCHEMA_VERSION,
        harness: "motolii-testkit/perf",
        recorded_at_unix_ms: unix_ms_now(),
        samples,
        external_bench_slots: EXTERNAL_BENCH_SLOTS,
    }
}

/// レポートをstderrへ人間可読サマリとして出力する(CIログ用)。
pub fn log_report_summary(report: &PerfReport) {
    eprintln!("=== motolii perf harness (M3E-2) ===");
    eprintln!("schema_version={}", report.schema_version);
    eprintln!("recorded_at_unix_ms={}", report.recorded_at_unix_ms);
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
    fn measure_startup_returns_elapsed() {
        let (_, ms) = measure_startup(|| std::thread::sleep(Duration::from_millis(5)));
        assert!(ms >= 4.0);
    }

    #[test]
    fn run_harness_includes_self_check_ok() {
        let report = run_harness();
        assert_eq!(report.schema_version, SCHEMA_VERSION);
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
