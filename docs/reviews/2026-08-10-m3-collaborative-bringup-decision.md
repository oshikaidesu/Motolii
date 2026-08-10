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
| Stage | Rerun固定commitから裁定したviewport／renderer lifecycle／picking／outline／composite pattern、現行RN Stage、単一`GpuCtx` | 通常RN Stageで同revisionのpreview／selection overlayが表示される |
| Timeline | 既存Skia実行fixture、Timeline設計決定、現行projection／gesture oracle | 既存fixtureを再設計せずRN native Timeline surfaceで表示する |
| Browser／Inspector | 現行RN panel、Place／Undo／Redo、initial snapshot projection | Place後の更新snapshotをInspectorへ再投影する |
| integration seat | RN app root、`rn_product_host` ABI、native registration、単一GPU owner | laneを一つの起動可能なRN artifactへ順次取り込む |

StageでRerunのEntity、Blueprint、store、cache key、View classまたは公開型を採らない。Timelineをゼロから再設計せず、旧Vello routeへ新機能を足さない。

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
- **ADOPTION ROUTE**: `REUSE / VERIFY_CANDIDATE / PORT / PATTERN`
- **REJECTED CANDIDATES**: 新しい統合framework、第二Host、第二writer、第二GPU device、全surface一括merge gate
- **THIN MOTOLII SEAM**: typed intent、revisioned snapshot、native component registration、GPU composite
- **THIN MOTOLII RESIDUAL**: 製品固有identity、D2 admission、Undo／Redo、Motolii fixture
- **RETIREMENT**: R2-E2E後に旧direct-wgpu／Vello製品入口を退役
- **BUILD JUSTIFICATION**: `NONE`
- **BUILD**: 既存seamの接続だけを許可

## 5. 非目標

- 整合性、single writer、GPU owner、revision一致を弱めること
- 未検収branchを一括mergeすること
- 全laneの完了を一つのcommitまたは一つの巨大PRへ束ねること
- 共同開発管理用の新しいqueue、DB、runner、frameworkを作ること
