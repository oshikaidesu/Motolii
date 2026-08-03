# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。本書は常時規律と条件別の正本routingだけを持つ。
詳細手順、時点依存のmodel情報、phase進捗、固定SHAはリンク先が所有する。

## 常時規律

- **自己発注禁止**: 許可されたoutcome、成果物、mutation、validation、外部model呼出し、完了条件を主担当Codexが増やさない
- **findingは権限ではない**: 調査、test、review、隔離、安全性、技術的有用性から追加施工を始めない。既存完了条件を阻むscope内原因だけを許可範囲の最小修正へ戻し、他はfindingとして報告する
- **既決を未決へ戻さない**: 提案・設計・実装前に[決定逆引き台帳](docs/decision-index.md)を主題keywordで検索し、正本、現行状態、コード事実を確認する。衝突時だけ該当操作を`AUTHORITY_CONFLICT`として止める
- **既知実装優先 — 新設前に探索・採択する**: 一般機構と製品意味を計画・仕様化・発注・実装する前に、repo、decision index、[references](docs/references.md)、製品先例、一次資料を調べる。主担当preflightには`MECHANISM CLASS / KNOWN IMPLEMENTATION SEARCH / CANDIDATES / ADOPTION ROUTE / REJECTED CANDIDATES / THIN MOTOLII SEAM / THIN MOTOLII RESIDUAL / RETIREMENT / BUILD JUSTIFICATION / BUILD: FORBIDDEN`を短く記録する。欠落、検索先なし、候補なし、裁定なし、一般frameworkの薄い残余への偽装では実装しない。`BUILD JUSTIFICATION`が`NONE`以外なら通常発注を止め、利用者例外へ戻す。詳細は[既知実装採択・置換開発モデル](docs/known-implementation-adoption-model.md)
- **STOPは局所信号**: 危険操作、未決契約の発明、該当粒の施工だけを止め、親taskと接続可能なlaneは`REUSE / REMAP / REDUCE / 再調査`で続ける。利用者判断なしに安全な次手がない場合だけ返す
- **状態を繰り上げない**: WIP、fixture、probe、test green、外部review、main統合、通常製品route、製品完成を分ける。LLMの賛同をauthorityや採用資格にしない
- **外部LLMは途中stream必須**: 発注、相談、調査、検収、診断、再開を問わず、provider-nativeの構造化途中streamを有効化し、主担当が実行中に観測できる状態で生streamを保存してから起動する。対応optionを現行CLIのhelp／一次資料で確認できない、途中eventを保存・観測できない、最終textしか得られない場合はfail closedする。heartbeatやwall timeoutをprovider eventの代用、寡黙判定、早期終了根拠にしない。詳細は[薄いCLI監視](docs/reviews/2026-08-03-thin-observed-cli-harness-decision.md)

## 条件別に読む正本

最初に[docs/README.md](docs/README.md)、対象phaseの[仕様](docs/specs/README.md)末尾の実装ガード、[implementation ledger](docs/implementation-ledger.md)を読む。次の条件に該当する時だけ追加正本を読み、未確認なら当該施工を開始しない。

| 条件 | 必須正本 |
|---|---|
| 外部LLMへの発注、相談、検収 | [runner非依存監督](docs/reviews/2026-08-03-runner-independent-supervision-decision.md)、[LLM役割選択](docs/reviews/2026-08-03-history-calibrated-llm-role-selection-decision.md)、[薄いCLI監視](docs/reviews/2026-08-03-thin-observed-cli-harness-decision.md) |
| 一般機構の新設・置換 | [既知実装採択・置換開発モデル](docs/known-implementation-adoption-model.md)、[依存優先・責任最小化ゲート](docs/reviews/2026-07-24-dependency-first-responsibility-gate.md) |
| M2 Document、schema、journal | [恒久焼き込みの予防](docs/reviews/2026-07-12-m2-permanence-prevention.md) |
| M3製品実装 | [M2基盤再締結gate](docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md)、対象M3正本 |
| M3 UI、入力、Timeline、panel | [UI境界の規律](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)、[UI成果物用語](docs/ui-artifact-terminology.md)、[UI参照地図](docs/ui-reference-map.md) |
| React mock／product source asset | [React製品資産の直接移管契約](docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md) |
| Rerunのsource、crate、画面、pattern | [Rerun inventory](docs/reviews/2026-07-20-rerun-source-asset-inventory.md)、[Rerun学習・転移計画 §9](docs/reviews/2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線) |
| pluginの作成・量産 | [plugin authoring](docs/plugin-authoring.md) |

## Motoliiの絶対規律

