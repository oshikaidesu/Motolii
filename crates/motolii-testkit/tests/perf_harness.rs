//! M3E-2: 性能ハーネスが CI / `cargo test` で実行され、ベースライン記録口があることの検証。
//!
//! 数値閾値は固定しない(M3ガード10)。構造と記録経路だけを審判する。

use motolii_testkit::perf::{emit_baseline, run_harness, SampleStatus, SCHEMA_VERSION};

#[test]
fn perf_harness_records_baseline_without_thresholds() {
    let report = run_harness();

    assert_eq!(report.schema_version, SCHEMA_VERSION);
    assert_eq!(report.harness, "motolii-testkit/perf");
    assert!(!report.samples.is_empty());
    assert!(!report.external_bench_slots.is_empty());

    let self_check = report
        .samples
        .iter()
        .find(|s| s.id == "harness_self_check")
        .expect("harness_self_check sample");
    assert_eq!(self_check.status, SampleStatus::Ok);
    assert!(self_check.startup_ms.is_some());

    let registry = report
        .samples
        .iter()
        .find(|s| s.id == "plugin_registry_init")
        .expect("plugin_registry_init sample");
    assert_eq!(registry.status, SampleStatus::Ok);

    // GPUは環境依存。閾値は設けず、サンプル存在とステータス列挙のみ。
    let gpu = report
        .samples
        .iter()
        .find(|s| s.id == "headless_gpu_ctx")
        .expect("headless_gpu_ctx sample");
    assert!(matches!(
        gpu.status,
        SampleStatus::Ok | SampleStatus::Unavailable
    ));

    emit_baseline(&report).expect("baseline emit");
}
