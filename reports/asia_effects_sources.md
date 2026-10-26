# アジアの効果の元を、言葉ではなく物で探す（10/26、途中）

## 考え方
- 記事の「おすすめ」も、メイキングの言葉も、本当のことを言わない（売る記事は売れる物を、作り手は「AE ですごいことをした」を言う）。
- なので、ごまかせない物を 2 つだけ見る。
  1. **何がかっこいいとされているか**：アジアのアプリ（抖音・剪映・CapCut・Alight Motion）の中で、実際に使われた回数。
  2. **それがどう作られているか**：流行った効果を作り直して公開したコード。
- アニメ系の MV やエディットの作り手が多いのは、中国（Bilibili・抖音）、日本（ボカロ MV）、東南アジア（Alight Motion と CapCut のエディット）、韓国。

## 安っぽい物は持ってこない（10/26、ユーザーの指示）

物差し：Tracery 2（Dragoy、aescripts、https://aescripts.com/tracery/ 、製品の説明は「FUI オーバーレイ」）の見本の絵が高級に見える理由を、絵から読んだもの。
- 色を絞る：黒と白に、蛍光の黄緑 1 色、細かい数字にだけマゼンタ。光らせない、キラキラさせない。
- 線は細く、文字は小さい：髪の毛ほどの円と線、「0.9724%」のような小さな数字。精密な計測の見た目で、機械がこの絵を読んでいると感じさせる。
- 絵の中身に沿う：円は関節や鼻の上、線はそれをつなぐ。かぶせる飾りではなく、映っている物の説明。
- 余白が多く、主役は絵のまま。
- 文字組みはポスターの作法（縦の細長い大きな文字、色の帯、小さな注記）。効果の文化ではなく、グラフィックデザインの文化。

安っぽい物はこの逆：色が多い、何でも光る、中身と関係ない飾り（ハート・蝶・キラキラ）、画面いっぱい、誰が使っても同じ。下の抖音の一覧の Bling・爱心・蝴蝶・撒金粉・彩虹光斑・羽毛・飘花瓣はこちら側なので持ってこない。高級さは効果の種類より使い方で決まる。欲しいのは「キラキラの粒子」ではなく、Plexus や Tracery のように絵の中身に沿って抑えて使える仕組み。

## 1. アプリが自分で並べた効果の一覧（抖音、2019 年ごろ）

AwemeLike（https://github.com/ZZZZou/AwemeLike 、MIT と書いてある）の `AwemeLike/Effect/effect.bundle` に、抖音のアプリの効果の包（`GESticker_SkinNeedling` のような抖音の中の名前、ByteDance の効果エンジン amaz のファイル）がそのまま入っている。**中身は ByteDance の物なので借りない**（アプリから取り出した物は使わない決まり）。名前だけを「抖音が並べた効果の一覧」として使う。

| 種類 | 名前（抖音の表示名） | Motolii の近い物（未確認） |
|---|---|---|
| 滤镜 39 | 70s、Bling、X-Signal、下雨、下雪、人鱼滤镜、大雨、幻觉、彩虹光斑、抖动、摇摆、撒金粉、斑驳、星光、毛刺、波纹、流光、灵魂出窍、炫彩、爱心bling、爱心光斑、爱心泡泡、窗格、粒子、线性、缩放、羽毛、老电视、花火、蒸汽波、蝴蝶、轻颤、迷幻烟雾、迷离、闪屏、闪白、霓虹、飘花瓣、黑白电影 | 老电视・70s → ISF の Bad TV、CRT ディスプレイ風。幻觉 → ISF の RGB Trails。闪白・闪屏 → ISF の Strobe。霓虹 → ISF の Neon。波纹 → ISF の Ripples。灵魂出窍 → 写し（大きさと薄さを変える）の組み合わせ |
| 转场 13 | 倒计时、光斑模糊变清晰、卷动、圆环、开场、旋转、模糊变清晰、横滑、横线、电视关机、电视开机、竖线、缩放转场 | 一部は ISF の Wipe・Zoom |
| 分屏 7 | 三屏、两屏、九屏、六屏、四屏、模糊分屏、黑白三屏 | ISF の Quad Tile・Collage |

分かること：抖音の「面白い」側の効果（Bling、撒金粉、星光、爱心、下雨・下雪、花火、蝴蝶、羽毛、飘花瓣、彩虹光斑、迷幻烟雾）は、ほとんどが粒子か光の粒を重ねる物で、上の物差しでは安っぽい側。粒子の仕組みが要るとしても、目的はこれらではなく、中身に沿って抑えて使う物（Plexus・Tracery の側）。

## 2. 作り直しのコード

中国の開発者は、流行った抖音の効果を OpenGL で作り直し、掘金（juejin.cn）や CSDN に解説を書き、GitHub に置く。

