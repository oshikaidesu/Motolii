# Vism表現候補の横断stress test — AviUtl／Blender／TouchDesigner／Cavalry／GitHub

状態: **観察**

日付: 2026-07-29

## 1. 目的

既存の[Vismプラグインカタログ](../vism-plugin-catalog.md)を、AviUtl、Blender、
TouchDesigner、Cavalry、GitHub上のshader／creative coding資産から横断して見直し、
既存候補の別名ではない表現族と、第三者作者が専用裏口なしで実装するために不足する席を探す。

これは人気順位、採択、実装順、公開APIの決定ではない。外部製品名、parameter、node graph、
shader sourceをMotoliiへ複製せず、次だけを観察する。

- 何を入力し、rasterize前までどのidentityを保つか
- `f(t)`、有限時間窓、Feedback、Simulation／Bakeのどこへ置くか
- 一つの完成effectか、複数表現へ再利用できるprimitiveか
- 現行の作者インターフェース群と実装laneだけで表現できるか
- Host、Adapter、Authoring Toolへ残すべき責任は何か

外部repositoryはREADME、公開file名、公式仕様だけを確認した。sourceの採用、翻訳、port、
vendorは行っていない。特にLYGIAはProsperity／Patronのdual licenseであり、名前と分類の
観察を依存・copy許可へ読み替えない。各assetの利用を比較する場合は
[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)と
[参考ライブラリ一覧](../references.md)でlicenseを個別裁定する。

## 2. 現行カタログとの差分

現行カタログにはGlow、Pixel Sort、Dither、Halftone、Displace、Fractal Field、
Text、Particle、Connected Points、MIDI、Temporal effect、Transition等が既にある。
今回それらを候補数として数え直さない。

現行の作者入口はFilter／Pixel、Source／Generator、Path／Vector、Input／Data、
Mapping／Behavior、Instance／Particle、Text、Simulation／Bake、Tool／Bridgeである。
これは新しいRust traitや`PluginKind`ではなく、作者が責任を選ぶ入口分類である。

一方、カタログの実装laneは`SINGLE / PORTS / MULTIPASS / BAKE / TEXT / TEMPORAL /
SIM / KIT`であり、次はまだ明示的な席を持たない。

1. Pathを受け、Path identityを保ったままPathを返す処理
2. RGBA以外のDepth、ID、Motion Vector、Distance Field等を出すprovider
3. 前回出力を明示状態として読むFeedback
4. DataTrack／event／fieldを受け、別のtyped dataへ変換する処理
5. 3D Surface／Material／Geometryをrasterize前に保つ表現

この五つは直ちに新しいkindを要求するという結論ではない。既存seatへ正規化できるか、
Host capability moduleまたはtyped artifactとして持つか、新しい公開境界が本当に必要かを
反対側レビューで比較するための空白である。

## 3. 外部文化から確認できた構造

### 3.1 AviUtl — 面白さはeffect数だけでなく、一つのPathの多用途化にある

