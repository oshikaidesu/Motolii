# Relational Group — 最小の手で最大の関係性を作る

2026-09-22 草案。**概念境界を議論するための RFC であり、公開 API・保存 schema・solver・UI を確定しない。実装 PR ではない。**

## 0. 一文

> **個々のオブジェクトの座標を保存するのではなく、Group に少数の関係を宣言し、その一手を多数の子へ伝播させる。**

Motolii の Group を単なる階層整理や一括 Transform の箱ではなく、**子同士・基準オブジェクト・空間の関係を所有する宣言的な container** として育てる。

これは Precomp の置換ではない。Precomp が複雑さを境界の内側へ **encapsulate** する道具なら、Relational Group は複雑さを閉じずに **relationships として記述する**道具である。

## 1. 既存の「箱と流し込み」の次

既存の [箱と流し込みの法](reviews/2026-09-14-layout-law.md) は、すでに重要な一歩を取っている。

- すべての層が箱を持つ。
- Group が `Display = None / Flex / Grid` を持つ。
- Flex / Grid の結果座標は書類へ保存せず、その時刻に resolve する。
- 親が集団の法則を持ち、子は `Align Self` 等で局所的に参加する。
- 2D / 2.5D / 3D と layout を直交させる。

本 RFC はこの所有モデルを捨てない。むしろ一般化する。

現在の Flex / Grid は、

> **存在する子を、Group の平面上でどう並べるか**

を宣言する。

Relational Group が扱いたい次の範囲は、

> **何を基準に、どの領域へ、どのように増やし・配り・向け・避け・変奏させるか**

である。

「3D CSS」を別系統で追加する提案ではない。CSS から借りるのは構文ではなく、**親へ少数の法則を書けば任意個の子へ結果が伝播し、computed geometry は保存しない**という所有モデルである。

## 2. 動機

モーショングラフィックスでは、見た目上は数十〜数百の要素があっても、作者が本当に決めたいことは少ない場合がある。

例:

- 中央のロゴを避けながら、花・星・SVG を周囲へ散らす。
- 基準の 3D object の周囲へ文字・線・板を配置する。
- path に沿って要素を等間隔に並べ、接線へ向ける。
- mesh の surface に object を散布し、normal へ向ける。
- volume を instance で満たす。
- 基準 object の size が変わったら、周囲の decoration も関係を保ったまま reflow する。

これらを個々の Transform として authoring すると、**見えている個数だけ編集点が増える**。しかし画の意図は「40 個の座標」ではなく「中央を避けて周囲へ散らす」である。

Relational Group は、authoring data を結果座標ではなくその意図へ寄せる。

```text
Authoring declarations
        ↓
Relationships / constraints
        ↓
Resolve at time t
        ↓
Computed placement / orientation / instances
        ↓
Render
```

## 3. Group が relation の owner

UI 上の最小単位は新しい巨大な subsystem ではなく、既存の **Group Layer** を第一候補とする。

```text
Group "Decorations"
│
├─ Reference: Logo
├─ Repeat: 40
├─ Region: Outside
├─ Distribution: Random
├─ Gap: 30
├─ Avoid: Logo
├─ Variation
│    ├─ Scale: 0.5 .. 1.5
│    └─ Rotation: 0 .. 360deg
│
├─ Flower.svg
├─ Star.svg
└─ Burst.svg
```

この例で保存したい正本は 40 個の最終 Transform ではない。Logo と decoration 群の**関係**である。

Logo の大きさ・位置・時刻が変われば、resolve 結果も変わる。結果を Document へ書き戻さない原則は既存 Flex / Grid と同じ。

### 親と子の責務

CSS/Flex の先例と同様に、責務を二段に分ける。

- **Group**: 集団に対する法則を所有する。
- **Child / source**: その法則への局所的な参加方法を持てる。

例として、Group が `Distribution = Around` を持ち、特定の child だけ `Orient Self = Tangent` や weight を override できる形を想定できる。

具体的な field 名は本 RFC では凍結しない。

## 4. 語彙の草案

必要な概念は、少なくとも次の軸へ分解できそうである。

```text
Group
├─ Sources       何を対象 / source にするか
├─ Generate      いくつ存在させるか
├─ Reference     何を基準にするか
├─ Region        どの空間を候補にするか
├─ Distribution  候補空間へどう配るか
├─ Constraint    何を守る / 避けるか
├─ Orientation   どちらを向くか
└─ Variation     規則性をどう崩すか
```

候補語彙の例:

- Region: Plane / Outside / Path / Surface / Volume
- Distribution: Linear / Grid / Around / Even / Random / Fill
- Constraint: Avoid / Contain / Gap
- Orientation: Fixed / Look At / Tangent / Normal / Billboard
- Variation: Scale / Rotation / Position / source choice

これは API 一覧ではない。**複数の見た目を同じ少数の関係軸で説明できるか**を見るための分類である。

## 5. Repeat を特殊機能として孤立させない

Repeater は重要だが、単独では「同じものを増やす」までしか言えない。

典型的な画では、

