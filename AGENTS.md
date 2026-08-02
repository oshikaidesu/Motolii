# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。実装に着手する前にここを読む。

## 実装前の既知実装fail-close

- **一般機構を先に作らない**: 計画・仕様化・発注・実装前にrepo、[決定台帳](docs/decision-index.md)、[references](docs/references.md)、一次資料を調べ、主担当preflightへ`MECHANISM CLASS / KNOWN IMPLEMENTATION / ADOPTION ROUTE / THIN MOTOLII RESIDUAL / RETIREMENT / BUILD: FORBIDDEN`を記録する。欠落、未調査、裁定なし、一般frameworkの薄い残余への偽装では実装担当を起動しない。既決routeは正本を示して継承し、`BUILD`はmodelが許可せず利用者例外へ返す。これはtransport schemaではない。詳細は[既知実装採択・置換開発モデル](docs/known-implementation-adoption-model.md)

## 監督runnerの廃止と薄いCLI監視（2026-08-03）

- 旧delegate/activateはexit 64のまま。外部CLIの起動と生ログ保存だけは[`run-observed-cli.py`](scripts/run-observed-cli.py)を使う。
  worktree、監督、採否、session資格は所有しない。詳細は[薄いCLI監視決定](docs/reviews/2026-08-03-thin-observed-cli-harness-decision.md)。

## 最上位の権限保存

- **自己発注禁止**: 主担当Codexは、ユーザーが許可した`AUTHORIZED_OUTCOME / AUTHORIZED_ARTIFACTS / AUTHORIZED_MUTATIONS / AUTHORIZED_VALIDATION`を自分で増やさない。次の一手が成果物、owner、権限、完了条件、model呼出し、検収周回を増やす場合、その追加分は未許可として施工せず、既存scopeの最小次手を続けるかユーザーへ返す
- **findingは権限ではない**: 調査、test、review、Grok、Opus、Fable、別Codexが新しい問題を発見しても、同一taskでの追加施工権限にはならない。既存完了条件を満たせない`IN_SCOPE_BLOCKER`だけを許可済みallowlist内の最小修正へ戻し、それ以外は`OUT_OF_SCOPE_FINDING`または`FOLLOW_UP`として報告する。reviewerはorder、scope、完了条件を増やさない
- **既決を未決へ戻さない**: 提案、再設計、仕様化、発注の前に[決定逆引き台帳](docs/decision-index.md)を主題keywordで検索し、該当decision ID／正本path／現行状態を示す。該当決定を読まずに新しい仕組みを提案しない。正本と現行コードが衝突する場合だけ`AUTHORITY_CONFLICT`として当該操作を止める
- 上記三則は主担当Codexを含む全modelに適用する。自己反証、隔離worktree、検収、技術的有用性、安全性は追加権限の代わりにならない。機械判定できる禁止は既存のsandbox、hook、CI、testで拒否する

## 「発注」時のrunner非依存監督

