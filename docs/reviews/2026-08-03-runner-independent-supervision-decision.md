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

## 外部LLM起動の共通前提

発注、相談、調査、検収、診断、session再開を問わず、外部LLMは[薄いCLI harness決定](2026-08-03-thin-observed-cli-harness-decision.md)の
途中stream条件を満たす時だけ起動する。provider-nativeの構造化途中eventを有効化し、生streamを保存しながら主担当Codexが
実行中に読めることを起動前に確認する。heartbeatとwall timeoutはprocess管理の補助であり、途中event、進捗、最終回答位置の
観測を代替しない。条件を閉じられないprovider／versionは、別modelへfallbackせず局所的に利用不能とする。

## 発注フロー

ユーザーが「発注」を依頼動詞として明示した場合だけ外部実装を起動する。主担当Codexは次を行う。

1. decision index、正本、現行コードからauthority、owner、変更境界、非目標、oracleを閉じる
2. 意味、owner、原因、再利用、oracleのいずれかが未閉鎖なら`WIDE`として実装を起動せず、調査・縮小・ユーザー判断へ戻す
3. cleanな隔離worktreeを作り、開始baseとfingerprintを保存する
4. 必要最小のコード事実、対象path、変更境界、負例、確認commandを実装担当へ渡す
5. capsule、diff、oracle、許可snippet、想定tool turnから動的context／token予算を記録し、超過する粒を契約境界またはfinding群で分割する
6. 実装後のdiff、fingerprint、試験をCodexが確認する
7. 実装担当と異なるfresh read-only reviewerへ、視野幅から算出したeffortと実diff／検証対象を渡す
8. Codexが正本、diff、試験、review結果を再照合して採用・差戻し・粒分割・局所STOPを決める

これは固定stage数を要求しない。機械変更では外部preflightを省ける。共有境界や原因競合では実装前のread-only相談を
追加できる。reviewerを複数にするか、Grok／Opus／Codexのどれを使うかはtaskのriskとユーザー指定で決め、全作業へ
固定routeとして課さない。明示されたmodelは完全IDで起動し、利用不能時に別modelへ黙ってfallbackしない。

modelとsessionはharnessでなく総監督`gpt-5.6-sol`が[履歴較正によるLLM役割選択](2026-08-03-history-calibrated-llm-role-selection-decision.md)に
従って選ぶ。Solはauthority衝突、粒分割／STOP、複数契約統合、最終採否／main統合を保持する。閉じた粒のpreflight、次手選定、
bounded施工、同一境界finding修正、diff／test照合はfreshなLuna Maxへ委譲できる。一sessionは一契約境界または同じoutcome、
owner、scope、oracleの短いwaveだけを扱い、終了後の会話履歴を
project memoryへしない。長期状態はGit、正本、decision/ledger、raw logが所有する。

Lunaは`model_reasoning_effort=max`を標準とし、単純施工・修正はfreshなSparkも選べる。maxでもcapsule、read set、一契約境界を
広げず、この担当範囲拡張に再benchmarkを要求しない。Claudeはsemanticな反対側、Grokはscope・exact target・負例・実diffの
列挙監査へ使う。同じtaskの設計・施工へ深く関与した
model familyを独立最終reviewerへ再利用しない。Spark、Luna、Solは同じOpenAI familyなので相互の独立検収を兼ねない。この選択は
固定stage、fallback順、receipt資格を新設しない。

modelの利用不能時はCodexが同じbase、scope、allowlist、oracleを再確認し、CLIで完全IDを確認できたmodelをfresh sessionで
明示選択する。失敗したsessionを別modelへ引き継がず、選択変更と理由をlogへ残す。これは黙ったfallbackや固定fallback順ではない。

Claude系のeffortは[履歴較正によるLLM役割選択](2026-08-03-history-calibrated-llm-role-selection-decision.md)の
`CLOSED / ADJACENT / WIDE / CONFLICTING`から算出する。高effortをread scope拡大の許可にせず、Luna Maxも同じcapsule／read set／
一契約境界へ閉じる。複数file／資料が同じ契約結論を補強するだけなら`WIDE`へ上げず、未収束のauthority主張、意味owner、
writer、原因、reuseの競合を広域化の根拠にする。hazardはeffortでなく負例、非LLM oracle、reviewer構成を強める。
予算超過はmodel失敗へ押し込まず、施工またはreview grainの分割信号として扱う。`TOOL_TURN_BUDGET`は予定するtool-result cycleと
最終回答として置き、file数やtool call数から機械算出しない。hard turn capは起動するexact Claude Code binaryのhelpに存在する時だけ
使い、2026-08-03に確認した2.1.216／2.1.220には無かった。完了eventのnatural turnをprovider-native streamで観測する。

外部reviewerのread入口は[履歴較正によるLLM役割選択](2026-08-03-history-calibrated-llm-role-selection-decision.md)のblind evidence
envelopeを標準とする。exact原文、source/range/hash、記録したquery／anchor scope内の全hit inventoryを一artifactへ機械連結し、
Codexの推奨結論を混ぜない。未収録の関連hitがあれば`ACCEPT`を許さず、reviewerが返したexact rangeの`EVIDENCE_GAP`をSolが
現行sourceへ再照合してfresh waveへ追加する。Fableで実証済みの共通方式をOpus／Grokへも適用し、provider固有の効果量だけを
初回数粒で注記する。未較正を固定fallback、自由探索、適用保留の理由にしない。

## 採用と停止

採用に必要なのはrunner receiptでなく、Codexが直接確認した次の事実である。

- 正しいbase/cwdと開始前後fingerprint
- 許可scope内の実diff
- task固有test／fixture／非LLM oracle
- reviewerがfreshな別familyで、設計・実装へ深く関与せず、review中mutationが0であること
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
