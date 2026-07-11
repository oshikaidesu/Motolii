# AGENTS.md — コーディングエージェント向け作業規約

Cursor / Claude Code / その他のLLMエージェント共通の入口。実装に着手する前にここを読む。

## 最初に読む

1. [docs/README.md](docs/README.md) — プロジェクト全体像・ドキュメントの読む順序・用語
2. 着手するフェーズの仕様書([docs/specs/](docs/specs/README.md)): タスク表(完了条件・依存つき)と、**末尾の「実装ガード」節**(先行ツールの失敗・ユーザー不満をタスクIDに紐付けた注意リスト。完了条件を追加している場合がある)
3. プラグインを書く/量産する時: [docs/plugin-authoring.md](docs/plugin-authoring.md)(種別・NodeDesc必須欄・禁止事項・型紙)

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
- **仕様書の未決事項に依存するタスクに着手しない**: 未決を「もっともらしいデフォルト」で埋めない。仕様書改訂PRで先に潰す(specs/README.md 粒度ルール7)
- **完了報告は証跡付き**: 実行したコマンドとテスト出力を添える。「動くはず」を報告にしない
- 提出前に `cargo test --workspace` 全緑を確認
- **プラグイン規約の機械判定(INF-7a〜f)**: 提出前に `cargo test -p motolii-plugin` と、Filter/ParamDriverを触ったら `cargo test -p motolii-testkit --test purity` を回す。新規プラグインは `./scripts/new-plugin.sh <kind> <name>` から始め、純関数は `motolii_testkit::purity` で固定する
- インターフェース契約(specの型シグネチャ)を変えたくなったら、実装を止めて仕様書改訂を先に
