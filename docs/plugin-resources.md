# プラグインのリソースライフサイクルとアセット境界

作成日: 2026-07-10
ステータス: **縮小採用**。`PipelineCache`と`AssetRef`結線、時間参照型の予約は実装済み。`GpuAssetCache`／Importer／Feedback executorは未実装・未凍結([歴史回収](reviews/2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md))
関連: [plugin-authoring.md](plugin-authoring.md)(作者契約)、[pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) F-10/G-1、[memory-model.md](memory-model.md)(VRAM予算)、[concept.md](concept.md)

## 1. 問題(なぜこの文書が要るか)

2026-07-10の`PipelineCache`導入前、プラグイン契約には両立不能な2つの規律があった:

- 「隠れた可変状態を持たない」(純関数契約。[plugin-authoring.md](plugin-authoring.md)§3-3)
- 「ループ内でGPUリソースを毎フレーム新規生成しない」(同§3-6)

参照プラグイン(Clear系)はリソースを持たないため矛盾が露呈していなかったが、**パイプラインを1本持つ普通のFilterを書いた瞬間に両立機構がないことが露呈した**。この問題はHost所有`PipelineCache`とTint実証で解消済みである。

さらに上位の要件として「点群インポートもプラグインでやれる」がある。数千万点の点群を毎フレームVRAMへアップロードするのは論外であり、(a) ファイル→GPU常駐データの経路、(b) 常駐データの寿命管理、(c) プラグインがそれを参照する語彙、の3つが契約に必要になる。現在のプラグイン間データ語彙はテクスチャ(`TextureRef`)とDataTrackの2つだけで、頂点バッファはどちらでもない。

これを決めずにM2並列化に入ると、D1(Assetスキーマ)とD3(doc→グラフ変換)が各自の解釈でリソース管理を発明し、OpenCut型(境界の後付け→全書き直し)を再演する。

## 2. 先人の解決策(パッチワーク元の台帳)

先人は誰も「毎フレームアップロード」をプラグイン契約の工夫で解いていない。全員が**アセット=常駐(ホスト管理・1回GPU化)/評価=毎フレーム(純関数・ハンドル参照)の分離**と、**内容ハッシュをキーにしたメモ化**で解いている。

| 困りごと | 先人 | 機構 |
|---|---|---|
| パイプラインを毎フレーム作れない | Bevy `PipelineCache` / 自前の`RenderSession` | プラグインはパイプラインを所有せず、descriptorをキーに**ホストのキャッシュへ要求**。初回だけコンパイル、以降ヒット。状態を持つのはホストだけ→純関数契約が壊れない |
| アセットを毎フレームアップロードできない | Bevy `RenderAsset` + Nuke Opハッシュ | CPUアセットが作成/変更された時だけ`prepare`が走りGPU表現(頂点バッファ等)を作る。キャッシュキーは**アセット内容ハッシュ**なのでメモ化=純関数のまま |
| リソース生成の「時点」の分離 | OpenFX(`createInstance`/GLコンテキストattach) / AE SDK(GPUDeviceSetup) | 「純粋なrender呼び出し」と「リソース生成アクション」をライフサイクル上で分離。ただしプラグイン側が状態を所有する方式で、うちの純関数契約とは相性が悪い(採らない。キャッシュ所有はホスト側=Bevy方式を採る) |
| 部分無効化(点だけ動いた等) | Nuke `GeoOp`のハッシュグループ | ジオメトリのハッシュをPoints/Primitives/Attributes/Matrixに分割し、点移動でトポロジを再構築しない |
| 数千万点のVRAM超過 | Potree / Unreal LiDAR Point Cloud / COPC | インポート時に**八分木LOD**を前処理構築し、実行時は**ポイントバジェット+スクリーンスペース誤差**で必要ノードだけVRAM常駐(out-of-core)。数十億点でもVRAM使用はバジェット分で一定。COPCは八分木階層をファイル内に埋め込み部分読み出し前提 |
| 大量点の描画自体 | Schützら(compute rasterization) | compute shaderによるソフトウェアラスタライズで数十億点をリアルタイム描画。wgpu computeで再現可能=ベンダー非依存(F-9と整合) |
| ストリーミングと決定性の衝突 | オフラインレンダラ一般 | 書き出し時は必要データのロード完了まで**ブロック**。プレビューだけがバジェット内近似を許す |

