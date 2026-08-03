# renewal branch reconciliation handoff

状態: **引継ぎsnapshot**（branch削除・統合許可・製品決定ではない）

日付: 2026-08-03

## 2026-08-04 PR #441 CI再検証による訂正

本snapshotが第一製品grain候補とした`CU-201P-TRIM`の`DO`は撤回する。先行`CU-201P-MOVE`のnative pointer接続が
`CU-0B04NA`のlifecycle-only raw-input guardに拒否され、記録済みの`workspace check green`は成立しなかった。
違反実装grain `43727b77`はPR #441のCI修復で局所revertし、MOVEを`EXTERNAL_GATE_PENDING`、TRIMを`WAIT_TARGET`へ戻す。
exact private pointer adapter authority成立前に、以下のCU-201P-TRIM開始packetを実行しない。独立候補`GAP-25`の状態は本訂正で変更しない。

## 目的

監督規約のrenewal後に、旧sessionやbranch名をauthorityとして自動再開せず、現行local `refs/heads/main`、正本、decision index、
implementation ledgerから全担当が同じ開始点へ戻れるようにする。branchとworktreeは削除せず、Git objectとして保全する。

## 開始authority

- local `refs/heads/main`: `3013732cec396610f0dce0fa07d183b5adc3cc49`
- main worktree: `/private/tmp/motolii-main-agentex-integration-20260802`、clean
- main status: `origin/main`より152 commit ahead。`origin/main`は本snapshotのauthorityや同期指示ではない
- authority順: local mainのGit object → `docs/README.md` → `docs/decision-index.md` → 対象spec／decision →
  `docs/implementation-ledger.md` → 現行code／oracle
- 旧会話、外部LLM session、branch名、worktreeの存在、過去receiptは再開許可ではない

SHA、ahead数、branch数、worktree数はtime-specificであり、renewal後の各開始時に再計測する。

## inventory snapshot

handoff branch作成前の2026-08-03計測ではlocal headは533本で、mainの祖先またはmain自身が362本、mainの祖先でないheadが171本
だった。worktreeは218件で、Gitがprunableと報告しないものが95件、prunableが123件だった。この数は作業量、未統合成果、DO数を
表さない。

分類は次で固定する。

| 分類 | 判定 | renewal後の扱い |
|---|---|---|
| `MAIN_CONTAINED` | branchがmain祖先、または`git cherry main <branch>`の`+`が0 | 再merge・再実行しない。branchは保全可 |
| `CONTENT_SUPERSEDED` | unique commitはあるが、現行mainの正本／ledgerが同じoutcomeを後続状態で閉じている | wholesale mergeしない。必要時だけclaim単位で現行mainへ再照合する |
| `HISTORICAL_UNADJUDICATED` | main非祖先で、現行DO／owner／oracleとの対応を本snapshotで裁定していない | 保存のみ。作業再開、cherry-pick、統合候補とみなさない |
| `DIRTY_OR_ACTIVE_WORKTREE` | dirty、実process使用中、または利用者作業の可能性がある | 変更・clean・removeしない |

`git cherry`はpatch-equivalenceの補助であり、製品意味の同一性を単独証明しない。`CONTENT_SUPERSEDED`のclaim回収が必要なら、
blind evidence envelopeへ現行mainとbranchのexact diff、source hash、query scope内の全hit inventoryを入れ、未収録候補は
`EVIDENCE_GAP`へ戻す。

## 直近branchの処分

| branch | Git事実 | 処分 |
|---|---|---|
| `codex/claude-effort-routing-20260803` | mainと同じ`3013732c` | `MAIN_CONTAINED`。renewal監督規約として発効済み |
| `codex/p06-c1-mac-rfd-gate-20260803` | main比behind 6 / ahead 1、`git cherry` unique 0 | `MAIN_CONTAINED`。P06-C1-MAC固定Mac gateはmainでDONE、P06-C1全体は未完了 |
| `codex/known-implementation-search-order-20260803` | behind 7 / ahead 1、unique 0 | `MAIN_CONTAINED`。既知実装優先規律はmainで発効済み |
| `codex/p12-c1-document-lifecycle-adoption-20260803` | behind 8 / ahead 1、unique 0 | `MAIN_CONTAINED`。意味決定はmain、実装は`SPEC_ONLY / WAIT` |
| `codex/known-implementation-preflight-20260803` | behind 22 / ahead 1、unique 0 | `MAIN_CONTAINED`。独自一般機構のfail-closeはmainで発効済み |
| `agentex/retire-supervised-runner-20260802` | behind 73 / ahead 1、unique 0 | `MAIN_CONTAINED`。旧runnerを再起動しない |
| `agentex/cu-201tc-field-validation-20260802` | behind 73 / ahead 2、unique 0 | `MAIN_CONTAINED`。CU-201T-CとoracleはmainでDONE |
| `codex/m3-supervise-20260803` | behind 19 / ahead 2、unique 2 | `CONTENT_SUPERSEDED`。旧P06 probe途中状態で、mainは固定Mac gateまで後続閉鎖済み。wholesale mergeしない |
| `codex/m3-cu-201n-s-20260803` | behind 64 / ahead 4、unique 3 | `CONTENT_SUPERSEDED`。mainはCU-201N-S、P03-C1、MOVE、TRIM-SをDONE、TRIMをDOへ進めた。旧WAIT_TARGETへ戻さない |
| `codex/m3-local-alpha-20260801` | behind 75 / ahead 34、unique 33 | `HISTORICAL_UNADJUDICATED`。複数ownerの旧集約branchでありwholesale merge／自動再開しない |
| 旧canonical runner／receipt／profile／route branch群 | 現行mainのrunner廃止・tombstone決定より前または撤回済み | 歴史証跡のみ。activation、receipt、model列を復活させない |
| その他main非祖先head | 本snapshotで個別意味裁定なし | `HISTORICAL_UNADJUDICATED`。削除せず、現行粒から必要になった時だけclaim単位で回収する |

