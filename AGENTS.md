# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。実装に着手する前にここを読む。

## 「発注」時のClaude Code自動委任

- ユーザーが「発注して」「実装を発注」等、**発注を依頼動詞として明示した時だけ**自動委任を発火する。通常の「実装して」、説明・引用・ファイル内に現れただけの「発注」では発火しない
- **2026-07-22運用改訂**: Cursor/Grok/Composerの期限付き運用を終了し、Claude Codeへ切り替える。完全model IDを固定し、退役modelへ解決され得る`sonnet`/`opus` aliasは使わない。Claude Fableはユーザーが明示した時だけ使い、通常発注では消費しない
- 発火時の役割は固定する。**主担当Codexは先例調査・コード事実の確認・長期展望、発注書の正本化と、恒久形式/公開API/plugin契約/停止線など重要境界の最終判断**を担う。承認済み発注書だけを**Claude Sonnet 5 (`claude-sonnet-5`)が受注者**として実装し、**Claude Opus 4.8 (`claude-opus-4-8`)が実装担当とは別のread-only検収者**として実diffと試験を監査する。Codexが最後に統合を判断する。Opusの発注書draftは論点抽出用の任意助言であり、`ORDER: STOP`を別backendで上書きせず、未決をCodexが正本へ戻してから新しい発注書を作る
- 実装発注は隔離worktreeで行う。Codexが対象仕様ID、目的、現状、変更許可ファイル、非目標、再利用箇所、STOP条件、必須負例、実行コマンドを含むclosed orderを作り、`ORDER: READY`、task hash、`CODEX PRECHECK: APPROVED`を明示してから`./scripts/delegate-claude-supervised.sh execute <worktree> <order-file> "<task>"`でSonnetを起動する。`prepare`はOpusのread-only助言が有用な時だけ使い、その出力を自動承認しない
- Opus検収が`VERDICT: ACCEPT`でなければ実装差分を採用・commit・pushしない
- `delegate-cursor-supervised.sh`と`delegate-cursor-review.sh`はClaude Codeが利用不能で、ユーザーへfallback理由を明示した場合だけ使う。並列助言は調査・論点抽出専用であり、実装の指揮系統には使わない。Sonnetに仕様判断・発注範囲変更・代替設計をさせない
- 主担当は監督者として、外部エージェントの差分を仕様・依存・実装ガード・既存API・テスト期待値に照らしてコードレビューする。レビュー未完了の差分を採用せず、必要な修正と検証を行ってから主作業ツリーへ反映する
- 実装発注は一度に1つの契約境界へ分割し、発注文に **変更許可ファイル・非目標・STOP条件・必須負例・実行コマンド** を明記する。複数境界を同時に満たす「便利な共通化」を発注側から要求しない
- Sonnet実装には「例外追加・lint抑制・テスト期待値変更・生JSON/文字列走査・公開raw API・重複planner/helper」で契約を迂回しないよう明示する。必要に見えた時点で実装を止め、既存の正規境界と仕様IDを報告させる
- Sonnet差分は、実装担当とは別のOpus read-only反対側レビューで **P0/P1=0** を確認するまで採用しない。テスト緑は採用条件の一部であって、契約適合の代わりにしない
- 委任結果は根拠ではなく未検証の助言として扱う。最終判断、統合、必須テスト、完了報告は主担当が行う。Claude子エージェントは委任を再帰実行しない
- 外部モデルへ秘密情報、認証情報、未公開の個人データを渡さない。片方が失敗しても安全に進められる作業は続行し、完了報告に失敗を明記する。両方の成功を完了条件の代わりにしない
- 発注書作成・実装・検収が失敗、STOP、REJECT、timeoutになっても、停止報告だけで未検収差分を放置しない。原因を分類し、発注書の差し戻し、実装修正、検収再実行のうち該当段階へ戻って、契約を迂回せず改善ループを回す。timeout時は残存差分と証跡を確認し、再開可能な段階から続ける。ループ中の差分は隔離worktreeに留め、`VERDICT: ACCEPT`前に採用・commit・pushしない
- 同じ阻害要因が反復し、発注書・回答・差分・検収結果に有意な改善がなくなった場合だけループを止める。その際は、反復した阻害要因、試した修正、未解決の選択肢を示してユーザーの判断を仰ぐ。単なる難しさ、1回のtimeout、外部モデル片方の失敗を停止理由にしない

