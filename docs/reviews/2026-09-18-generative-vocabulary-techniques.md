# 技法の表(遅れて届いた子調査の版、URL を curl で実在確認した物)— 2026-09-18-generative-vocabulary.md の付録

親の報告(§1、66 行)と同じ項目を、URL の実在確認付きで別に書いた版。純/記憶の列は同じ定義。
Shadertoy(403)・fxhash(402)・OpenProcessing(403)・Observable(429)は照合不能 → URL 未確認。toxiclibs.org は乗っ取り済み(GitHub の postspectacular/toxiclibs を使う)。

| 名前(英 / 和) | 見分け方 | 代表作 1 | 代表作 2 | 純/記憶 | 出自 |
|---|---|---|---|---|---|
| Flow field / 流れ場 | 格子の角度に沿って粒子が描く平行気味の曲線群 | Hobbs「Flow Fields」2020 https://tylerxhobbs.com/essays/2020/flow-fields | Coding Train #24 https://thecodingtrain.com/challenges/24-perlin-noise-flow-field | 両方(場は純、粒子は記憶) | Processing/p5 |
| Fidenza 系 | 太さ違いの非交差の帯、衝突で止まる | Hobbs「Fidenza」2021 https://tylerxhobbs.com/fidenza | Art Blocks #78 | 記憶 | Art Blocks |
| Perlin noise loops | 円上を回って noise を引き、継ぎ目なくループ | Etienne Jacob 2017 https://necessarydisorder.wordpress.com/2017/11/15/drawing-from-noise-and-then-making-animated-loopy-gifs-from-there/ | Coding Train #136 | 純 | Processing/p5 |
| Lorenz / Clifford / de Jong attractor | 蝶の渦・煙の密度画 | Paul Bourke https://paulbourke.net/fractals/lorenz/ https://paulbourke.net/fractals/clifford/ | Coding Train #12 | 記憶(反復)、密度画は時刻に依らない | 数学 |
| Reaction-diffusion(Gray-Scott) | 珊瑚・指紋・分裂する斑点 | Karl Sims https://www.karlsims.com/rd.html | pmneila https://pmneila.github.io/jsexp/grayscott/ | 記憶 | GPU |
| Voronoi / Delaunay | 細胞状の分割 / 点群の三角網 | d3-delaunay https://github.com/d3/d3-delaunay | iq https://iquilezles.org/articles/voronoilines/ | 純 | d3/Shadertoy |
| Circle packing | 重ならない円が密に埋まる | https://generativeartistry.com/tutorials/circle-packing/ | Coding Train #50 | 記憶 | Processing |
| L-system | 文字列置換 → タートル | Paul Bourke https://paulbourke.net/fractals/lsys/ | Processing 例 penrosetile | 純(世代の関数) | Algorithmic Botany |
| Truchet tiles | 1 種のタイルの 4 回転が繋がる迷路 | Carlson 2018 https://christophercarlson.com/portfolio/multi-scale-truchet-patterns/ | https://paulbourke.net/geometry/truchet/ | 純 | Processing |
| 10 PRINT | / と \ の並び | https://10print.org/ | — | 純 | BASIC |
| Boids / flocking | 分離・整列・結合 | Reynolds 1986 https://www.red3d.com/cwr/boids/ | Processing 例 flocking | 記憶 | Nature of Code |
| Particle trails | 半透明で重ねた軌跡 | Processing 例 multipleparticlesystems | Karl Sims「Flow」2018 https://www.karlsims.com/flow.html | 記憶 | Processing/TouchDesigner |
| Metaballs | 距離の逆数の和を閾値で切る | Coding Train #28 | — | 純 | Shadertoy |
| Marching squares | 等値線 | d3-contour https://github.com/d3/d3-contour | CONREC https://paulbourke.net/papers/conrec/ | 純 | d3 |
| Dithering(Floyd–Steinberg) | 2 値なのに階調 | Surma「Ditherpunk」2021 https://surma.dev/things/ditherpunk/ | Coding Train #90 | 記憶(走査順) | 画像処理 |
| Halftone / Moiré / Op-art | 網点 / 周期格子の重なり / 錯視の縞 | Processing 例 pointillism | Riley(Tate) | 純 | 印刷/絵画 |
| Lissajous / Harmonograph / Spirograph | sin の合成 / 減衰振り子 / トロコイド | Coding Train #116、Karl Sims https://www.karlsims.com/harmonograph/ | https://paulbourke.net/geometry/harmonograph/ | 純 | 数学/機械 |
| Superformula | 1 式で星・花・多角形 | https://paulbourke.net/geometry/supershape/ | Coding Train #23 | 純 | Gielis |
| Differential growth | 線が自分を避けて膨らむ | inconvergent https://inconvergent.net/generative/differential-line/ | Jason Webb https://github.com/jasonwebb/2d-differential-growth-experiments | 記憶 | Python/Houdini |
| Space colonization | 引力点へ枝が伸びる | https://github.com/jasonwebb/2d-space-colonization-experiments | inconvergent Hyphae https://inconvergent.net/generative/hyphae/ | 記憶 | Algorithmic Botany |
| Wave Function Collapse / MarkovJunior | 局所パターンで矛盾なく埋める / 書き換え規則 | Gumin https://github.com/mxgmn/WaveFunctionCollapse | https://github.com/mxgmn/MarkovJunior | 記憶 | ゲーム開発 |
| Cellular automata(Life・Rule 30/110) | 格子の生死 / 1 行ずつ積む縞 | Processing 例 gameoflife・wolfram | MathWorld Rule 30 | 記憶(行 = 時刻で決定的) | Wolfram |
| Noise displacement / Sine wave grid | noise で揺れる格子 / sin(x+t) の位相ずれ | Processing 例 noisewave・sinewave・additivewave | Sighack https://sighack.com/post/getting-creative-with-perlin-noise-fields | 純 | Processing |
| Molnár (Dés)Ordres / Nees Schotter / Mohr cube | 震える同心正方形 / 下へ行くほど乱れる格子 / 立方体の断片 | DAM https://dam.org/museum/artists_ui/artists/molnar-vera/、V&A Schotter https://collections.vam.ac.uk/item/O221321/schotter-print-nees-georg/ | https://www.emohr.com/ | 純(番号の乱数) | plotter |
| Sol LeWitt wall drawing | 指示を実行者が描く | MASS MoCA https://massmoca.org/sol-lewitt/ | Solving Sol https://solvingsol.com/ | 純 | 概念美術 |
| Substrate / Sand stroke(Tarbell) | 直角に折れる線が既存線で止まる | http://www.complexification.net/gallery/machines/substrate/ | …/sandstroke/ | 記憶 | Processing |
| Chaos game / IFS / Barnsley fern | 頂点へ跳ぶ点 | Coding Train #123、#108 | https://paulbourke.net/fractals/ifs/ | 記憶(反復)、密度画は決定的 | 数学 |
| Mandelbrot / Julia / zoom | 発散回数の色 | Processing 例 mandelbrot | https://paulbourke.net/fractals/juliaset/、Coding Train #22 | 純 | Shadertoy |
| Domain warping / fbm / SDF raymarching / palettes | fbm(p + fbm(p)) / 距離関数 / cos パレット | iq https://iquilezles.org/articles/warp/ https://iquilezles.org/articles/distfunctions/ https://iquilezles.org/articles/raymarchingdf/ https://iquilezles.org/articles/palettes/ | — | 純 | Shadertoy |
| Phyllotaxis | 黄金角 137.5° | Coding Train #30 | — | 純 | Processing |
| Hexagonal / Penrose tiling / Hilbert curve / Chladni | 蜂の巣 / 5 回対称 / 空間充填 / 節線 | Red Blob https://www.redblobgames.com/grids/hexagons/、Preshing https://preshing.com/20110831/penrose-tiling-explained/ | https://paulbourke.net/geometry/chladni/ | 純 | 数学 |
| Quadtree / Recursive subdivision(Mondrian) | 密な所ほど細かい / 再帰で割って原色 | Coding Train #98 | https://generativeartistry.com/tutorials/piet-mondrian/ | 純 | Processing |
| Pixel sort / ASCII / Kaleidoscope | 明度順に流れる筋 / 文字の濃さ / n 回対称 | Asendorf https://github.com/kimasendorf/ASDFPixelSort、Coding Train #47 #166 #155 | https://www.dmitricherniak.com/ | 純 | Processing/p5 |
| Plotter hatching | 平行線の密度で塗る | vpype https://github.com/abey79/vpype | vsketch https://github.com/abey79/vsketch | 純 | plotter |
| Bezier ribbons / blobs | 制御点を noise で揺らす帯 / 円周の点を揺らして閉じる | Coding Train #163、#36 | Processing 例 bezier | 純 | Processing/p5 |
| Spring / Verlet cloth / Strand / Gravity / Ripple | ばね・布 / 紐 / 逆二乗 / 2 バッファの波 | Jakobsen 2001 https://www.cs.cmu.edu/afs/cs/academic/class/15462-s13/www/lec_slides/Jakobsen.pdf、toxiclibs https://github.com/postspectacular/toxiclibs、Hugo Elias 2D water(archive) | Coding Train #160 #144 #102、Nature of Code forces | 記憶 | Processing/GPU |
| Sand spline(agent 線) | 曲線を粒子の集合として動かし痕を重ねる | https://inconvergent.net/generative/sand-spline/ | https://github.com/inconvergent/sand-spline | 記憶 | Python |
| Grid of rotating lines | 格子の短い線の角度が場で変わる | Loren Bednar(fxhash 402) | https://piterpasma.nl/ | 純 | fxhash/plotter |
| Generative typography / SDF text | 文字の輪郭を崩す / 距離場の文字 | Tim Rodenbröker https://timrodenbroeker.de/ | msdfgen https://github.com/Chlumsky/msdfgen | 純 | p5/GPU |
| Physarum(粘菌) | 粒子がフェロモンを置き嗅ぐ網 | Sage Jenson 2019 https://www.sagejenson.com/physarum | — | 記憶 | GPU |
| Weighted Voronoi stippling / Poisson disk | 暗い所ほど密の点 / 最小距離の均一乱点 | Coding Train #181、#33 | Jason Davies https://www.jasondavies.com/poisson-disc/ | 記憶(Lloyd / 生成順) | d3/p5 |
| Curl noise | 発散なしの流れ | Bridson 2007 https://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph2007-curlnoise.pdf | — | 両方 | Houdini/GPU |
| Glass / refraction | 背景が歪んで透ける | Art Blocks「Pixel Glass」#24 | — | 純 | GPU |
| Video feedback | 前フレームを縮小・回転して重ねる | TouchDesigner Feedback TOP https://docs.derivative.ca/Feedback_TOP | — | 記憶 | TouchDesigner |
| Ringers(紐と杭)/ Archetype / Meridian・Subscapes / zancan の植物 | 杭巻き / 反復の格子 / 筆致の地層 / プロッタの葉 | Art Blocks #13、#23、#163・#53 | https://generated.space/、https://zancan.art/、canvas-sketch https://github.com/mattdesl/canvas-sketch | 純 / 純 / 両方 / 記憶 | Art Blocks/fxhash |
| Falling sand | 砂粒が落ちる格子 | Coding Train #180 | — | 記憶 | p5 |
