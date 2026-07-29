# 実装進行台帳

最終確認: **2026-07-28**

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
| M3 | **VS-1 Rectangle配置とUndo** | React chrome + native Stage/Timeline + headless interaction、1 top-level wgpu Surface + 2 native viewport + opaque child WebView islandsを正本化。R4CまでのInspector product ownership、CU-0A08IP、`CU-G09`／`CU-G09O`／docs-only `CU-G09R`は完了した。`CU-0A08BP`、docs-only `CU-104`/`U2h-1S`/`CU-104E`、実装粒`U2h-1I`、docs-only `U2h-1PR`/`CU-105R`/`CU-106S`/`U3a-2S0`、docs-only `U3a-2S`、docs-only `U3a-2R`、docs-only `U3a-2Z`、docs-only `U3a-2A`、docs-only `U3a-2P`、docs-only `U3a-2Q`、docs-only `U3a-2Q-P`、docs-only `U3a-2Q-P2`、docs-only `U3a-2Q-P3`、docs-only `U3a-2Q-P4`、docs-only `CU-109S0`、docs-only `CU-109S`、docs-only `CU-109SP`、docs-only `CU-G04S0`、docs-only `CU-G04S`、docs-only `CU-G04SC0`、docs-only `CU-G04SC`、実装粒`CU-109`、docs-only `CU-110S`、docs-only `CU-110D`、docs-only `CU-107S`、docs-only `CU-107D`、docs-only `CU-107R`、docs-only `CU-107N`、docs-only `CU-107W`は`DONE`。U2h-1P単独producer粒は停止し、CU-105とCU-106を責任分割した。`U2h-1P`/`CU-106P`/`CU-106F`は実consumer surfaceまで`WAIT`。docs-only `CU-0A08RS0`/`CU-0A08RS`/`CU-0A08RM0`/`CU-0A08RMD`/`CU-0A08BD0`/`CU-0A08BDD`/`CU-0A08SS0`/`CU-0A08SSD`/`CU-0A08SSC`/`CU-0A08SSCD`/`CU-0A08SSCS`/`CU-0A08SSCSD`は`DONE`。`CU-0A08RM`はOpus `ORDER: STOP`により`WAIT`。実装粒 `CU-0A08SSCI` は当初Opus 5 order判定 `STOP` だったが、`CU-0A08BTR`でprivate seam責任をBTPへ吸収して`SPLIT`。docs-only `CU-0A08SSCI-P` は`DONE`。ORACLE-GUARD `CU-0A08SSCI-P1` は`DONE`。`(P)` は authority と guard 実装の両面で閉じた。ORACLE-GUARD `CU-0A08SSCI-T1`は`DONE`。`(T)`はauthorityとguard実装の両面で閉じた。`CU-0A08BTR`は`DONE`、親`CU-0A08BT`は`SPLIT`。`CU-0A08BTP`は`DONE`。`CU-0A08ITP-P`は`DONE`、次の唯一のPRODUCT-ASSET `DO`は`CU-0A08ITP`。`CU-0A08SSCI`はBTPへ実装責任を吸収して`SPLIT`。`CU-0A08BTP`以外の未完了PRODUCT-ASSET lane行は`WAIT`。`U3a-2Q-V`は`WAIT`。`CU-0A08BT`は`SPLIT`。Motolii Studio Previewは未実装。visible summary chromeは未決で実装しない。G0-6HはU0e-3を止める並行人間審判、G0-9DはDistribution Ready用hardware gate。eguiへ新規製品面を実装せず、plugin UI公開契約はG0-3 / GAP-13まで停止する |
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
| PRODUCT-ASSET | CU-G09O | M3 / VS-1 / SPEC / Browser decoder output contract | `DONE` | — | `CU-G09`は[発注依存証跡](#発注依存証跡)で`DONE`。CU-0A08BP order事前審査で出力shape、型gate、B11/B12写像のauthority不足を確認 | [CU-G09O Browser decoder output契約決定](reviews/2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md)。validated snapshot deep clone、label非分岐、B13/B14を閉じる。code/fixture/public API/`S`意味変更0 |
| PRODUCT-ASSET | CU-G09R | M3 / VS-1 / SPEC / Browser rejection precedence | `DONE` | [2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md](reviews/2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md) | `CU-G09`／`CU-G09O`は[発注依存証跡](#発注依存証跡)で`DONE` | [CU-G09R Browser decoder拒否優先順決定](reviews/2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md)。拒否family優先順・`B15`新設・`B8`予約・残余ID/ref境界を閉じた。code/fixture/public API/`S`意味変更0 |
| PRODUCT-ASSET | CU-0A08BP | M3 / VS-1 / B / Browser catalog decoder | `DONE` | — | `node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`（115 pass）、`npm run test:reference-guard` | product-owned・非export pure decoder、fixture/testのみ。typed intent、Host transport、JSX binding、drag payload、`S`行はCU-0A08BT以降 |
| PRODUCT-ASSET | CU-104 | M3 / VS-1 / SPEC / selection publish envelope decision | `DONE` | — | `CU-G02`／`CU-G03`は[発注依存証跡](#発注依存証跡)で`DONE`。既存U2h-1再採択を越えず、owner・visibility・`projection_generation`更新条件・Apply/Undo/Redo後publish前reconcile時点だけをdocsで閉じる。完了証拠は[CU-104 selection publish envelope決定](reviews/2026-07-27-cu-104-selection-publish-envelope-decision.md) | U2h-1実装は別粒。CU-109/CU-111/CU-110、consumer接続、additive/range/marquee/AX、Document/journal/Undoへのselection保存は含めない |
| PRODUCT-ASSET | U2h-1S | M3 / VS-1 / SPEC / primary selection implementation split | `DONE` | — | `CU-104`／`U2c-1`／`U2c-4`は[発注依存証跡](#発注依存証跡)で`DONE`。[U2h-1 primary selection implementation split決定](reviews/2026-07-27-u2h-1-primary-selection-implementation-split-decision.md)で既存Apply/Undo/Redo publish経路とselection-only入力面を分離 | `U2h-1I`を次実装粒へ選定。selection-only actionは`U2h-1P`、Place receiptはCU-110へ分離。Rust/JS/test/fixture、CU-109/110/111/106、公開API・Document・serde・journal・Undo・plugin契約は変更しない |
| PRODUCT-ASSET | CU-104E | M3 / VS-1 / SPEC / projection generation exhaustion | `DONE` | — | `CU-104`／`U2h-1S`は[発注依存証跡](#発注依存証跡)で`DONE`。[CU-104E projection generation枯渇境界決定](reviews/2026-07-27-cu-104e-projection-generation-exhaustion-decision.md)で枯渇preflight・queue消費・自動retry禁止・CU-109 poison非先取りを閉じた | `U2h-1I`を次実装粒へ戻す。Rust/test/public API/Document/serde/journal/plugin変更はしない |
| PRODUCT-ASSET | U2h-1I | M3 / VS-1 / B / primary selection publish envelope | `DONE` | — | `U2h-1S`は[発注依存証跡](#発注依存証跡)で`DONE`。意味正本は[U2h-1 split決定](reviews/2026-07-27-u2h-1-primary-selection-implementation-split-decision.md)と[CU-104](reviews/2026-07-27-cu-104-selection-publish-envelope-decision.md)。`U2h-1I`証拠: `crates/motolii-ui/src/document_edit_runtime.rs` + `crates/motolii-ui/src/app.rs` + 新規テスト。実行: `cargo fmt --all`、`cargo fmt --all --check`、`cargo clippy -p motolii-ui --all-targets -- -D warnings`、`cargo test -p motolii-ui`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace`、`./scripts/check-docs.sh` |
| PRODUCT-ASSET | U2h-1PR | M3 / VS-1 / SPEC / selection input reachability | `DONE` | — | U2h-1P事前審査とRust lint probeで、production caller 0かつ未使用`pub(crate)` method / variantが`-D warnings`で拒否されることを確認。[selection入力到達性決定](reviews/2026-07-27-u2h-1p-selection-input-reachability-decision.md)を正とする | 単独producer粒を停止し、存在拒否→same-id no-op→枯渇preflight順を固定。Rust/public API/CU-105/106実装は変更しない |
| PRODUCT-ASSET | U2h-1P | M3 / VS-1 / B / selection-only primary input acceptance | `WAIT` | — | `U2h-1I` `DONE` + `CU-106S` `DONE` + CU-106P | P5受入IDとしてCU-106Pへ統合。producerと実在する最小callerを同じ差分で成立させ、lint抑制・dummy caller・公開intent追加を行わない |
| PRODUCT-ASSET | CU-105R | M3 / VS-1 / SPEC / dense Timeline responsibility recheck | `DONE` | — | `U3a-1I`は[発注依存証跡](#発注依存証跡)で`DONE`。[CU-105責任再確認](reviews/2026-07-27-cu-105-dense-timeline-responsibility-recheck.md)を正とする | layout/hit-test `PASS`、capacity `REDUCE`、semantic zoomとselection/playhead/range `STOP`。code/fixture/bench/threshold変更0 |
| PRODUCT-ASSET | CU-105 | M3 / VS-1 / B / dense Timeline parent | `SPLIT` | — | `CU-105R` `DONE` | U3a-1I、既存capacity evidence、U3a-2、CU-106-familyへ配送済み。親名でclosed orderを作らない |
| PRODUCT-ASSET | CU-106S | M3 / VS-1 / SPEC / selection consumer split recheck | `DONE` | — | [CU-106 selection consumer分割決定](reviews/2026-07-27-cu-106-selection-consumer-split-decision.md)。production Timeline caller 0、通常起動のDocumentEditQueue caller 0、pointer入力0を確認 | CU-106P/Fへ分離し両方`WAIT`。Rust/fixture/public API変更0 |
| PRODUCT-ASSET | CU-106 | M3 / VS-1 / B / selection and focus parent | `SPLIT` | — | `CU-106S` `DONE` | CU-106P/Fへ配送済み。親名でclosed orderを作らない |
| PRODUCT-ASSET | CU-106P | M3 / VS-1 / B / primary selection consumer | `WAIT` | — | U3a-2入場範囲決定 + non-test Timeline caller + production pointer入力 | U2h-1P P5を内包。存在拒否→no-op→枯渇preflightとproduction到達性を固定 |
| PRODUCT-ASSET | CU-106F | M3 / VS-1 / B / essential focus | `WAIT` | — | 実consumer surface + U3a-2 / Host focus owner | primary/hoverと分離。三surface、hidden件数、additive/range/marquee/AXを束ねない |
| PRODUCT-ASSET | U3a-2S0 | M3 / VS-1 / SPEC / windowed Timeline dependency evidence closure | `DONE` | — | 発注依存証跡に`G0-9L`／`U2h-1PR`／`CU-105R`／`CU-106S`の一意な`DONE`行を追加 | spec/code/fixture/renderer意味変更0。`U3a-2S`を`DO`へ上げる |
| PRODUCT-ASSET | U3a-2S | M3 / VS-1 / SPEC / windowed Timeline readiness split | `DONE` | [2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md](reviews/2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) | `U3a-1I`／`G0-9L`／`U2h-1PR`／`CU-105R`／`CU-106S`は[発注依存証跡](#発注依存証跡)で`DONE` | G0-9依存を(A)〜(D)に分割し、次判断をdocs-only `U3a-2R`へ送った。input/public API・製品window・renderer勝者は先取りしない |
| PRODUCT-ASSET | U3a-2S-R2 | M3 / VS-1 / SPEC / U3a-2S stale mirror synchronization | `DONE` | — | `U3a-2S`は[発注依存証跡](#発注依存証跡)で`DONE`、Grok R1 `REJECT` P1=1 | ledger/README/縦slice決定のcurrent mirrorだけを`U3a-2S DONE`／`U3a-2R DO`へ同期。意味・順序・code変更0 |
| PRODUCT-ASSET | U3a-2S-R3 | M3 / VS-1 / SPEC / U3a-2S handoff mirror synchronization | `DONE` | — | `U3a-2S-R2`は[発注依存証跡](#発注依存証跡)で`DONE`、R2全文負例の残存2件 | CU-106S/U2h-1PRの現行handoffだけを`U3a-2S DONE`／次判断`U3a-2R DO`へ同期。意味・順序・code変更0 |
| PRODUCT-ASSET | U3a-2R | M3 / VS-1 / SPEC / windowed Timeline renderer adoption scope | `DONE` | [2026-07-27-u3a-2r-renderer-adoption-scope-decision.md](reviews/2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) | `U3a-2S`／`U3a-2S-R2`／`U3a-2S-R3`は[発注依存証跡](#発注依存証跡)で`DONE`。着手時にCU-0G02B raw `DONE / FROZEN`、絶対閾値未追加を再照合 | 区分(D)のrenderer採択**範囲**をdocsで閉じた。勝者・egui削除・閾値・Rust/UI変更0 |
| PRODUCT-ASSET | U3a-2Z | M3 / VS-1 / SPEC / semantic zoom responsibility | `DONE` | [2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md](reviews/2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) | `U3a-2R`は[発注依存証跡](#発注依存証跡)で`DONE` | semantic zoom段階の**責任所在**をdocsで閉じた。段階の中身・renderer勝者・Rust/UI変更0 |
| PRODUCT-ASSET | U3a-2A | M3 / VS-1 / SPEC / windowed Timeline renderer adoption decision | `DONE` | [2026-07-27-u3a-2a-renderer-adoption-decision.md](reviews/2026-07-27-u3a-2a-renderer-adoption-decision.md) | `U3a-2Z`は[発注依存証跡](#発注依存証跡)で`DONE`。U3a-2R §7 の 4 条件を BASE_SHA 事実で再照合 | confirmation型で`direct_vello`採択。性能勝者判定・egui削除・絶対閾値・Rust/UI変更0 |
| PRODUCT-ASSET | U3a-2P | M3 / VS-1 / SPEC / playhead visible range owner decision scope | `DONE` | [2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md](reviews/2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) | `U3a-2A`は[発注依存証跡](#発注依存証跡)で`DONE` | playhead / visible range owner docs 範囲粒。owner自体は本粒で決めない。Rust/UI変更0 |
| PRODUCT-ASSET | U3a-2Q | M3 / VS-1 / SPEC / playhead visible range owner adoption split decision | `DONE` | [2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md](reviews/2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) | `U3a-2P`は[発注依存証跡](#発注依存証跡)で`DONE` | §6 証拠 coverage 非対称により owner 判断を `U3a-2Q-P` / `U3a-2Q-V` へ分割。owner 自体は本粒で決めない。Rust/UI 変更 0 |
| PRODUCT-ASSET | U3a-2Q-P | M3 / VS-1 / SPEC / playhead owner admissibility evidence supplement | `DONE` | [2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md](reviews/2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md) | `U3a-2Q`は[発注依存証跡](#発注依存証跡)で`DONE` | E1〜E4 admissibility 補遺・T1 で owner 一意導出不可を記録。owner 未決維持。Rust/UI 変更 0 |
| PRODUCT-ASSET | U3a-2Q-P2 | M3 / VS-1 / SPEC / playhead reopen lifetime decision | `DONE` | [2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md](reviews/2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) | `U3a-2Q-P`は[発注依存証跡](#発注依存証跡)で`DONE` | fresh Host coordinator の project 再 open では以前の playhead を復元せず、安全な初期位置へ戻す。具体値・五層 owner・state shape / serialization・製品 surface は決めない |
| PRODUCT-ASSET | U3a-2Q-P3 | M3 / VS-1 / SPEC / playhead future reopen restore posture | `DONE` | [2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md](reviews/2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md) | `U3a-2Q-P2`は[発注依存証跡](#発注依存証跡)で`DONE`。owner採択orderはOpus 5 `ORDER: STOP` | 将来のbest-effort reopen復元を延期・追加可能とし恒久棄却しない。現行no-restore維持。owner・shape / serialization・製品surfaceは決めない |
| PRODUCT-ASSET | U3a-2Q-P4 | M3 / VS-1 / SPEC / playhead five-layer owner adoption | `DONE` | [2026-07-27-u3a-2q-p4-playhead-five-layer-owner-adoption-decision.md](reviews/2026-07-27-u3a-2q-p4-playhead-five-layer-owner-adoption-decision.md) | `U3a-2Q-P3`は[発注依存証跡](#発注依存証跡)で`DONE` | `T2`で`Project session`をplayhead state ownerとして採択。具体値・shape / serialization・製品surfaceは未決 |
| PRODUCT-ASSET | CU-109S0 | M3 / VS-1 / SPEC / CU-109 readiness recheck selection | `DONE` | [CU-109S0選定](reviews/2026-07-27-cu-109s0-readiness-recheck-selection-decision.md) | `U3a-2Q-P4`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-109`実装を起動せず、prepared-action順序の再確認だけを`CU-109S`へ分離 |
| PRODUCT-ASSET | CU-109S | M3 / VS-1 / SPEC / Undo Redo prepared-action order recheck | `DONE` | [CU-109S順序再確認](reviews/2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) | `CU-G03D`／`CU-G03R`／`D1m`／`D2`／`U2b-1`／`CU-104`／`CU-104E`は[発注依存証跡](#発注依存証跡)で`DONE` | R2・候補(b)。次PRODUCT-ASSET `DO`はdocs-only `CU-109SP`。`CU-109`は`WAIT`維持 |
| PRODUCT-ASSET | CU-109SP | M3 / VS-1 / SPEC / CU-111 prepared-action order prerequisite | `DONE` | [CU-109SP prerequisite決定](reviews/2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) | `CU-109S`は[発注依存証跡](#発注依存証跡)で`DONE` | P1 precedence。`CU-111`順序前提をdocsで閉じた。typed shape/API/配送は決めない |
| PRODUCT-ASSET | CU-109SP-R1 | M3 / VS-1 / SPEC / CU-109SP stale rolling-mirror repair | `DONE` | — | `CU-109SP`／`CU-109S`は[発注依存証跡](#発注依存証跡)で`DONE` | decision-index rolling current行と縦slice決定のselection行のmirrorを同期し、次PRODUCT-ASSET `DO`を`CU-109`へ戻した。意味・順序・code変更0 |
| PRODUCT-ASSET | CU-G04S0 | M3 / VS-1 / SPEC / edit runtime session source selection | `DONE` | [CU-G04S0選定](reviews/2026-07-27-cu-g04s0-session-source-selection-decision.md) | `CU-G03D`／`CU-G03R`／`CU-109S`／`CU-109SP`／`CU-109SP-R1`／`D1m`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-109`実装を起動せず、session source判断だけを`CU-G04S`へ分離。4問の結論・code変更0 |
| PRODUCT-ASSET | CU-G04S | M3 / VS-1 / SPEC / edit runtime session source | `DONE` | [CU-G04S session source決定](reviews/2026-07-27-cu-g04s-edit-runtime-session-source-decision.md) | `CU-G03D`／`CU-G03R`／`CU-109S`／`CU-109SP`／`CU-109SP-R1`／`D1m`は[発注依存証跡](#発注依存証跡)で`DONE` | D1〜D7でsession source・no-session・CU-111前typed rejection・U2B1 smoke再係留事前承認をdocsで閉じた。CU-G04親は`DECIDE`維持 |
| PRODUCT-ASSET | CU-G04SC0 | M3 / VS-1 / SPEC / edit runtime product path handoff selection | `DONE` | [CU-G04SC0選定](reviews/2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md) | `CU-G04S`／`CU-G03D`／`CU-G03R`／`CU-109SP`／`CU-109SP-R1`／`D1m`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-109`実装を起動せず、binary→session-backed entryのproduct path handoff判断だけを`CU-G04SC`へ分離。4問の結論・code変更0 |
| PRODUCT-ASSET | CU-G04SC | M3 / VS-1 / SPEC / edit runtime product path handoff | `DONE` | [CU-G04SC product path handoff決定](reviews/2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md) | `CU-G04SC0`は[発注依存証跡](#発注依存証跡)で`DONE` | carrier・entry境界・failure処分をdocsで閉じた。次PRODUCT-ASSET `DO`は`CU-109` |
| PRODUCT-ASSET | CU-109 | M3 / VS-1 / CORE / edit durability runtime wiring | `DONE` | [#425](https://github.com/oshikaidesu/Motolii/pull/425) | `CU-G04SC`は[発注依存証跡](#発注依存証跡)で`DONE`。実装commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304` | P1。acceptance evidenceはApply roundtripに限定。明示project path・no-sessionではedit runtime構築しない。Undo/Redo durable/poison/journal/reconcile/publish配線はCU-109所有 |
| PRODUCT-ASSET | CU-110S | M3 / VS-1 / SPEC / CU-110 prerequisite scope selection | `DONE` | [CU-110S選定](reviews/2026-07-28-cu-110s-dependency-scope-decision-selection.md) | `CU-109`／`CU-G04SC`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-110D`へhandoff済み。答え・`CU-107`/`CU-110`/`CU-111`実装は含めない |
| PRODUCT-ASSET | CU-110D | M3 / VS-1 / SPEC / CU-110 CU-107 dependency scope decision | `DONE` | [CU-110D決定](reviews/2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) | `CU-109`／`CU-G04SC`／`CU-110S`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-110`の`CU-107`依存を全体待ちのままにするか、より狭い名前付き前提へ分割するかを一問だけ裁定する。`CU-107`/`CU-110`/`CU-111`実装、child grain命名、event shape、APIは含めない |
| PRODUCT-ASSET | CU-107S | M3 / VS-1 / SPEC / CU-107 split concretization selection | `DONE` | [CU-107S選定](reviews/2026-07-28-cu-107s-split-concretization-scope-selection.md) | `CU-109`／`CU-G04SC`／`CU-110S`／`CU-110D`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-107D`へhandoff済み。答え・`CU-107`/`CU-110`/`CU-111`実装・`WAIT`解除は含めない |
| PRODUCT-ASSET | CU-107D | M3 / VS-1 / SPEC / CU-107 split concretization scope decision | `DONE` | [CU-107D決定](reviews/2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md) | `CU-109`／`CU-G04SC`／`CU-110S`／`CU-110D`／`CU-107S`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(B)を採択し、`CU-110`が必要とする責任範囲の限定を先に閉じる順序を裁定。子粒名・個数・event shape・API・W0表書換え・`WAIT`解除は含めない |
| PRODUCT-ASSET | CU-107R | M3 / VS-1 / SPEC / CU-110-required CU-107 responsibility scope | `DONE` | [CU-107R決定](reviews/2026-07-28-cu-107r-cu-110-required-responsibility-decision.md) | `CU-109`／`CU-G04SC`／`CU-110S`／`CU-110D`／`CU-107S`／`CU-107D`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-110`が必要とする`CU-107`責任範囲だけを限定する。子粒名・個数・event shape・API・W0表書換え・`WAIT`解除は含めない |
| PRODUCT-ASSET | CU-107N | M3 / VS-1 / SPEC / CU-107 narrow prerequisite closed set | `DONE` | [CU-107N決定](reviews/2026-07-28-cu-107n-cu-107-narrow-prerequisite-closed-set.md) | `CU-107R`は[発注依存証跡](#発注依存証跡)で`DONE` | 7 load-bearing clause を4前提の閉集合へ分割し、単一 owner 割当と依存順を確定。次PRODUCT-ASSET `DO`はdocs-only `CU-107W` |
| PRODUCT-ASSET | CU-107W | M3 / VS-1 / SPEC / CU-107 W0 mirror rewrite | `DONE` | [CU-107W決定](reviews/2026-07-28-cu-107w-w0-mirror-rewrite-decision.md) | `CU-107N`は[発注依存証跡](#発注依存証跡)で`DONE` | W0 表と `CU-110` 依存リストを本閉集合の名前へ書き換える裁定（docs-only） |
| PRODUCT-ASSET | CU-107W-R1 | M3 / VS-1 / SPEC / CU-107W review mirror repair | `DONE` | — | `CU-107W`／`CU-107N`／`CU-107R`は[発注依存証跡](#発注依存証跡)で`DONE`、Grok検収 `REJECT` P0=0 / P1=1、P2非blocking助言1件 | decision-indexの`CU-107W DONE`明記とCU-107W決定W-2/W-7の承認発注文面復元だけを行った。意味・順序・code変更0 |
| PRODUCT-ASSET | CU-0A08RS0 | M3 / VS-1 / SPEC / Browser and Inspector read-projection dependency scope selection | `DONE` | [CU-0A08RS0選定](reviews/2026-07-29-cu-0a08rs0-browser-inspector-read-projection-dependency-scope-selection.md) | `CU-107W`／`CU-107W-R1`／`CU-0A08BP`／`CU-0A08IP`／`CU-0A08IS`／`CU-G09`／`CU-G09O`／`CU-G09R`／`CU-101`／`CU-102`は[発注依存証跡](#発注依存証跡)で`DONE` | U4a-2 Direct製品入口がVS-1のBrowser/Inspector read-only projectionにもload-bearingかだけをdocs-only `CU-0A08RS`へ選定する。答え、WAIT解除、API、event shape、code変更0 |
| PRODUCT-ASSET | CU-0A08RS | M3 / VS-1 / SPEC / Browser and Inspector read-only projection U4a-2 dependency adjudication | `DONE` | [CU-0A08RS裁定](reviews/2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md) | `CU-0A08RS0`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(B)を採択し、VS-1 read-only projectionに`U4a-2`はload-bearingでないと裁定。既存依存クラスを限定し、`CU-0A08BT`/`CU-0A08IT`の`WAIT`・依存セルは不変 |
| PRODUCT-ASSET | CU-0A08RM | M3 / VS-1 / SPEC / Browser and Inspector dependency cell mirror | `WAIT` | Opus `ORDER: STOP`（Browser typed-intent依存のauthority未裁定） | `CU-0A08RMD`は[発注依存証跡](#発注依存証跡)で`DONE`、`CU-0A08BD0`以降の依存方向裁定待ち | [CU-0A08RMD裁定](reviews/2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md)のRectangle Place分類を維持。依存の向きと実装順を裁定するまでBT/IT依存セルを書き換えない |
| PRODUCT-ASSET | CU-0A08RM0 | M3 / VS-1 / SPEC / Browser typed-intent dependency adjudication scope selection | `DONE` | [CU-0A08RM0選定](reviews/2026-07-29-cu-0a08rm0-browser-typed-intent-dependency-adjudication-scope-selection.md) | `CU-0A08RS`／`CU-0A08RS0`は[発注依存証跡](#発注依存証跡)で`DONE`、`CU-0A08RM` Opus `ORDER: STOP`を再現 | Browser typed-intentが既存Rectangle Place chainか別Direct責任かだけをdocs-only `CU-0A08RMD`へ一問選定。答え、BT/ITの`WAIT`・依存セル、API、event shape、code変更0 |
| PRODUCT-ASSET | CU-0A08RMD | M3 / VS-1 / SPEC / Browser typed-intent dependency adjudication | `DONE` | [CU-0A08RMD裁定](reviews/2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md) | `CU-0A08RM0`／`CU-0A08RS`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(A)をVS-1 Rectangleに限定採択。BT/ITの`WAIT`・依存セル・W0表・`U4a-2`は不変。依存の向きと実装順はdocs-only `CU-0A08BD0`へ送る |
| PRODUCT-ASSET | CU-0A08BD0 | M3 / VS-1 / SPEC / Browser typed-intent dependency direction scope selection | `DONE` | [CU-0A08BD0選定](reviews/2026-07-29-cu-0a08bd0-browser-typed-intent-dependency-direction-scope-selection.md) | `CU-0A08RMD`／`CU-0A08RM0`／`CU-0A08RS`は[発注依存証跡](#発注依存証跡)で`DONE` | `CU-0A08BT`とPlace連鎖の依存の向き・実装順を決める一問だけを次のdocs-only粒へ選定する。答え、BT/ITの`WAIT`・依存セル、`CU-110`前提、API、event shape、code変更0 |
| PRODUCT-ASSET | CU-0A08BDD | M3 / VS-1 / SPEC / Browser typed-intent dependency direction adjudication | `DONE` | [CU-0A08BDD裁定](reviews/2026-07-29-cu-0a08bdd-browser-typed-intent-dependency-direction-decision.md) | `CU-0A08BD0`／`CU-0A08RMD`／`CU-0A08RS`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(A) Browser source-seam firstをVS-1 Rectangleに限定採択。契約具体・BT/IT/RM行・Place ownerは不変 |
| PRODUCT-ASSET | CU-0A08SS0 | M3 / VS-1 / SPEC / Browser Place source seam implementation-boundary scope selection | `DONE` | [CU-0A08SS0選定](reviews/2026-07-29-cu-0a08ss0-browser-place-source-seam-implementation-boundary-scope-selection.md) | `CU-0A08BDD`／`CU-0A08BD0`／`CU-0A08RMD`は[発注依存証跡](#発注依存証跡)で`DONE` | Browser source seamの最小実装境界を特定する一問だけを選定する。型名、event、payload、API、module path、code変更は0 |
| PRODUCT-ASSET | CU-0A08SSD | M3 / VS-1 / SPEC / Browser Place source seam implementation-boundary adjudication | `DONE` | [CU-0A08SSD裁定](reviews/2026-07-29-cu-0a08ssd-browser-place-source-seam-implementation-boundary-decision.md) | `CU-0A08SS0`／`CU-0A08BDD`／`CU-0A08BD0`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(A) product-owned React Browser source seamをVS-1 Rectangleに限定採択。契約具体・BT/IT/RM行・Place ownerは不変 |
| PRODUCT-ASSET | CU-0A08SSC | M3 / VS-1 / SPEC / Browser Place source seam contract concretization scope selection | `DONE` | [CU-0A08SSC選定](reviews/2026-07-29-cu-0a08ssc-browser-place-source-seam-contract-concretization-scope-selection.md) | `CU-0A08SSD`／`CU-0A08SS0`／`CU-0A08BDD`は[発注依存証跡](#発注依存証跡)で`DONE` | scoped identityの受け渡し責任を置く既存component境界の一問だけを選定。答え、契約名、code変更は0 |
| PRODUCT-ASSET | CU-0A08SSCD | M3 / VS-1 / SPEC / Browser Place source seam contract concretization adjudication | `DONE` | [CU-0A08SSCD裁定](reviews/2026-07-29-cu-0a08sscd-browser-place-source-seam-contract-concretization-decision.md) | `CU-0A08SSC`／`CU-0A08SSD`／`CU-0A08SS0`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(B) CandidateCreateBrowser境界をVS-1 Rectangleに限定採択。契約名、code変更、Place ownerは不変 |
| PRODUCT-ASSET | CU-0A08SSCS | M3 / VS-1 / SPEC / Browser Place source seam implementation scope selection | `DONE` | [CU-0A08SSCS選定](reviews/2026-07-29-cu-0a08sscs-browser-place-source-seam-implementation-scope-selection.md) | `CU-0A08SSCD`／`CU-0A08SSC`／`CU-0A08SSD`は[発注依存証跡](#発注依存証跡)で`DONE` | 最小closed implementation orderの範囲を一問へ限定。答え、契約名、code変更は0 |
| PRODUCT-ASSET | CU-0A08SSCSD | M3 / VS-1 / SPEC / Browser Place source seam implementation scope adjudication | `DONE` | [CU-0A08SSCSD裁定](reviews/2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md) | `CU-0A08SSCS`／`CU-0A08SSCD`／`CU-0A08SSC`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(A) 内部source seamのみをVS-1 Rectangleに限定採択。契約名、code変更、Place ownerは不変 |
| PRODUCT-ASSET | CU-0A08SSCI | M3 / VS-1 / B / Browser Place internal source seam | `SPLIT` | [CU-0A08BTR依存再締結](reviews/2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md) | `CU-0A08SSCI-P1`／`CU-0A08SSCI-I`／`CU-0A08SSCI-T1`は[発注依存証跡](#発注依存証跡)で`DONE` | private seam実装責任を`CU-0A08BTP`のcomponent入力と同じclosed diffへ吸収。Host transport、typed intent、D2、drop終端は非目標 |
| PRODUCT-ASSET | U3a-2Q-V | M3 / VS-1 / SPEC / visible range owner adoption | `WAIT` | — | actual consumer surface evidence 待ち | visible range owner。production pointer 入力・`TimelineHit` production caller・製品 window 結合の実成立待ち |
| PRODUCT-ASSET | CU-0A08BT | M3 / VS-1 / B / Direct Browser connection | `SPLIT` | [CU-0A08BTR依存再締結](reviews/2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md) | CU-0A08BTP → CU-0A08BTI | read-only projection/JSX connectionとtyped intentを分離。BTPはU4a-2非依存、BTIは既決Place chain待ち |
| PRODUCT-ASSET | CU-0A08ITP-P | M3 / VS-1 / SPEC / Inspector post-promotion authority amendment | `DONE` | [CU-0A08ITP-P改訂](reviews/2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md) | `CU-0A08IP`／`CU-0A08RS`は[発注依存証跡](#発注依存証跡)で`DONE` | K-1 Inspector component固定byteとprovenance v1単一配列を専用chainへ狭く改訂。次の唯一のPRODUCT-ASSET `DO`は`CU-0A08ITP` |
| PRODUCT-ASSET | CU-0A08ITP | M3 / VS-1 / B / Inspector read-only projection and JSX connection | `DO` | [CU-0A08ITP-P改訂](reviews/2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md) | CU-0A08ITP-P + CU-0A08IP + CU-0A08RS | decoded target既決3 fieldの既存installed identity JSX接続だけ。runtime producer/Host wire/intent/他branch/`S`行変更0 |
| PRODUCT-ASSET | CU-0A08IT | M3 / VS-1 / B / Direct Inspector connection | `WAIT` | — | CU-0A08ITP完了後に分割確定 | Host transport・typed intentはU4a-2 Direct製品入口待ち |
| ORACLE-GUARD | CU-0A08IQ | M3 prerequisite / Inspector stale-pattern oracle precision | `DONE` | — | pre-edit inventory testで、完了済みInspector inventoryのtokenから別taskのBrowser decoder current-state tokenまで同一行を横断するT7誤検知を再現。hash driftとは独立 | 4 path・5 patternを維持し、sentence/task境界を越えない精密化、真のstale拒否、別task current-state受理を独立ticketで固定する。検査対象file/line個別除外、仕様本文整形、hash更新、BP差分は含めない |
| ORACLE-GUARD | CU-0A08IR | M3 prerequisite / Inspector authority hash reclosure | `DONE` | — | `b0c0f916`でInspector分割決定の進行状態とDirect/U4c依存整理が正当に更新された一方、inventory guardの固定hashが旧blobのままであることを再現。CU-0A08IPの意味・実装は変更しない。pre-edit testで独立T7誤検知も判明したためCU-0A08IQ待ち | CU-0A08IQ完了後、authority本文を変更せずguardの固定hashだけを現行blobへ再締結して専用testと全reference guardを緑へ戻す。CU-0A08BPとは別ticket・別commit |
| ORACLE-GUARD | CU-0A08BQ | M3 prerequisite / Browser completion-transition oracle precision | `DONE` | — | 隔離BP v5でCU-0A08BPを`DONE`へ正当更新すると、T7 A1のlive selectorが別taskの状態語`DO`へ依存しているためreference guardが216中215となることを再現 | 4 path・5 patternを維持し、A1 selectorを状態非依存化してBP `DO`／`DONE` synthetic acceptを固定する。BP実装、仕様本文、authority hash、React／Rust変更は含めない |
| ORACLE-GUARD | CU-0A08BR | M3 prerequisite / Browser post-completion lane oracle precision | `DONE` | — | Browser decoderは[発注依存証跡](#発注依存証跡)で完了済み。次の正当な製品粒選定を隔離差分で追加すると、Browser decoder guardの完了後oracleが対象IDを限定せず全後続`DO`を拒否し、287中286 passを再現。`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`（117 pass） | 完了済みdecoder粒のstale状態拒否を維持しつつ、別IDの正当な後続`DO`をsynthetic受理するよう専用guardだけを精密化する。製品実装、仕様本文、台帳の次粒選定、期待値・threshold変更は含めない |
| ORACLE-GUARD | CU-0A08BS | M3 prerequisite / Browser prose stale-oracle precision | `DONE` | — | `CU-0A08BR`は[発注依存証跡](#発注依存証跡)で`DONE`。古いprose行scannerが完了済みdecoder IDと別粒の状態語を単同行で誤結合するsynthetic mirrorをpre-edit負例とする。`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`（118 pass）、`npm run test:reference-guard`、`./scripts/check-docs.sh` | prose鏡は既存の完全一致`（DO）`拒否を維持し、lane状態は既存exact table selectorが所有する。別IDの後続`DO`同居をsynthetic受理し、文字列ID個別除外を残さない。decoder/fixture/hash/React/製品/台帳次粒変更0 |
| ORACLE-GUARD | CU-0A08SSCI-P | M3 prerequisite / Browser post-promotion provenance multi-entry oracle | `DONE` | [CU-0A08SSCI-P改訂](reviews/2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md) | `CU-0A08SSCI`は[前提順序裁定](reviews/2026-07-29-cu-0a08ssci-browser-place-source-seam-prerequisite-order-decision.md)で`WAIT`。P-first採択 | append-only hash chain authorityへ§H-1 Guard 1全文置換。(B)採択。guard実装との未統一は`CU-0A08SSCI-P1`へ送った。React byte・provenance実データ変更0 |
| ORACLE-GUARD | CU-0A08SSCI-P1 | M3 prerequisite / Browser post-promotion provenance guard authority reconciliation | `DONE` | [CU-0A08SSCI-P1 guard整合](reviews/2026-07-29-cu-0a08ssci-p1-browser-post-promotion-provenance-chain-guard-reconciliation-decision.md) | `CU-0A08SSCI-P`は[CU-0A08SSCI-P改訂](reviews/2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md)で`DONE` | `validatePostPromotionChanges` を改訂済み §H-1 Guard 1（PC-1〜PC-9/R-1〜R-8）へ一致。(P)はauthorityとguard両面で閉じた。完全一致 `` `DO` `` は0件。(I)/(T)未採番 |
| PRODUCT-ASSET | CU-0A08SSCI-I0 | M3 / VS-1 / SPEC / Browser scoped identity input seam scope selection | `DONE` | [CU-0A08SSCI-I0採番](reviews/2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md) | `CU-0A08SSCI-P1`は[発注依存証跡](#発注依存証跡)で`DONE` | 未採番前提(I)を`CU-0A08SSCI-I`として採番し、次docs-only裁定が閉じる唯一の問いを固定。code・型・props名・公開境界変更0 |
| PRODUCT-ASSET | CU-0A08SSCI-I | M3 / VS-1 / SPEC / Browser scoped identity input seam contract shape adjudication | `DONE` | [CU-0A08SSCI-I裁定](reviews/2026-07-29-cu-0a08ssci-i-browser-scoped-identity-input-seam-contract-shape-decision.md) | `CU-0A08SSCI-I0`は[発注依存証跡](#発注依存証跡)で`DONE` | 候補(A) VS-1 Rectangle scoped identity 1件をprivate inputとして受け、Rectangle cardだけへ2-field identityを非推測透過。code・型・props名・公開境界変更0 |
| PRODUCT-ASSET | CU-0A08SSCI-T0 | M3 / VS-1 / SPEC / Browser private component verification harness grain numbering | `DONE` | [CU-0A08SSCI-T0採番](reviews/2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md) | `CU-0A08SSCI-I`は[発注依存証跡](#発注依存証跡)で`DONE` | 未採番前提(T)を`CU-0A08SSCI-T`として採番し、次docs-only裁定の唯一の問いを固定。code・型・harness形・公開境界変更0 |
| PRODUCT-ASSET | CU-0A08SSCI-T | M3 / VS-1 / SPEC / Browser private component verification harness boundary adjudication | `DONE` | [CU-0A08SSCI-T裁定](reviews/2026-07-29-cu-0a08ssci-t-browser-private-component-verification-harness-boundary-decision.md) | `CU-0A08SSCI-T0`は[発注依存証跡](#発注依存証跡)で`DONE` | module-private `CandidateCreateBrowser`のprivate input seamを正負両方で検証するharness境界をdocs-onlyで裁定。候補(a) AST静的検査境界を採択。code・型・harness形・公開境界変更0 |
| ORACLE-GUARD | CU-0A08SSCI-T1 | M3 / VS-1 / Browser private component verification harness implementation | `DONE` | [CU-0A08SSCI-T1実装決定](reviews/2026-07-29-cu-0a08ssci-t1-browser-private-component-verification-harness-implementation-decision.md) | `CU-0A08SSCI-T`は[発注依存証跡](#発注依存証跡)で`DONE` | AST正負harness 4/4。(T)はauthorityとguard実装の両面で閉じ、後続BTPへ送った。BTPは後続実装で`DONE`。T1自身のproduct React byte・公開export変更0 |
| PRODUCT-ASSET | CU-0A08BTR | M3 / VS-1 / SPEC / Browser read-projection dependency reclosure | `DONE` | [CU-0A08BTR依存再締結](reviews/2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md) | `CU-0A08RS`／`CU-0A08RMD`／`CU-0A08BDD`／`CU-0A08SSCI-P1`／`CU-0A08SSCI-I`／`CU-0A08SSCI-T1`は[発注依存証跡](#発注依存証跡)で`DONE` | 親BTをread-only projectionとtyped intentへ分割してBTPへ送った。BTPは後続実装で`DONE` |
| PRODUCT-ASSET | CU-0A08BTP | M3 / VS-1 / B / Browser read-only projection and JSX connection | `DONE` | [CU-0A08BTP実装決定](reviews/2026-07-29-cu-0a08btp-browser-read-projection-jsx-connection-implementation-decision.md) | `CU-0A08BP`／`CU-0A08RS`／`CU-0A08SSCI-P1`／`CU-0A08SSCI-I`／`CU-0A08SSCI-T1`は[発注依存証跡](#発注依存証跡)で`DONE` | decoded `(scope_ref, item_id)`をproduct Browser rootからprivate CandidateCreateBrowserへ非推測透過。AST 5/5、Browser 118/118、Inspector 39/39。旧immutable current-route generation `44e538c97807-ead41d4d6562`のmanifest hash再publicationはG0-6H evidence lane所有。runtime producer/Host wire/intent/drag payload 0 |
| PRODUCT-ASSET | CU-0A08BTI | M3 / VS-1 / B / Browser typed intent | `WAIT` | — | CU-0A08BTP + CU-0A08RMD + CU-0A08BDD + CU-107PV→CU-107TC→CU-107AD→CU-107TD | 既決Place chainへの1 intent。別終端、UI所有Undo/selectionは禁止 |
| ORACLE-GUARD | CU-104R | M3 prerequisite / CU-104 completion mirror closure | `DONE` | — | 隔離CU-104 decision差分でCU-104を`DONE`へ正当更新した後、Grok検収がCU-104 next-DO／選定済み表現の残存と、SN6のpost-publish follow-up reconcile禁止欠落を検出。現行入口・台帳・M3運用順・縦slice・U枝番mirrorを`CU-104 DONE / 次PRODUCT-ASSET未選定`へ同期し、Browser専用test 113/115→115/115、`npm run test:reference-guard`、docs整合を完了 | guard期待値・CU-104意味・code/fixture・次粒選定を変更せず、U2h-1実装待ちを維持して完了表現だけを閉じた |
| VISUAL-RESPONSE | G0-6H | M3 evidence / VS-1 / visual | `DO / HUMAN` | — | 5 reference screenと30 PNG | U0e-3だけを解禁可 |
| VISUAL-RESPONSE | G0-6H-E0 | M3 evidence / VS-1 / approval evidence selection | `DONE` | — | 2026-07-28のユーザー承認と[選定決定](reviews/2026-07-28-g0-6h-e-candidate-approval-evidence-selection.md) | 承認範囲を拡張せずdocs-only `G0-6H-E`を選定 |
| VISUAL-RESPONSE | G0-6H-E | M3 evidence / VS-1 / candidate approval evidence intake | `DONE` | [G0-6H-E限定観察](reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md) | `G0-6H-E0`は[発注依存証跡](#発注依存証跡)で`DONE`。Grok `ACCEPT` P0/P1=0、`./scripts/check-docs.sh`、Browser decoder 118/118 | 現行候補normal色5画面への肯定的応答だけを記録。G0-6H/CU-0B01/U0e-3状態変更0 |
| VISUAL-RESPONSE | G0-6H-R0 | M3 evidence / VS-1 / reference authority reconciliation selection | `DONE` | [G0-6H-R0選定](reviews/2026-07-28-g0-6h-r0-reference-authority-reconciliation-selection.md) | `G0-6H-E`は[発注依存証跡](#発注依存証跡)で`DONE` | 旧generationと現行product sourceのauthority再照合をdocs-only `G0-6H-R`へ選定 |
| VISUAL-RESPONSE | G0-6H-R | M3 evidence / VS-1 / reference authority reconciliation | `DONE` | [G0-6H-R決定](reviews/2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md) | `G0-6H-R0`は[発注依存証跡](#発注依存証跡)で`DONE`。Composer fallback実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118、Git ancestry exit 0 | 二つの固定commitのauthorityを非競合の別役割へ分類。route/画像/token変更0 |
| VISUAL-RESPONSE | G0-6H-S | M3 evidence / VS-1 / human-judgment input route adjudication | `DONE` | [G0-6H-S決定](reviews/2026-07-28-g0-6h-s-human-judgment-input-route-decision.md) | `G0-6H-R`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118、reference guard 290/290 | `#plugin-browser-candidate`を唯一のforward-looking人間審判入力へ裁定。旧generationは不変の再現・派生回帰証拠として保存 |
| VISUAL-RESPONSE | G0-6H-M0 | M3 evidence / VS-1 / current-route semantic gap selection | `DONE` | [G0-6H-M0選定](reviews/2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md) | `G0-6H-S`は[発注依存証跡](#発注依存証跡)で`DONE`。承認済みBrowser検索0件画面がempty projectではないことを事前照合で確認 | 意味を足さず承認5状態とG0-6必須表示要素のmapping/gapだけをdocs-only `G0-6H-M`へ送る |
| VISUAL-RESPONSE | G0-6H-M | M3 evidence / VS-1 / current-route semantic gap mapping | `DONE` | [G0-6H-M観察](reviews/2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md) | `G0-6H-M0`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118 | `対応 / partial / 対応なし / 未確認`表を閉じ、Browser検索0件はempty projectに`対応なし`、残る4画面は`partial`。scenario意味またはscreen 1改訂の人間裁定一点を返し、adapter/fixture/route/spec意味変更0 |
| VISUAL-RESPONSE | G0-6H-A0 | M3 evidence / VS-1 / empty-project scenario selection | `DONE` | [G0-6H-A0選定](reviews/2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md) | `G0-6H-M`は[発注依存証跡](#発注依存証跡)で`DONE`。ユーザーが選択肢(a)とlocal Starter Media方向を採択 | docs-only `G0-6H-A`だけを次粒に選定。asset/route/schema/code変更0 |
| VISUAL-RESPONSE | G0-6H-A | M3 evidence / VS-1 / empty-project Starter Media scenario contract | `DONE` | [G0-6H-A契約](reviews/2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md) | `G0-6H-A0`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark capacity停止後Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 | Project空、BrowserのProject外local Starter Media参照、offline fixture所有とprovenanceの契約を閉じた。asset byte/path/schema/route/code変更0 |
| VISUAL-RESPONSE | G0-6H-AF | M3 evidence / VS-1 / Starter Media source and provenance decision | `DONE` | [G0-6H-AF裁定](reviews/2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md) | `G0-6H-A`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark capacity停止後Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 | 決定的生成を採択し、pinned vendoringは本fixtureに限り棄却。media byte/path/schema/command/code変更0 |
| VISUAL-RESPONSE | G0-6H-AG0 | M3 evidence / VS-1 / Starter Media generator closure inventory | `DONE` | [G0-6H-AG0裁定](reviews/2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md) | `G0-6H-AF`は[発注依存証跡](#発注依存証跡)で`DONE`。Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 | `WRAP` / `FROZEN / DELETE-LATER`へ責任処分。既存Node/pngjs/hash/atomic patternと外部ffmpeg境界だけを再利用し、Rust WAV境界越えと新frameworkを棄却 |
| VISUAL-RESPONSE | G0-6H-AG | M3 evidence / VS-1 / Starter Media fixed evidence capsule | `DONE` | `e4ad5c9f` | `G0-6H-AG0`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark capacity停止後Composer fallback実装、Grok修復検収を経て最終`ACCEPT` P0/P1/P2=0、capsule guard 3/3、reference guard 293/293、reference check、docs整合 | PNG / MP4 / WAV / SVGの固定byte、raw provenance、closed-schema・signature・read-only integrity checkを`FROZEN / DELETE-LATER`証拠カプセルに閉じた。route/React/Document/public境界変更0 |
| VISUAL-RESPONSE | G0-6H-V0 | M3 evidence / VS-1 / current-route variant evidence contract | `DONE` | [G0-6H-V0契約](reviews/2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md) | `G0-6H-AG`は[発注依存証跡](#発注依存証跡)で`DONE`。Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Starter Media guard 3/3、Browser decoder 118/118、reference guard 293/293 | 5状態semantic mapping、capture環境9軸、normal＋派生5variant、immutable manifest/read-only check、記録された人間sessionの要求をdocsで閉じた。画像/script/fixture/token/threshold/G0-6H/U0e-3状態変更0 |
| VISUAL-RESPONSE | G0-6H-V1S | M3 evidence / VS-1 / current-route capture boundary decision | `DONE` | [G0-6H-V1S裁定](reviews/2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md) | `G0-6H-V0`は[発注依存証跡](#発注依存証跡)で`DONE`。Spark施工後のGrok REJECTをCodex限定修復し、最終Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118 | screen2〜5の同一route既存interaction、screen1のdevelopment専用typed fixture projection、Starter Media表示意味、capture環境9軸のgeneration manifest記録責任を裁定。画像/script/fixture/code変更0 |
| VISUAL-RESPONSE | G0-6H-V1P | M3 evidence / VS-1 / current-route capture prerequisite decision | `DONE` | [G0-6H-V1P選定](reviews/2026-07-28-g0-6h-v1p-capture-prerequisite-selection.md)、[G0-6H-V1P裁定](reviews/2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md) | `G0-6H-V1S`は[発注依存証跡](#発注依存証跡)で`DONE`。V1 order draftのOpus 5 `ORDER: STOP`と現行code事実を再照合 | screen1 mock-owned typed projection seam、screen2〜5操作/oracle、font fixture観測境界の三問だけをdocsで閉じる |
| VISUAL-RESPONSE | G0-6H-V1 | M3 evidence / VS-1 / current-route evidence generation | `SPLIT` | [G0-6H-V1R裁定](reviews/2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md) | `G0-6H-V1P`は[発注依存証跡](#発注依存証跡)で`DONE`。Opus 5 order draft三度の`ORDER: STOP`とFable 5 read-only助言を現行codeへ再照合 | presentation envelope `V1E`とevidence generation `V1G`へ分割 |
| VISUAL-RESPONSE | G0-6H-V1E | M3 evidence / VS-1 / screen-1 typed envelope projection | `SPLIT` | [G0-6H-V1R裁定](reviews/2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md) | Browserとmock Timeline/Stage/Inspectorのowner境界を分離 | `V1EB`→`V1ET`へ分割 |
| VISUAL-RESPONSE | G0-6H-V1EB | M3 evidence / VS-1 / Browser development projection decoder | `DONE` | [G0-6H-V1R裁定](reviews/2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md) | Opus/Spark/Grok修復ループ後に`ACCEPT` P0/P1=0。専用94/94、reference guard 387/387、Browser ownership 3/3、reference generation 30 PNG | product Browserの非公開development projection decoderだけを閉じた。component byte、provenance/guard、R-9実描画はV1ETへ |
| VISUAL-RESPONSE | G0-6H-V1ET | M3 evidence / VS-1 / mock empty projection and ready oracle | `SPLIT` | [G0-6H-V1ETA裁定](reviews/2026-07-28-g0-6h-v1eta-empty-projection-staging-decision.md) | Opus 5 order draftの`ORDER: STOP`と現行owner境界を再照合 | `V1ETC`→`V1ETB`→`V1ETT`→`V1ETE`へ分割 |
| VISUAL-RESPONSE | G0-6H-V1ETC | M3 evidence / VS-1 / carrier and Host empty projection | `DONE` | — | Spark施工、Grok `ACCEPT` P0/P1=0。専用Playwright 2/2、reference guard 387/387、Browser ownership 3/3、通常Playwright 71/71、docs整合 | Vite mode channel、legacy script抑止、Inspector / Stage空投影を閉じた。Browser / Timeline / readyは後続粒へ留保 |
| VISUAL-RESPONSE | G0-6H-V1ETB-H | M3 evidence / VS-1 / Browser post-promotion authority reclosure decision | `DONE` | — | 1. [新規decision doc](reviews/2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md) 2. `G0-6H-V1ETA` が発注依存証跡で`DONE` 3. `G0-6H-V1ETC` が発注依存証跡で`DONE` 4. `./scripts/check-docs.sh` がOK 5. `browser-catalog-decoder.test.mjs` 118/118 6. `inspector-read-model-decoder.test.mjs` 39/39 7. 4 fileの`grep -c '[[:space:]]$'`が各0で、`source-provenance.json` / `DiscoveryBrowserCandidate.jsx` SHA-256 が現行値の不変 | H-1/H-2/H-3/H-4 を閉じた。G0-6H-V1ETB-H はdocs-onlyで、`code / fixture / guard / provenance` の変更が0件 |
| VISUAL-RESPONSE | G0-6H-V1ETB-P | M3 evidence / VS-1 / Browser projection consumer and capsule boundary decision | `DONE` | [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md) | 1. [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md) 2. `G0-6H-V1ETA` が発注依存証跡で`DONE` 3. `G0-6H-V1ETC` が発注依存証跡で`DONE` 4. `G0-6H-V1EB` が発注依存証跡で`DONE` 5. `G0-6H-V1ETB-H` が発注依存証跡で`DONE` 6. `./scripts/check-docs.sh` がOK 7. `node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` が118/118 8. `node --test docs/mocks-ui/guard-tests/inspector-read-model-decoder.test.mjs` が39/39 9. 変更4 file の `grep -c '[[:space:]]$'` が各0 | capsule token閉集合、`main.jsx`単独配線、basename-only 4件envelopeをdocs-onlyで閉じる |
| VISUAL-RESPONSE | G0-6H-V1ETB-Q | M3 evidence / VS-1 / Browser route oracle allowlist correction | `DONE` | — | [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-q-browser-route-oracle-allowlist-correction-decision.md) と4依存grainのDONE確認 | V1ETB implementation allowlistを9点で確定。新規test/config 0 |
| VISUAL-RESPONSE | G0-6H-V1ETB | M3 evidence / VS-1 / Browser projected Media face | `DONE` | — | Composer代替施工をGrokが`ACCEPT`（P0/P1=0）、Codex採用commit `ae8771af`。専用Playwright 4/4、通常Playwright 71/71、`browser-catalog-decoder.test.mjs` 118/118、`inspector-read-model-decoder.test.mjs` 39/39、`browser-ownership.test.mjs` 3/3、`starter-media-capsule.test.mjs` 6/6、`npm run test:reference-guard` 390/390、ownership追試験3/3、`./scripts/check-docs.sh` OK | capture mode `#plugin-browser-candidate` で4件development Media projection、post-promotion provenance、Guard 1〜3とcapsule allowlistを閉じた |
| VISUAL-RESPONSE | G0-6H-V1ETT | M3 evidence / VS-1 / Timeline empty projection | `DONE` | — | `G0-6H-V1ETB`は[発注依存証跡](#発注依存証跡)で`DONE` | 専用6/6、通常71/71、`source-asset-inventory` 23/23、reference-guard 390/390、`npm run check-reference` OK、`./scripts/check-docs.sh` OK、protected diff OK |
| VISUAL-RESPONSE | G0-6H-V1ETE | M3 evidence / VS-1 / integrated ready oracle | `DONE` | — | `G0-6H-V1ETT`は[発注依存証跡](#発注依存証跡)で`DONE` | 専用7/7（新規1含む）、通常71/71、`source-asset-inventory` 23/23、`reference-guard` 390/390、`npm run check-reference` OK（30 PNG）、`./scripts/check-docs.sh` OK、`./scripts/check-protected-diff.sh`/`git diff --check` OK |
| VISUAL-RESPONSE | G0-6H-V1G | M3 evidence / VS-1 / current-route immutable generation | `DONE` | — | `G0-6H-V1G-P` / `G0-6H-V1G-I` / `G0-6H-V1G-C` / `G0-6H-V1G-O`は[発注依存証跡](#発注依存証跡)で`DONE` | 旧v1不変のままmanifest v2、offline capture、immutable publication、read-only再照合を閉じた |
| VISUAL-RESPONSE | G0-6H-V1G-P | M3 evidence / VS-1 / current-route generation mechanics decision | `DONE` | — | `G0-6H-V1ETE`は[発注依存証跡](#発注依存証跡)で`DONE` | [本決定](reviews/2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md)で旧v1不変とV1G-I→V1G-C→V1G-Oの契約境界を閉じた |
| VISUAL-RESPONSE | G0-6H-V1G-I | M3 evidence / VS-1 / current-route manifest and fingerprint infrastructure | `DONE` | — | `G0-6H-V1G-P`は[発注依存証跡](#発注依存証跡)で`DONE`。専用27/27、実repo closure 50 assets、旧reference generation 23/23、reference guard 417/417、ownership 3/3、旧30 PNG照合、docs/protected diff green | manifest v2閉schema、transitive source closure/provenance、旧CLIと共有するrepository fingerprintだけを閉じた。capture/publication/output/provenance実値は未作成 |
| VISUAL-RESPONSE | G0-6H-V1G-C | M3 evidence / VS-1 / current-route offline capture | `DONE` | — | `G0-6H-V1G-C-P`が[発注依存証跡](#発注依存証跡)で`DONE`。専用8/8、infrastructure 27/27、source inventory 23/23、reference generation 23/23、reference guard 425/425、ownership 3/3、通常Playwright 71/71、current-route 7/7、旧30 PNG照合、docs/protected diff green | 2 mode、5 normal、6 variant・30 PNGの非公開bundle、provenance、offline/font/browser/fingerprint検証を閉じた。publication/output/CURRENTは未作成 |
| VISUAL-RESPONSE | G0-6H-V1G-C-P | M3 evidence / VS-1 / current-route capture environment authority correction | `DONE` | [G0-6H-V1G-C-P決定](reviews/2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md) | `G0-6H-V1G-I`は[発注依存証跡](#発注依存証跡)で`DONE`。locale/timezone、pinned browser version/revision、font fixture/computed familyの既存authority衝突を再締結 | 3件のcapture環境authority衝突だけをdocs-onlyで閉じる |
| VISUAL-RESPONSE | G0-6H-V1G-O | M3 evidence / VS-1 / current-route immutable publication | `DONE` | — | `G0-6H-V1G-O-H`は[発注依存証跡](#発注依存証跡)で`DONE`。publication 93/93、capture 8/8、manifest 8/8、旧reference 23/23、source inventory 23/23、reference guard 518/518、ownership 3/3、通常Playwright 71/71、current-route 7/7、旧30 PNG照合、docs/protected diff green | 別rootのmanifest v2と30 PNGを原子的に公開し、再生成拒否、全checkpoint fault、read-only再照合、旧RG3失敗意味を閉じた |
| VISUAL-RESPONSE | G0-6H-V1G-O-H | M3 evidence / VS-1 / current-route command authority hash correction | `DONE` | [G0-6H-V1G-O-H決定](reviews/2026-07-29-g0-6h-v1g-o-h-current-route-command-authority-hash-correction-decision.md) | `G0-6H-V1G-C`は[発注依存証跡](#発注依存証跡)で`DONE` | OH-1〜OH-4で2 command wiringと3 guard hash literalの同一commit更新条件・事前検証・禁止形をdocs-onlyで閉じた |
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
| CU-0A08ITP-P | `DONE` | [CU-0A08ITP-P改訂](reviews/2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md)でInspector component byteと専用provenance chainのauthorityを先行閉鎖 |
| CU-0A08ITP | `DO` | CU-0A08ITP-P完了後の唯一のPRODUCT-ASSET実装粒 |
| CU-0A08IQ | `DONE` | commit `6f259b6e`でT7のsentence/task境界精密化を完了 |
| CU-0A08IR | `DONE` | commit `1cf46779`でInspector authority hash再締結を完了 |
| CU-0A04 | `DONE` | [快適利用粒度化 W0表](reviews/2026-07-22-m3-comfortable-use-granulation.md)の`CU-0A04`行でR1 Browserのproduct ownerへの直接移管完了を確認 |
| CU-0A08B | `SPLIT` | CU-G09完了後のreadiness再確認で、CU-G09 §4/§6/§7だけで閉じるCU-0A08BP pure decoderと、U4a-2を要するCU-0A08BT Host projection/typed intent/JSX接続へ分割 |
| CU-G09O | `DONE` | [CU-G09O Browser decoder output契約決定](reviews/2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md) |
| CU-G09R | `DONE` | [CU-G09R Browser decoder拒否優先順決定](reviews/2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md) |
| CU-0A08BP | `DONE` | `ui/motolii-web/src/read-model/browserCatalogDecoder.js`、`docs/mocks-ui/fixtures/browser-catalog-parts.json`、`docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`、`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs` |
| CU-0A08BR | `DONE` | current laneのexact lane-kind / task-ID / state selectorとsynthetic negative/positiveで、完了済みdecoder stale状態と別IDの後続状態語を分離 |
| CU-0A08BS | `DONE` | `selectStaleBrowserDecoderProseLines`とsynthetic negative/positiveで、完全一致stale prose拒否と別粒`DO`同居受理を分離。`node --test docs/mocks-ui/guard-tests/browser-catalog-decoder.test.mjs`（118 pass）、`npm run test:reference-guard`、`./scripts/check-docs.sh` |
| CU-G02 | `DONE` | 本変更でM3仕様[運用順](specs/M3-ui-integration.md)、[implementation-ledger](implementation-ledger.md)、[decision-index](decision-index.md)を同時更新し、次PRODUCT-ASSET粒を`U3a-1`へ固定した |
| CU-G09 | `DONE` | [CU-G09 Browser catalog projection契約決定](reviews/2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) |
| U3a-1 | `SPLIT` | [U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)で、公開API STOPとpub再export成果の自己矛盾を解消するため`U3a-1S`（docs decision）→`U3a-1I`（実装）へ分割 |
| U3a-1S | `DONE` | 本変更で[U3a-1 owner/visibility分割決定](reviews/2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md)を確定し、owner=`motolii-ui`内module、`motolii-timeline` crate=`REJECT`、visibility=pub再export+integration test（制約6点）を固定。`U3a-1I`のclosed orderは本文書を直接authorityにできる |
| U3a-1I | `DONE` | headless Timeline projectionを`motolii-ui::timeline_projection`として実装し、小さな決定的Document fixtureで歴史回収§6のread-only projection/layout/cull/hit-testを閉じた |
| G0-9L | `DONE` | [固定Mac local platform evidence manifest](spikes/g0-9-local-platform-evidence/manifest.json)の`PASS_FIXED_MAC_PREREQUISITE_EVIDENCE_ONLY`。W0b/H1b/Motolii Studio Preview/通常製品window/Distribution Ready/renderer winnerを解禁しない |
| D1m | `DONE` | [M2仕様 D1m](specs/M2-document-model.md)でproject-scoped sidecar identity、process間session lock、`ProjectSession`経由のproject/journal変更を完了 |
| D2 | `DONE` | [M2仕様 D2](specs/M2-document-model.md)でコマンドapply/revert、Undo/Redo、atomic gesture macroを[#109](https://github.com/oshikaidesu/Motolii/pull/109) / [#130](https://github.com/oshikaidesu/Motolii/pull/130)により完了 |
| U2b-1 | `DONE` | [M3仕様 U2b](specs/M3-ui-integration.md)と[次にIssue化するもの](#次にissue化するもの)で、prepared requestのsingle writer配送と成功snapshot publishを完了 |
| CU-G03D | `DONE` | [CU-G03 edit durability / publish順序決定](reviews/2026-07-26-cu-g03-edit-durability-ordering-decision.md)で、VS-1単一command actionのjournal→live Apply/Undo/Redo→revision→selection reconcile→1 publish、failure poison、複数command STOPを確定 |
| CU-G03R | `DONE` | [#369](https://github.com/oshikaidesu/Motolii/pull/369)でcatalog未反映committed Edit tailをMainFile fast pathから既存replayへ送り、stale-catalog負例、main原本不変、純checkpoint・二重checkpoint対照、Grok ACCEPT P0/P1=0、CI 4/4を完了 |
| CU-G03 | `DONE` | 子粒`CU-G03D`と`CU-G03R`が本表でともに`DONE` |
| CU-101 | `DONE` | [U2b-2 Place product core再採択 §3.2](reviews/2026-07-23-historical-d2-selection-timeline-lineage-recovery.md#32-product-placeの閉じた意味)でtarget/start/duration/recipe/position/nameの閉じた値とappearance非目標を確認 |
| CU-102 | `DONE` | [fresh LayerId + AddTrackItem原子性決定](reviews/2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md)でclone候補、fresh live二条件、live mint 0、1 Command/1 macro、failure不変、既存journal互換を確定 |
| CU-104 | `DONE` | [CU-104 selection publish envelope決定](reviews/2026-07-27-cu-104-selection-publish-envelope-decision.md) |
| U2h-1 | `SPLIT` | [U2h-1 primary selection implementation split決定](reviews/2026-07-27-u2h-1-primary-selection-implementation-split-decision.md) |
| U2h-1S | `DONE` | [U2h-1 primary selection implementation split決定](reviews/2026-07-27-u2h-1-primary-selection-implementation-split-decision.md) |
| U2h-1I | `DONE` | `crates/motolii-ui/src/document_edit_runtime.rs` + `crates/motolii-ui/src/app.rs` と実行コマンド: `cargo fmt --all`/`cargo fmt --all --check`、`cargo clippy -p motolii-ui --all-targets -- -D warnings`、`cargo test -p motolii-ui`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --locked --workspace`、`./scripts/check-docs.sh` |
| CU-104E | `DONE` | [CU-104E projection generation枯渇境界決定](reviews/2026-07-27-cu-104e-projection-generation-exhaustion-decision.md) |
| U2h-1PR | `DONE` | [U2h-1P selection入力到達性決定](reviews/2026-07-27-u2h-1p-selection-input-reachability-decision.md)でproducer-only粒を停止し、production caller同差分条件を固定 |
| CU-105R | `DONE` | [CU-105 dense Timeline責任再確認](reviews/2026-07-27-cu-105-dense-timeline-responsibility-recheck.md)でlayout/cull/hit-test、capacity、semantic zoom、selection-familyを`PASS / REDUCE / STOP`へ分割 |
| CU-106S | `DONE` | [CU-106 selection consumer分割決定](reviews/2026-07-27-cu-106-selection-consumer-split-decision.md)でCU-106P/Fを分離し、production caller 0のままproducerを実装しない入場条件を固定 |
| U3a-2S | `DONE` | [U3a-2S windowed native Timeline readiness分割決定](reviews/2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md)でG0-9依存を(A)〜(D)に分割し、次PRODUCT-ASSET判断を`U3a-2R` `DO`へ送った |
| U3a-2S-R2 | `DONE` | 本変更でimplementation-ledger M3行、[docs/README](README.md)現況、[縦slice実行方針](reviews/2026-07-24-m3-vertical-slice-execution-decision.md)のcurrent表を`U3a-2S` `DONE`／次判断`U3a-2R` `DO`へ同期した。意味・順序・code変更0 |
| U3a-2S-R3 | `DONE` | 本変更で[CU-106 selection consumer分割決定](reviews/2026-07-27-cu-106-selection-consumer-split-decision.md) §5と[U2h-1P selection入力到達性決定](reviews/2026-07-27-u2h-1p-selection-input-reachability-decision.md) §6の現行handoffを`U3a-2S` `DONE`／次判断`U3a-2R` `DO`へ同期した。意味・順序・code変更0 |
| U3a-2R | `DONE` | [U3a-2R renderer採択範囲決定](reviews/2026-07-27-u3a-2r-renderer-adoption-scope-decision.md)で区分(D)のcandidate閉集合・証拠admissibility・owner・entry gateを閉じ、次PRODUCT-ASSET判断を`U3a-2Z` `DO`へ送った |
| U3a-2Z | `DONE` | [U3a-2Z semantic zoom責任所在決定](reviews/2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md)で段階の中身を決めず責任所在だけを閉じ、次PRODUCT-ASSET判断を`U3a-2A` `DO`へ送った |
| U3a-2A | `DONE` | [U3a-2A renderer採択決定](reviews/2026-07-27-u3a-2a-renderer-adoption-decision.md)でconfirmation型`direct_vello`採択を閉じ、次PRODUCT-ASSET判断を`U3a-2P` `DO`へ送った |
| U3a-2P | `DONE` | [U3a-2P playhead visible range範囲決定](reviews/2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md)で五層閉集合・admissibility・不変規則・entry gateを閉じ、次PRODUCT-ASSET判断を`U3a-2Q` `DO`へ送った |
| U3a-2Q | `DONE` | [U3a-2Q playhead visible range owner採択分割判断](reviews/2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md)で§6証拠coverage非対称によりplayhead/visible range owner判断を`U3a-2Q-P`/`U3a-2Q-V`へ分割し、次PRODUCT-ASSET判断を`U3a-2Q-P` `DO`へ送った |
| U3a-2Q-P | `DONE` | [U3a-2Q-P playhead owner admissibility evidence補遺](reviews/2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md)でE1〜E4証拠面を補いT1でowner一意導出不可を記録し、次PRODUCT-ASSET判断を`U3a-2Q-P2` `DECIDE`へ送った |
| U3a-2Q-P2 | `DONE` | [U3a-2Q-P2 playhead 再 open lifetime 決定](reviews/2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md)でfresh Host coordinatorのproject再open時は以前のplayheadを復元せず安全な初期位置へ戻すと決め、次PRODUCT-ASSET判断を`U3a-2Q-P3` `DO`へ送った |
| U3a-2Q-P3 | `DONE` | owner採択orderのOpus 5 `ORDER: STOP`を受け、[U3a-2Q-P3 playhead 将来reopen復元posture決定](reviews/2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md)で将来のbest-effort復元を延期・追加可能と裁定し、次PRODUCT-ASSET判断を`U3a-2Q-P4` `DO`へ送った |
| U3a-2Q-P4 | `DONE` | [U3a-2Q-P4 playhead 五層 state owner 採択](reviews/2026-07-27-u3a-2q-p4-playhead-five-layer-owner-adoption-decision.md)で`T2`によりplayhead state ownerを`Project session`一層として採択し、次PRODUCT-ASSET `DO`を0件（`U3a-2Q-V` `WAIT`）へ送った |
| CU-109S0 | `DONE` | [CU-109S0選定](reviews/2026-07-27-cu-109s0-readiness-recheck-selection-decision.md)で、CU-109実装前のprepared-action順序再確認を`CU-109S`として次PRODUCT-ASSET `DO`へ選定 |
| CU-109S | `DONE` | [CU-109S順序再確認](reviews/2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md)でR2・候補(b)を裁定し、次PRODUCT-ASSET `DO`をdocs-only `CU-109SP`へ送った。`CU-109`は`WAIT`維持 |
| CU-109SP | `DONE` | [CU-109SP prerequisite決定](reviews/2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md)でP1を裁定し、次PRODUCT-ASSET `DO`を`CU-109`へ送った。acceptance evidenceはApply roundtripに限定 |
| CU-109SP-R1 | `DONE` | Grok `REJECT` P1=2後のrolling mirror修復として、[decision-index](decision-index.md) rolling M3 VS-1行のP1 precedence／acceptance evidence表記と[縦slice実行方針](reviews/2026-07-24-m3-vertical-slice-execution-decision.md) `selection / Undo再投影`行の`CU-109S DONE`を同期し、次PRODUCT-ASSET `DO`を`CU-109`へ戻した。意味・公開API・code変更0 |
| CU-G04S0 | `DONE` | [CU-G04S0選定](reviews/2026-07-27-cu-g04s0-session-source-selection-decision.md)でCU-109 orderのOpus `STOP`とFable助言を再照合し、session source未決だけをCU-G04側docs粒`CU-G04S`へ選定。`CU-109`は`WAIT`へ戻した |
| CU-G04S | `DONE` | [CU-G04S session source決定](reviews/2026-07-27-cu-g04s-edit-runtime-session-source-decision.md)でD1〜D7を裁定し、次PRODUCT-ASSET `DO`を`CU-109`へ送った。acceptance evidenceはApply roundtripに限定 |
| CU-G04SC0 | `DONE` | [CU-G04SC0選定](reviews/2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md)でCU-109再発注orderのOpus `STOP`とFable助言を再照合し、製品path carrier未決だけをCU-G04側docs粒`CU-G04SC`へ選定。`CU-109`は`WAIT`へ戻した |
| CU-G04SC | `DONE` | [CU-G04SC product path handoff決定](reviews/2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md)でC1〜C6を裁定し、次PRODUCT-ASSET `DO`を`CU-109`へ送った。acceptance evidenceはApply roundtripに限定 |
| CU-109 | `DONE` | [#425](https://github.com/oshikaidesu/Motolii/pull/425)、実装commit `356d703f`、merge commit `32cf8902bf5c96fc60400a91335e72a9886cf304`、Grok `ACCEPT` P0=0 / P1=0 / P2=0 |
| CU-110S | `DONE` | [CU-110S選定](reviews/2026-07-28-cu-110s-dependency-scope-decision-selection.md)で`CU-110`前提範囲の未決だけをdocs-only `CU-110D`へ選定。次PRODUCT-ASSET `DO`をdocs-only `CU-110D`へ送った |
| CU-110D | `DONE` | [CU-110D決定](reviews/2026-07-28-cu-110d-cu-107-dependency-scope-decision.md)で候補(B)を採り、より狭い名前付き前提への分割方針を裁定。`CU-110`はnon-test production drop sourceまで`WAIT`維持。次PRODUCT-ASSET `DO`をdocs-only `CU-107S`へ送った |
| CU-107S | `DONE` | [CU-107S選定](reviews/2026-07-28-cu-107s-split-concretization-scope-selection.md)で`CU-107`分割具体化の未決範囲をdocs-only `CU-107D`へ選定。次PRODUCT-ASSET `DO`をdocs-only `CU-107D`へ送った |
| CU-107D | `DONE` | [CU-107D決定](reviews/2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md)で候補(B)を採択し、`CU-110`が必要とする責任範囲の限定を先に閉じる順序を裁定。次PRODUCT-ASSET `DO`をdocs-only `CU-107R`へ送った |
| CU-107R | `DONE` | [CU-107R決定](reviews/2026-07-28-cu-107r-cu-110-required-responsibility-decision.md)で `CU-110` が必要とする `CU-107` 責任範囲を厳密な部分集合へ再確定し、次PRODUCT-ASSET `DO` をdocs-only `CU-107N`へ送った |
| CU-107N | `DONE` | [CU-107N決定](reviews/2026-07-28-cu-107n-cu-107-narrow-prerequisite-closed-set.md)で7 load-bearing clause を4前提の閉集合へ分割し、単一 owner 割当と依存順を確定。次PRODUCT-ASSET `DO` をdocs-only `CU-107W`へ送った |
| CU-107W | `DONE` | [CU-107W決定](reviews/2026-07-28-cu-107w-w0-mirror-rewrite-decision.md)でW0 §8 W1表へ4前提行を追加し、親`CU-107`の合格cellをroll-upへ、`CU-110`依存を`CU-102`+4前提+`CU-109`へ書き換えた。次PRODUCT-ASSET `DO`は未選定（0件） |
| CU-107W-R1 | `DONE` | Grok検収 `REJECT`（P0=0 / P1=1、P2非blocking助言1件）の修復として、[decision-index](decision-index.md)の`CU-110S`〜`CU-107W`行へ`CU-107W`の`DONE`明記を追加し、[CU-107W決定](reviews/2026-07-28-cu-107w-w0-mirror-rewrite-decision.md) W-2／W-7を承認発注書の逐語文面へ復元した。意味・状態・依存・公開境界・code変更0 |
| CU-0A08RS0 | `DONE` | [CU-0A08RS0選定](reviews/2026-07-29-cu-0a08rs0-browser-inspector-read-projection-dependency-scope-selection.md)で`U4a-2` Direct製品入口依存のload-bearing可否をdocs-only `CU-0A08RS`へ一問選定。次PRODUCT-ASSET `DO`をdocs-only `CU-0A08RS`へ送った |
| CU-0A08RS | `DONE` | [CU-0A08RS裁定](reviews/2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md)でVS-1 read-only projectionに`U4a-2`はload-bearingでないと候補(B)を採択。次PRODUCT-ASSET `DO`をdocs-only `CU-0A08RM`へ送った |
| CU-0A08RM0 | `DONE` | [CU-0A08RM0選定](reviews/2026-07-29-cu-0a08rm0-browser-typed-intent-dependency-adjudication-scope-selection.md)でBrowser typed-intent依存の未裁定を一問へ限定し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08RMD`へ送った |
| CU-0A08RMD | `DONE` | [CU-0A08RMD裁定](reviews/2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md)で候補(A)をVS-1 Rectangleに限定採択し、依存の向きと実装順の選定をdocs-only `CU-0A08BD0`へ送った |
| CU-0A08BD0 | `DONE` | [CU-0A08BD0選定](reviews/2026-07-29-cu-0a08bd0-browser-typed-intent-dependency-direction-scope-selection.md)で`CU-0A08BT`とPlace連鎖の依存方向・実装順の一問を限定し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08BDD`へ送った |
| CU-0A08BDD | `DONE` | [CU-0A08BDD裁定](reviews/2026-07-29-cu-0a08bdd-browser-typed-intent-dependency-direction-decision.md)で候補(A) Browser source-seam firstをVS-1 Rectangleに限定採択し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SS0`へ送った |
| CU-0A08SS0 | `DONE` | [CU-0A08SS0選定](reviews/2026-07-29-cu-0a08ss0-browser-place-source-seam-implementation-boundary-scope-selection.md)でBrowser Place source seamの最小実装境界を一問へ限定し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SSD`へ送った |
| CU-0A08SSD | `DONE` | [CU-0A08SSD裁定](reviews/2026-07-29-cu-0a08ssd-browser-place-source-seam-implementation-boundary-decision.md)で候補(A) product-owned React Browser source seamをVS-1 Rectangleに限定採択し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SSC`へ送った |
| CU-0A08SSC | `DONE` | [CU-0A08SSC選定](reviews/2026-07-29-cu-0a08ssc-browser-place-source-seam-contract-concretization-scope-selection.md)でscoped identityの受け渡し責任を置く既存component境界の一問へ限定し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SSCD`へ送った |
| CU-0A08SSCD | `DONE` | [CU-0A08SSCD裁定](reviews/2026-07-29-cu-0a08sscd-browser-place-source-seam-contract-concretization-decision.md)で候補(B) CandidateCreateBrowser境界をVS-1 Rectangleに限定採択し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SSCS`へ送った |
| CU-0A08SSCS | `DONE` | [CU-0A08SSCS選定](reviews/2026-07-29-cu-0a08sscs-browser-place-source-seam-implementation-scope-selection.md)で最小closed implementation orderの範囲を一問へ限定し、次PRODUCT-ASSET `DO`をdocs-only `CU-0A08SSCSD`へ送った |
| CU-0A08SSCSD | `DONE` | [CU-0A08SSCSD裁定](reviews/2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md)で候補(A) 内部source seamのみをVS-1 Rectangleに限定採択し、次PRODUCT-ASSET `DO`を最小コード実装粒`CU-0A08SSCI`へ送った |
| CU-0A08SSCI-P | `DONE` | [CU-0A08SSCI-P改訂](reviews/2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md)で§H-1 Guard 1をappend-only hash chain authorityへ全文置換し、次ORACLE-GUARD `DO`を`CU-0A08SSCI-P1`へ送った |
| CU-0A08SSCI-P1 | `DONE` | [CU-0A08SSCI-P1 guard整合](reviews/2026-07-29-cu-0a08ssci-p1-browser-post-promotion-provenance-chain-guard-reconciliation-decision.md)で`browser-ownership.test.mjs`を改訂済み§H-1 Guard 1へ一致させ`(P)`をauthorityとguard両面で閉じ、完全一致 `` `DO` `` を0件にした |
| CU-0A08SSCI-I0 | `DONE` | [CU-0A08SSCI-I0採番](reviews/2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md)で前提(I)を`CU-0A08SSCI-I`として採番し、次PRODUCT-ASSET/SPEC `` `DO` `` をdocs-only `CU-0A08SSCI-I`へ送った |
| CU-0A08SSCI-I | `DONE` | [CU-0A08SSCI-I裁定](reviews/2026-07-29-cu-0a08ssci-i-browser-scoped-identity-input-seam-contract-shape-decision.md)で候補(A)を採択し、次PRODUCT-ASSET/SPEC `` `DO` `` をdocs-only `CU-0A08SSCI-T0`へ送った |
| CU-0A08SSCI-T0 | `DONE` | [CU-0A08SSCI-T0採番](reviews/2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md)で前提(T)を`CU-0A08SSCI-T`として採番し、次PRODUCT-ASSET/SPEC `` `DO` `` をdocs-only `CU-0A08SSCI-T`へ送った |
| CU-0A08SSCI-T | `DONE` | [CU-0A08SSCI-T裁定](reviews/2026-07-29-cu-0a08ssci-t-browser-private-component-verification-harness-boundary-decision.md)で候補(a) AST静的検査境界を採択し、次ORACLE-GUARD `` `DO` `` を`CU-0A08SSCI-T1`へ送った |
| CU-0A08SSCI-T1 | `DONE` | [CU-0A08SSCI-T1実装決定](reviews/2026-07-29-cu-0a08ssci-t1-browser-private-component-verification-harness-implementation-decision.md)でsynthetic AST正負harnessを実装し、`(T)`をauthorityとguard実装の両面で閉じた |
| G0-6H-E0 | `DONE` | [G0-6H-E0選定](reviews/2026-07-28-g0-6h-e-candidate-approval-evidence-selection.md)で、現行候補5画面へのユーザー承認を旧U0e-2 generation全体へ拡張せず記録するdocs-only `G0-6H-E`を選定 |
| G0-6H-E | `DONE` | [G0-6H-E限定観察](reviews/2026-07-28-g0-6h-e-candidate-approval-observation.md)で、現行候補normal色5画面への肯定的応答を旧generation・派生variant・具体tokenへ拡張せず取込。Grok `ACCEPT` P0/P1=0、docs整合とBrowser decoder 118/118 |
| G0-6H-R0 | `DONE` | [G0-6H-R0選定](reviews/2026-07-28-g0-6h-r0-reference-authority-reconciliation-selection.md)で、旧generation authorityと現行product source authorityの関係だけを再照合するdocs-only `G0-6H-R`を選定 |
| G0-6H-R | `DONE` | [G0-6H-R決定](reviews/2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md)で、`eb16d06f`を旧generation限定の再現authority、`56c318ed`を現行product sourceと承認5画面のprovenanceへ分類。Composer fallback実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118、Git ancestry exit 0 |
| G0-6H-S | `DONE` | [G0-6H-S決定](reviews/2026-07-28-g0-6h-s-human-judgment-input-route-decision.md)で、G0-6H人間審判入力を現行`#plugin-browser-candidate`へ一本化し、旧generationを不変の再現・派生回帰証拠へ分類。Spark実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118、reference guard 290/290 |
| G0-6H-M0 | `DONE` | [G0-6H-M0選定](reviews/2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md)で、承認済みBrowser検索0件画面がG0-6のempty projectを満たさないgapを確認し、V0の前へdocs-only mapping粒`G0-6H-M`を選定 |
| G0-6H-M | `DONE` | [G0-6H-M観察](reviews/2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md)で承認5画面とG0-6意図のelement-level gapを非推測で固定。Spark実装、Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118。`G0-6H-V0`はempty-project scenario意味またはscreen 1改訂の人間裁定待ちで`WAIT`維持 |
| G0-6H-A0 | `DONE` | [G0-6H-A0選定](reviews/2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md)で、ユーザー採択のempty Project＋Project外local Starter Media方向を限定受領し、docs-only `G0-6H-A`を唯一の次粒へ選定 |
| G0-6H-A | `DONE` | [G0-6H-A契約](reviews/2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md)で、Project assets 0・空Stage/Inspector/TimelineとProject外fixture-only Starter Mediaを同時成立させ、offline固定provenanceとproduction正本化禁止を確定。Spark capacity停止後Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 |
| G0-6H-AF | `DONE` | [G0-6H-AF裁定](reviews/2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md)で、決定的生成を採択し、pinned vendoringは本Starter Media fixtureに限り棄却。cross-platform byte決定性は未主張のまま`G0-6H-AG0`へgenerator責任処分を送った。Spark capacity停止後Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 |
| G0-6H-AG0 | `DONE` | [G0-6H-AG0裁定](reviews/2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md)で、既存Node/pngjs/hash/atomic patternと外部ffmpeg CLI境界を`WRAP`、証拠カプセルを`FROZEN / DELETE-LATER`へ処分。Rust WAV helperのJS境界越えと新codec/framework/serviceを棄却。Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Browser decoder 118/118 |
| G0-6H-AG | `DONE` | `e4ad5c9f`でPNG / MP4 / WAV / SVGの固定byte、raw provenance、closed-schema・signature・read-only integrity checkを`FROZEN / DELETE-LATER`証拠カプセルへ固定。Composer fallback実装をGrok修復検収2周後に`ACCEPT` P0/P1/P2=0、capsule guard 3/3、reference guard 293/293、reference check、docs整合 |
| G0-6H-V0 | `DONE` | [G0-6H-V0契約](reviews/2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md)で、5状態semantic mapping、capture環境9軸、1画面6variant、immutable manifest＋read-only照合、human session記録項目の閉集合を確定。Composer fallback実装、Grok `ACCEPT` P0/P1/P2=0、docs整合、Starter Media guard 3/3、Browser decoder 118/118、reference guard 293/293 |
| G0-6H-V1S | `DONE` | [G0-6H-V1S裁定](reviews/2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md)で、screen2〜5の同一route既存interaction、screen1のdevelopment専用typed fixture projection、Starter Media表示意味、capture環境9軸のgeneration manifest記録責任を確定。Spark施工、Grok REJECT後のCodex限定修復、最終Grok `ACCEPT` P0/P1=0、docs整合、Browser decoder 118/118 |
| G0-6H-V1P | `DONE` | [G0-6H-V1P裁定](reviews/2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md)で、screen1 seam / screen2〜5 interaction oracle / font fixture観測境界の三問を裁定。 |
| G0-6H-V1R | `DONE` | [G0-6H-V1R裁定](reviews/2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md)で、screen 1のInspector / Stage / Timeline / Browser / ready oracleを閉じ、親V1をpresentation `V1E`とgeneration `V1G`へ分割。 |
| G0-6H-V1EB | `DONE` | product-owned非公開Starter Media projection decoderを閉じた。Spark再実装、Grok `ACCEPT` P0/P1=0、専用94/94、reference guard 387/387、Browser ownership 3/3、reference generation 30 PNG、docs整合。component seamと実DOMは`G0-6H-V1ET`へ留保。 |
| G0-6H-V1ETA | `DONE` | [G0-6H-V1ETA裁定](reviews/2026-07-28-g0-6h-v1eta-empty-projection-staging-decision.md)でV1ETをcarrier / Host、Browser、Timeline、統合readyへ段階化し、専用Playwright channel、selector閉集合、provenance再締結、ready意味を固定。 |
| G0-6H-V1ETC | `DONE` | commit `affadf3e`。Spark施工、Grok `ACCEPT` P0/P1=0、Codex再実測で専用Playwright 2/2、reference guard 387/387、Browser ownership 3/3、通常Playwright 71/71、docs整合。capture mode限定のHost / Inspector / Stage空投影を閉じた。 |
| G0-6H-V1ETB-H | `DONE` | [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md)でH-1/H-2/H-3/H-4閉済 |
| G0-6H-V1ETB-P | `DONE` | [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md)でP-1/P-2/P-3閉済 |
| G0-6H-V1ETB-Q | `DONE` | [本粒decision doc](reviews/2026-07-28-g0-6h-v1etb-q-browser-route-oracle-allowlist-correction-decision.md)でQ-1/Q-2/Q-3閉済 |
| G0-6H-V1ETB | `DONE` | Composer代替施工をGrokが`ACCEPT`（P0/P1=0）、Codex採用commit `ae8771af`。専用Playwright 4/4、通常Playwright 71/71、catalog decoder 118/118、inspector 39/39、ownership 3/3、starter-media capsule 6/6、reference guard 390/390、ownership追試験3/3、`./scripts/check-docs.sh` OK。9 file閉包のみ変更 |
| G0-6H-V1ETT | `DONE` | 専用Playwright 6/6、通常Playwright 71/71、`source-asset-inventory` 23/23、reference-guard 390/390、`npm run check-reference` OK、`./scripts/check-docs.sh` OK、`./scripts/check-protected-diff.sh`/`git diff --check` OK |
| G0-6H-V1ETE | `DONE` | 専用Playwright 7/7（うち新規1件）、通常Playwright 71/71、`source-asset-inventory` 23/23、reference-guard 390/390、`npm run check-reference` OK（30 PNG）、`./scripts/check-docs.sh` OK、`./scripts/check-protected-diff.sh`/`git diff --check` OK |
| G0-6H-V1G-P | `DONE` | [G0-6H-V1G-P決定](reviews/2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md)で旧v1不変、現行route root/command、manifest v2、2 mode・5 screen、transitive closure、font fallback STOP、offline、共有fingerprint、V1G-I→C→O直列をdocs-onlyで閉じた |
| G0-6H-V1G-I | `DONE` | manifest v2閉schema、transitive closure/provenance、共有repository fingerprintを実装。専用27/27、実repo closure 50 assets、旧reference generation 23/23、reference guard 417/417、ownership 3/3、旧30 PNG照合、docs/protected diff green。capture/publicationは後続へ留保 |
| G0-6H-V1G-C-P | `DONE` | [G0-6H-V1G-C-P決定](reviews/2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md)でlocale/timezone、pinned browser version/revision、font fixture/computedFamilyの3衝突、runtime観測、manifest sole ownerをdocs-onlyで閉じた |
| G0-6H-V1G-C | `DONE` | 2 mode、5 normal、6 variant・30 PNGの非公開bundleとprovenanceを実装。専用8/8、infrastructure 27/27、source inventory 23/23、reference generation 23/23、reference guard 425/425、ownership 3/3、通常Playwright 71/71、current-route 7/7、旧30 PNG照合、docs/protected diff green。publication/output/CURRENTは後続へ留保 |
| G0-6H-V1G-O-H | `DONE` | [G0-6H-V1G-O-H決定](reviews/2026-07-29-g0-6h-v1g-o-h-current-route-command-authority-hash-correction-decision.md)で2 command wiringと3 guard hash literalの同一commit更新条件・事前検証・禁止形をdocs-onlyで閉じた |
| G0-6H-V1G-O | `DONE` | 2 command、共有immutable planner、manifest v2と30 PNGの32-file generation、read-only照合を実装。publication 93/93、capture 8/8、manifest 8/8、旧reference 23/23、source inventory 23/23、reference guard 518/518、ownership 3/3、通常Playwright 71/71、current-route 7/7、旧v1 hash不変 |
| G0-6H-V1G | `DONE` | V1G-P / V1G-I / V1G-C / V1G-Oの直列完了により、現行routeのmanifest v2、offline capture、immutable publication、read-only再照合を閉じた |
| U2c-1 | `DONE` | [U2c-1 共通interaction state machine契約](reviews/2026-07-21-m3-u2c-1-interaction-state-contract.md)と[M3仕様](specs/M3-ui-integration.md)で、selectionを非目標とするtoolkit非依存6状態machineの実装完了を確認 |
| U2c-4 | `DONE` | [U2c-4 Transient Diagnostic Envelope契約](reviews/2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md)と[M3仕様](specs/M3-ui-integration.md)で、selection/target推測を持たない診断adapter境界の実装完了を確認 |
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

したがって現在の短い運用判断は、**CU-0A03 / R0からCU-0A07C / R4Cまでは完了済み。CU-0A08IP、`U3a-1S`/`U3a-1I`、`CU-G03D`/`CU-G03R`、親`CU-G03`、`CU-101`、`CU-102`、`CU-G09`/`CU-G09O`/`CU-G09R`、`CU-0A08BP`、docs-only `CU-104`/`U2h-1S`/`CU-104E`、`U2h-1I`、docs-only `U2h-1PR`/`CU-105R`/`CU-106S`/`U3a-2S0`、docs-only `U3a-2S`、docs-only `U3a-2R`、docs-only `U3a-2Z`、docs-only `U3a-2A`、docs-only `U3a-2P`、docs-only `U3a-2Q`、docs-only `U3a-2Q-P`、docs-only `U3a-2Q-P2`、docs-only `U3a-2Q-P3`、docs-only `U3a-2Q-P4`、docs-only `CU-109S0`、docs-only `CU-109S`、docs-only `CU-109SP`、docs-only `CU-G04S0`、docs-only `CU-G04S`、docs-only `CU-G04SC0`、docs-only `CU-G04SC`、実装粒`CU-109`、docs-only `CU-110S`、docs-only `CU-110D`、docs-only `CU-107S`、docs-only `CU-107D`、docs-only `CU-107R`、docs-only `CU-107N`、docs-only `CU-107W`は`DONE`。`CU-105`/`CU-106`親は`SPLIT`し、`U2h-1P`/`CU-106P`/`CU-106F`は実consumer surfaceまで`WAIT`。docs-only `CU-0A08RS0`/`CU-0A08RS`/`CU-0A08RM0`/`CU-0A08RMD`/`CU-0A08BD0`/`CU-0A08BDD`/`CU-0A08SS0`/`CU-0A08SSD`/`CU-0A08SSC`/`CU-0A08SSCD`/`CU-0A08SSCS`/`CU-0A08SSCSD`は`DONE`。`CU-0A08RM`はOpus `ORDER: STOP`により`WAIT`。実装粒 `CU-0A08SSCI` は当初Opus 5 order判定 `STOP` だったが、`CU-0A08BTR`でprivate seam責任をBTPへ吸収して`SPLIT`。docs-only `CU-0A08SSCI-P` は`DONE`。ORACLE-GUARD `CU-0A08SSCI-P1` は`DONE`。`(P)` は authority と guard 実装の両面で閉じた。ORACLE-GUARD `CU-0A08SSCI-T1`は`DONE`。`(T)`はauthorityとguard実装の両面で閉じた。`CU-0A08BTR`は`DONE`、親`CU-0A08BT`は`SPLIT`。`CU-0A08BTP`は`DONE`。`CU-0A08ITP-P`は`DONE`、次の唯一のPRODUCT-ASSET `DO`は`CU-0A08ITP`。`CU-0A08SSCI`はBTPへ実装責任を吸収して`SPLIT`。`CU-0A08BTP`以外の未完了PRODUCT-ASSET lane行は`WAIT`。`U3a-2Q-V`は`WAIT`。CU-109 runtime配線、CU-110 Place、CU-111 prepared Undo/Redoを混ぜない。`CU-0A08BTP`は`DONE`。Inspector read-only projectionはITPを実装中。`CU-0A08BTI`/`CU-0A08IT`/`U2c-2`は既存依存待ち。Host transport、typed intent、JSX binding、drag payload、`S`行、Rust/schema/plugin変更を`U3a-2Q-V` visible range owner 粒へ混ぜるならSTOPする。Motolii Studio Previewは未実装。**G0-6HはU0e-3だけを止め、G0-9DはDistribution Readyまで`WAIT / HARDWARE`。D1n、D5等の独立follow-upをVS-1の再停止理由へしない。

## 更新規則

- Issue作成時: ID、Issue URL、依存、完了後の出口を追加する。
- PR merge時: 対象を`DONE`へ移すか行を削り、直接の後続を`ISSUE`または`DO`へ上げる。
- decision完了時: `DECIDE`を消し、実装タスクを`ISSUE`へ上げ、後続発注が依存するIDを発注依存証跡へ同じ変更で追加する。
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
