# 動画にしか無い操作の照らし合わせ（10/25）

キーフレーム、イーズ、時間、マーカー、再生、カメラ・3D、効果の重ね順、値のつなぎ、レイヤーとマスク、セルを、動画・アニメのツールから集めて Motolii に当てはめる。印は ableton_map.md と同じ（同じ・違う・無い・外す）。

**お手本の順番**（Adobe は古い仕組みを避けるための操作が多いので、比べるだけ）：
1. 新しいもの：Cavalry、Blender、Jitter、Rive、Theatre.js、Spline、Ableton（第三者の記事も）、Figma、クリスタ、DaVinci Resolve、Final Cut Pro
2. 比べるだけ：After Effects、Photoshop、Premiere、Illustrator

行の「どこ」に **Adobe だけ** とあるものは、新しいツールのどれにも無い。古い縛りの回避を疑う。

集めた数（どれも公式。Ableton の第三者だけは記事を公式と 1 つずつ照らした）：

| お手本 | 行 | 出典 |
|---|---|---|
| Cavalry | 191 | https://cavalry.studio/docs/ （旧 docs.cavalry.scenegroup.co）78 ページ |
| Blender | 224 | https://docs.blender.org/manual/en/latest/ （5.2）49 ページ |
| Jitter・Rive・Theatre.js・Spline | 165 | 各公式。一覧は ops_sources/jitter_rive_theatre_spline.txt |
| クリスタ・Photoshop | 242 | help.clip-studio.com 42 ページ、helpx.adobe.com/photoshop 59 ページ。一覧は ops_sources/clipstudio_photoshop.txt |
| Resolve・Final Cut・Premiere | 133 | Resolve 21 リファレンスマニュアル（PDF）、support.apple.com/guide/final-cut-pro、helpx.adobe.com/premiere |
| Ableton の第三者の記事 | 約 100 | 17 の出版元（EDMProd、MusicTech、Bonedo、Splice、Ableton 公式掲示板ほか）。公式と照らして 公式にもある 69・公式に無い 15・食い違い 17 |
| After Effects（比べるだけ） | 231 | helpx.adobe.com/after-effects 33 ページ。一覧は ops_sources/after_effects.txt |
| **計** | **約 1,290** | |

公式どうしの食い違い（その操作はお手本にしない）：After Effects 11 か所（カメラの道具が 2 世代並ぶ、回転の刻み 15° と 5°、ベジェの分け方、ホイールの意味が逆 など）、Jitter 3 か所、クリスタ 2 か所（選択解除と再選択のキーがページで逆）、Final Cut 1 か所。

## 数

| 印 | 行 |
|---|---|
| 同じ | 17 |
| 違う | 26 |
| 無い | 81 |
| 外す | 5 |

## 考え方の違い（先に決めること）

新しいツールは、Motolii（After Effects 流）と考え方が違う所がある。1 行ずつ直す前に、ここを決める。

| 何 | Motolii いま（出典） | 新しいツール | 印 |
|---|---|---|---|
| キーがいつ打たれるか | キーのある物を変えると自動でキー（After Effects） | **モードで分ける**：Design と Animate を Tab や M で切り替え、Animate の時だけ変えるとキー（Rive、Jitter）。Blender・Cavalry は Auto Key を自分で入れる | 違う |
| キーかどうかの見え方 | インスペクタの菱形 | **値の色**：このコマにキー＝黄、ほかのコマ＝緑、キーから変えた＝橙、つながっている＝紫（Blender）。菱形 3 状態（Cavalry、Rive） | 違う |
| 値の上でキーを打つ | 菱形を押す・右クリック | 値に乗って **I** で打つ、**⌥I** で消す、右クリックでも（Blender） | 無い |
| キーを持たない動き | なし | **操作を足す**：物に「終わりの値・長さ・イーズ」の区間を足していく、行は物でなく操作（Jitter） | 無い |
| 値と値のつなぎ | 関係（Around・Row など）、親の一覧、Relate…（右クリック） | **出っ張りをドラッグしてつなぐ**、⌥ で選んだ全部に、⌘ で置き換え（Cavalry）。右クリック → Data Bind（Rive）。右クリック → Add Behaviour で揺れやノイズを付ける（Cavalry）。⌃D でドライバ（Blender） | 違う |
| キーの上に動きを重ねる | なし | **F カーブのモディファイア**（ノイズ、繰り返し、段々）をキーの上に積む（Blender）。Animation Offset（Cavalry）。キーフレームレイヤー（Cavalry） | 無い |
| 効果の重ね順 | 効果のカードを長押しでドラッグ | **つかみをドラッグ**、オン・オフ、焼く（Apply）、選んだ全部にコピー、⌥ で選んだ全部に（Blender のモディファイア・コンストレイント）。⌥ で同じ操作をほかの全部に（Ableton） | 違う |
| まとまり | 親子と Group の関係 | グループ（Cavalry ⌘G、Jitter はアニメーションの区間もまとめる）。プリコンポーズは After Effects の縛りの回避（**Adobe だけ** の形） | 違う |

## 1. キーを打つ・選ぶ・動かす

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 値の横の菱形を押す → キー（3 つの状態の見た目） | Cavalry・Rive・After Effects | 菱形を押すとキー・もう一度で消す | 同じ |
| ⌥ を押して菱形 → x/y/z を全部 | Cavalry | なし | 無い |
| ⌘ を押して菱形 → このコマのキーを消す | Cavalry | 同じ菱形をもう一度 | 違う |
| 右クリック → このキー・このアニメーションを消す | Cavalry・Blender・Theatre.js | 同じ | 同じ |
| 右クリック → Sequence（この値を動かせるようにする）／Make static | Theatre.js | なし（菱形で始まる） | 違う |
| キーを押す → 選ぶ、⇧ で足す、囲んで選ぶ | 全部 | 同じ | 同じ |
| ⌥ ドラッグで囲む → 投げ縄で選ぶ | Cavalry・Blender（⌃右ドラッグ） | なし | 無い |
| 再生位置の前・後ろを全部選ぶ（[ ]） | Blender | なし | 無い |
| 同じ時刻の列を全部選ぶ（K） | Blender | なし | 無い |
| キーをドラッグ → 時間を動かす | 全部 | 同じ | 同じ |
| ⌥ ドラッグ → キーを複製 | Cavalry・クリスタ | なし | 無い |
| 選んだキーの端を ⌥ ドラッグ → 間隔を伸び縮み | Rive・Cavalry（箱）・After Effects | なし | 無い |
| 箱（Transform）で選んだキーを移動・伸縮 | Cavalry・Blender（S）・クリスタ | なし | 無い |
| = / − でキーを 1 コマずらす、⇧ で 5 コマ | Cavalry（Rive は ⌥, ⌥.） | なし | 無い |
| ⌘C → 再生位置を動かす → ⌘V（いちばん早いキーが再生位置に、別の物にも） | Cavalry・Jitter・Theatre.js・Blender | なし | 無い |
| イーズだけコピーして貼る | Cavalry・Jitter | なし | 無い |
| キーに色 | Cavalry | なし | 無い |
| キーをダブルクリック → 値を打つ窓 | Cavalry・Theatre.js（クリック） | なし | 無い |
| キーを右クリック → 値をデータにつなぐ | Rive | なし | 無い |
| 揃える（左・中・右） | Cavalry | なし | 無い |
| 逆にする（Reverse Animation） | Cavalry・Blender（⌃M 鏡） | なし | 無い |
| 焼く（Bake：毎コマにキー） | Cavalry・Blender | なし | 無い |
| 間引く（Decimate・Clean） | Blender | なし | 無い |
| 選んだ物を ⌘⌥ でずらして並べる（Stagger） | Jitter・Rive・After Effects（Quick Offset） | 関係の Group で下から順に出る | 違う |
| 物の行をたたんだ時のまとめのキー → 中のキーが一緒に動く | Rive・Theatre.js | なし | 無い |
| キーの種類（Keyframe・Breakdown・Extreme…）の印 | Blender | なし | 無い |
| 次・前のキーへ（Up/Down、⌘⌥←→） | Blender・Cavalry・Rive | なし | 無い |

