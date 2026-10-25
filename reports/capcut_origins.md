# CapCut の効果の「本当の元ネタ」（見た目と動きで探した、10/25）

## 式で確かめた 7 本：剪映・CapCut の初期の切り替えは gl-transitions そのもの（10/26）

qcut（ライセンス無し、事実だけ使う）の `research/jianying-runtime-probe/TWENTY-TRANSITIONS.md` が 剪映の切り替え 33 本の式と数を書き出している。それを gl-transitions の元のファイルと 1 行ずつ比べた（research_notes/capcut/origins_code.md）。

| 剪映 | gl-transitions（ライセンス） | どれだけ同じか |
|---|---|---|
| 爱心 | HeartWipe（MIT） | 同じ |
| 窗格 | WindowSlices（MIT） | 同じ |
| 弹跳 | Bounce（MIT） | 影の高さだけ半分（0.0375、元は 0.075） |
| 倒影 | Swap（MIT） | 同じ |
| 放射 | CrossZoom（MIT） | 同じ |
| 风车 | Pinwheel（MIT） | 同じ |
| 翻页 | InvertedPageCurl（BSD-3） | 同じ |

- 7 本とも 2019 年（剪映の 1 年目）に作られた（素材の番号の頭 32 ビットが作った時刻：6 本が 10/15〜16、放射が 8/12）。CapCut も同じ素材（翻页・倒影は md5 まで同じ）。
- 2022 年より後の切り替えは gl-transitions ではない（AE 風のキーを Lua で動かす AETools、Lumi、連番の絵）。
- ほかの CapCut の解析：montage、OpenFrame（GPL-3）、rea-capcut（MIT）、TrimCut、jianying-headless。名前と番号の一覧：pyJianYingDraft（Apache-2.0、切り替え 453）、pyCapCut（1,137）。
- 借りない物：trinity・AwemeLike（抖音の素材そのものが入っているらしい）、effect-lab（ByteDance のモデルと素材を配っている）。ByteDance は 2023 年に 10 件以上を DMCA で消させている。
- 調べた言葉：中国語・日本語・韓国語・ロシア語・ベトナム語・インドネシア語・ポルトガル語・スペイン語・タイ語・トルコ語・アラビア語（research_notes/capcut/origins_*.md）。「CapCut がどこから写したか」を書いた人はどの言葉にもいない。抖音の「6 つの定番」の再現記事は 2018 年の 1 本から広がった物。
- ByteDance 自身の数え：CapCut のテンプレート約 3 万 5 千本でよく使われる切り替えの上位 30（AutoTransition、ECCV 2022、arxiv 2207.13479）。風車・キューブ・つぶし・ブラインド・時計回し・万華鏡は gl-transitions にある。
- 借りられる新しい出所：CainCamera（Apache-2.0、抖音の 灵魂出窍・幻觉・抖动・缩放・闪白 と画面分割 7 種の GLSL、README にだけライセンス）。

## 手元の CapCut の中を見た（10/26）

中身は写していない。暗号は解いていない。読んだのは、手元の CapCut が落とした効果のファイルにある「出典の書き込み」（URL・作者・ライセンス）だけ。

- 場所：`~/Movies/CapCut/User Data/Cache/effect`（効果 905 個、シェーダーと Lua 12,640 本、普通の文字）。
- 出典の URL が残っていた物は 16 本だけ（0.1%）：
  - Patrick Hebron の Sobel のエッジの gist（ライセンス書かれず）：「ヤンデレ」「ブラー2」「サンフランシスコ」「カラフルなキラキラ」ほか、計 10 本。
  - ISF の共有サイトの「Bounding Box」（editor.isf.video、作者・ライセンス書かれず）：粒子ノイズ（Grain）の部品。
  - ashima / stegu の webgl-noise（MIT）：粒のステッカー。
  - OpenCV の説明と StackOverflow（Photoshop のハイパス）：考え方の出所。
- ほかに残らない理由：CapCut の GLSL は、ByteDance 自身のシェーダー言語（.ausl）から機械で作った物（変数名が `_f0` など、コメントは消える）。出典の書き込みは元の .ausl にしか無く、それは配られていない。
- 部品の名前は After Effects の効果名そのまま（`AETools`、`AdjustGlow`、`Grain`、`GaussianBlur`、`StrongSharpen`）。作り手は AE の効果をまねて組んでいる。
- アプリ本体に入っている外の物：Qt 6（QtBodymovin＝Lottie を含む）、Chromium（CEF）、FFmpeg（libavfilter ほか）、dav1d、LAME、OpenFX（smart_cut、BSD-3）、Sparkle。効果は `libEffectPlatform` が配り、`libvecryptor` という暗号の部品もある。

### CapCut を解析している公開プロジェクト
- **chukcut**（GPL-3、Linux の動画編集アプリ）：`docs/research/effect-package-format.md` が CapCut の効果の入れ物の形を 1,474 行で書いている（逆コンパイルせず、キャッシュのファイルを読んだだけ）。`capcut-stack.md`・`effect-runtime.md` も。GPL なのでコードは借りない。分かった事実だけ使える。
- **qcut**（ライセンス無し）：CapCut の美顔などの効果を調べている。ライセンスが無いので借りない。
- **pyCapCut・CapCutAPI**：下書きのファイル（draft）の形。
- **marcosp34bpm/capcut-rea**：Windows 版の中の部品の一覧。

