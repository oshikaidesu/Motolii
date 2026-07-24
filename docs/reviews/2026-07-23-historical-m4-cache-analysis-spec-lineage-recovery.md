# M4 cache／analysis仕様の価値回収（Unit 5J、2026-07-23）

状態: **cache契約維持／K0〜K8未実装**

対象: [M4 cache／analysis仕様](../specs/M4-cache-and-analysis.md) cutoff全20版（200,882 bytes）

関連: [memory model](../memory-model.md)、[simulation model](../simulation-model.md)、[GAP-3](../backlog.md)、[coverage台帳](2026-07-23-historical-value-recovery-coverage-ledger.md)

## 1. 結論

20版から残す核は、cacheを作品意味から切り離したHost専権の再計算資産として扱うことである。完全key、参照handleと遅延解放、単段lock、hard budget、破損／欠落時のmiss、容量pressureとdeadlineの分離、確定成果物だけの非同期copy-outを維持する。

途中には「逐次simulationは設計上存在しない」とする敗北枝がある。現行はrender内のhidden stateを禁止しつつ、render外でHostがStateTrack／checkpointを管理し、renderは確定状態を純関数入力として読む。逆に、現行`PipelineCache`と`RenderSession`のping-pong texture再利用はM1資産であり、M4のResourceLedger、cache store、K7 freeze、K8 coverageの実装証拠ではない。

## 2. 20版の遷移

| 段階 | 処分 |
|---|---|
| 初期K1〜K5 | node×interval×input hash、RAM／VRAM LRU、proxy、CFR、analysis、RoIの問題分割を維持 |
| K6／K7追加 | SVG／Velloとgroup仮出力を独立taskへ分離した判断を維持。製品実装済みとはしない |
| F-2／memory model | handle、遅延解放、単段lock、generation snapshot、message返却、disk bakeを維持 |
| analysis移動 | K3／K5 producerを最終phaseへ移した判断を維持 |
| simulation分岐 | StateTrackを第3のinterval clientにした枝を採択。「逐次simulation全拒否」枝を敗北扱いにする |
| Quality／完全key | pixel cacheへQuality／FrameDesc、Quality非依存DataTrack／StateTrack、Host所有を維持 |
| guard 1〜16 | Purge不要、content identity、environment salt、atomic commit、corrupt=miss、hard budget、proxy非解釈、editor nonblockingを維持 |
| K0／K1a〜d／K7a〜c／K8a〜b | umbrella ticketを契約境界へ分割した現行構成を維持 |

## 3. 現行コードとの照合

- `motolii-render::RenderSession`は2枚のping-pong targetをframe間再利用するが、M4 cache entry、budget、LRU、disk tierではない。
- `motolii-gpu::PipelineCache`はWGSL pipelineの同期get-or-createで、frame／interval成果物storeではない。
- `ResourceLedger`、`SpatialExtent`、`PreviewPressureController`、cache handle／store、K7／K8 executorはworkspaceに存在しない。
- `PluginKind::Simulation`と`TemporalFootprint`は予約済みだが、StateTrack／Simulation executorは未実装である。
- DataTrack値評価は実装済みだが、analysis producerとinterval cacheは未実装である。
- Vello／usvgはworkspace dependencyにもproduct pathにも無く、K6は未実装である。

## 4. 再入場条件

1. K0はDocument schemaを増やさず、Unknownの全域fallback、過小Finite拒否、pixel同一、同期readback無しをfixture化する。
2. K1a→b→cはhard cap、完全key、handle lifecycle、loom／stress、corrupt=miss、全pin typed rejectを順番に閉じる。
3. K1dはK1c＋K4後、capacityとdeadlineを別注入し、hysteresis、Document／Final不変を判定する。
4. K2はtarget／Shared Effectの選択的invalidatonをfixture化し、全purgeで代用しない。
5. K4はGAP-3でversion付きfingerprint意味を先に閉じ、mtime／pathをidentityにしない。
6. K6はCQ-7の単一straight→premul境界と局所Vello renderer判断に従い、SVG core意味からproduct APIを発明しない。
7. K7／K8はcache policy／Transient stateに留め、Document、journal、Undo、Final意味へ混ぜない。

## 5. 復活させないもの

- cache warmth、LRU、proxy availability、manual freezeでpixel意味を変えること。
- plugin APIへcache配置、予算、self-cache、backend型を出すこと。
- path、mtime、時計だけをidentityにし、またはGAP-3未決のhead／tail hash形式を恒久化すること。
- cache破損をProject error／Media Offline扱いにし、Purgeを通常回復にすること。
- lock保持中のGPU wait、使用中entryの破棄、background jobからDocument／cacheへの直書き。
- 評価chain途中の同期readback、全pin時のOS OOM、測定前の固定staging本数。
- capacity不足をframe dropだけで直すこと、deadline遅延をcache purgeだけで直すこと。
- 敗北枝から逐次simulation全面禁止を戻すこと、またはFilter内部stateを合法化すること。
- 同一machine／driver内のcache透明性oracleをcross-GPU bit一致公約へ拡張すること。
- OpenCV／ONNX、Vello／usvg、RoI公開契約を一括で導入すること。

## 6. 固定証跡とcoverage

20 blobの完全SHAはreceipt `05j-m4-cache-analysis-spec.tsv`を正本とする。合計200,882 bytes。

本Unit後のstrict progressは390 / 1,797（21.7%）、未処分1,407である。
