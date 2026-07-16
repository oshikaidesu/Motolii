# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。実装に着手する前にここを読む。

## 「発注」時のCursor自動委任

- ユーザーが「発注して」「実装を発注」等、**発注を依頼動詞として明示した時だけ**自動委任を発火する。通常の「実装して」、説明・引用・ファイル内に現れただけの「発注」では発火しない
- 発火時の役割は固定する。**Grok 4.5 Fastが現場監督**として仕様・やりたいこと・現状差分から発注書案を作り、**主担当Codexが実装前に正しさ・抜け・ユーザー意図との一致を審査**し、承認済み発注書だけを**Composer 2.5が受注者**として実装する。最後に同じGrokが差分を検収する
- 実装発注は隔離worktreeで二段階実行する。まず`./scripts/delegate-cursor-supervised.sh prepare <worktree> <order-file> "<task>"`でGrok案だけを生成して停止する。Codexは誤りがあればComposerへ流さずGrokへ差し戻す。正しければorder fileへ`CODEX PRECHECK: APPROVED`を明示し、`execute`で初めてComposerを起動する
- Grokの発注書は対象仕様ID、目的、現状、変更許可ファイル、非目標、再利用箇所、STOP条件、必須負例、実行コマンドを含む。`ORDER: READY`かつCodex事前承認がない限りComposerを起動しない。Grok検収が`VERDICT: ACCEPT`でなければ差分を採用・commit・pushしない
- `delegate-cursor-review.sh`の並列助言は調査・論点抽出専用であり、実装の指揮系統には使わない。Composerに仕様判断・発注範囲変更・代替設計をさせない
- 主担当は監督者として、外部エージェントの差分を仕様・依存・実装ガード・既存API・テスト期待値に照らしてコードレビューする。レビュー未完了の差分を採用せず、必要な修正と検証を行ってから主作業ツリーへ反映する
- 実装発注は一度に1つの契約境界へ分割し、発注文に **変更許可ファイル・非目標・STOP条件・必須負例・実行コマンド** を明記する。複数境界を同時に満たす「便利な共通化」を発注側から要求しない
- Cursor実装には「例外追加・lint抑制・テスト期待値変更・生JSON/文字列走査・公開raw API・重複planner/helper」で契約を迂回しないよう明示する。必要に見えた時点で実装を止め、既存の正規境界と仕様IDを報告させる
- Cursor差分は、実装担当とは別のread-only反対側レビューで **P0/P1=0** を確認するまで採用しない。テスト緑は採用条件の一部であって、契約適合の代わりにしない
- 委任結果は根拠ではなく未検証の助言として扱う。最終判断、統合、必須テスト、完了報告は主担当が行う。Cursor子エージェントは委任を再帰実行しない
- 外部モデルへ秘密情報、認証情報、未公開の個人データを渡さない。片方が失敗しても安全に進められる作業は続行し、完了報告に失敗を明記する。両方の成功を完了条件の代わりにしない

## 最初に読む

1. [docs/README.md](docs/README.md) — プロジェクト全体像・ドキュメントの読む順序・用語
2. 着手するフェーズの仕様書([docs/specs/](docs/specs/README.md)): タスク表(完了条件・依存つき)と、**末尾の「実装ガード」節**(先行ツールの失敗・ユーザー不満をタスクIDに紐付けた注意リスト。完了条件を追加している場合がある)
3. プラグインを書く/量産する時: [docs/plugin-authoring.md](docs/plugin-authoring.md)(種別・NodeDesc必須欄・禁止事項・型紙)
4. M2 Document/スキーマ/ジャーナルに触る時: **先に**[docs/reviews/2026-07-12-m2-permanence-prevention.md](docs/reviews/2026-07-12-m2-permanence-prevention.md)(予防5手)。背景の先人調査は[rework-prior-art](docs/reviews/2026-07-12-rework-prior-art.md)
5. M3製品実装に触る時: **先に**[docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md](docs/reviews/2026-07-15-m2-foundation-reclosure-gate.md)を読み、ステータスが発効中なら実装を止める。調査・fixtureも公開APIや永続形式へ焼かない
6. M3 UI/入力/タイムライン/プラグインパネルに触る時: **先に**[docs/reviews/2026-07-14-m3-ui-boundary-prevention.md](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)(UI境界の規律8本)
7. M3の外観・timeline・panelに触る時: **先に**[docs/ui-visual-language.md](docs/ui-visual-language.md)と[高密度メインUIモック](docs/mocks/README.md)を読む。モックの具体色値や未決機能をそのまま契約へ焼かない

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

正本: [docs/reviews/2026-07-14-m3-ui-boundary-prevention.md](docs/reviews/2026-07-14-m3-ui-boundary-prevention.md)。採否記録は[反対側レビュー](docs/reviews/2026-07-14-m3-ui-boundary-counter-review.md)。UIはDocumentの投影であり、Slintの状態・px/DPI・入力イベント列を永続意味論へしない。

M3仕様のGR-UI審判割当表で対象タスクに割り当てられた項目だけを確認する。非該当を形式的にYesにしない。該当項目が1つでもNoなら仕様改訂または依存待ちへ:

1. **状態の持ち場が決まったか**: Document / User settings / Workspace-session候補 / Transientを分類したか
2. **書き込み口が一つか**: 永続編集はD2コマンドと単一writerだけを通るか
3. **1ジェスチャー=1履歴か**: D2のmacro/merge/Undo単位を使い、未決transaction APIを発明していないか
4. **UIスレッドを待たせないか**: worker分離、非blocking最新値mailbox、generation破棄があり、同期読み戻しが無いか
5. **UI単位を焼いていないか**: px/DPI/度/ウィンドウ座標をDocument・評価・公開契約へ流していないか
6. **Slintを隔離したか**: `motolii-ui`外の製品クレートとdomain公開APIへSlint依存・型を出していないか
7. **未決を埋めていないか**: GAP-13/GAP-6等の判断前に公開UI APIや恒久設定形式を足していないか
8. **審判が再現可能か**: fixture・command・合否条件があり、基準機性能とIME等の人間確認を自動試験から分離したか
9. **読む前に識別できるか**: 主要状態を文字だけ/色だけで表さず、新規componentを既存のtheme・icon・spacingへ馴染ませたか
