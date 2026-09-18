# 「番号 k と時刻 t の純関数が、物ごとの属性を書く口」— 先例 8 本の定規(2026-09-18 取得)

課題: Motolii の block(`vism/*.wgsl`、`fn block(k: u32, p: BlockParams) -> Offset`、`Offset { translate: vec2f, rotate: f32, scale: f32 }` — crates/motolii-render/src/compositor/effects/block_program.rs:110)を、不透明・色・時間のずれ等へ広げる前に、先例が「物ごとの属性の集合」と「複数の法の合成」をどう設計しているかを一次資料(公式 docs・SDK reference・作者本人の文)だけで並べる。判断・提案は書かない。取れなかった物は §9 に書いた。

## 0. 要約(5 行)

1. **属性の集合は 3 型に分かれる**。(A) 固定の名前付き struct 型 — AviUtl `obj.ox/oy/oz/rx/ry/rz/cx/cy/cz/zoom(sx/sy/sz)/alpha/aspect`、C4D MoData `MATRIX/COLOR/SIZE/UVW/FLAGS/WEIGHT/CLONE/TIME/…`、MASH `position/rotation/scale/id/visibility/velocity/angularVelocity` + CPV 色。(B) 名前規約 + 任意追加 — Houdini(`@P @N @Cd @Alpha @pscale @orient @id @v`、書けば作られる)、MASH の named array、C4D `MoData.AddArray`。(C) 属性という概念が無い — Processing/p5(draw が全部書く)、ISF(出力は画素だけ)。
2. **加算/上書きは「法ごとの mode」が主流**。C4D Effector: Transform Mode = Relative(足す)/Absolute/Remap、Color = Blending Mode(Mix/Add/Subtract/Multiply/Divide)。MASH Offset: Offset(足す)/Multiply/Overwrite/Multiply by Time、Color: Normal(上書き)/Add/Subtract/Multiply/Screen/Overlay。AviUtl: `obj.ox` 等への代入は上書き(スクリプト側が `obj.ox = obj.ox + …` と書けば加算)。Cavalry: 1 属性に入力は 1 本だけ、複数は Behaviour Mixer(Normal(Add)/Min/Max/Minus/Multiply/Screen/Overlay、上の index が下に効く)。
3. **複数の法の順は「一覧の上から下」**。C4D Cloner の Effectors tab は drag で順を変え、Delay は「効かせたい Effector の後(下)に置け」と明記。MASH も node の積み(Waiter に足した順)。Cavalry は入力 1 本の制約で順序問題を Mixer に押し込む。Houdini は node graph の順そのもの。
4. **時間のずれ(time offset)を属性として持つ物**: C4D `MODATA_TIME`(Cloner Transform tab の Time + Animation Mode Play/Loop/Fixed/Fixed Loop、Effector の Time Offset)、Cavalry Duplicator `Shape Time Offset`(Stagger と `Stagger Value + Current Frame = Shape Time Offset`)、MASH Time node(component animation を offset)、AviUtl `obj.multiobject(num, func)` の func が **秒の time offset を返せる**(AviUtl2)。Houdini には無い(`@Time` は読むだけ)。GSAP stagger は start time の per-target offset。
5. **色**: C4D `MODATA_COLOR` + Color Mode、MASH CPV + Color node、Houdini `@Cd`/`@Alpha`(viewport と renderer が名前で読む)、Cavalry は Duplicator に色の欄は無く Index to Color / Color Array を Shape の fill に index 文脈で繋ぐ、AviUtl は `obj.alpha` のみで色は無い(`obj.effect("単色化",…)` 等の filter 経由)。ISF/p5 は画素そのもの。

## 1. AviUtl 拡張編集 anm/obj(個別オブジェクト)

