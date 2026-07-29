# M3縦slice実行方針（2026-07-24）

状態: **決定 / Fable最終ACCEPT / Codex採否済み**

## 1. 決めること

M3は新しい作品意味や基盤を増やすphaseではなく、M0〜M2で成立した能力を通常製品UIから
利用できるように**接続するphase**として管理する。M3の進捗、現在地、完成判定は、
技術層やUI部品の完了数ではなく、利用者が通常製品routeで完走できる**縦slice**を単位にする。

既存の`M3-A Presentation Ownership`、`M3-B Host Projection / Intent`、
`M3-C Product Runtime Integration`、`M3-D Editing Loop`は廃止しない。ただしM3全体を
A→B→C→Dの横段階で進める進捗軸から、各縦sliceに必要な境界を確認する
**接続checklist**へ位置づけを変更する。

```text
変更前（横段階が進捗軸）

全surfaceのA -> 全surfaceのB -> 全surfaceのC -> 全surfaceのD

変更後（利用者成果が進捗軸）

Slice 1: 通常製品route -> Rectangle配置 -> 三面投影 -> Undo
           Aの必要部分 / Bの必要部分 / Cの必要部分 / Dの必要部分

Slice 2: 選択 -> 値変更 -> key/easing -> seek/playback -> Undo/Redo
           Aの必要部分 / Bの必要部分 / Cの必要部分 / Dの必要部分
```

ロードマップと完成判定は縦sliceで持つ。実装発注とcommitは、既存規律どおり一つの
契約境界へ分ける。縦sliceを一枚の巨大な発注へ束ねない。

## 2. 変更が必要になった事実

現行文書では、同じ作業を次の複数座標で手書きしている。

```text
M3仕様のG/U task
  -> M3-A〜D stage
  -> 快適利用Work MapのW地点
  -> 粒度化文書のCU粒
  -> implementation ledgerの現在地
  -> closed orderの目的・依存・STOP
```

例として固定React baselineの再結合は、`U0e-2R`、`M3-A`、`W0a`、`CU-0A01`、
ledgerの現在選択中という複数名で現れる。これは必要な契約確認を増やすというより、
同じ意味・依存・STOPの転記と同期を増やしている。

全将来作業を先に最小粒へ分けると、未実装領域のowner、負例、依存、未決意味まで
先回りして決める必要が生じる。粒化から仕様判断が発生し、その仕様をさらに粒化する
循環ができ、UI接続より粒の保守が主要作業になる。

## 3. 用語

| 語 | 意味 |
|---|---|
| 縦slice | 通常製品routeの利用者操作から、既存domain能力、観測可能な結果、Undo/失敗までを一続きに審判できる制作経路 |
| slice出口 | UI部品の存在ではなく、同じfixtureとrevisionで利用者成果をE2E確認できる状態 |
| enabling order | sliceの成立に必要なasset移管、projection、platform証拠、core、codec等の一契約境界。単独完了をslice完成と数えない |
| 接続checklist | A〜D、GR-UI、React必須block等、各orderで該当境界の抜けを防ぐ横断規律 |
| rolling horizon | 現在sliceだけをblocking decisionと発注可能な精度にし、次の二sliceは出口と主依存まで、それ以降は成果地点だけを保持する範囲 |

縦sliceは新しい永続型、公開API、Document意味、plugin契約を作る分類ではない。
既存仕様とコードに存在する能力を製品UIへ接続する観測単位である。

## 4. 最初の縦slice

M3の最初の完成線を次に固定する。

### VS-1 Rectangle配置とUndo

```text
通常製品routeを起動
  -> product-owned BrowserからRectangleを選ぶ
  -> Stageへ配置をpreviewする
  -> release時だけD2 single writerへcommitする
  -> Stage / Timeline / React Inspectorが同じrevisionとLayerIdを表示する
  -> Undo一回で三面から消える
  -> Redo一回で同じ意味が戻る
```

必須正例:

- diagnostic route、fixture専用shell、CLIを経由せず通常製品routeで完走する
- React Browser、native Stage / Timeline、Host snapshotを読むReact Inspectorが
  一つのrevisionとLayerIdを投影する
- preview中はDocument write 0、確定時は1 gesture = 1 Undo
- PreviewとExportへ渡るRectangle意味は既存D3/Vector経路と一致する