- 「発注して」「実装を発注」等、**発注を依頼動詞として明示した時だけ**自動委任する。通常の「実装して」、説明、引用、ファイル内の語では発火しない
- 主担当Codexがユーザーscope、正本、base/cwd、変更対象、非目標、oracleを所有し、一つの契約境界だけを実装担当へ渡す。意味、owner、原因、再利用、oracleのいずれかが閉じない`WIDE`は実装担当へ送らず、調査またはユーザー判断へ戻す
- worktreeは主担当Codexが用意する。外部CLIは[`run-observed-cli.py`](scripts/run-observed-cli.py)へ完全なmodel/mode/sandbox引数を渡して起動し、利用不能時に別modelへ黙ってfallbackしない。外部modelは再委任しない
- 実装担当と最終reviewerは別session・別役割にする。reviewerはread-onlyで実diffと試験を確認し、変更した場合は検収を無効とする。必要なmodelと段階はtaskに応じてCodexが選び、全発注へ固定直列routeを課さない
- modelは履歴上の得意領域で選ぶ。意味・owner・原因・共有契約の閉鎖には`claude-opus-5`を実装前のread-only相談へ使い、scope・allowlist・exact target・負例・実diffの列挙監査には`cursor-grok-4.5-high`を使う。閉じた機械施工は`gpt-5.3-codex-spark`へ渡せるが、性能・安全性・永続性の合否は非LLM oracleで決める
- Sparkがcapacity／rate limitで起動・完了できない場合、Codexは失敗をlogへ残し、同じbase・scope・allowlist・oracleを再確認した上でComposerまたはLuna Max等の低コストmodelを新しい実装担当として明示選択できる。これはsilent fallbackや自動retry列ではない。CLIで利用可能な完全model IDを確認して記録し、推測したaliasを使わず、変更後のmodel familyと異なる最終reviewerを選ぶ
- 同じtaskで設計・契約閉鎖へ深く関与したmodel familyは最終reviewerにしない。Claudeを事前相談へ使ったらGrok等の別family、Cursor/Grokを施工またはpreflightへ使ったらClaude等の別familyをfresh read-only reviewerにする。小さく閉じたtaskはpreflightを省き、Grokの寡黙・timeout・空の完了結果は失敗として記録して黙って別modelへ切り替えない
- 採用はCodexが実base、開始前後fingerprint、diff、試験、review結果を直接照合して決める。LLMの賛同やlogの存在だけで採用せず、P0/P1未解決、scope外変更、reviewer mutation、ユーザーSTOP後の実行を拒否する
- promptは正本を再送せず、判断に必要なコード事実、対象path、変更境界、負例、確認commandだけに絞る。不足時は推測やrepo横断探索をさせずCodexへ戻す。詳細は[runner非依存監督決定](docs/reviews/2026-08-03-runner-independent-supervision-decision.md)
- 旧runner固有の起動・状態・証跡語彙は歴史資料であり、現行の起動条件・採用資格・必須fieldに使わない
- model別役割の根拠と分岐は[履歴較正によるLLM役割選択](docs/reviews/2026-08-03-history-calibrated-llm-role-selection-decision.md)に従う。これは固定routeでも採用資格でもない

## 発注外のscope自己反証とコーディングパートナー

- **違和感で親taskを止めない**: 調査から比較実験、検収から再実装、修復から新機構、報告から強制介入のように、次の一手が元の依頼から動詞、成果物、owner、権限、完了条件のいずれかを増やす時は、親taskでなく**その考え**を疑う。実行前に`ORIGINAL_OUTCOME / PROPOSED_NEXT_ACTION / WHY_NEEDED / ADDED_SCOPE / DISCONFIRMING_EVIDENCE / SMALLEST_IN_SCOPE_ACTION / DISPOSITION`を短く書く。`DISPOSITION`は`KEEP / REVISE / DROP / ESCALATE`だけとし、説明できない案は`DROP`、過大な案は`REVISE`して、親taskを最小のscope内経路で継続する
- **相談packet自体を第一のブレーキにする**: packet作成中に、ユーザーが求めていない成果物、自己追加した完了条件、「隔離されているから実行してよい」という安全性と権限の混同、既に完了した施工の再実行が見えた場合は、相談相手の返答を待たず候補案を`DROP / REVISE`する。疑念を無期限WAIT、親task全体の停止、無関係laneの停止へ一般化しない
- **別Codexを高速なscope反対側に使う**: 自己反証後も`KEEP`か`REVISE`かが割れ、回答で次の実行が変わり得る場合だけ、会話全文を渡さないfresh-contextの別Codexをread-onlyで一つ呼ぶ。渡すのは上記packetと検証済みコード事実だけとし、編集、外部model起動、再委任を許さない。回答は`FACTS / SCOPE_DELTA / COUNTEREXAMPLE / SMALLEST_NEXT_ACTION / DISPOSITION`に限定し、同じmodelの賛同をauthorityや実行許可にしない
- **段階的に昇格する**: 正本とコード事実だけで閉じる通常作業は主担当Codexが続行する。別Codexでも要求解釈、owner、原因、再利用境界が閉じない時だけOpus 5へ進み、共有公開境界、恒久契約、長期展望、またはCodexとOpusの結論衝突だけをFable 5へ上げる。外部LLMを全作業の直列barrierにしない
- **`STOP`は局所信号**: 既存規約の`STOP`は危険な候補操作、契約を発明する施工、または該当粒を実行しないという意味であり、親taskを放棄する命令ではない。主担当Codexへ戻して`REUSE / REMAP / REDUCE / 再調査 / 別lane継続`の次手を選ぶ。不可逆操作に必要な権限が無い等、利用者判断なしに安全な次手が存在しない場合だけ当該粒をユーザーへ返す

