# M4 既知実装採択・並列実装地図

状態: **直列核契約追補済み／runtime未発注**（2026-08-08）

## 1. この地図が置き換えるもの

本書はM4のK0〜K8を独自cache frameworkの施工順として読む計画を置き換える。親項目は検索と
供給routeの単位、子項目は一つのowner、接続target、oracle、cutoverを持つ実装packageである。
既知実装の調査証跡、version、license、非証明範囲は
[M4既知実装調査](reviews/2026-08-02-m4-known-implementation-survey.md)を正とする。

本書だけでは実装を許可しない。候補はまず`ADOPTION_PROBE`でdependency closureとfailure modeを閉じ、
採択後に薄い接続粒を[implementation ledger](implementation-ledger.md)の一意な`DO`へ上げる。K0〜K8の
旧IDは製品oracleと依存の来歴として残すが、旧列を粒ごとに再証明したり、自動的に順番実装したりしない。

## 2. 固定原則

1. Motoliiは完全key、作品意味、hard admission、優先順位、製品oracleを所有する。cache collection、
   区間集合、priority collection、SVG変換の一般機構は既知実装へ委ねる。diskはCASを前提にせず、
   映像編集製品で成立済みの再生成可能な通常artifact file方式へ合わせる。
2. `REUSE / ADOPT / WRAP / PATTERN`を親で一度裁定し、子は再選定しない。
3. 外部型をDocument、公開plugin API、serde、journalへ漏らさない。adapterはprivateな一方向写像にする。
4. cache miss、破損、不完全writeは作品failureにせず再計算へ落とす。key漏れはpurge UIで隠さない。
5. budget admissionはallocationまたはbounded write chunkの前、GPU waitとUI thread blockingは禁止、previewとFinalは同じ評価関数を使う。
6. 並列施工はDocument writer、GPU device、artifact commit、製品runtime合流だけを直列点にする。
7. probe不合格は`BUILD`許可ではない。同じ成果を`REMAP / REDUCE`し、独自DB、WAL、scheduler runtime、
   SVG parser、invalidation frameworkを作らない。
8. verified artifactからHost所有GPU resourceへの昇格は作品意味でなくHost privateな実行policyとする。
   現行portable stagingを`REUSE`し、platform direct I/Oはsource／license／platform matrix、wgpu接続、代表workload
   実測が揃うまで[観察](reviews/2026-08-06-storage-to-gpu-direct-io-design-observation.md)に留める。backend名、
   vendor handle、架空trait、設定UIをDocument／公開plugin API／Vismへ予約せず、本地図の新parent／`DO`を増やさない。
9. Rerun Spatial Viewerはinteractive spatial runtimeとして`ADOPT / WRAP`する。scene／view／query／camera／picking／
   visualizer／GPU drawをM4で再実装しない。M4はその上のhard budget、cache identity、invalidation、proxy、artifact、
   pressure／縮退を補完し、Rerunのstore／viewer cacheをDocument、作品cache、ResourceLedger、Preview／Exportの第二authorityにしない。
   Rerun採択をPBR、Simulation solver、またはcross-tier resource policyの完成証拠とみなさない。

## 3. 親項目の検索入口

