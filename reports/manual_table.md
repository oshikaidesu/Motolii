# 手順書の表：やる事 × ソフト（11/1）

ユーザーの言葉：「手順書やユーザーマニュアルを作ろうって思った時に、まず何をすればいいのかっていうのを、一回表として作ってからの方がいいのかもしれないね。これをすれば各ソフトと何が違うのかっていうのは評価できるでしょう」。

各社をまたいで同じ作業を比べた公式の資料は見つからなかった。公式の 3 つを背骨にして、自分で表にする。

- 行：Adobe Certified Professional の試験範囲を背骨に、Apple Motion の目次、Alight Motion の画面、シナリオ P01〜P24 で足す
- 列：After Effects、Apple Motion、Alight Motion、Cavalry、Motolii
- 欄：最初の 1 手、手数、出典

## 数え方

- 始めの状態：物はある。何も選ばれていない。パネルは既定の並び
- 1 手：クリック、ドラッグ、キー、文字入力のどれか 1 回（golf_noise.md の決まり）
- 公式に書かれた道で、いちばん短い物を数える
- 実際に打ったのは Motolii だけ。ほかの 4 つは手順を読んで数えた
- 公式に無い手は「＋？」と書く

## 見本の 6 行

| # | やる事 | After Effects | Motion | Alight Motion | Cavalry | Motolii |
|---|---|---|---|---|---|---|
| ① | 新しい文書 | **2**＋？：新規コンポジションのキー・箱を閉じる | **4**：⌥⌘N・空白・Motion プロジェクト・開く | **2**：＋・箱で決める | **2**：Composition メニュー・New Composition | **2**：⌘N・Esc |
| ② | 文字を置いて打つ | **3**＋？：文字ツール・舞台をクリック・打つ | **4**：T・舞台をクリック・打つ・Esc | **4**：＋・Text・打つ・決定 | **3**＋？：文字ツール・舞台をクリック・打つ | **4**：T・舞台をクリック・打つ・Tab |
| ③ | 位置に 2 つキーを打って動かす | **6**：物を選ぶ・開く・Transform を開く・ストップウォッチ・時間を動かす・値を変える | **4**：物を選ぶ・◇・再生ヘッド・値を変える | **5**：物を選ぶ・移動のパネル・◇・再生ヘッド・値を変える | **4**：物を選ぶ・◇（⌥ で全軸）・再生ヘッド・値を変える | **5**：物を選ぶ・X の ◇・Y の ◇・定規・舞台をドラッグ |
| ④ | 名前の分かっている効果を載せる | **4**：物を選ぶ・⌘Return・打つ・Return | **4**：物を選ぶ・フィルタ・分類・種類 | **6**：物を選ぶ・効果・Add Effect・分類・効果・足す | **4**：物を選ぶ・Shape の札・Filters の＋・選ぶ | **5**：物を選ぶ・効果の棚・探す欄・打つ・ダブルクリック |
| ⑤ | 文字を 1 文字ずつ出す | **4**：物を選ぶ・探す欄・打つ・ダブルクリック（手組みは約 10） | **4**：物を選ぶ・ビヘイビア・テキストアニメーション・タイプオン | **約 10**：④の 6 手・Text Progress の End にキー | **4**：物を選ぶ・Create・Text Animation Presets・選ぶ | **5**：物を選ぶ・効果の棚・探す欄・打つ・ダブルクリック（打たない道は 6） |
| ⑥ | 動画に書き出す | **6**：コンポを選ぶ・Composition・Add to Render Queue・出力先・保存・Render | **3**＋？：⌘E・設定・保存 | **3**：書き出し・Video・Export | **4**：Window・Render Manager・＋ Current Composition・Render | **2**：Render・Render |
| | 合計 | 25＋？ | 23＋？ | 30 | 21＋？ | 23 |

合計は目安。AE と Cavalry の「＋？」は、手順のページが確定の手に触れていない分。

## 分かった差

