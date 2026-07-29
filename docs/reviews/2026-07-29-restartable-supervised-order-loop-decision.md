# 再開可能な監督発注loop

作成日: 2026-07-29

状態: **決定／GR-D4実装済み**

対象: `scripts/delegate-cursor-supervised.sh`

## 1. 問題

従来の`prepare`は、Opus 5に機械fieldと施工本文を同時に書かせていた。このため、同じ契約を
Opus、Codex、Spark、Grokが自由文から再解釈し、order形式不備と設計STOP、Grok利用不能と
実装REJECTが同じ失敗へ潰れていた。

2026-07-29のM5並列loopでは、三order全てが最初のCodex precheckで形式不備となり、手修正後も
RCA2はGrok markerなし、RCB2/RCC2は`REJECT`となった。さらに現行code監査で、Opus draftによる
自己承認行混入、prepare中のBash mutation、失敗prepareによる既存order上書き、REJECT後の
同一diff再検収口を確認した。

## 2. 単一order artifact

機械manifestを別fileへ分けない。最終order file内にCodex所有のblockを一つ置き、既存の
order SHA、attempt copy、checkpoint、integrity検査で本文と一緒に保護する。

```text
<!-- ORDER MACHINE BLOCK BEGIN -->
GRAIN: ...
BASE_REF: ...
BASE_SHA: ...
DEPENDENCY: ...
AUTHORITY: ... SHA256:...
ALLOWED_FILE: ...
LOOP_PROFILE: opus-spark-grok
ORDER_MANAGER_MODEL: claude-opus-5
IMPLEMENTER_MODEL: gpt-5.3-codex-spark
REVIEW_MODEL: cursor-grok-4.5-high
TASK_SHA256: ...
<!-- ORDER MACHINE BLOCK END -->
```

`manifest` modeがbase、authority hash、ledger stateを機械照合してblockを作る。Opusは施工本文だけを
書き、machine key、delimiter、`CODEX PRECHECK`を出力できない。`CODEX PRECHECK: APPROVED`はblock外へ
主担当Codexが一度だけ追記する。

正規手順:

1. `manifest <worktree> <order> <task> <grain> <base-ref> --dependency ... --authority ... --allowed-file ...`
2. `prepare <worktree> <order> <task>`
3. Codexが本文を審査し、block外へ`CODEX PRECHECK: APPROVED`を追記
4. `execute <worktree> <order> <task>`
5. `REVIEW_UNAVAILABLE`だけ`inspect <worktree> <order> <task>`で再開

mode名`prepare / execute / inspect`、既存argv順、model ID、marker文法、evidence directory名、
既存exit code 2〜8は変更しない。`manifest`、exit 9〜11、`OUTCOME`は追加だけである。

## 3. 親process所有の結果分類

model本文の文字列から推測せず、runner親processが次を一度だけ出力し、attemptの
`stage-result.txt`へ記録する。

| outcome | 意味 | 戻り先 |
|---|---|---|
| `ORDER_INVALID` | manifest、hash、allowlist、machine block、承認形式の不備 | Codexが形式だけ直し、Sparkを起動しない |
| `DESIGN_STOP` | Opusが`ORDER: STOP`を返した | Codexが正本／未決を解消し、新しいorderへ戻す |
| `IMPLEMENTATION_FAILED` / `IMPLEMENTATION_INVALID` | Spark process失敗またはcommit等の施工違反 | Codexが原因を裁定し、新grain |
| `IMPLEMENTATION_REJECT` | Grokが有効な`VERDICT: REJECT`を返した | terminal。差分不採用、新grain |
| `REVIEW_UNAVAILABLE` | Grok stdoutが0 byteのprocess級利用不能 | 同一checkpointを`inspect`で再開可 |
| `REVIEW_INVALID` | stdoutが1 byte以上あるがmarker文法不成立 | terminal。再検収しない |
| `COMPLETE` | Grok `ACCEPT`、scope／integrity維持 | Codex最終照合へ |

