# AviUtl 変換器の次の手（10/26、自分たちで決めた）

変換器（`filament/aviutl.cpp`・`aviutl_fx.h`）に足す物は、どれも「借りた決まり ＋ Motolii がもう持っている借り物」へ写すだけにする。自分で効果や 3D の描き方は書かない。

## お手本（どちらも MIT）
| 物 | 何を借りるか | 何を借りないか |
|---|---|---|
| **sashimono-edit**（kagemorikosame、Python） | `engine/gpu/projection.py`：AviUtl の 3D の決まり（カメラまで **1024 画素**、回す順は **X→Y→Z**、X が正なら上の端が奥へ、Y が正なら右の端が奥へ、Z は時計回り、近すぎる点は 1 画素で止める）。`compat/aviutl/mapping.py`：本体の効果の名前とつまみの名前の対応。`tests/compat/`：本物の AviUtl で測った効果の答え（こちらの検査の答えに使う） | `effects/builtin.py` の効果の中身（Python で書き直した物。Motolii は ISF の借り物を使う） |
| **YMM4-LuaScript**（routersys、C#） | `obj.pixelshader` を D3D11 で動かすときのふるまい | `DrawPolyMath.cs`：3 つの角だけの 2D の貼り方（奥行きを捨てる）。AviUtl より単純なので 3D の手本にしない |

## 3D：描かずに「置く」
- `obj.draw(ox, oy, oz, zoom, alpha, rx, ry, rz)` と `obj.drawpoly(4 点＋uv)` は、どちらも「四隅を画面のどこへ写すか」で決まる（sashimono の projection.py の考え）。
- Motolii では、四隅を 3D の世界に置いた板（Filament の頂点を毎コマ書き換える形、Filament の見本にある）にし、カメラを AviUtl の既定（距離 1024 画素）に合わせる。自分でラスタライズしない。
- 検査：sashimono の projection のテストの値と、同じ入力で四隅が同じ所へ来るか。

## 本体の効果を名前で：上位 30 種の当て先
数は、呼んでいるパッケージの数（無印 ／ AviUtl2）。当て先は Motolii に今ある借り物（ISF-Files、MIT）か、Motolii の仕組み。

| 名前 | 数 | 当て先 |
|---|---|---|
| 領域拡張 | 4 ／ 14 | 絵の大きさを広げるだけ（もう入っている） |
| ぼかし | 4 ／ 10 | `Fast Blur.fs`（または物の blur） |
| 単色化 | 4 ／ 8 | `Color Monochrome.fs` |
| クリッピング | 4 ／ 7 | 絵の大きさを詰めるだけ |
| 反転 | 3 ／ 7 | 上下・左右は `Flip V.fs`・`Flip H.fs`、輝度・色相・透明度は `Color Invert.fs` |
| リサイズ | 3 ／ 6 | 絵の大きさを変えるだけ |
| 色調補正 | 4 ／ 4 | `Color Controls.fs` |
| グラデーション | 4 ／ 3 | `Linear Gradient.fs`・`Radial Gradient.fs` |
| マスク | 4 ／ 3 | 物の切り抜き（clip） |
| 斜めクリッピング | 4 ／ 2 | 物の切り抜き（角度つき） |
| ノイズ | 4 ／ 2 | `Noise.fs` |
| 方向ブラー | 4 ／ 1 | `Linear Blur.fs` |
| 境界ぼかし | 4 ／ 1 | `Edge Blur.fs` |
| 縁取り | 3 ／ 2 | 物の stroke（絵の輪郭）※確かめる |
| 発光 | 4 ／ 1 | `Glow.fs`・`Bloom.fs` |
| 色ずれ | 4 ／ 1 | `Chromatic Aberration.fs` |
| 極座標変換 | 4 ／ 0 | ossia の `Polar Coordinates.fs`（CC BY-SA 4.0） |
| エッジ抽出 | 3 ／ 1 | `Edges.fs` |
| 凸エッジ | 4 ／ 0 | `Emboss.fs` |
| ルミナンスキー | 2 ／ 2 | ossia の `Luma Key.fs` |
| カラーキー・クロマキー | 4 ／ 2 | `Chroma Mask.fs` |
| ミラー | 3 ／ 0 | `Mirror.fs` |
| 画像ループ | 3 ／ 0 | `Quad Tile.fs` |
| グロー | 3 ／ 0 | `Glow.fs` |
| モザイク | 3 ／ 0 | `Pixellate.fs` |
| 放射ブラー | 3 ／ 0 | `Zoom.fs`（中心から放つ） |
| 波紋 | 2 ／ 0 | `Ripples.fs` |
| ディスプレイスメントマップ | 2 ／ 0 | `Displace.fs` |
| レンズブラー | 2 ／ 0 | `Soft Blur.fs`（近いだけ） |
| ライト・シャドー・ラスター・振動・ローテーション | 各 1〜3 | 未定（探す） |

- AviUtl2 のスクリプトは、ほかのスクリプトを名前で呼ぶ（`ホワイトバランスσ@WhiteBalance_S` など）。同梱したスクリプトどうしをつなぐだけで済む。
- つまみの名前と単位は sashimono の `mapping.py` に合わせ、ISF のつまみへ写す。ふるまいの答えは sashimono の測った値で確かめる。

## Web の表現（p5.js など）：ブラウザ表示
- AviUtl で p5.js が動くのは、oov の aviutl_browser（MIT、無印用）と、その AviUtl2 版の HexBrowns の AviUtl2Browser_H（MIT）。中は CEF（Chromium を部品にした物）で、HTML・CSS・JavaScript を毎コマ物の絵にする。Windows 専用の exe と DLL なので、今の変換器では止まる。
- 約束はとても小さい：スクリプトは `require("browser").execute({abc|dir, param, tabid, userfile})`、コンテンツの index.html は `AviUtlBrowser.registerRenderer(async params => 文字列)`。左上の 1 画素は書き終わりの合図に使われる。
- Motolii では、変換器に `require("browser")` の代役を足し、同じ約束でブラウザに描かせる。そうすれば aviutl_browser 用のコンテンツ（.abc・フォルダ、p5.js のスケッチを含む）がそのまま動く。
- ブラウザの部品の候補：CEF（BSD、元と同じ、Mac と Windows で同じ動き、oov の動かす側のコード `src/exe/client.cc` などが MIT のお手本、約 180 MB）か、OS のブラウザ（Mac は WKWebView、Windows は WebView2、軽いが 2 通り）。

## 決まり
- 新しいファイルに分け、どれも 500 行以内（今の `aviutl_fx.h` は 385 行、合わせて 654 行で超えている）。
- 効果の中身は書かない。名前の当て先の表だけ。当て先の無い物は「まだ」と言って止める。
