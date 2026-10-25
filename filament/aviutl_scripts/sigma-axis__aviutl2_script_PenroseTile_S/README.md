# ペンローズタイル AviUtl ExEdit2 スクリプト

[ペンローズタイル (Penrose Tiling)](https://ja.wikipedia.org/wiki/ペンローズ・タイル) を生成したり，ペンローズタイルを利用したフィルタ効果を適用する AivUtl ExEdit 2 のスクリプトです．

[ダウンロードはこちら．](https://github.com/sigma-axis/aviutl2_script_PenroseTile_S/releases) [紹介動画．](https://www.nicovideo.jp/watch/sm45361368)

次が追加されます:

1.  [ペンローズタイルσ](#ペンローズタイルσ) (カスタムオブジェクト)
1.  [ペンローズタイル モザイクσ](#ペンローズタイル-モザイクσ)
1.  [ペンローズタイル ぼかしσ](#ペンローズタイル-ぼかしσ)
1.  [ペンローズタイル レンズσ](#ペンローズタイル-レンズσ)
1.  [ペンローズタイル モザイクフェードσ](#ペンローズタイル-モザイクフェードσ)

<img width="320" height="240" alt="ペンローズタイルσの使用例" src="https://github.com/user-attachments/assets/8e63a0c2-3a65-4962-be27-50881c8da484" />

<img width="320" height="200" alt="ペンローズタイル モザイクσの使用例" src="https://github.com/user-attachments/assets/37278491-67b6-4445-80dd-9a01f72680f4" />

<img width="320" height="200" alt="ペンローズタイル ぼかしσの使用例" src="https://github.com/user-attachments/assets/9b5f3104-b26b-453e-bbdc-07fdf3341e6f" />

<img width="300" height="300" alt="ペンローズタイル レンズσの使用例" src="https://github.com/user-attachments/assets/2859f48e-9cd7-452a-a2c3-a4070c904821" />

https://github.com/user-attachments/assets/ab940b10-4bb4-464e-a8af-786f893ceb6a

- <details>
  <summary>元画像出典 (クリックで表示):</summary>

  1.  https://www.pexels.com/photo/green-leafed-tree-beside-body-of-water-during-daytime-158063
  1.  https://www.pexels.com/photo/assorted-color-kittens-45170
  1.  https://www.pexels.com/photo/assorted-color-house-facade-in-park-534124
  1.  https://www.pexels.com/photo/brown-wooden-signage-hanging-beside-wall-434446
  1.  https://www.pexels.com/photo/assorted-car-license-plates-533669
  1.  https://www.pexels.com/photo/four-ace-game-cards-534181
  1.  https://www.pexels.com/photo/vegetables-and-tomatoes-on-cutting-board-255501
  </details>

##  お願い

このスクリプトを使った動画などでは，ニコニコの親作品にこのスクリプトの紹介動画を登録してくれると嬉しいです．任意ではありますが，登録してくれたほうが励みになります．

- 登録 ID: `sm45361368`

##  動作要件

- AviUtl ExEdit2

  http://spring-fragrance.mints.ne.jp/aviutl

  - `beta25` で動作確認済み．

## 導入方法

`ペンローズタイルσ.obj2`, `@PenroseTile_S.anm2`, `ペンローズタイル モザイクフェードσ.scn2` の 3 つのファイルに対して，以下のいずれかの操作をしてください．

1.  AviUtl2 のプレビュー画面にドラッグ&ドロップ．

1.  以下のフォルダのいずれかにコピー．

    1.  スクリプトフォルダ
        - AviUtl2 のメニューの「その他」 :arrow_right: 「アプリケーションデータ」 :arrow_right: 「スクリプトフォルダ」で表示されます．
    1.  (1) のフォルダにある任意の名前のフォルダ

### For non-Japanese speaking users

You may be able to find language translation file for this script from [this repository](https://github.com/sigma-axis/aviutl2_translations_sigma-axis). 
Translation files enable names and parameters of the scripts / filters to be displayed in other languages.

Although, usage documentations for this script in languages other than Japanese are not available now.

##  スクリプトの種類

1 つのオブジェクトと 3 つのフィルタ効果，1 つのシーンチェンジが追加されます．

- 追加メニュー内の分類は「オブジェクト追加メニューの設定」の「ラベル」項目で変更できます．

### ペンローズタイルσ

ペンローズタイルの画像を生成するオブジェクトです．

<img width="320" height="240" alt="ペンローズタイルの例" src="https://github.com/user-attachments/assets/8e63a0c2-3a65-4962-be27-50881c8da484" />

- 初期状態だと「オブジェクトを追加」メニューの「カスタムオブジェクト」に「ペンローズタイルσ」が追加されています．

### ペンローズタイル モザイクσ

タイルごとに単一色に塗り分ける形でモザイクをかけるフィルタ効果です．

<img width="320" height="200" alt="モザイクの例" src="https://github.com/user-attachments/assets/37278491-67b6-4445-80dd-9a01f72680f4" />

- 初期状態だと「フィルタ効果を追加」メニューの「加工」に「ペンローズタイル モザイクσ@PenroseTile_S」が追加されています．

### ペンローズタイル ぼかしσ

タイルの外側に「色が滲み出ない」ようなぼかしをかけるフィルタ効果です．

<img width="320" height="200" alt="ぼかしの例" src="https://github.com/user-attachments/assets/9b5f3104-b26b-453e-bbdc-07fdf3341e6f" />

- 初期状態だと「フィルタ効果を追加」メニューの「ぼかし」に「ペンローズタイル ぼかしσ@PenroseTile_S」が追加されています．

### ペンローズタイル レンズσ

タイルごとにレンズのように歪ませた変形をするフィルタ効果です．

<img width="320" height="320" alt="レンズ効果の例" src="https://github.com/user-attachments/assets/2859f48e-9cd7-452a-a2c3-a4070c904821" />

- 初期状態だと「フィルタ効果を追加」メニューの「変形」に「ペンローズタイル レンズσ@PenroseTile_S」が追加されています．

### ペンローズタイル モザイクフェードσ

[ペンローズタイル モザイクσ](#ペンローズタイル-モザイクσ)のモザイクがだんだん粗くかかっていき，最大サイズのモザイクでフェード，モザイクが細かくなって次のシーンに切り替えるシーンチェンジです．遷移画像ファイル (`Transition` フォルダ内に配置する形式のファイル) を選んで，シーンチェンジの形状を指定することもできます．

https://github.com/user-attachments/assets/ab940b10-4bb4-464e-a8af-786f893ceb6a

- 初期状態だと「フィルタ効果を追加」メニューの「シーンチェンジ」に「ペンローズタイル モザイクフェードσ」が追加されています．

##  パラメタの説明

追加されるカスタムオブジェクトやフィルタ効果には共通して，ペンローズタイルの特性を設定する項目があります．

それに加えて各オブジェクトやフィルタごとに固有の項目もあります．

### 共通の設定項目

<img width="500" height="386" alt="モザイクのGUI" src="https://github.com/user-attachments/assets/df9c1c4b-446f-431c-a33f-08f53fd3dcbd" />

ペンローズタイルの特性を設定するパラメタです．

####  種類

ペンローズタイルの種類を P2, P3 の2種類から選びます．

| `P2(Kite+Dart)` | `P3(菱形2種)` |
|:---:|:---:|
| <img width="240" height="180" alt="P2タイリングの例" src="https://github.com/user-attachments/assets/f9408668-e089-4e39-97d5-4f2749ea0577" /> | <img width="240" height="180" alt="P3タイリングの例" src="https://github.com/user-attachments/assets/f1c19316-7312-4d09-8c63-09f5754bbfb8" /> |
| Kite と Dart と呼ばれる図形で構成される． | 2種類の菱形で構成される． |

初期値は `P2(Kite+Dart)`.

####  中心の形状

Sun と Star の2種類から中心となる形状を選びます．

| 中心の形状 | P2 | P3 |
|:---:|:---:|:---:|
| Sun | <img width="128" height="128" alt="P2のSun" src="https://github.com/user-attachments/assets/b3f4d0c8-ed06-4ee6-aa63-0e254fb19f4f" /> | <img width="128" height="128" alt="P3のSun" src="https://github.com/user-attachments/assets/8af1cda8-2bfe-454d-a112-7da7d6a9f90f" /> |
| Star | <img width="128" height="128" alt="P2のStar" src="https://github.com/user-attachments/assets/ef7663a2-8a73-4760-bf44-128738cd3543" /> | <img width="128" height="128" alt="P3のStar" src="https://github.com/user-attachments/assets/082d7798-37d1-4e8b-b840-6f267a6f7918" /> |

- “Sun”, “Star” は本来，P2 での形状を指しますが，P3 でのこの呼び名はこのスクリプト独自の便宜的なものです．

初期値は `Sun`.

####  サイズ

タイル1枚当たりの大きさを指定します．構成タイルに含まれる辺で最長のものが対象です．

ピクセル単位で指定，最小値は 2, 最大値は 4000, 初期値は 64.

####  再分割

ペンローズタイルに [deflation](https://ja.wikipedia.org/wiki/ペンローズ・タイル#インフレーションとデフレーション) という操作を適用してタイルをさらに細分化します．

[「サイズ」](#サイズ)がさらに小さくなり，[「種類」](#種類)や[「中心の形状」](#中心の形状)も切り替わりますが，細分化前と後で多くの辺や頂点を共有しているため，連続な変化と相性がいいです．

[ペンローズタイル モザイクσ](#ペンローズタイル-モザイクσ)，[ペンローズタイル ぼかしσ](#ペンローズタイル-ぼかしσ)，[ペンローズタイル レンズσ](#ペンローズタイル-レンズσ)にのみある設定です．

細分化の回数で指定，最小値は 0, 最大値は 35, 初期値は 0.

####  回転

タイル模様全体を回転します．回転の中心は[「中心の形状」](#中心の形状)の部分です．

時計回りの度数法で指定，最小値は -720, 最大値は 720, 初期値は 0.

####  X, Y

タイル模様全体を平行移動します．[「中心の形状」](#中心の形状)を配置する位置を指定します．

プレビュー画面のアンカー操作でも設定できます．

ピクセル単位で指定，最小値は -4000, 最大値は 4000, 初期値は 0.

####  アンチエイリアス, 境界処理

タイル間の境界にアンチエイリアスをかけたり，明確な境界線を引いたりできます．

- [ペンローズタイルσ](#ペンローズタイルσ)と[ペンローズタイル モザイクフェードσ](#ペンローズタイル-モザイクフェードσ)では「アンチエイリアス」が利用できます．チェックボックスで ON/OFF を切り替えられます．

  初期値は ON.

- [ペンローズタイル モザイクσ](#ペンローズタイル-モザイクσ)，[ペンローズタイル ぼかしσ](#ペンローズタイル-ぼかしσ)，[ペンローズタイル レンズσ](#ペンローズタイル-レンズσ)では「境界処理」が利用できます．

  `なし`, `アンチエイリアス`, `タイル風` の3つから選びます:

  | 境界処理 | 説明 |
  |:---:|:---|
  | `なし` | タイル間の境界にアンチエイリアス処理をしません．ジャギーな見え方になることがあります． |
  | `アンチエイリアス` | タイル間の境界にアンチエイリアス処理をします． |
  | `タイル風` | タイル間の境界に凹凸感のある境界線を引きます． |

  初期値は以下の通り:

  1.  ペンローズタイル モザイクσ の場合は `アンチエイリアス`.
  1.  ペンローズタイル ぼかしσ の場合は `アンチエイリアス`.
  1.  ペンローズタイル レンズσ の場合は `タイル風`.


### ペンローズタイルσの設定項目

<img width="500" height="772" alt="ペンローズタイルσのGUI" src="https://github.com/user-attachments/assets/e6c34106-e99a-42c9-b643-9b9ef6026691" />

####  幅, 高さ, 背景サイズ

オブジェクトのサイズを指定します．「背景サイズ」が ON の場合は，シーンのサイズと同じになり，「幅」と「高さ」は無視されます．

- 「幅」と「高さ」はピクセル単位で指定，最小値は 0, 最大値は 4000, 初期値は 320.

- 「背景サイズ」の初期値は OFF.

####  ライン幅, ラインぼかし

各タイル部分の縁部分のサイズやぼかし方を指定します．縁部分の色は[「色1」「色2」](#色1-色2)で，内部の色は[「内部色1」「内部色2」](#内部色1-内部色2)で指定します．

- 「ライン幅」は縁部分のサイズをピクセル単位で指定，最小値は 0, 最大値は 1000, 初期値は 1000.

- 「ラインぼかし」は縁部分のぼかし量を，「ライン幅」からの割合で % 単位で指定，最小値は 0, 最大値は 100, 初期値は 0.

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

####  余白幅

各タイル間の “隙間” のサイズを指定します．余白部分の色や透明度は[「余白色」](#余白色)と[「余白透明度」](#余白透明度)で指定します．

ピクセル単位で指定，最小値は 0, 最大値は 2000, 初期値は 2.

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

####  色1, 色2

タイル部分の色を，各タイルでそれぞれ指定します．[「ライン幅」](#ライン幅)が指定されている場合は，縁部分の色です．

P1, P2 ともに，大きい方のタイルが「色1」,小さい方が「色2」です．

初期値は「色1」が `ffffff` (白), 「色2」が `c0c0c0` (75% の灰色).

####  内部色1, 内部色2

タイル部分の内側の色を，各タイルでそれぞれ指定します．[「ライン幅」](#ライン幅-ラインぼかし)が指定されている場合の，縁部分より内側の色です．

- 初期状態だと「ライン幅」でタイル全体が覆われているためこの設定は反映されません．「ライン幅」を[「サイズ」](#サイズ)に比べて十分小さい値を指定してください．

初期値は「内部色1」が `a0a0a0` (63% の灰色), 「内部色2」が `808080` (50% の灰色).

####  透明度1, 透明度2

タイル部分の透明度を，各タイルでそれぞれ指定します．透明度を 100 にすると「穴」が空いたような画像になります．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 0.

####  余白色, 余白透明度

[「余白幅」](#余白幅)を指定したときに見える，各タイル間の「隙間」の色や透明度を指定します．

- 「余白色」の初期値は `000000` (黒).

- 「余白透明度」は % 単位で指定，最小値は 0, 最大値は 100, 初期値は 100.

### ペンローズタイル モザイクσの設定項目

<img width="500" height="386" alt="モザイクのGUI" src="https://github.com/user-attachments/assets/df9c1c4b-446f-431c-a33f-08f53fd3dcbd" />

####  単点サンプル

モザイク効果の各タイルの色を，タイル内の色の平均をとるのではなく，中央の1点の色そのままで決定します．

平均されない分，原色そのままの鮮やかな色合いになりやすいですが，動きのある映像に対して適用すると変化や “チラつき” が激しくなりやすくなります．

初期値は OFF.

### ペンローズタイル ぼかしσの設定項目

<img width="500" height="386" alt="ぼかしのGUI" src="https://github.com/user-attachments/assets/ebac362e-c643-4286-b1c8-b49749acb747" />

####  範囲

標準のぼかし効果の「範囲」と同等のパラメタで，ピクセルの平均を計算する範囲の半径です．

ピクセル単位で指定，最小値は 0, 最大値は 1000, 初期値は 5.

### ペンローズタイル レンズσの設定項目

<img width="500" height="910" alt="レンズのGUI" src="https://github.com/user-attachments/assets/84b4dc14-3494-472c-8a38-4f9df7c3f5d4" />

####  移動X1, 移動Y1, 移動X2, 移動Y2

各タイルの中身を平行移動します．

P1, P2 ともに，大きい方のタイルが「移動X1」「移動Y1」に,小さい方が「移動X2」「移動Y2」に対応しています．

ピクセル単位で指定，最小値は -5000, 最大値は 5000, 初期値は，「移動X1」「移動Y1」が $(-2, 0)$, 「移動X2」「移動Y2」が $(+2, 0)$.

####  回転1, 回転2

各タイルの中身を回転します．

時計回りの度数法で指定，最小値は -720, 最大値は 720, 初期値は 0.

####  拡大率1, 拡大縦横比1, 拡大率2, 拡大縦横比2

各タイルのレンズとしての拡大率を指定します．「拡大縦横比1」「拡大縦横比2」で円柱レンズのような歪み方にもできます．

|タイル | 拡大率: 200<br>縦横比: 0 | 拡大率: 50<br>縦横比: 0 | 拡大率: 200<br>縦横比: +100 | 拡大率: 200<br>縦横比: -100 |
|:---:|:---:|:---:|:---:|:---:|
| 大きい方 | <img width="118" height="160" alt="大きい方 / 200% x 200%" src="https://github.com/user-attachments/assets/e43595c2-b4dd-4827-a38c-93a775ebe13e" /> | <img width="118" height="160" alt="大きい方 / 50% x 50%" src="https://github.com/user-attachments/assets/0868a410-725f-402a-b53c-a04ddefb0911" /> | <img width="118" height="160" alt="大きい方 / 100% x 200%" src="https://github.com/user-attachments/assets/46af8c4c-05a5-493c-a027-7b70caefbf37" /> | <img width="118" height="160" alt="大きい方 / 200% x 200%" src="https://github.com/user-attachments/assets/7304670d-8161-4df5-9e30-598f44960696" /> |
| 小さい方 | <img width="192" height="64" alt="小さい方 / 200% x 200%" src="https://github.com/user-attachments/assets/e2fdd029-86e4-4149-8561-f6695d9448a6" /> | <img width="192" height="64" alt="小さい方 / 50% x 50%" src="https://github.com/user-attachments/assets/e85c6501-7649-41f5-a8a1-98d619a004df" /> | <img width="192" height="64" alt="小さい方 / 100% x 200%" src="https://github.com/user-attachments/assets/ad544bd1-42ae-404c-87d8-8467f1c48887" /> | <img width="192" height="64" alt="小さい方 / 200% x 100%" src="https://github.com/user-attachments/assets/9fb09af3-8ede-4fb6-83d1-2fb7f909eab0" /> |

拡大率を負にすると反転します．

- 「拡大率1」「拡大率2」は % 単位で指定，最小値は -1000, 最大値は 1000, 初期値は 120.

- 「拡大縦横比1」「拡大縦横比2」の最小値は -100, 最大値は 100, 初期値は 0.

####  丸み1, 丸み2

各タイルの外側の部分への，[「拡大率」](#拡大率1-拡大縦横比1-拡大率2-拡大縦横比2)の減退の仕方を調整します．

- 大きいと長い距離で緩やかに減退，小さいと短い距離で急激に減退します．
- [「ハイライト」](#ハイライト1-ハイライト2)の幅にも影響します．

| タイル | 丸み: 10 | 丸み: 50 | 丸み: 100 | 
|:---:|:---:|:---:|:---:|
| 大きい方 | <img width="118" height="160" alt="大きい方 / 丸み 10%" src="https://github.com/user-attachments/assets/b957e638-a063-4ae8-a8da-71c58b4f2168" /> | <img width="118" height="160" alt="大きい方 / 丸み 50%" src="https://github.com/user-attachments/assets/83f68efa-72f5-4e97-9d08-d4894075da0f" /> | <img width="118" height="160" alt="大きい方 / 丸み 100%" src="https://github.com/user-attachments/assets/f22b90b3-78b5-4d68-9e6f-e4bd2ed3d868" /> |
| 小さい方 | <img width="192" height="64" alt="小さい方 / 丸み 10%" src="https://github.com/user-attachments/assets/3a73ba83-8aab-4b41-9caf-566aa5ffcb66" /> | <img width="192" height="64" alt="小さい方 / 丸み 50%" src="https://github.com/user-attachments/assets/b0a4f811-eb14-4153-a73c-38d3df493836" /> | <img width="192" height="64" alt="小さい方 / 丸み 100%" src="https://github.com/user-attachments/assets/06c5551d-0157-42d6-8b4e-d35eb5e5e0dc" /> |

最小値は 0, 最大値は 100, 初期値は 50.

####  ハイライト1, ハイライト2

各タイルに加える反射光のハイライトを調整します．

- ±100 に近いほど強く表示されます．
- 正だと凸型に，負だと凹型にハイライトを表示します．

% 単位で指定，最小値は -100, 最大値は 100, 初期値は 25.

####  光源角度

[「ハイライト」](#ハイライト1-ハイライト2)の位置を決定するのに使う光源の入射角を指定します．

真上を 0 とする時計回りの度数法で指定，最小値は -360, 最大値は 360, 初期値は -45.

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

####  余白幅

各タイルの外側に，レンズとしての歪みのない “平坦” な領域を設けます．

- 余白部分には[「拡大率」](#拡大率1-拡大縦横比1-拡大率2-拡大縦横比2)が影響しません．
- [「ハイライト」](#ハイライト1-ハイライト2)も余白の内側に表示されます．

ピクセル単位で指定，最小値は 0, 最大値は 1000, 初期値は 4.

[`PI`](#pi) を利用すれば，各タイルごとに個別に指定できます．

### ペンローズタイル モザイクフェードσの設定項目

<img width="500" height="630" alt="モザイクフェードのGUI" src="https://github.com/user-attachments/assets/96aaef3a-ff09-4556-934f-ce38cdcec755" />

####  変化幅

シーンチェンジ中の効果が画面内を占める割合が変化します．[「形状」](#形状)と時間経過による影響度のバランスを決定します．

| 変化幅 | 説明 |
|:---:|:---|
| 0 | 前シーンと次シーンのピクセルしか存在せず，変化の途中経過が見えない． |
| 0 -- 100 | 画面の一部分だけが変化途中になる．前シーンと次シーンのピクセルが両方同時に存在するタイミングがある． |
| 100 | 画面全体が変化途中になるタイミングが一瞬あり，そのあと次シーンのピクセルに順次置き換わっていく． |
| 100 -- 200 | 一度，画面全体が変化途中になり，その後しばらくして次シーンのピクセルに順次置き換わっていく． |
| 200 | 画面位置によらず，一律の変化過程になる．この場合，「形状」の影響は受けない． |

最小値は 0, 最大値は 200, 初期値は 75.

####  フェード割合

モザイクが最大サイズになっている時間割合を指定します．この間に次シーンのピクセルの色にフェードしていきます．

% 単位で指定，最小値は 0, 最大値は 100, 初期値は 15.

####  形状

遷移画像のパターンを，`円`, `四角`, `時計`, `ライン`, `ダブルライン`, `クロス` から選びます．

| `円` | `四角` | `時計` | `ライン` | `ダブルライン` | `クロス` |
|:---:|:---:|:---:|:---:|:---:|:---:|
| <img width="160" height="160" alt="円の遷移画像" src="https://github.com/user-attachments/assets/41c09501-b303-49bd-a266-a1586ae84f2d" /> | <img width="160" height="160" alt="四角の遷移画像" src="https://github.com/user-attachments/assets/506a92ff-dca0-43a0-90af-41e5c29e5bef" /> | <img width="160" height="160" alt="時計の遷移画像" src="https://github.com/user-attachments/assets/a46d9be1-7e6c-4315-bd59-4c02820a359d" /> | <img width="160" height="160" alt="ラインの遷移画像" src="https://github.com/user-attachments/assets/b1652a46-79ab-4d91-88fa-3807b6dd406e" /> | <img width="160" height="160" alt="ダブルラインの遷移画像" src="https://github.com/user-attachments/assets/2f69a7ef-0ce3-4419-b9ca-8a96d93c3150" /> | <img width="160" height="160" alt="クロスの遷移画像" src="https://github.com/user-attachments/assets/c10f635f-2dd5-4439-8163-8cd15e92943b" /> |

初期値は `円`.

####  形状回転

[「形状」](#形状)で指定したパターン画像を回転します．

時計回りの度数法で指定，最小値は -360, 最大値は 360, 初期値は 0.

####  縦横比固定

[「形状」](#形状)や[「遷移画像ファイル」](#遷移画像ファイル)で指定した遷移画像を，シーンのアスペクト比に合わせて変形するかどうかを指定します．

- ON で変形せず，元の遷移画像のアスペクト比を保ちます．
- OFF だと変形して，シーンのアスペクト比に合わせます．

初期値は ON.

####  遷移画像ファイル

[「形状」](#形状)の代わりに使用する遷移画像ファイル (`Transition` フォルダ内に配置する形式のファイル) を指定します．

画像の輝度が，低いピクセルから高いピクセルにかけて順次変化していきます．

####  反転

シーンチェンジの方向を反転します．

初期値は OFF.

####  最大サイズ

[「サイズ」](#サイズ)と同様でペンローズタイルのサイズを，最もモザイクが粗いタイミングでの長さで指定します．

ピクセル単位で指定，最小値は 2, 最大値は 2000, 初期値は 192.

####  分割回数

[「再分割」](#再分割)と同様で，変化し始めや変化の終わりでの，最もモザイクが細かいタイミングでのサイズを，[「最大サイズ」](#最大サイズ)からの deflation の回数で指定します．

最小値は 0, 最大値は 35, 初期値は 8.

### PI

パラメタインジェクション (parameter injection) です．初期値は空欄. テーブル型の中身として解釈され，各種パラメタの代替値として使用されます．また，任意のスクリプトコードを実行する記述領域にもなります．

- テキストボックスには冒頭末尾の波括弧 (`{}`) を省略して記述してください．

一部のパラメタについては特殊な記法になっています:

1.  各タイルごとに数値を指定可能な設定値については，次の記述ができます．

    1.  `line = 3.5` (number 型を指定)

        「ライン幅」をタイル全体で一律に指定．

    1.  `line = { 1.5, 2.5 }` (table 型で配列を指定)

        「ライン幅」を，大きい方のタイルが 1.5, 小さい方がが 2.5 と，個別に指定．

    この指定が可能なフィールドには，以下の形式のコメントを記述してあります．
    ```lua
    -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
    ```

1.  各タイルごとに座標を指定可能な設定値については，次の記述ができます．

    1.  `move = 12.5` (number 型を指定)

        「移動X1」「移動Y1」「移動X2」「移動Y2」を全て 12.5 に指定．

    1.  `move = { 15, 25 }` (number 型 2 つ組の table 型を指定)

        「移動X1」「移動X2」を 15 に，「移動Y1」「移動Y2」を 25 に指定．

    1.  `move = { { 10, 20 }, { 30, 40 } }` (table 型で table 型の配列を指定)

        「移動X1」を 10 に，「移動Y1」を 20 に，「移動X2」を 30 に，「移動Y2」を 40 に指定．

    この指定が可能なフィールドには，以下の形式のコメントを記述してあります．
    ```lua
    -- number 型で "移動*" の項目を上書き，{ X, Y } で一律指定，{ { X1, Y1 }, { X2, Y2 } } で各タイルごとに指定，または nil.
    ```

####  ペンローズタイルσの `PI`

```lua
{
  width = num,              -- number 型で "幅" の項目を上書き，または nil.
  height = num,             -- number 型で "高さ" の項目を上書き，または nil.
  screen_size = bool,       -- boolean 型で "背景サイズ" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  pattern = str,            -- string 型で "種類" の項目を上書き，または nil.
  center_form = str,        -- string 型で "中心の形状" の項目を上書き，または nil.
  size = num,               -- number 型で "サイズ" の項目を上書き，または nil.
  line = { line1, line2 },  -- number 型で "ライン幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  line_blur = { bl1, bl2 }, -- number 型で "ラインぼかし" の項目を上書き，table 型で各タイルごとに指定，または nil.
  back = { back1, back2 },  -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col = { col1, col2 },     -- number 型で "色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_inner = { ci1, ci2 }, -- number 型で "内部色*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  alpha = { a1, a2 },       -- number 型で "透明度*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  col_back = num,           -- number 型で "余白色" の項目を上書き，または nil.
  alpha_back = num,         -- number 型で "余白透明度" の項目を上書き，または nil.
  rotate = num,             -- number 型で "回転" の項目を上書き，または nil.
  X = num,                  -- number 型で "X" の項目を上書き，または nil.
  Y = num,                  -- number 型で "Y" の項目を上書き，または nil.
  antialias = bool,         -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
}
```

- [「ライン幅」「ラインぼかし」](#ライン幅-ラインぼかし)，[「余白幅」](#余白幅)に関しては `PI` 経由でのみタイルごとに個別指定できます．

####  ペンローズタイル モザイクσの `PI`

```lua
{
  one_sample = bool, -- boolean 型で "単点サンプル" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  pattern = str,     -- string 型で "種類" の項目を上書き，または nil.
  center_form = str, -- string 型で "中心の形状" の項目を上書き，または nil.
  size = num,        -- number 型で "サイズ" の項目を上書き，または nil.
  deflate = num,     -- number 型で "再分割" の項目を上書き，または nil.
  rotate = num,      -- number 型で "回転" の項目を上書き，または nil.
  X = num,           -- number 型で "X" の項目を上書き，または nil.
  Y = num,           -- number 型で "Y" の項目を上書き，または nil.
  edge_style = str,  -- string 型で "境界処理" の項目を上書き，または nil.
}
```

####  ペンローズタイル ぼかしσの `PI`

```lua
{
  range = num,       -- number 型で "範囲" の項目を上書き，または nil.
  pattern = str,     -- string 型で "種類" の項目を上書き，または nil.
  center_form = str, -- string 型で "中心の形状" の項目を上書き，または nil.
  size = num,        -- number 型で "サイズ" の項目を上書き，または nil.
  deflate = num,     -- number 型で "再分割" の項目を上書き，または nil.
  rotate = num,      -- number 型で "回転" の項目を上書き，または nil.
  X = num,           -- number 型で "X" の項目を上書き，または nil.
  Y = num,           -- number 型で "Y" の項目を上書き，または nil.
  edge_style = str,  -- string 型で "境界処理" の項目を上書き，または nil.
}
```

####  ペンローズタイル レンズσの `PI`

```lua
{
  move = { { X1, Y1 }, { X2, Y2 } }, -- number 型で "移動*" の項目を上書き，{ X, Y } で一律指定，{ { X1, Y1 }, { X2, Y2 } } で各タイルごとに指定，または nil.
  rot = { rot1, rot2 },       -- number 型で "回転*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  zoom = { { X1, Y1 }, { X2, Y2 } }, -- number 型で "拡大率*" の項目を上書き，{ X, Y } で一律指定，{ { X1, Y1 }, { X2, Y2 } } で各タイルごとに指定，または nil.
  roundness = { rnd1, rnd2 }, -- number 型で "丸み*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  highlight = { hl1, hl2 },   -- number 型で "ハイライト*" の項目を上書き，table 型で各タイルごとに指定，または nil.
  light_angle = { la1, la2 }, -- number 型で "光源角度" の項目を上書き，table 型で各タイルごとに指定，または nil.
  back = { back1, back2 },    -- number 型で "余白幅" の項目を上書き，table 型で各タイルごとに指定，または nil.
  pattern = str,              -- string 型で "種類" の項目を上書き，または nil.
  center_form = str,          -- string 型で "中心の形状" の項目を上書き，または nil.
  size = num,                 -- number 型で "サイズ" の項目を上書き，または nil.
  deflate = num,              -- number 型で "再分割" の項目を上書き，または nil.
  rotate = num,               -- number 型で "回転" の項目を上書き，または nil.
  X = num,                    -- number 型で "X" の項目を上書き，または nil.
  Y = num,                    -- number 型で "Y" の項目を上書き，または nil.
  edge_style = str,           -- string 型で "境界処理" の項目を上書き，または nil.
}
```

- [「光源角度」](#光源角度)[「余白幅」](#余白幅-1)に関しては `PI` 経由でのみタイルごとに個別指定できます．

- `zoom` のフィールドに関しては，`{ X, Y }` の形式でそれぞれ `X`, `Y` 方向の拡大率を % 単位で個別に指定します．

  - [「拡大縦横比」](#拡大率1-拡大縦横比1-拡大率2-拡大縦横比2)を加味しての値で，「拡大縦横比」の直接指定はできません．

    例えば，「拡大率」が 150, 「拡大縦横比」が -40 に相当する指定は次の通りに計算します:

    1.  「拡大縦横比」が負なので X 方向はそのまま 150.
    1.  Y 方向は，100% との差を「拡大縦横比」分だけ縮める:

        $Y = 1+(X-1)(1-|r|)$ を $X=1.50, r = -0.40$ で計算して $Y=1.30$.

    1.  数値を % 単位に変換して `{ 150, 130 }` が対応する指定になる．
    1.  「拡大率」が負の場合は，その絶対値で計算してから X, Y ともに符号を反転する．

  - X, Y の符号が異なる指定も可能です (「拡大縦横比」との組み合わせでは不可能).

####  ペンローズタイル モザイクフェードσの `PI`

```lua
{
  bandwidth = num,   -- number 型で "変化幅" の項目を上書き，または nil.
  fade_time = num,   -- number 型で "フェード割合" の項目を上書き，または nil.
  shape = str,       -- string 型で "形状" の項目を上書き，または nil.
  shape_rot = num,   -- number 型で "形状回転" の項目を上書き，または nil.
  shape_orth = bool, -- boolean 型で "縦横比固定" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  file_image = num,  -- string 型で "遷移画像ファイル" の項目を上書き，または nil.
  reverse = num,     -- boolean 型で "反転" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  pattern = str,     -- string 型で "種類" の項目を上書き，または nil.
  center_form = str, -- string 型で "中心の形状" の項目を上書き，または nil.
  size = num,        -- number 型で "最大サイズ" の項目を上書き，または nil.
  deflate = num,     -- number 型で "分割回数" の項目を上書き，または nil.
  rotate = num,      -- number 型で "回転" の項目を上書き，または nil.
  X = num,           -- number 型で "X" の項目を上書き，または nil.
  Y = num,           -- number 型で "Y" の項目を上書き，または nil.
  antialias = num,   -- boolean 型で "アンチエイリアス" の項目を上書き，または nil. 0 を false, 0 以外を true 扱いとして number 型も可能．
  phase = num,       -- number 型でシーンチェンジの進捗を直接指定 (0.0 -- 1.0)，または nil.
}
```


##  TIPS

1.  このスクリプトで生成するペンローズタイルは，P2 型の “Sun” に対して deflation の操作を繰り返すことで生成されるものに限ります．ペンローズタイルはもっと一般に多くの，非可算無限個の組み合わせがありますが，その中でも限られたものしか生成されません．


##  既知の問題

1.  [ペンローズタイル ぼかしσ](#ペンローズタイル-ぼかしσ)で，[「サイズ」](#サイズ)と[「範囲」](#範囲)を大きく取り，対象オブジェクトの端のほうにタイルの一部分のみが小さく見えている状況だと，そのタイルのぼかし具合が正確に計算されず，階段状の見え方になることがあります．

##  改版履歴

- **v1.23 (for beta25)** (2025-12-21)

  - パラメタをグループ化して整理・並び替え．
  - フィルタオブジェクトとして動作するように設定．
  - 一部パラメタインジェクションに string 型も受け付けるように変更．
  - 「ペンローズタイル レンズσ」のパラメタ初期値を一部変更．
  - `beta25` での動作確認．

- **v1.22 (for beta20)** (2025-11-17)

  - 「ペンローズタイル レンズσ」でのシェーダー処理を一部簡略化．

  - `beta20` での動作確認．

- **v1.21 (for beta12)** (2025-09-21)

  - 「ペンローズタイル モザイクフェードσ」の「PI」の項目に `phase` フィールドを追加，シーンチェンジの進捗を直接指定できるように．

- **v1.20 (for beta12)** (2025-09-21)

  - 「ペンローズタイル モザイクフェードσ」で「形状を画像で指定」の項目を削除．
    - ファイル選択を解除する機能が追加されたことによって不要になったため．
    - それに伴って，パラメタインジェクションの `use_image` の項目も削除．

  - 「ペンローズタイル モザイクフェードσ」で「X」「Y」がアンカー操作できるように．

  - `beta12` での動作確認．

- **v1.10 (for beta11a)** (2025-09-18)

  - シーンチェンジの「ペンローズタイル モザイクフェードσ」を追加．

  - `beta11a` での動作確認．

- **v1.00 (for beta9)** (2025-09-01)

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