### ライセンスがあり、借りられる物
| リポジトリ | ライセンス | 中身 |
|---|---|---|
| JYangkai/MediaDemo（星 127、解説は掘金の「mirai」） | MIT | 抖音の传送带（ベルトコンベア、縦・横）、蓝线挑战（青い線が通った所から絵が止まる：`other/move_line_*`・`retain_frame_*`）、2・3・4・9 分割、ピクチャインピクチャ、ゆがみ、モーションブラー、転場 30 種（渦 vortex、めくり page_up、押す・引く、反転、ワイプ 9 方向、カット 4 種） |
| yishuinanfeng/UnitedPlayer（星 141） | Apache-2.0 | 仿抖音の滤镜 4 種（`record/src/main/res/raw/fragment_shader1〜3.glsl`、中身は未読） |
| numberwolf/FFmpeg-PlusPlus（星 388） | MIT | FFmpeg に GLSL の滤镜・転場を入れる仕組み。転場 6 種（上下左右へ・回転） |
| wysaid/android-gpuimage-plus（星 1,934） | MIT | 画像・カメラ・動画の滤镜の土台（流行りの効果の一覧ではない） |

### ライセンスが無い物（作り方の出典にだけ使う）
| リポジトリ | 中身 |
|---|---|
| githubhaohao/OpenGLCamera2（星 1,089） | 滤镜と抖音特效 30 種以上 |
| qw9685/OpenGL-imageFilter | 分割、モザイク（四角・六角・三角・円）、缩放、闪白、抖动、灵魂出窍、毛刺、幻觉ほか |
| kikisTUTU/OpenGLESSixFilterDemo | 抖音の定番 6 種：缩放・灵魂出窍・抖动・闪白・毛刺・幻觉 |
| wangchao0837/OpenGlCameraRender | 灵魂出窍、快慢録画、美顔 |

## 3. 使われた回数がどこで取れるか
| どこ | 取れる数 | 状態 |
|---|---|---|
| CapCut のテンプレート（https://www.capcut.com/templates） | 各テンプレートに数（例：3.3M、1.1M）。ページには何の数かが書かれていない | 読めた。何の数か、テンプレートがどの効果を使っているかは未確認 |
| Alight Motion のプリセット | XML か QR で配られる。場所は YouTube・TikTok・r/AlightMotion・Discord。インドネシア語の「mentahan」（素材）が探す言葉 | 数はまだ取っていない |
| 抖音・剪映の効果 | アプリの中に「何万人が使った」が出る | アプリの中を見る必要がある（取り出しはしない） |
| Bilibili | 剪映・AE の教程の再生数・いいね・コイン | まだ見ていない |

## 次
1. CapCut のテンプレートの数が何の数かを確かめ、上位のテンプレートが使う効果を並べる。
2. 抖音の一覧の 1 つずつに、Motolii の近い物が本当に近いかを絵で確かめる。
3. MediaDemo（MIT）の GLSL を、手を入れずに Motolii で動かせるか（Motolii が読めるのは ISF・WGSL・three.js の効果。生の GLSL ES の断片を読む口は無い）。

## Tracery 級の追跡オーバーレイの出典（10/26、ゴール 2）
- **blob-tracey**（Batkixni、MIT、https://github.com/Batkixni/blob-tracey 、TypeScript 約 3,900 行、外の部品はアイコンだけ）：AE Blob Tracker を真似た Web の道具。枠（四角・角かっこ・十字・輪郭・凸包・六角）、中心の印、座標のラベル（16 進・百分率ほか）、点のつなぎ方（ドロネー・最小全域木・近い K 個・星形・全部）、枠の中だけの画素の崩し、ちらつきの抑え、設定の JSON。
- 見つける所（BlobDetector）も描く所（OverlayRenderer）もブラウザの Canvas（`document.createElement('canvas')`・`getImageData`）に頼る。手を入れずに動かすには本物のブラウザが要る。
- **リアルタイムの元：basementstudio/shader-lab**（Apache-2.0、https://github.com/basementstudio/shader-lab 、星 713）。GPU で絵を 128×72 の格子（R＝前のコマとの差、G＝明るさ、B＝薄れていく動きの量）にして読み戻し、CPU の `src/lib/blob-tracking/tracker.ts`（518 行、ブラウザの部品を使わない）が塗りつぶしで塊を出し、前のコマと距離で結んで番号を保つ。10/26、tracker.ts を esbuild で型だけ外し（368 行、中身はそのまま）QuickJS-ng 0.17 で回した：14 秒の見本の動画 430 コマ、1 コマ平均 2.0 ミリ秒（最大 3）、番号 19 本、動く猫に枠が付く。格子は試しに ffmpeg（tblend の差・lagfun）で作った代わりの物で、大きい枠が猫より少し上にずれる。原因は確かめていない。
- Shader Lab の効果は約 50 本（21,896 行）で、どれも three.js の WebGPU と TSL。DOM は使わないので、JS から WebGPU を呼べれば手を入れずに動くはず。その口の既存の物：**mystralengine/mystralnative**（MIT、星 415、v0.1.6）。ブラウザ無しで JS に WebGPU（Dawn）・Canvas 2D（Skia）・音・fetch を出す。QuickJS も選べ、Mac・Windows がある。three.js の WebGPU と PixiJS v8 の手引きがあり、C++ に組み込む口（`include/mystral/runtime.h`、`getWGPUDevice`）もある。まだ動かしていない。
- p5.js を動かすのと同じ道（AviUtl2Browser_H の約束）。手本は oov の aviutl_browser（MIT、中身は CEF）。次はそのソースを読み、要る物を出す。

