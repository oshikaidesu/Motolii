# Vism意味SDK — Cavalry先例のMotolii責任境界への翻訳

状態: **決定**。Vism作者へ長く残すSDKを、特定のTypeScript engine、package、ABI、UI toolkitではなく、
**型付き意味値、純粋operation、明示capability、評価結果の契約**として定義する。Cavalryのdomain objectと
能力分離は先例として採用するが、Cavalry互換、巨大な万能namespace、scene-wide mutable APIは採用しない。
具体的なTypeScript名、module path、signature、runtime、package schema、Document／serde変更は本決定に含めない。

関連:
[AviUtlコミュニティ比較と縮小採用](2026-08-01-motolii-semantic-sdk-aviutl-community-comparison.md)、
[Vism作者programの言語境界](2026-08-01-vism-authoring-language-boundary-decision.md)、
[Vism Inspector・作者source・Automation責任境界](2026-08-01-vism-inspector-source-automation-boundary-decision.md)、
[作者連続性と変更カプセル](2026-07-31-authoring-continuity-capsule-goal-contract.md)、
[Vism作者journey](2026-07-27-vism-authoring-journey-decision.md)、
[PathOp語彙比較](2026-07-12-pathop-ae-cavalry-comparison.md)、
[時間軸の自由度モデル](../simulation-model.md)、
[M5 3D合成とポストプロセス](../specs/M5-3d-and-post.md)、
[plugin authoring](../plugin-authoring.md)

## 1. 「意味SDK」が固定するもの

意味SDKは作者sourceとHostの間にある、言語より長寿命な語彙である。固定するのは次の四点だけとする。

1. Vismが受け取り、返せる**意味値**。例: Path、Instance集合、Field、Texture、Data。
2. 意味値へ適用できる**決定論的operation**と失敗の型。
3. 通常の値入力では表せない能力を要求する**明示capability**。
4. Inspector、接続判定、診断、conformanceへ投影できる**typed contract**。

意味SDKはRust trait、Wasm ABI、JavaScript global、npm package、WGSL binding、Document schemaの別名ではない。
TypeScript、WGSL、Rust admitted moduleは、同じ意味契約を各実行席へlowerするfront-end／implementationである。
したがって作者は一つのVism contractを扱い、Hostは必要な内部artifactを分けても、通常UIへ言語選択を出さない。

```text
Vism typed contract
  ├─ semantic values and operations
  ├─ declared capabilities
  └─ typed result and diagnostics
          ↓ lower / admit
  TypeScript seat | WGSL seat | Host Simulation | Rust implementation
          ↓ project
  one Vism identity | one Inspector | one Preview route
```

この分離により、将来TypeScript engineやABIを交換してもPathやInstanceの意味を捨てずに済む。一方で、意味型まで
曖昧なまま「後でruntimeを差し替えられる」とする逃避も許さない。

## 2. Cavalryから採るもの、採らないもの

