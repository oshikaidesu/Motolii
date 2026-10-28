# Alight Motion チェックリスト（自走用）

[alight_map.md](alight_map.md) の全行を、抜けが無いように 1 行 1 箱にしたもの。表から機械で作り、数を数えて合わせた（効果 169＋機能 32＝201）。実機で見た分を H・I に足した（alight_ui.md）。

- 箱を付ける決まり：Motolii の窓で、その物に掛けて、絵か動画で見て、Alight Motion の公式の一文どおりに動いたら [x]。「有る」も見るまでは [ ]。
- 「一部」は、足りない所を埋めたら [x]。
- 借りた先（ファイル・ライセンス）を行の後ろに書き足す。
- Alight Motion は下限。Motolii にだけあるものは表の最後の段。

## A. 画面全体にしか掛からない色を、物ごとにも（9）

- [x] **Brightness / Contrast**（一部）：Adjusts the brightness or contrast of the layer. — いま：段の exposure・contrast（全体だけ、物ごとは無い） — ISF：Color Controls — 済（10/28）：物の fx に `filament/effects/Color Controls.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [x] **Color Temperature**（一部）：Corrects a photo or video so it appears as if illuminated by a light source with a different color temperature. Useful for changing the mood of a scene or correcting for the wrong illumination or whit — いま：段の temperature（全体だけ） — ISF：White Point Adjust — 済（10/28）：物の fx に `filament/effects/White Point Adjust.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [x] **Color Tune**（一部）：Smoothly tune the color of dark and light parts of a photo or video segmented by lift, gamma, and gain. Use this to achieve various color "looks", to change the mood of the video, or to match the colo — いま：段の slope・offset・power と影・中間・明るい所の色（全体だけ） — ISF：LGG — 済（10/28）：物の fx に `filament/effects/LGG.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [x] **Exposure / Gamma**（一部）：Adjusts the exposure, gamma curve, and luminance offset of photo and video layers. — いま：段の exposure（全体だけ） — ISF：Exposure Adjust / Gamma Correction — 済（10/28）：物の fx に `filament/effects/Exposure Adjust.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [x] **Highlights and Shadows**（一部）：Independently adjusts the intensity of the lightest and darkest parts of the layer. — いま：段の影・明るい所の色（全体だけ） — ISF：Color Levels — 済（10/28）：AbsentMotion の `effects/am_highlightshadow.wgsl`（Apache-2.0、Highlights 0.15・Shadows 1.8、ファイルの頭が宣言する範囲）。窓で写真 2 枚の左だけが、空は薄く草は暗くなるのを絵で見た（左 68358、右 13319、`skin/book/integration_test/alight_check_test.dart` SECTION=X）
- [x] **Lens Flare**（一部）：Renders a lens flare in the layer. — いま：段の lens_flare（全体だけ） — ISF：Lens Flare — 済（10/28）：物の fx に `filament/effects/Lens Flare.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [x] **Light Glow**（一部）：Creates a soft glow and optional halo around the lighter areas within a photo or video layer based on a luminance threshold. — いま：段の bloom（全体だけ） — ISF：Bloom — 済（10/28）：物の fx に `filament/effects/Bloom.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`） — 済（10/28）：AbsentMotion の `effects/am_lightglow.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Saturation / Vibrance**（一部）：Adjusts the saturation or purity of colors in the layer. — いま：段の saturation・vibrance（全体だけ） — ISF：Vibrance — 済（10/28）：物の fx に `filament/effects/Vibrance.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_check_test.dart`）
- [ ] **Solid Matte**（一部）：Composites the layer on a solid color matte, filling transparent parts of the layer with a solid background color. — いま：段の background（全体だけ） — 見た（10/28、SECTION=T）：AbsentMotion の `effects/am_solidmatte.wgsl`（Apache-2.0、既定の青、文字の size 0.32）。文字の透けた所は青で埋まるが、その板が右の気球の左端まで被る（右 58841、段の右半分）

## B. 形にしか掛からない曲げを、写真・動画・文字にも（6）

- [x] **Fractal Warp**（一部）：Distorts the layer using a procedurally generated fractal pattern. Useful for simulating distortions caused by real-world phenomena such as fire, water, and heat. — いま：形の warp・wave_noise（形だけ） — ISF：Noise Displace — 済（10/28）：写真の物の fx に `filament/effects/Noise Displace.fs`（Vidvox ISF-Files、MIT）、x と y を 0.05。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=B）
- [x] **Pinch/Bulge**（一部）：Distorts the layer by pinching it towards the center or expanding it outward. — いま：形の pinch・bloat（形だけ） — ISF：Bump Distortion — 済（10/28）：写真の物の fx に `filament/effects/Bump Distortion.fs`（Vidvox ISF-Files、MIT）、膨らみ 0.8、半径 0.4。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=B）
- [x] **Roughen Edges**（一部）：Roughens the edges of the opaque parts of the layer. — いま：形の rough（形だけ） — 済（10/28）：AbsentMotion の `effects/am_roughenedges.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [ ] **Stretch Axis**（一部）：Stretches the layer in an arbitrary direction. — いま：形の stretch・skew（形だけ） — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_stretchaxis.wgsl`（Scale 2.2）。左は大きく伸びたが、右の写真まで被った（右 37701、無変更の右は約 13300） — 見た（10/28、SECTION=AF）：同じファイルの Scale 1.45（範囲 0.01〜50、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/stretchaxis.json ）。左の気球は横に伸びるが、その右端が右の気球の左を隠す
- [x] **Turbulent Displace**（一部）：Distorts the layer along selected coordinate axes using a random pattern of smooth fractal noise. Distortion may be along one or more axis in either cartesian or polar coordinate space. Useful for sim — いま：形の wiggle・wave_noise（形だけ） — ISF：Noise Displace — 済（10/28）：写真の物の fx に `filament/effects/Noise Displace.fs`（Vidvox ISF-Files、MIT）、x だけ 0.05。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=B）
- [ ] **Wave Warp**（一部）：Distorts the layer based on a sine wave. — いま：形の warp・ISF Sine Warp Tile（形だけ） — 見た（10/28、SECTION=S2）：AbsentMotion の `effects/am_wavewarp.wgsl`（既定 Magnitude 4）。左は 19522 画素変わったが、正弦の曲がりとしては見えない。SECTION=Z の `effects/Sine Warp Tile.fs`（angle 0.35）は正弦ではなく、丸い中に縮んだ絵 — 見た（10/28、SECTION=AF）：同じファイルの Magnitude 12・Spacing 8（範囲は Magnitude 0〜30、Spacing 0〜500、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/wavewarp.json ）。輪郭は動かず、正弦の曲がりとしては見えない。右の気球は元の絵

## C. レイヤーの絵を立体の面に貼る（faux-3D）（16）

- [x] **Box**（一部）：Renders a faux-3D box using the layer as a surface texture. — いま：立体 cube（キットの形）。物の絵を面に貼るのは無い — 済（10/28）：物の fx に `filament/solids/Box.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Cube**（一部）：Renders a faux-3D cube using the layer as a surface texture. — いま：立体 cube。絵を面に貼るのは無い — ISF：cube — 済（10/28）：物の fx に `filament/solids/Cube.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Cylinder**（無い）：Renders a faux-3D cylinder using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Cylinder.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Ellipsoid**（一部）：Renders a faux-3D ellipsoid using the layer as a surface texture. — いま：立体 ball。絵を面に貼るのは無い — 済（10/28）：物の fx に `filament/solids/Ellipsoid.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Heart**（無い）：Renders a faux-3D heart using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Heart.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Hexagonal Prism**（無い）：Renders a faux-3D hexagonal prism using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Hexagonal_Prism.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Hollow Box**（無い）：Renders a faux-3D hollow box, using the layer as a surface texture, with sides that can be turned on or off, punched out with preset shapes, or offset outward. — 済（10/28）：物の fx に `filament/solids/Hollow_Box.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Octahedron**（無い）：Renders a faux-3D regular octahedron using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Octahedron.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Pyramid**（無い）：Renders a faux-3D regular pyramid with a square base, using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Pyramid.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [ ] **Raster Extrude**（無い）：Renders a faux-3D raster extrusion of the layer based on the opaque and transparent pixels within the layer, using the opaque parts of the layer as a surface texture for the front and back of the extr — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_rasterextrude.wgsl`（既定 Depth 0.25）。左は 52274 画素変わったが、押し出した厚みとしては見えない。右の気球はそのまま
- [x] **Spherize**（一部）：Renders a faux-3D sphere, interpreting the layer as an equirectangular projection and using it as a surface texture for the sphere. — いま：ISF Sphere Map — 済（10/28）：物の fx に `filament/solids/Spherize.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Star Polyhedron**（無い）：Renders a faux-3D star polyhedron using the layer as a surface texture. — 済（10/28）：AbsentMotion の `effects/am_starpolyhedron.wgsl`（Apache-2.0、既定 Radius 0.95）。窓で写真 2 枚の左だけが、写真を面にした星になるのを絵で見た（左 74700、右 13327、`skin/book/integration_test/alight_check_test.dart` SECTION=X）
- [x] **Star Prism**（無い）：Renders a faux-3D star prism using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Star_Prism.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Three-axis Cross**（無い）：Renders a faux-3D three-axis cross using the layer as a surface texture. — 済（10/28）：物の fx に `filament/solids/Three-axis_Cross.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Torus**（一部）：Renders a faux-3D torus using the layer as a surface texture. — いま：立体 ring（絵を面に貼るのは無い） — 済（10/28）：物の fx に `filament/solids/Torus.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）
- [x] **Tunnel**（無い）：Renders a round faux-3D tunnel using the layer as the interior surface texture. — 済（10/28）：物の fx に `filament/solids/Tunnel.js`（物の絵を立体の面に、three.js の形：各ファイルの頭）。窓で写真 2 枚の左を少し傾けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=C）

## D. 手元の ISF-Files（MIT）に近いものがある：置いて確かめる（55）

