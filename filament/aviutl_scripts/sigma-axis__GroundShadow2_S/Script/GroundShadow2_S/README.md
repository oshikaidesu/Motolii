# GroundShadow2_S AviUtl/AviUtl ExEdit2 スクリプト

影を立体的に地面へ落とすアニメーション効果 / フィルタ効果．地面だけでなく壁や天井など任意角度の平面に影を落とせます．[GroundShadow_S](https://github.com/sigma-axis/aviutl_script_GroundShadow_S) の拡張版です ([変更点はこちら](#groundshadow_s-からの変更点)).

無印の AviUtl と AviUtl ExEdit2 の両方に対応しています．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_GroundShadow2_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45275176)

![使用例1](https://github.com/user-attachments/assets/a5057d40-4e49-4f54-9390-a7a86b8c061e)

![使用例2](https://github.com/user-attachments/assets/79a40754-4e61-442b-a6c5-ea967b5bb9a6)
- イラスト: 琴葉茜 琴葉葵 (c) AI Inc.

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45275176`

##  動作要件

### AviUtl (無印)

- AviUtl 1.10

  http://spring-fragrance.mints.ne.jp/aviutl

- 拡張編集 0.92

- GLShaderKit

  https://github.com/karoterra/aviutl-GLShaderKit

  - `v0.4.0` / `v0.5.0` で動作確認．

- **(推奨)** patch.aul (謎さうなフォーク版)

  https://github.com/nazonoSAUNA/patch.aul

  アンカー位置の認識がずれる原因が 1 つ減ります．
  - 設定ファイル `patch.aul.json` で `"switch"` 以下の `"lua"` と `"lua.getvalue"` を `true` (初期値) にしてください．

### AviUtl ExEdit2

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `v2.1.9` で動作確認済み．

## 導入方法

- AviUtl (無印) の場合

  以下のフォルダのいずれかに `GroundShadow2_S.anm`, `GroundShadow2_S.lua`, `GroundShadow2_S.frag` の 3 つのファイルをコピーしてください．

  1. `exedit.auf` のあるフォルダにある `script` フォルダ
  1. (1) のフォルダにある任意の名前のフォルダ

- AviUtl ExEdit2 の場合

  ダウンロードした `aviutl2_script_GroundShadow2_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

  初期状態だと「フィルタ効果を追加」メニューの「装飾」に GroundShadow2_S が追加されています．
  - 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users of AviUtl ExEdit2

You may be able to find language translation file for this script / plugin from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis).

Translation files enable names, parameters and commands of the scripts / plugins to be displayed in other languages.

Although, usage documentations for this script / plugin in languages other than Japanese are not available now.

##  パラメタの説明

AviUtl (無印) 版では AviUtl ExEdit2 版と比べてパラメタの並びが異なっていたり，一部がパラメタ設定ダイアログ経由での設定になりますが，特記事項がない限り基本的には同じ機能です．

### 地面位置 / 地面位置を画面基準

影を落とす “地面” の位置を指定します．ここで指定した座標を通り，[「地面角度」](#地面角度)と[「回転」](#回転)で指定した角度の平面に影を落とします．プレビューのアンカーをドラッグすることでも調整できます．また「地面位置を画面基準」のチェックを入れると，オブジェクトの位置によらず画面の指定位置に固定されるようになります．

- 地面位置
  - 緑のアンカー線を操作することで指定できます．

    - オブジェクトが “地面” に接する点にアンカーを置くことで “地面に立っている” ような表現ができます．オブジェクトから離すと “地面から浮いている” ように見えます．

    - AviUtl (無印) の場合，オブジェクトに拡大率や回転などが設定されていると，アンカー位置を正しく取得できない場合があります．

  - 座標は 3 次元で指定します．オブジェクトと平行で，オブジェクトの奥にある平面などに影を投射することもできます．

  オブジェクトの中心からの相対座標でピクセル単位で指定．AviUtl (無印) 版と AviUtl2 版で指定方法が少し異なります:

  - AviUtl 無印版:

    「地面位置」に `{<X座標>, <Y座標>, <Z座標>}` と記述．初期値は `{0,200,0}`.

  - AviUtl2 版:

    「地面位置X/Y/Z」のトラックバーで指定．最小値は -4000, 最大値は 4000, 初期値は $(0, 200, 0)$.

- 地面位置を画面基準
  - AviUtl 無印版の場合，「カメラ位置」の直下にある「画面基準」という項目が，[「カメラ位置を画面基準」](#カメラ位置--カメラ位置を画面基準)と兼用しています．また，ON の場合は地面位置のアンカーが表示されません．
    - パラメタ設定ダイアログの個数上限 16 個に引っかかるため，兼用となっています．[`PI`](#aviutl-無印-版の-pi) で個別に設定できます．

  初期値は OFF.

> [!NOTE]
> 「地面位置を画面基準」が ON の場合，グループ制御の座標移動や回転，後続のフィルタ効果で移動を伴うものなどの影響で，位置がずれることがあります．

### 地面角度

影を落とす先の “地面” の仰角を指定します．[「回転」](#回転)が 0 の場合，カメラ視点を基準として次のように決まります:

| 負の値 | 0 | 正の値 |
|:---:|:---:|:---:|
| 下り坂 | 水平 | 上り坂 |

単位は度数法で，初期値は 0.
- AviUtl 無印版の場合，最小値は -180, 最大値は 180.
- AviUtl2 版の場合，最小値は -1800, 最大値は 1800.

### 光源角度 / 光源傾斜

光の入射角を指定します．[「回転」](#回転)が 0 の場合，「光源角度」で上下方向が，「光源傾斜」で左右方向が，カメラ視点を基準として次のように決まります:

| パラメタ | 負の値 | 0 | 正の値 |
|:---:|:---:|:---:|:---:|
| 「光源角度」 | 上から下 | 手前から奥への水平方向 | 下から上 |
| 「光源傾斜」 | 右から左 | 左右にまっすぐ | 左から右 |

「光源角度」の単位は度数法で，初期値は -45.
- AviUtl 無印版の場合，最小値は -180, 最大値は 180.
- AviUtl2 版の場合，最小値は -1800, 最大値は 1800.

「光源傾斜」は傾斜量 (角度の正接 ($\tan$ 関数) の値) の % 単位で，初期値は 0.
- AviUtl 無印版の場合，最小値・最小値は ±800 (約 $\pm 82.87\degree$).
- AviUtl2 版の場合，最小値・最小値は ±1600 (約 $\pm 86.42\degree$).

### 回転

影を落とす先の「地面」の回転角を指定します．回転軸は $z$ 軸方向 (手前から奥への方向) で，時計回りに正．これに連動して，[「光源角度」や「光源傾斜」](#光源角度--光源傾斜)で指定した “光源” の角度も回転します．

単位は度数法で，初期値は 0.
- AviUtl 無印版の場合，最小値は -720, 最大値は 720.
- AviUtl2 版の場合，最小値は -3600, 最大値は 3600.

### カメラ位置 / カメラ位置を画面基準

立体射影を計算する際の，カメラ位置の $x, y$ 座標を指定します．プレビューのアンカーをドラッグすることでも調整できます．また「カメラ位置を画面基準」のチェックを入れると，オブジェクトの位置によらず画面の指定位置に固定されるようになります．

- カメラ位置
  - 青いアンカー線を操作することで指定できます．
  - オブジェクトに拡大率や回転などが設定されていた場合，アンカー位置を正しく取得できない場合があります．

  オブジェクトの中心からの相対座標でピクセル単位で指定．AviUtl (無印) 版と AviUtl2 版で指定方法が少し異なります:

  - AviUtl 無印版:

    「カメラ位置」に `{<X座標>, <Y座標>}` と記述．初期値は `{0,-200}`.

  - AviUtl2 版:

    「カメラ位置X/Y」のトラックバーで指定．最小値は -4000, 最大値は 4000, 初期値は $(0, -200)$.

- カメラ位置を画面基準
  - AviUtl 無印版の場合，「カメラ位置」の直下にある「画面基準」という項目が，[「地面位置を画面基準」](#地面位置--地面位置を画面基準)と兼用しています．また，ON の場合はカメラ位置のアンカーが表示されません．
    - パラメタ設定ダイアログの個数上限 16 個に引っかかるため，兼用となっています．[`PI`](#aviutl-無印-版の-pi) で個別に設定できます．

  初期値は OFF.

> [!NOTE]
> 「カメラ位置を画面基準」が ON の場合，グループ制御の座標移動や回転，後続のフィルタ効果で移動を伴うものなどの影響で，位置がずれることがあります．

### 視野幅

立体射影を計算する際の，カメラの視野角を調整します．単位は % で，視野角の正接 ($\tan$ 関数の値) の割合で指定します．

- 大きくすると立体感が強くなり，遠くの影がさらに小さく，近くの影はさらに大きくなります．
- 小さくすると立体感が弱くなり，遠くの影と近くの影の大きさに違いが出にくくなります．
- 0 を指定すると[直投影](https://ja.wikipedia.org/wiki/%E7%9B%B4%E6%8A%95%E5%BD%B1) ([orthographic projection](https://en.wikipedia.org/wiki/Orthographic_projection)) な描画ができます．
  - この場合[「カメラ位置」](#カメラ位置--カメラ位置を画面基準)のパラメタは影響しなくなります．

最小値は 0, 最大値は 10000, 初期値は 100.

### 影色

影の色を指定します．初期値は `000000` (黒).

### 影色強さ

[「影色」](#影色)を適用する強さ指定します．0 だと元オブジェクトと同じ色で影を描画します．

単位は % で最小値は 0, 最大値は 100, 初期値は 100.

### 影の濃さ

影の不透明度を指定します．

単位は % で最小値は 0, 最大値は 100, 初期値は 50.

### 前景透明度

影の元となるオブジェクトの透明度を指定します．

単位は % で最小値は 0, 最大値は 100, 初期値は 0.

### 光拡散 / 境界ぼかし

影のぼかし方を指定します．

- 「光拡散」は光源の入射角のブレ量で，オブジェクトと “地面” に映る影との距離が長いほどぼかし量が大きくなります．

- 「境界ぼかし」は元画像にそのままかけるぼかし量で，“地面” との距離に依存しません．

「光拡散」は拡散角度の正接 ($\tan$ 関数の値) の % 単位で，最小値は 0, 最大値は 70 (約 $\pm 34.992\degree$), 初期値は 5 (約 $\pm 2.862\degree$).

「境界ぼかし」はピクセル単位で最小値は 0, 最大値は 500, 初期値は 4.

### 影の範囲

[「地面位置」](#地面位置--地面位置を画面基準)で指定した $x, y$ 座標 に近い部分のみを影の描画対象にします．描画対象の範囲を「地面位置」からの距離で指定，0 を指定すると現在オブジェクト全てが描画対象になります．

ピクセル単位で指定し最小値は 0, 最大値は 4000, 初期値は 0.

### 先端ぼかし

[「影の範囲」](#影の範囲)が 0 以外の場合にのみ有効で，描画範囲の先端部分の不透明度を距離に応じて減衰させます．

薄くなる部分の距離をピクセル単位で指定．最小値は 0, 最大値は 2000, 初期値は 0.

### 影位置移動

影の描画位置を平行移動で動かします．プレビューのアンカーをドラッグすることでも調整できます．

- 線のついていないアンカーを操作することで指定できます．

  - AviUtl (無印) の場合，オブジェクトに拡大率や回転などが設定されていると，アンカー位置を正しく取得できない場合があります．

移動量をピクセル単位で指定．AviUtl (無印) 版と AviUtl2 版で指定方法が少し異なります:

- AviUtl 無印版:

  「影位置移動」に `{<X移動量>, <Y移動量>}` と記述．初期値は `{0,0}` (移動なし).

- AviUtl2 版:

  「影位置移動X/Y」のトラックバーで指定．最小値は -4000, 最大値は 4000, 初期値は $(0,0)$ (移動なし).

### 影を別オブジェクトで描画 / 合成モード

影の部分だけを直接フレームバッファに描画します．このとき通常とは異なる合成モードで描画できます．ただしこの場合，後続のフィルタ効果は影部分には適用されません．

![合成モードの比較例](https://github.com/user-attachments/assets/025ad2f1-7a25-472c-93f6-bf60e6226c7c)

- 「影を別オブジェクトで描画」の初期値は OFF.
- 「合成モード」は次から選びます．

  AviUtl (無印) 版の場合は文字列 (e.g. `"加算"`) または対応する数値 (e.g. `1`) で入力:

  - `元の合成モード` / `-1`
  - `通常` / `0`
  - `加算` / `1`
  - `減算` / `2`
  - `乗算` / `3`
  - `スクリーン` / `4`
  - `オーバーレイ` / `5`
  - `比較(明)` / `6`
  - `比較(暗)` / `7`
  - `輝度` / `8`
  - `色差` / `9`
  - `陰影` / `10`
  - `明暗` / `11`
  - `差分` / `12`

  以下は AviUtl (無印) 版の場合は文字列で (e.g. `"alpha_add"`)，AviUtl2 版の場合は [`PI`](#aviutl-exedit2-版の-pi)でのみ指定可能です:

  - `alpha_add`
  - `alpha_max`
  - `alpha_sub`
  - `alpha_add2`
  - `rgba_add` (AviUtl2 版のみ)

  初期値は AviUtl `元の合成モード`.

> [!NOTE]
> AviUtl (無印) 版では，`alpha_add` など仮想バッファ専用の合成モードは (指定はできますが) 指定しても正しく適用されません．

### 精度

[「光拡散」](#光拡散--境界ぼかし)の計算の精度を指定します．数値が大きいほど滑らかな描画結果になりますが動作が遅くなります．シェーダープログラム内での繰り返し回数の上限値で，奇数の平方のうち指定値を超えない最大数が実効的な値です．

最小値は 1, 初期値は 529.

AviUtl ExEdit2 版だと，最大値は 9801.

### 画像最大幅 / 画像最大高さ

AviUtl ExEdit2 版のみにある設定です．AviUtl (無印) 版では [`PI`](#aviutl-無印-版の-pi) のみで設定可能で，省略時は環境設定の最大画像サイズです．

フィルタ効果後の画像の最大サイズをピクセル単位で指定します．

- 光源と地面の角度の組み合わせによっては，理論上の計算結果が現実的には扱えない画像サイズになることがあるため，予め上限を設定しておく必要があります．初期状態だと $2000 \times 2000$ のサイズを最大想定にしています．
- この値を大きくする場合，`system.conf` ファイル内の `[Config]` 以下，`TemporaryImageCacheSize` の項目の見直しも検討してください．`max_w` と `max_h` の積が `TemporaryImageCacheSize` の半分未満に収まる程度がサイズ限界の目安です．
- フィルタオブジェクトとして使用した場合，この設定は無視されます (常にシーンの画面サイズに相当する値).

最小値は 1024, 最大値は 16384, 初期値は 2000.

### PI

パラメタインジェクション (parameter injection) です．各種パラメタを Lua の数式で直接指定できます．また，任意のスクリプトコードを実行する記述領域にもなります．

AviUtl (無印) 版と AviUtl ExEdit2 版で指定方法が異なります．

####  AviUtl (無印) 版の `PI`

初期値は `nil`. テーブル型を指定すると `obj.track0` などの代替値として使用されます．

```lua
{
  [0] = check0,         -- boolean 型で "影を別オブジェクトで描画" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  [1] = track0,         -- number 型で "地面角度" の項目を上書き，または nil.
  [2] = track1,         -- number 型で "光源角度" の項目を上書き，または nil.
  [3] = track2,         -- number 型で "光源傾斜" の項目を上書き，または nil.
  [4] = track3,         -- number 型で "回転" の項目を上書き，または nil.
  ground_screen = bool, -- -- boolean 型で "地面位置を画面基準" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  camera_screen = bool, -- -- boolean 型で "カメラ位置を画面基準" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  max_w = num,          -- number 型で "画像最大幅" の項目を上書き，または nil.
  max_h = num,          -- number 型で "画像最大高さ" の項目を上書き，または nil.
}
```

####  AviUtl ExEdit2 版の `PI`

テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．初期値は空欄．

```lua
{
  ground_pos = { x, y, z }, -- table 型で "地面位置" の項目を上書き，または nil.
  ground_screen = bool      -- boolean 型で "地面位置を画面基準" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  ground_angle = num,       -- number 型で "地面角度" の項目を上書き，または nil.
  light_angle = num,        -- number 型で "光源角度" の項目を上書き，または nil.
  light_slope = num,        -- number 型で "光源傾斜" の項目を上書き，または nil.
  rotation = num,           -- number 型で "回転" の項目を上書き，または nil.
  camera_pos = { x, y },    -- table 型で "カメラ位置" の項目を上書き，または nil.
  camera_screen = bool      -- boolean 型で "カメラ位置を画面基準" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  camera_fov = num,         -- number 型で "視野幅" の項目を上書き，または nil.
  col = num,                -- number 型で "影色" の項目を上書き，または nil.
  col_alpha = num,          -- number 型で "影色強さ" の項目を上書き，または nil.
  alpha = num,              -- number 型で "影の濃さ" の項目を上書き，または nil.
  front_alpha = num,        -- number 型で "前景透明度" の項目を上書き，または nil.
  conic_blur = num,         -- number 型で "光分散" の項目を上書き，または nil.
  edge_blur = num,          -- number 型で "境界ぼかし" の項目を上書き，または nil.
  len = num,                -- number 型で "影の範囲" の項目を上書き，または nil.
  tip_blur = num,           -- number 型で "先端ぼかし" の項目を上書き，または nil.
  pos = { x, y },           -- table 型で "影位置移動" の項目を上書き，または nil.,
  draw = bool,              -- boolean 型で "影を別オブジェクトで描画" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  blend = str,              -- string 型で "合成モード" を上書き，または nil.
  quality = num,            -- number 型で "精度" の項目を上書き，または nil.
  max_w = num,              -- number 型で "画像最大幅" の項目を上書き，または nil.
  max_h = num,              -- number 型で "画像最大高さ" の項目を上書き，または nil.
}
```
- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

- `blend` に指定できる string は以下の通りです:

  ```lua
  "元の合成モード",
  "通常", "加算", "減算", "乗算", "スクリーン", "オーバーレイ",
  "比較(明)", "比較(暗)", "輝度", "色差",
  "陰影", "明暗", "差分",
  "alpha_add", "alpha_max", "alpha_sub", "alpha_add2", "rgba_add",
  ```

##  GroundShadow_S からの変更点

[GroundShadow_S](https://github.com/sigma-axis/aviutl_script_GroundShadow_S) からの変更点は以下の通りです．

- 追加項目

  1.  [「回転」](#回転).
  1.  [「光拡散」](#光拡散--境界ぼかし).
  1.  [「影を別オブジェクトで描画」「合成モード」](#影を別オブジェクトで描画--合成モード).
  1.  [「精度」](#精度).
  1.  [「画像最大幅」「画像最大高さ」](#画像最大幅--画像最大高さ) (AviUtl ExEdit2 版).

      AviUtl2 の取り扱える画像サイズの仕様変更に伴って追加．

- 削除項目

  1.  「左右に射影」

      [「回転」](#回転)に機能を移行・拡張．

  1.  「カメラ距離」

      [「視野幅」](#視野幅)と機能が重複していたのを統一．

- その他

  1.  光源の入射角を[「光源角度」と「光源傾斜」](#光源角度--光源傾斜)から計算する手順を変更．
      - 手前側に影を落とした場合，「光源傾斜」による影の動く方向が逆になります．
      - 奥側に落とした場合でも，同じ数値を指定しても微妙に違う角度になります．

  1.  「濃さ」を[「影の濃さ」](#影の濃さ)に名前変更．
  1.  「影拡散」を[「境界ぼかし」](#光拡散--境界ぼかし)に名前変更．
  1.  [「影の範囲」](#影の範囲)で全体を描画する特殊な指定値を，-1 から 0 に変更．
  1.  「影先端ぼかし」を[「先端ぼかし」](#先端ぼかし)に名前変更．

##  TIPS

1.  AviUtl 無印版の場合，テキストエディタで `GroundShadow2_S.anm`, `GroundShadow2_S.lua`, `GroundShadow2_S.frag` を開くと冒頭付近にファイルバージョンが付記されています．

    ```lua
    --
    -- VERSION: v1.40
    --
    ```

    ファイル間でバージョンが異なる場合，更新漏れの可能性があるためご確認ください．

##  改版履歴

- **v1.40** (2026-09-12)

  - 「影を別オブジェクトで描画」が ON のとき，対象オブジェクトの合成モードが「通常」以外でも「合成モード」の設定項目が反映されるように変更．
  - 「合成モード」の設定に「元の合成モード」の選択項目を追加．
  - 「合成モード」の初期値を「元の合成モード」に変更．

- **v1.32** (2026-09-06)

  - AviUtl2 版で一部設定項目を条件によって非表示にするように．

- **v1.31** (2026-07-14)

  - AviUtl2 版で「地面位置を画面基準」のチェックボックスを追加．
  - AviUtl (無印) 版で「画面基準」のチェックボックスが「カメラ位置」だけでなく「地面位置」も画面基準にするように変更．
    - パラメタの個数上限 16 個に対応する措置です．個別に指定する場合，`PI` に `ground_screen=true,camera_screen=false` の形の記述をしてください．

- **v1.30** (2026-07-14)

  - 「カメラ位置を画面基準」のチェックボックスを追加．
  - AviUtl2 版で「影を別オブジェクトで描画」のチェックボックスを，中間点毎に可変なタイプに置き換え．
  - AviUtl2 版で一部パラメタにツールチップ表示を追加．

  - AviUtl ExEdit2 version 2.1.0 で動作確認．

- **v1.20 (for beta43b)** (2026-04-29)

  - 「影を別オブジェクトで描画」「合成モード」のパラメタを追加．

- **v1.10 (for beta43b)** (2026-04-29)

  - AviUtl2 版で，前回更新からの AviUtl2 本体のスクリプト機能追加を適用 (`--trackgroup` 指定やトラックバーの操作倍率など). それに伴って一部トラックバーの最大・最小の範囲を拡大．

  - AviUtl2 版で **一部パラメタを互換対応扱い (非推奨) に**:
    1.  「地面位置」，代わりにトラックバーの「地面位置X/Y/Z」を追加．
    1.  「カメラ位置」，代わりにトラックバーの「カメラ位置X/Y」を追加．
    1.  「影位置移動」，代わりにトラックバーの「影位置移動X/Y」を追加．

    以前のものよりもトラックバーのほうが，アニメーションさせることができるなど利点がある一方，`beta37` より前だと複数アンカーをトラックバーに割り当てられなかった仕様の名残です．**将来的に削除するつもりなので，これらを利用しているプロジェクトやプリセット，スクリプトなどは，新規追加のパラメタに移行をお願いします．**

  - AviUtl2 版で配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `GroundShadow2_S.anm2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - AviUtl2 beta43b で動作確認．

- **v1.05 (for beta25)** (2025-12-22)

  - AviUtl2 版でパラメタ「精度」「画像最大幅」「画像最大高さ」を追加．

    - パラメタインジェクション経由のみの指定だったのを，独立のパラメタに分離．
    - それに伴って，パラメタインジェクションの初期値を空欄に変更．

  - AviUtl2 版でフィルタオブジェクトとして使えるように設定．

  - AviUtl2 beta25 で動作確認．

- **v1.04 (for beta22a)** (2025-12-05)

  - AviUtl2 版でパラメタをグループ化して整理．

  - AviUtl2 beta22a で動作確認．

- **v1.03 (for beta20)** (2025-11-17)

  - AviUtl2 版でのシェーダー処理を一部簡略化．

  - AviUtl2 beta20 で動作確認．

- **v1.02 (for beta12)** (2025-09-25)

  - AviUtl (無印) 版で `影色強さ` が低いとき，影の上下左右端の 1 ピクセル幅が黒っぽくなることがあったのを修正．

  - AviUtl2 beta12 で動作確認．

- **v1.01 (for beta11a)** (2025-09-20)

  - AviUtl (無印) 版で，エラーメッセージが間違っていたのを修正．

  - AviUtl2 beta11a で動作確認．

- **v1.00 (for beta5)** (2025-08-10)

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