## 疲れている時はここだけ

1. よく使われる 100 本のうち、元ネタが見つかった（確か・たぶん）のは **30 本**。似ているだけが 59 本、見つからないのが 11 本。「ByteDance が写した」と書いた公開の資料は **0**。
2. 多かった元ネタ：① **Adobe の Premiere・AE に最初から入っている物**（Strobe Light・Dip to White・Clock Wipe など、8 本） ② **2014 年の glsl-transitions（いまの gl-transitions）**（9 本。Cube・Squeeze・Windmill は見た目がいちばん近い） ③ **先に出ていた携帯アプリ**（8mm Vintage Camera・VHS Camcorder・Kirakira+・Instagram・Picsart・Facebook の 3D 写真、7 本）。次が Video Copilot と YouTuber のつなぎ目パック（6 本）。
3. 元ネタをそのまま借りられるのは 30 本のうち **12 本**（gl-transitions 9・Android の ElectronBeam 2・KinoGlitch 1）。残りは有料の物、他社のアプリ、非商用の物。
4. 代わりなら 100 本のうち **97 本**を、手元の ISF-Files・gl-transitions・MediaPipe など借りられる物で作れる。作れないのは 3 つ（Film Frame の枠の絵、水墨の墨の映像、フィルムの色の表）。
5. 注意：**見た目を絵・動画・記事で確かめたのは 31 本だけ**。「※」の付いた 69 本は、名前とつまみから推した見た目。Shadertoy の元は見つからなかった（VHS だけ候補が出た）。

## 調べ方

- 100 本の選び方：CapCut の公式の人気順の表は公開されていない。なので、教程・紹介記事・抖音の定番 6 つ（缩放・灵魂出窍・抖动・闪白・毛刺・幻觉）・`reports/capcut_map.md` の名前の表から、よく出てくる物を選んだ。内わけは画面の効果 61、人の効果 6、つなぎ目 23、クリップの動き 3、文字の動き 3、フィルター 4。
- 見た目は、公開されているページ・教程の動画のサムネイル・記事から取った。CapCut のアプリや素材は落としていないし、開いてもいない。
- 元ネタとして書いたのは、目立つ特徴が同じで、CapCut（2020-04）・剪映（2019-05）・抖音（2016-09）より前の物だけ。
- 確かさ：**確か**＝同じ動きで、昔から決まった型の物 ／ **たぶん**＝目立つ特徴がいくつも同じで、元のほうが先 ／ **似ているだけ**＝種類が同じというだけ。
- 借りられるか：○＝MIT・BSD・Apache・CC0・CC BY（商用に使える）、✕＝有料・他社のアプリ・非商用（CC BY-NC）。「手元」＝`research_notes/isf/ISF-Files/ISF` の ISF のファイル（MIT）。

## 分かった事

- **公開の「写した」証拠は無い。** 中国の作り直しの記事（掘金 2019 https://juejin.cn/post/6844903814214991886 ）も、抖音の効果そのものを元としていて、AE や Shadertoy の名前は出していない。B站・知乎で「これは AE の何々と同じ」と元を名指しした作り直しも見つからなかった。
- **中国の動画の SDK には共通の効果の一式があるらしい。** 腾讯云の動画 SDK にも「灵魂出窍」（SOUL_OUT）がある https://www-sg.tencentcloud.com/zh/document/product/1069/38029 。どちらが先かは分からない。
- **阿里云のつなぎ目 63 本は gl-transitions の物で、中にある名前（SubType）が gl-transitions のファイル名と同じ**（风车＝pinwheel、立方体＝cube、百叶窗＝windowblinds、燃烧＝burn、故障＝glitchmemories、圆形扫描＝radial…） https://help.aliyun.com/zh/ims/developer-reference/normal-transition-effect-example 。その中国語の名前のうち 18 個は CapCut のつなぎ目にもある（风车・立方体・百叶窗・燃烧・故障・圆形遮罩・圆形扫描・万花筒・水滴・色彩溶解・爱心遮罩・开幕・回忆・渐变擦除・4 方向の擦除）。重なるのは名前だけで、中身が同じかは分からない。ただ、中国の業界で gl-transitions が共通の土台になっているのは確か。
- **見た目がいちばん近いのは gl-transitions のつなぎ目。** Cube（黒い地に、床への映り込みまで同じ。Keynote の Cube には映り込みが無い）、Squeeze、Windmill。
- **抖音の定番 6 つ**：抖动は Video Copilot Twitch の「拍で拡大＋色ずらし」、闪白は Adobe の Strobe Light、毛刺は KinoGlitch の Analog に近い。**灵魂出窍と幻觉は、前からある物が見つからなかった**（抖音が自分で作った可能性がある）。
- **人の効果**（縁の炎・流れる光・稲妻）：AE と Saber で前から作られていた見た目を、人の切り抜きで自動にした物に見える。ただ、名指しした資料は無い。
- **「速さ・背景の動き」だけがつまみの粒の効果**（金粉・星空・ハート・蝶・雪・雨）は、ByteDance が作った重ねる用の動画の素材らしい。どこかの shader を写した物ではなさそう。
- 調べきれなかった所：Shadertoy は 403 で開けなかった（ID はほかのサイトの出典一覧から取った物だけ）。B站・小紅書の中身はほとんど引けなかった。