- [x] **Block Dissolve**（無い）：Randomly removes grid-aligned rectangular pieces from the layer by making them transparent. Keyframe animation can be applied to remove progressively more and more rectangles over time. — ISF：Random Squares Mask — 済（10/28）：物の fx に `filament/effects/Random Squares Mask.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Block Noise**（無い）：Multiplies randomly generated rectangular blocks of noise with the existing colors of opaque parts of the layer. — ISF：JPEG Block Corruption — 済（10/28）：物の fx に `filament/effects/JPEG Block Corruption.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Box Blur**（一部）：Blurs the layer by averaging the colors of nearby pixels in a surrounding rectangular area (hence "box" blur). — いま：物の blur（ガウス） — ISF：Fast Blur — 済（10/28）：物の fx に `filament/effects/Fast Blur.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Bump Map**（無い）：Uses a height map to model bumps or perturbations in the surface of the layer, then applies simulated lighting and shading. — ISF：Emboss — 済（10/28）：物の fx に `filament/effects/Emboss.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Channel Remap (HSV)**（無い）：Remaps the hue, saturation, value, and alpha channels in the layer. — ISF：RGBtoHSV / HSVtoRGB — 済（10/28）：物の fx に `filament/effects/RGBtoHSV.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Channel Remap (RGB)**（無い）：Remaps the red, green, blue, and alpha channels in the layer. — ISF：RGBA Swap — 済（10/28）：物の fx に `filament/effects/RGBA Swap.fs`（Vidvox ISF-Files、MIT）、赤と青の入れ替え。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D2）
- [x] **Checker**（無い）：Renders a checkerboard pattern in the opaque parts of the layer. — ISF：Checkerboard — 済（10/28）：物の fx に `filament/effects/Checkerboard.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Chroma Key**（一部）：Keys out all pixels similar to a particular color, making those parts of the layer transparent. Useful for greenscreening or bluescreening. — いま：動画の keyed（四隅の色を抜く。色は選べない） — ISF：Chroma Mask — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_chromakey.wgsl`（既定の緑）。草は抜けず、写真は見た目のまま。SECTION=P で Tolerance 0.7 にすると周りは黒くなるが、残った気球も暗く、一色を抜く一文としては見えない。同じ回の `effects/Chroma Mask.fs`（既定の緑）も、草が抜ける絵にはならない — 済（10/28、SECTION=AE）：同じ `effects/am_chromakey.wgsl`（Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/chromakey.json ）。Key Color を空の色（R 0.50 G 0.69 B 0.90、つまみの 0〜1）、Tolerance 0.12（ファイルの既定は 0.25、範囲 0〜1）。左の空と、空に近い青の筋だけが抜け、気球のほかの色と草は残る。右の気球は元の絵（段の右は無変更と同じ 3079）。Tolerance 0.25 は草や人まで抜けるので、一文に合うのは 0.12
- [x] **Circular Ripple**（一部）：Distorts the layer in a pattern concentric circles around a center point, similar to the ripples caused by a pebble dropped in water. — いま：ISF Circle Splash Distortion — ISF：Ripples — 済（10/28）：物の fx に `filament/effects/Ripples.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Clouds**（無い）：Renders a procedurally generated cloud pattern in the opaque parts of the layer. — ISF：Simplex Noise — 済（10/28）：物の fx に `filament/effects/Simplex Noise.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Curl**（無い）：Deforms the layer by curling it into a circular shape from one end. — ISF：Inverted Page Curl — 済（10/28）：物の fx に `filament/effects/Inverted Page Curl.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Directional Blur**（一部）：Blurs the layer by averaging nearby pixel colors along a specified angle, similar to the blur caused by motion in a fixed direction. — いま：物の blur の向き（横・縦だけ、角度は無い） — ISF：Linear Blur — 済（10/28）：AbsentMotion の `effects/am_directionalblur.wgsl`（Apache-2.0、既定 Length 20）。窓で写真 2 枚の左だけが向きのあるぼかしになるのを絵と数で見た（左 45541、右 7918、`skin/book/integration_test/alight_check_test.dart` SECTION=S2）
- [ ] **Displacement Map**（無い）：Displaces pixels in the background based on the color of pixels in the layer. — ISF：Displace — 見た（10/28、SECTION=P）：`effects/Displace.fs`（Vidvox ISF-Files、MIT、uDisplaceAmt 1）。後ろの絵ではなく、左の写真自身が裂けて黒地に散った。右の写真は変わらない — クラス（backdrop を読む）：AbsentMotion は backdrop を nullptr で渡す（effects.cpp、Apache-2.0、上の Copy Background と同じ出典）。SECTION=AC の fill behind は白い板のままで、クラスは通っていない
- [x] **Dissolve**（一部）：Randomly removes pixels in the layer. This effect can be animated to create a pixel dissolve transition. — いま：点群の dissolve だけ — ISF：Noise — 済（10/28）：AbsentMotion の `effects/am_dissolve.wgsl`（Apache-2.0、既定の Strength 0.75）。窓で写真 2 枚の左だけに掛け、左だけが粒で抜けるのを絵と数で見た（左 78489、右 8186、`skin/book/integration_test/alight_check_test.dart` SECTION=S）
- [x] **Dots**（無い）：Draws a pattern of random dots in the opaque parts of the layer. Each dot is placed randomly, but the randomness is controlled so that the overall distribution of dots when taken together is approxima — ISF：Polka Dots Curtain — 済（10/28）：AbsentMotion の `effects/am_dots.wgsl`（Apache-2.0、既定の Radius 0.2）。窓で写真 2 枚の左だけに掛け、左だけに点が出るのを絵と数で見た（左 23706、右 8183、`skin/book/integration_test/alight_check_test.dart` SECTION=S）
- [x] **Edge Glow**（一部）：Detects edges in a photo or video and creates a glow around them. — いま：ISF Edge Blowout — ISF：Edges — 済（10/28）：物の fx に `filament/effects/Edges.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D） — 済（10/28）：AbsentMotion の `effects/am_edgeglow.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [ ] **Fade In/Out**（一部）：Automatically animates the layer opacity to fade the layer in at its beginning, and out again at its end. — いま：キーで。出入りに自動で付くのは無い — ISF：Fade — 見た（10/28、SECTION=AA）：`effects/Fade.fs`（progress 0.5。gl-transitions の fade.glsl、MIT、https://github.com/gl-transitions/gl-transitions/blob/master/transitions/fade.glsl ）。左は暗くなるだけで、層の始めと終わりに自動で消える一文にはなっていない。右の気球は同じ絵のまま — 見た（10/28、SECTION=AC）：`filament/animators/fade_in.js` と `fade_out.js`（Cavalry の Auto-Animate、Fade Up / Fade Down、https://cavalry.studio/docs/nodes/behaviours/auto-animate/ 、文書の参照、コードは写していない）。0 秒では字が無く、0.9 秒で MOTOLII が出るが、終わりの II が右の気球に被る。fade_out は 0 秒で全部出て同じく被り、3.6 秒で薄く残る。層の不透明度が始めと終わりに自動で変わる一文ではなく、字ごとの二つに分かれた動き
- [x] **Find Edges**（無い）：Finds edges in a photo or video layer by detecting sharp changes in luminance between nearby pixels. — ISF：Edges — 済（10/28）：物の fx に `filament/effects/Edges.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Flicker**（一部）：Randomly flickers the layer by automatically adjusting the opacity over time. — いま：ふるまい random を opacity に — ISF：RGB Strobe — 見た（10/28、SECTION=S2）：AbsentMotion の `effects/am_flicker.wgsl`。止め絵 1 枚では、時間でちらつく一文まで見えない — 見た（10/28、SECTION=AC）：ふるまい noise を opacity に（minimum 0.15、maximum 1、https://cavalry.studio/docs/nodes/behaviours/noise/ 、文書の参照）。0.2 秒と 0.85 秒で左は少し暗いだけで、ちらつきとしては見えない。右は変わらない
- [x] **Four-color Gradient**（一部）：Overlays the opaque parts of the layer with a gradient made between four individually positioned colors. — いま：形・文字の gradient（2 色） — ISF：Corner Colors — 済（10/28）：物の fx に `filament/effects/Corner Colors.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Fractal Ridges**（無い）：Renders a procedurally generated pattern of turbulent ridges in the opaque parts of the layer. — ISF：Ridgelines — 済（10/28）：物の fx に `filament/effects/Ridgelines.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Glass**（一部）：Renders the layer as if the opaque parts were made of beveled glass, refracting the background and overlaying specular highlights. — いま：立体の ior・dispersion（本物の屈折、立体だけ）。平らな物のガラスは無い — ISF：Frosted Glass — 見た（10/28、SECTION=P）：`effects/Frosted Glass.fs`（Vidvox ISF-Files、MIT、Magnitude 0.1）。すりガラスの粒で、背景を屈折させる面取りのガラスではない。右の写真は変わらない — クラス（backdrop を読む）：Copy Background と同じ。SECTION=AC の fill behind は白い板で、クラスは通っていない
- [x] **Gradient Map**（一部）：Maps colors in the layer onto a gradient based on their lightness. — いま：物の tritone（3 色）・ISF False Color・Trio Tone — ISF：Duotone — 済（10/28）：AbsentMotion の `effects/am_gradientmap.wgsl`（Apache-2.0、明るさで 3 色）。窓で写真 2 枚の左だけが明るさで塗り替わるのを絵と数で見た（左 78998、右 8009、`skin/book/integration_test/alight_check_test.dart` SECTION=S2）
- [ ] **Grid**（無い）：Overlays the opaque parts of the layer with grid lines. — ISF：Graph Paper — 見た（10/28、SECTION=S）：AbsentMotion の `effects/am_grid.wgsl` を既定のまま左に掛けると、格子の線ではなく白い板になる。右の写真は変わらない。SECTION=Z の `effects/Graph Paper.fs`（Vidvox ISF-Files）は写真の上の格子ではなく、左が方眼紙に置き換わった。SECTION=AI：同じ `am_grid.wgsl`（https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/grid.json 、Apache-2.0、Count 12、Width 0.04、色 R 1 G 0.1 B 0.1 A 1。どれもファイルの範囲）。左は格子ではなく赤い板。右の気球の内側は元の写真（平均差 約 3）
- [x] **Halftone Lines**（無い）：Replaces continuous tones in the layer with a pattern of monochrome lines of varying thickness, a variation of the process used in newspaper or comic book printing. — ISF：Line Screen — 済（10/28）：物の fx に `filament/effects/Line Screen.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Hexagon Tiling**（無い）：Repeats the layer in a pattern of hexagonal tiles, filling the screen. — ISF：Hexagonalize — 見た（10/28、SECTION=S2）：AbsentMotion の `effects/am_hexagontiling.wgsl`。六角の繰り返しは出たが、右の写真まで覆った（右 33172）。画面を埋める一文どおりか、層をはみ出したかは、この絵だけでは分けられない
- [x] **Hot Color**（一部）：Remaps a grayscale image (or luminance values of a color image) into an intense color ramp based on two selected hues. — いま：ISF False Color — ISF：Thermal Camera — 済（10/28）：`filament/effects/False Color.fs`（Vidvox ISF-Files、MIT、CREDIT by zoidberg、明るい色と暗い色の 2 色）。窓で写真 2 枚の左だけが、明るさでその 2 色に塗り替わるのを絵で見た（左 75067、右 13335、`skin/book/integration_test/alight_check_test.dart` SECTION=Y）。同じ回の Thermal Camera は熱分布の多色で、右の写真の左端が黒く被った（右 38233）
- [x] **Hue Shift**（無い）：Rotates the color hues in the layer. — ISF：Multi Hue Shift — 済（10/28）：物の fx に `filament/effects/Multi Hue Shift.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Invert**（無い）：Inverts the layer's red, green, or blue channels independently. — ISF：Color Invert / RGB Invert — 済（10/28）：物の fx に `filament/effects/RGB Invert.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Matte Choker**（無い）：Expands the transparent parts of the layer, shrinking the opaque parts. — ISF：Erode / Dilate — 済（10/28）：AbsentMotion の `effects/am_mattechoker.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Mosaic**（一部）：Divides the layer into array of squares and averages the color of each square, resulting in a pixelated look. — いま：ISF Multi-Pixellate・Noise Pixellate — ISF：Mosaic — 済（10/28）：AbsentMotion の `effects/am_mosaic.wgsl`（Apache-2.0、既定の Count 20）。窓で写真 2 枚の左だけに掛け、左だけが四角の平均色になるのを絵と数で見た（左 61032、右 8196、`skin/book/integration_test/alight_check_test.dart` SECTION=S）
- [ ] **Motion Blur**（無い）：Simulates motion blur by blurring the layer based on position, rotation, and scale keyframe animation from the layer's Move & Transform settings, as well as motion from certain other effects that are  — ISF：VVMotionBlur 3.0 — 見た（10/28、SECTION=Y）：`effects/VVMotionBlur 3.0.fs`（既定）。キーの動きが無い止め絵では、ぼかしは見えない
- [ ] **Offset**（無い）：Offsets the layer content, wrapping at the edges of the bounding box of the layer. — ISF：Side Scroller And Flip — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_offset.wgsl`（Offset x 220）。中身は右へ動いたが、左は黒のままで端の折り返しは見えず、右の写真に被った（右 37169）。SECTION=AA：`effects/Side Scroller And Flip.fs`（Vidvox ISF-Files、MIT、https://github.com/Vidvox/ISF-Files 、slide 0.35・shift 0.15、鏡映はファイルの bool を切った）。左は下へずれて上の切れ端が下に出るが、その切れ端が右の気球の下まで被る（段の右で差の強い画素 2598）
- [x] **Palette Map**（無い）：Maps pixel colors in the layer to the closest matching color in a palette. — ISF：Color Schemes — 済（10/28）：AbsentMotion の `effects/am_palettemap.wgsl`（Apache-2.0、既定の Colors 5）。窓で写真 2 枚の左だけに掛け、左だけが限られた色になるのを絵と数で見た（左 69306、右 8220、`skin/book/integration_test/alight_check_test.dart` SECTION=S）
- [x] **Pixelate**（一部）：Decreases the layer resolution so that individual pixels become visible, sampling at the centers of the pixels, and drawing each pixel with an optional vignette. — いま：ISF Multi-Pixellate — ISF：Pixellate — 済（10/28）：物の fx に `filament/effects/Pixellate.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Radial Wipe**（無い）：Masks out a region of the layer bounded by two radial lines. — ISF：Radial — 済（10/28）：物の fx に `filament/effects/Radial.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Replace Color**（無い）：Replaces one color in the layer with another. — ISF：Color Replacement — 済（10/28）：物の fx に `filament/effects/Color Replacement.fs`（Vidvox ISF-Files、MIT）、空の青を赤に。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D2）
- [x] **Sharpen**（無い）：Enhances details in the layer by increasing the contrast wherever there are abrupt differences in the luminance of nearby pixels in the layer. — ISF：Sharpen RGB — 済（10/28）：物の fx に `filament/effects/Sharpen RGB.fs`（Vidvox ISF-Files、MIT）、強さ 5。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D2）
- [x] **Simple Starfield**（一部）：Renders a simple parallax starfield pattern, with a depth component, in the opaque parts of the layer. — いま：ISF Hyperspace — ISF：Star — 済（10/28）：AbsentMotion の `effects/am_simplestarfield.wgsl`（Apache-2.0、Size 3、ファイルの頭が宣言する範囲）。窓で写真 2 枚の左だけに、大きさの違う星が不透明な所へ乗るのを絵で見た（左 34372、右 13312、`skin/book/integration_test/alight_check_test.dart` SECTION=Z）。既定の Size 0.5 では星は見えなかった。止め絵では視差の動きまでは見ていない
- [x] **Soft Glow**（一部）：Creates a soft Gaussian-based glow within a photo or video layer, or around the outside edge of any layer. — いま：物の glow — ISF：Glow — 済（10/28）：AbsentMotion の `effects/am_softglow.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [ ] **Spectral Map**（無い）：Maps each color in the layer into a custom spectrum of colors based on the brightness of the original color. — ISF：Color Scales — 見た（10/28、SECTION=Y）：`effects/Color Scales.fs`（Author を Newton）。明るさで色を割り当てる一文ではなく、左が一面の桃色。右の写真は変わらない
- [ ] **Spin**（一部）：Automatically spins the layer at a given speed. — いま：rotate のキー（ずっと回り続けるふるまいは無い） — ISF：Rotate — 見た（10/28、SECTION=P）：`effects/Rotate.fs`（Vidvox ISF-Files、MIT、angle 0.25）。止め絵で 1 度回っただけで、速度で回り続ける一文にはなっていない。右の写真は変わらない
- [ ] **Spot Color**（無い）：Keeps one color while decreasing the saturation of other colors in the layer. — ISF：Chroma Desaturation Mask — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_spotcolor.wgsl`（既定の赤）。左は灰色になり、残った赤は気球の先の点だけで、一色を残す一文としては見えない。SECTION=P の `effects/Chroma Desaturation Mask.fs`（既定）は左が全部灰色で、残る一色が無い
- [x] **Star**（一部）：Renders an individual star in the current layer. — いま：形の star（光る星ではない） — ISF：Star — 済（10/28）：物の fx に `filament/effects/Star.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [x] **Stripes**（無い）：Renders a pattern of stripes in the opaque parts of the layer. — ISF：Random Stripes — 済（10/28）：物の fx に `filament/effects/Random Stripes.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Text Randomizer**（無い）：Randomly changes individual characters in a text layer. — ISF：Random Characters — 見た（10/28、SECTION=R）：`effects/Random Characters.fs`（Vidvox ISF-Files、MIT）。左の写真が緑の文字の板に置き換わり、文字レイヤーの字を 1 字ずつ入れ替える一文にはなっていない。右の写真は変わらない
- [x] **Threshold**（無い）：Converts the layer to black and white with a hard cutoff at the specified luminance threshold. — ISF：Color Monochrome — 済（10/28）：AbsentMotion の `effects/am_threshold.wgsl`（Apache-2.0、既定の Threshold 0.4）。窓で写真 2 枚の左だけに掛け、左だけが白黒になるのを絵と数で見た（左 70636、右 4886、`skin/book/integration_test/alight_check_test.dart` SECTION=S）
- [ ] **Tile Rotate**（無い）：Divides the layer into a grid of tiles and rotates the pixels in each tile independently. — ISF：Grid Flip — 見た（10/28、SECTION=S4）：AbsentMotion の `effects/am_tilerotate.wgsl`（既定 Count 10）。左は 19525 画素変わったが、タイルが回る一文としては見えない。SECTION=P の `effects/Grid Flip.fs`（progress 0.5）はタイルが回るのではなく、左が黒く欠け、右の写真の左端まで被った（右 24284）
- [x] **Tiles**（一部）：Repeats the layer in a tile pattern, filling the screen. — いま：ISF Quad Tile・Kaleidoscope Tile — ISF：Quad Tile — 済（10/28）：物の fx に `filament/effects/Quad Tile.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Timecode**（無い）：Replaces the text in the layer with a running timecode. — ISF：Digital Clock — 見た（10/28、SECTION=R）：`effects/Digital Clock.fs`（Vidvox ISF-Files、MIT、Clock Mode はファイルが宣言する Counter＝2）。左は 0:00:00 の数字になり、文字をタイムコードに置き換える一文ではなく、止め絵では走り続けるところも見ていない。右の写真は変わらない
- [x] **Turbulence**（無い）：Renders smooth patterns of procedurally-generated fractal noise in the opaque parts of the layer, useful for simulating real-world phenomena like clouds, or for use as a luminance map. — ISF：Simplex Noise — 済（10/28）：物の fx に `filament/effects/Simplex Noise.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Unsharp Mask**（無い）：Enhances the perceptual sharpness or "acutance" of a photo or video by increasing local contrast near edges, while avoiding changes to areas that were originally smooth. — ISF：Unsharp Mask — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_unsharpmask.wgsl`（Strength 1.5・Amount 2）。輪郭の強調は見えない。SECTION=R の `effects/Unsharp Mask.fs`（Vidvox ISF-Files、MIT、Intensity 6、頭の範囲は 0〜10）は、滑らかな所を残す強調ではなく、空も気球も粗い塊になる。右の写真は変わらない（右 13325）
- [x] **Voronoi Cells**（無い）：Renders procedurally-generated pattern of Voronoi cells based on randomly placed points. — ISF：Worley Cells — 済（10/28）：物の fx に `filament/effects/Worley Cells.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=D）
- [ ] **Wipe**（無い）：Masks out rectangular segments from opposite sides of the layer. Use keyframe animation to create a wipe transition. — ISF：Wipe Left / Right / Up / Down — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_wipe.wgsl`（End 0.4）。右半分は黒く欠けたが、両側から抜く一文にはなっていない。SECTION=Z で Start 0.2・End 0.75 にしても、両側から抜く絵にはならない
- [x] **Zoom Blur**（無い）：Applies a radial blur to the layer, as though zooming in or out. — ISF：Zoom — 済（10/28）：AbsentMotion の `effects/am_radialblur.wgsl`（Apache-2.0、既定 Strength 0.15）。窓で写真 2 枚の左だけが中心から放射状にぼけるのを絵と数で見た（左 36991、右 7975、`skin/book/integration_test/alight_check_test.dart` SECTION=S2）

## E. 借りる先をまだ探していない（50）

- [ ] **360º Reorient Sphere**（無い）：Reorients a 360º photo or video in equirectangular format. — 見た（10/28、SECTION=Y）：AbsentMotion の `effects/am_reorient360.wgsl`。頭に宣言されたつまみが無く、普通の写真は見た目のまま
- [ ] **360º Viewer**（一部）：Projects a 360º photo or video onto a flat surface, as if viewed from within the 360º sphere. — いま：部屋（environment）に正距円筒の絵を貼り、environment_rotate で回す。平らな物の効果としては無い — 見た（10/28、SECTION=Y）：AbsentMotion の `effects/am_viewer360.wgsl`（既定 Field of View 90）。普通の写真を引き伸ばしたように見え、球の中から見る一文にはなっておらず、右の写真まで被った（右 38129）
- [x] **Bend**（一部）：Bends the layer along a circular arc. — いま：形の bend（shape だけ）。写真・動画・文字には無い — 済（10/28）：物の fx に `filament/effects/Circle Wrap Distortion.fs`（Vidvox ISF-Files、MIT、alight_sources_E.md の借りる先）、そのまま。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=E）
- [x] **Blink**（一部）：Automatically adjusts the opacity of the layer, alternating between its normal value and zero over time. — 済（10/28）：ふるまい oscillator を opacity に、step 1（minimum 0、maximum 1、frequency 1）。0.25 秒は左の写真が元のまま、0.75 秒は左が消え、右の気球は両方そのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=V）。出典: https://cavalry.studio/docs/nodes/behaviours/oscillator/ と `filament/doc_relations.h` の oscillator、step は `filament/doc_kinds.h`。ライセンス: 文書の参照（Scene Group）。この回にコードは取り込んでいない
- [x] **Contour Gradient**（無い）：Renders a gradient around the contours of the opaque parts of the layer. — 済（10/28）：AbsentMotion の `effects/am_contourgradient.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Contour Lines**（一部）：Draws one or more lines around the contours of the opaque parts of the layer. — いま：形・文字の stroke（輪郭 1 本） — 済（10/28）：AbsentMotion の `effects/am_contourlines.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [ ] **Contour Strips**（無い）：Renders strips of thin paper-like material, layered front to back, using the layer luminance or opacity as a height map for each strip. — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_contourstrips.wgsl`（既定）。紙の帯ではなく左が真っ黒。右の写真は変わらない
- [ ] **Copy Background**（一部）：Copies the background into the layer, effectively turning the layer into an adjustment layer. Effects applied to the layer after this will apply to the background pixels covered by the opaque parts of — いま：段の script（画面全体に .wgsl）。下だけに掛ける調整レイヤーは無い — 見た（10/28、SECTION=AC）：左の気球の上に四角、fill を behind。白い板になり、下の気球は写らない。右の気球はそのまま。通り道は AbsentMotion の `effects.cpp`（backdrop を nullptr＝すでに描いた下として渡す、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/cpp/effects.cpp 、Apache-2.0）と `filament/flat.mat` の behind。その値はまだ 1 にならないので、クラス（backdrop を読む効果）は通っていない。この回にシェーダは書いていない
- [ ] **Count Up/Down**（無い）：Replaces existing numbers in a text layer with the same number added to or multiplied with a fixed value. Useful for creating animations that count up or down.
- [x] **Dark Glow**（無い）：Creates dark blooms around darker areas within a photo or video layer based on a luminance threshold. — 済（10/28）：AbsentMotion の `effects/am_darkglow.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [ ] **Echo Keyframes**（一部）：Creates one or more copies or "echoes" of the layer as it appeared or will appear at different points in time, based only on keyframe animation applied to the layer. — いま：形の trail・ISF Echo Trace・RGB Trails — 見た（10/28、SECTION=P）：`effects/Echo Trace.fs`（既定）。キーの動きが無い止め絵では、残像は見えない
- [x] **Electric Edges**（無い）：Creates a glowing energy-like outline and optional contour lines around the opaque parts of the layer, which can be animated to look like arcs of electricity. — 済（10/28）：AbsentMotion の `effects/am_electricedges.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Feather**（一部）：Softens the edges of the opaque part of the layer by blurring the alpha channel. — いま：クリップの clip_feather（切り抜きの縁だけ） — 済（10/28）：AbsentMotion の `effects/am_feather.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Fill Behind**（無い）：Fills the area behind the layer with a modified copy of the layer, with optional adjustments such as blur, saturation, tint, and brightness, providing a convenient way to quickly adapt a photo or vide — 済（10/28）：AbsentMotion の `effects/am_fillbehind.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Glow Scan**（無い）：Renders a blur combined with a smooth glow within a region of the layer defined by a luminance band. — 済（10/28）：AbsentMotion の `effects/am_glowscan.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Hexagon Array**（無い）：Covers opaque parts of the layer in an array of hexagons. — 済（10/28）：物の fx に `filament/effects/Hexagonalize.fs`（Vidvox ISF-Files、MIT、alight_sources_E.md の借りる先）、progress 0.5。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=E）
- [x] **Hexagon Tile Rotate**（無い）：Divides the layer into a grid of hexagonal tiles and rotates the pixels in each tile independently. — 済（10/28）：AbsentMotion の `effects/am_hexagontilerotate.wgsl`（Apache-2.0、Randomize 1、ファイルの頭が宣言する範囲）。窓で写真 2 枚の左だけが、六角のタイルごとに別の向きになるのを絵で見た（左 47532、右 13341、`skin/book/integration_test/alight_check_test.dart` SECTION=Y）
- [ ] **Hexagon Tile Shift**（無い）：Divides the layer into a grid of hexagonal tiles and shifts the contents of each tile in a random direction. — 見た（10/28、SECTION=Y）：AbsentMotion の `effects/am_hexagontileshift.wgsl`（Strength 0.8）。六角のずれは出たが、右の写真までタイルが飛んだ（右 17534、無変更の右は約 13300）
- [x] **Inner Blur**（無い）：Blurs the opaque parts of the layer without blurring the opacity. — 済（10/28）：AbsentMotion の `effects/am_innerblur.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Inner Glow**（無い）：Draws a glow along the inside edges of the opaque parts the layer. — 済（10/28）：AbsentMotion の `effects/am_innerglow.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Inner Pinch/Bulge**（無い）：Distorts the layer by pinching it towards the center or expanding it outwards, without affecting the edges or opacity of the layer. — 済（10/28）：AbsentMotion の `effects/am_innerbulge.wgsl`（Apache-2.0、既定 Strength 0.5）。窓で写真 2 枚の左だけが中で膨らみ、縁は残るのを絵と数で見た（左 66376、右 6745、`skin/book/integration_test/alight_check_test.dart` SECTION=S4）
- [x] **Iridescence**（無い）：Renders a flowing iridescent pattern in opaque parts of the layer. — 済（10/28）：AbsentMotion の `effects/am_iridescence.wgsl`（Apache-2.0、既定 Saturation 0.6・Alpha 0.8）。窓で写真 2 枚の左だけが油膜の虹になるのを絵と数で見た（左 76397、右 8274、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [ ] **Lens Blur**（一部）：Applies an angular blur to the layer, excluding the central area. — いま：カメラの focus・aperture（立体と世界の物だけ） — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_lensblur.wgsl`（既定 Radius 0.25）。中心を除く角度のぼかしは見えない。Radius 1.5 でも見えない
- [ ] **Lightning**（無い）：Renders fractal arcs of lightning between two points in the layer. — 見た（10/28、SECTION=S4）：AbsentMotion の `effects/am_lightning.wgsl` を既定のまま左に掛けると、稲妻ではなく白い板になる。右の写真は変わらない
- [x] **Linear Streaks**（無い）：Creates line-shaped streaks by sliding the layer at an angle and blending together overlapping pixels. — 済（10/28）：物の写し（AE のリピーターの形、alight_sources_E.md の借りる先）、copies 12・copy_x 0.03・copy_y −0.015・blend lighten。窓で写真 2 枚の左に掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=E2）
- [ ] **Long Shadow**（無い）：Renders long parallel shadows extending from opaque parts of the layer. — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_longshadow.wgsl`（既定 Length 300）。気球から伸びる影ではなく、右の写真の左端に灰色の帯（右 38001）
- [x] **Luma Key**（一部）：Keys out all pixels lighter than or darker than a given threshold, making those parts of the layer transparent. — いま：クリップの clip_matte luma（下の物で切る） — 済（10/28）：AbsentMotion の `effects/am_lumakey.wgsl`（Apache-2.0、既定 Threshold 0.3）。窓で写真 2 枚の左だけ、明るい空が消え暗い気球が残るのを絵と数で見た（左 51819、右 8282、`skin/book/integration_test/alight_check_test.dart` SECTION=S4）。舞台が黒いので、抜けた所が透明か黒塗りかは、この絵だけでは分けられない
- [ ] **Magnify Background**（無い）：Makes the layer transparent and magnifies the area behind the layer, as if the layer were a magnifying glass. — クラス（backdrop を読む）：Copy Background と同じ。SECTION=AC の fill behind は白い板で、クラスは通っていない
- [ ] **Mask Blur**（無い）：Blurs the background behind the layer, with the blur strength based on the layer luminance. — クラス（backdrop を読む）：Copy Background と同じ。SECTION=AC の fill behind は白い板で、クラスは通っていない
- [ ] **Move Along Path**（一部）：Positions the layer at a point partway way along the outline path of the layer immediately beneath. — いま：copy_on（写しを別の物の輪郭に）。1 個を道に沿わせて動かすのは写し 1 個で — 見た（10/28、SECTION=T）：copies 1・copy_on を下の円へ・copy_on_offset 0.25。黄色い四角は円の中心にいて、輪郭の途中にはいない。右の写真は変わらない。copy_on は写しが複数あるときだけ輪郭に乗る（`doc_staggered`）
- [x] **Omino Diffusion+**（一部）：Renders a photo or video layer with a limited palette using error diffusion, but damping or exaggerating the quantization errors to create intentional distortions. Individual images become unrecogniza — いま：ISF Dither-Bayer（並べた点のディザ、誤差拡散は無い） — 済（10/28）：AbsentMotion の `effects/am_ominodiffusion.wgsl`（Apache-2.0、Error Weight 8、ファイルの頭が宣言する 0〜10）。窓で写真 2 枚の左だけが、限られた色の誤差で気球と読めなくなるのを絵で見た（左 74981、右 13399、`skin/book/integration_test/alight_check_test.dart` SECTION=Z）。既定の Weight 0.75 では気球はまだ読めた
- [ ] **Omino Glass**（無い）：Renders the layer as though it were made of glass, with refraction and a glossy contour, and chromatic aberration based on segmentation of the visible spectrum into seven regions. The layer luminance — クラス（backdrop を読む）：Copy Background と同じ。SECTION=AC の fill behind は白い板で、クラスは通っていない 
- [x] **Polar Coordinates**（無い）：Warps the layer by mapping from cartesian coordinates to polar coordinates. — 済（10/28）：AbsentMotion の `effects/am_polar.wgsl`（Apache-2.0、既定 Interpolation 1）。窓で写真 2 枚の左だけが極座標に曲がり、右の気球はそのままなのを絵で見た（左 72274、右 19996、無変更の右は約 13300、`skin/book/integration_test/alight_check_test.dart` SECTION=W）
- [ ] **Polar Displacement Map**（無い）：Angularly displaces pixels in the background based on the color of pixels in the layer. — クラス（backdrop を読む）：Copy Background と同じ。SECTION=AC の fill behind は白い板で、クラスは通っていない
- [ ] **Precise Box Blur**（一部）：Performs a slow but precise box blur of the layer. — いま：物の blur
- [ ] **Radial Rays**（無い）：Overlays multiple solid-color radial rays in the opaque parts of the layer. — 見た（10/28、SECTION=S4）：AbsentMotion の `effects/am_radialrays.wgsl` を既定のまま左に掛けると、放射の線ではなく紫の板になる。右の写真は変わらない。SECTION=AI：同じファイル（https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/radialrays.json 、Apache-2.0、Count 8、Width 0.08、Alpha 1、色 R 1 G 0.15 B 0.1 A 1。どれもファイルの範囲）。左は放射の線ではなく赤い板。右の気球の内側は元の写真（平均差 約 3）
- [ ] **Radial Shadow**（無い）：Renders long shadows radiating away from a point, based on the opaque parts of the layer. — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_radialshadow.wgsl`（Length 2・Alpha 0.9）。気球から放射する影ではなく、右の写真の左端が暗く被った（右 38230）
- [ ] **Random Jitter**（一部）：Automatically jumps the layer randomly back and forth over time, along a selected angle. — 見た（10/28、SECTION=V）：ふるまい random を x に（minimum -0.75、maximum -0.35）。0 秒と 1 秒で左は同じ場所に寄り、時刻で行ったり来たりしない。右は変わらない。出典: https://cavalry.studio/docs/nodes/behaviours/random/ と `filament/doc_relations.h` の random（コピー番号のハッシュで、時刻を見ない）。ライセンス: 文書の参照（Scene Group）
- [ ] **Ribbon**（無い）：Renders a twisted, flowing ribbon of varying width in the opaque parts of the layer. — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_ribbon.wgsl`（既定）。ねじれた帯ではなく紫の板。右の写真は変わらない
- [x] **Smooth Bevel**（無い）：Draws a smooth curved bevel inside the edges of the opaque parts of the layer. — 済（10/28）：AbsentMotion の `effects/am_smoothbevel.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Smooth Edges**（無い）：Smooths the edges of the opaque parts of a layer without affecting the pixel colors within the layer. — 済（10/28）：AbsentMotion の `effects/am_smoothedges.wgsl`（Apache-2.0、ぼかした絵は AbsentMotion の effects.cpp と同じ sigma＝blur/3 のガウスを横と縦の 2 パスで）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを数と絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=AM）
- [x] **Spin Blur**（無い）：Applies an angular blur to the layer, as though it were spinning. — 済（10/28）：`effects/Radial Blur.fs`（ossia score-user-library、CC BY-SA 4.0、Mode はファイルが宣言する Spin＝1、Amount 0.4、Samples 2）。窓で写真 2 枚の左だけが中心のまわりに回るぼかしになり、右の気球はそのままなのを絵で見た（左 68369、右 13326、`skin/book/integration_test/alight_check_test.dart` SECTION=R）
- [x] **Spin Streaks**（無い）：Creates arc-shaped streaks by spinning the layer around a point and blending together overlapping pixels. — 済（10/28）：物の写し（AE のリピーターの形、alight_sources_E.md の借りる先）、copies 12・copy_rotate 3・blend lighten。窓で写真 2 枚の左に掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=E2）
- [ ] **Starfield**（一部）：Renders a complex parallax starfield that can be animated in two dimensions. — いま：ISF Hyperspace — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_starfield.wgsl`（Brightness 1.5）。左は大きさの違う星の野になった（左 74991、右 13361）。止め絵 1 枚では、視差で動く一文までは見ていない。SECTION=AA：`effects/Hyperspace.fs`（Vidvox ISF-Files、MIT、https://github.com/Vidvox/ISF-Files 、scrollAmount 0.8）。星ではなく、左の写真が二つに割れて曲がった絵。視差も見ていない。SECTION=AH：同じ `am_starfield.wgsl`（https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/starfield.json 、Apache-2.0、Brightness 1.5、Scale 0.3、Offset x はファイルの範囲 -100000..100000 の 0 と 80）。左はどちらも星の野で、Offset x を変えると星の並びが変わる。右の気球の内側は元の写真のまま（平均差 約 3）。同じ星を追って、近い星が遠い星より大きく動くところ、および Offset y の二方向は見ていないので箱は付けない
- [ ] **Stretch Segment**（無い）：Slices the layer along an angle and then moves the pieces apart, stretching the pixels between them. — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_stretch.wgsl`（既定 Stretch 10）。切れ目と引き伸ばしは見えない。Stretch 400 では左が大きく伸び、右の写真まで被った（右 37430）
- [ ] **Stroke Taper**（無い）：Tapers the strokes of a freehand drawing layer.
- [x] **Text Progress**（一部）：Partially displays a text layer, character by character. — いま：文字の split＋出来合いの動き（出る数） — 済（10/28）：`animators/print_on.js`（Cavalry の Sub-Mesh が字ごとに出す形、`filament/script.cpp` の animate の print、長さはファイルの頭の 0.67、再生 0.35）。窓で左の MOTOLII が MOTO まで出て残りは無く、右の気球はそのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=T）
- [ ] **Tile Shift**（一部）：Divides the layer into a grid of tiles and shifts the contents of each tile in a random direction. — いま：ISF Pixel Shifter — 見た（10/28、SECTION=X）：AbsentMotion の `effects/am_tileshift.wgsl`（Strength 0.8）。タイルのずれは出たが、右の写真までタイルが飛んだ（右 17722、無変更の右は約 13300）。SECTION=AA：`effects/Pixel Shifter.fs`（Vidvox ISF-Files、MIT、https://github.com/Vidvox/ISF-Files 、hFrequency 4・vFrequency 3）。タイルではなく正弦の帯で、その帯が右の気球まで被る（段の右 25664）
- [ ] **Time Quantization**（無い）：Causes time to pass in steps, but just for this layer.
- [x] **Zoom Streaks**（一部）：Creates radial streaks by scaling the layer around a point and blending together overlapping pixels. — いま：ISF Hyperspace — 済（10/28）：AbsentMotion の `effects/am_zoomstreaks.wgsl`（Apache-2.0、既定 Strength 0.15）。窓で写真 2 枚の左だけが中心から筋になるのを絵と数で見た（左 48469、右 8503、`skin/book/integration_test/alight_check_test.dart` SECTION=S4）

