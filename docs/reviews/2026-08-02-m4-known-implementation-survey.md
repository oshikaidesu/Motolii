# M4既知実装調査（2026-08-02）

状態: **比較中**

対象: [M4 cache／analysis仕様](../specs/M4-cache-and-analysis.md)、
[既知実装採択・置換開発モデル](../known-implementation-adoption-model.md)

## 1. 調査目的と判定境界

M4のK0〜K8を独自実装の順番として読まず、必要な一般機構を既知実装へ割り当てる。調査対象は
具体的なAPI、固定version／commit、license、thread model、failure mode、platform条件である。
Motolii固有の作品意味、完全keyの構成、hard budget値、優先順位、製品oracleは外部実装から逆算しない。

本調査は依存追加やruntime実装を許可しない。`ADOPT-PROBE`の候補は小さいcompatibility fixtureで
依存closure、macOS／Windows、cancel、破損、hard capを確認してから`ADOPT`へ移す。候補が確認できない
ことを新機構の実装許可にせず、同じ利用者成果を保つ`REMAP / REDUCE`へ戻す。

## 2. 現行コード事実

| 領域 | 現行事実 | M4として証明しないもの |
|---|---|---|
| GPU render | `motolii-render::RenderSession`はpipeline、solid、transparent、dynamic target poolをframe間再利用する | frame／interval cache、hard budget、LRU、disk tier、copy-out |
| GPU pipeline | `motolii-gpu::PipelineCache`はWGSLとstatic IDをkeyに同期get-or-createする | pixel成果物の完全key、generation、並行store |
| 領域 | `crates/motolii-render/tests/k0_region_contract.rs`にprivateな`SpatialExtent` modelと15 testがある | runtime型、公開plugin契約、RoI最適化 |
| GPU transfer | `motolii-gpu::RgbaDownloader`はdownload bufferを再利用する | 複数in-flight copy、評価chain非blocking、RAM／disk admission |
| preview worker | `motolii-ui::render_worker`は単一pending slot、世代、latest resultを持つ | background priority queue、実行中jobの細粒度cancel、coverage planner |
| media | `motolii-media`はffmpeg／ffprobe sidecar、`FrameReaderCancel`／kill、bounded stderr、typed errorを持つ | proxy artifact、VFR→CFR生成job、pool、再起動後reuse |
| identity | `Asset.content_hash`と`sha2`依存はあるが、GAP-3のversion付きfingerprint形式は未決 | path／mtime非依存の恒久source identity、完全cache key |
| audio | `AudioProgram`は`HashMap<(content_hash, ordinal), Arc<PcmCache>>`を呼出側から受け取る | budget、eviction、共通M4 store、disk persistence |
| vector | workspaceはVello 0.9を使うがusvg／vello_svgは未導入 | SVG product import、対応subset、premul境界 |

したがって既存名の`cache`、`generation`、`pool`をM4完成へ数えない。一方、sidecar lifecycle、latest
mailbox、download buffer、Vello rendererを捨てて同義機構を作り直さない。

## 3. 機構class別の一次資料照合

### 3.1 RoD／RoI

- **既知実装**: OpenFX 1.5.1 Image Effect referenceのRegion of Definition、
  `kOfxImageEffectActionGetRegionsOfInterest`、tiled rendering。
- **供給route**: `PATTERN`。K0 private modelとfixtureは`REUSE`する。
- **移すもの**: outputが生成し得る範囲と、要求outputから入力へ逆伝播する範囲の分離。tiled非対応時は
  full RoDを要求する保守fallback。
- **移さないもの**: C ABI、property set、pixel座標、OpenFX plugin API、NatronのGPL source。
- **非証明範囲**: K0の`Finite / Infinite / Unknown`はMotoliiの安全policyで、OpenFX既定値から
  Document／公開plugin意味を作らない。

### 3.2 完全key、CAS、generation

- **既知実装**: Bazel remote cacheのaction cacheとcontent-addressable store。actionは宣言済み入力、
  output名、command、environmentから決まり、CASはcontent digestでblobを参照する。
- **供給route**: `PATTERN`。workspaceの`sha2`は`REUSE`。
- **移すもの**: recipe keyとcontent digestを分離し、入力が変われば新keyへ移る。旧generationを
  mutate／purgeせず、旧snapshotのhandleは旧keyを読み切る。
- **移さないもの**: Bazel process、remote protocol、protobuf、build action型。
- **非証明範囲**: Motolii keyのfield列挙、source fingerprint形式、GPU／driver salt、Quality非依存成果物の
  分類は製品authorityである。`document_fingerprint()`やmigration用`semantic_fingerprint()`を転用しない。

この写像により、独立した「generation invalidation framework」は作らない。writerが新しいimmutable
snapshotをpublishし、新しいrecipe keyを生成することがgeneration切替になる。

