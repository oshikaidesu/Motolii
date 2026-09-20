# 窓を一台の機械にする — 色と形の文法、専用部品

2026-09-20。利用者の UI 検討 memo(Ableton の密度 + OP-1 の所有感 + Nintendo の楽しさ +
Superflat Pop の平面 + Bauhaus/Swiss の規律)を、この repo の実物へ束ねた物。
**新しいレイアウトを作る話ではない。** 今の情報量と構造を維持したまま、
見た瞬間の魅力・触りたさ・所有欲・Motolii 固有の製品感を足す。

## 決まっている法

- **土台は dark のまま**(`EditorTheme.app #292929`)。明るくしない。
- **作品はキラキラ、机はフラット** — ガラス・光・noise は Stage に出る絵の質感で、
  窓の枠(panel・button・track header・timeline)は影・glass・gradient に頼らない。
- **Hide less, distinguish more** — 情報を減らして簡単に見せない。色・形・位置・字で区別する。
- 見た目の変更は **1 レーンずつ**。合否は利用者。

出自とも一致している: Motolii = Motley、**源流 JSR・育ち うごメモ・芯 AviUtl**。
Superflat Pop と Nintendo は後から足した趣味ではなく、最初からそこにある。

## 今の窓が既に持っている物

memo の中心(色 = 意味 = 操作)は、**名指しで実装済み**。[foundation/theme.dart:203](../../motolii/ui/lib/foundation/theme.dart)：

```dart
// Character families: what a number is for, told by hue (the OP-1 rule:
// one colour per family, on the glyph, the track and the handle alike).
static const spatial, amount, time, count, seed, angle;
```

| memo の要求 | 現状 |
|---|---|
| Material をそのまま使わない | **達成済み** — `WidgetsApp` 直、自前の `EditorTheme` / `EditorInk` |
| 色 = 意味 = 操作 | **半分実装、衝突中**(下記) |
| 情報密度を下げない | 維持 — `EditorMetrics` が既に密 |
| 形の文法 | 未着手 |
| 専用 widget | `foundation/` 7 file・`panels/` 20 以上。素地はある |
| `ThemeExtension` で統一 | 未使用(自前の `static const`) |

## 1. 色 — まず衝突を外す

同じ 6 色が **3 つの違う意味**で使われている。

| 色 | 家族(数の意味) | 層の識別 | 種類 |
|---|---|---|---|
| `#93a5f5` | spatial(位置) | 層 #0 | Shape |
| `#eedb73` | seed(種) | 層 #1 | Text |
| `#c18bd3` | time(時刻) | 層 #2 | — |
| `#79c4ca` | count(数) | 層 #3 | — |
| `#e6a275` | amount(量) | 層 #4 | — |
| `#95c78b` | angle(角) | 層 #5 | — |

`identityColors` は家族の色を並べ替えただけで(7 番目以外すべて一致)、
層の識別は `layerColor(id) = identityColors[id % 7]` の**任意割り当て**。
つまり **同じ青が「位置の値」と「3 番目の層」の両方を意味している**。
OP-1 の規則は「1 色 = 1 つの意味」なので、ここが破れている限り色を足しても効かない。

使用箇所: `layerColor` は timeline 4・inspector 1・depth_desk 1。
`kindColor` は inspector 1・browser 2。

### 案 — 消すのではなく、色域を離す

**層(レイヤー)の識別色は残す。** 一度「識別を色以外へ逃がす」案を書いたが、それは原則に反していた
(2026-09-20 利用者の指摘で撤回):

- **家族は閉じた 6 種**。意味が固定されていて増えない。
- **層は開いた集合**。作る人が増やし続け、一つ一つが別物になる。**個性が生まれるのはこちら側**。
- 密な timeline を目で走査する時、色は最強の識別子。**色を消したら区別が減る** ——
  これは「Hide less, **distinguish more**」の反対。Ableton がトラック色を持つのも同じ理由。

本当の問題は「識別色がある」ことではなく、**2 つが同じ色域を共有している**こと。
離せばよい。役割は既に分かれている:

| | 出る場所 | 面積 |
|---|---|---|
| 家族(数の意味) | glyph・track・handle・property の tint | **小さい印** |
| 識別(どの層か) | timeline の行・clip、Depth の物、Inspector の頭 | **広い面** |

同じ色相が「小さい印」と「広い面」で別の意味を持つのは、OP-1 の規則(1 色 = 1 意味)には反する。
役割で分かれているので実害は今の所小さいが、**色を足す前にここを明示的に離す**。

1. **識別に自前の定数を持たせる** — 今は `identityColors` が家族の色の並べ替え
   (7 番目以外すべて一致)。別の定数にして、2 つが独立に動けるようにする。
2. **register を分ける** — 広い面は dark の机の上に乗るので、家族の淡い帯より
   **深く・彩度を落とした帯**が合う。小さい印は今の淡い帯のまま目立たせる。
   同じ色相でも「濃さの帯」が違えば、意味を取り違えない。
