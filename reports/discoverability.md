# 見つけるための仕組み：操作体系を丸ごと借りる（10/25）

初めて触る人はキーを覚えていない。どの操作も、メニュー・右クリック・検索・ヒントのどこかから見つけられて、見つけた所でキーも覚えられる（「キーを教える」形）ようにする。そのための仕組みを、どこから借りられるか。

## 借りられるもの

| 何 | 中身 | 借りられるか | 出典 |
|---|---|---|---|
| **VS Code の命令の仕組み** | 命令を 1 回書く（id、名前、分類、いつ使えるか）→ 命令の検索（⌘⇧P）、メニュー、右クリック、キーが全部そこから出る。キーは表示のたびに命令から引く（メニューの文字にキーを書き込まない）。`when`（いつ使えるか）の式。キーの設定画面（全命令の一覧、名前でもキーでも絞れる） | **MIT**。TypeScript なので考え方を移す。actions.ts（メニューの場所）、keybindingResolver.ts（キーの決まり方）、contextkey.ts（when の式）、commandsQuickAccess.ts（検索と最近使った命令）、keybindingsEditor.ts（設定画面） | github.com/microsoft/vscode 、code.visualstudio.com/api/references/contribution-points |
| **Flutter の PlatformMenuBar** | 本物の Mac のメニューバー。キーを Mac に渡すので、メニューにキーが出て Mac が受ける。ヘルプメニューの検索で全部の命令が見つかる（Mac がただでやる） | **BSD-3**（Flutter そのもの） | api.flutter.dev/flutter/widgets/PlatformMenuBar-class.html |
| **Flutter の MenuAnchor・ContextMenuController** | 右クリックのメニュー | BSD-3 | api.flutter.dev/flutter/material/MenuAnchor-class.html |
| **Flutter の Shortcuts・Actions** | キー → 命令 → 動き。使えない時は灰色 | BSD-3（Motolii はもう使っている） | api.flutter.dev/flutter/widgets/Actions-class.html |
| **flutter_kbar** | 命令の検索の画面。cmdk の点数の付け方（「gs」で「Git status」が先）、各行にキーの札、最近の履歴 | **MIT**、1.0.1（2026-08） | github.com/alirezat66/flutter_kbar |
| **Apple の決まり** | メニューの順番（アプリ、ファイル、編集、表示、…、ウィンドウ、ヘルプ）。ほかに入口があっても全部の命令をメニューバーに。使えない命令は隠さず灰色。右クリックは使えるものだけ、キーは書かない。ツールバーの命令は全部メニューにも。ヒントは 60〜75 文字で、名前を繰り返さず何をするか | 決まりを借りる | developer.apple.com/design/human-interface-guidelines/the-menu-bar 、/context-menus 、/offering-help |

## 形だけ借りるもの（コードは借りられない）

| 何 | 中身 | 出典 |
|---|---|---|
| Ableton の Info View | 何に乗っても、隅の欄に名前と何をするかが出る（`?` で出し入れ）。初心者向けで一番有名 | ableton.com/en/live-manual/12/first-steps/ |
| Illustrator・Photoshop の Contextual Task Bar | 選んだ物のそばに「次にできること」のボタンが出る | helpx.adobe.com/illustrator/using/contextual-task-bar.html |
| Figma | ⌘/ で操作を探す。ショートカットのパネルで、使ったことのあるキーと無いキーを色分け | forum.figma.com/t/new-quick-actions-menu/1788（確度中） |
| Blender | F3 の検索が、見つけた命令のメニューの場所も出す（場所を教える）。ヒントにキーが出る（GPL なのでコードは借りない） | docs.blender.org/manual/en/latest/interface/controls/templates/operator_search.html |
| Cavalry | `/` で探す（最後の 3 つを上に）。ショートカットの設定画面でぶつかると赤く光る | cavalry.studio/docs/user-interface/menus/window-menu/shortcut-manager/ |
| Final Cut | 命令の設定画面がキーボードの絵。押したキーで何が起きるかが見える | support.apple.com/guide/final-cut-pro/ver4df74f72/mac |

## Motolii に当てると（おすすめの組み合わせ）

Motolii はもう、VS Code の形のキーの表（keys.dart）と keybindings.json を持っている。その先を借りる。

1. **命令の表を 1 つに**（VS Code の Action2 の形、MIT）：keys.dart の行を「id、名前、分類、メニューの場所と順番、いつ使えるか、何をするか」に広げる。キーは keybindings.json だけに置き、キーを出す所は全部、表示の時に id から引く。
2. **いつ使えるか**（VS Code の contextkey の形、MIT）：選んでいる物の種類、どのパネルにいるか、などで命令を有効・無効にする。キーが効くかどうかと、メニューの灰色の両方に使う。
3. **メニューバー**（Flutter の PlatformMenuBar、BSD-3）：表から作る。Mac がキーを描き、ヘルプの検索で全部の命令が見つかる。順番と灰色は Apple の決まり。
4. **右クリック**（Flutter の MenuAnchor）：同じ表から、その場所の命令を出す。使えるものだけ、キーは書かない（Apple）。
5. **命令の検索**（flutter_kbar、MIT）：最近使った命令を上に（VS Code）、各行にキー、メニューの場所も出す（Blender）。
6. **ヒント**：何をするかを 1 行とキー（Blender、長さは Apple）。乗ると隅に説明が出る欄は Ableton の Info View。
7. **キーの設定画面**：全部の命令の一覧、名前でもキーでも絞る、キーを押して決める、ぶつかると赤く（VS Code の設定画面、Final Cut のキーボードの絵、Cavalry の赤く光る）。

