# M3 共同開発bring-up決定

日付: 2026-08-10

状態: **決定 / ACTIVE**

## 1. 決定

M3はR0受入後の段階を、全surfaceの整合を先に閉じてから一括統合する候補開発ではなく、通常RN製品artifactを常に起動可能に保ちながら既存資産を継続統合する**共同開発bring-up**として進める。

共同出口は次である。

> 最新mainのRN製品artifactで、Browser、Stage、Timeline、Inspectorが同じproject、identity、revisionを表示・操作し、Document mutationはD2 single writerだけを通り、Undo／Redoまで動く。

これはR1-E2E／R2-E2Eを省略する決定ではない。各laneの小さい製品出口をmainへ積み上げ、最後の一括統合までcandidateを隔離し続けないという施工順の変更である。

## 2. 最初の並列lane

| lane | 既存入力 | 最初の製品出口 |
|---|---|---|
| Stage | 固定Rerun Spatial Viewer subsystem、現行RN Stage、単一`GpuCtx` | Document入力をRerunへ薄く翻訳し、通常RN Stageで同revisionの結果を表示する |
| Timeline | 既存Skia実行fixture、Timeline設計決定、現行projection／gesture oracle | 既存fixtureを再設計せずRN native Timeline surfaceで表示する |
| Browser／Inspector | 現行RN panel、Place／Undo／Redo、initial snapshot projection | Place後の更新snapshotをInspectorへ再投影する |
| integration seat | RN app root、`rn_product_host` ABI、native registration、単一GPU owner | laneを一つの起動可能なRN artifactへ順次取り込む |

Rerunのstore／query／View／visualizer／camera／picking／renderer閉包を製品runtimeとして使う。ただしRerun Entity／BlueprintをDocument、Undo、selection、playheadの第二authorityにはしない。Timelineをゼロから再設計せず、旧Vello routeへ新機能を足さない。

## 3. mergeと停止

- `1 contract = 1 owner = 1 commit`を維持する。
- lane固有のfocused oracleと通常製品routeへの接続が通れば、他surfaceの完成をmerge条件にしない。
- `rn_product_host` ABI、RN app root／publication、D2 command／journal、wgpu Device／Queue、selection／playhead ownerはintegration seatだけが合流する。
- 共有境界の変更が必要なlaneだけを局所returnし、file-disjointなlaneを止めない。
- mock、probe、fixture、candidate、test greenを製品統合へ繰り上げない。各短waveはmain到達後に状態を更新する。
- Windows実機、人間審判、署名／配布はR4外部gateのまま残す。

## 4. 既知実装preflight

- **MECHANISM CLASS**: desktop product UI bring-upとGit共同開発
- **KNOWN IMPLEMENTATION SEARCH**: 現行RN seat、Rerun transfer map、Skia fixture、Git branch／worktree／commit／PR
- **CANDIDATES**: 既存Stage candidate、既存Timeline fixture、既存RN panels
- **ADOPTION ROUTE**: Rerun Spatial Viewerを`ADOPT / WRAP`、既存RN shell／D2／Skia Timelineを`REUSE`
- **REJECTED CANDIDATES**: 新しい統合framework、第二Host、第二writer、第二GPU device、全surface一括merge gate
- **THIN MOTOLII SEAM**: Document→Rerun identity／time／asset翻訳、typed terminal intent、native Stage mount
- **THIN MOTOLII RESIDUAL**: 製品固有identity、D2 admission、Undo／Redo、Motolii fixture
- **RETIREMENT**: R2-E2E後に旧direct-wgpu／Vello製品入口を退役
- **BUILD JUSTIFICATION**: `NONE`
- **BUILD**: Rerun内部機構を作らず、既存subsystemへの薄い接続だけを許可

## 5. 非目標

- 整合性、single writer、GPU owner、revision一致を弱めること
- 未検収branchを一括mergeすること
- 全laneの完了を一つのcommitまたは一つの巨大PRへ束ねること
- 共同開発管理用の新しいqueue、DB、runner、frameworkを作ること

## 6. RN製品UI target凍結（2026-08-11追補）

M3のRN製品UI接続先を **`ui/motolii-rn/` 一つへ凍結**する。ここが製品shell、app root、Browser、Inspector、通常panel、Fabric native component registrationを所有する。Stage、Timeline、Rerun、Skia、Host snapshot／intentの製品接続は、この既存targetへ追加する。

`spikes/motolii-rn-probe/`は依存共存、描画方式、visual reference、実機成立性を確かめる **`PROBE ONLY`** targetである。probeの画面、`App.tsx`、native registration、renderer fixtureを製品sourceまたは第二app rootへ昇格させず、製品機能をprobe側へ継続実装しない。必要な知見だけを、既存`ui/motolii-rn/`のownerとcontractへ翻訳する。

次を禁止する。

- 新しいRN app、shell、app root、製品entrypointを作る
- `ui/motolii-rn/`の既存panelまたはcomponentを縮約copyして別targetへ置く
- `spikes/motolii-rn-probe/`を製品route、製品fallback、feature branch間の中継targetにする
- buildや接続が難しいことを理由に、別UI、第二Host、第二Document writer、第二GPU ownerへ迂回する

この凍結は「UIを変更しない」意味ではない。製品UIは`ui/motolii-rn/`の内部で継続開発する。target自体を変更する場合だけ、利用者の明示判断を受け、代替target、移行route、cutover oracle、既存targetの退役を閉じた解凍decisionを先にmainへ入れる。同じcommitでdecision indexとimplementation ledgerを更新し、会話、probe成功、局所build failureだけでは解凍しない。