### Reactモック製品資産を含む発注の強制動線（無視禁止）

Browser、Inspector、`KEYS / LAYERS`、Easing Panel等のReact所有面は、
[React製品資産の直接移管契約](docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)を先に読む。
固定モックを見た目だけのoracleとして製品用componentを別途縮約再実装せず、固定sourceをproduct packageへ
直接所有移管し、mockをproduct exportのconsumerへ反転する。mock固有state、legacy bridge、fixture adapterだけを
Host projection / typed intentへ交換する。DOM/CSSを公開契約へ焼かない規律を、source assetを捨てる理由にしない。

該当発注書は通常項目に加えて、次のラベルを順番どおり持たなければならない。

1. `REACT AUTHORITY`: 対象面、移管契約、UI runtime境界、対応spec ID
2. `SOURCE ASSET`: 固定SHA、旧path、export、CSS/model/test closure
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
のいずれかで`ORDER: STOP`とする。

正しい独立React sourceが存在しない領域は製品packageへ縮約版を先に作らない。固定モック内で同形React化し、
既存visual/interaction oracleへ合格してから所有移管する。presentation移管とHost state接続、WebView統合、D2 commitを
一つの発注へ束ねない。

### Rerun参照を含む発注の強制動線（無視禁止）

Rerunは主要な製品先例だがMotoliiの仕様正本ではない。Rerunを参照する調査・設計・実装発注は、必ず **Motolii仕様 → 現行コード事実 → Rerun先例 → Motolii fixture** の順に通す。Rerunのcrate、型、画面、内部責任からMotoliiの目的・公開API・Document・plugin契約を逆算しない。正本と詳細動線は[Rerun学習・転移計画 §9](docs/reviews/2026-07-20-rerun-learning-transfer-plan.md#9-rerun参照を発注へ入れる強制動線)。候補assetの母集団と監査済み範囲は[Rerun source asset inventory](docs/reviews/2026-07-20-rerun-source-asset-inventory.md)を読み、同文書の「候補分類」を採用裁定として扱わない。

Rerunを一度でも根拠・再利用箇所・変更案に含める発注書は、通常の必須項目に加えて次のラベルを順番どおり持たなければならない。欠落、順序逆転、内容不一致が一つでもあればCodex事前審査は承認せず、Composerを起動しない。

1. `MOTOLII AUTHORITY`: 対象spec ID、決定、既存公開契約、完成条件
2. `CODE FACT GAP`: 現行コードで未成立の事実と再現証跡
3. `RERUN EVIDENCE`: 固定commit、packageだけでなく対象file/API、監査済み範囲と非証明範囲。Motolii要件そのものを書かない
4. `TRANSFER CLASS`: 裁定済みの`DEPEND / VENDOR / PORT / PATTERN / REJECT`
5. `TRANSFER LIMIT`: 変更許可ファイル、持込禁止型・状態・意味、既存境界で自作する比較案
6. `MOTOLII ORACLE`: Rerunとの類似ではなくMotolii fixture/testで判定する合否

次のどれかが起きた時点で`ORDER: STOP`とし、仕様を発明せずCodexへ戻す: Rerunの内部構造を採らないと実装不能に見える／package名またはinventoryの候補分類だけでasset範囲を決めた／未裁定assetの依存・vendoring・移植が必要／公開API・Document・plugin契約・永続形式の変更が必要／Rerunに無いMotolii固有要件を削る必要がある／Rerunの見た目やsnapshotへ合わせるため既存期待値を変更したくなった。検収はRerunへの外観・構造類似を合格根拠にせず、上記6ラベル、Motoliiの負例、依存差分、公開型、serde面、license由来を再確認する。

## 最初に読む

1. [docs/README.md](docs/README.md) — プロジェクト全体像・ドキュメントの読む順序・用語
2. 着手するフェーズの仕様書([docs/specs/](docs/specs/README.md)): タスク表(完了条件・依存つき)と、**末尾の「実装ガード」節**(先行ツールの失敗・ユーザー不満をタスクIDに紐付けた注意リスト。完了条件を追加している場合がある)
3. プラグインを書く/量産する時: [docs/plugin-authoring.md](docs/plugin-authoring.md)(種別・NodeDesc必須欄・禁止事項・型紙)
4. M2 Document/スキーマ/ジャーナルに触る時: **先に**[docs/reviews/2026-07-12-m2-permanence-prevention.md](docs/reviews/2026-07-12-m2-permanence-prevention.md)(予防5手)。背景の先人調査は[rework-prior-art](docs/reviews/2026-07-12-rework-prior-art.md)
5. M3製品実装に触る時: **先に**[docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md](docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md)を読み、ステータスが発効中なら実装を止める。調査・fixtureも公開APIや永続形式へ焼かない
6. M3 UI/入力/タイムライン/プラグインパネルに触る時: **先に**[docs/reviews/2026-07-14-m3-ui-boundary-prevention.md](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)(UI境界の規律8本)
7. M3の外観・timeline・panelに触る時: **先に**[M3 UI参照地図](docs/ui-reference-map.md)、[docs/ui-visual-language.md](docs/ui-visual-language.md)、[React製品資産の直接移管契約](docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md)を読む。Reactモックの実体と`README.md`は固定commit `56c318edcddab7cf95d263cc2f7dd2b4e6791134`で読み、main側にまだ無い時は`docs/mocks/`を代替の現行実装として変更せず、React側の再結合または対象worktreeへの移動を先に行う。`docs/mocks/`は**ARCHIVED・新規変更禁止**。通常入場と`#catalog`はReact候補だけ、legacyは`#archive/*`とparity testだけから参照する。新しいUI判断、操作、goldenをHTMLへ入れようとした時点、またはReact source assetを縮約再実装しようとした時点でSTOPする。モックの具体色値や未決機能をDocument/公開契約へ焼かない
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

- **会話中の仕様ドリフトを先に回収する**: 会話が当初の論点からずれ始めた、新しい用途・用語・状態所有・操作・配布形式へ広がった、既存決定と違う案が出た、と認識した時点で実装を一旦止める。会話を正本にせず、(1) 単なる観察は`docs/reviews/`のobservation、(2) 比較中の案はprototype／decision ledger、(3) 採択済みの意味は対象spec、(4) 後続課題はbacklogへ、**状態（観察／比較中／決定／棄却／停止）と非目標つき**でコードより先に記録する
- **着手前に[決定逆引き台帳](docs/decision-index.md)を主題キーワードで引く**。既決を「未決」と誤認して埋め直さない。決定・撤回・未統一が新しく生まれたら、正本へ書いた上で同じ変更で台帳へ1行登録する(登録規則は[docs/reviews/README.md](docs/reviews/README.md))。docs/reviewsを触ったら`scripts/check-docs.sh`を通す
- ドリフト検知時に既存仕様を黙って上書きしない。矛盾する旧記述と新案を同じ「現行」として残さず、未統一なら入口文書へ両者と解消条件を明記する。恒久形式、公開API、plugin契約、Document意味へ波及する場合は通常のSTOP条件と仕様改訂を優先する
- 作業完了前に、その会話で新しく決まったこと、保留したこと、撤回したことがdocsへ回収され、Codexタスク履歴だけに残っていないか確認する。雑談的な発想は無理に規範化せず、実装判断へ影響し始めた時だけ台帳化する
- **1チケット=1コミット**。完了時に仕様書のチケット表・実装状況表を更新する
- 完了条件は自動判定(`cargo test`/ゴールデンイメージ)。「動いた気がする」を完了条件にしない
- **テストを「直して」通さない**: ゴールデン参照画像・受け入れテストの削除・期待値書き換え・実装のspecial-caseで緑にすることを禁止。**テストが間違っていると思ったら実装を止めて報告する**。参照画像の正当な更新は理由を明記した独立PRに分離(specs/README.md 粒度ルール6、[pitfalls H-2](docs/pitfalls-and-roadmap.md))
- **新規ヘルパーを書く前に既存を検索する**: 同等物が既にないかgrepしてから書く(LLM開発の最大の負債はコピペ増殖 — [pitfalls H-3](docs/pitfalls-and-roadmap.md))。テストヘルパーのtestkit集約ルールの一般化
- **仕様書の未決事項に依存するタスクに着手しない**: 未決を「もっともらしいデフォルト」で埋めない。仕様書改訂PRで先に潰す(specs/README.md 粒度ルール7、GR-PV)
- **完了報告は証跡付き**: 実行したコマンドとテスト出力を添える。「動くはず」を報告にしない
- 提出前に `cargo test --workspace` 全緑を確認
- **プラグイン規約の機械判定(INF-7a〜f)**: 提出前に `cargo test -p motolii-plugin` と、Filter/ParamDriverを触ったら `cargo test -p motolii-testkit --test purity` を回す。新規プラグインは `./scripts/new-plugin.sh <kind> <name>` から始め、純関数は `motolii_testkit::purity` で固定する
- インターフェース契約(specの型シグネチャ)を変えたくなったら、実装を止めて仕様書改訂を先に

## 恒久焼き込みの予防(M2 — GR-PV)

正本: [docs/reviews/2026-07-12-m2-permanence-prevention.md](docs/reviews/2026-07-12-m2-permanence-prevention.md)。失敗後のmigration/Legacyは副次([rework-prior-art](docs/reviews/2026-07-12-rework-prior-art.md))。

着手前チェック(1つでも No なら実装を止め、仕様改訂または依存チケット待ちへ):

1. **意味が先か**: 焼く対象の意味論表/宣言が仕様にあるか。無ければ仕様改訂PRが先(コードで発明しない)
2. **恒久面は狭いか**: 未決・未証明・UI都合だけのフィールドを足していないか
3. **追加的か**: 新フィールド/新variant/defaultか。既存フィールドの解釈変更ではないか
4. **依存直列か**: M2並列レーンを守っているか。特に **D1i-2完了前にD3しない**
5. **完了条件に意味の審判があるか**: 拒否テストまたは意味論ゴールデン。`cargo test`緑だけで「完了」と書かない

破れたときの出口だけ: 形状→D1e migration、画素→新variant(既存ゴールデン更新で通さない)、migration PRにnon-goals。

## UI境界汚染の予防(M3 — GR-UI)

正本: [docs/reviews/2026-07-14-m3-ui-boundary-prevention.md](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)。UI基盤の現行判断は[egui採用記録](docs/reviews/2026-07-18-m3-egui-selection.md)、旧Slint時点の採否記録は[反対側レビュー](docs/reviews/2026-07-14-m3-ui-boundary-counter-review.md)。UIはDocumentの投影であり、eguiの状態・px/DPI・入力イベント列を永続意味論へしない。

M3仕様のGR-UI審判割当表で対象タスクに割り当てられた項目だけを確認する。非該当を形式的にYesにしない。該当項目が1つでもNoなら仕様改訂または依存待ちへ:

1. **状態の持ち場が決まったか**: Document / User settings / Workspace profile / Project session / Transientの5層へ分類したか
2. **書き込み口が一つか**: 永続編集はD2コマンドと単一writerだけを通るか
3. **1ジェスチャー=1履歴か**: D2のmacro/merge/Undo単位を使い、未決transaction APIを発明していないか
4. **UIスレッドを待たせないか**: worker分離、非blocking最新値mailbox、generation破棄があり、同期読み戻しが無いか
5. **UI単位を焼いていないか**: px/DPI/度/ウィンドウ座標をDocument・評価・公開契約へ流していないか
6. **UI toolkitを隔離したか**: `motolii-ui`外の製品クレートとdomain公開APIへegui/eframe/winit依存・型を出していないか
7. **未決を埋めていないか**: GAP-13/GAP-6等の判断前に公開UI APIや恒久設定形式を足していないか
8. **審判が再現可能か**: fixture・command・合否条件があり、基準機性能とIME等の人間確認を自動試験から分離したか
9. **読む前に識別できるか**: 主要状態を文字だけ/色だけで表さず、新規componentを既存のtheme・icon・spacingへ馴染ませたか
