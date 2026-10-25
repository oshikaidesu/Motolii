# CapCut の効果を「しくみ」でまとめる（10/26）

CapCut の 4,453 本（`reports/capcut_map.md`）を、中で動いている**しくみ（アルゴリズム）**ごとに分けた。
CapCut は同じしくみを、つまみの少ない「型」として何十本も並べている。
なので、つまみの多い開いた実装を 1 つ借りて、型（プリセット）をたくさん持てば、族ごとまとめて覆えて、CapCut より自由にできる。

CapCut のファイル（シェーダー・Lua・絵・色の表）は取らない。中も見ない。使うのは名前と組と、つまみの名前だけ。

## 疲れている時はここだけ

- しくみで分けると **61 族**。ほかに、名前だけでは中身が分からない物が **224 本**。
- 多い順に上の **20 族で 2,633 本（59%）**。
- 作る順のはじめの 3 つ（入り出の動きの型、グリッチ・VHS、光る・点滅）は、どれも今ある物と手元の ISF だけでできる。この 3 つで **1,015 本（23%）**。
- 7 番目（色の表を読む口）まで進めると **2,701 本（61%）**。
- 大きな仕事は 2 つ。**粒の絵**（429 本、CC0 の絵と粒を出す仕組み）と、**人の切り抜き**（229 本、MediaPipe）。

## 印の読み方

- **◎ 動く**：今の Motolii（`filament/effects/`・animators・キー・ふるまい）で、同じしくみが出る。足りないのは型だけ。
- **○ 部品はある**：部品は入っている。組み合わせるか、手元の ISF を足せば出る。
- **△ 入れていない**：借りる先は手元にあるか、はっきりしている。まだ Motolii に入れていない。
- **✕ 無い**：大きな仕組みが要る（人の切り抜き）か、開いた見本が見つかっていない。
- 「手元」＝ `research_notes/isf/ISF-Files/ISF`（MIT、371 本）か `research_notes/threejs_shaders`（MIT）にある物。
- 「入っている」＝ `filament/effects/` にある物。`reports/alight_isf/abd_sheet.png`・`reports/capcut_try/vhs_sheet.png` で写真に掛けて効いた物も含む。

## 表（CapCut の数が多い順）

