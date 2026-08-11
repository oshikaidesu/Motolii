# PR席候補一覧

日付: 2026-08-11
状態: **運用候補表 / 発注authorityではない**

## 目的

利用者に見える次の一歩を、GitHub Issueへ翻訳する前の候補として並べる。一覧の行は予約、実装許可、固定順序ではない。`READY_TO_OPEN`へ上げる時にcurrent mainから一契約境界を再compileし、`.github/ISSUE_TEMPLATE/closed-contract.yml`へ転記して初めてIssueを開ける。

進捗の正本はIssue、PR、main、[implementation ledger](implementation-ledger.md)である。この表をqueue、lock、第二ledger、遠い将来の仮API一覧にしない。

## 既知実装preflight

```text
MECHANISM CLASS: Issue化前の候補catalogとshared-seat衝突の可視化
KNOWN IMPLEMENTATION SEARCH: GitHub Issue／PR、Closed contract template、implementation ledger、各採択地図、叩き台PR統合決定
CANDIDATES: GitHub Issue／PRを正本としてREUSE、docs候補表をIssue前の薄い入口としてPATTERN
ADOPTION ROUTE: REUSE / PATTERN
REJECTED CANDIDATES: GitHub Project／独自queue／lock service／第二ledger :: draft itemと実装状態のownerを増やす
THIN MOTOLII SEAM: candidate rowをcurrent mainでclosed orderへcompileし、既存Issue templateへ転記する
THIN MOTOLII RESIDUAL: 利用者成果、shared seat、再入場条件、製品状態の候補だけを保持する
RETIREMENT: Issue化後はGitHubとmainを正とし、rowはURLと状態だけへ縮退する
BUILD JUSTIFICATION: NONE
BUILD: FORBIDDEN
```

## 状態

| 状態 | 意味 |
|---|---|
| `OBSERVE` | 利用者成果は見えるが、owner、target、既知route、oracleのどれかが未閉鎖 |
| `COMPILE` | current mainの調査とclosed-order化を行える。まだIssueを開かない |
| `READY_TO_OPEN` | current main、owner、shared seat、exact target、正負oracle、既知routeが閉じ、Issue化できる |
| `OPEN` | Issueあり、未発注 |
| `IN_PR` | IssueからPRが開いている。次の同一shared seatを発注しない |
| `WAIT` | dependency、authority、semantic ownerまたはexternal gate待ち。再入場条件を持つ |
| `LANDED` | mainへ到達済み。次edge選定の入力であり、再発注しない |

`COMPILE`は実装発注状態ではない。候補をIssueへ上げる直前に、`MECHANISM CLASS / KNOWN IMPLEMENTATION SEARCH / CANDIDATES / ADOPTION ROUTE / REJECTED CANDIDATES / THIN MOTOLII SEAM / THIN MOTOLII RESIDUAL / RETIREMENT / BUILD JUSTIFICATION / BUILD: FORBIDDEN`を閉じる。

## Rectangleの正準意味と分散したprojection

PR前の根本authorityは[Stage Heroとprojection root決定](reviews/2026-08-11-m3-m5-stage-hero-projection-root-decision.md)とする。M5由来のRerun Stageを第一consumerにするが、Document／D2の意味ownerを移さない。

Rectangleの正準な永続意味とUndo ownerを新設しない。ただし「一型、一moduleへ製品意味が集約済み」とも扱わない。現状は次の背骨とsurface別projectionへ分かれている。

```text
RN Browser place_rectangle
  -> rn_product_host
  -> DocumentEditRuntime / PlaceRectangleRequest
  -> D2 Command::AddTrackItem
  -> Document
       LayerId
       Clip(start, duration)
       ClipSource::Vector
       VectorRecipe
       VectorContent::StandardShape
       StandardShape::Rect(width, height)
       transform.position
  -> accepted revision / projection_generation
  -> surface別のStage / Timeline / Inspector read projection
  -> Undoで三面から消える / Redoで同じ意味が戻る
```

