# Shader Lab の効果 44 本：元の形が外にある物と、TSL にしか無い物（10/26）

Shader Lab（basementstudio/shader-lab、Apache-2.0）の効果を、外にある素のシェーダー（WGSL・GLSL・ISF）と 1 本ずつ照らした。
「TSL を根本から殺したら何を失うか」を量るため。

- 照らした先：
  - Motolii に入っている 295 本（`filament/effects`、ISF・three.js・vgpu）
  - Noisemaker（MIT、`shaders/effects/*/*/wgsl`、WGSL と GLSL の両方あり、約 230 本）
  - EffectCraft（Apache-2.0／MIT、`docs/effects.md`、AE の効果の写し、GPU 280 本）
  - Paper Shaders（Apache-2.0）
- 照らし方：Shader Lab の説明（`src/lib/editor/config/layer-catalog.ts`）と、相手の説明（Noisemaker の `help.md`、EffectCraft の一覧）を読み比べた。
  - **絵を並べて見比べてはいない。** 「同じ」は、効き方が同じという意味。
- 数えた物：効果の層 39 本と、絵を作る層 5 本（Mesh Gradient、Fluid、Magnify Lens、Pixel Trail、Shape）。
- 数えない物：画像・動画・カメラ・文字・グループ・Custom Shader（使う人が書く TSL）。

## 数

| | 本数 |
|---|---:|
| 同じ効き方の物が外にある | 20 |
| 近い物はあるが、一部だけ | 13 |
| TSL にしか無い | 11 |

## 同じ効き方の物が外にある（20 本）

| Shader Lab | 外の物 |
|---|---|
| ASCII | Noisemaker `filter/glyphMap` |
| Bloom | Noisemaker `filter/bloom`、Motolii `Bloom.fs` |
| Chromatic Aberration | Noisemaker `filter/chromaticAberration`、Motolii `rgbshift` |
| CRT | Noisemaker `filter/crt`、Motolii `crt_finish`・`Bad TV.fs` |
| Directional Blur | Noisemaker `filter/directionalBlur`、Motolii `Directional Blur.fs`・`Radial Blur.fs` |
| Displacement Map | EffectCraft `ec.distort.displacementmap`、Motolii `Displace.fs` |
| Dithering | Noisemaker `filter/dither`、Motolii `Dither-Bayer.fs` |
| Edge Detect | Noisemaker `filter/edge`、Motolii `Edges.fs` |
| Halftone | Noisemaker `filter/halftone`、Motolii `CMYK Halftone.fs`・`RGB Halftone.fs`・`halftone` |
| Dot Grid | Noisemaker `filter/halftone`（点の型）、Motolii `dot_screen` |
| Pixelation | Noisemaker `filter/pixels`、Motolii `Pixellate.fs` |
| Pixel Sorting | Noisemaker `filter/pixelSort` |
| Posterize | Noisemaker `filter/posterize`、Motolii `Posterize.fs` |
| Threshold | Noisemaker `filter/threshold` |
| Gradient Map | EffectCraft `ec.color.colorama` |
| Grain | Noisemaker `filter/grain`、Motolii `fine_grain`・`film` |
| Photocopy | Noisemaker `filter/photocopy` |
| Mesh Gradient | Paper Shaders `packages/shaders/src/shaders/mesh-gradient.ts`（GLSL） |
| Magnify Lens | EffectCraft `ec.distort.magnify` |
| Shape | Motolii の形（ThorVG）。シェーダーではない |

## 近い物はあるが、一部だけ（13 本）

