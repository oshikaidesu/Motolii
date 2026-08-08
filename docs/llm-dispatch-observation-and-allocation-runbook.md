# 外部LLM発注の観測・実行・可変配分runbook

日付: 2026-08-07
状態: **運用正本／CLI snapshotは起動前更新**
対象: Motoliiの外部LLM相談、調査、発注、施工、correction、review

## 1. 目的

[利用者成果からの発注コンパイルと調査返却ループ](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)を、
実際のprovider CLI起動、途中stream観測、証跡保存、利用枠に応じたmodel配分へ接続する。

発注の意味と実行方法を分ける。

| 層 | 所有するもの |
|---|---|
| closed order | outcome、authority、owner、target、allowlist、正負oracle、STOP/RETURN |
| execution envelope | limit group、model family、role、exact model、effort、permission、CLI version、allocation profile |
| observed run | exact argv、生stream、生stderr、lifecycle、exit/signal、hash、provider固有terminal result |
| supervisor disposition | 実diff、oracle、独立性、return採否、次edge再選定 |

model名、CLI flag、利用可能枠は変わる。closed orderへ埋め込まず、本書のCLI snapshotと一runのexecution envelopeで差し替える。

## Launch card

これは**開始直前に主担当が照合済みのclosed order capsuleがある場合だけ**使う短路である。capsuleに
`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / POSITIVE AND NEGATIVE ORACLES / NON-GOALS / RETURN`
の一つでも無い、開始fingerprintが古い、またはauthority競合があれば起動せず、§2の通常動線へ戻る。

1. 極小closed施工はCodex directのfresh Spark、通常〜重めのclosed施工はCursor first-partyのfresh Grok 4.5 non-fast mediumを第一候補にする。候補compileが必要な時だけCodex directのTerraを使う。Composer 2.5 standardは価格、capacity、task実測の理由を記録した代替であり、自動fallbackではない。
2. §5のexecution envelopeへread set、動的予算、想定usage、capsule外read禁止、`CONTEXT_GAP`返却を記録する。起動直前に実binaryの`--version`、`--help`、利用可能ならcatalogでexact IDとstream flagを再確認する。別model、`auto`、fast variantへ黙って変えない。
3. 通常サイズのCursor施工は、許可済みの隔離worktreeとdisjoint log dirに対して次の形で一回だけ起動する。`-p`はwrite／shellを持つため、capsuleのallowlist、read set、sandboxで拘束する。

```sh
python3 scripts/run-observed-cli.py \
  --cwd /absolute/prepared-worktree \
  --log-dir /absolute/disjoint-evidence/run-id \
  --timeout-seconds 300 \
  --grace-seconds 5 \
  --heartbeat-seconds 10 \
  -- /Users/member_ottoto/.local/bin/cursor-agent -p \
  --workspace /absolute/prepared-worktree --trust --sandbox enabled \
  --model cursor-grok-4.5-medium \
  --output-format stream-json --stream-partial-output \
  "$(< /absolute/closed-order-prompt.txt)"
```

4. 実行中に`stdout.log`のprovider eventを観測し、終了後に`stderr.log`、`lifecycle.jsonl`、`meta.json`、provider固有terminal result、開始前後fingerprint、実diff、allowlist、実read、指定oracle、予定値対actual usageを照合する。heartbeatは進捗や品質の証拠ではない。
5. `IMPLEMENTED / PARTIAL / RESEARCH_RETURN / EVIDENCE_GAP / OBSERVATION_FAILURE`へ処分し、採否と次edgeは主担当が決める。同じ施工modelに自己reviewさせず、失敗sessionを別modelへresumeしない。

Spark、read-only、Claude、代替model、独立review、未閉鎖orderはこのcardから推測せず、§2以降とリンク先へ進む。

## 2. 最短動線

外部LLMを使う時は次の順で読む。

1. [発注コンパイルと調査返却loop](reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)でclosed orderを閉じる
2. 本書と[Terra / Grok / Composer役割再配置](reviews/2026-08-07-terra-grok-composer-role-reallocation-decision.md)でallocation profile、role、limit group、model familyを選ぶ
3. 起動する実binaryの`--version`、`--help`、利用可能ならmodel catalogを確認する
4. [薄いobserved CLI harness](reviews/2026-08-03-thin-observed-cli-harness-decision.md)でexact argvを一回だけ起動する
5. 生streamを実行中に読み、provider固有のterminal resultまで保存する
6. [runner非依存監督](reviews/2026-08-03-runner-independent-supervision-decision.md)でdiff、oracle、独立性、returnを処分する
7. current codeから次edgeを再選定し、配分実績を次のmodel選択へ反映する

