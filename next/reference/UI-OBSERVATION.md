# 実窓 UI 観測の道路

画面を目視して手で撮る工程を、次の一組へ固定する。

```text
シナリオ台帳(JSON)
    ↓ argv → production CLI → Shell::update
実ウィンドウ(PID / window id)
    ↓ screencapture -l
scenario/window.png + capture.json
    ↓ Pillow + numpy
region metrics → checks → score → scores.json / scores.tsv
```

## 何が証拠か

`next/reference/ui-observation-scenarios.json` は、動作名・production CLI の入口・
比較対象・画面へ求める機械条件を持つ。現在の5行は既存の `main.rs` が実際に
`Shell::update` へ送っている操作である。

`scripts/capture_ui_scenarios.py` は macOS CoreGraphics で起動した PID の window id を
特定し、`screencapture -l` でその窓を撮る。したがって `--fixture --screenshot` の
オフスクリーン再現画像はこの道路へ入らない。`--fixture` は素材の状態を決定論的に
揃えるための seed であり、画像は Iced の実窓から取る。`capture.json` の
`source=real-window` が無い画像は、解析器が赤にする。

出力は `target/ui-observations/<UTC>/` に置く。

- `run.json`: PID、window id、argv、撮影方法、失敗理由
- `scenarios/<id>/window.png`: 動作別の実窓画像
- `scenarios/<id>/capture.json`: 画像の出所
- `scenarios/<id>/process.log`: 起動プロセスの標準出力
- `scores.json`: 領域ごとの実測値、各検収条件、赤/緑、数値 score
- `scores.tsv`: 監督が一覧で読む集計

## 実行

preview binary をコード変更ごとに一度だけ更新してから、シナリオ一式を回す。

```bash
cargo build --manifest-path "$(git rev-parse --show-toplevel)/next/Cargo.toml" \
  --profile preview -p motolii-shell -j 4
python3 scripts/capture_ui_scenarios.py "$(git rev-parse --show-toplevel)"
```

まず経路と argv だけを確認する場合は、窓を開かずに次を使う。

```bash
python3 scripts/capture_ui_scenarios.py "$(git rev-parse --show-toplevel)" --dry-run
```

特定シナリオだけを回す時も、比較元を同時に指定する。

```bash
python3 scripts/capture_ui_scenarios.py "$(git rev-parse --show-toplevel)" \
  --scenario fixture-boot --scenario observe-camera
```

画面の採点は主観的な「かっこよさ」ではない。現在の検査は次を測る。

- `luma_p95_p05`: 面が一色に潰れていないか
- `edge_density`: 文字・境界・構造が消えていないか
- `color_std` / `activity`: 面の中身が存在するか
- `delta.mean_abs_rgb` / `delta.changed_fraction`: 操作前後で見える結果が変わったか

各条件は JSON の閾値に対して機械的に比較し、通過数 / 条件数を 0〜100 の score として
出す。score は美的評価ではなく、観測契約の充足率である。画像が無い、実窓でない、
比較元が無い場合は未確認のままにせず赤になる。

## 追加方法

新しい操作は Rust の意味をこの台帳へ複製しない。既存の production CLI 入口があるなら、
次の3つを一行追加する。

1. `operation`: `Shell::update` までの意味を書く
2. `argv`: 既存 CLI が送る操作を列挙する
3. `reference` / `delta`: 操作結果が画像へ現れる比較元と変化床を書く

CLI 入口がまだ無い操作は、先に component の `entry → meaning → evaluation → render →
observable` を実装する。その後この台帳へ追加する。画像解析器に操作の意味を持たせたり、
`screenshot.rs` の別描画を「実窓」と呼んだりしない。

## 機械採点の限界

この道路は、画面が空白になった、構造が消えた、操作しても画素が変わらない、という
低層の契約を自動で閉じる。ラベルの意味、ギズモが正しい対象を掴んだか、視線の自然さの
最終判定までは画像の統計だけでは証明しない。そこは既存の drive test と、最後にまとめる
実窓操作検収を同じ scenario id へ結び付ける。つまり「画像が緑」だけで UX 完了とはしない。

## 出典と検収条件

- 出典: macOS `screencapture` / CoreGraphics の実窓キャプチャ。製品仕様に依存しない
  観測手段として採用。外部 URL の見た目を正本にはしない
- 証拠: `scripts/capture_ui_scenarios.py`、`scripts/analyze_ui_screenshots.py`、
  `next/reference/ui-observation-scenarios.json`
- 検収: `scores.json` の `source=real-window`、全 scenario `status=GREEN`、かつ
  `scores.tsv` の `score=100.0`
- 柵: `python3 scripts/test_ui_observations.py` と、波末の一度の Cargo 関門
