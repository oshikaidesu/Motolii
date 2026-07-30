# native Timeline product asset transfer implementation decision

状態: **決定 / 実装済み**

## 決定

旧 `g0-9-timeline-visual-parity` の比較面を製品へ直接importせず、そこで確定した責任境界と
scene構成を通常製品windowへ移管した。

- React所有面は既存product export `KeyToolsCandidate`を左端202 logical pxのopaque child
  WebViewへ載せる。Reactが受け取る製品値は同じ`TimelineProjection`のlayer/key件数だけで、
  mode / scope / section開閉はlocal presentation stateとする。
- native所有面はheader、time ruler、S/M rail、row、bar、key、playhead、labelとする。
  起動時とD2 publish後の同じ`Document`、`TimelineProjection`、primaryを長寿命の
  `NativeTimelineRenderer`へ渡し、同じwgpu device/queue上のVello local passをtop-level
  Surfaceへ合成する。
- bar/keyのhit-test領域もReact 202pxとnative rail 54pxを除いたtime surfaceへ合わせる。
- 旧CU-110PTの単色矩形pipelineは生成・描画しない。
- Key Toolsに未接続の編集operationはDocument commandを推測せず、private IPC診断だけを出す。

## 不変条件

- Document / selection / Undo / journal / plugin契約 / 永続形式 / 公開APIを変更しない。
- ReactにDocument正本、primary、historyを置かない。
- spike crate、archived mock、fixture stateを製品runtimeからimportしない。
- visible range、semantic zoom、playhead永続owner、S/M意味をこの移管で発明しない。
- VelloはGPU rendererとして既存wgpu device/queueを共有し、CPU renderer/readbackを使わない。

## 実機証跡

固定Mac Retina 2xの通常製品windowで次を確認した。

- Timeline Tools bounds: `x=0, y=600, width=202, height=200` logical
- native Timeline bounds: `x=0, y=1200, width=2400, height=400` physical
- 起動時: `rows=1, bars=1, keys=0, text_runs=13`
- Rectangle追加後: `rows=2, bars=2`へ再起動なしで更新し、React layer countも2へpublish
- 追加配置を続けた同一sessionでnative row/barとStage Rectangleが同数へ更新
- 実画面でReact `KEYS / LAYERS`、native header/ruler/S/M rail/bar labels/playheadを同時表示

## 審判

```bash
npm --prefix ui/motolii-web run build:host
npm --prefix ui/motolii-web run check:host
cargo test -p motolii-ui
```
