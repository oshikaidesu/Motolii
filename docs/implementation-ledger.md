# 実装進行台帳

最終確認: **2026-07-25**

このファイルは、実装者が「次に何をするか」を1枚で判断するための現場用台帳。M0〜M5の意味や完了条件を再定義せず、現在の依存関係と発注順だけを示す。

## 使い方

1. まず本ページの「現在の並列レーン」を確認する。PRODUCT-ASSET内だけは現在選択中の1件を守る。
2. Issueと該当する[マイルストーン仕様](specs/README.md)のタスク行・実装ガードを読む。
3. 依存が1件でも未mergeなら着手しない。
4. 完了時は、実装PR内で仕様のタスク表と本ページを同時に更新する。

情報が食い違う場合の優先順位は次の通り。

1. **意味・完了条件**: `docs/specs/M*.md` と判定済みdecision文書
2. **実際のmerge状態**: GitHub Issue / PR / main
3. **発注順・現在地**: 本ページ
4. **未仕様化の候補**: [backlog.md](backlog.md)

本ページを根拠にschema、公開API、既存タスクの意味を変更してはならない。

## 状態語

| 状態 | 現場での意味 |
|---|---|
| `DO` | 意味と完了条件が固定済み。記載依存のmerge確認後、今すぐ着手できる |
| `ISSUE` | 意味は固定済み。最新mainで型名を再確認してIssue化する |
| `READY-RECHECK` | 依存は満たしたが、依存元の成果が当該粒の必要責任を含むか再判定するまでclosed orderを作らない |
| `WAIT` | 後続タスク。依存が終わるまでIssue化・実装しない |
| `DECIDE` | 意味または公開契約が未決。decision/spec PRだけ進める |
| `ACTIVE` | 実装または修復が進行中。重複着手しない |
| `DONE` | main到達済み |
| `LATER` | v1.xまたはv2へ明示延期 |

## 現在地

| Phase | 状態 | 現在の出口 |
|---|---|---|
| M0 | `DONE` | spike完了。S2は`ffmpeg-sidecar`クレート不採用、自前子process pipe／CFR seek成立まで。VFR、長尺／4K、pool、停止中readのkillは製品完成証拠にせずK4／GAP-26へ分離 |
| M1 | `DONE` | exit demo・E2E golden・凍結ゲート宣言済み。RenderTargetPoolは直列2枚を下限にbranch livenessで伸長するが、O(n²)未来scan最適化、fp16／path fusion、40-layer性能は未成立。R9/T11は当時の歴史sign-offで、現行製品Stage／実素材release受入はGAP-32。出荷hardening候補G1〜G8は完了条件外で、2026-07-23再照合の未到達process/artifact reliabilityはGAP-26、GPU health分類はGAP-27、同期export readbackの原因分離／staging採択はGAP-29、GPU RGB→YUV export接続はGAP-31。G7の同期1-frame boundedをcopy重畳完成、decode側GPU色変換や出力tagをinverse変換完成とみなさない |
| M2 | **基盤再締結済み / narrow follow-up pending** | D1lのDocument／lifecycle意味、D3e、D1m、CAM-G0→D1j→D1k-S→D1k→D3fとA〜C証跡はmain発効済み。2026-07-23監査で`new_v1` enforcement driftをGAP-23、known Edit apply failureのsnapshot fallback driftをGAP-24、semantic oracle gate自己保護をGAP-25として狭く再開。Param Pipeline／Element Domain／Constraint GraphはM2-GAP-15の解凍gate前は未実装のままが正しい。D5は骨格到達・統合審判pendingで、AG-2 mixer coreは成立したが製品`PlaybackSession`のmixed `AudioProgram`接続はGAP-28。D1n external revisionも未実装で、cloud-safe公約不可 |
| M3 | **VS-1 Rectangle配置とUndo / CU-0A08BP Browser catalog decoder** | React chrome + native Stage/Timeline + headless interaction、1 top-level wgpu Surface + 2 native viewport + opaque child WebView islandsを正本化。R4CまでのInspector product ownershipとCU-0A08IPは完了した。CU-G09を単一oracleに、CU-0A08BPだけを次のproduct-owned・非export pure decoder粒とする。Motolii Studio Previewは未実装。visible summary chromeは未決で実装しない。G0-6HはU0e-3を止める並行人間審判、G0-9DはDistribution Ready用hardware gate。eguiへ新規製品面を実装せず、plugin UI公開契約はG0-3 / GAP-13まで停止する |
| M4 | **K0契約凍結済み(test-only) / K1a再判定** | [歴史20版再照合](reviews/2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md)と[memory model 6版再照合](reviews/2026-07-23-historical-memory-model-lineage-recovery.md)後、K0はRoD/RoIの契約意味をtest-only spikeで凍結済みで、K1〜K8は未実装。runtime実装はK1a以降の別粒とし、K1階層基盤→K7 group freeze→K8全曲Draft coverageへ進む。現行`PipelineCache`／dynamic target pool／wgpu budget thresholdをResourceLedger、copy-out、disk store完成と数えない。K4の恒久`source_id`／再リンク／package意味はGAP-3／7の再調査前に焼かないが、このgateをK0や独立K1へ広げない。K6のVello／usvg製品統合は未実装で、R8成立性だけを完成証拠にしない |
| M5 | **identity meaning decision可 / fixture WAIT** | P0I自身が所有するcontinuity／transform／nested identity／寿命／cache入力境界／PRNG処分をdocsで先に閉じ、TextCluster内部写像とPrototype ownerは明示留保する。fixtureはdecision merge後に分割する。P6のfontique／harfrust／Vello text stackは未実装で、K6とpremul adapterを重複実装しない |

