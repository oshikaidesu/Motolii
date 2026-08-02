# M4 disk artifact store再検索（2026-08-02）

状態: **縮小採用**

対象: [M4既知実装調査](2026-08-02-m4-known-implementation-survey.md)、
[M4既知実装採択・並列実装地図](../m4-known-implementation-adoption-map.md)、
[M4仕様](../specs/M4-cache-and-analysis.md)

## 1. 問題提起

初版地図は、完全recipe keyと再起動後reuseをBazel型のaction cache／CASへ写した後、disk実装候補を
`cacache 13.1.0`へ割り当てた。しかし一次資料再照合で、同crateはruntime featureなしでcompile不能、
Windows write-heavy破損が未解決、maintainerがdormantとfork保守を明言していることが分かった。

ここで別のCAS crateを機械的に選ぶ前に、現行authorityが本当に要求する成果を再確認した。

## 2. authorityが要求する成果

M4 K1b/K1c/K7/K8が要求するdisk側の成果は次である。

1. 完全recipe keyからartifactを再起動後もlookupできる。
2. 同一filesystem内の一意tempへ書き、検証後のrenameでだけ可視化する。
3. 欠落、partial、hash不一致、読取失敗をcache missへ写す。
4. Host single writer、allocation前hard budget、watermark、LRU、pinを守る。
5. proxy等はFFmpegが直接開ける通常fileとして保持できる。
6. cache削除時にDocument、Undo、Project、Final意味が変わらない。

現行正本は、異なるrecipeが偶然同じbytesを出した時のglobal content dedup、remote CAS protocol、永続DB、
cache format migrationを要求していない。したがって**disk CASは利用者成果でなく、初版地図が追加した
機構強度**である。これを維持するには、その追加責任を上回る実測利益が必要である。

## 3. 候補比較

| 候補 | version／状態 | 成立するもの | 増える責任／非成立 | 裁定 |
|---|---|---|---|---|
| `cacache` | 13.1.0、Apache-2.0、dormant | key index、content store、integrity、atomic write | fork保守、runtime feature workaround、Windows破損、古い依存、GCなし | `REJECT` |
| `syntheca + apotheca` | 0.3.2 + 0.3.3、MIT OR Apache-2.0、2026-05初出 | streaming CAS、SHA-256 integrity、write-once local store | 実績0に近い二crate、独自protocol/pinax/meta、raw artifact path APIなし | `REJECT` |
| `casq_core` | 0.12.0、Apache-2.0、2026-01初出 | BLAKE3、temp write、verify、GC | backup向けtree/chunk/zstd/journal/refs format、single-thread、materialize copy | `REJECT` |
| `cassadilia` | 0.4.7、MIT OR Apache-2.0、MSRV 1.89 | large blob CAS、root path | 独自WAL/index/serialization/orphan recoveryを恒久所有 | `REJECT` |
| `object_store` local | 0.14.1、MIT/Apache-2.0、Apache管理 | activeなlocal object API | async/Tokio/cloud abstraction、CAS/integrity/LRUでない、raw path契約でない | `REJECT` |
| `cache-manager` | 0.4.1、MIT OR Apache-2.0 | managed directory、mtime順eviction | best-effort削除でhard capを閉じず、atomic/integrity/CASなし | `REJECT` |
| `sccache` local cache | active、Apache-2.0 | sharded key path、same-dir temp、size LRU、single-server、lazy initの出荷先例 | library依存はTokio/build cache全体を持込む。content integrityは別 | `PATTERN` |
| `tempfile` | 3.27.0、MIT OR Apache-2.0、MSRV 1.63 | unique same-dir temp、cross-platform persist、drop cleanup | `persist_noclobber`は全platform atomic保証でない。single writer前提が必要 | `ADOPT-PROBE` |
| workspace `sha2` + `std::fs` | 0.10 + standard library | content verification、通常file、direct path、依存追加最小 | recipe path、LRU metadata、typed missの薄いadapterはMotolii側 | `REUSE` |
| D1 atomic persist | 現行`motolii-doc::persist` | unique temp→file sync→replace→dir syncとfault fixtureのrepo内先例 | private Document writerをcache APIへ昇格してはならない | `PATTERN` |

### 3.1 新規CAS候補が勝たない理由

検索で見つかった専用CASは、いずれも2025〜2026年開始で利用実績が小さいか、backup／container／WASM／
database向けのformat、compression、chunking、WAL、HMAC、remote backendを同時に所有する。Motoliiの
再生成可能cacheへ必要な責任より大きい。`cacache`の代わりに若いcrateの保守riskを移すだけでは、既知解へ
責任を渡したことにならない。

### 3.2 成立した既知route

採るrouteは新しいCAS frameworkではなく、既知のlocal artifact cache patternの合成である。

