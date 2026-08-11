# クリエイター翻訳機構・叩き台PR統合決定

日付: 2026-08-10
状態: **決定**
判断者: 利用者(owner)

## 決定

Motoliiは、個々の描画・編集技術を第一原理から作り直す製品ではなく、**Rerun Spatial Viewerをcreator向け映像制作へ翻訳する薄いwrapper**である。Document／D2、creator意味、identity／time／asset翻訳、admission、Preview／Export policyを持ち、Rerunのscene／view／query／camera／picking／rendererを作り直さない。

M3〜M5の開発では、一発で最終正答を作ることを施工条件にしない。まず一つの制作意図について、操作、型付き意味、既知実装、画または自動oracleまでの理論を実際に通す**叩き台**を作り、製品の現在地と既知の限界を明示したままmainへ現像する。main上の実物を次の修正入力にする。

```text
creator intent
  -> Path / Filter / Instance / Time等の型付き意味
  -> Skia / Rerun / wgpu / FFmpeg等の採択済み実装
  -> Stageの知覚可能なfeedback
  -> M4の連続再生・Preview / Export共通評価
```

## 良い塊をPRで運ぶ

- Rerunのように、責任と内部整合性を既に持つsubsystemは、細片へ再発明せず**良い塊**として採択・移植する。
- 一つのPRは、一つの利用者成果と一つの意味ownerへ閉じる。その成果を通すためなら、Rust、React Native、shader、fixture、test、docsを横断してよい。file数や施工step数を契約境界と取り違えない。
- PRはdiff、根拠、既知の限界、事後観測をまとめる**landing envelope**である。承認待ちの官僚的gateや、一発で完成を証明する容器にはしない。直接mainへ入れる既存許可も維持する。
- probe、製品source、main統合、通常製品route、製品完成の状態は引き続き分ける。叩き台であることは状態の繰り上げを許さない。

## parallelとconflict

共有seatを先に完全分割してから並列化するのではなく、Stage、Timeline、Browser、Inspector、Vism表現等の**利用者に見える縦slice**を並列に進め、統合担当が順にmainへ着地させる。

- Gitの機械的conflict、module登録、同じ索引・台帳・app rootへの追記は、失敗ではなく通常のintegration workとして統合担当が解消する。
- stable identity、Document意味、D2 single writer、単一GPU owner、公開／永続contractの競合はsemantic conflictであり、当該統合を止めてownerを一つに戻す。
- 同じ共有seatを複数laneが独立実装しない。共有seat自体は直列、そこへ接続する製品sliceは並列とする。
- 寸分違わない事前分割図、独自queue、broker、監督framework、placeholder APIを、conflict回避のために新設しない。

## M3・M4・M5への適用

- **M3**は完成し続ける制作面であり、Stageを主役に、Timelineと各panelから制作意図を画へ現像する。
- **M5**はRerun Spatial Viewerのcustom visualizer等へPath、Filter、Instanceを載せ、Stageの表現能力を増やす。独自spatial runtimeを増やさない。
- **M4**はM3とM5の間で同じ評価を時間上に連続させる。独立した主役UIを増やすためではない。
- Skia Timeline、Rerun Stage、Path morph、clipping filter、particle、glow、feedback、datamosh等のprobeは、個別機能一覧ではなく、この翻訳routeが成立するかを確かめる入力として扱う。

## 非目標

- 複数の意味ownerを持つ巨大PRを許可しない。
- test、review、実機gate、状態区分を省略またはgreenと偽装しない。これらはmain統合後も事実として観測する。
- probeのprivate表現を、検証なしに公開plugin契約やDocument schemaへ昇格させない。
- すべての変更へPRを義務化せず、PR approvalをmainのマージ条件へ戻さない。
- RerunやSkiaの製品意味をそのままMotoliiの製品意味として保存しない。Motoliiが所有するのは薄い翻訳、admission、作品意味、oracleである。

