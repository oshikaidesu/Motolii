# Cavalry の「いい例」を棚の札に写す索引 — Example Scenes・Behaviour/Utility・代表の画 10・純関数だけで写せる 10(2026-09-18 取得)

課題: Cavalry の代表的な画を 1 個ずつ Motolii の棚の札(Vism block = `fn block(k, time, inputs) -> {位置・回転・大きさ・色/不透明}` の純関数)に写すための一覧。Motolii 側は「物の番号 k と時刻 t の純関数」だけを写せる。記憶を持つ物(Value Solver、Trails、Forge Dynamics、Particles、Spring、Mesh Solver)は写せない(解き手側)。一次資料のみ: cavalry.studio/docs(旧 docs.cavalry.scenegroup.co は全て 301 でここへ)、scenery.io の @cavalry 公式 scene、Scene Group 公式 YouTube の Reel 2025、Medium の公式 blog(Cavalry Animation)。docs の一覧ページは名前だけで定義文が無いので、各 node の個別ページを 130 頁ほど当たった。取れなかった物は末尾。

## 0. 要約(5 行)

1. **公式 Example Scenes は 14 本(docs)+ scenery.io/@cavalry に 17 本**。全て `Duplicator × (Stagger | Oscillator | Noise | Value Array | Falloff)` の組で、**記憶を持つ node は 1 本も使っていない**(Joy Rig の Rig Control と Night and Day の Animation Control も「keyframe 曲線を % で読む」純関数)。
2. **Behaviour 78・Utility 100・Distribution 21・Layout 3 のうち、記憶を持つ物は明確に少数**: Value/Value2 Solver(「Store the values from previous frames」)、Mesh Solver(「Keep Previous Meshes」)、Spring、Motion Stretch(速度)、Velocity/Velocity Magnitude Context、Trails、Particle 系 15、Forge Dynamics の Field/Constraint/Collision Event 16、Timeline Counter(Accumulate)、Local Time(実時計)。**残りは全部 (k, t) の純関数**。docs 自身の言葉は "Behaviours can be used to animate values or deform Shapes"、Context の頁「During those 10 times, Random receives Index values of: 0…9」。
3. **Cavalry の芯は "Duplicator が index を配り、Behaviour が index と time で値を返す"**(Context 頁)。Stagger は「first Id に Minimum、last Id に Maximum、間は補間」+ `Stagger Value + Current Frame = Shape Time Offset` の式。これは Motolii の Stagger(GSAP distribute)と同型。
4. **Motolii に無くて純関数で写せる物の上位**: Falloff(形で強さを掛ける、Circle/Rectangle/Linear/Sweep/Shape + Graph + Probability)、Range Falloff(index の範囲で掛ける、Transition/Travel)、Noise の場(Simplex/Cellular/Cubic/Value、Position/Index Context、Loop)、Sequence(非重複の並べ替え、Travel)、Modulate(mod で 0,1,2,0,1,2)、Distribution の族(Circle/Grid/Fibonacci/Rose/Path/Shape Edges/Sort/Shuffle)、Visibility Sequence、Number Range/Area Range、Position Blend、Blend Sub-Mesh Positions。
5. **Scenery の "Sync"(利用者の当面の目標)は ewn 作(scenery.io/scenes/sync-rQysTIq1bYH、Cavalry 2.5.2)**。使用 node は Duplicator・Connect Shape・Forge Dynamics・Attractor Field・Color Collision Event・Pathfinder・Wave・Noise・Stagger・Random・Falloff・Align・Frame・Math・String 系・Shader/Filter 9 種。**Forge Dynamics + Attractor + Collision Event の部分だけが記憶持ち**、残り(升目の箱・円の切り欠き・Wave/Noise の揺れ・Pathfinder の円の道)は純関数。poster は 4×4 の箱の升目に円と半円の切り欠き。

## 1. 公式 Example Scenes(docs「Example Files」14 本 + scenery.io/@cavalry の追加 3 本)

docs: https://cavalry.studio/docs/getting-started/example-files/ (「All the examples below are available to download via Scenery」)。scenery.io の各頁に「Made in Cavalry x.y.z with …」の node 一覧と poster.png がある。

