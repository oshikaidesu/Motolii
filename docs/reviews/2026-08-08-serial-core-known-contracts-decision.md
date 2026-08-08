# M3/M4/M5直列核4契約の既知実装採択決定

状態: **決定**（2026-08-08）

M3の通常製品routeからM4のcache／job、将来のM5 importer／rendererへ進む時に共有する直列核を、次の4契約へ閉じる。

1. Assetのactionとlifecycle
2. source identityとderived artifact recipe
3. resource admission、artifact publication、job終端
4. Document mutationからcache invalidationへのpublication

これはruntime完成報告ではない。既存のDocument single writer、D2 Command、M4 hard budget、M5休止線を使って実装境界を固定するdecisionであり、製品runtime、公開plugin API、永続package format、M5意味開放を同時に開始しない。

## 1. 結論

| 契約 | owner | 採択する意味 | 実装へ残すもの |
|---|---|---|---|
| Asset action/lifecycle | Document edit thread + D2 Command + `AssetTable` | `AdmitAsset`、既存Assetのuse作成、unused Assetの`RemoveAsset`を別actionにする。stable `AssetId`は内容重複排除keyではない | Command variant、table-local `peek_next`／inverse-only `restore`、全typed useの参照検査、product import adapter |
| source identity/recipe | Host-private canonical codec | source bytesの`sha256`+size、versioned canonical recipe、成果物bytes digestを別identityにする | strict decoder、canonical encoder、mutation oracle、既存legacy Assetの移行扱い |
| resource/artifact/job | Hostの単一admission policy owner + artifact/catalog owner | tier横断hard cap、tier別store、workerは候補をmessageで返し、検証済み成果物だけをHost ownerがatomic publishする。cancelはcooperative | K1a ResourceLedger、tier adapter、job runner、task固有artifact validator |
| mutation/invalidation publication | Document edit runtimeがaccepted publish後に呼ぶHost-private classifier + M4 cache/session owner | 完全recipe keyを正しさ、invalidation footprintを効率化とする。未知は全compositionへ倒し、private `CacheEpoch`でlate resultを拒否する | exhaustive Command classifier、依存区間、cache epoch fence、K1b/K2接続 |

4契約の共通不変条件は次である。

- Documentを書き換えるのは編集threadだけであり、importer、cache、job、M5 runtimeは第二writerにならない。
- `AssetId != locator != SourceFingerprintV1 != RecipeKeyV1 != ArtifactDigest != JobId` とする。
- background workの完了はDocument actionの成功条件にせず、cache miss、失敗、cancel後もeditorを操作できる。
- Preview／Exportは同じ評価意味を使い、cacheやDraftの有無でFinalの意味を変えない。
- 既存projectのopaque／legacy hashを、検証済みcontent identityへ黙って昇格しない。

## 2. 既知実装preflight

