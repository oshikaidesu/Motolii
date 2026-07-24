# Controlled Microkernel／全体並列化 Fable反対側レビュー

作成日: 2026-07-25

状態: **初回REVISE後に訂正、限定再検収ACCEPT。P0/P1=0**

対象:

- [Controlled MicrokernelとHost capability module並列化決定](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md)
- `concept.md`、`extensible-core-model.md`、`plugin-authoring.md`、`dev-experience.md`
- 現行のDocument／D2、plugin contract、render worker、GPU health、Preview／Exportコード

検収者: Claude Fable 5 (`claude-fable-5`)、read-only。最終採否とコード事実の再照合はCodexが行った。

## 1. 結論

設計は実行可能な方向であり、次段のread-only seat inventoryへ進めてよい。

Fable初回レビューは、設計そのものではなく次の事実精度2件をP1として`REVISE`した。

1. `concept.md`と決定文書が、未実装のwgpu error scope、device復帰、instance再生成を現行防御層の
   ように読めた。
2. `pitfalls-and-roadmap.md` F-9が、2026-07-25信頼境界改訂で置換された
   「ピクセル系＝ネイティブ第一級市民」を維持と書いていた。

Codexが現行コードを再照合して両方を訂正した。限定再検収では新規P0/P1なし、
`VERDICT: ACCEPT`となった。

## 2. 現行コードに実在するseam

| seam | コード事実 | 現在言えること |
|---|---|---|
| Commit Authority | `crates/motolii-doc/src/lib.rs`の`DocumentWriter`、`apply_macro` rollback、revision | `SERIAL AUTHORITY`は実証済み |
| immutable snapshot | `DocumentWriter::snapshot()`とUIのrevision付き`PublishedDocument` | read-only consumerの基礎は存在 |
| plugin contract | `PluginContract`、`PluginCatalog`、`PluginRegistry`、`PluginRuntime::try_new` | 表現pluginの`PARALLEL IMPLEMENTATION`は三つの外部crateで一度実証 |
| typed failure | `PluginDiagnostic`、Exportのdegraded plugin拒否 | 欠落／future versionを局所診断できる |
| Preview／Export | 両consumerが`build_document_frame_graph`と`render_graph_cached`を使用 | 同一評価意味のconsumer分離は存在 |
| generation付きworker | latest-wins mailbox、`RenderGeneration`、worker境界`catch_unwind` | 一worker規模の`PARALLEL RUNTIME`原型 |

## 3. 未成立を完成扱いしない

次は設計候補または仕様であり、現行コードでは未成立である。

- frame cache。現行はshader `PipelineCache`のみ。
- VRAM resource admission／ledger。
- journalと通常編集commit点の接続、journal／session完全復元。
- 評価DAGの独立subgraph並列。現行`LinearRenderGraph`はstep順に実行する。
- plugin process isolation、worker再spawn、device再生成。
- plugin dispatch単位panic隔離、wgpu error scope、device-lost復帰。
- Host moduleのhot swap transaction。
- `exactly one / many / ordered chain`を一般化したauthority slot registry。

したがって「moduleへ分けられる」「並列に実装できる」「runtimeで同時実行できる」
「別processで障害隔離できる」を同じ完了宣言にしない。

## 4. seat別の暫定判定

| seat | 判定 |
|---|---|
| Commit Authority、Stable ID Authority | `SERIAL AUTHORITY` |
| 表現plugin | `PARALLEL IMPLEMENTATION`実証済み。`ISOLATED WORKER`は未実証 |
| Journal durability | 二つのfilesystem実装があり実装面は候補。commit配線は未実証 |
| Evaluation planner | `NOT YET PROVEN`。contract型の中立化候補をinventoryで記録 |
| Cache、resource admission、asset resolver | `NOT YET PROVEN` |
| Preview worker | 一worker規模の`PARALLEL RUNTIME`原型 |
| Export consumer | `PARALLEL IMPLEMENTATION`原型。frame loopは意図的直列 |
| UI projection `many` | React製品接続前のため`NOT YET PROVEN` |
| lifecycle／instance交換 | `NOT YET PROVEN` |

## 5. 最小proof順序

1. **seat×code read-only inventory**
   - 各seatへpath、実装数、owner、多重度、執行地点、負例を記録する。
   - コード移動や新しい公開型が必要なら`EXTRACT CONTRACT`と記して停止する。
2. **authority不変条件の被覆地図**
   - single writer、変更0、snapshot、session lockを既存test IDへ対応づける。
3. **二consumer同一意味oracle**
   - 同じsnapshot、時刻、`Quality::FINAL`でPreview／Exportのpixel意味を比較する。
4. **instance交換dry run**
   - 現行入口だけでrender workerを破棄・再spawnし、同じrevisionから同じ結果へ戻れるか測る。
   - swap用公開APIが必要なら実装せず、独立decisionへ戻す。
5. **journal-at-commit oracle案**
   - apply、append、kill、replayの等価と失敗時変更0をfixtureとして閉じる。
6. **最初のseat裁定**
   - 二実装とfault fixtureを持つjournal、またはM4仕様を持つcacheを候補にし、
     一seatずつ個別decisionへ上げる。

## 6. 継続するP2

- scheduling、backpressure、observabilityはCoreが意味contractだけを持ち、queue／thread pool／backendは
  module policyへ置く。
- snapshotとrollbackがDocument cloneを使うため、多consumer化前に40 layer fixtureで費用を測る。
- Host module交換ではplugin IDだけでなくmodule実装世代／artifact identityをcacheへ寄与させるか、
  activate時に影響範囲を全無効化する。
- Bitwigから移すのはfailure domainを段階化する思想まで。manual crash reloadをdeveloper hot reloadと
  同一視せず、音声IPCをGPU shared textureの成立証拠にしない。

## 7. 停止線

- inventoryからtrait、crate、ABI、manifestを自動生成しない。
- `NOT YET PROVEN`を並列化済みに読み替えない。
- first-party provenanceをTCB admissionの代わりにしない。
- runtime DAG、process外GPU worker、cache key、journal形式が公開API／Document／serde面を変えるなら
  対象seatの独立decisionへ戻す。
- Fableの助言を仕様正本そのものにせず、現行コードとMotolii fixtureで再審判する。
