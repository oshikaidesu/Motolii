# ジェネラティブアートの語彙 — 見た目の名前・作家・場・Examples の索引(2026-09-18 取得)

課題: 「欲しい画をどう言えばいいか分からない」。界隈が見た目に付けている名前を集め、それぞれが **(index, time, position) の純関数か、記憶(前の時刻の状態)を持つか** を表にする。一次資料(作家本人サイト/GitHub/投稿、fxhash・Art Blocks の作品ページ、Processing/p5 公式、genuary.art)を当日取得。取れなかった物は「未確認」「取得できず」と書いた。判断は書かない。

付録: 2026-09-18-generative-vocabulary-techniques.md(同じ技法表を、URL を curl で実在確認した版)。

## 0. 要約(5 行)

1. 名前は 3 層で付く: **数学の名(Lissajous・Voronoi・Lorenz)**、**アルゴリズムの名(flow field・circle packing・WFC・reaction-diffusion)**、**作家の名(Fidenza 系・Molnár の Des(Ordres)・Nees の Schotter・Tarbell の Substrate)**。作家名が技法名になる例が 2021 以降(Art Blocks)に急増した。
2. 純/記憶の境目は「粒子や線が**進む**か」。格子・タイル・関数の絵(Truchet、sine grid、Lissajous、superformula、SDF、Voronoi)は純。粒子・成長・群れ・反応拡散・物理(flow field trail、differential growth、boids、Gray-Scott、verlet)は記憶。**記憶系でも「時刻の純関数に書き直せる」物**(Brownian → hash、軌道 → 閉形式)がある。
3. 作家 30 人(海外 22 + 日本 8)は道具で 4 群: p5/JS(Hobbs、Cherniak、Golid、Xie、takawo、reona396)、Processing/openFrameworks(Reas、Lieberman、Gamboa Naon、Kwok、junkiyoshi)、plotter(inconvergent、Licia He、Zancan、Pasma)、shader/3D/AI(Pasma、Yanagisawa、Klingemann、Anadol)。
4. 場は 12。語彙が付く仕組みは **Art Blocks の Aesthetic/Theme(Botanical・Geometric・Painterly・Minimal…)**、fxhash のタグ、Shadertoy のタグ(ともに 403/402 で当日未取得)、**Genuary の日課お題 186 本(2021〜2026 全年、§3B)**。Genuary が最も語彙の密度が高い(「Draw 10,000 of something」「Wobbly function day」「Definitely not a grid」)。
5. Processing 公式 Examples は Basics 11 + Topics 17 + Demos、p5.js は 55 件。純関数の芯は Basics/Math(Noise・Sine・PolarToCartesian)、Transform、Form、Fractals。記憶は Topics/Motion・Simulate(Flocking・ParticleSystem・Smoke)・Cellular Automata。

## 1. 技法/見た目の名前(66 行)

純 = (index, time, position) の純関数で描ける / 記憶 = 前の時刻の状態が要る / 両 = 実装で選べる。URL は当日開けた物のみ、それ以外は「URL 未確認」。

