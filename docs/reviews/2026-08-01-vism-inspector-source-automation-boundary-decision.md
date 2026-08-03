# Vism Inspector・作者source・Automation責任境界

状態: **決定**。通常利用者が扱う製品上の表現単位をVism、選択中Vismの意味を読む第一面を
Inspectorとする。TypeScript／WGSL／Simulation等の内部実行席を通常UIへ並べず、作者sourceは
必要時だけ同じVism identityから外部IDEへ開く。将来のHost AutomationはVism評価と別席に保つ。
これはInspectorの具体layout、TypeScript SDK API、Automation API、package schema、live runtimeの
実装許可ではない。

関連:
[Vism意味SDK](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md)、
[Vism作者programの言語境界](2026-08-01-vism-authoring-language-boundary-decision.md)、
[作者連続性と変更カプセル](2026-07-31-authoring-continuity-capsule-goal-contract.md)、
[Vism作者journey](2026-07-27-vism-authoring-journey-decision.md)、
[Vism concept](../vism-package-concept.md)、
[Vism / Kitモデル](../vism-kit-model.md)、
[時間軸の自由度モデル](../simulation-model.md)、
[M5 3D合成とポストプロセス](../specs/M5-3d-and-post.md)、
[UI interaction language](../ui-interaction-language.md)

## 1. 製品面はVism、codeは作者source

MotoliiのScene／Timeline／Inspectorへ`TypeScript Object`、`JavaScript Layer`、`WGSL Effect`等の
言語名を通常の表現単位として追加しない。利用者が追加、選択、調整、複製、無効化、診断する対象は
一貫してVismである。TypeScriptはVismを実装する公式作者source、WGSLは必要なVismが要求するGPU kernel席、
SimulationはHost所有の時間発展席であり、利用者へ同格の三種類のObjectとして見せない。

一つのVismは、表現意味を変えずに複数の内部artifact／実行席へlowerできる。

```text
一つのVism identity
  ├─ typed parameter / input / output contract
  ├─ TypeScript semantic program（必要な場合）
  ├─ WGSL kernel closure（必要な場合）
  ├─ Host Simulation / StateTrack（必要な場合）
  └─ LayerSource / Composite等のHost内部実行分類
                 ↓
        一つのInspectorとPreview
```

内部artifact数、payload class、engine、plugin kindを通常利用者へ選ばせない。Hostは同じVism identity、
同じparameter、同じtyped connection、同じPreview、同じ診断へ投影する。内部artifactが複数になる場合も、
作者連続性契約の一変更カプセル、preflight、atomic adoption、last-goodを通す。

## 2. Inspectorを意味の第一面にする

Inspectorはcodeの代替表示ではない。選択中Vismが**どこへ、何を受け、どの空間と時間モデルで作用し、
何を返しているか**をHostが型付き契約から投影する面である。通常表示では少なくとも次を読めるようにする。

| 意味 | 通常表示 | Hostが隠してはならないこと |
|---|---|---|
| Vism | 表現名、enabled／missing／unavailable | 言語名や内部kindを表現名の代わりにしない |
| 作用先 | 選択Objectとの関係、対象domain／scope | opaque ID、名前検索、暗黙の全Scene作用 |
| typed input | 接続元の利用者向け名、型、不足／不一致 | `n0`／`n1`、provider ID、scripting path |
| output | Path、Text、Instance、Mesh、Texture、Data等の意味 | 最終RGBAだけへ早期に畳んで再利用可能性を失わない |
| space | Local／Parent／World／View等、必要な時だけ | 2D／3Dを別worldへ分裂、`Depth Z`と`Rotation Z`の混同 |
| temporal mode | 即時評価、Track、Simulation／Bakeと状態 | 隠れた前frame状態、PreviewとExportで異なる状態列 |
| parameter | 作者が宣言した利用者control | code変数、engine設定、binding番号 |
| diagnostics | 入力、capability、budget、Bakeの行動可能な理由 | Rust／WGSL／JS engine語彙だけの失敗 |

Inspectorを唯一の因果表示にはしない。Stageは選択対象と空間的な作用範囲、Timelineは適用区間、Automation、
Bakeのready／stale範囲を同じrevisionのread-only projectionとして示す。Document、selection、job、cache、
Undoの第二正本を各面へ作らない。通常の理解はInspectorだけで完結でき、空間または時間の位置確認だけを
Stage／Timelineが補う。

