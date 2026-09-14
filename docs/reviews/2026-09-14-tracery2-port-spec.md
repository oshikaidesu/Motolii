# Tracery 2 を移す — 仕様の写し

作成日: 2026-09-14

状態: **決定**(2026-09-14 利用者「推しで」: 移し方 A = 効果 1 枚、掛ける先は調整層と同じく下の合成)。**1 の骨組み済み**(Keying・Box・Marker・Show Mask)。

出所: 利用者 2026-09-14「これの仕様を移すぞ」「2 の方の取説も読んで、自由度が上がってるから」。
取説 PDF は公開ページから辿れず、体験版はサイトのボット対策(403)で取れなかった。以下は
[Tracery 2 の製品ページ](https://aescripts.com/tracery/)(作者 Dragoy)の本文と、同ページの Modules 節の画面 6 枚
(Effect Controls を節ごとに開いた状態)から読んだ事実。値は画面に写っていた一例。

## 1. 置き方

画面では **Adjustment Layer に Tracery 2 を掛けている**(Effect Controls: Adjustment Layer 1)。つまり下の合成を読んで、その上に FUI を描く。
Motolii の効果の法(2026-09-13)でいう「出力 = 画面座標(AE の調整層)」の札に当たる。

## 2. 欄(画面から)

**Presets** — 40 個同梱、JSON で保存・読み込み。

**Keying**
- Detection Method: Motion Detection / Key Color
- Threshold(28.0)、Blur Strength(0.0)、Min Region Pixels(16)
- Show Mask(チェック)、Alpha Layer Mode(チェック)

**Box**
- Box Enabled、Box Shape(Rectangle / Square / Ellipse / **Circle**)、Custom Size(チェック)、Custom Radius(32)
- Stroke: Box Stroke Enabled、Color Box Stroke、Box Stroke Opacity(1.0)、Box Stroke(太さ 7.1)、**Box Gap Enabled、Box Gap Size(0.3)**(円の枠が途切れた弧になる)
- Fill: Box Fill Enabled、Box Fill Opacity、Box Fill Mode(Solid / Diagonal Hatch / Invert / Random Patch / X-ray LUT / BW Duotone / Glitch)、Box Fill Color
- 本文: segmented corners(角だけの枠)

**Marker**
- Marker Enabled、Marker Type(Dot / Plus / Cross / **Polygon**)、Marker Color、Marker Opacity(0.91)
- Marker Size(46)、Marker Line Thickness(8.4)、Marker Rotation(101°)、Polygon Sides(3、3〜12)、Polygon Fill
- 本文: Plus と Cross は箱の端から端まで伸ばせる

**Grid**
- Grid Enabled、View Mode(Edge / **Cartesian**)、Grid Columns(36)、Grid Rows(19)、Grid Color、Grid Opacity(0.35)、Line Thickness(1.0)
- Scale: Scale Ticks、Tick Size(6)、Tick Labels、Value Mode(Pixels / **Percent**)、Value Size(1.00)、Scale Padding(24)
- Projections: Projection Dash、Dash Length(8)、Gap Length(6)、Projection Opacity(0.85)、Projection Color、Projection Values(印から軸へ点線を落とし、値を出す)

**Connection Lines**
- Connection Enabled、Connection Type(**Spline** / PCB Traces / Smooth Bend / Step Bend、1.1 の Star / Full / MST)
- Connection Order(**Distance From Center** …)、Connection Color、Connection Opacity(1.00)、Connection Thickness(3.6)
- Spline Tension(-1.00)、Spline Continuity(0.00)、Spline Bias(Kochanek–Bartels の 3 値)、Spline Handles
- Random Chance(0)、K Neighbors(50)
- 本文: Bend Amount、Bend Balance、Corner Position
- Dash: Dash Enabled、Dash Length(10)、Gap Length(5)
- Arrow: Arrow Enabled、Arrow Color、Arrow Opacity(1.00)、Arrow Size(23)、Arrow Angle(0.4)、Arrow Position(0.7)、Arrow Filled

**Labels**
- Label Enabled、Display Mode(Coordinates / Dimensions / Area / Node / Hex / Percent / **Matrix**)、Label Color、Label Opacity(0.85)、Font Size(1.5)
- Use System Fonts、Font Family
- Background: Background、Color、Opacity(0.75)、Padding(6.0)、Corner Radius(50.0)、Stroke、Stroke Color、Stroke Thickness(1.0)
- Position: Anchor Point(Center Marker …)、Vertical Alignment(Top)、Horizontal Alignment(Center)、Offset X(0)、Offset Y(34)

## 3. Motolii への写し(案)

| Tracery 2 | Motolii |
|---|---|
| Keying | 解析の橋 + `media/blob.rs`(Motion = 前のコマとの差、Key Color = 色の距離)。Blur Strength は二値化の前のぼかし、Min Region Pixels = Min Size。Show Mask は二値の絵を出す。Detail・Separation・Max Size は Motolii の足し |
| Box / Marker / Grid / Connection Lines | fork の re_renderer の線と面(描画の境目は深く 1 本) |
| Box Fill Mode | 箱の中だけに掛かる shader(箱の並びを data texture で渡す) |
| Labels | 文字の系統(Parley / fontdb、OS の書体) |
| Presets | JSON(書類の外、利用者の庫) |

既にある Blob Track(配置として出す)は、「箱の場所に任意の層を置く」道として残す。

## 4. 順番(案)

1. 骨組み: 効果 1 枚 + Keying + Box(形 4 種・線・途切れ・塗り Solid)+ Marker
2. Connection Lines(順・種類・Spline の 3 値・K Neighbors・Random Chance・Dash・Arrow)
3. Labels(Display Mode 7 種・背景・位置)
4. Box Fill Mode 6 種 + Grid(Scale・Projections)
5. Presets(JSON)