| 名前 | 何を見せる scene(docs の説明) | 主な Behaviour/Utility(docs + scenery の "Made with") | 純関数か | URL |
|---|---|---|---|---|
| Text, Sleep, Repeat | Text Shape の String Manipulator = Transition。Percentage を scrub すると Final Text へ文字列が遷移。String Array + Duplicator + Group | Text Shape, String Manipulator(Transition), String Array, Duplicator, Group, Bevel, Color Array, Custom Shape | 純 | https://scenery.io/scenes/text-sleep-repeat-WcMTh0V9DbP |
| Infrequency | Line を Duplicator で並べ、Oscillator を Deformer(Use Normals)に。Stagger を Oscillator の Frequency に繋いで線ごとに周波数をずらす | Basic Line, Duplicator, Oscillator, Stagger | 純 | https://scenery.io/scenes/infrequency-pEHqGo9q8o1 |
| Concentrick | Circle を Point Distribution の Duplicator に。Stagger で各円の Radius を段階に、Oscillator で揺らす(同心円の干渉縞) | Basic Shape, Duplicator, Stagger, Oscillator | 純 | https://scenery.io/scenes/concentrick-bKzaa2YsAMq |
| Mazin | 45° の Basic Line を Group 下に。Value Array {0, 90} + Value Behaviour で向きを切替、Falloff と JS Math で升目の迷路模様 | Basic Line, Group, Duplicator, Value Array, Value, Falloff, Jsmath, Oscillator | 純 | https://scenery.io/scenes/mazin-A80p6r9DrqC |
| Joy Rig | Rig Control(joystick)と Keyframe Layers のデモ。5 つの pose(中央・左右・上下 = frame 0〜4)を 2D で補間 | Rig Control, Keyframe Layers, Editable Shape | 純(pose の補間) | https://scenery.io/scenes/rig-control-d1OHdBJSO1j |
| Ring Ting | 点を Path Distribution(円)で環に。Input Path の円を Noise で変形、Falloff で範囲を絞る | Basic Shape, Duplicator, Path Distribution, Noise, Falloff | 純 | https://scenery.io/scenes/ring-ting-GRUG07vs3gJ |
| Interlink | Arc を Point Distribution で複製、Value Array で回転を指定。Duplicator の入れ子で組み紐 | Basic Shape(Arc), Duplicator(nested), Value Array, Color Array | 純 | https://scenery.io/scenes/interlink-2k3gXRKjXRg |
| Focus | Circle に Conical Gradient Shader(Overlay)+ Blur Filter、Blur の Amount を Oscillator で | Basic Shape, Duplicator, Oscillator, Fast Blur, Gradient Shader | 純 | https://scenery.io/scenes/focus-J9JaB3umkHv |
| Optical Art | 円を別の円で mask、Mask の Position.x を Stagger に。Oscillator + Modulate で交互の位相 | Basic Shape, Duplicator, Stagger, Oscillator, Modulate | 純 | https://scenery.io/scenes/optical-art-wjFjSbxGqms |
| Button | UI のボタンの mock。Value2 を Drop Shadow の Offset に、Falloff で押下 | Basic Shape, Duplicator, Group, Value2, Color Array, Falloff, Drop Shadow, Gradient Shader | 純 | https://scenery.io/scenes/button-n9BfjgRtQrY |
| See | 矩形を Frame(時刻)で回転、Rectangle(Path)の上に Path Distribution で複製、Noise で乱す | Basic Shape, Duplicator, Frame, Noise, Path Distribution | 純 | https://scenery.io/scenes/see-b7HkEaFKhrr |
| Night and Day | 「rig」。Null の x を Value/Value2 の乗数にして Moon の位置・Sun の線の長さ・Stars の Count・Sky の色を一度に。Number Range・Position Blend・Animation Control・Math | Null, Value, Value2, Duplicator, Align, Number Range, Position Blend, Animation Control, Math | 純(Animation Control は「animation curve を 0〜100% に remap」) | https://scenery.io/scenes/night-and-day-3vVkbaOnaXG |
| Quad Tree | Sub-Mesh Distribution + Quad Tree Shape で他の comp を位置・拡縮(点の密度で四分木) | Quad Tree Shape, Duplicator(Sub-Mesh Distribution), Composition Reference, Noise, Random, Sequence, String Array | 純 | https://scenery.io/scenes/future-user-interface-2Z3wNmTVg8X(scenery では "Future User Interface") |
| Data | Rectangle + Text を Group、Linear Distribution の Duplicator に。Google Sheet を Spreadsheet で読み、Number Range で棒の高さ、Align で底を固定 | Duplicator, Spreadsheet, Number Range, Align, Bounding Box Constraint, Comparison, Contrasting Color, Index To Color | 純 | https://scenery.io/scenes/bar-chart-a77P2y2ismp · https://scenery.io/scenes/simple-bar-chart-MjUMy6Q0Jxo |
| Voronaffe(scenery のみ) | Voronoi Shader + Null | Basic Shape, Null, Voronoi(Shader) | 純(Shader) | https://scenery.io/scenes/voronaffe-16mbDTUweSf |
| Walk the walk(scenery のみ) | Rubber Hose Limb(2 bone IK)の歩き | Rubber Hose Limb, Random, Value, Null | 純(IK は位置の関数) | https://scenery.io/scenes/walk-the-walk-MCpQgvzi2Zd |

docs の Create > Demo Scenes には別に JavaScript の demo(「SpiroGraph Distribution」等、Custom Distribution 頁で言及)があるが、一覧頁は見つからず。

## 2. Behaviour / Utility / Distribution / Layout の一覧

判定の根拠: docs の定義文と attribute。**記** = 記憶を持つ(前 frame・simulation・cache・実時計)。**純** = (k, t, 入力) の純関数。Motolii 列は既存の札(Wave・Arrive・Hang・Bounce・Push Apart・Field・Oscillator・Repeater・Stagger・Split・Connect・Rope)との対応。

### 2a. Behaviours(78、https://cavalry.studio/docs/nodes/behaviours/ — 「Behaviours can be thought of as effects」)

値を返す物(位置・回転・大きさ・色に繋ぐ):

