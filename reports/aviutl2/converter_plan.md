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
- 10/26 oov の aviutl_browser（MIT、https://github.com/oov/aviutl_browser ）を読んだ。本体は約 1,300 行で、ほとんどが CEF の呼び出し。CEF を画面に出さずに描く形（`src/exe/app.cc:41` の `SetAsWindowless`）。ページへの頼みは `ExecuteJavaScript` で `registerRenderer` の関数を呼ぶ（`client.cc:399`、`app.cc:129`）。描き終わりは、ページが左上の 1 画素に「何回目か」を表す色を置き、受け取り側の `OnPaint` がその色を確かめてから絵を受け取る（`client.cc:139-175`）。AviUtl とは別の exe で動く。同じ作者の中身の例：aviutl_browser_glb（3D の物）、_minecraft_skin、_emoji、_markdown。
- 10/26 確かめ：CEF 156（Chromium 156.0.8078.48、Mac の Apple シリコン版、BSD、https://cef-builds.spotifycdn.com/ の標準配布）を README の手順どおり `cmake -G Ninja -DPROJECT_ARCH=arm64` で組み立て、見本 cefsimple・cefclient と検査 ceftests が手を入れずにできた（36 秒と 80 秒）。CEF 自身の「画面に出さずに描く」検査 `ceftests --gtest_filter='OSRTest.*'` が **93 本すべて通った**（画素の確かめを含む）。注意：cefclient を動かすと macOS の通知の許可を求める知らせが出る。
- 同じ道で、Tracery 級の追跡オーバーレイ（blob-tracey、MIT、ブラウザの Canvas に頼る）と p5.js のスケッチが手を入れずに動く。
- Mac での手順（手本どおり）：(1) CEF の Mac 版（BSD）を、CEF 公式の見本 cefsimple のまま動かす。(2) oov の client.cc の受け渡しをそのまま当てた別アプリにする。(3) 変換器の `require("browser")` からそれを呼ぶ。
- Motolii では、変換器に `require("browser")` の代役を足し、同じ約束でブラウザに描かせる。そうすれば aviutl_browser 用のコンテンツ（.abc・フォルダ、p5.js のスケッチを含む）がそのまま動く。
- ブラウザの部品の候補：CEF（BSD、元と同じ、Mac と Windows で同じ動き、oov の動かす側のコード `src/exe/client.cc` などが MIT のお手本、約 180 MB）か、OS のブラウザ（Mac は WKWebView、Windows は WebView2、軽いが 2 通り）。

## 決まり
- 新しいファイルに分け、どれも 500 行以内（今の `aviutl_fx.h` は 385 行、合わせて 654 行で超えている）。
- 効果の中身は書かない。名前の当て先の表だけ。当て先の無い物は「まだ」と言って止める。

## 未解決：スクリプトが広げた絵を物の箱に収める（10/26、試して戻した）
- 今：`image_max` が物の箱の大きさなので、Open Glow・Long Shadow などの「領域拡張」（57 包が呼ぶ、35 包が `image_max` を読む）が箱で止まる。箱の余白は手で決める `fx_margin` だけ。
- 作り（AE の PreRender と同じ）：スクリプトには広げる前の物の絵を渡し（`obj.w`・`obj.h` は物の大きさ）、返った絵が広がった分だけ物の箱を広げる。`image_max` は画面の画素の大きさ（sashimono-edit の決まり）。言った量が前の回と違えば、同じコマで箱を合わせ直して描き直す。ISF は広げる量を言わないので `fx_margin`（AE の Grow Bounds にあたる）を残す。
- 試した結果（MV の文字、拡散 300 の Open Glow）：光は画面いっぱいまで広がったが、光った絵が左上に小さく出て元の場所には光の無い文字が出た。グロー → エッジ → グローでは絵が上にずれて二重になった。Long Shadow は箱が広がらず影が切れた。
- 次に読む物：同じコマで物の View を大きさを変えて 2 回描いてよいか（Filament の View・RenderTarget の決まり）、`thing_fx_aim` の板の置き方と絵の大きさの合わせ方。読んでから直す。
- 10/26 その後：手で決める余白 `fx_margin` は消し、sashimono-edit の `BAKE_MARGIN`（128 画素、`engine/render/script_bake.py`）に替えた。sashimono-edit の `image_max` は画面の大きさ（`compat/aviutl/objapi.py` の `lua_getinfo`）、領域拡張は物の絵そのものを広げる（同 `_expand`、上限 4096）。`image_max` を画面の大きさにすると、Open Glow を使うルック（Neon Edges・Glow Contours）の光が斜めにずれて二重になり、文字の中の線が消えた。値は出典どおりなので、ずれは変換器の不具合。原因は確かめていない。`image_max` は物の箱の大きさのまま残した。
- 10/26 出典の確かめ：`image_max` を画面の大きさにするのは 2 つの手本が同じ（sashimono-edit `compat/aviutl/objapi.py` の `lua_getinfo`、YMM4-LuaScript `native/worker.lua:747`・`LuaScript.Runtime/Engine/LuaScriptEngine.cs:989`、どちらも MIT）。ただ、「スクリプトの中で領域拡張して、広げた絵に `obj.pixelshader` を掛ける」流れ（Open Glow がこれ）は、どちらも持っていない。sashimono は広げるが `obj.pixelshader` が無い（`objapi.py` の `lua_pixelshader` は「無い」と記録するだけ）。YMM4 は `obj.pixelshader` を持つが、領域拡張は本体の余白の効果 `MarginEffect` へ渡す（`LuaScript.Compat/AviUtlCompatMap.xml`）。出典が無いので直さない。手本が見つかるまで `image_max` は物の箱の大きさのまま。
