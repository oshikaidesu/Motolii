# E. 借りる先をまだ探していない 50 本の借りる先（10/25 調べ）

Alight Motion の効果のうち、[alight_checklist.md](alight_checklist.md) の E にある 50 本。自分では書かず、在る物を借りる。借りる先と許可（ライセンス）を 1 本ずつ書いた。

## 数

| | 数 | 中身 |
|---|---|---|
| 置くだけで出る（コードを書かない） | **30** | ISF の .fs を置く・ISF をつなぐ・今ある仕組み（写し・ふるまい・出来合いの動き）で組む |
| 　うち、近いだけ（同じではない） | 7 | Bend・Electric Edges・Glow Scan・Hexagon Array・Iridescence・Omino Glass・Stretch Segment |
| エンジンの仕事が要る | **15** | 読み口を 1 つ足す・後ろの絵を効果に渡す・距離の絵（SDF）を作る・QuickJS で JS を回す など |
| 見つからない | **5** | Contour Strips・Hexagon Tile Rotate・Hexagon Tile Shift・Ribbon・Tile Shift（ちょうど同じ物） |

「置くだけ」には、ISF をいくつかつなぐ物と、隠した写しを別の効果の「別の物の絵」に渡す物も入れた（両方とも今の Motolii でできる：ASSETS.md の ISF の行・検査用の絵の行）。

## 借りる先の一覧（確かめた物）

| 借りる先 | 許可 | 確かめたこと | 入れ方 |
|---|---|---|---|
| Vidvox ISF-Files（手元 `research_notes/isf/ISF-Files/ISF`） | MIT | 327 本。GitHub の今の物と同じ（数え直した） | 効果のフォルダに置くだけ |
| ossia score-user-library の `Presets/GLSL_shaders`（https://github.com/ossia/score-user-library） | 倉庫は CC BY-SA 4.0（商用可、手を入れた物は同じ許可で配る）。VIDVOX 由来の物は同じ所の `LICENSE-ISF`（MIT） | ISF の .fs が 2055 本。`linear/` は ossia の Jean-Michaël Celerier 作（CREDIT: ossia score）。`grigm/` は Shadertoy の写しが多く許可が分からないので使わない | 置くだけ（ISF） |
| pixi-filters v6.1.5（https://github.com/pixijs/filters） | MIT | 38 種。各効果に .frag（GLSL ES 3）と .wgsl が別のファイルで在る。既定の値は *Filter.ts、`__DIST__` などは JS が文字を置き換える | 読み口が要る：three_isf と同じ道（QuickJS で lib/index.mjs を読み、Filter と GlProgram を小さな代役にして frag と uniforms を取る） |
| cables の ops（https://github.com/cables-gl/cables `src/ops/base/Ops.Gl.ImageCompose.*`） | MIT（package.json） | 画の op が約 200。frag は `IN`・`UNI`・`outColor` の短い GLSL、つまみは同じ所の .json | 読み口が要る（.json と .frag から ISF に） |
| three.js（手元 `research_notes/threejs_shaders/three`、r152 の `examples/jsm/geometries/LightningStrike.js`） | MIT | LightningStrike.js は r152 まで在り r153 で消えた（raw の URL で確かめた） | 効果の .js は今の three_isf。形を作る .js は読み口が要る |
| Godot の `servers/rendering/renderer_rd/shaders/canvas_sdf.glsl` | MIT | 2D の距離の絵を Jump Flood で作る計算シェーダー | エンジン：Dawn の計算の口（cbeam と同じ） |
| pannellum の `src/js/libpannellum.js`（fragEquirectangular） | MIT | 正距円筒の絵を yaw・pitch・roll・hfov で平らな視野に | ISF に包む読み口か糊 |
| perfect-freehand（https://github.com/steveruizok/perfect-freehand） | MIT | 点の列から太さの変わる線の輪郭（始めと終わりの細り） | QuickJS で回し、輪郭を libtess2 へ |
| CountUp.js（https://github.com/inorganik/countUp.js） | MIT | 数を数え上げる・桁区切り・小数・イーズ | QuickJS で回す（画面の要素は小さな代役） |
| FFmpeg の v360（libavfilter/vf_v360.c） | LGPL-2.1 以上 | 正距円筒→正距円筒を yaw・pitch・roll で、正距円筒→平ら も | Motolii はもう ffmpeg を呼ぶ（`filament/mosh.h`）。同じ形で裏で焼く |
| GStreamer の GstVideoDither（`gst/video/video-dither.h`） | LGPL | Floyd–Steinberg・Sierra Lite・Bayer、段の数（quantizer）。手元の /opt/homebrew の頭で確かめた | もう入っている GStreamer の関数を呼ぶ |
| Skia の画の効果（SkImageFilters の Blur・Morphology・DistantLitSpecular など） | BSD-3 | SVG の feSpecularLighting と同じ。Motolii は今 pathops だけ作る（`filament/skia.sh`） | Skia の core の効果を作る所まで広げる |