### Opus 5コーディングパートナー

- `claude-opus-5`は「発注」外でもread-only広域相談に使えるが、正本とコード事実だけで閉じる作業の形式的barrierにしない。要求解釈、複数owner／原因、再利用境界、負例、公開API／Document／永続形式への波及で判断が変わり得る場合だけ呼ぶ
- 相談packetは確定仕様、コード事実、仮説、選択肢、非目標、反例、改善機会を含め、回答を`FACTS / INFERENCES / OPTIONS / OPPORTUNITIES / ADVICE / RECOMMENDATION / STOP CONDITIONS`へ分離する。編集、commit、push、PR、Spark、再委任を許さず、助言をauthorityや追加施工権限にしない
- Opus／Fable相談は薄いCLI harnessから起動し、生stream、timeout、exit status、process回収を保存する。完了前stdout空を空回答と判定せず、別modelへ黙ってfallbackしない
- 大地図、長期展望、複数仕様衝突、共有公開境界、恒久契約、CodexとOpusの結論衝突だけを`claude-fable-5`へread-only昇格する。加えて、新機構が必要に見え、列挙した既知routeが必須oracle、license、platform、security、maintenanceで落ちる証拠が揃った時は、利用者例外へ返す直前に一回だけFableへ先例の取りこぼし／再写像を照会する。Fableは発明を認可・仕様化しない。Claude Code CLIから直接呼び、Cursor同名modelで代替しない

### Reactモック製品資産を含む発注の強制動線（無視禁止）

Browser、Inspector、`KEYS / LAYERS`、Easing Panel等のReact所有面は、
[React製品資産の直接移管契約](docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)を先に読む。
固定モックを見た目だけのoracleとして製品用componentを別途縮約再実装せず、固定sourceをproduct packageへ
直接所有移管し、mockをproduct exportのconsumerへ反転する。mock固有state、legacy bridge、fixture adapterだけを
Host projection / typed intentへ交換する。DOM/CSSを公開契約へ焼かない規律を、source assetを捨てる理由にしない。
静止画、render結果、DOM snapshot、live DOM、React DevToolsから動的挙動を補完しない。未移管assetは固定mock SHA、
移管済みassetはprovenance manifestが固定した現product closure hashのReact実コードを開き、対象exportから
到達するcomponent、hook、local state、event handler、effect、model、Storybook、Playwright操作列をsource closureとして
列挙する。各動的stateを`維持するlocal presentation`または`Host projection / typed intentへ交換するsemantic state`へ
一件ずつ分類できない施工は`STOP`とする。

主担当Codexは施工前に次を確認する。これはtransportへ渡すschemaではない。

1. `REACT AUTHORITY`: 対象面、移管契約、UI runtime境界、対応spec ID
2. `SOURCE ASSET`: `FIXED_MOCK`または`PRODUCT_CLOSURE`のprovenance hash、path、export、component/hook/state/event/effect、CSS/model/story/test closure。対象source closureを実装担当へ渡す
3. `PRESERVE`: DOM、class、stable ID、ARIA、interaction、visual state
4. `REPLACE`: mock/legacy stateからprojection / intentへ交換する範囲
5. `STATE OWNER`: Document / User settings / Workspace / Project session / Transient / local presentation
6. `DIAGNOSTIC ROUTE`: 正しい製品画面とdevelopment専用契約確認画面の分離
7. `NEGATIVE ORACLE`: 二重copy、legacy import、opaque-ID分岐、二重state、threshold変更の拒否
8. `STOP`: 未決意味、公開契約、source不在、owner境界違反に遭遇した場合の停止

