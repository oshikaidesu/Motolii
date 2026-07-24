# プラグイン作者向け規約(LLM / 人間共通)

作成日: 2026-07-10

並列エージェントやLLMがプラグインを量産するときの**唯一の契約書**。  
実装の型紙は `crates/motolii-plugin` の参照プラグイン(`reference`モジュール)。  
設計根拠は [concept.md](concept.md)・落とし穴F-8/F-9([pitfalls-and-roadmap.md](pitfalls-and-roadmap.md))。

> 現行製品は同一binaryへ静的に組み立てるfirst-party pluginだけを実行する。Vismのsource／WGSL／WASM／native payload、動的load、install、sandboxは比較前であり、本書は「現行公開façadeで書ける境界」を固定する。

ここで固定するのはplugin ID、kind、parameter、入出力、失敗等の**意味**であり、現行`PluginId(&'static str)`や`&'static dyn Plugin`という所有／lifetime形を将来loaderのABIとして凍結するものではない。M1監査以来この区別は維持されており、dynamic load、source配布、WASM/native境界へ進む時は所有形、ABI、version negotiation、権限を独立contractで再締結する。現行のstatic型を、そのまま配布仕様や安全境界へ外挿しない。

> **信頼境界:** first-partyはprovenanceと参照実装を表すだけで、実行上の特権や信頼を表さない。公開plugin境界を通るcodeは供給元を問わず非信頼とし、将来runtimeでは同じcapability制限、resource budget、failure containmentへ通す。Controlled Microkernelと製品buildへ明示的にadmitされたHost capability moduleだけがTCBであり、「core plugin」という例外分類は作らない。現行static first-partyは同一processなのでクラッシュ隔離をまだ満たさない。

## 0. この境界を作る理由

Motoliiの長期の北極星は、映像表現を特定projectの手順から切り離し、実行・再利用・保存・配布できる単位にすることにある。「映像制作におけるVST」はHostと拡張単位を分ける構造の類比に限り、このplugin境界は単なる内製effect追加口ではない。Host全体をforkせず、ひとつの表現に集中できる作者面を作る。

plugin作者は通常の制作者と別の身分ではない。既存表現を使い、調整し、接続し、inspectionやforkを経て新しい表現を共有するまでを同じ学習曲線に置く。first-party pluginは第三者が到達できない完成品ではなく、この規約、参照実装、scaffold、testkitだけで到達可能性を証明する手本とする([Creator / Developer連続体](reviews/2026-07-22-creator-developer-continuum-decision.md))。

長期的なユーザー向け配布単位は[Vism](vism-package-concept.md)である。Vismは一つの持ち運べる映像表現、`Filter`等はHost内部の実行分類であり、同義ではない。Vismは別VismのIDを直接要求せず型付きinputを宣言し、具体provider・接続・初期値は[Kit](vism-kit-model.md)が束ねる。v1のpluginは静的リンクされたpre-Vism参照実装として、将来のpackage境界を公開契約だけで反証する。`.vism` loader、Kit schema、package manifestを本書から先行実装しない。

VSM-A0I-1〜3でContract Catalog、Documentのprepared resolution、graph／exportのruntime必須化までは実装済みである。VSM-A1-3で `core.filter.opacity` を `plugins/motolii-plugin-opacity` へ、VSM-A2で `core.param.sine` を `plugins/motolii-plugin-sine` へ、VSM-A3で `core.layer_source.radial_repeater` を `plugins/motolii-plugin-radial-repeater` へ外部化した。first-party組み立てと依存allowlist検査の実証が完了している。

> **現在の停止線**: plugin crateは`motolii-plugin`だけへ依存し、GPU golden／purity／parityはHost側の審判から検査する。private依存の例外やtestkitへのdev依存を追加しない。

### 0.1 現在できるauthoringと、まだ無いdistributionを分ける

