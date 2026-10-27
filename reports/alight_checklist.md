# Alight Motion チェックリスト（自走用）

[alight_map.md](alight_map.md) の全行を、抜けが無いように 1 行 1 箱にしたもの。表から機械で作り、数を数えて合わせた（効果 169＋機能 32＝201）。実機で見た分を H・I に足した（alight_ui.md）。

- 箱を付ける決まり：Motolii の窓で、その物に掛けて、絵か動画で見て、Alight Motion の公式の一文どおりに動いたら [x]。「有る」も見るまでは [ ]。
- 「一部」は、足りない所を埋めたら [x]。
- 借りた先（ファイル・ライセンス）を行の後ろに書き足す。
- Alight Motion は下限。Motolii にだけあるものは表の最後の段。

## A. 画面全体にしか掛からない色を、物ごとにも（9）

- [x] **Brightness / Contrast**（一部）：Adjusts the brightness or contrast of the layer. — いま：段の exposure・contrast（全体だけ、物ごとは無い） — ISF：Color Controls — 済（10/28）：物の fx に `filament/effects/Color Controls.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [x] **Color Temperature**（一部）：Corrects a photo or video so it appears as if illuminated by a light source with a different color temperature. Useful for changing the mood of a scene or correcting for the wrong illumination or whit — いま：段の temperature（全体だけ） — ISF：White Point Adjust — 済（10/28）：物の fx に `filament/effects/White Point Adjust.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [x] **Color Tune**（一部）：Smoothly tune the color of dark and light parts of a photo or video segmented by lift, gamma, and gain. Use this to achieve various color "looks", to change the mood of the video, or to match the colo — いま：段の slope・offset・power と影・中間・明るい所の色（全体だけ） — ISF：LGG — 済（10/28）：物の fx に `filament/effects/LGG.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [x] **Exposure / Gamma**（一部）：Adjusts the exposure, gamma curve, and luminance offset of photo and video layers. — いま：段の exposure（全体だけ） — ISF：Exposure Adjust / Gamma Correction — 済（10/28）：物の fx に `filament/effects/Exposure Adjust.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [ ] **Highlights and Shadows**（一部）：Independently adjusts the intensity of the lightest and darkest parts of the layer. — いま：段の影・明るい所の色（全体だけ） — ISF：Color Levels
- [x] **Lens Flare**（一部）：Renders a lens flare in the layer. — いま：段の lens_flare（全体だけ） — ISF：Lens Flare — 済（10/28）：物の fx に `filament/effects/Lens Flare.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [x] **Light Glow**（一部）：Creates a soft glow and optional halo around the lighter areas within a photo or video layer based on a luminance threshold. — いま：段の bloom（全体だけ） — ISF：Bloom — 済（10/28）：物の fx に `filament/effects/Bloom.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [x] **Saturation / Vibrance**（一部）：Adjusts the saturation or purity of colors in the layer. — いま：段の saturation・vibrance（全体だけ） — ISF：Vibrance — 済（10/28）：物の fx に `filament/effects/Vibrance.fs`（Vidvox ISF-Files、MIT）。窓で写真 2 枚の左だけに掛け、左だけが変わるのを絵と数で見た（`skin/book/integration_test/alight_a_test.dart`）
- [ ] **Solid Matte**（一部）：Composites the layer on a solid color matte, filling transparent parts of the layer with a solid background color. — いま：段の background（全体だけ）

## B. 形にしか掛からない曲げを、写真・動画・文字にも（6）

- [ ] **Fractal Warp**（一部）：Distorts the layer using a procedurally generated fractal pattern. Useful for simulating distortions caused by real-world phenomena such as fire, water, and heat. — いま：形の warp・wave_noise（形だけ） — ISF：Noise Displace
- [ ] **Pinch/Bulge**（一部）：Distorts the layer by pinching it towards the center or expanding it outward. — いま：形の pinch・bloat（形だけ） — ISF：Bump Distortion
- [ ] **Roughen Edges**（一部）：Roughens the edges of the opaque parts of the layer. — いま：形の rough（形だけ）
- [ ] **Stretch Axis**（一部）：Stretches the layer in an arbitrary direction. — いま：形の stretch・skew（形だけ）
- [ ] **Turbulent Displace**（一部）：Distorts the layer along selected coordinate axes using a random pattern of smooth fractal noise. Distortion may be along one or more axis in either cartesian or polar coordinate space. Useful for sim — いま：形の wiggle・wave_noise（形だけ） — ISF：Noise Displace
- [ ] **Wave Warp**（一部）：Distorts the layer based on a sine wave. — いま：形の warp・ISF Sine Warp Tile（形だけ）

## C. レイヤーの絵を立体の面に貼る（faux-3D）（16）