| # | 族 | CapCut の数 | 借りる先 | Motolii で今動くか |
|---|---|---|---|---|
| 1 | 色の味付け（フィルターの LUT） | 371 | 読む口：three.js LUTCubeLoader・LUTPass（MIT、手元に無い）／色：Motolii の色の段で作る | ○ 色の段で寄せられる。LUT を読む口が無い |
| 2 | 粒の絵が舞う・降る（雪・花びら・ハート・蝶・泡・雨） | 231 | 物の copies・scatter・段の gravity（入っている）＋絵：Kenney Particle Pack（CC0）・Twemoji（CC BY 4.0）／つまみの見本：PlayCanvas ParticleEmitter（MIT） | ○ 並べて落とす仕組みはある。絵が無い |
| 3 | 光る・点滅（フラッシュ・ストロボ・闪白・闪黑） | 215 | ISF Fade Color・Strobe（手元）、RGB Strobe（入っている） | ◎ |
| 4 | 波・ゆがみ・渦（波紋・ねじれ・液体） | 177 | ISF Ripples・Twirl・Noise Displace（入っている）、Shockwave・Water Drop、gl-transitions crosswarp（手元） | ◎ |
| 5 | グリッチ（帯ずれ・ブロック・信号の乱れ） | 147 | Bad TV・Pixel Shifter・Time Glitch RGB・datamosh（入っている）、VHS Glitch・Glitch Shifter・Stylize Glitch（手元、効いた） | ◎ |
| 6 | 光のにじみ（glow・bloom・halo） | 139 | 段の bloom・ISF Bloom・Glow（入っている） | ◎ |
| 7 | 帯・格子に割れる（スライス・パズル） | 123 | Grid Flip（入っている）、Window Slice・Sliding Strips・Random Squares（手元） | ○ |
| 8 | つなぎ目：押し出し・滑り | 110 | gl-transitions directional・Slides・squeeze（ISF Directional・Squeeze が手元） | △ 手元の ISF を入れるだけ |
| 9 | 枠を重ねる（スマホ・カメラ・写真・窓） | 107 | 形の stroke（ThorVG）、Tiny Date Time Overlay（手元） | ○ 枠の絵は形で描く |
| 10 | ぼかし・ピント | 106 | Fast Blur・Linear Blur・VVMotionBlur・Frosted Glass（入っている）、gl-transitions DefocusBlur | ◎ |
| 11 | 人の切り抜き＋ほかの効果（人だけ・背景だけ） | 104 | MediaPipe Image Segmenter（Apache-2.0、手元に無い） | ✕ 切り抜きが無い |
| 12 | 古いフィルム・粒・ノイズ | 99 | film・grain・fine_grain・sepia・technicolor（入っている）、v002 Light Leak・Film Burn（手元） | ◎ |
| 13 | 紙が破れる・割れる・砕ける | 97 | 見つからない（候補：Worley Cells の区切り、入っている） | ✕ 見本を探す |
| 14 | 入り出：滑って入る・落ちる | 96 | animators slide_in・slide_out（入っている）、Animista（BSD-2）・Animate.css v3.7.2（MIT） | ◎ |
| 15 | 光漏れ・光の筋・スキャン・フレア | 92 | God Rays・Lens Flare（入っている）、v002 Light Leak・Linear Gradient（手元） | ○ |
| 16 | VHS・古いテレビ・DV | 87 | crt_finish・Bad TV（入っている）、VHS Glitch・v002 CRT・TV Static（手元、効いた） | ◎ |
| 17 | 絵・漫画・落書き（線画・網点・油絵） | 87 | Edges・CMYK Halftone・halftone・Line Screen（入っている）、Doodler・Sketch・Toon・Crosshatch（手元） | ○ |
| 18 | 拭き取り・幕・扉・ブラインド | 82 | Wipe 4 方向（入っている）、Directional Wipe・Doorway・Window Blinds（手元） | ◎ |
| 19 | 色を変える（白黒・色をのせる・暗く・周辺減光） | 82 | Color Monochrome・Duotone・LGG・Multi Hue Shift・vignette（入っている） | ◎ |
| 20 | きらきら（光る所に十字の星） | 81 | 明るい所：Bloom（入っている）、星：ISF Star（入っている）・Kenney の星（CC0）、点を選ぶ所：見つからない | △ |
| 21 | 人の縁が光る・燃える・電気 | 76 | MediaPipe ＋ Edges・Glow（入っている）・ISF Neon（手元） | ✕ 切り抜きが無い |
| 22 | 入り出：揺れる・振り子 | 68 | ふるまい noise・oscillator、animators swing_3d（入っている） | ◎ |
| 23 | フィルム風の色（フィルター） | 64 | film・technicolor・bleach_bypass・sepia・grain（入っている）＋1 の LUT の口 | ○ |
| 24 | 残像・分身・灵魂出窍 | 62 | Echo Trace・RGB Trails 3.0（入っている）、Ghosting（手元）、three.js AfterimageShader（手元）、CainCamera（Apache-2.0） | ◎ |
| 25 | RGB ずれ・色の分散・プリズム | 57 | rgbshift（入っている）、Chromatic Aberration・Channel Slide・Chroma Zoom（手元、効いた） | ◎ |
| 26 | つなぎ目：立方体・カードで裏返る | 56 | cube（入っている）、gl-transitions SimpleFlip・BookFlip | ○ |
| 27 | 画面：拍でズーム（zoom punch・脈打つ） | 55 | キー scale・ISF Zoom・ふるまい oscillator（入っている）、CainCamera 缩放 | ◎ |
| 28 | ネガ・反転・熱カメラ | 52 | Color Invert・RGB Invert・Thermal Camera・False Color（入っている） | ◎ |
| 29 | 花火・爆発・火花（はじける粒） | 51 | レシピ burst.js・confetti.js・点群 Appear Eruption（入っている）＋Kenney の火花（CC0） | ○ 絵が無い |
| 30 | 虹・色が回る（ディスコ・サイケ） | 51 | Multi Hue Shift（入っている）、Color Phase（手元） | ◎ |
| 31 | ページめくり・折る | 51 | Inverted Page Curl（入っている、BSD-3）、gl-transitions Fold | ◎ |
| 32 | 入り出：回って入る | 51 | キー rotate、Animista rotate-in | ◎ |
| 33 | 画面：動かす（横ずれ・伸ばす・カード） | 49 | キー x・y・scale_x・skew・2.5D（入っている） | ◎ |
| 34 | モザイク・ピクセル（ピクセルソートも） | 47 | Pixellate・Mosaic・Multi-Pixellate・Hexagonalize（入っている）、Sorting Smear（手元） | ◎ |
| 35 | ネオンの縁取り（画面全体） | 44 | Edges・Edge Blowout・Glow（入っている）、ISF Neon・three.js SobelOperatorShader（手元） | ○ |
| 36 | 入り出：弾む・ばね | 44 | animators bounce_in・spring_in（入っている） | ◎ |
| 37 | つなぎ目：ズームで飛び込む | 43 | gl-transitions CrossZoom・SimpleZoom（手元） | △ 入れるだけ |
| 38 | 炎・燃える | 42 | Burn Away（入っている）、ISF Burn・Film Burn・Undulating Burn Out（手元）、Kenney の炎（CC0） | ○ |
| 39 | 文字：1 字ずつ（タイプライター） | 42 | animators print_on・print_off（入っている）、anime.js stagger（MIT） | ◎ |
| 40 | 顔・目・頭（顔の点） | 39 | MediaPipe Face Landmarker（Apache-2.0）＋Bump Distortion（入っている） | ✕ |
| 41 | 入り出：3D で裏返る・傾く | 39 | rotate_x・rotate_y、animators swing_3d（入っている） | ◎ |
| 42 | 魚眼・レンズ・虫めがね | 38 | Bump Distortion・Sphere Map（入っている）、Fly Eye（手元）、GPUImage Bulge（BSD-3） | ◎ |
| 43 | つなぎ目：回る・風車 | 38 | Radial（入っている）、Rotate Scale Fade・Pinwheel・Swirl（手元） | △ 入れるだけ |
| 44 | 入り出：拡大・縮小・脈打つ | 37 | キー scale、ふるまい oscillator | ◎ |
| 45 | 画面：揺れ（シェイク） | 36 | ふるまい noise（入っている）、ISF Shake（手元）、CainCamera 抖动 | ◎ |
| 46 | 溶ける・にじんで替わる | 33 | Fade（入っている）、Perlin Transition・Luma Transition・Luminance Melt（手元） | ○ |
| 47 | 画面：3D の空間・カメラワーク（运镜） | 32 | 2.5D・Hyperspace（入っている）、奥行き：Depth Anything V2 Small（Apache-2.0、要確認） | ○（3D 写真は ✕） |
| 48 | 画面分割（2・3・4・9 画面） | 29 | Quad Tile・Collage（入っている）、MultiFrame・Replicate（手元）、CainCamera の分割 7 種 | ◎ |
| 49 | 人の残像・分身 | 27 | MediaPipe ＋ Echo Trace（入っている） | ✕ |
| 50 | 形で開く（丸・ハート・時計） | 27 | Radial（入っている）、Circle Open・Heart Transition（手元） | ○ |
| 51 | 稲妻・電気 | 26 | three.js LightningStrike（MIT、手元に無い）＋Glow | ✕ |
| 52 | 煙・霧・雲 | 24 | 段の fog・Simplex Noise（入っている）、Kenney の煙（CC0）、Perlin（手元） | ○ |
| 53 | つなぎ目：揺れ・弾む | 24 | ふるまい noise を 2 本に、ISF Bounce（手元） | ○ |
| 54 | コマ止め・コマ落とし・速さ | 23 | Freeze Frame・Random Freeze（手元）、datamosh（入っている） | △ |
| 55 | 鏡・万華鏡 | 22 | mirror・kaleido・Kaleidoscope Tile（入っている） | ◎ |
| 56 | 人の周りに粒・飾り（翼・ハート） | 22 | MediaPipe ＋ 2 の粒 | ✕ |
| 57 | 白黒（フィルター） | 19 | Color Monochrome（入っている） | ◎ |
| 58 | ホラー（暗く・赤・揺れ・血） | 18 | ほかの族の組み合わせ、Night Vision（手元） | ○ 血の絵は無い |
| 59 | 画面：回る | 14 | キー rotate | ◎ |
| 60 | カウントダウン・文字を重ねる | 14 | 文字＋print_on、Digital Clock（入っている） | ○ |
| 61 | 入り出：じわっと出る | 10 | animators fade_in・fade_out | ◎ |
| — | 名前だけでは分からない | 224 | 見てから | — |