| 段階 | 現在地 | 証拠／停止線 |
|---|---|---|
| `motolii-plugin`内の参照実装 | 実装済み | `new-plugin.sh`はこのin-tree形を生成する |
| bundled first-party workspace crate | 実装済み | Opacity / Sine / Radial Repeater、公開façade、依存allowlist、Host composition root |
| 同形crateを作る作者scaffold | **未実装・再入場可** | A1で「A2の二例後」へ延期され、A2/A3は完了した。VSM-A4でin-tree scaffoldと分けて閉じる |
| third-party install / load / update | 未実装 | `.vism` package、resolver、trust、権限、rollbackが未決 |
| WASM / native payload runtime | 予約・比較前 | `PluginKind::ScriptWasm`はenum予約だけ。native Rust first-party crateの静的組み立てと、OS別binaryの動的loadを同義にしない |

first-party無特権は「第三者配布が完成した」または「現行static実行が安全境界を完成した」という意味ではない。現行成果は、同じ公開Rust façadeで表現計算を書けることの証明までである。作者が配布・導入できる成果物へ到達するには、authoring scaffoldとVism distribution/trustの二つの未実装境界が残る。

そのため、pluginは一枚の絵を出せれば完成ではない。次の全条件を満たして初めて、作品に置ける表現単位になる。

- Hostが所有する時刻、型付きparameter、入力、seed、Qualityだけから再現できる
- 安定IDとversionを持ち、Documentにはrecipeだけが保存される
- PreviewとExportで同じ実装を使い、OS・GPU vendor・解像度に意味を依存させない
- Hostのcache、resource lifecycle、error、欠落診断、UI discoveryへ参加し、独自の裏口を作らない
- 人間とLLMのどちらも、参照実装と機械判定可能なtestから適合性を確認できる

