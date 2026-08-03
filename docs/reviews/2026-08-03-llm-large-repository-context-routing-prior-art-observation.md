# LLM大規模repository開発のcontext routing先例調査

日付: 2026-08-03

状態: **観察**

対象: AGENTS入口、現行docs、コード検索、依存関係、LLM向け文脈カプセル

関連: [レビュー文書の規律](README.md)、[決定逆引き台帳](../decision-index.md)、
[歴史価値回収の意味グラフ補助境界](2026-07-23-historical-semantic-graph-recovery-tooling.md)、
[現行監督ループ](2026-07-25-opus-spark-grok-supervision-loop-decision.md)

## 1. 状態と非目標

本書は、大規模repositoryをLLMで開発する際の文脈取得について、2026-08-03時点の公開先例と
Fable 5のread-only助言を整理した**調査メモ**である。Motoliiの検索方式、metadata schema、
embedding model、外部service、AGENTS構成を採択する決定ではない。反対側レビューと比較実験を
通さず、ここから運用規則、実装発注、常設index、必須gateを直接導入しない。

非目標は次のとおり。

- 全文書をvector databaseへ移すこと
- semantic scoreをauthority、現行性、採用可否へ変換すること
- 現行のdecision-index、spec、ledger、Git、コード、試験を検索indexで置換すること
- 歴史回収専用HVR-D01〜D04を通常の現行実装検索へ黙って一般化すること
- AGENTSを直ちに削減、分割、再編すること

## 2. 観測した問題

現checkoutでは`docs/**/*.md`が289件、45,854行あり、うち`docs/reviews/**/*.md`が231件である。
入口候補だけでも`AGENTS.md` 170行、`docs/README.md` 182行、`docs/decision-index.md` 184行、
`docs/implementation-ledger.md` 258行ある。件数や行数だけで問題は証明できないが、毎taskで入口、
spec、review、ledgerを広く再読すると、次の失敗仮説が成立する。

1. taskに無関係な規律と履歴がcontextを占め、重要な現行targetの信号を薄める。
2. 同じ主題の観察、比較中、決定、撤回、停止線を同列に取得し、古い意味を現行へ混ぜる。
3. 文書間と文書→コード→試験の依存が明示されず、読む順序を各LLMが再推測する。
4. 利用者が一度に多くの意図を提示した時、検索前の解釈が誤ると、正しい資料へ到達しない。
5. 全文読込を避けるために純粋なvector検索へ寄せると、spec ID、型名、pathなどのexact targetを
   逆に落とす。

## 3. 公開先例

### 3.1 短い入口地図とprogressive disclosure

