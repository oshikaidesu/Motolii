# M1: 垂直スライス「1ショット作れる」

ステータス: **確定**(M0の採否判断を受けて該当箇所を更新する)

## 実装状況(2026-07-07 更新)

| タスク | 状況 |
|---|---|
| T0 | **部分完了**: workspace + CI(fmt/clippy/test + ffmpeg)は稼働。lavapipeのwgpu smokeテストはoc-gpu(T3)導入時に追加 |
| T1 | **完了**: `oc-core`(RationalTime/Fps/FrameDesc/PixelFormat/ColorSpace/CpuFrame)。NTSC往復・非蓄積ドリフトのテスト済み |
| T2 | **完了**: `oc-media`(probe/FrameReader/read_frame_at/Encoder)。ffmpeg実機での往復・フレーム正確シークテスト済み。S2の判断は[docs/spikes/s2-decode.md](../spikes/s2-decode.md)(ffmpeg-sidecarクレート不採用、自前パイプ採用) |
| T6 | **完了**: `oc-eval`(Value/KeyframeTrack/Interp/cubic_bezier_ease/DataTrack/ParamSource/DataTracks)。補間・イージング・DataTrack参照のテスト済み |
| T3-T5, T7-T11 | 未着手。M0-S1(プレビューブリッジ)はGUI環境が必要なため開発主機で実施すること |

## 目的(退治する落とし穴)

B-3(色空間事故)、B-4(プレビュー/書き出し分岐)、C-3(評価エンジン過小評価)、D-2(自己検証基盤)。
完成物: **1本の動画を読み込み → GPU色解析 → 解析データ駆動のプロシージャルオーバーレイ → 合成 → mp4書き出し** ができるCLIツール。UIなし。キャッシュなし。

## スコープ外

- UI(M3)、Undo/ドキュメント編集(M2)、キャッシュ(M4)、3D/glTF(M5)、音声再生(M2)、動的プラグインロード(v2)、OpenCV(M4)

## Cargo workspace構成(このフェーズで確立し、凍結ゲートの分割単位になる)

```
crates/
  oc-core     フレーム記述子・有理数時間・エラー型(全クレートの共通語彙)
  oc-media    ffmpegサイドカー: probe / デコードIterator / エンコードパイプ
  oc-gpu      wgpu初期化・テクスチャ管理・YUV→RGB変換・シェーダ共通基盤
  oc-eval     パラメータ評価: キーフレーム補間・イージング・データ列参照
  oc-analyze  GPU色解析(ヒストグラム・支配色・色マスク重心)
  oc-nodes    ノードtrait + 標準ノード(オーバーレイ生成・コンポジット)
  oc-render   グラフ実行器: render(t) → フレーム(プレビューと書き出しの共通経路)
  oc-export   書き出しループ(連番 / ffmpegエンコードパイプ)
  oc-testkit  ゴールデンイメージテスト基盤(dev-dependency)
  oc-cli      M1ドライバ(プロジェクト設定ファイル → 書き出し)
```

## インターフェース契約(並列タスクの境界。変更は仕様書改訂PRを先に)

```rust
// oc-core(S3の結果で最終化)
pub struct RationalTime { pub num: i64, pub den: i64 }   // 秒 = num/den
pub struct FrameDesc {
    pub width: u32, pub height: u32, pub stride: u32,
    pub format: PixelFormat,      // wgpu::TextureFormat準拠 + Yuv420p等
    pub color_space: ColorSpace,  // LinearRgb | Srgb | Rec709Limited | ...
    pub premultiplied: bool,
}

// oc-eval: 評価器は「時刻→値」の純関数。トラッキング結果参照もキーフレームと同一機構
pub enum ParamSource { Keyframes(KeyframeTrack), DataTrack(DataTrackId), Const(Value) }
pub trait ParamEval { fn eval(&self, t: RationalTime, ctx: &DataTracks) -> Value; }

// oc-nodes: ノードは状態を持たない。時刻とパラメータ・入力テクスチャから出力を決定。
// エフェクトモデルの決定(concept.md)により、ユーザーに見えるのは「レイヤーに積む
// エフェクトスタック」であり、これはRenderNodeの線形チェーンとして展開される。
// RenderNodeの単純な境界(入力テクスチャ+パラメータ→出力)はそのまま
// エフェクトプラグインの境界になる(LLMでのプラグイン自作を想定した書きやすさ最優先)。
pub trait RenderNode {
    fn describe(&self) -> NodeDesc; // 入出力数・パラメータ定義
    fn render(&self, gpu: &GpuCtx, t: RationalTime,
              params: &ResolvedParams, inputs: &[TextureHandle]) -> TextureHandle;
}

// oc-analyze: 解析は「動画区間 → データ列」。結果はoc-evalのDataTrackとして参照される
pub trait Analyzer {
    fn analyze(&self, gpu: &GpuCtx, frames: impl Iterator<Item = Frame>) -> DataTrack;
}

// oc-render: プレビューも書き出しもこの1関数のみを通る(B-4)
pub fn render_frame(graph: &Graph, gpu: &GpuCtx, t: RationalTime, q: Quality) -> TextureHandle;
```

