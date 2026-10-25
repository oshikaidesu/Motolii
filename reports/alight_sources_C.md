# C. レイヤーの絵を立体の面に貼る（faux-3D）16 種：借りる先

2026-10-25 調べ。`reports/alight_checklist.md` の C の 16 種。

## しくみは 1 つ：物の絵を、借りた形の表面に貼る

**物の絵（もう在る）→ Filament の立体のテクスチャ。形は three.js の形の関数を QuickJS でそのまま動かして作る。**

1. **物の絵**：`filament/thing_fx.h` がもう、物を 1 つだけ別の画に描いて、その上に効果を順にかけている（`st.seen` が出来上がりの絵）。これをそのままテクスチャにする。効果を先にかけた絵が貼られるので、Alight の Twirl（貼る前の渦）は ISF の `Twirl.fs` を fx に足せば済む。
   - 気をつける所：いまの絵は「カメラから見た物の箱」。2D の物ならそのまま平らな絵だが、2.5D で傾いた物は傾いて写る。貼る用には、物の面に真っすぐ向いた平行投影のカメラで撮る aim がもう 1 つ要る（`thing_fx_aim` の隣、30 行ほど）。
2. **形**：three.js（MIT、r186）の `three.core.js` を QuickJS-ng でそのまま読み、`BoxGeometry` などを呼んで、位置・法線・UV・番号の配列を Motolii の `Mesh`（`kit_stage.h`、もう UV0 を Filament に渡している）へ写す。`three_isf.cpp`（three.js のシェーダーを QuickJS で動かす、151 行）と同じやり方。
   - **確かめた**（qjs 0.17.0、`three.core.js` を手を入れずに）：読み込み 24 ms。Box 24 点、Cylinder 292 点、6 角柱 40 点、Pyramid 19 点、Octahedron 24 点、Sphere 1225 点、Torus 5289 点（8 ms）、Heart 3444 点、Star Prism 108 点、Tunnel 3185 点。どれも UV がある。`RoundedBoxGeometry` と `BufferGeometryUtils.mergeGeometries`（examples、MIT）も、`'three'` の読み先を差し替えるだけで動いた。面取りした星の柱 100 個で 89 ms。
   - three.js の置き場：今は過去の倉庫の `rust_ae/Motolii/work/path-tube/drei-ref/node_modules/three`（v0.186.1）にしか丸ごと無い（読むだけ）。`research_notes/threejs_shaders/three` は部分 clone で、形のファイルの中身が無い。`filament/third_party/js/three/` に `three.core.js`（1.4 MB）と examples の 2 本を置く事になる。
3. **材質**：Filament の見本 `samples/materials/texturedLit.mat`（Apache-2.0）の `baseColor = texture(albedo, getUV0())` を、いまの `floor_lit.mat` に足した物を 1 本。UV の行列（three.js の `Matrix3.setUvTransform` と同じ並び）、透明な所は masked、内側も見える doubleSided（Tunnel・Hollow Box）。Flat（光なし）は unlit の版をもう 1 本（`samples/materials/image.mat` の形）。

**Alight より良くなる所**：Alight はレイマーチ（どの形にも Precision のつまみ）でレイヤーの中だけに描く。Motolii では本物の立体として世界に入るので、他の立体と前後し、影を落とし受け、Filament の光（sun・point・spot）と反射が効く。Alight の Orientation・Rotation・Position・Scale は物の x・y・z・rotate_x/y・scale_x/y/z のキーがそのまま受け持つ。

**Alight の共通のつまみの受け手**：
- Texture Mapping：Mirror → Filament の `WrapMode::MIRRORED_REPEAT`、Repeat X/Y・Angle・Offset・Crop → UV の行列（`setUvTransform`）、Twirl → ISF `Twirl.fs`、Stretch → UV の行列の縦横比。
- Light Source：Point Light・Directional Light → Filament の光（Motolii にもう在る light の point・sun）。Shine・Specularity → metallic・roughness。Ambient → 部屋の光の強さ。Depth Map → Filament の見本 `samples/materials/depthVisualizer.mat`（Apache-2.0）。**Normal Map の出力には見本が無い。**
- Smoothing（角を丸く）：形ごとに下に書く。

