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

Motolii側の正本は[Vism concept](../vism-package-concept.md)と[Vism実装計画](2026-07-17-vism-implementation-plan.md)である。v1のfirst-partyは静的plugin境界を外側から反証する**pre-Vism reference**であり、調査候補からpackage形式を逆算しない。

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
