# 作業割り(機械導出)

`scripts/plan_waves.py` が生成。**手で編集しない。**

原理: **2つの作業項目が並列可能 ⟺ write-set が交わらない**。
責任ファイルを union-find で束ね、連結成分をそのまま1レーンにしている。
成分どうしはファイルが交わらないので**同時に走れる**。

- 穴 89件 / レーン 28本 / 責任ファイル未記入 15件

## 責任ファイルが実装に見当たらない(要確認)

- `core/motolii-store/tests/document.rs`
- `engine/motolii-compositor/src/blend.rs`
- `probes/r1-frame-throughput/tests/r1.rs`
- `probes/r2-view-projection/tests/r2.rs`
- `shell/motolii-shell/tests/suite/render_pipeline_fence.rs`
- `ui/motolii-timeline-pane/src/key_rows.rs`
- `ui/motolii-timeline-pane/src/rail.rs`

## レーン(重い順)

| レーン | 穴 | write-set | 過積載 |
|---|---|---|---|
| `core/motolii-store/src/persist.rs` | 25 | `core/motolii-store/src/persist.rs`, `probes/r4-widget-timeline/src/lib.rs`, `shell/motolii-shell/src/lib.rs`, `ui/motolii-browser-pane/src/lib.rs` ほか3 | ※ |
| `ui/motolii-inspector-pane/src/transform.rs` | 7 | `ui/motolii-inspector-pane/src/transform.rs` | ※ |
| `ui/motolii-stage-pane/src/gizmo.rs` | 5 | `ui/motolii-stage-pane/src/gizmo.rs` |  |
| `ui/motolii-timeline-pane/src/write.rs` | 4 | `ui/motolii-timeline-pane/src/write.rs` | ※ |
| `ui/motolii-inspector-pane/src/lib.rs` | 3 | `ui/motolii-inspector-pane/src/lib.rs` | ※ |
| `ui/motolii-inspector-pane/src/link.rs` | 2 | `ui/motolii-inspector-pane/src/link.rs` | ※ |
| `ui/motolii-timeline-pane/src/key_rows.rs` | 2 | `ui/motolii-timeline-pane/src/key_rows.rs` |  |
| `ui/motolii-inspector-pane/src/attrs.rs` | 2 | `ui/motolii-inspector-pane/src/attrs.rs` |  |
| `ui/motolii-inspector-pane/src/matte.rs` | 2 | `ui/motolii-inspector-pane/src/matte.rs` |  |
| `ui/motolii-inspector-pane/src/mask.rs` | 2 | `ui/motolii-inspector-pane/src/mask.rs` |  |
| `ui/motolii-timeline-pane/src/clip_gesture.rs` | 2 | `ui/motolii-timeline-pane/src/clip_gesture.rs` |  |
| `core/motolii-store/tests/document.rs` | 2 | `core/motolii-store/tests/document.rs`, `probes/r1-frame-throughput/tests/r1.rs`, `probes/r2-view-projection/tests/r2.rs` |  |
| `ui/motolii-timeline-pane/src/rail.rs` | 1 | `ui/motolii-timeline-pane/src/rail.rs` |  |
| `shell/motolii-shell/src/menu.rs` | 1 | `shell/motolii-shell/src/menu.rs` |  |
| `ui/motolii-stage-pane/src/zoom.rs` | 1 | `ui/motolii-stage-pane/src/zoom.rs` |  |
| `core/motolii-store/src/lib.rs` | 1 | `core/motolii-store/src/lib.rs` |  |
| `core/motolii-store/src/attrs.rs` | 1 | `core/motolii-store/src/attrs.rs` |  |
| `engine/motolii-compositor/src/lib.rs` | 1 | `engine/motolii-compositor/src/lib.rs` | ※ |
| `engine/motolii-engine/src/lib.rs` | 1 | `engine/motolii-engine/src/lib.rs` |  |
| `core/motolii-store/src/asset.rs` | 1 | `core/motolii-store/src/asset.rs` |  |
| `engine/motolii-compositor/src/blend.rs` | 1 | `engine/motolii-compositor/src/blend.rs` |  |
| `ui/motolii-timeline-pane/src/input.rs` | 1 | `ui/motolii-timeline-pane/src/input.rs` |  |
| `ui/motolii-timeline-pane/src/work_area.rs` | 1 | `ui/motolii-timeline-pane/src/work_area.rs` |  |
| `ui/motolii-timeline-pane/src/markers.rs` | 1 | `ui/motolii-timeline-pane/src/markers.rs` |  |
| `shell/motolii-shell/tests/suite/render_pipeline_fence.rs` | 1 | `shell/motolii-shell/tests/suite/render_pipeline_fence.rs` |  |
| `ui/motolii-timeline-pane/src/canvas.rs` | 1 | `ui/motolii-timeline-pane/src/canvas.rs` |  |
| `ui/motolii-stage-pane/src/marquee.rs` | 1 | `ui/motolii-stage-pane/src/marquee.rs` |  |
| `probes/r6-text-shaping/src/lib.rs` | 1 | `probes/r6-text-shaping/src/lib.rs` |  |

## 責任ファイルが書かれていない穴(発注できない)

- A01 `SetCameraPropertyModulators`(裁定213新設)
- A01 `SetCameraTrack`
- A01 `SetCameraPropertySlot`
- A01 `SetSlots`
- A06 最近使ったファイル(MRU、Open Recent)
- A06 playhead位置(Session.playhead)
- A06 選択レイヤー(Session.selection/selected_layers)
- A06 Timelineキー選択(selected_keys)・Shift範囲基点(key_anchor)・行の折り畳み(tim
- A06 paneレイアウト(実働側 `pane_layout::Layout` ── pane_grid分割比率・Browser
- A06 名前付きワークスペース(WorkspaceBook、"New Workspace…"機能)
- A06 UI scale(ui_scale)
- A11 B2 gesture中フレーム落ち(16.7ms超連続2枚禁止)
- A11 B4 入力→視覚(host往復)≤2フレーム
- A11 B6 起動系スパイク(定常運転50ms超禁止、初回500ms以内)
- A12 ピンチズーム(トラックパッド、2本指つまむ)
