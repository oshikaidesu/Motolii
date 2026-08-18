# iced M-2 Stage 島 — 証拠

2026-08-18。レーン: `claude/m2-stage-island-20260818`(M-2 Stage 島の製品 adapter 化)。
生成元はすべて `crates/motolii-shell-iced/tests/` の常設テスト(macOS / Metal、
`iced_test::Simulator` の headless wgpu renderer、scale_factor = 2.0)。

| file | 何の証拠か | 生成元 |
|---|---|---|
| `red.txt` | 実装前に受入テスト18件が落ちていた記録(red 先行) | `cargo test -p motolii-shell-iced --no-fail-fast` |
| `green.txt` | 同じ台本が全緑(48件)になった記録。bind group 床の実測値入り | 同上(実装後) |
| `head-on-quadrants-wgpu.png` | E0 と同じ4象限の既知絵が、**iced の shader widget を通って**正対(document camera)で画枠ちょうどに写る。四隅 = 赤/緑/青/黄、支配チャンネル oracle 全象限 >70% | `stage_island_pixels.rs::the_known_frame_maps_head_on_through_the_iced_widget` |
| `stage-after-drag-wgpu.png` | 左ドラッグ一式(15手)をブリッジ経由で流した**後**の絵。検証層エラー 0。視点が動いていないのは document camera(sticky)の既知の相互作用(Rerun fork 台帳 §4)の実写 | `stage_island_pixels.rs::a_drag_through_the_widget_reaches_the_stage_without_validation_errors` |

## bind group 床(fork seam 2)の実測

`stage_bind_groups_oracle.rs::the_bind_group_floor_reaches_iceds_real_device`
(iced fork seam 台帳 §4 が「M-2 の受け入れ条件に含める」とした実効 oracle):

```
observed max_bind_groups on iced's headless device: before floor = Some(2), after floor = 4
```

- 床を上げる前: iced の headless renderer が建てた device は上流既定の **2** を取得
- `install_rerun_device_floor()`(= `iced_wgpu::device_limits::request_min_max_bind_groups(4)`)後:
  同じ経路で **4** を取得
- 併設の `the_floor_constant_covers_what_re_renderer_actually_asks_for` が、
  定数 4 が `re_renderer` の実要求(`DeviceCaps::from_adapter(..).device_descriptor()`)を
  下回っていないことを adapter 実物で照合

## PNG の再生成

```sh
cargo test -p motolii-shell-iced -j 5 --test stage_island_pixels
```

(`Snapshot::matches_image` は無ければ書く仕様。テストは毎回消してから書き直すので、
この PNG は常に最新の走行の絵である。)