| 名前 | 定義(docs) | 主な attribute | 純/記(根拠) | Motolii |
|---|---|---|---|---|
| Stagger | "Generate sequential values between a minimum and maximum." | Minimum, Maximum, Offset, Graph | 純 — "the first Id is assigned the Minimum … the last Id the Maximum, with all the other Ids being assigned the values in-between"、`Stagger Value + Current Frame = Shape Time Offset` | Stagger(有。Graph = ease に当たる) |
| Oscillator | "Deform and affect Shapes using trigonometric wave patterns." | Wave Type(Sine/Cosine/Tangent), Wave Style(Normal/Square/Triangle/Sawtooth/Custom Graph), Minimum/Maximum, Value Offset, Strength Fade to Zero, Frequency, Time Mode(Seconds/BPM), Time Offset, Time Scale, Number of Waves, Stagger, Separate Channels, Use Normals | 純(時刻の三角関数) | Oscillator(有。Wave Style・BPM・Use Normals は無い) |
| Noise | "Generate noise patterns to deform Shapes or generate values." | Type(Cellular/Cubic/Simplex/Value), Min/Max, Offset, Frequency, Seed, Use Layer as Seed, Time("automatically connected to the comp's frame number. Disconnect it if static Noise is required"), Time Scale, Noise Position/Rotation/Scale, Separate Channels, Stagger, Looping + Loop Length, Use Position Context, Use Index Context, Octaves/Lacunarity/Gain/Curl | 純(seed と時刻の関数、Loop 有) | Field に近いが**無い**(Motolii の Field は場の形、Noise の場は無い) |
| Random | "Generate Random numbers." | Minimum, Maximum, Seed, Use Layer as Seed, Separate Channels, Use Bias Graph | 純(Context 頁: index ごとに Seed を変える) | Stagger の `from: Random` の生成器と同族。**独立の札は無い** |
| Value / Value2 / Value3 | "A simple Behaviour that can store and manipulate a value."(2D/3D 版も同文) | Value, Offset, Time Offset("offset any animation curves by"、負も可) | 純(Time Offset は t のずらし) | 無い(Motolii では属性の keyframe 直) |
| Value Blend / Value2 Blend / Value3 Blend | "Blend (lerp) between two one dimensional values." | First(Strength=0 で出る), Second(Strength=100 で出る) | 純 | 無い |
| Position Blend | "Blend (lerp) between two positions." | First(origin), Second(destination、Falloff が origin を通ると移る) | 純 | 無い(Falloff と組で 1 つ) |
| Color Blend | "Blend between two colours using a gradient to control the transition." | Gradient Mode, Gradient, Use Alpha | 純 | 無い |
| Number Range | "The Number Range can be used to remap values within a range." | Source Min/Max, Value, Min/Max, Graph, Clamp(外すと loop), Offset | 純 | 無い(remap) |
| Number Range to Color | "remap values to colors based on a gradient" | Source Min/Max, Value, Gradient, Clamp | 純 | 無い |
| Area Range | "Interpret a range of input values as an area and then remap them to dimensions." | Value, Maximum Value, Maximum Dimensions, Clamp | 純 | 無い |
| Modulate | "Output repeating sequences of numbers using the modulus function." | Mode(Remainder/Pass-Fail/Custom Pattern), Divisor, Index Offset, Offset, Pass/Fail Value, Custom Pattern | 純(k mod n) | 無い |
| Round | "Round values up, down or to the nearest multiple of a value." | Value, Rounding, Type(Round/Up/Down), Deformer Mode(X/Y/Both) | 純 | 無い |
| Frame | "Use and manipulate time to drive other attributes." | Value(乗数), Interpolation Mode(Frame/Seconds), Cycle Length, Graph, Offset, Start Frame, Time | 純(t × 値) | 無い(時計の秒針の例: Value −6) |
| Distance | "Drive an attribute of one layer using its distance from another." | Target, Offset | 純(2 点の距離。Measure と違い入力側の transform を駆動できる) | 無い |
| Look At | "Rotate shapes towards a target." | Target, Offset | 純(atan2) | 無い |
| Get Vector | "return the direction between a point and a target" | Target, Normalize | 純 | 無い |
| Is Within | "Switch properties based on whether a Shape's position falls within another." 0/1 | Within Shape, Invert(2 点の線なら左側が inside) | 純 | 無い |
| Align | "set dynamic pivot points by pinning Shapes based on their bounding box" | X(−1〜1), Y(−1〜1) | 純 | 無い(pivot の札) |
| Manipulator | "Apply transformations to Shapes." | Position, Rotation, Scale, Pivot | 純 | (属性そのもの) |
| 3D Matrix | "Apply 3D transformations to Shapes and Sub-Meshes." | Position/Z, Rotation/Z, Align X/Y, Use Levels, Level Mode | 純 | 無い(2.5D は札の意図で宣言、no-auto-projection) |
| Skew | skew 変換 | Strength, Skew[x,y], Pivot X/Y | 純 | 無い |
| Visibility Sequence | sub-mesh の可視を範囲で | Start %, End %, Travel, Always On/Off("1,3,5:8"), Invert | 純 | 無い |
| Auto-Animate | "Add a variety of animations to Shapes with only a few keyframes." | Position Mode(Slide from Edges/Drift/Zoom), Rotation(Spin/Swing), Scale(Up/Down), Opacity(Fade), Visibility, Progress(100% で完了), Use Levels(Text Characters/Words/Lines), Timing Mode(Normal/Random/Middle Out) | 純(Progress の関数) | Arrive に近い(入場の型) |
| Behaviour Mixer | "Combine and control the effect of multiple Behaviours." | 各 index: Input Behaviour, Enabled, Strength, Blend Mode(Add/Min/Max/Minus/Multiply/Screen/Overlay)。Deformer 非対応 | 純 | 無い(札の合成の法) |
| Material Sampler | "samples an Input Shape and then outputs values of between 0-1 depending on the value (lightness/darkness)" | Value, Input Path, Graph, Offset | 純(画素読み) | 無い |
| Sound | 音声の周波数帯を値に | Frequency Scale(Log/Mel/Bark/Linear), Range, Bands, Clip, A-weighting, Sub Steps, Smoothing Frames, Frame Offset, Use Index Context | 純(音声 file の時刻の関数。Smoothing は前 frame を混ぜるが file 由来で再現的) | 無い(DAW を再発明しない) |
| Value Solver / Value2 Solver | "Store the values from previous frames in order to accumulate and/or fade them over time." | Mode(Accumulate/Highest/Lowest/Velocity), Fade Mode/Value, Start Frame, Use Cache(.sdcache), Cache Offset | **記** | 解き手側 |
| Spring | "Add dynamic secondary motion to a Shape's position keyframes." | Damping, Mass, Speed Limit(実験機能) | **記**(2 次の動き = 速度を持つ) | Bounce/Hang は純関数で似せる型 |
| Motion Stretch | "Deform shapes based on their motion to create an effect similar to smearing." | Speed Threshold(実験機能) | **記**(速度) | 無い(解析で速度を取れば純に出来る) |
| Squash and Stretch | "Squash and stretch Shapes with options to add bulge and preserve a Shape's area." | Amount, Pin(Centre/Top/Bottom), Bulge, Area Preservation, Subdivide | 純(Amount を自分で keyframe) | 無い |

形を変える物(Deformers — 頁は behaviours/ の下。/nodes/deformers/ は 404):