## 3. 決定案(凍結ゲートでコード実証を経て確定)

### D1. パイプライン等の共有リソース: ホスト所有キャッシュ

プラグインはGPUパイプライン/シェーダモジュール/サンプラを所有しない。renderに渡るコンテキスト経由で**ホストのPipelineCache**にdescriptor(WGSLソース+レイアウト)をキーとして要求する。ホストが生成・保持・再利用する。

- `RenderSession`の一般化として実装する(コアノードのOverlayNode/CompositeNodeも最終的に同じキャッシュに乗せてよい)
- プラグインの`&self`は引き続き状態ゼロ。純関数契約§3-3/§3-6は「両方守れる」ようになる

### D2. アセット候補: Importer種別 + ホスト所有GpuAssetCache（未実装・未凍結）

- **Importer種別の候補**(予約済み`PluginKind::Input`とはまだ同義でない): `ファイル(バイト列) → Assetペイロード(opaque blob + type文字列 + 内容ハッシュ)`。CPU側の変換であり、GPU・時刻・パラメータ評価に触らない
- Assetペイロードの中身は**コアが解釈しない**(点群の八分木もフォントもコアには不透明)。type文字列(例: `pointcloud.octree.v1`)で消費側プラグインが自分の食えるものか判定する
- **GpuAssetCache(ホスト所有)の候補**: 消費側プラグイン(主にLayerSource)が`prepare(gpu, asset) → GPU常駐表現`を実装し、ホストがキャッシュする。現行repositoryに型・trait・登録口は無い。key、handle、budget、失敗、再試行、sandboxをM4/Vism/実素材fixtureで閉じる前に、この擬似signatureを公開APIへ転記しない
- 寿命・予算は[memory-model.md](memory-model.md)のVRAM予算+LRUに従う(M4のキャッシュ層と同じ台帳。キャッシュキー思想は既存の「ノードID×時間区間×パラメータハッシュ」と同型)

点群の`Layer Order`時の姿: `Importer(.ply/.laz/COPC → 八分木blob)` + `LayerSource(アセットハンドル + CompCamera → RGBAテクスチャ)` の2プラグイン。`Group Depth`等へ参加する場合は、RGBAへdepthを密輸せず、M5-P2Dで定めるHostのobject/world/depth参加境界へ同じ点群表現を供給する。既存LayerSource契約だけを全遮蔽ポリシーの唯一経路とはしない。

### D3. パラメータ語彙: AssetRef（予約・Document結線とも実装済み）

`ValueType`/`Value`の**AssetRef(アセットID参照)**は、パラメータスキーマの互換性(F-9/G-1のparam同一性)に波及するため凍結ゲートで予約し、M2でDocument `AssetId`、存在検査、prepared resolutionまで結線した。これはImporter/GpuAssetCacheの実行契約があることを意味しない。

- M2 D1のAsset定義は「動画・SVG素材」から「**opaqueペイロード+type文字列を持つ一般アセット**」に広げる(動画・SVGはその特殊ケース)

### D4. レジデンシと決定性: residencyを出力入力にしない

ストリーミングLODは`render_frame(t, Quality)`の決定性(B-4)と衝突する。先人に倣い:

- **FINAL候補**: 要求LODのロード完了まで待つ。具体的な待機・失敗契約は未決
- **DRAFT**: 低密度LOD等を使う場合も、選択は明示`Quality`と安定入力の決定的な関数にする。cacheに「今あるもの」だけを描いてwarm/coldでpixelを変えない