**触るファイルと大きさ（目安）**：
- 新しく `filament/solid_three.cpp`（+ `.h`）：QuickJS で three.js を読み、形のファイルを走らせ、配列を `Mesh` に写す。150 行ほど（`three_isf.cpp` と同じ大きさ）。
- 新しく `filament/solids/*.js`：1 種 1 ファイル、5〜20 行。頭に AviUtl2 式のつまみ（レシピ `script.h` と同じ書き方）、下で three.js の形を呼ぶだけ。
- 新しく `filament/solid_image.mat`（50 行ほど）と unlit の版。
- `filament/doc_moment_fx.h`：物の fx の中に形の .js があれば、絵の板（`effected.mat`）の代わりにその形を物の場所に置き、毎コマ `image` に `st.seen` を渡す（20〜30 行）。Alight と同じく「効果の 1 つ」として扱える。
- `filament/thing_fx.h`：平らに撮る aim（30 行ほど）。
- `build_lib.sh`・`edit.sh`：1 行ずつ。
- 合わせて C++ 300 行ほど＋材質 2 本＋形の .js 16 本。
- ついで：いまのキットの形（`kit_stage.h` の `cube`・`sphere`・`torus`・`gem`）は自分で書いた物で、`cube` と `gem` は UV が全部 0（絵を貼れない）。three.js の形に替えれば、立体 cube・ball・ring も絵を貼れるようになる。

## 形ごとの出所

- **Box**：three.js `BoxGeometry`（MIT）。Smoothing は examples の `RoundedBoxGeometry`（MIT、確かめた：2916 点）— 入れ方：Width・Height・Depth をそのまま渡す。6 面それぞれに絵がまるごと 1 枚（Alight の「正方形の絵にすると歪まない」と合う）。
- **Cube**：Box と同じ形で 3 辺を等しく（`BoxGeometry` / `RoundedBoxGeometry`、MIT）— 入れ方：Box の .js を辺 1 本のつまみで。ISF の `cube.fs` は gl-transitions の 2 枚の絵の切り替え（回る箱の 2 面だけ）で、立体の箱ではない。
- **Cylinder**：three.js `CylinderGeometry`（MIT）— 入れ方：側面は絵が 1 周、ふたは円の UV。Smoothing は `Path.absarc` で角を丸めた輪郭を `LatheGeometry`（MIT）に回す。
- **Ellipsoid**：three.js `SphereGeometry`（MIT）— 入れ方：球を作り、物の scale_x/y/z で楕円に（法線は Filament が直す）。
- **Heart**：three.js の見本 `examples/webgl_geometry_shapes.html` の `heartShape`（ベジェ 6 本、MIT）を `ExtrudeGeometry`（MIT）で押し出す — 入れ方：Smoothing は ExtrudeGeometry の bevel。前後の面の UV は `UVGenerator` のつまみで「絵の 0〜1」に合わせる（確かめた：UV が 0〜1 に収まる）。
- **Hexagonal Prism**：three.js `CylinderGeometry(r, r, h, 6)`（MIT）— 入れ方：そのままだと法線がなめらかで丸く見えるので、three.js の `toNonIndexed()`＋`computeVertexNormals()` で平らな面に。Smoothing は Cylinder と同じく `LatheGeometry` を 6 分割で。
- **Hollow Box**：板 6 枚を three.js `Shape`＋`ExtrudeGeometry`（MIT）で、厚み＝Thickness — 入れ方：
  - Hidden ＝その板を作らない。
  - Circular Cutout・Square Cutout ＝ `Shape.holes` に `Path.absarc` の円か四角を足す（確かめた：円の穴の板 1200 点、UV 0〜1）。大きさは Progress。
  - Circular Crop・Square Crop（角から削る）＝四角と円（四角）の重なり。重なりの計算は Filament の中にもう在る Clipper 4.8.8（`third_party/filament_src/third_party/libassimp/contrib/clipper`、Boost）か、JS なら polygon-clipping（MIT）。Motolii の libtess2（SGI Free B）の巻き数 ABS_GEQ_TWO でも重なりが取れる。
  - Outset（全体と面ごと）＝形ではなく板の置き場：板を 1 枚ずつ別の部品にして法線の向きへずらす（`add_parts` の places）。
  - Smoothing ＝ bevel。
