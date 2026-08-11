# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。本書は常時規律と条件別の正本routingだけを持つ。
詳細手順、時点依存のmodel情報、phase進捗、固定SHAはリンク先が所有する。

## 常時規律

- **自己発注禁止**: 許可されたoutcome、成果物、mutation、validation、外部model呼出し、完了条件を主担当Codexが増やさない
- **findingは権限ではない**: 調査、test、review、隔離、安全性、技術的有用性から追加施工を始めない。既存完了条件を阻むscope内原因だけを許可範囲の最小修正へ戻し、他はfindingとして報告する
- **既決を未決へ戻さない**: 提案・設計・実装前に[決定逆引き台帳](docs/decision-index.md)を主題keywordで検索し、正本、現行状態、コード事実を確認する。衝突時だけ該当操作を`AUTHORITY_CONFLICT`として止める
- **既知実装優先 — 新設前に探索・採択する**: 一般機構と製品意味を計画・仕様化・発注・実装する前に、repo、decision index、[references](docs/references.md)、製品先例、一次資料を調べる。開発組織も同じ対象とし、GitHub Issue／branch／worktree／PR／review／mainで足りる時に独自queue、broker、監督frameworkを作らない。主担当preflightには`MECHANISM CLASS / KNOWN IMPLEMENTATION SEARCH / CANDIDATES / ADOPTION ROUTE / REJECTED CANDIDATES / THIN MOTOLII SEAM / THIN MOTOLII RESIDUAL / RETIREMENT / BUILD JUSTIFICATION / BUILD: FORBIDDEN`を短く記録する。欠落、検索先なし、候補なし、裁定なし、一般frameworkの薄い残余への偽装では実装しない。`BUILD JUSTIFICATION`が`NONE`以外なら通常発注を止め、利用者例外へ戻す。これは現行標準として確定するが凍結契約ではなく、具体的反証と実測に基づき正本とdecision indexを同時更新して改訂できる。詳細は[既知実装採択・置換開発モデル](docs/known-implementation-adoption-model.md)
- **発注はreturn後の再選定まで閉じる**: 通常製品routeの操作列、stable identity、成功出口、失敗回復、自動oracle、external gateを利用者成果の背骨として先に固定し、そこから一契約境界をcompileする。実装できない時は検索場所、候補、採否、不適合理由、exact gap、再入場条件を`RESEARCH_RETURN`として返す。調査不足は`REUSE / REMAP / REDUCE / 再調査 / WAIT_TARGET`へ局所処分し、主担当は古い`next`や粒数へ戻らずcurrent codeから次edgeを再選定する。詳細は[発注コンパイルと調査返却loop](docs/reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)と[利用者成果の背骨](docs/reviews/2026-08-04-outcome-spine-autonomous-gap-research-decision.md)
- **STOPは局所信号**: 危険操作、未決契約の発明、該当粒の施工だけを止め、親taskと接続可能なlaneは`REUSE / REMAP / REDUCE / 再調査`で続ける。利用者判断なしに安全な次手がない場合だけ返す
- **状態を繰り上げない**: WIP、fixture、probe、test green、外部review、main統合、通常製品route、製品完成を分ける。LLMの賛同をauthorityや採用資格にしない
- **外部LLMは途中stream必須**: 発注、相談、調査、検収、診断、再開を問わず、provider-nativeの構造化途中streamを有効化し、主担当が実行中に観測できる状態で生streamを保存してから起動する。対応optionを現行CLIのhelp／一次資料で確認できない、途中eventを保存・観測できない、最終textしか得られない場合はfail closedする。heartbeatやwall timeoutをprovider eventの代用、寡黙判定、早期終了根拠にしない。詳細は[薄いCLI監視](docs/reviews/2026-08-03-thin-observed-cli-harness-decision.md)

## 条件別に読む正本