数えると：◎ 31 族（2,103 本）、○ 18 族（1,440 本）、△ 5 族（295 本）、✕ 7 族（391 本）。
（「動く」は同じしくみが出る、という意味。CapCut と同じ見た目かは型を作って絵で見ないと分からない。）

## 作る順（1 手間で覆う数が多い順）

手間はざっくりの見当：1＝入っている物と手元の ISF を入れて型を並べるだけ、3＝新しい口を 1 つ足す、5＝大きな仕組みを 1 つ入れる。

| 順 | まとめて作る物 | 族（表の #） | 覆う数 | 手間 | 1 手間あたり | ここまで |
|---|---|---|---|---|---|---|
| 1 | 入り出・文字の動きの型（向き・イーズ違いを並べる） | 14・22・32・36・39・41・44・61 | 387 | 1 | 387 | 387（9%） |
| 2 | グリッチ・VHS・RGB ずれ・モザイク・コマ止め（試した ISF を入れる） | 5・16・25・34・54 | 361 | 1 | 361 | 748（17%） |
| 3 | 光る・点滅・ネガ | 3・28 | 267 | 1 | 267 | 1,015（23%） |
| 4 | つなぎ目：gl-transitions の残り（手元に ISF に直した物 68 本） | 8・26・31・37・43・46・50・53 | 382 | 1.5 | 255 | 1,397（31%） |
| 5 | ゆがみ・レンズ・鏡・ぼかし | 4・10・42・55 | 343 | 1.5 | 229 | 1,740（39%） |
| 6 | 光のにじみ・ネオン・光漏れ・虹 | 6・15・30・35 | 326 | 1.5 | 217 | 2,066（46%） |
| 7 | 色：LUT を読む口＋自分で作った色の表 | 1・12・19・23・57 | 635 | 3 | 212 | 2,701（61%） |
| 8 | 拭き取り・帯に割れる・画面分割・枠 | 7・9・18・48 | 341 | 2 | 170 | 3,042（68%） |
| 9 | 画面の動き（揺れ・拍ズーム・回る・3D 空間） | 27・33・45・47・59 | 186 | 1.5 | 124 | 3,228（72%） |
| 10 | 粒の絵（CC0 の絵＋粒を出す仕組み） | 2・20・29・38・52 | 429 | 4 | 107 | 3,657（82%） |
| 11 | 絵・漫画・残像 | 17・24 | 149 | 1.5 | 99 | 3,806（85%） |
| 12 | 人の切り抜き（MediaPipe） | 11・21・49・56 | 229 | 5 | 46 | 4,035（91%） |
| 13 | 見本が無い物を探す：紙・割れる、稲妻、ホラーの絵、文字 | 13・51・58・60 | 155 | 4 | 39 | 4,190（94%） |
| 14 | 顔の点（Face Landmarker） | 40 | 39 | 3 | 13 | 4,229（95%） |
| — | 名前だけでは分からない 224 本は、見てから族に入れる | — | 224 | — | — | 4,453 |

メモ：
- 1〜6 は、ほぼ「型を作る」仕事。新しいしくみはほとんど要らない。
- 7 は数がいちばん多い（635）。LUT を読む口は 1 つで済む。手間の多くは色の表を作る所。
- 10 の「粒を出す仕組み」は 2・20・29・38・52 と、12 の人の周りの粒（56）にも使える。先に作ると 12 が軽くなる。
- 12 の人の切り抜きは、ほかの族の効果を「人だけ／背景だけ」にかける口。11 の 104 本はほとんどが、切り抜きの上にほかの族を載せた物。

## 族ごとの中身

書き方：例（CapCut の名前）／しくみ（1 行）／借りる先／いま／Motolii で足すつまみ（CapCut より多い物）／取れない物と代わり／当て方。
当て方：**高**＝名前の言葉がそのしくみをはっきり指す、**中**＝言葉は合うが別のしくみもありうる、**低**＝CapCut の組（光・動き など）だけで入れた。

### 1. 色の味付け（フィルターの LUT）— 371
- 例：Blue Hour、Candlelight、Cinematic Dusk、Sea Blue、冷蓝
- しくみ：色ごとに行き先を決めた表（3D LUT）で塗り替える。
- 借りる先：読む口は three.js LUTCubeLoader・LUTPass（MIT、手元に無い）。色そのものは Motolii の色の段（LGG・色温度・Duotone・Vibrance）で作り、.cube で持つ。
- いま：○ 色の段で近くはできる。LUT のファイルを読む口が無い。
- 足すつまみ：CapCut は「色の濃さ」1 つ。Motolii は 強さ・影・中間・明るい所ごとの色・肌を守る・前後の比べ。
- 取れない：CapCut の色の表。名前（Dune・Wong Kar-wai・Like Jennie など）も使わない。→ 自分の色で作る。RawTherapee のフィルムの LUT はライセンスが分からないので使わない。
- 当て方：しくみは確か（フィルターは全部 LUT）。どんな色かは 103 本が名前で分からない。

### 2. 粒の絵が舞う・降る — 231
- 例：Snowfall（大雪纷飞）、Cherry Blossom（落樱）、Butterflies（蝴蝶光斑）、Rain（下雨）、Confetti II（金片）
- しくみ：小さな絵を何十〜何百も出し、落とす・舞わせる・回す・ぼかす。
- 借りる先：仕組みは Motolii の物の copies・scatter・段の gravity・点群 Appear Rain（入っている）。つまみの多い見本は PlayCanvas ParticleEmitter（MIT、手元の写しには入っていない）。絵は Kenney Particle Pack（CC0）、Twemoji（CC BY 4.0、名前の表示が要る）。
- いま：○ 並べて落とす仕組みはある。絵が無い。ISF Star・Heart は 1 個だけ。
- 足すつまみ：CapCut は 速さ・背景の動き・大きさ くらい。Motolii は 数・出る場所（全体／縁／人の周り）・風・重さ・回転・ちらつき・奥行きのぼけ・色のばらつき・自分の絵に差し替え・拍に合わせる。
- 取れない：CapCut の粒の絵と重ねる動画。→ Kenney（CC0）・ThorVG の形。
- 当て方：中。

