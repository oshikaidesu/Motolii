# G0-9 Web UI runtime spike

React/Viteを製品へ組み込まず、G0-9の比較項目だけを隔離して測る。

- Browser: 10,000 itemを固定高virtual listへ投影し、DOM row数がvisible rangeに比例すること
- Timeline: 1,000 clip / 100,000 keyを単一Canvas 2Dまたはbrowser WebGPU surfaceへ描画すること
- identity: 最終itemまでscrollしてもstable IDで選択できること
- hot reload: Viteのvirtual module更新をRust process再起動なしでacceptできること

本スパイクは製品WebView、native wgpu texture共有、plugin sandbox、Document/command接続を実装しない。
数値は同一環境内の比較証拠であり、製品性能閾値ではない。

```sh
cd spikes/g0-9-web-ui
npm ci
npm run build
G0_9_EVIDENCE=../../docs/spikes/g0-9-web-ui-evidence/report.json npm test
```