| 意味／翻訳 | 現在のowner | current code fact |
|---|---|---|
| identity、Clip時間、Rect width／height、position、永続化 | `motolii-doc::Document` | `LayerId`付き`ClipSource::Vector / VectorRecipe / StandardShape::Rect` |
| 作成、revision、selection reconcile、Undo/Redo | `DocumentEditRuntime` | D2 `Command::AddTrackItem`と`PublishedDocument { snapshot, primary, projection_generation }` |
| Preview／Export評価 | `motolii-doc::graph`から既存render graph | Vector Rectを評価して不透明白`OverlayRect`へlower済み |
| Timeline時間表示 | `motolii-ui::timeline_projection` | Document Clipを`TimelineBar { layer, start, end, band }`へ投影済み。shape値は持たない |
| RN Inspector／selection carrier | `RnProductHost::snapshot_wire` | revision、time、primary、`layer_id / display_name`まで。Rect width／height／positionはwireへ未投影 |
| Stage編集幾何／hit-test | `motolii-ui::stage_geometry_projection` | modifierなしの`StandardShape::Rect`は同じ`LayerId`／size／world／cameraで幾何化済み。他のVectorとmodifier付きRectは`StageGeometryUnavailable::VectorSource` |
| Rerun Stage表示 | Rerun Spatial Viewer + Motolii Host seam | custom Path2D表示はprobe済み。Document snapshot／time／LayerIdからRerun入力への製品projectionは未接続 |

管理上の合流点は新しい永続Shape schemaやRerun storeではなく、既存`PublishedDocument`相当のaccepted snapshot／primary／revision／projection generationである。そこから各surfaceが必要なbounded read modelだけを導出し、同じDocument clone、selection store、historyをsurface別に持たない。surface出力自体は既決どおりStage／Timeline／Inspectorへ分け、巨大な共通UI objectや三面同時pixel barrierへ再統合しない。

Draft PR #470のB003はこのspineをまだ読まない。Reactのbooleanからnative renderer内の固定5点pathへ直結した表示probeで、`LayerId`、Clip時間、Rect parameter、accepted revision、Undo/Redoを持たない。したがってB003の着地をM3 Rectangle本体の成立や、下記R1 consumerのdependency解除として数えない。

## 現在の候補

