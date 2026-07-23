# 発注パイプライン並列化の比較案

状態: **比較中**

日付: 2026-07-23

## 1. 問題

現行のTerra + Grok発注運用は、1つの契約境界をclosed orderへ閉じ、隔離worktreeで実装し、独立検収後にCodexが統合する。この安全性は維持する。一方で、実装が終わってから次の調査・発注書作成・検収を始めると、モデルの実行時間ではなく工程間の待ち時間が全体速度を支配する。

ここで比較するのは、1件の発注範囲を広げる案ではない。**契約境界ごとのclosed orderを小さいまま保ち、独立性を証明できる工程だけを重ねる運用**である。

本書は運用比較であり、次を自動的に変更しない。

- 現行implementation ledgerの`DO / WAIT / DECIDE / ACTIVE / DONE`
- Selected U seriesの直列順と「現在選択中の1件」
- React製品資産、Rerun、Document、公開API、plugin契約、永続形式の停止線
- 1チケット=1コミット、1発注=1契約境界
- Terra実装、Grok read-only検収、Codex正本化・統合という役割分離
- ユーザーが依頼動詞として明示した時だけ実装発注を開始する条件

## 2. 目的と非目標

目的は次の3つである。

1. Terra実装中に、Codexが別境界のコード事実確認と次のclosed order候補を準備できるようにする
2. Grokの事前反証と完了diff検収を、実装指揮系統へ混ぜず別工程として重ねる
3. 独立した実装レーンが本当に存在する場合だけ、複数の隔離worktreeを安全に進められる判定を作る

非目標は次のとおり。

- モデル利用率を上げるためだけに`WAIT / DECIDE`を解除すること
- 同じ契約変更を複数モデルへ競作させ、良さそうな差分を選ぶこと
- 複数境界を共通helper、planner、公開APIへまとめること
- レビューや必須試験を後回しにした未検収差分の積み上げ
- commit数、経過時間、モデルの自己申告を品質の代理指標にすること

## 3. 並列化する対象を分ける

「並列発注」を一語で扱わず、工程と書き込み権限で分ける。

| 種別 | 内容 | repository write | 初期案 |
|---|---|---:|---|
| `PREFLIGHT` | 仕様・決定台帳・コード事実・既存helper・負例の確認 | なし | 台帳・specが並走を許す対象だけ実装中も並行可 |
| `ORDER DRAFT` | 次候補のclosed order草案とSTOP条件の準備 | order/evidence領域だけ | 対象が既に`DO`で、台帳・specが並走を許す時だけ並行可。承認・dispatchしない |
| `COUNTER` | Grokによる通常read-only反証。Fableはユーザー明示の大地図・全体レビューだけ | なし | 対象と役割が重ならなければ並行可 |
| `IMPLEMENT` | Terraが承認済みclosed orderを隔離worktreeで実装 | 対象worktreeだけ | 独立性gate合格時だけ複数候補を比較 |
| `VERIFY` | Grokが実diff・試験・scopeをread-only検収 | なし | 別実装やpreflightと並行可 |
| `INTEGRATE` | Codexが仕様照合、必須試験、主枝への採否を決定 | 統合対象だけ | 原則1件ずつ直列 |

工程の重なりは次を基本形とする。

```text
Lane A: Codex APPROVE -> Terra IMPLEMENT ------> Grok VERIFY -> Codex INTEGRATE
Lane B:                  Codex PREFLIGHT/ORDER -----------------------------> APPROVE待ち -> Terra IMPLEMENT
Lane C:                  Grok COUNTER -----------------------> disposition
```

Bでは前laneの`INTEGRATE`完了または破棄まで次の`IMPLEMENT`をdispatchしない。重ねるのはpreflight・許可済みorder draft・read-only counterだけであり、同時に未採否の実装diffを2件持たない。

工程種別による並行可否は、implementation ledger・spec・決定文書の直列指定に劣後する。Selected U series、現在のRerun監査並走禁止、GR-PV-4、React R0〜R6等は、repository writeが無い工程でも自動解除しない。

`ORDER DRAFT`の完成は発注許可ではない。対象authorityのpath+SHA-256をdraftへ埋め、dispatch時に`GR-D1`で依存先の統合、`BASE SHA`、粒`DO`、authority同一性、必須ラベルを機械照合する。不一致draftは手修正で延命せず失効させ、最新authorityから作り直す。`GR-D1 / GR-D2`がmainへ到達し負例が合格するまでBを発効しない。

