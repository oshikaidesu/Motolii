# PixelSnap_S AviUtl/AviUtl ExEdit2 スクリプト

画像が半ピクセルだけズレるなどの影響でぼやけてしまうのを防ぐ AviUtl スクリプトです．小数点以下の座標調整をせずともピクセルパーフェクトな描画になってくれます．

無印の AviUtl と AviUtl ExEdit2 の両方に対応しています．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_PixelSnap_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45452975)

![半ピクセルずれを補正](https://github.com/user-attachments/assets/92e967d7-4ad8-4aa0-b3ea-06e5b82e1169)

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45452975`

##  動作要件

### AviUtl (無印)

- AviUtl 1.10

  http://spring-fragrance.mints.ne.jp/aviutl

- 拡張編集 0.92

### AviUtl ExEdit2

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `2.1.10` で動作確認済み．

## 導入方法

- AviUtl (無印) の場合

  以下のフォルダのいずれかに `PixelSnap_S.anm` をコピーしてください．

  1. `exedit.auf` のあるフォルダにある `script` フォルダ
  1. (1) のフォルダにある任意の名前のフォルダ

  アニメーション効果として PixelSnap_S が追加されています．

- AviUtl ExEdit2 の場合

  ダウンロードした `aviutl2_script_PixelSnap_S-v*.**.au2pkg.zip` を AviUtl2 のウィンドウにドラッグ & ドロップしてください．

  初期状態だと「フィルタ効果を追加」メニューの「配置」に PixelSnap_S が追加されています．
  - 「オブジェクト追加メニューの設定」の「ラベル」項目で分類を変更できます．


##  仕組み

オブジェクトの座標位置と回転中心を小数点以下のピクセル数だけ動かして，シーンのピクセルとオブジェクトのピクセルが 1 対 1 に対応する位置に微調整します．適用前と比べてオブジェクトの位置が 1 ピクセル未満だけズレますが，オブジェクトに余計なボケが発生しない，ピクセルパーフェクトな描画になります．

- 具体的には `obj.ox`, `obj.oy`, `obj.cx`, `obj.cy` の数値を操作して，これらの座標がオブジェクトやシーンのピクセルの頂点，または中央に来るように微調整しています．


##  パラメタの説明

ありません．

##  TIPS

1.  AviUtl (無印) 版の場合，テキストエディタで `PixelSnap_S.anm` を開くと冒頭付近にファイルバージョンが付記されています．

    ```lua
    --
    -- VERSION: v1.20
    --
    ```

1.  次のような場合はこのスクリプトを適用していても，ピクセルパーフェクトな描画にならないことがあります．

    1.  標準描画の「拡大率」や「縦横比」，フィルタ効果の「拡大率」がかかっているとき．

        拡大縮小の画像処理が入ってしまうためぼやけてしまいます．

    1.  オブジェクトの Z 座標が 0 でなかったり，立体回転 (X, Y 軸の回転) がかかっていたり，カメラ制御下にあるとき．

        遠近法の表現のため，拡大縮小処理がかかります．

    1.  オブジェクトに 90° の整数倍でない，または立体的な回転がかかっているとき．

        90° の整数倍の平面回転 (Z 軸回転) なら問題ありません．

    1.  AviUtl (無印) の場合，グループ制御下に入れているオブジェクトで，そのグループ制御で小数点以下のピクセル数だけズレている場合．

    1.  AviUtl ExEdit2 の場合，「時間制御(オブジェクト)」の配下のオブジェクトで，時間制御の外側にあるグループ制御の影響を受けている場合．
        - スクリプトで取得できるグループ制御の状態が，「時間制御(オブジェクト)」の影響を受けた時間のものになってしまうため．

    1.  その他，上記のような拡大率や回転，Z 座標などを操作するフィルタ効果がかかっているとき．

    1.  このフィルタ効果 (アニメーション効果) の後続フィルタにオブジェクトの座標を動かすものが適用されている場合．

1.  このスクリプトを適用している場合，オブジェクトを平行移動してもピクセル単位で移動するためガクガクした動きになります．

    移動系のアニメーションなどと併用する場合は，このスクリプトの後続に配置したフィルタを使って動かすとスムーズな動きになります．ただしその場合，動きの始点終点などの動きのないタイミングでは，整数単位の移動量を指定するなど通常通りのピクセル単位の配慮が必要です．


##  改版履歴

- **v1.20** (2026-09-19)

  - AviUtl ExEdit2 版で，グループ制御やその入れ子の場合でも正しく動作するように．
  - 立体射影によるピクセル位置も計算に入れるように (AviUtl 無印 / ExEdit2 版両方).
    - 例えば，Z = 1024 (カメラから 2 倍の距離) で拡大率 200% の場合でも，ピクセル単位に正しく揃えるように．
    - AviUtl 無印版の場合は，グループ制御下では正しく動作しない場合があります．

- **v1.10** (2026-09-13)

  - AviUtl ExEdit2 版で，1階層までならグループ制御の配下でも正しく動作するように．
    - 入れ子になっているグループ制御の場合，対象オブジェクト直上のグループ制御以外で，座標や回転，拡大率を操作しているものがある場合は正しく動作しません．
  - コード整理．
  - AviUtl2 version 2.1.9 で動作確認．

- **v1.05 (for beta50)** (2026-06-19)

  - コード整理．
  - AviUtl2 版で配布形式を `.au2pkg.zip` (AviUtl2 のパッケージ形式) に変更．
    - **以前のバージョンから更新する際は，以前の導入時にコピーしたファイルを一度削除してから導入してください．**

      同名ファイルが複数フォルダに分散して重複して認識されないようにするためで，次のファイルが削除対象です:

      1.  `PixelSnap_S.anm2`

      スクリプトフォルダ，またはその 1 階層下のサブフォルダ内に配置されています．

  - AviUtl2 beta50 で動作確認．

- **v1.04 (for beta20)** (2025-11-16)

  - AviUtl ExEdit2 版で，beta20 における拡大率の仕様変更に対応．

- **v1.03 (for beta13)** (2025-09-29)

  - AviUtl ExEdit2 版でグループ制御に付けた場合の動作を変更．

    グループ制御の座標を反映していたのを，配下のオブジェクトの座標を反映するように．グループ制御で設定した座標は矯正されなくなりました．

    AviUtl (無印) と同等の挙動になりました．

- **v1.02 (for beta13)** (2025-09-28)

  - AviUtl (無印) 版で標準描画の拡大率を操作変更していた場合に計算が正しくなかったのを修正．

  - AviUtl (無印) 版のファイルにもコメントとしてバージョン表記を追加．

- **v1.01 (for beta13)** (2025-09-28)

  - AviUtl (無印) 版で回転角度を設定しているオブジェクトに対して，矯正方向が正しくなかったのを修正．

  - AviUtl ExEdit2 版のスクリプト情報表示が間違っていたのを修正．

- **v1.00 (for beta13)** (2025-09-27)

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


#  連絡・バグ報告

- GitHub: https://github.com/sigma-axis
- Twitter: https://x.com/sigma_axis
- nicovideo: https://www.nicovideo.jp/user/51492481
- Misskey.io: https://misskey.io/@sigma_axis
- Bluesky: https://bsky.app/profile/sigma-axis.bsky.social
