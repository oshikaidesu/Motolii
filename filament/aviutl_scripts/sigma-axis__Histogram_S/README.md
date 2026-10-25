# Histogram_S AviUtl ExEdit2 ヒストグラム表示スクリプト

AviUtl2 のプレビュー画面上にヒストグラムを表示する AivUtl ExEdit 2 のスクリプトです．画像ファイルに対して色調補正をする場面などに表示しておくと便利です．

- 現在の編集画面全体や，特定のレイヤーに限ったヒストグラムを表示するオブジェクトを配置できます．

- 色空間も RGB や YUV, Oklab など複数から選べ，対数スケールでの表示も可能です．

- 動画出力中やプレビュー再生中は自動的に非表示に設定することもできます．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_Histogram_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45695828)

<img width="648" height="368" alt="Image of two histograms in front of the source image" src="https://github.com/user-attachments/assets/f411d100-3c88-49f2-8034-a62b6bcb14b6" />

- 元画像出典: https://www.pexels.com/photo/assorted-car-license-plates-533669

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `beta30a` で動作確認済み．

##  導入方法

`Histogram_S.obj2` ファイルに対して，以下のいずれかの操作をしてください．

1.  AviUtl2 のプレビュー画面にドラッグ&ドロップ．

1.  以下のフォルダのいずれかにコピー．

    1.  スクリプトフォルダ
        - AviUtl2 のメニューの「その他」 :arrow_right: 「アプリケーションデータ」 :arrow_right: 「スクリプトフォルダ」で表示されます．
    1.  (1) のフォルダにある任意の名前のフォルダ

初期状態だと「オブジェクトを追加」メニューの「編集補助 :arrow_right: 色調整」に「Histogram_S」が追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

##  ヒストグラムの仕様

