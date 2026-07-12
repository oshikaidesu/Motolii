# PathOp語彙比較: AE/Lottie × Cavalry(2026-07-12)

ステータス: **調査メモ(未採用)**。PathOp意味論の【決定】には使わない。確定前の比較材料。

規律: [reviews/README.md](README.md)。本ファイルの対応表をそのままスキーマ根拠にしない。採用する行は仕様改訂PRで【決定】を明示する。

## なぜCavalryを見るか

ユーザー観察: **CavalryはAEのパスエフェクトより豊富** — 機能インベントリでもこれは成立する(下節「豊富さスコアカード」)。AE/Lottie閉集合だけを前例に意味を焼くと、Cavalryが既に解いている角(両面オフセット・波モード・Stroke側Trim・点単位ベベル・Bend/Lattice等)を後から足すときに恒久破壊が起きやすい。

ただし北極星はCavalryではない([concept](../concept.md): ベクター寄り・ブーリアン等はスコープ外、F-8は原子粒の逆張り)。**語彙の豊かさは認める。ユーザー露出は意図単位に畳む。** 豊富さの承認 ≠ スープ全体の採用。

## 豊富さスコアカード(AE Path operators × Cavalry path/deform族)

数え方の約束: AEはシェイプレイヤー Add メニューの**パス演算子**(Fill/Strokeは除外。Wiggle Transformは変形揺れでパス頂点変形ではないがAE側に含めて記載)。CavalryはShape `Deformers`に積めるパス/メッシュ変形系+Stroke側Trim(描画だがTrim族)。公式docs一次。完全網羅ではないが方向は十分。

### A. AE閉集合(ほぼこれで終わり)