### 3.3 RAM cacheのhandle、外部参照、eviction

- **第一候補**: `foyer-memory` 0.22.3、tag commit
  `ff6b01512e580665a217c2bd892e0a884ae749e6`、Apache-2.0、MSRV 1.85。
- **具体API**: `CacheBuilder::new(capacity)`、`Cache::get/insert/remove/resize/usage`、
  `CacheEntry::weight/refs/is_outdated`、LRU／S3-FIFO等のeviction config、`Filter` admission。
- **供給route**: `ADOPT-PROBE`。`CacheEntry`が外部参照を保持し、cache本体とexternal caller双方の
  使用量を追跡する点がK1bのhandle＋遅延解放へ直接対応する。
- **probe必須**: default featureが導入するruntime、現在の同期workerとの共存、Mac／Windows build、
  全pin時の挙動、capacityを越える瞬間、drop後`usage/refs`、loomまたはupstream concurrency証跡。
- **非証明範囲**: VRAM texture resident、disk CAS、hard admission前の全owner合算、Motolii key、
  generation、GPU wait禁止。

`moka`はconcurrent cache、weighted capacity、eviction listenerを持つが、weightをeviction選択に使わず、
外部参照込み使用量を直接所有しないため第一候補にしない。`foyer-memory` probeが失敗した場合の
`REMAP`候補に限る。

### 3.4 GPU／RAM／diskのhard accounting

- **既知実装**: workspace lockのwgpu 29.0.4。`TextureDescriptor`、`BufferDescriptor`、
  `TextureFormat::block_copy_size`、`Device::generate_allocator_report()`、
  `MemoryBudgetThresholds`。
- **供給route**: descriptor計算とdiagnosticを`REUSE`、allocation-before-admissionを`PATTERN`。
- **制限**: allocator reportは`None`になり得る。budget thresholdはD3D12と任意Vulkanだけで、Metalを
  portableに守らない。よって正本はdescriptorからのHost側見積りと設定hard capである。
- **RAM**: `foyer-memory::Cache::usage`と外部`CacheEntry`参照をprobe後に接続する。
- **disk watermark候補**: Fable反対側確認後、8年以上更新のない`fs2 0.4.3`を候補から外し、同系APIを
  `rustix`／`windows-sys`で維持する`fs4 1.1.0`（MIT OR Apache-2.0、MSRV 1.75）の
  `available_space/statvfs/allocation_granularity`を`ADOPT-PROBE`する。
- **薄い残余**: owner、tier、resident/pinned、要求量を既存resource生成点から一つのadmission inputへ
  翻訳する。これは製品policy adapterであり、汎用allocator／resource frameworkを新設しない。

### 3.5 disk artifact、atomic commit、corrupt→miss

- **旧第一候補／再検索の反証**: `cacache` 13.1.0、release commit
  `66eae4b78f75eb2a38a2d25e838a56561294aebf`、Apache-2.0。
- **具体API**: `Writer/SyncWriter::commit`、`Reader/SyncReader::check`、`write_hash/read_hash`、
  SRI `Integrity`、key index、atomic content write、full-data verification。
- **供給route**: Fable助言を一次資料へ再照合した結果、第一候補から降ろして`RESURVEY`する。runtime
  featureなしでは`State`未定義でcompile不能（issue #92）、Windows write-heavyでcontent/indexがNUL破損
  する未解決報告（#83）があり、maintainerはprojectをdormantとし利用者へforkを推奨している（#94）。
  fork保守は最小コアと矛盾する。
- **再検索oracle**: 同一filesystem atomic commit、process kill、欠落index、bit flip、ENOSPC、concurrent
  same-key write、Windows rename、外部FFmpegが開ける実file、100GBを生成しない境界。
- **failure policy**: not-found、integrity error、incomplete writerはすべてcache missへ写し、Project errorや
  Media Offlineへ昇格しない。検証成功後だけstoreへ見えるcommitを行う。
- **非証明範囲**: disk hard budget／watermark、LRU policy、GPU copy-out、artifact codec、Final pixel意味。

全面`foyer::HybridCache`はRAM／disk自動階層を提供するが、heavy development、default Tokio runtime、
block cache engine、serialization contractまで所有する。さらにFFmpeg proxyが実fileを開く境界を直接
閉じない。K1cの責任を減らすかより新しいplatform／runtime責任を増やす可能性が高いため現routeでは
`REJECT`を維持する。RAMとdiskのowner分離は維持し、disk側だけを再検索する。

