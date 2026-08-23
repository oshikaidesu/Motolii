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

#[path = "suite/anchor_drag_drive.rs"]
mod anchor_drag_drive;
#[path = "suite/auto_save_drive.rs"]
mod auto_save_drive;
#[path = "suite/band_line_fence.rs"]
mod band_line_fence;
#[path = "suite/blend_drive.rs"]
mod blend_drive;
#[path = "suite/browser_drive.rs"]
mod browser_drive;
#[path = "suite/clipboard_drive.rs"]
mod clipboard_drive;
#[path = "suite/create_from_card_drive.rs"]
mod create_from_card_drive;
#[path = "suite/drive.rs"]
mod drive;
#[path = "suite/entrance_atlas_dump.rs"]
mod entrance_atlas_dump;
#[path = "suite/export_drive.rs"]
mod export_drive;
#[path = "suite/file_drive.rs"]
mod file_drive;
#[path = "suite/fixture.rs"]
mod fixture;
#[path = "suite/gizmo_drive.rs"]
mod gizmo_drive;
#[path = "suite/group_drive.rs"]
mod group_drive;
#[path = "suite/iced_test_spike.rs"]
mod iced_test_spike;
#[path = "suite/ident_band_drive.rs"]
mod ident_band_drive;
#[path = "suite/inspector_drive.rs"]
mod inspector_drive;
#[path = "suite/inspector_key_drive.rs"]
mod inspector_key_drive;
#[path = "suite/inspector_pixel_fence.rs"]
mod inspector_pixel_fence;
#[path = "suite/marker_keymap_drive.rs"]
mod marker_keymap_drive;
#[path = "suite/marquee_drive.rs"]
mod marquee_drive;
#[path = "suite/mask_effect_from_card_drive.rs"]
mod mask_effect_from_card_drive;
#[path = "suite/remove_asset_from_card_drive.rs"]
mod remove_asset_from_card_drive;
#[path = "suite/keymap_equivalence.rs"]
mod keymap_equivalence;
#[path = "suite/lyric_text_layer_drive.rs"]
mod lyric_text_layer_drive;
#[path = "suite/menu_drive.rs"]
mod menu_drive;
#[path = "suite/nav_drive.rs"]
mod nav_drive;
#[path = "suite/observation_camera_drive.rs"]
mod observation_camera_drive;
#[path = "suite/pane_band_drive.rs"]
mod pane_band_drive;
#[path = "suite/playback_drive.rs"]
mod playback_drive;
#[path = "suite/q0_fence.rs"]
mod q0_fence;
#[path = "suite/rename_drive.rs"]
mod rename_drive;
#[path = "suite/restack_drive.rs"]
mod restack_drive;
#[path = "suite/selection_bulk_drive.rs"]
mod selection_bulk_drive;
#[path = "suite/settings_drive.rs"]
mod settings_drive;
#[path = "suite/sheet_drive.rs"]
mod sheet_drive;
#[path = "suite/shortcut_drive.rs"]
mod shortcut_drive;
#[path = "suite/shuttle_work_area_drive.rs"]
mod shuttle_work_area_drive;
#[path = "suite/speed_drive.rs"]
mod speed_drive;
#[path = "suite/stage_band_drive.rs"]
mod stage_band_drive;
#[path = "suite/target_walk.rs"]
mod target_walk;
#[path = "suite/theme_wiring_fence.rs"]
mod theme_wiring_fence;
#[path = "suite/timeline_key_gesture_drive.rs"]
mod timeline_key_gesture_drive;
#[path = "suite/timeline_key_rows_drive.rs"]
mod timeline_key_rows_drive;
#[path = "suite/timeline_preview_drive.rs"]
mod timeline_preview_drive;
#[path = "suite/timeline_tree_rows_drive.rs"]
mod timeline_tree_rows_drive;
#[path = "suite/tonmana_token_fence.rs"]
mod tonmana_token_fence;
#[path = "suite/ui_scale_fence.rs"]
mod ui_scale_fence;
#[path = "suite/window_drive.rs"]
mod window_drive;