| seat | 利用者成果 | 状態 / 再入場 | semantic owner / shared seats | 既知routeと薄い残余 | primary oracle / known limits |
|---|---|---|---|---|---|
| `RN-B003-RECTANGLE` | 既存probe BrowserのCreateから矩形を選び、既存probe Stageへpath rectangleを表示する | `IN_PR / SCOPE DELTA` [Issue #469](https://github.com/oshikaidesu/Motolii/issues/469) / [Draft PR #470](https://github.com/oshikaidesu/Motolii/pull/470)。新Issueを作らない。IssueのB002合成維持と利用者指示による判別用単独表示の差を着地前にIssue／PRへ同期する | probe内RN App state。`App.tsx`、Fabric component、C ABI、`RendererCore`。M3のsemantic ownerではない | 既存RN probe shellとRerun `LineDrawableBuilder`を`REUSE`。Motolii残余はCreate操作、bool translation、判別用fixture | 実アプリでpath単独表示、targeted Jest、Rust test、Xcode build。固定5点pathで、`LayerId`／Clip時間／Rect parameter／accepted revision／Undo/Redoは0。M3 VS-1へ繰り上げない |
| `M3-R1-HOST-RECTANGLE` | BrowserのRectangleを一度だけD2へ置き、同じ`LayerId`／revisionをpublishし、Undo/Redoする | `LANDED`。`R1-BROWSER`=`5b6e6c56`、`R1-HOST-EDIT`=`37d88be0`。再発注しない | `rn_product_host`、`DocumentEditRuntime`、D2 single writer、Documentの`VectorRecipe / StandardShape::Rect` | 既存`PlaceRectangleRequest`、`Command::AddTrackItem`、published snapshotを`REUSE`。Rerun entityやRN stateへ意味を移さない | `r1_rn_product_edit_intents`、`cu110_product_place_commit`、`cu111_product_undo_redo`。本体は成立済みだがRN三面の製品表示完走は未成立 |
| `M3-R1-BROWSER-RECTANGLE` | 現RN Browserの通常Create catalogからRectangleを選べる | `PARTIAL / UI ENTRY LANDED`。Effectsを親にした共通Browser view、Media由来のサムネイルのみ表示、Rectangleの表示／選択はcurrent product sourceへ着地。terminalは`R1-SHELL`の共通Host接続待ち | `ui/motolii-rn/App.tsx`のBrowser catalog／view。Document writer 0 | `DiscoveryBrowserCandidate`の情報階層と既存Media thumbnail densityを`REUSE`し、3タブはデータだけを共通viewへ渡す。専用bridge、mock Document、固定中央配置を作らない | Jestで3タブ共通view、サムネイルのみ／カード／リスト、CREATE→Rectangle表示／選択。未達は明示project path、Product Host staticlib、共通intent bridge、source identity、terminal一回 |
| `M3-R1-STAGE-RECTANGLE` | accepted DocumentのRectangleを、同じ`LayerId`／revision／timeでRN Stage Heroへ表示する | `COMPILE / PR NOT OPEN`。[Stage Hero root](reviews/2026-08-11-m3-m5-stage-hero-projection-root-decision.md#5-prへcompileする順序)に従い、既存`R1-STAGE`とM5 S2のsnapshot/time/identity projectionを一契約へ閉じる。`R1-GPU-BINDING`残余を同時に再計測する | `ProductStageProjection` oracle、R0 native Stage、単一Host GPU binding、Rerun Spatial Viewer。Document writer 0 | Vector RectのStage編集幾何は`REUSE / LANDED`。accepted snapshotからRerun Spatial Viewer entity入力へ写す薄いconsumerを閉じる。固定bool、第二scene owner、第二deviceを作らない | same LayerId/revision/time、stale reject、固定bool 0、Document write 0、第二device 0。Rerun製品projectionは未接続であり、#470の固定path表示を代用しない |
| `M3-R1-TIMELINE-RECTANGLE` | 同じRectangleのClip区間をRN Timelineへ表示し、Undoで消えRedoで戻す | `WAIT(R1-GPU-BINDING)`。既存`R1-TIMELINE`だけをcurrent mainから再compileする | `timeline_projection.rs`、`ProductTimelineProjection`、native rust-skia Timeline。Document writer 0 | 既存Document→`TimelineBar { layer, start, end, band }`投影を`REUSE`し、RN componentへ渡す。Rerun entity pathから時間意味を逆生成しない | `cu110pt`系、same LayerId/revision、visible bound、resize／zoom read-only、Undo/Redo再投影 |
| `M3-R1-INSPECTOR-RECTANGLE` | 選択中Rectangleのidentityと値を、同じaccepted snapshotから既存RN Inspectorへ表示する | `COMPILE / EXISTING PARTIAL`。panel新設は禁止。initial snapshot固定からの更新とRect値表示のexact gapだけを閉じる | 既存RN Inspector、Host snapshot、primary selection、Document read projection。D2 mutationは後続の既存R2契約 | `primary_layer_id`／boundsの既存decoderとDocumentのRect／position値を`REUSE`。mock reducer、第二Inspector、汎用parameter frameworkを作らない | `cu110pih`／primary-selection consumer、same LayerId/revision、none/stale表示、write 0。現状はidentity/nameまででRect値は未投影 |
| `M3-R1-E2E-RECTANGLE` | Browser→D2→Stage／Timeline／Inspector→Undo→Redoを一つのRN製品artifactで完走する | `WAIT(R1-SHELL..INSPECTOR)`。実装Issueではなく既存`R1-E2E`統合受入 | 一つのRN product artifact。新しいsemantic ownerを持たない | 上記LANDED／consumerを統合し、既存deterministic sequenceを`REUSE` | same LayerId／revision列／journal、Undoで三面から消失、Redoで復帰、second owner 0、reopen前提を壊さない |
| `RN-B004-LIVE-PROP` | Rectangle作成時にStageを再生成せず、同じnative Stage instanceへpath表示を反映する | `WAIT`。#470がmainへ着地後、current codeでprop更新経路を再計測 | 既存Fabric component state。`MotoliiGpuView` lifecycle、macOS component、`RendererCore`を直列所有 | Fabric `updateProps`と既存setterを`REUSE`。第二bridge、event bus、Stageは作らない | native instance identityとframe continuityを維持したままfalse→trueが見える。remount、第二renderer、stale falseを負例にする |
| `DEV-RN-MAC-WARM-BUILD` | RN probeの二回目以降のmacOS buildを、正しいsource invalidationを保ったまま短くする | `COMPILE`。current mainでcold/warm計測、共有可能範囲、exact commandとtargetを閉じる | 開発tooling owner。Cargo target、Xcode DerivedData、RN codegen、Pods。製品M4 cacheとは別seat | Cargo `--target-dir`／`CARGO_TARGET_DIR`、Xcode `-derivedDataPath`、既存RN codegenとPodsの増分buildを`REUSE`。APFS copy-on-write複製は主routeにしない | 同一sourceの二回目buildでRust、Pods、Skiaの不要再compileが0。Rust／ObjC／TS変更は各担当層を必ず再buildし、現行Build IDが起動する。今回のrun-local観測はRust incremental 4.65秒、Xcode incremental 9秒、copy後のRust release再判定3分39秒 |
| `RN-CREATE-CIRCLE` | 既存Createから標準円を選び、同じStageへpath-based circleを表示する | `WAIT`。#470 landing後、B003のshared seatと実コードを再利用できるかcompile | probe内RN App stateとStage path translation。B003と同じ`App.tsx`／Fabric／`RendererCore` | Rerun line／point実装を先に検索し`REUSE`。公開Shape enumや第二rendererを作らない | Create操作前後の一画面差、閉じた円、単一Stage。Rectangleの見た目を円成功へ流用しない |
| `RN-PATH-OVERLAP` | 円と矩形を同じz=0平面へ置き、重なりとalpha／overlay順を知覚できる | `WAIT`。RectangleとCircleの各routeがmainへ着地後にcompile | Stage composition owner。単一GPU device／surface、overlay順 | 既存Rerun／Skia compositeを`REUSE`。新scene graph、第二texture owner、Document意味を作らない | 二形状の重なり領域と順序が一目で判別でき、単独表示にも戻せる。probe結果をPreview／Export完成へ繰り上げない |
| `RN-PATH-INSPECTOR-EDIT` | 既存InspectorからRectangleの既存parameterをD2経由で変え、同じStageへ反映する | `WAIT(M3-R1-E2E)`。新しい永続shape意味の判断待ちではない。既決VS-2／R2の一操作familyをcurrent codeから再選定する | Inspector typed intent、D2 single writer、Documentの既存Rect／transform parameter、同revision再投影 | 既存Inspector projection、SetProperty／position edit、gestureごと一Undoの契約を`REUSE`。B003 bool translationや汎用parameter frameworkへ接続しない | same LayerId／revision、1 gesture=1 Undo、cancel／stale／不正値write 0。VS-1表示完走前に先行発注しない |
| `README-RN-PROGRESS` | mainのREADMEからB003の現在地と次の一歩へ到達できる | `WAIT`。#470 landing後、そのmain事実だけでdocs-only Issueへcompile | docs routing owner。README、reviews index、decision indexは機械的shared seat | 既存README進捗節を`REUSE`。実装状態を別台帳へ複製しない | mainのBuild ID、製品状態、known limitと一致。Draft PRやbranchだけをmain進捗へ繰り上げない |
| `M4-K1B-PRODUCT-CACHE` | 同一評価の再利用により連続再生の再計算を減らし、cache欠落／破損時も同じ出力へ戻る | `WAIT`。[M4仕様](specs/M4-cache-and-analysis.md#実装順序と完了条件)とledgerのK1b dependencyが満たされ、一意な`DO`が出た時だけ再入場 | Host-private cache identity／store。ResourceLedger、source identity、artifact publication、invalidationと直列 | [M4採択地図](m4-known-implementation-adoption-map.md)を継承。`foyer-memory`等のprobeを製品ownerへ自動昇格せず、独自cache frameworkを作らない | 完全key、使用中handle保護、corrupt→miss、pixel一致、bounded budget。開発用DerivedData／Cargo cacheの成果をM4進捗へ数えない |

## 今開ける順序

1. `IN_PR`の#470はprobeとして着地または返却する。M3のRectangle本体やR1 nodeのdependencyにしない。
2. M3は[Stage Hero root](reviews/2026-08-11-m3-m5-stage-hero-projection-root-decision.md)から`M3-R1-STAGE-RECTANGLE`のexact targetと`R1-GPU-BINDING`残余をcompileする。Stage入力を固定後、file-disjointな`R1-TIMELINE`／`R1-INSPECTOR` consumerを並列化できる。新しいshape意味席は作らない。
3. `R1-E2E`は三consumerとshellがmainへ着地した後の統合受入であり、実装PR席として開かない。
4. それらと独立して`DEV-RN-MAC-WARM-BUILD`のread-only計測とclosed-order compileを行える。
5. #470着地後、probe側の`RN-B004-LIVE-PROP`と`RN-CREATE-CIRCLE`をcurrent mainから再選定する。同じFabric／Stage seatを同時発注しない。
6. `README-RN-PROGRESS`は#470のmain事実だけを記録するdocs-only seatとする。
7. M4製品cacheは既存ledgerと採択地図が開くまでIssue化しない。

## Issueを開く時の最小操作

1. rowをcurrent mainへ照合し、古いPRや候補branchをauthorityにしない。
2. 一つの利用者成果、semantic owner、shared seats、exact target、正負oracle、known limitsを閉じる。
3. 一般機構なら既知実装preflightを閉じる。候補なしを独自実装許可へ変換しない。
4. rowを`READY_TO_OPEN`へ更新する同じ判断でClosed contract Issueを開き、Issue URLを記録する。
5. 発注後は`OPEN`または`IN_PR`にし、同じshared seatの次Issueを止める。
6. main着地後は`LANDED`へ移し、current codeから次edgeを再選定する。
