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

## Dioxus / Blitz(blitz rev 64eb278、dioxus-dnd 3.1.0、anyrender 0.13、`B` = git checkout)

### 今日、feature flag / 既存 API で賄える物

| # | Motolii の今 | 弊社で賄える形 | 見込み |
|---|---|---|---|
| B1 | motolii/Cargo.toml の dioxus-native / blitz-shell に `accessibility` が無い — X 波 20 項目が platform に届いていない | `dioxus-native` feature `accessibility`(blitz-shell/window.rs:150 で accesskit adapter、毎 frame update_tree、blitz-dom/accessibility.rs:72-106 に menuitemcheckbox・tablist・status の role 変換) | 2 語。X1〜X20 が VoiceOver に出る |
| B2 | K10 Cmd+C/V/X が無い | blitz-shell feature `clipboard`(text.rs:219-229 が action-mod+c/x/v を配線済み、arboard) | 1 語で K10 |
| B3 | host.rs `realise()` が View::init → context → initial_build を手写し、`Windows` が BlitzApplication を直接包む | dioxus_application.rs:132-176 に同じ手順、`add_window` も在る。`pending_window: Option` を Vec にする上流 PR 1 本 | ≒50 行減、H14・Q12 の土台 |
| B4 | host.rs `place_ime` 38 行 | blitz-dom node.rs:657-694 の focus/blur が既に IME enable + cursor_area。壊れているのは blitz-shell lib.rs:119 の `ImeCapabilities::new()` に `.with_cursor_area()` が無い 1 行 | 上流 1 行で 38 行が消える(K7・S14) |
| B5 | thumbnail.rs が描画中に同期 decode / ffmpeg spawn(M6) | `DocumentConfig.net_provider`: blitz-net が file: を読み、blitz-dom net.rs:517-540 の ImageHandler が別 thread で decode して cache。動画だけ薄い NetProvider | 60→20 行、image/base64 依存が落ちる |
| B6 | custom widget が `_styles: &ComputedStyles` を捨て、tokens.rs が色・級数を手写し(V25) | custom_widget.rs:124-135 は stylo の ComputedValues を渡している(background_color・color・font_size) | ≒30 行、V25・V22・V23 |
| B7 | gui.rs `pointer_raw` / `click_super`、keys.rs の合成 click(Q15・K19) | blitz-test-harness input.rs:29-48 `pointer_event(id,x,y,button,buttons,mods)`、`press_with`、`ime()`、drag | ≒40 行、Q15 |
| B8 | dioxus-dnd を 5 型しか使っていない(FileDropSurface・drop_role_at が自前) | dioxus-dnd files.rs FileDrop/FileRejection、external.rs ExternalDropZone、canvas.rs canvas_keyboard_pointer+SnapGrid、autoscroll、sortable、tree、a11y LiveRegion、desktop MultiWindowProvider | M8・K9/M15・Q16・C1・F20 がほぼ新規 0 |
| B9 | keys.rs `commit_field_outside` / `send_chord` | focus.rs:19-27 が blur/focusout を dispatch、dioxus_document.rs:316 が onblur へ | ≒35 行(全選択の公開 API は無い → BU3) |
| B10 | H22 scale_factor・H18 dark 固定 | window.rs:629-637 が ScaleFactorChanged / ThemeChanged を既に追う、stylo_device.rs:81 が prefers-color-scheme に写す | H22 は 0 行(試験のみ)、H18 は @media |
| B11 | scrollbars / svg feature が off | blitz-paint `scrollbars`(render.rs:689 overlay scrollbar)、`svg` | .iscroll の自前帯不要、C13 のアイコン |

### 上流に無い物 — 弊社 PR で対応

| # | 内容 | Motolii 側の影響 |
|---|---|---|
| BU1 | 焦点のある button が Enter/Space で押せない(keyboard.rs:15-53 は Tab と Cmd+C だけ) | keys.rs `activate_focused_control` 36 行(K2) |
| BU2 | Pinch/Pan/Rotation/DragEntered… が window.rs:848-855 で no-op | host.rs の PinchGesture 翻訳 27 行と drag 捌き(H17・M16・Q16) |
| BU3 | TextInputData に select_all / move_to_end / set_selection の公開 API が無い | keys.rs の偽 KeyEvent(S14) |
| BU4 | custom widget が文字を描けない(blitz-paint text.rs が pub(crate)) | V8・ST10・A17・S12 の名前 |
| BU5 | custom widget の accessibility_tree がコメントアウト(custom_widget.rs:138) | Stage / Timeline / Ease が読み上げに出ない、ST20 の根も同 file の layout TODO |
| BU6 | menu の roving focus が無い(focus_next_node は Tab 順のみ) | semantic_menu.rs 263 行が半分以下に |
| BU7 | 窓の Focused(false) で blur が出ない(window.rs:846) | host.rs:697-702 |
| BU8 | NSMainMenu は winit にも blitz-shell にも無い(担当範囲外、muda / objc2-app-kit) | H1〜H4・H14 |

見込み: feature 4 語 + 既存 API へ ≒230 行 + dioxus-dnd で宿題 6 件 + 上流 PR 待ち ≒100 行。
