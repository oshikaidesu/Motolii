# Codex Spark CLI起動smoke観察

日付: 2026-08-07  
状態: **観察／起動経路確認済み、実装能力とcontext上限は未検証**  
対象: `gpt-5.3-codex-spark`、Codex CLI、薄いobserved CLI harness

## 結果

現行Codex CLIからexact model ID `gpt-5.3-codex-spark`をread-only、ephemeral、JSONL途中stream付きで起動し、正常完了した。

| 項目 | 観測値 |
|---|---|
| Codex CLI | `0.146.0-alpha.3.1` |
| model ID | `gpt-5.3-codex-spark` |
| sandbox | `read-only` |
| prompt | workspaceを読まず`SPARK_SMOKE_OK`だけを返す |
| duration | 4,864 ms |
| exit | 0、timeoutなし、signalなし |
| terminal text | `SPARK_SMOKE_OK` |
| usage | input 10,063、cached input 7,936、output 51、reasoning output 40 |
| stdout SHA-256 | `683f9f218970a8ee877e4b5b1eadb58a807400be8e6473e91be1ab8864b50cd0` |
| stderr SHA-256 | `4b1c1ae9e61a6aa23020f50ac4163b768d0bda708f7c2b8fdb6d046f6ebdf8ba` |

stdoutは`thread.started -> turn.started -> item.completed -> turn.completed`をJSONLで返した。stderrにはCodex local state DBの
`state db discrepancy ... falling_back` warningが5件出たが、model起動、terminal result、exitには影響しなかった。

このsmokeが証明するのは、現行CLI／accountでexact IDが起動でき、provider-native JSONLとusageを薄いharnessで回収できることだけである。
repo読込、tool使用、編集速度、実装品質、context容量、長いtaskでの保持、利用上限、独立review適格性は証明しない。

## Sparkの運用位置

利用者の既知制約として、Sparkは非常に速い実装modelだがcontext保持が小さい。context上限値は本smokeで測定していないため、
数値を発明せず、運用契約として次へ閉じる。

- authority、owner、意味、target、allowlist、oracleが閉じた一契約境界の施工へ使う
- fresh sessionへ短いcapsule、exact path／symbol／range、必要な正本抜粋、指定commandだけを渡す
- repo全体探索、歴史発掘、authority競合、複数owner統合、semantic final reviewへ使わない
- 不足時は自律的にread scopeを広げず、`CONTEXT_GAP: <path/symbol/range>`を返す
- 同じpromptへ資料を継ぎ足し続けず、主担当が契約を縮小するか別modelをfreshに選ぶ
- OpenAI family施工として、最終reviewは未関与の別familyを選ぶ

Sparkの速度を生かす単位は「短い命令」ではなく、意味が閉じた一契約境界である。construction stepは複数でもよいが、
owner、allowlist、oracle、non-goalを増やさない。

## 実行形

```text
scripts/run-observed-cli.py
  -- <codex> exec --json --ephemeral --ignore-user-config --ignore-rules --strict-config
     --skip-git-repo-check --model gpt-5.3-codex-spark
     -c 'project_doc_max_bytes=0' --sandbox read-only <smoke-prompt>
```

実装発注ではユーザーが許可した隔離worktreeだけを`workspace-write`にし、order、execution envelope、生log、実diff、oracle、
別family reviewを[発注観測・実行・可変配分runbook](../llm-dispatch-observation-and-allocation-runbook.md)へ従って閉じる。