- **MECHANISM CLASS**: undoable object lifecycle、content descriptor、action-result cache、hard-budget admission、atomic file publication、cooperative job cancellation、dependency invalidation。
- **KNOWN IMPLEMENTATION SEARCH**: repoの`AssetTable::insert`／`remove`、`LayerIdTable::peek_next`／`restore`、D2 `AddTrackItem`↔`RemoveTrackItem` inverse、`DocumentEditRuntime`、M3 `projection_generation`、private `RenderGeneration`、K1a〜K4、[OCI content descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)、[Bazel remote cache](https://bazel.build/versions/7.1.0/remote/caching)、[Qt Undo Framework](https://doc.qt.io/qt-6/qundo.html)、[Blender Data-Blocks](https://docs.blender.org/manual/en/5.0/files/data_blocks.html)、[`CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)、[`tempfile::NamedTempFile`](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html)を照合した。
- **CANDIDATES**: repo既存のtable-local `peek_next`／inverse-only `restore`とD2 inverse、global `StableIdReservation`、OCI型digest+size、Bazel型recipe→result metadataとartifact bytesの分離、Qt型macro、Blender型unlink／orphan、Tokio型cooperative cancellation、same-directory temp→verify→persist。
- **ADOPTION ROUTE**: Assetは`LayerIdTable`とD2 `AddTrackItem`のtable-local復元先例を`REUSE/PATTERN`し、global `Document.next_stable_id`／`StableIdReservation`へ混ぜない。workspace `sha2`を`REUSE`、OCIとBazelのidentity分離を`PATTERN`、hard-budget K1a契約を`REUSE`、Tokio cancellationと`tempfile` publicationを`WRAP/PATTERN`とする。
- **REJECTED CANDIDATES**: path／mtime identity、head/tail hash authority、AssetIdによるcontent dedup、全tier共通store、global CAS必須化、universal public Job型、完了eventの永続event bus、通常修復としての全cache purge、M5 private型からの契約逆算。
- **THIN MOTOLII SEAM**: Motolii固有なのはD2 action、render-relevant recipe fields、resource owner名、Commandごとのdependency footprint、task固有artifact検証だけである。
- **THIN MOTOLII RESIDUAL**: SourceFingerprint／RecipeKey codec、Asset Command、ResourceLedger、invalidation classifierの小さいHost-private実装。
- **RETIREMENT**: legacy `head_hash`／`tail_hash`はidentity authorityから外す。raw `content_hash`は読み取り互換のため保持し、新しいpersistent cache／relink authorityだけをstrict V1へ移す。移行完了前にfieldを削除しない。
- **BUILD JUSTIFICATION**: `NONE`。新規一般frameworkは不要で、既存ownerへ薄い製品意味を接続する。
- **BUILD: FORBIDDEN**: 独自CAS／DB／WAL、第二writer、Asset manager framework、汎用scheduler、公開event bus、M5用先行resource API。

## 3. 契約A — Asset actionとlifecycle

### 3.1 三つのactionを分離する

Assetの取り込み、作品内での使用、台帳からの除去は一つの暗黙transactionにしない。

1. **probe**: workerがfileを読み、型、full content digest、size、必要なmedia metadataを求める。DocumentとIDを変更しない。
2. **admit**: edit threadが`AssetTable::peek_next`と同型のtable-local値から`AssetId`を決定し、完全な`Asset`を一つのreplayable `AdmitAsset` Commandとしてjournal／applyする。probeはIDを予約しない。
3. **use**: admit成功で公開された`AssetId`を、既存のplacement／Soundtrack／typed `AssetRef` actionが使う。
4. **remove**: 全typed useが0の時だけ、完全な`Asset`値を保持する`RemoveAsset` Commandで台帳から外す。

file dropで「importしてTimelineへ置く」通常操作は、admitとplacementの二つのaccepted durable actionである。一回目のUndoはplacementだけを戻してunused Assetを残し、二回目のUndoがunused Assetを除去する。二action間のcrash／失敗は、参照切れや部分placementでなく、再利用または後でpurgeできるunused Assetを残す。

現行product durable routeは一accepted actionを一replayable Commandへ閉じているため、Qt型macroをこの境界の必須条件にしない。将来、一gesture一Undoのproduct要件が実測された場合は、durable macro自体を別decision／journal migrationとして閉じる。

### 3.2 ID、Undo、Redo

- `AssetId`はproject object identityであり、同じbytesを別名・別用途で二度admitしても別IDを許す。
- IDはworker、file path、hashから導出しない。edit threadが`AssetId::from_raw(doc.assets.peek_next())`と同型のtable-local採番値をCommandへ記録する。`AssetId`は`Document.next_stable_id`やglobal `StableIdReservation`の空間へ入れない。
- `AdmitAsset { asset }`と`RemoveAsset { asset }`をD2の相互inverseとする。`AssetTable`へ`peek_next`とinverse-only `restore`を追加し、initial applyとRedo／Undo復元はいずれも`restore`を使う。`restore`はduplicateを拒否し、`id < next`の同一entity復元を許し、必要なら`next`を進めるが巻き戻さない。通常の`insert`は退役IDを引き続き拒否する。
- Commandはcomplete `Asset`値、duplicate、採番closure、全typed useを事前検証し、現行D2と同じclone→apply→Document validate→swapのatomic routeへ載せる。`RemoveAsset`は参照0を確認してから外す。現行の最低参照集合はClip source、Soundtrack、`DocValue::AssetRef`、SVG Asset、Text Pathのfont Assetであり、新しいtyped Asset参照を追加した時は同じ参照検査をexhaustiveに更新する。
- crash recoveryはjournal済みCommandから最終Document意味を戻す。再起動後のUndo stack復元は現行契約外のままとする。

### 3.3 unlink、relink、replace、purge

- placement／Soundtrack／parameterからAsset useを外す操作は、Assetそのものの削除ではない。
- relinkは候補pathのbytesが保存済み`SourceFingerprintV1`と完全一致した時だけ、同じ`AssetId`のlocatorを更新する。locator変更だけではrender keyを変えない。ただしD2 Command variant、old/new locator、offline時failureのexact形はGAP-3の別cutで閉じ、1Aへ混ぜない。
- 異なるbytesへの差し替えはsilent relinkでなく、将来の明示的な`Replace Source` actionである。fingerprintを変え、全依存useを無効化する。
- `Purge Unused`は未参照Assetを列挙・確認して複数除去する将来のmaintenance actionである。今回、durable macroや自動purgeを作らない。
- source fileはユーザーまたは外部filesystem所有であり、Asset除去やcache purgeで原本を削除しない。

## 4. 契約B — source identityとderived artifact recipe

### 4.1 SourceFingerprintV1

新しくadmitするfile-backed Assetのcontent identityは、次の両方が揃った時だけ`SourceFingerprintV1`として有効とする。

- `content_hash = "sha256:<64 lowercase hex>"`
- `size_bytes = Some(<source file全byte長>)`

hash対象はsource fileのexact bytesであり、decode後のpixel、container正規化、path、mtime、file nameではない。streaming SHA-256で全byteを読む。digestとsizeの両方をrelink／persistent cache hit前に照合する。

`SourceFingerprintV1`のV1はdecoderと意味のversionである。digest文字列のalgorithm tagは将来拡張を許すが、v1 Hostが知らないalgorithmは保持のみ可能で、検証、relink authority、persistent cache keyへ使わない。`head_hash`／`tail_hash`はlegacy hintとして保持できるが、identity、collision fallback、cache hitのauthorityにしない。

既存fixtureや旧projectにある短い`sha256:*`、任意文字列、size欠落は読み取り互換上のlegacy opaque valueである。明示的な再hash／確認が終わるまで、persistent cacheはmiss、automatic relinkは不一致として扱う。

### 4.2 RecipeKeyV1

`RecipeKeyV1`はHost-private canonical encoderが、出力を変え得る意味だけを型付き・domain separated・versioned bytesへencodeし、そのbytesをSHA-256した値である。値はtag+length-prefixで連結し、整数幅／endianness、float canonicalization、string bytes、collection順を固定する。順不同mapはstable typed keyでsortし、意味またはencodingを変える時はrecipe format versionを上げる。

各recipeは対象artifactに必要な次を含む。

- render graph／node／pluginのidentity、semantic version、実行内容hash
- typed parameter値と、入力Assetの`SourceFingerprintV1`
- 意味を持つ入力順、対象時刻またはhalf-open区間、`Quality`
- `FrameDesc`、color／alpha／sample／cache artifact format
- renderer、decoder、toolchain等、出力差が実測または仕様上あり得るenvironment salt

次は含めない。

- `AssetId`、path、file name、表示名
- UI route、選択、panel状態、JobId
- Document revision／`CacheEpoch`／`projection_generation`／`RenderGeneration`そのもの
- 出力を変えないmetadataや、無関係nodeの状態

完全keyは「同じkeyなら同じ意味のartifactを要求している」を保証する。`ArtifactDigest`はpublish対象のactual bytesを別にSHA-256したintegrity identityである。`RecipeKeyV1 -> verified result metadata + ArtifactDigest`とartifact bytesの分離だけをBazelから採り、global CAS、cross-project dedup、remote cacheは要求しない。

## 5. 契約C — resource、artifact、job

### 5.1 hard-budget admission

ResourceLedgerはHost内で一つのadmission policy ownerを持ち、VRAM、RAM、disk、shared memoryのhard capをallocation／write開始前に判定する。store、allocator、eviction、metric、lifetimeはtier別adapterに保ち、全resourceを一つの巨大cache／providerへ統合しない。

admission permitは最低でもowner、tier、上限bytes、purpose、resident／pinned区分を持つ。permitなしに管理対象resourceを生成せず、実allocationがpermit上限を越える場合は追加admission成功前に利用せず、失敗なら生成物を公開せず解放する。drop／明示releaseでaccountingを戻す。unified memoryではRAMとVRAMの個別表示に加えshared aggregate capを守る。backend reportは診断照合であり、Host hard capの代替ではない。

区分は次の通りである。

- source file: 外部所有。cache budget外であり、Motoliiが自動削除しない。
- resident working set: GPU texture／buffer、decode surface、RAM working data。permitとlifetimeを必須にする。
- reconstructible artifact: proxy、analysis、bake、Draft、frame cache。hard-budgeted、evict可能、欠落／破損はmiss→再計算。
- durable user export: 明示destinationへcommitするユーザー成果。cache eviction対象外で、失敗／cancelをユーザーへ返す。
- transient preview／in-flight result: generation fence対象。stale完了はpublishせず破棄する。

### 5.2 artifact publication

reconstructible artifactとexportは、完成前のfileを有効成果物として公開しない。workerはadmission済みのsame-directory tempへ書き、task固有validatorでsize、decodeability、frame／duration等を確認し、`ArtifactDigest`を添えたcandidate receiptをmessageで返すだけとする。Hostのartifact／catalog ownerがcancel、`CacheEpoch`、admission、競合を再照合し、platform adapterのatomic persist／replaceを行い、その後だけcatalogへ登録する。workerはcache catalogにもDocumentにも直接書かず、既存destinationを先に削除しない。old-final保持を保証できないplatform／filesystemでは成功扱いにせず、typed failureとして旧成果を維持する。

失敗、cancel、process kill、ENOSPC、validator不一致、競合loserは既存のlast-good artifactを置換しない。stale tempは再起動時に未公開物として回収できる。cache catalog entryの欠落、file欠落、digest不一致、decode失敗はすべてmissへ縮退する。

### 5.3 job終端

共通化するのは語彙と不変条件だけで、全taskを一つの公開`Job`型へ押し込まない。

`queued -> running -> candidate-ready(receipt) -> succeeded(published receipt) | failed(typed cause) | cancelled`

- cancelはcooperativeで、boundedなtile／frame／chunk境界にcheckpointを置く。
- worker completionはcandidate-readyにすぎず、Host ownerのpublication成功だけをjob successとする。failed／cancelledはpartial resultを有効化しない。
- M4 cache/session ownerは非永続のprivate `CacheEpoch(u64)`を所有し、accepted durable Document Commandのsnapshot publishごとに進める。jobは開始時のimmutable snapshotとepochを保持し、Host ownerはcandidate publish時のepoch不一致をstaleとして破棄する。
- `CacheEpoch`はUI選択でも進み得るM3 `DocumentEditRuntime.projection_generation`、render要求順のprivate `RenderGeneration`、journal generation、Document revisionの別名ではない。selection-only publishはcache epochを進めず、値をrecipeやDocumentへserializeしない。
- editor、Preview、通常操作はproxy／analysis／cache jobを待たない。
- capacity pressureとrender deadline超過は別signal、別policyにする。
- durable exportのfailure／cancelと、透明にmissへ戻せるcache jobのfailureを同じUXへ潰さない。

## 6. 契約D — Document mutationからcache invalidationへのpublication

cache correctnessは二層に分ける。

1. **完全な`RecipeKeyV1`**: stale artifactへhitしないための正しさ。
2. **InvalidationFootprint**: 不要な再計算を減らす効率化。過大無効化は許せるが過小無効化は許さない。

accepted Commandがjournal／applyされ、新しいimmutable snapshotがpublishされた後、Document edit runtimeはそのCommandをHost-privateなexhaustive classifierへ渡す。classifierは最低でもrender relevance、stable affected identity、half-open affected interval、known／unknownを返す。公開event bus、serialized invalidation event、第二writerは作らない。

- 全`Command` variantを明示matchし、新variant追加時は分類を追加しない限りcompileまたはtestをfailさせる。
- dependencyや時間窓が不明、将来variant、classifier errorの場合はwhole-composition invalidationへ倒す。
- pixelを変え得るmutationは、affected recipeまたはそのtyped dependency identityを必ず変える。
- display name、同一fingerprintへのlocator更新等のmetadata-only mutationはrecipeを変えない。
- unused Assetのadmit／removeはrender invalidation不要。Asset useの追加／削除は配置区間、source replacementは全依存区間を無効化する。
- accepted durable Commandのsnapshot publishごとにM4 cache/session ownerの`CacheEpoch`を進め、旧snapshotで走るjobの完了を新catalogへpublishしない。旧snapshot readerは保持済みhandleを読み切ってよい。`CacheEpoch`はpersistent recipe keyへ含めない。
- 通常のcorrectness回復として全cache purgeを使わない。unknown fallbackとしての全composition再計算と、disk全削除は別操作である。

## 7. 失敗しても捨て直しにならない分割

4契約は一つの巨大sandbox実装にしない。各cutは前cutのpublic／internal contractだけを読み、独立oracleで採否できる。

| 順序 | closed order | primary oracle | 並列解放 |
|---|---|---|---|
| 1A | M2 Asset: strict fingerprint codec、`AdmitAsset`／`RemoveAsset`、table-local `peek_next`／inverse-only `restore`／typed use count | roundtrip、legacy拒否、atomic apply、Undo remove→Redo same ID、通常`insert`のretired拒否、inverse `restore`のretired同一entity許可、参照中remove拒否 | media／Soundtrack import adapterとM4 identity fixture |
| 1B | M4 K1a: thin ResourceLedger policy ownerと注入budget | cap不超過、全pin typed refusal、release後0、shared aggregate | tier別GPU／RAM／disk adapter |
| 2 | M4 P02: canonical RecipeKey／ArtifactDigest codec | mutation corpus、order／float encoding、rename不変、content／plugin／Quality変化 | artifact store、proxy／analysis recipe |
| 3 | M4 K1b/K2: exhaustive invalidation classifierとgeneration fence | 全Command分類、unknown全域、影響区間、stale completion拒否 | cache store、K7/K8、将来M5 consumer |
| 4 | task別job／atomic artifact adapter | cancel／kill／ENOSPC／corrupt／raceでlast-good保持 | K4 proxy、K7 bake、analysis、export adapter |

1Aと1Bはownerとfile allowlistを分けて直ちに並列化できる。P02 recipe codecは1Aのstrict fingerprint typeを入力に使うが、1Aのallowlistと重ならないcanonical encoder／mutation corpusの準備は並行できる。tier adapterはK1a policy acceptance後、task別artifact adapterはrecipe／publication contractの各acceptance後に並列化できる。

M5はこの分割のconsumerだが、[M5休止・M3意味開放契約](2026-08-02-m5-pause-until-m3-semantic-release.md)を維持する。M5 private fixtureは契約oracleの候補になれても、M3意味開放前に製品runtime、schema、provider、GPU resource接続を開始しない。

## 8. acceptanceと停止線

このdecisionの完了は、4契約と順序を正本へ回収したことだけを意味する。次は含まない。

- Asset Command、ResourceLedger、RecipeKey、cache store、job runnerの実装完了
- M4 K1a／K1b／K2／K4のacceptanceまたはmain統合
- M5製品runtimeの解放
- global CAS、remote cache、cross-project dedup、package／install identity
- `Replace Source`、一括relink、Purge Unused UI、durable macro

各実装cutは、exact owner／allowlist／positive and negative oracleをcurrent mainから再compileして一ticket一commitで発注する。既存型またはconsumerが不足する場合、そのedgeだけを`RESEARCH_RETURN`し、private substituteを恒久契約へ昇格させない。
