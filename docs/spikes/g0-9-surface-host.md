# G0-9 wgpu 29 surface host実機spike（2026-07-21）

状態: **macOS部分合格／製品統合は継続停止**。

公開API、Document、plugin契約、永続layoutへ触れないisolated harnessとして
[`spikes/g0-9-surface-host/`](../../spikes/g0-9-surface-host/)を作り、決定済みtopologyを製品と同じ
wgpu majorで確認した。これはrenderer採用やG0-9完了を意味しない。

## 構成と審判

- top-level `wgpu::Surface` 1枚、frameごとのacquire/presentは1系統
- 同一surface textureをStageとTimelineの2 viewportへ分割
- 左右にopaque child WKWebViewを2枚配置
- window titleとJSON reportでresize、layout epoch、acquire、present、readback、drag、Web入力を計数
- `SurfaceLayout`のDPI変換、無効寸法、境界hit-test、present不変条件をRust unit testで固定

実行コマンド:

```bash
cargo fmt --manifest-path spikes/g0-9-surface-host/Cargo.toml -- --check
cargo test --manifest-path spikes/g0-9-surface-host/Cargo.toml
cargo build --manifest-path spikes/g0-9-surface-host/Cargo.toml
```

unit testは4件合格した。macOS実機ではComputer UseでApp bundleを操作し、最終reportは次だった。

| 観測 | 結果 |
|---|---:|
| wgpu major | 29 |
| surface / native viewport / WebView | 1 / 2 / 2 |
| resize event / layout epoch | 104 / 106 |
| acquire / present / CPU readback | 200 / 200 / 0 |
| nativeからWebViewへ境界drag | move 2、境界通過true、release true |
| Web入力 | 4 events、左右の値をAX経由で再取得 |
| WebViewからnativeへdrag | start 1、move 0、end 1 |

Stage、Timeline、両WebViewが同時表示され、100回resize後も位置が一致した。Web入力後と
minimize/restore後とfullscreen進入後にもnative描画とWebViewのAX treeを再取得できた。したがってmacOSの通常windowで
「1 surface / 2 viewport / 非重複opaque WebView islands」が成立し、CPU pixel bridgeを必要としないことは
このfixtureで合格とする。

## 未証明と停止線

- Computer Useの一括dragはWebKitへ`pointerdown`と`pointerup`を届けたが中間`pointermove`を生成しなかった。
  WebViewからnative Stageへのdrag token handoffは人間のactual pointerか分割可能な入力fixtureで再審判する
- AX treeはBrowser/Inspectorのheading、input、buttonを露出したが、GPU描画のStage/Timelineには意味ノードがない。
  bounded AccessKit treeまたは同等proxyをhost側で持つまでVoiceOver合格にしない
- 日本語文字列の値設定はIME composition、候補窓、preedit、取消の証明ではないため、人間の日本語IME試験を残す
- fullscreen進入時の描画は成立したが、focused WebViewからの自動shortcutで退出を再現できなかった。
  WebView間focus traversalとfullscreen往復はactual keyboardで再審判する
- 異DPI monitor移動、surface/device lost、Web content process終了、sandbox、実penは未試験
- Windows WebView2、per-monitor DPI、MS-IME、NVDA、offline runtimeはWindows実機だけで判定する

未証明項目を埋めるために透明WebView、複数surface、CPU readback、raw plugin権限へfallbackしない。