### 3. 光る・点滅 — 215
- 例：White Flash（闪白）、Black Flash（闪黑）、Strobe（蹦迪光）、Flicker（闪动）、Negative Strobe は 28 へ
- しくみ：数コマだけ白（黒・色）を足す・露出を上げる。くり返せばストロボ。
- 借りる先：ISF Fade Color・Strobe（手元）、gl-transitions fadecolor。抖音の 闪白 は CainCamera（Apache-2.0）。
- いま：◎ 段の exposure・opacity のキー、RGB Strobe。
- 足すつまみ：色・回数・間隔（拍に合わせる）・光る形（全面・半分・帯）・上がり方のイーズ。
- 当て方：中（「光る」は 6 の glow とまぎれる）。

### 4. 波・ゆがみ・渦 — 177
- 例：Ripples（水波纹）、Distortion（扭曲）、Vortex Setting（漩涡背景）、Squeeze（压缩）、Wave Left（水波向左）
- しくみ：画素を読む場所をずらす（サイン波・輪・渦・ノイズ）。
- 借りる先：ISF Ripples・Twirl・Bump Distortion・Noise Displace・Circle Splash（入っている）、Shockwave・Water Drop・Sine Warp、gl-transitions crosswarp・ripple・WaterDrop（手元）。文字の波は anime.js stagger（MIT）。
- いま：◎
- 足すつまみ：中心・半径・波の数・減り方・時間で広がる・ノイズの粗さ・かける場所（マスク）。
- 当て方：中。

### 5. グリッチ — 147
- 例：Fuzzy（毛刺）、Level Glitch（横纹故障）、Color Glitch（幻彩故障）、Jitter（放映机抖动）、故障
- しくみ：横の帯や四角をずらす＋色ずれ＋ノイズを、ときどき起こす。
- 借りる先：Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・datamosh（入っている）、VHS Glitch・Glitch Shifter・Stylize Glitch・v002 Glitch Analog（手元、vhs_sheet で効いた）、gl-transitions GlitchMemories・GlitchDisplace、three.js DigitalGlitch（手元）。
- いま：◎
- 足すつまみ：起こる頻度・帯の高さ・ずれの量・色ずれ・たね（シード）・拍に合わせる・かける場所。
- 当て方：高。

### 6. 光のにじみ（glow・bloom）— 139
- 例：Halo（光晕）、Soft（柔光）、Luminance（发光）、Ethereal（空灵）、金色辉光
- しくみ：明るい所を取り出し、ぼかして足す。
- 借りる先：段の bloom・物の glow・ISF Bloom・Glow（入っている）。three.js UnrealBloomPass（MIT、手元に無い）。
- いま：◎
- 足すつまみ：しきい値・半径・色・何段か・横に伸びる光・脈打つ。
- 当て方：中。

### 7. 帯・格子に割れる — 123
- 例：Horizontal Slice（横向分割）、Split（分割 I）、Blocks（黑色块）、Striped Glass（长虹玻璃）、不规则拼图
- しくみ：画面を帯・格子に切り、1 片ずつ時間差で動かす。
- 借りる先：Grid Flip（入っている）、Window Slice・Sliding Strips・Random Squares（手元）、gl-transitions windowslice・randomsquares・PuzzleRight・splitSlideIn。
- いま：○
- 足すつまみ：縦横の数・向き・時間差の順（前から・中から・ランダム）・すき間・1 片の動き（滑る・回る・裏返る）。
- 当て方：中〜低（27 本は組だけ）。

### 8. つなぎ目：押し出し・滑り — 110
- 例：Left（左移）、Up（上移）、Slide（滑动）、Stretch Left（向左拉伸）、Bottom Left II
- しくみ：前の絵と次の絵を、同じ向きに押し出す。
- 借りる先：gl-transitions directional・Slides・squeeze・directionalwarp（ISF Directional・Squeeze が手元）。
- いま：△ 手元の ISF を入れるだけ（Wipe はもう入っている）。
- 足すつまみ：角度（8 方向でなく何度でも）・イーズ・動きぼかし・ばね・すき間の色。CapCut のつなぎ目はつまみ 0（長さだけ）。
- 当て方：中。

### 9. 枠を重ねる — 107
- 例：Smartphone 1（手机）、Frame 1（录制框）、Insta（ins边框）、Photo Frame（画展边框）、宝丽莱 III
- しくみ：枠の絵を上に重ね、中の絵を少し縮める。
- 借りる先：枠は ThorVG の形（形の stroke・四角、入っている）。REC・日付の文字は ISF Tiny Date Time Overlay（手元）。
- いま：○
- 足すつまみ：枠の形・太さ・角丸・影・揺れ・中の絵の位置・文字。
- 取れない：CapCut の枠の絵。→ 形で描く。
- 当て方：中。

### 10. ぼかし・ピント — 106
- 例：Blur（模糊）、Motion Blur（动感模糊）、Camera Focus（变清晰）、Radial Blur（旋焦）、Vertical Blur
- しくみ：ぼかす（丸・方向・回転・ズーム）、ピントをキーで動かす。
- 借りる先：Fast Blur・Linear Blur・VVMotionBlur・Frosted Glass（入っている）、gl-transitions DefocusBlur・LinearBlur。
- いま：◎
- 足すつまみ：ぼかしの種類・中心・かける場所・ボケの形・明るい所を強く。
- 当て方：高。

### 11. 人の切り抜き＋ほかの効果 — 104
- 例：Invisible Person（隐形人）、Distorted Whirl（背景旋转）、Chroma Diffusion（色差扩散）、恶魔之翼、血迹
- しくみ：人を切り抜き、人だけ・背景だけにほかの族の効果をかける。
- 借りる先：MediaPipe Image Segmenter（Apache-2.0、手元に無い）。
- いま：✕ 切り抜きが無い。
- 足すつまみ：縁のやわらかさ・人と背景の入れ替え・何人目か。
- 取れない：翼・角・血の絵。→ CC0 の絵を探す。
- 当て方：低（多くは組だけ、35 本は名前で分からない）。