詳細なsource、capability、package identity、provider、実行profileは`Advanced / Developer info`へ下げる。
正常時にすべてを常設せず、missing、stale、unsupported、budget failure等の逸脱だけを通常面へ上げる。
このAdvanced面は同じInspect surfaceから一操作で到達でき、作者連続性契約`ACG-O1`が要求する
実装identity、version、typed入出力、要求能力、由来、欠落診断への到達性を弱めない。
`ACG-O1`の一操作は公開Vism instanceを起点に数える。通常選択後に別navigationを辿る経路だけへ依存せず、
`Inspect implementation`相当の一操作が同じInspectorのDeveloper infoを直接開く。具体gestureとlabelは未決だが、
合否は`ACG-O1` fixtureを正とする。

## 3. 3D、Particle、物理も同じVismとして見せる

Motoliiの3Dは別software／別Scene／別rendererへ移るmodeではない。2Dは正準worldの`Z=0`平面にあり、
Path、Text、Mesh、Instance、Texture、Dataを同じtyped graphで接続する。Vismは必要な時だけ空間、camera
observation、depth、collider等の**typed inputと要求capability**を宣言し、Inspectorがその意味を投影する。
ここでいう宣言はVismへScene authorityを渡すことではない。active camera binding、Observation配布、
`Depth Participant`を含むdepth参加境界、bounds／pickingはHost／Documentが所有し、collider参照と物理係数は
DocumentからHostが解決してSDF＋world transformへ正規化する。

Particle Vismは衝突の有無で別Vismへ交換しない。既決の時間軸はしごに従い、同じ表現identityとparameter面を
保ったままHostが評価席を選ぶ。

| 使用意味 | 内部評価 | 利用者へ追加して見せるもの |
|---|---|---|
| seed＋時刻で閉じるparticle | L0 pure evaluation | 通常parameterと出力Instance情報 |
| 区間一括生成で閉じる軌道 | L1 Track生成 | 生成／stale診断が必要な時だけ |
| 3D衝突、蓄積乱流、粒子間相互作用 | L3 Simulation＋StateTrack | `Bake対象`、ready／stale範囲、失効理由 |

衝突をONにした時、InspectorのVism名、既存parameter、既存typed input／outputを別物へ差し替えず、temporal mode、
Bake状態、collision用に宣言済みのtyped input（collider参照、物理係数、形状解釈）を追加する。
simulation-model §8の「parameter panelは変わらない」は既存parameterを別panel／別Vismへ差し替えない意味であり、
collision宣言済みtyped inputとBake lifecycleの追加表示を隠す意味ではない。
Simulation状態はHost所有StateTrackへ置き、TypeScript global、Vism-private
`saveObject()`、Filterの前frame出力へ隠さない。状態生成と描画を分離し、同じParticle stateをpoint、trail、
mesh等の複数LayerSource表現へ再利用できるようにする。
ready／stale／Bake診断は状態を生成するVism identityへ帰属させ、consumerには同じrevisionのtyped-input診断として
伝播する。rendererごとに別のBake正本を持たない。

3D／Simulationは現行製品で完成済みではない。M5のSpatial camera、mesh／point renderer、Duplicator、
SimulationPlugin＋StateTrackの多くは仕様または予約席であり、本文書は実装済み能力を主張しない。

## 4. TypeScriptからInspectorへ投影するもの

TypeScript作者sourceはHost UIをimperativeに組み立てず、Vismのtyped parameter、input、output、要求capability、
計算を記述する。Hostが同じ宣言契約からInspector、接続可否、diagnostic、conformance fixtureを生成する。

通常authoring surfaceへ次を持ち込まない。

- `thisLayer`、`app.project`、scene-wide object graph等のambient scene traversal。
- layer名、layer ID、attribute ID、scripting pathによる作用先指定。
- `n0`、`n1`等、UI上の意味名と型を失う可変位置input。
- 作者codeが通常Inspectorへ任意HTML／React／canvasを注入するcustom UI。
- TypeScript／WGSL／Rustの選択、engine、module resolver、binding番号等のruntime設定。

作用先はVismを適用したHost operation、typed port、Kit接続から解決する。作者sourceは解決済みtyped inputと
宣言capabilityを受け、意味結果またはtyped proposalを返す。Document採番、selection、Undo、resource admission、
Bake schedulingはHostが所有する。

