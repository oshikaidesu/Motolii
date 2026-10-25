# PageRoll_S AviUtl/AviUtl ExEdit2 スクリプト

ページを丸めるように画像を変形するアニメーション効果と，シーンチェンジです．AviUtl (無印) のシーンチェンジ「ロール(横)」「ロール(縦)」の拡張版・AviUtl2 移植版で，めくる方向を斜めにしたり，丸める部分の太さの調整などの追加機能があります．

無印の AviUtl と AviUtl ExEdit2 の両方に対応しています．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_PageRoll_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45441228)

https://github.com/user-attachments/assets/ee82a50a-59bc-4256-9163-d843ca322a18

https://github.com/user-attachments/assets/b83fc0ba-7e1a-4ee5-b033-b4d8879243a4

- <details>
  <summary>元画像出典 (クリックで表示):</summary>

  1.  https://www.pexels.com/photo/assorted-color-kittens-45170
  1.  https://www.pexels.com/photo/four-ace-game-cards-534181
  1.  https://www.pexels.com/photo/green-leafed-tree-beside-body-of-water-during-daytime-158063
  1.  https://www.pexels.com/photo/vegetables-and-tomatoes-on-cutting-board-255501
  1.  https://www.pexels.com/photo/assorted-color-house-facade-in-park-534124
  1.  https://www.pexels.com/photo/brown-wooden-signage-hanging-beside-wall-434446
  1.  https://www.pexels.com/photo/assorted-car-license-plates-533669
  </details>

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45441228`

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

  - `2.1.8` で動作確認済み．

## 導入方法

- AviUtl (無印) の場合

  以下のフォルダのいずれかに `PageRoll_S.anm`, `PageRoll_S.scn`, `PageRoll_S.lua`, `PageRoll_S.frag` の 4 つのファイルをコピーしてください．

  1. `exedit.auf` のあるフォルダにある `script` フォルダ
  1. (1) のフォルダにある任意の名前のフォルダ

- AviUtl ExEdit2 の場合

  ダウンロードした `aviutl2_script_PageRoll_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  スクリプトの種類

フィルタ効果 (アニメーション効果) とシーンチェンジが 1 つずつ追加されます．

- AviUtl2 の場合，追加メニュー内の分類は「オブジェクト追加メニューの設定」の「ラベル」項目で変更できます．

### フィルタ効果 (アニメーション効果)

`PageRoll_S` という名前のフィルタ効果 (アニメーション効果)です．画像単体にページを丸めたような変形を適用します．

https://github.com/user-attachments/assets/b83fc0ba-7e1a-4ee5-b033-b4d8879243a4

- AviUtl2 の場合，初期状態だと「フィルタ効果を追加」メニューの「変形」に「PageRoll_S」が追加されています．

### シーンチェンジ

AviUtl (無印) だと `PageRoll_S` という名前，AviUtl2 だと `PageRoll_S(シーンチェンジ)` の名前のシーンチェンジ効果です．シーン全体をページを丸めるように動かして次のシーンに移行します．

https://github.com/user-attachments/assets/ee82a50a-59bc-4256-9163-d843ca322a18

- AviUtl2 の場合，初期状態だと「オブジェクトを追加」メニューの「シーンチェンジ」に「PageRoll_S(シーンチェンジ)」が追加されています．
- AviUtl (無印) と AviUtl2 で名前が違うのは，AviUtl2 だとスクリプトの分類がフィルタ効果とシーンチェンジで違っていても，名前が重複できない仕様のためです．

##  パラメタの説明

AviUtl (無印) 版では AviUtl ExEdit2 版と比べてパラメタの並びが異なっていたり，一部がパラメタ設定ダイアログ経由での設定になりますが，特記事項がない限り基本的には同じ機能です．

