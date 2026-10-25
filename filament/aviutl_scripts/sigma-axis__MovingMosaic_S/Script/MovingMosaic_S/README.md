# MovingMosaic_S AviUtl ExEdit2 動くモザイクスクリプト

静止画にかけても動くモザイクがかけられるスクリプトです．マス内のランダムな位置から色を拾います．

マスの位置を上下左右にずらしたり，サイズも小数点以下の細かさで指定したり，縦長や横長のマスにもできます．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_MovingMosaic_S/releases) [紹介動画．](https://www.nicovideo.jp/shorts/ss46874324)

![動くモザイクの適用例](https://github.com/user-attachments/assets/381cb864-3ee1-4225-aa56-4b9d02c7e10b)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `ss46874324`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.11a` で動作確認済み．

## 導入方法

ダウンロードした `aviutl2_script_MovingMosaic_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

初期状態だと「メディアオブジェクトを追加」や「フィルタオブジェクトを追加」メニューの「加工」に追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  パラメタの説明

### サイズ

モザイクのマスのサイズをピクセル単位で指定します．

最小値は 1, 最大値は 2000, 初期値は 12.

> [!TIP]
> 小数点以下のピクセル数の精度でも指定できます．

### シード

モザイクのそれぞれのマスの，ランダムな位置から拾った色をマスの色にしますが，そのランダムさを決定するシードを指定します．

- 0 以上の場合，同じシードでも別オブジェクトだと別の乱数になります．
- 負の場合，同じシードなら別オブジェクトでも同じ乱数になります．

最小値は -65536, 最大値は 65535, 初期値は 10000.

### タイル風

モザイクの各マスの境界に明暗のある枠線を付けて，タイルのような見た目にします．

初期値は OFF.

### 縦横比

モザイクのマスを縦長，または横長にできます．

- 正の数だと縦長．
- 負の数だと横長．

最小値は -100, 最大値は 100, 初期値は 0.

### X / Y

モザイクのマスを上下左右にずらします．初期位置からのずれをピクセル単位で指定します．プレビュー画面でアンカーをマウス操作で動かすことでも調節できます．

最小値は -4000, 最大値は 4000, 初期値は 0.

### PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

```lua
{
  size = num,    -- number 型で "サイズ" の項目を上書き，または nil.
  seed = num,    -- number 型で "シード" の項目を上書き，または nil.
  convex = bool, -- boolean 型で "タイル風" を上書き，または nil. 0 を false, 0 以外を true として number 型も可能．
  aspect = num,  -- number 型で "縦横比" の項目を上書き，または nil.
  X = num,       -- number 型で "X" の項目を上書き，または nil.
  Y = num,       -- number 型で "Y" の項目を上書き，または nil.
}
```

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．


## 改版履歴

- **v1.01** (2026-10-03)

  - マスの平行移動などでの小数点以下のピクセル数の取り扱いが間違っていたのを修正．
  - 「サイズ」の項目が 1.00 の場合は何もしないように変更．

- **v1.00** (2026-10-02)

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
