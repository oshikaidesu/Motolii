# HexScript

AviUtl ExEdit2 向けに作ったスクリプト集です。設定ダイアログでは `HexScript` の下に並びます。

- 開発環境: AviUtl ExEdit2 2.1.12
- ライセンス: [MIT-0](LICENSE)
- 使い方は [docs](docs/) の各ファイルにあります

## 導入

1. [Releases](https://github.com/HexBrowns/HexScript/releases) から `HexScript_v*.zip` をダウンロードして展開する
2. 使用したいスクリプトを `Script` から取り出し、AviUtl2 の `Script` フォルダへコピーする。
3. プリセットが必要であれば、`Preset`・`Default`・`Alias` の中身を、AviUtl2 のデータフォルダの同名のフォルダへコピーする
4. AviUtl2を起動する。

**ファイル名の先頭の `@` は消さないでください。** 1 つのファイルに複数のスクリプトが入っているものは、`@` で始まる名前でないと分割されません。

### 一緒に置くファイル

| スクリプト                        | 一緒に置くもの                                              |
| ---------------------------- | ---------------------------------------------------- |
| `@Filters_H.anm2`            | `EffectUtils_Noise.lua` `EffectUtils_ColorGrade.lua` `EffectUtils_Budget.lua`。タイムディスプレイスメントを使うなら `TimeDisplace_H` のフォルダごと |
| `プロシージャル模様_H.obj2`           | `EffectUtils_ColorGrade.lua` `EffectUtils_ColorSpace.lua` |
| `@ローポリ化.anm2` `@ローポリ背景.obj2` | `EffectUtils_Lowpoly.lua` `EffectUtils_ColorSpace.lua` |
| `@拡張パーティクル_H.anm2`           | `Particle_H.lua` `EffectUtils_ColorSpace.lua`          |
| `カラフルランダム配置_H.obj2` `カラフルパーティクル_H.obj2` | 色空間を HSV / Oklch にするなら `EffectUtils_ColorSpace.lua` |
| `@キーイング_H.anm2`             | 同型塗りつぶしを使うなら `SameShapeFill_H` のフォルダごと。色差領域塗りつぶしを使うなら σ軸さんの Region_S（HexScript には入っていない） |
| `@ShapeDistort_H.anm2`       | パペットを使うなら `Puppet_H` のフォルダごと |
| `@キーイング_H.anm2` `@2Dライティング_H.anm2` `ビーム化_H.anm2` `光束_H.anm2` `ベクトルブラー_H.anm2` `押し出し_H.anm2` `ボールアクション_H.anm2` `コンテンツに応じた塗りつぶし_H.anm2` | `EffectUtils_Budget.lua`（cache と tempbuffer の枠の見積もり） |

## 収録スクリプト

### 画面全体・色

| 見本                                                                    | ファイル               | 内容                                                                                                                                                                                  | 説明書                              |
| --------------------------------------------------------------------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| <img src="images/HexBrowns.ColorSpaceBlend_H_thumbnail.png" width="160"> | `色空間合成_H.anm2` | 下の絵と合成する。色相・カラー・彩度・輝度を Oklch / LCh で（明るさの並びが崩れない）、通常・加算・スクリーン・乗算・比較(明・暗)をリニア（光の強さ）で。extra_blend_mode.anm2（sevenc-nanashi、MIT）のフォーク | [docs](docs/色空間合成_H.md) |
| <img src="images/HexBrowns.ColorAndTone_H_thumbnail.png" width="160"> | `@色と階調_H.anm2` | 色と階調の処理 4 種（ディザ＝減色の格子模様・誤差拡散、版ずれ網点＝CMYK 4 版の網点と版ずれ、白飛び＝露出を上げてトーンマッピングで戻す、彩度圧縮＝鮮やかすぎる色だけを開始から上限へなめらかに抑える） | [docs](docs/色と階調_H.md) |
| <img src="images/HexBrowns.Filters_H_thumbnail.png" width="160">      | `@Filters_H.anm2`  | フィルム調の加工 29 種（グリッチ、グレイン、ハレーション、ライトリーク、ビネット、アナモルフィックフレア、リキッドディストーション、シマー、フォグヘイズ、フィルムバーン、カラーグレード、リフト・ガンマ・ゲイン、バレル歪み、ノイズオーバーレイ、バイブランス、HSL選択補正、LUT適用、色調転写、局所トーン補正、ローカルラプラシアン、暗所視、ホログラム箔、多面体ガラス、レンズの汚れ、古いフィルム、自動レベル補正、美肌、タイムディスプレイスメント＝場所ごとに何フレーム前の絵を出すかを変える、色ノイズ） | [docs](docs/Filtersフィルタ使用ガイド.md) |
| <img src="images/HexBrowns.OpticalBokeh_thumbnail.png" width="160">   | `@光学系ボケ.anm2`      | レンズで撮ったようなボケ 5 種（玉ボケ、レンズぼかし、チルトシフト、球面収差ボケ、マップぼかし＝深度マップなどの明るさで場所ごとにぼかしの量を変える）                                                                                                                                          | [docs](docs/光学系ボケ.md)            |
| <img src="images/HexBrowns.Keying_H_thumbnail.png" width="160">       | `@キーイング_H.anm2`    | 背景を抜いた素材の仕上げ 7 種（マット調整、スピル除去、トラックマット、動体抽出＝前のフレームとの差で動いた所だけ残す / 同型塗りつぶし＝同じ形の領域を全部見つけて塗る / 光の回り込み＝背景の光を縁へにじませてなじませる / 色差領域塗りつぶし＝Region_S の色領域塗りつぶしに色差を足したフォーク）                                                                                                                       | [docs](docs/キーイング_H.md)          |
| <img src="images/HexBrowns.Lighting2D_H_thumbnail.gif" width="160">   | `@2Dライティング_H.anm2` | 平たい素材に立体の光と陰を足す 3 種（大域照明＝明るい所を光源にして、物で遮りながら光を広げ影を落とす / 環境遮蔽＝くぼみ・溝・接触部に陰を落とす / 光の帯＝表面をなめる光沢の帯）                                                                                                        | [docs](docs/2Dライティング_H.md)       |
| <img src="images/HexBrowns.RangeWarning_H_thumbnail.png" width="160"> | `レンジ警告_H.anm2`     | 白飛び・黒潰れ・色域外の画素をゼブラで示す検査用の効果                                                                                                                                                         | [docs](docs/レンジ警告_H.md)          |

### 変形・配置

| 見本                                                                    | ファイル                         | 内容                                    | 説明書                            |
| --------------------------------------------------------------------- | ---------------------------- | ------------------------------------- | ------------------------------ |
| <img src="images/HexBrowns.ShapeDistort_H_thumbnail.png" width="160"> | `@ShapeDistort_H.anm2`       | 画像をゆがませる 6 種（回転、放射、方向、ジグザグ、無限ズーム＝入れ子の自分へ終わりなく寄り続ける / パペット＝ピンを動かして絵を曲げる）           | [docs](docs/ShapeDistort_H.md) |
| <img src="images/HexBrowns.Lowpoly_thumbnail.png" width="160">        | `@ローポリ化.anm2` `@ローポリ背景.obj2` | オブジェクトのローポリ化と、ローポリの背景                 | [docs](docs/ローポリ.md)           |
| <img src="images/HexBrowns.VerticalText_thumbnail.png" width="160">   | `テキスト縦書き化.anm2`              | 横書きのテキストを、文字ごとのオブジェクトのまま縦書きに並べる       | [docs](docs/テキスト縦書き化.md)       |
| <img src="images/HexBrowns.AutoFitText_H_thumbnail.png" width="160">  | `枠内自動フィット_H.obj2`            | 指定した枠に収まる最大のフォントサイズで文字を出す。自動改行・禁則処理つき | [docs](docs/枠内自動フィット_H.md)     |

### 光・粒・模様

| 見本                                                                         | ファイル               | 内容                                                                                                 | 説明書                         |
| -------------------------------------------------------------------------- | ------------------ | -------------------------------------------------------------------------------------------------- | --------------------------- |
| <img src="images/HexBrowns.LightPath_H_thumbnail.gif" width="160">         | `光線経路_H.obj2`      | 鏡で反射し、プリズムで分光する光線の経路を描く                                                                            | [docs](docs/光線経路_H.md)      |
| <img src="images/HexBrowns.LightShaft_H_thumbnail.gif" width="160">        | `光束_H.anm2`        | オブジェクトの形から光の筋を伸ばす                                                                                  | [docs](docs/光束_H.md)        |
| <img src="images/HexBrowns.Beam_H_thumbnail.gif" width="160">              | `ビーム化_H.anm2`      | 形を、白い芯と色の付いた光のビームに描き替える                                                                            | [docs](docs/ビーム化_H.md)      |
| <img src="images/HexBrowns.FineParticles_H_thumbnail.gif" width="160">     | `微粒子_H.obj2`       | ほこり・浮遊物・細かい雪などの粒を流す                                                                                | [docs](docs/微粒子_H.md)       |
| <img src="images/HexBrowns.Bubbles_H_thumbnail.gif" width="160">           | `泡_H.obj2`         | 泡を描く                                                                                               | [docs](docs/泡_H.md)         |
| <img src="images/HexBrowns.ProceduralPattern_H_thumbnail.png" width="160"> | `プロシージャル模様_H.obj2` | 雲・縞・セル・六方格子・反応拡散から模様を作る（プリセット 22 本）                                                                | [docs](docs/プロシージャル模様_H.md) |
| <img src="images/HexBrowns.Grid_H_thumbnail.gif" width="160">              | `グリッド_H.obj2`      | 線の網目や、図形を等間隔に並べた模様                                                                                 | [docs](docs/グリッド_H.md)      |
| <img src="images/HexBrowns.DecoParticle_H_thumbnail.gif" width="160">      | `装飾パーティクル_H.anm2`  | オブジェクトの範囲に粒を出しては消す（アスさんの `装飾パーティクル` の軽量化フォーク版）                                                     | [docs](docs/装飾パーティクル_H.md)  |
| <img src="images/HexBrowns.ExtendedParticle_H_thumbnail.gif" width="160">  | `@拡張パーティクル_H.anm2` | 画像を粒子として放出する。風・力場・パス・跳ね返り・子粒子・軌跡・光源・pmd モデルなど 32 種類の拡張を積んで組み合わせる。（「拡張パーティクル(R)」をAviUtl2向けに作り直したもの） | [docs](docs/拡張パーティクル_H.md)  |

### After Effects の効果に当たるもの

After Effectsの効果を参考にAviUtl2向けに作ったものです。後のカッコがAfter Effectsでの効果の名称になります。

| 見本                                                                        | ファイル                    | 内容                                                       | 説明書                              |
| ------------------------------------------------------------------------- | ----------------------- | -------------------------------------------------------- | -------------------------------- |
| <img src="images/HexBrowns.RadioWaves_H_thumbnail.gif" width="160">       | `電波_H.obj2`             | 中心から広がる円・多角形・星の波を描く（電波）                                  | [docs](docs/電波_H.md)             |
| <img src="images/HexBrowns.VectorBlur_H_thumbnail.png" width="160">       | `ベクトルブラー_H.anm2`        | 画像の輪郭の向きに沿ってぼかす。別のレイヤーを流れの地図にもできる（ベクトルブラー）               | [docs](docs/ベクトルブラー_H.md)        |
| <img src="images/HexBrowns.CardDance_H_thumbnail.gif" width="160">        | `カードダンス_H.anm2`         | 画像をカードに分け、横・縦・放射・別のレイヤーの明るさの順に、裏返す・飛ばす・消す（カードダンス・カードワイプ） | [docs](docs/カードダンス_H.md)         |
| <img src="images/HexBrowns.BallAction_H_thumbnail.gif" width="160">       | `ボールアクション_H.anm2`       | 画像を光の当たったボールの集まりにして、散らす・ねじる・揺らす（CC Ball Action）          | [docs](docs/ボールアクション_H.md)       |
| <img src="images/HexBrowns.Caustics_H_thumbnail.gif" width="160">         | `コースティクス_H.anm2`        | 揺れる水面越しに見たように、屈折させて底に光の網目を描く。波紋も出せる（ウェーブワールド＋コースティクス）    | [docs](docs/コースティクス_H.md)        |
| <img src="images/HexBrowns.TimeWarp_H_thumbnail.gif" width="160">         | `タイムワープ_H.obj2`         | 動画を好きな速さで再生し、フレームの間を動きから補間する。なめらかなスロー（タイムワープ）            | [docs](docs/タイムワープ_H.md)         |
| <img src="images/HexBrowns.ContentAwareFill_H_thumbnail.png" width="160"> | `コンテンツに応じた塗りつぶし_H.anm2` | 範囲の中を周りの絵で埋めて、写り込んだ物を消す（コンテンツに応じた塗りつぶし）                  | [docs](docs/コンテンツに応じた塗りつぶし_H.md) |
| <img src="images/HexBrowns.Extrude_H_thumbnail.gif" width="160">          | `押し出し_H.anm2`           | 文字や図形を奥行き方向に押し出し、縁に面取りか丸みを付ける（押し出し・ベベル）                  | [docs](docs/押し出し_H.md)           |
| <img src="images/HexBrowns.PlanarTracking_H_thumbnail.gif" width="160">   | `平面トラッキング_H.obj2`       | 動画の中の平らな面を追い、別のレイヤーの画像を四隅に透視で貼る（平面トラッカー＋コーナーピン）          | [docs](docs/平面トラッキング_H.md)       |

### 図表・効果の呼び出し

YMM4のプラグインの説明を元にAviUtl2向けに作ったものです（コードは使っていません）。後のカッコがYMM4でのプラグインの名称になります。

| 見本 | ファイル | 内容 | 説明書 |
|---|---|---|---|
| <img src="images/HexBrowns.Charts_H_thumbnail.png" width="160"> | `@図表_H.obj2` | テキストを書くだけで、表・関数のグラフ・ツリー図を描く 3 種（表テーブル・簡易関数グラフ・ツリー・階層図プラグイン） | [docs](docs/図表_H.md) |
| <img src="images/HexBrowns.EffectCall_H_thumbnail.gif" width="160"> | `@効果呼び出し_H.anm2` | ほかの効果を名前で指定して掛ける 2 種（ランダム＝候補から選んで掛ける / 登場退場＝どの効果でも登場・退場の演出にする）（エフェクトランダム切り替え・登場退場（汎用）） | [docs](docs/効果呼び出し_H.md) |

### カメラ・シーン

| 見本                                                                      | ファイル                     | 内容                                      | 説明書                              |
| ----------------------------------------------------------------------- | ------------------------ | --------------------------------------- | -------------------------------- |
| <img src="images/HexBrowns.HandheldCamera_H_thumbnail.gif" width="160"> | `手持ちカメラ_H.cam2`          | 手持ち撮影のような揺れ                             | [docs](docs/手持ちカメラ_H.md)         |
| <img src="images/HexBrowns.SceneTransitions_thumbnail.gif" width="160"> | `@SceneTransitions.scn2` | シーンチェンジ 3 種（スマッシュカット、ズームトランジション、パララックス） | [docs](docs/SceneTransitions.md) |

### 制作補助

| 見本 | ファイル | 内容 | 説明書 |
|---|---|---|---|
| <img src="images/HexBrowns.TestPattern_H_thumbnail.png" width="160"> | `検証パターン_H.obj2` | 階調・色相・肌色などのテストパターン 10 種 | [docs](docs/検証パターン_H.md) |

### 改造（元作品の作者またはライセンスが不明）

元作品の部分の権利は元作者にあります。元作品は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) にまとめてあります。