また，いくつかのパラメタは[フィルタ効果](#フィルタ効果-アニメーション効果)と[シーンチェンジ](#シーンチェンジ)で共通しています．

### 共通の設定項目

####  角度

ページをめくる方向を指定します．0 で真下から，時計回りに正です．

度数法で指定，初期値は -90.

- AviUtl (無印) 版の場合，最小値は -720, 最大値は 720.
- AviUtl2 版の場合，最小値は -3600, 最大値は 3600.

####  太さ

丸めたページの，円柱としての直径が変化します．

1.  [フィルタ効果](#フィルタ効果-アニメーション効果)の場合:

    丸めた円柱の画面上の幅 (直径に [「視野角」](#視野角) の影響を加味した大きさ) をピクセル単位で指定します．

    最小値は 8, 最大値は 4000, 初期値は 80.

1.  [シーンチェンジ](#シーンチェンジ)の場合:

    シーンの対角線の長さに対する比で指定．ただし最小でも画面上で 8 ピクセル以上になります．

    % 単位で指定，最小値は 2, 最大値は 150, 初期値は 20.

####  視点 / 視点X / 視点Y

丸めたページの立体感の基準となる視点の位置を指定します．シーンの中央からの相対位置で，ピクセル単位で指定します．マウスによるアンカー操作でも指定できます．

- AviUtl (無印) の場合

  「視点」がパラメタ設定ダイアログで指定できます．

  `{ X座標, Y座標 }` の形式で指定，初期値は `{0,0}`.

  - [シーンチェンジ](#シーンチェンジ)の場合はアンカーが無効です．(そもそも非対応?)

- AviUtl ExEdit2 の場合

  「視点X」と「視点Y」がトラックバーで指定できます．

  X, Y 座標ともに，最小値は -4000, 最大値は 4000, 初期値は 0.

####  視野角

丸めたページの立体感の基準となる視野角を指定します．ここでの視野角は，オブジェクトやシーンの対角線の視程角度です．

大きくすると遠近感が強くなり，0 を指定すると [orthographic projection (正射影)](https://en.wikipedia.org/wiki/Orthographic_projection) になります．

度数法で指定，最小値は 0, 最大値は 120, 初期値は 70.

####  陰影

丸めたページの縁部分を影がかかったように暗く描画します．

強さを % 単位で指定，最小値は 0, 最大値は 100, 初期値は 50.

####  裏地画像

丸めたページの裏側に画像ファイルを設定することができます．未指定の場合は丸めた画像をそのまま使用します．

- フィルタ効果 (アニメーション効果) では，[「裏地種類」](#裏地種類)に `元画像/画像ファイル` を選んでいる場合のみ有効です．

- シーンチェンジでは，[「裏地」](#裏地)に `指定画像` を選んでいる場合のみ有効です．

設定方法は以下の通り:

- AviUtl (無印) の場合

  `[[C:\images\back.png]]` の形式で画像ファイルのパスを指定します．

  - パスは `[[...]]` で囲ってください．囲っていない場合，[一部の文字](https://sites.google.com/site/fudist/Home/grep/sjis-damemoji-jp)でパスの解釈が正しくできません．
  - 冒頭末尾前後にダブルクォート `"` があっても無視されます (`[["C:\images\back.png"]]` でも OK).

  - 空の文字列 (`[[]]`) を指定すると，未指定扱いになります．


- AviUtl ExEdit2 の場合

  ファイル選択ダイアログか，ファイルのドラッグ & ドロップで画像ファイルを選んでください．

初期値は未指定．

####  キャッシュ名

丸めたページの裏側に一時キャッシュからの画像を設定することができます．`"cache:***"` の後半部分の文字列を指定します．

- フィルタ効果 (アニメーション効果) では，[「裏地種類」](#裏地種類)に `一時キャッシュ` を選んでいる場合のみ有効です．

- シーンチェンジでは，[「裏地」](#裏地)に `一時キャッシュ` を選んでいる場合のみ有効です．

初期値は `my_cache` (`"cache:my_cache"` に対応).

####  裏地向き

丸めたページの裏側の，画像の向きを上下左右反転できます．

- AviUtl (無印) の場合

  `0` から `3` の整数を指定します．数値と指定の対応は以下の通り:

  | 数値 | 指定 |
  |:---:|:---:|
  | `0` | `通常` |
  | `1` | `左右反転` |
  | `2` | `上下反転` |
  | `3` | `180°反転` |

- AviUtl ExEdit2 の場合

  `通常`, `左右反転`, `上下反転`, `180°反転` の 4 つから選びます．

初期値は `通常`.

### フィルタ効果 (アニメーション効果) の設定項目

####  距離

「丸めて転がした」距離を指定します．

ピクセル単位で指定，最小値は 0, 最大値は 4000, 初期値は 0.

####  領域外も描画

オブジェクトの画像サイズを拡げて，丸まった部分がサイズ内に収まるようにします．

- AviUtl2 版の場合でフィルタオブジェクトとして使用した場合，無視されます (常に OFF 相当).

初期値は ON.

####  裏地種類

丸めたページの裏側の画像を指定します．

- AviUtl (無印) の場合

  `1` から `4` の整数を指定します．数値と指定の対応は以下の通り:

  | 数値 | 指定 |
  |:---:|:---:|
  | `1` | `元画像/画像ファイル` |
  | `2` | `フレームバッファ` |
  | `3` | `仮想バッファ` |
  | `4` | `一時キャッシュ` |

- AviUtl ExEdit2 の場合

  `元画像/画像ファイル`, `フレームバッファ`, `仮想バッファ`, `一時キャッシュ` の 4 つから選びます．

指定の意味は以下の通りです:

| 指定 | 意味 |
|:---:|:---|
| `元画像/画像ファイル` | 丸められる画像，または指定がある場合[「裏地画像」](#裏地画像)の画像． |
| `仮想バッファ` | 現在の仮想バッファの内容． |
| `一時キャッシュ` | スクリプト内部で扱われる，`"cache:***"` の形式の一時画像． |

- `仮想バッファ` は他スクリプトで頻繁に書き換えられるため，退避・復元の処理が必要なことがあります．

  - AviUtl2 版なら[Basic_S の「バッファをコピー」](https://github.com/sigma-axis/aviutl2_script_Basic_S/wiki/バッファをコピー)が利用できます．

- `一時キャッシュ` を選んだ場合，[「キャッシュ名」](#キャッシュ名)でキャッシュの名前を指定します．

初期値は `元画像/画像ファイル`.

### シーンチェンジの設定項目

####  裏地

丸めたページの裏側の画像を指定します．

- AviUtl (無印) の場合

  `0` から `4` の整数を指定します．数値と指定の対応は以下の通り:

  | 数値 | 指定 |
  |:---:|:---:|
  | `0` | `上側` |
  | `1` | `下側` |
  | `2` | `指定画像` |
  | `3` | `仮想バッファ` |
  | `4` | `一時キャッシュ` |

- AviUtl ExEdit2 の場合

  `上側`, `下側`, `指定画像`, `仮想バッファ`, `一時キャッシュ` の 5 つから選びます．

指定の意味は以下の通りです:

| 指定 | 意味 |
|:---:|:---|
| `上側` | 丸められる側のシーンの画像． |
| `下側` | 丸められない側のシーンの画像． |
| `指定画像` | [「裏地画像」](#裏地画像) で指定した画像． |
| `仮想バッファ` | 現在の仮想バッファの内容． |
| `一時キャッシュ` | スクリプト内部で扱われる，`"cache:***"` の形式の一時画像． |

- `仮想バッファ` は他スクリプトで頻繁に書き換えられるため，退避・復元の処理が必要なことがあります．

  - AviUtl2 版なら[Basic_S の「バッファをコピー」](https://github.com/sigma-axis/aviutl2_script_Basic_S/wiki/バッファをコピー)が利用できます．

- `一時キャッシュ` を選んだ場合，[「キャッシュ名」](#キャッシュ名)でキャッシュの名前を指定します．

初期値は `上側`.

####  反転

AviUtl ExEdit2 版にのみあります．AviUtl (無印) のシーンチェンジ効果にある「反転」と同じ機能のチェックボックスです．

初期値は OFF.

### PI

パラメタインジェクション (parameter injection) です．各種パラメタを Lua の数式で直接指定できます．また，任意のスクリプトコードを実行する記述領域にもなります．`PI` 経由でのみ指定できる設定もあります．

[フィルタ効果](#フィルタ効果-アニメーション効果)と[シーンチェンジ](#シーンチェンジ)，AviUtl (無印) 版と AviUtl ExEdit2 版で指定方法が異なります．

####  アニメーション効果の `PI` (AviUtl (無印) 版)

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [0] = check0, -- boolean 型で "領域外も描画" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  [1] = track0, -- number 型で "距離" の項目を上書き，または nil.
  [2] = track1, -- number 型で "角度" の項目を上書き，または nil.
  [3] = track2, -- number 型で "太さ" の項目を上書き，または nil.
  [4] = track3, -- number 型で "陰影" の項目を上書き，または nil.
}
```

####  フィルタ効果の `PI` (AviUtl ExEdit2 版)

初期値は空欄．テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．

```lua
{
  distance = num,           -- number 型で "距離" の項目を上書き，または nil.
  angle = num,              -- number 型で "角度" の項目を上書き，または nil.
  width = num,              -- number 型で "太さ" の項目を上書き，または nil.
  X = num,                  -- number 型で "視点X" の項目を上書き，または nil.
  Y = num,                  -- number 型で "視点Y" の項目を上書き，または nil.
  fov = num,                -- number 型で "視野角" の項目を上書き，または nil.
  shadow = num,             -- number 型で "陰影" の項目を上書き，または nil.
  unbound = bool,           -- boolean 型で "領域外も描画" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  backface = str,           -- string 型で "裏地種類" の項目を上書き，または nil.
  file_image = str,         -- string 型で "裏地画像" の項目を上書き，または nil.
  cache_name = str,         -- string 型で "キャッシュ名" の項目を上書き，または nil.
  back_orient = num_or_str, -- number 型または string 型で "裏地向き" の項目を上書き，または nil.
}
```
- `backface` に指定できる string は以下の通り:

  ```lua
  "元画像", "元画像/画像ファイル", "フレームバッファ", "仮想バッファ", "一時キャッシュ"
  ```

  - v1.12 (for beta25) 以前のように数値でも指定ができますが，互換対応です．

- `back_orient` に指定できる string は以下の通り:

  ```lua
  "通常", "左右反転", "上下反転", "180°反転"
  ```

  number 型との対応は，AviUtl (無印) 版の[「裏地向き」](#裏地向き)と同じです．

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

####  シーンチェンジの `PI` (AviUtl (無印) 版)

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [1] = track0,  -- number 型で "角度" の項目を上書き，または nil.
  [2] = track1,  -- number 型で "太さ" の項目を上書き，または nil.
  phase = phase, -- number 型でシーンチェンジの進捗を直接指定 (0.0 -- 1.0)，または nil.
}
```

####  シーンチェンジの `PI` (AviUtl ExEdit2 版)

初期値は空欄．テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．

```lua
{
  angle = num,              -- number 型で "角度" の項目を上書き，または nil.
  width = num,              -- number 型で "太さ" の項目を上書き，または nil.
  X = num,                  -- number 型で "視点X" の項目を上書き，または nil.
  Y = num,                  -- number 型で "視点Y" の項目を上書き，または nil.
  fov = num,                -- number 型で "視野角" の項目を上書き，または nil.
  shadow = num,             -- number 型で "陰影" の項目を上書き，または nil.
  backface = num_or_str,    -- number 型または string 型で "裏地" の項目を上書き，または nil.
  file_image = str,         -- string 型で "裏地画像" の項目を上書き，または nil.
  cache_name = str,         -- string 型で "キャッシュ名" の項目を上書き，または nil.
  back_orient = num_or_str, -- number 型または string 型で "裏地向き" の項目を上書き，または nil.
  reverse = bool,           -- boolean 型で "反転" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  phase = num,              -- number 型でシーンチェンジの進捗を直接指定 (0.0 -- 1.0)，または nil.
}
```
- `backface` に指定できる string は以下の通り:

  ```lua
  "上側", "下側", "指定画像", "仮想バッファ", "一時キャッシュ"
  ```

  number 型との対応は，AviUtl (無印) 版の[「裏地画像」](#裏地画像)と同じです．

- `back_orient` に指定できる string は以下の通り:

  ```lua
  "通常", "左右反転", "上下反転", "180°反転"
  ```

  number 型との対応は，AviUtl (無印) 版の[「裏地向き」](#裏地向き)と同じです．

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

##  TIPS

1.  AviUtl 無印版の場合，テキストエディタで `PageRoll_S.anm`, `PageRoll_S.scn`, `PageRoll_S.lua`, `PageRoll_S.frag` を開くと冒頭付近にファイルバージョンが付記されています．

    ```lua
    --
    -- VERSION: v1.21
    --
    ```

    ファイル間でバージョンが異なる場合，更新漏れの可能性があるためご確認ください．

1.  AviUtl (無印) のシーンチェンジ「ロール(横)」「ロール(縦)」の，[「太さ」](#太さ) に対応する値の計算方法は正確には不明ですが， $1920 \times 1080$ のシーンで確認したところ，概ね画面の横幅 / 縦幅の 25% 程度でした．シーンの対角線との比だと以下の通りです．

    - ロール(横) :arrow_right: 約 21.79% (480 ピクセル).
    - ロール(縦) :arrow_right: 約 12.26% (270 ピクセル).


##  改版履歴

- **v1.22** (2026-09-06)

  - AviUtl2 版で，ツールチップ項目が抜けていたのを追加．

- **v1.21** (2026-09-06)

  - AviUtl2 版で，一部設定項目を条件によって非表示にするように．
  - AviUtl2 版で，一部設定項目にツールチップを追加．

- **v1.20 (for beta51)** (2026-06-24)

  - 裏地にフレームバッファや仮想バッファ，一時キャッシュを指定できるように．

    - フィルタ効果・アニメーション効果では「裏地種類」の設定を追加．
    - シーンチェンジでは「裏地」の設定に選択項目を追加．
    - 両方に「キャッシュ名」の設定を追加．

  - AviUtl2 版で，チェックボックスを中間点ごとに可変なタイプに置き換え，トラックバーのマウス操作倍率の設定など GUI の調整．

  - 仮想バッファを上書きしないように変更．

  - AviUtl2 版で「拡大率」フィルタなど一部の効果がリセットされていたのを修正．

  - AviUtl2 版で配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `PageRoll_S.anm2`
      1.  `PageRoll_S(シーンチェンジ).scn2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - `beta51` で動作確認．

- **v1.12 (for beta25)** (2025-12-22)

  - AviUtl2 版のフィルタ効果をフィルタオブジェクトとして使えるよう設定．
  - `beta25` で動作確認．

- **v1.11 (for beta22a)** (2025-12-06)

  - AviUtl2 版でパラメタをグループ化して整理．
  - AviUtl2 版でパラメタインジェクションの一部に文字列型を受け付けるように．
  - `beta22a` で動作確認．

- **v1.10 (for beta13)** (2025-10-04)

  - 丸めたページの「裏地」の画像や向きを指定できるように．
  - `beta13` で動作確認．

- **v1.00 (for beta12)** (2025-09-25)

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
