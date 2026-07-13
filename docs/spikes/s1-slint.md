# S1 Slint スパイク結果

作成日: 2026-07-08 / INF-1更新: 2026-07-11

## 結論(INF-1 **合格**)

人間判定でOK(開発主機・Apple M4 / Metal)。Slint採用を確定。退避(egui / Tauri)は不要。

| 層 | 内容 | 状態 | 証拠 |
|---|---|---|---|
| 1 構造 | Manual共有デバイス・`Image::try_from`・YUV→RGBA 1280×720 | **合格** | [struct-manifest.json](s1-evidence/struct-manifest.json) + PNG3枚 |
| 2 証拠 | ウィンドウ表示・日本語ラベル・プレビュー更新・IME/タイムラインUI | **合格** | [10-window-id.png](s1-evidence/10-window-id.png) / [13-window-id-b.png](s1-evidence/13-window-id-b.png) |
| 3 体感 | IME実用感・再生tearing | **合格**(人間判定) | 開発主機で事前確認済み |

層1の再現: `S1_EVIDENCE_DIR=… S1_EVIDENCE_ONLY=1 cargo run` (`spikes/s1-slint/`)

## 層1 実測(2026-07-11)

- adapter: **Apple M4 (Metal)** / IntegratedGpu
- `GpuCtx::new_for_ui` → `WGPUConfiguration::Manual` → `BackendSelector::require_wgpu_29` → 起動OK
- 3色相フレームで `slint::Image::try_from(texture)` 成功
- GPU download PNG: `struct-frame-00-hue0.png` / `01-hue120` / `02-hue240` (1280×720)

## 層2 実測(2026-07-11・蓋開き)

- ウィンドウタイトル/日本語ラベル化けなし
- プレビュー色相がフレーム間で変化(別スレッドレンダ→埋め込み)
- LineEditフォーカス・IMEインジケータ「あ」表示
- タイムライン playhead 表示

M3本番の深いIMEチェックリスト(OS横断・長文など)は [M3実装ガード1](../specs/M3-ui-integration.md) に移譲。INF-1の基盤採否判定としては合格で閉じる。

---

## 履歴: OpenGL/WGPU29 ミスマッチ(2026-07-08)

### 結論

- `WGPUConfiguration::Manual` と `require_wgpu_29()` の経路は有効。
- 今回の起動失敗は依存欠落ではなく、**OpenGLレンダラとWGPU29要求の実行時ミスマッチ**。
- `slint` の feature を `renderer-femtovg-wgpu` 前提に固定したことで解消し、GUI起動まで確認した。

### 発生したエラー

`spikes/s1-slint` の `cargo run` 実行時に以下で停止:

`Error: WGPU 29.x rendering is not supported with an OpenGL renderer`

### 原因分析

- `spikes/s1-slint/Cargo.toml` が `slint` デフォルト機能のままだと、
  `renderer-femtovg` (OpenGL) が有効化される。
- 一方で実装側は `BackendSelector::require_wgpu_29()` で WGPU29 経路を強制している。
- その結果、**「OpenGLレンダラで初期化されたバックエンド」vs「WGPU29必須」** が衝突して失敗する。

### 対応

`slint` を `default-features = false` にして、feature を明示固定:

- `compat-1-2`
- `std`
- `backend-winit`
- `renderer-femtovg-wgpu`
- `unstable-wgpu-29`

これにより OpenGL 経路が混入せず、`cargo run` で GUI 起動を確認。

### 他リポジトリでの解決パターン

- `tronical/slint-off-thread-rendering`:
  - `default-features = false` で不要レンダラを避ける
  - `backend-winit` + `renderer-skia` + `unstable-wgpu-29` を明示
  - `require_wgpu_29(WGPUConfiguration::Manual { ... })` で同一 device を共有
- Slint公式ドキュメント:
  - Winit バックエンドは renderer を feature/`SLINT_BACKEND` で明示選択する前提
  - `require_wgpu_29()` を使うなら、WGPU対応レンダラをコンパイル時に有効化する必要がある

### 実行メモ

- 依存解決: `cargo tree -e features` で `renderer-femtovg` (OpenGL) を検出
- 修正後: `renderer-femtovg-wgpu` が有効化され、起動時の OpenGL ミスマッチは消失
