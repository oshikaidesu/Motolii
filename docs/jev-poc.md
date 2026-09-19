# Jev: 開発コマンド終了後の仕分け PoC

既存の `scripts/run-observed-cli.py` に任意の終了後 observer を接続する。失敗ログから **失敗種別と担当領域**を1回の Jev リクエストで分類し、`jev.json` に次アクションを返す。生成、設計、修正、レビューは既存 LLM／人間が担当する。初期状態は **off**。Codex／Claude の内部ループや個人設定を変更するものではない。

## 調査と接続点

2026-09-19、GitHub `main` の `82fc75b44` を調査した。

| 現行の入口 | 確認と採否 |
|---|---|
| `docs/stage5/workspace.json` / `scripts/motolii-ui.sh` | Flutter UI、Rust の document/render/native。ローカルの `test` は Rust テスト、Dart lint、Flutter analyze/test を実行する。製品ビルドの手順は変えない。 |
| `.github/workflows/ledger-fences.yml` | main push／手動の構成・文書チェック。終了後の仕分けを同じ観測 CLI に接続する。失敗したチェックを成功扱いしない。 |
| `scripts/run-observed-cli.py` | 実在する共通入口。stdout/stderr、終了コード、timeout、signal を保存済みなので、この出口1箇所を採用する。 |
| `scripts/delegate-cursor-supervised.sh` | 冒頭で RETIRED として終了する。復活させない。 |
| `scripts/delegate-cursor-review.sh` | 実質的なレビューの経路。軽量分類でレビューを置き換えない。 |
| `docs/llm-dispatch-observation-and-allocation-runbook.md` | 冒頭で歴史資料に降格済み。固定モデル配分を再導入しない。 |

探索範囲は現行 scripts、workflow、Stage 5 manifest、decision-index と上記の旧 runner。現在の自動分類 LLM 呼び出しを削除したわけではなく、呼び出し側が分類専用ターンを省ける出口を追加した。Issue の自由文ルーティングやレビュー免除は、今回より判断誤りの影響が大きいため対象外。

## ローカルで使う