未準備時に待つ、typed unavailableを返す、最後に確定した別generationをUIだけへ投影する、の選択はM4/M5で別に閉じる。readinessはUI投影やscheduler状態であり、Document意味またはrender結果の隠れ入力ではない。

v1では口の予約のみ(実装は解析駆動フェーズ/点群導入時)。Quality型が既にこの拡張の自然な口になっている。

### D5. 部分無効化の口(実装はM4以降)

GpuAssetCacheのキーを将来「構成要素別ハッシュ」(Nuke GeoOp方式: 位置/トポロジ/属性)に分割できるよう、キー型を単一ハッシュ値に固定しない(タプル/構造体にしておく)。v1は全体ハッシュ1本でよい。

## 4. v1でやること / やらないこと

| やる(凍結ゲート〜M2) | やらない(予約のみ) |
|---|---|
| PipelineCacheの最小実装+パイプラインを持つ参照Filter 1個での実証 | Importer種別と`PluginKind::Input`の具体的意味 |
| `ValueType::AssetRef`の予約とDocument結線 | 点群本体(八分木・バジェット・compute rasterizer) |
| M2 D1のAsset metadata一般化(type+hash) | LODストリーミング、residency/readiness契約 |
| Document Asset metadataと`AssetRef`結線（実装済み） | GpuAssetCache／Importerの公開契約、構成要素別ハッシュ分割 |

## 5. 凍結ゲートとの関係(並列化の入場条件)

[pitfalls-and-roadmap.md](pitfalls-and-roadmap.md) G-1の入場条件に従い、**紙のまま凍結しない**。この文書の決定を凍結ゲートに通すための実証:

1. **グラフ実行器のプラグインディスパッチ**: `RenderStep`に一般ステップ(プラグインID+params+入出力)を追加し、参照FilterがPluginRegistry経由でグラフ内から呼ばれてゴールデンが通る（当時の不足。後続で実装済み）
2. **パイプラインを持つFilter 1個**: Clear系ではない実Filter(単純なティント/ボックスブラー程度)がPipelineCache経由でパイプラインを取得し、毎フレーム再コンパイルなしで動くことをテストで確認(R1所見4と同じ検査をプラグインにも適用)
3. 1と2が通った時点で、render trait+PipelineCacheの**契約シグネチャ**を凍結する。AssetRefは型を予約し、Document結線をM2で実装する。GpuAssetCache/Importerは設計方向だけを残し、実コードなしにsignatureを凍結しない

このうち1=Filterディスパッチ、2=TintFilter+PipelineCacheは2026-07-10に達し、AssetRefのDocument結線もM2で実装済みである。歴史的な残件と当時の入場条件は[reviews/2026-07-10-freeze-gate-remaining.md](reviews/2026-07-10-freeze-gate-remaining.md)に保存する。GpuAssetCache/Importerはその完了から自動的に成立しておらず、再入場時にM4 ResourceLedger/cache identity、Vism trust、実素材fixtureを要する。

## 6. 時間参照: lookbehind / フィードバック(F-11。口の予約のみ)

作成日: 2026-07-10。「合体後(グループ/コンポ)の前フレーム」を使う効果(残像・データモッシュ的グリッチ)は、現行契約では書けない — Filterが見るのは自レイヤー1枚、Compositeが見るのは現在tの入力列だけで、**合体結果を別時刻で取りに行く語彙がない**。プラグインが前フレームを`&self`に覚える解は純関数契約・フレーム並列・スクラブを壊すため恒久禁止([plugin-authoring.md](plugin-authoring.md)§3-3)。解はプラグインの賢さではなく、**ホストが渡す時間参照**である。

### 6-1. 契約の形(予約)

新種別は作らない。Composite/Filterの**追加入力としてホストが`TextureRef`列を渡す**だけ:

```text
CompLookbehind {
  target: GroupId | CompRoot,   // 合体単位
  offsets: [-1, -2, ...],       // フレーム
  exclude: [自エフェクト等],      // 自己参照の切断(非再帰化)
}
```

