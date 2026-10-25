# CapCut の効果を Motolii に：元ネタと借りる先（10/25）

CapCut の効果の一覧を「作る物の見本（仕様）」にして、1 本ずつ、Motolii に今ある物と、借りられる開いた物（ライセンスつき）を並べた。CapCut の効果ファイルそのものは ByteDance の物で、取れない（下の「規約」）。

前の調べ：`research_notes/capcut/en.md`・`ja_ko.md`・`zh.md`（10/23、誰が作るか・仕組み・規約・抖音っぽい shader の記事）。ここではその上に、効果の名前の全部の表と、名前の突き合わせと、公開された部品表を足した。

## 元ネタ（どこから来たか）

### 答え

- **CapCut の効果は、開いた shader 集の移植ではなく、ByteDance の社内の素材の店（BytePlus CreativeStore／火山引擎の资源商城）の物。** 外に売っている同じ会社の編集 SDK（BytePlus Video Editor SDK）の素材一覧の名前が、CapCut の名前とそのまま重なる（下の 3）。BytePlus 自身が「VE SDK は CapCut・抖音・TikTok と同じ社内チーム（Intelligent Creation）の物」と書いている。
- **CapCut の兄弟の BytePlus VE SDK の公開された部品表に、効果の元になる開いた物（GPUImage・gl-transitions・Shadertoy・OpenCV など）は 1 つも無い。** 載っているのは土台だけ：ffmpeg（3.x を手直し、LGPL-2.1）、Lua 5.1（MIT、効果の台本）、bullet 2.8.9（物理）、libtess2（形を三角形に、Motolii も同じ物を使う）、Clipper2（形の足し引き）、Eigen、HarfBuzz・FreeType・ICU（文字）、libyuv、Lottie（Android、Apache-2.0）、PocketSVG。効果そのものは「EffectSDK」という閉じた塊で、素材はアプリに入れずにネットから配る、と書いてある。
- **CapCut 本体のオープンソース告知の一覧は、公開の場所では見つからなかった。** アプリの中（設定 → 情報）にはあるはずだが、アプリを落として開くのは今回やらない約束。代わりに上の兄弟 SDK の部品表と、他の人が公開した Windows 版の中身の調べ（Qt 6・CEF・Lynx・FFmpeg（avcodec-61）・SVT-AV1・libjxl・dav1d、効果は VECreator.dll・lens.dll・cccreator.dll という自社の塊）を使った。
- **名前が開いた見本と同じになる物はある（gl-transitions の Swirl・Cube・Mosaic・Burn・Film Burn・Kaleidoscope・Squeeze、文字の動きの Slide In Left・Zoom In・Bounce In など）。** ただしどれもありふれた言葉で、中身が同じ証拠ではない。「元ネタ」として借りられるのは、同じ種類の開いた物、という意味になる。

### 1. CapCut 自身のオープンソース告知を探した

| 探した所 | 分かった事 |
|---|---|
| capcut.com の規約・素材の使用許諾・ヘルプ、「CapCut open source licenses」「剪映 开源软件声明」「com.lemon.lvoverseas open source」で検索 | 公開の場所に告知の一覧は無かった。アプリの中にしか無いと見られる（未確認） |
| BytePlus Video Editor SDK の Android の手引き（同じ社内チームの外売り版） | 部品表あり（下の表） |
| BytePlus Video Editor SDK の iOS の手引き | 部品表あり（下の表、ライセンスの欄は無い） |
| BytePlus Effects の「第三者ソフトの告知」 | 33 本。ほとんどが見本アプリの部品（AFNetworking・Masonry・SDWebImage・YYText・expo・Docker…）。効果に関わる物は 0 |
| 他の人が公開した CapCut 9.6（Windows）の中身の調べ（marcosp34bpm/capcut-rea） | Qt 6・CEF・Lynx（ByteDance の画面の仕組み）・FFmpeg・SVT-AV1・libjxl・dav1d。効果は VECreator.dll（271 MB）・lens.dll・cccreator.dll・videoeditor.dll の自社の塊 |

部品表のうち、絵・動画・効果に関わる物（BytePlus VE SDK、Android と iOS）：

| 部品 | 版 | ライセンス | 何の役 | Motolii |
|---|---|---|---|---|
| ffmpeg | 3.x を手直し | LGPL-2.1 | 動画の読み書き | GStreamer（同じ役）。ffmpeg はデータモッシュで使う |
| Lua ＋ lua_cjson | 5.1.5 ／ 2.1.0 | MIT | 効果の台本（効果の中で動く小さな言語） | QuickJS（同じ役：レシピ・anime.js・three.js の効果を動かす） |
| bullet | 2.8.9 | zlib | 物理 | Jolt（同じ役） |
| libtess2 | 1.0.2 | SGI Free B | 形を三角形に | **同じ物を使っている** |
| Clipper2 | 1.0.4 | BSL-1.0 | 形の足し引き | ThorVG の中 |
| Eigen | 3.3.3 | MPL-2.0 | 行列の計算 | Filament の math |
| HarfBuzz ・ FreeType ・ ICU | 2.2.0 ・ 2.10.0 ・ 72.1 | MIT ・ FTL ・ ICU | 文字を並べて描く | ThorVG の文字 |
| libyuv | 1816 | BSD-3 | 動画の色の形の変換 | GStreamer |
| stb_image ・ libpng ・ giflib | 2.22 ・ 1.6.37 ・ 5.1.4 | MIT ・ libpng ・ MIT | 絵のファイル | — |
| Lottie（Android） | 5.2.0 | Apache-2.0 | AE の動く絵 | ThorVG の Lottie |
| PocketSVG（iOS） | 2.7.3 | MIT | SVG の形 | ThorVG |
| RGBConverter | — | MIT | 色の形の変換 | — |
| EffectSDK | — | 閉じた物 | **効果そのもの** | — |

分かる事：効果の中身は「Lua の台本＋自社の描く係＋配る素材」。開いた shader 集から持ってきた跡は部品表に無い。

### 2. 開いた見本の名前と突き合わせた

CapCut の名前のうち英語の名前が分かる 1088 本を、見本の名前と同じか比べた（大文字小文字・空白・後ろの番号は無視。入りは「Slide Left」→「slideInLeft」のように in を足して比べた。Fade・Blur・Zoom・Flip・Rotate・Swing・Flash・Shake・Mix・Slide・Spin だけの名前は、ありふれすぎるので数えない）。中国語だけの名前（約 3,400 本）は名前が比べられないので、「同じ種類」の数に入る。

| 見本 | ライセンス | 名前が同じ（本） | 例 | 同じ種類で借りる先になった行 |
|---|---|---|---|---|
| gl-transitions | MIT（2 本 BSD） | 27 | Dissolve、Wipe Up、Wipe Right、Wipe Left、Zoom Out 2＝zoomInOut、Horizontal Close、Wipe Down、Squeeze | 1018 |
| ISF-Files | MIT（1 本 CC BY-NC、数本 Shadertoy 由来） | 29 | Wipe Up、Wipe Right、Wipe Left、Wipe Down、Squeeze、Burn、Cube、Strobe | 2809 |
| GPUImage | BSD-3 | 5 | Mosaic、Swirl、Motion Blur、Luminance、Vignette | 53 |
| three.js shaders | MIT | 7 | Focus、Horizontal Blur、Vertical Blur、Vertical Blur II、Vignette、Mirror、Film 2 | 64 |
| Animate.css | v4 は Hippocratic 2.1、v3.7.2 は MIT | 26 | Slide Up＝slideInUp、Slide Right＝slideInRight、Slide Left＝slideInLeft、Zoom In、Fade In、Slide Down＝slideInDown、Bounce In、Zoom Out 2 | — |
| Animista | FreeBSD（BSD-2） | 8 | Slide Right＝slide-in-right、Slide Left＝slide-in-left、Fade In、Flicker、Slide Right＝slide-out-right、Slide Left＝slide-out-left、Fade Out、Blink | 473 |
| CSSgram（Instagram の名前） | MIT | 1 | Lo-Fi | — |
| Lygia | Prosperity（商用は 30 日だけ、要買い取り） | 調べず | 関数の集まりで、効果の名前の一覧ではない | 使わない |
| Shadertoy | 書かなければ CC BY-NC-SA 3.0 | 調べず | 題名を全部引く口が無い。題名が同じでも多くは商用不可 | 使わない |
| VSCO・Instagram の色の名前 | — | 1（Lo-Fi） | CapCut のフィルター名は映画（Dune・Poor Things・Wong Kar-wai・オッペンハイマー）、場所（西雅图＝シアトル・布达佩斯）、フィルムの銘柄（柯达 500T・ektar100・gold200・普罗维亚 100・爱克发 400・哈苏）、人（Like Jennie・Kendall）の名前。VSCO の型番（A6・C1…）は 0 | — |
| BytePlus VE SDK の素材一覧（同じ会社） | 閉じた物 | 11 組 | Vintage DV＝复古 DV、Fisheye II＝鱼眼 II、Square prism＝方形棱镜、Tree shadow＝Tree Shade（树影）、Flash white＝White Flash（闪白）、Push closer＝Pull in（推近）、Zoom out＝Pull Out（拉远）、Up right＝向右上、Seattle＝西雅图、Film、Doll | 借りられない（同じ会社の売り物） |
| Alight Motion（参考、閉じた物） | — | 21 | Dissolve、Flicker、Lightning、Squeeze、Blink、Cube、Dissolve II、Dissolve III | — |

読み方：名前が一番重なるのは **gl-transitions（つなぎ目）と Animate.css／Animista（入り・出）** で、どちらも「その分野の決まった言葉」。CapCut が一番そっくり重なるのは、開いた物ではなく **同じ会社の BytePlus の素材一覧**。

### 3. 公開されている証拠

- BytePlus：VE SDK は ByteDance の Intelligent Creation チームの物で、TikTok・抖音・CapCut・西瓜视频・FaceU・Ulike が使う（en.md の問1、BytePlus の概要ページ）。
- BytePlus VE SDK の素材一覧：フィルター 10・効果 10・つなぎ目 5 が入っていて、もっと欲しければ「CreativeStore で買う」。名前は CapCut と同じ（上の表）。
- 火山引擎（中国の同じ事業）：2021-07 に「抖音と同じ効果を 1 万以上」企業に開いた、効果づくりの道具も（zh.md の問3、每日经济新闻）。
- 抖音の効果は社員と、外の作家（Effect Creator で組んで出す）が作る。スタジオから 1 本約 3,000 元で買っていた時期もある（zh.md の問1、数英 2019）。
- BytePlus Effects の FAQ：社内にデザイナーの班がいて客の注文で作る。客は EffectCreator の使用権を買えば自分で作れる。
- 剪映の算法の届け出（公示）：効果は「編集・端末での描画・内容の理解」の上にあり、一部は深層学習を使う、とだけ書く。
- 知乎の記事：抖音は Unity・Unreal を使わず、短い動画と AR のための軽い描く係を自分で作った（二次情報）。
- 中国の開発者の記事（掘金 2019・2021）：抖音の定番 6 つ（灵魂出窍・抖动・闪白・毛刺・幻觉・缩放）を GLSL でまねて書いた物がある。ライセンス表示が無いのでコードは借りない（zh.md の問4）。
- 見つからなかった物：CapCut の効果が gl-transitions・ISF・Shadertoy・GPUImage の移植だと示す記事・逆解析・特許。キャッシュの効果ファイルの中身（Lua か shader か）を書いた公開資料。

### 4. だから「借りる先」の決め方

- 元ネタの本物（ByteDance の素材）は借りられない。なので各行の借りる先は、①名前が同じ開いた物があればそれを先頭に（「名前が同じ」と書く）、②無ければ同じ種類の開いた物、の順。
- 一番の借り先は **gl-transitions**（つなぎ目、125 本、MIT。手元の ISF-Files に 68 本が ISF に直して入っている）、**Vidvox ISF-Files**（画面の効果、手元に 327 本、MIT）、**Animista**（入り・出・文字、BSD-2）と **Animate.css v3.7.2**（MIT、v4 は Hippocratic なので v3 を使う）、粒の絵は **Kenney Particle Pack**（CC0）、人の切り抜きは **MediaPipe**（Apache-2.0）。
- 使わない：GSAP（独自ライセンス）、Lygia（Prosperity、商用は買い取り）、Shadertoy の多く（CC BY-NC-SA）、ISF の Boxinator（CC BY-NC、`filament/effects/` に入っているので外す候補）、Smoke Screen・ASCII Art（Shadertoy 由来でライセンスの表示なし）、掘金の抖音まねのコード（ライセンス無し）。

## 規約（取れない理由）

- CapCut の利用規約（最終更新 2026-04-15）6 条 2 項：サービスの逆アセンブル・逆コンパイル・派生物づくり、ソースコードや手法を割り出そうとする事を禁止。10 条：会社の素材（Company Content）は CapCut の中で作る作品のための、取り消せる・人に渡せない使用許諾だけ。それ以上は書面の許可が要る。 https://www.capcut.com/clause/terms-of-service
- 素材の使用許諾（Materials License Agreement）：素材に「特殊効果・フィルター・動画アニメーション・文字の特殊効果」がはっきり入る。 https://www.capcut.com/clause/material-license-agreement?lang=en
- なので：キャッシュの効果を取り出す・形式を解く・同じ物を作るために中を見る、はやらない。ここで使うのは**名前と、つまみの名前と、既定の長さ**だけ（第三者の表から。下の出典）。

## 数

一覧の出所：pyCapCut（GuanYixuan、CapCut の下書きを作る道具、ライセンスの表示なし）の効果の表 4,369 本と、CapCutAPI（sun-guannan、Apache-2.0）の CapCut 版の表のうち pyCapCut に無い物、を番号で合わせて重なりを除いた。英語名は CapCutAPI の表から番号で付けた。名前・有料か・つまみ・既定の長さだけで、中身は入っていない。

「いま」の判定は**名前の言葉から種類を当てた**物で、見て確かめてはいない（Alight Motion の時と違い、CapCut の公式の説明文は無い）。「Motolii に有る」＝同じ種類の物が Motolii にある、で、同じ見た目とは限らない。フィルターは色の表（LUT）が CapCut だけの物なので、いちばん良くて「近い」。人の効果は人の切り抜き（Motolii に無い）が要るので、いちばん良くて「近い」。

| 組 | 全部 | Motolii に有る（同じ種類） | 近い物を借りられる | 見つからない |
|---|---|---|---|---|
| Video effects（画面の効果） | 1595 | 1320 | 149 | 126 |
| Body effects（人の効果） | 260 | 0 | 208 | 52 |
| Transitions（つなぎ目） | 1140 | 347 | 688 | 105 |
| Animations › In（クリップの入り） | 251 | 216 | 19 | 16 |
| Animations › Out（クリップの出） | 219 | 178 | 20 | 21 |
| Animations › Combo（入りから出まで） | 117 | 116 | 1 | 0 |
| Text › In（文字の入り） | 204 | 190 | 7 | 7 |
| Text › Out（文字の出） | 117 | 111 | 3 | 3 |
| Text › Loop（文字のくり返し） | 96 | 84 | 7 | 5 |
| Filters（フィルター） | 454 | 0 | 331 | 123 |
| **合計** | **4453** | **2562** | **1433** | **458** |

### どれだけ全部か

- CapCut の効果はネットから配られ、毎週増える。この表は pyCapCut の表（最終更新 2026-10-16）の時点の物。CapCut の画面の効果は公称「数千」、BytePlus は効果とフィルターで「7 万以上」と書く（それは抖音・TikTok の顔の効果も込み）。**画面の効果・つなぎ目・入り出・文字・フィルターの「名前が付いて棚に出る物」はかなり拾えている、と思う。** 人の効果・ステッカー・文字のテンプレート・音の効果は全部ではない（ステッカーとテンプレートは効果ではないので入れていない）。
- CapCut 自身の棚の分け方（Trending・Basic・Party・Retro…）は表に入っていないし、版と国で変わる。公式ヘルプに出てくる分け方は Web 版の「All・Trending・Cinematic・Retro・Glitch・AI Effects」と「Stylize・Distort・Blur・Color Grading」くらい。なので下の分類は**名前から当てた Motolii 側の分類**。
- 英語の名前があるのは 1088 本、残りは中国語の名前だけ（pyCapCut が中国語の画面で取った名前）。日本語の意味は名前の言葉から機械で付けた。

## 借りる先が見つからない大きな組

| 組 | 種類 | 本数 |
|---|---|---|
| Filters（フィルター） | 名前だけでは分からない | 103 |
| Video effects（画面の効果） | 名前だけでは分からない | 100 |
| Transitions（つなぎ目） | 名前だけでは分からない | 83 |
| Body effects（人の効果） | 名前だけでは分からない | 35 |
| Filters（フィルター） | 映画・ゲームの色 | 20 |
| Animations › In（クリップの入り） | 名前だけでは分からない | 15 |
| Animations › Out（クリップの出） | 名前だけでは分からない | 15 |
| Body effects（人の効果） | ホラー | 9 |
| Video effects（画面の効果） | 稲妻・電気 | 7 |
| Transitions（つなぎ目） | 紙 | 6 |
| Body effects（人の効果） | 稲妻・電気 | 6 |
| Video effects（画面の効果） | ホラー | 6 |
| Text › Loop（文字のくり返し） | 名前だけでは分からない | 5 |
| Transitions（つなぎ目） | 破れる・ちぎれる | 5 |
| Transitions（つなぎ目） | 割れる | 5 |
| Text › In（文字の入り） | 名前だけでは分からない | 4 |

まとめると：
- **名前だけでは中身が分からない物**（Daily・Passion・WOW・中国語の造語など）が一番多い。見てからでないと借りる先が決まらない。
- **フィルターの「映画の色」「場所の色」**（Dune・Wong Kar-wai・西雅图…）：色の表は CapCut の物。開いた LUT 集で同じ名前の物は無い。Motolii の色の段で寄せるしかない。
- **稲妻・電気**（Lightning・Thunderbolt・闪电）：開いた物で使える物が見つからない（Shadertoy の物は商用不可が多い）。
- **紙が破れる・割れる**（Paper Tear・撕纸・破碎・裂纹）：同じく見つからない。
- **ホラー**（Horror・恐怖・恶魔・血）：暗くする・揺らす等は有るが、血や悪魔の絵は無い。
- **人の効果**：切り抜き（MediaPipe）は借りられるが、翼・炎の目・猫耳などの飾りの絵は無い。

## Video effects（画面の効果）（1595）

Motolii に有る 1320・近い物あり 149・見つからない 126。

### 光（265）

- [ ] **3条爆闪**（光・有料）：画面の効果。名前から：強く光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Angel**（光）：画面の効果。名前から：天使・光（中国語名 天使光）。つまみ：強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Black Flash**（光）：画面の効果。名前から：黒く光る・光る（フラッシュ）（中国語名 闪黑）。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Black Flash 2**（光）：画面の効果。名前から：黒く光る・光る（フラッシュ）（中国語名 闪黑 II）。つまみ：速さ・強さ・大きさ・ゆがみ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Blink**（光）：画面の効果。名前から：点滅（中国語名 咔嚓）。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：Animista blink-1（BSD-2、名前が同じ）／ISF Strobe（MIT）
- [ ] **Blinking**（光）：画面の効果。名前から：点滅。つまみ：強さ・背景の動き・色・ぼかし・明るさ・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **Bouncing Glow**（光・有料）：画面の効果。名前から：光る・弾む。つまみ：速さ・強さ・ぼかし・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Butterfly Dream**（光）：画面の効果。名前から：夢・蝶・翼（中国語名 梦蝶）。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Color Phosphor**（光）：画面の効果。名前から：蛍光・カラー・虹・色（中国語名 彩色荧光） — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Colorful**（光）：画面の効果。名前から：画面が光る・虹・色（中国語名 闪屏）。つまみ：速さ・色の濃さ — いま：段の exposure・opacity をキー・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Fade Color（MIT）／ISF Color Phase（MIT）
- [ ] **Diamond Halo**（光・有料）：画面の効果。名前から：光のにじみ・夢っぽい・きらきら・星。つまみ：色の濃さ・範囲・色・ぼかし・速さ・縦ずれ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Disco**（光・有料）：画面の効果。名前から：ディスコ・光・虹・色（中国語名 蹦迪彩光）。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Disco Ball 1**（光）：画面の効果。名前から：光・虹・色・パーティー（中国語名 KTV灯光）。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Dreamy Halo**（光・有料）：画面の効果。名前から：光のにじみ・夢っぽい。つまみ：明るさ・範囲・大きさ・やわらかさ・色の濃さ・シャープ・ぼかし — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Edge Glow**（光）：画面の効果。名前から：光る・縁・縁取り（中国語名 边缘发光）。つまみ：明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **Electro Border**（光）：画面の効果。名前から：縁取り・電気の光・稲妻・電気（中国語名 电光描边）。つまみ：色・横ずれ・速さ・大きさ・強さ・回転 — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **Energy**（光）：画面の効果。名前から：光る（中国語名 冲刺） — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Ethereal**（光・有料）：画面の効果。名前から：光のにじみ・夢っぽい（中国語名 空灵）。つまみ：色の濃さ・大きさ・回転・ぼかし — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Flash**（光）：画面の効果。名前から：光る・光る（フラッシュ）（中国語名 闪光灯）。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Flash**（光・有料）：画面の効果。名前から：光る（フラッシュ）。つまみ：速さ・色・強さ・明るさ・ぼかし — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Flash Light**（光・有料）：画面の効果。名前から：光る（フラッシュ）。つまみ：質感・速さ・明るさ・色・ぼかし — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Flicker**（光・有料）：画面の効果。名前から：点滅（中国語名 闪动）。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：Animista flicker-1（BSD-2、名前が同じ）／ISF Strobe（MIT）
- [ ] **Flickery Shots**（光）：画面の効果。名前から：点滅。つまみ：ぼかし・明るさ・速さ・色 — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **Glamor Shots**（光・有料）：画面の効果。名前から：華やかな撮影。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Gleam**（光）：画面の効果。名前から：光・光る（中国語名 散光）。つまみ：大きさ・明るさ・範囲・色・強さ・やわらかさ・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Glowburst**（光・有料）：画面の効果。名前から：光の玉・点滅・光る（中国語名 闪动光斑）。つまみ：速さ・背景の動き — いま：ISF RGB Strobe・ふるまい random を opacity に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／ISF Strobe（MIT）
- [ ] **Halo**（光）：画面の効果。名前から：光のにじみ・夢っぽい（中国語名 光晕）。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Halo 2**（光）：画面の効果。名前から：光のにじみ・夢っぽい（中国語名 光晕 II）。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Heartbeat**（光・有料）：画面の効果。名前から：光の波・ハート・脈打つ（中国語名 爱心光波）。つまみ：速さ — いま：ふるまい oscillator を scale に — 借りる先：Animista heartbeat（BSD-2、名前が同じ）／Animate.css heartBeat（v3.7.2 は MIT、名前が同じ）／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Ins描边**（光・有料）：画面の効果。名前から：縁取り。つまみ：色・強さ・横の色ずれ・速さ・範囲・大きさ・数・色の濃さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **Lightning Crack**（光・有料）：画面の効果。名前から：雷・画面が光る・稲妻・電気（中国語名 闪屏雷电）。つまみ：ゆがみ・速さ・飾り・色の濃さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Low Exposure**（光）：画面の効果。名前から：露出・周辺減光・暗く（中国語名 曝光降低）。つまみ：速さ — いま：段の exposure・opacity をキー・vignette.wgsl・段の vignette — 借りる先：ISF Fade Color（MIT）
- [ ] **Luminance**（光）：画面の効果。名前から：光る（中国語名 发光）。つまみ：強さ・色の濃さ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：GPUImage Luminance（BSD-3、名前が同じ）
- [ ] **Mist**（光）：画面の効果。名前から：光のにじみ・夢っぽい。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Negative Strobe**（光・有料）：画面の効果。名前から：ストロボ・ネガ・点滅・反転（中国語名 负片频闪）。つまみ：色・速さ・色の濃さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Color Invert・RGB Invert — 借りる先：ISF Strobe（MIT）
- [ ] **Neon**（光）：画面の効果。名前から：蛍光・ネオン（中国語名 荧光线描）。つまみ：背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF-Files Neon（MIT、名前が同じ）／ISF Neon（MIT）
- [ ] **Neon Flash**（光・有料）：画面の効果。名前から：ネオン・光る（フラッシュ）。つまみ：速さ・強さ・明るさ・横の色ずれ — いま：段の exposure・opacity をキー — 借りる先：ISF Neon（MIT）／ISF Fade Color（MIT）
- [ ] **Neon Outline**（光・有料）：画面の効果。名前から：蛍光・縁・ネオン・縁取り（中国語名 边缘荧光）。つまみ：強さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Edges・Edge Blowout＋Glow — 借りる先：ISF Neon（MIT）
- [ ] **Neon Swing**（光・有料）：画面の効果。名前から：ネオン・左右に揺れる・揺れ動く（振り子）（中国語名 霓虹摇摆）。つまみ：速さ・明るさ — いま：ふるまい oscillator を rotate に — 借りる先：ISF Neon（MIT）
- [ ] **Pink Hearts**（光）：画面の効果。名前から：蛍光・ハート（中国語名 荧光爱心）。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Pool Reflection**（光・有料）：画面の効果。名前から：光・影・波紋・鏡（中国語名 水光影）。つまみ：速さ・大きさ・色の濃さ・色・ゆがみ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **Radiating Love**（光）：画面の効果。名前から：放射・ハート・光る（中国語名 爱心放射）。つまみ：速さ・色の濃さ・背景の動き — いま：ISF God Rays・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Rainbow Neon**（光）：画面の効果。名前から：電気の光・虹・ネオン・色（中国語名 彩虹电光）。つまみ：速さ・回転・ゆがみ・大きさ・明るさ・強さ・ぼかし — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／ISF Neon（MIT）
- [ ] **Slit Lighting**（光）：画面の効果。名前から：光の筋。つまみ：強さ・速さ・大きさ・回転・色・色の濃さ・ぼかし — いま：ISF God Rays — 借りる先：見つからない
- [ ] **Snapshot**（光・有料）：画面の効果。名前から：光る（フラッシュ）。つまみ：質感・速さ・明るさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Soft**（光）：画面の効果。名前から：やわらかい光・光のにじみ・夢っぽい（中国語名 柔光）。つまみ：やわらかさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Spectrum Scan**（光）：画面の効果。名前から：スキャン・光・光が走る（中国語名 光谱扫描）。つまみ：背景の動き・強さ・明るさ・速さ・範囲 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Stretched Rays**（光・有料）：画面の効果。名前から：光の筋・伸ばす・傾く。つまみ：数・強さ・色・範囲・速さ・ぼかし — いま：ISF God Rays・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **Strobe**（光）：画面の効果。名前から：ディスコ・光・点滅（中国語名 蹦迪光）。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF-Files Strobe（MIT、名前が同じ）／ISF Strobe（MIT）
- [ ] **Sun**（光）：画面の効果。名前から：光の筋（中国語名 晴天光线）。つまみ：速さ・背景の動き — いま：ISF God Rays — 借りる先：見つからない
- [ ] **Sunset 4**（光）：画面の効果。名前から：光の筋・日の出・日の入り（中国語名 丁达尔光线）。つまみ：速さ・背景の動き — いま：ISF God Rays・キー x・y — 借りる先：見つからない
- [ ] **Sunshine**（光・有料）：画面の効果。名前から：光の筋。つまみ：色の濃さ・背景の動き・速さ・大きさ — いま：ISF God Rays — 借りる先：見つからない
- [ ] **Take Selfie**（光・有料）：画面の効果。名前から：光る（フラッシュ）。つまみ：ぼかし・範囲・強さ・質感・速さ・大きさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **White Border**（光・有料）：画面の効果。名前から：縁取り（中国語名 白色描边）。つまみ：速さ・横ずれ・縦ずれ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **White Diffusion**（光）：画面の効果。名前から：光の波・光のにじみ・夢っぽい（中国語名 白色光波）。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **White Flash**（光）：画面の効果。名前から：白く光る・光る（フラッシュ）（中国語名 闪白）。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Window**（光・有料）：画面の効果。名前から：光・窓・格子・枠（中国語名 窗格光）。つまみ：背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **丁达尔旋焦**（光・有料）：画面の効果。名前から：光の筋。つまみ：質感・色の濃さ・明るさ・ぼかし・範囲・背景の動き・大きさ — いま：ISF God Rays — 借りる先：見つからない
- [ ] **三层闪光**（光・有料）：画面の効果。名前から：光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **交叉震闪**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：色・速さ・背景の動き・強さ・明るさ・ゆがみ — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **倒转闪烁**（光・有料）：画面の効果。名前から：点滅・逆さ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・rotate_x・rotate_y — 借りる先：ISF Strobe（MIT）
- [ ] **光幕**（光・有料）：画面の効果。名前から：光。つまみ：強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **光线拖影**（光・有料）：画面の効果。名前から：光の筋・残像。つまみ：明るさ・ぼかし・速さ・色 — いま：ISF God Rays・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **光速淡紫色**（光・有料）：画面の効果。名前から：光・紫。つまみ：速さ・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **全息扫描**（光・有料）：画面の効果。名前から：スキャン・ホログラム。つまみ：明るさ・ぼかし・背景の動き・速さ・色・縦ずれ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Color Phase（MIT）
- [ ] **分割光感**（光・有料）：画面の効果。名前から：光・分割。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Sliding Strips（MIT）
- [ ] **分割闪白**（光・有料）：画面の効果。名前から：白く光る・分割。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）／ISF Sliding Strips（MIT）
- [ ] **分屏闪光**（光・有料）：画面の効果。名前から：光る・分割画面。つまみ：速さ・明るさ・飾り — いま：段の exposure・opacity をキー・ISF Quad Tile — 借りる先：ISF Fade Color（MIT）／ISF MultiFrame 2x2・3x3（MIT）
- [ ] **动感光束**（光・有料）：画面の効果。名前から：光の束。つまみ：強さ・範囲・速さ・横ずれ — いま：ISF God Rays — 借りる先：見つからない
- [ ] **十字爆闪**（光・有料）：画面の効果。名前から：強く光る・字。つまみ：強さ・範囲・明るさ・大きさ — いま：段の exposure・opacity をキー・animators print_on・print_off — 借りる先：ISF Fade Color（MIT）
- [ ] **半屏闪光**（光・有料）：画面の効果。名前から：光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **卡带闪回**（光・有料）：画面の効果。名前から：光る・カード。つまみ：大きさ・ぼかし・明るさ・横の色ずれ・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Strobe（MIT）
- [ ] **卡片闪屏**（光・有料）：画面の効果。名前から：画面が光る・カード。つまみ：速さ — いま：段の exposure・opacity をキー・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Fade Color（MIT）
- [ ] **卡片闪现**（光・有料）：画面の効果。名前から：ちらっと出る・カード。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Strobe（MIT）
- [ ] **双重爆闪**（光・有料）：画面の効果。名前から：強く光る。つまみ：速さ・強さ・大きさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **双重辉光**（光・有料）：画面の効果。名前から：光のにじみ。つまみ：範囲・横ずれ・縦の色ずれ・ぼかし・明るさ・色・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **发光 2**（光・有料）：画面の効果。名前から：光る。つまみ：明るさ・大きさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **发光代码**（光・有料）：画面の効果。名前から：光る。つまみ：色 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **发光旋转器**（光・有料）：画面の効果。名前から：光る・回転。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・キー rotate — 借りる先：見つからない
- [ ] **发光漂移**（光・有料）：画面の効果。名前から：光る。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **变色闪光**（光）：画面の効果。名前から：光る・色が変わる。つまみ：範囲・大きさ・色・明るさ・速さ・背景の動き・強さ・ぼかし — いま：段の exposure・opacity をキー・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Fade Color（MIT）／ISF Color Phase（MIT）
- [ ] **四分闪转**（光・有料）：画面の効果。名前から：光る・回る — いま：ISF RGB Strobe・ふるまい random を opacity に・キー rotate — 借りる先：ISF Strobe（MIT）
- [ ] **多图快闪**（光・有料）：画面の効果。名前から：光る — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **天使光辉**（光・有料）：画面の効果。名前から：天使・光。つまみ：速さ・強さ・範囲 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **失焦光斑**（光・有料）：画面の効果。名前から：光の玉・ピンぼけ。つまみ：大きさ・数・色の濃さ・速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **好莱坞光效**（光・有料）：画面の効果。名前から：光。つまみ：速さ・数・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **居中闪切**（光・有料）：画面の効果。名前から：光って切り替わる。つまみ：速さ・色の濃さ・ノイズ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **岩石光感**（光・有料）：画面の効果。名前から：光。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **巫师的光环**（光・有料）：画面の効果。名前から：光の輪。つまみ：速さ・大きさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **幻境漂移**（光・有料）：画面の効果。名前から：幻の世界。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **开关**（光・有料）：画面の効果。名前から：スイッチ。つまみ：強さ・数 — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **弹闪**（光・有料）：画面の効果。名前から：光る・弾む。つまみ：速さ・ゆがみ・明るさ・範囲・強さ・ぼかし — いま：ISF RGB Strobe・ふるまい random を opacity に・animators bounce_in・spring_in — 借りる先：ISF Strobe（MIT）
- [ ] **强烈日光**（光）：画面の効果。名前から：光。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **强速频闪**（光・有料）：画面の効果。名前から：ストロボ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **强震频闪**（光・有料）：画面の効果。名前から：ストロボ・強い揺れ。つまみ：速さ・色の濃さ・色 — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **彩色射线**（光・有料）：画面の効果。名前から：光の筋・カラー。つまみ：速さ — いま：ISF God Rays・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色电光**（光）：画面の効果。名前から：電気の光・カラー。つまみ：強さ・範囲・速さ・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色边缘**（光・有料）：画面の効果。名前から：縁・カラー。つまみ：色の濃さ・大きさ・強さ・回転 — いま：ISF Edges・Edge Blowout＋Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色闪烁**（光・有料）：画面の効果。名前から：点滅・カラー。つまみ：色・シャープ・明るさ・背景の動き・強さ・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Strobe（MIT）／ISF Color Phase（MIT）
- [ ] **彩虹泛光**（光・有料）：画面の効果。名前から：光のにじみ・虹。つまみ：色の濃さ・明るさ・大きさ・数・範囲 — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩虹闪屏**（光・有料）：画面の効果。名前から：画面が光る・虹。つまみ：背景の動き・大きさ・ぼかし・色・速さ — いま：段の exposure・opacity をキー・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Fade Color（MIT）／ISF Color Phase（MIT）
- [ ] **微震闪黑**（光・有料）：画面の効果。名前から：黒く光る・揺れ。つまみ：強さ・背景の動き・明るさ・速さ — いま：段の exposure・opacity をキー・ふるまい noise を x・y に — 借りる先：ISF Fade Color（MIT）／ISF Shake（MIT）
- [ ] **快门闪光**（光・有料）：画面の効果。名前から：シャッター・光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **慢速快门**（光・有料）：画面の効果。名前から：シャッター。つまみ：強さ・色・速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **扫描复制**（光・有料）：画面の効果。名前から：スキャン・複製。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Ghosting（MIT）
- [ ] **抖动轮廓**（光・有料）：画面の効果。名前から：輪郭・揺れ。つまみ：速さ・強さ — いま：ISF Edges・Edge Blowout＋Glow・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **抖动闪光**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：ぼかし・強さ — いま：段の exposure・opacity をキー・ふるまい noise を x・y に — 借りる先：ISF Fade Color（MIT）／ISF Shake（MIT）
- [ ] **折射光效**（光・有料）：画面の効果。名前から：光・折る。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：gl-transitions Fold（MIT）
- [ ] **推闪三切**（光・有料）：画面の効果。名前から：光る・押す。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・キー x・y — 借りる先：ISF Strobe（MIT）
- [ ] **摇晃闪光灯**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：速さ — いま：段の exposure・opacity をキー・ふるまい noise を x・y に — 借りる先：ISF Fade Color（MIT）／ISF Shake（MIT）
- [ ] **摇曳光环**（光・有料）：画面の効果。名前から：光の輪・揺れ。つまみ：速さ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **放射幻影**（光・有料）：画面の効果。名前から：放射・幻影。つまみ：ぼかし・範囲・強さ・大きさ・速さ — いま：ISF God Rays・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **放射性脉冲**（光・有料）：画面の効果。名前から：放射・脈打つ。つまみ：速さ — いま：ISF God Rays・ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **旋转发光**（光・有料）：画面の効果。名前から：光る・回転。つまみ：強さ・大きさ・横ずれ・縦ずれ・明るさ・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow・キー rotate — 借りる先：見つからない
- [ ] **旋转快门**（光・有料）：画面の効果。名前から：シャッター・回転。つまみ：速さ — いま：段の exposure・opacity をキー・キー rotate — 借りる先：ISF Fade Color（MIT）
- [ ] **明亮光晕特效**（光・有料）：画面の効果。名前から：光のにじみ・明るい。つまみ：縦の色ずれ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow・段の exposure・ISF Exposure Adjust — 借りる先：見つからない
- [ ] **星星光晕**（光・有料）：画面の効果。名前から：光のにじみ・星。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **暗黑频闪**（光・有料）：画面の効果。名前から：ストロボ・暗い。つまみ：色の濃さ・範囲・速さ・質感 — いま：ISF RGB Strobe・ふるまい random を opacity に・vignette.wgsl・段の vignette — 借りる先：ISF Strobe（MIT）
- [ ] **曝光变焦**（光・有料）：画面の効果。名前から：露出・ズーム。つまみ：ぼかし・やわらかさ・明るさ・速さ — いま：段の exposure・opacity をキー・キー scale・ISF Zoom — 借りる先：ISF Fade Color（MIT）
- [ ] **条形光效**（光）：画面の効果。名前から：光・帯。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Sliding Strips（MIT）
- [ ] **极地光束**（光・有料）：画面の効果。名前から：光の束。つまみ：速さ — いま：ISF God Rays — 借りる先：見つからない
- [ ] **极闪速旋**（光・有料）：画面の効果。名前から：光る。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **柔和辉光**（光・有料）：画面の効果。名前から：光のにじみ。つまみ：明るさ・範囲・大きさ・色・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦回回响**（光・有料）：画面の効果。名前から：夢。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦境**（光）：画面の効果。名前から：夢の中。つまみ：大きさ・数・色の濃さ・明るさ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦境 II**（光・有料）：画面の効果。名前から：夢の中。つまみ：大きさ・数・色の濃さ・明るさ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦境 III**（光）：画面の効果。名前から：夢の中。つまみ：大きさ・数・色の濃さ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦境 IV**（光）：画面の効果。名前から：夢の中。つまみ：大きさ・数・色の濃さ・やわらかさ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦境印象**（光・有料）：画面の効果。名前から：夢の中。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **梦幻微光**（光・有料）：画面の効果。名前から：かすかな光・夢。つまみ：速さ・大きさ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **模糊光环**（光・有料）：画面の効果。名前から：光の輪・ぼかし。つまみ：色の濃さ・ぼかし — いま：段の bloom・物の glow・ISF Bloom・Glow・物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **横向闪光**（光・有料）：画面の効果。名前から：光る。つまみ：明るさ・範囲・大きさ・色の濃さ・強さ・質感・速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **横向闪光 II**（光・有料）：画面の効果。名前から：光る。つまみ：明るさ・範囲・ぼかし・強さ・速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **氛围边框**（光）：画面の効果。名前から：雰囲気・枠。つまみ：質感・色・背景の動き・色の濃さ・速さ・明るさ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **泛光扫描**（光・有料）：画面の効果。名前から：光のにじみ・スキャン。つまみ：強さ・明るさ・ぼかし・色・背景の動き・速さ・横ずれ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **泛光爆闪**（光）：画面の効果。名前から：光のにじみ・強く光る。つまみ：明るさ・背景の動き・速さ・強さ・ぼかし — いま：段の bloom・物の glow・ISF Bloom・Glow・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **泛光闪动**（光・有料）：画面の効果。名前から：光のにじみ・点滅。つまみ：ぼかし・強さ・明るさ・速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **派对节拍**（光・有料）：画面の効果。名前から：パーティー・拍子。つまみ：速さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **流光碎闪**（光・有料）：画面の効果。名前から：流れる光・光る。つまみ：速さ・色の濃さ・強さ・明るさ・色 — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Strobe（MIT）
- [ ] **海报描边**（光・有料）：画面の効果。名前から：縁取り・ポスター。つまみ：ノイズ・大きさ・数・色・色の濃さ・ゆがみ — いま：ISF Edges・Edge Blowout＋Glow・ISF Posterize — 借りる先：見つからない
- [ ] **涂鸦曝光**（光・有料）：画面の効果。名前から：露出・落書き。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）／ISF Doodler（MIT）
- [ ] **混乱闪烁**（光・有料）：画面の効果。名前から：点滅・乱れ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Strobe（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **漏光对焦**（光・有料）：画面の効果。名前から：光漏れ・ピント合わせ。つまみ：速さ・ぼかし・大きさ・背景の動き・強さ・ゆがみ — いま：段の focus・aperture＋物の blur をキー — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **漏光闪烁**（光・有料）：画面の効果。名前から：光漏れ・点滅。つまみ：速さ・強さ・明るさ・大きさ・色 — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF v002 Light Leak（MIT）／ISF Strobe（MIT）
- [ ] **激光滑动**（光・有料）：画面の効果。名前から：レーザー・滑る。つまみ：強さ — いま：キー x・y — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **火花闪烁**（光・有料）：画面の効果。名前から：点滅・火花。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）／Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **烟花光晕**（光・有料）：画面の効果。名前から：光のにじみ・花火。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **爆闪锐化**（光・有料）：画面の効果。名前から：強く光る・シャープ。つまみ：速さ・強さ・色・明るさ・シャープ — いま：段の exposure・opacity をキー・ISF Sharpen RGB・Unsharp Mask — 借りる先：ISF Fade Color（MIT）
- [ ] **珠光碎闪**（光・有料）：画面の効果。名前から：光る・光。つまみ：色の濃さ・範囲・大きさ・色・ぼかし — いま：ISF RGB Strobe・ふるまい random を opacity に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Strobe（MIT）
- [ ] **眩光滑动**（光・有料）：画面の効果。名前から：まぶしい光・滑る。つまみ：速さ・色の濃さ・背景の動き・色 — いま：段の lens_flare・ISF Lens Flare・キー x・y — 借りる先：見つからない
- [ ] **矩形闪白**（光・有料）：画面の効果。名前から：白く光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **矩阵频闪**（光・有料）：画面の効果。名前から：ストロボ。つまみ：速さ・強さ・色・大きさ・範囲 — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **碎闪描边**（光・有料）：画面の効果。名前から：縁取り・光る。つまみ：色の濃さ・色・強さ — いま：ISF Edges・Edge Blowout＋Glow・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **秀场高光**（光・有料）：画面の効果。名前から：光 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **竖向闪光**（光）：画面の効果。名前から：光る。つまみ：数・ぼかし・強さ・明るさ・範囲・背景の動き・速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **竖线屏闪**（光・有料）：画面の効果。名前から：画面がちらつく。つまみ：明るさ・ぼかし・範囲・強さ・色の濃さ・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **粉色光滴**（光・有料）：画面の効果。名前から：光・ピンク。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **精致辉光**（光・有料）：画面の効果。名前から：光のにじみ。つまみ：明るさ・範囲・大きさ・やわらかさ・シャープ・背景の動き・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **糖果微光**（光・有料）：画面の効果。名前から：かすかな光。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **紧急闪光**（光・有料）：画面の効果。名前から：光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **紫光夜**（光）：画面の効果。名前から：光・紫。つまみ：数・色・強さ・大きさ・回転・縦ずれ・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **红色闪耀**（光・有料）：画面の効果。名前から：光る・赤。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Strobe（MIT）／ISF Color Phase（MIT）
- [ ] **聚光摇曳特效**（光・有料）：画面の効果。名前から：光・揺れ。つまみ：速さ・大きさ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **聚光灯闪烁 2**（光・有料）：画面の効果。名前から：点滅・光。つまみ：明るさ — いま：ISF RGB Strobe・ふるまい random を opacity に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Strobe（MIT）
- [ ] **背光调节**（光・有料）：画面の効果。名前から：光。つまみ：速さ・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **膨胀闪光**（光・有料）：画面の効果。名前から：光る。つまみ：ゆがみ・明るさ・大きさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **舞动闪光**（光・有料）：画面の効果。名前から：光る。つまみ：速さ・強さ・シャープ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **色彩扫描**（光・有料）：画面の効果。名前から：スキャン・色。つまみ：速さ・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Color Phase（MIT）
- [ ] **荧光玫瑰**（光・有料）：画面の効果。名前から：蛍光。つまみ：速さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **荧光蝙蝠**（光）：画面の効果。名前から：蛍光。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **萤光**（光）：画面の効果。名前から：光。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **蓝光扫描**（光・有料）：画面の効果。名前から：光が走る・青。つまみ：明るさ・ぼかし・強さ・速さ・範囲 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Color Phase（MIT）
- [ ] **蓝光爆闪**（光）：画面の効果。名前から：強く光る・光・青。つまみ：明るさ・速さ・範囲・色 — いま：段の exposure・opacity をキー・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Fade Color（MIT）／ISF Color Phase（MIT）
- [ ] **裂开的闪光**（光・有料）：画面の効果。名前から：光る・裂ける。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）／ISF Sliding Strips（MIT）
- [ ] **负片描边**（光・有料）：画面の効果。名前から：縁取り・ネガ。つまみ：速さ — いま：ISF Edges・Edge Blowout＋Glow・ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **负片闪烁**（光）：画面の効果。名前から：点滅・ネガ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Color Invert・RGB Invert — 借りる先：ISF Strobe（MIT）
- [ ] **负片闪电**（光・有料）：画面の効果。名前から：稲妻・ネガ。つまみ：速さ・質感・色の濃さ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **赛博故障**（光・有料）：画面の効果。名前から：サイバー・グリッチ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Neon（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **超强光感**（光・有料）：画面の効果。名前から：光。つまみ：速さ・強さ・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **轰鸣闪光**（光・有料）：画面の効果。名前から：光る。つまみ：明るさ・数 — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **边缘加色**（光・有料）：画面の効果。名前から：縁。つまみ：強さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **边缘加色 II**（光・有料）：画面の効果。名前から：縁。つまみ：強さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **边缘加色 III**（光・有料）：画面の効果。名前から：縁。つまみ：強さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **运镜快门**（光・有料）：画面の効果。名前から：シャッター・カメラワーク。つまみ：速さ — いま：段の exposure・opacity をキー・キー scale・ISF Zoom — 借りる先：ISF Fade Color（MIT）
- [ ] **迷幻轮廓**（光・有料）：画面の効果。名前から：輪郭・サイケ。つまみ：強さ — いま：ISF Edges・Edge Blowout＋Glow・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **速切闪黑**（光・有料）：画面の効果。名前から：黒く光る。つまみ：速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **重复震闪**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：明るさ・色・強さ・ぼかし・範囲・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **金色辉光**（光・有料）：画面の効果。名前から：光のにじみ・金色。つまみ：色の濃さ・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金边闪烁**（光・有料）：画面の効果。名前から：点滅・金色。つまみ：色・明るさ・速さ・横ずれ・縦ずれ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Star（1 個） — 借りる先：ISF Strobe（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **钻石闪光**（光・有料）：画面の効果。名前から：光る・ダイヤ。つまみ：速さ・質感 — いま：段の exposure・opacity をキー・ISF Star（1 個） — 借りる先：ISF Fade Color（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪光**（光・有料）：画面の効果。名前から：光る。つまみ：大きさ・回転・ノイズ・ぼかし・強さ・色の濃さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光上移**（光・有料）：画面の効果。名前から：光る・上へ動く。つまみ：速さ — いま：段の exposure・opacity をキー・キー x・y — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光摇动**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：速さ — いま：段の exposure・opacity をキー・ふるまい noise を x・y に — 借りる先：ISF Fade Color（MIT）／ISF Shake（MIT）
- [ ] **闪光灯 I**（光・有料）：画面の効果。名前から：光る。つまみ：速さ・強さ・ゆがみ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光灯 II**（光・有料）：画面の効果。名前から：光る。つまみ：強さ・色の濃さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光灯 IV**（光・有料）：画面の効果。名前から：光る。つまみ：明るさ・速さ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光翻转**（光・有料）：画面の効果。名前から：光る・裏返る。つまみ：明るさ — いま：段の exposure・opacity をキー・rotate_x・rotate_y — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光跳动**（光・有料）：画面の効果。名前から：光る・跳ねる。つまみ：速さ・強さ・明るさ・ぼかし — いま：段の exposure・opacity をキー・animators bounce_in・spring_in — 借りる先：ISF Fade Color（MIT）
- [ ] **闪光震动**（光・有料）：画面の効果。名前から：光る・振動。つまみ：色・範囲・背景の動き・明るさ・速さ — いま：段の exposure・opacity をキー・ふるまい noise を x・y に — 借りる先：ISF Fade Color（MIT）／ISF Shake（MIT）
- [ ] **闪击**（光・有料）：画面の効果。名前から：光る・打つ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **闪刺晃影**（光・有料）：画面の効果。名前から：光る・影・揺れ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Strobe（MIT）／ISF Ghosting（MIT）
- [ ] **闪动光影**（光・有料）：画面の効果。名前から：点滅・光・影。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Strobe（MIT）／ISF Ghosting（MIT）
- [ ] **闪动拖影**（光・有料）：画面の効果。名前から：点滅・残像。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Strobe（MIT）／ISF Ghosting（MIT）
- [ ] **闪动页面**（光・有料）：画面の効果。名前から：点滅。つまみ：速さ・大きさ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **闪烁迪斯科**（光・有料）：画面の効果。名前から：ディスコ・点滅。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **闪爆裂影**（光・有料）：画面の効果。名前から：光る・影。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Strobe（MIT）／ISF Ghosting（MIT）
- [ ] **闪白 II**（光・有料）：画面の効果。名前から：白く光る。つまみ：速さ・明るさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **闪闪发光**（光・有料）：画面の効果。名前から：光る・きらきら。つまみ：色の濃さ・大きさ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪闪发光 II**（光・有料）：画面の効果。名前から：光る・きらきら。つまみ：色の濃さ・大きさ・回転 — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **随机闪切**（光）：画面の効果。名前から：光って切り替わる。つまみ：背景の動き・速さ・ぼかし・横の色ずれ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **震动光束**（光・有料）：画面の効果。名前から：光の束・振動。つまみ：強さ・色・縦ずれ・明るさ・ぼかし・速さ・範囲 — いま：ISF God Rays・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **震动发光**（光）：画面の効果。名前から：光る・振動。つまみ：速さ・強さ・やわらかさ・横の色ずれ・明るさ・ゆがみ — いま：段の bloom・物の glow・ISF Bloom・Glow・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **震动屏闪**（光・有料）：画面の効果。名前から：画面がちらつく・振動。つまみ：速さ・強さ・明るさ・範囲 — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **震动扫光**（光・有料）：画面の効果。名前から：光が走る・振動。つまみ：強さ・速さ・色・横ずれ・縦ずれ — いま：ふるまい noise を x・y に — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Shake（MIT）
- [ ] **震闪渐黑**（光・有料）：画面の効果。名前から：光る・揺れ・じわっと。つまみ：強さ・明るさ・範囲・速さ・背景の動き — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **震颤泛光**（光・有料）：画面の効果。名前から：光のにじみ・揺れ。つまみ：強さ・やわらかさ・ノイズ・明るさ・ゆがみ・色の濃さ・速さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **霓虹光线**（光）：画面の効果。名前から：光の筋・ネオン。つまみ：明るさ・ぼかし・強さ・速さ・色・範囲・横ずれ — いま：ISF God Rays — 借りる先：ISF Neon（MIT）
- [ ] **霓虹旋转**（光・有料）：画面の効果。名前から：ネオン・回転。つまみ：速さ・強さ — いま：キー rotate — 借りる先：ISF Neon（MIT）
- [ ] **霓虹碰撞**（光・有料）：画面の効果。名前から：ネオン・ぶつかる。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Neon（MIT）／ISF Shake（MIT）
- [ ] **霓虹闪乱**（光・有料）：画面の効果。名前から：ネオン・光る。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Neon（MIT）／ISF Strobe（MIT）
- [ ] **霓虹鬼影**（光・有料）：画面の効果。名前から：ネオン・影。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Neon（MIT）／ISF Ghosting（MIT）
- [ ] **颜色频闪**（光・有料）：画面の効果。名前から：ストロボ・色。つまみ：速さ・色・色の濃さ・強さ・ぼかし — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Strobe（MIT）／ISF Color Phase（MIT）
- [ ] **高饱和负闪**（光・有料）：画面の効果。名前から：光る・ネガ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Color Invert・RGB Invert — 借りる先：ISF Strobe（MIT）
- [ ] **魔法闪光**（光・有料）：画面の効果。名前から：光る・魔法。つまみ：速さ — いま：段の exposure・opacity をキー・ISF Star（1 個） — 借りる先：ISF Fade Color（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **鲜艳闪光**（光・有料）：画面の効果。名前から：光る。つまみ：数・大きさ — いま：段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **黑白眩光**（光・有料）：画面の効果。名前から：まぶしい光・白黒。つまみ：強さ・色 — いま：段の lens_flare・ISF Lens Flare・ISF Color Monochrome — 借りる先：見つからない
- [ ] **黑边闪震**（光・有料）：画面の効果。名前から：光る・揺れ。つまみ：速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ふるまい noise を x・y に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **Butterflies**（光・有料）：画面の効果。名前から：光の玉・蝶・翼（中国語名 蝴蝶光斑）。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Electro Heart**（光・有料）：画面の効果。名前から：電気の光・ハート・稲妻・電気（中国語名 电光爱心）。つまみ：範囲・速さ・強さ・色・大きさ・横ずれ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Lightflow Scan**（光・有料）：画面の効果。名前から：光が走る・スキャン。つまみ：速さ・明るさ・ゆがみ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Sci-Fi Scan**（光・有料）：画面の効果。名前から：光が走る・スキャン。つまみ：やわらかさ・明るさ・大きさ・色・ぼかし・範囲・速さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **六边形光斑**（光・有料）：画面の効果。名前から：光の玉。つまみ：大きさ・強さ・速さ・横ずれ — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **动感光波**（光・有料）：画面の効果。名前から：光の波 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **南瓜光斑**（光）：画面の効果。名前から：光の玉・かぼちゃ。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞光斑**（光・有料）：画面の効果。名前から：光の玉・クリスマス。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **天际扫描**（光・有料）：画面の効果。名前から：スキャン。つまみ：速さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **扫描展开**（光・有料）：画面の効果。名前から：スキャン・広がる。つまみ：色 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **扫描条**（光・有料）：画面の効果。名前から：スキャン。つまみ：速さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **散景漏光**（光・有料）：画面の効果。名前から：光漏れ。つまみ：速さ・強さ — いま：無い — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **景深扫光**（光・有料）：画面の効果。名前から：光が走る。つまみ：速さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **景深扫描**（光・有料）：画面の効果。名前から：スキャン。つまみ：速さ・色・明るさ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **暖冬光斑**（光・有料）：画面の効果。名前から：光の玉。つまみ：大きさ・数・色の濃さ・ぼかし・やわらかさ・速さ — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **极光扫描**（光・有料）：画面の効果。名前から：光が走る。つまみ：速さ・強さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **泡泡光斑**（光・有料）：画面の効果。名前から：光の玉・泡。つまみ：大きさ・数・速さ・強さ・回転・色・色の濃さ・ぼかし・やわらかさ — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **爱心光斑**（光）：画面の効果。名前から：光の玉・ハート。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter／ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心扫光**（光）：画面の効果。名前から：光が走る・ハート。つまみ：明るさ・やわらかさ・回転・速さ・色の濃さ・強さ・大きさ・背景の動き — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心扫光2**（光・有料）：画面の効果。名前から：光が走る・ハート。つまみ：速さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Heart（MIT）＋物の copies・scatter
- [ ] **篝火派对**（光・有料）：画面の効果。名前から：パーティー・火。つまみ：色・縦ずれ・速さ・強さ・色の濃さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **绚丽光斑**（光・有料）：画面の効果。名前から：光の玉。つまみ：速さ・明るさ・範囲・強さ・ノイズ — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **赛博未来**（光）：画面の効果。名前から：サイバー — いま：無い — 借りる先：ISF Neon（MIT）
- [ ] **超大光斑**（光・有料）：画面の効果。名前から：光の玉。つまみ：大きさ・数・強さ・速さ・色の濃さ — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **隔行扫描**（光）：画面の効果。名前から：スキャン。つまみ：強さ・色の濃さ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **霓虹印记**（光・有料）：画面の効果。名前から：ネオン。つまみ：速さ — いま：無い — 借りる先：ISF Neon（MIT）
- [ ] **霓虹彩边**（光・有料）：画面の効果。名前から：ネオン。つまみ：速さ・数・範囲・強さ — いま：無い — 借りる先：ISF Neon（MIT）
- [ ] **霓虹碎片**（光）：画面の効果。名前から：ネオン・破片 — いま：無い — 借りる先：ISF Neon（MIT）
- [ ] **高光扫屏**（光・有料）：画面の効果。名前から：光が走る。つまみ：速さ・ゆがみ — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Lightning**（光）：画面の効果。名前から：稲妻・電気。つまみ：色・明るさ・速さ・回転・飾り・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **Thunderbolt**（光・有料）：画面の効果。名前から：稲妻・電気。つまみ：速さ・ゆがみ・明るさ・強さ・色・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **Thunderbolt 2**（光・有料）：画面の効果。名前から：稲妻・電気。つまみ：背景の動き・速さ・色・ゆがみ・強さ・明るさ — いま：無い — 借りる先：見つからない
- [ ] **告白氛围**（光）：画面の効果。名前から：雰囲気 — いま：無い — 借りる先：見つからない
- [ ] **夜店氛围**（光・有料）：画面の効果。名前から：雰囲気。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **电光包围**（光）：画面の効果。名前から：電気の光。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **电击**（光・有料）：画面の効果。名前から：電撃。つまみ：速さ・明るさ — いま：無い — 借りる先：見つからない
- [ ] **童趣闪电**（光・有料）：画面の効果。名前から：稲妻。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **闪电**（光）：画面の効果。名前から：稲妻。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない

### 動き（236）

- [ ] **360度运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **360运镜 II**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ・明るさ・ぼかし — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **3D Surround**（動き・有料）：画面の効果。名前から：立体。つまみ：速さ・強さ・大きさ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D互动弹窗**（動き・有料）：画面の効果。名前から：3D・弾む・窓 — いま：rotate_x・rotate_y・z（2.5D）・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **3D动态照片**（動き）：画面の効果。名前から：3D・写真 — いま：rotate_x・rotate_y・z（2.5D）・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **3D卡片**（動き・有料）：画面の効果。名前から：3D・カード。つまみ：質感・強さ・速さ・大きさ・色・縦ずれ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D卡片切开**（動き・有料）：画面の効果。名前から：3D・カード。つまみ：速さ・大きさ・明るさ・ゆがみ・範囲 — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D层次**（動き・有料）：画面の効果。名前から：3D。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D旋转小花**（動き・有料）：画面の効果。名前から：回転・3D・花。つまみ：速さ — いま：キー rotate・rotate_x・rotate_y・z（2.5D） — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **3D气球掉落**（動き・有料）：画面の効果。名前から：落ちる・3D。つまみ：速さ・大きさ — いま：キー x・y・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D浮动花朵**（動き・有料）：画面の効果。名前から：3D・花。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **3D环绕屏**（動き・有料）：画面の効果。名前から：周りを回る・3D。つまみ：速さ・強さ・ぼかし・大きさ — いま：キー rotate・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D轮播**（動き・有料）：画面の効果。名前から：3D・めくり替え。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D运镜**（動き）：画面の効果。名前から：カメラワーク・3D — いま：キー scale・ISF Zoom・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3次推近**（動き・有料）：画面の効果。名前から：寄る。つまみ：速さ・ぼかし・大きさ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **90度缩小**（動き・有料）：画面の効果。名前から：縮小。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **Beat Shots**（動き）：画面の効果。名前から：脈打つ。つまみ：速さ・ゆがみ・明るさ・強さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **Bling飘落**（動き・有料）：画面の効果。名前から：漂う・落ちる。つまみ：速さ・数・背景の動き・大きさ・明るさ・色の濃さ — いま：キー x・y — 借りる先：見つからない
- [ ] **Camera Beats**（動き・有料）：画面の効果。名前から：脈打つ。つまみ：横ずれ・縦ずれ・強さ・背景の動き・明るさ・範囲・速さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **Camera Beats 2**（動き・有料）：画面の効果。名前から：脈打つ。つまみ：速さ・範囲・ぼかし・明るさ・強さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **Camera Dance**（動き・有料）：画面の効果。名前から：踊る（揺れ）。つまみ：明るさ・ぼかし・速さ・範囲・大きさ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Camera Shake**（動き）：画面の効果。名前から：揺れ（中国語名 轻微抖动）。つまみ：範囲・速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Cherry Blossom**（動き）：画面の効果。名前から：落ちる・花びら・羽（中国語名 落樱）。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **Circuit**（動き・有料）：画面の効果。名前から：流れる文字・9 分割画面（中国語名 九屏跑马灯） — いま：キー x・y・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Club Mood**（動き・有料）：画面の効果。名前から：左右に揺れる・パーティー（中国語名 幻术摇摆）。つまみ：速さ・色の濃さ — いま：ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **Dizzy**（動き）：画面の効果。名前から：揺れ。つまみ：大きさ・速さ・背景の動き・色の濃さ・質感・色 — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **DJ旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：大きさ — いま：キー rotate — 借りる先：見つからない
- [ ] **Dolly Back**（動き・有料）：画面の効果。名前から：拡大・縮小・寄る・顔。つまみ：速さ・大きさ・強さ・明るさ・背景の動き・ぼかし — いま：キー scale・ISF Zoom — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Mini Zoom**（動き）：画面の効果。名前から：拡大・縮小・寄る（中国語名 轻微放大）。つまみ：速さ・範囲 — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **Move Track**（動き・有料）：画面の効果。名前から：滑る・動く。つまみ：速さ・数・色・飾り・強さ・背景の動き・色の濃さ・質感・大きさ — いま：キー x・y — 借りる先：見つからない
- [ ] **Pulse**（動き）：画面の効果。名前から：鼓動・脈打つ（中国語名 心跳）。つまみ：速さ — いま：ふるまい oscillator を scale に — 借りる先：Animate.css pulse（v3.7.2 は MIT、名前が同じ）
- [ ] **Quick Math**（動き）：画面の効果。名前から：飛ぶ（中国語名 飞速计算） — いま：キー x・y — 借りる先：見つからない
- [ ] **Rebound Swing**（動き）：画面の効果。名前から：左右に揺れる・跳ね返る・揺れ動く（振り子）・弾む（中国語名 回弹摇摆）。つまみ：速さ・大きさ・強さ — いま：ふるまい oscillator を rotate に・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Shake**（動き）：画面の効果。名前から：左右に揺れる・揺れ（中国語名 摇摆 I）。つまみ：速さ・範囲 — いま：ふるまい oscillator を rotate に・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Shake**（動き）：画面の効果。名前から：振動・揺れ（中国語名 震动）。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Shake 2**（動き）：画面の効果。名前から：左右に揺れる・揺れ（中国語名 摇摆 II）。つまみ：速さ・範囲 — いま：ふるまい oscillator を rotate に・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Shake 3**（動き）：画面の効果。名前から：揺れ（中国語名 抖动）。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Shaky Dolly**（動き）：画面の効果。名前から：揺れ・拡大・縮小・寄る。つまみ：強さ・速さ・大きさ — いま：ふるまい noise を x・y に・キー scale・ISF Zoom — 借りる先：ISF Shake（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Slide Viral**（動き・有料）：画面の効果。名前から：滑る・動く。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **Slinky 扭转**（動き・有料）：画面の効果。名前から：回る。つまみ：速さ・強さ — いま：キー rotate — 借りる先：見つからない
- [ ] **Spin & Shake**（動き・有料）：画面の効果。名前から：揺れ・回転。つまみ：速さ・回転・強さ・ぼかし・明るさ — いま：ふるまい noise を x・y に・キー rotate — 借りる先：ISF Shake（MIT）
- [ ] **Spin & Shake 2**（動き・有料）：画面の効果。名前から：揺れ・回転。つまみ：速さ・ぼかし・回転・縦ずれ・横ずれ・明るさ — いま：ふるまい noise を x・y に・キー rotate — 借りる先：ISF Shake（MIT）
- [ ] **Spinning Space**（動き・有料）：画面の効果。名前から：回転・空間・渦・ねじれ（中国語名 旋转空间）。つまみ：大きさ・ゆがみ・回転・色の濃さ・数 — いま：キー rotate・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **Stretch & Swivel**（動き・有料）：画面の効果。名前から：伸ばす・傾く。つまみ：強さ・ぼかし・明るさ・ゆがみ・速さ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **Stuck Frame**（動き）：画面の効果。名前から：カード・コマ止め・コマ落とし・枠（中国語名 卡机）。つまみ：大きさ・速さ — いま：rotate_x・rotate_y・z（2.5D）・形の stroke・四角（枠は形で描ける） — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **Swing**（動き・有料）：画面の効果。名前から：揺れ動く（振り子） — いま：ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **S型运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **S运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：強さ・回転・明るさ・大きさ・ぼかし・速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **Throb**（動き）：画面の効果。名前から：跳ねる・揺れ・脈打つ（中国語名 脉搏跳动）。つまみ：明るさ・強さ・色の濃さ・範囲 — いま：animators bounce_in・spring_in・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Tracking Shot 2**（動き・有料）：画面の効果。名前から：拡大・縮小・寄る。つまみ：大きさ・速さ・回転・範囲 — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **Whiplash**（動き・有料）：画面の効果。名前から：滑る・動く。つまみ：速さ・強さ — いま：キー x・y — 借りる先：見つからない
- [ ] **White Out**（動き・有料）：画面の効果。名前から：じわっと出る・光る（フラッシュ）（中国語名 白色渐显）。つまみ：速さ — いま：キー opacity・ISF Fade・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Zoom Lens**（動き）：画面の効果。名前から：ズーム・カメラ・レンズ・魚眼（中国語名 镜头变焦）。つまみ：速さ・範囲 — いま：キー scale・ISF Zoom・ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／GPUImage3 ZoomBlur（BSD-3）
- [ ] **三图抛掷**（動き・有料）：画面の効果。名前から：投げる・3 枚。つまみ：速さ・大きさ — いま：キー x・y・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **三维旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：質感 — いま：キー rotate — 借りる先：見つからない
- [ ] **三维镜摇**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **上下摆动**（動き・有料）：画面の効果。名前から：振り子。つまみ：速さ — いま：ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **上下缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **下拉菜单**（動き・有料）：画面の効果。名前から：引く。つまみ：速さ・回転 — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **不稳定镜头**（動き・有料）：画面の効果。名前から：カメラ。つまみ：速さ・強さ・ぼかし・大きさ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **丝滑运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ・強さ・明るさ・ぼかし・ゆがみ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **两张卡片**（動き・有料）：画面の効果。名前から：カード。つまみ：ぼかし・強さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **二层空间**（動き・有料）：画面の効果。名前から：空間 — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **二次元运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **云空间垃圾**（動き・有料）：画面の効果。名前から：空間・雲。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D）・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **五官乱飞**（動き・有料）：画面の効果。名前から：飛ぶ。つまみ：速さ・大きさ・強さ・色・範囲 — いま：キー x・y — 借りる先：見つからない
- [ ] **五角旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：速さ・範囲 — いま：キー rotate — 借りる先：見つからない
- [ ] **倾斜抽卡**（動き・有料）：画面の効果。名前から：傾く・カードを引く。つまみ：速さ — いま：キー scale_x・scale_y・skew・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **光球镜头**（動き・有料）：画面の効果。名前から：カメラ・光。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：キー scale・ISF Zoom・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **光边卡片**（動き・有料）：画面の効果。名前から：カード・光。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **冻结轮播**（動き・有料）：画面の効果。名前から：めくり替え。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **切削滑动**（動き・有料）：画面の効果。名前から：滑る — いま：キー x・y — 借りる先：見つからない
- [ ] **切碎拉伸**（動き・有料）：画面の効果。名前から：伸ばす。つまみ：数 — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **动感猛震**（動き・有料）：画面の効果。名前から：揺れ。つまみ：色の濃さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **卡片抽取**（動き・有料）：画面の効果。名前から：カード。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **卡片摇摆**（動き・有料）：画面の効果。名前から：左右に揺れる・カード。つまみ：速さ — いま：ふるまい oscillator を rotate に・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **卡片运镜**（動き・有料）：画面の効果。名前から：カメラワーク・カード。つまみ：速さ — いま：キー scale・ISF Zoom・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **卡片遮挡**（動き・有料）：画面の効果。名前から：カード。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **卡片飞幕**（動き・有料）：画面の効果。名前から：カード・飛ぶ。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D）・キー x・y — 借りる先：見つからない
- [ ] **卷带滚动**（動き・有料）：画面の効果。名前から：スクロール。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **双切冲屏**（動き・有料）：画面の効果。名前から：画面に飛び込む。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **双层缩小**（動き・有料）：画面の効果。名前から：縮小。つまみ：速さ・色・強さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **双翻转**（動き・有料）：画面の効果。名前から：裏返る。つまみ：速さ・横の色ずれ — いま：rotate_x・rotate_y — 借りる先：見つからない
- [ ] **双面旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：速さ — いま：キー rotate — 借りる先：見つからない
- [ ] **反转间隙**（動き・有料）：画面の効果。名前から：回る。つまみ：速さ — いま：キー rotate — 借りる先：見つからない
- [ ] **变焦推镜**（動き）：画面の効果。名前から：ズーム・押す。つまみ：強さ・速さ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：見つからない
- [ ] **变焦烟花**（動き・有料）：画面の効果。名前から：ズーム・花火。つまみ：速さ — いま：キー scale・ISF Zoom・レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **变速拉扭**（動き・有料）：画面の効果。名前から：速さが変わる・引く。つまみ：速さ — いま：キー scale・ISF Zoom・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **向上排队**（動き・有料）：画面の効果。名前から：上へ。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **向右缩放**（動き・有料）：画面の効果。名前から：拡大縮小・右へ。つまみ：速さ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：見つからない
- [ ] **向左缩放**（動き・有料）：画面の効果。名前から：拡大縮小・左へ。つまみ：速さ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：見つからない
- [ ] **嗖滑动**（動き・有料）：画面の効果。名前から：滑る — いま：キー x・y — 借りる先：見つからない
- [ ] **四叶旋转**（動き・有料）：画面の効果。名前から：回転・葉。つまみ：速さ・範囲 — いま：キー rotate — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **固定节奏**（動き・有料）：画面の効果。名前から：リズム。つまみ：速さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **地球变焦**（動き・有料）：画面の効果。名前から：ズーム。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **复制幻灯片**（動き・有料）：画面の効果。名前から：スライドショー・複製。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D）・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **夜店弹跳**（動き・有料）：画面の効果。名前から：弾む。つまみ：ぼかし — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **对角挤压**（動き・有料）：画面の効果。名前から：押しつぶす。つまみ：速さ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **尘光飞舞**（動き・有料）：画面の効果。名前から：飛ぶ・光・ちり。つまみ：速さ — いま：キー x・y・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack（CC0）
- [ ] **局部推镜**（動き・有料）：画面の効果。名前から：押す。つまみ：速さ・強さ・明るさ・横の色ずれ・ゆがみ・横ずれ・縦ずれ — いま：キー x・y — 借りる先：見つからない
- [ ] **层叠**（動き・有料）：画面の効果。名前から：重なり。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **层层冲击**（動き・有料）：画面の効果。名前から：衝撃。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **层层飞近**（動き・有料）：画面の効果。名前から：飛ぶ。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **层次叠加**（動き・有料）：画面の効果。名前から：重ねる。つまみ：速さ — いま：キー opacity・ISF Fade — 借りる先：見つからない
- [ ] **岩石幻灯片**（動き・有料）：画面の効果。名前から：スライドショー。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **岩石节拍**（動き・有料）：画面の効果。名前から：拍子。つまみ：速さ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **左右微摇**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **左右摇晃**（動き）：画面の効果。名前から：揺れ。つまみ：強さ・ゆがみ・速さ・シャープ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **左右甩入**（動き・有料）：画面の効果。名前から：振り込む — いま：キー x・y — 借りる先：見つからない
- [ ] **布拉格**（動き・有料）：画面の効果。名前から：引く・格子。つまみ：強さ・色の濃さ — いま：キー scale_x・scale_y・skew・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **平拉放大**（動き・有料）：画面の効果。名前から：拡大・引く — いま：キー scale・ISF Zoom・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **平铺视图旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：速さ — いま：キー rotate — 借りる先：見つからない
- [ ] **幽影摇摆**（動き・有料）：画面の効果。名前から：左右に揺れる・影。つまみ：速さ・横の色ずれ — いま：ふるまい oscillator を rotate に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **底部摇晃**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **弧形丝滑滑动**（動き・有料）：画面の効果。名前から：滑る。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **弹出卡片**（動き・有料）：画面の効果。名前から：弾んで出る・カード。つまみ：速さ・範囲 — いま：animators bounce_in・spring_in・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **弹出猫咪**（動き・有料）：画面の効果。名前から：弾んで出る。つまみ：質感・大きさ — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **弹跳运镜**（動き）：画面の効果。名前から：弾む・カメラワーク — いま：animators bounce_in・spring_in・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **弹震运镜**（動き・有料）：画面の効果。名前から：カメラワーク・揺れ・弾む。つまみ：速さ・明るさ・強さ・ぼかし — いま：キー scale・ISF Zoom・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **强震黑框**（動き・有料）：画面の効果。名前から：強い揺れ・枠 — いま：ふるまい noise を x・y に・形の stroke・四角（枠は形で描ける） — 借りる先：ISF Shake（MIT）
- [ ] **彩光摇晃**（動き・有料）：画面の効果。名前から：揺れ・光。つまみ：強さ・速さ・明るさ・色・背景の動き — いま：ふるまい noise を x・y に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Shake（MIT）
- [ ] **彩斑变焦**（動き・有料）：画面の効果。名前から：ズーム。つまみ：大きさ・色・強さ・速さ・背景の動き — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **微震抖动**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ・明るさ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **心跳 II**（動き・有料）：画面の効果。名前から：鼓動。つまみ：速さ・強さ・背景の動き・明るさ・ゆがみ・横の色ずれ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **心跳之光 2**（動き・有料）：画面の効果。名前から：鼓動・光。つまみ：明るさ・色の濃さ — いま：ふるまい oscillator を scale に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **快速变焦**（動き・有料）：画面の効果。名前から：ズーム。つまみ：明るさ・色の濃さ・速さ・範囲・強さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **快速缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：速さ・強さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **快闪运镜**（動き・有料）：画面の効果。名前から：カメラワーク・光る。つまみ：速さ・明るさ・ぼかし・強さ・背景の動き・シャープ — いま：キー scale・ISF Zoom・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **恍惚扭动**（動き・有料）：画面の効果。名前から：くねる。つまみ：速さ — いま：ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **慢动作摇摆**（動き・有料）：画面の効果。名前から：左右に揺れる。つまみ：速さ — いま：ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **戏剧性旋转**（動き・有料）：画面の効果。名前から：回転。つまみ：速さ — いま：キー rotate — 借りる先：見つからない
- [ ] **手持运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **抖动微闪**（動き・有料）：画面の効果。名前から：揺れ・光る — いま：ふるまい noise を x・y に・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Shake（MIT）／ISF Strobe（MIT）
- [ ] **抖动运镜**（動き）：画面の効果。名前から：揺れ・カメラワーク — いま：ふるまい noise を x・y に・キー scale・ISF Zoom — 借りる先：ISF Shake（MIT）
- [ ] **拉伸魅力**（動き・有料）：画面の効果。名前から：伸ばす。つまみ：速さ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **拉扯震动**（動き・有料）：画面の効果。名前から：振動・引っぱる。つまみ：ゆがみ・範囲・強さ・速さ・ぼかし — いま：ふるまい noise を x・y に・キー scale_x・scale_y・skew — 借りる先：ISF Shake（MIT）
- [ ] **拉远拍摄**（動き・有料）：画面の効果。名前から：引く・打つ。つまみ：速さ — いま：キー scale・ISF Zoom・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **拍摄与堆叠**（動き・有料）：画面の効果。名前から：積み重ね・打つ。つまみ：背景の動き — いま：rotate_x・rotate_y・z（2.5D）・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **拖动放大**（動き・有料）：画面の効果。名前から：拡大。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **拼接隧道**（動き・有料）：画面の効果。名前から：トンネル・つなぎ合わせ。つまみ：速さ・回転・背景の動き・縦ずれ — いま：キー scale・ISF Zoom・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **指尖缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：大きさ・色・背景の動き — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **振动飘动**（動き・有料）：画面の効果。名前から：漂う。つまみ：速さ・強さ — いま：キー x・y — 借りる先：見つからない
- [ ] **推拉跟随**（動き・有料）：画面の効果。名前から：押し引き。つまみ：強さ・範囲・速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **推拉运镜**（動き・有料）：画面の効果。名前から：押し引き・カメラワーク。つまみ：速さ・強さ・明るさ・ゆがみ・ぼかし — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **推近推远**（動き・有料）：画面の効果。名前から：寄る・引く。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **摆动镜头**（動き・有料）：画面の効果。名前から：振り子・カメラ。つまみ：速さ — いま：ふるまい oscillator を rotate に・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **摇摆缩放**（動き・有料）：画面の効果。名前から：左右に揺れる・拡大縮小。つまみ：明るさ・シャープ — いま：ふるまい oscillator を rotate に・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **摇摇晃晃地离开**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **摇晃推镜**（動き・有料）：画面の効果。名前から：揺れ・押す。つまみ：範囲・回転・速さ — いま：ふるまい noise を x・y に・キー x・y — 借りる先：ISF Shake（MIT）
- [ ] **摇晃运镜**（動き・有料）：画面の効果。名前から：揺れ・カメラワーク。つまみ：速さ・強さ・ぼかし・明るさ・大きさ — いま：ふるまい noise を x・y に・キー scale・ISF Zoom — 借りる先：ISF Shake（MIT）
- [ ] **摇晃高光**（動き・有料）：画面の効果。名前から：揺れ・光。つまみ：速さ — いま：ふるまい noise を x・y に・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Shake（MIT）
- [ ] **斜线震动**（動き・有料）：画面の効果。名前から：振動・斜め。つまみ：大きさ・速さ・明るさ・色・強さ・背景の動き — いま：ふるまい noise を x・y に・キー scale_x・scale_y・skew — 借りる先：ISF Shake（MIT）
- [ ] **旋切冲屏**（動き・有料）：画面の効果。名前から：画面に飛び込む。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **旋转圆球**（動き・有料）：画面の効果。名前から：回転・丸。つまみ：速さ・強さ・色 — いま：キー rotate — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **旋转捕捉**（動き・有料）：画面の効果。名前から：回転。つまみ：速さ — いま：キー rotate — 借りる先：見つからない
- [ ] **旋转缩放**（動き・有料）：画面の効果。名前から：回転・拡大縮小。つまみ：速さ — いま：キー rotate・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **旋转镜头**（動き・有料）：画面の効果。名前から：回転・カメラ。つまみ：速さ — いま：キー rotate・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **无限穿越**（動き）：画面の効果。名前から：通り抜ける — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **时空隧道**（動き・有料）：画面の効果。名前から：トンネル — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **星光变焦**（動き・有料）：画面の効果。名前から：ズーム・星の光。つまみ：大きさ・数・明るさ・速さ — いま：キー scale・ISF Zoom・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星星变焦**（動き・有料）：画面の効果。名前から：ズーム・星。つまみ：速さ — いま：キー scale・ISF Zoom・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **极限缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **框架自适应摆动**（動き・有料）：画面の効果。名前から：振り子・枠。つまみ：速さ — いま：ふるまい oscillator を rotate に・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **樱桃脉冲**（動き・有料）：画面の効果。名前から：脈打つ。つまみ：速さ・数 — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **武士刀挥砍**（動き・有料）：画面の効果。名前から：振る。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **气球拉伸**（動き・有料）：画面の効果。名前から：伸ばす。つまみ：速さ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **气球飘浮**（動き・有料）：画面の効果。名前から：漂う。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **泡泡变焦**（動き・有料）：画面の効果。名前から：ズーム・泡。つまみ：速さ・強さ — いま：キー scale・ISF Zoom — 借りる先：Kenney Particle Pack（CC0）
- [ ] **流动放大**（動き・有料）：画面の効果。名前から：拡大・流れる。つまみ：速さ — いま：キー scale・ISF Zoom・ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **流动缩放**（動き・有料）：画面の効果。名前から：拡大縮小・流れる。つまみ：速さ — いま：キー scale・ISF Zoom・ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **混乱翻转**（動き・有料）：画面の効果。名前から：裏返る・乱れ。つまみ：速さ — いま：rotate_x・rotate_y・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **渐渐放大**（動き・有料）：画面の効果。名前から：拡大・じわっと。つまみ：強さ・速さ — いま：キー scale・ISF Zoom・キー opacity・ISF Fade — 借りる先：見つからない
- [ ] **滑动碰撞**（動き・有料）：画面の効果。名前から：滑る・ぶつかる。つまみ：速さ — いま：キー x・y・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **滑动载入**（動き・有料）：画面の効果。名前から：滑る。つまみ：速さ・色・大きさ・強さ・ぼかし — いま：キー x・y — 借りる先：見つからない
- [ ] **滚动缩放**（動き・有料）：画面の効果。名前から：拡大縮小・スクロール。つまみ：速さ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：見つからない
- [ ] **漂浮的云**（動き・有料）：画面の効果。名前から：浮かぶ・雲。つまみ：速さ — いま：キー x・y・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **漂浮空间**（動き・有料）：画面の効果。名前から：浮かぶ・空間。つまみ：速さ — いま：キー x・y・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **灰色震动**（動き・有料）：画面の効果。名前から：振動。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **热缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **热震**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **特写推进**（動き・有料）：画面の効果。名前から：アップ・押す。つまみ：強さ・大きさ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：見つからない
- [ ] **特写高光**（動き・有料）：画面の効果。名前から：アップ・光。つまみ：速さ — いま：キー scale・ISF Zoom・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **玫瑰冲屏**（動き・有料）：画面の効果。名前から：画面に飛び込む。つまみ：速さ・大きさ・背景の動き — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **环形抽卡**（動き・有料）：画面の効果。名前から：カードを引く。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **玻璃滑动**（動き・有料）：画面の効果。名前から：滑る・ガラス。つまみ：速さ — いま：キー x・y・ISF Frosted Glass — 借りる先：見つからない
- [ ] **疾速飞线**（動き・有料）：画面の効果。名前から：飛ぶ。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **相册 II**（動き）：画面の効果。名前から：アルバム。つまみ：強さ・ノイズ・色の濃さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **眩晕剪辑**（動き・有料）：画面の効果。名前から：めまい。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **眩晕狂欢**（動き・有料）：画面の効果。名前から：めまい。つまみ：速さ・明るさ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **移动画布**（動き・有料）：画面の効果。名前から：動く。つまみ：色の濃さ・色 — いま：キー x・y — 借りる先：見つからない
- [ ] **移动纹理**（動き・有料）：画面の効果。名前から：動く。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **移动肖像 2**（動き・有料）：画面の効果。名前から：動く。つまみ：色・明るさ・ぼかし — いま：キー x・y — 借りる先：見つからない
- [ ] **移轴镜头**（動き・有料）：画面の効果。名前から：カメラ。つまみ：範囲・ぼかし・縦ずれ・回転・色・色の濃さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **空间扰动**（動き・有料）：画面の効果。名前から：空間。つまみ：速さ・ゆがみ・明るさ・色・強さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **空间旋动**（動き・有料）：画面の効果。名前から：空間 — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **红色节奏**（動き・有料）：画面の効果。名前から：リズム・赤。つまみ：速さ・色の濃さ・背景の動き・ノイズ — いま：ふるまい oscillator を scale に・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **红色节拍**（動き・有料）：画面の効果。名前から：拍子・赤。つまみ：速さ・強さ — いま：ふるまい oscillator を scale に・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **纵向抖动**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ・ぼかし・強さ・縦の色ずれ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **纵向滑动**（動き・有料）：画面の効果。名前から：滑る。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **纵横拉扯**（動き・有料）：画面の効果。名前から：引っぱる。つまみ：速さ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **线光变速**（動き・有料）：画面の効果。名前から：速さが変わる・光。つまみ：強さ・明るさ・横ずれ・数・速さ・範囲 — いま：キー scale・ISF Zoom・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **组合卡片**（動き・有料）：画面の効果。名前から：カード。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **细节滑动**（動き・有料）：画面の効果。名前から：滑る。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **缩小翻转**（動き・有料）：画面の効果。名前から：裏返る・縮小。つまみ：色 — いま：rotate_x・rotate_y・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **缩放幻灯片**（動き・有料）：画面の効果。名前から：スライドショー・拡大縮小。つまみ：速さ・ぼかし・背景の動き — いま：rotate_x・rotate_y・z（2.5D）・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **缩放摆动**（動き・有料）：画面の効果。名前から：振り子・拡大縮小。つまみ：速さ — いま：ふるまい oscillator を rotate に・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **缩放网格**（動き・有料）：画面の効果。名前から：拡大縮小・格子。つまみ：速さ — いま：キー scale・ISF Zoom・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **缩放运镜**（動き・有料）：画面の効果。名前から：拡大縮小・カメラワーク。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **翻转变焦**（動き・有料）：画面の効果。名前から：ズーム・裏返る。つまみ：速さ・大きさ・強さ・ぼかし・シャープ・明るさ — いま：キー scale・ISF Zoom・rotate_x・rotate_y — 借りる先：見つからない
- [ ] **背景旋转**（動き）：画面の効果。名前から：回転。つまみ：範囲・ノイズ・強さ — いま：キー rotate — 借りる先：見つからない
- [ ] **背景渐隐**（動き・有料）：画面の効果。名前から：じわっと消える。つまみ：速さ — いま：キー opacity・ISF Fade — 借りる先：見つからない
- [ ] **脉冲漂移**（動き・有料）：画面の効果。名前から：脈打つ。つまみ：速さ・背景の動き・ぼかし — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **色度脉冲**（動き・有料）：画面の効果。名前から：脈打つ。つまみ：速さ・強さ・横の色ずれ — いま：ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **节奏拉扯**（動き・有料）：画面の効果。名前から：引っぱる・リズム。つまみ：速さ — いま：キー scale_x・scale_y・skew・ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **节奏游戏**（動き・有料）：画面の効果。名前から：リズム・ゲーム。つまみ：速さ — いま：ふるまい oscillator を scale に・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **节奏缩放**（動き・有料）：画面の効果。名前から：拡大縮小・リズム。つまみ：速さ — いま：キー scale・ISF Zoom・ふるまい oscillator を scale に — 借りる先：見つからない
- [ ] **落日印记**（動き・有料）：画面の効果。名前から：落ちる。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **虚焦拍摄**（動き・有料）：画面の効果。名前から：打つ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **裸眼3D**（動き）：画面の効果。名前から：3D・目 — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **视差堆叠**（動き・有料）：画面の効果。名前から：積み重ね。つまみ：大きさ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **角落夹击**（動き・有料）：画面の効果。名前から：落ちる・打つ。つまみ：速さ — いま：キー x・y・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **负向滚动**（動き・有料）：画面の効果。名前から：スクロール・ネガ。つまみ：大きさ — いま：キー x・y・ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **跟随运镜**（動き・有料）：画面の効果。名前から：カメラワーク。つまみ：範囲・横ずれ・縦ずれ・速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **跳动空间**（動き・有料）：画面の効果。名前から：跳ねる・空間。つまみ：速さ — いま：animators bounce_in・spring_in・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **跳色四宫格**（動き・有料）：画面の効果。名前から：跳ぶ・格子。つまみ：速さ — いま：animators bounce_in・spring_in・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **跷跷板滑动**（動き・有料）：画面の効果。名前から：滑る。つまみ：速さ — いま：キー x・y — 借りる先：見つからない
- [ ] **轰动摇动**（動き・有料）：画面の効果。名前から：揺れ。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **重复击打**（動き・有料）：画面の効果。名前から：打つ。つまみ：速さ — いま：ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **重复变焦**（動き）：画面の効果。名前から：ズーム。つまみ：大きさ・強さ・明るさ・質感・速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **锯齿缩放**（動き・有料）：画面の効果。名前から：拡大縮小。つまみ：速さ・強さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **镜头宠儿**（動き・有料）：画面の効果。名前から：カメラ。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **阴影节拍**（動き・有料）：画面の効果。名前から：拍子・影。つまみ：速さ — いま：ふるまい oscillator を scale に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **震动失真**（動き・有料）：画面の効果。名前から：振動・ゆがみ。つまみ：大きさ・数 — いま：ふるまい noise を x・y に・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：ISF Shake（MIT）
- [ ] **风扇旋转**（動き・有料）：画面の効果。名前から：回転・風。つまみ：強さ — いま：キー rotate・キー x・y — 借りる先：見つからない
- [ ] **飞吻**（動き・有料）：画面の効果。名前から：飛ぶ・キス。つまみ：速さ — いま：キー x・y — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **飞机窗**（動き・有料）：画面の効果。名前から：飛ぶ・窓。つまみ：速さ・大きさ・明るさ — いま：キー x・y・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない

### 粒・飾り（205）

- [ ] **321粒子倒数**（粒・飾り・有料）：画面の効果。名前から：粒子・カウントダウン。つまみ：速さ — いま：点群の Appear Ripple・Appear Rain・Appear Eruption・animators print_on・print_off — 借りる先：Kenney Particle Pack（CC0）
- [ ] **3D Love**（粒・飾り・有料）：画面の効果。名前から：ハート・立体。つまみ：速さ・縦ずれ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **3D LoveII**（粒・飾り・有料）：画面の効果。名前から：ハート・立体。つまみ：速さ・縦ずれ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **3D消散**（粒・飾り・有料）：画面の効果。名前から：消え散る・3D。つまみ：速さ・色・明るさ・ぼかし・大きさ・数 — いま：点群の dissolve・ISF Noise・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D爱心**（粒・飾り・有料）：画面の効果。名前から：ハート・3D。つまみ：速さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **3d钻石**（粒・飾り・有料）：画面の効果。名前から：ダイヤ。つまみ：縦ずれ・速さ・回転・横ずれ・色の濃さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Astral**（粒・飾り）：画面の効果。名前から：きらきら・星。つまみ：速さ・範囲 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Collision Sparks**（粒・飾り・有料）：画面の効果。名前から：火の粉・ぶつかる・火花（中国語名 星火碰撞） — いま：ふるまい noise を x・y に — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter／ISF Shake（MIT）
- [ ] **Confetti II**（粒・飾り・有料）：画面の効果。名前から：金色・紙吹雪（中国語名 金片）。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Constellation**（粒・飾り）：画面の効果。名前から：星・きらきら（中国語名 星夜）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Explosion**（粒・飾り）：画面の効果。名前から：爆発・花火（中国語名 爆炸）。つまみ：速さ・背景の動き — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **Explosion**（粒・飾り・有料）：画面の効果。名前から：はじける・煙・花火・爆発（中国語名 烟雾炸开）。つまみ：速さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **Fire Edges**（粒・飾り）：画面の効果。名前から：火の光・縁取り・炎（中国語名 火光包围）。つまみ：速さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Firefly**（粒・飾り）：画面の効果。名前から：ホタル・炎・きらきら・星（中国語名 萤火）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Fireworks 2**（粒・飾り）：画面の効果。名前から：火・花・炎・花火（中国語名 花火）。つまみ：速さ・背景の動き — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter／Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **Flame Frame**（粒・飾り）：画面の効果。名前から：炎・枠（中国語名 火焰边框 I）。つまみ：色・色の濃さ・速さ・大きさ・強さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Flames 2**（粒・飾り）：画面の効果。名前から：火の光・刷毛・炎（中国語名 火光刷过）。つまみ：速さ — いま：ISF Wipe Left・Right・Up・Down — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Flipped**（粒・飾り・有料）：画面の効果。名前から：ときめき・裏返る（中国語名 怦然心动）。つまみ：速さ — いま：rotate_x・rotate_y — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Floating Hearts**（粒・飾り・有料）：画面の効果。名前から：ハート・滑る・動く。つまみ：色の濃さ・大きさ・明るさ・色・速さ — いま：キー x・y — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Gold Dust**（粒・飾り）：画面の効果。名前から：金の粉・粒子・きらきら・星（中国語名 金粉）。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **Gold Sequins**（粒・飾り）：画面の効果。名前から：スパンコール・金色・きらきら・星（中国語名 金色亮片）。つまみ：色の濃さ・速さ・飾り — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Gold Sparkles**（粒・飾り）：画面の効果。名前から：きらきら・金の粉・火花・星（中国語名 金粉闪闪）。つまみ：速さ・色の濃さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **Heart Disco**（粒・飾り・有料）：画面の効果。名前から：ハート・虹・色（中国語名 爱心DISCO）。つまみ：速さ・背景の動き・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Heart（MIT）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **Iridescent**（粒・飾り）：画面の効果。名前から：星・色ずれ・虹・色（中国語名 星移）。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Chromatic Aberration（MIT）
- [ ] **Jackpot Spin**（粒・飾り・有料）：画面の効果。名前から：きらきら・星・回転。つまみ：速さ・明るさ — いま：ISF Star（1 個）・キー rotate — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **kirakira**（粒・飾り）：画面の効果。名前から：きらきら・星。つまみ：大きさ・数 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **KIRA泡泡**（粒・飾り・有料）：画面の効果。名前から：きらきら・泡。つまみ：明るさ・やわらかさ・大きさ・色の濃さ・速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **Magic**（粒・飾り・有料）：画面の効果。名前から：魔法・きらきら・星（中国語名 魔法）。つまみ：速さ・色の濃さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Meteor**（粒・飾り・有料）：画面の効果。名前から：火の粉・はじける・きらきら・星（中国語名 星火炸开）。つまみ：速さ・背景の動き — いま：レシピ burst.js・confetti.js・点群 Appear Eruption・ISF Star（1 個） — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Milky Way 2**（粒・飾り）：画面の効果。名前から：星・きらきら（中国語名 星河 II）。つまみ：速さ・色の濃さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Mini Stars**（粒・飾り）：画面の効果。名前から：細かいきらめき・きらきら・星（中国語名 细闪）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Mini stars II**（粒・飾り）：画面の効果。名前から：細かいきらめき・きらきら・星（中国語名 细闪 II）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Purple Mist**（粒・飾り・有料）：画面の効果。名前から：霧・紫・光のにじみ・夢っぽい（中国語名 紫雾）。つまみ：速さ・背景の動き — いま：段の fog・ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の煙（CC0）／ISF Color Phase（MIT）
- [ ] **Rain**（粒・飾り）：画面の効果。名前から：雨（中国語名 下雨）。つまみ：速さ・背景の動き — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **Snowfall**（粒・飾り）：画面の効果。名前から：雪・飛ぶ（中国語名 大雪纷飞）。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **Snowflakes**（粒・飾り）：画面の効果。名前から：雪・細かいきらめき（中国語名 雪花细闪）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Soft Rose**（粒・飾り）：画面の効果。名前から：花びら・飛ぶ・光のにじみ・夢っぽい（中国語名 花瓣飞扬）。つまみ：速さ・背景の動き — いま：キー x・y・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **Sparkle Spiral**（粒・飾り）：画面の効果。名前から：金の粉・回転・渦・ねじれ（中国語名 金粉旋转）。つまみ：速さ・色の濃さ・背景の動き — いま：ISF Star（1 個）・キー rotate — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **Star**（粒・飾り）：画面の効果。名前から：星・きらきら（中国語名 星辰 I）。つまみ：大きさ・数・色の濃さ — いま：ISF Star（1 個） — 借りる先：ISF-Files Star（MIT、名前が同じ）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Star Rain**（粒・飾り・有料）：画面の効果。名前から：星・雨・きらきら（中国語名 撒星星）。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **Star Rain 2**（粒・飾り）：画面の効果。名前から：星・雨・きらきら（中国語名 撒星星 II）。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **Starry**（粒・飾り）：画面の効果。名前から：星空・きらきら・星（中国語名 星空）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Waterfall**（粒・飾り・有料）：画面の効果。名前から：雨。つまみ：速さ・強さ・質感・ゆがみ・範囲・ぼかし — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **Wonderland**（粒・飾り）：画面の効果。名前から：雪・漂う（中国語名 飘雪 II）。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **Xmas Stars**（粒・飾り）：画面の効果。名前から：星の光・クリスマス・きらきら・星（中国語名 圣诞星光）。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **上升气泡**（粒・飾り・有料）：画面の効果。名前から：泡・上がる。つまみ：強さ — いま：キー x・y — 借りる先：Kenney Particle Pack（CC0）
- [ ] **仙女变身**（粒・飾り）：画面の効果。名前から：妖精。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **仙女变身 II**（粒・飾り・有料）：画面の効果。名前から：妖精。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **仙女棒**（粒・飾り・有料）：画面の効果。名前から：妖精。つまみ：ぼかし・背景の動き・大きさ・色の濃さ・速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **仙女棒II**（粒・飾り・有料）：画面の効果。名前から：妖精。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **仙尘闪闪**（粒・飾り）：画面の効果。名前から：きらきら・ちり。つまみ：速さ・背景の動き — いま：ISF Star（1 個）・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **假日闪闪**（粒・飾り・有料）：画面の効果。名前から：きらきら・休日。つまみ：大きさ・数・回転・色の濃さ・やわらかさ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞Kira**（粒・飾り・有料）：画面の効果。名前から：きらきら・クリスマス。つまみ：大きさ・数 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞kira**（粒・飾り・有料）：画面の効果。名前から：きらきら・クリスマス。つまみ：速さ・大きさ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞边框**（粒・飾り・有料）：画面の効果。名前から：クリスマス・枠。つまみ：速さ・縦ずれ・大きさ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **在云端**（粒・飾り・有料）：画面の効果。名前から：雲。つまみ：速さ・大きさ・縦ずれ・色・強さ — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **彩带球**（粒・飾り・有料）：画面の効果。名前から：紙テープ。つまみ：色の濃さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **彩绘星星**（粒・飾り・有料）：画面の効果。名前から：星。つまみ：色 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **彩色Kira**（粒・飾り・有料）：画面の効果。名前から：きらきら・カラー。つまみ：色の濃さ・大きさ・回転・強さ — いま：ISF Star（1 個）・ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **彩色kira II**（粒・飾り・有料）：画面の効果。名前から：きらきら・カラー。つまみ：色の濃さ・範囲・色・ぼかし・速さ — いま：ISF Star（1 個）・ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **摇曳蒸汽**（粒・飾り・有料）：画面の効果。名前から：湯気・揺れ。つまみ：速さ — いま：段の fog・ふるまい noise を x・y に — 借りる先：Kenney Particle Pack の煙（CC0）／ISF Shake（MIT）
- [ ] **新年仙女棒**（粒・飾り・有料）：画面の効果。名前から：妖精・新年。つまみ：大きさ・横ずれ・縦ずれ・範囲・色の濃さ・速さ・強さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **新年烟花**（粒・飾り・有料）：画面の効果。名前から：花火・新年。つまみ：速さ・背景の動き・強さ・明るさ・ノイズ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **新年烟花 II**（粒・飾り・有料）：画面の効果。名前から：花火・新年。つまみ：速さ・色の濃さ・横ずれ・縦ずれ・大きさ・強さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **旋转雪花**（粒・飾り・有料）：画面の効果。名前から：雪・回転。つまみ：速さ — いま：キー rotate — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **旋风花瓣**（粒・飾り・有料）：画面の効果。名前から：花びら・風。つまみ：速さ・大きさ — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **日落魔法**（粒・飾り・有料）：画面の効果。名前から：魔法・落ちる。つまみ：大きさ — いま：ISF Star（1 個）・キー x・y — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星光绽放**（粒・飾り・有料）：画面の効果。名前から：星の光。つまみ：速さ・色 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星星灯**（粒・飾り）：画面の効果。名前から：星。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星星飞舞**（粒・飾り・有料）：画面の効果。名前から：星・飛ぶ。つまみ：速さ・大きさ — いま：ISF Star（1 個）・キー x・y — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星月**（粒・飾り）：画面の効果。名前から：星。つまみ：色の濃さ・速さ・横ずれ・縦ずれ・強さ・飾り — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星辰 II**（粒・飾り）：画面の効果。名前から：星。つまみ：大きさ・数・色の濃さ・回転 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **星辰 III**（粒・飾り・有料）：画面の効果。名前から：星。つまみ：大きさ・数・色の濃さ・明るさ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **月光闪闪**（粒・飾り・有料）：画面の効果。名前から：きらきら・光。つまみ：大きさ・数・色の濃さ・横の色ずれ・縦の色ずれ — いま：ISF Star（1 個）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **枫叶飘落**（粒・飾り・有料）：画面の効果。名前から：葉・漂う・落ちる。つまみ：速さ・明るさ — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **梦境KIRA**（粒・飾り・有料）：画面の効果。名前から：きらきら・夢の中。つまみ：数・範囲・ぼかし・大きさ・強さ・色の濃さ — いま：ISF Star（1 個）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **梦幻萤火虫**（粒・飾り・有料）：画面の効果。名前から：ホタル・夢。つまみ：速さ・強さ — いま：ISF Star（1 個）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **梦幻雪花**（粒・飾り・有料）：画面の効果。名前から：雪・夢。つまみ：速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **水晶烟火**（粒・飾り・有料）：画面の効果。名前から：水晶・火・煙。つまみ：強さ — いま：ISF Star（1 個）・段の fog — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **泡泡冲屏**（粒・飾り・有料）：画面の効果。名前から：泡・画面に飛び込む。つまみ：大きさ・速さ・色の濃さ — いま：キー scale・ISF Zoom — 借りる先：Kenney Particle Pack（CC0）
- [ ] **浓雾**（粒・飾り）：画面の効果。名前から：霧。つまみ：速さ・背景の動き — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **浪漫烟火**（粒・飾り・有料）：画面の効果。名前から：火・煙。つまみ：速さ — いま：段の fog — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter／Kenney Particle Pack の煙（CC0）
- [ ] **火光翻滚**（粒・飾り・有料）：画面の効果。名前から：火の光・めくる。つまみ：速さ — いま：rotate_x・rotate_y — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **火焰边框 II**（粒・飾り・有料）：画面の効果。名前から：炎・枠。つまみ：色・色の濃さ・速さ・大きさ・強さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **火花飞扬**（粒・飾り・有料）：画面の効果。名前から：火花・飛ぶ。つまみ：速さ — いま：キー x・y — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **火花飞溅**（粒・飾り・有料）：画面の効果。名前から：火花・飛ぶ。つまみ：速さ・明るさ・やわらかさ — いま：キー x・y — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **灼烧打孔**（粒・飾り・有料）：画面の効果。名前から：焼ける。つまみ：質感・強さ・色の濃さ・速さ・横ずれ — いま：点群の Burn Away — 借りる先：ISF Film Burn・Undulating Burn Out（MIT）
- [ ] **灿金彩带**（粒・飾り・有料）：画面の効果。名前から：紙テープ・金色。つまみ：質感・速さ・明るさ・色の濃さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **烟火星河**（粒・飾り・有料）：画面の効果。名前から：火・煙・星。つまみ：速さ — いま：段の fog・ISF Star（1 個） — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter／Kenney Particle Pack の煙（CC0）
- [ ] **烟花**（粒・飾り・有料）：画面の効果。名前から：花火。つまみ：速さ・背景の動き — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **烟花 III**（粒・飾り）：画面の効果。名前から：花火。つまみ：色の濃さ・速さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **烟花2024**（粒・飾り・有料）：画面の効果。名前から：花火。つまみ：大きさ・縦ずれ・横ずれ・色の濃さ・速さ・範囲 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **烟花之心**（粒・飾り・有料）：画面の効果。名前から：花火・ハート。つまみ：速さ・強さ・色の濃さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **烟雾**（粒・飾り・有料）：画面の効果。名前から：煙。つまみ：速さ・ノイズ・色の濃さ — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **爱心Kira**（粒・飾り）：画面の効果。名前から：きらきら・ハート。つまみ：大きさ・数・回転 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心kira2**（粒・飾り・有料）：画面の効果。名前から：きらきら・ハート。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心冲屏**（粒・飾り・有料）：画面の効果。名前から：ハート・画面に飛び込む。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心投影**（粒・飾り）：画面の効果。名前から：ハート・影。つまみ：速さ・大きさ・数・色の濃さ・回転・ぼかし — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Heart（MIT）＋物の copies・scatter／ISF Ghosting（MIT）
- [ ] **爱心炸开**（粒・飾り・有料）：画面の効果。名前から：はじける・ハート。つまみ：速さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **粉雪**（粒・飾り・有料）：画面の効果。名前から：雪・ピンク。つまみ：背景の動き・明るさ・範囲・速さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity／ISF Color Phase（MIT）
- [ ] **粒子消散1**（粒・飾り）：画面の効果。名前から：消え散る・粒子 — いま：点群の dissolve・ISF Noise・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack（CC0）
- [ ] **粒子消散2**（粒・飾り）：画面の効果。名前から：消え散る・粒子 — いま：点群の dissolve・ISF Noise・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack（CC0）
- [ ] **粒子空间**（粒・飾り・有料）：画面の効果。名前から：粒子・空間。つまみ：速さ — いま：点群の Appear Ripple・Appear Rain・Appear Eruption・rotate_x・rotate_y・z（2.5D） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **紫色火花**（粒・飾り・有料）：画面の効果。名前から：火花・紫。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **细闪 III**（粒・飾り・有料）：画面の効果。名前から：細かいきらめき。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **羽毛飘落**（粒・飾り・有料）：画面の効果。名前から：羽・漂う・落ちる。つまみ：大きさ・速さ・強さ・明るさ・背景の動き・色の濃さ — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **花瓣雨**（粒・飾り・有料）：画面の効果。名前から：花びら・雨。つまみ：速さ — いま：点群 Appear Rain（点群だけ） — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **花瓣飘落**（粒・飾り）：画面の効果。名前から：花びら・漂う・落ちる。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **蒸汽腾腾**（粒・飾り）：画面の効果。名前から：湯気。つまみ：速さ・背景の動き — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **蓝调Kira**（粒・飾り）：画面の効果。名前から：きらきら・青。つまみ：数・ぼかし・縦ずれ・範囲・色の濃さ — いま：ISF Star（1 個）・ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **蝙蝠kira**（粒・飾り）：画面の効果。名前から：きらきら。つまみ：背景の動き・横の色ずれ・縦の色ずれ・大きさ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **蝴蝶Kira**（粒・飾り・有料）：画面の効果。名前から：きらきら・蝶。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **蝴蝶投影**（粒・飾り・有料）：画面の効果。名前から：蝶・影。つまみ：飾り・強さ・速さ・数・色の濃さ・横ずれ・大きさ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies／ISF Ghosting（MIT）
- [ ] **蝴蝶飞舞**（粒・飾り・有料）：画面の効果。名前から：蝶・飛ぶ。つまみ：飾り・範囲・速さ・色の濃さ — いま：キー x・y — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **贴身粒子**（粒・飾り・有料）：画面の効果。名前から：粒子。つまみ：数・速さ・大きさ・色・ゆがみ — いま：点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack（CC0）
- [ ] **迷雾**（粒・飾り）：画面の効果。名前から：霧。つまみ：速さ・背景の動き — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **迷雾消散**（粒・飾り・有料）：画面の効果。名前から：消え散る・霧。つまみ：強さ・範囲・色の濃さ・速さ・回転 — いま：点群の dissolve・ISF Noise・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **金片 II**（粒・飾り）：画面の効果。名前から：金色。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金粉 III**（粒・飾り・有料）：画面の効果。名前から：金の粉。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金粉飘落**（粒・飾り・有料）：画面の効果。名前から：金の粉・漂う・落ちる。つまみ：速さ・色の濃さ・強さ・大きさ — いま：ISF Star（1 個）・キー x・y — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金色大爆炸**（粒・飾り・有料）：画面の効果。名前から：爆発・金色。つまみ：速さ・ぼかし・背景の動き — いま：レシピ burst.js・confetti.js・点群 Appear Eruption・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金色的雪**（粒・飾り・有料）：画面の効果。名前から：雪・金色。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金色雪花**（粒・飾り・有料）：画面の効果。名前から：雪・金色。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金色魔法边框**（粒・飾り・有料）：画面の効果。名前から：魔法・枠・金色 — いま：ISF Star（1 個）・形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪亮小星星**（粒・飾り・有料）：画面の効果。名前から：星・きらめく。つまみ：速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪亮星尘**（粒・飾り・有料）：画面の効果。名前から：きらめく・星・ちり。つまみ：速さ — いま：ISF Star（1 個）・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **闪亮登场**（粒・飾り・有料）：画面の効果。名前から：きらめく。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪亮的粉彩**（粒・飾り・有料）：画面の効果。名前から：きらめく・ピンク。つまみ：速さ — いま：ISF Star（1 個）・ISF Multi Hue Shift・形の gradient — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Color Phase（MIT）
- [ ] **闪亮裂纹**（粒・飾り・有料）：画面の効果。名前から：きらめく・ひび。つまみ：速さ・色の濃さ・明るさ・やわらかさ・大きさ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪亮飞机**（粒・飾り・有料）：画面の効果。名前から：きらめく・飛ぶ。つまみ：速さ — いま：ISF Star（1 個）・キー x・y — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪闪蝴蝶**（粒・飾り・有料）：画面の効果。名前から：きらきら・蝶。つまみ：大きさ・数・色の濃さ・強さ・速さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **雨天**（粒・飾り・有料）：画面の効果。名前から：雨。つまみ：速さ — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **雨季 I**（粒・飾り・有料）：画面の効果。名前から：雨。つまみ：ぼかし・色の濃さ・背景の動き・速さ — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **雨季 II**（粒・飾り・有料）：画面の効果。名前から：雨。つまみ：ぼかし・色の濃さ・背景の動き・速さ — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **雨滴**（粒・飾り・有料）：画面の効果。名前から：雨。つまみ：大きさ・質感・ぼかし・速さ・ノイズ・強さ・範囲 — いま：点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **雨滴晕开**（粒・飾り・有料）：画面の効果。名前から：にじむ・雨。つまみ：速さ・背景の動き — いま：点群の dissolve・ISF Noise・点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **雪花冲屏**（粒・飾り・有料）：画面の効果。名前から：雪・画面に飛び込む。つまみ：速さ — いま：キー scale・ISF Zoom — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **雪花散落**（粒・飾り・有料）：画面の効果。名前から：雪・落ちる。つまみ：速さ — いま：キー x・y — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **雪花闪闪**（粒・飾り・有料）：画面の効果。名前から：雪・きらきら。つまみ：大きさ・数・強さ・色の濃さ・速さ・色 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **雪雾**（粒・飾り・有料）：画面の効果。名前から：霧・雪。つまみ：飾り・色の濃さ・質感・背景の動き — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **雾气**（粒・飾り・有料）：画面の効果。名前から：霧。つまみ：速さ・背景の動き — いま：段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **雾窗**（粒・飾り・有料）：画面の効果。名前から：霧・窓。つまみ：背景の動き — いま：段の fog・形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **飘落花瓣 II**（粒・飾り・有料）：画面の効果。名前から：花びら・漂う・落ちる。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **飘雪**（粒・飾り）：画面の効果。名前から：雪・漂う。つまみ：速さ・背景の動き — いま：キー x・y — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **飞舞花瓣**（粒・飾り・有料）：画面の効果。名前から：花びら・飛ぶ。つまみ：速さ — いま：キー x・y — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **马尾烟花**（粒・飾り・有料）：画面の効果。名前から：花火。つまみ：大きさ・横ずれ・縦ずれ・強さ・明るさ・色の濃さ・速さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：見つからない
- [ ] **魔法变身**（粒・飾り）：画面の効果。名前から：魔法。つまみ：速さ・背景の動き — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **魔法甜心**（粒・飾り）：画面の効果。名前から：魔法・ハート。つまみ：縦ずれ・横ずれ・速さ・回転・色の濃さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter／ISF Heart（MIT）＋物の copies・scatter
- [ ] **魔法边框 II**（粒・飾り・有料）：画面の効果。名前から：魔法・枠 — いま：ISF Star（1 個）・形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **1998**（粒・飾り）：画面の効果。名前から：季節の行事。つまみ：色の濃さ・質感 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **Butterfly**（粒・飾り・有料）：画面の効果。名前から：蝶・翼。つまみ：大きさ・速さ・背景の動き — いま：無い — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **By the Fireplace**（粒・飾り・有料）：画面の効果。名前から：火の光・炎（中国語名 壁炉火光）。つまみ：速さ・飾り・色の濃さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Feathers**（粒・飾り）：画面の効果。名前から：羽・花びら（中国語名 羽毛）。つまみ：速さ・背景の動き — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **Fire**（粒・飾り）：画面の効果。名前から：火の光・炎（中国語名 火光）。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Flames**（粒・飾り）：画面の効果。名前から：火の光・炎（中国語名 火光蔓延）。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Girl's Secrets**（粒・飾り・有料）：画面の効果。名前から：ハート（中国語名 少女心事）。つまみ：速さ・背景の動き・色の濃さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **Heart Kisses**（粒・飾り・有料）：画面の効果。名前から：ハート（中国語名 爱心啵啵）。つまみ：速さ・背景の動き — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **New Year**（粒・飾り）：画面の効果。名前から：季節の行事。つまみ：大きさ・速さ・色・色の濃さ・背景の動き — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **Night Snow**（粒・飾り・有料）：画面の効果。名前から：雪。つまみ：背景の動き・強さ・色の濃さ・色・明るさ・飾り — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **Spark**（粒・飾り）：画面の効果。名前から：火の粉・火花（中国語名 星火）。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **Spark 2**（粒・飾り）：画面の効果。名前から：火の粉・火花（中国語名 星火 II）。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **Super Like**（粒・飾り）：画面の効果。名前から：ハート（中国語名 超级喜欢）。つまみ：縦ずれ・大きさ・速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **万圣夜**（粒・飾り・有料）：画面の効果。名前から：ハロウィン。つまみ：ぼかし・速さ・色の濃さ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **万圣节快乐**（粒・飾り）：画面の効果。名前から：ハロウィン。つまみ：速さ・背景の動き — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **南瓜笑脸**（粒・飾り）：画面の効果。名前から：かぼちゃ・顔。つまみ：速さ・背景の動き — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **吻标记**（粒・飾り・有料）：画面の効果。名前から：キス。つまみ：明るさ・やわらかさ・強さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **圣诞开幕**（粒・飾り・有料）：画面の効果。名前から：クリスマス・幕が開く。つまみ：速さ・強さ・大きさ・背景の動き — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **圣诞彩球**（粒・飾り・有料）：画面の効果。名前から：クリスマス。つまみ：速さ・大きさ・数 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞日记**（粒・飾り）：画面の効果。名前から：クリスマス。つまみ：色・大きさ・色の濃さ・速さ・強さ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞明信片**（粒・飾り・有料）：画面の効果。名前から：クリスマス。つまみ：速さ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞树**（粒・飾り）：画面の効果。名前から：クリスマス。つまみ：速さ・背景の動き・大きさ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **圣诞派对**（粒・飾り・有料）：画面の効果。名前から：クリスマス・パーティー。つまみ：大きさ・色の濃さ・強さ・範囲・回転 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **塑料花**（粒・飾り・有料）：画面の効果。名前から：花。つまみ：速さ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **多彩爱心**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **大雪**（粒・飾り）：画面の効果。名前から：雪。つまみ：速さ・背景の動き — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **小圣诞**（粒・飾り）：画面の効果。名前から：クリスマス。つまみ：速さ・背景の動き・色の濃さ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **小心心表情符号**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：大きさ・横ずれ・縦ずれ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **心形气泡**（粒・飾り・有料）：画面の効果。名前から：ハート・泡。つまみ：大きさ・縦ずれ・横ずれ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **心灵萌芽**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ・明るさ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **心磁铁**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **心花怒放II**（粒・飾り・有料）：画面の効果。名前から：ハート・花。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter／Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **星火消逝**（粒・飾り・有料）：画面の効果。名前から：火の粉。つまみ：強さ — いま：無い — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **泡泡火花**（粒・飾り・有料）：画面の効果。名前から：火花・泡。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter／Kenney Particle Pack（CC0）
- [ ] **火热展示**（粒・飾り・有料）：画面の効果。名前から：火。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **火焰爆发**（粒・飾り・有料）：画面の効果。名前から：炎。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **火焰绽放**（粒・飾り・有料）：画面の効果。名前から：炎。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **点点心形**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **点赞**（粒・飾り）：画面の効果。名前から：いいね — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **点赞发布**（粒・飾り・有料）：画面の効果。名前から：いいね。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **热恋**（粒・飾り・有料）：画面の効果。名前から：熱愛。つまみ：大きさ・数・回転・ぼかし・範囲・シャープ・色の濃さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心QQ糖**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ・背景の動き — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心蝴蝶**（粒・飾り）：画面の効果。名前から：ハート・蝶。つまみ：縦ずれ・横ずれ・速さ・回転・色の濃さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **爱心软糖**（粒・飾り・有料）：画面の効果。名前から：ハート。つまみ：速さ・色の濃さ・強さ・背景の動き — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **狂欢节羽毛**（粒・飾り・有料）：画面の効果。名前から：羽。つまみ：速さ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **生日快乐**（粒・飾り）：画面の効果。名前から：誕生日。つまみ：速さ — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **电子蝴蝶 I**（粒・飾り・有料）：画面の効果。名前から：蝶。つまみ：速さ・大きさ・強さ・色の濃さ・範囲・質感 — いま：無い — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **电子蝴蝶 II**（粒・飾り・有料）：画面の効果。名前から：蝶。つまみ：速さ・強さ・色の濃さ・範囲 — いま：無い — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **真爱至上**（粒・飾り・有料）：画面の効果。名前から：愛。つまみ：速さ — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **繁花之镜 II**（粒・飾り・有料）：画面の効果。名前から：花。つまみ：背景の動き・色の濃さ・強さ・範囲・明るさ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **花卉邮票**（粒・飾り・有料）：画面の効果。名前から：花。つまみ：大きさ・強さ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **花瓣扑**（粒・飾り・有料）：画面の効果。名前から：花びら。つまみ：速さ・明るさ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **迷醉蝴蝶**（粒・飾り・有料）：画面の効果。名前から：蝶。つまみ：色の濃さ・数・飾り・横ずれ・縦ずれ — いま：無い — 借りる先：Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **雪仙子**（粒・飾り・有料）：画面の効果。名前から：雪。つまみ：速さ — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **黑羽毛**（粒・飾り・有料）：画面の効果。名前から：羽。つまみ：速さ・背景の動き — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **黑羽毛 II**（粒・飾り）：画面の効果。名前から：羽。つまみ：速さ・背景の動き — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter

### レトロ（140）

- [ ] **70's**（レトロ）：画面の効果。名前から：古いフィルム・レトロ。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **90s**（レトロ・有料）：画面の効果。名前から：古いフィルム・レトロ。つまみ：シャープ・ぼかし・横の色ずれ・縦の色ずれ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **B&W VHS**（レトロ）：画面の効果。名前から：VHS・白黒・古いビデオ・テレビ（中国語名 黑白VHS）。つまみ：強さ・色の濃さ・速さ・横の色ずれ・縦の色ずれ — いま：crt_finish.wgsl・ISF Bad TV・ISF Color Monochrome — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Bad TV**（レトロ）：画面の効果。名前から：古いテレビ・カクつき・古いビデオ・テレビ（中国語名 老电视卡顿）。つまみ：強さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF-Files Bad TV（MIT、名前が同じ）／ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF Freeze Frame・Random Freeze（MIT）
- [ ] **Betamax**（レトロ）：画面の効果。名前から：古いビデオ・テレビ。つまみ：色の濃さ・ぼかし・シャープ・ゆがみ・質感・大きさ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Black Noise**（レトロ）：画面の効果。名前から：ノイズ・粒（中国語名 黑色噪点）。つまみ：速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **Blu-ray Scanning**（レトロ）：画面の効果。名前から：古いビデオ・テレビ・光の筋・光が走る。つまみ：明るさ・ぼかし・強さ・速さ・範囲 — いま：crt_finish.wgsl・ISF Bad TV・ISF God Rays — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Camcorder**（レトロ・有料）：画面の効果。名前から：ビデオ・古いビデオ・テレビ（中国語名 录像机） — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **CCD**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：強さ・色の濃さ・飾り・横ずれ・縦ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **CD**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：速さ・大きさ・ぼかし・縦ずれ・横ずれ・色の濃さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Cinema**（レトロ）：画面の効果。名前から：映画・古いフィルム・レトロ（中国語名 电影感） — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **CTR屏幕**（レトロ・有料）：画面の効果。名前から：画面。つまみ：色の濃さ・大きさ・ノイズ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Develop**（レトロ・有料）：画面の効果。名前から：フィルム・影・古いフィルム・レトロ（中国語名 胶片显影）。つまみ：速さ・背景の動き・やわらかさ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **DVD**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：大きさ・色の濃さ・範囲・明るさ・速さ・ノイズ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **DVD光盘框**（レトロ・有料）：画面の効果。名前から：DV カメラ・枠・光。つまみ：色の濃さ・ノイズ・ぼかし・大きさ・縦ずれ・横ずれ — いま：crt_finish.wgsl・ISF Bad TV・形の stroke・四角（枠は形で描ける） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **DV录制框**（レトロ・有料）：画面の効果。名前から：DV カメラ・枠。つまみ：速さ・ぼかし・色の濃さ・シャープ — いま：crt_finish.wgsl・ISF Bad TV・形の stroke・四角（枠は形で描ける） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **DV纹理**（レトロ）：画面の効果。名前から：DV カメラ。つまみ：ノイズ・質感 — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Film 2**（レトロ）：画面の効果。名前から：古いフィルム・レトロ。つまみ：横の色ずれ・縦の色ずれ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：three.js shaders FilmShader（MIT、名前が同じ）
- [ ] **Jitter**（レトロ）：画面の効果。名前から：映写機・揺れ・グリッチ（中国語名 放映机抖动）。つまみ：範囲 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **JVC**（レトロ）：画面の効果。名前から：古いビデオ・テレビ。つまみ：色・強さ・色の濃さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Leak 1**（レトロ）：画面の効果。名前から：フィルム・光漏れ（中国語名 胶片漏光）。つまみ：速さ・背景の動き — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **Leak 2**（レトロ）：画面の効果。名前から：フィルム・光漏れ（中国語名 胶片漏光 II）。つまみ：速さ・背景の動き — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **Light leak**（レトロ・有料）：画面の効果。名前から：ノイズ・光漏れ（中国語名 漏光噪点）。つまみ：質感 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **Noise**（レトロ）：画面の効果。名前から：ノイズ・粒（中国語名 噪点）。つまみ：ノイズ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF-Files Noise（MIT、名前が同じ）
- [ ] **Noise 1**（レトロ）：画面の効果。名前から：ノイズ・粒（中国語名 荧幕噪点）。つまみ：速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF-Files Noise（MIT、名前が同じ）
- [ ] **Noise 2**（レトロ）：画面の効果。名前から：ノイズ・粒（中国語名 荧幕噪点 II）。つまみ：速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF-Files Noise（MIT、名前が同じ）
- [ ] **Old Footage**（レトロ・有料）：画面の効果。名前から：古いフィルム・レトロ。つまみ：速さ・色の濃さ・質感・背景の動き・強さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Old TV 2**（レトロ）：画面の効果。名前から：古い映画・古いビデオ・テレビ・古いフィルム（中国語名 老电影 II）。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Player 3**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：速さ・明るさ・大きさ・縦の色ずれ・回転・質感・横ずれ・縦ずれ・横の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **PS2游戏**（レトロ・有料）：画面の効果。名前から：ゲーム。つまみ：大きさ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Retro Cam**（レトロ・有料）：画面の効果。名前から：ビデオテープ・レトロ・古いビデオ・テレビ（中国語名 复古录像带 III） — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Retro Film**（レトロ・有料）：画面の効果。名前から：フィルム・レトロ・古いフィルム（中国語名 复古胶片）。つまみ：速さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Reversal Film**（レトロ・有料）：画面の効果。名前から：リバーサルフィルム・古いフィルム・レトロ（中国語名 反转片 I）。つまみ：色の濃さ・強さ・色・質感 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Rolling Film**（レトロ・有料）：画面の効果。名前から：フィルム・スクロール・古いフィルム・レトロ（中国語名 滚动胶片）。つまみ：色の濃さ・速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・キー x・y — 借りる先：見つからない
- [ ] **Screen Pulse**（レトロ）：画面の効果。名前から：画面・リズム・分割画面・脈打つ（中国語名 屏幕律动）。つまみ：速さ・強さ・大きさ — いま：crt_finish.wgsl・ISF Bad TV・ふるまい oscillator を scale に — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Spooky Camera**（レトロ・有料）：画面の効果。名前から：監視カメラ・光・緑・ホラー（中国語名 绿光监控）。つまみ：色の濃さ・背景の動き・シャープ・質感・飾り — いま：crt_finish.wgsl・ISF Bad TV・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF Color Phase（MIT）
- [ ] **TV Colored Lines**（レトロ・有料）：画面の効果。名前から：テレビ・虹・古いビデオ・色（中国語名 电视彩虹屏）。つまみ：速さ・質感 — いま：crt_finish.wgsl・ISF Bad TV・ISF Multi Hue Shift・形の gradient — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF Color Phase（MIT）
- [ ] **TV Girl**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **TV Lines**（レトロ）：画面の効果。名前から：テレビ・古いビデオ・帯に割れる（中国語名 电视纹理）。つまみ：質感・範囲・ゆがみ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF Sliding Strips（MIT）
- [ ] **TV Off**（レトロ）：画面の効果。名前から：テレビ・古いビデオ（中国語名 电视关机）。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **TV On**（レトロ）：画面の効果。名前から：テレビ・古いビデオ（中国語名 电视开机）。つまみ：強さ・速さ・横の色ずれ・縦の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **VCR**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：速さ・シャープ・色の濃さ・横の色ずれ・縦の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Vintage Flash**（レトロ・有料）：画面の効果。名前から：古いフィルム・レトロ・光る（フラッシュ）。つまみ：質感・色の濃さ・背景の動き・ゆがみ・速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **VX1000**（レトロ・有料）：画面の効果。名前から：古いビデオ・テレビ。つまみ：横の色ずれ・色の濃さ・シャープ・ゆがみ・ぼかし — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **单色电影**（レトロ・有料）：画面の効果。名前から：映画・単色。つまみ：範囲 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Color Monochrome — 借りる先：見つからない
- [ ] **单色胶片**（レトロ・有料）：画面の効果。名前から：フィルム・単色。つまみ：質感 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Color Monochrome — 借りる先：見つからない
- [ ] **反转片 II**（レトロ・有料）：画面の効果。名前から：リバーサルフィルム。つまみ：色の濃さ・色・範囲・質感・ノイズ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **发光显示器**（レトロ・有料）：画面の効果。名前から：モニター・光る。つまみ：速さ・大きさ・色の濃さ・明るさ — いま：crt_finish.wgsl・ISF Bad TV・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **哈苏胶片**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：色の濃さ・ノイズ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **噪点屏闪**（レトロ・有料）：画面の効果。名前から：ノイズ・画面がちらつく。つまみ：速さ・色の濃さ・ノイズ・強さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **噪片映射**（レトロ）：画面の効果。名前から：ノイズ。つまみ：ノイズ・大きさ・色の濃さ・色・強さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **噪音闪震**（レトロ・有料）：画面の効果。名前から：ノイズ・光る・揺れ。つまみ：速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）／ISF Shake（MIT）
- [ ] **坏掉的电视 2**（レトロ・有料）：画面の効果。名前から：テレビ。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **堆叠屏幕**（レトロ・有料）：画面の効果。名前から：画面・積み重ね。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古 DV**（レトロ）：画面の効果。名前から：DV カメラ・レトロ。つまみ：ぼかし・速さ・色の濃さ・シャープ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古 DV II**（レトロ・有料）：画面の効果。名前から：DV カメラ・レトロ。つまみ：ぼかし・速さ・色の濃さ・シャープ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古 DV III**（レトロ・有料）：画面の効果。名前から：DV カメラ・レトロ。つまみ：速さ・強さ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古 DV IV**（レトロ・有料）：画面の効果。名前から：DV カメラ・レトロ。つまみ：速さ・ぼかし・色の濃さ・強さ・横の色ずれ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古光盘**（レトロ・有料）：画面の効果。名前から：レトロ・光。つまみ：強さ・シャープ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **复古发光**（レトロ）：画面の効果。名前から：レトロ・光る。つまみ：強さ・色の濃さ・範囲 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **复古多屏**（レトロ・有料）：画面の効果。名前から：レトロ・多分割画面。つまみ：速さ・ノイズ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **复古多格**（レトロ・有料）：画面の効果。名前から：レトロ・格子。つまみ：色の濃さ・質感・シャープ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **复古彩虹**（レトロ・有料）：画面の効果。名前から：レトロ・虹。つまみ：大きさ・速さ・色の濃さ・質感 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **复古手机**（レトロ・有料）：画面の効果。名前から：レトロ・スマホ。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **复古投影仪**（レトロ・有料）：画面の効果。名前から：レトロ・影。つまみ：色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **复古撕纸**（レトロ・有料）：画面の効果。名前から：レトロ・紙をちぎる — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **复古旧电视**（レトロ・有料）：画面の効果。名前から：テレビ・レトロ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古星光**（レトロ・有料）：画面の効果。名前から：レトロ・星の光。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **复古海报II**（レトロ・有料）：画面の効果。名前から：レトロ・ポスター。つまみ：質感・色 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Posterize — 借りる先：見つからない
- [ ] **复古海报Ⅰ**（レトロ・有料）：画面の効果。名前から：レトロ・ポスター。つまみ：質感・色 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Posterize — 借りる先：見つからない
- [ ] **复古游戏**（レトロ・有料）：画面の効果。名前から：レトロ・ゲーム。つまみ：色の濃さ・背景の動き・色・強さ・質感・明るさ・ノイズ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古漫画**（レトロ・有料）：画面の効果。名前から：レトロ・漫画。つまみ：色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **复古火花**（レトロ・有料）：画面の効果。名前から：レトロ・火花。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **复古甜心**（レトロ・有料）：画面の効果。名前から：レトロ・ハート。つまみ：速さ・背景の動き — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **复古电视**（レトロ・有料）：画面の効果。名前から：テレビ・レトロ。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古电视机**（レトロ・有料）：画面の効果。名前から：テレビ・レトロ。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **复古碎钻**（レトロ・有料）：画面の効果。名前から：レトロ。つまみ：大きさ・数・色の濃さ・ノイズ・回転 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **复古紫调**（レトロ）：画面の効果。名前から：レトロ・紫。つまみ：質感・明るさ・シャープ・強さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **复古红调**（レトロ・有料）：画面の効果。名前から：レトロ・赤。つまみ：大きさ・数・色の濃さ・ぼかし・シャープ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **复古蓝光**（レトロ・有料）：画面の効果。名前から：レトロ・光・青。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Color Phase（MIT）
- [ ] **复古蓝调**（レトロ）：画面の効果。名前から：レトロ・青。つまみ：シャープ・ぼかし・明るさ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **复古计算机**（レトロ・有料）：画面の効果。名前から：レトロ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **复古负片效果**（レトロ・有料）：画面の効果。名前から：レトロ・ネガ。つまみ：速さ・質感・色 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **复古辉光**（レトロ・有料）：画面の効果。名前から：レトロ・光のにじみ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **复古边框**（レトロ・有料）：画面の効果。名前から：レトロ・枠。つまみ：色の濃さ・大きさ・横ずれ・縦ずれ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **复古连拍**（レトロ・有料）：画面の効果。名前から：レトロ・打つ。つまみ：速さ・色の濃さ・ノイズ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **复古闪切**（レトロ・有料）：画面の効果。名前から：レトロ・光って切り替わる。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **安全摄像**（レトロ・有料）：画面の効果。名前から：防犯カメラ。つまみ：縦ずれ・横ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **屏幕暴雪**（レトロ・有料）：画面の効果。名前から：画面・雪。つまみ：速さ・強さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **屏幕裂纹**（レトロ・有料）：画面の効果。名前から：画面・ひび。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **屏幕通知**（レトロ・有料）：画面の効果。名前から：画面。つまみ：強さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **录像带**（レトロ・有料）：画面の効果。名前から：ビデオテープ。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **录像带 II**（レトロ・有料）：画面の効果。名前から：ビデオテープ。つまみ：ノイズ・質感 — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **录像带恐慌**（レトロ・有料）：画面の効果。名前から：ビデオテープ。つまみ：ぼかし・強さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **录像机边框**（レトロ・有料）：画面の効果。名前から：ビデオ・枠。つまみ：ノイズ — いま：crt_finish.wgsl・ISF Bad TV・形の stroke・四角（枠は形で描ける） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **彩色胶片**（レトロ・有料）：画面の効果。名前から：フィルム・カラー。つまみ：色・大きさ・速さ・強さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色胶片 I**（レトロ・有料）：画面の効果。名前から：フィルム・カラー。つまみ：強さ・速さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **怀旧浪漫喜剧**（レトロ・有料）：画面の効果。名前から：懐かしい。つまみ：強さ・色 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **怀旧边框**（レトロ・有料）：画面の効果。名前から：懐かしい・枠。つまみ：色の濃さ・横の色ずれ・縦の色ずれ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **损坏的 VHS**（レトロ・有料）：画面の効果。名前から：VHS。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **摄像机 II**（レトロ・有料）：画面の効果。名前から：ビデオカメラ。つまみ：色の濃さ・強さ・ぼかし・シャープ・質感・横の色ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **放映机**（レトロ・有料）：画面の効果。名前から：映写機。つまみ：速さ・色の濃さ・強さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **故障录像带**（レトロ・有料）：画面の効果。名前から：ビデオテープ・グリッチ。つまみ：速さ・強さ — いま：crt_finish.wgsl・ISF Bad TV・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **星光CCD**（レトロ・有料）：画面の効果。名前から：CCD カメラ・星の光。つまみ：大きさ・数・回転・色の濃さ・明るさ・ノイズ・シャープ — いま：crt_finish.wgsl・ISF Bad TV・ISF Star（1 個） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **柔彩胶卷**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：色の濃さ・強さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **桌面摄像头**（レトロ・有料）：画面の効果。名前から：ビデオカメラ・頭。つまみ：横ずれ・縦ずれ・大きさ・背景の動き — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **浓郁彩噪**（レトロ・有料）：画面の効果。名前から：ノイズ。つまみ：色の濃さ・ノイズ・大きさ・強さ・速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **海鸥DC**（レトロ・有料）：画面の効果。名前から：デジカメ。つまみ：色の濃さ・シャープ・質感・ぼかし — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **深红噪点**（レトロ・有料）：画面の効果。名前から：ノイズ・赤。つまみ：速さ・強さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **炙热电影**（レトロ・有料）：画面の効果。名前から：映画。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **电子复古粉**（レトロ・有料）：画面の効果。名前から：レトロ・ピンク。つまみ：速さ・背景の動き — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **电视商店**（レトロ・有料）：画面の効果。名前から：テレビ。つまみ：大きさ・縦ずれ・横ずれ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **电视边框**（レトロ・有料）：画面の効果。名前から：テレビ・枠。つまみ：色の濃さ・質感 — いま：crt_finish.wgsl・ISF Bad TV・形の stroke・四角（枠は形で描ける） — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **电视雪花**（レトロ・有料）：画面の効果。名前から：テレビ・雪。つまみ：ノイズ・シャープ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **电视频率**（レトロ・有料）：画面の効果。名前から：テレビ。つまみ：速さ・背景の動き — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **监控现场**（レトロ・有料）：画面の効果。名前から：監視カメラ。つまみ：強さ・大きさ・ノイズ・範囲・色の濃さ・横の色ずれ・色・質感・速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **美式胶片**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：強さ・シャープ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **老式DV**（レトロ・有料）：画面の効果。名前から：DV カメラ。つまみ：色の濃さ・シャープ・ノイズ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **老式游戏机**（レトロ・有料）：画面の効果。名前から：ゲーム機。つまみ：速さ・大きさ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **老纪录片**（レトロ・有料）：画面の効果。名前から：記録映画。つまみ：速さ・強さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **胶片**（レトロ）：画面の効果。名前から：フィルム。つまみ：ノイズ・速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **胶片 I**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：速さ・色の濃さ・横ずれ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **胶片 II**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：速さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **胶片孔**（レトロ・有料）：画面の効果。名前から：フィルム。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **胶片漏光III**（レトロ・有料）：画面の効果。名前から：フィルム・光漏れ。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：ISF v002 Light Leak（MIT）
- [ ] **胶片燃烧**（レトロ・有料）：画面の効果。名前から：フィルム・燃える。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・点群の Burn Away — 借りる先：ISF Film Burn・Undulating Burn Out（MIT）
- [ ] **胶片翻转**（レトロ・有料）：画面の効果。名前から：フィルム・裏返る。つまみ：速さ・強さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・rotate_x・rotate_y — 借りる先：見つからない
- [ ] **胶片边框**（レトロ・有料）：画面の効果。名前から：フィルム・枠 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **胶片连拍**（レトロ・有料）：画面の効果。名前から：フィルム・打つ。つまみ：速さ・色の濃さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **胶片闪切**（レトロ・有料）：画面の効果。名前から：フィルム・光って切り替わる。つまみ：速さ・強さ・質感 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **苍白噪音**（レトロ・有料）：画面の効果。名前から：ノイズ。つまみ：シャープ・強さ・数 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **街机格斗**（レトロ・有料）：画面の効果。名前から：アーケードゲーム・格子。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV・ISF Random Squares Mask・Triangle Mosaic — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **被黑入的显示器**（レトロ・有料）：画面の効果。名前から：モニター。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **闪粉胶片**（レトロ・有料）：画面の効果。名前から：フィルム・光る・ピンク。つまみ：速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）／ISF Color Phase（MIT）
- [ ] **颗粒爆闪**（レトロ・有料）：画面の効果。名前から：粒・強く光る — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **颗粒特效**（レトロ・有料）：画面の効果。名前から：粒。つまみ：速さ・質感 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **颗粒金黄色**（レトロ・有料）：画面の効果。名前から：粒・金色。つまみ：速さ・ノイズ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **颗粒雾化**（レトロ・有料）：画面の効果。名前から：粒・霧。つまみ：速さ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **黄边噪片**（レトロ）：画面の効果。名前から：ノイズ。つまみ：強さ・範囲・色の濃さ・大きさ・数・質感・色 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **黑白胶片**（レトロ）：画面の効果。名前から：フィルム・白黒。つまみ：色の濃さ・速さ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl・ISF Color Monochrome — 借りる先：見つからない

### レンズ・ゆがみ（127）

- [ ] **AD Lens**（レンズ・ゆがみ・有料）：画面の効果。名前から：レンズ・魚眼。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Binoculars**（レンズ・ゆがみ・有料）：画面の効果。名前から：望遠鏡・レンズ・魚眼（中国語名 望远镜）。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Chromatic**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・色ずれ（中国語名 波纹色差）。つまみ：速さ・強さ・横の色ずれ — いま：ISF Ripples・Circle Splash Distortion・rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **Cold Lab**（レンズ・ゆがみ）：画面の効果。名前から：氷（中国語名 冰冷实验室）。つまみ：範囲・強さ・色の濃さ・速さ — いま：ISF Frosted Glass — 借りる先：見つからない
- [ ] **Diamond Zoom**（レンズ・ゆがみ）：画面の効果。名前から：変形・きらきら・星・拡大（中国語名 菱形变焦）。つまみ：大きさ・強さ・速さ・横ずれ・回転 — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Distorted**（レンズ・ゆがみ・有料）：画面の効果。名前から：変形・ゆがみ（中国語名 变形了） — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **Fisheye**（レンズ・ゆがみ）：画面の効果。名前から：魚眼・レンズ（中国語名 鱼眼）。つまみ：ゆがみ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Fisheye 2**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼・レンズ（中国語名 鱼眼 II）。つまみ：速さ・色の濃さ・範囲・大きさ・ゆがみ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Fisheye 4**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼・レンズ（中国語名 鱼眼IV）。つまみ：強さ・色の濃さ・大きさ・質感 — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Fisheye Shine**（レンズ・ゆがみ・有料）：画面の効果。名前から：レンズ・魚眼・きらきら・星。つまみ：速さ — いま：ISF Bump Distortion・ISF Star（1 個） — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Fisheye Spark**（レンズ・ゆがみ・有料）：画面の効果。名前から：レンズ・魚眼・火花。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／Kenney Particle Pack の火花（CC0）＋物の copies・scatter
- [ ] **Gentle Ripples**（レンズ・ゆがみ）：画面の効果。名前から：波紋・カクつき・ちぎれる（中国語名 波纹卡顿）。つまみ：ゆがみ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **Heart Magnifier**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね・ぼかし・ハート・レンズ（中国語名 模糊爱心放大镜）。つまみ：大きさ・強さ・色・縦ずれ・横ずれ・ぼかし — いま：ISF Bump Distortion・物の blur・ISF Fast Blur — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／ISF Heart（MIT）＋物の copies・scatter
- [ ] **ins边框放大镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね・枠。つまみ：大きさ・強さ・縦ずれ・横ずれ・ぼかし — いま：ISF Bump Distortion・形の stroke・四角（枠は形で描ける） — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Jittered Ripples**（レンズ・ゆがみ）：画面の効果。名前から：波紋・揺れ・グリッチ・ちぎれる（中国語名 波纹抖动）。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Jittered Ripples 2**（レンズ・ゆがみ）：画面の効果。名前から：波紋・揺れ・グリッチ・ちぎれる（中国語名 波纹抖动2）。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Jittered Ripples 3**（レンズ・ゆがみ）：画面の効果。名前から：波紋・揺れ・グリッチ・ちぎれる（中国語名 波纹抖动3）。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Lightning Twist**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・ねじれ・稲妻・電気。つまみ：明るさ・色・強さ・ゆがみ・速さ — いま：ISF Twirl — 借りる先：見つからない
- [ ] **Magnifying Glass**（レンズ・ゆがみ）：画面の効果。名前から：レンズ・魚眼・ガラス。つまみ：色・大きさ・強さ・縦ずれ・横ずれ — いま：ISF Bump Distortion・ISF Frosted Glass — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Mirror**（レンズ・ゆがみ）：画面の効果。名前から：鏡像・鏡（中国語名 镜像） — いま：mirror.wgsl — 借りる先：three.js shaders MirrorShader（MIT、名前が同じ）／ISF-Files Mirror（MIT、名前が同じ）
- [ ] **Mosaic**（レンズ・ゆがみ）：画面の効果。名前から：モザイク・ピクセル（中国語名 马赛克）。つまみ：ぼかし — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：GPUImage MosaicFilter（BSD-3、名前が同じ）／ISF-Files Mosaic（MIT、名前が同じ）／gl-transitions Mosaic（MIT、名前が同じ）
- [ ] **Pixel Glitch**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク・ピクセル・グリッチ。つまみ：範囲・強さ・速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Pixel Scan**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク・ピクセル・光が走る・スキャン。つまみ：回転・大きさ・範囲・ゆがみ・シャープ・明るさ・ぼかし・色・速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Rainbow Twist**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・ねじれ・虹・色。つまみ：速さ・明るさ・ゆがみ — いま：ISF Twirl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Ripple**（レンズ・ゆがみ）：画面の効果。名前から：波紋・ゆがみ・ちぎれる（中国語名 波纹扭曲）。つまみ：シャープ・ぼかし・大きさ・速さ・強さ・ゆがみ — いま：ISF Ripples・Circle Splash Distortion・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：gl-transitions ripple（MIT、名前が同じ）
- [ ] **Ripples**（レンズ・ゆがみ）：画面の効果。名前から：水の波紋・波紋・ちぎれる（中国語名 水波纹）。つまみ：速さ・ゆがみ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：ISF-Files Ripples（MIT、名前が同じ）
- [ ] **Ripples Clear**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・ちぎれる。つまみ：速さ・範囲・ゆがみ・色の濃さ・シャープ・ぼかし — いま：ISF Ripples・Circle Splash Distortion — 借りる先：見つからない
- [ ] **Rippling**（レンズ・ゆがみ・有料）：画面の効果。名前から：揺らめく・波紋・ちぎれる（中国語名 荡漾）。つまみ：範囲・強さ — いま：ISF Noise Displace・形の warp・ISF Ripples・Circle Splash Distortion — 借りる先：見つからない
- [ ] **Shaky Ripples**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・ちぎれる・揺れ。つまみ：強さ・速さ・ゆがみ・明るさ・背景の動き — いま：ISF Ripples・Circle Splash Distortion・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Shockwave**（レンズ・ゆがみ・有料）：画面の効果。名前から：衝撃波・波紋・稲妻・電気（中国語名 冲击波）。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：ISF-Files Shockwave（MIT、名前が同じ）
- [ ] **Striped Glass**（レンズ・ゆがみ・有料）：画面の効果。名前から：ガラス・帯に割れる（中国語名 长虹玻璃）。つまみ：ゆがみ・横ずれ・ぼかし・回転・大きさ — いま：ISF Frosted Glass — 借りる先：ISF Sliding Strips（MIT）
- [ ] **Tri-layer Mirror**（レンズ・ゆがみ）：画面の効果。名前から：鏡像・格子・鏡（中国語名 镜像三格）。つまみ：縦ずれ・色の濃さ — いま：mirror.wgsl・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **Twisted Focus**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・ねじれ・ピント。つまみ：大きさ・ゆがみ・速さ・ぼかし・明るさ — いま：ISF Twirl・段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Vortex Setting**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・ねじれ（中国語名 漩涡背景）。つまみ：強さ — いま：ISF Twirl — 借りる先：見つからない
- [ ] **Whippy Twist**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・ねじれ・滑る・動く。つまみ：速さ — いま：ISF Twirl・キー x・y — 借りる先：見つからない
- [ ] **Wide Angle**（レンズ・ゆがみ）：画面の効果。名前から：広角・レンズ・魚眼（中国語名 广角）。つまみ：強さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Y2K金属漩涡**（レンズ・ゆがみ・有料）：画面の効果。名前から：渦・金属。つまみ：速さ・やわらかさ・ゆがみ — いま：ISF Twirl・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **万向镜像**（レンズ・ゆがみ）：画面の効果。名前から：鏡像。つまみ：回転・範囲・ぼかし・背景の動き・横ずれ・縦ずれ — いま：mirror.wgsl — 借りる先：見つからない
- [ ] **万花筒**（レンズ・ゆがみ・有料）：画面の効果。名前から：万華鏡。つまみ：強さ・背景の動き — いま：kaleido.wgsl・ISF Kaleidoscope Tile — 借りる先：見つからない
- [ ] **万花筒 II**（レンズ・ゆがみ・有料）：画面の効果。名前から：万華鏡。つまみ：強さ・背景の動き — いま：kaleido.wgsl・ISF Kaleidoscope Tile — 借りる先：見つからない
- [ ] **下雨涟漪**（レンズ・ゆがみ・有料）：画面の効果。名前から：さざ波・雨。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・点群 Appear Rain（点群だけ） — 借りる先：Kenney Particle Pack（CC0）
- [ ] **低像素光效**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・光。つまみ：速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **低像素风写真**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・風。つまみ：明るさ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・キー x・y — 借りる先：見つからない
- [ ] **低像素风写真镜头**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・カメラ・風。つまみ：強さ・ぼかし — いま：ISF Pixellate・Mosaic・Multi-Pixellate・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **像素屏幕**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・画面。つまみ：速さ・大きさ・強さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **像素拉伸**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・伸ばす。つまみ：ゆがみ・回転・強さ・横ずれ・縦ずれ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **像素排序**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル。つまみ：数・明るさ・ノイズ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **像素条纹**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・しま — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：ISF Sliding Strips（MIT）
- [ ] **像素狂潮**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル。つまみ：速さ・大きさ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **像素画**（レンズ・ゆがみ）：画面の効果。名前から：ピクセル。つまみ：色・色の濃さ・大きさ・数・背景の動き — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **像素翡翠**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル。つまみ：速さ・強さ・質感・色 — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **像素迷雾**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・霧。つまみ：速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・段の fog — 借りる先：Kenney Particle Pack の煙（CC0）
- [ ] **光影流动**（レンズ・ゆがみ・有料）：画面の効果。名前から：流れる・光・影。つまみ：ノイズ・範囲・強さ・色の濃さ — いま：ISF Noise Displace・形の warp・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Ghosting（MIT）
- [ ] **光效扭曲**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・光。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **冰霜**（レンズ・ゆがみ・有料）：画面の効果。名前から：霜。つまみ：速さ・背景の動き — いま：ISF Frosted Glass — 借りる先：見つからない
- [ ] **反向鱼眼**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼。つまみ：強さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **哈哈镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：びっくり鏡。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **声波涟漪**（レンズ・ゆがみ・有料）：画面の効果。名前から：さざ波・波。つまみ：速さ・色の濃さ — いま：ISF Ripples・Circle Splash Distortion・ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **复古马赛克**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク・レトロ。つまみ：ぼかし — いま：ISF Pixellate・Mosaic・Multi-Pixellate・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **广角畸变**（レンズ・ゆがみ・有料）：画面の効果。名前から：広角。つまみ：強さ・ぼかし・横の色ずれ・横ずれ・縦ずれ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **弯曲透镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：レンズ。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **彩光流动 II**（レンズ・ゆがみ・有料）：画面の効果。名前から：流れる・光。つまみ：大きさ・明るさ・背景の動き・色・ぼかし・強さ — いま：ISF Noise Displace・形の warp・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **彩光流动 III**（レンズ・ゆがみ・有料）：画面の効果。名前から：流れる・光。つまみ：明るさ・背景の動き・色・範囲・横ずれ — いま：ISF Noise Displace・形の warp・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **彩色像素**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・カラー。つまみ：明るさ・ぼかし・背景の動き・速さ・色・数・大きさ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色像素排序**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・カラー。つまみ：強さ・背景の動き・色・やわらかさ・明るさ・ぼかし・速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩虹水滴**（レンズ・ゆがみ）：画面の効果。名前から：しずく・虹。つまみ：色・強さ・速さ — いま：ISF Ripples・Circle Splash Distortion・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **心形波纹**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・ハート。つまみ：速さ・強さ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **快照冲击波**（レンズ・ゆがみ・有料）：画面の効果。名前から：衝撃波・スナップ。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **截图放大镜**（レンズ・ゆがみ）：画面の効果。名前から：虫めがね・スクショ。つまみ：大きさ・強さ・縦ずれ・横ずれ・ぼかし — いま：ISF Bump Distortion・段の exposure・opacity をキー — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／ISF Fade Color（MIT）
- [ ] **扭曲360**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **扭曲光感**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・光。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **扭曲变焦**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・ズーム。つまみ：強さ・ゆがみ・ぼかし・横ずれ・縦ずれ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **扭曲旋入**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・回って入る。つまみ：速さ・強さ・ぼかし・明るさ・ゆがみ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・キー rotate — 借りる先：見つからない
- [ ] **扭曲模糊**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・ぼかし。つまみ：ぼかし・縦の色ずれ・速さ・強さ・明るさ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **扭曲缩放**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・拡大縮小。つまみ：速さ・ぼかし — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **护目镜视角**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゴーグル。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **放大镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **放大镜盘**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **放大镜视角**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね。つまみ：横ずれ・縦ずれ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **旋转透镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：レンズ・回転。つまみ：ぼかし・範囲・色の濃さ・ノイズ・ゆがみ・大きさ — いま：ISF Bump Distortion・キー rotate — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **星光鱼眼**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼・星の光。つまみ：大きさ・横ずれ・数・強さ・色の濃さ・質感 — いま：ISF Bump Distortion・ISF Star（1 個） — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **显微镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：顕微鏡。つまみ：速さ・強さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **望远镜视图**（レンズ・ゆがみ・有料）：画面の効果。名前から：望遠鏡。つまみ：速さ・数 — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **水波倒影**（レンズ・ゆがみ）：画面の効果。名前から：水の波紋・映り込み。つまみ：縦ずれ・大きさ・ゆがみ・ぼかし・速さ — いま：ISF Ripples・Circle Splash Distortion・mirror.wgsl — 借りる先：見つからない
- [ ] **水波模糊**（レンズ・ゆがみ・有料）：画面の効果。名前から：水の波紋・ぼかし。つまみ：横の色ずれ・ゆがみ・速さ・大きさ・ぼかし・色の濃さ — いま：ISF Ripples・Circle Splash Distortion・物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **水波泛起**（レンズ・ゆがみ・有料）：画面の効果。名前から：水の波紋。つまみ：ゆがみ・大きさ・強さ・数・横の色ずれ・速さ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：見つからない
- [ ] **水波流动**（レンズ・ゆがみ・有料）：画面の効果。名前から：水の波紋・流れる。つまみ：ゆがみ・大きさ・強さ・数・横の色ずれ・速さ — いま：ISF Ripples・Circle Splash Distortion・ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **水波纹 I**（レンズ・ゆがみ・有料）：画面の効果。名前から：水の波紋。つまみ：縦ずれ・速さ・色の濃さ・大きさ・ゆがみ — いま：ISF Ripples・Circle Splash Distortion — 借りる先：見つからない
- [ ] **水波负片**（レンズ・ゆがみ・有料）：画面の効果。名前から：水の波紋・ネガ。つまみ：速さ・ゆがみ — いま：ISF Ripples・Circle Splash Distortion・ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **水滴扩散**（レンズ・ゆがみ・有料）：画面の効果。名前から：しずく。つまみ：明るさ・速さ・横ずれ・横の色ずれ・ゆがみ・ぼかし — いま：ISF Ripples・Circle Splash Distortion — 借りる先：見つからない
- [ ] **波浪丝印**（レンズ・ゆがみ）：画面の効果。名前から：波打つ。つまみ：質感・色の濃さ・色・大きさ・強さ・回転 — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **波浪热感应**（レンズ・ゆがみ・有料）：画面の効果。名前から：波打つ — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **波纹故障**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・グリッチ。つまみ：速さ・強さ・数 — いま：ISF Ripples・Circle Splash Distortion・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **流体冲屏**（レンズ・ゆがみ・有料）：画面の効果。名前から：流体・画面に飛び込む。つまみ：速さ・範囲・色・明るさ・ゆがみ・背景の動き・ぼかし — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **流体荡开**（レンズ・ゆがみ・有料）：画面の効果。名前から：流体。つまみ：ゆがみ・色 — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **流动画面**（レンズ・ゆがみ・有料）：画面の効果。名前から：流れる。つまみ：速さ — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **涟漪摇摆**（レンズ・ゆがみ・有料）：画面の効果。名前から：さざ波・左右に揺れる。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・ふるまい oscillator を rotate に — 借りる先：見つからない
- [ ] **液体螺旋**（レンズ・ゆがみ・有料）：画面の効果。名前から：らせん・液体。つまみ：速さ・質感 — いま：ISF Twirl・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **液态分离**（レンズ・ゆがみ・有料）：画面の効果。名前から：液体。つまみ：強さ・ゆがみ・速さ・色 — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **溶液**（レンズ・ゆがみ・有料）：画面の効果。名前から：溶液。つまみ：速さ・明るさ・大きさ・範囲・ゆがみ・強さ・横ずれ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **热浪**（レンズ・ゆがみ・有料）：画面の効果。名前から：陽炎。つまみ：速さ — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **玻璃框选**（レンズ・ゆがみ）：画面の効果。名前から：ガラス・枠。つまみ：速さ — いま：ISF Frosted Glass・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **电光波动**（レンズ・ゆがみ・有料）：画面の効果。名前から：波打つ・電気の光。つまみ：速さ・範囲・明るさ・色・ゆがみ・ぼかし・背景の動き — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **电光漩涡**（レンズ・ゆがみ）：画面の効果。名前から：渦・電気の光。つまみ：速さ — いま：ISF Twirl — 借りる先：見つからない
- [ ] **破冰**（レンズ・ゆがみ）：画面の効果。名前から：氷。つまみ：速さ・背景の動き — いま：ISF Frosted Glass — 借りる先：見つからない
- [ ] **破碎镜像**（レンズ・ゆがみ・有料）：画面の効果。名前から：鏡像・割れる。つまみ：速さ — いま：mirror.wgsl — 借りる先：見つからない
- [ ] **移动像素**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル・動く。つまみ：速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・キー x・y — 借りる先：見つからない
- [ ] **箭头放大镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね・頭。つまみ：大きさ・強さ・色・縦ずれ・横ずれ・ぼかし — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **粉色震波**（レンズ・ゆがみ・有料）：画面の効果。名前から：衝撃波・ピンク。つまみ：速さ — いま：ISF Ripples・Circle Splash Distortion・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **紫色失真**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・紫。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **红黄蓝扭曲**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・赤・青。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **绿光波纹**（レンズ・ゆがみ・有料）：画面の効果。名前から：波紋・光・緑。つまみ：速さ・明るさ — いま：ISF Ripples・Circle Splash Distortion・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Color Phase（MIT）
- [ ] **荡漾 II**（レンズ・ゆがみ・有料）：画面の効果。名前から：揺らめく。つまみ：速さ・強さ・ゆがみ・数 — いま：ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **虚线放大镜**（レンズ・ゆがみ・有料）：画面の効果。名前から：虫めがね。つまみ：大きさ・強さ・色・縦ずれ・横ずれ・ぼかし — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **蛇形扭曲**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ。つまみ：速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：見つからない
- [ ] **负片漫画扭曲**（レンズ・ゆがみ・有料）：画面の効果。名前から：ゆがみ・ネガ・漫画 — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Color Invert・RGB Invert — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **锯齿像素**（レンズ・ゆがみ・有料）：画面の効果。名前から：ピクセル。つまみ：ぼかし — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **镜面星星**（レンズ・ゆがみ・有料）：画面の効果。名前から：鏡面・星。つまみ：速さ — いま：mirror.wgsl・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪光波浪**（レンズ・ゆがみ・有料）：画面の効果。名前から：波打つ・光る。つまみ：速さ — いま：ISF Noise Displace・形の warp・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **随机马赛克**（レンズ・ゆがみ）：画面の効果。名前から：モザイク。つまみ：大きさ・数・色の濃さ・色・背景の動き・ノイズ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **马赛克人物投影**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク・人物・影 — いま：ISF Pixellate・Mosaic・Multi-Pixellate・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）／ISF Ghosting（MIT）
- [ ] **马赛克变焦**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク・ズーム。つまみ：大きさ・ぼかし・明るさ・強さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **马赛克群**（レンズ・ゆがみ・有料）：画面の効果。名前から：モザイク。つまみ：速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：見つからない
- [ ] **马赛克闪切**（レンズ・ゆがみ）：画面の効果。名前から：モザイク・光って切り替わる。つまみ：大きさ・速さ・範囲 — いま：ISF Pixellate・Mosaic・Multi-Pixellate・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **鱼眼 III**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼。つまみ：色の濃さ・強さ・ゆがみ・範囲 — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **鱼眼涂鸦**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼・落書き。つまみ：速さ — いま：ISF Bump Distortion — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）／ISF Doodler（MIT）
- [ ] **鱼眼缩放**（レンズ・ゆがみ・有料）：画面の効果。名前から：魚眼・拡大縮小。つまみ：速さ・強さ・範囲 — いま：ISF Bump Distortion・キー scale・ISF Zoom — 借りる先：GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）

### 色（122）

- [ ] **4K HDR**（色・有料）：画面の効果。名前から：くっきり。つまみ：明るさ・ぼかし・シャープ・範囲・大きさ・やわらかさ・色の濃さ — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **90s画质**（色・有料）：画面の効果。名前から：画質。つまみ：質感・ノイズ・背景の動き・数・色の濃さ・色 — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **Alt. B&W**（色・有料）：画面の効果。名前から：白黒・格子（中国語名 黑白三格） — いま：ISF Color Monochrome・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **B&W Frame**（色・有料）：画面の効果。名前から：白黒・枠。つまみ：速さ — いま：ISF Color Monochrome・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **B&W Sketch**（色）：画面の効果。名前から：白黒・鉛筆画（中国語名 黑白线描）。つまみ：背景の動き — いま：ISF Color Monochrome — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **Blue Mosaic**（色）：画面の効果。名前から：青・モザイク・ピクセル（中国語名 蓝色丝印）。つまみ：質感・大きさ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient・ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：ISF Color Phase（MIT）
- [ ] **Butterfly Color**（色・有料）：画面の効果。名前から：カラー・蝶・虹・色（中国語名 彩色蝴蝶） — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Camera Focus**（色）：画面の効果。名前から：くっきり・ピント（中国語名 变清晰）。つまみ：速さ・ぼかし — いま：ISF Sharpen RGB・Unsharp Mask・段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Chrome Disco**（色・有料）：画面の効果。名前から：虹・色。つまみ：速さ・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Color Diamond**（色）：画面の効果。名前から：虹・色・きらきら・星（中国語名 彩钻）。つまみ：大きさ・数・色の濃さ・回転 — いま：ISF Multi Hue Shift・形の gradient・ISF Star（1 個） — 借りる先：ISF Color Phase（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Color Flame**（色・有料）：画面の効果。名前から：カラー・炎・虹・色（中国語名 彩色火焰）。つまみ：強さ・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Color Negative**（色・有料）：画面の効果。名前から：ネガ・カラー・反転・虹（中国語名 彩色负片）。つまみ：速さ・色 — いま：ISF Color Invert・RGB Invert・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Dark Night**（色）：画面の効果。名前から：闇夜・周辺減光・暗く（中国語名 暗夜）。つまみ：色の濃さ・質感・範囲 — いま：vignette.wgsl・段の vignette — 借りる先：見つからない
- [ ] **Frosted Quality**（色）：画面の効果。名前から：画質・ノイズ・ぼかし（中国語名 彩噪画质）。つまみ：強さ・速さ・大きさ・色の濃さ・ぼかし・シャープ・ノイズ — いま：ISF Sharpen RGB・Unsharp Mask・grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **HDR**（色・有料）：画面の効果。名前から：くっきり。つまみ：大きさ・数・強さ・色の濃さ・明るさ・範囲 — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **Infrared Lights**（色）：画面の効果。名前から：熱カメラ。つまみ：色・明るさ・範囲・速さ — いま：ISF Thermal Camera — 借りる先：ISF Night Vision（MIT）
- [ ] **Orange Negative**（色）：画面の効果。名前から：ネガ・反転（中国語名 橘色负片）。つまみ：速さ・色 — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **Polychromatic**（色・有料）：画面の効果。名前から：カラフル・虹・色（中国語名 缤纷）。つまみ：大きさ・数・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Rainbow Sparkle**（色・有料）：画面の効果。名前から：虹・泡・色・火花（中国語名 彩虹气泡）。つまみ：速さ・背景の動き・色の濃さ — いま：ISF Multi Hue Shift・形の gradient・ISF Star（1 個） — 借りる先：ISF Color Phase（MIT）／Kenney Particle Pack（CC0）
- [ ] **Shadow**（色）：画面の効果。名前から：影（中国語名 人鱼滤镜）。つまみ：速さ・横の色ずれ・縦の色ずれ・範囲 — いま：vignette.wgsl・段の vignette — 借りる先：見つからない
- [ ] **Sharpen Edges**（色）：画面の効果。名前から：シャープ・縁取り・くっきり（中国語名 精细锐化）。つまみ：ぼかし・シャープ・大きさ・範囲・色の濃さ — いま：ISF Sharpen RGB・Unsharp Mask・ISF Edges・Edge Blowout＋Glow — 借りる先：見つからない
- [ ] **Surreal**（色）：画面の効果。名前から：虹・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **To Color**（色）：画面の効果。名前から：カラー・白黒・虹・色（中国語名 变彩色）。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient・ISF Color Monochrome — 借りる先：ISF Color Phase（MIT）
- [ ] **Trippy**（色・有料）：画面の効果。名前から：幻覚・虹・色（中国語名 幻觉）。つまみ：速さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **Vignette**（色）：画面の効果。名前から：周辺減光・暗く（中国語名 暗角）。つまみ：質感 — いま：vignette.wgsl・段の vignette — 借りる先：three.js shaders VignetteShader（MIT、名前が同じ）／GPUImage VignetteFilter（BSD-3、名前が同じ）
- [ ] **三段式黑白**（色・有料）：画面の効果。名前から：白黒。つまみ：速さ — いま：ISF Color Monochrome — 借りる先：見つからない
- [ ] **五彩缤纷**（色）：画面の効果。名前から：カラフル。つまみ：強さ・速さ・回転・背景の動き・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **低画质**（色）：画面の効果。名前から：画質。つまみ：ぼかし・シャープ・横の色ずれ・色・色の濃さ — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **冬日缤纷**（色・有料）：画面の効果。名前から：カラフル。つまみ：大きさ・色の濃さ・範囲・回転・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **分区显色**（色・有料）：画面の効果。名前から：色を出す。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **动感色卡**（色）：画面の効果。名前から：色見本。つまみ：速さ・色の濃さ・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **单彩渐变**（色・有料）：画面の効果。名前から：グラデーション。つまみ：速さ・回転・色の濃さ・強さ・明るさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **单色**（色・有料）：画面の効果。名前から：単色。つまみ：速さ — いま：ISF Color Monochrome — 借りる先：見つからない
- [ ] **单色涂鸦**（色）：画面の効果。名前から：単色・落書き。つまみ：範囲・色・色の濃さ・強さ — いま：ISF Color Monochrome — 借りる先：ISF Doodler（MIT）
- [ ] **发光HDR**（色・有料）：画面の効果。名前から：HDR・光る。つまみ：明るさ・範囲・大きさ・シャープ・背景の動き・ぼかし・速さ — いま：ISF Sharpen RGB・Unsharp Mask・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **变清晰 II**（色・有料）：画面の効果。名前から：くっきり。つまみ：速さ・ぼかし — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **变色狙击**（色・有料）：画面の効果。名前から：色が変わる・打つ。つまみ：質感・色の濃さ・速さ・大きさ・強さ — いま：ISF Multi Hue Shift・形の gradient・ふるまい noise を x・y に — 借りる先：ISF Color Phase（MIT）／ISF Shake（MIT）
- [ ] **变色锐化**（色）：画面の効果。名前から：色が変わる・シャープ。つまみ：シャープ・明るさ・範囲・色・背景の動き・色の濃さ — いま：ISF Multi Hue Shift・形の gradient・ISF Sharpen RGB・Unsharp Mask — 借りる先：ISF Color Phase（MIT）
- [ ] **四色相组合**（色・有料）：画面の効果。名前から：色相。つまみ：色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **夜视仪**（色）：画面の効果。名前から：暗視 — いま：ISF Thermal Camera — 借りる先：ISF Night Vision（MIT）
- [ ] **夜视电光**（色・有料）：画面の効果。名前から：暗視・電気の光。つまみ：質感・範囲・強さ・色の濃さ — いま：ISF Thermal Camera — 借りる先：ISF Night Vision（MIT）
- [ ] **小彩虹**（色）：画面の効果。名前から：虹 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **局部上色**（色・有料）：画面の効果。名前から：色を付ける。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **局部变色**（色・有料）：画面の効果。名前から：色が変わる。つまみ：範囲・強さ・数・速さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **幻彩文字**（色）：画面の効果。名前から：虹色・文字。つまみ：速さ・大きさ — いま：ISF Multi Hue Shift・形の gradient・animators print_on・print_off — 借りる先：ISF Color Phase（MIT）
- [ ] **幻觉 II**（色・有料）：画面の効果。名前から：幻覚。つまみ：速さ・ゆがみ・大きさ・やわらかさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **强锐化**（色）：画面の効果。名前から：シャープ。つまみ：強さ・横の色ずれ・縦の色ずれ — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **彩色影调**（色・有料）：画面の効果。名前から：カラー・影。つまみ：速さ・大きさ — いま：ISF Multi Hue Shift・形の gradient・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Color Phase（MIT）／ISF Ghosting（MIT）
- [ ] **彩色泼墨**（色・有料）：画面の効果。名前から：カラー・墨。つまみ：速さ・色 — いま：ISF Multi Hue Shift・形の gradient・点群の dissolve・ISF Noise — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色漫画**（色・有料）：画面の効果。名前から：カラー・漫画。つまみ：色の濃さ・質感 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **彩色珠滴**（色）：画面の効果。名前から：カラー。つまみ：数・ぼかし・速さ・背景の動き・色の濃さ・大きさ・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色碎彩**（色・有料）：画面の効果。名前から：カラー。つまみ：速さ・大きさ・強さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **彩色碎闪**（色・有料）：画面の効果。名前から：カラー・光る。つまみ：数・強さ・大きさ・色の濃さ・速さ — いま：ISF Multi Hue Shift・形の gradient・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Color Phase（MIT）／ISF Strobe（MIT）
- [ ] **彩色雪花**（色・有料）：画面の効果。名前から：カラー・雪。つまみ：色・数・速さ・大きさ・背景の動き — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **彩虹光影**（色・有料）：画面の効果。名前から：虹・光・影。つまみ：横ずれ・縦ずれ・回転・大きさ・色の濃さ・背景の動き — いま：ISF Multi Hue Shift・形の gradient・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Color Phase（MIT）／ISF Ghosting（MIT）
- [ ] **彩虹贴纸**（色・有料）：画面の効果。名前から：虹・紙。つまみ：大きさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **暗黑蝙蝠**（色）：画面の効果。名前から：暗い。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：vignette.wgsl・段の vignette — 借りる先：見つからない
- [ ] **未来红**（色・有料）：画面の効果。名前から：赤。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **液态金属**（色・有料）：画面の効果。名前から：金属・液体。つまみ：色・ぼかし・質感・色の濃さ — いま：ISF Multi Hue Shift・形の gradient・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：ISF Color Phase（MIT）
- [ ] **深红之夜**（色・有料）：画面の効果。名前から：赤。つまみ：数・質感 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **渐变色**（色・有料）：画面の効果。名前から：色が変わる・じわっと — いま：ISF Multi Hue Shift・形の gradient・キー opacity・ISF Fade — 借りる先：ISF Color Phase（MIT）
- [ ] **灵异幻觉**（色・有料）：画面の効果。名前から：幻覚。つまみ：速さ・ゆがみ・色・明るさ・ノイズ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **热成像**（色・有料）：画面の効果。名前から：熱カメラ。つまみ：速さ — いま：ISF Thermal Camera — 借りる先：ISF Night Vision（MIT）
- [ ] **爱心暗角**（色）：画面の効果。名前から：周辺減光・ハート。つまみ：速さ・色の濃さ・横の色ずれ・縦の色ずれ — いま：vignette.wgsl・段の vignette — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **画布颜色打开**（色・有料）：画面の効果。名前から：色。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **白色负片**（色・有料）：画面の効果。名前から：ネガ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **粉色轻曝**（色）：画面の効果。名前から：軽い露出・ピンク。つまみ：色の濃さ・強さ — いま：段の exposure・ISF Exposure Adjust・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **紫橘渐变**（色）：画面の効果。名前から：グラデーション・紫。つまみ：範囲・明るさ・色・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **紫罗兰画布**（色・有料）：画面の効果。名前から：紫。つまみ：範囲・強さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **紫色负片**（色・有料）：画面の効果。名前から：ネガ・紫。つまみ：速さ・色 — いま：ISF Color Invert・RGB Invert・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **红色包装**（色・有料）：画面の効果。名前から：赤。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **绿色双重**（色・有料）：画面の効果。名前から：緑。つまみ：強さ・大きさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **绿色视野**（色・有料）：画面の効果。名前から：緑。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **缤纷彩带**（色・有料）：画面の効果。名前から：カラフル・紙テープ。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient・レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：ISF Color Phase（MIT）
- [ ] **缤纷烟花**（色・有料）：画面の効果。名前から：カラフル・花火。つまみ：大きさ・縦ずれ・横ずれ・色の濃さ・速さ — いま：ISF Multi Hue Shift・形の gradient・レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：ISF Color Phase（MIT）
- [ ] **脱色**（色・有料）：画面の効果。名前から：色を抜く。つまみ：速さ・明るさ — いま：ISF Color Monochrome — 借りる先：見つからない
- [ ] **致幻彩噪**（色・有料）：画面の効果。名前から：虹色・ノイズ。つまみ：色・色の濃さ — いま：ISF Multi Hue Shift・形の gradient・grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF Color Phase（MIT）
- [ ] **色相狂欢**（色・有料）：画面の効果。名前から：色相。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **节奏热成像**（色）：画面の効果。名前から：熱カメラ・リズム。つまみ：速さ・大きさ・明るさ — いま：ISF Thermal Camera・ふるまい oscillator を scale に — 借りる先：ISF Night Vision（MIT）
- [ ] **蓝屏dv**（色・有料）：画面の効果。名前から：青。つまみ：速さ・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **蓝色渐变**（色・有料）：画面の効果。名前から：グラデーション・青。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **蓝色负片**（色）：画面の効果。名前から：ネガ・青。つまみ：速さ・色 — いま：ISF Color Invert・RGB Invert・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **蓝边电子**（色・有料）：画面の効果。名前から：青。つまみ：明るさ・質感・色の濃さ・ぼかし・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **血墨晕染**（色・有料）：画面の効果。名前から：血・墨。つまみ：背景の動き・色・明るさ・大きさ・質感・ゆがみ・色の濃さ・速さ — いま：点群の dissolve・ISF Noise — 借りる先：見つからない
- [ ] **负变异**（色・有料）：画面の効果。名前から：ネガ。つまみ：速さ・強さ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **负片分屏**（色・有料）：画面の効果。名前から：ネガ・分割画面。つまみ：速さ・強さ — いま：ISF Color Invert・RGB Invert・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **负片切割**（色・有料）：画面の効果。名前から：ネガ・切る — いま：ISF Color Invert・RGB Invert — 借りる先：ISF Sliding Strips（MIT）
- [ ] **负片崩溃**（色・有料）：画面の効果。名前から：ネガ。つまみ：速さ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **负片涂鸦**（色・有料）：画面の効果。名前から：ネガ・落書き。つまみ：質感・大きさ・強さ・色の濃さ・速さ — いま：ISF Color Invert・RGB Invert — 借りる先：ISF Doodler（MIT）
- [ ] **负片涂鸦 II**（色・有料）：画面の効果。名前から：ネガ・落書き。つまみ：質感・大きさ・速さ・ノイズ・色の濃さ・背景の動き — いま：ISF Color Invert・RGB Invert — 借りる先：ISF Doodler（MIT）
- [ ] **负片涂鸦 III**（色・有料）：画面の効果。名前から：ネガ・落書き。つまみ：色・明るさ — いま：ISF Color Invert・RGB Invert — 借りる先：ISF Doodler（MIT）
- [ ] **负片游移**（色・有料）：画面の効果。名前から：ネガ。つまみ：色・大きさ・色の濃さ・速さ・縦ずれ・横ずれ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **负片滑动**（色・有料）：画面の効果。名前から：ネガ・滑る。つまみ：速さ — いま：ISF Color Invert・RGB Invert・キー x・y — 借りる先：見つからない
- [ ] **负片闪频 II**（色）：画面の効果。名前から：ネガ・光る。つまみ：速さ・色の濃さ・質感・明るさ・色 — いま：ISF Color Invert・RGB Invert・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **超强锐化**（色・有料）：画面の効果。名前から：シャープ。つまみ：シャープ・範囲・色の濃さ・ぼかし — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **超强锐化 II**（色・有料）：画面の効果。名前から：シャープ。つまみ：シャープ・範囲・強さ・色・色の濃さ・ぼかし — いま：ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **超级色彩震撼**（色・有料）：画面の効果。名前から：色・揺れ。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient・ふるまい noise を x・y に — 借りる先：ISF Color Phase（MIT）／ISF Shake（MIT）
- [ ] **超负面**（色・有料）：画面の効果。名前から：ネガ。つまみ：速さ・明るさ — いま：ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **迷幻荡漾**（色・有料）：画面の効果。名前から：サイケ・揺らめく。つまみ：強さ・ぼかし・横ずれ・速さ・範囲・ゆがみ — いま：ISF Multi Hue Shift・形の gradient・ISF Noise Displace・形の warp — 借りる先：ISF Color Phase（MIT）
- [ ] **迷幻震动**（色・有料）：画面の効果。名前から：サイケ・振動。つまみ：ゆがみ・横の色ずれ・強さ・色・速さ — いま：ISF Multi Hue Shift・形の gradient・ふるまい noise を x・y に — 借りる先：ISF Color Phase（MIT）／ISF Shake（MIT）
- [ ] **重新上色闪耀**（色・有料）：画面の効果。名前から：色を付ける・光る。つまみ：速さ・横ずれ — いま：ISF Multi Hue Shift・形の gradient・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Color Phase（MIT）／ISF Strobe（MIT）
- [ ] **金色负片**（色・有料）：画面の効果。名前から：ネガ・金色 — いま：ISF Color Invert・RGB Invert・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **锐化负片**（色・有料）：画面の効果。名前から：ネガ・シャープ。つまみ：速さ — いま：ISF Color Invert・RGB Invert・ISF Sharpen RGB・Unsharp Mask — 借りる先：見つからない
- [ ] **镭射边界**（色）：画面の効果。名前から：レーザー光。つまみ：速さ・背景の動き — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **阴影负片**（色・有料）：画面の効果。名前から：ネガ・影。つまみ：範囲・強さ・色・大きさ・色の濃さ — いま：ISF Color Invert・RGB Invert・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **随机单色**（色）：画面の効果。名前から：単色。つまみ：速さ・色の濃さ・強さ・色 — いま：ISF Color Monochrome — 借りる先：見つからない
- [ ] **雪花变色**（色・有料）：画面の効果。名前から：色が変わる・雪。つまみ：速さ・強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **颜色调整**（色・有料）：画面の効果。名前から：色。つまみ：速さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **黑白十字绣**（色・有料）：画面の効果。名前から：白黒・字。つまみ：質感・横ずれ・縦ずれ・大きさ — いま：ISF Color Monochrome・animators print_on・print_off — 借りる先：見つからない
- [ ] **黑白叠加**（色・有料）：画面の効果。名前から：白黒・重ねる。つまみ：速さ — いま：ISF Color Monochrome・キー opacity・ISF Fade — 借りる先：見つからない
- [ ] **黑白影调**（色・有料）：画面の効果。名前から：白黒・影。つまみ：速さ — いま：ISF Color Monochrome・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **黑白滚动**（色・有料）：画面の効果。名前から：白黒・スクロール。つまみ：速さ — いま：ISF Color Monochrome・キー x・y — 借りる先：見つからない
- [ ] **黑白漫画**（色）：画面の効果。名前から：白黒・漫画。つまみ：色の濃さ・質感 — いま：ISF Color Monochrome — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **黑白漫画 II**（色）：画面の効果。名前から：白黒・漫画。つまみ：色の濃さ — いま：ISF Color Monochrome — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **黑白疾闪**（色・有料）：画面の効果。名前から：白黒・光る。つまみ：速さ — いま：ISF Color Monochrome・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **黑白雪花**（色・有料）：画面の効果。名前から：白黒・雪。つまみ：速さ — いま：ISF Color Monochrome — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **Dripping Blood**（色・有料）：画面の効果。名前から：ホラー。つまみ：速さ・大きさ・横ずれ・縦ずれ・質感・強さ — いま：無い — 借りる先：見つからない
- [ ] **Warning**（色）：画面の効果。名前から：ホラー（中国語名 预警） — いま：無い — 借りる先：見つからない
- [ ] **恐怖故事 III**（色）：画面の効果。名前から：ホラー。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **恐怖来袭**（色・有料）：画面の効果。名前から：ホラー。つまみ：速さ・強さ — いま：無い — 借りる先：見つからない
- [ ] **暗夜精灵**（色・有料）：画面の効果。名前から：闇夜。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **暗夜蝙蝠**（色）：画面の効果。名前から：闇夜。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない

### グリッチ（101）

- [ ] **Chromatic Quirk**（グリッチ）：画面の効果。名前から：色ずれ（中国語名 色差）。つまみ：横の色ずれ・縦の色ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **Chrome Blur**（グリッチ）：画面の効果。名前から：色ずれ・放射・ぼかし（中国語名 色差放射）。つまみ：ぼかし・横の色ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF God Rays — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **Chromo-zoom**（グリッチ）：画面の効果。名前から：色ずれ・拡大・放射ぼかし・縮小（中国語名 色差放大）。つまみ：速さ・大きさ — いま：rgbshift.wgsl（three.js RGBShiftShader）・キー scale・ISF Zoom — 借りる先：ISF Chromatic Aberration（MIT）／GPUImage3 ZoomBlur（BSD-3）
- [ ] **Color Glitch**（グリッチ・有料）：画面の効果。名前から：グリッチ・虹色・虹・色（中国語名 幻彩故障）。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Multi Hue Shift・形の gradient — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Color Phase（MIT）
- [ ] **Color Glitch**（グリッチ・有料）：画面の効果。名前から：色ずれ・グリッチ・虹・色（中国語名 色差故障）。つまみ：速さ・強さ・横の色ずれ・縦の色ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Chromatic Aberration（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Color Glitch 2**（グリッチ）：画面の効果。名前から：色ずれ・グリッチ・虹・色（中国語名 色差故障 II）。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Chromatic Aberration（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Distort**（グリッチ・有料）：画面の効果。名前から：グリッチ・ゆがみ（中国語名 故障 II）。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Flashing Frame**（グリッチ）：画面の効果。名前から：コマ止め・点滅・光る（フラッシュ）・枠（中国語名 定格闪烁）。つまみ：速さ・色の濃さ — いま：ISF RGB Strobe・ふるまい random を opacity に・段の exposure・opacity をキー — 借りる先：ISF Freeze Frame・Random Freeze（MIT）／ISF Strobe（MIT）
- [ ] **Fuzzy**（グリッチ・有料）：画面の効果。名前から：グリッチ（毛羽）・粒・ノイズ・ぼかし（中国語名 毛刺）。つまみ：速さ・強さ・横の色ずれ・縦の色ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Glitch**（グリッチ）：画面の効果。名前から：グリッチ。つまみ：速さ・色の濃さ・範囲 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Glitch Intro**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：シャープ・横の色ずれ・強さ・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Level Glitch**（グリッチ・有料）：画面の効果。名前から：グリッチ（中国語名 横纹故障）。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Level Glitch 2**（グリッチ）：画面の効果。名前から：グリッチ（中国語名 横纹故障 II）。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **RGB Border**（グリッチ・有料）：画面の効果。名前から：RGB ずれ・縁取り・色ずれ（中国語名 RGB描边）。つまみ：速さ・横ずれ・縦ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Edges・Edge Blowout＋Glow — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **RGB Shake**（グリッチ・有料）：画面の効果。名前から：色ずれ・揺れ。つまみ：強さ・縦の色ずれ・ぼかし・背景の動き・明るさ・色・速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ふるまい noise を x・y に — 借りる先：ISF Chromatic Aberration（MIT）／ISF Shake（MIT）
- [ ] **RGB 故障**（グリッチ・有料）：画面の効果。名前から：RGB ずれ・グリッチ。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Chromatic Aberration（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **RGB闪光**（グリッチ・有料）：画面の効果。名前から：RGB ずれ・光る。つまみ：速さ・強さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・段の exposure・opacity をキー — 借りる先：ISF Chromatic Aberration（MIT）／ISF Fade Color（MIT）
- [ ] **Snow Glitch**（グリッチ）：画面の効果。名前から：グリッチ・雪。つまみ：範囲・ノイズ・横の色ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **X-Signal**（グリッチ）：画面の効果。名前から：グリッチ・衝撃。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ふるまい noise を x・y に — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Shake（MIT）
- [ ] **三重旋转**（グリッチ・有料）：画面の効果。名前から：三重・回転。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・キー rotate — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **三重照片**（グリッチ・有料）：画面の効果。名前から：三重・写真。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・形の stroke・四角（枠は形で描ける） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **三重缩小**（グリッチ・有料）：画面の効果。名前から：三重・縮小。つまみ：速さ・質感 — いま：rgbshift.wgsl（three.js RGBShiftShader）・キー scale・ISF Zoom — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **三重面板**（グリッチ・有料）：画面の効果。名前から：三重。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **上下混乱**（グリッチ・有料）：画面の効果。名前から：乱れ。つまみ：横の色ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **乱码涂鸦**（グリッチ・有料）：画面の効果。名前から：文字化け・落書き。つまみ：質感・大きさ・色の濃さ・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Doodler（MIT）
- [ ] **乱码背景**（グリッチ・有料）：画面の効果。名前から：文字化け。つまみ：速さ・色・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **信号不好**（グリッチ・有料）：画面の効果。名前から：信号の乱れ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **信号干扰**（グリッチ・有料）：画面の効果。名前から：信号の乱れ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **分格故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・格子。つまみ：速さ・数・強さ・シャープ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Random Squares Mask・Triangle Mosaic — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **单色错误**（グリッチ・有料）：画面の効果。名前から：エラー・単色。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Color Monochrome — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **卡顿电视**（グリッチ・有料）：画面の効果。名前から：カクつき・テレビ。つまみ：速さ・強さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：ISF Freeze Frame・Random Freeze（MIT）／ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **古怪撕裂**（グリッチ・有料）：画面の効果。名前から：裂ける。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **夸张信号**（グリッチ・有料）：画面の効果。名前から：信号の乱れ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **定格旋转木马 2**（グリッチ・有料）：画面の効果。名前から：コマ止め・回転。つまみ：ぼかし — いま：キー rotate — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **广告故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ・色 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **广告故障 2**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ・色 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **弯曲故障**（グリッチ）：画面の効果。名前から：グリッチ。つまみ：速さ・ゆがみ・背景の動き・質感・強さ・色の濃さ・範囲 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **彩虹棱镜**（グリッチ・有料）：画面の効果。名前から：プリズム・虹。つまみ：大きさ・質感・速さ・範囲・強さ・明るさ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Chromatic Aberration（MIT）／ISF Color Phase（MIT）
- [ ] **彩虹色差**（グリッチ・有料）：画面の効果。名前から：色ずれ・虹。つまみ：強さ・横の色ずれ・縦の色ずれ・大きさ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Chromatic Aberration（MIT）／ISF Color Phase（MIT）
- [ ] **微型三重奏**（グリッチ・有料）：画面の効果。名前から：三重。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **快照旋转**（グリッチ・有料）：画面の効果。名前から：スナップ・回転。つまみ：強さ — いま：キー rotate — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **恐怖故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・ホラー。つまみ：速さ・色の濃さ・ノイズ・シャープ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **拉伸快照**（グリッチ・有料）：画面の効果。名前から：スナップ・伸ばす。つまみ：速さ — いま：キー scale_x・scale_y・skew — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **摇晃故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・揺れ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ふるまい noise を x・y に — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Shake（MIT）
- [ ] **故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **故障卡带**（グリッチ・有料）：画面の効果。名前から：グリッチ・カード。つまみ：速さ・色の濃さ・ぼかし・横の色ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **故障定格**（グリッチ・有料）：画面の効果。名前から：グリッチ・コマ止め。つまみ：ぼかし・速さ・強さ・色の濃さ・背景の動き — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Freeze Frame・Random Freeze（MIT）
- [ ] **故障扫屏**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：強さ・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **故障散影**（グリッチ・有料）：画面の効果。名前から：グリッチ・影。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Ghosting（MIT）
- [ ] **故障线闪**（グリッチ・有料）：画面の効果。名前から：グリッチ・光る。つまみ：強さ・色・大きさ・ゆがみ・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Strobe（MIT）
- [ ] **故障遮蔽**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：大きさ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **故障镜头**（グリッチ・有料）：画面の効果。名前から：グリッチ・カメラ。つまみ：色・明るさ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・キー scale・ISF Zoom — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **故障闪爆**（グリッチ・有料）：画面の効果。名前から：グリッチ・光る。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Strobe（MIT）
- [ ] **故障震闪**（グリッチ・有料）：画面の効果。名前から：グリッチ・光る・揺れ。つまみ：明るさ・強さ・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Strobe（MIT）
- [ ] **数字相机**（グリッチ・有料）：画面の効果。名前から：デジタル・カメラ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・形の stroke・四角（枠は形で描ける） — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **数字矩阵**（グリッチ・有料）：画面の効果。名前から：デジタル。つまみ：ぼかし・背景の動き・速さ・色・大きさ・範囲 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **数据加载故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・データ崩れ。つまみ：速さ・ノイズ・色の濃さ・シャープ・質感 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **方形棱镜**（グリッチ）：画面の効果。名前から：プリズム。つまみ：速さ・範囲・縦ずれ・横ずれ・色の濃さ — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **末日故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **极光棱镜**（グリッチ・有料）：画面の効果。名前から：プリズム・光。つまみ：ノイズ・色の濃さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **棱镜幻影**（グリッチ・有料）：画面の効果。名前から：プリズム・幻影。つまみ：色・ぼかし — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Chromatic Aberration（MIT）／ISF Ghosting（MIT）
- [ ] **棱镜镜像**（グリッチ・有料）：画面の効果。名前から：プリズム・鏡像。つまみ：速さ・ぼかし — いま：rgbshift.wgsl（three.js RGBShiftShader）・mirror.wgsl — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **棱镜闪烁**（グリッチ・有料）：画面の効果。名前から：プリズム・点滅。つまみ：速さ・色・色の濃さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Chromatic Aberration（MIT）／ISF Strobe（MIT）
- [ ] **橙色故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **混乱**（グリッチ・有料）：画面の効果。名前から：乱れ。つまみ：ゆがみ・強さ・背景の動き・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **混乱展示**（グリッチ・有料）：画面の効果。名前から：乱れ。つまみ：速さ・明るさ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **激光故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・レーザー。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **环形棱镜**（グリッチ・有料）：画面の効果。名前から：プリズム。つまみ：速さ・色の濃さ・強さ・質感 — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **电视没信号-嘶（Bad TV）3**（グリッチ・有料）：画面の効果。名前から：信号の乱れ・テレビ。つまみ：強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **画面瑕疵**（グリッチ・有料）：画面の効果。名前から：傷。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **相片定格**（グリッチ）：画面の効果。名前から：コマ止め・写真。つまみ：速さ・色・強さ・色の濃さ・背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **眩光抽帧**（グリッチ・有料）：画面の効果。名前から：コマ落とし・まぶしい光。つまみ：速さ — いま：段の lens_flare・ISF Lens Flare — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **破旧故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ・質感 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **碎纸故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・紙。つまみ：速さ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **纹理故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：大きさ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **编辑错误**（グリッチ・有料）：画面の効果。名前から：エラー。つまみ：色の濃さ・質感 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **网格故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・格子。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Random Squares Mask・Triangle Mosaic — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **翻转错误**（グリッチ・有料）：画面の効果。名前から：エラー・裏返る。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・rotate_x・rotate_y — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **聚焦色散**（グリッチ・有料）：画面の効果。名前から：色の分散・ピント合わせ。つまみ：強さ・縦の色ずれ・範囲・ぼかし・ゆがみ・シャープ — いま：rgbshift.wgsl（three.js RGBShiftShader）・段の focus・aperture＋物の blur をキー — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **色差开幕**（グリッチ・有料）：画面の効果。名前から：色ずれ・幕が開く。つまみ：速さ・縦の色ずれ・横の色ずれ・質感・ゆがみ — いま：rgbshift.wgsl（three.js RGBShiftShader） — 借りる先：ISF Chromatic Aberration（MIT）／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **色差拉伸**（グリッチ・有料）：画面の効果。名前から：色ずれ・伸ばす。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・キー scale_x・scale_y・skew — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **色差故障2**（グリッチ・有料）：画面の効果。名前から：色ずれ・グリッチ。つまみ：速さ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Chromatic Aberration（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **色差星闪**（グリッチ）：画面の効果。名前から：色ずれ・光る・星。つまみ：速さ・色の濃さ・横の色ずれ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Chromatic Aberration（MIT）／ISF Strobe（MIT）
- [ ] **色差流动**（グリッチ・有料）：画面の効果。名前から：色ずれ・流れる。つまみ：大きさ・速さ・背景の動き・色の濃さ・質感・色 — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Noise Displace・形の warp — 借りる先：ISF Chromatic Aberration（MIT）
- [ ] **色散冲击**（グリッチ・有料）：画面の効果。名前から：色の分散・衝撃。つまみ：ゆがみ・ぼかし・速さ・横の色ずれ・縦の色ずれ・明るさ — いま：rgbshift.wgsl（three.js RGBShiftShader）・ふるまい noise を x・y に — 借りる先：ISF Chromatic Aberration（MIT）／ISF Shake（MIT）
- [ ] **色散故障**（グリッチ・有料）：画面の効果。名前から：色の分散・グリッチ。つまみ：速さ・色 — いま：rgbshift.wgsl（three.js RGBShiftShader）・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF Chromatic Aberration（MIT）／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **花屏故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・花。つまみ：横の色ずれ・横ずれ・速さ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **花屏故障 II**（グリッチ・有料）：画面の効果。名前から：グリッチ・花。つまみ：強さ・明るさ・ぼかし — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **诡异故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **轮廓错误**（グリッチ・有料）：画面の効果。名前から：エラー・輪郭。つまみ：速さ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Edges・Edge Blowout＋Glow — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **轻微故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：速さ・色の濃さ・ノイズ・シャープ・ぼかし・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **迷幻故障**（グリッチ・有料）：画面の効果。名前から：グリッチ・サイケ。つまみ：強さ・速さ・色 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Multi Hue Shift・形の gradient — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Color Phase（MIT）
- [ ] **阻隔错误**（グリッチ・有料）：画面の効果。名前から：エラー。つまみ：強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **雪花故障 I**（グリッチ）：画面の効果。名前から：グリッチ・雪。つまみ：範囲・ノイズ・横の色ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity
- [ ] **黑白错误**（グリッチ・有料）：画面の効果。名前から：エラー・白黒。つまみ：速さ・強さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Color Monochrome — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **黑色故障**（グリッチ・有料）：画面の効果。名前から：グリッチ。つまみ：縦ずれ・横ずれ・速さ・回転・色の濃さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **Step Printing**（グリッチ・有料）：画面の効果。名前から：コマ止め・コマ落とし。つまみ：ぼかし・範囲・速さ・色の濃さ — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **抽帧**（グリッチ・有料）：画面の効果。名前から：コマ落とし。つまみ：速さ — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT）
- [ ] **樱花定格**（グリッチ・有料）：画面の効果。名前から：コマ止め・桜 — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT）／Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **涂鸦抽帧**（グリッチ・有料）：画面の効果。名前から：コマ落とし・落書き。つまみ：速さ・強さ・質感・色 — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT）／ISF Doodler（MIT）
- [ ] **纸质抽帧**（グリッチ・有料）：画面の効果。名前から：コマ落とし・紙。つまみ：速さ・大きさ・ノイズ — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT）

### 分割（82）

- [ ] **Close Up**（分割・有料）：画面の効果。名前から：幕・扉が開く・拡大・縮小。つまみ：速さ・強さ・大きさ・横ずれ・縦ずれ — いま：キー scale・ISF Zoom・キー x・y — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Dual Screens**（分割・有料）：画面の効果。名前から：分割画面（中国語名 两屏） — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Fade In**（分割）：画面の効果。名前から：幕が開く・じわっと出る・フェード（中国語名 渐显开幕）。つまみ：速さ — いま：キー opacity・ISF Fade — 借りる先：Animista fade-in（BSD-2、名前が同じ）／Animate.css fadeIn（v3.7.2 は MIT、名前が同じ）／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Fade Out**（分割）：画面の効果。名前から：幕が閉じる・じわっと消える・フェード（中国語名 渐隐闭幕）。つまみ：速さ — いま：キー opacity・ISF Fade — 借りる先：Animista fade-out（BSD-2、名前が同じ）／Animate.css fadeOut（v3.7.2 は MIT、名前が同じ）／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Four Screens**（分割）：画面の効果。名前から：4 分割画面・分割画面（中国語名 四屏） — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Nine Screens**（分割）：画面の効果。名前から：9 分割画面・分割画面（中国語名 九屏） — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Pulse Line**（分割・有料）：画面の効果。名前から：帯に割れる・脈打つ。つまみ：ゆがみ・速さ・範囲・強さ・色・明るさ・やわらかさ・ぼかし — いま：ふるまい oscillator を scale に — 借りる先：ISF Sliding Strips（MIT）
- [ ] **Six Screens**（分割・有料）：画面の効果。名前から：6 分割画面・分割画面（中国語名 六屏） — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Split Screen**（分割）：画面の効果。名前から：分割・分割画面・帯に割れる（中国語名 两屏分割）。つまみ：速さ・色の濃さ・範囲 — いま：ISF Quad Tile — 借りる先：ISF Sliding Strips（MIT）／ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Three Screens**（分割・有料）：画面の効果。名前から：3 分割画面・分割画面（中国語名 三屏） — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **Torn Frames**（分割）：画面の効果。名前から：ちぎれる・枠。つまみ：速さ・色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Wipe Board**（分割・有料）：画面の効果。名前から：幕が開く・拭き取る・枠（中国語名 擦拭开幕）。つまみ：速さ — いま：ISF Wipe Left・Right・Up・Down・形の stroke・四角（枠は形で描ける） — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **不规则拼图**（分割・有料）：画面の効果。名前から：パズル。つまみ：速さ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **光盘擦拭**（分割・有料）：画面の効果。名前から：拭き取る・光。つまみ：速さ — いま：ISF Wipe Left・Right・Up・Down・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **动态照片拼贴**（分割・有料）：画面の効果。名前から：コラージュ・写真。つまみ：速さ — いま：ISF Quad Tile・形の stroke・四角（枠は形で描ける） — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **华容道拼图**（分割・有料）：画面の効果。名前から：パズル。つまみ：速さ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **卡片分屏**（分割・有料）：画面の効果。名前から：分割画面・カード。つまみ：速さ・色 — いま：ISF Quad Tile・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **反转开幕**（分割・有料）：画面の効果。名前から：幕が開く・回る。つまみ：速さ・大きさ・色の濃さ・色 — いま：キー rotate — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **四格递进**（分割・有料）：画面の効果。名前から：4 分割画面。つまみ：速さ — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **多屏抖动**（分割・有料）：画面の効果。名前から：多分割画面・揺れ — いま：ISF Quad Tile・ふるまい noise を x・y に — 借りる先：ISF MultiFrame 2x2・3x3（MIT）／ISF Shake（MIT）
- [ ] **多屏波普**（分割・有料）：画面の効果。名前から：多分割画面・波。つまみ：質感・色・ぼかし・シャープ・速さ — いま：ISF Quad Tile・ISF Noise Displace・形の warp — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **多机位**（分割・有料）：画面の効果。名前から：マルチカメラ。つまみ：明るさ・大きさ・範囲・数 — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **彩虹立方体**（分割・有料）：画面の効果。名前から：立方体・虹。つまみ：速さ — いま：ISF cube（gl-transitions cube）・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **情人节拼贴**（分割・有料）：画面の効果。名前から：コラージュ。つまみ：速さ — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **情绪照片拼贴**（分割・有料）：画面の効果。名前から：コラージュ・写真。つまみ：速さ・色の濃さ・ぼかし — いま：ISF Quad Tile・形の stroke・四角（枠は形で描ける） — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **拼贴开幕**（分割・有料）：画面の効果。名前から：コラージュ・幕が開く。つまみ：速さ・横ずれ・縦ずれ・大きさ・範囲 — いま：ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）／gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **撕拉片**（分割・有料）：画面の効果。名前から：ちぎる・引く。つまみ：速さ・色の濃さ・大きさ・ゆがみ — いま：キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **撕纸特写**（分割・有料）：画面の効果。名前から：紙をちぎる・アップ。つまみ：速さ・色の濃さ・質感 — いま：キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **方块剪辑**（分割・有料）：画面の効果。名前から：四角いブロック。つまみ：速さ・横ずれ・縦ずれ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **格纹纸质 II**（分割・有料）：画面の効果。名前から：紙・格子。つまみ：質感 — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **滑稽俄罗斯方块**（分割・有料）：画面の効果。名前から：四角いブロック。つまみ：速さ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **火焰魔方**（分割・有料）：画面の効果。名前から：ルービックキューブ・炎。つまみ：強さ — いま：ISF cube（gl-transitions cube） — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **瓷砖分割**（分割・有料）：画面の効果。名前から：分割・タイル。つまみ：速さ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：ISF Sliding Strips（MIT）
- [ ] **瓷砖拼盘**（分割・有料）：画面の効果。名前から：タイル・つなぎ合わせ。つまみ：速さ・背景の動き — いま：ISF Random Squares Mask・Triangle Mosaic・ISF Quad Tile — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **禁忌俄罗斯方块**（分割・有料）：画面の効果。名前から：四角いブロック。つまみ：速さ・質感 — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **纸醉金迷**（分割・有料）：画面の効果。名前から：紙・金色。つまみ：速さ・回転・ノイズ・背景の動き・色の濃さ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **线框立方体**（分割・有料）：画面の効果。名前から：立方体・枠。つまみ：速さ — いま：ISF cube（gl-transitions cube）・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **负相擦除**（分割・有料）：画面の効果。名前から：拭き取る・ネガ。つまみ：速さ・色 — いま：ISF Wipe Left・Right・Up・Down・ISF Color Invert・RGB Invert — 借りる先：見つからない
- [ ] **透视九宫格**（分割・有料）：画面の効果。名前から：9 分割画面・遠近。つまみ：速さ・質感・背景の動き — いま：ISF Quad Tile・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF MultiFrame 2x2・3x3（MIT）
- [ ] **重复栅格**（分割・有料）：画面の効果。名前から：格子。つまみ：速さ — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **金色碎片**（分割・有料）：画面の効果。名前から：破片・金色。つまみ：速さ・色の濃さ・大きさ・ノイズ — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **钻石碎片**（分割・有料）：画面の効果。名前から：破片・ダイヤ。つまみ：大きさ・速さ・数 — いま：ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Folds**（分割・有料）：画面の効果。名前から：折り目・折る（中国語名 折痕）。つまみ：質感 — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **Folds 2**（分割・有料）：画面の効果。名前から：折り目・折る（中国語名 折痕 II）。つまみ：質感 — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **Folds 4**（分割・有料）：画面の効果。名前から：折り目・折る（中国語名 折痕 IV）。つまみ：質感 — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **Folds 5**（分割・有料）：画面の効果。名前から：折り目・折る（中国語名 折痕 V）。つまみ：質感 — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **Horizontal Close**（分割）：画面の効果。名前から：幕が閉じる・幕・扉が開く（中国語名 闭幕） — いま：無い — 借りる先：gl-transitions HorizontalClose（MIT、名前が同じ）
- [ ] **Horizontal Open**（分割）：画面の効果。名前から：幕が開く・幕・扉が開く（中国語名 开幕 I） — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT、名前が同じ）
- [ ] **Landscape Close**（分割）：画面の効果。名前から：幕が閉じる・幕・扉が開く（中国語名 横向闭幕） — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Portrait Open**（分割）：画面の効果。名前から：幕が開く・幕・扉が開く（中国語名 纵向开幕） — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Vertical Close**（分割）：画面の効果。名前から：幕が閉じる・幕・扉が開く（中国語名 闭幕 II） — いま：無い — 借りる先：gl-transitions VerticalClose（MIT、名前が同じ）
- [ ] **Vertical Open**（分割）：画面の効果。名前から：幕が開く・幕・扉が開く（中国語名 开幕 II） — いま：無い — 借りる先：gl-transitions VerticalOpen（MIT、名前が同じ）
- [ ] **五图展开**（分割・有料）：画面の効果。名前から：広がる。つまみ：速さ・範囲 — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **交叉分割**（分割・有料）：画面の効果。名前から：分割。つまみ：速さ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **分割砖块**（分割・有料）：画面の効果。名前から：分割。つまみ：速さ・色 — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **分割素材**（分割・有料）：画面の効果。名前から：分割。つまみ：質感・強さ・横の色ずれ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **初始展开**（分割・有料）：画面の効果。名前から：広がる。つまみ：強さ — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **实况开幕**（分割・有料）：画面の効果。名前から：幕が開く。つまみ：速さ・色・明るさ・ぼかし — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **开门**（分割・有料）：画面の効果。名前から：扉が開く。つまみ：速さ・大きさ・質感・範囲 — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **弧形开幕**（分割・有料）：画面の効果。名前から：幕が開く。つまみ：速さ — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **彩幕展开**（分割・有料）：画面の効果。名前から：広がる。つまみ：速さ — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **快速条纹**（分割・有料）：画面の効果。名前から：しま。つまみ：速さ・強さ・大きさ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **折纸 2**（分割・有料）：画面の効果。名前から：紙・折る。つまみ：速さ・質感 — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **条纹侵蚀**（分割・有料）：画面の効果。名前から：しま。つまみ：速さ・色・横の色ずれ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **条纹构图**（分割・有料）：画面の効果。名前から：しま。つまみ：速さ・数 — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **横条开幕**（分割・有料）：画面の効果。名前から：幕が開く。つまみ：速さ・明るさ・ぼかし — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **浏览器关闭**（分割・有料）：画面の効果。名前から：閉じる。つまみ：速さ — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **竖向开幕**（分割・有料）：画面の効果。名前から：幕が開く。つまみ：ぼかし・明るさ・強さ・ゆがみ — いま：無い — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **纸张片头**（分割・有料）：画面の効果。名前から：紙・頭 — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **视频分割**（分割・有料）：画面の効果。名前から：分割。つまみ：速さ・強さ・横の色ずれ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **诡异分割**（分割・有料）：画面の効果。名前から：分割。つまみ：速さ・大きさ・ぼかし・色の濃さ・背景の動き — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **过热切片**（分割・有料）：画面の効果。名前から：切れ目。つまみ：速さ・明るさ・ぼかし — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **颠倒分割**（分割・有料）：画面の効果。名前から：分割。つまみ：速さ・シャープ・ノイズ・横の色ずれ — いま：無い — 借りる先：ISF Sliding Strips（MIT）
- [ ] **Paper Tear**（分割・有料）：画面の効果。名前から：ちぎる・紙・ちぎれる（中国語名 纸质撕边）。つまみ：質感 — いま：無い — 借りる先：見つからない
- [ ] **不规则撕纸**（分割・有料）：画面の効果。名前から：紙をちぎる — いま：無い — 借りる先：見つからない
- [ ] **五彩纸屑**（分割）：画面の効果。名前から：紙。つまみ：速さ・強さ・質感・回転・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **叠纸时空**（分割・有料）：画面の効果。名前から：紙。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **天空裂缝**（分割・有料）：画面の効果。名前から：裂け目。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **撕纸动画**（分割・有料）：画面の効果。名前から：紙をちぎる — いま：無い — 借りる先：見つからない
- [ ] **断裂胶带**（分割・有料）：画面の効果。名前から：切れる。つまみ：速さ・ノイズ・色の濃さ・横の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **白色裂痕**（分割・有料）：画面の効果。名前から：ひび。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **线描折痕**（分割・有料）：画面の効果。名前から：折り目 — いま：無い — 借りる先：見つからない

### 枠（66）

- [ ] **3D手机2**（枠・有料）：画面の効果。名前から：スマホ・3D。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **3D透边框**（枠・有料）：画面の効果。名前から：枠・3D — いま：形の stroke・四角（枠は形で描ける）・rotate_x・rotate_y・z（2.5D） — 借りる先：見つからない
- [ ] **Camcorder 2**（枠・有料）：画面の効果。名前から：枠・古いビデオ・テレビ（中国語名 录制边框 II） — いま：形の stroke・四角（枠は形で描ける）・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **Camcorder 3**（枠）：画面の効果。名前から：枠・古いビデオ・テレビ（中国語名 录制边框 III） — いま：形の stroke・四角（枠は形で描ける）・crt_finish.wgsl・ISF Bad TV — 借りる先：ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **ccd界面**（枠・有料）：画面の効果。名前から：画面の枠。つまみ：速さ・色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Daily**（枠・有料）：画面の効果。名前から：日常（中国語名 自然 I）。つまみ：大きさ・数・色の濃さ・回転 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Daily 2**（枠）：画面の効果。名前から：日常（中国語名 自然 III）。つまみ：大きさ・数・色の濃さ・回転 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Daily 4**（枠）：画面の効果。名前から：日常（中国語名 自然 IV）。つまみ：大きさ・数・色の濃さ・回転 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Film Frame**（枠）：画面の効果。名前から：フィルムの枠・古いフィルム・レトロ・枠（中国語名 胶片框） — いま：形の stroke・四角（枠は形で描ける）・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Film Frame 2**（枠・有料）：画面の効果。名前から：フィルムの枠・古いフィルム・レトロ・枠（中国語名 胶片框 II） — いま：形の stroke・四角（枠は形で描ける）・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Frame 1**（枠・有料）：画面の効果。名前から：枠（中国語名 录制框）。つまみ：ノイズ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Insta**（枠・有料）：画面の効果。名前から：枠（中国語名 ins边框） — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **Old**（枠）：画面の効果。名前から：写真・古いフィルム・レトロ（中国語名 老照片）。つまみ：質感・背景の動き — いま：形の stroke・四角（枠は形で描ける）・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Photo Frame**（枠）：画面の効果。名前から：枠（中国語名 画展边框） — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **White Sprockets**（枠・有料）：画面の効果。名前から：枠・古いフィルム・レトロ（中国語名 白胶边框）。つまみ：質感 — いま：形の stroke・四角（枠は形で描ける）・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl — 借りる先：見つからない
- [ ] **Wobbly Black**（枠）：画面の効果。名前から：枠・波打つ（中国語名 不规则黑框）。つまみ：背景の動き — いま：形の stroke・四角（枠は形で描ける）・ISF Noise Displace・形の warp — 借りる先：見つからない
- [ ] **三部手机**（枠・有料）：画面の効果。名前から：スマホ。つまみ：背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **分层边框**（枠・有料）：画面の効果。名前から：枠。つまみ：ぼかし — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **原相机滤镜**（枠・有料）：画面の効果。名前から：カメラ。つまみ：速さ・横ずれ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **原相机缩放**（枠・有料）：画面の効果。名前から：カメラ・拡大縮小。つまみ：速さ・横ずれ・縦ずれ・大きさ — いま：形の stroke・四角（枠は形で描ける）・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **取景框**（枠・有料）：画面の効果。名前から：ファインダー。つまみ：ノイズ・ぼかし — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **取景框 II**（枠）：画面の効果。名前から：ファインダー — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **古董相机**（枠・有料）：画面の効果。名前から：カメラ。つまみ：大きさ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **图框缩放**（枠・有料）：画面の効果。名前から：枠・拡大縮小。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **天使降临边框**（枠）：画面の効果。名前から：枠・天使 — いま：形の stroke・四角（枠は形で描ける）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **宝丽莱 III**（枠）：画面の効果。名前から：ポラロイド。つまみ：質感・大きさ・横ずれ・縦ずれ・回転・色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **弹出图框**（枠・有料）：画面の効果。名前から：枠・弾んで出る。つまみ：色の濃さ — いま：形の stroke・四角（枠は形で描ける）・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **弹弓相机**（枠・有料）：画面の効果。名前から：カメラ・弾む。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **微扭边框**（枠・有料）：画面の効果。名前から：枠 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **手写边框**（枠・有料）：画面の効果。名前から：枠。つまみ：色・色の濃さ・ノイズ・速さ・飾り — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **手持相机**（枠・有料）：画面の効果。名前から：カメラ。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **打开文件夹**（枠・有料）：画面の効果。名前から：フォルダ。つまみ：強さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **抖动图框**（枠・有料）：画面の効果。名前から：枠・揺れ。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **拍立得展开**（枠・有料）：画面の効果。名前から：インスタントカメラ・広がる。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **拍立得成像**（枠・有料）：画面の効果。名前から：インスタントカメラ。つまみ：速さ・色の濃さ・横ずれ・縦ずれ・ぼかし・強さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **播放器**（枠）：画面の効果。名前から：プレーヤー。つまみ：ぼかし — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **播放界面**（枠）：画面の効果。名前から：画面の枠。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **数码相机**（枠・有料）：画面の効果。名前から：カメラ。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **旧相机**（枠・有料）：画面の効果。名前から：カメラ。つまみ：色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **暗夜归来边框**（枠）：画面の効果。名前から：枠・闇夜 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **柜窗**（枠・有料）：画面の効果。名前から：窓。つまみ：速さ・強さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **毛绒边框**（枠・有料）：画面の効果。名前から：枠。つまみ：背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **涂鸦拍立得**（枠・有料）：画面の効果。名前から：インスタントカメラ・落書き。つまみ：速さ・強さ・色の濃さ・背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：ISF Doodler（MIT）
- [ ] **烧焦的拍立得**（枠・有料）：画面の効果。名前から：インスタントカメラ・焼ける。つまみ：速さ・色・大きさ — いま：形の stroke・四角（枠は形で描ける）・点群の Burn Away — 借りる先：ISF Film Burn・Undulating Burn Out（MIT）
- [ ] **玻璃边框**（枠）：画面の効果。名前から：枠・ガラス — いま：形の stroke・四角（枠は形で描ける）・ISF Frosted Glass — 借りる先：見つからない
- [ ] **电子录屏**（枠・有料）：画面の効果。名前から：画面録画 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **相机光标**（枠・有料）：画面の効果。名前から：カメラ・光。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **相机抓拍**（枠・有料）：画面の効果。名前から：カメラ・打つ。つまみ：ぼかし・速さ・色の濃さ・明るさ・縦ずれ・横ずれ — いま：形の stroke・四角（枠は形で描ける）・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **相机框抖动**（枠・有料）：画面の効果。名前から：カメラ・揺れ・枠。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **碎照片**（枠・有料）：画面の効果。名前から：写真。つまみ：ノイズ・速さ・強さ・色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **简易框架**（枠・有料）：画面の効果。名前から：枠。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **简约边框**（枠）：画面の効果。名前から：枠。つまみ：色 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **素材播放器**（枠・有料）：画面の効果。名前から：プレーヤー。つまみ：色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **纸质边框**（枠・有料）：画面の効果。名前から：枠・紙。つまみ：質感 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **胶片框光斑**（枠・有料）：画面の効果。名前から：フィルムの枠・光の玉 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter
- [ ] **舞蹈图框**（枠・有料）：画面の効果。名前から：枠。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **花园图框**（枠・有料）：画面の効果。名前から：枠・花。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **解压图框**（枠・有料）：画面の効果。名前から：枠。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **车窗 II**（枠・有料）：画面の効果。名前から：窓。つまみ：範囲・強さ・色の濃さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **选中框**（枠・有料）：画面の効果。名前から：枠 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **音乐播放器**（枠・有料）：画面の効果。名前から：プレーヤー。つまみ：横ずれ・縦ずれ・背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **音乐播放器 II**（枠・有料）：画面の効果。名前から：プレーヤー。つまみ：強さ・大きさ・横ずれ・縦ずれ・ぼかし・背景の動き — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **飞舞的照片**（枠・有料）：画面の効果。名前から：写真・飛ぶ。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける）・キー x・y — 借りる先：見つからない
- [ ] **黑胶边框**（枠・有料）：画面の効果。名前から：枠。つまみ：質感 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **鼠标选中**（枠・有料）：画面の効果。名前から：マウス。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **鼠标选择**（枠・有料）：画面の効果。名前から：マウス。つまみ：速さ — いま：形の stroke・四角（枠は形で描ける） — 借りる先：見つからない

### ぼかし（58）

- [ ] **Back to Focus**（ぼかし・有料）：画面の効果。名前から：ピント。つまみ：速さ・回転・範囲・ぼかし・明るさ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Backlit Focus**（ぼかし・有料）：画面の効果。名前から：ピント合わせ・光・ピント（中国語名 逆光对焦）。つまみ：やわらかさ・速さ・ぼかし — いま：段の focus・aperture＋物の blur をキー・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Blackout**（ぼかし・有料）：画面の効果。名前から：ぼかし・光る（フラッシュ）（中国語名 表面模糊）。つまみ：ぼかし・シャープ — いま：物の blur・ISF Fast Blur・段の exposure・opacity をキー — 借りる先：ISF Fade Color（MIT）
- [ ] **Blur**（ぼかし）：画面の効果。名前から：ぼかし（中国語名 模糊）。つまみ：ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **Blur**（ぼかし・有料）：画面の効果。名前から：ぼかし・一瞬（中国語名 瞬间模糊）。つまみ：ぼかし・範囲 — いま：物の blur・ISF Fast Blur・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **Blurry Focus**（ぼかし）：画面の効果。名前から：ぼかし・幕が開く・ピント（中国語名 模糊开幕）。つまみ：速さ・ぼかし — いま：物の blur・ISF Fast Blur・段の focus・aperture＋物の blur をキー — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **Camera Focus 2**（ぼかし）：画面の効果。名前から：ピント。つまみ：速さ・ぼかし — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Curvy Blur**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：速さ・明るさ・強さ・回転・横ずれ・縦ずれ・ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **Defrost**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：速さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **Focus Shake**（ぼかし・有料）：画面の効果。名前から：ピント・揺れ。つまみ：強さ・明るさ・ぼかし・速さ — いま：段の focus・aperture＋物の blur をキー・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **Free Focus**（ぼかし・有料）：画面の効果。名前から：ピント。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Grain**（ぼかし）：画面の効果。名前から：すりガラス・粒・ノイズ（中国語名 磨砂纹理）。つまみ：質感 — いま：物の blur・ISF Fast Blur・grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：見つからない
- [ ] **Hazy**（ぼかし）：画面の効果。名前から：ぼかし・光のにじみ・夢っぽい（中国語名 虚化）。つまみ：ぼかし・横ずれ・縦ずれ — いま：物の blur・ISF Fast Blur・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：見つからない
- [ ] **Motion Blur**（ぼかし）：画面の効果。名前から：動きのぶれ・ぼかし（中国語名 动感模糊）。つまみ：横ずれ・強さ — いま：ISF VVMotionBlur 3.0・物の blur・ISF Fast Blur — 借りる先：GPUImage MotionBlurFilter（BSD-3、名前が同じ）
- [ ] **Pixel Blur**（ぼかし・有料）：画面の効果。名前から：ぼかし・粒子・モザイク・ピクセル（中国語名 粒子模糊）。つまみ：ノイズ・速さ — いま：物の blur・ISF Fast Blur・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack（CC0）
- [ ] **Rotary Focus**（ぼかし・有料）：画面の効果。名前から：ピント（中国語名 旋焦）。つまみ：強さ・速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **Spinning Focus**（ぼかし）：画面の効果。名前から：ピント・回転。つまみ：強さ・ぼかし・明るさ・速さ — いま：段の focus・aperture＋物の blur をキー・キー rotate — 借りる先：見つからない
- [ ] **Vertical Blur**（ぼかし）：画面の効果。名前から：ぼかし（中国語名 纵向模糊）。つまみ：ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：three.js shaders VerticalBlurShader（MIT、名前が同じ）
- [ ] **交叉模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：速さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **侧滑模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：明るさ・速さ・ぼかし・色の濃さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **倾斜模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・傾く。つまみ：速さ — いま：物の blur・ISF Fast Blur・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **力量模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **动感虚化**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：色・明るさ・強さ・範囲 — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **十字形模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・字。つまみ：速さ・横ずれ・縦ずれ・明るさ・範囲・ぼかし — いま：物の blur・ISF Fast Blur・animators print_on・print_off — 借りる先：見つからない
- [ ] **坚持模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：強さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **失去焦点**（ぼかし・有料）：画面の効果。名前から：焦点。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **失焦**（ぼかし）：画面の効果。名前から：ピンぼけ。つまみ：ぼかし・色の濃さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **弹跳聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ・弾む。つまみ：強さ — いま：段の focus・aperture＋物の blur をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **快速模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：ぼかし・速さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **抖动模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・揺れ。つまみ：色・色の濃さ・ぼかし・範囲・強さ・回転 — いま：物の blur・ISF Fast Blur・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **拼图聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ・パズル。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー・ISF Random Squares Mask・Triangle Mosaic — 借りる先：見つからない
- [ ] **收缩焦点**（ぼかし・有料）：画面の効果。名前から：焦点。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **斜向模糊**（ぼかし）：画面の効果。名前から：ぼかし・斜め。つまみ：ぼかし・回転 — いま：物の blur・ISF Fast Blur・キー scale_x・scale_y・skew — 借りる先：見つからない
- [ ] **方形模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：ぼかし・回転・強さ・色・明るさ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **旋转与重聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ・回転。つまみ：ぼかし — いま：段の focus・aperture＋物の blur をキー・キー rotate — 借りる先：見つからない
- [ ] **旋转模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・回転。つまみ：回転・大きさ・範囲・明るさ・速さ・横ずれ — いま：物の blur・ISF Fast Blur・キー rotate — 借りる先：見つからない
- [ ] **模糊旋转**（ぼかし・有料）：画面の効果。名前から：ぼかし・回転。つまみ：速さ — いま：物の blur・ISF Fast Blur・キー rotate — 借りる先：見つからない
- [ ] **模糊星光**（ぼかし）：画面の効果。名前から：ぼかし・星の光。つまみ：速さ・ぼかし・横の色ずれ・縦の色ずれ — いま：物の blur・ISF Fast Blur・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **模糊特写**（ぼかし・有料）：画面の効果。名前から：ぼかし・アップ。つまみ：速さ — いま：物の blur・ISF Fast Blur・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **模糊闭幕**（ぼかし・有料）：画面の効果。名前から：ぼかし・幕が閉じる。つまみ：速さ・質感 — いま：物の blur・ISF Fast Blur — 借りる先：gl-transitions HorizontalOpen・VerticalOpen（MIT）
- [ ] **模糊震动**（ぼかし・有料）：画面の効果。名前から：ぼかし・振動。つまみ：速さ・質感 — いま：物の blur・ISF Fast Blur・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **滑稽聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **爱心模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・ハート。つまみ：大きさ・強さ・速さ・横ずれ・背景の動き — いま：物の blur・ISF Fast Blur — 借りる先：ISF Heart（MIT）＋物の copies・scatter
- [ ] **画布模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：速さ・強さ — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **磨砂水晶**（ぼかし・有料）：画面の効果。名前から：すりガラス・水晶。つまみ：縦ずれ・横ずれ・速さ・回転・色の濃さ — いま：物の blur・ISF Fast Blur・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **竖闪模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・光る。つまみ：強さ・明るさ・速さ — いま：物の blur・ISF Fast Blur・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：ISF Strobe（MIT）
- [ ] **粒子模糊 II**（ぼかし・有料）：画面の効果。名前から：ぼかし・粒子。つまみ：ぼかし・速さ・横ずれ・縦ずれ — いま：物の blur・ISF Fast Blur・点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：Kenney Particle Pack（CC0）
- [ ] **精选聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ。つまみ：速さ・ぼかし — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **红边模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・赤。つまみ：ぼかし・強さ・色の濃さ — いま：物の blur・ISF Fast Blur・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **脉动聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ。つまみ：速さ・明るさ・ぼかし — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **花朵模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・花 — いま：物の blur・ISF Fast Blur — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter
- [ ] **超级聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ。つまみ：数・強さ・大きさ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない
- [ ] **超速模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし。つまみ：速さ・ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：見つからない
- [ ] **迷幻模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・サイケ。つまみ：強さ・ぼかし・明るさ・速さ・横ずれ・縦ずれ — いま：物の blur・ISF Fast Blur・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Color Phase（MIT）
- [ ] **镜头模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・カメラ。つまみ：範囲・ぼかし・色の濃さ・速さ — いま：物の blur・ISF Fast Blur・キー scale・ISF Zoom — 借りる先：見つからない
- [ ] **闪亮亮焦点**（ぼかし・有料）：画面の効果。名前から：焦点・きらめく。つまみ：速さ — いま：段の focus・aperture＋物の blur をキー・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **震动模糊**（ぼかし・有料）：画面の効果。名前から：ぼかし・振動。つまみ：速さ・ゆがみ・ぼかし・強さ — いま：物の blur・ISF Fast Blur・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）
- [ ] **黑色聚焦**（ぼかし・有料）：画面の効果。名前から：ピント合わせ — いま：段の focus・aperture＋物の blur をキー — 借りる先：見つからない

### 残像（41）

- [ ] **36倍复制**（残像・有料）：画面の効果。名前から：複製。つまみ：強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **9克隆介绍**（残像・有料）：画面の効果。名前から：クローン。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **Delay**（残像・有料）：画面の効果。名前から：残像・分身（中国語名 迷离）。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **Ghosting**（残像・有料）：画面の効果。名前から：残像・分身。つまみ：明るさ・やわらかさ・ぼかし・強さ・速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF-Files Ghosting（MIT、名前が同じ）／ISF Ghosting（MIT）
- [ ] **Rainbow Haze**（残像）：画面の効果。名前から：幻影・虹・光のにじみ・夢っぽい（中国語名 彩虹幻影）。つまみ：速さ・色の濃さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Ghosting（MIT）／ISF Color Phase（MIT）
- [ ] **Train Window**（残像・有料）：画面の効果。名前から：影・窓・枠・滑る（中国語名 车窗影）。つまみ：速さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・形の stroke・四角（枠は形で描ける） — 借りる先：ISF Ghosting（MIT）
- [ ] **Tree Shade**（残像・有料）：画面の効果。名前から：影（中国語名 树影）。つまみ：速さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **Tree Shade 2**（残像・有料）：画面の効果。名前から：影（中国語名 树影 II）。つまみ：速さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **Unreal**（残像・有料）：画面の効果。名前から：幻影（中国語名 幻影）。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **克隆排位**（残像・有料）：画面の効果。名前から：クローン。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **南瓜幽灵**（残像・有料）：画面の効果。名前から：幽霊・かぼちゃ。つまみ：速さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **复制剪切**（残像・有料）：画面の効果。名前から：複製。つまみ：大きさ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **复制堆叠**（残像・有料）：画面の効果。名前から：複製・積み重ね。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・rotate_x・rotate_y・z（2.5D） — 借りる先：ISF Ghosting（MIT）
- [ ] **平滑复制**（残像・有料）：画面の効果。名前から：複製。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **幻影 II**（残像）：画面の効果。名前から：幻影。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **幻影缩放**（残像・有料）：画面の効果。名前から：幻影・拡大縮小。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・キー scale・ISF Zoom — 借りる先：ISF Ghosting（MIT）
- [ ] **幻影节拍**（残像・有料）：画面の効果。名前から：幻影・拍子。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ふるまい oscillator を scale に — 借りる先：ISF Ghosting（MIT）
- [ ] **幽灵漂浮**（残像）：画面の効果。名前から：幽霊・浮かぶ。つまみ：速さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・キー x・y — 借りる先：ISF Ghosting（MIT）
- [ ] **幽灵鬼影**（残像・有料）：画面の効果。名前から：幽霊・影。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **彩光幻影**（残像・有料）：画面の効果。名前から：幻影・光。つまみ：速さ・明るさ・ゆがみ・色 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Ghosting（MIT）
- [ ] **彩屏重影**（残像・有料）：画面の効果。名前から：二重像。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **彩色拖影**（残像・有料）：画面の効果。名前から：残像・カラー。つまみ：強さ・色・ぼかし・範囲 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Ghosting（MIT）／ISF Color Phase（MIT）
- [ ] **影印**（残像・有料）：画面の効果。名前から：影。つまみ：数・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **慢动作回声**（残像・有料）：画面の効果。名前から：こだま（残像）。つまみ：速さ・大きさ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **找到的影像**（残像・有料）：画面の効果。名前から：影。つまみ：シャープ・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **拖尾震动**（残像・有料）：画面の効果。名前から：尾を引く・振動。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ふるまい noise を x・y に — 借りる先：ISF Ghosting（MIT）／ISF Shake（MIT）
- [ ] **拖拽残影**（残像・有料）：画面の効果。名前から：残像。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **摇晃叠影**（残像・有料）：画面の効果。名前から：重なる像・揺れ。つまみ：速さ・強さ・色の濃さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ふるまい noise を x・y に — 借りる先：ISF Ghosting（MIT）／ISF Shake（MIT）
- [ ] **暗黑剪影**（残像）：画面の効果。名前から：影・暗い。つまみ：色の濃さ・横の色ずれ・縦の色ずれ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・vignette.wgsl・段の vignette — 借りる先：ISF Ghosting（MIT）
- [ ] **朦胧幽影**（残像・有料）：画面の効果。名前から：影。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **未排序的复制**（残像・有料）：画面の効果。名前から：複製。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **梦幻投影**（残像・有料）：画面の効果。名前から：影・夢。つまみ：強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：ISF Ghosting（MIT）
- [ ] **流动回声**（残像・有料）：画面の効果。名前から：こだま（残像）・流れる。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Noise Displace・形の warp — 借りる先：ISF Ghosting（MIT）
- [ ] **灵魂出窍**（残像・有料）：画面の効果。名前から：魂が抜ける（拡大した写しを半透明で重ねる）。つまみ：強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **灵魂出窍 II**（残像・有料）：画面の効果。名前から：魂が抜ける（拡大した写しを半透明で重ねる）。つまみ：速さ・強さ・明るさ・ゆがみ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **琉璃拖影**（残像）：画面の効果。名前から：残像。つまみ：色の濃さ・範囲・強さ・大きさ・色 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **色偏复制**（残像・有料）：画面の効果。名前から：複製 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **蓝调拖影**（残像・有料）：画面の効果。名前から：残像・青。つまみ：速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：ISF Ghosting（MIT）／ISF Color Phase（MIT）
- [ ] **负片拖影**（残像・有料）：画面の効果。名前から：残像・ネガ。つまみ：速さ・色の濃さ・強さ・範囲 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Color Invert・RGB Invert — 借りる先：ISF Ghosting（MIT）
- [ ] **重叠复制**（残像・有料）：画面の効果。名前から：複製。つまみ：強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **鬼影设备**（残像・有料）：画面の効果。名前から：影。つまみ：シャープ・ぼかし — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）

### 絵の具（38）

- [ ] **Cartoon Border**（絵の具・有料）：画面の効果。名前から：漫画・枠・縁取り・アニメ（中国語名 漫画边框）。つまみ：速さ・ぼかし — いま：形の stroke・四角（枠は形で描ける）・ISF Edges・Edge Blowout＋Glow — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **Dot Silkscreen**（絵の具）：画面の効果。名前から：網点（中国語名 网点丝印）。つまみ：質感・色の濃さ・色・大きさ — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone — 借りる先：見つからない
- [ ] **Graphic 3**（絵の具・有料）：画面の効果。名前から：手描き・枠（中国語名 手绘边框 III） — いま：形の stroke・四角（枠は形で描ける） — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **Splice**（絵の具・有料）：画面の効果。名前から：漫画・格子（中国語名 三格漫画） — いま：ISF Random Squares Mask・Triangle Mosaic — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **分屏涂鸦**（絵の具）：画面の効果。名前から：落書き・分割画面。つまみ：強さ・飾り・背景の動き・色の濃さ・明るさ・速さ — いま：ISF Quad Tile — 借りる先：ISF Doodler（MIT）／ISF MultiFrame 2x2・3x3（MIT）
- [ ] **手绘星星**（絵の具・有料）：画面の効果。名前から：手描き・星 — いま：ISF Star（1 個） — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **报纸图框**（絵の具・有料）：画面の効果。名前から：新聞・枠。つまみ：速さ — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **水彩游戏**（絵の具・有料）：画面の効果。名前から：水彩・ゲーム。つまみ：速さ — いま：crt_finish.wgsl・ISF Bad TV — 借りる先：GPUImage3 KuwaharaFilter（BSD-3）／ISF VHS Glitch・v002-CRT-Mask（MIT）
- [ ] **波点彩绘**（絵の具・有料）：画面の効果。名前から：水玉。つまみ：速さ・大きさ・色 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone — 借りる先：見つからない
- [ ] **浮雕边框**（絵の具・有料）：画面の効果。名前から：浮き彫り・枠。つまみ：範囲・強さ — いま：ISF Emboss・形の stroke・四角（枠は形で描ける） — 借りる先：見つからない
- [ ] **涂鸦蓝**（絵の具・有料）：画面の効果。名前から：落書き・青。つまみ：速さ・横ずれ・縦ずれ・大きさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Doodler（MIT）／ISF Color Phase（MIT）
- [ ] **粉笔涂鸦**（絵の具・有料）：画面の効果。名前から：落書き・ピンク。つまみ：ノイズ・背景の動き・明るさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Doodler（MIT）／ISF Color Phase（MIT）
- [ ] **粉笔素描**（絵の具・有料）：画面の効果。名前から：鉛筆画・ピンク。つまみ：速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）／ISF Color Phase（MIT）
- [ ] **网点闪亮**（絵の具・有料）：画面の効果。名前から：網点・きらめく。つまみ：速さ — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone・ISF Star（1 個） — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **黑噪涂鸦**（絵の具・有料）：画面の効果。名前から：落書き・ノイズ — いま：grain.wgsl・fine_grain.wgsl・ISF Noise — 借りる先：ISF Doodler（MIT）
- [ ] **Painting**（絵の具）：画面の効果。名前から：油絵・絵画（中国語名 油画纹理）。つまみ：質感・色の濃さ — いま：無い — 借りる先：GPUImage3 KuwaharaFilter（BSD-3）
- [ ] **Trendy Graffiti**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：強さ・速さ・質感 — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **丝印涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：色・強さ・大きさ・背景の動き・数・速さ・質感 — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **地铁涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **布艺涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ・質感 — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **恐怖涂鸦**（絵の具・有料）：画面の効果。名前から：落書き・ホラー。つまみ：色・強さ・速さ・色の濃さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **手绘小爱心**（絵の具・有料）：画面の効果。名前から：手描き・ハート。つまみ：速さ — いま：無い — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）／ISF Heart（MIT）＋物の copies・scatter
- [ ] **时尚涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ・強さ・色・質感 — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **油画**（絵の具）：画面の効果。名前から：油絵 — いま：無い — 借りる先：GPUImage3 KuwaharaFilter（BSD-3）
- [ ] **法式涂鸦**（絵の具）：画面の効果。名前から：落書き。つまみ：色・範囲・色の濃さ・数・大きさ・質感・速さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **活泼涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ・横ずれ・縦ずれ・大きさ・背景の動き — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **涂鸦揭晓**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **涂鸦日记**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：色の濃さ・ノイズ・速さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **涂鸦日记 I**（絵の具）：画面の効果。名前から：落書き。つまみ：質感・色・速さ・飾り・色の濃さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **涂鸦黄色**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ・大きさ・横ずれ・縦ずれ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **漫画冲刺**（絵の具・有料）：画面の効果。名前から：漫画。つまみ：速さ — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）
- [ ] **漫画分割**（絵の具・有料）：画面の効果。名前から：漫画・分割。つまみ：速さ — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）／ISF Sliding Strips（MIT）
- [ ] **线条涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：質感・速さ・色の濃さ — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **蜡笔复刻**（絵の具・有料）：画面の効果。名前から：クレヨン。つまみ：横ずれ・縦ずれ — いま：無い — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **蜡笔趣味**（絵の具・有料）：画面の効果。名前から：クレヨン。つまみ：速さ — いま：無い — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **街头涂鸦**（絵の具・有料）：画面の効果。名前から：落書き・頭 — いま：無い — 借りる先：ISF Doodler（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **西海岸涂鸦**（絵の具・有料）：画面の効果。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）
- [ ] **铅笔涂鸦**（絵の具・有料）：画面の効果。名前から：落書き。つまみ：速さ — いま：無い — 借りる先：ISF Doodler（MIT）

### 人（6）

- [ ] **粉色眨眼**（人・有料）：画面の効果。名前から：目・ピンク。つまみ：強さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion／ISF Color Phase（MIT）
- [ ] **Doll**（人）：画面の効果。名前から：顔・人 — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Invisible Person**（人）：画面の効果。名前から：透明人間（中国語名 隐形人）。つまみ：強さ・色の濃さ・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）
- [ ] **猫眼凹槽**（人・有料）：画面の効果。名前から：目。つまみ：強さ・大きさ — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **隐形现身**（人・有料）：画面の効果。名前から：透明人間。つまみ：速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）
- [ ] **隐身人**（人・有料）：画面の効果。名前から：透明人間。つまみ：強さ・速さ・縦ずれ・横ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）

### 文字（6）

- [ ] **倒计时**（文字・有料）：画面の効果。名前から：カウントダウン — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **倒计时 II**（文字）：画面の効果。名前から：カウントダウン。つまみ：ぼかし・速さ — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **倒计时 III**（文字・有料）：画面の効果。名前から：カウントダウン。つまみ：速さ・ぼかし・大きさ — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **投影文字**（文字・有料）：画面の効果。名前から：文字・影。つまみ：やわらかさ・色の濃さ・ぼかし・大きさ・ゆがみ・速さ — いま：animators print_on・print_off・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：ISF Ghosting（MIT）
- [ ] **遮幕倒计时**（文字・有料）：画面の効果。名前から：カウントダウン — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **震撼倒计时**（文字・有料）：画面の効果。名前から：カウントダウン・揺れ。つまみ：速さ — いま：animators print_on・print_off・ふるまい noise を x・y に — 借りる先：ISF Shake（MIT）

### その他（102）

- [ ] **B&W 电气**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・色の濃さ・色・明るさ・質感・強さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **Blinds**（その他）：画面の効果。名前から：ブラインド（中国語名 百叶窗 I）。つまみ：背景の動き — いま：無い — 借りる先：見つからない
- [ ] **Blinds 2**（その他・有料）：画面の効果。名前から：ブラインド（中国語名 百叶窗 II）。つまみ：背景の動き — いま：無い — 借りる先：見つからない
- [ ] **CD播放**（その他）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Edgy**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・横の色ずれ・縦の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **Horror 画面**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **MV封面**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：ぼかし・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **OH**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Oh My God**（その他）：画面の効果。名前だけでは中身が分からない（中国語名 天啊） — いま：無い — 借りる先：見つからない
- [ ] **Passion**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **Question Marks**（その他）：画面の効果。名前だけでは中身が分からない（中国語名 满屏问号） — いま：無い — 借りる先：見つからない
- [ ] **The End**（その他）：画面の効果。名前だけでは中身が分からない（中国語名 全剧终） — いま：無い — 借りる先：見つからない
- [ ] **T形展示**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **WOW**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **一刀两断**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：色・強さ・大きさ・明るさ・速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **一键三连**（その他）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **三层展示**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **三连步**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ — いま：無い — 借りる先：見つからない
- [ ] **不和谐**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：強さ — いま：無い — 借りる先：見つからない
- [ ] **交叉置换**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **人行道混搭**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・質感 — いま：無い — 借りる先：見つからない
- [ ] **低保真**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：明るさ・ノイズ・色・色の濃さ・ぼかし・シャープ — いま：無い — 借りる先：見つからない
- [ ] **倍增**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：色 — いま：無い — 借りる先：見つからない
- [ ] **倒带**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・横の色ずれ・ぼかし — いま：無い — 借りる先：見つからない
- [ ] **倒置点**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：強さ — いま：無い — 借りる先：見つからない
- [ ] **催眠**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **入侵**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ・数 — いま：無い — 借りる先：見つからない
- [ ] **入冬**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **凌乱踩踏**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **别致电梯**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：横ずれ — いま：無い — 借りる先：見つからない
- [ ] **剪贴**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・質感 — いま：無い — 借りる先：見つからない
- [ ] **剪辑模拟**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **动态肖像**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：色の濃さ・ぼかし — いま：無い — 借りる先：見つからない
- [ ] **半色调反向**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **卷动**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **合集**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **名片**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **四点平移**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **地图剪切**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・質感・大きさ — いま：無い — 借りる先：見つからない
- [ ] **地狱使者**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：色・速さ — いま：無い — 借りる先：見つからない
- [ ] **城市交通**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **塑料包裹**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **塑料封面**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・質感 — いま：無い — 借りる先：見つからない
- [ ] **塑料封面 II**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：質感 — いま：無い — 借りる先：見つからない
- [ ] **塑料封面 III**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：質感 — いま：無い — 借りる先：見つからない
- [ ] **夕阳**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **多种大小**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・色の濃さ・質感 — いま：無い — 借りる先：見つからない
- [ ] **多边形狂欢**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **天堂之阳**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：背景の動き — いま：無い — 借りる先：見つからない
- [ ] **导演剪辑版**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：ノイズ — いま：無い — 借りる先：見つからない
- [ ] **小冬日**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **小熊软糖**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **层压效果**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・大きさ — いま：無い — 借りる先：見つからない
- [ ] **循环训练**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **恐慌频率**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：色の濃さ・ぼかし — いま：無い — 借りる先：見つからない
- [ ] **情感分离**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・ぼかし・横ずれ・縦ずれ — いま：無い — 借りる先：見つからない
- [ ] **手写日记**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：色・大きさ・速さ・色の濃さ・背景の動き — いま：無い — 借りる先：見つからない
- [ ] **播放预览**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **数学公式**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・大きさ・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **斑斓**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **旗帜横幅**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ — いま：無い — 借りる先：見つからない
- [ ] **无法修复**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **未来揭示**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ — いま：無い — 借りる先：見つからない
- [ ] **治疗通话**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：強さ — いま：無い — 借りる先：見つからない
- [ ] **法式暖调**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・範囲・強さ・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **法式浪漫**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：質感・範囲・強さ・色の濃さ・やわらかさ — いま：無い — 借りる先：見つからない
- [ ] **涂绘嘉年华**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・色の濃さ・シャープ — いま：無い — 借りる先：見つからない
- [ ] **深度出现**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ・質感・色・ぼかし — いま：無い — 借りる先：見つからない
- [ ] **滚筒卷动**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・大きさ・強さ・シャープ — いま：無い — 借りる先：見つからない
- [ ] **牌组洗牌**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **现实崩塌**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **电子模拟器**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **电子炫目**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・やわらかさ・明るさ — いま：無い — 借りる先：見つからない
- [ ] **电气妖气**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・明るさ — いま：無い — 借りる先：見つからない
- [ ] **疯狂搜索**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **瘫软在地**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：質感・色・範囲・大きさ・速さ — いま：無い — 借りる先：見つからない
- [ ] **白色凹槽**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：明るさ・ぼかし — いま：無い — 借りる先：見つからない
- [ ] **白鸽**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・速さ・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **目眩酷热**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・強さ — いま：無い — 借りる先：見つからない
- [ ] **直播表情**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **磁力幻象**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **粗糙墙面**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **细节展示**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **细节灯照**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **织布机编织**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：質感 — いま：無い — 借りる先：見つからない
- [ ] **缓慢前进**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **缩略图导入**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・明るさ — いま：無い — 借りる先：見つからない
- [ ] **美式**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・シャープ・横の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **美式 II**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・横の色ずれ・縦の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **美式 III**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・横の色ずれ・縦の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **美式 IV**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・シャープ・横の色ずれ — いま：無い — 借りる先：見つからない
- [ ] **美式 V**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・速さ — いま：無い — 借りる先：見つからない
- [ ] **自然 II**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ・回転 — いま：無い — 借りる先：見つからない
- [ ] **自然 V**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：大きさ・数・色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **艺术背景**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **轻流**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ・明るさ・大きさ — いま：無い — 借りる先：見つからない
- [ ] **重叠世界**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：速さ — いま：無い — 借りる先：見つからない
- [ ] **错位**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：明るさ — いま：無い — 借りる先：見つからない
- [ ] **难过 II**（その他・有料）：画面の効果。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **零点解锁**（その他）：画面の効果。名前だけでは中身が分からない。つまみ：ぼかし — いま：無い — 借りる先：見つからない
- [ ] **频繁衰变**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：強さ・ノイズ — いま：無い — 借りる先：見つからない
- [ ] **鸽洞**（その他・有料）：画面の効果。名前だけでは中身が分からない。つまみ：ノイズ・強さ — いま：無い — 借りる先：見つからない

## Body effects（人の効果）（260）

Motolii に有る 0・近い物あり 208・見つからない 52。

### 光（74）

- [ ] **Aura**（光）：人に合わせる効果。名前から：雰囲気・光る・人の縁（中国語名 背景氛围II）。つまみ：速さ・色 — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Bright Idea**（光）：人に合わせる効果。名前から：光る。つまみ：速さ・背景の動き・飾り・大きさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Electric Storm**（光）：人に合わせる効果。名前から：電気の光・雰囲気・稲妻・電気（中国語名 电光氛围）。つまみ：速さ・色 — いま：ISF Twirl — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Electro Border**（光）：人に合わせる効果。名前から：縁取り・稲妻・電気。つまみ：色・速さ・大きさ・強さ・色の濃さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Eye Reflection**（光）：人に合わせる効果。名前から：光・目・鏡・顔（中国語名 眼神光）。つまみ：強さ・範囲・色 — いま：段の bloom・物の glow・ISF Bloom・Glow・mirror.wgsl — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Flame Outline**（光）：人に合わせる効果。名前から：縁取り・炎・人の縁（中国語名 火焰描边 I）。つまみ：色・範囲・強さ・速さ・色の濃さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Fluoro Flash**（光）：人に合わせる効果。名前から：蛍光・画面が光る・光る（フラッシュ）（中国語名 荧光闪屏）。つまみ：色の濃さ・速さ・飾り・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow・段の exposure・opacity をキー — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Fade Color（MIT）
- [ ] **Frame Scatter 2**（光）：人に合わせる効果。名前から：縁取り・枠（中国語名 描边发散 II）。つまみ：速さ・範囲・色 — いま：ISF Edges・Edge Blowout＋Glow・形の stroke・四角（枠は形で描ける） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Geometric Lasers**（光・有料）：人に合わせる効果。名前から：レーザー・光が走る・スキャン（中国語名 激光几何）。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Halo**（光）：人に合わせる効果。名前から：光の輪・光のにじみ・夢っぽい（中国語名 光环 I）。つまみ：縦ずれ・大きさ・色 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Halo 2**（光）：人に合わせる効果。名前から：光の輪・光のにじみ・夢っぽい（中国語名 光环 II）。つまみ：縦ずれ・大きさ・色 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Light Trails**（光・有料）：人に合わせる効果。名前から：光る・影・残像・分身（中国語名 闪影）。つまみ：強さ・ぼかし・速さ — いま：ISF RGB Strobe・ふるまい random を opacity に・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Strobe（MIT）／ISF Ghosting（MIT）
- [ ] **Lightning Bound**（光・有料）：人に合わせる効果。名前から：稲妻・周りを回る・電気（中国語名 闪电环绕）。つまみ：速さ・縦ずれ・大きさ・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Lightning Eyes**（光）：人に合わせる効果。名前から：稲妻・目・電気・顔（中国語名 闪电眼）。つまみ：色・明るさ・色の濃さ・強さ・範囲 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Neon Doodle**（光）：人に合わせる効果。名前から：ネオン・落書き（中国語名 霓虹涂鸦）。つまみ：大きさ・速さ・縦ずれ・横ずれ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）／ISF Doodler（MIT）
- [ ] **Neon Motion**（光・有料）：人に合わせる効果。名前から：ネオン。つまみ：大きさ・横の色ずれ・範囲 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）
- [ ] **Neon Variation**（光）：人に合わせる効果。名前から：ネオン・グラデーション（中国語名 霓虹渐变）。つまみ：速さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）／ISF Color Phase（MIT）
- [ ] **Radiate Bolts**（光）：人に合わせる効果。名前から：放射・電気の光・光る・稲妻（中国語名 电光放射）。つまみ：色・縦ずれ・大きさ — いま：ISF God Rays・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Scan**（光）：人に合わせる効果。名前から：光が走る・スキャン。つまみ：範囲・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Scan 2**（光・有料）：人に合わせる効果。名前から：スキャン・光が走る（中国語名 扫描 II）。つまみ：範囲・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Starlight**（光）：人に合わせる効果。名前から：放射・星の光・きらきら・星（中国語名 星光放射）。つまみ：速さ・大きさ・色 — いま：ISF God Rays・ISF Star（1 個） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **Streamer Stroke**（光）：人に合わせる効果。名前から：流れる光・縁取り（中国語名 流光描边）。つまみ：強さ・大きさ・範囲・速さ・背景の動き・色 — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **Thermal Aura**（光）：人に合わせる効果。名前から：光・熱カメラ・光る・人の縁（中国語名 热力光谱）。つまみ：背景の動き・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Thermal Camera — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Night Vision（MIT）
- [ ] **Thermal Aura 2**（光）：人に合わせる効果。名前から：光・熱カメラ・光る・人の縁（中国語名 热力光谱 Ⅱ）。つまみ：背景の動き・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Thermal Camera — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Night Vision（MIT）
- [ ] **Tyndall Effect**（光）：人に合わせる効果。名前から：光の筋（中国語名 丁达尔效应）。つまみ：強さ・色・色の濃さ — いま：ISF God Rays — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **X瞬移**（光・有料）：人に合わせる効果。名前から：瞬間移動。つまみ：速さ・明るさ・横ずれ・強さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Strobe（MIT）
- [ ] **人影爆闪**（光）：人に合わせる効果。名前から：強く光る・影。つまみ：速さ・色・明るさ・強さ・背景の動き — いま：段の exposure・opacity をキー・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Fade Color（MIT）／ISF Ghosting（MIT）
- [ ] **人物发光I**（光・有料）：人に合わせる効果。名前から：光る・人物 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Image Segmenter（Apache-2.0）
- [ ] **发光分身**（光・有料）：人に合わせる効果。名前から：光る・分身。つまみ：色・強さ・横ずれ・大きさ・ゆがみ — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **发光拖影**（光・有料）：人に合わせる効果。名前から：光る・残像。つまみ：強さ・ゆがみ・色・数・明るさ・範囲 — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **发光标记**（光・有料）：人に合わせる効果。名前から：光る。つまみ：速さ・大きさ・色・明るさ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **发光溶解**（光・有料）：人に合わせる効果。名前から：光る・溶ける。つまみ：明るさ・色・ゆがみ・大きさ・速さ・背景の動き — いま：段の bloom・物の glow・ISF Bloom・Glow・点群の dissolve・ISF Noise — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **发光贴纸**（光・有料）：人に合わせる効果。名前から：光る・紙。つまみ：数・色の濃さ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **基础发光**（光・有料）：人に合わせる効果。名前から：光る。つまみ：色・範囲・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **天使**（光・有料）：人に合わせる効果。名前から：天使。つまみ：質感・大きさ・数・色の濃さ・強さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **天使光环**（光・有料）：人に合わせる効果。名前から：光の輪・天使。つまみ：大きさ・横ずれ・強さ・数・色の濃さ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **天使环**（光・有料）：人に合わせる効果。名前から：天使 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **天使翅膀**（光）：人に合わせる効果。名前から：天使・翼。つまみ：縦ずれ・大きさ・色 — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **幻彩流光**（光・有料）：人に合わせる効果。名前から：流れる光・虹色。つまみ：質感・明るさ・強さ・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす／ISF Color Phase（MIT）
- [ ] **弥散流光**（光・有料）：人に合わせる効果。名前から：流れる光。つまみ：速さ・ノイズ・背景の動き・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **彩色蜡笔描边**（光）：人に合わせる効果。名前から：縁取り・カラー・クレヨン。つまみ：色・大きさ — いま：ISF Edges・Edge Blowout＋Glow・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）／ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **彩虹边缘**（光・有料）：人に合わせる効果。名前から：縁・虹。つまみ：強さ・背景の動き・大きさ・回転・明るさ — いま：ISF Edges・Edge Blowout＋Glow・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **手绘描边**（光・有料）：人に合わせる効果。名前から：縁取り・手描き。つまみ：色・大きさ・範囲 — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3）
- [ ] **扫描 I**（光）：人に合わせる効果。名前から：スキャン。つまみ：範囲・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **描边发散 I**（光・有料）：人に合わせる効果。名前から：縁取り。つまみ：速さ・範囲・色 — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **梦境**（光・有料）：人に合わせる効果。名前から：夢の中。つまみ：大きさ・色の濃さ・縦ずれ — いま：段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **涂鸦闪光 2**（光・有料）：人に合わせる効果。名前から：光る・落書き。つまみ：速さ・横ずれ・縦ずれ・大きさ — いま：段の exposure・opacity をキー — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Fade Color（MIT）／ISF Doodler（MIT）
- [ ] **火焰描边II**（光）：人に合わせる効果。名前から：縁取り・炎。つまみ：色・範囲・強さ・速さ・色の濃さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **电光扫描**（光）：人に合わせる効果。名前から：光が走る。つまみ：大きさ・範囲・ゆがみ・色・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **电光描边**（光・有料）：人に合わせる効果。名前から：縁取り・電気の光。つまみ：色・速さ・大きさ・強さ・色の濃さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **电光描边 II**（光）：人に合わせる効果。名前から：縁取り・電気の光。つまみ：明るさ・ゆがみ・大きさ・色・範囲・速さ・強さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **电光灼烧**（光）：人に合わせる効果。名前から：電気の光・焼ける。つまみ：回転・色・強さ・ゆがみ・速さ・大きさ — いま：点群の Burn Away — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Film Burn・Undulating Burn Out（MIT）
- [ ] **电光眼**（光）：人に合わせる効果。名前から：電気の光・目。つまみ：色・範囲・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **瞬移**（光・有料）：人に合わせる効果。名前から：瞬間移動。つまみ：明るさ・速さ・大きさ・強さ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Strobe（MIT）
- [ ] **碎闪边缘**（光・有料）：人に合わせる効果。名前から：縁・光る。つまみ：範囲・背景の動き・大きさ・速さ・明るさ — いま：ISF Edges・Edge Blowout＋Glow・ISF RGB Strobe・ふるまい random を opacity に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Strobe（MIT）
- [ ] **纸质描边**（光）：人に合わせる効果。名前から：縁取り・紙。つまみ：質感・大きさ・範囲・背景の動き — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **线条描边**（光・有料）：人に合わせる効果。名前から：縁取り — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **脸部扫描特效**（光・有料）：人に合わせる効果。名前から：スキャン・顔。つまみ：速さ・明るさ・横ずれ・縦ずれ・大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **赛博朋克 I**（光）：人に合わせる効果。名前から：サイバーパンク。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）
- [ ] **赛博朋克 II**（光）：人に合わせる効果。名前から：サイバーパンク。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）
- [ ] **轮廓扫描**（光・有料）：人に合わせる効果。名前から：スキャン・輪郭。つまみ：色・速さ・回転・やわらかさ・強さ — いま：ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Linear Gradient（MIT）を足し算の重ねで動かす
- [ ] **迷幻光效**（光・有料）：人に合わせる効果。名前から：光・サイケ。つまみ：速さ・質感・色・大きさ・ゆがみ・背景の動き・ぼかし — いま：段の bloom・物の glow・ISF Bloom・Glow・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **闪光放大 III**（光・有料）：人に合わせる効果。名前から：光る・拡大。つまみ：速さ・強さ・明るさ・ゆがみ — いま：段の exposure・opacity をキー・キー scale・ISF Zoom — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Fade Color（MIT）
- [ ] **闪烁眼镜**（光・有料）：人に合わせる効果。名前から：点滅・目。つまみ：速さ・大きさ — いま：ISF RGB Strobe・ふるまい random を opacity に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Strobe（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **阳光**（光・有料）：人に合わせる効果。名前から：日の光。つまみ：速さ・大きさ・縦ずれ — いま：ISF God Rays — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **霓虹爱心**（光・有料）：人に合わせる効果。名前から：ネオン・ハート。つまみ：速さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Neon（MIT）／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Crackling**（光）：人に合わせる効果。名前から：稲妻・電気・割れる（中国語名 我麻了）。つまみ：大きさ・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Electric Shock**（光）：人に合わせる効果。名前から：電撃・稲妻・電気（中国語名 电击）。つまみ：速さ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Lightning**（光）：人に合わせる効果。名前から：稲妻・電気（中国語名 闪电）。つまみ：色・速さ・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Technology 2**（光・有料）：人に合わせる効果。名前から：雰囲気（中国語名 科技氛围 II）。つまみ：速さ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **电光耳机**（光・有料）：人に合わせる効果。名前から：電気の光。つまみ：色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **电流 I**（光・有料）：人に合わせる効果。名前から：電流。つまみ：色・速さ・色の濃さ・強さ・背景の動き・ぼかし・ゆがみ・大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **电流 II**（光・有料）：人に合わせる効果。名前から：電流。つまみ：質感・横の色ずれ・強さ・色・範囲・速さ・ぼかし — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **科技氛围 III**（光・有料）：人に合わせる効果。名前から：雰囲気。つまみ：速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

### 粒・飾り（34）

- [ ] **<3 Love**（粒・飾り・有料）：人に合わせる効果。名前から：ハート。つまみ：色・大きさ・飾り・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Bubbles 2**（粒・飾り）：人に合わせる効果。名前から：泡（中国語名 气泡 II）。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack（CC0）
- [ ] **Butterfly Wings**（粒・飾り）：人に合わせる効果。名前から：蝶・翼（中国語名 蝴蝶翅膀）。つまみ：縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Club Hearts**（粒・飾り）：人に合わせる効果。名前から：ハート・パーティー（中国語名 动感爱心）。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Firework**（粒・飾り）：人に合わせる効果。名前から：火・炎・花火・爆発（中国語名 焰火）。つまみ：大きさ・色・速さ — いま：レシピ burst.js・confetti.js・点群 Appear Eruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Flame Eyes**（粒・飾り）：人に合わせる効果。名前から：炎・目・顔・人（中国語名 火焰眼 I）。つまみ：色・範囲・速さ・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Flame Wings 1**（粒・飾り）：人に合わせる効果。名前から：炎・翼・蝶（中国語名 火焰翅膀I）。つまみ：速さ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Flame Wings 2**（粒・飾り・有料）：人に合わせる効果。名前から：炎・翼・蝶（中国語名 火焰翅膀II）。つまみ：色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter／Twemoji の蝶（CC BY 4.0）＋物の copies
- [ ] **Flipped**（粒・飾り）：人に合わせる効果。名前から：ときめき・裏返る（中国語名 怦然心动）。つまみ：速さ・色の濃さ — いま：rotate_x・rotate_y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Heart Background**（粒・飾り・有料）：人に合わせる効果。名前から：ハート（中国語名 爱心背景）。つまみ：大きさ・色・速さ・背景の動き・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Heart Flames**（粒・飾り・有料）：人に合わせる効果。名前から：ハート・火・炎（中国語名 爱心焰火）。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Heart Lenses**（粒・飾り）：人に合わせる効果。名前から：ハート・レンズ・魚眼（中国語名 爱心美瞳）。つまみ：強さ・範囲・色の濃さ — いま：ISF Bump Distortion — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter／GPUImage3 BulgeDistortion・SphereRefraction（BSD-3）
- [ ] **Hellfire**（粒・飾り・有料）：人に合わせる効果。名前から：火・炎（中国語名 狱火）。つまみ：色・速さ・大きさ・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **In My Heart**（粒・飾り・有料）：人に合わせる効果。名前から：ハート（中国語名 心里是你）。つまみ：大きさ・縦ずれ・横ずれ・速さ・背景の動き・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Like**（粒・飾り）：人に合わせる効果。名前から：ハート（中国語名 喜欢）。つまみ：数・横ずれ・縦ずれ・大きさ・色の濃さ・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Lovestruck**（粒・飾り）：人に合わせる効果。名前から：熱愛・ハート（中国語名 热恋）。つまみ：色の濃さ・大きさ・色・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **Loving Gaze**（粒・飾り）：人に合わせる効果。名前から：ハート・打つ・顔・人（中国語名 爱心攻击）。つまみ：色の濃さ・色・速さ・横ずれ・縦ずれ — いま：ふるまい noise を x・y に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter／ISF Shake（MIT）
- [ ] **Motion Pattern**（粒・飾り・有料）：人に合わせる効果。名前から：動く模様。つまみ：数・質感・色 — いま：点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack（CC0）
- [ ] **Revolving Flames**（粒・飾り・有料）：人に合わせる効果。名前から：炎・周りを回る・回転（中国語名 火焰环绕）。つまみ：速さ・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Totem Flames**（粒・飾り・有料）：人に合わせる効果。名前から：炎（中国語名 火焰图腾）。つまみ：速さ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **光溶解1**（粒・飾り・有料）：人に合わせる効果。名前から：溶ける・光。つまみ：ゆがみ・速さ・色・明るさ・大きさ — いま：点群の dissolve・ISF Noise・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **可爱猪**（粒・飾り・有料）：人に合わせる効果。名前から：愛。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **吸收溶解**（粒・飾り・有料）：人に合わせる効果。名前から：溶ける・吸い込む。つまみ：速さ・明るさ・色・ゆがみ — いま：点群の dissolve・ISF Noise・ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **圣诞辣妺**（粒・飾り・有料）：人に合わせる効果。名前から：クリスマス。つまみ：質感・強さ・色の濃さ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Twemoji（CC BY 4.0）の絵を物の copies で
- [ ] **气泡 I**（粒・飾り）：人に合わせる効果。名前から：泡。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack（CC0）
- [ ] **火焰恶魔角**（粒・飾り・有料）：人に合わせる効果。名前から：炎・悪魔。つまみ：色・大きさ・縦ずれ・強さ・速さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **火焰眼II**（粒・飾り・有料）：人に合わせる効果。名前から：炎・目。つまみ：色・大きさ・範囲・速さ・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の炎（CC0）＋物の copies・scatter／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **点赞**（粒・飾り・有料）：人に合わせる効果。名前から：いいね。つまみ：横ずれ・縦ずれ・大きさ・色の濃さ・速さ・数 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **爱心眼**（粒・飾り）：人に合わせる効果。名前から：ハート・目。つまみ：色・大きさ・範囲・速さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **爱心荆棘**（粒・飾り・有料）：人に合わせる効果。名前から：ハート。つまみ：範囲・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Heart（MIT）＋物の copies・scatter
- [ ] **粒子弥散**（粒・飾り・有料）：人に合わせる効果。名前から：粒子。つまみ：横ずれ・速さ — いま：点群の Appear Ripple・Appear Rain・Appear Eruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack（CC0）
- [ ] **萤火**（粒・飾り）：人に合わせる効果。名前から：ホタル。つまみ：縦ずれ・大きさ・色 — いま：ISF Star（1 個） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **金厲麋鹿**（粒・飾り・有料）：人に合わせる効果。名前から：金色。つまみ：質感・色の濃さ・強さ・背景の動き — いま：ISF Star（1 個） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **闪闪骷髅**（粒・飾り・有料）：人に合わせる効果。名前から：きらきら。つまみ：色の濃さ・大きさ・強さ・回転・明るさ — いま：ISF Star（1 個） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter

### 残像（24）

- [ ] **Apparate 1**（残像）：人に合わせる効果。名前から：幻影・残像・分身（中国語名 移形幻影  I）。つまみ：回転・ぼかし・速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Color Fringe**（残像）：人に合わせる効果。名前から：二重像・カラー・色ずれ・虹（中国語名 彩色重影）。つまみ：速さ・強さ・色の濃さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／ISF Color Phase（MIT）
- [ ] **Dopelgangers**（残像）：人に合わせる効果。名前から：分身・残像（中国語名 分身）。つまみ：ゆがみ・速さ・数・強さ・回転 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Dot Shadow**（残像）：人に合わせる効果。名前から：分身・水玉・網点・影（中国語名 波点分身）。つまみ：色・強さ・範囲・横ずれ・色の濃さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Flame Trails**（残像）：人に合わせる効果。名前から：尾を引く・炎・残像・分身（中国語名 火焰拖尾） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／Kenney Particle Pack の炎（CC0）＋物の copies・scatter
- [ ] **Ghost**（残像）：人に合わせる効果。名前から：幽霊・残像・分身（中国語名 幽灵）。つまみ：速さ・色 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Musical Notes**（残像・有料）：人に合わせる効果。名前から：尾を引く（中国語名 音符拖尾） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Musical Notes 2**（残像）：人に合わせる効果。名前から：尾を引く（中国語名 音符拖尾 II） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Phantom**（残像）：人に合わせる効果。名前から：幻影・残像・分身（中国語名 幻影）。つまみ：速さ・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Phantom Face**（残像）：人に合わせる効果。名前から：幻影・顔・残像・分身（中国語名 幻影脸）。つまみ：回転・ぼかし・速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Shape Trails**（残像・有料）：人に合わせる効果。名前から：尾を引く・残像・分身（中国語名 几何拖尾I） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Shape Trails 2**（残像・有料）：人に合わせる効果。名前から：尾を引く・残像・分身（中国語名 几何拖尾II） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Speed Streaks**（残像）：人に合わせる効果。名前から：残像・動きのぶれ（中国語名 背景拖影）。つまみ：横ずれ・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF VVMotionBlur 3.0 — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **Star Trails**（残像）：人に合わせる効果。名前から：尾を引く・星・残像・分身（中国語名 星星拖尾） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Star（1 個） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter
- [ ] **分身 II**（残像）：人に合わせる効果。名前から：分身。つまみ：縦ずれ・大きさ・範囲・速さ・横ずれ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **分身 III**（残像）：人に合わせる効果。名前から：分身。つまみ：速さ・ぼかし・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **幻影平移**（残像・有料）：人に合わせる効果。名前から：幻影。つまみ：回転・速さ・ぼかし・色の濃さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **影分身**（残像・有料）：人に合わせる効果。名前から：分身・影。つまみ：速さ・明るさ・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **拖影分身**（残像・有料）：人に合わせる効果。名前から：残像・分身。つまみ：範囲・色の濃さ・色・速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **旋转分身**（残像・有料）：人に合わせる効果。名前から：分身・回転。つまみ：速さ・範囲・ぼかし・背景の動き・大きさ・数 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **碎片分身**（残像）：人に合わせる効果。名前から：分身・破片。つまみ：範囲・強さ・速さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **移形幻影2**（残像・有料）：人に合わせる効果。名前から：幻影。つまみ：速さ・回転・強さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）
- [ ] **迷幻分身**（残像・有料）：人に合わせる効果。名前から：分身・サイケ。つまみ：速さ・数・ぼかし・回転・明るさ・強さ・大きさ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／ISF Color Phase（MIT）
- [ ] **迷幻拖影**（残像・有料）：人に合わせる効果。名前から：残像・サイケ。つまみ：明るさ・範囲・色の濃さ・背景の動き — いま：ISF Echo Trace・RGB Trails 3.0・形の trail・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）／ISF Color Phase（MIT）

### 動き（20）

- [ ] **Axis Rotation**（動き）：人に合わせる効果。名前から：回転（中国語名 中轴旋转）。つまみ：速さ・回転・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Distorted Whirl**（動き）：人に合わせる効果。名前から：回転・渦・ねじれ・ゆがみ（中国語名 背景旋转）。つまみ：色・強さ・明るさ・ゆがみ・速さ — いま：キー rotate・ISF Twirl — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Hurricane**（動き・有料）：人に合わせる効果。名前から：風・嵐（中国語名 飓风）。つまみ：速さ・色 — いま：キー x・y・ISF Twirl — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Looping Arrows**（動き・有料）：人に合わせる効果。名前から：周りを回る・頭（中国語名 箭头环绕）。つまみ：大きさ・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Mechanical**（動き）：人に合わせる効果。名前から：周りを回る・裏返る（中国語名 机械环绕 II）。つまみ：速さ・縦ずれ・大きさ・色 — いま：キー rotate・rotate_x・rotate_y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Pop-Out**（動き）：人に合わせる効果。名前から：弾む — いま：animators bounce_in・spring_in — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Revolving Text**（動き）：人に合わせる効果。名前から：周りを回る・文字・回転（中国語名 文字环绕）。つまみ：色の濃さ・速さ・色・強さ — いま：キー rotate・animators print_on・print_off — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Sinking**（動き）：人に合わせる効果。名前から：滑る・動く（中国語名 沉沦）。つまみ：範囲・色・速さ — いま：キー x・y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Struck**（動き）：人に合わせる効果。名前から：打つ・稲妻・電気（中国語名 打击）。つまみ：速さ・強さ — いま：ふるまい noise を x・y に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Shake（MIT）
- [ ] **X**（動き・有料）：人に合わせる効果。名前から：衝撃。つまみ：大きさ・縦ずれ・速さ — いま：ふるまい noise を x・y に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Shake（MIT）
- [ ] **单向移动**（動き）：人に合わせる効果。名前から：動く。つまみ：速さ・回転 — いま：キー x・y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **旋转反弹**（動き・有料）：人に合わせる効果。名前から：回転・弾む。つまみ：速さ・大きさ・強さ・数・色・質感 — いま：キー rotate・animators bounce_in・spring_in — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **旋转摆动**（動き・有料）：人に合わせる効果。名前から：振り子・回転。つまみ：速さ・ゆがみ・ぼかし・強さ・大きさ — いま：ふるまい oscillator を rotate に・キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **无限穿越**（動き・有料）：人に合わせる効果。名前から：通り抜ける。つまみ：速さ・数 — いま：キー scale・ISF Zoom — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **机械环绕 I**（動き）：人に合わせる効果。名前から：周りを回る。つまみ：速さ・縦ずれ・大きさ・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **空间扰动**（動き・有料）：人に合わせる効果。名前から：空間。つまみ：速さ・ゆがみ・明るさ・色・強さ — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **立体相册**（動き）：人に合わせる効果。名前から：立体・アルバム — いま：rotate_x・rotate_y・z（2.5D） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **纵轴旋转**（動き・有料）：人に合わせる効果。名前から：回転。つまみ：速さ・回転・色 — いま：キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **老钱风**（動き）：人に合わせる効果。名前から：風 — いま：キー x・y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **飞翔的帽子**（動き）：人に合わせる効果。名前から：飛ぶ。つまみ：速さ・大きさ・色 — いま：キー x・y — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

### 人（20）

- [ ] **Big Head**（人）：人に合わせる効果。名前から：頭を大きく・顔・人（中国語名 大头）。つまみ：速さ・範囲・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Big Mouth**（人）：人に合わせる効果。名前から：口・顔・人（中国語名 大嘴） — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Cool**（人）：人に合わせる効果。名前から：顔・人（中国語名 拽酷）。つまみ：大きさ・縦ずれ・横ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Doll**（人・有料）：人に合わせる効果。名前から：顔・人 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Embarrassed Face**（人）：人に合わせる効果。名前から：顔・人（中国語名 尴尬脸） — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Gorilla Face**（人）：人に合わせる効果。名前から：顔・人（中国語名 猩猩脸） — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Man**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国男性 I）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Man 2**（人）：人に合わせる効果。名前から：顔・人（中国語名 美国男性 II）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Man 3**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国男性 III）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Man 4**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国男性 IV）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Sad**（人）：人に合わせる効果。名前から：顔・人（中国語名 难过）。つまみ：背景の動き・飾り — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Woman**（人）：人に合わせる効果。名前から：顔・人（中国語名 美国女性 I）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Woman 2**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国女性 II）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Woman 3**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国女性 III）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Woman 4**（人・有料）：人に合わせる効果。名前から：顔・人（中国語名 美国女性 IV）。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **委屈脸**（人）：人に合わせる効果。名前から：顔 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **未来眼镜**（人・有料）：人に合わせる効果。名前から：目。つまみ：色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **眼机器**（人・有料）：人に合わせる効果。名前から：目。つまみ：大きさ・横ずれ・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **脸红**（人）：人に合わせる効果。名前から：顔・赤。つまみ：飾り・背景の動き・大きさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion／ISF Color Phase（MIT）
- [ ] **黑色眼泪**（人・有料）：人に合わせる効果。名前から：目。つまみ：強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion

### 色（18）

- [ ] **彩色负片**（色）：人に合わせる効果。名前から：ネガ・カラー。つまみ：大きさ・速さ — いま：ISF Color Invert・RGB Invert・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **彩虹眼泪**（色）：人に合わせる効果。名前から：虹・目。つまみ：飾り・速さ・強さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **热成像**（色・有料）：人に合わせる効果。名前から：熱カメラ。つまみ：速さ・色・明るさ・大きさ・強さ — いま：ISF Thermal Camera — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Night Vision（MIT）
- [ ] **粉色便便**（色・有料）：人に合わせる効果。名前から：ピンク。つまみ：大きさ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **红恶魔**（色・有料）：人に合わせる効果。名前から：悪魔・赤。つまみ：色の濃さ・強さ・大きさ・質感 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **绿怪**（色・有料）：人に合わせる効果。名前から：緑。つまみ：強さ・色の濃さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **镭射眼 Ⅰ**（色）：人に合わせる効果。名前から：レーザー光・目。つまみ：色の濃さ・強さ・色・速さ — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **镭射眼 ⅠⅠ**（色）：人に合わせる効果。名前から：レーザー光・目。つまみ：色の濃さ・強さ・色 — いま：ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **阴暗面**（色・有料）：人に合わせる効果。名前から：暗い。つまみ：大きさ・色・縦ずれ — いま：vignette.wgsl・段の vignette — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **吸血鬼**（色・有料）：人に合わせる効果。名前から：血。つまみ：色・質感・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恐怖小丑**（色・有料）：人に合わせる効果。名前から：ホラー。つまみ：色の濃さ・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恶魔之翼**（色）：人に合わせる効果。名前から：悪魔。つまみ：色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恶魔印记**（色）：人に合わせる効果。名前から：悪魔。つまみ：大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恶魔尾巴**（色・有料）：人に合わせる効果。名前から：悪魔。つまみ：大きさ・色・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恶魔角**（色・有料）：人に合わせる効果。名前から：悪魔 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **血迹 I**（色・有料）：人に合わせる効果。名前から：血。つまみ：強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **血迹 II**（色）：人に合わせる効果。名前から：血。つまみ：強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **血迹 III**（色・有料）：人に合わせる効果。名前から：血。つまみ：強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

### レンズ・ゆがみ（17）

- [ ] **Face Mosaic**（レンズ・ゆがみ）：人に合わせる効果。名前から：モザイク・ピクセル・顔・人。つまみ：範囲・大きさ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **Gas Flow**（レンズ・ゆがみ）：人に合わせる効果。名前から：流れる・煙・霧（中国語名 气体流动）。つまみ：大きさ・色の濃さ・速さ・色 — いま：ISF Noise Displace・形の warp・段の fog — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の煙（CC0）
- [ ] **Gas Waves**（レンズ・ゆがみ）：人に合わせる効果。名前から：波・波打つ・煙・霧（中国語名 气波）。つまみ：色・速さ — いま：ISF Noise Displace・形の warp・段の fog — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／Kenney Particle Pack の煙（CC0）
- [ ] **Sound Wave**（レンズ・ゆがみ）：人に合わせる効果。名前から：波打つ。つまみ：色 — いま：ISF Noise Displace・形の warp — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **像素定格**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：ピクセル・コマ止め。つまみ：速さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Freeze Frame・Random Freeze（MIT）
- [ ] **像素面具**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：ピクセル。つまみ：数・色・大きさ・飾り・色の濃さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **声波**（レンズ・ゆがみ）：人に合わせる効果。名前から：波。つまみ：色 — いま：ISF Noise Displace・形の warp — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **局部扭曲**（レンズ・ゆがみ）：人に合わせる効果。名前から：ゆがみ。つまみ：強さ・速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **彩虹流体**（レンズ・ゆがみ）：人に合わせる効果。名前から：流体・虹。つまみ：ゆがみ・強さ・明るさ・大きさ・色・速さ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **旋转扭曲**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：ゆがみ・回転。つまみ：速さ・大きさ・ゆがみ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・キー rotate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **流体故障**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：流体・グリッチ。つまみ：速さ・ゆがみ・横の色ずれ・明るさ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace・ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **漩涡溶解**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：渦・溶ける。つまみ：ゆがみ・速さ・横ずれ・縦ずれ・大きさ・範囲 — いま：ISF Twirl・点群の dissolve・ISF Noise — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **空气流体**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：流体。つまみ：強さ・速さ・横の色ずれ・明るさ・大きさ — いま：ISF Bump Distortion・Cubic Warp・Noise Displace — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **脸部马赛克**（レンズ・ゆがみ）：人に合わせる効果。名前から：モザイク・顔。つまみ：範囲・大きさ — いま：ISF Pixellate・Mosaic・Multi-Pixellate — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **迷幻震波**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：衝撃波・サイケ。つまみ：速さ・色・強さ・ゆがみ・明るさ — いま：ISF Ripples・Circle Splash Distortion・ISF Multi Hue Shift・形の gradient — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Color Phase（MIT）
- [ ] **闪电漩涡**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：渦・稲妻。つまみ：大きさ・速さ・色・明るさ・背景の動き・強さ — いま：ISF Twirl — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **马赛克分身**（レンズ・ゆがみ・有料）：人に合わせる効果。名前から：モザイク・分身。つまみ：大きさ・範囲・速さ・強さ — いま：ISF Pixellate・Mosaic・Multi-Pixellate・ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Ghosting（MIT）

### グリッチ（9）

- [ ] **Chroma Diffusion**（グリッチ）：人に合わせる効果。名前から：色ずれ・光のにじみ・夢っぽい（中国語名 色差扩散）。つまみ：ぼかし・横の色ずれ・色の濃さ・大きさ — いま：rgbshift.wgsl（three.js RGBShiftShader）・段の bloom・物の glow・ISF Bloom・Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Chromatic Aberration（MIT）
- [ ] **Face Glitch**（グリッチ・有料）：人に合わせる効果。名前から：グリッチ・顔・人（中国語名 脸部故障）。つまみ：背景の動き・強さ・色・速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion
- [ ] **人物故障**（グリッチ・有料）：人に合わせる効果。名前から：グリッチ・人物 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／MediaPipe Image Segmenter（Apache-2.0）
- [ ] **卡顿线条**（グリッチ・有料）：人に合わせる効果。名前から：カクつき。つまみ：速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Freeze Frame・Random Freeze（MIT）
- [ ] **心动信号**（グリッチ・有料）：人に合わせる効果。名前から：信号の乱れ・ときめき。つまみ：大きさ・速さ・縦ずれ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）／ISF Heart（MIT）＋物の copies・scatter
- [ ] **情绪定格**（グリッチ）：人に合わせる効果。名前から：コマ止め。つまみ：強さ・範囲・速さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Freeze Frame・Random Freeze（MIT）
- [ ] **故障描边 I**（グリッチ）：人に合わせる効果。名前から：グリッチ・縁取り。つまみ：速さ — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption・ISF Edges・Edge Blowout＋Glow — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **电子屏故障**（グリッチ・有料）：人に合わせる効果。名前から：グリッチ。つまみ：速さ・縦ずれ・大きさ・色 — いま：ISF Bad TV・Pixel Shifter・Time Glitch RGB・JPEG Block Corruption — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF VHS Glitch・Glitch Shifter・Poly Glitch（MIT）
- [ ] **轨迹定格**（グリッチ・有料）：人に合わせる効果。名前から：コマ止め — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Freeze Frame・Random Freeze（MIT）

### 絵の具（5）

- [ ] **Cutout Poster**（絵の具・有料）：人に合わせる効果。名前から：ポスター・コラージュ（中国語名 拼贴海报）。つまみ：縦の色ずれ・横の色ずれ・大きさ・質感・色の濃さ・速さ・強さ — いま：ISF Posterize・ISF Quad Tile — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF MultiFrame 2x2・3x3（MIT）
- [ ] **动态涂鸦**（絵の具・有料）：人に合わせる効果。名前から：落書き。つまみ：速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Doodler（MIT）
- [ ] **涂鸦弹出**（絵の具・有料）：人に合わせる効果。名前から：落書き・弾んで出る。つまみ：速さ・大きさ・縦ずれ — いま：animators bounce_in・spring_in — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Doodler（MIT）
- [ ] **涂鸦扩散**（絵の具・有料）：人に合わせる効果。名前から：落書き — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Doodler（MIT）
- [ ] **涂鸦鬼影**（絵の具）：人に合わせる効果。名前から：落書き・影。つまみ：色・色の濃さ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Doodler（MIT）／ISF Ghosting（MIT）

### 分割（2）

- [ ] **CRASH!**（分割）：人に合わせる効果。名前から：割れる・衝撃。つまみ：速さ・縦ずれ・大きさ・色 — いま：ふるまい noise を x・y に — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF Shake（MIT）
- [ ] **纸张拼贴**（分割・有料）：人に合わせる効果。名前から：コラージュ・紙 — いま：ISF Quad Tile — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き／ISF MultiFrame 2x2・3x3（MIT）

### ぼかし（1）

- [ ] **局部模糊**（ぼかし）：人に合わせる効果。名前から：ぼかし。つまみ：色の濃さ・ノイズ・横の色ずれ・縦の色ずれ・ぼかし — いま：物の blur・ISF Fast Blur — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

### 枠（1）

- [ ] **Frame Scatter**（枠）：人に合わせる効果。名前から：枠。つまみ：速さ・範囲・色 — いま：形の stroke・四角（枠は形で描ける） — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

### その他（35）

- [ ] **BOOM！**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **Consciousness**（その他）：人に合わせる効果。名前だけでは中身が分からない（中国語名 意识流）。つまみ：速さ・縦ずれ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **九尾狐**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ・色・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **交叉移形**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・範囲・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **健美写真**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **千层1**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：数・強さ・縦ずれ・速さ・背景の動き・色・明るさ・範囲・大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **千层2**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：強さ・速さ・数・背景の動き・色・明るさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **千禧辣妹**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：数・速さ・強さ・色の濃さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **哥特**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：色の濃さ・大きさ・強さ・回転 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **啦啦队队长**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **嬉皮士**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **孤独卓别林**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **害羞**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：背景の動き・飾り — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **巴哥犬**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **恶灵骑士**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **敲打**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **机械几何**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **机械姬 I**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **机械姬 II**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **棒球选手**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **潮流入侵**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ・速さ・縦ずれ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **灵机一动 I**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・背景の動き・飾り・大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **灵魂出走**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・強さ・飾り — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **狼人**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：色の濃さ・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **猫耳女孩**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **生气**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：数・横ずれ・縦ずれ・大きさ・色の濃さ・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **移形回位**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・明るさ・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **笑哭**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：横ずれ・縦ずれ・大きさ・色の濃さ・速さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **篮球运动员**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **美式橄榄球**（その他）：人に合わせる効果。名前だけでは中身が分からない — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **舞者**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・縦ずれ・大きさ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **舞者 II**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ・縦ずれ・色 — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **视线遮挡**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：速さ・色・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **迷茫**（その他）：人に合わせる効果。名前だけでは中身が分からない。つまみ：大きさ・縦ずれ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き
- [ ] **鬼灯**（その他・有料）：人に合わせる効果。名前だけでは中身が分からない。つまみ：背景の動き・範囲・色・質感・強さ — いま：無い — 借りる先：MediaPipe Image Segmenter（Apache-2.0）で人を切り抜き

## Transitions（つなぎ目）（1140）

Motolii に有る 347・近い物あり 688・見つからない 105。

### 動き（283）

- [ ] **3 框滑动**（動き・有料）：つなぎ目。名前から：滑る・枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **3D照片**（動き・有料）：つなぎ目。名前から：3D・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **CRT 快拍**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **Dazzle Pulse**（動き・有料）：つなぎ目。名前から：脈打つ — いま：ふるまい oscillator を scale に＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **Jerky Camera**（動き・有料）：つなぎ目。名前から：揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **Shake 3**（動き）：つなぎ目。名前から：揺れ（中国語名 抖动） — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **Shake down**（動き・有料）：つなぎ目。名前から：揺れ・滑る・動く — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Twinkle Zoom**（動き）：つなぎ目。名前から：ズーム・星・点滅・きらきら（中国語名 星星变焦） — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **X形震动**（動き・有料）：つなぎ目。名前から：振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **Zoom Shake 2**（動き・有料）：つなぎ目。名前から：揺れ・拡大・縮小・寄る — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **上下眩晕**（動き・有料）：つなぎ目。名前から：めまい — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **丝滑渐隐**（動き・有料）：つなぎ目。名前から：じわっと消える — いま：ISF Fade（gl-transitions fade） — 借りる先：見つからない
- [ ] **全息投影旋转**（動き・有料）：つなぎ目。名前から：回転・ホログラム・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **冲击感**（動き・有料）：つなぎ目。名前から：衝撃 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **击掌**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **分格翻转**（動き・有料）：つなぎ目。名前から：裏返る・格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **刷卡**（動き・有料）：つなぎ目。名前から：カード・刷毛 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **向下涂刷**（動き・有料）：つなぎ目。名前から：下へ・刷毛 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **嗡嗡滑动**（動き・有料）：つなぎ目。名前から：滑る・ぶーん（揺れ） — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **四窗口翻转**（動き・有料）：つなぎ目。名前から：裏返る・窓 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **弹窗转换**（動き・有料）：つなぎ目。名前から：弾む・回る・窓 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **微抖动**（動き・有料）：つなぎ目。名前から：揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **微晃拉扯**（動き・有料）：つなぎ目。名前から：引っぱる・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **手动点击**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **投影幻灯片**（動き・有料）：つなぎ目。名前から：スライドショー・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions cube・Revolve_Left（MIT）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **抖动 II**（動き）：つなぎ目。名前から：揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **抖动放大**（動き・有料）：つなぎ目。名前から：揺れ・拡大 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **抖动缩小**（動き・有料）：つなぎ目。名前から：揺れ・縮小 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **摇摆左移**（動き・有料）：つなぎ目。名前から：左右に揺れる・左へ動く — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **摇摆旋转**（動き・有料）：つなぎ目。名前から：左右に揺れる・回転 — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **摇摆晃动**（動き・有料）：つなぎ目。名前から：揺れ・左右に揺れる — いま：ふるまい noise を 2 本に（つなぎ目の前後）・ふるまい oscillator を 2 本に — 借りる先：見つからない
- [ ] **摇摆缩放**（動き・有料）：つなぎ目。名前から：左右に揺れる・拡大縮小 — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **摇晃拉扯**（動き・有料）：つなぎ目。名前から：揺れ・引っぱる — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **摇镜**（動き・有料）：つなぎ目。名前から：揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **方框旋转**（動き・有料）：つなぎ目。名前から：回転・枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **暗影滑动**（動き・有料）：つなぎ目。名前から：滑る・影・暗い — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ・vignette.wgsl・段の vignette＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **横向微震**（動き・有料）：つなぎ目。名前から：揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **渐隐转移**（動き・有料）：つなぎ目。名前から：じわっと消える・回る — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **滑动网格**（動き・有料）：つなぎ目。名前から：滑る・格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **点击揭示**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **照片堆叠**（動き・有料）：つなぎ目。名前から：積み重ね・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **玻璃块缩放 2**（動き・有料）：つなぎ目。名前から：拡大縮小・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **玻璃块缩放 3**（動き・有料）：つなぎ目。名前から：拡大縮小・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **玻璃移动**（動き・有料）：つなぎ目。名前から：動く・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **眩晕切换**（動き・有料）：つなぎ目。名前から：めまい — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **窗口滑动**（動き・有料）：つなぎ目。名前から：滑る・窓 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **网点幻灯片**（動き・有料）：つなぎ目。名前から：スライドショー・網点 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **视觉嗡嗡**（動き・有料）：つなぎ目。名前から：ぶーん（揺れ） — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **资金翻转**（動き・有料）：つなぎ目。名前から：裏返る・金色 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **转场-叠加**（動き）：つなぎ目。名前から：重ねる・回る — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **边角拍击**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **重叠渐隐**（動き・有料）：つなぎ目。名前から：じわっと消える — いま：ISF Fade（gl-transitions fade） — 借りる先：見つからない
- [ ] **震动**（動き）：つなぎ目。名前から：振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **震动 II**（動き・有料）：つなぎ目。名前から：振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **震感放大**（動き・有料）：つなぎ目。名前から：拡大・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **震感缩小**（動き・有料）：つなぎ目。名前から：縮小・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **验证码点击**（動き・有料）：つなぎ目。名前から：打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **2 卡洗牌**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D 旋转木马 2**（動き・有料）：つなぎ目。名前から：回転・3D — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D小卡**（動き・有料）：つなぎ目。名前から：3D・カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D环绕**（動き・有料）：つなぎ目。名前から：周りを回る・3D — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D空间**（動き）：つなぎ目。名前から：3D・空間 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D轮播**（動き・有料）：つなぎ目。名前から：3D・めくり替え — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D钥匙**（動き・有料）：つなぎ目。名前から：3D — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **Axis Rotation**（動き）：つなぎ目。名前から：回転（中国語名 中轴旋转） — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Bottom Left II**（動き）：つなぎ目。名前から：滑る・動く（中国語名 左下方 II） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Center Rotate**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Corner Slide**（動き・有料）：つなぎ目。名前から：滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Cutout Flip**（動き）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **Drop & Expand**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る・滑る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Drop 3**（動き・有料）：つなぎ目。名前から：滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Flip**（動き）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **Flip & Zoom**（動き・有料）：つなぎ目。名前から：裏返る・拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Flip II**（動き）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **Gameboy 滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **IG 弹出**（動き・有料）：つなぎ目。名前から：弾んで出る — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **LED 隧道**（動き・有料）：つなぎ目。名前から：トンネル — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Left**（動き）：つなぎ目。名前から：左へ動く・滑る・動く（中国語名 左移） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **O 缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Paper Flip**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **Pass Through 3**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Pull in**（動き）：つなぎ目。名前から：寄る・拡大・縮小（中国語名 推近） — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Pull Out**（動き）：つなぎ目。名前から：引く・拡大・縮小・寄る（中国語名 拉远） — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Push Away 2**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Right**（動き）：つなぎ目。名前から：右へ動く・滑る・動く（中国語名 右移） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Right**（動き）：つなぎ目。名前から：右へ・滑る・動く（中国語名 向右） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Rotate & Change**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Shrink**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Sideway Bounce**（動き・有料）：つなぎ目。名前から：弾む — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **Slide**（動き）：つなぎ目。名前から：滑る・動く（中国語名 滑动） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Slide Drop**（動き・有料）：つなぎ目。名前から：滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Slide Interface**（動き・有料）：つなぎ目。名前から：滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Snap Zoom**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Squeeze**（動き）：つなぎ目。名前から：縮める・ゆがみ（中国語名 压缩） — いま：無い — 借りる先：ISF-Files Squeeze（MIT、名前が同じ）／gl-transitions squeeze（MIT、名前が同じ）
- [ ] **Stretch**（動き）：つなぎ目。名前から：伸ばす・傾く（中国語名 拉伸） — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **Stretch Left**（動き）：つなぎ目。名前から：左へ・伸ばす・滑る・動く（中国語名 向左拉伸） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **Stretch ll**（動き）：つなぎ目。名前から：伸ばす・傾く（中国語名 拉伸 ll） — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **Stretch Right**（動き）：つなぎ目。名前から：右へ・伸ばす・滑る・動く（中国語名 向右拉伸） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **Swipe Left**（動き・有料）：つなぎ目。名前から：滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Turn Table**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Up**（動き）：つなぎ目。名前から：上へ動く・滑る・動く（中国語名 上移） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Windmill**（動き）：つなぎ目。名前から：風・回転（中国語名 风车） — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **X光拉开**（動き・有料）：つなぎ目。名前から：引く・光 — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）／gl-transitions Overexposure（MIT）
- [ ] **Zoom to Change**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **Zoom Transition**（動き・有料）：つなぎ目。名前から：拡大・縮小・寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **三卡切换**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **三帧幻灯片 2**（動き・有料）：つなぎ目。名前から：スライドショー・コマ — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）／gl-transitions splitSlideIn*（MIT）
- [ ] **三维旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **三行立体翻转**（動き・有料）：つなぎ目。名前から：裏返る・立体 — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **下坠**（動き・有料）：つなぎ目。名前から：落ちる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **下滑**（動き・有料）：つなぎ目。名前から：下へ滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **下移**（動き）：つなぎ目。名前から：下へ動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **中轴翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **二分滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **交错滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **传单飞走**（動き・有料）：つなぎ目。名前から：飛ぶ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **侧边翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **倾斜开关**（動き・有料）：つなぎ目。名前から：傾く・スイッチ — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **倾斜拉伸**（動き・有料）：つなぎ目。名前から：伸ばす・傾く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **倾斜拉开**（動き・有料）：つなぎ目。名前から：傾く・引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **倾斜滑动**（動き・有料）：つなぎ目。名前から：滑る・傾く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **倾斜滚动**（動き・有料）：つなぎ目。名前から：スクロール・傾く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **几何滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **分区缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **动力缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **动感下拉**（動き・有料）：つなぎ目。名前から：引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **半色调幻灯片**（動き・有料）：つなぎ目。名前から：スライドショー — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡住的滑块**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡片弹出**（動き・有料）：つなぎ目。名前から：弾んで出る・カード — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡片洗牌**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡片空间**（動き・有料）：つなぎ目。名前から：カード・空間 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡片选择**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **卡纸掉落**（動き・有料）：つなぎ目。名前から：落ちる・紙・カード — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **压缩滑动**（動き・有料）：つなぎ目。名前から：滑る・縮める — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **双卡切换**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **双重翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **反相转场**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **变速滚动**（動き・有料）：つなぎ目。名前から：速さが変わる・スクロール — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **变速翻转**（動き・有料）：つなぎ目。名前から：裏返る・速さが変わる — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **向上**（動き）：つなぎ目。名前から：上へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向下**（動き）：つなぎ目。名前から：下へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向下拖拽**（動き・有料）：つなぎ目。名前から：下へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向下翻转**（動き・有料）：つなぎ目。名前から：裏返る・下へ — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向右上**（動き）：つなぎ目。名前から：右へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向右下**（動き）：つなぎ目。名前から：右へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向右滑动**（動き・有料）：つなぎ目。名前から：滑る・右へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向右翻转**（動き・有料）：つなぎ目。名前から：裏返る・右へ — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向左**（動き）：つなぎ目。名前から：左へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向左上**（動き）：つなぎ目。名前から：左へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向左下**（動き）：つなぎ目。名前から：左へ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向左拉屏**（動き・有料）：つなぎ目。名前から：左へ・引く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **嗖嗖滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **回忆下滑**（動き）：つなぎ目。名前から：下へ滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **回忆拉屏**（動き・有料）：つなぎ目。名前から：引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **回忆拉屏 II**（動き・有料）：つなぎ目。名前から：引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **图片放大**（動き・有料）：つなぎ目。名前から：拡大 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **地平线拉伸**（動き・有料）：つなぎ目。名前から：伸ばす — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **垂直滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **多轨滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **多边形缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **多面旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **夹扣翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **宇宙翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **宝石滴落**（動き・有料）：つなぎ目。名前から：落ちる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **居中翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **左下旋入**（動き・有料）：つなぎ目。名前から：回って入る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **左移切换**（動き・有料）：つなぎ目。名前から：左へ動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **左移弹动**（動き・有料）：つなぎ目。名前から：弾む・左へ動く — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **平铺视图滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **弹出**（動き・有料）：つなぎ目。名前から：弾んで出る — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **弹力拉扯**（動き・有料）：つなぎ目。名前から：引っぱる・弾む — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **弹射旋转**（動き・有料）：つなぎ目。名前から：回転・弾む — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **弹性下坠**（動き・有料）：つなぎ目。名前から：弾力・落ちる — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **弹性镜头**（動き・有料）：つなぎ目。名前から：弾力・カメラ — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **弹跳**（動き）：つなぎ目。名前から：弾む — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **弹跳光芒**（動き・有料）：つなぎ目。名前から：弾む・光 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions Overexposure（MIT）
- [ ] **弹跳压缩**（動き・有料）：つなぎ目。名前から：弾む・縮める — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **微微旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **微调翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **快速挥动**（動き・有料）：つなぎ目。名前から：振る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **快速缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **扇形卡片**（動き・有料）：つなぎ目。名前から：カード — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **扭转**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **投掷切换**（動き・有料）：つなぎ目。名前から：投げる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **抛掷与切换**（動き・有料）：つなぎ目。名前から：投げる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **拉伸包裹**（動き・有料）：つなぎ目。名前から：伸ばす — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **拉开**（動き・有料）：つなぎ目。名前から：引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **拉远一闪**（動き・有料）：つなぎ目。名前から：引く・光る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **拉链**（動き・有料）：つなぎ目。名前から：引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **拖拽滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **按压回弹**（動き・有料）：つなぎ目。名前から：跳ね返る — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **推倒效果**（動き・有料）：つなぎ目。名前から：押す — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **推近 II**（動き・有料）：つなぎ目。名前から：寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **放大上移**（動き・有料）：つなぎ目。名前から：拡大・上へ動く — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **斜线切换**（動き・有料）：つなぎ目。名前から：斜め — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **旋转圆球**（動き・有料）：つなぎ目。名前から：回転・丸 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **旋转圆盘**（動き・有料）：つなぎ目。名前から：回転・丸 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **旋转圆盘 ll**（動き・有料）：つなぎ目。名前から：回転・丸 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **旋转拨盘**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **旋转纵深**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **旋转跳卡**（動き・有料）：つなぎ目。名前から：回転・跳ぶ・カード — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **旋转转场**（動き・有料）：つなぎ目。名前から：回転・回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **旋风转**（動き・有料）：つなぎ目。名前から：回る・風 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **无限穿越 Ⅰ**（動き）：つなぎ目。名前から：通り抜ける — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **无限穿越 Ⅱ**（動き）：つなぎ目。名前から：通り抜ける — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **明信片缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **极速缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **梦幻转变**（動き・有料）：つなぎ目。名前から：回る・夢 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions Overexposure（MIT）
- [ ] **槽位压缩**（動き・有料）：つなぎ目。名前から：縮める — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **横向拉伸**（動き・有料）：つなぎ目。名前から：伸ばす — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **横向滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **横条挤压**（動き・有料）：つなぎ目。名前から：押しつぶす — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **水平旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **汇聚**（動き・有料）：つなぎ目。名前から：集まる — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **海浪飞溅**（動き・有料）：つなぎ目。名前から：飛ぶ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **混乱旋转**（動き・有料）：つなぎ目。名前から：回転・乱れ — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **滑动压迫感**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **滑动炸裂特效**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **滑动箭头**（動き・有料）：つなぎ目。名前から：滑る・頭 — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **滑动队列**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **猫眼缩放**（動き・有料）：つなぎ目。名前から：拡大縮小・目 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **环回推近**（動き・有料）：つなぎ目。名前から：寄る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **环形抽卡**（動き・有料）：つなぎ目。名前から：カードを引く — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **甩鞭转场**（動き・有料）：つなぎ目。名前から：回る・振る・むち — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **画中画放大**（動き）：つなぎ目。名前から：拡大 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **目标缩放**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **相册翻转**（動き・有料）：つなぎ目。名前から：裏返る・アルバム — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **瞬间移动**（動き・有料）：つなぎ目。名前から：動く・一瞬 — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **社媒旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **空间上移**（動き・有料）：つなぎ目。名前から：上へ動く・空間 — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间切分**（動き・有料）：つなぎ目。名前から：空間 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间弹动**（動き・有料）：つなぎ目。名前から：弾む・空間 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间弹动 II**（動き・有料）：つなぎ目。名前から：弾む・空間 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间弹动 III**（動き・有料）：つなぎ目。名前から：弾む・空間 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间弹动 IV**（動き・有料）：つなぎ目。名前から：弾む・空間 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间拖拽**（動き・有料）：つなぎ目。名前から：空間 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间旋转**（動き・有料）：つなぎ目。名前から：回転・空間 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间旋转 II**（動き・有料）：つなぎ目。名前から：回転・空間 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间旋转 III**（動き・有料）：つなぎ目。名前から：回転・空間 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间翻转 I**（動き・有料）：つなぎ目。名前から：裏返る・空間 — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间翻转 II**（動き・有料）：つなぎ目。名前から：裏返る・空間 — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **空间跳跃**（動き・有料）：つなぎ目。名前から：空間・跳ぶ — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **空间速推**（動き・有料）：つなぎ目。名前から：空間・押す — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **穿越**（動き・有料）：つなぎ目。名前から：通り抜ける — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **穿越 II**（動き・有料）：つなぎ目。名前から：通り抜ける — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **突然跳跃**（動き・有料）：つなぎ目。名前から：跳ぶ — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **立体双向**（動き・有料）：つなぎ目。名前から：立体 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **立体层次**（動き・有料）：つなぎ目。名前から：立体 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **立体层次II**（動き・有料）：つなぎ目。名前から：立体 — いま：無い — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **立体旋转**（動き・有料）：つなぎ目。名前から：回転・立体 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **竖线拉伸**（動き・有料）：つなぎ目。名前から：伸ばす — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **简单的鞭打**（動き・有料）：つなぎ目。名前から：むち — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **箭头向上**（動き・有料）：つなぎ目。名前から：上へ・頭 — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **红色滑动**（動き・有料）：つなぎ目。名前から：滑る・赤 — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **纵向滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **纸张旋转**（動き・有料）：つなぎ目。名前から：回転・紙 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **线描转换**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **缩小滑动**（動き・有料）：つなぎ目。名前から：縮小・滑る — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **缩放切换**（動き・有料）：つなぎ目。名前から：拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **缩放旋转**（動き・有料）：つなぎ目。名前から：回転・拡大縮小 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **缩放翻转**（動き・有料）：つなぎ目。名前から：裏返る・拡大縮小 — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **缩放轮播**（動き・有料）：つなぎ目。名前から：拡大縮小・めくり替え — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **网屏缩小**（動き・有料）：つなぎ目。名前から：縮小 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **翻滚切换**（動き・有料）：つなぎ目。名前から：めくる — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **翻转**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **翻转与扩写**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **翻转视角**（動き・有料）：つなぎ目。名前から：裏返る — いま：無い — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **草图转场**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **蓄力放大**（動き・有料）：つなぎ目。名前から：拡大 — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **角落挥摆**（動き・有料）：つなぎ目。名前から：振る・落ちる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **跳动炫光**（動き・有料）：つなぎ目。名前から：跳ねる・光 — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）／gl-transitions Overexposure（MIT）
- [ ] **跳跃切换**（動き・有料）：つなぎ目。名前から：跳ぶ — いま：無い — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **转换切屏**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **运镜压缩**（動き・有料）：つなぎ目。名前から：カメラワーク・縮める — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **进阶旋转**（動き・有料）：つなぎ目。名前から：回転 — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **重叠上滑**（動き・有料）：つなぎ目。名前から：上へ滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **镜头右移**（動き・有料）：つなぎ目。名前から：カメラ・右へ動く — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **镜头缩小**（動き・有料）：つなぎ目。名前から：縮小・カメラ — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **镜头速移**（動き・有料）：つなぎ目。名前から：カメラ — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **镜头重叠**（動き・有料）：つなぎ目。名前から：カメラ — いま：無い — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **高速滑动**（動き・有料）：つなぎ目。名前から：滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **鬼魅转曲**（動き・有料）：つなぎ目。名前から：回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **黏滑飞溅**（動き・有料）：つなぎ目。名前から：飛ぶ — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）

### 光（165）

- [ ] **Black Fade**（光）：つなぎ目。名前から：黒く光る・フェード（中国語名 闪黑） — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Clock wipe**（光）：つなぎ目。名前から：スキャン・丸・時計回りに拭く・拭き取る（中国語名 圆形扫描） — いま：ISF Radial（gl-transitions radial）・ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **Montage Snippets**（光）：つなぎ目。名前から：まぶしい光・重ねて溶ける・分割画面（中国語名 眩光叠化） — いま：段の lens_flare・ISF Lens Flare＋ISF Fade でつなぐ — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）／gl-transitions splitSlideIn*（MIT）
- [ ] **光学耀斑**（光・有料）：つなぎ目。名前から：フレア・光 — いま：段の lens_flare・ISF Lens Flare＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **光影闪动**（光・有料）：つなぎ目。名前から：点滅・光・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions Overexposure（MIT）
- [ ] **光标点击**（光・有料）：つなぎ目。名前から：光・打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **克隆激光**（光・有料）：つなぎ目。名前から：レーザー・クローン — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **冰冷光环**（光・有料）：つなぎ目。名前から：光の輪・氷 — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **发光扭动**（光・有料）：つなぎ目。名前から：光る・くねる — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **发光重影**（光・有料）：つなぎ目。名前から：光る・二重像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **复古赛博朋克**（光・有料）：つなぎ目。名前から：サイバーパンク・レトロ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **太阳粉**（光・有料）：つなぎ目。名前から：太陽・ピンク — いま：段の lens_flare・ISF Lens Flare＋ISF Fade でつなぐ — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **太阳耀斑**（光・有料）：つなぎ目。名前から：太陽・フレア — いま：段の lens_flare・ISF Lens Flare＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **幻影闪现**（光・有料）：つなぎ目。名前から：ちらっと出る・幻影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **幽灵闪光**（光・有料）：つなぎ目。名前から：光る・幽霊 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **强震闪动**（光・有料）：つなぎ目。名前から：点滅・強い揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **微震闪黑**（光・有料）：つなぎ目。名前から：黒く光る・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **快速震闪**（光・有料）：つなぎ目。名前から：光る・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **报纸闪屏**（光・有料）：つなぎ目。名前から：画面が光る・新聞 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **拖影光效**（光・有料）：つなぎ目。名前から：光・残像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **描边震动**（光・有料）：つなぎ目。名前から：縁取り・振動 — いま：ISF Edges・Edge Blowout＋Glow＋ISF Fade でつなぐ・ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **晃动快门**（光・有料）：つなぎ目。名前から：シャッター・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **曝光拉影**（光・有料）：つなぎ目。名前から：露出・引く・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **曝光摇镜**（光・有料）：つなぎ目。名前から：露出・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **柔光呼吸**（光・有料）：つなぎ目。名前から：やわらかい光・呼吸 — いま：ふるまい oscillator を scale に＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **波点闪切**（光・有料）：つなぎ目。名前から：光って切り替わる・水玉 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **淡入淡出闪烁**（光・有料）：つなぎ目。名前から：点滅・フェードイン・フェードアウト — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **溶解轮廓**（光・有料）：つなぎ目。名前から：輪郭・溶ける — いま：ISF Edges・Edge Blowout＋Glow＋ISF Fade でつなぐ — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **烟花光斑**（光・有料）：つなぎ目。名前から：光の玉・花火 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **相机快门**（光・有料）：つなぎ目。名前から：シャッター・カメラ — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **网格闪白**（光・有料）：つなぎ目。名前から：白く光る・格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **耀斑切换**（光・有料）：つなぎ目。名前から：フレア — いま：段の lens_flare・ISF Lens Flare＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **荧光律动**（光・有料）：つなぎ目。名前から：蛍光・リズム — いま：ふるまい oscillator を scale に＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **闪光金**（光・有料）：つなぎ目。名前から：光る・金色 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **闪拍**（光・有料）：つなぎ目。名前から：光る・打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **阳光擦拭**（光・有料）：つなぎ目。名前から：日の光・拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **霓虹阴影**（光・有料）：つなぎ目。名前から：ネオン・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **B&W Flash**（光）：つなぎ目。名前から：白く光る・光・光る（フラッシュ）・白黒（中国語名 黑白闪光） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions Overexposure（MIT）
- [ ] **Blanch**（光）：つなぎ目。名前から：光る（フラッシュ）（中国語名 泛白） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Camera Glow**（光）：つなぎ目。名前から：光のにじみ・カメラワーク・光る・光る（フラッシュ）（中国語名 辉光运镜） — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **CRT扫描**（光・有料）：つなぎ目。名前から：スキャン — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **Dreamy Bubbles**（光・有料）：つなぎ目。名前から：光のにじみ・夢っぽい・泡 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／Kenney Particle Pack（CC0）＋ISF Fade でつなぐ
- [ ] **Elastic & Glowing**（光・有料）：つなぎ目。名前から：光る・弾む — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **Flash**（光）：つなぎ目。名前から：光る・光る（フラッシュ）（中国語名 闪光灯） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Flash**（光・有料）：つなぎ目。名前から：光る（フラッシュ） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Light Beam**（光）：つなぎ目。名前から：光の束・光の筋（中国語名 光束） — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **Light Leaks**（光・有料）：つなぎ目。名前から：光漏れ — いま：無い — 借りる先：gl-transitions Overexposure（MIT）＋ISF v002 Light Leak（MIT）
- [ ] **Light Sweep II**（光）：つなぎ目。名前から：蛍光・スキャン・光が走る（中国語名 荧光扫描） — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **Lightflow Scan**（光・有料）：つなぎ目。名前から：光が走る・スキャン — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **Lightning**（光）：つなぎ目。名前から：稲妻・アニメ・電気（中国語名 动漫闪电） — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **Lumin Flash**（光・有料）：つなぎ目。名前から：光る・光る（フラッシュ） — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Neon**（光・有料）：つなぎ目。名前から：ネオン — いま：無い — 借りる先：ISF-Files Neon（MIT、名前が同じ）／ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **Neon 2**（光・有料）：つなぎ目。名前から：ネオン — いま：無い — 借りる先：ISF-Files Neon（MIT、名前が同じ）／ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **Rainbow Warp**（光）：つなぎ目。名前から：ストロボ・ネガ・カラー・ゆがみ（中国語名 彩色负片频闪） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **Shutter**（光）：つなぎ目。名前から：シャッター・光る（フラッシュ）（中国語名 快门） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Shutter II**（光）：つなぎ目。名前から：シャッター・光る（フラッシュ）（中国語名 快门 II） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Snapshot**（光・有料）：つなぎ目。名前から：光る（フラッシュ） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Strobe**（光）：つなぎ目。名前から：ストロボ・点滅（中国語名 频闪） — いま：無い — 借りる先：ISF-Files Strobe（MIT、名前が同じ）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Subject Flash**（光・有料）：つなぎ目。名前から：光る（フラッシュ） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **White Flash**（光）：つなぎ目。名前から：光る（フラッシュ） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **White Flash**（光）：つなぎ目。名前から：白く光る・光る（フラッシュ）（中国語名 闪白） — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **Woosh**（光）：つなぎ目。名前から：放射・滑る・動く（中国語名 放射） — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **一闪而过**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **一闪而过II**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **三屏闪切**（光・有料）：つなぎ目。名前から：光って切り替わる・3 分割画面 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions splitSlideIn*（MIT）
- [ ] **上下扭闪**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **主扫描**（光・有料）：つなぎ目。名前から：スキャン — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **传送光束**（光・有料）：つなぎ目。名前から：光の束・転送 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **余烬快闪**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **光晕变焦**（光・有料）：つなぎ目。名前から：光のにじみ・ズーム — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **光晕缩放**（光・有料）：つなぎ目。名前から：光のにじみ・拡大縮小 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **光条**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **光爆**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **光盘切换**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **光线涂鸦**（光・有料）：つなぎ目。名前から：光の筋・落書き — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **光辉出口**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **全息扫描**（光・有料）：つなぎ目。名前から：スキャン・ホログラム — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **切换快门**（光・有料）：つなぎ目。名前から：シャッター — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **反色频闪**（光・有料）：つなぎ目。名前から：ストロボ・色反転 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **发光变焦**（光・有料）：つなぎ目。名前から：光る・ズーム — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **发光圆盘**（光・有料）：つなぎ目。名前から：光る・丸 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **垂直闪烁**（光・有料）：つなぎ目。名前から：点滅 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **屏蔽闪烁**（光・有料）：つなぎ目。名前から：点滅 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **强烈阳光**（光・有料）：つなぎ目。名前から：日の光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **快门滑动**（光・有料）：つなぎ目。名前から：シャッター・滑る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **惊悚频闪**（光・有料）：つなぎ目。名前から：ストロボ・スリラー — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **截图**（光・有料）：つなぎ目。名前から：スクショ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **扫光**（光・有料）：つなぎ目。名前から：光が走る — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **扫描捕捉**（光・有料）：つなぎ目。名前から：スキャン — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **斜向闪光**（光・有料）：つなぎ目。名前から：光る・斜め — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **方形闪烁**（光・有料）：つなぎ目。名前から：点滅 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **日落闪光**（光・有料）：つなぎ目。名前から：光る・落ちる — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **时光穿梭**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **曝光拉丝**（光・有料）：つなぎ目。名前から：露出・引く — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **未来光谱**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **未来光谱 II**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **极地光感**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **极致耀光**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **柔光斜移**（光・有料）：つなぎ目。名前から：やわらかい光・斜め — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **橙色闪光**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **泛光**（光）：つなぎ目。名前から：光のにじみ — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **流光**（光・有料）：つなぎ目。名前から：流れる光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **浪漫光彩**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **滑动闪烁**（光・有料）：つなぎ目。名前から：点滅・滑る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **漏光涂鸦**（光・有料）：つなぎ目。名前から：光漏れ・落書き — いま：無い — 借りる先：gl-transitions Overexposure（MIT）＋ISF v002 Light Leak（MIT）／ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **漏光涂鸦I**（光・有料）：つなぎ目。名前から：光漏れ・落書き — いま：無い — 借りる先：gl-transitions Overexposure（MIT）＋ISF v002 Light Leak（MIT）／ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **激光切割**（光・有料）：つなぎ目。名前から：レーザー・切る — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **激光碎片**（光・有料）：つなぎ目。名前から：レーザー・破片 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **激光迪斯科 2**（光・有料）：つなぎ目。名前から：ディスコ・レーザー — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **火焰闪光**（光・有料）：つなぎ目。名前から：光る・炎 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **炫光**（光）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **炫光 II**（光）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **炫光 III**（光）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **热感瞬移**（光・有料）：つなぎ目。名前から：瞬間移動 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **热闪**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **爆竹快闪**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **爆闪 II**（光・有料）：つなぎ目。名前から：強く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **狗仔高光**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **玫瑰闪光**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **环形传送**（光・有料）：つなぎ目。名前から：転送 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **电光涂鸦**（光・有料）：つなぎ目。名前から：電気の光・落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **白闪下扯**（光・有料）：つなぎ目。名前から：白く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **瞬移切换**（光・有料）：つなぎ目。名前から：瞬間移動 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **神圣闪现**（光・有料）：つなぎ目。名前から：ちらっと出る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **简单闪回**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **紫红闪光**（光・有料）：つなぎ目。名前から：光る・赤・紫 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **红光**（光・有料）：つなぎ目。名前から：光・赤 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **红光扫描**（光・有料）：つなぎ目。名前から：光が走る・赤 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **红白闪切**（光・有料）：つなぎ目。名前から：白く光る・赤 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **红色闪电**（光・有料）：つなぎ目。名前から：稲妻・赤 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **缩放闪光**（光・有料）：つなぎ目。名前から：光る・拡大縮小 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **翻转与阳光**（光・有料）：つなぎ目。名前から：日の光・裏返る — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **聚光灯**（光・有料）：つなぎ目。名前から：光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **蓝光扫描**（光・有料）：つなぎ目。名前から：光が走る・青 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **血色闪烁**（光・有料）：つなぎ目。名前から：点滅・血 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **裂拼闪现**（光・有料）：つなぎ目。名前から：ちらっと出る・つなぎ合わせ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions splitSlideIn*（MIT）
- [ ] **负片爆闪**（光）：つなぎ目。名前から：強く光る・ネガ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **赛博朋克**（光・有料）：つなぎ目。名前から：サイバーパンク — いま：無い — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **迷幻频闪**（光・有料）：つなぎ目。名前から：ストロボ・サイケ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **透明光晕**（光・有料）：つなぎ目。名前から：光のにじみ — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **部分闪白**（光・有料）：つなぎ目。名前から：白く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **重绘闪现**（光・有料）：つなぎ目。名前から：ちらっと出る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **长曝光**（光・有料）：つなぎ目。名前から：露出 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪光弹**（光・有料）：つなぎ目。名前から：光る・弾む — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **闪动光斑**（光）：つなぎ目。名前から：光の玉・点滅 — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪动光斑 II**（光・有料）：つなぎ目。名前から：光の玉・点滅 — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪动斜线**（光・有料）：つなぎ目。名前から：点滅・斜め — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **闪回**（光・有料）：つなぎ目。名前から：光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪烁失调**（光・有料）：つなぎ目。名前から：点滅 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪白 II**（光・有料）：つなぎ目。名前から：白く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪黑 II**（光・有料）：つなぎ目。名前から：黒く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **阳光光感**（光・有料）：つなぎ目。名前から：光・日の光 — いま：無い — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **霓虹涂鸦**（光・有料）：つなぎ目。名前から：ネオン・落書き — いま：無い — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ／ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **霓虹渗漏**（光・有料）：つなぎ目。名前から：ネオン — いま：無い — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **霓虹热浪**（光・有料）：つなぎ目。名前から：ネオン・陽炎 — いま：無い — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ／gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **霓虹负片闪烁**（光・有料）：つなぎ目。名前から：ネオン・点滅・ネガ — いま：無い — 借りる先：ISF Neon（MIT）＋ISF Fade でつなぐ／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **靛蓝闪烁**（光・有料）：つなぎ目。名前から：点滅・青 — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **频闪 II**（光・有料）：つなぎ目。名前から：ストロボ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **频闪 III**（光・有料）：つなぎ目。名前から：ストロボ — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **黑白闪现**（光・有料）：つなぎ目。名前から：白く光る — いま：無い — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **派对流苏**（光・有料）：つなぎ目。名前から：パーティー — いま：無い — 借りる先：見つからない
- [ ] **电光**（光・有料）：つなぎ目。名前から：電気の光 — いま：無い — 借りる先：見つからない
- [ ] **电光 II**（光・有料）：つなぎ目。名前から：電気の光 — いま：無い — 借りる先：見つからない
- [ ] **电流**（光・有料）：つなぎ目。名前から：電流 — いま：無い — 借りる先：見つからない
- [ ] **闪电岩石**（光・有料）：つなぎ目。名前から：稲妻 — いま：無い — 借りる先：見つからない

### 分割（163）

- [ ] **3D翻页01**（分割・有料）：つなぎ目。名前から：ページめくり・3D — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D翻页02**（分割・有料）：つなぎ目。名前から：ページめくり・3D — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D魔方**（分割・有料）：つなぎ目。名前から：ルービックキューブ・3D — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **Anti-CW Swirl**（分割）：つなぎ目。名前から：反時計回り・色ずれ・渦・ねじれ（中国語名 色差逆时针） — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **Blocks**（分割）：つなぎ目。名前から：パズル・ブロック（中国語名 黑色块） — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **Cube**（分割）：つなぎ目。名前から：立方体（中国語名 立方体） — いま：ISF cube（gl-transitions cube） — 借りる先：ISF-Files cube（MIT、名前が同じ）／gl-transitions cube（MIT、名前が同じ）
- [ ] **Cube Rotate**（分割・有料）：つなぎ目。名前から：立方体・回転 — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Cubic Flip**（分割・有料）：つなぎ目。名前から：立方体・裏返る — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **CW Swirl**（分割）：つなぎ目。名前から：時計回り・色ずれ・渦・ねじれ（中国語名 色差顺时针） — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **Flip Page**（分割・有料）：つなぎ目。名前から：ページめくり・裏返る — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **Page Turning**（分割）：つなぎ目。名前から：ページめくり・回転（中国語名 翻页） — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Reveal Brush**（分割・有料）：つなぎ目。名前から：拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **Rotate CCW II**（分割）：つなぎ目。名前から：反時計回り・回転（中国語名 逆时针旋转 II） — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Rotate CW II**（分割）：つなぎ目。名前から：時計回り・回転（中国語名 顺时针旋转 II） — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Wipe Left**（分割）：つなぎ目。名前から：拭き取る・左へ・滑る・動く（中国語名 向左擦除） — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：ISF-Files Wipe Left（MIT、名前が同じ）／gl-transitions wipeLeft（MIT、名前が同じ）
- [ ] **Wipe Right**（分割）：つなぎ目。名前から：拭き取る・右へ・滑る・動く（中国語名 向右擦除） — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：ISF-Files Wipe Right（MIT、名前が同じ）／gl-transitions wipeRight（MIT、名前が同じ）
- [ ] **Wipe Up**（分割）：つなぎ目。名前から：拭き取る・上へ・滑る・動く（中国語名 向上擦除） — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：ISF-Files Wipe Up（MIT、名前が同じ）／gl-transitions wipeUp（MIT、名前が同じ）
- [ ] **倒放俄罗斯方块**（分割・有料）：つなぎ目。名前から：四角いブロック — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **凌乱的橡皮擦**（分割・有料）：つなぎ目。名前から：拭く — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **凸起瓷砖**（分割・有料）：つなぎ目。名前から：タイル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **刷门户**（分割・有料）：つなぎ目。名前から：扉・刷毛 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **剥离**（分割・有料）：つなぎ目。名前から：はがす — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：見つからない
- [ ] **半色调擦除**（分割・有料）：つなぎ目。名前から：拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **向下擦除**（分割）：つなぎ目。名前から：拭き取る・下へ — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **喷漆**（分割・有料）：つなぎ目。名前から：スプレー — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **圆形擦除**（分割・有料）：つなぎ目。名前から：拭き取る・丸 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **圆角暗点**（分割・有料）：つなぎ目。名前から：丸・暗い — いま：vignette.wgsl・段の vignette＋ISF Fade でつなぐ — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **威弗利格子**（分割・有料）：つなぎ目。名前から：格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **字母拼贴**（分割・有料）：つなぎ目。名前から：コラージュ・字 — いま：animators print_on・print_off＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **幻灯片立方体**（分割・有料）：つなぎ目。名前から：立方体・スライドショー — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **弹性擦拭**（分割・有料）：つなぎ目。名前から：拭き取る・弾力 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **快涂擦除**（分割・有料）：つなぎ目。名前から：拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **抹布擦拭**（分割・有料）：つなぎ目。名前から：拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **抽象涂抹**（分割・有料）：つなぎ目。名前から：塗る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **拼图剪辑**（分割・有料）：つなぎ目。名前から：パズル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **拼图盒子**（分割・有料）：つなぎ目。名前から：パズル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **拼图碎片**（分割・有料）：つなぎ目。名前から：破片・パズル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **方块弹转**（分割・有料）：つなぎ目。名前から：四角いブロック・弾む・回る — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **方块滑动**（分割・有料）：つなぎ目。名前から：四角いブロック・滑る — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **方块翻转**（分割・有料）：つなぎ目。名前から：四角いブロック・裏返る — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **时针切换**（分割・有料）：つなぎ目。名前から：時計回り — いま：ISF Radial（gl-transitions radial） — 借りる先：見つからない
- [ ] **暗影擦除**（分割・有料）：つなぎ目。名前から：拭き取る・影・暗い — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*）・ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **棋盘翻转**（分割・有料）：つなぎ目。名前から：チェッカー・裏返る — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **清洗刷新**（分割・有料）：つなぎ目。名前から：刷毛 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **滚动立方**（分割・有料）：つなぎ目。名前から：立方体・スクロール — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **点击圆圈**（分割・有料）：つなぎ目。名前から：丸・輪・打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **点阵立方**（分割・有料）：つなぎ目。名前から：立方体 — いま：ISF cube（gl-transitions cube） — 借りる先：見つからない
- [ ] **照片分割**（分割・有料）：つなぎ目。名前から：分割・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **照片拼贴**（分割・有料）：つなぎ目。名前から：コラージュ・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **照片拼贴切换**（分割・有料）：つなぎ目。名前から：コラージュ・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **照片拼贴旋转**（分割・有料）：つなぎ目。名前から：コラージュ・回転・写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **玻璃破碎**（分割・有料）：つなぎ目。名前から：割れる・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **玻璃破碎 II**（分割・有料）：つなぎ目。名前から：割れる・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **瓷砖合并**（分割・有料）：つなぎ目。名前から：タイル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **瓷砖展开**（分割・有料）：つなぎ目。名前から：タイル・広がる — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **画笔擦除**（分割）：つなぎ目。名前から：拭き取る・筆 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **画笔笔触环**（分割・有料）：つなぎ目。名前から：筆・筆あと — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **画笔重叠**（分割・有料）：つなぎ目。名前から：筆 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **碎片淡出**（分割・有料）：つなぎ目。名前から：破片・フェードアウト — いま：ISF Fade（gl-transitions fade） — 借りる先：見つからない
- [ ] **碎裂玻璃**（分割・有料）：つなぎ目。名前から：砕ける・ガラス — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **窗口拼图**（分割・有料）：つなぎ目。名前から：パズル・窓 — いま：ISF Grid Flip（gl-transitions GridFlip）・形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **窗户擦拭器**（分割・有料）：つなぎ目。名前から：拭き取る・窓 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*）・形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **立体翻页**（分割・有料）：つなぎ目。名前から：ページめくり・立体 — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **立体翻页 II**（分割・有料）：つなぎ目。名前から：ページめくり・立体 — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：gl-transitions cube・Revolve_Left（MIT）
- [ ] **立方体旋转 2**（分割・有料）：つなぎ目。名前から：立方体・回転 — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **立方体脸部**（分割・有料）：つなぎ目。名前から：立方体・顔 — いま：ISF cube（gl-transitions cube） — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **立方体集群**（分割・有料）：つなぎ目。名前から：立方体 — いま：ISF cube（gl-transitions cube） — 借りる先：見つからない
- [ ] **立方旋转**（分割・有料）：つなぎ目。名前から：立方体・回転 — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **立方脸部**（分割・有料）：つなぎ目。名前から：立方体・顔 — いま：ISF cube（gl-transitions cube） — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **立方重组**（分割・有料）：つなぎ目。名前から：立方体 — いま：ISF cube（gl-transitions cube） — 借りる先：見つからない
- [ ] **简单翻页**（分割・有料）：つなぎ目。名前から：ページめくり — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：見つからない
- [ ] **紫色摩擦**（分割・有料）：つなぎ目。名前から：拭く・紫 — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **网格打乱**（分割・有料）：つなぎ目。名前から：格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **网格漂移**（分割・有料）：つなぎ目。名前から：格子 — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **翻页 Ⅱ**（分割・有料）：つなぎ目。名前から：ページめくり — いま：ISF Inverted Page Curl（gl-transitions InvertedPageCurl、BSD-3） — 借りる先：見つからない
- [ ] **色相照片拼贴**（分割・有料）：つなぎ目。名前から：コラージュ・写真・色相 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **裂开的立方体**（分割・有料）：つなぎ目。名前から：立方体・裂ける — いま：ISF cube（gl-transitions cube） — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **逆时针旋转**（分割）：つなぎ目。名前から：反時計回り・回転 — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **闪耀方块**（分割・有料）：つなぎ目。名前から：四角いブロック・光る — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **震动条形**（分割・有料）：つなぎ目。名前から：帯・振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **顺时针三角**（分割・有料）：つなぎ目。名前から：時計回り — いま：ISF Radial（gl-transitions radial） — 借りる先：見つからない
- [ ] **顺时针旋转**（分割）：つなぎ目。名前から：時計回り・回転 — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **顺时针转场**（分割・有料）：つなぎ目。名前から：時計回り・回る — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **顺时针闪烁**（分割・有料）：つなぎ目。名前から：時計回り・点滅 — いま：ISF Radial（gl-transitions radial） — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **魔方碎片**（分割・有料）：つなぎ目。名前から：ルービックキューブ・破片 — いま：ISF cube（gl-transitions cube） — 借りる先：見つからない
- [ ] **黑板擦除**（分割・有料）：つなぎ目。名前から：拭き取る — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **2-Screen Swipe**（分割・有料）：つなぎ目。名前から：分割画面・滑る・動く — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **4帧切换**（分割・有料）：つなぎ目。名前から：コマ — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **Circular Slices II**（分割）：つなぎ目。名前から：分割・丸・帯に割れる（中国語名 圆形分割 II） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **Diagonal Slices**（分割）：つなぎ目。名前から：分割・斜め・帯に割れる（中国語名 斜向分割） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **Fold Over**（分割）：つなぎ目。名前から：折る — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **Horizontal Lines**（分割）：つなぎ目。名前から：帯に割れる（中国語名 横线） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Horizontal Slice**（分割）：つなぎ目。名前から：分割・帯に割れる（中国語名 横向分割） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Horizontal Triptych**（分割・有料）：つなぎ目。名前から：分割画面 — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **Open**（分割）：つなぎ目。名前から：幕が開く・幕・扉が開く（中国語名 开幕） — いま：無い — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **Open Horizontally**（分割）：つなぎ目。名前から：幕を引く・幕・扉が開く（中国語名 横向拉幕） — いま：無い — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **Open Vertically**（分割）：つなぎ目。名前から：幕を引く・幕・扉が開く（中国語名 竖向拉幕） — いま：無い — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **Split**（分割）：つなぎ目。名前から：分割・帯に割れる（中国語名 分割 I） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Split Drop**（分割・有料）：つなぎ目。名前から：帯に割れる・滑る・動く — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Split Drop 1**（分割・有料）：つなぎ目。名前から：帯に割れる・滑る・動く — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Split III**（分割）：つなぎ目。名前から：分割・帯に割れる（中国語名 分割 III） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Split IV**（分割）：つなぎ目。名前から：分割・帯に割れる（中国語名 分割 IV） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Then and Now**（分割）：つなぎ目。名前から：比べる（中国語名 岁月的痕迹） — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **Vertical Slices**（分割）：つなぎ目。名前から：分割・帯に割れる（中国語名 竖向分割） — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Whip Tear**（分割・有料）：つなぎ目。名前から：ちぎれる・滑る・動く — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **三屏放大**（分割・有料）：つなぎ目。名前から：3 分割画面・拡大 — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **三角分割**（分割・有料）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **倾斜分割**（分割・有料）：つなぎ目。名前から：分割・傾く — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **倾斜切片**（分割・有料）：つなぎ目。名前から：切れ目・傾く — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **光剑切割**（分割・有料）：つなぎ目。名前から：切る・光 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions Overexposure（MIT）
- [ ] **几何分割**（分割・有料）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **分割**（分割・有料）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **分割 II**（分割）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **分割缩放**（分割・有料）：つなぎ目。名前から：分割・拡大縮小 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **分割翻转**（分割・有料）：つなぎ目。名前から：分割・裏返る — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **分割转场**（分割・有料）：つなぎ目。名前から：分割・回る — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **切片滑动**（分割・有料）：つなぎ目。名前から：切れ目・滑る — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **前后对比**（分割・有料）：つなぎ目。名前から：比べる・前後比較 — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **四格展开**（分割・有料）：つなぎ目。名前から：4 分割画面・広がる — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **四边展开**（分割・有料）：つなぎ目。名前から：広がる — いま：無い — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **圆形分割**（分割・有料）：つなぎ目。名前から：分割・丸 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **圆形滚动**（分割・有料）：つなぎ目。名前から：丸・スクロール — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **圆形遮罩**（分割）：つなぎ目。名前から：丸 — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **圆形遮罩 II**（分割）：つなぎ目。名前から：丸 — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **多屏球形**（分割・有料）：つなぎ目。名前から：多分割画面 — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **多边形展开**（分割・有料）：つなぎ目。名前から：広がる — いま：無い — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **大圆盘**（分割・有料）：つなぎ目。名前から：丸 — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **对折**（分割・有料）：つなぎ目。名前から：折る — いま：無い — 借りる先：gl-transitions Fold（MIT）
- [ ] **对比分屏**（分割・有料）：つなぎ目。名前から：分割画面・比べる — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **对角切割**（分割・有料）：つなぎ目。名前から：切る — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **居中分割**（分割・有料）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **拼贴旋转**（分割・有料）：つなぎ目。名前から：コラージュ・回転 — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **揉皱圆圈**（分割・有料）：つなぎ目。名前から：丸・輪・しわ — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **撕纸快拼**（分割・有料）：つなぎ目。名前から：紙をちぎる・つなぎ合わせ — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **撕纸拉屏**（分割・有料）：つなぎ目。名前から：紙をちぎる・引く — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **斜向撕纸**（分割・有料）：つなぎ目。名前から：紙をちぎる・斜め — いま：無い — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **方形分割**（分割・有料）：つなぎ目。名前から：分割 — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **混乱分割**（分割・有料）：つなぎ目。名前から：分割・乱れ — いま：無い — 借りる先：gl-transitions windowslice（MIT、ISF Window Slice）／gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **滑块拼贴**（分割・有料）：つなぎ目。名前から：コラージュ — いま：無い — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **甜甜圈裁剪**（分割・有料）：つなぎ目。名前から：輪 — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）
- [ ] **百叶翻折**（分割・有料）：つなぎ目。名前から：ブラインド・折る・めくる — いま：無い — 借りる先：gl-transitions windowblinds（MIT、ISF Window Blinds）／gl-transitions Fold（MIT）
- [ ] **碎裂滑动**（分割・有料）：つなぎ目。名前から：砕ける・滑る — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **纸张飞落**（分割・有料）：つなぎ目。名前から：紙・飛ぶ・落ちる — いま：無い — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **纸质快转**（分割・有料）：つなぎ目。名前から：紙・回る — いま：無い — 借りる先：gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **翻折转图**（分割・有料）：つなぎ目。名前から：折る・回る・めくる — いま：無い — 借りる先：gl-transitions Fold（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **花卉剪纸**（分割・有料）：つなぎ目。名前から：紙・花 — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **负圆**（分割・有料）：つなぎ目。名前から：丸・ネガ — いま：無い — 借りる先：gl-transitions circleopen（MIT、ISF Circle Open）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **交叉破裂**（分割・有料）：つなぎ目。名前から：破裂 — いま：無い — 借りる先：見つからない
- [ ] **划痕纸**（分割・有料）：つなぎ目。名前から：紙 — いま：無い — 借りる先：見つからない
- [ ] **刮划**（分割・有料）：つなぎ目。名前から：ひっかく — いま：無い — 借りる先：見つからない
- [ ] **多元宇宙碎裂**（分割・有料）：つなぎ目。名前から：砕ける — いま：無い — 借りる先：見つからない
- [ ] **对角撕纸**（分割・有料）：つなぎ目。名前から：紙をちぎる — いま：無い — 借りる先：見つからない
- [ ] **折痕胶带**（分割・有料）：つなぎ目。名前から：折り目 — いま：無い — 借りる先：見つからない
- [ ] **撕纸**（分割）：つなぎ目。名前から：紙をちぎる — いま：無い — 借りる先：見つからない
- [ ] **撕纸出现**（分割・有料）：つなぎ目。名前から：紙をちぎる — いま：無い — 借りる先：見つからない
- [ ] **未压皱**（分割・有料）：つなぎ目。名前から：しわ — いま：無い — 借りる先：見つからない
- [ ] **破碎切换**（分割・有料）：つなぎ目。名前から：割れる — いま：無い — 借りる先：見つからない
- [ ] **碎纸**（分割・有料）：つなぎ目。名前から：紙 — いま：無い — 借りる先：見つからない
- [ ] **纸团**（分割・有料）：つなぎ目。名前から：紙 — いま：無い — 借りる先：見つからない
- [ ] **纸张侵蚀**（分割・有料）：つなぎ目。名前から：紙 — いま：無い — 借りる先：見つからない
- [ ] **蛋壳裂纹**（分割・有料）：つなぎ目。名前から：ひび — いま：無い — 借りる先：見つからない
- [ ] **轮胎刮痕**（分割・有料）：つなぎ目。名前から：ひっかく — いま：無い — 借りる先：見つからない
- [ ] **错位的碎片**（分割・有料）：つなぎ目。名前から：破片 — いま：無い — 借りる先：見つからない

### レンズ・ゆがみ（121）

- [ ] **Shaky Inhale**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **像素渐隐**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・じわっと消える — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **幻影波动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・幻影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **扭动漩涡旋转**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・くねる・回転 — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **扭曲摇摆**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・左右に揺れる — いま：ふるまい oscillator を 2 本に — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **扭曲模糊**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions DefocusBlur（MIT）
- [ ] **拼图波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・パズル — いま：ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **模糊震波**（レンズ・ゆがみ・有料）：つなぎ目。名前から：衝撃波・ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions DefocusBlur（MIT）
- [ ] **玻璃折射**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ガラス・折る — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions Fold（MIT）
- [ ] **融化轮廓**（レンズ・ゆがみ・有料）：つなぎ目。名前から：溶けて垂れる・輪郭 — いま：ISF Edges・Edge Blowout＋Glow＋ISF Fade でつなぐ — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **融化金**（レンズ・ゆがみ・有料）：つなぎ目。名前から：溶けて垂れる・金色 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **震动波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **震荡波切换**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **鱼眼拍摄**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）
- [ ] **Big Wave**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **Bump**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **Cartoon Swirl**（レンズ・ゆがみ）：つなぎ目。名前から：渦・アニメ・ねじれ・漫画（中国語名 动漫漩涡） — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **Color Swirl**（レンズ・ゆがみ）：つなぎ目。名前から：渦・カラー・ねじれ・虹（中国語名 彩色漩涡） — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **Curling Wave**（レンズ・ゆがみ）：つなぎ目。名前から：水の波紋・波打つ（中国語名 水波卷动） — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **Distortion**（レンズ・ゆがみ）：つなぎ目。名前から：ゆがみ（中国語名 扭曲） — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **Inhale**（レンズ・ゆがみ）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **Kaleidoscope**（レンズ・ゆがみ・有料）：つなぎ目。名前から：万華鏡 — いま：無い — 借りる先：ISF-Files Kaleidoscope（MIT、名前が同じ）／gl-transitions kaleidoscope（MIT、名前が同じ）
- [ ] **Kurzgesagt 液体**（レンズ・ゆがみ・有料）：つなぎ目。名前から：液体 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **Mosaic**（レンズ・ゆがみ）：つなぎ目。名前から：モザイク・ピクセル（中国語名 马赛克） — いま：無い — 借りる先：GPUImage MosaicFilter（BSD-3、名前が同じ）／ISF-Files Mosaic（MIT、名前が同じ）／gl-transitions Mosaic（MIT、名前が同じ）
- [ ] **Rainbow Filter**（レンズ・ゆがみ）：つなぎ目。名前から：波紋・カラー・虹・色（中国語名 彩色波纹） — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **Rainbow Twist**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・ねじれ・虹・色 — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **Slosh**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **Swirl**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・ねじれ — いま：無い — 借りる先：GPUImage SwirlFilter（BSD-3、名前が同じ）／ISF-Files Swirl（MIT、名前が同じ）／gl-transitions Swirl（MIT、名前が同じ）
- [ ] **Switch**（レンズ・ゆがみ）：つなぎ目。名前から：映り込み（中国語名 倒影） — いま：無い — 借りる先：gl-transitions StereoViewer（BSD-2）
- [ ] **Wave Left**（レンズ・ゆがみ）：つなぎ目。名前から：水の波紋・左へ・波打つ・滑る（中国語名 水波向左） — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Wave Right**（レンズ・ゆがみ）：つなぎ目。名前から：水の波紋・右へ・波打つ・滑る（中国語名 水波向右） — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Whirl**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・ねじれ — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **Whirlpool**（レンズ・ゆがみ）：つなぎ目。名前から：渦・ねじれ（中国語名 漩涡） — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **Wide Ripple**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・ちぎれる — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **万花筒**（レンズ・ゆがみ・有料）：つなぎ目。名前から：万華鏡 — いま：無い — 借りる先：gl-transitions kaleidoscope（MIT、ISF Kaleidoscope Transition）
- [ ] **丝滑马赛克**（レンズ・ゆがみ・有料）：つなぎ目。名前から：モザイク — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **俏皮像素化效果**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **催眠漩涡**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦 — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **像素条形分割**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・分割・帯 — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **像素洞**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **像素漫画**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・漫画 — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **像素爱心**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・ハート — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **像素粉**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・ピンク — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **光速扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・光 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions Overexposure（MIT）
- [ ] **六边形变焦**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **冲击波**（レンズ・ゆがみ・有料）：つなぎ目。名前から：衝撃波 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **剪切失真**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **变形旋转**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形・回転 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **叠化扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・重ねて溶ける — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **吃掉**（レンズ・ゆがみ・有料）：つなぎ目。名前から：食べる — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **向上波动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・上へ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **向左波动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・左へ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **吸收**（レンズ・ゆがみ・有料）：つなぎ目。名前から：吸い込む — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **大像素**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **弹动扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・弾む — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **感知变形**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **扭曲一闪**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・光る — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **扭曲像素**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・ピクセル — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **扭曲弹动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・弾む — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **扭曲扫光**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・光が走る — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions Overexposure（MIT）
- [ ] **扭曲溶解**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・溶ける — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **扭曲滑动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ・滑る — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **抽象扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **提丰扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **放大镜**（レンズ・ゆがみ・有料）：つなぎ目。名前から：虫めがね — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）
- [ ] **方形变焦**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **无缝扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **望远镜闪烁**（レンズ・ゆがみ・有料）：つなぎ目。名前から：望遠鏡・点滅 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **梦境变形**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形・夢の中 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions Overexposure（MIT）
- [ ] **气泡水滴**（レンズ・ゆがみ・有料）：つなぎ目。名前から：しずく・泡 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／Kenney Particle Pack（CC0）＋ISF Fade でつなぐ
- [ ] **水波切换**（レンズ・ゆがみ・有料）：つなぎ目。名前から：水の波紋 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **水滴**（レンズ・ゆがみ・有料）：つなぎ目。名前から：しずく — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **水滴 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：しずく — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **水滴溶解**（レンズ・ゆがみ・有料）：つなぎ目。名前から：しずく・溶ける — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **水纹变形特效**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **水面涟漪**（レンズ・ゆがみ・有料）：つなぎ目。名前から：さざ波・水面 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **波光粼粼**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波・光 — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions Overexposure（MIT）
- [ ] **波动 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **波动故障**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・グリッチ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **波浪变形特效**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・変形 — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **波浪滚动**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ・スクロール — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **波纹切换**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）
- [ ] **波纹镜头**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・カメラ — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **流光冲击波**（レンズ・ゆがみ・有料）：つなぎ目。名前から：衝撃波・流れる光 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions Overexposure（MIT）
- [ ] **液体扩散**（レンズ・ゆがみ・有料）：つなぎ目。名前から：液体 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **漩涡 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦 — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **漩涡切换**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦 — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）
- [ ] **漩涡扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・ゆがみ — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **炽烈变形**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **爱情冲击波**（レンズ・ゆがみ・有料）：つなぎ目。名前から：衝撃波・愛 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **电子波浪**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波打つ — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **画中画变形**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **白化像素**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **眼中倒影**（レンズ・ゆがみ・有料）：つなぎ目。名前から：映り込み・目 — いま：無い — 借りる先：gl-transitions StereoViewer（BSD-2）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ
- [ ] **绿光波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・光・緑 — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions Overexposure（MIT）
- [ ] **膨胀扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ゆがみ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **色散漩涡**（レンズ・ゆがみ・有料）：つなぎ目。名前から：渦・色の分散 — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions GlitchMemories（MIT、ISF Glitch Memories）
- [ ] **荡漾**（レンズ・ゆがみ・有料）：つなぎ目。名前から：揺らめく — いま：無い — 借りる先：gl-transitions ButterflyWaveScrawler・TilesWave（MIT）
- [ ] **融化**（レンズ・ゆがみ・有料）：つなぎ目。名前から：溶けて垂れる — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **融化 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：溶けて垂れる — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **融化飞溅**（レンズ・ゆがみ・有料）：つなぎ目。名前から：溶けて垂れる・飛ぶ — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **螺旋上升**（レンズ・ゆがみ・有料）：つなぎ目。名前から：らせん・上がる — いま：無い — 借りる先：gl-transitions Swirl（MIT、ISF Swirl）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **负片鱼眼**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・ネガ — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **赛博马赛克**（レンズ・ゆがみ・有料）：つなぎ目。名前から：モザイク・サイバー — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／ISF Neon（MIT）＋ISF Fade でつなぐ
- [ ] **迷幻波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・サイケ — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **透镜上移**（レンズ・ゆがみ・有料）：つなぎ目。名前から：レンズ・上へ動く — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **透镜下移**（レンズ・ゆがみ・有料）：つなぎ目。名前から：レンズ・下へ動く — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **透镜故障**（レンズ・ゆがみ・有料）：つなぎ目。名前から：レンズ・グリッチ — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **镜像翻转**（レンズ・ゆがみ・有料）：つなぎ目。名前から：鏡像・裏返る — いま：無い — 借りる先：gl-transitions StereoViewer（BSD-2）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **闪光像素**（レンズ・ゆがみ・有料）：つなぎ目。名前から：ピクセル・光る — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪光波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：波紋・光る — いま：無い — 借りる先：gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **闪光鱼眼**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・光る — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **颠覆变形**（レンズ・ゆがみ・有料）：つなぎ目。名前から：変形 — いま：無い — 借りる先：gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **马赛克 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：モザイク — いま：無い — 借りる先：gl-transitions mosaic_transition・pixelize（MIT、ISF Mosaic・Pixelize）
- [ ] **鱼眼**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）
- [ ] **鱼眼 II**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）
- [ ] **鱼眼 III**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）
- [ ] **鱼眼快照**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・スナップ — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions StripDatamoshGlitch（MIT）
- [ ] **鱼眼扭曲**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・ゆがみ — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **鱼眼旋转**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・回転 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **鱼眼波纹**（レンズ・ゆがみ・有料）：つなぎ目。名前から：魚眼・波紋 — いま：無い — 借りる先：gl-transitions flyeye（MIT、ISF Fly Eye）／gl-transitions ripple・WaterDrop（MIT、ISF Ripple Transition・Water Drop）

### 粒・飾り（85）

- [ ] **Hot Shimmers**（粒・飾り・有料）：つなぎ目。名前から：きらきら・星 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **Mix**（粒・飾り）：つなぎ目。名前から：重ねて溶ける・フェード（中国語名 叠化） — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **六边形Kira**（粒・飾り・有料）：つなぎ目。名前から：きらきら — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **星光叠化**（粒・飾り・有料）：つなぎ目。名前から：重ねて溶ける・星の光 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **星光炸开**（粒・飾り・有料）：つなぎ目。名前から：はじける・星の光 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ・ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **星星**（粒・飾り）：つなぎ目。名前から：星 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **星星 II**（粒・飾り・有料）：つなぎ目。名前から：星 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **星星冲屏**（粒・飾り・有料）：つなぎ目。名前から：星・画面に飛び込む — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **星星吸入**（粒・飾り・有料）：つなぎ目。名前から：星・吸い込む — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **星级切换**（粒・飾り・有料）：つなぎ目。名前から：星 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **枫叶烟花**（粒・飾り・有料）：つなぎ目。名前から：花火・葉 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **水晶簇**（粒・飾り・有料）：つなぎ目。名前から：水晶 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **水晶葡萄**（粒・飾り・有料）：つなぎ目。名前から：水晶 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **爆炸门**（粒・飾り・有料）：つなぎ目。名前から：爆発・扉 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **爱心冲击**（粒・飾り）：つなぎ目。名前から：ハート・衝撃 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **粉红爆炸**（粒・飾り・有料）：つなぎ目。名前から：爆発・赤・ピンク — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **衰退爆炸**（粒・飾り・有料）：つなぎ目。名前から：爆発 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **金粉**（粒・飾り・有料）：つなぎ目。名前から：金の粉 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **金色铭牌**（粒・飾り・有料）：つなぎ目。名前から：金色 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **钻石开场**（粒・飾り・有料）：つなぎ目。名前から：ダイヤ・オープニング — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions doorway・HorizontalOpen・VerticalOpen（MIT、ISF Doorway）
- [ ] **钻石消融**（粒・飾り・有料）：つなぎ目。名前から：ダイヤ・溶ける — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **银杏雨**（粒・飾り・有料）：つなぎ目。名前から：雨・銀色 — いま：点群 Appear Rain（点群だけ）＋ISF Fade でつなぐ・ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack（CC0）＋ISF Fade でつなぐ／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **雨刷转场**（粒・飾り・有料）：つなぎ目。名前から：雨・回る・刷毛 — いま：点群 Appear Rain（点群だけ）＋ISF Fade でつなぐ・ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：Kenney Particle Pack（CC0）＋ISF Fade でつなぐ／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **魔法爱心**（粒・飾り・有料）：つなぎ目。名前から：魔法・ハート — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **3D爱心**（粒・飾り・有料）：つなぎ目。名前から：ハート・3D — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **3D爱心气球**（粒・飾り・有料）：つなぎ目。名前から：ハート・3D — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）／gl-transitions cube・Revolve_Left（MIT）
- [ ] **Black smoke**（粒・飾り）：つなぎ目。名前から：煙・霧（中国語名 黑色烟雾） — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **Burn**（粒・飾り）：つなぎ目。名前から：燃える（中国語名 燃烧 Ⅰ） — いま：無い — 借りる先：ISF-Files Burn（MIT、名前が同じ）／gl-transitions burn（MIT、名前が同じ）
- [ ] **Like**（粒・飾り）：つなぎ目。名前から：ハート（中国語名 喜欢） — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **Particles**（粒・飾り）：つなぎ目。名前から：粒子（中国語名 粒子） — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **Snow**（粒・飾り）：つなぎ目。名前から：雪・漂う（中国語名 飘雪） — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity＋ISF Fade でつなぐ／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **Super Like**（粒・飾り）：つなぎ目。名前から：ハート（中国語名 超赞） — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **White Ink**（粒・飾り）：つなぎ目。名前から：花・墨・溶ける（中国語名 白色墨花） — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **White smoke**（粒・飾り）：つなぎ目。名前から：煙・霧（中国語名 白色烟雾） — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **中心切开**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **中心隐藏**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **二次元烟效**（粒・飾り・有料）：つなぎ目。名前から：煙 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **云朵**（粒・飾り）：つなぎ目。名前から：雲 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **云朵 II**（粒・飾り）：つなぎ目。名前から：雲 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **云爆发**（粒・飾り・有料）：つなぎ目。名前から：雲 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **云雾散开**（粒・飾り・有料）：つなぎ目。名前から：霧・雲 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **叠化反相**（粒・飾り・有料）：つなぎ目。名前から：重ねて溶ける — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **可爱龙龙**（粒・飾り・有料）：つなぎ目。名前から：愛 — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **圣诞树**（粒・飾り・有料）：つなぎ目。名前から：クリスマス — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で＋ISF Fade でつなぐ
- [ ] **圣诞礼盒**（粒・飾り・有料）：つなぎ目。名前から：クリスマス — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で＋ISF Fade でつなぐ
- [ ] **开心**（粒・飾り）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **心动缩放**（粒・飾り・有料）：つなぎ目。名前から：ときめき・拡大縮小 — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **心形飞溅**（粒・飾り・有料）：つなぎ目。名前から：ハート・飛ぶ — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **心脏爆裂**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **手势爱心2**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **手比心**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **柔彩云爆**（粒・飾り・有料）：つなぎ目。名前から：雲 — いま：無い — 借りる先：gl-transitions perlin（MIT）
- [ ] **樱花翻转**（粒・飾り・有料）：つなぎ目。名前から：桜・裏返る — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **水墨**（粒・飾り）：つなぎ目。名前から：水墨 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **水墨渲染**（粒・飾り・有料）：つなぎ目。名前から：水墨 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **沙漏**（粒・飾り・有料）：つなぎ目。名前から：砂時計 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **溶解缩入**（粒・飾り・有料）：つなぎ目。名前から：溶ける — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **溶解重组**（粒・飾り・有料）：つなぎ目。名前から：溶ける — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **火幕**（粒・飾り・有料）：つなぎ目。名前から：火 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **火焰**（粒・飾り・有料）：つなぎ目。名前から：炎 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **火焰泄漏**（粒・飾り・有料）：つなぎ目。名前から：炎 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **火焰爆发**（粒・飾り・有料）：つなぎ目。名前から：炎 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **火焰飞溅**（粒・飾り・有料）：つなぎ目。名前から：炎・飛ぶ — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **火箭发射**（粒・飾り・有料）：つなぎ目。名前から：火 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **烈焰升腾**（粒・飾り・有料）：つなぎ目。名前から：猛火 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **烈焰绽放**（粒・飾り・有料）：つなぎ目。名前から：猛火 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **烟雾弹**（粒・飾り・有料）：つなぎ目。名前から：煙・弾む — いま：無い — 借りる先：gl-transitions perlin（MIT）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **烟雾转场**（粒・飾り）：つなぎ目。名前から：煙・回る — いま：無い — 借りる先：gl-transitions perlin（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **燃烧 ⅠⅠ**（粒・飾り）：つなぎ目。名前から：燃える — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **燃烧 ⅠⅠⅠ**（粒・飾り）：つなぎ目。名前から：燃える — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **燃烧线**（粒・飾り・有料）：つなぎ目。名前から：燃える — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **爱心 I**（粒・飾り）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **爱心 II**（粒・飾り）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **爱心软糖**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **爱心遮罩**（粒・飾り・有料）：つなぎ目。名前から：ハート — いま：無い — 借りる先：gl-transitions heart（MIT、ISF Heart Transition）
- [ ] **立体雪花**（粒・飾り・有料）：つなぎ目。名前から：雪・立体 — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity＋ISF Fade でつなぐ／gl-transitions cube・Revolve_Left（MIT）
- [ ] **粒子传送门**（粒・飾り・有料）：つなぎ目。名前から：粒子・転送・扉 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **缩放燃烧**（粒・飾り・有料）：つなぎ目。名前から：燃える・拡大縮小 — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **花瓣飞舞**（粒・飾り・有料）：つなぎ目。名前から：花びら・飛ぶ — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **蓝光溶解**（粒・飾り・有料）：つなぎ目。名前から：溶ける・光・青 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）／gl-transitions Overexposure（MIT）
- [ ] **角落阴燃**（粒・飾り・有料）：つなぎ目。名前から：くすぶる・落ちる — いま：無い — 借りる先：gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **金沙**（粒・飾り・有料）：つなぎ目。名前から：金の砂 — いま：無い — 借りる先：gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **雪崩**（粒・飾り・有料）：つなぎ目。名前から：雪 — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity＋ISF Fade でつなぐ
- [ ] **雪花四散**（粒・飾り・有料）：つなぎ目。名前から：雪 — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity＋ISF Fade でつなぐ
- [ ] **雪花环绕**（粒・飾り・有料）：つなぎ目。名前から：雪・周りを回る — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity＋ISF Fade でつなぐ／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）

### グリッチ（69）

- [ ] **RGB 阴影**（グリッチ・有料）：つなぎ目。名前から：RGB ずれ・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **乱码胶片**（グリッチ・有料）：つなぎ目。名前から：文字化け・フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **乱码胶片 II**（グリッチ・有料）：つなぎ目。名前から：文字化け・フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **克隆错误**（グリッチ・有料）：つなぎ目。名前から：エラー・クローン — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **冰柱故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・氷 — いま：ISF Frosted Glass＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **复古撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける・レトロ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **故障弹窗转场**（グリッチ・有料）：つなぎ目。名前から：グリッチ・弾む・回る・窓 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **数字橡皮擦**（グリッチ・有料）：つなぎ目。名前から：デジタル・拭く — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **棱镜快拍**（グリッチ・有料）：つなぎ目。名前から：プリズム・打つ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）
- [ ] **波点乱码**（グリッチ・有料）：つなぎ目。名前から：文字化け・水玉 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **胶片定格**（グリッチ・有料）：つなぎ目。名前から：コマ止め・フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions StripDatamoshGlitch（MIT）
- [ ] **色散拖尾**（グリッチ・有料）：つなぎ目。名前から：色の分散・尾を引く — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **色散拖尾2**（グリッチ・有料）：つなぎ目。名前から：色の分散・尾を引く — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **色散模糊**（グリッチ・有料）：つなぎ目。名前から：色の分散・ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions DefocusBlur（MIT）
- [ ] **Color Glitch**（グリッチ）：つなぎ目。名前から：色ずれ・グリッチ・虹・色（中国語名 色差故障） — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **Glitch**（グリッチ）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **RGB Glitch**（グリッチ）：つなぎ目。名前から：グリッチ・カラー・色ずれ（中国語名 彩色故障） — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **RGB 弹射**（グリッチ・有料）：つなぎ目。名前から：RGB ずれ・弾む — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **RGB旋转**（グリッチ・有料）：つなぎ目。名前から：RGB ずれ・回転 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Signal Glitch 2**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **Urban Glitch**（グリッチ）：つなぎ目。名前から：グリッチ・強く光る（中国語名 故障爆闪） — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **三片撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **三重断裂**（グリッチ・有料）：つなぎ目。名前から：三重・切れる — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）
- [ ] **信号故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・信号の乱れ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **信号错误**（グリッチ・有料）：つなぎ目。名前から：エラー・信号の乱れ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **全息撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける・ホログラム — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **双重故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **同步错误**（グリッチ・有料）：つなぎ目。名前から：エラー — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **多屏定格**（グリッチ・有料）：つなぎ目。名前から：コマ止め・多分割画面 — いま：無い — 借りる先：gl-transitions StripDatamoshGlitch（MIT）／gl-transitions splitSlideIn*（MIT）
- [ ] **多彩错误**（グリッチ・有料）：つなぎ目。名前から：エラー — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **彩色快照**（グリッチ・有料）：つなぎ目。名前から：スナップ・カラー — いま：無い — 借りる先：gl-transitions StripDatamoshGlitch（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **快照切换 2**（グリッチ・有料）：つなぎ目。名前から：スナップ — いま：無い — 借りる先：gl-transitions StripDatamoshGlitch（MIT）
- [ ] **抽帧下移**（グリッチ・有料）：つなぎ目。名前から：コマ落とし・下へ動く — いま：無い — 借りる先：gl-transitions StripDatamoshGlitch（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **撕裂光晕**（グリッチ・有料）：つなぎ目。名前から：裂ける・光のにじみ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions Overexposure（MIT）
- [ ] **撕裂图层**（グリッチ・有料）：つなぎ目。名前から：裂ける — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **撕裂翻转**（グリッチ・有料）：つなぎ目。名前から：裂ける・裏返る — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **故障**（グリッチ）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **故障扩写**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **故障拉屏**（グリッチ・有料）：つなぎ目。名前から：グリッチ・引く — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **数字矩阵**（グリッチ・有料）：つなぎ目。名前から：デジタル — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **数字绘画**（グリッチ・有料）：つなぎ目。名前から：デジタル — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **旋转故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・回転 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **无缝撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **条码故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **棱镜旋转**（グリッチ・有料）：つなぎ目。名前から：プリズム・回転 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **棱镜火花**（グリッチ・有料）：つなぎ目。名前から：プリズム・火花 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／Kenney Particle Pack の火花（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **棱镜环**（グリッチ・有料）：つなぎ目。名前から：プリズム — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）
- [ ] **洋红故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・赤 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **液态故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・液体 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions crosswarp・displacement（MIT、ISF Crosswarp）
- [ ] **混乱剪裁**（グリッチ・有料）：つなぎ目。名前から：乱れ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **混音混乱**（グリッチ・有料）：つなぎ目。名前から：乱れ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **渐变撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける・グラデーション — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **滑动故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・滑る — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **炫彩定格**（グリッチ・有料）：つなぎ目。名前から：コマ止め・カラフル — いま：無い — 借りる先：gl-transitions StripDatamoshGlitch（MIT）／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **熔岩故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **电脑故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **电视故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・テレビ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **短路**（グリッチ・有料）：つなぎ目。名前から：ショート — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **碎片故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・破片 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **纸张撕裂特效**（グリッチ・有料）：つなぎ目。名前から：裂ける・紙 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **纸撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける・紙 — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **色块故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **色散光斑**（グリッチ・有料）：つなぎ目。名前から：色の分散・光の玉 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **色散闪烁**（グリッチ・有料）：つなぎ目。名前から：色の分散・点滅 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **色散闪烁 II**（グリッチ・有料）：つなぎ目。名前から：色の分散・点滅 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **色散闪烁 III**（グリッチ・有料）：つなぎ目。名前から：色の分散・点滅 — いま：無い — 借りる先：gl-transitions GlitchMemories（MIT、ISF Glitch Memories）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **被撕裂**（グリッチ・有料）：つなぎ目。名前から：裂ける — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）
- [ ] **负向故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ・ネガ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）／gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **青色故障**（グリッチ・有料）：つなぎ目。名前から：グリッチ — いま：無い — 借りる先：gl-transitions GlitchMemories・GlitchDisplace（MIT、ISF 版あり）

### ぼかし（37）

- [ ] **Blur**（ぼかし）：つなぎ目。名前から：ぼかし・回る（中国語名 转场-模糊） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **Blurred Highlight**（ぼかし）：つなぎ目。名前から：ぼかし（中国語名 亮点模糊） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **Horizontal Blur**（ぼかし）：つなぎ目。名前から：ぼかし（中国語名 横向模糊） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：three.js shaders HorizontalBlurShader（MIT、名前が同じ）／gl-transitions DefocusBlur（MIT）
- [ ] **Radial Blur**（ぼかし）：つなぎ目。名前から：ぼかし・放射ぼかし（中国語名 旋焦） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions CrossZoom（MIT、ISF CrossZoom）
- [ ] **Transform Shimmer**（ぼかし）：つなぎ目。名前から：ぼかし・細かいきらめき・きらきら・星（中国語名 模糊细闪） — いま：ISF Linear Blur（gl-transitions LinearBlur）・ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions DefocusBlur（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **Vertical Blur**（ぼかし）：つなぎ目。名前から：ぼかし（中国語名 竖向模糊） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：three.js shaders VerticalBlurShader（MIT、名前が同じ）／gl-transitions DefocusBlur（MIT）
- [ ] **Vertical Blur II**（ぼかし）：つなぎ目。名前から：ぼかし（中国語名 竖向模糊 II） — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：three.js shaders VerticalBlurShader（MIT、名前が同じ）／gl-transitions DefocusBlur（MIT）
- [ ] **垂直模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **形状模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **快闪模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・光る — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **斜向模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・斜め — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **方形模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **方形模糊 II**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **旋转模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・回転 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **条形模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・帯 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **模糊下扯**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **模糊冲刺**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **模糊切换**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **模糊扩展**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **模糊放大**（ぼかし・有料）：つなぎ目。名前から：ぼかし・拡大 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **模糊旋转**（ぼかし・有料）：つなぎ目。名前から：ぼかし・回転 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **模糊横移**（ぼかし・有料）：つなぎ目。名前から：ぼかし・横に動く — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **模糊滑动**（ぼかし・有料）：つなぎ目。名前から：ぼかし・滑る — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **模糊滑动 2**（ぼかし・有料）：つなぎ目。名前から：ぼかし・滑る — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **模糊滚动**（ぼかし・有料）：つなぎ目。名前から：ぼかし・スクロール — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **模糊熄灭**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **模糊缩小**（ぼかし・有料）：つなぎ目。名前から：ぼかし・縮小 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **泡泡模糊**（ぼかし）：つなぎ目。名前から：ぼかし・泡 — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／Kenney Particle Pack（CC0）＋ISF Fade でつなぐ
- [ ] **滑动模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・滑る — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **焦点缩放**（ぼかし・有料）：つなぎ目。名前から：焦点・拡大縮小 — いま：段の focus・aperture＋物の blur をキー＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **简单模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **美元模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **聚焦**（ぼかし・有料）：つなぎ目。名前から：ピント合わせ — いま：段の focus・aperture＋物の blur をキー＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **自动对焦**（ぼかし・有料）：つなぎ目。名前から：ピント合わせ — いま：段の focus・aperture＋物の blur をキー＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **轻盈模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **钻石模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・ダイヤ — いま：ISF Linear Blur（gl-transitions LinearBlur）・ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：gl-transitions DefocusBlur（MIT）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **镜头模糊**（ぼかし・有料）：つなぎ目。名前から：ぼかし・カメラ — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）

### 色（37）

- [ ] **Gradient Wipe**（色）：つなぎ目。名前から：グラデーション・拭き取る（中国語名 渐变擦除） — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **Thermal Imaging 3**（色・有料）：つなぎ目。名前から：熱カメラ — いま：ISF Thermal Camera＋ISF Fade でつなぐ — 借りる先：ISF Night Vision（MIT）＋ISF Fade でつなぐ
- [ ] **全息投影**（色・有料）：つなぎ目。名前から：ホログラム・影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **渐变律动**（色・有料）：つなぎ目。名前から：グラデーション・リズム — いま：ふるまい oscillator を scale に＋ISF Fade でつなぐ — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **炫彩渐隐**（色・有料）：つなぎ目。名前から：カラフル・じわっと消える — いま：ISF Fade（gl-transitions fade） — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **热成像**（色・有料）：つなぎ目。名前から：熱カメラ — いま：ISF Thermal Camera＋ISF Fade でつなぐ — 借りる先：ISF Night Vision（MIT）＋ISF Fade でつなぐ
- [ ] **热成像 II**（色・有料）：つなぎ目。名前から：熱カメラ — いま：ISF Thermal Camera＋ISF Fade でつなぐ — 借りる先：ISF Night Vision（MIT）＋ISF Fade でつなぐ
- [ ] **热成像运镜**（色・有料）：つなぎ目。名前から：熱カメラ・カメラワーク — いま：ISF Thermal Camera＋ISF Fade でつなぐ — 借りる先：ISF Night Vision（MIT）＋ISF Fade でつなぐ／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **红外闪光**（色・有料）：つなぎ目。名前から：赤外線・光る — いま：ISF Thermal Camera＋ISF Fade でつなぐ — 借りる先：ISF Night Vision（MIT）＋ISF Fade でつなぐ／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **迷幻震动**（色・有料）：つなぎ目。名前から：サイケ・振動 — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **黑白摇镜**（色・有料）：つなぎ目。名前から：白黒・揺れ — いま：ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：gl-transitions fadegrayscale（MIT、ISF Fade Gray Scale）
- [ ] **Blue Lines**（色）：つなぎ目。名前から：青・帯に割れる（中国語名 蓝色线条） — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **Dissolve**（色）：つなぎ目。名前から：色・溶ける（中国語名 色彩溶解） — いま：無い — 借りる先：gl-transitions dissolve（MIT、名前が同じ）
- [ ] **Dissolve II**（色）：つなぎ目。名前から：色・溶ける（中国語名 色彩溶解 II） — いま：無い — 借りる先：gl-transitions dissolve（MIT、名前が同じ）
- [ ] **Dissolve III**（色）：つなぎ目。名前から：色・溶ける（中国語名 色彩溶解 III） — いま：無い — 借りる先：gl-transitions dissolve（MIT、名前が同じ）
- [ ] **EDM 蓝调**（色・有料）：つなぎ目。名前から：青 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **下拉负片**（色・有料）：つなぎ目。名前から：ネガ・引く — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **幻觉**（色・有料）：つなぎ目。名前から：幻覚 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **弹跳黑白**（色・有料）：つなぎ目。名前から：白黒・弾む — いま：無い — 借りる先：gl-transitions fadegrayscale（MIT、ISF Fade Gray Scale）／gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **彩色滑片**（色・有料）：つなぎ目。名前から：カラー — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **彩虹烟雾**（色・有料）：つなぎ目。名前から：虹・煙 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions perlin（MIT）
- [ ] **彩虹焦散**（色・有料）：つなぎ目。名前から：虹 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **炫彩耀光**（色・有料）：つなぎ目。名前から：カラフル・光 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions Overexposure（MIT）
- [ ] **粉色迷幻**（色・有料）：つなぎ目。名前から：サイケ・ピンク — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **绿色危机**（色・有料）：つなぎ目。名前から：緑 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **色彩溶解 V**（色・有料）：つなぎ目。名前から：色・溶ける — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **色彩溶解 Ⅳ**（色・有料）：つなぎ目。名前から：色・溶ける — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions dissolve・perlin（MIT、ISF Perlin Transition）
- [ ] **色彩负片**（色・有料）：つなぎ目。名前から：ネガ・色 — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **蓝色捕捉**（色・有料）：つなぎ目。名前から：青 — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **负片粉色 2**（色・有料）：つなぎ目。名前から：ネガ・ピンク — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **负片红**（色・有料）：つなぎ目。名前から：ネガ・赤 — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **负粉色**（色・有料）：つなぎ目。名前から：ネガ・ピンク — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **负蓝**（色・有料）：つなぎ目。名前から：ネガ・青 — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert／gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **负面标记**（色・有料）：つなぎ目。名前から：ネガ — いま：無い — 借りる先：gl-transitions fadecolor（MIT）＋ISF Color Invert
- [ ] **镜头炫彩**（色・有料）：つなぎ目。名前から：カラフル・カメラ — いま：無い — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **黑白暂停**（色・有料）：つなぎ目。名前から：白黒 — いま：無い — 借りる先：gl-transitions fadegrayscale（MIT、ISF Fade Gray Scale）
- [ ] **Little Devil**（色）：つなぎ目。名前から：悪魔・ホラー（中国語名 小恶魔） — いま：無い — 借りる先：見つからない

### レトロ（34）

- [ ] **Film Burn**（レトロ・有料）：つなぎ目。名前から：古いフィルム・レトロ・燃える — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：ISF-Files Film Burn（MIT、名前が同じ）／gl-transitions FilmBurn（MIT、名前が同じ）
- [ ] **Vintage Screening**（レトロ）：つなぎ目。名前から：レトロ・古いフィルム・分割画面（中国語名 复古放映） — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions splitSlideIn*（MIT）
- [ ] **复古泛光**（レトロ・有料）：つなぎ目。名前から：レトロ・光のにじみ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **复古漏光**（レトロ・有料）：つなぎ目。名前から：レトロ・光漏れ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）＋ISF v002 Light Leak（MIT）
- [ ] **复古胶片**（レトロ・有料）：つなぎ目。名前から：フィルム・レトロ — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **复古闪白**（レトロ・有料）：つなぎ目。名前から：レトロ・白く光る — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **旧胶片**（レトロ）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **旧胶片 II**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **涂鸦胶片**（レトロ・有料）：つなぎ目。名前から：フィルム・落書き — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **电影剪辑**（レトロ・有料）：つなぎ目。名前から：映画 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **红胶片**（レトロ・有料）：つなぎ目。名前から：フィルム・赤 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions colorphase（MIT、ISF Color Phase）
- [ ] **胶卷**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **胶卷1**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **胶卷滑动**（レトロ・有料）：つなぎ目。名前から：フィルム・滑る — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **胶片切换**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **胶片孔痕**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **胶片擦除**（レトロ・有料）：つなぎ目。名前から：フィルム・拭き取る — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ・ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **胶片模糊**（レトロ・有料）：つなぎ目。名前から：フィルム・ぼかし — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ・ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions DefocusBlur（MIT）
- [ ] **胶片污渍**（レトロ・有料）：つなぎ目。名前から：フィルム — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **胶片滚动**（レトロ・有料）：つなぎ目。名前から：フィルム・スクロール — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **胶片磨花**（レトロ・有料）：つなぎ目。名前から：フィルム・花 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **荧光胶片**（レトロ・有料）：つなぎ目。名前から：フィルム・蛍光 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl＋ISF Fade でつなぐ — 借りる先：gl-transitions Overexposure（MIT）
- [ ] **颗粒模糊**（レトロ・有料）：つなぎ目。名前から：粒・ぼかし — いま：ISF Linear Blur（gl-transitions LinearBlur） — 借りる先：gl-transitions StaticFade（MIT）／gl-transitions DefocusBlur（MIT）
- [ ] **Recorder**（レトロ）：つなぎ目。名前から：ビデオカメラ・古いビデオ・テレビ（中国語名 摄像机） — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **TV Flickers**（レトロ・有料）：つなぎ目。名前から：古いビデオ・テレビ・点滅 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **屏幕切换**（レトロ・有料）：つなぎ目。名前から：画面 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **屏幕切换 2**（レトロ・有料）：つなぎ目。名前から：画面 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **屏幕异常**（レトロ・有料）：つなぎ目。名前から：画面 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **换台**（レトロ・有料）：つなぎ目。名前から：チャンネル切替 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **断电切换**（レトロ・有料）：つなぎ目。名前から：停電 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **未来屏幕**（レトロ・有料）：つなぎ目。名前から：画面 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **橙色屏幕**（レトロ・有料）：つなぎ目。名前から：画面 — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **电视替换**（レトロ・有料）：つなぎ目。名前から：テレビ — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）
- [ ] **电视穿越**（レトロ・有料）：つなぎ目。名前から：テレビ・通り抜ける — いま：無い — 借りる先：gl-transitions TVStatic・old_tv_lost_signal（MIT、ISF TV Static）／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）

### 枠（23）

- [ ] **Dirty Frame**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **刷子图框**（枠・有料）：つなぎ目。名前から：枠・刷毛 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ・ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：見つからない
- [ ] **图框弹跳**（枠・有料）：つなぎ目。名前から：枠・弾む — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **图框收缩**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **图框滑动**（枠・有料）：つなぎ目。名前から：枠・滑る — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **图框选择**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **对话框**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **弹框切换**（枠・有料）：つなぎ目。名前から：枠・弾む — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions Bounce（MIT、ISF Bounce）
- [ ] **手机屏放大**（枠・有料）：つなぎ目。名前から：スマホ・拡大 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **手机相机**（枠・有料）：つなぎ目。名前から：カメラ・スマホ — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **拉框入屏**（枠・有料）：つなぎ目。名前から：枠・引く — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：gl-transitions squeeze・directionalwarp（MIT、ISF Squeeze）
- [ ] **拍立得切换**（枠・有料）：つなぎ目。名前から：インスタントカメラ — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **播放界面**（枠）：つなぎ目。名前から：画面の枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **照片投送**（枠・有料）：つなぎ目。名前から：写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **界面切换**（枠・有料）：つなぎ目。名前から：画面の枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **相机剪影**（枠・有料）：つなぎ目。名前から：カメラ・影 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ・ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **相机网格**（枠・有料）：つなぎ目。名前から：カメラ・格子 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ・ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **相片切换**（枠・有料）：つなぎ目。名前から：写真 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **窗格**（枠）：つなぎ目。名前から：窓・格子 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ・ISF Grid Flip（gl-transitions GridFlip） — 借りる先：gl-transitions randomsquares・PuzzleRight（MIT）
- [ ] **视窗 95**（枠・有料）：つなぎ目。名前から：ウィンドウ — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **边框切换**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **边框左切**（枠・有料）：つなぎ目。名前から：枠 — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **音乐播放器 3**（枠・有料）：つなぎ目。名前から：プレーヤー — いま：形の stroke・四角（枠は形で描ける）＋ISF Fade でつなぐ — 借りる先：見つからない

### 絵の具（20）

- [ ] **Dots Right**（絵の具）：つなぎ目。名前から：水玉・右へ・網点・滑る（中国語名 波点向右） — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ — 借りる先：gl-transitions directional・Slides（MIT、ISF Directional）
- [ ] **动漫爆炸**（絵の具・有料）：つなぎ目。名前から：アニメ・爆発 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption＋ISF Fade でつなぐ — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **波点震动**（絵の具・有料）：つなぎ目。名前から：水玉・振動 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone＋ISF Fade でつなぐ・ふるまい noise を 2 本に（つなぎ目の前後） — 借りる先：見つからない
- [ ] **涂鸦刮擦**（絵の具・有料）：つなぎ目。名前から：落書き・拭く・ひっかく — いま：ISF Wipe Left・Right・Up・Down（gl-transitions wipe*） — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **涂鸦星星**（絵の具・有料）：つなぎ目。名前から：落書き・星 — いま：ISF Star（1 個）＋ISF Fade でつなぐ — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter＋ISF Fade でつなぐ
- [ ] **Cloud**（絵の具）：つなぎ目。名前から：アニメ・雲・煙・霧（中国語名 动漫云朵） — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ／gl-transitions perlin（MIT）
- [ ] **Flame**（絵の具）：つなぎ目。名前から：アニメ・炎（中国語名 动漫火焰） — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ／gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **动漫变速**（絵の具・有料）：つなぎ目。名前から：アニメ・速さが変わる — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ／gl-transitions SimpleZoom・CrossZoom・zoomInOut（MIT、ISF Simple Zoom Transition・CrossZoom）
- [ ] **卡通火光蔓延**（絵の具・有料）：つなぎ目。名前から：カートゥーン・火の光 — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ／gl-transitions burn・FilmBurn（MIT、ISF Burn・Film Burn）
- [ ] **标记涂鸦**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **涂鸦分割**（絵の具・有料）：つなぎ目。名前から：落書き・分割 — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ／gl-transitions windowslice（MIT、ISF Window Slice）
- [ ] **涂鸦切换**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **涂鸦翻转**（絵の具・有料）：つなぎ目。名前から：落書き・裏返る — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **涂鸦遮黑**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **漫画撕纸**（絵の具・有料）：つなぎ目。名前から：漫画・紙をちぎる — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **漫画面板**（絵の具・有料）：つなぎ目。名前から：漫画 — いま：無い — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT）＋ISF Fade でつなぐ
- [ ] **白色涂鸦**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **粗糙涂鸦**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **艺术涂鸦**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ
- [ ] **艺术涂鸦 II**（絵の具・有料）：つなぎ目。名前から：落書き — いま：無い — 借りる先：ISF Doodler（MIT）＋ISF Fade でつなぐ

### 残像（14）

- [ ] **克隆翻转**（残像・有料）：つなぎ目。名前から：クローン・裏返る — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ／gl-transitions SimpleFlip・BookFlip（MIT）
- [ ] **分身拼贴**（残像・有料）：つなぎ目。名前から：分身・コラージュ — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ／gl-transitions splitSlideIn*（MIT）
- [ ] **剪影位移**（残像・有料）：つなぎ目。名前から：影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **叠影幻灯**（残像・有料）：つなぎ目。名前から：重なる像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **叠影旋转**（残像・有料）：つなぎ目。名前から：重なる像・回転 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ／gl-transitions rotate_scale_fade・rotateTransition（MIT、ISF Rotate Scale Fade）
- [ ] **复制品画廊**（残像・有料）：つなぎ目。名前から：複製 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **幻影**（残像・有料）：つなぎ目。名前から：幻影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **幻影疾驰**（残像・有料）：つなぎ目。名前から：幻影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **灵魂出窍1**（残像・有料）：つなぎ目。名前から：魂が抜ける（拡大した写しを半透明で重ねる） — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **灼片反影**（残像・有料）：つなぎ目。名前から：影 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **电子叠影**（残像・有料）：つなぎ目。名前から：重なる像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **轻巧叠影**（残像・有料）：つなぎ目。名前から：重なる像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **重影上划**（残像・有料）：つなぎ目。名前から：二重像 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ
- [ ] **黑白重影**（残像・有料）：つなぎ目。名前から：二重像・白黒 — いま：ISF Echo Trace・RGB Trails 3.0・形の trail＋ISF Fade でつなぐ — 借りる先：ISF Ghosting（MIT）＋ISF Fade でつなぐ／gl-transitions fadegrayscale（MIT、ISF Fade Gray Scale）

### 人（2）

- [ ] **Blink**（人）：つなぎ目。名前から：目・点滅（中国語名 眨眼） — いま：無い — 借りる先：Animista blink-1（BSD-2、名前が同じ）／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ／gl-transitions fadecolor（MIT、ISF Fade Color）
- [ ] **街头路标**（人・有料）：つなぎ目。名前から：頭 — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion＋ISF Fade でつなぐ

### 文字（2）

- [ ] **倒计时突袭**（文字・有料）：つなぎ目。名前から：カウントダウン — いま：animators print_on・print_off＋ISF Fade でつなぐ — 借りる先：見つからない
- [ ] **十字切分**（文字・有料）：つなぎ目。名前から：字 — いま：animators print_on・print_off＋ISF Fade でつなぐ — 借りる先：見つからない

### その他（85）

- [ ] **百叶窗**（その他）：つなぎ目。名前から：ブラインド — いま：無い — 借りる先：gl-transitions windowblinds（MIT、ISF Window Blinds）
- [ ] **百叶窗 II**（その他・有料）：つなぎ目。名前から：ブラインド — いま：無い — 借りる先：gl-transitions windowblinds（MIT、ISF Window Blinds）
- [ ] **2024回忆流**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **GPS 指示**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Hand gesture**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **下扭滑页**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **丑角催眠**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **交叉剪辑**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **亮度控制**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **位移器**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **倒置半屏**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **偏移重复**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **分解**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **划掉**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **列车驶过**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **剪切与扩写**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **剪裁切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **剪贴簿装裱**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **剪辑场景混搭**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **劈开**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **动作剪辑**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **半色调切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **双图切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **后台切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **咬嚼特效**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **回忆**（その他）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **回忆 II**（その他）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **塑料封面**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **多层环形**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **多彩破坏**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **平滑切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **应用切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **开箱**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **强劲下扯**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **往复剪辑**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **快速三段切**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **快速切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **怪异轨迹**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **怪物干扰**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **打孔特效**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拆分旋切**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拆解**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拖动切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拖动切换特效**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **旧杂志**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **柔彩雏菊**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **桌面切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **横向显现**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **歌曲更改**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **气态绘画**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **浏览画廊**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **点侵蚀**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **点切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **热捕捉**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **狗仔队**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **球体升起**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **瓶堆**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **生气**（その他）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **生物危害视觉**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **白色盒子**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **目的地更改**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **竖线**（その他）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **线性加深**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **绘画画廊**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **绽放剪切**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **编织解构**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **美元**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **美元 II**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **美元符号**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **脏乱**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **航班目的地**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **被破坏的文件**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **西瓜乐趣**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **超音速**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **轻盈爆裂**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **选择切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **逐一裁剪**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **重叠**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **重复切换**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **重复底片**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **重新定位**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **重新排列**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **铃铛**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **顺滑标记**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **验证码切换 2**（その他・有料）：つなぎ目。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない

## Animations › In（クリップの入り）（251）

Motolii に有る 216・近い物あり 19・見つからない 16。

### 動き（84）

- [ ] **Crisscross Shake**（動き・有料）：クリップの入り。名前から：揺れ。既定 0.8 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Drop down**（動き・有料）：クリップの入り。名前から：滑る・動く。既定 1.4 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fade In**（動き）：クリップの入り。名前から：じわっと出る・フェード（中国語名 渐显）。既定 0.5 秒 — いま：animators fade_in・fade_out — 借りる先：Animista fade-in（BSD-2、名前が同じ）／Animate.css fadeIn（v3.7.2 は MIT、名前が同じ）
- [ ] **Mini Zoom**（動き・有料）：クリップの入り。名前から：拡大・縮小・寄る（中国語名 轻微放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Puzzle**（動き）：クリップの入り。名前から：滑る・パズル・ブロック（中国語名 碎块滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out・ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **Rock Horizontally**（動き）：クリップの入り。名前から：揺れ・揺れ動く（振り子）（中国語名 左右抖动）。既定 0.5 秒 — いま：ふるまい noise・animators swing_3d_in・swing_3d_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Rock Vertically**（動き）：クリップの入り。名前から：揺れ・揺れ動く（振り子）（中国語名 上下抖动）。既定 0.5 秒 — いま：ふるまい noise・animators swing_3d_in・swing_3d_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Roll Right**（動き）：クリップの入り。名前から：回って入る・右へ・回転・滑る（中国語名 向右转入）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista roll-in-right（BSD-2、名前が同じ）
- [ ] **Rotate**（動き）：クリップの入り。名前から：回転（中国語名 旋转）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animate.css rotateIn（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Shake 1**（動き）：クリップの入り。名前から：揺れ（中国語名 轻微抖动）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Shake 2**（動き）：クリップの入り。名前から：揺れ（中国語名 轻微抖动 II）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Shake 3**（動き）：クリップの入り。名前から：揺れ（中国語名 轻微抖动 III）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Shake Down**（動き）：クリップの入り。名前から：揺れ・下がる・滑る・動く（中国語名 抖动下降）。既定 0.5 秒 — いま：ふるまい noise・animators slide_in・slide_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Skew Shake**（動き・有料）：クリップの入り。名前から：揺れ・伸ばす・傾く。既定 1.0 秒 — いま：ふるまい noise・キー scale_x・skew — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **Slide Down**（動き）：クリップの入り。名前から：滑る・下へ・動く（中国語名 向下滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideInDown（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Left**（動き）：クリップの入り。名前から：滑る・左へ・動く（中国語名 向左滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-in-left（BSD-2、名前が同じ）／Animate.css slideInLeft（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Right**（動き）：クリップの入り。名前から：滑る・右へ・動く（中国語名 向右滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-in-right（BSD-2、名前が同じ）／Animate.css slideInRight（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Up**（動き）：クリップの入り。名前から：滑る・上へ・動く（中国語名 向上滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideInUp（v3.7.2 は MIT、名前が同じ）
- [ ] **Spin Left**（動き）：クリップの入り。名前から：回って入る・左へ・回転・滑る（中国語名 向左转入）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin Up 1**（動き）：クリップの入り。名前から：回って入る・上へ・回転・滑る（中国語名 向上转入）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin Up 2**（動き）：クリップの入り。名前から：回って入る・上へ・回転・滑る（中国語名 向上转入 II）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Swing**（動き）：クリップの入り。名前から：振り子・揺れ動く（振り子）（中国語名 钟摆）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Bottom**（動き）：クリップの入り。名前から：振り込む・下へ・揺れ動く（振り子）（中国語名 向下甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Bottom Left**（動き）：クリップの入り。名前から：振り込む・左へ・揺れ動く（振り子）・滑る（中国語名 向左下甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Bottom Right**（動き）：クリップの入り。名前から：振り込む・右へ・揺れ動く（振り子）・滑る（中国語名 向右下甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Right**（動き）：クリップの入り。名前から：振り込む・右へ・揺れ動く（振り子）・滑る（中国語名 向右甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Top Left**（動き）：クリップの入り。名前から：振り込む・左へ・揺れ動く（振り子）・滑る（中国語名 向左上甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Swing Top Right**（動き）：クリップの入り。名前から：振り込む・右へ・揺れ動く（振り子）・滑る（中国語名 向右上甩入）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **X震动**（動き・有料）：クリップの入り。名前から：振動。既定 0.8 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Zoom 1**（動き）：クリップの入り。名前から：縮小・拡大・寄る（中国語名 动感缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom 2**（動き）：クリップの入り。名前から：拡大・縮小・寄る（中国語名 动感放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom In**（動き）：クリップの入り。名前から：拡大・縮小・寄る（中国語名 放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom Out**（動き）：クリップの入り。名前から：縮小・拡大・寄る（中国語名 缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomOut（v3.7.2 は MIT、名前が同じ）／gl-transitions zoomInOut（MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **冲撞**（動き・有料）：クリップの入り。名前から：ぶつかる。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **卡片滚动**（動き・有料）：クリップの入り。名前から：スクロール・カード。既定 2.0 秒 — いま：animators slide_in・slide_out・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **卡阵悬浮**（動き・有料）：クリップの入り。名前から：カード。既定 2.0 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **反转回正**（動き）：クリップの入り。名前から：回る。既定 1.3 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **叠加弹动**（動き・有料）：クリップの入り。名前から：弾む・重ねる。既定 1.0 秒 — いま：animators bounce_in・spring_in・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **向上滚动**（動き・有料）：クリップの入り。名前から：スクロール・上へ。既定 1.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **向上闪入**（動き・有料）：クリップの入り。名前から：上へ・光る。既定 0.7 秒 — いま：animators slide_in・slide_out・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **地球变焦**（動き・有料）：クリップの入り。名前から：ズーム。既定 1.7 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **多维空间**（動き・有料）：クリップの入り。名前から：空間。既定 1.4 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **幕布拉开**（動き・有料）：クリップの入り。名前から：引く。既定 2.0 秒 — いま：キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **强震抖动**（動き・有料）：クリップの入り。名前から：揺れ・強い揺れ。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抖动变焦**（動き・有料）：クリップの入り。名前から：ズーム・揺れ。既定 0.8 秒 — いま：キー scale（区間のイーズ）・ふるまい noise — 借りる先：Animista scale-in-center（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抖动横移**（動き・有料）：クリップの入り。名前から：揺れ・横に動く。既定 0.6 秒 — いま：ふるまい noise・animators slide_in・slide_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抽卡**（動き・有料）：クリップの入り。名前から：カードを引く。既定 1.4 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **拉丝滑入**（動き・有料）：クリップの入り。名前から：滑って入る・引く。既定 0.5 秒 — いま：animators slide_in・slide_out・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **摇一摇**（動き・有料）：クリップの入り。名前から：揺れ。既定 1.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **旋转进入**（動き・有料）：クリップの入り。名前から：回転。既定 2.0 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **晃落**（動き・有料）：クリップの入り。名前から：落ちる・揺れ。既定 1.3 秒 — いま：animators slide_in・slide_out・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **暴雪缩放**（動き・有料）：クリップの入り。名前から：拡大縮小・雪。既定 2.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity をキー
- [ ] **暴雪隧道**（動き・有料）：クリップの入り。名前から：トンネル・雪。既定 1.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity をキー
- [ ] **果冻I**（動き・有料）：クリップの入り。名前から：ゼリー。既定 0.8 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **果冻II**（動き・有料）：クリップの入り。名前から：ゼリー。既定 0.8 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **涂抹旋转**（動き・有料）：クリップの入り。名前から：回転・塗る。既定 1.0 秒 — いま：キー rotate・ISF Wipe をキー — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **滋滋碰击**（動き・有料）：クリップの入り。名前から：打つ。既定 1.2 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **滑片滑动**（動き・有料）：クリップの入り。名前から：滑る。既定 1.4 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **滚动进入**（動き・有料）：クリップの入り。名前から：スクロール。既定 1.6 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **炫目淡入**（動き・有料）：クリップの入り。名前から：フェードイン。既定 2.0 秒 — いま：animators fade_in・fade_out — 借りる先：見つからない
- [ ] **白色嗡鸣**（動き・有料）：クリップの入り。名前から：ぶーん（揺れ）。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **盘形抽卡**（動き・有料）：クリップの入り。名前から：カードを引く。既定 2.0 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **眩晕旋转**（動き・有料）：クリップの入り。名前から：めまい・回転。既定 2.0 秒 — いま：ふるまい noise・キー rotate — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **碰撞**（動き・有料）：クリップの入り。名前から：ぶつかる。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **空间拉扯**（動き・有料）：クリップの入り。名前から：引っぱる・空間。既定 1.0 秒 — いま：キー scale_x・skew・rotate_x・rotate_y・z — 借りる先：Animista tracking-in-expand（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **空间穿梭**（動き・有料）：クリップの入り。名前から：空間。既定 1.7 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **穿越虫洞**（動き・有料）：クリップの入り。名前から：通り抜ける・ワームホール。既定 2.0 秒 — いま：キー scale（区間のイーズ）・ISF Twirl をキー — 借りる先：Animista scale-in-center（BSD-2）／Animista swirl-in-fwd（BSD-2）
- [ ] **立体切换**（動き・有料）：クリップの入り。名前から：立体。既定 1.1 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **立体卡片**（動き・有料）：クリップの入り。名前から：立体・カード。既定 1.6 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **红色镜头**（動き・有料）：クリップの入り。名前から：カメラ・赤。既定 2.0 秒 — いま：キー scale（区間のイーズ）・ISF Multi Hue Shift をキー — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **维度穿越**（動き・有料）：クリップの入り。名前から：通り抜ける。既定 2.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **缓缓下落**（動き・有料）：クリップの入り。名前から：落ちる。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **翻卡**（動き・有料）：クリップの入り。名前から：めくる・カード。既定 1.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **耀斑震动**（動き・有料）：クリップの入り。名前から：振動・フレア。既定 1.2 秒 — いま：ふるまい noise・段の lens_flare・ISF Lens Flare をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **聚合**（動き・有料）：クリップの入り。名前から：集まる。既定 2.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **脉冲**（動き・有料）：クリップの入り。名前から：脈打つ。既定 0.9 秒 — いま：ふるまい oscillator を scale に — 借りる先：Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **轻巧摇入**（動き・有料）：クリップの入り。名前から：揺れ。既定 1.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **锯齿变速放大**（動き・有料）：クリップの入り。名前から：拡大・速さが変わる。既定 3.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **镜头晃动**（動き・有料）：クリップの入り。名前から：揺れ・カメラ。既定 2.0 秒 — いま：ふるまい noise・キー scale（区間のイーズ） — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **陨石汇聚**（動き・有料）：クリップの入り。名前から：集まる。既定 2.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **震动倒数**（動き・有料）：クリップの入り。名前から：振動・カウントダウン。既定 3.0 秒 — いま：ふるまい noise・animators print_on・print_off — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **飞行模式 2**（動き・有料）：クリップの入り。名前から：飛ぶ。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **飞近**（動き・有料）：クリップの入り。名前から：飛ぶ。既定 1.6 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **黑暗震颤**（動き・有料）：クリップの入り。名前から：揺れ・暗い。既定 2.0 秒 — いま：ふるまい noise・vignette.wgsl・段の vignette をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）

### 光（27）

- [ ] **Lumin Flash**（光・有料）：クリップの入り。名前から：光る・光る（フラッシュ）。既定 1.0 秒 — いま：物の glow をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **Lumin Matrix**（光・有料）：クリップの入り。名前から：光る。既定 1.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **RGB Scanlines**（光）：クリップの入り。名前から：光・引く・色ずれ・古いビデオ（中国語名 光效拉丝）。既定 1.0 秒 — いま：物の glow をキー・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）／ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **之字形闪光**（光・有料）：クリップの入り。名前から：光る・字。既定 1.0 秒 — いま：段の exposure をキー・animators print_on・print_off — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **冲屏闪现**（光・有料）：クリップの入り。名前から：ちらっと出る・画面に飛び込む。既定 1.0 秒 — いま：ふるまい random を opacity に・キー scale（区間のイーズ） — 借りる先：Animista flicker-in-1（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **快抖频闪**（光・有料）：クリップの入り。名前から：ストロボ・揺れ。既定 1.0 秒 — いま：ふるまい random を opacity に・ふるまい noise — 借りる先：Animista flicker-in-1（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **放射性红**（光・有料）：クリップの入り。名前から：放射・赤。既定 2.0 秒 — いま：ISF God Rays をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **旋转光线**（光・有料）：クリップの入り。名前から：光の筋・回転。既定 2.0 秒 — いま：ISF God Rays をキー・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **曝光放射**（光・有料）：クリップの入り。名前から：放射・露出。既定 0.8 秒 — いま：ISF God Rays をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **爆闪拉远**（光・有料）：クリップの入り。名前から：強く光る・引く。既定 2.0 秒 — いま：段の exposure をキー・キー scale（区間のイーズ） — 借りる先：Animista flicker-in-glow（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **特写闪光**（光・有料）：クリップの入り。名前から：光る・アップ。既定 2.0 秒 — いま：段の exposure をキー・キー scale（区間のイーズ） — 借りる先：Animista flicker-in-glow（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **电光热浪**（光・有料）：クリップの入り。名前から：電気の光・陽炎。既定 2.0 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **瞬间传送**（光・有料）：クリップの入り。名前から：転送・一瞬。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **聚光展入**（光・有料）：クリップの入り。名前から：光。既定 2.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **背光**（光・有料）：クリップの入り。名前から：光。既定 2.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **蓝色闪现**（光・有料）：クリップの入り。名前から：ちらっと出る・青。既定 1.0 秒 — いま：ふるまい random を opacity に・ISF Multi Hue Shift をキー — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **闪光光晕**（光・有料）：クリップの入り。名前から：光のにじみ・光る。既定 1.8 秒 — いま：物の glow をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪光入场**（光・有料）：クリップの入り。名前から：光る。既定 1.0 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪光弹**（光・有料）：クリップの入り。名前から：光る・弾む。既定 2.0 秒 — いま：段の exposure をキー・animators bounce_in・spring_in — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪屏**（光・有料）：クリップの入り。名前から：画面が光る。既定 1.2 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪现**（光・有料）：クリップの入り。名前から：ちらっと出る。既定 1.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **闪白缓入**（光・有料）：クリップの入り。名前から：白く光る。既定 1.2 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪速扯入**（光・有料）：クリップの入り。名前から：光る。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **扫描**（光・有料）：クリップの入り。名前から：スキャン。既定 0.6 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **流光一瞬**（光・有料）：クリップの入り。名前から：流れる光。既定 2.0 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **爱心光波监测**（光・有料）：クリップの入り。名前から：光の波・ハート。既定 2.0 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー／ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **闪电碎片**（光・有料）：クリップの入り。名前から：稲妻・破片。既定 2.0 秒 — いま：無い — 借りる先：見つからない

### 分割（27）

- [ ] **Cubic Flip**（分割・有料）：クリップの入り。名前から：立方体・裏返る。既定 1.1 秒 — いま：rotate_x・rotate_y・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in（BSD-2）／Animista flip-in-hor-bottom（BSD-2）
- [ ] **Oblique Lines**（分割・有料）：クリップの入り。名前から：帯に割れる・伸ばす・傾く。既定 0.7 秒 — いま：キー scale_x・skew — 借りる先：ISF Window Slice（MIT）をキー／Animista tracking-in-expand（BSD-2）
- [ ] **Puzzled up**（分割・有料）：クリップの入り。名前から：パズル・ブロック・滑る・動く。既定 1.7 秒 — いま：ISF Random Squares Mask をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Rotation Opening**（分割）：クリップの入り。名前から：幕が開く・回転・幕・扉が開く（中国語名 旋转开幕）。既定 1.0 秒 — いま：キー rotate — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Screen Wipe**（分割・有料）：クリップの入り。名前から：拭き取る・分割画面（中国語名 画面擦除）。既定 2.0 秒 — いま：ISF Wipe をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **Tesseract**（分割・有料）：クリップの入り。名前から：立方体。既定 1.3 秒 — いま：rotate_x・rotate_y — 借りる先：Animista flip-in（BSD-2）
- [ ] **Vibrating Panels**（分割）：クリップの入り。名前から：分割画面・振動・揺れ（中国語名 分屏震动）。既定 2.0 秒 — いま：ISF Quad Tile をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **三屏切闪**（分割・有料）：クリップの入り。名前から：3 分割画面・光る。既定 1.0 秒 — いま：ISF Quad Tile をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **分割嗡嗡声**（分割・有料）：クリップの入り。名前から：分割・ぶーん（揺れ）。既定 2.0 秒 — いま：ふるまい noise — 借りる先：ISF Window Slice（MIT）をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **分屏横移**（分割・有料）：クリップの入り。名前から：分割画面・横に動く。既定 1.0 秒 — いま：ISF Quad Tile をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **分屏翻转**（分割・有料）：クリップの入り。名前から：分割画面・裏返る。既定 0.7 秒 — いま：ISF Quad Tile をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **动感拼接**（分割・有料）：クリップの入り。名前から：つなぎ合わせ。既定 2.0 秒 — いま：ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **圆形下坠**（分割・有料）：クリップの入り。名前から：丸・落ちる。既定 1.0 秒 — いま：animators slide_in・slide_out — 借りる先：gl-transitions circleopen（MIT）をキー
- [ ] **微震开幕**（分割・有料）：クリップの入り。名前から：幕が開く・揺れ。既定 0.9 秒 — いま：ふるまい noise — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **拼图**（分割・有料）：クリップの入り。名前から：パズル。既定 1.1 秒 — いま：ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **斜切开幕**（分割・有料）：クリップの入り。名前から：幕が開く・斜め。既定 1.7 秒 — いま：キー scale_x・skew — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista tracking-in-expand（BSD-2）
- [ ] **立方分体**（分割・有料）：クリップの入り。名前から：立方体。既定 1.5 秒 — いま：rotate_x・rotate_y — 借りる先：Animista flip-in（BSD-2）
- [ ] **翻书**（分割・有料）：クリップの入り。名前から：本をめくる。既定 1.6 秒 — いま：ISF Inverted Page Curl をキー — 借りる先：見つからない
- [ ] **转圈圈**（分割・有料）：クリップの入り。名前から：輪・回る。既定 0.8 秒 — いま：キー rotate — 借りる先：gl-transitions circleopen（MIT）をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **魔方 III**（分割・有料）：クリップの入り。名前から：ルービックキューブ。既定 1.1 秒 — いま：rotate_x・rotate_y — 借りる先：Animista flip-in（BSD-2）
- [ ] **Cross Open**（分割・有料）：クリップの入り。名前から：帯に割れる・幕・扉が開く。既定 1.1 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー／gl-transitions HorizontalOpen（MIT）をキー
- [ ] **Folding Fan**（分割・有料）：クリップの入り。名前から：折る。既定 1.1 秒 — いま：無い — 借りる先：Animista flip-in（BSD-2）
- [ ] **分割**（分割・有料）：クリップの入り。名前から：分割。既定 1.0 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **展开**（分割・有料）：クリップの入り。名前から：広がる。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **条形展开**（分割・有料）：クリップの入り。名前から：帯・広がる。既定 1.0 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー／gl-transitions HorizontalOpen（MIT）をキー
- [ ] **破镜重圆**（分割・有料）：クリップの入り。名前から：丸。既定 1.7 秒 — いま：無い — 借りる先：gl-transitions circleopen（MIT）をキー
- [ ] **黑边展开**（分割・有料）：クリップの入り。名前から：広がる。既定 2.0 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー

### 粒・飾り（25）

- [ ] **Flame Risen**（粒・飾り・有料）：クリップの入り。名前から：焼ける・炎・滑る・動く（中国語名 灼烧出现）。既定 2.9 秒 — いま：点群の Burn Away をキー・animators slide_in・slide_out — 借りる先：ISF Film Burn・Undulating Burn Out（MIT） をキー／Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Kira游动**（粒・飾り・有料）：クリップの入り。名前から：きらきら。既定 2.3 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **MC爆炸**（粒・飾り）：クリップの入り。名前から：爆発。既定 2.0 秒 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption をキー — 借りる先：見つからない
- [ ] **Wiper**（粒・飾り）：クリップの入り。名前から：雨・刷毛・拭き取る（中国語名 雨刷）。既定 0.5 秒 — いま：点群 Appear Rain（点群だけ） をキー・ISF Wipe をキー — 借りる先：Kenney Particle Pack（CC0） をキー
- [ ] **仙女的彗星**（粒・飾り・有料）：クリップの入り。名前から：妖精・星。既定 2.0 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **墨迹显现**（粒・飾り・有料）：クリップの入り。名前から：墨。既定 2.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **散落花瓣**（粒・飾り・有料）：クリップの入り。名前から：花びら・落ちる。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter をキー
- [ ] **旋转火焰**（粒・飾り・有料）：クリップの入り。名前から：炎・回転。既定 2.0 秒 — いま：キー rotate — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **流金**（粒・飾り・有料）：クリップの入り。名前から：金色。既定 1.5 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **火焰冲击动画**（粒・飾り・有料）：クリップの入り。名前から：炎・衝撃。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **烟雾弹**（粒・飾り・有料）：クリップの入り。名前から：煙・弾む。既定 1.2 秒 — いま：段の fog をキー・animators bounce_in・spring_in — 借りる先：Kenney Particle Pack の煙（CC0） をキー
- [ ] **粒子爱心**（粒・飾り・有料）：クリップの入り。名前から：粒子・ハート。既定 3.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）／ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **金沙**（粒・飾り・有料）：クリップの入り。名前から：金の砂。既定 2.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **魔法出现**（粒・飾り・有料）：クリップの入り。名前から：魔法。既定 2.0 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **魔法爆发**（粒・飾り・有料）：クリップの入り。名前から：魔法。既定 2.0 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **魔法睁眼**（粒・飾り・有料）：クリップの入り。名前から：魔法・目。既定 2.0 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー／MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion をキー
- [ ] **魔法粒子I**（粒・飾り・有料）：クリップの入り。名前から：粒子・魔法。既定 3.0 秒 — いま：点群 dissolve（点群だけ）・ISF Star（1 個） をキー — 借りる先：Animista puff-in-center（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **魔法粒子II**（粒・飾り・有料）：クリップの入り。名前から：粒子・魔法。既定 3.0 秒 — いま：点群 dissolve（点群だけ）・ISF Star（1 個） をキー — 借りる先：Animista puff-in-center（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **2025**（粒・飾り・有料）：クリップの入り。名前から：季節の行事。既定 1.5 秒 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で をキー
- [ ] **夜焰**（粒・飾り・有料）：クリップの入り。名前から：炎。既定 2.0 秒 — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **心灵门户**（粒・飾り・有料）：クリップの入り。名前から：ハート・扉。既定 2.0 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー／gl-transitions HorizontalOpen（MIT）をキー
- [ ] **枫叶遮罩**（粒・飾り・有料）：クリップの入り。名前から：葉。既定 2.3 秒 — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter をキー
- [ ] **温柔火焰**（粒・飾り・有料）：クリップの入り。名前から：炎。既定 2.0 秒 — いま：無い — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **爱心信封**（粒・飾り・有料）：クリップの入り。名前から：ハート。既定 1.0 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **画出爱心**（粒・飾り・有料）：クリップの入り。名前から：ハート。既定 1.6 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー

### レンズ・ゆがみ（25）

- [ ] **Chroma Wave**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波打つ・色ずれ。既定 0.8 秒 — いま：animators（文字ごとにずらす）・rgbshift.wgsl をキー — 借りる先：anime.js stagger（MIT）
- [ ] **Flip**（レンズ・ゆがみ）：クリップの入り。名前から：鏡像・裏返る（中国語名 镜像翻转）。既定 0.5 秒 — いま：mirror.wgsl をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Whirl**（レンズ・ゆがみ）：クリップの入り。名前から：渦・回転・ねじれ（中国語名 漩涡旋转）。既定 0.5 秒 — いま：ISF Twirl をキー・キー rotate — 借りる先：Animista swirl-in-fwd（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **像素拼图**（レンズ・ゆがみ・有料）：クリップの入り。名前から：ピクセル・パズル。既定 2.0 秒 — いま：ISF Pixellate をキー・ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **冰块**（レンズ・ゆがみ・有料）：クリップの入り。名前から：氷。既定 1.3 秒 — いま：ISF Frosted Glass をキー — 借りる先：見つからない
- [ ] **冲击波入口**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 2.0 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **反物质冲击波**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 2.0 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **彩震幻波**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波・揺れ。既定 2.0 秒 — いま：animators（文字ごとにずらす）・ふるまい noise — 借りる先：anime.js stagger（MIT）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **模糊涟漪**（レンズ・ゆがみ・有料）：クリップの入り。名前から：さざ波・ぼかし。既定 1.1 秒 — いま：ISF Ripples をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **波浪慢扭**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波打つ。既定 2.0 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **波浪震动**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波打つ・振動。既定 1.0 秒 — いま：animators（文字ごとにずらす）・ふるまい noise — 借りる先：anime.js stagger（MIT）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **波纹油画**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波紋・油絵。既定 0.8 秒 — いま：ISF Ripples をキー — 借りる先：GPUImage3 KuwaharaFilter（BSD-3） をキー
- [ ] **液态玻璃**（レンズ・ゆがみ）：クリップの入り。名前から：ガラス・液体。既定 1.2 秒 — いま：ISF Frosted Glass をキー・ISF Bump Distortion をキー — 借りる先：見つからない
- [ ] **玻璃切入**（レンズ・ゆがみ・有料）：クリップの入り。名前から：ガラス。既定 1.0 秒 — いま：ISF Frosted Glass をキー — 借りる先：見つからない
- [ ] **砸出波纹**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波紋。既定 1.6 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **空间扭曲**（レンズ・ゆがみ・有料）：クリップの入り。名前から：ゆがみ・空間。既定 1.2 秒 — いま：ISF Bump Distortion をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **虫洞入口**（レンズ・ゆがみ・有料）：クリップの入り。名前から：ワームホール。既定 1.0 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **进入冲击波**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 2.0 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **震动波纹**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波紋・振動。既定 1.5 秒 — いま：ISF Ripples をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **震波**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 0.8 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **震波 II**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 0.8 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **震波 III**（レンズ・ゆがみ・有料）：クリップの入り。名前から：衝撃波。既定 1.7 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **马赛克**（レンズ・ゆがみ・有料）：クリップの入り。名前から：モザイク。既定 1.0 秒 — いま：ISF Pixellate をキー — 借りる先：見つからない
- [ ] **黑洞**（レンズ・ゆがみ・有料）：クリップの入り。名前から：ブラックホール。既定 0.7 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **黑白波纹**（レンズ・ゆがみ・有料）：クリップの入り。名前から：波紋・白黒。既定 2.0 秒 — いま：ISF Ripples をキー・ISF Color Monochrome をキー — 借りる先：見つからない

### ぼかし（12）

- [ ] **Focus**（ぼかし・有料）：クリップの入り。名前から：ピント。既定 1.2 秒 — いま：物の blur をキー — 借りる先：three.js shaders FocusShader（MIT、名前が同じ）／Animista text-focus-in（BSD-2）
- [ ] **Retro Fade-in**（ぼかし）：クリップの入り。名前から：ぼかし・じわっと出る・古いフィルム・レトロ（中国語名 模糊渐显）。既定 2.0 秒 — いま：物の blur をキー・animators fade_in・fade_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Retro Fade-in 2**（ぼかし）：クリップの入り。名前から：ぼかし・じわっと出る・古いフィルム・レトロ（中国語名 模糊渐显2）。既定 2.0 秒 — いま：物の blur をキー・animators fade_in・fade_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **旋转聚焦**（ぼかし・有料）：クリップの入り。名前から：ピント合わせ・回転。既定 2.0 秒 — いま：物の blur をキー・キー rotate — 借りる先：Animista text-focus-in（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **无限对焦**（ぼかし・有料）：クリップの入り。名前から：ピント合わせ。既定 1.0 秒 — いま：物の blur をキー — 借りる先：Animista text-focus-in（BSD-2）
- [ ] **模糊记忆**（ぼかし・有料）：クリップの入り。名前から：ぼかし。既定 2.0 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊闪入**（ぼかし・有料）：クリップの入り。名前から：ぼかし・光る。既定 1.0 秒 — いま：物の blur をキー・ふるまい random を opacity に — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）／Animista flicker-in-1（BSD-2）
- [ ] **横向模糊**（ぼかし・有料）：クリップの入り。名前から：ぼかし。既定 0.5 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **毛边模糊**（ぼかし・有料）：クリップの入り。名前から：ぼかし。既定 1.0 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **炫酷聚焦**（ぼかし・有料）：クリップの入り。名前から：ピント合わせ。既定 1.0 秒 — いま：物の blur をキー — 借りる先：Animista text-focus-in（BSD-2）
- [ ] **边角聚焦**（ぼかし・有料）：クリップの入り。名前から：ピント合わせ。既定 1.0 秒 — いま：物の blur をキー — 借りる先：Animista text-focus-in（BSD-2）
- [ ] **锯齿模糊**（ぼかし・有料）：クリップの入り。名前から：ぼかし。既定 2.0 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）

### グリッチ（11）

- [ ] **三重弹出**（グリッチ・有料）：クリップの入り。名前から：三重・弾んで出る。既定 1.2 秒 — いま：rgbshift.wgsl をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **修复故障**（グリッチ・有料）：クリップの入り。名前から：グリッチ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **撕裂开启**（グリッチ・有料）：クリップの入り。名前から：裂ける。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障开场**（グリッチ・有料）：クリップの入り。名前から：グリッチ・オープニング。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **故障抖动**（グリッチ・有料）：クリップの入り。名前から：グリッチ・揺れ。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **故障结尾**（グリッチ・有料）：クリップの入り。名前から：グリッチ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **数字空间**（グリッチ・有料）：クリップの入り。名前から：デジタル・空間。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **无信号**（グリッチ・有料）：クリップの入り。名前から：信号の乱れ。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **残酷故障**（グリッチ・有料）：クリップの入り。名前から：グリッチ。既定 1.5 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **激进故障**（グリッチ・有料）：クリップの入り。名前から：グリッチ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **高亮错误**（グリッチ・有料）：クリップの入り。名前から：エラー。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない

### レトロ（9）

- [ ] **CRT Bands**（レトロ）：クリップの入り。名前から：テレビ・光が走る・古いビデオ・帯に割れる（中国語名 电视屏扫光）。既定 2.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー／ISF Window Slice（MIT）をキー
- [ ] **TV On**（レトロ・有料）：クリップの入り。名前から：古いビデオ・テレビ。既定 1.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **厚重显示器**（レトロ・有料）：クリップの入り。名前から：モニター。既定 2.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **噪点拽入**（レトロ・有料）：クリップの入り。名前から：ノイズ。既定 1.0 秒 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise をキー — 借りる先：見つからない
- [ ] **噪点渐显**（レトロ・有料）：クリップの入り。名前から：ノイズ・じわっと出る。既定 1.9 秒 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise をキー・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **电影杂乱**（レトロ・有料）：クリップの入り。名前から：映画。既定 2.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない
- [ ] **老式显示器**（レトロ・有料）：クリップの入り。名前から：モニター。既定 2.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **老电视**（レトロ・有料）：クリップの入り。名前から：古いテレビ。既定 1.4 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **胶卷放映**（レトロ・有料）：クリップの入り。名前から：フィルム。既定 1.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない

### 色（4）

- [ ] **Gray Mask**（色）：クリップの入り。名前から：白黒（中国語名 灰色蒙层）。既定 1.5 秒 — いま：ISF Color Monochrome をキー — 借りる先：見つからない
- [ ] **变色闪震**（色・有料）：クリップの入り。名前から：色が変わる・光る・揺れ。既定 1.0 秒 — いま：ISF Multi Hue Shift をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **负片破碎**（色・有料）：クリップの入り。名前から：ネガ・割れる。既定 1.0 秒 — いま：ISF Color Invert・RGB Invert をキー — 借りる先：見つからない
- [ ] **黑白快刷**（色・有料）：クリップの入り。名前から：白黒・刷毛。既定 0.5 秒 — いま：ISF Color Monochrome をキー・ISF Wipe をキー — 借りる先：見つからない

### 枠（4）

- [ ] **3D手机**（枠・有料）：クリップの入り。名前から：スマホ・3D。既定 2.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **Anime Frame**（枠）：クリップの入り。名前から：枠・跳ねる・漫画・アニメ（中国語名 跳动边框）。既定 1.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー・animators bounce_in・spring_in — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT） をキー
- [ ] **漂浮拍立得**（枠・有料）：クリップの入り。名前から：インスタントカメラ・浮かぶ。既定 2.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **相片铺开**（枠・有料）：クリップの入り。名前から：写真。既定 1.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない

### 残像（2）

- [ ] **Out of Body**（残像・有料）：クリップの入り。名前から：魂が抜ける（拡大した写しを重ねる）・残像・分身。既定 1.5 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **破旧影片**（残像・有料）：クリップの入り。名前から：影。既定 2.0 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない

### 絵の具（2）

- [ ] **报纸拼贴Ⅰ**（絵の具・有料）：クリップの入り。名前から：新聞・コラージュ。既定 1.5 秒 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **报纸放大**（絵の具・有料）：クリップの入り。名前から：新聞・拡大。既定 1.7 秒 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）

### 文字（2）

- [ ] **时间倒计时**（文字・有料）：クリップの入り。名前から：カウントダウン。既定 2.0 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **黑白倒计时**（文字・有料）：クリップの入り。名前から：カウントダウン・白黒。既定 3.0 秒 — いま：animators print_on・print_off・ISF Color Monochrome をキー — 借りる先：見つからない

### 人（1）

- [ ] **宇宙之眼**（人・有料）：クリップの入り。名前から：目。既定 2.0 秒 — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion をキー

### その他（16）

- [ ] **Blinds**（その他）：クリップの入り。名前から：ブラインド（中国語名 百叶窗）。既定 0.5 秒 — いま：無い — 借りる先：ISF Window Blinds（MIT）をキー
- [ ] **TV开启**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **专辑封面**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **侧滑**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 0.6 秒 — いま：無い — 借りる先：見つからない
- [ ] **便利贴**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 0.9 秒 — いま：無い — 借りる先：見つからない
- [ ] **划水**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 0.8 秒 — いま：無い — 借りる先：見つからない
- [ ] **多层环形**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **快速缩远**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **数学公式**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **时间卷轴**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **点开**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 1.3 秒 — いま：無い — 借りる先：見つからない
- [ ] **爬入**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 1.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **破损美元**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **迷失方向**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **醒来**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **隐秘手电筒**（その他・有料）：クリップの入り。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない

## Animations › Out（クリップの出）（219）

Motolii に有る 178・近い物あり 20・見つからない 21。

### 動き（54）

- [ ] **Crisscross Shake**（動き・有料）：クリップの出。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Drop down**（動き・有料）：クリップの出。名前から：滑る・動く。既定 1.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fade Out**（動き）：クリップの出。名前から：じわっと消える・フェード（中国語名 渐隐）。既定 0.5 秒 — いま：animators fade_in・fade_out — 借りる先：Animista fade-out（BSD-2、名前が同じ）／Animate.css fadeOut（v3.7.2 は MIT、名前が同じ）
- [ ] **Mini Zoom**（動き）：クリップの出。名前から：拡大・縮小・寄る（中国語名 轻微放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Rotate**（動き）：クリップの出。名前から：回転（中国語名 旋转）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animate.css rotateOut（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Rotate Out 1**（動き）：クリップの出。名前から：回って出る・上へ・回転（中国語名 向上转出）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animate.css rotateOut（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Rotate Out 2**（動き）：クリップの出。名前から：回って出る・上へ・回転（中国語名 向上转出 II）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animate.css rotateOut（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Skew Shake**（動き・有料）：クリップの出。名前から：揺れ・伸ばす・傾く。既定 1.0 秒 — いま：ふるまい noise・キー scale_x・skew — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **Slide Down**（動き）：クリップの出。名前から：滑る・下へ・動く（中国語名 向下滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideOutDown（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Left**（動き）：クリップの出。名前から：滑る・左へ・動く（中国語名 向左滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-out-left（BSD-2、名前が同じ）／Animate.css slideOutLeft（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Right**（動き）：クリップの出。名前から：滑る・右へ・動く（中国語名 向右滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-out-right（BSD-2、名前が同じ）／Animate.css slideOutRight（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Up**（動き）：クリップの出。名前から：滑る・上へ・動く（中国語名 向上滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideOutUp（v3.7.2 は MIT、名前が同じ）
- [ ] **X震动**（動き・有料）：クリップの出。名前から：振動。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Zoom In**（動き）：クリップの出。名前から：拡大・縮小・寄る（中国語名 放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom Out**（動き）：クリップの出。名前から：縮小・拡大・寄る（中国語名 缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomOut（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **传单飞走**（動き・有料）：クリップの出。名前から：飛ぶ。既定 1.6 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **余烬渐隐**（動き・有料）：クリップの出。名前から：じわっと消える。既定 2.0 秒 — いま：animators fade_in・fade_out — 借りる先：見つからない
- [ ] **光芒渐逝**（動き・有料）：クリップの出。名前から：じわっと・光。既定 2.0 秒 — いま：animators fade_in・fade_out・物の glow をキー — 借りる先：見つからない
- [ ] **冲撞**（動き・有料）：クリップの出。名前から：ぶつかる。既定 0.8 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **加速放大**（動き・有料）：クリップの出。名前から：拡大。既定 1.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **区域色块滑动**（動き・有料）：クリップの出。名前から：滑る。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **卡阵悬浮**（動き・有料）：クリップの出。名前から：カード。既定 1.0 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **变速时空**（動き・有料）：クリップの出。名前から：速さが変わる。既定 1.2 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **向上滚动**（動き・有料）：クリップの出。名前から：スクロール・上へ。既定 1.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **向上闪出**（動き・有料）：クリップの出。名前から：上へ・光る。既定 0.7 秒 — いま：animators slide_in・slide_out・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **嗡嗡远离**（動き・有料）：クリップの出。名前から：ぶーん（揺れ）。既定 1.4 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **多维空间**（動き・有料）：クリップの出。名前から：空間。既定 1.4 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **层层飞远**（動き・有料）：クリップの出。名前から：飛ぶ。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **强震抖动**（動き・有料）：クリップの出。名前から：揺れ・強い揺れ。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抖动变焦**（動き・有料）：クリップの出。名前から：ズーム・揺れ。既定 0.5 秒 — いま：キー scale（区間のイーズ）・ふるまい noise — 借りる先：Animista scale-in-center（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抖动横移**（動き・有料）：クリップの出。名前から：揺れ・横に動く。既定 0.4 秒 — いま：ふるまい noise・animators slide_in・slide_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抽卡**（動き・有料）：クリップの出。名前から：カードを引く。既定 1.4 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **拉丝滑出**（動き・有料）：クリップの出。名前から：滑って出る・引く。既定 0.5 秒 — いま：animators slide_in・slide_out・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **拉扯移出**（動き・有料）：クリップの出。名前から：引っぱる。既定 2.0 秒 — いま：キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **收缩滑动**（動き・有料）：クリップの出。名前から：滑る。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **方形镜头**（動き・有料）：クリップの出。名前から：カメラ。既定 2.0 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **旋转出**（動き・有料）：クリップの出。名前から：回転。既定 2.0 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **旋转圆球**（動き・有料）：クリップの出。名前から：回転・丸。既定 0.8 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／gl-transitions circleopen（MIT）をキー
- [ ] **旋转熄屏**（動き・有料）：クリップの出。名前から：回転。既定 1.7 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **晃落**（動き・有料）：クリップの出。名前から：落ちる・揺れ。既定 1.0 秒 — いま：animators slide_in・slide_out・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **滑动选中**（動き・有料）：クリップの出。名前から：滑る。既定 3.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **滑片滑动**（動き・有料）：クリップの出。名前から：滑る。既定 1.2 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **炫目淡出**（動き・有料）：クリップの出。名前から：フェードアウト。既定 2.0 秒 — いま：animators fade_in・fade_out — 借りる先：見つからない
- [ ] **猛击而出**（動き・有料）：クリップの出。名前から：打つ。既定 2.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **盘形抽卡**（動き・有料）：クリップの出。名前から：カードを引く。既定 2.0 秒 — いま：rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **矩阵淡出**（動き・有料）：クリップの出。名前から：フェードアウト。既定 1.0 秒 — いま：animators fade_in・fade_out — 借りる先：見つからない
- [ ] **神奇弹窗**（動き・有料）：クリップの出。名前から：弾む・窓。既定 0.6 秒 — いま：animators bounce_in・spring_in・形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない
- [ ] **维度跃迁 2**（動き・有料）：クリップの出。名前から：跳ぶ。既定 2.0 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **缩小卡片**（動き・有料）：クリップの出。名前から：縮小・カード。既定 1.9 秒 — いま：キー scale（区間のイーズ）・rotate_x・rotate_y・z — 借りる先：Animista scale-in-center（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **翻卡**（動き・有料）：クリップの出。名前から：めくる・カード。既定 1.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **脉冲**（動き・有料）：クリップの出。名前から：脈打つ。既定 0.8 秒 — いま：ふるまい oscillator を scale に — 借りる先：Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **顺滑下落**（動き・有料）：クリップの出。名前から：落ちる。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **飘散**（動き・有料）：クリップの出。名前から：漂う。既定 2.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **飞远**（動き・有料）：クリップの出。名前から：飛ぶ。既定 1.6 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない

### 分割（35）

- [ ] **3D翻页**（分割・有料）：クリップの出。名前から：ページめくり・3D。既定 2.0 秒 — いま：ISF Inverted Page Curl をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **Cubic Flip**（分割・有料）：クリップの出。名前から：立方体・裏返る。既定 1.1 秒 — いま：rotate_x・rotate_y・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in（BSD-2）／Animista flip-in-hor-bottom（BSD-2）
- [ ] **Grid Puzzle**（分割・有料）：クリップの出。名前から：パズル・ブロック。既定 2.0 秒 — いま：ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **Oblique Lines**（分割・有料）：クリップの出。名前から：帯に割れる・伸ばす・傾く。既定 0.5 秒 — いま：キー scale_x・skew — 借りる先：ISF Window Slice（MIT）をキー／Animista tracking-in-expand（BSD-2）
- [ ] **Rotation Closing**（分割）：クリップの出。名前から：幕が閉じる・回転（中国語名 旋转闭幕）。既定 1.0 秒 — いま：キー rotate — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Screen Wipe**（分割・有料）：クリップの出。名前から：拭き取る・分割画面（中国語名 画面擦除）。既定 2.0 秒 — いま：ISF Wipe をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **Tesseract**（分割・有料）：クリップの出。名前から：立方体。既定 1.1 秒 — いま：rotate_x・rotate_y — 借りる先：Animista flip-in（BSD-2）
- [ ] **Vibrating Panels**（分割）：クリップの出。名前から：分割画面・振動・揺れ（中国語名 分屏震动）。既定 2.0 秒 — いま：ISF Quad Tile をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **三切闪闭幕**（分割・有料）：クリップの出。名前から：幕が閉じる・光る。既定 0.8 秒 — いま：ふるまい random を opacity に — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista flicker-in-1（BSD-2）
- [ ] **三屏切闪**（分割・有料）：クリップの出。名前から：3 分割画面・光る。既定 1.0 秒 — いま：ISF Quad Tile をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **分屏横移**（分割・有料）：クリップの出。名前から：分割画面・横に動く。既定 0.9 秒 — いま：ISF Quad Tile をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **分屏翻转**（分割・有料）：クリップの出。名前から：分割画面・裏返る。既定 0.6 秒 — いま：ISF Quad Tile をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **动感拼接**（分割・有料）：クリップの出。名前から：つなぎ合わせ。既定 2.0 秒 — いま：ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **微震闭幕**（分割・有料）：クリップの出。名前から：幕が閉じる・揺れ。既定 0.9 秒 — いま：ふるまい noise — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **快速翻页**（分割・有料）：クリップの出。名前から：ページめくり。既定 0.2 秒 — いま：ISF Inverted Page Curl をキー — 借りる先：見つからない
- [ ] **拼图**（分割・有料）：クリップの出。名前から：パズル。既定 1.1 秒 — いま：ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **方块滑出**（分割・有料）：クリップの出。名前から：四角いブロック・滑って出る。既定 1.5 秒 — いま：ISF Random Squares Mask をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **涂抹**（分割・有料）：クリップの出。名前から：塗る。既定 1.0 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **立方合体**（分割・有料）：クリップの出。名前から：立方体。既定 1.3 秒 — いま：rotate_x・rotate_y — 借りる先：Animista flip-in（BSD-2）
- [ ] **竖向拼接**（分割・有料）：クリップの出。名前から：つなぎ合わせ。既定 2.5 秒 — いま：ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **网格图纸**（分割・有料）：クリップの出。名前から：格子・紙。既定 2.0 秒 — いま：ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **转圈圈**（分割・有料）：クリップの出。名前から：輪・回る。既定 0.8 秒 — いま：キー rotate — 借りる先：gl-transitions circleopen（MIT）をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Cross Close**（分割・有料）：クリップの出。名前から：帯に割れる・幕・扉が開く。既定 1.1 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー／gl-transitions HorizontalOpen（MIT）をキー
- [ ] **Folding Fan**（分割・有料）：クリップの出。名前から：折る。既定 1.1 秒 — いま：無い — 借りる先：Animista flip-in（BSD-2）
- [ ] **关闭电源**（分割・有料）：クリップの出。名前から：閉じる。既定 2.0 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **分割**（分割・有料）：クリップの出。名前から：分割。既定 0.9 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **幕布关闭**（分割・有料）：クリップの出。名前から：閉じる。既定 2.0 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **思维曲折**（分割・有料）：クリップの出。名前から：折る。既定 2.0 秒 — いま：無い — 借りる先：Animista flip-in（BSD-2）
- [ ] **折叠**（分割・有料）：クリップの出。名前から：折りたたむ。既定 0.3 秒 — いま：無い — 借りる先：Animista flip-in（BSD-2）
- [ ] **条形折叠**（分割・有料）：クリップの出。名前から：帯・折りたたむ。既定 1.0 秒 — いま：無い — 借りる先：ISF Window Slice（MIT）をキー／Animista flip-in（BSD-2）
- [ ] **黑边展开**（分割・有料）：クリップの出。名前から：広がる。既定 2.0 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **Shattered**（分割・有料）：クリップの出。名前から：割れる。既定 0.7 秒 — いま：無い — 借りる先：見つからない
- [ ] **皱缩**（分割・有料）：クリップの出。名前から：しわ。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **破碎**（分割・有料）：クリップの出。名前から：割れる。既定 1.8 秒 — いま：無い — 借りる先：見つからない
- [ ] **碎纸出场**（分割・有料）：クリップの出。名前から：紙。既定 1.5 秒 — いま：無い — 借りる先：見つからない

### 光（24）

- [ ] **Homing Light**（光・有料）：クリップの出。名前から：光が戻る。既定 2.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **Lumin Flash**（光・有料）：クリップの出。名前から：光る・光る（フラッシュ）。既定 0.8 秒 — いま：物の glow をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **Lumin Matrix**（光・有料）：クリップの出。名前から：光る。既定 1.1 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **RGB Scanlines**（光）：クリップの出。名前から：光・引く・色ずれ・古いビデオ（中国語名 光效拉丝）。既定 1.0 秒 — いま：物の glow をキー・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）／ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **分割闪烁**（光・有料）：クリップの出。名前から：点滅・分割。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）／ISF Window Slice（MIT）をキー
- [ ] **分割闪烁 2**（光・有料）：クリップの出。名前から：点滅・分割。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）／ISF Window Slice（MIT）をキー
- [ ] **卡片扫光**（光・有料）：クリップの出。名前から：光が走る・カード。既定 1.8 秒 — いま：rotate_x・rotate_y・z — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー／Animista tilt-in（BSD-2）
- [ ] **旋光登场**（光・有料）：クリップの出。名前から：光。既定 2.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **曝光放射**（光・有料）：クリップの出。名前から：放射・露出。既定 0.5 秒 — いま：ISF God Rays をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **模糊快门**（光・有料）：クリップの出。名前から：シャッター・ぼかし。既定 2.0 秒 — いま：段の exposure をキー・物の blur をキー — 借りる先：Animista flicker-in-glow（BSD-2）／Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **眩光弹动**（光・有料）：クリップの出。名前から：まぶしい光・弾む。既定 1.3 秒 — いま：段の lens_flare・ISF Lens Flare をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **负片闪屏**（光・有料）：クリップの出。名前から：画面が光る・ネガ。既定 1.5 秒 — いま：段の exposure をキー・ISF Color Invert・RGB Invert をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **赛博光线**（光・有料）：クリップの出。名前から：サイバー・光の筋。既定 0.6 秒 — いま：ISF God Rays をキー — 借りる先：ISF Neon（MIT） をキー
- [ ] **迷幻流光**（光・有料）：クリップの出。名前から：流れる光・サイケ。既定 1.5 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **透明光晕**（光・有料）：クリップの出。名前から：光のにじみ。既定 2.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **闪光出场**（光・有料）：クリップの出。名前から：光る。既定 1.0 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪动卡片**（光・有料）：クリップの出。名前から：点滅・カード。既定 1.0 秒 — いま：ふるまい random を opacity に・rotate_x・rotate_y・z — 借りる先：Animista flicker-in-1（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **闪屏**（光・有料）：クリップの出。名前から：画面が光る。既定 0.8 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **闪烁关闭**（光・有料）：クリップの出。名前から：点滅・閉じる。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）／gl-transitions HorizontalOpen（MIT）をキー
- [ ] **闪现**（光・有料）：クリップの出。名前から：ちらっと出る。既定 0.2 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **闪电分屏**（光）：クリップの出。名前から：稲妻・分割画面。既定 2.0 秒 — いま：ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **闪白缓出**（光・有料）：クリップの出。名前から：白く光る。既定 1.2 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **快速光斑**（光・有料）：クリップの出。名前から：光の玉。既定 1.0 秒 — いま：無い — 借りる先：Kenney Particle Pack の光の玉（CC0）＋物の copies・scatter をキー
- [ ] **扫描**（光・有料）：クリップの出。名前から：スキャン。既定 0.6 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー

### 粒・飾り（22）

- [ ] **Cross Meld**（粒・飾り・有料）：クリップの出。名前から：溶ける。既定 0.7 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Flame Risen**（粒・飾り・有料）：クリップの出。名前から：焼ける・炎・滑る・動く（中国語名 灼烧出现）。既定 3.0 秒 — いま：点群の Burn Away をキー・animators slide_in・slide_out — 借りる先：ISF Film Burn・Undulating Burn Out（MIT） をキー／Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Kira游动**（粒・飾り・有料）：クリップの出。名前から：きらきら。既定 2.3 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **水纹消失**（粒・飾り・有料）：クリップの出。名前から：消える。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **流金**（粒・飾り・有料）：クリップの出。名前から：金色。既定 1.1 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **炸开下坠**（粒・飾り・有料）：クリップの出。名前から：はじける・落ちる。既定 1.5 秒 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **烟雾弹**（粒・飾り・有料）：クリップの出。名前から：煙・弾む。既定 0.9 秒 — いま：段の fog をキー・animators bounce_in・spring_in — 借りる先：Kenney Particle Pack の煙（CC0） をキー
- [ ] **爱心碰撞**（粒・飾り・有料）：クリップの出。名前から：ハート・ぶつかる。既定 2.3 秒 — いま：ふるまい noise — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **疾闪消失**（粒・飾り・有料）：クリップの出。名前から：消える・光る。既定 1.0 秒 — いま：点群 dissolve（点群だけ）・ふるまい random を opacity に — 借りる先：Animista puff-in-center（BSD-2）／Animista flicker-in-1（BSD-2）
- [ ] **粒子爱心**（粒・飾り・有料）：クリップの出。名前から：粒子・ハート。既定 3.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）／ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **细闪渐隐**（粒・飾り・有料）：クリップの出。名前から：細かいきらめき・じわっと消える。既定 1.8 秒 — いま：ISF Star（1 個） をキー・animators fade_in・fade_out — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **结晶消失**（粒・飾り・有料）：クリップの出。名前から：消える。既定 2.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **金沙**（粒・飾り・有料）：クリップの出。名前から：金の砂。既定 2.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **钻石传送门 2**（粒・飾り・有料）：クリップの出。名前から：ダイヤ・転送・扉。既定 1.0 秒 — いま：ISF Star（1 個） をキー・ふるまい random を opacity に — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー／Animista flicker-in-1（BSD-2）
- [ ] **魔法粒子I**（粒・飾り・有料）：クリップの出。名前から：粒子・魔法。既定 3.0 秒 — いま：点群 dissolve（点群だけ）・ISF Star（1 個） をキー — 借りる先：Animista puff-in-center（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **魔法粒子II**（粒・飾り・有料）：クリップの出。名前から：粒子・魔法。既定 3.0 秒 — いま：点群 dissolve（点群だけ）・ISF Star（1 個） をキー — 借りる先：Animista puff-in-center（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **2025**（粒・飾り・有料）：クリップの出。名前から：季節の行事。既定 1.0 秒 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で をキー
- [ ] **Cash Out**（粒・飾り・有料）：クリップの出。名前から：お札。既定 1.7 秒 — いま：無い — 借りる先：Twemoji（CC BY 4.0）の絵を物の copies で をキー
- [ ] **枫叶遮罩**（粒・飾り・有料）：クリップの出。名前から：葉。既定 2.3 秒 — いま：無い — 借りる先：Twemoji の花・葉（CC BY 4.0）＋物の copies・scatter をキー
- [ ] **爱心信封**（粒・飾り・有料）：クリップの出。名前から：ハート。既定 1.1 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **爱情手账本**（粒・飾り・有料）：クリップの出。名前から：愛。既定 2.0 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **画出爱心**（粒・飾り・有料）：クリップの出。名前から：ハート。既定 1.1 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー

### レンズ・ゆがみ（21）

- [ ] **90s鱼眼**（レンズ・ゆがみ・有料）：クリップの出。名前から：魚眼。既定 2.0 秒 — いま：ISF Bump Distortion をキー — 借りる先：見つからない
- [ ] **Chroma Wave**（レンズ・ゆがみ・有料）：クリップの出。名前から：波打つ・色ずれ。既定 0.8 秒 — いま：animators（文字ごとにずらす）・rgbshift.wgsl をキー — 借りる先：anime.js stagger（MIT）
- [ ] **Flip**（レンズ・ゆがみ）：クリップの出。名前から：鏡像・裏返る（中国語名 镜像翻转）。既定 0.5 秒 — いま：mirror.wgsl をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Waterdrop**（レンズ・ゆがみ・有料）：クリップの出。名前から：波紋。既定 1.2 秒 — いま：ISF Ripples をキー — 借りる先：ISF-Files Water Drop（MIT、名前が同じ）／gl-transitions WaterDrop（MIT、名前が同じ）
- [ ] **Whirl**（レンズ・ゆがみ）：クリップの出。名前から：渦・回転・ねじれ（中国語名 漩涡旋转）。既定 0.5 秒 — いま：ISF Twirl をキー・キー rotate — 借りる先：Animista swirl-in-fwd（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **像素潮汐**（レンズ・ゆがみ・有料）：クリップの出。名前から：ピクセル。既定 2.0 秒 — いま：ISF Pixellate をキー — 借りる先：見つからない
- [ ] **像素爆炸**（レンズ・ゆがみ・有料）：クリップの出。名前から：ピクセル・爆発。既定 2.0 秒 — いま：ISF Pixellate をキー・レシピ burst.js・confetti.js・点群 Appear Eruption をキー — 借りる先：見つからない
- [ ] **冰块**（レンズ・ゆがみ・有料）：クリップの出。名前から：氷。既定 1.3 秒 — いま：ISF Frosted Glass をキー — 借りる先：見つからない
- [ ] **水滴遮罩**（レンズ・ゆがみ・有料）：クリップの出。名前から：しずく。既定 2.0 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **波浪慢扭**（レンズ・ゆがみ・有料）：クリップの出。名前から：波打つ。既定 2.0 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **波纹油画**（レンズ・ゆがみ・有料）：クリップの出。名前から：波紋・油絵。既定 0.8 秒 — いま：ISF Ripples をキー — 借りる先：GPUImage3 KuwaharaFilter（BSD-3） をキー
- [ ] **玻璃切入**（レンズ・ゆがみ・有料）：クリップの出。名前から：ガラス。既定 0.8 秒 — いま：ISF Frosted Glass をキー — 借りる先：見つからない
- [ ] **砸出波纹**（レンズ・ゆがみ・有料）：クリップの出。名前から：波紋。既定 1.4 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **空间扭曲**（レンズ・ゆがみ・有料）：クリップの出。名前から：ゆがみ・空間。既定 0.9 秒 — いま：ISF Bump Distortion をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）
- [ ] **虫洞出口**（レンズ・ゆがみ・有料）：クリップの出。名前から：ワームホール。既定 1.0 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **透镜晃影**（レンズ・ゆがみ・有料）：クリップの出。名前から：レンズ・影・揺れ。既定 1.0 秒 — いま：ISF Bump Distortion をキー・ISF Echo Trace をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **镜面拆分**（レンズ・ゆがみ・有料）：クリップの出。名前から：鏡面。既定 1.5 秒 — いま：mirror.wgsl をキー — 借りる先：見つからない
- [ ] **震动波纹**（レンズ・ゆがみ・有料）：クリップの出。名前から：波紋・振動。既定 1.5 秒 — いま：ISF Ripples をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **震波 III**（レンズ・ゆがみ・有料）：クリップの出。名前から：衝撃波。既定 0.7 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **马赛克**（レンズ・ゆがみ・有料）：クリップの出。名前から：モザイク。既定 1.0 秒 — いま：ISF Pixellate をキー — 借りる先：見つからない
- [ ] **黑洞**（レンズ・ゆがみ・有料）：クリップの出。名前から：ブラックホール。既定 0.7 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）

### レトロ（13）

- [ ] **CRT Bands**（レトロ）：クリップの出。名前から：テレビ・光が走る・古いビデオ・帯に割れる（中国語名 电视屏扫光）。既定 2.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー／ISF Window Slice（MIT）をキー
- [ ] **TV Off**（レトロ・有料）：クリップの出。名前から：古いビデオ・テレビ。既定 1.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **噪点拽出**（レトロ・有料）：クリップの出。名前から：ノイズ。既定 1.0 秒 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise をキー — 借りる先：見つからない
- [ ] **噪点渐隐**（レトロ・有料）：クリップの出。名前から：ノイズ・じわっと消える。既定 1.8 秒 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise をキー・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **录像带分屏**（レトロ・有料）：クリップの出。名前から：ビデオテープ・分割画面。既定 2.0 秒 — いま：crt_finish.wgsl をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **杂噪裂出**（レトロ・有料）：クリップの出。名前から：ノイズ。既定 1.0 秒 — いま：grain.wgsl・fine_grain.wgsl・ISF Noise をキー — 借りる先：見つからない
- [ ] **电视光束 2**（レトロ・有料）：クリップの出。名前から：テレビ・光の束。既定 2.0 秒 — いま：crt_finish.wgsl をキー・ISF God Rays をキー — 借りる先：見つからない
- [ ] **电视关闭**（レトロ・有料）：クリップの出。名前から：テレビ・閉じる。既定 2.0 秒 — いま：crt_finish.wgsl をキー — 借りる先：gl-transitions HorizontalOpen（MIT）をキー
- [ ] **电视熄屏**（レトロ・有料）：クリップの出。名前から：テレビ。既定 0.5 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **老电视**（レトロ・有料）：クリップの出。名前から：古いテレビ。既定 1.6 秒 — いま：crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **胶卷放映**（レトロ・有料）：クリップの出。名前から：フィルム。既定 1.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない
- [ ] **褪色出口**（レトロ・有料）：クリップの出。名前から：色あせ。既定 3.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない
- [ ] **褪色变色**（レトロ・有料）：クリップの出。名前から：色あせ・色が変わる。既定 3.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない

### グリッチ（11）

- [ ] **Frame Shudder**（グリッチ・有料）：クリップの出。名前から：コマ止め・コマ落とし・枠・揺れ。既定 3.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー・ふるまい noise — 借りる先：ISF Freeze Frame・Random Freeze（MIT） をキー／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Glitch撕裂**（グリッチ・有料）：クリップの出。名前から：裂ける。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障抖动**（グリッチ・有料）：クリップの出。名前から：グリッチ・揺れ。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **故障消失**（グリッチ・有料）：クリップの出。名前から：グリッチ・消える。既定 1.2 秒 — いま：ISF Bad TV・rgbshift をキー・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **故障溶解**（グリッチ・有料）：クリップの出。名前から：グリッチ・溶ける。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **故障结尾**（グリッチ・有料）：クリップの出。名前から：グリッチ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障闪黑**（グリッチ・有料）：クリップの出。名前から：グリッチ・黒く光る。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー・段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **无信号 2**（グリッチ・有料）：クリップの出。名前から：信号の乱れ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **特写错误**（グリッチ・有料）：クリップの出。名前から：エラー・アップ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **电视故障**（グリッチ・有料）：クリップの出。名前から：グリッチ・テレビ。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・crt_finish.wgsl をキー — 借りる先：見つからない
- [ ] **粉色故障**（グリッチ・有料）：クリップの出。名前から：グリッチ・ピンク。既定 1.5 秒 — いま：ISF Bad TV・rgbshift をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない

### ぼかし（7）

- [ ] **Blurred Fade-in**（ぼかし）：クリップの出。名前から：ぼかし・じわっと消える・フェード（中国語名 模糊渐隐2）。既定 2.0 秒 — いま：物の blur をキー・animators fade_in・fade_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blurred Fade-in**（ぼかし）：クリップの出。名前から：ぼかし・じわっと消える・フェード（中国語名 模糊渐隐）。既定 2.0 秒 — いま：物の blur をキー・animators fade_in・fade_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Focus**（ぼかし・有料）：クリップの出。名前から：ピント。既定 0.8 秒 — いま：物の blur をキー — 借りる先：three.js shaders FocusShader（MIT、名前が同じ）／Animista text-focus-in（BSD-2）
- [ ] **方块模糊**（ぼかし・有料）：クリップの出。名前から：ぼかし・四角いブロック。既定 1.5 秒 — いま：物の blur をキー・ISF Random Squares Mask をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊闪出**（ぼかし・有料）：クリップの出。名前から：ぼかし・光る。既定 1.0 秒 — いま：物の blur をキー・ふるまい random を opacity に — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）／Animista flicker-in-1（BSD-2）
- [ ] **横向模糊**（ぼかし・有料）：クリップの出。名前から：ぼかし。既定 0.5 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **毛边模糊**（ぼかし・有料）：クリップの出。名前から：ぼかし。既定 1.0 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）

### 色（6）

- [ ] **Gray Mask**（色）：クリップの出。名前から：白黒（中国語名 灰色蒙层）。既定 1.5 秒 — いま：ISF Color Monochrome をキー — 借りる先：見つからない
- [ ] **变色闪震**（色・有料）：クリップの出。名前から：色が変わる・光る・揺れ。既定 1.0 秒 — いま：ISF Multi Hue Shift をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **明亮重置**（色・有料）：クリップの出。名前から：明るい。既定 1.2 秒 — いま：段の exposure・ISF Exposure Adjust をキー — 借りる先：見つからない
- [ ] **蓝色砖块**（色・有料）：クリップの出。名前から：青。既定 2.0 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **恐怖景观出口**（色・有料）：クリップの出。名前から：ホラー。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **血色晕染**（色・有料）：クリップの出。名前から：血。既定 1.0 秒 — いま：無い — 借りる先：見つからない

### 絵の具（6）

- [ ] **报纸拼贴Ⅰ**（絵の具・有料）：クリップの出。名前から：新聞・コラージュ。既定 1.5 秒 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **报纸拼贴Ⅱ**（絵の具・有料）：クリップの出。名前から：新聞・コラージュ。既定 1.5 秒 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone をキー・ISF Quad Tile をキー — 借りる先：見つからない
- [ ] **报纸缩小**（絵の具・有料）：クリップの出。名前から：新聞・縮小。既定 1.9 秒 — いま：halftone.wgsl・dot_screen.wgsl・ISF CMYK Halftone をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **海报飞出**（絵の具・有料）：クリップの出。名前から：ポスター・飛んで出る。既定 2.0 秒 — いま：ISF Posterize をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **交叉涂鸦**（絵の具・有料）：クリップの出。名前から：落書き。既定 2.0 秒 — いま：無い — 借りる先：ISF Doodler（MIT） をキー
- [ ] **涂鸦划除**（絵の具・有料）：クリップの出。名前から：落書き。既定 1.0 秒 — いま：無い — 借りる先：ISF Doodler（MIT） をキー

### 枠（2）

- [ ] **Anime Frame**（枠）：クリップの出。名前から：枠・跳ねる・漫画・アニメ（中国語名 跳动边框）。既定 1.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー・animators bounce_in・spring_in — 借りる先：ISF Toon（MIT）・three.js ToonShader（MIT） をキー
- [ ] **相片铺开**（枠・有料）：クリップの出。名前から：写真。既定 1.0 秒 — いま：形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない

### 残像（2）

- [ ] **Out of Body**（残像・有料）：クリップの出。名前から：魂が抜ける（拡大した写しを重ねる）・残像・分身。既定 1.5 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **克隆与传送**（残像・有料）：クリップの出。名前から：クローン・転送。既定 1.7 秒 — いま：ISF Echo Trace をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）

### 人（1）

- [ ] **眨眼离场**（人・有料）：クリップの出。名前から：目。既定 2.0 秒 — いま：無い — 借りる先：MediaPipe Face Landmarker（Apache-2.0）＋ISF Bump Distortion をキー

### その他（15）

- [ ] **侧滑**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 0.4 秒 — いま：無い — 借りる先：見つからない
- [ ] **便利贴**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 0.8 秒 — いま：無い — 借りる先：見つからない
- [ ] **关机选项**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **划水**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 0.8 秒 — いま：無い — 借りる先：見つからない
- [ ] **动感分区**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **合并面板**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 0.7 秒 — いま：無い — 借りる先：見つからない
- [ ] **四切退出**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 1.1 秒 — いま：無い — 借りる先：見つからない
- [ ] **图阵还原**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **多层环形**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 1.6 秒 — いま：無い — 借りる先：見つからない
- [ ] **幕布**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **抬起**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **补丁美元**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 1.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **轻巧扭出**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **迅速退出**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **魔术呈现**（その他・有料）：クリップの出。名前だけでは中身が分からない。既定 2.0 秒 — いま：無い — 借りる先：見つからない

## Animations › Combo（入りから出まで）（117）

Motolii に有る 116・近い物あり 1・見つからない 0。

### 動き（68）

- [ ] **3D card 1**（動き・有料）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **3D card 2**（動き・有料）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐 II）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **3D card 3**（動き）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐 III）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **3D card 4**（動き・有料）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐 IV）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **3D card 5**（動き・有料）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐 V）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **3D card 6**（動き・有料）：クリップの入りから出まで。名前から：裏返る・立体（中国語名 叠叠乐 Ⅵ）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・rotate_x・rotate_y・z — 借りる先：Animista flip-in-hor-bottom（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **Angle 1**（動き）：クリップの入りから出まで。名前から：回る・斜め（中国語名 斜转）。既定 0.5 秒 — いま：キー rotate・キー scale_x・skew — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **Angle 2**（動き・有料）：クリップの入りから出まで。名前から：回る・斜め（中国語名 斜转 II）。既定 0.5 秒 — いま：キー rotate・キー scale_x・skew — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **Bounce 1**（動き）：クリップの入りから出まで。名前から：跳ね返る・弾む（中国語名 回弹伸缩）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：Animate.css bounce（v3.7.2 は MIT、名前が同じ）／ISF-Files Bounce（MIT、名前が同じ）／gl-transitions Bounce（MIT、名前が同じ）
- [ ] **Bounce 2**（動き）：クリップの入りから出まで。名前から：回転・弾む（中国語名 旋转伸缩）。既定 0.5 秒 — いま：キー rotate・animators bounce_in・spring_in — 借りる先：Animate.css bounce（v3.7.2 は MIT、名前が同じ）／ISF-Files Bounce（MIT、名前が同じ）／gl-transitions Bounce（MIT、名前が同じ）
- [ ] **Enlarge and bounce**（動き）：クリップの入りから出まで。名前から：弾む・拡大・縮小・寄る（中国語名 放大弹动）。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Fall Bottom Left**（動き）：クリップの入りから出まで。名前から：左へ・下がる・滑る・動く（中国語名 向左下降）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fall Bottom Right**（動き）：クリップの入りから出まで。名前から：右へ・下がる・滑る・動く（中国語名 向右下降）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fall Left**（動き）：クリップの入りから出まで。名前から：左へ・下がる・滑る・動く（中国語名 下降向左）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fall Right**（動き）：クリップの入りから出まで。名前から：右へ・下がる・滑る・動く（中国語名 下降向右）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fall Spin**（動き）：クリップの入りから出まで。名前から：回転・落ちる・滑る・動く（中国語名 降落旋转）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Flip 2**（動き）：クリップの入りから出まで。名前から：裏返る（中国語名 翻转 II）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip 3**（動き）：クリップの入りから出まで。名前から：裏返る。既定 5.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip 4**（動き）：クリップの入りから出まで。名前から：裏返る。既定 5.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip 5**（動き）：クリップの入りから出まで。名前から：裏返る。既定 5.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip 6**（動き）：クリップの入りから出まで。名前から：裏返る。既定 5.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip II**（動き）：クリップの入りから出まで。名前から：裏返る（中国語名 翻转）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Left Zoom**（動き）：クリップの入りから出まで。名前から：縮小・左へ・拡大・寄る（中国語名 向左缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・animators slide_in・slide_out — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Pan Left**（動き）：クリップの入りから出まで。名前から：ズーム・滑る・動く（中国語名 左拉镜）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・animators slide_in・slide_out — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Pan Right**（動き）：クリップの入りから出まで。名前から：ズーム・滑る・動く（中国語名 右拉镜）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・animators slide_in・slide_out — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Pendulum 1**（動き）：クリップの入りから出まで。名前から：ブランコ・揺れ動く（振り子）（中国語名 荡秋千）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Pendulum 2**（動き）：クリップの入りから出まで。名前から：ブランコ・揺れ動く（振り子）（中国語名 荡秋千 II）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Pirate ship 1**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 海盗船）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Pirate ship 2**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 海盗船 II）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Pirate ship 3**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 海盗船 III）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Pirate ship 4**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 海盗船 IV）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Puzzle**（動き・有料）：クリップの入りから出まで。名前から：滑る・パズル・ブロック（中国語名 碎块滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out・ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **Right Zoom**（動き）：クリップの入りから出まで。名前から：縮小・右へ・拡大・寄る（中国語名 向右缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・animators slide_in・slide_out — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Rise Spin**（動き）：クリップの入りから出まで。名前から：回転・上がる・滑る・動く（中国語名 上升旋转）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Roll In&Out 1**（動き）：クリップの入りから出まで。名前から：回って入る・回って出る・回転（中国語名 转入转出）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Roll In&Out 2**（動き）：クリップの入りから出まで。名前から：回って入る・回って出る・回転（中国語名 转入转出 II）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Roller-coaster 1**（動き）：クリップの入りから出まで。名前から：回転・滑る・動く（中国語名 过山车）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Roller-coaster 2**（動き）：クリップの入りから出まで。名前から：回転・滑る・動く（中国語名 过山车 II）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Shrink and bounce**（動き）：クリップの入りから出まで。名前から：弾む・縮小・拡大・寄る（中国語名 缩小弹动）。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Slice & Rotate 1**（動き・有料）：クリップの入りから出まで。名前から：回る・帯に割れる・回転（中国語名 方片转动）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／ISF Window Slice（MIT）をキー
- [ ] **Slice & Rotate 2**（動き・有料）：クリップの入りから出まで。名前から：回る・帯に割れる・回転（中国語名 方片转动 II）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／ISF Window Slice（MIT）をキー
- [ ] **Slip and slide 1**（動き）：クリップの入りから出まで。名前から：滑る・動く（中国語名 滑滑梯）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Slip and slide 2**（動き）：クリップの入りから出まで。名前から：滑る・動く（中国語名 滑滑梯 II）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Smaller**（動き）：クリップの入りから出まで。名前から：回転・縮小・拡大・寄る（中国語名 旋转缩小）。既定 0.5 秒 — いま：キー rotate・キー scale（区間のイーズ） — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **Spin Fall**（動き）：クリップの入りから出まで。名前から：回転・落ちる・滑る・動く（中国語名 旋转降落）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin In**（動き）：クリップの入りから出まで。名前から：弾んで入る・回転（中国語名 弹入旋转）。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin Out**（動き）：クリップの入りから出まで。名前から：回転（中国語名 旋转回吸）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin Rise**（動き）：クリップの入りから出まで。名前から：回転・上がる・滑る・動く（中国語名 旋转上升）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spinning top 1**（動き）：クリップの入りから出まで。名前から：回転（中国語名 小陀螺）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spinning top 2**（動き）：クリップの入りから出まで。名前から：回転（中国語名 小陀螺 II）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Sway In**（動き）：クリップの入りから出まで。名前から：揺れ・回って入る・揺れ動く（振り子）（中国語名 旋入晃动）。既定 0.5 秒 — いま：ふるまい noise・キー rotate — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Sway Out**（動き）：クリップの入りから出まで。名前から：揺れ・回って出る・揺れ動く（振り子）（中国語名 晃动旋出）。既定 0.5 秒 — いま：ふるまい noise・キー rotate — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Waltzer 3**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）。既定 5.0 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Waltzer 4**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）。既定 5.0 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Wobble**（動き）：クリップの入りから出まで。名前から：拡大・揺れ・揺れ動く（振り子）（中国語名 抖入放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・ふるまい noise — 借りる先：Animate.css wobble（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Yo-yo 1**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 悠悠球）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Yo-yo 2**（動き）：クリップの入りから出まで。名前から：揺れ動く（振り子）（中国語名 悠悠球 II）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Zoom 1**（動き）：クリップの入りから出まで。名前から：拡大縮小・拡大・縮小・寄る（中国語名 缩放）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Zoom 2**（動き）：クリップの入りから出まで。名前から：拡大縮小・拡大・縮小・寄る（中国語名 缩放 II）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Zoom Spin**（動き）：クリップの入りから出まで。名前から：回転・縮小・拡大・寄る（中国語名 缩小旋转）。既定 0.5 秒 — いま：キー rotate・キー scale（区間のイーズ） — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **Zoom-Roll**（動き・有料）：クリップの入りから出まで。名前から：回って出る・縮小・回転・拡大（中国語名 缩小转出）。既定 0.5 秒 — いま：キー rotate・キー scale（区間のイーズ） — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **动感摇晃 I**（動き・有料）：クリップの入りから出まで。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **动感摇晃 II**（動き・有料）：クリップの入りから出まで。名前から：揺れ。既定 100.0 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **坠落**（動き・有料）：クリップの入りから出まで。名前から：落ちる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **弹动冲屏**（動き・有料）：クリップの入りから出まで。名前から：弾む・画面に飛び込む。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **旋出渐隐**（動き）：クリップの入りから出まで。名前から：回って出る・じわっと消える。既定 0.5 秒 — いま：キー rotate・animators fade_in・fade_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **红酒摇晃**（動き・有料）：クリップの入りから出まで。名前から：揺れ・赤。既定 0.8 秒 — いま：ふるまい noise・ISF Multi Hue Shift をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **跳跳糖**（動き・有料）：クリップの入りから出まで。名前から：跳ぶ。既定 0.7 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない

### 分割（22）

- [ ] **Bisect 1**（分割）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 中间分割）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Bisect 2**（分割）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 中间分割 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Bisect Domino 1**（分割・有料）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 左右分割）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Bisect Domino 2**（分割）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 左右分割 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Checker Slide 1**（分割）：クリップの入りから出まで。名前から：パズル・ブロック・滑る・動く。既定 5.0 秒 — いま：ISF Random Squares Mask をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Checker Slide 2**（分割）：クリップの入りから出まで。名前から：パズル・ブロック・滑る・動く。既定 5.0 秒 — いま：ISF Random Squares Mask をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Cube**（分割・有料）：クリップの入りから出まで。名前から：立方体（中国語名 立方体）。既定 0.5 秒 — いま：rotate_x・rotate_y — 借りる先：ISF-Files cube（MIT、名前が同じ）／gl-transitions cube（MIT、名前が同じ）／Animista flip-in（BSD-2）
- [ ] **Cube 2**（分割・有料）：クリップの入りから出まで。名前から：立方体（中国語名 立方体 II）。既定 0.5 秒 — いま：rotate_x・rotate_y — 借りる先：ISF-Files cube（MIT、名前が同じ）／gl-transitions cube（MIT、名前が同じ）／Animista flip-in（BSD-2）
- [ ] **Cube 3**（分割・有料）：クリップの入りから出まで。名前から：立方体（中国語名 立方体 III）。既定 0.5 秒 — いま：rotate_x・rotate_y — 借りる先：ISF-Files cube（MIT、名前が同じ）／gl-transitions cube（MIT、名前が同じ）／Animista flip-in（BSD-2）
- [ ] **Magic Cube 1**（分割・有料）：クリップの入りから出まで。名前から：ルービックキューブ・立方体・きらきら・星（中国語名 魔方）。既定 0.5 秒 — いま：rotate_x・rotate_y・ISF Star（1 個） をキー — 借りる先：Animista flip-in（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **Magic Cube 2**（分割）：クリップの入りから出まで。名前から：立方体・きらきら・星。既定 4.5 秒 — いま：rotate_x・rotate_y・ISF Star（1 個） をキー — 借りる先：Animista flip-in（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **Quarter 1**（分割）：クリップの入りから出まで。名前から：4 分割画面・裏返る・分割画面（中国語名 四格翻转）。既定 0.5 秒 — いま：ISF Quad Tile をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Quarter 2**（分割・有料）：クリップの入りから出まで。名前から：4 分割画面・裏返る・分割画面（中国語名 四格翻转 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Revolving Checker 1**（分割・有料）：クリップの入りから出まで。名前から：4 分割画面・回る・パズル・ブロック（中国語名 四格转动）。既定 0.5 秒 — いま：ISF Quad Tile をキー・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Revolving Checker 2**（分割・有料）：クリップの入りから出まで。名前から：4 分割画面・回る・パズル・ブロック（中国語名 四格转动 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin**（分割）：クリップの入りから出まで。名前から：輪・回る・回転（中国語名 转圈圈）。既定 0.5 秒 — いま：キー rotate — 借りる先：gl-transitions circleopen（MIT）をキー／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Split up & down 1**（分割・有料）：クリップの入りから出まで。名前から：分割・帯に割れる・滑る・動く（中国語名 上下分割）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Split up & down 2**（分割・有料）：クリップの入りから出まで。名前から：分割・帯に割れる・滑る・動く（中国語名 上下分割 II）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Trisect 1**（分割）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 三分割）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Trisect 2**（分割・有料）：クリップの入りから出まで。名前から：分割・分割画面（中国語名 三分割 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Window Slice（MIT）をキー
- [ ] **Waltzer 1**（分割）：クリップの入りから出まで。名前から：輪・揺れ動く（振り子）（中国語名 绕圈圈）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：gl-transitions circleopen（MIT）をキー
- [ ] **Waltzer 2**（分割）：クリップの入りから出まで。名前から：輪・揺れ動く（振り子）（中国語名 绕圈圈 II）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：gl-transitions circleopen（MIT）をキー

### レンズ・ゆがみ（11）

- [ ] **Bend-Zoom**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：変形・縮小・ゆがみ・拡大（中国語名 形变缩小）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Distort and stretch**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：ゆがみ・伸ばす・傾く（中国語名 扭曲拉伸）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **Distort Left**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：変形・ゆがみ・滑る・動く（中国語名 形变左缩）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Distort Right**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：変形・ゆがみ・滑る・動く（中国語名 形变右缩）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Funhouse mirror 1**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：びっくり鏡・ゆがみ・鏡（中国語名 哈哈镜）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・mirror.wgsl をキー — 借りる先：見つからない
- [ ] **Funhouse mirror 2**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：びっくり鏡・ゆがみ・鏡（中国語名 哈哈镜 II）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・mirror.wgsl をキー — 借りる先：見つからない
- [ ] **Slide and wave**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：波打つ・滑って入る・滑る・動く（中国語名 滑入波动）。既定 0.5 秒 — いま：animators（文字ごとにずらす）・animators slide_in・slide_out — 借りる先：anime.js stagger（MIT）
- [ ] **Stretch and distort**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：ゆがみ・伸ばす・傾く（中国語名 拉伸扭曲）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **Wave and slide**（レンズ・ゆがみ）：クリップの入りから出まで。名前から：波打つ・滑って出る・滑る・動く（中国語名 波动滑出）。既定 0.5 秒 — いま：animators（文字ごとにずらす）・animators slide_in・slide_out — 借りる先：anime.js stagger（MIT）
- [ ] **波动吸收**（レンズ・ゆがみ・有料）：クリップの入りから出まで。名前から：波打つ・吸い込む。既定 0.5 秒 — いま：animators（文字ごとにずらす）・ISF Bump Distortion をキー — 借りる先：anime.js stagger（MIT）
- [ ] **波动放大**（レンズ・ゆがみ・有料）：クリップの入りから出まで。名前から：波打つ・拡大。既定 0.5 秒 — いま：animators（文字ごとにずらす）・キー scale（区間のイーズ） — 借りる先：anime.js stagger（MIT）／Animista scale-in-center（BSD-2）

### 粒・飾り（8）

- [ ] **Crystal 1**（粒・飾り・有料）：クリップの入りから出まで。名前から：水晶（中国語名 水晶）。既定 0.5 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **Crystal 2**（粒・飾り）：クリップの入りから出まで。名前から：水晶（中国語名 水晶 II）。既定 0.5 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **Train 1**（粒・飾り）：クリップの入りから出まで。名前から：火・滑る・動く（中国語名 小火车）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Train 2**（粒・飾り）：クリップの入りから出まで。名前から：火・滑る・動く（中国語名 小火车 II）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Train 3**（粒・飾り）：クリップの入りから出まで。名前から：火・滑る・動く（中国語名 小火车 III）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Train 4**（粒・飾り）：クリップの入りから出まで。名前から：火・滑る・動く（中国語名 小火车 IV）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **Triplets 1**（粒・飾り・有料）：クリップの入りから出まで。名前から：ハート・分割画面（中国語名 夹心饼干）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **Triplets 2**（粒・飾り・有料）：クリップの入りから出まで。名前から：ハート・分割画面（中国語名 夹心饼干 II）。既定 0.5 秒 — いま：ISF Quad Tile をキー — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー

### 枠（3）

- [ ] **Smartphone 1**（枠・有料）：クリップの入りから出まで。名前から：スマホ・枠（中国語名 手机）。既定 0.5 秒 — いま：形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない
- [ ] **Smartphone 2**（枠・有料）：クリップの入りから出まで。名前から：スマホ・枠（中国語名 手机 II）。既定 0.5 秒 — いま：形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない
- [ ] **Smartphone 3**（枠・有料）：クリップの入りから出まで。名前から：スマホ・枠（中国語名 手机 III）。既定 0.5 秒 — いま：形の stroke・四角（枠は形で描ける） をキー — 借りる先：見つからない

### 残像（2）

- [ ] **Fan Out**（残像）：クリップの入りから出まで。名前から：分身（中国語名 分身 II）。既定 0.5 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **Sliding Doors 1**（残像）：クリップの入りから出まで。名前から：分身・幕・扉が開く（中国語名 分身）。既定 0.5 秒 — いま：ISF Echo Trace をキー — 借りる先：gl-transitions HorizontalOpen（MIT）をキー

### 光（2）

- [ ] **闪光放大**（光・有料）：クリップの入りから出まで。名前から：光る・拡大。既定 0.5 秒 — いま：段の exposure をキー・キー scale（区間のイーズ） — 借りる先：Animista flicker-in-glow（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **闪光放大 II**（光・有料）：クリップの入りから出まで。名前から：光る・拡大。既定 0.5 秒 — いま：段の exposure をキー・キー scale（区間のイーズ） — 借りる先：Animista flicker-in-glow（BSD-2）／Animista scale-in-center（BSD-2）

### その他（1）

- [ ] **Blinds**（その他）：クリップの入りから出まで。名前から：ブラインド（中国語名 百叶窗）。既定 0.5 秒 — いま：無い — 借りる先：ISF Window Blinds（MIT）をキー

## Text › In（文字の入り）（204）

Motolii に有る 190・近い物あり 7・見つからない 7。

### 動き（87）

- [ ] **Boing**（動き）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Bounce from TR**（動き・有料）：文字の入り。名前から：弾んで入る・弾む（中国語名 右上弹入）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Bounce In**（動き・有料）：文字の入り。名前から：弾んで入る・弾む（中国語名 弹入）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：Animate.css bounceIn（v3.7.2 は MIT、名前が同じ）
- [ ] **Bounce In TL**（動き・有料）：文字の入り。名前から：弾んで入る・弾む（中国語名 左上弹入）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Ease In Right**（動き・有料）：文字の入り。名前から：右へ・滑る・動く（中国語名 向右缓入）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Elastic Telescopic II**（動き）：文字の入り。名前から：弾力・弾む・拡大・縮小（中国語名 弹性伸缩 II）。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Fade In**（動き）：文字の入り。名前から：じわっと出る・フェード（中国語名 渐显）。既定 0.5 秒 — いま：animators fade_in・fade_out — 借りる先：Animista fade-in（BSD-2、名前が同じ）／Animate.css fadeIn（v3.7.2 は MIT、名前が同じ）
- [ ] **Flip and jump**（動き）：文字の入り。名前から：めくる・弾む・裏返る（中国語名 空翻）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators bounce_in・spring_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip up**（動き）：文字の入り。名前から：裏返る・滑る・動く。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators slide_in・slide_out — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip Verbatim**（動き・有料）：文字の入り。名前から：裏返る・一字ずつ（中国語名 逐字翻转）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators print_on・print_off — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flipping**（動き・有料）：文字の入り。名前から：めくる・裏返る（中国語名 翻动）。既定 1.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Float Down**（動き・有料）：文字の入り。名前から：飛んで入る・下へ・滑る・動く（中国語名 向下飞入）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fly In**（動き）：文字の入り。名前から：滑る・動く。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Grow**（動き）：文字の入り。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Jolt Forward**（動き）：文字の入り。名前から：画面に飛び込む・揺れ（中国語名 冲屏位移）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・ふるまい noise — 借りる先：Animista scale-in-center（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Judder Up**（動き）：文字の入り。名前から：揺れ・滑る・動く。既定 0.5 秒 — いま：ふるまい noise・animators slide_in・slide_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Mini Zoom**（動き）：文字の入り。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Pop Up**（動き）：文字の入り。名前から：弾んで入る・上へ・弾む・滑る（中国語名 向上弹入）。既定 0.5 秒 — いま：animators bounce_in・spring_in・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Random Bounce**（動き）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Roll In**（動き）：文字の入り。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animate.css rollIn（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Rotation Verbatim**（動き）：文字の入り。名前から：回転・一字ずつ。既定 0.5 秒 — いま：キー rotate・animators print_on・print_off — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Set to the Right**（動き）：文字の入り。名前から：滑る・動く。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Showing Down**（動き・有料）：文字の入り。名前から：下へ・滑る・動く（中国語名 向下露出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Showing Left**（動き・有料）：文字の入り。名前から：左へ・滑る・動く（中国語名 向左露出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Showing Right**（動き・有料）：文字の入り。名前から：右へ・滑る・動く（中国語名 向右露出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Showing Up**（動き）：文字の入り。名前から：上へ・滑る・動く（中国語名 向上露出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Slanted Expand**（動き・有料）：文字の入り。名前から：拡大・縮小・寄る・伸ばす。既定 0.5 秒 — いま：キー scale（区間のイーズ）・キー scale_x・skew — 借りる先：Animista scale-in-center（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **Slide Down**（動き・有料）：文字の入り。名前から：滑る・下へ・動く（中国語名 向下滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideInDown（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Left**（動き）：文字の入り。名前から：滑る・左へ・動く（中国語名 向左滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-in-left（BSD-2、名前が同じ）／Animate.css slideInLeft（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Right**（動き）：文字の入り。名前から：滑る・右へ・動く（中国語名 向右滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-in-right（BSD-2、名前が同じ）／Animate.css slideInRight（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Up**（動き）：文字の入り。名前から：滑る・上へ・動く（中国語名 向上滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideInUp（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Up**（動き）：文字の入り。名前から：滑る・上がる・動く（中国語名 滑动上升）。既定 1.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideInUp（v3.7.2 は MIT、名前が同じ）
- [ ] **Slingshot**（動き）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Spin In 1**（動き）：文字の入り。名前から：回って入る・回転（中国語名 旋入）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin In 2**（動き・有料）：文字の入り。名前から：回転・飛んで入る（中国語名 旋转飞入）。既定 0.5 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spring**（動き・有料）：文字の入り。名前から：ばね・弾む（中国語名 弹簧）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Sunrise**（動き）：文字の入り。名前から：日の出・日の入り。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **S线条上升**（動き・有料）：文字の入り。名前から：上がる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Throw Out**（動き）：文字の入り。名前から：振り出す・滑る・動く（中国語名 甩出）。既定 0.9 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Turn Horizontal**（動き）：文字の入り。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Wobble**（動き）：文字の入り。名前から：揺れ動く（振り子）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：Animate.css wobble（v3.7.2 は MIT、名前が同じ）
- [ ] **X轴翻起**（動き・有料）：文字の入り。名前から：めくる。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Zoom In**（動き）：文字の入り。名前から：拡大・縮小・寄る（中国語名 放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom Out**（動き）：文字の入り。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomOut（v3.7.2 は MIT、名前が同じ）／gl-transitions zoomInOut（MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom Out 2**（動き・有料）：文字の入り。名前から：縮小・拡大・寄る（中国語名 缩小 II）。既定 1.6 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomOut（v3.7.2 は MIT、名前が同じ）／gl-transitions zoomInOut（MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **上升掉落**（動き・有料）：文字の入り。名前から：上がる・落ちる。既定 1.1 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **交响弹动**（動き）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **侧身翻转**（動き）：文字の入り。名前から：裏返る。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **俏皮旋入**（動き）：文字の入り。名前から：回って入る。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **分层下落**（動き・有料）：文字の入り。名前から：落ちる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **反弹掉落**（動き・有料）：文字の入り。名前から：落ちる・弾む。既定 0.5 秒 — いま：animators slide_in・slide_out・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **向上翻转**（動き・有料）：文字の入り。名前から：裏返る・上へ。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators slide_in・slide_out — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **声纹弹跳**（動き）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **左移弹动**（動き）：文字の入り。名前から：弾む・左へ動く。既定 1.0 秒 — いま：animators bounce_in・spring_in・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **弹入跳动**（動き・有料）：文字の入り。名前から：弾んで入る・跳ねる。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **弹出**（動き）：文字の入り。名前から：弾んで出る。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **快速落下**（動き・有料）：文字の入り。名前から：落ちる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **扭转缩放**（動き・有料）：文字の入り。名前から：拡大縮小・回る。既定 0.5 秒 — いま：キー scale（区間のイーズ）・キー rotate — 借りる先：Animista scale-in-center（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **抖动拼凑**（動き・有料）：文字の入り。名前から：揺れ・つなぎ合わせ。既定 0.5 秒 — いま：ふるまい noise・ISF Quad Tile をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **抖动甩入**（動き・有料）：文字の入り。名前から：揺れ・振り込む。既定 0.5 秒 — いま：ふるまい noise・animators slide_in・slide_out — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **掀起**（動き・有料）：文字の入り。名前から：めくる。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **放大旋转**（動き・有料）：文字の入り。名前から：回転・拡大。既定 0.5 秒 — いま：キー rotate・キー scale（区間のイーズ） — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **放大震动**（動き・有料）：文字の入り。名前から：振動・拡大。既定 0.5 秒 — いま：ふるまい noise・キー scale（区間のイーズ） — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **整行环绕**（動き・有料）：文字の入り。名前から：周りを回る。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **旋转拉起**（動き・有料）：文字の入り。名前から：回転・引く。既定 0.5 秒 — いま：キー rotate・キー scale_x・skew — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **旋转摇摆**（動き・有料）：文字の入り。名前から：左右に揺れる・回転。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **旋转聚集**（動き・有料）：文字の入り。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **汇聚**（動き・有料）：文字の入り。名前から：集まる。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **滑入上升**（動き・有料）：文字の入り。名前から：滑って入る・上がる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **潜浮**（動き・有料）：文字の入り。名前から：沈む。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **环绕滑入**（動き・有料）：文字の入り。名前から：周りを回る・滑って入る。既定 1.0 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **翻转入场**（動き）：文字の入り。名前から：回って入る・めくる。既定 0.5 秒 — いま：キー rotate・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista flip-in-hor-bottom（BSD-2）
- [ ] **背景滑入**（動き・有料）：文字の入り。名前から：滑って入る。既定 1.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **跃进**（動き・有料）：文字の入り。名前から：跳ぶ。既定 1.2 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **跳跳糖**（動き・有料）：文字の入り。名前から：跳ぶ。既定 1.3 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **逆转回旋**（動き）：文字の入り。名前から：回る。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **逐字旋入**（動き・有料）：文字の入り。名前から：回って入る・一字ずつ。既定 1.5 秒 — いま：キー rotate・animators print_on・print_off — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **重叠旋转**（動き・有料）：文字の入り。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **金光弹出**（動き・有料）：文字の入り。名前から：弾んで出る・光・金色。既定 0.5 秒 — いま：animators bounce_in・spring_in・物の glow をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **错开落下**（動き・有料）：文字の入り。名前から：落ちる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **镂空跳入**（動き・有料）：文字の入り。名前から：跳ぶ。既定 1.2 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **随机上升**（動き・有料）：文字の入り。名前から：上がる。既定 1.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **随机弹跳 II**（動き・有料）：文字の入り。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **随机落下**（動き・有料）：文字の入り。名前から：落ちる。既定 1.2 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **音符抖动**（動き・有料）：文字の入り。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **顺序上滑**（動き・有料）：文字の入り。名前から：上へ滑る。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **魔幻渐显**（動き）：文字の入り。名前から：じわっと出る。既定 0.5 秒 — いま：animators fade_in・fade_out — 借りる先：見つからない

### 光（30）

- [ ] **Flicker**（光）：文字の入り。名前から：点滅（中国語名 闪动）。既定 0.5 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-1（BSD-2、名前が同じ）
- [ ] **丁达尔光效**（光）：文字の入り。名前から：光の筋・光。既定 0.5 秒 — いま：ISF God Rays をキー・物の glow をキー — 借りる先：見つからない
- [ ] **光晕**（光）：文字の入り。名前から：光のにじみ。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **光线扫描**（光）：文字の入り。名前から：光の筋・スキャン。既定 0.5 秒 — いま：ISF God Rays をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **发光弹出**（光・有料）：文字の入り。名前から：光る・弾んで出る。既定 0.5 秒 — いま：物の glow をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **发光闪入**（光・有料）：文字の入り。名前から：光る。既定 1.8 秒 — いま：物の glow をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **微光弹入**（光）：文字の入り。名前から：かすかな光・弾んで入る。既定 0.5 秒 — いま：物の glow をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **微光弹入-2**（光）：文字の入り。名前から：かすかな光・弾んで入る。既定 0.5 秒 — いま：物の glow をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **描边填充**（光・有料）：文字の入り。名前から：縁取り。既定 0.9 秒 — いま：形・文字の stroke をキー — 借りる先：見つからない
- [ ] **模糊发光**（光・有料）：文字の入り。名前から：光る・ぼかし。既定 1.8 秒 — いま：物の glow をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **聚光**（光・有料）：文字の入り。名前から：光。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **荧光聚合**（光）：文字の入り。名前から：蛍光・集まる。既定 0.5 秒 — いま：物の glow をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **荧光聚合-2**（光）：文字の入り。名前から：蛍光・集まる。既定 0.5 秒 — いま：物の glow をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **荧光聚合-3**（光）：文字の入り。名前から：蛍光・集まる。既定 0.5 秒 — いま：物の glow をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **辉光**（光・有料）：文字の入り。名前から：光のにじみ。既定 1.0 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **辉光入场**（光）：文字の入り。名前から：光のにじみ。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **辉光入场-3**（光・有料）：文字の入り。名前から：光のにじみ。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **辉光扫描**（光・有料）：文字の入り。名前から：光のにじみ・スキャン。既定 1.8 秒 — いま：物の glow をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **辉光降临**（光）：文字の入り。名前から：光のにじみ。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **辉光飞入**（光）：文字の入り。名前から：光のにじみ・飛んで入る。既定 0.5 秒 — いま：物の glow をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **金光四射**（光・有料）：文字の入り。名前から：光・金色。既定 2.0 秒 — いま：物の glow をキー・ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **闪光出击**（光・有料）：文字の入り。名前から：光る・打つ。既定 0.5 秒 — いま：段の exposure をキー・ふるまい noise — 借りる先：Animista flicker-in-glow（BSD-2）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **闪烁 II**（光・有料）：文字の入り。名前から：点滅。既定 0.5 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **闪烁集合**（光・有料）：文字の入り。名前から：点滅。既定 1.5 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **霓虹流动**（光・有料）：文字の入り。名前から：ネオン・流れる。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：ISF Neon（MIT） をキー／anime.js stagger（MIT）
- [ ] **霓虹灯闪烁**（光・有料）：文字の入り。名前から：ネオン・点滅。既定 2.0 秒 — いま：ふるまい random を opacity に — 借りる先：ISF Neon（MIT） をキー／Animista flicker-in-1（BSD-2）
- [ ] **扫光入场**（光）：文字の入り。名前から：光が走る。既定 0.5 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **激光雕刻**（光・有料）：文字の入り。名前から：レーザー。既定 3.0 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **电光**（光・有料）：文字の入り。名前から：電気の光。既定 1.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **电光 II**（光・有料）：文字の入り。名前から：電気の光。既定 1.0 秒 — いま：無い — 借りる先：見つからない

### レンズ・ゆがみ（17）

- [ ] **Bumper Car**（レンズ・ゆがみ・有料）：文字の入り。名前から：ゆがみ・弾む。既定 0.6 秒 — いま：ISF Bump Distortion をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Flutter**（レンズ・ゆがみ）：文字の入り。名前から：ゆがみ・ぼかし・揺れ動く（振り子）（中国語名 扭曲模糊）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Spiral**（レンズ・ゆがみ）：文字の入り。名前から：渦・ねじれ。既定 0.5 秒 — いま：ISF Twirl をキー — 借りる先：ISF-Files Spiral（MIT、名前が同じ）／Animista swirl-in-fwd（BSD-2）
- [ ] **Squeeze**（レンズ・ゆがみ）：文字の入り。名前から：ゆがみ。既定 0.5 秒 — いま：ISF Bump Distortion をキー — 借りる先：ISF-Files Squeeze（MIT、名前が同じ）／gl-transitions squeeze（MIT、名前が同じ）
- [ ] **Wave in**（レンズ・ゆがみ）：文字の入り。名前から：波打つ・弾んで入る（中国語名 波浪弹入）。既定 0.5 秒 — いま：animators（文字ごとにずらす）・animators bounce_in・spring_in — 借りる先：anime.js stagger（MIT）
- [ ] **像素抖动**（レンズ・ゆがみ・有料）：文字の入り。名前から：ピクセル・揺れ。既定 0.5 秒 — いま：ISF Pixellate をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **吸入**（レンズ・ゆがみ・有料）：文字の入り。名前から：吸い込む。既定 0.3 秒 — いま：ISF Bump Distortion をキー — 借りる先：見つからない
- [ ] **呐喊声波**（レンズ・ゆがみ・有料）：文字の入り。名前から：波。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **扭曲发光**（レンズ・ゆがみ・有料）：文字の入り。名前から：ゆがみ・光る。既定 0.5 秒 — いま：ISF Bump Distortion をキー・物の glow をキー — 借りる先：見つからない
- [ ] **时空漩涡-入场**（レンズ・ゆがみ・有料）：文字の入り。名前から：渦。既定 0.5 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **波浪弹跳**（レンズ・ゆがみ・有料）：文字の入り。名前から：波打つ・弾む。既定 1.0 秒 — いま：animators（文字ごとにずらす）・animators bounce_in・spring_in — 借りる先：anime.js stagger（MIT）
- [ ] **波浪律动**（レンズ・ゆがみ）：文字の入り。名前から：波打つ・リズム。既定 0.5 秒 — いま：animators（文字ごとにずらす）・ふるまい oscillator を scale に — 借りる先：anime.js stagger（MIT）／Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **波浪放大**（レンズ・ゆがみ・有料）：文字の入り。名前から：波打つ・拡大。既定 0.5 秒 — いま：animators（文字ごとにずらす）・キー scale（区間のイーズ） — 借りる先：anime.js stagger（MIT）／Animista scale-in-center（BSD-2）
- [ ] **波浪流动 入场**（レンズ・ゆがみ・有料）：文字の入り。名前から：波打つ・流れる。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **波浪翻转**（レンズ・ゆがみ）：文字の入り。名前から：波打つ・裏返る。既定 0.5 秒 — いま：animators（文字ごとにずらす）・rotate_x・rotate_y・animators swing_3d_in — 借りる先：anime.js stagger（MIT）／Animista flip-in-hor-bottom（BSD-2）
- [ ] **色彩漩涡-入场**（レンズ・ゆがみ・有料）：文字の入り。名前から：渦・色。既定 0.5 秒 — いま：ISF Twirl をキー・ISF Multi Hue Shift をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **螺旋丸**（レンズ・ゆがみ）：文字の入り。名前から：らせん。既定 0.5 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）

### 分割（16）

- [ ] **Blur Wipe Left**（分割・有料）：文字の入り。名前から：左へ拭く・ぼかし・拭き取る・滑る（中国語名 羽化向左擦开）。既定 0.5 秒 — いま：ISF Wipe をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blur Wipe Right**（分割・有料）：文字の入り。名前から：右へ拭く・ぼかし・拭き取る・滑る（中国語名 羽化向右擦开）。既定 0.5 秒 — いま：ISF Wipe をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Clock wipe**（分割）：文字の入り。名前から：時計回りに拭く・拭き取る。既定 0.5 秒 — いま：ISF Radial をキー・ISF Wipe をキー — 借りる先：見つからない
- [ ] **Wipe Down**（分割）：文字の入り。名前から：拭き取る・滑る・動く。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Down（MIT、名前が同じ）／gl-transitions wipeDown（MIT、名前が同じ）
- [ ] **Wipe In LR**（分割）：文字の入り。名前から：拭き取る。既定 0.5 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **Wipe Left**（分割・有料）：文字の入り。名前から：拭き取る・左へ・滑る・動く（中国語名 向左擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Left（MIT、名前が同じ）／gl-transitions wipeLeft（MIT、名前が同じ）
- [ ] **Wipe Right**（分割・有料）：文字の入り。名前から：拭き取る・右へ・滑る・動く（中国語名 向右擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Right（MIT、名前が同じ）／gl-transitions wipeRight（MIT、名前が同じ）
- [ ] **Wipe Up**（分割・有料）：文字の入り。名前から：拭き取る・上へ・滑る・動く（中国語名 向上擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Up（MIT、名前が同じ）／gl-transitions wipeUp（MIT、名前が同じ）
- [ ] **Wiping In**（分割・有料）：文字の入り。名前から：拭き取る。既定 0.5 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **喷绘**（分割・有料）：文字の入り。名前から：スプレー描き。既定 0.5 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **漂浮翻页**（分割）：文字の入り。名前から：ページめくり・浮かぶ。既定 0.5 秒 — いま：ISF Inverted Page Curl をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **炫彩翻页**（分割）：文字の入り。名前から：ページめくり・カラフル。既定 0.5 秒 — いま：ISF Inverted Page Curl をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **翻页入场**（分割）：文字の入り。名前から：ページめくり。既定 0.5 秒 — いま：ISF Inverted Page Curl をキー — 借りる先：見つからない
- [ ] **翻页渐显**（分割）：文字の入り。名前から：ページめくり・じわっと出る。既定 0.5 秒 — いま：ISF Inverted Page Curl をキー・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **Fold**（分割）：文字の入り。名前から：折る。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions Fold（MIT、名前が同じ）／Animista flip-in（BSD-2）
- [ ] **Open**（分割）：文字の入り。名前から：幕が開く・幕・扉が開く（中国語名 开幕）。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions HorizontalOpen（MIT）をキー

### 文字（14）

- [ ] **Concentrate**（文字・有料）：文字の入り。名前から：一字ずつ・影・拡大・縮小（中国語名 逐字显影）。既定 0.5 秒 — いま：animators print_on・print_off・ISF Echo Trace をキー — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Faulty text**（文字）：文字の入り。名前から：タイプライター・グリッチ（中国語名 故障打字机）。既定 0.5 秒 — いま：animators print_on・print_off・ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **Random Typewriter**（文字）：文字の入り。名前から：タイプライター・一字ずつ（中国語名 随机打字机）。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 1**（文字）：文字の入り。名前から：タイプライター・一字ずつ（中国語名 打字机 I）。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 2**（文字）：文字の入り。名前から：タイプライター・一字ずつ（中国語名 打字机 II）。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 3**（文字・有料）：文字の入り。名前から：タイプライター・一字ずつ（中国語名 打字机 III）。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Typewriter**（文字）：文字の入り。名前から：タイプライター・一字ずつ（中国語名 打字机）。既定 2.0 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **倒数**（文字・有料）：文字の入り。名前から：カウントダウン。既定 1.8 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **复古打字机**（文字・有料）：文字の入り。名前から：タイプライター・レトロ。既定 0.8 秒 — いま：animators print_on・print_off・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない
- [ ] **打字光标**（文字・有料）：文字の入り。名前から：タイプ・光。既定 2.0 秒 — いま：animators print_on・print_off・物の glow をキー — 借りる先：見つからない
- [ ] **逐字**（文字）：文字の入り。名前から：一字ずつ。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **逐字冒出**（文字・有料）：文字の入り。名前から：一字ずつ。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **预览打字**（文字・有料）：文字の入り。名前から：タイプ。既定 0.8 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Karaoke**（文字）：文字の入り。名前から：カラオケ（中国語名 卡拉OK）。既定 0.5 秒 — いま：無い — 借りる先：見つからない

### 粒・飾り（11）

- [ ] **Blow In**（粒・飾り）：文字の入り。名前から：溶ける。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Dissolve**（粒・飾り）：文字の入り。名前から：溶ける（中国語名 溶解（入场））。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：gl-transitions dissolve（MIT、名前が同じ）／Animista puff-in-center（BSD-2）
- [ ] **Dissolve Down**（粒・飾り）：文字の入り。名前から：溶ける・下へ・滑る・動く（中国語名 向下溶解）。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・animators slide_in・slide_out — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Golden Dust**（粒・飾り・有料）：文字の入り。名前から：粒子。既定 2.0 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **冰雪飘动**（粒・飾り・有料）：文字の入り。名前から：雪・漂う・氷。既定 1.5 秒 — いま：animators slide_in・slide_out・ISF Frosted Glass をキー — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity をキー
- [ ] **星光闪闪**（粒・飾り・有料）：文字の入り。名前から：星の光・きらきら。既定 1.5 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **星光闪闪 II**（粒・飾り・有料）：文字の入り。名前から：星の光・きらきら。既定 1.0 秒 — いま：ISF Star（1 個） をキー — 借りる先：Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **星星粒子**（粒・飾り・有料）：文字の入り。名前から：粒子・星。既定 3.0 秒 — いま：点群 dissolve（点群だけ）・ISF Star（1 個） をキー — 借りる先：Animista puff-in-center（BSD-2）／Kenney Particle Pack の星・光（CC0）＋物の copies・scatter をキー
- [ ] **可爱悦动**（粒・飾り）：文字の入り。名前から：愛。既定 0.5 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **心动**（粒・飾り・有料）：文字の入り。名前から：ときめき。既定 1.6 秒 — いま：無い — 借りる先：ISF Heart（MIT）＋物の copies・scatter をキー
- [ ] **玩雪**（粒・飾り・有料）：文字の入り。名前から：雪。既定 60.0 秒 — いま：無い — 借りる先：Kenney Particle Pack（CC0）＋物の scatter・段の gravity をキー

### グリッチ（9）

- [ ] **Glitch**（グリッチ）：文字の入り。名前から：グリッチ（中国語名 故障）。既定 1.6 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **rgb棱镜**（グリッチ・有料）：文字の入り。名前から：プリズム。既定 0.5 秒 — いま：rgbshift.wgsl をキー — 借りる先：見つからない
- [ ] **乱码故障**（グリッチ・有料）：文字の入り。名前から：グリッチ・文字化け。既定 1.0 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障弹跳**（グリッチ）：文字の入り。名前から：グリッチ・弾む。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **故障翻转**（グリッチ・有料）：文字の入り。名前から：グリッチ・裏返る。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **故障色差 II**（グリッチ・有料）：文字の入り。名前から：色ずれ・グリッチ。既定 0.5 秒 — いま：rgbshift.wgsl をキー・ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障闪动**（グリッチ・有料）：文字の入り。名前から：グリッチ・点滅。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **色差拖尾**（グリッチ・有料）：文字の入り。名前から：色ずれ・尾を引く。既定 0.5 秒 — いま：rgbshift.wgsl をキー・ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **色散拖影**（グリッチ・有料）：文字の入り。名前から：色の分散・残像。既定 1.1 秒 — いま：rgbshift.wgsl をキー・ISF Echo Trace をキー — 借りる先：見つからない

### ぼかし（8）

- [ ] **Blur**（ぼかし）：文字の入り。名前から：ぼかし（中国語名 模糊）。既定 0.5 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blur to the Left**（ぼかし）：文字の入り。名前から：ぼかし・左へ・滑る・動く（中国語名 向左模糊）。既定 0.5 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blur to the right**（ぼかし・有料）：文字の入り。名前から：ぼかし・右へ・滑る・動く（中国語名 向右模糊）。既定 0.5 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **向右模糊 II**（ぼかし・有料）：文字の入り。名前から：ぼかし・右へ。既定 0.5 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊拼凑**（ぼかし）：文字の入り。名前から：ぼかし・つなぎ合わせ。既定 0.5 秒 — いま：物の blur をキー・ISF Quad Tile をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊滚动**（ぼかし・有料）：文字の入り。名前から：ぼかし・スクロール。既定 0.8 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊缩小**（ぼかし・有料）：文字の入り。名前から：ぼかし・縮小。既定 1.5 秒 — いま：物の blur をキー・キー scale（区間のイーズ） — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）／Animista scale-in-center（BSD-2）
- [ ] **雪光模糊**（ぼかし・有料）：文字の入り。名前から：ぼかし・雪・光。既定 1.2 秒 — いま：物の blur をキー・物の glow をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）／Kenney Particle Pack（CC0）＋物の scatter・段の gravity をキー

### 色（5）

- [ ] **Chroma Type**（色・有料）：文字の入り。名前から：色が変わる・カラー・タイプ・色ずれ（中国語名 彩色变色打字）。既定 0.8 秒 — いま：ISF Multi Hue Shift をキー・animators print_on・print_off — 借りる先：見つからない
- [ ] **七彩散光**（色）：文字の入り。名前から：虹色・光。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー・物の glow をキー — 借りる先：見つからない
- [ ] **炫彩弹入**（色）：文字の入り。名前から：カラフル・弾んで入る。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **缤纷冲屏**（色・有料）：文字の入り。名前から：カラフル・画面に飛び込む。既定 2.0 秒 — いま：ISF Multi Hue Shift をキー・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **缤纷弹跳**（色・有料）：文字の入り。名前から：カラフル・弾む。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー・animators bounce_in・spring_in — 借りる先：見つからない

### 残像（3）

- [ ] **Trail**（残像・有料）：文字の入り。名前から：尾を引く・残像・分身（中国語名 拖尾）。既定 0.5 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **叠影并入**（残像・有料）：文字の入り。名前から：重なる像。既定 1.0 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **空间重影**（残像・有料）：文字の入り。名前から：二重像・空間。既定 0.5 秒 — いま：ISF Echo Trace をキー・rotate_x・rotate_y・z — 借りる先：Animista tilt-in（BSD-2）

### その他（4）

- [ ] **Click**（その他・有料）：文字の入り。名前だけでは中身が分からない。既定 1.6 秒 — いま：無い — 借りる先：見つからない
- [ ] **丝滑环形 入场**（その他・有料）：文字の入り。名前だけでは中身が分からない。既定 0.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **往上靠近**（その他・有料）：文字の入り。名前だけでは中身が分からない。既定 0.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **顶出**（その他・有料）：文字の入り。名前だけでは中身が分からない。既定 1.1 秒 — いま：無い — 借りる先：見つからない

## Text › Out（文字の出）（117）

Motolii に有る 111・近い物あり 3・見つからない 3。

### 動き（48）

- [ ] **Blow Away**（動き・有料）：文字の出。名前から：飛んで出る・溶ける（中国語名 随机飞出）。既定 0.5 秒 — いま：animators slide_in・slide_out・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Boing**（動き）：文字の出。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Bounce Out**（動き・有料）：文字の出。名前から：弾んで出る・弾む（中国語名 弹出）。既定 1.6 秒 — いま：animators bounce_in・spring_in — 借りる先：Animate.css bounceOut（v3.7.2 は MIT、名前が同じ）
- [ ] **Bounce Out TL**（動き）：文字の出。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Bounce Out TR**（動き・有料）：文字の出。名前から：弾んで出る・弾む（中国語名 右上弹出）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Ease Out Right**（動き・有料）：文字の出。名前から：右へ・滑る・動く（中国語名 向右缓出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Elastic Telescopic II**（動き・有料）：文字の出。名前から：弾力・弾む・拡大・縮小（中国語名 弹性伸缩 II）。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Fade Out**（動き）：文字の出。名前から：じわっと消える・フェード（中国語名 渐隐）。既定 1.6 秒 — いま：animators fade_in・fade_out — 借りる先：Animista fade-out（BSD-2、名前が同じ）／Animate.css fadeOut（v3.7.2 は MIT、名前が同じ）
- [ ] **Flip Down**（動き）：文字の出。名前から：裏返る・滑る・動く。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators slide_in・slide_out — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flip Verbatim**（動き・有料）：文字の出。名前から：裏返る・一字ずつ（中国語名 逐字翻转）。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators print_on・print_off — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Flipping**（動き）：文字の出。名前から：めくる・裏返る（中国語名 翻动）。既定 1.0 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Float Up**（動き・有料）：文字の出。名前から：飛んで出る・上へ・滑る・動く（中国語名 向上飞出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Fly Out**（動き）：文字の出。名前から：飛んで出る・滑る・動く（中国語名 飞出）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Grow**（動き）：文字の出。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Jump and flip**（動き）：文字の出。名前から：弾む・裏返る。既定 0.5 秒 — いま：animators bounce_in・spring_in・rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Mini Zoom**（動き）：文字の出。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Pop Down**（動き・有料）：文字の出。名前から：弾んで出る・下へ・弾む・滑る（中国語名 向下弹出）。既定 0.5 秒 — いま：animators bounce_in・spring_in・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Random Bounce**（動き）：文字の出。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Roll Out**（動き）：文字の出。名前から：回転（中国語名 滚出）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animate.css rollOut（v3.7.2 は MIT、名前が同じ）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Rotation Verbatim**（動き）：文字の出。名前から：回転・一字ずつ。既定 0.5 秒 — いま：キー rotate・animators print_on・print_off — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Slide Down**（動き）：文字の出。名前から：滑る・下へ・動く（中国語名 向下滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideOutDown（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Down**（動き）：文字の出。名前から：滑る・落ちる・動く（中国語名 滑动下落）。既定 1.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideOutDown（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Left**（動き）：文字の出。名前から：滑る・左へ・動く（中国語名 向左滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-out-left（BSD-2、名前が同じ）／Animate.css slideOutLeft（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Right**（動き）：文字の出。名前から：滑る・右へ・動く（中国語名 向右滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animista slide-out-right（BSD-2、名前が同じ）／Animate.css slideOutRight（v3.7.2 は MIT、名前が同じ）
- [ ] **Slide Up**（動き・有料）：文字の出。名前から：滑る・上へ・動く（中国語名 向上滑动）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：Animate.css slideOutUp（v3.7.2 は MIT、名前が同じ）
- [ ] **Slingshot**（動き・有料）：文字の出。名前から：弾む（中国語名 弹弓）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Spin Out 1**（動き）：文字の出。名前から：回って出る・回転（中国語名 旋出）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spin Out 2**（動き）：文字の出。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Spring**（動き）：文字の出。名前から：ばね・弾む（中国語名 弹簧）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Sunset**（動き・有料）：文字の出。名前から：落ちる・日の出・日の入り（中国語名 日落）。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Throw Back**（動き・有料）：文字の出。名前から：振る・滑る・動く（中国語名 甩回）。既定 0.6 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Turn Horizontal**（動き）：文字の出。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Zoom In**（動き）：文字の出。名前から：拡大・縮小・寄る。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom In 2**（動き・有料）：文字の出。名前から：拡大・縮小・寄る（中国語名 放大 II）。既定 1.6 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomIn（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **Zoom Out**（動き）：文字の出。名前から：縮小・拡大・寄る（中国語名 缩小）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animate.css zoomOut（v3.7.2 は MIT、名前が同じ）／Animista scale-in-center（BSD-2）
- [ ] **下降掉落**（動き・有料）：文字の出。名前から：下がる・落ちる。既定 1.1 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **二段缩放**（動き・有料）：文字の出。名前から：拡大縮小。既定 1.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **向下翻转**（動き・有料）：文字の出。名前から：裏返る・下へ。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in・animators slide_in・slide_out — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **左移弹动**（動き・有料）：文字の出。名前から：弾む・左へ動く。既定 1.0 秒 — いま：animators bounce_in・spring_in・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **收缩震动**（動き・有料）：文字の出。名前から：振動。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **时光隧道**（動き）：文字の出。名前から：トンネル・光。既定 0.5 秒 — いま：キー scale（区間のイーズ）・物の glow をキー — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **时空旋转**（動き）：文字の出。名前から：回転。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **滑出下降**（動き・有料）：文字の出。名前から：滑って出る・下がる。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **狂风呼啸**（動き）：文字の出。名前から：強風。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **环绕滑出**（動き・有料）：文字の出。名前から：周りを回る・滑って出る。既定 1.0 秒 — いま：キー rotate・animators slide_in・slide_out — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **镂空跳出**（動き・有料）：文字の出。名前から：跳ぶ。既定 1.2 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **随机弹跳 II**（動き・有料）：文字の出。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **顺序下滑**（動き・有料）：文字の出。名前から：下へ滑る。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない

### 分割（12）

- [ ] **Blur Erase Left**（分割・有料）：文字の出。名前から：拭き取る・左へ・ぼかし・滑る（中国語名 羽化向左擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blur Erase Right**（分割・有料）：文字の出。名前から：拭き取る・右へ・ぼかし・滑る（中国語名 羽化向右擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Expand**（分割）：文字の出。名前から：広がる・拡大・縮小・寄る（中国語名 展开）。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：gl-transitions HorizontalOpen（MIT）をキー／Animista scale-in-center（BSD-2）
- [ ] **Ink Print**（分割・有料）：文字の出。名前から：スプレー描き・溶ける（中国語名 喷绘）。既定 0.5 秒 — いま：ISF Wipe をキー・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Wipe Down**（分割・有料）：文字の出。名前から：拭き取る・下へ・滑る・動く（中国語名 向下擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Down（MIT、名前が同じ）／gl-transitions wipeDown（MIT、名前が同じ）
- [ ] **Wipe Left**（分割・有料）：文字の出。名前から：拭き取る・左へ・滑る・動く（中国語名 向左擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Left（MIT、名前が同じ）／gl-transitions wipeLeft（MIT、名前が同じ）
- [ ] **Wipe Right**（分割・有料）：文字の出。名前から：拭き取る・右へ・滑る・動く（中国語名 向右擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Right（MIT、名前が同じ）／gl-transitions wipeRight（MIT、名前が同じ）
- [ ] **Wipe Up**（分割・有料）：文字の出。名前から：拭き取る・上へ・滑る・動く（中国語名 向上擦除）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：ISF-Files Wipe Up（MIT、名前が同じ）／gl-transitions wipeUp（MIT、名前が同じ）
- [ ] **Wiping Out**（分割・有料）：文字の出。名前から：拭き取る。既定 0.5 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **格式化**（分割）：文字の出。名前から：格子。既定 0.5 秒 — いま：ISF Random Squares Mask をキー — 借りる先：見つからない
- [ ] **Fold**（分割）：文字の出。名前から：折る。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions Fold（MIT、名前が同じ）／Animista flip-in（BSD-2）
- [ ] **Horizontal Close**（分割）：文字の出。名前から：幕が閉じる・幕・扉が開く（中国語名 闭幕）。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions HorizontalClose（MIT、名前が同じ）

### 文字（12）

- [ ] **Dilute**（文字・有料）：文字の出。名前から：一字ずつ・影・溶ける（中国語名 逐字虚影）。既定 0.5 秒 — いま：animators print_on・print_off・ISF Echo Trace をキー — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Faulty text**（文字）：文字の出。名前から：タイプライター・グリッチ（中国語名 故障打字机）。既定 0.5 秒 — いま：animators print_on・print_off・ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **Random Typewriter**（文字）：文字の出。名前から：タイプライター・一字ずつ（中国語名 随机打字机）。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 1**（文字）：文字の出。名前から：一字ずつ。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 2**（文字）：文字の出。名前から：一字ずつ。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Type 3**（文字）：文字の出。名前から：一字ずつ。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **Typewriter**（文字）：文字の出。名前から：タイプライター・一字ずつ（中国語名 打字机）。既定 2.0 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **复古打字机**（文字・有料）：文字の出。名前から：タイプライター・レトロ。既定 0.8 秒 — いま：animators print_on・print_off・film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー — 借りる先：見つからない
- [ ] **字符延长**（文字・有料）：文字の出。名前から：字。既定 2.0 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **打字光标**（文字・有料）：文字の出。名前から：タイプ・光。既定 2.0 秒 — いま：animators print_on・print_off・物の glow をキー — 借りる先：見つからない
- [ ] **破开文字_英文**（文字）：文字の出。名前から：文字。既定 2.0 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **预览打字**（文字・有料）：文字の出。名前から：タイプ。既定 0.8 秒 — いま：animators print_on・print_off — 借りる先：見つからない

### 光（12）

- [ ] **Clock wipe**（光・有料）：文字の出。名前から：スキャン・丸・時計回りに拭く・拭き取る（中国語名 圆形扫描）。既定 0.5 秒 — いま：ISF Radial をキー・ISF Wipe をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー／gl-transitions circleopen（MIT）をキー
- [ ] **Flicker**（光）：文字の出。名前から：点滅。既定 0.5 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-1（BSD-2、名前が同じ）
- [ ] **Stroke Fill**（光・有料）：文字の出。名前から：縁取り（中国語名 描边填充）。既定 0.9 秒 — いま：形・文字の stroke をキー — 借りる先：見つからない
- [ ] **发光消散溶解**（光・有料）：文字の出。名前から：光る・溶ける・消え散る。既定 0.5 秒 — いま：物の glow をキー・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **发光闪出**（光・有料）：文字の出。名前から：光る。既定 1.8 秒 — いま：物の glow をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **模糊发光**（光・有料）：文字の出。名前から：光る・ぼかし。既定 1.8 秒 — いま：物の glow をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **紫光扫射**（光・有料）：文字の出。名前から：光が走る・紫。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **辉光渐隐**（光）：文字の出。名前から：光のにじみ・じわっと消える。既定 0.5 秒 — いま：物の glow をキー・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **辉光渐隐-2**（光）：文字の出。名前から：光のにじみ・じわっと消える。既定 0.5 秒 — いま：物の glow をキー・animators fade_in・fade_out — 借りる先：見つからない
- [ ] **闪烁散开**（光・有料）：文字の出。名前から：点滅。既定 1.0 秒 — いま：ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **激光雕刻**（光・有料）：文字の出。名前から：レーザー。既定 1.2 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **电光碎裂**（光）：文字の出。名前から：電気の光・砕ける。既定 0.5 秒 — いま：無い — 借りる先：見つからない

### 粒・飾り（11）

- [ ] **Disband to the Left**（粒・飾り）：文字の出。名前から：溶ける・滑る・動く。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・animators slide_in・slide_out — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Dissolve**（粒・飾り）：文字の出。名前から：溶ける（中国語名 溶解（出场））。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：gl-transitions dissolve（MIT、名前が同じ）／Animista puff-in-center（BSD-2）
- [ ] **Dissolve Up**（粒・飾り）：文字の出。名前から：溶ける・上へ・滑る・動く（中国語名 向上溶解）。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・animators slide_in・slide_out — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **倾斜消失**（粒・飾り・有料）：文字の出。名前から：消える・傾く。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・キー scale_x・skew — 借りる先：Animista puff-in-center（BSD-2）／Animista tracking-in-expand（BSD-2）
- [ ] **彩墨消融**（粒・飾り・有料）：文字の出。名前から：溶ける・墨。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **水墨晕开**（粒・飾り）：文字の出。名前から：水墨・にじむ。既定 1.2 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **消散**（粒・飾り・有料）：文字の出。名前から：消え散る。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **炸开**（粒・飾り・有料）：文字の出。名前から：はじける。既定 1.2 秒 — いま：レシピ burst.js・confetti.js・点群 Appear Eruption をキー — 借りる先：見つからない
- [ ] **粉尘辐射**（粒・飾り・有料）：文字の出。名前から：粉じん。既定 0.5 秒 — いま：点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **粒子碎落**（粒・飾り・有料）：文字の出。名前から：粒子・落ちる。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・animators slide_in・slide_out — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **红月消散**（粒・飾り・有料）：文字の出。名前から：消え散る・赤。既定 0.5 秒 — いま：点群 dissolve（点群だけ）・ISF Multi Hue Shift をキー — 借りる先：Animista puff-in-center（BSD-2）

### レンズ・ゆがみ（9）

- [ ] **Flutter**（レンズ・ゆがみ）：文字の出。名前から：ゆがみ・ぼかし・揺れ動く（振り子）（中国語名 扭曲模糊）。既定 0.5 秒 — いま：ISF Bump Distortion をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Vrille**（レンズ・ゆがみ）：文字の出。名前から：らせん・下がる・揺れ動く（振り子）・回転（中国語名 螺旋下降）。既定 0.5 秒 — いま：ISF Twirl をキー・animators slide_in・slide_out — 借りる先：Animista swirl-in-fwd（BSD-2）／Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Wave out**（レンズ・ゆがみ・有料）：文字の出。名前から：波打つ・弾んで出る（中国語名 波浪弹出）。既定 0.5 秒 — いま：animators（文字ごとにずらす）・animators bounce_in・spring_in — 借りる先：anime.js stagger（MIT）
- [ ] **吸出**（レンズ・ゆがみ・有料）：文字の出。名前から：吐き出す。既定 0.3 秒 — いま：ISF Bump Distortion をキー — 借りる先：見つからない
- [ ] **时空漩涡**（レンズ・ゆがみ・有料）：文字の出。名前から：渦。既定 0.5 秒 — いま：ISF Twirl をキー — 借りる先：Animista swirl-in-fwd（BSD-2）
- [ ] **波浪弹跳**（レンズ・ゆがみ・有料）：文字の出。名前から：波打つ・弾む。既定 1.0 秒 — いま：animators（文字ごとにずらす）・animators bounce_in・spring_in — 借りる先：anime.js stagger（MIT）
- [ ] **波浪流动 出场**（レンズ・ゆがみ・有料）：文字の出。名前から：波打つ・流れる。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **涟漪**（レンズ・ゆがみ）：文字の出。名前から：さざ波。既定 0.5 秒 — いま：ISF Ripples をキー — 借りる先：見つからない
- [ ] **色彩漩涡-出场**（レンズ・ゆがみ・有料）：文字の出。名前から：渦・色。既定 0.5 秒 — いま：ISF Twirl をキー・ISF Multi Hue Shift をキー — 借りる先：Animista swirl-in-fwd（BSD-2）

### グリッチ（4）

- [ ] **Chromatic Aberration Tailing**（グリッチ・有料）：文字の出。名前から：色ずれ・尾を引く（中国語名 色差拖尾）。既定 0.5 秒 — いま：rgbshift.wgsl をキー・ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **Glitch**（グリッチ）：文字の出。名前から：グリッチ（中国語名 故障）。既定 1.6 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **信号失真**（グリッチ）：文字の出。名前から：信号の乱れ・ゆがみ。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・ISF Bump Distortion をキー — 借りる先：見つからない
- [ ] **故障闪动**（グリッチ・有料）：文字の出。名前から：グリッチ・点滅。既定 2.0 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）

### ぼかし（4）

- [ ] **Blur**（ぼかし）：文字の出。名前から：ぼかし（中国語名 模糊）。既定 0.5 秒 — いま：物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Blur to the Left**（ぼかし・有料）：文字の出。名前から：ぼかし・左へ・滑る・動く（中国語名 向左模糊）。既定 0.5 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **向左模糊 II**（ぼかし・有料）：文字の出。名前から：ぼかし・左へ。既定 0.5 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **模糊滚动**（ぼかし・有料）：文字の出。名前から：ぼかし・スクロール。既定 0.8 秒 — いま：物の blur をキー・animators slide_in・slide_out — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）

### 残像（2）

- [ ] **Trail**（残像）：文字の出。名前から：尾を引く・残像・分身（中国語名 拖尾）。既定 0.5 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **叠影并出**（残像・有料）：文字の出。名前から：重なる像。既定 1.0 秒 — いま：ISF Echo Trace をキー — 借りる先：見つからない

### 色（1）

- [ ] **流紫**（色）：文字の出。名前から：紫。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：見つからない

### その他（2）

- [ ] **丝滑环形 出场**（その他・有料）：文字の出。名前だけでは中身が分からない。既定 0.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **顶出**（その他・有料）：文字の出。名前だけでは中身が分からない。既定 1.1 秒 — いま：無い — 借りる先：見つからない

## Text › Loop（文字のくり返し）（96）

Motolii に有る 84・近い物あり 7・見つからない 5。

### 動き（37）

- [ ] **Dance**（動き・有料）：文字のくり返し。名前から：揺れ・踊る（揺れ）（中国語名 摇荡）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Enlarge Letters**（動き）：文字のくり返し。名前から：拡大・一字ずつ・縮小・寄る（中国語名 逐字放大）。既定 0.5 秒 — いま：キー scale（区間のイーズ）・animators print_on・print_off — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **Flip**（動き）：文字のくり返し。名前から：裏返る。既定 0.5 秒 — いま：rotate_x・rotate_y・animators swing_3d_in — 借りる先：Animista flip-in-hor-bottom（BSD-2）
- [ ] **Gushing**（動き）：文字のくり返し。名前から：文字が大きく動く。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Jiggle**（動き・有料）：文字のくり返し。名前から：跳ねる・揺れ（中国語名 轻微跳动）。既定 0.5 秒 — いま：animators bounce_in・spring_in・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Jiggly**（動き）：文字のくり返し。名前から：揺れ（中国語名 调皮）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Jump**（動き）：文字のくり返し。名前から：弾む。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Pressing**（動き）：文字のくり返し。名前から：脈打つ。既定 0.5 秒 — いま：ふるまい oscillator を scale に — 借りる先：Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **Pulse**（動き）：文字のくり返し。名前から：脈打つ。既定 0.5 秒 — いま：ふるまい oscillator を scale に — 借りる先：Animate.css pulse（v3.7.2 は MIT、名前が同じ）／Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **Random Bounce**（動き）：文字のくり返し。名前から：弾む（中国語名 随机弹跳）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Rotate**（動き・有料）：文字のくり返し。名前から：周りを回る・回転（中国語名 环绕）。既定 0.5 秒 — いま：キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Scream**（動き・有料）：文字のくり返し。名前から：文字が大きく動く（中国語名 呐喊）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Scroll Up**（動き・有料）：文字のくり返し。名前から：スクロール・字幕・滑る・動く（中国語名 字幕滚动）。既定 0.5 秒 — いま：animators slide_in・slide_out・animators print_on・print_off — 借りる先：見つからない
- [ ] **Shake**（動き・有料）：文字のくり返し。名前から：左右に揺れる・揺れ（中国語名 摇摆 I）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Shake 1**（動き）：文字のくり返し。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Shout Out**（動き）：文字のくり返し。名前から：文字が大きく動く（中国語名 呼喊）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Strong Tremble 1**（動き）：文字のくり返し。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Strong Tremble 2**（動き）：文字のくり返し。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Swing**（動き）：文字のくり返し。名前から：揺れ動く（振り子）。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **Top Arch**（動き）：文字のくり返し。名前から：文字が大きく動く（中国語名 上弧）。既定 0.5 秒 — いま：animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Tremble**（動き）：文字のくり返し。名前から：揺れ（中国語名 晃动）。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Wobble**（動き・有料）：文字のくり返し。名前から：震え・揺れ動く（振り子）（中国語名 颤抖）。既定 0.5 秒 — いま：ふるまい noise・animators swing_3d_in・swing_3d_out — 借りる先：Animate.css wobble（v3.7.2 は MIT、名前が同じ）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Wobble 2**（動き）：文字のくり返し。名前から：震え・揺れ動く（振り子）（中国語名 颤抖 II）。既定 0.5 秒 — いま：ふるまい noise・animators swing_3d_in・swing_3d_out — 借りる先：Animate.css wobble（v3.7.2 は MIT、名前が同じ）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Wobble 3**（動き）：文字のくり返し。名前から：震え・揺れ動く（振り子）（中国語名 颤抖 III）。既定 0.5 秒 — いま：ふるまい noise・animators swing_3d_in・swing_3d_out — 借りる先：Animate.css wobble（v3.7.2 は MIT、名前が同じ）／Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **Wobbly Projector 1**（動き）：文字のくり返し。名前から：震え・影・波打つ（中国語名 投影颤抖）。既定 0.5 秒 — いま：ふるまい noise・ISF Echo Trace をキー — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／anime.js stagger（MIT）
- [ ] **墨水漂浮**（動き・有料）：文字のくり返し。名前から：浮かぶ・墨。既定 0.5 秒 — いま：animators slide_in・slide_out・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **声波震动**（動き・有料）：文字のくり返し。名前から：振動・波。既定 2.0 秒 — いま：ふるまい noise・animators（文字ごとにずらす） — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）／anime.js stagger（MIT）
- [ ] **弹跳拉伸**（動き・有料）：文字のくり返し。名前から：弾む・伸ばす。既定 0.5 秒 — いま：animators bounce_in・spring_in・キー scale_x・skew — 借りる先：Animista tracking-in-expand（BSD-2）
- [ ] **影像叠加**（動き・有料）：文字のくり返し。名前から：重ねる・影。既定 0.5 秒 — いま：animators fade_in・fade_out・ISF Echo Trace をキー — 借りる先：見つからない
- [ ] **心跳**（動き・有料）：文字のくり返し。名前から：鼓動。既定 1.5 秒 — いま：ふるまい oscillator を scale に — 借りる先：Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **扭动**（動き・有料）：文字のくり返し。名前から：くねる。既定 0.5 秒 — いま：animators swing_3d_in・swing_3d_out — 借りる先：見つからない
- [ ] **放大缩小**（動き・有料）：文字のくり返し。名前から：拡大・縮小。既定 0.5 秒 — いま：キー scale（区間のイーズ） — 借りる先：Animista scale-in-center（BSD-2）
- [ ] **漂浮**（動き）：文字のくり返し。名前から：浮かぶ。既定 1.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **立体旋转**（動き・有料）：文字のくり返し。名前から：回転・立体。既定 1.0 秒 — いま：キー rotate・rotate_x・rotate_y・z — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）／Animista tilt-in（BSD-2）
- [ ] **风吹**（動き・有料）：文字のくり返し。名前から：風。既定 0.5 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **飘起**（動き・有料）：文字のくり返し。名前から：漂う。既定 3.0 秒 — いま：animators slide_in・slide_out — 借りる先：見つからない
- [ ] **魔幻晃动**（動き・有料）：文字のくり返し。名前から：揺れ。既定 0.5 秒 — いま：ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）

### 光（15）

- [ ] **Blocks**（光・有料）：文字のくり返し。名前から：縁取り・字・ピンク・パズル（中国語名 描边粉笔字）。既定 2.0 秒 — いま：形・文字の stroke をキー・animators print_on・print_off — 借りる先：見つからない
- [ ] **Converge**（光・有料）：文字のくり返し。名前から：光る・ぼかし（中国語名 发光模糊）。既定 0.8 秒 — いま：物の glow をキー・物の blur をキー — 借りる先：Animista slide-in-blurred・text-focus-in（BSD-2）
- [ ] **Flash**（光・有料）：文字のくり返し。名前から：点滅・光る（フラッシュ）（中国語名 闪烁）。既定 0.5 秒 — いま：ふるまい random を opacity に・段の exposure をキー — 借りる先：Animista flicker-in-1（BSD-2）／Animista flicker-in-glow（BSD-2）
- [ ] **叠影闪烁**（光・有料）：文字のくり返し。名前から：点滅・重なる像。既定 1.0 秒 — いま：ふるまい random を opacity に・ISF Echo Trace をキー — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **字符闪烁**（光・有料）：文字のくり返し。名前から：点滅・字。既定 0.5 秒 — いま：ふるまい random を opacity に・animators print_on・print_off — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **彩色闪烁**（光・有料）：文字のくり返し。名前から：点滅・カラー。既定 0.5 秒 — いま：ふるまい random を opacity に・ISF Multi Hue Shift をキー — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **心跳频闪**（光）：文字のくり返し。名前から：ストロボ・鼓動。既定 0.5 秒 — いま：ふるまい random を opacity に・ふるまい oscillator を scale に — 借りる先：Animista flicker-in-1（BSD-2）／Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **泛光 I**（光・有料）：文字のくり返し。名前から：光のにじみ。既定 0.5 秒 — いま：物の glow をキー — 借りる先：見つからない
- [ ] **漂浮发光**（光）：文字のくり返し。名前から：光る・浮かぶ。既定 0.5 秒 — いま：物の glow をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **爆闪**（光・有料）：文字のくり返し。名前から：強く光る。既定 0.5 秒 — いま：段の exposure をキー — 借りる先：Animista flicker-in-glow（BSD-2）
- [ ] **脉冲闪动**（光・有料）：文字のくり返し。名前から：点滅・脈打つ。既定 0.5 秒 — いま：ふるまい random を opacity に・ふるまい oscillator を scale に — 借りる先：Animista flicker-in-1（BSD-2）／Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **闪动脉冲**（光）：文字のくり返し。名前から：点滅・脈打つ。既定 0.5 秒 — いま：ふるまい random を opacity に・ふるまい oscillator を scale に — 借りる先：Animista flicker-in-1（BSD-2）／Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **频闪渐显渐隐**（光）：文字のくり返し。名前から：ストロボ・じわっと出る・じわっと消える。既定 0.5 秒 — いま：ふるまい random を opacity に・animators fade_in・fade_out — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **上下流光**（光・有料）：文字のくり返し。名前から：流れる光。既定 3.0 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **流光**（光・有料）：文字のくり返し。名前から：流れる光。既定 1.6 秒 — いま：無い — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー

### レンズ・ゆがみ（10）

- [ ] **Pixel Bounce**（レンズ・ゆがみ）：文字のくり返し。名前から：ピクセル・跳ねる・モザイク・弾む（中国語名 像素跳动）。既定 0.5 秒 — いま：ISF Pixellate をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Super Wavy**（レンズ・ゆがみ・有料）：文字のくり返し。名前から：波打つ（中国語名 超强波浪）。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Super Wavy 2**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ（中国語名 超强波浪 II）。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Wave**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ（中国語名 波浪）。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Wave 2**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ（中国語名 波浪 II）。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Wave 3**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ（中国語名 波浪 III）。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Wobbly Projector 2**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **Wobbly Projector 3**（レンズ・ゆがみ）：文字のくり返し。名前から：波打つ。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **循环波浪**（レンズ・ゆがみ・有料）：文字のくり返し。名前から：波打つ。既定 0.5 秒 — いま：animators（文字ごとにずらす） — 借りる先：anime.js stagger（MIT）
- [ ] **鱼眼漂浮**（レンズ・ゆがみ・有料）：文字のくり返し。名前から：魚眼・浮かぶ。既定 0.5 秒 — いま：ISF Bump Distortion をキー・animators slide_in・slide_out — 借りる先：見つからない

### 分割（8）

- [ ] **Bullet Screen**（分割）：文字のくり返し。名前から：分割画面・一字ずつ。既定 0.5 秒 — いま：ISF Quad Tile をキー・animators print_on・print_off — 借りる先：見つからない
- [ ] **Ink Print**（分割・有料）：文字のくり返し。名前から：スプレー描き・溶ける（中国語名 喷绘）。既定 0.5 秒 — いま：ISF Wipe をキー・点群 dissolve（点群だけ） — 借りる先：Animista puff-in-center（BSD-2）
- [ ] **Jump Pieces**（分割・有料）：文字のくり返し。名前から：コラージュ・紙・弾む（中国語名 拼贴纹理贴纸）。既定 0.8 秒 — いま：ISF Quad Tile をキー・animators bounce_in・spring_in — 借りる先：見つからない
- [ ] **Page Turning I**（分割・有料）：文字のくり返し。名前から：ページめくり・回転（中国語名 翻页）。既定 1.2 秒 — いま：ISF Inverted Page Curl をキー・キー rotate — 借りる先：Animista rotate-in-center・roll-in-left（BSD-2）
- [ ] **Scrolling**（分割）：文字のくり返し。名前から：刷毛・滑る・動く（中国語名 刷屏）。既定 0.5 秒 — いま：ISF Wipe をキー・animators slide_in・slide_out — 借りる先：見つからない
- [ ] **Wiper**（分割）：文字のくり返し。名前から：拭き取る。既定 0.5 秒 — いま：ISF Wipe をキー — 借りる先：見つからない
- [ ] **Donut**（分割）：文字のくり返し。名前から：輪・丸（中国語名 甜甜圈）。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions circleopen（MIT）をキー
- [ ] **Fold**（分割）：文字のくり返し。名前から：折る。既定 0.5 秒 — いま：無い — 借りる先：gl-transitions Fold（MIT、名前が同じ）／Animista flip-in（BSD-2）

### グリッチ（7）

- [ ] **Color Glitch**（グリッチ）：文字のくり返し。名前から：色ずれ・グリッチ・虹・色（中国語名 色差故障）。既定 0.5 秒 — いま：rgbshift.wgsl をキー・ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障扫屏**（グリッチ・有料）：文字のくり返し。名前から：グリッチ。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：見つからない
- [ ] **故障抖动**（グリッチ・有料）：文字のくり返し。名前から：グリッチ・揺れ。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい noise — 借りる先：Animista shake-horizontal・vibrate-1（BSD-2）
- [ ] **故障闪动**（グリッチ）：文字のくり返し。名前から：グリッチ・点滅。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・ふるまい random を opacity に — 借りる先：Animista flicker-in-1（BSD-2）
- [ ] **流光故障**（グリッチ・有料）：文字のくり返し。名前から：グリッチ・流れる光。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー — 借りる先：ISF Linear Gradient（MIT）を足し算の重ねで動かす をキー
- [ ] **迷幻故障**（グリッチ・有料）：文字のくり返し。名前から：グリッチ・サイケ。既定 0.5 秒 — いま：ISF Bad TV・rgbshift をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **动态抽帧**（グリッチ・有料）：文字のくり返し。名前から：コマ落とし。既定 0.8 秒 — いま：無い — 借りる先：ISF Freeze Frame・Random Freeze（MIT） をキー

### 色（4）

- [ ] **Space Type**（色・有料）：文字のくり返し。名前から：青・一字ずつ・立体（中国語名 蓝黄镂空手写）。既定 2.0 秒 — いま：ISF Multi Hue Shift をキー・animators print_on・print_off — 借りる先：Animista tilt-in（BSD-2）
- [ ] **彩色切换**（色・有料）：文字のくり返し。名前から：カラー。既定 1.5 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **彩色火焰**（色・有料）：文字のくり返し。名前から：カラー・炎。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー
- [ ] **缤纷弹跳**（色・有料）：文字のくり返し。名前から：カラフル・弾む。既定 0.5 秒 — いま：ISF Multi Hue Shift をキー・animators bounce_in・spring_in — 借りる先：見つからない

### レトロ（3）

- [ ] **Retro B&W**（レトロ・有料）：文字のくり返し。名前から：レトロ・白黒・落書き・古いフィルム（中国語名 复古黑白涂鸦）。既定 1.0 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー・ISF Color Monochrome をキー — 借りる先：ISF Doodler（MIT） をキー
- [ ] **VHS**（レトロ・有料）：文字のくり返し。名前から：VHS・青・古いビデオ・テレビ（中国語名 蓝白VHS）。既定 2.0 秒 — いま：crt_finish.wgsl をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない
- [ ] **复古扭动**（レトロ・有料）：文字のくり返し。名前から：レトロ・くねる。既定 0.5 秒 — いま：film.wgsl・grain.wgsl・sepia.wgsl・technicolor.wgsl をキー・animators swing_3d_in・swing_3d_out — 借りる先：見つからない

### 文字（2）

- [ ] **Font Reel**（文字・有料）：文字のくり返し。名前から：字・一字ずつ（中国語名 字体变换）。既定 0.8 秒 — いま：animators print_on・print_off — 借りる先：見つからない
- [ ] **诡异文字**（文字・有料）：文字のくり返し。名前から：文字。既定 0.5 秒 — いま：animators print_on・print_off — 借りる先：見つからない

### 残像（2）

- [ ] **Trailing Beating**（残像・有料）：文字のくり返し。名前から：尾を引く・跳ねる・残像・分身（中国語名 拖尾跳动）。既定 0.5 秒 — いま：ISF Echo Trace をキー・animators bounce_in・spring_in — 借りる先：Animista heartbeat・pulsate-fwd（BSD-2）
- [ ] **渐变拖尾**（残像・有料）：文字のくり返し。名前から：尾を引く・グラデーション。既定 0.8 秒 — いま：ISF Echo Trace をキー・ISF Multi Hue Shift をキー — 借りる先：見つからない

### 絵の具（2）

- [ ] **涂鸦手绘**（絵の具・有料）：文字のくり返し。名前から：手描き・落書き。既定 0.5 秒 — いま：無い — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3） をキー／ISF Doodler（MIT） をキー
- [ ] **涂鸦手绘 II**（絵の具・有料）：文字のくり返し。名前から：手描き・落書き。既定 0.5 秒 — いま：無い — 借りる先：ISF Sketch（MIT）・GPUImage3 Sketch（BSD-3） をキー／ISF Doodler（MIT） をキー

### 粒・飾り（1）

- [ ] **Flare**（粒・飾り・有料）：文字のくり返し。名前から：炎・レンズの光（中国語名 火焰）。既定 0.5 秒 — いま：段の lens_flare・ISF Lens Flare をキー — 借りる先：Kenney Particle Pack の炎（CC0）＋物の copies・scatter をキー

### その他（5）

- [ ] **Emotional Loading**（その他）：文字のくり返し。名前だけでは中身が分からない。既定 0.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **多重延伸-循环**（その他・有料）：文字のくり返し。名前だけでは中身が分からない。既定 1.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **排队入场**（その他・有料）：文字のくり返し。名前だけでは中身が分からない。既定 1.0 秒 — いま：無い — 借りる先：見つからない
- [ ] **活力舞台**（その他・有料）：文字のくり返し。名前だけでは中身が分からない。既定 0.5 秒 — いま：無い — 借りる先：見つからない
- [ ] **纵向加深-循环**（その他・有料）：文字のくり返し。名前だけでは中身が分からない。既定 1.0 秒 — いま：無い — 借りる先：見つからない

## Filters（フィルター）（454）

Motolii に有る 0・近い物あり 331・見つからない 123。

### 暖色・寒色（65）

- [ ] **Blue Hour**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Candlelight**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Cinematic Dusk**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Crisp Air**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Cyan Nature**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色・鮮やか。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **Dark Gold**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・暗い。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **Fall Foliage**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Glowing Tan**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・やわらか — いま：段の temperature・ISF White Point Adjust・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Lover Blue**（暖色・寒色）：色の味付け（LUT）。名前から：寒色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Moody Fall**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Mulled Wine**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Purple Sunset**（暖色・寒色）：色の味付け（LUT）。名前から：暖色・色かぶり — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Sea Blue**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Summer Shine**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Sun-drenched**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Sunlight**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Sunsoaked**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **Tangerine Dream**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・やわらか。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Twilight Star**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **丹宁**（暖色・寒色・有料）：色の味付け（LUT）。名前から：デニム色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **亮古铜**（暖色・寒色・有料）：色の味付け（LUT）。名前から：ブロンズ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **健康古铜**（暖色・寒色・有料）：色の味付け（LUT）。名前から：ブロンズ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **冷蓝**（暖色・寒色）：色の味付け（LUT）。名前から：寒色・青。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **冷调**（暖色・寒色）：色の味付け（LUT）。名前から：寒色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **冷都**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **冷静**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **加州阳光**（暖色・寒色・有料）：色の味付け（LUT）。名前から：日の光 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **古铜色**（暖色・寒色・有料）：色の味付け（LUT）。名前から：ブロンズ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **可可**（暖色・寒色・有料）：色の味付け（LUT）。名前から：ココア色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **墨蓝**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **幽蓝**（暖色・寒色）：色の味付け（LUT）。名前から：青 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **春日暖黄**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・黄・春。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **暖橙街影**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・オレンジ。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **暖调时光（顶楼咖啡时间）**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・光。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の bloom — 借りる先：見つからない
- [ ] **暖阳**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **暖黄**（暖色・寒色）：色の味付け（LUT）。名前から：暖色・黄 — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **暗夜暖金**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・金色・夜・暗い。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **橘子干**（暖色・寒色・有料）：色の味付け（LUT）。名前から：みかん色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **橘子海**（暖色・寒色・有料）：色の味付け（LUT）。名前から：海・みかん色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **橙蓝**（暖色・寒色）：色の味付け（LUT）。名前から：青・オレンジ。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **海洋之眼**（暖色・寒色・有料）：色の味付け（LUT）。名前から：海。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **海边度假**（暖色・寒色・有料）：色の味付け（LUT）。名前から：海。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **深沙滩色**（暖色・寒色）：色の味付け（LUT）。名前から：砂浜 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **深蜜糖色**（暖色・寒色・有料）：色の味付け（LUT）。名前から：はちみつ色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **爱在日落黄昏时**（暖色・寒色・有料）：色の味付け（LUT）。名前から：日没・たそがれ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **牛皮纸**（暖色・寒色）：色の味付け（LUT）。名前から：クラフト紙 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **琥珀**（暖色・寒色・有料）：色の味付け（LUT）。名前から：琥珀色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **红与蓝**（暖色・寒色）：色の味付け（LUT）。名前から：青・赤 — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **肉桂**（暖色・寒色・有料）：色の味付け（LUT）。名前から：シナモン色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **艳丽暖肤**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・肌・鮮やか。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **花海暖阳**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・海。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **苍橘**（暖色・寒色・有料）：色の味付け（LUT）。名前から：みかん色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **蓝珀光影**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青・光。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の bloom — 借りる先：見つからない
- [ ] **蓝紫锈影**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青・紫。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **蓝雾**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青・霧。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **蓝魔**（暖色・寒色）：色の味付け（LUT）。名前から：青 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **蜜桃**（暖色・寒色）：色の味付け（LUT）。名前から：はちみつ色 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **西冷**（暖色・寒色・有料）：色の味付け（LUT）。名前から：寒色。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **轻暖鹅黄**（暖色・寒色・有料）：色の味付け（LUT）。名前から：暖色・黄。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **酚蓝**（暖色・寒色）：色の味付け（LUT）。名前から：青 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **银蓝**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **阳光色**（暖色・寒色・有料）：色の味付け（LUT）。名前から：日の光 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **青橙**（暖色・寒色）：色の味付け（LUT）。名前から：オレンジ・青緑 — いま：段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **青红**（暖色・寒色・有料）：色の味付け（LUT）。名前から：青緑・赤。つまみ：色の濃さ — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **青黄**（暖色・寒色）：色の味付け（LUT）。名前から：青緑・黄 — いま：段の temperature・ISF White Point Adjust・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない

### 色かぶり（63）

- [ ] **Cozy Xmas**（色かぶり・有料）：色の味付け（LUT）。名前から：クリスマス — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Peach Fuzz**（色かぶり）：色の味付け（LUT）。名前から：色かぶり。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Peach Fuzz 2**（色かぶり・有料）：色の味付け（LUT）。名前から：色かぶり — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Pink World**（色かぶり・有料）：色の味付け（LUT）。名前から：色かぶり — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Platinum Bell**（色かぶり・有料）：色の味付け（LUT）。名前から：クリスマス — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Reindeer**（色かぶり）：色の味付け（LUT）。名前から：クリスマス — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Somber Green**（色かぶり・有料）：色の味付け（LUT）。名前から：色かぶり。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Wreath**（色かぶり・有料）：色の味付け（LUT）。名前から：クリスマス — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **墨绿对比**（色かぶり・有料）：色の味付け（LUT）。名前から：緑。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **奶绿**（色かぶり）：色の味付け（LUT）。名前から：緑・ミルク — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **姜饼红**（色かぶり）：色の味付け（LUT）。名前から：赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **寂静岭（棕）**（色かぶり・有料）：色の味付け（LUT）。名前から：茶色。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **旧金山**（色かぶり）：色の味付け（LUT）。名前から：金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **暗棕**（色かぶり）：色の味付け（LUT）。名前から：茶色・暗い — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗紫**（色かぶり・有料）：色の味付け（LUT）。名前から：紫・暗い — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗黑王**（色かぶり・有料）：色の味付け（LUT）。名前から：黒・暗い。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **松果棕**（色かぶり）：色の味付け（LUT）。名前から：茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **棕榈世界**（色かぶり・有料）：色の味付け（LUT）。名前から：茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **棕石**（色かぶり・有料）：色の味付け（LUT）。名前から：茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **流光金属**（色かぶり）：色の味付け（LUT）。名前から：金色・光 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の bloom — 借りる先：見つからない
- [ ] **润粉皮**（色かぶり・有料）：色の味付け（LUT）。名前から：ピンク — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **深褐**（色かぶり）：色の味付け（LUT）。名前から：茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **清晖绿野**（色かぶり・有料）：色の味付け（LUT）。名前から：緑。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **清透净白**（色かぶり・有料）：色の味付け（LUT）。名前から：白。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **灿金彩带**（色かぶり・有料）：色の味付け（LUT）。名前から：金色。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **熔金**（色かぶり）：色の味付け（LUT）。名前から：金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **生化危机（红）**（色かぶり・有料）：色の味付け（LUT）。名前から：赤。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **白茶**（色かぶり）：色の味付け（LUT）。名前から：白 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **白莲花浓Rich white lotus**（色かぶり・有料）：色の味付け（LUT）。名前から：白・濃い。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **白莲花淡Pale white lotus**（色かぶり・有料）：色の味付け（LUT）。名前から：白。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **砂金**（色かぶり）：色の味付け（LUT）。名前から：金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **紫红**（色かぶり）：色の味付け（LUT）。名前から：紫・赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **紫罗兰之梦**（色かぶり・有料）：色の味付け（LUT）。名前から：紫・夢。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **红棕**（色かぶり）：色の味付け（LUT）。名前から：赤・茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **红色沙漠**（色かぶり・有料）：色の味付け（LUT）。名前から：赤。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **红魔**（色かぶり・有料）：色の味付け（LUT）。名前から：赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **绝对红**（色かぶり）：色の味付け（LUT）。名前から：赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **绿光监控**（色かぶり・有料）：色の味付け（LUT）。名前から：緑・光。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の bloom — 借りる先：見つからない
- [ ] **绿幽精灵**（色かぶり・有料）：色の味付け（LUT）。名前から：緑。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **绿洲**（色かぶり・有料）：色の味付け（LUT）。名前から：緑 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **绿湖**（色かぶり）：色の味付け（LUT）。名前から：緑。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **茶红**（色かぶり・有料）：色の味付け（LUT）。名前から：赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **荆红**（色かぶり・有料）：色の味付け（LUT）。名前から：赤 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **落叶棕**（色かぶり・有料）：色の味付け（LUT）。名前から：茶色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **蒸汽波**（色かぶり）：色の味付け（LUT）。名前から：ヴェイパーウェイヴ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **贝松绿**（色かぶり）：色の味付け（LUT）。名前から：緑 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **金200**（色かぶり）：色の味付け（LUT）。名前から：金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **金属**（色かぶり）：色の味付け（LUT）。名前から：金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **金粉飘落**（色かぶり・有料）：色の味付け（LUT）。名前から：ピンク・金色。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **鎏金旧笺**（色かぶり・有料）：色の味付け（LUT）。名前から：金色。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **镜粉**（色かぶり・有料）：色の味付け（LUT）。名前から：ピンク — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **飘雪圣诞**（色かぶり・有料）：色の味付け（LUT）。名前から：クリスマス。つまみ：色の濃さ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **香芋粉**（色かぶり・有料）：色の味付け（LUT）。名前から：ピンク — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黄石**（色かぶり・有料）：色の味付け（LUT）。名前から：黄 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黄紫**（色かぶり・有料）：色の味付け（LUT）。名前から：紫・黄 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黄金时刻**（色かぶり・有料）：色の味付け（LUT）。名前から：黄・金色 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黄金海岸**（色かぶり・有料）：色の味付け（LUT）。名前から：黄・金色・海 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **黄铜**（色かぶり）：色の味付け（LUT）。名前から：黄・ブロンズ — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **黑冰**（色かぶり・有料）：色の味付け（LUT）。名前から：黒 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黑森林**（色かぶり・有料）：色の味付け（LUT）。名前から：黒 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黑胶**（色かぶり・有料）：色の味付け（LUT）。名前から：黒 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黑豹**（色かぶり・有料）：色の味付け（LUT）。名前から：黒 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **黑黄**（色かぶり・有料）：色の味付け（LUT）。名前から：黄・黒 — いま：ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない

### 鮮やか（58）

- [ ] **4K画质**（鮮やか・有料）：色の味付け（LUT）。名前から：画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **8K画质**（鮮やか・有料）：色の味付け（LUT）。名前から：画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **Enhance**（鮮やか・有料）：色の味付け（LUT）。名前から：鮮やか — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **HD Dark**（鮮やか・有料）：色の味付け（LUT）。名前から：鮮やか・暗い。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **Tropical Velvet**（鮮やか）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **Vivid**（鮮やか・有料）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **Vivid 2**（鮮やか・有料）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **Wild Venture**（鮮やか・有料）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **净透**（鮮やか）：色の味付け（LUT）。名前から：澄んだ。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **北欧厨房**（鮮やか・有料）：色の味付け（LUT）。名前から：台所。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **增色**（鮮やか・有料）：色の味付け（LUT）。名前から：色を足す。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **夏天的故事**（鮮やか・有料）：色の味付け（LUT）。名前から：夏 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **夏威夷**（鮮やか・有料）：色の味付け（LUT）。名前から：夏 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **微暗清晰**（鮮やか・有料）：色の味付け（LUT）。名前から：くっきり・暗い。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **悠长夏日**（鮮やか・有料）：色の味付け（LUT）。名前から：夏。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **户外饱和（加州嬉皮士）**（鮮やか・有料）：色の味付け（LUT）。名前から：彩度。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **提亮**（鮮やか・有料）：色の味付け（LUT）。名前から：明るく。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **料理**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **日食**（鮮やか）：色の味付け（LUT）。名前から：食べ物 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **明媚初夏**（鮮やか・有料）：色の味付け（LUT）。名前から：夏。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **晴天野餐**（鮮やか・有料）：色の味付け（LUT）。名前から：ピクニック・晴れ。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **晴好假日**（鮮やか・有料）：色の味付け（LUT）。名前から：晴れ。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **晴朗**（鮮やか・有料）：色の味付け（LUT）。名前から：晴れ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **晴空**（鮮やか）：色の味付け（LUT）。名前から：晴れ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **晴空 ll**（鮮やか）：色の味付け（LUT）。名前から：晴れ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **暗光提亮**（鮮やか・有料）：色の味付け（LUT）。名前から：明るく・暗い・光 — いま：段の saturation・vibrance・ISF Vibrance・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **法餐**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **浓橘胶粒**（鮮やか・有料）：色の味付け（LUT）。名前から：濃い・みかん色。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **浓烈**（鮮やか・有料）：色の味付け（LUT）。名前から：濃い — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **浓郁彩色**（鮮やか・有料）：色の味付け（LUT）。名前から：濃い。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **清凉夏意**（鮮やか・有料）：色の味付け（LUT）。名前から：夏 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **清晰**（鮮やか・有料）：色の味付け（LUT）。名前から：くっきり — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **清晰冷调**（鮮やか・有料）：色の味付け（LUT）。名前から：くっきり・寒色 — いま：段の saturation・vibrance・ISF Vibrance・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **清晰去灰**（鮮やか・有料）：色の味付け（LUT）。名前から：くっきり・グレー。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **炎夏假日**（鮮やか・有料）：色の味付け（LUT）。名前から：夏。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **烘焙**（鮮やか）：色の味付け（LUT）。名前から：パン・菓子 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **田园野餐**（鮮やか・有料）：色の味付け（LUT）。名前から：ピクニック。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **画质修复**（鮮やか・有料）：色の味付け（LUT）。名前から：画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **秋天**（鮮やか）：色の味付け（LUT）。名前から：秋 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **秋枫**（鮮やか・有料）：色の味付け（LUT）。名前から：秋 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **简餐**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **美食1**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **美食2**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **自然**（鮮やか）：色の味付け（LUT）。名前から：自然 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **自然褪色1**（鮮やか・有料）：色の味付け（LUT）。名前から：自然・色あせ。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **自然褪色2**（鮮やか・有料）：色の味付け（LUT）。名前から：自然・色あせ。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **西餐**（鮮やか）：色の味付け（LUT）。名前から：料理 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **野餐**（鮮やか・有料）：色の味付け（LUT）。名前から：ピクニック — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **风景提亮（北欧列车）**（鮮やか・有料）：色の味付け（LUT）。名前から：風景・明るく。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **食色**（鮮やか・有料）：色の味付け（LUT）。名前から：食べ物。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **高清增质**（鮮やか・有料）：色の味付け（LUT）。名前から：高画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **高清影视荧幕**（鮮やか・有料）：色の味付け（LUT）。名前から：高画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **高清暖调**（鮮やか・有料）：色の味付け（LUT）。名前から：高画質・暖色。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **高清美食**（鮮やか・有料）：色の味付け（LUT）。名前から：料理・高画質。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **高饱和**（鮮やか・有料）：色の味付け（LUT）。名前から：彩度 — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **高饱和战士**（鮮やか）：色の味付け（LUT）。名前から：彩度。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **鲜艳 I**（鮮やか）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **鲜艳 II**（鮮やか）：色の味付け（LUT）。名前から：鮮やか。つまみ：色の濃さ — いま：段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない

### フィルム・レトロ（54）

- [ ] **1980**（フィルム・レトロ）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **Cool CCD**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **cowboy retro**（フィルム・レトロ）：色の味付け（LUT）。名前から：フィルム・レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **Flash CCD**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **ins古早 2**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：昔風 — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **Lo-Fi**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：懐かしい — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：CSSgram（Instagram の名前） Lo-Fi（MIT、名前が同じ）
- [ ] **Nostalgic Negative**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **Throwback Vintage**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **VHS I**（フィルム・レトロ）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **VHS II**（フィルム・レトロ）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **VHS III**（フィルム・レトロ）：色の味付け（LUT）。名前から：フィルム・レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **Vintage Pink**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ・色かぶり。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **Yellowed Memories**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：懐かしい。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **中性褪色**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：色あせ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **低调褪色**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：色あせ・ローキー。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **假日复古**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **古早**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：昔風 — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **古早光感**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：昔風・光。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の bloom — 借りる先：見つからない
- [ ] **古早微曝**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：昔風。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **古早棕调**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：昔風・茶色。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **复古1**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古1968**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古1978**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古1988**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古1998**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古2**（フィルム・レトロ）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古3**（フィルム・レトロ）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古4**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古90S**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古吧台**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古暖潮**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ・暖色。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **复古橙红**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ・オレンジ・赤。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **复古流金**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ・金色。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **复古纽约**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **复古肉桂**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ・シナモン色。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **复古胶片**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **夏日怀旧**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：懐かしい・夏。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の saturation・vibrance・ISF Vibrance — 借りる先：見つからない
- [ ] **富士胶片**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・富士フイルム。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **怀旧**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：懐かしい — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **暖调褪色**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：色あせ・暖色 — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **暗调复古**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ・暗い。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗调电影**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：映画・暗い。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **未来感胶片**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **电影光效**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：映画・光。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の bloom — 借りる先：見つからない
- [ ] **经典电影**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：映画。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **老友记**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：古い — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **蓝绿胶片**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：フィルム・青・緑。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **褪色黄调**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：色あせ・黄。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **负片**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：ネガ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **质感**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：質感 — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **质感压暗**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：質感・暗い。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **质感对比**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：質感。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **轻微复古**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：レトロ。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **香槟色老钱风**（フィルム・レトロ・有料）：色の味付け（LUT）。名前から：古い — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない

### 暗い・夜（27）

- [ ] **Cyber Shot**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **Cyber Shot 2**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **Cyberpunk**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **Dark Humble**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **东京夜场**（暗い・夜・有料）：色の味付け（LUT）。名前から：夜。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **低调**（暗い・夜・有料）：色の味付け（LUT）。名前から：ローキー — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **圣善夜**（暗い・夜・有料）：色の味付け（LUT）。名前から：夜 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **夜视仪**（暗い・夜）：色の味付け（LUT）。名前から：夜 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **微暗**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗夜**（暗い・夜・有料）：色の味付け（LUT）。名前から：夜・暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗夜发光**（暗い・夜・有料）：色の味付け（LUT）。名前から：夜・暗い・光。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels・段の bloom — 借りる先：見つからない
- [ ] **暗灰低调**（暗い・夜・有料）：色の味付け（LUT）。名前から：ローキー・グレー・暗い。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels・ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **暗调**（暗い・夜）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗调ll**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **暗调发光**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い・光。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels・段の bloom — 借りる先：見つからない
- [ ] **暗银**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **月夜**（暗い・夜）：色の味付け（LUT）。名前から：夜 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **炭烧**（暗い・夜・有料）：色の味付け（LUT）。名前から：炭色 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **硬朗**（暗い・夜）：色の味付け（LUT）。名前から：硬調 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **粉蔼霓虹**（暗い・夜・有料）：色の味付け（LUT）。名前から：ネオン・ピンク。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **赛博2077**（暗い・夜・有料）：色の味付け（LUT）。名前から：サイバー — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **赛博朋克**（暗い・夜）：色の味付け（LUT）。名前から：サイバー — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **赛博蓝**（暗い・夜・有料）：色の味付け（LUT）。名前から：サイバー・青。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **霓虹 I**（暗い・夜・有料）：色の味付け（LUT）。名前から：ネオン — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **霓虹 II**（暗い・夜・有料）：色の味付け（LUT）。名前から：ネオン — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **高级暗调**（暗い・夜・有料）：色の味付け（LUT）。名前から：暗い。つまみ：色の濃さ — いま：段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **黑金**（暗い・夜・有料）：色の味付け（LUT）。名前から：黒金 — いま：段の LGG・ISF Color Levels — 借りる先：見つからない

### 映画の色（20）

- [ ] **Anora**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **Deadpool**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **Dune 1**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **Dune 2**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **GTA I**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **GTA II**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **GTA III**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **Poor Things**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **Speakeasy**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **Wong Kar-wai**（映画の色・有料）：色の味付け（LUT）。名前から：映画・ゲームの色 — いま：無い — 借りる先：見つからない
- [ ] **奥本海默**（映画の色・有料）：色の味付け（LUT）。名前から：映画『オッペンハイマー』風。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **好莱坞1**（映画の色・有料）：色の味付け（LUT）。名前から：ハリウッド映画風 — いま：無い — 借りる先：見つからない
- [ ] **好莱坞2**（映画の色・有料）：色の味付け（LUT）。名前から：ハリウッド映画風 — いま：無い — 借りる先：見つからない
- [ ] **布达佩斯**（映画の色）：色の味付け（LUT）。名前から：映画『グランド・ブダペスト・ホテル』風 — いま：無い — 借りる先：見つからない
- [ ] **敦刻尔克**（映画の色）：色の味付け（LUT）。名前から：映画『ダンケルク』風 — いま：無い — 借りる先：見つからない
- [ ] **断背山**（映画の色・有料）：色の味付け（LUT）。名前から：映画『ブロークバック・マウンテン』風 — いま：無い — 借りる先：見つからない
- [ ] **月升王国**（映画の色・有料）：色の味付け（LUT）。名前から：映画『ムーンライズ・キングダム』風 — いま：無い — 借りる先：見つからない
- [ ] **电视广告片**（映画の色・有料）：色の味付け（LUT）。名前から：テレビ CM 風 — いま：無い — 借りる先：見つからない
- [ ] **芭比**（映画の色・有料）：色の味付け（LUT）。名前から：映画『バービー』風 — いま：無い — 借りる先：見つからない
- [ ] **雷诺阿**（映画の色・有料）：色の味付け（LUT）。名前から：ルノワールの絵風 — いま：無い — 借りる先：見つからない

### 白黒（19）

- [ ] **Anime B&W**（白黒・有料）：色の味付け（LUT）。名前から：白黒。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **BW 2**（白黒）：色の味付け（LUT）。名前から：白黒 — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **BW 3**（白黒）：色の味付け（LUT）。名前から：白黒 — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **Gritty Noir**（白黒・有料）：色の味付け（LUT）。名前から：白黒・暗い。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・段の LGG・ISF Color Levels — 借りる先：見つからない
- [ ] **亮白黑白**（白黒・有料）：色の味付け（LUT）。名前から：白黒・白。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **去灰**（白黒・有料）：色の味付け（LUT）。名前から：グレー。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **报纸**（白黒・有料）：色の味付け（LUT）。名前から：新聞 — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **智性灰**（白黒・有料）：色の味付け（LUT）。名前から：グレー。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **灰调**（白黒）：色の味付け（LUT）。名前から：グレー — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **灰野暖红**（白黒・有料）：色の味付け（LUT）。名前から：グレー・暖色・赤。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **美式灰粉**（白黒・有料）：色の味付け（LUT）。名前から：グレー・ピンク。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・ISF Duotone・Trio Tone・段の影・中間・明るい所の色 — 借りる先：見つからない
- [ ] **蓝灰**（白黒・有料）：色の味付け（LUT）。名前から：グレー・青 — いま：ISF Color Monochrome（白黒）・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **褪色黑白**（白黒・有料）：色の味付け（LUT）。名前から：白黒・色あせ。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **轻冷灰调**（白黒・有料）：色の味付け（LUT）。名前から：グレー・寒色。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **青灰**（白黒）：色の味付け（LUT）。名前から：グレー・青緑。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **高级灰**（白黒・有料）：色の味付け（LUT）。名前から：グレー。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない
- [ ] **黑白幽光**（白黒・有料）：色の味付け（LUT）。名前から：白黒・光。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・段の bloom — 借りる先：見つからない
- [ ] **黑白负片**（白黒・有料）：色の味付け（LUT）。名前から：白黒・ネガ。つまみ：色の濃さ — いま：ISF Color Monochrome（白黒）・film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：見つからない
- [ ] **默片**（白黒）：色の味付け（LUT）。名前から：無声映画 — いま：ISF Color Monochrome（白黒） — 借りる先：見つからない

### やわらか（19）

- [ ] **Center Light**（やわらか・有料）：色の味付け（LUT）。名前から：やわらか — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Daisies Glow**（やわらか）：色の味付け（LUT）。名前から：やわらか。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Dreamy Halo**（やわらか・有料）：色の味付け（LUT）。名前から：やわらか。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Lucid Plus**（やわらか・有料）：色の味付け（LUT）。名前から：やわらか。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **Soft Light**（やわらか・有料）：色の味付け（LUT）。名前から：やわらか — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **去雾**（やわらか・有料）：色の味付け（LUT）。名前から：霧。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **好梦**（やわらか）：色の味付け（LUT）。名前から：夢。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **小麦肤感**（やわらか・有料）：色の味付け（LUT）。名前から：肌。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **少年梦**（やわらか・有料）：色の味付け（LUT）。名前から：夢 — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **淡彩**（やわらか・有料）：色の味付け（LUT）。名前から：淡い色。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **温柔**（やわらか）：色の味付け（LUT）。名前から：やわらか — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **焕肤**（やわらか・有料）：色の味付け（LUT）。名前から：肌。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **盗梦空间**（やわらか・有料）：色の味付け（LUT）。名前から：夢 — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **磨砂小麦**（やわらか・有料）：色の味付け（LUT）。名前から：マット — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **磨砂小麦II**（やわらか・有料）：色の味付け（LUT）。名前から：マット — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **磨砂肌**（やわらか）：色の味付け（LUT）。名前から：マット — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **迷雾**（やわらか・有料）：色の味付け（LUT）。名前から：霧。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **阴天**（やわらか・有料）：色の味付け（LUT）。名前から：くもり — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない
- [ ] **阴天拯救**（やわらか・有料）：色の味付け（LUT）。名前から：くもり。つまみ：色の濃さ — いま：段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない

### 光（12）

- [ ] **光辉**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **发光橘子**（光・有料）：色の味付け（LUT）。名前から：光・みかん色。つまみ：色の濃さ — いま：段の bloom・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **幽影光华**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **微光**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **日光吻**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **日光浴**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **暮光**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **水光肌**（光・有料）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない
- [ ] **水光蜜桃**（光・有料）：色の味付け（LUT）。名前から：光・はちみつ色 — いま：段の bloom・段の temperature・ISF White Point Adjust — 借りる先：見つからない
- [ ] **调亮微光**（光・有料）：色の味付け（LUT）。名前から：光。つまみ：色の濃さ — いま：段の bloom — 借りる先：見つからない
- [ ] **逆光拯救**（光・有料）：色の味付け（LUT）。名前から：光。つまみ：色の濃さ — いま：段の bloom — 借りる先：見つからない
- [ ] **高光肌**（光）：色の味付け（LUT）。名前から：光 — いま：段の bloom — 借りる先：見つからない

### フィルムの銘柄（9）

- [ ] **Nikon胶片**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：ニコン・フィルム。つまみ：色の濃さ — いま：film.wgsl・technicolor・bleach_bypass・sepia — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **哈苏**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：ハッセルブラッド — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **哈苏2**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：ハッセルブラッド — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **富士回忆**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：富士フイルム。つまみ：色の濃さ — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **普罗维亚100**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：フジ プロビア — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **柯达500T**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：コダックのフィルム。つまみ：色の濃さ — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **柯达ektar100**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：エクター・コダックのフィルム。つまみ：色の濃さ — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **柯达gold200**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：コダックのフィルム。つまみ：色の濃さ — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）
- [ ] **爱克发400**（フィルムの銘柄・有料）：色の味付け（LUT）。名前から：アグファのフィルム — いま：無い — 借りる先：フィルムの再現 LUT：RawTherapee Film Simulation Collection（ライセンスは要確認）

### ぼかし（4）

- [ ] **Blur**（ぼかし・有料）：色の味付け（LUT）。名前から：ぼかし — いま：物の blur — 借りる先：見つからない
- [ ] **Focus**（ぼかし・有料）：色の味付け（LUT）。名前から：ぼかし — いま：物の blur — 借りる先：three.js shaders FocusShader（MIT、名前が同じ）
- [ ] **模糊 II**（ぼかし・有料）：色の味付け（LUT）。名前から：ぼかし — いま：物の blur — 借りる先：見つからない
- [ ] **背景虚化**（ぼかし・有料）：色の味付け（LUT）。名前から：背景ぼかし — いま：物の blur — 借りる先：見つからない

### 粒（1）

- [ ] **Soft Grain**（粒）：色の味付け（LUT）。名前から：粒・やわらか。つまみ：色の濃さ — いま：grain.wgsl・段の LGG・contrast を下げる・段の bloom — 借りる先：見つからない

### その他（103）

- [ ] **Badbunny**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Kendall**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Leaf Gaze**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Like Jennie**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **Party Tonight**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **Short n' Sweet**（その他）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **Tyler's Play**（その他）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **上色**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **不要抬头**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **专注**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **佛罗里达**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **佛罗里达乐园**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **假日**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **傍晚**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **冰城**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **十二月**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **南法假日**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **南法午后**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **南瓜**（その他）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **卡萨布兰卡**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **圣塔莫妮卡**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **地下铁**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：強さ — いま：無い — 借りる先：見つからない
- [ ] **城市穿行**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **大热**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **嬉皮士**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **子弹列车**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **巫师**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **希望**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **干枯玫瑰**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **幻境**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **幽灵**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **康尼岛**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **彩虹**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **徒步**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **怦然心动**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **怪诞**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拉斯维加斯**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拐杖糖**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **拽酷**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **摩登**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **撒丁岛**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **日常1**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **日常2**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **日照**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **旧时来信**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **明快街拍**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **普林斯顿**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **暧昧**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **有一天**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **杜乐丽**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **柏林**（その他）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **棒棒糖**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **森山**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **槲寄生**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **比佛利**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **气质**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **江浙沪**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **沿途**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **法棍**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **派遣**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **流行90s**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **温情肯尼亚**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **炙热**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **烈日灼痕**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **焰色**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **照片展位**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **爱之城**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **爱之城2**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **爵士**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **独行侠**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **璀璨**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **病娇**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **盐系**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **空谷**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **罗马假日**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **腰果**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **自由**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **航程**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **花园**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **落日**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **落日派对**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **西西里**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **西雅图**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **谦逊**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **贝果**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **迈阿密**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **迷林**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **邂逅**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **里昂**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **钢铁与汗水**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **铃儿响叮当**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **银质**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **闪耀派对**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **闻香识人**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **随性**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **隧道**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **雪莉：现实的愿景**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **香槟之宴**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない。つまみ：色の濃さ — いま：無い — 借りる先：見つからない
- [ ] **马尔代夫**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **骄傲**（その他）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **高质1**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **高质2**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない
- [ ] **鬼魅**（その他・有料）：色の味付け（LUT）。名前だけでは中身が分からない — いま：無い — 借りる先：見つからない

## 出典

- 一覧：pyCapCut（GuanYixuan、ライセンス表示なし、名前・番号の表だけ使った） https://github.com/GuanYixuan/pyCapCut/tree/main/pycapcut/metadata
- 一覧の英語名：CapCutAPI（sun-guannan、Apache-2.0）の capcut_*_meta.py https://github.com/sun-guannan/CapCutAPI/tree/main/pyJianYingDraft/metadata
- 規約：CapCut Terms of Service（2026-04-15） https://www.capcut.com/clause/terms-of-service
- 規約：CapCut Materials License Agreement https://www.capcut.com/clause/material-license-agreement?lang=en
- 部品表：BytePlus Video Editor SDK、Android の手引き https://docs.byteplus.com/docs/video-editor-sdk/160-getting-started-with-android
- 部品表：BytePlus Video Editor SDK、iOS の手引き https://docs.byteplus.com/en/docs/video-editor-sdk/160-getting-started-with-ios
- 素材一覧：BytePlus Video Editor SDK assets list https://docs.byteplus.com/en/docs/video-editor-sdk/130-video-editor-sdk-assets-list
- 告知：BytePlus Effects の第三者ソフトの告知 https://docs.byteplus.com/en/effects/docs/notices-for-third-party-software-that-may-be-contained-in-portions-of-this-product
- FAQ：BytePlus Effects（社内デザイナー、EffectCreator） https://docs.byteplus.com/docs/effects/docs-faq-sdks
- 同じチーム：BytePlus VE SDK の概要 https://docs.byteplus.com/en/docs/byteplus-video-editor-sdk/docs-one-pager-overview
- Windows 版の中身（他の人の調べ、ライセンス表示なし、読んだだけ） https://github.com/marcosp34bpm/capcut-rea
- 剪映の算法の届け出 https://lf26-cdn-tos.draftstatic.com/obj/ies-hotsoon-draft/vco/dabb71c9-8e8e-4bfa-ae61-500e2d1e9c31.html
- 抖音の自前の描く係（二次情報） https://zhuanlan.zhihu.com/p/395413668
- 前の調べ：research_notes/capcut/en.md・ja_ko.md・zh.md（数英・每日经济新闻・火山引擎・掘金の出典はそちら）
- gl-transitions（MIT、InvertedPageCurl は BSD-3、StereoViewer は BSD-2。125 本の頭のライセンスを全部見た） https://github.com/gl-transitions/gl-transitions
- Vidvox ISF-Files（MIT、手元 research_notes/isf/ISF-Files） https://github.com/Vidvox/ISF-Files
- GPUImage・GPUImage3（BSD-3） https://github.com/BradLarson/GPUImage3
- three.js examples/jsm/shaders（MIT） https://github.com/mrdoob/three.js/tree/dev/examples/jsm/shaders
- Lygia（Prosperity Public License 3.0.0） https://github.com/patriciogonzalezvivo/lygia/blob/main/LICENSE.md
- Animate.css（v4：Hippocratic 2.1、v3.7.2：MIT） https://github.com/animate-css/animate.css/tree/3.7.2
- Animista（FreeBSD ライセンス） https://animista.net/faq
- anime.js（MIT、Motolii に入っている） https://github.com/juliangarnier/anime
- CSSgram（MIT） https://github.com/una/CSSgram
- Kenney Particle Pack（CC0） https://kenney.nl/assets/particle-pack
- Twemoji（絵は CC BY 4.0） https://github.com/jdecked/twemoji
- MediaPipe（Apache-2.0：Image Segmenter・Face Landmarker） https://github.com/google-ai-edge/mediapipe
- RawTherapee Film Simulation Collection（ライセンスは今回確かめられず） https://rawpedia.rawtherapee.com/
- CapCut の効果の棚の分け方（Web 版） https://www.capcut.com/help/recommended-alternative-effects
