# トラックパッドの照らし合わせ（10/25）

トラックパッドの操作を、Apple の決まりと 13 本のお手本から集めて、Motolii の場所ごとに当てはめる。印は ableton_map.md と同じ（同じ・違う・無い・外す）。

## 10/25 に直したこと

タイムライン・ステージ・関係のパネルで、トラックパッドの 2 本指とピンチを Flutter の pan/zoom のまま受ける（skin/book/lib/workspace/two_fingers.dart）。帯やステージのドラッグ（GestureDetector）は `supportedDevices` でトラックパッドの 2 本指を外した（Flutter の決まりでは、外さないと 2 本指をマウスのドラッグとして取る）。関係のパネルの 2 本指は Blender と同じく回すのまま。
確かめ：test/workspace_trackpad_test.dart（タイムラインは 2 本指で 156 → それより後、ピンチで見える幅が減る、帯は動かない。ステージはピンチで Fit → 50% → Fit、2 本指で絵が (40, 30) 動き、物は動かない）。前のコードでは 2 つとも落ちる（156 のまま、ズーム 0 のまま）。

## 確かめたこと（直す前の窓のタイムライン）

Flutter のテストの道具で、同じ量のマウスのホイールとトラックパッドの操作を、タイムラインに送って比べた（テストは scratchpad に移した。リポジトリには入れていない）。

| 送ったもの | ルーラーの同じ場所のコマ／見えている幅 | 結果 |
|---|---|---|
| マウスのホイールで横に 300 | 156 → 252 | 動く |
| トラックパッドの 2 本指で横に 300 | 156 → 156 | **動かない** |
| トラックパッドのピンチ 2 倍 | 192 コマ → 192 コマ | **拡大しない** |

わけ（Flutter の公式）：Flutter 3.3 から、Mac のトラックパッドの 2 本指・ピンチ・回転は `PointerPanZoomStart/Update/End` で届き、`PointerScrollEvent` はマウスのホイールだけになった。Motolii のタイムライン（timeline.dart:85）、ステージ（stage_view_2.dart:359）、関係のパネル（desk_relations.dart:88）は `PointerScrollEvent` だけを見ている。トラックパッドを受けているのはインスペクタの数字の欄（inspector_number_field_num_field_state.dart:203）だけ。
https://docs.flutter.dev/release/breaking-changes/trackpad-gestures

ステージと関係のパネルは、同じ書き方なので同じだと思うが、まだ確かめていない。

## 持ち主

- **意味の持ち主：Apple**。ピンチ＝拡大、回す＝回す、2 本指のダブルタップ＝スマートズーム、2 本指のクリック・タップ＝右クリック、2 本指で動かす＝スクロール。アプリが変えてはいけないもの：ページ間のスワイプ、フルスクリーンの切り替え、Mission Control、App Exposé、通知センター、Launchpad、デスクトップを表示、調べる（3 本指タップ・強めのクリック）。足してよいのは、よく使う専門の操作だけで、必ずキーなどの別の手段も用意する。
  https://developer.apple.com/design/human-interface-guidelines/pointing-devices
  https://developer.apple.com/design/human-interface-guidelines/gestures
  https://support.apple.com/en-us/102482
- **受け取りの持ち主：Flutter**。手で `Listener` を書かずに、Flutter 自身の部品に任せられる：`GestureDetector` の拡大（scale）とドラッグはトラックパッドを自分で受ける。`InteractiveViewer` と `ScaleGestureRecognizer` には `trackpadScrollCausesScale`（2 本指の上下を拡大にするか）がある。スクロールの部品はトラックパッドを最初から受ける。
  https://api.flutter.dev/flutter/widgets/InteractiveViewer/trackpadScrollCausesScale.html
- **Flutter が届けないもの**（Mac のエンジンのソース）：スマートズーム（2 本指のダブルタップ）、強めのクリックの圧力、3 本指のスワイプ。慣性は Flutter が自分で作る。
  https://github.com/flutter/flutter/blob/master/engine/src/flutter/shell/platform/darwin/macos/framework/Source/FlutterViewController.mm

## お手本どうしで揃っているもの