Python 3.9 以上、追加パッケージなし。TypeSafe console の [API keys](https://console.typesafe.ai/settings/keys) で発行した値を **`TYPESAFE_API_KEY` 環境変数**に設定する。キーをコード、引数、ログ、チャットに貼らない。

```sh
# リポジトリルートで実行。ログの親は checkout と重ならない外部ディレクトリ。
run_root=$(mktemp -d)
python3 scripts/run-observed-cli.py \
  --cwd "$PWD" --log-dir "$run_root/structure" --jev-mode shadow \
  -- "$(command -v python3)" scripts/check-stage5.py
cat "$run_root/structure/jev.json"
```

同じ入口で `-- "$(command -v bash)" scripts/motolii-ui.sh test` を渡せば、既存のテスト手順の結果を仕分けできる。通常のテストには任意の timeout を新設しない。元の stdout/stderr と終了コードはそのまま。raw logs と `meta.json` は従来どおりローカルに残るので共有先を選ぶ。

| モード | 挙動 |
|---|---|
| `off`（省略時） | Jev を読み込まず、通信も `jev.json` 作成もしない。 |
| `shadow` | 失敗時に Jev を評価するが `next_action=existing_llm` を維持。候補だけ記録。 |
| `route` | 有効で十分な信頼度の応答なら、分類専用 LLM ターンを省略できる次アクション・担当を返す。 |

呼び出し側は元のコマンド終了コードを先に扱い、存在する `jev.json` を読む。`next_action=existing_llm`、ファイル欠損／不正、未知の schema は既存の分類へ戻す。`classification_llm_needed=false` の場合は分類専用の追加問い合わせを省略し、`owner` と `next_action` を既存の担当 LLM に渡せる。**修正用の LLM 呼び出しは省略しない**。この PoC 自体はエージェントを起動せず、リトライ、修正、Issue 更新、merge もしない。

成功は終了コード0から機械的に `continue` とし Jev を呼ばない。これはそのコマンドの完了を意味し、製品検収・レビュー完了ではない。signal／timeout／観測失敗では Jev を呼ばず、既存判断へ戻す。

## HTTP 契約と fallback

- [公式 HTTP API](https://docs.typesafe.ai/api) の `POST https://api.typesafe.ai/v1/systemone`。`state` と2つの Choice 質問を送り、名前付き `answers` を受ける。
- [公式モデル一覧](https://docs.typesafe.ai/models) の **`jev-1.13.0` に固定**。違う返却モデル、未知の選択肢、欠落質問、型違い、NaN、異常な確率分布は採用しない。
- [公式 Confidence 指針](https://docs.typesafe.ai/confidence) に従い、不確かな答えを fallback に流す。両質問とも confidence と選択肢確率が0.85以上、かつ unknown 以外で採用する。**0.85はPoCの仮閾値で、正答率85%の保証ではない**。実ログで調整する。
- 成功判定はコード、分類は Jev、次アクションへの対応は固定表。`review_required=true` を常に保ち、Jev に review waiver や executable を返させない。
- キー未設定、401、429、529、通信障害、2秒の壁時計 deadline、空ログ、不正応答は `existing_llm`。自動 retry は0回。DNSや応答本文の待ちも別プロセスの deadline で打ち切る。
- 送る内容は終了コードと stdout/stderr の末尾各8 KiB以下のみ。argv、作業パス、diff、環境全体、会話履歴は送らない。既知 secret 形式と secret 系環境変数値を伏せるが、**伏字は完全な secret 検出ではない**。外部送信してよいログだけで有効化する。
- キーと要求本文は子プロセスの stdin 経由。HTTP redirect を追わない。report には生ログ、キー、provider の自由文を保存しない。

実装の外部基準は上記公式 API と [公式 Python SDK](https://github.com/typesafe-ai/typesafe-sdk-python)、[intent routing](https://docs.typesafe.ai/patterns/intent-routing)、[fan-out](https://docs.typesafe.ai/patterns/fan-out)。既存 runner の lifecycle と標準ライブラリを再利用し、追加 SDK 依存と既定 retry の待ち時間を避けた。

## GitHub Actions

1. repository Actions secret に `TYPESAFE_API_KEY` を設定する。
2. repository Actions variable `MOTOLII_JEV_MODE=shadow` で開始する。実ログで分類を確認後に `route` へ切り替える。
3. `stage5-structure` の既存チェックが観測 CLI を通り、`jev-decisions` artifact に **`jev.json` だけ**を7日保存する。off なら artifact は作られない。検査の失敗コードと既存の後続 step の skip 条件を保つ。

キーや変数はこのPRで設定していない。secret 未設定なら fallback。変数の未設定／不正値は off。PR で動く新しい `jev-poc` workflow はオフラインの契約・回帰テストのみで、secret を参照しない。`pull_request_target` や PR ログを読む特権 workflow は追加しない。required checks、branch protection、merge 権限も変更しない。

rollback は変数を `off` にし、ローカルでは `--jev-mode` を省く。PoC ファイルを残しても外部通信はなくなる。

## テストと測定

```sh
python3 -m unittest discover -s scripts -p 'test_jev*.py' -v
python3 -m unittest discover -s scripts -p test_run_observed_cli.py -v
python3 scripts/benchmark_jev.py --repeats 25
# APIキー取得後。合成4例 × repeats 回の有料APIリクエストを明示的に許可する。
python3 scripts/benchmark_jev.py --live --repeats 5
```

新規試験は公式 Choice 応答契約の正常／異常ケース、認証・rate limit・deadline、秘密値の伏字、shadow、成功時通信ゼロを検証する。実 runner に模擬応答を接続し、採用時も元の失敗コード101と review 必須を保持することを確認する。既存の process group／timeout／raw stream 試験も維持する。

実施結果: Jev関連12件＋既存CLI8件、計20件通過。`actionlint` も通過。構成チェックと文書チェックは失敗するが、変更前 `82fc75b44` の別checkoutと **出力が完全一致**。既存のUI依存違反2件、review索引漏れ、歴史文書の相対リンク切れ、decision-indexの状態語彙が原因。今回の範囲外として維持し、実チェックの失敗コード1も observer 経由で保持した。

2026-09-19 のローカル Python 3.9.6、キー未設定 fallback 100回で、分類関数のみ **p50 0.0025 ms / p95 0.006 ms**。API通信0回、fallback 100回。これは **Jev推論速度でも全プロセス所要時間でもない**。

実APIのレイテンシ、実ログの分類精度、実運用のLLM削減回数、開発ループ全体の短縮は **未測定**。キーがなく、現行の分類専用LLM呼び出しのbaselineも未取得のため。模擬応答では2質問を1要求で処理し、採用した失敗1件につき最大1回の「分類専用ターン」が不要になる経路を確認した。既存LLMが1ターンで分類と修正を兼ねる場合、呼び出し削減は0回になり得る。

report の `jev_requests_attempted` は要求の試行数、`jev_latency_ms` は通信プロセスを含む待ち時間、`triage_latency_ms` は全分類処理。`potential_classification_calls_avoided` は比較仮定が「失敗1件につき分類専用LLMを1回」の場合の候補値で、実績ではない。shadow／fallbackでは0。benchmark の `production_llm_calls_saved` と `end_to_end_speedup` は未測定を示す null のままにする。

live benchmark は合成例の smoke test に過ぎない。採用率、採用時のラベル一致、fallback率、p50/p95を出す。運用移行は実ログの正解ラベルを別途付け、同じログ・同じ分類契約で既存LLMと比較し、誤配分による修正時間と fallback 時の追加2秒も含めて判断する。

## 次の拡張候補

まず実ログで閾値と誤配分率を確認する。その後、既存の呼び出し側で分類専用ターンを実際に省いた件数を記録する。次候補は Issue 受付時の担当領域選択。レビューは「追加レビューの必要性を知らせる」方向だけを検討し、既存レビューの免除や設計承認には広げない。