- [ ] **Box**（一部）：Renders a faux-3D box using the layer as a surface texture. — いま：立体 cube（キットの形）。物の絵を面に貼るのは無い
- [ ] **Cube**（一部）：Renders a faux-3D cube using the layer as a surface texture. — いま：立体 cube。絵を面に貼るのは無い — ISF：cube
- [ ] **Cylinder**（無い）：Renders a faux-3D cylinder using the layer as a surface texture.
- [ ] **Ellipsoid**（一部）：Renders a faux-3D ellipsoid using the layer as a surface texture. — いま：立体 ball。絵を面に貼るのは無い
- [ ] **Heart**（無い）：Renders a faux-3D heart using the layer as a surface texture.
- [ ] **Hexagonal Prism**（無い）：Renders a faux-3D hexagonal prism using the layer as a surface texture.
- [ ] **Hollow Box**（無い）：Renders a faux-3D hollow box, using the layer as a surface texture, with sides that can be turned on or off, punched out with preset shapes, or offset outward.
- [ ] **Octahedron**（無い）：Renders a faux-3D regular octahedron using the layer as a surface texture.
- [ ] **Pyramid**（無い）：Renders a faux-3D regular pyramid with a square base, using the layer as a surface texture.
- [ ] **Raster Extrude**（無い）：Renders a faux-3D raster extrusion of the layer based on the opaque and transparent pixels within the layer, using the opaque parts of the layer as a surface texture for the front and back of the extr
- [ ] **Spherize**（一部）：Renders a faux-3D sphere, interpreting the layer as an equirectangular projection and using it as a surface texture for the sphere. — いま：ISF Sphere Map
- [ ] **Star Polyhedron**（無い）：Renders a faux-3D star polyhedron using the layer as a surface texture.
- [ ] **Star Prism**（無い）：Renders a faux-3D star prism using the layer as a surface texture.
- [ ] **Three-axis Cross**（無い）：Renders a faux-3D three-axis cross using the layer as a surface texture.
- [ ] **Torus**（一部）：Renders a faux-3D torus using the layer as a surface texture. — いま：立体 ring（絵を面に貼るのは無い）
- [ ] **Tunnel**（無い）：Renders a round faux-3D tunnel using the layer as the interior surface texture.

## D. 手元の ISF-Files（MIT）に近いものがある：置いて確かめる（55）