| # | 名前(英 / 和) | 見分け方 | 代表作 1 | 代表作 2 | 純/記憶 | 出自 |
|---|---|---|---|---|---|---|
| 1 | flow field / 流れ場 | 向きの場に沿って粒子が線を引く。線が束になって流れる | Tyler Hobbs "Fidenza"(2021)https://www.artblocks.io/collection/fidenza-by-tyler-hobbs | Hobbs 本人の解説 "How to hack a flow field" URL 未確認 | 記憶(粒子が進む)。場そのものは純 | Processing → p5 / Art Blocks |
| 2 | Perlin noise loop / ノイズの周回 | 時刻を円周上に置いて noise を引き、継ぎ目無く loop する | Golan Levin / Etienne Jacob の tutorial URL 未確認 | Genuary 2023 Day 1「Perfect loop」 | 純 | Processing / p5 |
| 3 | strange attractor(Lorenz / Clifford / de Jong)/ ストレンジアトラクタ | 点が軌道を描き、蝶形・渦の密度で濃淡 | Lorenz 1963(数学) | Paul Bourke の Clifford/de Jong 解説 http://paulbourke.net/fractals/clifford/ | 記憶(反復写像) | Processing / plotter |
| 4 | reaction-diffusion(Gray-Scott)/ 反応拡散 | 珊瑚・指紋・迷路の縞、時間で育つ | Karl Sims の解説 https://www.karlsims.com/rd.html | Shiffman Coding Train #… URL 未確認 | 記憶(セル状態の更新) | Processing / Shadertoy / TouchDesigner |
| 5 | Voronoi / ボロノイ | 種点への最近傍で割った多角形の細胞 | Wikipedia(数学) | Genuary 2023 Day 12「Tessellation」系 | 純(種が固定なら) | Processing / Shadertoy / Houdini |
| 6 | circle packing / 円詰め | 大小の円が触れずに埋まる | Genuary 2022 Day 12「Packing」 | Genuary 2021 Day 29「Any shape, none can touch」 | 記憶(置いた円に依存)。Apollonian gasket は純 | Processing / p5 |
| 7 | L-system / L システム | 文字列の書き換えで枝分かれ(植物) | Prusinkiewicz & Lindenmayer "The Algorithmic Beauty of Plants"(1990)http://algorithmicbotany.org/papers/#abop | Processing Examples: Topics/Fractals and L-Systems | 純(世代数 = 時刻) | Processing |
| 8 | Truchet tiles / トルシェタイル | 四半円や斜線の 1 種類のタイルを回転して敷き、道がつながる | Truchet 1704 / Smith 1987 | Genuary 系多数 | 純 | Processing / p5 / Shadertoy |
| 9 | 10 PRINT | `/` と `\` をランダムに敷いた迷路。C64 の 1 行 | 10 PRINT CHR$(205.5+RND(1)); : GOTO 10(書籍 2012)https://10print.org/ | — | 純(hash で決める) | BASIC → Processing |
| 10 | boids / flocking / 群れ | 群れが揃って向きを変える | Craig Reynolds 1986 https://www.red3d.com/cwr/boids/ | Processing Examples: Topics/Simulate/Flocking | 記憶 | Processing / p5 / Houdini |
| 11 | particle trails / 粒子の軌跡 | 点の後ろに尾が残る(背景を薄く塗り重ねる) | Genuary 2024 Day 1「Particles, lots of them」 | Processing Examples: Topics/Simulate/ParticleSystem | 記憶 | Processing |
| 12 | metaballs / メタボール | 円がくっつく時に溶ける輪郭 | Blinn 1982 | Shadertoy 多数 URL 未確認 | 純(場の等値線) | Shadertoy / Processing |
| 13 | marching squares / マーチングスクエア | 格子の等値線を折れ線で抜く | Lorensen & Cline 1987(cubes) | Coding Train URL 未確認 | 純 | Processing / p5 |
| 14 | dithering(Floyd–Steinberg)/ ディザ | 白黒 2 値で写真の階調を点の密度で出す | Floyd & Steinberg 1976 | Genuary 2022 Day 2「Dithering」 | 記憶(誤差拡散は走査順に依存)。ordered/Bayer は純 | Processing / Shadertoy |
| 15 | halftone / 網点 | 大小の点の格子で階調 | Genuary 系 | — | 純 | Processing / Shadertoy |
| 16 | moiré / モアレ | 2 つの周期格子の干渉縞 | Genuary 2021 Day 9「Interference patterns」 | Genuary 2023 Day 23「More Moiré」 | 純 | Processing / plotter |
| 17 | op-art / オプアート | 錯視・黒白の膨らみ(Riley・Vasarely 系) | Genuary 2025 Day 19「Op Art」 | Bridget Riley(参照) | 純 | Processing |
| 18 | Lissajous / リサージュ | sin(a t)・sin(b t) の閉曲線 | Lissajous 1857(数学) | Processing Examples: Basics/Math | 純 | Processing |
| 19 | harmonograph / ハーモノグラフ | 減衰する振り子の重ね書き | Wikipedia | plotter 界隈 | 純(減衰は時刻の関数) | plotter |
| 20 | spirograph / スピログラフ | 円の中を転がる円が描く花 | hypotrochoid(数学) | Processing Examples 系 | 純 | Processing |
| 21 | superformula / スーパーフォーミュラ | 1 式で貝・星・花の輪郭 | Johan Gielis 2003 | Processing Examples 系 URL 未確認 | 純 | Processing / Shadertoy |
| 22 | differential growth / 差分成長 | 閉曲線が自分を避けながら膨らんで襞になる | inconvergent "differential line" https://inconvergent.net/generative/differential-line/ | Anders Hoff の GitHub https://github.com/inconvergent | 記憶 | plotter / Processing / Houdini |
| 23 | space colonization / 空間占有 | 引力点に向かって枝が伸びる(葉脈・木) | Runions et al. 2005 http://algorithmicbotany.org/papers/#colonization | Coding Train URL 未確認 | 記憶 | Processing / Houdini |
| 24 | Wave Function Collapse / 波動関数収縮 | タイル制約を満たす配置を局所から決めていく | Maxim Gumin 2016 https://github.com/mxgmn/WaveFunctionCollapse | — | 記憶(伝播の順) | 独自 → Processing / Houdini |
| 25 | cellular automata(Game of Life・Rule 30/110)/ セルオートマトン | 格子が世代ごとに更新、縞・グライダー | Genuary 2021 Day 2「Rule 30」 | Raven Kwok "Rule 110"(2020–21)https://ravenkwok.com/about/ | 記憶(世代)。Rule 90 等一部は閉形式 | Processing |
| 26 | Delaunay / ドロネー三角形 | 点群を三角で結ぶ(Voronoi の双対) | Wikipedia | Genuary 2021 Day 6「Triangle subdivision」 | 純 | Processing / Houdini |
| 27 | noise displacement / ノイズ変位 | 格子や線を noise でずらして揺らぐ | Processing Examples: Basics/Math/Noise2D | — | 純 | Processing / Shadertoy |
| 28 | sine wave grid / 正弦波の格子 | 格子点を sin(x+t) で上下 | Processing Examples: Basics/Math/SineWave, AdditiveWave | Genuary 2023 Day 15「Sine waves」 | 純 | Processing |
| 29 | Fidenza 系 flow(Hobbs)| 太さの違う帯、非衝突、限られた palette、丸みの角 | Fidenza(2021)上記 | Genuary 2022 Day 4「The next next Fidenza」 | 記憶 | p5 / Art Blocks |
| 30 | Molnár 系 disorder(Des(Ordres))| 正方形の入れ子が少しずつ乱れる | Vera Molnár "(Dés)Ordres"(1974)URL 未確認 | Genuary 2024 Day 5「In the style of Vera Molnár」 | 純(乱れ = hash) | plotter(元) |
| 31 | Nees の Schotter / 砂利 | 上は整列、下へ行くほど回転と位置が乱れる格子 | Georg Nees "Schotter"(1968)URL 未確認 | Genuary 2021 Day 19「Increase the randomness along the Y-axis」 | 純 | plotter(元) |
| 32 | Mohr の cube(Manfred Mohr)| 高次元立方体の投影線 | Manfred Mohr(1970s〜)https://www.emohr.com/ | — | 純 | 独自 |
| 33 | LeWitt 系 wall drawing / 指示の絵 | 文の指示だけで描く | Genuary 2022 Day 7「Sol LeWitt Wall Drawing」 | Genuary 2021 Day 7「Generate some rules, then follow them by hand」 | 純(規則) | 概念美術 |
| 34 | Substrate / 結晶の線(Tarbell)| 線が既存の線に垂直に生えて街区のように埋まる | Jared Tarbell "Substrate"(2003)http://www.complexification.net/gallery/machines/substrate/ | Sand Stroke 同サイト | 記憶 | Processing |
| 35 | chaos game / IFS | 点をランダムな写像で飛ばして図形が浮かぶ(Sierpinski・Barnsley fern) | Barnsley 1988 | — | 記憶(反復) | Processing |
| 36 | Mandelbrot / Julia | 複素反復で縁がフラクタル | Processing Examples: Topics/Fractals/Mandelbrot | Shadertoy 多数 | 純(画素ごと独立) | Shadertoy / Processing |
| 37 | domain warping / 領域歪み | noise の中に noise を入れて雲・大理石 | Inigo Quilez https://iquilezles.org/articles/warp/ | — | 純 | Shadertoy |
| 38 | raymarching SDF / 距離関数 | 距離関数で 3D を描く、柔らかい結合 | Inigo Quilez https://iquilezles.org/articles/distfunctions/ | Piter Pasma "Universal Rayhatcher"(2023)https://www.fxhash.xyz/u/Piter%20Pasma | 純 | Shadertoy |
| 39 | phyllotaxis / 葉序 | 黄金角で並ぶ種(ひまわり) | Vogel 1979 | Processing Examples 系 URL 未確認 | 純 | Processing |
| 40 | hexagonal tiling / 六角格子 | 蜂の巣 | Genuary 2024 Day 10「Hexagonal」 | — | 純 | Processing |
| 41 | Penrose tiling / ペンローズ | 非周期の菱形 2 種 | Penrose 1974 | — | 純(deflation の世代) | Processing / plotter |
| 42 | Hilbert / space-filling curve / 空間充填曲線 | 1 本の線が面を埋める | Hilbert 1891 | Genuary 2021 Day 8「Curve only」系 | 純(index → 位置) | plotter / Processing |
| 43 | Chladni / クラドニ図形 | 板の振動節の砂模様 | Chladni 1787 | Genuary 2025 Day 27 系 | 純 | Processing / Shadertoy |
| 44 | quadtree subdivision / 四分木分割 | 正方形を再帰で割る | Genuary 2021 Day 14「SUBDIVISION」 | Genuary 2025 Day 12「Subdivision」 | 純 | Processing |
| 45 | recursive subdivision(Mondrian)/ 再帰分割 | 長方形を割って原色で塗る | Genuary 2023 Day 17「A grid inside a grid inside a grid」 | — | 純 | Processing |
| 46 | glitch / pixel sort / ピクセルソート | 画素を明度で並べ替えて縦横に流れる | Kim Asendorf pixel sorting(2010)https://github.com/kimasendorf/ASDFPixelSort | Genuary 2025 Day 31「Pixel sorting」 | 純(行ごとの sort) | Processing |
| 47 | ASCII art / 文字絵 | 文字の濃さで階調 | Andreas Gysin play.core https://github.com/ertdfgcvb | Genuary 2024 Day 9「ASCII」 | 純 | JS / Processing |
| 48 | kaleidoscope / symmetry / 万華鏡・対称 | 放射・鏡映で繰り返す | Genuary 2025 Day 26「Symmetry」 | Genuary 2026 Day 17「Wallpaper group」 | 純 | Shadertoy / Processing |
| 49 | plotter hatching / ハッチング | 平行線の密度で陰影(ペンプロッタ) | Genuary 2026 Day 22「Pen plotter ready」 | Piter Pasma "Rayhatching" | 純 | plotter |
| 50 | Bezier ribbons / 帯 | 太さの変わる曲線の帯 | Genuary 2022 Day 8「Single curve only」 | — | 純 | Processing |
| 51 | spring / verlet cloth / ばね・布 | 布・鎖が垂れて揺れる | p5 Examples: Soft Body | Genuary 2024 Day 15「Use a physics library」 | 記憶 | Processing / Houdini |
| 52 | sand spline / agent 線(inconvergent)| 線を多数の点で描き、微小にずらして砂のよう | inconvergent https://inconvergent.net/generative/sand-spline/ | — | 記憶 | plotter / Common Lisp |
| 53 | Bezier blobs / 有機的な塊 | 頂点を noise で揺らした閉曲線 | Genuary 2026 Day 25「Organic Geometry」 | — | 純 | p5 |
| 54 | grid of rotating lines / 回る線の格子 | 格子の各点で線が時刻で回転、波が走る | Genuary 2026 Day 20「One line」系 | — | 純 | Processing |
| 55 | generative typography / 生成タイポ | 文字を粒子・格子・歪みで | Genuary 2024 Day 20「Generative typography」 | Genuary 2022 Day 19「Use text/typography」 | 両 | Processing / p5 |
| 56 | SDF text / 距離関数の文字 | 文字を距離場で太らせ・溶かす | Green 2007(Valve)URL 未確認 | — | 純 | Shadertoy |
| 57 | slime mold(Physarum)/ 粘菌 | 粒子がフェロモンを残し、網が育つ | Jones 2010(論文)URL 未確認 | Sage Jenson "physarum" https://cargocollective.com/sagejenson/physarum | 記憶 | Shadertoy / TouchDesigner / compute |
| 58 | weighted Voronoi stippling / 点描 | 写真を点の密度で(Lloyd 緩和) | Secord 2002 | Coding Train URL 未確認 | 記憶(緩和の反復) | Processing / plotter |
| 59 | Poisson disk / ポアソン円盤 | 均一で重ならない点のばら撒き | Bridson 2007 | — | 記憶(既存点に依存)。blue-noise texture 化で純 | Processing |
| 60 | curl noise / カールノイズ | 発散ゼロの流れ場(粒子が固まらない) | Bridson 2007 | — | 場は純、粒子は記憶 | Houdini / TouchDesigner / Shadertoy |
| 61 | strand / rope / 紐と杭 | 杭に紐を巻く | Dmitri Cherniak "Ringers"(2021)https://www.artblocks.io/collection/ringers-by-dmitri-cherniak | — | 純 | p5 |
| 62 | gravity / orbits / 重力・軌道 | 天体が引き合う | p5 Examples: Forces | Genuary 2021 Day 30「gravity, flocking, path following」 | 記憶(N 体)。2 体・円軌道は純 | Processing |
| 63 | ripple / wave equation / 波紋 | 水面の波が伝わる | Hugo Elias の 2D water URL 未確認 | Shadertoy 多数 | 記憶(差分方程式)。sin の重ね合わせは純 | Shadertoy |
| 64 | glass / refraction / ガラス・屈折 | 背景が歪んで透ける | Shadertoy 多数 | Sync(Cavalry)の作例 URL 未確認 | 純(画素ごと) | Shadertoy / TouchDesigner |
| 65 | feedback loop / 映像フィードバック | 前フレームを縮小・回転して重ねる | TouchDesigner の Feedback TOP | Genuary 2023 Day 16「Reflection of a reflection」 | 記憶(前フレーム) | TouchDesigner / Shadertoy |
| 66 | color quantization / palette shift / 減色・色替え | 色数を絞る・掌で色を回す | Genuary 2022 Day 17「3 colors」 | Genuary 2024 Day 2「No palettes. Generative colors」 | 純 | Processing / Shadertoy |

補: Truchet・10 PRINT・Schotter・(Dés)Ordres・LeWitt は「乱数 = hash(index)」なら時刻を持たず純。flow field・Substrate・differential growth・boids は「時刻 t の絵は t−1 の絵から」で記憶。

## 2. 作家(30 人、2020〜2026)

出典は本人サイト/公式ページ/本人インタビュー。「未確認」は一次で取れなかった項。

| 作家 | 型 | 代表作(名前・年・URL) | 道具 | 本人の言葉 1 行(出典) | 拠点 |
|---|---|---|---|---|---|
| Tyler Hobbs | flow field / Fidenza 系 | Fidenza(2021)、QQL(2022)、Please Respond(2026)https://www.tylerxhobbs.com/about | JS(p5 系) | "I want to deeply merge the hand with the algorithm"(同上) | Austin |
| Zach Lieberman | 幾何抽象・色の日課 | Daily Sketches(2016〜) | openFrameworks | "Color to me is about feeling." https://layer.com/artists/zach-lieberman/interview | NY / MIT |
| Matt DesLauriers | 地形・ストローク・省 byte | Meridian(2021)https://mattdesl.substack.com/p/notes-on-meridian、Sierra(2024)https://sierra.mattdesl.com/ | JS(15 KB) | — | UK |
| Casey Reas | ソフトウェアペインティング → GAN | An Empty Room(2023, LACMA)、METAJUDD(2025)https://reas.com/ | Processing(共同創始) | — | LA |
| Manolo Gamboa Naon | 色彩幾何・量産 | monta(2020)、88 Allegories(2021)https://www.katevassgalerie.com/manoloide | Processing | "The most beautiful parts of the work are born from the errors."(同上) | Argentina |
| Anders Hoff(inconvergent) | 単純規則の有機構造 | differential line、sand spline https://inconvergent.net/ | Common Lisp、plotter https://github.com/inconvergent | — | Oslo |
| Jared Tarbell | Substrate 系譜 | Circle Inversion(2023)、Ray Marching the Full Moon(2024)https://www.lerandom.art/artists/jared-tarbell | Processing、Grasshopper | — | New Mexico(未確認) |
| Saskia Freeke | 幾何・遊びの日課 | Daily Art(2015〜)https://sasj.nl/portfolio/daily/ | Processing | — | Amsterdam |
| Raven Kwok | セルオートマトン映像 | Rule 110(2020–21)、Knotted(2023–25)https://ravenkwok.com/about/ | Processing(旧 Flash) | — | Shanghai |
| William Mapan | 手描き感テクスチャ | Dragons(2021)、Anticyclone(2022)、Distance(2023, LACMA)https://www.rightclicksave.com/article/an-interview-with-william-mapan | JS | — | Paris |
| Piter Pasma | shader / Rayhatching(SDF → plotter) | Skulptuur(2021, Art Blocks)、Universal Rayhatcher(2023, fxhash)https://www.fxhash.xyz/u/Piter%20Pasma | JS(依存無し)、GLSL、SVG→plotter | "For me, the code is everything." https://www.rightclicksave.com/article/the-interview-piter-pasma-generative-art | Groningen |
| Iskra Velitchkova | 幾何抽象 + 有機線 | INK(Art Blocks Curated)、ToSolaris https://www.artblocks.io/artists/iskra-velitchkova | 未確認 | "The print process... extends the digital dialogue into the physical world." https://avantarte.com/insights/avant-essay/iskra-velitchkova-notes-on-creation | Madrid |
| Aaron Penne | 幾何抽象・層状 | Apparitions、Rituals(2022 Lumen Prize)、Catalina(2024)https://www.aaronpenne.io/ | JavaScript | — | 未確認 |
| Kjetil Golid | 幾何抽象(格子・反復・オートマトン) | Archetype(2021)https://www.artblocks.io/collection/archetype-by-kjetil-golid、Orbifold(2023) | Processing → p5/JS、全コード公開 generated.space | "The visual rendering is a derivative of the underlying concept." https://www.brightmoments.io/quarterly/kjetilgolid | Trondheim |
| Mario Klingemann | AI(GAN・自律 agent) | Botto(2021〜)https://botto.com、Appropriate Response(2020) | Python、JS https://github.com/Quasimondo | — | München(Wikipedia) |
| Refik Anadol | AI データ彫刻 | Unsupervised(2022–23, MoMA)https://www.moma.org/calendar/exhibitions/5535、Sphere(2023) | 独自 AI https://refikanadol.com/works/ | "Taking the data that surrounds us as primary material..." https://refikanadol.com/refik-anadol/ | Los Angeles |
| Licia He | plotter + 水彩 | Running Moon(2022, AB Curated)、Sparkling Goodbye(2023)https://www.eyesofpanda.com/gallery/ | ペンプロッタ | "algorithms are my love language, and the plotter is my quill" https://www.eyesofpanda.com/about/ | 東京 / ロンドン |
| Dmitri Cherniak | 幾何抽象(紐と杭) | Ringers(2021)https://www.artblocks.io/collection/ringers-by-dmitri-cherniak | JS / p5 | "I regard automation as my artistic medium" https://www.rightclicksave.com/article/an-interview-with-dmitri-cherniak | New York |
| Snowfro(Erick Calderon) | 1 本のグラデーション線 | Chromie Squiggle(2020) | オンチェーン JS | "Do I think this could make someone smile?" https://www.dalosdov.com/writing/snowfro-interview | Houston |
| Emily Xie | コラージュ(木版・筆致) | Memories of Qilin(2022)https://emilyxie.art/projects/memories_of_qilin | JS + p5 | "Entirely made with code and onchain."(同上) | New York |
| Zancan | plotter 植物線描 | Garden Monoliths(2022, fxhash)https://zancan.art/Series/Garden-Monoliths | JS + plotter | — | Bordeaux |
| Kim Asendorf | pixel sorting | Cargo(2023, Art Blocks)、Event Horizon(2023)https://www.brightmoments.io/quarterly/kimasendorf | Processing / JS | "ordered and destructed simultaneously"(同上) | Berlin |
| Julian Hespenheide(jhesp) | 幾何・分割 | KERNELS https://www.artblocks.io/collection/kernels-by-julian-hespenheide | Processing / p5 | — | Berlin |
| Loren Bednar | 色の層・回る線 | phase(Art Blocks Curated)、Pressed Pause(2024) | JS / WebGL | — | Michigan |
| Marcin Ignac | data art・3D | PEX(自作 3D lib) | JS | — | London |
| Joshua Davis | ランダム規則(HYPE Framework) | HYPE Framework https://thegreatdiscontent.com/interview/joshua-davis/ | 旧 Flash → Processing | — | NY |
| Daniel Shiffman | 教育(Nature of Code) | Nature of Code(2024 版)https://natureofcode.com/introduction/ | p5 | — | NYU ITP |
| Andreas Gysin(ertdfgcvb) | ASCII | play.core https://github.com/ertdfgcvb | JS | — | Milan |
| Ivan Dianov | セルオートマトン・生成文法 | Prayer for Wisdom(2024)https://ivandianov.com/works/ | p5 / GLSL | — | Moscow |
| Nicolas Barradeau | WebGL | Hydra https://experiments.withgoogle.com/hydra | JS / WebGL | — | Paris(未確認) |
| Yazid | ミニマル幾何 | Hashed Arcs(2021, fxhash)https://www.kaloh.xyz/p/-yazid-is-creating-simple-minimal | JS | — | 未確認 |
| 高尾俊介(takawo) | デイリーコーディング / #つぶやきProcessing | Generativemasks(2021)https://ccbt.rekibun.or.jp/players/takawo-shunsuke | p5 | 「コードを思考や感性を媒介する表現メディアとして」https://cenkhor.org/ | 兵庫 |
| Reona Hibino(reona396) | グラデーション・柔らかい形 | Gradation #9(2022)、Zero Gravity Ice Cream #2(2023)https://www.rightclicksave.com/article/generative-art-and-japans-bright-future | p5、fxhash / OpenProcessing / NEORT | — | 日本 |
| 中村勇吾 | インタラクション | HUMANITY(2023)https://designing.jp/tha-nakamura | Flash → Unity / Unreal | — | 東京 |
| Yuma Yanagisawa | shader・リアルタイム 3D 流体 | https://www.stirworld.com/see-features-japanese-artist-yuma-yanagisawa-and-the-new-arts-discourse | TouchDesigner / Unity / Unreal / Max | "Code plays a vital role in my art."(同上) | 日本 |
| Hisa(deconbatch) | 「偶然のプログラミング法」 | sine・random walk・vector field の作例 https://www.deconbatch.com/ | Processing | — | 日本 |
| Senbaku | 民俗学・幽霊 | FLUX(2022, fxhash)、Signage Ghosts(2023) | p5 | "There isn't a competitive atmosphere here in Japan…"(rightclicksave 上記) | 日本 |
| junkiyoshi(中内純) | openFrameworks 3D メッシュ + Perlin、毎日投稿 8 年 | https://junkiyoshi.com/about/ | openFrameworks | — | 名古屋 |
| Ryoji Ikeda(参考) | data・音 | data-verse 3(2020)、data.cosm(2023–25)https://www.ryojiikeda.com/archive/works/ | 独自 | — | Paris |

未確認・除外: 岡村健太(一次で作家として確認できず)、梅沢和木(コラージュ作家、生成ではない)、Kenta Watashima(テクニカルディレクター)、Jen Lowe(作家として未確認)。

## 3. 場(画廊)

### 3A. 場の表

| 場 | URL | 何が置かれるか | 語彙の付き方 | 備考 |
|---|---|---|---|---|
| fxhash | https://www.fxhash.xyz/ (docs https://docs.fxhash.xyz/) | 生成トークン(GENTK)。code を zip で上げ hash で決定論的に出力。fx(params)(収集者が調整)、open-form(親 hash を継承して子を mint)、fx(lens)、fx(text) | mint 手順 Step 10「Project Details」で **title / description / tags / labels**。labels は挙動(**Animated / Interactive / Image Composition**)。Explore は filter、Marketplace は sort(sought after / recent hits)。キュレーションは外部(Deca、Objkt Curations)を案内 | Tezos / Ethereum / Base。本体は 402、docs から |
| Art Blocks | https://www.artblocks.io/ | 「each work is created the moment it's collected」— mint 時に生成する ETH 作品 | 系列: **Studio / Playground / Factory / Presents / Art Blocks 500**。Aesthetic: **Botanical・Geometric・Painterly・Space・Minimal・Digital remnants・Figurative・Landscape**。価格帯、Tags & Features(作品ごとの trait 名) | /collections は 404 |
| OpenProcessing | https://openprocessing.org/ | p5.js / Processing のスケッチ、code 公開、Curation | **取得できず**(本体・/discover・/about とも 403) | — |
| Shadertoy | https://www.shadertoy.com/ | GLSL の画素シェーダ | **取得できず**(本体・/browse とも 403) | — |
| Genuary | https://genuary.art/ | 1 月 31 日のお題に沿った code 作品(場は SNS)。「an artificially generated month of time where we build code that makes beautiful things」 | 語彙 = **お題そのもの**(§3B)+ hashtag `#genuary` `#genuary2026` `#genuary1`〜`31` | 第 1 回 2021(karaman.is「This year was the first #genuary」)。Piter Pasma 本人サイトに「lead organizer of Genuary 2021 and 2022」https://piterpasma.nl/。2021 のお題は 12 名の共同(github.io の credits)。年別 /2022〜/2025/prompts、現行 /prompts、2021・2022 は github.io |
| #plottertwitter / plotter 界隈 | https://drawingbots.com/plottertwitter · https://penplotter.art/p/ptpx-plotter-postcards · https://buttondown.com/ptpx | ペンプロッタの紙作品・機材・工程。drawingbots(Maks Surguy)が投稿を手で選んで時系列に | 正式タグ無し。hashtag **#plottertwitter, #ptpx**(PTPX = Plotter Postcard Exchange、Paul Butler、住所を無作為に割り当て葉書を交換)。Instagram は @penplotart の **Plot Party** | drawingbots.net → .com |
| Processing Community Day | https://day.processing.org/ (財団 https://processingfoundation.org/) | 各地の地域主催イベント(Events Map、Organizer Kit、Zine Library) | 「Started in 2017 … a global network of community-led events」。語彙 = 都市・年(NEORT の `#PCDTokyo2026`) | 単一の公式日は無く 10 月推奨 |
| NEORT(日本) | https://neort.io/ | ブラウザ作品(GLSL/JS)、NFT、Space、Exhibition。NIINOMI 運営、実空間 NEORT++ | nav **NFT / Explore / Challenge / Space / Exhibition**、Popular / Latest。Explore に Tags。**Challenge = hashtag のお題**: `#PCDTokyo2026` `#Clock_Challege` `#SCREENS_CONTEXTUALIZED` `#MEDIA_ECOLOGY_TOKYO_NODE` `#Computer_Poetry_TOKYO_NODE` `#short_films`。**Playlists**(「Open Cube Vol.2」「CURATION FREE #2」) | Tag 一覧は JS 描画で個々の名は未取得 |
| Bright Moments | https://www.brightmoments.io/ (docs https://docs.brightmoments.io/) | 2021–2024 の 3 年計画(終了)。CryptoCitizens 10,000 + 生成/AI 約 20,000 を**都市ごと**にライブ mint | 語彙 = **都市**(Venice Beach・New York・Berlin・London・Mexico City・Tokyo・Buenos Aires・Paris・Venice)+ Collections / Golden Tokens / Finale。Quarterly の作家頁が一次資料(Golid・Asendorf) | — |
| Verse | https://verse.works/ | キュレータが出す editions | **Releases / Artworks / Featured Collections / Upcoming**。キュレータ名(Fakewhale、Heft)が単位 | Piter Pasma の作家頁あり |
| objkt | https://objkt.com/ (API https://data.objkt.com/v3/graphql) | Tezos NFT、fxhash の二次市場 | `tag` は自由記述: `#fxparams` `continuous animation` `co-creative` `#GeometryArt` | 本体 SPA で空、GraphQL から |
| Highlight | https://highlight.xyz/ (https://github.com/highlightxyz/generative-art) | mint 時に code が描く collection(hl-gen.js) | **Collections / Tokens / Traits**。`hl.token.setTraits()` で key-value の trait(Color: Blue) | Ethereum・Base・Arbitrum・Optimism・Polygon・Zora |