必須負例:

- Cancel、outside drop、capture loss、duplicate、stale、D2失敗でDocument、history、
  revision、ID counterを部分変更しない
- surface別Document、selection、Undo、stable ID counterを作らない
- Rectangle専用の公開planner、raw ID mint、汎用transactionを追加しない
- React sourceの縮約再実装、mock/product二重copy、legacy runtime importを行わない
- UI不足を理由にDocument、journal、plugin契約、永続layoutを変更しない

VS-1には、実装orderより先に閉じる判断と順序改訂がある。旧CU IDは履歴上の対応名として
残すが、粒度化文書の格下げ後は次の表を現在sliceのblocking decision一覧とする。

| Blocking decision | 旧CU対応 | 閉じること | 現行処分 |
|---|---|---|---|
| Local Alpha用platform gate | CU-G01 | **DONE**。G0-9Lは固定Mac prerequisite evidenceだけを限定確定し、G0-9DへWindows・追加hardware・配布対象Macを残す | [G0-9段階化](2026-07-23-m3-g0-9-staged-platform-gates.md)を正とし、W0b/H1b/Preview完成へ外挿しない |
| Selected U seriesの順序 | CU-G02 | U4a/U4c/U2c-2へ先行する現行列と、VS-1に必要なU2h/Rectangle接続の矛盾を解消 | **DONE**。G0-6H人間`ACCEPT`後の次PRODUCT-ASSET粒は`CU-0B02` / `U0e-3`。[M3仕様 運用順](../specs/M3-ui-integration.md)、[implementation-ledger](../implementation-ledger.md)、[decision-index](../decision-index.md)を同期 |
| journal durabilityとsnapshot publish順 | CU-G03 DONE / CU-109S DONE / CU-109SP DONE / CU-G04S0 DONE / CU-G04S DONE / CU-G04SC0 DONE / CU-G04SC DONE / CU-109 DONE（[#425](https://github.com/oshikaidesu/Motolii/pull/425)） / CU-110S DONE / CU-110D DONE / CU-107S DONE / CU-107D DONE / CU-107R DONE / CU-107N DONE / CU-107W DONE | **CU-G03/D/R DONE**。単一command順序とcatalog未反映committed tail recovery guardを閉じた | [CU-G03決定](2026-07-26-cu-g03-edit-durability-ordering-decision.md)と[#369](https://github.com/oshikaidesu/Motolii/pull/369)を正とする。[CU-109S順序再確認](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md)でR2・候補(b)を裁定。[CU-109SP prerequisite決定](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md)でP1を裁定。[CU-G04S0選定](2026-07-27-cu-g04s0-session-source-selection-decision.md)でsession source判断を`CU-G04S`へ分割し、[CU-G04S session source決定](2026-07-27-cu-g04s-edit-runtime-session-source-decision.md)でD1〜D7を閉じた。CU-109再発注STOP後、[CU-G04SC0選定](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md)でproduct path handoff判断を`CU-G04SC`へ分割し、[CU-G04SC product path handoff決定](2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md)でC1〜C6を閉じ、`CU-109`を次PRODUCT-ASSET `DO`へ送った。docs-only `CU-110S`は`DONE`、docs-only `CU-110D`は`DONE`、docs-only `CU-107S`は`DONE`、docs-only `CU-107D`は`DONE`（[CU-107D決定](2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md)で候補(B)を採択し、`CU-110`が必要とする責任範囲の限定を先に閉じる順序を裁定）、docs-only `CU-107R`は`DONE`（[CU-107R決定](2026-07-28-cu-107r-cu-110-required-responsibility-decision.md)）、docs-only `CU-107N`は`DONE`（[CU-107N決定](2026-07-28-cu-107n-cu-107-narrow-prerequisite-closed-set.md)）、docs-only `CU-107W`は`DONE`（[CU-107W決定](2026-07-28-cu-107w-w0-mirror-rewrite-decision.md)）、docs-only `CU-0A08RS0`/`CU-0A08RS`/`CU-0A08RM0`/`CU-0A08RMD`/`CU-0A08BD0`/`CU-0A08BDD`/`CU-0A08SS0`/`CU-0A08SSD`/`CU-0A08SSC`/`CU-0A08SSCD`/`CU-0A08SSCS`/`CU-0A08SSCSD`は`DONE`。`CU-0A08RM`はOpus `ORDER: STOP`により`WAIT`。実装粒 `CU-0A08SSCI` は当初Opus 5 order判定 `STOP` だったが、`CU-0A08BTR`でprivate seam責任をBTPへ吸収して`SPLIT`。docs-only `CU-0A08SSCI-P` は`DONE`。ORACLE-GUARD `CU-0A08SSCI-P1` は`DONE`。`(P)` は authority と guard 実装の両面で閉じた。ORACLE-GUARD `CU-0A08SSCI-T1`は`DONE`。`(T)`はauthorityとguard実装の両面で閉じた。`CU-0A08BTR`は`DONE`、親`CU-0A08BT`は`SPLIT`。`CU-0A08BTP`は`DONE`。`CU-0A08ITP-P`は`DONE`、親`CU-0A08IT`は`SPLIT`、`CU-0A08ITP`は`DONE`、`CU-0A08ITI`は`WAIT`。次PRODUCT-ASSET `DO`は未選定（0件）。`CU-0A08SSCI`はBTPへ実装責任を吸収して`SPLIT`。 |
| Browser catalog projection | CU-G09 DONE / CU-G09O DONE / CU-G09R DONE / CU-0A08BP DONE / CU-0A08BTR DONE | Rectangleを含むcardの型付きread modelとunknown/dangling拒否 | [CU-G09 Browser catalog projection契約決定](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md)、[CU-G09O Browser decoder output契約決定](2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md)、[CU-G09R Browser decoder拒否優先順決定](2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md)とCU-0A08BP実装を`DONE`とした。[CU-0A08BTR依存再締結](2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md)で親`CU-0A08BT`を`SPLIT`し、read-only projection `CU-0A08BTP`を`U4a-2`非依存で実装済みの`DONE`、typed intent `CU-0A08BTI`を既決Place chain待ちとした |
| Rectangle Place意味とidentity | CU-101 DONE / CU-102 DONE | target、start、duration、recipe、position、nameは歴史回収§3.2どおり確認済み。[CU-102決定](2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md)でfresh LayerIdとAddTrackItem原子性も閉じた | 既存Document意味だけで閉じ、appearance・公開raw mint・汎用transaction・CU-110実装を束ねない |
| selection / Undo再投影 | CU-104 DONE / U2h-1S DONE / CU-104E DONE / U2h-1I DONE / U2h-1PR DONE / CU-105R DONE / CU-106S DONE / U3a-2S0 DONE / U3a-2S DONE / U3a-2R DONE / U3a-2Z DONE / U3a-2A DONE / U3a-2P DONE / U3a-2Q DONE / U3a-2Q-P DONE / U3a-2Q-P2 DONE / U3a-2Q-P3 DONE / U3a-2Q-P4 DONE / CU-109S DONE / CU-109SP DONE / CU-G04S DONE / CU-G04SC0 DONE / CU-G04SC DONE / CU-109 DONE / CU-107N DONE / CU-107W DONE | commit/Undo/Redo後にTransient selectionと三面投影をどう更新するか | owner・visibility・generation規則・Apply/Undo/Redo publishは閉じ済み。U2h-1P事前審査でproduction caller 0とlint抑制必須を検出したため、[selection入力到達性決定](2026-07-27-u2h-1p-selection-input-reachability-decision.md)でproducer-only粒を停止し、[CU-106 selection consumer分割決定](2026-07-27-cu-106-selection-consumer-split-decision.md)でprimary consumer CU-106Pへ統合した。存在拒否→same-id no-op→枯渇preflight順を使う。CU-G04S D2後もproduct path carrierが不在だったため、[CU-G04SC0選定](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md)で次PRODUCT-ASSET `DO`をdocs-only `CU-G04SC`へ更新し、[CU-G04SC product path handoff決定](2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md)でC1〜C6を閉じて`CU-109`を次PRODUCT-ASSET `DO`へ送った。実装粒`CU-109`は[#425](https://github.com/oshikaidesu/Motolii/pull/425)で`DONE`、docs-only `CU-107S`とdocs-only `CU-107D`とdocs-only `CU-107R`とdocs-only `CU-107N`とdocs-only `CU-107W`は`DONE`、docs-only `CU-0A08RS0`/`CU-0A08RS`/`CU-0A08RM0`/`CU-0A08RMD`/`CU-0A08BD0`/`CU-0A08BDD`/`CU-0A08SS0`/`CU-0A08SSD`/`CU-0A08SSC`/`CU-0A08SSCD`/`CU-0A08SSCS`/`CU-0A08SSCSD`は`DONE`。`CU-0A08RM`はOpus `ORDER: STOP`により`WAIT`。実装粒 `CU-0A08SSCI` は当初Opus 5 order判定 `STOP` だったが、`CU-0A08BTR`でprivate seam責任をBTPへ吸収して`SPLIT`。docs-only `CU-0A08SSCI-P` は`DONE`。ORACLE-GUARD `CU-0A08SSCI-P1` は`DONE`。`(P)` は authority と guard 実装の両面で閉じた。ORACLE-GUARD `CU-0A08SSCI-T1`は`DONE`。`(T)`はauthorityとguard実装の両面で閉じた。`CU-0A08BTR`は`DONE`、親`CU-0A08BT`は`SPLIT`。`CU-0A08BTP`は`DONE`。`CU-0A08ITP-P`は`DONE`、親`CU-0A08IT`は`SPLIT`、`CU-0A08ITP`は`DONE`、`CU-0A08ITI`は`WAIT`。次PRODUCT-ASSET `DO`は未選定（0件）。`CU-0A08SSCI`はBTPへ実装責任を吸収して`SPLIT`。`U3a-2Q-V`は`WAIT` |

上表は未決を決めたことにしない。各行を閉じる時はM3仕様または既存decisionへ結果を移し、
上表を`DONE`へ更新する。未決SPEC粒を履歴snapshotだけへ残さない。

各行の長文セルに残る「次PRODUCT-ASSET `DO`は未選定」等は、その子粒完了時点の
handoff履歴であり現在選定ではない。本表の現行選定はSelected U series行と
[implementation ledger](../implementation-ledger.md)を正とし、`CU-0B02` / `U0e-3`
だけを次PRODUCT-ASSET `DO`とする。独立ORACLE-GUARD `G0-6H-V1G-RP`は
current-route provenance修復を所有し、PRODUCT-ASSET順を停止・代替しない。

`U0e-2R`、React R0〜R6、上表のblocking decision、G0-9の採択済みlocal platform証拠、
U3a、U2h、Rectangle D2/D3の各作業はVS-1のenabling setである。これらの一つが完了しても
VS-1完成とは記録しない。未決意味または既存停止線に到達した場合は、上表または既存authorityへ戻り、
VS-1の見かけだけをfixture special-caseで成立させない。

## 5. 後続sliceとrolling horizon

VS-1以外は本方針時点でclosed orderの粒へ展開しない。次の二sliceは出口と主依存だけを保持する。

| Slice | 利用者成果 | 出口 | 主依存・未決入口 | 現時点で固定しないもの |
|---|---|---|---|---|
| **VS-2 値と時間を編集する** | VS-1のRectangleを選択し、値、key、easingを変更してseek/playbackし、Undo/Redoする | Stage / Timeline / Inspectorが同じ時刻・対象・意味を表示し、1 gesture=1 Undo | VS-1、selection policy、D5/U5段階完了判断（旧CU-G07）、既存U3b/U4a/U4b/U5 | 全parameter UI、全Timeline機能、全Transport品質、具体order分解 |
| **VS-3 実素材とprojectを往復する** | 素材を追加し、Save、終了、reopenして同じprojectを再表示する | 通常製品routeで同じDocument意味とasset参照が戻り、欠落/corruptを型付き表示する | VS-1、project lifecycle判断（旧CU-G04）、durability判断、既存U6/D1c/D1m | Save As/Unsaved Changes等の未決UX、全codec、Export完成 |

それ以降は次の成果地点だけを保持し、前sliceの実装事実が得られるまで粒化しない。

- Exportと失敗/cancel/atomic output
- Local Alphaの日常操作、応答、reload/crash復旧
- Distribution Readyのplatform別同一制作fixture

VS-1が閉じた時点でVS-2を発注可能な精度へ上げ、VS-3の依存を再監査する。
遠いsliceのCU粒を事前に修復し続けない。

旧粒度化文書にだけ残るCU-G04〜G10、CU-5B01、CU-605等は、採択済み意味ではなく
後続sliceの未決候補である。次の二sliceでは上表の主依存名だけを保持し、blocking decisionへ
細分化しない。対象sliceが**現在sliceへ昇格する時**、必要な問いだけを本書のblocking decision表
またはM3仕様へ昇格し、authority、依存、STOPを再監査する。昇格前の候補を実装defaultや
発注根拠にしない。

slice集合の終了線は[快適利用Work Map](2026-07-22-m3-comfortable-use-work-map.md)で既決の
`Local Alpha`と`Distribution Ready`だけとする。後続sliceはその二つの完成線を
満たす経路を細分化・再構成できるが、完成線自体を置換しない。完成線の変更はWork Mapと
decision-indexの通常改訂、反対側レビューへ戻る。新しい完成線を追加してM3を延長せず、
新しい表現意味、M4のcache/resource能力、M5の3D/post能力をM3 sliceとして追加しない。

## 6. A〜Dと既存規律の扱い

A〜Dはsliceを順番に支配する親stageではなく、各enabling orderへ必要なものだけ割り当てる。

| Checklist | 問うこと |
|---|---|
| A Presentation Ownership | このorderが触るsource assetと単一ownerは確定しているか |
| B Host Projection / Intent | 表示はHost snapshotのread-only投影で、操作はtyped intentか |
| C Product Runtime Integration | 通常製品routeと該当platformで正しいsurface ownerへ接続されるか |
| D Editing Loop | Transient preview、D2、Undo、snapshot再投影が一続きか |

一つのorderがA〜Dを全部閉じる必要はない。`NOT APPLICABLE`を量産するstage分類は要求しないが、
stage packetが保持していた安全情報は削らない。全M3 orderは名称を問わず次を持つ。

1. `M3 ENTRY EVIDENCE`: 今回の境界へ入場できる直前成果と、未到達の依存
2. `M3 CLOSES`: 今回だけで閉じる一つの契約境界
3. `M3 DOES NOT CLOSE`: slice全体、後続接続、platform、D2、配布、公開契約等の残差
4. `M3 STATE OWNER`: 該当するDocument / User settings / Workspace / Project session /
   Transient / local presentationと、非owner
5. `M3 AUTHORITY / TASK IDS`: 現行spec、decision、既存G/U ID、固定source/fixture
6. `M3 POSITIVE ORACLE`と`M3 NEGATIVE ORACLE`
7. `M3 STOP / RETURN`: 停止条件と、戻るblocking decision、spec、asset、core、platform authority
8. `M3 HANDOFF`: VS-1内の次orderへ渡す成果と、まだ`WAIT`の依存

これはA〜Dのstage名や前stage全完了を要求するものではない。order間の証拠、owner、非目標、
戻り先を機械的に確認する最小handoff contractである。React/Rerun必須blockが同じ情報を
より強い固定ラベルで要求する場合も、既存の規定順と機械guardを維持する。
発注書実体では各`M3 `ラベルをラベルだけの独立行にし、内容は次行以降へ書く。
`M3 `prefixを外してReact blockの`STATE OWNER`、`NEGATIVE ORACLE`等を先取りしない。

次は変更しない。

- React source assetのpresentation移管とHost state接続、WebView統合、D2 commitを
  一つの発注へ束ねない
- React必須8ラベル、Rerun必須6ラベル、変更許可ファイル、非目標、STOP、必須負例
- GR-UI、GR-PV、single writer、VRAM常駐、Preview/Export同一関数
- G0-9、G0-3 / GAP-13、G0-6H、未決意味、hardwareの既存停止線
- 1 ticket = 1 commitと独立検収

縦sliceは安全境界を広げる許可ではなく、複数の安全なorderが最終的にどの利用者成果へ
合流するかを一意にする進捗軸である。

## 7. 文書責任の整理

| 文書 | 今後の責任 | 持たせないもの |
|---|---|---|
| M3仕様 | 作品意味、G/U ID、依存、完了条件、実装ガード、採択済みblocking decision | W/CU/stageごとの重複説明 |
| 本決定 | 縦sliceの進捗原則、最初のslice、rolling horizon、未決blocking decisionの現在一覧、文書移行 | 個別実装file/API |
| 快適利用Work Map | Local Alpha / Distribution Readyと遠方の利用者成果地図 | 個別order、現在状態、全粒のSTOP |
| 粒度化文書 | 2026-07-22時点の候補分解・依存監査の履歴資料。tooling移行までは粒IDと状態cellだけを暫定dispatch台帳として保持 | tooling移行後の現在地、全将来orderの意味・順序拘束、継続的な粒修復 |
| implementation ledger | 現在slice、そのslice内の現在order、直後のhandoff | 意味・完了条件・全将来粒の再記述 |
| closed order | 今回のallowlist、非目標、正負oracle、STOP、commands | M3全体地図の再掲 |

同じ完了条件を複数文書へ手書きしない。表示用の対応表が必要な場合は、
正本IDへの参照だけを持ち、別の意味・依存・STOPを加えない。

## 8. 既存文書の非破壊移行

本方針の採択時は、既存G/U/CU/W ID、過去のFable証跡、完了済みtaskを削除・改名しない。

1. 本書§4のblocking decision一覧を現行入口にし、未決SPEC粒を履歴snapshotから切り離す
2. CU-G01が指していたG0-9段階化改訂を先に、または次の順序改訂と同じ原子的変更で閉じる。
   その上でCU-G02が指していたSelected U seriesの矛盾を解消する仕様・ledger順序改訂案を作り、
   read-only反対側レビューを通す
3. M3仕様のA〜Dを「統合の背骨」から「縦slice接続checklist」へ変更する
4. VS-1をM3の最初の製品完成線としてM3仕様とledgerへ登録し、手順2の順序改訂を
   同じ変更で反映する
5. Work MapはW0〜W6の順序を強制実行列でなく、VS-1以降が到達する成果地図として残す
6. 粒度化文書を`order draft母集団`から`候補分解の履歴snapshot`へ格下げする。ただし手順11が
   mainへ到達するまで、`scripts/delegate-cursor-supervised.sh`が読む粒IDと`DO/WAIT`状態cellだけは
   暫定dispatch authorityとして残す。Codexは現在orderの遷移時だけledgerと同じ変更で該当cellを更新し、
   意味、依存、STOPを粒度化文書へ新たに追加・修復しない。対応する旧CU rowが無い新orderは
   tooling移行前にdispatchせず、mirror専用の新しい粒を作らない
7. `CU-0A01 = U0e-2R`等の対応は履歴として残すが、CU粒の完了数をM3進捗に使わない
8. ledgerは`現在slice = VS-1`、`現在order = CU-0A03 / R0 source inventory`、`blocking decision`を別欄で表示する。U0e-2R/U0e-2とG0-9Lは完了済みとして再選択しない
9. decision-indexのM3-A〜D、快適利用、粒度化の現行決定行を本方針へ同期し、
   古いstage packet必須とorder draft母集団の記述を現行として残さない
10. `docs/README.md`と`docs/reviews/README.md`へ本決定を登録し、Work Mapと粒度化文書の
    状態表示を同期する
11. 独立tooling変更で`GRAIN_LEDGER`と`GRAIN:`/`DO` dispatchを粒度化文書から外し、
    implementation ledgerの「現在slice / 現在order / 状態」を単一dispatch sourceとして検査する。
    同じ変更で全M3 orderの最小handoff contractを本書§6へ切り替える機械guardと正負testを追加し、
    prepare promptに残る「main comfortable-use granulation ledgerのDO」記述と
    `scripts/test-delegate-cursor-supervised.sh`のCU grain fixtureも新しいdispatch sourceへ同期する。
    既存React/Rerun guardを弱めず、移行後にだけ粒度化文書の粒ID/state cellを履歴へ固定する
12. VS-1の次に必要なorderだけを、実装事実に基づいてrollingに再判定する

手順1〜10を同じdocs変更で閉じるまで、旧stage packetと現行Selected U seriesを拘束から外さない。
途中状態で新旧二つの意味・順序authorityを混用しない。手順11がmainへ到達するまで、
orderはledgerで選んだ現在orderに対応する粒ID/state cellと旧stage packetを機械入力として使う。
この期間も現在地と順序の人間向け正本はledgerであり、粒度化表はdispatch mirrorに限定する。

移行時に遠いCU粒を最新仕様へ修復しない。参照される現在orderだけ既存authorityと照合する。
履歴文書を削除して過去の判断根拠を失わず、現行拘束から外す。

## 9. 方針のSTOP

次の場合は縦sliceの名のもとに実装を進めず、既存authorityへ戻る。

- slice出口に新しいDocument意味、公開API、journal、plugin/community契約が必要
- 通常製品routeでなくdiagnostic routeまたはfixture専用経路だけで成立する
- 一つのorderへpresentation、Host connection、runtime integration、D2を束ねないと
  sliceを進められない
- platform、hardware、人間審判をsynthetic testや別OSの結果で代用する
- slice専用helper、planner、state、Document clone、UI側Undoで既存境界を迂回する
- visual threshold、golden、期待値を変えないと合格しない
- 遠いsliceの未決を現在sliceの実装defaultとして先取りする

## 10. Fableへ問うこと

Fableは実装担当や仕様authorityではなく、本文書をread-onlyで反対側監査する。

1. 横段階から縦sliceへ進捗軸を移しても、React直接移管、single writer、platform gate、
   公開契約停止線を弱めていないか
2. VS-1は利用者成果として十分に縦で、単なる別名の巨大粒になっていないか
3. roadmapを縦、orderを一契約境界とする二層分離に循環や到達不能がないか
4. rolling horizonにより必要な先行意味判断や負例を遅らせすぎないか
5. A〜D stage packetから削ってはならない情報があるか
6. Work Map、粒度化文書、ledgerの格下げでauthorityが曖昧にならないか
7. 既存G/U/CU/W IDを非破壊に残しつつ、LLMが古い粒を現行orderと誤認しないか
8. 本方針が粒化作業を自己増殖させない機械的な終了条件を持つか

P0/P1、未解決のauthority衝突、既存停止線の弱体化があれば採択しない。
Fableの助言はCodexが実ファイルと既存正本へ照合し、採否を本文書へ記録する。

## 11. FableレビューとCodex採否

| 回 | 判定 | 指摘 | Codex採否 |
|---|---|---|---|
| 初回 | `VERDICT: REJECT` | P0=0 / P1=3 / P2=3。未決SPEC粒の置換先不在、entry evidence/state owner/return先の脱落、Selected U順序矛盾の修正経路消失。索引同期、Inspector所有表記、slice集合終了線も不足 | 全件採用。§4へblocking decisionの現行一覧、§6へ最小handoff contract、§8へ順序改訂・索引・toolingの原子的移行を追加。Inspector表記と二完成線による終了条件を修正 |
| 再回 | `VERDICT: REJECT` | P0=0 / P1=1 / P2=4。handoff labelがReact機械guardと衝突。VS-2/3主依存、CU-G01→G02順、完成線の「置換」、tooling過渡期も曖昧 | 全件採用。handoffを`M3 `prefixへ統一し独立行書式を維持。後続sliceの主依存、G0-9段階化の先行、完成線不変、§3.1暫定機械authorityを明記 |
| 第3回 | `VERDICT: REJECT` | P0=0 / P1=1 / P2=3。実scriptは§3.1でなく粒ID/state cellをdispatch gateとして読む。header、索引、rolling昇格時点も残差 | 全件採用。粒ID/stateだけを暫定dispatch mirrorとしてledgerと同期し、tooling変更で`GRAIN_LEDGER`参照ごと移管する。header・索引を同期し、blocking decisionへの昇格を現在slice移行時へ固定 |
| 最終 | `VERDICT: ACCEPT` | P0=0 / P1=0 / P2=2。tooling移行時のprepare prompt/test fixture同期と、mirror専用grain追加禁止を明記するとより安全 | 全件採用。§8手順6/11へ追記。既存STOP、React/Rerun guard、single writer、platform gateの弱体化0をCodexが再確認 |

最終レビューのACCEPTは助言であり、Codexが実script、現行ledger、粒ID/state、M3仕様、
React直接移管契約を照合してP0/P1=0を確認した。本決定は縦sliceを進捗軸へ採択するが、
§8の非破壊移行と既存停止線を完了前に解除しない。
