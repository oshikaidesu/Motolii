# Tile_S AviUtl ExEdit2 タイルパターン生成スクリプト

規則的なパターン模様を生成するスクリプトです．動画の背景素材などとして利用できます．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_Tile_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45311665)

次のオブジェクトが追加されます:

1.  [四角タイルσ](#四角タイルσ)
1.  [三角タイルσ](#三角タイルσ)
1.  [六角タイルσ](#六角タイルσ)
1.  [菱形タイルσ](#菱形タイルσ)

![使用例](https://github.com/user-attachments/assets/a20a779d-907e-40f6-b4e8-265c412ce0a4)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45311665`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.8` で動作確認済み．

## 導入方法

ダウンロードした `aviutl2_script_Tile_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

初期状態だと「オブジェクトを追加」メニューの「Tile_S」に「～タイルσ@Tile_S」が追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  オブジェクトの種類

4種類のオブジェクトが追加されます．

### 四角タイルσ

長方形を並べたパターン模様です．4色に塗り分けられます．

![四角タイルの例](https://github.com/user-attachments/assets/07fbeb1a-d086-4eaf-aeb4-772b6d7d5d1a)

### 三角タイルσ

正三角形を並べたパターン模様です．6色に塗り分けられます．

![三角タイルの例](https://github.com/user-attachments/assets/a0023573-6f0d-43cf-9020-a6ed52144a96)

### 六角タイルσ

正六角形を並べたパターン模様です．3色に塗り分けられます．

![六角タイルの例](https://github.com/user-attachments/assets/67d5d69e-c55b-407a-8c5c-9ace0ee2eb1a)

### 菱形タイルσ

菱形を並べたパターン模様です．4色に塗り分けられます．

![菱形タイルの例](https://github.com/user-attachments/assets/01fc8a3f-5d77-495d-8a65-8fc8d2c303c3)

##  パラメタの説明

各オブジェクトごとにパラメタの名前や効果，初期値が微妙に違いますが，基本的な部分は同じです．

### 幅, 高さ, 背景サイズ

オブジェクトのサイズを指定します．「背景サイズ」が ON の場合は，シーンのサイズと同じになり，「幅」と「高さ」は無視されます．

「幅」と「高さ」はピクセル単位で指定，最小値は 0, 最大値は 4000, 初期値は 320.

「背景サイズ」の初期値は OFF.

### タイル種類

タイルの色や透明度，ライン幅や余白などは，スクリプトによって 3 -- 6 種類に分けられていますが，この種類をもっと少ない種類に制限します．

![種類数を制限したタイルの例](https://github.com/user-attachments/assets/35ba68bd-e4bf-4cdf-ba31-0036a7922175)

- 制限した場合，数字の小さいタイル設定のみが使われるようになります．

  例えば `3種類` に制限した場合，[「色1」「色2」「色3」の設定](#色1-色2-)でそれぞれ「色4」「色5」「色6」を兼用するようになり，「色4」「色5」「色6」の設定は無視されます．

例えば[「四角タイルσ」](#四角タイルσ)で「2色」を指定すると[市松模様](https://ja.wikipedia.org/wiki/市松模様)になるなど，簡単なパターンなら少ない設定で済むようになります．

初期値は，そのスクリプトでの最大の種類数 (スクリプトによって異なる).

### 色1, 色2, ...

タイル部分の色を，各タイルでそれぞれ指定します．[「ライン幅」](#ライン幅)が指定されている場合は，縁部分の色です．

指定できる色の数は以下の通りです:

| スクリプト名 | 色の数 |
|:---:|:---:|
| [四角タイルσ](#四角タイルσ) | 4 |
| [三角タイルσ](#三角タイルσ) | 6 |
| [六角タイルσ](#六角タイルσ) | 3 |
| [菱形タイルσ](#菱形タイルσ) | 4 |

色の順序は基本的には，[「X」「Y」](#x-y)で指定した位置の真上から時計回りに並んでいます．

### 内部色1, 内部色2, ...

タイル部分の内側の色を，各タイルでそれぞれ指定します．[「ライン幅」](#ライン幅)が指定されている場合の，縁部分より内側の色です．

- 初期状態だと「ライン幅」でタイル全体が覆われているためこの設定は反映されません．「ライン幅」を[「マス幅」](#マス幅-マス高さ)に比べて小さい値を指定してください．

### 透明度1, 透明度2, ...

タイル部分の透明度を，各タイルでそれぞれ指定します．透明度を 100 にすると「穴」が空いたような画像になります．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 0.

### マス幅, マス高さ, マス正方形(幅のみで指定)

各タイルのサイズを指定します．

- 「マス高さ」は[四角タイルσ](#四角タイルσ)と[菱形タイルσ](#菱形タイルσ)のみ．
- 「マス正方形(幅のみで指定)」は[四角タイルσ](#四角タイルσ)のみ．

正確な意味は以下の通り:

| スクリプト名 | マス幅 | マス高さ |
|:---:|:---|:---|
| [四角タイルσ](#四角タイルσ) | 長方形の横の辺の長さ | 長方形の縦の辺の長さ |
| [三角タイルσ](#三角タイルσ) | 正三角形の1辺の長さ | - |
| [六角タイルσ](#六角タイルσ) | 正六角形の最長の対角線の長さ | - |
| [菱形タイルσ](#菱形タイルσ) | 菱形の横の対角線の長さ | 菱形の縦の対角線の長さ |

「マス正方形(幅のみで指定)」が ON の場合，「マス高さ」が無視され，代わりに「マス幅」が適用されます．

「マス幅」と「マス高さ」はピクセル単位で指定，最小値は 1, 最大値は 2000, 初期値は原則 64 ([菱形タイルσ](#菱形タイルσ)の「マス幅」のみ 96).

「マス正方形(幅のみで指定)」の初期値は ON.

### ライン幅

各タイル部分の縁部分のサイズを指定します．縁部分の色は[「色1」や「色2」など](#色1-色2-)で，内部の色は[「内部色1」や「内部色2」など](#内部色1-内部色2-)で指定します．

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

ピクセル単位で指定，最小値は 0, 最大値は 1000, 初期値は 1000.

### 角半径

各タイル部分の角部分を丸めます．丸める場合の曲率半径を指定します．

- 各タイルが真円になる半径が上限で，これ以上は真円のままです．

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

ピクセル単位で指定，最小値は 0, 最大値は 1000, 初期値は 0.

### 余白色

[「角半径」](#角半径)や[「余白幅」](#余白幅-余白高さ)を指定したときに見える，各タイル間の「隙間」の色を指定します．

初期値は `000000` (黒).

### 余白透明度

[「角半径」](#角半径)や[「余白幅」](#余白幅-余白高さ)を指定したときに見える，各タイル間の “隙間” の透明度を指定します．

- 各タイルの [「透明度1」など](#透明度1-透明度2-)が高い場合，そこには「余白色」が見えるのではなく，“穴” が空いたような画像になります．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 100.

### 余白幅, 余白高さ

各タイル間の “隙間” のサイズを指定します．余白部分の色や透明度は[「余白色」](#余白色)と[「余白透明度」](#余白透明度)で指定します．

- 「余白高さ」は[四角タイルσ](#四角タイルσ)のみ．

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

ピクセル単位で指定，最小値は 0, 最大値は 2000, 初期値は 0.

### 回転

タイル模様全体を指定した角度だけ回転できます．回転中心は[「X」「Y」](#x-y)で指定します．

単位は度数法，最小値は -3600, 最大値は 3600, 初期値は 0.

### X, Y

タイル模様全体を平行移動できます．プレビュー画面のアンカー操作でも設定できます．

ピクセル単位で指定，最小値は -4000, 最大値は 4000, 初期値は 0.

### アンチエイリアス

タイル模様の描画にアンチエイリアスを適用します．[「角半径」](#角半径)や[「回転」](#回転)を適用した際にジャギーさが軽減されます．

初期値は，[四角タイルσ](#四角タイルσ)の場合は OFF, それ以外で ON.

### `PI`

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

[「ライン幅」](#ライン幅)，[「角半径」](#角半径)，[「余白幅」「余白高さ」](#余白幅-余白高さ)に関しては，タイルごとに個別の設定ができるようにもなります．

- 各タイルごとに指定可能な設定値については，次の記述ができます．

  1.  `line = 3.5` (number 型を指定)

      「ライン幅」をタイル全体で一律に指定．

  1.  `line = { 1.5, 2.5, 3.5, 4.5 }` (table 型で配列を指定)

      「ライン幅」を，1番目のタイルが 1.5, 2番目が 2.5, ... と，個別に指定．

  この指定が可能なフィールドには，以下の形式のコメントを記述してあります．
  ```lua
  -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  ```

`PI` のテキストボックスの記述では，下記における冒頭末尾の波括弧 (`{}`) は省略してください．

####  四角タイルσ の `PI`
```lua
{
  width = num,         -- number 型で "幅" の項目を上書き，または nil.
  height = num,        -- number 型で "高さ" の項目を上書き，または nil.
  screen_size = bool,  -- boolean 型で "背景サイズ" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  limit_tiles = str,   -- string 型で "タイル種類" の項目を上書き，または nil.
  col = num,           -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_inner = num,     -- number 型で "内部色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  alpha = num,         -- number 型で "透明度*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  block_w = num,       -- number 型で "マス幅" の項目を上書き，または nil.
  block_h = num,       -- number 型で "マス高さ" の項目を上書き，または nil.
  block_square = bool, -- boolean 型で "マス正方形(幅のみで指定)" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  line = num,          -- number 型で "ライン幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  radius = num,        -- number 型で "角半径" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_back = num,      -- number 型で "余白色" の項目を上書き，または nil.
  alpha_back = num,    -- number 型で "余白透明度" の項目を上書き，または nil.
  back_w = num,        -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  back_h = num,        -- number 型で "余白高さ" の項目を上書き，table 型で各タイルごとに指定，または nil.
  rotate = num,        -- number 型で "回転" の項目を上書き，または nil.
  X = num,             -- number 型で "X" の項目を上書き，または nil.
  Y = num,             -- number 型で "Y" の項目を上書き，または nil.
  antialias = bool,    -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
}
```

- `limit_tiles` に指定できる文字列は以下の通り:

  ```lua
  "1種類", "2種類", "4種類"
  ```

####  三角タイルσ の `PI`
```lua
{
  width = num,        -- number 型で "幅" の項目を上書き，または nil.
  height = num,       -- number 型で "高さ" の項目を上書き，または nil.
  screen_size = bool, -- boolean 型で "背景サイズ" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  limit_tiles = str,  -- string 型で "タイル種類" の項目を上書き，または nil.
  col = num,          -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_inner = num,    -- number 型で "内部色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  alpha = num,        -- number 型で "透明度*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  block = num,        -- number 型で "マス幅" の項目を上書き，または nil.
  line = num,         -- number 型で "ライン幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  radius = num,       -- number 型で "角半径" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_back = num,     -- number 型で "余白色" の項目を上書き，または nil.
  alpha_back = num,   -- number 型で "余白透明度" の項目を上書き，または nil.
  back = num,         -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  rotate = num,       -- number 型で "回転" の項目を上書き，または nil.
  X = num,            -- number 型で "X" の項目を上書き，または nil.
  Y = num,            -- number 型で "Y" の項目を上書き，または nil.
  antialias = bool,   -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
}
```

- `limit_tiles` に指定できる文字列は以下の通り:

  ```lua
  "1種類", "2種類", "3種類", "6種類"
  ```


####  六角タイルσ の `PI`
```lua
{
  width = num,        -- number 型で "幅" の項目を上書き，または nil.
  height = num,       -- number 型で "高さ" の項目を上書き，または nil.
  screen_size = bool, -- boolean 型で "背景サイズ" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  limit_tiles = str,  -- string 型で "タイル種類" の項目を上書き，または nil.
  col = num,          -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_inner = num,    -- number 型で "内部色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  alpha = num,        -- number 型で "透明度*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  block = num,        -- number 型で "マス幅" の項目を上書き，または nil.
  line = num,         -- number 型で "ライン幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  radius = num,       -- number 型で "角半径" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_back = num,     -- number 型で "余白色" の項目を上書き，または nil.
  alpha_back = num,   -- number 型で "余白透明度" の項目を上書き，または nil.
  back = num,         -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  rotate = num,       -- number 型で "回転" の項目を上書き，または nil.
  X = num,            -- number 型で "X" の項目を上書き，または nil.
  Y = num,            -- number 型で "Y" の項目を上書き，または nil.
  antialias = bool,   -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
}
```

- `limit_tiles` に指定できる文字列は以下の通り:

  ```lua
  "1種類", "3種類"
  ```


####  菱形タイルσ の `PI`
```lua
{
  width = width,             -- number 型で "幅" の項目を上書き，または nil.
  height = height,           -- number 型で "高さ" の項目を上書き，または nil.
  screen_size = screen_size, -- boolean 型で "背景サイズ" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  limit_tiles = str,         -- string 型で "タイル種類" の項目を上書き，または nil.
  col = col,                 -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_inner = col_inner,     -- number 型で "内部色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  alpha = alpha,             -- number 型で "透明度*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  block_w = block_w,         -- number 型で "マス幅" の項目を上書き，または nil.
  block_h = block_h,         -- number 型で "マス高さ" の項目を上書き，または nil.
  line = line,               -- number 型で "ライン幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  radius = radius,           -- number 型で "角半径" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_back = col_back,       -- number 型で "余白色" の項目を上書き，または nil.
  alpha_back = alpha_back,   -- number 型で "余白透明度" の項目を上書き，または nil.
  back = back,               -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  rotate = rotate,           -- number 型で "回転" の項目を上書き，または nil.
  X = X,                     -- number 型で "X" の項目を上書き，または nil.
  Y = Y,                     -- number 型で "Y" の項目を上書き，または nil.
  antialias = antialias,     -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
}
```

- `limit_tiles` に指定できる文字列は以下の通り:

  ```lua
  "1種類", "2種類", "4種類"
  ```

##  改版履歴

- **v1.32** (2026-09-06)

  - ツールチップ用ファイルの名前が正しくなかったのを修正．

- **v1.31** (2026-09-06)

  - 「タイル種類」を少なくしたときの色設定など，一部設定項目を特定条件下で非表示にするように．
  - コード整理．

- **v1.30** (2026-07-15)

  - 一部設定項目にツールチップによる説明表示を追加．

  - `v2.1.0` での動作確認．

- **v1.23 (for beta52)** (2026-06-29)

  - パッケージのバージョン表記が間違っていたのを修正．

- **v1.22 (for beta52)** (2026-06-29)

  - 「アンチエイリアス」が OFF の場合の前回 (v1.21) の修正が不完全だった部分を修正．

    - まだ条件によっては混色が起こりますが，これ以上の補正処理はしないものとします．

- **v1.21 (for beta52)** (2026-06-29)

  - 「アンチエイリアス」が OFF の場合，ピクセルのちょうど中央を通るタイル境界が歪な形状になったり，穴抜け・重複・混色が起こっていたのを修正．

  - D&D でインストール時のメッセージの誤字修正．

  - `beta52` での動作確認．

- **v1.20 (for beta51)** (2026-06-21)

  - 「タイル種類」の設定項目を追加．4 色分けのタイルでも 2 色や 1 色に種類を制限して，色や透明度の設定を簡略化できます．
  - 「三角タイルσ」と「六角タイルσ」でアンチエイリアスが有効の場合，頂点部分が明るくなる傾向があったのを修正．
  - 「三角タイルσ」の「余白」が v1.03 と比べて倍の大きさになっていたのを修正．

- **v1.11 (for beta50)** (2026-06-19)

  - 「六角タイルσ」の色の順序が間違っていたのを修正．

- **v1.10 (for beta50)** (2026-06-19)

  - 描画処理を書き直し，アンチエイリアス処理を改善．
  - 初期ラベルを「カスタムオブジェクト」から「Tile_S」に変更．
  - 各種トラックバーのマウス操作倍率の設定や，チェックボックスを中間点ごとに可変なタイプへ置き換えるなどの GUI 調整．

  - 配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `@Tile_S.obj2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - `beta50` での動作確認．

- **v1.03 (for beta25)** (2025-12-22)

  - 「余白色」「余白透明度」のグループ化分類を「タイル設定」から「色設定」に移動．
  - `beta25` での動作確認．

- **v1.02 (for beta22a)** (2025-12-06)

  - パラメタをグループ化して整理．
  - `beta22a` での動作確認．

- **v1.01 (for beta7)** (2025-08-21)

  - パラメタインジェクションの `PI` で `X`, `Y` の指定が反映されなかったのを修正．

- **v1.00 (for beta7)** (2025-08-20)

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