| ファイル | 内容 | 説明書 |
|---|---|---|
| `@KD_DepthMap_H.anm2` | 深度画像で、オブジェクトを奥行きで切り抜く | [docs](docs/KD_DepthMap_H.md) |
| `@カメラ効果_H.cam2` | カメラを回す・動かす・目標をずらす・傾ける・寄る | [docs](docs/カメラ効果_H.md) |
| `テキスト両端揃え_H.anm2` | テキストの各行を両端揃えに並べ直す | [docs](docs/テキスト両端揃え_H.md) |
| `キラキラ_H.anm2` | オブジェクトを粒にして放射状に弾け飛ばす | [docs](docs/キラキラ_H.md) |
| `カラフルパーティクル_H.obj2` | 図形の粒を放射状に飛ばす | [docs](docs/カラフルパーティクル_H.md) |
| `カラフルランダム配置_H.obj2` | 図形の粒を画面にばらまく | [docs](docs/カラフルランダム配置_H.md) |
| `円形変位_H.anm2` | 今の位置を中心とした円の上へ動かす | [docs](docs/円形変位_H.md) |
| `ランダム2_H.anm2` | 位置・回転・拡大率・透明度をオブジェクトごとにずらす | [docs](docs/ランダム2_H.md) |
| `偽被写界深度_H.anm2` | カメラからの距離に応じてぼかす | [docs](docs/偽被写界深度_H.md) |
| `円形配置プラス.anm2` | 円（楕円）の周に並べる | [docs](docs/円形配置プラス.md) |
| `sin揺れ_HB.anm2` | 帯ごとにずらした正弦波で揺らす | [docs](docs/sin揺れ_HB.md) |
| `@図形グラデーション_H.anm2` | 形はそのままに、中身をグラデーションで塗り替える | [docs](docs/図形グラデーション_H.md) |
| `レベル補正_H.anm2` | 黒点・白点・中間調と出力の範囲で明るさを整える | [docs](docs/レベル補正_H.md) |
| `個別遅延_H.anm2` | テキストの文字などの個別オブジェクトを 1 つずつ遅らせて動かす。遅れにベジェで緩急を付けられ（配り方）、最後の中間点で止まったものから消せる | [docs](docs/個別遅延_H.md) |

