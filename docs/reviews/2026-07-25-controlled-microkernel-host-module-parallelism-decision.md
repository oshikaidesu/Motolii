# 制御されたMicrokernelとHost capability module並列化決定

作成日: 2026-07-25

状態: **決定**

対象: Core kernel、bundled Host module、first-party／third-party plugin、今後の発注粒度と並列化条件

## 1. 決めること

MotoliiのCoreは、Document、Undo、cache、resource、Preview、Export等の全機能実装を一枚岩で
所有するのではなく、作品の正本と排他authorityを制御する**極小のtyped protocol kernel**へ
収束させる。

従来Core責任と呼んだ機能の具体実装は、信頼済みの**Host capability module**へ分離できる。
各moduleは他moduleのprivate型や実装順へ依存せず、Coreが定めるtyped port、authority slot、
revision、lifecycle、failure contractだけへ依存する。共通contractが締結済みなら、同じslotの
複数実装、異なるHost capability、consumer、表現pluginを並列に実装・検収できる。

```text
Controlled Microkernel
  ├─ identity / canonical time / revision
  ├─ typed capability and port contracts
  ├─ immutable snapshot publication
  ├─ atomic commit arbitration
  ├─ authority multiplicity and ordering
  └─ lifecycle / version / typed failure
          │
          ├─ admitted Host capability modules (TCB)
          │    Document reducer / journal / Undo projection
          │    evaluator / cache / resource admission
          │    Preview / Export / asset / package / UI projection
          │
          └─ constrained expression and authoring plugins
               Filter / Composite / LayerSource / ParamDriver
               future Provider / Tool / Vism
```

これは「作品正本を任意pluginへ渡す」「すべてを同じ公開plugin ABIにする」という決定ではない。
Hostが責任を手放すのではなく、**Host所有をauthority ownershipとして固定し、具体実装の所有を
moduleへ分離する**決定である。

## 2. なぜ変えるか

これまでの「Coreが手放さない責任」は、第三者へ基礎責任を投棄しない安全規律として必要だった。
一方、責任所有と具体実装を同一視すると、Document、Undo、journal、cache、resource、UI、
Preview、Exportのどれかが未完了なだけで、相互に独立な実装まで同じ直列列へ入りやすい。

Motoliiが欲しいのは、機能を増やすたびに中央実装を変更する構造ではない。

```text
意味と排他authorityの裁定          少数・直列
締結済みcontract上のprovider実装   多数・並列
通常製品routeでの統合審判          slice単位で合流
```

AviUtlの「違法建築」という比喩から採るのは、独立作者が小さな接続点から能力を増やせた成長性だけで
ある。暗黙共有runtime、配置path、patch、opaque ID、隠れstate、衝突を利用者が解決する構造は
転写しない。Motoliiは型、authority、多重度、version、failure、conformanceをCoreが制御した上で、
同じ並列成長性を得る。

この比喩はAviUtlの歴史的因果を証明する先例調査ではなく、本決定の構造を説明する観察に限定する。

## 3. Coreに残す最小責任

Coreへ残すのは機能一覧ではなく、複数moduleが同じ作品意味を壊さず参加するための制御面である。

1. **identityとrevision**
   - stable identity、canonical time、Document revision、provider bindingの識別
   - ID発行authorityとrevision確定点の一意性
2. **typed capability／port contract**
   - input、output、parameter、failure、version、provenance
   - provider private型、生JSON、表示名検索をconsumer契約へ出さない
3. **immutable snapshot publication**
   - accepted revisionから一つのsnapshotを発行し、各surface／workerはread-onlyに読む
4. **atomic commit arbitration**
   - proposalを全体preflightし、一回のcommitまたは変更0にする
   - single writerは実装package名でなく、一つだけ有効なauthority slotとして守る
5. **authority multiplicityとordering**
   - `exactly one`、`many`、`ordered chain`等をslotごとに固定し、重複登録を拒否する
6. **lifecycle、version、typed failure**
   - 欠落、future version、capability不足、stale revision、module停止を局所化して診断する
7. **絶対規律のenforcement point**
   - VRAM常駐、色変換一元化、純関数、正準座標、Preview／Export同一意味を、実装の慣習でなく
     port、composition、conformanceの合否にする
