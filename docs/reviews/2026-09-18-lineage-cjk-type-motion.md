# Cavalry 製 29 秒の書体モーション動画 — 出自と「動きの遺伝子」の系譜(2026-09-18 取得)

課題: 720×720 / 24fps / 29 秒、漢字の書(楷書・北魏楷書・隷書風)を弧・螺旋・万華鏡・格子・放射・環・トンネル・ページめくりに置き、全カットに Cavalry の選択枠・ノード点・ガイド線が映る動画。作者の特定(問い 1)、素材の裏取り(問い 2、3 行)、**動きの系譜(問い 3、主)**、完成を先に見ていた証拠(問い 4)。一次資料(本人の文・研究者・博物館・公式 docs)だけを当日取得。取れなかった物は「未検証」「見つからず」と書いた。判断と Motolii への含意は書かない。

## 0. 要約(5 行)

1. **作者は特定できず**。Cavalry 公式(Medium「projects we love」2023 が最新、Instagram/X は取得不可)、Behance、Scenery、Vimeo、中華圏 SNS の検索で当該作は出なかった。近い作例は §1 末尾に 3 つ。
2. 素材: 赤い丸の姓氏 = 揮春(赤紙に黒/金字)と字姓燈(紅燈籠に姓)の混成が有力、緑の墨字 = 北魏体は香港招牌・霓虹の主流(M+ NEONSIGNS.HK、陳濬人)、青地白抜き = 小巴牌は**白地に赤/青字**であり一致せず(麥錦生)。いずれも「この動画がそれを引いた」証拠は無い。
3. **動きの遺伝子は 4 系統に分解できる**: (a) 装置 — 万華鏡(Brewster 1817 特許「Forms and Patterns…multiplied symmetrically」)、ゾートロープ(Horner 1834)、キネオグラフ = ページめくり(Linnett 1868)。(b) 抽象映画 — Fischinger「音楽の導きで新しい運動とリズムが生まれた」、McLaren「コマの間で起きる事の方が大事」、Lye「pure figures of motion」、**Whitney「物が差分的に動くと運動は模様になる」(Digital Harmony 1980)** と M-5 砲撃照準器のカム機構(Vertigo の螺旋、Catalog 1961 の文字の放射対称、Lapis の同心環)。
4. (c) 手続き型 — Cinema 4D MoGraph(2006、Per-Anders Edwards、Cloner)→ Mainframe の MASH(Maya、2015 Autodesk 買収、Distribute/Offset/Random/Curve/Falloff の node)→ Cavalry(2019、Waters「proceduralism の力が全ての専門ツールの芯にあるべき」)。Cavalry の Duplicator(Grid/Circle 分布)+ Stagger + Falloff がこの動画の弧・格子・放射・時間差の直接の道具。
5. (d) **Disney の 12 原則は含まれない**: Thomas & Johnston 1981 は登場人物の「生命の錯覚」(squash/stretch、anticipation…)の原則で、Cavalry 公式 docs・創業者の発言・MASH docs のどこにも 12 原則への言及は見つからず。この動画の運動は個体の演技ではなく「群の写像」(同一則を N 個に配る)で、Whitney–MoGraph 系。

## 1. 作者と作品の特定

| 見つけた物 | 一次資料 URL | 確度 |
|---|---|---|
| **特定できず**。Cavalry 公式の作品紹介は Medium「Cavalry projects we love in 2023」(Chris Hardcastle、2023-12)が検索上の最新。2024/2025 版は無い | https://medium.com/cavalry-animation/cavalry-projects-we-love-in-2023-5a48fffe4dfc | 高(無い事の確認) |
| Cavalry Instagram(@cavalry.app、90K)/ X(@cavalry__app)/ Medium(Ian Waters の文)は **403 で本文取得不可**。Threads の @cavalry.app 投稿は検索に出るが当該作は出ず | https://www.instagram.com/cavalry.app/ · https://x.com/cavalry__app · https://www.threads.com/@cavalry.app/post/C8pQ2obNoC1 | 未検証 |
| Scenery(scenery.io、Cavalry の scene 共有)に漢字 scene は検索で出ず | https://scenery.io/ · https://scenery.io/@cavalry | 中 |
| 香港北魏真書 @zansyu(282 投稿)、北魏真書體 FB、justfont、Monotype 香港、Sandy Chan の「Cavalry」投稿は検索で出ず | https://www.instagram.com/zansyu/ · https://www.facebook.com/beiweizansyu/ | 中(SNS 本文は取得不可) |
| 中華圏 SNS(小紅書・微博・bilibili)の「Cavalry 漢字/書法」は一般記事のみ。知乎の Cavalry 評価スレッドは 2020 | https://www.zhihu.com/question/369847222 | 中 |
| Cavalry 2.6(2026-02-25)は Extrude の Combine Meshes 修正を含む → 「念」の押し出し螺旋は 2.x の Extrude で可能(制作時期の下限の目安) | https://cavalry.studio/docs/tech-info/release-notes/2.6/2-6-0-release-notes/ · https://www.kamilk.co.uk/2026/02/extrude-text-layers-in-cavalry/ | 中 |