### 3B. Genuary のお題 全年(2021〜2026、186 本)

出典: https://genuary2021.github.io/prompts、https://genuary.art/2022/prompts、/2023/prompts、/2024/prompts、/2025/prompts、https://genuary.art/prompts(2026 = 現行。/2026/prompts は 404 で、現行 /prompts が「One color, one shape」始まりの 2026 分)。

| 年 | 日 | お題(原文) | 出題 |
|---|---|---|---|
| 2021 | 1 | TRIPLE NESTED LOOP | Piter Pasma |
| 2021 | 2 | Rule 30 (elementary cellular automaton) | Harold |
| 2021 | 3 | Make something human. | Sam Corzine |
| 2021 | 4 | Small areas of symmetry. | Louis-André Labadie |
| 2021 | 5 | Do some code golf! How little code can you write to make something interesting? | Louis-André Labadie |
| 2021 | 6 | Triangle subdivision. | Stevan Dedovic |
| 2021 | 7 | Generate some rules, then follow them by hand on paper. | Louis-André Labadie |
| 2021 | 8 | Curve only. | Licia He |
| 2021 | 9 | Interference patterns. | Sam Corzine |
| 2021 | 10 | TREE | Piter Pasma |
| 2021 | 11 | Use something other than a computer as an autonomous process | Piter Pasma |
| 2021 | 12 | Use an API (e.g. the weather). | Jonathan Barbeau |
| 2021 | 13 | Do not repeat. | Louis-André Labadie |
| 2021 | 14 | SUBDIVISION | Piter Pasma |
| 2021 | 15 | Let someone else decide the general rules of your piece. | Louis-André Labadie |
| 2021 | 16 | Circles only | Aaron Penne |
| 2021 | 17 | Draw a line, pick a new color, move a bit. | Louis-André Labadie |
| 2021 | 18 | One process grows, another process prunes. | Piter Pasma |
| 2021 | 19 | Increase the randomness along the Y-axis. | Piter Pasma |
| 2021 | 20 | No loops. | Aaron Penne |
| 2021 | 21 | function f(x){ DRAW(x); f(1*x/4); f(2*x/4); f(3*x/4); }(再帰・分数分割) | Harold |
| 2021 | 22 | Draw a line. Wrong answers only. | Louis-André Labadie |
| 2021 | 23 | #264653 #2a9d8f #e9c46a #f4a261 #e76f51, no gradients | Richard Vigniel |
| 2021 | 24 | 500 lines. | Aaron Penne |
| 2021 | 25 | Make a grid of permutations of something. | Piter Pasma |
| 2021 | 26 | 2D Perspective. | Stevan Dedovic |
| 2021 | 27 | Monochrome gradients without lines. | Aaron Penne |
| 2021 | 28 | Use sound. | Louis-André Labadie |
| 2021 | 29 | Any shape, none can touch. | Aaron Penne |
| 2021 | 30 | Replicate a natural concept (e.g. gravity, flocking, path following). | Jonathan Barbeau |
| 2021 | 31 | Search for "ENO'S OBLIQUE STRATEGIES" and use as your prompt | Piter Pasma |
| 2022 | 1 | Draw 10,000 of something. | Michael Lowe |
| 2022 | 2 | Dithering. | Anna Lucia |
| 2022 | 3 | Space. | Lionel Radisson |
| 2022 | 4 | The next next Fidenza. | Alexis André |
| 2022 | 5 | Destroy a square. | Thomas Lin Pedersen |
| 2022 | 6 | Trade styles with a friend. | Alex Naka |
| 2022 | 7 | Sol LeWitt Wall Drawing. | Roni Kaufman |
| 2022 | 8 | Single curve only. | Bruce Holmer |
| 2022 | 9 | Architecture. | Lionel Radisson |
| 2022 | 10 | Machine learning, wrong answers only. | Roni Kaufman |
| 2022 | 11 | No computer. | Lionel Radisson |
| 2022 | 12 | Packing (squares, circles, any shape…) | Richard Vigniel |
| 2022 | 13 | 800x80. | Ben Kovach |
| 2022 | 14 | Something you'd never make. | Devi Parikh |
| 2022 | 15 | Sand. | Thomas Lin Pedersen |
| 2022 | 16 | Color gradients gone wrong. | Quag |
| 2022 | 17 | 3 colors. | Deniz |
| 2022 | 18 | VHS. | GenerateMe |
| 2022 | 19 | Use text/typography. | Piter Pasma |
| 2022 | 20 | Make a sea of shapes. | Sabin T |
| 2022 | 21 | Combine two (or more) of your pieces from previous days to make a new piece. | Michael Lowe |
| 2022 | 22 | Make something that will look completely different in a year. | Michael Lowe |
| 2022 | 23 | Abstract vegetation. | Louis-André Labadie |
| 2022 | 24 | Create your own pseudo-random number generator and visually check the results. | Quag |
| 2022 | 25 | Perspective. | Jos Vromans |
| 2022 | 26 | Airport carpet. | Quag |
| 2022 | 27 | #2E294E #541388 #F1E9DA #FFD400 #D90368 | Michael Lowe |
| 2022 | 28 | Self portrait. | Michael Lowe |
| 2022 | 29 | Isometric perspective. | Chris Ried |
| 2022 | 30 | Organic looking output using only rectangular shapes. | Bart Simons |
| 2022 | 31 | Negative space. | Michael Lowe |
| 2023 | 1 | Perfect loop / Infinite loop / endless GIFs | arikwex, Reva |
| 2023 | 2 | Made in 10 minutes | Daniel Simu (hapiel) |
| 2023 | 3 | Glitch Art | generetame |
| 2023 | 4 | Intersections | Yazid |
| 2023 | 5 | Debug view | jenslabs |
| 2023 | 6 | Steal Like An Artist | Daniel Catt |
| 2023 | 7 | Sample a color palette from your favorite movie/album cover | Luca |
| 2023 | 8 | Signed Distance Functions | Camille Roux |
| 2023 | 9 | Plants | Lionel Radisson |
| 2023 | 10 | Generative music | Steve Pikelny (scyclow) |
| 2023 | 11 | Suprematism | Eric Davidson |
| 2023 | 12 | Tessellation | Devi |
| 2023 | 13 | Something you've always wanted to learn | huminaboz |
| 2023 | 14 | Aesemic | Melissa Wiederrecht |
| 2023 | 15 | Sine waves | Jess Hewitt, Yazid |
| 2023 | 16 | Reflection of a reflection | Ryan |
| 2023 | 17 | A grid inside a grid inside a grid | Andrea Diotallevi |
| 2023 | 18 | Definitely not a grid | Daniel Simu (hapiel) |
| 2023 | 19 | Black and white | Lionel Radisson |
| 2023 | 20 | Art Deco | Artefakt |
| 2023 | 21 | Persian Rug | arikwex |
| 2023 | 22 | Shadows | Luca |
| 2023 | 23 | More Moiré | Daniel Simu (hapiel) |
| 2023 | 24 | Textile | Daniel Simu (hapiel) |
| 2023 | 25 | Yayoi Kusama | Anna Lucia |
| 2023 | 26 | My kid could have made that | Melissa Wiederrecht |
| 2023 | 27 | In the style of Hilma Af Klint | Roni |
| 2023 | 28 | Generative poetry | Roni |
| 2023 | 29 | Maximalism | Steve Pikelny, Yazid |
| 2023 | 30 | Minimalism | Steve Pikelny, Yazid |
| 2023 | 31 | Deliberately break one of your previous images... | Alex Naka, RalenArc |
| 2024 | 1 | Particles, lots of them. | Melissa Wiederrecht, Nicolas Barradeau |
| 2024 | 2 | No palettes. Generative colors, procedural colors, emergent colors. | Luis Fraguada |
| 2024 | 3 | Droste effect. | Stranger in the Q |
| 2024 | 4 | Pixels. | Piter Pasma |
| 2024 | 5 | In the style of Vera Molnár (1924-2023). | Melissa Wiederrecht, Piter Pasma |
| 2024 | 6 | Screensaver. | Nicolas Barradeau, Yazid, Jess Hewitt |
| 2024 | 7 | Progress bar / indicator / loading animation. | Piter Pasma |
| 2024 | 8 | Chaotic system. | Darien Brito |
| 2024 | 9 | ASCII. | Camille Roux |
| 2024 | 10 | Hexagonal. | greweb |
| 2024 | 11 | In the style of Anni Albers (1899-1994). | Roni Kaufman |
| 2024 | 12 | Lava lamp. | Melissa Wiederrecht |
| 2024 | 13 | Wobbly function day. | Piter Pasma |
| 2024 | 14 | Less than 1KB artwork. | Heeey |
| 2024 | 15 | Use a physics library. | Amy Goodchild |
| 2024 | 16 | Draw 10 000 of something. | Bruce Holmer, Michael Lowe |
| 2024 | 17 | Inspired by Islamic art. | Melissa Wiederrecht |
| 2024 | 18 | Bauhaus. | Chris Barber |
| 2024 | 19 | Flocking. | Shaderism |
| 2024 | 20 | Generative typography. | Roni Kaufman |
| 2024 | 21 | Use a library that you haven't used before. | Neel Shivdasani |
| 2024 | 22 | Point - line - plane. | Paolo Curtoni |
| 2024 | 23 | 16×16. | Marc Edwards |
| 2024 | 24 | Impossible objects (undecided geometry). | Jorge Ledezma |
| 2024 | 25 | Recreate patterns, textures, or shapes you've photographed. | Piter Pasma |
| 2024 | 26 | Grow a seed. | Monokai |
| 2024 | 27 | Code for one hour. At the one hour mark, you're done. | Amy Goodchild |
| 2024 | 28 | Skeuomorphism. | Melissa Wiederrecht |
| 2024 | 29 | Signed Distance Functions. | Melissa Wiederrecht, Camille Roux |
| 2024 | 30 | Shaders. | Melissa Wiederrecht |
| 2024 | 31 | Generative music / Generative audio / Generative sound. | Neel Shivdasani, Monokai |
| 2025 | 1 | Vertical or horizontal lines only. | Stranger in the Q |
| 2025 | 2 | Layers upon layers upon layers. | Monokai |
| 2025 | 3 | Exactly 42 lines of code. | Roni Kaufman |
| 2025 | 4 | Black on black. | Stranger in the Q |
| 2025 | 5 | Isometric Art (No vanishing points). | P1xelboy |
| 2025 | 6 | Make a landscape using only primitive shapes. | Jonathan Barbeau |
| 2025 | 7 | Use software that is not intended to create art or images. | Camille Roux |
| 2025 | 8 | Draw one million of something. | Piter Pasma |
| 2025 | 9 | The textile design patterns of public transport seating. | Piter Pasma |
| 2025 | 10 | You can only use TAU in your code, no other number allowed. | Darien Brito |
| 2025 | 11 | Impossible day - Try to do something that feels impossible for you to do. | Rachel Ehrlich, Recurse Center |
| 2025 | 12 | Subdivision. | Melissa Wiederrecht |
| 2025 | 13 | Triangles and nothing else. | Heeey |
| 2025 | 14 | Pure black and white. No gray. | Melissa Wiederrecht |
| 2025 | 15 | Design a rug. | Melissa Wiederrecht |
| 2025 | 16 | Generative palette. | Stranger in the Q |
| 2025 | 17 | What happens if pi=4? | Roni Kaufman |
| 2025 | 18 | What does wind look like? | Melissa Wiederrecht |
| 2025 | 19 | Op Art. | Melissa Wiederrecht |
| 2025 | 20 | Generative Architecture. | Melissa Wiederrecht |
| 2025 | 21 | Create a collision detection system (no libraries allowed). | Darien Brito |
| 2025 | 22 | Gradients only. | Melissa Wiederrecht |
| 2025 | 23 | Inspired by brutalism. | Melissa Wiederrecht, Roni Kaufman |
| 2025 | 24 | Geometric art - pick either a circle, rectangle, or triangle and use only that geometric shape. | Bruce Holmer |
| 2025 | 25 | One line that may or may not intersect itself | Bruce Holmer, Chris Barber, Heeey, Monokai |
| 2025 | 26 | Symmetry. | Melissa Wiederrecht |
| 2025 | 27 | Make something interesting with no randomness or noise or trig. | Melissa Wiederrecht |
| 2025 | 28 | Infinite Scroll. | Sophia (fractal kitty) |
| 2025 | 29 | Grid-based graphic design. | Melissa Wiederrecht |
| 2025 | 30 | Abstract map. | Melissa Wiederrecht |
| 2025 | 31 | Pixel sorting. | Melissa Wiederrecht |
| 2026 | 1 | One color, one shape | Piero |
| 2026 | 2 | Twelve principles of animation | Anna Lucia |
| 2026 | 3 | Fibonacci forever | PaoloCurtoni |
| 2026 | 4 | Lowres | Manuel Larino |
| 2026 | 5 | Write "Genuary" | Piero |
| 2026 | 6 | Lights on/off | George Henry Rowe |
| 2026 | 7 | Boolean algebra | PaoloCurtoni |
| 2026 | 8 | A City | PaoloCurtoni |
| 2026 | 9 | Crazy automaton | PaoloCurtoni |
| 2026 | 10 | Polar coordinates | Sophia (fractal kitty) |
| 2026 | 11 | Quine | Manuel Larino |
| 2026 | 12 | Boxes only | Stranger in the Q |
| 2026 | 13 | Self portrait | Jos Vromans |
| 2026 | 14 | Everything fits perfectly | Roni |
| 2026 | 15 | Create an invisible object where only the shadows can be seen | P1xelboy |
| 2026 | 16 | Order and disorder | Ivan Dianov |
| 2026 | 17 | Wallpaper group | Ivan Dianov |
| 2026 | 18 | Unexpected path | Baret LaVida |
| 2026 | 19 | 16x16 | Jos Vromans |
| 2026 | 20 | One line | Jos Vromans |
| 2026 | 21 | Bauhaus Poster | Piero |
| 2026 | 22 | Pen plotter ready | Sophia (fractal kitty) |
| 2026 | 23 | Transparency | PaoloCurtoni |
| 2026 | 24 | Perfectionist's nightmare | Sophia (fractal kitty) |
| 2026 | 25 | Organic Geometry | Manuel Larino |
| 2026 | 26 | Recursive Grids | Piero |
| 2026 | 27 | Lifeform | Manuel Larino |
| 2026 | 28 | No libraries, no canvas, only HTML elements | Piero |
| 2026 | 29 | Genetic evolution and mutation | Monokai |
| 2026 | 30 | Its not a bug, its a feature | Bart Simons |
| 2026 | 31 | GLSL day | Piero |