## フォーク

元作品が GitHub で公開されているものは、元のリポジトリのフォークに置いています。

| ファイル                                          | 置き場所                                                                                        | 元作品                                             |
| --------------------------------------------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| `ロングシャドー_H.anm2` `トーンカーブ_H.anm2`              | [HexBrowns/aviutl2-scripts](https://github.com/HexBrowns/aviutl2-scripts)                   | `こんにゃくねこ` さんの LongShadow / Tonecurve_C（CC0-1.0） |
| `ブラインドループ_H.anm2`                             | [gist のフォーク](https://gist.github.com/HexBrowns/ba9203470e0fad888421a76760afb417)            | `zopty` さんのブラインドループ（BSD-3-Clause）               |
| `モザイク_H.anm2`                                 | [HexBrowns/aviutl2_script_Pixelizer](https://github.com/HexBrowns/aviutl2_script_Pixelizer) | `nctype` さんの Pixelizer（MIT）                     |
| `多色グラデーション_H.anm2`                            | [HexBrowns/AviUtl2-GradientPlus](https://github.com/HexBrowns/AviUtl2-GradientPlus)         | `azurite` さんのグラデーション+（CC0-1.0）                  |
| `@連結ライン_Hex.anm2`                             | [HexBrowns/aviutl2_script_Path_S](https://github.com/HexBrowns/aviutl2_script_Path_S)       | `σ軸` さんの Path_S（MIT）                            |
| `放射分布_H.anm2` `オートターゲット_H.cam2` `座標公開_H.anm2` | [HexBrowns/ported_tim](https://github.com/HexBrowns/ported_tim)                             | `ティム` さんのスクリプト                                  |
