# Motolii統一並列開始baseline決定

日付: 2026-08-09
状態: **決定 / candidate branchで収束済み / main統合と並列発注は未実施**

## 1. 決定

Motoliiの全体並列接続は、製品code、現行authority、直列核契約、UI配置逃げ道、仮コード調査、未commit設計資料、監督停止耐性を一つの開始履歴から参照できる状態へ収束してから始める。

ここでいう「統一」は、存在するbranchをすべてmainへ入れることではない。各成果を次の四状態へ一意に分類し、同じ開始線から検索・再検証できることをいう。

| 状態 | 意味 | 並列発注への効力 |
|---|---|---|
| `INTEGRATED` | current baselineのtreeと履歴に入り、現行authorityと矛盾しない | 次の契約をcompileする入力にできる |
| `CANDIDATE` | code／docsは存在するが、scope・oracle・review・採用のいずれかが未閉鎖 | 再実装せず検収する。次依存の成立根拠にはしない |
| `WAIT` | exact gapまたは外部gateが残る | gapと再入場条件が満たされるまで起動しない |
| `REJECTED` | 現行正本とoracleにより不採用 | branch名や投入工数から復活させない |

`READY-RECHECK`、probe、fixture、test green、外部review、candidate diff、main統合、通常製品route、製品完成を相互に繰り上げない。

## 2. 収束したsource

| source | 統一baselineでの処分 |
|---|---|
| local product main `9b2deac4` | 製品code baseとして`INTEGRATED`。RN product runtime seat、shell slots、Inspector initial snapshot、Stage GPU surface、initial Stage preview pixelsを含む |
| current authority `f800cb4f` | 現行docs／AGENTS／M3 RN + rust-skia + wgpu再基線として`INTEGRATED` |
| serial-core `debc149e`、`02867ab4` | Asset lifecycle、source/recipe identity、resource/artifact/job、mutation/invalidationの直列核4契約として`INTEGRATED`。runtime実装済みを意味しない |
| UI placement deferral `a60a1da5` | 操作意味とownerが閉じ、配置だけ未決のcontrolを止めないstaging surface決定として`INTEGRATED` |
| 2026-08-08 dirty docs 6件 | source SHA-256一致で`INTEGRATED`。引継ぎ文書は歴史／routingでありauthorityではない |
| external supervision review logs | run-local evidence。正本、receipt DB、採用資格にはしない |

統一candidateは`codex/supervision-ha-authority-20260809`で作り、元root、local main、既存worktreeを変更しない。最終main統合は別gateである。

## 3. 統合しないcandidate

### R2 spine `68546b8d`

local main `9b2deac4`上の5 commitで、Stage geometry read projection、pointer transport、transient evaluation time、primary selectionを含む。現行treeへ実在するが、全体baselineへは未採用である。

処分: **`CANDIDATE / MAIN NOT INTEGRATED`**。

- 再実装しない
- commit単位でowner、allowlist、oracle、現行authorityとの整合を再検証する
- `rn_product_host.rs`の共有union／matchへ集中した物理衝突を意味上の直列核と誤認しない
- 採用前にR0 acceptance未閉鎖との依存を再計測する

### N-OVERLAY dependency `ed9024fc`

R2 spine上でrust-skia依存を追加する1 commitであり、overlay renderer本体の製品接続ではない。

処分: **`CANDIDATE / DEPENDENCY ONLY / MAIN NOT INTEGRATED`**。

- dependency追加をoverlay完成と呼ばない
- R2 spineの採否と、現行`Cargo.lock`／wgpu closureを再照合する
- N-OVERLAYを待たずに成立するrender済みtexture blit／transient Command previewを止めない

## 4. 現行開始状態

M3のRN codeはlocal mainに存在する。しかし、`R0-HOST / R0-MAC-SEAT / R0-STAGE-LIFECYCLE / R0-ACCEPT`を責任別に再現し一つの通常RN artifactとしてacceptした記録はない。

したがって開始状態は次である。

```text
UI runtime selection: DONE
local-main RN code: PRESENT
R0 acceptance: NOT CLOSED
R2 spine: CANDIDATE
N-OVERLAY dependency: CANDIDATE
serial core four contracts: CLOSED / IMPLEMENTATION NOT STARTED
M4/M5 call-site sketches: OBSERVATION / NON-COMPILE
parallel implementation campaign: NOT STARTED
```

codeが存在するnodeは再実装せず、まず`REUSE / VERIFY_CANDIDATE / REMAP`する。仮コードの`???`とsurveyの`ABSENT`は、repo外asset、禁止済みroute、実在するが挿入不能なseamまで照合してからgapとする。

## 5. 並列開始gate

全体並列発注を開始できるのは次をすべて満たした後だけである。

1. 本baseline、直列核4契約、停止耐性契約がdecision indexとreviews indexから一意に到達できる
2. `./scripts/check-docs.sh`と`git diff --check`が通る
3. baselineに含む製品codeがlocal mainと同一であること、または差分が別ticketとして説明される
4. R2 spine／N-OVERLAYを`INTEGRATED`と誤記した箇所がない
5. 外部review用evidence envelopeがsource hash、range、literal queryの全hit inventoryを非LLM preflightで通る
6. 停止耐性のfailure injection oracleが、二重権威、偽死亡、真死亡、base drift、write-set衝突、reviewer mutation、user STOP、integration crashを覆う
7. freshな別family reviewerが実diffをread-onlyで監査し、reviewer mutation 0、P0/P1 0、未解決`EVIDENCE_GAP` 0になる
8. 主担当Codexがcurrent tree、diff、oracle、reviewを直接再照合し、main統合と最初のcampaignを別々に採否する

external reviewerの賛同だけでgateを閉じない。main統合済みでもcampaign発注済みとは扱わない。

## 6. 非目標

- 歴史branch、probe、candidateを一括mergeすること
- 未検収candidateを失わないためにmainへ退避すること
- 新しい長期status DB、receipt DB、queue serviceを作ること
- 一つの24時間sessionを開始線にすること
- baseline作成をM3、M4、M5またはMotolii完成と呼ぶこと

このbaselineは「どこから始めるか」を一つにする。何を採用するか、どの順で実装するか、製品が完成したかは各契約とoracleが別に決める。
