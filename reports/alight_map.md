# Alight Motion と照らす（10/25）

Alight Motion は**下限**（ここまでは揃える線）で、ゴールではない。表は 2 段にした：Alight Motion にあって Motolii に無いもの（揃える分）と、Motolii にあって Alight Motion に無いもの（もう先にある分）。全部の機能は同じ重さで数える。

出典：Alight Motion の公式の効果ガイド（guide.alightmotion.com/effects、169 本、各ページの最初の一文）、公式ヘルプ（support.alightmotion.com：Camera Objects Guide、Layer Parenting and Null Objects、Elements: The Complete Guide、Retiming Marks、Animation Easing Curves、Preview Pan and Zoom、copy and paste effects、existing project settings）、alightcreative.com、App Store（6.2.60）。Motolii は `filament/doc_kinds.h` の値、`filament/effects/`（ISF 33・WGSL 18・点群 4）、`filament/recipes/`（45）、ASSETS.md。

## 答え：効果が足りないのか

| | 数 |
|---|---|
| Alight Motion の効果 | 169 |
| Motolii に有る | **33**（20%） |
| 一部 | **56**（33%） |
| 無い | **80**（47%） |
| 無いもののうち、手元の ISF-Files（`research_notes/isf/ISF-Files`、327 本、MIT）に近いものがある | 36 |
| 一部のもののうち、同じく ISF-Files に近いものがある | 31 |

**効果は足りない。** ただ、足りなさは 3 つの形にまとまる：

1. **物ごとに掛からない（9 本）**：明るさ・色温度・Color Tune・彩度・ハイライトとシャドウ・ブルーム・レンズフレアは、段（画面全体）の値にしかない。Alight Motion は全部レイヤーごと。
2. **形にしか掛からない（6 本）**：曲げ・膨らみ・波・荒らしは形の値で、写真・動画・文字には掛からない。
3. **絵を立体の面に貼れない（4 本＋α）**：立方体・球・輪はあるが、レイヤーの絵を面に貼る（Box・Cube・Cylinder・Torus…の faux-3D 16 本）が無い。

残りの「無い」80 本のうち 36 本は、ISF-Files の .fs を効果のフォルダに置くだけで近いものが出る（置けば棚に出る仕組みはもうある：ASSETS.md の ISF の行）。書かずに埋まる。

## 効果の行（169）