| 名前 | 定義(docs) | 主な attribute | 純/記 | Motolii |
|---|---|---|---|---|
| Wave | "Create looping wave distortions along open and closed Paths." | Mode(Sine/Square/Sawtooth/Triangle), Number of Waves, Adaptive Wave Counts, Amplitude, Travel, Sample Points, Output Béziers | 純 | Wave(有。Motolii の Wave は値の波、Cavalry の Wave は path の変形) |
| Bend Deformer | "Deform Shapes by bending them around the circumference of a circle." | Bend Angle, Direction(X/Y), Pin, High Quality, Use Levels | 純 | 無い |
| Push Along Vector | "Deform a Shape along a vector." | Use Normals, Direction, Reverse, Strength, Falloff | 純 | Push Apart とは別(点の押し出し) |
| Pinch | "Pinch and pull Shapes within a Falloff by moving a Null." | Active, Bind Area(Falloff), Pinch Point(Null) | 純 | 無い |
| Lattice Deformer(2.6) | "Deform Shapes using a flexible grid of control points." | Grid Count, Triangulate, Deformers, Controllers | 純 | 無い |
| Four Point Warp(2.6) | "Deform Shapes using a four corner warp with bézier handles." | Centre, Size, Triangulate, Quality, Corner/In/Out Handle × 4 | 純 | 無い |
| Flare | 縦/横に先細り | Direction, Amount, Start/End Stretch, Graph | 純 | 無い |
| Travel Deformer | "Reposition the start points for all Contours within a Path." | Travel % | 純 | 無い |
| Pathfinder | "Move or deform closed Shapes along a path." | Travel, Loop, Reverse, Input Shape, Taper Graph, Path Offset, Flip, Rotation Offset, Rotation(出力) | 純(Travel の関数) | Rope に近い(道の上の位置) |
| Path Offset | "Shrink or grow paths" | Closed Mode(Single/Double), Cap Style, Offset, Rounded | 純 | 無い |
| Chop Path | "Chop a Shape into several slices." | Count, Angle, Offset, Spread, Flatten Shape | 純 | Split に近い(切る単位が違う) |
| Stitches | 閉じた path を縫い目で塗る | Distance, Spread, Disconnection Multiplier, Rotation, Keep Original Path | 純 | 無い |
| Knot | "Add gaps to open or closed Paths where a Contour self-intersects." | Mode(Over/Under First, Both, Geometric), Gap Width | 純 | 無い |
| Morph | 点数が違う形の間を morph | Mode(Simple/Wave=FFT), Morph Divisions, Sampling Offset, Strength | 純 | 無い |
| Blend Shape | "Blend between multiple Shapes with matching verb Counts." | Mode(Replace/Deltas), Input Shape, Strength | 純 | 無い |
| Blend Sub-Mesh Positions | 2 つの Shape の sub-mesh の位置の間 | Blend, Destination Shape, Use Global Positions, Time Offset | 純 | 無い(Grid → Circle の並び替えの例) |
| Boolean | "remove or add shapes to other shapes" | Mode(Union/Subtract/Intersect/Exclude), Clipping Shapes。"Booleans are additive" | 純 | 無い(交わりの箱に要る) |
| Voxelize | "Convert Shapes into voxels with control over their size and scale to create gaps." | Size, Voxel Scale | 純 | 無い |
| Bevel | "a deformer to create round or mitred edges" | Mode(Fillet/Chamfer), Radius Mode(Per Sub-Mesh/Per Point), Radius, Min/Max Angle | 純 | 無い |
| Rubber Hose Limb | "A simple 2 bone IK system for bendy or straight limbs." | Start/End Controller, Joint Position, Stretch, Curvature, Flip, Length, Taper Graph | 純 | 無い |
| Path Relax(2.6) | "Move path points away from each other." | Iterations, Radius, Relaxation Strength | 純(反復は frame 内) | Push Apart と同族(点版) |
| Path Average(2.6) | "Smooth Path points by averaging their positions." | Iterations, Cutoff Frequency | 純 | 無い |
| Mesh Solver(2.6) | "Connect deformers to create drawing machines or iterative effects like differential growth." | Level Mode, Keep Previous Meshes("Number of previous frame results to keep") | **記** | 解き手側 |
| JavaScript Deformer | 点を code で | Expression(def.getPoints/setPoints/getBoundingBox) | 純にも記にも(context は frame を跨いで持てる) | 台本 |
| Add Divisions / Subdivide / Resample Path / Curves to Lines / Clean Up / Reverse Path / Fill Rule / Extend Open Paths / Contours to Sub-Meshes / Flatten Shape Layers / Auto-Crop / Sub-Mesh / Apply Distribution / Apply Layout | 幾何の整形(divisions・平滑・簡略・向き・塗り規則・端の延長・contour→sub-mesh・平坦化・crop・階層の指定・sub-mesh に Distribution/Layout を掛ける) | 各頁の通り | 純 | 無い(幾何の道具、札ではない) |
| Color/Alpha/HSV Material Override, Swap Color Override | Sub-Mesh の Fill/Stroke を上書き、Falloff で範囲 | Color / Alpha / H,S,V / Old,New Color | 純 | 無い |

### 2b. Utilities(100、https://cavalry.studio/docs/nodes/utilities/ — "general purpose building blocks")

| 名前 | 定義(docs) | 主な attribute | 純/記 | Motolii |
|---|---|---|---|---|
| Falloff | "By default, a Falloff outputs a value of 1 at its centre which falls off to 0 at its edges." "used as a multiplier for the Strength of the Behaviour(s)" | Shape(Circle/Rectangle/Linear/Sweep/Shape), Strength, Size, Repetitions, Angles, Path Mode(Filled/Edges), Distance, Falloff Graph, Probability, Seed | 純 | Field に近いが**無い**(Motolii の Field は力の場、これは強さの乗数) |
| Range Falloff | "similar to the Falloff … but rather than using a shape, it uses a range." | Strength, Mode(Specific Indices/Percentage/Transition), Indices("0,1:3,8", first/last), Start/End, Offset(travel), Transition Size, Completion, Use Graph | 純(k の関数) | 無い |
| Sequence | "Generate random (or non-random), non-repeating number sequences." | Auto Index, Index, Sequence, Offset, Travel, Randomize, Seed, Skip Indices。"the same value (or color) will never appear twice" | 純(seed の順列) | 無い |
| Index Context | "Layers that have an input connection from other Layers may choose to provide context to these input layers" | Depth, id | 純(k の取り出し) | (k そのもの) |
| Length Context | "read the length of a path or paths and pass values through" | Remap(None/Area Range/Number Range/…to Color) | 純 | 無い |
| Velocity Context / Velocity Magnitude Context | "Extract a Shape within a Duplicator's direction." / "extract speed data" | Strength, Offset | **記**(速度、実験機能) | 無い |
| Accumulator | "Create 'stacks' of Shapes or custom layouts." | Value, Padding, Offset | 純(index までの和) | 無い(積み上げ) |
| Value Array / Value2 Array / Value3 Array | "Create a list of values that can be assigned to other Layers via their indices." | Mode(Index/Min/Max/Average/Accumulate), Auto Index, Array Index, Reverse | 純 | 無い(表の引き) |
| Color Array / Shape Array / String Array / Asset Array / Shader Array / Typeface Array | 同型の配列(index で引く) | Auto Index, Index, Reverse, Count | 純 | 無い |
| Index to Color | "Map a gradient to each index" | Value, Gradient Mode, Reverse, Use Index Context | 純 | 無い |
| Layer Seed | "that attribute's parent Layer to pass unique values to its output connections" | Offset | 純 | (seed) |
| Math / Math2 / Math3 / Jsmath | 四則・JS 式(`n0`, `n1`…) | First, Operation, Second / Expression | 純 | 台本 |
| Comparison / If Else / Logic | 比較 0/1・分岐・AND/OR/XOR(≥0.5 = true) | First, Second, Operation | 純 | 台本 |
| Animation Control | "remaps an attribute's animation curve … to a percentage" | Active, Amount(0 = 最初の key, 100 = 最後) | 純(曲線を % で読む) | 無い(Night and Day の芯) |
| Frame(→Behaviour) / Seconds to Frames | 秒→frame(`FPS∗Seconds`) | Seconds, FPS | 純 | — |
| Local Time | 実時計を出す。"primarily a playback feature and so may not always render to file as expected" | Strength, Mode(ms〜years, Day of Week), Output Mode(Unit/Ratio), Offset | **記**(実時計) | 不採用 |
| Timeline Counter | "Use Time Markers or Pacing Markers to accumulate or trigger a value over time." | Mode(Accumulate/Trigger), Marker Type(Time/BPM/Seconds), Convolution, Color filter | Trigger は純(marker 上か)、Accumulate は marker の数 = 時刻の純関数だが docs は「playhead が通るたび +1」 | 無い |
| Scheduling Group | "Procedurally position child layers in time." | Child Offset, Sequencing, Overlap(負で gap), Schedule from End, Ordering Policy(Layer Order/Alphabetical), Flip Order | 純(時刻の配置) | Stagger の時間版と同族(Overlap は無い) |
| Null | "drawable … but not renderable"、Limit Position/Rotation/Scale | Shape, Limits | 純 | (制御点) |
| Rig Control | "a 2d controller (or joystick …) which can be used for pose-based rigging" 5 pose = frame 0〜4 | Active, Keyframe Layer | 純 | 無い |
| Measure / Distance Constraint 等 | Measure: "the distance between two Shapes" | First/Second Target | 純 | — |
| Radius / Bounding Box / Bounding Box Constraint / Composition Constraint / Component Constraint / Transform Constraint | 外接円・外接箱・箱の辺に拘束・comp の辺に拘束・他の形の点/辺に拘束・他の transform に追従(Position/Rotation Strength, Rest) | 各頁 | 純(Bounding Box の Sample at Frame も純) | Connect の「線を張る先」と同族。**無い** |
| Get Sub-Mesh Transform / Count Sub-Meshes / Path Length / Get Name / Measure Text / String Length | sub-mesh k の transform を読む・数・path 長・名前・文字の箱 | Level Mode, Sub-Mesh Index | 純 | 無い(k の transform を読む口は要る) |
| Spreadsheet / Spreadsheet Lookup / Data 系 | Google Sheet/CSV/XLSX の列を index で読む、Factorize、Remap | Column, Row, Sort Order | 純 | 無い |
| String / String Generator / String Manipulator / String From Asset / Typeface / Apply * (Font Size/Style/OpenType/Text Material/Typeface/Character Spacing) | 文字列の生成・遷移・正規表現の範囲指定(Regex/Specific Indices/All + Range Falloff) | Mode, Indices | 純 | 無い(文字は別線) |
| Image Sampler / HSV Color / Color Info / Contrasting Color / Fill / Stroke / Stroke Duplicator | 画素の明度を 0..1・色の分解合成・WCAG 対比・色の共有・Multi Stroke | 各頁 | 純 | 無い |
| Camera / Camera Guide | 2.5D の見え(Freeform/Look At/Guide) | Position, Zoom, Look At | 純 | 無い |
| Attractor / Direction / Drag / Buoyancy / Path / Vortex Field | "in a Forge Dynamics simulation" の力(Attractor: "The further away from the Attractor, the stronger the force") | Strength, Affects Groups, Falloffs | **記**(Forge Dynamics) | Field は Motolii にも有るが**解き手側** |
| Pin / Distance / Bridge Constraint | "in a Forge Dynamics simulation" の拘束(Frequency/Damping/Breakable) | 各頁 | **記** | 解き手側 |
| Body Settings / Color / Impulse / Sticky / Visibility Collision Event | 衝突時に設定・色・力・接着・可視を変える(Fade Time 有) | Specific Collisions, Use Collision Index | **記** | 解き手側 |
| Particle Emitter / Distribution Emitter / JavaScript Emitter / Data・Force・Goal・Image・Magnetic・Path・Speed・Turbulence・Visual・Vortex・Flow Field・JavaScript Modifier | 粒子(実験機能)。Particle Shape の Time は "connected to the Composition's Time to animate the Particles on each frame" | 各頁 | **記** | 解き手側 |
| Lattice Controller / JavaScript Utility / Asset from Smart Folder | Lattice の点の操作・任意 JS・folder の index | — | 純 / JS は任意 | — |