欠落、順序逆転、固定SHA/pathとの不一致が一つでもあればCodex事前審査は承認せず、実装担当を起動しない。
source assetがあるのに別leafを新設した、CSS修理だけでparityへ寄せ始めた、skeletonを製品面にした、
`TimelineCandidate`全体をnative Timelineの代わりに持ち込んだ、productが`docs/mocks-ui`/legacy scriptをruntime
importした、mock/productへ同じcomponent copyを残した、catalog ID/label/thumbnail tokenから欠落意味を推測した、
ReactへDocument/selection/Undo正本を追加した、visual threshold/goldenを変えた、diagnostic routeだけを成果にした、
静止画、render結果、DOM snapshot、live DOM、React DevToolsからinteraction/stateを補った、動的stateのownerを推測した、
のいずれかで施工を`STOP`する。

正しい独立React sourceが存在しない領域は製品packageへ縮約版を先に作らない。固定モック内で同形React化し、
既存visual/interaction oracleへ合格してから所有移管する。presentation移管とHost state接続、WebView統合、D2 commitを
一つの発注へ束ねない。

### Rerun参照を含む発注の強制動線（無視禁止）

Rerunは主要な製品先例だがMotoliiの仕様正本ではない。Rerunを参照する調査・設計・実装発注は、必ず **Motolii仕様 → 現行コード事実 → Rerun先例 → Motolii fixture** の順に通す。Rerunのcrate、型、画面、内部責任からMotoliiの目的・公開API・Document・plugin契約を逆算しない。正本と詳細動線は[Rerun学習・転移計画 §9](docs/reviews/2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)。候補assetの母集団と監査済み範囲は[Rerun source asset inventory](docs/reviews/2026-07-20-rerun-source-asset-inventory.md)を読み、同文書の「候補分類」を採用裁定として扱わない。

Rerunを一度でも根拠・再利用箇所・変更案に含める場合、主担当Codexは施工前に次を確認する。これはtransportへ渡すschemaではない。欠落または内容不一致があれば実装担当を起動しない。

1. `MOTOLII AUTHORITY`: 対象spec ID、決定、既存公開契約、完成条件
2. `CODE FACT GAP`: 現行コードで未成立の事実と再現証跡
3. `RERUN EVIDENCE`: 固定commit、packageだけでなく対象file/API、監査済み範囲と非証明範囲。Motolii要件そのものを書かない
4. `TRANSFER CLASS`: 裁定済みの`DEPEND / VENDOR / PORT / PATTERN / REJECT`
5. `TRANSFER LIMIT`: 変更許可ファイル、持込禁止型・状態・意味、既存境界で自作する比較案
6. `MOTOLII ORACLE`: Rerunとの類似ではなくMotolii fixture/testで判定する合否

次のどれかが起きた時点で施工を`STOP`し、仕様を発明せずCodexへ戻す: Rerunの内部構造を採らないと実装不能に見える／package名またはinventoryの候補分類だけでasset範囲を決めた／未裁定assetの依存・vendoring・移植が必要／公開API・Document・plugin契約・永続形式の変更が必要／Rerunに無いMotolii固有要件を削る必要がある／Rerunの見た目やsnapshotへ合わせるため既存期待値を変更したくなった。検収はRerunへの外観・構造類似を合格根拠にせず、上記6項目、Motoliiの負例、依存差分、公開型、serde面、license由来を再確認する。

## 最初に読む