| 親ID | 検索key | 利用者成果 | 採択route | M4範囲 |
|---|---|---|---|---|
| `M4-P01-REGION` | RoD RoI tile extent unknown propagation | 必要領域だけを安全に評価し、未知は全域fallbackする | K0 fixtureを`REUSE`、OpenFXを`PATTERN` | K0、K2 |
| `M4-P02-IDENTITY` | recipe key content digest generation snapshot source fingerprint | 再起動やrename後も正しい成果だけを再利用する | Bazel action/CASを`PATTERN`、`sha2`を`REUSE` | K1b、K2、K4、GAP-3のrelink／offline route |
| `M4-P03-RAM` | weighted cache handle refs pin eviction usage | 参照中成果を壊さずRAM hard cap内で再利用する | `foyer-memory 0.22.3`を`REMAP / VERIFIED`。外部生存量はprivate owner | K1a〜K1c |
| `M4-P04-RESOURCE` | VRAM RAM disk admission descriptor allocator report watermark | allocation／bounded write chunk前に三tierの上限を守る | wgpu descriptor/reportを`REUSE/PATTERN`、`fs4 1.1.0`を`VERIFIED (observation)` | K1a、K1c、K1d |
| `M4-P05-DISK` | artifact store generated media render cache proxy integrity atomic corrupt miss LRU | 再起動後も壊れていない成果だけをdiskから読む | Blender/AE/Premiere/Resolve/FCPの再生成mediaを`PATTERN`、`sha2/std::fs`を`REUSE`、sccache/D1 persistを`PATTERN`、`tempfile 3.27.0`を`VERIFIED (V1 BASELINE ONLY)`。kill／ENOSPC／race／Windows runtimeを含むV2以降はruntime absent STOP | K1c、K7、K8 |
| `M4-P06-INTERVAL` | half-open range coverage gaps invalidation coalesce | 変更の影響区間だけを再計算する | `rangemap 1.7.1`を`REMAP / VERIFIED` | K2、K7b、K8a |
| `M4-P07-SCHEDULE` | priority bounded queue reprioritize cancel heartbeat latest | 編集を止めず、必要なbackground成果から作る | `priority-queue 2.7.0`を`REMAP / VERIFIED`、`LatestWorker`を`REUSE/PATTERN` | K1d、K4、K7、K8 |
| `M4-P08-PROXY` | ffmpeg ffprobe VFR CFR proxy PTS source id | 重い素材を決定的proxyへ置換して編集する | 現行FFmpeg sidecarを`REUSE/WRAP` | K4、GAP-3、GAP-26 |
| `M4-P09-COPYOUT` | wgpu copy map staging ring readback overlap | GPU評価を止めずに再利用成果をRAM/diskへ送る | `RgbaDownloader`を`REUSE`、wgpu copy/mapを`PATTERN`。方式採択は`STOP / GAP-29` | K1c、K7a、GAP-29 |
| `M4-P10-BAKE` | group bake atomic artifact substitute freeze | Groupの編集可能性を保ったまま仮出力を再利用する | P02/P05/P06/P07/P09の合成、独立frameworkなし | K7a〜K7c |
| `M4-P11-COVERAGE` | whole composition draft coverage planner 100GB | 全曲Draftを計画し、disk成果で通し再生する | P05/P06/P07の既知機構を合成 | K8a、K8b |
| `M4-P12-PRESSURE` | capacity deadline preview degrade resource snapshot | 容量不足と締切遅延を混同せずpreviewを縮退する | wgpu/resource snapshotとlatest mailboxを`REUSE` | K1d |
| `M4-P13-VECTOR` | SVG usvg Vello path fill stroke unsupported premul | SVGの必要subsetを独自parserなしで描く | `vello_svg 0.10.0`を`REMAP / VERIFIED`、Vello 0.9を`REUSE` | K6 |

## 4. 現在のdispatch状態

| 子 | 状態 | 閉じるもの |
|---|---|---|
| `P03-C1` | `REMAP / VERIFIED` | `foyer-memory 0.22.3`のweight／handle／resize／filter／並行操作、3 target cross-build。外部handle生存量はprivate ownerへREMAP |
| `P04-C4` | `VERIFIED` | `fs4 1.1.0`のfree-space／allocation granularity観測、missing path typed error、3 target cross-build |
| `P05-C1` | `VERIFIED (V1 BASELINE ONLY)` | `tempfile 3.27.0` same-dir temp／persist、FFmpeg temp artifact、atomic visibility fixture、path-independent SHA-256、stale temp隔離。kill／ENOSPC／same-recipe race／Windows runtime visibilityは未検証でV2/V3へ残る |
| `P06-C1` | `REMAP / VERIFIED` | half-open integer timebase、coalesce、gap、境界overflow。raw empty rangeはpanicするためprivate guardを必須化 |
| `P07-C1` | `REMAP / VERIFIED` | MPL-2.0選択、reprioritize/remove/pop、composite priorityによるdeterministic ordering。bounded admissionとgeneration filterはprivate owner |
| `P13-C1` | `REMAP / VERIFIED` | `vello_svg 0.10.0`のpath/group/fill/stroke、typed parse error、pattern diagnostic、3 target cross-build。外部fileはusvg段で無言dropするためprivate preflightへREMAP |
| `P02-C1` | `CONTRACT CLOSED / CODEC IMPLEMENTED 2026-08-10` | [直列核4契約](reviews/2026-08-08-serial-core-known-contracts-decision.md)でHost-private canonical `RecipeKeyV1`と別`ArtifactDigest`を決定。codec+mutation corpusは`f731384c`で`crates/motolii-render/src/recipe_key.rs`へmain到達(8 test)。runtime key helper(render graph実値収集)のみ別ticket |
| `P02-C2` | `CONTRACT CLOSED / IMPLEMENTED 2026-08-09` | `SourceFingerprintV1`をprovenance tag付きsource exact bytes SHA-256+sizeへ決定し、producer/decodeが`crates/motolii-doc/src/asset.rs`(`d273061d`/`260bcfde`)、budgeted SourceBindingが`crates/motolii-media/src/source_binding.rs`(`a287c828`)としてmain到達。legacy opaque hashの非昇格・strict再hashも実装済み。relink adapter(M2-ASSET-1C)は別ticketのまま |
| `M4-P02-C3` | `CONTRACT CLOSED / IMPLEMENTATION NOT STARTED` | exhaustive Command classifierとatomic state envelopeの唯一のowner。K1b storeとK2統合は別ticket |
| `P09-C1` | `STOP / GAP-29` | 現行baselineの同期1-buffer readback guardは確認済み。copy/map/encode/disk原因分離とring数採択は未計測のため固定しない |
| `P05-C2` | `STOP / RUNTIME ABSENT` | V1後のrestart／recipe・store-format・catalog version／store handleを接続するprivate disk storeが現行codeにない。新しいstore ownerをこの検証branchで発明しない |
| `P05-C3` | `STOP / RUNTIME ABSENT` | ResourceLedger、disk hard budget、pin/committing eviction routeが未実装。P04観測だけでresource integrationを証明しない |
| `K7a/K8b` | `STOP / RUNTIME ABSENT` | group bake／full-composition Draft／cache playbackのproduct producerとE2E routeが未実装。現行PipelineCache/exportを完成証拠にしない |
| その他 | `STOP / DEPENDENCY` | 停止したauthority／runtime absentに依存する後続粒。新しいownerや意味を発明せず、実装branchで再入場 |