8. **scheduling／backpressure／observabilityの意味**
   - dependency、generation、cancel／discard、deadline、admission、trace identityを共通contractにする
   - thread pool、queue、優先度、計測backend等の具体policyはscheduler／consumer moduleへ置く
   - UI、Preview、Export、asset moduleが独自の最新値規則や無上限queueを持つことを拒否する

Coreがこれらを制御することは、全機能のアルゴリズム、保存backend、UI、policyをCore crateへ置く
ことを意味しない。`motolii-core` crateの現行item一覧を、この最小Coreの完成形とみなさない。

## 4. Host capability moduleへ分離できるもの

| 従来のCore／Host責任 | 分離後のmodule候補 | Coreが保持する制御 |
|---|---|---|
| Document編集 | domain command reducer | revision、identity、atomic commit、unknown保持 |
| journal | durability provider | accepted commitとの順序、failure時不変 |
| Undo／Redo | commit log projection／history provider | 1 gesture＝1 history、single writer |
| 評価計画 | evaluation planner | canonical time、typed graph、循環拒否、同一意味 |
| cache | cache provider | 完全key、透明なmiss、invalidation authority |
| resource管理 | admission／ledger provider | hard cap、owner、lifecycle、GPU境界 |
| Preview | evaluator consumer／display provider | revision、generation、Quality、同一評価 |
| Export | evaluator consumer／artifact provider | Final意味、finish、atomic artifact |
| asset | resolver／import adapter | stable identity、provenance、欠落診断 |
| UI | snapshot projection／typed intent adapter | state owner、revision、commit authority |
| package／trust | resolver／verifier | capability、version、permission、failure |

表のmodule名、trait、crate、ABI、wire形式は未決である。現行実装を表に合わせて移動する許可ではない。
各seatは現在コード、二つ目の実装、負例、conformanceを確認してから個別に締結する。

## 5. 権限と多重度

module化は、すべての能力を自由に複数登録できるという意味ではない。

| slot例 | 多重度 | 規律 |
|---|---|---|
| Commit Authority | `exactly one active` | 二重writerをcomposition時に拒否 |
| Stable ID Authority | `exactly one active` | module独自採番を拒否 |
| Final Color Transform | `exactly one terminal` | plugin内変換と二重適用を拒否 |
| Cache Provider | policyごとに`exactly one active` | plugin自前cacheを拒否 |
| Asset Resolver | `many / explicit priority` | 黙示順、path推測を拒否 |
| UI Projection | `many` | 全surfaceが同じrevisionを読む |
| Render Effect／Provider | `many` | typed input、評価順、resource contractに従う |

多重度や順序の具体語彙を本書から公開enumへしない。最初のseat実装時に必要な最小閉集合だけを
仕様化する。

## 6. pluginという語と信頼境界の分離

並列性のためのmodule化と、利用者が導入できる公開plugin化を同一視しない。

| 分類 | trust／権限 | 配布 | 欠落時 |
|---|---|---|---|
| Controlled Microkernel | TCB。製品正本の制御者 | 製品必須 | 製品を成立させない |
| admitted Host capability module | TCB。seatごとに限定・審査された内部権限 | 製品buildへ明示的に同梱 | capability単位で停止／縮退 |
| first-party expression plugin | **非信頼**。公開façadeの制限能力。現行同一processは隔離未達 | 現行は静的同梱 | recipe保持＋診断 |
| third-party plugin／Provider | **非信頼**。同じ制限能力 | runtime／配布は未決 | 局所欠落として保持・診断 |

`module`であることはdynamic load、欠落可能、第三者差替え、公開ABIを意味しない。逆に静的同梱でも、
private依存なしのtyped contractとconformanceだけで独立実装できれば並列化の目的を満たす。

ここでの信頼は供給元のラベルではなく、**どのauthorityへ到達できる実行roleか**で決める。
TCBへ含めるのはControlled Microkernelと、製品buildがseat、権限、lifecycle、failure contractを
明示的に審査して組み込むHost capability moduleだけである。これを「core plugin」と呼ばない。
利用者が導入・交換する公開pluginではなく、製品を成立させる内部moduleだからである。

first-partyはsource、同梱、保守責任、参照実装であることを示すprovenanceにすぎない。公開plugin境界を
通るcodeはfirst-party／third-partyを問わず非信頼とし、同じcapability制限、resource budget、
failure containment、将来のsandbox／worker審判へ通す。「標準搭載だからeditor process内で任意codeを
実行してよい」「署名済みだから作品正本へ触れてよい」という例外を作らない。