### 2c. Distribution(Duplicator の配り方 21、https://cavalry.studio/docs/nodes/general/distribution-types/)

Duplicator: "The Duplicator can be used to copy and distribute Shapes to create grids, circles and other patterns." 属性: Distribution, Shape Position/Rotation/Scale/Visibility/Opacity(per duplicate), Shape Time Offset("Offset the animation curve for duplicates on a per shape basis")、Auto Id / Shape Id、Index Context(Advanced)。"The transform information of an Input Shape is ignored."

| Distribution | 定義(docs) | 主な attribute | 純 | Motolii(Repeater) |
|---|---|---|---|---|
| Grid | "Distribute points in a grid." | Count[x,y], Size, Pattern Offset(互い違い), Size Mode(Fit/Step), Direction(Flow Columns/Rows) | 純 | Repeater の升目に近い(Pattern Offset・Flow 順は無い) |
| Circle | "Distribute points in a radial pattern." | Count, Radius, Start Angle, Angle, Include End, Use Rotation(法線に揃える), Flip, Travel | 純 | 無い |
| Linear | "Distribute points in a horizontal or vertical line." | Count, Size, Direction, Size Mode(Fit/Step) | 純 | Repeater |
| Path | "Distribute points along a path." | Count Mode(Per Shape/Sub-Mesh/Contour), Count, Input Shape, Travel, Length, Use Rotation, Flip, Exclude End, Offset | 純 | Rope に近い |
| Point | "Distribute points at 0,0."(Spreadsheet や Value2 Array で位置を与える時) | Count | 純 | 無い |
| Array | "Distribute points by specifying positions for each index." | (n): Position | 純 | 無い |
| Random | "Distribute points to random positions." | Shape, Count, Seed, Use Probability(Image/Material Sampler), Threshold, Relax + Relax Distance + Max Iterations, Keep Shape | 純(Relax は frame 内反復) | 無い(Push Apart は動きの札) |
| Fibonacci | "Distribute points in a Fibonacci spiral pattern." | Count, Radius, Angle | 純 | 無い |
| Rose | "Distribute points in a rose pattern." | Count, Radius, Seed(歯車の半径), Length | 純 | 無い |
| Math | "Distribute points using mathematical expressions." | Count, Expression | 純 | 台本 |
| Custom | "Distribute points using a JavaScript Utility via the Point Cloud class." | Input Distribution(JS) | 純 | 台本 |
| Shape Edges / Shape Points | 他の形の辺 / 頂点の上 | Distribution Shape, Fill All, Count, Edge Bias, Use Rotation | 純 | 無い |
| Intersections | "Distribute points where paths intersect." | Intersection Shapes | 純 | 無い(交わりの箱に要る) |
| Mask | "Distribute points and remove any that fall outside another Shape." | Input Distribution, Distribution Masks | 純 | 無い |
| Sort | "Distribute points by reordering the Ids of another Distribution." | Mode(Horizontal/Vertical/Distance from Centroid/Target), Reverse | 純(k の並べ替え) | Stagger の `from` と同族。**無い** |
| Shuffle | "Distribute points by rearranging the Ids" | Reverse, Shuffle, Seed。"Shuffling does not affect the draw order" | 純 | Stagger `from: Random` |
| Sub-Mesh | "Distribute points onto the positions of child-meshes." | Input Shape, Fill All, Count, Scale To Fit, Scale Multiplier, Keep Aspect, Ignore Empty | 純 | 無い(Quad Tree の芯) |
| Transform | "Distribute points onto the position of a Layer" | Input Shape | 純 | 無い |
| Voxelize | "Distribute points by voxelizing another Shape." | Input Path, Size, Screen Space | 純 | 無い |
| Particle | "Distribute points onto a Particle Shape." | Input Shape | 記(粒子) | 解き手側 |

