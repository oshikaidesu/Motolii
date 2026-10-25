# WhiteBalance_S AviUtl ExEdit2 スクリプト

ホワイトバランスや色温度の調整のできるアニメーション効果 / フィルタ効果を追加する AviUtl スクリプトです．

無印の AviUtl と AviUtl ExEdit2 の両方に対応しています．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_WhiteBalance_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45861771)

![ホワイトバランス調整のデモ画像](https://github.com/user-attachments/assets/74f82234-fa60-4ae8-8a6b-f4154828cc47)

- 元画像: https://www.pexels.com/photo/assorted-color-kittens-45170

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45861771`

##  動作要件

### AviUtl (無印)

- AviUtl 1.10

  http://spring-fragrance.mints.ne.jp/aviutl

- 拡張編集 0.92

- GLShaderKit

  https://github.com/karoterra/aviutl-GLShaderKit

  - `v0.4.0` / `v0.5.0` で動作確認．

### AviUtl ExEdit2

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.10` で動作確認済み．

##  導入方法

- AviUtl (無印) の場合

  以下のフォルダのいずれかに `@WhiteBalance_S.anm`, `WhiteBalance_S.lua`, `WhiteBalance_S.frag` の 4 つのファイルをコピーしてください．

  1. `exedit.auf` のあるフォルダにある `script` フォルダ
  1. (1) のフォルダにある任意の名前のフォルダ

- AviUtl ExEdit2 の場合

  ダウンロードした `aviutl2_script_WhiteBalance_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

  初期状態だと「フィルタ効果を追加」メニューの「WhiteBalance_S」以下に各種フィルタ効果が追加されています．
  - 「オブジェクト追加メニューの設定」や「トラックバー移動メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users (only AviUtl2)

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  ホワイトバランスσ

色を 2 色指定して，指定色をもう 1 つの色に変化させるようにホワイトバランスを調整します．

各色成分に定数を乗算することで色変換します．色成分の計算方式は[「色空間」や「RGB色空間」](#色空間--rgb色空間)で指定します．

### 強さ

フィルタ効果の強さを % 単位で指定します．バランス調整前の画像と連続的につなげられます．

最小値は 0, 最大値は 100, 初期値は 100.

### 変換元 / 変換先

「変換元」で指定した色が「変換先」の色になるようにホワイトバランスを調整します．

- [「成分を個別に指定」](#成分を個別に指定)が ON の場合，無視されます．

初期値は両方 `ffffff` (白).

### 正規化

[「変換元」「変換先」](#変換元--変換先)で指定した色が同じ輝度を持つように正規化します．

- [「成分を個別に指定」](#成分を個別に指定)が ON の場合，無視されます．
- 正規化に用いる輝度は [XYZ 色空間](https://ja.wikipedia.org/wiki/CIE_1931_色空間#XYZ色空間)の Y 値です．

初期値は ON.

### 成分を個別に指定

ON にすると[「倍率X/R」「倍率Y/G」「倍率Z/B」](#倍率xr-倍率yg-倍率zb)の指定が有効になり，各色成分を何倍するかの倍率を個別に指定できるようになります．

- ON にすると[「変換元」「変換先」](#変換元--変換先)や[「正規化」](#正規化)は無視されます．

初期値は OFF

### 倍率X/R, 倍率Y/G, 倍率Z/B

各色成分に乗算する倍率を % 単位で指定します．

- [「色空間」「RGB色空間」](#色空間--rgb色空間)で XYZ を指定している場合，それぞれ X, Y, Z の倍率です．
- 「色空間 / RGB色空間」が RGB を指定している場合，それぞれ R, G, B の倍率です．
- [「成分を個別に指定」](#成分を個別に指定)が ON の場合のみ有効です．

最小値は 0, 最大値は 200, 初期値は 100.

### 色空間 / RGB色空間

色変換の計算を行う座標系を指定します．[`XYZ`](https://ja.wikipedia.org/wiki/CIE_1931_色空間#XYZ色空間) と `RGB` の 2 つから選びます．

- AviUtl (無印) 版の場合は 「RGB色空間」で指定します．

  OFF だと XYZ 色空間，ON だと RGB 色空間です．

  初期値は OFF.

- AviUtl ExEdit2 版の場合は 「色空間」で指定します．

  `XYZ` と `RGB` から選びます．

  初期値は `XYZ`

### PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

####  AviUtl (無印) 版 の PI

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [0] = check0, -- boolean 型で "成分を個別に指定" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  [1] = track0, -- number 型で "強さ" の項目を上書き，または nil.
  [2] = track1, -- number 型で "倍率X/R" の項目を上書き，または nil.
  [3] = track2, -- number 型で "倍率Y/G" の項目を上書き，または nil.
  [4] = track3, -- number 型で "倍率Z/B" の項目を上書き，または nil.
}
```

####  AviUtl ExEdit2 版の PI

```lua
{
  rate = num,         -- number 型で "強さ" の項目を上書き，または nil.
  col_base = num,     -- number 型で "変換元" の項目を上書き，または nil.
  col_dest = num_tbl, -- number 型，table 型，または nil．詳細後述．
  normalize = bool,   -- boolean 型で "中心の位置を変更" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  space = str,        -- string 型で "色空間" を上書き，または nil.
}
```
- `col_dest` は number 型または table 型で以下のように解釈されます (nil の場合は無視):

  - number 型の場合，[「変換先」](#変換元--変換先)の項目を上書きします．
    - `0xRRGGBB` の形式で色を指定します．
    - このとき[「成分を個別に指定」](#成分を個別に指定)は OFF 扱いになります．

  - table 型の場合 `{ rate_XR, rate_YG, rate_ZB }` の形の指定で，[「倍率X/R」「倍率Y/G」「倍率Z/B」](#倍率xr-倍率yg-倍率zb)を上書きします．
    - 各 `rate_**` は number 型で 0 以上，または nil です．
    - このとき[「成分を個別に指定」](#成分を個別に指定)は ON 扱いになります．

- `space` に指定できる string は以下の通り:

  ```lua
  "XYZ", "RGB"
  ```

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

##  色温度σ / 色温度σ(Mired)

画像の[色温度](https://ja.wikipedia.org/wiki/色温度)を調整するようにホワイトバランスを調整します．

色温度は[ケルビン (K) 単位](https://ja.wikipedia.org/wiki/ケルビン)，または[ミレッド (M) 単位](https://ja.wikipedia.org/wiki/ミレッド)で指定します．

- AviUtl 無印版の場合，「色温度σ」はケルビン単位，「色温度σ(Mired)」はミレッド単位です．
- AviUtl ExEdit2 版の場合「色温度σ」の[「単位」](#単位)で切り替えて指定します．

  また，「色温度σ(Mired)」は存在しますが互換性のためだけに残していて非推奨です．フィルタ効果追加メニューからは追加できません．

### 変換元K / 変換先K / 変換元M / 変換先M

「変換元」で指定した温度の色を，「変換先」で指定した温度の色に変換します．「変換元」には元画像が撮影されたと想定される光源の温度を，「変換先」には別の色にその光源を置き換えた時の温度に指定すると，光源を置き換えたような変換ができます．

- 「変換元K」「変換先K」はケルビン単位で色温度を指定します．

  最小値は 1666.67, 最大値は 25000, 初期値は 6500.

  - AviUtl 無印版の場合，「色温度σ」のみにある項目です．

- 「変換元M」「変換先M」はミレッド単位で色温度を指定します．

  最小値は 40, 最大値は 600, 初期値は 154 (およそ 6500 K 相当).

  - AviUtl 無印版の場合，「色温度σ(Mired)」のみにある項目です．

### 単位

AviUtl ExEdit2 版の「色温度σ」にのみある設定項目です．色温度をケルビンとミレッドどちらの単位で指定するかを選びます．

- `K`: ケルビン単位で指定します．
- `Mired`: ミレッド単位で指定します．

初期値は `K`.

### 色空間 / RGB色空間

色変換の計算を行う座標系を指定します．`XYZ` と `RGB` の 2 つから選びます．

- AviUtl (無印) 版の場合は 「RGB色空間」で指定します．

  OFF だと XYZ 色空間，ON だと RGB 色空間です．

  初期値は OFF.

- AviUtl ExEdit2 版の場合は 「色空間」で指定します．

  `XYZ` と `RGB` から選びます．

  初期値は `XYZ`

### 色温度σ / 色温度σ(Mired) の PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

####  AviUtl (無印) 版の色温度σの PI

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [0] = check0, -- boolean 型で "RGB色空間" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  [1] = track0, -- number 型で "変換元K" の項目を上書き，または nil.
  [2] = track1, -- number 型で "変換先K" の項目を上書き，または nil.
}
```

####  AviUtl ExEdit2 版の色温度σの PI

```lua
{
  temp_base = num,  -- number 型で "変換元K" の項目を上書き，または nil.
  temp_dest = num,  -- number 型で "変換先K" の項目を上書き，または nil.
  mired_base = num, -- number 型で "変換元M" の項目を上書き，または nil.
  mired_dest = num, -- number 型で "変換先M" の項目を上書き，または nil.
  unit = str        -- string 型で "単位" を上書き，または nil.
  space = str,      -- string 型で "色空間" を上書き，または nil.
}
```
- `unit` に指定できる string は以下の通り:

  ```lua
  "K", "Mired"
  ```

- `space` の指定は[「ホワイトバランスσ」の PI](#aviutl-exedit2-版の-pi) の `space`と同様です．
- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

####  AviUtl (無印) 版の色温度σ(Mired) の PI

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [0] = check0, -- boolean 型で "RGB色空間" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  [1] = track0, -- number 型で "変換元M" の項目を上書き，または nil.
  [2] = track1, -- number 型で "変換先M" の項目を上書き，または nil.
}
```

####  AviUtl ExEdit2 版の色温度σ(Mired) の PI

```lua
{
  mired_base = num, -- number 型で "変換元M" の項目を上書き，または nil.
  mired_dest = num, -- number 型で "変換先M" の項目を上書き，または nil.
  space = str,      -- string 型で "色空間" を上書き，または nil.
}
```
- `space` の指定は[「ホワイトバランスσ」の PI](#aviutl-exedit2-版の-pi) の `space`と同様です．
- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

##  TIPS

1.  AviUtl 無印版の場合，テキストエディタで `@WhiteBalance_S.anm`, `WhiteBalance_S.lua`, `WhiteBalance_S.frag` を開くと冒頭付近にファイルバージョンが付記されています．

    ```lua
    --
    -- VERSION: v1.10
    --
    ```

    ファイル間でバージョンが異なる場合，更新漏れの可能性があるためご確認ください．

## 改版履歴

- **v1.10** (2026-09-22)

  - AviUtl2 版で，「色温度σ」に設定項目「単位」「変換元M」「変換先M」を追加．「色温度σ(Mired)」の機能を兼任するように．
  - AviUtl2 版で，「色温度σ(Mired)」を非推奨扱いに．
    - フィルタ効果の追加メニューなどからは選べません．
    - 「色温度σ」の新規パラメタ「単位」で「Mired」を選ぶことで，ミレッド単位で調整できます．
  - AviUtl2 版で，一部設定項目を条件に応じて非表示にするように．
  - AviUtl2 版で，「ホワイトバランスσ」の一部設定項目を並び替え．

- **v1.01 (for beta51)** (2026-06-21)

  - AviUtl2 版で「正規化」「成分を個別に指定」のチェックボックスを，中間点区間毎に可変なものに置き換え，マウス操作倍率の設定．

  - AviUtl2 版で初期ラベルを「色調整」から「WhiteBalance_S」に変更．

  - AviUtl2 版で配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `@WhiteBalance_S.anm2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - コード整理．
  - AviUtl2 beta51 での動作確認．

- **v1.00 (for beta29)** (2026-01-21)

  - 初版．


## ライセンス

このプログラムの利用・改変・再頒布等に関しては MIT ライセンスに従うものとします．

---

The MIT License (MIT)

Copyright (C) 2026 sigma-axis

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
