# Rerun上の透明レイヤー表示座席 — 実測

作成日: 2026-08-17

状態: **決定**（表示座席の選択）＋ **観察**（4件の実測）

対象: Vism Filter／LayerSource の出力を、3D `SpatialStage` へ透明レイヤーとして表示する経路。

関連: [M5 3Dとpost](../specs/M5-3d-and-post.md)、[Vism既知実装採択マップ](../vism-known-implementation-adoption-map.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)

## 0. なぜ測ったか

`Mesh3D` がテクスチャのalphaを出さないという報告があり、それが「Rerunでは透明レイヤーが成立しない」を意味するのかを確かめた。結論は**成立する**だが、`Mesh3D` ではない。

この文書の主張はすべてPNGまたはテスト出力に紐づく。**コード読解だけの推論は本文へ入れない**（同じ調査で3回外している。§5参照）。

## 1. `Mesh3D` は画素ごとのalphaを出さない（観察）

pin `8c6865ac` の実装を原文で確認した。

- `crates/viewer/re_renderer/shader/instanced_mesh.wgsl:87` — `var albedo = vec4f(texture * in.color, 1.0) * material.albedo_factor;`。texture alphaを捨てて `1.0` 固定にしている。直上のコメントが理由を述べる（CPU側に透明メッシュの検出が無いため、alphaを使うとtint／albedo_factorを変えた瞬間に不意に有効化される）
- 同 `:61` — 頂点色も `.rgb` のみ。per-vertex alphaも落ちる
- `crates/viewer/re_renderer/src/mesh.rs:387` — `let is_transparent = material.albedo_factor.a() < 1.0;`。透明フェーズ判定が一様係数だけを見る。直上に `TODO(#12223): handle texture transparency`

**可能なのはメッシュ一様の不透明度まで。** Blurの透明gutter、soft edge、chroma抜き、glassのような「背景が画素単位で透ける」出力は出ない。

これは設計契約ではなく上流の未実装である（`#12223`）。

## 2. `Image` archetypeは3D viewで平面として描かれない（観察）

`SpatialView3D` に `Image` を置いても描画されない。次の4通りを試し、いずれも前景が出なかった。

- CPU由来の `Image`、transformを自分のpathへ
- CPU由来の `Image`、transformを親pathへ
- `copy_gpu_image` 経由のGPU texture
- 同上（差し替えあり）

同じ矩形へ置いた `GridMap` は描かれるので、幾何やtransformの問題ではない。`Image` の可視化器は `add_pickable_rect(..., SpaceKind::TwoD)` で登録される（`crates/viewer/re_view_spatial/src/visualizers/images.rs`）のに対し、`GridMap` は `SpaceKind::ThreeD` である（`crates/viewer/re_view_spatial/src/visualizers/grid_map.rs:215`、コメント「The bounding box is flat, but this is distinctively a 3D object in a 3D space!」）。

証拠: 対照群を1フレームへ並べたキャプチャで、`Image` 2通りが不在、`GridMap` が描画。`Image` を足す前後のPNGがバイト単位で同一だった。

## 3. `GridMap` 経由なら画素ごとのalphaが通る（決定）

`SpatialStage::copy_gpu_image` が log する archetype を `Image` から `GridMap` へ変えた。`GridMap` は3D viewで `RectangleRenderer` を通り、そこは RGBA・premultiplied・透明ブレンドを実装済みである。

**上流の挙動は変えていない。** `crates/viewer/re_viewer_context/src/gpu_bridge/image_to_gpu.rs` のCPUアップロード経路は `AlphaChannelUsage::DontKnow` を報告したままで（`#12223`）、そこを通るCPU画像は今も不透明矩形になる。GPU経路は `TextureManager2D` が書くcache entryを直接叩くため、その行に到達しない。

証拠: 同一フレーム内の陽性・陰性対照。GPU経由の前景は透明gutterから背景が透け、同じ素材をCPU経由の `GridMap` で流した対照は黒い矩形のまま残る。**後者が黒いままであることが、上流を書き換えていない証拠になる。**

画素oracle: 黒画素0、checkerboard 629,659、marker 4,117。

## 4. ゼロコピーで渡せる（決定）