作者が公開するcontrolのgroup、label、単位、範囲、初期値、短い説明を宣言契約へ含めることはできるが、
具体的なschema、decorator、関数名、layout hintは`LANG-TS-F0`、custom UI許可範囲は既存の
`G0-3 / GAP-13` plugin UI公開契約で比較する。
この決定からSDK APIを逆算しない。

## 5. sourceは段階開示し、IDEをHostへ持ち込まない

通常利用者の動線は、Vismを追加しInspectorで調整してPreviewするところで完結する。sourceを見なくても
作用先、入力、出力、parameter、評価状態、失敗理由を読めなければ不合格とする。

作者へ進む動線は同じVismから始める。

```text
Inspect Vism
  → Fork / Create local candidate
  → Open Source in External IDE
  → 保存変更をHostが検出
  → validate / compile / preflight
  → last-goodを保ったPreview
  → atomic adoption
```

Motolii本体へ汎用IDEを実装しない。Hostが所有するのは、source位置の解決、外部IDEで開く操作、変更検出、
検査、compile、source位置つきdiagnostic、last-good Preview、採用／棄却である。補完、全文検索、Git、
refactoring、debugger、extension ecosystemは外部IDEへ任せる。

外部IDEとの具体transport、watch方式、LSP、workspace layout、複数file、source map、debug protocolは未決である。
`Open Source`を決めたことから、VS Code専用extension、localhost server、任意filesystem権限を正本化しない。

## 6. Host Automationは別の将来席

複数Objectの作成、接続、parameter変更、Bake、Export等をまとめるAutomation需要は閉じない。ただし、
Vismの毎frame評価、TypeScript作者program、inline expressionへ混ぜない。将来Automationは、少なくとも
immutable snapshotまたは明示targetからtyped operation proposalを作り、Hostが全体preflightした後、
D2 single writerでatomicに適用するAuthoring Tool席として比較する。

```text
Automation request
  → explicit target / permission / immutable snapshot
  → typed operation proposal
  → Host preflight
  → user consent（必要な操作）
  → Document mutationだけをone D2 macro / one Undoで採用
  → Bake / Exportは採用後のHost job
```

Bake／ExportはDocument mutationでもUndo対象でもない。Automationがそれらを要求する場合、Document変更の
atomic adoptionと、StateTrack／Export jobのschedule、cancel、diagnosticを分け、cacheや出力jobをDocument意味へ
焼かない。

概念席を残すことは、`app`／`project` global、scene-wide mutable object model、任意shell／network、常駐listener、
Automation API、CLI、headless service、公開schemaを現時点で追加することではない。External Authoring Bridge、
Generator／Materialize、将来Automationの共通点はtyped proposalとHost adoptionであり、同じruntime／package／
permission schemaへ先回りして畳まない。

## 7. Cavalry／After Effects先例の処分

CavalryはJavaScript Editorのscene-mutating `api`と、composition内JavaScript Layerの計算用`ctx`を分け、
typed dynamic attributeと`cavalry.Path`等のdomain objectを持つ。この能力分離、typed input、domain object、
GUIからcodeへ段階移行できる原則は採用する。ただしCavalryの`Copy as JavaScript`が出すscene構築命令列を
成果形式として採らず、Motoliiでは選択Vismのcontract、fixture、sourceを開く。

一方、JavaScript Shape／Utility／Deformerを通常Layerとして見せ、`layerId`、`attributeId`、`n0`等を
作者と利用者へ露出し、Editor APIからscene、Web、shellへ広いauthorityを与える構造は採用しない。
After EffectsのExpression／Script／Extensionを一つのJavaScript文化で連続させた利点は認めるが、
「codeがどこへ作用するかを利用者が復元する」「Hostが内蔵IDEとscript UI ecosystemを背負う」構造を
Motoliiへ再生産しない。

MotoliiでGUIからcodeへ進む対象は、描画命令の断片や匿名expressionではなく、現在選択中のVism contract、
fixture、sourceである。通常UIの主語をcodeからVismへ反転する。

## 8. 反証を現在と後続runtimeに分ける

未実装のM5／Simulationを`LANG-TS-F0`へ前倒ししない。現在走らせられる意味fixtureと、依存実装後の
runtime fixtureを分ける。

### 8.1 `LANG-TS-F0`とInspector projectionで先に閉じる

1. **Path → Path**: 入力Path、offset量、出力Path。codeなしで作用先と再利用可能な出力が読める。
2. **3D Instance contract projection**: runtimeを捏造せず、既存authorityから固定したcontract fixtureで
   emitter、BeatEvents、Mesh、World Space、Instance出力、capability不足を同じInspectorへ投影できるか確認する。
