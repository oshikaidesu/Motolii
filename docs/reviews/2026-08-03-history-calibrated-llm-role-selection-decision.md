# 履歴較正によるLLM役割選択

> 2026-08-07追補: 通常施工の第一候補は[Terra / Grok 4.5 / Composer 2.5役割再配置](2026-08-07-terra-grok-composer-role-reallocation-decision.md)が上書きする。Terraはboundedな候補order compile、Grok 4.5 non-fastは通常〜重めのclosed implementation、Composer 2.5 standardは明示理由のある代替施工とする。固定pipelineにはしない。本書の視野幅、effort、blind evidence envelope、別family reviewは継続する。

状態: **決定**（main統合後発効）

日付: 2026-08-03

## 目的

主担当がauthority、次粒、owner、scope、oracle、最終統合を所有する。Terraはboundedな候補order compile、Sparkは極小closed施工、
Grok 4.5 non-fastは通常〜重めのclosed implementation、Composer 2.5 standardは明示理由のある代替施工へ置く。一方で、これらを
全task共通の固定順へせず、taskの判定対象と過去に観測した失敗形から役割を選ぶ。transport、監督責任、採用資格は増やさない。

## 履歴から確認したこと

- Grok reviewはallowlist、base、authority hash、削除数、exact oracle、余分な宣言等を具体的に列挙し、実装がtest greenでも
  必須guard不足をP1としてREJECTした。一方で完了stdoutが空の試行もあり、常時応答を前提にできない
- Claude Opusは未閉鎖の意味、owner、共有契約を実装前にSTOPへ戻す用途と、閉じた実diffの意味監査に適した。実例では
  hidden roleとshare再配分の欠落をP1としてREJECTし、修正後にACCEPTした
- 一行fixtureではGrok preflight 22秒、Spark施工18秒、Opus final 9秒で疎通したが、同一条件A/Bではなく品質・速度改善の
  証明ではない。固定routeの復活根拠にはしない
- 同じOpusを設計相談と最終reviewへ使うと、実装担当と別sessionでも判断の独立性が弱い。family分離を優先する
- LLM間の誤りは相関する。性能、安全性、永続性、platform挙動はmodelの賛同でなくbench、negative test、schema fixture、
  OS oracleへ置く
- `gpt-5.6-luna` maxへ6文書と履歴を広く探索させた試行は正しく`STOP`したが、360.865秒、input 1,535,651、stdout
  1.53 MBとなり、長いcommit SHAを2回誤った。安価なmodelでも全履歴の反復読込を許せば運用costと転記riskが膨らむ
- 同じLuna sessionへ検証済みの小さなcapsuleだけを渡した二turn試行は、合計約39.5秒、input 33,616、cached input
  27,904、output 1,964で、`CU-201P = WAIT_TARGET`を発明で迂回せず、履歴記述と現行状態を区別した。これは接続flowの
  bounded fixtureであり、一般的な品質優位や実料金の証明ではない
- Claudeの現行一次資料では、effortはresponse全体の思考、text、tool callへ効くsoftな行動信号である。`low`は単純・速度重視、
  `xhigh`は長時間agentic探索、`max`はtoken支出を制約しない最高能力向けであり、高effortをread scope拡大の権限にしてはならない。
  Opus 5では`low / medium / high / xhigh / max`を利用できる
- Claude Code利用者には、具体的なpromptでは`low`／`medium`の方が単純作業で脱線しにくいという報告と、長いcontextでは
  startup instructionを見落とすという反例の両方がある。これは採用oracleではなく、Motoliiの同一capsuleでeffort sweepを
  続ける理由としてのみ使う

根拠は[監督ループ速度支配項観察](2026-08-01-supervision-loop-cost-driver-observation.md)と
[SD-02G Opus検収記録](2026-07-30-sd-02g-product-host-layout-geometry-implementation-decision.md)、
[Claude low CLOSED review較正観察](2026-08-03-claude-low-closed-review-calibration-observation.md)、
[blind evidence envelope反例観察](2026-08-03-blind-evidence-envelope-counterexample-observation.md)に置く。過去のraw streamは
履歴確認用であり、新しいreceipt DBやmodel scoreへ集約しない。

