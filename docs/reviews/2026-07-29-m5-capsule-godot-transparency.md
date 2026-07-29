# Godot transparency／screen-read証拠capsule

状態: **観察** / `FROZEN / DELETE-LATER` / 製品import禁止

- source: Godot `4.6`公式docs、取得日2026-07-29
- URLs: <https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>,
  <https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html>
- license: Godot docsの利用条件に従う。本文転載なし
- 削除条件: P2D-RCIで公式URLへの直接引用へ置換後

## 観察

- transparentはopaque後に描画され、object位置基準のback-to-front sortには重なり誤順序が残る。
- Godot 4.6はOITを提供せず、alpha scissor、depth pre-pass、alpha hash等を用途別回避策とする。
- `ALPHA`を書けばtransparent pipelineへ入り、sorting問題が生じ得る。
- transparent materialはscreen/depth textureへ現れず、screen-space reflection／refractionへ制限が出る。

## 非証明範囲

Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。