## 表（100 本）

※＝見た目は名前とつまみから推した物（絵では確かめていない）。

### 画面の効果：抖音の定番・グリッチ・レトロ

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 1 | Shake 3（抖动） | 赤と青を逆にずらし、拍で少し拡大 | Video Copilot Twitch（AE の有料の道具、2009 年ごろ。拍で拡大＋色ずらし） https://www.videocopilot.net/products/twitch/features 。コードの近い物：three.js RGBShiftShader https://raw.githubusercontent.com/mrdoob/three.js/r100/examples/js/shaders/RGBShiftShader.js | たぶん | Twitch ✕。three.js は MIT で ○。手元：Channel Slide＋Zoom |
| 2 | 灵魂出窍 | 半透明の拡大した写しが重なり、大きくなりながら消える | 見つからない（腾讯云の SDK にも同じ物。どちらが先かは不明） | 見つからない | 手元：Zoom の結果を元の絵に重ねて薄くする ○ |
| 3 | Trippy（幻觉） | 絵が円を描いて動き、赤い残像が尾を引く | 見つからない（色ごとの残像という種類が同じなだけ） | 似ているだけ | 手元：RGB Trails 3.0・Time Glitch RGB ○ |
| 4 | White Flash（闪白） | 白が一瞬かぶさって光る | Premiere・AE の Strobe Light（Premiere 6.0 の説明書、2001） https://discmaster.textfiles.com/file/30729/Digital%20Video%20Collection%20CD.iso/Premiere%206.0/ARCHIVE.Z/c09af80.htm?html=true | たぶん | Adobe ✕。手元：Strobe（白）、gl-transitions fadecolor（MIT）○ |
| 5 | Black Flash（闪黑） | 一瞬黒くなる※ | 同じく Strobe Light（色を黒に） | 似ているだけ | 手元：Strobe（黒）○ |
| 6 | Fuzzy（毛刺） | 横の行がばらばらにずれる＋うすい色ずれ | keijiro の KinoGlitch の Analog Glitch（2015） https://github.com/keijiro/KinoGlitch | たぶん | MIT ○。手元：Glitch Shifter |
| 7 | Pulse（心跳） | 拍に合わせて拡大と縮小をくり返す | Twitch の Scale（1 と同じ） | 似ているだけ | 手元：Zoom ○ |
| 8 | Glitch（故障） | 帯や四角が飛ぶ＋色ずれ＋砂嵐※ | three.js DigitalGlitch https://raw.githubusercontent.com/mrdoob/three.js/r100/examples/js/shaders/DigitalGlitch.js 、KinoGlitch の Digital（2015） | 似ているだけ | MIT ○。手元：Glitch Displace・Glitch Memories |
| 9 | Chromatic Quirk（色差） | 色の版が横にずれる※ | three.js RGBShiftShader（1 と同じ） | 似ているだけ | MIT ○。手元：Channel Slide |
| 10 | Chrome Blur（色差放射） | 真ん中から放射状に色ずれ＋ぼかし※ | 見つからない（レンズの色ずれという種類だけ） | 似ているだけ | 手元：Chromatic Aberration・Chroma Zoom ○ |
| 11 | RGB Glitch（彩色故障） | RGB が別々にずれて四角が崩れる※ | Glitché アプリ（2015 の紹介） https://lifeinlofi.com/2015/12/09/review-glitche-a-modern-photo-app-that-delightfully-destroys-photos/ 、three.js DigitalGlitch | 似ているだけ | Glitché ✕。手元：Time Glitch RGB ○ |
| 12 | Level Glitch（横纹故障） | 横の帯が切れて左右にずれる※ | KinoGlitch の Analog（2015、6 と同じ） | 似ているだけ | MIT ○。手元：Glitch Shifter |
| 13 | VHS（蓝白VHS） | 青白いビデオテープ（乱れ・走査線・ノイズ）※ | Shadertoy の VHS 物（ryk「VCR distortion」ldjGzV など、年は未確認。ReShade の出典一覧で確認 https://www.reshade.me/forum/shader-presentation/1258-vhs-shader ）、unity-vhsglitch https://github.com/staffantan/unity-vhsglitch | 似ているだけ | Shadertoy ✕（非商用）。unity-vhsglitch は CC BY 3.0 で ○（名前を書く）。手元の VHS Glitch はこれが元なので、同じく名前を書く |
| 14 | Retro Cam（复古录像带 III） | REC と日付の文字＋テープの乱れ※ | Rarevision の VHS Camcorder アプリ（2015-08） https://nofilmschool.com/2015/08/vhs-camcorder-rarevision-ios-iphone-ipad-app | たぶん | アプリ ✕。手元：VHS Glitch＋Tiny Date Time Overlay |
| 15 | Bad TV（老电视卡顿） | 古いテレビの縦流れ・波打ち・ノイズ※ | Felix Turner の Bad TV Shader（年は未確認） https://github.com/felixturner/bad-tv-shader | 似ているだけ | MIT ○。手元：Bad TV |
| 16 | TV Off（电视关机） | 絵が縦につぶれて白い横線 1 本になり、消える※ | Android の画面を消す時の動き（2010） https://www.droid-life.com/2010/10/21/video-gingerbread-shows-a-screen-off-animation/ 。コード：AOSP の ElectronBeam.java（2012） https://android.googlesource.com/platform/frameworks/base/+/refs/tags/android-4.2_r1/services/java/com/android/server/power/ElectronBeam.java | たぶん | Apache-2.0 ○ |
| 17 | TV On（电视开机） | 16 の逆。線から広がって絵が出る※ | 同じ ElectronBeam の点ける時の動き | たぶん | Apache-2.0 ○ |
| 18 | Old TV 2（老电影 II） | 色あせ・傷とほこり・ちらつき・枠のゆれ※ | 8mm Vintage Camera アプリ（2010） https://apps.apple.com/app/id406541444 | たぶん | アプリ ✕。手元：Sepia Tone＋Noise＋Shake |
| 19 | Retro Film（复古胶片） | 暖かい色・粒・光漏れのフィルム風※ | 8mm Vintage Camera（2010、18 と同じ） | 似ているだけ | ✕。手元：v002 Technicolor・Sepia Tone＋Noise |
| 20 | Film Frame（胶片框） | 送り穴のあるフィルムの枠で囲む※ | 見つからない | 見つからない | 代わり無し（枠の絵を置くしかない） |
| 21 | Leak 1（胶片漏光） | 端から暖かい色の光が流れ込む※ | 8mm Vintage Camera の Light Leak（2010） | 似ているだけ | ✕。手元：Film Burn（gl-transitions FilmBurn、MIT）○ |
| 22 | Jitter（放映机抖动） | 映写機のコマが縦にがたつく※ | 8mm Vintage Camera の Jitter ボタン（2010、「映写機のコマ揺れ」と説明） | たぶん | ✕。手元：Shake ○ |
| 23 | Rolling Film（滚动胶片） | コマが縦に流れ、コマの境の黒い線が見える※ | Rampant の Frame Rolling の素材（年は未確認） https://www.provideocoalition.com/ask-rampant-frame-rolling-in-adobe-after-effects-and-premiere-pro/ | 似ているだけ | 売り物 ✕。手元：HorizVertHold ○ |
| 24 | Strobe（频闪） | 決まった間隔で白く点滅※ | Strobe Light（2001、4 と同じ） | たぶん | ✕。手元：Strobe ○ |
| 25 | Negative Strobe（负片频闪） | 決まった間隔でネガ（色の反転）に点滅※ | Strobe Light（説明書に「色が反転する」例がある、2001） | たぶん | ✕。手元：Strobe（反転）・Color Invert ○ |

