# 天井の棚卸し — 物の数で CPU が伸びる所

2026-09-15。利用者「x10 レイヤーで 4000 シェイプ。この天井を作るべきでない」。コードを読んだだけ(測っていない)。n = 層、c = 写し、S = 移り方の時刻の数(最大 120、半端な遅れで倍)。

## 一番重い 3 つ

| 天井 | 所 | 伸び方 | 属する |
|---|---|---|---|
| 移り方が過去の配置を解き直す | layout.rs:359–506(`laid_out`/`nudge`/`nudge_z`/`group_size` が時刻ごとに `layout_frame`)、547–586 | 1 コマに最大約 240 回の配置 × (n + 押し合い) | ブロック(移り方) |
| 押し合い | layout.rs:796–886(32 回 × 同じ親の全組) | O(32·k²) を上の回数だけ | ブロック(間合い) |
| 形は写しごとに輪郭を描く | render.rs:1189–1208、texture.rs:321–422、paths.rs:129–131(形の材質が全部透明なので GPU の instancing の道に入らない) | 写しごとに形の書類・cache の鍵の JSON・絵か網 | 描く側 |

## その次

- 写しごとに `ResolvedLayer` を丸ごと複製し、時刻のずれた写しは解決をやり直す(resolve.rs:870–985)
- `snap_to_grids` が 1 層ごとに全層の 3D 変換を作り直す(transform.rs:111–125)→ O(n²)
- Stencil の配り(resolve.rs:799–868)、`layer_box` の Display 無し Group(layout.rs:678–698)が O(n²)
- 解析: 描いた絵を CPU へ読み戻し(analysis.rs:241)、CPU で縮めて塊を数える(media/blob.rs)。1 コマに `resolved_layers` を何度も
- 配置の覚え(`layout_memo`)は view ごとで、コマを跨がない(view.rs:66–74)
- 粒子は CPU で頭から 1 コマずつ(particles.rs:184–192)

## 数の上限

- 重なり順が `i16`(store.rs:487、frame.rs:211、並べた順の振り直し resolve.rs:1235)→ 3 万 2767 を超えると同点
- Repeater の数が 1000 まで(placement.rs:72)
- 選択の輪郭 255、選択の箱の配列 1024(selection_bounds.rs:16)

## 使える GPU の道具

- 計算シェーダーと storage buffer は `compositor/selection_bounds.rs` の 1 つだけ。rerun の fork には無い
- instancing: `GpuMeshInstance`(surface_scene.rs:220–291)。透明の材質があると諦める
- 無い: GPU の箱のバッファ、GPU の塊の数え、GPU の配置

## 読み

コア(層・箱・時間・taffy)は O(n) で天井でない。天井はブロック(移り方・押し合い・写し・スナップ・場・解析)と、形を 1 つずつ描くこと。CSS の型(配置は CPU、transform は GPU で他へ効かない、子は親に乗る)でブロックを配置の後の GPU の段にすると、上の表の上 2 つと解析が消える。

## 追記(2026-09-16)

- vism の fx は欄 1 つにつき uniform buffer を 1 本束ねる(vism.rs:194)。欄 + 2 本が段の uniform の上限を越えると束ねが無効になり、層は黙って描かれない(`layer_failures` にも出ない)。4-Color Gradient の欄 10 個で踏んだ。→ 2026-09-16 device を頼む時に uniform buffer の上限を adapter の範囲(31 本まで)に広げて外した(storage buffer と同じ構図: 選択の籠の分だけ小さく借りていた)。欄を 1 本に詰める直しは、31 本を越える fx が出た時に