1.  各[色空間](#色空間)ごとに色チャンネル 3 つがグラフとして表示されます．加えてアルファチャンネルも[「アルファ値」](#アルファ値)が有効なら表示されます．

1.  各チャンネルは 256 段階の度数分布として表現されます．この段階数より細かい差は反映されません．

1.  半透明ピクセルに対しては次のようにカウントされます:
    1.  色チャンネルのグラフでは，そのアルファ値で重み付けしてカウント (例えば 50% の半透明ピクセルは，0.5 ピクセル分).
    1.  アルファチャンネルでは，そのまま 1 ピクセル分としてカウント．

1.  HSV などのように「色相」を持っている色空間に対しては，次のように「色相」がカウントされます:
    1.  無彩色なピクセルはカウントされません．
    1.  [「色相を彩度で重み付け」](#色相を彩度で重み付け)が ON の場合は「彩度」に相当する成分でカウントに重み付けをします (例えば彩度が 50% のピクセルは 100% の場合の 0.5 ピクセル分).
    1.  これらに加えて，上述のアルファチャンネルの重みも適用されます．

##  パラメタの説明

<img width="500" height="702" alt="Image of GUI of the script" src="https://github.com/user-attachments/assets/0afb5410-dbaf-46c8-93b4-5e4bca7009bc" />

####  幅

ヒストグラムのグラフ1つ当たりの幅をピクセル単位で指定します．

- 実際のオブジェクト幅は左右に追加で 8 ピクセルずつの余白があります．
- ヒストグラムが 256 段階なので，256 の倍数だと歪みのないグラフになります．

最小値は 256, 最大値は 1024, 初期値は 256.

####  高さ

ヒストグラムのグラフ1つ当たりの高さをピクセル単位で指定します．

- 実際のグラフは上に追加で 8 ピクセルの余白 / オーバーフロー領域があります．

最小値は 16, 最大値は 400, 初期値は 100.

####  対象, レイヤー指定

ヒストグラムの統計対象を指定します．

1.  「対象」は次から選びます．

    1.  `フレームバッファ(アルファ値なし)`

        フレームバッファ全体を対象にします．半透明ピクセルは背景に黒を置いた不透明ピクセルとして取り扱われます．

    1.  `フレームバッファ(アルファ値あり)`

        フレームバッファ全体を対象にします．半透明ピクセルはそのまま半透明ピクセルとして取り扱われます．

    1.  `レイヤー(絶対指定)`

        「レイヤー指定」で指定したレイヤーを対象にします．

    1.  `レイヤー(相対指定)`

        ヒストグラムオブジェクトのあるレイヤーから「レイヤー指定」で指定したレイヤー分だけ，上下にあるレイヤーを対象にします．

    初期値は `フレームバッファ(アルファ値あり)`.

1.  レイヤー指定

    「対象」に `レイヤー(絶対指定)` や `レイヤー(相対指定)` を選んだ場合のレイヤーを指定します．それ以外の場合は無視されます．

    なお，指定したレイヤーは描画処理がもう 1 度実行されます．その分処理速度が遅くなる可能性があるので注意．

    - 「対象」が `レイヤー(絶対指定)` の場合，「レイヤー指定」の番号がそのままレイヤー番号になります．

      例: `8` だと `Layer8`.

    - 「対象」が `レイヤー(相対指定)` の場合，ヒストグラムオブジェクトのあるレイヤーから「レイヤー指定」で指定したレイヤー分だけ上下にあるレイヤーを対象にします．

      例: `-2` だと 2 つ上のレイヤー．

    初期値は -1.

> [!NOTE]
> AviUtl2 では1シーン当たりのレイヤー数に明確な上限がなく，またこのパラメタは時間経過で変化させる使用法は想定されにくいので，「レイヤー指定」はトラックバーではなくテキストボックスに直接入力する設定方法にしています．

####  色空間

色チャンネルのグラフに表示する色空間・座標を指定します．次の選択肢があります:

| 色空間 | チャンネル構成 | 参考 |
|:---|:---|:---|
| `XYZ` | $X\\;/\\;Y\\;/\\;Z$ | [sRGB](https://ja.wikipedia.org/wiki/SRGB) に準じた XYZ 色空間への変換． |
| `RGB` | $R\\;/\\;G\\;/\\;B$ | 変換なし． |
| `sRGB` | $R\\;/\\;G\\;/\\;B$ | [sRGB 色空間](https://ja.wikipedia.org/wiki/SRGB) (線形 RGB). |
| `Oklab` | $L\\;/\\;a\\;/\\;b, \quad (L: 0\\% \sim 100\\%;\\; a, b: -33.3\\% \sim +33.3\\%)$ | [Oklab 色空間](https://en.wikipedia.org/wiki/Oklab_color_space)． |
| `OKLCH` | $L\\;/\\;C\\;/\\;h, \quad (L: 0\\% \sim 100\\%;\\; C: 0\\% \sim 33.3\\%, h: 0\degree \sim 360\degree)$ | [OKLCh 色空間](https://en.wikipedia.org/wiki/Oklab_color_space)． |
| `YUV(BT.601)` | $Y\\;/\\;U\\;/\\;V, \quad (L: 0\\% \sim 100\\%;\\; U, V: -50\\% \sim +50\\%)$ | [YUV](https://ja.wikipedia.org/wiki/YUV) への BT.601 準拠の変換． |
| `YUV(BT.709)` | $Y\\;/\\;U\\;/\\;V, \quad (L: 0\\% \sim 100\\%;\\; U, V: -50\\% \sim +50\\%)$ | YUV への BT.709 準拠の変換． |
| `YUV(BT.2020)` | $Y\\;/\\;U\\;/\\;V, \quad (L: 0\\% \sim 100\\%;\\; U, V: -50\\% \sim +50\\%)$ | YUV への BT.2020 準拠の変換． |
| `HSV(円柱)` | $H\\;/\\;S\\;/\\;V, \quad (H: 0\degree \sim 360\degree;\\; S, V: 0\\% \sim 100\\%)$ | [HSV 色空間](https://ja.wikipedia.org/wiki/HSV色空間)の円柱座標系． |
| `HSV(円錐)` | $H\\;/\\;S\\;/\\;V, \quad (H: 0\degree \sim 360\degree;\\; S, V: 0\\% \sim 100\\%)$ | HSV 色空間の円錐座標系． |
| `HSL(円柱)` | $H\\;/\\;S\\;/\\;L, \quad (H: 0\degree \sim 360\degree;\\; S, L: 0\\% \sim 100\\%)$ | [HSL 色空間](https://ja.wikipedia.org/wiki/HLS色空間)の円柱座標系． |
| `HSL(双円錐)` | $H\\;/\\;S\\;/\\;L, \quad (H: 0\degree \sim 360\degree;\\; S, L: 0\\% \sim 100\\%)$ | HSL 色空間の双円錐座標系． |

- `Oklab` や `OKLCH` での彩度の上限が 33.3% なのは，RGB で表現可能な色は概ねこの範囲が限界のためです．

初期値は `RGB`.

####  色相を彩度で重み付け

HSV などのように「色相」を持っている色空間に対して，「色相」を「彩度」に相当する成分でカウントに重み付けをします．

- 例えば彩度が 50% のピクセルは 100% の場合の 0.5 ピクセル分となります．
- OFF の場合でも，無彩色なピクセルはカウントされません．
- RGB などのように「色相」を持たない色空間に対しては無視されます．

初期値は OFF

####  アルファ値

アルファチャンネルのグラフを表示するかどうかを指定します．アルファチャンネルは 4 つ目のグラフとして表示されます．

初期値は OFF.

####  拡大率

グラフの縦方向の拡大率を調整します．大きくすると小さい数値の差も見分けやすくなりますが，極端に大きな値は上端で途切れてしまいます．

[「スケール」](#スケール)が `線形スケール` の場合はグラフの下端を基準として, `対数スケール` の場合はグラフの下から 75% の線を基準として拡大縮小します．

% 単位で指定，最小値は 0, 最大値は 1600, 初期値は 100.

####  スケール

グラフで使う目盛りの種類を指定します．次の選択肢があります:

1.  `線形スケール`

    ピクセル数に比例するだけの長さでグラフを表示します．

1.  `対数スケール`

    ピクセル数の対数でグラフを表示します．極端にサンプル数が少ない部分も見分けやすくなります．


初期値は `線形スケール`.

####  平滑化

統計結果をグラフにする前に，度数分布を隣り合う区間と平均をとることで，グラフが滑らかになります．パレットベースなど減色した画像や，分布に偏りのある色空間を選んだときに見られる不自然なギザギザが軽減されます．

<img width="552" height="224" alt="Mollified graph of gradiation in a small image" src="https://github.com/user-attachments/assets/35d5fdd2-d78c-41d5-9326-050fc21a7855" />
<img width="552" height="224" alt="Mollified graph of image with decreased colors" src="https://github.com/user-attachments/assets/c5c1fa49-bae1-4719-85da-bd9902013b66" />

平均をとる隣り合う区間の個数 (自身を除く) を指定，最小値は 0, 最大値は 15, 初期値は 0.

####  描画色

グラフや[罫線](#罫線)の色を指定します．

- [「色を付ける」](#色を付ける)が ON の場合，一部のグラフの色にはこの「描画色」が反映されません．

初期値は `ffffff` (白色).

####  背景色, 背景透明度

ヒストグラムオブジェクトの背景色とその透明度を指定します．

1.  「背景色」の初期値は `000000` (黒色).
1.  「背景透明度」は % 単位で指定，最小値は 0, 最大値は 100, 初期値は 0.

####  色を付ける

一部の色チャンネルのグラフを，[「描画色」](#描画色)以外の色で描画します．

どの色チャンネルがどの色になるかは[「色空間」](#色空間)によって変わります．この色は調整できません．

初期値は ON.

####  罫線

罫線を表示するかどうかを設定します．

初期値は ON.

####  ラベル位置

グラフの各成分の名前を文字で表示する位置を調整します．左右位置を % 単位で指定，±100% だと非表示になります．

- フォントや文字の大きさの指定はできません．
  - 文字の大きさ[「幅」](#幅)や[「高さ」](#高さ)から計算されます．

最小値は -100, 最大値は 100, 初期値は -100 (非表示).

####  表示モード

動画出力中は自動的に非表示に設定することができます．次の選択肢があります:

1.  `常に表示`

    動画出力中やプレビュー再生中でも表示されます．

1.  `動画出力中は非表示`

    動画出力中には非表示になりますが，プレビュー再生中は表示されます．

初期値は `動画出力中は非表示`.

##  改版履歴

- **v1.10 (for beta30a)** (2026-01-??)

  - 「表示モード」から「プレビュー再生中も非表示」の選択肢を削除．
    - AviUtl2 beta28 での仕様変更により機能しなくなっていた．

  - `beta30a` での動作確認．

- **v1.00 (for beta21)** (2025-12-05)

  - 初版．


## ライセンス

このプログラムの利用・改変・再頒布等に関しては MIT ライセンスに従うものとします．

---

The MIT License (MIT)

Copyright (C) 2025-2026 sigma-axis

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

https://mit-license.org/


#  連絡・バグ報告

- GitHub: https://github.com/sigma-axis
- Twitter: https://x.com/sigma_axis
- nicovideo: https://www.nicovideo.jp/user/51492481
- Misskey.io: https://misskey.io/@sigma_axis
- Bluesky: https://bsky.app/profile/sigma-axis.bsky.social