通常は最初に[docs/README.md](docs/README.md)、対象phaseの[仕様](docs/specs/README.md)末尾の実装ガード、[implementation ledger](docs/implementation-ledger.md)を読む。次の条件に該当する時だけ追加正本を読み、未確認なら当該施工を開始しない。

例外として、主担当が開始直前に照合済みのclosed order capsuleを渡し、その中に`BASE / AUTHORITY / CURRENT STATE / OWNER / EXACT TARGET / ALLOWLIST / READ SET / POSITIVE AND NEGATIVE ORACLES / NON-GOALS / RETURN`が揃う外部施工準備では、fresh agentに一般bootstrapを再実行させない。[発注runbookのLaunch card](docs/llm-dispatch-observation-and-allocation-runbook.md#launch-card)だけを先に読み、欠落、古いfingerprint、競合が見つかった時に限り通常bootstrapへ戻る。これは主担当のauthority照合を省略する規則ではない。

| 条件 | 必須正本 |
|---|---|
| 計画、発注、`STOP / RETURN`、次粒再選定 | [発注コンパイルと調査返却loop](docs/reviews/2026-08-07-outcome-order-compilation-and-research-return-loop.md)、[既知実装採択モデル](docs/known-implementation-adoption-model.md) |
| M3〜M5全体並列campaign、deputy／field、総監督takeover、停止耐性 | [統一並列開始baseline](docs/reviews/2026-08-09-unified-parallel-start-baseline-decision.md)、[cold-replaceable監督と停止封じ込め](docs/reviews/2026-08-09-cold-replaceable-supervision-failure-containment-decision.md) |
| 外部LLMへの発注、相談、検収、利用枠に応じたmodel配分 | まず[Launch card](docs/llm-dispatch-observation-and-allocation-runbook.md#launch-card)。意味未閉鎖、例外選択、検収ではcardからリンクされた詳細正本だけを追加する |
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
- `TARGET_MISSING`や`STOP`は状態語だけで返さず、検索範囲、候補、採否、不適合理由、exact gap、再入場条件、安全に継続できるedgeを返す。主担当は返却後に現行codeから次の一契約境界を選び直す
- `GAP`は未調査やUI名称差でなく、現行型・source・試験の不在または契約矛盾で示す。既存targetがあれば再決定せず接続し、公開API、Document意味、plugin契約、永続形式の新設・変更は仕様粒で先に閉じる
- 一回の実装は一契約境界と閉じた変更fileへ限定する。施工step数を粒数とみなさず、owner、意味、完了条件が増えるなら別の利用者許可へ戻す
- 新規helper、依存、一般機構、UI componentを書く前に同等物を検索する。React source assetが存在する時は縮約copyを作らず、Rerunを参照する時はMotolii仕様から逆算しない
- test、golden、threshold、期待値を実装都合で変更してgreenにしない。testが誤りに見える場合は施工を止め、独立した仕様・oracle変更として扱う
- 会話で新しい意味、状態owner、操作、配布形式が生じたら、観察／比較中／決定／棄却／停止と非目標をコードより先に正本へ回収する。会話だけをauthorityにしない
- **M3 RN製品UI target凍結**: 製品shell、app root、Browser、Inspector、通常panel、native componentの唯一の接続先は`ui/motolii-rn/`である。`spikes/motolii-rn-probe/`は検証・visual oracle専用で、製品source、次のapp root、移植元UIにはしない。別RN app／shell／root、既存panelの縮約copy、probe側への製品機能追加を作らない。変更が必要なら、利用者の明示判断と、移行route・cutover・旧target退役を記した解凍decisionを先にmainへ入れ、decision indexとimplementation ledgerを同時更新する
- 新規施工は **1 Issue = 1契約境界 = 1 owner = 1 commit = 1 PR**。ここでいう契約境界は一つの利用者成果と意味ownerであり、それを通すRust、React Native、shader、fixture、test、docsをfile数だけで分割しない。PRは良い塊を運ぶlanding envelopeであってapproval gateではない。既存成果へIssue新設、history rewrite、PR分割、再reviewを遡及要求しない。仕様・decisionを変更したら同じcommitでdecision indexと必要なledgerを更新する
- 並列発注では各Issue／PRへ`OUTCOME / SEMANTIC OWNER / SHARED SEATS TOUCHED / INTEGRATION OWNER / PRODUCT STATE / ORACLE / KNOWN LIMITS`を明記する。同じshared seatを触る複数PRを同時発注せず、各branchはcurrent mainから独立に作り、feature branch同士をmergeしない。integration ownerがmechanical conflictを解消してmainへ順に着地させ、task起因redは同じoutcomeのfix-forwardをそのseatの次発注より先に入れる。独自queue、lock service、merge frameworkを新設しない。詳細は[叩き台PR統合決定](docs/reviews/2026-08-10-creator-translation-working-draft-pr-integration-decision.md#並列pr発注loop-v0)

## 外部LLMと検収

- ユーザーが「発注して」「実装を発注」等を依頼動詞として明示した時だけ外部実装を起動する。通常の「実装して」、説明、引用内の語では自動委任しない
- 主担当Codexがbase/cwd、worktree、authority、scope、allowlist、非目標、oracle、fingerprint、diff、最終採否を所有する。意味、owner、原因、再利用、oracleが閉じない`WIDE`は実装担当へ送らない
- 外部CLIは[`run-observed-cli.py`](scripts/run-observed-cli.py)でexact argvを起動し、provider-nativeの構造化途中stream、生stderr、exit／signal、process回収を保存する。呼出側は実行中のeventを観測し、provider固有の最終結果位置まで読む。harnessはJSON意味解釈、worktree、意味判断、採否、session資格を所有しない
- model配分は[可変配分runbook](docs/llm-dispatch-observation-and-allocation-runbook.md)の`ALLOCATION_PROFILE`一つで短waveごとに切り替える。利用枠の`limit group`と独立検収の`model family`を分け、weightは適格候補間のsoft targetに限る。比率達成のため不要なcall、固定fallback、authority移譲を行わない
- 通常channelはOpenAI系をCodex direct、Anthropic系をClaude direct、Cursor first-partyのGrok 4.5／Composer 2.5をCursorとする。Cursor上のthird-party modelは対応direct channelがlimit／capacity／障害で実際に利用不能な時だけ、fresh runへ代用理由とexact IDを記録して使う。Cursor `auto`や黙ったfallbackは使わない
- 総監督はauthority、次粒、owner、scope、oracle、finding処分、最終統合を所有する。order未閉鎖かつ探索範囲boundedならTerraで候補compile、極小closed施工はfresh Spark、通常〜重めのclosed施工はGrok 4.5 non-fastを第一候補とし、Composer 2.5 standardは価格／capacity／task実測の明示理由がある代替施工にする。既にclosedならTerraを省略し、同じoutcome／owner／scope／oracleの短wave内でもreturn後に次edgeを選び直す。固定直列route、自動fallback、同じ施工modelの自己reviewを作らない。詳細は[役割再配置](docs/reviews/2026-08-07-terra-grok-composer-role-reallocation-decision.md)と[LLM役割選択](docs/reviews/2026-08-03-history-calibrated-llm-role-selection-decision.md)
- Claude系のread-only相談／reviewは、値を気分で固定せず視野幅からeffortを算出する。閉じたexact finding照合は`low`、隣接契約までの負例探索は`medium`、未収束のauthority主張／意味owner／writer／原因／reuseの競合調査は`high`、衝突する共有・恒久境界は`xhigh`を基準とし、`max`は理由を記録した例外だけにする。複数file／資料が同じ契約結論を補強するだけなら広域化しない。effortは思考深度であってread拡大許可ではない。Lunaの同一境界修正は`max`を維持してもcapsule、read set、token観測上限を広げない
- `HAZARD_TAG`は視野幅と別軸とし、security、persistence、destructive FS、concurrency、platform、恒久形式の危険だけでeffortを自動昇格しない。hazardは必須負例、task固有の非LLM oracle、platform／実機lane、独立reviewerの要否と構成を強め、競合原因や未決境界が実際に見つかった場合だけ視野幅を再分類する
- 外部model起動前に、実際のcapsule、diff、oracle、許可snippet、想定tool-result cycleから動的context／token予算を記録する。全task共通の固定byte値へ押し込まず、予算超過は圧縮やrepo再読でなく契約境界／finding群の分割信号とする。Claude Code CLIに未提供のtask-budget hard capを実装済みと扱わず、利用できるprovider-native制御、生stream usage、wall回収を区別する。2026-08-03に実測したClaude Code 2.1.216と2.1.220の実binary helpは`--max-turns`を公開していないため、一次資料の記載だけでhard turn capを利用可能と報告したり起動引数へ入れたりしない。`TOOL_TURN_BUDGET`は予定するtool-result cycleと最終回答であり、file数やtool call数から導かず、完了eventのnatural turnを記録する。boundedな合成packetと過去diff再現では`CLOSED=low`を支持したが、未閉鎖packet、`ADJACENT / WIDE / CONFLICTING`、一般的な同等品質へ外挿しない。packet不足はeffort昇格で補わずSolへ戻し、粒を分割する。strict `--json-schema`の全review必須化は別決定とする
- closed施工／reviewでは`READ SET`をexact path、range、snippetまたは一つのevidence envelopeとして固定し、capsule外readを禁止する。予算内で証拠が足りなければ探索を自己拡張せず`CONTEXT_GAP: <exact missing evidence>`を返す。主担当は不足原文を検証してfreshな短waveを作る
- 外部LLM reviewerへは、意味要約でなくexact原文を機械連結した一つのblind evidence envelopeを標準とする。manifestへsource path、range、source hash、抽出に使ったliteral query／anchorとそのscope内の全hit inventoryを記録し、envelope hashは自己参照させず起動logへ記録する。query外の意味的完全性まで証明したと扱わない。関連hitのraw bytesが未収録ならreviewerは`ACCEPT`せず`EVIDENCE_GAP: <path>:<range>`を返し、Solが原文一致をpreflightしてexact rangeだけをfreshな短waveへ追加する。reviewerへ自由repo探索を解禁せず、Codexの推奨結論をenvelope本文へ混ぜない。この共通方式はFable lowで反例捕捉まで実証済みとしてOpus／Grokにも適用し、両者のprovider固有turn、cost、schema遵守率は初回数粒の注記として自然観測する。未較正だけを理由に方式適用を待たない
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
- mainへのマージに事前gateを課さない([段差撤廃決定](docs/reviews/2026-08-10-main-merge-friction-removal-decision.md))。conflict-free mergeは即実行し、Git上の機械的conflictはintegration ownerが解消してよい。stable identity、Document意味、single writer、GPU owner、公開／永続contractのsemantic conflictだけは当該統合を止める。PRは良い塊のlanding envelopeとして使えるがapproval gateにしない。`./scripts/validate.sh local`や`./scripts/check-docs.sh`は事後観測として実行し、redはmain上でfix-forwardする。成果は当日中にmainへ入れ、ブランチ・リポ外workdirに滞留する成果は完了と数えない
- 作業終了時は`./scripts/check-stray-work.sh`で滞留5層(ローカル/リモートブランチ、worktree、リポ外workdir、docsのリポ外パス参照)を観測し、**自分の成果がSTRAYに残っていない状態で終える**。過去の滞留を調べる歴史調査を自分で再発明しない — このスクリプトの出力が正本
- 一つのlaneが既知不具合でredでも全体をgreenと報告しない。実diff、製品route、validation／review、integration、blocker、未実行gateを分離して報告する
- 完了時は実行commandと結果、commit、main統合有無、残存dirty差分、次の一粒と非目標を示す。「動くはず」「たぶん完了」を使わない
