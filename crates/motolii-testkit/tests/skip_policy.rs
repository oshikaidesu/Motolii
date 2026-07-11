//! M2E-1: スキップ方針の負例テストと、CI環境の完全性を主張するカナリア。
//!
//! 判定ロジック(`skip_decision`/`apply_skip_decision`)は環境変数・実GPUに
//! 依存しない純関数なので、「必須環境で依存が無ければ赤」を通常のCI環境の
//! 欠損に依存せず検証できる(ゲート完了条件(a)の方式)。
//!
//! さらに、手書きスキップ(`eprintln!("SKIP...`)をソース走査でdenyする。
//! 手書きスキップはポリシーを迂回する抜け道 — カナリアだけ成功して個別
//! テストが無音スキップされる穴 — になるため、スキップは必ずtestkitの
//! ヘルパー(`gpu_or_skip`/`ffmpeg_or_skip`/`unavailable_dep`)を通す。

use std::fs;
use std::path::Path;

use motolii_testkit::{
    apply_skip_decision, deps_required, skip_decision, tool_status, SkipDecision, ToolStatus,
};

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
/// ffmpeg/ffprobeは「未導入」と「導入済みだが実行失敗」を区別して報告する。
#[test]
fn ci_canary_gpu_and_ffmpeg_present() {
    if !deps_required() {
        eprintln!("SKIP: MOTOLII_REQUIRE_GPU is not set (local run)");
        return;
    }
    motolii_gpu::GpuCtx::new_headless()
        .expect("canary: GPU adapter must be available when MOTOLII_REQUIRE_GPU=1");
    for bin in ["ffmpeg", "ffprobe"] {
        match tool_status(bin) {
            ToolStatus::Ok => {}
            ToolStatus::NotInstalled => panic!(
                "canary: {bin} is not installed (not on PATH) — \
                 the CI image is missing the dependency"
            ),
            ToolStatus::Failed(detail) => panic!(
                "canary: {bin} is installed but failed to run — \
                 broken installation, not a missing one: {detail}"
            ),
        }
    }
}

// --- 手書きスキップの走査deny ---

/// 判定本体(文字列マッチ)。走査テストから分離して正例/負例を単体テスト可能に。
fn line_is_hand_rolled_skip(line: &str) -> bool {
    line.contains("eprintln!") && line.contains("SKIP")
}

#[test]
fn matcher_detects_hand_rolled_skip() {
    // 正例: かつて実在した手書きスキップの形
    assert!(line_is_hand_rolled_skip(
        r#"eprintln!("SKIP: ffmpeg not found");"#
    ));
    assert!(line_is_hand_rolled_skip(
        r#"eprintln!("SKIP: no GPU adapter");"#
    ));
    // 負例: コメントや無関係の出力は拾わない
    assert!(!line_is_hand_rolled_skip("// SKIPについてのコメント"));
    assert!(!line_is_hand_rolled_skip(r#"eprintln!("hello");"#));
    assert!(!line_is_hand_rolled_skip(r#"println!("SKIP")"#));
}

/// testkit以外の全クレートのソースから手書きスキップをdenyする。
/// (testkit自身はポリシー実装としてSKIP出力を持つため除外)
#[test]
fn no_hand_rolled_skip_paths_outside_testkit() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates dir");
    let mut violations = Vec::new();
    scan_dir(crates_dir, &mut violations);
    assert!(
        violations.is_empty(),
        "手書きスキップはM2E-1のポリシー(REQUIRE時panic)を迂回する抜け道。\
         testkitのgpu_or_skip / ffmpeg_or_skip / unavailable_dep経由に置き換えること:\n{}",
        violations.join("\n")
    );
}

fn scan_dir(dir: &Path, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            // testkit自身(ポリシー実装+本走査)とビルド成果物は対象外
            if name == "motolii-testkit" || name == "target" {
                continue;
            }
            scan_dir(&path, violations);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            for (i, line) in content.lines().enumerate() {
                if line_is_hand_rolled_skip(line) {
                    violations.push(format!("{}:{}: {}", path.display(), i + 1, line.trim()));
                }
            }
        }
    }
}