### 画面の効果：光・粒・レンズ

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 26 | Edge Glow（边缘发光） | 縁を取り出し、色をつけて光らせて重ねる | Boris FX Sapphire の S_GlowEdges https://borisfx.com/documentation/sapphire/ae/glowedges （AE の Find Edges＋Glow でも作れる） | 似ているだけ | 有料 ✕。手元：Edges＋Glow ○ |
| 27 | Neon（荧光线描） | 暗い地に、縁だけが蛍光の線で光る※ | 26 と同じ | 似ているだけ | 手元：Neon ○、three.js SobelOperatorShader（MIT）○ |
| 28 | Neon Outline（边缘荧光） | 輪郭が蛍光色で縁取られる | 26 と同じ | 似ているだけ | 手元：Neon＋Edges ○ |
| 29 | Halo（光晕） | 明るい所がにじんで光り、動く※ | 見つからない（光のにじみ全般） | 似ているだけ | 手元：Bloom・Glow、three.js UnrealBloomPass（MIT）○ |
| 30 | Starlight（星光放射） | 明るい所から星の光の筋が出る※ | Trapcode Starglow（2007 には v1.5） https://awn.com/news/red-giant-software-releases-new-versions-trapcode-shine-starglow-3d-stroke 、Apple Motion の Dazzle | 似ているだけ | 有料 ✕。手元：God Rays＋Star ○ |
| 31 | 闪闪发光 | 光る所・反射する所に、十字の星がきらめく | Kirakira+（2015 に公開、2017 年秋に大流行） https://techcrunch.com/2017/12/17/kirakira-vs-the-world/ ・ https://apps.apple.com/app/id955687901 。CapCut には「kirakira」という名前の効果まである https://www.flexclip.com/learn/sparkle-effect.html | たぶん | 他社のアプリ ✕。手元：Star を明るい所に散らす、Kenney の星（CC0）○ |
| 32 | Gold Dust（金粉） | 金の粒が舞う※ | Trapcode Particular などの粒の素材（種類が同じだけ） | 似ているだけ | ✕。Kenney Particle Pack（CC0）○ |
| 33 | Starry（星空） | 星がまたたく※ | 見つからない | 似ているだけ | 手元：Star ○ |
| 34 | Pink Hearts（荧光爱心） | 光るピンクのハートが舞う※ | 見つからない | 見つからない | 手元：Heart ○ |
| 35 | Lightning（闪电） | 稲妻が走って光る※ | AE に最初から入っている Advanced Lightning https://origin-www.lynda.com/After-Effects-tutorials/Electrifying-graphics-Advanced-Lightning/439417/514582-4.html | 似ているだけ | ✕。three.js LightningStrike（MIT）○ |
| 36 | Snowfall（大雪纷飞） | 雪が落ちる※ | AE の CC Snowfall | 似ているだけ | ✕。three.js の点の雪（MIT）○ |
| 37 | Rain（下雨） | 雨が降る（線か水滴かは未確認） https://www.capcut.com/it-it/explore/Rain-video-effect | AE の CC Rainfall | 似ているだけ | ✕。Kenney（CC0）を落とす ○ |
| 38 | Butterflies（蝴蝶光斑） | 蝶の形の光のボケが舞う※ | 見つからない | 見つからない | Twemoji の蝶（CC BY 4.0）＋手元の Bloom ○ |
| 39 | Spectrum Scan（光谱扫描） | 虹色の光の帯が画面を走る※ | 見つからない | 見つからない | 手元：Linear Gradient＋Multi Hue Shift ○ |
| 40 | Fisheye（鱼眼） | 真ん中がふくらむ※ | GPUImage の BulgeDistortion（2012） https://github.com/BradLarson/GPUImage | 似ているだけ | BSD-3 ○。手元：Bump Distortion |
| 41 | Mirror（镜像） | 左右か上下を鏡写しにする※ | three.js MirrorShader、ISF Mirror | 似ているだけ | MIT ○ |
| 42 | Tri-layer Mirror（镜像三格） | 3 本の帯に同じ絵、互い違いに鏡返し※ | 見つからない | 見つからない | 手元：Replicate＋Mirror ○ |
| 43 | Three Screens（三屏） | 同じ絵を 3 つ並べる※ | 見つからない（よくある分割） | 似ているだけ | 手元：Replicate・Quad Tile ○ |
| 44 | Mosaic（马赛克） | 四角いモザイク※ | GPUImage の Pixellate（2012）、gl-transitions Mosaic | 似ているだけ | ○。手元：Pixellate・Mosaic |
| 45 | Shockwave（冲击波） | 輪が広がり、そこだけ絵が曲がって見える※ | Shadertoy の shockwave の物（多くは非商用）、Godot の CC0 の物 https://godotshaders.com/shader/shock-wave-shader/ | 似ているだけ | Godot の物は CC0 で ○。手元：Shockwave ○ |
| 46 | Ripples（水波纹） | 輪の波紋で絵がゆれる※ | gl-transitions ripple | 似ているだけ | MIT ○。手元：Ripples |
| 47 | Whirlpool（漩涡） | 真ん中を軸に渦にねじれる※ | GPUImage Swirl、gl-transitions Swirl | 似ているだけ | ○。手元：Swirl・Twirl |
| 48 | Heart Magnifier（模糊爱心放大镜） | 全体はぼけて、ハートの所だけくっきり拡大※ | 見つからない | 見つからない | GPUImage Bulge＋手元の Heart＋Fast Blur ○ |
| 49 | Dot Silkscreen（网点丝印） | 網点の印刷風※ | Photoshop の Color Halftone、three.js HalftoneShader | 似ているだけ | three.js MIT ○。手元：CMYK Halftone・Dot Screen |
| 50 | Splice（三格漫画） | 画面を 3 コマの漫画の枠に割る※ | 見つからない | 見つからない | 手元：Toon＋3 つに分ける ○ |

