//! **`owns:` 宣言の根拠 token 柵**(裁定215 の施工、2026-08-23 発注)。
//!
//! `next/ui/motolii-stage-pane/tests/inverse_fence.rs` / `next/core/motolii-core/
//! tests/glam_assert_fence.rs` と同じ「禁止パターンを grep する」のではなく
//! **明示マーカーの存在を要求する**形の柵(裁定209)。ただしあの2本が「1つの
//! 危険な呼び出し構文に対して直前6行以内の注記」を要求する**呼び出し箇所単位**
//! の柵なのに対し、こちらは「`owns:` を名乗るファイルという**宣言単位**に
//! 対して、ファイルのどこかに根拠 token が1つあるか」を見る点が違う——宣言
//! 自体が呼び出し構文ではなくファイル冒頭の1回きりの宣言だから、直前N行の窓は
//! 要らない。
//!
//! ## 縛る不変(裁定215)
//!
//! `next/DECISIONS.md` 裁定215: `//! owns:` を宣言してよいのは
//! (a) それを強制する**意見を名指しできる**時、または
//! (b) **上流に相当物が無いことを実際に探して確かめた**時だけ。
//! (c) 測定器具(probes/testkit)は意見/借用判定そのものの対象外として別枠。
//! (d) 立証が足りない場合は、捏造せず「立証不足」と正直に書いてよい
//!     (でないと嘘を書く圧力になる — 発注書の明記)。
//!
//! この柵は「`//! owns:` を持つファイルが `OWNS-JUSTIFICATION(<KIND>): ...` の
//! 形の一意な token を持つか」だけを検査する。`<KIND>` は `A`/`B`/`C-PROBE`/
//! `C-TESTKIT`/`D` のいずれか。**(d) は合格として数える**——正直な「立証不足」の
//! 記録を落とすと、次に嘘を書く動機を作ってしまう(発注書の明記)。
//!
//! `OWNS-JUSTIFICATION` という文字列は地の文に自然発生しない一意な token
//! (`SAFE-INVERSE`/`SAFE-GLAM-ASSERT` と同じ設計、裁定209)。トークンさえ書けば
//! 中身の正しさまでは検証しない——それでも「根拠を一切書かずに `owns:` を
//! 宣言する」をゼロにする効果はある。
//!
//! ## 除外(2026-08-23 発注の write-set 境界、別レーン走行中)
//!
//! `core/motolii-store/` と `core/motolii-eval/` はこの発注の write-set 外
//! (別レーンが `owns:` の根拠を書いている最中)。この柵はこの2 crate を
//! **「未記入」として検出してよい(むしろ検出すべき)**——見つからなくても
//! 柵は落とさず、別枠のカウントとして記録するだけに留める。
//!
//! ## 自己参照を避ける(裁定209 の教訓)
//!
//! この柵ファイル自身は `//! owns:` から始まらない(このファイルは crate の
//! 根ではなく `tests/` 配下の統合試験)ので、自分自身を対象ファイルとして
//! 拾ってしまう心配はもとから無い——念のため、ファイル名で自分自身を除外
//! しておく。

use std::fs;
use std::path::{Path, PathBuf};

/// この crate の `tests/` から見て `next/` の根。
/// (`core/motolii-store/tests/lottie_coverage.rs` の `../../reference` と
/// 同じ深さの相対関係 — `core/motolii-testkit` も `next/` から見て2階層下)。
fn next_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const SELF_FILE_NAME: &str = "owns_justification_fence.rs";

/// `next/` 配下の全 `*.rs` を再帰収集する。`target/`(ビルド成果物)と隠しディレクトリは飛ばす
/// (`.git` 等 — この workspace の `next/` 配下には無いはずだが念のため)。
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// ファイルが `//! owns:` 宣言を持つか(check.sh の `grep -m1 -E '^\s*//! owns:'`
/// と同じ判定 — 行頭の空白を許し、`//! owns:` で始まる行を探す)。
fn declares_owns(text: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start().starts_with("//! owns:"))
}

/// `OWNS-JUSTIFICATION(<KIND>): ...` の形の doc 行を探し、`<KIND>` を返す。
/// `//!` で始まる行だけを見る(コード中の文字列リテラルや通常コメントでの
/// 言及は対象外 — `inverse_fence.rs`/`glam_assert_fence.rs` の
/// `trim_start().starts_with("//")` によるコメント除外と同じ思想だが、こちらは
/// 逆に「doc コメント **だけ** を根拠として認める」形)。
fn find_justification_kind(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("//!") {
            continue;
        }
        if let Some(after) = trimmed.split("OWNS-JUSTIFICATION(").nth(1) {
            if let Some(kind) = after.split(')').next() {
                return Some(kind.to_string());
            }
        }
    }
    None
}

#[derive(Debug, Default)]
struct Tally {
    a: usize,
    b: usize,
    c: usize,
    d: usize,
}

#[test]
fn every_owns_declaration_has_a_justification_token() {
    let root = next_root();
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    assert!(
        !files.is_empty(),
        "next/ 配下で *.rs が1つも見つからなかった — next_root() のパス計算が壊れている: {}",
        root.display()
    );

    let mut tally = Tally::default();
    let mut violations: Vec<String> = Vec::new();
    let mut malformed: Vec<String> = Vec::new();
    let mut excluded_pending: Vec<String> = Vec::new();

    for path in &files {
        if path.file_name().and_then(|n| n.to_str()) == Some(SELF_FILE_NAME) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        if !declares_owns(&text) {
            continue;
        }

        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        // 2026-08-23 発注の write-set 境界: この2 crate は別レーンが書いている
        // 最中なので、根拠 token が無くても柵は落とさない(「未記入」として
        // 別枠でカウントするだけ)。
        let is_excluded_lane =
            rel.starts_with("core/motolii-store/") || rel.starts_with("core/motolii-eval/");

        match find_justification_kind(&text) {
            None => {
                if is_excluded_lane {
                    excluded_pending.push(rel);
                } else {
                    violations.push(rel);
                }
            }
            Some(kind) => match kind.as_str() {
                "A" => tally.a += 1,
                "B" => tally.b += 1,
                "C-PROBE" | "C-TESTKIT" => tally.c += 1,
                "D" => tally.d += 1,
                other => malformed.push(format!("{rel}: 未知の分類 `{other}` (A/B/C-PROBE/C-TESTKIT/D のいずれかであること)")),
            },
        }
    }

    assert!(
        violations.is_empty() && malformed.is_empty(),
        "`//! owns:` を宣言しているのに根拠 token(`OWNS-JUSTIFICATION(...)`)が無いか、\
         分類が不正なファイルが見つかった(裁定215: owns は既定禁止・持つなら根拠必須)。\n\
         根拠なし: {violations:#?}\n分類不正: {malformed:#?}\n\
         書式: `//! OWNS-JUSTIFICATION(A|B|C-PROBE|C-TESTKIT|D): <根拠>` を doc に追記すること。\
         立証が足りないなら (D) で正直に書いてよい(捏造しないこと)。\
         (store/eval 配下は別レーン走行中のため対象外 — 未記入: {excluded_pending:#?})"
    );

    // 情報表示(fail させない、check.sh のLottie/Intent節と同じ流儀)。
    // cargo test は既定で成功時 stdout を隠すので `-- --nocapture` を付けて読むこと。
    println!(
        "owns justification tally — A={} B={} C={} D={} (捏造しない正直な立証不足)\n\
         除外(store/eval、別レーン走行中・未記入として検出)={} 件: {:#?}",
        tally.a,
        tally.b,
        tally.c,
        tally.d,
        excluded_pending.len(),
        excluded_pending
    );
}
