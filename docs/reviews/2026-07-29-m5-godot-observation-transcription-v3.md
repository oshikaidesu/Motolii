# M5 Godot transparency観察転記 v3

作成日: 2026-07-29

状態: **完了／P2D-RCC3-GODOT Grok ACCEPT（P0/P1/P2=0）**

変更許可: 本fileの`転記欄`だけ

単一動詞: **転記する**

## 入力

- [Render Contribution証拠Wave親task](2026-07-29-m5-render-contribution-evidence-wave.md) §7
- [Godot 4.6 transparency capsule](2026-07-29-m5-capsule-godot-transparency.md)

## 固定転記欄

| 観測項目 | capsuleの固定観察 | source anchor | 非証明 | Motoliiへ持込禁止 |
|---|---|---|---|---|
| phase admission / ordering | transparentはopaque後に描画され、object位置基準のback-to-front sortには重なり誤順序が残る。 | Godot 4.6／取得日2026-07-29／<https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>／<https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html> | Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。 | FROZEN / DELETE-LATER / 製品import禁止 |
| depth / opaque / cutout / soft alpha | Godot 4.6はOITを提供せず、alpha scissor、depth pre-pass、alpha hash等を用途別回避策とする。 | Godot 4.6／取得日2026-07-29／<https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>／<https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html> | Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。 | FROZEN / DELETE-LATER / 製品import禁止 |
| transparent交差 / sorting / OIT追加位置 | transparentはopaque後に描画され、object位置基準のback-to-front sortには重なり誤順序が残る。 Godot 4.6はOITを提供せず、alpha scissor、depth pre-pass、alpha hash等を用途別回避策とする。 `ALPHA`を書けばtransparent pipelineへ入り、sorting問題が生じ得る。 | Godot 4.6／取得日2026-07-29／<https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>／<https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html> | Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。 | FROZEN / DELETE-LATER / 製品import禁止 |
| scene-color / refraction / resource lifetime | transparent materialはscreen/depth textureへ現れず、screen-space reflection／refractionへ制限が出る。 | Godot 4.6／取得日2026-07-29／<https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>／<https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html> | Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。 | FROZEN / DELETE-LATER / 製品import禁止 |
| capability不足 / unsupported / cyclic read | 該当する固定観察なし。 | Godot 4.6／取得日2026-07-29／<https://docs.godotengine.org/en/4.6/tutorials/3d/3d_rendering_limitations.html>／<https://docs.godotengine.org/en/4.6/tutorials/shaders/shader_reference/spatial_shader.html> | Godot material mode、render priority、threshold、renderer別featureをMotolii要件にしない。 | FROZEN / DELETE-LATER / 製品import禁止 |

## 非目標

- capsule外の事実、Motolii fixture対応、engine横断比較、推奨、方式採択、公開契約を足さない。
- render graph、material、scene、camera、queue／phase enum、threshold、copy方式をMotolii語彙へしない。

## STOP

- capsuleに無い観測を埋める、別version／network／engine sourceが必要になる。
- 本fileの`転記欄`以外の変更が必要になる。
