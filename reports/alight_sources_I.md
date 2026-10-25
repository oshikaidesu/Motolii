# I. アプリだけにある効果 16 の借りる先（10/25）

[alight_checklist.md](alight_checklist.md) の I 段。公式ガイド（guide.alightmotion.com/effects、169 本）にも公式ヘルプ（support.alightmotion.com、72 本の記事）にも説明が無い 16 本。

## 数

| | 数 | どれ |
|---|---|---|
| そのまま置ける（借りた .fs と、Motolii にもう在る部品の組み合わせだけ。新しく書かない） | **6** | Advanced Chroma Key、Energy Beam、Key Spill Cleaner（近い物）、Luma Stamper、Page Curl（一部）、Raster Transform |
| エンジンに手が要る（借りる先はある。読み込む口・渡す口が足りない） | **7** | Chromatic Zoom Blur、Color Balance、Grunge、Parenting Helper、Ring、Scatter Field、Streak Strips |
| 借りる先が見つからない | **3** | Matte Fringe、Scale Assist、Warp Blur |

「そのまま置ける」6 本のうち 5 本は、要る .fs がもう `filament/effects/` に入っている。足すのは `Layer Position.fs` の 1 本だけ（Raster Transform 用）。

## 何をする物か、どう調べたか

- **Alight Motion 自身の説明文とつまみ**：エミュレーターに入っている Alight Motion 6.2.60 の APK を adb で取り出して読んだ。画面には触っていない（Ease の編集の途中だったので、そのままにした）。効果は 1 本ずつ `assets/effects/<名前>.xml`（名前・説明・分類・つまみ・シェーダー）で、説明の文は `resources.arsc` の `effect_*_desc`。下の説明はこの文を短く訳した物。つまみの名前と既定値もここから取った。
- **Alight Motion のシェーダーは借りない**（独自の物）。読んだのは何をするか・つまみの名前だけ。
- **入った版**：Raster Transform・Scale Assist・Parenting Helper は 4.0.2、Streak Strips・Page Curl・Luma Stamper は 4.1.0、Advanced Chroma Key・Key Spill Cleaner・Matte Fringe・Energy Beam・Grunge は 4.5.0（2023 年 3 月。公式の X では Energy Beam を「Laser Beam」と呼んでいる）、Chromatic Zoom Blur・Color Balance・Warp Blur は 5.0.0。どれも APKMirror の更新履歴を検索で見ただけで、元のページは開けていない。Ring と Scatter Field の版は分からない。
- **公式ヘルプで説明があるのは Parenting Helper だけ**（「Layer Parenting and Null Objects」の記事。観覧車のゴンドラ、車輪の例）。

## 借りない物