## 2. イーズとグラフ

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| プリセットを選ぶ、**強さのつまみ 1 本** で効き具合 | Jitter | プリセットの札（強さは無い） | 違う |
| ベジェの取っ手 2 つをドラッグ、4 つの数を打つ・貼る | Rive・Jitter・Theatre.js | 取っ手をドラッグ、数をこする | 同じ |
| CSS としてコピー | Jitter | なし | 無い |
| 名前で探す（linear、quad-in-out） | Theatre.js | なし | 無い |
| Hold・Linear・Cubic・Elastic（行き過ぎの大きさと収まる速さ）・Spring | Rive・Spline・Blender（Back・Bounce・Elastic） | Linear・Ease・In・Out・Back・Hold | 違う |
| 式でイーズ（Magic Easing） | Cavalry | 文書のイーズを数で | 違う |
| 新しいキーの既定のイーズを決める | Rive | なし | 無い |
| 影響（%）と速度の数 | **Adobe だけ**（After Effects） | 表示している | 違う |
| グラフの上でキーを上下左右に動かす（時間と値） | Cavalry・Blender・クリスタ | なし（イーズのパネルは 1 区間だけ） | 無い |
| ⌥ クリックで Linear ↔ Bezier、⌥ ドラッグで取っ手を出す・分ける | Cavalry・Blender | なし | 無い |
| 曲線をダブルクリック → キーを足す | Cavalry・Blender（I） | なし | 無い |
| 区間をドラッグ → 両側の取っ手が動く | Cavalry | なし | 無い |
| 複数の行の曲線を重ねて見る（色分け） | Theatre.js・Blender・Cavalry | 関係の後ろの物は薄い曲線（Blender） | 違う |
| ゴースト（前の曲線を残して見比べる）・A/B | Blender・Cavalry | なし | 無い |
| 繰り返し（前後の Loop・Cycle・Ping-pong） | Cavalry・Blender（⇧E）・Rive | なし | 無い |
| 隣のキーと混ぜる・押し引き（⌥D の混ぜ方） | Blender | なし | 無い |
| F / Home → 選んだ所に合わせる、全部 | Cavalry・Blender | なし | 無い |
| 正規化（−1〜1 に揃えて見る） | Blender | なし | 無い |
| 時刻を細かくずらしてキーを打てる・打てない | Rive（Snap Keys 60 分割） | コマ単位 | 外す |

## 3. 時間（帯・切り詰め・範囲）

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 帯の端をドラッグ → 長さ、⌥ で同じコマの帯を全部 | Cavalry | 端をドラッグ（⌥ は無い） | 違う |
| [ ] → 帯の始まり・終わりを再生位置へ（帯ごと動く）、⌥[ ⌥] → 切り詰め | Cavalry・After Effects | なし | 無い |
| 再生位置で帯を切る | Cavalry（右クリック）・Blender K・Ableton ⌘E・Final Cut ⌘B | なし | 無い |
| 帯をつなぐ（Merge・Consolidate） | Cavalry・Ableton ⌘J | なし | 無い |
| 帯の中身だけずらす（スリップ） | Ableton ⇧⌥ドラッグ・Blender S・Resolve・Final Cut | なし | 無い |
| 帯の端を ⌥ ドラッグ → 伸ばして速さを変える | クリスタ・Premiere R | なし（速さは値だけ） | 無い |
| リップル（後ろを詰める）・ロール（境目だけ） | Resolve・Final Cut・Blender（E） | なし | 無い |
| すき間を詰める・差し込む | Blender（Backspace・⇧=）・Ableton（⌘I・⇧⌘⌫） | なし | 無い |
| 帯どうしを入れ替える | Resolve ⌘⇧ドラッグ・Blender ⌥←→ | なし | 無い |
| 帯をミュート・鍵 | Blender H・⌃H | 目・鍵（段ごと） | 同じ |
| 区間をまとめる（Group）→ 端を引くと中身が比例で伸びる | Jitter | なし | 無い |
| 範囲を B / N で再生位置に | Cavalry・After Effects | 端をドラッグだけ | 無い |
| 範囲の外を書き出さない・範囲を回す | 全部 | 同じ | 同じ |
| ⇧ を押して再生位置をドラッグ → 10 コマ・マーカーに吸着 | Cavalry・Blender（⌃）・After Effects | なし | 無い |
| Q を押してどこでもドラッグ → 時間をこする | Cavalry | なし | 無い |
| 時刻表示を押す → コマ数と時刻の切り替え | Cavalry・Blender ⌃T | なし | 無い |
| J・K・L で速さを変えながら見る | Resolve・Final Cut・Premiere（3 つとも同じ） | なし | 無い |

