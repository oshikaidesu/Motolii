# 上流の契約を外れている所 — 9件の台帳

- 日付: 2026-08-30
- 契機: 「板を傾けると消える」(宿題G)の真因が、`re_renderer` の並べ替え規則を
  Motolii が読まずに繋いでいたことだった。同型が他にもあるはずだと考えて棚卸しした
- 位置づけ: 測定 + 利用者裁定2件。**推測と実測を分けて書く**

## この台帳の型

欠陥の形はどれも同じ:

> **上流が前提にしている事を読まずに繋ぎ、Motolii 側が別の前提でコードを書いた。**

「Rerun に寄せる」より効くのは「**上流が何を前提にしているかを読む**」。
今日の実例では `depth_offset` が典型で、Motolii は「層の重ね順」だと思っていたが、
上流にとっては「同一平面クラスタ内の同点処理」でしかなかった。

## 利用者裁定(2026-08-30)

1. **上流に沿うことが優先。絵が変わるのは許容する** — 「rerun に沿っていない方が大問題」
2. **z=0 の間はレイヤー順、z を動かしたら奥行き優先** — これは**上流の既定の挙動そのもの**
   (下記 8番)。新しい機構は要らず、邪魔している Motolii 側のコードを外すだけ

## 一次資料(次のセッションが再導出しないための実測)

### 矩形の並べ替え規則

- `re_renderer/src/renderer/mod.rs:97` — `distance_sort_key = world_position.distance_squared(camera)`
  が**一次キー**
- `re_renderer/src/renderer/rectangles.rs:582` — `secondary_sort_key = depth_offset as f32`
  は**同点処理でしかない**
- `re_renderer/src/renderer/plane_clustering.rs:38-60` — **同一平面かつ重なっている**矩形は
  1クラスタになり sorting_position を共有する → 距離が同点 → `depth_offset` で並ぶ

つまり全層 z=0 ならレイヤー順、z を持った層はクラスタから抜けて距離順。

### 背景板が z を持つ層を消していた機序(修正済み)

comp 1920×1080・カメラ eye=(960,540,-1037.3)・背景の中心=(960,540,0) → 距離² = 1,076,000。

| 板の位置 | z | 中心の距離² | 結果 |
|---|---|---|---|
| (800,450) 中央 | -50 | 974,761 | 見える |
| (700,400) | -50 | 987,261 | 見える |
| (400,225) | -50 | **1,185,386** | **消える** |
| (0,0) 隅 | -50 | **1,817,261** | **消える** |
| (0,0) 隅 | 0(平ら) | 同一平面 → クラスタ | 見える |

**符号に対称で二値**なのは、横方向のずれが距離を支配するため。傾きでも `position.z` でも同じ。

### premultiplied の二重掛け(修正済み)

`upload_rgba` は premultiplied を載せる契約なのに `ColormappedTexture::from_unorm_rgba`
(= `TextureAlpha::SeparateAlpha`)で渡していた。shader が `rgb * a` をもう一度掛ける
(`re_renderer/shader/rectangle_fs.wgsl:44-45`)。

```
入力 [128,128,128,128] (premultiplied の白・α=0.5)
出力 [ 93, 93, 93,128]
```
検算: sRGB を解いて 0.2140 → ×0.502 = 0.1074 → sRGB へ戻して 93。
**α=0.5 の画素で rgb が 27% 暗い。**アンチエイリアスの縁・glow・matte の出力が対象。

`lib.rs` の `background_rect` **だけ**が正しく `AlreadyPremultiplied` を使っており、
コメントに「`from_unorm_rgba` の既定 `SeparateAlpha` は使えない」と残っていた。
**一箇所で分かった事を他へ持って行かなかった**のが欠陥の形。

### export と Stage で premultiplied sRGB の規約が違う(未着手)

- export(`ScreenshotProcessor` → `composite.wgsl` 末尾): **unmultiply → sRGB符号化 → ×α**
- Stage(`ViewBuilder::new_with_external_resolved`、`view_builder.rs:726-764`):
  呼び手の texture を `main_target_resolved` として import するので、
  **premultiplied のまま sRGB 符号化**される

α<1 で `srgb(P) ≠ srgb(P/a)·a`。検算(誤差1未満で一致):
```
stage=186 → sRGB 0.7294 → linear premultiplied P = 0.4910
P/a (a=226/255) = 0.5540 → srgb → 0.7699 → ×a → 0.6824 → 174 = 実測の export
```

**正しいのは export 側**(8bit の premultiplied sRGB の標準は `srgb(straight) × a`)。
Stage 側は linear の値を sRGB フォーマットの面へ書いた副産物で、規約ではない。

## 9件

| | 件 | 状態 |
|---|---|---|
| 1 | premultiplied な texture を `SeparateAlpha` で渡していた | **修正済み**(`tests/premultiplied.rs`) |
| 2 | 逐次合成の accumulator が今も z=0 の全面板 | 未着手。**背景板と違い仕事が有る**(run 間で合成結果を運ぶ)ので削除では済まない |
| 3 | `render_into` が blend mode を黙って `Normal` に落としていた | **構造は修正済み**。色の残差は9番へ |
| 4 | fork pool の texture を `ViewBuilder` を落としてから持ち出す | 未着手(推測。`finalize_texture` と `point_cloud.rs`) |
| 5 | 全画面三角形 VS を5箇所で自作 | 未着手。上流に `screen_triangle_vertex_shader`(`renderer/mod.rs:222`)。**`&RenderContext` が要るので `device.rs` を家に入れないと着地しない** |
| 6 | `EffectScratch` が上流 `GpuTexturePool` の作り直し | 未着手。実害は無く維持コストのみ |
| 7 | `begin_frame` を1描画で数回進めている | 未着手。単独では低リスク、4番の増幅器 |
| 8 | `depth_offset` を z 跨ぎで重ね順として信頼 | 未着手。**上流が既に裁定どおり**なので、assert と run 分割を外すだけ |
| 9 | export と Stage で premultiplied sRGB の規約が違う | 未着手。**背骨(preview = export)** |

## 9番について

**今日の作業が作った物ではなく、掘り当てた物。**既存の番人
`motolii-engine/tests/zero_copy_matte_text.rs::matte_zero_copy_matches_cpu_export_within_tolerance`
が今まで通っていたのは、CPU 側と GPU 側が**互いに打ち消し合う形で両方間違っていた**から
(1番の二重掛けが誤差を埋めていた)。片方を正しくしたので表に出た。
**赤が増えたのは後退ではなく、嘘だった緑が正直になった状態。**

## 監督の失敗(繰り返さないため)

1. **一次資料より先に配線を追いかけた。**宿題Gの疑い(`DepthOffset` が同一平面前提)は
   `depth_offset.rs` を1本読めば否定できた(正体は `w` へのバイアスで、板を深度の外へ
   落とす機構は無い)。読む前に窓と配線を2往復追った
2. **家を切る前に「その家で終わるか」を確かめなかった**(5番)。上流の口が `&RenderContext`
   を要求するのに `device.rs` を非目標にしたので、レーンは原理的に着地できず止まった
3. **窓を見てもらう時に起動条件を渡さなかった。**`MOTOLII_TESTDATA` 未設定で Browser が
   空になるのを「素材が消えた」と誤診しかけた(正本は `motolii/AGENTS.md`「窓の起動条件」)
