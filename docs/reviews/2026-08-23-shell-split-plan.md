# `motolii-shell/src/lib.rs` の分割線(6,228行 → 11モジュール)

軸台帳の `責任` 列がこのファイルを **102回**指名した(2位の6倍)。集まっている穴は互いに無関係で、
**穴の種類がそのまま分割線**になる。発明はしていない — この crate には既に
`auto_save` / `clipboard` / `file_dialogs` / `menu` / `pane_layout` / `screenshot` / `transport` / `metrics` / `fixture`
というモジュール群が在り、`lib.rs` だけがそのパターンに従わずメソッドを手放さなかった。

**この分割の目的は整頓ではない。** 波C の並列レーンが**互いに素な write-set を持てるようにする**こと
(利用者裁定 2026-08-23:「事前に並列にできるよう分解して切っておけばコードを最低限にできる。
ひとつのファイルの責任は多すぎるとダメ」)。

---

## 移送表(この表以外の判断をしない)

行番号は分割前の `lib.rs` のもの。**中身は変えない。移すだけ。**

| 新モジュール | 移すもの(行) | 概算 | ここに住む穴 |
|---|---|---|---|
| **`stage_presenter.rs`** | `stage_presenter_rgba`(218-295)・`build_stage_presenter_rgba`(5004-5080)・WGSL 文字列・`Uniforms`/`VertexOutput`/`vs_main`/`fs_main`・`stage_presenter_letterbox_ndc`(+その test mod)・`StagePresenterProgram`/`Primitive`/`Texture`/`GpuTarget`/`Pipeline`・`ActivePresenter`(5237-5747) | ~820 | 無し(純粋な移送。**編集の意味と一切関係ない GPU 層**) |
| **`input.rs`** | `inspector_pointer_event`(5748-5858)・`resolve_navigation_key`(5859-6065) | ~320 | **Backspace/Delete 二重発火**。`inspector_pointer_event` が非 pub ゆえ `keymap_equivalence.rs` の対象外 → **pub 化して柵の射程へ入れる**(移送の副産物として最重要) |
| **`view.rs`** | `view`(4299-4465)・`pane_title_bar`・`header`・`header_icon_action`・`view_window`・`window_title`・`view_settings_window`・`view_export_window`・`stage_pane`(6066-6187)・`band_chrome_style`・`status_band`(6188-6228)・`stage_overlay`/`stage_gizmo_overlay`/`stage_sheet_overlay`/`stage_marquee_overlay`(4019-4167) | ~900 | **未保存●が無い**(`window_title` が固定文字列・`is_project_dirty` を呼んでいない) |
| **`document_io.rs`** | `default_document`(1103-1118)・`is_dirty`・`confirm_discard_future`・`confirm_then`・`confirm_then_pick_open`・`reset_document`・`perform_save_as`・`perform_save_a_copy`・`perform_open`・`run_auto_save`(1819-1970) | ~180 | **Cmd+S 欠落 / 閉じるボタンが確認を飛ばす / 再起動で前回が開かない / autosave の読み返しが無い / M17 安全網の握り潰し**。波C「保存と復帰」の家 |
| **`selection.rs`** | `select_single`・`apply_stage_selection`(1542-1599)・`copy_layer`/`paste_layer`/`cut_layer`/`duplicate_layer`・`select_all_layers`/`deselect_all_layers`・`group_selected_layers`/`ungroup_selected_layers`/`set_selected_groups_frozen`(1971-2158) | ~250 | **選択の正本3本 / `duplicate_layer` が単数 / `cut_layer` が `selected_layers` を無視 / 複数レイヤー削除が不可能**。波C「選択と一括」の家 |
| **`inspector_ops.rs`** | `update_inspector`(2322-2604)・`commit_inspector_field`/`commit_inspector_name`・`toggle_inspector_key`/`toggle_inspector_hidden`・`cycle_inspector_blend_mode`・`commit_inspector_speed`/`reset_inspector_speed`/`apply_speed`・`toggle_layer_hidden`/`toggle_layer_solo`/`toggle_layer_lock`(2605-2871)・`start_field_drag`/`continue_field_drag`/`finish_field_drag`/`enter_field_editing`/`cancel_inspector_interaction`(3602-3687) | ~600 | **drag 機構の親(A02)/ 時間軸の分岐(A03)/ solo にトラックが無い** |
| **`playback.rs`** | `step_playhead`・`jump_meaning_point`・`jump_clip_edge`・`scrub_to`・`toggle_playback`・`apply_shuttle`・`is_dragging`・`freeze_playhead_from_transport`・`advance_playback_tick`・`debug_start_playback_with_session`・`is_playing`(2872-3085) | ~210 | Next/Prev Keyframe が実はクリップ編集点ジャンプ(台帳 id504/505 の不一致) |
| **`export_ops.rs`** | `toggle_export_window`・`update_export`・`export_default_file_name`・`start_export`(3471-3601)+ `export_*` accessor 群(3801-3834) | ~170 | **同期ブロッキングで cancel が効かない / 音声 mux が呼ばれず無音の mp4 が出る**。波C「書き出し」の家 |
| **`render.rs`** | `refresh_frame`(4638-4831)・`build_preview_snapshot`・`ensure_rgba_fresh`・`checkerboard_preview_source`・`observation_preview_source`・`compute_display_source`(4832-5003)・`media_natural_size`・`frame_rgba`/`checkerboard_preview_rgba`/`observation`/`observation_rgba` | ~450 | B 系の器具を掛ける先(いま `next/` に器具が無い) |
| **`create.rs`** | `create_from_card`・`default_shape_path_source`・`default_new_object_fill`・`add_mask_to_selected_layer`・`default_mask_shape`・`apply_effect_to_selected_layer`(1600-1818)・`next_layer_id`・`label_color_for_new_layer`(4611-4637) | ~250 | 裁定205「追加するの家は Browser」の shell 側。**死んだ `SetSource` の配線先** |
| **`assets.rs`** | `admit`・`fingerprint_source`・`guess_asset_type`(2159-2310) | ~150 | fingerprint が全読み hash(大容量素材で B7 未計測) |

