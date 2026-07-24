# Plugin resource runtime lineageの価値回収（Unit 3B-runtime-A、2026-07-23）

状態: **縮小採用**（歴史文書5 blobの処分、F-10/F-11の実装状態訂正）

対象: `plugin-resources.md` 4版と`2026-07-17-vism-a0-plugin-boundary-inventory.md` 1版。

関連: [全歴史coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)、[plugin resource正本](../plugin-resources.md)、[VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)、[M4仕様](../specs/M4-cache-and-analysis.md)

## 1. 結論

2 path / 5 blobを、初版全文、全版差分、現行コード事実で処分した。履歴が見つけた問題は正しかったが、「凍結した設計」と「コードに存在する公開契約」が途中で混ざっていた。

```text
plugin純関数 + 毎frame resource生成禁止の衝突
  → Host所有PipelineCacheを導入・Tintで実証
  → AssetRefとImporter/GpuAssetCache構想を追加
  → 未実装GpuAssetCacheまで「契約シグネチャ凍結」と記述
  → Document AssetRef結線は実装、Importer/GpuAssetCacheは未実装のまま
  → lookbehind予約とHost所有Feedback設計を追加
```

現行へ維持する境界は次の四つである。

1. GPU pipeline等の反復資源はplugin instanceへ隠さず、Host所有cacheから決定的なdescriptor keyで借りる。`PipelineCache`、`RenderStep::Plugin`、Tintおよび外部first-party pluginの利用で実装済みである。
2. `ValueType::AssetRef` / `Value::AssetRef(u64)`とDocumentの`AssetId`、存在検査、prepared resolutionは実装済みである。一方、Importer trait、opaque payload実行契約、`GpuAssetCache`型、prepare lifecycle、budget admissionは存在しない。これらを「M2完了」または「凍結済みsignature」と称さない。
3. cache residencyは実行状態であって作品入力ではない。同じ入力・`t`・明示`Quality`で「今residentなもの」だけを描いて結果を変える旧D4は棄却する。DraftのLODやbudgetを許す場合も明示入力から決定的に選び、未準備時の待機・unavailable・代替投影はM4/M5で別に閉じる。
4. `CompLookbehind` / `TemporalFootprint`は型の予約まで実装済みだが、texture解決とFeedback checkpoint executorは未実装である。前frameをplugin内部へ隠す`StatefulFilter`は禁止を維持し、Feedbackは初期条件・固定step・checkpointをHostが所有する後続契約とする。

## 2. 個別処分

| 歴史path / blob | 分類 | 判定 | 現在の回収先 |
|---|---|---|---|
| `docs/plugin-resources.md` / `ed4f8bea`,`28e0e1f3`,`cc8df184`,`fc095c62` | **現行規範 + 成立理由 + 未実装候補 + 負例** | PipelineCache、AssetRef、Host所有時間依存という責任分離は維持。未実装GpuAssetCacheのsignature凍結、Importerの具体payload、availability依存Draftを現行契約として扱わない。点群のLayer OrderとGroup Depthの分離、p5.js型蓄積のFeedback分類は後続正本へ吸収済み | 本書§3〜5、[plugin-resources](../plugin-resources.md)、[M4](../specs/M4-cache-and-analysis.md)、[M5](../specs/M5-3d-and-post.md) |
| `docs/reviews/2026-07-17-vism-a0-plugin-boundary-inventory.md` / `2b385734` | **歴史的コードsnapshot + 負例** | 当時のstatic registry、手書き既知表、非対称product route、予約kindを正確に記録した。後続A0D/A0I/A1〜A3がcontract catalog、first-party crate、一般LayerSourceを実装したため現在値として再掲しない。trait存在だけで製品接続済みとしない審判は維持 | [VSM-A0 inventory](2026-07-17-vism-a0-plugin-boundary-inventory.md)、本書§4 |

## 3. F-10の現在地

### 3.1 実装済み

| 境界 | コード事実 |
|---|---|
| pipeline cache | `motolii-gpu::PipelineCache`と限定されたpipeline定型、`motolii-plugin` façade |
| plugin dispatch | `RenderStep::Plugin`、Filter/Composite/LayerSource runtime dispatch |
| first-party利用 | Opacity、Sine、Radial Repeaterが公開façadeとHost composition rootを通る |
| asset identity | Document `AssetId` / `AssetTable`、type、content hash、path多重キー |
| parameter reference | `ValueType::AssetRef`、`Value::AssetRef(u64)`、Document `DocValue::AssetRef(AssetId)`、dangling拒否 |

### 3.2 未実装・未凍結

repository全体に`GpuAssetCache`の型・trait・実装は無く、`PluginKind::Input`は予約enumでregistry entry 0件である。`asset.rs`のコメントとM2 D1aは将来責任の置き場を示すが、executorを証明しない。

