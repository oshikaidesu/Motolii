# 実装進行台帳

最終確認: **2026-07-29**

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
| M3 | **VS-1 Rectangle配置とUndo / R2A再入場decision** | React chrome + native Stage/Timeline + headless interaction、1 top-level wgpu Surface + 2 native viewport + opaque child WebView islandsを正本化。U0e-2R/U0e-2、GR-D1〜R3、固定MacのG0-9L platform prerequisite evidence、固定SHA`56c318ed`の6面R0 source inventory、CU-0A04/R1 Browser ownershipは完了済み。Easing triggerの独立React source不在を受けR2を05A/05Bへ分割したが、Opus prepareで固定／抽出後hash、status、単一owner経路の未決を検出した。現在は`CU-0A05A / R2A`のdocs decisionだけ進め、抽出実装はdecision merge後に再発注する。R2とMotolii Studio Previewは未実装。visible summary chromeは未決で実装しない。G0-6HはU0e-3を止める並行人間審判、G0-9DはDistribution Ready用hardware gate。eguiへ新規製品面を実装せず、plugin UI公開契約はG0-3 / GAP-13まで停止する |
| M4 | **契約spike可** | [歴史20版再照合](reviews/2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md)と[memory model 6版再照合](reviews/2026-07-23-historical-memory-model-lineage-recovery.md)後もK0〜K8は未実装。K0でRoD/RoIのruntime契約を凍結し、その後K1階層基盤→K7 group freeze→K8全曲Draft coverageへ進む。現行`PipelineCache`／dynamic target pool／wgpu budget thresholdをResourceLedger、copy-out、disk store完成と数えない。K4の恒久`source_id`／再リンク／package意味はGAP-3／7の再調査前に焼かないが、このgateをK0や独立K1へ広げない。K6のVello／usvg製品統合は未実装で、R8成立性だけを完成証拠にしない |
| M5 | **identity meaning decision可 / RCI・RCS1 DONE / RCD1 decision可** | P0I自身が所有するcontinuity／transform／nested identity／寿命／cache入力境界／PRNG処分をdocsで先に閉じ、TextCluster内部写像とPrototype ownerは明示留保する。P2D-RCIは要求／contribution分離、Host所有、能力分離、追加的進化、First Vism conformance役割を閉じ、`P2D-RCS1`はprivate実depth spikeでF1/F6を成立させた。次はRCI §2.2のcamera／Observation非所有を保つ`P2D-RCD1` docs decisionだけDO。Document、plugin契約、alpha/refraction方式と公開実装はWAIT。P6のfontique／harfrust／Vello text stackは未実装で、K6とpremul adapterを重複実装しない |

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
→ G0-9L DONE → R0 source inventory DONE → R1 Browser ownership DONE → R2A Easing trigger mock extraction → R2B product ownership → rolling VS-1 enabling order

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
| PRODUCT-ASSET | CU-0A05A | M3 / VS-1 / A / R2A | `DECIDE` | — | CU-0A04、固定SHA、直接移管契約。固定／抽出後hash、status、単一owner経路を先に決める。[旧隔離差分](reviews/2026-07-25-cu-0a05a-interrupted-worktree-restart-disposition.md)は証拠カプセル | decision merge後にmock-side extractionを再発注 |
| VISUAL-RESPONSE | G0-6H | M3 evidence / VS-1 / visual | `DO / HUMAN` | — | 5 reference screenと30 PNG | U0e-3だけを解禁可 |
| AUTHORING-SCAFFOLD | VSM-A4S | Vism / spec | `DO / SPEC` | — | VSM-A1/A2/A3、仕様と実装の別PR決定 | VSM-A4Iは全体レビュー後 |
| DELEGATION-GUARD | GR-D3 | supervised runner / derived output closure | `DO` | [#329](https://github.com/oshikaidesu/Motolii/issues/329) | GR-D2、K0で再現したworktree-root `target/`汚染 | 実停止経路でGrok到達を証明後、K0を再発注 |
| DELEGATION-GUARD | GR-D4 | supervised runner / restartable order loop | `DONE` | — | GR-D2、M5 RCA2/B2/C2停止証跡、Opus/Fable read-only監査 | GR-D3を従来scopeのまま継続 |
| SPATIAL-CONTRACT | K0 | M4 / contract spike | `WAIT` | [#167](https://github.com/oshikaidesu/Motolii/issues/167) | M2-D3、凍結ゲート解凍手続き、GR-D3 | fresh worktree/orderで再発注し、K1系を粒ごとに再判定 |
| IDENTITY-CONTRACT | P0I | M5 / identity decision | `DECIDE` | [#170](https://github.com/oshikaidesu/Motolii/issues/170) | 凍結ゲート、2026-07-15決定。Text／Prototype側の未決は留保 | 意味decision後にfixture粒を分割して再判定 |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA | M5 / P2D / authority draft | `STOPPED` | — | 旧一括grainはGrok REJECT、差分不採用。後継P2D-RCA2登録済み | 旧IDを発注しない |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB | M5 / P2D / Rerun evidence | `STOPPED` | — | 旧一括grainはSpark context枯渇、差分なし。後継P2D-RCB2登録済み | 旧IDを発注しない |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC | M5 / P2D / primary-source evidence | `STOPPED` | — | 旧一括grainはSpark context枯渇、差分なし。後継P2D-RCC2登録済み | 旧IDを発注しない |
| RENDER-CONTRIBUTION-EVIDENCE | P2D-RCE0 | M5 / P2D / fixed evidence acquisition | `DONE` | — | Rerun固定6 asset、Bevy 0.19、Godot 4.6、Unreal 5.8を6 capsuleへ固定。Rerun A1〜A5=`PATTERN`、A6=`REJECT` | 新しい比較／転記／fixture翻訳grainだけがread-only入力にする |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA2 | M5 / P2D / boundary comparison | `STOPPED` | — | Spark成功、Grok有効markerなし。差分不採用 | 後継P2D-RCA3登録済み |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB2 | M5 / P2D / adjudicated evidence transcription | `STOPPED` | — | Grok REJECT P0=1/P1=1。差分不採用 | 後継P2D-RCB3登録済み |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC2 | M5 / P2D / fixture evidence comparison | `STOPPED` | — | Grok REJECT P0=0/P1=5/P2=2。差分不採用 | 後継P2D-RCC3三provider登録済み |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA3 | M5 / P2D / fixed boundary comparison | `STOPPED` | — | Grok REJECT P0=2/P1=1、差分不採用 | 後継P2D-RCA4 |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB3 | M5 / P2D / fixed Rerun transcription | `STOPPED` | — | Grok REJECT P0=0/P1=1、差分不採用 | 後継P2D-RCB4 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC3-BEVY | M5 / P2D / Bevy transcription | `STOPPED` | — | Grok REJECT P0=0/P1=5、差分不採用 | 後継P2D-RCC4-BEVY |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC3-GODOT | M5 / P2D / Godot transcription | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | RCC5入力 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC3-UNREAL | M5 / P2D / Unreal transcription | `STOPPED` | — | Opus DESIGN_STOP、差分なし | 後継P2D-RCC4-UNREAL |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB4 | M5 / P2D / fixed Rerun fragment map | `STOPPED` | — | Opus command-oracle不備3回、Spark未起動 | 後継P2D-RCB5 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC4-BEVY | M5 / P2D / Bevy fragment map | `STOPPED` | — | Grok ACCEPT後、現行固定配置と2 cell不一致。差分不採用 | 後継P2D-RCC4B-BEVY |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC4-UNREAL | M5 / P2D / Unreal fragment map | `STOPPED` | — | Opus command-oracle不備6回、Spark未起動 | 後継P2D-RCC4B-UNREAL |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA4 | M5 / P2D / boundary fragment map | `STOPPED` | — | Opus command-oracle不備4回、Spark未起動 | 後継P2D-RCA5 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC4B-BEVY | M5 / P2D / fixed Bevy fragment map | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | RCC5入力 |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA5 | M5 / P2D / fixed boundary map | `STOPPED` | — | command machine block前、Opus不備3回、Spark未起動 | 後継P2D-RCA6 |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB5 | M5 / P2D / fixed Rerun map | `STOPPED` | — | Grok REJECT P0=1、A1行二重挿入、差分不採用 | 後継P2D-RCB6 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC4B-UNREAL | M5 / P2D / fixed Unreal map | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | RCC5入力 |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA6 | M5 / P2D / machine-command boundary map | `STOPPED` | — | Opus本文のrunner所有操作列挙でORDER_INVALID、Spark未起動 | 後継P2D-RCA7 |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA7 | M5 / P2D / fixed boundary map | `STOPPED` | — | Opus 3稿が形式収束せずORDER_INVALID、Spark未起動 | 後継P2D-RCA8 |
| RENDER-CONTRIBUTION-RERUN | P2D-RCB6 | M5 / P2D / fixed Rerun map | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | P2D-RCI入力 |
| RENDER-CONTRIBUTION-AUTHORITY | P2D-RCA8 | M5 / P2D / fixed boundary map | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | P2D-RCI入力 |
| RENDER-CONTRIBUTION-ENGINE | P2D-RCC5 | M5 / P2D / fixture comparison | `DONE` | — | Grok ACCEPT P0/P1/P2=0、主担当再照合 | P2D-RCI入力 |
| RENDER-CONTRIBUTION-INTEGRATION | P2D-RCI | M5 / P2D / semantic integration decision | `DONE` | — | 主担当Codex決定、Fable read-only反対側監査 | private RCS1だけ解禁 |
| RENDER-CONTRIBUTION-SPIKE | P2D-RCS1 | M5 / P2D / private opaque Group Depth | `DONE` | — | Grok ACCEPT P0/P1=0、runner固定3 command exit 0、主担当再照合 | 実depth F1/F6、group外pixel不変、公開API／Document／serde変更0 |
| RENDER-CONTRIBUTION-CONTRACT | P2D-RCD1 | M5 / P2D / typed seam decision | `DO` | — | P2D-RCS1、RCI §2.2のcamera／Observation非所有 | docs decision一seat。P3型と公開実装は先取りしない |
| RENDER-CONTRIBUTION-SCHEMA | P2D-RCD2 | M5 / P2D / policy schema decision | `WAIT` | — | P2D-RCD1、M2-D1e | GR-PV、追加migration |
| RENDER-CONTRIBUTION-FIXTURE | P2D-RCF1 | M5 / P2D / conformance harness | `WAIT` | — | P2D-RCD1 | First Vism専用口なし |
| RENDER-CONTRIBUTION-ALPHA | P2D-RCT1 | M5 / P2D / cutout soft alpha semantics | `WAIT` | — | P2D-RCD1 | F2/F3、OIT方式別裁定 |
| RENDER-CONTRIBUTION-OIT | P2D-RCO1 | M5 / P2D / transparent OIT decision | `WAIT` | — | P2D-RCT1、P2D-RCS1 | 方式、品質、budget、unsupported |
| RENDER-CONTRIBUTION-FORMAT | P2D-RCFP1 | M5 / P2D / scene-color format decision | `WAIT` | — | M1、M4-K0 | linear FP16推奨案を再裁定 |
| RENDER-CONTRIBUTION-REFRACTION | P2D-RCR1 | M5 / P2D / scene-color input contract | `WAIT` | — | P2D-RCD1、P2D-RCFP1 | snapshot、範囲、順序、failure |
| RENDER-CONTRIBUTION-COPY | P2D-RCP1 | M5 / P2D / scene-color copy decision | `WAIT` | — | P2D-RCR1、P2D-RCFP1 | lifetime、同期、画面外sample、budget |
| RENDER-CONTRIBUTION-BUDGET | P2D-RCBUD1 | M5 / P2D / cache and budget integration | `WAIT` | — | P2D-RCD1、M4-K1 | Host計上とcache key完全性 |
| M2-REPAIR | GAP-23 | M2 / narrow repair | `WAIT` | — | 独立D1i-4 LookAt/Follow oracle分離の採番・完了 | GAP-23全25件を再発注し、後にGAP-24を判定 |
| M2-REPAIR | GAP-24 | M2 / narrow repair | `WAIT` | — | GAP-23 | GAP-24後にclose |
| ORACLE-GUARD | GAP-25 | M2 / guard repair | `DO / CHECK-PATH` | — | GAP-23との許可path非重複 | semantic oracle gate自己保護 |
| PRODUCT-ASSET | U2c-2 | M3 / VS-2 / D | `WAIT` | — | U4a-2のDirect製品入口とU4cのAdvanced製品入口 | 実在入口のDocument意味/Undo同値conformance |

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

K0 [#167](https://github.com/oshikaidesu/Motolii/issues/167)のcontract spikeとP0I
[#170](https://github.com/oshikaidesu/Motolii/issues/170)のdocs decisionはPRODUCT-ASSETと同時着手できる。
Selected U seriesの一時点`DO`一粒はPRODUCT-ASSET lane内だけに適用し、K0、P0I decision、
M2 prerequisite、Vism spec laneを同じ待ち列へ入れない。P0I fixtureとGAP-23実装は各lane-localな前提待ちで、
共有contract変更または変更許可pathの重複が判明したlaneだけをSTOPし、他laneは継続する。

## 発注依存証跡

`DEPENDENCY`の機械判定専用表。現在粒の依存を散文や別phaseの状態から推測せず、この表で`DONE`の
一意な行だけを受理する。完了証拠が変わった時は、該当spec／decisionと同じ変更で更新する。

| ID | 状態 | 完了証拠 |
|---|---|---|
| CU-0A04 | `DONE` | R1 Browser ownership完了。現在粒`CU-0A05A`の直前product asset |
| GR-D2 | `DONE` | [監督ループ決定](reviews/2026-07-25-opus-spark-grok-supervision-loop-decision.md)へ変更許可閉集合、append-only証跡、timeout分離、検収者mutation拒否、resumeを継承済み |
| GR-D4 | `DONE` | [再開可能な監督発注loop](reviews/2026-07-29-restartable-supervised-order-loop-decision.md)と専用testがmachine block、prepare非破壊、parent outcome、terminal verdict、availability限定resumeを固定 |
| M1-FREEZE-GATE | `DONE` | [凍結ゲート宣言](reviews/2026-07-10-freeze-gate-declaration.md)がM2〜M5並列laneを解禁 |
| M2-D3 | `DONE` | [M2仕様 D3](specs/M2-document-model.md)のDocument→render graph接続が完了 |
| M2-FOUNDATION-RECLOSURE | `DONE` | [M2基盤再締結ゲート](reviews/2026-07-15-m2-foundation-reclosure-gate.md)はmainで解除済み |
| VSM-A1 | `DONE` | Vism計画Phase Aのfirst-party公開境界監査完了 |
| VSM-A2 | `DONE` | Vism計画Phase AのParamDriver外部crate参照実装完了 |
| VSM-A3 | `DONE` | Vism計画Phase AのRadial Repeater実装・審判完了 |
| P2D-RC0 | `DONE` | [Render Contribution証拠Wave親task](reviews/2026-07-29-m5-render-contribution-evidence-wave.md)が共通Host境界、固定語彙／anchor、code fact hash、共通非目標、責任分離、旧RCA/B/C停止、後続再登録WAITを固定 |
| P2D-RCE0 | `DONE` | [Rerun転移裁定](reviews/2026-07-29-m5-rerun-transfer-adjudication.md)と6 evidence capsuleが固定source／version、license、非証明範囲、製品非import、削除条件、Rerun asset別分類を固定 |
| P2D-RCA8 | `DONE` | [Motolii境界map](reviews/2026-07-29-m5-render-contribution-boundary-map-v4.md)がGrok ACCEPT P0/P1/P2=0、主担当再照合済み |
| P2D-RCB6 | `DONE` | [Rerun観察map](reviews/2026-07-29-m5-rerun-observation-map-v4.md)がGrok ACCEPT P0/P1/P2=0、主担当再照合済み |
| P2D-RCC5 | `DONE` | [provider横断fixture map](reviews/2026-07-29-m5-provider-fixture-map-v5.md)がGrok ACCEPT P0/P1/P2=0、主担当再照合済み |
| P2D-RCI | `DONE` | [Render Contribution統合decision](reviews/2026-07-29-m5-render-contribution-integration-decision.md)が意味、負例、private spike、後続停止線を固定 |
| P2D-RCS1 | `DONE` | private `motolii-render` spikeが実GPU depth attachmentによるF1、未使用／group外pixel不変のF6、FINAL／DRAFT同一評価関数を固定。Grok ACCEPT P0/P1=0、公開API／Document／serde変更0 |

## 次にIssue化するもの

前段PRがmainへ入った時点で、最新の型名・fixture・依存を確認してから起票する。

| 順序 | ID | Phase | 状態 | 起票条件 | 次の出口 |
|---|---|---|---|---|---|
| 1 | D1j | M2 | `DONE` | CAM-G0 merge（D1lはmain到達済み） | v5 planar camera schema/default migration |
| 2 | U2b-1 | M3 | `DONE` | U1b-2 merge | prepared requestをsingle writerへ配送し、成功snapshotをUI/render workerへ購読 |
| 3 | U3a-1 | M3 | `WAIT` | Selected U seriesのU2c-5までmain到達（論理依存はU0a+U0bのみ） | toolkit/renderer非依存のDocument→Timeline projection/layout/cull/hit-testを小さなfixtureで閉じる。G0-9や100k再実測を入場条件にしない |
| 4 | U3a-2 | M3 | `WAIT` | U3a-1 + G0-9 platform受入 | direct wgpu+Vello候補をwindowed fixture、input、WebView同居、presentまで閉じる。Canvas/browser WebGPUは先例baselineで製品枝にしない |
| 5 | U2g | M3 | `WAIT` | D1l + D3e + U0e + U2b + U3a-2 merge | Effect常時接続線 |
| 6 | K1a | M4 | `WAIT` | K0 merge | ResourceLedgerとhard budget。backendの空きVRAM値を正本にしない |
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

したがって現在の短い運用判断は、**CU-0A03 / R0とCU-0A04 / R1 Browser ownershipは完了済み。固定SHAのEasing triggerはarchived HTMLにだけ存在し独立React sourceが無いためR2を05A/05Bへ分割した。VS-1の現在の製品orderは`CU-0A05A / R2A mock-side extraction`であり、同形React化と既存parityだけを閉じる。accessibleなobject・channel・pressed/disabled状態を超えるvisible summaryは未決・実装禁止。R2とMotolii Studio Previewは未実装。**G0-6Hは同時に進められる人間審判だが、未完了でもR0〜R2やPreview骨格を止めず、U0e-3だけを止める。G0-9DはDistribution Readyまで`WAIT / HARDWARE`。`U2c-2`はVS-2候補かつ実製品入口待ちである。D1n、D5等の独立follow-upをVS-1の再停止理由へしない。

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