[Path_S](https://github.com/sigma-axis/aviutl2_script_Path_S)は、折れ線／Bezierの描画、
Path内mask、Path線上mask、Path範囲への後続filter、Path沿い配置を同じscript群で扱う。
重要なのは個別名称より、**一つのPath sourceを描画、scope、mask、instance配置へ再利用する**
構造である。Motoliiで各用途を別のpixel effectとして実装すると、vertex、open／closed、
first point、arc length、trim順序が失われる。

[AviUtl油絵script](https://github.com/karoterra/aviutl-OilPainting)はKuwahara filterを
完成した作風として提供する。これはGlow等と重ならない小さいsignature Filter候補である一方、
近傍半径、Draft縮退、境界sample、GPU予算を明示する必要がある。

[AviUtl／AviUtl2 script topic](https://github.com/topics/aviutl2-script)には、基礎shape、
stylize、camera tool、blur、Path、page roll等が同居する。従って「scriptとして配布される」
こと自体をVism判定に使わず、表現、Host操作、Adapter、Infrastructureへ責任分解する既存方針を
維持する。

### 3.2 Cavalry — FalloffとPath identityが複数表現の共通語彙になる

Cavalryの[Falloff](https://cavalry.studio/docs/nodes/utilities/falloff/)はcircle、rectangle、
linear、sweep、Shape edge等から値を作り、多数のBehavior／Fieldのstrengthへ共通接続する。
[Range Falloff](https://cavalry.studio/docs/nodes/utilities/range-falloff/)はindex範囲、
percentage、transitionを扱う。これはText、Path、Instanceごとに別のstagger機構を作るより、
stable identityへ作用するtyped Field／Mappingを共有する需要を示す。

[Trails](https://cavalry.studio/docs/nodes/shapes/trails/)は動くShapeからBezierまたはlineの
軌跡を生成する。これは単なるpixel Echoではなく、後段でstroke、mask、instance配置へ使える
Path出力である。必要な履歴が閉形式のmotionから直接sampleできるか、宣言時間窓が必要かを
分けなければならない。

[Shapes](https://cavalry.studio/docs/nodes/shapes/)はPath、Contour、child Mesh、Textの
line／word／character hierarchyをraster前に保持する。
[Path Distribution](https://cavalry.studio/docs/nodes/general/distribution-types/path-distribution/)
もPath全体、sub-mesh、contourを区別して配置する。早期rasterizeは、この区別を失う。

[Light Sweep](https://cavalry.studio/docs/nodes/effects/filters/light-sweep-filter/)、
[Scrape](https://cavalry.studio/docs/nodes/effects/filters/scrape-filter/)、
[Edge Detection](https://cavalry.studio/docs/nodes/effects/filters/edge-detection-filter/)は
完成filter候補になる。一方、[Voronoi Shader](https://cavalry.studio/docs/nodes/effects/shaders/voronoi-shader/)
と[Shape to Shader](https://cavalry.studio/docs/nodes/effects/shaders/shape-to-shader/)は、
patternやShape由来textureを複数consumerへ渡すprovider／material語彙の需要を示す。

### 3.3 Blender — Geometry、Path、Materialを最後まで別物として扱う

Blender Geometry NodesはCurve、Instance、Mesh、Point、Volume、Materialを別domainとして持ち、
Curveではtrim、resample、fill、curve-to-mesh、curve-to-points等を分けている。
[Geometry Nodes一覧](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/index.html)
からMotoliiへnode systemを移植するのではなく、Path→Path、Path→Point、Path→Geometryを
同じRGBA出力へ潰さない必要を読む。

[Grease Pencil Line Art](https://docs.blender.org/manual/en/4.2/grease_pencil/modifiers/generate/line_art.html)
はscene／object geometry、camera、occlusion、contour、crease、intersection、material border、
light contourから編集可能なstrokeを作る。画像のSobel edgeとは別の、
**Geometry／Depth／CameraからPathを生成する表現**である。

BlenderのmaterialはSurface、Volume、Displacementを分け、procedural textureを3D座標で
評価する。[Voronoi Texture](https://docs.blender.org/manual/en/5.0/compositing/types/texture/voronoi.html)
も1Dから4Dまでの座標、cell位置、edge距離等を出す。Motoliiが将来shader sourceを受ける場合、
full-screen texture Filterと、surface position／normal／UV／view／lightを入力にするMaterialを
同じ入口とみなせない。

[Simulation Zone](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/simulation/simulation_zone.html)
は前frame状態、sub-step、cache、disk bakeを明示する。ここからnode graphやcache形式を採らず、
逐次状態をFilterへ隠さずHost所有Bakeへ置く既存方針と照合する。

### 3.4 TouchDesigner — textureは画だけでなくfield、point、状態も運ぶ

[Feedback TOP](https://docs.derivative.ca/Feedback_TOP)は下流のTarget TOPを前回状態として読む。
これは有限lookbehindとは異なる再帰であり、Motoliiでは通常Filterの`&self`へ隠さず、
明示初期値、step、checkpoint、reset、無効化をHost所有にする必要がある。

[Optical Flow TOP](https://docs.derivative.ca/Optical_Flow_TOP)は動きを2channelのvector fieldとして
出し、particle forceへ接続できる。特定GPU／OS制約も記載されており、Motoliiではvendor APIを
plugin契約へ出さず、Analysis／Bake providerとportable consumerを分ける反例になる。

[particlesGpu](https://docs.derivative.ca/Palette%3AparticlesGpu)はposition、color、velocity、
optical flow、effectorを別texture入力にする。[Point Clouds](https://docs.derivative.ca/Point_Clouds)
はfloat textureをXYZ／attributeのcarrierとして扱う。MotoliiはRGBAというstorage shapeから
意味を推測せず、Point Set、Vector Field、Instance channel等のtyped payloadを維持する必要がある。

[GLSL Multi TOP](https://docs.derivative.ca/GLSL_Multi_TOP)はmulti-input、compute、
3D texture、複数color buffer、read-write出力を持つ。これは任意shaderへ同等のambient authorityを
与える根拠ではなく、multi-pass／compute／typed input／budgetをHostが宣言的に所有する必要の
stress caseである。

### 3.5 GitHub — effect collectionより再利用可能な小語彙が生態系を作る

[gl-transitions](https://github.com/gl-transitions/gl-transitions)は`from`、`to`、`progress`、
viewport ratioという小さい契約で多数のtransitionを集める。個別transitionをすべてfirst-partyへ
移すより、2 texture＋progressの共通境界と作者conformanceを先に閉じる既存方針と一致する。

[ISF-Files](https://github.com/Vidvox/ISF-Files)はISF 2.0に従う200超のgenerator／filterを
一つのcollectionへ置く。これはshader payloadとmetadataをHostが理解できる場合に多数作者が
増やせる観察であり、ISF runtimeやfilesystem配置をMotoliiへ採る決定ではない。

[LYGIA](https://github.com/patriciogonzalezvivo/lygia)はfilter、generative、morphological、
SDF、simulation等を細粒度関数へ分け、WGSLを含む複数言語版を持つ。公開treeにはKuwahara、
Voronoi、jump flood、marching squares、Gray-Scott、ripple、fluid、SDF boolean等の系統がある。
ここからcodeを採らず、Distance Field、morphology、Feedback Simulationが複数の完成表現を
生む共通primitiveであることだけを調査候補へ置く。

## 4. 横断候補

次表の優先度は**調査価値**であり、採択順または実装順ではない。

| 候補 | 主な文化 | 既存候補との差 | 作者インターフェース／時間 | 必要なgate | 調査価値 |
|---|---|---|---|---|---|
| **Path Region / Local Effect Mask** | AviUtl Path_S | texture maskでなく、Path内／stroke上／後続effect scopeを同じPathから作る | Path／Vector → typed mask、L0 | GAP-15、PORTS、scope所有 | 高。Path再利用の最小fixture |
| **Pattern Stroke / Path Ribbon** | AviUtl、Cavalry | Energy Strokeの発光を除き、dash、wave、arrow、trim、widthをPathのまま保持 | Path／Vector → Path／Geometry、L0 | Path→Path／Geometry席 | 高。早期rasterizeの反証 |
| **Painterly Kuwahara / Oil Paint** | AviUtl、GitHub shader | Grain／Halftoneと異なるedge-preserving painterly stylize | Filter／Pixel、L0、必要時MULTIPASS | GPU近傍sample、Draft、budget | 高。小さいsignature候補 |
| **Light Sweep / Specular Sweep** | Cavalry | Rays／Flareでなく、surface上を移動する限定的highlight | Filter／Pixel、L0 | SINGLE、将来mask PORTS | 中。近いwaveの完成effect |
| **Spatial / Index Falloff Field** | Cavalry | 各Text／Particle固有staggerでなく、Shape距離とstable indexへ共通作用 | Mapping／Behavior、L0 | typed field、P0I、Text identity | 最優先primitive |
| **Motion Trails as Path** | Cavalry | Echoのpixel履歴でなく、後段で再利用できるPathを生成 | Path／Vector、L0またはL2 | Path出力、TEMPORAL | 高。Pathと時間窓の交点 |
| **Cellular / Voronoi Field** | Blender、Cavalry、GitHub | Fractal Fieldと異なりcell ID、feature point、edge distanceを出せる | Source／Generator → Field、L0 | typed multi-output field | 高。material／instanceへ再利用 |
| **Distance Field / Morphology Provider** | GitHub shader、既存Simulation設計 | Shape Fill内部だけでなくoutline、stroke、collider、maskへ共有 | Source／Generator／Bake、L0またはBake | PORTS、BAKE、SDF意味fixture | 最優先primitive |
| **Geometry Line Art** | Blender | raster edge detectでなく、geometry／depth／cameraからPathを作る | Geometry input → Path、L0またはBake | M5 Observation／Depth、補助pass | 高。2D／3D接続のsignature |
| **Curve Sweep / 3D Ribbon** | Blender Geometry Nodes | 2D strokeでなくPathをtube／ribbon geometryへ変換 | Path／Vector → Geometry、L0 | M5 Geometry／Material | 高。Path Vismの3D出口 |
| **Procedural Surface Material** | Blender、GitHub shader | full-screen Filterでなくsurface contextを読む | Material／Surface候補、L0 | M5 material、camera／light authority | 最優先の将来席stress |
| **Optical Flow Vector Field** | TouchDesigner、ISF | pixel effectでなく解析結果を複数consumerへ供給 | Input／Data + Analysis／Bake | typed vector field、BAKE | 最優先provider |
| **Flow Advection / Particle Force** | TouchDesigner | Optical Flowの推定と見た目を一体化せず、field consumerにする | Mapping／Instance、L0またはL3 | P0I、Field input、SIM | 高。provider差替えを反証 |
| **Declared Feedback Canvas** | TouchDesigner、p5.js | Echoの有限入力窓でなく前回出力が意味そのもの | Feedback／Simulation Bake、L3 | reset、checkpoint、cache、budget | 最優先の時間空席 |
| **Reaction–Diffusion / Ripple / Fluid Field** | GitHub shader、TouchDesigner | Fractal Fieldの無状態noiseでなく逐次field | Simulation／Bake、L3 | Declared Feedback／SIM | 高。Feedback席のsignature |
| **Point Cloud Visualizer / Ribbons** | TouchDesigner、Blender | Connected Pointsの生成元を2D pointだけに限定せず、XYZ／attributeを保持 | Input／Data → Instance／Geometry | M5 P0I、Point Set、Material | 高。外部3D dataのMV出口 |
| **SDF Morph / Metaball Composition** | GitHub shader | pixel Warpでなくshape fieldのunion／subtract／smooth morph | Field／Path／Generator、L0 | Distance Field、typed output | 高。小Vism構成のsignature |
| **DataTrack Operator** | TouchDesigner CHOP型dataflow、MIDI／BPM | provider→consumerの間でlag、quantize、limit、derivative等を行う | Input／Data → Mapping／Data、L1 | data→data席、identity／sample意味 | 高。MIDI／音反応の共通穴 |

## 5. 優先して閉じる問い

候補名を増やす前に、次の順で意味fixtureを作れるか比較する。

1. **Path→Path／Path→typed mask**
   同じPathを描画、mask、scope、instance配置へ使い、vertex／contour／arc lengthを失わないか。
2. **補助typed output**
   Distance Field、Depth、Motion Vector、Instance IDをRGBAというstorage形状から推測せず、
   producerとconsumerを差し替えられるか。
3. **Declared Feedback**
   有限時間窓と前回出力の漸化式を分け、状態、reset、checkpoint、無効化、予算をHostが所有できるか。
4. **Data→Data**
   MIDI／BPM／解析eventをparameterへ直結するだけでなく、typed dataのまま整形、合成、
   resampleしてから複数consumerへ渡せるか。
5. **Surface／Material／Geometry**
   Blender shaderを「GLSLをWGSLへ変換する問題」に縮小せず、camera、depth、normal、UV、
   light、object identityのauthorityをM5に残せるか。

`Procedural Surface Material`とraymarch系shaderは同じではない。前者はHostのworld／camera／surface
contextを読む候補、後者はshader内に独自camera、ray、depth、worldを持ち込みやすい反例である。
raymarch sourceを受ける場合は、M5のcamera／Observation、Depth policy、Freeze cache keyと
矛盾しないことを先に審判し、独自cameraを通常Vismの隠れauthorityにしない。

## 6. Opus 5 read-only相談の処分

2026-07-29に`claude-opus-5`へ現行docsを渡し、編集、再委任、実装なしで横断候補と
分類漏れを相談した。外部候補の記憶は根拠にせず、上記の公開一次資料と現行正本へ再照合した。

| 助言 | 処分 | 理由 |
|---|---|---|
| 完成effect増設より、primitiveを主、3D／Feedbackは席予約、各waveにsignature一個とする | **縮小採用** | 現行barbell方針とlane再利用に一致するが、primitive実装はまだ許可しない |
| 補助typed pass、Feedback、Path→Path、Data→Dataが空席 | **観察として採用** | 現行カタログ、作者入口、正規5経路の差分に一致 |
| Distance Fieldを複数effectの共通前提として比較する | **観察として採用** | Shape Fill、collider、outline、strokeで重複可能性がある |
| Speed Lines、Object Split、Posterize Time、Audio Waveformを追加候補にする | **延期** | 今回確認した一次資料だけでは実作品上の役割とHost責任の切分けが不足 |
| raymarchを候補にする | **棄却** | 採択候補でなくcamera／depth authorityを破る負例probeとして扱う |
| 今回の候補を直ちに一般カタログへ追加する | **棄却** | 共通seat／laneが未決で、個別候補から公開境界を逆算する危険がある |

## 7. 次の調査

1. 本観察に対し、席の増設がCoreへの表現集積の別形にならないかを反対側レビューする。
2. Path Region、Spatial Falloff、Distance Field、Declared Feedback、DataTrack Operator、
   Procedural Surface Materialの六fixtureを、公開型を決めないloss tableとして比較する。
3. 反対側レビュー後に、採択、縮小、延期、棄却を個別に処分する。
4. 採択された完成effectだけをVismカタログへ追加し、共通能力はlane／Host capability側へ置く。
5. AviUtl固有候補はplugin名でなく、MV実作品で何のidentityを作り何と合成したかを標本化する。

## 8. 非目標とSTOP

- 本書から`PluginKind`、trait、`NodeDesc`、`FrameDesc`、Document、serde面を変更しない。
- `PathPlugin`、`FeedbackPlugin`、`MaterialPlugin`等のkindを候補名から逆算しない。
- Depth、ID、Motion Vector、SDFを一つの曖昧な「aux texture」へ潰さない。
- Blender／TouchDesignerのnode graph、world、camera、cache、device APIをHostへ移植しない。
- GLSL、HLSL、ISF、LYGIAを保存後runtime依存または無審査source copyにしない。
- FeedbackをFilter内の前frame texture、static、global、`&self`の隠れstateで近似しない。
- Geometry Line ArtをSobel edge、Optical Flowをframe差分だけで完成扱いしない。
- 反対側レビューとseat処分前に、本書の候補を採択済みまたは実装可能としてcatalogへ登録しない。
- 未決のM5 Camera／Depth／Geometry／Material契約をshader import都合から変更しない。
