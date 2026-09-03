# 2026-09-03 売り込み(営業ペルソナ)— 弊社の技術で賄える部分

利用者の依頼: Dioxus / Blitz と rerun の営業担当(技術者畑)に、自前で作っている物のうち上流で賄える物を売り込ませる。誇張なし、file:line で裏を取った物だけ。宿題番号は [persona-backlog](2026-09-03-persona-backlog.md)。

## rerun(fork rev 346a0b3、`$R` = git checkout)

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| R1 | timeline_widget.rs の `nice_step` / `snapped_time` / `snap_targets` / `snapped_delta`(時間→x・段・吸い付きを自作、目盛の数字無し) | `re_time_ruler/src/time_ranges_ui.rs` の `x_from_time` / `time_from_x_f64` / `pan` / `zoom_at` / `snap_time_control`、段は `re_format/src/time.rs next_grid_tick_magnitude_nanos`(egui 非依存)、整形は `re_log_types time_cell.rs format_compact`。`TimeRangesUi` は `emath::Rangef` を取る(描画は持たない) | ≒70 行、V8・A15・F18 |
| R2 | stage_widget.rs の当たり判定は軸並行の箱(α 抜き・mesh・点群は誤る) | `re_renderer draw_phases/picking_layer.rs PickingLayerProcessor`(`picked_id` / `picked_world_position`)— 既に使っている ViewBuilder の中 | 正しさ、ST10 の前提 |
| R3 | 選択枠は箱を vello で線引き | `draw_phases/outlines.rs OutlineMaskProcessor`、`view_builder.rs outline_config` に `OutlineConfig` + `OutlineMaskPreference` | 新規 20 行で形に沿う縁、ST5・ST16 の足場 |
| R4 | `Fit`(s/fx/fy/pan_putting/at_z)、`orbit_angles`、`camera_center_for` | 2D は `re_renderer transform.rs RectTransform`、3D は `re_view_spatial eye.rs rotate/translate/handle_zoom/focus_entity/focus_point`(`EyeState::update` は egui 依存なので関数単位で) | Fit ≒90 行、ST20 |
| R5 | Stage にグリッド無し | `re_renderer renderer/world_grid.rs WorldGridConfiguration`(QueueableDrawData) | 新規 15 行、ST15 のグリッド分 |
| R6 | thumbnail.rs が ffmpeg を同期 spawn、probe は ffprobe 子プロセス | `re_video demux VideoDataDescription`(duration・VideoEncodingDetails・mp4_tracks)+ `player frame_at` — `engine/texture.rs:390-436` で既に同じ API を使用 | ≒20 行減・外部プロセス 1 本減、M5・M6・M7・A19 |
| R7 | `media/mesh.rs` が tobj/gltf/stl_io を直接叩き AABB 手計算、`compositor/mesh.rs` は同じ 3 形式を `re_renderer::importer` で読む(二重) | `re_renderer::importer::{gltf,obj,stl}` → `mesh.rs bbox: macaw::BoundingBox` | ≒155 行と依存 3 本 |
| R8 | 取っ手の大きさが場所ごとに `fit.s` 割り / `sfac()` 掛け | `re_renderer size.rs Size`(`new_ui_points` / `new_scene_units`)— point_cloud.rs で既使用 | 再発を型で止める(ST11・F19) |
| R9 | C4 Blend のサムネイルが空 | 既存の `compositor/headless.rs` + `render_basic.rs` の ScreenshotProcessor を blend ごとに回す | 新規 30 行、C4・C13 |

弊社に無い物(正直に): 音(A1・A2・A3・A7・A8・A10・A17)、市松・セーフエリア・定規・ガイド(ST15 の残り、fork 追加案件)、文字組み(S4・S8・S9・S10)、キーと ease の編集(F5・F8・F11・F14・F15)、Lottie / ffmpeg 書き出し、HIG(H*)。

## Dioxus / Blitz

(報告待ち)