1. [docs/README.md](docs/README.md) — プロジェクト全体像・ドキュメントの読む順序・用語
2. 着手するフェーズの仕様書([docs/specs/](docs/specs/README.md)): タスク表(完了条件・依存つき)と、**末尾の「実装ガード」節**(先行ツールの失敗・ユーザー不満をタスクIDに紐付けた注意リスト。完了条件を追加している場合がある)
3. プラグインを書く/量産する時: [docs/plugin-authoring.md](docs/plugin-authoring.md)(種別・NodeDesc必須欄・禁止事項・型紙)
4. M2 Document/スキーマ/ジャーナルに触る時: **先に**[docs/reviews/2026-07-12-m2-permanence-prevention.md](docs/reviews/2026-07-12-m2-permanence-prevention.md)(予防5手)。背景の先人調査は[rework-prior-art](docs/reviews/2026-07-12-rework-prior-art.md)
5. M3製品実装に触る時: **先に**[docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md](docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md)を読み、ステータスが発効中なら実装を止める。調査・fixtureも公開APIや永続形式へ焼かない
6. M3 UI/入力/タイムライン/プラグインパネルに触る時: **先に**[docs/reviews/2026-07-14-m3-ui-boundary-prevention.md](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)(UI境界の規律8本)
7. M3の外観・timeline・panelに触る時、またはUI実行物を表示・起動・比較する時: **最初に**[UI成果物・実装状態の用語](docs/ui-artifact-terminology.md)で要求名を成果物へ分類し、次に[M3 UI参照地図](docs/ui-reference-map.md)、[docs/ui-visual-language.md](docs/ui-visual-language.md)、[React製品資産の直接移管契約](docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)を読む。`Motolii Studio Preview`が未実装なら、Mock、Native Shell Baseline、個別spike、egui比較baselineを代替起動せず、未実装と報告する。別成果物を見せる時はユーザーがその固有名を指定した場合だけにする。Reactモックの実体と`README.md`は固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`で読み、main側にまだ無い時は`docs/mocks/`を代替の現行実装として変更せず、React側の再結合または対象worktreeへの移動を先に行う。`docs/mocks/`は**ARCHIVED・新規変更禁止**。通常入場と`#catalog`はReact候補だけ、legacyは`#archive/*`とparity testだけから参照する。新しいUI判断、操作、goldenをHTMLへ入れようとした時点、またはReact source assetを縮約再実装しようとした時点でSTOPする。モックの具体色値や未決機能をDocument/公開契約へ焼かない
8. Rerunのsource、crate、画面、実装patternを調査・発注・実装へ使う時: **先に**[Rerun source asset inventory](docs/reviews/2026-07-20-rerun-source-asset-inventory.md)と[Rerun学習・転移計画](docs/reviews/2026-07-20-rerun-learning-transfer-plan.md)、特に後者§4/§8/§9を読む。Rerun起点で発注書を書かない

## 絶対規律(破ると設計の根拠が崩れる。レビュー最重視項目)

1. **VRAM常駐**: ピクセルはwgpuテクスチャとしてGPUに置いたまま処理。安易なCPU処理の混入禁止
2. **色変換の一元化**: 色変換はレンダ直前の1箇所のみ
3. **プラグイン純関数契約**: 出力は時刻tと入力だけで決まる。隠れた可変状態の禁止(正本は`docs/concept.md`「馬鹿正直にシミュレートしない」— 第一選択は常にf(t)の安い力)。物理・前後フレーム等の時間軸依存が本当に要る表現だけ正規ルート(レンダ外のベイク境界)へ — [docs/simulation-model.md](docs/simulation-model.md)の5段はしごを参照。Filterに状態を隠すハックのPRは受けない
4. **単一writer**: ドキュメントを書き換えるのは編集スレッドだけ。他は`Arc<Document>`の読み手
5. **正準座標系**: 空間パラメータは正準空間(単位なし・原点中央・Y-up・高さ=1.0)で持つ。絶対px値のパラメータ禁止
6. **プレビュー/書き出し同一関数**: 差は`Quality`引数のみ。並行レンダ経路を作らない
7. **プラグイン契約にベンダー/OS固有APIを出さない**: 見せるGPUはwgpu/WGSL抽象のみ(CUDA/Metal/DX等を契約に露出しない)。OS分断の再生産防止(落とし穴F-9)

## 実装規約(2026-07-09 コードレビューの教訓より)

