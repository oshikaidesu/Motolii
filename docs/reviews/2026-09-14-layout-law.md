# 箱と流し込みの法 — Web の Flex / Grid を層の基礎に

2026-09-14 提案(裁定待ち)。きっかけは利用者「今足りないのはフレキシブルボックス系の概念」
「実際の GUI は web 世界の方が整っている。bento grid など普通に使えるようにしたい。これは基礎機能であり、
拡張としてしか足せないためみんな余計な手順を踏まされている」。

## 診断 — なぜ拡張でしか足せないか

並べる側は隣の**大きさ**を知らなければならない。AE の層は箱を持たない(アンカー・位置・スケールだけ)ので、
`sourceRectAtTime()` を毎フレーム式で読み、式でつなぐしかない。拡張(Pins & Boxes、Flex)はその式を代わりに張る道具。

Motolii も同じ所に居る:

- [store.rs](../../motolii/crates/motolii-doc/src/store.rs) `LayerSource::declared_size` は全種で `None`。
- [resolve.rs](../../motolii/crates/motolii-render/src/picture/resolve.rs) の Motion Blur の道のりは「形・文字は宣言の大きさを持たない(描くまで分からない)」ので原点のまわり 200 px 四方で代用している。

**足りないのは並べる機能ではなく、層が箱であること。** 箱があれば Flex も Grid も「並べ方の値を 1 つ選ぶ」だけになる。

## 先例の段