- **Octahedron**：three.js `OctahedronGeometry`（MIT）— 入れ方：そのまま（面ごとの角の点で、もう平らな法線）。UV は球の投影なので、Alight が面ごとに絵 1 枚なら Alight で見比べて `UVGenerator` 相当を考える。
- **Pyramid**：three.js `ConeGeometry(r, h, 4, 1, false, π/4)`（MIT、底が正方形になる向き）— 入れ方：法線は Hexagonal Prism と同じく平らに直す。Smoothing は `LatheGeometry` を 4 分割で。
- **Raster Extrude**：輪郭は d3-contour（ISC、マーチングスクエア、`smooth` で線形補間＝Alight の「縁を少しなめらかに」）、押し出しは three.js `ExtrudeGeometry`（MIT、穴は `Shape.holes`、三角形は three.js 同梱の earcut）— 入れ方：
  - 物の絵の不透明度を CPU に読む（Filament の `Renderer::readPixels`）。格子の細かさ＝Alight の Precision（10〜800）。
  - d3-contour の結果（GeoJSON の決まりで、1 本目が外側、2 本目からが穴）を Shape と holes にして押し出す。
  - 前後の面は `UVGenerator` で点の (x, y) を絵の UV に。側面も同じ点の UV を渡すと、縁の画素が奥へ伸びる（Alight の Interior 0% と同じ）。Interior Color・Fade Color は材質で奥行き（物の中の z）に合わせて混ぜる。
  - 点が多すぎる時は simplify-js（BSD-2）で減らす。
  - 絵が変わるコマ（動画）だけ作り直す。
  - 使わない物：potrace（GPL-2、曲線はきれいだが同梱できない）、OpenCV `findContours`（Apache-2.0、大きすぎる）。
  - 楽な代わり：Filament の見本 `samples/heightfield.cpp`＋`materials/heightfield.mat`（Apache-2.0、格子を絵で押し上げる）。ただし側面が斜めになり、Alight の真っすぐな側面にはならない。
- **Spherize**：three.js `SphereGeometry`（MIT）— 入れ方：この UV がそのまま正距円筒（u＝経度、v＝緯度。確かめた：先頭の点は v=1、u=0.0625 から）なので、絵を貼るだけで Alight の「equirectangular として読む」と同じになる。手元の ISF `Sphere Map.fs`（VIDVOX、MIT）は 2D の魚眼のような写し方で、正距円筒ではない（近いだけ）。
- **Star Polyhedron**：**出所なし**。Alight のつまみは Points 2〜16、Size、Depth（前の中心から後ろの中心まで）、Point Length、Smoothing 0〜5。前後の中心がとがった星（星の双角錐）らしいが、ガイドに形の名前も絵も無い。
  - 近い物：Blender の Add Mesh Extra Objects の Star（`add_mesh_star.py`、GPL-2.0+）はただの星の柱で、これではない。three.js `PolyhedronGeometry`（MIT）は点と面の一覧を渡せば作るが、その一覧を作る物が無い。
  - 先に Alight で形を撮って確かめる。
