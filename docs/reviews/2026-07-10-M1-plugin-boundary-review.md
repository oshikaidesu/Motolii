# 設計レビュー所見 2026-07-10 (M1完了後・プラグイン境界の凍結前監査)

対象: M1残チケットR1–R9消化時点のワークツリー。観点は「あとからプラグインを足しても大丈夫か(最大ケース: 点群インポートをプラグインで)」。
これは凍結ゲートの**入場前チェックリスト**である。全項目が解消(またはスコープ外と明示判断)されるまでM2以降の並列レーンを開かない。
修正したらチェックを入れ、修正コミットにこのファイルの項番を書くこと。

## 重大(紙の凍結=G-1違反になりかねないもの)

- [ ] **1. グラフ実行器がプラグインを一度も呼ばない** — `crates/motolii-render/src/lib.rs:59`
  `RenderStep`が閉じたenum(VideoSource/SolidSource/OverlayRect/CompositeNormal)で、実行器は`OverlayNode`/`CompositeNode`を直接呼ぶ。`validate_linear_graph`は「Overlayちょうど1個・Compositeちょうど1個」までハードコード。PluginRegistryのうち製品経路を通るのはParamDriverのみで、Filter/Composite/LayerSourceは「登録できるが呼ばれる口がない」。このままtrait凍結するとG-1の入場条件(種別レジストリ経由で動く)を紙で満たしたことになる。
  修正方針: `RenderStep::Plugin { id, params, inputs, output }`相当の一般ステップを追加し、参照Filterがレジストリ経由でグラフ内から呼ばれるゴールデンを1本通す。[plugin-resources.md](../plugin-resources.md)§5-1。

- [ ] **2. 純関数契約とリソース生成禁止の両立機構がない** — `crates/motolii-plugin/src/lib.rs:113`
  「`&self`に状態を持つな」+「GPUリソースを毎フレーム作るな」を同時に守る手段がプラグインに存在しない(参照プラグインがClear系だから露呈していないだけ。コア側はRenderSessionで解決済み)。パイプラインを持つ普通のFilterを書いた瞬間に踏む。R1所見4(毎フレームのパイプライン再生成)のプラグイン版が構造的に不可避。
  修正方針: ホスト所有PipelineCache(+将来GpuAssetCache)。決定案は[plugin-resources.md](../plugin-resources.md)§3。

- [ ] **3. アセット語彙がない(点群インポートの前提欠落)** — `crates/motolii-eval/src/value.rs:5`
  `ValueType`/`Value`はF64/Vec2/Vec3/Colorのみ。ファイルパスもアセット参照も渡せず、「ファイルを食うプラグイン」が書けない。プラグイン間データはテクスチャとDataTrackの2語彙だけで、頂点バッファ等の非テクスチャ常駐データの居場所がない。後からの変種追加はparam互換(G-1)に波及する。
  修正方針: `ValueType::AssetRef`の予約+M2 D1のAsset一般化(opaque blob+type文字列)+Importer種別の契約定義。[plugin-resources.md](../plugin-resources.md)§3 D2/D3。

## 中(凍結の書き方で防げるもの)

- [ ] **4. `&'static`がv1静的リンク前提を型に焼き込んでいる** — `crates/motolii-plugin/src/lib.rs:15`
  `PluginId(&'static str)`・`ParamDef.id: &'static str`・レジストリの`&'static dyn Plugin`。v2のdylib/WASMロードは`&'static`を自然に供給できない。今の型のまま「凍結」すると、v2で凍結を破る側になる。
  修正方針: 今すぐString化はしない(静的リンクv1ではこの形が軽い)。凍結ゲート文書に「凍結対象はid/version/paramの**セマンティクス**であり、lifetime/所有形態(&'static→Cow/String化)は互換変更として許す」と明記する。

- [ ] **5. by-name参照がParamDriverにしかない** — `crates/motolii-plugin/src/lib.rs:235`
  `param_driver_by_name`のみ存在(線形走査)。ドキュメントモデル(JSON)からFilter/Composite/LayerSourceを参照する時点で全種別に必要になる。
  修正方針: 所見1の一般ステップ実装と同時に全種別のby-name(またはキーをStringに統一)を足す。

## 確認のみ(既知・担当レーンが決まっているもの)

- [ ] **6. 未知プラグインIDが即エラー(F-9パススルー未実装)** — `crates/motolii-cli/src/project.rs:109`
  現状`UnknownParamDriver`でロード失敗。F-9の決定は「警告+パススルー」。M2ドキュメントスキーマ側の残作業として認識済み — M2仕様書のD1完了条件に明記されていることだけ確認する。

- [ ] **7. T8残(中間バッファのピンポン再利用)** — M1仕様書に記載済みの残タスク。所見1のRenderStep改修と同じ箇所を触るため、同一PRか直後に片付けるのが安い。