- Bazel action cacheの`recipe key → artifact`分離を`PATTERN`として維持する。
- sccacheのsharded key path、single owner、startup scan、size LRUを`PATTERN`として使う。
- workspace `sha2`で公開前と読出時にartifact bytesを検証する。
- `tempfile 3.27.0`のsame-directory temp／persistを`ADOPT-PROBE`し、cache ownerをsingle writerに保つ。
- `fs4 1.1.0`をfree-space／watermarkだけに使う。
- proxyは通常file、GPU copy-outは採択codecの通常fileまたはsegmentとして同じprivate storeへ置く。

これは`REUSE / PATTERN / ADOPT-PROBE`であり、汎用cache protocolや永続DBを新設する`BUILD`ではない。

### 3.3 映像編集製品で成立済みの妥協範囲

専用CAS crateが勝たないことは、disk cacheそのものを独自frameworkとして発明する理由にならない。出荷済みの
映像編集製品は、global content dedupより、再生成できるframe／proxy／render mediaを製品固有keyで通常fileへ
保存し、無効化または利用者操作で捨てるrouteを広く採っている。

| 先例 | 公開資料から確認できる成立範囲 | Motoliiへ転移しないもの |
|---|---|---|
| Blender VSE | RAM cacheのdisk拡張、保存先、容量上限、圧縮。導入実装は100 frame単位fileとsize超過削除 | GPL実装、100-frame format、過剰無効化、Blender identity |
| After Effects | sessionを跨ぐrendered-frame cache、起動時scan、最大容量、空き容量reserve、明示purge | 非公開key／format、Adobe database、Final適格性の推測 |
| Premiere Pro | `.cfa`／`.pek`等の派生fileと対応database、削除後の再生成、age／size cleanup | 形式、共有database、後追いcleanupをhard admissionの代用にしない |
| DaVinci Resolve | persistentなRender Cache／Optimized Media、通常codec、変更後の再cache、明示削除 | proprietary format、Deliverへのcache利用policy |
| Final Cut Pro | libraryまたは外部保存先のrender／proxy／optimized media、unused／all削除、originalから再生成 | library format、macOS専用storage contract |

これらは「通常fileの再生成cacheで製品価値が成立する」ことだけを証明する。partial publication 0、content
integrity、allocation前hard budget、Windows atomic visibility、Motoliiの完全recipe keyは証明しない。その差だけを
`P05-C1`以降のfixtureで閉じる。Rerunのchunk store／query／viewer persistenceはこの成果のownerではなく、
M4-P05の採択routeへ入れない。

## 4. REDUCE後のprivate store境界

### 4.1 identity

disk entryのidentityは完全recipe keyのdigestである。raw path、mtime、表示名、cache warmthをidentityへ
入れない。artifactのcontent digestはintegrity用であり、初期routeでは別recipe間のglobal dedupを約束しない。
同一recipeは同一pathへ収束するため、通常の再利用は失われない。

### 4.2 publication

1. Host cache writerがhard budgetとwatermarkをadmitする。
2. finalと同一directory／filesystemの一意tempへproducerが書く。
3. producer finish後にsize、content digest、必要なcodec probeを検証する。
4. single writerがtempをfinal recipe pathへpersistする。
5. publish後だけRAM index／coverageへmessageを反映する。

process kill、ENOSPC、producer failureでtempが残ってもentryとして列挙しない。cacheはDocumentほどのdurabilityを
要求せず、renameやdir syncが失われた場合は次回missで再生成する。

### 4.3 lookupとretirement

- startup scanはbackground/lazyに行い、editor entryを待たせない。
- publish前と再起動後の初回admissionでsize/content digestを検証し、不一致はmiss＋retirement候補とする。
  検証済みhandleをprocess内で再利用し、frameごとに全fileを再hashしない。
- mtime／volatile access metadataはeviction順だけに使い、identityへ使わない。
- hard budget超過前にLRU候補を同期選定し、pin／open中entryは飛ばす。
- 削除失敗はbudgetから消したことにせず、別候補またはtyped refusalへ進む。

## 5. 音MADとglobal content dedup

同一素材区間を多数sliceする負荷は、RAM側の`recipe → content digest → buffer`二段参照で先に閉じる。
disk側で異なるrecipe間のbytes重複が製品budgetを実際に支配する証拠はまだない。初期routeへhard-link、
content refcount、GC livesetを追加しない。fixtureで重複率とdisk bytesが支配的だと反証された場合だけ、同一
filesystem hard-linkまたはcontent blob層を独立`PATTERN`比較する。これは現在の完成条件ではない。

## 6. 採択probe

`P05-C1`はCAS crate比較でなく、次のfilesystem artifact fixtureを閉じる。