旧`delegate-cursor-supervised.sh`、固定`Grok -> Spark -> Opus`、route version、compiled grain、runner receiptは起動入口ではない。

## 3. 配分で混ぜてはいけない三軸

### 3.1 limit group

どの利用上限、契約、請求枠を消費するか。初期語彙は次の三つとする。

- `codex_direct`: 現在のCodex taskと、同じ上限へ算入されると利用者が判断したCodex CLI
- `cursor`: Cursor Agent経由。通常はCursor first-partyのGrok 4.5／Composer 2.5だけを使う
- `claude_direct`: Claude Code直接経由

実際の課金・limit共有関係を製品名から推測しない。契約表示で別枠と確認した時だけ新しいlimit groupへ分ける。

### 3.2 model family

独立reviewの判定軸。`OpenAI / Anthropic / xAI / Google / other`を、実際に選んだmodel identityから記録する。
Cursor経由のOpusは`limit group=cursor`かつ`family=Anthropic`、Cursor経由のSolは`limit group=cursor`かつ
`family=OpenAI`である。limit groupが違っても同じfamilyなら独立reviewにならない。

### 3.3 channel優先

通常はmodel提供元のdirect channelを使う。

- OpenAI系のTerra／Luna／Sol／Spark: `codex_direct`
- Anthropic系のOpus／Fable／Sonnet: `claude_direct`
- Cursor first-partyのGrok 4.5／Composer 2.5: `cursor`

Cursor catalogにあるGPT／Claude／Gemini等のthird-party modelは通常候補にしない。対応するdirect channelがlimit、capacity、障害で
実際に利用不能になり、同じtaskを継続する必要がある時だけ、fresh runのexecution envelopeへ
`SUBSTITUTION_REASON: DIRECT_LIMIT_UNAVAILABLE | DIRECT_CAPACITY | DIRECT_OUTAGE`、失敗したdirect run、exact Cursor model IDを記録して
明示代用できる。Cursorの`auto`、alias、無記録fallback、途中sessionの引継ぎは使わない。

### 3.4 role

`candidate compile / research / implementation / correction / diff audit / semantic review`の用途。
role適格性、独立性、permission、利用可能性を先に満たした候補だけが配分weightの対象になる。weightを満たすために
不適格modelへ意味判断、書込、独立reviewを割り当てない。

## 4. allocation profile

profileは固定routeでも自動fallback順でもない。一つの短waveで、**適格な次dispatch**をどのlimit groupへ寄せるかのsoft targetである。
一つのfresh外部sessionを一単位として実績を数え、各return後に目標比率より不足している適格groupを優先する。task固有の
独立性、能力、permission、ユーザー指定が常にweightより優先する。

| profile | `codex_direct` | `cursor` | `claude_direct` | 用途 |
|---|---:|---:|---:|---|
| `balanced` | 30 | 40 | 30 | 通常。三枠を偏らせない |
| `codex-conserve` | 10 | 55 | 35 | 現在のCodex枠が逼迫。Codexはauthority照合、採否、利用者対話へ集中 |
| `cursor-conserve` | 35 | 10 | 55 | Cursor枠が逼迫 |
| `claude-conserve` | 40 | 50 | 10 | Claude枠が逼迫 |
| `manual` | runごと | runごと | runごと | 利用者がexact channel/modelを指定 |

profileの切替は次の一行で行う。比率を微調整する時だけ二行目を上書きする。

```text
ALLOCATION_PROFILE: codex-conserve
ALLOCATION_WEIGHTS: codex_direct=10,cursor=55,claude_direct=35
```

これはtoken、USD、wall timeがprovider間で同じという主張ではない。providerがusageを返す場合は別に記録し、返さない値を0で埋めない。
一waveが短く比率へ収束しない場合も、hard quotaにするため不要なLLM callを増やさない。

### `codex-conserve`でCodexに残す仕事

- ユーザー許可、現行authority、dirty state、最終scopeの確認
- 外部modelが返した原文、候補、diff、oracleの短い再照合
- 採否、局所STOP、次edgeの再選定

repo sweep、候補orderの下書き、closed order内の施工、別family reviewは、適格ならCursor／Claude側へ寄せる。Codexの
authority責任を外部回答へ移すのではなく、Codex自身が読む量と生成量を証拠の差分へ縮める。

## 5. 一runのexecution envelope

起動前に次を短く記録する。これは採用DBやrunner receiptではなく、raw logを発注意味へ戻すrun-local indexである。

