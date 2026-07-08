# M1: 垂直スライス「1ショット作れる」

ステータス: **確定**(M0の採否判断を受けて該当箇所を更新する)

## 実装状況(2026-07-09 更新)

| タスク | 状況 |
|---|---|
| R1 | **完了**: `oc-core::Quality`/`SampleTier`を追加。`render_frame`/`render_graph`/`render_frame_with_background_texture`へ配線。`resolution_scale`のみ実効。Finalゴールデン不変+Draft半解像度出力テスト済み |
| R2 | **完了**: `ParamRectOverlay`(center/size/color=`ParamSource`)を追加し、exportがフレーム`t`で評価。ProjectV1は定数配列とKeyframes JSONを受理。先頭/中間/末尾ゴールデン通過 |
| T0 | **部分完了**: workspace + CI(fmt/clippy/test + ffmpeg)は稼働。lavapipeのwgpu smokeテストはoc-gpu(T3)導入時に追加 |
| T1 | **完了**: `oc-core`(RationalTime/Fps/FrameDesc/PixelFormat/ColorSpace/CpuFrame)。NTSC往復・非蓄積ドリフトのテスト済み |
| T2 | **完了(2026-07-08 レビュー対応で改訂)**: `oc-media`。デコードは**生YUV420pで受ける**(ffmpegに色変換させない — 指摘#2)。probeは色タグ(matrix/range)と**回転メタデータ**(指摘#4: スマホ縦動画で寸法スワップ)を取得し、durationはfpsグリッドにスナップ(指摘#7)。Encoderは**BT.709色タグを明示出力**(指摘#5)。回転素材・色タグのテスト済み |
| T3 | **完了(2026-07-08 レビュー対応で改訂)**: `oc-gpu`。YUV→RGB変換は**係数/レンジをuniformで受け、FrameDesc.color_spaceから選択**(指摘#3: 709決め打ち廃止)。ゴールデンテストは709limited/709full/601limitedの3空間×6色でCPU参照実装と完全一致(lavapipe実測 diff=0)。`GpuCtx::from_device_queue`でUI(Slint)のデバイス共有に対応。**wgpuはSlint対応の29に統一**(指摘#1) |
| T4 | **部分完了**: `oc-testkit`。RGBAゴールデン比較(最大誤差/平均誤差/差分RGBA生成)を共通化し、`oc-gpu`のYUVゴールデンテストをtestkit経由に置き換え済み。参照PNG保存・差分ファイル出力はT7/T8の実画像ゴールデン導入時に追加 |
| T6 | **完了**: `oc-eval`(Value/KeyframeTrack/Interp/cubic_bezier_ease/DataTrack/ParamSource/DataTracks)。補間・イージング・DataTrack参照のテスト済み |
| T7 | **部分完了**: `oc-nodes`。正準座標(原点中央・Y-up・高さ=1.0)→px変換を`ViewportTransform`に集約し、`FilterPlugin`をノード経由でGPU実行する最小橋を追加。`ClearFilter(赤)`を`oc-nodes`経由で実行し、`oc-testkit`ゴールデンで一致確認済み。`OverlayNode`最小版(正準座標の矩形1つ)を追加し、偶数/奇数・整数/端数境界の複数解像度で、背景グラデーションの外側一致と先塗り漏れ検出込みのゴールデンを追加。`CompositeNode`最小版(normal over)でpremultiplied alphaの期待値式をGPUゴールデン化 |
| T12 | **部分完了**: `oc-plugin`。静的リンク版の種別レジストリ(Filter / ParamDriver / Composite)とGPUテクスチャ境界のtraitを追加。参照プラグインはCPUフレームを受け取らず、Filter/Compositeは`wgpu::CommandEncoder`へGPU render passを積む。ParamDriverはDataTrack生成を単体テスト済み。export経路で`SineParamDriver`→`DataTracks`→`ParamSource::Data`接続済み。Filterは`oc-nodes`経由のGPUゴールデンで実証済み、Compositeのalpha契約は`oc-nodes::CompositeNode`で実証済み |
| T5 | **後方移動**: GPU色解析はプラグイン/解析拡張領域であり、M1完了条件から外す。M1では合成DataTrackまたはParamDriver参照プラグインで「値列がパラメータを駆動する」境界だけを検証する |
| T11 | 未着手。M0-S1(Slint UI統合スパイク)はGUI環境が必要なため開発主機で実施すること |
| T8 | **部分完了**: 固定グラフに加え、外部GPU背景テクスチャ(動画フレームをYUV→RGBA変換したもの)を受けてOverlay/Compositeする入口を追加。動画SourceNodeの正式グラフ化は後続 |
| T9 | **部分完了**: `oc-export`を追加し、`FrameReader`→`YuvToRgba`→`oc-render`→`Encoder`の最小mp4書き出しループを実装。小さな入力動画で3フレームのmp4書き出し(`64x48 @ 12fps`)を実地確認済み。JSONプロジェクト接続は最小実装済み |
| T10 | **部分完了**: `oc-cli export-overlay`に加えて `oc-cli export-project`（versioned JSON）を追加。JSON→`oc-export`接続を最小実装し、小さな入力動画→JSON→mp4の統合テストを確認済み。キーフレーム/ParamDriver DataTrack駆動のゴールデン化済み。正式なプロジェクトE2Eサンプル同梱、合成DataTrack接続の拡張は後続 |

## 残タスクチケット(2026-07-09 監査。全消化+凍結ゲートレビューでM1完了)

上の実装状況表の「部分完了」を、1チケット=1エージェント=1PRの粒度に落としたもの。
完了したらこの表と実装状況表の両方を更新すること。

| ID | 内容 | 依存 | 完了条件(自動判定) |
|---|---|---|---|
| R1 | **完了**: `Quality { resolution_scale, precise_color, effect_samples }`を`oc-core`に定義し`render_frame`系へ配線。v1は`resolution_scale`のみ実効、他は口のみ | なし | Final(scale=1)の既存ゴールデンが全て不変。Draft(scale=2)で同グラフが「クラッシュせず出る」テストが通る(performance-modelの保証水準どおり) |
| R2 | **完了**: `ParamRectOverlay`でParamSource駆動、exportがフレームごと評価、ProjectV1がKeyframes JSONを受理。先頭/中間/末尾ゴールデン通過 | R1 | キーフレームで矩形が移動するプロジェクトの書き出しで、先頭/中間/末尾3フレームのゴールデン比較が通る(T10完了条件) |
| R3 | **完了**: `ParamDriver/DataTrack接続`。参照ParamDriver(`core.param.sine`)がDataTrackを生成し、`ParamSource::Data`+`Vec2Axes`でoverlay center.xを駆動。exportが事前構築した`DataTracks`を評価に渡す。先頭/中間/末尾ゴールデン通過 | R2 | DataTrack駆動のE2Eゴールデンが通る |
| R4 | **動画SourceNodeの正式グラフ化**(T8残)。外部引数`BackgroundTextureRequest`方式に加え、グラフが動画ソースをノードとして持てる形に | R1 | ソースノード込みグラフのゴールデンが通り、既存の外部テクスチャ経路テストも不変 |
| R5 | **T9完了条件の検証テスト**。(1)タイムコード焼き込み素材で30fps数秒を書き出し全フレームの時刻対応を検証 (2)書き出しmp4の色タグ(bt709/limited)をprobeで検証 | なし | 両テストがCIで通る |
| R6 | **oc-testkit拡充**(T4残)。参照PNG保存・差分画像ファイル出力(ゴールデン失敗時のデバッグ運用) | なし | 意図的に壊したゴールデンで差分PNGが出力されるテストが通る |
| R7 | **OverlayNode形状追加(円/線)+ Composite add/multiply**(T7残)。空間パラメータは正準座標、alpha契約は既存のnormal over式に準拠 | R1 | 各形状・各ブレンドの複数解像度ゴールデンが通る |
| R8 | **Velloの採否評価**(T7未決事項)。SVG読み込み前提のベクター描画基盤としてVello+usvgをスパイク評価し、採否を本仕様と`references.md`に記録 | なし(独立スパイク) | 採否判断がドキュメント化される(コードは使い捨て可) |
| R9 | **実素材検証**(T11)。自分のMV素材で1ショット書き出し+プレビュー/書き出しピクセル一致確認。**開発主機でのGUI(S1方式)が必要なため人間の関与必須** | R2 | 一致テストが通る+主観品質OK |

並列性: R1/R5/R6/R8は互いに独立で同時着手可。R2→R3は直列。R4/R7はR1後に並列可。R9は最後。
全チケット消化後、凍結ゲートレビュー(pitfalls-and-roadmap.mdの9項目)を実施してM2/M4/M5仕様を確定する。

補足(2026-07-09): CI(ubuntu)は`mesa-vulkan-drivers`導入済みでwgpuゴールデンテストがlavapipeで実行されている(T3で実測diff=0)。T0の「smokeテスト追加」はゴールデンテスト自体が上位互換として満たしているため、独立チケットにしない。

## 目的(退治する落とし穴)

B-3(色空間事故)、B-4(プレビュー/書き出し分岐)、C-3(評価エンジン過小評価)、D-2(自己検証基盤)。
完成物: **1本の動画を読み込み → キーフレーム/合成DataTrackでシェイプを駆動 → 合成 → mp4書き出し** ができるCLIツール。UIなし。キャッシュなし。

2026-07-08 方針修正: GPU色解析はAE的な基礎操作ではなく、最終的にはプラグイン/ユーザーコミュニティへ委ねる拡張領域である。M1では「解析そのもの」を必須にせず、`oc-eval`のキーフレーム・DataTrack参照、標準シェイプ制御、Compositeの基礎を優先する。DataTrack境界の検証は、合成テストデータまたはParamDriver参照プラグインで足りる。

## スコープ外

- UI(M3)、Undo/ドキュメント編集(M2)、キャッシュ(M4)、3D/glTF(M5)、音声再生(M2)、動的プラグインロード(v2)、OpenCV(M4)

## Cargo workspace構成(このフェーズで確立し、凍結ゲートの分割単位になる)

```
crates/
  oc-core     フレーム記述子・有理数時間・エラー型(全クレートの共通語彙)
  oc-media    ffmpegサイドカー: probe / デコードIterator / エンコードパイプ
  oc-gpu      wgpu初期化・テクスチャ管理・YUV→RGB変換・シェーダ共通基盤
  oc-eval     パラメータ評価: キーフレーム補間・イージング・データ列参照
  oc-nodes    ノードtrait + 標準ノード(オーバーレイ生成・コンポジット)
  oc-plugin   静的リンク版プラグイン種別レジストリ(Filter/ParamDriver/Composite)
  oc-testkit  ゴールデンイメージテスト基盤(dev-dependency)
  oc-render   グラフ実行器: render(t) → フレーム(プレビューと書き出しの共通経路)
  oc-export   書き出しループ(連番 / ffmpegエンコードパイプ)
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

// oc-eval: 評価器は「時刻→値」の純関数。DataTrack参照もキーフレームと同一機構
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

// oc-plugin(v1): プラグインは種別レジストリへ静的登録する。dylib/配布はv2。
// Render系プラグインは必ずGPUテクスチャ境界で、CPUフレームを受け渡す製品経路は持たない。
pub enum PluginKind { Input, Filter, ParamDriver, Composite, ScriptWasm }
pub trait FilterPlugin {
    fn render(&self, gpu: &GpuCtx, encoder: &mut wgpu::CommandEncoder,
              t: RationalTime, params: &ResolvedParams,
              input: TextureRef, output: TextureRef);
}
pub trait CompositePlugin {
    fn render(&self, gpu: &GpuCtx, encoder: &mut wgpu::CommandEncoder,
              t: RationalTime, params: &ResolvedParams,
              inputs: &[TextureRef], output: TextureRef);
}
pub trait ParamDriverPlugin {
    fn build_track(&self, ctx: ParamDriverContext, params: &ResolvedParams) -> DataTrack;
}

// oc-render: プレビューも書き出しもこの1関数のみを通る(B-4)
pub fn render_frame(graph: &Graph, gpu: &GpuCtx, t: RationalTime, q: Quality) -> TextureHandle;

// Quality: performance-model.mdのプレビュー品質モードの実体。
// Draft=半解像度/fp16/色ショートカット許容/draftサンプル数、Final=厳密。
// ゴールデンテストはFinalに対してのみ厳密比較する。
pub struct Quality {
    pub resolution_scale: u32,     // 1 = full, 2 = 1/2, 4 = 1/4
    pub precise_color: bool,       // false: sRGB空間ブレンド等の近似を許容
    pub effect_samples: SampleTier, // Draft | Full(モーションブラー等プラグインのサンプル数の口)
}
```

## タスク分割

| ID | 内容 | 依存 | 完了条件 |
|---|---|---|---|
| T0 | workspace雛形 + CI(fmt/clippy/test、lavapipe(ソフトウェアVulkan)でwgpuテストが走る環境) | M0完了 | CIがグリーン。lavapipe上でwgpuのcompute1本が実行されるsmokeテストが通る |
| T1 | oc-core: `RationalTime`(S3の型)・`FrameDesc`・`PixelFormat`/`ColorSpace`・変換ユーティリティ | T0 | 単体テスト(fps変換・丸め・境界値)が通る |
| T2 | oc-media: probe(解像度/fps/長さ)+ デコードIterator + フレーム正確シーク(S2の採否に従いffmpeg-sidecar利用 or 自前パイプ) | T1 | 実素材とテスト用生成素材で「フレームNを要求→正しいフレームが返る」テストが通る(先頭/末尾/ランダム10点) |
| T3 | oc-gpu: device初期化・テクスチャアップロード・YUV(Rec.709 limited)→リニアRGB変換シェーダ | T1 | カラーバー素材の変換結果が理論値±1/255で一致する数値テストが通る(B-3) |
| T4 | oc-testkit: ゴールデンイメージ比較(参照PNG・許容誤差・差分画像出力)+ テスト用素材生成(カラーバー動画等) | T3 | oc-gpuのYUVテストがtestkit経由に置き換わり、CIで安定して通る |
| T5 | 後続へ移動: oc-analyze(GPU色解析/トラッキング)。M1では実装しない | - | M4/解析プラグイン側で再定義する |
| T6 | oc-eval: キーフレームトラック(線形/ベジェ/ホールド)+ イージング + DataTrack参照 + 単体テスト | T1 | 補間の数値テスト、DataTrack参照込みの評価テストが通る(C-3) |
| T7 | oc-nodes: `RenderNode` trait + SourceNode(動画フレーム)+ OverlayNode(パラメータ駆動の2D図形: 円/矩形/線、色・位置・サイズ)+ CompositeNode(normal/add/multiply、premultiplied正しく) | T3, T6 | 各ノード単体のゴールデンイメージテストが通る |
| T8 | oc-render: グラフ実行器 `render_frame()`(トポロジカル順に実行、毎回全計算) | T7 | 「ソース→オーバーレイ→合成」グラフのゴールデンイメージテストが通る |
| T9 | oc-export: `render_frame()`ループ→PNG連番 / ffmpegエンコードパイプ→mp4 | T2, T8 | 30fpsの数秒グラフを書き出し、フレーム数・時刻対応が正確(全フレームにタイムコード焼き込みで検証)。**書き出しmp4の色タグ(bt709/limited)をprobeで検証**(レビュー指摘#5。Encoderのタグ付けは実装・テスト済み) |
| T10 | oc-cli: プロジェクト設定ファイル(バージョンフィールド付きJSON)を読み、キーフレーム/合成DataTrack評価→シェイプ制御→レンダ→書き出しを一気通貫実行。サンプルプロジェクト同梱 | T6, T9 | サンプルプロジェクトのE2Eゴールデンテスト(先頭/中間/末尾の3フレーム比較)がCIで通る。M1時点ではGPU解析を含めない |
| T11 | 実素材検証: 自分のMV素材で1ショット(数秒)を書き出す。プレビュー(M0-S1採用方式での簡易表示)と書き出しのピクセル一致確認 | T10 | 書き出したショットがMV制作に使える品質(主観)+ プレビュー/書き出し一致テスト(B-4)が通る |
| T12 | oc-plugin: 静的リンク版の種別レジストリ + 参照プラグイン(Filter / ParamDriver / Composite)。Render系は`wgpu::Texture` in/outのみ | T3, T6 | 種別レジストリの単体テストが通る。参照Filter/CompositeがGPU render passを発行し、T7/T8のゴールデンで実レンダ検証される。CPUフレームを受け取るプラグイン経路が存在しない |

## 並列レーン

```
T0 → T1 →┬ レーンA(media): T2 ───────────────┐
          ├ レーンB(gpu):   T3 → T4 ──────────┤
          ├ レーンC(eval):  T6 ────────────────┼→ T7 → T8 → T9 → T10 → T11
          └(T7はT3,T6完了後に着手可)──────────┘
```

T1完了後、最大3エージェント(A/B/C)並列。T7以降は直列に近いが、T9(export)はT8と部分並列可(モックグラフで先行実装)。

## フェーズ完了条件

- T0〜T12のうちT5を除く全タスクのCI完了条件が通っている
- 凍結ゲートのレビューを実施: `FrameDesc`・`RenderNode`・`ParamEval`・時間型・workspace分割に加え、**正準座標系(F-1)・並行性/所有権モデル(F-2)・単一評価モデル(F-3)・クリップ時間写像(F-4)**を凍結し、M2/M4/M5仕様書を確定させる(pitfalls-and-roadmap.md「凍結ゲート」の9項目)

注(2026-07-08、F-1): T7のOverlayNode等で空間パラメータ(位置・サイズ・ブラー半径)を実装する際、**絶対pxではなく正準空間(単位なし・原点中央・Y-up・高さ=1.0)の値で受け、px変換はレンダ直前の1箇所に集約**する。Draft(半解像度)とFinalの見た目一致(B-4の約束)はこれが前提。

注(2026-07-08、alpha): 内部レンダターゲット/合成は**premultiplied alpha**を正規形にする。UIやJSON等のユーザー入力色はstraight alphaとして受け、レンダ直前またはComposite境界でpremulへ変換する。T7のOverlay最小版はalpha=1.0のみを実証対象とし、alpha<1の期待値式は`CompositeNode`のnormal overで固定済み: `out.rgb = fg.rgb + bg.rgb * (1 - fg.a)`, `out.a = fg.a + bg.a * (1 - fg.a)`。

## 未決事項

- プロジェクト設定ファイルのスキーマ詳細(T10で最小形を決め、M2で正式化)
- ~~OverlayNodeのベクター描画を自前シェーダにするかVello依存にするか~~ → **方針確定(2026-07-07)**: SVG読み込みがコア機能に決まったため(concept.md)、T7ではVello(+将来のusvg接続)を前提に評価する。Velloが性能・安定性で不合格だった場合のみ自前シェーダに退避し、SVG対応方針を再検討する