### 画面の効果：ぼかし・残像・動き

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 51 | Blur（模糊） | 全体がぼける | 決まった元は無い（Photoshop・AE の基本） | 似ているだけ | 手元：Fast Blur ○ |
| 52 | Motion Blur（动感模糊） | 一つの向きにこすったようにぼける※ | Photoshop の Motion Blur（中国語版の名前が「动感模糊」で同じ。見た目は普通） https://helpx.adobe.com/vn_vi/photoshop/how-to/add-motion-blur-effects.html | 似ているだけ | ✕。手元：Diagonal Blur・VVMotionBlur 3.0 ○ |
| 53 | Open（开幕） | 上下の黒い帯が開いて絵が出る※ | 決まった元は無い（映画の黒帯・左右に開く扉の型）。gl-transitions HorizontalOpen は 2023 年なので後の物 | 似ているだけ | gl-transitions HorizontalOpen・VerticalOpen（MIT）○ |
| 54 | Blurry Focus（模糊开幕） | ぼけた所からピントが合う※ | 決まった元は無い。gl-transitions LinearBlur | 似ているだけ | 手元：Linear Blur ○ |
| 55 | Dopelgangers（分身） | 人を切り抜いて写しを何体も並べる※ | 見つからない | 見つからない | MediaPipe（Apache-2.0）＋手元の Ghosting ○ |
| 56 | Rainbow Haze（彩虹幻影） | 動きに虹色の残像がつく※ | AE の Echo＋色相ずらしの作り方（2017） https://nofilmschool.com/2017/09/how-create-isolated-echo-motion-trail-effect-after-effects | 似ているだけ | ✕。手元：RGB Trails 3.0＋Multi Hue Shift ○ |
| 57 | Zoom Lens（镜头变焦） | 拡大・縮小する | 決まった元は無い | 似ているだけ | 手元：Zoom ○ |
| 58 | 3D运镜（3D Zoom） | 写真の人は寄り、背景は引いて、奥行きのある動き | Facebook の 3D 写真（2018-10） https://thenextweb.com/facebook/2018/10/11/facebook-rolls-out-3d-photos/ 、3D Ken Burns の論文（2019-09） https://arxiv.org/abs/1909.05483 。CapCut 版の流行 https://www.dexerto.com/entertainment/how-to-get-the-3d-photo-effect-on-tiktok-1580724/ | たぶん | ✕（3D Ken Burns のコードは非商用）。代わり：MiDaS（MIT）か Depth-Anything の小さい型（Apache-2.0）で奥行き＋MediaPipe ○ |
| 59 | Camera Shake（轻微抖动） | 小刻みにゆれる | AE の wiggle・揺れのプリセット | 似ているだけ | 手元：Shake ○ |
| 60 | Color Negative（彩色负片） | 色を反転する※ | 決まった元は無い | 似ているだけ | 手元：Color Invert ○ |
| 61 | Clock wipe（圆形扫描） | 時計の針のように回って拭き取る※ | Premiere の Clock Wipe https://filmimpact.com/clock-wipe-transition-premiere-pro 、gl-transitions Radial（2014）・angular（2017） | たぶん | gl-transitions MIT ○。手元：Radial |