## renewal後の開始候補

現行ledgerの実作業候補は二つで、同じtaskへ束ねない。

### 1. 製品前進: CU-201P-TRIM（2026-08-04訂正により実行禁止）

- `AUTHORITY`: `docs/implementation-ledger.md` CU-201P-TRIM、M3 U3b、CU-201P-TRIM known-semantics adoption
- `CURRENT STATE`: `WAIT_TARGET`。MOVE-S、TRIM-S、trim command／Writer／journal／Undoの意味はDONEだが、製品MOVEは`EXTERNAL_GATE_PENDING`
- `INTERNAL TARGET`: 既存`ProductApp`のTimeline projection／pointer経路、public `TimelineProjection::hit_test`、既存`prepare_trim_clip_in/out`
- `GAP`: `ProductTimelineHit`は現行型ではなく、DOが既存`ProductApp`内だけに許したprivate refinementの新設名。public hitを変更しない
- `OWNER / WRITE ROUTE`: Product sessionのtransient gesture → release時に既存D2 trim commandを一回commit
- `PRIMARY ORACLE`: drag中write 0、release 1 Undo、same-value／no-jump／cancel／stale／invalid 0、public hit/APIとDocument意味不変
- `NON-GOALS`: snap、slip／slide／roll／ripple、multi-select、generic gesture、public API、Document／journal schema変更
- `STOP`: 既存projection／pointer target、derived height、selection写像、trim prepareのどれかが現行mainで不一致なら実装せず再調査する

これはrenewal後の第一製品grain候補であり、旧`codex/m3-cu-201n-s-20260803`をresumeしない。fresh branch、fresh session、
mainから作ったblind evidence envelopeで開始する。

### 2. 独立guard修復: GAP-25

- `AUTHORITY`: M2 D1i-4、decision indexのsemantic oracle gate、implementation ledger GAP-25
- `CURRENT STATE`: `DO / CHECK-PATH`。semantic oracle意味は変更せず、gate script／CI route自体の自己保護だけが残る
- `INTERNAL TARGET`: 現行`.github/CODEOWNERS`、`.github/workflows/ci.yml`のD1i-4 lane、`scripts/check-golden-update-policy.sh`、
  `crates/motolii-testkit/tests/golden_update_policy.rs`
- `GAP`: CODEOWNERSは自身、golden、golden policy、cpu reference、toleranceを保護するが、D1i-4 gate scriptとそれを起動するCI workflowを
  owner対象へ列挙していない。exact ownerは既存`@oshikaidesu`で、ruleset発効履歴はCODEOWNERS本文にある
- `NON-GOALS`: oracle値変更、golden更新、regenerate迂回、harness／runtime配線の凍結、CU-201P-TRIMとの同一commit化
- `STOP`: CODEOWNERS追加だけでは自己保護の回帰oracleが閉じないため、既存test owner内でscript／workflow両pathの欠落を拒否するfixtureを
  一契約に閉じられなければ施工せず、接続票を更新する

GAP-25はM3製品grainと別ownerの候補である。並行可否はrenewal後にpath／oracle非重複を再計測して決め、branchの古さから自動起動しない。

## renewal開始手順

1. local mainのbranch、HEAD、status、worktree listを再計測する
2. ledgerの`DO`と`DO / CHECK-PATH`だけを列挙し、WAIT／SPEC_ONLY／歴史branchを除外する
3. 一契約を選び、authority、target、owner、write route、gap、oracle、non-goals、STOPを現行mainへ再照合する
4. exact原文、source/range/hash、query scope内の全hit inventoryをblind evidence envelopeへ機械連結する
5. 未収録hitは`EVIDENCE_GAP`でfresh waveへ追加し、外部LLMへ自由repo探索を許可しない
6. fresh branch／sessionで一粒だけ開始し、完了・STOP・境界変更で閉じる

## 非目標

- branch、worktree、Git object、raw logを削除またはcleanすること
- 171本のmain非祖先headを一括でmerge、rebase、cherry-pick、採否すること
- origin/mainへreset、push、同期すること
- old session、receipt、branch名から作業を自動復活させること
- CU-201P-TRIMとGAP-25を一つのrenewal taskへ束ねること
- P12-C1、P06-C1残余、CU-201P残余WAIT_TARGET、M4/M5 runtimeをDOへ昇格すること