Cavalryは[`cavalry.Path`、`Mesh`、`PointCloud`、`Material`、`Matrix`](https://cavalry.studio/docs/tech-info/scripting/cavalry-module/)
を単なる数値配列でなくdomain objectとして作者へ見せる。また
[JavaScript Layer](https://cavalry.studio/docs/nodes/general/javascript-layers/)はtyped dynamic attribute、
[Context](https://cavalry.studio/docs/tech-info/scripting/context-module/)は複製評価のcontext、
[Deformer](https://cavalry.studio/docs/tech-info/scripting/deformer-module/)はgeometry入出力、
[Particle Shape](https://cavalry.studio/docs/nodes/shapes/particle-shape/)はemitterとmodifierの構成を提供する。
これは「作者にRGBA filterだけを渡さない」「表現domainを型として見せる」先例として強い。

しかし、Cavalryの名前やObject Modelを互換対象にはしない。特に
[API module](https://cavalry.studio/docs/tech-info/scripting/api-module/)のscene mutation、file／Web／shell能力、
Contextの`layerId`／`attributeId`／`saveObject()`、可変位置入力`n0`／`n1`、
[JavaScript Emitter](https://cavalry.studio/docs/nodes/utilities/javascript-emitter/)のrawな`particles`配列と暗黙globalは、
Motoliiのsingle writer、純関数、明示authority、stable identityに反する。

| Cavalry先例 | Motoliiでの処分 | 理由 |
|---|---|---|
| `Path` domain object | **採用・一般化** | 正準座標、typed error、既存PathOp意味へ接続する |
| `Mesh`のPath／Material階層 | **分割して一般化** | 2D vector hierarchyと3D spatial geometryを一語へ早期統合しない |
| `Material` | **一般化・延期** | geometryからstyleを分ける原則だけ採用し、3D material契約はM5へ残す |
| `Matrix` | **一般化** | space-tagged transformとし、2D／3D、Local／Worldを暗黙混在させない |
| `PointCloud`／distribution | **Instance集合へ一般化** | ordinalとidentityを分け、nested instanceと2D／3Dを同じモデルへ接続する |
| typed dynamic attribute | **採用** | 作者が付けた意味名と型を保ち、`n0`／`n1`は拒否する |
| `ctx.index`／`count` | **縮小採用** | read-only contextへ入れるがstable InstanceIdとnestingを別に持つ |
| context transform | **条件付き採用** | 明示されたspace capability／typed inputからだけ得る |
| `saveObject()`／`loadObject()` | **棄却** | hidden stateでなくTrack／Host StateTrack／Bakeへ送る |
| layer／attribute ID、nice name | **棄却** | scene traversalと名前依存を作者契約へ出さない |
| Deformerのraw get／set | **純粋入出力へ一般化** | 一評価内builderは許すが永続mutable meshを渡さない |
| raw `particles[]` push／mutation | **棄却** | typed emission resultまたはInstance recipeを返す |
| broad scene API | **意味SDKから棄却** | 将来Automationのtyped proposal＋Host preflightへ分離する |
| font enumeration | **意味SDKから棄却** | font／assetはHostが解決したtyped inputとして渡す |
| math／random helper | **採用** | seed、algorithm version、有限値規則を明示して決定論を固定する |
| 暗黙の型変換 | **棄却** | Inspectorと接続診断が意味を失う |

## 3. 意味domainを一つの万能namespaceにしない

意味SDKは次のcapability familyへ分ける。これはmodule名やpackage構成の決定ではなく、Vismが必要能力だけを
宣言し、Hostが不足を診断するための意味上の分割である。

| family | 長く残す意味 | Hostに残す責任 | 現在の成熟度 |
|---|---|---|---|
| Core | finite scalar、bool、color、Vec2／Vec3、time、duration、seed、typed port | 数値制限、version、診断 | 一部既存、作者契約は未実装 |
| Path2D | contour、curve、Path構築／query／純粋変換 | tessellation、GPU resource、既存PathOp正本 | internal実装あり、公開作者契約なし |
| Shape2D | Path＋style＋groupの意味階層 | render lowering、asset解決 | VectorRecipeあり、作者契約なし |
| Spatial | spatial geometry、space-tagged transform、surface意味 | camera、depth、bounds、picking、GPU resource | M5仕様、主要runtime未実装 |
| Instance | stable identity、prototype、transform、typed channels、nesting | ID発行、world解決、renderer | P0I／P7で未決・未実装を含む |
| Field | scalar／vector field、mask、collider input | scene解決、SDF正規化、budget | 候補意味、公開契約未決 |
| Texture | texture／maskのopaque意味値、declared kernel binding | wgpu resource、lifetime、pass scheduling、色変換 | `TextureRef`は現行first-party plugin façade型で、作者意味SDK契約ではない |
| Data | sampled data、event、typed channel | media解析、cache、revision | DataTrack一部実装、identity未閉鎖 |
| Text | run、cluster、glyph、pathへの投影 | font shaping、font／asset admission | M5 P6系、未閉鎖 |
| Simulation | initial state、step meaning、state output | StateTrack、Bake、invalidate、scheduling | 仕様決定、runtime未実装 |

Vismは必要familyとoperationを宣言する。Hostは「unknown method」ではなく、`Spatial未実装`、`Collider入力不足`、
`Simulation Bakeがstale`等の利用者が行動できる診断へ変換する。すべてのfamilyを一つのdefault globalへ注入しない。

## 4. 値、空間、単位の規律

生の`number`／`Vec3`だけで位置、角度、時間、速度を兼用しない。意味契約は少なくともdimensionとspaceの不一致を
検出できなければならない。ただし、具体的なTypeScript wrapper class／branded type／generic表現は未決とする。

- canonical scalarはfiniteを要求し、NaN／Infinityをtyped diagnosticにする。
- angle、length、normalized value、time、duration、seedを暗黙に交換しない。
- position／direction／normalを同じvectorとして無条件に混ぜない。
- Local／Parent／World／View間の変換は明示operationまたは明示typed inputを通す。
- 2Dは同じcanonical XYZ worldの`z=0`であり、2D用と3D用の別worldを作らない。
- camera、active observation、depth participation、collider正規化をcontext globalから引かない。

変換の結果が新しい意味値である限り、作者実装が一評価の内部でbuilderを局所的にmutateすることは許せる。
禁止するのは評価外へ残るmutable object、別Vismから共有されるhidden state、Document／GPU resourceへの直接mutationである。
受け取ったtyped input自体はimmutable valueとして扱い、builderは新しい所有値またはcopy-on-write意味を持つ。
同じ入力を読む別consumerへ局所mutationが波及してはならない。
これにより「純関数だから全行immutable」という不要な負担と、Cavalry型の永続scene mutationを区別する。

## 5. Geometry: `Path`を入口にし、`Mesh`を早まって統一しない

最初のdurable sliceは`Path2D → Path2D`とする。現行`pathgeom`のPoint／Vertex／Contour／Path実装と、Documentの
closedなPathOp意味を再利用候補にするが、internal Rust fieldや`PathOp` enum自体を公開SDKへ昇格しない。

意味として必要なのは次である。

- move／line／curve／closeによる構築。
- contour、bounds、length、point／tangent等のquery。
- transform、offset、trim等、既存authorityとfixtureで採択された純粋operation。
- invalid contour、非有限値、degenerate geometry、budget超過のtyped diagnostic。
- operationとparameterから同じ入力へ同じ結果を返すversioned conformance。

ただしCavalryに存在する全operationをv1へ移植しない。特にpath booleanは既存決定で対象外であり、意味SDKから
黙って復活させない。resample等の追加候補も、既存PathOp authorityと現在の実装、表現fixtureで裏付けたものだけを
`SUPPORTED`、意味は必要だがalgorithm／oracleが閉じないものを`DEFERRED`、scene authorityやhidden stateを要求するものを
`REJECTED`として個別に処分する。対象外operationの再入場にはconcept／PathOp正本の改訂と独立oracleを要求する。

`Mesh`という語は現時点で公開しない。2DではPath、style、groupからなるvector hierarchy、3Dではvertex／index、surface、
topologyを持つspatial geometryを意味し、lifetime、renderer、material、pickingの責任が異なる。共通化するのは将来のfixtureが
共有operationを証明した範囲だけとし、名前の似たCavalry APIから統一型を逆算しない。

## 6. Instance、Particle、Fieldの境界

Instance集合は単なるpoint arrayでなく、少なくともstable identity、ordinal、prototype、transform、typed channel、nestingを
持つ意味値として扱う。`index`は順序でありidentityではない。filter、sort、LOD、nested duplicationで順序が変わっても、
同じinstanceを追跡できる意味がP0Iで閉じるまで公開形式へ焼かない。

read-only Instance contextが将来提供できる候補は、stable ID、ordinal／count、nesting path、semantic time、明示seedから得る
random stream、明示space transformである。layer ID、attribute ID、scene lookup、wall clock、global randomは含めない。

Particleは別のraw object modelではなく、lifecycleを持つInstance集合として接続する。

- seedと時刻で閉じるL0はpureなInstance resultを返す。
- 区間生成で閉じるL1はHost Track生成へlowerする。
- collision、蓄積、相互作用が必要なL3はHost Simulation＋StateTrackへlowerする。

同じVism identityとtyped resultを保っても、作者SDKへmutable `particles[]`や前frame stateを渡さない。emission recipe、
initial state、step meaningは宣言できるが、StateTrack identity、Bake、invalidate、scheduleはHostが所有する。

Field、Mask、Colliderはtyped input候補として分ける。Vismがscene全体を探索してcolliderを集めたりGPU bufferを作ったりせず、
HostがDocument参照を解決し、canonical worldとSDF等へ正規化したopaque／bounded inputを渡す。具体representationは
Simulation／M5 fixtureが閉じるまで公開しない。

## 7. Texture、WGSL、resource責任

Textureは意味値として接続できるが、作者へraw `wgpu::Texture`、Metal／CUDA／DX handle、device、encoder、allocatorを渡さない。
TypeScript semantic programはtexture operationまたはkernel requirementを宣言し、必要な計算だけをWGSL席へlowerできる。
Hostはbinding、resource lifetime、一時texture、pass scheduling、budget、色変換位置を所有する。

意味SDKがWGSL sourceを文字列で好きな場所へ差し込むAPIになってはいけない。kernelはtyped binding closure、静的検査、
resource budget、preview／export同一関数のgateを通る。CPU fallbackやreadbackを作者判断の通常operationとして出さない。

## 8. Evaluation contextと結果

通常評価contextへ置けるのは、入力値では表しにくく、同じ入力で決定論を壊さない情報に限定する。

| 候補 | 処分 |
|---|---|
| semantic time／duration | 明示的に採用。frame numberだけを正本にしない |
| deterministic seed／random stream | 明示seedとalgorithm versionつきで採用 |
| instance context | Instance capability要求時だけread-onlyで提供 |
| Quality | renderer最適化だけに限定。意味結果やSimulation stateを変えない |
| explicit transform／space | typed inputまたは要求capabilityとして提供 |
| wall clock、filesystem、network、environment | ambient authorityとして拒否 |
| current selection、active layer、scene traversal | Host UI／Document authorityとして拒否 |
| hidden object store | Track／StateTrack／Bakeへ送るため拒否 |

一回の評価は、宣言済みoutput型の意味値とtyped diagnosticsを返す。Document mutation、selection変更、Undo登録、Bake／Export
scheduleは返り値の副作用として実行しない。複数Objectを作成する用途は将来Automationのtyped proposalへ分離する。

## 9. Inspectorと作者連続性への投影

意味SDKのcontractから、HostはVism名、作用先、typed input／output、parameter、space、temporal mode、capability、診断を
投影する。作者が任意HTML／ReactでInspectorを組み立てず、SDK名やbinding番号を通常利用者へ見せない。

AviUtl continuity floorとして、capability familyの厳密さを作者sourceのimport、manifest、wrapper、複数fileへ
そのまま転嫁しない。最小作者面は一つの可視source／recipeから開始し、宣言parameterを通常VismのInspectorへ
自動投影する。TypeScript、WGSL、Simulation等の複数artifactが必要でも、Hostが同じVism identity、Preview、診断へ
束ね、最初の有意味な変更へseat間配線や配布packageの完成を要求しない。

同じcontractは外部IDEの型情報、compile diagnostic、Host preflight、last-good Preview、conformance fixtureの共通正本になる。
ただし外部IDE transport、LSP、source map、workspace layoutを意味SDKへ含めない。作者の入口を便利にする責任と、
表現意味の責任を一つの公開APIへ混ぜない。

## 10. 最初のdurable sliceと後続順序

`LANG-TS-F0`は使い捨てsyntax実験ではなく、次の**SDK-S0意味profile**を反証するfixtureとする。

1. named typed Path input、finite offset parameter、Path outputを宣言する。
2. `Path2D → Path2D`の純粋operationを、Host native oracleと同じfixtureで比較する。
3. sourceを読まずInspector projectionから作用先、入出力、単位、space、失敗理由を読める。
4. ambient global、scene ID、hidden state、implicit conversion、raw GPU resourceを拒否する。
5. engine、module、package、Document、live reloadを実装せず、意味と診断だけを固定する。
6. 一つの可視source／recipe、通常Vism、parameter自動投影というACG-O6／O7を同じfixtureで確認する。

このsliceで具体的なTypeScript API名を恒久化しない。fixtureは意味operation ID、typed value、入力、期待結果、診断を
言語非依存に持ち、TypeScript surface候補とHost native oracleの両方をconsumerにする。F0合格後もlive authoring F1は
VSM-C2等の既存依存を越えるまで開始しない。

後続はauthorityが閉じた順に進める。

1. SDK-S0: Path2Dとtyped contract。
2. Instance identity／nestingがP0I／P7で閉じた後にInstance profile。
3. M5 camera／geometry／rendererが閉じた後にSpatial profile。
4. Simulation／collider fixture成立後にField／Simulation profile。
5. DataTrack identity、Text shaping責任が閉じた後にData／Text profile。

後段の可能性を示すcontract projectionはよいが、未実装runtimeの成功fixtureをS0へ捏造しない。

## 11. 負例とSTOP

少なくとも次をconformanceの拒否例にする。

- 一つの`motolii` globalから全scene、filesystem、network、GPU deviceへ到達できる。
- Path、Instance、Textureがuntyped object／JSON／arrayへ潰れ、Inspectorが作用意味を復元できない。
- `index`をinstance identityとして永続化する。
- Local／World、position／direction、time／durationが暗黙変換される。
- builder mutationが評価外へ残り、同じ入力で結果が変わる。
- 受け取ったtyped inputをin-place mutateし、同じ入力を読む別consumerの結果が変わる。
- particle状態をTypeScript global、Filter、private object storeへ保存する。
- camera、depth、collider、SDF、StateTrack、GPU resourceをVismが所有する。
- PathOp enum、internal Rust struct、`TextureRef`を検討なしに公開SDKへ再exportする。
- Cavalry互換のため、Motoliiの既存authority、canonical coordinates、single writerを曲げる。
- SDK familyごとに別Vism、別Inspector、別Preview routeを利用者へ選ばせる。

次のどれかが必要になった粒は`STOP`して、該当authorityを先に閉じる。

- 具体的な公開TypeScript名、signature、module path、versioning policyの決定。
- Document／serde、public Rust API、plugin package／ABIの追加・変更。
- unified `Mesh`、InstanceId、DataTrack identity、Text／font、Field／Collider representationの新設。
- engine、loader、npm、Node、DOM、Wasm、WGSL binding、外部IDE protocolの採択。
- SimulationPlugin／StateTrack、3D renderer、camera、depth、physicsの前倒し実装。
- custom Inspector UI、Automation API、scene-wide mutable object modelの追加。

## 12. 現在の実装状態

| 項目 | 現在 |
|---|---|
| 意味SDKの責任とfamily分割 | 本文書で決定 |
| TypeScript公式作者言語 | 決定、runtime／surface未実装 |
| Path2D internal geometry／PathOp | 部分実装済み、作者SDKとして未公開 |
| SDK-S0／LANG-TS-F0 | SDK-S0S仕様review中、実装未着手。外部Rust crate scaffoldのVSM-A4Sとはowner／artifactを共有しない別lane。LANG-TS-F0はSDK-S0I待ち |
| Inspector typed projection | docs決定、製品経路への統合未実装 |
| Instance stable identity／nesting | P0I／P7で未閉鎖・未実装を含む |
| Spatial／3D | M5仕様、主要runtime未実装 |
| Field／Collider／Simulation | 意味候補とHost責任だけ固定、公開representation／runtime未実装 |
| Texture／WGSL author seat | 方向決定、binding／runtime／package未実装 |
| Data／Text semantic profile | 既存部分実装あり、作者契約未決 |

## 13. Fable 5反対側レビューとCodex採否

2026-08-01、Claude Code経由のFable 5 (`claude-fable-5`)へ本文書と限定した現行authority／コード事実を渡し、
巨大万能SDK化、2D／3D `Mesh`の早期統一、Host責任漏れ、局所mutationと外部純粋性、instance identity、
既存PathOp／M5／P0I／Data／Text authorityとの衝突という観点でread-only検収させた。

初回判定は`REVISE（P0=0、P1=1、P2=2）`だった。

- path booleanを「必要なoperation」に含めたP1を採用した。conceptの既決非目標と衝突するため必要列から削り、
  対象外operationの再入場条件を§5へ固定した。
- `TextureRef`をinternal implementationとしたP2を採用した。現行first-party plugin façadeの公開型だが、
  作者意味SDK契約ではないという正確な状態へ修正した。
- 局所builder mutationが共有typed inputへ波及する負例不足のP2を採用し、input immutable／copy-on-write意味と
  consumer間非干渉を§4／§11へ追加した。

修正後の再審査は`ACCEPT（P0=0、P1=0、P2=0）`。Fableは、path booleanの再入場条件、
`TextureRef`の実装状態、typed inputのconsumer間非干渉が正本と一致し、新しい公開API、Document／schema決定、
runtime公約、unified `Mesh`、Host authority漏れを導入していないと確認した。Fableの出力はauthorityではなく、
Codexが現行正本と実装事実へ再照合して採否した。