この形にすれば、操作を 1 つ足すたびに、メニュー・右クリック・検索・キー・ヒントの入口が全部そろう。

## 効果が 5 万件になった時：人に書かせずに札を付ける（10/30 昼）

ユーザー（10/30）：「これからどんどん fx 表現は増える、50000 件まで行ったらどうする？ 売買サイトを参考にできないかな、メルカリとか」「かれらはその情報すら入力させないようにしているよね。こっちは静的解析できるから似たアプローチが取れるはず」。棚を眺めて選ぶのは 1,200 件（glow だけで 16 枚）でもう苦しい。メルカリは写真を撮るだけで題名・カテゴリ・説明を埋め（AI 出品サポート、2024/9）、写真で探す（絵の埋め込みと近いものの検索）。効果のファイルは中身を全部読めて何度でも描けるので、同じ形が取れる：**読んで分かる札**と**描いて測って分かる札**を、置いた瞬間に自動で付ける。調べ：サブエージェント（読むだけ）、主な行は確かめた。

**今あるもの**
- ISF の読み：filament/isf.cpp。絵の入力・入力の無い物（生成）83〜86・170〜171 行、音の入力 89・141〜145 行、PASSES・PERSISTENT 70〜78 行、CATEGORIES を棚の 7 語（Glitch・Distort・Color・Film・Light・Time・Stylize）に部分一致で寄せる 201〜206 行、DESCRIPTION を説明に 207〜210 行。glslang と Tint はもう入っている（Tint の WGSL inspector と SPIRV-Cross も組んであるが使っていない）
- 棚：dock_effects.dart（名前か説明文の部分一致で探す 47 行、札は label から 48〜58 行）、★ は ~/.motolii/effect_stars.json。効果の「最近使った」は無い（字体だけ）
- 札の絵：effect_tiles.dart（気球の写真と Aa、8 コマ、つなぎは progress を 0→1、PERSISTENT は頭から描く）。tileSame は「見本と見分けがつかない」（ffmpeg の PSNR が 8 コマとも 40 dB 超 → NO CHANGE）で、**効果どうしが似ているかは見ていない**
- AbsentMotion の tools/effect_showcase.json：181 本の効果ごとに写真 9 枚のどれに掛けるか・動かし方。Motolii は params と split だけ読み、photo・animate は読んでいない

**数えたもの**（filament/effects）
- .fs 420・.wgsl 168・.am 20・.mjs 4・AviUtl2 276
- 説明が無い .fs：198 / 420（キーが無い 105、空 93）
- カテゴリの綴りがばらばら（Color と Color Effect、Distort・Distortion・Distortion Effect、タブ入り）
- 読むだけでもう分かる札：つなぎ 67、音で動く 7、前のコマを覚える（PERSISTENT）46、生成（絵の入力が無い）45

**やること**（S 小・M 中・L 大）

| やること | 借りる物（ライセンス） | Motolii のどこ | 大きさ |
|---|---|---|---|
| 読んで付ける札：効果 / 生成 / つなぎ、音、前のコマ、TIME、パスの数、他の物の絵、外へ描くか | Tint の WGSL inspector（Apache-2.0、組み済み）、rapidjson（使用中） | doc_recipes.h の棚の一覧に列を足す、motolii_shelves.dart で読む | S |
| カテゴリの綴りを 1 語に寄せる — **済（10/30 午後、ad4cb00b9）**：VIDVOX の「… Effect」「… Adjustment」を外す（Geometry 17＋15 → 32）。あわせて ISF のつなぎ（startImage・endImage・progress）を 1 つの札「Transition」に（After Effects の Transition の分類、67 本、Wipe・Dissolve などに散っていた） | ISF と AbsentMotion のカテゴリの一覧そのもの | isf.cpp 201〜206 行 | S |
| 効果ごとに見本の写真を選ぶ（顔・グリーンバック・格子…） — **止めている（10/30 午後）**：写真 9 枚はリポジトリに無く、AbsentMotion の tools/effect_thumbs.py が Wikimedia Commons から落としている（「File:Fishmonger smiling.jpg」など）。落とすにはユーザーの許しが要る | AbsentMotion の effect_showcase.json の photo（Apache-2.0） | effect_tiles.dart | S |
| 重い・軽い（60fps の札） | m.still の前後の時間、motolii_spent（motolii_hand.h 467 行）。Dawn の時刻の問い合わせは Filament で使えるか未確認 | effect_tiles.dart | S〜M |
| 描いて測る「何をするか」：ぼかす・色・歪む・動く | OpenCV（Apache-2.0）か scikit-image（BSD-3）：SSIM、ラプラシアンの分散、色の分布、オプティカルフロー。しきい値は Motolii 側（今の 40 dB と同じく測って決める） | fx_first.sh の横（もう Pillow・numpy を使う）、札の絵の置き場に .tsv | M |
| 物の外へどれだけ描くか（㊿ の部屋の広さにも使う） | 借りる物なし：小さな物に掛けて、箱の外の透明度を測る | effect_tiles.dart、screen.h の spill | M |
| 似た効果を 1 枚にまとめる（glow 16 枚） | imagehash（BSD-2）か OpenCV の img_hash（Apache-2.0）。pHash.org は GPLv3 なので使わない | 札の絵の横にハッシュ、まとめるのは dock_effects.dart | M |
| 貼った絵で探す・言葉で探す（「光る写真」で札を付けるのにも） | SigLIP 2（コードも重みも Apache-2.0、多言語）。OpenCLIP はコードは MIT だが LAION の重みは「配って使う」のが対象外。動かすのは ONNX Runtime（MIT、合うかは未確認） | 札の絵のコマを埋め込み、探す欄は dock_effects.dart | L |
| 埋め込みの置き場 | sqlite-vec（MIT/Apache-2.0、v1 前）。5 万 × 768 で約 154 MB、総当たりで足りる（hnswlib・FAISS は 50 万を超えてから） | 札の絵の置き場の横に別の DB | S〜M |

