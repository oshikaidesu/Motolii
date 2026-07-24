# First-party Vism表現需要の初期調査

状態: **観察**

日付: 2026-07-23

## 1. 目的と限界

After Effectsの標準effect／第三者plugin、AviUtl 2の公開script群、Cavalryのpixel filterを横断し、Motoliiのfirst-party pre-Vism参照実装がどの表現需要を先に反証すべきかを粗く地図化する。

これは売上順位や採用決定ではない。vendor自身の製品説明、Adobe公式一覧、GitHub topicの公開repositoryを主な観察母集団とし、次を証明しない。

- 各pluginの実利用者数、市場占有率、作品品質
- 外部製品と同じparameter、UI、内部実装をMotoliiへ移す妥当性
- Vism package、container、loader、typed port、custom UIの実装許可
- 第三者製品名・preset・見た目をfirst-party資産へ複製する許可

Motolii側の正本は[Vism concept](../vism-package-concept.md)、[Vism実装計画](2026-07-17-vism-implementation-plan.md)、一覧入口である[Vismプラグインカタログ](../vism-plugin-catalog.md)である。v1のfirst-partyは静的plugin境界を外側から反証する**pre-Vism reference**であり、調査候補からpackage形式を逆算しない。

## 2. 外部で繰り返し現れる需要

### 2.1 AE標準effectは「合成可能な知覚primitive」が厚い