## タスク分割

| ID | 内容 | 依存 | 完了条件 |
|---|---|---|---|
| T0 | workspace雛形 + CI(fmt/clippy/test、lavapipe(ソフトウェアVulkan)でwgpuテストが走る環境) | M0完了 | CIがグリーン。lavapipe上でwgpuのcompute1本が実行されるsmokeテストが通る |
| T1 | oc-core: `RationalTime`(S3の型)・`FrameDesc`・`PixelFormat`/`ColorSpace`・変換ユーティリティ | T0 | 単体テスト(fps変換・丸め・境界値)が通る |
| T2 | oc-media: probe(解像度/fps/長さ)+ デコードIterator + フレーム正確シーク(S2の採否に従いffmpeg-sidecar利用 or 自前パイプ) | T1 | 実素材とテスト用生成素材で「フレームNを要求→正しいフレームが返る」テストが通る(先頭/末尾/ランダム10点) |
| T3 | oc-gpu: device初期化・テクスチャアップロード・YUV(Rec.709 limited)→リニアRGB変換シェーダ | T1 | カラーバー素材の変換結果が理論値±1/255で一致する数値テストが通る(B-3) |
| T4 | oc-testkit: ゴールデンイメージ比較(参照PNG・許容誤差・差分画像出力)+ テスト用素材生成(カラーバー動画等) | T3 | oc-gpuのYUVテストがtestkit経由に置き換わり、CIで安定して通る |
| T5 | oc-analyze: ヒストグラム・支配色抽出・色距離マスクの重心追跡(いずれもcompute shader)+ CPU参照実装との一致テスト | T3, T4 | 合成テスト素材(既知の色の矩形が移動)で重心軌跡がCPU参照実装と一致 |
| T6 | oc-eval: キーフレームトラック(線形/ベジェ/ホールド)+ イージング + DataTrack参照 + 単体テスト | T1 | 補間の数値テスト、DataTrack参照込みの評価テストが通る(C-3) |
| T7 | oc-nodes: `RenderNode` trait + SourceNode(動画フレーム)+ OverlayNode(パラメータ駆動の2D図形: 円/矩形/線、色・位置・サイズ)+ CompositeNode(normal/add/multiply、premultiplied正しく) | T3, T6 | 各ノード単体のゴールデンイメージテストが通る |
| T8 | oc-render: グラフ実行器 `render_frame()`(トポロジカル順に実行、毎回全計算) | T7 | 「ソース→オーバーレイ→合成」グラフのゴールデンイメージテストが通る |
| T9 | oc-export: `render_frame()`ループ→PNG連番 / ffmpegエンコードパイプ→mp4 | T2, T8 | 30fpsの数秒グラフを書き出し、フレーム数・時刻対応が正確(全フレームにタイムコード焼き込みで検証) |
| T10 | oc-cli: プロジェクト設定ファイル(バージョンフィールド付きJSON)を読み、解析→評価→レンダ→書き出しを一気通貫実行。サンプルプロジェクト同梱 | T5, T6, T9 | サンプルプロジェクトのE2Eゴールデンテスト(先頭/中間/末尾の3フレーム比較)がCIで通る |
| T11 | 実素材検証: 自分のMV素材で1ショット(数秒)を書き出す。プレビュー(M0-S1採用方式での簡易表示)と書き出しのピクセル一致確認 | T10 | 書き出したショットがMV制作に使える品質(主観)+ プレビュー/書き出し一致テスト(B-4)が通る |

## 並列レーン

```
T0 → T1 →┬ レーンA(media): T2 ───────────────┐
          ├ レーンB(gpu):   T3 → T4 → T5 ──────┤
          ├ レーンC(eval):  T6 ────────────────┼→ T7 → T8 → T9 → T10 → T11
          └(T7はT3,T6完了後に着手可)──────────┘
```

T1完了後、最大3エージェント(A/B/C)並列。T7以降は直列に近いが、T9(export)はT8と部分並列可(モックグラフで先行実装)。

## フェーズ完了条件

- T11まで全タスクのCI完了条件が通っている
- 凍結ゲートのレビューを実施: `FrameDesc`・`RenderNode`・`ParamEval`・時間型・workspace分割を凍結し、M2/M4/M5仕様書を確定させる

## 未決事項

- プロジェクト設定ファイルのスキーマ詳細(T10で最小形を決め、M2で正式化)
- ~~OverlayNodeのベクター描画を自前シェーダにするかVello依存にするか~~ → **方針確定(2026-07-07)**: SVG読み込みがコア機能に決まったため(concept.md)、T7ではVello(+将来のusvg接続)を前提に評価する。Velloが性能・安定性で不合格だった場合のみ自前シェーダに退避し、SVG対応方針を再検討する