### `lib.rs` に残す物(骨だけ)

`Message` enum(296-620)・`Shell` struct(775-988)・`new`/`new_with_dialogs`/`boot`/`boot_fixture`/
`with_main_window`/`main_window`/`new_fixture`・`title`・`subscription`・**`update`(1247-1541)**・
`update_settings`/`update_settings_legacy`/`toggle_settings_window`・`update_stage`/`update_gizmo`/
`cancel_gizmo_drag`・`update_marker`・読み取り専用 accessor 群・`build_timeline_pane`・`tokens`/`dims`/`colors`。

**概算 1,700〜1,900行。** `update` は Message の振り分け役なので残す(ここを割ると Message の全体像が消える)。

---

## 規律

1. **中身を変えない。** 移送のみ。バグ修正・整形・リネームを混ぜない(混ぜると差分が読めなくなり、波C が壊れたとき原因を切り分けられない)
2. **例外は1つだけ**: `inspector_pointer_event` を `pub` にする(`keymap_equivalence.rs` の射程へ入れるため)。**穴は直さない** — 柵に見えるようにするだけ
3. 可視性は `pub(crate)` を既定にし、外部から呼ばれている物(既存の `pub fn`)は `pub` のまま `pub use` で `lib.rs` から再輸出する。**crate 外の呼び手を壊さない**
4. 検収は `cargo check -p motolii-shell --tests`(利用者裁定: ビルドは最後の最後、それまでは静的検査で十分)。**前景・`timeout 600000`**
5. `screenshot.rs`(1,119行)は今回触らない。次の分割候補として記録だけ残す

---

## この分割が可能にすること

波C の4レーンが、**互いに素な write-set**を持てる:

| 波C レーン | write-set |
|---|---|
| 保存と復帰 | `document_io.rs` + `view.rs`(未保存●) |
| 選択と一括 | `selection.rs` |
| 書き出し | `export_ops.rs` |
| 入口の結線 | `create.rs` + `input.rs` |

分割前は**4本とも `lib.rs` を奪い合う**。これが裁定198(並列の強みは割る圧力・漏斗を作るのは強みの放棄)の実体。
