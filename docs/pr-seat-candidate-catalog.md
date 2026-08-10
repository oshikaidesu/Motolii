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

## 現在の候補

| seat | 利用者成果 | 状態 / 再入場 | semantic owner / shared seats | 既知routeと薄い残余 | primary oracle / known limits |
|---|---|---|---|---|---|
| `RN-B003-RECTANGLE` | 既存BrowserのCreateから矩形を選び、既存Stageへpath rectangleを表示する | `IN_PR / SCOPE DELTA` [Issue #469](https://github.com/oshikaidesu/Motolii/issues/469) / [Draft PR #470](https://github.com/oshikaidesu/Motolii/pull/470)。新Issueを作らない。IssueのB002合成維持と利用者指示による判別用単独表示の差を着地前にIssue／PRへ同期する | probe内RN App state。`App.tsx`、Fabric component、C ABI、`RendererCore` | 既存RN shellとRerun `LineDrawableBuilder`を`REUSE`。Motolii残余はCreate操作、bool translation、判別用fixture | 実アプリでpath単独表示、targeted Jest、Rust test、Xcode build。既存rendererへのlive bool反映は未成立でkey remountを使用。probeであり製品shape schemaではない |
| `RN-B004-LIVE-PROP` | Rectangle作成時にStageを再生成せず、同じnative Stage instanceへpath表示を反映する | `WAIT`。#470がmainへ着地後、current codeでprop更新経路を再計測 | 既存Fabric component state。`MotoliiGpuView` lifecycle、macOS component、`RendererCore`を直列所有 | Fabric `updateProps`と既存setterを`REUSE`。第二bridge、event bus、Stageは作らない | native instance identityとframe continuityを維持したままfalse→trueが見える。remount、第二renderer、stale falseを負例にする |
| `DEV-RN-MAC-WARM-BUILD` | RN probeの二回目以降のmacOS buildを、正しいsource invalidationを保ったまま短くする | `COMPILE`。current mainでcold/warm計測、共有可能範囲、exact commandとtargetを閉じる | 開発tooling owner。Cargo target、Xcode DerivedData、RN codegen、Pods。製品M4 cacheとは別seat | Cargo `--target-dir`／`CARGO_TARGET_DIR`、Xcode `-derivedDataPath`、既存RN codegenとPodsの増分buildを`REUSE`。APFS copy-on-write複製は主routeにしない | 同一sourceの二回目buildでRust、Pods、Skiaの不要再compileが0。Rust／ObjC／TS変更は各担当層を必ず再buildし、現行Build IDが起動する。今回のrun-local観測はRust incremental 4.65秒、Xcode incremental 9秒、copy後のRust release再判定3分39秒 |
| `RN-CREATE-CIRCLE` | 既存Createから標準円を選び、同じStageへpath-based circleを表示する | `WAIT`。#470 landing後、B003のshared seatと実コードを再利用できるかcompile | probe内RN App stateとStage path translation。B003と同じ`App.tsx`／Fabric／`RendererCore` | Rerun line／point実装を先に検索し`REUSE`。公開Shape enumや第二rendererを作らない | Create操作前後の一画面差、閉じた円、単一Stage。Rectangleの見た目を円成功へ流用しない |
| `RN-PATH-OVERLAP` | 円と矩形を同じz=0平面へ置き、重なりとalpha／overlay順を知覚できる | `WAIT`。RectangleとCircleの各routeがmainへ着地後にcompile | Stage composition owner。単一GPU device／surface、overlay順 | 既存Rerun／Skia compositeを`REUSE`。新scene graph、第二texture owner、Document意味を作らない | 二形状の重なり領域と順序が一目で判別でき、単独表示にも戻せる。probe結果をPreview／Export完成へ繰り上げない |
| `RN-PATH-INSPECTOR-EDIT` | 既存Inspectorから矩形の位置または大きさを変え、同じStageへ即時反映する | `OBSERVE`。永続shape意味、probe-local transient、Inspector ownerのどこへ置くか未閉鎖 | Inspector control、path parameter、Stage feedback。Document writerへ触れる場合は別契約 | 既存Inspector projectionとB003 translationを検索して採択する。汎用parameter frameworkを先に作らない | 一つのparameter変更だけが一つの矩形へ反映し、cancel／不正値は画とwriterを変えない。Undo／永続化は未決 |
| `README-RN-PROGRESS` | mainのREADMEからB003の現在地と次の一歩へ到達できる | `WAIT`。#470 landing後、そのmain事実だけでdocs-only Issueへcompile | docs routing owner。README、reviews index、decision indexは機械的shared seat | 既存README進捗節を`REUSE`。実装状態を別台帳へ複製しない | mainのBuild ID、製品状態、known limitと一致。Draft PRやbranchだけをmain進捗へ繰り上げない |
| `M4-K1B-PRODUCT-CACHE` | 同一評価の再利用により連続再生の再計算を減らし、cache欠落／破損時も同じ出力へ戻る | `WAIT`。[M4仕様](specs/M4-cache-and-analysis.md#実装順序と完了条件)とledgerのK1b dependencyが満たされ、一意な`DO`が出た時だけ再入場 | Host-private cache identity／store。ResourceLedger、source identity、artifact publication、invalidationと直列 | [M4採択地図](m4-known-implementation-adoption-map.md)を継承。`foyer-memory`等のprobeを製品ownerへ自動昇格せず、独自cache frameworkを作らない | 完全key、使用中handle保護、corrupt→miss、pixel一致、bounded budget。開発用DerivedData／Cargo cacheの成果をM4進捗へ数えない |

## 今開ける順序

1. `IN_PR`の#470を着地または返却し、同じRN shared seatを解放する。
2. それと独立して`DEV-RN-MAC-WARM-BUILD`のread-only計測とclosed-order compileを行える。
3. #470着地後、`RN-B004-LIVE-PROP`と`RN-CREATE-CIRCLE`をcurrent mainから再選定する。同じFabric／Stage seatを同時発注しない。
4. `README-RN-PROGRESS`は#470のmain事実だけを記録するdocs-only seatとする。
5. M4製品cacheは既存ledgerと採択地図が開くまでIssue化しない。

## Issueを開く時の最小操作

1. rowをcurrent mainへ照合し、古いPRや候補branchをauthorityにしない。
2. 一つの利用者成果、semantic owner、shared seats、exact target、正負oracle、known limitsを閉じる。
3. 一般機構なら既知実装preflightを閉じる。候補なしを独自実装許可へ変換しない。
4. rowを`READY_TO_OPEN`へ更新する同じ判断でClosed contract Issueを開き、Issue URLを記録する。
5. 発注後は`OPEN`または`IN_PR`にし、同じshared seatの次Issueを止める。
6. main着地後は`LANDED`へ移し、current codeから次edgeを再選定する。