probeは互いにruntime ownerを書かない小fixtureとして並列化できる。ただし同じ`Cargo.toml`／lockfileの
変更を競合させないため、各probeは独立差分で検収し、採択依存を一つの直列publicationへまとめる。

## 5. 子項目

### M4-P01-REGION — 評価領域

#### `P01-C1` private runtime extent seam

- **結果**: K0の`Finite / Infinite / Unknown`とunion/intersection/fallbackをprivate runtime型へ接続する。
- **再利用target**: `crates/motolii-render/tests/k0_region_contract.rs`、render graphのinput/output extent。
- **薄い残余**: canonical spaceとtexture boundsの一回限りの写像。
- **oracle**: Unknown/Infiniteは過小評価0、empty/overflow安全、公開plugin型とDocument変更0。
- **cutover**: component別rect helper、pixel座標の恒久意味を作らない。

#### `P01-C2` dependency propagation

- **結果**: output要求から各input RoIへ逆伝播し、非対応nodeはfull RoDへfallbackする。
- **依存／並列**: P01-C1後。P02 key構築とfile分離できる。
- **oracle**: full renderとtile renderの同一pixel、Unknownで欠落0、領域外work削減を測定。

### M4-P02-IDENTITY — 完全keyと世代

#### `P02-C1` recipe key and content digest

- **結果**: node/version/parameter/input digest/time/Quality/platform saltを正本順でencodeし、recipe keyとartifact digestを分ける方針を確認した。
- **再利用target**: `sha2`、既存typed IDs、render graph入力。
- **oracle**: 各fieldの単独変異でmiss、並べ替え非同値、path/label/display名の混入0。
- **判定**: `CONTRACT CLOSED / IMPLEMENTATION NOT STARTED`。[直列核4契約](reviews/2026-08-08-serial-core-known-contracts-decision.md)でdomain-separated、tag+length-prefixのHost-private canonical `RecipeKeyV1`と別`ArtifactDigest`を決定した。exact codec、mutation oracle、runtime key helperは別ticketで実装する。
- **cutover**: ad-hoc文字列key、`semantic_fingerprint()`転用、cache別key helperはcanonical codec acceptance後にretireする。

#### `P02-C2` source identity closure

- **結果**: `SourceFingerprintV1`をsource exact bytesの`motolii-source-v1:sha256:<64 lowercase hex>`+sizeへ決定し、proxy/cache/relinkへ同じcontent identityを渡す。
- **状態**: `CONTRACT CLOSED / IMPLEMENTATION NOT STARTED`。prefixなしlegacy opaque hashは文字列shapeだけでauthorityへ昇格せず、strict再hashを要求する。strict codec、Asset admission、M2-ASSET-1C product admit／SourceBinding、relink adapterは別ticket。
- **oracle**: rename同一、内容差異は不一致、full SHA-256形のlegacy値もV1へ自動昇格せず、loaded V1 tagも最初のbinding一致前はpersistent-cache hit authorityにしない。source-consuming workerへraw locatorを渡さず、Host-owned immutable bindingのexact bytesと保存済みfingerprintを一致させる。full-copy bindingはsource全長を事前予約し、queued／running job中はpin、最終capability drop後にだけreleaseする。

#### `M4-P02-C3` immutable generation snapshot