```text
RUN_ID:
ORDER / CAPSULE SHA256:
ALLOCATION_PROFILE / CURRENT COUNTS:
LIMIT_GROUP / MODEL_FAMILY:
ROLE / INDEPENDENCE EXCLUSIONS:
CLI ABSOLUTE PATH / VERSION:
EXACT MODEL / EFFORT OR VARIANT:
PERMISSION / SANDBOX / TOOL ALLOWLIST:
READ SET / CAPSULE-EXTERNAL READ: FORBIDDEN:
CAPSULE_BYTES / DIFF_BYTES / ORACLE_BYTES / AUTHORIZED_READ_BYTES:
TOOL_TURN_BUDGET / EXPECTED_OUTPUT_BYTES:
EXPECTED_PROVIDER_USAGE: input=<range|UNKNOWN>,output=<range|UNKNOWN>,cache=<range|UNKNOWN>
CONTEXT_GAP RETURN: <exact missing path/range/symbol/oracle>
PROVIDER STREAM FLAGS / TERMINAL RESULT RULE:
TIMEOUT / LOG DIR:
SELECTION REASON:
```

`CURRENT COUNTS`は当該short waveの外部session実績だけでよい。長期score DB、modelランキング、固定fallback chainを作らない。
model変更時は同じ欄をfresh runとして作り、変更理由を残す。失敗sessionを別modelへresumeしない。

`READ SET`はexact pathだけでなく、必要ならrange、symbol、snippet hash、または一つのblind evidence envelopeまで閉じる。
closed施工／reviewはcapsule外readを自己許可しない。予算内で判定不能なら`CONTEXT_GAP`として不足証拠を特定し、主担当へ返す。

予算はmodelの最大contextではなく、このrunに必要な一意なartifactから作る。全run共通の固定byte／token閾値へ戻さない。
`EXPECTED_PROVIDER_USAGE`は同じ用途、model、effortの直近raw logがある時だけrangeを置き、較正が無ければ`UNKNOWN`とする。
hard capがCLIに無い場合も、read setと入力artifactを起動前に閉じ、実行後にprovider eventのnatural turnとusageへ照合する。

## 6. 共通観測入口

全providerを[`scripts/run-observed-cli.py`](../scripts/run-observed-cli.py)経由で起動する。

```sh
python3 scripts/run-observed-cli.py \
  --cwd /absolute/prepared-worktree \
  --log-dir /absolute/disjoint-evidence/run-id \
  --timeout-seconds 300 \
  --grace-seconds 5 \
  --heartbeat-seconds 10 \
  -- /absolute/path/to/provider-cli provider-specific-arguments
```

保存物は`stdout.log`、`stderr.log`、`lifecycle.jsonl`、`meta.json`。`meta.json`のexact argv、version確認結果、execution envelopeを
合わせて一runを復元する。heartbeatはprocess生存、hashはbyte一致だけを証明し、回答品質、進捗、採用資格を証明しない。

promptや認証情報をargvに置くと`meta.json`へ残る。secretをargvへ入れない。証跡の保存先、保持期間、機密区分は利用者の環境規則に従う。

## 7. 2026-08-07実機CLI snapshot

起動前に毎回再確認する。ここに無いflagやmodel IDを推測しない。

| channel | 実binary／version | 構造化途中stream | catalog |
|---|---|---|---|
| Cursor Agent | `/Users/member_ottoto/.local/bin/cursor-agent` / `2026.08.04-aaa8809` | `-p --output-format stream-json --stream-partial-output` | `--list-models`あり |
| Claude Code | `/Users/member_ottoto/.npm-global/bin/claude` / `2.1.223` | `-p --output-format stream-json --include-partial-messages --verbose` | account catalog一覧flagなし。exact IDを起動前記録 |
| Codex CLI | `/Applications/ChatGPT.app/Contents/Resources/codex` / `0.146.0-alpha.3.1` | `exec --json` | `exec --help`に一覧flagなし。exact IDを起動前記録 |

Claude Code 2.1.223のhelpにhard turn capは無い。`--fallback-model`は存在するが、無記録fallbackになるためMotoliiでは使わない。
`--max-budget-usd`はAPI課金境界が確認できる場合の補助であり、subscription limitや完了を代替しない。

### Cursor Agent read-only

```text
<cursor-agent> -p --workspace <cwd> --trust --mode ask --sandbox enabled
  --model <exact-id> --output-format stream-json --stream-partial-output <prompt>
```