**近い作例(同じ手法・同じ素材の Cavalry 作)**

| 作例 | 何が近いか | URL | 確度 |
|---|---|---|---|
| 「kinetic typography made with Cavalry Part 1」(Behance、作者名は取得不可) | Cavalry の Text Shape + Duplicator の文字実験の連作 | https://www.behance.net/gallery/173345949/kinetic-typography-made-with-Cavalry-Part-1 | 中 |
| heyalisa「Typography Animation in Cavalry — Duplicators and Context Index」(YouTube + project file) | 文字を Duplicator に乗せ Index で時間差、この動画の弧・格子と同じ組み立て | https://www.youtube.com/watch?v=INL473G-1kA · https://heyalisa.gumroad.com/l/pf_duptypo | 高(手法) |
| Alice Delecolle Chong Li Min「Kinetic Motion Graphic (Chinese Typography)」2016、AE 製 | 漢字の書 + 印章 + 紙質の素材の型。作者本人の弁: "it's hard to find a example as chinese kinetic motion graphic"(2016 時点) | https://www.behance.net/gallery/35696125/Kinetic-Motion-Graphic-(Chinese-Typography) | 高(素材)/道具は AE |

## 2. 素材の出自(3 行)

- 赤い丸の姓氏: 揮春は「通常紅色底，黑色或者金色字」(粵語 Wikipedia)、姓を書く紅燈籠 = 字姓燈(台灣光華雜誌・台北市教育局教材)。丸に姓 1 字は印章(圓章)の型とも重なる。**この動画がどれを引いたかの証拠は無い**。 https://zh-yue.wikipedia.org/wiki/揮春 · https://www.taiwan-panorama.com/zh/Articles/Details?Guid=14510061-9656-472f-ad59-37d022712f01
- 緑の墨字 = 北魏楷書: 區建公・蘇世傑・卓少衡が 1940〜70 年代の香港招牌に多用(陳濬人《香港北魏真書》三聯 2018)、霓虹でも「北魏體が最も普及」(M+ NEONSIGNS.HK、譚智恒)。**緑という色の裏取りは無し**。 https://book.douban.com/subject/30280319/ · https://www.neonsigns.hk/new-posts/typography-of-neon-signs/ · https://medium.com/invisibledesigns/溝通的建築-香港霓虹招牌的視覺語言-3051ce66e3b4
- 青地白抜き ≠ 小巴牌: 小巴牌は「白底、紅字横 = 終点、藍字縦 = 経由」(麥錦生、端傳媒 2016 / SCMP 2016)。青地白字なら 60 年代の T 型街牌・監獄體路牌の方が近い(香港道路研究社、hk01)。 https://theinitium.com/article/20160811-hongkong-hkoldfonts · https://www.scmp.com/magazines/hk-magazine/article/2038283/mak-kam-sang-last-minibus-sign-writer-hong-kong · https://www.hk01.com/熱爆話題/360523

## 3. 動きの系譜 — この動画の運動はどこから来たか(主)

動画の運動を 8 つに分け、各系統の一次資料に当てる。**「言及あり」は本人・研究者がその運動を語っている事、「機構で一致」は装置/ソフトの仕組みが同じ事**。

### 3a. 光学と機械の装置