## 4. マーカー

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| M → 再生位置にマーカー（再生中も） | Blender・Resolve・Final Cut・Premiere | ダブルクリックで足す | 違う |
| マーカーをドラッグ → 動かす | 全部 | 同じ | 同じ |
| ダブルクリック → 名前・色・長さの窓 | Cavalry | ダブルクリックで名前 | 違う |
| マーカーにキーや帯の端をつなぐ → マーカーを動かすと付いてくる | Cavalry | なし | 無い |
| マーカーの番号キー（1〜9、< >）で飛ぶ | Cavalry | なし | 無い |
| マーカーでカメラを切り替える | Blender ⌃B | なし | 無い |
| マーカーに長さ（範囲） | Cavalry・Resolve・After Effects | なし | 無い |
| 拍の目盛り（Pacing Markers：秒か BPM） | Cavalry | なし（MIDI の BPM は物の speed） | 無い |

## 5. 再生

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| Space → 再生・停止 | 全部（Ableton は始まりから、⇧Space で止めた所から） | 今の位置から | 同じ |
| 逆再生（⇧⌃Space） | Blender | なし | 無い |
| 範囲だけ・選んだ操作だけ回す | Jitter・Rive（Work Area） | 作業範囲 | 同じ |
| 再生の種類（1 回・ループ・往復） | Rive・Blender | ループの切り替え | 違う |

## 6. カメラと 3D

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| カメラ越しに見る・自由に見る | Blender Numpad0・Spline | ステージのカメラ／自由 | 同じ |
| 今の見え方にカメラを合わせる | Blender ⌃⌥Numpad0 | なし | 無い |
| カメラを見え方に縛る（見回すとカメラが動く） | Blender | なし | 無い |
| 見回すのをキーで記録 → カメラの道 | Blender（歩く・飛ぶ＋自動キー） | なし | 無い |
| ピントをスポイトで取る、ピントの十字をドラッグ | Blender | 深さのパネルで板を押す・線を上下 | 違う |
| 焦点距離・絞り・距離 | 全部 | 同じ | 同じ |
| 中ボタンで回す・⇧で動かす・ホイールで寄る | Blender | 関係のパネルだけ（ドラッグで回す） | 違う |
| ⌥ ドラッグで回す、Space ドラッグで動かす | Spline | なし | 無い |
| 軸の玉を押す → その向き | Blender・Spline | 関係のパネルで同じ（Blender） | 同じ |
| G・R・S のあと X・Y・Z で軸、数を打つ | Blender | なし | 無い |
| ギズモ：矢印で 1 軸、四角で 2 軸、輪で回す | Blender・After Effects | なし（ステージは 2D の取っ手） | 無い |
| 2.5D：物に Z を足す、カメラの道具（⌥ で横、⇧ で奥、⌘ で回す） | Cavalry | 2D / 2.5D / 3D のプレート | 違う |
| 奥行きに散らす（見た目を保って Z に広げる） | Cavalry | なし | 無い |
| カメラの切り替えを時間に並べる（マルチカム） | Cavalry Camera Guide・Spline Switch Camera | なし | 無い |
| 2D カメラ（枠を動かす・拡大・回転してキー） | クリスタ | なし | 無い |

