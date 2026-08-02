# 薄いobserved CLI harness決定

状態: **決定・実装済み（main統合後に発効）**

日付: 2026-08-03

## 決定

Claude Code、Codex CLI、Cursor Agentを接続するMotolii側の共通入口は、provider固有SDKや監督状態機械ではなく
[`scripts/run-observed-cli.py`](../../scripts/run-observed-cli.py)とする。呼出側が作った完全なargvを一回だけshellなしで
起動し、生stdout、生stderr、process lifecycle、終了metadataを保存する。

本harnessが所有する責任は次の閉集合だけである。

- absolute executableを含むargvを変換せず起動する
- stdoutとstderrをbyteのまま別fileへteeする
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

これにより、[旧runner廃止決定](2026-08-02-supervised-runner-retirement-decision.md)の旧transport廃止は維持しつつ、
同文書の「Agentexを唯一の直接入口とする」部分だけを撤回する。Agentexは比較対象として残せるが、標準入口ではない。

## CLI契約

```sh
python3 scripts/run-observed-cli.py \
  --cwd /absolute/prepared-worktree \
  --log-dir /absolute/evidence/run-id \
  --timeout-seconds 300 \
  --grace-seconds 5 \
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
| `lifecycle.jsonl` | `started / timeout / parent_signal / signal_sent / completed / harness_error`のprocess事実 |
| `meta.json` | schema 1のargv、cwd、PID、時刻、終了状態、timeout、byte数、stdout/stderr SHA-256 |

正常終了は子のexit codeを返す。wall timeoutは124、harnessの引数・境界違反は64、起動不能は70、signal終了は
`128 + signal`を返す。`meta.json`は子の終了後にatomic replaceし、実行中の観測は生streamと`lifecycle.jsonl`で行う。

## 2026-08-03確認済みprovider接続

以下はharnessへ内蔵するprofileではなく、各CLIの一次資料と実機helpを照合した呼出側の構成要素である。CLI更新時は
その時点のhelpと一次資料へ再照合する。

- Claude Code 2.1.216: `-p --model <exact> --effort <level> --permission-mode plan --setting-sources ''
  --strict-mcp-config --mcp-config '{"mcpServers":{}}' --tools '' --disable-slash-commands --no-chrome
  --no-session-persistence --max-turns 1 --output-format stream-json --verbose`。subscription認証時はAPI key専用の`--bare`へ
  黙って置換しない。[CLI reference](https://code.claude.com/docs/en/cli-reference)
- Codex CLI 0.146.0-alpha.3.1: `exec --json --ephemeral --ignore-user-config --ignore-rules --strict-config
  --skip-git-repo-check --model <exact> -c 'model_reasoning_effort="<level>"' -c 'project_doc_max_bytes=0'
  --sandbox <mode>`に、不要な
  MCP/web/apps/browser/memory/multi-agent/plugin/skill機能のdisableを明示する。`project_doc_max_bytes=0`は総監督が必要文脈を
  promptへ渡した場合だけ使う。[Non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode.md)
- Cursor Agent 2026.07.23-e383d2b: `-p --workspace <absolute> --trust --mode ask --sandbox enabled
  --model <exact> --output-format stream-json --stream-partial-output`。ambient ruleを完全無効化するCLI optionは確認できないため、
  空staging cwdを使う。genericな`agent`名でなくCursor binaryのabsolute pathを渡す。
  [Output format](https://docs.cursor.com/en/cli/reference/output-format)

Claudeの`--max-turns`以外に共通のwall timeoutはなく、途中streamのevent形も三者で異なる。この差を正規化せず、
harnessのwall timeoutとraw byte保存だけを共通化する。

## 検証と負例

専用testは、引数中の空白を含むexact argv、生NUL byteを含むstderr、非zero exitの保存、親終了後もpipeを保持する場合を含む
process group内grandchildのtimeout回収、既存log directoryの上書き拒否、cwd/log tree重複、relative executable、
非positive timeoutの拒否を固定する。

本粒の完了は専用test、docs check、diff check、および三CLIのread-only smokeで判定する。smokeはmodel回答の品質や
採用資格を証明せず、実model event、途中stream、終了状態、生log保存の接続だけを確認する。