- **結果／owner**: 本cutだけがexhaustive Command classifierとatomic state envelopeを所有する。Document edit threadがvalidated candidateをaccepted Commandで分類し、journal acceptance後に`(immutable snapshot, CacheEpoch, InvalidationFootprint)`を一つのHost-private state envelopeとしてserialized atomic publishする。render-relevant／unknown時だけepochを進め、reader／jobは一つのenvelopeを一回だけ取得する。current-state transient resultだけをepoch不一致で拒否し、完全key付きartifactは旧keyの下へpublishできる。K1bはcache store、K2は接続consumerでありclassifier／envelopeを再実装しない。P06-C2は本cutが分類済みのaffected identityを区間へ写すpure helper、P06-C3はpublished footprintをcoverageへ反映するconsumerに限る。
- **依存／並列**: M2-D3、M2-D3e、対象target／source Command意味のacceptance後。K1bとは別ownerで並列可、K2、P06-C2/C3、P07-C2は本cutを待つ。
- **薄い残余**: 非永続`CacheEpoch { session_identity, counter }`、immutable snapshot、footprint、job candidate receiptのprivate結合だけ。checked counter exhaustion前にfresh non-reused session identity／counter 0／whole-composition invalidationをatomic installし、Document edit failureやepoch衝突にしない。
- **oracle**: 新snapshot／旧epoch観測0、単一envelope capture、unknown全域fallback、epoch不一致のcurrent-state result publish拒否、完全key付き旧snapshot artifactの旧key publish許可、session renewal衝突0、参照中artifactの早期解放0。

### M4-P03-RAM — 参照handle付きRAM cache

#### `P03-C1` foyer-memory compatibility probe

- **結果**: `foyer-memory 0.22.3`のdefault feature（`foyer-tokio` runtime closure）でK1bのweighted cache、external refs、resize、filter、並行操作を実測した。
- **変更範囲**: dependencyと隔離fixtureのみ。製品runtime、公開型、Document変更0。
- **証拠**: `crates/motolii-testkit/tests/m4_p03_foyer_memory.rs`の4 fixtureがgreen。weight／clone refs／outdated handle／resize／filter／concurrent get-insert-removeを確認し、`cargo check --locked`を`x86_64-pc-windows-gnu`、`aarch64-apple-darwin`、`x86_64-unknown-linux-gnu`で通過させた。
- **判定**: `REMAP / VERIFIED`。weighted eviction、entry handle、resize、filter、並行データ構造は採用する。ただしentryがcacheからremoveされると、外部handleを保持していても`Cache::usage()`が0へ下がるため、実メモリhard capの外部生存量はfoyerへ委ねずprivate ownerで加算する。独自concurrent cacheは作らない。

#### `P03-C2` RAM artifact cache adapter

- **結果**: recipe key→content digest→bufferの二段参照をRAMにも通し、digest/weight/admissionを
  `foyer-memory::Cache`へprivateに写してtyped handleを返す。
- **依存／並列**: K1a/P03-C1/P04-C1/M4-P02-CODEC acceptance後。disk adapterと並列。
- **oracle**: single lock order、lookup時GPU wait 0、pin中evict後もread可、missは再計算。同一source区間・
  同一parameterのN sliceでcontent bufferは一つ。
- **cutover**: cache別HashMap/LRU、同義Arc registryをretire。

#### `P03-C3` audio cache migration

- **結果**: `AudioProgram`へ外から渡す無制限HashMapを共通RAM admissionへ接続する。
- **依存／並列**: M2-ASSET-1CとP03-C2 acceptance後。K1cと並列可だが、K8b通し再生E2Eは本cutを待つ。
- **oracle**: audio clock非blocking、同一PCM、budget accounting、旧二重owner 0。

### M4-P04-RESOURCE — 三tier admission

#### `P04-C1` descriptor-based estimator

- **結果**: texture/buffer/PCM/artifactのdescriptorからallocation前bytesを算出する。
- **再利用target**: wgpu descriptors、`TextureFormat::block_copy_size`、既存`MemoryBudgetThresholds`。
- **oracle**: mip/array/alignment/format境界、overflow拒否、実allocation後の差分diagnostic。

#### `P04-C2` unified hard admission

- **結果／owner**: owner/tier/resident/pinned/requested bytesをK1aが所有する一つのHost policyへ渡すprivate tier adapter。本cutは第二policy、permit identity、allocator frameworkを所有しない。
- **依存／並列**: K1a + P04-C1 acceptance後。P04-C3 diagnosticとは別consumerとして進められるが、P05-C2/C3とK1cは本cutを待つ。
- **薄い残余**: 製品policyとtyped refusalだけ。allocatorやresource frameworkを作らない。
- **oracle**: allocation／bounded write chunk前の保守的上限予約、上限不明typed refusal、事後追加admission 0、全pin時typed refusal、VRAM/RAM/disk二重計上0。

#### `P04-C3` diagnostic snapshots

- **結果**: wgpu allocator reportがある時だけ差分診断へ使い、hard cap正本にはしない。
- **oracle**: report `None`でも成立、Metal/Windowsで同じHost policy、HUDはread-only。

#### `P04-C4` disk watermark probe

- **結果**: `fs4 1.1.0`のsync APIで既存directory／fileのfree spaceとallocation granularityを観測し、missing pathをtyped errorとして確認した。
- **証拠**: `crates/motolii-testkit/tests/m4_p04_fs4.rs`の3 fixtureがmacOS hostでgreen。`cargo check --locked`を`x86_64-pc-windows-gnu`、`aarch64-apple-darwin`、`x86_64-unknown-linux-gnu`で通過させた。
- **判定**: `VERIFIED`。fs4は観測だけを肩代わりし、hard budget、admission、eviction、Document/Project failureはMotolii private ownerに残す。cross-buildは各OSのruntime permission挙動を証明しないため、製品fallbackの実機保証とは分離する。

