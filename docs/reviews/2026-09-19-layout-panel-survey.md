# Layout の欄を「道具」の形にする — 先例の調査と Motolii の案(2026-09-19)

対象: Inspector の `Layout` 節(`motolii/ui/lib/panels/inspector.dart:769` `_layoutRows` が `layout.*` を表の順に全部並べる物)。欄の正本は `motolii/crates/motolii-doc/src/store/layout.rs` の `GROUP_ROWS`(31 行)・`ITEM_ROWS`(18 行)・`SPACE_ROWS`(23 行)。利用者の言: 「壁みたいに欄が並ぶ。CSS はもっとかっこよくできる」。Ease / Transition の行は本題ではない(ease は host / timeline に既にある)。調査のみ、code は触っていない。出典は末尾、本文の [n] が対応。

## 要約 5 行

1. **最良の道具は Layout を「絵の欄」にしている**: 向き・揃え・間合いを文字の選択肢ではなく **glyph の並び(3×3 の揃えの箱、↔↕ の向き、⇋ の折り返し)** で見せ、数字は Gap と Padding の 2 つだけ残す(Figma [1][2]、Penpot [5]、Webflow [7]、Chrome DevTools [10])。
2. **大きさの語は 3 つで足りる**: Hug / Fill / Fixed(Figma)= Fit / Fill / Fixed(Framer [6])= Fit content / Fix / Fill(Penpot)。Width / Height の数字は Fixed の時だけ意味を持ち、それ以外は灰。Motolii の `SIZING` は既にこの 3 語 [layout.rs:148]。
3. **主戦場は canvas**: Gap と Padding は Stage 上の桃色の掴み(Figma の pink handles、Adobe XD の帯、Webflow の「gap をつまんで引く・Shift で両方」[8][9])、格子の線は canvas の端の pill(Figma Grid [3])。panel の数字は「読み」と「打ち込み」の口に下がる。
4. **段階的な開示が共通**: 既定で見えるのは 5〜7 個(Flow / Wrap / Align 箱 / Gap / Padding / W / H)、それ以外は `⋯` の奥(Figma の「Auto layout settings」: Spacing mode・Strokes・Canvas stacking・Text baseline)、または **親が Flex / Grid の時だけ子に現れる**(Ignore auto layout・Align self・Span)。
5. Motolii の案: 72 行を **既定 6 行 + 子 3 行**に畳む。Display は segmented の glyph、Direction + Wrap + Justify + Align を **1 つの揃えの箱 + 向きの 2 つ**にまとめ、Gap / Padding は Stage の掴みと結ぶ。Transition・Loop・Stagger・Field・Offset Path・Shape Outside・Anchor・Constraint・Clip・Shadow は Layout から外へ(それぞれの棚へ)。

## 事例の表(道具・何が見える・何が隠れる・出典)

