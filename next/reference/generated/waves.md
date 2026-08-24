# 作業割り(機械導出)

`scripts/plan_waves.py` が生成。**手で編集しない。**

原理: **意味componentのwrite-setが交わらない作業項目は同時に走れる**。
`//! responsibility: wire` を持つShell rootは意味レーンから除外し、
最後に1本のWIRE結線で接続する。これで結線の共有と意味の所有を混ぜない。

- 穴 83件 / 意味レーン 39本 / WIRE結線 1本
- WIRE宣言 4ファイル / WIRE関与 23件(結線だけ 0件) / 外部依存 1件 / 責任ファイル未記入 1件

## 意味レーン(重い順)

WIRE結線はここへ含めない。意味componentの実装後にsupervisorがまとめて接続する。

| レーン | 穴 | semantic write-set | WIRE | 過積載 |
|---|---:|---|---|---|
| `shell/motolii-shell/src/metrics.rs` | 8 | `shell/motolii-shell/src/metrics.rs` | ※ |  |
| `ui/motolii-inspector-pane/src/lib.rs` | 6 | `ui/motolii-inspector-pane/src/lib.rs` | ※ | ※ |
| `ui/motolii-stage-pane/src/gizmo.rs` | 5 | `ui/motolii-stage-pane/src/gizmo.rs` |  |  |
| `ui/motolii-inspector-pane/src/transform/mod.rs` | 4 | `ui/motolii-inspector-pane/src/transform/mod.rs` |  | ※ |
| `shell/motolii-shell/src/selection.rs` | 4 | `shell/motolii-shell/src/selection.rs` | ※ |  |
| `ui/motolii-inspector-pane/src/matte.rs` | 3 | `ui/motolii-inspector-pane/src/matte.rs` | ※ |  |
| `core/motolii-store/src/document.rs` | 3 | `core/motolii-store/src/document.rs`, `core/motolii-store/src/view.rs`, `engine/motolii-compositor/src/lib.rs` |  | ※ |
| `ui/motolii-timeline-pane/src/write/mod.rs` | 3 | `ui/motolii-timeline-pane/src/write/mod.rs` |  | ※ |
| `ui/motolii-shell-state/src/lib.rs` | 3 | `ui/motolii-shell-state/src/lib.rs` |  |  |
| `ui/motolii-inspector-pane/src/transform/interaction.rs` | 3 | `ui/motolii-inspector-pane/src/transform/interaction.rs` |  |  |
| `ui/motolii-timeline-pane/src/key_rows.rs` | 2 | `ui/motolii-timeline-pane/src/key_rows.rs` |  |  |
| `ui/motolii-inspector-pane/src/attrs.rs` | 2 | `ui/motolii-inspector-pane/src/attrs.rs` |  |  |
| `ui/motolii-stage-pane/src/zoom.rs` | 2 | `ui/motolii-stage-pane/src/zoom.rs` | ※ |  |
| `ui/motolii-inspector-pane/src/mask.rs` | 2 | `ui/motolii-inspector-pane/src/mask.rs` |  |  |
| `shell/motolii-shell/src/inspector_ops.rs` | 2 | `shell/motolii-shell/src/inspector_ops.rs` |  |  |
| `ui/motolii-browser-pane/src/state.rs` | 2 | `ui/motolii-browser-pane/src/state.rs` | ※ | ※ |
| `shell/motolii-shell/src/document_io.rs` | 2 | `shell/motolii-shell/src/document_io.rs` | ※ |  |
| `ui/motolii-timeline-pane/src/lib.rs` | 2 | `ui/motolii-timeline-pane/src/lib.rs` |  | ※ |
| `shell/motolii-shell/src/create.rs` | 2 | `shell/motolii-shell/src/create.rs` | ※ |  |
| `ui/motolii-timeline-pane/src/clip_gesture.rs` | 2 | `ui/motolii-timeline-pane/src/clip_gesture.rs` |  |  |
| `ui/motolii-timeline-pane/src/rail.rs` | 1 | `ui/motolii-timeline-pane/src/rail.rs` |  |  |
| `shell/motolii-shell/src/menu.rs` | 1 | `shell/motolii-shell/src/menu.rs` |  |  |
| `core/motolii-store/src/lib.rs` | 1 | `core/motolii-store/src/lib.rs` |  |  |
| `ui/motolii-inspector-pane/src/link.rs` | 1 | `ui/motolii-inspector-pane/src/link.rs` |  |  |
| `core/motolii-store/src/persist.rs` | 1 | `core/motolii-store/src/persist.rs` |  |  |
| `engine/motolii-engine/src/render.rs` | 1 | `engine/motolii-engine/src/render.rs`, `engine/motolii-engine/src/texture.rs` |  |  |
| `core/motolii-store/src/asset.rs` | 1 | `core/motolii-store/src/asset.rs` |  |  |
| `engine/motolii-compositor/src/blend.rs` | 1 | `engine/motolii-compositor/src/blend.rs` |  |  |
| `shell/motolii-shell/src/pane_layout.rs` | 1 | `shell/motolii-shell/src/pane_layout.rs` |  |  |
| `ui/motolii-shell-state/src/layout.rs` | 1 | `ui/motolii-shell-state/src/layout.rs` | ※ |  |
| `ui/motolii-tokens-rs/src/tokens.rs` | 1 | `ui/motolii-tokens-rs/src/tokens.rs` |  |  |
| `ui/motolii-timeline-pane/src/write/misc.rs` | 1 | `ui/motolii-timeline-pane/src/write/misc.rs` |  | ※ |
| `ui/motolii-inspector-pane/src/text/value.rs` | 1 | `ui/motolii-inspector-pane/src/text/value.rs` | ※ |  |
| `ui/motolii-timeline-pane/src/input.rs` | 1 | `ui/motolii-timeline-pane/src/input.rs` |  |  |
| `ui/motolii-timeline-pane/src/work_area.rs` | 1 | `ui/motolii-timeline-pane/src/work_area.rs` |  |  |
| `ui/motolii-timeline-pane/src/markers.rs` | 1 | `ui/motolii-timeline-pane/src/markers.rs` |  |  |
| `ui/motolii-timeline-pane/src/canvas.rs` | 1 | `ui/motolii-timeline-pane/src/canvas.rs` |  |  |
| `ui/motolii-stage-pane/src/marquee.rs` | 1 | `ui/motolii-stage-pane/src/marquee.rs` |  |  |
| `probes/r6-text-shaping/src/lib.rs` | 1 | `probes/r6-text-shaping/src/lib.rs` |  |  |

## WIRE結線(直列)

意味レーンが完成した後、Shell rootへ結線する。WIREファイルは意味レーンを連結しない。

| WIREファイル | 責任参照 | 判定 |
|---|---:|---|
| `shell/motolii-shell/src/input.rs` | 1 | WIRE |
| `shell/motolii-shell/src/lib.rs` | 23 | WIRE |
| `shell/motolii-shell/src/render_dispatch.rs` | 0 | WIRE |
| `shell/motolii-shell/src/value_drag.rs` | 0 | WIRE |

## 責任ファイルが書かれていない穴(発注できない)

- A06 最近使ったファイル(MRU、Open Recent)

## 外部依存(このrepoのwrite-setへ入れない)

外部上流の欠如はMotoliiの意味componentへ偽の責任を割り当てない。

- A12 ピンチズーム(トラックパッド、2本指つまむ)
