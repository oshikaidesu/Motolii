# G0-9 Web UI runtime spike

React/Viteを製品へ組み込まず、G0-9の比較項目だけを隔離して測る。

- Browser: 10,000 itemを固定高virtual listへ投影し、DOM row数がvisible rangeに比例すること
- Timeline: 1,000 clip / 100,000 keyを単一Canvas 2Dまたはbrowser WebGPU surfaceへ描画すること
- dynamic drag: PixiJS / Konvaの既成scene graphで1 / 1,000 / 10,000 keyと
  1 / 100 / 1,000 objectをgroup移動し、React stateをper-frame更新しないこと
- identity: 最終itemまでscrollしてもstable IDで選択できること
- hot reload: Viteのvirtual module更新をRust process再起動なしでacceptできること
- actual interaction: Konvaの実pointer eventでgroup drag、10px snap、canvas外drag、Escape cancel、
  marquee selectionを通し、選択をaccessible DOM proxyへ投影すること
- IME boundary: composition中のshortcutを抑止し、composition event列をReact inputで保持すること
- sandbox negative: opaque-origin iframeからparent DOM、storage、network、native bridgeへ直接触れず、
  explicit `postMessage`だけを受けること
- object handles: Konva Transformerで2D move/scale/rotate/multi-select/Cancel/zoom-invariant hit target、
  Three.js TransformControlsで3D translate/rotate/scale、world/local、snap、camera操作排他を比較すること

本スパイクは製品WebView、native wgpu texture共有、plugin sandbox、Document/command接続を実装しない。
Cancel / release callbackはadapter harnessの境界確認であり、実D2 Undo接続ではない。
数値は同一環境内の比較証拠であり、製品性能閾値ではない。

```sh
cd spikes/g0-9-web-ui
npm ci
npm run build
G0_9_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/report.json \
G0_9_INTERACTION_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/interaction-report.json \
G0_9_SANDBOX_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/sandbox-report.json \
G0_9_HANDLE_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/object-handles-report.json npm test
```