1. **VRAM常駐**: ピクセルはwgpu textureとしてGPUに置き、安易なCPU処理を混ぜない
2. **色変換一元化**: 色変換はrender直前の一箇所だけ
3. **plugin純関数**: 出力は時刻`t`と型付き入力で決まり、隠れた可変状態を持たない。時間依存が必要な表現は[simulation model](docs/simulation-model.md)の正規routeへ送る
4. **single writer**: Documentを書き換えるのは編集threadだけ。他はimmutable snapshotのreader
5. **正準座標**: 空間parameterは単位なし、原点中央、Y-up、高さ1.0。絶対pxを永続意味にしない
6. **Preview / Export同一評価**: 差は`Quality`だけ。別render経路を作らない
7. **vendor／OS非依存契約**: pluginへCUDA、Metal、DX等を露出せず、wgpu／WGSL抽象を使う

## 計画と実装

- 着手前にbranch、HEAD、`git status --short --branch`、local mainとの関係、`git worktree list`を確認する。dirty worktreeの既存差分は利用者のものとして保持し、編集はlocal `main`から専用clean worktreeを作る
- 利用者成果を一つのownerと契約境界へ閉じ、`AUTHORITY → INTERNAL TARGET → OWNER → WRITE ROUTE → GAP → RESOLUTION ROUTE → DISPOSITION`へ写す。実在identity、command、consumer、layout slot、公開契約がなければ推測しない
- `GAP`は未調査やUI名称差でなく、現行型・source・試験の不在または契約矛盾で示す。既存targetがあれば再決定せず接続し、公開API、Document意味、plugin契約、永続形式の新設・変更は仕様粒で先に閉じる
- 一回の実装は一契約境界と閉じた変更fileへ限定する。施工step数を粒数とみなさず、owner、意味、完了条件が増えるなら別の利用者許可へ戻す
- 新規helper、依存、一般機構、UI componentを書く前に同等物を検索する。React source assetが存在する時は縮約copyを作らず、Rerunを参照する時はMotolii仕様から逆算しない
- test、golden、threshold、期待値を実装都合で変更してgreenにしない。testが誤りに見える場合は施工を止め、独立した仕様・oracle変更として扱う
- 会話で新しい意味、状態owner、操作、配布形式が生じたら、観察／比較中／決定／棄却／停止と非目標をコードより先に正本へ回収する。会話だけをauthorityにしない
- **1 ticket = 1 commit**。仕様・decisionを変更したら同じcommitでdecision indexと必要なledgerを更新する

## 外部LLMと検収

- ユーザーが「発注して」「実装を発注」等を依頼動詞として明示した時だけ外部実装を起動する。通常の「実装して」、説明、引用内の語では自動委任しない
- 主担当Codexがbase/cwd、worktree、authority、scope、allowlist、非目標、oracle、fingerprint、diff、最終採否を所有する。意味、owner、原因、再利用、oracleが閉じない`WIDE`は実装担当へ送らない
- 外部CLIは[`run-observed-cli.py`](scripts/run-observed-cli.py)でexact argvを起動し、provider-nativeの構造化途中stream、生stderr、exit／signal、process回収を保存する。呼出側は実行中のeventを観測し、provider固有の最終結果位置まで読む。harnessはJSON意味解釈、worktree、意味判断、採否、session資格を所有しない
- modelはtaskの判定対象で選び、利用不能時に別modelへ黙ってfallbackしない。外部modelへ再委任、秘密情報、認証情報、未公開個人情報を渡さない
- 実装担当と最終reviewerは別session・別役割にし、同taskの設計・施工へ深く関与したmodel familyを最終reviewerに使わない。reviewerはread-onlyで実diffと試験を監査し、mutationした検収を無効とする。性能、安全性、永続性、platform correctnessは非LLM oracleで判定する
- 採用前にCodexが開始前後fingerprint、実diff、scope、試験、review、P0/P1、reviewer mutationを再照合する。ユーザーSTOP後は対象processを止め、新しい編集・試験・reviewを開始しない

## 実装規約

- 公開APIで入力起因の失敗をpanic／`assert!`にせず、構造化した`Result`で返す
- GPU resourceをloop内で生成しない。texture、buffer、pipeline、shaderは再利用する
- `?`の早期returnが後始末を飛ばさないか確認する。特に`Encoder::finish()`前のreturnを避ける
- errorを文字列へ潰さず、`#[from]`／transparent errorで原因構造を保つ
- test helperは`motolii-testkit`へ集約し、`gpu_or_skip`等を複製しない
- コメントは日本語で「なぜ」だけを書く

## 検証と完了報告

- 各粒へ`PRIMARY_ORACLE / REPO_LANES / EXTERNAL_GATES`を固定する。`cargo test`はRust laneであり、React、docs、製品E2E、実機、人間審判を代替しない
- docs／review変更は`./scripts/check-docs.sh`と`git diff --check`を通す。通常提出は`./scripts/validate.sh local`、必要なtask固有test、利用可能なら`./scripts/test-local.sh`を実行する
- 一つのlaneが既知不具合でredでも全体をgreenと報告しない。実diff、製品route、validation／review、integration、blocker、未実行gateを分離して報告する
- 完了時は実行commandと結果、commit、main統合有無、残存dirty差分、次の一粒と非目標を示す。「動くはず」「たぶん完了」を使わない