- **Star Prism**：星の輪郭は three.js の見本 `examples/webgl_geometry_extrude_shapes.html` の星（半径を交互に 10 点、MIT）か Motolii の star の形（points・inner）。押し出しは `ExtrudeGeometry`（MIT）— 入れ方：Heart と同じ（bevel で Smoothing、`UVGenerator`）。Motolii の形の輪郭ならどれでも押し出せるので、Alight より広い。
- **Three-axis Cross**：three.js `BoxGeometry` 3 本（x・y・z に細長く）を examples の `BufferGeometryUtils.mergeGeometries`（MIT）で 1 つに（確かめた：72 点）— 入れ方：Smoothing は `RoundedBoxGeometry` を 3 本。Alight のガイドにはつまみが書いてない（「まだ書いている途中」）ので、腕の太さ・長さは Alight で見て決める。
- **Torus**：three.js `TorusGeometry`（MIT）— 入れ方：そのまま（u が輪を 1 周、v が管を 1 周）。いまの立体 ring は自分で書いた `torus()` なので、これに替える。
- **Tunnel**：three.js `CylinderGeometry(…, openEnded=true)`（MIT）を内側から見る。曲がったトンネルにするなら `TubeGeometry`＋`CatmullRomCurve3`（MIT、確かめた：3185 点）— 入れ方：材質は doubleSided（内側が見える）、`WrapMode::REPEAT` で絵を奥へ繰り返し、UV の v をずらすと奥へ流れる。遠くを暗くするのは Filament の View の fog。Alight のガイドにはつまみが無い。手元の ISF に Tunnel の物は無い（`filament/recipes/tunnel.js` は写しを並べるレシピで別物）。

## 出所が無い・決まっていない物

- **Star Polyhedron の形を作る物**：見つからない（上）。
- **Normal Map の出力**（Light Source の Shading の 1 つ）：Filament の見本に材質が無い。Depth Map は `depthVisualizer.mat` が在る。
- **Alight の面への貼り方**（Octahedron・6 角柱・Pyramid が、面ごとに絵 1 枚か、ぐるりと 1 周か）：ガイドに書いてない。Alight で撮って合わせる。
- **Three-axis Cross・Tunnel・Spherize のつまみ**：ガイドが「まだ書いている途中」。

## 調べた所

- Alight のガイド：[Hollow Box](https://guide.alightmotion.com/effects/hollow-box)、[Star Polyhedron](https://guide.alightmotion.com/effects/star-polyhedron)、[Raster Extrude](https://guide.alightmotion.com/effects/raster-extrude)、[Three-axis Cross](https://guide.alightmotion.com/effects/three-axis-cross)、[Cube](https://guide.alightmotion.com/effects/cube)、[Tunnel](https://guide.alightmotion.com/effects/tunnel)、[Spherize](https://guide.alightmotion.com/effects/spherize)
- three.js の見本：[webgl_geometry_shapes.html](https://raw.githubusercontent.com/mrdoob/three.js/r180/examples/webgl_geometry_shapes.html)（heartShape）、[webgl_geometry_extrude_shapes.html](https://raw.githubusercontent.com/mrdoob/three.js/r180/examples/webgl_geometry_extrude_shapes.html)（星）
- [d3-contour](https://github.com/d3/d3-contour)（ISC）、[contours の説明](https://d3js.org/d3-contour/contour)
- Blender の Star：[add_mesh_star.py](https://raw.githubusercontent.com/blender/blender-addons/main/add_mesh_extra_objects/add_mesh_star.py)（GPL-2.0+）、[Add Mesh Extra Objects](https://docs.blender.org/manual/en/2.81/addons/add_mesh/mesh_extra_objects.html)
- 手元：`filament/thing_fx.h`、`kit_stage.h`、`doc_match.h`、`three_isf.cpp`、`floor_lit.mat`、`effected.mat`、`third_party/filament_src/samples/`（`rendertarget.cpp`、`materials/texturedLit.mat`、`image.mat`、`depthVisualizer.mat`、`heightfield.mat`）、`research_notes/isf/ISF-Files/ISF/`（`Sphere Map.fs`、`cube.fs`、`Twirl.fs`）