## 4. 独立性gate案

2つの`IMPLEMENT`を同時に動かせるのは、両方が次を満たす場合だけとする案を比較対象にする。

1. **依存独立**: 一方の未統合diff、生成物、型、判断を他方が前提にしない
2. **正本独立**: 同じDocument意味、serde面、公開API、plugin契約、single writer、状態ownerを変更しない
3. **差分独立**: 実装allowlistが交差しない。Cargo workspace、lockfile、共通testkit、共通fixture、goldenは交差として扱う。仕様タスク表・implementation ledger・decision index等のcoordination docsはTerraのallowlistから外し、Codexが`INTEGRATE`で1件ずつ更新して同じ最終ticket commitへ含める
4. **審判独立**: 同じgolden、期待値、acceptance fixture、migration corpusを変更しない
5. **統合独立**: どちらかを単独でrevert、検収、commitできる
6. **停止独立**: 一方のSTOP/REJECTが他方の仕様再判断を要求しない
7. **資源独立**: 同じ実機、GPU排他測定、外部service、固定port、生成cacheを同時利用しない
8. **権限独立**: 各worktreeのallowlistと証跡directoryが別で、検収者は書き込まない

path非交差だけでは合格にしない。たとえばUI projectionとD2 commandが別crateでも、同じselection ownerやUndo意味を同時に決めるなら直列である。read-onlyな先例監査と隔離された実装も、台帳・specがその並走を許す場合だけ同時に進められる。

基準6はdispatch時点の予測だけで合格済みにしない。進行中laneがSTOPし、その原因が未決意味・公開契約・owner境界など共有authorityに触れる場合、そのauthorityを`AUTHORITIES`または`READS FROM`に持つ全進行laneを直ちに凍結する。Codexが影響範囲と再開条件を処分するまで、修正・検収・後続dispatchを進めない。

## 5. Lane manifest案

implementation ledgerの意味を複製せず、dispatch時点の運用情報だけを持つmanifestを候補とする。

| Field | 意味 |
|---|---|
| `LANE ID` | 一時的な運用識別子。仕様IDの代用にしない |
| `SPEC / TASK` | 正本の仕様IDと粒ID |
| `STATE AT DISPATCH` | ledger上の状態と確認commit |
| `BASE SHA` | 隔離worktreeの基準 |
| `STAGE` | `PREFLIGHT / ORDER / IMPLEMENT / VERIFY / INTEGRATE` |
| `ALLOWLIST` | 変更許可path |
| `AUTHORITIES` | 参照する仕様・決定・停止線 |
| `READS FROM` | 依存するmain到達済みtask |
| `CONFLICT SURFACES` | schema/API/state owner/fixture/lockfile/hardware等 |
| `EVIDENCE PATH` | lane固有directory。timeout後も残る発注・試験・検収証跡 |
| `INTEGRATION ORDER` | 独立でも統合順が必要なら明記 |
| `REVIEW BINDING` | task hash、BASE SHA、検収対象diff SHA-256 |
| `COUNTER HISTORY` | 同じ検収者が事前反証へ関与したかと観点 |

manifestは単一共有ファイルにせず、laneごとのorder/evidence directoryへ置く。`STATE AT DISPATCH`と`INTEGRATION ORDER`は記録であり、spec・実merge状態・implementation ledgerを上書きする正本ではない。`VERDICT: ACCEPT`本文にはtask hash、BASE SHA、Terraの検収対象実装diff SHA-256を引用させ、`INTEGRATE`時に機械照合する。Codexが`INTEGRATE`で加えるcoordination docs差分はTerra allowlistと検収対象diffに含めず、Codexが別途正本照合する。同じGrokがCOUNTERとVERIFYを担う場合は関与を記録し、VERIFYでは実diffに対する独立した反証観点を明示する。

新しい恒久formatや公開toolを先に作らず、最初は発注書とevidence内の表で運用を実証する。manifest parserやschedulerの実装は、手動運用で必要性と閉集合が確認されるまで非目標とする。ただしdispatch同一性とreview bindingはU0e-2ガードの機械境界を再利用し、散文確認へ戻さない。

## 6. 比較する運用案

### A. 完全直列を維持