| 段 | 形 | 読み |
|---|---|---|
| AE + 拡張 | [Pins & Boxes](https://aescripts.com/pins-and-boxes/): 辺の Pin を親にし、Box が Pin を包む。Flex(aescripts) | 式の代行。層が箱でないことの回避 |
| Cavalry | [Layout Group](https://cavalry.studio/docs/nodes/shapes/layouts/layout-group/)(並べる)・[Bounding Box](https://cavalry.studio/docs/nodes/utilities/bounding-box/)(測って矩形へ繋ぐ)・[Align](https://cavalry.studio/docs/nodes/behaviours/align/)(伸びる辺) の 3 部品。旧 Layout Shape は非推奨、効果の形 Apply Layout は experimental | 手探り。形が箱でないので 3 つに割れた |
| Web | CSS の箱(背景・padding・角丸・`overflow`)+ [Flexbox](https://www.w3.org/TR/css-flexbox-1/) / [Grid](https://www.w3.org/TR/css-grid-2/)。デザイン道具は [Figma Auto Layout](https://help.figma.com/hc/en-us/articles/360040451373-Guide-to-auto-layout)(Hug / Fill / Fixed、Absolute)、[Penpot](https://help.penpot.app/user-guide/designing/flexible-layouts/)(欄を CSS の属性にそのまま写す) | 整っている。Cavalry の 3 部品が 1 つに畳まれる |

Web で畳まれる所:

- **テロップの帯** = 容器の背景 + padding + Hug。測って繋ぐ部品が要らない。
- **伸びる向き** = 親の justify / align。Align の部品が要らない。
- **大きくする物は 2 つ**。`zoom` は並べる途中で効き隣を押す、`transform: scale` は並べた後で見た目だけ([CSS Viewport](https://drafts.csswg.org/css-viewport/#zoom-property)、2024 に主要ブラウザが揃った)。列から外すのは `position: absolute`。
- **効果の広がりは並びに効かない**(CSS `filter`・`box-shadow`)。
- **文字の大きさは行の箱**(line box)。字形のインクではないので、文字を差し替えても帯が上下にぶれない。

実装は [taffy](https://github.com/DioxusLabs/taffy)(CSS Block / Flexbox / Grid。Servo・Bevy・Zed・Blitz が使う)。並べる計算は書かない。

## 意図 → 札

| 押す人 | 意図 | 札 | 先例 |
|---|---|---|---|
| テロップ・下三分の一(放送、配信) | 名前が長くなると帯が伸びる、アイコンが押される | Flex(Row)+ 箱 | Figma Auto Layout、CSS `inline-flex` |
| 商品紹介・SNS 告知(Figma / Web から来た人) | bento grid、セルに動画を Cover で詰める | Grid + Object Fit | CSS Grid + `object-fit`、Penpot Grid |
| 箇条書き・UI モック・歌詞の行 | 1 行消すと下が詰まる | Flex(Column) | CSS Flexbox、Flutter Column |
| AE から来た人の自由配置 | 手で置く | None(今の Group と同じ) | AE の親子 |

## 法(案)

1. **すべての層は箱を持つ** — その時刻の大きさを doc の resolve で 1 本に解く。文字 = 組版の行の箱(cosmic-text の行高)、形 = Size、画像・動画 = 元の寸法、Flex / Grid の Group = 並べた結果、Null / Camera = 箱無し(並びに参加しない)。効果の広がりは含めない。
2. **Group の札 Display = None / Flex / Grid**。既定は None で今の Group と同じ。札は自動で変えない。
3. **欄の名前は CSS の語を窓の書き方で**(Penpot と同じ)。CSS に一語が無い大きさの決め方だけ Figma の Hug / Fill / Fixed。名前は [names.rs](../../motolii/crates/motolii-doc/src/store/names.rs) の表に足し、窓とスクリプトが同じ名前を使う([スクリプトの口](2026-09-14-script-mouth.md))。持ち込まない CSS: カスケード・詳細度・継承・セレクタ(スクリプトに出たら赤にして行き先を言う)。計算は taffy。
4. **層の Scale は `zoom`** — 並べる時の箱は「測った大きさ × Scale」で、隣を押す。文字の Character size が素の大きさ、Scale が動かす大きさで、並びはその積を見る(2026-09-13「文字は Scale で動かす」と同期)。文字だけの特例にしない。**見た目だけ大きくするのは Transform 効果**(`transform: scale`、法 1 の「効果は並びに効かない」で受ける。AE 本体の Transform 効果と同じ)。**Rotation は並びに効かない**。**Position は並べた位置からのずれ**(`position: relative`)。Absolute で列から外れる。
5. **Fill の結果は書類に書かない**。resolve がその時刻に解く(解析の橋と同じ扱い)。子の Size は Fixed の時だけ効き、Hug / Fill では保存されたまま使われない(CSS の `width: auto` と `width: 100px`)。
6. **Flex / Grid の Group は箱** — Padding・Gap・Background・Border・Border Radius・Overflow Clip。背景は re_renderer の既存の形で描く(Motolii 側に描き分けを作らない)。
7. **中身の合わせ方 Object Fit = Fill / Contain / Cover / None**(CSS `object-fit`)。セルに動画や画像を詰める時の必須。
8. **並び順は timeline の上から**(Cavalry の Layer Order、DOM の順)。その時刻に居ない層は並びに参加しない(`display: none`)。
9. **時間は毎フレーム解く**。箱が連続して変われば並びも連続する。層の出入り・並び替えでの跳びを滑らかにする欄(FLIP、[Motion の layout](https://motion.dev/docs/react-layout-animations))は延期。
10. **2D / 2.5D / 3D とは直交**。並べるのは Group の平面の上の x, y だけで、z と札には触らない。

## 欄

窓の名前(= スクリプトの名前)と、写した元。

| 持ち主 | 窓の名前 | 元 |
|---|---|---|
| Group | Display(None / Flex / Grid) | `display` |
| Group(Flex) | Flex Direction、Flex Wrap、Justify Content、Align Items | 同名の CSS |
| Group(Grid) | Grid Columns、Grid Rows(数と、**線ごとの大きさ Column 1…n / Row 1…n、鍵が打てる**。単位は fr / px / Auto) | `grid-template-columns / rows`(CSS でも補間できる) |
| Group(箱) | Width、Height(Hug / Fixed)、Padding、Gap、Background、Border Radius、Border、Overflow(Visible / Clip) | Figma Hug / Fixed、同名の CSS |
| 子 | Width、Height(Hug / Fill / Fixed)、Min Width、Max Width、Min Height、Max Height、Align Self、Position Type(Relative / Absolute) | Figma、同名の CSS |
| 子(Grid) | Column Span、Row Span | `grid-column / grid-row: span n` |
| 子(画像・動画・Group) | Object Fit(Fill / Contain / Cover / None) | `object-fit` |
| 層(既存) | Scale = `zoom`、Position = `position: relative` のずれ、Rotation = 並びに効かない | 法 4 |
| 効果(新) | Transform(Scale・Position・Rotation、見た目だけ) | `transform`、AE の Transform 効果 |

## 実例での分解

### Bento grid(商品紹介、1920×1080)

```css
.bento { display: grid; grid-template-columns: repeat(4, 1fr); grid-template-rows: repeat(3, 1fr);
         gap: 16px; padding: 48px; }
.hero  { grid-column: span 2; grid-row: span 2; border-radius: 24px; overflow: hidden; }
.hero video { object-fit: cover; }
```

```
┌───────────────┬───────┬───────┐   Group "Bento"   Display Grid, Grid Columns 4, Grid Rows 3, Gap 16, Padding 48, Fixed 1920×1080
│               │ Stat  │ Logo  │   ├ Group "Hero"   Span 2×2, Border Radius 24, Overflow Clip
│   Hero        ├───────┴───────┤   │  └ 動画          Width/Height Fill, Object Fit Cover
│   (video)     │ Title         │   ├ Group "Stat"   Background, Border Radius 24, Padding 32 / Display Flex, Flex Direction Column, Justify Content End
│               │               │   │  └ 文字 "4.8×"  Hug
├───────┬───────┼───────┬───────┤   ├ Group "Logo"   ...
│ Photo │ Photo │ Quote │ CTA   │   ├ Group "Title"  Span 2×1 ...
└───────┴───────┴───────┴───────┘   └ ...
```

登場は既存のグループの効果で出る: Bento に Fade + Position を scope Each、ずれは番号比例(Each)と種(Random)。**追加の手順は無い。**

### 格子の squash & stretch(2026-09-14 利用者が示した実例)

Threads で伸びていた作品(cancelled.name「Squash and stretch」): 格子の**行と列の太さそのものが鍵で動き**、真ん中の行が細い線に潰れ、両側が膨らむ。セルの中の丸・四角・数字はセルいっぱい(Fill)なので一緒に潰れて伸び、玉が格子を斜めに跳ねて見える。

```
Group "Grid"  Display Grid, Grid Columns 9, Grid Rows 9, Gap 1, Background(線の色)
  Row 5 Size   ◇ 1fr → 0.05fr → 1fr(Ease)      ← 潰れる行
  Row 4 / 6    ◇ 1fr → 1.6fr → 1fr              ← 膨らむ行
  ├ 形 丸      Column 1, Row 1 → …(Grid 上の位置)  Width/Height Fill, Object Fit Contain(丸のまま縮む)
  └ 形 四角    Width/Height Fill, Object Fit Fill(潰れて伸びる)
```

今これは、AE で矩形を 1 枚ずつ手でキーを打つか、CSS / p5 を書くしかない。**線ごとの太さに鍵を打てるだけで、格子全体が呼吸する。** これが「CSS グリッド表現の民主化」の芯で、Grid の欄は数だけでは足りない。

### 下三分の一のテロップ

```
┌──────────────────────────┐   Group "Lower third"  Display Flex, Flex Direction Row, Align Items Center, Gap 16, Padding 16/24, Background, Border Radius 12, Hug
│ (icon)  Taro Yamada      │   ├ 形 "icon"  Fixed 48×48
└──────────────────────────┘   └ 文字 "name" Hug(行の箱)
```

文字にタイプライターを掛けると、行の箱が毎フレーム伸び、帯は Hug なので追いかける。AE では `sourceRectAtTime()` の式、Cavalry では Bounding Box の部品が要る所。

## 実装の順(案)

1. **箱** — doc に「層のその時刻の箱」を 1 本(文字は行の箱)。`declared_size` の置き換え。Motion Blur の 200 px 代用もこれを読む。
2. **流し込み** — Group の Display を resolve で taffy に渡し、子の placement に足す。
3. **箱としての Group** — 背景・Clip を既存の形と matte の道で描く。
4. **Object Fit** — 画像・動画の Cover / Contain。
5. **窓** — Inspector の Display card、Stage に箱と Gap の印。

審判の定規は taffy の test(Chrome の実測から生成された fixture)。自作 test は「欄 → taffy の Style」の写像と、resolve が置いた位置だけを確かめる。

## 実装(2026-09-14)

- 並べる計算: doc `store/layout.rs`(taffy 0.13)。箱は形の輪郭の canvas・文字の行の箱・並べない Group の子の合わせ。結果は view の寿命で時刻ごとに 1 回解き、書類に書かない
- 背景: 形の層と同じ道で描く。重ね順は子孫の一番奥の 1 つ下
- **Overflow Clip**: 子孫の mask に、Group の箱(角丸)を子の素材座標へ写した Intersect を足す
- **文字の Fill**: 横が Fill の文字は、taffy の measure(幅 → 高さ)で並べた幅に折り返す。Scale は zoom なので折り返し幅は「枠 ÷ Scale」
- 2D の上下: 透明相の描き順が camera からの距離で決まっていた(並べる法の前から)。fork re_renderer に `layer_sort_key` を足し、2D は積み順を距離より先に比べる
- 見本: `ui/native/src/editor/script/examples/` の bento.js・grid_squash.js・swiss_grid.js

## 裁定済み(2026-09-14)

- 文字の大きさと Scale は同期する: 層の Scale = `zoom`、見た目だけは Transform 効果(利用者「逆に文字の大きさとスケールを同期できないか」)。
- 欄の名前は CSS の語、大きさの決め方だけ Figma(利用者「推しで」)。

## 聞くこと

1. **Group の Width / Height の既定**。仮に Hug(Figma の既定)、実窓のスクショと動きを見てから決める(利用者 2026-09-14)。
2. ~~Grid Columns / Rows は数だけ、並びは Advanced~~ → 撤回。squash & stretch の実例で、**線ごとの太さに鍵を打つのが主役**と分かった。窓での見せ方(Inspector の行か、Stage の線を掴んで引くか)は実窓で見てから。
3. (実装時に確かめる)Fill の子に Scale を掛けた時。セルの大きさで決まり Scale は中身だけ、が CSS に近い見込み。taffy の挙動で確かめる。

判定語: 比較中(裁定で決定へ)。