## F. 有る（確かめるだけ）（33）

- [x] **Auto-Shake**（有る）：Moves the layer randomly over time within specified parameters. — 済（10/28）：ふるまい noise を x と y に（x は -0.75〜-0.35、y は -0.12〜0.12、frequency 1、y の seed 7）。0.2 秒と 0.85 秒で左の写真の位置が違い、右の気球は両方そのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=V）。出典: https://cavalry.studio/docs/nodes/behaviours/noise/ と `filament/doc_relations.h` の noise。ライセンス: 文書の参照（Scene Group）。この回にコードは取り込んでいない — ISF：Shake
- [x] **CMYK Halftone Dots**（有る）：Replaces continuous tones in the layer with a pattern of dots of varying sizes for each CMYK color channel, similar to the process used in magazine or comic book printing. — いま：ISF CMYK Halftone — ISF：CMYK Halftone — 済（10/28）：AbsentMotion の `effects/am_cmykhalftone.wgsl`（Apache-2.0、既定 Amount 100）。窓で写真 2 枚の左だけが印刷の網点になるのを絵と数で見た（左 70794、右 8243、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [x] **Colorize**（有る）：Changes the color of the layer, keeping the original luminance. — いま：物の tint — 済（10/28）：物の tint 0.8、黒→黒・白→橙。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Drawing Progress**（有る）：Clips the beginning and end of the vector outline of the layer so that the outline is only partially drawn. — いま：形の trim・trim_begin・trim_offset — 済（10/28）：形の trim_begin 0.1・trim 0.4、白い線。窓で左の円の輪郭が欠け、右の写真はそのままなのを絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=Q）
- [x] **Flip Layer**（有る）：Flips the layer in 3D along a specified axis. — いま：rotate_x・rotate_y（2.5D） — ISF：Flip H / Flip V — 済（10/28）：物の rotate_y 180（左右に裏返る）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Gaussian Blur**（有る）：Applies a fast, smooth blur to the layer using a gaussian function, reducing noise and details in the layer. — いま：物の blur — ISF：Multi Pass Gaussian Blur — 済（10/28）：物の blur 20。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [ ] **Glow**（有る）：Draws a glow around the outside of the opaque parts of the layer. — いま：物の glow — 見た（10/28、SECTION=F）：物の glow ffffff・glow_blur 30 で、外の光は薄く、写真の中も明るくなる
- [x] **Gradient Overlay**（有る）：Overlays the opaque parts of the layer with a gradient. — いま：形・文字の gradient — ISF：Linear Gradient — 済（10/28）：AbsentMotion の `effects/am_gradientoverlay.wgsl`（Apache-2.0、既定 Opacity 0.6）。窓で写真 2 枚の左だけにグラデーションが乗るのを絵と数で見た（左 78436、右 4846、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [x] **Grid Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged in a grid. — いま：copies＋copy_columns — 済（10/28）：物の copies 4・copy_columns 2・copy_x・copy_y 0.45（写真を 0.4 に）。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Halftone Dots**（有る）：Replaces continuous tones in the layer with a pattern of monochrome dots of varying sizes, similar to the process used in newspaper or comic book printing. — いま：効果 halftone・dot_screen — ISF：Dot Screen — 済（10/28）：`effects/dot_screen.wgsl`（three.js の DotScreenShader、MIT）。窓で写真 2 枚の左だけが、大きさの違う白黒の点になるのを絵で見た（左 70955、右 13357、`skin/book/integration_test/alight_check_test.dart` SECTION=Z）。`effects/halftone.wgsl` は色の点で、右まで被った
- [x] **Kaleidoscope**（有る）：Creates symmetric radial reflections of the layer. — いま：効果 kaleido・ISF Kaleidoscope Tile — ISF：Kaleidoscope — 済（10/28）：AbsentMotion の `effects/am_kaleidoscope.wgsl`（Apache-2.0、既定 Segments 6）。窓で写真 2 枚の左だけが放射の鏡になるのを絵と数で見た（左 72749、右 8766、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [x] **Linear Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged along a line. — いま：copies＋copy_x・copy_y — 済（10/28）：物の copies 4・copy_x 0.15・copy_y 0.05。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Mirror**（有る）：Overlays the layer with a copy of itself mirrored along either the horizontal or vertical axis, with optional alpha and blending mode control. — いま：効果 mirror — ISF：Mirror — 済（10/28）：AbsentMotion の `effects/am_mirror.wgsl`（Apache-2.0）。窓で写真 2 枚の左だけが上下に鏡写しになるのを絵と数で見た（左 48249、右 8463、`skin/book/integration_test/alight_check_test.dart` SECTION=S4）
- [x] **Noise**（有る）：Blends random noise with the opaque parts of the layer. — いま：効果 grain・fine_grain — ISF：Noise — 済（10/28）：棚の `effects/fine_grain.wgsl`（Amount 0.8、ファイルの頭が宣言する範囲）。窓で写真 2 枚の左だけに粒が乗るのを絵で見た（左 69734、右 13338、`skin/book/integration_test/alight_check_test.dart` SECTION=W）。AbsentMotion の `effects/am_noise.wgsl`（既定 Strength 0.25）は粒として見えず、箱はそちらでは付けていない
- [x] **Oscillate**（有る）：Moves the layer back and forth repeatedly. — いま：ふるまい oscillator — 済（10/28）：ふるまい oscillator を x に（minimum -0.7、maximum -0.2、frequency 1）。窓で 0.25 秒は左の写真が右へ、0.75 秒は左へ寄り、右の気球は両方そのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=U）
- [x] **Posterize**（有る）：Decreases the number of step in the layer's color channels. — いま：ISF Posterize — ISF：Color Posterize — 済（10/28）：AbsentMotion の `effects/am_posterize.wgsl`（Apache-2.0、既定 Steps 10）。窓で写真 2 枚の左だけが段の色になるのを絵と数で見た（左 46519、右 8244、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [x] **Pulse Opacity**（有る）：Automatically pulses the layer opacity over time, based on a sine wave or triangular wave. — いま：ふるまい oscillator を opacity に — 済（10/28）：ふるまい oscillator を opacity に（minimum 0.15、maximum 1、frequency 1）。0.25 秒は元の明るさ、0.75 秒は左だけが暗く、右の気球はそのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=U）
- [x] **Pulse Size**（有る）：Automatically grows and shrinks the layer repeatedly over time, based on a sine wave or triangular wave. — いま：ふるまい oscillator を size に — 済（10/28）：ふるまい oscillator を size に（minimum 0.35、maximum 0.8、frequency 1）。0.25 秒は左がやや小さく、0.75 秒はさらに小さく、右の気球はそのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=U）
- [ ] **Radial Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged in a circular pattern. — いま：copies＋copy_rotate — ISF：Radial Replicate — 見た（10/28、SECTION=Q）：copies 8・copy_turn 360・size 0.28。写しは同じ場所で回って重なり、円周に並ぶ一文にはなっていない。右の写真は変わらない
- [ ] **Random Displacement**（有る）：Automatically moves the layer smoothly but randomly over time — いま：ふるまい noise — 見た（10/28、SECTION=AA）：ふるまい noise を x に（minimum -0.8、maximum -0.35、frequency 1）。0.05・0.3・0.55・0.8 秒で左は同じ向きに滑らかに寄るだけで、不規則には見えない。右の気球は 4 枚ともそのまま（段の右 0）。出典: https://cavalry.studio/docs/nodes/behaviours/noise/ と `filament/doc_relations.h`。ライセンス: 文書の参照（Scene Group）。この回にコードは取り込んでいない
- [ ] **Rays**（有る）：Renders light rays from bright parts of the layer, based on the threshold setting. — いま：ISF God Rays — ISF：God Rays — 見た（10/28、SECTION=W）：`effects/God Rays.fs`（既定）。光の筋ではなく薄い重なりで、右にも被った（右 31000）
- [x] **Repeat**（有る）：Dynamically generates multiple copies of the layer. — いま：copies — 済（10/28）：物の copies 3・copy_x 0.2。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Repeat Along Path**（有る）：Dynamically generates multiple copies of the layer along the outline path of the layer immediately beneath. — いま：copy_on・copy_on_offset — 済（10/28）：物の copies 8・copy_on を下の円へ（輪郭の等間隔は Skia の SkContourMeasure、`filament/skpath.cpp` の sk_along）。窓で左の円の輪郭に黄色い四角が 8 つ並び、右の写真はそのままなのを絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=R）
- [x] **RGB Split**（有る）：Splits the red, green and blue channels of the layer, keeping one fixed in position and shifting the other two channels in opposite directions. — いま：効果 rgbshift・chroma — ISF：Chromatic Aberration — 済（10/28）：AbsentMotion の `effects/am_rgbsplit.wgsl`（Apache-2.0、既定 Strength 0.15）。窓で写真 2 枚の左だけが赤と青に分かれるのを絵と数で見た（左 39337、右 8242、`skin/book/integration_test/alight_check_test.dart` SECTION=S3）
- [x] **Scatter Repeat**（有る）：Dynamically generates multiple copies of the layer, scattered randomly around the original layer location. — いま：copies＋scatter_x・scatter_y — ISF：Replicate Random — 済（10/28）：物の copies 6・scatter_x・scatter_y 0.3。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [x] **Solid Color**（有る）：Fills the opaque parts of the layer with a solid color. — いま：物の tint・fill — ISF：Solid Color — 済（10/28）：物の tint 1、黒も白も 3366ff。窓で写真 2 枚の左だけに掛けて見た（`skin/book/integration_test/alight_check_test.dart`、SECTION=F）
- [ ] **Squeeze**（有る）：Distorts the layer by squeezing it along one axis while expanding it along the other axis. — いま：形の squash — ISF：Squeeze — 見た（10/28、SECTION=W）：AbsentMotion の `effects/am_squeeze.wgsl`（Strength 1.2）。中が細く両脇が黒で、軸を縮めてもう一方を伸ばす一文には見えない。右の写真は変わらない
- [x] **Stroke Color**（有る）：Overrides the layer's stroke color. — いま：stroke — 済（10/28）：形の stroke ff2266・stroke_width 36、塗りは 2244aa。窓で左の四角の縁が赤く、右の写真はそのままなのを絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=Q）
- [x] **Swing**（有る）：Automatically rotates the layer back and forth over time. — 済（10/28）：ふるまい oscillator を rotate に（minimum -8、maximum 8、frequency 1、写真の size 0.5）。0.25 秒と 0.75 秒で左が逆向きに傾き、右の気球は両方そのままなのを絵で見た（右 0、`skin/book/integration_test/alight_check_test.dart` SECTION=V）。出典: https://cavalry.studio/docs/nodes/behaviours/oscillator/ と `filament/doc_relations.h`。ライセンス: 文書の参照（Scene Group）。この回にコードは取り込んでいない。SECTION=U の ±18 度は角が右に入った（右 770 と 653）
- [x] **Swirl**（有る）：Warps the layer by rotating it different amounts based on the distance from a center-point, resulting in a vortex or whirlpool-like distortion. — いま：ISF Twirl・形の twist — ISF：Swirl — 済（10/28）：AbsentMotion の `effects/am_twirl.wgsl`（Apache-2.0、既定 Strength 0.1・Radius 0.3）。窓で写真 2 枚の左だけが渦になるのを絵と数で見た（左 63057、右 8351、`skin/book/integration_test/alight_check_test.dart` SECTION=S4）
- [x] **Text Spacing**（有る）：Adjusts the spacing between individual letters or between lines of text in a text layer. — いま：letter_spacing・line_spacing — 済（10/28）：文字の letter_spacing 1.5。窓で左の MOTOLII の字が離れ、右の写真はそのままなのを絵で見た（`skin/book/integration_test/alight_check_test.dart` SECTION=Q）
- [ ] **Text Transform**（有る）：Transforms individual letters, words, lines, or pieces of letters within a specified range of a text layer. — いま：split＋stagger（Cavalry の Animators の形） — 見た（10/28、SECTION=T）：文字の split を letters、copy_y 0.045、size 0.32。字は 1 字ずつ高さが違うが、後ろの字が右の気球に被る（右 2283）
- [ ] **Vignette**（有る）：Darkens the edges of the layer. — いま：効果 vignette・段の vignette — ISF：v002 Vignette — 見た（10/28、SECTION=S3）：AbsentMotion の `effects/am_vignette.wgsl`（既定 Size 0.95・Strength 0.8）。左は 19525 画素変わったが、縁が暗くなる一文としては見えない。SECTION=W の `effects/vignette.wgsl`（three.js、既定 offset 1・darkness 1）も、縁の暗さとしては見えない — 見た（10/28、SECTION=AG）：同じ `am_vignette.wgsl` の Size 0.55・Strength 1・Feather 0.4（どれもファイルの範囲、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/vignette.json ）。角は 3 段階も暗くならず、縁が暗くなる一文には見えない。右の気球は元の絵