Grok timeoutでもstdoutが1 byte以上残れば`REVIEW_INVALID`とする。availability再開は初回`execute`を含め
最大3回で、上限到達時はcheckpointを無効化しterminal outcomeを残す。別model fallbackはしない。

## 4. terminalと再開

- `REVIEW_UNAVAILABLE`だけ、order SHA、task hash、base、HEAD、worktree fingerprintが一致する
  checkpointを再発行する。`inspect`はSparkを再起動しない。
- `REJECT`、`REVIEW_INVALID`、`ACCEPT`はterminal outcomeを残してcheckpointを無効化する。
  同一order／diffへの再投票を許さない。
- order、worktree、authority、scopeが変わればcheckpointを使わず、新ID／新order／新証跡へ戻す。
- 生文字列の存在だけで意味違反を判定するoracleをrunnerへ追加しない。正負例は変更節、field構造、
  裁定値、fixture意味で検査する。

## 5. prepareの非破壊性

- Opus起動前後のworktree fingerprintを比較し、Bash経由mutationをfail closedする。
- Opus draftにmachine key、delimiter、PRECHECKがあれば`ORDER_INVALID`。
- `ORDER: STOP`、model利用不能、不正draftでは既存manifest/orderを上書きしない。
- READY本文だけを同一filesystemの一時fileへ組み立て、検証後にatomic renameする。

### 5.1 command oracleの親process所有

M5 fragment配置grainで、Opusが行番号、`grep` count、token presence、STOP時の`git checkout`を
orderへ発明し、意味が閉じていてもPRECHECKを反復停止させた。実行commandはtask／authorityが名指す
既存script／testと`git diff --check`だけに限定する。独自`grep`／`awk`／`sed`／`wc`／`find`、
行番号、token presence、件数oracle、checkout/reset/clean/deleteをOpus proseへ置かない。
scope、fingerprint、authority、manifest、marker integrityはrunner親processが検証する。
追加oracleが無いと安全に閉じないgrainは、Opusがoracleを即席実装せず`ORDER: STOP`へ戻す。

## 6. 非目標と残余

- React 8ラベル、Rerun 6ラベルの位置／順序を変更しない。
- 別manifest file、第二のhash体系、汎用workflow engine、background retry serviceを作らない。
- Opus、Spark、Grokの判断をrunnerが代行しない。
- danger-full-access modelからworktree外evidenceを暗号学的に隔離したとは主張しない。
- 同じ阻害要因が3回続き有意な改善が無ければ、親Codexがloopを停止してユーザーへ戻す。

## 7. 完了条件

- manifest外machine key、Opus自己承認、prepare mutation、失敗時order上書きを拒否する。
- empty reviewだけがSparkなしでresumeできる。
- REJECT／invalid review／ACCEPT後は再`inspect`できない。
- availabilityは3回でterminalになる。
- reviewer mutation、ledger state、dependency、authority、allowlist、model routingの既存負例が維持される。
- prepare promptが独自command oracleとdiff復元commandを禁止する。
- `scripts/test-delegate-cursor-supervised.sh`、`scripts/check-docs.sh`、`git diff --check`が通る。

## 8. 助言の処分

2026-07-29、主担当Codexセッション`019faae0-2508-7812-88cf-d6ad25973d38`からOpus 5とFable 5を
read-onlyで呼んだ。両者の出力はauthorityにせず、現行runner、専用test、M5生証跡へ再照合した。

- Opusのorder内machine block、prepare非破壊、parent-owned outcome、REJECT terminal化を採用した。
- Fableのstdout 0 byteだけをavailabilityとする条件、3回上限、PRECHECK block外維持、
  §10.4同時改訂を採用した。
- 別manifest file、既存exit code再割当、runnerへの新しい生文字列oracle、nonceの偽造不能公約は採用しない。
