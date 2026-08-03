# 既知実装採択・置換開発モデル

状態: **横断開発原則／M3〜M5と将来phaseへ適用**（2026-08-02）

## 1. 決めること

Motoliiは、一般機構を第一原理から設計する**発明工程を通常workflowに持たない**。
利用者成果とMotolii固有の不変条件を先に固定し、その成果を成立させている既知実装を調べ、
採択routeを一度だけ決めてから製品経路へ接続する。

これは「外部libraryを無条件に増やす」という意味ではない。`REUSE / ADOPT / WRAP / PORT /
PATTERN / EXTERNAL / REJECT`から、総責任が最も小さいrouteを選ぶ。Motoliiが所有するのは、
作品意味、製品policy、admission、acceptance oracle、絶対規律のenforcement pointと、採択routeを
接続する薄いtranslationである。

この原則はM3だけの速度改善ではない。M4のcache／resource／media処理、M5の3D／text／post／
spatial interactionにも同じ順序を適用する。既存task IDや過去の独自実装は、既知routeの調査前に
実装順や保守対象として自動採用しない。

## 2. なぜ最小コアに必要か

新機構は初期差分だけでなく、owner、状態、identity、failure mode、migration、platform対応、
test、後続規則、廃止判断を恒久的に増やす。private helperでも後続が依存すれば事実上の基盤になる。
投入工数を理由にそれを残すと、Controlled Microkernelの外側へ責任が再膨張する。

したがって、`Motolii固有`なのは目的と採択policyであって、一般機構まで独自である必要はない。
既知実装へ接続できる仕事を独自設計の証明問題へ変換せず、同じoracleを通る置換routeが成立した時は
単一ownerを切り替え、旧routeを`FROZEN → RETIRE`する。

## 3. 実装前の固定順序

M3〜M5の各phaseは、次を順番どおり閉じる。

1. **USER OUTCOME**: 通常製品routeで利用者が得る結果と、失敗時の回復を固定する
2. **MOTOLII AUTHORITY**: 作品意味、絶対規律、既存公開境界、非目標を逆引きする
3. **MECHANISM CLASS**: 必要能力を一般機構の単位へ分け、既存task IDやcrate構成から逆算しない
4. **CURRENT FACT**: 現行codeの成立済み能力、未接続、重複owner、独自実装負債を記録する
5. **KNOWN IMPLEMENTATION SURVEY**: repo内実装、採択済みdecision、`references.md`、一次資料の順に、
   実在するfile／API／algorithm、license、thread model、状態所有、failure mode、platform条件を調べる
6. **ROUTE DISPOSITION**: 機構classごとに`REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL /
   REJECT`と、旧routeの`KEEP / REPLACE / RETIRE`を一度だけ裁定する
7. **ADOPTION MAP**: 検索用の親項目と実装可能な子項目へ、接続target、薄い残余、依存、並列点、
   oracle、cutover、再調査条件を記録する
8. **IMPLEMENT**: 採択地図から一契約境界を発注し、既知routeを薄く接続する
9. **CUTOVER**: 同一oracleで新旧を比較し、writer／consumerを一回だけ切り替え、旧routeを退役する

`ADOPTION MAP`が無いphaseで、遠いtask列を先に詳細化したり、既存IDを順番に実装したりしない。
調査結果から必要な子項目が減ることは成功であり、古い粒数を残量として維持しない。

### 3.1 計画・発注・実装のfail-close記録

一般機構を新設・置換する可能性がある粒は、主担当Codexが計画または発注前に次の6欄を記録する。
これはrunner、CLI harness、外部model transportのschemaではなく、主担当が実装開始可否を判定するpreflightである。

1. `MECHANISM CLASS`: 製品task IDやcrate名から独立した必要能力
2. `KNOWN IMPLEMENTATION`: repo内実装、採択済みdecision、または固定version／commit／APIを持つ既知解
3. `ADOPTION ROUTE`: `REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL / REJECT`
4. `THIN MOTOLII RESIDUAL`: Motolii固有のpolicy、admission、translation、fixture
5. `RETIREMENT`: 旧ownerの`KEEP / REPLACE / FROZEN → RETIRE`、一回のcutover条件、または`NONE`
6. `BUILD: FORBIDDEN`: 通常workflowの固定値。model、reviewer、採択失敗は変更権限を持たない