- [ ] **Block Dissolve**（無い）：Randomly removes grid-aligned rectangular pieces from the layer by making them transparent. Keyframe animation can be applied to remove progressively more and more rectangles over time. — ISF：Random Squares Mask
- [ ] **Block Noise**（無い）：Multiplies randomly generated rectangular blocks of noise with the existing colors of opaque parts of the layer. — ISF：JPEG Block Corruption
- [ ] **Box Blur**（一部）：Blurs the layer by averaging the colors of nearby pixels in a surrounding rectangular area (hence "box" blur). — いま：物の blur（ガウス） — ISF：Fast Blur
- [ ] **Bump Map**（無い）：Uses a height map to model bumps or perturbations in the surface of the layer, then applies simulated lighting and shading. — ISF：Emboss
- [ ] **Channel Remap (HSV)**（無い）：Remaps the hue, saturation, value, and alpha channels in the layer. — ISF：RGBtoHSV / HSVtoRGB
- [ ] **Channel Remap (RGB)**（無い）：Remaps the red, green, blue, and alpha channels in the layer. — ISF：RGBA Swap
- [ ] **Checker**（無い）：Renders a checkerboard pattern in the opaque parts of the layer. — ISF：Checkerboard
- [ ] **Chroma Key**（一部）：Keys out all pixels similar to a particular color, making those parts of the layer transparent. Useful for greenscreening or bluescreening. — いま：動画の keyed（四隅の色を抜く。色は選べない） — ISF：Chroma Mask
- [ ] **Circular Ripple**（一部）：Distorts the layer in a pattern concentric circles around a center point, similar to the ripples caused by a pebble dropped in water. — いま：ISF Circle Splash Distortion — ISF：Ripples
- [ ] **Clouds**（無い）：Renders a procedurally generated cloud pattern in the opaque parts of the layer. — ISF：Simplex Noise
- [ ] **Curl**（無い）：Deforms the layer by curling it into a circular shape from one end. — ISF：Inverted Page Curl
- [ ] **Directional Blur**（一部）：Blurs the layer by averaging nearby pixel colors along a specified angle, similar to the blur caused by motion in a fixed direction. — いま：物の blur の向き（横・縦だけ、角度は無い） — ISF：Linear Blur
- [ ] **Displacement Map**（無い）：Displaces pixels in the background based on the color of pixels in the layer. — ISF：Displace
- [ ] **Dissolve**（一部）：Randomly removes pixels in the layer. This effect can be animated to create a pixel dissolve transition. — いま：点群の dissolve だけ — ISF：Noise
- [ ] **Dots**（無い）：Draws a pattern of random dots in the opaque parts of the layer. Each dot is placed randomly, but the randomness is controlled so that the overall distribution of dots when taken together is approxima — ISF：Polka Dots Curtain
- [ ] **Edge Glow**（一部）：Detects edges in a photo or video and creates a glow around them. — いま：ISF Edge Blowout — ISF：Edges
- [ ] **Fade In/Out**（一部）：Automatically animates the layer opacity to fade the layer in at its beginning, and out again at its end. — いま：キーで。出入りに自動で付くのは無い — ISF：Fade
- [ ] **Find Edges**（無い）：Finds edges in a photo or video layer by detecting sharp changes in luminance between nearby pixels. — ISF：Edges
- [ ] **Flicker**（一部）：Randomly flickers the layer by automatically adjusting the opacity over time. — いま：ふるまい random を opacity に — ISF：RGB Strobe
- [ ] **Four-color Gradient**（一部）：Overlays the opaque parts of the layer with a gradient made between four individually positioned colors. — いま：形・文字の gradient（2 色） — ISF：Corner Colors
- [ ] **Fractal Ridges**（無い）：Renders a procedurally generated pattern of turbulent ridges in the opaque parts of the layer. — ISF：Ridgelines
- [ ] **Glass**（一部）：Renders the layer as if the opaque parts were made of beveled glass, refracting the background and overlaying specular highlights. — いま：立体の ior・dispersion（本物の屈折、立体だけ）。平らな物のガラスは無い — ISF：Frosted Glass
- [ ] **Gradient Map**（一部）：Maps colors in the layer onto a gradient based on their lightness. — いま：物の tritone（3 色）・ISF False Color・Trio Tone — ISF：Duotone
- [ ] **Grid**（無い）：Overlays the opaque parts of the layer with grid lines. — ISF：Graph Paper
- [ ] **Halftone Lines**（無い）：Replaces continuous tones in the layer with a pattern of monochrome lines of varying thickness, a variation of the process used in newspaper or comic book printing. — ISF：Line Screen
- [ ] **Hexagon Tiling**（無い）：Repeats the layer in a pattern of hexagonal tiles, filling the screen. — ISF：Hexagonalize
- [ ] **Hot Color**（一部）：Remaps a grayscale image (or luminance values of a color image) into an intense color ramp based on two selected hues. — いま：ISF False Color — ISF：Thermal Camera
- [ ] **Hue Shift**（無い）：Rotates the color hues in the layer. — ISF：Multi Hue Shift
- [ ] **Invert**（無い）：Inverts the layer's red, green, or blue channels independently. — ISF：Color Invert / RGB Invert
- [ ] **Matte Choker**（無い）：Expands the transparent parts of the layer, shrinking the opaque parts. — ISF：Erode / Dilate
- [ ] **Mosaic**（一部）：Divides the layer into array of squares and averages the color of each square, resulting in a pixelated look. — いま：ISF Multi-Pixellate・Noise Pixellate — ISF：Mosaic
- [ ] **Motion Blur**（無い）：Simulates motion blur by blurring the layer based on position, rotation, and scale keyframe animation from the layer's Move & Transform settings, as well as motion from certain other effects that are  — ISF：VVMotionBlur 3.0
- [ ] **Offset**（無い）：Offsets the layer content, wrapping at the edges of the bounding box of the layer. — ISF：Side Scroller And Flip
- [ ] **Palette Map**（無い）：Maps pixel colors in the layer to the closest matching color in a palette. — ISF：Color Schemes
- [ ] **Pixelate**（一部）：Decreases the layer resolution so that individual pixels become visible, sampling at the centers of the pixels, and drawing each pixel with an optional vignette. — いま：ISF Multi-Pixellate — ISF：Pixellate
- [ ] **Radial Wipe**（無い）：Masks out a region of the layer bounded by two radial lines. — ISF：Radial
- [ ] **Replace Color**（無い）：Replaces one color in the layer with another. — ISF：Color Replacement
- [ ] **Sharpen**（無い）：Enhances details in the layer by increasing the contrast wherever there are abrupt differences in the luminance of nearby pixels in the layer. — ISF：Sharpen RGB
- [ ] **Simple Starfield**（一部）：Renders a simple parallax starfield pattern, with a depth component, in the opaque parts of the layer. — いま：ISF Hyperspace — ISF：Star
- [ ] **Soft Glow**（一部）：Creates a soft Gaussian-based glow within a photo or video layer, or around the outside edge of any layer. — いま：物の glow — ISF：Glow
- [ ] **Spectral Map**（無い）：Maps each color in the layer into a custom spectrum of colors based on the brightness of the original color. — ISF：Color Scales
- [ ] **Spin**（一部）：Automatically spins the layer at a given speed. — いま：rotate のキー（ずっと回り続けるふるまいは無い） — ISF：Rotate
- [ ] **Spot Color**（無い）：Keeps one color while decreasing the saturation of other colors in the layer. — ISF：Chroma Desaturation Mask
- [ ] **Star**（一部）：Renders an individual star in the current layer. — いま：形の star（光る星ではない） — ISF：Star
- [ ] **Stripes**（無い）：Renders a pattern of stripes in the opaque parts of the layer. — ISF：Random Stripes
- [ ] **Text Randomizer**（無い）：Randomly changes individual characters in a text layer. — ISF：Random Characters
- [ ] **Threshold**（無い）：Converts the layer to black and white with a hard cutoff at the specified luminance threshold. — ISF：Color Monochrome
- [ ] **Tile Rotate**（無い）：Divides the layer into a grid of tiles and rotates the pixels in each tile independently. — ISF：Grid Flip
- [ ] **Tiles**（一部）：Repeats the layer in a tile pattern, filling the screen. — いま：ISF Quad Tile・Kaleidoscope Tile — ISF：Quad Tile
- [ ] **Timecode**（無い）：Replaces the text in the layer with a running timecode. — ISF：Digital Clock
- [ ] **Turbulence**（無い）：Renders smooth patterns of procedurally-generated fractal noise in the opaque parts of the layer, useful for simulating real-world phenomena like clouds, or for use as a luminance map. — ISF：Simplex Noise
- [ ] **Unsharp Mask**（無い）：Enhances the perceptual sharpness or "acutance" of a photo or video by increasing local contrast near edges, while avoiding changes to areas that were originally smooth. — ISF：Unsharp Mask
- [ ] **Voronoi Cells**（無い）：Renders procedurally-generated pattern of Voronoi cells based on randomly placed points. — ISF：Worley Cells
- [ ] **Wipe**（無い）：Masks out rectangular segments from opposite sides of the layer. Use keyframe animation to create a wipe transition. — ISF：Wipe Left / Right / Up / Down
- [ ] **Zoom Blur**（無い）：Applies a radial blur to the layer, as though zooming in or out. — ISF：Zoom