OpenAIの[Harness engineering](https://openai.com/index/harness-engineering/)は、agent-firstな
大規模開発の実践で、一つの巨大な`AGENTS.md`がcontextを圧迫し、指示の優先度を失わせ、腐敗し、
機械検証しにくかったと報告する。代替は、約100行のAGENTSを目次にし、構造化docsをsystem of
recordへ置き、必要時だけ深い正本へ移るprogressive disclosure、link・鮮度・依存方向のlint、
定期的なdoc gardeningを組み合わせる方法である。これは単一組織の実践報告であり、Motoliiでの
因果や最適規模を証明しないが、今回の症状に最も直接的な先例である。

Anthropicの[Effective context engineering for AI agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
も、contextを有限資源として扱い、最小の高信号token集合を維持すること、固定入口と`glob`／`grep`
によるjust-in-time探索を併用すること、長期taskではcompactionや外部noteでcontext汚染を抑えることを
提案する。runtime探索は遅く、toolと探索heuristicに依存するため、入口情報をゼロにする根拠ではない。

### 3.2 lexical、semantic、long contextのどれか一つへ固定しない

Anthropicの[Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval)は、
embeddingがexact phraseや識別子に弱いためBM25とembeddingを併用し、chunkへ由来文脈を付加する方式を
報告する。同社評価ではcontextual embeddingとcontextual BM25の併用がtop-20 retrieval failureを
49%減らし、reranking込みで67%減らした。ただしvendor内評価であり、Motolii文書への再現性や
authority誤認率を示さない。

[CodeRAG-Bench](https://arxiv.org/abs/2406.14497)は、良いretrievalがコード生成を改善する一方、
retrieverは語彙重複の少ない関連codeを見落とし、generatorは取得contextを統合できない場合があると
報告する。したがってsemantic indexの存在を品質証明にせず、Motoliiのgold queryでrecallを測る必要が
ある。

[Self-Route](https://arxiv.org/abs/2407.16833)は、十分な資源がある条件ではlong-context LLMが平均で
RAGを上回る一方、RAGは大幅に低costであり、queryごとにrouteすると品質とcostを両立できると報告する。
これは「検索結果だけを常に渡す」案への反例である。検索の確信度が低い、正本が衝突する、共有境界を
扱う場合は、関連全文または広いcontextへ昇格できなければならない。

### 3.3 dependency mapと単純な実装flow

[CodePlan](https://arxiv.org/abs/2309.12499)はrepository-wide変更に対し、変更が影響するentityを反復して
解析し、依存に応じてplanを更新する方式を示す。[RepoGraph](https://arxiv.org/abs/2410.14684)はrepository
構造とcross-file関係をgraphとして既存agentへ渡し、SWE-benchやCrossCodeEvalで改善を報告する。
[Aider repository map](https://aider.chat/docs/repomap.html)は実装先例として、file dependency graphを
rank付けし、token budget内で重要なclass、function、signatureを短いmapへ投影する。いずれもMotoliiの
文書状態語彙やauthorityを自動判定するものではない。

一方、[Agentless](https://arxiv.org/abs/2407.01489)は、複雑な自律agent frameworkを使わず、
localization、repair、validationの単純な三段でもrepository修復で競争力を持てると報告する。これは
検索問題を新しい大型frameworkや状態機械へ拡大する前に、薄いroutingと評価から始める根拠になる。

### 3.4 意図の曖昧性と継続評価

[ClarifyGPT](https://arxiv.org/abs/2310.10996)は、要求の曖昧性を検出し、実装結果を変える質問だけを返す
ことでcode generationを改善したと報告する。Motoliiでは毎回質問を増やすのでなく、owner、成果物、
公開境界、完了条件の読みが複数あり、検索routeが変わる場合だけ短い意図packetまたは確認へ移す候補に
なる。

[Seven Failure Points When Engineering a Retrieval Augmented Generation System](https://arxiv.org/abs/2401.05856)
は、retrieval品質を導入時だけで閉じず、運用中に観測し続ける必要を示す。Motoliiでもindex構築成功、
検索結果が返ること、実装に必要な正本が揃うことを別のoracleにしなければならない。

### 3.5 原子的knowledge graphと時間状態

[Basic Memoryのknowledge format](https://docs.basicmemory.com/concepts/knowledge-format)はplain Markdownへ
原子的なobservation、typed relation、任意metadata、stable permalinkを持たせる。
[semantic search](https://docs.basicmemory.com/concepts/semantic-search)は全文／vector／hybrid検索とmetadata
filterを提供する。これは「一文書を一つの意味単位として検索しない」既製形式の候補だが、Motoliiの
現行性、authority、ownerを自動導出することや、既存docsを無加工で高精度に検索できることは証明しない。

[Graphiti](https://github.com/getzep/graphiti)はevent provenance、valid timeとtransaction time、typed
entity/edge、増分更新を持つ時間knowledge graphである。一方でLLMによるentity/fact抽出と外部graph DBを
前提にする。[Microsoft GraphRAGのDRIFT search](https://microsoft.github.io/graphrag/query/drift_search/)も
global summaryと局所探索を組み合わせるが、LLM生成graph／summaryを検索経路へ入れる。いずれも大域的な
質問には候補になる一方、派生意味を現行正本と誤認しないこと、再生成時の非決定性、初期・更新costを
Motoliiで解決した先例ではない。

## 4. Fable 5相談

Fable 5へ、全資料再読の遅延、依存不明、意図誤認、純粋vector検索の危険、既存HVR境界、同日比較の
反例を含むpacketをread-onlyで渡し、公開候補の広域検索と圧縮再評価を依頼した。助言は次へ収束した。

1. 最初はlexical検索、status/metadata filter、依存graph、progressive disclosureを組み合わせる。
2. vector検索は測定されたrecall gapを補う場合だけ追加し、authority判定へ使わない。
3. gold query、recall、旧文書誤混入、token、latencyを比較してから運用へ採る。
4. intent ambiguityがrouteを変える時だけ明示的に解消する。
5. retrievalの信頼度が低い時は、検索結果へ閉じず広い正本へ戻る。

相談実行では`claude-fable-5`、`--effort xhigh`、`--permission-mode dontAsk`、read-only tool
allowlist、`stream-json`観測を用いた。Web結果は検索snippet水準で、全文取得済み一次資料と同じ強度へ
上げず、上記で使う公開資料はCodexが別途確認した。FableはBasic Memory hybrid searchを自然言語入口、
typed graphを状態filterとする直列構成を最有力としたが、後述6.4の実測前の助言であり、そのまま採らない。
Fable出力は本書のauthorityでも独立レビューでもない。

## 5. Motoliiへの転移仮説

先例から直ちにvector databaseを採るのでなく、次の**authority-aware repository map**を検証仮説と
する。

```text
利用者の成果・語彙
  → exact lexical search（ID、型、path、用語）
  → status / owner / supersedes filter
  → docs・code・testのdependencyを限定展開
  → 出典pathとfingerprintつきauthority capsule
  → 実装前の不足・衝突判定
  → 必要時だけsemantic補完または関連全文へ昇格
```

最小capsuleは、少なくとも次を区別する候補とする。

- 利用者成果と検索語。検索語は利用者文とMotolii exact語彙を分ける
- 現行authorityと状態（観察／比較中／決定／棄却／停止）
- owner、write route、code target、test target
- `depends_on`、`supersedes`、衝突または未統一
- 各引用のpath、対象範囲、Git fingerprint
- 未取得範囲、検索確信度、全文へ昇格する条件

semantic score、LLM要約、graph rankは候補発見に限る。`decision-index`と正本の状態、現行コード、試験、
Git fingerprintより強い裁定へしない。既存HVRは固定歴史corpusの回収専用であり、通常の現行実装route
は責任を共有してもcorpus、receipt、完了条件を共有しない。

## 6. 比較実験案

実装へ入る前に、過去の完了済みまたは停止済みgrainから30〜50 queryを固定する。各queryに、人手で
確認した現行authority、必須code target、必須test、混入してはいけない撤回／旧route、正しい
dispositionをgoldとして持たせる。

比較対象は次の順とする。

| variant | 取得方式 | 目的 |
|---|---|---|
| A | 現行の入口・spec・review・ledger広読 | baselineのtoken、時間、誤認を測る |
| B | lexical検索＋短い入口map | 全文読込なしでexact targetを保てるか |
| C | B＋status/supersedes/owner filter＋dependency 1-hop | 旧文書混入と依存欠落を減らせるか |
| D | C＋semantic retrievalのshadow候補 | lexicalで落とした関連だけを補えるか |
| E | CまたはDから関連全文へ昇格 | low-confidence時の品質を回復できるか |

最低限の指標は次とする。

- authority、code target、test targetの`recall@k`
- 撤回済み、別owner、無関係phaseのfalse inclusion率
- capsule token数、検索時間、LLMが実装判断へ到達するwall time
- owner、公開境界、完了条件のintent mismatch率
- `PASS / REDUCE / RESOLVE / STOP`のgold一致
- 同じqueryを再実行した時の取得差と、index更新後の回帰

採用条件の候補は、CがAよりtokenと時間を減らしながらauthority/code/test recallを落とさず、旧文書の
誤混入を増やさないこととする。DはCで実測したrecall gapを改善し、false inclusionを悪化させない場合
だけ残す。単発の高速化、検索結果の見栄え、LLMの主観評価だけでは採択しない。

### 6.1 比較前に固定する最低限の環境

比較対象ごとに前処理や対象corpusが違うままでは、取得品質より環境差を測ってしまう。そこで本比較は、
同一のdetached worktree、同一HEAD、同一query／gold、同一出力上限を使い、cold startとwarm queryを
分ける。生成cache、tool設定、履歴は一時directoryへ閉じ、製品checkoutへ導入しない。

最低限の三層は次とする。これらを一つの新しい検索frameworkへ統合することは比較の前提にしない。

| 層 | 最小tool | 証明する範囲 | 証明しない範囲 |
|---|---|---|---|
| authority候補発見 | `rg`＋`decision-index`／spec／ledgerの状態語彙 | exact ID、path、状態語、候補文書 | 意味的同義語の完全recall、最終採否 |
| code topology | Aider repository map、4,096 token上限 | 主要file、型、signature、中心性の高い接続 | 全参照、文書authority、状態 |
| exact code navigation | Serena read-only project、Rust／TypeScript／Markdown | symbol定義、参照元、見出し階層 | semantic score、文書の現行性、最終採否 |

2026-08-03の隔離smokeでは、Aider 0.86.2が970 fileを解析し、warm mapを約3.0秒、約26.5 KiBで
生成した。`PipelineCache`の定義と中央plugin契約への接続はmapへ入ったが、全参照一覧ではない。
Serenaは公式repositoryのcommit `29d07d4f6b7a04a0db3981d6c6be6f736cfb44d2`を固定し、Rust 283 file、
TypeScript 65 fileを初期indexした。初期準備は約24.8秒、project server起動後の最初のRust symbol
queryは24.1秒、同一warm queryは0.34秒だった。`find_referencing_symbols`は参照を関数／method単位で
返した。Markdown language serverは`decision-index`の見出し階層を返したが、`Export`の広域symbol
検索にはJavaScriptの同名symbolも混ざったため、authorityの初動検索を置換できない。

対照の`rg`は同じ`PipelineCache` exact queryで77行を約0.06秒、Save／Export停止状態の複合queryで
10行を約0.03秒で返した。これはlexical baselineの速度を示すだけで、77行がsemantic referenceである
ことや10行から現行authorityが自動確定することを示さない。このsmokeはtoolの実行可能性と役割差の
確認であり、A〜Eの優劣や採択結果ではない。

文書の同義語recall gapに対しては、Markdown向け既存OSSのQMD 2.1.0も同じ隔離worktreeでshadow確認
した。tracked docs 260件のBM25 indexは約1.3秒で作れたが、利用者表現
「動画編集プロジェクトを保存して再起動後に戻す入口」は0件だった。日本語を含む比較のため公式推奨の
Qwen3-Embedding-0.6B Q8 modelを選ぶと、model単体が639.15 MBであり、取得後も717 chunk中96件の時点で
残り約43分と表示されたため停止した。既定の英語寄りmodelへ替えて速度だけを測ることもしない。
QMDは同一index上のBM25／vector／hybrid比較とbenchmark機能を持つ有力な後続候補だが、現時点の
「最低限環境」へ常設するには重すぎ、recall改善も未確認である。

### 6.2 五つの固定queryによる最小比較

同じdetached HEAD上で、検索結果を見る前に次のfile-level goldを固定した。goldはretrieverへの入力に
せず、結果の採点だけに用いた。

| query | 利用者表現またはexact symbol | gold |
|---|---|---|
| Q1 | プロジェクトを保存して終了し、再起動後に同じ編集状態へ戻す入口 | `2026-07-22-m3-comfortable-use-granulation.md`、`2026-07-24-m3-vertical-slice-execution-decision.md` |
| Q2 | 固定Reactモックを作り直さず製品へ持ってくる規則 | `2026-07-22-m3-react-product-asset-promotion-contract.md` |
| Q3 | Rerun先例をMotoliiへ移す順序 | `2026-07-20-rerun-learning-transfer-plan.md`、`2026-07-20-rerun-source-asset-inventory.md` |
| Q4 | `PipelineCache`の定義と参照 | `crates/motolii-gpu/src/pipeline_cache.rs`とsemantic reference |
| Q5 | `DocumentEditQueue`の定義と参照 | `crates/motolii-ui/src/document_edit_runtime.rs`、製品consumer `crates/motolii-ui/src/app.rs` |

文書queryでは、利用者の生語だけをOR検索する`raw`と、既知のrepo語彙をCodexが足した`expanded`を
分けた。これを分けないと、retrieval性能とCodexの意図翻訳性能を混同する。

| query | `rg raw`候補／gold | `rg expanded`候補／gold | Serena heading raw | Serena heading expanded |
|---|---:|---:|---:|---:|
| Q1 | 192 file／2/2 | 7 file／2/2 | 7 heading／0/2 | 1 heading／1/2 |
| Q2 | 118 file／1/1 | 22 file／1/1 | 33 symbol／1/1 | 1 heading／1/1 |
| Q3 | 232 file／2/2 | 39 file／2/2 | 10 heading／2/2 | 0 heading／0/2 |

`rg`は各文書queryを約0.012〜0.024秒で返した。rawはgold recallを保ったが118〜232 fileへ膨らみ、
毎taskの全文読込を避ける候補集合として大きすぎる。expandedは7〜39 fileへ減らしたが、その改善は
検索器でなくCodexが`project lifecycle`、`CU-G04`、`直接所有移管`等を正しく推測できたことに依存する。
今回問題にしている意図誤認が起きれば、この段でgold自体を落とせる。

SerenaのMarkdown language serverはheading階層を返すため、正しい正本語彙を既に知るQ2では1件へ
絞れた。一方Q1の生語`保存`は無関係な7 headingだけを返しgold 0/2、repo語彙`project lifecycle`でも
1/2だった。Q3はrawの固有語`Rerun`で2/2を拾ったが、別の正しそうな展開`Rerun学習`では0/2だった。
したがってMarkdown symbol検索をauthority初動へ使わず、lexical候補の内部navigationへ限る。

コードqueryの結果は次のとおり。

| query | `rg` | Aider repo map | Serena |
|---|---|---|---|
| Q4 | 0.011秒、77行、21 file | 4.59秒の4,096-token map内に定義と中央plugin契約を収録 | cold 19.30秒、warm definition約0.34秒、reference 2.39秒、20 file |
| Q5 | 0.010秒、12行、2 file | 同じmap内に`DocumentEditQueue` implと主要methodを収録 | warm definition 0.39秒、reference 0.15秒、2 file |

Aider map全体は833行、26,338 byteで、Q4/Q5のownershipと中央接続を一度に把握できたが、全参照を
証明しない。SerenaはQ4で20 fileのsemantic referenceを返し、`rg`だけが拾った残り1 fileは
`crates/motolii-plugin/tests/conformance.rs`内の文字列fixtureだった。Q5では定義fileと製品consumerの
2 fileを一致させた。exact symbolが分かった後の参照分離はSerenaが最も強い。

この5 queryだけから一般的なrecall値は主張しないが、最低限のrouteは反証できた。文書とコードを同じ
retrieverへ統一せず、文書は`decision-index`を先頭にlexical候補を状態で絞り、コードはAider mapで
owner候補を得てSerenaで定義・参照を閉じる。利用者語からrepo語彙への展開は検索結果ではなく仮説として
複数保持し、一つの展開が0件でも別展開と`decision-index`検索を続ける。semantic indexはこのrouteの
実測missを集めてから再評価する。

### 6.3 property graph型retrievalの隔離比較

Neo4j製品の運用costを先に混ぜず、同じdetached HEADからPython標準ライブラリだけで一時in-memory
property graphを作った。tracked Markdown 260 fileをnode、Markdown link 1,825本をedge、
`decision-index` 161行を状態つきdecision nodeとした。graphと実行scriptはrepo外の一時領域だけに置き、
tracked fileや製品checkoutへindex／cacheを作らなかった。

最初に全Markdown linkを無型edgeとして2-hop展開すると、gold recallは戻る一方、候補がQ1 45件、
Q2 42件、Q3 14件まで再膨張した。DB backendでなくedge意味が汚染を支配する反例である。そこで次の
決定的edgeだけをtyped traversalへ許した。

- `decision-index`の正本link列
- 文書冒頭8行以内の来歴／関連link
- linkと同じ行に`正本 / 正とする / 優先 / 併読`が明記されたlink

`決定 / 契約 / 計画 / 仕様 / 関連`のような一般語まで型判定へ入れると候補が増えたため除外した。
LLM推論edge、embedding、要約、手書きのquery別edgeは使っていない。query seedは前節と同じ
`raw / expanded`を使い、decision行をIDFつきterm一致で上位3件へ固定した。

| query | expanded lexical | typed graph 1-hop | typed graph 2-hop |
|---|---:|---:|---:|
| Q1 Save/reopen | 7候補、2/2、状態汚染0 | 4候補、1/2、0 | 25候補、2/2、2 |
| Q2 React直接移管 | 22候補、1/1、状態汚染2 | 6候補、1/1、1 | 12候補、1/1、2 |
| Q3 Rerun転移 | 39候補、2/2、状態汚染4 | 3候補、1/2、0 | 11候補、2/2、0 |
| 合計 | 68候補、5/5、状態汚染6 | 13候補、3/5、1 | 48候補、5/5、4 |

ここで「状態汚染」はgold以外の候補のうち、文書自身が`観察 / 比較中 / 棄却 / 撤回 / ARCHIVED / 停止`
を明記した件数である。gold外の決定文書も有用な関連contextになり得るため、全非goldを意味汚染とは
数えない。逆に状態表記のない誤関連はこの値へ出ないので、汚染率の下限である。

10回のbuildと各query 300回のwarm反復では、graph構築median 57.84 ms、expanded lexical query
median 8.22 ms（p95 8.84 ms）、typed graph 2-hop median 1.76 ms（p95 1.89 ms）だった。query部だけなら
graphは約4.7倍速く、約9 queryで一回の構築costを回収する。この規模では専用graph DBのquery engineは
速度の前提ではない。

精度は一様に改善しなかった。typed 2-hopは合計gold 5/5を保って候補を68から48、明示状態汚染を6から
4へ減らしたが、Q1単体では7候補・汚染0から25候補・汚染2へ悪化した。Q3のような「計画からinventoryを
辿る」依存queryには強く、Q1のようにexpanded lexicalだけで閉じるqueryへgraph展開すると汚染を増やす。
またraw語だけのgraphはQ1でgold 0/2だったため、利用者語からrepo語彙への意味翻訳問題は解決しない。

現時点の処分は、graphを常時retrieverやauthority DBにせず、`decision-index`で正しいseedが得られ、
さらに依存／正本linkの展開が必要なqueryだけにtyped 1-hopを使うことである。1-hopで必要closureが揃わない
場合だけtyped 2-hopをshadow候補として返し、状態違いを除外してから読む。無型edgeの2-hop、自動推論edge、
Neo4j常設化は採らない。

### 6.4 Basic Memory直接索引の隔離比較

Fableの直列構成を反証するため、同じdetached HEAD `8b7a76ada61935b609cbf6881b3f10ae5f216e23`の
tracked `docs/`をrepo外へ複製し、既存HVRと同じBasic Memory `0.22.1`、FastEmbed、
`sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2`で直接索引した。設定、model、DBは
`/private/tmp`へ隔離し、sourceへfrontmatterを追加せず、製品checkoutへcacheを作らなかった。

全文索引はすぐ終わったが、初回model取得は252 MB、516 file／513 entityのfull reindexは251.88秒だった。
model cache後のhybrid queryはprocess起動込みで各2.26〜2.30秒だった。上位10件のfile-level goldは次の
とおりである。

| query | Basic Memory top 10 | gold |
|---|---:|---:|
| Q1 Save/reopen | `vism-kit-model.md`が1位。必要2文書はともに圏外 | 0/2 |
| Q2 React直接移管 | 必要契約が9位 | 1/1 |
| Q3 Rerun転移 | inventoryが1位、planが3位 | 2/2 |
| 合計 | 30候補 | 3/5 |

Q1では`ARCHIVED`なmock READMEやVism文書、Q2では旧監督policyやobservationが上位へ混ざった。
現行docsを一文書一entityとして直接索引すると、metadataへMotoliiの状態が無いため検索時に決定的な
現行性filterを掛けられない。従って「hybrid searchの後でgraph filterを掛ければよい」というFable案は、
filter前のtop-kがgoldを落とすQ1で成立しない。candidate poolを広げればrecallを戻せる可能性はあるが、
読込量と状態汚染を戻すため、この結果からは採らない。

一方、この失敗はBasic Memoryの原子的observation／typed relation形式そのものを反証しない。現在の
Markdownをそのままnoteへしたことが主因候補である。次に比較する価値がある最小形は、正本を変更せず、
`decision-index`、spec task、ledgerから決定的に生成した**出典spanつきatomic claim projection**である。
各claimはstable ID、state、owner、validity、source path/span/SHAだけを持ち、本文はsourceの原文spanから
作る。LLM要約、推論edge、scoreによる採否を入れず、source hash不一致ならfail closedする。この派生層を
Basic Memoryへ渡す案と、SQLite FTS／in-memory typed graphだけの案を同じgoldで比較し、既製CLIの価値が
無ければ後者へ縮小する。

同日中に、この最小形の第一段も実行した。`decision-index`の160 data rowを、各一つのnoteへ決定的に
投影した。各noteは行番号由来permalink、固定状態、source path/line/SHA、元行の主題語・一行要旨・正本・
反映先だけを持ち、LLM要約や手書きaliasを追加していない。model cache後のfull reindexは13.10秒まで
縮み、atomicityと更新costは改善した。しかし利用者の生表現ではQ1とQ3が0件、Q2も無関係な2件だけで、
file-level goldへ到達しなかった。repo語彙へ展開した`Reactモック 製品資産 直接移管 source asset`と
`Rerun 発注 動線 転移 順序`では各正しいrowを取得した一方、Q1の`save/reopen/project lifecycle`展開は
M2 session rowへ寄り、M3完成線の二正本を閉じなかった。

また`decision-index`のMarkdown linkをanchor除去後に現HEADへ解決すると、tracked Markdown 260件中
188件が少なくとも一度直接参照され、72件は直接参照されなかった。未参照72件がすべて非正本とは限らず、
この台帳だけを全docsの完全なsemantic catalogとして扱えない。既存README／spec入口と、0件・衝突時の
広域`rg` fallbackを残す必要がある。

従ってatomic claimは状態・provenance・更新負債を改善するが、利用者語からrepo語彙への翻訳を解かない。
現時点でBasic Memory／embeddingを間に置く利得はなく、既に5/5だった複数のlexical展開と比べて遅く、
recallも低い。次のbaselineは、LLMが一つの答えを要約するのでなく3〜5個の**検索仮説**だけを作り、
`decision-index`と`rg`へ独立に流して和集合を取り、明示状態とtyped 1-hopで絞る方式とする。仮説が割れる、
正本が衝突する、必要owner／testが閉じない時だけ関連全文へ昇格する。これは意味を新DBへ移す案でなく、
既存の原子的台帳をsemantic compilation boundaryとして使う案である。

## 7. 現時点の処分

- **観察**: 巨大な単一入口より、短いmapとprogressive disclosureを組み合わせる公開先例がある。
- **観察**: lexical、semantic、dependency graph、long contextは相互排他ではなくroute対象である。
- **観察**: 純粋なvector検索と常時全文contextの双方に反例がある。
- **比較中**: Motoliiの通常実装検索へauthority/status/dependency capsuleを導入すると、正確性を落とさず
  読込量と遅延を減らせるか。
- **比較結果**: 文書初動は`rg`＋状態正本、コードtopologyはAider、exact definition/referenceはSerenaへ
  責任分離する。Serena Markdownは候補文書内の見出しnavigationに限る。
- **比較結果**: typed graphは依存queryの候補削減に効くが、常時2-hopは局所queryの意味汚染を増やす。
  `decision-index` seed後の必要時1-hop、closure不足時だけshadow 2-hopへ限定する。
- **見送り**: QMD semantic常設化。日本語modelとindexが最低限環境には重く、recall改善も未確認。
- **比較結果**: 現行docsをBasic Memoryへ直接索引する案は3/5 recall、初回251.88秒、warm query約2.3秒で
  不採用。atomic claim投影でも生表現のrecallを回復しなかったため、通常routeのsemantic retriever候補から
  外す。既存HVRの任意歴史探索という限定責任は変更しない。
- **見送り**: Neo4j常設化と無型Markdown link graph。現規模では一時in-memory graphで速度が十分で、
  backendよりtyped edgeと状態filterが精度を支配した。
- **見送り**: Graphiti／GraphRAG系を現行authority retrievalへ使う案。時間状態のmodelは参考になるが、
  LLM fact抽出／summaryを正本経路へ置くと、今回重視する意味汚染と再現性の条件を先に悪化させる。
- **停止**: 五queryだけを根拠にしたAGENTS全面再編、常設vector DB、外部service必須化、semantic scoreによる採否。

次の一手は、既存三層routeをbaselineにし、同じ利用者文から3〜5個の検索仮説を作るだけのquery rewriteを
read-only shadowで比較することである。まず10〜15 queryで、単一展開とのrecall差、候補数、状態汚染、
追加token、wall timeを測り、成立する場合だけgoldを30〜50件へ増やす。AGENTSやdocsの再編は、そこで
反復したmissを短い入口map、既存decision row、機械lintのどれで解消できるか確認した箇所だけを別の
決定・変更へ送る。
