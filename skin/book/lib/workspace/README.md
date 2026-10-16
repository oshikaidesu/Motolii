# Workspace — Motolii の窓（2026-10-16）

Widgetbook の `Workspace › Workspace › Live`（route `/?path=workspace/workspace/live`）が本物の窓。書類は C++ の中身（`../motolii.dart` の `Motolii`、各メソッドが `motolii_*` の C 呼び出し 1 つ）にあり、Stage の絵も中身（Filament）が描く。窓は見せて、手を書類に渡すだけ。`.../workspace/window` は中身なしの見本。

## 自分のパネルを足す（入口はここだけ）
`import 'package:lab_book/workspace/door.dart';` の 1 行で、下の全部（`Workspace`、`WsPane`、`DeskTool`、`Ws`・`WsScope`、`Motolii`）が使える。
```dart
Workspace(document: path, extra: [
  WsPane('Notes', null, () => const MyNotes()),   // (名前、アイコン（null なら頭文字）、中身)
])
```
- 足したパネルは Inspector の横にタブで並ぶ。本家のファイルは 1 行も変えない。
- パネルの中では `WsScope.of(context)` が窓の状態（`Ws`）：`inside` が書類（`Motolii`）、`selected` が選ばれた物、`frame` が今のコマ。値は `inside.value(thing, name, seconds)`、変えるのは `inside.set(...)`、キーは `inside.keyHere(...)`。
- 物を絵の上で動かす（画素で）：`inside.hold(t)`、`inside.carry('place', 10, 0)`、`inside.letGo(true)`（手で引いたのと同じ 1 歩）。値をそのまま変えるのは `inside.set(t, name, value)`（書類の単位）。キーの数は `inside.keyMoments(t, name).length`。
- 気をつける 4 つ：選ばれた物 `selected` は文字なので `int.parse` してから書類に渡す。今の秒は `frame / Ws.fps`。物の名前は `inside.word(t, 'name')`。書類を変えたら `ws.reread()`（窓の他のパネルが読み直す）。
- 例：`test/workspace_window_test.dart` の「a pane of your own」（書類なし）。本物の書類で試すなら `import 'door.dart';`（`test/door.dart`）の `openWorkspace(tester, extra: [...], tools: [...])` と `wsOf(tester)`、`closeWorkspace`、`skip: !insideBuilt`。

## 自分の desk の道具を足す
```dart
Workspace(document: path, tools: [
  DeskTool('Snap', Color(0xFF55AA88), 'Picked thing', build: MySnap.new),   // (名前、色、何に効くか、中身)
])
```
- 窓の道具の後ろに並び、選ぶと desk がその `build` を出す。アイコンは頭文字。中では パネルと同じく `WsScope.of(context)`。
- 例：`test/workspace_window_test.dart` の「a desk tool of your own」。

## 自分のプリセット（Ease の動き方）を足す
窓の一覧ではなく書類の動き方として足す。足した物は Ease の desk にタイルで出て、書類と一緒に保存される。
```dart
ws.inside!.define('Snappy', 'a + (b - a) * bezier(u, .2, 0, 0, 1)');   // (a: 前のキーの値、b: 次のキーの値、u: 区間の 0〜1)
```
- 例：`test/workspace_joined_test.dart` の「a preset of your own」。

## 開き方・確かめ方
```bash
cd skin/book
F=~/rust_ae/Motolii/.tools/flutter/bin
$F/flutter build macos --debug '--dart-define=LAB_ROUTE=/?path=workspace/workspace/live'
$F/flutter analyze lib test integration_test
$F/flutter test                                  # 中身とつないだ確かめは test/workspace_joined_test.dart
```
- 窓の文字は英語。会話と文書は日本語。

## トンマナ(Pop)
- 値は `WsT`(窓)と `Pop`(Inspector、`sets/inspector/inspector_parts.dart`)と `Grey` / `T`(`tokens.dart`)から取る。**数値を直書きしない**。
- 箱(席・カード)は四角く、隙間なく並べる(間は `WsT.gutter` の暗い線)。触る物と読む物(欄・チップ・ボタン)は丸い。
- 色は平らで不透明。アクセントはライム 1 色(`#B3C66B`、上の文字は `#15170D`)。再生ヘッドもアクセント。
- レイヤーの種類ごとに色を 1 つ(`wsTone`)。Timeline のバー、Dock、Stage で同じ色を使う。

## カラーテーマ(Dark / Light)
- 上のバーの半円ボタン(key `ws-shade`)で切り替える。`Grey.shade`(`ValueNotifier<Shade>`)を反転して `reassembleApplication()` で窓全体を建て直す。State は残るので編集中の内容は消えない。
- **Workspace と Inspector(`sets/inspector/`)では灰色を `Grey.g07` などで取る。`N.g07` は使わない。** `N` は古い研究ページ用の暗い段で、テーマに追従しない(そのページは Dark のみ)。
- `Grey` は `N` と同じ名前の段を持つ。役割(地・席・カード・欄・線・控えめな文字・文字)は両テーマで同じで、明るさだけが逆になる。Light ではカードが一番明るく、文字が一番暗い。
- テーマで変わる色は `const` にできない。`static const` / トップレベルの `final` / 引数の既定値に `Grey` や `WsT.body` などを置かず、getter か `Color?` + `?? Grey.xx` にする(`final` に入れると最初の色のまま固まる)。
- アクセントは塗りなら `WsT.accent` / `Pop.accent`。**線や文字として地の上に描くときは `accentInk`**(Light ではライムが白に溶けるので濃いオリーブ)。再生ヘッドの線は `accentInk`、頭の札は `accent`。
- 種類の色(`tone*`)と Stage の絵(`ArtInk`)はテーマで変えない。
- `T.name()` などの文字色の既定は `Grey` から取る。暗すぎる・薄すぎる文字の補正(`T._floor`)もテーマごとに向きが逆。

