# PathOp語彙比較: AE/Lottie × Cavalry(2026-07-12)

ステータス: **調査メモ(未採用)**。PathOp意味論の【決定】には使わない。確定前の比較材料。

規律: [reviews/README.md](README.md)。本ファイルの対応表をそのままスキーマ根拠にしない。採用する行は仕様改訂PRで【決定】を明示する。

## なぜCavalryを見るか

ユーザー観察: パス操作の発達度はAEよりCavalry側が厚い可能性がある。AE/Lottie閉集合だけを前例に意味を焼くと、Cavalryが既に解いている角(両面オフセット・波モード・Stroke側Trim・点単位ベベル等)を後から足すときに恒久破壊が起きやすい。

ただし北極星はCavalryではない([concept](../concept.md): ベクター寄り・ブーリアン等はスコープ外、F-8は原子粒の逆張り)。**語彙の豊かさと比較軸**として読む。採用は「意図単位の閉集合」に畳めるものだけ。

## アーキテクチャの差(焼かない前提)

| 軸 | AE / Lottie | Cavalry | Motoliiへの含意 |
|---|---|---|---|
| 置き場 | シェイプレイヤー内の固定パス演算子スタック | Shapeの`Deformers`リストへBehaviourを積む([Common Attributes](https://cavalry.studio/docs/nodes/shapes/common-attributes/)) | v1はAE/Lottie型の`Vec<PathOp>`閉集合を維持(F-13)。Cavalryの開放グラフを契約に出さない |
| Trimの所属 | パス演算子(幾何を切る) | 主に[Stroke Utility](https://cavalry.studio/docs/nodes/utilities/stroke/)のTrim(+taper/dash)。輪郭始点は[Travel Deformer](https://cavalry.studio/docs/nodes/behaviours/travel-deformer/)が別口 | 「Trim=パス幾何」か「Trim=ストローク描画」かを意味表で明示する必要。未決 |
| 複製 | Repeater(変換の累乗) | Duplicator + Stagger(インデックス駆動。F-7既存分析) | Repeater席とDuplicator評価口は別問題として扱う |
| 拡張 | 閉集合(+スクリプト) | Behaviourを任意合成。Mesh Solver等で反復も可 | F-8どおりユーザー露出は意図単位。原子Deformerのスープは採らない |

## 対応表(Motolii候補 ↔ 先例)

出典はすべて公式docs(一次)。「相当」は機能族の近似であり、パラメータ同型を主張しない。

| Motolii `op`(候補) | AE / Lottie | Cavalry(一次) | Cavalryが厚い点 | v1閉集合への暫定読み |
|---|---|---|---|---|
| `pucker_bloat` | Pucker & Bloat / `pb` | **専用ノードなし**。[Noise](https://cavalry.studio/docs/nodes/behaviours/noise/)(`Use Normals`)や[Pinch](https://cavalry.studio/docs/nodes/behaviours/pinch/)は別意味 | — | AE/Lottie前例を維持候補。Cavalry不足は棄却理由にならない |
| `zig_zag` | Zig Zag / `zz` | [Wave](https://cavalry.studio/docs/nodes/behaviours/wave/): Sine/Square/Sawtooth/**Triangle** | 波形モード・Adaptive Wave Counts・Travel・Sample Points・Output Béziers | Triangle≈ZigZag。他モードは将来variant候補。意味確定時に「zz相当=どのWave Modeか」を書く |
| `offset` | Offset Paths / `op` | [Path Offset](https://cavalry.studio/docs/nodes/behaviours/path-offset/) | Single/Double Sided、開パスCap(Flat/Round/Projecting/Joined)、Rounded。**閉+開Contour混在は非対応** | 両面・Cap・混在拒否は意味表の未決項目。AE片面オフセットだけ焼くと後で足りない可能性 |
| `round_corners` | Round Corners / `rd` | [Bevel](https://cavalry.studio/docs/nodes/behaviours/bevel/): Fillet/Chamfer | Per Sub-Mesh / Per Point半径、Min/Max Angle | Fillet≈角丸。Chamfer・点別半径は追加席候補 |
| `trim` | Trim Paths / `tm` | Stroke.Trim(Start/End/Travel) + Travel Deformer + [Segment Path](https://cavalry.studio/docs/nodes/shapes/segment-path/) | taper/dash/Alignと同居。Travelは輪郭始点の再配置。Segmentは切断→sub-mesh | 「幾何Trim」と「描画Trim」を分けないとStroke機能をPathOpに押し込む事故が起きる |
| `twist` | Twist / `tw` | **専用ノード未確認**。[Lattice](https://cavalry.studio/docs/nodes/behaviours/lattice/) / [Four Point Warp](https://cavalry.studio/docs/nodes/behaviours/four-point-warp/)が空間ワープ族 | 格子・四隅ベジェ | TwistはAE前例維持候補。Lattice族はPathOp閉集合外(空間デフォーマ) |
| `wiggle` | Wiggle Paths | Noise as Deformer(同上) | Noise Type多数・Seed/Stagger/Looping/Index Context・値生成と変形の兼用 | Wiggleは意図単位として残し、NoiseスープはParamDriver/将来に分離(既存方針と整合) |
| `repeater` | Repeater / `rp` | Duplicator(+Stagger) | 分布・インデックス・配列駆動が本体 | F-7。`transform`席は別途。Cavalry Duplicator全体をPathOpに畳まない |

## CavalryにあってAE PathOp閉集合に無いもの(v1で焼かない候補)

意図的にPathOpへ入れない／別口検討。ブーリアン等はconceptどおりスコープ外。

| Cavalry | 公式 | なぜ今焼かないか |
|---|---|---|
| Pathfinder | [docs](https://cavalry.studio/docs/nodes/behaviours/pathfinder/) | パスに沿う配置/変形。レイヤー変形・制約に近い |
| Pinch | [docs](https://cavalry.studio/docs/nodes/behaviours/pinch/) | Falloff+Null駆動。空間デフォーマ |
| Lattice / Controller | [Lattice](https://cavalry.studio/docs/nodes/behaviours/lattice/) | 制御点グリッド。永続形状が大きい |
| Four Point Warp | [docs](https://cavalry.studio/docs/nodes/behaviours/four-point-warp/) | 四隅ワープ |
| Path Relax / Path Average | [Path Relax](https://cavalry.studio/docs/nodes/behaviours/path-relax/) / 2.6 notes | 点の分離・平滑。反復パラメータ |
| Segment Path | [docs](https://cavalry.studio/docs/nodes/shapes/segment-path/) | 切断→sub-mesh。Trimとは別演算 |
| Stroke taper/dash/align | [Stroke](https://cavalry.studio/docs/nodes/utilities/stroke/) | 描画属性。パス→パス演算ではない |

## 意味確定前に潰すべき未決(本調査から)

確定PRで答える。未決のままD1i-2実装に入らない(GR-PV-1)。

1. **Offset**: 片面のみか、Cavalry式Double Sided/開パスCap/閉開混在拒否をv1に入れるか
2. **ZigZag vs Wave**: `zig_zag`をTriangle固定にするか、波形`mode`を最初から持つか(追加的拡張なら後でも可)
3. **RoundCorners vs Bevel**: Filletのみか、Chamfer/点別半径/角度フィルタを席予約するか
4. **Trimの所属**: パス幾何を切るか、Stroke描画範囲か。Travel(輪郭始点)をTrimに内包するか別opか
5. **Twist中心・Wiggleアルゴリズム**: AE寄せでよいか(Cavalry専用相当が薄いので前例はAE側が強い)
6. **閉集合の境界**: Lattice/Pinch/PathfinderはPathOpに入れないことを仕様一文で明示するか

## いまやらないこと

- 本メモの数値・モードをスキーマやゴールデンに焼く
- CavalryのBehaviourグラフをプラグイン契約へ露出する
- 「Cavalryの方が発達→閉集合を広げる」への短絡(F-8逆張りを崩す)

次工程: 上記未決をユーザー判断で潰したあと、M2「PathOp意味論表」を【決定】へ昇格し、その写しとしてD1i-2実装。