1. **③ ◇ が軸ごと**：Motolii は位置の X と Y で別々に押す。X だけ押して舞台を動かしたら、Y にキーが入らず全体が動いた（11/1、窓で本当に外した）。Cavalry は ⌥ を押しながら ◇ で全軸に打てる。AE の位置はストップウォッチ 1 つ
2. **④⑤ 探す欄を押す 1 手**：Motolii は棚を開いても欄にキーが向かず、1 手多い（dock.dart と dock_effects.dart に autofocus も requestFocus も無い）。AE の Quick Apply は ⌘Return ですぐ打てる
3. **② 打つ場所が違う**：AE、Motion、Cavalry は舞台の上で打つ。Motolii は右の欄で打つ。手数は同じで、目の行き先が違う
4. **⑥ と ① は Motolii が最短に並ぶ**：Render 2 手は 5 つの中でいちばん短い。11/1 に書き出し、H.264・1920×1080・480 枚の table.mp4 を確かめた
5. **「新しい文書」の箱に決定の札が無い**：Alight Motion の箱には作る札がある。Motolii は閉じるだけ

## 表の外で出た事

- Animate all のスイッチが、選んだ物の名前の長さで左右に動く。TURN で x=1345、Text で 1337、IN で 1327。IN で押し損ねた
- Render の札も同じ（TURN 951、IN 944）。上の帯の真ん中が名前の長さで押されている
- 定規の上の灰色の帯は、押しても再生ヘッドが動かない。最初に外した

## 確かめた度合い

- Motolii ①：⌘N の代わりにメニューで開き、箱が出るところまで窓。閉じる Esc は top_bar.dart:511 で読んだだけ
- Motolii ②：T と舞台のクリックで欄が開き中身が選ばれるところまで 11/1 に窓で確かめた。私の道具は窓に字を打てないので、打って Tab は 10/29 の記録（golf_noise.md ⑩）
- Motolii ③：11/1 に窓で 2 つのキーと動きの線を確かめた
- Motolii ④：3 手（物・棚・ダブルクリック）は 11/1 に窓で通した。欄に打つ 2 手は数えただけ
- Motolii ⑤：打たない道 6 手は 10/30 の記録（scenarios.md P10）。打つ道 5 手は数えただけ
- Motolii ⑥：11/1 に窓で通した
- AE ①：キー表のページで項目は読めたが、キーの絵は読めていない
- AE ⑤：プリセットの手順は ops_sources の効果の行から。animate-text-presets 本文は未読
- Motion ③：◇ が位置の全軸に効くかは未確認
- Motion ⑥：最後の保存の手はページで未確認
- Alight Motion：④⑤⑥ は 10/25 の画面の記録と AbsentMotion から数えた。公式の手順の文章は無い（入門は動画 2 本で、中身は見ていない）
- 公式チュートリアルの動画は本文と字幕の中身まで読んでいない。読んだのはページの手順の文章だけ

## 全部の行（55 行）

Adobe 公式の欄は [ae_tutorials.txt](ops_sources/ae_tutorials.txt) の 144 本の slug。ACP は試験範囲の番号。見本は上の表。