Adobeの[現行effect一覧](https://helpx.adobe.com/jp/after-effects/using/effect-list.html)には、Blur、Glow、Fractal/Turbulent Noise、Displacement、Turbulent Displace、Echo、Posterize Time、Time Displacement、各種Wipe、Key/Matteが並ぶ。[Noise / Grain公式説明](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/noise-grain-effects.html)はFractal Noiseを背景だけでなく、displacement map、texture、雲・炎・水蒸気等の入力として位置づける。

ここで重要なのは単品数ではなく、noise→mask／displace、matte→glow、repeat→blurのように、**一つの結果を次のeffectの入力へ回す小さいprimitive**が制作語彙になっていることである。

### 2.2 第三者pluginは三つの塊を埋める

Adobe自身も[第三者plugin一覧](https://helpx.adobe.com/jp/after-effects/plug-ins.html)で、Red GiantのTrapcode／Magic Bulletを映画・broadcast post-productionの標準的な製品群として扱い、Particular、Form、Shine、Starglow、Sound Keys等を列挙している。vendor説明は自己評価を含むため人気の独立証明ではないが、長期に維持される需要領域の観察には使える。

1. **画像を強く変える小さいeffect**: glow、light rays、blur、distortion、film/grain、chromatic split、glitch、VHS、halftone、dither。Maxon Universeは[89個のGPU effect](https://www.maxon.net/en/product-detail/red-giant/universe/tools?categories=606798)としてPixel Dither、VHS、Glitch、text、wipe群を提供し、Boris Sapphireは[glow／flare／blur／stylize／transition／distortion](https://borisfx.com/products/sapphire)を一貫したparameterとpreset browserで扱う。
2. **一つでは組めない生成系**: particle、replica、3D object、force、volume。Trapcode Particularは[複数particle systemとphysics](https://www.maxon.net/en/product-detail/red-giant/particles-and-3d/trapcode-particular)、Stardustは[modular particle／object graph](https://superluminal.tv/user-guide)を売りにする。
3. **Host操作の摩擦を消すtool**: easing、anchor、selection、vector transfer、rigging。Motion M4は[easing／anchor等を集約したanimation toolkit](https://motion.mtmograph.com/)、Overlordは[Illustratorのshapeをfile準備なしでAEへ渡す](https://help.battleaxe.co/overlord/)、RubberHoseは[調整可能なbendy rig](https://battleaxe.co/rubberhose)を提供する。

3は表現需要の重要な証拠だが、そのままVism候補ではない。Host command、import、selection、rig ownerへ置くべきものを「人気pluginだから」とVismへ押し込まない。

### 2.3 Pixel Sorterは周辺的な一発芸ではない

Cavalryはvector／procedural motionを主戦場にしながら、公式の[Pixel Sorting Filter](https://cavalry.studio/docs/nodes/effects/filters/pixel-sorting-filter/)を持つ。brightness等でpixelを並べ替え、direction、detail、threshold、reverse、shape maskを公開し、計算負荷が高いことも明示する。同じ公式filter群にはglow、halftone、pixelate、RGB split、scan line、slit scan、dither系が隣接する。

AE側でもaescriptsの[AE Pixel Sorterタグ](https://aescripts.com/learn/tag/AE-Pixel-Sorter/)にはcommercial、reel、glitch tutorial等の継続例があり、particle、Deep Glow、audio reaction、motion blur等との併用例が見える。AviUtl 2にも独立した[Pixel Sorter script](https://gist.github.com/zopty/ea8b643b13fb7802763d8ad49625e7e2)があり、threshold、direction、速度／精度tuningを公開している。

したがってpixel操作は「raster製品だけの基礎機能」ではなく、vector中心のHostへ異なる物質感を持ち込み、importした画像・動画をそのHost固有のmotion語彙へ接続する**注目獲得用の隣接表現**になり得る。Motoliiでは特に、VRAM常駐、Draft/Final、mask入力、parameter animationを同時に実証できる候補である。

### 2.4 AviUtl文化は基礎穴埋めと作風発明を同時に行う

2026-07-23時点のGitHub [`aviutl2-script` topic](https://github.com/topics/aviutl2-script)には41 repositoryがあり、上位には次が混在する。

- 丸角矩形、回転補助、傾斜変形等のbasic集
- text animation支援、auto lyric animation
- stylize集、NTSC、grunge、halftone
- object motion blur、複合direction/radial/rotation blur
- gradient editor、resize、auto clipping、camera adjustment
- point zoom、page roll、ground shadow、kaleidoscope、tile pattern

これは「first-partyは派手な看板だけでよい」という仮説を支持しない。小作者の生態系は、標準Hostの地味な穴を埋める道具と、作品の顔になる狭い表現の両方から育つ。first-party参照実装も、**基礎primitiveと記憶に残るsignature effectのbarbell**を持つ必要がある。

### 2.5 Tutorialの購入済み前提が示すHost境界の不足

AE周辺では、plugin名を説明せず「必要なもの」として列挙するtutorial、template、配布projectが一つの文化になっている。[Motion Arrayのtemplate依存案内](https://help.motionarray.com/hc/en-us/articles/9332163360541-After-Effects-Templates-That-Require-Plugins)はElement 3D、Trapcode Particular／Form／Shine／3D Stroke、Optical Flares、Saber、Plexus、Duik、Stardust等を通常の追加要件として扱う。Adobeも[network renderの説明](https://helpx.adobe.com/after-effects/desktop/render-and-export/automate-rendering/automated-rendering-network-rendering.html)で、第三者pluginを全render machineへ同じように導入しlicense条件を確認する必要を明記する。

第三者が不足を見つけて製品を作り、制作者が対価を払って使うこと自体は健全である。問題はpluginの存在や有料性ではない。Hostの基礎能力と第三者のspecializationが十分に分離されず、作品を開くための依存、version、導入、欠落診断、可搬性まで個別vendor packageへまとまって流出する点にある。代表的な塊は次である。

| 暗黙必須stack | tutorialで担う役割 | 境界上の問題 | Motoliiへの翻訳 |
|---|---|---|---|
| Trapcode Particular / Form / Stardust | particle、trail、smoke、abstract field | AE標準particleではなく有料suiteを学習開始点にする。[School of Motion](https://schoolofmotion.com/blog/trapcode-particular-tutorials-after-effects)もParticularをprofessional MoGraphで最も広く使われるparticle systemとしてtutorialを編成 | 決定的Particle Fieldをfirst-party参照にし、相互作用physicsだけSimulation/Bakeへ |
| Sapphire | glow、shake、warp、transition、stylize等の汎用語彙 | 一つの表現でなく巨大suite全体がproject依存になり、どの小意味が必要か隠れる | 小Filterへ分解し、Projectは使ったstable plugin IDだけを参照 |
| Twixtor + RSMB | slow motion／time remapとmotion blur | 時間写像と動きの見え方という制作基礎が二つの有料plugin前提になる。[2026年のAMV講座](https://www.udemy.com/course/after-effects-masterclass/)もTwixtorとRSMB Proを「cinematic slow motion」の学習項目にする | usableなTimeMap／motion blur baselineはHost責任。高度補間は後続pluginでも基礎操作を欠落させない |
| Deep Glow | 発光、bloom、neon finish | 標準Glowを使う説明が省略され、第三者Glowが実質的な既定になる | 色変換を混ぜないPerceptual Glowをfirst-party候補へ |
| Element 3D + Optical Flares | 3D title／logo、flare、space scene | tutorial一つがElement、Particular、3D Stroke、Optical Flares等を同時要求する例がある。[title recreation例](https://www.motiontutorials.net/blog-tutorials/tag/Optical%2BFlares) | 3D scene基礎はHost capability、flare/raysは小Vism候補。巨大な一体bundleにしない |
| Magic Bullet Looks / BCC / Universe | color finish、glitch、transition、汎用effect | color、質感、transitionのほぼ全域をvendor suiteへ委ね、Project可搬性をsuite版へ依存させる | creative lookと出力色変換を分離し、grain/glitch/wipeを小さい表現へ |
| Flow / Motion / FX Console / EaseCopy | easing、effect検索、anchor、反復操作 | 画を作るeffectでなく、Hostの基本操作摩擦を外部extensionで補うことがtutorial前提になる | VismにせずHost input／command／easing UIで吸収 |
| Saber / 3D Stroke / Plexus | glowing stroke、write-on、connected dots | 特定のよく見る作風がplugin名でしか説明されなくなる | path stroke、repeat、typed connectionの共通能力とstyle Vismを分ける |

AMV／short edit側ではHost境界の不足がさらに見えやすい。配布projectがSapphire、RSMB、Magic Bullet Looks、BCC、Video Copilot、Twixtor、Twitch、Deep Glowを同時要求する[実例](https://payhip.com/b/1WYf9)があり、communityでもTwixtor、Deep Glow、Sapphire、S_Shake等が「project fileを開くために足りないもの」として語られる。この種の投稿は市場統計ではないが、初心者が表現の意味より先にplugin packを集める導線の一次観察になる。

Motoliiが反転すべきなのは「第三者pluginを不要にする」ことでも、需要が見えた機能をすべてCoreへ取り込むことでもない。**最小Coreがidentity、型付き入出力、時間、依存、lifecycle、Undo、resource、failure／missing diagnosticsを共通境界として持ち、未知の表現語彙を第三者が小さく発明できる余白を作ること**である。時間、blur、glow、noise、mask、repeat、検索、easingのbaselineはHost／first-party公開境界だけで成立させ、tutorialが作品の意味から始められるようにする。第三者Vismは基礎欠落の通行料ではなく、新しい作風・精度・作者性を持ち込む健全な発明として増えるべきである。

### 2.6 xClonerは不足を埋めた健全な発明であり、Core境界のprobeである

[xCloner公式製品説明](https://aescripts.com/xcloner/)は、動画、画像、shape、text、vector等の任意layerをLinear／Radial／Grid／Path／Objectの5 modeで複製する。Pathはcount／distance／vertex配置と方向追従、Objectはsource layerのalpha／lumaから配置し、最大20 source、個別transform、blend、offset／random timeを扱う。v1.4はmatteのalpha／luma値から各cloneが表示するsource時刻まで選ぶ。

vendorは標準layer duplicationと比べ数千cloneを扱える性能を前面に出す。数値はvendor benchmarkで独立検証前だが、製品が売っているものは明瞭である。足りない能力を独立作者が実装したことは、拡張生態系の正しい働きである。同時に、AEではその発明を小さい宣言的capabilityとしてHostへ接続する余白が狭く、次の評価系を一つのeffectへまとめて所有せざるを得なかったことも示す。

1. instance populationをlayer copyなしで持つ
2. linear／radial／grid／path／image fieldでdistributionする
3. cloneごとのtransform、順序、randomness、blendを評価する
4. cloneごとのlocal timeをoffsetまたはfieldで変える
5. 数千instanceを一つのrender境界で処理する

MotoliiではxCloner相当を万能Vism一個として再現しない。現行P0I/P7 Duplicator判断へ、次の分離を入力する。

- stable instance identity、評価順、bounds、選択、GPU populationはCore／Host側の審判
- distribution recipeと局所的な見た目はLayerSource／Vism候補
- effector／fieldはP0Iで比較中の評価形を先に閉じ、専用公開APIを発明しない
- path／mask／texture入力は具体provider IDでなく型付き入力として比較する
- per-instance source timeはTimeMap／TemporalFootprint／cache境界へ触れるため、xClonerの存在を理由に先行実装しない

xClonerが隠れやすいのは、完成画のstyle名ではなく**多数の表現を生む文法**だからである。ここから「ClonerをCore機能にする」と短絡しない。Motoliiの最小Coreが持つのはstable identity、決定的な時間評価、型付き接続、依存とlifecycle、GPU resource、欠落診断という発明の土台であり、Linear／Radial／Grid／Path／Object等の具体的な生成文法はfirst-party／third-partyが同じ公開境界で競争・派生できる。需要が反復し意味が安定した時だけ、個別製品の模倣ではなく共通primitiveの昇格を別途裁定する。

この境界の強さは、小さいeffectだけでなく、xClonerを越える複雑な作者成果で反証する。多数のprovider、distribution、effector、field、local time、styleを束ねる「変態的」なmacro packageが生まれても、Host内部API、生JSON／文字列走査、opaque ID分岐、隠れた可変state、専用backdoorを必要としないことが目標である。現行概念上は、万能Vism一個へ閉じ込めることを既定にせず、独立して交換可能な小さいVismと、それらを型付きに接続してProjectへmaterializeするKitへ翻訳する。Vism／Kitのcontainer、schema、linked updateは未決のままとし、この観察からpackage形式を確定しない。

### 2.7 Glowによる現行Vism境界の監査

Glowは[Vism concept](../vism-package-concept.md)が未知の表現動詞とfirst-party例に明記し、[plugin authoring](../plugin-authoring.md)もtexture 1→1の`FilterPlugin`用途として明記している。概念分類は成立している。現行traitもGPU texture input/output、時刻、型付きparameter、`Quality`、Host所有`PipelineCache`を渡すため、現在frameだけを使う決定的な単一pass Glowならpre-Vism referenceとして実験できる。

ただし、現行コードでproduction品質のPerceptual Glowが成立するとはまだ言えない。

| Glow要求 | 現行事実 | 判定 |
|---|---|---|
| texture 1→1、parameter animation、純関数、Preview/Export共通 | `FilterPlugin::render`と`RenderCtx::quality`に存在 | 成立 |
| threshold、限定半径の近傍sample、元画像との合成 | 一つのWGSL pass内なら表現可能 | 限定成立 |
| linear light、1.0超のhighlight、HDR中間 | `FrameDesc`には`LinearRgb`／`Rgba16Float`の型があるが、現行renderはpremultiplied `Rgba8Unorm`／`Srgb`だけを受理し、pipeline targetも固定 | 未成立 |
| separable blur、downsample pyramid、upsample、複数scale bloom | pluginへHost所有の一時texture／pass graphを要求する公開口がなく、render中のtexture生成や`&self`の隠れpoolは規約違反 | 未成立 |
| optional mask／別texture入力 | 現行Filter arityは厳密に1→1で、`NodeDesc`はtyped texture portをまだ表せない | 未成立 |
| Draft/Finalのsample／scale差 | `Quality`は到達するが、Glow固有の縮退規則とoracleは未定 | 口のみ |
| `.vism`として導入・配布 | container、manifest、loaderは停止線内 | 未実装 |

したがって答えは、**Vismの意味定義には収まるが、現行実行境界だけで本命Glowを完成させることはできない**である。単一pass近似を製品版と呼んだり、plugin内でsRGB↔linear変換したり、毎frame中間textureを生成したりして穴を隠さない。先に必要なのはGlow専用APIではなく、少なくとも次の共通能力の審判である。

1. Render全体で一元化されたlinear/HDR中間表現と最終色変換
2. Host所有の一時texture lifecycleと、複数passを宣言・計画・cache keyへ反映する境界
3. mask等を具体plugin IDなしで渡すtyped texture port

これらはBlur、Bloom、large-radius Displace、outline、depth系post等にも再利用可能かをfixtureで確認し、一用途専用の`GlowPlugin` backdoorとして追加しない。package／schema／公開APIはこの観察だけで解凍しない。

### 2.8 締結前の最終stress search

表現名を追加収集するのでなく、Vismがまだ名前を持たない実行寿命を探した。現行CavalryではGlowは[公式Filter](https://cavalry.studio/docs/nodes/effects/filters/glow-filter/)であり、2.1.1 release notesは[Glowがfeatureになった](https://cavalry.studio/docs/tech-info/release-notes/2.1/2-1-1-release-notes/)と記録する。一方、第三者SDKは[custom Shader／Filter／JavaScript Layerとmulti-pass Filter](https://cavalry.studio/docs/tech-info/third-party-plugins/)を明示的に許す。したがって「Glowが存在する」だけでなく、community表現が標準へ昇格でき、第三者も同じmulti-pass級へ到達できることが比較点になる。

主要な境界圧力を追加で照合した結果は次である。

| 外部例 | 箱へ掛かる圧力 | Motoliiでの正規経路 | 残る論点 |
|---|---|---|---|
| Cavalry Glow／第三者multi-pass Filter | linear/HDR、複数pass、中間texture、標準昇格 | Filter + Host所有pass/resource | §2.7の3共通能力。Glow専用口にしない |
| [Overlord](https://help.battleaxe.co/overlord/) | Illustrator／Figma側選択をfile準備なしでshapeへ渡す双方向bridge | External adapter → typed VectorRecipe／Asset → Authoring Tool materialize | Adobe/Figma object IDをDocumentへ焼かず、companion要件とpermissionを宣言 |
| [Lockdown](https://aescripts.com/lockdown/)／[Mocha Pro](https://borisfx.com/products/mocha-pro/)／[Depth Scanner](https://aescripts.com/depth-scanner/)／[Tracery](https://aescripts.com/tracery/) | tracking、mesh、depth、mask、解析結果と専用workspace | Analysis/Bake provider → typed DataTrack／mask／mesh／depth artifact | 長時間job、進捗、取消、stale、model/version、編集可能結果の型 |
| [GEOlayers](https://aescripts.com/geolayers/)／[Templater](https://aescripts.com/templater/)／ComfyUI連携 | network data、認証、外部database/model、batch生成 | 編集時External capability → Asset／Materialize／Bake | render時network禁止、secret非保存、由来・offline・再実行・permission |
| [Neat Video](https://www.neatvideo.com/features)／[Datamosh 2](https://aescripts.com/datamosh/) | 前後frame解析、codec motion、marker範囲、時間窓 | TemporalFootprintまたはHost所有codec/analysis Bake | 任意seek、cache key、再帰有無、範囲dirty、外部tool version |
| [Newton](https://aescripts.com/newton/) | 2D physicsを対話設定し時間列へ確定 | Simulation／StateTrack Bake | 固定step、checkpoint、部分再Bake、1操作のCommit |
| [Bodymovin／Lottie](https://github.com/airbnb/lottie-web) | Project意味を別runtime向けartifactへ変換し、非対応機能を診断 | **Exporter／Delivery Adapter候補** | 現行Vismの未知の動詞には収まるが、作者分類とlifecycleに明示名がない |

前六群は既存のMaterialize、Pure Live、Temporal Window、Simulation Bake、External Materialと、§2.7のmulti-pass資源境界で受け止められる。ComfyUIとAEを接続して背景除去やupscaleを行うcommunity実験も既に観察でき、将来仮説だけではない。ただし個別scriptをMotolii要件の根拠にせず、外部jobを同期Filterへ偽装しないという負例確認に限って使う。

最終探索で新しく残ったのは**外向きのDelivery Adapter**である。Vism packageは未知の動詞、要求capability、型付きinput/output、artifact、trustを概念上保持できるため箱の再設計は不要だが、現行の`Filter / Composite / LayerSource / ParamDriver / Generator / Simulation / Authoring Tool`列挙だけではBodymovin型を正しく分類できない。

v1の製品出力は既存の音声mux込み完成映像だけに閉じ、Delivery Adapterは将来capabilityの席だけを[Vism concept §5.2](../vism-package-concept.md#52-外向きdeliveryはv1映像だけに閉じ能力席だけ残す)へ置く。Lottie、animated SVG、OTIO等を実装せず、次を将来の比較条件として残す。

- read-only Project／selection／rendered frameのどこまでを入力にできるか
- 出力artifactの型、destination、上書き、取消、atomic publishを誰が所有するか
- 対応外のVism／effect／font／expressionを黙って落とさず、preflightで診断するか
- filesystem／network permissionとsecretをpackage／Projectから分離できるか
- Exporterを通常のVism成果として配布するか、Host adapterの別成果物にするか

これはExporter trait、schema、Lottie対応の実装許可ではない。将来比較を始めるまで、既存の映像Exportや`.vism` package契約へBodymovin由来のfieldを足さない。

## 3. Motoliiへ翻訳した表現地図

外部名をそのままtask名や公開IDにせず、Motoliiの責任境界へ翻訳する。

| 表現需要 | 外部での反復 | Motoliiで反証する責任 | 初期位置 |
|---|---|---|---|
| Glow / bloom / rays | AE、Sapphire、Red Giant | linearなtexture filter、alpha／HDR、色変換一元化、mask | pre-Vism Filter候補 |
| Fractal／turbulent field | AE noise、displacement入力 | 決定的seed、正準座標、texture生成とconsumer分離 | LayerSource/provider候補 |
| Displace / warp | AE、Cavalry、Sapphire | texture入力、sampling、枠外、preview/export同一 | Filter候補 |
| Pixel Sort | AE、Cavalry、AviUtl 2 | VRAM内sort、mask、方向、threshold、Draft/Final負荷 | signature Filter候補 |
| Dither / halftone / pixelate | Universe、Cavalry、AviUtl 2 | 解像度依存と正準意味の分離、palette／pattern入力 | Filter候補 |
| RGB split / scanline / VHS / grain | Universe、Cavalry、AviUtl 2 | creative color effectと表示色変換の分離、seed | Filter候補 |
| Repeat / tile / kaleidoscope | AE、Stardust Replica、AviUtl 2 | stable identity、正準配置、LayerSourceとFilterの分離 | Radial Repeater後続候補 |
| Kinetic text / lyric | Universe text、AviUtl 2 lyric | grapheme identity、sequence、read-only score、IME | Text gate後の候補 |
| Particle / field / force | Trapcode、Stardust | 第一選択は決定的`f(t)`、必要時だけHost bake | LayerSource／後続Simulation |
| Audio reaction | Sound Keys、AE作品の併用 | DataTrack providerとconsumerを型で分離しKitが接続 | A7後続／Kit候補 |
| Echo / slit scan / time displacement | AE、Cavalry | `TemporalFootprint`、cache、任意時刻access禁止 | v1.x後続 |
| Easing / anchor / vector transfer / rig | Motion、Overlord、RubberHose | Host command、import、selection、tool owner | Vismへ入れない |

## 4. First-party portfolioの初期仮説

最初から大suiteを作らず、異なる契約を1本ずつ反証する候補である。採択順ではない。

### 4.1 近い候補

1. **Pixel Sort** — MotoliiのGPU texture境界とDraft/Finalを視覚的に示すsignature Filter。Cavalry同様、vector中心という予想を裏切る隣接表現になる。
2. **Perceptual Glow** — 利用頻度の高い基礎Filter。ただしcreative bloomとoutput color transformを混ぜないことが最重要負例。
3. **Fractal Field** — 背景presetではなく、mask／displace／parameter driverへ再利用できる決定的providerとして作れるかを問う。
4. **Dither / Halftone** — 小さく、結果が読め、palette・pixel scale・Draft縮退の意味を検証しやすい。
5. **RGB Split / Scanline** — signatureというより小さい合成primitive。単独Vism identityを持つかpresetで十分かを比較する。

### 4.2 依存gate後の候補

1. **Kinetic Lyrics** — Text identityとCharacter Scoreが成立した後。現行Text Motion決定を迂回しない。
2. **Audio Pulse Kit** — beat／DataTrack providerとGlow、Scale、Repeater等をKitが接続する実証。consumer Vismがprovider IDを検索しない。
3. **Deterministic Particle Field** — まず純関数の軽量`f(t)`で成立する範囲。相互作用を持つphysicsはSimulation/Bakeへ分離する。
4. **Echo / Slit Scan** — TemporalFootprintとcache意味の決定後。Filterに過去frame stateを隠さない。
5. **Film / NTSC material** — style presetでなく、再現可能なnoise、scan、bleed、time挙動をどの小表現へ分けるかを先に比較する。

## 5. 選定審判の候補

first-party候補は「有名pluginに似ているか」でなく、次を満たす数で比較する。

1. 複数の制作文化で独立に再発明されている
2. 単品でも結果が読み取れ、他の小表現とも合成できる
3. Motoliiの公開plugin境界を内部APIなしで反証する
4. VRAM常駐、色変換一元化、純関数、正準座標、preview/export同一を守れる
5. Use→Tune→Inspect→Fork→Authorの教材になる
6. 既存Host command、D2、Document、Kit、Simulationの責任を奪わない
7. 失敗例と性能縮退をfixtureで説明できる
8. first-party catalog全体で同じ能力だけを重複証明しない
9. 複雑なVism／Kit構成でも、内部API、生データ走査、opaque ID分岐、隠れstate、表現専用backdoorなしに成立する

「人気」「派手」「実装が簡単」は単独の採択理由にしない。一方、基礎effectだけでcatalogを埋めると製品の想像力を示せないため、各waveに少なくとも一つsignature表現を置く仮説は残す。

## 6. 次の調査

今回の母集団は製品一覧と公開repositoryであり、実作品の使用頻度を測っていない。次段では、MV／title sequence／explainer／VJ等を層別し、制作breakdownまたはproject公開で使用toolが確認できる作品だけを標本にする。

記録単位はplugin名でなく、次とする。

- 表現結果: glow、sort、trail、repeat、kinetic text等
- 入力: texture、mask、path、text、audio/data、過去frame
- 時間性: `f(t)`、時間窓、bake/simulation
- 合成相手: 何と組み合わせて作品になったか
- 役割: Vism、Kit、Host tool、Asset、Presetのどこへ翻訳されるか
- Motoliiの不足契約とSTOP条件

この作品標本と反対側レビューを経るまで、§4をfirst-party実装順へ昇格しない。

## 7. 2026-07-24 AE人気plugin再現候補サーチ

### 7.1 「人気」の扱い

単一marketplaceの売上順位は公開範囲と時期で変わるため、順位表を捏造しない。今回は次の複数signalを使い、**需要の反復**を探した。

1. Adobeが現行の第三者plugin資料で標準的な製品群として明示している。
2. 2025〜2026にも公式compatibility／release／製品更新が続いている。
3. vendor suiteまたはmarketplaceで独立製品として長期維持されている。
4. 別vendorが同じ表現需要を別実装で提供している。
5. tutorial、preset、templateで他表現との組み合わせが反復する。

[Adobeの現行第三者plugin一覧](https://helpx.adobe.com/ae_en/after-effects/plug-ins.html)はRed GiantのMagic Bullet／Trapcodeをfilm・broadcast post-productionのstandardと説明する。[Maxonの現行分類](https://support.maxon.net/hc/en-us/articles/21234625849116-What-happened-to-the-Magic-Bullet-Trapcode-VFX-and-Universe-folders)と[Universe locator](https://help.maxon.net/rg/en-us/Content/html/universe-tools-locator.html)は、Glow、Blur、Dither、Fractal、RGB Separation、Particle、Text、Transition等が2026時点でも一suite内の独立語彙として維持されていることを示す。[Red Giant 2026.2](https://support.maxon.net/hc/en-us/articles/24114684657692-Red-Giant-2026-2-0-December-3-2025)はAE内で全pluginを検索・preset閲覧するPlugin Pageを追加しており、本調査の「Plugin Browser型一覧」方針とも一致する。

これは市場占有率の測定ではない。vendorの自己評価、価格、preset数をMotolii採択理由にせず、同じ需要が複数系統で生き残っているかだけを読む。

### 7.2 候補の処分

| 外部plugin／系統 | 反復する需要 | Motoliiでの処分 | カタログ反映 |
|---|---|---|---|
| Red Giant Optical Glow／各Glow、Deep Glow | 少ない調整で自然な広がり、HDR、downsample、品質調整 | 共通MULTIPASS能力の後にPerceptual Glow。色変換と一体化しない | 既存候補を維持 |
| [Pixel Sorter 4](https://aescripts.com/pixel-sorter/) | threshold、direction、mask、noiseを持つGPU pixel sort。2026にもcross-host更新 | typed maskを受けるVRAM内Filter。CPU readback禁止 | 既存signature候補を強化 |
| Trapcode Particular／Form、[Stardust](https://aescripts.com/stardust) | particle、replica、field、physics、3D objectを一つの制作語彙へする | L0 Particle、typed field、L3 Simulation、Kitへ分解。node UIと万能suiteを複製しない | 既存Particle／Repeaterを維持 |
| [Plexus 3](https://aescripts.com/plexus/) | point間の距離等からline／facet／triangulationを生成するprocedural network | typed Point Setを読むConnected Points／Facets Vism。2Dから反証し、3D scene ownershipを奪わない | **新規候補** |
| [Optical Flares](https://www.videocopilot.net/products/opticalflares/)／Real Lens Flares | flare object、occlusion、luminance source、mask、shimmerを再利用可能な光表現へまとめる | Lens Flare Vism + preset／Kit。AE light直参照、専用designer、vendor object型を公開契約へ写さない | **新規候補** |
| Video Copilot Saber／Red Giant 3D Stroke／AutoFill | path、mask、text輪郭に沿う発光stroke、write-on、領域伝播 | Energy Strokeはpath→textureのL0。Shape Fillはarrival-time fieldをHost Bakeしてrenderを純関数化 | **新規候補** |
| Video Copilot Heat Distortion／Red Giant Heatwave | noise fieldで局所的な熱揺らぎ／屈折を作る | deterministic field + Displace。field生成とconsumerを分離 | **新規候補** |
| [TypeMonkey](https://aescripts.com/after-effects/text/typemonkey/)／Red Giant Text群／[Type](https://aescripts.com/type/) | kinetic layout、type-on、cursor、word／line単位animation、music marker同期 | Text Sequence、小Vism、Kit、必要時Authoring Toolへ分解。layer大量生成や独自cameraを正本にしない | Kinetic Word Layoutを追加 |
| [BeatEdit](https://aescripts.com/after-effects/automation/audio/beatedit-for-after-effects/) | beat検出、選択、marker、key反復、staggerを一つの製品へまとめる | BPM Rhythm Vism、Audio Analysis provider、Beat Pulse consumer、Host marker／commandへ分解 | 既存BPM決定を強化 |
| [Newton 4](https://aescripts.com/newton/) | AE layerを2D physicsへ送り、結果を編集へ戻す | Host所有Simulation／StateTrack Bake。Newton固有world／UIをVismへ複製しない | 既存SIM候補を維持 |
| Universe／Sapphire等のTransition群 | wipe、glitch、light、blur、shakeを完成transitionとして探す | 2入力textureとTimeline transition placementを分離し、Shape Wipe等の小Vism + preset／Kitへ | **新規候補群** |
| [Flow](https://aescripts.com/flow/)／Motion Tools系 | easing、anchor、key clone、sequence等の操作摩擦を解消する | Host command／easing UI／Authoring Tool。画を評価するVismへ入れない | 除外を維持 |
| Element 3D | 3D object import、particle、material、camera、renderをAE内へ持ち込む | M5のHost-owned world／depth／Observation配布と、換装可能Camera Provider、scene renderer、小Vismへ分解。巨大なElement互換Vismを作らない | カタログ外 |

### 7.3 新規候補の実装見通し

#### Lens Flare

最小形は`source position + intensity + seed + flare recipe → texture`のL0 LayerSource／Filter候補である。sourceは2D位置、typed luminance／mask、将来の3D light projectionを別providerとして受ける。flare element列、texture asset、occlusion、shimmerを一つの公開raw object treeへ固定しない。まず少数のclosed elementとpresetで意味を反証し、複数要素の構成はKit／private payload候補として別審判する。

#### Energy Stroke

path／mask／text outlineをtyped geometry inputとして受け、距離場またはstroke tessellationから発光線を描く。Glowとの組み合わせを専用実装へ閉じず、Stroke出力→Perceptual Glowの構成を第一候補にする。write-onはpath-local progressのL0で表現し、Text identityやshape編集を所有しない。

#### Shape Fill / Write-on

複雑領域を種点から自然に埋める到達時刻を毎frame再計算しない。編集時またはcache jobで`arrival-time field`をHost所有artifactへBakeし、render時は`field <= t`を読む純関数Filter／LayerSourceとする。plugin内cache、壁時計、前frame出力、同期CPU flood fillを禁止する。

#### Connected Points / Facets

入力を`Point Set + stable point identity + optional attributes`として比較し、距離、近傍数、thresholdからline／facetを決定的に生成する。Plexusのnode graphやAE light／null走査を持ち込まない。2D L0 fixtureでtyped input、stable identity、GPU populationを反証した後、3D point／depth occlusionをM5へ接続する。

#### Transition

表現計算とTimeline意味を分ける。Vismは`from texture + to texture + progress → output`を評価し、Hostはclip overlap、duration、trim、Undo、selectionを所有する。最初の候補はShape Wipe、Glitch Transition、Light／Bokeh Transitionだが、2入力typed portとtransition placementが未決の間は実装しない。

### 7.4 非目標

- 製品名、preset名、parameter名、UI、thumbnail、デフォルト値を複製しない。
- 一suiteを一Vismへ再現しない。
- AE layer、marker、light、effect stackの走査を公開契約へ持ち込まない。
- 「人気」を理由にCore enum、Document field、raw plugin APIを追加しない。
- vendorの宣伝文句、価格、更新年だけで採択順を決めない。

## 8. 2026-07-24 AviUtl plugin／script候補サーチ

### 8.1 調査範囲と用語

AviUtlでは「plugin」が必ずしも画を作るeffectを意味しない。[AviUtl2利用案内](https://docs.aviutl2.jp/usage)は、scriptがanimation effect、custom object、camera effect、scene change、trackbar movement等を追加し、pluginが入出力、編集window、object、filter effect等を追加すると区別している。AviUtl 1.xの[Plugin SDK mirror](https://github.com/mtripg6666tdr/aviutl_plugin_sdk)にもfilter、input、output、color、languageの別系統がある。したがって本調査は拡張子や配布名でなく、**何を所有する成果物か**でVism、Kit、Host tool、Infrastructure、Adapterへ分類する。

現在の候補母集団は、AviUtl 1.xで長く使われた公開script／pluginと、2026-07-24時点のGitHub [`aviutl2-script` topic](https://github.com/topics/aviutl2-script)に現れる公開repositoryである。topicのstar順は利用者全体の人気順位ではなく、公開GitHub上の発見signalにすぎない。現在も更新されるAviUtl2側を主にし、1.x側は再発明の継続性とHost境界の教訓を読む。

### 8.2 候補の処分

| AviUtl系統 | 反復する需要 | Motoliiでの処分 | カタログ反映 |
|---|---|---|---|
| [Basic系script](https://github.com/sigma-axis/aviutl2_script_Basic_S)、丸角矩形、回転補助、傾斜変形 | 標準Hostで足りないshape／transform primitiveの穴埋め | 丸角矩形と通常transformはHost Shape／Transform基礎能力。作風を持つwarpだけ小Vism。基礎不足をVism乱立で覆わない | Warpを維持、基本形は除外 |
| [Stylize群](https://github.com/korarei/AviUtl2_Stylize_K_Script)のMosaic、Threshold、Posterize、Gradient Map、Tile／Repeat | 画像を少数の知覚primitiveで作風化し、組み合わせる | Pixelate、Dither／Halftone、Gradient Field、Tile等の小Filter／providerへ分解 | 既存候補を強化、Gradient Fieldを追加 |
| Stylize群のASCII | 明暗を文字密度へ置換する映像表現 | typed glyph atlas／font assetを受けるASCII Raster。font選択UI、Text Document、組版正本はHost／Text側 | **新規候補** |
| [Color Halftone](https://github.com/azurite581/AviUtl2-ColorHalftone)、Dither系script | 網点／patternを色・alpha・wipeにも再利用する | Halftone／Ditherを小Filterとして保持し、wipeはTransition Kitへ接続 | 既存候補を強化 |
| [NTSC移植](https://github.com/sevenc-nanashi/ntsc-rs.anm2)、grunge、scan／bleed／noise系 | 古い映像・印刷物の素材感を小effectの組合せで作る | Grain、Scanline、RGB Split等 + VHS／NTSC Material Kit。時間窓が本質の成分だけL2 | 既存候補を強化 |
| [Radial／Rotational／Directional composite blur](https://github.com/sigma-axis/aviutl2_script_RadRotDirBlur_S) | 複数の方向モデル、色収差を一つの操作語彙で使う | 共通sampling／Host transient textureを使うBlur小Vism群。色収差とtransitionを本体へ固定しない | **新規候補群** |
| [Ground Shadow](https://github.com/sigma-axis/aviutl2_script_GroundShadow2_S)、長い影、neumorphism系 | alpha／shapeから立体感を持つ影や光を作る | typed alpha／path、投影面、方向から描くGround／Long Shadow。scene ownershipやlayer走査は持たない | **新規候補** |
| [Page Roll](https://github.com/sigma-axis/aviutl2_script_PageRoll_S) | 紙を丸めるような変形とscene change | 2 texture、progress、deformationを評価するTransition Vism。Timeline overlap／UndoはHost | **新規候補** |
| Kaleidoscope、Tile、Repeat、Point Zoom | 閉形式のsampling変形と反復 | Kaleidoscope／TileはSINGLE。Point Zoomはcamera操作ならHost、局所warpならWarp preset候補 | 既存候補を強化 |
| [FlowType](https://github.com/korarei/AviUtl2_FlowType_K_Script)、[Auto Lyric Animation](https://github.com/korarei/AviUtl2_AutoLyricAnimation_K_Script) | text animationの生成支援と、完成したlyric motion | authoring assistantはHost tool、評価表現はText Sequence／小Vism／Kitへ分離 | Kinetic Lyrics／Word Layoutを強化 |
| [Gradient Editor](https://github.com/azurite581/AviUtl2-GradientEditor) | 複数stopを視覚的に編集し、effectへ渡す | editorはHost parameter UI、Gradient Field／RampはVism provider | Gradient Fieldを追加 |
| [Object Motion Blur](https://github.com/korarei/AviUtl2_ObjectMotionBlur_LK_Script) | object motionに基づく基礎的な時間sampling | 通常用途はHost render基礎能力。意図的なEcho／Slit ScanだけTemporal Vism | Vismから除外 |
| Auto Clipping、Resize | 透明領域整理、sampling algorithm、領域管理 | Host render／bounds／quality基礎能力。個別表現Vismへしない | Vismから除外 |
| [ShowWaveform](https://github.com/hebiiro/AviUtl-Plugin-ShowWaveform)、camera transform、layer ordering | Timeline診断、操作支援、編集効率 | 操作UIはHost Timeline／command／authoring tool。camera観測modelは換装可能なCamera Providerであり、Host固定実装の根拠にしない | Vismから除外 |
| patch.aul、LuaJIT、plugin manager | runtime修正、高速化、配布・依存管理 | Host infrastructureとpackage lifecycleの先例。画を作るVismではない | Vismから除外 |
| L-SMASH Works、x264／NVEnc等の入出力、PSD読込 | codec／format bridge、入出力、asset連携 | Input／Delivery／Asset adapter。Vismのtexture Filter契約へ混ぜない | Vismから除外 |

この表は各repositoryの実装・parameter・UIを複製する採用表ではない。AviUtl文化から読むべき強いsignalは、巨大suiteよりも「不足を小scriptで埋め、それを組み合わせる」需要と、表現以外のpluginが同じ導入面へ集まるため分類が必要になる点である。

### 8.3 新規候補の実装見通し

#### Directional / Radial / Spin Blur

共通入力を`texture + sampling path + radius/angle + quality → texture`として比較し、一方向、中心放射、中心回転を別々のGPU／resource backdoorで実装しない。Draft／Finalは同じ関数のsample数または近似精度だけを変える。Host所有の一時textureとpass記述が閉じるまでは、plugin内texture pool、CPU readback、loop内resource生成で先行しない。

#### Ground / Long Shadow

最小形は`alpha/path + light direction + projection recipe → shadow texture`である。2D閉形式で反証し、必要なblurは共通Blur Vism、highlightとの組合せはKitへ分ける。AviUtlのobject／layerを走査する互換APIや、将来の3D scene／light ownershipをこの候補へ焼かない。

#### Gradient Field / Ramp

Vismは正準位置とstop列からcolorまたはscalar fieldを決定的に評価する。stopの追加・並べ替え・color picker・preset管理はHost parameter UIであり、Vism固有editorを必須契約にしない。creative colorの生成とrender直前の表示色変換を分離し、Gradient Map consumerはtyped texture／field port成立後に接続する。

#### ASCII Raster

入力のcellごとの明暗をglyph selectionへ写し、atlasから描画するL0表現として始める。font fileの動的探索、IME、文字編集、書記素animationを所有しない。glyph集合、atlas、cell grid、seed付きshuffleを個別入力として反証し、Text Sequenceと接続する場合もidentityを二重化しない。

#### Page Roll

最小形は`from texture + to texture + progress + fold recipe → output`である。2D／限定2.5Dの変形と陰影を同じfixtureで固定し、camera、汎用mesh、clip overlap、duration、selection、UndoをVismへ持たせない。2入力typed portとTimeline transition placementが決まるまではGate待ちとする。

### 8.4 並列実装と第三者接続への教訓

AviUtlの公開文化は、小さなscript、編集plugin、入出力plugin、runtime patchが独立配布され、利用者環境で並存する実例である。一方で共有runtimeへの暗黙依存、配置規則、patch前提、同じ名前空間への混在は、Motoliiがそのまま再現してよい契約ではない。

将来の各Vism実装laneは、同一Host能力を使っても互いの実装順へ依存しない。Blur、Gradient、Text、Transition等の共通能力は先に仕様IDとconformance fixtureを閉じ、個別Vismは公開されたversioned契約だけを消費する。第三者packageもfirst-partyと同じmanifest、capability、resource budget、failure isolation、typed port、determinism審判を通し、patch前提や具体plugin探索で接続しない。

### 8.5 非目標

- AviUtl／AviUtl2の全plugin、全script、配布siteを網羅した人気順位表にはしない。
- `.auf`、`.anm`、`.anm2`等の形式、Lua／HLSL API、parameter名、UI、配置規則をMotolii契約へ複製しない。
- Host基礎能力の不足を、first-party Vismの専用補助機能で隠さない。
- patchや共有runtimeを前提にした暗黙依存を第三者package modelへ持ち込まない。
- 候補追加だけを理由にDocument、Core enum、公開raw API、永続schemaを変更しない。
