# 第三者の作品とライセンス

## スクリプトに含まれる第三者のコード

### Dave Hoskins「Hash without Sine」

`@ShapeDistort_H.anm2` `@ローポリ化.anm2` `@ローポリ背景.obj2` `泡_H.obj2` `プロシージャル模様_H.obj2` のハッシュ関数。
出典: https://www.shadertoy.com/view/4djSRW

```
MIT License

Copyright (c) 2014 David Hoskins.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Inigo Quilez の 2D 距離関数

`グリッド_H.obj2` の `sd_box` `sd_ellipse` `sd_rhombus` `sd_polygon`。
出典: https://iquilezles.org/articles/distfunctions2d/

```
MIT License

Copyright (c) Inigo Quilez

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### sevenc-nanashi「extra_blend_mode.anm2」

`色空間合成_H.anm2` は、extra_blend_mode.anm2（https://github.com/sevenc-nanashi/extra_blend_mode.anm2 ）のフォーク。
上下の絵の取り出し方（Lua の呼び出し）とアルファの扱いを原作から使い、色空間に関係するモードを Oklch / LCh とリニアで計算し直した。

```
MIT License

Copyright (c) 2026 Nanashi. <sevenc7c.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Korarei「Diffusion_K」の Noise(HSLA)

`@Filters_H.anm2` の「色ノイズ」は、Diffusion_K（https://github.com/korarei/AviUtl2_Diffusion_K_Script ）の Noise(HSLA) v1.2.3 のフォーク。
HSLA のノイズに Oklch を選べるようにしたもの。

```
MIT License

Copyright (c) 2026 Korarei

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Mark Jarzynski・Marc Olano「PCG3D」

`@Filters_H.anm2` の「色ノイズ」の乱数（`pcg4d`）。原作の Diffusion_K が PCG3D を改めたものをそのまま使っている。
出典: https://github.com/markjarzynski/PCG3D

```
The MIT License

Copyright 2020 Mark Jarzynski & Marc Olano

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

## 元作品の規約に沿って公開しているもの

| ファイル | 元作品 | 規約 |
|---|---|---|
| `装飾パーティクル_H.anm2` | アス（AVILITY）「装飾パーティクル」 https://avilityyy.blogspot.com/2025/11/decoParticle.html | [AVILITY の利用規約](https://avilityyy.blogspot.com/p/blog-page_83.html)。元作品を組み込んだ自作スクリプトとしての配布が認められている。クレジット: AVILITY |

## 作り直したもの（元作品のコードは使っていない）

| ファイル | 元作品 | 説明 |
|---|---|---|
| `@拡張パーティクル_H.anm2` `Particle_H.lua` | rikky 氏「拡張パーティクル(R)」（AviUtl 無印用） | 元作品の説明書から機能を拾い、AviUtl2 向けにすべて書き直した。元作品の計算部分（DLL）のソースは公開されておらず、コードは使っていない。設定の考え方と機能の構成は元作品にならっている |

## 改造（元作品の作者またはライセンスが不明）

次のファイルは、作者またはライセンスの分からない作品を元にしています。
**元作品の部分の権利は元作品の作者にあり、`LICENSE`（MIT-0）の対象外です。**
元作品の作者の方で、公開に問題がある場合は、Issue か下記の連絡先からお知らせください。該当するファイルを取り下げます。

| ファイル | 元作品 | 分からないもの |
|---|---|---|
| `@KD_DepthMap_H.anm2` | Toolbox_kurodonut v1.1「深度マスク」（kurodonut 氏とみられる） | ライセンス |
| `@カメラ効果_H.cam2` | `@Camera.cam2` | 作者・ライセンス |
| `テキスト両端揃え_H.anm2` | 微熱 氏「テキスト両端揃え」（AviUtl 無印用。https://www.nicovideo.jp/watch/sm43100896 ） | ライセンス |
| `キラキラ_H.anm2` | `@ANM1.anm2` の「キラキラ」 | 作者・ライセンス |
| `カラフルパーティクル_H.obj2` | `@FIGURE.obj2` の「カラフルパーティクル」 | 作者・ライセンス |
| `カラフルランダム配置_H.obj2` | `@FIGURE.obj2` の「カラフルランダム配置」 | 作者・ライセンス |
| `円形変位_H.anm2` | `円形変位.anm2` | 作者・ライセンス |
| `ランダム2_H.anm2` | `ランダム2.anm2` | 作者・ライセンス |
| `偽被写界深度_H.anm2` | `偽被写界深度2.anm2` | 作者・ライセンス |
| `円形配置プラス.anm2` | `円形配置改.anm2` | 作者・ライセンス |
| `sin揺れ_HB.anm2` | 「Sin揺れ(調整版)」（発想と式の骨格。コードは書き直し） | 作者・ライセンス |
| `@図形グラデーション_H.anm2` | しけもち 氏「グラデーションジェネレーター」（v2.00 で機能を取り込んだ。 https://note.com/sikemoti/n/n23f4da3cd318 ） | ライセンス |
| `レベル補正_H.anm2` | しけもち 氏「レベル補正」（項目の構成。 https://note.com/sikemoti/n/n9b1843bfb32f ）、こんにゃくねこ 氏「Level」（CC0-1.0） | しけもち 氏版のライセンス |
| `個別遅延_H.anm2` | `@DelayMove.anm2` の「Delay個別」（AviUtl2 への移植版） | 作者・ライセンス |

連絡先: https://github.com/HexBrowns