詳しい設計審判とv1の非目標は[concept.md「長期の北極星」](concept.md#長期の北極星-映像表現を実行再利用配布できる単位にする)を正本とする。以下の規約は、その北極星を現在の実装で守るための具体化である。

## 1. 種別を選ぶ(混ぜない)

| 種別 | trait | 入出力 | 用途 |
|---|---|---|---|
| Filter | `FilterPlugin` | テクスチャ1→1 | グロー・歪み・色補正など |
| Composite | `CompositePlugin` | テクスチャN→1 | ブレンド・レイヤー合成 |
| LayerSource | `LayerSourcePlugin` | 入力0→テクスチャ1 | 図形生成・点群投影など |
| ParamDriver | `ParamDriverPlugin` | params＋区間＋sample rate→`DataTrack` | LFO・決定的な値列生成 |

- テクスチャを触るものはRender系(Filter/Composite/LayerSource)。値だけならParamDriver。
- 「何でもFilterに詰める」は禁止(G-2)。迷ったら種別を増やすのではなく、既存種別に収まるか見直す。
- 他のplugin／VismをIDや表示名で検索しない。必要な値、event、texture、assetは型付きinputとして宣言し、具体providerの選択と接続はKit／Hostへ残す。
- 予約種別(口のみ): `Simulation`(逐次状態シミュレーション。[simulation-model.md](simulation-model.md))、`ScriptWasm`。どちらもtrait／登録／製品実行能力として数えない。

### 1.1 位置や大きさを編集する拡張は現行traitへ偽装しない

本節の4種は現在の評価plugin境界である。位置、Scale、key、接続等を編集する将来拡張は、次の責任寿命で別に審判する。

- Commit後にpluginが不要な一回限りの編集: **Authoring Tool**候補。read-only snapshotからtyped command batchを提案し、Hostがpreflightして1 macro commitする。
- 時刻や入力変更後も続く関係: **Behavior / Driver**候補。入力、出力、scope、評価順、時間依存、削除/欠落時挙動を宣言する。
- 独自recipeやnode graphが正本: **Generator / Structured Recipe**候補。recipe、version、依存、Materialize/Live/Bakeを宣言する。

これは分類の決定であって、新しいtrait/APIの実装許可ではない。`FilterPlugin`や`ParamDriverPlugin`へ自由な`&mut Document`、layer名検索、UI callback、独自Undo、隠れcontrollerを足して代用しない。公開境界の解凍条件、capability分割、Hostへ昇格する審判は[小さなコアと探索可能な拡張](extensible-core-model.md)を正本とする。

## 1.5. UIは書かない(v1)

**現在のプラグイン公開契約にカスタムUIはまだ無い。** `NodeDesc.params`からHost標準parameter panelを生成することはM3 U4aの必須fallbackとして決定済みだが、製品接続は未実装である。plugin所有egui/native/Web code・wgpu描画panel・独自widgetを書いてもホストはロードしない。標準製品surfaceのnative／React選定はplugin分類と独立であり、custom UIのruntime・sandbox・互換・配布は[軸分離決定](reviews/2026-07-22-m3-surface-extension-axis-separation.md)に従ってG0-3 / GAP-13で比較する。本書からAPIを推測しない。

- パラメータは`ParamDef`で足りる粒度に抑える(スライダー/カラー等の自動生成で操作可能であること)
- 将来カスタムUIが解凍されても、**自動生成パネルだけで全パラメータを操作できること**が不変条件
- 表現の調整、値source、automation、接続、診断をParameter Panelへ集約するUI力学は[UI操作言語 §5.4](ui-interaction-language.md#54-parameter-panelを表現のホームにする)を正本とする。plugin固有panelや文字列expressionを意味の唯一の家にしない

## 2. 必須メタデータ(`NodeDesc`)

すべてのプラグインは `desc()` で次を返す。欠けたらレビュー却下。

| フィールド | 規則 |
|---|---|
| `id` | 安定文字列。`vendor.kind.name` 形式(例: `core.filter.clear`)。一度公開したらリネームしない |
| `version` | パラメータスキーマの互換バージョン。破壊的変更で+1 |
| `display_name` | UI表示名。ユーザーの意図が分かる語(「Clear」「Sine」) |
| `category` | ブラウザ用カテゴリ。例: `Color` / `Distort` / `Generate` / `Composite` / `Utility` |
| `tags` | 検索用。小文字・短い語。将来サムネイル口とは別 |
| `params` | `ParamDef`列。idは安定、default必須 |
| `min_inputs` / `max_inputs` | 種別の契約に合わせる |

未知の`id`を含むプロジェクトは**ロード失敗にしない**(警告+パススルー)。ホスト側の責務だが、作者はid/versionを安易に変えないことで可搬性を守る(F-9)。

## 3. 絶対禁止(破ると設計根拠が崩れる)

1. **ベンダー/OS固有APIを契約・実装に出さない** — CUDA / Metal / D3D / TensorRT 等。見せるGPUはwgpu + WGSLのみ(F-9)
2. **CPUフレームを製品経路で受け渡さない** — 入出力は`TextureRef`(wgpuテクスチャ+`FrameDesc`)。CPU参照はゴールデンテスト専用
3. **隠れた可変状態を持たない** — 出力は時刻`t`と入力(テクスチャ/params/ctx)だけで決まる。`&self`にキャッシュや乱数シードを溜めない(純関数契約)。**フレームN-1依存の物理ステッパ・積分型シミュレーションをレンダ系traitに入れるのも禁止**(2D/3D問わず)。これは物理・時間表現そのものの禁止ではない — 第一選択はf(t)の安い力([concept.md](concept.md)「馬鹿正直にシミュレートしない」)、それで書けない表現には§4.5の正規ルート(レンダ外のベイク境界)がある。禁止しているのは「ホストに宣言しない状態」だけ(キャッシュ/並列/シークでサイレントに壊れるため)
4. **空間パラメータに絶対pxを書かない** — 正準空間(原点中央・Y-up・高さ=1.0)。px変換はホスト/レンダ直前
5. **色変換をプラグイン内で勝手にやらない** — 色変換はレンダ直前の1箇所のみ(OCIO-shaped)
6. **ループ内でGPUリソースを毎フレーム新規生成しない** — パイプライン/バッファは初期化時に作り再利用
7. **公開APIで`assert!`/panicしない** — 入力起因は`PluginError`(または型付きResult)

### 3.1 `TextureRef`と`FrameDesc`を分離しない

`TextureRef`は借用`wgpu::Texture`と`FrameDesc`の対である。`FrameDesc`のwidth/height/stride/format/color space/premultipliedを入力の意味として読み、texture label、Host内部ID、cacheの状態から推測しない。`FrameDesc`はGPU handle、Document画像schema、Vism wire formatではない。

M1仕様にあるplugin traitは歴史的skeletonで、現行signatureではない。作者は本書§5と`motolii-plugin`公開面を使う。`FrameDesc`の意味凍結と、constructor/serde/error面の現行gapは[共有型lineage回収](reviews/2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md)を参照する。gapを回避する独自descriptor、文字列error、黙ったclampをplugin側へ作らない。

## 4. 推奨すること

- **意図単位の1プラグイン** — 「グロー」「シェイク」のように完成した意図。原子プリミティブの組み立てをユーザーに強いない(F-8)
- **参照実装を型紙にする** — `ClearFilter` / `SineParamDriver` / `ClearComposite` をコピーしてから肉付けする
- **パラメータは少ない** — LLM生成でも人間が触れる数に抑える。内部定数はコード側へ
- **paramsは型付きアクセサで読む** — `require_f64` / `require_color` / `require_vec2`。`f64_or`のサイレントフォールバックは禁止(「もっともらしく間違う絵」。M2E-8)。ロード側は`NodeDesc::resolve_params`を使い、手書きのdefault充填を複製しない
- **premultiplied alphaを前提にする** — Compositeは既存のnormal over式に合わせる。straight alphaを勝手に混ぜない
- **テスト** — Render系は`motolii-testkit`ゴールデン。ParamDriverは値列の単体テスト

## 5. 最小スケルトン(Filter)

`motolii-plugin`内へ参照実装を追加する時は、手書きコピーより生成から始める(INF-7e):

```bash
./scripts/new-plugin.sh filter glow \
  --out /tmp/glow.rs \
  --out-test /tmp/glow_test.rs \
  --plugin-import motolii_plugin::reference
# kind: filter | layer_source | param_driver | composite
# --out のみでも `{stem}_test.rs` を同ディレクトリに書く
```

このscriptは**in-tree参照実装専用**であり、workspace plugin crate、Cargo manifest、first-party composition登録、Vism packageを生成しない。外部作者向けscaffold完成の証拠にしない。生成物は**2成果物**(M2E-10 / INF-7e。plugin↔testkit 循環回避):

1. **製品コード**(`--out`) → `motolii-plugin` **クレート内**に貼る。`use crate::{...}` + `validate_node_desc` + **ParamDef 例**。`motolii_testkit` / `motolii_plugin::` は参照しない
2. **testkit テスト**(`--out-test`) → `motolii-testkit/tests/` に置く。**purity**(`assert_*_pure` + `gpu_or_skip`) + **ゴールデン**(RGBA は `assert_rgba_close`、ParamDriver は値列)。期待オラクル未設定時は fail-closed

`--plugin-import` でテスト側の `use` 先を登録モジュールに合わせる。以下は手書き時の型紙(クレート外の例。クレート内では `use crate::...`)。

```rust
use motolii_plugin::{FilterPlugin, NodeDesc, PluginError, PluginId, RenderCtx, ResolvedParams, TextureRef, ValueType};
use motolii_gpu::{GpuCtx, PipelineCache};

pub struct MyGlow;

impl FilterPlugin for MyGlow {
    fn desc(&self) -> &NodeDesc {
        // version/category/tags を必ず埋める
        todo!("static NodeDesc")
    }

    fn render(
        &self,
        gpu: &GpuCtx,
        pipelines: &mut PipelineCache,
        encoder: &mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        // wgpu/WGSLのみ。CUDA等は書かない。
        // パイプラインは pipelines.get_or_create_* でホストから借りる(所有しない)。
        // 出力は ctx.t + params + input だけで決める。Draft/Final は ctx.quality。
        let _ = (gpu, pipelines, encoder, ctx, params, input, output);
        Err(PluginError::Render("unimplemented".into()))
    }
}
```

ParamDriverは`build_track`で`DataTrack`を返すだけ。ピクセルに触らない。

## 4.5 物理・前後フレームが欲しいとき(時間軸自由度のはしご)

「バネで揺らしたい」「残像が欲しい」「パーティクルを降らせたい」は正当な要求で、**正規ルートがある**。禁止(§3-3)に触れずに、欲しい表現に対して**最も安いレベル**を選ぶ。全体設計は[simulation-model.md](simulation-model.md)。

| レベル | 手段 | 例 | 状態 |
|---|---|---|---|
| 0 | **tの閉形式純関数**。減衰振動・イージング・シード+tの手続き生成は数式で書ける | バネ/バウンス、ウィグル、決定論パーティクル(tから位置を直接計算) | 今すぐ可能 |
| 1 | **`build_track`内の逐次計算**。ParamDriverは区間を一括生成する契約なので、**内部で**フレーム順に積分してよい(外から見れば決定論的な純関数)。結果はDataTrackとしてレンダ側が読む | バネ質点の軌道、群れの重心、簡易物理の位置列 | 今すぐ可能 |
| 2 | **宣言的時間窓**(`NodeDesc`に前後フレーム/サブフレームサンプル数を静的宣言) | エコー/残像、フレームブレンド、モーションブラー | 口の予約待ち(凍結ゲート) |
| 3 | **`SimulationPlugin`**: `init`/`step`/`snapshot`だけ書く。状態の保存・キャッシュ・スクラブ・無効化は全部ホストの仕事 | 布、液体、本物のパーティクル | 口の予約待ち、実装v1.x |

- 迷ったら下のレベルから検討する。レベル0/1で書ける表現をレベル3にしない(逆も同様 — レベル3が必要なものをFilterにハックで押し込まない)
- パーティクルは標準搭載(ファーストパーティ第2号)がL0+L3の型紙になる予定([simulation-model.md](simulation-model.md)§8)。汎用パーティクルを自作する前に、標準のパラメータ拡張で足りないか確認する
- **`Filter`の`&self`に状態を隠すのは、レベルに関係なく恒久禁止**。シーケンシャル再生では動いて見えるが、キャッシュON・スクラブ・並列書き出しで壊れる(壊れ方の一覧はsimulation-model.md§4)

## 6. 解析・AI系を足したくなったとき

- コンセプトの本線は「色解析・単純トラッキング → DataTrack → パラメータ駆動」。YOLO級は必須ではない
- 将来入れるなら**ホスト側にクロスプラットフォーム実行器**(例: ONNX)を置き、プラグインは結果をDataTrack化するだけ
- プラグイン契約を緩めてCUDAを露出するのは禁止(F-9)。詳細は[plugin-authoring]この文書§3と落とし穴F-9

## 7. レビューチェックリスト(エージェント提出前)

- [ ] 種別が1つに決まっている
- [ ] `NodeDesc`に id / version / display_name / category / tags / params がある
- [ ] paramsは`require_*`で読み、`f64_or`を使っていない
- [ ] wgpu/WGSL以外のGPU APIが無い
- [ ] 製品経路にCPUフレームが無い
- [ ] `&self`にフレーム間状態が無い
- [ ] 空間値が正準座標
- [ ] 参照実装またはゴールデン/単体テストがある
- [ ] **純関数**: 同じ`t`+入力で2回呼んでも同一出力(`motolii_testkit::purity::assert_filter_pure` / `assert_param_driver_pure`)
- [ ] 表示名が「意図単位」になっている
- [ ] カスタムUI(plugin所有egui/native code / wgpu panel / 独自widget)を製品コードに含めていない

## 8. まだ凍結していない口(触らない / 予約のみ)

並列実装で勝手に広げない。凍結ゲート待ち:

> 口を広げる提案は**キャッシュキーへの寄与定義とセット**で出すこと(M4「キャッシュキーの完全性原則」)。キャッシュ自体はホストの専権事項で、プラグインからのキャッシュヒント・自前キャッシュは受けない(§3-3)。コスト優先度はホストが実測する。

- Vismのsource／WGSL／WASM／native payload、動的load、install、sandbox。`PluginKind::ScriptWasm`は予約だけで、Rust dylibやWASMを既定方式にしない
- bundled first-partyと同じ公開façade・依存allowlistを最初から満たす**外部crate作者scaffold**。現行`new-plugin.sh`をそのまま拡張せず、in-tree生成とVSM-A4のcrate/conformance生成を別モードまたは別toolとして審判する
- 評価コンテキストのinstance情報(F-7) — 現行予約`InstanceIndex(i,count)`をそのまま恒久化せず、M5-P0I/P7で`stable InstanceId + index/count + nested depth + position`へ解凍する。乱数identityにindexを使わず、`user_seed + InstanceId + channel`のPCG32だけを許可。P7前は`RenderCtx::instance`を拡張しない
- サムネイル画像フィールド(F-8、口だけ将来)
- ハンドルID化(A-3: 現状は`&wgpu::Texture`直渡し。内部更新閉じ込めは後続)
- **ホスト所有PipelineCache**(F-10実証済み)。`ValueType::AssetRef`とDocument `AssetId`結線は実装済み。**GpuAssetCache／Importerは未実装・未凍結**で、M4/Vism/実素材fixtureを通すまで作者APIとして使えない([歴史回収](reviews/2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md))
- **時間参照 `CompLookbehind`**(F-11) — 型を予約([plugin-resources.md](plugin-resources.md)§6)。**配線口は`RenderCtx::lookbehind`(M2E-7)**。実装はM4後。**前フレームを`&self`に覚えて自作するのは§3-3違反で恒久却下** — 現時点で書けるのは現在フレームのみの空間グリッチまで
- **テキスト組版**(F-6) — コアは `itemize` / `shape(軸・クラスタ対応表)` / `draw` のみ([M5-P6](specs/M5-3d-and-post.md))。一発`draw_text`やシェーピング自作は禁止。縦書き・ルビ・行組・歌詞タイミングはプラグインの領分
- `NodeDesc`の時間フットプリント宣言(前後フレーム/サブフレームサンプル。F-12) — 型`TemporalFootprint`と**`RenderCtx::temporal_footprint`口はM2E-7で予約**。窓テクスチャ解決はホスト側(未配線)
- `SimulationPlugin` trait+StateTrack(F-12。`PluginKind::Simulation`はenum予約済み。traitシグネチャは[simulation-model.md](simulation-model.md)§3.2の叩き台を解凍手続きで確定)
- **Backdrop input** — 画像処理はFilter/Composite pluginでよいが、timeline走査・「下のlayer」推論は禁止。Hostが評価地点の合成済みtextureを型付き入力として渡す口は[2026-07-15決定](reviews/2026-07-15-relative-scope-duplicator-decision.md)後も未凍結であり、scope/migration/cache key/循環拒否を同時に解凍するまで追加しない
- **カスタムプラグインUI** — G0-3 / GAP-13の決定までplugin所有egui/native/Web code / wgpu自由描画を公開しない。G0-9の製品surface合格だけでは解除しない。Host所有の宣言レイアウト / gizmoも型ごとの解凍判断。将来契約は[UI runtime再選定](reviews/2026-07-21-m3-react-webview-runtime-reconsideration.md)の証拠を入力に別途仕様化する