- ピンチ → 拡大（Figma、Cavalry、Spline、Final Cut、Resolve のビューア、クリスタ。タイムラインでも Theatre.js、Jitter、Cavalry、Final Cut）
- 2 本指で動かす → 動かす・スクロール（3D の Blender だけは回す）
- 2 本指のクリック・タップ → 右クリック
- マウスの代わりは ⌘＋スクロールで拡大（Figma、Rive、Spline、Ableton）

ばらばらなもの：スクロールで拡大する時の修飾キー（⌘ Figma・Rive・Spline・Ableton／⌃ Theatre.js／⌥ Cavalry のタイムライン・Resolve）、3D の 2 本指（Blender は回す、Spline は動かす）、回転（Photoshop とクリスタだけ画面を回す）、段の高さ（Ableton は ⌥＋ピンチ）。

## 数

| 印 | 行 |
|---|---|
| 同じ | 7 |
| 違う | 3 |
| 無い | 5 |
| 外す | 3 |

## Motolii の場所ごと

### ステージ

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| ピンチ → 拡大（指の位置を中心に） | Figma・Cavalry・Spline・Apple | 10/25 から：ピンチで Fit・50%・100% を 1 段ずつ（広げても狭めても 4 分の 1 で 1 段。拡大が 3 段しか無いため、指の位置を中心にした連続の拡大ではない） | 違う |
| 2 本指で動かす → 動かす | Figma・Spline・Apple | 10/25 から：拡大している時、絵が指と一緒に動く（Fit の時は動かない、ホイールと同じ）。物は掴まない | 同じ |
| ⌘＋スクロール → 拡大 | Figma・Rive・Spline | なし | 無い |
| 2 本指のダブルタップ → 全体と拡大の行き来（スマートズーム） | Apple・Cavalry | Flutter が届けない | 外す |
| 2 本指で回す → 選んだ物を回す（Apple）／画面を回す（Photoshop・クリスタ） | Apple・Photoshop・クリスタ | なし | 無い |
| 2 本指のクリック → 右クリック | Apple・全部 | 右クリックが値の行にしか無い（ステージには無い） | 無い |

### タイムライン

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| ピンチ → 時間の拡大 | Theatre.js・Jitter・Cavalry・Final Cut | 10/25 から：指の下のコマを中心に拡大 | 同じ |
| 2 本指で横 → 時間を動かす、縦 → 段を動かす | Final Cut・Theatre.js・Cavalry | 10/25 から：同じ（名前の列も上下）。帯は掴まない | 同じ |
| ⌘＋スクロール → 拡大 | Ableton | ⌘ ホイールで拡大（マウスだけ） | 違う |
| ⌥＋ピンチ → 段の高さ | Ableton | 段の高さは変えられない | 無い |
| 2 本指のダブルタップ → 全部が見えるように | Final Cut | Flutter が届けない（Fit ボタンはある） | 外す |
| 3 本指で横 → 次・前の切れ目へ | Final Cut（設定を変えた時だけ） | Apple のページ間スワイプとぶつかる | 外す |

### 関係のパネル（3D の見え方）

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 2 本指 → 回す、⇧ → 動かす、⌘ → 寄る | Blender | ドラッグで回す、ホイールで寄る（トラックパッドは受けていないと思う） | 違う |
| ピンチ → 寄る | Spline・Blender | 10/25 から：寄る | 同じ |

### インスペクタの数字の欄

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 2 本指で横 → 値を動かす | お手本に書いたものは無い | 受けている（Motolii の自作） | 同じ |

### 棚・一覧・パネル

| お手本の操作 | どこ | Motolii いま | 印 |
|---|---|---|---|
| 2 本指でスクロール（慣性つき） | Apple・全部 | Flutter のスクロールの部品がそのまま受ける | 同じ |
| 2 本指のクリック → 右クリック | Apple・全部 | 棚に右クリックが無い | 無い |
| 修飾キー＋ドラッグの意味は、マウスでもトラックパッドでも同じ（⌥ドラッグで複製） | Apple | 同じ（どちらでもドラッグは同じ道） | 同じ |
