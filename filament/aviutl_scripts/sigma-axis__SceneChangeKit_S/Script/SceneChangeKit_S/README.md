# SceneChangeKit_S AviUtl ExEdit2 シーンチェンジ自作キットスクリプト

スクリプトを書かなくてもシーンチェンジをタイムライン上で構成して自作できる，汎用的なシーンチェンジスクリプトです．シーンチェンジの「前シーンを時間延長したもの」を描画する機能を直接利用できます．

「前シーン」と「次シーン」をレイヤーに配置，フィルタ効果などを追加してシーンチェンジをタイムライン上で構成します．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_SceneChangeKit_S/releases) [紹介動画．](https://www.nicovideo.jp/shorts/ss46859392)

![タイムラインでの配置例](https://github.com/user-attachments/assets/5aaaa3be-7f0b-4448-b062-09deaf499fc8)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `ss46859392`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.11` で動作確認済み．

## 導入方法

ダウンロードした `aviutl2_script_SceneChangeKit_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

初期状態だと「メディアオブジェクトを追加」や「フィルタオブジェクトを追加」メニューの「SceneChangeKit_S」に追加されています．
- 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  使い方

通常のシーンチェンジは 1 レイヤーで済みますが，このスクリプトは複数レイヤー (通常は 3 レイヤー以上) が必要です．

典型的は配置は以下の通りです:

1.  シーンチェンジを設定したいところにシーンチェンジの「自作シーンチェンジ」を配置．
1.  その下にオブジェクト「前シーン」を配置．
1.  さらにその下にオブジェクト「次シーン」を配置．
1.  「前シーン」オブジェクトにはフィルタ効果などでアニメーションを設定して，時間経過で隠れるように．
1.  「次シーン」オブジェクトにもフィルタ効果などでアニメーションを設定して，時間経過で現れるように．

![タイムラインでの配置例](https://github.com/user-attachments/assets/5aaaa3be-7f0b-4448-b062-09deaf499fc8)

> [!TIP]
> 「前シーン」や「次シーン」は複数レイヤーにまたがって複数設置可能なので，複数レイヤーを組み合わせた複雑な表現も可能です．

##  パラメタの説明

### 自作シーンチェンジ

シーンチェンジの本体です．これを通常のシーンチェンジと同じように，オブジェクトの先頭フレームが，シーン切り替わりの次シーンの開始位置と一致するように配置します．

この下のレイヤーに[「前シーン」や「次シーン」](#前シーン--次シーン)オブジェクトを配置してください．

####  フレームバッファをクリア

フレームバッファを初期化して，次レイヤー以降でゼロから描画するようにします．

OFF のときはフレームバッファには，「次シーン」と同じ内容が残ります．

- 「次シーン」オブジェクトの配置を省略可能で，内部手順的にも無駄処理を省いていることになります．

初期値は ON.

> [!TIP]
> 標準の「フレームバッファ」オブジェクトの「フレームバッファをクリア」とほぼ同じ機能の設定項目です．

####  背景色

[「フレームバッファをクリア」](#フレームバッファをクリア)が ON のときのみ有効な設定です．クリア時に設定する背景色を指定します．未指定の場合は透明ピクセルになります．

初期値は `未指定`.

### 前シーン / 次シーン

それぞれシーンチェンジの前 / 次のシーンの画像を読み込みます．[「自作シーンチェンジ」]より下のレイヤーに配置してください．

1.  「前シーン」は，時間経過でだんだん隠れるような演出を設定してください．
1.  「次シーン」は，時間経過でだんだん現れるような演出を設定してください．

####  アルファチャンネルを維持

読み込んだ画像のアルファチャンネルを，そのまま維持するかどうかを指定します．OFF の場合は，半透明透明ピクセルの背景を黒色で埋めます．

初期値は ON.

> [!TIP]
> 標準の「フレームバッファ」オブジェクトの「アルファチャンネルを維持」とほぼ同じ機能の設定項目です．


## 改版履歴

- **v1.00** (2026-09-28)

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