Claude effortの一次資料は[Anthropic Effort](https://platform.claude.com/docs/en/build-with-claude/effort)と
[Steering thinking](https://platform.claude.com/docs/en/build-with-claude/thinking-steering-and-cost)、利用者報告は
[specific promptでlowが脱線を減らした例](https://www.reddit.com/r/ClaudeCode/comments/1rrjkus/claude_code_defaults_to_medium_effort_now_heres/)と
[low常用／long-context反例を含む議論](https://www.reddit.com/r/ClaudeAI/comments/1uuvthn/how_does_effort_in_claude_models_affect_the_output/)を参照する。

## 通常姿勢

1. 総監督は完全model ID `gpt-5.6-sol`、通常`model_reasoning_effort=medium`を基準とし、authority、次粒、owner、scope、oracle、
   外部model選択、review findingの処分、最終統合を所有する。riskに応じたeffort増加は許すが、Sol自身を独立検収票に数えない
2. 主担当Codexは起動前に、利用者outcome、base/HEAD、現行authority、状態、exact target、owner/write route、oracle、非目標、
   STOPと、それらを直接示す少数の検証済みsnippetをcontext capsuleへまとめる。外部modelへrepo全体、全正本、会話全履歴を
   初動から再読させない。不足は推測で埋めず、Codexへ返して検索結果を更新する
3. 一sessionは一契約境界または同じoutcome、owner、scope、oracleに閉じた短いwaveだけを扱う。その四点が不変な間だけresumeし、
   統合、STOP、境界変更のいずれかで閉じる。新しい粒はfresh sessionとfresh capsuleから始める
4. 長期状態はGit、現行正本、decision index、implementation ledger、raw実行logへ置く。会話履歴、session token、modelの記憶を
   authority、採用DB、project memoryにしない
5. order未閉鎖かつ探索範囲boundedならCodex directのTerraで候補orderをcompileする。既にclosedならTerraを省略し、極小施工は
   完全model ID `gpt-5.3-codex-spark`、通常〜重めの施工はCursor Grok 4.5 non-fastを第一候補にする。Composer 2.5 standardは
   価格、capacity、task実測の明示理由がある時だけ選び、自動fallbackにしない
6. Spark、Luna、Sol、Terraは同じOpenAI familyなので相互に独立検収者とはみなさず、Grok施工後にGrokをreviewerへ再利用しない。
   独立reviewが必要な施工では、設計・施工へ深く関与していないClaude等の別familyをfresh read-only sessionで選ぶ

## 視野幅とClaude effort

effortとread breadthを別軸にする。effortはmodelが与えられた証拠内で使う思考深度であり、repo、正本、履歴、toolを追加で
開く権限ではない。SolはClaude起動前に次の四状態をコード事実つきで一つ選び、完全model IDと一緒にraw logへ残す。

| 視野幅 | 判定 | Claude effortの基準 | 典型用途 |
|---|---|---|---|
| `CLOSED` | 複数file／資料を含んでも同じ契約結論、唯一のwriter、局所原因、閉じたoracle／reuseへ収束し、exact findingまたは固定diffだけを照合する | `low` | finding closure、allowlist／hash／label照合、docs-only整合、固定source転記確認 |
| `ADJACENT` | 一契約は閉じているが、隣接owner／負例／回帰を一段だけ探索する | `medium` | 通常の一契約review、新規P1探索、局所原因比較 |
| `WIDE` | authorityの主張が競合する、複数の意味owner／writer候補や原因候補が未収束、部分oracle、未選択reuseがあり、実装前に閉鎖条件を探す | `high` | read-only調査、設計反例、競合候補の原因収束 |
| `CONFLICTING` | 正本衝突、共有公開境界、恒久契約、CodexとClaudeの結論衝突がある | `xhigh` | Fable／Opusの反対側相談、恒久境界の選択肢比較 |

file数、資料数、crate数だけで`WIDE`へ上げない。複数資料が同じ意味を相互補強し、owner／writer／契約結論が一つなら
`CLOSED`になり得る。一方、同じ名前でも異なる正本主張、複数writer候補、両立しない原因仮説、採否未決のreuseが残る場合は
`WIDE`以上とする。数ではなく未収束の意味分岐を分類根拠にし、その分岐をcapsuleへ列挙する。

`max`は視野幅から自動選択しない。能力上限を試すeval、利用者が明示した非cost-sensitiveな難問、または`xhigh`で残った
具体的な反証だけに理由を記録して使う。通常reviewを`max`へ置かず、`low`でschema違反、証拠不足、見落としが出た時は
同じsessionへ全文を足さない。Solが分類とcapsuleを直し、同じmodelのfresh sessionを`medium`以上へ明示昇格する。

`HAZARD_TAG`と視野幅も別軸にする。security、persistence、destructive FS、concurrency、platform、恒久形式の危険があっても、
意味、owner、原因、契約、oracleが閉じていれば、それだけを理由にClaude effortやread breadthを自動昇格しない。hazardは
必須負例、task固有の非LLM oracle、platform／実機lane、独立reviewerの要否と構成を強める。hazard調査によって競合原因、
部分oracle、未決共有境界が実際に見つかった場合だけ、そのコード事実から視野幅を再分類する。

Grok 4.5を使えない契約または同一契約内finding修正でLunaを明示選択する場合、`model_reasoning_effort=max`を候補にできる。
ただしこれは思考深度の選択だけであり、capsule外read、repo横断探索、複数契約施工、無制限tool turnを許さない。

## 動的context／token予算と分割

全task共通の固定token値を正本にしない。Solは起動前に、実際に渡す一意なartifactから次を計測し、同じ用途・model・effortの
raw logにある直近分布と比較して予算を決める。

- `CAPSULE_BYTES`: outcome、authority、owner、scope、oracle、STOP
- `DIFF_BYTES`: review対象の実diff。施工前相談では0
- `ORACLE_BYTES`: test結果、負例、非LLM測定の必要部分
- `AUTHORIZED_READ_BYTES`: 許可した一意なsnippet／file範囲の合計。全文fileを自動加算しない
- `TOOL_TURN_BUDGET`: 許可したtool-result cycleと最終回答の予定。同じfileの再読も実際のcycleとして観測するが、file数や
  tool call数から機械算出しない
- `EXPECTED_OUTPUT_BYTES`: schema、finding上限、必要な根拠行から導く

予算は「modelが読める最大context」ではなく、この粒の判定に必要な証拠量から決める。起動前に既に予算を超える、同じfileや
問いを反復しないと閉じない、または一つのoutput schemaへ異なるowner／oracleが混ざる場合は、圧縮して押し込まず契約境界、
finding群、調査問いのいずれかで分割する。外部modelが不足を返した場合も、自由探索を解禁せずSolが追加証拠を検証して次の
短いwaveを作る。

Claude Code CLIは2026-08-03時点でMessages APIのbeta `task_budget`を提供しないため、task budgetによるhard capを実装済みと
報告しない。[Task budgets](https://platform.claude.com/docs/en/build-with-claude/task-budgets)とCLI一次資料は実binaryの提供機能と
分けて扱う。Claude Code 2.1.216と同日npm latest 2.1.220の実binary helpは`--max-turns`を公開していなかったため、一次資料の
記載だけでhard turn capを利用可能と報告したり起動引数へ入れたりしない。入力artifactの事前上限、provider-native stream、
`--max-budget-usd`等の実binaryで確認した制御、wall回収、完了後usage／natural turnを別々に記録する。
同じ役割の代表粒で`low / medium / high`を比較し、`CONFLICTING`の代表粒では`xhigh`も含めて、P0/P1 recall、false finding、
schema成功、wall time、tool turn、input／output／cache tokenを測る。品質gateを外した短縮値を採用根拠にせず、分布が得られたら
固定秒数でなく役割別の観測分布から上限を更新する。代表実測では通常回答が`Read → answer`の2 turn、strict schemaが
`Read → StructuredOutput → completion`の3 turnとなり、mediumで同じfileを再読して3 turnになった例もあった。turnはfile数でなく
provider eventから観測する。boundedな合成packetと保存済み過去diffの再現では、`CLOSED=low`が既知のACCEPT／REJECTを再現した。
これは`CLOSED`の通常候補を支持する較正であり、未閉鎖packet、実施工の全分布、`ADJACENT / WIDE / CONFLICTING`、または
medium／highとの一般的な同等品質を証明しない。packetが不完全ならeffortを上げて推測させずSolへ戻し、capsule修正または粒分割を
行う。strict `--json-schema`は構造化出力に成功したがtool-result cycleを一つ加えるため、全review必須化は別決定とする。

## 外部reviewer共通のblind evidence envelope

外部LLM reviewerへは、複数sourceを自由にReadさせる代わりに、判定に使うexact原文を機械連結した一つのblind evidence envelopeを
標準で渡す。これは意味を要約するartifactではなく、探索空間を一契約境界へ閉じるartifactである。Codexの推奨結論、採否、
未検証の解釈を本文へ混ぜない。

manifestはsource path、exact range、source SHA-256、抽出に使ったliteral query／symbol／anchor、そのscope内の全hit inventoryを
持つ。envelope SHA-256は自己参照させず起動logへ記録する。Solは起動前にsource bytes、range、hash、inventory、envelope bytesの
一致を機械確認する。inventoryは
記録したquery／anchor scopeのcoverage witnessであり、query外を含む意味的完全性の証明ではない。query自体がoutcome、authority、
変更symbol、既知の競合語から導かれているかはSolが所有する。

関連hitのraw bytesがenvelopeに無い場合、reviewerは不足を推測したり`ACCEPT`したりせず、`EVIDENCE_GAP: <path>:<range>`を返す。
Solが要求と現行sourceを再照合し、必要なexact原文だけを追加したfreshな短waveを作る。自由repo探索、全文file追加、同sessionへの
継ぎ足しで解消しない。要求範囲が広い場合は、全hit inventoryを使って候補rangeを先に狭める。

この方式はFable lowで、複数rangeの個別Readに対する単一envelopeのturn／wall／cost削減と、未収録の競合authorityを
`EVIDENCE_GAP → fresh wave → REJECT`へ送る反例捕捉を実証した。構造はprovider固有能力に依存しないため、OpusとGrokを含む
外部LLM reviewerへ共通適用する。Opus／Grokで未較正なのはprovider固有のnatural turn、cost、schema遵守率、permission／最終event
位置の効果量であり、方式の適用条件ではない。初回数粒で自然観測し、問題が出たproviderだけ補正する。

## 短い実行flow

1. Solが正本、diff、コード事実を一度だけ確認する
2. finding closure表または判定問いを作り、視野幅とeffortを算出する
3. exact原文、hash、全hit inventoryを持つ一つのblind evidence envelopeと動的予算を作る
4. fresh外部modelを途中stream付きで起動し、不足は追加探索でなくexact rangeの`EVIDENCE_GAP`としてSolへ返させる
5. reviewerの`ACCEPT / REJECT / NO_VERDICT`、P0/P1、scope、非LLM oracleをSolが再照合する
6. 採用、同一境界の修正、粒分割、局所STOPのいずれか一つへ処分する

## 役割選択

| taskの状態・判定対象 | 第一候補 | 用途 | 最終reviewer |
|---|---|---|---|
| authority、次粒、owner、scope、oracle、最終統合 | Sol medium以上 | 総監督、capsule作成、外部model選択、finding処分、最終採否 | SolはOpenAI family施工の独立reviewを兼ねない |
| authority衝突、意味、owner、原因、共有契約が未閉鎖 | Sol medium以上、必要ならClaude Opus read-only | 反例、STOP、選択肢、閉鎖条件 | 閉鎖後に実装するなら関与していない別family |
| authorityは閉じたがscope、allowlist、exact target、負例が複雑 | Terra read-only | boundedな候補order compile、漏れの列挙 | Terraは採否しない。施工後は未関与の別family |
| 一契約境界に閉じた初回の機械施工 | Spark | 指定pathの変更と指定試験 | 設計へ未関与のfresh Claude等、別family |
| 通常〜重めのclosed implementation、同一境界のfinding修正 | Grok 4.5 non-fastを第一候補。別family制約やtask適合でLuna／Composerも選べる | 指定pathの変更と指定試験 | 施工・設計へ未関与の別family |
| main統合直前、複数粒の整合 | Sol medium以上 | authority、非目標、diff、oracleの全体照合 | SolはOpenAI family施工の独立reviewを兼ねない |
| 実diffのscope、削除、guard、負例を詳しく監査 | 施工に未関与のTerraまたは別family reviewer | concrete diff audit | reviewer自身は採否しない |
| 実diffの意味、owner、既存契約との統合を監査 | Claude Opus read-only。`CLOSED=low`、`ADJACENT=medium`を通常候補にする | semantic final audit | Claude自身は採否しない |
| 性能、安全性、永続形式、platform correctness | 非LLM oracle | bench、negative test、schema/OS fixture | LLMは補助監査のみ |

## 分岐規則

1. Codexが先にauthority、base/cwd、worktree、scope、oracle、実装担当候補を確認する
2. 主担当が閉じた極小の機械taskはSpark、通常〜重めのclosed taskはGrok 4.5へ直接送れる。order未閉鎖時だけTerraを候補compileへ使い、全modelを通すことを完了条件にしない
3. 主担当だけで閉じない意味をClaudeへ、boundedな境界列挙をTerraへ送る。複数が必要なら独立に並べず、先の回答を主担当が正本へ
   再照合して残った問いだけを次へ送る
4. 設計・契約閉鎖へ深く関与したmodel familyは同じtaskの最終reviewer候補から外す
5. modelのcapacity／rate limitは観測失敗として一度Codexへ戻す。同じtask境界がなお有効なら、利用可能なmodelを新しい
   実装担当として明示選択できる。CLIで確認した完全model ID、変更理由、fresh sessionをlogへ残し、自動retry列、alias推測、
   途中sessionのmodel間引継ぎ、無記録のfallbackは行わない
6. Grokのtimeout、CLI失敗、空の完了結果は観測失敗としてCodexへ戻す。未完了streamの一時的なstdout空とは区別し、
   別modelへ黙ってfallbackしない
7. reviewerはfindingを列挙するだけでscope、order、実装、採用を増やさない。最終採否はCodexが非LLM oracleと合わせて行う
8. Grok、Composer、LunaまたはSparkのtool call、読込量、転記誤り、wall timeがcapsuleの意図を超えて膨らんだ場合は、同じsessionへ全文を追加せず閉じる。
   Solがauthority検索とcapsuleを修正し、同じ問いと証拠を反復しない

## 非目標

- `Grok → Spark → Opus`または`Claude → Spark → Grok`を固定routeにすること
- `Sol → Spark → Claude → Sol`または`Sol → Luna → reviewer → Sol`を全task共通の固定routeにすること
- modelごとのscore、学習DB、append-only receipt、retry状態機械を作ること
- model失敗時に無条件で特定modelへ切り替える固定fallback chainを作ること
- 過去のwall timeだけでmodelを格付けすること
- Claude/Grokの賛同をauthority、ユーザー権限、採用資格にすること
- 一つのLuna sessionへproject全履歴を保持し続けること

## Fable read-only経路

Fableは大地図、長期展望、複数仕様衝突、共有公開境界、恒久契約、CodexとOpusの結論衝突、または
一般機構の既知routeが具体的反証で尽きた時の一回の取りこぼし検査だけに使う。正規model IDは
`claude-fable-5`で、Claude Code CLIから薄いCLI harnessを介してread-onlyで直接起動する。
Cursorの同名modelや別modelへ黙ってfallbackせず、編集、Bash、commit、push、外部model起動、再委任を許可しない。
出力は助言であり、Codexが正本、現行コード、取得済み一次資料へ再照合して採否する。