### 6-2. コストモデル(2種類は別物。混ぜない)

| モード | 順再生/書き出し | スクラブ | フレーム並列 | 先人 |
|---|---|---|---|---|
| **lookbehind(非再帰。excludeで自己参照なし)** | 償却ほぼゼロ — `G_ex(t-1)`は前フレーム描画時の中間テクスチャそのもので、M4キャッシュ(ノード×時間×params)にヒットする。払うのはVRAM(offsets数×グループ1枚、[memory-model.md](memory-model.md)予算内) | 有界(offsets数×グループ再評価、キャッシュヒットあり) | 保たれる(各フレームの依存は純関数) | **Nuke**: 任意ノードが入力を別時刻で要求し、コストはホストのハッシュキャッシュが吸収(TimeOffset/TemporalMedian)。反面教師は**AE Echo**(キャッシュ規律なしの素朴なN回再評価で遅い) |
| **フィードバック(再帰。自分を含む)** | ゼロ — 前フレーム最終出力を1枚生かすだけ | ≤Kフレームのリプレイ — **Kフレームごとにチェックポイントを焼き、直近から再生**(グループ仮出力=ベイクが供給源) | 区間内は直列(漸化式なので本質的)。**チェックポイント区間単位では並列**(コーデックがGOP単位で並列エンコードするのと同型) | 順再生コストは**TouchDesigner Feedback TOP / AviUtlフレームバッファ**。ただし両者は再生ヘッド依存(非決定)なのでそのまま採らない。決定性の回復は**動画コーデックのGOP構造**(チェックポイント+リプレイ。C-1のスナップショット+ジャーナルリプレイのピクセル版) |

### 6-3. 決定性の定義(ここが再生ヘッド依存との分水嶺)

フィードバックは「**クリップ開始時刻を初期条件とする漸化式**」として定義する。これにより`render_frame(t, Quality)`は純関数のまま(素朴に計算すると高いだけ)で、順再生・チェックポイントは定義ではなく**最適化**になる。TD/AviUtl型の「スクラブすると結果が変わる」を構造的に排除する。Draftはチェックポイント間隔・リプレイ解像度を粗くしてよい(Draftの近似はB-4上もともと許容)。FINALは厳密リプレイ。

非clear canvas型の蓄積描画は、この一般形の具体例として扱う:

```text
A₀ = transparent
Aₙ = Composite(DecayOrTransform(Aₙ₋₁), Drawₙ)
```

`Drawₙ`はShapeScript等をone-shot実行して得た固定seed・固定stepの命令列で、JS runtimeはレンダへ常駐しない。`DecayOrTransform`が恒等で各draw命令を通常shapeの出現時刻へ畳んでもpixel意味が保てる場合は、フィードバックを使わずmaterializeする(安い力優先)。半透明の反復合成・blur・変形等で前出力そのものが必要な場合だけ、ホスト所有のFeedback stateへ昇格する。

「画面全体を毎step更新しない」は意味論ではなく実行最適化である。Feedbackは論理的にはtarget surfaceを定義するが、実更新はK0のRoD/RoIを基礎にSCR-4でdamage伝播を定義して限定する。局所drawはdirty領域だけ、blur/transformはfootprint分だけ拡張し、global effectだけが全target更新へ退化する。チェックポイントtextureと中間結果はVRAM常駐のまま扱う。

### 6-4. v1の態度

- **今**: 空間グリッチ(現在フレームのみの歪み)はFilterで書く。合体前フレームは当てにしない
- **予約(凍結ゲート項目17)**: `CompLookbehind`のスキーマ/契約口。F-7(インスタンスインデックス)と同じ「口の予約のみ・実装は後」の棚
- **実装時期**: lookbehindはM4キャッシュ後ならいつでも安い。フィードバック(チェックポイント)はグループ仮出力(M4)に依存
- **恒久にやらない**: StatefulFilter、再生ヘッド依存の隠しバッファ
