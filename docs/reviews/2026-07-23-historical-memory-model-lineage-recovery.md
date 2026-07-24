# memory modelの価値回収（Unit 5L、2026-07-23）

状態: **階層責任／hard budget決定維持／ResourceLedger・K7・K8未実装**

対象: [memory model](../memory-model.md) cutoff全6版（61,009 bytes）

関連: [Unit 5C readback回収](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md)、[Unit 5J M4回収](2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md)、[Unit 5K performance回収](2026-07-23-historical-performance-model-lineage-recovery.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

6版を通じた恒久価値は、pixel処理の作業セットをVRAMへ保ちながら、容量を食うcache／bake／proxyをRAMとdiskへ逃がす責任分離である。容量はHost所有のResourceLedgerで事前見積りし、driver pagingやportableでない空きVRAM値へ委ねず、allocation admission前のhard capで止める。容量逼迫と再生期限超過を別の制御loopにし、DocumentではなくUser settings／Transientへ置く。

同時に、歴史文書の強すぎる性能表現を分離する。固定2枚の中間target、約400MB、40動画layerでも1GB未満は直列graphのfloorにすぎない。現行render target poolはbranchのlive入力を避けて伸びる。非同期copy-out pipeline、ResourceLedger、disk store、K7 group bake、K8全曲Draft coverageはまだ製品codeにない。

## 2. 6版の処分

| 主題 | 処分 |
|---|---|
| VRAM作業セット／RAM・disk容量階層 | **維持**。CPU合成へ戻す理由にはしない |
| 評価途中の同期readback禁止 | **維持**。確定出力の非同期copy-outだけを許可 |
| copy-out実証済み | **敗北**。後続版でbuffer再利用までへ訂正され、現行exportも直列待ち |
| fixed 2 target／約400MB／layer非依存 | **縮小**。直列graphのfloorで、branch liveness等を含む保証ではない |
| diskは実質無制限 | **敗北**。User settingsのhard budgetとaccountingが必要 |
| Host ResourceLedger／hard cap | **維持**。M4 K1契約だが未実装 |
| capacity／deadline別制御 | **維持**。frame dropで1 frameの容量は減らない |
| K7 group bake／K8全曲Draft | **維持**。disk階層の要件であり未実装 |
| 37GB／100GB | **審判用試算**。製品既定値、実format、性能保証ではない |

## 3. 現行コードとの照合

- render target poolは2枚から始まるが、branchのfuture live inputを避け、必要ならtargetを追加する。固定2枚のVRAM計算を一般化できない。
- exportはframeごとにrender、download完了待ち、encodeを直列実行する。buffer再利用とbounded in-flight copy-outは別である。
- wgpu budget threshold設定とGPU health検出は存在するが、全texture／buffer／cache／prefetch／stagingをowner別に数えるResourceLedgerではない。
- `PipelineCache`とrender target poolはGPU resource再利用であり、RAM／disk cache store、LRU、hard budget admissionの完成証拠ではない。
- M4台帳上、K1、K7、K8は全て`WAIT`である。freeze policy、全曲coverage、100GB fake／sparse fixtureも未成立である。

## 4. 再入場条件

1. K1は全ownerのdescriptor accounting、resident／pinned、共有memory合算、解放後zero、typed rejectionを小さい注入budgetで固定する。
2. copy-outはexport stagingとcache成果物を分け、Queue競合、bounded memory、cancel、device lost、backpressureを測ってから方式と本数を採択する。
3. capacity制御はallocation admission、deadline制御はGPU time／queue latencyで判定し、project fps、audio clock、Finalを変更しない。
4. K7／K8はK0／K1の完成後に入り、generation invalidation、partial interval、priority、disk hard budget、restart recoveryをfixture化する。
5. 40-layer／100GBは実formatと全ownerの計測を持ち、歴史概算をSLOへ昇格させない。

## 5. 復活させないもの

- branch graphを固定2 targetへ押し込み、live textureを上書きすること。
- GPU名、総RAM、allocator report、driver pagingだけでadmissionを決めること。
- diskを無制限と扱い、User settingsのhard budgetを迂回すること。
- capacity不足をFPS低下だけで解決したことにすること。
- Draft縮退やframe dropをFinal、project time、audio clockへ持ち込むこと。
- 既存pool、wgpu threshold、headless DRSをK1／K7／K8完成と数えること。
- 100GBの実file生成をCIへ要求すること、または100GBをDocument意味へ焼くこと。
- cache都合のfreeze state、budget、backend型をDocument／plugin公開契約へ入れること。

## 6. 固定証跡とcoverage

6 blobの完全SHAはreceipt `05l-memory-model.tsv`を正本とする。合計61,009 bytes。

本Unit後のstrict progressは417 / 1,797（23.2%）、未処分1,380である。
