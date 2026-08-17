# Rerun `copy_gpu_image` alpha-layer probe

上流無改変の pin (`8c6865ac`) のまま、`SpatialStage::copy_gpu_image` で渡した
GPU-resident な premultiplied RGBA texture が、3D `SpatialStage` 上で
画素ごとの alpha として合成されるかを確認する Rerun-only の隔離 probe である。

```text
premultiplied RGBA wgpu::Texture
  -> SpatialStage::copy_gpu_image (TextureManager2D::copy_from_gpu_premultiplied)
  -> AlphaChannelUsage::AlphaChannelInUse でタグ付けされた texture cache entry
  -> standard Image visualizer / RectangleRenderer (3D 登録済み)
  -> 3D SpatialStage screenshot
```

`spikes/rerun-vism-layer-alpha-probe` との違いは前景の入口だけである。あちらは CPU 画像を
`GridMap` で流すため `image_to_gpu` の `AlphaChannelUsage::DontKnow` を通り、opaque pass に
落ちて黒い矩形になる。こちらは `copy_gpu_image` が texture を直接タグ付けするため、
上流の当該行を通らない。

正常なら、前景の透明 gutter から背面 checkerboard が見える。前景が不透明な黒い矩形に
なるなら、上流無改変では成立しない。

## 必要な fork の版

`SpatialStage::copy_gpu_image` が `Image` ではなく `GridMap` を log する版が要る。
`Image` は `SpatialView3D` で平面として描かれないため、それ以前の版では前景が
一切出ない(このprobeで確認済み)。fork 側のコミットは

    feat(spatial): show embedded GPU frames as transparent 3D layers

`Cargo.toml` の rev がまだそれを含まない間は、ローカル checkout を指す
`[patch]` を **worktree ローカルで** 足して走らせる。絶対pathを含むので
コミットはしない。

```toml
[patch."https://github.com/oshikaidesu/rerun"]
re_chunk = { path = "<rerun checkout>/crates/store/re_chunk" }
re_log_types = { path = "<rerun checkout>/crates/store/re_log_types" }
re_renderer = { path = "<rerun checkout>/crates/viewer/re_renderer" }
re_sdk_types = { path = "<rerun checkout>/crates/store/re_sdk_types" }
re_view_spatial = { path = "<rerun checkout>/crates/viewer/re_view_spatial" }
```

```bash
cargo run -p rerun-vism-gpu-alpha-probe -- \
  --screenshot /private/tmp/rerun-vism-gpu-alpha.png
```

## 読み方

- 左: 背景 checkerboard の上に、`copy_gpu_image` 経由のGPU textureが重なる。
  透明gutterから格子が見え、soft edgeが混ざり、core(alpha 0.92)も薄く透ける。
- 右: 同じ blur を CPU 経路の `GridMap` で流したもの。上流の `DontKnow` を通るため
  黒い矩形のまま残る。これが残っていることが「上流を書き換えていない」証拠になる。
- 8フレーム目でhostが別のtextureへ差し替える。マーカー(緑=画像左上/白=右下)が
  出ていれば、importが毎回取り直している。出なければ古いtextureに貼り付いている。
- 背景と前景は厳密に同一平面(z=0)で、前後は`draw_order`だけで決めている。
  `copy_gpu_image`が自分のpathへ置くz=-0.01は親の+0.01で打ち消す。ちらつきや斑が
  出れば、coplanar clusterの検出か透明フェーズへの強制が効いていない。
- マーカーの位置は上下方向も兼ねる。緑(画像の行0)が奥側に出れば反転なし。

これは標準 3D consumer の透明合成だけを確認する fixture であり、Vism runtime、Filter、
Vism API、Preview/Export、`motolii-blitz-shell` へ接続しない。