## 7. 効果の重ね順

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 棚の効果をダブルクリック・ドラッグで入れる | 全部 | 同じ | 同じ |
| つかみをドラッグ → 順番（上から効く） | Blender・Cavalry・Rive | 長押ししてドラッグ | 違う |
| オン・オフ | 全部 | 同じ | 同じ |
| 焼く（Apply）・複製・選んだ全部にコピー・先頭へ・最後へ | Blender | More（前へ・後へ・戻す・ランダム・消す） | 違う |
| ⌥ を押して足す・焼く・消す → 選んだ物全部に | Blender | なし | 無い |
| 効き具合（Influence・Strength） | Blender・Cavalry・Rive | なし | 無い |
| 効果を時間で効かせる（帯がある間だけ） | Cavalry | なし | 無い |
| AI で効果を作らせる | Jitter | なし | 外す |
| 素材の層（色・光を積む、層ごとに不透明度と混ぜ方） | Spline | なし | 無い |
| 調整レイヤー（下に全部効く層） | **Adobe だけ**（After Effects） | なし | 外す |

## 8. レイヤー・マスク・クリッピング（クリスタを第一）

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 目を押す、⌥ で選んだもの以外を隠す・もう一度で戻す | クリスタ・Photoshop | 目とソロ | 違う |
| 目の右クリック → 見せる・隠すのメニュー | クリスタ | なし | 無い |
| 名前をダブルクリック → 名前を変える | クリスタ・Photoshop・Cavalry（Return） | なし | 無い |
| 上下にドラッグ → 順番、フォルダに入れる・出す | 全部 | 順番だけ | 違う |
| ⌥ ドラッグ → 複製 | クリスタ | なし | 無い |
| フォルダの三角、⌥ で中も全部、⌘ で同じ段を全部 | クリスタ | なし | 無い |
| ⌥] ⌥[ → 上・下の層を選ぶ | クリスタ | なし | 無い |
| ステージで ⌘⇧ クリック → そこに描いてある層を選ぶ | クリスタ | なし | 無い |
| 下の層でクリッピング（アイコンで切り替え、印の帯） | クリスタ（Photoshop は境目を ⌥ クリック） | 下の層で切る（Clip） | 同じ |
| マスク：選んだ所で作る、消しゴムで隠す、オン・オフ、焼く、層と一緒に動くかの鎖 | クリスタ・Photoshop | 四角・ペンの Mask だけ | 違う |
| 透明部分を守る（塗った所にだけ描く） | クリスタ・Photoshop | なし | 外す |
| 参照する層、下書きの層（書き出さない） | クリスタ | なし | 無い |
| 層の色の印 | クリスタ・Photoshop・Cavalry | 札の色（Ableton） | 同じ |
| 層の色をまとめて変える（層カラー） | クリスタ ⌘B | なし | 無い |
| 統合（下と ⌘E、見えるもの ⌘⇧E、選んだもの） | クリスタ・Photoshop | なし | 無い |
| ペーストの位置（同じ場所か画面の真ん中か） | クリスタと Photoshop で逆 | なし | 無い |

## 9. セルと時間（クリスタ）

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| セル（絵）をコマに割り当てる：コマを右クリック → セルの一覧、右ドラッグで等間隔に | クリスタ | なし（近い：複製と script） | 無い |
| オニオンスキン（前後のセルを薄く） | クリスタ・Photoshop | なし | 無い |
| キーで前後のセルをパラパラ見る | クリスタ | なし | 無い |
| ライトテーブル | クリスタ | なし | 外す |
| タイムラインの音、音をこする | クリスタ・Photoshop | 音の物はある（波形の絵） | 違う |