## 施工時の問い

新しいM3〜M5施工は、細かな作業列より先に次を答える。

1. 制作者は何を行い、Stageで何を知覚するか。
2. その意図をどの既存の型付き意味へ翻訳するか。
3. どの既知実装を良い塊として採択し、Motolii固有の薄い残余は何か。
4. 一つの意味ownerは誰か。機械的conflictを解消するintegration ownerは誰か。
5. 今回mainへ入る叩き台が通す理論、既知の限界、製品状態、次に観測できるedgeは何か。

この問いが閉じれば施工できる。将来の全panel、全Vism、全conflictを先に閉じることは開始条件にしない。

## 並列PR発注loop v0

全ての実装を一つの直列queueへ入れるのではなく、**閉じた利用者成果ごとに同じloopを並列起動**する。並列化するのはPRの施工であり、共有seatの意味決定とmainへの着地順はintegration ownerが直列に所有する。

```text
current mainから一つの利用者成果をcompile
  -> IssueでOUTCOME / OWNER / SHARED SEATS / ORACLE / KNOWN LIMITSを宣言
  -> current mainから独立branch / worktreeへ発注
  -> PRへ叩き台、実diff、製品状態、既知の限界を返す
  -> integration ownerがmechanical / semantic conflictを分類
  -> current mainへ着地
  -> main上で事後観測
  -> task起因redは同じoutcomeのfix-forward
  -> current mainから次edgeを再選定
```

同じshared seatを触る複数PRを同時発注しない。異なるseatへ接続するStage、Timeline、panel、Vism等は並列発注できる。feature branch同士のmergeや、未着地branchを次PRのbaseにする依存鎖は作らず、すべてcurrent mainを合流点にする。

PR作成やreview approvalをマージgateにはしない。integration ownerはPR本文と実diffからsemantic conflictの有無だけを着地前に判定し、Git上の機械的conflictは解消してmainへ入れる。validationはmain上の事後観測とし、当該PRが新たに生んだredは、同じowner／outcomeのfix-forwardをそのseatの新規発注より先に着地させる。既知redを無関係なPRへ帰属させない。

## risk rails

| risk | 最小rail | 停止／回復 |
|---|---|---|
| 同じ意味の二重発明 | `SEMANTIC OWNER / SHARED SEATS TOUCHED`をIssueとPRへ明記 | 同じseatの同時発注を止め、一ownerへ戻す |
| stale baseとbranch依存鎖 | 各branchをcurrent mainから作り、mainだけを合流点にする | 着地時にcurrent mainへ合わせ、別feature branch経由を棄却する |
| probeの製品昇格 | `PRODUCT STATE / KNOWN LIMITS`を返す | 状態を繰り上げず、次の製品edgeを再選定する |
| redの累積と責任消失 | main上で事後観測し、`FIX-FORWARD OWNER`をPRへ残す | task起因redを同じoutcomeへ戻して先に着地させる |
| integration ownerの詰まり | 一PRを一成果・一ownerへ閉じ、機械的conflict解消だけを集中させる | semantic conflictだけを返却し、全laneの再設計へ広げない |
| 大きすぎるPR | file数でなく利用者成果と意味ownerで境界を判定する | ownerまたは利用者成果が増えた地点で別Issueへ返す |

このrailは新しいscheduler、lock service、receipt database、merge queueを要求しない。GitHub Issue、branch、worktree、PR、mainと既存の事後観測だけを使う。

## PR席候補の入口

次にIssue化し得る利用者成果は[PR席候補一覧](../pr-seat-candidate-catalog.md)へ置く。この一覧は予約、queue、実装許可、第二implementation ledgerではない。候補はcurrent mainでclosed orderへ再compileし、`READY_TO_OPEN`になった一行だけを既存Closed contract Issueへ翻訳する。IssueまたはPRが開いた後の状態はGitHubとmainを正とし、同じshared seatへ次Issueを重ねない。
