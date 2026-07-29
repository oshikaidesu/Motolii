//! M3E-2: 性能ハーネスが CI / `cargo test` で実行され、ベースライン記録口があることの検証。
//!
//! 数値閾値は固定しない(M3ガード10)。構造と記録経路だけを審判する。

use motolii_testkit::perf::{
    emit_baseline, m4_validation_manifest, run_harness, write_m4_validation_bundle, SampleStatus,
    ValidationKind, M4_VALIDATION_BUNDLE_SCHEMA_VERSION, SCHEMA_VERSION,
};

#[test]
fn perf_harness_records_baseline_without_thresholds() {
    let report = run_harness();

    assert_eq!(report.schema_version, SCHEMA_VERSION);
    assert_eq!(report.harness, "motolii-testkit/perf");
    assert_eq!(report.hardware.os, std::env::consts::OS);
    assert_eq!(report.hardware.arch, std::env::consts::ARCH);
    assert!(report.hardware.logical_cpu_count.is_some());
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

    let ffmpeg = report
        .samples
        .iter()
        .find(|s| s.id == "ffmpeg_capabilities")
        .expect("ffmpeg_capabilities sample");
    assert!(matches!(
        ffmpeg.status,
        SampleStatus::Ok | SampleStatus::Unavailable
    ));

    emit_baseline(&report).expect("baseline emit");
}

#[test]
fn m4_bundle_separates_observations_from_unresolved_policy() {
    let artifact_dir = std::env::temp_dir().join("motolii-m4-manifest-contract");
    let manifest = m4_validation_manifest(Some("fixture-revision".into()), &artifact_dir);

    assert_eq!(manifest.schema_version, M4_VALIDATION_BUNDLE_SCHEMA_VERSION);
    assert_eq!(
        manifest.repository_revision.as_deref(),
        Some("fixture-revision")
    );
    assert!(manifest.commands.iter().any(|command| {
        command.id == "decode-software" && command.kind == ValidationKind::Observation
    }));
    assert!(manifest.commands.iter().any(|command| {
        command.id == "resource-ledger-contract" && command.kind == ValidationKind::Contract
    }));
    assert!(manifest
        .commands
        .iter()
        .all(|command| command.working_directory == "repository_root"));
    assert!(manifest
        .unresolved_policy_inputs
        .iter()
        .all(|input| input.selected_value.is_none()));
    assert!(manifest
        .external_gates
        .iter()
        .all(|gate| gate.status == "pending"));
    let external_gate_ids: Vec<_> = manifest.external_gates.iter().map(|gate| gate.id).collect();
    assert_eq!(
        external_gate_ids,
        [
            "low_spec_windows",
            "native_decoder_surface_import",
            "wgpu_external_texture_lowering",
            "surface_lifetime_fence",
            "gpu_surface_pixel_oracle",
            "product_preview_path",
        ]
    );
    assert!(!external_gate_ids.contains(&"gpu_surface_import"));
    let hardware = manifest
        .commands
        .iter()
        .find(|command| command.id == "decode-hardware-download")
        .expect("hardware-download command");
    assert!(!hardware.env.contains_key("MOTOLII_DECODE_HWACCEL"));
    assert!(hardware
        .required_user_env
        .contains(&"MOTOLII_DECODE_HWACCEL"));
    assert!(!hardware
        .optional_user_env
        .contains(&"MOTOLII_DECODE_HWACCEL"));
}

#[test]
fn m4_bundle_writes_only_manifest_and_hardware_inventory() {
    let output_dir = motolii_testkit::tmp_dir("m4-validation-bundle");
    let manifest =
        write_m4_validation_bundle(&output_dir, Some("fixture-revision".into())).unwrap();

    assert!(output_dir.join("manifest.json").is_file());
    assert!(output_dir.join("hardware.json").is_file());
    assert_eq!(manifest.generated_artifacts.len(), 2);
    assert!(!output_dir.join("decode-software.json").exists());
    assert!(!output_dir.join("audio-mad-graph.json").exists());
}