従って、次は再入場時に個別に決める。

- importerの入力bytes、出力payload identity、version、diagnostic、sandbox、再現性
- CPU payloadからGPU表現へのprepare owner、handle、破棄、失敗、再試行
- content hashとplugin/version/prepare descriptor/Qualityを含むcache key
- VRAM/RAM hard budget、resident/pinned、eviction、使用中handleの寿命
- missing file、hash mismatch、未install importer、未準備assetのtyped failure

旧文書の`opaque blob + type文字列 + 内容ハッシュ`と「消費pluginがprepareする」は候補であり、そのままpublic traitへ転記しない。M4 K1a/K1bのResourceLedgerとcache identity、Vism package/trust、実素材fixtureを通してから閉じる。

## 4. VSM-A0 snapshotから維持する審判

VSM-A0は2026-07-17時点で、static singleton registryが将来Vismの配布境界ではないことを示した。その後、immutable Contract CatalogとExecutor Registryの分離、外部first-party plugin crate、一般LayerSource loweringは実装された。従って当時の「五つの継ぎ目」やA1 WAITを現在のbacklogへ戻さない。

ただし次の負例は現在も有効である。

- `PluginKind` enum、trait、render dispatchの存在だけで、Documentからの製品routeや配布可能性を証明しない。
- 予約kind `Input / Simulation / ScriptWasm`を実装済み能力と数えない。
- `'static`な`PluginId`やtrait objectをVism package identity、install、version併存の意味へ昇格しない。
- resource要求、capability、migration、diagnosticを`NodeDesc`へ便宜的に足してmanifestを代用しない。
- Host composition rootへ登録されたfirst-party実装と、第三者を安全にloadできるruntimeを同義にしない。

## 5. residencyと時間依存の訂正

### 5.1 residencyは出力入力ではない

旧D4のDraft「ポイントバジェット内で今あるものを描く」は、同じ`(Document, assets, plugin version, t, Quality)`でもcache warm/coldやI/O競合でpixelが変わる。そのため現行決定論と両立しない。

Draftで低解像度、低poly、低密度等を使うこと自体は維持できる。ただし選択は明示`Quality`と安定入力の関数でなければならない。実データがまだ無い場合に、待つ、typed unavailableを返す、最後に確定した別generationをUIだけへ投影する、のどれを採るかは作品意味、render結果、UI readinessを混ぜず後続契約で決める。cacheに偶然残っているsubsetを意味にしない。

### 5.2 lookbehindとFeedback

`CompLookbehind`と`RenderCtx::lookbehind`、`TemporalFootprint`は予約済みだが、常に未配線または既定値である。これを「F-11実装済み」と読まない。

非clear canvasは次の漸化式として扱う方針を維持する。

```text
A0 = transparent
An = Composite(DecayOrTransform(An-1), Drawn)
```

有限のshape出現へ意味を保って畳める場合はmaterializeし、前出力そのものが必要な場合だけHost所有Feedbackへ送る。checkpoint間隔、cache key、invalidation、Quality差、RoD/RoI damageはM4/SCR-4の実装契約まで未決であり、予約structから推測しない。

## 6. 復活させない旧具体とSTOP線

- `GpuAssetCache`、Importer、opaque payload、prepare handleを凍結済みpublic APIとして実装しない。
- M2完了をGpuAssetCache完成の証拠にしない。M2が閉じたのはAsset metadata、AssetId参照、検査・resolutionまでである。
- cache residency、I/O完了順、前回previewの偶然をrender結果の入力にしない。
- `PluginKind::Input`をImporterと読み替え、既存kind名だけでbytes/payload/package意味を決めない。
- point cloudのRGBA LayerSourceからdepthを密輸しない。Layer Orderとshared depth参加はM5境界で分ける。
- lookbehind予約を任意時刻アクセスAPIへ広げず、plugin instanceに前frame、GPU handle、checkpointを保持させない。
- VSM-A0当時のstatic registry不足を理由に、後続Contract Catalogやfirst-party composition実装を巻き戻さない。

## 7. 固定歴史出典

| lineage | 読み方 |
|---|---|
| plugin resources | 初版`ed4f8bea`全文、PipelineCache/Tint実証版`28e0e1f3`、凍結表現版`cc8df184`、point cloud depth・p5 Feedback追補版`fc095c62`まで全diff確認 |
| VSM-A0 inventory | 単一版`2b385734`全文を読み、現行Contract Catalog、first-party crates、general LayerSource、予約kind 0件と照合 |

これら5 blobは本書でDISPOSITIONEDとする。native/WASM、能力・sandbox、first/third-party runtimeの残りはUnit 3B-runtime-Bとして別処分する。
