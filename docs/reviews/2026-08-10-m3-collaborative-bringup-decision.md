# M3 共同開発bring-up決定

日付: 2026-08-10

状態: **決定 / ACTIVE**

## 1. 決定

M3はR0受入後の段階を、全surfaceの整合を先に閉じてから一括統合する候補開発ではなく、`ui/motolii-rn/`のRN artifactを常に起動可能に保ちながら既存資産を継続統合する**共同開発bring-up**として進める。このpathを別製品UIへの移植元にはしない。

共同出口は次である。

> 最新mainの`ui/motolii-rn/` artifactで、Browser、Stage、Timeline、Inspectorが同じproject、identity、revisionを表示・操作し、Document mutationはD2 single writerだけを通り、Undo／Redoまで動く。

これはR1-E2E／R2-E2Eを省略する決定ではない。各laneの小さい製品出口をmainへ積み上げ、最後の一括統合までcandidateを隔離し続けないという施工順の変更である。

## 2. 最初の並列lane

| lane | 既存入力 | 最初の製品出口 |
|---|---|---|
| Stage | 固定Rerun Spatial Viewer subsystem、`ui/motolii-rn/`の現行Stage、単一`GpuCtx` | Document入力をRerunへ薄く翻訳し、同じRN Stageで同revisionの結果を表示する |
| Timeline | 既存Skia実行fixture、Timeline設計決定、現行projection／gesture oracle | 既存fixtureを再設計せずRN native Timeline surfaceで表示する |
| Browser／Inspector | 現行RN panel、Place／Undo／Redo、initial snapshot projection | Place後の更新snapshotをInspectorへ再投影する |
| integration seat | RN app root、`rn_product_host` ABI、native registration、単一GPU owner | laneを一つの起動可能なRN artifactへ順次取り込む |

Rerunのstore／query／View／visualizer／camera／picking／renderer閉包を製品runtimeとして使う。ただしRerun Entity／BlueprintをDocument、Undo、selection、playheadの第二authorityにはしない。Timelineをゼロから再設計せず、旧Vello routeへ新機能を足さない。

## 3. mergeと停止

- `1 contract = 1 owner = 1 commit`を維持する。
- lane固有のfocused oracleと通常製品routeへの接続が通れば、他surfaceの完成をmerge条件にしない。
- `rn_product_host` ABI、RN app root／publication、D2 command／journal、wgpu Device／Queue、selection／playhead ownerはintegration seatだけが合流する。
- 共有境界の変更が必要なlaneだけを局所returnし、file-disjointなlaneを止めない。
- probe内の局所fixtureやtest greenを製品完成へ繰り上げない。ただし同じartifactへ実Document入力を接続して製品oracleを通した時は、copyせずartifactの状態を`PRODUCT_SOURCE`へ更新する。
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

## 6. RN製品sourceへのcutover（2026-08-11再訂正）

**`ui/motolii-rn/`** を唯一のRN製品source兼write targetとする。ここにはRN shell、Browser、Inspector、Timeline、Rerun、Skia、Fabric native component registrationがすでに同居する。別targetへ移植せず、このartifactへDocument／D2の実入力を一つずつ接続する。

旧名`MotoliiRnProbe`と`spikes/motolii-rn-probe/`は2026-08-11のcutoverで退役した。app identityは`MotoliiRn`、product pathは`ui/motolii-rn/`である。固定fixtureとBuild IDは局所能力の証拠に限り、Document入力へ接続できた箇所から置き換える。

`ui/motolii-rn-legacy/`はread-onlyの旧製品shellとする。利用者がexact path付きで明示的に解凍するまで、ここへ製品接続を追加しない。

次を禁止する。

- 新しいRN app、shell、app root、製品entrypointを作る
- 製品の既存画面、registration、rendererを`ui/motolii-rn-legacy/`へcopy／再実装する
- `ui/motolii-rn-legacy/`へwriteする、または二つのRN appを並行して製品化する
- buildや接続が難しいことを理由に、別UI、第二Host、第二Document writer、第二GPU ownerへ迂回する

この凍結は現行製品UIを変更しない意味ではない。製品接続は`ui/motolii-rn/`内で継続する。`ui/motolii-rn-legacy/`を再びtargetにする場合だけ、利用者の明示判断を受け、代替target、移行route、cutover oracle、現targetの退役を閉じた解凍decisionを先にmainへ入れる。