| Shader Lab | 近い外の物 | 足りない所（説明から） |
|---|---|---|
| Blur（被写界深度・ティルトシフト・ボケ） | Noisemaker `mixer/focusBlur`、EffectCraft `ec.blur.cameralens` | 1 本で全部の型を持つ形ではない |
| Progressive Blur | three.js のティルトシフト（Motolii には未収録） | 傾きの向き・ランプ |
| Relief | Noisemaker `filter/relief`・`filter/emboss`、Motolii `Emboss.fs` | 銀・金箔の照らし |
| Outline | Noisemaker `filter/outline` | 二重・破線・波形の輪の繰り返し |
| Flares | Noisemaker `filter/lensFlare` | 明るい点ごとの十字・星 |
| Glass | Motolii `Frosted Glass.fs` | 筋・槌目・六角のレンズの粒 |
| Slice | Noisemaker `filter/scanlineError` | 横の帯を左右にずらす形そのもの |
| Signal Rot | Noisemaker `filter/degauss`・`filter/corrupt`、Motolii `Bad TV.fs` | 引きずり・ゆらぎを 1 本に |
| Pattern | Noisemaker `filter/glyphMap` | 図柄で明暗を組み直す |
| Plotter | Noisemaker `filter/hatch`・`filter/stipple`・`filter/strokes` | 流線・渦・等高線 |
| Voxel | Noisemaker `filter/extrude` | 斜めから見た立方体 |
| Fluid | Noisemaker `synth/navierStokes` | 絵を流す使い方 |
| Pixel Trail | Motolii `RGB Trails 3.0.fs`・`Echo Trace.fs`、Noisemaker `filter/feedback` | 道に沿う尾 |

## TSL にしか無い（11 本）

| Shader Lab | 説明 | 一番近い外の物 |
|---|---|---|
| Photographic Cells | 格子の粒をつないだ領域で写真を見せる | Noisemaker `mixer/cellSplit` |
| Displaced Rings | 同心円や半円に切ってずらす | なし |
| Circuit Bent | 走査線にして、1 点のまわりで曲げる | なし |
| Annotations | 照準・十字・枠・定規・読み取り値の飾り | Noisemaker `filter/osd` |
| Blob Tracking の飾り | 枠・かっこ・札 | なし（見つける所は `tracker.ts` で JS。TSL ではない） |
| Connected Dots | 点を近くどうし線でつなぐ（plexus） | なし |
| Ink | にじむ光、インクのような縁 | Noisemaker `filter/glowingEdge` |
| Particle Grid | 光る粒の格子に分ける | なし |
| Erosion | 縁から粒に崩れ、外へ飛び散る | Noisemaker `filter/scatter` |
| Lumen Print | 日光写真（陰の色、ソラリゼーション、にじみ、焼けた縁、粒） | なし |
| Fluted Glass | 縦の筋ガラス、少し色が割れる | なし |

## 分かったこと

- 半分足らず（20 本）は、昔からある効き方の書き直しだった。TSL を殺しても、元の形で手に入る。
- TSL にしか無い 11 本は、照準の飾り、plexus、追跡の枠、日光写真、筋ガラスなど、Shader Lab らしい「今っぽい」見た目に集中している。TSL を殺して失うのは、ここ。
- 追跡の「見つける所」は TSL ではなく、普通の JS（`tracker.ts`）だった。

## 10/26 の直し：照らした先が間違っていた

- **EffectCraft は手本にしない。** 自分で「つまみの名前・順番・初めの値まで After Effects に合わせた」と書いている（`docs/effects.md` の頭）。AE の古い技を、古いまま写した物。
- **Noisemaker の多くも同じ。** `filter/hatch`（Graphic Pen、Charcoal…）や `filter/relief`（Bas Relief、Plaster、Note Paper）は、Photoshop の古いフィルタの名前をそのまま並べている。
- なので「同じ効き方の物が外にある 20 本」の多くは、古い技の代わりが見つかっただけ。新しい表現の元ではない。
- **数え方が逆だった。** Motolii が欲しいのは、ネットで見つかる新しい表現。それは TSL にしか無い 11 本の側と、Shader Lab が古い技を広げた 13 本の側にある。TSL を殺すと、ちょうどその新しい所を失う。
- 新しい表現が TSL で書かれて出てくる以上、殺すべきなのは TSL ではない。TSL から GPU までの間に Motolii 自身が足している翻訳の層（WGSL の口の決まり、Filament）の方。