3. **いずれ層の色は作る人が選べるようにする**(Ableton の型)。個性が生まれるのがこちら側なら、
   `id % 7` の任意割り当てではなく**本人が決める**のが筋。今回の範囲ではないが、
   1・2 はその前提を壊さない形にしておく。

**種類の色**(`kindColor`)は棚で効いている。種類は数える程度
(File・Shape・Text・Group・Camera・Stage・Null・Particles)なので、**形の記号を主、色を従**に。

強い色は希少資源。neutral の面積を十分残す。

## 2. 形 — 文法にする

色だけでなく形にも意味を持たせる。既にある物へ束ねる。

| 形 | 意味 | 今どこに居るか |
|---|---|---|
| 円 | 連続値・つながり | parameter の handle、connector の端 |
| 四角 | 状態・item | timeline の clip、browser の tile |
| 細長い矩形 | 時間・長さ | clip の尺、ruler |
| 三角 | 再生・向き | transport、playhead |
| 色点 | active / enabled / linked | toggle、automation の印 |

**厳密な規則は絵を見てから決める。** 先に決めるのは「形にも文法を持たせる」という方針だけ。

## 3. 専用部品 — 目に入る順に 10 個

汎用 Button / Slider で全部済ませない。頻繁に見る物には Motolii 固有の姿を与える。
今の窓で面積と滞在時間が大きい順:

| # | 部品 | 今の場所 |
|---|---|---|
| 1 | Clip(timeline の 1 本) | `panels/timeline.dart` |
| 2 | Track header(lane の頭) | `panels/timeline.dart` |
| 3 | Parameter(数の欄 + 取っ手) | `foundation/panel_controls.dart` |
| 4 | Toggle | `foundation/panel_controls.dart` |
| 5 | Playhead | `panels/timeline.dart` |
| 6 | Effect strip(Inspector の効果の行) | `panels/inspector.dart` |
| 7 | Media item(棚の 1 枚) | `panels/browser/tile.dart` |
| 8 | Transport | `app/editor_window.dart` |
| 9 | Node / connection | `panels/desk.dart` ほか |
| 10 | Panel の縁(区画の線) | `foundation/theme.dart` の `line` / `border` |

**10 個に固有の姿があれば製品感はかなり変わる。** 全部を一度に作らない。
1 個ずつ、実窓で見て合否を取る。

## 4. やらないこと(この repo の具体で)

- 角丸カードの中に角丸カードを入れない。**一つの筐体の中に区画がある**形にする。
- 余白を増やして高級感を作らない。`EditorMetrics` の圧縮率を維持する。
- 影・glass・gradient で階層を作らない。線・色面・配置で作る。
- 状態を操作した瞬間だけ見せない。active / selected / muted / automated / linked は**画面に残す**。
- 反応は明確だが過剰にしない — hover で微かな色変化、press で 1px 沈む、
  active で色点が残る、selection で明確な outline、drag 中に対応箇所だけ反応。
- Material をそのまま使わない(既に使っていない)。Nothing OS のようなほぼモノクロにもしない。
- OP-1 の外見を写さない。取るのは**規則**(色 = 意味 = 操作、専用部品、閉じた一台)であって絵ではない。

## 進め方

1. **色域を離す**(1 レーン)。識別に自前の定数を与え、広い面の register を深い帯へ。
   `layerColor` の 6 箇所と `kindColor` の 3 箇所。**最初にここ** — 土台なので。
   **識別色は消さない。**
2. `ThemeExtension` へ寄せるか決める。記憶の「標準の仕組みが先」に従うなら本体の仕組み。
   ただし今の `static const` で困っていないなら急がない。
3. 専用部品を 1 個ずつ。Clip → Track header → Parameter の順が効く(面積が大きい)。

各段で実窓の絵を見て、合否は利用者が出す。code は直すが、直すかどうかは利用者が決める。

## 参考(memo より)

- [Ableton – Learn Live: Interface](https://help.ableton.com/hc/en-us/articles/360000134170-Learn-Live-Videos-Interface)
- [Teenage Engineering – OP-1 layout / color coding](https://teenage.engineering/guides/op-1/original/layout)
- [Nintendo – UI/UX デザインの仕事](https://www.nintendo.co.jp/jobs/introduction/design/work03.html)
- [Nintendo Labo – Toy-Con Garage](https://www.nintendo.com/jp/labo/invention/index.html)
- [CARI – Superflat Pop](https://cari.institute/aesthetics/superflat-pop) / [Four Colors](https://cari.institute/aesthetics/four-colors)
- [Animal Crossing UI – Flutter 実装例](https://github.com/sazardev/animal_crossing_ui)(見た目を写すのではなく、
  Flutter の素の部品から独自の design system を組む方法を見る用途)