## E. 借りる先をまだ探していない（50）

- [ ] **360º Reorient Sphere**（無い）：Reorients a 360º photo or video in equirectangular format.
- [ ] **360º Viewer**（一部）：Projects a 360º photo or video onto a flat surface, as if viewed from within the 360º sphere. — いま：部屋（environment）に正距円筒の絵を貼り、environment_rotate で回す。平らな物の効果としては無い
- [ ] **Bend**（一部）：Bends the layer along a circular arc. — いま：形の bend（shape だけ）。写真・動画・文字には無い
- [ ] **Blink**（一部）：Automatically adjusts the opacity of the layer, alternating between its normal value and zero over time. — いま：ふるまい oscillator を opacity に＋間 Steps
- [ ] **Contour Gradient**（無い）：Renders a gradient around the contours of the opaque parts of the layer.
- [ ] **Contour Lines**（一部）：Draws one or more lines around the contours of the opaque parts of the layer. — いま：形・文字の stroke（輪郭 1 本）
- [ ] **Contour Strips**（無い）：Renders strips of thin paper-like material, layered front to back, using the layer luminance or opacity as a height map for each strip.
- [ ] **Copy Background**（一部）：Copies the background into the layer, effectively turning the layer into an adjustment layer. Effects applied to the layer after this will apply to the background pixels covered by the opaque parts of — いま：段の script（画面全体に .wgsl）。下だけに掛ける調整レイヤーは無い
- [ ] **Count Up/Down**（無い）：Replaces existing numbers in a text layer with the same number added to or multiplied with a fixed value. Useful for creating animations that count up or down.
- [ ] **Dark Glow**（無い）：Creates dark blooms around darker areas within a photo or video layer based on a luminance threshold.
- [ ] **Echo Keyframes**（一部）：Creates one or more copies or "echoes" of the layer as it appeared or will appear at different points in time, based only on keyframe animation applied to the layer. — いま：形の trail・ISF Echo Trace・RGB Trails
- [ ] **Electric Edges**（無い）：Creates a glowing energy-like outline and optional contour lines around the opaque parts of the layer, which can be animated to look like arcs of electricity.
- [ ] **Feather**（一部）：Softens the edges of the opaque part of the layer by blurring the alpha channel. — いま：クリップの clip_feather（切り抜きの縁だけ）
- [ ] **Fill Behind**（無い）：Fills the area behind the layer with a modified copy of the layer, with optional adjustments such as blur, saturation, tint, and brightness, providing a convenient way to quickly adapt a photo or vide
- [ ] **Glow Scan**（無い）：Renders a blur combined with a smooth glow within a region of the layer defined by a luminance band.
- [ ] **Hexagon Array**（無い）：Covers opaque parts of the layer in an array of hexagons.
- [ ] **Hexagon Tile Rotate**（無い）：Divides the layer into a grid of hexagonal tiles and rotates the pixels in each tile independently.
- [ ] **Hexagon Tile Shift**（無い）：Divides the layer into a grid of hexagonal tiles and shifts the contents of each tile in a random direction.
- [ ] **Inner Blur**（無い）：Blurs the opaque parts of the layer without blurring the opacity.
- [ ] **Inner Glow**（無い）：Draws a glow along the inside edges of the opaque parts the layer.
- [ ] **Inner Pinch/Bulge**（無い）：Distorts the layer by pinching it towards the center or expanding it outwards, without affecting the edges or opacity of the layer.
- [ ] **Iridescence**（無い）：Renders a flowing iridescent pattern in opaque parts of the layer.
- [ ] **Lens Blur**（一部）：Applies an angular blur to the layer, excluding the central area. — いま：カメラの focus・aperture（立体と世界の物だけ）
- [ ] **Lightning**（無い）：Renders fractal arcs of lightning between two points in the layer.
- [ ] **Linear Streaks**（無い）：Creates line-shaped streaks by sliding the layer at an angle and blending together overlapping pixels.
- [ ] **Long Shadow**（無い）：Renders long parallel shadows extending from opaque parts of the layer.
- [ ] **Luma Key**（一部）：Keys out all pixels lighter than or darker than a given threshold, making those parts of the layer transparent. — いま：クリップの clip_matte luma（下の物で切る）
- [ ] **Magnify Background**（無い）：Makes the layer transparent and magnifies the area behind the layer, as if the layer were a magnifying glass.
- [ ] **Mask Blur**（無い）：Blurs the background behind the layer, with the blur strength based on the layer luminance.
- [ ] **Move Along Path**（一部）：Positions the layer at a point partway way along the outline path of the layer immediately beneath. — いま：copy_on（写しを別の物の輪郭に）。1 個を道に沿わせて動かすのは写し 1 個で
- [ ] **Omino Diffusion+**（一部）：Renders a photo or video layer with a limited palette using error diffusion, but damping or exaggerating the quantization errors to create intentional distortions. Individual images become unrecogniza — いま：ISF Dither-Bayer（並べた点のディザ、誤差拡散は無い）
- [ ] **Omino Glass**（無い）：Renders the layer as though it were made of glass, with refraction and a glossy contour, and chromatic aberration based on segmentation of the visible spectrum into seven regions. The layer luminance 
- [ ] **Polar Coordinates**（無い）：Warps the layer by mapping from cartesian coordinates to polar coordinates.
- [ ] **Polar Displacement Map**（無い）：Angularly displaces pixels in the background based on the color of pixels in the layer.
- [ ] **Precise Box Blur**（一部）：Performs a slow but precise box blur of the layer. — いま：物の blur
- [ ] **Radial Rays**（無い）：Overlays multiple solid-color radial rays in the opaque parts of the layer.
- [ ] **Radial Shadow**（無い）：Renders long shadows radiating away from a point, based on the opaque parts of the layer.
- [ ] **Random Jitter**（一部）：Automatically jumps the layer randomly back and forth over time, along a selected angle. — いま：ふるまい random
- [ ] **Ribbon**（無い）：Renders a twisted, flowing ribbon of varying width in the opaque parts of the layer.
- [ ] **Smooth Bevel**（無い）：Draws a smooth curved bevel inside the edges of the opaque parts of the layer.
- [ ] **Smooth Edges**（無い）：Smooths the edges of the opaque parts of a layer without affecting the pixel colors within the layer.
- [ ] **Spin Blur**（無い）：Applies an angular blur to the layer, as though it were spinning.
- [ ] **Spin Streaks**（無い）：Creates arc-shaped streaks by spinning the layer around a point and blending together overlapping pixels.
- [ ] **Starfield**（一部）：Renders a complex parallax starfield that can be animated in two dimensions. — いま：ISF Hyperspace
- [ ] **Stretch Segment**（無い）：Slices the layer along an angle and then moves the pieces apart, stretching the pixels between them.
- [ ] **Stroke Taper**（無い）：Tapers the strokes of a freehand drawing layer.
- [ ] **Text Progress**（一部）：Partially displays a text layer, character by character. — いま：文字の split＋出来合いの動き（出る数）
- [ ] **Tile Shift**（一部）：Divides the layer into a grid of tiles and shifts the contents of each tile in a random direction. — いま：ISF Pixel Shifter
- [ ] **Time Quantization**（無い）：Causes time to pass in steps, but just for this layer.
- [ ] **Zoom Streaks**（一部）：Creates radial streaks by scaling the layer around a point and blending together overlapping pixels. — いま：ISF Hyperspace