| Alight Motion | 何をする（公式の一文） | Motolii | いまの Motolii | ISF-Files（手元、MIT）の近いもの |
|---|---|---|---|---|
| [360º Reorient Sphere](https://guide.alightmotion.com/effects/360-reorient-sphere) | Reorients a 360º photo or video in equirectangular format. | 無い |  |  |
| [360º Viewer](https://guide.alightmotion.com/effects/360-viewer) | Projects a 360º photo or video onto a flat surface, as if viewed from within the 360º sphere. | 一部 | 部屋（environment）に正距円筒の絵を貼り、environment_rotate で回す。平らな物の効果としては無い |  |
| [Auto-Shake](https://guide.alightmotion.com/effects/auto-shake) | Moves the layer randomly over time within specified parameters. | 有る | ふるまい noise を x・y に | Shake |
| [Bend](https://guide.alightmotion.com/effects/bend) | Bends the layer along a circular arc. | 一部 | 形の bend（shape だけ）。写真・動画・文字には無い |  |
| [Blink](https://guide.alightmotion.com/effects/blink) | Automatically adjusts the opacity of the layer, alternating between its normal value and zero over time. | 一部 | ふるまい oscillator を opacity に＋間 Steps |  |
| [Block Dissolve](https://guide.alightmotion.com/effects/block-dissolve) | Randomly removes grid-aligned rectangular pieces from the layer by making them transparent. Keyframe animation can be applied to remove progressively more and more rectangles over time. | 無い |  | Random Squares Mask |
| [Block Noise](https://guide.alightmotion.com/effects/block-noise) | Multiplies randomly generated rectangular blocks of noise with the existing colors of opaque parts of the layer. | 無い |  | JPEG Block Corruption |
| [Box](https://guide.alightmotion.com/effects/box) | Renders a faux-3D box using the layer as a surface texture. | 一部 | 立体 cube（キットの形）。物の絵を面に貼るのは無い |  |
| [Box Blur](https://guide.alightmotion.com/effects/box-blur) | Blurs the layer by averaging the colors of nearby pixels in a surrounding rectangular area (hence "box" blur). | 一部 | 物の blur（ガウス） | Fast Blur |
| [Brightness / Contrast](https://guide.alightmotion.com/effects/brightness-contrast) | Adjusts the brightness or contrast of the layer. | 一部 | 段の exposure・contrast（全体だけ、物ごとは無い） | Color Controls |
| [Bump Map](https://guide.alightmotion.com/effects/bump-map) | Uses a height map to model bumps or perturbations in the surface of the layer, then applies simulated lighting and shading. | 無い |  | Emboss |
| [Channel Remap (HSV)](https://guide.alightmotion.com/effects/channel-remap-hsv) | Remaps the hue, saturation, value, and alpha channels in the layer. | 無い |  | RGBtoHSV / HSVtoRGB |
| [Channel Remap (RGB)](https://guide.alightmotion.com/effects/channel-remap-rgb) | Remaps the red, green, blue, and alpha channels in the layer. | 無い |  | RGBA Swap |
| [Checker](https://guide.alightmotion.com/effects/checker) | Renders a checkerboard pattern in the opaque parts of the layer. | 無い |  | Checkerboard |
| [Chroma Key](https://guide.alightmotion.com/effects/chroma-key) | Keys out all pixels similar to a particular color, making those parts of the layer transparent. Useful for greenscreening or bluescreening. | 一部 | 動画の keyed（四隅の色を抜く。色は選べない） | Chroma Mask |
| [Circular Ripple](https://guide.alightmotion.com/effects/circular-ripple) | Distorts the layer in a pattern concentric circles around a center point, similar to the ripples caused by a pebble dropped in water. | 一部 | ISF Circle Splash Distortion | Ripples |
| [Clouds](https://guide.alightmotion.com/effects/clouds) | Renders a procedurally generated cloud pattern in the opaque parts of the layer. | 無い |  | Simplex Noise |
| [CMYK Halftone Dots](https://guide.alightmotion.com/effects/cmyk-halftone-dots) | Replaces continuous tones in the layer with a pattern of dots of varying sizes for each CMYK color channel, similar to the process used in magazine or comic book printing. | 有る | ISF CMYK Halftone | CMYK Halftone |
| [Color Temperature](https://guide.alightmotion.com/effects/color-temperature) | Corrects a photo or video so it appears as if illuminated by a light source with a different color temperature. Useful for changing the mood of a scene or correcting for the wrong illumination or whit | 一部 | 段の temperature（全体だけ） | White Point Adjust |
| [Color Tune](https://guide.alightmotion.com/effects/color-tune) | Smoothly tune the color of dark and light parts of a photo or video segmented by lift, gamma, and gain. Use this to achieve various color "looks", to change the mood of the video, or to match the colo | 一部 | 段の slope・offset・power と影・中間・明るい所の色（全体だけ） | LGG |
| [Colorize](https://guide.alightmotion.com/effects/colorize) | Changes the color of the layer, keeping the original luminance. | 有る | 物の tint |  |
| [Contour Gradient](https://guide.alightmotion.com/effects/contour-gradient) | Renders a gradient around the contours of the opaque parts of the layer. | 無い |  |  |
| [Contour Lines](https://guide.alightmotion.com/effects/contour-lines) | Draws one or more lines around the contours of the opaque parts of the layer. | 一部 | 形・文字の stroke（輪郭 1 本） |  |
| [Contour Strips](https://guide.alightmotion.com/effects/contour-strips) | Renders strips of thin paper-like material, layered front to back, using the layer luminance or opacity as a height map for each strip. | 無い |  |  |
| [Copy Background](https://guide.alightmotion.com/effects/copy-background) | Copies the background into the layer, effectively turning the layer into an adjustment layer. Effects applied to the layer after this will apply to the background pixels covered by the opaque parts of | 一部 | 段の script（画面全体に .wgsl）。下だけに掛ける調整レイヤーは無い |  |
| [Count Up/Down](https://guide.alightmotion.com/effects/count-up-down) | Replaces existing numbers in a text layer with the same number added to or multiplied with a fixed value. Useful for creating animations that count up or down. | 無い |  |  |
| [Cube](https://guide.alightmotion.com/effects/cube) | Renders a faux-3D cube using the layer as a surface texture. | 一部 | 立体 cube。絵を面に貼るのは無い | cube |
| [Curl](https://guide.alightmotion.com/effects/curl) | Deforms the layer by curling it into a circular shape from one end. | 無い |  | Inverted Page Curl |
| [Cylinder](https://guide.alightmotion.com/effects/cylinder) | Renders a faux-3D cylinder using the layer as a surface texture. | 無い |  |  |
| [Dark Glow](https://guide.alightmotion.com/effects/dark-glow) | Creates dark blooms around darker areas within a photo or video layer based on a luminance threshold. | 無い |  |  |
| [Directional Blur](https://guide.alightmotion.com/effects/directional-blur) | Blurs the layer by averaging nearby pixel colors along a specified angle, similar to the blur caused by motion in a fixed direction. | 一部 | 物の blur の向き（横・縦だけ、角度は無い） | Linear Blur |
| [Displacement Map](https://guide.alightmotion.com/effects/displacement-map) | Displaces pixels in the background based on the color of pixels in the layer. | 無い |  | Displace |
| [Dissolve](https://guide.alightmotion.com/effects/dissolve) | Randomly removes pixels in the layer. This effect can be animated to create a pixel dissolve transition. | 一部 | 点群の dissolve だけ | Noise |
| [Dots](https://guide.alightmotion.com/effects/dots) | Draws a pattern of random dots in the opaque parts of the layer. Each dot is placed randomly, but the randomness is controlled so that the overall distribution of dots when taken together is approxima | 無い |  | Polka Dots Curtain |
| [Drawing Progress](https://guide.alightmotion.com/effects/drawing-progress) | Clips the beginning and end of the vector outline of the layer so that the outline is only partially drawn. | 有る | 形の trim・trim_begin・trim_offset |  |
| [Echo Keyframes](https://guide.alightmotion.com/effects/echo-keyframes) | Creates one or more copies or "echoes" of the layer as it appeared or will appear at different points in time, based only on keyframe animation applied to the layer. | 一部 | 形の trail・ISF Echo Trace・RGB Trails |  |
| [Edge Glow](https://guide.alightmotion.com/effects/edge-glow) | Detects edges in a photo or video and creates a glow around them. | 一部 | ISF Edge Blowout | Edges |
| [Electric Edges](https://guide.alightmotion.com/effects/electric-edges) | Creates a glowing energy-like outline and optional contour lines around the opaque parts of the layer, which can be animated to look like arcs of electricity. | 無い |  |  |
| [Ellipsoid](https://guide.alightmotion.com/effects/ellipsoid) | Renders a faux-3D ellipsoid using the layer as a surface texture. | 一部 | 立体 ball。絵を面に貼るのは無い |  |
| [Exposure / Gamma](https://guide.alightmotion.com/effects/exposure-gamma) | Adjusts the exposure, gamma curve, and luminance offset of photo and video layers. | 一部 | 段の exposure（全体だけ） | Exposure Adjust / Gamma Correction |
| [Fade In/Out](https://guide.alightmotion.com/effects/fade-in-out) | Automatically animates the layer opacity to fade the layer in at its beginning, and out again at its end. | 一部 | キーで。出入りに自動で付くのは無い | Fade |
| [Feather](https://guide.alightmotion.com/effects/feather) | Softens the edges of the opaque part of the layer by blurring the alpha channel. | 一部 | クリップの clip_feather（切り抜きの縁だけ） |  |
| [Fill Behind](https://guide.alightmotion.com/effects/fill-behind) | Fills the area behind the layer with a modified copy of the layer, with optional adjustments such as blur, saturation, tint, and brightness, providing a convenient way to quickly adapt a photo or vide | 無い |  |  |
| [Find Edges](https://guide.alightmotion.com/effects/find-edges) | Finds edges in a photo or video layer by detecting sharp changes in luminance between nearby pixels. | 無い |  | Edges |
| [Flicker](https://guide.alightmotion.com/effects/flicker) | Randomly flickers the layer by automatically adjusting the opacity over time. | 一部 | ふるまい random を opacity に | RGB Strobe |
| [Flip Layer](https://guide.alightmotion.com/effects/flip-layer) | Flips the layer in 3D along a specified axis. | 有る | rotate_x・rotate_y（2.5D） | Flip H / Flip V |
| [Four-color Gradient](https://guide.alightmotion.com/effects/four-color-gradient) | Overlays the opaque parts of the layer with a gradient made between four individually positioned colors. | 一部 | 形・文字の gradient（2 色） | Corner Colors |
| [Fractal Ridges](https://guide.alightmotion.com/effects/fractal-ridges) | Renders a procedurally generated pattern of turbulent ridges in the opaque parts of the layer. | 無い |  | Ridgelines |
| [Fractal Warp](https://guide.alightmotion.com/effects/fractal-warp) | Distorts the layer using a procedurally generated fractal pattern. Useful for simulating distortions caused by real-world phenomena such as fire, water, and heat. | 一部 | 形の warp・wave_noise（形だけ） | Noise Displace |
| [Gaussian Blur](https://guide.alightmotion.com/effects/gaussian-blur) | Applies a fast, smooth blur to the layer using a gaussian function, reducing noise and details in the layer. | 有る | 物の blur | Multi Pass Gaussian Blur |
| [Glass](https://guide.alightmotion.com/effects/glass) | Renders the layer as if the opaque parts were made of beveled glass, refracting the background and overlaying specular highlights. | 一部 | 立体の ior・dispersion（本物の屈折、立体だけ）。平らな物のガラスは無い | Frosted Glass |
| [Glow](https://guide.alightmotion.com/effects/glow) | Draws a glow around the outside of the opaque parts of the layer. | 有る | 物の glow |  |
| [Glow Scan](https://guide.alightmotion.com/effects/glow-scan) | Renders a blur combined with a smooth glow within a region of the layer defined by a luminance band. | 無い |  |  |
| [Gradient Map](https://guide.alightmotion.com/effects/gradient-map) | Maps colors in the layer onto a gradient based on their lightness. | 一部 | 物の tritone（3 色）・ISF False Color・Trio Tone | Duotone |
| [Gradient Overlay](https://guide.alightmotion.com/effects/gradient-overlay) | Overlays the opaque parts of the layer with a gradient. | 有る | 形・文字の gradient | Linear Gradient |
| [Grid](https://guide.alightmotion.com/effects/grid) | Overlays the opaque parts of the layer with grid lines. | 無い |  | Graph Paper |
| [Grid Repeat](https://guide.alightmotion.com/effects/grid-repeat) | Dynamically generates multiple copies of the layer, arranged in a grid. | 有る | copies＋copy_columns |  |
| [Halftone Dots](https://guide.alightmotion.com/effects/halftone-dots) | Replaces continuous tones in the layer with a pattern of monochrome dots of varying sizes, similar to the process used in newspaper or comic book printing. | 有る | 効果 halftone・dot_screen | Dot Screen |
| [Halftone Lines](https://guide.alightmotion.com/effects/halftone-lines) | Replaces continuous tones in the layer with a pattern of monochrome lines of varying thickness, a variation of the process used in newspaper or comic book printing. | 無い |  | Line Screen |
| [Heart](https://guide.alightmotion.com/effects/heart) | Renders a faux-3D heart using the layer as a surface texture. | 無い |  |  |
| [Hexagon Array](https://guide.alightmotion.com/effects/hexagon-array) | Covers opaque parts of the layer in an array of hexagons. | 無い |  |  |
| [Hexagon Tile Rotate](https://guide.alightmotion.com/effects/hexagon-tile-rotate) | Divides the layer into a grid of hexagonal tiles and rotates the pixels in each tile independently. | 無い |  |  |
| [Hexagon Tile Shift](https://guide.alightmotion.com/effects/hexagon-tile-shift) | Divides the layer into a grid of hexagonal tiles and shifts the contents of each tile in a random direction. | 無い |  |  |
| [Hexagon Tiling](https://guide.alightmotion.com/effects/hexagon-tiling) | Repeats the layer in a pattern of hexagonal tiles, filling the screen. | 無い |  | Hexagonalize |
| [Hexagonal Prism](https://guide.alightmotion.com/effects/hexagonal-prism) | Renders a faux-3D hexagonal prism using the layer as a surface texture. | 無い |  |  |
| [Highlights and Shadows](https://guide.alightmotion.com/effects/highlights-and-shadows) | Independently adjusts the intensity of the lightest and darkest parts of the layer. | 一部 | 段の影・明るい所の色（全体だけ） | Color Levels |
| [Hollow Box](https://guide.alightmotion.com/effects/hollow-box) | Renders a faux-3D hollow box, using the layer as a surface texture, with sides that can be turned on or off, punched out with preset shapes, or offset outward. | 無い |  |  |
| [Hot Color](https://guide.alightmotion.com/effects/hot-color) | Remaps a grayscale image (or luminance values of a color image) into an intense color ramp based on two selected hues. | 一部 | ISF False Color | Thermal Camera |
| [Hue Shift](https://guide.alightmotion.com/effects/hue-shift) | Rotates the color hues in the layer. | 無い |  | Multi Hue Shift |
| [Inner Blur](https://guide.alightmotion.com/effects/inner-blur) | Blurs the opaque parts of the layer without blurring the opacity. | 無い |  |  |
| [Inner Glow](https://guide.alightmotion.com/effects/inner-glow) | Draws a glow along the inside edges of the opaque parts the layer. | 無い |  |  |
| [Inner Pinch/Bulge](https://guide.alightmotion.com/effects/inner-pinch-bulge) | Distorts the layer by pinching it towards the center or expanding it outwards, without affecting the edges or opacity of the layer. | 無い |  |  |
| [Invert](https://guide.alightmotion.com/effects/invert) | Inverts the layer's red, green, or blue channels independently. | 無い |  | Color Invert / RGB Invert |
| [Iridescence](https://guide.alightmotion.com/effects/iridescence) | Renders a flowing iridescent pattern in opaque parts of the layer. | 無い |  |  |
| [Kaleidoscope](https://guide.alightmotion.com/effects/kaleidoscope) | Creates symmetric radial reflections of the layer. | 有る | 効果 kaleido・ISF Kaleidoscope Tile | Kaleidoscope |
| [Lens Blur](https://guide.alightmotion.com/effects/lens-blur) | Applies an angular blur to the layer, excluding the central area. | 一部 | カメラの focus・aperture（立体と世界の物だけ） |  |
| [Lens Flare](https://guide.alightmotion.com/effects/lens-flare) | Renders a lens flare in the layer. | 一部 | 段の lens_flare（全体だけ） | Lens Flare |
| [Light Glow](https://guide.alightmotion.com/effects/light-glow) | Creates a soft glow and optional halo around the lighter areas within a photo or video layer based on a luminance threshold. | 一部 | 段の bloom（全体だけ） | Bloom |
| [Lightning](https://guide.alightmotion.com/effects/lightning) | Renders fractal arcs of lightning between two points in the layer. | 無い |  |  |
| [Linear Repeat](https://guide.alightmotion.com/effects/linear-repeat) | Dynamically generates multiple copies of the layer, arranged along a line. | 有る | copies＋copy_x・copy_y |  |
| [Linear Streaks](https://guide.alightmotion.com/effects/linear-streaks) | Creates line-shaped streaks by sliding the layer at an angle and blending together overlapping pixels. | 無い |  |  |
| [Long Shadow](https://guide.alightmotion.com/effects/long-shadow) | Renders long parallel shadows extending from opaque parts of the layer. | 無い |  |  |
| [Luma Key](https://guide.alightmotion.com/effects/luma-key) | Keys out all pixels lighter than or darker than a given threshold, making those parts of the layer transparent. | 一部 | クリップの clip_matte luma（下の物で切る） |  |
| [Magnify Background](https://guide.alightmotion.com/effects/magnify-background) | Makes the layer transparent and magnifies the area behind the layer, as if the layer were a magnifying glass. | 無い |  |  |
| [Mask Blur](https://guide.alightmotion.com/effects/mask-blur) | Blurs the background behind the layer, with the blur strength based on the layer luminance. | 無い |  |  |
| [Matte Choker](https://guide.alightmotion.com/effects/matte-choker) | Expands the transparent parts of the layer, shrinking the opaque parts. | 無い |  | Erode / Dilate |
| [Mirror](https://guide.alightmotion.com/effects/mirror) | Overlays the layer with a copy of itself mirrored along either the horizontal or vertical axis, with optional alpha and blending mode control. | 有る | 効果 mirror | Mirror |
| [Mosaic](https://guide.alightmotion.com/effects/mosaic) | Divides the layer into array of squares and averages the color of each square, resulting in a pixelated look. | 一部 | ISF Multi-Pixellate・Noise Pixellate | Mosaic |
| [Motion Blur](https://guide.alightmotion.com/effects/motion-blur) | Simulates motion blur by blurring the layer based on position, rotation, and scale keyframe animation from the layer's Move & Transform settings, as well as motion from certain other effects that are  | 無い |  | VVMotionBlur 3.0 |
| [Move Along Path](https://guide.alightmotion.com/effects/move-along-path) | Positions the layer at a point partway way along the outline path of the layer immediately beneath. | 一部 | copy_on（写しを別の物の輪郭に）。1 個を道に沿わせて動かすのは写し 1 個で |  |
| [Noise](https://guide.alightmotion.com/effects/noise) | Blends random noise with the opaque parts of the layer. | 有る | 効果 grain・fine_grain | Noise |
| [Octahedron](https://guide.alightmotion.com/effects/octahedron) | Renders a faux-3D regular octahedron using the layer as a surface texture. | 無い |  |  |
| [Offset](https://guide.alightmotion.com/effects/offset) | Offsets the layer content, wrapping at the edges of the bounding box of the layer. | 無い |  | Side Scroller And Flip |
| [Omino Diffusion+](https://guide.alightmotion.com/effects/omino-diffusion) | Renders a photo or video layer with a limited palette using error diffusion, but damping or exaggerating the quantization errors to create intentional distortions. Individual images become unrecogniza | 一部 | ISF Dither-Bayer（並べた点のディザ、誤差拡散は無い） |  |
| [Omino Glass](https://guide.alightmotion.com/effects/omino-glass) | Renders the layer as though it were made of glass, with refraction and a glossy contour, and chromatic aberration based on segmentation of the visible spectrum into seven regions. The layer luminance  | 無い |  |  |
| [Oscillate](https://guide.alightmotion.com/effects/oscillate) | Moves the layer back and forth repeatedly. | 有る | ふるまい oscillator |  |
| [Palette Map](https://guide.alightmotion.com/effects/palette-map) | Maps pixel colors in the layer to the closest matching color in a palette. | 無い |  | Color Schemes |
| [Pinch/Bulge](https://guide.alightmotion.com/effects/pinch-bulge) | Distorts the layer by pinching it towards the center or expanding it outward. | 一部 | 形の pinch・bloat（形だけ） | Bump Distortion |
| [Pixelate](https://guide.alightmotion.com/effects/pixelate) | Decreases the layer resolution so that individual pixels become visible, sampling at the centers of the pixels, and drawing each pixel with an optional vignette. | 一部 | ISF Multi-Pixellate | Pixellate |
| [Polar Coordinates](https://guide.alightmotion.com/effects/polar-coordinates) | Warps the layer by mapping from cartesian coordinates to polar coordinates. | 無い |  |  |
| [Polar Displacement Map](https://guide.alightmotion.com/effects/polar-displacement-map) | Angularly displaces pixels in the background based on the color of pixels in the layer. | 無い |  |  |
| [Posterize](https://guide.alightmotion.com/effects/posterize) | Decreases the number of step in the layer's color channels. | 有る | ISF Posterize | Color Posterize |
| [Precise Box Blur](https://guide.alightmotion.com/effects/precise-box-blur) | Performs a slow but precise box blur of the layer. | 一部 | 物の blur |  |
| [Pulse Opacity](https://guide.alightmotion.com/effects/pulse-opacity) | Automatically pulses the layer opacity over time, based on a sine wave or triangular wave. | 有る | ふるまい oscillator を opacity に |  |
| [Pulse Size](https://guide.alightmotion.com/effects/pulse-size) | Automatically grows and shrinks the layer repeatedly over time, based on a sine wave or triangular wave. | 有る | ふるまい oscillator を size に |  |
| [Pyramid](https://guide.alightmotion.com/effects/pyramid) | Renders a faux-3D regular pyramid with a square base, using the layer as a surface texture. | 無い |  |  |
| [Radial Rays](https://guide.alightmotion.com/effects/radial-rays) | Overlays multiple solid-color radial rays in the opaque parts of the layer. | 無い |  |  |
| [Radial Repeat](https://guide.alightmotion.com/effects/radial-repeat) | Dynamically generates multiple copies of the layer, arranged in a circular pattern. | 有る | copies＋copy_rotate | Radial Replicate |
| [Radial Shadow](https://guide.alightmotion.com/effects/radial-shadow) | Renders long shadows radiating away from a point, based on the opaque parts of the layer. | 無い |  |  |
| [Radial Wipe](https://guide.alightmotion.com/effects/radial-wipe) | Masks out a region of the layer bounded by two radial lines. | 無い |  | Radial |
| [Random Displacement](https://guide.alightmotion.com/effects/random-displacement) | Automatically moves the layer smoothly but randomly over time | 有る | ふるまい noise |  |
| [Random Jitter](https://guide.alightmotion.com/effects/random-jitter) | Automatically jumps the layer randomly back and forth over time, along a selected angle. | 一部 | ふるまい random |  |
| [Raster Extrude](https://guide.alightmotion.com/effects/raster-extrude) | Renders a faux-3D raster extrusion of the layer based on the opaque and transparent pixels within the layer, using the opaque parts of the layer as a surface texture for the front and back of the extr | 無い |  |  |
| [Rays](https://guide.alightmotion.com/effects/rays) | Renders light rays from bright parts of the layer, based on the threshold setting. | 有る | ISF God Rays | God Rays |
| [Repeat](https://guide.alightmotion.com/effects/repeat) | Dynamically generates multiple copies of the layer. | 有る | copies |  |
| [Repeat Along Path](https://guide.alightmotion.com/effects/repeat-along-path) | Dynamically generates multiple copies of the layer along the outline path of the layer immediately beneath. | 有る | copy_on・copy_on_offset |  |
| [Replace Color](https://guide.alightmotion.com/effects/replace-color) | Replaces one color in the layer with another. | 無い |  | Color Replacement |
| [RGB Split](https://guide.alightmotion.com/effects/rgb-split) | Splits the red, green and blue channels of the layer, keeping one fixed in position and shifting the other two channels in opposite directions. | 有る | 効果 rgbshift・chroma | Chromatic Aberration |
| [Ribbon](https://guide.alightmotion.com/effects/ribbon) | Renders a twisted, flowing ribbon of varying width in the opaque parts of the layer. | 無い |  |  |
| [Roughen Edges](https://guide.alightmotion.com/effects/roughen-edges) | Roughens the edges of the opaque parts of the layer. | 一部 | 形の rough（形だけ） |  |
| [Saturation / Vibrance](https://guide.alightmotion.com/effects/saturation-vibrance) | Adjusts the saturation or purity of colors in the layer. | 一部 | 段の saturation・vibrance（全体だけ） | Vibrance |
| [Scatter Repeat](https://guide.alightmotion.com/effects/scatter-repeat) | Dynamically generates multiple copies of the layer, scattered randomly around the original layer location. | 有る | copies＋scatter_x・scatter_y | Replicate Random |
| [Sharpen](https://guide.alightmotion.com/effects/sharpen) | Enhances details in the layer by increasing the contrast wherever there are abrupt differences in the luminance of nearby pixels in the layer. | 無い |  | Sharpen RGB |
| [Simple Starfield](https://guide.alightmotion.com/effects/simple-starfield) | Renders a simple parallax starfield pattern, with a depth component, in the opaque parts of the layer. | 一部 | ISF Hyperspace | Star |
| [Smooth Bevel](https://guide.alightmotion.com/effects/smooth-bevel) | Draws a smooth curved bevel inside the edges of the opaque parts of the layer. | 無い |  |  |
| [Smooth Edges](https://guide.alightmotion.com/effects/smooth-edges) | Smooths the edges of the opaque parts of a layer without affecting the pixel colors within the layer. | 無い |  |  |
| [Soft Glow](https://guide.alightmotion.com/effects/soft-glow) | Creates a soft Gaussian-based glow within a photo or video layer, or around the outside edge of any layer. | 一部 | 物の glow | Glow |
| [Solid Color](https://guide.alightmotion.com/effects/solid-color) | Fills the opaque parts of the layer with a solid color. | 有る | 物の tint・fill | Solid Color |
| [Solid Matte](https://guide.alightmotion.com/effects/solid-matte) | Composites the layer on a solid color matte, filling transparent parts of the layer with a solid background color. | 一部 | 段の background（全体だけ） |  |
| [Spectral Map](https://guide.alightmotion.com/effects/spectral-map) | Maps each color in the layer into a custom spectrum of colors based on the brightness of the original color. | 無い |  | Color Scales |
| [Spherize](https://guide.alightmotion.com/effects/spherize) | Renders a faux-3D sphere, interpreting the layer as an equirectangular projection and using it as a surface texture for the sphere. | 一部 | ISF Sphere Map |  |
| [Spin](https://guide.alightmotion.com/effects/spin) | Automatically spins the layer at a given speed. | 一部 | rotate のキー（ずっと回り続けるふるまいは無い） | Rotate |
| [Spin Blur](https://guide.alightmotion.com/effects/spin-blur) | Applies an angular blur to the layer, as though it were spinning. | 無い |  |  |
| [Spin Streaks](https://guide.alightmotion.com/effects/spin-streaks) | Creates arc-shaped streaks by spinning the layer around a point and blending together overlapping pixels. | 無い |  |  |
| [Spot Color](https://guide.alightmotion.com/effects/spot-color) | Keeps one color while decreasing the saturation of other colors in the layer. | 無い |  | Chroma Desaturation Mask |
| [Squeeze](https://guide.alightmotion.com/effects/squeeze) | Distorts the layer by squeezing it along one axis while expanding it along the other axis. | 有る | 形の squash | Squeeze |
| [Star](https://guide.alightmotion.com/effects/star) | Renders an individual star in the current layer. | 一部 | 形の star（光る星ではない） | Star |
| [Star Polyhedron](https://guide.alightmotion.com/effects/star-polyhedron) | Renders a faux-3D star polyhedron using the layer as a surface texture. | 無い |  |  |
| [Star Prism](https://guide.alightmotion.com/effects/star-prism) | Renders a faux-3D star prism using the layer as a surface texture. | 無い |  |  |
| [Starfield](https://guide.alightmotion.com/effects/starfield) | Renders a complex parallax starfield that can be animated in two dimensions. | 一部 | ISF Hyperspace |  |
| [Stretch Axis](https://guide.alightmotion.com/effects/stretch-axis) | Stretches the layer in an arbitrary direction. | 一部 | 形の stretch・skew（形だけ） |  |
| [Stretch Segment](https://guide.alightmotion.com/effects/stretch-segment) | Slices the layer along an angle and then moves the pieces apart, stretching the pixels between them. | 無い |  |  |
| [Stripes](https://guide.alightmotion.com/effects/stripes) | Renders a pattern of stripes in the opaque parts of the layer. | 無い |  | Random Stripes |
| [Stroke Color](https://guide.alightmotion.com/effects/stroke-color) | Overrides the layer's stroke color. | 有る | stroke |  |
| [Stroke Taper](https://guide.alightmotion.com/effects/stroke-taper) | Tapers the strokes of a freehand drawing layer. | 無い |  |  |
| [Swing](https://guide.alightmotion.com/effects/swing) | Automatically rotates the layer back and forth over time. | 有る | ふるまい oscillator を rotate に |  |
| [Swirl](https://guide.alightmotion.com/effects/swirl) | Warps the layer by rotating it different amounts based on the distance from a center-point, resulting in a vortex or whirlpool-like distortion. | 有る | ISF Twirl・形の twist | Swirl |
| [Text Progress](https://guide.alightmotion.com/effects/text-progress) | Partially displays a text layer, character by character. | 一部 | 文字の split＋出来合いの動き（出る数） |  |
| [Text Randomizer](https://guide.alightmotion.com/effects/text-randomizer) | Randomly changes individual characters in a text layer. | 無い |  | Random Characters |
| [Text Spacing](https://guide.alightmotion.com/effects/text-spacing) | Adjusts the spacing between individual letters or between lines of text in a text layer. | 有る | letter_spacing・line_spacing |  |
| [Text Transform](https://guide.alightmotion.com/effects/text-transform) | Transforms individual letters, words, lines, or pieces of letters within a specified range of a text layer. | 有る | split＋stagger（Cavalry の Animators の形） |  |
| [Three-axis Cross](https://guide.alightmotion.com/effects/three-axis-cross) | Renders a faux-3D three-axis cross using the layer as a surface texture. | 無い |  |  |
| [Threshold](https://guide.alightmotion.com/effects/threshold) | Converts the layer to black and white with a hard cutoff at the specified luminance threshold. | 無い |  | Color Monochrome |
| [Tile Rotate](https://guide.alightmotion.com/effects/tile-rotate) | Divides the layer into a grid of tiles and rotates the pixels in each tile independently. | 無い |  | Grid Flip |
| [Tile Shift](https://guide.alightmotion.com/effects/tile-shift) | Divides the layer into a grid of tiles and shifts the contents of each tile in a random direction. | 一部 | ISF Pixel Shifter |  |
| [Tiles](https://guide.alightmotion.com/effects/tiles) | Repeats the layer in a tile pattern, filling the screen. | 一部 | ISF Quad Tile・Kaleidoscope Tile | Quad Tile |
| [Time Quantization](https://guide.alightmotion.com/effects/time-quantization) | Causes time to pass in steps, but just for this layer. | 無い |  |  |
| [Timecode](https://guide.alightmotion.com/effects/timecode) | Replaces the text in the layer with a running timecode. | 無い |  | Digital Clock |
| [Torus](https://guide.alightmotion.com/effects/torus) | Renders a faux-3D torus using the layer as a surface texture. | 一部 | 立体 ring（絵を面に貼るのは無い） |  |
| [Tunnel](https://guide.alightmotion.com/effects/tunnel) | Renders a round faux-3D tunnel using the layer as the interior surface texture. | 無い |  |  |
| [Turbulence](https://guide.alightmotion.com/effects/turbulence) | Renders smooth patterns of procedurally-generated fractal noise in the opaque parts of the layer, useful for simulating real-world phenomena like clouds, or for use as a luminance map. | 無い |  | Simplex Noise |
| [Turbulent Displace](https://guide.alightmotion.com/effects/turbulent-displace) | Distorts the layer along selected coordinate axes using a random pattern of smooth fractal noise. Distortion may be along one or more axis in either cartesian or polar coordinate space. Useful for sim | 一部 | 形の wiggle・wave_noise（形だけ） | Noise Displace |
| [Unsharp Mask](https://guide.alightmotion.com/effects/unsharp-mask) | Enhances the perceptual sharpness or "acutance" of a photo or video by increasing local contrast near edges, while avoiding changes to areas that were originally smooth. | 無い |  | Unsharp Mask |
| [Vignette](https://guide.alightmotion.com/effects/vignette) | Darkens the edges of the layer. | 有る | 効果 vignette・段の vignette | v002 Vignette |
| [Voronoi Cells](https://guide.alightmotion.com/effects/voronoi-cells) | Renders procedurally-generated pattern of Voronoi cells based on randomly placed points. | 無い |  | Worley Cells |
| [Wave Warp](https://guide.alightmotion.com/effects/wave-warp) | Distorts the layer based on a sine wave. | 一部 | 形の warp・ISF Sine Warp Tile（形だけ） |  |
| [Wipe](https://guide.alightmotion.com/effects/wipe) | Masks out rectangular segments from opposite sides of the layer. Use keyframe animation to create a wipe transition. | 無い |  | Wipe Left / Right / Up / Down |
| [Zoom Blur](https://guide.alightmotion.com/effects/zoom-blur) | Applies a radial blur to the layer, as though zooming in or out. | 無い |  | Zoom |
| [Zoom Streaks](https://guide.alightmotion.com/effects/zoom-streaks) | Creates radial streaks by scaling the layer around a point and blending together overlapping pixels. | 一部 | ISF Hyperspace |  |

## 効果以外の機能

| Alight Motion | Motolii | いまの Motolii |
|---|---|---|
| 図形・文字・画像・動画・音のレイヤー | 有る | film・picture・text・shape |
| ベクター描画（ペン） | 有る | ペン（Skia でパスを読む） |
| **手描き（フリーハンド）** | 無い | |
| グループ | 一部 | 関係の group（並びだけ） |
| **Elements**（使い回す部品、元とつながる、文字だけ差し替え、Intro・Outro と Freeze・Stretch・Loop の伸ばし方） | 無い | |
| ヌル | 有る | 形の null |
| カメラ（Z、ズーム距離・画角、焦点ぼけ、フォグ） | 有る | camera の distance・zoom・focus・aperture、段の fog |
| カメラを複数置いて切り替える | 未確認 | |
| カメラに親子づけ（字幕を画面に固定） | 有る | parent |
| 親子づけ | 有る | parent |
| **Parenting Helper**（親の回転・大きさを無視・倍に、車輪の自動回転） | 無い | |
| 全部の値にキー | 有る | |
| イーズ 9 種（ベジェ・Bounce・Elastic・Cyclic・Random・Steps・Elastic Steps） | 有る | `filament/interp.h`（Alight Motion を手本に作ったもの。出典をそう直した） |
| 曲線エディタとプリセット | 有る | Ease の机、利用者のプリセット |
| **曲線のコピー・貼り付け、全部のキーに貼る** | 無い | |
| **レイヤー・見た目・効果のコピー・貼り付け** | 無い | |
| 重ね方（ブレンドモード） | 有る | blend 25 種 |
| マスク | 有る | clip（alpha・luma、反転、ぼかし） |
| 塗り：単色・グラデーション・**メディア** | 一部 | fill・gradient。絵で塗るのは無い |
| 線・影・**Multi-Border** | 一部 | stroke・shadow。縁取りを何重にもは無い |
| 自分のフォント | 有る | ドックのフォント |
| ブックマーク | 有る | ロケーター（Ableton どおり） |
| お気に入り | 無い | |
| **速度に応じたモーションブラー** | 無い | |
| **Time Quantization（コマ落とし。ポスタリゼーション時間）** | 無い | |
| **レイヤーを切る** | 無い | ⌘E はエンジンに切るが無い |
| 動画から音を取り出す | 無い | |
| 作った後でプロジェクトの設定（大きさ・コマの速さ）を変える | 無い | 隠した欄だけ（screens.md） |
| プレビューの拡大・手のひら | 有る | トラックパッド 2 本指・ピンチ |
| 書き出し MP4（H.264） | 一部 | ffmpeg で決め打ち（export_map.md） |
| **書き出し HEVC・GIF・PNG 連番・静止画・透明** | 無い | |
| **プロジェクトを包んで共有（リンク・QR）** | 無い | |

効果以外 32 行：有る 14、一部 4、無い 13、未確認 1。

## Motolii にあって Alight Motion に無いもの（もう先にある分）

| 何 | どこ |
|---|---|
| 本物の 3D：立体の材質（金属・粗さ・屈折・分散・クリアコート・シーン）、光と影、部屋の光 | `doc_kinds.h` の solid・light・stage（Filament） |
| 3D ファイルとその動き | solid の file・animation |
| 点群・3DGS のスプラットと、その効果 | `points.h`、`effects/*.mjs` |
| 物理（落ちる・ぶつかる） | Jolt、`physics.h` |
| 本物のデータモッシュ（圧縮データそのもの） | `mosh.h` |
| MIDI を畳まずに部品に | `midi.h`、`midi_roll.cpp` |
| After Effects のファイル（Lottie）をそのまま、マーカーごとに | ThorVG |
| ネットの効果を置くだけ（ISF .fs、WGSL） | `isf.h`、`screen.h` |
| 関係（around・row・follow・push）と、ふるまいの落ち方（falloff） | `doc_relations.h` |
| レシピ（JS で写しを並べる、45 本） | `filament/recipes/` |
| 形の操作：ブーリアン・モーフ・オフセット・ジグザグ・四隅ピン・ボクセル | `doc_kinds.h` の shape |
| 音の拍で動かす | `sound.h` |
| Animate all（触った値に全部キー） | `doc.keying` |
| 裏で書き出しの質のコマを溜める | ASSETS.md |
| Ableton どおりのキーと操作、Mac のトラックパッド | `workspace/ableton.dart`、`two_fingers.dart` |

## 揃えるとき、どこから借りるか

- 効果の「無い」36 本：ISF-Files（MIT）の .fs を置く。中身を書かない。
- 物ごとの色（9 本）：段の値と同じ中身（Filament の ColorGrading）を物ごとの別の画に掛ける形がもうある（物の fx、`thing_fx.h`）。新しく書くのは「どの値をそこへ渡すか」だけ。
- faux-3D の面（16 本）：Filament の立体に物の絵を材質として貼る（Filament の見本 `samples/`、テクスチャの材質）。
- Elements の伸ばし方（Freeze・Stretch・Loop・Loop & Stretch・Blank）：Alight Motion の公式の定義をそのまま仕様にする。
- モーションブラー：ISF-Files の VVMotionBlur 3.0、または Filament の時間方向の重ね。
