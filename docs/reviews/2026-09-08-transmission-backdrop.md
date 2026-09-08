# ガラスは背後に描かれた層を屈折して通す

利用者指示(2026-09-08): 「修正しておいてくれ」— 9/7 環境 review の宿題「透過は環境だけを通す。背後の層の屈折は未着手」を消す。借りたのは vgpu `examples/transmission`(MIT)の作法: 背後のフレームを mip ピラミッドにし、Snell 屈折の出口を画面へ投影して粗さの段で読む。

## 描く側

- **fork(re_renderer、commit b0b12d28)**: `TargetConfiguration::backdrop`(画面空間、乗算済み、mip 付き)を group 0 の binding 7 に、clamp の trilinear `screen_sampler` を 8 に。`shade_surface` は位置と厚みを受け、屈折した光線が厚み分の板を抜けた点を画面へ投影して backdrop を `pow(r, 0.8)·(levels−1)·0.55` の段で読む。backdrop の alpha が 0 の所は環境。`SurfaceIn.thickness` = instance の scale(world_from_mesh の 1 軸の長さ)。`TextureManager2D::generate_mipmaps` を公開。
- **Motolii**: `accumulate_sequential` は**網の手前で run を切る**。網を含む run の前に、ここまでの合成を mip 付きの 1 枚に写して(`backdrop_pyramid`、copy + GPU mip)config に渡す。網より上の層は屈折されない(同じ run の中)。
- `vism/glass.wgsl` と解析スタブ(`subtype.rs` の `hook_stub`)は新しいシグネチャへ。

## 厚み

vgpu は立方体の出口を解析的に追う。Motolii の網は任意なので、instance の scale を「屈折光が抜ける板の厚み」に使う。単位球なら直径相当、細長い網では過大。網ごとの厚みが要る時は param に出す(宿題)。

## 検収

- render `glass_refracts_the_layers_drawn_behind_it`: 白い空の前に赤い板、その手前のガラス(ior 1.5・transmission 1)の中心は赤(R>150, G<80)。鏡にすると空の白を映す。
- 既存の環境 3 本(鏡 < 艶消し < ガラス、空が背景と照明、exr)は据え置きで通過。render 44 本、ui/native 28 本。
- 実 HDRI(Poly Haven monochrome_studio_02_4k)で、ガラス球の背後に置いた板が屈折して見える絵を目視。

## 宿題

- 同じ run の中の層(網より上の順序の 2D 層)は屈折されない。網が 2 つ重なる時、後ろの網は前の網の backdrop に入る(run を切るので)が、互いの映り込みは無い。
- 分散(色ずれ)・吸収(色付きガラス)は vgpu にあるが未採用。欄が増えるので要る時に。
- 厚みは instance scale の近似。
