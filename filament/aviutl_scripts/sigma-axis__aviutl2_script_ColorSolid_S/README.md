# 色立体描画 AviUtl ExEdit2 スクリプト

色ピッカー / パレット代わりにも使える色立体を AivUtl ExEdit 2 で表示するスクリプトです．色調補正のサンプルなどにも利用できます．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_ColorSolid_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45695828)

次が追加されます:

1.  [XYZ色立体](#xyz色立体)
1.  [RGB色立体](#rgb色立体)
1.  [OKLCh色立体](#oklch色立体)
1.  [YUV色立体](#yuv色立体)
1.  [HSV色立体](#hsv色立体)
1.  [HSL色立体](#hsl色立体)
1.  [色ピッカー](#色ピッカー)

![Demo of 6 color solids of 6 color spaces](https://github.com/user-attachments/assets/161f5d4f-a868-4a3e-86e9-41dca899b011)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45695828`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `beta22a` で動作確認済み．

## 導入方法

`@ColorSolid_S.obj2` ファイルに対して，以下のいずれかの操作をしてください．

1.  AviUtl2 のプレビュー画面にドラッグ&ドロップ．

1.  以下のフォルダのいずれかにコピー．

    1.  スクリプトフォルダ
        - AviUtl2 のメニューの「その他」 :arrow_right: 「アプリケーションデータ」 :arrow_right: 「スクリプトフォルダ」で表示されます．
    1.  (1) のフォルダにある任意の名前のフォルダ

初期状態だと「オブジェクトを追加」メニューの「編集補助 :arrow_right: 色調整」に各種オブジェクトが追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  スクリプトの種類

いずれのスクリプトも，[「断面角度」](#断面角度)が 0° の場合を基準として，オブジェクトの中心を原点として無彩色 (黒・白・灰色など)を配置しています．

[「断面角度」](#断面角度)が 0° の場合，色立体を白と黒を結ぶ直線に垂直な断面で表示します．「断面角度」が 90° の場合，白と黒を結ぶ直線を含む断面で表示します．

- 「断面角度」の図示

  <img width="1500" height="640" alt="Image of planes crossing a color solid" src="https://github.com/user-attachments/assets/326ee12a-dca2-49cd-838c-8528bc1e752c" />

以下の色成分の配置を記述では，横の軸は右方向に正，縦の軸は上方向に正としています．

### XYZ色立体

[XYZ 色空間](https://ja.wikipedia.org/wiki/CIE_1931_色空間)での色立体を表示します．

![Demo of a color solid in XYZ space](https://github.com/user-attachments/assets/248c13ec-3845-4c8c-acee-0f73b7168958)

横軸が $X$, 縦軸が $Y$, [「明るさ」](#明るさ)は $X+Y+Z$ に対応しています．

- ただし $X$ と $Y$ は D65 の白点が中央に来るように調整されています．

### RGB色立体

[RGB 色空間](https://ja.wikipedia.org/wiki/RGB)での色立体を表示します．ガンマ補正を除いた線形 RGB での表示もできます．

![Demo of a color solid in RGB space](https://github.com/user-attachments/assets/2d1b395d-5bda-4c34-8f51-4d92a6c03a6c)

縦軸が $R$, そこから時計回りに 120° ごとに $G$, $B$, [「明るさ」](#明るさ)は $R+G+B$ に対応しています．

### OKLCh色立体

[Oklab 色空間](https://en.wikipedia.org/wiki/Oklab_color_space)での色立体を表示します．

![Demo of a color solid in OKLab space](https://github.com/user-attachments/assets/44acadd4-349a-4e50-bc9f-9dea5a883dd9)

縦軸が $a$, 横軸が $b$, [「明るさ」](#明るさ)は $L$ に対応しています．

### YUV色立体

[YUV 色空間](https://ja.wikipedia.org/wiki/YUV)での色立体を表示します．BT.601, BT.709, BT.2020 の 3 規格を選択できます．

![Demo of a color solid in YUV space](https://github.com/user-attachments/assets/3e4c621b-d279-4b2d-8622-492956b0b418)

横軸が $U$, 縦軸が $V$, [「明るさ」](#明るさ)は $Y$ に対応しています．

### HSV色立体

[HSV 色空間](https://ja.wikipedia.org/wiki/HSV色空間)での色立体を表示します．円柱モデルと円錐モデルから選べます．形状も円形と六角形の 2 種類あります．

![Demo of a color solid in HSV space](https://github.com/user-attachments/assets/ba34046c-64ea-4e09-a4a1-cc93714f32bb)

上方向に赤が，そこから時計回りに 120° ごとに緑，青と並びます．[「明るさ」](#明るさ)は $V$ に対応しています．

### HSL色立体

[HSL 色空間](https://ja.wikipedia.org/wiki/HLS色空間)での色立体を表示します．円柱モデルと双円錐モデルから選べます．形状も円形と六角形の 2 種類あります．

![Demo of a color solid in HSL space](https://github.com/user-attachments/assets/9fd803bc-4e8c-4147-b4c1-d5fb8ab50f27)

上方向に赤が，そこから時計回りに 120° ごとに緑，青と並びます．[「明るさ」](#明るさ)は $L$ に対応しています．

### 色ピッカー

色相環を表示します．外縁部に純色を，中央に無彩色を配置して，色ピッカーとして利用できます．

[「HSV色立体」](#hsv色立体)や[「HSL色立体」](#hsl色立体)に似ていますが，特に色空間として標準化されたものではありません．

外縁部や中央の大きさを自由に調整できます．

![Demo of a color picker object](https://github.com/user-attachments/assets/348365e7-d8ed-48d1-a51e-694bfec19b9d)

上方向に赤が，そこから時計回りに 120° ごとに緑，青と並びます．[「明るさ」](#明るさ)は中央の色を調整します．

##  パラメタの説明

各種オブジェクトには一部，共通した設定項目があります．

### 共通の設定項目

####  サイズ

オブジェクトのサイズをピクセル単位で指定します．

最小値は 1, 最大値は 4096, 初期値は 256.

####  明るさ

色立体のどの明るさ部分を表示するかを指定します．各種色立体をスライスする平面位置になります．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 50.

####  断面角度

色立体を表示する断面の角度を，白と黒を結ぶ直線に垂直な平面とのなす角度で指定します．

- 「断面角度」の図示

  <img width="1500" height="640" alt="Image of planes crossing a color solid" src="https://github.com/user-attachments/assets/326ee12a-dca2-49cd-838c-8528bc1e752c" />

度数単位で指定，最小値は 0, 最大値は 90, 初期値は 0.

- 水平の断面と垂直の断面の両方を表示できるようにする目的で用意したパラメタです．中途半端な角度の断面に余り深い意味はありません．

####  回転 / 色相ずれ

色立体の色相の基準値を調整します．[「断面角度」](#断面角度)が 0° の場合，ほとんどの場合は画像が回転するだけですが，一部は回転せずに色相だけがずれます．

度数単位で指定，最小値は -720, 最大値は 720, 初期値は 0.

####  RGB限定

色立体によっては RGB で表せる色範囲を超えた部分が現れることがあります．本来は規格外の色で，表示上は RGB の範囲内に収まるように調整された色になりますが，そういった色がある部分を完全透明ピクセルに置き換えます．

OFF にするとオブジェクト全体がグラデーションで埋まります．

初期値は ON.


### XYZ色立体のパラメタ

各パラメタは[共通の設定項目](#共通の設定項目)を参照．

### RGB色立体のパラメタ

####  色空間

色空間モデルを選択します．次の選択肢があります:

1.  `RGB(gamma)`

    ガンマ適用済みの数値を座標と対応させて描画します (通常の RGB).

1.  `RGB(linear)`

    ガンマ適用前の数値を座標と対応させて描画します ([sRGB](https://ja.wikipedia.org/wiki/SRGB) / 線形 RGB).

初期値は `RGB(gamma)`.

### OKLCh色立体のパラメタ

各パラメタは[共通の設定項目](#共通の設定項目)を参照．

### YUV色立体のパラメタ

####  色空間

色空間モデルを選択します．次の選択肢があります:

1.  [`BT.601`](https://ja.wikipedia.org/wiki/Rec._601)

    - 標準のフィルタ効果「色調補正」などに用いられている色空間です．

1.  [`BT.709`](https://ja.wikipedia.org/wiki/Rec._709)
1.  [`BT.2020`](https://ja.wikipedia.org/wiki/Rec._2020)

初期値は `BT.601`.

### HSV色立体のパラメタ

####  モデル

色空間モデルを選択します．次の選択肢があります:

1.  `円柱`

    S (彩度) の範囲は V (明るさ) に依存せず常に 0% ～ 100% をとります．

1.  `円錐`

    S (彩度) の範囲は V (明るさ) に依存します．

1.  `六角柱`

    `円柱` モデルを六角柱に変形したものです．

1.  `六角錐`

    `円錐` モデルを六角錐に変形したものです．

初期値は `円柱`.

### HSL色立体のパラメタ

####  モデル

色空間モデルを選択します．次の選択肢があります:

1.  `円柱`

    S (彩度) の範囲は L (明るさ) に依存せず常に 0% ～ 100% をとります．

1.  `双円錐`

    S (彩度) の範囲は L (明るさ) に依存します．

1.  `六角柱`

    `円柱` モデルを六角柱に変形したものです．

1.  `双六角錐`

    `双円錐` モデルを六角錐に変形したものです．

初期値は `円柱`.

### 色ピッカーのパラメタ

####  純色幅 / 無彩色幅

色相環の外縁の純色部分の幅や，中央の無彩色部分の半径を調整できます．% 単位で指定，一方が 100% だと「無彩色 :arrow_right: 純色」のグラデーション部分は表示されません．

色ピッカーとして使う場合，大きくすると純色や無彩色を拾いやすくなります．

最小値は 0, 最大値は 100, 初期値は 0.

####  モデル

色相環の形状を `円`, `六角形` から選びます．

初期値は `円`.

##  改版履歴

- **v1.00 (for beta22a)** (2025-12-05)

  - 初版．


## ライセンス

このプログラムの利用・改変・再頒布等に関しては MIT ライセンスに従うものとします．

---

The MIT License (MIT)

Copyright (C) 2025 sigma-axis

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