3. **collision contract projection**: L3を実行せず、collision用typed input、Bake対象、未実装／利用不能診断を
   codeなしで読めるか確認する。

### 8.2 `SIM-1`／M5依存成立後に初めて閉じる

1. **3D Instance Particle runtime**: 同じcontractを実mesh／point rendererとSpatial observationで評価する。
2. **Particle collision昇格 runtime**: 同じVismでcollisionをONにし、L0からL3へ移る時もidentity、既存parameter、
   Preview経路を保ち、collision typed inputとBake lifecycleだけが追加されることを確認する。
3. **producer／consumer診断**: 一つのStateTrackをpoint、trail、meshへ渡し、Bake診断がproducerへ一意に帰属し、
   consumerへ同revisionで伝播することを確認する。

L0→L3で同じVism identityを保つことは本書の意味方針だが、製品runtimeとしての成立はこの後続fixtureが
合格するまで**反証待ち／未実装**である。合格前に3D／Simulation対応を名乗らない。

少なくとも次を負例にする。

- sourceを開かないと作用先、入力、出力、Simulation理由を判別できない。
- `n0`／`n1`、opaque ID、layer名検索、provider実装名が通常Inspectorへ出る。
- collision ONで別Vism、別parameter panel、別world、別Preview経路へ交換される。
- TypeScript globalやFilter内部に前frame状態が残る。
- Inspector、Stage、Timelineが別selection、別Bake状態、別revisionを表示する。
- compile／runtime failureがlast-goodを消し、別VismまたはDocument編集を止める。
- sourceを開くためMotolii内蔵editor、特定IDE extension、Node／npm runtimeが必須になる。

## 9. STOPと未決

本決定から次を実装しない。

- TypeScript SDKの関数名、entry signature、parameter metadata schema。
- custom Inspector UI、任意HTML／React注入、plugin所有selection／Undo。
- Inspectorのpx layout、常設group数、icon、色、具体component。
- external IDE protocol、特定IDE extension、LSP、watch、source map、debugger。
- Automation API、global object model、CLI、headless service、permission schema。
- live engine、package container、payload、loader、signature、module resolver。
- SimulationPlugin／StateTrack、Spatial camera、3D rendererの前倒し実装。

これらが必要になった粒は、既存authorityと実コードを再確認し、該当する仕様／decisionを先に閉じる。
「Inspectorへ表示できそう」「TypeScriptなら書けそう」を公開API、Document、serde、package、runtime実装の
許可にしない。

## 10. Fable 5反対側レビューとCodex採否

2026-08-01、Claude Code経由のFable 5 (`claude-fable-5`)へ本書、言語境界、作者連続性、Simulation、
M5の現行正本をread-onlyで渡した。初回判定は`REVISE（P0=0、P1=2）`だった。

- 3D／camera／depth／colliderの「宣言」がVism側scene authorityに読める曖昧さをP1として採用し、§3で
  Host／Document所有を固定した。
- `LANG-TS-F0`へ未実装M5／Simulation runtime fixtureを混ぜていたP1を採用し、§8を現在のcontract projectionと
  `SIM-1`／M5後のruntime反証へ分けた。
- collision入力、plugin UI正本、Bake／ExportとUndo、Cavalry GUI→code、ACG-O1、producer診断のP2六件も、
  現行正本に一致する範囲で採用した。

修正後の再審査は`ACCEPT（P0=0、P1=0）`。任意P2二件は、ACG-O1の一操作の起点と、既存parameterを
差し替えないこととcollision typed input追加表示の両立を明示する指摘で、§2／§3へ追加採用した。
Fableの判定はauthorityではなく、Codexが現行正本、実装状態、docs検査へ再照合した結果として処分する。

## 11. 実装状態

| 境界 | 現在 |
|---|---|
| Vism identity、typed parameter、Host projectionの責任 | docs決定、現行static first-party境界で部分実証 |
| TypeScript作者言語、MTS-1候補 | 決定、`LANG-TS-F0`未実装 |
| Inspectorを通常のVism意味面にする本決定 | 決定、Vism製品経路への統合未実装 |
| 外部IDE open／reload／last-good authoring | 方針決定、transport／製品経路未実装 |
| 3D／Particle／Simulationの一Vism投影 | 既存M5／Simulation決定へ接続、主要runtime未実装 |
| Host Automation | 将来席だけ決定、API／runtime／schema未決 |