### M4-P05-DISK — verified recipe artifact store

#### `P05-C1` filesystem artifact compatibility probe

- **結果**: `tempfile 3.27.0`のsame-directory temp／persist、workspace `sha2`、現行FFmpeg `Encoder`の通常file出力を隔離fixtureで確認した。
- **証拠**: `crates/motolii-media/tests/m4_p05_artifact_compatibility.rs`の4 fixtureがmacOS hostでgreen。tempとfinalの同一filesystem、publish前の旧final保持、FFmpeg tempのprobe、path-independent SHA-256とbit-flip検出、stale tempのfinal非可視を確認し、3 target cross-buildも通過した。
- **判定**: `VERIFIED (V1 BASELINE ONLY)`。実証済みはcross-build、通常Rust／FFmpeg same-dir temp／persist、atomic visibility fixture、SHA-256、stale temp隔離だけ。V2 store model（restart／kill／corruption／same-recipe race／Windows runtime／handle）、V3 resource integration（ENOSPC／hard budget／lazy scan）、V4 K7/K8 product E2Eはこのfixtureの完了へ読み替えず、別層として残す。
- **証明分界**: 先行製品は通常file／再生成／明示削除の妥協範囲だけを証明する。process kill、同一recipe競合、allocation前hard budget、Windows runtime atomic visibilityは後続層の未検証責任である。
- **STOP**: fork、独自DB/WAL、cache format migration、常駐runtime、global content dedupを採択前提にしない。

#### `P05-C2` private disk store adapter

- **結果**: project／cache-local immutable artifact object、完全recipe key→result metadata+content digest、volatile LRU metadataをprivate
  APIへ閉じる方針を確認した。global CAS／cross-project dedupへ広げない。
- **依存／並列**: K1a/P05-C1/P04-C2/M4-P02-CODEC acceptance後。RAM adapterと並列。
- **oracle**: restart hit、rename hit、wrong recipe／store-format／catalog version miss、外部storage型の公開面0。同一process内で
  検証済みhandleを再利用し、frameごとの全file再hash 0。
- **判定**: `STOP / RUNTIME ABSENT`。P02-C1の意味は閉じたが、現行repoにcanonical codec、store owner／routeがない。test-only storeを製品adapterの代替として発明しない。
- **cutover**: 独自DB/WAL/catalog/repair protocolを作らない。

#### `P05-C3` disk budget and retirement

- **結果**: watermarkとvolatile access metadataでeviction候補を選び、content digestは不変に保つ方針を確認した。
- **依存／並列**: P05-C2、P04-C2、P04-C4 acceptance後。K1cのdisk tier合流は本cutを待つ。
- **oracle**: pinned/committing artifact削除0、削除失敗はtyped diagnostic、再起動後missから回復。
- **判定**: `STOP / RUNTIME ABSENT`。ResourceLedger／disk admission／eviction ownerが未実装で、`fs4`観測やV1 fixtureから統合済みと推論しない。

### M4-P06-INTERVAL — coverageと部分無効化

#### `P06-C1` RangeSet compatibility probe

- **結果**: 映像はframe index、音声はsample indexの別`RangeSet`としてinteger half-open
  rangeのinsert/remove/gaps/coalesceを固定する。
- **実証**: `crates/motolii-testkit/tests/m4_p06_rangemap.rs`で6 testsが`--locked` green。adjacent／overlapのcoalesce、
  remove分割、bounded gaps、end-exclusive、i64境界、frame/sample owner分離を確認した。
- **負例**: `RangeSet::insert(5..5)`はraw APIのassert panic。製品routeへ直接漏らさず、`start < end`を先に検査する
  private typed guardへ`REMAP`する。これはempty/overflowを「試験期待値変更」で隠したものではなく、採択条件として固定した。
- **裁定**: `REMAP / VERIFIED`。`rangemap 1.7.1`の集合機構は採択し、入力検査・RationalTime変換・generation・Document意味はMotolii側に残す。
- **oracle**: adjacent/empty/end-exclusive/overflow、random model同値、track種別を跨ぐ丸めと
  `RationalTime`丸め二重化0。

#### `P06-C2` affected-window projection

- **結果**: M4-P02-C3がexhaustive Command classifierで既に確定したaffected identity／known-unknownとnode temporal footprintを入力に、half-open影響区間を求めるprivate pure projection helper。`Command` variant match、render relevance、epoch、state publicationを所有しない。
- **依存／並列**: M4-P02-C3 acceptance後。rangemapのpure interval fixtureはP06-C1として独立済みだが、製品projectionはclassifier入力型を推測して先行しない。
- **oracle**: Unknown入力は全区間、無関係node保持、削除／並替／parameter分類corpusから渡されたidentityの区間写像、M4-P02-C3以外の`Command` match 0。
- **cutover**: Salsa DB、汎用dependency framework、全cache purgeを採らない。