## F. 有る（確かめるだけ）（33）

- [ ] **Auto-Shake**（有る）：Moves the layer randomly over time within specified parameters. — いま：ふるまい noise を x・y に — ISF：Shake
- [ ] **CMYK Halftone Dots**（有る）：Replaces continuous tones in the layer with a pattern of dots of varying sizes for each CMYK color channel, similar to the process used in magazine or comic book printing. — いま：ISF CMYK Halftone — ISF：CMYK Halftone
- [ ] **Colorize**（有る）：Changes the color of the layer, keeping the original luminance. — いま：物の tint
- [ ] **Drawing Progress**（有る）：Clips the beginning and end of the vector outline of the layer so that the outline is only partially drawn. — いま：形の trim・trim_begin・trim_offset
- [ ] **Flip Layer**（有る）：Flips the layer in 3D along a specified axis. — いま：rotate_x・rotate_y（2.5D） — ISF：Flip H / Flip V
- [ ] **Gaussian Blur**（有る）：Applies a fast, smooth blur to the layer using a gaussian function, reducing noise and details in the layer. — いま：物の blur — ISF：Multi Pass Gaussian Blur
- [ ] **Glow**（有る）：Draws a glow around the outside of the opaque parts of the layer. — いま：物の glow
- [ ] **Gradient Overlay**（有る）：Overlays the opaque parts of the layer with a gradient. — いま：形・文字の gradient — ISF：Linear Gradient
- [ ] **Grid Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged in a grid. — いま：copies＋copy_columns
- [ ] **Halftone Dots**（有る）：Replaces continuous tones in the layer with a pattern of monochrome dots of varying sizes, similar to the process used in newspaper or comic book printing. — いま：効果 halftone・dot_screen — ISF：Dot Screen
- [ ] **Kaleidoscope**（有る）：Creates symmetric radial reflections of the layer. — いま：効果 kaleido・ISF Kaleidoscope Tile — ISF：Kaleidoscope
- [ ] **Linear Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged along a line. — いま：copies＋copy_x・copy_y
- [ ] **Mirror**（有る）：Overlays the layer with a copy of itself mirrored along either the horizontal or vertical axis, with optional alpha and blending mode control. — いま：効果 mirror — ISF：Mirror
- [ ] **Noise**（有る）：Blends random noise with the opaque parts of the layer. — いま：効果 grain・fine_grain — ISF：Noise
- [ ] **Oscillate**（有る）：Moves the layer back and forth repeatedly. — いま：ふるまい oscillator
- [ ] **Posterize**（有る）：Decreases the number of step in the layer's color channels. — いま：ISF Posterize — ISF：Color Posterize
- [ ] **Pulse Opacity**（有る）：Automatically pulses the layer opacity over time, based on a sine wave or triangular wave. — いま：ふるまい oscillator を opacity に
- [ ] **Pulse Size**（有る）：Automatically grows and shrinks the layer repeatedly over time, based on a sine wave or triangular wave. — いま：ふるまい oscillator を size に
- [ ] **Radial Repeat**（有る）：Dynamically generates multiple copies of the layer, arranged in a circular pattern. — いま：copies＋copy_rotate — ISF：Radial Replicate
- [ ] **Random Displacement**（有る）：Automatically moves the layer smoothly but randomly over time — いま：ふるまい noise
- [ ] **Rays**（有る）：Renders light rays from bright parts of the layer, based on the threshold setting. — いま：ISF God Rays — ISF：God Rays
- [ ] **Repeat**（有る）：Dynamically generates multiple copies of the layer. — いま：copies
- [ ] **Repeat Along Path**（有る）：Dynamically generates multiple copies of the layer along the outline path of the layer immediately beneath. — いま：copy_on・copy_on_offset
- [ ] **RGB Split**（有る）：Splits the red, green and blue channels of the layer, keeping one fixed in position and shifting the other two channels in opposite directions. — いま：効果 rgbshift・chroma — ISF：Chromatic Aberration
- [ ] **Scatter Repeat**（有る）：Dynamically generates multiple copies of the layer, scattered randomly around the original layer location. — いま：copies＋scatter_x・scatter_y — ISF：Replicate Random
- [ ] **Solid Color**（有る）：Fills the opaque parts of the layer with a solid color. — いま：物の tint・fill — ISF：Solid Color
- [ ] **Squeeze**（有る）：Distorts the layer by squeezing it along one axis while expanding it along the other axis. — いま：形の squash — ISF：Squeeze
- [ ] **Stroke Color**（有る）：Overrides the layer's stroke color. — いま：stroke
- [ ] **Swing**（有る）：Automatically rotates the layer back and forth over time. — いま：ふるまい oscillator を rotate に
- [ ] **Swirl**（有る）：Warps the layer by rotating it different amounts based on the distance from a center-point, resulting in a vortex or whirlpool-like distortion. — いま：ISF Twirl・形の twist — ISF：Swirl
- [ ] **Text Spacing**（有る）：Adjusts the spacing between individual letters or between lines of text in a text layer. — いま：letter_spacing・line_spacing
- [ ] **Text Transform**（有る）：Transforms individual letters, words, lines, or pieces of letters within a specified range of a text layer. — いま：split＋stagger（Cavalry の Animators の形）
- [ ] **Vignette**（有る）：Darkens the edges of the layer. — いま：効果 vignette・段の vignette — ISF：v002 Vignette