## 4. Processing / p5.js 公式 Examples

出典: https://processing.org/examples、https://github.com/processing/processing-examples、https://p5js.org/examples/。純/記憶は (index, time, position) の純関数で描けるか。

### 4A. Processing(Basics 11 カテゴリ + Topics 17 + Demos)

| 区分 | カテゴリ | 名前 | 1 行 | 純/記憶 |
|---|---|---|---|---|
| Basics | Math | AdditiveWave | 複数 sin を足した波 | 純 |
| Basics | Math | Noise1D / Noise2D / Noise3D | noise を線・面・時間で | 純 |
| Basics | Math | NoiseWave | noise の波線 | 純 |
| Basics | Math | PolarToCartesian | 極座標で回る点 | 純 |
| Basics | Math | Sine / SineCosine / SineWave | 三角関数の動き | 純 |
| Basics | Math | Graphing2DEquation | 画素ごとに式を評価 | 純 |
| Basics | Math | Random / Distance1D / Distance2D / Arctangent / IncrementDecrement / Interpolate / OperatorPrecedence | 数学の基礎 | 純(Random は hash 化) |
| Basics | Transform | Arm / Rotate / Scale / Translate / RotateXY / RotatePushPop | 変換 | 純 |
| Basics | Form | RegularPolygon / Star / Bezier / PieChart / Vertices / TriangleStrip / ShapePrimitives / 3D Primitives | 形 | 純 |
| Basics | Color | Hue / Saturation / Brightness / ColorWheel / LinearGradient / RadialGradient / Relativity / WaveGradient | 色 | 純 |
| Basics | Arrays / Control / Data / Input / Structure / Image / Web / Objects / Typography | 言語の基礎、入力、画像、文字 | 資源・入力(純/記憶の外) |
| Topics | Fractals and L-Systems | Mandelbrot / Tree / Koch / PenroseTile / PenroseSnowflake / Pentigree / LSystem | フラクタル、L-system | 純(世代 = 時刻) |
| Topics | Geometry | Icosahedra / SpaceJunk / Toroid / Vertices / RGBCube / Cubic Grid | 3D 幾何 | 純 |
| Topics | Motion | Bounce / BouncyBubbles / Brownian / CircleCollision / Reflection1/2 / Linear / MovingOnCurves / Morph | 動き | 記憶(Bounce・Bubbles・Brownian・Collision・Reflection)、純に書き換え可(Linear・MovingOnCurves・Morph) |
| Topics | Simulate | Flocking / ParticleSystem / MultipleParticleSystems / SimpleParticleSystem / Smoke / Spring / Springs / SpringsAndForces / Chain / SoftBody / Fluid | 物理・群れ | 記憶 |
| Topics | Cellular Automata | GameOfLife / Wolfram / Conway / Spore1/2 | セルオートマトン | 記憶 |
| Topics | Interaction | Follow1/2/3 / Reach1/2/3 / Tickle / Wiggle | 追従 | 記憶 + 入力 |
| Topics | Vectors | AccelerationWithVectors / BouncingBall / VectorMath / Motion | ベクトル | 記憶 |
| Topics | Effects | (公式には無い。Shaders / Image Processing が相当)| — | — |
| Topics | Shaders / Textures / Lights / Camera / Advanced Data / File IO / GUI / Sound / Drawing / Image Processing / Continuous Lines / Pattern | 描画資源・入力 | 資源 |
| Demos | Performance | Esfera / CubicGrid / CubicGridImmediate / CubicGridRetained / LineRendering / StaticParticlesRetained / DynamicParticlesImmediate | 大量描画 | 純(Esfera・CubicGrid)、記憶(Particles) |
| Demos | Graphics | Planets / Trefoil / Wiggling / Yellowtail / Bezier Patch / Reach | 3D 見本 | 混在 |
| Demos | Tests | 性能試験 | — | — |