**先例**
- Shadertoy はタグのほかに「何が出来るか」で絞る（音を出す・音を聞く・カメラ・マルチパス）：Motolii が読むだけで分かる札と同じ種類
- メルカリは多言語の SigLIP を自分の品物で調整して、写真検索のクリックが 50%・購入が 14% 増えた（A/B、2025 の論文 arxiv.org/abs/2510.13359）。借りる候補を SigLIP にする裏付け
- 出典：openai.com/index/mercari、cloud.google.com のメルカリのベクトル検索の記事、huggingface.co/google/siglip2-base-patch16-224、github.com/asg017/sqlite-vec、github.com/JohannesBuchner/imagehash、shadertoy.com/howto

**先の話（ユーザー 10/30）**：p5.js（Processing）や Blender の人気プリセットを「互換ではなく取り込み」。取り込んだ物も、置いた瞬間に同じ「読んで・描いて・測って札を付ける」に通せば、同じ棚に同じ探し方で並ぶ。順番は「普通に使える」の穴埋め（golf_noise.md）が先
- Blender はジオメトリノード（ユーザー：「でもジオメトリーノードも absent motion みたいなのがいるかもしれない」）。道は 2 つ：(1) Blender を裏で走らせて（blender -b）glTF・Alembic に焼いて取り込む。Blender が計算するので確かだが、つまみをその場で動かすたびに焼き直すので 60fps で触れない。(2) AbsentMotion が Alight Motion を外から作り直したように、ジオメトリノードを Blender の外で動かす実装を借りる。Blender のコードは GPL なので、その移植も GPL になる。外で独立に書かれた物があるかは、調べた（10/30 午後、サブエージェント、読むだけ）：
  - **そのまま使える独立の実装はまだ無い**。形が AbsentMotion と同じなのは geonodes-kit（MIT、JavaScript、.blend 5.x を直に読み、Blender 5.2.2 の出力と比べる。Blender のコードを読む人と書く人を分ける作り）だけで、2026-09-28 に始まったばかり、360 のノードのうち確かめ済みは 1。ThreeGN（three.js、EPL-2.0、2023 年で止まっている、「足りないノードは Blender のコードを読んで写せ」とあり線引きが曖昧）
  - 別の仕組み（Houdini・Unreal PCG・three.js TSL・Godot）へ直す道具は見つからない
  - Blender を裏で走らせる例はある：AlterMesh（Unreal、有料）、unigine-geometry-nodes（MIT、作者いわく「その場では動かない」）。bpy（Blender を Python の部品にした物、GPL-3.0、5.2.3）を別の process にして、つまみの値を毎コマ送り、形を受け取る形。小さな木の計算は数 ms と見込むが、形を渡す往復が 60fps に収まるかは**測った物が無い**
  - .blend を Blender 無しで読む部品（Rust の blend・blend-rs、Kaitai の blender_blend：MIT・Apache-2.0・CC0）はあるが、5.x に対応するかは未確認。読めるのは木のつながりと値まで、各ノードが何をするかは Blender のコードの中にしか無い
  - 作り直すなら土台になる部品（どれも Blender の外の独立の物）：Manifold（ブール、Blender 4.5 も使う）・OpenSubdiv・OpenVDB・libigl（面の上の点）・nanoflann・tinyspline・Embree（ライセンスは記憶から、確かめ直していない）
  - 順番の見込み：(1) 今すぐ：bpy を別 process の計算係にして、往復の時間を先に測る。(2) 長い目：geonodes-kit を見守るか手伝う（AbsentMotion の道）。自分で Blender のノードを写すことはしない