**使わない（許可が合わない）**：Lygia（Prosperity 3.0.0＝商用は 30 日の試しだけ、商用はスポンサーの Patron License。LICENSE.md と README で確かめた）、Remotion の `@remotion/effects`（Long Shadows・Bevel Alpha・Contour Lines まで揃うが Remotion License、package は UNLICENSED）、obs-shaderfilter（GPL-2.0）、Natron・openfx-misc（GPL-2.0）、Ethereios/ISF-shaders（README が CC 非営利、Shadertoy から）、grigM/ISF-shaders-collection（許可なし、Shadertoy の写し）、GSAP（独自）。godotshaders.com は 1 本ごとに CC0・MIT・GPL-3 を作者が選ぶ。読み口と 1 本ごとの確認が要るので、今回は当てにしなかった。

## 50 本

- **360º Reorient Sphere**：360º の正距円筒の写真・動画の向きを変える — 借りる先：FFmpeg の v360（`v360=e:e:yaw=…:pitch=…:roll=…`）（LGPL-2.1）— 入れ方：エンジン。Datamosh と同じに裏で焼いて ~/Library/Caches/Motolii に置き、キーは sendcmd のファイルで渡す
- **360º Viewer**：360º の絵を球の内から見たように平らに映す — 借りる先：pannellum の fragEquirectangular（MIT）。もう一つは FFmpeg の v360（e→flat、h_fov・v_fov）（LGPL-2.1）— 入れ方：エンジン。pannellum の GLSL を ISF に包む糊、または v360 を Reorient と同じ形で焼く。今の「部屋に貼って environment_rotate」は物の効果ではない
- **Bend**：レイヤーを円の弧に沿って曲げる — 借りる先：`Circle Wrap Distortion.fs`（inputAngle で弧の開き）（MIT）— 入れ方：置くだけ（近い）。同じにするなら、形の bend（`filament/flat.mat` の頂点の段）を写真・文字の板にも掛ける（板を細かい三角形にする）エンジンの仕事
- **Blink**：不透明度を元の値と 0 で交互に — 借りる先：今あるふるまい oscillator を opacity に、`step` で 0 か 1 に丸める。画素なら `Strobe.fs`（MIT）。動きの見本は AE の式 `Math.round(time*f)%2` — 入れ方：置くだけ（今ある仕組み）
- **Contour Gradient**：不透明な所の輪郭からの距離でグラデーション — 借りる先：距離の絵は Godot の canvas_sdf.glsl（MIT）。見た目の見本は GIMP の Shapeburst・Photoshop の「シェイプバースト」 — 入れ方：エンジン。透明度から距離の絵を作る段を足し、距離を色の表で引く（この一歩の見本はまだ無い）
- **Contour Lines**：輪郭に沿って線を何本も — 借りる先：距離の絵は Godot の canvas_sdf.glsl（MIT）。1 本だけなら pixi-filters の OutlineFilter（MIT）か今の形・文字の stroke — 入れ方：エンジン。距離の絵から等間隔の線（Contour Gradient と同じ段を使う）
- **Contour Strips**：明るさか透明度を高さにして、細い紙の帯を前後に重ねる — 借りる先：見つからない（ossia の `depth/Depth Topo Bands.fs` などは深さの入力用で、レイヤーの明るさには使えない）— 入れ方：—
- **Copy Background**：後ろの絵をレイヤーに写し、調整レイヤーにする — 借りる先：仕組みの見本は pixi の BackdropBlurFilter（`blendRequired`・`backTexture`）（MIT）、AE の調整レイヤー — 入れ方：エンジン。物の効果の入力に「その物より下に描いた絵」を渡す（`filament/thing_fx.h`、`screen.h` の ScreenStack）。Magnify Background・Mask Blur・Polar Displacement Map も同じ仕事で出る
- **Count Up/Down**：文字の中の数に一定の値を足す・掛ける（数え上げ） — 借りる先：CountUp.js（MIT）。動きの見本は AE の Numbers と sourceText の式 — 入れ方：エンジン。QuickJS で CountUp.js を回し、文字の中の数だけを毎コマ書き換える口を足す
- **Dark Glow**：明るさのしきい値より暗い所の周りに暗いにじみ — 借りる先：`Gloom.fs`（Bloom の暗い版：最後に `min(mix(元, ぼかし*0.9, intensity), 元)`）（MIT）— 入れ方：置くだけ（`Gloom.fs` と `Gloom.vs` を効果のフォルダへ）
- **Echo Keyframes**：キーの動きだけを元に、前後の時刻の姿の写しを出す — 借りる先：今ある `copies`＋`stagger`（写しごとに秒ずれて動く）、色は `copy_fill`。動きの見本は AE の Echo — 入れ方：置くだけ（今ある仕組み）。写しごとに薄くするのに opacity のふるまいが要るかは確かめる
- **Electric Edges**：輪郭に電気のように光る線 — 借りる先：`Noise Displace.fs` → `Neon.fs` →（足すなら）`Glow-Fast.fs`（MIT）— 入れ方：置くだけ（近い、つなぐ）。輪郭の線を何本も出すのは Contour Lines を待つ
- **Feather**：透明度だけをぼかして縁を柔らかく — 借りる先：隠した写しに `Show Alpha.fs` → `Fast Blur.fs`、元に `Layer Mask.fs`（maskImage＝その写し、alphaMode＝Replace）（MIT）— 入れ方：置くだけ（つなぐ）。縁の外の色が黒く出るときは、`Layer Mask.fs` の前に色を広げる `Dilate.fs`
- **Fill Behind**：後ろを、ぼかし・彩度・色・明るさを変えた自分の写しで埋める — 借りる先：下に置いた写しに `Multi Pass Gaussian Blur.fs`＋`Color Controls.fs`（MIT）、大きさは物の size — 入れ方：置くだけ（今ある仕組みでつなぐ）。1 つの効果にまとめるのは、効果が物の四角より外に描ける仕事
- **Glow Scan**：明るさの帯の所だけをぼかして光らせる — 借りる先：上に重ねた写しに ossia の `basic/filter/Luma Key.fs`（threshold・softness）→ `Glow-Fast.fs`、重ね方 add（MIT）。帯（下限と上限）にするなら cables の LumaKey_v3（MIT）— 入れ方：置くだけ（近い、下限だけ）。帯は cables の読み口が要る
- **Hexagon Array**：不透明な所を六角形の並びで覆う — 借りる先：`Hexagonalize.fs`（gl-transitions から。2 枚目も同じ物、progress を止める）（MIT）。もう一つは glfx.js の hexagonalPixelate（MIT）— 入れ方：置くだけ（近い）。glfx は読み口が要る
- **Hexagon Tile Rotate**：六角形の升ごとに中身を回す — 借りる先：見つからない — 入れ方：—
- **Hexagon Tile Shift**：六角形の升ごとに中身をでたらめな向きにずらす — 借りる先：見つからない — 入れ方：—
- **Inner Blur**：透明度はぼかさず中身だけぼかす — 借りる先：`Fast Blur.fs` → `Layer Mask.fs`（maskImage＝`Show Alpha.fs` を掛けた隠した写し、alphaMode＝Replace）（MIT）— 入れ方：置くだけ（つなぐ）。AE の「透明部分を保持」の札を効果ごとに 1 つ足せば 1 本で済む（生成系ではもうやっている）
- **Inner Glow**：内側の縁に沿って光 — 借りる先：pixi-filters の GlowFilter（innerStrength・outerStrength・knockout）（MIT）— 入れ方：エンジン。pixi-filters の読み口
- **Inner Pinch/Bulge**：縁と透明度はそのままで、中を縮める・膨らませる — 借りる先：`Bump Distortion.fs`（level・radius・center）＋ Inner Blur と同じ `Layer Mask.fs` の鎖（MIT）。もう一つは pixi-filters の BulgePinchFilter（MIT）— 入れ方：置くだけ（つなぐ）
- **Iridescence**：不透明な所に流れる虹色の模様 — 借りる先：`Sine Warp Gradient.fs`（生成系、物の形の中だけに描く）（MIT）— 入れ方：置くだけ（近い。虹の色の並びはつまみで寄せる）
- **Lens Blur**：真ん中を除いて、回る向きにぼかす — 借りる先：ossia の `linear/Radial Blur.fs`（mode＝Spin、falloff で真ん中を残す）（CC BY-SA 4.0）。もう一つは pixi-filters の RadialBlurFilter（MIT）— 入れ方：置くだけ。ossia の物は透明度をぼかさない（元の a のまま）
- **Lightning**：2 点の間に枝分かれする稲妻 — 借りる先：three.js r152 の `examples/jsm/geometries/LightningStrike.js`（sourceOffset・destOffset・roughness・ramification、SimplexNoise を使う）（MIT）— 入れ方：エンジン。QuickJS で回し（three の Vector3 と BufferGeometry は代役）、出た三角形を形として Filament へ
- **Linear Streaks**：角度をつけて滑らせた写しを重ねた線状の筋 — 借りる先：今ある `copies`＋`copy_x`・`copy_y`、重ね方 `blend` lighten。画素なら `Diagonal Blur.fs`（MIT）か ossia の `linear/Directional Blur.fs`（CC BY-SA 4.0）— 入れ方：置くだけ（今ある仕組み）
- **Long Shadow**：不透明な所から長い平行の影 — 借りる先：下に置いた写しに `copies`＋`copy_x`・`copy_y`（AE のリピーター）、`tint` を黒に — 入れ方：置くだけ（今ある仕組み）
- **Luma Key**：しきい値より明るい・暗い所を透明に — 借りる先：ossia の `basic/filter/Luma Key.fs`（作者 IMIMOT、同じフォルダの LICENSE-ISF は MIT）。明るい側と暗い側の両方なら cables の LumaKey_v3（threshholdLow・High・INVERT）（MIT）— 入れ方：置くだけ。cables の物は読み口が要る。three.js の LuminosityHighPassShader.js は three の `luminance()` を使うので、three_isf で通るかは確かめる
- **Magnify Background**：レイヤーを透明にして後ろを拡大（虫めがね） — 借りる先：後ろの絵は Copy Background と同じ。拡大は `Zoom.fs` か `Bump Distortion.fs`（MIT）— 入れ方：エンジン（後ろの絵を効果に渡す）
- **Mask Blur**：レイヤーの明るさに応じた強さで後ろをぼかす — 借りる先：後ろの絵は Copy Background と同じ。動きの見本は AE の Compound Blur — 入れ方：エンジン（後ろの絵を渡す）。明るさで強さを変えるぼかしのシェーダーは見つかっていない
- **Move Along Path**：すぐ下のレイヤーの輪郭の途中に置く — 借りる先：今ある `copy_on`（写しを別の物の輪郭に）を `copies` 1 で、`copy_on_offset` にキー — 入れ方：置くだけ（今ある仕組み）
- **Omino Diffusion+**：誤差拡散で少ない色に。誤差を弱めたり強めたりして崩す — 借りる先：GStreamer の GstVideoDither（Floyd–Steinberg・Sierra Lite、段の数）（LGPL）。動きの見本は Omino の AE プラグインの説明 — 入れ方：エンジン（CPU で 1 コマずつ、もう入っている GStreamer の関数）。誤差を強める・弱めるつまみは GStreamer に無い
- **Omino Glass**：ガラスのような屈折、つやのある縁、7 つに分けた色ずれ — 借りる先：隠した写しに three.js の `NormalMapShader.js`、元に `Displace.fs`（displaceImage＝その写し）→ `Chromatic Aberration.fs`（MIT）— 入れ方：置くだけ（近い）。7 つに分ける色ずれは無い。NormalMapShader.js が three_isf で通るかは確かめる
- **Polar Coordinates**：直交座標を極座標に写して曲げる — 借りる先：ossia の `linear/Polar Coordinates.fs`（両向き、center・angle_offset・radius_scale・repeat）（CC BY-SA 4.0）。もう一つは cables の PolarCoords（MIT）— 入れ方：置くだけ
- **Polar Displacement Map**：レイヤーの色で後ろの画素を回る向きにずらす — 借りる先：後ろの絵は Copy Background と同じ。ずらし方は ossia の `linear/Polar Coordinates.fs` → `Displace.fs`（MIT）→ `linear/Polar Coordinates.fs`（戻す）の鎖 — 入れ方：エンジン（後ろの絵を渡す）
- **Precise Box Blur**：遅いが正確な箱形のぼかし — 借りる先：`Diagonal Blur.fs` を 2 回（angle 0 と 0.5、quality 2。中は等しい重みの線＝箱形）（MIT）— 入れ方：置くだけ（つなぐ）
- **Radial Rays**：不透明な所に単色の放射状の光線を何本も — 借りる先：`Stripes.fs`（生成系）→ ossia の `linear/Polar Coordinates.fs`（直交→極）で縞が光線になる。形の中に収めるのは `Layer Mask.fs` — 入れ方：置くだけ（つなぐ）
- **Radial Shadow**：1 点から放射状に伸びる長い影 — 借りる先：下に置いた写しに `copies`＋`copy_size`（中心＝光の点に anchor）、`tint` を黒に。画素なら `Replicate.fs`（MIT）— 入れ方：置くだけ（今ある仕組み）
- **Random Jitter**：選んだ角度に沿ってでたらめに行き来 — 借りる先：今あるふるまい random を x と y に、同じ seed で、minimum・maximum を cos・sin の割合に — 入れ方：置くだけ（今ある仕組み）。角度の札 1 つにまとめるなら小さな仕事
- **Ribbon**：不透明な所にねじれて流れる幅の変わるリボン — 借りる先：見つからない（近いのはレシピの `helix.js` の写し）— 入れ方：—
- **Smooth Bevel**：内側の縁になめらかな面取り — 借りる先：pixi-filters の BevelFilter（MIT、角ばった面取り）。なめらかにするなら Skia の SkImageFilters（Blur の後に DistantLitSpecular＝SVG の feSpecularLighting）（BSD-3）— 入れ方：エンジン。pixi の読み口か、Skia の効果を作る所まで広げる
- **Smooth Edges**：中の色はそのまま、縁のギザギザをなめらかに — 借りる先：Feather と同じ鎖で、`Layer Mask.fs` の contrast を上げてしきい値に戻す（MIT）— 入れ方：置くだけ（つなぐ）
- **Spin Blur**：回っているように角度の向きにぼかす — 借りる先：ossia の `linear/Radial Blur.fs`（mode＝Spin）（CC BY-SA 4.0）。もう一つは pixi-filters の RadialBlurFilter（MIT）— 入れ方：置くだけ
- **Spin Streaks**：1 点の周りに回した写しを重ねた弧の筋 — 借りる先：今ある `copies`＋`copy_rotate`（anchor を中心に）、重ね方 lighten。画素なら ossia の `linear/Radial Blur.fs`（Spin）— 入れ方：置くだけ（今ある仕組み）
- **Starfield**：2 次元で動かせる視差のある星空 — 借りる先：小さな丸に `copies`＋`scatter_z` ほか散らす値、カメラを動かす（本当の視差）。今の `Hyperspace.fs` も — 入れ方：置くだけ（今ある仕組み）
- **Stretch Segment**：角度に沿って切り、離した間を画素を引き伸ばして埋める — 借りる先：`Rotate.fs` → `Edge Blowout.fs`（insideBleed）→ `Rotate.fs`（MIT）— 入れ方：置くだけ（近い。切れ目の両側が離れて動くのは無い）
- **Stroke Taper**：手描きの線の始めと終わりを細く — 借りる先：perfect-freehand（taper・easing・thinning）（MIT）— 入れ方：エンジン。QuickJS で回し、出た輪郭を libtess2 で塗る
- **Text Progress**：文字を 1 字ずつ途中まで見せる — 借りる先：今ある文字の split＋出来合いの動き（`filament/animators/*.js` の「出る数」）。動きの見本は AE のテキストアニメーターの範囲セレクター — 入れ方：置くだけ（今ある仕組み）
- **Tile Shift**：格子に分け、升ごとに中身をでたらめな向きにずらす — 借りる先：ちょうど同じ物は見つからない（近いのは今の `Pixel Shifter.fs` と `Glitch Shifter.fs`）— 入れ方：—
- **Time Quantization**：この物だけ時間を段々に進める — 借りる先：動きの見本は AE の Posterize Time（フレームレートのつまみ）。キーだけなら今の anime.js の `steps(n)` — 入れ方：エンジン（小さい）。物の時間を `floor(t×fps)/fps` にして、動画・Lottie・キー・ふるまいに渡す
- **Zoom Streaks**：1 点の周りに拡大した写しを重ねた放射の筋 — 借りる先：今ある `copies`＋`copy_size`（anchor を中心に）、重ね方 lighten。画素なら `God Rays.fs`（MIT）か ossia の `linear/Radial Blur.fs`（Zoom）— 入れ方：置くだけ（今ある仕組み）

## エンジンの仕事をまとめると（1 つで何本出るか）

| 仕事 | 出る効果 |
|---|---|
| 後ろの絵を物の効果に渡す | Copy Background・Magnify Background・Mask Blur・Polar Displacement Map（4） |
| 透明度から距離の絵（Godot canvas_sdf.glsl） | Contour Gradient・Contour Lines（2）、Smooth Bevel・Inner Glow も描き直せる |
| pixi-filters の読み口（three_isf と同じ道） | Inner Glow・Smooth Bevel（2）、ほか RadialBlur・ZoomBlur・BulgePinch・Outline も置けるようになる |
| QuickJS で JS の部品を回す（今の仕組みを使う） | Lightning・Stroke Taper・Count Up/Down（3） |
| ffmpeg の v360 を裏で焼く | 360º Reorient Sphere・360º Viewer（2） |
| 物の時間を段々に | Time Quantization（1） |
| GStreamer の誤差拡散 | Omino Diffusion+（1） |

まだ確かめていない：つないだ ISF を実際に描いて Alight Motion と見比べること、three.js の NormalMapShader.js と LuminosityHighPassShader.js が three_isf で通るか、写しごとの不透明度を下げられるか。