1件を統合してから次件のpreflightを始める。最も単純だが、Terra/Grok実行中の待ち時間を回収できない。

### B. パイプライン化、実装は1件

`IMPLEMENT`は常に1件に制限し、その間に次候補の`PREFLIGHT / ORDER DRAFT / COUNTER`だけを進める。統合前の変更diffは1件なので、現行の安全性をほぼ維持しながら手待ちを減らせる。

Bの発効条件は、`GR-D1 / GR-D2`のmain到達と負例合格、verdictのtask/base/diff hash束縛、delegate toolingのlane固有temp/evidence・timeout・排他資源について並行再入性を確認することである。`WAIT / DECIDE / HUMAN / HARDWARE`粒のorder draftは作らず、ready draft在庫は1件までとする。authority hashが変わったdraftは自動失効する。

### C. 証明済み独立レーンだけ複数実装

Bを前提に、独立性gateを満たす場合だけ複数`IMPLEMENT`を許す。最も速い可能性がある一方、base陳腐化、共通集中点、検収待ち、統合WIPの増加が新しい失敗面になる。

Cを試すには、implementation ledgerが正当に同時`DO`を2件以上供給し、ユーザーが2件目も明示的に選択して発注した状態を必要とする。現行のSelected U series中はこの条件を満たさず、Cを試さない。台帳更新を暗黙に先取りせず、U系直列区間の完了またはユーザーによる正式な選択規則改訂を待つ。

初期候補は**上記guard完了後にBを通常形として実測し、Cは台帳が同時`DO`を供給した後、独立性が明白な2件でだけ試す**である。これは採択ではなくFableレビューへ渡す比較案である。

## 7. WIPと公平性の停止線案

並列化が未検収差分の在庫化にならないよう、次を候補とする。

- 前laneの`VERIFY`実行中、検収修正loop、`INTEGRATE`待ちを含め、採否が確定していない間は新しい`IMPLEMENT`をdispatchしない
- P0/P1、STOP、scope逸脱、base不一致が出たlaneを最優先で処分し、後続dispatchを止める
- dispatch時または事後にstale base/authority不一致を1件検出したら、全体をWIP=1へ自動縮退する
- 同じ契約境界を触る修正loopは新laneに数えず、元laneで完遂する
- 低リスク作業で枠を埋め、クリティカルパスのCodex判断・実機審判を飢餓状態にしない
- 人間審判・所有しないhardware・外部承認を、エージェント追加で短縮可能と見なさない
- WIP上限値は固定仕様へ焼かず、運用能力と失敗率に応じて見直す

Cの初回試行では、`stale-baseの事後検出`、`共有authority起因のlane横断STOP`、`証跡帰属の曖昧`のいずれかが1件でも発生した時点でCを中止し、BまたはAへ戻す。進行diffは隔離worktreeへ留め、最新mainに対して1件ずつ再preflight・再検収する。

## 8. 速度と品質の測定案

比較単位はcommit数やモデル時間でなく、closed orderが発注可能になってからmainへ安全に到達するまでとする。

| 指標 | 見たいこと | 誤用しないこと |
|---|---|---|
| lead time | order commit/dispatchから統合まで | 大きいtaskを有利にしない |
| wait time | preflight/実装/検収/統合の工程間待ち | モデルを常時稼働させる目標にしない |
| first-pass accept | 初回Grok検収でP0/P1=0か | 軽い検収へ誘導しない |
| rework count | STOP/REJECT後に有意な改善が何回必要か | 難しいtaskの回避に使わない |
| stale-base count | dispatch後にauthority/baseが陳腐化した回数 | rebaseで意味差を隠さない |
| escaped finding | 統合後に同scopeのP0/P1が見つかったか | テスト緑でゼロ扱いしない |
| Codex integration load | 同時laneが正本化・統合判断を圧迫したか | 自己申告だけで自動化しない |

order commit、dispatch、ACCEPT、mergeはgit/evidenceの機械timestampから採る。指標は新しいdispatchを正当化する目標にせず、試行中止・WIP縮退の判断だけに使う。first-pass acceptのために軽い検収へ寄せず、escaped findingは検出能力が落ちれば過小計数されることを記録する。粒ごとのrework countはlane固有evidenceへ残す。

BまたはCの試行は、比較対象となる直列taskと粒度・リスクが近くない限り「何倍速い」と結論しない。

## 9. 予想される失敗