### 人の効果

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 62 | Flame Outline（火焰描边） | 人の縁に沿って炎が燃える※ | Video Copilot Saber（2016-03、無料だが閉じた物）の縁取り https://www.provideocoalition.com/saber-new-free-effects-plug-video-copilot/ | 似ているだけ | ✕。MediaPipe＋手元の Edges＋Glow ○ |
| 63 | Streamer Stroke（流光描边） | 人の縁を光の筋が流れて回る※ | Saber の縁取りを流して動かす作り方 https://blog.motionarray.com/learn/post-production/video-copilots-free-saber-plug-in-review/ | たぶん | ✕。MediaPipe＋手元の Edges＋Linear Gradient ○ |
| 64 | Light Trails（闪影） | 人の写しが光り、ちらつきながら残る※ | AE の Echo＋Glow（56 と同じ作り方） | 似ているだけ | ✕。MediaPipe＋手元の Strobe・Ghosting・Comet Tails ○ |
| 65 | Lightning Bound（闪电环绕） | 人の周りを稲妻が回る※ | AE の作り方「electric shock effect」（Tuts+、2013-07） https://photography.tutsplus.com/tutorials/how-to-make-an-awesome-electric-shock-effect--ae-26175 | たぶん | ✕。MediaPipe＋three.js LightningStrike（MIT）○ |
| 66 | Neon Doodle（霓虹涂鸦） | 人の周りに手描きのネオンの線※ | Picsart の手描きのネオン縁取りの流行（2016-12） https://picsart.com/blog/light-up-your-shots-with-neon-outlines/ | たぶん | ✕。MediaPipe＋手元の Doodler・Neon ○ |
| 67 | Halo（光环） | 頭の上に光る輪※ | Snapchat の天使の輪のレンズ（年は未確認） https://support.lensstudio.snapchat.com/hc/en-us/community/posts/360018335023-Glow-around-a-face-effect/comments/360000845203 | 似ているだけ | ✕。MediaPipe＋手元の Circle＋Glow ○ |