#### `P06-C3` generation coverage switch

- **結果**: accepted state envelopeの`InvalidationFootprint`をK1bのtransient coverageへ反映し、invalid rangeだけを欠落へ戻して旧artifact handleを安全に読む。coverage generationはstore-local identityであり、`CacheEpoch`、Document revision、第二state envelopeの別名ではない。
- **依存／並列**: M4-P02-C3 + P06-C2 + K1b acceptance後。snapshot／epoch／footprintのpublishは行わない。
- **oracle**: overlap edit競合、epoch不一致のcurrent-state job拒否、旧coverage handle保持、coverage偽陽性0、第二classifier／epoch publisher 0。

### M4-P07-SCHEDULE — bounded background job

#### `P07-C1` priority collection probe

- **結果**: fixed 4 priority、reprioritize、remove、bounded admissionを`priority-queue`で確認する。
- **薄い残余**: seek時の全item即時reprioritizeを必須にせず、generationを進めてpop時にstale jobを捨てる
  lazy invalidationを既存patternとして使う。
- **実証**: `crates/motolii-testkit/tests/m4_p07_priority_queue.rs`で4 testsが`--locked` green。composite priorityの
  同順位決定性、change/remove/pop、重複item更新、generation／bounded guardの責任分離を確認した。
- **裁定**: `REMAP / VERIFIED`。queueはdata structureとして採択するが、bounded admission、stale generation、cancel、
  worker lifecycle、Document/cache writerはMotolii ownerに残す。raw queueをexecutorへ昇格しない。
- **oracle**: deterministic tie-break、latest seek昇格またはstale drop、cancelled job非実行、queue上限。

#### `P07-C2` bounded worker lifecycle

- **結果**: 既存`LatestWorker`のthread/Condvar/generation/closeを複数jobへ拡張し、queued/running/candidate-ready各段のfailure／cancel、Host-only exactly-once terminal、owner寿命中に再利用しないprivate `JobId`、job／state envelope／recipe／targetへ結合したreceipt、late receipt回収を閉じる。
- **依存／並列**: P07-C1とM4-P02-C3 acceptance後。media/GPU job adapterは別moduleで並列。
- **oracle**: editor thread block 0、shutdown join、panic typed failure、owner寿命中JobId再利用0、receipt binding不一致publish 0、terminal exactly once、queueからDocument write 0。
- **cutover**: Tokio runtime、独自async executor、OS thread priorityを導入しない。

#### `P07-C3` cooperative cancel and heartbeat

- **結果**: media branchはP08-C1の既存kill/cancel oracleへ吸収し、別cancel frameworkを作らない。残るCPU/GPU adapterはtile/数frame境界のtokenとheartbeatを使う。
- **依存／並列**: P07-C2 acceptance後。CPU/GPU cutはP09-C2とK8aを解放し、K7aへはP09-C2経由で推移する。
- **oracle**: cancel latency、GPU command途中強制停止0、stale completion publish 0。

### M4-P08-PROXY — media正規化

#### `P08-C1` proxy recipe and sidecar job

- **結果**: Host-private SourceBindingのexact bytes identity、scale、fps、codec versionをrecipeにし、既存sidecarへjobを追加する。sidecarへraw source locatorを渡さない。
- **依存／並列**: M2-ASSET-1C、M4-P02-CODEC、P05-C2/C3、P07-C2 acceptance後。K4 umbrellaのproduct producer開始点とする。
- **oracle**: 色/LUT/Document parameter混入0、cancel/finish/stderr、atomic artifact commit。sidecar出力はwhole-outputの保守的上限を先に予約するか、Host-owned bounded writerで各chunk前に予約し、cap到達前にchildを停止して未公開tempを回収する。unbounded path出力、事後admission、receipt後writeは0。

#### `P08-C2` VFR to CFR verification

- **結果**: ffprobe PTS/durationを基準にFFmpeg `fps`/`fps_mode`出力を検査する。
- **依存／並列**: P08-C1後。product substitutionとは別oracleで検証する。
- **oracle**: representative VFR、30000/1001、seek frame identity、duration drift上限。

#### `P08-C3` product substitution

- **結果**: proxy hit時だけdecoder inputを置換し、miss/failureはsourceへ戻る。
- **依存／並列**: P08-C1/P08-C2 acceptance後。K4はこのP08-C3 product merge cutそのものであり、別ownerではない。通常product decode routeへの唯一の合流点とする。
- **oracle**: Finalはsource、Offline意味不変、purge不要、path/mtime key 0。

### M4-P09-COPYOUT — GPUからartifactへの非同期境界

#### `P09-C1` copy-out cause-isolation probe