## G. 効果以外の機能（32）

- [ ] 図形・文字・画像・動画・音のレイヤー（有る）：いま film・picture・text・shape
- [ ] ベクター描画（ペン）（有る）：いま ペン（Skia でパスを読む）
- [ ] **手描き（フリーハンド）**（無い）
- [ ] グループ（一部）：いま 関係の group（並びだけ）
- [ ] **Elements**（使い回す部品、元とつながる、文字だけ差し替え、Intro・Outro と Freeze・Stretch・Loop の伸ばし方）（無い）
- [ ] ヌル（有る）：いま 形の null
- [ ] カメラ（Z、ズーム距離・画角、焦点ぼけ、フォグ）（有る）：いま camera の distance・zoom・focus・aperture、段の fog
- [ ] カメラを複数置いて切り替える（未確認）
- [ ] カメラに親子づけ（字幕を画面に固定）（有る）：いま parent
- [ ] 親子づけ（有る）：いま parent
- [ ] **Parenting Helper**（親の回転・大きさを無視・倍に、車輪の自動回転）（無い）
- [ ] 全部の値にキー（有る）
- [ ] イーズ 9 種（ベジェ・Bounce・Elastic・Cyclic・Random・Steps・Elastic Steps）（有る）：いま `filament/interp.h`（Alight Motion を手本に作ったもの。出典をそう直した）
- [ ] 曲線エディタとプリセット（有る）：いま Ease の机、利用者のプリセット
- [ ] **曲線のコピー・貼り付け、全部のキーに貼る**（無い）
- [ ] **レイヤー・見た目・効果のコピー・貼り付け**（無い）
- [ ] 重ね方（ブレンドモード）（有る）：いま blend 25 種
- [ ] マスク（有る）：いま clip（alpha・luma、反転、ぼかし）
- [ ] 塗り：単色・グラデーション・**メディア**（一部）：いま fill・gradient。絵で塗るのは無い
- [ ] 線・影・**Multi-Border**（一部）：いま stroke・shadow。縁取りを何重にもは無い
- [ ] 自分のフォント（有る）：いま ドックのフォント
- [ ] ブックマーク（有る）：いま ロケーター（Ableton どおり）
- [ ] お気に入り（無い）
- [ ] **速度に応じたモーションブラー**（無い）
- [ ] **Time Quantization（コマ落とし。ポスタリゼーション時間）**（無い）
- [ ] **レイヤーを切る**（無い）：いま ⌘E はエンジンに切るが無い
- [ ] 動画から音を取り出す（無い）
- [ ] 作った後でプロジェクトの設定（大きさ・コマの速さ）を変える（無い）：いま 隠した欄だけ（screens.md）
- [ ] プレビューの拡大・手のひら（有る）：いま トラックパッド 2 本指・ピンチ
- [ ] 書き出し MP4（H.264）（一部）：いま ffmpeg で決め打ち（export_map.md）
- [ ] **書き出し HEVC・GIF・PNG 連番・静止画・透明**（無い）
- [ ] **プロジェクトを包んで共有（リンク・QR）**（無い）