```text
Sources
   ↓
Repeat
   ↓
Distribute
   ↓
Constraint
   ↓
Orient
   ↓
Variation
```

が一続きの意図になっている。

そのため本 RFC では Repeater を独立した最終 UI / 保存モデルとして先に固定しない。既存の [ジェネラティブ表現とユーザー拡張の境界](../generative-user-boundary.md) が定める Materialize / Pure Live / Host 所有状態の原則を維持しつつ、**有限個の Document child と、描画時だけ存在する instance 集合を同一視しない**。

「Repeat が Group の宣言として見える」ことと、「40 個の Layer を Document に materialize する」ことは別の判断である。

## 6. 2D / 3D を別の思想にしない

本提案の出発点は 3D 表現だったが、専用の「3D CSS」を作ることが目的ではない。

同じ relation は空間によって解釈できる。

```text
2D decoration:
  Region = Outside(Logo)
  Distribution = Random

Path:
  Region = Path(Curve)
  Distribution = Even
  Orientation = Tangent

3D surface:
  Region = Surface(Mesh)
  Distribution = Even
  Orientation = Normal

3D volume:
  Region = Volume(Bounds)
  Distribution = Fill
```

既存 layout law の「2D / 2.5D / 3D とは直交」という原則を維持する。Flex / Grid を無理に 3 軸化することから始めず、**relation が必要とする geometry / bounds / path / surface を型付き入力として受ける**方向を検討する。

## 7. Precomp との境界

Relational Group は Precomp の廃止案ではない。

```text
Precomp
  complexity → encapsulate behind a composition boundary

Relational Group
  complexity → describe as relationships while children remain live
```

Precomp / nested composition が担う可能性のあるもの:

- 独立した時間軸
- 再利用可能な composition 境界
- 明示的な evaluation / render 境界

Relational Group が担うもの:

- 同じ composition 内での関係
- 子の live な編集可能性
- 一手を多数の子へ伝播させる spatial / generative rule

「整理のために中へ隠す」ことを、関係を表現できないことの代用品にしない。

## 8. Node architecture との関係

Group Layer を UI 上の簡単な入口にしても、内部実装を monolithic な特例にする必要はない。

概念的には、

```text
Sources
  → Generate / Repeat
  → Region / Distribution
  → Constraints
  → Orientation
  → Variation
  → Computed placement
```

のような評価へ分解できる。

ただし本 RFC は「必ずこれらを公開 node として実装する」「Group を node graph の sugar にする」とは決めない。現在進行中の FrameGraph / node ownership と二重管理を作らないことを条件に、実装時に最小の owner を選ぶ。

## 9. 既存方針との整合条件

実装へ進む場合、少なくとも以下を壊さない。

1. **computed placement を Document の正本へ書かない。**
2. **任意時刻へ deterministic に resolve できる。**
3. Random / Variation は明示 seed + stable identity を使う。
4. preview / export が同じ意味を通る。
5. Undo / Redo は結果座標ではなく authoring declaration を編集する。
6. Group / child の ownership を一本化し、UI と node 側に二重の正本を作らない。
7. 既存 `Display=None/Flex/Grid` の意味を本 RFC だけで変更しない。
8. 3D DCC 全体を内製する理由にしない。Motolii は素材そのものより**素材間の関係と時間変化**を扱う。

## 10. Non-goals / 未決定

この草案では以下を決めない。

- 公開 API / Rust type / serialization schema
- `Around`, `Surface`, `Volume` 等の正式名称
- solver の選択
- collision / packing の品質と計算量
- 3D bounds の厳密な定義
- instance の選択・materialize UI
- seed / stable identity の具体 schema
- Inspector の最終 UI
- node graph への露出方法
- evaluation order と循環参照の完全な規則
- v1 / v2 の搭載時期

これらは具体的な representative scene を選び、既存 owner と性能境界を確認してから別の決定として凍結する。

## 11. 次に試す representative scenes

設計を API から始めず、少数の画で圧縮率を測る。

1. **Logo + SVG decoration** — 中央の Logo を避け、複数 SVG source を repeat / variation 付きで外周へ散らす。Logo を resize / animate すると reflow する。
2. **3D hero + satellites** — 1 個の基準 object の周囲へ text / mesh / line を配置し、距離・向きを関係として保つ。
3. **Path typography / objects** — curve に沿って等間隔配置し tangent へ向ける。
4. **Surface / volume distribution** — mesh surface または volume に object を配置する。
5. **既存 Flex / Grid** — 同じ Group ownership の最小ケースとして regression を保つ。

評価したいのは「何機能できたか」ではなく、**作者が直接指定する値の数に対して、どれだけ多くの関係が安定して生まれるか**である。

## 12. 設計原則

> **最小の手で、最大の関係性を作る。**

座標を大量に authoring するより、少数の関係を authoring する。

Group は「フォルダ」から一歩進み、子を隠さず、子同士の関係を所有できる container になる。

Flex / Grid はその最初の実例であり、Relational Group はその考えを decoration、repeater、path、surface、volume、2D / 3D composition へ一般化できるかを検討するための草案である。