### 12. 古いフィルム・粒・ノイズ — 99
- 例：Retro Film（复古胶片）、Leak 1（胶片漏光）、Noise（噪点）、Develop（胶片显影）、Cinema（电影感）
- しくみ：粒＋傷とほこり＋ちらつき＋色あせ＋光漏れ＋がたつき。
- 借りる先：film・grain・fine_grain・sepia・technicolor（入っている）、v002 Light Leak・Film Burn（手元）。
- いま：◎
- 足すつまみ：粒の大きさ・色の粒・傷の数・ほこり・がたつき・光漏れの色と場所・送り穴の枠。
- 取れない：傷・ほこりの動画素材。→ ノイズで出す（Noise・Simplex Noise）。
- 当て方：高。

### 13. 紙が破れる・割れる・砕ける — 97
- 例：Paper Tear（纸质撕边）、玻璃破碎、屏幕裂纹、复古撕纸、Puzzle（碎块滑动）
- しくみ：ぎざぎざの縁で絵を分けて離す。または区切り（ボロノイ）ごとに飛ばす。
- 借りる先：見つからない。候補は Worley Cells（入っている）の区切り。自分で書かない約束なので、まず見本を探す。
- いま：✕
- 当て方：中。

### 14. 入り出：滑って入る・落ちる — 96
- 例：Slide Left（向左滑动）、Fall Bottom Left（向左下降）、Fly In、Drop down
- しくみ：位置のキー＋イーズ。
- 借りる先：animators slide_in・slide_out（入っている）、Animista（BSD-2）、Animate.css v3.7.2（MIT）。
- いま：◎
- 足すつまみ：角度・距離・イーズ（ways）・動きぼかし・1 字ずつ。
- 当て方：中（19 本は組だけ）。

### 15. 光漏れ・光の筋・スキャン・フレア — 92
- 例：Light Leaks、Light Beam（光束）、Spectrum Scan（光谱扫描）、Sun（晴天光线）、Sunset 4（丁达尔光线）
- しくみ：帯のグラデーションや光の筋を足し算で重ね、動かす。
- 借りる先：God Rays・Lens Flare（入っている）、v002 Light Leak・Linear Gradient（手元）、gl-transitions Overexposure。
- いま：○
- 足すつまみ：色・角度・幅・数・速さ・光源の位置・筋の長さ。
- 当て方：中。

### 16. VHS・古いテレビ・DV — 87
- 例：B&W VHS（黑白VHS）、Bad TV（老电视卡顿）、Retro Cam（复古录像带 III）、TV Off（电视关机）、复古 DV
- しくみ：走査線・にじみ・色ずれ・横の乱れ・ノイズ・画面の丸み。
- 借りる先：crt_finish・Bad TV（入っている）、VHS Glitch・v002 CRT Mask・CRT Displacement・TV Static（手元、効いた）。TV Off は AOSP ElectronBeam（Apache-2.0）。
- いま：◎
- 足すつまみ：テープの傷み・トラッキング・色のにじみ・走査線・REC と日付・4:3 の枠・画質を落とす。
- 当て方：高（21 本は組だけ）。

### 17. 絵・漫画・落書き — 87
- 例：B&W Sketch（黑白线描）、Dot Silkscreen（网点丝印）、Painting（油画纹理）、Splice（三格漫画）、街头涂鸦
- しくみ：縁を線に、色を減らす、網点、線をコマごとにゆらす。
- 借りる先：Edges・Emboss・Line Screen・CMYK Halftone・halftone・dot_screen（入っている）、Doodler・Sketch・Toon・Crosshatch（手元）、three.js ToonShader（手元）。
- いま：○
- 足すつまみ：線の太さ・色の段数・網点の大きさと角度・紙の色・ゆらぎ。
- 当て方：高。

### 18. 拭き取り・幕・扉・ブラインド — 82
- 例：Horizontal Open（开幕 I）、Vertical Close（闭幕 II）、Blinds（百叶窗 I）、Fade In（渐显开幕）、相机快门
- しくみ：境い目の線を動かし、片側を見せる。
- 借りる先：Wipe 4 方向（入っている）、Directional Wipe・Doorway・Window Blinds（手元）、gl-transitions HorizontalOpen・VerticalOpen。
- いま：◎
- 足すつまみ：角度・やわらかさ・境い目の線の色と太さ・中から開く・ぎざぎざ。
- 当て方：高。

### 19. 色を変える（白黒・色をのせる・暗く）— 82
- 例：Dark Night（暗夜）、Vignette（暗角）、局部上色、红色节拍、HDR
- しくみ：白黒・色をのせる・暗く・周辺を暗く・シャープ。
- 借りる先：Color Monochrome・Duotone・Trio Tone・LGG・Multi Hue Shift・vignette・Sharpen・Chroma Desaturation Mask（入っている）。
- いま：◎
- 足すつまみ：色・強さ・一部の色だけ残す・拍で切り替え。
- 当て方：中。

### 20. きらきら（光る所に十字の星）— 81
- 例：闪闪发光、kirakira、Gold Sparkles（金粉闪闪）、Diamond Halo、Mini Stars（细闪）
- しくみ：明るい点を見つけ、そこに十字の星を出して、またたかせる。
- 借りる先：明るい所の取り出しは Bloom（入っている）。星の形は ISF Star（入っている）・Kenney の星（CC0）。明るい点を何個か選ぶ所は見本が見つからない（探す）。
- いま：△
- 足すつまみ：筋の数（4・6・8）・長さ・回転・しきい値・数・またたく速さ・虹色。
- 取れない：Kirakira+ はよそのアプリ。CapCut の星の絵も取らない。
- 当て方：高。

### 21. 人の縁が光る・燃える・電気 — 76
- 例：Flame Outline（火焰描边 I）、Halo（光环 I）、Electro Border（电光描边）、Neon Doodle（霓虹涂鸦）、Aura
- しくみ：人を切り抜く→縁を取る→縁を光らせる・炎や電気を這わせる。
- 借りる先：MediaPipe（Apache-2.0）＋Edges・Glow（入っている）・ISF Neon（手元）。炎は 38、電気は 51 と同じ物。
- いま：✕ 切り抜きが無い。
- 足すつまみ：縁の太さ・色・流れる向き・内側も光る。
- 当て方：中。