[M2基盤再締結ゲート](reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み。M3はU0a入場済みで、[UI runtime責任境界](ui-runtime-architecture.md)と[G0-9段階化](reviews/2026-07-23-m3-g0-9-staged-platform-gates.md)も決定済み。G0-9Lは固定Macのplatform prerequisite evidenceだけを限定確定したが、W0b、H1b、Motolii Studio Preview、window結合を解禁しない。G0-6Hは独立し、U0e-3とW0bの製品前提を止め続ける。G0-9DまでDistribution Readyを名乗らない。plugin UI公開契約はG0-9合格と分離し、G0-3 / GAP-13の決定まで発注しない。headlessなTimeline/Stage projectionもSelected U seriesの前枝番がmainへ到達した時だけ次の1枝番を発注する。

### M3の1件を選ぶ動線

M3は[縦slice実行方針](reviews/2026-07-24-m3-vertical-slice-execution-decision.md)を使い、
一つの現在sliceと、その出口へ必要な一つの契約境界だけを現在orderにする。

1. `decision-index.md`で主題を逆引きする
2. 下の「現在の並列レーン」にあるPRODUCT-ASSETの現在粒と、現在sliceのblocking decision、
   暫定mirrorの`DO/WAIT/STOP`を照合する
3. `M3 ENTRY EVIDENCE`で直前成果と未到達の依存を確認する
4. `M3 CLOSES / M3 DOES NOT CLOSE / M3 STOP / RETURN / M3 HANDOFF`を固定してからIssue化・発注する
5. main到達後、spec task表・本台帳・証拠を同じ変更で更新して次orderをrollingに再判定する

既存G/U IDが意味と完了条件の正本であり、M3-A〜Dは各orderの接続checklistである。

## 主クリティカルパス

```text
Shared Effect:
D1l DONE → D3e → U2g（M3入場後）→ K2

Selected U series:
U0a DONE → U0b-1 DONE → U0b-2 DONE → U0c-1 DONE → U0c-2 DONE → U0d-1 DONE → U0d-2 DONE → U0d-3 DONE
→ U2a-0 DONE → U2a-1 DONE → U1a-1 DONE → U1a-2 DONE → U1b-1 DONE → U1b-2 DONE → U2b-1 DONE → U2c-1 DONE
→ U2c-4 DONE → U0e-1 DONE → U0e-2R DONE → GR-D1 DONE → GR-D2 DONE → GR-R1/R2 DONE → GR-R3 DONE → U0e-2 DONE
→ G0-9L DONE → R0 source inventory DONE → R1 Browser ownership DONE → R2A/R2B Easing DONE → R3A/R3B KEYS/LAYERS DONE → R4S/R4A/R4B/R4C DONE → CU-0A08I SPLIT → CU-0A08IS DONE → CU-0A08IP DONE → CU-G02 DONE → U3a-1 SPLIT → U3a-1S DONE → U3a-1I DONE

Parallel evidence:
G0-6H HUMAN（U0e-3だけを停止） / G0-9D WAIT-HARDWARE（Distribution Ready）

Unified Camera:
CAM-G0 → D1j → D1k-S → D1k → D3f → U1f #169 → U2d

Rerun learning（製品実装ではないsource監査はM3入場前も可）:
RR-0 inventory → RR-1〜RR-8 asset判定 → RR-9統合縦切り
詳細: reviews/2026-07-20-rerun-learning-transfer-plan.md

Editor scripting:
U2a → U2b → U9a → U9b → U9c
F-11 + K0 → K1b + K1c → K7 → SCR-4 (Accumulation/Feedback Canvas)

Bounds / cache:
D3 → K0 #167 → K1b → K2

Resource pressure / preview:
K0 → K1a → K1b → K1c
K1c + K4 → K1d
G0-8 + K1a → U0f
U1b + U1c + U5 + K1d → U1g → U1h

MV whole-song cache / freeze:
K1b + K1c + D3 → K7a
K7a + K2 → K7b → K7c → U8b
K1d + D3 → K8a
K7c + K8a + D5 → K8b

Duplicator:
P0I #170 → P7a → P7b → P7c → P7U
```

## 現在の並列レーン

現在sliceは**VS-1 Rectangle配置とUndo**。PRODUCT-ASSET laneは意味・所有境界を優先して
1チケットずつ進めるが、この直列性を他の独立contract／repair／authoring laneへ波及させない。
現在の全lane、変更path、STOP、Human Response Frontierは
[並列レーン着手地図](reviews/2026-07-25-parallel-lane-readiness-map.md)を正とする。
旧night 3分岐は直接統合しない。

| lane | 現在粒 | Phase / slice / checklist | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| PRODUCT-ASSET | CU-0A08IP | M3 / VS-1 / B / Inspector fixture decoder | `DONE` | — | [Inspector read-model inventory](reviews/2026-07-26-cu-0a08is-inspector-read-model-inventory.md)でCU-0A08IS閉包とCU-0A08IP着手境界を固定。`node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs` 39/39、`npm run test:reference-guard` 172/172 | product-owned・非export pure decoder module。fixture/testのみ。Host transport・intent・JSX binding・`S`行・Rust/schema/plugin変更は非目標 |
| PRODUCT-ASSET | CU-G02 | M3 / VS-1 / SPEC / Selected U series order | `DONE` | — | CU-G01は[G0-9段階化](reviews/2026-07-23-m3-g0-9-staged-platform-gates.md)で完了済み。本変更でSelected U seriesの次実装粒をU3a-1に固定した | 当時の次PRODUCT-ASSET粒は`U3a-1`。U4a-1/U2h-1は未選択、Rectangle blocking decisionは個別粒へ分離した。現行状態は本表の各粒を正とする |
| PRODUCT-ASSET | U3a-1S | M3 / VS-1 / SPEC / headless Timeline owner-visibility decision | `DONE` | — | 本変更で[U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)を確定。`U0a`/`U0b-1`/`U0b-2`は[発注依存証跡](#発注依存証跡)で`DONE` | owner=`motolii-ui`内module、`motolii-timeline` crate=`REJECT`、visibility=pub再export+integration test（制約6点）。後続の同名docs粒を作らない |
| PRODUCT-ASSET | U3a-1I | M3 / VS-1 / B / headless Timeline projection | `DONE` | — | `U3a-1S`は[発注依存証跡](#発注依存証跡)で`DONE`。意味正本は[specs/M3-ui-integration.md](specs/M3-ui-integration.md) U3a行、owner/visibilityは[U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)。G0-6H/U0e-3/U2c-3/U2c-5/G0-9は入場条件にしない | toolkit/renderer非依存のDocument→Timeline projection/layout/cull/hit-testを小さなfixtureで閉じる。G0-9や100k再実測を入場条件にしない |
| PRODUCT-ASSET | CU-G03 | M3 / VS-1 / SPEC+M2 recovery prerequisite | `DONE` | — | `CU-G03D`と`CU-G03R`へ分割し、両子粒を完了 | CU-109はUndo/Redo prepared-action順序の再確認後に選定 |
| PRODUCT-ASSET | CU-G03D | M3 / VS-1 / SPEC / edit durability ordering decision | `DONE` | — | [CU-G03決定](reviews/2026-07-26-cu-g03-edit-durability-ordering-decision.md)で、既存D1m/D2/U2b-1 authorityへ照合し、単一command actionのdurability/publish順序とfailure authorityを確定 | 新payload、複数command耐久、CU-109 runtime配線を含めない |
| PRODUCT-ASSET | CU-G03R | M2 prerequisite / committed Edit tail recovery guard | `DONE` | [#369](https://github.com/oshikaidesu/Motolii/pull/369) | catalog未反映committed tailをMainFile fast pathから既存replayへ送るprivate guard、stale-catalog負例、二重checkpoint対照。Grok ACCEPT P0/P1=0、CI 4/4 | catalog repair/truncate、新形式、GAP-24、poison/write拒否、CU-109は非目標のまま |
| PRODUCT-ASSET | CU-101 | M3 / VS-1 / SPEC / Rectangle Place meaning | `DONE` | — | [U2b-2 Place product core再採択](reviews/2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#3-u2b-2-place-product-core再採択) §3.2を現行authorityとして確認 | mandatory composition、top-level target、Rect recipe/size、canonical drop、Rectangle名、playhead〜composition endを閉じた。appearance/identity/durabilityは別粒 |
| PRODUCT-ASSET | CU-102 | M3 / VS-1 / SPEC / fresh LayerId + AddTrackItem atomicity | `DONE` | — | [CU-102決定](reviews/2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md)で歴史回収§3.1と既存D2/LayerIdTable/AddTrackItemへ照合 | fresh live-next一致+live不在、live mint 0、AddTrackItem 1件/apply_macro 1回、失敗不変、journal互換を閉じた。CU-110実装は非目標 |
| PRODUCT-ASSET | CU-G09 | M3 / VS-1 / SPEC / Browser catalog projection contract | `DONE` | [2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md](reviews/2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) | `CU-0A04`は[発注依存証跡](#発注依存証跡)で`DONE`。縦slice blocking decisionと[快適利用粒度化 CU-G09](reviews/2026-07-22-m3-comfortable-use-granulation.md#cu-g09-browser-catalog-projection契約)、および[Browser catalog projection契約決定](reviews/2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md)へ照合 | Rectangleを含むcardのtyped read model、unknown/dangling拒否、ID/label/thumbnail tokenからの推測禁止だけを決定。CU-0A08B実装は束ねない |
| PRODUCT-ASSET | CU-0A08BP | M3 / VS-1 / B / Browser catalog decoder | `DO` | — | `CU-0A04`と`CU-G09`は[発注依存証跡](#発注依存証跡)で`DONE`。CU-G09 §4/§6/§7を単一oracleにする | product-owned・非export pure decoder、fixture/testのみ。typed intent、Host transport、JSX binding、drag payload、`S`行はCU-0A08BT以降 |
| PRODUCT-ASSET | CU-0A08BT | M3 / VS-1 / B / Direct Browser connection | `WAIT` | — | CU-0A08BP + U4a-2 Direct製品入口 | scoped identityを保つHost projection/typed intent/JSX binding。bare ID推測、別intent終端、UI所有Undo/selectionは禁止 |
| PRODUCT-ASSET | CU-0A08IT | M3 / VS-1 / B / Direct Inspector connection | `WAIT` | — | CU-0A08IP `DONE` + U4a-2 Direct製品入口未成立 | Host transport・typed intent・JSX binding・`S`行は非目標のまま。Advanced入口のU4cはU2c-2依存 |
| VISUAL-RESPONSE | G0-6H | M3 evidence / VS-1 / visual | `DO / HUMAN` | — | 5 reference screenと30 PNG | U0e-3だけを解禁可 |
| AUTHORING-SCAFFOLD | VSM-A4S | Vism / spec | `DO / SPEC` | — | VSM-A1/A2/A3、仕様と実装の別PR決定 | VSM-A4Iは全体レビュー後 |
| DELEGATION-GUARD | GR-D3 | supervised runner / derived output closure | `DONE` | [#329](https://github.com/oshikaidesu/Motolii/issues/329) | [#336](https://github.com/oshikaidesu/Motolii/pull/336)をmainへ統合。専用runner試験、workspace、docs、実K0停止形、Grok `ACCEPT`で閉包 | 解禁後のK0は[#338](https://github.com/oshikaidesu/Motolii/pull/338)で完了。既知派生物だけのfail-closed清掃を後続発注へ維持 |
| SPATIAL-CONTRACT | K0 | M4 / contract spike | `DONE` | [#167](https://github.com/oshikaidesu/Motolii/issues/167) | [PR #338](https://github.com/oshikaidesu/Motolii/pull/338)をmainへ統合。2 file、15/15 test、workspace/fmt/clippy/docs green、`cursor-grok-4.5-high` `VERDICT: ACCEPT` P0=0 P1=0 P2=1。旧K0隔離差分は不採用のまま | K1系は自動起動しない。責任最小化ゲートでseat単位に再判定する |
| IDENTITY-CONTRACT | P0I | M5 / identity decision | `DECIDE` | [#170](https://github.com/oshikaidesu/Motolii/issues/170) | 凍結ゲート、2026-07-15決定。Text／Prototype側の未決は留保 | 意味decision後にfixture粒を分割して再判定 |
| M2-REPAIR | GAP-23 | M2 / narrow repair | `WAIT` | — | 独立D1i-4 LookAt/Follow oracle分離の採番・完了 | GAP-23全25件を再発注し、後にGAP-24を判定 |
| M2-REPAIR | GAP-24 | M2 / narrow repair | `WAIT` | — | GAP-23 | GAP-24後にclose |
| ORACLE-GUARD | GAP-25 | M2 / guard repair | `DO / CHECK-PATH` | — | GAP-23との許可path非重複 | semantic oracle gate自己保護 |
| PRODUCT-ASSET | U2c-2 | M3 / VS-2 / D | `WAIT` | — | U4a-2（Direct）製品入口とU4c（Advanced）製品入口 | 実在入口のDocument意味/Undo同値conformance |

### 独立 History tooling lane

[歴史価値回収の意味グラフ補助](reviews/2026-07-23-historical-semantic-graph-recovery-tooling.md)は
製品境界を変更しない独立tooling laneとして並行できるが、同lane内では`DO`を1件だけにする。

| 優先 | ID | Phase | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| 0 | HVR-G01 | History tooling | `DONE` | — | 意味グラフ補助境界を正本化 | HVR-D01の依存 |
| 1 | U0e-2R | M3 | `DONE` | — | 固定React baseline `eb16d06`を最新mainへ再結合し、43 visual testとworkspace gateを通過 | GR-D1を単独実行 |
| 2 | GR-D1 | M3 guard | `DONE` | — | Terra実装 + Grok検収の通常発注入口へBASE_REF/SHA・authority・粒状態・React labelのdispatch gateを固定し、専用負例とworkspace試験を通過 | GR-D2を単独実行 |
| 3 | GR-D2 | M3 guard | `DONE` | — | 変更許可閉集合、append-only検収証跡、timeout分離、検収者mutation拒否、検収resumeをrunnerへ固定し、専用負例とworkspace試験を通過。旧Terra + Grok backendはアーカイブし、防護をOpus 5 + Spark + Grokへ継承 | GR-R1/R2をDOへ移す |
| 4 | GR-R1/R2 | M3 guard | `DONE` | — | manifestのpath/export/SHAとAST/PostCSS closure、実render nodeへのfixture投影、route隔離、三層因果、raw token拒否を43負例・44 visual・Grok P0/P1=0で固定 | GR-R3をDOへ移す |
| 5 | GR-R3 | M3 guard | `DONE` | — | immutable generation + atomic CURRENT、PNG byte/RGBA、閉schema、17 I/O失敗点、全negative matrixを70 testsとGrok P0/P1=0で固定 | U0e-2をDOへ戻す |
| 6 | U0e-2 | M3 | `DONE` | — | 同一三層fixtureを既存React Surfaceへ直接投影する5 reference screen、固定Chromium/font、normal+5派生の30 PNG、provenance/atomic generation/read-only checkを固定 | G0-6H assisted human stop |
| 7 | U2c-2 | M3 | `WAIT` | — | U4a-2のDirect製品入口とU4cのAdvanced製品入口が揃うまで空harnessを作らない | 実在入口のDocument意味/Undo同値conformance |
| 8 | HVR-D01 | History tooling | `DONE` | — | HVR-G01完了。既存corpus/receiptを変更しない | 決定的な可搬projectionと負例 |
| 9 | HVR-D02 | History tooling | `DONE` | — | HVR-D01完了 | 任意のBasic Memory runner |
| 10 | HVR-D03 | History tooling | `DONE` | — | HVR-D01完了 | repo-local候補packet skill |
| 11 | HVR-D04 | History tooling | `WAIT` | — | HVR-D01〜D03完了 | Unit 5N以降へ候補packetを投入 |

K0 [#167](https://github.com/oshikaidesu/Motolii/issues/167)のcontract spikeはPRODUCT-ASSETと並行して完了した。
P0I [#170](https://github.com/oshikaidesu/Motolii/issues/170)のdocs decisionは引き続きPRODUCT-ASSETと同時着手できる。
Selected U seriesの一時点`DO`一粒はPRODUCT-ASSET lane内だけに適用し、P0I decision、
M2 prerequisite、Vism spec laneを同じ待ち列へ入れない。P0I fixtureとGAP-23実装は各lane-localな前提待ちで、
共有contract変更または変更許可pathの重複が判明したlaneだけをSTOPし、他laneは継続する。

## 発注依存証跡

`DEPENDENCY`の機械判定専用表。現在粒の依存を散文や別phaseの状態から推測せず、この表で`DONE`の
一意な行だけを受理する。完了証拠が変わった時は、該当spec／decisionと同じ変更で更新する。

| ID | 状態 | 完了証拠 |
|---|---|---|
| CU-0A05A | `DONE` | [#341](https://github.com/oshikaidesu/Motolii/pull/341)でR2A mock-side extraction完了。現在粒`CU-0A05B`の直前product asset |
| CU-0A05B | `DONE` | [#344](https://github.com/oshikaidesu/Motolii/pull/344)でR2B product ownership完了。次粒`CU-0A06`をREADY-RECHECKへ送る |
| CU-0A06 | `SPLIT` | readiness再確認で独立source不在を確認し、CU-0A06A mock-side extraction→CU-0A06B product ownershipへ分割 |
| CU-0A06A | `DONE` | [#347](https://github.com/oshikaidesu/Motolii/pull/347)でR3A mock-side extraction、6状態oracle、独立JSX/CSS current closure、Grok ACCEPTを完了。現在粒`CU-0A06B`の直前product asset |
| CU-0A06B | `DONE` | [#350](https://github.com/oshikaidesu/Motolii/pull/350)でR3B product ownership、mock consumer反転、二重copy 0、Timeline-state漏洩拒否、Grok ACCEPTを完了。次粒`CU-0A07`をREADY-RECHECKへ送る |
| CU-0A07S | `DONE` | [R4 readiness分割決定](reviews/2026-07-25-cu-0a07-r4-readiness-split-decision.md)で独立React source不在、skeleton非採用、R4A oracle→R4B mock React化→R4C ownershipの順序を固定 |
| CU-0A07A | `DONE` | [#353](https://github.com/oshikaidesu/Motolii/pull/353)で4 cross-page状態＋1 React-only状態、DOM/style/ARIA、exact count、主要4操作、source hashを固定。Grok ACCEPT P0/P1=0 |
| CU-0A07B | `DONE` | [#355](https://github.com/oshikaidesu/Motolii/pull/355)でR4B mock-side同形React化、一方向legacy adapter、containment負例を完了。Grok ACCEPT P0/P1/P2=0、CI 4/4。次粒`CU-0A07C`を`DO`へ上げる |
| CU-0A07C | `DONE` | [#357](https://github.com/oshikaidesu/Motolii/pull/357)でR4B InspectorのJSX/CSSをbyte同一でproduct ownerへ移し、mockを単一package importのconsumerへ反転。二重copy 0、legacy runtime import 0、source closure／公開境界／Host projectionの変更0。Grok ACCEPT P0=0 / P1=0 / P2=1（任意の重複負例提案は後続候補）、reference guard 109 + product guard 3、Playwright 71、docs／UI／fmt／clippy／workspace green、CI 4/4。次粒`CU-0A08I`を`READY-RECHECK`へ送る |
| CU-0A08I | `SPLIT` | [Inspector read-model分割決定](reviews/2026-07-26-cu-0a08i-inspector-read-model-split-decision.md)で、既決field不足、Host transport/selection不在、intentのU4a/U4c留保を確認。CU-0A08IS→CU-0A08IP→CU-0A08ITへ分割 |
| CU-0A08IS | `DONE` | [Inspector read-model inventory](reviews/2026-07-26-cu-0a08is-inspector-read-model-inventory.md)、`docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs`、`node --test docs/mocks-ui/guard-tests/inspector-read-model-inventory.test.mjs` |
| CU-0A08IP | `DONE` | `ui/motolii-web/src/read-model/inspectorReadModelDecoder.js`、`docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`、`node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs`（39）、`npm run test:reference-guard`（172） |
| CU-0A04 | `DONE` | [快適利用粒度化 W0表](reviews/2026-07-22-m3-comfortable-use-granulation.md)の`CU-0A04`行でR1 Browserのproduct ownerへの直接移管完了を確認 |
| CU-0A08B | `SPLIT` | CU-G09完了後のreadiness再確認で、CU-G09 §4/§6/§7だけで閉じるCU-0A08BP pure decoderと、U4a-2を要するCU-0A08BT Host projection/typed intent/JSX接続へ分割 |
| CU-G02 | `DONE` | 本変更でM3仕様[運用順](specs/M3-ui-integration.md)、[implementation-ledger](implementation-ledger.md)、[decision-index](decision-index.md)を同時更新し、次PRODUCT-ASSET粒を`U3a-1`へ固定した |
| CU-G09 | `DONE` | [CU-G09 Browser catalog projection契約決定](reviews/2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) |
| U3a-1 | `SPLIT` | [U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)で、公開API STOPとpub再export成果の自己矛盾を解消するため`U3a-1S`（docs decision）→`U3a-1I`（実装）へ分割 |
| U3a-1S | `DONE` | 本変更で[U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)を確定し、owner=`motolii-ui`内module、`motolii-timeline` crate=`REJECT`、visibility=pub再export+integration test（制約6点）を固定。`U3a-1I`のclosed orderは本文書を直接authorityにできる |
| U3a-1I | `DONE` | headless Timeline projectionを`motolii-ui::timeline_projection`として実装し、小さな決定的Document fixtureで歴史回収§6のread-only projection/layout/cull/hit-testを閉じた |
| D1m | `DONE` | [M2仕様 D1m](specs/M2-document-model.md)でproject-scoped sidecar identity、process間session lock、`ProjectSession`経由のproject/journal変更を完了 |
| D2 | `DONE` | [M2仕様 D2](specs/M2-document-model.md)でコマンドapply/revert、Undo/Redo、atomic gesture macroを[#109](https://github.com/oshikaidesu/Motolii/pull/109) / [#130](https://github.com/oshikaidesu/Motolii/pull/130)により完了 |
| U2b-1 | `DONE` | [M3仕様 U2b](specs/M3-ui-integration.md)と[次にIssue化するもの](#次にissue化するもの)で、prepared requestのsingle writer配送と成功snapshot publishを完了 |
| CU-G03D | `DONE` | [CU-G03 edit durability / publish順序決定](reviews/2026-07-26-cu-g03-edit-durability-ordering-decision.md)で、VS-1単一command actionのjournal→live Apply/Undo/Redo→revision→selection reconcile→1 publish、failure poison、複数command STOPを確定 |
| CU-G03R | `DONE` | [#369](https://github.com/oshikaidesu/Motolii/pull/369)でcatalog未反映committed Edit tailをMainFile fast pathから既存replayへ送り、stale-catalog負例、main原本不変、純checkpoint・二重checkpoint対照、Grok ACCEPT P0/P1=0、CI 4/4を完了 |
| CU-G03 | `DONE` | 子粒`CU-G03D`と`CU-G03R`が本表でともに`DONE` |
| CU-101 | `DONE` | [U2b-2 Place product core再採択 §3.2](reviews/2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#32-product-placeの閉じた意味)でtarget/start/duration/recipe/position/nameの閉じた値とappearance非目標を確認 |
| CU-102 | `DONE` | [fresh LayerId + AddTrackItem原子性決定](reviews/2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md)でclone候補、fresh live二条件、live mint 0、1 Command/1 macro、failure不変、既存journal互換を確定 |
| U0a | `DONE` | [主クリティカルパス](#主クリティカルパス) Selected U series行の`U0a DONE`、[M3仕様 運用順](specs/M3-ui-integration.md)、[M3入場判定](#m3への入場判定) |
| U0b-1 | `DONE` | [主クリティカルパス](#主クリティカルパス) Selected U series行の`U0b-1 DONE`、[M3仕様 運用順](specs/M3-ui-integration.md)、[M3入場判定](#m3への入場判定) |
| U0b-2 | `DONE` | [主クリティカルパス](#主クリティカルパス) Selected U series行の`U0b-2 DONE`、[M3仕様 運用順](specs/M3-ui-integration.md)、[M3入場判定](#m3への入場判定) |
| CU-G01 | `DONE` | [G0-9段階化](reviews/2026-07-23-m3-g0-9-staged-platform-gates.md)で固定Mac prerequisite evidenceをG0-9Lへ限定し、G0-9DをDistribution Readyまで分離 |
| GR-D2 | `DONE` | [監督ループ決定](reviews/2026-07-25-opus-spark-grok-supervision-loop-decision.md)へ変更許可閉集合、append-only証跡、timeout分離、検収者mutation拒否、resumeを継承済み |
| GR-D3 | `DONE` | [#336](https://github.com/oshikaidesu/Motolii/pull/336)で既知三entryのfail-closed清掃、HEAD／全ref不変、実K0停止形とGrok到達を閉包 |
| K0 | `DONE` | [#338](https://github.com/oshikaidesu/Motolii/pull/338)と[K0契約凍結報告](spikes/m4-k0-region-contract.md)で9条件を15 testに固定。**test-only契約凍結であり、runtime region関数・公開API・testkit昇格・ROI最適化は含まない**。K1系はこの行だけを根拠に起票しない |
| M1-FREEZE-GATE | `DONE` | [凍結ゲート宣言](reviews/2026-07-10-freeze-gate-declaration.md)がM2〜M5並列laneを解禁 |
| M2-D3 | `DONE` | [M2仕様 D3](specs/M2-document-model.md)のDocument→render graph接続が完了 |
| M2-FOUNDATION-RECLOSURE | `DONE` | [M2基盤再締結ゲート](reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み |
| VSM-A1 | `DONE` | Vism計画Phase Aのfirst-party公開境界監査完了 |
| VSM-A2 | `DONE` | Vism計画Phase AのParamDriver外部crate参照実装完了 |
| VSM-A3 | `DONE` | Vism計画Phase AのRadial Repeater実装・審判完了 |

## 次にIssue化するもの

前段PRがmainへ入った時点で、最新の型名・fixture・依存を確認してから起票する。

| 順序 | ID | Phase | 状態 | 起票条件 | 次の出口 |
|---|---|---|---|---|---|
| 1 | D1j | M2 | `DONE` | CAM-G0 merge（D1lはmain到達済み） | v5 planar camera schema/default migration |
| 2 | U2b-1 | M3 | `DONE` | U1b-2 merge | prepared requestをsingle writerへ配送し、成功snapshotをUI/render workerへ購読 |
| 3 | U3a-1S | M3 | `DONE` | 本変更で完了。後続docs粒を作らない | owner=`motolii-ui`内module、`motolii-timeline` crate=`REJECT`、visibility=pub再export+integration test（制約6点）を[U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)で確定済み |
| 3 | U3a-1I | M3 | `DONE` | `U3a-1S` `DONE`。論理依存の`U0a`/`U0b`は`DONE`。G0-6H/U0e-3/U2c-3/U2c-5/G0-9は入場条件にしない | toolkit/renderer非依存のDocument→Timeline projection/layout/cull/hit-testを小さなfixtureで閉じる。G0-9や100k再実測を入場条件にしない |
| 4 | U3a-2 | M3 | `WAIT` | U3a-1I + G0-9 platform受入 | direct wgpu+Vello候補をwindowed fixture、input、WebView同居、presentまで閉じる。Canvas/browser WebGPUは先例baselineで製品枝にしない |
| 5 | U2g | M3 | `WAIT` | D1l + D3e + U0e + U2b + U3a-2 merge | Effect常時接続線 |
| 6 | K1a | M4 | `READY-RECHECK` | K0凍結(test-only)。依存先行の責任最小化ゲートでK1aが必要とする責任を列挙し、K0成果を自動採用せず再判定した後に起票 | ResourceLedgerとhard budget。backendの空きVRAM値を正本にしない |
| 7 | K1b | M4 | `WAIT` | K1a merge | cache同一性/LRU/並行store |
| 8 | K1c | M4 | `WAIT` | K1a + K1b merge | VRAM/RAM/disk階層admissionと退避 |
| 9 | K1d | M4 | `WAIT` | K1c + K4 merge | 容量pressureとdeadlineを分離したpreview縮退signal |
| 10 | K7a | M4 | `WAIT` | K1b + K1c + D3 merge | group子合成のatomic bake成果物境界 |
| 11 | K7b | M4 | `WAIT` | K7a + K2 merge | 依存時間区間だけの無効化と旧世代再利用 |
| 12 | K7c | M4 | `WAIT` | K7a + K7b merge | bake hit時の内部graph置換と再freeze |
| 13 | K8a | M4 | `WAIT` | K1b + K1c + K1d + D3 merge | 全曲Draft coverage planner |
| 14 | K8b | M4 | `WAIT` | K7c + K8a + D5 merge | 100GB accounting fixtureと通し再生E2E |
| 14 | U0f | M3 | `WAIT` | G0-2 + G0-8 + U0b + K1a merge | resource policyをUser settingsへ。Documentへ入れない |
| 15 | U1g | M3 | `WAIT` | U1b + U1c + U5 + K1d merge | Transport時刻不変の最新frame表示/コマ落ち |
| 16 | U1h | M3 | `WAIT` | U0e + U0f + U1g merge | Performance/Memory settingsとpressure HUD |
| 17 | P7a | M5 | `WAIT` | P0I merge | Duplicator recipe schema |
| 18 | U9a | M3 | `WAIT` | U2b merge | 汎用one-shot Generator hook。script runtime型を公開契約へ焼かない |
| 19 | U9b | M3/v1.x | `WAIT` | U9a merge | Motolii ShapeScript。Paper.js互換やp5.js互換を名乗らない |
| 20 | U9c | M3/v1.x | `WAIT` | U9b merge | SVG materialize adapter。DOM/XMLをDocument意味へしない |
| 21 | SCR-4 | M4/v1.x | `WAIT` | U9b + F-11 + K0/K1b/K1c/K7 | 非clear drawをホスト所有Feedbackへ翻訳。隠しcanvasを作らない |

## 凍結済みだが依存待ちのIssue

| ID | 状態 | Issue | 待っているもの | 注意 |
|---|---|---|---|---|
| U2f | `BLOCKED` | [#168](https://github.com/oshikaidesu/Motolii/issues/168) | U0c、U0d、U2a、U2c | one-shotだけ。永続offset/Modifierへ広げない |
| U1f | `BLOCKED` | [#169](https://github.com/oshikaidesu/Motolii/issues/169) | U1b、U0e、D1k、D3 camera follow-up | K0は依存ではない。保守的Draftで成立させる。M2 camera実装済みとStage UI未実装を分離し、[Unit 4Q回収](reviews/2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md)のowner負例を維持 |

## 先に仕様を直すもの

| 対象 | 状態 | 問題 | 現場の行動 |
|---|---|---|---|
| [#51](https://github.com/oshikaidesu/Motolii/issues/51) | `DECIDE` / stale | Issue本文の`camera: Option<CompCamera>`・`None=DEFAULT`は、現行D1j/D1kの「全Compositionに常在」「Render入力必須」「DEFAULT直書き拒否」と不一致 | #51をそのまま実装しない。D1j schema → D1k runtime → D3接続の3PRへ再翻訳する |
| G0-2 | `DONE` | 入力/キーマップ/a11y最小意味論 | [M3着手前決定§2](reviews/2026-07-16-m3-preflight-decisions.md#2-g0-2-inputとui状態の意味)に従いU0bをIssue化 |
| G0-3 | `WAIT` / `比較中` | plugin UIモデル | `NodeDesc`自動panel fallbackを維持し、公開kit、sandbox、権限、互換、配布をG0-9製品surface合否と分離して再評価。G0-9証拠は入力にできるが、比較前に公開UI APIを実装しない |
| G0-4 | `DONE` | UI性能測定プロトコル | U1c/U3a等でraw結果を取り、絶対閾値は別改訂 |
| G0-6H | `DO` / `HUMAN` | 視覚token/認知審判 | U0e-2の5 reference screenと30画像を`docs/mocks-ui/reference-handoff.md`の未記入templateで目視し、具体tokenを固定してU0e-3へ |
| G0-7 | `DONE` | Direct/Tool/Advanced conformance | UI操作言語とU2c fixtureへ従う |
| G0-8 | `WAIT` / `MEASURE` | resource予算preset/安全余白/hysteresisの具体値 | G0-4手順+K1a実測後に値だけ固定。P3/P3aの意味は変更しない |

## M3への入場判定

U0a(egui骨格+依存方向CI)は本入場で完了。M2基盤再締結は解除済み。下表は論理上の直前条件を示すが、現在のUシリーズではSelected U seriesの直列順が追加の運用条件となる。#180/#191≠入場完了。

| 目的 | 必要な直前条件 |
|---|---|
| UI shellを始める | Selected U seriesのU2a-1までmain到達 + U1a固有依存 |
| Rerun sourceを読む・資産分類する | 入場前も可。commit/license/version、Motoliiへの転移条件、`DEPEND/VENDOR/PORT/PATTERN/REJECT`だけを文書化 |
| Rerun由来crate追加・vendoring・移植を始める | U0a入場 + [Rerun学習・転移計画](reviews/2026-07-20-rerun-learning-transfer-plan.md)の対象RRレーン反対側レビュー |
| 静止previewを出す | U0a + D3 + U1a |
| 枠外Stageを作る | U1b + U0e + D1k + D3 camera follow-up |
| Relative Moveを作る | U0c + U0d + U2a + U2c |
| Effect接続線を作る | D1l + D3e + U0e + U2b + U3a |
| 編集時Generator hookを作る | U2b。まずruntime非依存のD2 command batch境界だけを固定 |
| ShapeScriptを作る | U9a + D1i-2。正準座標・object/path/group・拒否表を先に固定 |
| SVG adapterを作る | U9b。viewport/Y-down変換と安全な採用subsetを先に固定 |
| 蓄積描画を作る | U9b + F-11 + K0/K1b/K1c/K7。畳めるshape履歴を先にmaterializeし、残りだけFeedbackへ昇格 |
| resource設定を出す | G0-2 + G0-8 + U0b + K1a → U0f。設定はUser settings、pressure実測値はTransient |
| 重いpreviewを追従させる | U1b + U1c + U5 + K1d → U1g。project fps/audio clockを変えず表示frameだけ落とす |

したがって現在の短い運用判断は、**CU-0A03 / R0からCU-0A07C / R4Cまでは完了済み。CU-0A08IP、`U3a-1S`/`U3a-1I`、`CU-G03D`/`CU-G03R`、親`CU-G03`、`CU-101`、`CU-102`、`CU-G09`は`DONE`。`CU-0A08B`はdecoderとHost接続へ分割し、次のPRODUCT-ASSET粒は`CU-0A08BP`（product-owned・非export pure Browser catalog decoder）を`DO`とする。`CU-0A08BT`は`WAIT`（CU-0A08BP + `U4a-2` Direct製品入口待ち）、`CU-0A08IT`は`WAIT`（`U4a-2` Direct製品入口待ち）。`U2c-2`は`WAIT`（U4a-2 Direct + U4c Advanced）。G0-6HはU0e-3だけを止める。CU-109 runtime配線とCU-104は未着手のまま束ねない。Host transport、typed intent、JSX binding、drag payload、`S`行の意味決定、Rust/schema/plugin変更をCU-0A08BPへ混ぜるならSTOPする。Motolii Studio Previewは未実装。**G0-6Hは同時に進められる人間審判だが、未完了でもR0〜R4やPreview骨格を止めず、U0e-3だけを止める。G0-9DはDistribution Readyまで`WAIT / HARDWARE`。`U2c-2`はVS-2候補かつ実製品入口待ちである。D1n、D5等の独立follow-upをVS-1の再停止理由へしない。

## 更新規則

- Issue作成時: ID、Issue URL、依存、完了後の出口を追加する。
- PR merge時: 対象を`DONE`へ移すか行を削り、直接の後続を`ISSUE`または`DO`へ上げる。
- decision完了時: `DECIDE`を消し、実装タスクを`ISSUE`へ上げる。
- 依存や型名が変わった時: Issue本文と本ページを同じspec PRで更新する。
- 完了条件、型シグネチャ、意味論表は本ページへ複製しない。
- GitHubのcheckboxが古い場合はmain/PRを確認し、本ページだけでなくIssue本文も同期する。

## 詳細への入口

- 全マイルストーン仕様: [specs/README.md](specs/README.md)
- M2: [M2-document-model.md](specs/M2-document-model.md)
- M3: [M3-ui-integration.md](specs/M3-ui-integration.md)
- M4: [M4-cache-and-analysis.md](specs/M4-cache-and-analysis.md)
- M5: [M5-3d-and-post.md](specs/M5-3d-and-post.md)
- 横断バックログ: [backlog.md](backlog.md)
- Recent motion readiness: [2026-07-15-implementation-readiness-ledger.md](reviews/2026-07-15-implementation-readiness-ledger.md)