出典: [AE Scripting match names](https://ae-scripting.docsforadobe.dev/matchnames/layer/shapelayer/) / 教育一覧。

| AE Path operator | 備考 |
|---|---|
| Offset Paths | 片面オフセット+Line Join |
| Trim Paths | Start/End/Offset。Parallel相当 |
| Round Corners | 半径1つ |
| Zig Zag | Size / Ridges / Points |
| Pucker & Bloat | Amount |
| Twist | Angle / Center |
| Wiggle Paths | 頂点荒れ |
| Repeater | コピー+累乗Transform |
| Merge Paths | ブーリアン系(Motolii v1スコープ外寄り) |
| (Wiggle Transform) | パス頂点ではなくTransform揺れ |

→ **パス幾何に効く中核はおおよそ 8〜9 個の固定メニュー。** ここがAEの「パスエフェクト」の天井。

### B. Cavalry: 同じ意図でもパラメータが厚い(上位互換寄り)

| AE相当 | Cavalry | 厚い点(公式) |
|---|---|---|
| Offset Paths | [Path Offset](https://cavalry.studio/docs/nodes/behaviours/path-offset/) | Single/Double Sided、開Cap 4種、Rounded、混在制約の明示 |
| Zig Zag | [Wave](https://cavalry.studio/docs/nodes/behaviours/wave/) | Sine/Square/Sawtooth/**Triangle**、Adaptive、Travel、Sample、Output Béziers |
| Round Corners | [Bevel](https://cavalry.studio/docs/nodes/behaviours/bevel/) | Fillet/**Chamfer**、点別/Sub-Mesh半径、Min/Max Angle |
| Trim Paths | [Stroke.Trim](https://cavalry.studio/docs/nodes/utilities/stroke/) + [Travel Deformer](https://cavalry.studio/docs/nodes/behaviours/travel-deformer/) | taper/dash/Align同居。始点Travelが別口 |
| Wiggle Paths | [Noise](https://cavalry.studio/docs/nodes/behaviours/noise/) as Deformer | 複数Noise Type・Normals・Stagger・Loop・Index Context |
| Repeater | Duplicator + Stagger | 分布・インデックス駆動(別クラスの豊かさ) |

→ **重なる族だけ見てもCavalryの方が厚い。** 「気がする」はここでも裏付けられる。

### C. CavalryにあってAE Path operatorメニューに無いもの(カタログ拡大)

公式docsで確認済みの例(パス/メッシュ変形族。不完全リスト):

| Cavalry | 公式 | AE PathOpメニューに相当が無い |
|---|---|---|
| Bend | [Bend Deformer](https://cavalry.studio/docs/nodes/behaviours/bend-deformer/) | 円周曲げ(AEは別エフェクト/プラグイン寄せ) |
| Squash and Stretch | [docs](https://cavalry.studio/docs/nodes/behaviours/squash-and-stretch/) | 面積保存・Bulge付き |
| Lattice | [docs](https://cavalry.studio/docs/nodes/behaviours/lattice/) | 制御点グリッド |
| Four Point Warp | [docs](https://cavalry.studio/docs/nodes/behaviours/four-point-warp/) | 四隅ベジェワープ |
| Pinch | [docs](https://cavalry.studio/docs/nodes/behaviours/pinch/) | Falloff+Null |
| Pathfinder | [docs](https://cavalry.studio/docs/nodes/behaviours/pathfinder/) | パスに沿う変形/配置 |
| Path Relax / Path Average | [Path Relax](https://cavalry.studio/docs/nodes/behaviours/path-relax/) / 2.6 notes | 点分離・平滑 |
| Chop Path | [docs](https://cavalry.studio/docs/nodes/behaviours/chop-path/) | スライス切断 |
| Segment Path | [docs](https://cavalry.studio/docs/nodes/shapes/segment-path/) | 切断→sub-mesh |
| Sub-Mesh | [docs](https://cavalry.studio/docs/nodes/behaviours/sub-mesh/) | 階層レベル指定変形 |
| Auto-Crop | [docs](https://cavalry.studio/docs/nodes/behaviours/auto-crop/) | bboxクロップ |
| Travel Deformer | [docs](https://cavalry.studio/docs/nodes/behaviours/travel-deformer/) | 輪郭始点(AEはfirst vertex手作業) |

→ **種数でもCavalryが明らかに勝つ。** AEの固定8〜9に対し、Cavalryはパス/メッシュ変形だけで上記+Wave/Offset/Bevel/Noise…が並ぶ開放カタログ。

### D. AE側が名前付きで勝つ点(公平のため)

| 演算 | 注 |
|---|---|
| Pucker & Bloat | Cavalryに**専用**無し(Noise/Pinchで近似は別意味) |
| Twist | Cavalryに**専用**未確認(空間ワープ族で代替) |
| Merge Paths | AEはシェイプレイヤー内ブーリアン。Cavalryも別系統あるがMotolii v1はconceptどおりスコープ外寄り |

### E. 判定(豊富さのみ。採用方針ではない)

| 軸 | 勝者 |
|---|---|
| カタログ種数(パス/メッシュ変形) | **Cavalry** |
| 重なり族のパラメータ厚み | **Cavalry**(Offset/Wave/Bevel/Trim周辺/Noise) |
| 「意図の名前がメニューに並ぶ」発見性 | **AE** |
| 専用Pucker/Twistの明示 | **AE** |

**結論(豊富さ)**: ユーザー観察どおり、**Cavalryの方がパスエフェクトとして豊富**。AEは少ないが意図単位で揃えた閉集合。Cavalryは厚く・広く・組み立て前提。

**Motoliiへの含意(まだ【決定】しない)**:
1. AE閉集合を「語彙の天井」にしない — 豊富さの正本はCavalry比較込み
2. ただしCavalryカタログをPathOpに全部焼かない — F-8。v1は意図単位に畳み、厚い角はパラメータ/将来variantで吸収
3. 「豊富→全部入れる」は発見可能性死。正しい読みは「豊富→意味を薄く焼かない」

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

## 実ユーザー声とバイアス補正(2026-07-12追記)

**【決定】ではない。** 「どちらが良いか」を人気投票で決めない。声の出所にエコーチェンバーが乗る。

### バイアス地図

| 側 | エコーの形 | 読み方 |
|---|---|---|
| **AE** | コミュニティ巨大。Trim Pathsは「通過儀礼」として称賛記事が多い。フォーラムは**回避策の共有**が多く、モデル欠陥が「職人技」に正規化されやすい | 「AEパス演算が愛されている」≠「意味論がきれい」。不満スレの方が設計入力になる |
| **Cavalry** | Envato / School of Motion / ベンダー隣接の「switching from AE」記事が厚い。生き残った早期採用者の声が目立つ | 「手続きが速い」の賛辞は**Duplicator/データ/リアルタイム**に寄り、個別Path Offsetの優劣投票ではない |
| **両側共通** | PathOp単体のA/B比較スレはほぼ無い。比較は「タイムライン vs システム」「補完関係」で語られる | 機能表の厚み比較(上節)と、声の比較は別軸 |

Reddit一次スレは本環境から安定取得できず、以下は**Adobe Community / Creative COW / 教育メディア / 実務レビュー**中心(再確認可能URL付き)。

### AE側の声(パス演算まわり — 不満が設計入力)

| 声 | 出典 | 抽出 |
|---|---|---|
| Offset Pathsの上にTrimがあると first vertex が効かない / 順を入れ替えると二重線 | [Adobe Community](https://community.adobe.com/questions-529/setting-first-vertex-for-trim-paths-with-offset-paths-on-top-in-order-59037) | **スタック相互作用**が慢性痛。回避策=TrimのOffsetパラメータ |
| Offset Pathsを足すとTrim方向が逆転し、パス方向反転が効かない(「2024でも直ってないquirk」) | [Creative COW 2024](https://creativecow.net/forums/thread/offset-paths-changes-direction-of-trim-path/) | ユーザー自身がlegacy quirkと認識 |
| 内側ストロークが無くOffset Pathsで代用。開パスのcopies接続・元ストローク保持・極端copiesの挙動に不満。**「Cavalryも触ったが、このクローン方式ではCinema/Cavalry級の複製には届かない」** | [Adobe Community(Betaフィードバック)](https://community.adobe.com/questions-534/offset-paths-314278) | AEユーザーがCavalryを引き合いに**複製の天井**を指摘 |
| first vertex / 開パスで形が壊れる。TrimはStart/Endの組みで方向制御せよ | [Adobe Community](https://community.adobe.com/questions-529/set-first-vertex-issue-on-open-path-46038) / [COW](https://creativecow.net/forums/thread/cant-select-shape-layer-to-set-first-vortex/) | Travel相当の明示口が無いことの症状 |
| dash+Trimの「marching ants」、RepeaterのTransform席の取り違え | [COW](https://creativecow.net/forums/thread/dashed-trim-paths-dont-want-line-to-dance/) / [COW Repeater](https://creativecow.net/forums/thread/problem-with-repeater/) | 意図単位UIでも**席の意味が伝わりにくい** |

称賛側(エコー): 教育記事はTrim/Repeater/Wiggleを「モーションデザインの親友」と書く([OlafMotion](https://olafmotion.com/motion-knowledge/shape-layers-vs-masks-in-after-effects/)等)。これは**意図単位の閉集合が覚えやすい**証拠であり、AEスタック相互作用が正しいことの証拠ではない。

### Cavalry側の声(パス単体より「手続き全体」)

| 声 | 出典 | 抽出 |
|---|---|---|
| AEの50個キーフレーム地獄 → Duplicator+Stagger。**補完であり置換ではない** | [School of Motion](https://www.schoolofmotion.com/blog/cavalry-houdini-of-2d-after-effects)(Greg等の実務談) | 勝ち筋は複製・関係性。パス演算スタックの優勝ではない |
| ノードはHoudiniより浅いがAEの直接操作より急。2–4週で生産的。エフェクト生態系・仕上げはAE | [SuperRenders 2026実務レビュー](https://superrendersfarm.com/article/cavalry-motion-design-review-2026) | スタジオは**両方**。Cavalry単体優勝なし |
| 「200+ building blocksでoverwhelm。まずStagger/Noise/Oscillator」 | [LinkedIn: Elena Kudriavtseva](https://www.linkedin.com/pulse/complete-guide-how-learn-cavalry-app-elena-kudriavtseva-ubikc) | **原子粒の発見可能性死**(F-8と同型の実体験) |
| Path Offsetはチュートリアル題材として紹介されるが「AEより良い」比較は薄い | [Lesterbanks](https://lesterbanks.com/2020/11/working-with-cavalrys-path-offset-behavior/)(ベンダー動画の再掲) | パス単体の民意データは弱い |
| Envato系「switching」記事 | [Envato](https://elements.envato.com/learn/cavalry-motion-graphics) | マーケ寄与大。リアルタイム・データ・プラグイン不要を強調。**Canva買収文脈**あり — 独立審判として割り引く |

リポジトリ既存: Cavalryは「技術者すぎる」(F-8)、docs発見性の弱さ(友人証言・[spec-holes](2026-07-12-d1-spec-holes-prior-art.md))。声のサンプルと整合。

### 「どちらが良い？」への仮答え(PathOp確定ではない)

0. **豊富さ**: カタログ種数でも重なり族の厚みでも **Cavalryが勝つ**(上節スコアカード)。ユーザー観察は正しい。
1. **ユーザー露出の形**: AEの意図単位閉集合(Trim / Offset / ZigZag / Repeater…)の方が、声としても教育コストとしても勝つ。Cavalryの「200+を組み立てる」は実務でもoverwhelm報告がある → Motoliiの**メニュー形**はAE型を維持(既存F-13/F-8)。**語彙の天井はAEに合わせない。**
2. **パラメータの厚み・相互作用の痛み**: AEフォーラムの慢性痛(Trim×Offset、first vertex、内側ストローク無し、開パスcopies、複製の天井)は、Cavalryが別口で解いている領域と重なる → **意味を焼くときAE最小実装だけを正解にしない**。
3. **賛辞の帰属**: Cavalry称賛の本丸の一つはDuplicatorだが、パス変形カタログ自体もAE Path operatorsより広い。それをPathOp閉集合の無制限拡大の根拠にはしない(発見可能性)。
4. **したがって採用方針(仮)**: 「AEの意図ラベル × Cavalryの豊富さから痛い角・厚いパラメータを選択取り込み」。スープ全体は採らない。未決6点は、この仮方針の下でユーザー判断を待つ。

## いまやらないこと

- 本メモの数値・モードをスキーマやゴールデンに焼く
- CavalryのBehaviourグラフをプラグイン契約へ露出する
- 「Cavalryの方が発達→閉集合を広げる」への短絡(F-8逆張りを崩す)
- AEフォーラムの回避策文化やCavalryマーケ記事を【決定】根拠にする

次工程: 上記未決をユーザー判断で潰したあと、M2「PathOp意味論表」を【決定】へ昇格し、その写しとしてD1i-2実装。