- **公開APIで`assert!`/panicしない**。入力起因の失敗は型付き`Result`(thiserror)で返す(例: JSON経由の値が直接届く関数)
- **ループ内でGPUリソースを作らない**。テクスチャ/バッファ/パイプライン/シェーダモジュールの生成はコンストラクタかループ外へ。再利用パターンは`motolii-gpu::RgbaDownloader`と`motolii-gpu::yuv::SizePool`を参照
- **`?`での早期returnが後始末を飛ばさないか確認**。特に`Encoder::finish()`(飛ばすとDropがffmpegをkillしmp4が壊れる)
- **エラー型を文字列に潰さない**。`#[from]`/`#[error(transparent)]`で構造を保ち、呼び出し側がmatchできる形を維持
- **テストヘルパーはmotolii-testkitへ**。`gpu_or_skip`等をテストファイル間でコピペしない
- **コメントは日本語で「なぜ」だけ**書く(何をしているかはコードが語る)

## ワークフロー

- **会話中の仕様ドリフトを先に回収する**: 会話が当初の論点からずれ始めた、新しい用途・用語・状態所有・操作・配布形式へ広がった、既存決定と違う案が出た、と認識した時点で、広がった候補案の実行だけを保留し、親taskは既存scope内の次手で継続する。会話を正本にせず、(1) 単なる観察は`docs/reviews/`のobservation、(2) 比較中の案はprototype／decision ledger、(3) 採択済みの意味は対象spec、(4) 後続課題はbacklogへ、**状態（観察／比較中／決定／棄却／停止）と非目標つき**でコードより先に記録する
- **着手前に[決定逆引き台帳](docs/decision-index.md)を主題キーワードで引く**。既決を「未決」と誤認して埋め直さない。決定・撤回・未統一が新しく生まれたら、正本へ書いた上で同じ変更で台帳へ1行登録する(登録規則は[docs/reviews/README.md](docs/reviews/README.md))。docs/reviewsを触ったら`scripts/check-docs.sh`を通す
- **要求を直接「新しい意味決定」へ送らない**: 各粒を `AUTHORITY → INTERNAL TARGET → OWNER → WRITE ROUTE → GAP → RESOLUTION ROUTE → DISPOSITION(PASS / REDUCE / RESOLVE)`へ写す。既存targetとrouteは再決定せず接続する。`GAP`は未調査やUI名称差でなく型・試験の不在または契約矛盾で証明する。解決段は `REUSE → REMAP → REDUCE → 正本／decision-indexの採択route参照 → 必要時Opus`とし、共有／恒久境界はFableへ上げる。仕様化してよいのは採択済みrouteの接続契約と製品policy／oracleだけで、modelは新機構targetを仕様化しない。既知routeが具体的反証で尽きた時はFableの一回の取りこぼし検査後、利用者例外へ返す。`STOP`は発明施工だけを止める局所信号であり、親taskと接続可能laneは続ける。解決粒は[implementation ledger](docs/implementation-ledger.md)へ登録し、新決定は正本と[decision-index](docs/decision-index.md)を同時更新する
- ドリフト検知時に既存仕様を黙って上書きしない。矛盾する旧記述と新案を同じ「現行」として残さず、未統一なら入口文書へ両者と解消条件を明記する。恒久形式、公開API、plugin契約、Document意味へ波及する場合は通常のSTOP条件と仕様改訂を優先する
- 作業完了前に、その会話で新しく決まったこと、保留したこと、撤回したことがdocsへ回収され、Codexタスク履歴だけに残っていないか確認する。雑談的な発想は無理に規範化せず、実装判断へ影響し始めた時だけ台帳化する
- **1チケット=1コミット**。完了時に仕様書のチケット表・実装状況表を更新する
- 完了条件は[repository validation topology](docs/reviews/2026-07-31-repository-validation-topology-decision.md)に従い、各粒へ`PRIMARY_ORACLE / REPO_LANES / EXTERNAL_GATES`を固定する。`cargo test`はRust laneであり、React、docs、製品E2E、実機、人間審判を代替しない。「動いた気がする」、変更面を観測できないgreen、未実行を完了条件にしない
- **テストを「直して」通さない**: ゴールデン参照画像・受け入れテストの削除・期待値書き換え・実装のspecial-caseで緑にすることを禁止。**テストが間違っていると思ったら実装を止めて報告する**。参照画像の正当な更新は理由を明記した独立PRに分離(specs/README.md 粒度ルール6、[pitfalls H-2](docs/pitfalls-and-roadmap.md))
- **新規ヘルパーを書く前に既存を検索する**: 同等物が既にないかgrepしてから書く(LLM開発の最大の負債はコピペ増殖 — [pitfalls H-3](docs/pitfalls-and-roadmap.md))。テストヘルパーのtestkit集約ルールの一般化
- **発明工程を持たない**: 冒頭の既知実装preflight 6欄を満たし、汎用機構をrepo、[決定逆引き台帳](docs/decision-index.md)、[参考ライブラリ一覧](docs/references.md)、一次資料から一度だけ`REUSE / ADOPT / WRAP / PORT / PATTERN / EXTERNAL / REJECT`へ裁定する。後続粒は採択routeを継承し、粒ごとの再調査は必須oracle、license、platform、security、maintenanceの反証またはadapterの共有基盤化がある時だけ。`PORT / PATTERN`は解決済み機構のMotolii所有実装であり、薄いtranslation／admission adapter、製品policy、fixtureを許す。新機構の`BUILD`は通常処分にせず、Fable検査でも回避不能な時だけ利用者例外へ返す。既完了や投入工数を維持理由にせず、同じoracleへ通す縦slice置換で単一ownerを切り替え、旧routeを`FROZEN → RETIRE`する。旧decisionの`BUILD`／採択非継承表記は歴史語彙であり現行orderへ使わない。詳細は[依存優先・責任最小化ゲート](docs/reviews/2026-07-24-dependency-first-responsibility-gate.md)
- **仕様書の未決事項に依存するタスクに着手しない**: 未決を「もっともらしいデフォルト」で埋めない。仕様書改訂PRで先に潰す(specs/README.md 粒度ルール7、GR-PV)
- **完了報告は証跡付き**: 実行したコマンドとテスト出力を添える。「動くはず」を報告にしない
- 提出前は`./scripts/validate.sh local`でportable local profileを確認し、local依存setup済みなら`./scripts/test-local.sh`でも同じprofileを確認する。さらに粒の`PRIMARY_ORACLE / REPO_LANES / EXTERNAL_GATES`を追加実行する。local profile greenはCI、platform、human／hardware greenの代替ではない
- 既知の既存不具合でlocal profileの一laneがredでも、profile全体をgreenと報告しない。独立な残りlaneは直接実行して個別証跡を残し、red laneと未実行gateを分離して報告する
- **プラグイン規約の機械判定(INF-7a〜f)**: 提出前に `cargo test -p motolii-plugin` と、Filter/ParamDriverを触ったら `cargo test -p motolii-testkit --test purity` を回す。新規プラグインは `./scripts/new-plugin.sh <kind> <name>` から始め、純関数は `motolii_testkit::purity` で固定する
- インターフェース契約(specの型シグネチャ)を変えたくなったら、実装を止めて仕様書改訂を先に

## 恒久焼き込みの予防(M2 — GR-PV)

正本は[恒久焼き込みの予防5手](docs/reviews/2026-07-12-m2-permanence-prevention.md)。M2 Document／schema／journalへ触る粒は、意味の仕様化、恒久面の最小化、追加的変更、依存順、意味の拒否試験を正本どおり確認する。一つでも閉じなければコードで補わず、当該施工を止めて仕様改訂または依存待ちへ戻す。migration／Legacyは予防の代替にしない。

## UI境界汚染の予防(M3 — GR-UI)

正本は[M3 UI境界の規律8本](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)。M3仕様のGR-UI審判割当表で対象粒に割り当てられた項目だけを確認し、非該当を形式的に合格させない。UIはDocumentの投影であり、toolkit型、UI state、px／DPI、入力event列を永続意味論へしない。owner、D2単一writer／Undo、非blocking worker、toolkit隔離、再現可能なfixture／構造化logが一つでも閉じなければ、当該施工を止めて仕様改訂または依存待ちへ戻す。