### 22. 入り出：揺れる・振り子 — 68
- 例：Shake 1（轻微抖动）、Swing（钟摆）、Rock Horizontally（左右抖动）、Pendulum 2（荡秋千 II）
- しくみ：位置・回転にノイズや振り子を足す。
- 借りる先：ふるまい noise・oscillator、animators swing_3d（入っている）、Animista shake・vibrate。
- いま：◎
- 足すつまみ：振れ幅・速さ・減り方・軸の位置・拍に合わせる。
- 当て方：高。

### 23. フィルム風の色（フィルター）— 64
- 例：Lo-Fi、VHS I、Yellowed Memories、复古1988、Cool CCD
- しくみ：フィルムの色カーブ＋粒＋光のにじみ（halation）。
- 借りる先：film・technicolor・bleach_bypass・sepia・grain（入っている）＋1 の LUT の口。
- いま：○
- 取れない：CapCut の色の表。→ 自分で作る。
- 当て方：しくみは確か。色は名前から。

### 24. 残像・分身・灵魂出窍 — 62
- 例：Delay（迷离）、Trail（拖尾）、Fan Out（分身 II）、灵魂出窍、Ghosting
- しくみ：前のコマを薄く重ねる。または、拡大・ずらした写しを薄く重ねる。
- 借りる先：Echo Trace・RGB Trails 3.0（入っている）、Ghosting（手元）、three.js AfterimageShader（手元）、CainCamera の 灵魂出窍（Apache-2.0）。
- いま：◎
- 足すつまみ：何コマ前・写しの数・薄くなり方・写しの色・写しを拡大・拍に合わせる。
- 当て方：中。

### 25. RGB ずれ・色の分散・プリズム — 57
- 例：Chromatic Quirk（色差）、Chrome Blur（色差放射）、Chromo-zoom（色差放大）、RGB Border、彩虹棱镜
- しくみ：R・G・B を別々にずらす（横・放射・回転）。
- 借りる先：rgbshift（入っている）、Chromatic Aberration・Channel Slide・Chroma Zoom（手元、効いた）。
- いま：◎
- 足すつまみ：向き・量・横か放射か・ぼかし・拍で強く。
- 当て方：高。

### 26. つなぎ目：立方体・カードで裏返る — 56
- 例：Cube（立方体）、Flip、Flip & Zoom、卡片洗牌、空间翻转 I
- しくみ：2 枚の絵を立方体の面やカードに貼って回す。
- 借りる先：cube（入っている）、gl-transitions SimpleFlip・BookFlip・Revolve。本物の立体なら Motolii の solids（Box）。
- いま：○
- 足すつまみ：回る軸・遠近・床の映り込み・影・すき間。
- 当て方：高。

### 27. 画面：拍でズーム（zoom punch）— 55
- 例：Pulse（心跳）、Camera Beats、Mini Zoom（轻微放大）、快速变焦、推近推远
- しくみ：拍に合わせて一瞬拡大して戻る。ぼかし・色ずれを足すことも。
- 借りる先：キー scale・ISF Zoom・ふるまい oscillator（入っている）、CainCamera の 缩放。
- いま：◎
- 足すつまみ：拍（音から取る）・量・中心・戻り方（ばね）・ズームぼかし・色ずれを足す。
- 当て方：高。

### 28. ネガ・反転・熱カメラ — 52
- 例：Negative Strobe（负片频闪）、Color Negative（彩色负片）、Orange Negative、热成像
- しくみ：色を反転する。熱カメラは明るさを色の帯に置きかえる。
- 借りる先：Color Invert・RGB Invert・Thermal Camera・False Color（入っている）、Night Vision（手元）。
- いま：◎
- 当て方：高。

### 29. 花火・爆発・火花 — 51
- 例：Fireworks 2（花火）、Explosion（爆炸）、Spark（星火）、Collision Sparks
- しくみ：1 点から粒をはじけさせ、重さで落とす。
- 借りる先：レシピ burst.js・confetti.js・点群 Appear Eruption（入っている）、Kenney の火花（CC0）、PlayCanvas ParticleEmitter（MIT）。
- いま：○ 動きはある。絵が無い。
- 当て方：高。

### 30. 虹・色が回る — 51
- 例：Disco（蹦迪彩光）、Trippy（幻觉）、Rainbow Haze（彩虹幻影）、Color Swirl（彩色漩涡）
- しくみ：色相を時間で回す。虹のグラデーションを重ねる。
- 借りる先：Multi Hue Shift（入っている）、Color Phase（手元）。
- いま：◎
- 当て方：中。

### 31. ページめくり・折る — 51
- 例：Page Turning（翻页）、Folds（折痕）、Flipping（翻动）、Curling Wave（水波卷动）
- しくみ：紙を円筒に巻くようにめくる。折り目で折る。
- 借りる先：Inverted Page Curl（入っている、BSD-3、剪映の 翻页 と同じ式）、gl-transitions Fold・BookFlip。
- いま：◎（めくり）、○（折る）
- 足すつまみ：めくる角度・丸まりの半径・裏の色・影。
- 当て方：高。

### 32. 入り出：回って入る — 51
- 例：Rotate（旋转）、Spin Left（向左转入）、Roll Right（向右转入）、Angle 1（斜转）
- しくみ：回転のキー（拡大・移動を足すことも）。
- 借りる先：キー rotate（入っている）、Animista rotate-in・roll-in。
- いま：◎
- 当て方：高。

### 33. 画面：動かす — 49
- 例：Camera Dance、Move Track、Whiplash、倾斜抽卡
- しくみ：画面ごと動かす・伸ばす・カードのように飛ばす。
- 借りる先：キー x・y・scale_x・skew・2.5D（入っている）。
- いま：◎
- 当て方：低（全部組だけ）。

### 34. モザイク・ピクセル — 47
- 例：Mosaic（马赛克）、Pixel Scan、像素排序（ピクセルソート）、像素画
- しくみ：四角（六角・三角）ごとに同じ色にする。並べ替え（ピクセルソート）も。
- 借りる先：Pixellate・Mosaic・Multi-Pixellate・Hexagonalize・Triangle Mosaic（入っている）、Sorting Smear（手元）。
- いま：◎
- 当て方：高。

