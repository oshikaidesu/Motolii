//! 統合ハーネス(裁定138): `tests/` 直下の integration test バイナリ10本を
//! 2本へ束ねる。各ファイルの中身・テスト関数名は無改変 — `tests/suite/` へ
//! `git mv` した上で `#[path]` で取り込むだけ(統合以外のリファクタはしない)。
//!
//! 10本の逐次フルリンクが test leg の9割を占めていた(`next/DECISIONS.md` #138)。
//! cargo は integration test 1バイナリにつき1回リンクするため、バイナリ数を
//! 減らすことでリンク回数そのものを減らす。
//!
//! `render_pipeline_fence.rs` だけは [`tests/metrics_main.rs`] へ分けて別プロセス
//! のまま残す — `motolii_shell::metrics` はプロセス全体で共有される static で、
//! この1本に統合すると他ファイルの `Shell::update()`(どれも `refresh_frame` 経由で
//! 同じ static を叩く)が並列スレッドから割り込み、`render_pipeline_fence.rs` の
//! 前後差分計測(`metrics::reset()` → 操作 → `metrics::handle_creations()` 比較)を
//! 汚染して間欠的に落ちることを実測した(このファイル単独なら常に緑)。
//! バイナリを分ければプロセスが分かれ、static も分かれるので汚染源が消える。

#[path = "suite/drive.rs"]
mod drive;
#[path = "suite/fixture.rs"]
mod fixture;
#[path = "suite/iced_test_spike.rs"]
mod iced_test_spike;
#[path = "suite/ident_band_drive.rs"]
mod ident_band_drive;
#[path = "suite/inspector_drive.rs"]
mod inspector_drive;
#[path = "suite/inspector_pixel_fence.rs"]
mod inspector_pixel_fence;
#[path = "suite/q0_fence.rs"]
mod q0_fence;
#[path = "suite/settings_drive.rs"]
mod settings_drive;
#[path = "suite/ui_scale_fence.rs"]
mod ui_scale_fence;
