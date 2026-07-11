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
///
/// マッチ規則: 文字列リテラル`"SKIP`を含む非コメント行。
/// - `eprintln!`との同一行ANDにしない理由: rustfmtが長いマクロ呼び出しを
///   折り返すと`eprintln!(`と`"SKIP: ..."`が別行になり見逃す(実測でfmtは
///   この折り返しを行う)。リテラル自体を対象にすれば折り返し行を拾える
/// - マクロ名を限定しない理由: `println!`/`log::warn!`等での手書きスキップも
///   同じ抜け道になるため、出力手段によらず「SKIPを印字する行為」を検出する
/// - コメント行(`//`開始)は除外: ルールを説明するコメントを誤検出しない
///
/// **検出できない残余(限界の明文化)**: (a) 小文字`"skip`等の別表記
/// (b) 何も印字しない無音return(`let Ok(_) = ... else { return }`)。
/// これらは字句走査では原理的に拾えないため、レビュー対象として残る。
/// 環境レベルの保証はカナリア(依存の実在主張)とヘルパーのForbid panicが
/// 担い、本走査は「スキップするならヘルパーを通せ」という規約層を守る。
fn line_is_hand_rolled_skip(line: &str) -> bool {
    !line.trim_start().starts_with("//") && line.contains(r#""SKIP"#)
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
    // 正例: rustfmtの折り返しでリテラルが単独行になった形(旧マッチャの見逃し)
    assert!(line_is_hand_rolled_skip(
        r#"            "SKIP: ffmpeg/ffprobe not found on PATH","#
    ));
    // 正例: 別マクロ経由の手書きスキップ(旧マッチャの見逃し)
    assert!(line_is_hand_rolled_skip(r#"println!("SKIP")"#));
    assert!(line_is_hand_rolled_skip(
        r#"log::warn!("SKIP: no adapter");"#
    ));
    // 負例: ルールを説明するコメントは拾わない(誤検出の抑制)
    assert!(!line_is_hand_rolled_skip(
        r#"// eprintln!("SKIP: ...") は禁止(ポリシー迂回)"#
    ));
    assert!(!line_is_hand_rolled_skip("    // SKIPについてのコメント"));
    // 負例: 無関係の出力・SKIPを含まないリテラル
    assert!(!line_is_hand_rolled_skip(r#"eprintln!("hello");"#));
    // 負例: 引用符なしのSKIP(識別子・コメント語)は対象外
    assert!(!line_is_hand_rolled_skip("let skip_count = 0; // SKIP"));
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