## G. 効果以外の機能（32）

- [x] 図形・文字・画像・動画・音のレイヤー（有る）：film・picture・text・shape（音は film） — 済：窓の試験（`workspace_joined_test_2.dart` の Create・Media の棚）
- [x] ベクター描画（ペン）（有る）：いま ペン（Skia でパスを読む） — 済：窓の試験（`workspace_joined_test_2.dart`：押した所に点、最初の点で閉じる、10/28）
- [x] **手描き（フリーハンド）**（無い→有る）：段の道具の Brush（B、Photoshop・After Effects のブラシのキー）。なぞった道を perfect-freehand（MIT、Steve Ruiz、Dart 版 Adil Hanney）が輪郭にし、閉じたパスの形 1 つに（中点を通る 2 次曲線、perfect-freehand の見本どおり）。太さは Thin・Medium・Thick、色はインク（無ければ白）。1 回で取り消せる — 済（10/28）：窓で Brush でなぞり、文書から読んで確かめた（`skin/book/test/workspace_joined_test_2.dart`）
- [ ] グループ（一部）：いま 関係の group（並びだけ）
- [ ] **Elements**（使い回す部品、元とつながる、文字だけ差し替え、Intro・Outro と Freeze・Stretch・Loop の伸ばし方）（無い）
- [x] ヌル（有る）：いま 形の null — 済：窓の試験（`workspace_joined_test_2.dart` の Create：Null）
- [x] カメラ（Z、ズーム距離・画角、焦点ぼけ、フォグ）（有る）：いま camera の distance・zoom・focus・aperture、段の fog — 済：窓の試験（`workspace_joined_test_2.dart`、`workspace_desk_test.dart`）、`check/fog_near.sh`
- [x] カメラを複数置いて切り替える：カメラごとに始まりと長さを決めると、その時に出ている一番手前のカメラで見る（今は 2D の図形もカメラのズームを受ける） — 済（10/28）：`check/camera_switch.sh`
- [ ] カメラに親子づけ（字幕を画面に固定）（有る）：いま parent — 見た（10/28、SECTION=AB）：カメラの x にふるまい oscillator（-0.35〜0.35）。世界の立方体は画面を横に動く。文字 SUB は space を 2.5d にしてそのカメラを parent にした。字幕は画面に残らず、立方体と同じ向きにずれる。2D の右の写真だけが画面に貼り付いたまま。出典: `filament/doc_relations.h` の doc_above（子の場所は親の場所）。ライセンス: この回にコードは取り込んでいない
- [x] 親子づけ（有る）：いま parent — 済：窓の試験（`workspace_joined_test_2.dart` の Parent、`workspace_joined_test_3.dart` の Parent カード）
- [x] **Parenting Helper**（親の回転・大きさを無視・倍に、車輪の自動回転）（無い→有る）：親がある間だけ出る Parent の札。Its turn・Its size（親の回転・大きさをどれだけ受けるか、0〜200%、キー可）、Roll（x か y に進んだ分だけ車輪として回る）と Roll radius。手本：bevy_alight_motion の parenthelper.rs（MIT/Apache-2.0）の式 — 済（10/28）：親を 45° 回して子の棒が 1・0・2 で 45°・水平・90°、親 150% で子の大きさ 0 なら元の大きさを絵で確かめた。札の出方は窓の試験（`skin/book/test/workspace_joined_test_3.dart`）
- [x] 全部の値にキー（有る）：キーの打てる値は内側の一覧（doc_kinds.h の moves）すべて — 済：`check/doc.sh`（キーの打てない値は断る、の確かめ）、窓の試験（キーの帯）
- [x] イーズ 9 種（ベジェ・Bounce・Elastic・Cyclic・Random・Steps・Elastic Steps）（有る）：`filament/interp.h` — 済：`check/interp_twin.sh`・`check/doc.sh`、窓の試験（`workspace_ease_library_test.dart`）。Steps は 10/28 に Alight の 4 段に合わせた
- [x] 曲線エディタとプリセット（有る）：いま Ease の机、利用者のプリセット — 済：窓の試験（`workspace_ease_library_test.dart`、`workspace_desk_test.dart`）
- [x] **曲線のコピー・貼り付け、全部のキーに貼る**（無い→有る）：Ease の机の上の帯に Copy・Paste・Paste All。Copy は今の曲線を窓を開いている間とっておく、Paste は選んだキーへ（プリセットを押した時と同じ道）、Paste All は選んだキーの層の全部のキーの後ろへ、1 回で取り消せる。手本：Alight Motion の Copy・Paste・Paste to All Keyframes — 済（10/28）：窓で押して文書から読んで確かめた（`skin/book/test/workspace_ease_library_test.dart`）
- [x] **レイヤー・見た目・効果のコピー・貼り付け**（無い→有る）：⌘C・⌘X・⌘V（文字を打っていない時）は選んだ物をまるごと（親だけ除く）写して、再生位置に貼る。キーも一緒に動く（Ableton Live が挿入位置に貼るのと同じ）。⌥⌘C・⌥⌘V（Mac の Copy Style・Paste Style）は見た目（効果・色・線・ふるまい）を選んだ物すべてに。何の物か・形・位置は運ばない。手本：Motolii のルック（doc_looks.h、SQLite の ATTACH）、Alight Motion の Paste Style — 済（10/28）：窓でキーを押し、文書から読んで確かめた（`skin/book/test/workspace_joined_test_3.dart`）
- [x] 重ね方（ブレンドモード）（有る）：いま blend 25 種 — 済：窓の試験（`workspace_desk_test.dart` の Blend）
- [x] マスク（有る）：いま clip（alpha・luma、反転、ぼかし） — 済：窓の試験（`workspace_joined_test.dart`・`_3.dart`）、`check/clip_matte.sh`
- [ ] 塗り：単色・グラデーション・**メディア**（一部）：いま fill・gradient。絵で塗るのは無い
- [x] 線・影・**Multi-Border**（一部→有る）：効果の棚の Multi-Border（ISF、Stylize）。3 つまでの縁を前の縁のまわりに。どの物（写真・文字・形）にも。手本：pixi-filters の OutlineFilter（MIT）の行そのまま、3 pass — 済（10/28）：文字に 2 重・3 重を描いて絵で確かめた
- [x] 自分のフォント（有る）：いま ドックのフォント — 済：窓の試験（`workspace_joined_test_2.dart` の Fonts の棚）
- [x] ブックマーク（有る）：いま ロケーター（Ableton どおり） — 済：窓の試験（`workspace_ableton_keys_test.dart` の locators）
- [x] お気に入り（一部→有る）：効果の棚の全部（ファイルの効果も）に ★、★ を付けた物だけの表（★ の札、All の次）。~/.motolii/effect_stars.json に残る（Ease の机の ease_stars.json と同じ）。タイルの枠が ★ を覆って押せなかったのも直した — 済（10/28）：窓で ★ を押し、★ の表に 1 つだけ出てファイルに残るのを確かめた（`skin/book/test/workspace_dock_test.dart`）
- [x] **速度に応じたモーションブラー**（無い→一部）：書き出しの Motion blur（描き出し窓の行、Off / On）。1 コマを中心に前後 1/4 コマ（シャッター角 180°、After Effects の Shutter Angle 180・Phase −90 と同じ）の間を 8 回描いて平均する。Blender の Eevee の書き出しも同じ「何回も描いて重ねる」やり方。まだ：プレビューには出ない — 済（10/28）：速く動く四角の 30 コマ目で、ぼけた画素がブラーなし 45 → あり 10769、止まった四角は 44 のまま（`filament/check/motion_blur.sh`）。窓で On を押して文書に入るのを確かめた（`skin/book/test/workspace_render_dialog_test.dart`）
- [x] **Time Quantization（コマ落とし。ポスタリゼーション時間）**（無い→一部）：物の Frame rate（Play の札、0 は段と同じ滑らかさ）。キーで動く値は、その物の始まりから数えたコマの時刻で読む（doc_stepped：doc_number と doc_literal の両方の読みに）。After Effects の Posterize Time と同じ考え。まだ：映像そのものの再生と、キーでない動き（揺れ・ふるまい）は段のまま — 済（10/28）：窓で 4 コマ/秒にして値が 0.25・0.25・0.5・0.75 と段になるのを確かめた（`skin/book/test/workspace_joined_test_3.dart`）
- [x] **レイヤーを切る**（無い→有る）：⌘E（Ableton Live の Split）とメニューの Split。選んだ物を再生位置で 2 つに：前は再生位置で終わり、後ろはそこから元の終わりまで、映像は前を切った分だけ先から（左端を詰める時と同じ計算）。キーは画面の時刻のまま。1 回で取り消せる。手本：AbsentMotion の Scene.splitAt（Apache-2.0） — 済（10/28）：窓で映像を選んで ⌘E、文書から両方を読んで確かめた（`skin/book/test/workspace_joined_test_3.dart`）
- [x] 動画から音を取り出す（無い→有る）：Edit > Extract Sound。映像の写しを隠して音だけ聞こえる物にし（隠した映像も音は鳴る）、元の映像は Volume 0。名前は「… (sound)」。1 回で取り消せる。映像に Volume（0〜200%、Sound の札）を足した：書き出しは ffmpeg の volume、再生は GStreamer の playbin の volume — 済（10/28）：窓で文書から読んで確かめた（`skin/book/test/workspace_joined_test_3.dart`）、書き出しの音量を ffmpeg の volumedetect で測った（1・0.5・0 で -10.6・-16.7・-91 dB）
- [ ] 作った後でプロジェクトの設定（大きさ・コマの速さ）を変える（一部）：大きさは上の帯の札で（1 回押すと縦横比と解像度の箱、2 回で数字、10/28）。まだ：コマの速さ（内側が 60 の決まり）
- [x] プレビューの拡大・手のひら（有る）：いま トラックパッド 2 本指・ピンチ — 済：窓の試験（`workspace_trackpad_test.dart`）
- [x] 書き出し MP4（H.264）（一部）：書き出しの箱の Resolution・Rate・Quality を ffmpeg にそのまま（10/28） — 済：`check/film_way.sh`
- [ ] **書き出し HEVC・GIF・PNG 連番・静止画・透明**（無い→一部）：HEVC（.mov）・GIF・PNG と JPEG の連番・今のコマの PNG（⇧E）は有る（10/28、`check/film_way.sh`）。まだ：透明（段の絵が不透明）
- [ ] **プロジェクトを包んで共有（リンク・QR）**（無い）（一部）：File › Package Project で書類と素材を 1 つのフォルダに（10/28、`check/pack.sh`）。まだ：リンク・QR（クラウドが無い）