毎フレームのblitを廃し、embedderのtextureをそのままサンプルする。`GpuTexturePool::import` が外部textureを実handleで登録し（bind group生成がhandleで解決するため必須）、`begin_frame` は借り物のhandleを帳簿から外すだけで `destroy()` を呼ばない。

`import_gpu_premultiplied` は key でキャッシュしない。ダブルバッファのembedderは同じkey・同じdescriptorの2枚を交互に渡し、`wgpu::Texture` に見分ける手段が無いためである。

証拠:
- 出力がコピー版とバイト一致（sha1 `9e8ecd1d…`、1回の比較）
- レイヤーを外し60フレーム後に**同じtexture**を再投入して、wgpu検証層が鳴らない（uncaptured error handlerでexit 3にしてある）
- texture pool常駐数がframe 40/120/200で 8 → 11 → 11。毎フレームre-importしても積み上がらない
- `cargo test -p re_renderer --lib` 29件パス

## 5. z=0のままdraw orderでレイヤー順序が成立する（観察）

背景と前景を厳密に同一平面へ置き、前後を `draw_order` だけで決めても z-fighting は出ない。

`crates/viewer/re_renderer/src/renderer/rectangles.rs:576` が同一平面で重なる矩形をクラスタとして検出し、透明フェーズへ強制する（`force_transparent = cluster_info.has_coplanar_overlap`）。並びは `secondary_sort_key: depth_offset`（`GridMap` の `draw_order` 由来）。透明パイプラインは `MAIN_TARGET_DEFAULT_DEPTH_STATE_NO_WRITE` で**深度を書かない**。

**含み**: 深度を書かないため、レイヤーは先に描かれたジオメトリに対しては深度テストされるが、後から描かれるジオメトリを遮蔽できない。2.5Dのレイヤースタックとしては望ましい挙動だが、`DeformedMesh`（`VSM-M5-G0`）や `PieceSet`（`VSM-M5-G1`）と同じStageで混ぜる段で非対称が効く。

z を「見た目の前後」と「3D空間内の位置」の二重帳簿にせずに済む。

## 6. 何を決めていないか

- **3 OS未検証。** 1台のGPU・1 OSのみ
- **coplanarに自動oracleが無い。** z-fightingは斑として出るため、現行の3判定（黒／checkerboard／marker）では捕まらない
- **ゼロコピー同一性はハッシュ1回**で自動化していない
- **`release_imported` に呼び出し元が無い**（デッドコード）。実際の解放は `Inner::begin_frame` の暗黙pruneに依存
- **`GridMap` は行0を上端に置く。** マーカー付きテクスチャで確認済みで反転は無いが、実画での確認は未実施
- **製品の合成経路は未確認。** `adapter.rs` は `DOCUMENT_FRAME_ENTITY`（単数）へ評価済み1枚を渡し、レイヤーはメッシュとして置かれる。呼び出し側がこのツリーに無いため、最終合成をHostが持つという読みはシグネチャからの推測である

## 7. `VSM-M5-S0` GlassSurfaceへの含み

[Vism既知実装採択マップ](../vism-known-implementation-adoption-map.md) と [M5仕様](../specs/M5-3d-and-post.md) は2026-08-12時点で「標準 `Mesh3D` のalbedo／textureをGlass BSDF完成とみなさない」と書いていた。§1により、これは**慎重な線引きではなく測定された制約**になった。`Mesh3D` はテクスチャのalphaを出さないので、custom renderer seamは選択ではなく必然である。

一方 `VSM-M5-G0`／`G1` は不透明ジオメトリなので、§1の制約に**触れない**。「`Mesh3D` が駄目」を全体へ広げないこと。

## 8. 実装

fork `oshikaidesu/rerun` branch `codex/spatial-alpha-probe-20260817`:

- `252c9cef7` — `GridMap` 化・ゼロコピー import・借り物を破棄しない解放経路
- `501a0403b` — 上が壊した `dynamic_resource_pool` のテスト8箇所の修正

probe は `spikes/rerun-vism-gpu-alpha-probe`（`--stress` で解放経路とlong runとPNG oracle）。CPU経路の陰性対照は `spikes/rerun-vism-layer-alpha-probe`。
