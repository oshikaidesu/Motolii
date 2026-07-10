# 設計レビュー所見 2026-07-10 (M1完了後・プラグイン境界の凍結前監査)

対象: M1残チケットR1–R9消化時点のワークツリー。観点は「あとからプラグインを足しても大丈夫か(最大ケース: 点群インポートをプラグインで)」。
これは凍結ゲートの**入場前チェックリスト**である。全項目が解消(またはスコープ外と明示判断)されるまでM2以降の並列レーンを開かない。
修正したらチェックを入れ、修正コミットにこのファイルの項番を書くこと。

## 重大(紙の凍結=G-1違反になりかねないもの)

- [x] **1. グラフ実行器がプラグインを一度も呼ばない** — `crates/motolii-render/src/lib.rs`
  `RenderStep::Plugin { id, params, inputs, output }` を追加。実行器が `PluginRegistry` 経由で Filter/Composite/LayerSource をディスパッチ。ゴールデン `plugin_filter_dispatches_via_registry_golden` で参照 ClearFilter がレジストリ経由で通ることを確認。所見5(by-name全種別)も同時対応。
  修正方針: `RenderStep::Plugin { id, params, inputs, output }`相当の一般ステップを追加し、参照Filterがレジストリ経由でグラフ内から呼ばれるゴールデンを1本通す。[plugin-resources.md](../plugin-resources.md)§5-1。

- [x] **2. 純関数契約とリソース生成禁止の両立機構がない** — `motolii-gpu::PipelineCache` + `TintFilter`(`core.filter.tint`)
  `RenderSession`がホスト所有キャッシュを持ち、Filter render に`&mut PipelineCache`を渡す。ゴールデン `tint_filter_uses_pipeline_cache_without_recompile` で miss=1→hit=1 を確認。`ValueType::AssetRef`/`Value::AssetRef`も予約。
  修正方針: ホスト所有PipelineCache(+将来GpuAssetCache)。決定案は[plugin-resources.md](../plugin-resources.md)§3。

- [x] **3. アセット語彙がない(点群インポートの前提欠落)** — `ValueType::AssetRef` / `Value::AssetRef(String)` を予約(所見2と同時)。Importer/GpuAssetCache結線はM2。
  修正方針: `ValueType::AssetRef`の予約+M2 D1のAsset一般化(opaque blob+type文字列)+Importer種別の契約定義。[plugin-resources.md](../plugin-resources.md)§3 D2/D3。

## 中(凍結の書き方で防げるもの)

- [x] **4. `&'static`がv1静的リンク前提を型に焼き込んでいる** — 凍結ゲート項目16に「凍結対象はセマンティクスであってlifetimeではない」を明記済み(docsコミット)。型のString化はしない。
  修正方針: 今すぐString化はしない(静的リンクv1ではこの形が軽い)。凍結ゲート文書に「凍結対象はid/version/paramの**セマンティクス**であり、lifetime/所有形態(&'static→Cow/String化)は互換変更として許す」と明記する。

- [x] **5. by-name参照がParamDriverにしかない** — `crates/motolii-plugin/src/lib.rs`
  `filter_by_name` / `composite_by_name` / `layer_source_by_name` / `param_driver_by_name` を追加(所見1と同時)。
  修正方針: 所見1の一般ステップ実装と同時に全種別のby-name(またはキーをStringに統一)を足す。

## 確認のみ(既知・担当レーンが決まっているもの)

- [x] **6. 未知プラグインIDが即エラー(F-9パススルー未実装)** — M2-D1完了条件に「未知プラグインIDは警告+パススルー」を明記済み。実装はM2。
  現状`UnknownParamDriver`でロード失敗。F-9の決定は「警告+パススルー」。M2ドキュメントスキーマ側の残作業として認識済み — M2仕様書のD1完了条件に明記されていることだけ確認する。

- [ ] **7. T8残(中間バッファのピンポン再利用)** — M1仕様書に記載済みの残タスク。所見1のRenderStep改修と同じ箇所を触るため、同一PRか直後に片付けるのが安い。