### 35. ネオンの縁取り（画面全体）— 44
- 例：Neon（荧光线描）、Edge Glow（边缘发光）、Neon Outline（边缘荧光）、White Border
- しくみ：縁を取り、色をつけて光らせ、暗い地に重ねる。
- 借りる先：Edges・Edge Blowout・Glow（入っている）、ISF Neon・three.js SobelOperatorShader（手元）。
- いま：○
- 足すつまみ：線の太さ・色（虹も）・地の暗さ・流れる光。
- 当て方：高。

### 36. 入り出：弾む・ばね — 44
- 例：Bounce In（弹入）、Bounce 1（回弹伸缩）、Pop Up（向上弹入）、Elastic Telescopic II
- しくみ：ばねのイーズで大きさ・位置を動かす。
- 借りる先：animators bounce_in・spring_in（入っている）。
- いま：◎
- 当て方：高。

### 37. つなぎ目：ズームで飛び込む — 43
- 例：Pull in（推近）、Pull Out（拉远）、冲击感、震感放大
- しくみ：前の絵に飛び込む（放射ぼかし＋拡大）。
- 借りる先：gl-transitions CrossZoom・SimpleZoom・zoomInOut（手元。剪映の 放射 は CrossZoom と同じ式）。
- いま：△ 入れるだけ。
- 当て方：高。

### 38. 炎・燃える — 42
- 例：Fire（火光）、Flame Frame（火焰边框 I）、Burn（燃烧 Ⅰ）、Color Flame
- しくみ：炎の絵を上へ流す／ノイズで炎を作る／燃えて消えるつなぎ目。
- 借りる先：Burn Away（入っている）、ISF Burn・Film Burn・Undulating Burn Out（手元）、Kenney の炎（CC0）。
- いま：○
- 当て方：高。

### 39. 文字：1 字ずつ — 42
- 例：Typewriter（打字机）、Random Typewriter、Flip Verbatim（逐字翻转）、Faulty text
- しくみ：文字を 1 字ずつ時間差で出す（打つ・裏返る・弾む）。
- 借りる先：animators print_on・print_off（入っている）、anime.js stagger（MIT）。
- いま：◎
- 足すつまみ：字・語・行ごと・順番（前・後・中・ランダム）・カーソル・音。
- 当て方：高。

### 40. 顔・目・頭 — 39
- 例：Big Head（大头）、Big Mouth（大嘴）、Flame Eyes（火焰眼 I）、Eye Reflection（眼神光）
- しくみ：顔の点（目・口）を取り、そこを大きく／飾りを付ける。
- 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋Bump Distortion（入っている）。
- いま：✕
- 取れない：猫耳・角などの飾りの絵。→ CC0 を探す。
- 当て方：高。

### 41. 入り出：3D で裏返る・傾く — 39
- 例：3D card 1（叠叠乐）、Flip 2（翻转 II）、Cube（立方体）
- 借りる先：rotate_x・rotate_y、animators swing_3d（入っている）、Animista flip-in・tilt-in。
- いま：◎
- 当て方：高。

### 42. 魚眼・レンズ・虫めがね — 38
- 例：Fisheye（鱼眼）、Wide Angle（广角）、Binoculars（望远镜）、Heart Magnifier
- しくみ：真ん中をふくらませる・へこませる。
- 借りる先：Bump Distortion・Sphere Map（入っている）、Fly Eye・Circle Warp（手元）、GPUImage Bulge（BSD-3）。
- いま：◎
- 当て方：高。

### 43. つなぎ目：回る・風車 — 38
- 例：Windmill（风车）、Axis Rotation（中轴旋转）、Rotate CW II、Turn Table
- 借りる先：Radial（入っている）、Rotate Scale Fade・Pinwheel・Swirl（手元。剪映の 风车 は Pinwheel と同じ式）。
- いま：△ 入れるだけ。
- 当て方：高。

### 44. 入り出：拡大・縮小・脈打つ — 37
- 例：Zoom In（放大）、Zoom Out（缩小）、Mini Zoom、Pulse、Pressing
- 借りる先：キー scale、ふるまい oscillator（入っている）、Animista scale-in・heartbeat。
- いま：◎
- 当て方：高。

### 45. 画面：揺れ（シェイク）— 36
- 例：Camera Shake（轻微抖动）、Shake 3（抖动）、Rebound Swing、Throb
- 借りる先：ふるまい noise を x・y に（入っている）、ISF Shake（手元）、CainCamera の 抖动。
- いま：◎
- 足すつまみ：x・y・回転の量を別々に・ノイズの粗さ・拍で強く・揺れの後のぼけ。
- 当て方：高。

### 46. 溶ける・にじんで替わる — 33
- 例：Dissolve（色彩溶解）、Mix（叠化）、融化、Ink Print（喷绘）
- しくみ：ノイズや明るさの順に入れ替える。液体のようににじむ。
- 借りる先：Fade（入っている）、Perlin Transition・Luma Transition・Luminance Melt（手元）、gl-transitions dissolve・crosswarp。
- いま：○
- 当て方：中。

### 47. 画面：3D の空間・カメラワーク — 32
- 例：3D运镜、360度运镜、3D Surround、丝滑运镜
- しくみ：画面を板にして 3D の空間で動かす。3D 写真は奥行きを推して人と背景を別に動かす。
- 借りる先：2.5D の rotate・z、Hyperspace（入っている）、solids。奥行きは Depth Anything V2 Small（Apache-2.0、要確認）。
- いま：○（3D 写真は ✕）
- 当て方：中。

### 48. 画面分割 — 29
- 例：Dual Screens（两屏）、Three Screens（三屏）、Four Screens（四屏）、Nine Screens（九屏）
- 借りる先：Quad Tile・Collage（入っている）、MultiFrame 2x2・3x3・Replicate（手元）、CainCamera の分割 7 種（Apache-2.0）。
- いま：◎
- 足すつまみ：縦横の数・すき間・1 画面ずつの時間差・鏡返し・色違い。
- 当て方：高。

### 49. 人の残像・分身 — 27
- 例：Dopelgangers（分身）、Phantom（幻影）、Light Trails（闪影）、Ghost（幽灵）
- 借りる先：MediaPipe ＋ Echo Trace（入っている）・Ghosting（手元）。
- いま：✕
- 当て方：高。

### 50. 形で開く（丸・ハート・時計）— 27
- 例：Circular Slices II（圆形分割 II）、圆形遮罩、圆形擦除
- 借りる先：Radial（入っている）、Circle Open・Heart Transition（手元。剪映の 爱心 は HeartWipe と同じ式）。
- いま：○
- 足すつまみ：形を自由に（ThorVG の形をマスクに）・中心・縁のやわらかさ。
- 当て方：中。

