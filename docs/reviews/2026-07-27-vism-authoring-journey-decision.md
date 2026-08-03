# Vism作者journeyとshader依存closure

作成日: 2026-07-27

状態: **比較中**。現行の公開plugin境界とVism／Kitの既決を作者の実作業へ並べ直し、
推奨路、自由WGSL、外部shader依存の閉じ方を比較する。package schema、container、payload、
loader、User Library形式、新しい公開APIを決めず、実装を許可しない。

関連正本: [Vismコンセプト](../vism-package-concept.md)、
[Vism / Kitモデル](../vism-kit-model.md)、
[plugin authoring](../plugin-authoring.md)、
[Vism実装計画](2026-07-17-vism-implementation-plan.md)、
[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)、
[悪性Vism封じ込めcontract](2026-07-26-vism-malware-containment-contract-decision.md)、
[External Authoring Bridgeの将来席予約](2026-07-29-external-authoring-bridge-seat-decision.md)。

## 1. 結論

Motoliiは「最小plugin APIさえあれば作者が自由に作れる」とは称さない。必要なのは、
制約の少ない実行外枠だけでなく、表現を作り、確認し、local成果へし、別Projectで再利用するまでの
作者journeyである。

推奨路は二層に分ける。

1. **Host pass shape**: binding、entry point、target、blend、resource寿命等、Hostが安全に実行する
   定型。現行`PipelineCache`に実在する舗装路である。
2. **標準operation／helper候補**: blur、noise、sampling、mask等を作者が再発明しないための
   authoring支援。具体語彙、version、module形式、materialize方式は未決である。

どちらも表現可能性の上限にしない。作者は許可されたpass shape上で独自WGSL本文を書ける。
一方、自由とは作者端末の絶対path、global shader search path、未固定library、実行時networkを
利用者へ暗黙に移す自由ではない。

このWGSL路を、Text、Path、Shape motionを含む全表現の単一入口にはしない。画素へ落とす前に
文字cluster、path vertex、open／closed、基本Shapeの編集意図、addressable instance identityを
保つ必要がある表現は、typed domain valueを扱う作者入口を使う。入口を分けることと、
`TextPlugin`、`ShapePlugin`等の表現名ごとの公開kindを増やすことは同義ではない。

```text
authoring frontend      payload class             semantic seat / typed value
WGSL / shader tool  ─┐  WGSL                  ─┐  texture / mask / field
Text motion tool    ─┼  declarative recipe    ─┼  text run / cluster / glyph
SVG / path tool     ─┼  source + import loss  ─┼  VectorRecipe / PathOp
math / data tool    ─┼  typed data / future code ┼ DataTrack / parameter
external DCC        ─┘  materialized asset     ─┘  texture / vector / baked track
                                      ↓
                    Host time / resource / cache / purity /
                    Preview-Export / diagnostics
```

三軸は非対称である。Text shapingはM5-P6のHost基礎能力、SVGはauthoring時import、
WGSLはpayload候補、Formula／inline expressionはPP-Gate待ち、外部素材は既決の正規経路であり、
同じ成熟度の五つのruntimeやSDKではない。既知の学習資産を活かすのは各入口のauthoring時であり、
外部runtimeの暗黙意味をMotoliiの保存・評価契約へ持ち込むことではない。
一般creator-authorがprogramを書く段の公式sourceは後続の
[言語境界決定](2026-08-01-vism-authoring-language-boundary-decision.md)でTypeScriptを採択したが、
本図のpayload class、semantic seat、inline式、engine、packageの未決を解消しない。
外部toolの現在選択をPush／Pullし、Create／Update／Forkへmaterializeする将来動線は
[External Authoring Bridge](2026-07-29-external-authoring-bridge-seat-decision.md)を正本とする。
外部DCC行をfile importだけへ閉じず、Bridgeのsession／permission／target解決をVism runtimeへ混ぜない。

```text
推奨operation / helper ─┐
package-local source ───┼─ authoring時に閉じたshader closure
独自WGSL本文 ───────────┘
                              ↓
                      Host validate / compile
                              ↓
                  通常Preview / Export共通評価
```

