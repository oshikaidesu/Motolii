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
| source identity/recipe | Host-private canonical codec + SourceBinding owner | provenance tag付きsource bytes fingerprint、exact consumed bytes binding、versioned canonical recipe、成果物bytes digestを別identityにする | strict decoder、budgeted immutable binding、canonical encoder、mutation oracle、既存legacy Assetの移行扱い |
| resource/artifact/job | Hostの単一admission policy owner + artifact/catalog owner | tier横断hard cap、tier別store、workerは候補をmessageで返し、検証済み成果物だけをHost ownerがatomic publishする。cancelはcooperative | K1a ResourceLedger、tier adapter、job runner、task固有artifact validator |
| mutation/invalidation publication | `M4-P02-C3`（Document edit thread上のaccepted Command classifier + M4 cache/session state envelope） | 完全recipe keyを正しさ、invalidation footprintを効率化とする。未知は全compositionへ倒し、immutable snapshot／private `CacheEpoch`／footprintを一組でpublishする | exhaustive Command classifier、依存区間、atomic state envelope、K1b/K2接続 |

4契約の共通不変条件は次である。

- Documentを書き換えるのは編集threadだけであり、importer、cache、job、M5 runtimeは第二writerにならない。
- `AssetId != locator != SourceFingerprintV1 != RecipeKeyV1 != ArtifactDigest != JobId` とする。
- `JobId`はHost process/session内の実行identityであり、recipe、artifact identity、永続Document IDへ流用しない。
- background workの完了はDocument actionの成功条件にせず、cache miss、失敗、cancel後もeditorを操作できる。
- Preview／Exportは同じ評価意味を使い、cacheやDraftの有無でFinalの意味を変えない。
- 既存projectのopaque／legacy hashを、検証済みcontent identityへ黙って昇格しない。

## 2. 既知実装preflight

