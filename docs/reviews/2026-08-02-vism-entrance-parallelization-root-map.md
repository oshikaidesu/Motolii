# Vism入口・並列解禁の根本マップ（2026-08-02）

状態: **決定**。

本書は[Vism実装計画 §8.1](2026-07-17-vism-implementation-plan.md#81-2026-07-23再基線の次の五手)が要求する
全体レビューとして、Vismの各入口を既存authority、target、owner、write routeへ接続し、何が仕様化でき、
何が依存待ちかを固定する。未決API／schemaを補完せず、実装順、依存、停止線、Phase C入場条件だけを扱う。
各入口の既知解、採用方式、private境界、probe、cutover、retirementは
[Vism既知実装採択マップ](../vism-known-implementation-adoption-map.md)を正とする。

## 1. 結論 — 固まった根と未閉鎖の根を分ける

### 1.1 意味の根は決定済み

全入口に共通する次の根は決定済みであり、入口ごとに再決定しない。

- semantic typed value、決定的operation、typed failure、declared capabilityを共通骨格とする。
- finite value、単位、space、time／duration、position／directionを暗黙変換しない。
- typed inputはimmutableで、同じ入力は同じ結果を返す。ambient scene／filesystem／network／GPU ownershipを与えない。
- resource、budget、camera、depth、adoption、StateTrackはHostが所有し、Document変更はsingle writerだけが行う。
- frontend、作者source、runtime seatが異なっても、一つのVism identity、Inspector、Previewを保つ。

正本は[Vism意味SDK決定](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md)、
[Inspector／source／Automation境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md)、
[作者言語境界](2026-08-01-vism-authoring-language-boundary-decision.md)である。

### 1.2 並列施工の根は未閉鎖

意味の根が決定済みでも、複数入口の実装を同時解禁する根はまだ閉じていない。

1. `VSM-A4I`: 外部crate作者scaffoldを実装する。
2. `VSM-A5`: kind横断missing／future／unknown round-trip matrixを閉じる。
3. `VSM-B0 → B1 → B2`: identity、成果物分類、provider／consumer／materialize方式を順に決める。
4. `VSM-A9`: 二つ以上のfixture pluginを同時追加し、他Vism source／test変更0、private依存0、競合のtyped rejectを証明する。

`VSM-A3`は独立LayerSourceの成立過程で共有Host API `PipelineCache::get_or_create_fullscreen_uniform16`の
追加を要した。これは「既存traitがある」だけでは共有境界が閉じない実例である。したがって`VSM-A9`合格前に、
どの入口にも「複数Vismを並列実装可能」という判定を置かない。

## 2. 判定語と読み方

本書は[並列レーン着手地図](2026-07-25-parallel-lane-readiness-map.md)の既存判定語から、入口の現在地に必要な
`READY-SPEC`と`WAIT`だけを使う。

| 判定 | 本書での意味 |
|---|---|
| `READY-SPEC` | docs-onlyの意味決定、責任仕様、拒否fixture設計を開始できる。実装許可ではない |
| `WAIT` | read-only調査を越えず、既存IDで示した依存またはownerを先に閉じる |

各入口の`DISPOSITION`は既存契約接続票の`PASS / REDUCE / RESOLVE`を使う。`PASS`は現行の狭い既存契約へ
接続できること、`REDUCE`は利用者成果を保つ最小sliceへ縮小すること、`RESOLVE`は既存解決段を先に通すことを
意味し、いずれも`VSM-A9`前の並列実装許可ではない。

## 3. 評価・描画入口マップ

| 入口 | AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | 判定／DISPOSITION |
|---|---|---|---|---|---|---|---|
| Filter — 単一pass既存shape | VSM-A1、plugin authoring | `FilterPlugin`、`RenderCtx`、`PipelineCache`、testkit purity／pixel | plugin runtime／Host GPU | Documentを書かずGPU texture in／out | 実証済みresource shapeを越えるbinding、transient、pass schedulingは無い | 再基線完了後も既存shapeに閉じる。共有APIが必要ならA8G0へSTOP | `WAIT`（§8.1直列背骨）／`PASS` |
| Filter — multipass／Texture | VSM-A8D、A8G0 | 現行`TextureRef`はfirst-party façadeのみ | Host render resource | 未成立。色変換はrender直前一箇所 | linear／HDR intermediate、typed texture／mask port、budget、transient lifetime未決 | `VSM-A8G0 → A8G1 → A8G2 → A8G3` | A8G0のみ`READY-SPEC`、実装`WAIT`／`RESOLVE` |
| Composite | `PluginKind::Composite`、VSM-A8G0 | `CompositePlugin`、registry | plugin runtime／Host GPU | 複数texture入力から一texture出力 | Filterとの責任差、multipass中間、resource shapeが未閉鎖 | A8G0でFilter／Composite／Host passの責任を比較 | `WAIT`／`RESOLVE` |
| Source／Generator — LayerSource | VSM-A3D／A3S | `LayerSourcePlugin`、prepared lowering、Radial Repeater、testkit oracle | plugin runtime／Host GPU | 0-inputからGPU texture出力。Documentは書かない | 現行`uniform16`外のresource shapeは共有契約を要し得る | 再基線完了後も現行shapeへ縮小。共有APIが必要なら独立仕様粒へSTOP | `WAIT`（§8.1直列背骨）／`PASS` |
| Parameter provider | VSM-A2、A7 | `ParamDriverPlugin`、`ParamDriverContext`、`DataTrack`、`ParamSource` | evaluation engine | providerが決定的trackを返し、既存parameter評価が読む | 一般typed consumer port、event、provider identityは未成立 | 再基線完了後もprovider-only fixtureへ縮小。consumerはVSM-B2へ | `WAIT`（§8.1直列背骨）／`REDUCE` |
| Data provider→consumer | VSM-A7D、B2 | 現行`DataTrackId → ParamSource::Data`はparameter駆動のみ | VSM-B2方式決定 | 未決。Document materializeならD2採用が必要 | DataTrack identity、event／typed channel、consumer port方式が未閉鎖 | `VSM-B0 → B1 → B2`で(a)既存param、(b)consumer port、(c)Authoring Toolを比較 | `WAIT`／`RESOLVE` |

Filter／LayerSource／ParamDriverには既存接続先があるが、§8.1の直列背骨と並行して新しいVism仕様を起動しない。
接続先の存在は、同時施工、任意resource shape、semantic SDK公開面を証明しない。

## 4. 意味値・表現入口マップ

| 入口 | AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | 判定／DISPOSITION |
|---|---|---|---|---|---|---|---|
| Path2D | SDK-S0S、意味SDK決定 | `pathgeom::apply`、`PathOp::Offset`、SDK-S0I consumer-neutral fixture | M2 Path native oracle／test-only fixture | immutable Path入力から新しいPath結果。Documentは書かない | SDK-S0Iは独立review済みだがmain未到達。公開TS／Rust API、runtime、製品Inspectorは未成立 | SDK-S0I main到達後に`LANG-TS-F0`を仕様粒化 | 現在`WAIT`、到達後`READY-SPEC`／`PASS` |
| Shape2D | 意味SDK決定 | `VectorRecipe`、`VectorContent`、Path2D | Document schema／render lowering | 現行recipeのread-only投影。新しい保存意味はD2＋GR-PV | style／groupを含む作者contractとPath結果のadoption未決 | Path2Dを再利用し、Shape2D作者contractを独立仕様化 | `WAIT`／`RESOLVE` |
| Text | M5 P6、意味SDK決定 | `VectorContent::TextPath`は保存席のみ | text shaping／font admission owner未閉鎖 | 未決。glyph／path投影からDocument writerを逆算しない | shaping、font asset admission、run／cluster／glyph identityが未閉鎖 | Text shaping ownerとP0Iのdomain identityを先に閉じる | `WAIT`／`RESOLVE` |
| Instance | M5-P0I／P7、意味SDK決定 | `InstanceIndex`はordinal予約でstable identityではない | M5-P0I docs decision | 現段階はdocs-only。Document／rendererへ接続しない | stable InstanceId、nesting、prototype、channel、domain寿命未決 | `M5-P0I`で意味決定後、fixtureとP7 schemaを別粒化 | `READY-SPEC`／`RESOLVE` |
| Spatial | M5 camera／geometry／renderer | canonical XYZ world、M5仕様候補 | Host camera／depth／bounds／renderer | 未成立。3D Instance projectionは`LANG-TS-F0`所有でありSpatial runtime routeではない | geometry、space-tagged transform、surface、renderer主要部未実装 | M5の既存順序でcamera／geometry／rendererを閉じる | `WAIT`／`RESOLVE` |
| Field | 意味SDK決定、simulation model | 候補はscalar／vector field、mask、collider input | Host SDF正規化／budget | 未成立。collision projectionは`LANG-TS-F0`所有でありField出力routeではない | Field／Collider representationが未決 | simulation／collider fixtureでrepresentationを反証してから仕様化 | `WAIT`／`RESOLVE` |
| Simulation | simulation model、SIM-1、VSM-A6 | `PluginKind::Simulation`は予約。trait／StateTrack runtimeは未実装 | Host Bake／StateTrack／invalidate／scheduler | render traitへ隠れ状態を入れず、将来はBake結果を既存LayerSource等が読む | SIM-1、StateTrack、checkpoint、collider runtime未成立 | `SIM-1 → VSM-A6`。L0 pureとL3 Bakeを同一Vism identityで後続反証 | `WAIT`／`RESOLVE` |

InstanceとSpatial、FieldとSimulationを同じ実装粒へ束ねない。前者はdocs decisionを先行でき、後者はruntime依存が
異なる。予約field、enum、schemaの席、Inspector projectionをruntime成功証拠にしない。

## 5. 変更・再利用・外部接続入口マップ

| 入口 | AUTHORITY | INTERNAL TARGET | OWNER | WRITE ROUTE | GAP | RESOLUTION ROUTE | 判定／DISPOSITION |
|---|---|---|---|---|---|---|---|
| Generator／Materialize | VSM-B2、U9a | D2 `apply_macro`、typed proposal候補 | Host preflight／Document single writer | 全commandを開始snapshotへpreflight後、一D2 macro／一Undo | 汎用atomic batchと方式決定が未成立 | `VSM-B0 → B1 → B2 → B2I`、U9a相当のbatch境界 | `WAIT`／`RESOLVE` |
| External Authoring Bridge | external authoring bridge決定 | explicit selection／typed proposal／Host adoption | Bridge seat＋Host | 外部toolはproposalまで、Document変更はHost D2だけ | app別identity、permission、transportを共通schemaへ畳めない | capability別に再入場し、Automation／packageと分離 | `WAIT`／`RESOLVE` |
| Automation | Inspector／source／Automation境界 | immutable snapshot、explicit target、typed operation proposal | 将来Authoring Tool seat＋Host | Host全体preflight後にDocument部分だけ一D2 macro。Bake／Exportは別job | proposal owner、permission、consent、atomic batch、job分離未閉鎖 | VSM-B2／B2IとU9a相当を先に閉じ、独立仕様化 | `WAIT`／`RESOLVE` |
| Kit／Preset | Vism／Kit model、VSM-B0〜B2 | 既存型をまだ公開Kit schemaへ昇格しない | identity／composition owner | 採択方式がmaterializeならHost D2、providerならtyped connection | Vism／Kit／Preset／Asset identityと展開後runtime意味が未閉鎖 | `VSM-B0 → B1 → B2`。実装はB2I以降 | `WAIT`／`RESOLVE` |

Bridge、Generator／Materialize、Automationは「typed proposal＋Host adoption」だけを共有する。runtime、package、
permission schema、lifecycleを一つへ統合しない。

## 6. 横断rail

railは入口に共通する参照軸であり、readiness状態でも独立taskでもない。

| rail | 役割 | owner | 現在の制約 |
|---|---|---|---|
| Inspector projection | 同じVism contractから作用先、typed I/O、space、temporal mode、capability不足を読む | contract fixtureは`LANG-TS-F0`、製品面はM3 U4a | 先行projectionはPath→Path、3D Instance、collisionの三つだけ。後二つはunavailable診断でruntimeを捏造しない |
| Identity | package、entry、Kit、Project instance、artifactの五identityと、Instance domain identityを混ぜない | 前者`VSM-B0`、後者`M5-P0I` | P0Iは`READY-SPEC`。B0は再基線順序上A4I／A5後。indexやcatalog IDをstable identityにしない |
| Package／runtime／admission | semantic contractと配布、loader、署名、隔離、trustを分離 | VSM-B3以降／Phase C、malware containment decision | semantic fixture成功からcontainer、ABI、engine、install、trustを逆算しない |

## 7. 現在着手できる根本粒

同時に**実装**できるVism入口はまだ無い。現在、既存authorityが着手を許すのは次のdocs／spec粒と、
本書main到達後に再判定する直列背骨だけである。

1. Vism直列背骨: 本書main到達後の`VSM-A4I`再判定、その後`VSM-A5 → VSM-B0 → B1 → B2`。
2. Render capability: `VSM-A8G0`のdocs-only仕様。
3. M5 identity: `M5-P0I`のdocs-only意味決定。
4. Path: SDK-S0I main到達後の`LANG-TS-F0`仕様粒。

実際の並列量産は`VSM-A9`が二つ以上のfixture pluginで非干渉を証明した後に、合格した対象laneだけを解禁する。

## 8. 共通STOP

各入口で次が必要になった時は、その入口だけをSTOPして既存解決段へ戻す。他のclosed laneを止めない。

- Document／serde、公開Rust／TypeScript API、module path、ABI、package／loader、plugin kindを追加・変更する。
- Host公開resource API、pass scheduling、cache key、GPU lifetimeを一Vism都合で追加する。
- `InstanceIndex`をstable identity、`TextureRef`をsemantic SDK、`TextPath`をshaping完成、`Simulation`予約をruntime完成として扱う。
- DataTrack identity、Text shaping owner、Field／Collider representation、unified Mesh、Automation schemaを推測する。
- Inspector projectionを新しい製品Inspector、custom UI、runtime成功fixtureへ昇格する。
- Bridge、Generator／Materialize、Automationを同じruntime／package／permission schemaへ畳む。
- 二つ以上のVism実装を`VSM-A9`前に同時起動する。

## 9. Phase C入場条件

Phase Cのcontainer／install比較へ進むには、次をすべて満たす。

1. `VSM-A4I`と`VSM-A5`が成立し、`VSM-A9`が対象laneの非干渉を証明する。
2. `VSM-B0〜B5`と`VSM-B3H`でidentity、成果物分類、provider／consumer／materialize、logical manifest、payload、headless compatibility、hostless distributionが閉じる。
3. `VSM-A6`のHost所有state証拠と`VSM-A8G3`のHost所有multipass証拠が成立する。
4. `VSM-B6`のPhase B反対側reviewがP0/P1=0で、payload classからcontainerを逆算していない。
5. Phase Cは`VSM-C0`から開始し、`C1／C2`は`VSM-B4`と`VSM-B6`、`C3`は`C0〜C2`、`C4`は`C0〜C3`の依存を越えない。

## 10. Opus 5 read-only反対側確認とCodex採否

Opus 5は、意味の根と並列施工の根の混同、`Composite`／Bridge／Kit入口の欠落、InstanceとSpatialの過剰結合、
新しい判定語の増設、SDK-S0Iのmain到達誤認を指摘した。Codexは現行実装計画、並列lane map、SDK-S0S、
Inspector／Automation境界、実コードへ再照合し、以下を採用した。

- `VSM-A9`前は全入口を並列実装可としない。
- readiness判定は既存`READY-SPEC / WAIT`を再利用し、横断railへ状態語を付けない。
- Composite、Kit／Preset、Bridgeを独立行にし、InstanceとSpatialを分ける。
- Inspector、identity、package／runtimeを横断railとし、意味familyへ数えない。

Opusの「VSM-B0を現在READY-SPEC」とする助言は縮小した。VSM-B0の個別表は着手可だが、同じ計画の後発§8.1が
`A4 → A5 → B0 → B1 → B2`を再基線として明示するため、本書はVSM-B0をA4I／A5後の直列位置へ置く。
