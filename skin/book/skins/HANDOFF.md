# Motolii の見た目：引き継ぎ（人と AI 向け）

Motolii の窓（Flutter、`skin/book`）の見た目は、中身（書類・動き・絵）に触らずに3段で変えられる。この文書は、続きを作る人や AI が、これだけ読めば手を動かせるように書いてある。スキンを作るだけなら [README.md](README.md) で足りる。

## 1. 3つの段

| 段 | 何が変わる | 書くもの | 配り方 |
|---|---|---|---|
| ① 値 | 色・明暗・角・余白・大きさ・文字の大きさ・書体 | `theme.xml`（名前と値を並べるだけ） | フォルダを置くだけ |
| ② 絵 | 印（アイコン）と、部品の枠 | `parts/*.svg` | フォルダを置くだけ |
| ③ フォーム | 部品の**組み立てそのもの**（ボタンが何の形か、選択肢がどう並ぶか、札がどう組まれるか） | Dart のクラス1つ（`lib/forms/<名前>.dart`） | アプリに組み込む |

①②は [README.md](README.md)・[ROLES.md](ROLES.md)・[PARTS.md](PARTS.md) に全部ある。外の LLM 3人に README だけで作らせて 6 → 7 → 8 点（10点満点）、指摘は直した。
③はヘッドレス UI（React Aria・Radix と同じ分け方）。ここから下は主に③の話。

## 2. ファイルの地図

| 場所 | 中身 |
|---|---|
| `lib/theme.dart` | theme.xml を読む・見張る（`ColorTheme`）。着ている革は `~/.motolii/window.xml` の `Skin` |
| `lib/tokens.dart` | 色の段（`Grey`、Background/Foreground から HCT で作る）、`Shape.r/s/z`（角・余白・大きさ）、`T.px`（文字）。数の効く範囲もここ |
| `lib/window_file.dart` | `~/.motolii/window.xml` の読み書き（倍率と着ている革） |
| `lib/parts/skin_parts.dart` | 印の SVG（`SkinParts.draw`）と枠（`SkinFrame`、9分割） |
| `lib/forms/forms.dart` | **フォームの約束**：渡す物（Spec）と、フォームが書く関数、登録表 `forms` |
| `lib/forms/own.dart` | いつもの見た目のフォーム（前からの見た目をそのまま移した） |
| `lib/forms/poster.dart` | 見本：スイスの幾何学ポスター風 |
| `lib/forms/lighttable.dart` | 見本：フィルムのライトテーブル風（外の AI が HANDOFF だけで書いた、7/10） |
| `skins/<名前>/` | リポジトリの見本の革（labs、paper、streets、poster、lighttable） |
| `skins/shoot.sh` | 革のフォルダを渡すと、その革を着た窓を撮って `preview.png` を書く |
| `live.sh` | 窓を開いたまま、`lib/` の .dart を保存した瞬間に描き直す（状態は保つ） |
| `test/skin_names_test.dart` | ROLES.md・PARTS.md と、コードが読む名前・描く印・枠が食い違ったら落ちる |
| `integration_test/skin_fit_test.dart` | 意地悪な革・無茶な値の革・ポスターのフォームで、文字の切れ・はみ出し・タブの消失を数える（全部0が合格） |

## 3. フォームの作り方（ヘッドレス UI）

