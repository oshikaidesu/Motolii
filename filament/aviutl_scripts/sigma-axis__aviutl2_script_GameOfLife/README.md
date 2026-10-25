# Conway's Game of Life AviUtl ExEdit2 スクリプト

与えられた画像を初期配置として[ライフゲーム](https://ja.wikipedia.org/wiki/ライフゲーム)をするフィルタ効果です．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_GameOfLife/releases) [紹介動画](https://www.nicovideo.jp/watch/sm45928264)

![Game of Life playing ping-pong with four gliders](https://github.com/user-attachments/assets/dd957b8a-5154-423e-9660-43104bbabb0e)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45928264`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `beta53a` で動作確認済み．

##  導入方法

ダウンロードした `aviutl2_script_GameOfLife-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

初期状態だと「フィルタ効果を追加」メニューの「加工」に追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

This script package contains language translation files, which enable names and parameters of the script to be displayed in other languages.

Currently the following languages are available:
- English

Although, usage documentations for this script in languages other than Japanese are not available now.

##  使い方

初期配置としたい画像に対してフィルタ効果 `GameOfLife` を適用してください．ライフゲームとしての[初期配置](#初代基準)は，画像のアルファ値や明るさを基準にして決まります．

[キャッシュ](#キャッシュ有効)を用いれば計算を効率化でき，大きい世代数でも短い時間で処理することができるようになります．ただし，元画像が動画などの動く画像の場合は不適です．

フィルタ処理後は，生きているマスを[指定色](#色)の不透明ピクセル，死んでいるマスを透明ピクセルとした画像になります．

##  パラメタの説明

### 世代数

初期配置から経過させる世代数を指定します．ただし[キャッシュ](#キャッシュ有効)がない場合，[「最大ステップ数」](#最大ステップ数)を超えた指定ができません．

最小値は 0, 最大値は 65536, 初期値は 0.

> [!TIP]
> 最大値の 65536 を超えた値を指定したい場合，[「等速移動(秒)@Basic_S」「等速移動(フレーム)@Basic_S」](https://github.com/sigma-axis/aviutl2_script_Basic_S/wiki/等速移動)など利用すると上限を無視できます．

### 初代基準

ライフゲームとしての初期配置を生成する方法を指定します．

| 初代基準 | 説明 |
|:---:|:---|
| `不透明ピクセル` | 画像の各ピクセルのアルファ値が，[「しきい値」](#しきい値)以上のピクセルを生きているマス，それ以外を死んでいるマスとする． |
| `暗い色` | 画像の各ピクセルの明るさの *白との差が* ，[「しきい値」](#しきい値)以上のピクセルを生きているマス，それ以外を死んでいるマスとする． |
| `明るい色` | 画像の各ピクセルの明るさが，[「しきい値」](#しきい値)以上のピクセルを生きているマス，それ以外を死んでいるマスとする． |

- 「明るさ」の計算は [YUV (BT.601)](https://ja.wikipedia.org/wiki/YUV) に従います．

初期値は `不透明ピクセル`.

### しきい値

ライフゲームとしての初期配置生成のしきい値を調整します．具体的な影響は[「初代基準」](#初代基準)を参照．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 50.

### 色

フィルタ効果後の画像で，生きているマスを表す色を指定します．

初期値は `ffffff` (白色).

### キャッシュ有効

計算の際にキャッシュ利用をするようにします．前回のフィルタ効果時に得られた世代の計算結果を再利用することで計算を効率化し，より大きい世代数を速く生成することができます．

- キャッシュはそのまま画像としてのデータを 2 枚分保持するため，`画像のピクセル数 × 8` バイトのメモリが必要です．
- 元画像に変更を加えても，キャッシュが有効の場合即座には反映されません．その場合は[「キャッシュ削除」](#キャッシュ削除)を利用してください．
- 次のパラメタが変化した場合，キャッシュの再利用が行われず初代から再計算されます:
  1.  [「初代基準」](#初代基準)
  1.  [「しきい値」](#しきい値)
  1.  [「画像ループ」](#画像ループ)

初期値は OFF.

### キャッシュ削除

キャッシュとして保持しているデータを破棄します．このチェックを入れたオブジェクトが描画されたタイミングで破棄されます．

- チェックを入れたオブジェクトが描画されるように，フレーム位置などを調整してからチェックを入れてください．
- 一度チェックを入れて画面の更新が確認できたなら，チェックを外してください．
- チェックを入れたオブジェクトに限らず， **全て** の `GameOfLife` スクリプトを利用しているオブジェクトに影響します．

初期値は OFF.

### 画像ループ

画面の上下左右でループしている (上端が同じ水平位置の下端と，左端が同じ垂直位置の右端と，それぞれつながっている) ような形式で，ライフゲームの計算をします．

| 画像ループ | 説明 |
|:---:|:---|
| `なし` | ループしない． |
| `横方向` | 水平方向にのみループする． |
| `縦方向` | 垂直方向にのみループする． |
| `両方向` | 上下左右 4 方向にループする． |

初期値は `なし`.

### 最大追加サイズ

一般にライフゲームは，世代が進むと生きているマスの存在範囲が広がっていきます．その広がりの上限を指定します．

- この上限を超えた位置のマスは常に死滅扱いとなります．
- [「画像ループ」](#画像ループ)でループ指定した方向には広がりません．

ピクセル数で指定，最小値は 0, 最大値は 2000, 初期値は 0.

### 最大ステップ数

1 回のスクリプトの計算あたりに進められる最大の世代数を指定します．極端に大きな世代数を計算すると GPU 負荷が大きくなりすぎるため，安全策としての指定です．

最小値は 1, 初期値は 1024.

### 初代はそのまま表示

[「世代数」](#世代数)が 0 の場合は，マスに[色](#色)を付けたりせず，無加工のまま処理を終了するようにします．

初期値は OFF.

### PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

```lua
{
  generation = num,   -- number 型で "世代数" の項目を上書き，または nil.
  channel = str,      -- string 型で "初代基準" の項目を上書き，または nil.
  threshold = num,    -- number 型で "しきい値" の項目を上書き，または nil.
  color = num,        -- number 型で "色" の項目を上書き，または nil.
  cache = bool,       -- boolean 型で "キャッシュ有効" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  clear_cache = bool, -- boolean 型で "キャッシュ削除" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  wrap = str,         -- string 型で "画像ループ" の項目を上書き，または nil.
  expand = num,       -- number 型で "最大追加サイズ" の項目を上書き，または nil.
  max_steps = num,    -- number 型で "最大ステップ数" の項目を上書き，または nil.
  ignore_gen0 = bool, -- boolean 型で "初代はそのまま表示" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
}
```

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

### `PI` の `channel`

指定できる string は以下の通りです:

```lua
"不透明ピクセル", "暗い色", "明るい色"
```

### `PI` の `wrap`

指定できる string は以下の通りです:

```lua
"なし", "横方向", "縦方向", "両方向"
```

##  改版履歴

- **v1.03 (for beta53a)** (2026-07-07)

  - バージョン表記が間違っていたのを修正．
  - `beta53a` での動作確認．

- **v1.02 (for beta52)** (2026-06-29)

  - D&D でインストール時のメッセージの誤字修正．
  - 「初代はそのまま表示」のチェックボックスを中間点毎に可変なものに置き換え．
  - コード整理．

  - `beta52` での動作確認．

- **v1.01 (for beta43a)** (2026-04-28)

  - 極端に大きなサイズに対しても適切に対応できるように．
  - 配布形式の変更．以前は2重の `.zip` ファイルだったのを1重に．
    - 更新する際は，一度以前のパッケージのアンインストールをしてから，更新版をインストールしてください．

  - `beta43a` での動作確認．

- **v1.00 (for beta32)** (2026-02-09)

  - 初版．


## ライセンス

このプログラムの利用・改変・再頒布等に関しては Unlicense ライセンスに従うものとします．

---

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org>


#  連絡・バグ報告

- GitHub: https://github.com/sigma-axis
- Twitter: https://x.com/sigma_axis
- nicovideo: https://www.nicovideo.jp/user/51492481
- Misskey.io: https://misskey.io/@sigma_axis
- Bluesky: https://bsky.app/profile/sigma-axis.bsky.social
