# M1: 垂直スライス「1ショット作れる」

ステータス: **確定・歴史的milestone**(M0の採否判断を受けて該当箇所を更新した)。UI toolkitは2026-07-18にeguiへ変更したが、M1のrender/media契約は変更しない。

## 実装状況(2026-07-10 更新)

| タスク | 状況 |
|---|---|
| R1 | **完了**: `motolii-core::Quality`/`SampleTier`を追加。`render_frame`/`render_graph`/`render_frame_with_background_texture`へ配線。`resolution_scale`のみ実効。Finalゴールデン不変+Draft半解像度出力テスト済み |
| R2 | **完了**: `ParamRectOverlay`(center/size/color=`ParamSource`)を追加し、exportがフレーム`t`で評価。ProjectV1は定数配列とKeyframes JSONを受理。先頭/中間/末尾ゴールデン通過 |
| T0 | **部分完了**: workspace + CI(fmt/clippy/test + ffmpeg)は稼働。lavapipeのwgpu smokeテストはmotolii-gpu(T3)導入時に追加 |
| T1 | **完了**: `motolii-core`(RationalTime/Fps/FrameDesc/PixelFormat/ColorSpace/CpuFrame)。NTSC往復・非蓄積ドリフトのテスト済み |
| T2 | **完了(2026-07-08 レビュー対応で改訂)**: `motolii-media`。デコードは**生YUV420pで受ける**(ffmpegに色変換させない — 指摘#2)。probeは色タグ(matrix/range)と**回転メタデータ**(指摘#4: スマホ縦動画で寸法スワップ)を取得し、durationはfpsグリッドにスナップ(指摘#7)。Encoderは**BT.709色タグを明示出力**(指摘#5)。回転素材・色タグのテスト済み |
| T3 | **完了(2026-07-08 レビュー対応で改訂)**: `motolii-gpu`。YUV→RGB変換は**係数/レンジをuniformで受け、FrameDesc.color_spaceから選択**(指摘#3: 709決め打ち廃止)。ゴールデンテストは709limited/709full/601limitedの3空間×6色でCPU参照実装と完全一致(lavapipe実測 diff=0)。`GpuCtx::from_device_queue`でUI(Slint)のデバイス共有に対応。**wgpuはSlint対応の29に統一**(指摘#1) |
| T4 | **完了**: `motolii-testkit`。RGBAゴールデン比較(最大誤差/平均誤差/差分RGBA生成)を共通化し、`motolii-gpu`のYUVゴールデンテストをtestkit経由に置き換え済み。参照/actual/diffのPNG保存と`OC_TESTKIT_ARTIFACT_DIR`によるゴールデン失敗時の差分出力を追加 |
| T6 | **完了**: `motolii-eval`(Value/KeyframeTrack/Interp/cubic_bezier_ease/DataTrack/ParamSource/DataTracks)。補間・イージング・DataTrack参照のテスト済み |
| T7 | **完了**: `motolii-nodes`。正準座標→`ViewportTransform`集約。`FilterPlugin`/`OverlayNode`(矩形・円・線)/`CompositeNode`(normal/add/multiply)をGPUゴールデン化。Vello統合は凍結ゲート後(R8採用済み) |
| T12 | **完了(所見1+2)**: レジストリディスパッチ(`RenderStep::Plugin`)+ホスト所有`PipelineCache`+参照`TintFilter`で再コンパイルなし実証。`AssetRef`予約済み（Document結線も後続M2で完了）。GpuAssetCache/ImporterはM2完了範囲ではなく未実装・未凍結 |
| T5 | **後方移動**: GPU色解析はプラグイン/解析拡張領域であり、M1完了条件から外す。M1では合成DataTrackまたはParamDriver参照プラグインで「値列がパラメータを駆動する」境界だけを検証する |
| T11 | **完了(2026-07-10)**: 実素材(1080p/4K)で書き出し+GUIプレビュー(`spikes/r9-preview`)+主観品質OK。B-4はCI合成素材+手元実素材(`qp0: false`書き出し)で確認 |
| T8 | **完了**: `RenderStep::VideoSource`+グラフ合流(R4)。中間RTは`RenderSession`ピンポン2枚でフレーム間再利用。Solid単色もセッションキャッシュ |
| T9 | **完了**: `motolii-export`を追加し、`FrameReader`→`YuvToRgba`→`motolii-render`→`Encoder`の最小mp4書き出しループを実装。30fps×3秒のタイムコード焼き込み素材で全フレームの時刻対応を検証(R5)。書き出しmp4のBT.709 limited色タグをprobeで検証(R5) |
| T10 | **完了**: `motolii-cli export-overlay` / `export-project`(versioned JSON)。JSON→`motolii-export`一気通貫。**M1出口デモ(実写(生成)背景 + Bezierイージングで右へ流れる矩形 → mp4)を正式サンプル `samples/exit-demo/` として同梱**し、**E2Eゴールデン `crates/motolii-cli/tests/exit_demo.rs`**(export経路でmp4化→出力mp4をデコード→先頭/中間/末尾で「背景動画の透過」+「矩形のイージング位置(左→右)」を検証。lavapipe+ffmpegで緑)を追加。キーフレーム/ParamDriver DataTrack駆動のゴールデンも済み。合成DataTrack接続の拡張は後続 |

## 残タスクチケット(2026-07-09 監査。全消化+凍結ゲートレビューでM1完了)

上の実装状況表の「部分完了」を、1チケット=1エージェント=1PRの粒度に落としたもの。
完了したらこの表と実装状況表の両方を更新すること。

| ID | 内容 | 依存 | 完了条件(自動判定) |
|---|---|---|---|
| R1 | **完了**: `Quality { resolution_scale, precise_color, effect_samples }`を`motolii-core`に定義し`render_frame`系へ配線。v1は`resolution_scale`のみ実効、他は口のみ | なし | Final(scale=1)の既存ゴールデンが全て不変。Draft(scale=2)で同グラフが「クラッシュせず出る」テストが通る(performance-modelの保証水準どおり) |
| R2 | **完了**: `ParamRectOverlay`でParamSource駆動、exportがフレームごと評価、ProjectV1がKeyframes JSONを受理。先頭/中間/末尾ゴールデン通過 | R1 | キーフレームで矩形が移動するプロジェクトの書き出しで、先頭/中間/末尾3フレームのゴールデン比較が通る(T10完了条件) |
| R3 | **完了**: `ParamDriver/DataTrack接続`。参照ParamDriver(`core.param.sine`)がDataTrackを生成し、`ParamSource::Data`+`Vec2Axes`でoverlay center.xを駆動。exportが事前構築した`DataTracks`を評価に渡す。先頭/中間/末尾ゴールデン通過 | R2 | DataTrack駆動のE2Eゴールデンが通る |
| R4 | **完了**: 動画SourceNodeの正式グラフ化。`RenderStep::VideoSource`+`linear_graph_with_video_source`で外部背景をグラフ表現。既存`render_frame_with_background_texture`経路は不変 | R1 | ソースノード込みグラフのゴールデンが通り、既存の外部テクスチャ経路テストも不変 |
| R5 | **完了**: T9完了条件の検証テスト。`motolii-export/tests/t9_validation.rs`で(1)30fps×3秒タイムコード焼き込み素材の全フレーム時刻対応 (2)書き出しmp4のBT.709 limited色タグ(ffprobe生タグ含む)を検証 | なし | 両テストがCIで通る |
| R6 | **完了**: **motolii-testkit拡充**(T4残)。参照PNG保存・差分画像ファイル出力(ゴールデン失敗時のデバッグ運用) | なし | 意図的に壊したゴールデンで差分PNGが出力されるテストが通る |
| R7 | **完了(2026-07-10)**: `OverlayNode`に円(`CircleOverlay`)・線(`LineOverlay`)追加、`CompositeMode`(Add/Multiply)追加。正準座標・premul契約維持。Vello統合はスコープ外 | R1 | 各形状・各ブレンドの複数解像度ゴールデンが通る |
| R8 | **完了(2026-07-10)**: Vello**採用**。vello 0.9=wgpu29一致・実レンダ合格。条件(Renderer長寿命/straight→premul境界変換/usvgアダプタ自前)は[spikes/s3-vello.md](../spikes/s3-vello.md) | なし(独立スパイク) | 採否判断がドキュメント化される(コードは使い捨て可) |
| R9 | **完了(2026-07-10)**: 実素材検証(T11)。`verify-b4`+`scripts/r9-verify.sh`+`spikes/r9-preview`。主観品質OK(人間サインオフ) → [reviews/2026-07-10-R9-real-material-checklist.md](../reviews/2026-07-10-R9-real-material-checklist.md) | R2 | 一致テストが通る+主観品質OK |

並列性: **R1–R9完了(2026-07-10)**。M1残タスクチケット表は消化済み。
**凍結ゲート残件 FG-C1〜C6 全緑 + レビュー3点対応(2026-07-10)**。
**凍結ゲート宣言済み**: [reviews/2026-07-10-freeze-gate-declaration.md](../reviews/2026-07-10-freeze-gate-declaration.md)。次: M2/M4/M5仕様を確定して並列レーン着手。

補足(2026-07-09): CI(ubuntu)は`mesa-vulkan-drivers`導入済みでwgpuゴールデンテストがlavapipeで実行されている(T3で実測diff=0)。T0の「smokeテスト追加」はゴールデンテスト自体が上位互換として満たしているため、独立チケットにしない。

## 目的(退治する落とし穴)

B-3(色空間事故)、B-4(プレビュー/書き出し分岐)、C-3(評価エンジン過小評価)、D-2(自己検証基盤)。
完成物: **1本の動画を読み込み → キーフレーム/合成DataTrackでシェイプを駆動 → 合成 → mp4書き出し** ができるCLIツール。UIなし。キャッシュなし。

### 操作単純化モデルへの割当（完了済み・契約基線）

M1の`render_frame(t, Quality)`、型付き`Value / ParamSource / DataTrack`、plugin純関数、正準座標、preview/export共通評価を、[操作単純化モデル](../interaction-simplicity-model.md)の契約基線とする。M1へUI機能を遡及追加しない。M2以降のDirect/Tool/Advancedはこの基線を迂回する別評価経路を作らず、同じDocument意味へ正規化する。Param Pipeline GateがM1公開契約の変更を要求する場合は、実装前に凍結解除・migration・既存意味論golden維持を行う。

2026-07-08 方針修正: GPU色解析はAE的な基礎操作ではなく、最終的にはプラグイン/ユーザーコミュニティへ委ねる拡張領域である。M1では「解析そのもの」を必須にせず、`motolii-eval`のキーフレーム・DataTrack参照、標準シェイプ制御、Compositeの基礎を優先する。DataTrack境界の検証は、合成テストデータまたはParamDriver参照プラグインで足りる。

## スコープ外

- UI(M3)、Undo/ドキュメント編集(M2)、キャッシュ(M4)、3D/glTF(M5)、音声再生(M2)、動的プラグインロード(v2)、OpenCV(M4)

## Cargo workspace構成(このフェーズで確立し、凍結ゲートの分割単位になる)

```
crates/
  motolii-core     フレーム記述子・有理数時間・エラー型(全クレートの共通語彙)
  motolii-media    ffmpegサイドカー: probe / デコードIterator / エンコードパイプ
  motolii-gpu      wgpu初期化・テクスチャ管理・YUV→RGB変換・シェーダ共通基盤
  motolii-eval     パラメータ評価: キーフレーム補間・イージング・データ列参照
  motolii-nodes    ノードtrait + 標準ノード(オーバーレイ生成・コンポジット)
  motolii-plugin   静的リンク版プラグイン種別レジストリ(Filter/ParamDriver/Composite)
  motolii-testkit  ゴールデンイメージテスト基盤(dev-dependency)
  motolii-render   グラフ実行器: render(t) → フレーム(プレビューと書き出しの共通経路)
  motolii-export   書き出しループ(連番 / ffmpegエンコードパイプ)
  motolii-cli      M1ドライバ(プロジェクト設定ファイル → 書き出し)
```

## M1時点のインターフェース基線

以下はM1で実証・凍結ゲートへ渡した**意味の基線**であり、現在のRust APIをそのまま転記したcode referenceではない。`FrameDesc`の6意味、GPU texture境界、純関数、preview/export共通評価は維持する。一方、plugin traitは後続の正規解凍で`PipelineCache`、`RenderCtx`、型付き`Result`、`LayerSourcePlugin`等を追加済みである。実装時は[plugin作者向け規約](../plugin-authoring.md)と`crates/motolii-plugin/src/lib.rs`を正とし、下の歴史的skeletonをcopyしない。全lineageの処分は[Unit 3C回収](../reviews/2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md)を参照する。

```rust
// M1で固定した共有意味の概略。constructor/serde/error/Rust ABIの永久凍結ではない。
pub struct RationalTime { pub num: i64, pub den: i64 }   // 秒 = num/den
pub struct FrameDesc {
    pub width: u32, pub height: u32, pub stride: u32,
    pub format: PixelFormat,      // wgpu::TextureFormat準拠 + Yuv420p等
    pub color_space: ColorSpace,  // LinearRgb | Srgb | Rec709Limited | ...
    pub premultiplied: bool,
}

// motolii-eval: 評価器は「時刻→値」の純関数。DataTrack参照もキーフレームと同一機構
pub enum ParamSource { Keyframes(KeyframeTrack), DataTrack(DataTrackId), Const(Value) }
pub trait ParamEval { fn eval(&self, t: RationalTime, ctx: &DataTracks) -> Value; }

// motolii-nodes: ノードは状態を持たない。時刻とパラメータ・入力テクスチャから出力を決定。
// エフェクトモデルの決定(concept.md)により、ユーザーに見えるのは「レイヤーに積む
// エフェクトスタック」であり、これはRenderNodeの線形チェーンとして展開される。
// RenderNodeの単純な境界(入力テクスチャ+パラメータ→出力)はそのまま
// エフェクトプラグインの境界になる(LLMでのプラグイン自作を想定した書きやすさ最優先)。
pub trait RenderNode {
    fn describe(&self) -> NodeDesc; // 入出力数・パラメータ定義
    fn render(&self, gpu: &GpuCtx, t: RationalTime,
              params: &ResolvedParams, inputs: &[TextureHandle]) -> TextureHandle;
}

// M1時点のplugin入場skeleton。現在の正確なtrait signatureではない。
// プラグインは種別レジストリへ静的登録する。dylib/配布はこのmilestoneの外。
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

// motolii-render: プレビューも書き出しもこの1関数のみを通る(B-4)
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
| T1 | motolii-core: `RationalTime`(S3の型)・`FrameDesc`・`PixelFormat`/`ColorSpace`・変換ユーティリティ | T0 | 単体テスト(fps変換・丸め・境界値)が通る |
| T2 | motolii-media: probe(解像度/fps/長さ)+ デコードIterator + フレーム正確シーク(S2の採否に従いffmpeg-sidecar利用 or 自前パイプ) | T1 | 実素材とテスト用生成素材で「フレームNを要求→正しいフレームが返る」テストが通る(先頭/末尾/ランダム10点) |
| T3 | motolii-gpu: device初期化・テクスチャアップロード・YUV(Rec.709 limited)→リニアRGB変換シェーダ | T1 | カラーバー素材の変換結果が理論値±1/255で一致する数値テストが通る(B-3) |
| T4 | motolii-testkit: ゴールデンイメージ比較(参照PNG・許容誤差・差分画像出力)+ テスト用素材生成(カラーバー動画等) | T3 | motolii-gpuのYUVテストがtestkit経由に置き換わり、CIで安定して通る |
| T5 | 後続へ移動: motolii-analyze(GPU色解析/トラッキング)。M1では実装しない | - | M4/解析プラグイン側で再定義する |
| T6 | motolii-eval: キーフレームトラック(線形/ベジェ/ホールド)+ イージング + DataTrack参照 + 単体テスト | T1 | 補間の数値テスト、DataTrack参照込みの評価テストが通る(C-3) |
| T7 | motolii-nodes: `RenderNode` trait + SourceNode(動画フレーム)+ OverlayNode(パラメータ駆動の2D図形: 円/矩形/線、色・位置・サイズ)+ CompositeNode(normal/add/multiply、premultiplied正しく) | T3, T6 | 各ノード単体のゴールデンイメージテストが通る |
| T8 | motolii-render: グラフ実行器 `render_frame()`(トポロジカル順に実行、毎回全計算) | T7 | 「ソース→オーバーレイ→合成」グラフのゴールデンイメージテストが通る |
| T9 | motolii-export: `render_frame()`ループ→PNG連番 / ffmpegエンコードパイプ→mp4 | T2, T8 | 30fpsの数秒グラフを書き出し、フレーム数・時刻対応が正確(全フレームにタイムコード焼き込みで検証)。**書き出しmp4の色タグ(bt709/limited)をprobeで検証**(レビュー指摘#5。Encoderのタグ付けは実装・テスト済み) |
| T10 | motolii-cli: プロジェクト設定ファイル(バージョンフィールド付きJSON)を読み、キーフレーム/合成DataTrack評価→シェイプ制御→レンダ→書き出しを一気通貫実行。サンプルプロジェクト同梱 | T6, T9 | サンプルプロジェクトのE2Eゴールデンテスト(先頭/中間/末尾の3フレーム比較)がCIで通る。M1時点ではGPU解析を含めない |
| T11 | 実素材検証: 自分のMV素材で1ショット(数秒)を書き出す。プレビュー(M0-S1採用方式での簡易表示)と書き出しのピクセル一致確認 | T10 | 書き出したショットがMV制作に使える品質(主観)+ プレビュー/書き出し一致テスト(B-4)が通る |
| T12 | motolii-plugin: 静的リンク版の種別レジストリ + 参照プラグイン(Filter / ParamDriver / Composite)。Render系は`wgpu::Texture` in/outのみ | T3, T6 | 種別レジストリの単体テストが通る。参照Filter/CompositeがGPU render passを発行し、T7/T8のゴールデンで実レンダ検証される。CPUフレームを受け取るプラグイン経路が存在しない |

## 並列レーン

```
T0 → T1 →┬ レーンA(media): T2 ───────────────┐
          ├ レーンB(gpu):   T3 → T4 ──────────┤
          ├ レーンC(eval):  T6 ────────────────┼→ T7 → T8 → T9 → T10 → T11
          └(T7はT3,T6完了後に着手可)──────────┘
```

T1完了後、最大3エージェント(A/B/C)並列。T7以降は直列に近いが、T9(export)はT8と部分並列可(モックグラフで先行実装)。

## 出口デモ(M1のゴール。2026-07-09確定)

**背景=実写動画 + 上レイヤー=右へイージング(ベジェ)で流れる四角シェイプ、の2レイヤー合成をmp4出力できること。**

- 具体形: サンプル `project.json`(`version` / `input`=実写動画 / `overlay.center`=`ParamSource::Keyframes`(x座標をベジェイージングで左→右) / `size` / `color`)を `motolii-cli export-project` に渡し、mp4を書き出す。
- 解析駆動は含めない(最終フェーズへ後回し済み)。UI操作は含めない(それはM3)。CLIでのファイル出力までがM1。
- 完了条件(自動判定): このサンプルの**先頭/中間/末尾フレームのE2Eゴールデンテスト**がCI(lavapipe+ffmpeg)で緑。矩形が中間フレームで右側へ、イージングにより等速でない位置に来ることを含む。
- 実装状況: `ProjectV1`は`input`+`overlay`(ParamSource)を受理済み、`motolii-eval`にベジェイージング実装済み、`motolii-export`でmp4書き出し済み。**残り=この2レイヤー構成の正式サンプル同梱+E2Eゴールデン**(T10残タスク)。

## フェーズ完了条件

- 上記**出口デモのE2Eゴールデンが緑**であること
- T0〜T12のうちT5を除く全タスクのCI完了条件が通っている
- 凍結ゲートのレビューを実施: `FrameDesc`・`RenderNode`・`ParamEval`・時間型・workspace分割に加え、**正準座標系(F-1)・並行性/所有権モデル(F-2)・単一評価モデル(F-3)・クリップ時間写像(F-4)**を凍結し、M2/M4/M5仕様書を確定させる(pitfalls-and-roadmap.md「凍結ゲート」の9項目)

注(2026-07-08、F-1): T7のOverlayNode等で空間パラメータ(位置・サイズ・ブラー半径)を実装する際、**絶対pxではなく正準空間(単位なし・原点中央・Y-up・高さ=1.0)の値で受け、px変換はレンダ直前の1箇所に集約**する。Draft(半解像度)とFinalの見た目一致(B-4の約束)はこれが前提。

注(2026-07-08、alpha): 内部レンダターゲット/合成は**premultiplied alpha**を正規形にする。UIやJSON等のユーザー入力色はstraight alphaとして受け、レンダ直前またはComposite境界でpremulへ変換する。T7のOverlay最小版はalpha=1.0のみを実証対象とし、alpha<1の期待値式は`CompositeNode`のnormal overで固定済み: `out.rgb = fg.rgb + bg.rgb * (1 - fg.a)`, `out.a = fg.a + bg.a * (1 - fg.a)`。

## 未決事項

- プロジェクト設定ファイルのスキーマ詳細(T10で最小形を決め、M2で正式化)
- ~~OverlayNodeのベクター描画を自前シェーダにするかVello依存にするか~~ → **採否確定(2026-07-10、R8/S3)**: Vello採用。実測根拠と統合条件は[spikes/s3-vello.md](../spikes/s3-vello.md)。既存の自前シェーダ(矩形等)は当面併存し、複雑パス/SVGをVelloに任せる

## 実装ガード(先行ツールの失敗・ユーザー不満クロスチェック 2026-07-11)

出荷済みツール(Premiere/Resolve/OBS/AviUtl/Shotcut等)の書き出し・デコード領域の実ユーザー苦情と、ffmpegをサブプロセスとして使うOSSプロジェクト(moviepy/editly/Remotion/lossless-cut)の障害履歴を調査し、既存カタログ(B-2/B-3/B-4)に無いガードを抽出した。M1出口デモは完了済みのため、これらは**motolii-media/motolii-exportの増強チケット候補**(M2以降の並列レーンで消化する。各項目は独立に1PR化できる)。

2026-07-23 live再照合: G1、G2、G3、G4、G8は未実装、G5/G6は部分到達、G7は同期1-frame経路で実質bounded。詳細と誤った旧`[x]`の処分は[Unit 5A歴史回収](../reviews/2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md)。これはM1 exit demoを撤回せず、出荷hardeningをGAP-26／27へ接続する現在地である。

| # | ガード | 先行事例 | 完了条件(自動判定) |
|---|---|---|---|
| G1 | **stderrは必ず専用スレッドで常時ドレイン**し、リングバッファに保持(エラー時の診断用)。パイプを開いたのに読み手がいない状態を作らない | ffmpeg-pythonの古典的デッドロック: stderr未読でOSパイプバッファ(約64KB)が警告出力で埋まりffmpegがブロック→stdin書き込み側も停止(kkroening/ffmpeg-python#195) | 大量警告を吐く壊れかけ素材を長時間パイプ処理してもハングしないストレステスト |
| G2 | **成否判定は終了コードでなく出力検証**: 書き出し完了時にffprobeでフレーム数・duration・ストリーム構成の一致を確認して初めて成功とする | 「exit 0でも期待通り変換されていない」(FFAStrans)、逆にレンダ成功後のクリーンアップでsegfaultし成功を失敗と誤報(mltframework/mlt#1175)。アンチウイルスによるサイドカーブロックの誤診も定番 | export APIが検証済み結果を返し、検証不一致がErrになるテスト |
| G3 | **書き出しは一時ファイル→検証→アトミックrename**。中断で部分ファイルを残さない。mp4はmoov atomが最後に書かれるため中断=全損になることを前提に設計(`+faststart`等の後処理はrename前) | OBS最大級の苦情源: クラッシュ/killでmp4全損 → OBS 30.2がfragmented MP4ベースのHybrid MP4を導入した経緯 | エンコード途中でサイドカーをSIGKILLし、出力先に不完全ファイルが存在しないことをアサート |
| G4 | **Dropでkill→wait(タイムアウト付き)→パイプclose**を保証(明示closeとの二重防御。AGENTS.mdの`Encoder::finish()`規約の構造化) | moviepyのFDリーク: 数百クリップ処理で`[Errno 24] Too many open files`(Zulko/moviepy#660) | デコードセッション1000回開閉後にFD数増加ゼロ・ゾンビプロセスゼロをアサート |
| G5 | **起動時にffmpegバージョンと必要フィルタ/muxerをプローブ**し、最低バージョン未満は実行前エラー(実行中エラーにしない) | editly: ディストリ任せのffmpeg 4.1系でフィルタオプション欠如により実行時失敗(mifi/editly#34) | 古いバージョンを名乗るモックで起動時に構造化エラーになるテスト |
| G6 | **シェル経由のffmpeg起動を恒久禁止**(argv配列のみ。フィルタ文字列内へのパス埋め込みも禁止)。日本語・絵文字・空白・記号入りパスのE2EをCIに置く | ffmpeg-python/madmom等で非ASCIIファイル名の失敗が反復(コードページ/クォート問題)。日本語ユーザーが主対象の本ツールでは初日に踏む | `日本語 🎬 &記号`入りパスのデコード→エンコードE2Eが全対象OSで通る |
| G7 | **in-flightフレーム数をbounded channelで制限**し、「エンコーダが遅い」を正常系として設計(バックプレッシャ) | Premiere/Resolveの書き出し失敗最大カテゴリ「GPU Render Error / GPU memory is full」は、デコード→合成→エンコード間のバックプレッシャ欠如でVRAMが溢れるのが一因(render speedを下げると直る、という回避策がそれを示す) | 人工的に遅いエンコーダを繋いでもVRAM使用が上限内に収まる計測テスト |
| G8 | **lavapipeのバージョンをピン留めし、更新をゴールデン再生成のトリガーとして扱う**。全GPUテストにper-testタイムアウト(ハング=失敗として検出) | pygfx実証済み戦略「バックエンド間でピクセルは一致しない。参照はCIと同じlavapipe環境で生成」。lavapipeでのランナーハングは既知(gfx-rs/wgpu#1974) | CIイメージのlavapipe版が固定され、GPUテストにタイムアウトが設定されている |

補足(色): 出荷ツールの「書き出すと色が違う」苦情の過半は、変換数式ではなく**コンテナメタデータのタグ付けと表示側解釈**で起きている(ResolveのNCLC 1-2-1タグがYouTube/QuickTimeで低コントラスト化、OBSのfull range/BT.601誤タグ、macOS ColorSyncのガンマ解釈差)。T2/T9のBT.709タグ明示+probe検証は正解であり、この規律(**無タグ出力を許さない・rangeも常に明示・既定はlimited+BT.709**)を今後のエンコードプリセット追加でも維持する。正しくタグ付けしたファイルでもmacOSでは表示解釈差が残ることは既知事項として文書化する(自分のバグと誤診しない)。

出典: kkroening/ffmpeg-python#195 / Zulko/moviepy#660 / mifi/editly#34 / mltframework/mlt#1175 / obsproject.com(Hybrid MP4) / gfx-rs/wgpu#1974 / helpx.adobe.com「washed out exports」/ filmmakingelements.com(Resolveガンマシフト) / aviutl.info(BT.601/709自動判定の混乱)