計画artifactが必要なら`--mode plan`を使う。terminal answerはassistant textだけでなく`createPlanToolCall.args`も検査する。
`--force / --yolo`、`--approve-mcps`は標準にしない。書込施工はユーザー許可済みの隔離worktreeで、current helpとsandbox挙動を
事前probeしてからexact argvを閉じる。

### Claude Code read-only

```text
<claude> -p --model <exact-id> --effort <level> --permission-mode plan
  --setting-sources '' --safe-mode --strict-mcp-config --mcp-config '{"mcpServers":{}}'
  --tools Read,Glob,Grep --disallowedTools Edit,Write,Bash
  --no-chrome --no-session-persistence
  --output-format stream-json --include-partial-messages --verbose <prompt>
```

`--bare`はAPI key専用認証へ変えるためsubscription運用へ黙って加えない。施工時も`--dangerously-skip-permissions`や
`--fallback-model`を標準化せず、隔離worktree、必要tool、permissionをclosed orderに合わせて明示する。

2026-08-07のWeb-only capability probeでは、fresh empty workspace、repo tool 0、`WebSearch,WebFetch`限定で、
`--permission-mode default`は非対話承認を閉じられずpermission denial、別fresh runの`--permission-mode auto`は
exact query一回と公式bodyのWebFetch一回を途中stream付きで完走した。これはClaude directの当該Web-only runの限定観察であり、
`auto`を施工、repo read、他provider、他toolの標準へしない。runnerへpermission仲裁、query counter、read-set強制を追加する根拠にも
しない。workspace内readのhard enforcementが必要なreviewは、empty workspaceまたは一つのblind evidence envelopeで閉じる。

### Codex CLI

```text
<codex> exec --json --ephemeral --ignore-user-config --ignore-rules --strict-config
  --skip-git-repo-check --model <exact-id>
  -c 'model_reasoning_effort="<level>"' -c 'project_doc_max_bytes=0'
  --sandbox read-only <prompt>
```

施工時だけ許可scopeに応じて`sandbox=workspace-write`を検討する。`danger-full-access`やapproval bypassを標準にしない。
OpenAI familyのmodelが施工したdiffを別のOpenAI modelでreviewしても独立family reviewにはならない。

## 8. 主要modelの役割候補

利用可能性はCursorの`--list-models`または各direct CLIで起動前確認する。表は固定割当でなく、現在較正済み用途の候補である。

| model／系列 | 主な候補role | 注意 |
|---|---|---|
| Cursor Grok 4.5 low／medium／high | 通常〜重めのclosed implementation、指定test、exact diff audit | exact IDは`cursor-grok-4.5-*`。non-fast mediumを通常施工、highを複雑な一境界の候補にする。自己施工のreviewと採否は行わない |
| Codex Spark | 閉じた一契約境界の超高速施工 | exact ID `gpt-5.3-codex-spark`をCodex CLIで起動確認済み。context保持が小さいため短いcapsule、exact target／read set、指定oracleに限定 |
| Composer 2.5 | 安価な代替施工、capacity回避、task適合が観測済みの通常施工 | standardの`composer-2.5`を候補にし、Fastやfallbackを暗黙defaultにしない |
| GPT-5.6 Terra | current fact整理、候補order、負例、return条件のcompile | Codex directの完全IDを確認して使う。closed orderへ毎回挟む固定副監督にしない。Cursor版はdirect不能時の明示代用だけ |
| GPT-5.6 Luna | 複雑施工、同一境界correction候補 | OpenAI family。Sol／Terra施工の独立reviewにならない |
| GPT-5.6 Sol | authority競合、広い統合判断候補 | 高価な常置routeにせず、Codex逼迫時は最小reconciliationへ縮める |
| Claude Opus 5 | semantic反対側、final review、難しい設計相談 | CLOSEDならlowを通常候補にできる。施工・設計関与時はfinal reviewerから外す |
| Claude Fable 5 | 大地図、恒久境界、候補取りこぼし検査 | Claude directを通常経路にし、広い時だけ使う。Cursor版はdirect不能時の明示代用かつ`NO ZDR`表示をdata policyへ照合する |
| Claude Sonnet 5 | 中程度のread-only調査、施工候補 | Opusを常用する必要がない境界で較正して使う |
| Gemini／Kimi等 | 将来の候補 | catalog存在だけでは採用しない。最初は短いread-only較正と非証明範囲を残す |

effortはroleの代替ではない。lowでも閉じた契約とexact evidenceなら十分な場合があり、high/maxでもscope不足やauthority競合を
解決しない。