## H. 実機で見つけた根幹の操作（alight_ui.md）（66）

- [ ] 新しい文書の箱：縦横比 6 種＋自由、解像度 7 段、fps 10 段、背景、PROJECT か ELEMENT（一部）：New（⌘N）で新しい書類を開くと、まず大きさの箱（縦横比 5 種と解像度 7 段、自由は 2 回押して打つ）が出る（10/28、`workspace_joined_test_3.dart`）。背景は Room カード。まだ：fps（内側の 60 の決まり）、ELEMENT
- [ ] 文書の設定を後から変える（同じ箱）と、編集した時間（一部）：大きさは同じ箱でいつでも変えられる（10/28）。まだ：fps、編集した時間
- [x] 書き出し：形式ごとの行、空き容量と大きさの見積もり：書き出しの箱の Format の札（MP4・HEVC・GIF・PNG・JPEG）、Folder の横に空き容量（Mac の df）、Quality を選ぶと大きさの見積もり（Mbps × 秒） — 済（10/28）：窓の試験（`workspace_render_dialog_test.dart`）。Auto のときは見積もりを出さない
- [x] 書き出し Video：解像度、H.264 / H.265、fps 3 段、画質 Mbps：書き出しの箱の Format（MP4 H.264・HEVC・GIF・PNG）、Resolution（Full と 2160p〜）、Rate（60・30・24・15）、Quality（Auto と 4〜32 Mbps）。値は書類に残る — 済（10/28）：`check/film_way.sh`、窓の試験（`workspace_render_dialog_test.dart`）
- [x] 書き出し：今のコマを PNG：File メニューの Export Frame・⇧E（Premiere Pro と同じ）。再生位置の絵を文書の隣に「文書名_コマ.png」で — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 書き出し：画像の連番（PNG / JPEG、解像度、fps、画質）：書き出しの箱の Format の PNG・JPEG（名前_0001 から 1 枚ずつ）、Resolution・Rate の行が効く。JPEG の画質は ffmpeg の -q:v 2（決まり） — 済（10/28）：`check/film_way.sh`、窓の試験（`workspace_render_dialog_test.dart`）
- [ ] 書き出し：プロジェクトを素材ごと包んで共有（リンク）（一部）：File › Package Project で書類と素材を 1 つのフォルダに（10/28）。リンクは無い
- [ ] 書き出し：テンプレートとしても書き出す
- [ ] プロジェクト一覧：カードに長さ・縦横比・解像度・大きさ・fps、日付の見出し
- [ ] プロジェクト一覧：並べ替え（名前・日付）、選ぶ・書き出す・複製・消す
- [ ] プロジェクト一覧：Templates・Elements のタブ、XML を読み込む
- [ ] 環境設定：低画質のプレビュー、既定のレイヤーの長さ、音の変換の質（一部）：低画質のプレビューは設定でなく、再生中に描くのが間に合わない時に内側が自動で縮めて描く（Filament の dynamic resolution と同じ考え、止まると元の大きさ：motolii_hand.h の stage_scale）。まだ：既定のレイヤーの長さ（今は書類の終わりまで）、音の変換の質
- [x] 取り消し・やり直しが常に画面にある：上の帯の左右の矢印（押すと 1 歩戻る・進む） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 全部のレイヤーを選ぶ：⌘A（Edit メニューの Select All） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] レイヤーを複製：⌘D（新しい方が選ばれる） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] レイヤーを見た目ごとコピー・貼り付け：⌘C・⌘V（まるごと、再生位置に） — 済（10/28）：G の行と同じ
- [x] 見た目だけ貼る（何を貼るか選ぶ）（別の形で有る）：Edit › Paste Style（⌥⌘V）で見た目を全部、何を貼るかはカードの頭の Copy・Paste でそのカードの値だけ（10/28）。Alight Motion の矢印の先の一覧は写真で灰色、中身が分からないので並びは借りていない
- [x] プリセットとして保存（値とキー）、読み込む：Edit メニューの Save Look（レイヤーの名前で、作り手のフォルダの looks/ に）。Effects の棚に Look の札で出て、2 回押すと選んだ物に当たる（値・効果・キー。形や位置は運ばない）。同梱のルック 10 個も同じ棚に — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 見え方：Pixel、Grid、Solo、カメラ越し、拡大縮小と %：舞台の上の帯の Fit・50%・100%（等倍）と今の %、Grid、Safe、Camera（カメラ越しと自由）。Solo は S の鍵と時間軸 — 済：窓の試験（`workspace_stage_test.dart`、Solo は `workspace_joined_test_3.dart`）
- [ ] ＋の 1 枚に全部：図形・メディア・音・物・テンプレート・手描き・ベクター・文字
- [x] 図形 54 種を絵で選ぶ：Create の棚の Recipes に 54 種（AbsentMotion の shapes.xml と shapes.cpp から）、札はそれぞれの形の絵（flutter_svg） — 済（10/28）：`check/absent_shapes.sh`、窓の試験（`workspace_dock_test.dart`）。まだ：計算する図形の値（半径・尾など）がキーで動かない
- [ ] 手描き：ペン・ブラシ・塗りつぶし・消しゴム・太さ・色
- [x] 空のグループを足す：Create の棚の Null（見えない物、親にして使う） — 済：窓の試験（`workspace_joined_test_2.dart` の Create：Null）
- [ ] Element：使い回す部品、元とつながる、文字だけ差し替え、グループに戻す
- [x] レイヤーを左右・上下に反転：Arrange > Flip Horizontal・Flip Vertical（scale_x か scale_y を −1 倍、キーも一緒に） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 画面に合わせる（Fit・Fill・Stretch）：Arrange > Fit to Picture・Fill Picture・Stretch to Picture（見えている箱を絵に合わせ、真ん中を絵の真ん中に） — 済（10/28）：窓の試験
- [x] クリッピングマスクを作る：舞台の Mask の道具で四角を描く（その物の真後ろに見えない型ができて切り抜く）、または Blending カードの Clip to below で下の物に切り抜く — 済：窓の試験（`workspace_joined_test.dart`・`workspace_joined_test_3.dart`）
- [x] レイヤーのラベルの色：Note カードの Label colour（Ableton Live のクリップの 16 色と auto）。選ぶと時間軸の行・関係の図・舞台の枠が同じ色に。文書の値なので取り消しも複製も効く — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [ ] 塗りの種類 4 つ（なし・単色・グラデーション・メディア）
- [ ] メディアで塗る
- [x] 色：スポイト、小さいピッカー、パレット、自分の色を足す：Colors の棚。スポイトは Wheel の Sample（Mac の NSColorSampler、画面のどこからでも。取った色を選んだ物に）、ピッカーは Wheel、パレットは棚の一覧、自分の色は Saved — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`、スポイトの答えは試験で肩代わり）。実際の画面から取るのは手ではまだ試していない
- [ ] 線の始点・終点の形 10 種と大きさ
- [x] 縁を何重にも足す（Multi-Border）：効果の棚の Multi-Border（pixi-filters の OutlineFilter、3 重まで） — 済（10/28）：G の行と同じ
- [x] 影：色・大きさ・アルファ・位置：Drop Shadow のカード（色 shadow、ぼかし shadow_blur、不透明度 shadow_opacity、位置は距離と角度、絵を引いて決める） — 済：窓の試験（`workspace_drawings_test.dart`：引くと向きと距離、つまみでぼかし）
- [x] 重ね方 28 を 7 分類で畳み、見本の絵で選ぶ：机の Blend。レイヤーごとの札を押すと、25 種を After Effects の 6 分類で並べ、それぞれ 2 つの円の重なりの絵で選ぶ — 済（10/28）：窓の試験（`workspace_desk_test.dart`）
- [x] Mask・Exclude の重ね方：Mask は Clip to below、Exclude はその Matte カードの Invert（下の物の外だけ見える）。alpha・luma とぼかしも — 済（10/28）：`check/clip_matte.sh`（赤い四角を青い円に切り抜き、反転で外だけ）
- [x] 回転をダイヤルで：Transform カードの絵の上のつまみを回す（下と左のつまみで X・Y の傾きも） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 大きさの幅・高さのつなぎ：Transform カードの Scale の横の鎖（入れると縦横が同じ割合で、外すと押した方だけ） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 傾き X・Y と、傾き・変形を戻す：傾きは Deform カードの Skew と Skew axis（どの向きにも傾く）。戻すのはカードの頭の Reset（そのカードの値とキーを初めの状態へ、1 歩で。After Effects の効果の Reset と同じ）。Deform 以外の窓のカードにも同じ Reset — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [ ] こする帯でレイヤーと点を動かす（指で隠れない）（一部）：物は Transform カードの X・Y のつまみを引いて、手で隠さずに動かせる（After Effects・Figma の引いて変える数字）。矢印キーは Ableton どおり時間の方向。まだ：点を同じように動かす
- [ ] 点を順に送って選ぶ、点を足す・消す、曲線の種類、閉じる・開く（一部）：ペン（doc_pen、Illustrator のペンと同じ）で、最初の点に戻ると閉じる、閉じた形の点の上で消す、輪郭の上で足す、引くと曲線の取っ手。まだ：点を順に送って選ぶ、角となめらかの切り替え、閉じた形を開く
- [ ] どの値も左の列に「キー・イーズ」が同じ場所
- [x] 数字は細かい目盛りをこする：どの数字の欄もこすると変わり、こすりながら ⇧ で 10 分の 1、⌃ で 10 倍（Ableton Live の決まり） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] イーズ：反転、前・次のカーブへ：Ease の机の帯。反転は右端の矢印、前・次のカーブは左端の ‹ ›（そのカーブの始まりのキーを選ぶ。端では出ない） — 済（10/28）：窓の試験（`workspace_ease_library_test.dart`）
- [x] イーズ：カーブのコピー・貼り付け、全部のキーに貼る、行き過ぎを許す：Ease の机の帯の Copy・Paste・Paste All（10/28）、グラフの取っ手は枠の上下へも引ける（-0.42〜1.62） — 済：窓の試験（`workspace_ease_library_test.dart`：取っ手を 1.4 まで引くと 0.8 秒で 1 を越える）
- [x] 速さ：亀とウサギのつまみ、どちらの端を止めるか（一部→有る、別の形）：Play カードの Speed と Hold at（start・now・end）。つまみを離すと、選んだ所を止めたまま長さとキーが速さに合わせて伸び縮みする（After Effects の Time Stretch の Hold in place、Pinnacle Studio の速さの基準点と同じ 3 つ）。Alight Motion の 4 つは説明が見つからず、絵だけでは意味を決められないので借りなかった。長さを言っていない物は書類の終わりまでを長さとする — 済（10/28）：窓で end に止めて速さを上げ、終わりが 2 秒のまま・長さ 2÷速さ・キーも終わりを中心に縮むのを確かめた（`skin/book/test/workspace_joined_test_3.dart`）。sweep_test の全部のつまみも通る
- [x] 帯を左で詰める・切る・右で詰める：⌥[（始まりを再生位置へ）・⌘E（切る）・⌥]（終わりを再生位置へ）。Final Cut Pro の Trim Start・Trim End と同じ鍵。Animation メニューにも — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [ ] 文字：揃え・フォント・大きさ・色を 1 段に（一部）：Text カードに文字・Font・Size・Alignment、色は Fill カード。1 段ではない
- [x] フォント：最近使ったもの、太さごと、星：Fonts の棚の Recent（最近当てた 8 個、新しい順、次に開いても残る、10/28）、Starred、太さは CoreText の読む bold・light の札 — 済：窓の試験（`workspace_joined_test_2.dart`）
- [x] フォント：分類（Serif…）、言語、並べ替え、読み込む：Fonts の棚の札。分類は CoreText の serif・sans・script・mono など、言語は CoreText の対応言語からラテン以外（japanese・korean・chinese・russian・greek・arabic・hebrew・thai・hindi、10/28）、並べ替えは名前と Recent（新しい順）、読み込みは ~/Library/Fonts や書類の隣に置く — 済：窓の試験（`workspace_joined_test_2.dart`）
- [x] 複数選ぶと上の帯が変わり、数が出る：舞台の上の帯が「N picked」と結びつけ（関係）の札に変わり、下の帯も「N picked」（先に 1 つ選んでいても、10/28） — 済：窓の試験（`workspace_joined_test_3.dart`）
- [ ] グループにする・グループにしてマスク・グループにして抜く（一部）：⌘G で選んだ物を、真ん中に置いた Null（名前 Group）にまとめて親子づけ（Figma のグループ、箱の中に入れるプリコンプではない、10/28）。まだ：グループにしてマスク・抜く
- [x] 時間の揃え：始まりを伸ばす・再生位置へ寄せる・左揃え・ずらして並べる・右揃え：Animation メニューの Align Starts・Align Ends・Move to Playhead・Stagger・Extend Start to Playhead（選んだ物に、選んだ順で。動かした物はキーも一緒。始まりを伸ばすは Final Cut Pro の Extend Edit と同じく両向き、終わりは動かない） — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 画面の揃え：左・中・右・上・中・下・等間隔 2 種：Arrange > Align Left〜Bottom・Spread Across・Spread Down（Keynote・Figma の Align：1 つなら絵に、いくつかならその箱に、3 つ以上を等間隔に） — 済（10/28）：窓の試験
- [x] グループに入って編集、パンくず：要らない作りにした。グループは Null と親子づけ（⌘G、10/28）で、中の物は時間軸にそのまま並び、そのまま選んで直せる（Figma と同じ）
- [x] グループの設定：幅・高さ・背景・Precompose・伸び方（Freeze・Stretch・Loop・Loop & Stretch・Blank）：持ち込まない（After Effects のプリコンプ由来の箱の設定。Motolii のグループは箱を作らない）
- [x] カメラ：遠近、画角とズーム：カメラのカード（Projection：遠近と正投影）と Depth の机（焦点距離＝ズーム） — 済：窓の試験（`workspace_joined_test_2.dart`・`workspace_desk_test.dart`）
- [x] カメラ：焦点ぼけ（距離・深度・強さ）：カメラの focus（距離）・aperture（深さ、f 値）・blur_level（強さ） — 済：窓の試験（`workspace_joined_test_2.dart` の Depth の机）
- [ ] カメラ：フォグ（色・近い・遠い）（一部）：Lens カードの Fog（濃さ）・Fog near（始まり、10/28、`check/fog_near.sh`）・色。まだ：遠い（Filament の霧は始まりと濃さで決まり、終わりの距離は無い）
- [ ] 初めての所で押す場所を吹き出しで指す
- [x] 効果の検索（説明文の中まで）：Effects の棚の検索が名前と、効果の作り手の説明（頭の --about:。AbsentMotion の description 126 本、ISF の DESCRIPTION）を読む。下の欄にも説明 — 済（10/28）：窓の試験（`workspace_dock_test.dart`、「checkerboard」で Checker）
- [ ] 効果のおすすめ、分類、動く絵
- [ ] 効果の大きな動く見本・説明・タグ・Guide
- [ ] 効果ごとのプリセット、お気に入り
- [ ] 効果の札：畳む、詳しく、既定に戻す、複製、コピー、消す（一部）：窓のカードは名前を押して畳む、Advanced で詳しく、頭の Copy・Paste（そのカードの値だけを別の物に、10/28）・Reset。Remove（効果のカードだけ。棚で押して外すのと同じ外し方、1 回で取り消せる、10/28、`skin/book/test/workspace_fx_order_test.dart`）。まだ：複製（同じ効果を 2 つ持つと値が 1 組しか無いので、効果ごとに値を分ける仕組みが先）
- [x] レイヤーの種類ごとの帯の色（別の形で有る）：Motolii の帯は物ごとに違う色（Ableton のクリップの色、Note カードの label で選べる）。関係図・ステージの枠・Ease の線で隣の物を見分けるのに使っているので、種類で揃えない。種類は帯の頭の種類の札（WtKindChip の印）で分かる

## I. アプリだけにある効果（公式ガイドに無い 16）（16）

- [ ] Advanced Chroma Key
- [ ] Chromatic Zoom Blur
- [ ] Color Balance — 見た（10/28、SECTION=AD）：`effects/am_colorbalance.wgsl`（AbsentMotion の colorbalance、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/colorbalance.json 、Amount 1。ファイルの影の色へ寄る）。左の暗い草は青くなる。その青い角が右の気球の左端に被る。右の気球の本体は元の絵
- [ ] Energy Beam
- [ ] Grunge
- [ ] Key Spill Cleaner
- [ ] Luma Stamper
- [ ] Matte Fringe
- [ ] Page Curl — 見た（10/28、SECTION=AD）：`effects/am_curl.wgsl`（AbsentMotion の curl、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/curl.json 、Offset 0.2・Radius 40。どちらもファイルの範囲）。端が円に巻く絵には見えない。右の気球は元の絵
- [ ] Parenting Helper
- [ ] Raster Transform — 見た（10/28、SECTION=AD）：`effects/am_rastertransform.wgsl`（AbsentMotion の rastertransform、Apache-2.0、https://github.com/Comett1225/AbsentMotion/blob/0f5d891b7e6fd7260ca6191b4eafef23141230c8/app/src/main/assets/effects/rastertransform.json 、Scale 1.6・Offset x 80。どちらもファイルの範囲）。左は大きくなり右へ寄るが、その絵が右の気球まで被る
- [ ] Ring
- [ ] Scale Assist
- [ ] Scatter Field
- [ ] Streak Strips
- [ ] Warp Blur


## J. 写真・動画・音（実機、alight_ui.md）（16）

- [ ] Media のタブに見本の素材（Sample Media）
- [ ] アルバムの切り替えと数
- [x] 写真は足す前に長さを決める（無い→有る）：Media の棚で写真を選ぶと、下の欄に「for − 2 s +」（0.5・1・2・3・5・10 秒と End、既定は Alight Motion と同じ 2 秒、次の写真にも残る）。置くとその長さ、何枚かまとめて置く時も同じ長さで並ぶ。置くのと長さは 1 回で取り消せる — 済（10/28）：窓で PNG を置いて 2 秒、End で書類の終わりまでを確かめた（`skin/book/test/workspace_joined_test_3.dart`）
- [x] 動画は足す前に使う範囲を切る（再生できる）（無い→一部）：Media の棚で動画を選ぶと、下の欄に use と細い帯（左右の端を引く）と秒の範囲。置くとその範囲の頭から（begin）その長さで、置くのと切るのが 1 回で取り消せる。まだ：帯の上での試し再生 — 済（10/28）：窓で左端を真ん中まで引いて置き、begin と長さが半分になるのを確かめた（`skin/book/test/workspace_joined_test_3.dart`）
- [x] 写真・動画を複数選んで足す：Media の棚で ⌘ を押しながら選ぶ（Finder と同じ）と下の欄が「Place N」。動画はそれぞれの長さ（Discoverer）で、選んだ順に続けて並ぶ。写真は長さが無いので同じ所に（10/28） — 済：窓の試験（`workspace_joined_test_3.dart`）
- [ ] 写真を図形と同じ仕組みで（塗り・縁・影が効く）
- [ ] 写真の切り抜きの大きさと取っ手（角・辺）
- [x] 動画の帯にコマの絵：動画の帯を 6 つに分け、それぞれの真ん中で流れる絵（begin・speed・逆向きも）を内側が GStreamer で小さく作り、帯いっぱいに並べる — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 音量のつまみにキー・イーズ（無い→有る）：動画の Volume にキーとイーズが打てる。再生中は今の時刻の大きさで鳴り（heard.h、GStreamer の playbin の volume を毎コマ）、書き出しは 1/30 秒ごとの大きさを ffmpeg の asendcmd で volume に伝える — 済（10/28）：0 秒で 0・2 秒で 1 のキーで書き出し、キーなしより早い所が −12.6 dB、遅い所が −0.1 dB（`filament/check/volume_keys.sh`）
- [x] 動画から音を別のレイヤーに取り出す — 済（10/28）：G の「動画から音を取り出す」と同じ（Edit > Extract Sound）
- [ ] Convert to Outline（輪郭に変換）
- [x] Media Info（大きさ・fps・長さ・形式・ファイルの大きさ）：Media の棚で選ぶと下の欄に「幅 × 高さ · fps · 秒 · 形式 · 拡張子 · ファイルの大きさ」（GStreamer の Discoverer、10/28） — 済：窓の試験（`workspace_dock_test.dart`）
- [x] 数字の欄を押すと数字のキーパッド（＋/−・小数点）：Mac ではキーボードがその役。数字の欄を押して数字を打つと、その数字から打ち始める（Ableton Live の決まり）、Delete で既定に — 済：窓の試験（`workspace_ableton_fields_test.dart`）
- [ ] 音の棚（曲・アルバム・アーティスト・ジャンル）
- [x] 曲を試しに鳴らしてから足す：Media の棚で音のファイル（絵の無いもの、Discoverer で見分ける）を選ぶと、下の欄の絵が ▶。押すと Mac の afplay が鳴らし、もう一度押すか他を選ぶと止まる（10/28） — 済：窓の試験（`workspace_dock_test.dart`）
- [x] 音の帯に波形：動画の帯にその音の大きさ（内側が別スレッドで読み、帯の見えている所に 1 コマ 1 つ）、拍も定規に — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）


## K. 効果の札とタイムラインの絵（実機）（6）

- [ ] 効果が 2 つ以上なら札の一覧（入り切りの目・並べ替え）
- [ ] 効果の小見出しを畳む
- [ ] 効果ごとに複数のプリセット（動きつきも）
- [ ] 効果の最初の値を「掛かって見える値」に
- [x] タイムラインの帯にコマの絵：動画の帯を 6 つに分け、それぞれの真ん中で流れる絵（begin・speed・逆向きも）を内側が GStreamer で小さく作り、帯いっぱいに並べる — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [x] 音の帯に波形：動画の帯にその音の大きさ（内側が別スレッドで読み、帯の見えている所に 1 コマ 1 つ）、拍も定規に — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）


## L. 隠れた操作とタイムライン・Element（実機。Alight Motion で長押しに隠れている物は「無い」扱い。Motolii は見える所に置く）（7）

- [x] アンカーポイントを動かす（Alight Motion は隠れている。Motolii は見える所に）：Transform カードの下の Anchor カード（立方体の点を押す・数字） — 済：窓の試験（`workspace_joined_test_2.dart` の Anchor）
- [x] Z を動かす帯：Transform カードの Position の Z（2.5D と 3D の物で出る、こすって動かす） — 済：窓の試験（`workspace_joined_test_2.dart` の Space）
- [x] 値を動かすと次のキーが自動で入る：キーのある値を動かすと、その時点にキーが入る — 済：窓の試験（`workspace_joined_test.dart` の Trim：手を離した所にキー）
- [ ] 有料の機能を使うと書き出しに鍵（Motolii は真似しない）
- [ ] Element の伸ばし方の印（イントロの終わり・アウトロの始まり）
- [ ] プロジェクトの絵にするコマを決める
- [ ] 隠れた操作を作らない（長押しだけの操作は、メニューか右クリックにも出す）


## M. キャンバスで物を動かす（実機）（6）

- [ ] 掴んだ取っ手だけ光らせ、ほかを消す
- [x] 動かしている最中に数字を出す（Alight Motion に無い）：舞台で物を動かす間、手の横に小さな札で、運ぶなら位置、角なら幅 × 高さ、回すなら角度、中心点ならその位置（Figma と同じ、10/28）。下の帯の数字も同じく動く — 済：窓の試験（`workspace_joined_test_3.dart`）
- [ ] 吸い付きとガイドの線（Alight Motion に無い）
- [x] 反対側を止めて伸ばす（Alight Motion は真ん中からだけ）：舞台の角の取っ手を引くと反対の角（辺の取っ手なら反対の辺）が止まる。⌥ を押しながらだと中心点から（Figma・Keynote と同じ、10/28） — 済：窓の試験（`workspace_joined_test_3.dart`、回した物でも）
- [ ] キャンバスに動きの道筋とその取っ手（Alight Motion に無い）（一部）：道筋は出る（10/28）。まだ：その取っ手で道を曲げる
- [x] 再生位置で切って後ろ半分を別のレイヤーに：⌘E（Split） — 済：窓の試験（`workspace_joined_test_3.dart`）


## N. マスク（実機）（7）

- [x] クリッピングマスク：すぐ下のレイヤーの形で切る（Create / Release、行に ↘ の印）：Blending カードの Clip to below（入れる・外す）、時間軸の行の名前の前に ↘（10/28） — 済：窓の試験（`workspace_joined_test.dart`）
- [x] クリッピングマスク：型の塗りは消え、型を動かすと切る場所が動く：舞台の Mask の道具で作る型は見えない（opacity 0）、型は物なので動かせば切る所も動く — 済：窓の試験（`workspace_joined_test_3.dart`）
- [ ] グループにしてマスク：グループの一番上が型、その下を切る
- [ ] グループにして抜く：一番上と重なる所を抜く
- [x] グループを解く（Ungroup）：⌘⇧G（Figma と同じ）。物は今の場所のまま、まとめていた Null が消える — 済（10/28）：窓の試験（`workspace_joined_test_3.dart`）
- [ ] 図形の取っ手で角の数・角の丸みをキャンバスで変える
- [x] 取り消しの履歴を開き直しても残す（Alight Motion は消える）：取り消しは書類（SQLite）に残る — 済：窓の試験（`workspace_joined_test.dart`：大きさを変えて開き直した後に取り消すと戻る）


## O. 有料の中身：イーズと親（実機、無料体験）（12）

- [ ] イーズの既定の数字と取っ手を Alight Motion に合わせる：Bounce
- [ ] 同：Elastic（今は大きくずれている）
- [ ] 同：Cyclic
- [ ] 同：Random（今はおとなしすぎる）
- [x] 同：Steps（今は 6 段、Alight Motion は 4 段）：既定を 4 段に（0.25・0.5・0.75 で 4 分の 1 ずつ、終わりで 1。実機の絵 135 と同じ、10/28） — 済：`check/doc.sh`
- [ ] 同：Elastic Steps
- [ ] イーズの取っ手を曲線の上で直接動かす（黄＝形、白＝始めと終わり）
- [x] ベジェの 4 つのプリセット（有る）：Ease の机のプリセットに Linear・In・Out・In-out（Alight Motion の直線・加速・減速・加速して減速と同じ形、CSS の ease-in・ease-out・ease-in-out の数）、ほかに Ease・Back・Hold。曲線の白い取っ手 2 つは引ける（`skin/book/lib/workspace/desk_ease.dart` の easePresets、`desk_ease_graph.dart`）
- [x] 親：一覧で選ぶ（None・自分は灰色・音は出ない）：Transform カードの Parent（None と他の物の名前、自分は出ない） — 済：窓の試験（`workspace_joined_test_2.dart` の Parent）
- [x] 親：子を選ぶと親の外枠を出す：親のある物を選ぶと、舞台に親の枠が灰色の線で（10/28） — 済：窓の試験（`workspace_joined_test_3.dart`）
- [x] 動きの道筋を細い線と向きの矢印で出す：選んだ物が動くと、舞台にその道筋（内側の位置の計算 motolii_path、揺れも含む）を細い線と矢印で（10/28） — 済：窓の試験（`workspace_joined_test_3.dart`）
- [x] 書き出し GIF：書き出しの箱の GIF（ffmpeg の palettegen・paletteuse、15 コマ毎秒） — 済：`check/film_way.sh`（10/28）、窓の試験（`workspace_render_dialog_test.dart`）

合計：337 箱。
