# ColoredCollar_S AviUtl ExEdit2 スクリプト

オブジェクトの境界部分の色を引き延ばして縁取りする AviUtl ExEdit 2 用のスクリプトです．縁取り部分は半透明にしたり，指定色で色を付けたり，距離に応じてグラデーションさせられます．標準の「縁取り」よりも高速に動作します．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_ColoredCollar_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45257124)

![使用例1](https://github.com/user-attachments/assets/62234847-ed86-41fc-a50a-f01cc4077356)

![使用例2](https://github.com/user-attachments/assets/3056ac42-c983-42b8-b243-fb632590a201)

- イラスト: 琴葉茜 琴葉葵 (c) AI Inc.

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45257124`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.10` で動作確認済み．

##  導入方法

ダウンロードした `aviutl2_script_ColoredCollar_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

初期状態だと「フィルタ効果を追加」メニューの「装飾」に ColoredCollar_S が追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  パラメタの説明

### サイズ

縁取りの幅をピクセル単位で指定します．

最小値は 0, 最大値は 500, 初期値は 5.

### ぼかし

縁取りの透明度を，外側からの距離に応じて変化させます．変化幅を[「サイズ」](#サイズ)からの比として % 単位で指定．

最小値は 0, 最大値は 100, 初期値は 5.

### 色拡散

元画像の外側の色の付いた縁取り部分を，元画像からの距離に応じてぼかします．ぼかしの程度を % 単位で指定．

最小値は 0, 最大値は 100, 初期値は 5.

### αしきい値

元画像の各ピクセルを不透明とみなすアルファ値のしきい値を，% 単位で指定します．

最小値は 0, 最大値は 100, 初期値は 50.

### サイズ固定

元画像のサイズを超えた部分をクリッピングして，サイズを変えないようにします．

- フィルタオブジェクトでは常に ON 相当の挙動になります．

初期値は OFF.

### 縁色

縁部分に付ける色を指定します．着色の強さは [「色の濃さ」](#色の濃さ)で調整できます．

[「縁色外側」](#縁色外側)を指定している場合は，グラデーションの内側の色です．

初期値は `ffffff` (白).

### 縁色外側

[「縁色」](#縁色)縁部分につける色で外側の色を指定します．「縁色」から「縁色外側」に距離に応じて変化するグラデーションが付きます．

未指定の場合は「縁色」の単色になります．

初期値は未指定．

### 色の濃さ

[「縁色」](#縁色)または[「縁色外側」](#縁色外側)で指定した色を着色する強さを指定します．% 単位で指定，大きくすれば指定色に，小さくすれば元画像の色に近くなります．

- スクリプトや参照式から呼び出す際は「縁色」に対応するものは `"色の濃さ"`, 「縁色外側」に対応するものは `"outer::色の濃さ"` で指定してください．

最小値は 0, 最大値は 100, 初期値は 30.

### 透明度

縁部分を半透明にできます．% 単位で指定，大きくすれば透明に，小さくすれば不透明になります．

最小値は 0, 最大値は 100, 初期値は 0.

### 前景透明度

縁取り元のオブジェクトを半透明にできます．% 単位で指定，大きくすれば透明に，小さくすれば不透明になります．

最小値は 0, 最大値は 100, 初期値は 0.

### 錯視軽減

[「ぼかし」](#ぼかし)や[「縁色外側」](#縁色外側)での距離グラデーションで，切り込んだ谷のように見える錯視的な模様を軽減します．

![錯視軽減の適用例](https://github.com/user-attachments/assets/879b5b97-a74c-4bf3-8a9d-0beb9afd2a3e)

距離のスカラー場にぼかし処理をする方法で，そのぼかし幅を[「サイズ」](#サイズ)からの比で % 単位で指定します．

最小値は 0, 最大値は 100, 初期値は 0.

### PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

```lua
{
  thick = num,           -- number 型で "サイズ" の項目を上書き，または nil.
  blur = num,            -- number 型で "ぼかし" の項目を上書き，または nil.
  col_blur = num,        -- number 型で "色拡散" の項目を上書き，または nil.
  threshold = num,       -- number 型で "αしきい値" の項目を上書き，または nil.
  fixed_size = num,      -- boolean 型で "サイズ固定" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  color = num,           -- number 型で "縁色" の項目を上書き，または nil.
  col_alpha = num,       -- number 型で "色の濃さ" の項目を上書き，または nil.
  color_outer = num,     -- number 型で "縁色外側" の項目を上書き，false で未指定に，または nil.
  col_alpha_outer = num, -- number 型で "outer::色の濃さ" の項目を上書き，または nil.
  alpha = num,           -- number 型で "透明度" の項目を上書き，または nil.
  front_alpha = num,     -- number 型で "前景透明度" の項目を上書き，または nil.
  mollify = num,         -- number 型で "錯視軽減" の項目を上書き，または nil.
}
```
- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

##  TIPS

1.  [「ぼかし」](#ぼかし)は 0 だと境界がジャギーになりますが，0.01 など 0 より大きければ，最低限の 1 ピクセル幅のアンチエイリアスがかかります．

##  改版履歴

- **v2.11** (2026-09-21)

  - 一部ツールチップの変更．

- **v2.10** (2026-09-21)

  - 一部設定項目にツールチップを表示するように．
  - 一部設定項目を条件に応じて非表示にするように．

- **v2.00 (for beta51)** (2026-06-26)

  - 大幅刷新．

    - 以前の版とは見え方が異なりますが，各種パラメタは引き継がれます．特に縁取りサイズが大きい場合，外側付近の見え方が大きく異なります．

    - 縁取りの色を，厳密に最も近いピクセルの色から取得するように．

  - 動作を高速化 (「色拡散」が `0` の場合で， $O(WH \log(\max\\{W,H\\}))$ の計算量).
  - 「縁色外側」のパラメタ追加，縁取りの距離に応じたグラデーションができるように．
  - 「αしきい値」「錯視軽減」のパラメタ追加，[「縁取りT」](http://www.nicovideo.jp/watch/sm33598259)の「α基準」「錯覚補正」と類似の挙動．
  - 「色拡散」を追加，距離に応じて引き伸ばした色のぼかし量が変化．
  - 「サイズ固定」で外側の色だけを引き伸ばす効果ができるように．
  - フィルタオブジェクトに対応．

  - AviUtl2 版で配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `ColoredCollar_S.anm2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - `beta51` での動作確認．

- **v1.00 (for beta5)** (2025-08-04)

  - 初版．


##  ライセンス

このプログラムの利用・改変・再頒布等に関しては MIT ライセンスに従うものとします．

---

The MIT License (MIT)

Copyright (C) 2025-2026 sigma-axis

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

https://mit-license.org/


# 連絡・バグ報告

- GitHub: https://github.com/sigma-axis
- Twitter: https://x.com/sigma_axis
- nicovideo: https://www.nicovideo.jp/user/51492481
- Misskey.io: https://misskey.io/@sigma_axis
- Bluesky: https://bsky.app/profile/sigma-axis.bsky.social
