//! `cargo bench -p motolii-testkit --bench perf_startup` の入口。
//!
//! 統合テスト `perf_harness` と同じ計測を実行し、CIログまたは
//! `MOTOLII_PERF_BASELINE_OUT` へベースラインを記録する。

use motolii_testkit::perf::{emit_baseline, run_harness};

fn main() {
    let report = run_harness();
    if let Err(err) = emit_baseline(&report) {
        eprintln!("perf baseline emit failed: {err}");
        std::process::exit(1);
    }
}
