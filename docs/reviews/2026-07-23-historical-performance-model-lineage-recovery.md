# performance modelの価値回収（Unit 5K、2026-07-23）

状態: **帯域／VRAM規律維持／liveness-aware pool実装済み／製品性能未証明**

対象: [performance model](../performance-model.md) cutoff全21版（290,748 bytes）

関連: [Unit 5C readback回収](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md)、[Unit 5D transport回収](2026-07-23-historical-d5-transport-lineage-recovery.md)、[Unit 5J M4回収](2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

performance modelの核は、comp workが帯域支配であるため、pixelをVRAMへ置き、CPU↔GPU往復と同期点を避け、resourceをframe間再利用することにある。この規律は現行codeにも残る。

最も重要な回収は、中間render targetが固定2枚ではないことである。直列graphでは2枚のping-pongになるが、branchでは後続stepがまだ読むtextureを上書きできない。現行`RenderTargetPool`はlive入力を避け、全候補がliveならpoolを伸ばす。歴史版`902e723e…`が記録したこの仕様は現行codeと一致し、その後の文書で失われていた。

一方、fp16既定、選択的fp32、path fusion、40動画layer、decode pool、製品GPU timestamp DRS、AE比1〜2桁は未実装または未測定である。設計の方向と製品SLOを分離する。

## 2. 21版の処分

| 主題 | 処分 |
|---|---|
| 帯域支配／VRAM常駐／同期readback禁止 | **維持**。確定出力の非同期copy-outだけを別境界で許す |
| 固定2枚ping-pong | **縮小**。直列時の下限。branchではlivenessに応じてpool伸長 |
| O(n²) liveness scan | **現行code fact**。graph planでの事前計算は候補 |
| pure render／immutable snapshot | **維持**。逐次stateはrender外Bakeへ送る |
| simulation全面禁止枝 | **敗北**。hidden render state禁止へ縮小 |
| DRS 1/2→1/4 | **方向維持**。headless controller成立とproduct GPU接続を分離 |
| capacity pressure／deadline分離 | **維持**。M4 K1dは未実装 |
| Quality render desc | **維持**。両辺整除・exact aspect時だけ縮小する現行codeと一致 |
| fp16／Final linear／fusion／batching | **候補**。現行RGBA8 pathを完成扱いしない |
| 40動画layer／AE比 | **歴史仮説**。benchmarkとbudgetの根拠にしない |

## 3. 現行コードとの照合

- `RenderSession::acquire_render_target`はdescriptor変更時に2枚で初期化し、`avoid`に含まれないtextureをround-robinで返す。全候補がliveなら新規targetを追加する。
- future live inputは各stepで後続graphを走査して集める。正しさは成立するがO(n²)で、plan事前計算は未実装である。
- `FilterPlugin`／`CompositePlugin`はHostのGPU context、pipeline cache、encoder、render contextとtyped `TextureRef`を使う。歴史上のraw引数説明は現行signatureではない。
- canonical outputはRGBA8。`Rgba16Float`予約や性能散文からfp16 pipeline成立を推論しない。
- `Quality::render_desc`はscale 1 identity、両辺整除／exact aspect／packed成功時だけ縮小し、それ以外はfullを返す。
- DRS controllerは成立しているがproduct render passはtimestampを書かず、UI／previewの実測接続は未成立である。

## 4. 再入場条件

1. liveness precomputeは現行branch fixtureのpixel不変とpool上限を固定してから、graph planだけを差し替える。
2. fp16／fp32はformat別shader、blend、alpha、color、copy／export、device featureのfixtureを先に閉じる。
3. path fusion／batchingは代表graphのdispatch数とpixel同一を測り、plugin意味やcache keyを変えない。
4. 40-layerはdecode-only、upload、render、readback、encodeを分離し、VRAM／RAM／disk hard budgetと実機evidenceを持つ。
5. product DRSはGPU timestamp、unsupported backend、audio clock、frame drop、fixed resolution、Final不変を同じfixtureで閉じる。

## 5. 復活させないもの

- branch graphを固定2 targetで上書きし、livenessを削ること。
- 古い帯域概算をhardware budget、SLO、実装完成証拠にすること。
- current controllerをproduct DRS、PipelineCacheをframe cache、Rgba16Float予約をfp16完成と数えること。
- timestamp非対応時にwall clockだけでauto DRSし、project fps／audio clock／Finalを変えること。
- Draft／Finalでstate trajectoryを変え、simulation stateをplugin内部へ隠すこと。
- nondivisible render descをtruncateし、scaled descをcanonical authorityにすること。
- evaluation途中の同期readback、CPU pixel fallback、vendor APIをplugin契約へ追加すること。
- benchmarkを通すためgolden、tolerance、期待値を変更すること。

## 6. 固定証跡とcoverage

21 blobの完全SHAはreceipt `05k-performance-model.tsv`を正本とする。合計290,748 bytes。

本Unit後のstrict progressは411 / 1,797（22.9%）、未処分1,386である。
