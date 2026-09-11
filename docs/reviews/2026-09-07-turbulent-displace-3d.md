# 3D 版 Turbulent Displace — 場の効果

利用者裁定(2026-09-07): 入口は効果棚。AE の Turbulent Displace と同じ欄(Amount・Size・Complexity・Evolution・Offset)を、画素のワープでなく**世界を動かす場**として持つ(裁定 2026-09-02「Displacement は位置の変化を担う。Mesh の頂点・点群の点・2D のサンプル位置へ native に生かす」の最初の実装)。

## 使い方

1. 層(網・点群・板 = 画像/文字/図形)を選び、Effects 棚の **Turbulent Displace** を載せる。2D は 3D の部分集合なので板にも同じ場が効く(2026-09-11)
2. Amount(世界単位の振れ幅)・Size(模様の大きさ)・Complexity(重ねる段数 1〜8)・Evolution(進めると流れる。キーを打てば乱流)・Along(Normal = 法線に沿って高さで動かす、Space = ベクトル場で動かす)・Offset X/Y/Z
3. Glass など表面効果と重ねられる(場は頂点、面はその後の陰影)

## 中身

| 家 | 何を | 正本 |
|---|---|---|
| 棚 | 場の効果 `motolii.turbulent_displace` は manifest 付き WGSL の 1 file(同日の最小コア C で `store/field.rs` から移した)。選択肢は ISF の `long` + `LABELS` | `render/vism/turbulent_displace.wgsl` |
| fork | `GpuMeshInstance::displace`(MeshDisplace)。頂点 shader で instance の枠(回転・拡縮、平行移動なし)の座標を Size で割り、Evolution で流した simplex fbm を引く。Normal は法線方向に高さを足し、勾配で法線を曲げる(bump)。Space は 3 つの場でベクトル | `shader/utils/noise.wgsl`、`shader/instanced_mesh.wgsl`、`src/noise.rs` |
| render | 網は fork の hook(GPU)、点群は CPU の写し `PointDisplace` で同じ座標の取り方・同じ場を引く。点群は法線が無いので Normal でもベクトル場。CPU の写しが id を名指ししているのは継ぎ目 D | `compositor/point_cloud.rs` |
| fork(板) | 板は同じ `motolii_field` を断片で引き、変位を**標本位置のずれ**として見せる: 面内の成分はそのまま、法線方向の成分は視線で写す(parallax mapping、Kaneko 2001。掠め角は 0.1 で止める)。曲げた法線は Glass 等の表面 hook へ渡る。籠(outline mask)もずれた絵に従う。枠は板の左上が原点、extent_u/v が軸 | `shader/rectangle_fragment.wgsl` `sample_field`、`shader/utils/field.wgsl`、`mesh_program.rs` |
| ui | 欄は property。Along は選択肢 | Inspector の generic な効果行 |

## 借りた定規

- Simplex noise: Ashima Arts / Stefan Gustavson の webgl-noise(MIT)。WGSL と Rust の 2 つの写しを同じ家(fork)に並べる
- fbm: lacunarity 2・gain 0.5、段の和で正規化
- 法線の曲げ: 高さ場の勾配の接線成分を引く(bump mapping の式)
- 製品先例: AE Turbulent Displace(欄の名前と意味)

## 契約 test

- fork: noise が有界・滑らか・変化する
- doc: 既定と選択肢
- render(実 GPU): Space の変位で絵が変わり、Evolution でまた変わり、Normal の変位で陰影が変わる。点群は Amount を超えて動かず、Evolution で動く。板も Space でずれ、Evolution で流れ、Amount 0 は掛けない絵と同一(`turbulent_displace_warps_a_flat_picture_too`)
- 棚: 並んでいて pass にならず、既定 Amount 50

## 宿題

- GPU と CPU の場が数値で一致することの検証(同じ写しだが、GPU 読み戻しの test は未作成)
- 板の変位は枠の外へ出ない(標本が枠内に留まる)。網の bounds と同じ宿題
- 変位後の bounds(選択枠・fit)は変位前のまま
