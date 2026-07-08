# S1 Slint スパイク結果

作成日: 2026-07-08

## 結論

- `WGPUConfiguration::Manual` と `require_wgpu_29()` の経路は有効。
- 今回の起動失敗は依存欠落ではなく、**OpenGLレンダラとWGPU29要求の実行時ミスマッチ**。
- `slint` の feature を `renderer-femtovg-wgpu` 前提に固定したことで解消し、GUI起動まで確認した。

## 発生したエラー

`spikes/s1-slint` の `cargo run` 実行時に以下で停止:

`Error: WGPU 29.x rendering is not supported with an OpenGL renderer`

## 原因分析

- `spikes/s1-slint/Cargo.toml` が `slint` デフォルト機能のままだと、
  `renderer-femtovg` (OpenGL) が有効化される。
- 一方で実装側は `BackendSelector::require_wgpu_29()` で WGPU29 経路を強制している。
- その結果、**「OpenGLレンダラで初期化されたバックエンド」vs「WGPU29必須」** が衝突して失敗する。

## 対応

`slint` を `default-features = false` にして、feature を明示固定:

- `compat-1-2`
- `std`
- `backend-winit`
- `renderer-femtovg-wgpu`
- `unstable-wgpu-29`

これにより OpenGL 経路が混入せず、`cargo run` で GUI 起動を確認。

## 他リポジトリでの解決パターン

- `tronical/slint-off-thread-rendering`:
  - `default-features = false` で不要レンダラを避ける
  - `backend-winit` + `renderer-skia` + `unstable-wgpu-29` を明示
  - `require_wgpu_29(WGPUConfiguration::Manual { ... })` で同一 device を共有
- Slint公式ドキュメント:
  - Winit バックエンドは renderer を feature/`SLINT_BACKEND` で明示選択する前提
  - `require_wgpu_29()` を使うなら、WGPU対応レンダラをコンパイル時に有効化する必要がある

## 実行メモ

- 依存解決: `cargo tree -e features` で `renderer-femtovg` (OpenGL) を検出
- 修正後: `renderer-femtovg-wgpu` が有効化され、起動時の OpenGL ミスマッチは消失

