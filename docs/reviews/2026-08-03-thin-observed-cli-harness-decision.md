# 薄いobserved CLI harness決定

状態: **決定・実装済み（main統合後に発効）**

日付: 2026-08-03

## 決定

Claude Code、Codex CLI、Cursor Agentを接続するMotolii側の共通入口は、provider固有SDKや監督状態機械ではなく
[`scripts/run-observed-cli.py`](../../scripts/run-observed-cli.py)とする。呼出側が作った完全なargvを一回だけshellなしで
起動し、生stdout、生stderr、process lifecycle、終了metadataを保存する。

この接続規律は発注flowの下位手順ではない。**外部LLMを呼ぶ全用途に先行する絶対条件**であり、実装、read-only相談、
調査、検収、診断、session再開の別を問わない。呼出側は起動前に、現行CLIがprovider-nativeの構造化途中streamを提供する
optionを確認し、そのoptionをexact argvへ入れ、生streamを保存しながら主担当が実行中に観測できることを閉じる。次のどれかに
該当する呼出しは起動しない。

- 現行CLIのhelpまたは一次資料で途中stream optionを確認できない
- final textだけを受け取り、thinking、assistant、tool、result等の途中eventを捨てる
- 生streamを実行中に読めない、または終了後まで保存されない
- provider固有のterminal event／tool callを確認せず、chat textだけを最終回答とみなす

heartbeatはproviderがeventを出さない区間でもprocess事実を残す補助であり、provider eventの代替ではない。wall timeoutも暴走時の
回収境界であって、途中進捗の観測、寡黙判定、正常な長時間処理の早期終了根拠を代替しない。

本harnessが所有する責任は次の閉集合だけである。

- absolute executableを含むargvを変換せず起動する
- stdoutとstderrをbyteのまま別fileへteeする
- process生存中は定期heartbeatとして経過時間、PID生存、stdout/stderr byte数、最終出力からのidle時間を記録する
- wall timeoutと親SIGINT/SIGTERMをprocess groupへ転送し、grace後にSIGKILLする
- PID、開始・終了時刻、duration、exit code/signal、timeout、log byte数とSHA-256を記録する
- 既存log directoryを上書きせず、一実行一directoryにする

本harnessは次を所有しない。

- worktreeの作成、選択、清掃
- model、mode、sandbox、ambient設定の選択またはprovider間変換
- authority/base/order hash、exact target、allowlist、WIDE判定、fingerprint/diff
- reviewer独立性・mutation判定、P0/P1判定、最終採否
- session resume/cancelの採用資格、append-only receipt DB、order schema
- JSON eventの解釈、重複除去、final answer抽出、無回答判定

これらの監督責任は[runner非依存監督決定](2026-08-03-runner-independent-supervision-decision.md)に置く。
したがってharnessはprovider非依存のbyte transportのままとし、provider固有eventの観測と最終結果抽出は呼出側のobserverが
所有する。共通化のために新しいrunner状態機械や採用DBをharnessへ戻さない。

これにより、[旧runner廃止決定](2026-08-02-supervised-runner-retirement-decision.md)の旧transport廃止は維持しつつ、
同文書の「Agentexを唯一の直接入口とする」部分だけを撤回する。Agentexは比較対象として残せるが、標準入口ではない。

## CLI契約

```sh
python3 scripts/run-observed-cli.py \
  --cwd /absolute/prepared-worktree \
  --log-dir /absolute/evidence/run-id \
  --timeout-seconds 300 \
  --grace-seconds 5 \
  --heartbeat-seconds 10 \
  -- /absolute/path/to/provider-cli provider-specific fixed arguments
```

`argv[0]`、cwd、log directoryはabsolute pathを渡す。認証情報をargvへ含めると`meta.json`へ残るため、既存CLI認証か
environmentを使う。harnessはenvironmentを記録・消去・補完しない。ambient無効化は各CLIのargv/environmentへ明示し、
無効化できないproviderでは総監督が空のstaging cwdと限定promptを用意する。子から証跡を分離するため、cwdとlog directoryは
互いの配下に置けない。

## 保存物

| file | 内容 |
|---|---|
| `stdout.log` | 子processの生stdout。JSONLであることをharnessは要求しない |
| `stderr.log` | 子processの生stderr。harness lifecycleを混ぜない |
| `lifecycle.jsonl` | `started / heartbeat / timeout / parent_signal / signal_sent / completed / harness_error`のprocess事実 |
| `meta.json` | schema 1のargv、cwd、PID、時刻、終了状態、timeout、heartbeat間隔、byte数、stdout/stderr SHA-256 |