## 2. 現行コード事実

| 事実 | 証拠 | 意味 |
|---|---|---|
| Host cache keyは`id + &'static WGSL` | `crates/motolii-gpu/src/pipeline_cache.rs` | runtime所有shader textやlocal Vism loaderは現行公開面で表せない |
| Host pass shapeはfullscreen uniform16とtexture＋sampler＋uniform4の二つ | 同上 | 現行の推奨APIはshader関数集でなくbinding／pipeline定型である |
| Radial Repeaterはplugin所有の独自WGSL本文をHost pass shapeで実行する | `plugins/motolii-plugin-radial-repeater/src/lib.rs` | 「舗装路＋独自表現」のpre-Vism実証は既にある |
| plugin identity、registry、trait objectは`'static` | `crates/motolii-plugin/src/lib.rs` | 現行v1はworkspace build時の静的compositionである |
| first-party登録は明示composition root | `crates/motolii-plugins-firstparty/src/lib.rs` | source fork後は登録、rebuild、restartが必要 |
| 外部crate scaffoldは未実装 | [plugin authoring §0.1](../plugin-authoring.md#01-現在できるauthoringとまだ無いdistributionを分ける)、`VSM-A4S/A4I` | 現行`new-plugin.sh`を外部作者完成経路と称さない |
| WGSL hot reloadは未着手のdev経路 | [開発体験](../dev-experience.md) | compile失敗時last-good維持は方針だが製品作者UIはまだ無い |
| Kitのselection closure、atomic materialize、local保存形式は未成立 | [Vism / Kitモデル §5](../vism-kit-model.md#5-v1-kitはuser-libraryへ保存しmaterializeする)、`VSM-B2/B2I/B2S` | Kitの目標journeyと現在できることを分ける |

## 3. 状態別の境界

### 3.1 決定 — 既決の作者journeyへの再投影

- Hostは時刻、型付き入出力、parameter、Quality、GPU resource、cache、評価順、診断を所有し、
  作者は表現計算を所有する。
- bindingとparameter意味は宣言契約が正本であり、WGSL sourceの文字列解析やreflectionから
  `NodeDesc`、UI、migration意味を逆算しない。
- shader compile／pipeline／backend binary等の生成物はHost所有の再生成可能cacheであり、
  Document、Project Lock、Kit、Vism source authorityにしない。
- Previewは通常の`render_frame(t, Quality)`系を通し、作者専用の第二rendererを作らない。
- local forkと上流、materialize後のProjectとKitは暗黙追従しない。更新、再fork、mergeは
  明示操作にする。
- Project openはnetwork、download、install、build、executeを起こさない。

### 3.2 比較中 — package形式より先にfixtureで閉じる

| 主題 | 比較する候補 | 合格条件 |
|---|---|---|
| 標準operation | Host pass shapeだけ／version付きhelper source／authoring時生成 | 独自WGSLを拒否せず、Host責任と作者責任を混ぜない |
| shader closure | 保存時flatten／package-local module＋content hash／解決済みimmutable dependency | 別端末でglobal pathやnetworkなしに同じsource authorityを再構成できる |
| asset closure | package-local font／glyph／SVG等の同梱、content identity、Host解決 | system font、絶対path、外部参照、networkなしに同じ入力authorityを再構成でき、欠落・代替を診断する |
| typed中間値の寿命 | texture、text cluster／glyph、VectorRecipe／PathOp、DataTrack、instance set | rasterize前に必要なidentityを保持し、どの変換で何が失われるかをfixtureで説明できる |
| 外部shader import | GLSL／HLSL／ISF等をauthoring時変換／source copy／非対応 | lossと由来を示し、runtime adapterや黙った近似にしない |
| 外部vector／motion import | SVG／Lottie等をauthoring時に検証・正準化・materialize | Y-down／px／fps／外部参照等のlossを示し、外部runtimeを保存後の意味に残さない |
| GPU互換 | 要求feature／limitとHost診断 | vendor／backend名で表現を分岐せず、非対応を型付き拒否する |
| local fork identity | 上流由来＋独立identity／version | 上流更新や同名導入で既存Projectの意味を変更しない |
| Kit selection closure | 選択対象、参照Definition、DataTrack、asset、scopeの包含／拒否表 | 名前検索やinstall pathから依存を推測しない |

ここでいうclosureは「一fileへ文字列結合する」とは限らない。package-local moduleを保持する案も、
保存時に一つのsourceへmaterializeする案も比較対象である。禁止するのは未解決依存を作者端末や
networkへ逃がすことであり、authoring支援のmodule性そのものではない。

### 3.3 未決

- TypeScript作者profileのglobal allowlist、SDK module／entry signature、engine、payload lowering。言語方針とMTS-1候補基線は[言語境界決定](2026-08-01-vism-authoring-language-boundary-decision.md)で採択済み。
- 標準operationの語彙、source、version、互換方針、廃止手順。
- WGSL include／moduleの具体構文と、closureを保存する論理形式。
- `.vism` container、manifest field、payload、署名、loader、install store。
- local User Libraryのowner、schema、atomic write、fork／Kit保存形式。
- runtime所有WGSLを受ける公開型、binding宣言形式、compile admission、GPU deadline。
- 外部shader変換toolの採否と、由来／license claimを保持する形式。

### 3.4 停止線

- absolute path、home directory、環境変数、global shader search pathから実行依存を解決する。
- Project openを契機にnetwork取得、install、build、executeを行う。Preview開始時もnetwork取得、
  install、外部process、未解決の外部shader依存へ到達しない。Host shader compileのadmissionは
  GAP-30の別審判に従う。
- WGSLからparameter、UI、binding、migrationを自動発明し、宣言契約の正本にする。
- pluginへ`wgpu::Device`、raw pipeline／bind group mutation、backend／vendor／OS APIを公開する。
- shader module、pipeline、Naga IR、Metal／DXIL／SPIR-V等の派生物を恒久正本にする。
- compile失敗を黒frame、旧版への黙示fallback、近似shaderで隠す。
- Text／Path／Shapeのtyped identityが必要な表現を、結果が似て見えるWGSL pixel処理へ黙って
  rasterizeする。WGSLでpath風の画を描いた結果をPathOp、glyph風の画を描いた結果をTextとして
  再解釈しない。
- system font、font名だけの解決、SVG外部参照、Lottie runtimeをasset closureの代わりに使う。
- local forkが上流identityを再利用し、上流更新がforkや既存Projectを自動変更する。
- WGSL payloadをambient authority 0だけで「安全」と称し、hang、OOM、device loss、hard budgetを
  未証明のまま合格にする。

## 4. Journey A — 標準Filterをforkして独自WGSL表現を作る

### 4.1 v1で実証できるsource fork

これは一般利用者のlocal Vism authoringではなく、workspace作者経路である。

```text
first-party参照crateを選ぶ
  → 新しいcrate／identityへfork
  → 宣言parameterとWGSL本文を編集
  → conformance／purity／pixel oracle
  → first-party composition rootへ登録
  → workspace rebuild／restart
  → Host標準parameter投影と通常render経路で確認
```

`VSM-A4S`はRadial Repeater等をfork oracleにし、公開façadeだけを使う生成物、
out-of-treeでも実行できるHost側conformance入口、compile／test失敗時の診断までを仕様化する。
この経路が成立しても`.vism`、install、local save、hot reloadの成立とは称さない。

### 4.2 v2で比較するlocal WGSL Vism

目標journeyは次であるが、各矢印の形式は`VSM-B4W`とPhase C/Dの審判前に決めない。

```text
Filter型紙または既存Vismを選ぶ
  → 独立fork identityを採番
  → parameter契約とWGSL closureを編集
  → parse/validate
  → binding/conformance/予算を検査
  → last-goodを保つ通常Preview
  → local User Library artifactとして明示保存
  → 別Projectでpreflight後に利用
```

検査結果は少なくとも、shader構文／型、Motolii契約適合、GPU互換、供給元／由来、安全性を
一つの「valid」へ潰さず分ける。source authorityは少なくとも論理上
`WGSL closure + pass-shape要求 + 宣言binding + 表現identity/version`から再構成できる必要がある。
backend生成物は端末、adapter、driver、wgpu／compiler versionに依存するため、消して再生成できる
cacheに閉じる。

## 5. Journey B — 既存Vismをlocal Kitへする

```text
Project内の接続済み構成を選ぶ
  → selection closureを表示
  → Vism要求、型付き接続、初期値、公開control、asset要求をpreflight
  → local User Libraryへatomic save
  → 別Projectで依存／欠落／互換を再preflight
  → 全commandを開始snapshotへ検証
  → 1 macro commitでmaterialize
```

Kitはshader sourceを再解決する第二package managerにならない。Kitが持つのはVism要求と型付き接続であり、
絶対path、install store path、表示名検索、利用者端末のregistry状態を意味にしない。
選択がGroup／scope境界をまたぐ、必要DefinitionやDataTrackを閉じられない、asset identityが
不明等の場合は、暗黙に範囲を広げず包含候補と拒否理由を表示する。

`VSM-B2`はselection closure表を意味fixtureで閉じられる。一方、`VSM-B2I`はatomic batch、
`VSM-B2S`はUser Library ownerと保存形式を待つため、この文書で実装可能へ変更しない。

## 5.5 作者インターフェース群 — Text／Path／Input／Instanceを画素へ潰さない

### 5.5.1 答え

別の**作者入口**は必要である。ただし、入口ごとに別のHost、保存世界、実行scheduler、
万能SDKを作らない。作者が扱うsource assetとtyped中間値は分け、評価時の時刻、resource、
cache、純関数、Preview／Export、診断は共通Hostへ合流させる。

| 作者入口 | 活用する既知資産 | 保持するtyped値／identity | WGSLへ早期変換すると失うもの | 現行の置き場 |
|---|---|---|---|---|
| Shader／pixel | WGSL、WebGL／GLSL／HLSL／ISFの知識とsource | texture、mask、field、declared buffer | vector／textの編集identityは元から所有しない | Host pass shape、VSM-B4W/B5 |
| Text motion | OpenType、HarfBuzz系shaping、typography／kinetic typeの知識 | run、cluster対応、glyph、style span、selector対象 | 文字・単語・行の対応、再shape要否、font fallback診断 | M5-P6、Text Sequence／Effector比較 |
| Vector／Path | SVG、Lottie、Bezier、path animationの知識 | 基本Shape意図、VectorRecipe、vertex、open／closed、PathOp順序 | 頂点、first vertex、trim／offset順序、再編集可能性 | GAP-15、既決PathOp、authoring時import |
| Input／data provider | MIDI、BPM、tap、解析、外部eventの知識 | DataTrack、event、field、source時刻と由来 | event identity、note duration、provider差替え、再現可能な記録 | ParamDriver、VSM-B2、Host Input Adapter比較 |
| Instance／particle | particle、cloner、Geometry Nodes、procedural instanceの知識 | stable InstanceId、birth、lifetime、source、per-instance channel | 個体追従、seed、選択、release、2D／3D差替え | LayerSource、M5-P0I/P7、Simulation Bake |
| Motion／relationship | easing、expression、DataTrack、procedural motionの知識 | typed parameter、DataTrack、scope、stable identity | 値の由来、対象、循環、Undo／再評価意味 | ParamDriver、Behavior／Authoring Tool比較 |
| External／Bake | DCC、SVG、glTF、画像／動画、simulation asset | 正準化済みassetまたはHost所有bake | source側の編集構造。lossを明示して選ぶ | Materialize／External／Simulation Bake |

「Motion」は単独の描画backendではない。Text cluster、path vertex、instance、通常parameter等の
対象へ、時刻とtyped valueをどう作用させるかという横断責任である。一回限りの配置やkey生成は
Authoring Tool、入力変更後も続く関係はBehavior／Driver、独自recipeはGenerator、
逐次状態はHost所有Bakeへ送る。新しい`TextAnimatorPlugin`や`ShapeMotionPlugin`をこの文書から
逆算しない。

同じく`PathPlugin`、`MidiPlugin`、`ParticlePlugin`という表現名kindを上表から逆算しない。
Pathはtyped vector identity、MIDIはproviderと記録済みevent、Particleは閉形式Instanceまたは
Host所有Simulationという責任へ分ける。作者が見る大分類とHost内部の実行kindは直交し、
一つのVismが複数の入口をKitで接続してもよい。第三者向けの短い選択手順は
[plugin authoring §1.2](../plugin-authoring.md#12-作者が最初に選ぶインターフェース群)を正本とする。

### 5.5.2 ユーティリティの昇格条件

ユーティリティは、WGSLで書きにくい処理だけでなく、WGSLへ書くとtyped意味を失う処理も対象にする。

- Shader側: blur、scan／compaction、reduction、distance field、sampling、deterministic noise、
  stroke／MSDF raster、Host所有multi-pass。
- Text側: itemize、fallback、shape、cluster対応、glyph transform。行組、selector、
  stagger、word／line timingは交換可能な上位recipeに残す。
- Vector側: Bezier評価、arc length、flatten／tessellate、stroke、boolean／PathOp。
  基本Shapeを早期に汎用pathへ潰さず、rasterizeは最後にする。
- Motion側: easing、正準座標変換、stable identity由来seed、selector／falloff、
  typed DataTrack接続。

共通化の判定は「LLMが生成しやすい」だけにしない。複数domain／作者が反復し、意味と負例が安定し、
Host所有resourceまたはidentityを必要とし、version付きfixtureで交換可能な時だけ舗装路へ昇格する。
推奨utilityを使わない独自表現を拒否せず、使用operationから導出できるcapability facetと、
作者承認が必要な作風tagを混同しない。

### 5.5.3 LLMと既知資産の活用

LLMにはWGSLだけを教えるのではなく、既知sourceからMotoliiのtyped意味へ翻訳するloss表と、
正しい参照実装を与える。

- WebGL／GLSL／ShaderToy系はauthoring時にWGSL closureへ移し、座標、色、binding、
  precision、unsupported featureを診断する。
- SVG／Lottie系はruntime互換層にせず、import時に正準座標、時間、PathOp、
  対応／拒否要素を明示してmaterializeする。
- text系はHTML canvasやshader文字列へ縮約せず、P6のrun／cluster／glyph語彙へ接続する。
- JS／p5／expression系は既存のMaterialize／Pure Live／Bake分類へ翻訳する。program authoring sourceの
  TypeScript採択からFormula／inline式や汎用live runtimeを逆算せず、後者はPP-Gate前に確定しない。

入口を増やす目的は独自言語を増やすことではない。外部の厚い学習資産をauthoring時に借りながら、
Motolii内部のtarget語彙、fixture、診断を少数のtyped境界へ収束させることである。

## 6. 他製品から移す仕組み

他製品も依存を消しておらず、Host管理の言語、namespace、package、compilerへ閉じている。

| 先例 | 確認できる仕組み | Motoliiへの転移 |
|---|---|---|
| [Blender OSL](https://docs.blender.org/manual/en/4.2/render/shader_nodes/osl.html) | source／bytecodeを内部data-blockへ保持できる一方、backend制約がある | source同梱とruntime capability診断を分ける |
| [Unreal plugin shader](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-shaders-in-plugins-unreal-engine) | Engine／Plugin仮想pathとplugin依存、compile input hash | 任意filesystemでなくpackage closureとcontent identityへ閉じる |
| [Godot shader preprocessor](https://docs.godotengine.org/en/4.4/tutorials/shaders/shader_reference/shader_preprocessor.html) | project resource内include、循環拒否、深度上限 | moduleを許す場合もbounded graphとして検査する |
| [Unity graphics API targeting](https://docs.unity3d.com/cn/2023.1/Manual/SL-ShaderCompilationAPIs.html) | Hostがtarget APIごとにcompileし非対応targetを宣言できる | backend binaryを正本にせず要求能力と診断を持つ |

これらの巨大SDK、shader方言、plugin directoryをそのまま採らない。転移するのは、管理namespace、
自己完結closure、Host compile、派生cache、非対応診断という責任分離だけである。

## 7. Opus 5 read-only相談の処分

2026-07-27に`claude-opus-5`へ現行docsとコードを渡し、編集・再委任なしで相談した。
外部出力は根拠ではないため、次のように現行正本とコードへ再照合した。

| 助言 | 処分 | 理由 |
|---|---|---|
| 現行推奨APIをshader関数集でなくpass shapeと読む | **縮小採用** | `PipelineCache`とRadial Repeaterのコード事実に一致 |
| v1 source forkとv2 local Vismを分ける | **採用** | `'static` registry／cache keyと現行VSM gateに一致 |
| source authorityとbackend派生物を分ける | **採用** | Host cache、再現性、環境依存の責任分離に一致 |
| helper library／includeを全面禁止する | **棄却** | package-local closure／authoring時materializeならambient依存を避けられ、推奨operationの比較を不必要に閉じる |
| bindingをWGSL reflectionから作らない | **採用** | 宣言契約、migration、Host UI正本を守る |
| WGSLを他payloadより小さい粒で比較する | **縮小採用** | `VSM-B4W`を意味fixtureとして分けるが、A8G3とPhase B gateを越えない |
| Text／PathはWGSLの不便さでなくtyped identity lossとして分ける | **採用** | P6 cluster対応、VectorRecipe、PathOp、instance identityの既決に一致 |
| `seat × payload class × authoring frontend`の三軸で整理する | **採用** | 新しい表現名kindを増やさず、既存B4/B5とHost capabilityへ配置できる |
| font／SVG／glyphも自己完結asset closureとして比較する | **採用** | shaderだけ環境依存を閉じてtext／vectorから再流入する穴を防ぐ |
| LLM corpusを理由に対等なruntime入口を増やす | **棄却** | 外部意味の誤差とMotolii target語彙の分裂を招く。authoring時loss表で回収する |

## 8. 実装計画への配置

- `VSM-A4S`: first-party source forkからrebuild／restartまでのv1作者journeyと、
  out-of-tree conformance入口を仕様化する。
- `LANG-TS-F0/F1`: TypeScript作者profileの意味・診断fixtureと、C2後のfeedback／回復fixtureを分ける。
  F0合格をlive runtime、package、製品対応の証拠にしない。
- `VSM-A8G0`: 単一Glow APIでなく、first／third-partyが同じHost pass shapeを使う境界として
  multi-pass、HDR、typed texture／mask、budgetを締結する。
- `VSM-B2`: Journey Bのselection closure、公開control、asset要求、拒否表を閉じる。
- `VSM-B4W`: WGSL payloadだけの意味fixture。closure、binding適合、source／派生物、
  fork identity、互換診断を比較する。A8G3とB0/B1前に締結しない。
- `VSM-B4`: payload分類をWGSLの結果だけで一般化せず、P6を消費するText declarative recipeと、
  既決`Vec<PathOp>`を合成するPath declarative recipeを意味fixtureへ含める。新しいkindは作らない。
- `VSM-B5`: GLSL／HLSL／ISF、SVG／Lottie等をruntime依存にせず、authoring時importの
  loss／由来／license／近似拒否表として比較する。

## 9. 非目標

- この文書から公開trait、`PipelineCacheKey`、`NodeDesc`、Document、serde面を変更しない。
- WGSL stdlib、include preprocessor、foreign shader converterを実装しない。
- `.vism`、Kit、User Libraryのfile／schema／fieldを予約しない。
- WGSLを唯一の将来payload、標準operationを必須の表現語彙と決めない。
- `TextAnimatorPlugin`、`ShapePlugin`、`PathPlugin`等の表現名kindを本書から追加しない。
- P6のText基礎API、既決PathOp閉集合、GAP-15の未決Shape語彙を入口都合で変更しない。
- SVG／Lottie／system font／WebGL runtimeを保存後の評価依存にしない。
- shader tagを使用関数だけから公開metadataへ自動確定しない。
- VSM-B2I/B2S、A8G1以降、Phase C/Dを解禁しない。