### 2d. Layouts(3、https://cavalry.studio/docs/nodes/shapes/layouts/layouts-intro/)

"stack Shapes by 'butting' them up against each other"、children の world-space bounding box で計算、top-level transform は無視。

| 名前 | 定義 | 主な attribute | 純 | Motolii |
|---|---|---|---|---|
| Layout Group | "Arrange Shapes in either a horizontal or vertical direction with options to wrap onto multiple rows/columns." | Direction, Wrap Mode(No Wrap/Wrap/Wrap Reverse), Min/Max Size, Margins, Spacing, Spacing Mode(Flex Start/Centre/Space Between/Around/Evenly), Line Spacing, Align Content, Ordering Policy(Layer/Alphabetical/Shuffle), Flip Order, Shuffle Seed | 純(flexbox) | Motolii の layout(store/layout.rs)と同族。関係の動きは連続性で測る |
| Grid Layout Group / Grid Layout Row | "Arrange Grid Layout Rows into columns." / "Arrange Shapes in rows" + Row Span / Column Span | Spacer(空の mesh)で隙間 | 純 | 同上 |
| Apply Layout(Behaviour、実験) | "stack Shapes within a Sub-Mesh" | Layout(Grid/Horizontal/Vertical), Size, Level, Alignment, Column/Row Span | 純 | — |

