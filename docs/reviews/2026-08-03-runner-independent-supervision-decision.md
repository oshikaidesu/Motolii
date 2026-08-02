# runner非依存の監督責任決定

状態: **決定**（main統合後発効）

日付: 2026-08-03

## 決定

Motoliiの監督は主担当Codexの責任であり、transport、order schema、receipt DB、固定model routeの責任ではない。
旧runnerを廃止しただけでなく、旧runnerを再生成する運用上の必須語彙も現行規約から外す。

次の旧機構はすべて歴史資料とし、新規作業の起動条件・必須field・採用資格へ使わない。

- `prepare / execute / inspect / cancel`とcanonical activation bundle
- `ROUTE_CONTRACT_VERSION / LOOP_PROFILE / RUNNER_SHA256`
- closed order、typed delta、compiled grain、checkpoint、独自receipt schema
- Grok→Spark→Opus等の全task共通固定直列route
- session resume/cancelを採用資格へ結び付ける状態機械

## 現行の責任配置

| owner | 責任 |
|---|---|
| ユーザー | outcome、許可する成果物・mutation・validation、STOP |
| 主担当Codex | 正本、base/cwd、worktree、scope、WIDE判定、対象file、fingerprint、diff、oracle、reviewer独立性、最終採否 |
| 薄いCLI harness | exact argv起動、生stdout/stderr、timeout、process-group回収、exit/signal、log hash |
| 実装担当 | Codexが渡した一契約境界の施工と指定試験。scopeや公開契約を増やさない |
| reviewer | fresh read-only sessionで実diffと試験を監査。修正、order再設計、採用を行わない |

## 発注フロー

ユーザーが「発注」を依頼動詞として明示した場合だけ外部実装を起動する。主担当Codexは次を行う。

1. decision index、正本、現行コードからauthority、owner、変更境界、非目標、oracleを閉じる
2. 意味、owner、原因、再利用、oracleのいずれかが未閉鎖なら`WIDE`として実装を起動せず、調査・縮小・ユーザー判断へ戻す
3. cleanな隔離worktreeを作り、開始baseとfingerprintを保存する
4. 必要最小のコード事実、対象path、変更境界、負例、確認commandを実装担当へ渡す
5. 実装後のdiff、fingerprint、試験をCodexが確認する
6. 実装担当と異なるfresh read-only reviewerへ、実diffと検証対象を渡す
7. Codexが正本、diff、試験、review結果を再照合して採用・差戻し・局所STOPを決める

これは固定stage数を要求しない。機械変更では外部preflightを省ける。共有境界や原因競合では実装前のread-only相談を
追加できる。reviewerを複数にするか、Grok／Opus／Codexのどれを使うかはtaskのriskとユーザー指定で決め、全作業へ
固定routeとして課さない。明示されたmodelは完全IDで起動し、利用不能時に別modelへ黙ってfallbackしない。

model選択は[履歴較正によるLLM役割選択](2026-08-03-history-calibrated-llm-role-selection-decision.md)に従い、taskの
判定対象へ合わせる。Claudeは意味・owner・契約閉鎖、Grokはscope・exact target・負例・実diffの列挙監査、Sparkは
閉じた機械施工を第一候補とする。同じtaskの設計へ深く関与したmodel familyを最終reviewerへ再利用せず、小taskでは
preflightを省く。この選択は固定stage、fallback順、receipt資格を新設しない。

## 採用と停止

採用に必要なのはrunner receiptでなく、Codexが直接確認した次の事実である。

- 正しいbase/cwdと開始前後fingerprint
- 許可scope内の実diff
- task固有test／fixture／非LLM oracle
- reviewerが実装担当と分離され、review中mutationが0であること
- P0/P1、scope違反、未決共有境界が残っていないこと

LLMの`ACCEPT`、test green、log hashはそれぞれ一部の証拠であり、単独では採用資格にならない。性能、安全性、永続形式等は
task固有のbench、negative test、schema fixtureで判定する。

ユーザーのSTOPは受信時点で対象CLI/processを終了し、新しい編集・試験・reviewを始めない。再開は残存diffとprocessを
Codexが再観測し、ユーザーの新しい指示から行う。cancel receiptやresume tokenを権限の代わりにしない。

## 証跡と歴史

[`run-observed-cli.py`](../../scripts/run-observed-cli.py)のlog directoryは実行観測であり、採用DBではない。必要な長期証跡は
Git commit、test結果、decision/ledger、外部review記録という既存物へ置く。新しい共通receipt schemaは作らない。

[旧Grok / Spark / Opus監督ループ](2026-07-25-opus-spark-grok-supervision-loop-decision.md)、
[runner派生物閉包](2026-07-25-supervised-runner-derived-target-closure.md)、過去receiptは事故分析と歴史比較のため残すが、
本文中の「現行」「必須」「正規入口」はすべて本決定で撤回される。継承するのは一契約境界、WIDE拒否、scope閉包、
独立read-only review、reviewer mutation拒否、Codex最終採否だけである。
