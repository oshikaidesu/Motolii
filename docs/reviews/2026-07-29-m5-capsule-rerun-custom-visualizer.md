# Rerun custom visualizer証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Rerun commit `954bf95a4e1a01de4cb67e0e92b8a5e059ee2b8e`
- files: `examples/rust/custom_visualizer/src/main.rs`,
  `height_field_visualizer.rs`, `height_field_renderer.rs`
- license: `MIT OR Apache-2.0`
- 削除条件: P2D-RCIで元sourceへの直接引用へ置換後

## 観察

- `App::extend_view_class`が既存`Spatial3DView`へvisualizerとfallback providerを登録する。
- visualizerはquery／transform／highlightからrenderer固有draw dataを生成する。
- draw dataは複数phaseへの参加を宣言し、例ではOpaque、Picking、Outlineへ分かれる。
- rendererはwgpu resource／pipelineを所有するが、View、query、selection、phase語彙はRerun固有である。

## 非証明範囲

Motoliiの公開plugin契約、万能renderer trait、Document形、phase enum、第三者UI、性能適合を証明しない。
採用／転移分類は本capsuleに含めない。
