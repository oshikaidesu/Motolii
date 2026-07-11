//! M2E-1: スキップ方針の負例テストと、CI環境の完全性を主張するカナリア。
//!
//! 判定ロジック(`skip_decision`/`apply_skip_decision`)は環境変数・実GPUに
//! 依存しない純関数なので、「必須環境で依存が無ければ赤」を通常のCI環境の
//! 欠損に依存せず検証できる(ゲート完了条件(a)の方式)。

use motolii_testkit::{apply_skip_decision, deps_required, skip_decision, SkipDecision};

#[test]
fn decision_matrix_covers_all_cases() {
    // 依存あり → 必須かどうかに関わらず実行
    assert_eq!(skip_decision(true, false), SkipDecision::Run);
    assert_eq!(skip_decision(true, true), SkipDecision::Run);
    // 依存なし+必須でない(ローカル) → スキップ許可
    assert_eq!(skip_decision(false, false), SkipDecision::Skip);
    // 依存なし+必須(CI) → スキップ禁止
    assert_eq!(skip_decision(false, true), SkipDecision::Forbid);
}

#[test]
fn apply_run_returns_true_and_skip_returns_false() {
    assert!(apply_skip_decision("dep", SkipDecision::Run, ""));
    assert!(!apply_skip_decision("dep", SkipDecision::Skip, "not found"));
}

/// 負例: Forbidはpanicする(=CIが赤になる)ことの直接検証。
#[test]
#[should_panic(expected = "silent skip is forbidden")]
fn apply_forbid_panics() {
    apply_skip_decision("GPU adapter", SkipDecision::Forbid, "no adapter");
}

/// カナリア: `MOTOLII_REQUIRE_GPU=1`の環境(CI)で、GPUとffmpeg/ffprobeが
/// 実際に使えることを主張する。依存が欠けたCIイメージ(mesaインストール
/// 失敗等)では、このテストが最初に赤になる番犬。
///
/// ローカル(環境変数なし)ではスキップ — 開発機にGPU/ffmpegを強制しない。
#[test]
fn ci_canary_gpu_and_ffmpeg_present() {
    if !deps_required() {
        eprintln!("SKIP: MOTOLII_REQUIRE_GPU is not set (local run)");
        return;
    }
    motolii_gpu::GpuCtx::new_headless()
        .expect("canary: GPU adapter must be available when MOTOLII_REQUIRE_GPU=1");
    for bin in ["ffmpeg", "ffprobe"] {
        let ok = std::process::Command::new(bin)
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        assert!(
            ok,
            "canary: {bin} must be on PATH when MOTOLII_REQUIRE_GPU=1"
        );
    }
}
