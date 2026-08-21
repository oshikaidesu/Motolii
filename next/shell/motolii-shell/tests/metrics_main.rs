//! 統合ハーネス(裁定138)の2本目。`render_pipeline_fence.rs` だけをここへ
//! 分離している理由は `tests/suite_main.rs` の doc comment を参照
//! (`motolii_shell::metrics` のプロセス共有 static を他ファイルの並列実行から
//! 汚染されないため、別プロセス=別バイナリのまま残す)。

#[path = "suite/render_pipeline_fence.rs"]
mod render_pipeline_fence;
#[path = "suite/zero_copy_presenter_fence.rs"]
mod zero_copy_presenter_fence;