| # | やる事 | ACP | Adobe 公式 | Motion の章 | シナリオ | 状態 |
|---|---|---|---|---|---|---|
| 1 | 新しい文書 | 2.1 | getting-started-after-effects | プロジェクト | | 見本① |
| 2 | 大きさと長さを変える | 2.1 | create-composition-animation | プロジェクト | P20 | |
| 3 | 保存して開き直す | 2.1 | getting-started-after-effects | プロジェクト | P24 | |
| 4 | 取り消す | | | プロジェクト | P22 | |
| 5 | 読み込む | 3.1 | getting-started-after-effects | メディア | | |
| 6 | 差し替える | 3.1 | find-relink-missing-files | メディア | P17 | |
| 7 | 切れた素材をつなぎ直す | 3.1 | find-missing-footage-fonts-aftereffects | メディア | | |
| 8 | 素材を整理する | 3.1 | organize-assets-projects-files | メディア | | |
| 9 | 図形 | 4.1 | create-shape-layers | シェイプ | | |
| 10 | 文字を置く | 4.2 | add-edit-animate-text | テキスト | P04 | 見本② |
| 11 | 文字の見た目 | 4.2 | aftereffects-create-text-elements-cc | テキスト | P13 | |
| 12 | ペンで形を描く | 4.1 | draw-complex-shapes | シェイプ | | |
| 13 | 手描き | 4.1 | animated-paint-brush-strokes | ペイント | | |
| 14 | 位置・大きさ・回転・アンカー | 3.2、4.3 | position-transform-layers | 変形 | | |
| 15 | 不透明度と重ね方 | 4.1 | understanding-layers-composition | レイヤー | | |
| 16 | 重ね順 | 3.2 | understanding-layers-composition | レイヤー | | |
| 17 | グループと親子 | 3.2 | animating-with-parenting、animate-layer-groups | レイヤー | | |
| 18 | マスクとマット | 4.1、5.3 | create-mask-pen-tool、alpha-channel | マスク | | |
| 19 | 整列 | 3.2 | position-transform-layers | 変形 | | |
| 20 | ガイドとスナップ | 2.2 | using-layer-snapping-aftereffects | | | |
| 21 | 帯を動かす・切る・分ける | 4.3 | create-composition-animation | タイミング | P18 | |
| 22 | 再生と範囲 | 2.2 | uninterrupted-preview | 再生 | | |
| 23 | 印 | 2.2 | | タイミング | P01、P02 | |
| 24 | 速さ | 4.4 | | タイミング | | |
| 25 | キーを打つ | 5.1 | keyframe-animation | キーフレーム | P07 | 見本③ |
| 26 | キーを動かす・消す | 5.1 | keyframe-animation | キーフレーム | | |
| 27 | イーズ | 5.1 | keyframe-animation | キーフレーム | P09 | |
| 28 | グラフ | 5.1 | adjusting-keyframes-dynamic-movement | キーフレーム | | |
| 29 | 動きの出来合い | 4.5 | animate-text-presets | ビヘイビア | | |
| 30 | 値をつなぐ | 5.2 | animating-with-expressions、link-properties | リグ | | |
| 31 | マスクを動かす | 5.3 | apply-effects-masks | マスク | | |
| 32 | 1 文字ずつ | 5.4 | animate-letter-by-letter | テキストアニメーション | P10 | 見本⑤ |
| 33 | 道に沿って動かす | 5.1 | animate-along-path | ビヘイビア | | |
| 34 | 繰り返し | 4.5 | | リプリケータ | | |
| 35 | 粒子 | 4.5 | add-rain-to-video | パーティクル | P06 | |
| 36 | 効果を載せる | 4.5 | add-video-special-effects | フィルタ | P08 | 見本④ |
| 37 | つまみを回す | 4.5 | add-video-special-effects | フィルタ | | |
| 38 | 効果を動かす | 5.2 | | フィルタ | P15 | |
| 39 | 効果の順番 | 4.5 | aftereffects-understanding-order-effect-ccs | フィルタ | | |
| 40 | 全体に掛ける | 4.5 | | フィルタ | P16 | |
| 41 | 効果を探す | 4.5 | | フィルタ | | |
| 42 | 奥行き | 4.4 | position-2d-layers-in-3d-space | 3D | P05 | |
| 43 | カメラ | 4.4 | dynamic-animation-with-3d-layers | 3D | P07、P09 | |
| 44 | ライト | 4.4 | add-environmental-lighting-to-3d-scene | 3D | | |
| 45 | 3D モデル | 4.4 | insert-objects-after-effects | 3D オブジェクト | | |
| 46 | 音を入れる | 3.1 | | オーディオ | | |
| 47 | 音に合わせる | | | オーディオ | P01 | |
| 48 | 動きを追う | 4.4 | animating-3d-objects-tracked-footage | トラッキング | | Motolii に無い |
| 49 | 手ぶれ補正 | 4.4 | stabilize-footage | トラッキング | | Motolii に無い |
| 50 | 色を抜く | | fix-green-screen-footage | キーイング | | |
| 51 | 動画に書き出す | 6.2 | render-video-composition | 共有 | | 見本⑥ |
| 52 | 1 枚書き出す | 6.2 | | 共有 | | |
| 53 | まとめて持ち出す | 6.1 | organize-assets-projects-files | | | |
| 54 | 窓の配置 | 2.2 | customize-workspaces | | | |
| 55 | キーを調べる | 2.2 | | ショートカット | | |