| 道具 | 既定で見える物 | 隠れる物(どこに) | canvas 側 | 出典 |
|---|---|---|---|---|
| **Figma Auto layout**(UI3, 2024) | Flow 3 択の glyph(Vertical / Horizontal / Grid)・Wrap・Gap 1 欄・Padding 2 欄(縦 / 横)・**揃えの箱 3×3**・W / H の dropdown(Hug contents / Fill container / Fixed) | 4 辺の Padding(⌘-click で開く)・Min / Max(dropdown の「Add min width」)・Auto gap の Between / Evenly / Around(settings modal、揃えの箱で `X` で切替)・Strokes in layout・Canvas stacking(First / Last on top)・Text baseline alignment | 桃色の掴みで Gap / Padding をドラッグ、Alt で両辺、Shift で大きい刻み。子は `Ignore auto layout`(旧 Absolute position)で流れの外、Ctrl-drag でも | [1][2][4] |
| **Figma Grid**(2025) | Grid picker(Number of columns / rows、rows は auto 既定)・Gap between rows / columns・Horizontal / Vertical padding | 線ごとの Fixed / Fill(fr)/ Hug / Min / Max は canvas の pill か dropdown。Column span / Row span は子が Fill container の時だけ効く。Toggle automatic positioning | 枠の上端 / 左端に **青い pill**(線の太さの札)、端を引いて Fixed に。⌘ / Shift で複数の線を選ぶ | [3] |
| **Penpot Flex / Grid**(OSS, CSS の語) | Direction 4 択 glyph・Wrap・Align items / Justify content の glyph 行・Gap の row / column・Padding 4 欄・子の Sizing(Fit / Fix / Fill)| 子の Position(Static / Absolute)・Z-index・Margin・Min / Max | Spacing は click-drag、Shift で対辺、Alt で 4 辺。Grid は「Edit grid」か double-click で線の頭(header)に掴み: 太さ・並べ替え・追加 / 削除。子は Auto / Manual / Area | [5] |
| **Framer Stack / Grid** | Direction・Distribute・Align・Gap・Padding・Wrap。子の Width / Height は Fixed / Relative / Fill / Fit Content / Viewport の dropdown | Position(relative / absolute)は子の Layout 節 | Grid は fr の Fill、parent 側で gap と min column width | [6] |
| **Webflow Style panel** | Display の glyph 行(Flex / Grid…)・Direction(Horizontal / Vertical + reverse)・Align / Justify の glyph・Children wrap・Gap column / row | 子の Sizing(Shrink if needed / Grow if possible / Don't shrink or grow / Custom = basis / grow / shrink)・Order・Align self は「子を選ぶと出る」 | Grid は **canvas で「Edit grid」**: `+` で線を足す、FR / px / minmax を線の札で、gap をつまんで引く(Shift で行と列を同時)、子を升目へ drag、span は桃色の札で。Esc / Done で抜ける | [7][8][9] |
| **Sketch Smart Layout** | 向きの **3×2 の矢印 glyph**(Left to Right / Right to Left / Center / Top to Bottom / Bottom to Top / Middle)・Min width か Min height(向きで片方だけ) | 間合いは Smart Distribute の掴みへ(panel に数字を持たない) | Smart Distribute の掴みで等間隔をドラッグ | [11] |
| **Adobe XD Stacks** | Responsive Resize の中の Stacks チェック・Vertical / Horizontal・Padding の toggle(1 値か 4 値) | Padding の 4 値は toggle の奥 | 間合いは **canvas の帯を直接引く**、並べ替えも drag | [12] |
| **Chrome DevTools** | Elements の `flex` / `grid` バッジ、Styles の `display: flex` の横に **flexbox editor**(direction / wrap / align-content / justify / align-items の glyph、direction で glyph が回る) | Layout pane に一覧・色・線番号 / 名前 / track sizes / area names の toggle | overlay: 点線の箱・gap の斜線・線番号(`1fr - 96.66px` のように書いた値と解いた値の両方) | [10] |
| **Josh Comeau の Flexbox / Grid ガイド** | playground: direction / justify / align / gap / grow / shrink / basis / wrap を 1 つずつ動かす図。主軸・交差軸の矢印が direction で回る | — | 「hypothetical size(仮の大きさ)」「content(群)と items(個)」の語で justify-content と align-items の違いを絵で | [13][14] |
| **MDN flexbox** | main axis / cross axis、main-start / main-end の図 4 枚 + live example | — | — | [15] |
| **Flexbox Froggy / Grid Garden** | 24 / 28 段。Froggy は justify-content → align-items → flex-direction → order → align-self → flex-wrap → flex-flow → align-content の順(蛙 = item、蓮 = 目標)。Garden は grid-column-start / end → span → grid-area → grid-template-columns(fr / repeat) | — | 絵そのものが結果、code は 1 行ずつ | [16][17] |
| **Cavalry Layout Group / Grid Layout** | Direction・Wrap Mode(No Wrap / Wrap / Wrap Reverse)・Spacing・**Spacing Mode**(Manual Spacers / Flex Start / Flex End / Centre / Space Between / Around / Evenly)・Margins 4 辺・Min / Max Size・Line Spacing・Align Content・Ordering Policy(Layer Order / Alphabetical / Shuffle)・Flip Order。Grid: Columns / Rows・Horizontal / Vertical Padding・Cell Margins・Direction(Flow Columns / Flow Rows)・Scale to Fit・Keep Aspect Ratio | 子の Vertical / Horizontal Layout Alignment・Exclude from Layout は **Advanced tab** | Draw Extents で枠を描く。AE 型の属性の縦並びで glyph は無い(反面教師: Motolii の今と同じ壁) | [18][19][20] |
| **C4D Cloner** | Mode 1 つ(Object / Linear / Radial / Grid Array / Honeycomb)で **下の欄が総入れ替え**。Linear: Count / Offset / Mode(Per Step / End Point)/ P.S.R、Radial: Count / Radius / Plane / Align / Start / End、Grid: Count xyz / Size / Form / Fill | Mode が違う欄は見えない | Radius 等は viewport の掴み | [21] |
| **AE 拡張(Gridder 2 / GridGuide / Grid Composer)** | AE の小 panel に glyph ボタン: Gridder は rectangle / 3D / circular / spiral のモード、GridGuide は「線をクリックするたびに次の格子線へ跳ぶ」揃え、Grid Composer は drag-and-drop の格子デザイナ | — | comp の上に格子を重ねる。AE 本体には無いので拡張が canvas に描く | [22][23] |

## 共通の文法

1. **向き・揃えは glyph、数字は 2 つだけ**。Figma・Penpot・Webflow・DevTools が同じ絵を使う: 向きは `→ ↓`(+ reverse)、折り返しは `⇋`、揃えは **3×3 の点の箱**(縦 = align-items、横 = justify-content)。Auto gap(Space Between 等)の時は箱が **3 列だけ**に減る [2]。数字は Gap と Padding のみ。
2. **大きさは Hug / Fill / Fixed の 3 語**。W / H の欄の頭に語の dropdown、Fixed の時だけ数字が効く(Figma [1]、Framer は Fit Content / Fill / Fixed + Relative / Viewport [6]、Penpot は Fit content / Fix / Fill [5])。Min / Max は dropdown の「Add min width」の奥。
3. **canvas が主、panel は読み**。Gap / Padding は桃色の掴み(Figma [2]、XD [12]、Penpot [5]、Webflow [9])。格子の線は枠の端の pill(Figma [3])か線の頭(Penpot、Webflow の Edit grid)。DevTools は逆に「見せる」だけの overlay(gap の斜線・線番号・`1fr - 96px` の 2 値表示)[10]。
4. **modifier で対称**: Alt(Option)で対辺を同時、Shift で行と列(または大きい刻み)、⌘-click で 4 辺の欄を開く。全社ほぼ同じ。
5. **段階的な開示は 3 段**: (a) 既定 5〜7 個、(b) `⋯` / settings の奥(Spacing mode・Strokes・Stacking・Baseline・Min / Max)、(c) **文脈で出る**(親が Flex / Grid の時だけ子に Ignore auto layout・Align self・Span・Sizing が出る。C4D は Mode で欄が総入れ替え)。
6. **流れの外は 1 つのボタン**: Figma「Ignore auto layout」、Penpot「Absolute」、Framer「absolute」。Ctrl-drag でも同じ。
7. **順は別の節**: Ordering Policy / Flip Order(Cavalry)、Order(Webflow)、Canvas stacking(Figma settings)。Layout の中心には置かない。
8. **教材は「軸が回る」を 1 枚で教える**: 主軸 / 交差軸の矢印が direction で回る(MDN・Comeau・DevTools の glyph 回転)。3×3 の箱も主軸に沿って読み替えるだけで済む。

## Motolii の Layout の案(mockup)

### 方針

- 欄の正本(`layout.rs`)は変えない。変えるのは **Inspector の見せ方と Stage の掴み**。`_layoutRows` が表の順に全部並べる所を、節ごとの専用 widget にする。
- 名前は CSS の語のまま(裁定 2026-09-14「名前は CSS の語、大きさの決め方だけ Figma」)。glyph 化しても tooltip は CSS の語。
- Ease / Transition は host / timeline にある(本題外)。ここでは **Layout の節から出す**判断だけ書く。

### 何が残り、何が glyph になり、何が Stage へ行き、何が出て行くか

| 欄(layout.rs) | 行き先 | 形 |
|---|---|---|
| Display(None / Flex / Grid) | 残す、節の頭 | segmented glyph 3 つ `▭ ⇉ ⊞`。None なら節はこの 1 行だけ |
| Flex Direction(Row / Column / Row Rev / Column Rev / Depth) | 残す | glyph 2 つ `→ ↓` + reverse は同じボタンの再押しで `←`、Depth は 3 つ目 `⊙`(奥へ) |
| Flex Wrap | 残す | toggle glyph `⇋`(Row の時だけ出す、Figma と同じ) |
| Justify Content + Align Items | **1 つの 3×3 の箱** | 箱の点 = Start / Center / End の 9 通り。Space Between / Around / Evenly は箱の右の `⋯` か、箱上で `X` を押すと 3 列に変わる(Figma の写し)。Stretch は箱の上の `↕` toggle |
| Depth Alignment | Direction = Depth の時だけ | 3×3 の箱の奥行き軸として `⊙` 列 |
| Gap | 残す(数字 1 つ)+ **Stage の掴み** | Grid の時は row / column の 2 欄(`⌥` で連動) |
| Padding | 残す(縦 / 横の 2 欄)+ **Stage の掴み** | ⌘-click で 4 辺に展開(Figma)。今の `Vec2` は縦横 2 値なので既存のまま |
| Horizontal / Vertical Sizing + Width / Height | 残す、**2 行に畳む** | `W [Hug ▾] 400` / `H [Fill ▾] 300`。Fixed 以外は数字が灰。子の側も同じ 2 行 |
| Grid Columns / Rows + `layout.column.n` / `row.n` | 残す(Grid の時だけ) | `⊞ 3 × auto` の 1 行。線ごとの fr は **Stage の枠の端の pill** に出し、panel には持たない(Figma Grid) |
| Overflow(Visible / Clip / Bounce) | 残す | glyph 3 つ。Bounce は Motolii だけの物なので語で tooltip |
| Position Type(Relative / Absolute) | 子に、**1 つの toggle** | `⌖ Ignore layout`(Figma の語を借りる。tooltip は `position: absolute`)。Stage で Ctrl-drag でも |
| Align Self | 子に、親が Flex / Grid の時だけ | 親の 3×3 の箱の小型(1 列 = 交差軸だけ) |
| Column / Row Start / Span | 子に、親が Grid の時だけ | Stage で升目を drag、span は桃色の札(Webflow)。panel は `⊞ c2 r1 ×2×1` の 1 行 |
| Snap to Grid / Snap Size / Object Fit | 子、`⋯` の奥 | — |
| Horizontal / Vertical Constraint | **Layout から出す** → Figma と同じく `Constraints` の小節(流れの外の子だけ) | 十字の glyph 1 つ(Figma の constraint picker) |
| Exclusions・Layout Rotation / Tilt | `⋯` の奥 | — |
| Background / Border Radius / Shadow ×4 / Clip ×5 | **出す** → `Box`(Fill / Stroke と同じ棚、CSS の box) | Layout ではない |
| Stagger / Stagger From / From End / Loop ×2 / Transition ×3 | **出す** → timeline / Ease desk(既に ease がある) | 時刻の欄 |
| Margin・Flex Shrink・Hardness・Heaviness | **出す** → `Space`(間合いの法、全ての物)。Flex Shrink は親が Flex の時だけ | Margin は Stage の掴み(Figma の padding と同じ絵、外向き) |
| Field ×5・Offset Path ×3・Shape Outside ×2・Transform Origin | **出す** → `Field` / `Path` / `Text` の各棚 | 関係と場の欄 |
| Position Anchor ×2 + Position Area | **出す** → `Anchor`(関係の型「名指し」)。Area は 3×3 の箱の再利用(外側 9 か所) | 同じ glyph、相手の箱の外側に置くだけ |

Layout の節に残るのは **親 6 行 + 子 3 行**。72 行 → 9 行。

### mockup(行 20 px、文字 11、glyph 列 22 px、Inspector の `_line` = glyph | slot | slot | slot | tail の型)

Group(Display = Flex, Row, Wrap):

```
┌ Layout ──────────────────────────────────────── ⋯ ┐
│ ⇉  [▭][⇉][⊞]                                      │  Display: None / Flex / Grid
│ →  [→][↓][⊙]  [⇋]        ┌·  ·  ·┐               │  Direction / Wrap        ┐ 揃えの箱 3×3
│                          │·  ●  ·│  [↕]           │                         │ (Justify × Align)
│                          └·  ·  ·┘               │                         ┘ ↕ = Stretch
│ ⟷  Gap  [ 12 ]           Pad  [ 16 ] [ 16 ]   ⌘  │  ⌘-click で 4 辺
│ W  [Hug ▾] [ 400 ]       H  [Fill ▾] [ 300 ]      │  Fixed 以外は数字が灰
│ ▣  [◻][⊡][⤾]                                      │  Overflow: Visible / Clip / Bounce
└────────────────────────────────────────────────────┘
```

Group(Display = Grid):

```
│ ⊞  [▭][⇉][⊞]                                      │
│ ⊞  Cols [ 3 ]  Rows [ auto ]   ┌·  ·  ·┐          │  線ごとの fr は Stage の pill
│                                │·  ●  ·│  [↕]     │
│                                └·  ·  ·┘          │
│ ⟷  Gap  [ 12 ] [ 12 ] ⌥        Pad  [ 16 ] [ 16 ] │  ⌥ で行と列を連動
│ W  [Hug ▾] [ 400 ]       H  [Fill ▾] [ 300 ]      │
│ ▣  [◻][⊡][⤾]                                      │
```

Item(親が Flex):

```
┌ Layout ────────────────────────────────────────── ┐
│ ⌖  [ Ignore layout ]     ┌ · ┐                    │  position: absolute / align-self(交差軸のみ)
│                          │ ● │                    │
│                          └ · ┘                    │
│ W  [Fill ▾] [ 100 ]      H  [Hug ▾] [ 100 ]       │
└────────────────────────────────────────────────────┘
```

Item(親が Grid): 2 行目が `⊞ c[ 2 ] r[ 1 ]  ×[ 2 ][ 1 ]`(Start と Span)。`Ignore layout` が On なら `Constraints` の小節(十字 1 つ)が下に出る。

Display = None: 1 行目だけ。

### Stage の掴み(canvas 側)

- Group を選ぶと **Gap の帯と Padding の帯**が桃色で出る(Figma / XD)。帯を引くと数字が動き、⌥ で対辺、⇧ で行と列(Grid)。Motolii の `previewProperties / commitPreview` の道に乗る(既にある「触ると先に絵が動く」の装置)。
- Grid の Group は枠の上端 / 左端に **線の pill**(`1fr` / `120`)。引くと Fixed、click で fr / px / Hug の切替。`layout.column.n` の鍵は pill から打つ。
- 子は升目へ drag、隣の升目まで引くと span(桃色の札 `×2`)。Ctrl-drag で流れの外へ(`Ignore layout` が On になる)。
- DevTools の overlay を **表示だけ**として借りる: Group を hover すると点線の箱・gap の斜線・線番号。書いた値と解いた値の両方(`1fr · 96px`)。

### 必要な glyph の一覧

| 用途 | glyph | 出典の絵 |
|---|---|---|
| Display None / Flex / Grid | `▭ ⇉ ⊞` | Figma の Flow 3 択、Webflow の Display 行 |
| Direction Row / Column / Depth | `→ ↓ ⊙`(reverse は `← ↑`) | Penpot、DevTools(direction で回る) |
| Wrap | `⇋` | Figma の Wrap、Cavalry の Wrap Mode |
| 揃えの箱 3×3(+ Space の 3 列) | 点 9 個、選択は `●` | Figma の alignment box、`X` で 3 列 |
| Stretch | `↕`(Column なら `↔`) | DevTools の align-items: stretch |
| Gap / Padding | `⟷` / `⊟`、Stage は桃色の帯 | Figma の pink handles |
| Sizing Hug / Fill / Fixed | `⊃⊂` / `⊂⊃` / `⊏⊐`(dropdown の頭) | Figma の Hug / Fill / Fixed の icon |
| Overflow Visible / Clip / Bounce | `◻ ⊡ ⤾` | Figma の Clip content、Bounce は Motolii |
| Ignore layout | `⌖` | Figma の Ignore auto layout(旧 Absolute position) |
| Grid の線の pill | 枠の端の丸札 | Figma Grid の blue pill |
| Span の札 | `×2` の桃色の札 | Webflow の pink label |
| Constraints | 十字(Figma の constraint picker) | Figma Constraints |

いずれも `Glyph.*` の既存の型(`_named(Glyph.center_focus_strong, …)`)に載る。新しい絵は揃えの箱と Sizing の 3 つだけ。

### 先例から借りる 3 つ(強い順)

1. **揃えの箱 3×3 に Justify + Align + Wrap の 3 行を畳む**(Figma / Penpot / DevTools 共通)。文字の選択肢 2 つ × 6 択が点 1 つになり、主軸が回っても読み方は同じ。
2. **Gap / Padding を Stage の桃色の掴みにして panel は読みに下げる**(Figma / XD / Webflow)。Motolii には既に preview の道があるので、帯を引く → `previewProperties` → 確定で足りる。
3. **子の欄は親の Display で出す・消す**(Figma の Ignore auto layout、Webflow の子 Sizing、C4D の Mode)。`ITEM_ROWS` は既に親が Flex / Grid の時だけ渡る仕組みなので、見せ方だけ 1 toggle + 小箱 + W / H に絞る。

### 待ち項目(利用者裁定)

- Layout から出す先の棚の名前(`Box` / `Space` / `Anchor` / `Field`)。名前の発明は待つ。
- `Ignore layout` の語(Figma 借用)か `Absolute`(CSS)か。今の欄名は `Position Type: Relative / Absolute`。
- Bounce の glyph。

## 出典

1. Figma Learn「Guide to auto layout」 https://help.figma.com/hc/en-us/articles/360040451373 — Flow 3 択、Wrap、Padding、Gap(Between / Around / Evenly)、Hug contents / Fill container / Fixed、Min / Max の dropdown、3×3 の alignment box、Ignore auto layout
2. Figma Learn「Use the horizontal and vertical flows in auto layout」 https://help.figma.com/hc/en-us/articles/31289464393751 — 桃色の掴み、⌘-click で 4 辺、Auto gap で箱が 3 列、`X` で切替、矢印キーで箱を動かす、settings(Text baseline / Strokes / Canvas stacking)
3. Figma Learn「Use the grid auto layout flow」 https://help.figma.com/hc/en-us/articles/31289469907863 — grid picker、Number of columns / rows(auto)、青い pill、線ごとの Fixed / Fill(fr)/ Hug / Min / Max、Gap between rows / columns、Column span / Row span(Fill container が条件)、Toggle automatic positioning
4. Figma Blog「Config 2024 In Review」 https://www.figma.com/blog/config-2024-recap/ — UI3、Wrap / Min / Max、Ignore auto layout(旧 Absolute position)、Ctrl-drag
5. Penpot User Guide「Flexible layouts」 https://help.penpot.app/user-guide/flexible-layouts/ — Flex / Grid の欄、Sizing(Fit / Fix / Fill)、Static / Absolute、Z-index、Shift / Alt の drag、Edit grid の線の頭、Auto / Manual / Area
6. Framer Academy「Build layouts that resize correctly」 https://www.framer.com/academy/lessons/sizing-modes、「Stacks and relative positioning」 https://www.framer.com/academy/lessons/framer-fundamentals-stacks-and-relative-positioning、「Stacks vs grids」 https://www.framer.com/academy/lessons/framer-fundamentals-stacks-vs-grids — Fixed / Relative / Fill / Fit Content / Viewport、Stack の gap / padding / alignment / distribution
7. Webflow University「Flexbox」 https://university.webflow.com/videos/intro-to-flexbox、Help Center「Flexbox」 https://help.webflow.com/hc/en-us/articles/33961260795155-Flexbox — Display / Direction / Justify / Align / Wrap / Gap、子の grow / shrink / basis / order / align
8. Webflow University「Grid」 https://university.webflow.com/videos/grid-2-0 — Edit grid、FR、右 click で単位、gap を drag、Shift で両方、Auto / Manual、span の桃色の札、Esc / Done
9. Webflow Updates「Drag to resize grid columns, rows, and gaps」 https://webflow.com/updates/drag-to-resize-grid-columns-rows-and-gaps
10. Chrome DevTools「Inspect CSS flexbox」 https://developer.chrome.com/docs/devtools/css/flexbox、「Inspect CSS grid」 https://developer.chrome.com/docs/devtools/css/grid — flex / grid バッジ、flexbox editor の glyph(direction で回る)、Layout pane、線番号 / 名前 / track sizes(`1fr - 96.66px`)/ area names / extend lines
11. Sketch Docs「Smart Layout」 https://www.sketch.com/docs/designing/smart-layout/ — 3×2 の向きの glyph、向きで Min width / height が片方だけ、Smart Distribute の掴み
12. Adobe XD「Create dynamic designs with stacks」 https://helpx.adobe.com/xd/help/create-dynamic-designs-with-stacks.html、「Set fixed padding」 https://helpx.adobe.com/xd/help/set-fixed-padding-for-components-groups.html — Stacks チェック、Vertical / Horizontal、canvas の帯、Padding の 1 値 / 4 値
13. Josh W. Comeau「An Interactive Guide to Flexbox」 https://www.joshwcomeau.com/css/interactive-guide-to-flexbox/ — 主軸 / 交差軸が回る図、hypothetical size、content と items の語
14. Josh W. Comeau「An Interactive Guide to CSS Grid」 https://www.joshwcomeau.com/css/interactive-guide-to-grid/ — 点線の線、fr と % の違い、justify-items / align-items、grid-template-areas
15. MDN「Basic concepts of flexbox」 https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_flexible_box_layout/Basic_concepts_of_flexbox — main / cross axis、main-start / end の図 4 枚、live example
16. Flexbox Froggy https://flexboxfroggy.com/(24 段の順は https://github.com/ckvignesh/flexboxFroggySolution で確認)
17. Grid Garden https://cssgridgarden.com/
18. Cavalry Docs「Layouts」 https://cavalry.studio/docs/nodes/shapes/layouts/layouts-intro/、「Layout Group」 https://cavalry.studio/docs/nodes/shapes/layouts/layout-group/ — Direction / Wrap Mode / Spacing / Spacing Mode / Margins / Min / Max / Line Spacing / Align Content / Ordering Policy / Flip Order、Advanced tab の Exclude from Layout
19. Cavalry Docs「Grid Layout Group」 https://cavalry.studio/docs/nodes/shapes/layouts/grid-layout-group/、「Apply Layout > Grid Layout」 https://cavalry.studio/docs/nodes/behaviours/apply-layout/grid-layout/ — Columns / Rows / Horizontal / Vertical Padding / Cell Margins / Flow Columns / Flow Rows / Scale to Fit / Keep Aspect Ratio / Draw Extents
20. Cavalry Docs「Distribution types」 https://cavalry.studio/docs/nodes/general/distribution-types/ — Linear / Grid / Circle / Path / Random 等 21 種
21. Maxon「Cloner Object: Object Tab」 https://help.maxon.net/c4d/s22/us/html/OMOGRAPH_CLONER-ID_OBJECTPROPERTIES.html — Mode(Object / Linear / Radial / Grid Array / Honeycomb)、Mode ごとの欄の入れ替え、Per Step / End Point、Radius の掴み
22. aescripts「Gridder 2」 https://aescripts.com/gridder/(rectangle / 3D / circular / spiral、C4D Cloner の写し)、「GridGuide」 https://aescripts.com/gridguide-for-after-effects/(click のたびに次の格子線へ)、「Grid Composer」 https://aescripts.com/grid-composer/(drag-and-drop の格子デザイナ)— ページは 403 で本文は検索の要約のみ
23. csslayout.io — 2026-09 時点でドメインが売りに出ており本文なし(不採用)

取れなかった物: Webflow Help Center と aescripts は 403、Framer の help 旧 URL と Figma の旧 Grid URL は 404(上の別 URL で代替)。Flexbox Froggy / Grid Garden の段の順は GitHub の解答集から。

## 利用者との詰め(2026-09-19 午後)

- 結果は離散(折り返し・cell・Hug/Fill)、動きは連続(Transition で補間、鍵の ease とは別の役目)。Transition の行は timeline へ出さず、Layout の 1 行に畳む。
- 連続値の側を出す: 整列は XY pad(9 点に snap、数値は横に)、Gap / Padding は数値 + Stage のピンチと取っ手、Heaviness / Hardness は連続のまま。離散の飛びは snap の瞬間だけ。
- 連続値なら Inspector の文法(擦る・Shift の段・打ち込み)が効くので、**Layout desk は作らない**。Layout = Inspector の 6 行(Display / Direction / Align pad / Gap・Padding / Sizing / Transition)+ 子 2 行。72 行 → 8 行、新しい部品は無し(pad は既存)。
- 線引き: 棚の札(WGSL、開いた語彙)は manifest から機械的に行を作る。Transform / World / Layout(閉じた語彙)は専用の部品で組んでよい。増える時は Motolii が Inspector を直す。
- 着手は FFI の lane の後(inspector.dart を 2 レーンで触らない)。

## Layout の節(裏)(2026-09-19 夜、実装。同夜の裁定「意図で組み直す」後の形)

触ったのは `motolii/ui/lib/**` と `motolii/ui/test/**` だけ。`layout.rs` の欄も `snapshot.rs` も変えていない — 変えたのは Inspector の見せ方と Stage の当たり判定。行番号は作業樹。

### 裁定 4 つ → code

1. **Stage は群を掴まない**(`stage.dart:661` `_grabbable` = locked でなく `kind != 'Group'`。`_hit` :663、`_handles` :670、選択の籠 :1375 の 3 箇所が同じ述語)。群へ触る道は Timeline / Inspector だけ。test: `stage_spatial_gizmo_test.dart:170` — 選ばれた群の子の上を hover しても outlines / handles は空、押すと `select [子]`、群しか無い所を押すと select は増えない(marquee)。
2. **並べ方は格子だけ**。Direction / Wrap / Display の glyph 行は消した。`grid` の行 1 本 = `⊞` の switch(`layout.display` 2 ↔ 0)+ `layout.grid_columns` + `layout.grid_rows` の数字 2 つ(横一列 = rows 1、縦一列 = columns 1)。Flex の行(`flex_direction` / `flex_wrap`)は UI から触らず、既存の Flex 書類では Advanced の奥に今までの cell で値が読める(test `:340`)。
3. **言葉を出さない**。名前の列は `_mark`(`inspector.dart:856`、glyph + tooltip に CSS の語)。数字の欄の `label` は tooltip / semantics だけ。Hug / Fill / Fixed は glyph 3 つ。文字が出るのは Advanced の畳みと ease の choice(値)だけ。test `:279` が row 名 12 語の不在を見る。
4. glyph は既存の `IconData` で足りた: Hug = `unfold_less`(内向きの矢印)、Fill = `unfold_more`(外向き)、Fixed = `lock_outline`(`glyphs.dart:225`)。W の行は `RotatedBox` で 90° 回す(`_GlyphChoice.turns`)。CustomPaint は足していない。

### 判定は 1 箇所

- `inspector.dart:772` `_isGroup` = `layer['kind'] == 'Group'`(親の行は群の層だけ)。`:779` `_hasLayout` = 群、または `layout.position_type` の欄を持つ層(native が並ぶ子にだけ渡す ITEM_ROWS)。それ以外の層に Layout の節は出ない(SPACE_ROWS を持っていても)。
- 節の形を決める欄 6 本(`_layoutShape`)を `_live2` で見張る。数字の欄は自分の欄だけ(`_well`)。
- 書き込みは今までの道: 数字は `_write`、選択肢は `:811` `_writeChoices`(`previewProperties` → `commitPreview`、複数選択にも絶対値)。鍵・Animate は `_well` の `EditorLamp` のまま。

### 行と欄の対応(親 = 群)

| 線(`ValueKey('layout:…')`) | 場所 | 部品 | layout.rs の欄 |
|---|---|---|---|
| `grid` | `:907` | `⊞` compact switch + 数字 2 つ。Off(None)ならこの 1 行だけ、数字の欄は空 | `layout.display`(0 / 2)、`layout.grid_columns`、`layout.grid_rows` |
| `gap` | `:937` | 数字 3 つ | `layout.gap`、`layout.padding`(Vec2 の 0 / 1) |
| `align` | `:981` `_alignLine` | 既存の `EditorPad` に `unit` / `snaps` を足した 9 点の箱(`panel_controls.dart:1528`、`snapped` :1539)。横 = Justify、縦 = Align。送る値は最寄りの snap、点は指に付いて来て離すと snap に落ちる、Shift で点も snap | `layout.justify_content`(横: 0 Start / 2 Center / 1 End)、`layout.align_items`(縦: 1 Start / 3 Center / 2 End)。**Grid の時 native はこの 2 欄を panel に渡さない**(`ui/native/src/snapshot.rs:379` の `flex` 除外)が、書類の格子は読む(`layout.rs:1692-1693`)ので pad は `always: true` で書く(`:1015`)。表示は欄が無い間 既定(Start / Stretch = 中央)。他 lane で snapshot.rs の除外を外せば表示も揃う |
| `width` / `height` | `:1032` `_sizingLines` | glyph 3 つ(Hug / Fill / Fixed)+ 数字、Fixed 以外は 45 % で触れない | `layout.horizontal_sizing` + `layout.width`、`layout.vertical_sizing` + `layout.height` |
| `transition` | `:959` | 数字 + ease の choice | `layout.transition_duration`、`layout.transition_easing` |

### 子(親が Grid)

| 線 | 場所 | 部品 | 欄 |
|---|---|---|---|
| `ignore` | `:1079` | `⌖` の switch(tooltip `position: absolute`) | `layout.position_type`(0 / 1) |
| `width` / `height` | 親と同じ | | ITEM_ROWS の sizing 4 欄 |
| `cell` | `:1113` | 数字 4 つ(start c / r、span c / r) | `layout.column_start`、`layout.row_start`、`layout.column_span`、`layout.row_span` |

### Advanced の奥(欄はそのまま `_control`)

上に無い `layout.*` は全部 `_AdvancedFold`(`id = layout:<layer>`)の中に、今までの cell のまま: Flex の 2 欄(direction / wrap)、depth_alignment、Box(background / border_radius / overflow / clip ×5 / shadow ×4)、Space(margin / hardness / heaviness / flex_shrink)、Field ×5、Offset Path ×3、Shape Outside ×2、Anchor ×3 + position_area、Constraints ×2、align_self、snap ×2 / object_fit、loop ×2 / transition_delay、stagger ×3、rotation / tilt ×2、exclusions、Grid の `layout.column.n` / `row.n`。棚の名前は利用者の裁定待ち。

### 消した物(前の版から)

Display の glyph 3 択、Direction `→ ↓ ⊙` + reverse、Wrap `⇋`、Depth Alignment の行、pad 横の文字の choice 2 つ、Space / Stretch の switch、子の Align self の glyph 4 つ、`_named` の言葉(Layout の節では)。UI から打てなくなった値: `justify_content` 3〜5(Space *)、`align_items` 0(Stretch)、`flex_wrap`、`align_self`(Advanced では読める・打てる)。

### 写せなかった物

- Padding の 4 辺(欄は Vec2)、Gap の行 / 列(欄は 1 値)。
- Stage の掴み(桃色の帯・格子の pill・升目 drag)は未着手。panel だけ。

### 試験

`inspector_layout_section_test.dart` 8 本(`:245` 矩形に節なし / `:255` None は grid 行だけ・switch で display 2 / `:279` Grid は 6 行・言葉 12 語なし・Advanced に Stagger・Overflow・Margin・Column 1 / `:340` Flex 書類は Advanced で読める / `:354` pad の snap → justify + align → commit / `:380` sizing の glyph と Fixed だけ数字 / `:414` 子は子の行だけ / `:436` Grid の子は cell)、`stage_spatial_gizmo_test.dart:170` 1 本。