## H. 実機で見つけた根幹の操作（alight_ui.md）（66）

- [ ] 新しい文書の箱：縦横比 6 種＋自由、解像度 7 段、fps 10 段、背景、PROJECT か ELEMENT
- [ ] 文書の設定を後から変える（同じ箱）と、編集した時間
- [ ] 書き出し：形式ごとの行、空き容量と大きさの見積もり
- [ ] 書き出し Video：解像度、H.264 / H.265、fps 3 段、画質 Mbps
- [ ] 書き出し：今のコマを PNG
- [ ] 書き出し：画像の連番（PNG / JPEG、解像度、fps、画質）
- [ ] 書き出し：プロジェクトを素材ごと包んで共有（リンク）
- [ ] 書き出し：テンプレートとしても書き出す
- [ ] プロジェクト一覧：カードに長さ・縦横比・解像度・大きさ・fps、日付の見出し
- [ ] プロジェクト一覧：並べ替え（名前・日付）、選ぶ・書き出す・複製・消す
- [ ] プロジェクト一覧：Templates・Elements のタブ、XML を読み込む
- [ ] 環境設定：低画質のプレビュー、既定のレイヤーの長さ、音の変換の質
- [ ] 取り消し・やり直しが常に画面にある
- [ ] 全部のレイヤーを選ぶ
- [ ] レイヤーを複製
- [ ] レイヤーを見た目ごとコピー・貼り付け
- [ ] 見た目だけ貼る（何を貼るか選ぶ）
- [ ] プリセットとして保存（値とキー）、読み込む
- [ ] 見え方：Pixel、Grid、Solo、カメラ越し、拡大縮小と %
- [ ] ＋の 1 枚に全部：図形・メディア・音・物・テンプレート・手描き・ベクター・文字
- [ ] 図形 54 種を絵で選ぶ
- [ ] 手描き：ペン・ブラシ・塗りつぶし・消しゴム・太さ・色
- [ ] 空のグループを足す
- [ ] Element：使い回す部品、元とつながる、文字だけ差し替え、グループに戻す
- [ ] レイヤーを左右・上下に反転
- [ ] 画面に合わせる（Fit・Fill・Stretch）
- [ ] クリッピングマスクを作る
- [ ] レイヤーのラベルの色
- [ ] 塗りの種類 4 つ（なし・単色・グラデーション・メディア）
- [ ] メディアで塗る
- [ ] 色：スポイト、小さいピッカー、パレット、自分の色を足す
- [ ] 線の始点・終点の形 10 種と大きさ
- [ ] 縁を何重にも足す（Multi-Border）
- [ ] 影：色・大きさ・アルファ・位置
- [ ] 重ね方 28 を 7 分類で畳み、見本の絵で選ぶ
- [ ] Mask・Exclude の重ね方
- [ ] 回転をダイヤルで
- [ ] 大きさの幅・高さのつなぎ
- [ ] 傾き X・Y と、傾き・変形を戻す
- [ ] こする帯でレイヤーと点を動かす（指で隠れない）
- [ ] 点を順に送って選ぶ、点を足す・消す、曲線の種類、閉じる・開く
- [ ] どの値も左の列に「キー・イーズ」が同じ場所
- [ ] 数字は細かい目盛りをこする
- [ ] イーズ：反転、前・次のカーブへ
- [ ] イーズ：カーブのコピー・貼り付け、全部のキーに貼る、行き過ぎを許す
- [ ] 速さ：亀とウサギのつまみ、どちらの端を止めるか
- [ ] 帯を左で詰める・切る・右で詰める
- [ ] 文字：揃え・フォント・大きさ・色を 1 段に
- [ ] フォント：最近使ったもの、太さごと、星
- [ ] フォント：分類（Serif…）、言語、並べ替え、読み込む
- [ ] 複数選ぶと上の帯が変わり、数が出る
- [ ] グループにする・グループにしてマスク・グループにして抜く
- [ ] 時間の揃え：始まりを伸ばす・再生位置へ寄せる・左揃え・ずらして並べる・右揃え
- [ ] 画面の揃え：左・中・右・上・中・下・等間隔 2 種
- [ ] グループに入って編集、パンくず
- [ ] グループの設定：幅・高さ・背景・Precompose・伸び方（Freeze・Stretch・Loop・Loop & Stretch・Blank）
- [ ] カメラ：遠近、画角とズーム
- [ ] カメラ：焦点ぼけ（距離・深度・強さ）
- [ ] カメラ：フォグ（色・近い・遠い）
- [ ] 初めての所で押す場所を吹き出しで指す
- [ ] 効果の検索（説明文の中まで）
- [ ] 効果のおすすめ、分類、動く絵
- [ ] 効果の大きな動く見本・説明・タグ・Guide
- [ ] 効果ごとのプリセット、お気に入り
- [ ] 効果の札：畳む、詳しく、既定に戻す、複製、コピー、消す
- [ ] レイヤーの種類ごとの帯の色