1. `tempfile 3.27.0` exact dependency、license、Mac／Windows build。
2. Rust writerとFFmpeg producerのsame-directory temp→finish→verify→persist。
3. process kill、ENOSPC、bit flip、truncate、missing file、stale tempをすべてmissへ写す。
4. 同一recipe同時完了をsingle cache writerが直列publishし、二重owner／二重finalを作らない。
5. 1GB fake budgetとsparse fixtureでbudget＋epsilonを超えない。
6. 10万entry相当のlazy scan／shardingを実file全生成なしで計測する。
7. external crate型、path、mtimeをDocument／公開API／serde／journalへ漏らさない。

### 6.1 検証の層と完了線

先例の存在はprobe合格ではなく、`P05-C1`の合格も製品cache完成ではない。2026-08-02のV1 fixtureはsame-dir publish／FFmpeg temp／digest／stale tempを閉じたが、process kill／ENOSPC／同一recipe競合／Windows runtime visibilityは未閉鎖のままである。repoにはP05-C2/C3やK7a/K8bのproduct producerも存在しないため、これらは`STOP / RUNTIME ABSENT`として別実装branchへ返す。検証を次の四層に分ける。

| 層 | 対象 | 必須oracle | 合格後に許すもの |
|---|---|---|---|
| `V1 compatibility` | P05-C1 isolated fixture | Mac／Windows build、Rust／FFmpeg通常file、same-dir publish、kill／ENOSPC／bit-flip／truncate／missing／stale temp、同一recipe競合、partial final 0 | `tempfile`採択とprivate adapter起票 |
| `V2 store model` | P05-C2 unit／model test | restart hit、wrong generation miss、検証済みhandle再利用、shard衝突0、公開型漏出0 | recipe file storeをK1b/K1cへ接続 |
| `V3 resource integration` | P05-C3 + P04-C2 stress | allocation前hard admission、1GB sparse/fake budget `+ epsilon`以内、pin／open／committing削除0、削除失敗を未回収計上、watermark refusal | disk tierの製品route接続 |
| `V4 product E2E` | K7a／K8b | process kill後restart、破損cacheから透明再生成、FFmpeg実読込、cache有無のFinal bit一致、全曲Draft coverage、実file 100GB生成0 | 旧disk routeの`FROZEN → RETIRE` |

`V1`〜`V4`は期待値更新、purge後だけ成功する試験、mock codec、実fileの巨大生成で代用しない。現時点は
**検証計画済み・runtime未検証**であり、先行製品の存在やdocs greenをP05完成証拠に数えない。

## 7. STOPと再入場

- `tempfile::persist`が対象Windows filesystemで可視性atomicを満たさない場合は、そのplatformのpublication
  primitiveだけを`REMAP`する。CAS frameworkやDBへ飛ばない。
- single writerではproducer throughputを満たせない実測が出た場合だけ、commit queueの並列化を比較する。
- global content dedupは重複率とbudget効果を実fixtureで示した場合だけ再入場する。
- cache formatのmigration、repair、remote protocol、fork保守が必要になった候補は採らない。

## 8. 一次資料

- [Bazel Remote Caching](https://bazel.build/remote/caching)
- [sccache local disk cache source](https://github.com/mozilla/sccache/blob/main/src/lru_disk_cache/mod.rs)、
  [local cache documentation](https://github.com/mozilla/sccache/blob/main/docs/Local.md)
- [`tempfile 3.27.0`](https://docs.rs/tempfile/3.27.0/tempfile/)
- [`fs4 1.1.0`](https://docs.rs/fs4/1.1.0/fs4/)
- [`syntheca 0.3.2`](https://docs.rs/syntheca/0.3.2/syntheca/)、
  [`apotheca 0.3.3`](https://docs.rs/apotheca/0.3.3/apotheca/)
- [`casq_core 0.12.0`](https://docs.rs/casq_core/0.12.0/casq_core/)
- [`cassadilia 0.4.7`](https://docs.rs/cassadilia/0.4.7/cassadilia/)
- [`object_store 0.14.1`](https://docs.rs/object_store/0.14.1/object_store/)
- [Motolii D1 atomic persist](../../crates/motolii-doc/src/persist.rs)
- [Blender VSE performance／disk cache](https://docs.blender.org/manual/en/4.5/editors/video_sequencer/introduction.html)、
  [VSE disk cache導入記録](https://www.mail-archive.com/bf-blender-cvs%40blender.org/msg123543.html)
- [After Effects memory and storage](https://helpx.adobe.com/after-effects/desktop/memory-storage-performance/memory-and-storage/memory-storage1.html)
- [Premiere media cache](https://helpx.adobe.com/premiere/desktop/troubleshooting/media-issues/manage-media-cache.html)
- [DaVinci Resolve reference manual](https://documents.blackmagicdesign.com/UserManuals/DaVinci_Resolve_15_Reference_Manual.pdf)
- [Final Cut Pro render files](https://support.apple.com/guide/final-cut-pro/manage-render-files-ver68a8c250/mac)
