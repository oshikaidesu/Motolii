# 作業割り(機械導出)

`scripts/plan_waves.py` が生成。**手で編集しない。**

原理: **2つの作業項目が並列可能 ⟺ write-set が交わらない**。
責任ファイルを union-find で束ね、連結成分をそのまま1レーンにしている。
成分どうしはファイルが交わらないので**同時に走れる**。

- 穴 83件 / レーン 31本 / 責任ファイル未記入 2件

## レーン(重い順)

| レーン | 穴 | write-set | 過積載 |
|---|---|---|---|
| `core/motolii-store/src/persist.rs` | 29 | `core/motolii-store/src/persist.rs`, `shell/motolii-shell/src/lib.rs`, `ui/motolii-browser-pane/src/state.rs`, `ui/motolii-shell-state/src/layout.rs` ほか1 | ※ |
| `ui/motolii-inspector-pane/src/lib.rs` | 5 | `ui/motolii-inspector-pane/src/lib.rs` | ※ |
| `ui/motolii-stage-pane/src/gizmo.rs` | 5 | `ui/motolii-stage-pane/src/gizmo.rs` |  |
| `ui/motolii-inspector-pane/src/transform/mod.rs` | 4 | `ui/motolii-inspector-pane/src/transform/mod.rs` | ※ |
| `core/motolii-store/src/document.rs` | 3 | `core/motolii-store/src/document.rs`, `core/motolii-store/src/view.rs`, `engine/motolii-compositor/src/lib.rs` | ※ |
| `ui/motolii-inspector-pane/src/transform/interaction.rs` | 3 | `ui/motolii-inspector-pane/src/transform/interaction.rs` |  |
| `ui/motolii-timeline-pane/src/key_rows.rs` | 2 | `ui/motolii-timeline-pane/src/key_rows.rs` |  |
| `ui/motolii-inspector-pane/src/attrs.rs` | 2 | `ui/motolii-inspector-pane/src/attrs.rs` |  |
| `ui/motolii-inspector-pane/src/matte.rs` | 2 | `ui/motolii-inspector-pane/src/matte.rs` |  |
| `ui/motolii-inspector-pane/src/mask.rs` | 2 | `ui/motolii-inspector-pane/src/mask.rs` |  |
| `ui/motolii-timeline-pane/src/write/mod.rs` | 2 | `ui/motolii-timeline-pane/src/write/mod.rs` | ※ |
| `ui/motolii-timeline-pane/src/lib.rs` | 2 | `ui/motolii-timeline-pane/src/lib.rs` | ※ |
| `ui/motolii-timeline-pane/src/clip_gesture.rs` | 2 | `ui/motolii-timeline-pane/src/clip_gesture.rs` |  |
| `ui/motolii-timeline-pane/src/rail.rs` | 1 | `ui/motolii-timeline-pane/src/rail.rs` |  |
| `shell/motolii-shell/src/menu.rs` | 1 | `shell/motolii-shell/src/menu.rs` |  |
| `ui/motolii-stage-pane/src/zoom.rs` | 1 | `ui/motolii-stage-pane/src/zoom.rs` |  |
| `core/motolii-store/src/lib.rs` | 1 | `core/motolii-store/src/lib.rs` |  |
| `ui/motolii-inspector-pane/src/link.rs` | 1 | `ui/motolii-inspector-pane/src/link.rs` |  |
| `engine/motolii-engine/src/lib.rs` | 1 | `engine/motolii-engine/src/lib.rs` |  |
| `core/motolii-store/src/asset.rs` | 1 | `core/motolii-store/src/asset.rs` |  |
| `engine/motolii-compositor/src/blend.rs` | 1 | `engine/motolii-compositor/src/blend.rs` |  |
| `shell/motolii-shell/src/pane_layout.rs` | 1 | `shell/motolii-shell/src/pane_layout.rs` |  |
| `ui/motolii-tokens-rs/src/tokens.rs` | 1 | `ui/motolii-tokens-rs/src/tokens.rs` |  |
| `ui/motolii-timeline-pane/src/write/clip_drag.rs` | 1 | `ui/motolii-timeline-pane/src/write/clip_drag.rs` | ※ |
| `ui/motolii-timeline-pane/src/write/misc.rs` | 1 | `ui/motolii-timeline-pane/src/write/misc.rs` | ※ |
| `ui/motolii-timeline-pane/src/input.rs` | 1 | `ui/motolii-timeline-pane/src/input.rs` |  |
| `ui/motolii-timeline-pane/src/work_area.rs` | 1 | `ui/motolii-timeline-pane/src/work_area.rs` |  |
| `ui/motolii-timeline-pane/src/markers.rs` | 1 | `ui/motolii-timeline-pane/src/markers.rs` |  |
| `ui/motolii-timeline-pane/src/canvas.rs` | 1 | `ui/motolii-timeline-pane/src/canvas.rs` |  |
| `ui/motolii-stage-pane/src/marquee.rs` | 1 | `ui/motolii-stage-pane/src/marquee.rs` |  |
| `probes/r6-text-shaping/src/lib.rs` | 1 | `probes/r6-text-shaping/src/lib.rs` |  |

## 責任ファイルが書かれていない穴(発注できない)

- A06 最近使ったファイル(MRU、Open Recent)
- A12 ピンチズーム(トラックパッド、2本指つまむ)