## 高級側の見本の置き場：Codrops（10/26）
- Codrops（tympanus.net）の見本は「特に書いていなければ MIT」（https://tympanus.net/codrops/licensing/ ）。「再配布・販売はだめ」はデザインの素材の方の決まりで、見本のコードには当たらない（前にこれを取り違えていた）。GitHub に 349 個。
- どれもブラウザで動く（DOM・CSS・SVG・WebGL）。ブラウザ表示の道ができれば、blob-tracey・p5.js と同じく手を入れずに動く。
- 物差しに合いそうな物（文字の動きが多い。MV の文字に直結する）：
  | 見本 | 星 | ライセンス表記 | 中身 |
  |---|---|---|---|
  | OnScrollTypographyAnimations | 349 | MIT | 文字の動きの案の集まり |
  | ScrollBlurTypography | 149 | MIT | ぼけた文字が現れる |
  | KineticTypePageTransition | 127 | MIT | キネティックタイポの場面転換 |
  | TypographyMotion | 85 | MIT | Thibaud Allie の文字のずらし（の作り直し） |
  | RepetitiveTypography | 43 | MIT | Domagoj Štrok のキネティックタイプ（の作り直し） |
  | TextClipScroll | 73 | MIT | SVG の切り抜きで文字 |
  | LetterEffects | 207 | 表記なし（サイトの決まりで MIT） | anime.js の文字の動き |
  | TextDistortionEffects | 100 | 表記なし（同） | Blotter.js の文字の歪み |
  | GlitchPerspective・GlitchyGrid | 51・41 | MIT | 抑えたグリッチ |
  | CrosshairDistortion | 47 | MIT | 十字の照準の歪み |
  | RainEffect | 1,785 | 表記なし（同） | 窓の雨粒（WebGL） |
  | LiquidDistortion | 474 | 表記なし（同） | 液体の歪み（PixiJS） |
  | WebGLBlobs | 94 | MIT | 球の変形と色（three.js） |
- 10/26 絵を撮った（`reports/codrops_sheet.jpg`）。tympanus.net は画面に出さないブラウザを 403 で弾く（ロボット避け。回り込まない）ので、GitHub のソース（MIT）を手元に取り、この Mac の中で開いて撮った。13 個のうち 12 個が組み立て無しで開けた（RainEffect だけ組み立てが要る）。
  - 物差しに合う：KineticTypePageTransition、GlitchyGrid、TypographyMotion、ScrollBlurTypography、TextClipScroll、OnScrollTypographyAnimations、RepetitiveTypography、TextDistortionEffects（雑誌やポスターの文字組み、色を絞る）。GlitchPerspective は強いがデザインされている。
  - 止め絵では分からない：CrosshairDistortion（マウスで動く）、WebGLBlobs（点だけ）。
  - 気を付ける点：ほとんどがスクロールやマウスで動き、時間では動かない。動画にするには時間に合わせてスクロールを進める必要がある（timesnap の `preparePageForScreenshot` で 1 コマごとにページを動かせる）。見本のページには Codrops の見出しや広告（hotjar、HIRED など）が入るので、効果の部分だけを使う手間がある。

## 映画の仕上げ系：three.js と pmndrs/postprocessing（10/26）
- three.js の効果（`examples/jsm/shaders`、MIT）は、Motolii がもう 52 本中 30 本をそのまま読める（ASSETS.md の three.js の行）。
- **pmndrs/postprocessing**（https://github.com/pmndrs/postprocessing 、Zlib、星 2,899）：効果 30 種余り（ASCII、Bloom、Bokeh、BrightnessContrast、ChromaticAberration、ColorAverage、ColorDepth、DepthOfField、DotScreen、Glitch、GodRays、Grid、HueSaturation、LUT1D・LUT3D、LensDistortion、Noise、Outline、Pixelation、RealisticBokeh、Scanline、Sepia、ShockWave、TiltShift、ToneMapping、Vignette ほか）。書き方は `mainImage(inputColor, uv, out outputColor)` の形で、three.js の効果ファイル（uniforms と fragmentShader）とは違う。今の Motolii の読み口（`filament/three_isf.cpp`）ではそのまま読めない。
- 物差しで見ると、この系統は「薄く掛けた時だけ高級」：粒子状のノイズ、周辺減光、ごく弱い色収差、ティルトシフト、走査線、グリッド、ASCII、色の段数を落とす（ColorDepth）。強く掛けると安っぽい側に落ちる。
- 絵はまだ撮っていない。
