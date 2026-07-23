# M0: リスク退治スパイク

ステータス: **確定・歴史的milestone**。S1の測定事実は維持するが、UI採用結論は2026-07-18の[egui判断](../reviews/2026-07-18-m3-egui-selection.md)で置換された。

## 目的(退治する落とし穴)

- A-1: 当時のSlint UI統合。旧WebView bridgeは2026-07-08に廃止し、現在はegui既存device/native texture共有で構造的解消
- B-2: 動画デコード(シーク・VFR)
- B-1: 時間表現と再生クロック

スパイクは**使い捨てコード**。品質・構造は不問、`spikes/`ディレクトリに隔離し、本体クレートには一切取り込まない。成果物はコードではなく**実測値と採否判断の記録**(`docs/spikes/`に1スパイク1レポート)。

## スコープ外

- 本体のクレート設計、プラグインtrait、キャッシュ、3D、音声実装(時間表現の設計のみ行い、音声再生はやらない)

## 操作単純化モデルへの割当（完了済み・遡及変更なし）

[操作単純化モデル](../interaction-simplicity-model.md)に対し、M0は製品操作を作るフェーズではなく成立性の測定を担当した。S1のrender負荷中drag/IME、S2のseek、S3の有理時間が後続のDirect操作を作法や丸め誤差へ依存させない土台である。完了済みM0へ機能を追加しない。今後も成立性が未証明のUI案は本体へ直接入れず、独立spikeで測ってから仕様へ戻す。

## タスク分割

| ID | 内容 | 依存 | 完了条件 |
|---|---|---|---|
| S1 | **(2026-07-08 改訂3)** Slint検証スパイク(`spikes/s1-slint/`)。UI基盤方針をWebViewからSlintに転換したため、旧S1a/S1b(WebViewブリッジ2方式)は廃止。検証項目: (1) **motolii-gpuが作ったテクスチャをSlintに渡すE2E結線**(レビュー指摘#1対応: workspaceのwgpuをSlint対応の29に統一し、`GpuCtx::new_for_ui`のdeviceを`WGPUConfiguration::Manual`で共有)、(2) 30fps更新とUI操作の共存、(3) 日本語IME入力(LineEditで変換→確定)、(4) 日本語ラベル表示、(5) タイムライン風ドラッグ操作。**実行時落とし穴**として「`require_wgpu_29`なのにOpenGLレンダラが選ばれる」ミスマッチを確認し、`slint` featureを`renderer-femtovg-wgpu`明示に固定して回避 | なし | `docs/spikes/s1-slint.md` に合否(特にIME)を記録し、M3仕様を確定 |
| S2 | `ffmpeg-sidecar`クレートを比較し、結果として不採用。自前ffprobe／ffmpeg子process pipeでraw YUV取得、GPU色変換、CFRフレーム正確seekを確認した歴史spike。VFR実素材、長尺／4K、停止中readの確実なcancelは未証明 | なし | [S2結果](../spikes/s2-decode.md)と[現行適用範囲](../reviews/2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md)に採否、成立範囲、再入場条件を記録 |
| S3 | 有理数時間型(`i64分子/i64分母`)の設計メモ: フレーム番号との相互変換、異なるfpsのクリップ混在、音声サンプル位置との対応。コードは最小限(型と変換関数+テストのみ) | なし | `docs/spikes/s3-time.md` に型定義とM1で使う最終形を記録 |

## 並列レーン

S1 / S2 / S3 は**全て独立・同時着手可**(S2/S3は完了済み)。

## S1の判断基準(事前固定)

| 基準 | 合格ライン |
|---|---|
| プレビュー描画 | 1080p相当のテクスチャが30fps更新で表示され、UI操作(ドラッグ・入力)と共存 |
| **デバイス主導権** | `WGPUConfiguration::Manual`で**コンポジタ要件(feature/limit)を明示したデバイス**を渡して起動できる(第2回レビュー#1。Slint任せの`default()`は不可 — featureは後から足せない。実装済み: `GpuCtx::new_for_ui()`) |
| **スレッド分離** | レンダは専用スレッド、UIスレッドは受信テクスチャの表示のみ(第2回レビュー#2。実装済み: mpscチャネル+try_send最新フレーム方式)。ドラッグ・IME入力がレンダ負荷でジャンクしないこと |
| **日本語IME** | LineEditで日本語を変換しながら入力・確定できる(変換候補ウィンドウの位置も含め実用レベル) |
| 日本語表示 | ラベル・ボタンの日本語が化けずに表示される |
| カスタム操作 | タイムライン風ドラッグが体感即応 |
| **フレーム完全性** | プレビューにtearing/未完成フレームの混入が見えない(第3回レビュー#3: レンダスレッドのsubmitとSlintの読み取りの同期は単一キューの順序保証頼みのため、目視で明示検証する。異常が見えたらフェンス導入を検討) |

不合格(特にIME)の場合の退避順: egui(IMEを妥協) → Tauri+WebView(ブリッジコストを払う)。判断理由の詳細は会話記録より: SlintはwgpuゼロコピーとIME実績([2025年Rust GUI調査](https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html)で合格)を両立する唯一のRustネイティブ候補。

## フェーズ完了条件

3本のスパイクレポートが揃い、S1判断・S2クレート採否・S3時間型がM1/M3仕様書に反映されていること。