## 出典

- [Adobe Certified Professional：After Effects](https://certifiedprofessional.adobe.com/after-effects)
- [Adobe Learn の After Effects](https://www.adobe.com/jp/learn/after-effects)
- [AE キーフレーム](https://www.adobe.com/jp/learn/after-effects/web/keyframe-animation)
- [AE 文字](https://www.adobe.com/jp/learn/after-effects/web/add-edit-animate-text)
- [AE 1 文字単位](https://www.adobe.com/jp/learn/after-effects/web/animate-letter-by-letter)
- [AE レンダリング](https://www.adobe.com/jp/learn/after-effects/web/render-video-composition)
- [AE キー表](https://helpx.adobe.com/after-effects/desktop/get-started/keyboard-shortcuts/keyboard-shortcuts-reference.html)
- AE の効果の載せ方：[after_effects.txt](ops_sources/after_effects.txt) の 198〜203 行
- [Motion ユーザガイド目次](https://support.apple.com/ja-jp/guide/motion/toc)
- [Motion 新しいプロジェクト](https://support.apple.com/ja-jp/guide/motion/motnc01b5f0d/6.4/mac/26.6)
- [Motion キーフレーム](https://support.apple.com/ja-jp/guide/motion/motn1474a02c/6.4/mac/26.6)
- [Motion テキスト](https://support.apple.com/ja-jp/guide/motion/motn922c5fab/6.4/mac/26.6)
- [Motion フィルタ](https://support.apple.com/ja-jp/guide/motion/motn169f7ae5/6.4/mac/26.6)
- [Motion テキストビヘイビア](https://support.apple.com/ja-jp/guide/motion/motn1607d4f1/6.4/mac/26.6)
- [Motion タイプオン](https://support.apple.com/ja-jp/guide/motion/motn1607df7f/6.4/mac/26.6)
- [Motion 書き出し](https://support.apple.com/ja-jp/guide/motion/motn189cfcd6/6.4/mac/26.6)
- [Alight Motion Quick Start Guide](https://support.alightmotion.com/hc/en-us/articles/10536777320337-Alight-Motion-Quick-Start-Guide)
- [Alight Motion Text Progress](https://support.alightmotion.com/hc/en-us/articles/10537154109841-Text-Progress)
- [Alight Motion Effects Guide](https://support.alightmotion.com/hc/en-us/articles/10536991943569-Effects-Guide)
- Alight Motion の画面：[alight_ui.md](alight_ui.md)、キーの動き：AbsentMotion の `editor/EditorActions.kt:152`、文字：同じく addText
- [Cavalry Quick Start](https://cavalry.studio/docs/getting-started/quick-start/)
- [Cavalry Composition menu](https://cavalry.studio/docs/user-interface/menus/composition-menu/)
- [Cavalry Create menu](https://cavalry.studio/docs/user-interface/menus/create-menu/)
- [Cavalry Text tool](https://cavalry.studio/docs/user-interface/menus/tool-menu/text-tool/)
- [Cavalry Control Rows](https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-interaction/)
- [Cavalry Filters](https://cavalry.studio/docs/nodes/effects/filters/)
- [Cavalry Render Manager](https://cavalry.studio/docs/user-interface/menus/window-menu/render-manager/)
