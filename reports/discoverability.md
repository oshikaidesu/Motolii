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
