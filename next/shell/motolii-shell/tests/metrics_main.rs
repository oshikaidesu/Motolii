//! 統合ハーネス(裁定138)の2本目。`render_pipeline_fence.rs` だけをここへ
//! 分離している理由は `tests/suite_main.rs` の doc comment を参照
//! (`motolii_shell::metrics` のプロセス共有 static を他ファイルの並列実行から
//! 汚染されないため、別プロセス=別バイナリのまま残す)。
//!
//! **裁定171 v2(M4)**: この1バイナリ内でも `#[test]` は既定で並列実行される
//! ので、`metrics::*` を触る2本(`render_pipeline_fence`/
//! `zero_copy_presenter_fence`)は**同じ1本の lock** を共有する必要がある —
//! モジュールごとに別々の `static METRICS_LOCK` を持つと、互いを排除しない
//! (実測: 2本目を足した直後、`playback_ticks_do_not_trigger_cpu_readback_
//! after_warmup` の `render_frame_calls()` が他テストの並列実行分だけ余計に
//! 増えて red になった)。この crate ルート(バイナリの root module)へ
//! 1本だけ置き、両モジュールが `crate::METRICS_LOCK` を借りる。
pub(crate) static METRICS_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[path = "suite/render_pipeline_fence.rs"]
mod render_pipeline_fence;
#[path = "suite/zero_copy_presenter_fence.rs"]
mod zero_copy_presenter_fence;
