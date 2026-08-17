# Rerun `GridMap` alpha-layer probe

Vism Filterの正しい出力境界に必要な、透明gutterを持つRGBA textureを、Rerun標準の
`GridMap` visualizerで3D `SpatialStage`へ重ねるRerun-onlyの隔離probeである。

```text
RGBA texture
  -> standard Rerun GridMap visualizer / RectangleRenderer
  -> 3D SpatialStage screenshot
```

背景と前景は各々ローカル`z=0`のGridMap面である。2.5Dの前後関係は親`Transform3D`だけが持つ。
正常なら、前景の透明gutterから背面checkerboardが見える。前景が不透明な黒い矩形になる、または
3D Stage以外でしか成立しないなら不合格である。

```bash
cargo run -p rerun-vism-layer-alpha-probe --profile fast -- \
  --screenshot /private/tmp/rerun-vism-layer-alpha.png
```

これは標準3D consumerの透明合成だけを確認するfixtureであり、Vism runtime、Filter、Vism API、
Preview/Export、`motolii-blitz-shell`へ接続しない。VismのGPU outputを渡す公開seamは別に検証する。