一次資料の状況: 原本 `lua.txt` は exedit92.zip 同梱で web 上に単独公開は無し(公式サイト https://spring-fragrance.mints.ne.jp/aviutl/ に doc の直 link は無し)。ここでは (a) scrapbox /aviutl の `obj` ページ(`lua.txt` の項目名を原文のまま列挙、API で本文取得)、(b) AviUtl2 の lua.txt を有志が写した docs.aviutl2.jp(「有志が作成した非公式ミラー」「原著 KENくん」と明記、AviUtl2 版なので `sx/sy/sz` 等 1.x に無い物を含む)を使う。

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| 書ける変数(相対座標・回転・中心・その他) | `obj.ox, obj.oy, obj.oz`(相対座標)、`obj.rx, obj.ry, obj.rz`(回転)、`obj.cx, obj.cy, obj.cz`(中心座標)、`obj.zoom`(拡大率)、`obj.alpha`(不透明度)、`obj.aspect`(縦横比、`[-1,1]`)。AviUtl2 では加えて `obj.sx, obj.sy, obj.sz`(Read/Write、拡大率) | scrapbox `obj` · docs.aviutl2.jp/lua/ |
| 読み取り専用 | `obj.x, obj.y, obj.z`(基準座標)、`obj.w, obj.h`、`obj.screen_w, obj.screen_h`、`obj.framerate`、`obj.frame`(現在フレーム)、`obj.time`(現在時間)、`obj.totalframe`(オブジェクトの長さ(f))、`obj.totaltime`(オブジェクトの長さ(sec))、`obj.layer`、`obj.track0..3`、`obj.check0`。「(読み取り専用)と書かれている変数は，書き込んでも拡張編集は反映させない」。AviUtl2 に `obj.id`(ReadOnly、Unique object ID) | scrapbox `obj` · docs.aviutl2.jp |
| 番号と総数 | `obj.index` ― 何番目の個別オブジェクトか、`obj.num` ― 個別オブジェクトの総数。AviUtl2 版原文: `obj.index`「複数オブジェクト時の番号」、`obj.num`「複数オブジェクト時の数(1=単体オブジェクト/0=不定)」、「個別オブジェクトを設定した場合に複数オブジェクトの値が設定されます」 | 同上 |
| 時刻 | `obj.time`「オブジェクト基準での現在の時間(秒)」、`obj.totaltime`「オブジェクトの総時間(秒)」、`obj.frame`「オブジェクト基準での現在のフレーム番号」 | docs.aviutl2.jp |
| 個別オブジェクト on/off | `obj.getoption("multi_object")`「個別オブジェクトが有効かどうかを boolean で返す」。on の時は文字/図形 1 つずつに対しスクリプトが走り `obj.index`/`obj.num` が入る(off では num=1)。AviUtl2 では `obj.multiobject(num, func)`: 「Renders object as multiple individual instances. callback func executes once per instance and **may return time offset in seconds**」。例: `obj.multiobject(#text, function() obj.load("text", text[obj.index+1]); obj.ox = ox; ox = ox + obj.w end)` | scrapbox `obj.getoption` · docs.aviutl2.jp/lua/examples |
| `obj.getvalue(target[, time[, section]])` | target: `0..3`(track)、`"x" "y" "z" "rx" "ry" "rz" "zoom"(100 で等倍)"alpha"[0,1] "aspect"[-1,1] "time" "layer<番号>.<種類>" "scenechange"`。注記「xyz やら拡大率やらは『オブジェクト』の値を取得しにいく 後から基本効果等で変更しても反映されない」。AviUtl2 は加えて `"pos" "angle" "center" "sx/sy/sz" "scale" "frame_s" "frame_e"` | scrapbox `obj.getvalue` · docs.aviutl2.jp |
| `obj.setoption(name,…)` | `"culling"`(裏面を表示しない)`"billboard"`(カメラの方向を向く)`"shadow"` `"antialias"` `"blend"`(合成モード)`"drawtarget"`(描画先の変更)`"draw_state"` `"focus_mode"` `"camera_param"`。AviUtl2: `"blend"` の値 `none/add/sub/mul/screen/overlay…`、`"sampler"`(clip/clamp/loop/mirror/dot)、`"camera_focus"` | scrapbox `obj.setoption` · docs.aviutl2.jp |
| `obj.getoption(name)` | `"track_mode" "section_num" "script_name" "gui" "camera_mode" "camera_param" "multi_object"`(+ AviUtl2: `"group_info" "camera_focus" "blend" "culling" "billboard" "drawtarget" "draw_state"`) | 同上 |
| 描画の口 | `obj.draw([ox,oy,oz,zoom,alpha,rx,ry,rz])` — 引数は全て省略可、obj.* の値に対する相対。`obj.drawpoly`、`obj.effect([name, param, value,…])`(引数無しなら「スクリプト以降のフィルタを実行」) | docs.aviutl2.jp |

## 2. Cinema 4D MoGraph Effector

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| 物ごとのデータ(MoData 配列) | `MODATA_MATRIX`「Matrix of the clone」、`MODATA_COLOR`「Color of the clone」、`MODATA_SIZE`「Size of the clone」、`MODATA_UVW`「UV position of the clone」、`MODATA_FLAGS`(`MOGENFLAG_CLONE_ON`「Particle is visible」、`DISABLE`、`BORN`、`MODATASET`、`COLORSET`、`TIMESET`)、`MODATA_WEIGHT`「Weight of the clone」、`MODATA_CLONE`「Clone Offset (picks which child of the Cloner gets cloned or the blending between those children)」、`MODATA_TIME`「Time offset of the clone」、`MODATA_LASTMAT`、`MODATA_STARTMAT`、`MODATA_ALT_INDEX`、`MODATA_FALLOFF_WGT`「Falloff weight」、`MODATA_SPLINE_SEGMENT`、`MODATA_GROWTH` | Python SDK consts/MODATA |
| 任意配列の追加 | `MoData.AddArray(id, name, default_flags)`「Add the specified array」、`GetArray/SetArray(id, array, apply_strength)`、`GetCount`「Get the length of the arrays」 | Python SDK MoData |
| Effector Parameter tab — Transform | Position / Scale(Uniform Scale・Absolute Scale)/ Rotation(`P [XYZ m]` `S [XYZ]` `R [HPB °]`)。**Transform Mode**: 「Relative mode adds the effector value to existing clone settings」、「Remap applies the effector value starting from 0」、「Absolute is good for use in conjunction with position transformations since the clones will orient themselves to a P.X, P.Y or P.Z point」。Transform Space: Node / Effector / Object | help.maxon.net r21 Formula / s22 Random Effector: Parameters |
| Parameter tab — Color | Color Mode: Off / Effector Color / Fields Color(推奨)/ Custom Color。**Blending Mode**: 「Mix: An average color between that of the Effector or Field color and of the Cloner Object will be interpolated」、Add、Subtract、Multiply、「Divide: The Cloner's color will be divided component-wise through the Effector color」。Use Alpha/Strength | 同上 |
| Parameter tab — Other | Weight Transform「assigns each clone a Weight of between 0% and 100%」、U Transform / V Transform(内部 UV)、Modify Clone「affect a clone's child objects」、**Time Offset**「generate clones from that object at different intervals」、Visibility「blend clones from view at a defined value」 | 同上 |
| Cloner Transform tab(初期値の側) | Display: None / Weight(red→yellow)/ UV / Color / Index。P・S・R、Color(MoGraph color shader が初期色)、Weight [0..+∞%]「initial weighting for each clone」、**Time** + Animation Mode: Play / Loop / Fixed「Clones will assume the state that exists at the location in the Time setting」/ Fixed Loop | help.maxon.net s22 Cloner: Transform Tab |
| 複数 Effector の順と合成 | Effectors tab: 「Place all Effectors into this field that should affect the Cloner」「The sequence of Effectors can be modified simply by dragging them to their desired positions」「Each Effector has a corresponding slider below the Effectors field, with which the Effector's strength can be adjusted」。Delay Effector: 「should be arranged after the Effectors it should affect, i.e., it should be arranged after (below) these Effectors in the MoGraph object's Effector list」 | help.maxon.net s22 Cloner: Effectors Tab · Delay |
| Effector tab(共通) | Strength「Values of less than 0% and greater than 100% can be entered」、Selection(MoGraph Selection tag = 選ばれた clone だけ、Weightmap tag = 「multiplied with the effector strength」)、Min/Max、Falloff は Fields(R20〜) | help.maxon.net s22 Plain: Effector |
| 計算の言い方 | 「calculating a final position for each clone」 by multiplying 「strength, min/max values, falloff, and with the values set in the Parameter tab」。「All Effectors can also be used as deformation objects for normal polygon objects」 | help.maxon.net s22 Effectors(7443) |

## 3. Maya MASH

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| Python node が読む物 | 「reads MASH network data into a class called 'md'」。読める: position, scale, rotation, id, visibility, angular velocity, vector velocity(`md.position[5].x`)。「The ID and Visibility arrays are stored as doubles」 | Autodesk Maya 2023 MotionGraphics: Python |
| 書く物 | `outPosition, outScale, outRotation, outId, outVisibility, angularVelocity, velocityVec, angularVelocityVec`(`outPosition[5].y = 5`)、`setData()`「Set the data to the out attributes; required」 | 同上 · openMASH |
| 任意 channel | `getNamedArray(channelName, typeName)`、`getNamedArrays()`「Returns all the array names in the dynamic array data」、`getVectorArray/getDoubleArray/getIntArray(attributeName)` と `set*Array`、`getMatrix(pointId)/setMatrix`、`getColorSet(setName)`、`setPointCount(count)`、`getFalloff(index)`「Returns a doubleArray of strengths from a falloff object」、`getFrame()` | openMASH(MASH Technical Documentation 4.5) |
| Offset node の合成 mode | 「Offset: Adds to the Offsets to the input array」「Multiply: Multiplies the Offsets to the input array」「Overwrite: Values will be set to these regardless of incoming values」「Multiply by Time: Multiplies the offset values by the frame number and adds the result to the input array」。Offset Position / Rotation / Scale、Space: World / Local、Strength「Fades the node's effect for all the objects at the same time」+ Random Strength / Step Strength | Autodesk Maya 2023: Offset |
| Color node | 「customize the display of a MASH network's CPV data」。Color Mode: Normal「Current CPV data is overwritten」/ Add / Subtract / Multiply / Screen / Overlay。Random Hue/Saturation/Value、Strength Map | Autodesk Maya 2022: Color |
| 時間 | Time node「Offsets component animation」、Delay node「Offsets an object's existing animation in time」 | MASH Nodes Overview |
| node 一覧(何を書くか) | Distribute(位置)、Offset(transforms)、Random、Signal(4D Noise / Trigonometric)、ID「Customizes the way that instanced objects are assigned to MASH points」、Visibility、Color、Orient、Strength「Controls the degree of connected nodes effect on the network」、Falloff は各 node の Strength Map | 同上 |
| 順序 | 「each node performs a specific function and can be combined with other nodes to compound their behaviors」。評価順の明文は取れず(§9) | 同上 |

## 4. Cavalry

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| Duplicator が子に配る属性 | Shape Position「Set positions on a per duplicate basis」、Shape Rotation、Shape Scale、Shape Visibility、Shape Opacity、Shape Id「Set the Id of the Input Shape(s) to be distributed」、Auto Id、**Shape Time Offset**「Offset the animation curve for duplicates on a per shape basis」、Use Index Context / Index Context「output the index for each duplicate at the relevant 'upstream depth'」、Skip Invisible Duplicates。**色の欄は無い** | cavalry.studio/docs Duplicator |
| Index(文脈) | 「A fundamental part of a Duplicator's role is to assign indices to the Shapes it generates. This index can be used by the Duplicator and other layers to dictate position, rotation, color, number of sides...anything really」「During those 10 times, Random receives Index values of: 0, 1, …, 9」 | Key Concepts: Context |
| 色 | Index to Color「Map a gradient to each index to set the colors of Shapes within a sub-mesh」→ Shape の fill color に繋ぐ。Color Array「Create a list of colors that can be assigned to other Shapes via their indices」(`colorArray.id → basicShape.color`、`random.id → colorArray.index`) | Index to Color · Color Array |
| Stagger | 「Generate sequential values between a minimum and maximum」、Minimum / Maximum / Offset / Graph(X 軸 = 最初〜最後の Id、Y = Min〜Max)。Shape Position Y と Shape Time Offset に繋ぐ例。「Stagger Value + Current Frame = Shape Time Offset」 | Stagger |
| Falloff | 「Falloffs can be used to isolate the strength of most Behaviours and Fields. By default, a Falloff outputs a value of 1 at its centre which falls off to 0 at its edges」、出力は Strength の乗数。Shape Type: Circle / Rectangle / Linear / Sweep / Shape、Probability(0/1 化)。複数 Falloff の合成の明文は無し | Falloff |
| 接続の型と本数 | 「Connections can be made between attributes with compatible data types. For example, a string (text) cannot be connected to a value (number)」「An attribute can have several outputs but can only have one input」「An animation curve (keyframes) is considered an input」 | Key Concepts: Connections |
| 属性の型 | int / double / int2 / double2 / double3 / Color(R,G,B,A,hex)/ Slider / String / Graph / Input List / Generator / Checkbox(bool)。Dynamic Attribute は「Add Point Data」で追加 | Attribute Editor: Control Rows – Types |
| 複数 Behaviour の合成 | Behaviour Mixer: 「Each index affects the result of the indices below」、Blend: Normal(Add)/ Min / Max / Minus / Multiply「decreasing values」/ Screen「Multiply the inverse of values, increasing values」/ Overlay、index ごとに Strength と Enabled。「The Behaviour Mixer does not support Deformers」 | Behaviour Mixer |
| Behaviour と Deformer | 「Behaviours can be thought of as effects. They can be used to animate or deform other Layers」。Deformer は Shape の `deformers` 属性に繋ぐ(`javaScriptDeformer.id → shape.deformers`)、「'per-point' manipulation」。JavaScript Deformer: `def.getPoints()`「Return an array of positions for each point of a Shape」/ `def.setPoints()`「Set the position for each point」— 点の位置だけ | Behaviours 一覧 · Deformer Module · JavaScript Deformer |

## 5. Houdini(point attribute の名前規約 + VEX wrangle)

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| 属性とは | 「Attributes are named values stored on vertices, points, primitives, and objects」「dynamics solvers and rendering engines will often use certain attributes on the geometry if they exist」「You can also set your own custom attributes to be used by node expressions, scripts, exporters, etc.」 | sidefx.com model/attributes |
| 標準名(誰が読むか) | `P`「Point position. The viewport uses this…」、`N`「Normal direction」、`Cd`「Diffuse color override. The viewport uses this to color OpenGL geometry」、`Alpha`「Alpha transparency override」、`pscale`「Uniform scaling factor… controls the size of the particle/point」、`scale`(非一様)、`orient`「Quaternion orientation… used for instancing geometry onto a point」、`rot`「additional offset-quaternion applied after all other attributes」、`up`、`id`「A unique element ID… keep track of them even if the point numbers change」、`v`「Velocity. The renderer uses this attribute to know where to add motion blur」、`w`(角速度)、`uv`、`instance`(path)、`pivot`、`trans`、`transform`、`name` | 同上 · copy/instanceattrs |
| 下流の読み方(Copy to Points) | transform あり: `X * M * T`、orient あり: `X * S * (O * R) * T`、それ以外: `X * S * L * R * T`(X = pivot、S = scale×pscale、L = N/v + up の整列、O = orient、R = rot、T = trans/P) | copy/instanceattrs |
| wrangle の書き方 | 「you can read/write the value of an attribute using `@‹attribute_name›`」「If you write to a `@attribute` in the VEX code and the attribute does not exist, Houdini will create it」。型接頭: `f@ v@ u@ p@ i@ s@ d@ 4@`。既知名は型が暗黙(`@P @Cd @N @scale @v` = vector、`@id @ptnum @numpt` = int、`@name @instance` = string)、「Houdini assumes all other `@` references are float unless you manually specify a different type」 | vex/snippets |
| 番号と時刻 | `@ptnum`「The point number of the current point」、`@numpt`「The total number of points」、`@Time`「Float time ($T)」、`@Frame`、`@TimeInc`「Float time step (1/$FPS)」。時間のずれの属性は無い(読むだけ) | 同上 |
| 加算/上書き | wrangle は代入(上書き)。`@P += …` と書けば加算、node graph の順が合成順。「Passing information down the network on attributes is inherently friendlier to parallel processing than using external references on later nodes to data on earlier nodes」 | 同上 |

## 6. ISF(Interactive Shader Format)

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| 定義 | 「fragment shaders which can be used to generate or process pixel information」「Currently the ISF Specification is used to describe generators, filters and transitions for 2D images」 | docs.isf.video primer ch.1 |
| INPUTS の TYPE | `"event", "bool", "long", "float", "point2D", "color", "image", "audio", "audioFFT"`(long は pop-up menu、color は float 配列で min/max/default、image は DEFAULT/MIN/MAX 無し、audio/audioFFT は image texture で渡る) | ref_json |
| 自動 uniform | `TIME`(秒)、`TIMEDELTA`、`RENDERSIZE`(vec2)、`FRAMEINDEX`(int)、`DATE`(vec4)、`PASSINDEX`、`isf_FragNormCoord` | ref_variables |
| 出力 | `gl_FragColor`(画素)のみ。PASSES は `TARGET`/`PERSISTENT`/`FLOAT`/`WIDTH`/`HEIGHT` の buffer。物の属性を書く口は仕様に無い | ref_json |

## 7. Processing / p5.js

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| Processing `draw()` | 「Called directly after setup(), the draw() function continuously executes the lines of code contained inside its block until the program is stopped or noLoop() is called」「All Processing programs update the screen at the end of draw(), never earlier」 | processing.org/reference/draw_ |
| p5.js `draw()` | 「A function that's called repeatedly while the sketch runs」「by default, draw() tries to run 60 times per second」 | p5js.org/reference/p5/draw |
| 属性 | 無い。物の位置・色・大きさは draw の中で `fill()/translate()/rect()` 等の命令として毎コマ書く(retained な物のリストは言語側に無い) | 同上 |

## 8. GSAP stagger / CSS custom property

| 項目 | 内容(原文) | 出所 |
|---|---|---|
| GSAP function-based values | 「Get incredibly dynamic animations by using a function for any value, and it will get called once for each target the first time the tween renders」引数 `(index, target, targets)`。例 `y: function(index, target, targets) { return index * 50; }` — **どの property でも** index の関数にできる | gsap.com/docs/v3/GSAP/gsap.to() |
| GSAP stagger | 「A value of `stagger: 0.1` would cause there to be 0.1 second between the start times of each tween」。負値で逆順。object 形: `each / amount / from("center","edges","random",index,grid 座標) / grid("auto") / axis / ease / repeat`、関数形(index, target, list)で任意の delay | gsap.com/docs/v3/Staggers |
| CSS custom property | `--x` で宣言、`var()` で読む、`calc(var(--base-width) * 2)` のように calc 内で使える、親から継承(`@property` で `inherits: false` にできる)、JS から `element.style.setProperty("--my-var", value)`。「`--i` を index として inline に置き `calc()` で値を作る」idiom の一次資料(spec/MDN)は無し — MDN が示すのは上記の部品だけ | developer.mozilla.org Using CSS custom properties |

## 9. 軸 (a)〜(f) の横比較

| 道具 | (a) 書ける属性の集合と名前 | (b) 加算/上書き・複数の法の合成 | (c) time offset を属性として持つか | (d) 色 | (e) 拡張が属性を足せるか | (f) 下流の読み方 |
|---|---|---|---|---|---|---|
| AviUtl anm/obj | 固定: `ox oy oz rx ry rz cx cy cz zoom alpha aspect`(2: `sx sy sz`)。`obj.index / obj.num / obj.time / obj.totaltime / obj.frame` は読む | 代入 = 上書き(`obj.ox = obj.ox + …` は作者が書く)。複数スクリプトは object に掛けた順(効果の並び)。`obj.getvalue` は「後から基本効果等で変更しても反映されない」= 元の値 | 2 の `obj.multiobject(num, func)` の戻り値が秒の time offset。1.x は `obj.time` を読んで自前 | 無い(`obj.alpha` のみ。色は `obj.effect` の filter か `obj.setoption("blend")`) | 不可(obj table は固定、`setoption` の名前も固定) | 拡張編集が obj.* を読んで描く(`obj.draw` は obj.* に対する相対) |
| C4D MoGraph | MoData 配列: MATRIX / COLOR / SIZE / UVW / FLAGS(visible) / WEIGHT / CLONE / TIME / FALLOFF_WGT / ALT_INDEX / LASTMAT / STARTMAT | Effector ごとに Transform Mode = Relative(足す)/Absolute/Remap、色は Blending Mode(Mix/Add/Sub/Mul/Div)。Effectors tab の一覧を上から下、drag で順変更、各 Effector に strength slider。Delay は「効かせたい物の下」 | `MODATA_TIME`「Time offset of the clone」。Cloner の Time + Animation Mode(Play/Loop/Fixed/Fixed Loop)、Effector の Time Offset | `MODATA_COLOR`、Color Mode Off/Effector/Fields/Custom | `MoData.AddArray(id, name)` で任意配列 | Cloner が MoData を読んで clone を置く、MoGraph color shader が色を読む、Deformation mode で polygon にも |
| Maya MASH | position / rotation / scale / id / visibility / velocity / angularVelocity(+ CPV 色、matrix、named array)。id と visibility は double | node ごとの mode: Offset(Add)/Multiply/Overwrite/Multiply by Time、Color: Normal(上書き)/Add/Sub/Mul/Screen/Overlay。各 node に Strength(+Random/Step、Strength Map = falloff)。積んだ順(明文は取れず) | Time node(component animation を offset)、Delay node。点の属性としては明文無し | CPV 経由の Color node(Repro の Color per vertex output を on) | Python node の `getNamedArray/setVectorArray` 等で「dynamic array data」に任意 channel | Repro / Instancer が out 配列を読む |
| Cavalry | Duplicator の per-duplicate 欄: Position / Rotation / Scale / Visibility / Opacity / Id / **Time Offset**。Index Context が index を下流に流す | 1 属性に入力は 1 本(keyframe も入力)。複数は Behaviour Mixer(Normal(Add)/Min/Max/Minus/Multiply/Screen/Overlay、index の上が下に効く、Strength と Enabled)。Falloff = Strength の乗数 | `Shape Time Offset`、Stagger で `Stagger Value + Current Frame` | Duplicator に色欄は無く、Index to Color / Color Array を Shape の fill に(index 文脈で per duplicate) | Dynamic Attribute(Add Point Data)、JavaScript Layer。型は int/double/int2/double2/double3/Color/String/bool | Shape が自分の属性を評価する時に index 文脈で上流が再評価される(「Random receives Index values of 0..9」) |
| Houdini | 名前規約: `P N Cd Alpha pscale scale orient rot up id v w uv pivot trans transform instance name`。未知名は書けば float で作られる | wrangle は代入。node graph の順が合成順(前の node の属性を次が読む) | 無い(`@Time @Frame @TimeInc` は読むだけ) | `Cd`(viewport が読む)/ `Alpha`、renderer も名前で読む | 可(`f@myattr = 12.5` で新属性、型接頭で型指定) | Copy to Points が `X*S*(O*R)*T` 等の決まった式で `pscale/scale/orient/rot/up/N/v/pivot/trans/transform` を読む、renderer が `v` で motion blur |
| ISF | 無い。INPUTS は event/bool/long/float/point2D/color/image/audio/audioFFT の**入力**型 | 画素の合成のみ(PASSES で buffer) | `TIME/TIMEDELTA/FRAMEINDEX` は読むだけ | 画素そのもの | 不可(出力は gl_FragColor) | 画素 |
| Processing / p5 | 無い。draw が毎コマ全部を命令で書く | 命令の順 | 無い | 命令(`fill()`) | 言語の変数で何でも | 無い(即時描画) |
| GSAP / CSS | GSAP: 任意 property が `(index, target, targets)` の関数。CSS: custom property を `calc()` で | GSAP: 同じ property への複数 tween は後勝ち(overwrite 設定)— 本調査では一次資料未取得。CSS: cascade | GSAP stagger = start time の per-target offset(`each/amount/from/grid/ease`) | 可(color も property) | 可(任意 property 名) | tween が DOM/CSS を書く |

## 10. 見つからなかった物・取れなかった物

- AviUtl 1.x の原本 `lua.txt`(exedit92.zip 同梱)の本文そのもの。scrapbox の写しは項目名と `getvalue/setoption/getoption` の一覧まで、`obj.index/obj.num/個別オブジェクト` の個別ページは無し(404)。AviUtl2 版は docs.aviutl2.jp(非公式ミラーと自称)のみ。ocraviutl.fandom は 402 で不可。
- AviUtl 1.x で「個別オブジェクト」on/off で **何が変わるか**の原文(off 時に `obj.num` が 1 になる等は AviUtl2 版の「1=単体オブジェクト」からの類推)。
- C4D: 複数 Effector が同じ属性に書いた時の**数式**(順に Relative なら累積する、Absolute が来たら置き換わる、の明文)。取れたのは「順は一覧の上から下」「Delay は下に」「strength・min/max・falloff・Parameter tab の値を掛ける」まで。
- MASH: node の評価順の明文(Waiter に足した順という記述)。点の time offset 属性の名前。
- Cavalry: 複数 Falloff の合成則、Duplicator の Shape Time Offset の**単位**(frame と読めるが明文なし)、Deformer の一般定義文(Behaviours 一覧に混ざる)。
- Houdini: 「時間のずれ」を点の属性として持つ標準名(無い、が「無い」の明文も無い)。
- GSAP: 同じ property を複数 tween が書いた時の overwrite 規則(`overwrite: "auto"` 等)は今回未取得。CSS `--i` idiom の一次資料は無し(MDN は部品のみ)。

## Sources(取得 2026-09-18、一次のみ)

- AviUtl: https://scrapbox.io/aviutl/obj(+ `/api/pages/aviutl/obj.setoption|obj.getvalue|obj.getoption|obj.aspect|obj.time/text`)· https://docs.aviutl2.jp/lua/ · https://docs.aviutl2.jp/lua/examples · https://spring-fragrance.mints.ne.jp/aviutl/
- C4D: https://developers.maxon.net/docs/py/2026_1_0/consts/MODATA.html · https://developers.maxon.net/docs/py/2024_4_0a/modules/c4d.modules/mograph/MoData/index.html · https://help.maxon.net/c4d/r21/us/html/OEFORMULA-ID_MG_BASEEFFECTOR_GROUPPARAMETER.html · https://help.maxon.net/c4d/s22/us/html/OERANDOMIZE-ID_MG_BASEEFFECTOR_GROUPPARAMETER.html · https://help.maxon.net/c4d/s22/us/html/OMOGRAPH_CLONER-ID_MG_TRANSFORM_GROUPTRANSFORM.html · https://help.maxon.net/c4d/s22/us/html/OMOGRAPH_CLONER-ID_MG_MOTIONGENERATOR_GROUP_EFFECTORS.html · https://help.maxon.net/c4d/s22/us/html/OEPLAIN-ID_MG_BASEEFFECTOR_GROUPEFFECTOR.html · https://help.maxon.net/c4d/s22/us/html/OEDELAY.html · https://help.maxon.net/c4d/s22/us/html/7443.html
- MASH: https://help.autodesk.com/cloudhelp/2023/ENU/Maya-MotionGraphics/files/GUID-DCB144BD-DFE2-418B-9838-E06963F61A20.htm · https://help.autodesk.com/cloudhelp/2018/ENU/Maya-Tech-Docs/MASH/openMASH.html · https://help.autodesk.com/cloudhelp/2023/ENU/Maya-MotionGraphics/files/GUID-F54566EF-496A-4311-B49F-8D8A61295929.htm · https://help.autodesk.com/cloudhelp/2022/ENU/Maya-MotionGraphics/files/GUID-6322679A-61CD-4747-8FB0-0EBDE787F1EA.htm · https://help.autodesk.com/cloudhelp/2020/ENU/Maya-MotionGraphics/files/GUID-D4FECFDC-F91A-4BDC-A1B0-A24EB087B2DD.htm
- Cavalry(docs.cavalry.scenegroup.co は cavalry.studio/docs へ 301): https://cavalry.studio/docs/nodes/shapes/duplicator/ · https://cavalry.studio/docs/nodes/behaviours/stagger/ · https://cavalry.studio/docs/nodes/utilities/falloff/ · https://cavalry.studio/docs/nodes/behaviours/ · https://cavalry.studio/docs/nodes/behaviours/behaviour-mixer/ · https://cavalry.studio/docs/nodes/behaviours/javascript-deformer/ · https://cavalry.studio/docs/tech-info/scripting/deformer-module/ · https://cavalry.studio/docs/getting-started/key-concepts/context/ · https://cavalry.studio/docs/getting-started/key-concepts/connections/ · https://cavalry.studio/docs/nodes/utilities/index-to-color/ · https://cavalry.studio/docs/nodes/utilities/color-array/ · https://cavalry.studio/docs/user-interface/menus/window-menu/attribute-editor/control-rows/control-rows-types/
- Houdini: https://www.sidefx.com/docs/houdini/model/attributes.html · https://www.sidefx.com/docs/houdini/vex/snippets.html · https://www.sidefx.com/docs/houdini/copy/instanceattrs.html
- ISF: https://docs.isf.video/ref_json.html · https://docs.isf.video/ref_variables.html · https://docs.isf.video/primer_chapter_1.html
- Processing / p5: https://processing.org/reference/draw_.html · https://p5js.org/reference/p5/draw/
- GSAP / CSS: https://gsap.com/docs/v3/GSAP/gsap.to() · https://gsap.com/docs/v3/Staggers/ · https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties
