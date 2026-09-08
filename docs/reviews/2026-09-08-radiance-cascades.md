# Radiance — 素材の光が周りへ回る(vgpu radiance-cascades の移植)

利用者指示(2026-09-08): vgpu の例から「元の素材があってこそ輝く」物を取る。自前で絵を作る生成器(black-hole・fractal)は外す。

## 何を借りたか

[vercel-labs/vgpu](https://github.com/vercel-labs/vgpu) `apps/docs/examples/radiance-cascades`(MIT)。jump flood で距離場を作り、6 段の radiance cascade を上から合流して 2D の global illumination にする。式・段の間隔(base 4、interval 2·4^c)・合流(merge してから平均)はそのまま。

## Motolii での形

- 効果棚の `motolii.radiance`(Light)。**素材の明るい所が光源、素材の形が遮蔽**。文字や切り抜きがそのまま光る物になり、自分の角で影を落とす。素材が無ければ何も起きない
- 欄: Threshold(何が光るか)・Intensity・Reach(余白 = 光が届く範囲、PADDING)・Air(空気中に光を見せる量)
- 1 file `vism/radiance.wgsl`、20 段。vgpu が TypeScript で ping-pong していた 6 枚の target は、**ISF の「同じ TARGET 名は同じ buffer」**で manifest に畳んだ。段の分岐は PASSINDEX
- 探針の間隔を 2px にして atlas を素材と同じ大きさに収めた(vgpu は 1px で 2 倍の atlas)。種は f16 なので座標が正確なのは 2048px まで
- 出力は glow と同じく非乗算 alpha の float。空気中の光は alpha = 最大成分、色 = 光 / alpha にして、黒の上で光の値そのものになる

## runtime に足した物(1 箇所)

`IsfManifest::target_slots`: TARGET 名の初出順で buffer を割り、同名のパスは同じ 1 枚へ書く(`vism.rs` の割当・書き先・pipeline format、`catalog.rs` の binding 検証)。既存の glow(名前が全部違う)は挙動不変。

## 見つけた穴

- WGSL の予約語 `pass` を引数名に使うと catalog の validation で落ち、**効果は棚から黙って消える**(`translate_effect_passes` が未知の id を捨て、`layer_failures` にも出ない)。今回は `refresh_effect_catalog().errors` を見て気づいた。棚から消えた理由を窓へ返す口は宿題

## 検収

`render_effects::tests::radiance_lights_the_air_around_emitters_and_occluders_cast_shadows`(実 GPU、validation scope 付き): 白い光源の横の空気が明るい、壁の裏は表の半分未満、遠い角は近くより暗い、効果を外すと空気は黒。render 43 本。

## 宿題

- 光の色付け(光源の色はそのまま通るが、素材の albedo に当たる分は `light × 素材色` の単純積)
- 段数・jump 数は固定(6 段・jump 512 まで)。4K 超の素材では距離場が近似になる
- 探針間隔 2px の解像度。細い線の光は少し太る
