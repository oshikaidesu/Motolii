# Vism作者journeyとshader依存closure

作成日: 2026-07-27

状態: **比較中**。現行の公開plugin境界とVism／Kitの既決を作者の実作業へ並べ直し、
推奨路、自由WGSL、外部shader依存の閉じ方を比較する。package schema、container、payload、
loader、User Library形式、新しい公開APIを決めず、実装を許可しない。

関連正本: [Vismコンセプト](../vism-package-concept.md)、
[Vism / Kitモデル](../vism-kit-model.md)、
[plugin authoring](../plugin-authoring.md)、
[Vism実装計画](2026-07-17-vism-implementation-plan.md)、
[依存優先・責任最小化ゲート](2026-07-24-dependency-first-responsibility-gate.md)。

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
| Kitのselection closureとatomic materializeは未成立 | [Vism / Kitモデル](../vism-kit-model.md)、`VSM-B2/B2I` | Kitの目標journeyと現在できることを分ける |

## 3. 決定、比較中、未決、停止線

### 3.1 決定 — 既決の作者journeyへの再投影

- Hostは時刻、型付き入出力、parameter、Quality、GPU resource、cache、評価順、診断を所有し、
  作者は表現計算を所有する。
- bindingとparameter意味は宣言契約が正本であり、WGSL sourceの文字列解析やreflectionから
  `NodeDesc`、UI、migration意味を逆算しない。
- shader compile／pipeline／backend binary等の生成物はHost所有の再生成可能cacheであり、
  Document、Project Lock、Kit、Vism source authorityにしない。
- Previewは通常の`render_frame(t, Quality)`系を通し、作者専用の第二rendererを作らない。
- local forkと上流、materialize後のProjectとKitは暗黙追従しない。
- Project openはnetwork、download、install、build、executeを起こさない。

### 3.2 比較中

| 主題 | 比較する候補 | 合格条件 |
|---|---|---|
| 標準operation | Host pass shapeだけ／version付きhelper source／authoring時生成 | 独自WGSLを拒否せず、Host責任と作者責任を混ぜない |
| shader closure | 保存時flatten／package-local module＋content hash／解決済みimmutable dependency | 別端末でglobal pathやnetworkなしに同じsource authorityを再構成できる |
| 外部shader import | GLSL／HLSL／ISF等をauthoring時変換／source copy／非対応 | lossと由来を示し、runtime adapterや黙った近似にしない |
| GPU互換 | 要求feature／limitとHost診断 | vendor／backend名で表現を分岐せず、非対応を型付き拒否する |
| local fork identity | 上流由来＋独立identity／version | 上流更新や同名導入で既存Projectの意味を変更しない |
| Kit selection closure | 選択対象、参照Definition、DataTrack、asset、scopeの包含／拒否表 | 名前検索やinstall pathから依存を推測しない |

closureは一fileへの文字列結合に限定しない。package-local moduleも保存時materializeも比較する。
禁止するのは未解決依存を作者端末やnetworkへ逃がすことであり、module性そのものではない。

### 3.3 未決

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
- WGSLからparameter、UI、binding、migrationを自動発明する。
- pluginへ`wgpu::Device`、raw pipeline／bind group mutation、backend／vendor／OS APIを公開する。
- shader module、pipeline、Naga IR、Metal／DXIL／SPIR-V等の派生物を恒久正本にする。
- compile失敗を黒frame、旧版への黙示fallback、近似shaderで隠す。
- local forkが上流identityを再利用し、上流更新がforkや既存Projectを自動変更する。
- WGSLをambient authority 0だけで「安全」と称し、hang、OOM、device loss、hard budgetを
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
out-of-treeでも実行できるHost側conformance入口までを仕様化する。この経路が成立しても
`.vism`、install、local save、hot reloadの成立とは称さない。

### 4.2 v2で比較するlocal WGSL Vism

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

各矢印の形式は`VSM-B4W`とPhase C/Dの審判前に決めない。source authorityは論理上
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
`VSM-B2`はselection closure表を意味fixtureで閉じられる。一方、`VSM-B2I`はatomic batchを待ち、
User Library ownerと保存形式も未決であるため、この文書で実装可能へ変更しない。

## 6. 他製品から移す仕組み

| 先例 | 確認できる仕組み | Motoliiへの転移 |
|---|---|---|
| [Blender OSL](https://docs.blender.org/manual/en/4.2/render/shader_nodes/osl.html) | source／bytecodeを内部data-blockへ保持できる一方、backend制約がある | source同梱とruntime capability診断を分ける |
| [Unreal plugin shader](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-shaders-in-plugins-unreal-engine) | Engine／Plugin仮想pathとplugin依存、compile input hash | 任意filesystemでなくpackage closureとcontent identityへ閉じる |
| [Godot shader preprocessor](https://docs.godotengine.org/en/4.4/tutorials/shaders/shader_reference/shader_preprocessor.html) | project resource内include、循環拒否、深度上限 | moduleを許す場合もbounded graphとして検査する |
| [Unity graphics API targeting](https://docs.unity3d.com/cn/2023.1/Manual/SL-ShaderCompilationAPIs.html) | Hostがtarget APIごとにcompileし非対応targetを宣言できる | backend binaryを正本にせず要求能力と診断を持つ |

巨大SDKやshader方言を採るのではなく、管理namespace、自己完結closure、Host compile、
派生cache、非対応診断という責任分離だけを転移する。

## 7. Opus 5 read-only相談の処分

2026-07-27に`claude-opus-5`へ現行docsとコードを渡し、編集・再委任なしで相談した。
外部出力は根拠ではないため、現行正本とコードへ再照合した。

| 助言 | 処分 | 理由 |
|---|---|---|
| 現行推奨APIをshader関数集でなくpass shapeと読む | **縮小採用** | `PipelineCache`とRadial Repeaterのコード事実に一致 |
| v1 source forkとv2 local Vismを分ける | **採用** | `'static` registry／cache keyと現行VSM gateに一致 |
| source authorityとbackend派生物を分ける | **採用** | Host cache、再現性、環境依存の責任分離に一致 |
| helper library／includeを全面禁止する | **棄却** | package-local closure／authoring時materializeならambient依存を避けられる |
| bindingをWGSL reflectionから作らない | **採用** | 宣言契約、migration、Host UI正本を守る |
| WGSLを他payloadより小さい粒で比較する | **縮小採用** | `VSM-B4W`を分けるがA8G3とPhase B gateを越えない |

## 8. 実装計画への配置と非目標

- `VSM-A4S`: v1 source forkとout-of-tree conformance入口。
- `VSM-A8G0`: first／third-party共通のHost pass shape。
- `VSM-B2`: Kit selection closure、公開control、asset要求、拒否表。
- `VSM-B4W`: WGSL closure、binding、source／派生物、fork identity、互換診断。
- `VSM-B5`: foreign shaderのruntime依存でなくauthoring時importのloss表。

この文書から公開trait、`PipelineCacheKey`、`NodeDesc`、Document、serde面を変更しない。
WGSL stdlib、include preprocessor、converter、`.vism`、Kit schemaを実装せず、
VSM-B2I、A8G1以降、Phase C/Dを解禁しない。
