# Rerunビュワーの組み上がり — 借りられる核はどこまでか

- 日付: 2026-08-30
- 契機: `re_renderer::video` への置き換えで自前1422行が消えた。
  **「まだ自作している物のうち、上流に在る物」を見つけるための地図**
- 位置づけ: 測定。裁定ではない

## 借りられる核は5クレート(egui依存ゼロ、grep実測)

| クレート | egui行 | Motoliiの現状 |
|---|---|---|
| `re_renderer` | 0 | 借りている |
| `re_video` | 0 | **2026-08-30に借りた** |
| `re_chunk_store` | 0 | 借りている |
| `re_entity_db` | 0 | 借りている |
| `re_query` | 0 | 借りている |

**5つのうち4つは既に借りていた。**egui が入るのは `re_viewer_context` からで、
そこですら egui を触るのは `gpu_bridge/re_renderer_callback.rs` の92行だけ。

既存の測定([front-end再比較](2026-08-29-frontend-choice-reopened-comparison.md))が
「`re_renderer` 以外に流用できる層が皆無」と結論したのは **`crates/viewer/` 内**の話。
本書が足すのは **`crates/store/` と `crates/utils/`** で、矛盾ではなく補完。

## 1フレームの流れ(ビューがre_rendererをどう駆動するか)

`re_view_spatial/src/ui_2d.rs`:
1. ピッキング判定(`:306-329`)
2. `setup_target_config`(`:332-344`)でカメラ/クリップ/ピッキング設定
3. `ViewBuilder::new(...)`(`:345`)— **1フレーム=1 ViewBuilder**
4. draw data を `view_builder.queue_draw(...)` へ積む(`:377-391`)
5. `gpu_bridge::new_renderer_callback(...)` で egui へ渡す(`:398`)
6. `re_renderer_callback.rs:43` の `prepare()` が `draw()`、`:89` の `paint()` が `composite()`

**egui 非依存の同型が上流に在る**: `re_renderer_examples/framework.rs`(436行、winit+wgpu のみ)。
`begin_frame`(`:233`)→ 各 `Example::draw`(`:239-247`)→ composite pass を自分で開いて
`view_builder.composite(...)` を直接呼ぶ(`:249-288`)→ submit/present(`:290-296`)。
**Motolii の Stage はこれと同じ立場**なので、構造の手本はこちら。

## 次の候補 — ピッキング

**Stage のクリック選択は今も自前**(`probe/src/stage_widget.rs:394-411`、
矩形の当たり判定と `placement.order` の比較)。

上流に在る: `re_renderer::draw_phases::picking_layer::PickingLayerProcessor`
(`picking_layer.rs:148`)。**`re_renderer` 内なので egui 非依存**で、
`re_renderer_examples/picking.rs:102-103` に **egui を使わない実証コードがある**。
本番側の呼び方は `re_view_spatial/src/picking.rs:197-198`。

egui に縛られているのはカーソル座標の取得だけ(`ui.hover_pos()`)で、
GPU readback 本体は縛られていない。

**未検証**: Motolii から実際に `PickingLayerProcessor` を呼べるかは試していない。

## FINDING(依頼に無かった発見)

`re_renderer_callback.rs` の92行は `prepare`(CPU側コマンド生成)/`paint`(render pass書き込み)の
2フェーズで、これは `egui_wgpu::CallbackTrait` の要求そのもの。
**blitz のカスタム描画が単一フェーズしか許さないなら、そこが最初に詰まる場所**になる(未検証)。