正常終了は子のexit codeを返す。wall timeoutは124、harnessの引数・境界違反は64、起動不能は70、signal終了は
`128 + signal`を返す。`meta.json`は子の終了後にatomic replaceし、実行中の観測は生streamと`lifecycle.jsonl`で行う。
heartbeatはprocessの生存とbyte進捗を示すだけで、stall判定、無回答判定、timeout短縮、採用資格には使わない。寡黙なmodelは
heartbeatとwall timeoutの間で正常に待ち、回答の有無はprocess完了後に呼出側がprovider出力と終了状態から判定する。

## 2026-08-07再確認済みprovider接続

以下はharnessへ内蔵するprofileではなく、各CLIの一次資料と実機helpを照合した呼出側の構成要素である。CLI更新時は
その時点のhelpと一次資料へ再照合する。role、主要model、limit group、可変配分と一runのexecution envelopeは
[外部LLM発注の観測・実行・可変配分runbook](../llm-dispatch-observation-and-allocation-runbook.md)を正本とする。

- Claude Code 2.1.223: `-p --model <exact> --effort <level> --permission-mode plan --setting-sources '' --safe-mode
  --strict-mcp-config --mcp-config '{"mcpServers":{}}' --tools '' --disable-slash-commands --no-chrome
  --no-session-persistence --output-format stream-json --include-partial-messages --verbose`。現行helpにhard turn capは無い。
  hook lifecycle自体が観測対象の時だけ`--include-hook-events`を加える。`--fallback-model`は確認できるがMotoliiでは使わず、
  subscription認証時はAPI key専用の`--bare`へ黙って置換しない。[CLI reference](https://code.claude.com/docs/en/cli-reference)
- Codex CLI 0.146.0-alpha.3.1: `exec --json --ephemeral --ignore-user-config --ignore-rules --strict-config
  --skip-git-repo-check --model <exact> -c 'model_reasoning_effort="<level>"' -c 'project_doc_max_bytes=0'
  --sandbox <mode>`に、不要な
  MCP/web/apps/browser/memory/multi-agent/plugin/skill機能のdisableを明示する。`project_doc_max_bytes=0`は総監督が必要文脈を
  promptへ渡した場合だけ使う。`gpt-5.3-codex-spark`は2026-08-07にexact ID、JSONL、usage、exit 0を
  [実機確認済み](2026-08-07-codex-spark-cli-smoke-observation.md)。[Non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode.md)
- Cursor Agent 2026.08.04-aaa8809: `-p --workspace <absolute> --trust --mode ask --sandbox enabled
  --model <exact> --output-format stream-json --stream-partial-output`。ambient ruleを完全無効化するCLI optionは確認できないため、
  空staging cwdを使う。genericな`agent`名でなくCursor binaryのabsolute pathを渡す。
  [Output format](https://docs.cursor.com/en/cli/reference/output-format)

三CLIに共通のhard turn capはなく、途中streamのevent形も異なる。この差をharness内で正規化せず、
raw byte保存とprocess lifecycleだけを共通化する。呼出側observerはClaudeのpartial/result event、CodexのJSON event、Cursorの
thinking／assistant／result／tool callを実行中に読み、provider固有のterminal statusと最終回答位置を確認する。特にCursorの
plan modeでは結論がchat textでなく`createPlanToolCall.args`に入る実測があるため、両方を検査する。根拠は
[監督loop観察 §7b](2026-08-01-supervision-loop-cost-driver-observation.md#7b-grokが寡黙の原因と解消実装済み2026-08-01)に置く。

## 検証と負例

専用testは、引数中の空白を含むexact argv、生NUL byteを含むstderr、非zero exitの保存、親終了後もpipeを保持する場合を含む
process group内grandchildのtimeout回収（OS process-group reclamation）、既存log directoryの上書き拒否、cwd/log tree重複、relative executable、
非positive timeout／grace／heartbeatの拒否を固定する。さらに、子が少量stdoutをflushしてから寡黙になるfixtureで、
終了前heartbeatにprocess生存、経過時間、byte進捗、idle時間が記録され、そのheartbeatがprocessを終了しないことを確認する。

本粒の完了は専用test、docs check、diff check、および三CLIのread-only smokeで判定する。smokeはmodel回答の品質や
採用資格を証明せず、実model eventが終了前に観測できること、provider固有の最終結果位置、終了状態、生log保存の接続だけを
確認する。CLI version更新後はhelp／一次資料との再照合が済むまで外部LLMを起動しない。