### 4B. p5.js(55 件)

| 区分 | 名前 | 1 行 | 純/記憶 |
|---|---|---|---|
| Repetition | Noise / Color Wheel / Bezier / Recursive Tree / Kaleidoscope / Radial Gradient | 反復と関数の絵 | 純 |
| Angles And Motion | Sine and Cosine / Triangle Strip / Interaction with Motion | 角度と動き | 純(Sine and Cosine)、他は入力 |
| 3D | Geometries / Custom Geometry / Materials / Orbit Control / Adjusting Positions with a Shader / Shader as a Texture | 3D と shader | 純 |
| Animation And Variables | Drawing Lines / Animation with Events / Linear Interpolation / Non-Orthogonal Reflection / Bounce / Bouncing Bubbles / Morph | 動き | 記憶(Bounce・Bubbles)、純に書き換え可 |
| Simulation | Flocking / Soft Body / Forces / Smoke / Game of Life / Wolfram CA / Particles / Random Walk / Snowflakes / Mandelbrot / L-System / Penrose | 物理・オートマトン | 記憶(Flocking・Soft Body・Forces・Smoke・GoL・Random Walk・Snowflakes)、純(Mandelbrot・L-System・Penrose) |
| Games | Snake / Ping Pong / Circle Clicker | ゲーム | 記憶 + 入力 |
| Imported Media / Loading / Sound / Input / DOM / Typography / Math(Map・Random・Noise)/ Accessibility | 資源・入力 | 資源 |