### 分け方
- **窓が渡す**：部品の言葉・今の値・押した時にすること（下の Spec）。
- **手で触った時の動き**：[naked_ui](https://pub.dev/packages/naked_ui)（BSD-3）。押す・手が乗る・選ぶ・切り替える・読み上げ。`NakedButton`、`NakedToggle`（`asSwitch: true` でスイッチ）、`NakedToggleGroup` と `NakedToggleOption`（どれか1つ）、ほかに `NakedSlider` `NakedTabs` `NakedSelect` `NakedMenu` など。builder に状態（`isHovered` `isPressed` `isSelected` `isFocused` `isDisabled`）が来る。
- **フォームが書く**：それをどう組み立てて見せるか、だけ。**動きを自分で作り直さない**（GestureDetector を足さず naked_ui を使う）。

### 約束（`lib/forms/forms.dart`）

| 関数 | 渡る物 | 窓の部品 |
|---|---|---|
| `action(context, ActionSpec)` | `label`、`onPressed` | 上の帯の Render |
| `tabs(context, ChoiceSpec)` | `labels`、`selected`、`onSelect`、`enabled` | 上の帯の Edit・Animate・Export |
| `choice(context, ChoiceSpec)` | 同上＋`glyphs`（16×14 の箱に描く関数） | インスペクタの Space（2D・2.5D・3D） |
| `onOff(context, SwitchSpec)` | `on`、`onChanged`、`enabled` | オン・オフ（Animate all など） |
| `card(context, CardSpec)` | `title`、`tone`（札の種類の色）、`open`、`onToggle`、`trailing`、`body`（行）、`head` と `own`（窓のいつもの組み立て） | インスペクタの札1枚 |
| `segment` | `ChoiceSpec`（言葉が並ぶ。幅は言葉の分だけ） | Fit・50%・100%、Layer・Group など |
| `flag` | `FlagSpec`（言葉、オン、印） | Snap、Safe、Grid など |
| `chips` | `ChipsSpec` | 棚の絞り込み |
| `track` | `TrackSpec`（名前、種類の印、目・ソロ・錠） | タイムラインの層の名前 |
| `number` | `NumberSpec`（数の文字、打っている欄、範囲） | 数の入力欄。こすって変える動きは窓が持つ |

約束の細かい所：
- `ChoiceSpec.glyphs` の関数は `(Canvas c, Paint stroke)`。**Paint はフォームが作って渡す**（色・太さ・塗りか線かはフォームが決める）。印は 16×14 の箱に描かれるので、大きく描くなら `c.scale()` してから呼ぶ。
- `CardSpec.head`：窓のいつもの見出しの行（開閉の印・種類の印・題・キーの点・畳んだ時の要約・`trailing` が組み立て済み）。自分の見出しを作るなら捨ててよい。ただし **`title` が null でも `head` がある札**（`trailing` だけの札）がある。その時は `head` を出す。
- `CardSpec.own`：見出しと行を余白で包んだ、外枠なしの札の中身全部。「外枠だけ変えたい」なら `own` を自分の枠で包むだけでよい（`own.dart` がそう）。
- 見た目の動き（`AnimatedContainer`、`TweenAnimationBuilder` など）は自由。手の受け取り（`GestureDetector`、`MouseRegion`、`Listener`）だけは書かず naked_ui に任せる。

### 使える材料

| 種類 | 名前 | 中身 |
|---|---|---|
| 文字の型 | `T.title()` `T.name()` `T.label()` `T.micro()` | 書体は革の `FontSans`。見出し・名前・小さな言葉・極小 |
| 文字の型 | `T.value()` | 書体は革の `FontMono`（等幅）。数と時間に |
| 文字の大きさ | `T.px(11)` | 革の TextScale を掛けた大きさ（`.copyWith(fontSize: T.px(n))`） |
| 書体の名前 | `T.sans` `T.mono` | 革が言う書体の名前 |
| 色 | `Grey.g00`〜`Grey.g100`、`Grey.bandA` など | 灰色の段（どの段が窓のどこかは [ROLES.md](ROLES.md) の表）。革の Background・Foreground から作られる |
| 色 | `WsT.accent` `WsT.onAccent` `Pop.accentInk` | 差し色、その上の文字、地の上に線や文字で使う差し色（`lib/workspace/ws.dart`、`lib/sets/inspector/inspector_parts.dart`） |
| 色 | `axX` `axY` `axZ`、Spec の `tone` | 軸の色、札の種類の色 |
| 形 | `Shape.r(n)` | 角の丸み（`Radius`、革の Corner を掛ける） |
| 形 | `Shape.s(n)` `Shape.z(n)` | 余白・大きさ（革の Spacing・Size を掛ける） |

フォームが自分の色を持つのも自由（`poster.dart` は自分の色の定数を持つ）。革の色に合わせたい所だけ上の材料を使う。

### 手順
1. `lib/forms/<名前>.dart` に `class XForms extends Forms` を書き、5つの関数を全部書く（`poster.dart` が見本、240行）。
2. `lib/forms/forms.dart` の `forms` に `'<名前>': XForms()` を1行足し、import する。
3. 革のフォルダの `theme.xml` に `<Forms Value="<名前>"/>` と書く（色は同じ theme.xml で）。
4. 見る：`./live.sh` で窓を開き、View › Skin でその革を着て、フォームのファイルを保存するたびに描き直るのを見る。窓を開かずに見るなら `skins/shoot.sh skins/<名前>`（2分ほど、`preview.png`）。shoot.sh は手を乗せた時・押した時を撮らないので、そこは live.sh の窓で手で確かめる。
5. 確かめる：`skin_fit_test.dart` の `_skins` に `'<名前>': '<Forms Value="<名前>"/>'` を足して流す（切れ0・はみ出し0）。色は文字の収まりに効かないので、書くのはフォームと書体（と Size・TextScale を変えるならそれ）だけでよい。`flutter test` で新しい落ちが無いこと（下の「落とし穴」）。

### 守ること
- 大きさは `Shape.z()`、余白は `Shape.s()`、文字は `T.name()` などの `T.*` と `T.px()` を通す（革の Size・Spacing・TextScale が効くように）。
- 文字は切らない。箱より広い時は `FittedBox(fit: BoxFit.scaleDown)` で縮める（長い名前は auto_size_text で2行まで）。
- `own.dart` は「いつもの見た目」の正本。触る時は見た目を1画素も変えない。

## 4. デザインの方針

Anthropic 公式の考え方（[frontend-design スキル](https://github.com/anthropics/skills/tree/main/skills/frontend-design)、[公式ブログ](https://claude.com/blog/improving-frontend-design-through-skills)、[cookbook](https://platform.claude.com/cookbook/coding-prompting-for-frontend-aesthetics)）と、このプロジェクトで決まったことを合わせると：

- **題材に根を張る**：Motolii は「アイデアで勝つ MV を最短で作る道具」。その世界の素材から選ぶ。ありがちな既定値（Inter、白地に紫のグラデ、同じ角丸カードの繰り返し、クリーム×セリフ×テラコッタ、黒×蛍光緑、全部大文字の小見出し）は選択ではない。
- **色は意味**：札の種類・層の種類など、意味がある時だけ色を付ける。名前から適当に色を選ばない。
- **大胆さは1か所**：選んでいる物と Render のような主役にだけ。ほかは抑える。
- **文字が性格**：太さと大きさは極端に（細と極太、3倍の跳び）。大文字だけの言葉は避ける。
- **密度**：Ableton のように一目で読める密度を保つ。説明の文やホバーの説明は足さない。動画の画面が主役。
- **動き**：触った結果、何が変わったかを見せる動きだけ。
- **作り方は2回**：まず色4〜6個・書体・ASCII の配置図を書き、依頼と照らして直してから、コードを書く。撮って自分で講評する。

今の方向：ユーザーはスイスの幾何学ポスター（Posterlad の Checkers、Xavito deVito のライブのポスター）に寄せたい。候補を絵で並べた時の推しは「ポスター（静かめ）」：色は種類の意味だけ、派手なのは選択と Render だけ、行の密度はミキサー卓のように詰める。今の `poster.dart` はまだ「名前から色」「大文字」「全部の札が派手」が残っている（直す対象）。

## 5. AI に頼む時の文

**値と絵の革**（置くだけで着られる）：

> `skin/book/skins/README.md` と `ROLES.md` を読んで、`~/.motolii/skins/<名前>/theme.xml` に「（欲しい見た目）」の革を書いて。`skin/book/skins/shoot.sh ~/.motolii/skins/<名前>` で撮って `preview.png` を見て、直して、また撮る。読めない文字や差し色の埋もれがなくなるまで。

**フォーム**（組み立てから変える）。届く部品は §3 の表。ドックのタブそのもの（`docking` が描く）はまだ外（§7）。

> `skin/book/skins/HANDOFF.md` を読んで、`lib/forms/<名前>.dart` に「（欲しい見た目）」のフォームを書き、登録して、`skin/book/skins/<名前>/theme.xml` で着る革を作って。先に色4〜6個・書体・ASCII の配置図を書いて「ありがちな所」を直してから書くこと。§3 の表の関数を全部書く（書かない関数は窓のいつもの絵のまま）。ドックのタブはまだ届かない。`skins/shoot.sh skins/<名前>` で撮って見て直す。最後に skin_fit_test に足して流して切れ0・はみ出し0を、`flutter test` で新しい落ち0を確かめる。

## 6. 落とし穴（踏んだ物）

- **テストの受け口**：integration test の中で `FlutterError.onError` を差し替えると、テスト本体の失敗まで吸い込み、途中で止まっても「合格」と出る。失敗させる前に受け口を戻す。結果は `reports/skins/fit.txt` にも書く（flutter drive の print は消えることがある）。
- **横に流れる行に Flexible**：スクロールする Row（例：舞台の下の DeskStrip）の子に Flexible/Expanded を入れると、窓が組めずに15件落ちた。
- **const**：`Shape.z()` などは getter なので、`const` の中では使えない。解析器の `const_*` の指摘の所の `const` を外す。
- **ROLES.md の書き方**：`` `名前` `` と書いた物は全部「名前」として照合される。値の例は普通の文字で書く。範囲（`Clip1`〜`Clip7`）でなく全部並べる。
- **数の範囲**：Size 0.85〜1.5、TextScale 0.85〜1.3、Spacing 0〜2、Corner 0〜10 の外は端に収める（`skin_fit_test` で決めた。外だとタブが消える・文字が切れる）。
- **テストの基準**：`flutter test` には前から約55件の落ちがある（スキンと無関係）。変更の前後で落ちの一覧を比べ、新しい物が0かを見る。
- **自動の札の色**：インスペクタの自動の札は、作った時の色を覚えていた。色は描く時に読む（`Fx.toneOf`）。

## 7. まだのこと

- フォームを通っている部品は、Render、Edit/Animate/Export、Space、オン・オフ、札、小さな切り替え（Fit、Layer/Group、Snap、Safe、Grid）、棚の絞り込み、タイムラインの層の名前と帯、数の入力欄。帯の角と地は `trackRadius` と `trackBand`。ドックのタブそのもの（`docking` パッケージが描く）は、まだフォームの外。
- フォームはアプリに組み込まれるので、配った人が置くだけでは着られない。
- 細い線（1画素）の太さは革で変わらない。
- 舞台の中（枠の外の暗い所も）は作品の絵なので、革では変わらない。

## 8. 履歴

`1004c21` 手引き → `6572bd0` 革はフォルダ1つ・2色で灰色の段・shoot.sh・名前の照合 → `220e361` `cdcebd5` 外の LLM の指摘 → `adef71e` 直書きの大きさ0・文字は切らない → `517985e` 無茶な値の守り → `283dee2` 枠（9分割） → `760d873` ヘッドレス UI（フォーム） → `56ce5b4` この引き継ぎ → 外の AI がこれだけでライトテーブルのフォームを書いた（7/10、指摘は §3 の約束の細かい所・使える材料・§5 に反映）