- **結果**: 現行`RgbaDownloader`のheadless同期readback許可／UI共有readback拒否を既存6 fixtureで再確認した。copy/map/encode/diskの各待ちを分離測定し、必要なin-flight数だけを採択する本probeはGAP-29で停止する。
- **再利用target**: `RgbaDownloader`、wgpu staging buffer/map_async。
- **oracle**: UI/render評価chain blocking 0、bounded bytes、device loss、1/2/N buffer比較。固定ring数・重畳方式・性能SLOは計測前に決めない。
- **判定**: `STOP / GAP-29`。既存baselineは同期1-bufferとorigin guardまでで、代表MVのcopy/map/encode/disk原因分離・overlap・backpressure・cancel cleanupを証明しない。

#### `P09-C2` admitted copy-out pipeline

- **結果**: P04 admission後だけcopyし、P05 commitへbounded resultを渡す。
- **依存／並列**: P09-C1の原因分離と方式採択、K1a、P04-C1/C2、P05-C2/C3、P07-C2、M4-P07-C3 acceptance後。GAP-29が残る間は実装しない。
- **oracle**: loop内resource新設0、premul/color変換追加0、cancel後partial publish 0。

### M4-P10-BAKE — Group仮出力

#### `P10-C1` atomic bake artifact

- **結果**: editable Groupの評価結果を一つのrecipe/CAS artifactとしてcommitする。
- **oracle**: 子identity/編集可能性保持、失敗時旧成果保持、Final同値。

#### `P10-C2` interval invalidation

- **結果**: 子変更のaffected windowだけをP06 coverageから欠落させる。
- **oracle**: 無関係区間hit、Unknown全区間、旧generation再利用。

#### `P10-C3` graph substitution and refreeze

- **結果**: hit時だけ内部graphをartifact sourceへ置換し、編集再開で自動的にlive graphへ戻る。
- **oracle**: Documentへhidden canvas/state 0、preview/export同一関数、manual purge不要。

### M4-P11-COVERAGE — 全曲Draft

#### `P11-C1` coverage state and planner

- **結果**: P06 gapsをP07のvisible/playhead-near/forward/background優先度へ変換する。
- **oracle**: seekでreprioritize、bounded queue、完了済み区間の再投入0。

#### `P11-C2` disk-backed playback

- **結果**: coverage hitはP05 artifact、missはlive evaluationを同じclockへ供給する。
- **oracle**: 音声主clock、欠落時停止0、stale generation混入0。

#### `P11-C3` 100GB accounting E2E

- **結果**: sparse/fake storeで100GB相当のaccounting、watermark、eviction、restartを通す。
- **oracle**: 実100GB生成0、hard cap越え0、全曲通し、Final pixel意味不変。

### M4-P12-PRESSURE — preview縮退

#### `P12-C1` capacity controller

- **結果**: VRAM/RAM/disk admission失敗をQuality候補へ型付きで通知する。
- **oracle**: capacityだけでdeadline signalを出さない、Document/Final不変、silent allocation retry 0。

#### `P12-C2` deadline controller

- **結果**: frame deadlineとlatest generationからpreview Qualityを選ぶ。
- **oracle**: memory余裕でも遅延時縮退、容量不足と別telemetry、hysteresis fixture。

#### `P12-C3` resource snapshot provider

- **結果**: M3 HUDへread-only snapshotを渡し、M3側にcache/schedulerを複製しない。
- **oracle**: provider不在はtyped unavailable、UIからpurge/write 0、bounded更新頻度。

### M4-P13-VECTOR — SVGからVello

#### `P13-C1` vello_svg compatibility probe

- **結果**: `vello_svg 0.10.0`（`vello 0.9`／`wgpu` feature）のpath/group/fill/stroke subsetをSceneへ接続する。
- **証拠**: `crates/motolii-testkit/tests/m4_p13_vello_svg.rs`の4 fixtureがgreen。supported Scene、pattern unsupported callback、malformed SVG typed errorを確認し、external file resourceは`usvg`がSceneへ入れずcallbackも呼ばないことを観測した。`cargo check --locked`を`x86_64-pc-windows-gnu`、`aarch64-apple-darwin`、`x86_64-unknown-linux-gnu`で通過させた。
- **判定**: `REMAP / VERIFIED`。K6のpath/group/fill/strokeとunsupported callbackは採用する。外部resource 0はcrate単体の拒否証明にならないため、Motolii import preflightでnetwork/fileをtyped拒否してからcrateへ渡す。premul一回とRenderer長寿命はP13-C1のScene probe外で、K6接続時に別oracleとして保持する。

#### `P13-C2` product import adapter

- **結果**: usvg treeをprivate import成果へ閉じ、rendererの長寿命Vello sceneへ渡す。
- **oracle**: custom parser/walker 0、public usvg型0、preview/export同一、Renderer毎frame生成0。

#### `P13-C3` subset acceptance