概数: 純 ≈ 20、記憶 ≈ 18、資源/入力 ≈ 17。

## 5. 見つからなかった物

- fxhash・OpenProcessing・Shadertoy の**タグ一覧**(当日 402/403。語彙の付き方は Art Blocks の Aesthetic/Theme と NEORT の Playlist/Challenge のみ一次で確認)。
- Genuary の創始の明記は genuary.art 本体には無し(Piter Pasma 本人サイトの「lead organizer 2021/2022」と karaman.is の「first #genuary」が根拠)。
- 技法表の代表作 URL のうち「URL 未確認」と書いた物(Molnár・Nees・Mohr の作品頁、Coding Train の各回、Shadertoy の個別作)。
- Aaron Penne の拠点、Iskra Velitchkova の道具、Yazid の拠点。
- 岡村健太・Jen Lowe(作家としての一次資料無し)。
- #plottertwitter の一次ページ(X はログイン要)。

## Sources(取得 2026-09-18)

- https://genuary.art/ · https://genuary.art/prompts · https://genuary2021.github.io/ · https://genuary2022.github.io/prompts · https://github.com/genuary2021 · https://piterpasma.nl/ · https://karaman.is/blog/2021/2/genuary-2021 · https://docs.fxhash.xyz/ · https://docs.fxhash.xyz/creating-on-fxhash/releasing-your-project/minting-interface-walkthrough.md · https://docs.fxhash.xyz/collecting-on-fxhash/platform-overview/primary-market.md · https://neort.io/explore · https://neort.io/challenge · https://www.brightmoments.io/ · https://docs.brightmoments.io/ · https://verse.works/ · https://data.objkt.com/v3/graphql · https://github.com/highlightxyz/generative-art · https://day.processing.org/about/ · https://processingfoundation.org/ · https://drawingbots.com/plottertwitter · https://penplotter.art/p/ptpx-plotter-postcards · https://buttondown.com/ptpx · https://genuary2021.github.io/prompts · https://genuary.art/2022/prompts · https://genuary.art/2023/prompts · https://genuary.art/2024/prompts · https://genuary.art/2025/prompts
- https://www.artblocks.io/ · https://neort.io/ · https://day.processing.org/
- https://processing.org/examples · https://github.com/processing/processing-examples · https://p5js.org/examples/
- 作家: https://www.tylerxhobbs.com/about · https://layer.com/artists/zach-lieberman/interview · https://mattdesl.substack.com/p/notes-on-meridian · https://sierra.mattdesl.com/ · https://reas.com/ · https://www.katevassgalerie.com/manoloide · https://github.com/inconvergent · https://inconvergent.net/ · https://www.lerandom.art/artists/jared-tarbell · https://sasj.nl/portfolio/daily/ · https://ravenkwok.com/about/ · https://www.rightclicksave.com/article/an-interview-with-william-mapan · https://www.fxhash.xyz/u/Piter%20Pasma · https://www.rightclicksave.com/article/the-interview-piter-pasma-generative-art · https://www.artblocks.io/artists/iskra-velitchkova · https://avantarte.com/insights/avant-essay/iskra-velitchkova-notes-on-creation · https://www.aaronpenne.io/ · https://www.artblocks.io/collection/archetype-by-kjetil-golid · https://www.brightmoments.io/quarterly/kjetilgolid · https://botto.com · https://github.com/Quasimondo · https://www.moma.org/calendar/exhibitions/5535 · https://refikanadol.com/works/ · https://refikanadol.com/refik-anadol/ · https://www.eyesofpanda.com/gallery/ · https://www.eyesofpanda.com/about/ · https://www.artblocks.io/collection/ringers-by-dmitri-cherniak · https://www.rightclicksave.com/article/an-interview-with-dmitri-cherniak · https://www.dalosdov.com/writing/snowfro-interview · https://emilyxie.art/projects/memories_of_qilin · https://zancan.art/Series/Garden-Monoliths · https://www.brightmoments.io/quarterly/kimasendorf · https://www.artblocks.io/collection/kernels-by-julian-hespenheide · https://thegreatdiscontent.com/interview/joshua-davis/ · https://natureofcode.com/introduction/ · https://github.com/ertdfgcvb · https://ivandianov.com/works/ · https://experiments.withgoogle.com/hydra · https://www.kaloh.xyz/p/-yazid-is-creating-simple-minimal · https://ccbt.rekibun.or.jp/players/takawo-shunsuke · https://cenkhor.org/ · https://www.rightclicksave.com/article/generative-art-and-japans-bright-future · https://designing.jp/tha-nakamura · https://www.stirworld.com/see-features-japanese-artist-yuma-yanagisawa-and-the-new-arts-discourse · https://www.deconbatch.com/ · https://junkiyoshi.com/about/ · https://www.ryojiikeda.com/archive/works/
- 技法: https://www.artblocks.io/collection/fidenza-by-tyler-hobbs · http://paulbourke.net/fractals/clifford/ · https://www.karlsims.com/rd.html · http://algorithmicbotany.org/papers/ · https://10print.org/ · https://www.red3d.com/cwr/boids/ · https://inconvergent.net/generative/differential-line/ · https://inconvergent.net/generative/sand-spline/ · https://github.com/mxgmn/WaveFunctionCollapse · https://www.emohr.com/ · http://www.complexification.net/gallery/machines/substrate/ · https://iquilezles.org/articles/warp/ · https://iquilezles.org/articles/distfunctions/ · https://github.com/kimasendorf/ASDFPixelSort · https://cargocollective.com/sagejenson/physarum
- 取得不可(当日): https://www.fxhash.xyz/(402)· https://openprocessing.org/(403)· https://www.shadertoy.com/(403)· https://genuary.art/2026/prompts(404)