| 運動 | 装置 | 一次資料 | 確度 |
|---|---|---|---|
| 万華鏡の四方対称 | Brewster 特許 1817-07(英 #4136)「a new optical instrument called the 'Kaleidoscope' for exhibiting and creating beautiful Forms and Patterns」、1819 Treatise(polyangular / polycentral の変種) | https://www.si.edu/object/kaleidoscope:nmah_1817881 · https://www.goodreads.com/en/book/show/68044628-a-treatise-on-the-kaleidoscope | 高(機構で一致) |
| 環に並べて回す(字が散って環に集まる、市松が回る) | Horner「Dædaleum」1834-01、Philosophical Magazine 3rd ser. vol.4 pp.36–41(Plateau の phénakisticope の円筒版) | https://en.wikipedia.org/wiki/William_George_Horner · https://journals.sagepub.com/doi/10.1177/17468477221085412(Veras 2022、zoetrope の忘れられた特性) | 高(機構で一致) |
| 本のページめくりで冒頭に戻る | Linnett「Kineograph」特許 1868-03-18(Birmingham の石版印刷屋、"optical illusion") | https://archive.org/details/kineograph-patent · https://collection.sciencemuseumgroup.org.uk/objects/co8208814/two-kineograph-flicker-books | 高(機構で一致) |
| 回転看板・印刷機 | 一次資料**見つからず**(この動画との対応も薄い) | — | — |

### 3b. 抽象映画

| 運動 | 作家 | 本人の言葉(引用は 15 語以内)/ 研究者 | 一次資料 URL | 確度 |
|---|---|---|---|---|
| 音の拍で図形が湧く(全体の切り替え) | Oskar Fischinger | 「My Statements are in My Work」(Art in Cinema 1947): "new motions and rhythms sprang out of the music"。Radio Dynamics 1942 は「色・音楽・作曲」で鏡映の図形。Whitney 兄弟は 1939–40 に LA Stendhal Gallery で Fischinger を見た(Moritz) | https://centerforvisualmusic.org/Fischinger/MyStatementsWork.htm · https://www.centrepompidou.fr/en/ressources/oeuvre/cabERe · https://unframed.lacma.org/2012/04/26/oskar-fischinger-and-california-abstract-animation | 高(言及あり) |
| コマの間 = 運動そのもの | Norman McLaren | "the art of movements-that-are-drawn … What happens between each frame is more important"。Rythmetic 1956(数字が主役)、Canon 1964(輪唱 = 同じ動きの時間差反復) | https://www.sensesofcinema.com/2005/cteq/norman_mclaren/ · https://en.wikipedia.org/wiki/Rythmetic · https://en.wikipedia.org/wiki/Canon_(film) | 高(言及あり)。Canon は「Stagger」の映画的先例 |
| 物ではなく「運動の figure」 | Len Lye | "pure figures of motion"(Figures of Motion: Selected Writings、Auckland UP) | https://www.lenlyefoundation.com/page/figures-of-motion/4/91/ · https://shop.govettbrewster.com/products/figures-of-motion-len-lye-selected-writings | 高(言及あり) |
| **放射・螺旋・環・文字の対称複製** | John Whitney | Digital Harmony 1980 ch.IV "First, motion becomes pattern if objects move differentially." / "A resolution to order in patterns of motion occurs at points of resonance."。機構: WWII M-5 砲撃照準器のカムとボール積分器を pantograph に置換 → 「差分(drift)」から運動を得る。Vertigo 1958 の Lissajous 螺旋(Bass の依頼、天井の振り子 + 回転台)、Catalog 1961(文字と点の変形、対称)、弟 James の Lapis 1966(同心の点の環、mandala) | https://archive.org/stream/DigitalHarmony_201611/Digital%20Harmony_djvu.txt · https://rhizome.org/editorial/2013/may/9/did-vertigo-introduce-computer-graphics-cinema/ · https://rhizome.org/editorial/2008/nov/21/catalog-1961-john-whitney/ · https://www.awn.com/mag/issue2.5/2.5pages/2.5moritzwhitney.html · https://www.academia.edu/12568356/From_the_Gun_Controller_to_the_Mandala_The_Cybernetic_Cinema_of_John_and_James_Whitney(Zinman) | 高(言及あり + 機構で一致)。この動画の「放射増殖」「散→環」「z のトンネル」に最も近い |
| 文字を題材にした題名の運動 | Saul Bass / Pablo Ferro | Bass: North by Northwest 1959 が「文字が飛び込む」最初の長編(Betancourt)。Ferro: Dr. Strangelove 1964 の手書き文字(AIGA Eye on Design)。**両者とも装置の系譜より「読む像」の系譜**(Betancourt『Typography and Motion Graphics: The Reading-Image』2019) | https://www.artofthetitle.com/designer/saul-bass/titles/ · https://eyeondesign.aiga.org/youre-influenced-by-film-title-designer-pablo-ferro-and-you-dont-even-know-it/ · https://www.routledge.com/Typography-and-Motion-Graphics-The-Reading-Image/Betancourt/p/book/9780367029289 · https://archive.org/details/historyofmotiong0000beta | 中(この動画の運動より素材の先例) |
| 中国語圏の先例 | — | 上海美術電影製片廠の水墨動画(1960)は絵の系譜で、**文字を配置して時間に置く先例は見つからず**。香港の Chris Cheung「Waving Script」2022(故宮館、運動する書)は近年 | https://zh.wikipedia.org/zh-hant/中国动画史 · https://en.wikipedia.org/wiki/Chris_Cheung | 低(見つからず) |

### 3c. 手続き型(procedural)— 道具の遺伝子

| 段 | 何が生まれたか | 本人の言葉 / 公式 docs | URL | 確度 |
|---|---|---|---|---|
| Cinema 4D MoGraph 2006(R9.6、Per-Anders Edwards) | Cloner = 「数千の物を少数の parameter で一括制御」、Effector で場を掛ける。SciTech 賞 | https://novedge.com/blogs/design-news/design-software-history-the-evolution-of-cinema-4d-from-amiga-origins-to-motion-graphics-powerhouse · https://usermanual.wiki/maxon/C4DmgR9e.1685243522/help(R9.6 MoGraph manual) | 高 |
| MASH(Mainframe、Maya、2015 Autodesk 買収、2016 Maya 同梱、2018 SciTech 候補) | 「Cinema 4D の Mograph に似た」手続き型 node 網: Distribute / Offset / Random / Curve / Signal / Time / Falloff / Audio / Color。Waiter node に node を足す | https://help.autodesk.com/cloudhelp/2026/ENU/Maya-MotionGraphics/files/GUID-D4FECFDC-F91A-4BDC-A1B0-A24EB087B2DD.htm · https://lesterbanks.com/2014/06/using-flocking-mash-maya/ · https://www.cgchannel.com/2020/08/check-out-cavalry-mainframes-cool-new-motion-design-tool/ | 高 |
| Cavalry(Scene Group、2019-01 発表、2020-08 1.0、2026-02 Canva 傘下) | Ian Waters(CTO): "Ever since my days as an animator, I've believed that the power of proceduralism should be at the heart of all professional digital content creation tools"(2019 発表文、CG Channel / Creative Bloq 引用)。Adam Jenns(CMO、2023): "all the innovation in animation seemed to be happening in 3d with the world of 2d/2.5d left largely stagnant"。Waters の Medium「How Cavalry came to be」は 403 で未検証 | https://www.cgchannel.com/2020/08/check-out-cavalry-mainframes-cool-new-motion-design-tool/ · https://www.creativebloq.com/features/cavalry-2d-motion-design-software · https://medium.com/cavalry-animation/how-cavalry-came-to-be-5ace628d8c28(未検証) · https://www.cgchannel.com/2026/02/canva-acquires-next-gen-motion-graphics-tool-cavalry/ | 高(引用は二次掲載) |
| Cavalry の道具 ↔ この動画の運動 | Duplicator(「grids, circles and other patterns」)+ Distribution Types(Grid / Circle = 「radial pattern」)+ Stagger(「sub-mesh such as a Duplicator or Text Shape」に時間差)+ Falloff(「Circle mode で放射状に sample」)+ Text Shape の per-character 制御。弧 = Circle 分布、格子 = Grid、放射増殖 = Duplicator + Falloff、散→環 = 分布の切替え(補間)、螺旋 = Extrude + Duplicator の回転 offset | https://cavalry.studio/docs/nodes/shapes/duplicator/ · https://cavalry.studio/docs/nodes/general/distribution-types/circle-distribution/ · https://docs.cavalry.scenegroup.co/nodes/behaviours/stagger/ · https://cavalry.studio/docs/nodes/utilities/falloff/ · https://medium.com/cavalry-animation/text-in-cavalry-b1c13086a567 | 高(機構で一致) |
| Processing(Reas & Fry 2001)| 「sketch」「指示の集合が自律的に作品を生む」= generative の定義(Reas)。Maeda の Design By Numbers の直系。**Cavalry 創業者が Processing を名指しした文は見つからず** | https://anthology.rhizome.org/processing · https://mitpress.mit.edu/9780262028288/processing/ | 中(並行系統) |
| MoGraph/MASH の作者が Whitney・Fischinger を名指しした文 | **見つからず** | — | — |

### 3d. Disney の 12 原則は含まれるか

| 見つけた物 | 一次資料 | 確度 |
|---|---|---|
| 12 原則の出所: Thomas & Johnston『The Illusion of Life』1981、第 3 章「The Principles of Animation」。対象は**登場人物の演技**(squash and stretch、anticipation、follow through…)。1930 年代の Disney の「より現実的な」動きの追求から | https://en.wikipedia.org/wiki/Twelve_basic_principles_of_animation · https://www.semanticscholar.org/paper/38a0fa90a395661f51f749cf42ce5f5de2c41d98 | 高 |
| Cavalry 公式 docs・release notes・創業者の記事・MASH docs に「12 principles」「squash and stretch」「Illusion of Life」の言及は**検索で 0 件** | (§3c の URL 群) | 高(無い事の確認) |
| この動画の運動は個体の演技(squash/anticipation/overlap)ではなく、**同一則を N 個に配って差分で模様を作る**型(Whitney 「move differentially」、MoGraph Cloner、Cavalry Duplicator + Stagger)。ゆえに **12 原則は含まれない**。ただし ease(slow in/out)だけは両系統に共通で、Cavalry のグラフエディタで付く | https://archive.org/stream/DigitalHarmony_201611/Digital%20Harmony_djvu.txt · https://docs.cavalry.scenegroup.co/nodes/behaviours/stagger/ | 中(材料からの分解、判断ではなく対応表) |

## 4. 「本体はどこにあるか」— 完成を先に見ていた証拠

| 見つけた物 | 一次資料 | 確度 |
|---|---|---|
| **この動画の作者のスケッチ・絵コンテ・静止画の先行作は見つからず**(作者未特定のため) | — | — |
| 系譜側の先例: Whitney は Vertigo の螺旋を Bass の**静止画の指定(Lissajous を 100% 正確に)**から機構で作った = 完成図が先、装置が後(Rhizome 2013、Typotheque King) | https://rhizome.org/editorial/2013/may/9/did-vertigo-introduce-computer-graphics-cinema/ · https://www.typotheque.com/articles/taking-credit-film-title-sequences-1955-1965-5-spiralling-aspirations-vertigo-1958 | 高 |
| Fischinger は Shakespeare の図解(静止)から始め「絵が運動を必要とした」(1947 の文)= 静止の図が先 | https://centerforvisualmusic.org/Fischinger/MyStatementsWork.htm | 高 |
| Lapis(James Whitney)は**手描きの点の模様が先**、camera を計算機で位置決め(Moritz) | https://www.awn.com/mag/issue2.5/2.5pages/2.5moritzwhitney.html | 中(403、検索抜粋のみ) |
| Cavalry 側の手つき: 動画に選択枠・ノード点・ガイド線を残すのは Cavalry 公式/Scenery の「scene を見せる」投稿の型(Scenery は「scene を共有・発見・学ぶ」場) | https://scenery.io/ | 中 |

## 5. 見つからなかった物(正直に)

- 作者・作品名・投稿 URL・投稿文・インタビュー(問い 1 の中心)。Instagram / X / Medium / AWN / Rhizome / Bright Lights は 403 で本文が取れず、検索抜粋だけ。
- Cavalry 創業者(Waters / Hardcastle / Jenns)が Whitney・Fischinger・McLaren・Disney 12 原則を名指しした文。
- MoGraph(Edwards)・MASH の作者が影響源を語った一次インタビュー。
- 中国語圏の「文字を配置して時間に置く」先例(1960〜90 年代の片頭・台徽)。
- 緑の墨字・青地白抜きの色の裏取り(北魏体の色、小巴牌は白地)。
- 「回転看板・印刷機」が運動の系譜に入る一次資料。