## 決まっていること(利用者の裁定)
- **Media と Files は一つの棚(Media)。** AEViewer 2 型: 左(狭い時は上)に場所(Project / Home / Desktop / Downloads)とフォルダだけの木、右に開いているフォルダの中身をサムネイルかリストで。フォルダのタイルを押すとそのフォルダを開く。パンくずの各段も押せる。検索は今いる場所の中を全部探し、結果に入っているフォルダ名を添える。サムネイルの格子は捨てない。
- **窓はドックパネル。** Motolii 本体と同じ pub の `docking`(本体の `ui/lib/workspace/dock_workspace.dart` を lab 用に書き直したもの)。ブラウザも棚ごとにペインに分けた(rail は無い)。**タブは記号で示す**: 棚のタブは記号だけ(中の見出しが名前を言う、5 枚並んでも細い)、Stage / Inspector / Timeline / Desk は記号 + 名前(席の中の見出しに同じ名前を重ねない)。前に出ている棚が `Ws.shelf`(タブを押すと変わり、`Ws.route` で棚のタブが前に出る)。別窓に出す Detach は native の窓が要るので lab には無い。ペインは閉じられない(開き直す手段がまだ無い)。
- **初期配置**: 棚のタブの列・Inspector・Timeline・Desk は基準の px で始め、Stage が残りを取る。`multi_split_view` は size を weight に直して足し合わせるので、残りを取るペインには weight を付けない。
- **ペインは広げたら得をする。** 列を固定して引き伸ばさない。
  - 格子は `wsCols(幅, セルの幅)` で列数を決め、`wsCell` でセルの幅を割り出す(Create のタイル、Colors の色見本、Desk の道具)。
  - 一覧の区切り(Create の群、Colors の Used / Palettes / Gradients / Swatches、Effects の族)は `DockMasonry` のカードにする。1 列は 260〜300px 以上、最大 4 列、短い列から詰める。
  - Fonts は 440px 以上で見本の大きいカード、狭いと行。Media は 440px 以上で木が左、狭いと上。
  - Effects はフィルター帯を開いていて 560px 以上なら、帯を一覧の横に縦いっぱいで立てる。狭いと上の帯。
  - **Inspector は縦積みのまま**(カードを横に並べない)。カードの中身を、Effects のパラメーターと同じブロックの格子(`Pop.tileMin` ごとに列が増える)にする。Layer の Opacity / Depth / Parent は `LayerTiles`。
- **Inspector は一目でわかること。** フォント・色の一覧・イージングは、それぞれ専用のパネル(Browser › Fonts、Browser › Colors、Ease の desk)が持つ。Inspector は今の値を見せて、そこへ誘導するだけ(`PanelLink` / `LinkLine` → `Ws.route`)。
- **Inspector の値は Stage と一致させる。** 位置・倍率・回転・塗りは `WsArt.boxOf` と絵の色から渡す(`InspHost(values:, strs:)`)。
- **パスの操作は同じものを何度でも重ねられる**(パンク膨張の後にまたパンク膨張)。UI 側で数を絞らない。
- **シードなど普段触らない値も、隠さずに Advanced に置く**(`AdvancedFold`)。
- **テキストは普通に**: 入力欄、その下にサイズ・太さ・色。
- **Effects の一覧にサムネイルは付けない。** 見た目は Stage で確かめる: 行を選ぶ → 選択中のレイヤーに仮で掛かる → Add / Enter で確定、Cancel / Esc で取り消し。
- **効果のタグは 2 種類。** 振る舞い(Seat / Applies to / Time / Inputs / Parameters / Origin)は shader の静的解析が出す。何に見えるか(Look)だけを作者が `@tags(...)` で書く。試し表示だけで伝わらないこと(下の合成を読む、1 フレーム目に何も出ない、など)は、作者の `@preview(...)` か解析結果から帯に一言出す(`FxDef.trialNote`)。
- **フィルター帯は Motolii 本体の草案に合わせる**: グループ同士は AND、グループ内は OR。クリックで 1 つ、Cmd/Ctrl-クリックで追加・解除。残る件数を札に出し、0 件になる札は押せない。件数と Clear は帯の一番下。**帯の高さは利用者が決める**(下端のグリップを上下にドラッグ、ダブルクリックで 168px に戻る)。

## 変えるときの作法
- 席をまたぐ情報は `Ws` に足す。席どうしで import し合わない。例外は二つだけ: Dock と Stage が共有するデータの `fx_catalog.dart`、Inspector に Stage の値を渡すために `stage_art.dart` を読む組み立て役の `workspace.dart`。
- 新しい振る舞いには `test/workspace_*_test.dart` を足す。窓全体の通しは `workspace_window_test.dart`。
- コメントは、コードから読めない制約だけを書く(`contract:` で始まる行がそれ)。
- `lib/sets/inspector_gui_set.dart` は利用者の作業中のファイル。触らず、コミットにも含めない。
- 区切りごとにコミットする(履歴が無いのが一番怖い、が利用者の方針)。

## まだ無いもの
- 見本と中身の二股（`inside == null`）が残っている（TASKS.md 24 番）。
