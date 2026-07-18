# PV-1 texture lifecycle spike 結果

作成日: 2026-07-18

## 結論（**未確定 — 人間審判 pending**）

自動振る舞い試験は `spikes/pv1-texture-lifecycle/tests/lifecycle_behavior.rs` で構造を検証する。
**overall 合格は人間実機審判（H1–H6）完了まで記録しない。**

| 層 | 内容 | 状態 | 証拠 |
|---|---|---|---|
| 自動 | 状態機械・counter・generation bind・定常 tick 非再生成 | **実装済** | `cargo test` 出力（下記） |
| 人間 | 10分継続・100 resize・hide/show 自動往復・minimize 自動復帰・DPI | **pending** | [manifest-skeleton.json](pv1-texture-lifecycle-evidence/manifest-skeleton.json) |
| Backend | Metal / DX12 / Vulkan | **Metal pending / 他 pending** | 開発主機実測後に更新 |

## 観測対象

- Manual 共有 device（`GpuCtx::new_for_ui` → `WGPUConfiguration::Manual`）
- worker 保持 texture + **GPU render-pass Clear** による定常内容更新（VRAM 常駐）
- 寸法変更・明示 regenerate 時のみ `create_texture` + `Image::try_from` 再 bind（generation 付き）
- hide/show / minimize 相当で texture を再 create しない（counter で証明）
- Worker→UI: Texture slot と Status slot を `LatestSlot<T>` で分離 replace
- OS 物理 `window.size()` 変化 → 同一 Resize 経路（同寸法 no-op、0 寸法は recoverable 非 mutation）

## H3 / H4 手順（単一 window 自動往復）

1. **Hide ボタン**: `window.hide()` → single-shot Timer（~400ms）で自動 `show()` + lifecycle `Show`
2. **Minimize ボタン**: `set_minimized(true)` → single-shot で自動 `set_minimized(false)` + lifecycle `Restore`

hide 後に同一 window の Show を押せない問題を、event loop 上の自動往復で回避する。

## 禁止構造（本 spike に含めない）

- `set_rendering_notifier`
- UI thread での Motolii frame render
- 共有 device での `poll(Wait)` / `download_rgba`
- 定常 frame ごとの `create_texture` / pipeline / shader 生成
- 定常 `queue.write_texture` / full-frame CPU `Vec` 画素バッファ
- Texture 配送に `sync_channel` / `try_send` を最新値 mailbox として使うこと
- 第 2 device / native child surface

## 自動検証コマンド

```bash
export CARGO_TARGET_DIR=/private/tmp/motolii-pv1-target
cd spikes/pv1-texture-lifecycle
cargo test
cargo build --release
```

### 自動層 実測 (2026-07-18)

```text
running 7 tests
test slot_tests::ui_tick_command_channel_fatal_is_sticky_over_status_line ... ok
test slot_tests::ui_tick_fatal_overrides_stale_status_line ... ok
test slot_tests::ui_tick_status_line_used_when_no_poison ... ok
test slot_tests::try_take_clears_poison_so_next_tick_is_observable ... ok
test slot_tests::replace_slot_returns_slot_poisoned_after_recovery_write ... ok
test slot_tests::try_take_reports_poisoned_not_would_block ... ok
test slot_tests::converge_mailbox_slot_poison_after_replace_error ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 15 tests
test manifest_overall_never_auto_passes_from_skeleton ... ok
test skeleton_manifest_all_pending ... ok
test failed_state_rejects_further_events ... ok
test display_bind_failed_marks_failed_for_current_generation ... ok
test display_bind_failed_stale_generation_is_no_op ... ok
test record_display_bound_advances_state_for_current_generation ... ok
test hide_show_and_minimize_restore_do_not_recreate ... ok
test invalid_resize_does_not_mutate_counters_or_generation ... ok
test regenerate_increments_create_and_generation ... ok
test new_for_ui_and_image_try_from ... ok
test pipeline_and_shader_counters_stay_zero ... ok
test content_tick_does_not_recreate_texture ... ok
test resize_failure_retains_texture_and_stays_recoverable ... ok
test stale_record_display_bound_does_not_advance_counters_or_state ... ok
test resize_increments_texture_create_once ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo build --release` も成功。**overall / 人間 H1–H6 / backend は pending のまま。**

## 人間審判コマンド

```bash
export CARGO_TARGET_DIR=/private/tmp/motolii-pv1-target
cd spikes/pv1-texture-lifecycle
cargo run --release
# 任意: PV1_EVIDENCE_DIR=../../docs/spikes/pv1-texture-lifecycle-evidence cargo run --release
# 証跡は Status 毎ではなく Shutdown→worker join 後に 1 回だけ書込
```

GUI ボタンで Hide/Minimize（自動往復）/ Regenerate/Resize を操作し、ステータス行の counter を確認する。OS ウィンドウ枠のドラッグ resize も `window.size()` 観測経由で worker へ届く。

## レビュー用 rg

```bash
rg -n "set_rendering_notifier" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "download_rgba|PollType::Wait|poll_wait" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "new_headless|Instance::new" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "allow\\(|#!\\[allow|expect\\(" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "solid_rgba_frame|vec!\\[0u8;" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "sync_channel" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
```

追加検収（握り潰し回帰）:

```bash
rg -n "try_lock\\(\\)\\.ok\\(\\)" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests
rg -n "record_ui_display_bound" spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests spikes/pv1-texture-lifecycle/README.md
rg -n "LifecycleState::Resizing|Resizing," spikes/pv1-texture-lifecycle/src spikes/pv1-texture-lifecycle/tests
```

## 参照

- S1 合格証跡: [s1-slint.md](s1-slint.md)
- M3 UI 規約（参照のみ）: [M3-ui-integration.md](../specs/M3-ui-integration.md) §デバイスとスレッド