procedural な Shape で札に関係する物(https://cavalry.studio/docs/nodes/shapes/): Connect Shape("Connect distributions of points to create intricate patterns of lines." Mode = Within Range/Nearest/Point Index、Search Distance、Connection Limit、Per Point Connection Limit、Line Type Linear/Auto Bézier — Motolii の Connect と同族、Within Range と Per Point Limit は無い)、Quad Tree Shape("subdivide a square into smaller squares depending on the density of points"、Max. Iterations)、Convex Hull、Shortest Path(Distribution 上の最短路)、Points to Path、Outline、Ray(衝突で止まる線)、Segment Path、Rectangle Pattern(積み棒/円グラフ)、Isolines、Trails(**記**: "Generate trails (lines) from the movement of other Shapes." Start Frame、Limit Length)、Particle Shape(**記**)、Forge Dynamics(**記**: "Use File Cache … saved to a .sdcache file"、Cache Offset で巻き戻し)。

## 3. 「Cavalry らしい画」ベスト 10

出所: 公式 Example Scenes(scenery.io/@cavalry の poster.png を実際に見た: Concentrick・Mazin・Sync)、Cavalry Reel 2025(YouTube Z8cb_zPtzUY、2025-03-28、@valerio.dimario ほか 34 名の credit)、公式 blog「Cavalry projects we love in 2023」「Cavalry: Features Galore」(Medium、Studio Feixen・Mario De Meyer・Antonin Waterkeyn・Anthony Velen 等)、cavalry.studio/en/ の feature 12 項目(Rig control, Rubber Hose, Connect shapes, Color Palettes, Magic Easing, Text Animation, Duplicator, Data Import, Lottie Export, Forge Dynamics, Falloffs, Quad Tree)。Reel の個々の cut の帰属は動画からは取れないので、画の型は docs と Example Scenes で裏付く物だけ。

| # | 画(何が動いているか) | 要る Behaviour の組(docs の名前) | 純関数だけで作れるか | URL |
|---|---|---|---|---|
| 1 | **Sync(Scenery、利用者の目標)**: 4×4 の箱の升目に円と半円の切り欠き、箱が物理で落ちて/吸い寄せられて升目に揃い、衝突で色が変わり、線で繋がる | Duplicator(Grid) + Boolean/Intersections(交わりの箱) + Pathfinder(円の道) + Wave/Noise/Stagger(揺れ) + Connect Shape(線) + Forge Dynamics + Attractor Field + Color Collision Event + Falloff + Align + Frame + Random + Shader/Filter(Glow/Blur/Pixelate/Mirror/Wipe) | **半分**: 升目・箱・切り欠き・道・揺れ・線・fx は純。落下・吸引・衝突色は記(Forge)。Motolii の順(物理 → 交わりの箱 → 升目 → 円の道 → fx)で物理だけ解き手 | https://scenery.io/scenes/sync-rQysTIq1bYH · poster https://scenery.b-cdn.net/scenes/rQysTIq1bYH/poster.png |
| 2 | **Concentrick**: 同心円が Stagger で半径を段階に、Oscillator で呼吸し干渉縞が走る | Duplicator(Point) + Stagger→Radius + Oscillator(Stagger 付) | 純 | https://scenery.io/scenes/concentrick-bKzaa2YsAMq(poster 確認済) |
| 3 | **Infrequency**: 平行線の束が Oscillator(Use Normals)でうねり、Stagger で周波数が線ごとに違う | Duplicator(Linear) + Oscillator(Deformer, Use Normals) + Stagger→Frequency | 純 | https://scenery.io/scenes/infrequency-pEHqGo9q8o1 |
| 4 | **Mazin**: 45° の線の升目が Value Array {0,90} で向きを切替え、Falloff が通った所だけ迷路が組み変わる | Duplicator(Grid) + Value Array + Value + Falloff + Jsmath + Oscillator | 純 | https://scenery.io/scenes/mazin-A80p6r9DrqC(poster 確認済) |
| 5 | **Optical Art**: 円の mask の x を Stagger、Modulate で交互位相、Oscillator で揺れる → 縞が波打つ op-art | Duplicator + Stagger + Modulate + Oscillator + Mask | 純 | https://scenery.io/scenes/optical-art-wjFjSbxGqms |
| 6 | **Ring Ting**: 環に並んだ点、環の path を Noise が変形、Falloff で一部だけ | Duplicator(Path) + Noise(Deformer) + Falloff | 純 | https://scenery.io/scenes/ring-ting-GRUG07vs3gJ |
| 7 | **Grid → Circle の並び替え**(docs Blend Sub-Mesh Positions の例、Reel の定番) | Duplicator(Grid) + Duplicator(Circle) + Blend Sub-Mesh Positions(Blend, Time Offset) | 純 | https://cavalry.studio/docs/nodes/behaviours/blend-sub-mesh-positions/ |
| 8 | **Falloff で入場**(docs の Position Blend・Color Blend・Material Override の例、Reel の text 入場): Falloff の円が通ると各 duplicate が origin → destination へ、色が変わる、文字が現れる | Falloff(Circle/Linear/Sweep, Graph) + Position Blend / Color Blend / Auto-Animate(Progress) / Range Falloff(Transition) | 純 | https://cavalry.studio/docs/nodes/utilities/falloff/ · /behaviours/position-blend/ · /behaviours/auto-animate/ |
| 9 | **Data(棒グラフ)**: Spreadsheet の列が Number Range で棒の高さ、Align で底固定、Index to Color、Contrasting Color で文字色 | Duplicator(Linear) + Spreadsheet + Number Range + Align + Index to Color + Comparison + Bounding Box Constraint | 純 | https://scenery.io/scenes/bar-chart-a77P2y2ismp |
| 10 | **Quad Tree / Future User Interface**: 点の密度で四分木が割れ、各 cell に別 comp が Sub-Mesh Distribution で入り Sequence で順に灯る | Quad Tree Shape + Duplicator(Sub-Mesh) + Sequence + Noise + Random + String Generator | 純 | https://scenery.io/scenes/future-user-interface-2Z3wNmTVg8X |

Reel 2025 と blog の「Cavalry らしい」もう 1 段(裏付けは feature 名のみ、cut の帰属は取れず): Text Animation(Sub-Mesh の Characters/Words/Lines に Range Falloff)、Connect shapes(点群の線)、Trails(記)、Forge Dynamics の kinetic type(記)、Rubber Hose の歩き(純)。

## 4. 純関数だけで写せて Motolii に無い物 — 優先順 10

判断はこの順位まで。名前は Cavalry の語そのまま。「欄」= Motolii の札の attribute の並び。

| 順 | 名前 | Cavalry の attribute(docs) | Motolii の欄にするなら | 根拠(どの画に要るか) |
|---|---|---|---|---|
| 1 | **Falloff** | Shape(Circle/Rectangle/Linear/Sweep/Shape), Strength, Size, Repetitions, Angles, Path Mode, Distance, Falloff Graph, Probability, Seed | `falloff: {shape, center, size, angle, graph(ease), probability, seed}` → 全ての札の Strength に掛かる乗数(0..1)。既存 Field と混同しない名で | 画 1・4・6・8。台帳 5 の「CSS/GSAP に無い 1 つ = 場を属性に掛ける」 |
| 2 | **Range Falloff** | Strength, Mode(Specific Indices/Percentage/Transition), Indices("0,1:3,8", first/last), Start, End, Offset, Transition Size, Completion, Use Graph, Falloff Graph | `range: {mode, indices|start..end, transition_size, completion, offset, graph}` → k の範囲で Strength を掛ける | 画 8(文字の入場)。Split の単位 = 物と組 |
| 3 | **Noise**(値の場) | Type(Simplex/Cellular/Cubic/Value), Min/Max, Frequency, Seed, Use Layer as Seed, Time, Time Scale, Noise Position/Rotation/Scale, Separate Channels, Stagger, Looping, Loop Length, Use Position/Index Context, Octaves/Lacunarity/Gain | `noise: {type, min, max, frequency, seed, time_scale, loop_length, context(position|index), separate_channels}` → 位置・回転・大きさ・色に足す | 画 1・6・10。Use Position Context = 「関係が見えるように見える」 |
| 4 | **Sequence** | Auto Index, Index, Sequence(範囲), Offset, Travel, Randomize, Seed, Skip Indices | `sequence: {range, offset, travel, randomize, seed, skip}` → k → 非重複の別番号 | 画 10。Stagger `from: Random` の一般化 |
| 5 | **Modulate** | Mode(Remainder/Pass-Fail/Custom Pattern), Divisor, Index Offset, Offset, Pass Value, Fail Value, Custom Pattern | `modulate: {divisor, index_offset, pass, fail, pattern}` → k mod n | 画 5。Repeater の色/形の交互 |
| 6 | **Distribution の族**(Circle / Fibonacci / Rose / Path / Shape Edges / Intersections / Sort / Shuffle) | Circle: Count, Radius, Start Angle, Angle, Include End, Use Rotation, Flip, Travel。Sort: Mode(Horizontal/Vertical/Distance from Centroid/Target), Reverse。Intersections: Intersection Shapes | Repeater の `distribution` 欄に `circle{…}`, `fibonacci{count,radius,angle}`, `rose{count,radius,seed,length}`, `path{shape,travel,length,use_rotation}`, `intersections{shapes}`, `sort{mode,target,reverse}` | 画 1(升目と円の道)・2・7。Grid の Pattern Offset と Flow 順も足す |
| 7 | **Blend Sub-Mesh Positions** | Blend, Destination Shape, Use Global Positions, Time Offset | `blend_positions: {destination(別の repeater), blend, time_offset}` → 2 つの配りの間を k ごとに lerp | 画 7 |
| 8 | **Visibility Sequence** | Start %, End %, Travel, Always On, Always Off, Invert | `visibility: {start, end, travel, always_on, always_off, invert}` → k の範囲で可視 | 画 10、文字の順灯 |
| 9 | **Number Range / Area Range** | Source Min/Max, Value, Min/Max, Graph, Clamp(外すと loop), Offset / Maximum Value, Maximum Dimensions | `range: {src_min, src_max, min, max, graph, clamp}`、面積版は `area: {max_value, max_dim}` | 画 9。Spreadsheet の前に要る |
| 10 | **Position Blend + Look At + Distance** | First, Second / Target, Offset / Target, Offset | `toward: {target, amount}`(位置)、`look_at: {target, offset}`(回転)、`distance: {target}`(値) — 3 つで「関係の札」の最小 | 画 8、Connect と対 |

次点(純だが後): Frame(時刻 × 値。時計)、Accumulator(積み上げ)、Scheduling Group(Overlap)、Align(pivot)、Auto-Animate(入場の型は Arrive で足りる)、Boolean/Intersections(交わりの箱 — 幾何、re_renderer 側)、Pathfinder(Rope で代替可か要確認)、Sort Distribution の Target mode(Stagger `from` の一般化)。

## 見つからなかった物

- **docs の一覧頁に定義文が無い**(behaviours/ utilities/ は名前と link だけ)。定義は各頁から取った。`/nodes/deformers/` と `/nodes/layouts/` は 404(Deformer は behaviours/ の下、Layout は shapes/layouts/ の下)。
- **Common Attributes(Behaviours)の頁**(Strength・Falloffs・Enabled・Layer Modes の定義)は 404/検索で取れず。Falloff 頁の「used as a multiplier for the Strength」で代用。
- **Scene Group の "Made with Cavalry" 公式一覧頁は無い**。cavalry.studio/en/ は Instagram handle 8 名のみ、YouTube channel 一覧は fetch で空。Reel 2025 の cut ごとの帰属も取れず(credit 34 名の handle のみ)。
- **Scenery "Sync" の Scene Graph**(node の繋ぎ)は patron 限定で読めず。node の一覧と poster だけ。Sync の作者 ewn の説明文も無し。
- **Medium の公式 blog は 403**、r.jina.ai 経由で本文を取った(「projects we love in 2023」「Features Galore」)。動画 URL は Instagram link のみで取れず。
- **Custom Distribution / JavaScript Deformer の関数が受ける引数**(index・count・time)は頁に無く、Point Cloud class と Deformer Module の reference が要る(未取得)。
- Stagger の Graph の式、Oscillator の Custom Graph の補間、Falloff Graph の既定 ease は docs に式が無い。

## Sources(取得 2026-09-18、一次のみ)

- 一覧: cavalry.studio/docs/nodes/behaviours/ · /docs/nodes/utilities/ · /docs/nodes/shapes/ · /docs/nodes/general/distribution-types/ · /docs/nodes/shapes/layouts/layouts-intro/ · /docs/nodes/shapes/duplicator/ · /docs/getting-started/example-files/ · /docs/getting-started/key-concepts/context/ · /docs/getting-started/key-concepts/layers/ · /docs/tech-info/release-notes/2.6/2-6-0-release-notes/
- Behaviours(個別頁、/docs/nodes/behaviours/ の下): stagger · oscillator · noise · wave · random · modulate · value · value2 · value3 · value-blend · value2-blend · value3-blend · position-blend · color-blend · number-range · number-range-to-color · area-range · round · frame · distance · look-at · get-vector · is-within · align · manipulator · 3d-matrix · skew · visibility-sequence · auto-animate · behaviour-mixer · material-sampler · sound · value-solver · value2-solver · spring · motion-stretch · squash-and-stretch · bend-deformer · push-along-vector · pinch · lattice · four-point-warp · flare · travel-deformer · pathfinder · path-offset · chop-path · stitches · knot · morph · blend-shape · blend-sub-mesh-positions · boolean · voxelize · bevel · rubber-hose-limb · path-relax · path-average · mesh-solver · javascript-deformer · add-divisions · subdivide · resample-path · curves-to-lines · clean-up · reverse-path · fill-rule · extend-open-paths · contours-to-sub-meshes · flatten-shape-layers · auto-crop · sub-mesh · apply-distribution · apply-layout · color-material-override · alpha-material-override · hsv-material-override · swap-color
- Utilities(/docs/nodes/utilities/ の下): falloff · range-falloff · sequence · index-context · length-context · velocity-context · velocity-magnitude-context · accumulator · value-array · value2-array · value3-array · color-array · shape-array · string-array · asset-array · shader-array · typeface-array · index-to-color · layer-seed · math · math2 · math3 · js-math · comparison · if-else · logic · animation-control · seconds-to-frames · local-time · timeline-counter · scheduling-group · null · rig-control · measure · radius · bounding-box · bounding-box-constraint · comp-constraint · component-constraint · transform-constraint · distance-constraint · pin-constraint · bridge-constraint · get-sub-mesh-transform · count-sub-meshes · path-length · get-name · measure-text · string-length · spreadsheet · spreadsheet-lookup · data-modifier · string-generator · string-manipulator · string-from-asset · typeface · apply-font-size · apply-font-style · apply-opentype · apply-text-material · apply-typeface · apply-character-spacing · image-sampler · hsv-color · color-info · contrasting-color · fill · stroke · stroke-duplicator · camera · camera-guide · attractor-field · direction-field · drag-field · buoyancy-field · path-field · vortex-field · body-settings-collision-event · color-collision-event · impulse-collision-event · sticky-collision-event · visibility-collision-event · particle-emitter · distribution-emitter · javascript-emitter · force-modifier · goal-modifier · image-modifier · magnetic-modifier · path-modifier · speed-modifier · turbulence-modifier · visual-modifier · vortex-modifier · flow-field-modifier · javascript-modifier · javascript-utility · lattice-controller · asset-from-smart-folder
- Distributions(/docs/nodes/general/distribution-types/ の下): grid · circle · linear · path · point · array · random · fibonacci · rose · math · custom · shape-edges · shape-points · intersections · mask · sort · shuffle · sub-mesh · transform · voxelize · particle
- Shapes: /docs/nodes/shapes/ の connect-shape · quad-tree-shape · trails · particle-shape · forge-dynamics/forge-dynamics-shape · forge-dynamics/fields · rectangle-pattern · convex-hull · shortest-path · isolines-shape · ray · segment-path · points-to-path · spacer · outline · custom-shape · extract-sub-meshes · sub-mesh-bounding-box · merge · layouts/layout-group · common-attributes
- Scenery(公式 scene と Sync): scenery.io/@cavalry · scenery.io/scenes/{sync-rQysTIq1bYH, see-b7HkEaFKhrr, concentrick-bKzaa2YsAMq, infrequency-pEHqGo9q8o1, ring-ting-GRUG07vs3gJ, optical-art-wjFjSbxGqms, interlink-2k3gXRKjXRg, mazin-A80p6r9DrqC, voronaffe-16mbDTUweSf, walk-the-walk-MCpQgvzi2Zd, future-user-interface-2Z3wNmTVg8X, night-and-day-3vVkbaOnaXG, text-sleep-repeat-WcMTh0V9DbP, focus-J9JaB3umkHv, button-n9BfjgRtQrY, simple-bar-chart-MjUMy6Q0Jxo, bar-chart-a77P2y2ismp, rig-control-d1OHdBJSO1j} · poster: scenery.b-cdn.net/scenes/{rQysTIq1bYH, bKzaa2YsAMq, A80p6r9DrqC}/poster.png(目視)
- 公式の画: youtube.com/watch?v=Z8cb_zPtzUY(Cavalry reel 2025、oEmbed で題名と author を確認、credit は r.jina.ai 経由)· cavalry.studio/en/ · medium.com/cavalry-animation/cavalry-projects-we-love-in-2023-5a48fffe4dfc · medium.com/cavalry-animation/cavalry-features-galore-21b1ffe66520(いずれも Medium 直は 403、r.jina.ai 経由)