[disk artifact store再検索](2026-08-02-m4-disk-artifact-store-resurvey.md)で、global CAS/dedup自体が
現行M4 authorityの要求ではないと確認した。採択routeはworkspace `sha2`／`std::fs`の`REUSE`、Bazelの
recipe分離・sccacheのsharded local cache・現行D1 atomic persistの`PATTERN`、`tempfile 3.27.0`の
`ADOPT-PROBE`へ`REDUCE`する。完全recipe keyで通常reuseを保ち、content digestはintegrityへ限定する。
異なるrecipe間のglobal content dedupは、音MADfixtureでdisk重複率がbudgetを支配すると測定された場合だけ
再入場する。

### 3.6 区間coverageと部分無効化

- **候補**: `rangemap` 1.7.1、commit
  `414e9c7c10afbe576abd2513a4ffc71343d5c7bd`、MIT/Apache-2.0、依存0。
- **具体API**: half-open `RangeSet::insert/remove/gaps/overlapping/intersection/union`。隣接・重複区間を
  自動coalesceする。
- **供給route**: `ADOPT-PROBE`。Motoliiの`RationalTime`をcacheの離散tick／frame indexへ一度写し、
  K7 invalid intervalとK8 coverageに同じcollectionを使う。
- **probe結果(2026-08-02)**: `rangemap 1.7.1`の6 fixtureは`cargo test -p motolii-testkit --test m4_p06_rangemap --locked`でgreen。
  ただしraw `RangeSet::insert`はempty rangeでassert panicするため、製品へは`start < end`のprivate guardを挟む
  `REMAP / VERIFIED`とした。外部型・panicを公開境界へ漏らさない。
- **非証明範囲**: どのDocument変異がどのnode／時間窓へ影響するか、Unknownの全区間fallback、
  Quality、generation、scheduler priority。

`salsa`はtracked query、revision、red-green algorithmを解決済みだが、Document評価をSalsa databaseへ
反転し、tracked lifetimeとmemo ownerを追加する。現行D2／render graphを置換するためdependencyとして
`REJECT`する。完全key＋immutable generation＋`RangeSet`で同じ成果を既存owner内へ接続する。

### 3.7 background priorityとcancel

- **候補**: `priority-queue` 2.7.0、commit
  `0c76fb8fe75e4457f16f8e6c8e86508d1a89ba1d`、`LGPL-3.0-or-later OR MPL-2.0`のうちMPL-2.0。
- **具体API**: `PriorityQueue::push/change_priority/remove/pop`。item lookup O(1)、priority更新 O(log N)。
- **供給route**: data structureだけ`ADOPT-PROBE`。executor lifecycleは既存`LatestWorker`の
  thread／Condvar／generation／typed close patternを`REUSE/PATTERN`する。
- **cancel**: `FrameReaderCancel`／killをmedia jobへ`REUSE`し、CPU／GPU jobはjob tokenをtile／数frame
  境界で確認する。Tokio `CancellationToken`を得るためだけにasync runtimeを新設しない。
- **薄い残余**: K8の固定4優先度、latest seekでのpriority更新、bounded queue、job result message、
  heartbeat。queueはDocument／cacheへ直接書かず、ownerへmessageを返す。
- **非証明範囲**: OS thread priority、強制preemption、GPU command途中停止、ffmpeg process supervisor共通化。

### 3.8 proxyとVFR→CFR

- **既知実装**: 現行`motolii-media`のFFmpeg sidecarを`REUSE/WRAP`。FFmpeg `fps` filterはPTSから
  指定CFRへframeをdrop／duplicateし、`-fps_mode cfr`もCFR出力を定義する。ffprobeはpacket／frameの
  PTS、duration、stream metadataを機械可読で出す。
- **route**: 新しいmedia runtimeや`ffmpeg-sidecar` crateを採らず、既存spawn／stderr／cancel／finishへ
  proxy jobを追加する。scaleとfpsだけをproxy artifactへ焼き、色解釈、LUT、Document parameterを焼かない。
- **identity**: proxyはGAP-3で確定する`source_id`とcodec recipeの派生CAS artifact。path／mtimeをkeyにしない。
- **非証明範囲**: hardware decode、decoder pool、GAP-26 process lifecycle、具体proxy codec／bitrate、
  `30000/1001`誤ラベルpolicy。

### 3.9 SVG→Vello

- **候補**: `vello_svg` 0.10.0（Apache-2.0 OR MIT、MSRV 1.88）。0.9.0は名前に反してVello 0.7依存であり、
  workspaceのVello 0.9に対応するのは0.10.0である。同crateはcompatible `vello`／`usvg`をre-exportし、
  `append_tree/render_tree`で`usvg::Tree`をVello `Scene`へ接続する。
- **供給route**: `ADOPT-PROBE`。独自SVG parserとusvg→Vello walkerを作らない。
- **probe必須**: K6のpath／group／fill／stroke corpus、unsupported featureのtyped診断、外部resource遮断、
  Renderer長寿命、CQ-7 straight→premul一回、現native device/format。
