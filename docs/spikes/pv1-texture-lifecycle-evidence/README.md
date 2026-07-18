# PV-1 証跡ディレクトリ

`spikes/pv1-texture-lifecycle` の人間実機審判結果を置く。自動 `cargo test` 緑だけでは **overall を pass にしない**。

## ファイル

| ファイル | 用途 |
|---|---|
| `manifest-skeleton.json` | 初期骨格。全項目 `pending` |
| （実走後）開発主機が手更新する manifest | H1–H6・backend の実測記録 |

## 更新手順

1. `export CARGO_TARGET_DIR=/private/tmp/motolii-pv1-target && cd spikes/pv1-texture-lifecycle && cargo run --release`
2. 任意: `PV1_EVIDENCE_DIR=../../docs/spikes/pv1-texture-lifecycle-evidence cargo run --release`
   - **証跡 JSON は Status 受信毎ではなく、アプリ終了時（Shutdown→worker join 後）に 1 回だけ** `manifest-skeleton.json` の実走時刻・counter・最終stateを更新する。既存の人間判定は保持する
3. 人間審判完了後、各 `human_checks` / `backends` の `verdict` を手で `pass` または `fail` に更新
4. **未実測の backend（DX12/Vulkan 等）は `pending` のまま** — pass と記録しない
5. 全必須項目が pass のときのみ `overall` を `pass` に変更

## H3 / H4

Hide / Minimize ボタン押下後、single-shot Timer で自動 Show / Restore が走る。手動 Show / Restore ボタンは補助。

## pending 規則

- クラウド CI / ヘッドレス環境では人間項目は常に pending
- Metal 以外を実行できない環境では DX12/Vulkan 行は pending を維持
- 自動テスト成功を人間合格の代替にしない