- OBS の chroma key・color key フィルター：GPL-2.0。
- Natron と openfx-misc の Keyer・PIK・Despill：GPL-2.0。
- LYGIA：Prosperity Public License 3.0.0。商用に使えるのは 30 日の試用だけなので使わない。
- GSAP：使わない。
- Alight Motion の effects/*.xml の中にあるシェーダーと JS：独自の物。
- 参考にだけ使える物：FFmpeg の `chromakey`・`hsvkey`・`despill`・`colorbalance`、GStreamer の `alpha`（どちらも LGPL）。CPU で動き、物ごとの GPU の効果にはならない。動画の物だけなら GStreamer の `alpha` をそのまま流れにつなげる。

## 1 本ずつ

- **Advanced Chroma Key**（分類 matte）：最大 4 色を、それぞれ「抜く（Exclude）」か「残す（Include）」にして抜く。色ごとに Threshold 0.10、Feather 0.05、Luma Sensitivity 0.25 があり、Show Mask で抜け方を白黒で見られる。普通の Chroma Key より細かく決められる。 — 借りる先：ISF `Chroma Mask.fs`（VIDVOX、ISF-Files は MIT、**もう `filament/effects/` に在る**）。HSV ごとの許す幅、dilate、blur、showAlpha が付いている。alphaMode の Multiply は、中の注記どおり「Chroma Mask を何枚も重ねる時」用。動画の物だけなら GStreamer `alpha`（LGPL、method=custom、target-r/g/b、angle、noise-level、black/white-sensitivity）も使える。 — 入れ方：物の fx に `Chroma Mask.fs` を色の数だけ並べ、2 枚目からは alphaMode=Multiply にする。Show Mask は showAlpha。**Include（その色だけ残す）は Chroma Mask に無い**ので、ここだけ足りない。

- **Chromatic Zoom Blur**（blur、会員）：中心から放射状にぼかし、赤・緑・青でぼかしの強さを変える。つまみは Center、Strength 0.15、Adaptive。 — 借りる先：glfx.js の `src/filters/blur/zoomblur.js`（MIT、Evan Wallace。centerX・centerY・strength、41 回読む放射のぼかし）と、色ごとにずらす所は ISF `Chroma Zoom.fs`（toneburst、ISF-Files MIT、まだ入れていない。red/green/blue_zoom、center）。 — 入れ方：glfx.js はシェーダーを `new gl.Shader(null, '…')` に文字で渡す作りなので、three_isf.cpp と同じように QuickJS で読み、代わりの小さな `gl` を渡して文字だけ取り出せば ISF の道に乗る（ここがエンジンの手）。まず `Chroma Zoom.fs` → zoomBlur の順に並べる。色ごとに強さを変えるのは近い形になるだけ。

- **Color Balance**（color）：影・中間・明るい所の 3 つに、それぞれ C/R・M/G・Y/B の 3 本（−100〜100）と Preserve Luminosity。Photoshop の「カラーバランス」と同じ形。 — 借りる先：android-gpuimage の `GPUImageColorBalanceFilter.java`（CyberAgent、Apache-2.0）。つまみが shadowsShift・midtonesShift・highlightsShift（vec3）と preserveLuminosity で、形がそのまま同じ。参考に FFmpeg `colorbalance`（LGPL、rs…bh、pl）。 — 入れ方：シェーダーが Java の文字の定数なので、その文字を取り出して ISF の頭（INPUTS 4 つ）を付ける口が要る（three_isf.cpp と同じ種類の手）。

- **Energy Beam**（procedural、会員）：2 点の間に光の線を描く。つまみは Start、End、Color、Glow と Glow Color、Halo（強さ・広がり・色）、Length、Progress、Blur、重ね方（Normal・Multiply・Screen）、Flicker（強さ・速さ）、Variation（強さ・速さ・位相・乱れ・Evolution）。 — 借りる先：Motolii にもう在る部品だけで組める。形の line、stroke、glow（glow_blur、glow_opacity）、trim と trim_offset（ThorVG の trimpath。Length と Progress にあたる）、形の wiggle（wiggle_detail、wiggle_speed。Variation にあたる）、振る舞いの noise を不透明度に（Flicker）、Halo は ISF `Glow.fs`（MIT、在る）。稲妻のように枝分かれさせたい時は three.js の `examples/jsm/geometries/LightningStrike.js`（MIT）があるが、three の本体が要るので今は使えない。 — 入れ方：recipes/ にこの組み合わせを 1 本置く（書くのは「make shape line」と「set」の行だけ）。

- **Grunge**（procedural）：汚れ・しみ・粒を最大 5 枚重ね、古びた・すり減った見た目にする。1 枚ごとに、出る場所（Edge・Center・Fill・Glow）、Grain（16 種）、Scale、Angle、Offset、Gamma、Contrast、Alpha、Color、Dodge/Burn、Spread、Choke、Invert、光の向きで陰を付ける、重ね方（Normal・Multiply・Screen・Punch Out・Copy Original）がある。Alight Motion は中に 2 枚の汚れの絵（grunge_a・grunge_b）を持っている。 — 借りる先：汚れの絵は ambientCG の SurfaceImperfections・Fingerprints・Smear の類（CC0）。粒を数で作るなら ISF `Noise.fs` と `Simplex Noise.fs`（MIT、在る）。縁の帯は ISF `Erode.fs` と `Edges.fs`（MIT、在る）。 — 入れ方：Fill は、汚れの絵の物を clip_matte alpha でその物に切り、blend で重ねれば今でもできる。**Edge・Center（縁から内へ何 px）と、1 つの札に 5 枚を重ねる所はエンジンの手**（縁からの距離の絵を効果に渡す口）。

- **Key Spill Cleaner**（matte）：抜いた後に縁に残る緑や青をきれいにする。縁の内側の細い帯の中で、内側の色相と彩度を外へ広げ、明るさはそのまま残す。つまみは Strength 0.02 の 1 本だけ。 — 借りる先：ISF `Chroma Desaturation Mask.fs`（VIDVOX、MIT、**在る**）。キーの色に近い所の彩度だけを落とすので、緑かぶりが消える。動画の物なら、今の keyed も flat.mat で緑を他の 2 色の大きい方までに抑えている。参考に FFmpeg `despill`（LGPL）。 — 入れ方：物の fx に `Chroma Desaturation Mask.fs` を置く。**近い物**：彩度を落とすだけで、内側の色を広げはしない。内側の色を外へ広げる見本は、許されたライセンスの物では見つからない（Nuke の IBKColour の考え方。Natron・openfx-misc は GPL）。

- **Luma Stamper**（matte）：明るさから型を取り、ごく暗い所・ごく明るい所の粒は落とし、その型で無地の色を押す。つまみは Color、Clip Low、Clip High、Gamma、Invert。 — 借りる先：Motolii の clip_matte luma（AE のトラックマットの作法、在る）と、明るさの両端を切るのに ISF `Color Levels.fs`（MIT、在る。min・mid・max が Clip Low・Gamma・Clip High にあたる）。Invert は clip_invert。1 つの .fs にまとめるなら three.js `examples/jsm/shaders/LuminosityHighPassShader.js`（MIT。luminosityThreshold・smoothWidth・defaultColor）も、three.js の効果ファイルとしてそのまま置ける形。 — 入れ方：その物の上に無地（solid）の物を置き、clip_matte luma でその物に切る。元の物には `Color Levels.fs` を掛ける。

- **Matte Fringe**（matte）：縁の内側に細いぼかした帯を取り、その帯の色相と彩度を「後ろにある物」の色に替え、明るさは残す。抜いた縁に残る変な色を、背景になじませる。つまみは Edge Size 5、Feather 0.5、除く色 3 つ（色と幅）。 — 借りる先：**見つからない**。後ろの色を縁に入れる見本で、許されたライセンスの物が無い（AE の Refine・Light Wrap の類。Natron は GPL）。部品ならある：縁の帯は ISF `Erode.fs`（在る）、色相と彩度だけ下から取るのは Motolii の blend の color。 — 入れ方（もし作るなら）：物ごとの効果に「この物の後ろの絵」を渡す口が要る。Alight Motion は `comp` という名前で後ろの絵を受けている。

- **Page Curl**（3d）：紙のように角をめくる。つまみは Progress 0.4、Radius 0.1、Angle 38°、Alpha、Blur、Back of Layer（色・重ね方）、Shine（色・Specularity）、Drop Shadow、Front Shadow。 — 借りる先：ISF `Inverted Page Curl.fs`（gl-transitions の InvertedPageCurl、Hewlett-Packard の BSD-3-Clause、**在る**）。 — 入れ方：トランジションの形（startImage→endImage、progress）なので、startImage にその物、endImage に透明な物を渡す。角度は前後に ISF `Rotate.fs`（在る）を挟む。**一部**：Radius・裏の色・光り・影のつまみは無い。

- **Parenting Helper**（分類なし。transform・parenting）：親が回ったり大きさを変えたりした時に、子がどうついて行くかを変える。Scale と Rotation のそれぞれを Normal（普通）・Locked（位置だけついて行き、向きと大きさは変えない）・Weighted（倍率をかけてついて行く。200% で 2 倍回る）から選ぶ。Auto Rotate（X 軸か Y 軸）は親の横か縦の動きから自動で回す（車輪）。Radius Adjust で車輪の半径を直す。 — 借りる先：Rive の C++ ランタイム `src/constraints/rotation_constraint.cpp`・`scale_constraint.cpp`（rive-app/rive-runtime、MIT）。doesCopy を外すと自分の向きを保つので Locked、copyFactor が Weighted の倍率にあたる。Locked だけなら Godot の RemoteTransform2D/3D（MIT）の update_rotation・update_scale の考え方も同じ。 — 入れ方：今の Motolii の parent は、親の置き方（doc_place の行列）を丸ごと掛ける（doc_relations.h の doc_above）。ここで親の行列を移動・回転・大きさに分け、回転と大きさにだけ倍率をかけてから掛け直す。これがエンジンの手。**Auto Rotate は見本が見つからない**。式は Alight Motion の説明そのまま（進んだ距離 ÷ 半径 で回す）。

- **Raster Transform**（transform）：Move & Transform と同じく大きさ・回転・ずらしを変える。ただし絵になった後、前の効果を掛けた後に動かすので、効果の結果ごと動かせる。つまみは Scale、Angle、Offset、Mask To Layer、Alpha、Fill、Sampling（Nearest・Linear）。 — 借りる先：ISF `Zoom.fs` と `Rotate.fs`（VIDVOX、MIT、**在る**）、ずらしは ISF `Layer Position.fs`（VIDVOX、MIT、**まだ入れていない**。offset、repeatImage）。 — 入れ方：`Layer Position.fs` を `filament/effects/` に足す。物の fx の中で、前の効果の後ろに Zoom → Rotate → Layer Position の順に並べる（Motolii の物ごとの効果は、並べた順に掛かる）。Mask To Layer・Fill・Nearest は無い。

- **Ring**（3d）：平らな面でできた立体の輪を描き、その物の絵を面に貼る。つまみは向き、回転、Radius、Width、Length、Smoothing、位置、Scale、光（Flat・点光源・平行光・Depth Map・Normal Map など）、貼り方（Mirror、Stretch、Repeat X/Y、Angle、Crop、Twirl、Offset）。 — 借りる先：輪の形は three.js の `LatheGeometry`（四角を回せば平らな面の輪）か、`CylinderGeometry` の openEnded（MIT）。Motolii には三角形の torus（kit_stage.h の torus()）と、形の ring がもう在る。 — 入れ方：A 段の「絵を立体の面に貼る」（Box・Cube・Cylinder・Torus など faux-3D の 16 本）と同じ手が要る。その物の画面外の絵（thing_fx が作る）を、Filament の 3D の物の材質に色の絵として渡す。

- **Scale Assist**（transform・parenting）：Move & Transform の大きさの上に、もう 1 段大きさを掛ける。Axis（X・Y・X,Y・X,−Y）、Scale、Damping。X,−Y は「横に伸びた分だけ縦が縮む」で、押しつぶしと伸び（squash & stretch）になる。 — 借りる先：**見つからない**。大きさに倍率を掛けるだけの物で、許されたライセンスの見本は無い（Alight Motion の中の JS は独自の物）。近い物は Motolii の形の squash（形だけ、-0.9〜2）。 — 入れ方（もし作るなら）：scale_x・scale_y に掛ける倍率の値を足す。振る舞い（stagger・oscillator・noise・random）につなげば、弾む動きもできる。

- **Scatter Field**（repeat）：その物の写しを、果てのない 3D の空間にばらまく（元は隠す）。キーで空間の中を進めるので、星の中を飛ぶような絵になる。つまみは Position（xyz）、Density、Evolution、Seed、Scale、Size、Angle、大きさと角度のばらつき、Amount 15、Fade、Alpha、Slack、重ね方、色（なし・単色・乱数で 1〜3 色・奥行きで 1〜3 色・奥行き＋乱数）、明るさ・彩度を保つ。 — 借りる先：Motolii にほとんど在る。copies、scatter_x・scatter_y・scatter_z・scatter_size・scatter_rotate、seed、drift_x・drift_y、copy_fill（写しごとの色）、段の fog（Fade と奥行きの色）。2D なら ISF `Replicate Random.fs`（在る）。 — 入れ方：**果てなく繰り返す所（カメラの周りの箱の中で、写しを回り込ませる）が無い**ので、ここがエンジンの手。この回り込みの見本は見つからない。

- **Streak Strips**（blur）：同じ幅の帯に分け、それぞれの帯の真ん中の 1 列を帯いっぱいに引き伸ばして、色の筋にする。つまみは Count 10、Angle 45°、Phase、Mask To Layer、Alpha。 — 借りる先：GPUImage の `framework/Source/GPUImagePixellateFilter.m`（Brad Larson ほか、BSD-3-Clause）。fractionalWidthOfPixel と aspectRatio があり、aspectRatio をとても大きくすると片方の向きだけの帯になり、それぞれの帯の真ん中を取るので、この効果と同じ絵になる。ISF `Pixellate.fs`（在る）は正方形のマスしか作れない。 — 入れ方：Objective-C の SHADER_STRING の中の文字を取り出して ISF の頭を付ける口が要る（Color Balance と同じ手）。角度は前後に ISF `Rotate.fs` を挟む。Phase は Layer Position。

- **Warp Blur**（blur）：フラクタルノイズの勾配に沿ってぼかし、筆で塗ったような筋の歪みを作る。つまみは Parallax、Offset、Strength 0.25、Evolution、向き（Φ・R・ΦR・X・Y・XY）、Scale、Stretch、Seed、Constrain Sample Axis、Screen Space、Octaves 6、Bias。 — 借りる先：**見つからない**。いちばん近いのは openfx-lic（tkarabela、MIT）の Line Integral Convolution。ただし OpenFX のプラグインで CPU で動き、ベクトルの場に沿ってノイズをなぞる物で、絵をなぞる物ではない。ノイズの場は ISF `Simplex Noise.fs`（在る）で作れる。 — 入れ方（もし作るなら）：ノイズの絵を効果の 2 枚目の絵として渡し、その勾配に沿って何回も読む .fs が要る。そういう ISF・three.js・gl-transitions の物は見つからない。

## 出典

- 説明文とつまみ：Alight Motion 6.2.60（Android）の APK。エミュレーターから取り出した物（`assets/effects/chromakey-advanced.xml`・`chromatic-zoomblur.xml`・`colorbalance.xml`・`energybeam.xml`・`grunge.xml`・`spill-cleaner.xml`・`lumastamper.xml`・`mattefringe.xml`・`pagecurl.xml`・`parenthelper.xml`・`transform.xml`・`s3d-ring.xml`・`scaleassist.xml`・`shapefield.xml`・`streakstrips.xml`・`warpblur.xml`）。
- Parenting Helper：support.alightmotion.com「Layer Parenting and Null Objects」（記事 10536997444369）。
- 入った版：APKMirror の 4.0.2・4.1.0・4.5.0.9491・5.0.0 の更新履歴（検索の結果で見ただけ）、Alight Motion 公式の X（2023-03-31）。
- ISF-Files：`research_notes/isf/ISF-Files`（MIT）。glfx.js：github.com/evanw/glfx.js（MIT）。android-gpuimage：github.com/wasabeef/android-gpuimage（Apache-2.0、README の License）。GPUImage：github.com/BradLarson/GPUImage（BSD-3-Clause）。three.js：github.com/mrdoob/three.js（MIT）。Rive：github.com/rive-app/rive-runtime（MIT）。openfx-lic：github.com/tkarabela/openfx-lic（MIT）。ambientCG：ambientcg.com（CC0）。LYGIA：Prosperity Public License 3.0.0（使わない）。