### 51. 稲妻・電気 — 26
- 例：Lightning、Thunderbolt、Electro Heart（电光爱心）、Rainbow Neon（彩虹电光）
- しくみ：折れ線を枝分かれさせて光らせる。
- 借りる先：three.js LightningStrike（MIT、手元に無い）＋Glow（入っている）。
- いま：✕
- 当て方：高。

### 52. 煙・霧・雲 — 24
- 例：Purple Mist（紫雾）、Black smoke（黑色烟雾）、Cloud（动漫云朵）、浓雾
- しくみ：ノイズの雲を流して重ねる。
- 借りる先：段の fog・Simplex Noise（入っている）、Kenney の煙（CC0）、Perlin（手元）。
- いま：○
- 当て方：高。

### 53. つなぎ目：揺れ・弾む — 24
- 例：Shake 3（抖动）、Jerky Camera、摇镜、IG 弹出
- 借りる先：ふるまい noise を 2 本に（入っている）、ISF Bounce（手元。剪映の 弹跳 は Bounce とほぼ同じ式）。
- いま：○
- 当て方：中。

### 54. コマ止め・コマ落とし・速さ — 23
- 例：Stuck Frame（卡机）、Flashing Frame（定格闪烁）、Step Printing、抽帧
- しくみ：あるコマで止める・コマを間引く・速さのカーブ。
- 借りる先：Freeze Frame・Random Freeze（手元）、datamosh（入っている）、gl-transitions StripDatamoshGlitch。
- いま：△
- 足すつまみ：止める長さ・間引く数・速さのカーブ・止めた絵に白い縁。
- 当て方：高。

### 55. 鏡・万華鏡 — 22
- 例：Mirror（镜像）、Tri-layer Mirror（镜像三格）、Switch（倒影）、Funhouse mirror（哈哈镜）
- 借りる先：mirror・kaleido・Kaleidoscope Tile（入っている）、Kaleidoscope・Mirror Edge（手元）。剪映の 倒影 は gl-transitions Swap と同じ式。
- いま：◎
- 当て方：高。

### 56. 人の周りに粒・飾り — 22
- 例：Butterfly Wings（蝴蝶翅膀）、Heart Background（爱心背景）、Bubbles 2、Lovestruck（热恋）
- 借りる先：MediaPipe ＋ 2 の粒。
- いま：✕
- 当て方：中。

### 57. 白黒（フィルター）— 19
- 例：BW 2、Gritty Noir、Anime B&W、报纸
- 借りる先：Color Monochrome（入っている）。
- いま：◎
- 足すつまみ：R・G・B の混ぜ方（色フィルター）・コントラスト・粒・セピアの色。
- 当て方：しくみは確か。

### 58. ホラー — 18
- 例：Spooky Camera（绿光监控）、Dripping Blood、Little Devil（小恶魔）、幽灵漂浮
- しくみ：暗く・赤く・揺れ・グリッチ・残像の組み合わせ＋血や悪魔の絵。
- 借りる先：ほかの族の物、Night Vision（手元）。
- いま：○
- 取れない：血・悪魔の絵。→ CC0 を探す。
- 当て方：中。

### 59. 画面：回る — 14
- 例：旋转镜头、DJ旋转、三维旋转
- 借りる先：キー rotate（入っている）。
- いま：◎
- 当て方：中。

### 60. カウントダウン・文字を重ねる — 14
- 例：倒计时、Karaoke（卡拉OK）、投影文字
- 借りる先：文字＋print_on、Digital Clock（入っている）、Tiny Date Time Overlay（手元）。
- いま：○
- 当て方：中。

### 61. 入り出：じわっと出る — 10
- 例：Fade In（渐显）、Fade Out（渐隐）、魔幻渐显
- 借りる先：animators fade_in・fade_out（入っている）。
- いま：◎
- 当て方：高。

### 名前だけでは分からない — 224
- 例：Passion、WOW、Oh My God、MV封面、一键三连
- 画面の効果 100・つなぎ目 83・入り出 30・文字 11。203 本が有料。
- 動く絵か説明を見てから、上の族に入れる。

## 取れない物と代わり

| 取れない物 | なぜ | 代わり |
|---|---|---|
| CapCut の効果ファイル（シェーダー・Lua・.ausl） | 規約で禁止（6 条 2 項・10 条）。中も見ない | 上の開いた物（MIT・BSD・Apache・CC0） |
| 粒の絵・重ねる動画（金粉・ハート・蝶・炎・星） | ByteDance の素材 | Kenney Particle Pack（CC0）、Twemoji（CC BY 4.0、表示が要る）、ThorVG の形 |
| 枠の絵（スマホ・送り穴・ポラロイド） | 同上 | ThorVG の形で描く |
| 色の表（LUT） | 同上 | Motolii の色の段で作り .cube で持つ |
| 顔・人の飾り（猫耳・角・翼・血） | 同上 | CC0 の絵を探す |
| 名前（Dune・Wong Kar-wai・Like Jennie など） | 人や映画の名前 | 自分で付ける |

使わない開いた物：Shadertoy の多く（非商用）、Lygia（商用は買い取り）、掘金の抖音まね（ライセンス無し）、chukcut（GPL-3）、qcut（ライセンス無し）、RawTherapee のフィルム LUT（ライセンスが分からない）、ISF Boxinator（CC BY-NC）。

## どうやって分けたか

- `capcut_map.md` の 1 行ずつから、名前・中国語名・「名前から」の言葉・組（光・動き…）を取り出した。
- 言葉に決まりを当てて族を決めた（例：色ずれ・RGB ずれ・プリズム → 25）。上から順に当て、最初に当たった族に入れた。
- 3,412 本は名前の言葉で入った。679 本は言葉が当たらず、CapCut の組（光・グリッチ…）だけで入れた（当て方：低）。フィルター 454 本は全部 LUT なので組で入れた。名前だけでは分からない物は 224 本。
- 中身を絵で見たのではない。「名前からの当て」なので、型を作る時に絵で確かめる。
- 「Motolii で今動くか」は `filament/effects/` の中身と、`capcut_map.md` の「いま」の欄から決めた。