- **見かけの独立**: pathは別でも同じ公開意味やownerを別々に発明する
- **baseの扇形分岐**: 全laneが古いmainから始まり、統合時に前提が崩れる
- **集中点衝突**: `Cargo.lock`、workspace manifest、spec表、decision index、共通testkitへ変更が集中する
- **レビューのボトルネック化**: Terraを増やしてもGrok検収とCodex採否が詰まり、未検収WIPだけ増える
- **easy-lane bias**: 並列化しやすい周辺taskがクリティカルパスより優先される
- **停止線の希釈**: 別laneが進んでいることを理由にSTOPしたlaneを迂回する
- **証跡混線**: どのbase/order/diff/testに対するACCEPTか分からなくなる
- **人間gateの偽装**: 目視・実機・未所有hardwareのWAITをagent reviewで解除する

## 10. Fable全体レビューgate

Claude Codeの`claude-fable-5`をread-onlyで使い、本書、AGENTS.md、Terra + Grok運用、implementation ledger、M2/M3の並列レーンと停止線、U0e-2発注ガードを横断監査する。

Fableには次を問う。

1. Bを通常形、Cを限定試行とする順序にP0/P1級の欠陥があるか
2. 独立性gateは意味上の競合を捕捉できるか。過剰に直列化または危険に許可する条件は何か
3. Codexの正本化・統合、Grok検収がボトルネックになった時のWIP停止線は十分か
4. `ORDER DRAFT`先行が古いauthorityを正本化する危険を、dispatch再確認で防げるか
5. lane manifestがledger/spec/order/evidenceを二重正本化しない最小形になっているか
6. Selected U series、React移管、Rerun、GR-PV/GR-UIの既存直列を誤って解除しないか
7. 速度測定が小粒化、軽い検収、周辺task優先というGoodhart化を招かないか
8. Cを試す前に必要な自動guard、失敗予算、rollback条件は何か

判定は`ACCEPT FOR LIMITED TRIAL`または`REVISE PARALLELIZATION`とし、P0/P1を列挙する。Fableは実装や仕様の決定者ではなく、出力はCodexが正本とコード事実へ照合して採否する。

## 11. 採択前の停止線

- FableレビューとCodex採否が完了するまで、`AGENTS.md`、delegate script、implementation ledgerのdispatch規則を変更しない
- 本書を根拠に複数Terra実装を開始しない
- `GR-D1 / GR-D2`、review binding、delegate並行再入性が未合格の間はBも開始しない
- review後も、ユーザーの明示的な発注なしに実装発注を開始しない
- P0/P1が残る場合は本文改訂と再レビューへ戻る

## 12. Fable初回レビューとCodex採否

Claude Codeの`claude-fable-5`をread-onlyで使い、本書、AGENTS.md、Terra + Grok運用、U0e-2発注ガード、implementation ledger、M2/M3仕様、React直接移管、Rerun強制動線を横断監査した。

| 回 | 判定 | 指摘 | Codex採否 |
|---|---|---|---|
| 初回 | `REVISE PARALLELIZATION` | P0=0 / P1=6 / P2=6。機械dispatch gate、coordination docs集中、既決直列への劣後、共有authority STOP、verdict hash束縛、C試行用同時`DO`不在 | P1/P2を全件採用。本書§3〜8・§11へ反映し再レビューへ戻す |
| 再回 | `ACCEPT FOR LIMITED TRIAL` | P0=0 / P1=0 / P2=3。次IMPLEMENT境界、Fable通常利用の限定、検収diff hash対象の精密化 | P2を全件採用。前lane採否確定、Fable用途限定、Terra実装diffとCodex coordination docsの照合分離を反映 |

初回P2のCOUNTER関与記録、機械timestamp、粒別rework、manifest分散、draft在庫上限と失効、delegate並行再入性も採用した。Fableの出力は助言であり、現行正本と照合した結果、U0e-2失敗原因・ledger直列・docs更新規則と一致するため採用した。

再回で限定試行の文書gateはP0/P1=0になった。ただし`GR-D1 / GR-D2`、review binding、delegate並行再入性のmain到達と負例合格は未充足であり、Bはまだ発効しない。現行台帳に同時`DO`が無いためCも試行しない。運用規則の正式採択と`AGENTS.md`等への反映は、これらの前提を満たす独立変更として扱う。