既決routeを継承する粒は再調査の代わりに正本pathと裁定を記録する。6欄の欠落、
`KNOWN IMPLEMENTATION: 未調査`、裁定なし、一般frameworkを`THIN MOTOLII RESIDUAL`へ入れた記録では、
計画を実装可能とせず、実装担当を起動しない。既知routeが具体的反証で尽きた場合は§6の例外経路へ返し、
`BUILD`へ書き換えて通常施工を続けない。

## 4. 採択地図の必須形式

親項目は機構classと供給routeを検索する単位、子項目は一つの製品成果とoracleを閉じる単位とする。
各子は最低限、次を持つ。

- **結果**: 通常製品routeで観測できる利用者成果
- **既知実装**: 固定version／commitと具体file／API／algorithm
- **採択方式**: `REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL`
- **接続target**: Motoliiの既存owner、型、command、projection、provider
- **薄い残余**: Motoliiにだけ残すtranslation、policy、admission、fixture
- **依存／並列**: 共有writer、event loop、GPU device、artifact publication等の直列点
- **正例／負例oracle**: 製品意味と境界漏出を直接失敗させられる審判
- **cutover**: 旧owner、切替点、parity条件、`FROZEN → RETIRE`の削除条件
- **再調査条件**: oracle、license、platform、security、maintenanceの具体的反証

地図は参考OSS一覧ではない。実装担当が解決策を再発明せず、採択済みrouteを正しいownerへ接続し、
終了後に旧責任を消せるところまで閉じる。

## 5. M3／M4／M5への適用

### M3

[M3既知技術採択・並列実装地図](m3-parallel-implementation-map.md)を現行実装入口とする。
M3で成立した12親・33子は、この横断モデルの最初の適用例であり、M4／M5の仕様正本ではない。

### M4

最初にcache、resource accounting、RoD／RoI、generation／invalidation、階層退避、proxy、
background scheduling、group bake、全曲Draft、corrupt→missを機構classとして調べる。
既存K0〜K8はMotoliiの意味、負例、候補依存を保持する入力であり、既知実装調査前の実装順ではない。
K0のtest-only契約凍結は維持するが、K1a以後を独自ResourceLedger／store／schedulerの実装へ自動接続しない。

M4の次の開発成果は、実装PRではなく**M4既知実装調査と採択地図**である。地図が閉じるまで、
既存codeの`PipelineCache`、dynamic target pool、wgpu budget thresholdをM4完成または採択済み基盤へ
昇格しない。

### M5

最初にscene／object representation、camera observation、spatial renderer、glTF import、depth、text、
Vello局所pass、post effect、picking／gizmo／bounds、deterministic duplicationを機構classとして調べる。
既存P0I〜P7はMotoliiの世界、identity、互換、操作、oracleを保持する入力であり、独自3D engine、scene
framework、text stack、gizmo frameworkを作る実装列ではない。

M5の意味decisionとtest-only fixtureは継続できるが、製品runtime実装は**M5既知実装調査と採択地図**を
通してから発注する。既存2D世界、renderer、Documentを複製して先に見た目だけを成立させない。

## 6. 外部LLMと例外

既知routeの通常接続では、Grok／Opus等の外部LLMは機構を再設計せず、接続ミス、境界漏出、
負例、cutover漏れだけをread-onlyで検査する。外部LLMの賛同はauthorityや実装許可ではない。

必須oracle、license、platform、security、maintenanceの具体的反証で既知routeが尽き、新機構が
不可避に見える場合だけ、Fableへ一回、先例の取りこぼしと既存routeへの再写像を問い合わせる。
Fableを含むmodelは`BUILD`を認可・仕様化しない。なお回避不能なら、利用者例外として選択肢、
追加責任、廃止条件を返し、通常workflow内で発明を始めない。

## 7. 非目標とSTOP

- 依存数、外部code量、著名さだけで採択しない
- libraryの型、scene、state、thread modelをDocument、公開API、plugin契約へ漏らさない
- `PORT / PATTERN`を、名前だけ借りた独自frameworkの新設に使わない
- 採択地図を遠い全taskの仮想API設計へ変えない
- 既完了、テスト緑、投入工数を旧routeの維持理由にしない
- 一つの置換期間にwriter／state ownerを二つ常設しない
- M4／M5の意味未決を、既知実装のdefaultから逆算して埋めない

運用の詳細な候補比較、反証、再裁定、置換票は
[依存優先・責任最小化ゲート](reviews/2026-07-24-dependency-first-responsibility-gate.md)を正本とする。