## I. アプリだけにある効果（公式ガイドに無い 16）（16）

- [ ] Advanced Chroma Key
- [ ] Chromatic Zoom Blur
- [ ] Color Balance
- [ ] Energy Beam
- [ ] Grunge
- [ ] Key Spill Cleaner
- [ ] Luma Stamper
- [ ] Matte Fringe
- [ ] Page Curl
- [ ] Parenting Helper
- [ ] Raster Transform
- [ ] Ring
- [ ] Scale Assist
- [ ] Scatter Field
- [ ] Streak Strips
- [ ] Warp Blur


## J. 写真・動画・音（実機、alight_ui.md）（16）

- [ ] Media のタブに見本の素材（Sample Media）
- [ ] アルバムの切り替えと数
- [ ] 写真は足す前に長さを決める
- [ ] 動画は足す前に使う範囲を切る（再生できる）
- [ ] 写真・動画を複数選んで足す
- [ ] 写真を図形と同じ仕組みで（塗り・縁・影が効く）
- [ ] 写真の切り抜きの大きさと取っ手（角・辺）
- [ ] 動画の帯にコマの絵
- [ ] 音量のつまみにキー・イーズ
- [ ] 動画から音を別のレイヤーに取り出す
- [ ] Convert to Outline（輪郭に変換）
- [ ] Media Info（大きさ・fps・長さ・形式・ファイルの大きさ）
- [ ] 数字の欄を押すと数字のキーパッド（＋/−・小数点）
- [ ] 音の棚（曲・アルバム・アーティスト・ジャンル）
- [ ] 曲を試しに鳴らしてから足す
- [ ] 音の帯に波形


## K. 効果の札とタイムラインの絵（実機）（6）

- [ ] 効果が 2 つ以上なら札の一覧（入り切りの目・並べ替え）
- [ ] 効果の小見出しを畳む
- [ ] 効果ごとに複数のプリセット（動きつきも）
- [ ] 効果の最初の値を「掛かって見える値」に
- [ ] タイムラインの帯にコマの絵
- [ ] 音の帯に波形


## L. 隠れた操作とタイムライン・Element（実機。Alight Motion で長押しに隠れている物は「無い」扱い。Motolii は見える所に置く）（7）

- [ ] アンカーポイントを動かす（Alight Motion は隠れている。Motolii は見える所に）
- [ ] Z を動かす帯
- [ ] 値を動かすと次のキーが自動で入る
- [ ] 有料の機能を使うと書き出しに鍵（Motolii は真似しない）
- [ ] Element の伸ばし方の印（イントロの終わり・アウトロの始まり）
- [ ] プロジェクトの絵にするコマを決める
- [ ] 隠れた操作を作らない（長押しだけの操作は、メニューか右クリックにも出す）


## M. キャンバスで物を動かす（実機）（6）

- [ ] 掴んだ取っ手だけ光らせ、ほかを消す
- [ ] 動かしている最中に数字を出す（Alight Motion に無い）
- [ ] 吸い付きとガイドの線（Alight Motion に無い）
- [ ] 反対側を止めて伸ばす（Alight Motion は真ん中からだけ）
- [ ] キャンバスに動きの道筋とその取っ手（Alight Motion に無い）
- [ ] 再生位置で切って後ろ半分を別のレイヤーに


## N. マスク（実機）（7）

- [ ] クリッピングマスク：すぐ下のレイヤーの形で切る（Create / Release、行に ↘ の印）
- [ ] クリッピングマスク：型の塗りは消え、型を動かすと切る場所が動く
- [ ] グループにしてマスク：グループの一番上が型、その下を切る
- [ ] グループにして抜く：一番上と重なる所を抜く
- [ ] グループを解く（Ungroup）
- [ ] 図形の取っ手で角の数・角の丸みをキャンバスで変える
- [ ] 取り消しの履歴を開き直しても残す（Alight Motion は消える）


## O. 有料の中身：イーズと親（実機、無料体験）（12）

- [ ] イーズの既定の数字と取っ手を Alight Motion に合わせる：Bounce
- [ ] 同：Elastic（今は大きくずれている）
- [ ] 同：Cyclic
- [ ] 同：Random（今はおとなしすぎる）
- [ ] 同：Steps（今は 6 段、Alight Motion は 4 段）
- [ ] 同：Elastic Steps
- [ ] イーズの取っ手を曲線の上で直接動かす（黄＝形、白＝始めと終わり）
- [ ] ベジェの 4 つのプリセット
- [ ] 親：一覧で選ぶ（None・自分は灰色・音は出ない）
- [ ] 親：子を選ぶと親の外枠を出す
- [ ] 動きの道筋を細い線と向きの矢印で出す
- [ ] 書き出し GIF

合計：337 箱。