通常選択は[Terra / Grok / Composer役割再配置決定](reviews/2026-08-07-terra-grok-composer-role-reallocation-decision.md)に従う。
orderが既に閉じていればTerraを省略し、極小はSpark、通常〜重めはGrok 4.5 non-fastを第一候補にする。Composer 2.5 standardは
価格、capacity、task実測の理由がある時の明示候補であり、Grok失敗時の自動fallbackではない。

### Sparkの専用契約

Sparkは候補探索や副監督ではなく、主担当が既に閉じたorderの施工へ使う。2026-08-07の
[CLI smoke](reviews/2026-08-07-codex-spark-cli-smoke-observation.md)では、`gpt-5.3-codex-spark`が4,864 ms、exit 0、
provider-native JSONL、usage付きで正常完了した。これは起動経路だけの証明で、context容量や実装品質のbenchではない。

Spark capsuleは少なくとも次へ縮める。

```text
OUTCOME EDGE / OWNER:
EXACT TARGET: <path:symbol/range>
AUTHORITY EXCERPT:
ALLOWLIST / READ SET:
INPUT -> TRANSITION -> TERMINAL:
POSITIVE / NEGATIVE ORACLE:
NON-GOALS:
RETURN: IMPLEMENTED | PARTIAL | CONTEXT_GAP <exact missing evidence>
```

全repo、全履歴、複数候補比較を渡さない。context不足を追加readの自己許可にせず`CONTEXT_GAP`で返させ、主担当が`REDUCE`または
別modelのfresh runを選ぶ。Spark施工はOpenAI familyなので、Sol、Terra、Luna、別Codexによるreviewを独立検収と数えない。

## 9. run完了と配分再計算

呼出側observerは終了後に次を閉じる。

1. `meta.json`とraw log hash、exit/signal/timeoutを確認
2. provider固有terminal resultと最終回答位置を確認
3. usage、cost、token、natural turnが出た場合は原値を記録し、無い値は`UNKNOWN`。failure、timeout、REJECT、差戻しでも省略しない
4. execution envelopeの`EXPECTED_PROVIDER_USAGE`とactual usage、`TOOL_TURN_BUDGET`とnatural turn、`READ SET / AUTHORIZED_READ_BYTES`と実tool readを同じ単位で比較する。byteとtokenを相互換算しない
5. 実装runなら開始前後fingerprint、実diff、allowlist、指定oracleを確認。capsule外readがあれば採用せずscope違反として主担当へ返す
6. returnを`IMPLEMENTED / PARTIAL / CONTEXT_GAP / RESEARCH_RETURN / EVIDENCE_GAP / OBSERVATION_FAILURE`へ処分
7. `CONTEXT_GAP`または予算超過では同じsessionへ全文を追加せず、主担当が不足原文を検証して`REDUCE`、粒分割、または更新したfresh waveを選ぶ
8. 当該waveのlimit group実績を一単位増やす
9. current codeから次edgeを再選定し、そのroleで適格な候補だけを次のweight比較へ入れる

CLI failure、空のterminal result、rate limit、capacityは`OBSERVATION_FAILURE`である。比率調整のため別modelへ黙って再送せず、
主担当がfresh run、exact model、変更理由を記録する。

## 10. 非目標

- profileを固定pipeline、fallback順、採用資格にする
- 利用枠が同じこととmodel familyが同じことを混同する
- 比率達成のため不要な発注を増やす
- raw logを長期authority、project memory、品質score DBにする
- exact model ID、CLI flag、subscription課金関係を古いsnapshotから推測する
- Codex枠節約を理由に、ユーザー権限、authority、最終採否を外部LLMへ移す

## 11. 導線QA

発注導線のdiscoverabilityは、repo既知情報を持たないfreshな**監督役**へ、利用者要求だけを渡して確認する。期待するmodel名、文書名、
command、判定をpromptへ列挙しない。read上限を設ける場合も、正解文書を指定するためでなく短路が実際に成立するかを見る。

この試験は最低でもLuna相当の監督判断能力を持つmodelで行う。Sparkはclosedな極小施工役であり、routing解釈、発注準備、監督導線の
合否判定には使わない。Sparkの失敗を導線FAIL、成功を導線PASSの証拠にしない。

2026-08-07のfresh Luna read-only試験では、`AGENTS.md`から本書のLaunch cardへ到達し、二文書だけで通常サイズ施工を
`cursor-grok-4.5-medium`、observed harness、provider stream、終了後証跡、return分類へ復元した。実closed-order capsuleを渡して
いないためplaceholderを確定せず停止したことも正しい。この観測は導線到達だけを支持し、実装品質やmodel固定を証明しない。