- **MECHANISM CLASS**: undoable object lifecycle、content descriptor、action-result cache、hard-budget admission、atomic file publication、cooperative job cancellation、dependency invalidation。
- **KNOWN IMPLEMENTATION SEARCH**: repoの`AssetTable::insert`／`remove`、`LayerIdTable::peek_next`／`restore`、D2 `AddTrackItem`↔`RemoveTrackItem` inverse、`DocumentEditRuntime`、`persist.rs`のunique temp→file sync→replace→directory sync、M3 `projection_generation`、private `RenderGeneration`、K1a〜K4、[OCI content descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)、[Bazel remote cache](https://bazel.build/versions/7.1.0/remote/caching)、[Qt Undo Framework](https://doc.qt.io/qt-6/qundo.html)、[Blender Data-Blocks](https://docs.blender.org/manual/en/5.0/files/data_blocks.html)、[`CancellationToken`](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)、[`tempfile::NamedTempFile`](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html)を照合した。
- **CANDIDATES**: repo既存のtable-local `peek_next`／inverse-only `restore`とD2 inverse、global `StableIdReservation`、OCI型digest+size、Bazel型recipe→result metadataとartifact bytesの分離、repo既存のSQLite型atomic file commit、Qt型macro、Blender型unlink／orphan、Tokio型cooperative cancellation、same-directory temp→verify→persist。
- **ADOPTION ROUTE**: Assetは`LayerIdTable`とD2 `AddTrackItem`のtable-local復元先例を`REUSE/PATTERN`し、global `Document.next_stable_id`／`StableIdReservation`へ混ぜない。workspace `sha2`を`REUSE`、OCIとBazelのidentity分離を`PATTERN`、hard-budget K1a契約を`REUSE`、Tokio cancellationと`tempfile` publicationを`WRAP/PATTERN`とする。
- **REJECTED CANDIDATES**: path／mtime identity、head/tail hash authority、AssetIdによるcontent dedup、全tier共通store、global CAS必須化、universal public Job型、完了eventの永続event bus、通常修復としての全cache purge、M5 private型からの契約逆算。
- **THIN MOTOLII SEAM**: Motolii固有なのはD2 action、render-relevant recipe fields、resource owner名、Commandごとのdependency footprint、task固有artifact検証だけである。
- **THIN MOTOLII RESIDUAL**: SourceFingerprint／RecipeKey codec、Asset Command、ResourceLedger、invalidation classifierの小さいHost-private実装。
- **RETIREMENT**: legacy `head_hash`／`tail_hash`はidentity authorityから外す。raw `content_hash`は読み取り互換のため保持し、新しいpersistent cache／relink authorityだけをstrict V1へ移す。移行完了前にfieldを削除しない。
- **BUILD JUSTIFICATION**: `NONE`。新規一般frameworkは不要で、既存ownerへ薄い製品意味を接続する。
- **BUILD: FORBIDDEN**: 独自CAS／DB／WAL、第二writer、Asset manager framework、汎用scheduler、公開event bus、M5用先行resource API。

## 3. 契約A — Asset actionとlifecycle

### 3.1 四つのactionを分離する

Assetの取り込み、作品内での使用、台帳からの除去は一つの暗黙transactionにしない。

1. **probe**: workerがfileを読み、型、full content digest、size、必要なmedia metadataを求める。DocumentとIDを変更せず、結果はadmit authorityではないadvisory receiptとする。
2. **admit**: product adapterがlocatorを再openしてfull fingerprintを取り直し、probe receiptと一致する時だけ、edit threadが`AssetTable::peek_next`と同型のtable-local値から`AssetId`を決定して完全な`Asset`を一つのreplayable `AdmitAsset` Commandとしてjournal／applyする。probeはIDを予約しない。
3. **use**: admit成功で公開された`AssetId`を、既存のplacement／Soundtrack／typed `AssetRef` actionが使う。
4. **remove**: 全typed useが0の時だけ、完全な`Asset`値を保持する`RemoveAsset` Commandで台帳から外す。

file dropで「importしてTimelineへ置く」通常操作は、admitとplacementの二つのaccepted durable actionである。一回目のUndoはplacementだけを戻してunused Assetを残し、二回目のUndoがunused Assetを除去する。二action間のcrash／失敗は、参照切れや部分placementでなく、再利用または後でpurgeできるunused Assetを残す。

probe receiptのfingerprint／sizeとadmit直前のfull再読結果が違う場合は、product adapterがtyped `source-changed`としてDocument／journalを無変更で拒否する。Commandは完全な`Asset`値だけを受ける純粋なdurable境界に保ち、path再読やhashingをedit threadへ入れない。admit後に原本が変化する可能性は、§4.1のHost-private `SourceBinding`でsourceを実際に消費するjobごとに閉じる。

現行product durable routeは一accepted actionを一replayable Commandへ閉じているため、Qt型macroをこの境界の必須条件にしない。将来、一gesture一Undoのproduct要件が実測された場合は、durable macro自体を別decision／journal migrationとして閉じる。

### 3.2 ID、Undo、Redo

- `AssetId`はproject object identityであり、同じbytesを別名・別用途で二度admitしても別IDを許す。
- IDはworker、file path、hashから導出しない。edit threadが`AssetId::from_raw(doc.assets.peek_next())`と同型のtable-local採番値をCommandへ記録する。`AssetId`は`Document.next_stable_id`やglobal `StableIdReservation`の空間へ入れない。
- `AdmitAsset { asset }`と`RemoveAsset { asset }`をD2の相互inverseとする。`AssetTable`へ`peek_next`とinverse-only `restore`を追加し、journal replay／Redo／Undoの同一entity復元は`restore`を使う。`restore`はduplicateを拒否し、`id < next`の復元を許し、必要なら`next`を進めるが巻き戻さない。通常の`insert`は退役IDを引き続き拒否する。新しいdurable Command variantを現行journal Edit v2へ黙って混ぜず、1AでEdit formatをv3へ上げる。v3はAsset Commandだけの局所versionでなく、cutover後の`JournalEdit::FORMAT_VERSION`が全新規Editへ記録するwriter versionである。
- 未知Editを`InvalidEditPayload`へ潰してsnapshot fallback後もwritable `ProjectSession`を返す現行routeは、旧reader保護として採用しない。1Aは既存のtop-level `JOURNAL_FORMAT_VERSION`をreader gateとしてv2へ上げる。v1 containerへ最初のv3 Editをcommitする時は、exclusive `ProjectSession`下でv2 header、既存accepted frame bytes、最初のv3 frameを全て含むsibling tempを作り、project IDとgeneration saltを保持する。temp上で既存frameまでのreplayが旧WALと同値であり、最初のv3 frameまでのreplayがcandidate Documentと同値であることを検証してから、file fsync→atomic file replace→directory fsyncする。headerだけを先に置換してから最初のv3 frameを別appendする中間状態は作らない。新readerはcontainer v1/v2とEdit v1/v2/v3を読み、v2 containerへだけv3をappendする。旧readerは既存`read_header`のunsupported container versionでreplay前にhard refusalし、stale snapshotをwritable projectとして返さない。初回のmigration+v3 commit、または既存v2 containerへの後続v3 appendが失敗した時は、transaction開始前のWAL、Document、journalを無変更に保つ。atomic replace後は最初のv3 commit成功であり、process crash時も新readerがそのCommandをreplayする。Document schema fieldは増えないため、このCommand追加だけを理由にDocument version／`min_reader_version`を上げない。
- 新規admitのprepareは`asset.id == AssetId::from_raw(snapshot.assets.peek_next())`を記録し、journal commit直前にlive `peek_next`が同じ値か再照合する。不一致はtyped stale-admission errorとしてDocument／journalを無変更で拒否する。inverse-only `restore`は任意の新規ID採番口として公開せず、accepted Commandのjournal replay／Undo／Redoだけが使う。
- Commandはcomplete `Asset`値、duplicate、採番closure、全typed useを事前検証し、現行D2と同じclone→apply→Document validate→swapのatomic routeへ載せる。1Aはprivateな一つのexhaustive `AssetRef` traversalを新設し、`Document::validate`のdangling検査と`RemoveAsset`の参照0判定をその同じtraversalのconsumerにする。現行実装に共通visitorが既にあるとは扱わず、Soundtrack、Clip source、SVG／font等のdirect carrierと、Layer／Group、`ClipSource::Plugin.params`、orphanを含む`EffectDefinition.params`、`VectorRecipe`の`PathOp` parameter、Composition cameraにnestedした`DocValue::AssetRef`を同じ入口で列挙する。新しいtyped carrier追加時は、この共通traversalを更新しない限りcompileまたはvalidation corpusをfailさせる。
- `RemoveAsset { asset }`は参照0に加え、live tableの同じ`AssetId`にある完全な`Asset`値がCommand payloadと等しいことをremove直前に照合する。relink等でlocator／fingerprintを含むpayloadが変わっていた場合はtyped stale-asset mismatchとしてDocument／journalを無変更で拒否し、古い値を削除またはUndoで復元しない。これは既存lifecycle Commandのpayload mismatch guardと同じCAS形であり、global CASを導入する規則ではない。
- 1A cutover後、現行`AssetTable::allocate`をproductの新規Asset作成口として使わない。productはprobe済みの完全な`Asset`を持つ`AdmitAsset`だけを通り、`allocate`を互換／fixtureのため一時保持する場合もproduct callsiteを0にする。任意の生hashを受け取って台帳へ即時挿入する旧routeを新Commandの裏口にしない。
- crash recoveryはjournal済みCommandから最終Document意味を戻す。再起動後のUndo stack復元は現行契約外のままとする。

### 3.3 unlink、relink、replace、purge

- placement／Soundtrack／parameterからAsset useを外す操作は、Assetそのものの削除ではない。
- relinkは候補pathのbytesが保存済み`SourceFingerprintV1`と完全一致した時だけ、同じ`AssetId`のlocatorを更新する。locator変更だけではrender key／`CacheEpoch`を変えない。offlineからonlineへ戻る同一fingerprint relinkでは、Host-privateなsource availability／diagnosticとcurrent-state requestを更新してmissを再試行できるようにするが、既存artifactを無効化しない。ただしD2 Command variant、old/new locator、offline時failure、transient refreshのexact形はGAP-3の別cutで閉じ、1Aへ混ぜない。
- 異なるbytesへの差し替えはsilent relinkでなく、将来の明示的な`Replace Source` actionである。fingerprintを変え、全依存useを無効化する。
- `Purge Unused`は未参照Assetを列挙・確認して複数除去する将来のmaintenance actionである。今回、durable macroや自動purgeを作らない。
- source fileはユーザーまたは外部filesystem所有であり、Asset除去やcache purgeで原本を削除しない。

## 4. 契約B — source identityとderived artifact recipe

### 4.1 SourceFingerprintV1

新しくadmitするfile-backed Assetのcontent identityは、strict codecがexact source bytesから生成した次の両方が揃った時だけ`SourceFingerprintV1`として有効とする。

- `content_hash = "motolii-source-v1:sha256:<64 lowercase hex>"`
- `size_bytes = Some(<source file全byte長>)`

`motolii-source-v1`はalgorithm名ではなく、exact source bytesをhashしたことを表すformat／provenance tagである。hash対象はsource fileのexact bytesであり、decode後のpixel、container正規化、path、mtime、file nameではない。streaming SHA-256で全byteを読む。digestとsizeの両方をrelink／persistent cache hit前に照合する。

`SourceFingerprintV1`のV1はdecoderと意味のversionである。digest文字列のalgorithm tagは将来拡張を許すが、v1 Hostが知らないalgorithmは保持のみ可能で、検証、relink authority、persistent cache keyへ使わない。`head_hash`／`tail_hash`はlegacy hintとして保持できるが、identity、collision fallback、cache hitのauthorityにしない。

既存fixtureや旧projectにある`sha256:*`は64桁とsizeが揃っていてもV1 provenanceを持たず、短いhash、任意文字列、size欠落と同じ読み取り互換上のlegacy opaque valueである。strict codecによる明示的な再hash／確認でV1 tagを書き込むまで、persistent cacheはmiss、automatic relinkは不一致として扱う。文字列shapeだけでlegacy値をV1へ昇格しない。projectからloadしたV1-shaped tagも文字列だけではsession内のverified identity capabilityにせず、最初のpersistent-cache hit／relink／source-consuming job前にSourceBindingでexact bytesを再hashする。一致後だけそのsessionでverifiedとし、不一致またはofflineではcache aliasへhitせずmiss／typed source failureへ倒す。

sourceを実際に読むproxy、decode、analysis、importerはraw locatorをworker／sidecarへそのまま渡さず、Hostが作るprivate `SourceBinding { SourceFingerprintV1, size, read capability }`だけを入力にする。Bindingは次のいずれかで、workerが読むexact bytesとfingerprintを同じbyte sequenceへ結合する。

- Host-owned immutable tempへsourceをbounded chunkでcopyしながらfull hash／sizeを求め、保存済みfingerprintと一致した後にread-only capabilityだけをworkerへ渡す。Motolii-owned temp／snapshotは外部source fileと違ってResourceLedgerのdisk budget対象であり、同じfingerprintのbindingをread-onlyで複数jobへ再利用できる。binding capabilityを保持するqueued／running jobが一つでもある間はresident／pinnedとして予約とtempを保持し、最後のcapability drop後だけevictできる。copy前にsource全長を含む保守的disk reservationを取れない場合はtyped budget refusalでjobを開始せず、raw locatorへfallbackしない。これは大きいsourceで一時的に原本とsnapshotの二重disk使用を生む明示的portable costであり、editor threadをblockせず診断へrequired bytes／budgetを返す。evict後は再bindする。
- platformが提供するstable immutable handleを採る場合は、そのhandleの全bytesをhashした後もworkerの全readが同じbyte identityへ固定されることをtask固有の非LLM oracleで証明する。証明できないfilesystem／sidecar routeではこの候補を使わない。

保存済みfingerprintとの不一致、binding作成中のsize／digest変化、offline、budget refusalはtyped source failureとしてjobを開始せず、古いrecipeへのartifact publication、Document fingerprintの黙った更新、raw locatorへのfallbackを行わない。これによりadmit後に原本が変わっても、旧`SourceFingerprintV1`の下で新しいbytesを消費しない。

### 4.2 RecipeKeyV1

`RecipeKeyV1`はHost-private canonical encoderが、出力を変え得る意味だけを型付き・domain separated・versioned bytesへencodeし、そのbytesをSHA-256した値である。値はtag+length-prefixで連結し、整数幅／endianness、float canonicalization、string bytes、collection順を固定する。順不同mapはstable typed keyでsortし、意味またはencodingを変える時はrecipe format versionを上げる。

各recipeは対象artifactに必要な次を含む。

- render graph／node／pluginのidentity、semantic version、実行内容hash
- typed parameter値と、入力Assetの`SourceFingerprintV1`
- 意味を持つ入力順、対象時刻またはhalf-open区間、`Quality`
- `FrameDesc`、color／alpha／sample／cache artifact format、artifact bytesまたは意味を変える論理RoD／要求RoI寄与
- renderer、decoder、toolchain等、出力差が実測または仕様上あり得るenvironment salt。各artifact recipe schemaがexact fieldとversionを列挙し、未定義の環境blobや実装時の任意追加へ逃がさない

次は含めない。

- `AssetId`、path、file name、表示名
- UI route、選択、panel状態、JobId
- Document revision／`CacheEpoch`／`projection_generation`／`RenderGeneration`そのもの
- 出力を変えないmetadataや、無関係nodeの状態

完全keyは「同じkeyなら同じ意味のartifactを要求している」を保証する。`ArtifactDigest`はpublish対象のactual bytesを別にSHA-256したintegrity identityである。`RecipeKeyV1 -> verified result metadata + ArtifactDigest`とartifact bytesの分離だけをBazelから採り、global CAS、cross-project dedup、remote cacheは要求しない。

## 5. 契約C — resource、artifact、job

### 5.1 hard-budget admission

ResourceLedgerはHost内で一つのadmission policy ownerを持ち、VRAM、RAM、disk、shared memoryのhard capをallocation／write開始前に判定する。store、allocator、eviction、metric、lifetimeはtier別adapterに保ち、全resourceを一つの巨大cache／providerへ統合しない。

admission permitは最低でもowner、tier、予約上限bytes、purpose、resident／pinned区分を持つ。各allocationまたはbounded write chunkの**前**に、format／mip／sample／alignment／allocator overheadを含む保守的上限を予約し、adapterは実accounted bytesが予約上限を越えないことを保証する。上限を事前にboundedにできないallocation／writeはtyped refusalとし、permitなしの生成、生成後の追加admission、実測超過分の事後精算をhard-cap routeに許さない。実使用が予約未満なら未使用分を直ちに返し、drop／明示releaseで残りのaccountingを戻す。backend reportは不変条件違反の診断と以後のadmission停止に使えても、Host hard capや事前予約の代替ではない。unified memoryではRAMとVRAMの個別表示に加えshared aggregate capを守る。

区分は次の通りである。

- source file: 外部所有。cache budget外であり、Motoliiが自動削除しない。
- resident working set: GPU texture／buffer、decode surface、RAM working data。permitとlifetimeを必須にする。
- reconstructible artifact: proxy、analysis、bake、Draft、frame cache。hard-budgeted、evict可能、欠落／破損はmiss→再計算。
- durable user export: 明示destinationへcommitするユーザー成果。cache eviction対象外で、失敗／cancelをユーザーへ返す。
- transient preview／in-flight result: generation fence対象。stale完了はpublishせず破棄する。

### 5.2 artifact publication

reconstructible artifactとexportは、完成前のfileを有効成果物として公開しない。Hostがadmission済みのsame-directory tempとwrite capabilityをworkerへ貸し、workerはtask固有validatorでsize、decodeability、frame／duration等を先行確認する。candidate receiptを返す前に当該jobのwrite phaseを終了し、sidecar subprocessと全write handleをcloseしてtempの唯一の所有権をHostへ返す。receipt後のworker mutationを許さない。Hostのartifact ownerはcandidateを再openし、実際にcommitするexact bytesの長さ／`ArtifactDigest`とtask固有のpublication invariantを権威ある最終検証として再照合する。workerはcache catalogにもDocumentにも直接書かず、既存destinationを先に削除しない。

reconstructible artifactは、検証済みcandidateをproject／cache-localなimmutable objectとして先に耐久化し、その後`RecipeKeyV1 -> result metadata + ArtifactDigest`の小さいresult recordをunique temp→file sync→atomic replace→platformで可能なdirectory syncでcommitする。**このresult recordのatomic replaceをpublication commit point**とし、in-memory catalogはcommit済みrecordから再構築できるreaderにする。commit前のcrashは旧recordを残し、commit後のrecordは既に耐久化済みobjectだけを参照する。同一recipeの競合はHost ownerがcommit pointで直列化し、loserを未公開物として回収する。これはglobal CAS、cross-project dedup、remote cacheを作る規則ではない。

durable exportはcatalogへ登録せず、検証済みsame-directory tempの明示destinationへのatomic replaceをcommit pointとする。commit前のfailure／cancel／crashでは旧finalが残り、commit後のcrashでは完全な新finalが残るため、保証は「完全な旧または完全な新」であってprocessを越えたjob成功通知ではない。old-or-new完全性を保証できないplatform／filesystemではcommit前にtyped failureとし、旧成果を維持する。stale tempと未参照immutable objectは再起動時に未公開物として回収でき、result record欠落、object欠落、digest不一致、decode失敗はすべてmissへ縮退する。

### 5.3 job終端

共通化するのは語彙と不変条件だけで、全taskを一つの公開`Job`型へ押し込まない。

許される遷移を次へ固定する。

- `queued -> running | failed(admission/start cause) | cancelled`
- `running -> candidate-ready(receipt) | failed(worker/validator cause) | cancelled`
- `candidate-ready -> succeeded(published receipt) | failed(publication cause) | cancelled`

- cancelはcooperativeで、boundedなtile／frame／chunk境界にcheckpointを置く。
- Host ownerだけがterminal transitionを一度だけ確定する。cancelがpublication commit pointより先に受理された時は`cancelled`としてpublishせず、commit point後のcancelは既に`succeeded`したjobを上書きしない。`failed`／`cancelled`後のlate receiptは破棄し、temp／permitを回収する。
- worker completionはcandidate-readyにすぎず、Host ownerのpublication成功だけをjob successとする。failed／cancelledはpartial resultを有効化しない。
- `JobId`はHost owner起動時のprivate process/session nonceと単調増加counterから発行し、owner寿命中は再利用しない。counter overflowはtyped refusalとし、restartを越えて永続化せずrecipeへ入れない。candidate receiptは`JobId`、開始時state envelope identity、必要な`RecipeKeyV1`、target／publication policyへ結合し、全てがlive job recordと一致する時だけ遷移に使う。unknown、terminal済み、別target、旧ownerのreceiptはlate receiptとして破棄・回収する。
- jobは開始時に一つのstate envelopeからimmutable snapshot、`CacheEpoch`、必要な`RecipeKeyV1`、publication policyを同時取得する。current editor／previewへ出すtransient resultは開始時epochとcurrent envelopeのepoch一致を必須とする。完全key付きreconstructible artifactは、旧snapshotのkeyの下へだけpublishでき、current stateのaliasへstale rebindしない。durable exportは開始時snapshotを明示destinationへ出す操作であり、後続Document editだけを理由にstale／cancelへしない。
- M4 cache/session ownerは非永続のprivate `CacheEpoch { session_identity, counter }`をstate envelope内に所有する。`session_identity`はHost-privateなref-counted allocation tokenで、値やpointer addressを公開／serializeせずallocation identityだけで比較する。envelopeとjob receiptがtoken cloneを保持するため、old identityが観測可能な間にallocator addressを再利用しても同じidentityにはならず、数値nonceの衝突／exhaustionを持たない。counterはchecked incrementする。counter exhaustion前にはold envelopeを保持可能なままfresh token／counter 0／whole-composition invalidationを一つのatomic state replacementとしてinstallし、次のDocument editを失敗させたり旧epochと衝突させたりしない。Document edit threadはCommandのclone→apply→validateで得たcandidate snapshotをpublic publishする前にexhaustive classifierへ渡し、render-relevant／unknownなら次epoch、metadata-onlyまたはunused Assetのadmit／removeなら現epochを選ぶ。journal acceptance後、`(immutable snapshot, CacheEpoch, InvalidationFootprint)`を一回のserialized atomic transactionでpublishする。classifier errorはunknown／whole-compositionへ倒し、snapshotだけを先に公開しない。
- `CacheEpoch`はUI選択でも進み得るM3 `DocumentEditRuntime.projection_generation`、render要求順のprivate `RenderGeneration`、journal generation、Document revisionの別名ではない。selection-only publishはcache epochを進めず、値をrecipeやDocumentへserializeしない。
- editor、Preview、通常操作はproxy／analysis／cache jobを待たない。
- capacity pressureとrender deadline超過は別signal、別policyにする。
- durable exportのfailure／cancelと、透明にmissへ戻せるcache jobのfailureを同じUXへ潰さない。

## 6. 契約D — Document mutationからcache invalidationへのpublication

cache correctnessは二層に分ける。

1. **完全な`RecipeKeyV1`**: stale artifactへhitしないための正しさ。
2. **InvalidationFootprint**: 不要な再計算を減らす効率化。過大無効化は許せるが過小無効化は許さない。

実装ownerは`M4-P02-C3`ただ一つとする。K1bは完全key付きcache store、K2はaccepted target／source意味へのrender接続consumerであり、classifierまたはstate envelopeをそれぞれ再実装しない。P06-C2はM4-P02-C3が既に分類したaffected identityとdeclared temporal footprintをhalf-open区間へ写すprivate pure projection helperであり、`Command` match、known／unknown判定、epoch、publicationを所有しない。P06-C3はaccepted state envelopeの`InvalidationFootprint`をK1bのcoverageへ反映するconsumerであり、そのcoverage generationはstore-local identityであって`CacheEpoch`の別名でも第二state publishでもない。

Document edit threadはCommandをcloneへapply／validateした後、public state publish前にそのCommandとold／candidate snapshotをHost-privateなexhaustive classifierへ渡す。classifierは最低でもrender relevance、stable affected identity、half-open affected interval、known／unknownを返す。journal acceptanceが成立したら、M4 cache/session ownerが`(immutable snapshot, CacheEpoch, InvalidationFootprint)`のprivate state envelopeを一回のserialized atomic transactionでpublishする。readerとjobはこのenvelopeを一回だけ取得し、snapshot、epoch、footprintを別々に読まない。公開event bus、serialized invalidation event、第二writerは作らない。

- 全`Command` variantを明示matchし、新variant追加時は分類を追加しない限りcompileまたはtestをfailさせる。
- dependencyや時間窓が不明、将来variant、classifier errorの場合はwhole-composition invalidationへ倒す。
- pixelを変え得るmutationは、affected recipeまたはそのtyped dependency identityを必ず変える。
- display name、同一fingerprintへのlocator更新等のmetadata-only mutationはrecipeを変えない。
- unused Assetのadmit／removeはrender invalidation不要。Asset useの追加／削除は配置区間、source replacementは全依存区間を無効化する。
- render-relevant／unknown分類ではpublishするenvelopeの`CacheEpoch`を進め、metadata-only分類では維持する。新snapshot／旧epochまたは旧snapshot／新epochが観測できる中間状態を作らない。epoch不一致のcurrent-state transient resultはpublishせず、完全`RecipeKeyV1`付きreconstructible artifactは旧keyの下へだけpublishできる。旧envelope readerは保持済みhandleを読み切ってよい。`CacheEpoch`はpersistent recipe keyへ含めない。
- 通常のcorrectness回復として全cache purgeを使わない。unknown fallbackとしての全composition再計算と、disk全削除は別操作である。

## 7. 失敗しても捨て直しにならない分割

4契約は一つの巨大sandbox実装にしない。各cutは前cutのpublic／internal contractだけを読み、独立oracleで採否できる。

| 順序 | closed order | primary oracle | 並列解放 |
|---|---|---|---|
| 1A | M2 Asset: strict fingerprint codec、`AdmitAsset`／`RemoveAsset`、table-local `peek_next`／inverse-only `restore`／typed use count | V1 provenance roundtrip、shape-only／未再検証loaded tagのhit拒否、live `peek_next`変化時の無変更拒否、atomic apply、Undo remove→Redo same ID、stale `RemoveAsset` payload mismatch時の無変更拒否、通常`insert`のretired拒否、inverse `restore`のretired同一entity許可、`Document::validate`とremoveが共有する一つのtyped traversalによる参照中remove拒否、journal container v1→v2 migrationと最初のv3 frameを一つのatomic replaceへ含めたreplay同値／失敗時transaction前WAL無変更、v2 containerでのglobal Edit v3 roundtrip、new readerのcontainer v1/v2・Edit v1/v2/v3互換、旧readerのheader段hard refusalとsnapshot fallback 0、product `AssetTable::allocate` callsite 0 | M2-ASSET-1C product adapterとM4 identity fixture |
| 1B | M4 K1a: thin ResourceLedger policy ownerと注入budget | cap不超過、全pin typed refusal、SourceBindingのfull-copy事前予約／job中pin／最終drop後release、release後0、shared aggregate | P04-C1 estimatorと、K1a policyを呼ぶだけのP04-C2 tier adapter、GPU／RAM／disk adapter |
| 1C | product Asset admit／source binding adapter | stale probe時Document／journal無変更、binding exact bytesのfingerprint一致、offline／変化／budget refusal時job開始0、`motolii-doc::resolve_asset_path`を直接使う現行`motolii-audio::AudioProgram`／`motolii-export` source収集と、その後のmedia decode／probe／muxを含むsource-consuming callsite inventoryでraw locator worker handoff 0 | K4とsource-consuming task adapter |
| 2 | M4-P02-CODEC: canonical RecipeKey／ArtifactDigest codec | mutation corpus、order／float encoding、rename不変、content／plugin／Quality／RoD・RoI／列挙済みenvironment salt変化 | K1b、artifact recipe |
| 3A | M4-P02-C3: exhaustive Command classifierとatomic state envelopeの唯一のowner | 全Command分類、unknown全域、影響区間、新snapshot／旧epoch観測0、単一envelope capture、epoch session renewal衝突0 | K2、P07-C2、K7/K8 |
| 3B | K1b: 完全key付きcache同一性／並行store | key mutation corpus、参照handle、evict／invalidator／reader並行stress | tier adapter、K2 |
| 3C | K2: K1b、M4-P02-C3、P06-C2/C3のrender統合 | accepted target／source Command意味の依存伝播、部分無効化、全cache purge 0、第二classifier／epoch publisher 0 | K7b、K8a |
| 3D | M4-P03-C3: `AudioProgram` product RAM adapter | M2-ASSET-1CのSourceBindingとP03-C2の共通RAM admissionだけを使い、旧caller-owned無制限HashMap 0、audio clock block 0、PCM同値、budget accounting | K8b通し再生E2E |
| 3E | M4-P07-C3: CPU／GPU cooperative cancel adapter | P07-C2のJobId／terminal ownerを再利用し、bounded checkpoint latency、heartbeat timeout時typed terminal、GPU command途中強制停止0、cancel後publish 0 | P09-C2、K8a。media branchはP08-C1へ吸収 |
| 4 | task別job／atomic artifact adapter | 全failure／cancel遷移、owner寿命中JobId再利用0、mismatched／terminal済みlate receipt、receipt後worker mutation 0、digest再検証、cancel／kill／ENOSPC／corrupt／raceでcommit前は旧、commit後は完全な新 | K4 proxy、K7 bake、analysis、export adapter |

1Aと1Bはownerとfile allowlistを分けて直ちに並列化できる。1Cは1AとK1a後にproduct admit／budgeted SourceBindingだけを閉じる。M4-P02-CODECは1Aのstrict fingerprint typeを入力に使うが、1Aのallowlistと重ならないcanonical encoder／mutation corpusの準備は並行できる。M4-P02-C3だけがclassifier／state envelopeを所有し、M2-D3、M2-D3e、対象Command／source意味のacceptance後に入る。P06-C2はM4-P02-C3が固定した分類出力を受けるpure projection、P06-C3はM4-P02-C3、P06-C2、K1b後のcoverage consumerとして進め、どちらも第二classifier／epoch publisherにしない。K1bはK1a、M4-P02-CODEC、M2-D8の全acceptance後に別ownerで進められ、classifierを所有しない。K2はK1b、M4-P02-C3、P06-C2/C3をaccepted target／source意味へ接続する。K1cはK1a、K1b、P03-C2、P04-C2、P05-C2、P05-C3の全acceptance後にだけtierを合流する。M4-P03-C3はM2-ASSET-1CとP03-C2後に旧audio HashMapをproduct routeから退役させ、K1cと並列に進められるが、K8b通し再生E2Eはそのacceptanceを待つ。M4-P07-C3のmedia cancel branchはP08-C1の既存kill/cancel oracleへ吸収し、別frameworkを作らない。残るCPU／GPU token・heartbeat adapterはP07-C2後に入り、P09-C2とK8aの前提になる。P09-C2はP09-C1の原因分離採択、K1a、P04-C1/C2、P05-C2/C3、P07-C2、M4-P07-C3後に入る。K7aはK1b、K1c、M2-D3、P07-C2、P09-C2後に入り、M4-P07-C3をP09-C2から推移的に受ける。K4はP08-C3 product substitutionそのものであり、第二のproduct decode merge ownerではない。M2-D1、M2-ASSET-1C、M4-P02-CODEC、P05-C2/C3、P07-C2、P08-C1/C2のacceptance後に入る。K8aはK1b、K1c、K1d、K2、M2-D3、P06-C2/C3、P07-C2、M4-P07-C3後に入る。他のtask別artifact adapterも該当source binding、tier admission、recipe、publication、job contractの各acceptance後に並列化できる。

M5はこの分割のconsumerだが、[M5休止・M3意味開放契約](2026-08-02-m5-pause-until-m3-semantic-release.md)を維持する。M5 private fixtureは契約oracleの候補になれても、M3意味開放前に製品runtime、schema、provider、GPU resource接続を開始しない。

## 8. acceptanceと停止線

このdecisionの完了は、4契約と順序を正本へ回収したことだけを意味する。次は含まない。

- Asset Command、ResourceLedger、RecipeKey、cache store、job runnerの実装完了
- M4 K1a／K1b／K2／K4のacceptanceまたはmain統合
- M5製品runtimeの解放
- global CAS、remote cache、cross-project dedup、package／install identity
- `Replace Source`、一括relink、Purge Unused UI、durable macro

各実装cutは、exact owner／allowlist／positive and negative oracleをcurrent mainから再compileして一ticket一commitで発注する。既存型またはconsumerが不足する場合、そのedgeだけを`RESEARCH_RETURN`し、private substituteを恒久契約へ昇格させない。
