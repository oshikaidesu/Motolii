# p5.js系ジェネラティブ表現の分類とMotoliiへの配置

作成日: **2026-07-15**
状態: **調査・配置案**（公開API、Document schema、実装順の決定文書ではない）

## 1. 結論

p5.js互換runtimeをレンダ経路へ入れるのではなく、p5.js/Processingで頻出する表現を、その意味に応じて既存のMotolii境界へ振り分ける。

1. 有限の図形生成は編集時one-shot generatorで通常のGroup/VectorRecipeへ実体化する
2. `t`と固定seedから求まる動きは閉形式または事前生成trackへ畳む
3. 過去の上流入力だけを読む効果は宣言的時間窓（TemporalFootprint）で扱う
4. 前回の自分の出力を使う蓄積描画は、ホスト所有Feedback stateとcheckpoint bakeで扱う
5. 衝突、boids、cellular automata等の逐次状態はSimulationPlugin + StateTrackへ送る
6. mouse、microphone、webcam等のlive入力はそのまま保存意味にせず、記録済みTrack/Assetへ変換する

第一選択は常に「状態を積まずに同じ見た目を作れるか」である。p5.jsコードの表面構文ではなく、表現の時間意味を判定する。

## 2. 調査範囲と一次資料

公式の[p5.js Examples](https://p5js.org/examples/)には、noise、再帰木、粒子、Soft Body、Game of Life、Mandelbrot、反射、shader等が並ぶ。[Processing Examples](https://processing.org/examples/)はCircle Collision、Flocking、Particle System、Cellular Automata、L-System、Fractal、画像処理まで含む。

逐次状態表現の分類には、p5.js版の一次教材であるThe Nature of Codeを使った。

- [Particle Systems](https://natureofcode.com/particles/)
- [Autonomous Agents / Flow Fields / Flocking](https://natureofcode.com/autonomous-agents/)
- [Physics Libraries / Collision](https://natureofcode.com/physics-libraries/)
- [Cellular Automata](https://natureofcode.com/cellular-automata/)

座標・object model・LLM向け入力形式の比較には以下を参照した。

- [Paper.js Path / Item position](https://paperjs.org/reference/path/)
- [Paper.js Project / SVG・JSON import/export](https://paperjs.org/reference/project/)
- [Paper.js View](https://paperjs.org/reference/view/)
- [SVG 2 Coordinate Systems](https://www.w3.org/TR/SVG2/coords.html)
- [Manim building blocks](https://docs.manim.community/en/stable/tutorials/building_blocks.html)

本調査は語彙・意味・境界だけを参照し、例示コードを製品へ取り込まない。

## 3. 頻出表現の配置表

| 表現 | p5.js系での典型 | 時間意味 | Motoliiでの第一候補 |
|---|---|---|---|
| grid / tile / geometric repetition | nested loopでshapeを配置 | 有限・静的 | one-shot ShapeScript → Group/VectorRecipe |
| recursive tree / L-System / Koch等 | 再帰または文字列展開 | 有限・静的 | ShapeScript。上限超過は実行前拒否 |
| parametric curve / spirograph | sin/cosで点列生成 | `f(u)`または`f(t)` | Path materialize / 通常animation |
| seeded random composition | random配置 | seed付き純関数 | ShapeScript。host由来`u64 seed`必須 |
| Perlin noise texture / motion | `noise(x,y,t)` | seed付き純関数 | ShapeScript、ParamDriver、またはWGSL |
| independent particles | birth time、初速、重力、寿命 | 閉形式に畳める | L0 particle（状態なし） |
| trails / non-clear canvas | 前frameへ追描き | 自己出力feedback | host Feedback + checkpoint bake |
| echo / frame blend | 過去の上流frameを参照 | 有界時間窓 | TemporalFootprint |
| flow-field agents | velocity/positionを逐次更新 | 逐次状態 | SimulationPlugin + StateTrack |
| boids / flocking | 近傍個体との相互作用 | 結合した逐次状態 | SimulationPlugin + spatial index |
| boundary bounce | 固定境界で反射 | 解析可能な場合あり | 解析反射を優先、必要時Simulation |
| object collision | body同士の接触・反発 | 結合した逐次状態 | SimulationPlugin + Collider |
| spring / chain / soft body | 前stepの位置・速度 | 逐次状態 | SimulationPlugin |
| Game of Life / Wolfram CA | gridₙ→gridₙ₊₁ | 決定的な逐次状態 | GPU ping-pong StateTrack |
| reaction-diffusion | fieldₙ→fieldₙ₊₁ | 決定的な逐次状態 | WGSL Simulation + checkpoint |
| Mandelbrot / pixel shader | pixelごとの式 | 現frameの純関数 | WGSL Generator/Filter |
| pointillism / halftone | imageをsampleしてshape配置 | 有限生成または蓄積 | Asset→ShapeScript、必要時Feedback |
| audio-reactive graphics | amplitude/FFTを参照 | 外部時系列入力 | Audio analysis→DataTrack |
| mouse drawing / webcam / microphone | live device state | 非決定外部入力 | editor記録→Track/AssetへBake |

### 3.1 Noiseは最優先で純関数へ畳める

p5.jsの[`noise()`](https://p5js.org/reference/p5/noise/)は、近い入力から滑らかな値を返し、`noiseSeed()`で再現できる。Motoliiでは壁時計や実行順に依存させず、Documentまたはgenerator invocation由来の固定seedと正準座標を入力にする。

```text
value = noise(seed, canonical_position, t)
```

静的texture、path wiggle、particle displacement、色変化のいずれも、まずこの形を試す。

### 3.2 再帰・L-Systemはone-shotでよい

Processingの[Recursion example](https://processing.org/examples/recursion)は`noLoop()`内で有限の再帰図形を作る。これはruntime animationではなく通常の編集時生成であり、ShapeScriptに自然に収まる。

停止条件、最大再帰深度、最大path点数、最大command数を事前に固定し、超過時に部分生成物を残さない。

## 4. 当たり判定は3種類へ分ける

Processing公式にも[Circle Collision](https://processing.org/examples/circlecollision.html)や境界反射があり、p5.js系作品では頻出する。ただし「collision」を1つの重い機能として扱わない。

### 4.1 固定境界への反射

床、矩形、固定平面など、衝突時刻を解析できる場合は反射運動を`t`の関数へ畳む。

```text
position(t) = reflected_motion(initial, velocity, boundary, t)
```

これはparticle state、physics world、checkpointを必要としない。

### 4.2 動くShapeをColliderとして読む

粒子同士は相互作用せず、外部Shapeだけに衝突する場合、Shape animation自体は`t`の純関数として評価できる。

- Circle / Rect等は解析プリミティブ
- 一般Path / SVGはSDFへ正規化
- collider recipe hashをsimulation cache keyへ含める
- 循環参照を拒否する

これは既存のSIM-6方針と一致する。単純な反射で閉じなければSimulationPluginへ昇格する。

### 4.3 物体同士の衝突・押し合い

body Aの結果がbody Bへ戻る場合は結合した逐次状態であり、L0へ偽装しない。The Nature of Codeも複雑なcollisionではMatter.js等のphysics libraryへ境界を移している。

Motoliiでは特定physics engineをDocument契約へ露出せず、固定step・固定seed・host-owned state・checkpoint replayを満たすSimulation境界へ置く。

## 5. 粒子・Flow Field・Boids

公式の[Simple Particle System](https://processing.org/examples/simpleparticlesystem)は、位置・速度・加速度・寿命を`draw()`ごとに更新する。一方、重力・風・単純抗力だけなら、同じ見た目をbirth timeと現在時刻から解析できる。

```text
particle = f(seed, birth_time, t, gravity, wind, drag)
```

このL0経路を標準にし、以下を有効にした時だけL3へ昇格する。

- 粒子間の反発・引力
- boidsのseparation / alignment / cohesion
- velocityを積分するflow field
- spring / chain / soft body
- 解けない回数のshape collision

Nature of Codeはflow fieldを2D vector gridとして定義し、agentが現在位置のvectorを読んでvelocityを更新する。見た目のflow field texture自体は純関数にできるが、それに従うagent軌跡は通常、逐次積分である。この2つを混同しない。

## 6. Feedbackと非clear canvas

p5.js公式の[Layered Rendering with Framebuffers](https://p5js.org/tutorials/layered-rendering-with-framebuffers)は、前frameと次frameの2枚をping-pongし、前frameを次frameへ描くFeedbackを紹介している。p5.jsの「backgroundを毎frame呼ばずに描画を蓄積する」表現も同じ一般形へ置ける。

```text
A₀ = transparent
Aₙ = Composite(DecayOrTransform(Aₙ₋₁), Drawₙ)
```

決定性を守る条件:

- `n=0`はclip開始時刻に固定する
- stepはComposition fps等から決まる固定値
- randomは明示seedのみ
- Feedback textureはscriptでなくhostが所有する
- K stepごとにcheckpointを保存する
- scrubは直近checkpointから再生する
- sequential renderとcheckpoint replayをpixel一致させる

### 6.1 先にmaterializeできないか判定する

`DecayOrTransform`が恒等で、有限draw命令を通常Shapeの出現時刻へ変換しても画素意味が保てる場合、Feedbackを使わない。通常layerへ実体化した方が編集可能で、frame並列とcacheも保ちやすい。

半透明の反復合成、blur、smear、前結果のscale/rotation等で前出力そのものが必要な場合だけFeedbackへ昇格する。

### 6.2 画面全体を毎step更新しない

Feedbackは論理的にはtarget surface全体を定義するが、実更新範囲は別に絞れる。

- 局所drawはdirty regionだけ更新
- blurはradius分だけregionを拡張
- transformは必要regionを逆写像する
- global shaderだけがtarget全域更新へ退化する
- checkpoint textureと中間結果はVRAM常駐を維持する

RoD/RoIを基礎に、Feedback固有のdamage伝播を実装前に仕様化する。

## 7. Chroma Keyを後段へ置く場合

Chroma Keyは基本的にpixelごとの色差判定とalpha生成であり、Feedback replayそのものより軽い。配置により再計算量と画素意味が変わる。

```text
Draw → Chroma Key → Feedback
```

新規draw regionだけkey処理して蓄積しやすい。ただし「key済み結果を蓄積する」意味になる。

```text
Draw → Feedback → Chroma Key
```

蓄積後に混ざった色を抜く意味になる。Feedback出力1枚への後処理なので順再生では軽いが、Feedback replayの各step内へ入れないことが重要。

重くなる条件はChroma Key本体より、長いcheckpoint間隔、高品質edge blur/despill、damageを無視した全域処理である。edge処理は半径分だけRoIを拡張し、key結果を通常node cacheへ置く。

## 8. Cellular AutomataとReaction-Diffusion

Nature of Codeの[Cellular Automata](https://natureofcode.com/cellular-automata/)は、現在gridから次gridを作る。決定的だがframe独立ではないため、通常Filterの隠しbufferには置かない。

- stateはGPU texture/buffer
- current/nextをping-pong
- fixed step
- host-owned checkpoint
- active regionまたはtile単位の更新
- cache keyにinitial state、rule、seed、step、Quality非依存state formatを含める

Feedback Canvasと実装部品は近いが、stateが完成RGBA画素か、cell/field値かでrecipeを分ける。

## 9. Shader・画像サンプリング・外部入力

### 9.1 Shader

p5.jsの[shader tutorial](https://p5js.org/tutorials/intro-to-shaders/)が扱うnoise、blur、vertex displacement、pixel effectは、ShapeScriptでCPU命令列へ展開せずWGSL Generator/Filterへ置く。GPU契約はwgpu/WGSL抽象だけを露出する。

### 9.2 Pointillism / Halftone

Processingの[Pointillism](https://processing.org/examples/pointillism)は画像pixelをsampleして半透明ellipseを配置する。

- 有限個ならAsset入力付きShapeScript
- 時刻とともに点を増やすならShape出現時刻へmaterialize
- 前画素への半透明蓄積が必要ならFeedback

scriptへfilesystem pathや生のGPU textureを渡さず、hostが検証済みAsset viewを提供する必要がある。

### 9.3 Audio / Webcam / Mouse

[p5.sound](https://p5js.org/reference/p5.sound/)はamplitude、waveform、FFT等を提供するが、live device値は同じprojectを再生しても一致しない。

- 音声: analysis→DataTrack
- webcam: import済みvideo Asset
- mouse/pen: editor gestureを時刻付きTrackへ記録
- clock: Composition timeへ置換

書き出し時にdeviceへ再接続しない。

## 10. LLMでポン出ししやすいAPI戦略

### 10.1 正本はMotolii ShapeScript

Paper.jsのItem/Path/Group思想を参照するが互換を名乗らず、Motoliiの正準座標を最初から使う。

- 原点中央
- Y-up
- Composition高さ=1.0
- shape位置はcenter基準
- rotation保存はradian
- `center` / `size`等のnamed field
- 1 invocation = 1 Group = 1 Undo

### 10.2 SVGを第2入口にする

SVGは公開例が多くLLMが生成しやすい。ただしSVG 2の初期座標は左上原点・Y-downなので、materialize時に正準座標へ決定的変換する。

採用subsetを固定し、script、event、外部URL、未解決font、filter graph等を黙って近似しない。SVG DOM/XMLをDocumentの実行意味に残さず、通常Group/VectorRecipeへ変換する。

### 10.3 p5.jsは互換runtimeでなく入力コーパスとして使う

LLMが生成したp5.js風コードは、そのまま毎frame実行するのでなく、次のいずれかへ分類する。

```text
finite shape commands   → ShapeScript materialize
closed-form time        → animation / ParamDriver
bounded past input      → TemporalFootprint
self-output recurrence  → Feedback + checkpoint
coupled state           → SimulationPlugin
live external input     → recorded Track / Asset
```

分類不能、上限不明、非決定入力ありの場合は型付きで拒否し、部分結果をDocumentへ入れない。

## 11. 実装前のLLMコーパス試験

API名を先に凍結せず、同じprompt群で候補表面を比較する。

最低20 fixture:

1. grid / tile
2. radial repetition
3. recursive tree
4. L-System
5. spirograph
6. seeded noise dots
7. noise-deformed path
8. independent ballistic particles
9. particle trail
10. non-clear paint accumulation
11. framebuffer feedback rotate/scale
12. boundary bounce
13. circle-circle collision
14. shape collider
15. flow-field agents
16. boids
17. Game of Life
18. reaction-diffusion
19. image pointillism
20. audio FFT bars

測定項目:

- 初回parse成功率
- 正準座標違反率
- seed欠落率
- 誤った実行境界への分類率
- 修正ターン数
- 通常Shapeとして編集可能に残った割合
- unsupported機能を黙って近似した割合
- 同一入力からのcommand batch / pixel再現性

この試験でShapeScript、SVG、限定syntax sugarの優先順位を決める。GitHub starsやdocs量だけでAPIを固定しない。

## 12. 実装候補の分割

| 候補 | 内容 | 依存 |
|---|---|---|
| GEN-1 | runtime非依存one-shot generator→型付きD2 command batch→1 macro | D2 command/writer |
| GEN-2 | ShapeScript Path/Shape/Group + 正準座標 + seed + resource limit | GEN-1, PathOp |
| GEN-3 | SVG materialize adapter + 安全subset + Y-down変換 | GEN-2, SVG importer |
| GEN-4 | Accumulation/Feedback Canvas + checkpoint replay + damage伝播 | F-11, M4-K1/K7, GEN-2 |
| GEN-5 | p5系20 fixtureによるLLM生成・分類conformance | GEN-2〜4 |
| SIM-FU | Collider/boids/CA/reaction-diffusionの参照Simulation | SimulationPlugin, StateTrack, SIM-6 |

公開trait、Document variant、script保存形式、JS engineをGEN-1で同時に決めない。意味論fixtureを先に固定し、既存D2・F-11・F-12境界で不足する場合だけ仕様改訂へ戻る。

## 13. 非目標

- p5.js完全互換
- browser DOM / Web Audio / webcam APIの再実装
- 再生head依存の隠しcanvas
- 未seed乱数、壁時計、thread順依存
- physics engine固有型のDocument永続化
- CPU pixel loopによるVRAM常駐原則の迂回
- unsupported表現のsilent fallback