### つなぎ目

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 68 | Pull in（推近） | 放射状にぼかしながら強く寄って、次の絵へ | Sam Kolder 風のズームのつなぎ（2017-02） https://www.4kshooters.net/2017/02/18/creating-the-appealing-sam-kolders-smooth-zoom-in-and-zoom-out-transition-effects-in-premiere-pro-cc 、gl-transitions CrossZoom（2014） | たぶん | CrossZoom は MIT で ○。手元：CrossZoom |
| 69 | Pull Out（拉远） | 寄った所から引いて、次の絵へ | Sam Kolder 風（68 と同じ記事） | たぶん | 手元：CrossZoom を逆に回す ○ |
| 70 | Mix（叠化） | 溶け合って入れ替わる | 映画で昔からあるクロスディゾルブ。gl-transitions fade | 確か（昔からの型） | MIT ○。手元：Fade |
| 71 | White Flash（闪白・つなぎ目） | 白く飛んでから次の絵へ | Premiere の Dip to White。gl-transitions fadecolor | 確か（昔からの型） | MIT ○。手元：Fade Color（白） |
| 72 | Black Fade（闪黑） | 黒を通って次の絵へ | Premiere の Dip to Black。gl-transitions fadecolor | 確か（昔からの型） | MIT ○。手元：Fade Color |
| 73 | Blur（模糊・つなぎ目） | ぼかしながら入れ替わる※ | gl-transitions LinearBlur（「ぼかしを通るクロスフェード」、2018 年まで） | たぶん | MIT ○。手元：Linear Blur |
| 74 | Shake 3（抖动・つなぎ目） | ゆれながら次の絵へ※ | Videohive の Premiere 用つなぎ目パック（2018 年ごろ、Shake・Camera Shake） https://videohive.net/item/handy-transitions-for-premiere-pro/22125468 | たぶん | 売り物 ✕。手元：Shake＋Fade ○ |
| 75 | Zoom Shake 2 | 寄りながらゆれて次の絵へ※ | 74 と同じパックの Zoom Shake（2018 年ごろ） | たぶん | ✕。手元：CrossZoom＋Shake ○ |
| 76 | Windmill（风车） | 真ん中から細い扇が回りながら開き、次の絵が見える | glsl-transitions の pinwheel（2014-05）→ gl-transitions https://raw.githubusercontent.com/gl-transitions/gl-transitions/master/transitions/pinwheel.glsl 。もっと前の型：SMIL 2.0 の pinWheelWipe（2001） https://www.w3.org/TR/2001/WD-smil20-20010301/smil-transitions.html 、PowerPoint の Wheel。CapCut の見た目 https://www.youtube.com/watch?v=XKbA5YhJgeM | **確か**（式と数が gl-transitions の Pinwheel と同じ、羽 8 枚。下の「式で確かめた 7 本」） | MIT ○。手元：Pinwheel |
| 77 | Cube（立方体） | 黒い地で立方体が回り、床に映り込む | glsl-transitions の cube（2014-06） https://raw.githubusercontent.com/gl-transitions/gl-transitions/master/transitions/cube.glsl 。CapCut の見た目 https://www.youtube.com/watch?v=IYO9_lu_gF4 | たぶん（床の映り込みまで同じ） | MIT ○。手元：cube |
| 78 | Page Turning（翻页） | 右から紙が筒のようにめくれ、裏は白っぽい | PowerPoint 2013 の Page Curl（2012） https://thepowerpointblog.com/new-page-curl-transition/ 、glsl-transitions の PageCurl（2014）。CapCut の見た目 https://www.youtube.com/watch?v=kNN1AK3-uJ0 | **確か**（式と数が gl-transitions の InvertedPageCurl と同じ） | 手元：Inverted Page Curl（BSD-3）○ |
| 79 | Blinds（百叶窗） | 横の帯ごとに次の絵が開く | PowerPoint の Blinds https://learn.microsoft.com/pt-br/previous-versions/office/developer/office-2010/cc795536(v=office.14) 、gl-transitions windowblinds（2017）。CapCut の見た目 https://www.youtube.com/watch?v=6WgcZLnZWWM | 似ているだけ | 手元：Window Slice（90° 回す）○ |
| 80 | Mosaic（马赛克・つなぎ目） | 大きな四角の点になってから次の絵へ※ | glsl-transitions の pixelize（2014） | 似ているだけ | MIT ○。手元：Pixelize |
| 81 | Whirlpool（漩涡・つなぎ目） | 強く渦にねじれたまま次の絵へ | glsl-transitions の Swirl（2014）。CapCut の見た目 https://www.youtube.com/watch?v=FUTxMCdHjU0 | 似ているだけ | MIT ○。手元：Swirl |
| 82 | Squeeze（压缩） | 前の絵が上下からつぶれて横 1 本の帯になり、外側に次の絵 | glsl-transitions の squeeze（2014-05） https://raw.githubusercontent.com/gl-transitions/gl-transitions/master/transitions/squeeze.glsl 。CapCut の見た目 https://www.youtube.com/watch?v=kPoA7sZcf1U | たぶん | MIT ○。手元：Squeeze |
| 83 | Stretch（拉伸） | 横に引きのばされ、放射状にぶれて次の絵へ | glsl-transitions の CrossZoom（2014、近いだけ）。CapCut の見た目 https://www.youtube.com/watch?v=d3UPV-v5Abg | 似ているだけ | 手元：CrossZoom ○ |
| 84 | 甩鞭转场・Swipe Left | 速く横に流れて、強い横ぶれで次の絵へ※ | 映画のホイップパン。glsl-transitions の Slide＋LinearBlur | 似ているだけ | 手元：Directional＋Linear Blur ○ |
| 85 | Burn（燃烧 Ⅰ） | 橙から黄に焼けて白く飛び、次の絵へ | glsl-transitions の burn（2014、色は橙） https://raw.githubusercontent.com/gl-transitions/gl-transitions/master/transitions/burn.glsl 、gl-transitions FilmBurn（2016）。CapCut の見た目 https://www.youtube.com/watch?v=SiT-wfoKBzI | 似ているだけ | MIT ○。手元：Burn・Film Burn |
| 86 | 水墨 | 墨が水に広がる形で次の絵が出る※ | 見つからない（墨の映像を明るさの型に使う作り方は前からある） | 見つからない | 手元の Luma Transition（MIT）に CC0 の墨の映像を入れる。映像はまだ無い |
| 87 | 圆形遮罩 | 円が開いて次の絵が見える※ | 映画で昔からあるアイリス、glsl-transitions の circleopen（2014） | 似ているだけ | MIT ○。手元：Circle Open・Circle Crop |
| 88 | Flip | 板が裏返るように回る（ぶれつき） | Keynote・PowerPoint の Flip、glsl-transitions の SimpleFlip（2014）。CapCut の見た目 https://www.youtube.com/watch?v=x1wylzELu0s | 似ているだけ | 手元：Soft Flip ○ |
| 89 | Glitch（故障・つなぎ目） | 横の帯がずれ、RGB が割れて次の絵へ | glsl-transitions の Glitch Memories（2014）・glitch displace（2015）。CapCut の見た目 https://www.youtube.com/watch?v=TqhE9AsiZgg | 似ているだけ | MIT ○。手元：Glitch Memories・Glitch Displace |
| 90 | Light Leaks（漏光・つなぎ目） | 暖かい色の光漏れが画面をなでて、次の絵へ | 光漏れの映像の素材集（Rampant など）、gl-transitions FilmBurn（2016）。CapCut の見た目 https://www.youtube.com/watch?v=cme7VUxhmnA | 似ているだけ | 素材集 ✕。手元：Film Burn ○ |