現行v1のfirst-party pluginは同一binary／同一processへ静的に組み立てられており、これは公開façadeを
実証する現在のコード事実であって、安全境界の完成形ではない。現在存在するのはrender worker境界の
`catch_unwind`(panic後はworker停止)とdevice-lost／uncaptured errorの型付き検知までである。
plugin dispatch単位のpanic隔離、wgpu error scope、device復帰、instance再生成は未実装であり、将来追加しても
memory corruptionやprocess abortを封じるクラッシュ隔離の代わりにしない。任意のnative dylibをeditor
processへloadする経路は採らない。

長期のruntime比較は、構造化data／parameter codeのWASM sandboxと、GPU workloadのprocess外workerを
候補にする。Hostはrecipe、identity、revision、cache key、resource admissionを保持し、plugin runtimeが
停止してもDocumentと他pluginを巻き込まず、instanceを破棄・再生成・再投影できなければならない。
process間GPU共有、ABI、署名、permission、quota、version negotiationは未決であり、本決定から公開APIを
先行実装しない。開発時の差し替えと障害復旧の共通lifecycleは
[開発体験](../dev-experience.md#31-hot-reloadとcrash-recoveryを同じ交換路へ畳む)を参照する。

## 7. 並列開発の解禁条件

能力laneは次を満たした時点で、製品全体の背骨完成を待たず並列実装を解禁できる。

1. seatの意味、owner、入力、出力、failure、多重度が固定されている。
2. provider／consumerが参照するtyped contractが一箇所にある。
3. fakeまたは参照providerと、正例・負例を含むconformance fixtureがある。
4. moduleが変更してよいfileと、変更してはいけない共有面が閉じている。
5. module追加が他moduleのsource、fixture期待値、Document schemaを変更しない。
6. duplicate identity、authority重複、private dependency、未宣言capabilityを機械拒否できる。
7. 通常製品routeへ合流する前の単体成果と、slice完成を区別できる。

共有contract、Document意味、永続形式、公開API、authority多重度の変更が必要になった実装は
`STOP`し、当該contractだけの独立decision／orderへ戻す。他moduleの実装へ便乗して広げない。

### 7.1 「全体を並列化する」の四面

moduleへ分けただけで、すべての処理が自動的に同時実行できるわけではない。Motoliiでは次の四面を
分離し、それぞれ別の合否で並列化する。

| 面 | 並列化できる条件 | 直列に残すもの |
|---|---|---|
| contract／設計 | 異なるseatの意味とownerが独立 | 同じ公開contract、永続意味、authority多重度の裁定 |
| 実装／検収 | contract、変更許可file、fixtureが閉じている | 共有contract変更と通常製品sliceの最終統合 |
| runtime評価 | immutable snapshot、純関数、宣言済み依存DAG、独立output、resource budget | dependency edge、ordered effect chain、同じGPU resourceへの排他操作 |
| lifecycle／障害復旧 | instance、generation、cache key、failure domainが局所化 | accepted revisionのcommit、active provider選択、TCB更新 |

runtimeではaccepted revisionから発行した同一snapshotを複数evaluator／surface／workerがread-onlyに
消費できる。依存の無いsubgraph、UI projection、Previewの準備、asset resolve等は並列候補になる。
一方、effect stackの意味順、D2 single writer、atomic commit、Final color transform等は直列性そのものが
契約であり、threadを増やして崩さない。

したがって本決定の完成条件は「module数が増えた」ことではなく、各seatについて
`SERIAL AUTHORITY / PARALLEL IMPLEMENTATION / PARALLEL RUNTIME / ISOLATED WORKER`
のどこまで成立したかを証拠付きで区別できることである。開発laneの独立性をruntime thread safetyの
証明にせず、runtime DAGを発注並列化の根拠にも使わない。

## 8. 直列に残すもの

次は設計上の負債ではなく、作品正本を一つに保つための意図的な直列点である。

- 新しいsemantic seatとauthority ownerの採否
- 永続意味、migration、公開contractの変更
- accepted revisionを確定するcommit
- 同じslotのactive provider選択
- 通常製品routeで複数moduleが合流する縦sliceの最終審判

ただし、これらの裁定待ちを無関係な締結済みlaneへ伝播させない。直列点はcontract単位で閉じ、
実装全体、milestone全体、全plugin作者を一列へ戻さない。

## 9. 既存決定の限定改訂

本決定は、次の既存規律を維持する。

- semantic seat、作品identity、single writer、Undo意味、Preview／Export同一意味はHostが所有する
- standard UI moduleとcommunity plugin、presentation runtime、provenance／trustは別軸である
- Document、selection、cache、Transportの第二正本を作らない
- first-party専用raw API、生JSON、opaque ID分岐、隠れstateを作らない
- 現行の公開plugin API、Document schema、serde面、dynamic loaderを本決定だけで変更しない

一方、次の読みを限定的に置換する。

- 「Coreが所有する」≠「Coreが全具体実装を一枚岩で持つ」
- 「Host module」≠「presentation moduleだけ」
- 「pluginへ委譲しない」≠「信頼済みinternal capability providerへ分離しない」
- 「private interfaceで交換可能」だけで終えず、二実装とconformanceが成立するseatは並列laneへ昇格できる

[surface実装と拡張所有の軸分離](2026-07-22-m3-surface-extension-axis-separation.md)§2／§6、
[Core／plugin境界lineage回収](2026-07-23-historical-core-plugin-boundary-lineage-recovery.md)§1／§3、
[小さなコア](../extensible-core-model.md)§1.1／§3の「欠落可能な外部pluginへ責任を投棄しない」
規律は維持する。ただし、具体実装をCore内へ固定する根拠としては本決定が後続の正本となる。

## 10. 現在地と非目標

本決定はアーキテクチャと今後の発注原則を固定する。次を完了・解禁しない。

- 現行crateの再編、trait追加、dynamic loader、module ABI、manifest
- Document／journal／cache／resourceの実装移動
- third-partyへHost authorityを公開すること
- M3 VS-1の現在order、G0-3、G0-6H、G0-9D、M4／M5の既存依存の自動変更
- 現行仕様taskを「module化できる」という理由だけで完了扱いすること

次の実装前作業は、現行crateとtaskを本書のseat候補へ分類し、
`KEEP IN KERNEL / EXTRACT CONTRACT / PROVIDER CANDIDATE / CONSUMER / REJECT`
およびauthority多重度を記録するread-only inventoryである。inventoryからtraitやcrateを自動生成せず、
二実装、conformance、製品sliceへの速度効果があるseatだけを個別decisionへ上げる。

## 11. 開発速度の審判

この設計の目的は「plugin数」ではなく、共有面を壊さず同時に進められる能力laneを増やすことである。
各seatの採否では次を測る。

- 新provider追加にCore enum、Document schema、全consumer変更が必要か
- module Aの実装やtest期待値をmodule Bが変更するか
- fake providerだけでconsumerを先行実装・検収できるか
- contract変更なしのprovider追加を独立commitへできるか
- 統合失敗が該当capabilityへ局所化されるか
- 新しい表現一件ごとに背骨変更が発生していないか

新しい能力を追加するたび共有Core変更が常態化するなら、module数が多くても本決定の目的は未達である。

## 12. 反対側レビュー

[Fable反対側レビュー](2026-07-25-controlled-microkernel-fable-counter-review.md)は、現行コードに
single writer、atomic commit、revision付きsnapshot、plugin contract、generation付きworker、
Preview／Export同一評価という実在seamがあることを確認した。初回は現行防御層と旧native信頼記述の
事実精度をP1として`REVISE`し、訂正後の限定再検収でP0/P1=0、`VERDICT: ACCEPT`となった。

これはruntime DAG、frame cache、resource ledger、process外worker、instance交換の成立を意味しない。
次段は§10のread-only seat inventoryであり、各seatを
`SERIAL AUTHORITY / PARALLEL IMPLEMENTATION / PARALLEL RUNTIME / ISOLATED WORKER /
NOT YET PROVEN`へ証拠付きで分類する。

inventoryを全体barrierにせず、開始条件を満たしたseatから通常製品route上の人間応答地点へ送る運用は
[並列Human Response Frontier実行決定](2026-07-25-parallel-human-response-frontier-execution-decision.md)
を正本とする。