- **結果**: K6対応表とgoldenを固定し、未対応text/clip/mask/filter等をsilent dropしない。
- **oracle**: supported corpus同値、unsupported全件診断、golden threshold変更0。

## 6. 並列waveと直列合流

| wave | 並列可能な成果 | 直列点 |
|---|---|---|
| `A: adoption probes` | P03-C1、P04-C4、P05-C1、P06-C1、P07-C1、P13-C1、P09-C1 | Cargo dependency/lock publication、各候補の採否記録 |
| `B: private foundations` | M2-ASSET-1A、K1a、P01-C1、P02 canonical encoder／mutation corpus準備、P04-C1/C3 | M2-ASSET-1Cは1A+K1a後。M4-P02-CODECはstrict SourceFingerprint後。M4-P02-C3はM2-D3/D3e/accepted Command semantics後。P07-C2はM4-P02-C3後。K1bはK1a+codec+M2-D8後。P03-C2はK1a+P03-C1+P04-C1+codec後。P05-C2はK1a+P05-C1+P04-C2+codec後、P05-C3はP05-C2+P04-C2/C4後 |
| `C: product producers` | P06-C2/C3、M4-P03-C3、M4-P07-C3、P12-C1/C2、P13-C2/C3 | P06-C2はM4-P02-C3後、P06-C3はM4-P02-C3+P06-C2+K1b後。M4-P03-C3はM2-ASSET-1C+P03-C2後でK1cと並列、K8bは本cutを待つ。M4-P07-C3のmedia branchはP08-C1へ吸収し、CPU/GPU cutはP07-C2後。P08-C1はM2-ASSET-1C+codec+P05-C2/C3+P07-C2後、P08-C2はP08-C1後。P09-C2はGAP-29のP09-C1採択+K1a+P04-C1/C2+P05-C2/C3+P07-C2+M4-P07-C3後。K1cはK1a+K1b+P03-C2+P04-C2+P05-C2/C3後。K7aはK1b+K1c+M2-D3+P07-C2+P09-C2後。K4はP08-C3そのもので、M2-D1+M2-ASSET-1C+codec+P05-C2/C3+P07-C2+P08-C1/C2後。K8aはK1b+K1c+K1d+K2+M2-D3+P06-C2/C3+P07-C2+M4-P07-C3後。artifact commitとproduct runtime合流 |
| `D: composed outcomes` | P08-C3、P10、P11、P12-C3 | 通常製品route E2E、human/hardware acceptance |

Waveは一括発注ではない。各子を一契約境界として検収し、共有fileへの合流だけownerが直列化する。
GAP-3の残るrelink／offline product routeや`SPEC_ONLY`のGAP-29を別候補で迂回しない。

### P05の検証梯子

`P05-C1 compatibility` → `P05-C2 store model` → `P05-C3 resource integration` → `K7a/K8b product E2E`
の順に閉じる。各段のexact oracleは[disk artifact再検索 §6.1](reviews/2026-08-02-m4-disk-artifact-store-resurvey.md#61-検証の層と完了線)を正とする。前段greenを後段完成へ読み替えず、現状態は検証計画済み・runtime未検証とする。

2026-08-02の検証branchではV1 compatibilityまでを実行し、P05-C2/C3とK7a/K8bは`STOP / RUNTIME ABSENT`へ閉じた。現行M4仕様自身がK1〜K8を未実装と明記し、repo検索でもResourceLedger／disk store／group bake／full Draft producerは存在しない。これらを埋める新ownerや恒久契約は「検証」の範囲を越えるため、製品runtime実装の別branchへ返す。

## 7. 旧負債の処分

| 旧route／誤読 | 新owner | 処分 |
|---|---|---|
| cache別HashMap、LRU、参照registry | P03 RAM cache | 同一oracle後に`FROZEN → RETIRE` |
| 独自disk DB/WAL/catalog／global dedup | P05 verified recipe artifact adapter | 新設禁止。既存断片があれば単一owner切替後retire |
| purgeを通常回復にするUI | complete key + miss policy | debug診断以外へ昇格しない |
| generationごとの全走査削除 | immutable key + P06 ranges | writer publish切替後retire |
| component別worker/runtime | P07 bounded worker | job adapter化後retire |
| custom SVG parser/walker | P13 vello_svg | 新設禁止 |
| 現行PipelineCache/target poolをM4完成と数える | 各現ownerの局所reuse | 名称だけを根拠に昇格しない |

## 8. route再開条件

採択routeを再び開けるのは、固定versionのlicense不適合、対象OS build不能、必須failure oracle不成立、
hard capを構造上守れない、外部型が恒久契約へ漏れる、maintenance責任が独自薄adapterより増える、のいずれかを
probeで再現した場合だけである。その場合も親の利用者成果とoracleは維持し、`REMAP / REDUCE`を先に行う。
既完了コード、投入工数、外部LLMの好み、候補の知名度は再裁定理由にしない。