### クリップの動き・文字の動き

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 91 | Zoom 1（缩放・入りから出まで） | 拡大と縮小をくり返す※ | Animate.css の pulse・zoomIn https://github.com/animate-css/animate.css | 似ているだけ | v3 は MIT で ○ |
| 92 | Rotate（旋转・入り） | 回りながら出てくる※ | Animate.css の rotateIn | 似ているだけ | v3 ○ |
| 93 | Swing（入り） | 振り子のようにゆれて止まる※ | Animate.css の swing、Animista の swing-in | 似ているだけ | v3 ○、Animista（BSD-2）○ |
| 94 | Type 1（打字机） | 1 文字ずつ出る | AE の Typewriter プリセット（2004 年ごろ） | 似ているだけ | ✕。Motolii の print_on で足りる |
| 95 | Bounce In・Spring（弹簧） | 行き過ぎて戻る、ばねの動き※ | Animate.css の bounceIn | 似ているだけ | v3 ○ |
| 96 | Wave in | 文字が順に波のように持ち上がって出る※ | 見つからない（1 文字ずつずらすのは AE の Text Animator から） | 見つからない | Motolii の文字の動きで作る |

### フィルター

| No | CapCut の効果 | 見た目 | 元ネタ（URL・年） | 確かさ | 借りられるか |
|---|---|---|---|---|---|
| 97 | 柯达500T | 夜の青み、光の周りに赤いにじみ※ | 本物のフィルム Kodak Vision3 500T と、それを売り直した CineStill 800T（2012 年ごろから流行） https://en.wikipedia.org/wiki/Cinestill | たぶん（フィルムとして） | 見た目そのものは自由。開いた色の表は無い。手元：White Point Adjust＋Bloom で寄せる |
| 98 | 柯达gold200・柯达ektar100・富士胶片 | Gold は暖かい黄、Ektar は色が濃い、富士は緑より※ | 本物のフィルム。開いた再現：Pat David の HaldCLUT（2015） https://patdavid.net/2015/03/film-emulation-in-rawtherapee/ | たぶん（フィルムとして） | HaldCLUT はファイルのライセンスが書いていない → 確かめるまで ✕。手元：Vibrance・LGG |
| 99 | Wong Kar-wai | 緑と青の影に、赤と黄の強い色、強いコントラスト※ | 映画『恋する惑星』『天使の涙』（1994〜95、撮影クリストファー・ドイル） https://colorculture.org/cinematography-analysis-of-chungking-express/ | 似ているだけ | 開いた色の表は無い。手元：Duotone・LGG |
| 100 | Lo-Fi・Flash CCD | Lo-Fi は色が濃く影が深い。CCD はフラッシュの白とびとにじみ※ | Instagram の Lo-fi（2010） https://www.cultofmac.com/557639/instagram-filters-popular/ 、古いデジカメ（CCD）の流行 https://www.gearpatrol.com/tech/a44476827/vintage-digital-camera/ | たぶん（Lo-Fi） | Instagram ✕。手元：Vibrance＋v002 Vignette、CCD は Exposure Adjust＋Bloom＋Noise |

## 数のまとめ

| 確かさ | 本数 |
|---|---|
| 確か | 3（Mix・White Flash・Black Fade。どれも昔からある編集の型） |
| たぶん | 27 |
| 似ているだけ | 59 |
| 見つからない | 11 |

| 元ネタの出どころ（確か・たぶんの 30 本、重なりあり） | 本数 | 借りられるか |
|---|---|---|
| gl-transitions（2014 年の glsl-transitions から） | 9 | ○ MIT |
| Adobe の Premiere・AE に最初から入っている物 | 8 | ✕ |
| 先に出ていた携帯アプリ（8mm Vintage Camera・VHS Camcorder・Kirakira+・Picsart・Instagram・Facebook） | 7 | ✕ |
| Video Copilot（Twitch・Saber）と YouTuber のつなぎ目パック | 6 | ✕ |
| Android の ElectronBeam | 2 | ○ Apache-2.0 |
| 本物のフィルム | 2 | 見た目は自由、色の表は無い |
| KinoGlitch | 1 | ○ MIT |

## 気をつける事

- 手元の v002 の ISF（Glitch Analog・Light Leak・Technicolor・Vignette）は、元の v002 のライセンスをまだ確かめていない。
- 手元の VHS Glitch は unity-vhsglitch（CC BY 3.0）が元。使うなら作者の名前を書く。
- 「※」の 69 本は、次に絵で確かめる時に元ネタが変わる事がある。とくに人の効果と粒の効果。