- **非証明範囲**: `vello_svg`が明記するtext、group opacity、clip、mask、filter等の未対応機能。
  K6の必要subsetを越えて支持を公約しない。

## 4. M4へ採らない一般機構

| 候補 | 処分 | 理由 |
|---|---|---|
| `cacache 13.1.0` | `REJECT` | dormant/fork保守推奨、runtime featureなしcompile不能、Windows破損未解決 |
| 若い専用CAS crate | `REJECT` | backup／DB／WASM等のformat、WAL、chunking、runtimeを増やし、出荷実績が小さい |
| Natron／Olive source | `REJECT` | GPL sourceを製品実装の移植元にしない。公開仕様と利用者成果だけを参照 |
| Salsa dependency | `REJECT` | D2／render graphの評価authorityをdatabase／tracked queryへ反転する |
| 全面Foyer HybridCache | `REJECT` | block engine、Tokio、serializationまで責任が広がり、CAS artifact境界が不透明になる |
| 独自DB／WAL／cache protocol | `REJECT` | journalとは別の恒久format、migration、repair、lockingを増やす |
| `moka`第一候補 | `REMAP ONLY` | 外部handle込みusageと厳密admissionを直接閉じず、K1bの主要責任が残る |
| async priority channel／Tokio runtime | `REJECT` | queueのためにruntime ownerを追加する。既存thread workerとpriority collectionで閉じる |
| custom SVG parser／walker | `REJECT` | `vello_svg`がversion-compatible seamを既に所有する |
| cache purgeを通常回復にするUI | `REJECT` | corrupt／missingはmiss、key漏れはbugとして修理する |

## 5. 採択probeの閉鎖条件

1. dependencyはexact version、license、feature closureを固定し、公開型／Document／serdeへ漏らさない。
2. `foyer-memory`はexternal handle込みusage、全pin、drop、Mac／Windows buildを確認する。
3. disk artifactは`tempfile`／`sha2`／single writerのprobeでprocess kill、bit flip、missing、
   concurrent completionをすべてmissまたは非公開artifactへ閉じる。global CASは初期契約にしない。
4. `rangemap`は`RationalTime`を直接keyにせず、既存composition timebase上のhalf-open integer intervalへ写す。
5. `priority-queue`はbounded、reprioritize、cancel、editor nonblockingを決定的fixtureで確認する。
6. `vello_svg`はK6 subsetだけを閉じ、unsupported featureをsilent dropしない。
7. probe不合格時は候補を独自実装へ置換せず、同じ親の`REMAP / REDUCE`へ戻す。

## 6. 一次資料

- [OpenFX Image Processing Architectures](https://openfx.readthedocs.io/en/latest/Reference/ofxProcessingArch.html)
- [Bazel Remote Caching](https://bazel.build/remote/caching)
- [foyer-memory API](https://docs.rs/foyer-memory/0.22.3/foyer_memory/)、
  [foyer architecture](https://foyer-rs.github.io/foyer/docs/design/architecture)
- [cacache 13.1.0](https://docs.rs/cacache/13.1.0/cacache/)（旧候補）、
  [sync-only compile issue #92](https://github.com/zkat/cacache-rs/issues/92)、
  [Windows corruption #83](https://github.com/zkat/cacache-rs/issues/83)、
  [maintenance status #94](https://github.com/zkat/cacache-rs/issues/94)
- [wgpu MemoryBudgetThresholds](https://docs.rs/wgpu/29.0.4/wgpu/struct.MemoryBudgetThresholds.html)、
  [Device allocator report](https://docs.rs/wgpu/29.0.4/wgpu/struct.Device.html#method.generate_allocator_report)
- [rangemap RangeSet](https://docs.rs/rangemap/1.7.1/rangemap/set/struct.RangeSet.html)
- [priority-queue PriorityQueue](https://docs.rs/priority-queue/2.7.0/priority_queue/struct.PriorityQueue.html)
- [FFmpeg fps filter](https://ffmpeg.org/ffmpeg-filters.html#fps)、
  [ffmpeg fps_mode](https://ffmpeg.org/ffmpeg.html)、[ffprobe](https://ffmpeg.org/ffprobe.html)
- [vello_svg 0.10.0](https://docs.rs/vello_svg/0.10.0/vello_svg/)、
  [usvg](https://docs.rs/usvg/)

初版調査時には既決の発明禁止原則を外部LLMへ二重相談しなかった。その後、候補技術の反対側比較という
新しい問いに限ってFable 5をread-onlyで呼び、`fs4`／`vello_svg`の訂正と`cacache`のriskを得た。Codexが
crates.io、docs.rs、GitHub issue、現行codeへ再照合し、disk routeは別途再検索した。runtime／platformで
未確認の部分は`ADOPT`へ過大昇格せず、下流地図の`ADOPTION_PROBE`に残す。
