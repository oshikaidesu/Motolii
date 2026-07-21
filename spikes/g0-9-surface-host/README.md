# G0-9 wgpu 29 surface host spike

製品workspaceから隔離したplatform受入ハーネス。公開API、Document、plugin契約、永続layoutを変更しない。

確認する構成:

- top-level `wgpu::Surface` 1枚、acquire/present 1系統
- 同一surface texture内のStage / Timeline 2 viewport
- 左右2つのopaque child WebView
- CPU pixel readback 0
- resize 100 eventと同じ回数以上のlayout epoch
- Web text focus、Web pointer capture、native→WebView境界drag
- minimize / restore後のsurfaceとAX tree

## 自動実行

```bash
cargo test --manifest-path spikes/g0-9-surface-host/Cargo.toml
G0_9_RESIZE_TARGET=100 cargo run --manifest-path spikes/g0-9-surface-host/Cargo.toml
```

実行中の状態はwindow titleと、既定で
`/tmp/motolii-g0-9-surface-host-report.json`へ出す。別pathは`G0_9_REPORT`で指定する。

## 合格と限界

- `surface_count == 1`、`native_viewport_count == 2`、`webview_count == 2`
- `acquire_count == present_count`、`readback_count == 0`
- `resize_events >= 100`、`layout_epoch >= 100`
- Computer Useで左右inputがAX treeに現れ、入力後もnative 2 viewportが表示される
- native領域からWebViewへdragし、releaseまでhostが受け取れば`native-drag=PASS`
- Browserのdrag targetからnative領域へdragし、JS pointer captureが継続すればweb start/move/endが増える

このMac試験はWindows WebView2、per-monitor DPI、IME候補窓、VoiceOver/NVDA、実pen、process/device lostを
合格にしない。透明WebView、複数surface、CompositionController、CEFへfallbackしない。
