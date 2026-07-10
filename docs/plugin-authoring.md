# プラグイン作者向け規約(LLM / 人間共通)

作成日: 2026-07-10

並列エージェントやLLMがプラグインを量産するときの**唯一の契約書**。  
実装の型紙は `crates/motolii-plugin` の参照プラグイン(`reference`モジュール)。  
設計根拠は [concept.md](concept.md)・落とし穴F-8/F-9([pitfalls-and-roadmap.md](pitfalls-and-roadmap.md))。

> v1は静的リンクのみ。dylib/WASM配布はv2。この文書は「書ける境界」を先に固定するためのもの。

## 1. 種別を選ぶ(混ぜない)

| 種別 | trait | 入出力 | 用途 |
|---|---|---|---|
| Filter | `FilterPlugin` | テクスチャ1→1 | グロー・歪み・色補正など |
| Composite | `CompositePlugin` | テクスチャN→1 | ブレンド・レイヤー合成 |
| LayerSource | `LayerSourcePlugin` | 入力0→テクスチャ1 | 図形生成・点群投影など |
| ParamDriver | `ParamDriverPlugin` | 構造化データ→`DataTrack` | LFO・解析結果の値列 |

- テクスチャを触るものはRender系(Filter/Composite/LayerSource)。値だけならParamDriver。
- 「何でもFilterに詰める」は禁止(G-2)。迷ったら種別を増やすのではなく、既存種別に収まるか見直す。

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
3. **隠れた可変状態を持たない** — 出力は時刻`t`と入力(テクスチャ/params/ctx)だけで決まる。`&self`にキャッシュや乱数シードを溜めない(純関数契約)
4. **空間パラメータに絶対pxを書かない** — 正準空間(原点中央・Y-up・高さ=1.0)。px変換はホスト/レンダ直前
5. **色変換をプラグイン内で勝手にやらない** — 色変換はレンダ直前の1箇所のみ(OCIO-shaped)
6. **ループ内でGPUリソースを毎フレーム新規生成しない** — パイプライン/バッファは初期化時に作り再利用
7. **公開APIで`assert!`/panicしない** — 入力起因は`PluginError`(または型付きResult)

## 4. 推奨すること

- **意図単位の1プラグイン** — 「グロー」「シェイク」のように完成した意図。原子プリミティブの組み立てをユーザーに強いない(F-8)
- **参照実装を型紙にする** — `ClearFilter` / `SineParamDriver` / `ClearComposite` をコピーしてから肉付けする
- **パラメータは少ない** — LLM生成でも人間が触れる数に抑える。内部定数はコード側へ
- **premultiplied alphaを前提にする** — Compositeは既存のnormal over式に合わせる。straight alphaを勝手に混ぜない
- **テスト** — Render系は`motolii-testkit`ゴールデン。ParamDriverは値列の単体テスト

## 5. 最小スケルトン(Filter)

```rust
use motolii_plugin::{FilterPlugin, NodeDesc, PluginError, PluginId, ResolvedParams, TextureRef, ValueType};
use motolii_core::RationalTime;
use motolii_gpu::GpuCtx;

pub struct MyGlow;

impl FilterPlugin for MyGlow {
    fn desc(&self) -> &NodeDesc {
        // version/category/tags を必ず埋める
        todo!("static NodeDesc")
    }

    fn render(
        &self,
        gpu: &GpuCtx,
        encoder: &mut wgpu::CommandEncoder,
        t: RationalTime,
        params: &ResolvedParams,
        input: TextureRef<'_>,
        output: TextureRef<'_>,
    ) -> Result<(), PluginError> {
        // wgpu/WGSLのみ。CUDA等は書かない。
        // 出力は t + params + input だけで決める。
        let _ = (gpu, encoder, t, params, input, output);
        Err(PluginError::Render("unimplemented".into()))
    }
}
```

ParamDriverは`build_track`で`DataTrack`を返すだけ。ピクセルに触らない。

## 6. 解析・AI系を足したくなったとき

- コンセプトの本線は「色解析・単純トラッキング → DataTrack → パラメータ駆動」。YOLO級は必須ではない
- 将来入れるなら**ホスト側にクロスプラットフォーム実行器**(例: ONNX)を置き、プラグインは結果をDataTrack化するだけ
- プラグイン契約を緩めてCUDAを露出するのは禁止(F-9)。詳細は[plugin-authoring]この文書§3と落とし穴F-9

## 7. レビューチェックリスト(エージェント提出前)

- [ ] 種別が1つに決まっている
- [ ] `NodeDesc`に id / version / display_name / category / tags / params がある
- [ ] wgpu/WGSL以外のGPU APIが無い
- [ ] 製品経路にCPUフレームが無い
- [ ] `&self`にフレーム間状態が無い
- [ ] 空間値が正準座標
- [ ] 参照実装またはゴールデン/単体テストがある
- [ ] 表示名が「意図単位」になっている

## 8. まだ凍結していない口(触らない / 予約のみ)

並列実装で勝手に広げない。凍結ゲート待ち:

- 動的ロード(dylib)・WASM配布(v2)
- 評価コンテキストのインスタンスインデックス`(i, count)`(F-7、口の予約のみ)
- サムネイル画像フィールド(F-8、口だけ将来)
- ハンドルID化(A-3: 現状は`&wgpu::Texture`直渡し。内部更新閉じ込めは後続)
- **ホスト所有PipelineCache / GpuAssetCache**(F-10、[plugin-resources.md](plugin-resources.md))。決まるまで「パイプラインを持つプラグイン」は書けない(§3-3と§3-6が両立しないため)。凍結ゲート待ち
- **Importer種別・`ValueType::AssetRef`**(F-10)。ファイルを食うプラグインはこの口の凍結後
- **時間参照 `CompLookbehind`**(F-11、[plugin-resources.md](plugin-resources.md)§6)。「合体結果の前フレーム」(残像・フィードバック)はホストが渡す口の凍結待ち。**前フレームを`&self`に覚えて自作するのは§3-3違反で恒久却下** — 現時点で書けるのは現在フレームのみの空間グリッチまで
